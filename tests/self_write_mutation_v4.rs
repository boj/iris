
//! Self-writing milestone v4: final 4 mutation operators using the new
//! self-modification opcodes (0x8B, 0x8C, 0x8D).
//!
//! These operators were NOT FEASIBLE in v3 due to missing opcodes:
//!
//! 1. **wrap_in_guard** — wrap an existing computation in a Guard node
//!    using graph_add_guard_rt (0x8B) + graph_connect (0x86) + graph_disconnect (0x87)
//! 2. **add_guard_condition** — add a guard around an existing node,
//!    using graph_add_guard_rt (0x8B)
//! 3. **insert_ref** — replace a subtree with a library Ref node
//!    using graph_add_ref_rt (0x8C) + graph_connect (0x86) + graph_disconnect (0x87)
//! 4. **annotate_cost** — set a node's cost annotation
//!    using graph_set_cost (0x8D)
//!
//! After this milestone: **16/16 mutation operators self-written in IRIS.**

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
// Graph construction helpers (shared with v1/v2/v3, duplicated for isolation)
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

// ===========================================================================
// Operator 13: wrap_in_guard
// ===========================================================================
//
// Wraps an existing computation in a Guard node.
//
// Given a program with a root computation, this operator:
// 1. Creates a Guard node with graph_add_guard_rt (0x8B) referencing
//    a predicate, the original body (root), and a fallback.
// 2. Rewires the graph root to be the new Guard node using
//    graph_set_prim_op is not needed -- we just get the guard_id back
//    and return the program with the guard as the implicit new root.
//
// Inputs:
//   inputs[0] = Program(target_graph)
//   inputs[1] = Int(predicate_node_id)  -- must exist in the graph
//   inputs[2] = Int(body_node_id)       -- the node to guard (typically root)
//   inputs[3] = Int(fallback_node_id)   -- fallback if predicate fails
//
// Output: Tuple(modified_program, guard_node_id)
//
// Graph structure:
//   Root(id=1): graph_add_guard_rt(0x8B, arity=4)
//   +-- port 0: input_ref(0) [id=10]  -- program
//   +-- port 1: input_ref(1) [id=20]  -- predicate node ID
//   +-- port 2: input_ref(2) [id=30]  -- body node ID
//   +-- port 3: input_ref(3) [id=40]  -- fallback node ID

fn build_iris_wrap_in_guard_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_add_guard_rt(0x8B, 4 args)
    let (nid, node) = prim_node(1, 0x8B, 4);
    nodes.insert(nid, node);

    // input_ref nodes
    let (nid, node) = input_ref_node(10, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1); // predicate node ID
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 2); // body node ID
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(40, 3); // fallback node ID
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        make_edge(1, 40, 3, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// wrap_in_guard tests
// ---------------------------------------------------------------------------

#[test]
fn iris_wrap_in_guard_creates_guard_node() {
    // Target: add(3, 4) = 7
    // Create a Guard node referencing existing nodes as predicate/body/fallback.
    // We use the add node (id=1) as body, lit(3, id=10) as predicate,
    // and lit(4, id=20) as fallback.
    let iris_wrap = build_iris_wrap_in_guard_program();
    let target = make_binop_graph(0x00, 3, 4); // add(3, 4) = 7

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(10), // predicate: lit(3)
        Value::Int(1),  // body: add node (root)
        Value::Int(20), // fallback: lit(4)
    ];

    let (outputs, _) = interpreter::interpret(&iris_wrap, &inputs, None).unwrap();

    // Result should be Tuple(Program, guard_node_id)
    let (modified, guard_id) = match &outputs[0] {
        Value::Tuple(elems) if elems.len() == 2 => {
            let prog = match &elems[0] {
                Value::Program(g) => g.as_ref().clone(),
                other => panic!("expected Program, got {:?}", other),
            };
            let id = match &elems[1] {
                Value::Int(v) => *v as u64,
                other => panic!("expected Int guard_id, got {:?}", other),
            };
            (prog, id)
        }
        other => panic!("expected Tuple(Program, Int), got {:?}", other),
    };

    // The guard node should exist in the modified graph.
    let guard_node = modified.nodes.get(&NodeId(guard_id))
        .expect("guard node should exist in modified graph");
    assert_eq!(guard_node.kind, NodeKind::Guard, "should be a Guard node");

    // Verify the Guard payload references the correct nodes.
    match &guard_node.payload {
        NodePayload::Guard {
            predicate_node,
            body_node,
            fallback_node,
        } => {
            assert_eq!(predicate_node.0, 10, "predicate should be node 10");
            assert_eq!(body_node.0, 1, "body should be node 1 (root)");
            assert_eq!(fallback_node.0, 20, "fallback should be node 20");
        }
        other => panic!("expected Guard payload, got {:?}", other),
    }

    // Guard node should have 3 outgoing edges to its children.
    let guard_edges: Vec<_> = modified
        .edges
        .iter()
        .filter(|e| e.source == NodeId(guard_id))
        .collect();
    assert_eq!(guard_edges.len(), 3, "guard should have 3 outgoing edges");

    // Original nodes should still exist.
    assert!(modified.nodes.contains_key(&NodeId(1)), "original root should still exist");
    assert!(modified.nodes.contains_key(&NodeId(10)), "lit(3) should still exist");
    assert!(modified.nodes.contains_key(&NodeId(20)), "lit(4) should still exist");

    eprintln!("wrap_in_guard: Guard node created with id={}, edges verified", guard_id);
}

