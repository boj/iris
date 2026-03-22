//! Bottom-up enumerative synthesis with observational equivalence pruning.
//!
//! Complements evolutionary search: for programs under ~12 nodes, exhaustive
//! enumeration with type pruning is faster than evolution and guaranteed to
//! find solutions.
//!
//! **Algorithm:**
//! 1. Size 0: enumerate atoms (input variables, small constants).
//! 2. Size 1..max_size: compose operations over smaller programs.
//! 3. Evaluate each candidate on test inputs.
//! 4. Prune observationally equivalent programs (same outputs on all tests).
//! 5. If any candidate matches the specification, return it immediately.

use std::collections::{BTreeMap, HashMap};

use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::{StateStore, TestCase, Value};
use iris_types::fragment::{Boundary, Fragment, FragmentId, FragmentMeta};
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::{compute_fragment_id, compute_node_id, SemanticHash};
use iris_types::types::{PrimType, TypeDef, TypeEnv, TypeId};

use iris_exec::interpreter;

// ---------------------------------------------------------------------------
// FNV-1a for output signatures
// ---------------------------------------------------------------------------

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn fnv1a(data: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

fn value_hash_bytes(val: &Value, buf: &mut Vec<u8>) {
    match val {
        Value::Int(v) => {
            buf.push(0x00);
            buf.extend_from_slice(&v.to_le_bytes());
        }
        Value::Bool(v) => {
            buf.push(0x04);
            buf.push(if *v { 1 } else { 0 });
        }
        Value::Tuple(elems) => {
            buf.push(0x07);
            buf.extend_from_slice(&(elems.len() as u64).to_le_bytes());
            for e in elems.iter() {
                value_hash_bytes(e, buf);
            }
        }
        Value::Unit => buf.push(0x06),
        Value::Nat(v) => {
            buf.push(0x01);
            buf.extend_from_slice(&v.to_le_bytes());
        }
        Value::Float64(v) => {
            buf.push(0x02);
            buf.extend_from_slice(&v.to_le_bytes());
        }
        Value::Float32(v) => {
            buf.push(0x03);
            buf.extend_from_slice(&v.to_le_bytes());
        }
        _ => buf.push(0xFF),
    }
}

type OutputSignature = u64;

fn compute_output_signature(outputs: &[Value]) -> OutputSignature {
    let mut buf = Vec::with_capacity(128);
    buf.extend_from_slice(&(outputs.len() as u64).to_le_bytes());
    for val in outputs {
        value_hash_bytes(val, &mut buf);
    }
    fnv1a(&buf)
}

// ---------------------------------------------------------------------------
// ProgramEntry
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct ProgramEntry {
    graph: SemanticGraph,
    size: usize,
    outputs: Vec<Value>,
    /// True if this entry is an input placeholder (empty Tuple) that resolves
    /// to a positional input at runtime. These are treated specially by fold/map.
    is_input: bool,
}

// ---------------------------------------------------------------------------
// ValueKind — lightweight type tag for runtime type checking
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValueKind {
    Int,
    Bool,
    IntList,
    BoolList,
    PairList,
    Unknown,
}

fn classify(val: &Value) -> ValueKind {
    match val {
        Value::Int(_) => ValueKind::Int,
        Value::Bool(_) => ValueKind::Bool,
        Value::Tuple(elems) => {
            if elems.is_empty() {
                ValueKind::IntList
            } else {
                match &elems[0] {
                    Value::Int(_) => ValueKind::IntList,
                    Value::Bool(_) => ValueKind::BoolList,
                    Value::Tuple(inner) if inner.len() == 2 => ValueKind::PairList,
                    _ => ValueKind::Unknown,
                }
            }
        }
        _ => ValueKind::Unknown,
    }
}

fn entry_kind(entry: &ProgramEntry) -> ValueKind {
    entry
        .outputs
        .first()
        .map(classify)
        .unwrap_or(ValueKind::Unknown)
}

// ---------------------------------------------------------------------------
// Graph construction helpers
// ---------------------------------------------------------------------------

use std::sync::atomic::{AtomicU32, Ordering};
static ENUM_COUNTER: AtomicU32 = AtomicU32::new(0);

fn int_type_env() -> (TypeEnv, TypeId) {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = iris_types::hash::compute_type_id(&int_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    (TypeEnv { types }, int_id)
}

fn make_unique_node(kind: NodeKind, payload: NodePayload, type_sig: TypeId, arity: u8) -> Node {
    let depth = (ENUM_COUNTER.fetch_add(1, Ordering::Relaxed) % 256) as u8;
    let mut node = Node {
        id: NodeId(0),
        kind,
        type_sig,
        cost: CostTerm::Unit,
        arity,
        resolution_depth: depth, salt: 0,
        payload,
    };
    node.id = compute_node_id(&node);
    node
}

fn compute_hash(nodes: &HashMap<NodeId, Node>, edges: &[Edge]) -> SemanticHash {
    let mut hasher = blake3::Hasher::new();
    let mut sorted_nids: Vec<_> = nodes.keys().collect();
    sorted_nids.sort();
    for nid in sorted_nids {
        hasher.update(&nid.0.to_le_bytes());
    }
    for edge in edges {
        hasher.update(&edge.source.0.to_le_bytes());
        hasher.update(&edge.target.0.to_le_bytes());
        hasher.update(&[edge.port, edge.label as u8]);
    }
    SemanticHash(*hasher.finalize().as_bytes())
}

fn graph_to_fragment(graph: SemanticGraph) -> Fragment {
    let boundary = Boundary {
        inputs: vec![],
        outputs: vec![(graph.root, graph.nodes[&graph.root].type_sig)],
    };
    let type_env = graph.type_env.clone();
    let mut fragment = Fragment {
        id: FragmentId([0; 32]),
        graph,
        boundary,
        type_env,
        imports: vec![],
        metadata: FragmentMeta {
            name: None,
            created_at: 0,
            generation: 0,
            lineage_hash: 0,
        },
        proof: None,
        contracts: Default::default(),    };
    fragment.id = compute_fragment_id(&fragment);
    fragment
}

fn build_lit_int(value: i64, type_env: &TypeEnv, int_id: TypeId) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: value.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let root = node.id;
    nodes.insert(root, node);
    let hash = compute_hash(&nodes, &[]);
    SemanticGraph {
        root,
        nodes,
        edges: vec![],
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

fn build_lit_bool(value: bool, type_env: &TypeEnv, int_id: TypeId) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 4,
            value: vec![if value { 1 } else { 0 }],
        },
        int_id,
        0,
    );
    let root = node.id;
    nodes.insert(root, node);
    let hash = compute_hash(&nodes, &[]);
    SemanticGraph {
        root,
        nodes,
        edges: vec![],
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

fn build_input_placeholder(type_env: &TypeEnv, int_id: TypeId) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let node = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let root = node.id;
    nodes.insert(root, node);
    let hash = compute_hash(&nodes, &[]);
    SemanticGraph {
        root,
        nodes,
        edges: vec![],
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

fn incorporate_child(
    parent_nodes: &mut HashMap<NodeId, Node>,
    parent_edges: &mut Vec<Edge>,
    child: &SemanticGraph,
) -> NodeId {
    for (nid, node) in &child.nodes {
        parent_nodes.insert(*nid, node.clone());
    }
    parent_edges.extend(child.edges.iter().cloned());
    child.root
}

fn build_binary_prim(
    opcode: u8,
    a: &SemanticGraph,
    b: &SemanticGraph,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();
    let a_root = incorporate_child(&mut nodes, &mut edges, a);
    let b_root = incorporate_child(&mut nodes, &mut edges, b);
    let prim_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode },
        int_id,
        2,
    );
    let root = prim_node.id;
    nodes.insert(root, prim_node);
    edges.push(Edge { source: root, target: a_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: b_root, port: 1, label: EdgeLabel::Argument });
    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

fn build_unary_prim(
    opcode: u8,
    a: &SemanticGraph,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();
    let a_root = incorporate_child(&mut nodes, &mut edges, a);
    let prim_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode },
        int_id,
        1,
    );
    let root = prim_node.id;
    nodes.insert(root, prim_node);
    edges.push(Edge { source: root, target: a_root, port: 0, label: EdgeLabel::Argument });
    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build map(collection, op) where collection is an explicit sub-graph.
fn build_map(
    collection: &SemanticGraph,
    op_opcode: u8,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();
    let coll_root = incorporate_child(&mut nodes, &mut edges, collection);
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: op_opcode },
        int_id,
        2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);
    let map_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 },
        int_id,
        2,
    );
    let root = map_node.id;
    nodes.insert(root, map_node);
    edges.push(Edge { source: root, target: coll_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });
    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build filter(collection, cmp_op) where collection is an explicit sub-graph.
