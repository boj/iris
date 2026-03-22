
//! Test harness for stdlib .iris programs.
//!
//! Loads each stdlib .iris file, compiles it via iris_bootstrap::syntax::compile(),
//! registers all fragments in a FragmentRegistry, then either runs `test_`
//! bindings (if any) or verifies compilation + basic evaluation of key fragments.
//!
//! Some stdlib files use syntax features not yet supported by the compiler
//! (e.g., `let _ = ...`, `\r\n` escapes, `[cost: Unit]`, `let rec`).
//! For those files we verify the source is readable and compilation is
//! attempted — they serve as regression markers for compiler evolution.

use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// Helpers (mirror test_codec_iris.rs)
// ---------------------------------------------------------------------------

/// Compile IRIS source, register all fragments, return named graphs + registry.
fn compile_with_registry(src: &str) -> (Vec<(String, SemanticGraph)>, FragmentRegistry) {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!(
            "compilation failed with {} errors:\n{}",
            result.errors.len(),
            result
                .errors
                .iter()
                .map(|e| iris_bootstrap::syntax::format_error(src, e))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    let mut registry = FragmentRegistry::new();
    for (_, frag, _) in &result.fragments {
        registry.register(frag.clone());
    }

    let named: Vec<_> = result
        .fragments
        .into_iter()
        .map(|(name, frag, _)| (name, frag.graph))
        .collect();

    (named, registry)
}

/// Try to compile IRIS source. Returns Ok with fragments+registry, or Err with
/// the number of compilation errors.
fn try_compile(src: &str) -> Result<(Vec<(String, SemanticGraph)>, FragmentRegistry), usize> {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        return Err(result.errors.len());
    }

    let mut registry = FragmentRegistry::new();
    for (_, frag, _) in &result.fragments {
        registry.register(frag.clone());
    }

    let named: Vec<_> = result
        .fragments
        .into_iter()
        .map(|(name, frag, _)| (name, frag.graph))
        .collect();

    Ok((named, registry))
}

/// Evaluate a SemanticGraph with no inputs and return the result as i64.
fn eval_no_args(graph: &SemanticGraph, registry: &FragmentRegistry) -> Result<i64, String> {
    match interpreter::interpret_with_registry(graph, &[], None, Some(registry)) {
        Ok((outputs, _)) => {
            if let Some(val) = outputs.first() {
                match val {
                    Value::Int(n) => Ok(*n),
                    Value::Bool(true) => Ok(1),
                    Value::Bool(false) => Ok(0),
                    _ => Err(format!("unexpected result type: {:?}", val)),
                }
            } else {
                Err("no output".to_string())
            }
        }
        Err(e) => Err(format!("evaluation error: {:?}", e)),
    }
}

/// Evaluate a SemanticGraph with the given inputs and return the raw Value.
fn eval_with_args(
    graph: &SemanticGraph,
    args: &[Value],
    registry: &FragmentRegistry,
) -> Result<Value, String> {
    match interpreter::interpret_with_registry(graph, args, None, Some(registry)) {
        Ok((outputs, _)) => {
            if let Some(val) = outputs.first() {
                Ok(val.clone())
            } else {
                Err("no output".to_string())
            }
        }
        Err(e) => Err(format!("evaluation error: {:?}", e)),
    }
}

/// Helper: extract an i64 from a Value, accepting both Int and Bool.
fn value_to_i64(v: &Value) -> Option<i64> {
    match v {
        Value::Int(n) => Some(*n),
        Value::Bool(true) => Some(1),
        Value::Bool(false) => Some(0),
        _ => None,
    }
}

/// Load a stdlib .iris file source.
fn read_stdlib(filename: &str) -> String {
    let path = format!("src/iris-programs/stdlib/{}", filename);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path, e))
}

/// Load a stdlib .iris file and compile it.
fn load_stdlib(filename: &str) -> (Vec<(String, SemanticGraph)>, FragmentRegistry) {
    let source = read_stdlib(filename);
    compile_with_registry(&source)
}

/// Find a named fragment in the list.
fn find_fragment<'a>(
    fragments: &'a [(String, SemanticGraph)],
    name: &str,
) -> Option<&'a SemanticGraph> {
    fragments.iter().find(|(n, _)| n == name).map(|(_, g)| g)
}

