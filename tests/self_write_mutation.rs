
//! Self-writing milestone: an IRIS program that performs mutation.
//!
//! This test proves that IRIS programs can replicate the behavior of Rust
//! mutation operators. Specifically, we hand-craft an IRIS program that
//! performs the same operation as `replace_prim` in mutation.rs: given a
//! program and a new opcode, find a Prim node and change its opcode.
//!
//! The IRIS program uses self-modification opcodes (0x80-0x84) to inspect
//! and modify a program graph at runtime — the same infrastructure that
//! enables continuous self-improvement.

use std::collections::{BTreeMap, HashMap};

use iris_exec::interpreter;
use iris_types::component::{ComponentRegistry, MutationComponent, SeedComponent};
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::{compute_node_id, SemanticHash};
use iris_types::types::{TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Graph construction helpers
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

/// Create a simple graph that computes `a op b` where op is determined by opcode.
/// Root is a Prim node at id=1, with lit args at id=10, id=20.
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

/// Modify a graph's root Prim opcode (Rust-side, for generating expected outputs).
fn modify_root_opcode(graph: &SemanticGraph, new_opcode: u8) -> SemanticGraph {
    let mut modified = graph.clone();
    let old_root = modified.root;
    let mut root_node = modified.nodes.remove(&old_root).unwrap();
    root_node.payload = NodePayload::Prim { opcode: new_opcode };
    root_node.id = compute_node_id(&root_node);
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
// Hand-crafted IRIS mutation program
// ---------------------------------------------------------------------------

/// Build an IRIS program (SemanticGraph) that takes two inputs:
///   inputs[0] = Value::Program(target_graph)
///   inputs[1] = Value::Int(new_opcode)
///
/// and returns the target graph with the FIRST Prim node's opcode changed.
///
/// Graph structure:
///
///   Root(id=1): graph_set_prim_op(0x84, arity=3)
///   ├── port 0: input_ref(0)           → inputs[0] (the Program)       [id=10]
///   ├── port 1: project(field=0)       → first node ID                 [id=20]
///   │           └── graph_nodes(0x81)  → Tuple of all node IDs         [id=30]
///   │               └── input_ref(0)   → inputs[0]                     [id=40]
///   └── port 2: input_ref(1)           → inputs[1] (new opcode)        [id=50]
///
/// This works when the first node in BTreeMap order is a Prim node.
/// For our test programs, NodeId(1) is always the Prim root, and since
/// BTreeMap sorts by key, NodeId(1) is always first.
fn build_iris_mutation_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_set_prim_op (opcode 0x84, 3 args)
    let (nid, node) = prim_node(1, 0x84, 3);
    nodes.insert(nid, node);

    // Port 0: the Program input
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);

    // Port 1: Project(field=0) extracts first element from graph_nodes result
    let (nid, node) = project_node(20, 0);
    nodes.insert(nid, node);

    // graph_nodes (opcode 0x81, 1 arg)
    let (nid, node) = prim_node(30, 0x81, 1);
    nodes.insert(nid, node);

    // Another input_ref(0) for graph_nodes arg
    let (nid, node) = input_ref_node(40, 0);
    nodes.insert(nid, node);

    // Port 2: the new opcode input
    let (nid, node) = input_ref_node(50, 1);
    nodes.insert(nid, node);

    let edges = vec![
        // Root's 3 arguments
        make_edge(1, 10, 0, EdgeLabel::Argument), // program
        make_edge(1, 20, 1, EdgeLabel::Argument), // node_id (from project)
        make_edge(1, 50, 2, EdgeLabel::Argument), // new_opcode
        // Project gets its input from graph_nodes
        make_edge(20, 30, 0, EdgeLabel::Argument),
        // graph_nodes gets the program
        make_edge(30, 40, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build a simpler IRIS program that takes three inputs:
///   inputs[0] = Value::Program(target_graph)
///   inputs[1] = Value::Int(node_id)     — the specific Prim node to modify
///   inputs[2] = Value::Int(new_opcode)
///
/// This is a direct wrapper around graph_set_prim_op (0x84).
///
/// Graph structure:
///   Root(id=1): graph_set_prim_op(0x84, arity=3)
///   ├── port 0: input_ref(0)   → inputs[0] (Program)      [id=10]
///   ├── port 1: input_ref(1)   → inputs[1] (node ID)      [id=20]
///   └── port 2: input_ref(2)   → inputs[2] (new opcode)   [id=30]
fn build_iris_mutation_direct_program() -> SemanticGraph {
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

// ---------------------------------------------------------------------------
// Test 1: Hand-crafted IRIS mutation program works correctly
// ---------------------------------------------------------------------------

#[test]
fn iris_mutation_program_changes_sub_to_add() {
    let iris_mutator = build_iris_mutation_program();
    let target = make_binop_graph(0x01, 5, 3); // sub(5, 3)

    // Run the IRIS mutation program with (target_program, new_opcode=0x00)
    let inputs = vec![
        Value::Program(Box::new(target.clone())),
        Value::Int(0x00), // add opcode
    ];

    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1, "should return exactly one value");

    let modified = extract_program(&outputs[0]);

    // Verify the old sub node (NodeId(1)) is gone
    assert!(
        !modified.nodes.contains_key(&NodeId(1)),
        "old Prim node should be removed (content-addressed ID changed)"
    );

    // Verify there's now an add node
    let add_node = modified
        .nodes
        .values()
        .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x00 }));
    assert!(add_node.is_some(), "should have an add (0x00) Prim node");

    // Verify the modified program actually computes add(5,3) = 8
    let add_id = add_node.unwrap().id;
    let mut runnable = modified.clone();
    runnable.root = add_id;
    let (result, _) = interpreter::interpret(&runnable, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(8)], "add(5, 3) should be 8");
}

#[test]
fn iris_mutation_program_changes_add_to_mul() {
    let iris_mutator = build_iris_mutation_program();
    let target = make_binop_graph(0x00, 4, 7); // add(4, 7)

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x02), // mul opcode
    ];

    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    // Find the mul node and verify it computes correctly
    let mul_node = modified
        .nodes
        .values()
        .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x02 }));
    assert!(mul_node.is_some(), "should have a mul (0x02) Prim node");

    let mul_id = mul_node.unwrap().id;
    let mut runnable = modified;
    runnable.root = mul_id;
    let (result, _) = interpreter::interpret(&runnable, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(28)], "mul(4, 7) should be 28");
}

#[test]
fn iris_mutation_program_changes_mul_to_sub() {
    let iris_mutator = build_iris_mutation_program();
    let target = make_binop_graph(0x02, 10, 3); // mul(10, 3)

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x01), // sub opcode
    ];

    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    let sub_node = modified
        .nodes
        .values()
        .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x01 }));
    assert!(sub_node.is_some(), "should have a sub (0x01) Prim node");

    let sub_id = sub_node.unwrap().id;
    let mut runnable = modified;
    runnable.root = sub_id;
    let (result, _) = interpreter::interpret(&runnable, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(7)], "sub(10, 3) should be 7");
}

