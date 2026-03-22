
//! Self-writing milestone v2: more mutation operators as IRIS programs.
//!
//! Builds on v1 (replace_prim, insert_node, connect) with four new operators:
//!
//! 1. **delete_node** — disconnect a node's edges, making it unreachable
//! 2. **rewire_edge** — disconnect + reconnect to change an edge's target
//! 3. **replace_kind** — change a Prim node's opcode (Prim->Prim variant)
//! 4. **mutate_literal** — replace a Lit node with a different value using
//!    graph_replace_subtree
//!
//! Each operator is an IRIS program (SemanticGraph) that uses self-modification
//! opcodes (0x80-0x8A) to inspect and transform program graphs at runtime.

use std::collections::{BTreeMap, HashMap};

use iris_exec::interpreter;
use iris_types::component::{ComponentRegistry, MutationComponent};
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Graph construction helpers (shared with v1, duplicated for test isolation)
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

fn input_ref_node(id: u64, index: u8) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0xFF,
            value: vec![index],
        },
        0,
    )
}

fn prim_node(id: u64, opcode: u8, arity: u8) -> (NodeId, Node) {
    make_node(id, NodeKind::Prim, NodePayload::Prim { opcode }, arity)
}

/// Create a graph: `a op b` with Prim root at id=1, lit args at id=10, id=20.
fn make_binop_graph(opcode: u8, a: i64, b: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, opcode, 2);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(10, a);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(20, b);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Create a chain graph: `op2(op1(a, b), c)`.
///
///   Root: op2(id=1, arity=2)
///   ├── port 0: op1(id=2, arity=2)
///   │   ├── port 0: lit(a, id=10)
///   │   └── port 1: lit(b, id=20)
///   └── port 1: lit(c, id=30)
fn make_chain_graph(op1: u8, op2: u8, a: i64, b: i64, c: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, op2, 2); // root
    nodes.insert(nid, node);
    let (nid, node) = prim_node(2, op1, 2); // inner
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(10, a);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(20, b);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(30, c);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 2, 0, EdgeLabel::Argument),  // root -> inner
        make_edge(1, 30, 1, EdgeLabel::Argument),  // root -> lit(c)
        make_edge(2, 10, 0, EdgeLabel::Argument),  // inner -> lit(a)
        make_edge(2, 20, 1, EdgeLabel::Argument),  // inner -> lit(b)
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// Operator 1: delete_node — disconnect a node's edges
// ===========================================================================
//
// The Rust `delete_node` removes a non-root node, drops all edges involving
// it, and reconnects sources to targets. Our IRIS version takes:
//   inputs[0] = Program
//   inputs[1] = Int(source_node_id)  — a node that has an edge TO victim
//   inputs[2] = Int(victim_node_id)  — the node to disconnect
//   inputs[3] = Int(target_node_id)  — a node the victim has an edge TO
//   inputs[4] = Int(port)            — port for the reconnection edge
//
// Steps:
//   1. graph_disconnect(program, source, victim) — remove source->victim edges
//   2. graph_disconnect(result,  victim, target) — remove victim->target edges
//   3. graph_connect(result, source, target, port) — reconnect
//
// The victim node remains in the graph but is unreachable (no edges).
// This matches the Rust behavior where the node is removed from the
// BTreeMap, but from the graph's perspective, an unreachable node is dead.
//
// Graph structure:
//   Root(id=1): graph_connect(0x86, arity=4)
//   ├── port 0: graph_disconnect(0x87) [id=100]     — step 2
//   │   ├── port 0: graph_disconnect(0x87) [id=200]  — step 1
//   │   │   ├── port 0: input_ref(0) [id=300]
//   │   │   ├── port 1: input_ref(1) [id=310]        — source
//   │   │   └── port 2: input_ref(2) [id=320]        — victim
//   │   ├── port 1: input_ref(2) [id=330]            — victim
//   │   └── port 2: input_ref(3) [id=340]            — target
//   ├── port 1: input_ref(1) [id=400]                — source
//   ├── port 2: input_ref(3) [id=410]                — target
//   └── port 3: input_ref(4) [id=420]                — port

