//! mini_eval: Minimal tree-walking evaluator for bootstrapping IRIS.
//!
//! This is the irreducible Rust substrate — just enough to run
//! self_interpreter.json, which handles all other evaluation in IRIS.
//!
//! ~500 LOC. No JIT, no flat evaluator, no optimization.
//! Pure recursive tree-walk + primitive dispatch.

use std::collections::BTreeMap;
use std::rc::Rc;

use iris_types::eval::Value;
use iris_types::fragment::FragmentId;
use iris_types::graph::{
    BinderId, Edge, EdgeLabel, NodeId, NodePayload, SemanticGraph,
};

/// Maximum evaluation steps before timeout.
const MAX_STEPS: u64 = 100_000_000;

/// Evaluation error.
#[derive(Debug)]
pub enum EvalError {
    TypeError(String),
    MissingNode(NodeId),
    MissingEdge { source: NodeId, port: u8 },
    Timeout,
    DivisionByZero,
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EvalError::TypeError(s) => write!(f, "type error: {}", s),
            EvalError::MissingNode(n) => write!(f, "missing node: {}", n.0),
            EvalError::MissingEdge { source, port } => write!(f, "missing edge: {}:{}", source.0, port),
            EvalError::Timeout => write!(f, "step limit exceeded"),
            EvalError::DivisionByZero => write!(f, "division by zero"),
        }
    }
}

/// Evaluation context.
struct Ctx<'a> {
    graph: &'a SemanticGraph,
    env: BTreeMap<BinderId, Value>,
    edges_from: BTreeMap<NodeId, Vec<&'a Edge>>,
    registry: &'a BTreeMap<FragmentId, SemanticGraph>,
    steps: u64,
    max_steps: u64,
}

impl<'a> Ctx<'a> {
    fn new(graph: &'a SemanticGraph, inputs: &[Value], registry: &'a BTreeMap<FragmentId, SemanticGraph>) -> Self {
        let mut edges_from: BTreeMap<NodeId, Vec<&Edge>> = BTreeMap::new();
        for edge in &graph.edges {
            edges_from.entry(edge.source).or_default().push(edge);
        }
        for edges in edges_from.values_mut() {
            edges.sort_by_key(|e| (e.port, e.label as u8));
        }
        let mut env = BTreeMap::new();
        for (i, val) in inputs.iter().enumerate() {
            env.insert(BinderId(0xFFFF_0000 + i as u32), val.clone());
        }
        Ctx { graph, env, edges_from, registry, steps: 0, max_steps: MAX_STEPS }
    }

    fn arg_targets(&self, node_id: NodeId) -> Vec<NodeId> {
        self.edges_from.get(&node_id)
            .map(|edges| edges.iter()
                .filter(|e| e.label == EdgeLabel::Argument)
                .map(|e| e.target)
                .collect())
            .unwrap_or_default()
    }

    fn cont_target(&self, node_id: NodeId) -> Option<NodeId> {
        self.edges_from.get(&node_id)
            .and_then(|edges| edges.iter()
                .find(|e| e.label == EdgeLabel::Continuation)
                .map(|e| e.target))
    }

    fn bind_target(&self, node_id: NodeId) -> Option<NodeId> {
        self.edges_from.get(&node_id)
            .and_then(|edges| edges.iter()
                .find(|e| e.label == EdgeLabel::Binding)
                .map(|e| e.target))
    }

    fn eval(&mut self, node_id: NodeId) -> Result<Value, EvalError> {
        self.steps += 1;
        if self.steps > self.max_steps {
            return Err(EvalError::Timeout);
        }

        let node = self.graph.nodes.get(&node_id)
            .ok_or(EvalError::MissingNode(node_id))?;

        match &node.payload {
            NodePayload::Lit { type_tag, value } => self.eval_lit(*type_tag, value),
            NodePayload::Prim { opcode } => self.eval_prim(node_id, *opcode),
            NodePayload::Guard { predicate_node, body_node, fallback_node } => {
                let pred = self.eval(*predicate_node)?;
                let take_body = match &pred {
                    Value::Bool(b) => *b,
                    Value::Int(n) => *n != 0,
                    _ => false,
                };
                if take_body { self.eval(*body_node) } else { self.eval(*fallback_node) }
            }
            NodePayload::Tuple => {
                let targets = self.arg_targets(node_id);
                let mut elems = Vec::with_capacity(targets.len());
                for t in targets {
                    elems.push(self.eval(t)?);
                }
                Ok(Value::tuple(elems))
            }
            NodePayload::Project { field_index } => {
                let targets = self.arg_targets(node_id);
                let src = targets.first().ok_or(EvalError::MissingEdge { source: node_id, port: 0 })?;
                let val = self.eval(*src)?;
                match val {
                    Value::Tuple(elems) => {
                        let fi = *field_index as usize;
                        elems.get(fi).cloned().ok_or_else(|| EvalError::TypeError(
                            format!("project: index {} out of range (len {})", fi, elems.len())
                        ))
                    }
                    Value::Range(s, e) => {
                        let fi = *field_index as i64;
                        if s + fi < e { Ok(Value::Int(s + fi)) }
                        else { Err(EvalError::TypeError("project: out of range".into())) }
                    }
                    _ => Err(EvalError::TypeError(format!("project: field {} on {:?} (node {}, src {})",
                        *field_index, val, node_id.0, targets.first().map_or(0, |n| n.0))))
                }
            }
            NodePayload::Lambda { binder, .. } => {
                let body = self.cont_target(node_id)
                    .or_else(|| self.arg_targets(node_id).first().copied())
                    .ok_or(EvalError::MissingEdge { source: node_id, port: 0 })?;
                Ok(Value::tuple(vec![
                    Value::Int(binder.0 as i64),
                    Value::Int(body.0 as i64),
                    Value::tuple(self.env.iter().map(|(k, v)| Value::tuple(vec![Value::Int(k.0 as i64), v.clone()])).collect()),
                ]))
            }
            NodePayload::Apply => self.eval_apply(node_id),
            NodePayload::Let => self.eval_let(node_id),
            NodePayload::Fold { .. } => self.eval_fold(node_id),
            NodePayload::Ref { fragment_id } => self.eval_ref(node_id, fragment_id),
            NodePayload::Inject { tag_index } => {
                let targets = self.arg_targets(node_id);
                let payload = if let Some(&t) = targets.first() {
                    self.eval(t)?
                } else {
                    Value::Unit
                };
                Ok(Value::tuple(vec![Value::Int(*tag_index as i64), payload]))
            }
            NodePayload::Rewrite { body, .. } => self.eval(*body),
            _ => Err(EvalError::TypeError(format!("unsupported node kind: {:?}", node.kind)))
        }
    }