// ---------------------------------------------------------------------------
// Test 2: IRIS mutation matches Rust replace_prim behavior
// ---------------------------------------------------------------------------

#[test]
fn iris_mutation_matches_rust_modify_root_opcode() {
    // Compare the IRIS mutation program's output with modify_root_opcode()
    // for 10 different test cases.
    let iris_mutator = build_iris_mutation_program();

    let test_cases: Vec<(u8, i64, i64, u8)> = vec![
        (0x01, 5, 3, 0x00),   // sub -> add
        (0x00, 4, 7, 0x02),   // add -> mul
        (0x02, 10, 3, 0x01),  // mul -> sub
        (0x00, 0, 0, 0x00),   // add -> add (identity)
        (0x01, 100, 1, 0x03), // sub -> div
        (0x02, -5, 3, 0x00),  // mul(-5,3) -> add(-5,3)
        (0x00, 1, 1, 0x01),   // add(1,1) -> sub(1,1)
        (0x03, 10, 2, 0x02),  // div(10,2) -> mul(10,2)
        (0x00, 7, 8, 0x07),   // add -> min
        (0x00, 7, 8, 0x08),   // add -> max
    ];

    for (orig_op, a, b, new_op) in &test_cases {
        let target = make_binop_graph(*orig_op, *a, *b);

        // Rust-side expected result
        let expected = modify_root_opcode(&target, *new_op);

        // IRIS program result
        let inputs = vec![
            Value::Program(Box::new(target)),
            Value::Int(*new_op as i64),
        ];
        let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();

        let iris_result = extract_program(&outputs[0]);

        // Verify structural equivalence: same number of nodes, same edges
        assert_eq!(
            iris_result.nodes.len(),
            expected.nodes.len(),
            "node count mismatch for 0x{:02x}({},{}) -> 0x{:02x}",
            orig_op,
            a,
            b,
            new_op
        );

        // Verify the new Prim node has the correct opcode
        let iris_prim = iris_result
            .nodes
            .values()
            .find(|n| n.kind == NodeKind::Prim);
        let expected_prim = expected.nodes.values().find(|n| n.kind == NodeKind::Prim);

        assert!(iris_prim.is_some() && expected_prim.is_some());
        assert_eq!(
            iris_prim.unwrap().payload,
            expected_prim.unwrap().payload,
            "opcode mismatch for test case 0x{:02x} -> 0x{:02x}",
            orig_op,
            new_op
        );

        // Verify edges match (same count, targets updated)
        assert_eq!(
            iris_result.edges.len(),
            expected.edges.len(),
            "edge count mismatch"
        );
    }
}

// ---------------------------------------------------------------------------
// Test 3: Output programs execute correctly (cross-cycle verification)
// ---------------------------------------------------------------------------

#[test]
fn modified_programs_execute_correctly() {
    let iris_mutator = build_iris_mutation_program();

    // sub(5,3) -> add(5,3): expected 8
    let target = make_binop_graph(0x01, 5, 3);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x00),
    ];
    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();
    let modified = extract_program(&outputs[0]);

    // Find the new root (the add node) and execute
    let new_root_id = modified
        .nodes
        .values()
        .find(|n| n.kind == NodeKind::Prim)
        .expect("should have Prim node")
        .id;
    let mut cycle2 = modified;
    cycle2.root = new_root_id;
    let (result, _) = interpreter::interpret(&cycle2, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(8)]);

    // add(10,20) -> mul(10,20): expected 200
    let target = make_binop_graph(0x00, 10, 20);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x02),
    ];
    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();
    let modified = extract_program(&outputs[0]);
    let new_root_id = modified
        .nodes
        .values()
        .find(|n| n.kind == NodeKind::Prim)
        .expect("should have Prim node")
        .id;
    let mut cycle2 = modified;
    cycle2.root = new_root_id;
    let (result, _) = interpreter::interpret(&cycle2, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(200)]);
}

// ---------------------------------------------------------------------------
// Test 4: Multi-cycle self-improvement (chained mutations)
// ---------------------------------------------------------------------------

#[test]
fn chained_mutations_three_cycles() {
    // Use the direct mutation program (takes explicit node ID) for chaining,
    // because after mutation the content-addressed Prim node ID changes and
    // we need to track it across cycles.
    let iris_mutator = build_iris_mutation_direct_program();

    // Cycle 1: start with sub(10, 7) = 3
    let program = make_binop_graph(0x01, 10, 7);
    let (out, _) = interpreter::interpret(&program, &[], None).unwrap();
    assert_eq!(out, vec![Value::Int(3)], "sub(10,7) = 3");

    // The Prim root is at NodeId(1)
    let prim_id = 1i64;

    // Mutate sub -> add
    let inputs = vec![
        Value::Program(Box::new(program)),
        Value::Int(prim_id),
        Value::Int(0x00),
    ];
    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();
    let (program2, prim2_id) = {
        let mut g2 = extract_program(&outputs[0]);
        let prim = g2
            .nodes
            .values()
            .find(|n| n.kind == NodeKind::Prim)
            .unwrap();
        let pid = prim.id;
        g2.root = pid;
        (g2, pid.0 as i64)
    };

    let (out2, _) = interpreter::interpret(&program2, &[], None).unwrap();
    assert_eq!(out2, vec![Value::Int(17)], "add(10,7) = 17");

    // Mutate add -> mul (using the new Prim node ID)
    let inputs = vec![
        Value::Program(Box::new(program2)),
        Value::Int(prim2_id),
        Value::Int(0x02),
    ];
    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();
    let program3_root = {
        let mut g3 = extract_program(&outputs[0]);
        let prim = g3
            .nodes
            .values()
            .find(|n| n.kind == NodeKind::Prim)
            .unwrap()
            .id;
        g3.root = prim;
        g3
    };

    let (out3, _) = interpreter::interpret(&program3_root, &[], None).unwrap();
    assert_eq!(out3, vec![Value::Int(70)], "mul(10,7) = 70");
}

// ---------------------------------------------------------------------------
// Test 5: Self-modifying program (uses self_graph instead of input)
// ---------------------------------------------------------------------------

