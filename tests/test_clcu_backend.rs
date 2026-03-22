//! Integration tests for the CLCU backend.
//!
//! Tests the CLCU backend through iris_exec::clcu_backend, which bridges
//! the IRIS runtime to the C CLCU interpreter via FFI.
//!
//! Run with: cargo test --release --features rust-scaffolding,jit,clcu --test test_clcu_backend

#[cfg(feature = "clcu")]
mod clcu_tests {
    use iris_exec::clcu_backend::{execute_clcu_container, is_clcu_available};

    #[test]
    fn test_clcu_available() {
        assert!(is_clcu_available());
    }

    #[test]
    fn test_clcu_empty_chain_error() {
        let result = execute_clcu_container(&[], &[]);
        assert!(result.is_err(), "empty chain should error");
    }

    #[test]
    fn test_clcu_vadd_doubles_input() {
        // VADD zmm0, zmm0, zmm0 → zmm0[0] = input * 2
        let mut c = iris_exec::clcu_backend::make_test_container(
            0x01, 0, 0, 0, // VADD z0, z0, z0
        );
        let result = execute_clcu_container(&[c], &[21]);
        assert_eq!(result, Ok(42), "21 + 21 = 42");
    }

    #[test]
    fn test_clcu_vadd_negative() {
        let c = iris_exec::clcu_backend::make_test_container(0x01, 0, 0, 0);
        let result = execute_clcu_container(&[c], &[-5]);
        assert_eq!(result, Ok(-10), "-5 + -5 = -10");
    }

    #[test]
    fn test_clcu_vadd_zero() {
        let c = iris_exec::clcu_backend::make_test_container(0x01, 0, 0, 0);
        let result = execute_clcu_container(&[c], &[0]);
        assert_eq!(result, Ok(0), "0 + 0 = 0");
    }

    #[test]
    fn test_clcu_multiple_inputs() {
        // VADD zmm0, zmm0, zmm0 doubles all lanes
        let c = iris_exec::clcu_backend::make_test_container(0x01, 0, 0, 0);
        // zmm0[0]=5 → result = 10
        let result = execute_clcu_container(&[c], &[5, 10, 15]);
        assert_eq!(result, Ok(10), "5+5=10 in lane 0");
    }
}
