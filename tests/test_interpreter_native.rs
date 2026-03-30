/// Integration tests for the IRIS meta-circular interpreter's native handling.
///
/// Each test:
/// 1. Compiles a simple .iris program to a SemanticGraph
/// 2. Evaluates it directly via the Rust bootstrap evaluator
/// 3. Evaluates it via the IRIS interpreter (full_interpreter.iris)
/// 4. Asserts identical results
///
/// This validates that the IRIS interpreter's native handling of node kinds
/// (Guard, Lit, Apply+Lambda, Let, TypeAbst, TypeApp, Tuple, Prim) produces
/// the same results as the Rust evaluator.

use std::collections::BTreeMap;
use std::rc::Rc;

use iris_bootstrap::syntax;
use iris_types::eval::Value;
use iris_types::fragment::FragmentId;
use iris_types::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compile source and find a named binding's graph + registry.
fn compile_and_find(src: &str, name: &str) -> (SemanticGraph, BTreeMap<FragmentId, SemanticGraph>) {
    let result = syntax::compile(src);
    assert!(result.errors.is_empty(), "compile errors: {:?}", result.errors);
    let mut reg = BTreeMap::new();
    let mut target = None;
    for (n, frag, _) in &result.fragments {
        reg.insert(frag.id, frag.graph.clone());
        if n == name {
            target = Some(frag.graph.clone());
        }
    }
    (target.expect(&format!("function '{}' not found", name)), reg)
}

/// Evaluate a program directly via the Rust bootstrap evaluator.
fn eval_direct(src: &str, name: &str, args: &[Value]) -> Value {
    let (graph, reg) = compile_and_find(src, name);
    iris_bootstrap::evaluate_with_fragments(&graph, args, 5_000_000, &reg)
        .expect("direct evaluation failed")
}

/// Compile the IRIS interpreter, then use it to evaluate a target program.
fn eval_via_interpreter(src: &str, name: &str, args: &[Value]) -> Value {
    // Compile the interpreter
    let interp_src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/iris-programs/interpreter/full_interpreter.iris"
    ))
    .expect("failed to read interpreter source");
    let interp_result = syntax::compile(&interp_src);
    assert!(
        interp_result.errors.is_empty(),
        "interpreter compile errors: {:?}",
        interp_result.errors
    );
    let interp_graph = interp_result
        .fragments
        .last()
        .expect("no fragments in interpreter")
        .1
        .graph
        .clone();

    // Compile the target program
    let (target_graph, _reg) = compile_and_find(src, name);

    // Build the inputs to the interpreter: (program, inputs_tuple)
    let program_val = Value::Program(Rc::new(target_graph));
    let inputs_tuple = Value::tuple(args.to_vec());
    let interp_inputs = vec![program_val, inputs_tuple];

    // Evaluate the interpreter with the target program
    let empty_reg = BTreeMap::new();
    iris_bootstrap::evaluate_with_fragments(&interp_graph, &interp_inputs, 5_000_000, &empty_reg)
        .expect("interpreter evaluation failed")
}

/// Run both direct and interpreter evaluation and assert they match.
fn assert_both_equal(src: &str, name: &str, args: &[Value], expected: Value) {
    let direct = eval_direct(src, name, args);
    assert_eq!(direct, expected, "direct eval mismatch for {}", name);
    let interp = eval_via_interpreter(src, name, args);
    assert_eq!(interp, expected, "interpreter eval mismatch for {}", name);
}

// ---------------------------------------------------------------------------
// Arithmetic prim tests
// ---------------------------------------------------------------------------

#[test]
fn native_add() {
    let src = "let f x y = x + y";
    assert_both_equal(src, "f", &[Value::Int(3), Value::Int(4)], Value::Int(7));
}

#[test]
fn native_sub() {
    let src = "let f x y = x - y";
    assert_both_equal(src, "f", &[Value::Int(10), Value::Int(3)], Value::Int(7));
}

#[test]
fn native_mul() {
    let src = "let f x y = x * y";
    assert_both_equal(src, "f", &[Value::Int(6), Value::Int(7)], Value::Int(42));
}

#[test]
fn native_div() {
    let src = "let f x y = x / y";
    assert_both_equal(src, "f", &[Value::Int(42), Value::Int(6)], Value::Int(7));
}

#[test]
fn native_div_by_zero() {
    // The Rust evaluator throws DivisionByZero, but the IRIS interpreter
    // returns 0 (safe fallback). Test only the interpreter path.
    let src = "let f x y = x / y";
    let interp = eval_via_interpreter(src, "f", &[Value::Int(42), Value::Int(0)]);
    assert_eq!(interp, Value::Int(0));
}

#[test]
fn native_mod() {
    let src = "let f x y = x % y";
    assert_both_equal(src, "f", &[Value::Int(17), Value::Int(5)], Value::Int(2));
}

#[test]
fn native_neg() {
    let src = "let f x = neg x";
    assert_both_equal(src, "f", &[Value::Int(42)], Value::Int(-42));
}

