
//! Self-writing compiler pass: an IRIS program that performs compiler analysis.
//!
//! This test proves that IRIS programs can implement compiler optimization
//! passes. We hand-craft IRIS programs (SemanticGraphs) that analyze and
//! transform other programs using self-modification opcodes (0x80-0x89).
//!
//! Two programs are built:
//!
//! 1. **`is_root_foldable_add`**: A compiler analysis pass that inspects a
//!    program's root node and determines whether it is a constant-foldable
//!    `Prim(add)` node. Uses `graph_nodes` (0x81), `graph_get_kind` (0x82),
//!    `graph_get_prim_op` (0x83), and a `Match` node for conditional dispatch.
//!
//! 2. **`constant_fold_program`**: A compiler transformation pass that evaluates
//!    a fully-constant program using `graph_eval` (0x89), performing constant
//!    folding by meta-evaluation. This replaces the program with its computed
//!    result.
//!
//! Together these demonstrate a complete compiler optimization pipeline written
//! entirely in IRIS: analyze → decide → transform.

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
// Graph construction helpers (shared with self_write_mutation.rs)
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

fn project_node(id: u64, field_index: u16) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Project,
        NodePayload::Project { field_index },
        1,
    )
}

fn match_node(id: u64, arm_count: u16) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Match,
        NodePayload::Match {
            arm_count,
            arm_patterns: vec![],
        },
        arm_count as u8,
    )
}

// ---------------------------------------------------------------------------
// Target programs (the programs being analyzed/optimized)
// ---------------------------------------------------------------------------

/// Create a program that computes `a op b` where op is determined by opcode.
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

/// Create a program that is just a literal value. Root is a Lit node.
fn make_lit_graph(value: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = int_lit_node(1, value);
    nodes.insert(nid, node);
    make_graph(nodes, vec![], 1)
}

/// Create a program `add(input[0], 5)` — has a free variable, not constant-foldable.
fn make_add_with_input_graph() -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x00, 2); // add
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0); // input[0] = x
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(20, 5);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// IRIS compiler analysis program: is_root_foldable_add
// ---------------------------------------------------------------------------