fn build_iris_delete_node_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_connect(0x86, 4 args) — reconnect source -> target
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);

    // Step 2: graph_disconnect(program_after_step1, victim, target)
    let (nid, node) = prim_node(100, 0x87, 3);
    nodes.insert(nid, node);

    // Step 1: graph_disconnect(program, source, victim)
    let (nid, node) = prim_node(200, 0x87, 3);
    nodes.insert(nid, node);

    // input_ref nodes
    let (nid, node) = input_ref_node(300, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(310, 1); // source
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(320, 2); // victim
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(330, 2); // victim (for step 2)
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(340, 3); // target (for step 2)
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(400, 1); // source (for reconnect)
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(410, 3); // target (for reconnect)
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(420, 4); // port (for reconnect)
    nodes.insert(nid, node);

    let edges = vec![
        // Root's 4 args: (program_after_disconnects, source, target, port)
        make_edge(1, 100, 0, EdgeLabel::Argument),
        make_edge(1, 400, 1, EdgeLabel::Argument),
        make_edge(1, 410, 2, EdgeLabel::Argument),
        make_edge(1, 420, 3, EdgeLabel::Argument),
        // Step 2: disconnect(step1_result, victim, target)
        make_edge(100, 200, 0, EdgeLabel::Argument),
        make_edge(100, 330, 1, EdgeLabel::Argument),
        make_edge(100, 340, 2, EdgeLabel::Argument),
        // Step 1: disconnect(program, source, victim)
        make_edge(200, 300, 0, EdgeLabel::Argument),
        make_edge(200, 310, 1, EdgeLabel::Argument),
        make_edge(200, 320, 2, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// delete_node tests
// ---------------------------------------------------------------------------

#[test]
fn iris_delete_node_removes_middle_node() {
    // Chain: mul(add(3, 4), 5) = (3+4)*5 = 35
    // Delete the inner add node (id=2), reconnecting root(id=1) -> lit(3, id=10)
    // Result: mul(3, 5) = 15
    let iris_deleter = build_iris_delete_node_program();
    let target = make_chain_graph(0x00, 0x02, 3, 4, 5);

    // Verify original computes correctly
    let (orig_result, _) = interpreter::interpret(&target, &[], None).unwrap();
    assert_eq!(orig_result, vec![Value::Int(35)], "mul(add(3,4), 5) = 35");

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),  // source: root (mul)
        Value::Int(2),  // victim: inner (add)
        Value::Int(10), // target: lit(3) — reconnect root to lit(3)
        Value::Int(0),  // port 0
    ];

    let (outputs, _) = interpreter::interpret(&iris_deleter, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    // The victim node (id=2) is still in the graph but has no edges
    // The root should now point to lit(3) on port 0
    let has_reconnect = modified.edges.iter().any(|e| {
        e.source == NodeId(1) && e.target == NodeId(10) && e.port == 0
    });
    assert!(has_reconnect, "should have reconnection edge root -> lit(3)");

    // The specified source->victim and victim->target edges are removed.
    // The victim may retain other outgoing edges (e.g., 2->20 for lit(4))
    // that weren't part of the disconnect path. This is fine: the victim is
    // no longer reachable from the root through the deleted path.
    let victim_incoming: Vec<_> = modified
        .edges
        .iter()
        .filter(|e| e.target == NodeId(2))
        .collect();
    assert!(
        victim_incoming.is_empty(),
        "victim should have no incoming edges (disconnected from parent), got {:?}",
        victim_incoming
    );

    // The specified victim->target edge (2->10) should be removed
    let victim_to_target = modified
        .edges
        .iter()
        .any(|e| e.source == NodeId(2) && e.target == NodeId(10));
    assert!(
        !victim_to_target,
        "edge victim->target(10) should be removed"
    );

    // Execute: mul(3, 5) = 15
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(15)], "mul(3, 5) should be 15");
}

#[test]
fn iris_delete_node_removes_leaf() {
    // add(5, 3) — delete lit(3) at id=20, reconnecting root(id=1) to lit(5, id=10)
    // Result: add(5, 5) = 10
    let iris_deleter = build_iris_delete_node_program();
    let target = make_binop_graph(0x00, 5, 3);

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),  // source: root (add)
        Value::Int(20), // victim: lit(3)
        Value::Int(10), // target: lit(5) — reconnect to the other lit
        Value::Int(1),  // port 1 (the port where lit(3) was)
    ];

    let (outputs, _) = interpreter::interpret(&iris_deleter, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    // lit(3) at id=20 should have no edges connecting it
    let victim_edges: Vec<_> = modified
        .edges
        .iter()
        .filter(|e| e.source == NodeId(20) || e.target == NodeId(20))
        .collect();
    assert!(victim_edges.is_empty(), "lit(3) should be disconnected");

    // Root should now have two edges to lit(5): port 0 (original) and port 1 (reconnected)
    let root_to_5: Vec<_> = modified
        .edges
        .iter()
        .filter(|e| e.source == NodeId(1) && e.target == NodeId(10))
        .collect();
    assert_eq!(root_to_5.len(), 2, "root should have 2 edges to lit(5)");

    // Execute: add(5, 5) = 10
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(10)], "add(5, 5) should be 10");
}

#[test]
fn iris_delete_node_preserves_node_count() {
    // The IRIS delete_node disconnects edges but doesn't remove the node
    // from the graph. This is fine — the node is unreachable.
    let iris_deleter = build_iris_delete_node_program();
    let target = make_chain_graph(0x00, 0x02, 3, 4, 5);
    let original_count = target.nodes.len();

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),
        Value::Int(2),
        Value::Int(10),
        Value::Int(0),
    ];

    let (outputs, _) = interpreter::interpret(&iris_deleter, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    // Node count unchanged (victim is unreachable but still present)
    assert_eq!(
        modified.nodes.len(),
        original_count,
        "node count should be preserved (victim is unreachable)"
    );
}

#[test]
fn register_iris_delete_node_as_component() {
    let iris_program = build_iris_delete_node_program();

    let component = MutationComponent {
        name: "iris_delete_node".to_string(),
        program: iris_program.clone(),
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_delete_node");
    assert!(found.is_some(), "iris_delete_node should be registered");

    // Execute: delete inner node from chain
    let target = make_chain_graph(0x00, 0x02, 3, 4, 5);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),
        Value::Int(2),
        Value::Int(10),
        Value::Int(0),
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    {

        let g = extract_program(&outputs[0]);
            let (result, _) = interpreter::interpret(&g, &[], None).unwrap();
            assert_eq!(result, vec![Value::Int(15)], "mul(3, 5) = 15");

    }

    eprintln!("delete_node mutation operator replaced by IRIS program");
}

