
//! Self-writing seed generation: IRIS programs that construct other programs.
//!
//! This test proves that IRIS programs can generate complete programs from
//! scratch using self-modification opcodes. Two seed generators are built:
//!
//! 1. A fold seed generator that constructs `fold(0, add, input)` — a program
//!    that sums a list of integers.
//! 2. An add seed generator that constructs `add(a, b)` — a program that adds
//!    two integer inputs.
//!
//! Both generators use:
//!   - self_graph (0x80) to capture their own graph (which contains embedded
//!     template nodes for the target program, without edges between them).
//!   - graph_replace_subtree (0x88) to re-root the captured graph onto the
//!     template root node.
//!   - graph_connect (0x86) to wire up edges between the template nodes.
//!
//! The result is a Value::Program with the correct structure and edges.

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Graph construction helpers (same pattern as self_write_mutation.rs)
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

fn fold_node(id: u64, arity: u8) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![],
        },
        arity,
    )
}

// ---------------------------------------------------------------------------
// IRIS Fold Seed Generator
// ---------------------------------------------------------------------------

/// Build an IRIS program that generates a `fold(0, add, input)` program.
///
/// The generator embeds the target fold program's nodes (without edges)
/// in its own graph. When executed, it:
///
/// 1. Calls self_graph (0x80) to get the full graph including template nodes.
/// 2. Uses graph_replace_subtree (0x88) to re-root onto the Fold template.
/// 3. Chains three graph_connect (0x86) calls to wire the Fold node's edges.
///
/// Graph structure:
///
///   Builder nodes (active execution path, deeply nested):
///
///   Root(id=1): graph_connect (0x86) — add edge: fold→input_ref, port 2
///   ├─0: id=2: graph_connect (0x86) — add edge: fold→add, port 1
///   │   ├─0: id=3: graph_connect (0x86) — add edge: fold→lit(0), port 0
///   │   │   ├─0: id=4: graph_replace_subtree (0x88) — re-root to fold node
///   │   │   │   ├─0: id=5: self_graph (0x80)
///   │   │   │   ├─1: id=6: lit(1)   — builder root id
///   │   │   │   ├─2: id=7: self_graph (0x80)
///   │   │   │   └─3: id=8: lit(1000) — fold template id
///   │   │   ├─1: id=9:  lit(1000)  — source node for edge
///   │   │   ├─2: id=11: lit(1010)  — target node for edge
///   │   │   └─3: id=12: lit(0)     — port number 0
///   │   ├─1: id=13: lit(1000)
///   │   ├─2: id=14: lit(1020)
///   │   └─3: id=15: lit(1)         — port number 1
///   ├─1: id=16: lit(1000)
///   ├─2: id=17: lit(1030)
///   └─3: id=18: lit(2)             — port number 2
///
///   Template nodes (no edges, embedded in the graph for extraction):
///     id=1000: Fold (arity=3)
///     id=1010: Lit(0)           — base case
///     id=1020: Prim(add=0x00)   — step function
///     id=1030: input_ref(0)     — collection input
fn build_fold_seed_generator() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // --- Builder nodes ---

    // Outermost: graph_connect (fold→input_ref, port 2)
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);

    // Middle: graph_connect (fold→add, port 1)
    let (nid, node) = prim_node(2, 0x86, 4);
    nodes.insert(nid, node);

    // Inner: graph_connect (fold→lit(0), port 0)
    let (nid, node) = prim_node(3, 0x86, 4);
    nodes.insert(nid, node);

    // Core: graph_replace_subtree
    let (nid, node) = prim_node(4, 0x88, 4);
    nodes.insert(nid, node);

    // self_graph calls
    let (nid, node) = prim_node(5, 0x80, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(7, 0x80, 0);
    nodes.insert(nid, node);

    // Literal arguments for replace_subtree
    let (nid, node) = int_lit_node(6, 1);     // builder root id
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(8, 1000);   // fold template id
    nodes.insert(nid, node);

    // Arguments for inner graph_connect (port 0: fold→lit(0))
    let (nid, node) = int_lit_node(9, 1000);   // source: fold node
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(11, 1010);  // target: lit(0) node
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(12, 0);     // port 0
    nodes.insert(nid, node);

    // Arguments for middle graph_connect (port 1: fold→add)
    let (nid, node) = int_lit_node(13, 1000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(14, 1020);  // target: add node
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(15, 1);     // port 1
    nodes.insert(nid, node);

    // Arguments for outer graph_connect (port 2: fold→input_ref)
    let (nid, node) = int_lit_node(16, 1000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(17, 1030);  // target: input_ref node
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(18, 2);     // port 2
    nodes.insert(nid, node);

    // --- Template nodes (no edges between them) ---

    let (nid, node) = fold_node(1000, 3);
    nodes.insert(nid, node);

    let (nid, node) = int_lit_node(1010, 0);   // base case: 0
    nodes.insert(nid, node);

    let (nid, node) = prim_node(1020, 0x00, 2); // step: add
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(1030, 0); // collection input
    nodes.insert(nid, node);

    // --- Edges (builder wiring only; no template edges) ---

    let edges = vec![
        // Root (id=1): graph_connect args
        make_edge(1, 2, 0, EdgeLabel::Argument),    // program (from middle connect)
        make_edge(1, 16, 1, EdgeLabel::Argument),   // source: fold
        make_edge(1, 17, 2, EdgeLabel::Argument),   // target: input_ref
        make_edge(1, 18, 3, EdgeLabel::Argument),   // port: 2

        // Middle (id=2): graph_connect args
        make_edge(2, 3, 0, EdgeLabel::Argument),    // program (from inner connect)
        make_edge(2, 13, 1, EdgeLabel::Argument),   // source: fold
        make_edge(2, 14, 2, EdgeLabel::Argument),   // target: add
        make_edge(2, 15, 3, EdgeLabel::Argument),   // port: 1

        // Inner (id=3): graph_connect args
        make_edge(3, 4, 0, EdgeLabel::Argument),    // program (from replace_subtree)
        make_edge(3, 9, 1, EdgeLabel::Argument),    // source: fold
        make_edge(3, 11, 2, EdgeLabel::Argument),   // target: lit(0)
        make_edge(3, 12, 3, EdgeLabel::Argument),   // port: 0

        // Core (id=4): replace_subtree args
        make_edge(4, 5, 0, EdgeLabel::Argument),    // target program (self_graph)
        make_edge(4, 6, 1, EdgeLabel::Argument),    // target node: 1 (root)
        make_edge(4, 7, 2, EdgeLabel::Argument),    // source program (self_graph)
        make_edge(4, 8, 3, EdgeLabel::Argument),    // source node: 1000 (fold)
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// IRIS Add Seed Generator
// ---------------------------------------------------------------------------

/// Build an IRIS program that generates an `add(a, b)` program.
///
/// Same strategy as the fold generator: template nodes are embedded without
/// edges, self_graph + replace_subtree re-roots, then graph_connect wires.
///
/// Builder structure:
///
///   Root(id=1): graph_connect (0x86) — add edge: add→input_ref(1), port 1
///   ├─0: id=2: graph_connect (0x86) — add edge: add→input_ref(0), port 0
///   │   ├─0: id=3: graph_replace_subtree (0x88) — re-root to add node
///   │   │   ├─0: id=4: self_graph (0x80)
///   │   │   ├─1: id=5: lit(1)
///   │   │   ├─2: id=6: self_graph (0x80)
///   │   │   └─3: id=7: lit(2000)
///   │   ├─1: id=8:  lit(2000) — source: add node
///   │   ├─2: id=9:  lit(2010) — target: input_ref(0)
///   │   └─3: id=10: lit(0)    — port 0
///   ├─1: id=11: lit(2000)
///   ├─2: id=12: lit(2020) — target: input_ref(1)
///   └─3: id=13: lit(1)    — port 1
///
///   Template nodes:
///     id=2000: Prim(add=0x00, arity=2)
///     id=2010: input_ref(0)
///     id=2020: input_ref(1)
fn build_add_seed_generator() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // --- Builder nodes ---

    // Outer: graph_connect (add→input_ref(1), port 1)
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);

    // Inner: graph_connect (add→input_ref(0), port 0)
    let (nid, node) = prim_node(2, 0x86, 4);
    nodes.insert(nid, node);

    // Core: replace_subtree
    let (nid, node) = prim_node(3, 0x88, 4);
    nodes.insert(nid, node);

    // self_graph calls
    let (nid, node) = prim_node(4, 0x80, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(6, 0x80, 0);
    nodes.insert(nid, node);

    // Literal args for replace_subtree
    let (nid, node) = int_lit_node(5, 1);     // builder root id
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(7, 2000);   // add template id
    nodes.insert(nid, node);

    // Args for inner graph_connect (port 0: add→input_ref(0))
    let (nid, node) = int_lit_node(8, 2000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(9, 2010);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(10, 0);     // port 0
    nodes.insert(nid, node);

    // Args for outer graph_connect (port 1: add→input_ref(1))
    let (nid, node) = int_lit_node(11, 2000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(12, 2020);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(13, 1);     // port 1
    nodes.insert(nid, node);

    // --- Template nodes (no edges) ---

    let (nid, node) = prim_node(2000, 0x00, 2);
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(2010, 0);
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(2020, 1);
    nodes.insert(nid, node);

    // --- Edges (builder only) ---

    let edges = vec![
        // Root (id=1): outer graph_connect
        make_edge(1, 2, 0, EdgeLabel::Argument),    // program (from inner connect)
        make_edge(1, 11, 1, EdgeLabel::Argument),   // source: add node
        make_edge(1, 12, 2, EdgeLabel::Argument),   // target: input_ref(1)
        make_edge(1, 13, 3, EdgeLabel::Argument),   // port: 1

        // Inner (id=2): graph_connect
        make_edge(2, 3, 0, EdgeLabel::Argument),    // program (from replace_subtree)
        make_edge(2, 8, 1, EdgeLabel::Argument),    // source: add
        make_edge(2, 9, 2, EdgeLabel::Argument),    // target: input_ref(0)
        make_edge(2, 10, 3, EdgeLabel::Argument),   // port: 0

        // Core (id=3): replace_subtree
        make_edge(3, 4, 0, EdgeLabel::Argument),    // target program (self_graph)
        make_edge(3, 5, 1, EdgeLabel::Argument),    // target node: 1 (root)
        make_edge(3, 6, 2, EdgeLabel::Argument),    // source program (self_graph)
        make_edge(3, 7, 3, EdgeLabel::Argument),    // source node: 2000 (add)
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Test 1: Fold seed generator produces a working fold(0, add, input) program
// ---------------------------------------------------------------------------

#[test]
fn fold_seed_generator_produces_sum_program() {
    let generator = build_fold_seed_generator();

    // Execute the generator — it takes no inputs and returns a Program.
    let (outputs, _) = interpreter::interpret(&generator, &[], None).unwrap();
    assert_eq!(outputs.len(), 1, "generator should return exactly one value");

    let fold_program = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // Verify the generated program has a Fold root.
    let root_node = fold_program
        .nodes
        .get(&fold_program.root)
        .expect("root node should exist");
    assert_eq!(
        root_node.kind,
        NodeKind::Fold,
        "generated program root should be a Fold node"
    );

    // Execute the generated fold program: fold(0, add, [1, 2, 3]) = 6
    let input_list = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let (result, _) = interpreter::interpret(&fold_program, &[input_list], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(6)],
        "fold(0, add, [1, 2, 3]) should be 6"
    );
}

#[test]
fn fold_seed_generator_handles_empty_list() {
    let generator = build_fold_seed_generator();
    let (outputs, _) = interpreter::interpret(&generator, &[], None).unwrap();

    let fold_program = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // fold(0, add, []) = 0 (base case)
    let input_list = Value::tuple(vec![]);
    let (result, _) = interpreter::interpret(&fold_program, &[input_list], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(0)],
        "fold(0, add, []) should be 0"
    );
}

#[test]
fn fold_seed_generator_handles_single_element() {
    let generator = build_fold_seed_generator();
    let (outputs, _) = interpreter::interpret(&generator, &[], None).unwrap();

    let fold_program = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // fold(0, add, [42]) = 42
    let input_list = Value::tuple(vec![Value::Int(42)]);
    let (result, _) = interpreter::interpret(&fold_program, &[input_list], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(42)],
        "fold(0, add, [42]) should be 42"
    );
}

#[test]
fn fold_seed_generator_sums_negative_numbers() {
    let generator = build_fold_seed_generator();
    let (outputs, _) = interpreter::interpret(&generator, &[], None).unwrap();

    let fold_program = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // fold(0, add, [-1, 2, -3, 4]) = 2
    let input_list = Value::tuple(vec![
        Value::Int(-1),
        Value::Int(2),
        Value::Int(-3),
        Value::Int(4),
    ]);
    let (result, _) = interpreter::interpret(&fold_program, &[input_list], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(2)],
        "fold(0, add, [-1, 2, -3, 4]) should be 2"
    );
}

// ---------------------------------------------------------------------------
// Test 2: Add seed generator produces a working add(a, b) program
// ---------------------------------------------------------------------------

#[test]
fn add_seed_generator_produces_add_program() {
    let generator = build_add_seed_generator();

    // Execute the generator — returns a Program.
    let (outputs, _) = interpreter::interpret(&generator, &[], None).unwrap();
    assert_eq!(outputs.len(), 1, "generator should return exactly one value");

    let add_program = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // Verify the generated program has a Prim root with add opcode.
    let root_node = add_program
        .nodes
        .get(&add_program.root)
        .expect("root node should exist");
    assert_eq!(
        root_node.kind,
        NodeKind::Prim,
        "generated program root should be a Prim node"
    );
    assert_eq!(
        root_node.payload,
        NodePayload::Prim { opcode: 0x00 },
        "generated program root should be add (0x00)"
    );

    // Execute the generated add program: add(3, 5) = 8
    let (result, _) =
        interpreter::interpret(&add_program, &[Value::Int(3), Value::Int(5)], None).unwrap();
    assert_eq!(result, vec![Value::Int(8)], "add(3, 5) should be 8");
}

#[test]
fn add_seed_generator_handles_zero() {
    let generator = build_add_seed_generator();
    let (outputs, _) = interpreter::interpret(&generator, &[], None).unwrap();

    let add_program = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    let (result, _) =
        interpreter::interpret(&add_program, &[Value::Int(0), Value::Int(0)], None).unwrap();
    assert_eq!(result, vec![Value::Int(0)], "add(0, 0) should be 0");
}

#[test]
fn add_seed_generator_handles_negatives() {
    let generator = build_add_seed_generator();
    let (outputs, _) = interpreter::interpret(&generator, &[], None).unwrap();

    let add_program = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    let (result, _) =
        interpreter::interpret(&add_program, &[Value::Int(-10), Value::Int(7)], None).unwrap();
    assert_eq!(result, vec![Value::Int(-3)], "add(-10, 7) should be -3");
}

// ---------------------------------------------------------------------------
// Test 3: Generated programs can be evaluated via graph_eval (0x89)
// ---------------------------------------------------------------------------

#[test]
fn generated_fold_program_runs_via_graph_eval() {
    // Step 1: Generate the fold program.
    let generator = build_fold_seed_generator();
    let (outputs, _) = interpreter::interpret(&generator, &[], None).unwrap();
    let fold_program = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // Step 2: Build an IRIS program that calls graph_eval on the fold program.
    //   Root: graph_eval (0x89, arity=2)
    //   ├── port 0: input_ref(0) — the fold program
    //   └── port 1: input_ref(1) — the list to sum
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x89, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];

    let eval_program = make_graph(nodes, edges, 1);

    // Execute: graph_eval(fold_program, Tuple(Tuple([1,2,3,4,5])))
    // The outer Tuple gets unpacked by graph_eval into inputs for the fold
    // program, so we wrap the list in an extra Tuple layer.
    let list = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4), Value::Int(5)]);
    let wrapped = Value::tuple(vec![list]);
    let inputs = vec![Value::Program(Rc::new(fold_program)), wrapped];
    let (result, _) = interpreter::interpret(&eval_program, &inputs, None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(15)],
        "graph_eval(fold_sum, [1,2,3,4,5]) should be 15"
    );
}