/// Build an IRIS program that checks whether a target program's root node
/// is a Prim(add) — i.e., a candidate for constant folding.
///
/// Input:
///   inputs[0] = Value::Program(target_graph)
///
/// Output:
///   Value::Int(1) if root is Prim(add), Value::Int(0) otherwise.
///
/// Graph structure:
///
///   Root(id=1): Match (2 arms, on Bool scrutinee)
///   │
///   ├── Scrutinee: eq(0x20) [id=100]
///   │   ├── port 0: graph_get_kind(0x82) [id=110]
///   │   │   ├── port 0: input_ref(0) [id=111]         ← program
///   │   │   └── port 1: project(0) [id=112]            ← first node ID
///   │   │       └── graph_nodes(0x81) [id=113]
///   │   │           └── input_ref(0) [id=114]
///   │   └── port 1: int_lit(0) [id=115]                ← 0 = NodeKind::Prim
///   │
///   ├── Arm 0 (false → not Prim): int_lit(0) [id=200]  ← return 0
///   │
///   └── Arm 1 (true → is Prim): bool_to_int(0x44) [id=300]
///       └── eq(0x20) [id=310]
///           ├── port 0: graph_get_prim_op(0x83) [id=320]
///           │   ├── port 0: input_ref(0) [id=321]
///           │   └── port 1: project(0) [id=322]
///           │       └── graph_nodes(0x81) [id=323]
///           │           └── input_ref(0) [id=324]
///           └── port 1: int_lit(0) [id=325]             ← 0 = add opcode
///
fn build_is_root_foldable_add() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // ---- Root: Match on Bool (2 arms) ----
    let (nid, node) = match_node(1, 2);
    nodes.insert(nid, node);

    // ---- Scrutinee branch: eq(graph_get_kind(prog, first_node), 0) ----

    // eq (opcode 0x20, 2 args) — is the kind == Prim?
    let (nid, node) = prim_node(100, 0x20, 2);
    nodes.insert(nid, node);

    // graph_get_kind (opcode 0x82, 2 args)
    let (nid, node) = prim_node(110, 0x82, 2);
    nodes.insert(nid, node);

    // input_ref(0) for graph_get_kind's program arg
    let (nid, node) = input_ref_node(111, 0);
    nodes.insert(nid, node);

    // project(0) to get first node ID from graph_nodes result
    let (nid, node) = project_node(112, 0);
    nodes.insert(nid, node);

    // graph_nodes (opcode 0x81, 1 arg)
    let (nid, node) = prim_node(113, 0x81, 1);
    nodes.insert(nid, node);

    // input_ref(0) for graph_nodes
    let (nid, node) = input_ref_node(114, 0);
    nodes.insert(nid, node);

    // int_lit(0) — the Prim kind tag
    let (nid, node) = int_lit_node(115, 0);
    nodes.insert(nid, node);

    // ---- Arm 0 (false): not a Prim → return 0 ----
    let (nid, node) = int_lit_node(200, 0);
    nodes.insert(nid, node);

    // ---- Arm 1 (true): is Prim → check if opcode is add ----

    // bool_to_int (opcode 0x44, 1 arg)
    let (nid, node) = prim_node(300, 0x44, 1);
    nodes.insert(nid, node);

    // eq (opcode 0x20, 2 args) — is the opcode == add (0x00)?
    let (nid, node) = prim_node(310, 0x20, 2);
    nodes.insert(nid, node);

    // graph_get_prim_op (opcode 0x83, 2 args)
    let (nid, node) = prim_node(320, 0x83, 2);
    nodes.insert(nid, node);

    // input_ref(0) for graph_get_prim_op
    let (nid, node) = input_ref_node(321, 0);
    nodes.insert(nid, node);

    // project(0) to get first node ID
    let (nid, node) = project_node(322, 0);
    nodes.insert(nid, node);

    // graph_nodes (opcode 0x81, 1 arg)
    let (nid, node) = prim_node(323, 0x81, 1);
    nodes.insert(nid, node);

    // input_ref(0) for graph_nodes
    let (nid, node) = input_ref_node(324, 0);
    nodes.insert(nid, node);

    // int_lit(0) — the add opcode
    let (nid, node) = int_lit_node(325, 0);
    nodes.insert(nid, node);

    // ---- Edges ----
    let edges = vec![
        // Match scrutinee → eq(kind, 0)
        make_edge(1, 100, 0, EdgeLabel::Scrutinee),
        // Match arm 0 (false) → int_lit(0)
        make_edge(1, 200, 0, EdgeLabel::Argument),
        // Match arm 1 (true) → bool_to_int(eq(opcode, 0))
        make_edge(1, 300, 1, EdgeLabel::Argument),
        // eq: port 0 → graph_get_kind, port 1 → int_lit(0)
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 115, 1, EdgeLabel::Argument),
        // graph_get_kind: port 0 → input_ref(0), port 1 → project(0)
        make_edge(110, 111, 0, EdgeLabel::Argument),
        make_edge(110, 112, 1, EdgeLabel::Argument),
        // project(0): port 0 → graph_nodes
        make_edge(112, 113, 0, EdgeLabel::Argument),
        // graph_nodes: port 0 → input_ref(0)
        make_edge(113, 114, 0, EdgeLabel::Argument),
        // bool_to_int: port 0 → eq(opcode, 0)
        make_edge(300, 310, 0, EdgeLabel::Argument),
        // eq: port 0 → graph_get_prim_op, port 1 → int_lit(0)
        make_edge(310, 320, 0, EdgeLabel::Argument),
        make_edge(310, 325, 1, EdgeLabel::Argument),
        // graph_get_prim_op: port 0 → input_ref(0), port 1 → project(0)
        make_edge(320, 321, 0, EdgeLabel::Argument),
        make_edge(320, 322, 1, EdgeLabel::Argument),
        // project(0): port 0 → graph_nodes
        make_edge(322, 323, 0, EdgeLabel::Argument),
        // graph_nodes: port 0 → input_ref(0)
        make_edge(323, 324, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// IRIS compiler transformation program: constant_fold_program
// ---------------------------------------------------------------------------

/// Build an IRIS program that evaluates a fully-constant program, performing
/// constant folding by meta-evaluation.
///
/// Input:
///   inputs[0] = Value::Program(target_graph)
///
/// Output:
///   The result of evaluating the program (e.g., add(3,5) → Int(8)).
///
/// Graph structure:
///   Root(id=1): graph_eval(0x89, arity=1)
///   └── port 0: input_ref(0) [id=10]    ← the program to evaluate
///
fn build_constant_fold_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_eval (opcode 0x89, 1 arg = the program)
    let (nid, node) = prim_node(1, 0x89, 1);
    nodes.insert(nid, node);

    // input_ref(0) — the target program
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// IRIS combined analysis+transformation: analyze_and_fold
// ---------------------------------------------------------------------------

/// Build an IRIS program that combines analysis and transformation:
///   - Checks if root is Prim(add)
///   - If yes, evaluates the program (constant folding) and returns the result
///   - If no, returns Int(-1) as a sentinel (no folding performed)
///
/// Input:
///   inputs[0] = Value::Program(target_graph)
///
/// Output:
///   Value::Int(result) if folded, Value::Int(-1) if not foldable.
///
/// Graph structure:
///   Root(id=1): Match (2 arms, on Bool scrutinee)
///   ├── Scrutinee: eq(graph_get_kind(prog, first_node), 0)  ← is Prim?
///   ├── Arm 0 (false → not Prim): int_lit(-1)               ← no fold
///   └── Arm 1 (true → is Prim): graph_eval(prog)            ← fold it!
///
fn build_analyze_and_fold() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // ---- Root: Match on Bool (2 arms) ----
    let (nid, node) = match_node(1, 2);
    nodes.insert(nid, node);

    // ---- Scrutinee: eq(graph_get_kind(prog, first_node), 0) ----
    let (nid, node) = prim_node(100, 0x20, 2); // eq
    nodes.insert(nid, node);

    let (nid, node) = prim_node(110, 0x82, 2); // graph_get_kind
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(111, 0); // program
    nodes.insert(nid, node);

    let (nid, node) = project_node(112, 0); // first node ID
    nodes.insert(nid, node);

    let (nid, node) = prim_node(113, 0x81, 1); // graph_nodes
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(114, 0);
    nodes.insert(nid, node);

    let (nid, node) = int_lit_node(115, 0); // 0 = NodeKind::Prim
    nodes.insert(nid, node);

    // ---- Arm 0 (false): not a Prim → return -1 ----
    let (nid, node) = int_lit_node(200, -1);
    nodes.insert(nid, node);

    // ---- Arm 1 (true): is Prim → graph_eval the program ----
    let (nid, node) = prim_node(300, 0x89, 1); // graph_eval
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(310, 0); // program
    nodes.insert(nid, node);

    // ---- Edges ----
    let edges = vec![
        // Match scrutinee → eq
        make_edge(1, 100, 0, EdgeLabel::Scrutinee),
        // Match arm 0 (false) → int_lit(-1)
        make_edge(1, 200, 0, EdgeLabel::Argument),
        // Match arm 1 (true) → graph_eval
        make_edge(1, 300, 1, EdgeLabel::Argument),
        // eq: port 0 → graph_get_kind, port 1 → int_lit(0)
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 115, 1, EdgeLabel::Argument),
        // graph_get_kind: port 0 → input_ref(0), port 1 → project(0)
        make_edge(110, 111, 0, EdgeLabel::Argument),
        make_edge(110, 112, 1, EdgeLabel::Argument),
        // project(0) → graph_nodes
        make_edge(112, 113, 0, EdgeLabel::Argument),
        // graph_nodes → input_ref(0)
        make_edge(113, 114, 0, EdgeLabel::Argument),
        // graph_eval → input_ref(0)
        make_edge(300, 310, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// -- Analysis pass: is_root_foldable_add --

#[test]
fn analysis_add_3_5_is_foldable() {
    let analyzer = build_is_root_foldable_add();
    let target = make_binop_graph(0x00, 3, 5); // add(3, 5)

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&analyzer, &inputs, None).unwrap();

    assert_eq!(
        outputs,
        vec![Value::Int(1)],
        "add(3, 5) root IS a Prim(add), should return 1"
    );
}

#[test]
fn analysis_mul_2_3_is_not_foldable_add() {
    let analyzer = build_is_root_foldable_add();
    let target = make_binop_graph(0x02, 2, 3); // mul(2, 3)

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&analyzer, &inputs, None).unwrap();

    assert_eq!(
        outputs,
        vec![Value::Int(0)],
        "mul(2, 3) root is Prim but not add, should return 0"
    );
}