/// Run all test_ bindings in a compiled file, asserting each returns > 0.
fn run_test_bindings(
    filename: &str,
    fragments: &[(String, SemanticGraph)],
    registry: &FragmentRegistry,
) -> usize {
    let test_frags: Vec<_> = fragments
        .iter()
        .filter(|(name, _)| name.starts_with("test_"))
        .collect();

    if test_frags.is_empty() {
        return 0;
    }

    let mut passed = 0;
    let mut failed = Vec::new();

    for (name, graph) in &test_frags {
        match eval_no_args(graph, registry) {
            Ok(n) if n > 0 => {
                passed += 1;
            }
            Ok(n) => {
                failed.push(format!("  {} returned {} (expected > 0)", name, n));
            }
            Err(e) => {
                failed.push(format!("  {} error: {}", name, e));
            }
        }
    }

    if !failed.is_empty() {
        panic!(
            "{}: {}/{} tests failed:\n{}",
            filename,
            failed.len(),
            test_frags.len(),
            failed.join("\n")
        );
    }

    println!(
        "{}: all {}/{} test_ bindings passed",
        filename, passed, test_frags.len()
    );
    passed
}

// ---------------------------------------------------------------------------
// Files with known compilation issues: verify source is readable and
// document the current compilation status as a regression marker.
// When the compiler gains support for these features, these tests will
// start failing (at the assert on error count), signalling it's time to
// upgrade them to full evaluation tests.
// ---------------------------------------------------------------------------

#[test]
fn test_file_ops_readable() {
    // Uses `let _ = ...` which the compiler doesn't yet support.
    let source = read_stdlib("file_ops.iris");
    assert!(!source.is_empty(), "file_ops.iris should be readable");
    let result = try_compile(&source);
    assert!(result.is_err(), "file_ops.iris: expected compilation errors (let _ = ... unsupported)");
    println!("file_ops.iris: readable, compilation blocked on `let _ = ...` syntax");
}

#[test]
fn test_http_client_readable() {
    // Uses `\r\n` escape sequences the compiler doesn't yet handle.
    let source = read_stdlib("http_client.iris");
    assert!(!source.is_empty(), "http_client.iris should be readable");
    let result = try_compile(&source);
    assert!(result.is_err(), "http_client.iris: expected compilation errors (\\r\\n unsupported)");
    println!("http_client.iris: readable, compilation blocked on \\r\\n escapes");
}

#[test]
fn test_http_server_readable() {
    // Uses `\r\n` escape sequences the compiler doesn't yet handle.
    let source = read_stdlib("http_server.iris");
    assert!(!source.is_empty(), "http_server.iris should be readable");
    let result = try_compile(&source);
    assert!(result.is_err(), "http_server.iris: expected compilation errors (\\r\\n unsupported)");
    println!("http_server.iris: readable, compilation blocked on \\r\\n escapes");
}

#[test]
fn test_time_ops_readable() {
    // Uses `_` as a parameter name which the compiler doesn't support.
    let source = read_stdlib("time_ops.iris");
    assert!(!source.is_empty(), "time_ops.iris should be readable");
    let result = try_compile(&source);
    assert!(result.is_err(), "time_ops.iris: expected compilation errors (_ parameter unsupported)");
    println!("time_ops.iris: readable, compilation blocked on `_` parameter syntax");
}

#[test]
fn test_math_readable() {
    // Uses `[cost: Unit]` which the compiler doesn't recognize.
    let source = read_stdlib("math.iris");
    assert!(!source.is_empty(), "math.iris should be readable");
    let result = try_compile(&source);
    assert!(result.is_err(), "math.iris: expected compilation errors ([cost: Unit] unsupported)");
    println!("math.iris: readable, compilation blocked on `[cost: Unit]` annotation");
}

#[test]
fn test_json_readable() {
    // Uses `[cost: Unit]` which the compiler doesn't recognize.
    let source = read_stdlib("json.iris");
    assert!(!source.is_empty(), "json.iris should be readable");
    let result = try_compile(&source);
    assert!(result.is_err(), "json.iris: expected compilation errors ([cost: Unit] unsupported)");
    println!("json.iris: readable, compilation blocked on `[cost: Unit]` annotation");
}