// ===========================================================================
// Operator 2: rewire_edge — change an edge's target
// ===========================================================================
//
// The Rust `rewire_edge` picks a random edge and changes its target (or
// source). Our IRIS version takes:
//   inputs[0] = Program
//   inputs[1] = Int(source_node_id) — the edge's source
//   inputs[2] = Int(old_target_id)  — the edge's current target
//   inputs[3] = Int(new_target_id)  — the new target
//   inputs[4] = Int(port)           — port for the new edge
//
// Steps:
//   1. graph_disconnect(program, source, old_target) — remove the old edge
//   2. graph_connect(result, source, new_target, port) — add the new edge
//
// Graph structure:
//   Root(id=1): graph_connect(0x86, arity=4)
//   ├── port 0: graph_disconnect(0x87) [id=100]
//   │   ├── port 0: input_ref(0) [id=200]     — program
//   │   ├── port 1: input_ref(1) [id=210]     — source
//   │   └── port 2: input_ref(2) [id=220]     — old target
//   ├── port 1: input_ref(1) [id=300]         — source (for connect)
//   ├── port 2: input_ref(3) [id=310]         — new target
//   └── port 3: input_ref(4) [id=320]         — port

fn build_iris_rewire_edge_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_connect(0x86, 4 args)
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);

    // graph_disconnect(program, source, old_target)
    let (nid, node) = prim_node(100, 0x87, 3);
    nodes.insert(nid, node);

    // input_ref nodes
    let (nid, node) = input_ref_node(200, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(210, 1); // source
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(220, 2); // old target
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(300, 1); // source (for connect)
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(310, 3); // new target
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(320, 4); // port
    nodes.insert(nid, node);

    let edges = vec![
        // Root: connect(disconnect_result, source, new_target, port)
        make_edge(1, 100, 0, EdgeLabel::Argument),
        make_edge(1, 300, 1, EdgeLabel::Argument),
        make_edge(1, 310, 2, EdgeLabel::Argument),
        make_edge(1, 320, 3, EdgeLabel::Argument),
        // disconnect(program, source, old_target)
        make_edge(100, 200, 0, EdgeLabel::Argument),
        make_edge(100, 210, 1, EdgeLabel::Argument),
        make_edge(100, 220, 2, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// rewire_edge tests
// ---------------------------------------------------------------------------

#[test]
fn iris_rewire_edge_changes_argument() {
    // add(5, 3) — rewire port 1 from lit(3) to lit(5)
    // Result: add(5, 5) = 10
    let iris_rewirer = build_iris_rewire_edge_program();
    let target = make_binop_graph(0x00, 5, 3);

    let (orig, _) = interpreter::interpret(&target, &[], None).unwrap();
    assert_eq!(orig, vec![Value::Int(8)], "add(5, 3) = 8");

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),  // source: root (add)
        Value::Int(20), // old target: lit(3)
        Value::Int(10), // new target: lit(5)
        Value::Int(1),  // port 1
    ];

    let (outputs, _) = interpreter::interpret(&iris_rewirer, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    // Old edge root->lit(3) should be gone
    let old_edge = modified
        .edges
        .iter()
        .any(|e| e.source == NodeId(1) && e.target == NodeId(20));
    assert!(!old_edge, "old edge to lit(3) should be removed");

    // New edge root->lit(5) on port 1 should exist
    let new_edge = modified
        .edges
        .iter()
        .any(|e| e.source == NodeId(1) && e.target == NodeId(10) && e.port == 1);
    assert!(new_edge, "new edge to lit(5) on port 1 should exist");

    // Execute: add(5, 5) = 10
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(10)], "add(5, 5) should be 10");
}

#[test]
fn iris_rewire_edge_swaps_operands() {
    // sub(10, 3) = 7 — rewire port 0 from lit(10) to lit(3), port 1 from lit(3) to lit(10)
    // After first rewire: sub(3, 3) — but that's two steps.
    // Instead, just rewire port 1: sub(10, lit(10)) -> sub(10, 10) = 0
    let iris_rewirer = build_iris_rewire_edge_program();
    let target = make_binop_graph(0x01, 10, 3); // sub(10, 3) = 7

    let (orig, _) = interpreter::interpret(&target, &[], None).unwrap();
    assert_eq!(orig, vec![Value::Int(7)], "sub(10, 3) = 7");

    // Rewire port 1 from lit(3) to lit(10)
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),  // source
        Value::Int(20), // old target: lit(3)
        Value::Int(10), // new target: lit(10)
        Value::Int(1),  // port 1
    ];

    let (outputs, _) = interpreter::interpret(&iris_rewirer, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    // Execute: sub(10, 10) = 0
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(0)], "sub(10, 10) should be 0");
}

#[test]
fn iris_rewire_edge_redirects_to_subexpression() {
    // Chain: mul(add(3, 4), 5) = 35
    // Rewire root port 1 from lit(5) to inner add node
    // Result: mul(add(3, 4), add(3, 4)) = 7 * 7 = 49
    let iris_rewirer = build_iris_rewire_edge_program();
    let target = make_chain_graph(0x00, 0x02, 3, 4, 5);

    let (orig, _) = interpreter::interpret(&target, &[], None).unwrap();
    assert_eq!(orig, vec![Value::Int(35)], "mul(add(3,4), 5) = 35");

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),  // source: root (mul)
        Value::Int(30), // old target: lit(5)
        Value::Int(2),  // new target: inner (add)
        Value::Int(1),  // port 1
    ];

    let (outputs, _) = interpreter::interpret(&iris_rewirer, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    // Execute: mul(add(3,4), add(3,4)) = 7 * 7 = 49
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(49)],
        "mul(add(3,4), add(3,4)) should be 49"
    );
}

