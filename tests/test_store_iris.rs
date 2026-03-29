//! Tests for src/iris-programs/store/ — persistence and registry utilities.

use std::rc::Rc;

use iris_bootstrap::syntax;
use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compile IRIS source, register all fragments, return named graphs + registry.
fn compile_with_registry(src: &str) -> (Vec<(String, SemanticGraph)>, FragmentRegistry) {
    let result = syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", syntax::format_error(src, err));
        }
        panic!(
            "compilation failed with {} errors:\n{}",
            result.errors.len(),
            result
                .errors
                .iter()
                .map(|e| syntax::format_error(src, e))
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

/// Find a named function and evaluate it with the given inputs.
fn eval_named(
    frags: &[(String, SemanticGraph)],
    registry: &FragmentRegistry,
    name: &str,
    inputs: &[Value],
) -> Value {
    let graph = frags
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, g)| g)
        .unwrap_or_else(|| panic!("function '{}' not found", name));
    let (out, _) =
        interpreter::interpret_with_registry(graph, inputs, None, Some(registry)).unwrap();
    out.into_iter().next().unwrap_or(Value::Unit)
}

fn as_int(v: &Value) -> i64 {
    match v {
        Value::Int(n) => *n,
        Value::Bool(true) => 1,
        Value::Bool(false) => 0,
        other => panic!("expected Int, got {:?}", other),
    }
}

fn as_tuple(v: &Value) -> &Vec<Value> {
    match v {
        Value::Tuple(t) => t,
        other => panic!("expected Tuple, got {:?}", other),
    }
}

fn as_string(v: &Value) -> &str {
    match v {
        Value::String(s) => s.as_str(),
        other => panic!("expected String, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// snapshot.iris tests
// ---------------------------------------------------------------------------

fn snapshot_src() -> String {
    std::fs::read_to_string("src/iris-programs/store/snapshot.iris")
        .expect("failed to read snapshot.iris")
}

#[test]
fn test_snapshot_size() {
    let src = snapshot_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "snapshot_size", &[
        Value::Int(10), Value::Int(5), Value::Int(3),
    ]);
    // 10*8192 + 5*1024 + 3*4096 = 81920 + 5120 + 12288 = 99328
    assert_eq!(as_int(&v), 99328);
}

#[test]
fn test_should_snapshot_yes() {
    let src = snapshot_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "should_snapshot", &[
        Value::Int(300), Value::Int(300),
    ]);
    assert_eq!(as_int(&v), 1);
}

#[test]
fn test_should_snapshot_no() {
    let src = snapshot_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "should_snapshot", &[
        Value::Int(100), Value::Int(300),
    ]);
    assert_eq!(as_int(&v), 0);
}

#[test]
fn test_default_snapshot_interval() {
    let src = snapshot_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "default_snapshot_interval", &[]);
    assert_eq!(as_int(&v), 300);
}

#[test]
fn test_is_valid_snapshot_valid() {
    let src = snapshot_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "is_valid_snapshot", &[
        Value::Int(5), Value::Int(1000),
    ]);
    assert_eq!(as_int(&v), 1);
}

#[test]
fn test_is_valid_snapshot_zero_frags() {
    let src = snapshot_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "is_valid_snapshot", &[
        Value::Int(0), Value::Int(1000),
    ]);
    assert_eq!(as_int(&v), 0);
}

#[test]
fn test_snapshot_stats() {
    let src = snapshot_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "snapshot_stats", &[
        Value::Int(10), Value::Int(5), Value::Int(3), Value::Int(1000),
    ]);
    let t = as_tuple(&v);
    assert_eq!(as_int(&t[0]), 99328); // size
    assert_eq!(as_int(&t[1]), 10);    // fragment_count
    assert_eq!(as_int(&t[2]), 1);     // is_valid
}

// ---------------------------------------------------------------------------
// file_store.iris tests (pure math only — skip file I/O functions)
// ---------------------------------------------------------------------------

fn file_store_src() -> String {
    std::fs::read_to_string("src/iris-programs/store/file_store.iris")
        .expect("failed to read file_store.iris")
}

#[test]
fn test_fragment_file_key() {
    let src = file_store_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "fragment_file_key", &[Value::Int(42)]);
    assert!(as_int(&v) > 0, "fragment_file_key(42) must be positive");
}