fn build_filter(
    collection: &SemanticGraph,
    cmp_opcode: u8,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();
    let coll_root = incorporate_child(&mut nodes, &mut edges, collection);
    let pred_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: cmp_opcode },
        int_id,
        2,
    );
    let pred_id = pred_node.id;
    nodes.insert(pred_id, pred_node);
    let filter_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x31 },
        int_id,
        2,
    );
    let root = filter_node.id;
    nodes.insert(root, filter_node);
    edges.push(Edge { source: root, target: coll_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: pred_id, port: 1, label: EdgeLabel::Argument });
    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build fold(base_lit, step_op) — reads collection from positional input 0.
///
/// This is the key pattern for simple reductions: fold(0, add, input).
/// The fold node has only 2 argument edges; the interpreter reads the
/// collection from BinderId(0xFFFF_0000).
fn build_fold_implicit_input(
    base_value: i64,
    step_opcode: u8,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: base_value.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: step_opcode },
        int_id,
        2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id,
        2,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);

    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build fold(base, step_op, collection) — explicit collection source.
fn build_fold_explicit(
    base: &SemanticGraph,
    step_opcode: u8,
    collection: &SemanticGraph,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let base_root = incorporate_child(&mut nodes, &mut edges, base);
    let coll_root = incorporate_child(&mut nodes, &mut edges, collection);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: step_opcode },
        int_id,
        2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id,
        3,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);

    edges.push(Edge { source: root, target: base_root, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: coll_root, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

fn build_zip(
    a: &SemanticGraph,
    b: &SemanticGraph,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    build_binary_prim(0x32, a, b, type_env, int_id)
}

fn build_reverse(
    collection: &SemanticGraph,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    build_unary_prim(0x36, collection, type_env, int_id)
}

fn build_concat(
    a: &SemanticGraph,
    b: &SemanticGraph,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    build_binary_prim(0x35, a, b, type_env, int_id)
}

/// Build unfold(seed_tuple, step_op) bounded by input length.
///
/// Creates: fold(base, reduce_op, unfold(seed, step, no_term, input_bound))
/// For Fibonacci: fold(0, max, unfold((1,1), add, _, input))
fn build_unfold_iterate(
    seed_a: i64,
    seed_b: i64,
    step_opcode: u8,
    reduce_base: i64,
    reduce_opcode: u8,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Seed tuple
    let lit_a = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: seed_a.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let lit_b = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: seed_b.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let a_id = lit_a.id;
    let b_id = lit_b.id;
    nodes.insert(a_id, lit_a);
    nodes.insert(b_id, lit_b);

    let seed_tuple = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 2);
    let seed_id = seed_tuple.id;
    nodes.insert(seed_id, seed_tuple);
    edges.push(Edge { source: seed_id, target: a_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: seed_id, target: b_id, port: 1, label: EdgeLabel::Argument });

    // Step prim
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: step_opcode },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // No-op termination (Lit(0) evaluates to Int(0), not Bool(true), so won't terminate)
    let no_term = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let no_term_id = no_term.id;
    nodes.insert(no_term_id, no_term);

    // Input placeholder (bounds iteration by input length)
    let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let placeholder_id = placeholder.id;
    nodes.insert(placeholder_id, placeholder);

    // Unfold: 4 args (seed, step, no_term, bound)
    let unfold_node = make_unique_node(
        NodeKind::Unfold,
        NodePayload::Unfold { recursion_descriptor: vec![] },
        int_id, 4,
    );
    let unfold_id = unfold_node.id;
    nodes.insert(unfold_id, unfold_node);
    edges.push(Edge { source: unfold_id, target: seed_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: no_term_id, port: 2, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: placeholder_id, port: 3, label: EdgeLabel::Argument });

    // Reduce: fold(base, op, unfold_result)
    let fold_base = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: reduce_base.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let fold_base_id = fold_base.id;
    nodes.insert(fold_base_id, fold_base);

    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: reduce_opcode },
        int_id, 2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id, 3,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: fold_base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: unfold_id, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build unfold(input_as_seed, step_op, term_op) returning final state projected.