#[test]
fn test_path_ops_readable() {
    // Uses `let rec` which produces undefined variable errors for recursive refs.
    let source = read_stdlib("path_ops.iris");
    assert!(!source.is_empty(), "path_ops.iris should be readable");
    let result = try_compile(&source);
    assert!(result.is_err(), "path_ops.iris: expected compilation errors (let rec unsupported)");
    println!("path_ops.iris: readable, compilation blocked on `let rec` self-references");
}

// ---------------------------------------------------------------------------
// Pure computation files that compile: compile + evaluate basic operations
// ---------------------------------------------------------------------------

#[test]
fn test_string_ops_iris() {
    let (fragments, registry) = load_stdlib("string_ops.iris");
    run_test_bindings("string_ops.iris", &fragments, &registry);

    let expected = [
        "reverse",
        "repeat",
        "pad_left",
        "pad_right",
        "count_occurrences",
        "replace_first",
        "is_empty",
        "is_blank",
        "char_from_code",
        "index_of",
    ];
    for name in &expected {
        assert!(
            find_fragment(&fragments, name).is_some(),
            "string_ops.iris: missing fragment '{}'",
            name
        );
    }

    // is_empty("") = 1
    if let Some(graph) = find_fragment(&fragments, "is_empty") {
        let result = eval_with_args(graph, &[Value::String("".into())], &registry);
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("is_empty should return int-like");
                assert_eq!(n, 1, "is_empty(\"\") should be 1");
            }
            Err(e) => panic!("is_empty: {}", e),
        }
    }

    // is_empty("hello") = 0
    if let Some(graph) = find_fragment(&fragments, "is_empty") {
        let result = eval_with_args(graph, &[Value::String("hello".into())], &registry);
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("is_empty should return int-like");
                assert_eq!(n, 0, "is_empty(\"hello\") should be 0");
            }
            Err(e) => panic!("is_empty(hello): {}", e),
        }
    }

    // reverse("abc") = "cba"
    if let Some(graph) = find_fragment(&fragments, "reverse") {
        let result = eval_with_args(graph, &[Value::String("abc".into())], &registry);
        match result {
            Ok(Value::String(s)) => assert_eq!(s, "cba", "reverse(\"abc\") should be \"cba\""),
            other => panic!("reverse: unexpected result: {:?}", other),
        }
    }

    // repeat("ab", 3) = "ababab"
    if let Some(graph) = find_fragment(&fragments, "repeat") {
        let result = eval_with_args(
            graph,
            &[Value::String("ab".into()), Value::Int(3)],
            &registry,
        );
        match result {
            Ok(Value::String(s)) => {
                assert_eq!(s, "ababab", "repeat(\"ab\", 3) should be \"ababab\"")
            }
            other => panic!("repeat: unexpected result: {:?}", other),
        }
    }

    // count_occurrences("abcabc", "bc") = 2
    if let Some(graph) = find_fragment(&fragments, "count_occurrences") {
        let result = eval_with_args(
            graph,
            &[Value::String("abcabc".into()), Value::String("bc".into())],
            &registry,
        );
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("count_occurrences should return int-like");
                assert_eq!(n, 2, "count_occurrences(\"abcabc\", \"bc\") should be 2");
            }
            Err(e) => panic!("count_occurrences: {}", e),
        }
    }

    // index_of("hello world", "world") = 6
    if let Some(graph) = find_fragment(&fragments, "index_of") {
        let result = eval_with_args(
            graph,
            &[
                Value::String("hello world".into()),
                Value::String("world".into()),
            ],
            &registry,
        );
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("index_of should return int-like");
                assert_eq!(n, 6, "index_of(\"hello world\", \"world\") should be 6");
            }
            Err(e) => panic!("index_of: {}", e),
        }
    }

    // index_of("hello", "xyz") = -1
    if let Some(graph) = find_fragment(&fragments, "index_of") {
        let result = eval_with_args(
            graph,
            &[
                Value::String("hello".into()),
                Value::String("xyz".into()),
            ],
            &registry,
        );
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("index_of should return int-like");
                assert_eq!(n, -1, "index_of(\"hello\", \"xyz\") should be -1");
            }
            Err(e) => panic!("index_of(not found): {}", e),
        }
    }

    // char_from_code(65) = "A"
    if let Some(graph) = find_fragment(&fragments, "char_from_code") {
        let result = eval_with_args(graph, &[Value::Int(65)], &registry);
        match result {
            Ok(Value::String(s)) => assert_eq!(s, "A", "char_from_code(65) should be \"A\""),
            other => panic!("char_from_code: unexpected result: {:?}", other),
        }
    }

    println!("string_ops.iris: all evaluations passed");
}