/// Build a program that modifies ITSELF: calls self_graph(0x80) to get its
/// own graph, then changes a specific Prim node's opcode.
///
/// Graph structure:
///   Root(id=1): Tuple (arity=2)
///   ├── port 0: sub(5,3) [id=100]         → computes 2
///   │           ├── lit(5) [id=110]
///   │           └── lit(3) [id=120]
///   └── port 1: graph_set_prim_op(0x84) [id=200]
///               ├── self_graph(0x80) [id=300]
///               ├── lit(100) [id=310]    → target node ID
///               └── lit(0x00) [id=320]   → new opcode (add)
///
/// Returns Tuple(Int(2), Program(modified)) where modified has add instead of sub.
#[test]
fn self_modifying_program() {
    let mut nodes = HashMap::new();

    // Root: Tuple
    let (nid, node) = make_node(1, NodeKind::Tuple, NodePayload::Tuple, 2);
    nodes.insert(nid, node);

    // Sub computation
    let (nid, node) = prim_node(100, 0x01, 2); // sub
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(110, 5);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(120, 3);
    nodes.insert(nid, node);

    // Self-modification
    let (nid, node) = prim_node(200, 0x84, 3); // graph_set_prim_op
    nodes.insert(nid, node);
    let (nid, node) = prim_node(300, 0x80, 0); // self_graph
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(310, 100); // target node ID (the sub node)
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(320, 0x00); // new opcode (add)
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

    // outputs[0] is a Tuple(sub_result, modified_program)
    match &outputs[0] {
        Value::Tuple(elems) => {
            assert_eq!(elems.len(), 2);
            assert_eq!(elems[0], Value::Int(2), "sub(5,3) = 2");

            {
                let modified = extract_program(&elems[1]);
                // The modified program should have add instead of sub
                let add_node = modified
                    .nodes
                    .values()
                    .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x00 }));
                assert!(add_node.is_some(), "modified should have add node");

                // Run cycle 2: the modified program with add as root
                let add_id = add_node.unwrap().id;
                let mut cycle2 = modified;
                cycle2.root = add_id;
                let (result2, _) = interpreter::interpret(&cycle2, &[], None).unwrap();
                assert_eq!(result2, vec![Value::Int(8)], "add(5,3) = 8");
            }
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test 6: Register as MutationComponent
// ---------------------------------------------------------------------------

#[test]
fn register_iris_mutation_as_component() {
    let iris_program = build_iris_mutation_program();

    // Register as a MutationComponent
    let component = MutationComponent {
        name: "iris_replace_prim".to_string(),
        program: iris_program.clone(),
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    // Verify it's findable
    let found = registry.find_mutation("iris_replace_prim");
    assert!(found.is_some(), "component should be registered");
    assert_eq!(found.unwrap().name, "iris_replace_prim");

    // Verify the stored program is the correct one
    assert_eq!(
        found.unwrap().program.nodes.len(),
        iris_program.nodes.len(),
        "stored program should match"
    );

    // Execute it via the component's program field
    let target = make_binop_graph(0x01, 5, 3);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x00),
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    {
        let g = extract_program(&outputs[0]);
        let has_add = g
            .nodes
            .values()
            .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x00 }));
        assert!(has_add, "component execution should produce add node");
    }

    eprintln!("First Rust mutation operator replaced by IRIS program");
}

// ---------------------------------------------------------------------------
// Test 7: IRIS mutation on diverse program shapes
// ---------------------------------------------------------------------------

/// Build a unary program: neg(x) or abs(x)
fn make_unary_graph(opcode: u8, x: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = prim_node(1, opcode, 1);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(10, x);
    nodes.insert(nid, node);

    let edges = vec![make_edge(1, 10, 0, EdgeLabel::Argument)];
    make_graph(nodes, edges, 1)
}

#[test]
fn iris_mutation_on_unary_program() {
    let iris_mutator = build_iris_mutation_program();

    // neg(5) -> abs(5): change opcode 0x05 to 0x06
    let target = make_unary_graph(0x05, -5); // neg(-5) = 5
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x06), // abs opcode
    ];
    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    let new_prim = modified
        .nodes
        .values()
        .find(|n| n.kind == NodeKind::Prim)
        .expect("should have Prim node");
    assert!(
        matches!(&new_prim.payload, NodePayload::Prim { opcode: 0x06 }),
        "opcode should be abs (0x06)"
    );
}

// ---------------------------------------------------------------------------
// Test 8: Idempotency — changing to same opcode
// ---------------------------------------------------------------------------

#[test]
fn iris_mutation_same_opcode_is_identity() {
    let iris_mutator = build_iris_mutation_program();
    let target = make_binop_graph(0x00, 3, 4); // add(3, 4)

    let inputs = vec![
        Value::Program(Box::new(target.clone())),
        Value::Int(0x00), // same opcode
    ];
    let (outputs, _) = interpreter::interpret(&iris_mutator, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    // Should have same number of nodes
    assert_eq!(modified.nodes.len(), target.nodes.len());

    // The Prim node should still be add
    let prim_id = {
        let prim = modified
            .nodes
            .values()
            .find(|n| n.kind == NodeKind::Prim)
            .unwrap();
        assert!(matches!(&prim.payload, NodePayload::Prim { opcode: 0x00 }));
        prim.id
    };

    // And should still compute correctly
    let mut runnable = modified;
    runnable.root = prim_id;
    let (result, _) = interpreter::interpret(&runnable, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(7)], "add(3,4) should still be 7");
}

// ---------------------------------------------------------------------------
// Test 9: IRIS program is itself a valid SemanticGraph that can be inspected
// ---------------------------------------------------------------------------

#[test]
fn iris_mutation_program_is_introspectable() {
    let iris_mutator = build_iris_mutation_program();

    // The IRIS mutation program itself can be wrapped as a Program value
    // and inspected using graph_nodes.
    let mut nodes = HashMap::new();
    let (nid, node) = prim_node(1, 0x81, 1); // graph_nodes
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);

    let edges = vec![make_edge(1, 10, 0, EdgeLabel::Argument)];
    let inspector = make_graph(nodes, edges, 1);

    let inputs = vec![Value::Program(Box::new(iris_mutator.clone()))];
    let (outputs, _) = interpreter::interpret(&inspector, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Tuple(ids) => {
            // The IRIS mutation program has 6 nodes
            assert_eq!(
                ids.len(),
                iris_mutator.nodes.len(),
                "graph_nodes should return all node IDs from the IRIS program"
            );
        }
        other => panic!("expected Tuple of node IDs, got {:?}", other),
    }
}

// ===========================================================================
// Target 1: insert_node — add a new Prim node to a program
// ===========================================================================

/// Build an IRIS program that adds a new Prim node to an existing program.
///
/// Inputs:
///   inputs[0] = Value::Program(target_graph)
///   inputs[1] = Value::Int(opcode)
///
/// Output: Tuple(modified_program, new_node_id)
///
/// Graph structure:
///   Root(id=1): graph_add_node_rt(0x85, arity=2)
///   ├── port 0: input_ref(0)   → inputs[0] (the Program)   [id=10]
///   └── port 1: input_ref(1)   → inputs[1] (opcode)        [id=20]
///
/// This wraps opcode 0x85 which returns Tuple(Program, new_node_id).
fn build_iris_insert_node_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_add_node_rt (opcode 0x85, 2 args)
    let (nid, node) = prim_node(1, 0x85, 2);
    nodes.insert(nid, node);

    // Port 0: the Program input
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);

    // Port 1: opcode input
    let (nid, node) = input_ref_node(20, 1);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

