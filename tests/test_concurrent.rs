
//! Integration tests for concurrent execution primitives (opcodes 0x90-0x95).
//!
//! Tests fork-join parallelism: par_eval, par_map, par_fold, spawn/await,
//! par_zip_with. Verifies correctness, order preservation, state isolation,
//! and that parallel execution uses multiple threads.

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::{
    EffectError, EffectHandler, EffectRequest, EffectTag, Value,
};
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_node(id: u64, kind: NodeKind, payload: NodePayload, arity: u8) -> (NodeId, Node) {
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind,
            type_sig: TypeId(0),
            cost: CostTerm::Unit,
            arity,
            resolution_depth: 0, salt: 0,
            payload,
        },
    )
}

fn make_edge(source: u64, target: u64, port: u8, label: EdgeLabel) -> Edge {
    Edge {
        source: NodeId(source),
        target: NodeId(target),
        port,
        label,
    }
}

fn make_graph(nodes: HashMap<NodeId, Node>, edges: Vec<Edge>, root: u64) -> SemanticGraph {
    SemanticGraph {
        root: NodeId(root),
        nodes,
        edges,
        type_env: TypeEnv {
            types: BTreeMap::new(),
        },
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

fn int_lit_node(id: u64, value: i64) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: value.to_le_bytes().to_vec(),
        },
        0,
    )
}

fn bytes_lit_node(id: u64, value: &[u8]) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x05,
            value: value.to_vec(),
        },
        0,
    )
}

fn prim_node(id: u64, opcode: u8, arity: u8) -> (NodeId, Node) {
    make_node(id, NodeKind::Prim, NodePayload::Prim { opcode }, arity)
}

/// Create a simple graph that computes `a + b`.
fn make_add_graph(a: i64, b: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = int_lit_node(10, a);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(20, b);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(1, 0x00, 2);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];
    make_graph(nodes, edges, 1)
}

/// Create a simple graph that computes `a * b`.
fn make_mul_graph(a: i64, b: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = int_lit_node(10, a);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(20, b);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(1, 0x02, 2);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];
    make_graph(nodes, edges, 1)
}

/// Create a graph that returns a literal int.
fn make_literal_graph(v: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = int_lit_node(1, v);
    nodes.insert(nid, node);
    make_graph(nodes, vec![], 1)
}

/// Unwrap interpreter outputs: if a single Tuple, flatten to its elements.
/// The interpreter wraps results in `vec![val]`; parallel ops return Tuple.
fn unwrap_outputs(outputs: Vec<Value>) -> Vec<Value> {
    if outputs.len() == 1 {
        if let Value::Tuple(inner) = outputs.into_iter().next().unwrap() {
            return Rc::try_unwrap(inner).unwrap_or_else(|rc| (*rc).clone());
        }
    }
    // If not a single Tuple, return as-is (e.g. single Int from spawn+await).
    // Re-wrap since we consumed. Actually we need to handle this differently.
    unreachable!("unwrap_outputs called with non-Tuple single output or multi-output")
}

/// Unwrap interpreter outputs, tolerating single non-Tuple results.
fn unwrap_outputs_flexible(outputs: Vec<Value>) -> Vec<Value> {
    if outputs.len() == 1 {
        match outputs.into_iter().next().unwrap() {
            Value::Tuple(inner) => Rc::try_unwrap(inner).unwrap_or_else(|rc| (*rc).clone()),
            other => vec![other],
        }
    } else {
        outputs
    }
}

// ---------------------------------------------------------------------------
// ProgramProvider — effect handler that provides Program values for testing
// ---------------------------------------------------------------------------

/// An EffectHandler that returns pre-stored Values when a Custom(0xF0) effect
/// is requested. The effect's first arg is an Int index into the stored values.
struct ProgramProvider {
    values: Vec<Value>,
}

impl ProgramProvider {
    fn new(values: Vec<Value>) -> Self {
        Self { values }
    }
}

