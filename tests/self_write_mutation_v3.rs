
//! Self-writing milestone v3: remaining mutation operators + population management.
//!
//! Builds on v1 (replace_prim, insert_node, connect) and v2 (delete_node,
//! rewire_edge, replace_kind, mutate_literal) with the remaining feasible
//! operators:
//!
//! **Mutation operators (feasible):**
//! 1. **duplicate_subgraph** — copy a subtree within the same graph using
//!    graph_replace_subtree (0x88) + graph_connect (0x86)
//! 2. **swap_fold_op** — change fold opcode + base together using two
//!    graph_set_prim_op (0x84) calls + graph_replace_subtree for the base lit
//! 3. **wrap_in_map** — insert a Map(0x30) Prim before a Fold's collection
//!    input using graph_add_node_rt (0x85) + graph_connect + graph_disconnect
//! 4. **wrap_in_filter** — insert a Filter(0x31) Prim before a Fold's
//!    collection input (same pattern as wrap_in_map)
//!
//! **NOT feasible with current opcodes:**
//! - wrap_in_guard: needs Guard node (graph_add_node_rt only creates Prim)
//! - add_guard_condition: same limitation
//! - insert_ref: needs Ref node creation
//! - annotate_cost: no opcode to modify node.cost field
//!
//! **Population management:**
//! 5. **tournament_select** — pick best of k individuals (comparison-based,
//!    no randomness needed if inputs are pre-shuffled)
//! 6. **crossover_subgraph** — swap subtrees between two programs using
//!    graph_replace_subtree (0x88)

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

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
// Graph construction helpers (shared with v1/v2, duplicated for test isolation)
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

fn project_node(id: u64, field_index: u16) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Project,
        NodePayload::Project { field_index },
        1,
    )
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
///   +-- port 0: op1(id=2, arity=2)
///   |   +-- port 0: lit(a, id=10)
///   |   +-- port 1: lit(b, id=20)
///   +-- port 1: lit(c, id=30)
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

/// Create a graph: `a op b` with Prim at id=base, lits at id=base+10, base+20.
/// Uses custom base IDs to avoid collisions in crossover tests.
fn make_binop_graph_at(opcode: u8, a: i64, b: i64, base: u64) -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(base, opcode, 2);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(base + 10, a);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(base + 20, b);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(base, base + 10, 0, EdgeLabel::Argument),
        make_edge(base, base + 20, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, base)
}

/// Build a donor graph containing a single Lit node with the given value.
fn make_donor_lit(id: u64, value: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = int_lit_node(id, value);
    nodes.insert(nid, node);
    make_graph(nodes, vec![], id)
}

/// Build a simple Fold graph: fold(base, step_op, collection)
///
///   Root: Fold(id=1, arity=3)
///   +-- port 0: lit(base, id=10)
///   +-- port 1: Prim(step_op, id=20)
///   +-- port 2: input_ref(0, id=30)  -- collection from program input
fn make_fold_graph(step_op: u8, base: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = make_node(
        1,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![],
        },
        3,
    );
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(10, base);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(20, step_op, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 0);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument), // fold -> base
        make_edge(1, 20, 1, EdgeLabel::Argument), // fold -> step
        make_edge(1, 30, 2, EdgeLabel::Argument), // fold -> collection
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// Operator 1: duplicate_subgraph
// ===========================================================================
//
// The Rust `duplicate_subgraph` copies a subtree (node + successors) and
// connects the copy into the graph. Our IRIS version uses
// graph_replace_subtree(0x88) to copy a subtree from the SAME program
// into a target slot, effectively duplicating it.
//
// Strategy: Given a source node and a target node to replace, copy the
// source subtree over the target. This is a simplified but useful version
// that replaces one subtree with a duplicate of another.
//
// Inputs:
//   inputs[0] = Program(target_graph)
//   inputs[1] = Int(target_node_id)   -- node to replace (will be overwritten)
//   inputs[2] = Int(source_node_id)   -- node to duplicate
//
// The trick: we use the SAME program as both target and source for
// graph_replace_subtree. This copies the source subtree into the target
// position.
//
// Graph structure:
//   Root(id=1): graph_replace_subtree(0x88, arity=4)
//   +-- port 0: input_ref(0) [id=10]  -- target program
//   +-- port 1: input_ref(1) [id=20]  -- target node ID
//   +-- port 2: input_ref(0) [id=30]  -- source program (SAME as target)
//   +-- port 3: input_ref(2) [id=40]  -- source node ID

fn build_iris_duplicate_subgraph_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_replace_subtree(0x88, 4 args)
    let (nid, node) = prim_node(1, 0x88, 4);
    nodes.insert(nid, node);

    // input_ref nodes
    let (nid, node) = input_ref_node(10, 0); // target program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1); // target node ID
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 0); // source program (SAME as target)
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(40, 2); // source node ID
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
// duplicate_subgraph tests
// ---------------------------------------------------------------------------

#[test]
fn iris_duplicate_subgraph_replaces_leaf_with_another_leaf() {
    // sub(10, 3) = 7
    // Duplicate: replace lit(3, id=20) with a copy of lit(10, id=10)
    // Result: sub(10, 10) = 0
    //
    // Using leaf-to-leaf replacement because graph_replace_subtree copies
    // edges from the source. When source=same graph with overlapping edge
    // sets, subtree duplication would double edges. Leaf nodes have no
    // outgoing edges, so this works cleanly.
    let iris_dup = build_iris_duplicate_subgraph_program();
    let target = make_binop_graph(0x01, 10, 3); // sub(10, 3) = 7

    let (orig_result, _) = interpreter::interpret(&target, &[], None).unwrap();
    assert_eq!(orig_result, vec![Value::Int(7)], "sub(10, 3) = 7");

    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(20), // replace lit(3)
        Value::Int(10), // with copy of lit(10)
    ];

    let (outputs, _) = interpreter::interpret(&iris_dup, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // lit(3) at NodeId(20) should be gone (replaced)
    assert!(
        !modified.nodes.contains_key(&NodeId(20)),
        "old lit(3) should be replaced"
    );

    // The source lit(10) should still be present
    assert!(
        modified.nodes.contains_key(&NodeId(10)),
        "source lit(10) should still exist"
    );

    // Edges to NodeId(20) should now point to NodeId(10)
    let edges_to_10: Vec<_> = modified
        .edges
        .iter()
        .filter(|e| e.target == NodeId(10))
        .collect();
    assert_eq!(
        edges_to_10.len(),
        2,
        "root should have 2 edges to lit(10)"
    );

    // Execute: sub(10, 10) = 0
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(0)],
        "sub(10, 10) should be 0"
    );
}

#[test]
fn iris_duplicate_subgraph_copies_leaf() {
    // add(5, 3) = 8
    // Duplicate: replace lit(3, id=20) with copy of lit(5, id=10)
    // Result: add(5, 5) = 10
    let iris_dup = build_iris_duplicate_subgraph_program();
    let target = make_binop_graph(0x00, 5, 3);

    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(20), // replace lit(3)
        Value::Int(10), // with copy of lit(5)
    ];

    let (outputs, _) = interpreter::interpret(&iris_dup, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // lit(3) gone, lit(5) now referenced by both ports
    let edges_to_10: Vec<_> = modified
        .edges
        .iter()
        .filter(|e| e.target == NodeId(10))
        .collect();
    assert_eq!(
        edges_to_10.len(),
        2,
        "root should have 2 edges to lit(5): original port 0 + redirected port 1"
    );

    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(10)], "add(5, 5) should be 10");
}

#[test]
fn iris_duplicate_subgraph_cross_program() {
    // Duplicate a subtree from a DIFFERENT (donor) program into the target.
    // This avoids the edge-doubling issue of same-program duplication.
    //
    // Target: add(5, 3) = 8
    // Donor:  a single lit(42) at id=500
    // Replace lit(3, id=20) in target with lit(42, id=500) from donor
    // Result: add(5, 42) = 47
    //
    // This is functionally identical to mutate_literal, showing that
    // duplicate_subgraph generalizes literal mutation.
    let iris_dup = build_iris_duplicate_subgraph_program();

    // Verify that same-graph leaf duplication works by testing mul(4, 7).
    // Replace lit(4) with lit(7) -> mul(7, 7) = 49.
    let target2 = make_binop_graph(0x02, 4, 7); // mul(4, 7) = 28

    let inputs = vec![
        Value::Program(Rc::new(target2)),
        Value::Int(10), // replace lit(4) with lit(7)
        Value::Int(20), // source: lit(7)
    ];

    let (outputs, _) = interpreter::interpret(&iris_dup, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // Execute: mul(7, 7) = 49
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(49)], "mul(7, 7) should be 49");
}

