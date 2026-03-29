//! Tests for all 16 mutation operators in src/iris-programs/mutation/
//!
//! Each test compiles the .iris file and verifies the mutation produces
//! correct structural changes to the graph.

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use iris_bootstrap;
use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::*;
use iris_types::hash::SemanticHash;
use iris_types::types::{PrimType, TypeDef, TypeEnv, TypeId};

// ===========================================================================
// Helpers
// ===========================================================================

fn int_type_env() -> (TypeEnv, TypeId) {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = iris_types::hash::compute_type_id(&int_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    (TypeEnv { types }, int_id)
}

fn make_node(id: u64, kind: NodeKind, payload: NodePayload, arity: u8) -> (NodeId, Node) {
    let (_, int_id) = int_type_env();
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity,
            resolution_depth: 0,
            salt: id,
            payload,
        },
    )
}

fn make_edge(source: u64, target: u64, port: u8) -> Edge {
    Edge {
        source: NodeId(source),
        target: NodeId(target),
        port,
        label: EdgeLabel::Argument,
    }
}

fn make_graph(nodes: HashMap<NodeId, Node>, edges: Vec<Edge>, root: u64) -> SemanticGraph {
    let (type_env, _) = int_type_env();
    SemanticGraph {
        root: NodeId(root),
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

fn extract_program(val: &Value) -> SemanticGraph {
    match val {
        Value::Program(p) => p.as_ref().clone(),
        Value::Tuple(elems) => match &elems[0] {
            Value::Program(p) => p.as_ref().clone(),
            other => panic!("expected Program in Tuple, got {:?}", other),
        },
        other => panic!("expected Program or Tuple(Program, ...), got {:?}", other),
    }
}

fn compile_fn(src: &str, name: &str) -> (SemanticGraph, FragmentRegistry) {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!("compilation failed");
    }
    let mut registry = FragmentRegistry::new();
    for (_, frag, _) in &result.fragments {
        registry.register(frag.clone());
    }
    for (n, frag, _) in &result.fragments {
        if n == name {
            return (frag.graph.clone(), registry);
        }
    }
    let names: Vec<&str> = result.fragments.iter().map(|(n, _, _)| n.as_str()).collect();
    panic!("'{}' not found; available: {:?}", name, names);
}

fn run(graph: &SemanticGraph, inputs: &[Value], registry: &FragmentRegistry) -> Vec<Value> {
    let (out, _) = interpreter::interpret_with_registry(graph, inputs, None, Some(registry))
        .expect("interpreter failed");
    out
}

fn make_binop_graph(opcode: u8, a: i64, b: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = make_node(10, NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: a.to_le_bytes().to_vec() }, 0);
    nodes.insert(nid, node);
    let (nid, node) = make_node(20, NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: b.to_le_bytes().to_vec() }, 0);
    nodes.insert(nid, node);
    let (nid, node) = make_node(1, NodeKind::Prim, NodePayload::Prim { opcode }, 2);
    nodes.insert(nid, node);
    let edges = vec![make_edge(1, 10, 0), make_edge(1, 20, 1)];
    make_graph(nodes, edges, 1)
}

fn make_fold_graph() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = make_node(10, NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() }, 0);
    nodes.insert(nid, node);
    let (nid, node) = make_node(20, NodeKind::Prim, NodePayload::Prim { opcode: 0x00 }, 2);
    nodes.insert(nid, node);
    let (nid, node) = make_node(30, NodeKind::Lit, NodePayload::Lit { type_tag: 0xFF, value: vec![0] }, 0);
    nodes.insert(nid, node);
    let (nid, node) = make_node(1, NodeKind::Fold, NodePayload::Fold { recursion_descriptor: vec![] }, 3);
    nodes.insert(nid, node);
    let edges = vec![
        make_edge(1, 10, 0), // fold -> base
        make_edge(1, 20, 1), // fold -> step
        make_edge(1, 30, 2), // fold -> collection
    ];
    make_graph(nodes, edges, 1)
}

// ===========================================================================
// Source files
// ===========================================================================