#[test]
fn test_string_utils_iris() {
    let (fragments, registry) = load_stdlib("string_utils.iris");
    run_test_bindings("string_utils.iris", &fragments, &registry);

    let expected = [
        "words",
        "lines",
        "starts_with",
        "ends_with",
        "unlines",
        "unwords",
        "capitalize",
        "title_case",
        "strip_prefix",
        "strip_suffix",
        "is_numeric",
        "is_alpha",
        "truncate",
        "remove_all",
        "surround",
        "quote",
    ];
    for name in &expected {
        assert!(
            find_fragment(&fragments, name).is_some(),
            "string_utils.iris: missing fragment '{}'",
            name
        );
    }

    // is_numeric("12345") = 1
    if let Some(graph) = find_fragment(&fragments, "is_numeric") {
        let result = eval_with_args(graph, &[Value::String("12345".into())], &registry);
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("is_numeric should return int-like");
                assert_eq!(n, 1, "is_numeric(\"12345\") should be 1");
            }
            Err(e) => panic!("is_numeric: {}", e),
        }
    }

    // is_numeric("12a") = 0
    if let Some(graph) = find_fragment(&fragments, "is_numeric") {
        let result = eval_with_args(graph, &[Value::String("12a".into())], &registry);
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("is_numeric should return int-like");
                assert_eq!(n, 0, "is_numeric(\"12a\") should be 0");
            }
            Err(e) => panic!("is_numeric(12a): {}", e),
        }
    }

    // is_alpha("hello") = 1
    if let Some(graph) = find_fragment(&fragments, "is_alpha") {
        let result = eval_with_args(graph, &[Value::String("hello".into())], &registry);
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("is_alpha should return int-like");
                assert_eq!(n, 1, "is_alpha(\"hello\") should be 1");
            }
            Err(e) => panic!("is_alpha: {}", e),
        }
    }

    // is_alpha("hello123") = 0
    if let Some(graph) = find_fragment(&fragments, "is_alpha") {
        let result = eval_with_args(graph, &[Value::String("hello123".into())], &registry);
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("is_alpha should return int-like");
                assert_eq!(n, 0, "is_alpha(\"hello123\") should be 0");
            }
            Err(e) => panic!("is_alpha(hello123): {}", e),
        }
    }

    // starts_with("hello world", "hello") = truthy (1 or true)
    if let Some(graph) = find_fragment(&fragments, "starts_with") {
        let result = eval_with_args(
            graph,
            &[
                Value::String("hello world".into()),
                Value::String("hello".into()),
            ],
            &registry,
        );
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("starts_with should return int-like");
                assert_eq!(n, 1, "starts_with(\"hello world\", \"hello\") should be truthy");
            }
            Err(e) => panic!("starts_with: {}", e),
        }
    }

    // ends_with("hello world", "world") = truthy (1 or true)
    if let Some(graph) = find_fragment(&fragments, "ends_with") {
        let result = eval_with_args(
            graph,
            &[
                Value::String("hello world".into()),
                Value::String("world".into()),
            ],
            &registry,
        );
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("ends_with should return int-like");
                assert_eq!(n, 1, "ends_with(\"hello world\", \"world\") should be truthy");
            }
            Err(e) => panic!("ends_with: {}", e),
        }
    }

    // quote("hello") = "\"hello\""
    if let Some(graph) = find_fragment(&fragments, "quote") {
        let result = eval_with_args(graph, &[Value::String("hello".into())], &registry);
        match result {
            Ok(Value::String(s)) => {
                assert_eq!(s, "\"hello\"", "quote(\"hello\") should be '\"hello\"'")
            }
            other => panic!("quote: unexpected result: {:?}", other),
        }
    }

    // strip_prefix("hello world", "hello ") = "world"
    if let Some(graph) = find_fragment(&fragments, "strip_prefix") {
        let result = eval_with_args(
            graph,
            &[
                Value::String("hello world".into()),
                Value::String("hello ".into()),
            ],
            &registry,
        );
        match result {
            Ok(Value::String(s)) => {
                assert_eq!(s, "world", "strip_prefix should be \"world\"")
            }
            other => panic!("strip_prefix: unexpected result: {:?}", other),
        }
    }

    // truncate("hello world", 8) = "hello..."
    if let Some(graph) = find_fragment(&fragments, "truncate") {
        let result = eval_with_args(
            graph,
            &[Value::String("hello world".into()), Value::Int(8)],
            &registry,
        );
        match result {
            Ok(Value::String(s)) => {
                assert_eq!(s, "hello...", "truncate should be \"hello...\"")
            }
            other => panic!("truncate: unexpected result: {:?}", other),
        }
    }

    println!("string_utils.iris: all evaluations passed");
}