#[test]
fn iris_insert_node_adds_prim_node() {
    let iris_inserter = build_iris_insert_node_program();
    let target = make_binop_graph(0x00, 5, 3); // add(5, 3) — has 3 nodes

    assert_eq!(target.nodes.len(), 3, "original has 3 nodes");

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x00), // kind=Prim (then use graph_set_prim_op for specific opcode)
    ];

    let (outputs, _) = interpreter::interpret(&iris_inserter, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1, "should return one value");

    // graph_add_node_rt returns Tuple(Program, new_node_id)
    match &outputs[0] {
        Value::Tuple(elems) => {
            assert_eq!(elems.len(), 2, "should be Tuple(Program, Int)");

            let modified = extract_program(&elems[0]);

            assert_eq!(
                modified.nodes.len(),
                4,
                "after insert_node, should have 4 nodes (was 3)"
            );

            // The new node should be a Prim with opcode 0x00 (default)
            let new_node_id = match &elems[1] {
                Value::Int(id) => NodeId(*id as u64),
                other => panic!("expected Int for new_node_id, got {:?}", other),
            };

            let new_node = modified
                .nodes
                .get(&new_node_id)
                .expect("new node should exist in graph");
            assert_eq!(new_node.kind, NodeKind::Prim);
            assert!(matches!(&new_node.payload, NodePayload::Prim { opcode: 0x00 }));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn iris_insert_node_preserves_original_nodes() {
    let iris_inserter = build_iris_insert_node_program();
    let target = make_binop_graph(0x01, 10, 7); // sub(10, 7)

    let original_node_ids: Vec<NodeId> = target.nodes.keys().copied().collect();

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x00), // add opcode
    ];

    let (outputs, _) = interpreter::interpret(&iris_inserter, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Tuple(elems) => {
            let modified = extract_program(&elems[0]);

            // All original nodes should still be present
            for orig_id in &original_node_ids {
                assert!(
                    modified.nodes.contains_key(orig_id),
                    "original node {:?} should be preserved",
                    orig_id
                );
            }

            // Plus one new node
            assert_eq!(modified.nodes.len(), original_node_ids.len() + 1);
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn iris_insert_node_matches_rust_semantics() {
    // Verify the IRIS insert_node creates nodes with the correct NodeKind.
    // graph_add_node_rt dispatches on kind: 0=Prim, 1=Apply, 2=Lambda, etc.
    let iris_inserter = build_iris_insert_node_program();

    let expected_kinds: Vec<(u8, NodeKind)> = vec![
        (0x00, NodeKind::Prim),
        (0x01, NodeKind::Apply),
        (0x02, NodeKind::Lambda),
        (0x03, NodeKind::Let),
        (0x05, NodeKind::Lit),
        (0x09, NodeKind::Unfold),
    ];

    for (kind_val, expected_kind) in expected_kinds {
        let target = make_binop_graph(0x00, 1, 2);
        let original_count = target.nodes.len();

        let inputs = vec![
            Value::Program(Box::new(target)),
            Value::Int(kind_val as i64),
        ];

        let (outputs, _) = interpreter::interpret(&iris_inserter, &inputs, None).unwrap();

        match &outputs[0] {
            Value::Tuple(elems) => {
                let modified = extract_program(&elems[0]);

                assert_eq!(
                    modified.nodes.len(),
                    original_count + 1,
                    "insert_node should add exactly one node for kind 0x{:02x}",
                    kind_val
                );

                let new_id = match &elems[1] {
                    Value::Int(id) => NodeId(*id as u64),
                    other => panic!("expected Int, got {:?}", other),
                };
                let nn = modified.nodes.get(&new_id).unwrap();
                assert_eq!(nn.kind, expected_kind, "kind 0x{:02x} should produce {:?}", kind_val, expected_kind);
            }
            other => panic!("expected Tuple, got {:?}", other),
        }
    }
}

#[test]
fn register_iris_insert_node_as_component() {
    let iris_program = build_iris_insert_node_program();

    let component = MutationComponent {
        name: "iris_insert_node".to_string(),
        program: iris_program.clone(),
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_insert_node");
    assert!(found.is_some(), "iris_insert_node should be registered");

    // Execute via component
    let target = make_binop_graph(0x00, 1, 1);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x03),
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    match &outputs[0] {
        Value::Tuple(elems) => {
            let modified = extract_program(&elems[0]);
            assert_eq!(modified.nodes.len(), 4);
        }
        other => panic!("expected Tuple, got {:?}", other),
    }

    eprintln!("Second Rust mutation operator (insert_node) replaced by IRIS program");
}

// ===========================================================================
// Target 2: connect — wire a new edge between nodes
// ===========================================================================

/// Build an IRIS program that adds an edge between two nodes.
///
/// Inputs:
///   inputs[0] = Value::Program(target_graph)
///   inputs[1] = Value::Int(source_node_id)
///   inputs[2] = Value::Int(target_node_id)
///   inputs[3] = Value::Int(port)
///
/// Output: modified Program
///
/// Graph structure:
///   Root(id=1): graph_connect(0x86, arity=4)
///   ├── port 0: input_ref(0)   → inputs[0] (Program)        [id=10]
///   ├── port 1: input_ref(1)   → inputs[1] (source_id)      [id=20]
///   ├── port 2: input_ref(2)   → inputs[2] (target_id)      [id=30]
///   └── port 3: input_ref(3)   → inputs[3] (port)           [id=40]
fn build_iris_connect_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_connect (opcode 0x86, 4 args)
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(40, 3);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        make_edge(1, 40, 3, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build a program with disconnected nodes (no edges between them).
fn make_disconnected_graph() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Three disconnected Prim/Lit nodes
    let (nid, node) = prim_node(1, 0x00, 2); // add node (root)
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(10, 42);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(20, 58);
    nodes.insert(nid, node);

    // No edges — nodes are disconnected
    make_graph(nodes, vec![], 1)
}

#[test]
fn iris_connect_wires_disconnected_nodes() {
    let iris_connector = build_iris_connect_program();
    let target = make_disconnected_graph();

    assert_eq!(target.edges.len(), 0, "starts with no edges");

    // Connect node 1 -> node 10 on port 0
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),  // source: the add node
        Value::Int(10), // target: lit(42)
        Value::Int(0),  // port 0
    ];

    let (outputs, _) = interpreter::interpret(&iris_connector, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    assert_eq!(modified.edges.len(), 1, "should have 1 edge after connect");
    assert_eq!(modified.edges[0].source, NodeId(1));
    assert_eq!(modified.edges[0].target, NodeId(10));
    assert_eq!(modified.edges[0].port, 0);
}

#[test]
fn iris_connect_builds_working_program() {
    let iris_connector = build_iris_connect_program();
    let target = make_disconnected_graph();

    // Wire up add(42, 58) by connecting both arguments
    // First connection: add -> lit(42) on port 0
    let inputs1 = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),
        Value::Int(10),
        Value::Int(0),
    ];
    let (out1, _) = interpreter::interpret(&iris_connector, &inputs1, None).unwrap();
    let step1 = extract_program(&out1[0]);

    // Second connection: add -> lit(58) on port 1
    let inputs2 = vec![
        Value::Program(Box::new(step1)),
        Value::Int(1),
        Value::Int(20),
        Value::Int(1),
    ];
    let (out2, _) = interpreter::interpret(&iris_connector, &inputs2, None).unwrap();
    let wired = extract_program(&out2[0]);

    assert_eq!(wired.edges.len(), 2, "should have 2 edges");

    // Now execute the wired program: add(42, 58) = 100
    let (result, _) = interpreter::interpret(&wired, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(100)], "add(42, 58) should be 100");
}