#[test]
fn generated_add_program_runs_via_graph_eval() {
    // Generate the add program.
    let generator = build_add_seed_generator();
    let (outputs, _) = interpreter::interpret(&generator, &[], None).unwrap();
    let add_program = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    // Build evaluator: graph_eval(program, Tuple(a, b))
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x89, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];

    let eval_program = make_graph(nodes, edges, 1);

    // graph_eval passes second arg as inputs; Tuple gets unpacked.
    let inputs = vec![
        Value::Program(Rc::new(add_program)),
        Value::tuple(vec![Value::Int(100), Value::Int(200)]),
    ];
    let (result, _) = interpreter::interpret(&eval_program, &inputs, None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(300)],
        "graph_eval(add_program, (100, 200)) should be 300"
    );
}

// ---------------------------------------------------------------------------
// Test 4: Structural verification of generated programs
// ---------------------------------------------------------------------------

#[test]
fn fold_program_has_correct_structure() {
    let generator = build_fold_seed_generator();
    let (outputs, _) = interpreter::interpret(&generator, &[], None).unwrap();

    let fold_program = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    let root = fold_program.root;

    // Collect reachable nodes via BFS from root.
    let mut reachable = std::collections::BTreeSet::new();
    let mut worklist = vec![root];
    while let Some(nid) = worklist.pop() {
        if reachable.insert(nid) {
            for edge in &fold_program.edges {
                if edge.source == nid && !reachable.contains(&edge.target) {
                    worklist.push(edge.target);
                }
            }
        }
    }

    // Count node kinds in the reachable set.
    let mut fold_count = 0;
    let mut lit_count = 0;
    let mut prim_count = 0;

    for nid in &reachable {
        if let Some(node) = fold_program.nodes.get(nid) {
            match node.kind {
                NodeKind::Fold => fold_count += 1,
                NodeKind::Lit => lit_count += 1,
                NodeKind::Prim => prim_count += 1,
                other => panic!("unexpected node kind in fold program: {:?}", other),
            }
        }
    }

    assert_eq!(fold_count, 1, "should have exactly 1 Fold node");
    assert_eq!(prim_count, 1, "should have exactly 1 Prim node (add)");
    assert_eq!(
        lit_count, 2,
        "should have exactly 2 Lit nodes (base + input_ref)"
    );

    // Verify edges from fold node.
    let fold_edges: Vec<&Edge> = fold_program
        .edges
        .iter()
        .filter(|e| e.source == root)
        .collect();
    assert_eq!(
        fold_edges.len(),
        3,
        "Fold node should have 3 outgoing edges"
    );

    // Ports should be 0, 1, 2.
    let mut ports: Vec<u8> = fold_edges.iter().map(|e| e.port).collect();
    ports.sort();
    assert_eq!(ports, vec![0, 1, 2], "Fold edges should use ports 0, 1, 2");
}