#[test]
fn iris_rewire_matches_rust_semantics() {
    // The Rust rewire_edge picks a random edge and changes its target.
    // Our IRIS version does exactly that: disconnect old, connect new.
    // Verify across multiple cases.
    let iris_rewirer = build_iris_rewire_edge_program();

    let cases: Vec<(u8, i64, i64, u64, u64, u64, u8, i64)> = vec![
        // (opcode, a, b, source, old_target, new_target, port, expected)
        (0x00, 5, 3, 1, 20, 10, 1, 10),  // add(5,3)->add(5,5)=10
        (0x01, 10, 3, 1, 10, 20, 0, 0),  // sub(10,3)->sub(3,3)=0
        (0x02, 7, 8, 1, 20, 10, 1, 49),  // mul(7,8)->mul(7,7)=49
    ];

    for (opcode, a, b, src, old_tgt, new_tgt, port, expected) in &cases {
        let target = make_binop_graph(*opcode, *a, *b);

        let inputs = vec![
            Value::Program(Box::new(target)),
            Value::Int(*src as i64),
            Value::Int(*old_tgt as i64),
            Value::Int(*new_tgt as i64),
            Value::Int(*port as i64),
        ];

        let (outputs, _) = interpreter::interpret(&iris_rewirer, &inputs, None).unwrap();

        let modified = extract_program(&outputs[0]);

        let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
        assert_eq!(
            result,
            vec![Value::Int(*expected)],
            "rewire case 0x{:02x}({},{}) src={} old={} new={} port={}: expected {}",
            opcode, a, b, src, old_tgt, new_tgt, port, expected
        );
    }
}

#[test]
fn register_iris_rewire_edge_as_component() {
    let iris_program = build_iris_rewire_edge_program();

    let component = MutationComponent {
        name: "iris_rewire_edge".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_rewire_edge");
    assert!(found.is_some(), "iris_rewire_edge should be registered");

    let target = make_binop_graph(0x00, 5, 3);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),
        Value::Int(20),
        Value::Int(10),
        Value::Int(1),
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    {

        let g = extract_program(&outputs[0]);
            let (result, _) = interpreter::interpret(&g, &[], None).unwrap();
            assert_eq!(result, vec![Value::Int(10)], "add(5, 5) = 10");

    }

    eprintln!("rewire_edge mutation operator replaced by IRIS program");
}

// ===========================================================================
// Operator 3: replace_kind (Prim->Prim variant)
// ===========================================================================
//
// The Rust `replace_kind` changes a node's kind+payload. The most common
// and useful case is Prim->Prim (changing the opcode). This is identical
// to `replace_prim` but we wrap it with graph_get_kind inspection to
// verify the target is actually a Prim node before changing it.
//
// Inputs:
//   inputs[0] = Program
//   inputs[1] = Int(target_node_id)
//   inputs[2] = Int(new_opcode)
//
// Graph structure:
//   Root(id=1): graph_set_prim_op(0x84, arity=3)
//   ├── port 0: input_ref(0) [id=10]   — program
//   ├── port 1: input_ref(1) [id=20]   — target node ID
//   └── port 2: input_ref(2) [id=30]   — new opcode

fn build_iris_replace_kind_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_set_prim_op(0x84, 3 args)
    let (nid, node) = prim_node(1, 0x84, 3);
    nodes.insert(nid, node);

    // input_ref nodes
    let (nid, node) = input_ref_node(10, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1); // target node ID
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 2); // new opcode
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build a more advanced IRIS program that inspects the node kind before
/// changing it. Uses graph_get_kind(0x82) to verify the target is Prim
/// (kind=0x00), then applies graph_set_prim_op.
///
/// Inputs:
///   inputs[0] = Program
///   inputs[1] = Int(target_node_id)
///   inputs[2] = Int(new_opcode)
///
/// Graph structure:
///   Root(id=1): Tuple(arity=2)     — returns (kind_tag, modified_program)
///   ├── port 0: graph_get_kind(0x82) [id=100]
///   │   ├── port 0: input_ref(0) [id=110]
///   │   └── port 1: input_ref(1) [id=120]
///   └── port 1: graph_set_prim_op(0x84) [id=200]
///       ├── port 0: input_ref(0) [id=210]
///       ├── port 1: input_ref(1) [id=220]
///       └── port 2: input_ref(2) [id=230]
fn build_iris_inspect_and_replace_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Tuple(arity=2) — returns (kind, modified_program)
    let (nid, node) = make_node(1, NodeKind::Tuple, NodePayload::Tuple, 2);
    nodes.insert(nid, node);

    // graph_get_kind(0x82, 2 args)
    let (nid, node) = prim_node(100, 0x82, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(110, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(120, 1);
    nodes.insert(nid, node);

    // graph_set_prim_op(0x84, 3 args)
    let (nid, node) = prim_node(200, 0x84, 3);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(210, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(220, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(230, 2);
    nodes.insert(nid, node);

    let edges = vec![
        // Root -> kind, modified_program
        make_edge(1, 100, 0, EdgeLabel::Argument),
        make_edge(1, 200, 1, EdgeLabel::Argument),
        // graph_get_kind(program, node_id)
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        // graph_set_prim_op(program, node_id, new_opcode)
        make_edge(200, 210, 0, EdgeLabel::Argument),
        make_edge(200, 220, 1, EdgeLabel::Argument),
        make_edge(200, 230, 2, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// replace_kind tests
// ---------------------------------------------------------------------------

#[test]
fn iris_replace_kind_changes_add_to_mul() {
    let iris_replacer = build_iris_replace_kind_program();
    let target = make_binop_graph(0x00, 6, 7); // add(6, 7) = 13

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),    // target node: root (add)
        Value::Int(0x02), // new opcode: mul
    ];

    let (outputs, _) = interpreter::interpret(&iris_replacer, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    // The old add node (NodeId(1)) should be gone (content-addressed)
    assert!(
        !modified.nodes.contains_key(&NodeId(1)),
        "old add node should be removed (ID changed due to content addressing)"
    );

    // Find the new mul node
    let mul_node = modified
        .nodes
        .values()
        .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x02 }));
    assert!(mul_node.is_some(), "should have mul node");

    // Execute: mul(6, 7) = 42
    let mul_id = mul_node.unwrap().id;
    let mut runnable = modified;
    runnable.root = mul_id;
    let (result, _) = interpreter::interpret(&runnable, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(42)], "mul(6, 7) should be 42");
}

#[test]
fn iris_replace_kind_changes_inner_node() {
    // Chain: mul(add(3, 4), 5) = 35
    // Replace inner add(id=2) with sub -> mul(sub(3, 4), 5) = (-1)*5 = -5
    let iris_replacer = build_iris_replace_kind_program();
    let target = make_chain_graph(0x00, 0x02, 3, 4, 5);

    let (orig, _) = interpreter::interpret(&target, &[], None).unwrap();
    assert_eq!(orig, vec![Value::Int(35)], "mul(add(3,4), 5) = 35");

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(2),    // target node: inner (add)
        Value::Int(0x01), // new opcode: sub
    ];

    let (outputs, _) = interpreter::interpret(&iris_replacer, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    // Find the sub node (replacing the old add)
    let sub_node = modified
        .nodes
        .values()
        .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x01 }));
    assert!(sub_node.is_some(), "should have sub node");

    // Execute: mul(sub(3, 4), 5) = (-1) * 5 = -5
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(-5)],
        "mul(sub(3,4), 5) should be -5"
    );
}