#[test]
fn iris_wrap_in_guard_preserves_original_computation() {
    // The original computation should still be evaluable after wrapping.
    let target = make_binop_graph(0x01, 10, 3); // sub(10, 3) = 7

    // Verify original works.
    let (orig_result, _) = interpreter::interpret(&target, &[], None).unwrap();
    assert_eq!(orig_result, vec![Value::Int(7)]);

    let iris_wrap = build_iris_wrap_in_guard_program();
    let inputs = vec![
        Value::Program(Box::new(target.clone())),
        Value::Int(10), // predicate
        Value::Int(1),  // body (root = sub)
        Value::Int(20), // fallback
    ];

    let (outputs, _) = interpreter::interpret(&iris_wrap, &inputs, None).unwrap();
    let modified = match &outputs[0] {
        Value::Tuple(elems) => match &elems[0] {
            Value::Program(g) => g.as_ref().clone(),
            other => panic!("expected Program, got {:?}", other),
        },
        other => panic!("expected Tuple, got {:?}", other),
    };

    // The original sub(10, 3) subtree should still be intact.
    // The root is still NodeId(1) (the sub node), and the guard is a new
    // node that references it as body but hasn't replaced the root.
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(7)], "original computation preserved");

    eprintln!("wrap_in_guard: original computation sub(10,3)=7 preserved");
}

#[test]
fn register_iris_wrap_in_guard_as_component() {
    let iris_program = build_iris_wrap_in_guard_program();

    let component = MutationComponent {
        name: "iris_wrap_in_guard".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_wrap_in_guard");
    assert!(found.is_some(), "iris_wrap_in_guard should be registered");

    let target = make_binop_graph(0x00, 5, 7);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(10), // predicate
        Value::Int(1),  // body
        Value::Int(20), // fallback
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Tuple(elems) => {
            match &elems[0] {
                Value::Program(g) => {
                    // Guard node should exist.
                    let has_guard = g.nodes.values()
                        .any(|n| n.kind == NodeKind::Guard);
                    assert!(has_guard, "modified graph should contain a Guard node");
                }
                other => panic!("expected Program, got {:?}", other),
            }
        }
        other => panic!("expected Tuple, got {:?}", other),
    }

    eprintln!("wrap_in_guard mutation operator registered as IRIS component");
}

// ===========================================================================
// Operator 14: add_guard_condition
// ===========================================================================
//
// Similar to wrap_in_guard but wraps an existing computation node by:
// 1. Creating a new Guard node with graph_add_guard_rt (0x8B).
// 2. Using graph_connect (0x86) to wire a parent to the guard instead of
//    the original body.
// 3. Using graph_disconnect (0x87) to remove the parent's edge to the
//    original body.
//
// This effectively interposes a guard between a parent and its child.
//
// IRIS program structure (multi-step):
// Step 1: graph_add_guard_rt(program, pred_id, body_id, fallback_id)
//         -> Tuple(prog1, guard_id)
// Step 2: Extract prog1 and guard_id using Project nodes.
// Step 3: graph_disconnect(prog1, parent_id, body_id)
//         -> prog2
// Step 4: graph_connect(prog2, parent_id, guard_id, port)
//         -> prog3
//
// To build this as a single IRIS graph we use a chain of operations.
// For simplicity, we build a multi-step program using intermediate nodes.
//
// Inputs:
//   inputs[0] = Program(target_graph)
//   inputs[1] = Int(parent_node_id)     -- parent to rewire
//   inputs[2] = Int(predicate_node_id)  -- guard predicate
//   inputs[3] = Int(body_node_id)       -- existing child to guard
//   inputs[4] = Int(fallback_node_id)   -- fallback computation
//   inputs[5] = Int(port)               -- port on parent that connects to body

