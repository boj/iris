
//! Integration tests for runtime self-modification (graph reification).
//!
//! Tests the self-modification primitives (opcodes 0x80-0x89) that enable
//! programs to inspect and modify their own graph structure at runtime.
//! This is the foundation for Daimon's continuous self-improvement: a
//! program executes, produces a modified version of itself, and the
//! modified version runs in the next cycle.

use std::collections::{BTreeMap, HashMap};

use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_program(val: &Value) -> SemanticGraph {
    match val {
        Value::Program(g) => g.as_ref().clone(),
        Value::Tuple(t) if !t.is_empty() => match &t[0] {
            Value::Program(g) => g.as_ref().clone(),
            other => panic!("expected Program in tuple[0], got {:?}", other),
        },
        other => panic!("expected Program, got {:?}", other),
    }
}

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

fn prim_node(id: u64, opcode: u8, arity: u8) -> (NodeId, Node) {
    make_node(id, NodeKind::Prim, NodePayload::Prim { opcode }, arity)
}

/// Create a simple graph that computes `a op b` where op is determined by opcode.
/// Root node is at id=1, lit_a at id=10, lit_b at id=20.
fn make_binop_graph(opcode: u8, a: i64, b: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = int_lit_node(10, a);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(20, b);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(1, opcode, 2);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Helper: modify the root node's opcode (simulates what graph_set_prim_op does).
fn modify_root_opcode(graph: &SemanticGraph, new_opcode: u8) -> SemanticGraph {
    let mut modified = graph.clone();
    let old_root = modified.root;
    let mut root_node = modified.nodes.remove(&old_root).unwrap();
    root_node.payload = NodePayload::Prim { opcode: new_opcode };
    root_node.id = iris_types::hash::compute_node_id(&root_node);
    let new_root = root_node.id;
    modified.nodes.insert(new_root, root_node);
    for edge in &mut modified.edges {
        if edge.source == old_root {
            edge.source = new_root;
        }
        if edge.target == old_root {
            edge.target = new_root;
        }
    }
    modified.root = new_root;
    modified
}

// ---------------------------------------------------------------------------
// Test: self_graph returns a valid Program value
// ---------------------------------------------------------------------------

#[test]
fn self_graph_returns_program() {
    let mut nodes = HashMap::new();
    let (nid, node) = prim_node(1, 0x80, 0); // self_graph
    nodes.insert(nid, node);

    let graph = make_graph(nodes, vec![], 1);
    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();

    assert_eq!(outputs.len(), 1);
    match &outputs[0] {
        Value::Program(g) => {
            assert_eq!(g.nodes.len(), 1);
            assert_eq!(g.root, NodeId(1));
        }
        other => panic!("expected Program, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test: graph_nodes lists all node IDs
// ---------------------------------------------------------------------------

#[test]
fn graph_nodes_lists_ids() {
    let mut nodes = HashMap::new();
    let (nid, node) = prim_node(10, 0x80, 0); // self_graph
    nodes.insert(nid, node);
    let (nid, node) = prim_node(1, 0x81, 1); // graph_nodes
    nodes.insert(nid, node);

    let edges = vec![make_edge(1, 10, 0, EdgeLabel::Argument)];
    let graph = make_graph(nodes, edges, 1);

    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();

    // graph_nodes returns Tuple(Ints), wrapped in a single-element output vec.
    assert_eq!(outputs.len(), 1);
    let inner = match &outputs[0] {
        Value::Tuple(t) => t,
        other => panic!("expected Tuple, got {:?}", other),
    };
    assert_eq!(inner.len(), 2);
    let id_vals: Vec<i64> = inner
        .iter()
        .map(|v| match v {
            Value::Int(n) => *n,
            _ => panic!("expected Int, got {:?}", v),
        })
        .collect();
    assert!(id_vals.contains(&1));
    assert!(id_vals.contains(&10));
}

// ---------------------------------------------------------------------------
// Test: graph_get_kind inspects a node's kind tag
// ---------------------------------------------------------------------------

#[test]
fn graph_get_kind_returns_kind() {
    let mut nodes = HashMap::new();
    let (nid, node) = prim_node(100, 0x80, 0); // self_graph (target to inspect)
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(200, 100); // node ID to inspect
    nodes.insert(nid, node);
    let (nid, node) = prim_node(1, 0x82, 2); // graph_get_kind
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 100, 0, EdgeLabel::Argument),
        make_edge(1, 200, 1, EdgeLabel::Argument),
    ];
    let graph = make_graph(nodes, edges, 1);

    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();
    assert_eq!(outputs, vec![Value::Int(0x00)]); // NodeKind::Prim = 0x00
}

// ---------------------------------------------------------------------------
// Test: graph_get_prim_op reads a Prim node's opcode
// ---------------------------------------------------------------------------

#[test]
fn graph_get_prim_op_reads_opcode() {
    let mut nodes = HashMap::new();
    let (nid, node) = prim_node(100, 0x80, 0); // self_graph with opcode 0x80
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(200, 100);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(1, 0x83, 2); // graph_get_prim_op
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 100, 0, EdgeLabel::Argument),
        make_edge(1, 200, 1, EdgeLabel::Argument),
    ];
    let graph = make_graph(nodes, edges, 1);

    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();
    assert_eq!(outputs, vec![Value::Int(0x80)]);
}