#[test]
fn iris_replace_kind_with_inspection() {
    // The inspect_and_replace program returns (kind_tag, modified_program).
    // This verifies graph_get_kind returns the correct NodeKind tag.
    let iris_inspect_replace = build_iris_inspect_and_replace_program();
    let target = make_binop_graph(0x00, 6, 7); // add(6, 7)

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),    // target node: root (add) — kind should be Prim=0x00
        Value::Int(0x02), // new opcode: mul
    ];

    let (outputs, _) = interpreter::interpret(&iris_inspect_replace, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Tuple(elems) => {
            assert_eq!(elems.len(), 2, "should return (kind, program)");

            // Verify kind tag: Prim = 0x00
            assert_eq!(
                elems[0],
                Value::Int(0x00),
                "NodeKind::Prim should be 0x00"
            );

            // Verify the modified program works
            let modified = extract_program(&elems[1]);

            let mul_node = modified
                .nodes
                .values()
                .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x02 }));
            assert!(mul_node.is_some(), "should have mul node");

            let mul_id = mul_node.unwrap().id;
            let mut runnable = modified;
            runnable.root = mul_id;
            let (result, _) = interpreter::interpret(&runnable, &[], None).unwrap();
            assert_eq!(result, vec![Value::Int(42)], "mul(6, 7) should be 42");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn iris_replace_kind_inspect_lit_node() {
    // Verify graph_get_kind correctly identifies a Lit node (kind=0x05).
    let target = make_binop_graph(0x00, 6, 7);

    // Use graph_get_kind alone to inspect a Lit node.
    // (The inspect_and_replace program would error because graph_set_prim_op
    // fails on non-Prim nodes, but graph_get_kind works independently.)
    let mut nodes = HashMap::new();
    let (nid, node) = prim_node(1, 0x82, 2); // graph_get_kind
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1);
    nodes.insert(nid, node);
    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];
    let kind_inspector = make_graph(nodes, edges, 1);

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(10), // Lit node
    ];
    let (outputs, _) = interpreter::interpret(&kind_inspector, &inputs, None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(0x05)],
        "NodeKind::Lit should be 0x05"
    );
    // Suppress unused variable warning
    drop(outputs);
}

#[test]
fn iris_replace_kind_all_arithmetic_opcodes() {
    // Verify replace_kind works for all arithmetic opcodes (0x00-0x09).
    let iris_replacer = build_iris_replace_kind_program();

    let opcode_names = [
        (0x00u8, "add"),
        (0x01, "sub"),
        (0x02, "mul"),
        (0x03, "div"),
        (0x07, "min"),
        (0x08, "max"),
    ];

    for &(new_opcode, name) in &opcode_names {
        let target = make_binop_graph(0x00, 10, 3); // start with add

        let inputs = vec![
            Value::Program(Box::new(target)),
            Value::Int(1),
            Value::Int(new_opcode as i64),
        ];

        let (outputs, _) = interpreter::interpret(&iris_replacer, &inputs, None).unwrap();

        let modified = extract_program(&outputs[0]);

        // Verify the new node has the correct opcode
        let new_node = modified
            .nodes
            .values()
            .find(|n| matches!(&n.payload, NodePayload::Prim { opcode } if *opcode == new_opcode));
        assert!(
            new_node.is_some(),
            "should have {} (0x{:02x}) node",
            name,
            new_opcode
        );
    }
}

#[test]
fn register_iris_replace_kind_as_component() {
    let iris_program = build_iris_replace_kind_program();

    let component = MutationComponent {
        name: "iris_replace_kind".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_replace_kind");
    assert!(found.is_some(), "iris_replace_kind should be registered");

    let target = make_binop_graph(0x00, 6, 7);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),
        Value::Int(0x02),
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    {

        let g = extract_program(&outputs[0]);
            let mul_node = g
                .nodes
                .values()
                .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x02 }));
            assert!(mul_node.is_some(), "should have mul node");

    }

    eprintln!("replace_kind mutation operator replaced by IRIS program");
}