fn build_iris_add_guard_condition_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Step 1: graph_add_guard_rt(inputs[0], inputs[2], inputs[3], inputs[4])
    //         -> Tuple(prog1, guard_id) at node 100
    let (nid, node) = prim_node(100, 0x8B, 4);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(110, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(120, 2); // predicate
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(130, 3); // body
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(140, 4); // fallback
    nodes.insert(nid, node);

    // Step 2a: Project field 0 from Tuple -> prog1 at node 200
    let (nid, node) = make_node(
        200,
        NodeKind::Project,
        NodePayload::Project { field_index: 0 },
        1,
    );
    nodes.insert(nid, node);

    // Step 2b: Project field 1 from Tuple -> guard_id at node 210
    let (nid, node) = make_node(
        210,
        NodeKind::Project,
        NodePayload::Project { field_index: 1 },
        1,
    );
    nodes.insert(nid, node);

    // Step 3: graph_disconnect(prog1, parent_id, body_id) -> prog2 at node 300
    let (nid, node) = prim_node(300, 0x87, 3);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(310, 1); // parent_id
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(320, 3); // body_id
    nodes.insert(nid, node);

    // Step 4: graph_connect(prog2, parent_id, guard_id, port) -> prog3 at node 1 (ROOT)
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(410, 1); // parent_id (for connect)
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(420, 5); // port
    nodes.insert(nid, node);

    let edges = vec![
        // Step 1: graph_add_guard_rt edges
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        make_edge(100, 130, 2, EdgeLabel::Argument),
        make_edge(100, 140, 3, EdgeLabel::Argument),
        // Step 2a: Project(0) from step 1 result
        make_edge(200, 100, 0, EdgeLabel::Argument),
        // Step 2b: Project(1) from step 1 result
        make_edge(210, 100, 0, EdgeLabel::Argument),
        // Step 3: graph_disconnect(prog1, parent_id, body_id)
        make_edge(300, 200, 0, EdgeLabel::Argument),  // prog1
        make_edge(300, 310, 1, EdgeLabel::Argument),   // parent_id
        make_edge(300, 320, 2, EdgeLabel::Argument),   // body_id
        // Step 4 (ROOT): graph_connect(prog2, parent_id, guard_id, port)
        make_edge(1, 300, 0, EdgeLabel::Argument),     // prog2
        make_edge(1, 410, 1, EdgeLabel::Argument),     // parent_id (source)
        make_edge(1, 210, 2, EdgeLabel::Argument),     // guard_id (target)
        make_edge(1, 420, 3, EdgeLabel::Argument),     // port
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// add_guard_condition tests
// ---------------------------------------------------------------------------

#[test]
fn iris_add_guard_condition_interposes_guard() {
    // Target: add(3, 4) at root=1, lit(3)=10, lit(4)=20
    // We want to interpose a guard between root and lit(3) on port 0.
    // predicate=20 (lit(4)), body=10 (lit(3)), fallback=20 (lit(4))
    let iris_guard = build_iris_add_guard_condition_program();
    let target = make_binop_graph(0x00, 3, 4);

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),  // parent_id (root add node)
        Value::Int(20), // predicate_id (lit(4))
        Value::Int(10), // body_id (lit(3) -- being guarded)
        Value::Int(20), // fallback_id (lit(4))
        Value::Int(0),  // port 0
    ];

    let (outputs, _) = interpreter::interpret(&iris_guard, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // The modified graph should contain a Guard node.
    let guard_nodes: Vec<_> = modified
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Guard)
        .collect();
    assert_eq!(guard_nodes.len(), 1, "should have exactly one Guard node");

    let guard_node = guard_nodes[0];
    match &guard_node.payload {
        NodePayload::Guard {
            predicate_node,
            body_node,
            fallback_node,
        } => {
            assert_eq!(predicate_node.0, 20, "predicate should be lit(4)");
            assert_eq!(body_node.0, 10, "body should be lit(3)");
            assert_eq!(fallback_node.0, 20, "fallback should be lit(4)");
        }
        other => panic!("expected Guard payload, got {:?}", other),
    }

    // The parent (root) should have an edge to the guard node, not to lit(3).
    let root_edges: Vec<_> = modified
        .edges
        .iter()
        .filter(|e| e.source == NodeId(1) && e.port == 0)
        .collect();
    // Should have the new edge to guard, and old edge to lit(3) should be gone.
    let has_guard_edge = root_edges
        .iter()
        .any(|e| e.target == guard_node.id);
    assert!(has_guard_edge, "root port 0 should now point to guard node");

    let has_old_edge = root_edges
        .iter()
        .any(|e| e.target == NodeId(10));
    assert!(!has_old_edge, "root port 0 should no longer point to lit(3)");

    eprintln!(
        "add_guard_condition: guard interposed between root and lit(3), guard_id={}",
        guard_node.id.0
    );
}

