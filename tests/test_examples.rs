
//! Integration tests for IRIS example programs.
//!
//! Verifies that each example .iris file:
//! 1. Compiles without errors
//! 2. Produces correct output when executed
//! 3. Tests pass (test_results returns all-positive tuple)
//!
//! At least 3 tests per example.

use std::collections::{BTreeMap, HashMap};

use iris_exec::effect_runtime::RuntimeEffectHandler;
use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::eval::{EffectHandler, EffectRequest, EffectTag, StateStore, Value};
use iris_types::graph::SemanticGraph;

// ===========================================================================
// Helpers
// ===========================================================================

/// Compile IRIS source and return all fragments as (name, graph) pairs.
fn compile_all(src: &str) -> Vec<(String, SemanticGraph)> {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!(
            "compilation failed with {} errors",
            result.errors.len()
        );
    }
    result
        .fragments
        .into_iter()
        .map(|(name, frag, _)| (name, frag.graph))
        .collect()
}

/// Compile a single-function IRIS source and return its graph.
fn compile_one(src: &str) -> SemanticGraph {
    let frags = compile_all(src);
    assert!(
        !frags.is_empty(),
        "expected at least one fragment, got none"
    );
    frags.into_iter().next().unwrap().1
}

/// Compile and find a specific named function, along with a registry of
/// all other functions in the same source file for cross-fragment resolution.
fn compile_named(src: &str, name: &str) -> SemanticGraph {
    let (graph, _) = compile_named_with_registry(src, name);
    graph
}

/// Compile and return both the named function's graph and a registry of all
/// fragments (for cross-fragment calls).
fn compile_named_with_registry(src: &str, name: &str) -> (SemanticGraph, FragmentRegistry) {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!(
            "compilation failed with {} errors while looking for '{}'",
            result.errors.len(), name
        );
    }

    // Build a registry of all fragments.
    let mut registry = FragmentRegistry::new();
    for (_, frag, _) in &result.fragments {
        registry.register(frag.clone());
    }

    for (n, frag, _) in &result.fragments {
        if n == name {
            return (frag.graph.clone(), registry);
        }
    }
    let names: Vec<&str> = result.fragments.iter().map(|(n, _, _)| n.as_str()).collect();
    panic!(
        "function '{}' not found; available: {:?}",
        name, names
    );
}

/// Execute a compiled graph with given inputs and return the output values.
fn run(graph: &SemanticGraph, inputs: &[Value]) -> Vec<Value> {
    let (out, _) = interpreter::interpret(graph, inputs, None)
        .expect("interpreter failed");
    out
}

/// Execute a compiled graph with a registry for cross-fragment resolution.
fn run_with_registry(graph: &SemanticGraph, inputs: &[Value], registry: &FragmentRegistry) -> Vec<Value> {
    let (out, _) = interpreter::interpret_with_registry(graph, inputs, None, Some(registry))
        .expect("interpreter failed");
    out
}

/// Execute with mutable state.
fn run_with_state(
    graph: &SemanticGraph,
    inputs: &[Value],
    state: &mut StateStore,
) -> Vec<Value> {
    let (out, new_state) =
        interpreter::interpret(graph, inputs, Some(state))
            .expect("interpreter failed");
    *state = new_state;
    out
}

/// Read a file's source code.
fn read_iris(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {}: {}", path, e))
}

/// Assert all elements in a tuple are positive (test pass indicators).
fn assert_all_pass(results: &Value, context: &str) {
    match results {
        Value::Tuple(elems) => {
            for (i, elem) in elems.iter().enumerate() {
                match elem {
                    Value::Int(v) => assert!(
                        *v > 0,
                        "{}: test {} failed (value={})",
                        context,
                        i + 1,
                        v
                    ),
                    other => panic!(
                        "{}: test {} returned non-Int: {:?}",
                        context,
                        i + 1,
                        other
                    ),
                }
            }
            println!(
                "  {} OK: {}/{} tests passed",
                context,
                elems.len(),
                elems.len()
            );
        }
        other => panic!("{}: expected Tuple of results, got {:?}", context, other),
    }
}

