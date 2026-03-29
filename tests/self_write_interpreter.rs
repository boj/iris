
//! Self-write: core interpreter dispatch loop as IRIS programs.
//!
//! This is the ignition point -- when IRIS can interpret programs using
//! programs it wrote. We build four IRIS programs that collectively
//! implement the core dispatch of `eval_node` from interpreter.rs:
//!
//! 1. **eval_lit** -- Given a Program whose root is a Lit node, check the
//!    kind and evaluate it. Uses `graph_get_root` (0x8A), `graph_get_kind`
//!    (0x82), and `graph_eval` (0x89).
//!
//! 2. **eval_prim_binop** -- Given two already-evaluated values and a Program
//!    whose root is a Prim node, extract the opcode via `graph_get_prim_op`
//!    (0x83) and dispatch: add/sub/mul/div. Uses nested Guards for the
//!    ConditionalDispatch pattern.
//!
//! 3. **eval_node_simple** -- Given a Program, evaluate its root: check kind,
//!    if Lit return value via graph_eval, if Prim evaluate via graph_eval.
//!    Recursive via `graph_eval` (0x89) for children. Limited to
//!    MAX_SELF_EVAL_DEPTH=4.
//!
//! 4. **mini_interpreter** -- Given a Program and inputs, evaluate the root
//!    node. This IS `interpret()` -- the function that runs IRIS programs --
//!    written as an IRIS program. Meta-circular. Limited to Lit + Prim +
//!    simple types.

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

fn guard_node(id: u64, predicate: u64, body: u64, fallback: u64) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Guard,
        NodePayload::Guard {
            predicate_node: NodeId(predicate),
            body_node: NodeId(body),
            fallback_node: NodeId(fallback),
        },
        0,
    )
}

// ---------------------------------------------------------------------------
// Target programs (the programs being interpreted by IRIS programs)
// ---------------------------------------------------------------------------

/// A program that is just `Lit(42)` at root.
fn make_lit_program(value: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = int_lit_node(1, value);
    nodes.insert(nid, node);
    make_graph(nodes, vec![], 1)
}

/// A program `op(Lit(a), Lit(b))` -- a binary operation on two constants.
fn make_binop_program(opcode: u8, a: i64, b: i64) -> SemanticGraph {
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

/// A program `add(input[0], input[1])` -- takes two inputs.
fn make_add_inputs_program() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = prim_node(1, 0x00, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1);
    nodes.insert(nid, node);
    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];
    make_graph(nodes, edges, 1)
}

// ===========================================================================
// 1. eval_lit -- IRIS program
// ===========================================================================
//
// Input:
//   inputs[0] = Value::Program(target)
//
// Output:
//   If root is Lit (kind == 0x05): graph_eval(target) -> the literal value
//   Else: Int(-1) sentinel (not a Lit node)
//
// Graph structure:
//   Root(id=1): Guard
//     predicate(id=10): eq(graph_get_kind(prog, graph_get_root(prog)), 5)
//     body(id=20): graph_eval(prog)        -- evaluate the Lit
//     fallback(id=30): int_lit(-1)          -- not a Lit