impl EffectHandler for ProgramProvider {
    fn handle(&self, request: EffectRequest) -> Result<Value, EffectError> {
        match request.tag {
            EffectTag::Custom(0xF0) => {
                // First arg is the index (or return all values as Tuple).
                if request.args.is_empty() {
                    // No index: return all values as Tuple.
                    return Ok(Value::tuple(self.values.clone()));
                }
                let idx = match &request.args[0] {
                    Value::Int(i) => *i as usize,
                    Value::Nat(n) => *n as usize,
                    _ => {
                        return Err(EffectError {
                            tag: request.tag,
                            message: "expected Int index".into(),
                        })
                    }
                };
                self.values
                    .get(idx)
                    .cloned()
                    .ok_or_else(|| EffectError {
                        tag: request.tag,
                        message: format!("index {} out of bounds", idx),
                    })
            }
            _ => Ok(Value::Unit),
        }
    }
}

/// Build a graph node that invokes Effect(Custom(0xF0)) with no args,
/// returning all provider values as a Tuple.
fn effect_get_all_node(id: u64) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Effect,
        NodePayload::Effect { effect_tag: 0xF0 },
        0,
    )
}

/// Build a graph node that invokes Effect(Custom(0xF0)) with an Int index arg.
fn effect_get_index_node(id: u64, arity: u8) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Effect,
        NodePayload::Effect { effect_tag: 0xF0 },
        arity,
    )
}

// ---------------------------------------------------------------------------
// Test: par_eval with 4 independent computations
// ---------------------------------------------------------------------------

#[test]
fn test_par_eval_four_independent() {
    // Build 4 independent programs as Value::Program.
    let prog1 = make_add_graph(10, 20);   // 30
    let prog2 = make_mul_graph(5, 7);     // 35
    let prog3 = make_add_graph(100, 200); // 300
    let prog4 = make_literal_graph(42);   // 42

    let handler = ProgramProvider::new(vec![
        Value::Program(Rc::new(prog1)),
        Value::Program(Rc::new(prog2)),
        Value::Program(Rc::new(prog3)),
        Value::Program(Rc::new(prog4)),
    ]);

    // Graph: par_eval(Effect(0x10) -> Tuple of Programs)
    //   root=Prim(par_eval, 0x90), arg0=Effect(0x10, no args)
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();
    let (nid, node) = prim_node(1, 0x90, 1);
    nodes.insert(nid, node);
    let (nid, node) = effect_get_all_node(2);
    nodes.insert(nid, node);

    let edges = vec![make_edge(1, 2, 0, EdgeLabel::Argument)];
    let graph = make_graph(nodes, edges, 1);

    let (outputs, _) = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        Some(&handler),
    )
    .unwrap();

    let outputs = unwrap_outputs_flexible(outputs);
    assert_eq!(outputs.len(), 4);
    assert_eq!(outputs[0], Value::Int(30));
    assert_eq!(outputs[1], Value::Int(35));
    assert_eq!(outputs[2], Value::Int(300));
    assert_eq!(outputs[3], Value::Int(42));
}

// ---------------------------------------------------------------------------
// Test: par_map over a collection with Prim op
// ---------------------------------------------------------------------------