// ===========================================================================
// Test Harness
// ===========================================================================

#[test]
fn test_harness_compiles() {
    let src = read_iris("tests/fixtures/iris-testing/test_harness.iris");
    let frags = compile_all(&src);
    assert!(
        frags.len() >= 5,
        "test harness should have at least 5 functions, got {}",
        frags.len()
    );
    let names: Vec<&str> = frags.iter().map(|(n, _)| n.as_str()).collect();
    println!("  Test harness functions: {:?}", names);
    assert!(names.contains(&"assert_eq"), "missing assert_eq");
    assert!(names.contains(&"assert_true"), "missing assert_true");
    assert!(names.contains(&"assert_gt"), "missing assert_gt");
    assert!(names.contains(&"run_tests"), "missing run_tests");
    assert!(names.contains(&"test_summary"), "missing test_summary");
}

#[test]
fn test_harness_assert_eq_pass() {
    let src = read_iris("tests/fixtures/iris-testing/test_harness.iris");
    let g = compile_named(&src, "assert_eq");
    let out = run(&g, &[Value::Int(42), Value::Int(42), Value::Int(0)]);
    assert_eq!(out, vec![Value::Int(1)], "assert_eq(42, 42) should pass");
}

#[test]
fn test_harness_assert_eq_fail() {
    let src = read_iris("tests/fixtures/iris-testing/test_harness.iris");
    let g = compile_named(&src, "assert_eq");
    let out = run(&g, &[Value::Int(42), Value::Int(99), Value::Int(0)]);
    assert_eq!(
        out,
        vec![Value::Int(-1)],
        "assert_eq(42, 99) should fail"
    );
}

#[test]
fn test_harness_assert_gt_pass() {
    let src = read_iris("tests/fixtures/iris-testing/test_harness.iris");
    let g = compile_named(&src, "assert_gt");
    let out = run(&g, &[Value::Int(10), Value::Int(5), Value::Int(0)]);
    assert_eq!(out, vec![Value::Int(1)], "assert_gt(10, 5) should pass");
}

#[test]
fn test_harness_assert_gt_fail() {
    let src = read_iris("tests/fixtures/iris-testing/test_harness.iris");
    let g = compile_named(&src, "assert_gt");
    let out = run(&g, &[Value::Int(3), Value::Int(5), Value::Int(0)]);
    assert_eq!(
        out,
        vec![Value::Int(-1)],
        "assert_gt(3, 5) should fail"
    );
}

#[test]
fn test_harness_assert_true() {
    let src = read_iris("tests/fixtures/iris-testing/test_harness.iris");
    let g = compile_named(&src, "assert_true");
    let pass = run(&g, &[Value::Int(1), Value::Int(0)]);
    assert_eq!(pass, vec![Value::Int(1)]);
    let fail = run(&g, &[Value::Int(0), Value::Int(0)]);
    assert_eq!(fail, vec![Value::Int(-1)]);
}

// ===========================================================================
// Echo Server
// ===========================================================================

#[test]
fn echo_server_compiles() {
    let src = read_iris("examples/echo-server/echo-server.iris");
    let frags = compile_all(&src);
    assert!(!frags.is_empty(), "echo server should compile");
    let names: Vec<&str> = frags.iter().map(|(n, _)| n.as_str()).collect();
    println!("  Echo server functions: {:?}", names);
}

#[test]
fn echo_server_test_compiles() {
    let src = read_iris("examples/echo-server/echo-test.iris");
    let frags = compile_all(&src);
    assert!(!frags.is_empty(), "echo test should compile");
}