    fn eval_lit(&self, type_tag: u8, value: &[u8]) -> Result<Value, EvalError> {
        match type_tag {
            0x00 if value.len() == 8 => {
                Ok(Value::Int(i64::from_le_bytes(value[..8].try_into().unwrap())))
            }
            0x02 if value.len() == 8 => {
                Ok(Value::Float64(f64::from_le_bytes(value[..8].try_into().unwrap())))
            }
            0x04 if value.len() == 1 => Ok(Value::Bool(value[0] != 0)),
            0x06 => Ok(Value::Unit),
            0x07 => Ok(Value::String(String::from_utf8_lossy(value).into_owned())),
            0xFF if !value.is_empty() => {
                let index = value[0] as u32;
                let binder = BinderId(0xFFFF_0000 + index);
                Ok(self.env.get(&binder).cloned().unwrap_or(Value::Unit))
            }
            _ => Ok(Value::Unit),
        }
    }

    fn eval_prim(&mut self, node_id: NodeId, opcode: u8) -> Result<Value, EvalError> {
        let targets = self.arg_targets(node_id);
        let mut args = Vec::with_capacity(targets.len());
        for t in &targets {
            args.push(self.eval(*t)?);
        }
        self.dispatch_prim(opcode, &args)
    }

    fn dispatch_prim(&mut self, opcode: u8, args: &[Value]) -> Result<Value, EvalError> {
        let a = args.first().cloned().unwrap_or(Value::Unit);
        let b = args.get(1).cloned().unwrap_or(Value::Unit);
        let c = args.get(2).cloned().unwrap_or(Value::Unit);

        match opcode {
            // Arithmetic
            0x00 => Ok(match (&a, &b) {
                (Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_add(*y)),
                (Value::Float64(x), Value::Float64(y)) => Value::Float64(x + y),
                (Value::Float64(x), Value::Int(y)) => Value::Float64(x + *y as f64),
                (Value::Int(x), Value::Float64(y)) => Value::Float64(*x as f64 + y),
                (Value::String(x), Value::String(y)) => Value::String(format!("{}{}", x, y)),
                _ => Value::Unit,
            }),
            0x01 => Ok(match (&a, &b) {
                (Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_sub(*y)),
                (Value::Float64(x), Value::Float64(y)) => Value::Float64(x - y),
                _ => Value::Unit,
            }),
            0x02 => Ok(match (&a, &b) {
                (Value::Int(x), Value::Int(y)) => Value::Int(x.wrapping_mul(*y)),
                (Value::Float64(x), Value::Float64(y)) => Value::Float64(x * y),
                _ => Value::Unit,
            }),
            0x03 => match (&a, &b) {
                (Value::Int(x), Value::Int(y)) => {
                    if *y == 0 { Err(EvalError::DivisionByZero) }
                    else { Ok(Value::Int(x / y)) }
                }
                (Value::Float64(x), Value::Float64(y)) => Ok(Value::Float64(x / y)),
                _ => Ok(Value::Unit),
            },
            0x04 => match (&a, &b) { // mod
                (Value::Int(x), Value::Int(y)) => {
                    if *y == 0 { Err(EvalError::DivisionByZero) }
                    else { Ok(Value::Int(x % y)) }
                }
                _ => Ok(Value::Unit),
            },
            0x05 => Ok(match &a { Value::Int(x) => Value::Int(-x), _ => Value::Unit }), // neg
            0x06 => Ok(match &a { Value::Int(x) => Value::Int(x.abs()), _ => Value::Unit }), // abs
            0x07 => Ok(match (&a, &b) { // min
                (Value::Int(x), Value::Int(y)) => Value::Int((*x).min(*y)),
                _ => Value::Unit,
            }),
            0x08 => Ok(match (&a, &b) { // max
                (Value::Int(x), Value::Int(y)) => Value::Int(*x.max(y)),
                _ => Value::Unit,
            }),
            // Bitwise
            0x10 => Ok(match (&a, &b) { (Value::Int(x), Value::Int(y)) => Value::Int(x & y), _ => Value::Unit }),
            0x11 => Ok(match (&a, &b) { (Value::Int(x), Value::Int(y)) => Value::Int(x | y), _ => Value::Unit }),
            0x12 => Ok(match (&a, &b) { (Value::Int(x), Value::Int(y)) => Value::Int(x ^ y), _ => Value::Unit }),
            // Comparison
            0x20 => Ok(Value::Bool(a == b)),
            0x21 => Ok(Value::Bool(a != b)),
            0x22 => Ok(match (&a, &b) { (Value::Int(x), Value::Int(y)) => Value::Bool(x < y), _ => Value::Bool(false) }),
            0x23 => Ok(match (&a, &b) { (Value::Int(x), Value::Int(y)) => Value::Bool(x > y), _ => Value::Bool(false) }),
            0x24 => Ok(match (&a, &b) { (Value::Int(x), Value::Int(y)) => Value::Bool(x <= y), _ => Value::Bool(false) }),
            0x25 => Ok(match (&a, &b) { (Value::Int(x), Value::Int(y)) => Value::Bool(x >= y), _ => Value::Bool(false) }),
            // Conversion
            0x40 => Ok(match &a { Value::Int(x) => Value::Float64(*x as f64), _ => Value::Unit }),
            0x41 => Ok(match &a { Value::Float64(x) => Value::Int(*x as i64), _ => Value::Unit }),
            // Self graph
            0x80 => Ok(Value::Program(Rc::new(self.graph.clone()))),
            // Graph introspection
            0x81 => self.prim_graph_nodes(&a),
            0x82 => self.prim_graph_get_kind(&a, &b),
            0x83 => self.prim_graph_get_prim_op(&a, &b),
            0x89 => self.prim_graph_eval(args),
            0x8A => self.prim_graph_get_root(&a),
            0x66 => self.prim_graph_get_lit_type_tag(&a, &b),
            0x8E => self.prim_graph_get_lit_value(&a, &b),
            0x8F => self.prim_graph_outgoing(&a, &b),
            0xEE => self.prim_graph_set_root(&a, &b),
            0x96 => self.prim_graph_edge_count(&a, &b),
            0x97 => self.prim_graph_edge_target(args),
            0x98 => self.prim_graph_get_binder(&a, &b),
            0x9A => self.prim_graph_get_tag(&a, &b),
            0x9B => self.prim_graph_get_field_index(&a, &b),
            0x9F => self.prim_graph_get_effect_tag(&a, &b),
            // graph_eval_ref
            0xA2 => self.prim_graph_eval_ref(args),
            // String ops
            0xB0 => Ok(match &a { Value::String(s) => Value::Int(s.len() as i64), _ => Value::Unit }),
            0xB1 => Ok(match (&a, &b) { // str_concat
                (Value::String(x), Value::String(y)) => Value::String(format!("{}{}", x, y)),
                _ => Value::Unit,
            }),
            0xB2 => Ok(match (&a, &b, &c) { // str_slice
                (Value::String(s), Value::Int(start), Value::Int(end)) => {
                    let s = s.as_str();
                    let start = (*start).max(0) as usize;
                    let end = (*end).max(0) as usize;
                    Value::String(s.get(start..end.min(s.len())).unwrap_or("").to_string())
                }
                _ => Value::Unit,
            }),
            0xB3 => Ok(match (&a, &b) { // str_contains
                (Value::String(s), Value::String(pat)) => Value::Bool(s.contains(pat.as_str())),
                _ => Value::Bool(false),
            }),
            0xB4 => Ok(match (&a, &b) { // str_split
                (Value::String(s), Value::String(sep)) => {
                    let parts: Vec<Value> = s.split(sep.as_str()).map(|p| Value::String(p.to_string())).collect();
                    Value::tuple(parts)
                }
                _ => Value::Unit,
            }),
            0xB5 => Ok(match (&a, &b) { // str_join
                (Value::Tuple(parts), Value::String(sep)) => {
                    let strs: Vec<&str> = parts.iter().filter_map(|v| match v { Value::String(s) => Some(s.as_str()), _ => None }).collect();
                    Value::String(strs.join(sep.as_str()))
                }
                _ => Value::Unit,
            }),
            0xBB => Ok(match (&a, &b, &c) { // str_replace
                (Value::String(s), Value::String(from), Value::String(to)) => Value::String(s.replace(from.as_str(), to.as_str())),
                _ => Value::Unit,
            }),
            0xBF => Ok(match &a { // str_chars
                Value::String(s) => Value::tuple(s.chars().map(|c| Value::Int(c as i64)).collect()),
                _ => Value::Unit,
            }),
            // char_at
            0xC0 => Ok(match (&a, &b) {
                (Value::String(s), Value::Int(i)) => {
                    s.as_bytes().get(*i as usize).map(|&b| Value::Int(b as i64)).unwrap_or(Value::Int(-1))
                }
                _ => Value::Unit,
            }),
            // Collection ops
            0xC1 => { // list_append
                let mut elems: Vec<Value> = match &a {
                    Value::Tuple(t) => t.as_ref().clone(),
                    Value::Unit => vec![],
                    Value::Range(s, e) => if *e > *s { (*s..*e).map(Value::Int).collect() } else { vec![] },
                    _ => return Err(EvalError::TypeError("list_append: expected Tuple".into())),
                };
                elems.push(b);
                Ok(Value::tuple(elems))
            }
            0xC2 => Ok(match (&a, &b) { // list_nth
                (Value::Tuple(t), Value::Int(i)) => t.get(*i as usize).cloned().unwrap_or(Value::Unit),
                (Value::Range(s, e), Value::Int(i)) => if s + i < *e { Value::Int(s + i) } else { Value::Unit },
                _ => Value::Unit,
            }),
            0xC7 => Ok(match (&a, &b) { // list_range
                (Value::Int(s), Value::Int(e)) => if *e <= *s { Value::Range(0, 0) } else { Value::Range(*s, *e) },
                _ => Value::Unit,
            }),
            0xC8 => Ok(match (&a, &b, &c) { // map_insert
                _ => Value::Unit, // TODO: proper map support
            }),
            // tuple_get, tuple_len
            0xD2 => Ok(match (&a, &b) { // tuple_get
                (Value::Tuple(t), Value::Int(i)) => t.get(*i as usize).cloned().unwrap_or(Value::Unit),
                _ => Value::Unit,
            }),
            0xD6 => Ok(match &a { // tuple_len
                Value::Tuple(t) => Value::Int(t.len() as i64),
                Value::Unit => Value::Int(0),
                _ => Value::Int(0),
            }),
            // Math
            0xD8 => Ok(match &a { // sqrt
                Value::Float64(x) => Value::Float64(x.sqrt()),
                Value::Int(x) => Value::Float64((*x as f64).sqrt()),
                _ => Value::Unit,
            }),
            // list_len
            0xF0 => Ok(match &a {
                Value::Tuple(t) => Value::Int(t.len() as i64),
                Value::Range(s, e) => Value::Int(if *e > *s { *e - *s } else { 0 }),
                Value::String(s) => Value::Int(s.len() as i64),
                Value::Unit => Value::Int(0),
                _ => Err(EvalError::TypeError(format!("list_len: expected collection, got {:?}", a)))?,
            }),
            // perform_effect
            0xA1 => {
                // perform_effect(tag, args) — for now, only handle Print (tag 0)
                match &a {
                    Value::Int(0) => { eprintln!("{:?}", b); Ok(Value::Unit) }
                    _ => Ok(Value::Unit), // Silently ignore unsupported effects
                }
            }
            // value_make_tagged
            0x9E => Ok(Value::tuple(vec![a, b])),
            // --- Graph mutation primitives (used by the IRIS lowerer) ---
            // graph_new: create empty graph with default Prim(add) root
            0xED => {
                use std::collections::HashMap;
                use iris_types::cost::CostTerm;
                use iris_types::hash::compute_node_id;
                use iris_types::types::TypeId;
                let mut nodes = HashMap::new();
                let node = iris_types::graph::Node {
                    id: NodeId(0), kind: iris_types::graph::NodeKind::Prim,
                    type_sig: TypeId(0), cost: CostTerm::Unit, arity: 0,
                    resolution_depth: 0, salt: 0,
                    payload: NodePayload::Prim { opcode: 0 },
                };
                let id = compute_node_id(&node);
                nodes.insert(id, iris_types::graph::Node { id, ..node });
                Ok(Value::Program(Rc::new(SemanticGraph {
                    root: id, nodes: nodes.into_iter().collect(),
                    edges: vec![], type_env: iris_types::types::TypeEnv { types: BTreeMap::new() },
                    cost: iris_types::cost::CostBound::Unknown, resolution: iris_types::graph::Resolution::Implementation,
                    hash: iris_types::hash::SemanticHash([0u8; 32]),
                })))
            }
            // graph_add_node_rt(program, kind_int) -> (new_program, node_id)
            0x85 => {
                use iris_types::cost::CostTerm;
                use iris_types::hash::compute_node_id;
                use iris_types::types::TypeId;
                let mut graph = match &a { Value::Program(g) => g.as_ref().clone(), _ => return Err(EvalError::TypeError("expected Program".into())) };
                let kind_u8 = match &b { Value::Int(n) => *n as u8, _ => return Err(EvalError::TypeError("expected Int".into())) };
                let type_sig = graph.type_env.types.keys().next().copied().unwrap_or(TypeId(0));
                let salt = graph.nodes.len() as u64 + 1;
                let kind = match kind_u8 {
                    0 => iris_types::graph::NodeKind::Prim,
                    1 => iris_types::graph::NodeKind::Apply,
                    2 => iris_types::graph::NodeKind::Lambda,
                    3 => iris_types::graph::NodeKind::Let,
                    5 => iris_types::graph::NodeKind::Lit,
                    8 => iris_types::graph::NodeKind::Fold,
                    11 => iris_types::graph::NodeKind::Tuple,
                    12 => iris_types::graph::NodeKind::Inject,
                    13 => iris_types::graph::NodeKind::Project,
                    17 => iris_types::graph::NodeKind::Guard,
                    _ => iris_types::graph::NodeKind::Prim,
                };
                let payload = match kind {
                    iris_types::graph::NodeKind::Prim => NodePayload::Prim { opcode: 0 },
                    iris_types::graph::NodeKind::Lit => NodePayload::Lit { type_tag: 0, value: vec![0; 8] },
                    iris_types::graph::NodeKind::Tuple => NodePayload::Tuple,
                    iris_types::graph::NodeKind::Project => NodePayload::Project { field_index: 0 },
                    iris_types::graph::NodeKind::Inject => NodePayload::Inject { tag_index: 0 },
                    iris_types::graph::NodeKind::Guard => NodePayload::Guard { predicate_node: NodeId(0), body_node: NodeId(0), fallback_node: NodeId(0) },
                    iris_types::graph::NodeKind::Lambda => NodePayload::Lambda { binder: BinderId(0), captured_count: 0 },
                    iris_types::graph::NodeKind::Fold => NodePayload::Fold { recursion_descriptor: vec![0] },
                    _ => NodePayload::Prim { opcode: 0 },
                };
                let mut node = iris_types::graph::Node { id: NodeId(0), kind, type_sig, cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt, payload };
                node.id = compute_node_id(&node);
                let mut new_id = node.id;
                while graph.nodes.contains_key(&new_id) { node.salt += 1; node.id = compute_node_id(&node); new_id = node.id; }
                graph.nodes.insert(new_id, node);
                Ok(Value::tuple(vec![Value::Program(Rc::new(graph)), Value::Int(new_id.0 as i64)]))
            }
            // graph_connect(program, source, target, port) -> program
            0x86 => {
                let mut graph = match &a { Value::Program(g) => g.as_ref().clone(), _ => return Err(EvalError::TypeError("expected Program".into())) };
                let source = NodeId(match &b { Value::Int(n) => *n as u64, _ => return Err(EvalError::TypeError("expected Int".into())) });
                let target = NodeId(match &c { Value::Int(n) => *n as u64, _ => return Err(EvalError::TypeError("expected Int".into())) });
                let port = match args.get(3) { Some(Value::Int(n)) => *n as u8, _ => 0 };
                let label = match args.get(4) {
                    Some(Value::Int(3)) => EdgeLabel::Continuation,
                    Some(Value::Int(2)) => EdgeLabel::Binding,
                    _ => EdgeLabel::Argument,
                };
                graph.edges.push(Edge { source, target, port, label });
                Ok(Value::Program(Rc::new(graph)))
            }
            // graph_connect_labeled(program, source, target, port, label) -> program
            0xA5 => {
                let mut graph = match &a { Value::Program(g) => g.as_ref().clone(), _ => return Err(EvalError::TypeError("expected Program".into())) };
                let source = NodeId(match &b { Value::Int(n) => *n as u64, _ => return Err(EvalError::TypeError("expected Int".into())) });
                let target = NodeId(match &c { Value::Int(n) => *n as u64, _ => return Err(EvalError::TypeError("expected Int".into())) });
                let port = match args.get(3) { Some(Value::Int(n)) => *n as u8, _ => 0 };
                let label = match args.get(4) { Some(Value::Int(3)) => EdgeLabel::Continuation, Some(Value::Int(2)) => EdgeLabel::Binding, _ => EdgeLabel::Argument };
                graph.edges.push(Edge { source, target, port, label });
                Ok(Value::Program(Rc::new(graph)))
            }
            // graph_set_binder(program, node_id, binder_id) -> (program, new_node_id)
            0xA4 => {
                use iris_types::hash::compute_node_id;
                let mut graph = match &a { Value::Program(g) => g.as_ref().clone(), _ => return Err(EvalError::TypeError("expected Program".into())) };
                let node_id = NodeId(match &b { Value::Int(n) => *n as u64, _ => return Err(EvalError::TypeError("expected Int".into())) });
                let binder_val = match &c { Value::Int(n) => *n as u32, _ => return Err(EvalError::TypeError("expected Int".into())) };
                let mut new_id = node_id;
                if let Some(mut node) = graph.nodes.remove(&node_id) {
                    let old_id = node.id;
                    match &mut node.payload {
                        NodePayload::Lambda { binder, .. } => *binder = BinderId(binder_val),
                        _ => {}
                    }
                    node.id = compute_node_id(&node);
                    new_id = node.id;
                    graph.nodes.insert(new_id, node);
                    for edge in &mut graph.edges { if edge.source == old_id { edge.source = new_id; } if edge.target == old_id { edge.target = new_id; } }
                    if graph.root == old_id { graph.root = new_id; }
                }
                Ok(Value::tuple(vec![Value::Program(Rc::new(graph)), Value::Int(new_id.0 as i64)]))
            }
            // graph_set_prim_op(program, node_id, opcode) -> (program, new_node_id)
            0x84 => {
                use iris_types::hash::compute_node_id;
                let mut graph = match &a { Value::Program(g) => g.as_ref().clone(), _ => return Err(EvalError::TypeError("expected Program".into())) };
                let node_id = NodeId(match &b { Value::Int(n) => *n as u64, _ => return Err(EvalError::TypeError("expected Int".into())) });
                let new_opcode = match &c { Value::Int(n) => *n as u8, _ => return Err(EvalError::TypeError("expected Int".into())) };
                let mut new_id = node_id;
                if let Some(mut node) = graph.nodes.remove(&node_id) {
                    let old_id = node.id;
                    node.payload = NodePayload::Prim { opcode: new_opcode };
                    node.id = compute_node_id(&node);
                    new_id = node.id;
                    graph.nodes.insert(new_id, node);
                    for edge in &mut graph.edges { if edge.source == old_id { edge.source = new_id; } if edge.target == old_id { edge.target = new_id; } }
                    if graph.root == old_id { graph.root = new_id; }
                }
                Ok(Value::tuple(vec![Value::Program(Rc::new(graph)), Value::Int(new_id.0 as i64)]))
            }
            // graph_add_guard_rt(program, pred, body, fallback) -> (program, guard_id)
            0x8B => {
                use iris_types::hash::compute_node_id;
                use iris_types::cost::CostTerm;
                use iris_types::types::TypeId;
                let mut graph = match &a { Value::Program(g) => g.as_ref().clone(), _ => return Err(EvalError::TypeError("expected Program".into())) };
                let pred = NodeId(match &b { Value::Int(n) => *n as u64, _ => return Err(EvalError::TypeError("expected Int".into())) });
                let body = NodeId(match &c { Value::Int(n) => *n as u64, _ => return Err(EvalError::TypeError("expected Int".into())) });
                let fall = NodeId(match args.get(3) { Some(Value::Int(n)) => *n as u64, _ => 0 });
                let type_sig = graph.type_env.types.keys().next().copied().unwrap_or(TypeId(0));
                let salt = graph.nodes.len() as u64 + 1;
                let mut node = iris_types::graph::Node { id: NodeId(0), kind: iris_types::graph::NodeKind::Guard, type_sig, cost: CostTerm::Unit, arity: 0, resolution_depth: 0, salt, payload: NodePayload::Guard { predicate_node: pred, body_node: body, fallback_node: fall } };
                node.id = compute_node_id(&node);
                let nid = node.id;
                graph.nodes.insert(nid, node);
                Ok(Value::tuple(vec![Value::Program(Rc::new(graph)), Value::Int(nid.0 as i64)]))
            }
            // graph_set_cost(program, node_id, cost_val) -> program
            0x8D => {
                let mut graph = match &a { Value::Program(g) => g.as_ref().clone(), _ => return Err(EvalError::TypeError("expected Program".into())) };
                let node_id = NodeId(match &b { Value::Int(n) => *n as u64, _ => return Err(EvalError::TypeError("expected Int".into())) });
                let cost_val = match &c { Value::Int(n) => *n, _ => 0 };
                if let Some(node) = graph.nodes.get_mut(&node_id) {
                    node.cost = match cost_val { 0 => iris_types::cost::CostTerm::Unit, 1 => iris_types::cost::CostTerm::Inherited, n => iris_types::cost::CostTerm::Annotated(iris_types::cost::CostBound::Constant(n as u64)) };
                }
                Ok(Value::Program(Rc::new(graph)))
            }
            // graph_set_lit_value(program, node_id, type_tag, value) -> program
            0xEF => {
                use iris_types::hash::compute_node_id;
                let mut graph = match &a { Value::Program(g) => g.as_ref().clone(), _ => return Err(EvalError::TypeError("expected Program".into())) };
                let node_id = NodeId(match &b { Value::Int(n) => *n as u64, _ => return Err(EvalError::TypeError("expected Int".into())) });
                let type_tag = match &c { Value::Int(n) => *n as u8, _ => 0 };
                let val = args.get(3).cloned().unwrap_or(Value::Unit);
                let value_bytes = match type_tag {
                    0x00 => match &val { Value::Int(n) => n.to_le_bytes().to_vec(), _ => vec![0; 8] },
                    0x04 => match &val { Value::Bool(b) => vec![*b as u8], Value::Int(n) => vec![(*n != 0) as u8], _ => vec![0] },
                    0x06 => vec![],
                    0x07 => match &val { Value::String(s) => s.as_bytes().to_vec(), _ => vec![] },
                    0xFF => match &val { Value::Int(n) => vec![*n as u8], _ => vec![0] },
                    _ => match &val { Value::Int(n) => n.to_le_bytes().to_vec(), _ => vec![0; 8] },
                };
                let mut new_id = node_id;
                if let Some(mut node) = graph.nodes.remove(&node_id) {
                    let old_id = node.id;
                    node.payload = NodePayload::Lit { type_tag, value: value_bytes };
                    node.kind = iris_types::graph::NodeKind::Lit;
                    node.id = compute_node_id(&node);
                    new_id = node.id;
                    graph.nodes.insert(new_id, node);
                    for edge in &mut graph.edges { if edge.source == old_id { edge.source = new_id; } if edge.target == old_id { edge.target = new_id; } }
                    if graph.root == old_id { graph.root = new_id; }
                }
                Ok(Value::tuple(vec![Value::Program(Rc::new(graph)), Value::Int(new_id.0 as i64)]))
            }
            // graph_set_field_index(program, node_id, field_index) -> program
            0xF1 => {
                use iris_types::hash::compute_node_id;
                let mut graph = match &a { Value::Program(g) => g.as_ref().clone(), _ => return Err(EvalError::TypeError("expected Program".into())) };
                let node_id = NodeId(match &b { Value::Int(n) => *n as u64, _ => return Err(EvalError::TypeError("expected Int".into())) });
                let fi = match &c { Value::Int(n) => *n as u16, _ => 0 };
                let mut new_id = node_id;
                if let Some(mut node) = graph.nodes.remove(&node_id) {
                    let old_id = node.id;
                    node.payload = NodePayload::Project { field_index: fi };
                    node.kind = iris_types::graph::NodeKind::Project;
                    node.id = compute_node_id(&node);
                    new_id = node.id;
                    graph.nodes.insert(new_id, node);
                    for edge in &mut graph.edges { if edge.source == old_id { edge.source = new_id; } if edge.target == old_id { edge.target = new_id; } }
                    if graph.root == old_id { graph.root = new_id; }
                }
                Ok(Value::tuple(vec![Value::Program(Rc::new(graph)), Value::Int(new_id.0 as i64)]))
            }
            // graph_set_node_type(program, node_id, type_tag) -> program
            0x61 => {
                let mut graph = match &a { Value::Program(g) => g.as_ref().clone(), _ => return Err(EvalError::TypeError("expected Program".into())) };
                let node_id = NodeId(match &b { Value::Int(n) => *n as u64, _ => return Err(EvalError::TypeError("expected Int".into())) });
                let type_tag = match &c { Value::Int(n) => *n as u64, _ => 0 };
                let type_id = iris_types::types::TypeId(type_tag);
                if !graph.type_env.types.contains_key(&type_id) {
                    graph.type_env.types.insert(type_id, iris_types::types::TypeDef::Primitive(iris_types::types::PrimType::Int));
                }
                if let Some(node) = graph.nodes.get_mut(&node_id) {
                    node.type_sig = type_id;
                }
                Ok(Value::Program(Rc::new(graph)))
            }
            _ => Err(EvalError::TypeError(format!("unsupported opcode: 0x{:02X}", opcode))),
        }
    }