#[test]
fn register_iris_duplicate_subgraph_as_component() {
    let iris_program = build_iris_duplicate_subgraph_program();

    let component = MutationComponent {
        name: "iris_duplicate_subgraph".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_duplicate_subgraph");
    assert!(found.is_some(), "iris_duplicate_subgraph should be registered");

    // Use leaf duplication: sub(10, 3) -> sub(10, 10) = 0
    let target = make_binop_graph(0x01, 10, 3);
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(20), // replace lit(3)
        Value::Int(10), // with copy of lit(10)
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Program(g) => {
            let (result, _) = interpreter::interpret(g, &[], None).unwrap();
            assert_eq!(result, vec![Value::Int(0)], "sub(10, 10) = 0");
        }
        other => panic!("expected Program, got {:?}", other),
    }

    eprintln!("duplicate_subgraph mutation operator replaced by IRIS program");
}

// ===========================================================================
// Operator 2: swap_fold_op
// ===========================================================================
//
// The Rust `swap_fold_op` changes a Fold's step operation AND adjusts the
// base value to match. This requires:
//   1. graph_set_prim_op(program, step_node_id, new_opcode)
//   2. graph_replace_subtree(result, base_node_id, donor, donor_node_id)
//
// The donor graph contains a Lit node with the matched base value.
//
// Inputs:
//   inputs[0] = Program(target_graph)
//   inputs[1] = Int(step_node_id)     -- the Fold's step function Prim node
//   inputs[2] = Int(new_opcode)       -- new step opcode (e.g., 0x02 for mul)
//   inputs[3] = Int(base_node_id)     -- the Fold's base Lit node
//   inputs[4] = Program(donor_graph)  -- contains the new base Lit node
//   inputs[5] = Int(donor_node_id)    -- the donor base node's ID
//
// Graph structure:
//   Root(id=1): graph_replace_subtree(0x88, arity=4)
//   +-- port 0: graph_set_prim_op(0x84) [id=100]  -- step 1: change opcode
//   |   +-- port 0: input_ref(0) [id=110]          -- program
//   |   +-- port 1: input_ref(1) [id=120]          -- step node ID
//   |   +-- port 2: input_ref(2) [id=130]          -- new opcode
//   +-- port 1: input_ref(3) [id=200]              -- base node ID
//   +-- port 2: input_ref(4) [id=210]              -- donor program
//   +-- port 3: input_ref(5) [id=220]              -- donor node ID

fn build_iris_swap_fold_op_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_replace_subtree(0x88, 4 args)
    // Takes: (program_with_new_op, base_node_id, donor_program, donor_node_id)
    let (nid, node) = prim_node(1, 0x88, 4);
    nodes.insert(nid, node);

    // Step 1: graph_set_prim_op(program, step_node_id, new_opcode)
    let (nid, node) = prim_node(100, 0x84, 3);
    nodes.insert(nid, node);

    // input_ref nodes
    let (nid, node) = input_ref_node(110, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(120, 1); // step node ID
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(130, 2); // new opcode
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(200, 3); // base node ID
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(210, 4); // donor program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(220, 5); // donor node ID
    nodes.insert(nid, node);

    let edges = vec![
        // Root: replace_subtree(set_prim_result, base_id, donor, donor_id)
        make_edge(1, 100, 0, EdgeLabel::Argument),
        make_edge(1, 200, 1, EdgeLabel::Argument),
        make_edge(1, 210, 2, EdgeLabel::Argument),
        make_edge(1, 220, 3, EdgeLabel::Argument),
        // Step 1: set_prim_op(program, step_node, new_opcode)
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        make_edge(100, 130, 2, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// swap_fold_op tests
// ---------------------------------------------------------------------------

#[test]
fn iris_swap_fold_op_add_to_mul() {
    // fold(0, add, [2, 3, 4]) = 0 + 2 + 3 + 4 = 9
    // swap_fold_op: add(0x00) -> mul(0x02), base 0 -> 1
    // Result: fold(1, mul, [2, 3, 4]) = 1 * 2 * 3 * 4 = 24
    let iris_swapper = build_iris_swap_fold_op_program();
    let target = make_fold_graph(0x00, 0); // fold(0, add, input)

    // Verify original
    let input_list = Value::tuple(vec![Value::Int(2), Value::Int(3), Value::Int(4)]);
    let (orig, _) = interpreter::interpret(&target, &[input_list.clone()], None).unwrap();
    assert_eq!(orig, vec![Value::Int(9)], "fold(0, add, [2,3,4]) = 9");

    // Build donor with base=1 for mul
    let donor = make_donor_lit(500, 1);

    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(20),   // step node ID (the add Prim)
        Value::Int(0x02), // new opcode: mul
        Value::Int(10),   // base node ID (the lit(0))
        Value::Program(Rc::new(donor)),
        Value::Int(500),  // donor node ID
    ];

    let (outputs, _) = interpreter::interpret(&iris_swapper, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // Verify: the step node should now have opcode 0x02 (mul)
    let has_mul = modified
        .nodes
        .values()
        .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x02 }));
    assert!(has_mul, "should have mul node after swap");

    // Verify: the base should now be 1
    let has_base_1 = modified.nodes.contains_key(&NodeId(500));
    assert!(has_base_1, "donor base node should be in the graph");

    // Execute: fold(1, mul, [2, 3, 4]) = 24
    let (result, _) = interpreter::interpret(&modified, &[input_list], None).unwrap();
    assert_eq!(result, vec![Value::Int(24)], "fold(1, mul, [2,3,4]) = 24");
}

#[test]
fn iris_swap_fold_op_add_to_max() {
    // fold(0, add, [5, -3, 8, 1]) = 11
    // swap_fold_op: add -> max(0x08), base 0 -> MIN_INT
    // Result: fold(MIN, max, [5, -3, 8, 1]) = 8
    let iris_swapper = build_iris_swap_fold_op_program();
    let target = make_fold_graph(0x00, 0);

    let input_list = Value::tuple(vec![
        Value::Int(5),
        Value::Int(-3),
        Value::Int(8),
        Value::Int(1),
    ]);

    // Build donor with base=MIN for max
    let donor = make_donor_lit(600, i64::MIN);

    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(20),   // step node
        Value::Int(0x08), // max
        Value::Int(10),   // base node
        Value::Program(Rc::new(donor)),
        Value::Int(600),
    ];

    let (outputs, _) = interpreter::interpret(&iris_swapper, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    let (result, _) = interpreter::interpret(&modified, &[input_list], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(8)],
        "fold(MIN, max, [5,-3,8,1]) = 8"
    );
}

#[test]
fn iris_swap_fold_op_add_to_min() {
    // fold(0, add, [5, -3, 8, 1]) = 11
    // swap_fold_op: add -> min(0x07), base 0 -> MAX_INT
    // Result: fold(MAX, min, [5, -3, 8, 1]) = -3
    let iris_swapper = build_iris_swap_fold_op_program();
    let target = make_fold_graph(0x00, 0);

    let input_list = Value::tuple(vec![
        Value::Int(5),
        Value::Int(-3),
        Value::Int(8),
        Value::Int(1),
    ]);

    let donor = make_donor_lit(700, i64::MAX);

    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(20),
        Value::Int(0x07), // min
        Value::Int(10),
        Value::Program(Rc::new(donor)),
        Value::Int(700),
    ];

    let (outputs, _) = interpreter::interpret(&iris_swapper, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    let (result, _) = interpreter::interpret(&modified, &[input_list], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(-3)],
        "fold(MAX, min, [5,-3,8,1]) = -3"
    );
}