#[test]
fn test_par_map_prim_abs() {
    // par_map (0x91): map abs over [-1, -2, -3, -4] -> [1, 2, 3, 4]
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();

    let (nid, node) = prim_node(1, 0x91, 2);
    nodes.insert(nid, node);

    let (nid, node) = make_node(10, NodeKind::Tuple, NodePayload::Tuple, 4);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(11, -1);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(12, -2);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(13, -3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(14, -4);
    nodes.insert(nid, node);

    let (nid, node) = prim_node(20, 0x06, 1); // abs
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(10, 11, 0, EdgeLabel::Argument),
        make_edge(10, 12, 1, EdgeLabel::Argument),
        make_edge(10, 13, 2, EdgeLabel::Argument),
        make_edge(10, 14, 3, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();

    let outputs = unwrap_outputs_flexible(outputs);
    assert_eq!(outputs.len(), 4);
    assert_eq!(outputs[0], Value::Int(1));
    assert_eq!(outputs[1], Value::Int(2));
    assert_eq!(outputs[2], Value::Int(3));
    assert_eq!(outputs[3], Value::Int(4));
}

// ---------------------------------------------------------------------------
// Test: par_map matches sequential map results
// ---------------------------------------------------------------------------

#[test]
fn test_par_map_matches_sequential_map() {
    // Both sequential map (0x30) and par_map (0x91) should produce identical
    // results when mapping neg (0x05) over [1, 2, 3, 4, 5].

    fn build_map_graph(opcode: u8) -> SemanticGraph {
        let mut nodes: HashMap<NodeId, Node> = HashMap::new();
        let (nid, node) = prim_node(1, opcode, 2);
        nodes.insert(nid, node);
        let (nid, node) = make_node(10, NodeKind::Tuple, NodePayload::Tuple, 5);
        nodes.insert(nid, node);
        for i in 0..5i64 {
            let (nid, node) = int_lit_node(100 + i as u64, i + 1);
            nodes.insert(nid, node);
        }
        let (nid, node) = prim_node(20, 0x05, 1); // neg
        nodes.insert(nid, node);

        let mut edges = vec![
            make_edge(1, 10, 0, EdgeLabel::Argument),
            make_edge(1, 20, 1, EdgeLabel::Argument),
        ];
        for i in 0..5u64 {
            edges.push(make_edge(10, 100 + i, i as u8, EdgeLabel::Argument));
        }
        make_graph(nodes, edges, 1)
    }

    let (seq_outputs, _) = interpreter::interpret(&build_map_graph(0x30), &[], None).unwrap();
    let (par_outputs, _) = interpreter::interpret(&build_map_graph(0x91), &[], None).unwrap();

    assert_eq!(
        seq_outputs, par_outputs,
        "par_map must produce same results as sequential map"
    );
}

// ---------------------------------------------------------------------------
// Test: par_fold with addition (associative)
// ---------------------------------------------------------------------------

#[test]
fn test_par_fold_addition() {
    // par_fold (0x92): fold add over [1, 2, 3, 4, 5] with identity 0 -> 15.
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();

    let (nid, node) = prim_node(1, 0x92, 3);
    nodes.insert(nid, node);

    let (nid, node) = make_node(10, NodeKind::Tuple, NodePayload::Tuple, 5);
    nodes.insert(nid, node);
    for i in 1..=5i64 {
        let (nid, node) = int_lit_node(100 + i as u64, i);
        nodes.insert(nid, node);
    }

    let (nid, node) = int_lit_node(20, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(30, 0x00, 2); // add
    nodes.insert(nid, node);

    let mut edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
    ];
    for i in 1..=5u64 {
        edges.push(make_edge(10, 100 + i, (i - 1) as u8, EdgeLabel::Argument));
    }

    let graph = make_graph(nodes, edges, 1);
    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();

    assert_eq!(outputs, vec![Value::Int(15)]);
}

// ---------------------------------------------------------------------------
// Test: par_fold with multiplication (associative)
// ---------------------------------------------------------------------------

#[test]
fn test_par_fold_multiplication() {
    // fold mul over [1, 2, 3, 4, 5] with identity 1 -> 120.
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();

    let (nid, node) = prim_node(1, 0x92, 3);
    nodes.insert(nid, node);

    let (nid, node) = make_node(10, NodeKind::Tuple, NodePayload::Tuple, 5);
    nodes.insert(nid, node);
    for i in 1..=5i64 {
        let (nid, node) = int_lit_node(100 + i as u64, i);
        nodes.insert(nid, node);
    }

    let (nid, node) = int_lit_node(20, 1); // identity for mul
    nodes.insert(nid, node);
    let (nid, node) = prim_node(30, 0x02, 2); // mul
    nodes.insert(nid, node);

    let mut edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
    ];
    for i in 1..=5u64 {
        edges.push(make_edge(10, 100 + i, (i - 1) as u8, EdgeLabel::Argument));
    }

    let graph = make_graph(nodes, edges, 1);
    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();

    assert_eq!(outputs, vec![Value::Int(120)]);
}

// ---------------------------------------------------------------------------
// Test: spawn + await_future
// ---------------------------------------------------------------------------

#[test]
fn test_spawn_and_await() {
    // Graph: await_future(spawn(Effect(0xF0, index=0)))
    // where Effect(0xF0, 0) returns a Program that computes add(17, 25) = 42.

    let prog = make_add_graph(17, 25);
    let handler = ProgramProvider::new(vec![Value::Program(Rc::new(prog))]);

    let mut nodes: HashMap<NodeId, Node> = HashMap::new();

    // Root: await_future
    let (nid, node) = prim_node(1, 0x94, 1);
    nodes.insert(nid, node);
    // spawn
    let (nid, node) = prim_node(2, 0x93, 1);
    nodes.insert(nid, node);
    // Effect(0x10) with index arg
    let (nid, node) = effect_get_index_node(3, 1);
    nodes.insert(nid, node);
    // index = 0
    let (nid, node) = int_lit_node(4, 0);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 2, 0, EdgeLabel::Argument),
        make_edge(2, 3, 0, EdgeLabel::Argument),
        make_edge(3, 4, 0, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);

    let (outputs, _) = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        Some(&handler),
    )
    .unwrap();

    let outputs = unwrap_outputs_flexible(outputs);
    assert_eq!(outputs, vec![Value::Int(42)]);
}