    fn eval_apply(&mut self, node_id: NodeId) -> Result<Value, EvalError> {
        let targets = self.arg_targets(node_id);
        if targets.len() < 2 {
            return Err(EvalError::MissingEdge { source: node_id, port: 1 });
        }
        let func = self.eval(targets[0])?;
        let arg = self.eval(targets[1])?;

        // Closure: (binder_id, body_node_id, captured_env)
        match &func {
            Value::Tuple(fields) if fields.len() == 3 => {
                if let (Value::Int(binder_raw), Value::Int(body_raw), Value::Tuple(captures)) =
                    (&fields[0], &fields[1], &fields[2])
                {
                    let binder = BinderId(*binder_raw as u32);
                    let body = NodeId(*body_raw as u64);
                    // Restore captured env
                    let saved = self.env.clone();
                    for cap in captures.iter() {
                        if let Value::Tuple(kv) = cap {
                            if let (Some(Value::Int(k)), Some(v)) = (kv.get(0), kv.get(1)) {
                                self.env.insert(BinderId(*k as u32), v.clone());
                            }
                        }
                    }
                    self.env.insert(binder, arg);
                    let result = self.eval(body);
                    self.env = saved;
                    return result;
                }
            }
            _ => {}
        }
        // Fallback: try applying as a program
        if let Value::Program(g) = &func {
            let empty_reg = BTreeMap::new();
            let mut sub_ctx = Ctx::new(g, &[arg], &empty_reg);
            sub_ctx.max_steps = self.max_steps.saturating_sub(self.steps);
            let result = sub_ctx.eval(g.root);
            self.steps += sub_ctx.steps;
            return result;
        }
        Err(EvalError::TypeError(format!("apply: expected closure or program, got {:?}", func)))
    }