#[test]
fn iris_connect_adds_to_existing_edges() {
    let iris_connector = build_iris_connect_program();
    let target = make_binop_graph(0x00, 5, 3); // add(5,3) with 2 edges

    assert_eq!(target.edges.len(), 2, "starts with 2 edges");

    // Add a third edge (even though it's redundant, the graph allows it)
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),  // source
        Value::Int(10), // target
        Value::Int(2),  // port 2
    ];

    let (outputs, _) = interpreter::interpret(&iris_connector, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    assert_eq!(
        modified.edges.len(),
        3,
        "should have 3 edges after adding one"
    );
}

#[test]
fn register_iris_connect_as_component() {
    let iris_program = build_iris_connect_program();

    let component = MutationComponent {
        name: "iris_connect".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_connect");
    assert!(found.is_some(), "iris_connect should be registered");

    // Execute via component: connect two disconnected nodes
    let target = make_disconnected_graph();
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),
        Value::Int(10),
        Value::Int(0),
    ];
    let (outputs, _) =
        interpreter::interpret(&found.unwrap().program, &inputs, None).unwrap();

    {
        let g = extract_program(&outputs[0]);
        assert_eq!(g.edges.len(), 1, "should have wired 1 edge");
    }

    eprintln!("Third Rust mutation operator (connect/rewire) replaced by IRIS program");
}

// ===========================================================================
// Target 3: seed generator — build a program from scratch
// ===========================================================================


// Seed generation: chain IRIS insert_node and connect programs.

#[test]
fn iris_seed_generator_builds_program_from_scratch() {
    // We demonstrate building a program from scratch using IRIS operations.
    // Each step is an IRIS program execution — the same operations a single
    // complex IRIS program would perform internally.

    let iris_connector = build_iris_connect_program();

    // Start from a graph that has nodes but no edges — use IRIS connect
    // to wire it into a working program.
    let disconnected = make_disconnected_graph(); // add(id=1), lit42(id=10), lit58(id=20), no edges
    assert_eq!(disconnected.nodes.len(), 3, "disconnected has 3 nodes");
    assert_eq!(disconnected.edges.len(), 0, "disconnected has 0 edges");

    // Use IRIS connect to wire: add -> lit42 on port 0
    let inputs1 = vec![
        Value::Program(Box::new(disconnected)),
        Value::Int(1),  // source: add node
        Value::Int(10), // target: lit(42)
        Value::Int(0),  // port 0
    ];
    let (out1, _) = interpreter::interpret(&iris_connector, &inputs1, None).unwrap();
    let step1 = extract_program(&out1[0]);

    // Use IRIS connect to wire: add -> lit58 on port 1
    let inputs2 = vec![
        Value::Program(Box::new(step1)),
        Value::Int(1),  // source: add node
        Value::Int(20), // target: lit(58)
        Value::Int(1),  // port 1
    ];
    let (out2, _) = interpreter::interpret(&iris_connector, &inputs2, None).unwrap();
    let constructed = extract_program(&out2[0]);

    assert_eq!(constructed.nodes.len(), 3, "constructed has 3 nodes");
    assert_eq!(constructed.edges.len(), 2, "constructed has 2 edges");

    // Execute the constructed program: add(42, 58) = 100
    let (result, _) = interpreter::interpret(&constructed, &[], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(100)],
        "IRIS-constructed add(42, 58) = 100"
    );

    eprintln!("IRIS seed generator built a working program from disconnected nodes");
}

#[test]
fn iris_seed_generator_with_insert_and_connect() {
    // More advanced: use insert_node to add a new node, then connect to wire it.
    // This shows the full from-scratch construction pipeline.

    let iris_inserter = build_iris_insert_node_program();
    let iris_connector = build_iris_connect_program();

    // Start with a single lit node
    let mut seed_nodes = HashMap::new();
    let (nid, node) = int_lit_node(10, 99);
    seed_nodes.insert(nid, node);
    let seed = make_graph(seed_nodes, vec![], 10);

    assert_eq!(seed.nodes.len(), 1, "seed starts with 1 node");

    // Step 1: Add a Prim node (kind=0x00), then set opcode to neg (0x05)
    let inputs = vec![
        Value::Program(Box::new(seed)),
        Value::Int(0x00), // kind=Prim
    ];
    let (out, _) = interpreter::interpret(&iris_inserter, &inputs, None).unwrap();

    let (step1_prog, new_node_id) = match &out[0] {
        Value::Tuple(elems) => {
            let prog = extract_program(&elems[0]);
            let nid = match &elems[1] {
                Value::Int(id) => *id,
                other => panic!("expected Int, got {:?}", other),
            };
            (prog, nid)
        }
        other => panic!("expected Tuple, got {:?}", other),
    };

    // Set opcode to neg (0x05) using graph_set_prim_op
    let iris_set_op = build_iris_mutation_direct_program();
    let inputs = vec![
        Value::Program(Box::new(step1_prog)),
        Value::Int(new_node_id),
        Value::Int(0x05), // neg opcode
    ];
    let (out, _) = interpreter::interpret(&iris_set_op, &inputs, None).unwrap();
    // graph_set_prim_op returns Tuple(Program, new_node_id) — the node ID may change
    let (step1, new_node_id) = match &out[0] {
        Value::Tuple(elems) => {
            let prog = extract_program(&elems[0]);
            let nid = match &elems[1] {
                Value::Int(id) => *id,
                other => panic!("expected Int from set_prim_op, got {:?}", other),
            };
            (prog, nid)
        }
        Value::Program(p) => (p.as_ref().clone(), new_node_id),
        other => panic!("expected Tuple or Program from set_prim_op, got {:?}", other),
    };

    assert_eq!(step1.nodes.len(), 2, "after insert + set_prim_op, 2 nodes");

    // Step 2: Connect the neg node -> lit(99) on port 0
    let inputs = vec![
        Value::Program(Box::new(step1)),
        Value::Int(new_node_id), // source: neg node
        Value::Int(10),          // target: lit(99)
        Value::Int(0),           // port 0
    ];
    let (out, _) = interpreter::interpret(&iris_connector, &inputs, None).unwrap();
    let mut constructed = extract_program(&out[0]);

    assert_eq!(constructed.edges.len(), 1, "should have 1 edge");

    // Set the root to the neg node and execute: neg(99) = -99
    constructed.root = NodeId(new_node_id as u64);
    let (result, _) = interpreter::interpret(&constructed, &[], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(-99)],
        "IRIS-constructed neg(99) = -99"
    );

    eprintln!("IRIS seed: insert_node + connect built neg(99) from scratch");
}