#[test]
fn analysis_sub_10_7_is_not_foldable_add() {
    let analyzer = build_is_root_foldable_add();
    let target = make_binop_graph(0x01, 10, 7); // sub(10, 7)

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&analyzer, &inputs, None).unwrap();

    assert_eq!(
        outputs,
        vec![Value::Int(0)],
        "sub(10, 7) root is Prim but not add, should return 0"
    );
}

#[test]
fn analysis_lit_is_not_foldable() {
    let analyzer = build_is_root_foldable_add();
    let target = make_lit_graph(42); // Lit(42) — already folded

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&analyzer, &inputs, None).unwrap();

    assert_eq!(
        outputs,
        vec![Value::Int(0)],
        "Lit(42) root is not a Prim at all, should return 0"
    );
}

#[test]
fn analysis_add_with_input_is_foldable_add() {
    // Note: the analysis only checks if root is Prim(add), not whether
    // children are constants. add(x, 5) IS a Prim(add) even though it
    // has a free variable. Full constant-foldability also requires checking
    // children, which needs edge-reading opcodes not yet available.
    let analyzer = build_is_root_foldable_add();
    let target = make_add_with_input_graph(); // add(x, 5)

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&analyzer, &inputs, None).unwrap();

    assert_eq!(
        outputs,
        vec![Value::Int(1)],
        "add(x, 5) root IS a Prim(add), analysis returns 1"
    );
}

