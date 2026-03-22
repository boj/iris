
//! Rust test runners for the 69 IRIS test functions in:
//!   - tests/fixtures/iris-testing/test_deploy.iris
//!
//! Dependencies: src/iris-programs/deploy/elf_native.iris,
//!               src/iris-programs/deploy/serialize_bytecode.iris,
//!               src/iris-programs/deploy/standalone.iris,
//!               src/iris-programs/deploy/shared_lib.iris,
//!               src/iris-programs/compiler/elf_wrapper.iris,
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

const ELF_NATIVE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/deploy/elf_native.iris"));

const SERIALIZE_BYTECODE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/deploy/serialize_bytecode.iris"));

const STANDALONE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/deploy/standalone.iris"));

const SHARED_LIB: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/deploy/shared_lib.iris"));

const ELF_WRAPPER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/compiler/elf_wrapper.iris"));

const TEST_DEPLOY: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/iris-testing/test_deploy.iris"));

fn deploy_module() -> CompiledModule {
    let combined = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        TEST_HARNESS,
        ELF_NATIVE,
        SERIALIZE_BYTECODE,
        STANDALONE,
        SHARED_LIB,
        ELF_WRAPPER,
        TEST_DEPLOY,
    );
    compile_module(&combined)
}

// ---------------------------------------------------------------------------
// Tests (67)
// ---------------------------------------------------------------------------

macro_rules! deploy_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let module = deploy_module();
            assert_test_passes(&module, stringify!($name));
        }
    };
}

// elf_native
deploy_test!(test_elf_build_ident);
deploy_test!(test_elf_validate_ident_correct);
deploy_test!(test_elf_validate_ident_bad_magic);
deploy_test!(test_elf_validate_code_valid);
deploy_test!(test_elf_validate_code_zero);
deploy_test!(test_elf_validate_elf_valid);
deploy_test!(test_elf_validate_elf_bad_args);
deploy_test!(test_elf_validate_elf_zero_code);
deploy_test!(test_elf_type_machine);
deploy_test!(test_elf_phdr_type);
deploy_test!(test_elf_phdr_flags);
deploy_test!(test_elf_sizes);

// serialize_bytecode
deploy_test!(test_serialize_magic);
deploy_test!(test_serialize_encode_u32);
deploy_test!(test_serialize_valid_opcode);
deploy_test!(test_serialize_invalid_opcode);
deploy_test!(test_serialize_valid_header);
deploy_test!(test_serialize_invalid_header);
deploy_test!(test_serialize_roundtrip_header);

// standalone
deploy_test!(test_standalone_step_add);
deploy_test!(test_standalone_step_sub);
deploy_test!(test_standalone_step_mul);
deploy_test!(test_standalone_step_div_zero);
deploy_test!(test_standalone_step_eq);
deploy_test!(test_standalone_step_lt);
deploy_test!(test_standalone_jump);
deploy_test!(test_standalone_parse_digit_0);
deploy_test!(test_standalone_parse_digit_9);
deploy_test!(test_standalone_parse_digit_A);
deploy_test!(test_standalone_count_digits_0);
deploy_test!(test_standalone_count_digits_42);
deploy_test!(test_standalone_count_digits_12345);
deploy_test!(test_standalone_fold_step_add);
deploy_test!(test_standalone_fold_step_mul);
deploy_test!(test_standalone_fold_step_min);
deploy_test!(test_standalone_fold_step_max);

// shared_lib
deploy_test!(test_shared_lib_arith_add);
deploy_test!(test_shared_lib_arith_sub);
deploy_test!(test_shared_lib_arith_mul);
deploy_test!(test_shared_lib_arith_div);
deploy_test!(test_shared_lib_arith_div_zero);
deploy_test!(test_shared_lib_compare_lt);
deploy_test!(test_shared_lib_compare_le);
deploy_test!(test_shared_lib_compare_lt_false);
deploy_test!(test_shared_lib_validate_valid);
deploy_test!(test_shared_lib_validate_too_many);
deploy_test!(test_shared_lib_validate);
deploy_test!(test_shared_lib_validate_small);
deploy_test!(test_shared_lib_instr_size_load);
deploy_test!(test_shared_lib_instr_size_loadarg);
deploy_test!(test_shared_lib_instr_size_add);

// integration
deploy_test!(test_serialize_program_nonempty);
deploy_test!(test_serialize_program_magic);
deploy_test!(test_elf_build_nonempty);
deploy_test!(test_elf_build_magic);
deploy_test!(test_shared_lib_repr_valid);
deploy_test!(test_shared_lib_repr_invalid);
deploy_test!(test_serialize_bytecode_magic);
deploy_test!(test_serialize_header_fields);
deploy_test!(test_serialize_prepend_int);
deploy_test!(test_shared_lib_generate_valid);
deploy_test!(test_shared_lib_generate_bad_args);
deploy_test!(test_shared_lib_vm_step_load);
deploy_test!(test_shared_lib_vm_step_add);
deploy_test!(test_shared_lib_vm_step_eq);
deploy_test!(test_shared_lib_dispatch);
deploy_test!(test_elf_generate_magic);
deploy_test!(test_elf_header_bytes_magic);
deploy_test!(test_elf_phdr_bytes_type);