    fn eval_let(&mut self, node_id: NodeId) -> Result<Value, EvalError> {
        let bind_target = self.bind_target(node_id)
            .ok_or(EvalError::MissingEdge { source: node_id, port: 0 })?;
        let cont_target = self.cont_target(node_id)
            .ok_or(EvalError::MissingEdge { source: node_id, port: 0 })?;

        let bound_val = self.eval(bind_target)?;

        // Check if continuation is a Lambda (let x = val in body)
        if let Some(node) = self.graph.nodes.get(&cont_target) {
            if let NodePayload::Lambda { binder, .. } = &node.payload {
                let binder = *binder;
                let body = self.cont_target(cont_target)
                    .or_else(|| self.arg_targets(cont_target).first().copied())
                    .ok_or(EvalError::MissingEdge { source: cont_target, port: 0 })?;
                let prev = self.env.insert(binder, bound_val);
                let result = self.eval(body);
                match prev {
                    Some(v) => { self.env.insert(binder, v); }
                    None => { self.env.remove(&binder); }
                }
                return result;
            }
        }
        self.eval(cont_target)
    }

    fn eval_fold(&mut self, node_id: NodeId) -> Result<Value, EvalError> {
        let targets = self.arg_targets(node_id);
        if targets.is_empty() {
            return Err(EvalError::MissingEdge { source: node_id, port: 0 });
        }

        let base = self.eval(targets[0])?;
        if targets.len() < 2 { return Ok(base); }
        let step_id = targets[1];

        let collection = if targets.len() >= 3 {
            self.eval(targets[2])?
        } else {
            Value::Int(0)
        };

        let count = match &collection {
            Value::Int(n) if *n > 0 => *n,
            Value::Range(s, e) => if *e > *s { *e - *s } else { 0 },
            Value::Tuple(t) => t.len() as i64,
            _ => 0,
        };

        // Check if step is a Prim (fast path)
        if let Some(node) = self.graph.nodes.get(&step_id) {
            if let NodePayload::Prim { opcode } = &node.payload {
                let op = *opcode;
                let mut acc = base;
                for i in 0..count {
                    let elem = match &collection {
                        Value::Range(s, _) => Value::Int(s + i),
                        Value::Tuple(t) => t.get(i as usize).cloned().unwrap_or(Value::Unit),
                        _ => Value::Int(i),
                    };
                    acc = self.dispatch_prim(op, &[acc, elem])?;
                }
                return Ok(acc);
            }
        }

        // Lambda step: evaluate closure
        let step_val = self.eval(step_id)?;
        let mut acc = base;

        if let Value::Tuple(fields) = &step_val {
            if fields.len() == 3 {
                if let (Value::Int(binder_raw), Value::Int(body_raw), Value::Tuple(captures)) =
                    (&fields[0], &fields[1], &fields[2])
                {
                    let binder = BinderId(*binder_raw as u32);
                    let body = NodeId(*body_raw as u64);

                    // Save env and install captures
                    let saved = self.env.clone();
                    for cap in captures.iter() {
                        if let Value::Tuple(kv) = cap {
                            if let (Some(Value::Int(k)), Some(v)) = (kv.get(0), kv.get(1)) {
                                self.env.insert(BinderId(*k as u32), v.clone());
                            }
                        }
                    }

                    for i in 0..count {
                        let elem = match &collection {
                            Value::Range(s, _) => Value::Int(s + i),
                            Value::Tuple(t) => t.get(i as usize).cloned().unwrap_or(Value::Unit),
                            _ => Value::Int(i),
                        };
                        let arg = Value::tuple(vec![
                            std::mem::replace(&mut acc, Value::Unit),
                            elem,
                        ]);
                        self.env.insert(binder, arg.clone());
                        match self.eval(body) {
                            Ok(v) => acc = v,
                            Err(e) => {
                                eprintln!("[fold debug] iteration {} failed: {}", i, e);
                                eprintln!("[fold debug] binder={:?} arg={:?}", binder, arg);
                                self.env = saved;
                                return Err(e);
                            }
                        }
                    }
                    self.env = saved;
                    return Ok(acc);
                }
            }
        }

        Ok(acc)
    }

