
//! IRIS fitness evaluator: programs that evaluate other programs.
//!
//! This test builds IRIS programs (SemanticGraphs) that replace part of
//! evaluator.rs with IRIS code. The single-case evaluator takes a program,
//! test inputs, and an expected output, runs the program via graph_eval
//! (0x89), compares the result, and returns 1 or 0. The multi-case
//! evaluator folds over a list of test cases, calling the single-case
//! logic for each, and returns the count of passed tests.

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

// ---------------------------------------------------------------------------
// Target program: add(input0, input1)
// ---------------------------------------------------------------------------

/// Build a program that computes add(input[0], input[1]).
/// Uses input_ref nodes so it takes dynamic inputs.
fn make_add_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: add (opcode 0x00, arity 2)
    let (nid, node) = prim_node(1, 0x00, 2);
    nodes.insert(nid, node);

    // input_ref(0)
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);

    // input_ref(1)
    let (nid, node) = input_ref_node(20, 1);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Single-case fitness evaluator (IRIS program)
// ---------------------------------------------------------------------------
//
// Takes 3 inputs:
//   inputs[0] = Value::Program(target_program)
//   inputs[1] = Value::Tuple(test_inputs)
//   inputs[2] = Value::Int(expected_output)
//
// Graph structure:
//
//   Root(id=1): bool_to_int(0x44, arity=1)
//   └── port 0: eq(0x20, arity=2)                       [id=10]
//        ├── port 0: graph_eval(0x89, arity=2)           [id=20]
//        │    ├── port 0: input_ref(0) → Program         [id=30]
//        │    └── port 1: input_ref(1) → Tuple(inputs)   [id=40]
//        └── port 1: input_ref(2) → Int(expected)        [id=50]
//
// Returns Int(1) if program(inputs) == expected, Int(0) otherwise.