#[test]
fn test_fragment_path() {
    let src = file_store_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "fragment_path", &[
        Value::String("./store".to_string()),
        Value::Int(42),
    ]);
    let s = as_string(&v);
    assert!(s.contains("./store/fragments/"), "path should contain base dir: {}", s);
    assert!(s.ends_with(".json"), "path should end with .json: {}", s);
}

#[test]
fn test_is_safe_key_valid() {
    let src = file_store_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "is_safe_key", &[Value::Int(10)]);
    assert_eq!(as_int(&v), 1);
}

#[test]
fn test_is_safe_key_zero() {
    let src = file_store_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "is_safe_key", &[Value::Int(0)]);
    assert_eq!(as_int(&v), 0);
}

#[test]
fn test_is_safe_key_too_large() {
    let src = file_store_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "is_safe_key", &[Value::Int(256)]);
    assert_eq!(as_int(&v), 0);
}

#[test]
fn test_fragment_disk_size() {
    let src = file_store_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "fragment_disk_size", &[
        Value::Int(10), Value::Int(5),
    ]);
    // (48 + 10*10 + 5*18) * 2 = (48+100+90)*2 = 476
    assert_eq!(as_int(&v), 476);
}

#[test]
fn test_disk_usage() {
    let src = file_store_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "disk_usage", &[
        Value::Int(10), Value::Int(5), Value::Int(3), Value::Int(2),
    ]);
    // 476 + 3*1024 + 2*4096 = 476 + 3072 + 8192 = 11740
    assert_eq!(as_int(&v), 11740);
}

#[test]
fn test_use_atomic_write() {
    let src = file_store_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "use_atomic_write", &[]);
    assert_eq!(as_int(&v), 1);
}

#[test]
fn test_file_count() {
    let src = file_store_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "file_count", &[
        Value::Int(10), Value::Int(5), Value::Int(3),
    ]);
    assert_eq!(as_int(&v), 18);
}

#[test]
fn test_file_store_stats() {
    let src = file_store_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "file_store_stats", &[
        Value::Int(10), Value::Int(10), Value::Int(5), Value::Int(3), Value::Int(2),
    ]);
    let t = as_tuple(&v);
    assert_eq!(as_int(&t[0]), 15);    // file_count(10, 3, 2)
    assert_eq!(as_int(&t[1]), 11740); // disk_usage(10, 5, 3, 2)
    assert_eq!(as_int(&t[2]), 1);     // healthy
}

// ---------------------------------------------------------------------------
// registry.iris tests
// ---------------------------------------------------------------------------

fn registry_src() -> String {
    std::fs::read_to_string("src/iris-programs/store/registry.iris")
        .expect("failed to read registry.iris")
}

/// Build a 3-entry registry tuple: ((1, 100), (2, 200), (3, 300))
fn sample_entries() -> Value {
    Value::tuple(vec![
        Value::tuple(vec![Value::Int(1), Value::Int(100)]),
        Value::tuple(vec![Value::Int(2), Value::Int(200)]),
        Value::tuple(vec![Value::Int(3), Value::Int(300)]),
    ])
}

#[test]
fn test_registry_count() {
    let src = registry_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "registry_count", &[sample_entries()]);
    assert_eq!(as_int(&v), 3);
}

#[test]
fn test_registry_contains_found() {
    let src = registry_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "registry_contains", &[
        sample_entries(), Value::Int(2),
    ]);
    assert_eq!(as_int(&v), 1);
}

#[test]
fn test_registry_contains_missing() {
    let src = registry_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "registry_contains", &[
        sample_entries(), Value::Int(99),
    ]);
    assert_eq!(as_int(&v), 0);
}

#[test]
fn test_registry_fingerprint() {
    let src = registry_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "registry_fingerprint", &[sample_entries()]);
    // XOR of 1, 2, 3 = 1 ^ 2 ^ 3 = 0
    assert_eq!(as_int(&v), 1 ^ 2 ^ 3);
}

#[test]
fn test_registry_remove() {
    let src = registry_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "registry_remove", &[
        sample_entries(), Value::Int(2),
    ]);
    let t = as_tuple(&v);
    assert_eq!(t.len(), 2, "should have 2 entries after removal");
}

// ---------------------------------------------------------------------------
// serialize_graph.iris tests (after fixing graph_get_children)
// ---------------------------------------------------------------------------

fn serialize_src() -> String {
    std::fs::read_to_string("src/iris-programs/store/serialize_graph.iris")
        .expect("failed to read serialize_graph.iris")
}