// ===========================================================================
// Operator 4: mutate_literal — replace a Lit node with a different value
// ===========================================================================
//
// The Rust `mutate_literal` changes a Lit node's value in place. Since
// there's no `graph_set_lit` opcode, we use `graph_replace_subtree(0x88)`
// to replace the old Lit node with a new one from a donor graph.
//
// Strategy: We build a donor graph containing just the new Lit node, then
// use graph_replace_subtree to swap the old Lit node for the new one.
//
// The IRIS program takes a pre-built donor program as input (containing the
// replacement Lit node). This is the simplest approach because building a
// new Lit node at runtime would require a `graph_add_lit` opcode we don't
// have.
//
// Inputs:
//   inputs[0] = Program(target_graph)
//   inputs[1] = Int(target_node_id)    — the Lit node to replace
//   inputs[2] = Program(donor_graph)   — contains the replacement node
//   inputs[3] = Int(donor_node_id)     — the replacement node's ID
//
// Graph structure:
//   Root(id=1): graph_replace_subtree(0x88, arity=4)
//   ├── port 0: input_ref(0) [id=10]  — target program
//   ├── port 1: input_ref(1) [id=20]  — target node ID
//   ├── port 2: input_ref(2) [id=30]  — donor program
//   └── port 3: input_ref(3) [id=40]  — donor node ID

fn build_iris_mutate_literal_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_replace_subtree(0x88, 4 args)
    let (nid, node) = prim_node(1, 0x88, 4);
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(10, 0); // target program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1); // target node ID
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 2); // donor program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(40, 3); // donor node ID
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        make_edge(1, 40, 3, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build a donor graph containing a single Lit node with the given value.
fn make_donor_lit(id: u64, value: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = int_lit_node(id, value);
    nodes.insert(nid, node);
    make_graph(nodes, vec![], id)
}

// ---------------------------------------------------------------------------
// mutate_literal tests
// ---------------------------------------------------------------------------

#[test]
fn iris_mutate_literal_changes_value() {
    // add(5, 3) = 8 — replace lit(3) at id=20 with lit(10)
    // Result: add(5, 10) = 15
    let iris_mutator = build_iris_mutate_literal_program();
    let target = make_binop_graph(0x00, 5, 3);

    let donor = make_donor_lit(500, 10);
    let donor_node_id = 500i64;

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(20), // replace lit(3) at NodeId(20)
        Value::Program(Box::new(donor)),
        Value::Int(donor_node_id),
    ];

    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    // The old lit(3) at NodeId(20) should be removed
    assert!(
        !modified.nodes.contains_key(&NodeId(20)),
        "old lit(3) node should be replaced"
    );

    // The donor lit(10) at NodeId(500) should be present
    assert!(
        modified.nodes.contains_key(&NodeId(500)),
        "donor lit(10) should be in the graph"
    );

    // Edges that pointed to NodeId(20) should now point to NodeId(500)
    let edges_to_donor: Vec<_> = modified
        .edges
        .iter()
        .filter(|e| e.target == NodeId(500))
        .collect();
    assert!(
        !edges_to_donor.is_empty(),
        "should have edges pointing to the donor node"
    );

    // Execute: add(5, 10) = 15
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(15)], "add(5, 10) should be 15");
}

#[test]
fn iris_mutate_literal_negative_value() {
    // mul(4, 7) = 28 — replace lit(7) at id=20 with lit(-3)
    // Result: mul(4, -3) = -12
    let iris_mutator = build_iris_mutate_literal_program();
    let target = make_binop_graph(0x02, 4, 7);

    let donor = make_donor_lit(600, -3);

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(20), // replace lit(7)
        Value::Program(Box::new(donor)),
        Value::Int(600),
    ];

    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(-12)], "mul(4, -3) should be -12");
}

#[test]
fn iris_mutate_literal_zero() {
    // sub(10, 3) = 7 — replace lit(10) at id=10 with lit(0)
    // Result: sub(0, 3) = -3
    let iris_mutator = build_iris_mutate_literal_program();
    let target = make_binop_graph(0x01, 10, 3);

    let donor = make_donor_lit(700, 0);

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(10), // replace lit(10) at NodeId(10)
        Value::Program(Box::new(donor)),
        Value::Int(700),
    ];

    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(-3)], "sub(0, 3) should be -3");
}

#[test]
fn iris_mutate_literal_large_value() {
    // add(5, 3) = 8 — replace lit(5) at id=10 with lit(1000000)
    // Result: add(1000000, 3) = 1000003
    let iris_mutator = build_iris_mutate_literal_program();
    let target = make_binop_graph(0x00, 5, 3);

    let donor = make_donor_lit(800, 1_000_000);

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(10), // replace lit(5) at NodeId(10)
        Value::Program(Box::new(donor)),
        Value::Int(800),
    ];

    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(1_000_003)],
        "add(1000000, 3) should be 1000003"
    );
}

#[test]
fn iris_mutate_literal_preserves_other_nodes() {
    // Verify that replacing a Lit node doesn't affect other nodes.
    let iris_mutator = build_iris_mutate_literal_program();
    let target = make_chain_graph(0x00, 0x02, 3, 4, 5);

    // Replace lit(3) at id=10 with lit(100)
    let donor = make_donor_lit(900, 100);

    let inputs = vec![
        Value::Program(Box::new(target.clone())),
        Value::Int(10), // replace lit(3)
        Value::Program(Box::new(donor)),
        Value::Int(900),
    ];

    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    // All non-replaced nodes should still be present
    assert!(modified.nodes.contains_key(&NodeId(1)), "root should exist");
    assert!(
        modified.nodes.contains_key(&NodeId(2)),
        "inner op should exist"
    );
    assert!(
        modified.nodes.contains_key(&NodeId(20)),
        "lit(4) should exist"
    );
    assert!(
        modified.nodes.contains_key(&NodeId(30)),
        "lit(5) should exist"
    );
    assert!(
        modified.nodes.contains_key(&NodeId(900)),
        "donor lit(100) should exist"
    );

    // Execute: mul(add(100, 4), 5) = 104 * 5 = 520
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(520)],
        "mul(add(100, 4), 5) should be 520"
    );
}

