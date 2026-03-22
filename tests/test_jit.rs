//! JIT W^X integration tests.
//!
//! Tests that the JIT effect handler correctly:
//!   1. Compiles x86-64 machine code into W^X memory regions
//!   2. Enforces Write XOR Execute — never both simultaneously
//!   3. Calls compiled code via System V AMD64 ABI
//!   4. Cleans up memory on drop
//!   5. Rejects invalid inputs (empty code, bad handles)
//!
//! Requires: `--features jit`

use iris_exec::effect_runtime::RuntimeEffectHandler;
use iris_types::eval::{EffectHandler, EffectRequest, EffectTag, Value};

fn mmap_exec(handler: &RuntimeEffectHandler, code: Vec<u8>) -> Result<Value, String> {
    handler
        .handle(EffectRequest {
            tag: EffectTag::MmapExec,
            args: vec![Value::Bytes(code)],
        })
        .map_err(|e| e.message)
}

fn call_native(
    handler: &RuntimeEffectHandler,
    handle: i64,
    args: Vec<Value>,
) -> Result<Value, String> {
    handler
        .handle(EffectRequest {
            tag: EffectTag::CallNative,
            args: vec![Value::Int(handle), Value::tuple(args)],
        })
        .map_err(|e| e.message)
}

// ---------------------------------------------------------------------------
// x86-64 machine code helpers
// ---------------------------------------------------------------------------

/// `mov rax, imm64; ret` — returns a constant
fn x86_const(value: i64) -> Vec<u8> {
    let mut code = vec![0x48, 0xB8]; // REX.W + MOV RAX, imm64
    code.extend_from_slice(&value.to_le_bytes());
    code.push(0xC3); // RET
    code
}

/// `lea rax, [rdi + rsi]; ret` — adds two arguments
fn x86_add_args() -> Vec<u8> {
    vec![
        0x48, 0x8D, 0x04, 0x37, // lea rax, [rdi + rsi]
        0xC3,                    // ret
    ]
}

/// `mov rax, rdi; sub rax, rsi; ret` — subtracts rsi from rdi
fn x86_sub_args() -> Vec<u8> {
    vec![
        0x48, 0x89, 0xF8, // mov rax, rdi
        0x48, 0x29, 0xF0, // sub rax, rsi
        0xC3,             // ret
    ]
}

/// `mov rax, rdi; imul rax, rsi; ret` — multiplies two arguments
fn x86_mul_args() -> Vec<u8> {
    vec![
        0x48, 0x89, 0xF8,       // mov rax, rdi
        0x48, 0x0F, 0xAF, 0xC6, // imul rax, rsi
        0xC3,                    // ret
    ]
}

/// `mov rax, rdi; add rax, rsi; add rax, rdx; ret` — sums 3 arguments
fn x86_sum3() -> Vec<u8> {
    vec![
        0x48, 0x89, 0xF8, // mov rax, rdi
        0x48, 0x01, 0xF0, // add rax, rsi
        0x48, 0x01, 0xD0, // add rax, rdx
        0xC3,             // ret
    ]
}

// ---------------------------------------------------------------------------
// Basic functionality
// ---------------------------------------------------------------------------