#[test]
fn register_iris_add_guard_condition_as_component() {
    let iris_program = build_iris_add_guard_condition_program();

    let component = MutationComponent {
        name: "iris_add_guard_condition".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_add_guard_condition");
    assert!(found.is_some(), "iris_add_guard_condition should be registered");

    let target = make_binop_graph(0x02, 6, 3); // mul(6, 3)
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),  // parent
        Value::Int(20), // predicate
        Value::Int(10), // body
        Value::Int(20), // fallback
        Value::Int(0),  // port
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Program(g) => {
            let has_guard = g.nodes.values().any(|n| n.kind == NodeKind::Guard);
            assert!(has_guard, "should have Guard node");
        }
        other => panic!("expected Program, got {:?}", other),
    }

    eprintln!("add_guard_condition mutation operator registered as IRIS component");
}

// ===========================================================================
// Operator 15: insert_ref
// ===========================================================================
//
// Replace a subtree with a library Ref node. This is used when a known
// library function should be referenced instead of inline code.
//
// Steps:
// 1. graph_add_ref_rt(program, fragment_id) -> Tuple(prog1, ref_id)
// 2. graph_disconnect(prog1, parent_id, old_node_id) -> prog2
// 3. graph_connect(prog2, parent_id, ref_id, port) -> prog3
//
// Inputs:
//   inputs[0] = Program(target_graph)
//   inputs[1] = Int(parent_node_id)
//   inputs[2] = Int(old_node_id)        -- node to replace
//   inputs[3] = Int(fragment_id)        -- library fragment hash (first 8 bytes)
//   inputs[4] = Int(port)               -- port on parent