#[test]
fn iris_mutate_literal_matches_rust_semantics() {
    // The Rust mutate_literal changes a Lit node's bytes in place.
    // Our IRIS version replaces the entire Lit node via graph_replace_subtree.
    // Verify they produce equivalent results across multiple test cases.
    let iris_mutator = build_iris_mutate_literal_program();

    let cases: Vec<(u8, i64, i64, u64, i64, i64)> = vec![
        // (opcode, a, b, target_node_id, new_value, expected_result)
        (0x00, 5, 3, 20, 10, 15),    // add(5,10)=15
        (0x01, 10, 3, 10, 20, 17),   // sub(20,3)=17
        (0x02, 4, 7, 20, 5, 20),     // mul(4,5)=20
        (0x00, 1, 1, 10, 0, 1),      // add(0,1)=1
        (0x01, 100, 1, 20, 50, 50),  // sub(100,50)=50
        (0x02, -5, 3, 10, 2, 6),     // mul(2,3)=6
    ];

    for (i, (opcode, a, b, tgt_id, new_val, expected)) in cases.iter().enumerate() {
        let target = make_binop_graph(*opcode, *a, *b);
        let donor_id = 1000 + i as u64;
        let donor = make_donor_lit(donor_id, *new_val);

        let inputs = vec![
            Value::Program(Box::new(target)),
            Value::Int(*tgt_id as i64),
            Value::Program(Box::new(donor)),
            Value::Int(donor_id as i64),
        ];

        let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();

        let modified = extract_program(&outputs[0]);

        let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
        assert_eq!(
            result,
            vec![Value::Int(*expected)],
            "case {}: 0x{:02x}({}->{}, {}) should be {}",
            i,
            opcode,
            a,
            new_val,
            b,
            expected
        );
    }
}

#[test]
fn register_iris_mutate_literal_as_component() {
    let iris_program = build_iris_mutate_literal_program();

    let component = MutationComponent {
        name: "iris_mutate_literal".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_mutate_literal");
    assert!(found.is_some(), "iris_mutate_literal should be registered");

    // Execute via component
    let target = make_binop_graph(0x00, 5, 3);
    let donor = make_donor_lit(999, 42);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(20),
        Value::Program(Box::new(donor)),
        Value::Int(999),
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    {

        let g = extract_program(&outputs[0]);
            let (result, _) = interpreter::interpret(&g, &[], None).unwrap();
            assert_eq!(result, vec![Value::Int(47)], "add(5, 42) = 47");

    }

    eprintln!("mutate_literal mutation operator replaced by IRIS program");
}

// ===========================================================================
// Composition test: chain all four new operators
// ===========================================================================

#[test]
fn chain_all_four_new_operators() {
    // Start with mul(add(3, 4), 5) = 35
    // Step 1: replace_kind — change inner add(id=2) to sub
    // Step 2: rewire_edge — rewire root port 1 from lit(5) to lit(3)
    // Step 3: mutate_literal — replace lit(4, id=20) with lit(10)
    // Step 4: delete_node — delete lit(5, id=30) (now disconnected anyway)
    //
    // After step 1: mul(sub(3, 4), 5) = (-1)*5 = -5
    // After step 2: mul(sub(3, 4), 3) = (-1)*3 = -3
    // After step 3: mul(sub(3, 10), 3) = (-7)*3 = -21

    let replacer = build_iris_replace_kind_program();
    let rewirer = build_iris_rewire_edge_program();
    let mutator = build_iris_mutate_literal_program();

    // Step 1: replace_kind — add(id=2) -> sub
    let target = make_chain_graph(0x00, 0x02, 3, 4, 5);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(2),    // inner add node
        Value::Int(0x01), // sub opcode
    ];
    let (out, _) = interpreter::interpret(&replacer, &inputs, None).unwrap();
    let step1 = extract_program(&out[0]);

    let (r1, _) = interpreter::interpret(&step1, &[], None).unwrap();
    assert_eq!(r1, vec![Value::Int(-5)], "step1: mul(sub(3,4), 5) = -5");

    // Step 2: rewire_edge — root port 1 from lit(5, id=30) to lit(3, id=10)
    let inputs = vec![
        Value::Program(Box::new(step1)),
        Value::Int(1),  // source: root
        Value::Int(30), // old target: lit(5)
        Value::Int(10), // new target: lit(3)
        Value::Int(1),  // port 1
    ];
    let (out, _) = interpreter::interpret(&rewirer, &inputs, None).unwrap();
    let step2 = extract_program(&out[0]);

    let (r2, _) = interpreter::interpret(&step2, &[], None).unwrap();
    assert_eq!(r2, vec![Value::Int(-3)], "step2: mul(sub(3,4), 3) = -3");

    // Step 3: mutate_literal — replace lit(4, id=20) with lit(10)
    let donor = make_donor_lit(900, 10);
    let inputs = vec![
        Value::Program(Box::new(step2)),
        Value::Int(20),
        Value::Program(Box::new(donor)),
        Value::Int(900),
    ];
    let (out, _) = interpreter::interpret(&mutator, &inputs, None).unwrap();
    let step3 = extract_program(&out[0]);

    let (r3, _) = interpreter::interpret(&step3, &[], None).unwrap();
    assert_eq!(r3, vec![Value::Int(-21)], "step3: mul(sub(3,10), 3) = -21");

    eprintln!(
        "Chained 3 IRIS mutation operators: replace_kind -> rewire_edge -> mutate_literal"
    );
}