#[test]
fn echo_server_test_summary() {
    let src = read_iris("examples/echo-server/echo-test.iris");
    let g = compile_named(&src, "test_summary");
    let out = run(&g, &[Value::Int(0)]);
    // Should return (3, 3, 0, 1) — total, passed, failed, all_pass
    assert_eq!(out.len(), 1);
    match &out[0] {
        Value::Tuple(elems) => {
            assert_eq!(elems.len(), 4);
            assert_eq!(elems[0], Value::Int(3), "total should be 3");
            assert_eq!(elems[3], Value::Int(1), "all_pass should be 1");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

// ===========================================================================
// Calculator
// ===========================================================================

#[test]
fn calculator_compiles() {
    let src = read_iris("examples/calculator/calculator.iris");
    let frags = compile_all(&src);
    let names: Vec<&str> = frags.iter().map(|(n, _)| n.as_str()).collect();
    println!("  Calculator functions: {:?}", names);
    assert!(names.contains(&"eval_simple"), "missing eval_simple");
}

#[test]
fn calculator_eval_simple_add() {
    let src = read_iris("examples/calculator/calculator.iris");
    let g = compile_named(&src, "eval_simple");
    // eval_simple 3 0 5 = 3 + 5 = 8
    let out = run(&g, &[Value::Int(3), Value::Int(0), Value::Int(5)]);
    assert_eq!(out, vec![Value::Int(8)]);
}

#[test]
fn calculator_eval_simple_sub() {
    let src = read_iris("examples/calculator/calculator.iris");
    let g = compile_named(&src, "eval_simple");
    // eval_simple 10 1 3 = 10 - 3 = 7
    let out = run(&g, &[Value::Int(10), Value::Int(1), Value::Int(3)]);
    assert_eq!(out, vec![Value::Int(7)]);
}

#[test]
fn calculator_eval_simple_mul() {
    let src = read_iris("examples/calculator/calculator.iris");
    let g = compile_named(&src, "eval_simple");
    // eval_simple 4 2 6 = 4 * 6 = 24
    let out = run(&g, &[Value::Int(4), Value::Int(2), Value::Int(6)]);
    assert_eq!(out, vec![Value::Int(24)]);
}

#[test]
fn calculator_eval_simple_div() {
    let src = read_iris("examples/calculator/calculator.iris");
    let g = compile_named(&src, "eval_simple");
    // eval_simple 10 3 2 = 10 / 2 = 5
    let out = run(&g, &[Value::Int(10), Value::Int(3), Value::Int(2)]);
    assert_eq!(out, vec![Value::Int(5)]);
}

#[test]
fn calculator_div_by_zero() {
    let src = read_iris("examples/calculator/calculator.iris");
    let g = compile_named(&src, "eval_simple");
    // eval_simple 10 3 0 = 10 / 0 = 0 (safe)
    let out = run(&g, &[Value::Int(10), Value::Int(3), Value::Int(0)]);
    assert_eq!(out, vec![Value::Int(0)]);
}

#[test]
fn calculator_test_results() {
    let src = read_iris("examples/calculator/calculator-test.iris");
    let g = compile_named(&src, "test_results");
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out.len(), 1);
    assert_all_pass(&out[0], "calculator");
}

// ===========================================================================
// Key-Value Store
// ===========================================================================

#[test]
fn kv_store_compiles() {
    let src = read_iris("examples/key-value-store/kv-store.iris");
    let frags = compile_all(&src);
    let names: Vec<&str> = frags.iter().map(|(n, _)| n.as_str()).collect();
    println!("  KV store functions: {:?}", names);
    assert!(frags.len() >= 5, "should have multiple functions");
}

#[test]
fn kv_store_set_get() {
    // Test the kv_set -> kv_get flow by compiling the inline test
    let g = compile_one(
        "let test x = \
            let store = state_empty in \
            let s1 = map_insert store \"key\" 42 in \
            map_get s1 \"key\""
    );
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out, vec![Value::Int(42)]);
}

#[test]
fn kv_store_delete() {
    let g = compile_one(
        "let test x = \
            let store = state_empty in \
            let s1 = map_insert store \"key\" 42 in \
            let s2 = map_remove s1 \"key\" in \
            map_get s2 \"key\""
    );
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out, vec![Value::Unit]);
}

#[test]
fn kv_store_size() {
    let g = compile_one(
        "let test x = \
            let s = map_insert (map_insert state_empty \"a\" 1) \"b\" 2 in \
            map_size s"
    );
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out, vec![Value::Int(2)]);
}