#[test]
fn native_abs() {
    let src = "let f x = abs x";
    assert_both_equal(src, "f", &[Value::Int(-42)], Value::Int(42));
}

#[test]
fn native_min() {
    let src = "let f x y = min x y";
    assert_both_equal(src, "f", &[Value::Int(3), Value::Int(7)], Value::Int(3));
}

#[test]
fn native_max() {
    let src = "let f x y = max x y";
    assert_both_equal(src, "f", &[Value::Int(3), Value::Int(7)], Value::Int(7));
}

#[test]
fn native_pow() {
    let src = "let f x y = pow x y";
    assert_both_equal(src, "f", &[Value::Int(2), Value::Int(10)], Value::Int(1024));
}

// ---------------------------------------------------------------------------
// Comparison prim tests
// ---------------------------------------------------------------------------

#[test]
fn native_eq_true() {
    let src = "let f x y = if x == y then 1 else 0";
    assert_both_equal(src, "f", &[Value::Int(5), Value::Int(5)], Value::Int(1));
}

#[test]
fn native_eq_false() {
    let src = "let f x y = if x == y then 1 else 0";
    assert_both_equal(src, "f", &[Value::Int(5), Value::Int(3)], Value::Int(0));
}

#[test]
fn native_ne() {
    let src = "let f x y = if x != y then 1 else 0";
    assert_both_equal(src, "f", &[Value::Int(5), Value::Int(3)], Value::Int(1));
}

#[test]
fn native_lt() {
    let src = "let f x y = if x < y then 1 else 0";
    assert_both_equal(src, "f", &[Value::Int(3), Value::Int(5)], Value::Int(1));
}

#[test]
fn native_gt() {
    let src = "let f x y = if x > y then 1 else 0";
    assert_both_equal(src, "f", &[Value::Int(5), Value::Int(3)], Value::Int(1));
}

#[test]
fn native_le() {
    let src = "let f x y = if x <= y then 1 else 0";
    assert_both_equal(src, "f", &[Value::Int(5), Value::Int(5)], Value::Int(1));
}

#[test]
fn native_ge() {
    let src = "let f x y = if x >= y then 1 else 0";
    assert_both_equal(src, "f", &[Value::Int(5), Value::Int(5)], Value::Int(1));
}

// ---------------------------------------------------------------------------
// Guard tests (if/then/else)
// ---------------------------------------------------------------------------

#[test]
fn native_guard_true() {
    let src = "let f x = if x > 0 then 42 else 0";
    assert_both_equal(src, "f", &[Value::Int(5)], Value::Int(42));
}

#[test]
fn native_guard_false() {
    let src = "let f x = if x > 0 then 42 else 0";
    assert_both_equal(src, "f", &[Value::Int(-1)], Value::Int(0));
}

#[test]
fn native_guard_nested() {
    let src = r#"
let classify x =
  if x > 10 then 3
  else if x > 5 then 2
  else if x > 0 then 1
  else 0
"#;
    assert_both_equal(src, "classify", &[Value::Int(15)], Value::Int(3));
    assert_both_equal(src, "classify", &[Value::Int(7)], Value::Int(2));
    assert_both_equal(src, "classify", &[Value::Int(3)], Value::Int(1));
    assert_both_equal(src, "classify", &[Value::Int(-1)], Value::Int(0));
}

// ---------------------------------------------------------------------------
// Literal tests
// ---------------------------------------------------------------------------

#[test]
fn native_lit_int() {
    let src = "let f = 42";
    assert_both_equal(src, "f", &[], Value::Int(42));
}

#[test]
fn native_lit_negative() {
    let src = "let f = 0 - 7";
    assert_both_equal(src, "f", &[], Value::Int(-7));
}

#[test]
fn native_lit_input_ref() {
    // The input reference is tested via a function that takes an argument
    let src = "let f x = x";
    assert_both_equal(src, "f", &[Value::Int(99)], Value::Int(99));
}

// ---------------------------------------------------------------------------
// Let binding tests
// ---------------------------------------------------------------------------

#[test]
fn native_let_simple() {
    let src = "let f x = let a = x + 1 in a + 2";
    assert_both_equal(src, "f", &[Value::Int(10)], Value::Int(13));
}

#[test]
fn native_let_nested() {
    let src = "let f x = let a = x + 1 in let b = a * 2 in b + 3";
    assert_both_equal(src, "f", &[Value::Int(5)], Value::Int(15));
}

#[test]
fn native_let_with_guard() {
    let src = r#"
let f x =
  let sign = if x > 0 then 1 else 0 - 1 in
  let result = sign * x in
  result
"#;
    assert_both_equal(src, "f", &[Value::Int(5)], Value::Int(5));
    assert_both_equal(src, "f", &[Value::Int(-3)], Value::Int(3));
}

// ---------------------------------------------------------------------------
// Lambda / Apply tests
// ---------------------------------------------------------------------------

#[test]
fn native_lambda_apply_simple() {
    let src = r#"
let f x =
  let double = \y -> y * 2 in
  double x
"#;
    assert_both_equal(src, "f", &[Value::Int(5)], Value::Int(10));
}