fn build_single_case_evaluator() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: bool_to_int (0x44, arity 1)
    let (nid, node) = prim_node(1, 0x44, 1);
    nodes.insert(nid, node);

    // eq (0x20, arity 2)
    let (nid, node) = prim_node(10, 0x20, 2);
    nodes.insert(nid, node);

    // graph_eval (0x89, arity 2)
    let (nid, node) = prim_node(20, 0x89, 2);
    nodes.insert(nid, node);

    // input_ref(0) — the Program to evaluate
    let (nid, node) = input_ref_node(30, 0);
    nodes.insert(nid, node);

    // input_ref(1) — the test inputs (Tuple)
    let (nid, node) = input_ref_node(40, 1);
    nodes.insert(nid, node);

    // input_ref(2) — the expected output
    let (nid, node) = input_ref_node(50, 2);
    nodes.insert(nid, node);

    let edges = vec![
        // bool_to_int → eq
        make_edge(1, 10, 0, EdgeLabel::Argument),
        // eq → graph_eval (left operand)
        make_edge(10, 20, 0, EdgeLabel::Argument),
        // eq → expected (right operand)
        make_edge(10, 50, 1, EdgeLabel::Argument),
        // graph_eval → program
        make_edge(20, 30, 0, EdgeLabel::Argument),
        // graph_eval → inputs
        make_edge(20, 40, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Multi-case fitness evaluator (IRIS program)
// ---------------------------------------------------------------------------
//
// Takes 2 inputs:
//   inputs[0] = Value::Program(target_program)
//   inputs[1] = Value::Tuple(test_cases)
//                where each test_case = Tuple(Tuple(inputs...), Int(expected))
//
// Uses Fold with a Lambda step function:
//
//   Root(id=1): Fold(mode=0x00)
//   ├── port 0: Lit(Int(0))                              → base accumulator  [id=10]
//   ├── port 1: Lambda(binder=0xFFFF_0002)                → step function     [id=20]
//   │   └── body: add(0x00, arity=2)                                          [id=100]
//   │        ├── port 0: Project(0) from input_ref(2)     → acc               [id=110, 115]
//   │        └── port 1: bool_to_int(0x44)                                    [id=120]
//   │             └── eq(0x20)                                                [id=130]
//   │                  ├── graph_eval(0x89)                                   [id=140]
//   │                  │    ├── input_ref(0)               → program          [id=150]
//   │                  │    └── Project(0) from
//   │                  │         Project(1) from input_ref(2) → tc inputs     [id=160,170,175]
//   │                  └── Project(1) from
//   │                       Project(1) from input_ref(2)  → tc expected       [id=180,190,195]
//   └── port 2: input_ref(1)                              → test_cases        [id=30]
//
// The Lambda's binder is BinderId(0xFFFF_0002), so inside the body
// input_ref(2) resolves to Tuple(acc, test_case).
// Project(0) from that = acc (Int).
// Project(1) from that = test_case = Tuple(Tuple(inputs...), Int(expected)).
// Project(0) from test_case = Tuple(inputs...).
// Project(1) from test_case = Int(expected).

fn build_multi_case_evaluator() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // --- Outer structure ---

    // Root: Fold (mode 0x00, arity 3)
    let (nid, node) = make_node(
        1,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![0x00],
        },
        3,
    );
    nodes.insert(nid, node);

    // Port 0: base accumulator = Int(0)
    let (nid, node) = int_lit_node(10, 0);
    nodes.insert(nid, node);

    // Port 1: Lambda step function
    // Binder = 0xFFFF_0002 so input_ref(2) references it in the body.
    let (nid, node) = make_node(
        20,
        NodeKind::Lambda,
        NodePayload::Lambda {
            binder: iris_types::graph::BinderId(0xFFFF_0002),
            captured_count: 0,
        },
        0, // Lambda has no argument edges; body is via Continuation
    );
    nodes.insert(nid, node);

    // Port 2: the test_cases collection = input_ref(1)
    let (nid, node) = input_ref_node(30, 1);
    nodes.insert(nid, node);

    // --- Lambda body ---

    // add(acc, score) — the step: acc + bool_to_int(eq(graph_eval(...), expected))
    // id=100: add (0x00, arity 2)
    let (nid, node) = prim_node(100, 0x00, 2);
    nodes.insert(nid, node);

    // id=110: Project(0) from input_ref(2) → extracts acc from Tuple(acc, elem)
    let (nid, node) = project_node(110, 0);
    nodes.insert(nid, node);

    // id=115: input_ref(2) — the Lambda-bound Tuple(acc, test_case)
    let (nid, node) = input_ref_node(115, 2);
    nodes.insert(nid, node);

    // id=120: bool_to_int (0x44, arity 1)
    let (nid, node) = prim_node(120, 0x44, 1);
    nodes.insert(nid, node);

    // id=130: eq (0x20, arity 2)
    let (nid, node) = prim_node(130, 0x20, 2);
    nodes.insert(nid, node);

    // id=140: graph_eval (0x89, arity 2)
    let (nid, node) = prim_node(140, 0x89, 2);
    nodes.insert(nid, node);

    // id=150: input_ref(0) — the Program (from captured outer env)
    let (nid, node) = input_ref_node(150, 0);
    nodes.insert(nid, node);

    // id=160: Project(0) from test_case — extracts Tuple(inputs)
    let (nid, node) = project_node(160, 0);
    nodes.insert(nid, node);

    // id=170: Project(1) from input_ref(2) — extracts test_case from Tuple(acc, test_case)
    let (nid, node) = project_node(170, 1);
    nodes.insert(nid, node);

    // id=175: input_ref(2) — another reference to Lambda-bound var
    let (nid, node) = input_ref_node(175, 2);
    nodes.insert(nid, node);

    // id=180: Project(1) from test_case — extracts expected
    let (nid, node) = project_node(180, 1);
    nodes.insert(nid, node);

    // id=190: Project(1) from input_ref(2) — extracts test_case from Tuple(acc, test_case)
    let (nid, node) = project_node(190, 1);
    nodes.insert(nid, node);

    // id=195: input_ref(2) — another reference to Lambda-bound var
    let (nid, node) = input_ref_node(195, 2);
    nodes.insert(nid, node);

    // --- Edges ---

    let edges = vec![
        // Fold ports
        make_edge(1, 10, 0, EdgeLabel::Argument),  // base
        make_edge(1, 20, 1, EdgeLabel::Argument),  // step (Lambda)
        make_edge(1, 30, 2, EdgeLabel::Argument),  // collection

        // Lambda body via Continuation edge
        Edge {
            source: NodeId(20),
            target: NodeId(100),
            port: 0,
            label: EdgeLabel::Continuation,
        },

        // add(acc, score)
        make_edge(100, 110, 0, EdgeLabel::Argument),  // acc
        make_edge(100, 120, 1, EdgeLabel::Argument),  // score

        // Project(0) from input_ref(2) → acc
        make_edge(110, 115, 0, EdgeLabel::Argument),

        // bool_to_int → eq
        make_edge(120, 130, 0, EdgeLabel::Argument),

        // eq(graph_eval_result, expected)
        make_edge(130, 140, 0, EdgeLabel::Argument),  // graph_eval result
        make_edge(130, 180, 1, EdgeLabel::Argument),  // expected

        // graph_eval(program, inputs)
        make_edge(140, 150, 0, EdgeLabel::Argument),  // program
        make_edge(140, 160, 1, EdgeLabel::Argument),  // test inputs

        // Project(0) from Project(1) from input_ref(2) → tc.inputs
        make_edge(160, 170, 0, EdgeLabel::Argument),
        // Project(1) from input_ref(2)
        make_edge(170, 175, 0, EdgeLabel::Argument),

        // Project(1) from Project(1) from input_ref(2) → tc.expected
        make_edge(180, 190, 0, EdgeLabel::Argument),
        // Project(1) from input_ref(2)
        make_edge(190, 195, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Verify the add program itself works: add(3, 5) = 8.
#[test]
fn add_program_works() {
    let add = make_add_program();
    let inputs = vec![Value::Int(3), Value::Int(5)];
    let (outputs, _) = interpreter::interpret(&add, &inputs, None).unwrap();
    assert_eq!(outputs, vec![Value::Int(8)]);
}

/// Single-case evaluator: add(3,5) with expected=8 should return 1 (pass).
#[test]
fn single_case_pass() {
    let evaluator = build_single_case_evaluator();
    let add_program = make_add_program();

    let inputs = vec![
        Value::Program(Rc::new(add_program)),
        Value::tuple(vec![Value::Int(3), Value::Int(5)]),
        Value::Int(8),
    ];

    let (outputs, _) = interpreter::interpret(&evaluator, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0], Value::Int(1), "add(3,5)==8 should score 1");
}

/// Single-case evaluator: add(3,5) with expected=9 should return 0 (fail).
#[test]
fn single_case_fail() {
    let evaluator = build_single_case_evaluator();
    let add_program = make_add_program();

    let inputs = vec![
        Value::Program(Rc::new(add_program)),
        Value::tuple(vec![Value::Int(3), Value::Int(5)]),
        Value::Int(9),
    ];

    let (outputs, _) = interpreter::interpret(&evaluator, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0], Value::Int(0), "add(3,5)!=9 should score 0");
}

/// Multi-case evaluator: add program with 3 test cases, all passing → returns 3.
#[test]
fn multi_case_all_pass() {
    let evaluator = build_multi_case_evaluator();
    let add_program = make_add_program();

    // Each test case is Tuple(Tuple(inputs...), Int(expected))
    let test_cases = Value::tuple(vec![
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(3), Value::Int(5)]),
            Value::Int(8),
        ]),
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(10), Value::Int(20)]),
            Value::Int(30),
        ]),
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(-1), Value::Int(1)]),
            Value::Int(0),
        ]),
    ]);

    let inputs = vec![
        Value::Program(Rc::new(add_program)),
        test_cases,
    ];

    let (outputs, _) = interpreter::interpret(&evaluator, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(3),
        "3 correct test cases should return 3"
    );
}

/// Multi-case evaluator: add program with 3 test cases, 2 passing → returns 2.
#[test]
fn multi_case_partial_pass() {
    let evaluator = build_multi_case_evaluator();
    let add_program = make_add_program();

    let test_cases = Value::tuple(vec![
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(3), Value::Int(5)]),
            Value::Int(8),   // correct
        ]),
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(10), Value::Int(20)]),
            Value::Int(99),  // wrong
        ]),
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(-1), Value::Int(1)]),
            Value::Int(0),   // correct
        ]),
    ]);

    let inputs = vec![
        Value::Program(Rc::new(add_program)),
        test_cases,
    ];

    let (outputs, _) = interpreter::interpret(&evaluator, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(2),
        "2 correct out of 3 test cases should return 2"
    );
}