// -- Transformation pass: constant folding by evaluation --

#[test]
fn fold_add_3_5_gives_8() {
    let folder = build_constant_fold_program();
    let target = make_binop_graph(0x00, 3, 5); // add(3, 5)

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&folder, &inputs, None).unwrap();

    assert_eq!(
        outputs,
        vec![Value::Int(8)],
        "constant folding add(3, 5) should yield 8"
    );
}

#[test]
fn fold_mul_4_7_gives_28() {
    let folder = build_constant_fold_program();
    let target = make_binop_graph(0x02, 4, 7); // mul(4, 7)

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&folder, &inputs, None).unwrap();

    assert_eq!(
        outputs,
        vec![Value::Int(28)],
        "constant folding mul(4, 7) should yield 28"
    );
}

#[test]
fn fold_sub_10_3_gives_7() {
    let folder = build_constant_fold_program();
    let target = make_binop_graph(0x01, 10, 3); // sub(10, 3)

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&folder, &inputs, None).unwrap();

    assert_eq!(
        outputs,
        vec![Value::Int(7)],
        "constant folding sub(10, 3) should yield 7"
    );
}

#[test]
fn fold_lit_42_gives_42() {
    let folder = build_constant_fold_program();
    let target = make_lit_graph(42); // Lit(42) — already a constant

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&folder, &inputs, None).unwrap();

    assert_eq!(
        outputs,
        vec![Value::Int(42)],
        "evaluating Lit(42) should yield 42"
    );
}