// ---------------------------------------------------------------------------
// Test: graph_set_prim_op changes an opcode (sub -> add)
// ---------------------------------------------------------------------------

#[test]
fn graph_set_prim_op_changes_opcode() {
    // Tuple(sub(5,3), graph_set_prim_op(self_graph(), 100, 0x00))
    // Returns (Int(2), Program with add instead of sub).
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(100, 0x01, 2); // sub
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(110, 5);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(120, 3);
    nodes.insert(nid, node);

    let (nid, node) = prim_node(200, 0x84, 3); // graph_set_prim_op
    nodes.insert(nid, node);
    let (nid, node) = prim_node(300, 0x80, 0); // self_graph
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(310, 100); // target node id
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(320, 0x00); // new opcode (add)
    nodes.insert(nid, node);

    let (nid, node) = make_node(1, NodeKind::Tuple, NodePayload::Tuple, 2);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 100, 0, EdgeLabel::Argument),
        make_edge(1, 200, 1, EdgeLabel::Argument),
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        make_edge(200, 300, 0, EdgeLabel::Argument),
        make_edge(200, 310, 1, EdgeLabel::Argument),
        make_edge(200, 320, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();

    // Root is a Tuple node, so result is wrapped: vec![Tuple([Int(2), Program(...)])]
    assert_eq!(outputs.len(), 1);
    let inner = match &outputs[0] {
        Value::Tuple(t) => t,
        other => panic!("expected Tuple, got {:?}", other),
    };
    assert_eq!(inner.len(), 2);

    assert_eq!(inner[0], Value::Int(2)); // sub(5,3)

    {
        let modified_graph = extract_program(&inner[1]);
        assert!(!modified_graph.nodes.contains_key(&NodeId(100)));
        let add_node = modified_graph
            .nodes
            .values()
            .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x00 }));
        assert!(add_node.is_some(), "modified program should have an add node");
        let add_id = add_node.unwrap().id;

        // Verify edges were updated
        let add_edges: Vec<&Edge> = modified_graph
            .edges
            .iter()
            .filter(|e| e.source == add_id)
            .collect();
        assert_eq!(add_edges.len(), 2);

        // Simulate cycle 2: re-root and evaluate
        let mut cycle2_graph = modified_graph.clone();
        cycle2_graph.root = add_id;
        let (cycle2_out, _) = interpreter::interpret(&cycle2_graph, &[], None).unwrap();
        assert_eq!(cycle2_out, vec![Value::Int(8)]); // add(5,3) = 8
    }
}

// ---------------------------------------------------------------------------
// Test: depth limit on graph_eval prevents infinite self-eval recursion
// ---------------------------------------------------------------------------

#[test]
fn graph_eval_depth_limit() {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x89, 2); // graph_eval
    nodes.insert(nid, node);
    let (nid, node) = prim_node(10, 0x80, 0); // self_graph
    nodes.insert(nid, node);
    let (nid, node) = make_node(
        20,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x06,
            value: vec![],
        },
        0,
    );
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];
    let graph = make_graph(nodes, edges, 1);

    let result = interpreter::interpret(&graph, &[], None);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("self-eval depth") || err_msg.contains("step count") || err_msg.contains("recursion depth"),
        "expected self-eval depth or step limit error, got: {}",
        err_msg
    );
}

// ---------------------------------------------------------------------------
// Test: graph_add_node_rt adds a new node
// ---------------------------------------------------------------------------

#[test]
fn graph_add_node_rt() {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x85, 2); // graph_add_node_rt
    nodes.insert(nid, node);
    let (nid, node) = prim_node(10, 0x80, 0); // self_graph
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(20, 0x00); // opcode for add
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];
    let graph = make_graph(nodes, edges, 1);

    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();

    // graph_add_node_rt returns Tuple(Program, node_id), wrapped in output vec.
    assert_eq!(outputs.len(), 1);
    let inner = match &outputs[0] {
        Value::Tuple(t) => t,
        other => panic!("expected Tuple, got {:?}", other),
    };
    assert_eq!(inner.len(), 2);
    match &inner[0] {
        Value::Program(g) => {
            assert_eq!(g.nodes.len(), 4); // 3 original + 1 new
            let add_node = g
                .nodes
                .values()
                .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x00 }));
            assert!(add_node.is_some());
        }
        other => panic!("expected Program, got {:?}", other),
    }
    assert!(matches!(&inner[1], Value::Int(id) if *id != 0));
}

