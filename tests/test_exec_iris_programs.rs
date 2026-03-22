
//! Tests that load and execute .iris programs for effects dispatch and semantic hashing.
//!
//! Verifies:
//! - Effects effects_dispatch.iris routes effect tags correctly
//! - Semantic hash semantic_hash.iris uses 128 probes
//!
//! JIT and VM tests were removed — MmapExec/CallNative permanently disabled for security,
//! and src/iris-programs/jit + src/iris-programs/vm directories were deleted.
//! All tests LOAD AND EXECUTE .iris files through the compiler and evaluator.

use iris_exec::effect_runtime::RuntimeEffectHandler;
use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct CompiledFile {
    fragments: Vec<(String, SemanticGraph)>,
    registry: FragmentRegistry,
}

fn compile_iris(path: &str) -> CompiledFile {
    let src = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path, e));
    let result = iris_bootstrap::syntax::compile(&src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}: {}", path, iris_bootstrap::syntax::format_error(&src, err));
        }
        panic!("{} failed to compile with {} errors", path, result.errors.len());
    }
    let mut registry = FragmentRegistry::new();
    let fragments: Vec<(String, SemanticGraph)> = result
        .fragments
        .into_iter()
        .map(|(name, frag, _)| {
            registry.register(frag.clone());
            (name, frag.graph)
        })
        .collect();
    CompiledFile { fragments, registry }
}

fn run(f: &CompiledFile, name: &str, inputs: &[Value]) -> Value {
    let graph = &f.fragments.iter().find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("fragment '{}' not found", name)).1;
    let handler = RuntimeEffectHandler::new();
    let (outputs, _) = interpreter::interpret_with_effects(
        graph, inputs, None, Some(&f.registry), 10_000_000, Some(&handler),
    ).expect("evaluation should succeed");
    assert!(!outputs.is_empty(), "should produce output");
    outputs.into_iter().next().unwrap()
}

fn run_int(f: &CompiledFile, name: &str, inputs: &[Value]) -> i64 {
    match run(f, name, inputs) { Value::Int(v) => v, o => panic!("expected Int, got {:?}", o) }
}

fn run_tuple_ints(f: &CompiledFile, name: &str, inputs: &[Value]) -> (i64, i64) {
    match run(f, name, inputs) {
        Value::Tuple(t) => {
            assert!(t.len() >= 2);
            let a = match &t[0] { Value::Int(v) => *v, o => panic!("t[0]: {:?}", o) };
            let b = match &t[1] { Value::Int(v) => *v, o => panic!("t[1]: {:?}", o) };
            (a, b)
        }
        o => panic!("expected Tuple, got {:?}", o),
    }
}

/// Extract the payload from a DispatchOk(value) result. Panics on other variants.
fn run_dispatch_ok(f: &CompiledFile, name: &str, inputs: &[Value]) -> i64 {
    match run(f, name, inputs) {
        Value::Tagged(0, payload) => match *payload {
            Value::Int(v) => v,
            o => panic!("expected Int payload in DispatchOk, got {:?}", o),
        },
        o => panic!("expected DispatchOk, got {:?}", o),
    }
}

/// Assert that a dispatch function returns a specific DispatchResult variant.
fn assert_dispatch(f: &CompiledFile, name: &str, inputs: &[Value], expected: Value) {
    let actual = run(f, name, inputs);
    assert_eq!(actual, expected, "dispatch result mismatch");
}

// =========================================================================
// effects_dispatch.iris
// =========================================================================

#[test]
fn test_effects_noop() {
    let f = compile_iris("src/iris-programs/exec/effects_dispatch.iris");
    assert_dispatch(&f, "noop_dispatch", &[Value::Int(0), Value::Int(42)],
        Value::Tagged(0, Box::new(Value::Int(0))));
}

#[test]
fn test_effects_logging() {
    let f = compile_iris("src/iris-programs/exec/effects_dispatch.iris");
    assert_dispatch(&f, "logging_dispatch", &[Value::Int(5), Value::Int(42)],
        Value::Tagged(0, Box::new(Value::Int(5))));
}

#[test]
fn test_effects_random() {
    let f = compile_iris("src/iris-programs/exec/effects_dispatch.iris");
    let r = run_dispatch_ok(&f, "real_dispatch", &[Value::Int(10), Value::Int(0)]);
    assert_ne!(r, 0, "random should produce non-zero");
}

#[test]
fn test_effects_random_deterministic() {
    let f = compile_iris("src/iris-programs/exec/effects_dispatch.iris");
    let r1 = run_dispatch_ok(&f, "real_dispatch", &[Value::Int(10), Value::Int(12345)]);
    let r2 = run_dispatch_ok(&f, "real_dispatch", &[Value::Int(10), Value::Int(12345)]);
    assert_eq!(r1, r2, "same seed = same result");
}

#[test]
fn test_effects_timestamp() {
    let f = compile_iris("src/iris-programs/exec/effects_dispatch.iris");
    assert_dispatch(&f, "real_dispatch", &[Value::Int(9), Value::Int(1711234567)],
        Value::Tagged(0, Box::new(Value::Int(1711234567))));
}