#[test]
fn test_serialized_size() {
    let src = serialize_src();
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "serialized_size", &[
        Value::Int(10), Value::Int(5),
    ]);
    // 48 + 10*10 + 5*18 = 48 + 100 + 90 = 238
    assert_eq!(as_int(&v), 238);
}

#[test]
fn test_get_root_with_self_graph() {
    // Compile a small program that uses self_graph to get a program,
    // then calls get_root on it.
    let mut src = serialize_src();
    src.push_str("\nlet test_get_root : Int = get_root (self_graph 0)\n");
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "test_get_root", &[]);
    // Root node ID is a u64 internally — just check we get an Int back
    let _ = as_int(&v);
}

#[test]
fn test_get_node_count_with_self_graph() {
    let mut src = serialize_src();
    src.push_str("\nlet test_get_node_count : Int = get_node_count (self_graph 0)\n");
    let (frags, reg) = compile_with_registry(&src);
    let v = eval_named(&frags, &reg, "test_get_node_count", &[]);
    assert!(as_int(&v) > 0, "node count should be positive, got {}", as_int(&v));
}

// ===========================================================================
// Integration tests: real compiled Program graphs through store pipelines
// ===========================================================================

/// Compile a single IRIS function and return its SemanticGraph.
fn compile_function(src: &str) -> SemanticGraph {
    let result = syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", syntax::format_error(src, err));
        }
        panic!("compile_function failed: {} errors", result.errors.len());
    }
    assert!(!result.fragments.is_empty(), "no fragments compiled");
    result.fragments.into_iter().next().unwrap().1.graph
}

/// Build a 3-entry registry with real compiled Program graphs.
fn real_program_entries() -> Value {
    let add_graph = compile_function("let add x y : Int -> Int -> Int = x + y");
    let dbl_graph = compile_function("let double x : Int -> Int = x * 2");
    let neg_graph = compile_function("let negate x : Int -> Int = 0 - x");
    Value::tuple(vec![
        Value::tuple(vec![Value::Int(1), Value::Program(Rc::new(add_graph))]),
        Value::tuple(vec![Value::Int(2), Value::Program(Rc::new(dbl_graph))]),
        Value::tuple(vec![Value::Int(3), Value::Program(Rc::new(neg_graph))]),
    ])
}

// --- registry.iris integration tests ---

#[test]
fn test_integration_registry_count_real_programs() {
    let src = registry_src();
    let (frags, reg) = compile_with_registry(&src);
    let entries = real_program_entries();
    let v = eval_named(&frags, &reg, "registry_count", &[entries]);
    assert_eq!(as_int(&v), 3);
}

#[test]
fn test_integration_registry_contains_real_programs() {
    let src = registry_src();
    let (frags, reg) = compile_with_registry(&src);
    let found = eval_named(
        &frags, &reg, "registry_contains",
        &[real_program_entries(), Value::Int(2)],
    );
    assert_eq!(as_int(&found), 1, "should find id=2");
    let missing = eval_named(
        &frags, &reg, "registry_contains",
        &[real_program_entries(), Value::Int(99)],
    );
    assert_eq!(as_int(&missing), 0, "should not find id=99");
}

#[test]
fn test_integration_registry_eval_add_program() {
    let src = registry_src();
    let (frags, reg) = compile_with_registry(&src);
    let entries = real_program_entries();
    let inputs = Value::tuple(vec![Value::Int(3), Value::Int(4)]);
    let v = eval_named(&frags, &reg, "registry_eval", &[entries, Value::Int(1), inputs]);
    assert_eq!(as_int(&v), 7, "add(3, 4) should be 7");
}

#[test]
fn test_integration_registry_eval_double_program() {
    let src = registry_src();
    let (frags, reg) = compile_with_registry(&src);
    let entries = real_program_entries();
    let inputs = Value::tuple(vec![Value::Int(5)]);
    let v = eval_named(&frags, &reg, "registry_eval", &[entries, Value::Int(2), inputs]);
    assert_eq!(as_int(&v), 10, "double(5) should be 10");
}

#[test]
fn test_integration_registry_eval_negate_program() {
    let src = registry_src();
    let (frags, reg) = compile_with_registry(&src);
    let entries = real_program_entries();
    let inputs = Value::tuple(vec![Value::Int(7)]);
    let v = eval_named(&frags, &reg, "registry_eval", &[entries, Value::Int(3), inputs]);
    assert_eq!(as_int(&v), -7, "negate(7) should be -7");
}