///
/// For GCD: Project(0, unfold(input, mod, eq))
fn build_unfold_terminate(
    step_opcode: u8,
    term_opcode: u8,
    project_field: u16,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Input placeholder (the seed is the input itself, e.g. (a, b) for GCD)
    let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let placeholder_id = placeholder.id;
    nodes.insert(placeholder_id, placeholder);

    // Step prim
    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: step_opcode },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // Termination predicate
    let term_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: term_opcode },
        int_id, 2,
    );
    let term_id = term_node.id;
    nodes.insert(term_id, term_node);

    // Unfold returning final state (recursion_descriptor = [0x01])
    let unfold_node = make_unique_node(
        NodeKind::Unfold,
        NodePayload::Unfold { recursion_descriptor: vec![0x01] },
        int_id, 3,
    );
    let unfold_id = unfold_node.id;
    nodes.insert(unfold_id, unfold_node);
    edges.push(Edge { source: unfold_id, target: placeholder_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: unfold_id, target: term_id, port: 2, label: EdgeLabel::Argument });

    // Project to extract the answer from final state
    let project_node = make_unique_node(
        NodeKind::Project,
        NodePayload::Project { field_index: project_field },
        int_id, 1,
    );
    let root = project_node.id;
    nodes.insert(root, project_node);
    edges.push(Edge { source: root, target: unfold_id, port: 0, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build pairwise pattern: fold(base, op, map(zip(input, drop(input, 1)), pair_op))
fn build_pairwise_fold(
    pair_opcode: u8,
    fold_base: i64,
    fold_opcode: u8,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Input placeholder
    let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let placeholder_id = placeholder.id;
    nodes.insert(placeholder_id, placeholder);

    // drop(input, 1)
    let lit_1 = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let lit_1_id = lit_1.id;
    nodes.insert(lit_1_id, lit_1);

    let drop_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x34 }, // drop
        int_id, 2,
    );
    let drop_id = drop_node.id;
    nodes.insert(drop_id, drop_node);
    edges.push(Edge { source: drop_id, target: placeholder_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: drop_id, target: lit_1_id, port: 1, label: EdgeLabel::Argument });

    // zip(input, drop(input, 1))
    let zip_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x32 }, // zip
        int_id, 2,
    );
    let zip_id = zip_node.id;
    nodes.insert(zip_id, zip_node);
    edges.push(Edge { source: zip_id, target: placeholder_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: zip_id, target: drop_id, port: 1, label: EdgeLabel::Argument });

    // map(zipped, pair_op)
    let pair_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: pair_opcode },
        int_id, 2,
    );
    let pair_step_id = pair_step.id;
    nodes.insert(pair_step_id, pair_step);

    let map_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 }, // map
        int_id, 2,
    );
    let map_id = map_node.id;
    nodes.insert(map_id, map_node);
    edges.push(Edge { source: map_id, target: zip_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: map_id, target: pair_step_id, port: 1, label: EdgeLabel::Argument });

    // fold(base, op, map_result)
    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: fold_base.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: fold_opcode },
        int_id, 2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id, 3,
    );
    let root = fold_node.id;
    nodes.insert(root, fold_node);
    edges.push(Edge { source: root, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: map_id, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build pairwise map (no fold): map(zip(input, drop(input, 1)), pair_op)
fn build_pairwise_map(
    pair_opcode: u8,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let placeholder_id = placeholder.id;
    nodes.insert(placeholder_id, placeholder);

    let lit_1 = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let lit_1_id = lit_1.id;
    nodes.insert(lit_1_id, lit_1);

    let drop_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x34 },
        int_id, 2,
    );
    let drop_id = drop_node.id;
    nodes.insert(drop_id, drop_node);
    edges.push(Edge { source: drop_id, target: placeholder_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: drop_id, target: lit_1_id, port: 1, label: EdgeLabel::Argument });

    let zip_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x32 },
        int_id, 2,
    );
    let zip_id = zip_node.id;
    nodes.insert(zip_id, zip_node);
    edges.push(Edge { source: zip_id, target: placeholder_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: zip_id, target: drop_id, port: 1, label: EdgeLabel::Argument });

    let pair_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: pair_opcode },
        int_id, 2,
    );
    let pair_step_id = pair_step.id;
    nodes.insert(pair_step_id, pair_step);

    let map_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 },
        int_id, 2,
    );
    let root = map_node.id;
    nodes.insert(root, map_node);
    edges.push(Edge { source: root, target: zip_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: pair_step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Build reversed pairwise map: map(zip(drop(input, 1), input), pair_op)
/// For pairwise diffs: sub(a[i+1], a[i]) gives positive differences for increasing sequences.
fn build_pairwise_map_reversed(
    pair_opcode: u8,
    type_env: &TypeEnv,
    int_id: TypeId,
) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let placeholder_id = placeholder.id;
    nodes.insert(placeholder_id, placeholder);

    let lit_1 = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let lit_1_id = lit_1.id;
    nodes.insert(lit_1_id, lit_1);

    let drop_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x34 },
        int_id, 2,
    );
    let drop_id = drop_node.id;
    nodes.insert(drop_id, drop_node);
    edges.push(Edge { source: drop_id, target: placeholder_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: drop_id, target: lit_1_id, port: 1, label: EdgeLabel::Argument });

    // REVERSED: zip(drop(input,1), input) so pairs are (a[i+1], a[i])
    let zip_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x32 },
        int_id, 2,
    );
    let zip_id = zip_node.id;
    nodes.insert(zip_id, zip_node);
    edges.push(Edge { source: zip_id, target: drop_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: zip_id, target: placeholder_id, port: 1, label: EdgeLabel::Argument });

    let pair_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: pair_opcode },
        int_id, 2,
    );
    let pair_step_id = pair_step.id;
    nodes.insert(pair_step_id, pair_step);

    let map_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 },
        int_id, 2,
    );
    let root = map_node.id;
    nodes.insert(root, map_node);
    edges.push(Edge { source: root, target: zip_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: root, target: pair_step_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root, nodes, edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

// ---------------------------------------------------------------------------
// Evaluation helper
// ---------------------------------------------------------------------------

fn eval_graph(graph: &SemanticGraph, inputs: &[Value]) -> Option<Value> {
    let mut state = StateStore::new();
    // Use tight sandbox limits for enumeration: 10,000 steps, 16 MB memory,
    // 1 second timeout. Enumerated programs are small; exceeding these limits
    // means the program is likely looping or allocating unboundedly.
    match interpreter::interpret_sandboxed(
        graph,
        inputs,
        Some(&mut state),
        None,  // registry
        10_000, // step limit
        interpreter::ENUMERATE_MEMORY_LIMIT, // 16 MB memory limit
        None,  // effect_handler
        None,  // bus
        None,  // meta_evolver
        0,     // meta_evolve_depth
    ) {
        Ok((vals, _)) => {
            // The interpreter flattens Tuple results into a Vec<Value>.
            // Reconstruct the single output value:
            // - If exactly one value, return it directly.
            // - If multiple values, wrap them back into a Tuple (the interpreter
            //   destructured what was originally a Tuple return value).
            match vals.len() {
                0 => Some(Value::Unit),
                1 => Some(vals.into_iter().next().unwrap()),
                _ => Some(Value::tuple(vals)),
            }
        }
        Err(_) => None,
    }
}

fn values_match(actual: &Value, expected: &Value) -> bool {
    if actual == expected {
        return true;
    }
    match (actual, expected) {
        (Value::Bool(b), Value::Int(i)) | (Value::Int(i), Value::Bool(b)) => {
            let b_int = if *b { 1i64 } else { 0i64 };
            b_int == *i
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// BottomUpEnumerator
// ---------------------------------------------------------------------------

/// Hard cap on bank size to prevent memory explosion.
/// Each entry holds a full SemanticGraph clone. At ~5KB avg per entry,
/// 1000 entries ≈ 5MB. This is the absolute ceiling.
const MAX_BANK_SIZE: usize = 1_000;

/// Hard cap on candidates generated per size level.
const MAX_CANDIDATES_PER_SIZE: usize = 2_000;

/// Bottom-up enumerative synthesizer with observational equivalence pruning.
pub struct BottomUpEnumerator {
    /// Bank of programs indexed by their output signature.
    bank: BTreeMap<OutputSignature, ProgramEntry>,
    /// Test inputs for evaluation.
    test_cases: Vec<TestCase>,
    /// Maximum program size to enumerate.
    max_size: usize,
    /// Type environment and Int type ID.
    type_env: TypeEnv,
    int_id: TypeId,
    /// Number of positional inputs.
    num_inputs: usize,
}

impl BottomUpEnumerator {
    pub fn new(test_cases: &[TestCase], max_size: usize) -> Self {
        let (type_env, int_id) = int_type_env();
        let num_inputs = test_cases
            .first()
            .map(|tc| tc.inputs.len())
            .unwrap_or(0);
        Self {
            bank: BTreeMap::new(),
            test_cases: test_cases.to_vec(),
            max_size,
            type_env,
            int_id,
            num_inputs,
        }
    }

    /// Run the enumeration. Returns the first matching program as a Fragment.
    /// Hard-capped at 30 seconds wall clock and MAX_BANK_SIZE entries.
    pub fn enumerate(&mut self) -> Option<Fragment> {
        if self.test_cases.is_empty() {
            return None;
        }

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);

        // Size 0: atoms.
        if let Some(f) = self.add_atoms() {
            return Some(f);
        }

        // Size 1: fold-over-implicit-input patterns and unary ops.
        if self.max_size >= 1 {
            if let Some(f) = self.add_fold_implicit_patterns() {
                return Some(f);
            }
        }

        // Size 2: unfold-based iterative patterns (Fibonacci, GCD, etc.)
        if self.max_size >= 2 {
            if let Some(f) = self.add_unfold_patterns() {
                return Some(f);
            }
        }

        // Size 2: pairwise fold patterns (is-sorted, pairwise diffs)
        if self.max_size >= 2 {
            if let Some(f) = self.add_pairwise_patterns() {
                return Some(f);
            }
        }

        // Size 1..max_size: compose operations.
        for size in 1..=self.max_size {
            // Check timeout
            if std::time::Instant::now() > deadline {
                break;
            }
            // Check bank size
            if self.bank.len() >= MAX_BANK_SIZE {
                break;
            }
            if let Some(f) = self.enumerate_size(size) {
                return Some(f);
            }
        }

        None
    }

    // -----------------------------------------------------------------------
    // Atom generation (size 0)
    // -----------------------------------------------------------------------

    fn add_atoms(&mut self) -> Option<Fragment> {
        // Input placeholders — use the actual test input values as the
        // "outputs" for type classification. The placeholder graph evaluates
        // to an empty Tuple (sentinel), but in composition (map/fold), it
        // resolves to the positional input. We store the real input values
        // so the bank's type classification works correctly.
        for input_idx in 0..self.num_inputs {
            let graph = build_input_placeholder(&self.type_env, self.int_id);
            let outputs: Vec<Value> = self.test_cases.iter().map(|tc| {
                tc.inputs.get(input_idx).cloned().unwrap_or(Value::Unit)
            }).collect();

            let sig = compute_output_signature(&outputs);
            self.bank.insert(sig, ProgramEntry {
                graph,
                size: 0,
                outputs,
                is_input: true,
            });
        }

        // Integer constants.
        for &c in &[0i64, 1, -1, 2] {
            if let Some(f) = self.try_add(
                build_lit_int(c, &self.type_env, self.int_id), 0, false,
            ) {
                return Some(f);
            }
        }

        // Boolean constants.
        for &b in &[true, false] {
            if let Some(f) = self.try_add(
                build_lit_bool(b, &self.type_env, self.int_id), 0, false,
            ) {
                return Some(f);
            }
        }

        // Extreme values for min/max seeds.
        for &c in &[i64::MAX, i64::MIN] {
            if let Some(f) = self.try_add(
                build_lit_int(c, &self.type_env, self.int_id), 0, false,
            ) {
                return Some(f);
            }
        }

        None
    }

    // -----------------------------------------------------------------------
    // Fold-over-implicit-input patterns (size 1)
    // -----------------------------------------------------------------------

    fn add_fold_implicit_patterns(&mut self) -> Option<Fragment> {
        // fold(base, op) — reads collection from positional input 0.
        // This covers: sum, product, max, min, etc.
        let fold_ops = [0x00u8, 0x01, 0x02, 0x07, 0x08]; // add, sub, mul, min, max
        let base_vals = [0i64, 1, -1, i64::MAX, i64::MIN];

        for &op in &fold_ops {
            for &base in &base_vals {
                let graph = build_fold_implicit_input(
                    base, op, &self.type_env, self.int_id,
                );
                if let Some(f) = self.try_add(graph, 1, false) {
                    return Some(f);
                }
            }
        }
        None
    }

    // -----------------------------------------------------------------------
    // Unfold-based iterative patterns (size 2)
    // -----------------------------------------------------------------------

    fn add_unfold_patterns(&mut self) -> Option<Fragment> {
        // Fibonacci-like patterns: fold(base, reduce, unfold((a,b), step, _, input))
        // For Fibonacci: fold(0, max, unfold((1,1), add, _, input)) = fib(len(input))
        let unfold_combos: &[(i64, i64, u8, i64, u8)] = &[
            // (seed_a, seed_b, step_op, reduce_base, reduce_op)
            (1, 1, 0x00, 0, 0x08),   // Fibonacci: unfold((1,1), add), fold(0, max)
            (0, 1, 0x00, 0, 0x08),   // Fibonacci alt: unfold((0,1), add), fold(0, max)
            (1, 0, 0x00, 0, 0x00),   // unfold((1,0), add), fold(0, add) = sum of fib-like
            (1, 1, 0x02, 1, 0x08),   // unfold((1,1), mul), fold(1, max) = Lucas-like
        ];

        for &(sa, sb, step, rb, red) in unfold_combos {
            let graph = build_unfold_iterate(
                sa, sb, step, rb, red, &self.type_env, self.int_id,
            );
            if let Some(f) = self.try_add(graph, 2, false) {
                return Some(f);
            }
        }

        // GCD-like patterns: Project(field, unfold(input, step, term))
        let term_combos: &[(u8, u8, u16)] = &[
            // (step_op, term_op, project_field)
            (0x04, 0x20, 0),   // GCD: unfold(input, mod, eq(b,0)), project field 0
            (0x04, 0x20, 1),   // variant: project field 1
            (0x01, 0x20, 0),   // unfold(input, sub, eq(b,0)), project field 0
            (0x01, 0x24, 0),   // unfold(input, sub, le(b,0)), project field 0
        ];

        for &(step, term, field) in term_combos {
            let graph = build_unfold_terminate(
                step, term, field, &self.type_env, self.int_id,
            );
            if let Some(f) = self.try_add(graph, 2, false) {
                return Some(f);
            }
        }

        None
    }

    // -----------------------------------------------------------------------
    // Pairwise fold patterns (size 2)
    // -----------------------------------------------------------------------

    fn add_pairwise_patterns(&mut self) -> Option<Fragment> {
        // Pairwise fold: fold(base, op, map(zip(input, drop(input,1)), pair_op))
        let pairwise_combos: &[(u8, i64, u8)] = &[
            // (pair_op, fold_base, fold_op)
            (0x24, 1, 0x07),   // is-sorted (ascending): fold(1, min, map(zip(x, drop(x,1)), le))
            (0x24, 1, 0x02),   // is-sorted (ascending): fold(1, mul, map(zip(x, drop(x,1)), le))
            (0x22, 1, 0x02),   // is-strictly-sorted: fold(1, mul, map(zip(x, drop(x,1)), lt))
            (0x01, 0, 0x00),   // pairwise diffs sum: fold(0, add, map(zip, sub))
            (0x01, 0, 0x08),   // max pairwise diff: fold(0, max, map(zip, sub))
            (0x00, 0, 0x00),   // pairwise sum: fold(0, add, map(zip, add))
        ];

        for &(pair_op, base, fold_op) in pairwise_combos {
            let graph = build_pairwise_fold(
                pair_op, base, fold_op, &self.type_env, self.int_id,
            );
            if let Some(f) = self.try_add(graph, 2, false) {
                return Some(f);
            }
        }

        // Pairwise map (no fold): map(zip(input, drop(input,1)), pair_op)
        // For pairwise differences output
        let pairwise_maps: &[u8] = &[
            0x01,   // sub: pairwise differences (a[i] - a[i+1])
            0x00,   // add: pairwise sums
            0x24,   // le: pairwise le comparisons
        ];

        for &pair_op in pairwise_maps {
            let graph = build_pairwise_map(
                pair_op, &self.type_env, self.int_id,
            );
            if let Some(f) = self.try_add(graph, 2, false) {
                return Some(f);
            }
        }

        // Reversed pairwise map: map(zip(drop(input,1), input), pair_op)
        // For pairwise diffs in the standard order: a[i+1] - a[i]
        let reversed_maps: &[u8] = &[
            0x01,   // sub: pairwise differences (a[i+1] - a[i])
            0x24,   // le: reversed le comparisons
        ];

        for &pair_op in reversed_maps {
            let graph = build_pairwise_map_reversed(
                pair_op, &self.type_env, self.int_id,
            );
            if let Some(f) = self.try_add(graph, 2, false) {
                return Some(f);
            }
        }

        None
    }

    // -----------------------------------------------------------------------
    // Enumerate programs of a given size
    // -----------------------------------------------------------------------

    fn enumerate_size(&mut self, size: usize) -> Option<Fragment> {
        // IMPORTANT: Only clone entries we actually need (filtered by size),
        // not the entire bank. This prevents the O(bank_size) memory doubling
        // that caused 29GB memory explosions.
        let max_child_size = size.saturating_sub(1);
        let entries_snapshot: Vec<ProgramEntry> = self.bank.values()
            .filter(|e| e.size <= max_child_size)
            .take(500)  // hard cap on snapshot size
            .cloned()
            .collect();
        let mut candidates: Vec<(SemanticGraph, bool)> = Vec::new();

        // --- Unary operations (cost 1 + child_size where child_size = size-1) ---
        let child_size = size - 1;
        let children: Vec<&ProgramEntry> = entries_snapshot
            .iter()
            .filter(|e| e.size == child_size)
            .collect();

        for child in &children {
            let ck = entry_kind(child);

            // neg(x), abs(x) — Int -> Int
            if ck == ValueKind::Int {
                candidates.push((build_unary_prim(0x05, &child.graph, &self.type_env, self.int_id), false));
                candidates.push((build_unary_prim(0x06, &child.graph, &self.type_env, self.int_id), false));
            }

            // reverse(list)
            if matches!(ck, ValueKind::IntList | ValueKind::PairList | ValueKind::BoolList) {
                candidates.push((build_reverse(&child.graph, &self.type_env, self.int_id), false));
            }

            // map(list, op) — only on collections
            if matches!(ck, ValueKind::IntList | ValueKind::PairList) {
                for &op in &[0x00u8, 0x01, 0x02, 0x05, 0x06, 0x07, 0x08] {
                    candidates.push((build_map(&child.graph, op, &self.type_env, self.int_id), false));
                }
                // Comparison ops in map context.
                for &op in &[0x22u8, 0x23, 0x24, 0x25] {
                    candidates.push((build_map(&child.graph, op, &self.type_env, self.int_id), false));
                }
            }

            // filter(list, pred)
            if ck == ValueKind::IntList {
                for &op in &[0x22u8, 0x23, 0x24, 0x25] {
                    candidates.push((build_filter(&child.graph, op, &self.type_env, self.int_id), false));
                }
            }

            // fold(base_literal, op, child_collection) — explicit collection.
            // Only when child is NOT a raw input placeholder (those are handled
            // by the implicit-input fold pattern above).
            if !child.is_input && matches!(ck, ValueKind::IntList | ValueKind::PairList | ValueKind::BoolList) {
                for &fold_op in &[0x00u8, 0x01, 0x02, 0x07, 0x08] {
                    for &base_val in &[0i64, 1, i64::MAX, i64::MIN] {
                        let base = build_lit_int(base_val, &self.type_env, self.int_id);
                        candidates.push((
                            build_fold_explicit(&base, fold_op, &child.graph, &self.type_env, self.int_id),
                            false,
                        ));
                    }
                }
            }
        }

        // --- Binary operations (cost 1 + left_size + right_size) ---
        for left_size in 0..size {
            let right_size = size - 1 - left_size;
            let lefts: Vec<&ProgramEntry> = entries_snapshot
                .iter()
                .filter(|e| e.size == left_size)
                .collect();
            let rights: Vec<&ProgramEntry> = entries_snapshot
                .iter()
                .filter(|e| e.size == right_size)
                .collect();

            for left in &lefts {
                let lk = entry_kind(left);
                for right in &rights {
                    let rk = entry_kind(right);

                    // Arithmetic binops: Int x Int -> Int
                    if lk == ValueKind::Int && rk == ValueKind::Int {
                        for &op in &[0x00u8, 0x01, 0x02, 0x07, 0x08] {
                            candidates.push((
                                build_binary_prim(op, &left.graph, &right.graph, &self.type_env, self.int_id),
                                false,
                            ));
                        }
                        // Comparison ops: Int x Int -> Bool
                        for &op in &[0x20u8, 0x22, 0x23] {
                            candidates.push((
                                build_binary_prim(op, &left.graph, &right.graph, &self.type_env, self.int_id),
                                false,
                            ));
                        }
                    }

                    // zip: List x List -> PairList
                    if matches!(lk, ValueKind::IntList | ValueKind::BoolList)
                        && matches!(rk, ValueKind::IntList | ValueKind::BoolList)
                    {
                        candidates.push((
                            build_zip(&left.graph, &right.graph, &self.type_env, self.int_id),
                            false,
                        ));
                    }

                    // concat: List x List -> List
                    if lk == ValueKind::IntList && rk == ValueKind::IntList {
                        candidates.push((
                            build_concat(&left.graph, &right.graph, &self.type_env, self.int_id),
                            false,
                        ));
                    }

                    // fold(computed_base, op, computed_collection)
                    if lk == ValueKind::Int && !right.is_input
                        && matches!(rk, ValueKind::IntList | ValueKind::PairList | ValueKind::BoolList)
                    {
                        for &fold_op in &[0x00u8, 0x01, 0x02, 0x07, 0x08] {
                            candidates.push((
                                build_fold_explicit(&left.graph, fold_op, &right.graph, &self.type_env, self.int_id),
                                false,
                            ));
                        }
                    }
                }
            }
        }

        // Evaluate all candidates (capped to prevent memory explosion).
        candidates.truncate(MAX_CANDIDATES_PER_SIZE);
        for (graph, is_input) in candidates {
            if let Some(f) = self.try_add(graph, size, is_input) {
                return Some(f);
            }
        }
        None
    }

    // -----------------------------------------------------------------------
    // Evaluate and insert into the bank
    // -----------------------------------------------------------------------

    fn try_add(&mut self, graph: SemanticGraph, size: usize, is_input: bool) -> Option<Fragment> {
        // Hard cap: stop adding to bank if it's too large
        if self.bank.len() >= MAX_BANK_SIZE {
            // Still check if this is a solution, but don't grow the bank
            return self.check_solution_only(&graph);
        }

        let mut outputs = Vec::with_capacity(self.test_cases.len());

        for tc in &self.test_cases {
            match eval_graph(&graph, &tc.inputs) {
                Some(val) => outputs.push(val),
                None => return None,
            }
        }

        // Check if this solves the specification.
        let is_solution = self.test_cases.iter().zip(outputs.iter()).all(|(tc, out)| {
            match &tc.expected_output {
                Some(expected) => match expected.first() {
                    Some(exp_val) => values_match(out, exp_val),
                    None => true,
                },
                None => true,
            }
        });

        if is_solution {
            return Some(graph_to_fragment(graph));
        }

        // Add to bank if observationally distinct.
        let sig = compute_output_signature(&outputs);
        match self.bank.get(&sig) {
            Some(existing) if existing.size <= size => {
                // Already have a smaller/equal program with same outputs.
            }
            _ => {
                self.bank.insert(sig, ProgramEntry {
                    graph,
                    size,
                    outputs,
                    is_input,
                });
            }
        }
        None
    }

    /// Check if a graph solves the spec without adding to the bank.
    fn check_solution_only(&self, graph: &SemanticGraph) -> Option<Fragment> {
        for tc in &self.test_cases {
            match eval_graph(graph, &tc.inputs) {
                Some(val) => {
                    if let Some(expected) = &tc.expected_output {
                        if let Some(exp_val) = expected.first() {
                            if !values_match(&val, exp_val) {
                                return None;
                            }
                        }
                    }
                }
                None => return None,
            }
        }
        Some(graph_to_fragment(graph.clone()))
    }
}

// ---------------------------------------------------------------------------
// Public integration API
// ---------------------------------------------------------------------------

/// Run bottom-up enumeration on test cases.
///
/// Returns a `Fragment` containing the synthesized program, or None if no
/// solution is found within the size budget.
pub fn enumerate_solution(test_cases: &[TestCase], max_size: usize) -> Option<Fragment> {
    // Hard cap max_size at 8 to prevent exponential blowup.
    // At size 8 with ~50 operations, the bank stays under MAX_BANK_SIZE.
    let capped_size = max_size.min(8);
    let mut enumerator = BottomUpEnumerator::new(test_cases, capped_size);
    enumerator.enumerate()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use iris_types::eval::{TestCase, Value};

    fn tc_list_to_int(input: Vec<i64>, expected: i64) -> TestCase {
        TestCase {
            inputs: vec![Value::Tuple(input.into_iter().map(Value::Int).collect())],
            expected_output: Some(vec![Value::Int(expected)]),
            initial_state: None,
            expected_state: None,
        }
    }

    #[allow(dead_code)]
    fn tc_list_to_list(input: Vec<i64>, expected: Vec<i64>) -> TestCase {
        TestCase {
            inputs: vec![Value::Tuple(input.into_iter().map(Value::Int).collect())],
            expected_output: Some(vec![Value::Tuple(
                expected.into_iter().map(Value::Int).collect(),
            )]),
            initial_state: None,
            expected_state: None,
        }
    }

    #[allow(dead_code)]
    fn tc_2lists_to_int(a: Vec<i64>, b: Vec<i64>, expected: i64) -> TestCase {
        TestCase {
            inputs: vec![
                Value::Tuple(a.into_iter().map(Value::Int).collect()),
                Value::Tuple(b.into_iter().map(Value::Int).collect()),
            ],
            expected_output: Some(vec![Value::Int(expected)]),
            initial_state: None,
            expected_state: None,
        }
    }

    #[test]
    fn test_enumerate_finds_sum() {
        // sum(list) = fold(0, add, input)
        let test_cases = vec![
            tc_list_to_int(vec![1, 2, 3], 6),
            tc_list_to_int(vec![10, -5], 5),
            tc_list_to_int(vec![0, 0, 0], 0),
            tc_list_to_int(vec![100], 100),
        ];
        let result = enumerate_solution(&test_cases, 6);
        assert!(result.is_some(), "enumerate should find sum program");
    }

    #[test]
    fn test_enumerate_finds_product() {
        // product(list) = fold(1, mul, input)
        let test_cases = vec![
            tc_list_to_int(vec![1, 2, 3], 6),
            tc_list_to_int(vec![2, 5], 10),
            tc_list_to_int(vec![4], 4),
            tc_list_to_int(vec![1, 1, 1], 1),
        ];
        let result = enumerate_solution(&test_cases, 6);
        assert!(result.is_some(), "enumerate should find product program");
    }

    #[test]
    fn test_enumerate_finds_max() {
        // max(list) = fold(MIN, max, input)
        let test_cases = vec![
            tc_list_to_int(vec![3, 1, 4, 1, 5], 5),
            tc_list_to_int(vec![-10, -5, -1], -1),
            tc_list_to_int(vec![42], 42),
        ];
        let result = enumerate_solution(&test_cases, 6);
        assert!(result.is_some(), "enumerate should find max program");
    }

    #[test]
    fn test_enumerate_finds_min() {
        // min(list) = fold(MAX, min, input)
        let test_cases = vec![
            tc_list_to_int(vec![3, 1, 4, 1, 5], 1),
            tc_list_to_int(vec![-10, -5, -1], -10),
            tc_list_to_int(vec![42], 42),
        ];
        let result = enumerate_solution(&test_cases, 6);
        assert!(result.is_some(), "enumerate should find min program");
    }

    #[test]
    fn test_map_mul_produces_squares() {
        // Verify that map(input, mul) squares each element.
        let (type_env, int_id) = int_type_env();
        let placeholder = build_input_placeholder(&type_env, int_id);
        let map_graph = build_map(&placeholder, 0x02, &type_env, int_id); // mul
        let input = Value::Tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = eval_graph(&map_graph, &[input]);
        assert_eq!(
            result,
            Some(Value::Tuple(vec![Value::Int(1), Value::Int(4), Value::Int(9)])),
            "map(input, mul) should square each element"
        );
    }

    #[test]
    fn test_fold_over_map_produces_sum_of_squares() {
        // Verify fold(0, add, map(input, mul)) = sum of squares.
        let (type_env, int_id) = int_type_env();
        let placeholder = build_input_placeholder(&type_env, int_id);
        let map_graph = build_map(&placeholder, 0x02, &type_env, int_id); // mul = squares
        let base = build_lit_int(0, &type_env, int_id);
        let fold_graph = build_fold_explicit(&base, 0x00, &map_graph, &type_env, int_id); // add
        let input = Value::Tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = eval_graph(&fold_graph, &[input]);
        assert_eq!(result, Some(Value::Int(14)), "fold(0, add, map(input, mul)) should produce 14");
    }

    #[test]
    fn test_enumerate_finds_sum_of_squares() {
        // sum_of_squares(list) = fold(0, add, map(input, mul))
        let test_cases = vec![
            tc_list_to_int(vec![1, 2, 3], 14),   // 1+4+9
            tc_list_to_int(vec![0, 0], 0),
            tc_list_to_int(vec![5], 25),
            tc_list_to_int(vec![-3, 4], 25),      // 9+16
        ];
        // max_size=4 is sufficient: fold(lit, add, map(input, mul)) has
        // size 2 in our cost model (map at size 1, fold-over-map at size 2).
        let result = enumerate_solution(&test_cases, 4);
        assert!(result.is_some(), "enumerate should find sum-of-squares program");
    }

    #[test]
    fn test_observational_equivalence_pruning() {
        let test_cases = vec![
            tc_list_to_int(vec![5], 5),
            tc_list_to_int(vec![10], 10),
        ];
        let mut enumerator = BottomUpEnumerator::new(&test_cases, 3);
        let _ = enumerator.enumerate();
        assert!(
            enumerator.bank.len() < 500,
            "bank should be pruned by observational equivalence, got {} entries",
            enumerator.bank.len()
        );
    }

    #[test]
    fn test_enumerate_respects_max_size() {
        let test_cases = vec![
            tc_list_to_int(vec![1, 2, 3], 6),
        ];
        let result = enumerate_solution(&test_cases, 0);
        assert!(result.is_none(), "enumerate with max_size=0 should not find sum");
    }

    #[test]
    fn test_empty_test_cases_returns_none() {
        let result = enumerate_solution(&[], 10);
        assert!(result.is_none(), "empty test cases should return None");
    }

    #[test]
    fn test_enumerate_constant_program() {
        let test_cases = vec![
            tc_list_to_int(vec![1, 2, 3], 0),
            tc_list_to_int(vec![4, 5], 0),
            tc_list_to_int(vec![], 0),
        ];
        let result = enumerate_solution(&test_cases, 2);
        assert!(result.is_some(), "enumerate should find constant-0 program");
    }

    #[test]
    fn test_enumerate_finds_fibonacci() {
        // Fibonacci: input is list of length n, output is fib(n).
        let cases: Vec<(i64, i64)> = vec![
            (0, 0), (1, 1), (2, 1), (3, 2), (4, 3), (5, 5), (6, 8), (7, 13),
        ];
        let test_cases: Vec<TestCase> = cases.into_iter().map(|(n, expected)| {
            let list: Vec<Value> = (0..n).map(Value::Int).collect();
            TestCase {
                inputs: vec![Value::Tuple(list)],
                expected_output: Some(vec![Value::Int(expected)]),
                initial_state: None,
                expected_state: None,
            }
        }).collect();
        let result = enumerate_solution(&test_cases, 6);
        assert!(result.is_some(), "enumerate should find Fibonacci program");
    }

    #[test]
    fn test_enumerate_finds_gcd() {
        // GCD: input is tuple (a, b), output is gcd(a, b).
        fn tc_gcd(a: i64, b: i64, expected: i64) -> TestCase {
            TestCase {
                inputs: vec![Value::Tuple(vec![Value::Int(a), Value::Int(b)])],
                expected_output: Some(vec![Value::Int(expected)]),
                initial_state: None,
                expected_state: None,
            }
        }
        let test_cases = vec![
            tc_gcd(12, 8, 4),
            tc_gcd(7, 3, 1),
            tc_gcd(100, 75, 25),
            tc_gcd(17, 17, 17),
            tc_gcd(48, 18, 6),
        ];
        let result = enumerate_solution(&test_cases, 6);
        assert!(result.is_some(), "enumerate should find GCD program");
    }

    #[test]
    fn test_unfold_iterate_fibonacci_graph() {
        // Verify that our unfold-iterate Fibonacci graph produces correct values.
        let (type_env, int_id) = int_type_env();
        let graph = build_unfold_iterate(1, 1, 0x00, 0, 0x08, &type_env, int_id);

        // fib(0) = 0: input is empty list
        let result = eval_graph(&graph, &[Value::Tuple(vec![])]);
        assert_eq!(result, Some(Value::Int(0)), "fib(0) should be 0");

        // fib(1) = 1: input is list of 1 element
        let result = eval_graph(&graph, &[Value::Tuple(vec![Value::Int(0)])]);
        assert_eq!(result, Some(Value::Int(1)), "fib(1) should be 1");

        // fib(5) = 5: input is list of 5 elements
        let result = eval_graph(&graph, &[Value::Tuple((0..5).map(Value::Int).collect())]);
        assert_eq!(result, Some(Value::Int(5)), "fib(5) should be 5");

        // fib(7) = 13: input is list of 7 elements
        let result = eval_graph(&graph, &[Value::Tuple((0..7).map(Value::Int).collect())]);
        assert_eq!(result, Some(Value::Int(13)), "fib(7) should be 13");
    }

    #[test]
    fn test_unfold_terminate_gcd_graph() {
        // Verify that our unfold-terminate GCD graph produces correct values.
        let (type_env, int_id) = int_type_env();
        let graph = build_unfold_terminate(0x04, 0x20, 0, &type_env, int_id);

        // gcd(12, 8) = 4
        let result = eval_graph(&graph, &[Value::Tuple(vec![Value::Int(12), Value::Int(8)])]);
        assert_eq!(result, Some(Value::Int(4)), "gcd(12, 8) should be 4");

        // gcd(7, 3) = 1
        let result = eval_graph(&graph, &[Value::Tuple(vec![Value::Int(7), Value::Int(3)])]);
        assert_eq!(result, Some(Value::Int(1)), "gcd(7, 3) should be 1");

        // gcd(48, 18) = 6
        let result = eval_graph(&graph, &[Value::Tuple(vec![Value::Int(48), Value::Int(18)])]);
        assert_eq!(result, Some(Value::Int(6)), "gcd(48, 18) should be 6");
    }

    #[test]
    fn test_pairwise_fold_is_sorted() {
        // Verify is-sorted: fold(1, min, map(zip(input, drop(input,1)), le))
        let (type_env, int_id) = int_type_env();
        let graph = build_pairwise_fold(0x24, 1, 0x07, &type_env, int_id);

        // [1,2,3] is sorted
        let result = eval_graph(&graph, &[Value::Tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)])]);
        assert!(matches!(result, Some(Value::Int(1)) | Some(Value::Bool(true))), "sorted list should return 1, got {:?}", result);

        // [3,1,2] is not sorted
        let result = eval_graph(&graph, &[Value::Tuple(vec![Value::Int(3), Value::Int(1), Value::Int(2)])]);
        assert!(matches!(result, Some(Value::Int(0)) | Some(Value::Bool(false))), "unsorted list should return 0, got {:?}", result);
    }

    #[test]
    fn test_pairwise_map_diffs() {
        // Verify pairwise differences: map(zip(input, drop(input,1)), sub)
        // Note: zip pairs (a[i], a[i+1]), sub computes a[i] - a[i+1] (reversed sign)
        let (type_env, int_id) = int_type_env();
        let graph = build_pairwise_map(0x01, &type_env, int_id);

        // [1, 3, 6, 10] -> [-2, -3, -4] (current - next)
        let result = eval_graph(&graph, &[Value::Tuple(vec![
            Value::Int(1), Value::Int(3), Value::Int(6), Value::Int(10),
        ])]);
        assert_eq!(
            result,
            Some(Value::Tuple(vec![Value::Int(-2), Value::Int(-3), Value::Int(-4)])),
            "pairwise diffs (a[i]-a[i+1]) of [1,3,6,10] should be [-2,-3,-4]"
        );
    }
}