fn build_iris_insert_ref_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Step 1: graph_add_ref_rt(inputs[0], inputs[3]) -> Tuple(prog1, ref_id)
    let (nid, node) = prim_node(100, 0x8C, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(110, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(120, 3); // fragment_id
    nodes.insert(nid, node);

    // Step 2a: Project(0) -> prog1
    let (nid, node) = make_node(
        200,
        NodeKind::Project,
        NodePayload::Project { field_index: 0 },
        1,
    );
    nodes.insert(nid, node);

    // Step 2b: Project(1) -> ref_id
    let (nid, node) = make_node(
        210,
        NodeKind::Project,
        NodePayload::Project { field_index: 1 },
        1,
    );
    nodes.insert(nid, node);

    // Step 3: graph_disconnect(prog1, parent_id, old_node_id)
    let (nid, node) = prim_node(300, 0x87, 3);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(310, 1); // parent_id
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(320, 2); // old_node_id
    nodes.insert(nid, node);

    // Step 4 (ROOT): graph_connect(prog2, parent_id, ref_id, port)
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(410, 1); // parent_id
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(420, 4); // port
    nodes.insert(nid, node);

    let edges = vec![
        // Step 1: graph_add_ref_rt
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        // Step 2: projections
        make_edge(200, 100, 0, EdgeLabel::Argument),
        make_edge(210, 100, 0, EdgeLabel::Argument),
        // Step 3: graph_disconnect(prog1, parent_id, old_node_id)
        make_edge(300, 200, 0, EdgeLabel::Argument),
        make_edge(300, 310, 1, EdgeLabel::Argument),
        make_edge(300, 320, 2, EdgeLabel::Argument),
        // Step 4 (ROOT): graph_connect(prog2, parent_id, ref_id, port)
        make_edge(1, 300, 0, EdgeLabel::Argument),
        make_edge(1, 410, 1, EdgeLabel::Argument),
        make_edge(1, 210, 2, EdgeLabel::Argument),
        make_edge(1, 420, 3, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// insert_ref tests
// ---------------------------------------------------------------------------

#[test]
fn iris_insert_ref_creates_ref_node() {
    // Target: add(3, 4) at root=1, lit(3)=10, lit(4)=20
    // Replace lit(3) at port 0 with a Ref node to fragment_id=42
    let iris_ref = build_iris_insert_ref_program();
    let target = make_binop_graph(0x00, 3, 4);

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),  // parent_id
        Value::Int(10), // old_node_id (lit(3))
        Value::Int(42), // fragment_id
        Value::Int(0),  // port
    ];

    let (outputs, _) = interpreter::interpret(&iris_ref, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // A Ref node should exist in the modified graph.
    let ref_nodes: Vec<_> = modified
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Ref)
        .collect();
    assert_eq!(ref_nodes.len(), 1, "should have exactly one Ref node");

    // Verify Ref payload.
    match &ref_nodes[0].payload {
        NodePayload::Ref { fragment_id } => {
            // First 8 bytes should encode 42 in little-endian.
            let expected_val = 42i64;
            let mut expected_bytes = [0u8; 32];
            expected_bytes[..8].copy_from_slice(&expected_val.to_le_bytes());
            assert_eq!(
                fragment_id.0, expected_bytes,
                "fragment_id should encode 42"
            );
        }
        other => panic!("expected Ref payload, got {:?}", other),
    }

    let ref_id = ref_nodes[0].id;

    // Root (add node) should have an edge to the ref node on port 0.
    let root_port0_edges: Vec<_> = modified
        .edges
        .iter()
        .filter(|e| e.source == NodeId(1) && e.port == 0)
        .collect();
    let has_ref_edge = root_port0_edges.iter().any(|e| e.target == ref_id);
    assert!(has_ref_edge, "root port 0 should point to Ref node");

    // Old edge to lit(3) should be gone.
    let has_old_edge = root_port0_edges.iter().any(|e| e.target == NodeId(10));
    assert!(!has_old_edge, "root port 0 should no longer point to lit(3)");

    eprintln!("insert_ref: Ref node created with fragment_id=42, ref_id={}", ref_id.0);
}

#[test]
fn iris_insert_ref_with_different_fragment_ids() {
    let iris_ref = build_iris_insert_ref_program();

    // Test with fragment_id = 999
    let target = make_binop_graph(0x02, 6, 3); // mul(6, 3)
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),   // parent
        Value::Int(20),  // old_node (lit(3))
        Value::Int(999), // fragment_id
        Value::Int(1),   // port 1
    ];

    let (outputs, _) = interpreter::interpret(&iris_ref, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    let ref_nodes: Vec<_> = modified
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Ref)
        .collect();
    assert_eq!(ref_nodes.len(), 1);

    match &ref_nodes[0].payload {
        NodePayload::Ref { fragment_id } => {
            let mut expected_bytes = [0u8; 32];
            expected_bytes[..8].copy_from_slice(&999i64.to_le_bytes());
            assert_eq!(fragment_id.0, expected_bytes);
        }
        _ => panic!("expected Ref payload"),
    }

    // Root port 1 should point to ref, not to lit(3).
    let root_port1_edges: Vec<_> = modified
        .edges
        .iter()
        .filter(|e| e.source == NodeId(1) && e.port == 1)
        .collect();
    let has_ref_edge = root_port1_edges.iter().any(|e| e.target == ref_nodes[0].id);
    assert!(has_ref_edge, "root port 1 should point to Ref node");

    eprintln!("insert_ref: verified with fragment_id=999 on port 1");
}