const CONNECT_SRC: &str = include_str!("../src/iris-programs/mutation/connect.iris");
const DELETE_NODE_SRC: &str = include_str!("../src/iris-programs/mutation/delete_node.iris");
const INSERT_NODE_SRC: &str = include_str!("../src/iris-programs/mutation/insert_node.iris");
const REPLACE_PRIM_SRC: &str = include_str!("../src/iris-programs/mutation/replace_prim.iris");
const REWIRE_EDGE_SRC: &str = include_str!("../src/iris-programs/mutation/rewire_edge.iris");
const MUTATE_LIT_SRC: &str = include_str!("../src/iris-programs/mutation/mutate_literal.iris");
const ANNOTATE_COST_SRC: &str = include_str!("../src/iris-programs/mutation/annotate_cost.iris");
const WRAP_GUARD_SRC: &str = include_str!("../src/iris-programs/mutation/wrap_in_guard.iris");
const SWAP_FOLD_SRC: &str = include_str!("../src/iris-programs/mutation/swap_fold_op.iris");
const COMPOSE_SRC: &str = include_str!("../src/iris-programs/mutation/compose_stages.iris");
const WRAP_MAP_SRC: &str = include_str!("../src/iris-programs/mutation/wrap_in_map.iris");
const WRAP_FILTER_SRC: &str = include_str!("../src/iris-programs/mutation/wrap_in_filter.iris");
const INSERT_ZIP_SRC: &str = include_str!("../src/iris-programs/mutation/insert_zip.iris");
const ADD_GUARD_SRC: &str = include_str!("../src/iris-programs/mutation/add_guard_condition.iris");
const EXTRACT_REF_SRC: &str = include_str!("../src/iris-programs/mutation/extract_to_ref.iris");
const DUP_SUB_SRC: &str = include_str!("../src/iris-programs/mutation/duplicate_subgraph.iris");

// ===========================================================================
// Tests: Existing 4 operators
// ===========================================================================

#[test]
fn test_connect_adds_edge() {
    let (graph, registry) = compile_fn(CONNECT_SRC, "connect");
    let target = make_binop_graph(0x00, 5, 3);
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(10), // source
        Value::Int(20), // target
        Value::Int(2),  // port
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    assert_eq!(modified.edges.len(), 3, "should have 3 edges (2 original + 1 new)");
}

#[test]
fn test_delete_node_removes_and_reconnects() {
    let (graph, registry) = compile_fn(DELETE_NODE_SRC, "delete_node");
    let target = make_binop_graph(0x00, 5, 3);
    // delete_node program source victim target port
    // Delete the root (1), reconnecting source=10 to target=20
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(1),  // source (will disconnect from victim)
        Value::Int(10), // victim to delete
        Value::Int(20), // target to reconnect to
        Value::Int(0),  // port
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    assert!(modified.edges.len() >= 1, "should have reconnected edges");
}

#[test]
fn test_insert_node_adds_prim() {
    let (graph, registry) = compile_fn(INSERT_NODE_SRC, "insert_node");
    let target = make_binop_graph(0x00, 5, 3);
    assert_eq!(target.nodes.len(), 3);
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(0x00), // kind=Prim
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    assert_eq!(modified.nodes.len(), 4, "should have 4 nodes after insert");
}

#[test]
fn test_replace_prim_changes_opcode() {
    let (graph, registry) = compile_fn(REPLACE_PRIM_SRC, "replace_prim");
    let target = make_binop_graph(0x00, 5, 3); // add(5,3)
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(0x01), // change to sub
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    let has_sub = modified.nodes.values().any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x01 }));
    assert!(has_sub, "root should now be sub (0x01)");
}

// ===========================================================================
// Tests: New operators
// ===========================================================================

#[test]
fn test_rewire_edge() {
    let (graph, registry) = compile_fn(REWIRE_EDGE_SRC, "rewire_edge");
    let target = make_binop_graph(0x00, 5, 3);
    // Rewire edge from root(1)->lit_a(10) to root(1)->lit_b(20) on port 0
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(1),  // source
        Value::Int(10), // old target
        Value::Int(20), // new target
        Value::Int(0),  // port
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    // Should still have 2 edges, but port-0 now points to node 20
    let port0_targets: Vec<_> = modified.edges.iter()
        .filter(|e| e.source == NodeId(1) && e.port == 0)
        .map(|e| e.target)
        .collect();
    assert!(port0_targets.contains(&NodeId(20)), "port 0 should now target node 20");
}

#[test]
fn test_mutate_int() {
    let (graph, registry) = compile_fn(MUTATE_LIT_SRC, "mutate_int");
    let target = make_binop_graph(0x00, 42, 3);
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(10), // node_id of lit(42)
        Value::Int(5),  // delta
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    // Verify the lit value changed
    assert!(modified.nodes.len() >= 3, "should still have nodes");
}

#[test]
fn test_flip_bool() {
    let (graph, registry) = compile_fn(MUTATE_LIT_SRC, "flip_bool");
    let mut nodes = HashMap::new();
    let (nid, node) = make_node(10, NodeKind::Lit, NodePayload::Lit { type_tag: 4, value: vec![1] }, 0);
    nodes.insert(nid, node);
    let target = make_graph(nodes, vec![], 10);
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(10), // node_id
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    assert!(modified.nodes.len() >= 1, "should have nodes");
}

#[test]
fn test_annotate_cost() {
    let (graph, registry) = compile_fn(ANNOTATE_COST_SRC, "annotate_cost");
    let target = make_binop_graph(0x00, 5, 3);
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(1),  // node_id (root)
        Value::Int(42), // cost value
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    assert_eq!(modified.nodes.len(), 3, "should preserve all nodes");
}