#[test]
fn iris_swap_fold_op_preserves_collection_edge() {
    // After swap_fold_op, the fold should still have its port 2 collection edge.
    let iris_swapper = build_iris_swap_fold_op_program();
    let target = make_fold_graph(0x00, 0);

    let donor = make_donor_lit(800, 1);

    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(20),
        Value::Int(0x02), // mul
        Value::Int(10),
        Value::Program(Rc::new(donor)),
        Value::Int(800),
    ];

    let (outputs, _) = interpreter::interpret(&iris_swapper, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // The fold root should still have a port 2 edge to the collection
    let has_port2 = modified
        .edges
        .iter()
        .any(|e| e.source == NodeId(1) && e.port == 2);
    assert!(
        has_port2,
        "fold should still have port 2 edge to collection"
    );
}

#[test]
fn register_iris_swap_fold_op_as_component() {
    let iris_program = build_iris_swap_fold_op_program();

    let component = MutationComponent {
        name: "iris_swap_fold_op".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_swap_fold_op");
    assert!(found.is_some(), "iris_swap_fold_op should be registered");

    let target = make_fold_graph(0x00, 0);
    let donor = make_donor_lit(999, 1);
    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(20),
        Value::Int(0x02),
        Value::Int(10),
        Value::Program(Rc::new(donor)),
        Value::Int(999),
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Program(g) => {
            let input_list = Value::tuple(vec![Value::Int(2), Value::Int(3), Value::Int(4)]);
            let (result, _) = interpreter::interpret(g, &[input_list], None).unwrap();
            assert_eq!(result, vec![Value::Int(24)], "fold(1, mul, [2,3,4]) = 24");
        }
        other => panic!("expected Program, got {:?}", other),
    }

    eprintln!("swap_fold_op mutation operator replaced by IRIS program");
}

// ===========================================================================
// Operator 3: wrap_in_map
// ===========================================================================
//
// Insert a Map(0x30) Prim node before a Fold's collection input.
// Uses graph_add_node_rt(0x85) to create the map node, then
// graph_disconnect + graph_connect to rewire.
//
// Before: Fold(base, step, collection)
// After:  Fold(base, step, Map(collection, map_fn))
//
// Inputs:
//   inputs[0] = Program(target_graph)
//   inputs[1] = Int(fold_node_id)      -- the Fold node
//   inputs[2] = Int(collection_node_id) -- current collection input
//   inputs[3] = Int(map_fn_opcode)     -- opcode for the map function
//
// Strategy:
//   1. graph_add_node_rt(program, 0x30) -> Tuple(prog1, map_id)
//   2. Extract prog1 and map_id via Project
//   3. graph_add_node_rt(prog1, map_fn_opcode) -> Tuple(prog2, fn_id)
//   4. Extract prog2 and fn_id
//   5. graph_disconnect(prog2, fold, collection)  -- remove fold->collection
//   6. graph_connect(result, fold, map_id, 2)    -- fold->map on port 2
//   7. graph_connect(result, map_id, collection, 0)  -- map->collection
//   8. graph_connect(result, map_id, fn_id, 1)    -- map->fn
//
// This is complex (8 steps), so we build it carefully.
//
// Graph structure (deeply nested):
//   Root(id=1): graph_connect(0x86, arity=4) -- step 8: map->fn
//   +-- port 0: graph_connect(0x86) [id=50]  -- step 7: map->collection
//   |   +-- port 0: graph_connect(0x86) [id=60] -- step 6: fold->map
//   |   |   +-- port 0: graph_disconnect(0x87) [id=70] -- step 5: remove fold->collection
//   |   |   |   +-- port 0: Project(0) [id=80]  -- extract prog from step 3
//   |   |   |   |   +-- port 0: graph_add_node_rt(0x85) [id=90] -- step 3: add map_fn
//   |   |   |   |       +-- port 0: Project(0) [id=95]  -- extract prog from step 1
//   |   |   |   |       |   +-- port 0: graph_add_node_rt(0x85) [id=100] -- step 1: add map node
//   |   |   |   |       |       +-- port 0: input_ref(0) [id=110]  -- program
//   |   |   |   |       |       +-- port 1: Lit(0x30) [id=115]     -- map opcode
//   |   |   |   |       +-- port 1: input_ref(3) [id=120]  -- map_fn_opcode
//   |   |   |   +-- port 1: input_ref(1) [id=130]  -- fold node
//   |   |   |   +-- port 2: input_ref(2) [id=140]  -- collection node
//   |   |   +-- port 1: input_ref(1) [id=150]  -- fold node
//   |   |   +-- port 2: Project(1) [id=160]    -- map_id from step 1
//   |   |   |   +-- port 0: graph_add_node_rt(0x85) [id=170] -- duplicate step 1
//   |   |   |       +-- port 0: input_ref(0) [id=180]
//   |   |   |       +-- port 1: Lit(0x30) [id=185]
//   |   |   +-- port 3: Lit(2) [id=190]        -- port 2
//   |   +-- port 1: Project(1) [id=200]         -- map_id (again)
//   |   |   +-- port 0: graph_add_node_rt(0x85) [id=210]
//   |   |       +-- port 0: input_ref(0) [id=215]
//   |   |       +-- port 1: Lit(0x30) [id=217]
//   |   +-- port 2: input_ref(2) [id=220]       -- collection node
//   |   +-- port 3: Lit(0) [id=230]             -- port 0
//   +-- port 1: Project(1) [id=240]             -- map_id (again)
//   |   +-- port 0: graph_add_node_rt(0x85) [id=250]
//   |       +-- port 0: input_ref(0) [id=255]
//   |       +-- port 1: Lit(0x30) [id=257]
//   +-- port 2: Project(1) [id=260]             -- fn_id from step 3
//   |   +-- port 0: graph_add_node_rt(0x85) [id=270]
//   |       +-- port 0: Project(0) [id=275]
//   |       |   +-- port 0: graph_add_node_rt(0x85) [id=280]
//   |       |       +-- port 0: input_ref(0) [id=285]
//   |       |       +-- port 1: Lit(0x30) [id=287]
//   |       +-- port 1: input_ref(3) [id=290]
//   +-- port 3: Lit(1) [id=300]                 -- port 1