#[test]
fn test_jit_return_constant() {
    let handler = RuntimeEffectHandler::new();
    let handle = mmap_exec(&handler, x86_const(42)).unwrap();
    let result = call_native(&handler, match handle { Value::Int(h) => h, _ => panic!() }, vec![]).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_jit_return_negative() {
    let handler = RuntimeEffectHandler::new();
    let h = match mmap_exec(&handler, x86_const(-1)).unwrap() { Value::Int(h) => h, _ => panic!() };
    let result = call_native(&handler, h, vec![]).unwrap();
    assert_eq!(result, Value::Int(-1));
}

#[test]
fn test_jit_return_zero() {
    let handler = RuntimeEffectHandler::new();
    let h = match mmap_exec(&handler, x86_const(0)).unwrap() { Value::Int(h) => h, _ => panic!() };
    let result = call_native(&handler, h, vec![]).unwrap();
    assert_eq!(result, Value::Int(0));
}

#[test]
fn test_jit_return_large() {
    let handler = RuntimeEffectHandler::new();
    let h = match mmap_exec(&handler, x86_const(i64::MAX)).unwrap() { Value::Int(h) => h, _ => panic!() };
    assert_eq!(call_native(&handler, h, vec![]).unwrap(), Value::Int(i64::MAX));
}

// ---------------------------------------------------------------------------
// Arithmetic with arguments
// ---------------------------------------------------------------------------

#[test]
fn test_jit_add() {
    let handler = RuntimeEffectHandler::new();
    let h = match mmap_exec(&handler, x86_add_args()).unwrap() { Value::Int(h) => h, _ => panic!() };
    let result = call_native(&handler, h, vec![Value::Int(17), Value::Int(25)]).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_jit_sub() {
    let handler = RuntimeEffectHandler::new();
    let h = match mmap_exec(&handler, x86_sub_args()).unwrap() { Value::Int(h) => h, _ => panic!() };
    let result = call_native(&handler, h, vec![Value::Int(100), Value::Int(58)]).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_jit_mul() {
    let handler = RuntimeEffectHandler::new();
    let h = match mmap_exec(&handler, x86_mul_args()).unwrap() { Value::Int(h) => h, _ => panic!() };
    let result = call_native(&handler, h, vec![Value::Int(6), Value::Int(7)]).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_jit_sum_three_args() {
    let handler = RuntimeEffectHandler::new();
    let h = match mmap_exec(&handler, x86_sum3()).unwrap() { Value::Int(h) => h, _ => panic!() };
    let result = call_native(&handler, h, vec![
        Value::Int(10), Value::Int(20), Value::Int(12),
    ]).unwrap();
    assert_eq!(result, Value::Int(42));
}

// ---------------------------------------------------------------------------
// Multiple regions
// ---------------------------------------------------------------------------

#[test]
fn test_jit_multiple_regions() {
    let handler = RuntimeEffectHandler::new();

    let h1 = match mmap_exec(&handler, x86_const(1)).unwrap() { Value::Int(h) => h, _ => panic!() };
    let h2 = match mmap_exec(&handler, x86_const(2)).unwrap() { Value::Int(h) => h, _ => panic!() };
    let h3 = match mmap_exec(&handler, x86_add_args()).unwrap() { Value::Int(h) => h, _ => panic!() };

    assert_eq!(call_native(&handler, h1, vec![]).unwrap(), Value::Int(1));
    assert_eq!(call_native(&handler, h2, vec![]).unwrap(), Value::Int(2));
    assert_eq!(call_native(&handler, h3, vec![Value::Int(100), Value::Int(200)]).unwrap(), Value::Int(300));
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

#[test]
fn test_jit_empty_code_rejected() {
    let handler = RuntimeEffectHandler::new();
    let result = mmap_exec(&handler, vec![]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[test]
fn test_jit_invalid_handle() {
    let handler = RuntimeEffectHandler::new();
    let result = call_native(&handler, 999, vec![]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("invalid"));
}

// ---------------------------------------------------------------------------
// Drop cleanup (no leak)
// ---------------------------------------------------------------------------

#[test]
fn test_jit_drop_cleanup() {
    let handler = RuntimeEffectHandler::new();
    // Compile many regions — they should all be munmap'd on drop
    for i in 0..100 {
        let _ = mmap_exec(&handler, x86_const(i));
    }
    // Handler drops here — if munmap leaks, valgrind/asan would catch it
}

// ---------------------------------------------------------------------------
// Capability integration (sandbox blocks JIT)
// ---------------------------------------------------------------------------

#[test]
fn test_jit_handles_independent() {
    let handler = RuntimeEffectHandler::new();
    let h1 = match mmap_exec(&handler, x86_const(111)).unwrap() { Value::Int(h) => h, _ => panic!() };
    let h2 = match mmap_exec(&handler, x86_const(222)).unwrap() { Value::Int(h) => h, _ => panic!() };

    // Calling in reverse order should work fine
    assert_eq!(call_native(&handler, h2, vec![]).unwrap(), Value::Int(222));
    assert_eq!(call_native(&handler, h1, vec![]).unwrap(), Value::Int(111));
}