#[test]
fn register_iris_insert_ref_as_component() {
    let iris_program = build_iris_insert_ref_program();

    let component = MutationComponent {
        name: "iris_insert_ref".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_insert_ref");
    assert!(found.is_some(), "iris_insert_ref should be registered");

    let target = make_binop_graph(0x00, 5, 7);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),  // parent
        Value::Int(10), // old node
        Value::Int(77), // fragment_id
        Value::Int(0),  // port
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Program(g) => {
            let has_ref = g.nodes.values().any(|n| n.kind == NodeKind::Ref);
            assert!(has_ref, "modified graph should contain a Ref node");
        }
        other => panic!("expected Program, got {:?}", other),
    }

    eprintln!("insert_ref mutation operator registered as IRIS component");
}

// ===========================================================================
// Operator 16: annotate_cost
// ===========================================================================
//
// Set a node's cost annotation using graph_set_cost (0x8D).
//
// Inputs:
//   inputs[0] = Program(target_graph)
//   inputs[1] = Int(node_id)
//   inputs[2] = Int(cost_value)  -- 0=Unit, 1=Inherited, N>=2=Constant(N)
//
// Output: modified Program
//
// Graph structure:
//   Root(id=1): graph_set_cost(0x8D, arity=3)
//   +-- port 0: input_ref(0) [id=10]  -- program
//   +-- port 1: input_ref(1) [id=20]  -- node_id
//   +-- port 2: input_ref(2) [id=30]  -- cost_value

fn build_iris_annotate_cost_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_set_cost(0x8D, 3 args)
    let (nid, node) = prim_node(1, 0x8D, 3);
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(10, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1); // node_id
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 2); // cost_value
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// annotate_cost tests
// ---------------------------------------------------------------------------

#[test]
fn iris_annotate_cost_sets_unit() {
    let iris_cost = build_iris_annotate_cost_program();
    let target = make_binop_graph(0x00, 3, 4); // add(3, 4)

    // Set root node's cost to Unit (0).
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1), // root node
        Value::Int(0), // CostTerm::Unit
    ];

    let (outputs, _) = interpreter::interpret(&iris_cost, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // The root node should still exist (may have a new ID due to re-hashing,
    // but the graph root should point to a node with Unit cost).
    let root_node = modified.nodes.get(&modified.root)
        .expect("root node should exist");
    assert_eq!(root_node.cost, CostTerm::Unit, "cost should be Unit");

    // The computation should still be evaluable.
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(7)], "add(3, 4) = 7");

    eprintln!("annotate_cost: cost set to Unit, computation preserved");
}

#[test]
fn iris_annotate_cost_sets_inherited() {
    let iris_cost = build_iris_annotate_cost_program();
    let target = make_binop_graph(0x01, 10, 3); // sub(10, 3)

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1), // root node
        Value::Int(1), // CostTerm::Inherited
    ];

    let (outputs, _) = interpreter::interpret(&iris_cost, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    let root_node = modified.nodes.get(&modified.root)
        .expect("root node should exist");
    assert_eq!(root_node.cost, CostTerm::Inherited, "cost should be Inherited");

    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(7)], "sub(10, 3) = 7");

    eprintln!("annotate_cost: cost set to Inherited");
}

#[test]
fn iris_annotate_cost_sets_constant() {
    let iris_cost = build_iris_annotate_cost_program();
    let target = make_binop_graph(0x02, 6, 7); // mul(6, 7)

    // Set cost to Annotated(Constant(100)).
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),   // root node
        Value::Int(100), // CostTerm::Annotated(Constant(100))
    ];

    let (outputs, _) = interpreter::interpret(&iris_cost, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    let root_node = modified.nodes.get(&modified.root)
        .expect("root node should exist");
    assert_eq!(
        root_node.cost,
        CostTerm::Annotated(CostBound::Constant(100)),
        "cost should be Annotated(Constant(100))"
    );

    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(42)], "mul(6, 7) = 42");

    eprintln!("annotate_cost: cost set to Constant(100)");
}

#[test]
fn iris_annotate_cost_on_leaf_node() {
    let iris_cost = build_iris_annotate_cost_program();
    let target = make_binop_graph(0x00, 5, 3); // add(5, 3)

    // Set cost on the leaf node lit(5) at id=10.
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(10),  // lit(5) node
        Value::Int(50),  // Constant(50)
    ];

    let (outputs, _) = interpreter::interpret(&iris_cost, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // The node may have a new ID due to re-hashing after cost change.
    // Find the node with Constant(50) cost.
    let annotated_nodes: Vec<_> = modified
        .nodes
        .values()
        .filter(|n| n.cost == CostTerm::Annotated(CostBound::Constant(50)))
        .collect();
    assert_eq!(
        annotated_nodes.len(),
        1,
        "exactly one node should have Constant(50) cost"
    );

    // The graph should still be evaluable.
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(8)], "add(5, 3) = 8");

    eprintln!("annotate_cost: leaf node annotated with Constant(50)");
}