#[test]
fn native_lambda_apply_with_let() {
    let src = r#"
let f x =
  let inc = \y -> y + 1 in
  let a = inc x in
  inc a
"#;
    assert_both_equal(src, "f", &[Value::Int(5)], Value::Int(7));
}

// ---------------------------------------------------------------------------
// Nested expression tests
// ---------------------------------------------------------------------------

#[test]
fn native_nested_arithmetic() {
    let src = "let f x y = (x + y) * (x - y)";
    assert_both_equal(src, "f", &[Value::Int(10), Value::Int(3)], Value::Int(91));
}

#[test]
fn native_complex_expression() {
    let src = r#"
let f x =
  let a = x * x in
  let b = a + x in
  if b > 100 then a else b
"#;
    assert_both_equal(src, "f", &[Value::Int(10)], Value::Int(100));
    assert_both_equal(src, "f", &[Value::Int(5)], Value::Int(30));
}

#[test]
fn native_guard_in_arithmetic() {
    let src = r#"
let f x y =
  let mx = if x > y then x else y in
  let mn = if x < y then x else y in
  mx - mn
"#;
    assert_both_equal(src, "f", &[Value::Int(10), Value::Int(3)], Value::Int(7));
    assert_both_equal(src, "f", &[Value::Int(3), Value::Int(10)], Value::Int(7));
}

// ---------------------------------------------------------------------------
// Introspection primitive tests (graph_edge_target, graph_get_binder, etc.)
// ---------------------------------------------------------------------------

#[test]
fn prim_graph_edge_target_basic() {
    // Test that graph_edge_target can find edges in a compiled program
    let src = "let f x = x + 1";
    let (graph, _reg) = compile_and_find(src, "f");
    let program_val = Value::Program(Rc::new(graph.clone()));

    // The root should be a Prim(add) node with Argument edges
    let root_id = Value::Int(graph.root.0 as i64);

    // Test via a program that calls graph_edge_target
    // NodeIds are content-addressed u64 hashes, so they can be very large.
    // We check that the result is not -1 (i.e. the edge was found).
    let test_src = r#"
let test pg root_id =
  let target = graph_edge_target pg root_id 0 0 in
  if target == 0 - 1 then 0 else 1
"#;
    let result = eval_direct(
        test_src,
        "test",
        &[program_val, root_id],
    );
    assert_eq!(result, Value::Int(1), "graph_edge_target should find an Argument edge");
}

#[test]
fn prim_graph_get_binder_lambda() {
    // Compile a lambda and check that graph_get_binder returns its binder ID
    let src = "let f = \\x -> x + 1";
    let (graph, _reg) = compile_and_find(src, "f");

    // The root should be a Lambda node
    let program_val = Value::Program(Rc::new(graph.clone()));
    let root_id = Value::Int(graph.root.0 as i64);

    let test_src = "let test pg nid = graph_get_binder pg nid";
    let result = eval_direct(test_src, "test", &[program_val, root_id]);
    match result {
        Value::Int(n) => assert!(n >= 0, "expected valid binder ID, got {}", n),
        _ => panic!("expected Int, got {:?}", result),
    }
}

#[test]
fn prim_graph_get_tag_inject() {
    // graph_get_tag should return -1 for non-Inject nodes
    let src = "let f x = x + 1";
    let (graph, _reg) = compile_and_find(src, "f");
    let program_val = Value::Program(Rc::new(graph.clone()));
    let root_id = Value::Int(graph.root.0 as i64);

    let test_src = "let test pg nid = graph_get_tag pg nid";
    let result = eval_direct(test_src, "test", &[program_val, root_id]);
    assert_eq!(result, Value::Int(-1));
}

#[test]
fn prim_graph_get_field_index_non_project() {
    // graph_get_field_index should return -1 for non-Project nodes
    let src = "let f x = x + 1";
    let (graph, _reg) = compile_and_find(src, "f");
    let program_val = Value::Program(Rc::new(graph.clone()));
    let root_id = Value::Int(graph.root.0 as i64);

    let test_src = "let test pg nid = graph_get_field_index pg nid";
    let result = eval_direct(test_src, "test", &[program_val, root_id]);
    assert_eq!(result, Value::Int(-1));
}

#[test]
fn prim_graph_eval_env_basic() {
    // Test graph_eval_env: compile a lambda body and evaluate with a bound variable
    let src = "let f = \\x -> x + 1";
    let (graph, _reg) = compile_and_find(src, "f");
    let program_val = Value::Program(Rc::new(graph.clone()));

    // Get the binder and body from the lambda
    let test_src = r#"
let test pg =
  let root = graph_get_root pg in
  let binder = graph_get_binder pg root in
  let body_id = graph_edge_target pg root 0 3 in
  let body_id2 = if body_id == 0 - 1 then graph_edge_target pg root 0 0 else body_id in
  let body_prog = graph_set_root pg body_id2 in
  graph_eval_env body_prog binder 41
"#;
    let result = eval_direct(test_src, "test", &[program_val]);
    assert_eq!(result, Value::Int(42));
}