#[test]
fn test_effects_log() {
    let f = compile_iris("src/iris-programs/exec/effects_dispatch.iris");
    assert_dispatch(&f, "real_dispatch", &[Value::Int(11), Value::Int(42)],
        Value::Tagged(0, Box::new(Value::Int(42))));
}

#[test]
fn test_effects_unsupported() {
    let f = compile_iris("src/iris-programs/exec/effects_dispatch.iris");
    assert_dispatch(&f, "real_dispatch", &[Value::Int(2), Value::Int(0)],
        Value::Tagged(1, Box::new(Value::Unit)));
}

#[test]
fn test_effects_no_host() {
    let f = compile_iris("src/iris-programs/exec/effects_dispatch.iris");
    assert_dispatch(&f, "real_dispatch", &[Value::Int(4), Value::Int(0)],
        Value::Tagged(3, Box::new(Value::Int(0))));
}

#[test]
fn test_effects_main_noop() {
    let f = compile_iris("src/iris-programs/exec/effects_dispatch.iris");
    assert_dispatch(&f, "effects_dispatch", &[Value::Int(10), Value::Int(42), Value::Int(0)],
        Value::Tagged(0, Box::new(Value::Int(0))));
}

#[test]
fn test_effects_main_real() {
    let f = compile_iris("src/iris-programs/exec/effects_dispatch.iris");
    let r = run_dispatch_ok(&f, "effects_dispatch", &[Value::Int(10), Value::Int(0), Value::Int(1)]);
    assert_ne!(r, 0);
}

#[test]
fn test_effects_main_logging() {
    let f = compile_iris("src/iris-programs/exec/effects_dispatch.iris");
    assert_dispatch(&f, "effects_dispatch", &[Value::Int(7), Value::Int(99), Value::Int(3)],
        Value::Tagged(0, Box::new(Value::Int(7))));
}

#[test]
fn test_effect_allowed() {
    let f = compile_iris("src/iris-programs/exec/effects_dispatch.iris");
    assert_eq!(run_int(&f, "effect_allowed", &[Value::Int(10), Value::Int(1024)]), 1);
    assert_eq!(run_int(&f, "effect_allowed", &[Value::Int(5), Value::Int(1024)]), 0);
}

#[test]
fn test_effect_risk_level() {
    let f = compile_iris("src/iris-programs/exec/effects_dispatch.iris");
    assert_eq!(run_int(&f, "effect_risk_level", &[Value::Int(9)]), 0);   // Timestamp = pure
    assert_eq!(run_int(&f, "effect_risk_level", &[Value::Int(10)]), 0);  // Random = pure
    assert_eq!(run_int(&f, "effect_risk_level", &[Value::Int(0)]), 1);   // Print = read-only
    assert_eq!(run_int(&f, "effect_risk_level", &[Value::Int(5)]), 2);   // FileWrite = write
    assert_eq!(run_int(&f, "effect_risk_level", &[Value::Int(2)]), 3);   // HttpGet = network
    assert_eq!(run_int(&f, "effect_risk_level", &[Value::Int(32)]), 4);  // ThreadSpawn = system
}

// =========================================================================
// semantic_hash.iris
// =========================================================================

#[test]
fn test_hash_default_probe_count() {
    let f = compile_iris("src/iris-programs/exec/semantic_hash.iris");
    assert_eq!(run_int(&f, "default_probe_count", &[]), 128);
}

#[test]
fn test_hash_probe_from_seed() {
    let f = compile_iris("src/iris-programs/exec/semantic_hash.iris");
    let r = run_int(&f, "probe_from_seed", &[Value::Int(42)]);
    assert!(r >= -1000 && r <= 1000, "probe in range, got {}", r);
}

#[test]
fn test_hash_probe_deterministic() {
    let f = compile_iris("src/iris-programs/exec/semantic_hash.iris");
    let r1 = run_int(&f, "probe_from_seed", &[Value::Int(12345)]);
    let r2 = run_int(&f, "probe_from_seed", &[Value::Int(12345)]);
    assert_eq!(r1, r2);
}

#[test]
fn test_hash_pair() {
    let f = compile_iris("src/iris-programs/exec/semantic_hash.iris");
    let h1 = run_int(&f, "hash_pair", &[Value::Int(1), Value::Int(2)]);
    let h2 = run_int(&f, "hash_pair", &[Value::Int(2), Value::Int(1)]);
    assert_ne!(h1, h2, "order-sensitive");
    assert!(h1 >= 0);
}

#[test]
fn test_semantic_eq() {
    let f = compile_iris("src/iris-programs/exec/semantic_hash.iris");
    assert_eq!(run_int(&f, "semantic_eq", &[Value::Int(42), Value::Int(42)]), 1);
    assert_eq!(run_int(&f, "semantic_eq", &[Value::Int(42), Value::Int(99)]), 0);
}