    fn eval_ref(&mut self, node_id: NodeId, fragment_id: &FragmentId) -> Result<Value, EvalError> {
        let ref_graph = self.registry.get(fragment_id)
            .ok_or_else(|| EvalError::TypeError(format!("ref: fragment not found")))?;
        let targets = self.arg_targets(node_id);
        let mut args = Vec::new();
        for t in &targets {
            args.push(self.eval(*t)?);
        }
        let empty_reg = BTreeMap::new();
        let mut sub_ctx = Ctx::new(ref_graph, &args, &empty_reg);
        sub_ctx.max_steps = self.max_steps.saturating_sub(self.steps);
        let result = sub_ctx.eval(ref_graph.root);
        self.steps += sub_ctx.steps;
        result
    }

    // --- Graph introspection primitives ---

    fn get_graph<'b>(&self, v: &'b Value) -> Result<&'b SemanticGraph, EvalError> {
        match v {
            Value::Program(g) => Ok(g.as_ref()),
            _ => Err(EvalError::TypeError("expected Program".into())),
        }
    }

    fn get_int(&self, v: &Value) -> Result<i64, EvalError> {
        match v {
            Value::Int(n) => Ok(*n),
            _ => Err(EvalError::TypeError("expected Int".into())),
        }
    }

    fn prim_graph_get_root(&self, a: &Value) -> Result<Value, EvalError> {
        Ok(Value::Int(self.get_graph(a)?.root.0 as i64))
    }

    fn prim_graph_get_kind(&self, a: &Value, b: &Value) -> Result<Value, EvalError> {
        let g = self.get_graph(a)?;
        let nid = NodeId(self.get_int(b)? as u64);
        let node = g.nodes.get(&nid).ok_or(EvalError::MissingNode(nid))?;
        Ok(Value::Int(node.kind as i64))
    }

    fn prim_graph_get_prim_op(&self, a: &Value, b: &Value) -> Result<Value, EvalError> {
        let g = self.get_graph(a)?;
        let nid = NodeId(self.get_int(b)? as u64);
        let node = g.nodes.get(&nid).ok_or(EvalError::MissingNode(nid))?;
        match &node.payload {
            NodePayload::Prim { opcode } => Ok(Value::Int(*opcode as i64)),
            _ => Ok(Value::Int(-1)),
        }
    }

    fn prim_graph_get_lit_type_tag(&self, a: &Value, b: &Value) -> Result<Value, EvalError> {
        let g = self.get_graph(a)?;
        let nid = NodeId(self.get_int(b)? as u64);
        let node = g.nodes.get(&nid).ok_or(EvalError::MissingNode(nid))?;
        match &node.payload {
            NodePayload::Lit { type_tag, .. } => Ok(Value::Int(*type_tag as i64)),
            _ => Ok(Value::Int(-1)),
        }
    }

    fn prim_graph_get_lit_value(&self, a: &Value, b: &Value) -> Result<Value, EvalError> {
        let g = self.get_graph(a)?;
        let nid = NodeId(self.get_int(b)? as u64);
        let node = g.nodes.get(&nid).ok_or(EvalError::MissingNode(nid))?;
        match &node.payload {
            NodePayload::Lit { type_tag, value } => match type_tag {
                0x00 if value.len() == 8 => Ok(Value::Int(i64::from_le_bytes(value[..8].try_into().unwrap()))),
                0x02 if value.len() == 8 => Ok(Value::Float64(f64::from_le_bytes(value[..8].try_into().unwrap()))),
                0x04 if value.len() == 1 => Ok(Value::Bool(value[0] != 0)),
                0x07 => Ok(Value::String(String::from_utf8_lossy(value).into_owned())),
                0xFF if !value.is_empty() => Ok(Value::Int(value[0] as i64)),
                _ => Ok(Value::Unit),
            },
            _ => Err(EvalError::TypeError("not a Lit node".into())),
        }
    }

    fn prim_graph_outgoing(&self, a: &Value, b: &Value) -> Result<Value, EvalError> {
        let g = self.get_graph(a)?;
        let nid = NodeId(self.get_int(b)? as u64);
        let children: Vec<Value> = g.edges.iter()
            .filter(|e| e.source == nid && e.label == EdgeLabel::Argument)
            .map(|e| Value::Int(e.target.0 as i64))
            .collect();
        Ok(Value::tuple(children))
    }

    fn prim_graph_set_root(&self, a: &Value, b: &Value) -> Result<Value, EvalError> {
        let g = self.get_graph(a)?;
        let new_root = NodeId(self.get_int(b)? as u64);
        let mut new_graph = g.clone();
        new_graph.root = new_root;
        Ok(Value::Program(Rc::new(new_graph)))
    }

    fn prim_graph_nodes(&self, a: &Value) -> Result<Value, EvalError> {
        let g = self.get_graph(a)?;
        Ok(Value::tuple(g.nodes.keys().map(|k| Value::Int(k.0 as i64)).collect()))
    }

    fn prim_graph_edge_count(&self, a: &Value, b: &Value) -> Result<Value, EvalError> {
        let g = self.get_graph(a)?;
        let nid = NodeId(self.get_int(b)? as u64);
        let count = g.edges.iter().filter(|e| e.source == nid).count();
        Ok(Value::Int(count as i64))
    }

    fn prim_graph_edge_target(&self, args: &[Value]) -> Result<Value, EvalError> {
        if args.len() < 4 { return Ok(Value::Int(-1)); }
        let g = self.get_graph(&args[0])?;
        let source = NodeId(self.get_int(&args[1])? as u64);
        let port = self.get_int(&args[2])? as u8;
        let label_val = self.get_int(&args[3])? as u8;
        let label = match label_val {
            0 => EdgeLabel::Argument,
            1 => EdgeLabel::Scrutinee,
            2 => EdgeLabel::Binding,
            3 => EdgeLabel::Continuation,
            _ => return Ok(Value::Int(-1)),
        };
        // Check Guard payload first
        if let Some(node) = g.nodes.get(&source) {
            if let NodePayload::Guard { predicate_node, body_node, fallback_node } = &node.payload {
                if label == EdgeLabel::Argument {
                    return Ok(match port {
                        0 => Value::Int(predicate_node.0 as i64),
                        1 => Value::Int(body_node.0 as i64),
                        2 => Value::Int(fallback_node.0 as i64),
                        _ => Value::Int(-1),
                    });
                }
            }
        }
        for edge in &g.edges {
            if edge.source == source && edge.port == port && edge.label == label {
                return Ok(Value::Int(edge.target.0 as i64));
            }
        }
        Ok(Value::Int(-1))
    }

    fn prim_graph_get_binder(&self, a: &Value, b: &Value) -> Result<Value, EvalError> {
        let g = self.get_graph(a)?;
        let nid = NodeId(self.get_int(b)? as u64);
        let node = g.nodes.get(&nid).ok_or(EvalError::MissingNode(nid))?;
        match &node.payload {
            NodePayload::Lambda { binder, .. } => Ok(Value::Int(binder.0 as i64)),
            NodePayload::LetRec { binder, .. } => Ok(Value::Int(binder.0 as i64)),
            _ => Ok(Value::Int(-1)),
        }
    }

    fn prim_graph_get_tag(&self, a: &Value, b: &Value) -> Result<Value, EvalError> {
        let g = self.get_graph(a)?;
        let nid = NodeId(self.get_int(b)? as u64);
        let node = g.nodes.get(&nid).ok_or(EvalError::MissingNode(nid))?;
        match &node.payload {
            NodePayload::Inject { tag_index } => Ok(Value::Int(*tag_index as i64)),
            _ => Ok(Value::Int(-1)),
        }
    }

    fn prim_graph_get_field_index(&self, a: &Value, b: &Value) -> Result<Value, EvalError> {
        let g = self.get_graph(a)?;
        let nid = NodeId(self.get_int(b)? as u64);
        let node = g.nodes.get(&nid).ok_or(EvalError::MissingNode(nid))?;
        match &node.payload {
            NodePayload::Project { field_index } => Ok(Value::Int(*field_index as i64)),
            _ => Ok(Value::Int(-1)),
        }
    }

    fn prim_graph_get_effect_tag(&self, a: &Value, b: &Value) -> Result<Value, EvalError> {
        let g = self.get_graph(a)?;
        let nid = NodeId(self.get_int(b)? as u64);
        let node = g.nodes.get(&nid).ok_or(EvalError::MissingNode(nid))?;
        match &node.payload {
            NodePayload::Effect { effect_tag } => Ok(Value::Int(*effect_tag as i64)),
            _ => Ok(Value::Int(-1)),
        }
    }

    fn prim_graph_eval(&mut self, args: &[Value]) -> Result<Value, EvalError> {
        if args.is_empty() { return Err(EvalError::TypeError("graph_eval: no args".into())); }
        let graph = match &args[0] {
            Value::Program(g) => g.clone(),
            _ => return Err(EvalError::TypeError("graph_eval: expected Program".into())),
        };
        let inputs: Vec<Value> = if args.len() > 1 {
            match &args[1] {
                Value::Tuple(elems) => elems.as_ref().clone(),
                other => vec![other.clone()],
            }
        } else { vec![] };

        let empty_reg = BTreeMap::new();
        let mut sub_ctx = Ctx::new(&graph, &inputs, &empty_reg);
        sub_ctx.max_steps = self.max_steps.saturating_sub(self.steps);
        let result = sub_ctx.eval(graph.root);
        self.steps += sub_ctx.steps;
        result
    }

    fn prim_graph_eval_ref(&mut self, args: &[Value]) -> Result<Value, EvalError> {
        if args.len() < 2 { return Err(EvalError::TypeError("graph_eval_ref: need 2+ args".into())); }
        let graph = match &args[0] {
            Value::Program(g) => g.clone(),
            _ => return Err(EvalError::TypeError("expected Program".into())),
        };
        let ref_node_id = NodeId(match &args[1] {
            Value::Int(n) => *n as u64,
            _ => return Err(EvalError::TypeError("expected Int".into())),
        });
        let node = graph.nodes.get(&ref_node_id)
            .ok_or(EvalError::MissingNode(ref_node_id))?;
        if let NodePayload::Ref { fragment_id } = &node.payload {
            if let Some(ref_graph) = self.registry.get(fragment_id) {
                let ref_inputs: Vec<Value> = args[2..].to_vec();
                let empty_reg = BTreeMap::new();
                let mut sub_ctx = Ctx::new(ref_graph, &ref_inputs, &empty_reg);
                sub_ctx.max_steps = self.max_steps.saturating_sub(self.steps);
                let result = sub_ctx.eval(ref_graph.root);
                self.steps += sub_ctx.steps;
                return result;
            }
        }
        Err(EvalError::TypeError("graph_eval_ref: not a Ref node".into()))
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Evaluate a SemanticGraph with inputs. Minimal tree-walker.
pub fn evaluate(graph: &SemanticGraph, inputs: &[Value]) -> Result<Value, EvalError> {
    let empty_reg = BTreeMap::new();
    evaluate_with_registry(graph, inputs, MAX_STEPS, &empty_reg)
}

/// Evaluate with step limit and fragment registry.
pub fn evaluate_with_registry(
    graph: &SemanticGraph,
    inputs: &[Value],
    max_steps: u64,
    registry: &BTreeMap<FragmentId, SemanticGraph>,
) -> Result<Value, EvalError> {
    let mut ctx = Ctx::new(graph, inputs, registry);
    ctx.max_steps = max_steps;
    ctx.eval(graph.root)
}

/// Load a SemanticGraph from a JSON file.
pub fn load_graph(path: &str) -> Result<SemanticGraph, String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path, e))?;
    serde_json::from_str(&data)
        .map_err(|e| format!("failed to parse {}: {}", path, e))
}
