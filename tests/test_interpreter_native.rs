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

// ---------------------------------------------------------------------------
// Match tests
// ---------------------------------------------------------------------------

#[test]
fn native_match_tagged_some() {
    // Match on a Tagged value (ADT constructor)
    let src = r#"
type Option = Some(Int) | None
let f x =
    match x with
    | Some(v) -> v + 1
    | None -> 0
"#;
    let direct = eval_direct(src, "f", &[Value::Tagged(0, Box::new(Value::Int(41)))]);
    assert_eq!(direct, Value::Int(42), "direct eval mismatch");
    let interp = eval_via_interpreter(src, "f", &[Value::Tagged(0, Box::new(Value::Int(41)))]);
    assert_eq!(interp, Value::Int(42), "interpreter eval mismatch");
}

#[test]
fn native_match_tagged_none() {
    let src = r#"
type Option = Some(Int) | None
let f x =
    match x with
    | Some(v) -> v + 1
    | None -> 0
"#;
    let direct = eval_direct(src, "f", &[Value::Tagged(1, Box::new(Value::Unit))]);
    assert_eq!(direct, Value::Int(0), "direct eval mismatch");
    let interp = eval_via_interpreter(src, "f", &[Value::Tagged(1, Box::new(Value::Unit))]);
    assert_eq!(interp, Value::Int(0), "interpreter eval mismatch");
}

#[test]
fn native_match_bool_like() {
    // Match on a 2-variant type acts as Bool match
    let src = r#"
type MyBool = MyFalse | MyTrue
let f x =
    match x with
    | MyFalse -> 10
    | MyTrue -> 20
"#;
    // Tag 1 = MyTrue -> arm 1 -> 20
    let direct = eval_direct(src, "f", &[Value::Tagged(1, Box::new(Value::Unit))]);
    assert_eq!(direct, Value::Int(20), "direct eval mismatch");
    let interp = eval_via_interpreter(src, "f", &[Value::Tagged(1, Box::new(Value::Unit))]);
    assert_eq!(interp, Value::Int(20), "interpreter eval mismatch");
}

#[test]
fn native_match_three_arms() {
    // Match with 3 constructors
    let src = r#"
type Color = Red | Green | Blue
let f x =
    match x with
    | Red -> 1
    | Green -> 2
    | Blue -> 3
"#;
    // Test all three arms
    for (tag, expected) in [(0, 1), (1, 2), (2, 3)] {
        let val = Value::Tagged(tag, Box::new(Value::Unit));
        let direct = eval_direct(src, "f", &[val.clone()]);
        assert_eq!(direct, Value::Int(expected), "direct mismatch for tag {}", tag);
        let interp = eval_via_interpreter(src, "f", &[val]);
        assert_eq!(interp, Value::Int(expected), "interpreter mismatch for tag {}", tag);
    }
}

// ---------------------------------------------------------------------------
// Fold tests
// ---------------------------------------------------------------------------

#[test]
fn native_fold_add_tuple() {
    // fold 0 add (1, 2, 3) = 6
    let src = "let f lst = fold 0 add lst";
    let tuple = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    assert_both_equal(src, "f", &[tuple], Value::Int(6));
}

#[test]
fn native_fold_mul_tuple() {
    // fold 1 mul (2, 3, 4) = 24
    let src = "let f lst = fold 1 mul lst";
    let tuple = Value::tuple(vec![Value::Int(2), Value::Int(3), Value::Int(4)]);
    assert_both_equal(src, "f", &[tuple], Value::Int(24));
}

#[test]
fn native_fold_add_single() {
    // fold 10 add (5,) = 15
    let src = "let f lst = fold 10 add lst";
    let tuple = Value::tuple(vec![Value::Int(5)]);
    assert_both_equal(src, "f", &[tuple], Value::Int(15));
}

#[test]
fn native_fold_add_empty() {
    // fold 42 add () = 42 (empty collection returns base)
    let src = "let f lst = fold 42 add lst";
    let tuple = Value::tuple(vec![]);
    assert_both_equal(src, "f", &[tuple], Value::Int(42));
}

#[test]
fn native_fold_min_tuple() {
    // fold 100 min (5, 3, 8, 1) = 1
    let src = "let f lst = fold 100 min lst";
    let tuple = Value::tuple(vec![
        Value::Int(5), Value::Int(3), Value::Int(8), Value::Int(1),
    ]);
    assert_both_equal(src, "f", &[tuple], Value::Int(1));
}

#[test]
fn native_fold_max_tuple() {
    // fold 0 max (5, 3, 8, 1) = 8
    let src = "let f lst = fold 0 max lst";
    let tuple = Value::tuple(vec![
        Value::Int(5), Value::Int(3), Value::Int(8), Value::Int(1),
    ]);
    assert_both_equal(src, "f", &[tuple], Value::Int(8));
}

#[test]
fn native_fold_sub_tuple() {
    // fold 20 sub (3, 5, 2) = 20 - 3 - 5 - 2 = 10
    let src = "let f lst = fold 20 sub lst";
    let tuple = Value::tuple(vec![Value::Int(3), Value::Int(5), Value::Int(2)]);
    assert_both_equal(src, "f", &[tuple], Value::Int(10));
}

#[test]
fn native_fold_add_8_elements() {
    // fold 0 add (1,2,3,4,5,6,7,8) = 36
    let src = "let f lst = fold 0 add lst";
    let tuple = Value::tuple(vec![
        Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4),
        Value::Int(5), Value::Int(6), Value::Int(7), Value::Int(8),
    ]);
    assert_both_equal(src, "f", &[tuple], Value::Int(36));
}

#[test]
fn native_fold_add_int_range() {
    // fold 0 add 5 = 0+0+1+2+3+4 = 10 (Int(5) is treated as range [0..5))
    let src = "let f n = fold 0 add n";
    assert_both_equal(src, "f", &[Value::Int(5)], Value::Int(10));
}

// ---------------------------------------------------------------------------
// New primitive tests (value_get_tag, value_get_payload, tuple_len)
// ---------------------------------------------------------------------------

#[test]
fn prim_value_get_tag_tagged() {
    let src = "let f x = value_get_tag x";
    let result = eval_direct(src, "f", &[Value::Tagged(3, Box::new(Value::Int(42)))]);
    assert_eq!(result, Value::Int(3));
}

#[test]
fn prim_value_get_tag_int() {
    let src = "let f x = value_get_tag x";
    let result = eval_direct(src, "f", &[Value::Int(7)]);
    assert_eq!(result, Value::Int(7));
}

#[test]
fn prim_value_get_payload_tagged() {
    let src = "let f x = value_get_payload x";
    let result = eval_direct(src, "f", &[Value::Tagged(1, Box::new(Value::Int(42)))]);
    assert_eq!(result, Value::Int(42));
}

#[test]
fn prim_tuple_len_basic() {
    let src = "let f x = tuple_len x";
    let tuple = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = eval_direct(src, "f", &[tuple]);
    assert_eq!(result, Value::Int(3));
}

#[test]
fn prim_tuple_len_empty() {
    let src = "let f x = tuple_len x";
    let tuple = Value::tuple(vec![]);
    let result = eval_direct(src, "f", &[tuple]);
    assert_eq!(result, Value::Int(0));
}

#[test]
fn prim_tuple_len_int() {
    // Int(n) for n >= 0 is treated as a range of size n
    let src = "let f x = tuple_len x";
    let result = eval_direct(src, "f", &[Value::Int(5)]);
    assert_eq!(result, Value::Int(5));
}