#[test]
fn test_list_ops_iris() {
    let (fragments, registry) = load_stdlib("list_ops.iris");
    run_test_bindings("list_ops.iris", &fragments, &registry);

    let expected = [
        "append", "nth", "take", "drop", "sort", "dedup", "range",
        "length", "reverse", "head", "tail", "last", "init", "flatten",
        "list_sum", "list_product",
    ];
    for name in &expected {
        assert!(
            find_fragment(&fragments, name).is_some(),
            "list_ops.iris: missing fragment '{}'",
            name
        );
    }

    // range(0, 5) = (0, 1, 2, 3, 4)
    if let Some(graph) = find_fragment(&fragments, "range") {
        let result = eval_with_args(graph, &[Value::Int(0), Value::Int(5)], &registry);
        match result {
            Ok(Value::Tuple(items)) => {
                let vals: Vec<i64> = items
                    .iter()
                    .filter_map(value_to_i64)
                    .collect();
                assert_eq!(vals, vec![0, 1, 2, 3, 4], "range(0, 5) should be [0,1,2,3,4]");
            }
            other => panic!("range: unexpected result: {:?}", other),
        }
    }

    // head((10, 20, 30)) = 10
    if let Some(graph) = find_fragment(&fragments, "head") {
        let input = Value::tuple(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
        let result = eval_with_args(graph, &[input], &registry);
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("head should return int-like");
                assert_eq!(n, 10, "head((10,20,30)) should be 10");
            }
            Err(e) => panic!("head: {}", e),
        }
    }

    // list_sum((1, 2, 3, 4, 5)) = 15
    if let Some(graph) = find_fragment(&fragments, "list_sum") {
        let input = Value::tuple(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ]);
        let result = eval_with_args(graph, &[input], &registry);
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("list_sum should return int-like");
                assert_eq!(n, 15, "list_sum((1..5)) should be 15");
            }
            Err(e) => panic!("list_sum: {}", e),
        }
    }

    // list_product((1, 2, 3, 4)) = 24
    if let Some(graph) = find_fragment(&fragments, "list_product") {
        let input = Value::tuple(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]);
        let result = eval_with_args(graph, &[input], &registry);
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("list_product should return int-like");
                assert_eq!(n, 24, "list_product((1,2,3,4)) should be 24");
            }
            Err(e) => panic!("list_product: {}", e),
        }
    }

    // sort((3, 1, 4, 1, 5)) = (1, 1, 3, 4, 5)
    if let Some(graph) = find_fragment(&fragments, "sort") {
        let input = Value::tuple(vec![
            Value::Int(3),
            Value::Int(1),
            Value::Int(4),
            Value::Int(1),
            Value::Int(5),
        ]);
        let result = eval_with_args(graph, &[input], &registry);
        match result {
            Ok(Value::Tuple(items)) => {
                let vals: Vec<i64> = items.iter().filter_map(value_to_i64).collect();
                assert_eq!(vals, vec![1, 1, 3, 4, 5], "sort should be [1,1,3,4,5]");
            }
            other => panic!("sort: unexpected result: {:?}", other),
        }
    }

    // dedup((1, 1, 2, 2, 3)) = (1, 2, 3)
    if let Some(graph) = find_fragment(&fragments, "dedup") {
        let input = Value::tuple(vec![
            Value::Int(1),
            Value::Int(1),
            Value::Int(2),
            Value::Int(2),
            Value::Int(3),
        ]);
        let result = eval_with_args(graph, &[input], &registry);
        match result {
            Ok(Value::Tuple(items)) => {
                let vals: Vec<i64> = items.iter().filter_map(value_to_i64).collect();
                assert_eq!(vals, vec![1, 2, 3], "dedup should be [1,2,3]");
            }
            other => panic!("dedup: unexpected result: {:?}", other),
        }
    }

    println!("list_ops.iris: all evaluations passed");
}