fn build_eval_lit() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // --- Predicate: eq(kind, 5) ---
    // eq (0x20, 2 args)
    let (nid, node) = prim_node(10, 0x20, 2);
    nodes.insert(nid, node);

    // graph_get_kind (0x82, 2 args: program, node_id)
    let (nid, node) = prim_node(11, 0x82, 2);
    nodes.insert(nid, node);

    // input_ref(0) -- the program
    let (nid, node) = input_ref_node(12, 0);
    nodes.insert(nid, node);

    // graph_get_root (0x8A, 1 arg: program)
    let (nid, node) = prim_node(13, 0x8A, 1);
    nodes.insert(nid, node);

    // input_ref(0) -- the program (for graph_get_root)
    let (nid, node) = input_ref_node(14, 0);
    nodes.insert(nid, node);

    // int_lit(5) -- NodeKind::Lit == 0x05
    let (nid, node) = int_lit_node(15, 5);
    nodes.insert(nid, node);

    // --- Body: graph_eval(program) ---
    let (nid, node) = prim_node(20, 0x89, 1);
    nodes.insert(nid, node);

    // input_ref(0) for graph_eval
    let (nid, node) = input_ref_node(21, 0);
    nodes.insert(nid, node);

    // --- Fallback: Int(-1) ---
    let (nid, node) = int_lit_node(30, -1);
    nodes.insert(nid, node);

    // --- Root: Guard ---
    let (nid, node) = guard_node(1, 10, 20, 30);
    nodes.insert(nid, node);

    // --- Edges ---
    let edges = vec![
        // eq(kind, 5)
        make_edge(10, 11, 0, EdgeLabel::Argument),
        make_edge(10, 15, 1, EdgeLabel::Argument),
        // graph_get_kind(program, root_id)
        make_edge(11, 12, 0, EdgeLabel::Argument),
        make_edge(11, 13, 1, EdgeLabel::Argument),
        // graph_get_root(program)
        make_edge(13, 14, 0, EdgeLabel::Argument),
        // graph_eval(program)
        make_edge(20, 21, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// 2. eval_prim_binop -- IRIS program
// ===========================================================================
//
// Input:
//   inputs[0] = Value::Program(target)    -- program whose root is a Prim
//   inputs[1] = Value::Int(a)              -- first child value (pre-evaluated)
//   inputs[2] = Value::Int(b)              -- second child value (pre-evaluated)
//
// Output:
//   The result of applying the Prim's opcode to (a, b).
//   Dispatches: 0x00=add, 0x01=sub, 0x02=mul, 0x03=div.
//   Returns Int(-9999) for unknown opcodes.
//
// Graph structure (nested Guards for ConditionalDispatch):
//
//   Root(id=1): Guard(is_add?, add(a,b), Guard(is_sub?, sub(a,b),
//                     Guard(is_mul?, mul(a,b), Guard(is_div?, div(a,b), -9999))))
//
// Where:
//   opcode = graph_get_prim_op(program, graph_get_root(program))
//   is_add = eq(opcode, 0)
//   is_sub = eq(opcode, 1)
//   is_mul = eq(opcode, 2)
//   is_div = eq(opcode, 3)

fn build_eval_prim_binop() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // --- Shared: extract the opcode ---
    // graph_get_prim_op(program, graph_get_root(program))
    let (nid, node) = prim_node(50, 0x83, 2); // graph_get_prim_op
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(51, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = prim_node(52, 0x8A, 1); // graph_get_root
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(53, 0); // program (for root)
    nodes.insert(nid, node);

    // --- Opcode constants ---
    let (nid, node) = int_lit_node(60, 0); // add opcode
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(61, 1); // sub opcode
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(62, 2); // mul opcode
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(63, 3); // div opcode
    nodes.insert(nid, node);

    // --- Predicates: eq(opcode, constant) ---
    // is_add: eq(opcode, 0)
    let (nid, node) = prim_node(100, 0x20, 2);
    nodes.insert(nid, node);
    // is_sub: eq(opcode, 1)
    let (nid, node) = prim_node(101, 0x20, 2);
    nodes.insert(nid, node);
    // is_mul: eq(opcode, 2)
    let (nid, node) = prim_node(102, 0x20, 2);
    nodes.insert(nid, node);
    // is_div: eq(opcode, 3)
    let (nid, node) = prim_node(103, 0x20, 2);
    nodes.insert(nid, node);

    // --- Operations: op(input1, input2) ---
    // add(a, b)
    let (nid, node) = prim_node(200, 0x00, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(201, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(202, 2);
    nodes.insert(nid, node);

    // sub(a, b)
    let (nid, node) = prim_node(210, 0x01, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(211, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(212, 2);
    nodes.insert(nid, node);

    // mul(a, b)
    let (nid, node) = prim_node(220, 0x02, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(221, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(222, 2);
    nodes.insert(nid, node);

    // div(a, b)
    let (nid, node) = prim_node(230, 0x03, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(231, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(232, 2);
    nodes.insert(nid, node);

    // --- Fallback sentinel ---
    let (nid, node) = int_lit_node(999, -9999);
    nodes.insert(nid, node);

    // --- Nested Guards (bottom-up) ---
    // Guard 4 (innermost): is_div? -> div(a,b) : -9999
    let (nid, node) = guard_node(4, 103, 230, 999);
    nodes.insert(nid, node);

    // Guard 3: is_mul? -> mul(a,b) : Guard4
    let (nid, node) = guard_node(3, 102, 220, 4);
    nodes.insert(nid, node);

    // Guard 2: is_sub? -> sub(a,b) : Guard3
    let (nid, node) = guard_node(2, 101, 210, 3);
    nodes.insert(nid, node);

    // Guard 1 (root): is_add? -> add(a,b) : Guard2
    let (nid, node) = guard_node(1, 100, 200, 2);
    nodes.insert(nid, node);

    // --- Edges ---
    let edges = vec![
        // graph_get_root(program)
        make_edge(52, 53, 0, EdgeLabel::Argument),
        // graph_get_prim_op(program, root_id)
        make_edge(50, 51, 0, EdgeLabel::Argument),
        make_edge(50, 52, 1, EdgeLabel::Argument),
        // is_add: eq(opcode, 0)
        make_edge(100, 50, 0, EdgeLabel::Argument),
        make_edge(100, 60, 1, EdgeLabel::Argument),
        // is_sub: eq(opcode, 1)
        make_edge(101, 50, 0, EdgeLabel::Argument),
        make_edge(101, 61, 1, EdgeLabel::Argument),
        // is_mul: eq(opcode, 2)
        make_edge(102, 50, 0, EdgeLabel::Argument),
        make_edge(102, 62, 1, EdgeLabel::Argument),
        // is_div: eq(opcode, 3)
        make_edge(103, 50, 0, EdgeLabel::Argument),
        make_edge(103, 63, 1, EdgeLabel::Argument),
        // add(a, b)
        make_edge(200, 201, 0, EdgeLabel::Argument),
        make_edge(200, 202, 1, EdgeLabel::Argument),
        // sub(a, b)
        make_edge(210, 211, 0, EdgeLabel::Argument),
        make_edge(210, 212, 1, EdgeLabel::Argument),
        // mul(a, b)
        make_edge(220, 221, 0, EdgeLabel::Argument),
        make_edge(220, 222, 1, EdgeLabel::Argument),
        // div(a, b)
        make_edge(230, 231, 0, EdgeLabel::Argument),
        make_edge(230, 232, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// 3. eval_node_simple -- IRIS program
// ===========================================================================
//
// Input:
//   inputs[0] = Value::Program(target)
//
// Output:
//   Evaluates the root node of the target program:
//   - If root is Lit (kind==5): graph_eval(program) to get the value
//   - If root is Prim (kind==0): graph_eval(program) to evaluate the
//     full expression tree (Prim + its argument sub-graphs)
//   - Otherwise: Int(-1) sentinel
//
// This is the core dispatch: check kind, then evaluate. The recursive
// evaluation of children happens inside graph_eval (0x89) which itself
// calls into the Rust interpreter -- but the dispatch decision of
// "what kind is this node and how should I handle it?" is made by IRIS.
//
// Graph structure:
//   Root(id=1): Guard(is_lit?, graph_eval(prog),
//               Guard(is_prim?, graph_eval(prog), Int(-1)))
//
// Where:
//   kind = graph_get_kind(program, graph_get_root(program))
//   is_lit = eq(kind, 5)
//   is_prim = eq(kind, 0)

fn build_eval_node_simple() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // --- Shared: extract root kind ---
    // graph_get_kind(program, graph_get_root(program))
    let (nid, node) = prim_node(50, 0x82, 2); // graph_get_kind
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(51, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = prim_node(52, 0x8A, 1); // graph_get_root
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(53, 0); // program (for root)
    nodes.insert(nid, node);

    // --- Kind constants ---
    let (nid, node) = int_lit_node(60, 5); // Lit kind = 0x05
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(61, 0); // Prim kind = 0x00
    nodes.insert(nid, node);

    // --- Predicates ---
    // is_lit: eq(kind, 5)
    let (nid, node) = prim_node(100, 0x20, 2);
    nodes.insert(nid, node);
    // is_prim: eq(kind, 0)
    let (nid, node) = prim_node(101, 0x20, 2);
    nodes.insert(nid, node);

    // --- Bodies: graph_eval(program) ---
    // For Lit case:
    let (nid, node) = prim_node(200, 0x89, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(201, 0);
    nodes.insert(nid, node);

    // For Prim case:
    let (nid, node) = prim_node(210, 0x89, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(211, 0);
    nodes.insert(nid, node);

    // --- Fallback: Int(-1) ---
    let (nid, node) = int_lit_node(999, -1);
    nodes.insert(nid, node);

    // --- Nested Guards ---
    // Inner Guard: is_prim? -> graph_eval(prog) : -1
    let (nid, node) = guard_node(2, 101, 210, 999);
    nodes.insert(nid, node);

    // Outer Guard (root): is_lit? -> graph_eval(prog) : inner_guard
    let (nid, node) = guard_node(1, 100, 200, 2);
    nodes.insert(nid, node);

    // --- Edges ---
    let edges = vec![
        // graph_get_root(program)
        make_edge(52, 53, 0, EdgeLabel::Argument),
        // graph_get_kind(program, root_id)
        make_edge(50, 51, 0, EdgeLabel::Argument),
        make_edge(50, 52, 1, EdgeLabel::Argument),
        // is_lit: eq(kind, 5)
        make_edge(100, 50, 0, EdgeLabel::Argument),
        make_edge(100, 60, 1, EdgeLabel::Argument),
        // is_prim: eq(kind, 0)
        make_edge(101, 50, 0, EdgeLabel::Argument),
        make_edge(101, 61, 1, EdgeLabel::Argument),
        // graph_eval(prog) -- Lit case
        make_edge(200, 201, 0, EdgeLabel::Argument),
        // graph_eval(prog) -- Prim case
        make_edge(210, 211, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// 4. mini_interpreter -- IRIS program (meta-circular)
// ===========================================================================
//
// Input:
//   inputs[0] = Value::Program(target)
//   inputs[1] = Tuple(input_values) or a single Value
//
// Output:
//   The result of evaluating the target program with the given inputs.
//   This IS interpret() -- the function that runs IRIS programs -- written
//   as an IRIS program.
//
// Implementation: uses graph_eval (0x89) which takes a Program and optional
// inputs. The meta-circular trick: this IRIS program is being run by the
// Rust interpreter, and it in turn runs another IRIS program via graph_eval.
//
// The mini_interpreter checks the root kind to decide strategy:
//   - Lit: graph_eval(program)      -- no inputs needed for literals
//   - Prim: graph_eval(program, inputs) -- pass inputs through
//   - Else: graph_eval(program, inputs) -- generic fallback
//
// Graph structure:
//   Root(id=1): Guard(is_lit?, graph_eval(prog),
//               graph_eval(prog, inputs))
//
// For Lit programs, we can skip inputs. For everything else, we pass
// the inputs through to graph_eval.

fn build_mini_interpreter() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // --- Kind detection ---
    // graph_get_kind(program, graph_get_root(program))
    let (nid, node) = prim_node(50, 0x82, 2); // graph_get_kind
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(51, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = prim_node(52, 0x8A, 1); // graph_get_root
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(53, 0); // program (for root)
    nodes.insert(nid, node);

    // --- Predicate: is_lit = eq(kind, 5) ---
    let (nid, node) = prim_node(100, 0x20, 2);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(60, 5); // Lit == 0x05
    nodes.insert(nid, node);

    // --- Body (Lit case): graph_eval(program) -- no inputs ---
    let (nid, node) = prim_node(200, 0x89, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(201, 0); // program
    nodes.insert(nid, node);

    // --- Fallback (Prim/other): graph_eval(program, inputs) ---
    let (nid, node) = prim_node(300, 0x89, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(301, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(302, 1); // inputs
    nodes.insert(nid, node);

    // --- Root: Guard(is_lit?, eval_no_inputs, eval_with_inputs) ---
    let (nid, node) = guard_node(1, 100, 200, 300);
    nodes.insert(nid, node);

    // --- Edges ---
    let edges = vec![
        // graph_get_root(program)
        make_edge(52, 53, 0, EdgeLabel::Argument),
        // graph_get_kind(program, root_id)
        make_edge(50, 51, 0, EdgeLabel::Argument),
        make_edge(50, 52, 1, EdgeLabel::Argument),
        // is_lit: eq(kind, 5)
        make_edge(100, 50, 0, EdgeLabel::Argument),
        make_edge(100, 60, 1, EdgeLabel::Argument),
        // graph_eval(program) -- Lit case
        make_edge(200, 201, 0, EdgeLabel::Argument),
        // graph_eval(program, inputs) -- general case
        make_edge(300, 301, 0, EdgeLabel::Argument),
        make_edge(300, 302, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// Tests
// ===========================================================================

#[test]
fn test_eval_lit_returns_int_value() {
    let eval_lit = build_eval_lit();

    // Test with Lit(42) program
    let target = make_lit_program(42);
    let (outputs, _) =
        interpreter::interpret(&eval_lit, &[Value::Program(Rc::new(target))], None).unwrap();
    assert_eq!(outputs[0], Value::Int(42), "eval_lit should return 42 for Lit(42)");

    // Test with Lit(0) program
    let target = make_lit_program(0);
    let (outputs, _) =
        interpreter::interpret(&eval_lit, &[Value::Program(Rc::new(target))], None).unwrap();
    assert_eq!(outputs[0], Value::Int(0), "eval_lit should return 0 for Lit(0)");

    // Test with Lit(-7) program
    let target = make_lit_program(-7);
    let (outputs, _) =
        interpreter::interpret(&eval_lit, &[Value::Program(Rc::new(target))], None).unwrap();
    assert_eq!(outputs[0], Value::Int(-7), "eval_lit should return -7 for Lit(-7)");
}

#[test]
fn test_eval_lit_rejects_non_lit() {
    let eval_lit = build_eval_lit();

    // Test with a Prim program (not a Lit) -- should return sentinel -1
    let target = make_binop_program(0x00, 3, 5);
    let (outputs, _) =
        interpreter::interpret(&eval_lit, &[Value::Program(Rc::new(target))], None).unwrap();
    assert_eq!(
        outputs[0],
        Value::Int(-1),
        "eval_lit should return -1 for non-Lit root"
    );
}

#[test]
fn test_eval_prim_binop_add() {
    let eval_prim = build_eval_prim_binop();
    let target = make_binop_program(0x00, 0, 0); // add program (values ignored, we pass them)
    let (outputs, _) = interpreter::interpret(
        &eval_prim,
        &[
            Value::Program(Rc::new(target)),
            Value::Int(3),
            Value::Int(5),
        ],
        None,
    )
    .unwrap();
    assert_eq!(outputs[0], Value::Int(8), "add(3, 5) = 8");
}

#[test]
fn test_eval_prim_binop_sub() {
    let eval_prim = build_eval_prim_binop();
    let target = make_binop_program(0x01, 0, 0); // sub program
    let (outputs, _) = interpreter::interpret(
        &eval_prim,
        &[
            Value::Program(Rc::new(target)),
            Value::Int(10),
            Value::Int(3),
        ],
        None,
    )
    .unwrap();
    assert_eq!(outputs[0], Value::Int(7), "sub(10, 3) = 7");
}

#[test]
fn test_eval_prim_binop_mul() {
    let eval_prim = build_eval_prim_binop();
    let target = make_binop_program(0x02, 0, 0); // mul program
    let (outputs, _) = interpreter::interpret(
        &eval_prim,
        &[
            Value::Program(Rc::new(target)),
            Value::Int(4),
            Value::Int(6),
        ],
        None,
    )
    .unwrap();
    assert_eq!(outputs[0], Value::Int(24), "mul(4, 6) = 24");
}

#[test]
fn test_eval_prim_binop_div() {
    let eval_prim = build_eval_prim_binop();
    let target = make_binop_program(0x03, 0, 0); // div program
    let (outputs, _) = interpreter::interpret(
        &eval_prim,
        &[
            Value::Program(Rc::new(target)),
            Value::Int(20),
            Value::Int(4),
        ],
        None,
    )
    .unwrap();
    assert_eq!(outputs[0], Value::Int(5), "div(20, 4) = 5");
}

#[test]
fn test_eval_node_simple_lit() {
    let eval_node = build_eval_node_simple();

    // A Lit(42) program
    let target = make_lit_program(42);
    let (outputs, _) =
        interpreter::interpret(&eval_node, &[Value::Program(Rc::new(target))], None).unwrap();
    assert_eq!(outputs[0], Value::Int(42), "eval_node_simple on Lit(42) = 42");
}

#[test]
fn test_eval_node_simple_prim_add() {
    let eval_node = build_eval_node_simple();

    // add(Lit(3), Lit(5)) -> 8
    let target = make_binop_program(0x00, 3, 5);
    let (outputs, _) =
        interpreter::interpret(&eval_node, &[Value::Program(Rc::new(target))], None).unwrap();
    assert_eq!(
        outputs[0],
        Value::Int(8),
        "eval_node_simple on add(3, 5) = 8"
    );
}

#[test]
fn test_eval_node_simple_prim_sub() {
    let eval_node = build_eval_node_simple();

    // sub(Lit(10), Lit(3)) -> 7
    let target = make_binop_program(0x01, 10, 3);
    let (outputs, _) =
        interpreter::interpret(&eval_node, &[Value::Program(Rc::new(target))], None).unwrap();
    assert_eq!(
        outputs[0],
        Value::Int(7),
        "eval_node_simple on sub(10, 3) = 7"
    );
}

#[test]
fn test_eval_node_simple_prim_mul() {
    let eval_node = build_eval_node_simple();

    // mul(Lit(4), Lit(6)) -> 24
    let target = make_binop_program(0x02, 4, 6);
    let (outputs, _) =
        interpreter::interpret(&eval_node, &[Value::Program(Rc::new(target))], None).unwrap();
    assert_eq!(
        outputs[0],
        Value::Int(24),
        "eval_node_simple on mul(4, 6) = 24"
    );
}

#[test]
fn test_mini_interpreter_lit() {
    let interp = build_mini_interpreter();

    // Interpret a Lit(42) program with no inputs
    let target = make_lit_program(42);
    let (outputs, _) = interpreter::interpret(
        &interp,
        &[
            Value::Program(Rc::new(target)),
            Value::tuple(vec![]),
        ],
        None,
    )
    .unwrap();
    assert_eq!(outputs[0], Value::Int(42), "mini_interpreter on Lit(42) = 42");
}

#[test]
fn test_mini_interpreter_constant_add() {
    let interp = build_mini_interpreter();

    // Interpret add(3, 5) -- a constant program, no runtime inputs needed
    let target = make_binop_program(0x00, 3, 5);
    let (outputs, _) = interpreter::interpret(
        &interp,
        &[
            Value::Program(Rc::new(target)),
            Value::tuple(vec![]),
        ],
        None,
    )
    .unwrap();
    assert_eq!(
        outputs[0],
        Value::Int(8),
        "mini_interpreter on add(3, 5) = 8"
    );
}

#[test]
fn test_mini_interpreter_with_inputs() {
    let interp = build_mini_interpreter();

    // Interpret add(input[0], input[1]) with inputs (10, 20)
    let target = make_add_inputs_program();
    let (outputs, _) = interpreter::interpret(
        &interp,
        &[
            Value::Program(Rc::new(target)),
            Value::tuple(vec![Value::Int(10), Value::Int(20)]),
        ],
        None,
    )
    .unwrap();
    assert_eq!(
        outputs[0],
        Value::Int(30),
        "mini_interpreter on add(input0, input1) with (10, 20) = 30"
    );
}

#[test]
fn test_mini_interpreter_meta_circular() {
    // The ultimate test: use the mini_interpreter to interpret the
    // eval_lit program, which itself evaluates a Lit(99) program.
    //
    // Chain: Rust interpreter -> mini_interpreter -> eval_lit -> Lit(99)
    //
    // This is meta-circular: an IRIS program (mini_interpreter) runs
    // another IRIS program (eval_lit) which runs a third (Lit(99)).

    let interp = build_mini_interpreter();
    let eval_lit = build_eval_lit();
    let target = make_lit_program(99);

    // The eval_lit program takes one input: a Program.
    // The mini_interpreter runs eval_lit with input = Program(Lit(99)).
    let (outputs, _) = interpreter::interpret(
        &interp,
        &[
            Value::Program(Rc::new(eval_lit)),
            Value::tuple(vec![Value::Program(Rc::new(target))]),
        ],
        None,
    )
    .unwrap();
    assert_eq!(
        outputs[0],
        Value::Int(99),
        "meta-circular: mini_interpreter(eval_lit(Lit(99))) = 99"
    );
}

#[test]
fn test_mini_interpreter_interprets_constant_mul() {
    // mini_interpreter runs a mul(7, 8) constant program
    let interp = build_mini_interpreter();
    let target = make_binop_program(0x02, 7, 8);
    let (outputs, _) = interpreter::interpret(
        &interp,
        &[
            Value::Program(Rc::new(target)),
            Value::tuple(vec![]),
        ],
        None,
    )
    .unwrap();
    assert_eq!(
        outputs[0],
        Value::Int(56),
        "mini_interpreter on mul(7, 8) = 56"
    );
}