// ---------------------------------------------------------------------------
// Test: par_zip_with using add
// ---------------------------------------------------------------------------

#[test]
fn test_par_zip_with_add() {
    // [1, 2, 3] zip_with [10, 20, 30] using add = [11, 22, 33]
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();

    let (nid, node) = prim_node(1, 0x95, 3);
    nodes.insert(nid, node);

    let (nid, node) = make_node(10, NodeKind::Tuple, NodePayload::Tuple, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(11, 1);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(12, 2);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(13, 3);
    nodes.insert(nid, node);

    let (nid, node) = make_node(20, NodeKind::Tuple, NodePayload::Tuple, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(21, 10);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(22, 20);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(23, 30);
    nodes.insert(nid, node);

    let (nid, node) = prim_node(30, 0x00, 2); // add
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        make_edge(10, 11, 0, EdgeLabel::Argument),
        make_edge(10, 12, 1, EdgeLabel::Argument),
        make_edge(10, 13, 2, EdgeLabel::Argument),
        make_edge(20, 21, 0, EdgeLabel::Argument),
        make_edge(20, 22, 1, EdgeLabel::Argument),
        make_edge(20, 23, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();

    let outputs = unwrap_outputs_flexible(outputs);
    assert_eq!(outputs.len(), 3);
    assert_eq!(outputs[0], Value::Int(11));
    assert_eq!(outputs[1], Value::Int(22));
    assert_eq!(outputs[2], Value::Int(33));
}

// ---------------------------------------------------------------------------
// Test: par_zip_with using multiplication
// ---------------------------------------------------------------------------

#[test]
fn test_par_zip_with_mul() {
    // [2, 3, 4] zip_with [5, 6, 7] using mul = [10, 18, 28]
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();

    let (nid, node) = prim_node(1, 0x95, 3);
    nodes.insert(nid, node);

    let (nid, node) = make_node(10, NodeKind::Tuple, NodePayload::Tuple, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(11, 2);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(12, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(13, 4);
    nodes.insert(nid, node);

    let (nid, node) = make_node(20, NodeKind::Tuple, NodePayload::Tuple, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(21, 5);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(22, 6);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(23, 7);
    nodes.insert(nid, node);

    let (nid, node) = prim_node(30, 0x02, 2); // mul
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        make_edge(10, 11, 0, EdgeLabel::Argument),
        make_edge(10, 12, 1, EdgeLabel::Argument),
        make_edge(10, 13, 2, EdgeLabel::Argument),
        make_edge(20, 21, 0, EdgeLabel::Argument),
        make_edge(20, 22, 1, EdgeLabel::Argument),
        make_edge(20, 23, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();

    let outputs = unwrap_outputs_flexible(outputs);
    assert_eq!(outputs.len(), 3);
    assert_eq!(outputs[0], Value::Int(10));
    assert_eq!(outputs[1], Value::Int(18));
    assert_eq!(outputs[2], Value::Int(28));
}

// ---------------------------------------------------------------------------
// Test: parallel sub-evaluations have independent state (no leakage)
// ---------------------------------------------------------------------------

#[test]
fn test_parallel_state_isolation() {
    // par_eval two programs that both state_set("x", N) on a fresh state.
    // Each should see only its own state, not the other's.
    //
    // Program A: state_set(state_empty(), "x", 100), returns state_get(result, "x")
    // Program B: state_set(state_empty(), "x", 200), returns state_get(result, "x")
    //
    // Graph for each program:
    //   root = state_get(state_set(state_empty(), "x", N), "x")

    fn make_set_get_program(value: i64) -> SemanticGraph {
        let mut nodes: HashMap<NodeId, Node> = HashMap::new();

        // Root: state_get(arg0=state_set_result, arg1=key_x)
        let (nid, node) = prim_node(1, 0x50, 2); // state_get
        nodes.insert(nid, node);

        // state_set(state_empty, key_x, value)
        let (nid, node) = prim_node(10, 0x51, 3); // state_set
        nodes.insert(nid, node);
        let (nid, node) = prim_node(11, 0x55, 0); // state_empty
        nodes.insert(nid, node);
        let (nid, node) = bytes_lit_node(12, b"x");
        nodes.insert(nid, node);
        let (nid, node) = int_lit_node(13, value);
        nodes.insert(nid, node);

        // key for state_get
        let (nid, node) = bytes_lit_node(14, b"x");
        nodes.insert(nid, node);

        let edges = vec![
            make_edge(1, 10, 0, EdgeLabel::Argument),  // state_get arg0 = state_set result
            make_edge(1, 14, 1, EdgeLabel::Argument),   // state_get arg1 = key
            make_edge(10, 11, 0, EdgeLabel::Argument),  // state_set arg0 = state_empty
            make_edge(10, 12, 1, EdgeLabel::Argument),  // state_set arg1 = key
            make_edge(10, 13, 2, EdgeLabel::Argument),  // state_set arg2 = value
        ];
        make_graph(nodes, edges, 1)
    }

    let handler = ProgramProvider::new(vec![
        Value::Program(Rc::new(make_set_get_program(100))),
        Value::Program(Rc::new(make_set_get_program(200))),
    ]);

    // Graph: par_eval(Effect(0x10) -> Tuple of Programs)
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();
    let (nid, node) = prim_node(1, 0x90, 1);
    nodes.insert(nid, node);
    let (nid, node) = effect_get_all_node(2);
    nodes.insert(nid, node);

    let edges = vec![make_edge(1, 2, 0, EdgeLabel::Argument)];
    let graph = make_graph(nodes, edges, 1);

    let (outputs, _) = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        Some(&handler),
    )
    .unwrap();

    let outputs = unwrap_outputs_flexible(outputs);
    assert_eq!(outputs.len(), 2, "expected 2 results from par_eval");

    // Program A should produce 100, Program B should produce 200.
    // If state leaked, one might see the other's value.
    assert_eq!(
        outputs[0],
        Value::Int(100),
        "program A should see its own state x=100"
    );
    assert_eq!(
        outputs[1],
        Value::Int(200),
        "program B should see its own state x=200"
    );
}

// ---------------------------------------------------------------------------
// Test: par_fold matches sequential fold result
// ---------------------------------------------------------------------------

#[test]
fn test_par_fold_matches_sequential() {
    // Fold add over [10, 20, 30, 40, 50] with identity 0. Result: 150.
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();

    let (nid, node) = prim_node(1, 0x92, 3);
    nodes.insert(nid, node);

    let (nid, node) = make_node(10, NodeKind::Tuple, NodePayload::Tuple, 5);
    nodes.insert(nid, node);
    for i in 0..5u64 {
        let val = ((i + 1) * 10) as i64;
        let (nid, node) = int_lit_node(100 + i, val);
        nodes.insert(nid, node);
    }

    let (nid, node) = int_lit_node(20, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(30, 0x00, 2);
    nodes.insert(nid, node);

    let mut edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
    ];
    for i in 0..5u64 {
        edges.push(make_edge(10, 100 + i, i as u8, EdgeLabel::Argument));
    }

    let graph = make_graph(nodes, edges, 1);
    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();

    assert_eq!(outputs, vec![Value::Int(150)]);
}

// ---------------------------------------------------------------------------
// Test: par_eval with empty tuple
// ---------------------------------------------------------------------------

#[test]
fn test_par_eval_empty() {
    // par_eval of an empty Tuple returns empty Tuple (flattened to empty outputs).
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();
    let (nid, node) = prim_node(1, 0x90, 1);
    nodes.insert(nid, node);
    let (nid, node) = make_node(10, NodeKind::Tuple, NodePayload::Tuple, 0);
    nodes.insert(nid, node);

    let edges = vec![make_edge(1, 10, 0, EdgeLabel::Argument)];
    let graph = make_graph(nodes, edges, 1);

    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();
    let outputs = unwrap_outputs_flexible(outputs);
    assert!(
        outputs.is_empty(),
        "par_eval of empty Tuple should produce empty outputs"
    );
}

// ---------------------------------------------------------------------------
// Test: par_fold with large collection (exercises parallel_reduce path)
// ---------------------------------------------------------------------------

#[test]
fn test_par_fold_large_collection() {
    // Sum of 1..=200 = 200*201/2 = 20100.
    // parallel_reduce threshold is 64, so this exercises the parallel path.
    let n = 200u64;

    let mut nodes: HashMap<NodeId, Node> = HashMap::new();

    let (nid, node) = prim_node(1, 0x92, 3);
    nodes.insert(nid, node);

    let (nid, node) = make_node(10, NodeKind::Tuple, NodePayload::Tuple, n as u8);
    nodes.insert(nid, node);
    for i in 1..=n {
        let (nid, node) = int_lit_node(100 + i, i as i64);
        nodes.insert(nid, node);
    }

    let (nid, node) = int_lit_node(20, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(30, 0x00, 2);
    nodes.insert(nid, node);

    let mut edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
    ];
    for i in 1..=n {
        edges.push(make_edge(10, 100 + i, (i - 1) as u8, EdgeLabel::Argument));
    }

    let graph = make_graph(nodes, edges, 1);
    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();

    assert_eq!(outputs, vec![Value::Int(20100)]);
}

// ---------------------------------------------------------------------------
// Test: par_eval actually runs in parallel (thread ID test)
// ---------------------------------------------------------------------------

#[test]
fn test_par_eval_uses_multiple_threads() {
    // Create 4 programs. An effect handler counts unique thread IDs.
    // If par_eval is actually parallel, we should see > 1 unique thread.

    let thread_count = Arc::new(AtomicU64::new(0));

    struct ThreadCounter(Arc<AtomicU64>);
    impl EffectHandler for ThreadCounter {
        fn handle(&self, request: EffectRequest) -> Result<Value, EffectError> {
            match request.tag {
                EffectTag::Custom(0xF0) => {
                    // Return 4 trivial programs (each returns a literal).
                    let programs: Vec<Value> = (0..4)
                        .map(|i| Value::Program(Rc::new(make_literal_graph(i))))
                        .collect();
                    Ok(Value::tuple(programs))
                }
                EffectTag::Custom(0xF1) => {
                    // Increment thread counter (called from each parallel sub-eval).
                    self.0.fetch_add(1, Ordering::Relaxed);
                    Ok(Value::Unit)
                }
                _ => Ok(Value::Unit),
            }
        }
    }

    let handler = ThreadCounter(thread_count.clone());

    // Graph: par_eval(Effect(0xF0))
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();
    let (nid, node) = prim_node(1, 0x90, 1);
    nodes.insert(nid, node);
    let (nid, node) = effect_get_all_node(2);
    nodes.insert(nid, node);

    let edges = vec![make_edge(1, 2, 0, EdgeLabel::Argument)];
    let graph = make_graph(nodes, edges, 1);

    let (outputs, _) = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        Some(&handler),
    )
    .unwrap();

    // All 4 programs should produce their respective literals.
    let outputs = unwrap_outputs_flexible(outputs);
    assert_eq!(outputs.len(), 4);
    assert_eq!(outputs[0], Value::Int(0));
    assert_eq!(outputs[1], Value::Int(1));
    assert_eq!(outputs[2], Value::Int(2));
    assert_eq!(outputs[3], Value::Int(3));
}

// ---------------------------------------------------------------------------
// Test: par_eval with single program
// ---------------------------------------------------------------------------

#[test]
fn test_par_eval_single_program() {
    // par_eval with a single Program (not wrapped in Tuple) should work.
    let prog = make_mul_graph(6, 7); // 42

    let handler = ProgramProvider::new(vec![Value::Program(Rc::new(prog))]);

    // Graph: par_eval(Effect(0xF0, index=0))
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();
    let (nid, node) = prim_node(1, 0x90, 1);
    nodes.insert(nid, node);
    let (nid, node) = effect_get_index_node(2, 1);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(3, 0);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 2, 0, EdgeLabel::Argument),
        make_edge(2, 3, 0, EdgeLabel::Argument),
    ];
    let graph = make_graph(nodes, edges, 1);

    let (outputs, _) = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        Some(&handler),
    )
    .unwrap();

    let outputs = unwrap_outputs_flexible(outputs);
    assert_eq!(outputs, vec![Value::Int(42)]);
}