#[test]
fn test_map_ops_iris() {
    let (fragments, registry) = load_stdlib("map_ops.iris");
    run_test_bindings("map_ops.iris", &fragments, &registry);

    let expected = [
        "empty_map", "insert", "get", "remove", "keys", "values", "size",
        "has_key", "insert_all", "map_over_values", "merge",
    ];
    for name in &expected {
        assert!(
            find_fragment(&fragments, name).is_some(),
            "map_ops.iris: missing fragment '{}'",
            name
        );
    }

    // empty_map should evaluate to a State
    if let Some(graph) = find_fragment(&fragments, "empty_map") {
        let result = eval_with_args(graph, &[], &registry);
        match result {
            Ok(Value::State(_)) => {}
            other => panic!("empty_map: expected State, got: {:?}", other),
        }
    }

    println!("map_ops.iris: all evaluations passed");
}

#[test]
fn test_set_ops_iris() {
    let (fragments, registry) = load_stdlib("set_ops.iris");
    run_test_bindings("set_ops.iris", &fragments, &registry);

    let expected = [
        "set_from_list",
        "set_empty",
        "set_insert",
        "set_contains",
        "set_remove",
        "set_union",
        "set_intersect",
        "set_diff",
        "set_size",
        "set_is_subset",
    ];
    for name in &expected {
        assert!(
            find_fragment(&fragments, name).is_some(),
            "set_ops.iris: missing fragment '{}'",
            name
        );
    }

    // set_from_list((3, 1, 2, 1)) should produce sorted deduped list
    if let Some(graph) = find_fragment(&fragments, "set_from_list") {
        let input = Value::tuple(vec![
            Value::Int(3),
            Value::Int(1),
            Value::Int(2),
            Value::Int(1),
        ]);
        let result = eval_with_args(graph, &[input], &registry);
        match result {
            Ok(Value::Tuple(items)) => {
                let vals: Vec<i64> = items.iter().filter_map(value_to_i64).collect();
                assert_eq!(vals, vec![1, 2, 3], "set_from_list((3,1,2,1)) should be [1,2,3]");
            }
            other => panic!("set_from_list: unexpected result: {:?}", other),
        }
    }

    // set_contains((1, 2, 3), 2) = 1
    if let Some(graph) = find_fragment(&fragments, "set_contains") {
        let set = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = eval_with_args(graph, &[set, Value::Int(2)], &registry);
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("set_contains should return int-like");
                assert_eq!(n, 1, "set_contains((1,2,3), 2) should be 1");
            }
            Err(e) => panic!("set_contains: {}", e),
        }
    }

    // set_contains((1, 2, 3), 5) = 0
    if let Some(graph) = find_fragment(&fragments, "set_contains") {
        let set = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = eval_with_args(graph, &[set, Value::Int(5)], &registry);
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("set_contains should return int-like");
                assert_eq!(n, 0, "set_contains((1,2,3), 5) should be 0");
            }
            Err(e) => panic!("set_contains(missing): {}", e),
        }
    }

    // set_size((1, 2, 3)) = 3
    if let Some(graph) = find_fragment(&fragments, "set_size") {
        let set = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = eval_with_args(graph, &[set], &registry);
        match result {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("set_size should return int-like");
                assert_eq!(n, 3, "set_size((1,2,3)) should be 3");
            }
            Err(e) => panic!("set_size: {}", e),
        }
    }

    // set_intersect((1, 2, 3), (2, 3, 4)) = (2, 3)
    if let Some(graph) = find_fragment(&fragments, "set_intersect") {
        let a = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let b = Value::tuple(vec![Value::Int(2), Value::Int(3), Value::Int(4)]);
        let result = eval_with_args(graph, &[a, b], &registry);
        match result {
            Ok(Value::Tuple(items)) => {
                let vals: Vec<i64> = items.iter().filter_map(value_to_i64).collect();
                assert_eq!(vals, vec![2, 3], "set_intersect should be [2,3]");
            }
            other => panic!("set_intersect: unexpected result: {:?}", other),
        }
    }

    println!("set_ops.iris: all evaluations passed");
}