#[test]
fn test_integration_registry_fingerprint_real_programs() {
    let src = registry_src();
    let (frags, reg) = compile_with_registry(&src);
    let entries = real_program_entries();
    let v = eval_named(&frags, &reg, "registry_fingerprint", &[entries]);
    assert_eq!(as_int(&v), 1 ^ 2 ^ 3, "XOR of IDs 1, 2, 3");
}

#[test]
fn test_integration_registry_remove_real_programs() {
    let src = registry_src();
    let (frags, reg) = compile_with_registry(&src);
    let entries = real_program_entries();
    let v = eval_named(&frags, &reg, "registry_remove", &[entries, Value::Int(2)]);
    let remaining = as_tuple(&v);
    assert_eq!(remaining.len(), 2, "should have 2 entries after removing id=2");
}

// --- serialize_graph.iris integration tests ---

#[test]
fn test_integration_get_root_real_program() {
    let src = serialize_src();
    let (frags, reg) = compile_with_registry(&src);
    let graph = compile_function("let add x y : Int -> Int -> Int = x + y");
    let prog = Value::Program(Rc::new(graph));
    let v = eval_named(&frags, &reg, "get_root", &[prog]);
    assert!(as_int(&v) > 0, "root node ID should be positive");
}

#[test]
fn test_integration_get_node_count_matches_graph() {
    let src = serialize_src();
    let (frags, reg) = compile_with_registry(&src);
    let graph = compile_function("let add x y : Int -> Int -> Int = x + y");
    let expected = graph.nodes.len() as i64;
    let prog = Value::Program(Rc::new(graph));
    let v = eval_named(&frags, &reg, "get_node_count", &[prog]);
    assert_eq!(as_int(&v), expected, "IRIS node count should match graph.nodes.len()");
}

#[test]
fn test_integration_serialize_node_root() {
    let src = serialize_src();
    let (frags, reg) = compile_with_registry(&src);
    let graph = compile_function("let add x y : Int -> Int -> Int = x + y");
    let prog = Value::Program(Rc::new(graph));
    let root_val = eval_named(&frags, &reg, "get_root", &[prog.clone()]);
    let root_id = as_int(&root_val);
    let v = eval_named(&frags, &reg, "serialize_node", &[prog, Value::Int(root_id)]);
    let t = as_tuple(&v);
    assert_eq!(t.len(), 3, "should return (kind, opcode, child_count)");
    assert!(as_int(&t[0]) >= 0, "kind should be non-negative");
}

#[test]
fn test_integration_graph_fingerprint_deterministic() {
    let src = serialize_src();
    let (frags, reg) = compile_with_registry(&src);
    let graph = compile_function("let add x y : Int -> Int -> Int = x + y");
    let fp1 = as_int(&eval_named(
        &frags, &reg, "graph_fingerprint",
        &[Value::Program(Rc::new(graph.clone()))],
    ));
    let fp2 = as_int(&eval_named(
        &frags, &reg, "graph_fingerprint",
        &[Value::Program(Rc::new(graph))],
    ));
    assert_ne!(fp1, 0, "fingerprint should be non-zero");
    assert_eq!(fp1, fp2, "same program should produce identical fingerprint");
}

#[test]
fn test_integration_graphs_equal_same_program() {
    let src = serialize_src();
    let (frags, reg) = compile_with_registry(&src);
    let graph = compile_function("let add x y : Int -> Int -> Int = x + y");
    let v = eval_named(&frags, &reg, "graphs_equal", &[
        Value::Program(Rc::new(graph.clone())),
        Value::Program(Rc::new(graph)),
    ]);
    assert_eq!(as_int(&v), 1, "same program should be structurally equal");
}

#[test]
fn test_integration_graphs_equal_different_programs() {
    let src = serialize_src();
    let (frags, reg) = compile_with_registry(&src);
    let add_graph = compile_function("let add x y : Int -> Int -> Int = x + y");
    let mul_graph = compile_function("let mul x y : Int -> Int -> Int = x * y");
    let v = eval_named(&frags, &reg, "graphs_equal", &[
        Value::Program(Rc::new(add_graph)),
        Value::Program(Rc::new(mul_graph)),
    ]);
    assert_eq!(as_int(&v), 0, "different programs should not be equal");
}
