//! CLCU backend — executes IRIS programs on the CLCU vectorized runtime.
//!
//! Pipeline:
//!   1. Load isel.iris + container_pack.iris (IRIS compiler passes)
//!   2. Compile user graph → CLCU micro-ops → packed containers
//!   3. Allocate arena, copy containers, execute chain via C FFI
//!   4. Read results from ZMM registers
//!
//! All compilation logic is pure IRIS. Rust provides the FFI bridge
//! to the C CLCU interpreter (iris-clcu/src/interpreter.c).
//!
//! Requires: `--features clcu`

#[cfg(feature = "clcu")]
mod inner {
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    use iris_clcu_sys::{
        self, arena_alloc, arena_create, arena_destroy, clcu_arena_t, clcu_container_t,
        exec_state_t, execute_chain, interpreter_init, CLCU_CONTAINER_SIZE, CLCU_FLAG_IS_ENTRY,
        CLCU_NEXT_TERMINAL, EXEC_STATUS_HALTED, EXEC_STATUS_OK,
    };
    use iris_types::eval::Value;
    use iris_types::graph::SemanticGraph;
    use iris_types::hash::SemanticHash;

    use crate::effect_runtime::RuntimeEffectHandler;

    // -------------------------------------------------------------------
    // Initialization
    // -------------------------------------------------------------------

    static INIT: std::sync::Once = std::sync::Once::new();

    fn ensure_init() {
        INIT.call_once(|| unsafe { interpreter_init() });
    }

    // -------------------------------------------------------------------
    // CLCU execution
    // -------------------------------------------------------------------

    /// Execute a simple integer computation on the CLCU runtime.
    ///
    /// Takes up to 8 i32 input values (loaded into zmm0 lanes),
    /// executes a single container chain, returns zmm0[0] as result.
    pub fn execute_clcu_container(
        containers: &[clcu_container_t],
        inputs: &[i32],
    ) -> Result<i32, String> {
        ensure_init();

        if containers.is_empty() {
            return Err("empty container chain".into());
        }

        unsafe {
            // Allocate arena
            let arena = arena_create();
            if arena.is_null() {
                return Err("arena_create failed".into());
            }

            // Allocate containers in arena
            let n = containers.len() as u32;
            let base = arena_alloc(arena, n);
            if base.is_null() {
                arena_destroy(arena);
                return Err("arena_alloc failed".into());
            }

            // Copy containers into arena memory
            for (i, c) in containers.iter().enumerate() {
                let ptr = base.add(i);
                std::ptr::copy_nonoverlapping(c, ptr, 1);
            }

            // Initialize execution state
            let mut state: exec_state_t = std::mem::zeroed();

            // Load inputs into zmm0 lanes
            for (i, &val) in inputs.iter().enumerate().take(16) {
                state.zmm[0][i] = val;
            }

            // Execute the container chain
            execute_chain(&mut state, base);

            // Clean up
            let result = state.zmm[0][0]; // Result in zmm0[0]
            arena_destroy(arena);

            Ok(result)
        }
    }

    /// Check if CLCU hardware is available (AVX-512 or scalar fallback).
    pub fn is_clcu_available() -> bool {
        // CLCU always works — scalar fallback is built-in
        true
    }

    /// Create a single terminal container with one micro-op for testing.
    pub fn make_test_container(opcode: u8, dst: u8, src1: u8, src2: u8) -> clcu_container_t {
        let mut c = clcu_container_t::default();
        c.magic_and_version = 0xC100;
        c.flags = CLCU_FLAG_IS_ENTRY;
        c.next_container = CLCU_NEXT_TERMINAL;
        let op = iris_clcu_sys::encode_micro_op(opcode, dst, src1, src2, 0, 0, 0);
        c.payload[..6].copy_from_slice(&op);
        c
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_clcu_init() {
            ensure_init();
            // Should not crash
        }

        #[test]
        fn test_clcu_simple_vadd() {
            // Create a single container with: VADD zmm0, zmm0, zmm1
            // zmm0 = inputs, zmm1 loaded with constant
            let mut container: clcu_container_t = unsafe { std::mem::zeroed() };
            container.magic_and_version = 0xC100; // magic=0xC, version=1
            container.flags = CLCU_FLAG_IS_ENTRY;
            container.next_container = CLCU_NEXT_TERMINAL;

            // Encode VADD: opcode=0x01, dst=0, src1=0, src2=0
            let op = iris_clcu_sys::encode_micro_op(0x01, 0, 0, 0, 0, 0, 0);
            container.payload[..6].copy_from_slice(&op);

            // zmm0[0] = 21, add zmm0 to itself → should get 42
            let result = execute_clcu_container(&[container], &[21]);
            assert_eq!(result, Ok(42));
        }
    }
}

#[cfg(feature = "clcu")]
pub use inner::*;