// ---------------------------------------------------------------------------
// Test: graph_connect adds edges
// ---------------------------------------------------------------------------

#[test]
fn graph_connect_adds_edge() {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x86, 4); // graph_connect
    nodes.insert(nid, node);
    let (nid, node) = prim_node(10, 0x80, 0); // self_graph
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(20, 100); // source
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(30, 110); // target
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(40, 0); // port
    nodes.insert(nid, node);
    let (nid, node) = prim_node(100, 0x00, 1);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(110, 42);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        make_edge(1, 40, 3, EdgeLabel::Argument),
    ];
    let graph = make_graph(nodes, edges, 1);

    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();

    match &outputs[0] {
        Value::Program(modified) => {
            let has_new_edge = modified
                .edges
                .iter()
                .any(|e| e.source == NodeId(100) && e.target == NodeId(110) && e.port == 0);
            assert!(has_new_edge, "edge 100->110 should exist after connect");
        }
        other => panic!("expected Program, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test: graph_disconnect removes edges
// ---------------------------------------------------------------------------

#[test]
fn graph_disconnect_removes_edges() {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x87, 3); // graph_disconnect
    nodes.insert(nid, node);
    let (nid, node) = prim_node(10, 0x80, 0); // self_graph
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(20, 100); // source
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(30, 110); // target
    nodes.insert(nid, node);
    let (nid, node) = prim_node(100, 0x00, 1);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(110, 42);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        make_edge(100, 110, 0, EdgeLabel::Argument), // to be disconnected
    ];
    let graph = make_graph(nodes, edges, 1);

    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();

    match &outputs[0] {
        Value::Program(modified) => {
            let has_edge = modified
                .edges
                .iter()
                .any(|e| e.source == NodeId(100) && e.target == NodeId(110));
            assert!(!has_edge, "edge 100->110 should be disconnected");
        }
        other => panic!("expected Program, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test: full cross-cycle self-improvement with graph_set_prim_op
// ---------------------------------------------------------------------------

#[test]
fn self_graph_set_prim_op_and_eval_cycle() {
    // Cycle 1: Tuple(sub(5,3), modified_program) where sub is changed to add
    // Cycle 2: Orchestrator runs modified program from the add subtree root

    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(100, 0x01, 2); // sub
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(110, 5);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(120, 3);
    nodes.insert(nid, node);

    let (nid, node) = prim_node(200, 0x84, 3); // graph_set_prim_op
    nodes.insert(nid, node);
    let (nid, node) = prim_node(300, 0x80, 0); // self_graph
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(310, 100); // target node id
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(320, 0x00); // new opcode (add)
    nodes.insert(nid, node);

    let (nid, node) = make_node(1, NodeKind::Tuple, NodePayload::Tuple, 2);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 100, 0, EdgeLabel::Argument),
        make_edge(1, 200, 1, EdgeLabel::Argument),
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        make_edge(200, 300, 0, EdgeLabel::Argument),
        make_edge(200, 310, 1, EdgeLabel::Argument),
        make_edge(200, 320, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);

    // Cycle 1
    let (outputs, _) = interpreter::interpret(&graph, &[], None).unwrap();
    assert_eq!(outputs.len(), 1);
    let inner = match &outputs[0] {
        Value::Tuple(t) => t,
        other => panic!("expected Tuple, got {:?}", other),
    };
    assert_eq!(inner[0], Value::Int(2)); // sub(5,3)

    let modified_program = extract_program(&inner[1]);

    // Find the add node and use it as root for cycle 2
    let add_node = modified_program
        .nodes
        .values()
        .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x00 }))
        .expect("should have an add node");
    let add_id = add_node.id;

    let mut cycle2_graph = modified_program;
    cycle2_graph.root = add_id;

    // Cycle 2
    let (outputs2, _) = interpreter::interpret(&cycle2_graph, &[], None).unwrap();
    assert_eq!(outputs2, vec![Value::Int(8)]); // add(5,3) = 8
}

// ---------------------------------------------------------------------------
// Test: multi-cycle self-improvement simulation
// ---------------------------------------------------------------------------

#[test]
fn cycle_based_self_improvement() {
    //   Cycle 1: sub(10, 7) = 3
    //   Cycle 2: add(10, 7) = 17
    //   Cycle 3: mul(10, 7) = 70

    let graph1 = make_binop_graph(0x01, 10, 7);
    let (out1, _) = interpreter::interpret(&graph1, &[], None).unwrap();
    assert_eq!(out1, vec![Value::Int(3)]);

    let graph2 = modify_root_opcode(&graph1, 0x00);
    let (out2, _) = interpreter::interpret(&graph2, &[], None).unwrap();
    assert_eq!(out2, vec![Value::Int(17)]);

    let graph3 = modify_root_opcode(&graph2, 0x02);
    let (out3, _) = interpreter::interpret(&graph3, &[], None).unwrap();
    assert_eq!(out3, vec![Value::Int(70)]);
}
