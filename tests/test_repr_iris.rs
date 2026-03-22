
//! Rust test runners for the 90 IRIS test functions in:
//!   - tests/fixtures/iris-testing/test_repr.iris
//!
//! Dependencies: src/iris-programs/repr/wire_format.iris, src/iris-programs/repr/hash.iris,
//!               src/iris-programs/repr/cost.iris, src/iris-programs/repr/eval.iris,
//!               src/iris-programs/repr/proof.iris, src/iris-programs/repr/guard.iris,
//!               src/iris-programs/repr/fragment_signature.iris, src/iris-programs/repr/types.iris,
//!               src/iris-programs/repr/resolution_level.iris,
//!               src/iris-programs/repr/abstract_machine.iris,
//!               src/iris-programs/repr/component.iris,
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
    let result = interpreter::interpret_with_step_limit(
        &graph.1,
        &[],
        None,
        Some(&module.registry),
        10_000_000,
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

const WIRE_FORMAT: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/repr/wire_format.iris"));

const HASH: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/repr/hash.iris"));

const COST: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/repr/cost.iris"));

const EVAL: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/repr/eval.iris"));

const PROOF: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/repr/proof.iris"));

const GUARD: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/repr/guard.iris"));

const FRAGMENT_SIGNATURE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/repr/fragment_signature.iris"));

const TYPES: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/repr/types.iris"));

const RESOLUTION_LEVEL: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/repr/resolution_level.iris"));

const ABSTRACT_MACHINE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/repr/abstract_machine.iris"));

const COMPONENT: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/repr/component.iris"));

const TEST_REPR: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/iris-testing/test_repr.iris"));

fn repr_module() -> CompiledModule {
    let combined = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        TEST_HARNESS,
        WIRE_FORMAT,
        HASH,
        COST,
        EVAL,
        PROOF,
        GUARD,
        FRAGMENT_SIGNATURE,
        TYPES,
        RESOLUTION_LEVEL,
        ABSTRACT_MACHINE,
        COMPONENT,
        TEST_REPR,
    );
    compile_module(&combined)
}

// ---------------------------------------------------------------------------
// Tests (87)
// ---------------------------------------------------------------------------

macro_rules! repr_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let module = repr_module();
            assert_test_passes(&module, stringify!($name));
        }
    };
}

// wire_format
repr_test!(test_wire_encode_u16_le);
repr_test!(test_wire_encode_u32_le);
repr_test!(test_wire_roundtrip_u16);
repr_test!(test_wire_roundtrip_u32);
repr_test!(test_wire_validate_header_valid);
repr_test!(test_wire_validate_header_bad_magic);
repr_test!(test_wire_validate_header_bad_version);
repr_test!(test_wire_magic_bytes);
repr_test!(test_wire_encode_node_kind);
repr_test!(test_wire_payload_sizes);
repr_test!(test_wire_encode_edge);
repr_test!(test_wire_encode_boundary);

// hash
repr_test!(test_hash_mix_deterministic);
repr_test!(test_hash_mix_distinct);
repr_test!(test_hash_mix_nonneg);
repr_test!(test_hash_verify_distinct_same);
repr_test!(test_hash_verify_distinct_diff);
repr_test!(test_hash_node_id_deterministic);
repr_test!(test_hash_node_id_different_kinds);
repr_test!(test_hash_blake3_g);
// blake3_compress tests are computationally impractical in the bootstrap evaluator
// (7-round BLAKE3 with nested function calls exceeds practical step budgets)
#[test]
#[ignore]
fn test_hash_blake3_compress() {
    let module = repr_module();
    assert_test_passes(&module, "test_hash_blake3_compress");
}
repr_test!(test_hash_salt_changes_id);
repr_test!(test_hash_type_deterministic);
repr_test!(test_hash_type_distinct);

// cost
repr_test!(test_cost_leaf_unknown);
repr_test!(test_cost_leaf_polynomial);
repr_test!(test_cost_leaf_sum);
repr_test!(test_cost_compare_equal);
repr_test!(test_cost_compare_less);
repr_test!(test_cost_universalize_hw);
repr_test!(test_cost_universalize_non_hw);

// eval
repr_test!(test_eval_tier_a);
repr_test!(test_eval_tier_b);
repr_test!(test_eval_tier_c);
repr_test!(test_eval_tier_test_count_a);
repr_test!(test_eval_tier_test_count_b);
repr_test!(test_eval_compute_ipc);
repr_test!(test_eval_compute_ipc_zero);
repr_test!(test_eval_valid_value_type);
repr_test!(test_eval_valid_value_type_max);
repr_test!(test_eval_invalid_value_type);

// proof
repr_test!(test_proof_tier_0);
repr_test!(test_proof_tier_1);
repr_test!(test_proof_tier_3);
repr_test!(test_proof_tier_time_0);
repr_test!(test_proof_tier_time_2);
repr_test!(test_proof_receipt_valid);
repr_test!(test_proof_receipt_invalid);

// guard
repr_test!(test_guard_valid_error_exact);
repr_test!(test_guard_valid_error_unverified);
repr_test!(test_guard_invalid_error);
repr_test!(test_guard_statistical_good);
repr_test!(test_guard_statistical_bad);
repr_test!(test_guard_compare_bounds);

// types
repr_test!(test_types_function_arrow);
repr_test!(test_types_function_prim);
repr_test!(test_types_recursive);
repr_test!(test_types_not_recursive);
repr_test!(test_types_polymorphic_forall);
repr_test!(test_types_polymorphic_exists);
repr_test!(test_types_not_polymorphic);

// resolution_level
repr_test!(test_resolution_depth_0);
repr_test!(test_resolution_depth_2);
repr_test!(test_resolution_visible);
repr_test!(test_resolution_not_visible);
repr_test!(test_resolution_visible_same);

// abstract_machine
repr_test!(test_machine_fits);
repr_test!(test_machine_exceeds_stack);
repr_test!(test_machine_backend_vm);
repr_test!(test_machine_backend_jit);

// component
repr_test!(test_component_classify_mutation);
repr_test!(test_component_classify_seed);
repr_test!(test_component_classify_invalid);
repr_test!(test_component_registry_contains);
repr_test!(test_component_registry_empty);
repr_test!(test_component_make_mutation);

// blake3 primitives
repr_test!(test_blake3_add32_wrap);
repr_test!(test_blake3_add32_basic);
repr_test!(test_blake3_xor8_identity);
repr_test!(test_blake3_xor8_self);
repr_test!(test_blake3_xor8_value);
repr_test!(test_blake3_xor32_identity);
repr_test!(test_blake3_xor32_self);
repr_test!(test_blake3_rotr32);
repr_test!(test_blake3_g_output);
#[test]
#[ignore]
fn test_blake3_compress_deterministic() {
    let module = repr_module();
    assert_test_passes(&module, "test_blake3_compress_deterministic");
}
#[test]
#[ignore]
fn test_blake3_compress_distinct() {
    let module = repr_module();
    assert_test_passes(&module, "test_blake3_compress_distinct");
}

// wire_format additional
repr_test!(test_wire_prepend_edge);
repr_test!(test_wire_prepend_node);
repr_test!(test_wire_hash32);