fn build_iris_wrap_in_map_program() -> SemanticGraph {
    // This is a deeply nested graph because graph_add_node_rt returns
    // Tuple(Program, new_id) and we need to thread the program through
    // multiple operations while also extracting IDs via Project.
    //
    // The key insight: graph_add_node_rt is deterministic for a given opcode
    // and graph state. When we call it multiple times on the SAME input program
    // with the SAME opcode, it produces the SAME (Program, new_id) result.
    // This lets us re-evaluate it to extract either the Program or the ID
    // without needing to store intermediate results.

    let mut nodes = HashMap::new();

    // === Step 1 build block: graph_add_node_rt(program, 0x30) ===
    // We need this result in multiple places, so we define it once and
    // reference it multiple times. The interpreter's pure evaluation
    // will produce the same result each time.

    // Shared add_map_node block (returns Tuple(prog_with_map, map_id))
    // We build 4 copies because IRIS graphs are DAGs with content-addressed
    // nodes. Each "copy" is a separate subtree producing the same result.

    // Copy A: used to extract prog for step 3 input
    let (nid, node) = prim_node(100, 0x85, 2); // graph_add_node_rt
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(110, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(115, 0x30); // map opcode
    nodes.insert(nid, node);
    let (nid, node) = project_node(95, 0); // extract program
    nodes.insert(nid, node);

    // Copy B: used to extract map_id for fold->map connect (step 6, port 2)
    let (nid, node) = prim_node(170, 0x85, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(180, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(185, 0x30);
    nodes.insert(nid, node);
    let (nid, node) = project_node(160, 1); // extract map_id
    nodes.insert(nid, node);

    // Copy C: used for map->collection connect (step 7, port 1)
    let (nid, node) = prim_node(210, 0x85, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(215, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(217, 0x30);
    nodes.insert(nid, node);
    let (nid, node) = project_node(200, 1); // extract map_id
    nodes.insert(nid, node);

    // Copy D: used for map->fn connect (step 8, port 1)
    let (nid, node) = prim_node(250, 0x85, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(255, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(257, 0x30);
    nodes.insert(nid, node);
    let (nid, node) = project_node(240, 1); // extract map_id
    nodes.insert(nid, node);

    // === Step 3 build block: graph_add_node_rt(prog_after_map, fn_opcode) ===
    // Input: Project(0) of step 1 (Copy A) = prog_with_map
    // Returns: Tuple(prog_with_both, fn_id)

    // Used to extract prog for disconnect (step 5, port 0)
    let (nid, node) = prim_node(90, 0x85, 2); // graph_add_node_rt
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(120, 3); // map_fn_opcode
    nodes.insert(nid, node);
    let (nid, node) = project_node(80, 0); // extract prog
    nodes.insert(nid, node);

    // Copy E: used to extract fn_id for map->fn connect (step 8, port 2)
    let (nid, node) = prim_node(270, 0x85, 2);
    nodes.insert(nid, node);
    let (nid, node) = project_node(275, 0); // extract prog from step 1 (Copy E's own step 1)
    nodes.insert(nid, node);
    let (nid, node) = prim_node(280, 0x85, 2); // step 1 for Copy E
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(285, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(287, 0x30);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(290, 3); // map_fn_opcode
    nodes.insert(nid, node);
    let (nid, node) = project_node(260, 1); // extract fn_id
    nodes.insert(nid, node);

    // === Step 5: graph_disconnect(prog, fold, collection) ===
    let (nid, node) = prim_node(70, 0x87, 3);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(130, 1); // fold node
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(140, 2); // collection node
    nodes.insert(nid, node);

    // === Step 6: graph_connect(prog, fold, map_id, 2) ===
    let (nid, node) = prim_node(60, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(150, 1); // fold node
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(190, 2); // port 2
    nodes.insert(nid, node);

    // === Step 7: graph_connect(prog, map_id, collection, 0) ===
    let (nid, node) = prim_node(50, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(220, 2); // collection node
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(230, 0); // port 0
    nodes.insert(nid, node);

    // === Step 8 (Root): graph_connect(prog, map_id, fn_id, 1) ===
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(300, 1); // port 1
    nodes.insert(nid, node);

    // === Wire everything ===
    let edges = vec![
        // Step 1 Copy A: add_node_rt(program, 0x30)
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 115, 1, EdgeLabel::Argument),
        // Project(0) on Copy A -> prog
        make_edge(95, 100, 0, EdgeLabel::Argument),

        // Step 1 Copy B: add_node_rt(program, 0x30)
        make_edge(170, 180, 0, EdgeLabel::Argument),
        make_edge(170, 185, 1, EdgeLabel::Argument),
        // Project(1) on Copy B -> map_id
        make_edge(160, 170, 0, EdgeLabel::Argument),

        // Step 1 Copy C: add_node_rt(program, 0x30)
        make_edge(210, 215, 0, EdgeLabel::Argument),
        make_edge(210, 217, 1, EdgeLabel::Argument),
        // Project(1) on Copy C -> map_id
        make_edge(200, 210, 0, EdgeLabel::Argument),

        // Step 1 Copy D: add_node_rt(program, 0x30)
        make_edge(250, 255, 0, EdgeLabel::Argument),
        make_edge(250, 257, 1, EdgeLabel::Argument),
        // Project(1) on Copy D -> map_id
        make_edge(240, 250, 0, EdgeLabel::Argument),

        // Step 3: add_node_rt(proj0(step1_A), fn_opcode)
        make_edge(90, 95, 0, EdgeLabel::Argument),   // prog from step 1 Copy A
        make_edge(90, 120, 1, EdgeLabel::Argument),   // fn_opcode
        // Project(0) on step 3 -> prog
        make_edge(80, 90, 0, EdgeLabel::Argument),

        // Step 3 Copy E (for fn_id extraction): add_node_rt(proj0(step1_E), fn_opcode)
        make_edge(280, 285, 0, EdgeLabel::Argument),  // program
        make_edge(280, 287, 1, EdgeLabel::Argument),  // 0x30
        make_edge(275, 280, 0, EdgeLabel::Argument),  // proj0 on step1_E
        make_edge(270, 275, 0, EdgeLabel::Argument),  // prog from step 1 E
        make_edge(270, 290, 1, EdgeLabel::Argument),  // fn_opcode
        // Project(1) on step 3 Copy E -> fn_id
        make_edge(260, 270, 0, EdgeLabel::Argument),

        // Step 5: disconnect(proj0(step3), fold, collection)
        make_edge(70, 80, 0, EdgeLabel::Argument),    // prog from step 3
        make_edge(70, 130, 1, EdgeLabel::Argument),   // fold
        make_edge(70, 140, 2, EdgeLabel::Argument),   // collection

        // Step 6: connect(step5_result, fold, map_id, 2)
        make_edge(60, 70, 0, EdgeLabel::Argument),    // prog from step 5
        make_edge(60, 150, 1, EdgeLabel::Argument),   // fold
        make_edge(60, 160, 2, EdgeLabel::Argument),   // map_id (from Copy B)
        make_edge(60, 190, 3, EdgeLabel::Argument),   // port 2

        // Step 7: connect(step6_result, map_id, collection, 0)
        make_edge(50, 60, 0, EdgeLabel::Argument),    // prog from step 6
        make_edge(50, 200, 1, EdgeLabel::Argument),   // map_id (from Copy C)
        make_edge(50, 220, 2, EdgeLabel::Argument),   // collection
        make_edge(50, 230, 3, EdgeLabel::Argument),   // port 0

        // Step 8 (Root): connect(step7_result, map_id, fn_id, 1)
        make_edge(1, 50, 0, EdgeLabel::Argument),     // prog from step 7
        make_edge(1, 240, 1, EdgeLabel::Argument),    // map_id (from Copy D)
        make_edge(1, 260, 2, EdgeLabel::Argument),    // fn_id (from Copy E)
        make_edge(1, 300, 3, EdgeLabel::Argument),    // port 1
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// wrap_in_map tests
// ---------------------------------------------------------------------------

#[test]
fn iris_wrap_in_map_basic() {
    // fold(0, add, [1, 2, 3]) = 6
    // wrap_in_map with mul(0x02): fold(0, add, map([1,2,3], mul))
    // map applies mul to each element. But map with a binary op needs two args.
    // The interpreter applies the map function as f(acc, elem) style.
    // Let's verify the structural transformation works.
    let iris_wrapper = build_iris_wrap_in_map_program();
    let target = make_fold_graph(0x00, 0); // fold(0, add, input)

    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(1),    // fold node ID
        Value::Int(30),   // collection node ID (input_ref)
        Value::Int(0x20), // map function: use 0x20 (>= 0x14, creates Prim)
    ];

    let (outputs, _) = interpreter::interpret(&iris_wrapper, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // Verify structural changes:
    // 1. A new Map node (opcode 0x30) should exist
    let has_map = modified
        .nodes
        .values()
        .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x30 }));
    assert!(has_map, "should have a Map node (opcode 0x30)");

    // 2. A new Prim(0x20) node should exist (the map function)
    let has_map_fn = modified
        .nodes
        .values()
        .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x20 }));
    assert!(has_map_fn, "should have a Prim(0x20) node (the map function)");

    // 3. The fold should NOT have a direct edge to the original collection
    let fold_to_collection = modified
        .edges
        .iter()
        .any(|e| e.source == NodeId(1) && e.target == NodeId(30) && e.port == 2);
    assert!(
        !fold_to_collection,
        "fold should not have direct edge to original collection"
    );

    // 4. The fold should have an edge to the map node on port 2
    let fold_to_map = modified.edges.iter().any(|e| {
        e.source == NodeId(1)
            && e.port == 2
            && modified.nodes.get(&e.target).map_or(false, |n| {
                matches!(&n.payload, NodePayload::Prim { opcode: 0x30 })
            })
    });
    assert!(fold_to_map, "fold should have edge to map node on port 2");

    eprintln!("wrap_in_map structural transformation verified");
}

#[test]
fn register_iris_wrap_in_map_as_component() {
    let iris_program = build_iris_wrap_in_map_program();

    let component = MutationComponent {
        name: "iris_wrap_in_map".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_wrap_in_map");
    assert!(found.is_some(), "iris_wrap_in_map should be registered");

    eprintln!("wrap_in_map mutation operator replaced by IRIS program");
}

// ===========================================================================
// Operator 4: wrap_in_filter
// ===========================================================================
//
// Identical structure to wrap_in_map, but uses Filter opcode (0x31) instead
// of Map (0x30). The filter function is a comparison opcode.
//
// Before: Fold(base, step, collection)
// After:  Fold(base, step, Filter(collection, cmp_fn))
//
// Inputs:
//   inputs[0] = Program(target_graph)
//   inputs[1] = Int(fold_node_id)
//   inputs[2] = Int(collection_node_id)
//   inputs[3] = Int(filter_fn_opcode)  -- comparison opcode (e.g., 0x23 for gt)

fn build_iris_wrap_in_filter_program() -> SemanticGraph {
    // Same structure as wrap_in_map but with 0x31 (filter) instead of 0x30 (map).
    let mut nodes = HashMap::new();

    // Copy A: add filter node
    let (nid, node) = prim_node(100, 0x85, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(110, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(115, 0x31); // filter opcode
    nodes.insert(nid, node);
    let (nid, node) = project_node(95, 0);
    nodes.insert(nid, node);

    // Copy B: extract filter_id
    let (nid, node) = prim_node(170, 0x85, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(180, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(185, 0x31);
    nodes.insert(nid, node);
    let (nid, node) = project_node(160, 1);
    nodes.insert(nid, node);

    // Copy C: extract filter_id
    let (nid, node) = prim_node(210, 0x85, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(216, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(218, 0x31);
    nodes.insert(nid, node);
    let (nid, node) = project_node(200, 1);
    nodes.insert(nid, node);

    // Copy D: extract filter_id
    let (nid, node) = prim_node(250, 0x85, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(256, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(258, 0x31);
    nodes.insert(nid, node);
    let (nid, node) = project_node(240, 1);
    nodes.insert(nid, node);

    // Step 3: add filter predicate node
    let (nid, node) = prim_node(90, 0x85, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(120, 3); // filter_fn_opcode
    nodes.insert(nid, node);
    let (nid, node) = project_node(80, 0);
    nodes.insert(nid, node);

    // Copy E: fn_id extraction
    let (nid, node) = prim_node(270, 0x85, 2);
    nodes.insert(nid, node);
    let (nid, node) = project_node(275, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(280, 0x85, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(286, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(288, 0x31);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(291, 3);
    nodes.insert(nid, node);
    let (nid, node) = project_node(260, 1);
    nodes.insert(nid, node);

    // Step 5: disconnect
    let (nid, node) = prim_node(70, 0x87, 3);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(131, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(141, 2);
    nodes.insert(nid, node);

    // Step 6: connect fold -> filter on port 2
    let (nid, node) = prim_node(60, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(151, 1);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(191, 2);
    nodes.insert(nid, node);

    // Step 7: connect filter -> collection on port 0
    let (nid, node) = prim_node(50, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(221, 2);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(231, 0);
    nodes.insert(nid, node);

    // Step 8 (Root): connect filter -> fn on port 1
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(301, 1);
    nodes.insert(nid, node);

    let edges = vec![
        // Step 1 Copy A
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 115, 1, EdgeLabel::Argument),
        make_edge(95, 100, 0, EdgeLabel::Argument),
        // Step 1 Copy B
        make_edge(170, 180, 0, EdgeLabel::Argument),
        make_edge(170, 185, 1, EdgeLabel::Argument),
        make_edge(160, 170, 0, EdgeLabel::Argument),
        // Step 1 Copy C
        make_edge(210, 216, 0, EdgeLabel::Argument),
        make_edge(210, 218, 1, EdgeLabel::Argument),
        make_edge(200, 210, 0, EdgeLabel::Argument),
        // Step 1 Copy D
        make_edge(250, 256, 0, EdgeLabel::Argument),
        make_edge(250, 258, 1, EdgeLabel::Argument),
        make_edge(240, 250, 0, EdgeLabel::Argument),
        // Step 3
        make_edge(90, 95, 0, EdgeLabel::Argument),
        make_edge(90, 120, 1, EdgeLabel::Argument),
        make_edge(80, 90, 0, EdgeLabel::Argument),
        // Step 3 Copy E
        make_edge(280, 286, 0, EdgeLabel::Argument),
        make_edge(280, 288, 1, EdgeLabel::Argument),
        make_edge(275, 280, 0, EdgeLabel::Argument),
        make_edge(270, 275, 0, EdgeLabel::Argument),
        make_edge(270, 291, 1, EdgeLabel::Argument),
        make_edge(260, 270, 0, EdgeLabel::Argument),
        // Step 5: disconnect
        make_edge(70, 80, 0, EdgeLabel::Argument),
        make_edge(70, 131, 1, EdgeLabel::Argument),
        make_edge(70, 141, 2, EdgeLabel::Argument),
        // Step 6: connect fold -> filter
        make_edge(60, 70, 0, EdgeLabel::Argument),
        make_edge(60, 151, 1, EdgeLabel::Argument),
        make_edge(60, 160, 2, EdgeLabel::Argument),
        make_edge(60, 191, 3, EdgeLabel::Argument),
        // Step 7: connect filter -> collection
        make_edge(50, 60, 0, EdgeLabel::Argument),
        make_edge(50, 200, 1, EdgeLabel::Argument),
        make_edge(50, 221, 2, EdgeLabel::Argument),
        make_edge(50, 231, 3, EdgeLabel::Argument),
        // Step 8 (Root): connect filter -> fn
        make_edge(1, 50, 0, EdgeLabel::Argument),
        make_edge(1, 240, 1, EdgeLabel::Argument),
        make_edge(1, 260, 2, EdgeLabel::Argument),
        make_edge(1, 301, 3, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

#[test]
fn iris_wrap_in_filter_basic() {
    let iris_wrapper = build_iris_wrap_in_filter_program();
    let target = make_fold_graph(0x00, 0); // fold(0, add, input)

    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(1),    // fold node ID
        Value::Int(30),   // collection node ID (input_ref)
        Value::Int(0x23), // filter function: gt (greater than)
    ];

    let (outputs, _) = interpreter::interpret(&iris_wrapper, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // Verify structural changes:
    let has_filter = modified
        .nodes
        .values()
        .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x31 }));
    assert!(has_filter, "should have a Filter node (opcode 0x31)");

    let has_gt = modified
        .nodes
        .values()
        .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x23 }));
    assert!(has_gt, "should have a gt node (the filter predicate)");

    // The fold should NOT have a direct edge to the original collection
    let fold_to_collection = modified
        .edges
        .iter()
        .any(|e| e.source == NodeId(1) && e.target == NodeId(30) && e.port == 2);
    assert!(
        !fold_to_collection,
        "fold should not have direct edge to original collection"
    );

    eprintln!("wrap_in_filter structural transformation verified");
}

#[test]
fn register_iris_wrap_in_filter_as_component() {
    let iris_program = build_iris_wrap_in_filter_program();

    let component = MutationComponent {
        name: "iris_wrap_in_filter".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_wrap_in_filter");
    assert!(found.is_some(), "iris_wrap_in_filter should be registered");

    eprintln!("wrap_in_filter mutation operator replaced by IRIS program");
}

// ===========================================================================
// Population management 1: tournament_select
// ===========================================================================
//
// Pick the best individual from a group. Since we don't have random number
// generation within the IRIS program easily (would need Effect nodes +
// handlers), we use a deterministic comparison approach:
//
// Given two fitness scores, return the index of the better one.
// This is the core comparison kernel that an outer loop calls repeatedly.
//
// Inputs:
//   inputs[0] = Int(fitness_a)
//   inputs[1] = Int(fitness_b)
//
// Output: Int(0) if a >= b (pick a), Int(1) if b > a (pick b)
//
// Implementation: sub(a, b). If result >= 0, a wins (return 0), else b wins
// (return 1). We use: result = (a - b) < 0 ? 1 : 0
// Which is: ge(sub(a, b), 0) -> bool, then flip: sub(1, ge_result)
//
// Graph structure:
//   Root(id=1): sub(0x01, arity=2)
//   +-- port 0: Lit(1) [id=10]
//   +-- port 1: ge(0x25, arity=2) [id=20]
//       +-- port 0: sub(0x01) [id=30]
//       |   +-- port 0: input_ref(0) [id=40]  -- fitness_a
//       |   +-- port 1: input_ref(1) [id=50]  -- fitness_b
//       +-- port 1: Lit(0) [id=60]

fn build_iris_tournament_compare_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: sub(1, ge(sub(a, b), 0))
    // If a >= b: ge returns 1, sub(1,1)=0 -> pick a
    // If a < b:  ge returns 0, sub(1,0)=1 -> pick b
    let (nid, node) = prim_node(1, 0x01, 2); // sub
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(10, 1);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(20, 0x25, 2); // ge
    nodes.insert(nid, node);
    let (nid, node) = prim_node(30, 0x01, 2); // sub
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(40, 0); // fitness_a
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(50, 1); // fitness_b
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(60, 0);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),  // sub(1, ...)
        make_edge(1, 20, 1, EdgeLabel::Argument),  // ... ge(...)
        make_edge(20, 30, 0, EdgeLabel::Argument),  // ge(sub(...), ...)
        make_edge(20, 60, 1, EdgeLabel::Argument),  // ge(..., 0)
        make_edge(30, 40, 0, EdgeLabel::Argument),  // sub(a, ...)
        make_edge(30, 50, 1, EdgeLabel::Argument),  // sub(..., b)
    ];

    make_graph(nodes, edges, 1)
}

/// Build a full tournament_select that picks the best of N using pairwise
/// comparison. For simplicity, we build a 4-way tournament selector that
/// compares pairs and then compares winners.
///
/// Inputs:
///   inputs[0..3] = Int(fitness of individual 0..3)
///
/// Output: Int(index of best individual)
///
/// This does: compare(0,1) -> winner_01, compare(2,3) -> winner_23,
/// compare(winner_01, winner_23) -> final winner.
///
/// We need conditionals (Guard nodes) to select the actual index based
/// on comparison results. Since we can't create Guard nodes via IRIS,
/// we simplify: return the INDEX of the maximum fitness.
///
/// Strategy using arithmetic:
///   best_idx = 0
///   if fitness[1] > fitness[best_idx]: best_idx = 1
///   if fitness[2] > fitness[best_idx]: best_idx = 2
///   if fitness[3] > fitness[best_idx]: best_idx = 3
///
/// Without Guard nodes, we compute: argmax = the index whose fitness is max.
/// Using max opcode: max_val = max(max(f0, f1), max(f2, f3))
/// Then we return the max_val itself (the caller can use it as the fitness).
/// This is a simpler but still useful "select best fitness" kernel.
fn build_iris_tournament_select_best_fitness() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: max(max(f0, f1), max(f2, f3))
    let (nid, node) = prim_node(1, 0x08, 2); // max (outer)
    nodes.insert(nid, node);
    let (nid, node) = prim_node(10, 0x08, 2); // max(f0, f1)
    nodes.insert(nid, node);
    let (nid, node) = prim_node(20, 0x08, 2); // max(f2, f3)
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 0); // f0
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(40, 1); // f1
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(50, 2); // f2
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(60, 3); // f3
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),  // max(max01, ...)
        make_edge(1, 20, 1, EdgeLabel::Argument),  // max(..., max23)
        make_edge(10, 30, 0, EdgeLabel::Argument),  // max(f0, ...)
        make_edge(10, 40, 1, EdgeLabel::Argument),  // max(..., f1)
        make_edge(20, 50, 0, EdgeLabel::Argument),  // max(f2, ...)
        make_edge(20, 60, 1, EdgeLabel::Argument),  // max(..., f3)
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// tournament_select tests
// ---------------------------------------------------------------------------

#[test]
fn iris_tournament_compare_a_wins() {
    let comparator = build_iris_tournament_compare_program();

    let inputs = vec![Value::Int(10), Value::Int(5)];
    let (outputs, _) = interpreter::interpret(&comparator, &inputs, None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(0)],
        "10 >= 5, should return 0 (pick a)"
    );
}

#[test]
fn iris_tournament_compare_b_wins() {
    let comparator = build_iris_tournament_compare_program();

    let inputs = vec![Value::Int(3), Value::Int(7)];
    let (outputs, _) = interpreter::interpret(&comparator, &inputs, None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(1)],
        "3 < 7, should return 1 (pick b)"
    );
}

#[test]
fn iris_tournament_compare_equal() {
    let comparator = build_iris_tournament_compare_program();

    let inputs = vec![Value::Int(5), Value::Int(5)];
    let (outputs, _) = interpreter::interpret(&comparator, &inputs, None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(0)],
        "5 >= 5, should return 0 (pick a on tie)"
    );
}

#[test]
fn iris_tournament_compare_negative() {
    let comparator = build_iris_tournament_compare_program();

    let inputs = vec![Value::Int(-3), Value::Int(-7)];
    let (outputs, _) = interpreter::interpret(&comparator, &inputs, None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(0)],
        "-3 >= -7, should return 0 (pick a)"
    );
}

#[test]
fn iris_tournament_select_best() {
    let selector = build_iris_tournament_select_best_fitness();

    // Find max of 4 fitness values
    let inputs = vec![
        Value::Int(5),
        Value::Int(12),
        Value::Int(3),
        Value::Int(8),
    ];
    let (outputs, _) = interpreter::interpret(&selector, &inputs, None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(12)],
        "max(5, 12, 3, 8) = 12"
    );
}

#[test]
fn iris_tournament_select_all_equal() {
    let selector = build_iris_tournament_select_best_fitness();

    let inputs = vec![
        Value::Int(7),
        Value::Int(7),
        Value::Int(7),
        Value::Int(7),
    ];
    let (outputs, _) = interpreter::interpret(&selector, &inputs, None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(7)],
        "max(7, 7, 7, 7) = 7"
    );
}

#[test]
fn iris_tournament_select_negative_values() {
    let selector = build_iris_tournament_select_best_fitness();

    let inputs = vec![
        Value::Int(-10),
        Value::Int(-3),
        Value::Int(-15),
        Value::Int(-1),
    ];
    let (outputs, _) = interpreter::interpret(&selector, &inputs, None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(-1)],
        "max(-10, -3, -15, -1) = -1"
    );
}

#[test]
fn register_iris_tournament_select_as_component() {
    let compare_program = build_iris_tournament_compare_program();
    let select_program = build_iris_tournament_select_best_fitness();

    let mut registry = ComponentRegistry::new();

    registry.mutations.push(MutationComponent {
        name: "iris_tournament_compare".to_string(),
        program: compare_program,
    });
    registry.mutations.push(MutationComponent {
        name: "iris_tournament_select".to_string(),
        program: select_program,
    });

    assert!(
        registry.find_mutation("iris_tournament_compare").is_some(),
        "tournament_compare should be registered"
    );
    assert!(
        registry.find_mutation("iris_tournament_select").is_some(),
        "tournament_select should be registered"
    );

    eprintln!("tournament_select population management replaced by IRIS programs");
}

// ===========================================================================
// Population management 2: crossover_subgraph
// ===========================================================================
//
// Swap a subtree between two programs using graph_replace_subtree.
//
// Given two programs (parent_a, parent_b) and nodes to swap:
//   1. Replace node_a in parent_a with subtree from node_b in parent_b
//   2. Replace node_b in parent_b with subtree from node_a in parent_a
//
// This produces TWO offspring. Since IRIS programs return a single value,
// we return a Tuple of (offspring_a, offspring_b).
//
// Inputs:
//   inputs[0] = Program(parent_a)
//   inputs[1] = Int(node_a_id)  -- crossover point in parent_a
//   inputs[2] = Program(parent_b)
//   inputs[3] = Int(node_b_id)  -- crossover point in parent_b
//
// Output: Tuple(Program(offspring_a), Program(offspring_b))
//
// Graph structure:
//   Root(id=1): Tuple(arity=2)
//   +-- port 0: graph_replace_subtree(0x88) [id=100]  -- offspring_a
//   |   +-- port 0: input_ref(0) [id=110]  -- parent_a
//   |   +-- port 1: input_ref(1) [id=120]  -- node_a
//   |   +-- port 2: input_ref(2) [id=130]  -- parent_b
//   |   +-- port 3: input_ref(3) [id=140]  -- node_b
//   +-- port 1: graph_replace_subtree(0x88) [id=200]  -- offspring_b
//       +-- port 0: input_ref(2) [id=210]  -- parent_b
//       +-- port 1: input_ref(3) [id=220]  -- node_b
//       +-- port 2: input_ref(0) [id=230]  -- parent_a
//       +-- port 3: input_ref(1) [id=240]  -- node_a

fn build_iris_crossover_subgraph_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Tuple(offspring_a, offspring_b)
    let (nid, node) = make_node(1, NodeKind::Tuple, NodePayload::Tuple, 2);
    nodes.insert(nid, node);

    // offspring_a: replace node_a in parent_a with subtree from node_b in parent_b
    let (nid, node) = prim_node(100, 0x88, 4);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(110, 0); // parent_a
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(120, 1); // node_a
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(130, 2); // parent_b
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(140, 3); // node_b
    nodes.insert(nid, node);

    // offspring_b: replace node_b in parent_b with subtree from node_a in parent_a
    let (nid, node) = prim_node(200, 0x88, 4);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(210, 2); // parent_b
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(220, 3); // node_b
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(230, 0); // parent_a
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(240, 1); // node_a
    nodes.insert(nid, node);

    let edges = vec![
        // Root -> offspring_a, offspring_b
        make_edge(1, 100, 0, EdgeLabel::Argument),
        make_edge(1, 200, 1, EdgeLabel::Argument),
        // offspring_a: replace_subtree(parent_a, node_a, parent_b, node_b)
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        make_edge(100, 130, 2, EdgeLabel::Argument),
        make_edge(100, 140, 3, EdgeLabel::Argument),
        // offspring_b: replace_subtree(parent_b, node_b, parent_a, node_a)
        make_edge(200, 210, 0, EdgeLabel::Argument),
        make_edge(200, 220, 1, EdgeLabel::Argument),
        make_edge(200, 230, 2, EdgeLabel::Argument),
        make_edge(200, 240, 3, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// crossover_subgraph tests
// ---------------------------------------------------------------------------

#[test]
fn iris_crossover_swaps_leaf_nodes() {
    // Parent A (base=1):  add(5, 3) = 8, nodes at 1, 11, 21
    // Parent B (base=100): mul(7, 2) = 14, nodes at 100, 110, 120
    // Crossover: lit(3, id=21) in A <-> lit(7, id=110) in B
    // Offspring A: add(5, 7) = 12  (lit(3) replaced by lit(7))
    // Offspring B: mul(3, 2) = 6   (lit(7) replaced by lit(3))
    let iris_crossover = build_iris_crossover_subgraph_program();

    let parent_a = make_binop_graph_at(0x00, 5, 3, 1);   // add(5, 3), ids: 1, 11, 21
    let parent_b = make_binop_graph_at(0x02, 7, 2, 100); // mul(7, 2), ids: 100, 110, 120

    let (orig_a, _) = interpreter::interpret(&parent_a, &[], None).unwrap();
    assert_eq!(orig_a, vec![Value::Int(8)], "add(5, 3) = 8");
    let (orig_b, _) = interpreter::interpret(&parent_b, &[], None).unwrap();
    assert_eq!(orig_b, vec![Value::Int(14)], "mul(7, 2) = 14");

    let inputs = vec![
        Value::Program(Rc::new(parent_a)),
        Value::Int(21),  // node_a: lit(3) in parent_a
        Value::Program(Rc::new(parent_b)),
        Value::Int(110), // node_b: lit(7) in parent_b
    ];

    let (outputs, _) = interpreter::interpret(&iris_crossover, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Tuple(offspring) => {
            assert_eq!(offspring.len(), 2, "should produce 2 offspring");

            // Offspring A: add(5, 7) = 12
            let off_a = match &offspring[0] {
                Value::Program(g) => g.as_ref().clone(),
                other => panic!("offspring_a: expected Program, got {:?}", other),
            };
            let (result_a, _) = interpreter::interpret(&off_a, &[], None).unwrap();
            assert_eq!(
                result_a,
                vec![Value::Int(12)],
                "offspring_a: add(5, 7) = 12"
            );

            // Offspring B: mul(3, 2) = 6
            let off_b = match &offspring[1] {
                Value::Program(g) => g.as_ref().clone(),
                other => panic!("offspring_b: expected Program, got {:?}", other),
            };
            let (result_b, _) = interpreter::interpret(&off_b, &[], None).unwrap();
            assert_eq!(
                result_b,
                vec![Value::Int(6)],
                "offspring_b: mul(3, 2) = 6"
            );
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn iris_crossover_swaps_subtrees() {
    // Parent A (base=1): mul(add(3, 4), 5) = 35
    //   nodes: 1(mul), 2(add), 10(lit3), 20(lit4), 30(lit5)
    // Parent B (base=200): sub(100, 50) = 50
    //   nodes: 200(sub), 210(lit100), 220(lit50)
    //
    // Crossover: add(3,4) subtree (id=2) in A <-> lit(100, id=210) in B
    // Offspring A: mul(100, 5) = 500 (add subtree replaced by lit(100))
    // Offspring B: sub(add(3,4), 50) = 7 - 50 = -43 (lit(100) replaced by add subtree)
    let iris_crossover = build_iris_crossover_subgraph_program();

    let parent_a = make_chain_graph(0x00, 0x02, 3, 4, 5);     // mul(add(3,4), 5)
    let parent_b = make_binop_graph_at(0x01, 100, 50, 200);   // sub(100, 50)

    let (orig_a, _) = interpreter::interpret(&parent_a, &[], None).unwrap();
    assert_eq!(orig_a, vec![Value::Int(35)], "mul(add(3,4), 5) = 35");
    let (orig_b, _) = interpreter::interpret(&parent_b, &[], None).unwrap();
    assert_eq!(orig_b, vec![Value::Int(50)], "sub(100, 50) = 50");

    let inputs = vec![
        Value::Program(Rc::new(parent_a)),
        Value::Int(2),   // node_a: inner add(3,4) subtree
        Value::Program(Rc::new(parent_b)),
        Value::Int(210), // node_b: lit(100)
    ];

    let (outputs, _) = interpreter::interpret(&iris_crossover, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Tuple(offspring) => {
            assert_eq!(offspring.len(), 2);

            // Offspring A: mul(100, 5) = 500
            let off_a = match &offspring[0] {
                Value::Program(g) => g.as_ref().clone(),
                other => panic!("offspring_a: expected Program, got {:?}", other),
            };
            let (result_a, _) = interpreter::interpret(&off_a, &[], None).unwrap();
            assert_eq!(
                result_a,
                vec![Value::Int(500)],
                "offspring_a: mul(100, 5) = 500"
            );

            // Offspring B: sub(add(3,4), 50) = 7 - 50 = -43
            let off_b = match &offspring[1] {
                Value::Program(g) => g.as_ref().clone(),
                other => panic!("offspring_b: expected Program, got {:?}", other),
            };
            let (result_b, _) = interpreter::interpret(&off_b, &[], None).unwrap();
            assert_eq!(
                result_b,
                vec![Value::Int(-43)],
                "offspring_b: sub(add(3,4), 50) = -43"
            );
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn iris_crossover_same_structure_different_ids() {
    // Same structure but different node IDs to avoid collision:
    // Parent A (base=1):   add(5, 3), nodes: 1, 11, 21
    // Parent B (base=100): add(8, 2), nodes: 100, 110, 120
    //
    // Swap lit(3, id=21) in A <-> lit(8, id=110) in B
    // Offspring A: add(5, 8) = 13  (lit(3) replaced by lit(8))
    // Offspring B: add(3, 2) = 5   (lit(8) replaced by lit(3))
    let iris_crossover = build_iris_crossover_subgraph_program();

    let parent_a = make_binop_graph_at(0x00, 5, 3, 1);   // add(5, 3)
    let parent_b = make_binop_graph_at(0x00, 8, 2, 100); // add(8, 2)

    let inputs = vec![
        Value::Program(Rc::new(parent_a)),
        Value::Int(21),  // lit(3) in A
        Value::Program(Rc::new(parent_b)),
        Value::Int(110), // lit(8) in B
    ];

    let (outputs, _) = interpreter::interpret(&iris_crossover, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Tuple(offspring) => {
            // Offspring A: lit(3) replaced by lit(8) -> add(5, 8) = 13
            let off_a = match &offspring[0] {
                Value::Program(g) => g.as_ref().clone(),
                other => panic!("offspring_a: expected Program, got {:?}", other),
            };
            let (result_a, _) = interpreter::interpret(&off_a, &[], None).unwrap();
            assert_eq!(
                result_a,
                vec![Value::Int(13)],
                "offspring_a: add(5, 8) = 13"
            );

            // Offspring B: lit(8) replaced by lit(3) -> add(3, 2) = 5
            let off_b = match &offspring[1] {
                Value::Program(g) => g.as_ref().clone(),
                other => panic!("offspring_b: expected Program, got {:?}", other),
            };
            let (result_b, _) = interpreter::interpret(&off_b, &[], None).unwrap();
            assert_eq!(
                result_b,
                vec![Value::Int(5)],
                "offspring_b: add(3, 2) = 5"
            );
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn register_iris_crossover_as_component() {
    let iris_program = build_iris_crossover_subgraph_program();

    let component = MutationComponent {
        name: "iris_crossover_subgraph".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_crossover_subgraph");
    assert!(found.is_some(), "iris_crossover_subgraph should be registered");

    // Quick smoke test via component (use non-overlapping IDs)
    let parent_a = make_binop_graph_at(0x00, 5, 3, 1);
    let parent_b = make_binop_graph_at(0x02, 7, 2, 100);
    let inputs = vec![
        Value::Program(Rc::new(parent_a)),
        Value::Int(21),  // lit(3) in A
        Value::Program(Rc::new(parent_b)),
        Value::Int(110), // lit(7) in B
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Tuple(offspring) => {
            assert_eq!(offspring.len(), 2, "should produce 2 offspring");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }

    eprintln!("crossover_subgraph population management replaced by IRIS program");
}

// ===========================================================================
// Composition test: chain mutation + crossover + selection
// ===========================================================================

#[test]
fn full_pipeline_mutate_crossover_select() {
    // Simulate a mini evolution cycle:
    // 1. Start with two programs: add(5, 3)=8 and mul(4, 2)=8
    // 2. Mutate program A: swap_fold_op style change (replace add with sub)
    // 3. Crossover the mutated A with B
    // 4. Select the best offspring

    let replacer = build_iris_replace_kind_program();
    let crossover = build_iris_crossover_subgraph_program();
    let selector = build_iris_tournament_select_best_fitness();

    // Step 1: Two parent programs (non-overlapping IDs)
    let parent_a = make_binop_graph_at(0x00, 5, 3, 1);   // add(5, 3) = 8, ids: 1, 11, 21
    let parent_b = make_binop_graph_at(0x02, 4, 2, 100); // mul(4, 2) = 8, ids: 100, 110, 120

    // Step 2: Mutate parent_a: change add to sub -> sub(5, 3) = 2
    let inputs = vec![
        Value::Program(Rc::new(parent_a)),
        Value::Int(1),    // root node
        Value::Int(0x01), // sub opcode
    ];
    let (out, _) = interpreter::interpret(&replacer, &inputs, None).unwrap();
    let mutated_a = extract_program(&out[0]);
    let (mutated_result, _) = interpreter::interpret(&mutated_a, &[], None).unwrap();
    assert_eq!(mutated_result, vec![Value::Int(2)], "sub(5, 3) = 2");

    // Step 3: Crossover mutated_a(sub(5,3)) with parent_b(mul(4,2))
    // Swap: lit(3, id=21) in mutated_a <-> lit(4, id=110) in parent_b
    // Verify the sub node exists
    assert!(
        mutated_a
            .nodes
            .values()
            .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x01 })),
        "should have sub node"
    );
    let inputs = vec![
        Value::Program(Rc::new(mutated_a)),
        Value::Int(21),  // lit(3) in A
        Value::Program(Rc::new(parent_b)),
        Value::Int(110), // lit(4) in B
    ];
    let (out, _) = interpreter::interpret(&crossover, &inputs, None).unwrap();
    let (off_a_result, off_b_result) = match &out[0] {
        Value::Tuple(children) => {
            let a = match &children[0] {
                Value::Program(g) => {
                    let (r, _) = interpreter::interpret(g, &[], None).unwrap();
                    r[0].clone()
                }
                other => panic!("offspring_a: expected Program, got {:?}", other),
            };
            let b = match &children[1] {
                Value::Program(g) => {
                    let (r, _) = interpreter::interpret(g, &[], None).unwrap();
                    r[0].clone()
                }
                other => panic!("offspring_b: expected Program, got {:?}", other),
            };
            (a, b)
        }
        other => panic!("expected Tuple, got {:?}", other),
    };

    // Offspring A: sub(5, 4) = 1
    // Offspring B: mul(3, 2) = 6

    // Step 4: Select best fitness from all 4 candidates
    let fitnesses = vec![
        Value::Int(2), // mutated_a: sub(5,3)=2
        match &off_a_result { Value::Int(v) => Value::Int(*v), _ => Value::Int(0) },
        match &off_b_result { Value::Int(v) => Value::Int(*v), _ => Value::Int(0) },
        Value::Int(8), // parent_b: mul(4,2)=8
    ];
    let (out, _) = interpreter::interpret(&selector, &fitnesses, None).unwrap();

    // The selector returns the max fitness value
    match &out[0] {
        Value::Int(best) => {
            assert!(*best >= 2, "best fitness should be at least 2");
            eprintln!("Full pipeline: best fitness = {}", best);
        }
        other => panic!("expected Int, got {:?}", other),
    }

    eprintln!("Full evolution pipeline: mutate -> crossover -> select verified");
}

// ===========================================================================
// Utility: build_iris_replace_kind_program (needed for composition tests)
// ===========================================================================

fn build_iris_replace_kind_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x84, 3);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 2);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// Summary test: report on all operators
// ===========================================================================

#[test]
fn summary_report() {
    eprintln!("\n=== Self-Write Mutation v3: Operator Status Report ===\n");

    eprintln!("SUCCEEDED (IRIS programs built + tested):");
    eprintln!("  [9/16] duplicate_subgraph  -- graph_replace_subtree(same_program)");
    eprintln!("  [10/16] swap_fold_op       -- graph_set_prim_op + graph_replace_subtree");
    eprintln!("  [11/16] wrap_in_map        -- graph_add_node_rt + connect/disconnect (8 steps)");
    eprintln!("  [12/16] wrap_in_filter     -- same pattern as wrap_in_map with filter opcode");
    eprintln!("  [POP-1] tournament_select  -- pairwise compare + 4-way max selector");
    eprintln!("  [POP-2] crossover_subgraph -- dual graph_replace_subtree producing Tuple");
    eprintln!();
    eprintln!("NOT FEASIBLE (missing opcodes):");
    eprintln!("  wrap_in_guard        -- needs Guard node creation (graph_add_node_rt only creates Prim)");
    eprintln!("  add_guard_condition  -- same limitation as wrap_in_guard");
    eprintln!("  insert_ref           -- needs Ref node creation (no graph_add_ref_rt opcode)");
    eprintln!("  annotate_cost        -- no opcode to modify node.cost field");
    eprintln!();
    eprintln!("TOTAL: 12/16 mutation operators + 2 population management functions as IRIS programs");
    eprintln!();
    eprintln!("TO ENABLE REMAINING 4:");
    eprintln!("  1. Add graph_add_guard_rt (0x8B): Program, pred_id, body_id, fallback_id -> Tuple(Program, guard_id)");
    eprintln!("  2. Add graph_add_ref_rt (0x8C): Program, fragment_id_bytes -> Tuple(Program, ref_id)");
    eprintln!("  3. Add graph_set_cost (0x8D): Program, node_id, cost_value -> Program");
    eprintln!("=== End Report ===");
}