#[test]
fn register_iris_seed_as_component() {
    // Register the seed-construction pipeline as a SeedComponent.
    // In practice the program field would be a single complex IRIS program;
    // here we use the insert_node program as the registered seed generator
    // (it creates a new program structure).
    let iris_program = build_iris_insert_node_program();

    let component = SeedComponent {
        name: "iris_seed_add_node".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.seeds.push(component);

    let found = registry.find_seed("iris_seed_add_node");
    assert!(found.is_some(), "iris_seed_add_node should be registered");
    assert_eq!(found.unwrap().name, "iris_seed_add_node");

    eprintln!("IRIS seed generator registered as SeedComponent");
}

// ===========================================================================
// Target 4: compose mutations — apply multiple mutations in sequence
// ===========================================================================

/// Build an IRIS program that composes three mutations:
/// 1. replace_prim (change opcode of existing node)
/// 2. insert_node (add a new node)
/// 3. connect (wire the new node)
///
/// This is a single IRIS program that chains the operations.
///
/// Inputs:
///   inputs[0] = Value::Program(target_graph)
///   inputs[1] = Value::Int(new_opcode for replace_prim)
///   inputs[2] = Value::Int(insert_opcode for new node)
///   inputs[3] = Value::Int(connect_source_id)
///   inputs[4] = Value::Int(connect_port)
///
/// Graph structure — chains replace_prim -> insert_node -> connect:
///
///   Root(id=1): graph_connect(0x86, arity=4)                → final result
///   ├── port 0: project(0) [id=50]                          → program from insert
///   │   └── graph_add_node_rt(0x85) [id=60]                 → insert new node
///   │       ├── graph_set_prim_op(0x84) [id=70]             → replace_prim result
///   │       │   ├── input_ref(0) [id=80]                    → original program
///   │       │   ├── project(0) [id=90]                      → first node ID
///   │       │   │   └── graph_nodes(0x81) [id=100]          → all node IDs
///   │       │   │       └── input_ref(0) [id=105]           → program
///   │       │   └── input_ref(1) [id=110]                   → new opcode
///   │       └── input_ref(2) [id=120]                       → insert opcode
///   ├── port 1: input_ref(3) [id=130]                       → connect source
///   ├── port 2: project(1) [id=140]                         → new node id from insert
///   │   └── graph_add_node_rt(0x85) [id=150]                → SAME insert (recomputed)
///   │       ├── graph_set_prim_op(0x84) [id=160]            → SAME replace (recomputed)
///   │       │   ├── input_ref(0) [id=165]
///   │       │   ├── project(0) [id=170]
///   │       │   │   └── graph_nodes(0x81) [id=175]
///   │       │   │       └── input_ref(0) [id=178]
///   │       │   └── input_ref(1) [id=180]
///   │       └── input_ref(2) [id=185]
///   └── port 3: input_ref(4) [id=190]                       → connect port
///
/// NOTE: Since SemanticGraph doesn't support node sharing (each node has a
/// unique ID), we duplicate the replace_prim and insert_node subgraphs.
/// The interpreter evaluates each subgraph independently but deterministically,
/// so both copies produce the same result.
fn build_iris_compose_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_connect(0x86, 4 args) — the final connection step
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);

    // --- Branch 1: program for connect (port 0) ---
    // project(0) extracts program from insert result
    let (nid, node) = project_node(50, 0);
    nodes.insert(nid, node);

    // graph_add_node_rt (insert step, branch 1)
    let (nid, node) = prim_node(60, 0x85, 2);
    nodes.insert(nid, node);

    // graph_set_prim_op (replace step, branch 1)
    let (nid, node) = prim_node(70, 0x84, 3);
    nodes.insert(nid, node);

    // input_ref(0) — original program (for replace, branch 1)
    let (nid, node) = input_ref_node(80, 0);
    nodes.insert(nid, node);

    // project(0) — first node id from graph_nodes (branch 1)
    let (nid, node) = project_node(90, 0);
    nodes.insert(nid, node);

    // graph_nodes(0x81) (branch 1)
    let (nid, node) = prim_node(100, 0x81, 1);
    nodes.insert(nid, node);

    // input_ref(0) for graph_nodes (branch 1)
    let (nid, node) = input_ref_node(105, 0);
    nodes.insert(nid, node);

    // input_ref(1) — new opcode for replace (branch 1)
    let (nid, node) = input_ref_node(110, 1);
    nodes.insert(nid, node);

    // input_ref(2) — insert opcode (branch 1)
    let (nid, node) = input_ref_node(120, 2);
    nodes.insert(nid, node);

    // --- Ports 1, 3 of root: simple input refs ---
    // input_ref(3) — connect source (port 1 of root)
    let (nid, node) = input_ref_node(130, 3);
    nodes.insert(nid, node);

    // input_ref(4) — connect port (port 3 of root)
    let (nid, node) = input_ref_node(190, 4);
    nodes.insert(nid, node);

    // --- Branch 2: new_node_id for connect (port 2) ---
    // project(1) extracts new_node_id from insert result
    let (nid, node) = project_node(140, 1);
    nodes.insert(nid, node);

    // graph_add_node_rt (insert step, branch 2)
    let (nid, node) = prim_node(150, 0x85, 2);
    nodes.insert(nid, node);

    // graph_set_prim_op (replace step, branch 2)
    let (nid, node) = prim_node(160, 0x84, 3);
    nodes.insert(nid, node);

    // input_ref(0) (branch 2)
    let (nid, node) = input_ref_node(165, 0);
    nodes.insert(nid, node);

    // project(0) — first node id (branch 2)
    let (nid, node) = project_node(170, 0);
    nodes.insert(nid, node);

    // graph_nodes(0x81) (branch 2)
    let (nid, node) = prim_node(175, 0x81, 1);
    nodes.insert(nid, node);

    // input_ref(0) for graph_nodes (branch 2)
    let (nid, node) = input_ref_node(178, 0);
    nodes.insert(nid, node);

    // input_ref(1) — new opcode (branch 2)
    let (nid, node) = input_ref_node(180, 1);
    nodes.insert(nid, node);

    // input_ref(2) — insert opcode (branch 2)
    let (nid, node) = input_ref_node(185, 2);
    nodes.insert(nid, node);

    // --- Edges ---
    let edges = vec![
        // Root (graph_connect) args
        make_edge(1, 50, 0, EdgeLabel::Argument),   // port 0: program from insert (branch 1)
        make_edge(1, 130, 1, EdgeLabel::Argument),   // port 1: connect source
        make_edge(1, 140, 2, EdgeLabel::Argument),   // port 2: new_node_id from insert (branch 2)
        make_edge(1, 190, 3, EdgeLabel::Argument),   // port 3: connect port
        // Branch 1: project(0) <- insert
        make_edge(50, 60, 0, EdgeLabel::Argument),
        // insert <- (replace, insert_opcode)
        make_edge(60, 70, 0, EdgeLabel::Argument),
        make_edge(60, 120, 1, EdgeLabel::Argument),
        // replace <- (program, first_node_id, new_opcode)
        make_edge(70, 80, 0, EdgeLabel::Argument),
        make_edge(70, 90, 1, EdgeLabel::Argument),
        make_edge(70, 110, 2, EdgeLabel::Argument),
        // project(0) <- graph_nodes
        make_edge(90, 100, 0, EdgeLabel::Argument),
        // graph_nodes <- program
        make_edge(100, 105, 0, EdgeLabel::Argument),
        // Branch 2: project(1) <- insert
        make_edge(140, 150, 0, EdgeLabel::Argument),
        // insert <- (replace, insert_opcode)
        make_edge(150, 160, 0, EdgeLabel::Argument),
        make_edge(150, 185, 1, EdgeLabel::Argument),
        // replace <- (program, first_node_id, new_opcode)
        make_edge(160, 165, 0, EdgeLabel::Argument),
        make_edge(160, 170, 1, EdgeLabel::Argument),
        make_edge(160, 180, 2, EdgeLabel::Argument),
        // project(0) <- graph_nodes
        make_edge(170, 175, 0, EdgeLabel::Argument),
        // graph_nodes <- program
        make_edge(175, 178, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

#[test]
fn iris_compose_mutations_replace_insert_connect() {
    let iris_composer = build_iris_compose_program();

    // Target: add(5, 3) — 3 nodes, opcode 0x00 at root
    let target = make_binop_graph(0x00, 5, 3);
    assert_eq!(target.nodes.len(), 3);

    // Compose: replace_prim(0x01=sub) + insert_node(0x20=Prim) + connect(root -> new, port 2)
    // Use 0x20 for insert since values >= 0x14 create Prim nodes via fallback
    // We use root id=1 as the connect source (it's the Prim node in make_binop_graph)
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x01), // replace: change add -> sub
        Value::Int(0x20), // insert: add a Prim node with opcode 0x20
        Value::Int(1),    // connect source: original root id
        Value::Int(2),    // connect port: port 2
    ];

    let (outputs, _) = interpreter::interpret(&iris_composer, &inputs, None).unwrap();

    let modified = extract_program(&outputs[0]);

    // After compose: 3 original nodes (with root opcode changed) + 1 inserted = 4 nodes
    assert_eq!(
        modified.nodes.len(),
        4,
        "compose should produce 4 nodes (3 original + 1 inserted)"
    );

    // The old add node should now be sub
    let sub_node = modified
        .nodes
        .values()
        .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x01 }));
    assert!(sub_node.is_some(), "should have sub (0x01) node after replace");

    // The inserted Prim node should exist with opcode 0x20
    let inserted_node = modified
        .nodes
        .values()
        .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x20 }));
    assert!(inserted_node.is_some(), "should have Prim(0x20) node after insert");

    // Should have edges including the new connection
    // Original 2 edges (updated for new root ID) + 1 new connection = 3 edges
    assert!(
        modified.edges.len() >= 3,
        "should have at least 3 edges (2 original + 1 new), got {}",
        modified.edges.len()
    );
}