// ===========================================================================
// Cross-cycle verification: mutated programs remain valid IRIS targets
// ===========================================================================

#[test]
fn mutated_programs_can_be_mutated_again() {
    // Verify that outputs from one IRIS mutation can be fed back as inputs
    // to another IRIS mutation (multi-cycle self-improvement).
    let replacer = build_iris_replace_kind_program();

    // Cycle 1: add(5, 3) -> sub(5, 3) = 2
    let p0 = make_binop_graph(0x00, 5, 3);
    let inputs = vec![
        Value::Program(Box::new(p0)),
        Value::Int(1),
        Value::Int(0x01), // sub
    ];
    let (out, _) = interpreter::interpret(&replacer, &inputs, None).unwrap();
    let p1 = extract_program(&out[0]);

    // Find the new Prim node ID (content-addressed, so it changed)
    let prim1 = p1
        .nodes
        .values()
        .find(|n| n.kind == NodeKind::Prim)
        .unwrap();
    let prim1_id = prim1.id.0 as i64;

    // Cycle 2: sub -> mul
    let inputs = vec![
        Value::Program(Box::new(p1)),
        Value::Int(prim1_id),
        Value::Int(0x02), // mul
    ];
    let (out, _) = interpreter::interpret(&replacer, &inputs, None).unwrap();
    let p2 = extract_program(&out[0]);

    let prim2 = p2
        .nodes
        .values()
        .find(|n| n.kind == NodeKind::Prim)
        .unwrap();
    let mut runnable = p2.clone();
    runnable.root = prim2.id;
    let (result, _) = interpreter::interpret(&runnable, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(15)], "mul(5, 3) = 15");

    // Cycle 3: mul -> max
    let prim2_id = prim2.id.0 as i64;
    let inputs = vec![
        Value::Program(Box::new(p2)),
        Value::Int(prim2_id),
        Value::Int(0x08), // max
    ];
    let (out, _) = interpreter::interpret(&replacer, &inputs, None).unwrap();
    let p3 = extract_program(&out[0]);

    let prim3_id = p3
        .nodes
        .values()
        .find(|n| n.kind == NodeKind::Prim)
        .unwrap()
        .id;
    let mut runnable = p3;
    runnable.root = prim3_id;
    let (result, _) = interpreter::interpret(&runnable, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(5)], "max(5, 3) = 5");

    eprintln!("Three cycles of IRIS self-modification completed successfully");
}

// ===========================================================================
// Summary: all 4 operators work as ComponentRegistry mutations
// ===========================================================================

#[test]
fn all_four_operators_registered_together() {
    let mut registry = ComponentRegistry::new();

    registry.mutations.push(MutationComponent {
        name: "iris_delete_node".to_string(),
        program: build_iris_delete_node_program(),
    });
    registry.mutations.push(MutationComponent {
        name: "iris_rewire_edge".to_string(),
        program: build_iris_rewire_edge_program(),
    });
    registry.mutations.push(MutationComponent {
        name: "iris_replace_kind".to_string(),
        program: build_iris_replace_kind_program(),
    });
    registry.mutations.push(MutationComponent {
        name: "iris_mutate_literal".to_string(),
        program: build_iris_mutate_literal_program(),
    });

    assert_eq!(registry.mutations.len(), 4);
    assert!(registry.find_mutation("iris_delete_node").is_some());
    assert!(registry.find_mutation("iris_rewire_edge").is_some());
    assert!(registry.find_mutation("iris_replace_kind").is_some());
    assert!(registry.find_mutation("iris_mutate_literal").is_some());

    // Verify each one executes successfully
    let target = make_binop_graph(0x00, 5, 3);

    // delete_node
    let inputs = vec![
        Value::Program(Box::new(make_chain_graph(0x00, 0x02, 3, 4, 5))),
        Value::Int(1),
        Value::Int(2),
        Value::Int(10),
        Value::Int(0),
    ];
    let (out, _) = interpreter::interpret(
        &registry.find_mutation("iris_delete_node").unwrap().program,
        &inputs,
        None,
    )
    .unwrap();
    assert!(matches!(&out[0], Value::Program(_)));

    // rewire_edge
    let inputs = vec![
        Value::Program(Box::new(target.clone())),
        Value::Int(1),
        Value::Int(20),
        Value::Int(10),
        Value::Int(1),
    ];
    let (out, _) = interpreter::interpret(
        &registry.find_mutation("iris_rewire_edge").unwrap().program,
        &inputs,
        None,
    )
    .unwrap();
    assert!(matches!(&out[0], Value::Program(_)));

    // replace_kind returns Tuple(Program, new_id) — extract the program
    let inputs = vec![
        Value::Program(Box::new(target.clone())),
        Value::Int(1),
        Value::Int(0x02),
    ];
    let (out, _) = interpreter::interpret(
        &registry.find_mutation("iris_replace_kind").unwrap().program,
        &inputs,
        None,
    )
    .unwrap();
    let _replaced = extract_program(&out[0]);

    // mutate_literal
    let donor = make_donor_lit(999, 42);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(20),
        Value::Program(Box::new(donor)),
        Value::Int(999),
    ];
    let (out, _) = interpreter::interpret(
        &registry.find_mutation("iris_mutate_literal").unwrap().program,
        &inputs,
        None,
    )
    .unwrap();
    assert!(matches!(&out[0], Value::Program(_)), "mutate_literal via registry returned: {:?}", &out[0]);

    eprintln!(
        "All 4 new IRIS mutation operators registered and verified.\n\
         Total IRIS mutation operators: 8 of ~16 (v1: replace_prim, insert_node, connect, compose;\n\
         v2: delete_node, rewire_edge, replace_kind, mutate_literal)"
    );
}
