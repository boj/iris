//! End-to-end JIT code generator tests.
//!
//! Loads jit_runtime.iris, compiles it to SemanticGraphs, and evaluates
//! the IRIS JIT code generator functions through the bootstrap evaluator
//! with a real RuntimeEffectHandler (JIT-enabled).
//!
//! This proves the full pipeline:
//!   .iris source → syntax compile → bootstrap eval → effect handler → W^X exec
//!
//! Requires: `--features jit`

use std::collections::BTreeMap;

use iris_bootstrap::{evaluate_with_effects, evaluate_with_fragments};
use iris_exec::effect_runtime::RuntimeEffectHandler;
use iris_types::eval::Value;
use iris_types::fragment::FragmentId;
use iris_types::graph::SemanticGraph;


// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_iris(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {}", path, e))
}

fn compile_all(src: &str) -> (Vec<(String, SemanticGraph)>, BTreeMap<FragmentId, SemanticGraph>) {
    let result = iris_bootstrap::syntax::compile(src);
    assert!(result.errors.is_empty(), "compile errors: {:?}", result.errors);
    let mut registry = BTreeMap::new();
    let mut fragments = Vec::new();
    for (name, frag, _smap) in result.fragments {
        registry.insert(frag.id, frag.graph.clone());
        fragments.push((name, frag.graph));
    }
    (fragments, registry)
}

fn find_graph(fragments: &[(String, SemanticGraph)], name: &str) -> SemanticGraph {
    fragments
        .iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("function '{}' not found", name))
        .1
        .clone()
}

fn eval_with_jit(
    graph: &SemanticGraph,
    inputs: &[Value],
    handler: &RuntimeEffectHandler,
    registry: &BTreeMap<FragmentId, SemanticGraph>,
) -> Value {
    evaluate_with_effects(graph, inputs, 100_000, registry, handler)
        .unwrap_or_else(|e| panic!("eval failed: {:?}", e))
}

fn eval_no_handler(
    graph: &SemanticGraph,
    inputs: &[Value],
    registry: &BTreeMap<FragmentId, SemanticGraph>,
) -> Value {
    evaluate_with_fragments(graph, inputs, 100_000, registry)
        .unwrap_or_else(|e| panic!("eval failed: {:?}", e))
}

// ---------------------------------------------------------------------------
// Tests: IRIS x86 instruction encoding
// ---------------------------------------------------------------------------

#[test]
fn test_iris_x86_mov_imm64() {
    let src = read_iris("src/iris-programs/exec/jit_runtime.iris");
    let (frags, reg) = compile_all(&src);
    let graph = find_graph(&frags, "x86_mov_imm64");

    // x86_mov_imm64(reg_rax=0, 42) should produce a nested tuple
    // representing: REX.W (0x48), B8+0 (0xB8), then 8 bytes of imm64
    let result = eval_no_handler(&graph, &[Value::Int(0), Value::Int(42)], &reg);

    // The result is a nested tuple — flatten it to verify the bytes
    let bytes = flatten_value_to_bytes(&result);
    assert_eq!(bytes.len(), 10, "MOV RAX, imm64 is 10 bytes");
    assert_eq!(bytes[0], 0x48, "REX.W prefix");
    assert_eq!(bytes[1], 0xB8, "MOV RAX opcode");
    assert_eq!(bytes[2], 42, "imm64 low byte = 42");
    for &b in &bytes[3..10] {
        assert_eq!(b, 0, "remaining imm64 bytes = 0");
    }
}

#[test]
fn test_iris_x86_add() {
    let src = read_iris("src/iris-programs/exec/jit_runtime.iris");
    let (frags, reg) = compile_all(&src);
    let graph = find_graph(&frags, "x86_add");

    // x86_add(reg_rax=0, reg_rsi=6)
    let result = eval_no_handler(&graph, &[Value::Int(0), Value::Int(6)], &reg);
    let bytes = flatten_value_to_bytes(&result);
    assert_eq!(bytes.len(), 3, "ADD reg, reg is 3 bytes");
    assert_eq!(bytes[0], 0x48, "REX.W prefix");
    assert_eq!(bytes[1], 0x01, "ADD opcode");
}

#[test]
fn test_iris_x86_ret() {
    let src = read_iris("src/iris-programs/exec/jit_runtime.iris");
    let (frags, reg) = compile_all(&src);
    let graph = find_graph(&frags, "x86_ret");

    let result = eval_no_handler(&graph, &[], &reg);
    let bytes = flatten_value_to_bytes(&result);
    assert_eq!(bytes, vec![0xC3], "RET is single byte 0xC3");
}

// ---------------------------------------------------------------------------
// Tests: IRIS flatten_code + bytes_from_ints
// ---------------------------------------------------------------------------