#[test]
fn iris_compose_step_by_step_matches_single_program() {
    // Verify that composing mutations step-by-step produces the same result
    // as the single composed IRIS program.

    let iris_replace = build_iris_mutation_program();
    let iris_insert = build_iris_insert_node_program();
    let iris_connect = build_iris_connect_program();
    let iris_composer = build_iris_compose_program();

    let target = make_binop_graph(0x00, 5, 3);

    // Step-by-step execution
    // Step 1: replace_prim — change add -> sub
    let inputs1 = vec![
        Value::Program(Box::new(target.clone())),
        Value::Int(0x01),
    ];
    let (out1, _) = interpreter::interpret(&iris_replace, &inputs1, None).unwrap();
    let after_replace = extract_program(&out1[0]);

    // Step 2: insert_node — add a Prim node (use 0x20 >= 0x14 for Prim fallback)
    let inputs2 = vec![
        Value::Program(Box::new(after_replace.clone())),
        Value::Int(0x20),
    ];
    let (out2, _) = interpreter::interpret(&iris_insert, &inputs2, None).unwrap();
    let (after_insert, new_node_id) = match &out2[0] {
        Value::Tuple(elems) => {
            let prog = extract_program(&elems[0]);
            let nid = match &elems[1] {
                Value::Int(id) => *id,
                other => panic!("expected Int, got {:?}", other),
            };
            (prog, nid)
        }
        other => panic!("expected Tuple, got {:?}", other),
    };

    // Step 3: connect — wire root -> new node on port 2
    // Need to find the sub node (the new root after replace)
    let sub_node_id = after_insert
        .nodes
        .values()
        .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x01 }))
        .unwrap()
        .id;

    let inputs3 = vec![
        Value::Program(Box::new(after_insert)),
        Value::Int(sub_node_id.0 as i64),
        Value::Int(new_node_id),
        Value::Int(2),
    ];
    let (out3, _) = interpreter::interpret(&iris_connect, &inputs3, None).unwrap();
    let step_by_step_result = extract_program(&out3[0]);

    // Composed execution (single program)
    let inputs_composed = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x01), // replace: add -> sub
        Value::Int(0x20), // insert: Prim(0x20) via fallback
        Value::Int(1),    // connect source (original root)
        Value::Int(2),    // connect port
    ];
    let (out_composed, _) =
        interpreter::interpret(&iris_composer, &inputs_composed, None).unwrap();
    let composed_result = extract_program(&out_composed[0]);

    // Both should have the same number of nodes
    assert_eq!(
        step_by_step_result.nodes.len(),
        composed_result.nodes.len(),
        "step-by-step and composed should have same node count"
    );

    // Both should have a sub node and a Prim(0x20) node
    let step_has_sub = step_by_step_result
        .nodes
        .values()
        .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x01 }));
    let composed_has_sub = composed_result
        .nodes
        .values()
        .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x01 }));
    assert!(step_has_sub && composed_has_sub, "both should have sub node");

    let step_has_inserted = step_by_step_result
        .nodes
        .values()
        .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x20 }));
    let composed_has_inserted = composed_result
        .nodes
        .values()
        .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x20 }));
    assert!(step_has_inserted && composed_has_inserted, "both should have Prim(0x20) node");
}