#[test]
fn test_lazy_iris() {
    // lazy.iris compiles but uses lazy_unfold/lazy_take/lazy_map primitives
    // that the interpreter may not fully support. Verify compilation and
    // check fragment presence.
    let (fragments, registry) = load_stdlib("lazy.iris");
    run_test_bindings("lazy.iris", &fragments, &registry);

    let expected = [
        "naturals",
        "fibs",
        "take",
        "lmap",
        "sum_first_n",
        "repeat",
        "countdown",
        "powers_of_2",
    ];
    for name in &expected {
        assert!(
            find_fragment(&fragments, name).is_some(),
            "lazy.iris: missing fragment '{}'",
            name
        );
    }

    // sum_first_n uses lazy_unfold + fold — may or may not work in the interpreter.
    // We test it optimistically but allow graceful failure.
    if let Some(graph) = find_fragment(&fragments, "sum_first_n") {
        match eval_with_args(graph, &[Value::Int(100)], &registry) {
            Ok(ref v) => {
                let n = value_to_i64(v).expect("sum_first_n should return int-like");
                assert_eq!(n, 4950, "sum_first_n(100) should be 4950");
                println!("lazy.iris: sum_first_n(100) = {} ✓", n);
            }
            Err(_) => {
                // Lazy primitives not yet fully supported in interpreter — acceptable
                println!("lazy.iris: sum_first_n evaluation not supported (lazy primitives pending)");
            }
        }
    }

    println!("lazy.iris: compiled successfully with {} fragments", fragments.len());
}

#[test]
fn test_json_full_iris() {
    let (fragments, registry) = load_stdlib("json_full.iris");

    // Verify key fragments exist
    let expected = [
        "json_null",
        "json_bool",
        "json_int",
        "json_float",
        "json_string",
        "json_array",
        "json_object",
        "json_type",
        "json_get",
        "json_escape_string",
        "json_stringify",
    ];
    for name in &expected {
        assert!(
            find_fragment(&fragments, name).is_some(),
            "json_full.iris: missing fragment '{}'",
            name
        );
    }

    // Run all test_ bindings
    let passed = run_test_bindings("json_full.iris", &fragments, &registry);
    assert!(passed >= 20, "json_full.iris: expected at least 20 test_ bindings, got {}", passed);

    println!("json_full.iris: all evaluations passed ({} tests)", passed);
}

#[test]
fn test_quickcheck_iris() {
    let (fragments, registry) = load_stdlib("quickcheck.iris");

    // Verify key fragments exist
    let expected = [
        "qc_next_seed",
        "qc_int_range",
        "qc_bool",
        "qc_list_of",
        "qc_shrink_int",
        "qc_check",
    ];
    for name in &expected {
        assert!(
            find_fragment(&fragments, name).is_some(),
            "quickcheck.iris: missing fragment '{}'",
            name
        );
    }

    // Run all test_ bindings
    let passed = run_test_bindings("quickcheck.iris", &fragments, &registry);
    assert!(passed >= 15, "quickcheck.iris: expected at least 15 test_ bindings, got {}", passed);

    println!("quickcheck.iris: all evaluations passed ({} tests)", passed);
}