#[test]
fn test_iris_flatten_code() {
    let src = read_iris("src/iris-programs/exec/jit_runtime.iris");
    let (frags, reg) = compile_all(&src);
    let flatten = find_graph(&frags, "flatten_code");

    // Build a simple nested tuple: (0x48, (0xB8, (42, 0)))
    // This mimics what x86 instruction builders produce
    let input = Value::tuple(vec![
        Value::Int(0x48),
        Value::tuple(vec![
            Value::Int(0xB8),
            Value::tuple(vec![Value::Int(42), Value::Int(0)]),
        ]),
    ]);

    let result = eval_no_handler(&flatten, &[input], &reg);
    match result {
        Value::Bytes(b) => {
            assert_eq!(b, vec![0x48, 0xB8, 42], "flatten should produce [0x48, 0xB8, 42]");
        }
        other => panic!("expected Bytes, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Tests: End-to-end JIT compilation via IRIS code generator
// ---------------------------------------------------------------------------

#[test]
fn test_iris_jit_const_fn() {
    let src = read_iris("src/iris-programs/exec/jit_runtime.iris");
    let (frags, reg) = compile_all(&src);
    let handler = RuntimeEffectHandler::new();

    // jit_const_fn(42) should: generate x86 code, mmap it, return handle
    let jit_const = find_graph(&frags, "jit_const_fn");
    let handle = eval_with_jit(&jit_const, &[Value::Int(42)], &handler, &reg);

    let handle_id = match handle {
        Value::Int(h) => h,
        other => panic!("expected Int handle, got {:?}", other),
    };
    assert!(handle_id > 0, "handle should be positive");

    // Now call the compiled function via jit_call
    let jit_call = find_graph(&frags, "jit_call");
    let result = eval_with_jit(
        &jit_call,
        &[Value::Int(handle_id), Value::tuple(vec![])],
        &handler,
        &reg,
    );

    assert_eq!(result, Value::Int(42), "JIT const function should return 42");
}

#[test]
fn test_iris_jit_add_fn() {
    let src = read_iris("src/iris-programs/exec/jit_runtime.iris");
    let (frags, reg) = compile_all(&src);
    let handler = RuntimeEffectHandler::new();

    // jit_add_fn(0) should generate an add function
    let jit_add = find_graph(&frags, "jit_add_fn");
    let handle = eval_with_jit(&jit_add, &[Value::Int(0)], &handler, &reg);
    let h = match handle { Value::Int(h) => h, other => panic!("expected Int, got {:?}", other) };

    // Call: add(17, 25) = 42
    let jit_call = find_graph(&frags, "jit_call");
    let result = eval_with_jit(
        &jit_call,
        &[Value::Int(h), Value::tuple(vec![Value::Int(17), Value::Int(25)])],
        &handler,
        &reg,
    );
    assert_eq!(result, Value::Int(42), "JIT add(17, 25) = 42");
}

#[test]
fn test_iris_jit_mul_fn() {
    let src = read_iris("src/iris-programs/exec/jit_runtime.iris");
    let (frags, reg) = compile_all(&src);
    let handler = RuntimeEffectHandler::new();

    let jit_mul = find_graph(&frags, "jit_mul_fn");
    let handle = eval_with_jit(&jit_mul, &[Value::Int(0)], &handler, &reg);
    let h = match handle { Value::Int(h) => h, other => panic!("expected Int, got {:?}", other) };

    let jit_call = find_graph(&frags, "jit_call");
    let result = eval_with_jit(
        &jit_call,
        &[Value::Int(h), Value::tuple(vec![Value::Int(6), Value::Int(7)])],
        &handler,
        &reg,
    );
    assert_eq!(result, Value::Int(42), "JIT mul(6, 7) = 42");
}

#[test]
fn test_iris_jit_identity_fn() {
    let src = read_iris("src/iris-programs/exec/jit_runtime.iris");
    let (frags, reg) = compile_all(&src);
    let handler = RuntimeEffectHandler::new();

    let jit_identity = find_graph(&frags, "jit_identity_fn");
    let handle = eval_with_jit(&jit_identity, &[Value::Int(0)], &handler, &reg);
    let h = match handle { Value::Int(h) => h, other => panic!("expected Int, got {:?}", other) };

    let jit_call = find_graph(&frags, "jit_call");
    let result = eval_with_jit(
        &jit_call,
        &[Value::Int(h), Value::tuple(vec![Value::Int(999)])],
        &handler,
        &reg,
    );
    assert_eq!(result, Value::Int(999), "JIT identity(999) = 999");
}

#[test]
fn test_iris_jit_compile_and_call() {
    let src = read_iris("src/iris-programs/exec/jit_runtime.iris");
    let (frags, reg) = compile_all(&src);
    let handler = RuntimeEffectHandler::new();

    // First generate add code bytes using IRIS
    let jit_add = find_graph(&frags, "jit_add_fn");
    let handle = eval_with_jit(&jit_add, &[Value::Int(0)], &handler, &reg);

    // Now call using jit_compile_and_call pattern
    let jit_call = find_graph(&frags, "jit_call");
    let h = match handle { Value::Int(h) => h, other => panic!("expected Int, got {:?}", other) };

    // Multiple calls to same compiled function
    for (a, b, expected) in [(1, 2, 3), (100, 200, 300), (0, 0, 0), (-5, 10, 5)] {
        let result = eval_with_jit(
            &jit_call,
            &[Value::Int(h), Value::tuple(vec![Value::Int(a), Value::Int(b)])],
            &handler,
            &reg,
        );
        assert_eq!(
            result,
            Value::Int(expected),
            "JIT add({}, {}) = {}",
            a,
            b,
            expected
        );
    }
}

// ---------------------------------------------------------------------------
// Helper: flatten nested Value tuples to bytes (for verification)
// ---------------------------------------------------------------------------

fn flatten_value_to_bytes(v: &Value) -> Vec<u8> {
    let mut out = Vec::new();
    flatten_value_rec(v, &mut out);
    out
}

fn flatten_value_rec(v: &Value, out: &mut Vec<u8>) {
    match v {
        Value::Int(n) => out.push(*n as u8),
        Value::Tuple(elems) => {
            for (i, e) in elems.iter().enumerate() {
                // Skip nil sentinel: Int(0) at tail of 2-element tuple
                if let Value::Int(0) = e {
                    if i == elems.len() - 1 && elems.len() == 2 {
                        continue;
                    }
                }
                flatten_value_rec(e, out);
            }
        }
        _ => {}
    }
}