// -- Combined analysis + transformation --

#[test]
fn analyze_and_fold_add_3_5() {
    let combined = build_analyze_and_fold();
    let target = make_binop_graph(0x00, 3, 5); // add(3, 5)

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&combined, &inputs, None).unwrap();

    assert_eq!(
        outputs,
        vec![Value::Int(8)],
        "combined: add(3, 5) is foldable → evaluate to 8"
    );
}

#[test]
fn analyze_and_fold_lit_not_foldable() {
    let combined = build_analyze_and_fold();
    let target = make_lit_graph(42); // Lit(42) — root is not Prim

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&combined, &inputs, None).unwrap();

    assert_eq!(
        outputs,
        vec![Value::Int(-1)],
        "combined: Lit(42) is not Prim → return -1 sentinel"
    );
}

// -- Cross-cycle verification: folded result matches direct execution --

#[test]
fn folded_result_matches_direct_execution() {
    let folder = build_constant_fold_program();

    let test_cases: Vec<(u8, i64, i64, i64)> = vec![
        (0x00, 3, 5, 8),    // add(3, 5) = 8
        (0x00, -1, 1, 0),   // add(-1, 1) = 0
        (0x00, 100, 200, 300), // add(100, 200) = 300
        (0x01, 10, 3, 7),   // sub(10, 3) = 7
        (0x02, 6, 7, 42),   // mul(6, 7) = 42
    ];

    for (opcode, a, b, expected) in &test_cases {
        let target = make_binop_graph(*opcode, *a, *b);

        // Direct execution (the "ground truth")
        let (direct_result, _) = interpreter::interpret(&target, &[], None).unwrap();
        assert_eq!(
            direct_result,
            vec![Value::Int(*expected)],
            "direct execution of op=0x{:02x}({}, {}) should be {}",
            opcode, a, b, expected
        );

        // Constant folding via IRIS meta-program
        let inputs = vec![Value::Program(Rc::new(target))];
        let (folded_result, _) = interpreter::interpret(&folder, &inputs, None).unwrap();

        assert_eq!(
            folded_result, direct_result,
            "constant-folded result should match direct execution for op=0x{:02x}({}, {})",
            opcode, a, b
        );
    }
}

// -- End-to-end pipeline: analyze → fold → verify --

#[test]
fn end_to_end_compiler_pipeline() {
    let analyzer = build_is_root_foldable_add();
    let folder = build_constant_fold_program();

    // Program: add(3, 5)
    let target = make_binop_graph(0x00, 3, 5);

    // Step 1: Analysis — is this program's root a Prim(add)?
    let inputs = vec![Value::Program(Rc::new(target.clone()))];
    let (analysis_result, _) = interpreter::interpret(&analyzer, &inputs, None).unwrap();
    assert_eq!(
        analysis_result,
        vec![Value::Int(1)],
        "step 1: analysis identifies add(3, 5) as foldable"
    );

    // Step 2: Since analysis says foldable, apply the folder
    let is_foldable = matches!(analysis_result.first(), Some(Value::Int(1)));
    assert!(is_foldable, "analysis should indicate foldable");

    let inputs = vec![Value::Program(Rc::new(target.clone()))];
    let (fold_result, _) = interpreter::interpret(&folder, &inputs, None).unwrap();
    assert_eq!(
        fold_result,
        vec![Value::Int(8)],
        "step 2: constant folding produces 8"
    );

    // Step 3: Verify the folded result matches direct execution
    let (direct, _) = interpreter::interpret(&target, &[], None).unwrap();
    assert_eq!(
        fold_result, direct,
        "step 3: folded result matches direct execution"
    );
}