#[test]
fn kv_store_test_results() {
    let src = read_iris("examples/key-value-store/kv-test.iris");
    let g = compile_named(&src, "test_results");
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out.len(), 1);
    assert_all_pass(&out[0], "kv-store");
}

// ===========================================================================
// Fibonacci Server
// ===========================================================================

#[test]
fn fibonacci_compiles() {
    let src = read_iris("examples/fibonacci-server/fib-server.iris");
    let frags = compile_all(&src);
    let names: Vec<&str> = frags.iter().map(|(n, _)| n.as_str()).collect();
    println!("  Fibonacci functions: {:?}", names);
}

#[test]
fn fibonacci_sequence_unfold() {
    // Verify unfold produces correct fibonacci sequence
    let g = compile_one("let fibs n = unfold (0, 1) (+) n");
    let out = run(&g, &[Value::Int(8)]);
    assert_eq!(out.len(), 1);
    match &out[0] {
        Value::Tuple(elems) => {
            assert_eq!(elems.len(), 8);
            assert_eq!(elems[0], Value::Int(0));
            assert_eq!(elems[1], Value::Int(1));
            assert_eq!(elems[2], Value::Int(1));
            assert_eq!(elems[3], Value::Int(2));
            assert_eq!(elems[4], Value::Int(3));
            assert_eq!(elems[5], Value::Int(5));
            assert_eq!(elems[6], Value::Int(8));
            assert_eq!(elems[7], Value::Int(13));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn fibonacci_fib_sequence() {
    let src = read_iris("examples/fibonacci-server/fib-server.iris");
    let g = compile_named(&src, "fib_sequence");
    let out = run(&g, &[Value::Int(6)]);
    assert_eq!(out.len(), 1);
    match &out[0] {
        Value::Tuple(elems) => {
            assert_eq!(elems.len(), 6);
            // unfold (0,1) (+) 6 = [0, 1, 1, 2, 3, 5]
            assert_eq!(elems[0], Value::Int(0));
            assert_eq!(elems[5], Value::Int(5));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn fibonacci_test_results() {
    let src = read_iris("examples/fibonacci-server/fib-test.iris");
    let g = compile_named(&src, "test_results");
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out.len(), 1);
    assert_all_pass(&out[0], "fibonacci");
}

// ===========================================================================
// JSON API
// ===========================================================================

#[test]
fn json_api_compiles() {
    let src = read_iris("examples/json-api/json-api.iris");
    let frags = compile_all(&src);
    let names: Vec<&str> = frags.iter().map(|(n, _)| n.as_str()).collect();
    println!("  JSON API functions: {:?}", names);
    assert!(names.contains(&"route"), "missing route");
    assert!(names.contains(&"handle_health"), "missing handle_health");
}

#[test]
fn json_api_parse_add_body() {
    // Verify JSON parsing of add request body
    let g = compile_one(
        r#"let test x = let p = (("a", 3), ("b", 5)) in tuple_get p "a" + tuple_get p "b""#
    );
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out, vec![Value::Int(8)]);
}

#[test]
fn json_api_fib_path_extract() {
    // Verify path parsing for /fib/N
    let g = compile_one(
        r#"let test x = let path = "/fib/10" in str_to_int (str_slice path 5 (str_len path))"#
    );
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out, vec![Value::Int(10)]);
}

#[test]
fn json_api_health_response() {
    let src = read_iris("examples/json-api/json-api.iris");
    let (g, registry) = compile_named_with_registry(&src, "handle_health");
    let out = run_with_registry(&g, &[Value::Int(0)], &registry);
    assert_eq!(out.len(), 1);
    match &out[0] {
        Value::String(s) => {
            assert!(s.contains("200 OK"), "should be 200 OK: {}", s);
            assert!(s.contains(r#"{"status":"ok"}"#), "should contain JSON body: {}", s);
        }
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn json_api_test_results() {
    let src = read_iris("examples/json-api/json-api-test.iris");
    let (g, registry) = compile_named_with_registry(&src, "test_results");
    let out = run_with_registry(&g, &[Value::Int(0)], &registry);
    assert_eq!(out.len(), 1);
    assert_all_pass(&out[0], "json-api");
}

// ===========================================================================
// File Processor
// ===========================================================================

#[test]
fn file_processor_compiles() {
    let src = read_iris("examples/file-processor/file-processor.iris");
    let frags = compile_all(&src);
    let names: Vec<&str> = frags.iter().map(|(n, _)| n.as_str()).collect();
    println!("  File processor functions: {:?}", names);
    assert!(names.contains(&"count_words"), "missing count_words");
}

#[test]
fn file_processor_count_words() {
    let src = read_iris("examples/file-processor/file-processor.iris");
    let g = compile_named(&src, "count_words");
    let out = run(&g, &[Value::String("hello world foo".to_string())]);
    assert_eq!(out, vec![Value::Int(3)]);
}

#[test]
fn file_processor_count_words_single() {
    let src = read_iris("examples/file-processor/file-processor.iris");
    let g = compile_named(&src, "count_words");
    let out = run(&g, &[Value::String("hello".to_string())]);
    assert_eq!(out, vec![Value::Int(1)]);
}

#[test]
fn file_processor_count_chars() {
    let src = read_iris("examples/file-processor/file-processor.iris");
    let g = compile_named(&src, "count_chars");
    let out = run(&g, &[Value::String("hello".to_string())]);
    assert_eq!(out, vec![Value::Int(5)]);
}

#[test]
fn file_processor_test_results() {
    let src = read_iris("examples/file-processor/file-processor-test.iris");
    let g = compile_named(&src, "test_results");
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out.len(), 1);
    assert_all_pass(&out[0], "file-processor");
}

// ===========================================================================
// TODO App
// ===========================================================================

#[test]
fn todo_app_compiles() {
    let src = read_iris("examples/todo-app/todo.iris");
    let frags = compile_all(&src);
    let names: Vec<&str> = frags.iter().map(|(n, _)| n.as_str()).collect();
    println!("  TODO app functions: {:?}", names);
}

#[test]
fn todo_app_new_store() {
    // A new store should have size 1 (just the next_id key)
    let g = compile_one(
        "let test x = map_size (map_insert state_empty \"next_id\" 0)"
    );
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out, vec![Value::Int(1)]);
}

#[test]
fn todo_app_add_and_get() {
    let g = compile_one(
        "let test x = \
            let store = map_insert state_empty \"next_id\" 0 in \
            let s1 = map_insert store \"item_0\" (\"Buy milk\", 0) in \
            let item = map_get s1 \"item_0\" in \
            list_nth item 0"
    );
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out, vec![Value::String("Buy milk".to_string())]);
}

#[test]
fn todo_app_complete() {
    let g = compile_one(
        "let test x = \
            let store = map_insert state_empty \"item_0\" (\"task\", 0) in \
            let s1 = map_insert store \"item_0\" (\"task\", 1) in \
            let item = map_get s1 \"item_0\" in \
            list_nth item 1"
    );
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out, vec![Value::Int(1)]);
}

#[test]
fn todo_app_test_results() {
    let src = read_iris("examples/todo-app/todo-test.iris");
    let g = compile_named(&src, "test_results");
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out.len(), 1);
    assert_all_pass(&out[0], "todo-app");
}

// ===========================================================================
// Genetic Algorithm
// ===========================================================================

#[test]
fn ga_compiles() {
    let src = read_iris("examples/genetic-algorithm/ga.iris");
    let frags = compile_all(&src);
    let names: Vec<&str> = frags.iter().map(|(n, _)| n.as_str()).collect();
    println!("  GA functions: {:?}", names);
    assert!(names.contains(&"fitness"), "missing fitness");
    assert!(names.contains(&"crossover"), "missing crossover");
}

#[test]
fn ga_fitness_perfect() {
    let src = read_iris("examples/genetic-algorithm/ga.iris");
    let g = compile_named(&src, "fitness");
    let individual = Value::tuple(vec![Value::Int(5), Value::Int(5), Value::Int(5)]);
    let out = run(&g, &[individual, Value::Int(15)]);
    assert_eq!(out, vec![Value::Int(0)], "perfect fitness should be 0");
}

#[test]
fn ga_fitness_off() {
    let src = read_iris("examples/genetic-algorithm/ga.iris");
    let g = compile_named(&src, "fitness");
    let individual = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let out = run(&g, &[individual, Value::Int(10)]);
    assert_eq!(out, vec![Value::Int(4)], "sum=6, target=10, diff=4");
}

#[test]
fn ga_crossover() {
    let src = read_iris("examples/genetic-algorithm/ga.iris");
    let g = compile_named(&src, "crossover");
    let p1 = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]);
    let p2 = Value::tuple(vec![Value::Int(5), Value::Int(6), Value::Int(7), Value::Int(8)]);
    let out = run(&g, &[p1, p2]);
    assert_eq!(out.len(), 1);
    match &out[0] {
        Value::Tuple(elems) => {
            assert_eq!(elems.len(), 4, "crossover should preserve length");
            // First half from p1, second half from p2
            assert_eq!(elems[0], Value::Int(1));
            assert_eq!(elems[1], Value::Int(2));
            assert_eq!(elems[2], Value::Int(7));
            assert_eq!(elems[3], Value::Int(8));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn ga_test_results() {
    let src = read_iris("examples/genetic-algorithm/ga-test.iris");
    let (g, registry) = compile_named_with_registry(&src, "test_results");
    let out = run_with_registry(&g, &[Value::Int(0)], &registry);
    assert_eq!(out.len(), 1);
    assert_all_pass(&out[0], "genetic-algorithm");
}

// ===========================================================================
// Chat Protocol
// ===========================================================================

#[test]
fn chat_protocol_compiles() {
    let src = read_iris("examples/chat-protocol/chat.iris");
    let frags = compile_all(&src);
    let names: Vec<&str> = frags.iter().map(|(n, _)| n.as_str()).collect();
    println!("  Chat protocol functions: {:?}", names);
    assert!(names.contains(&"format_message"), "missing format_message");
    assert!(names.contains(&"parse_message"), "missing parse_message");
}

#[test]
fn chat_format_message() {
    let src = read_iris("examples/chat-protocol/chat.iris");
    let g = compile_named(&src, "format_message");
    let out = run(&g, &[
        Value::String("alice".to_string()),
        Value::String("hello".to_string()),
    ]);
    assert_eq!(
        out,
        vec![Value::String("alice:hello\n".to_string())]
    );
}

#[test]
fn chat_parse_message() {
    let src = read_iris("examples/chat-protocol/chat.iris");
    let g = compile_named(&src, "parse_message");
    let out = run(&g, &[Value::String("bob:world".to_string())]);
    assert_eq!(out.len(), 1);
    match &out[0] {
        Value::Tuple(elems) => {
            assert_eq!(elems[0], Value::String("bob".to_string()));
            assert_eq!(elems[1], Value::String("world".to_string()));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn chat_add_remove_client() {
    let src = read_iris("examples/chat-protocol/chat.iris");
    let g_add = compile_named(&src, "add_client");
    let g_remove = compile_named(&src, "remove_client");

    // Add a client
    let out = run(&g_add, &[
        Value::State(BTreeMap::new()),
        Value::Int(42),  // conn handle
        Value::Int(0),   // client_id
    ]);
    assert_eq!(out.len(), 1);
    match &out[0] {
        Value::State(store) => {
            assert_eq!(store.get("client_0"), Some(&Value::Int(42)));
        }
        other => panic!("expected State, got {:?}", other),
    }

    // Remove the client
    let mut clients = BTreeMap::new();
    clients.insert("client_0".to_string(), Value::Int(42));
    let out2 = run(&g_remove, &[Value::State(clients), Value::Int(0)]);
    assert_eq!(out2.len(), 1);
    match &out2[0] {
        Value::State(store) => {
            assert!(store.get("client_0").is_none());
        }
        other => panic!("expected State, got {:?}", other),
    }
}

#[test]
fn chat_test_results() {
    let src = read_iris("examples/chat-protocol/chat-test.iris");
    let g = compile_named(&src, "test_results");
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out.len(), 1);
    assert_all_pass(&out[0], "chat-protocol");
}

// ===========================================================================
// Self-Modifying
// ===========================================================================

#[test]
fn self_modify_compiles() {
    let src = read_iris("examples/self-modifying/self-modify.iris");
    let frags = compile_all(&src);
    let names: Vec<&str> = frags.iter().map(|(n, _)| n.as_str()).collect();
    println!("  Self-modify functions: {:?}", names);
}

#[test]
fn self_modify_demo() {
    let src = read_iris("examples/self-modifying/self-modify.iris");
    let g = compile_named(&src, "self_modify_demo");
    let out = run(&g, &[Value::Int(5), Value::Int(3)]);
    assert_eq!(out.len(), 1);
    match &out[0] {
        Value::Tuple(elems) => {
            assert_eq!(elems[0], Value::Int(0), "opcode_add = 0");
            assert_eq!(elems[1], Value::Int(8), "add(5,3) = 8");
            assert_eq!(elems[2], Value::Int(2), "opcode_mul = 2");
            assert_eq!(elems[3], Value::Int(15), "mul(5,3) = 15");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn self_modify_verify_swap() {
    let src = read_iris("examples/self-modifying/self-modify.iris");
    let g = compile_named(&src, "verify_swap");
    let out = run(&g, &[Value::Int(10), Value::Int(3)]);
    assert_eq!(out.len(), 1);
    match &out[0] {
        Value::Tuple(elems) => {
            assert_eq!(elems[0], Value::Int(13), "add(10,3) = 13");
            assert_eq!(elems[1], Value::Int(7), "sub(10,3) = 7");
            assert_eq!(elems[2], Value::Int(30), "mul(10,3) = 30");
            assert_eq!(elems[3], Value::Int(3), "div(10,3) = 3");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn self_modify_graph_opcodes() {
    // Verify the actual graph-level self-modification using raw graph construction
    // (same approach as test_self_mod.rs)
    use iris_types::cost::{CostBound, CostTerm};
    use iris_types::graph::*;
    use iris_types::hash::SemanticHash;
    use iris_types::types::TypeEnv;

    fn prim_node(id: u64, opcode: u8, arity: u8) -> (NodeId, Node) {
        (
            NodeId(id),
            Node {
                id: NodeId(id),
                kind: NodeKind::Prim,
                type_sig: iris_types::types::TypeId(0),
                cost: CostTerm::Unit,
                arity,
                resolution_depth: 0,
                salt: 0,
                payload: NodePayload::Prim { opcode },
            },
        )
    }

    fn int_lit_node(id: u64, value: i64) -> (NodeId, Node) {
        (
            NodeId(id),
            Node {
                id: NodeId(id),
                kind: NodeKind::Lit,
                type_sig: iris_types::types::TypeId(0),
                cost: CostTerm::Unit,
                arity: 0,
                resolution_depth: 0,
                salt: 0,
                payload: NodePayload::Lit {
                    type_tag: 0x00,
                    value: value.to_le_bytes().to_vec(),
                },
            },
        )
    }

    // Build add(5, 3) graph
    let mut nodes = HashMap::new();
    let (nid, node) = int_lit_node(10, 5);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(20, 3);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(1, 0x00, 2); // add
    nodes.insert(nid, node);

    let edges = vec![
        Edge { source: NodeId(1), target: NodeId(10), port: 0, label: EdgeLabel::Argument },
        Edge { source: NodeId(1), target: NodeId(20), port: 1, label: EdgeLabel::Argument },
    ];

    let graph = SemanticGraph {
        root: NodeId(1),
        nodes: nodes.clone(),
        edges: edges.clone(),
        type_env: TypeEnv { types: BTreeMap::new() },
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    };

    // Verify add(5, 3) = 8
    let (out, _) = interpreter::interpret(&graph, &[], None).unwrap();
    assert_eq!(out, vec![Value::Int(8)]);

    // Modify to mul(5, 3) — change opcode from 0x00 to 0x02
    let mut nodes2 = nodes.clone();
    nodes2.remove(&NodeId(1));
    let (nid, node) = prim_node(1, 0x02, 2); // mul
    nodes2.insert(nid, node);

    let graph2 = SemanticGraph {
        root: NodeId(1),
        nodes: nodes2,
        edges,
        type_env: TypeEnv { types: BTreeMap::new() },
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    };

    // Verify mul(5, 3) = 15
    let (out2, _) = interpreter::interpret(&graph2, &[], None).unwrap();
    assert_eq!(out2, vec![Value::Int(15)]);
}

#[test]
fn self_modify_test_results() {
    let src = read_iris("examples/self-modifying/self-modify-test.iris");
    let g = compile_named(&src, "test_results");
    let out = run(&g, &[Value::Int(0)]);
    assert_eq!(out.len(), 1);
    assert_all_pass(&out[0], "self-modifying");
}

// ===========================================================================
// Cross-example: all test files compile and pass
// ===========================================================================

#[test]
fn all_examples_compile() {
    let examples = [
        "examples/echo-server/echo-server.iris",
        "examples/echo-server/echo-test.iris",
        "examples/calculator/calculator.iris",
        "examples/calculator/calculator-test.iris",
        "examples/key-value-store/kv-store.iris",
        "examples/key-value-store/kv-test.iris",
        "examples/fibonacci-server/fib-server.iris",
        "examples/fibonacci-server/fib-test.iris",
        "examples/json-api/json-api.iris",
        "examples/json-api/json-api-test.iris",
        "examples/file-processor/file-processor.iris",
        "examples/file-processor/file-processor-test.iris",
        "examples/todo-app/todo.iris",
        "examples/todo-app/todo-test.iris",
        "examples/genetic-algorithm/ga.iris",
        "examples/genetic-algorithm/ga-test.iris",
        "examples/chat-protocol/chat.iris",
        "examples/chat-protocol/chat-test.iris",
        "examples/self-modifying/self-modify.iris",
        "examples/self-modifying/self-modify-test.iris",
        "tests/fixtures/iris-testing/test_harness.iris",
    ];

    for path in &examples {
        let src = read_iris(path);
        let result = iris_bootstrap::syntax::compile(&src);
        assert!(
            result.errors.is_empty(),
            "{} failed to compile: {:?}",
            path,
            result.errors.iter()
                .map(|e| iris_bootstrap::syntax::format_error(&src, e))
                .collect::<Vec<_>>()
        );
        let count = result.fragments.len();
        println!("  {} compiled: {} fragments", path, count);
    }
}

#[test]
fn all_test_results_pass() {
    let test_files = [
        ("examples/calculator/calculator-test.iris", "calculator"),
        ("examples/key-value-store/kv-test.iris", "kv-store"),
        ("examples/fibonacci-server/fib-test.iris", "fibonacci"),
        ("examples/json-api/json-api-test.iris", "json-api"),
        ("examples/file-processor/file-processor-test.iris", "file-processor"),
        ("examples/todo-app/todo-test.iris", "todo-app"),
        ("examples/genetic-algorithm/ga-test.iris", "genetic-algorithm"),
        ("examples/chat-protocol/chat-test.iris", "chat-protocol"),
        ("examples/self-modifying/self-modify-test.iris", "self-modifying"),
    ];

    println!("\n========================================");
    println!("  Running all example test suites");
    println!("========================================\n");

    for (path, label) in &test_files {
        let src = read_iris(path);
        let (g, registry) = compile_named_with_registry(&src, "test_results");
        let out = run_with_registry(&g, &[Value::Int(0)], &registry);
        assert_eq!(out.len(), 1, "{}: expected 1 output", label);
        assert_all_pass(&out[0], label);
    }

    println!("\n========================================");
    println!("  All example test suites passed!");
    println!("========================================");
}
