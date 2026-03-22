
//! Rust test runners for the 39 IRIS test functions in:
//!   - tests/fixtures/iris-testing/test_capability_wiring.iris
//!
//! Dependencies: src/iris-programs/exec/capability_wiring.iris,
//!               tests/fixtures/iris-testing/test_harness.iris

use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct CompiledModule {
    bindings: Vec<(String, SemanticGraph)>,
    registry: FragmentRegistry,
}

fn compile_module(src: &str) -> CompiledModule {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!(
            "IRIS compilation failed with {} errors",
            result.errors.len()
        );
    }
    let mut registry = FragmentRegistry::new();
    let mut bindings = Vec::new();
    for (name, frag, _) in result.fragments {
        registry.register(frag.clone());
        bindings.push((name, frag.graph));
    }
    CompiledModule { bindings, registry }
}

fn assert_test_passes(module: &CompiledModule, name: &str) {
    let graph = module
        .bindings
        .iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("binding '{}' not found", name));
    let result = interpreter::interpret_with_registry(
        &graph.1,
        &[],
        None,
        Some(&module.registry),
    )
    .unwrap_or_else(|e| panic!("evaluation of '{}' failed: {:?}", name, e));
    let val = &result.0[0];
    match val {
        Value::Int(n) => assert!(
            *n > 0,
            "test '{}' failed: expected positive (pass), got {}",
            name,
            n
        ),
        other => panic!(
            "test '{}' returned non-Int: {:?}, expected positive Int",
            name, other
        ),
    }
}

// ---------------------------------------------------------------------------
// Source loading
// ---------------------------------------------------------------------------

const TEST_HARNESS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/iris-testing/test_harness.iris"));

const CAPABILITY_WIRING: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/exec/capability_wiring.iris"));

const TEST_CAPABILITY_WIRING: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/iris-testing/test_capability_wiring.iris"));

fn capability_wiring_module() -> CompiledModule {
    let combined = format!(
        "{}\n{}\n{}",
        TEST_HARNESS,
        CAPABILITY_WIRING,
        TEST_CAPABILITY_WIRING,
    );
    compile_module(&combined)
}

// ---------------------------------------------------------------------------
// Tests (39)
// ---------------------------------------------------------------------------

macro_rules! cap_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let module = capability_wiring_module();
            assert_test_passes(&module, stringify!($name));
        }
    };
}

// effect_name_to_tag
cap_test!(test_effect_tag_fileread);
cap_test!(test_effect_tag_filewrite);
cap_test!(test_effect_tag_print);
cap_test!(test_effect_tag_httpget);
cap_test!(test_effect_tag_threadspawn);
cap_test!(test_effect_tag_fficall);
cap_test!(test_effect_tag_mmapexec);
cap_test!(test_effect_tag_unknown);
cap_test!(test_effect_tag_wrong_len);

// pow2
cap_test!(test_pow2_zero);
cap_test!(test_pow2_four);
cap_test!(test_pow2_ten);
cap_test!(test_pow2_thirtytwo);
cap_test!(test_pow2_fortythree);
cap_test!(test_pow2_out_of_range);

// capability allow/deny
cap_test!(test_cap_allow_fileread);
cap_test!(test_cap_allow_two_effects);
cap_test!(test_cap_allow_threadspawn);
cap_test!(test_cap_deny_filewrite);
cap_test!(test_cap_allow_then_deny);
cap_test!(test_cap_empty_entries);

// enforce
cap_test!(test_enforce_unrestricted);
cap_test!(test_enforce_nothing);
cap_test!(test_enforce_fileread_only);
cap_test!(test_enforce_threadspawn_flag);
cap_test!(test_enforce_ffi_flag);
cap_test!(test_enforce_mmap_flag);

// compose
cap_test!(test_compose_restrict);
cap_test!(test_compose_identity);
cap_test!(test_compose_zero);

// violation
cap_test!(test_violation_allowed);
cap_test!(test_violation_denied_bitmask);
cap_test!(test_violation_threadspawn);
cap_test!(test_violation_fficall);
cap_test!(test_violation_mmapexec);

// end-to-end
cap_test!(test_e2e_allow_fileread_enforce);
cap_test!(test_e2e_deny_filewrite_enforce);
cap_test!(test_e2e_allow_deny_cancel);
cap_test!(test_e2e_compose_disjoint);