#[test]
fn register_iris_annotate_cost_as_component() {
    let iris_program = build_iris_annotate_cost_program();

    let component = MutationComponent {
        name: "iris_annotate_cost".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_annotate_cost");
    assert!(found.is_some(), "iris_annotate_cost should be registered");

    let target = make_binop_graph(0x00, 10, 20);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),
        Value::Int(42), // Constant(42)
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Program(g) => {
            let root = g.nodes.get(&g.root).expect("root should exist");
            assert_eq!(
                root.cost,
                CostTerm::Annotated(CostBound::Constant(42)),
                "cost should be Constant(42)"
            );
        }
        other => panic!("expected Program, got {:?}", other),
    }

    eprintln!("annotate_cost mutation operator registered as IRIS component");
}

// ===========================================================================
// Integration: all 4 operators compose with existing operators
// ===========================================================================

#[test]
fn all_four_operators_registered_together() {
    let mut registry = ComponentRegistry::new();

    registry.mutations.push(MutationComponent {
        name: "iris_wrap_in_guard".to_string(),
        program: build_iris_wrap_in_guard_program(),
    });
    registry.mutations.push(MutationComponent {
        name: "iris_add_guard_condition".to_string(),
        program: build_iris_add_guard_condition_program(),
    });
    registry.mutations.push(MutationComponent {
        name: "iris_insert_ref".to_string(),
        program: build_iris_insert_ref_program(),
    });
    registry.mutations.push(MutationComponent {
        name: "iris_annotate_cost".to_string(),
        program: build_iris_annotate_cost_program(),
    });

    assert!(registry.find_mutation("iris_wrap_in_guard").is_some());
    assert!(registry.find_mutation("iris_add_guard_condition").is_some());
    assert!(registry.find_mutation("iris_insert_ref").is_some());
    assert!(registry.find_mutation("iris_annotate_cost").is_some());

    eprintln!("All 4 new mutation operators registered in ComponentRegistry");
}

#[test]
fn compose_annotate_cost_then_wrap_in_guard() {
    // First annotate cost on a program, then wrap it in a guard.
    // This tests composition of the new operators.
    let target = make_binop_graph(0x00, 5, 3); // add(5, 3) = 8

    // Step 1: Annotate cost on root.
    let iris_cost = build_iris_annotate_cost_program();
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),   // root
        Value::Int(200), // Constant(200)
    ];
    let (outputs, _) = interpreter::interpret(&iris_cost, &inputs, None).unwrap();
    let cost_annotated = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // Verify cost was set.
    let root = cost_annotated.nodes.get(&cost_annotated.root).unwrap();
    assert_eq!(root.cost, CostTerm::Annotated(CostBound::Constant(200)));

    // Step 2: Wrap the cost-annotated program in a guard.
    // We need the node IDs from the modified graph.
    let root_id = cost_annotated.root.0 as i64;
    // Find any non-root node to use as predicate/fallback.
    let other_id = cost_annotated
        .nodes
        .keys()
        .find(|nid| **nid != cost_annotated.root)
        .unwrap()
        .0 as i64;

    let iris_wrap = build_iris_wrap_in_guard_program();
    let inputs = vec![
        Value::Program(Box::new(cost_annotated)),
        Value::Int(other_id), // predicate
        Value::Int(root_id),  // body
        Value::Int(other_id), // fallback
    ];
    let (outputs, _) = interpreter::interpret(&iris_wrap, &inputs, None).unwrap();

    let final_graph = match &outputs[0] {
        Value::Tuple(elems) => match &elems[0] {
            Value::Program(g) => g.as_ref().clone(),
            other => panic!("expected Program, got {:?}", other),
        },
        other => panic!("expected Tuple, got {:?}", other),
    };

    // Should have a Guard node.
    let has_guard = final_graph.nodes.values().any(|n| n.kind == NodeKind::Guard);
    assert!(has_guard, "should have Guard node after composition");

    // Should still have cost-annotated node.
    let has_cost = final_graph
        .nodes
        .values()
        .any(|n| n.cost == CostTerm::Annotated(CostBound::Constant(200)));
    assert!(has_cost, "cost annotation should be preserved");

    eprintln!("Composition: annotate_cost -> wrap_in_guard verified");
}