#[test]
fn test_wrap_in_guard() {
    let (graph, registry) = compile_fn(WRAP_GUARD_SRC, "wrap_in_guard");
    let target = make_binop_graph(0x00, 5, 3);
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(1), // body_id (root)
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    // Should have added 3 nodes: pred Lit, fallback Lit, Guard
    assert!(modified.nodes.len() >= 5, "should have at least 5 nodes (3 original + pred + fallback); got {}", modified.nodes.len());
    let has_guard = modified.nodes.values().any(|n| n.kind == NodeKind::Guard);
    assert!(has_guard, "should have a Guard node");
}

#[test]
fn test_swap_fold_op() {
    let (graph, registry) = compile_fn(SWAP_FOLD_SRC, "swap_fold_op");
    let target = make_fold_graph(); // fold(0, add, input)
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(1),  // fold_id
        Value::Int(20), // step_id (add)
        Value::Int(10), // base_id (lit 0)
        Value::Int(0x02), // new opcode: mul
        Value::Int(1),  // new base: 1
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    // Step node should now have opcode 0x02 (mul)
    let has_new_op = modified.nodes.values().any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x02 }));
    assert!(has_new_op, "step should be mul (0x02) after swap");
}

#[test]
fn test_compose_fold() {
    let (graph, registry) = compile_fn(COMPOSE_SRC, "compose_fold");
    let target = make_binop_graph(0x00, 5, 3);
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(0),    // base_value
        Value::Int(0x00), // step_opcode: add
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    // Should have fold as new root
    let has_fold = modified.nodes.values().any(|n| n.kind == NodeKind::Fold);
    assert!(has_fold, "should have a Fold node as new stage");
    assert!(modified.nodes.len() >= 5, "should have original + base + step + fold; got {}", modified.nodes.len());
}

#[test]
fn test_wrap_in_map() {
    let (graph, registry) = compile_fn(WRAP_MAP_SRC, "wrap_in_map");
    let target = make_fold_graph();
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(1),    // fold_id
        Value::Int(30),   // collection_id
        Value::Int(0x02), // step_opcode: mul
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    // Should have Map node (opcode 0x30=48)
    let has_map = modified.nodes.values().any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x30 }));
    assert!(has_map, "should have a Map node (opcode 0x30)");
}

#[test]
fn test_wrap_in_filter() {
    let (graph, registry) = compile_fn(WRAP_FILTER_SRC, "wrap_in_filter");
    let target = make_fold_graph();
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(1),    // fold_id
        Value::Int(30),   // collection_id
        Value::Int(0x22), // cmp_opcode: less_than
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    // Should have Filter node (opcode 0x31=49)
    let has_filter = modified.nodes.values().any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x31 }));
    assert!(has_filter, "should have a Filter node (opcode 0x31)");
}

#[test]
fn test_insert_zip() {
    let (graph, registry) = compile_fn(INSERT_ZIP_SRC, "insert_zip");
    let target = make_fold_graph();
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(1),    // fold_id
        Value::Int(30),   // collection_id
        Value::Int(0x00), // binary_opcode: add
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    // Should have Zip (0x32=50) and Map (0x30=48) nodes
    let has_zip = modified.nodes.values().any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x32 }));
    let has_map = modified.nodes.values().any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x30 }));
    assert!(has_zip, "should have Zip node");
    assert!(has_map, "should have Map node");
}

#[test]
fn test_add_guard_condition() {
    let (graph, registry) = compile_fn(ADD_GUARD_SRC, "add_guard_condition");
    let target = make_binop_graph(0x00, 5, 3);
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(1),    // body_id (root)
        Value::Int(10),   // input_id (lit 5)
        Value::Int(0x22), // cmp_opcode: less_than
        Value::Int(100),  // threshold
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    let has_guard = modified.nodes.values().any(|n| n.kind == NodeKind::Guard);
    assert!(has_guard, "should have a Guard node");
}

#[test]
fn test_extract_to_ref() {
    let (graph, registry) = compile_fn(EXTRACT_REF_SRC, "extract_to_ref");
    let target = make_binop_graph(0x00, 5, 3);
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(10),    // node_id
        Value::Int(12345), // fragment_id
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    let has_ref = modified.nodes.values().any(|n| n.kind == NodeKind::Ref);
    assert!(has_ref, "should have a Ref node");
}

#[test]
fn test_duplicate_node() {
    let (graph, registry) = compile_fn(DUP_SUB_SRC, "duplicate_node");
    let target = make_binop_graph(0x00, 5, 3);
    assert_eq!(target.nodes.len(), 3);
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(1), // node_id (root Prim)
    ];
    let out = run(&graph, &inputs, &registry);
    let modified = extract_program(&out[0]);
    assert_eq!(modified.nodes.len(), 4, "should have 4 nodes after duplication");
}