#[test]
fn add_program_has_correct_structure() {
    let generator = build_add_seed_generator();
    let (outputs, _) = interpreter::interpret(&generator, &[], None).unwrap();

    let add_program = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    let root = add_program.root;
    let root_node = add_program.nodes.get(&root).expect("root should exist");

    assert_eq!(root_node.kind, NodeKind::Prim);
    assert_eq!(root_node.payload, NodePayload::Prim { opcode: 0x00 });

    // Collect reachable nodes.
    let mut reachable = std::collections::BTreeSet::new();
    let mut worklist = vec![root];
    while let Some(nid) = worklist.pop() {
        if reachable.insert(nid) {
            for edge in &add_program.edges {
                if edge.source == nid && !reachable.contains(&edge.target) {
                    worklist.push(edge.target);
                }
            }
        }
    }

    assert_eq!(
        reachable.len(),
        3,
        "add program should have 3 reachable nodes"
    );

    let add_edges: Vec<&Edge> = add_program
        .edges
        .iter()
        .filter(|e| e.source == root)
        .collect();
    assert_eq!(
        add_edges.len(),
        2,
        "add node should have 2 outgoing edges"
    );

    let mut ports: Vec<u8> = add_edges.iter().map(|e| e.port).collect();
    ports.sort();
    assert_eq!(ports, vec![0, 1], "add edges should use ports 0, 1");
}