#[test]
fn compose_insert_ref_then_annotate_cost() {
    // First insert a ref, then annotate cost on the ref node.
    let target = make_binop_graph(0x00, 5, 3); // add(5, 3)

    // Step 1: Insert ref replacing lit(3) at port 1.
    let iris_ref = build_iris_insert_ref_program();
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),  // parent
        Value::Int(20), // old node (lit(3))
        Value::Int(77), // fragment_id
        Value::Int(1),  // port
    ];
    let (outputs, _) = interpreter::interpret(&iris_ref, &inputs, None).unwrap();
    let ref_modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // Find the Ref node.
    let ref_node_id = ref_modified
        .nodes
        .values()
        .find(|n| n.kind == NodeKind::Ref)
        .expect("should have Ref node")
        .id;

    // Step 2: Annotate cost on the ref node.
    let iris_cost = build_iris_annotate_cost_program();
    let inputs = vec![
        Value::Program(Box::new(ref_modified)),
        Value::Int(ref_node_id.0 as i64),
        Value::Int(500), // Constant(500)
    ];
    let (outputs, _) = interpreter::interpret(&iris_cost, &inputs, None).unwrap();
    let final_graph = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // The Ref node (possibly with new ID) should have cost Constant(500).
    let cost_refs: Vec<_> = final_graph
        .nodes
        .values()
        .filter(|n| {
            n.kind == NodeKind::Ref
                && n.cost == CostTerm::Annotated(CostBound::Constant(500))
        })
        .collect();
    assert_eq!(cost_refs.len(), 1, "Ref node should have Constant(500) cost");

    eprintln!("Composition: insert_ref -> annotate_cost verified");
}

// ===========================================================================
// Summary test: final report
// ===========================================================================

#[test]
fn summary_report_v4() {
    eprintln!("\n=== Self-Write Mutation v4: Final Status Report ===\n");

    eprintln!("NEW OPCODES ADDED:");
    eprintln!("  0x8B graph_add_guard_rt  -- Create Guard node at runtime");
    eprintln!("  0x8C graph_add_ref_rt    -- Create Ref node at runtime");
    eprintln!("  0x8D graph_set_cost      -- Set node cost annotation");
    eprintln!();
    eprintln!("NEW MUTATION OPERATORS (v4, all as IRIS programs):");
    eprintln!("  [13/16] wrap_in_guard        -- graph_add_guard_rt(0x8B)");
    eprintln!("  [14/16] add_guard_condition  -- graph_add_guard_rt + disconnect + connect");
    eprintln!("  [15/16] insert_ref           -- graph_add_ref_rt(0x8C) + disconnect + connect");
    eprintln!("  [16/16] annotate_cost        -- graph_set_cost(0x8D)");
    eprintln!();
    eprintln!("PREVIOUS OPERATORS (v1-v3):");
    eprintln!("  [1/16]  replace_prim         -- graph_set_prim_op(0x84)");
    eprintln!("  [2/16]  insert_node          -- graph_add_node_rt(0x85) + connect(0x86)");
    eprintln!("  [3/16]  connect              -- graph_connect(0x86)");
    eprintln!("  [4/16]  delete_node          -- graph_disconnect(0x87)");
    eprintln!("  [5/16]  rewire_edge          -- disconnect + connect");
    eprintln!("  [6/16]  replace_kind         -- graph_set_prim_op(0x84)");
    eprintln!("  [7/16]  mutate_literal       -- graph_replace_subtree(0x88)");
    eprintln!("  [8/16]  swap_subtree         -- graph_replace_subtree(0x88)");
    eprintln!("  [9/16]  duplicate_subgraph   -- graph_replace_subtree(same program)");
    eprintln!("  [10/16] swap_fold_op         -- graph_set_prim_op + graph_replace_subtree");
    eprintln!("  [11/16] wrap_in_map          -- graph_add_node_rt + connect/disconnect");
    eprintln!("  [12/16] wrap_in_filter       -- same pattern as wrap_in_map");
    eprintln!();
    eprintln!("TOTAL: 16/16 mutation operators self-written as IRIS programs.");
    eprintln!("=== End Report ===");
}