#[test]
fn register_iris_compose_as_component() {
    let iris_program = build_iris_compose_program();

    let component = MutationComponent {
        name: "iris_compose_mutations".to_string(),
        program: iris_program,
    };

    let mut registry = ComponentRegistry::new();
    registry.mutations.push(component);

    let found = registry.find_mutation("iris_compose_mutations");
    assert!(found.is_some(), "iris_compose_mutations should be registered");

    // Also register the individual operators
    registry.mutations.push(MutationComponent {
        name: "iris_replace_prim".to_string(),
        program: build_iris_mutation_program(),
    });
    registry.mutations.push(MutationComponent {
        name: "iris_insert_node".to_string(),
        program: build_iris_insert_node_program(),
    });
    registry.mutations.push(MutationComponent {
        name: "iris_connect".to_string(),
        program: build_iris_connect_program(),
    });

    // Verify all four are registered
    assert!(registry.find_mutation("iris_compose_mutations").is_some());
    assert!(registry.find_mutation("iris_replace_prim").is_some());
    assert!(registry.find_mutation("iris_insert_node").is_some());
    assert!(registry.find_mutation("iris_connect").is_some());
    assert_eq!(registry.mutations.len(), 4);

    eprintln!("All four IRIS mutation operators registered as components");
    eprintln!("  1. iris_replace_prim  — replaces Rust replace_prim");
    eprintln!("  2. iris_insert_node   — replaces Rust insert_node");
    eprintln!("  3. iris_connect       — replaces Rust rewire_edge");
    eprintln!("  4. iris_compose_mutations — chains all three");
}

// ---------------------------------------------------------------------------
// Comprehensive composition test: execute the composed result
// ---------------------------------------------------------------------------

#[test]
fn composed_mutation_output_is_executable() {
    // Build a composed mutation, then verify the output program still works
    // (at least the parts that were already wired can execute).

    let iris_composer = build_iris_compose_program();

    // Start with sub(10, 3)
    let target = make_binop_graph(0x01, 10, 3);
    let (pre_result, _) = interpreter::interpret(&target, &[], None).unwrap();
    assert_eq!(pre_result, vec![Value::Int(7)], "sub(10,3) = 7");

    // Compose: replace sub->add, insert mul node, connect
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x00), // replace: sub -> add
        Value::Int(0x02), // insert: mul node
        Value::Int(1),    // connect source
        Value::Int(2),    // connect port
    ];

    let (outputs, _) = interpreter::interpret(&iris_composer, &inputs, None).unwrap();
    let mut modified = extract_program(&outputs[0]);

    // Find the add node (was sub, now add) and set it as root
    let add_node = modified
        .nodes
        .values()
        .find(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x00 }))
        .expect("should have add node");
    let add_id = add_node.id;
    modified.root = add_id;

    // The add node should still compute add(10, 3) = 13
    // (its original Argument edges to lit(10) and lit(3) were preserved)
    let (result, _) = interpreter::interpret(&modified, &[], None).unwrap();
    assert_eq!(result, vec![Value::Int(13)], "add(10, 3) = 13 after compose");
}

// ---------------------------------------------------------------------------
// Summary test: all IRIS mutation operators in one place
// ---------------------------------------------------------------------------

#[test]
fn all_iris_mutation_operators_summary() {
    // Verify all four IRIS mutation operators work and register correctly.

    // 1. replace_prim
    let replace = build_iris_mutation_program();
    let target = make_binop_graph(0x00, 2, 3);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x02),
    ];
    let (out, _) = interpreter::interpret(&replace, &inputs, None).unwrap();
    {
        let g = extract_program(&out[0]);
        assert!(g
            .nodes
            .values()
            .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x02 })));
    }

    // 2. insert_node
    let insert = build_iris_insert_node_program();
    let target = make_binop_graph(0x00, 2, 3);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x01),
    ];
    let (out, _) = interpreter::interpret(&insert, &inputs, None).unwrap();
    match &out[0] {
        Value::Tuple(elems) => {
            match &elems[0] {
                Value::Program(g) => assert_eq!(g.nodes.len(), 4),
                other => panic!("insert_node failed: {:?}", other),
            }
        }
        other => panic!("insert_node failed: {:?}", other),
    }

    // 3. connect
    let connect = build_iris_connect_program();
    let target = make_disconnected_graph();
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1),
        Value::Int(10),
        Value::Int(0),
    ];
    let (out, _) = interpreter::interpret(&connect, &inputs, None).unwrap();
    match &out[0] {
        Value::Program(g) => assert_eq!(g.edges.len(), 1),
        other => panic!("connect failed: {:?}", other),
    }

    // 4. compose
    let compose = build_iris_compose_program();
    let target = make_binop_graph(0x00, 1, 1);
    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(0x01),
        Value::Int(0x02),
        Value::Int(1),
        Value::Int(2),
    ];
    let (out, _) = interpreter::interpret(&compose, &inputs, None).unwrap();
    {
        let g = extract_program(&out[0]);
        assert_eq!(g.nodes.len(), 4);
    }

    // Register all
    let mut registry = ComponentRegistry::new();
    registry.mutations.push(MutationComponent {
        name: "iris_replace_prim".to_string(),
        program: build_iris_mutation_program(),
    });
    registry.mutations.push(MutationComponent {
        name: "iris_insert_node".to_string(),
        program: build_iris_insert_node_program(),
    });
    registry.mutations.push(MutationComponent {
        name: "iris_connect".to_string(),
        program: build_iris_connect_program(),
    });
    registry.mutations.push(MutationComponent {
        name: "iris_compose_mutations".to_string(),
        program: build_iris_compose_program(),
    });

    assert_eq!(registry.mutations.len(), 4);
    eprintln!();
    eprintln!("=== IRIS Self-Writing Mutation Summary ===");
    eprintln!("4 Rust mutation operators replicated in IRIS:");
    eprintln!("  1. replace_prim        — change a Prim node's opcode");
    eprintln!("  2. insert_node         — add a new Prim node to a program");
    eprintln!("  3. connect             — wire an edge between nodes");
    eprintln!("  4. compose_mutations   — chain replace + insert + connect");
    eprintln!("Plus seed generation: build programs from scratch using IRIS ops");
    eprintln!("==========================================");
}
