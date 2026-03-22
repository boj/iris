
//! Rust test runners for the 132 IRIS test functions in:
//!   - tests/fixtures/iris-testing/test_analyzer.iris (25 tests)
//!   - tests/fixtures/iris-testing/test_evolution.iris (66 tests)
//!   - tests/fixtures/iris-testing/test_meta.iris (41 tests)
//!
//! Each test .iris file is loaded along with its dependencies, compiled via
//! iris_bootstrap::syntax::compile(), and each test_* binding is evaluated through the
//! interpreter with a FragmentRegistry for cross-fragment Ref resolution.
//! A result of 1 means pass, -1 means fail.

use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// Helpers: compile IRIS source and extract named bindings
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

// Analyzer dependencies
const DETECT_IDENTITY: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/analyzer/detect_identity.iris"));
const DETECT_CONSTANT: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/analyzer/detect_constant.iris"));
const DETECT_SUM: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/analyzer/detect_sum.iris"));
const DETECT_MAX: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/analyzer/detect_max.iris"));
const DETECT_PRODUCT: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/analyzer/detect_product.iris"));
const DETECT_NEGATION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/analyzer/detect_negation.iris"));
const DETECT_ABSOLUTE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/analyzer/detect_absolute.iris"));
const DETECT_LINEAR: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/analyzer/detect_linear.iris"));
const DETECT_BINARY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/iris-programs/analyzer/detect_binary_arithmetic.iris"
));
const DETECT_FOLD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/iris-programs/analyzer/detect_fold_reduction.iris"
));
const DETECT_MAP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/iris-programs/analyzer/detect_map_transform.iris"
));
const DETECT_LINEAR_TRANSFORM: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/iris-programs/analyzer/detect_linear_transform.iris"
));
const BUILD_SKELETON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/analyzer/build_skeleton.iris"));

// Test harness
const TEST_HARNESS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/iris-testing/test_harness.iris"));

// Test files
const TEST_ANALYZER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/iris-testing/test_analyzer.iris"));

// Evolution dependencies
const NOVELTY_SEARCH: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/iris-programs/evolution/novelty_search.iris"
));
const MAP_ELITES: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/evolution/map_elites.iris"));
const PHASE_DETECT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/iris-programs/evolution/phase_detect.iris"
));
const RESOURCE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/evolution/resource.iris"));
const MIGRATION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/evolution/migration.iris"));
const CHECKPOINT: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/evolution/checkpoint.iris"));
const COEVOLUTION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/evolution/coevolution.iris"));
const ENUMERATE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/evolution/enumerate.iris"));
const COMPONENT_BRIDGE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/iris-programs/evolution/component_bridge.iris"
));
const ECOSYSTEM: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/evolution/ecosystem.iris"));
const STIGMERGY: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/evolution/stigmergy.iris"));
const FULL_EVOLVE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/iris-programs/evolution/full_evolve_loop.iris"
));

const TEST_EVOLUTION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/iris-testing/test_evolution.iris"));

// Meta dependencies
const IMPROVEMENT_TRACKER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/iris-programs/meta/improvement_tracker.iris"
));
const AUTO_IMPROVE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/meta/auto_improve.iris"));
const DAEMON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/meta/daemon.iris"));
const META_EVOLVE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/iris-programs/evolution/meta_evolve.iris"
));

const TEST_META: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/iris-testing/test_meta.iris"));

// ---------------------------------------------------------------------------
// Analyzer tests (25 tests)
// ---------------------------------------------------------------------------

fn analyzer_module() -> CompiledModule {
    let combined = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        TEST_HARNESS,
        DETECT_IDENTITY,
        DETECT_CONSTANT,
        DETECT_SUM,
        DETECT_MAX,
        DETECT_PRODUCT,
        DETECT_NEGATION,
        DETECT_ABSOLUTE,
        DETECT_LINEAR,
        DETECT_BINARY,
        DETECT_FOLD,
        DETECT_MAP,
        DETECT_LINEAR_TRANSFORM,
        BUILD_SKELETON,
        TEST_ANALYZER,
    );
    compile_module(&combined)
}

macro_rules! analyzer_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let module = analyzer_module();
            assert_test_passes(&module, stringify!($name));
        }
    };
}

analyzer_test!(test_identity_detection);
analyzer_test!(test_identity_negative);
analyzer_test!(test_constant_detection);
analyzer_test!(test_constant_negative);
analyzer_test!(test_sum_detection);
analyzer_test!(test_max_detection);
analyzer_test!(test_product_detection);
analyzer_test!(test_negation_detection);
analyzer_test!(test_absolute_detection);
analyzer_test!(test_linear_detection);
analyzer_test!(test_binary_add);
analyzer_test!(test_binary_sub);
analyzer_test!(test_binary_mul);
analyzer_test!(test_fold_sum);
analyzer_test!(test_fold_product);
analyzer_test!(test_fold_max);
analyzer_test!(test_map_double);
analyzer_test!(test_map_square);
analyzer_test!(test_linear_transform_3x2);
analyzer_test!(test_linear_coefficients);
analyzer_test!(test_linear_negative_slope);
analyzer_test!(test_skeleton_identity);
analyzer_test!(test_skeleton_sum);
analyzer_test!(test_skeleton_binary_add);
analyzer_test!(test_build_dispatch);

// ---------------------------------------------------------------------------
// Evolution tests (66 tests)
// ---------------------------------------------------------------------------

fn evolution_module() -> CompiledModule {
    // Order matters: load dependency files first, then full_evolve_loop.iris
    // (which overrides some function signatures with 1-arg versions that the
    // tests expect), then test file last.
    let combined = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        TEST_HARNESS,
        NOVELTY_SEARCH,
        MAP_ELITES,
        PHASE_DETECT,
        RESOURCE,
        MIGRATION,
        CHECKPOINT,
        COEVOLUTION,
        ENUMERATE,
        COMPONENT_BRIDGE,
        ECOSYSTEM,
        STIGMERGY,
        BUILD_SKELETON,
        // full_evolve_loop LAST among deps: its 1-arg versions of
        // is_stagnating, should_migrate, ring_dest, etc. override the
        // multi-arg versions from the dependency files.
        FULL_EVOLVE,
        META_EVOLVE,
        TEST_EVOLUTION,
    );
    compile_module(&combined)
}

macro_rules! evolution_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let module = evolution_module();
            assert_test_passes(&module, stringify!($name));
        }
    };
}

// Fitness evaluation
evolution_test!(test_eval_perfect);
evolution_test!(test_node_count);
evolution_test!(test_simplicity_high);

// Novelty search
evolution_test!(test_novelty_same);
evolution_test!(test_novelty_distance);
evolution_test!(test_novelty_empty_archive);
evolution_test!(test_novelty_add);
evolution_test!(test_novelty_reject);
evolution_test!(test_adaptive_threshold_up);
evolution_test!(test_adaptive_threshold_down);

// NSGA-II (using full_evolve_loop's 3-obj dominates)
evolution_test!(test_nsga_dominates);
evolution_test!(test_nsga_no_dominate);
evolution_test!(test_nsga_tournament_rank);
evolution_test!(test_nsga_tournament_crowd);

// MAP-Elites
evolution_test!(test_mapelites_bin_small);
evolution_test!(test_mapelites_bin_large);
evolution_test!(test_mapelites_index);
evolution_test!(test_mapelites_empty);
evolution_test!(test_mapelites_insert);
evolution_test!(test_mapelites_replace);

// Phase detection
evolution_test!(test_phase_exploration);
evolution_test!(test_phase_exploitation);
evolution_test!(test_phase_steady);
evolution_test!(test_phase_mutation_rate);
evolution_test!(test_phase_transition);

// Resource competition
evolution_test!(test_resource_top);
evolution_test!(test_resource_bottom);
evolution_test!(test_resource_middle);

// Migration
evolution_test!(test_migrate_yes);
evolution_test!(test_migrate_no);
evolution_test!(test_ring_dest);

// Stagnation
evolution_test!(test_stag_reset);
evolution_test!(test_stag_increment);
evolution_test!(test_stag_detected);

// Checkpoints
evolution_test!(test_checkpoint_size);
evolution_test!(test_checkpoint_interval);
evolution_test!(test_checkpoint_stale);
evolution_test!(test_checkpoint_valid);
evolution_test!(test_checkpoint_invalid);

// Coevolution
evolution_test!(test_coevol_prog_fitness);
evolution_test!(test_coevol_test_fitness);
evolution_test!(test_coevol_intensity);
evolution_test!(test_coevol_red_queen);

// Enumeration
evolution_test!(test_enum_signature);
evolution_test!(test_enum_matches);
evolution_test!(test_enum_no_match);
evolution_test!(test_enum_novel);
evolution_test!(test_enum_not_novel);

// Component bridge
evolution_test!(test_bridge_valid_mutation);
evolution_test!(test_bridge_valid_seed);
evolution_test!(test_bridge_invalid);
evolution_test!(test_bridge_seed_fold_add);
evolution_test!(test_bridge_seed_fold_max);
evolution_test!(test_bridge_seed_map_fold);

// Ecosystem
evolution_test!(test_eco_keystone_yes);
evolution_test!(test_eco_keystone_no);
evolution_test!(test_eco_decaying);
evolution_test!(test_eco_prune);
evolution_test!(test_eco_protect_keystone);
evolution_test!(test_eco_turnover);

// Stigmergy
evolution_test!(test_stig_quantize);
evolution_test!(test_stig_avg_fitness);
evolution_test!(test_stig_failure_rate);
evolution_test!(test_stig_decay);
evolution_test!(test_stig_negligible);
evolution_test!(test_stig_net_signal);

// ---------------------------------------------------------------------------
// Meta tests (41 tests)
// ---------------------------------------------------------------------------

fn meta_module() -> CompiledModule {
    let combined = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        TEST_HARNESS,
        IMPROVEMENT_TRACKER,
        AUTO_IMPROVE,
        DAEMON,
        META_EVOLVE,
        TEST_META,
    );
    compile_module(&combined)
}

macro_rules! meta_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let module = meta_module();
            assert_test_passes(&module, stringify!($name));
        }
    };
}

// Improvement tracker
meta_test!(test_tracker_mean);
meta_test!(test_tracker_mean_empty);
meta_test!(test_tracker_slope_flat);
meta_test!(test_tracker_slope_up);
meta_test!(test_tracker_slope_down);
meta_test!(test_tracker_compounding_yes);
meta_test!(test_tracker_compounding_no);
meta_test!(test_tracker_op_record);
meta_test!(test_tracker_op_no_improve);
meta_test!(test_tracker_success_rate);
meta_test!(test_tracker_avg_improvement);
meta_test!(test_tracker_pph);
meta_test!(test_tracker_adaptive_low);
meta_test!(test_tracker_adaptive_high);

// Auto-improve
meta_test!(test_auto_gate_pass);
meta_test!(test_auto_gate_fail_corr);
meta_test!(test_auto_gate_fail_slow);
meta_test!(test_auto_slowdown_equal);
meta_test!(test_auto_slowdown_2x);
meta_test!(test_auto_correctness_all);
meta_test!(test_auto_correctness_half);
meta_test!(test_auto_count_tests);

// Daemon
meta_test!(test_daemon_stag_yes);
meta_test!(test_daemon_stag_no);
meta_test!(test_daemon_converged);
meta_test!(test_daemon_not_converged);
meta_test!(test_daemon_mode_improve);
meta_test!(test_daemon_mode_explore);
meta_test!(test_daemon_mode_reset);
meta_test!(test_daemon_mode_stag);
meta_test!(test_daemon_continue_solved);
meta_test!(test_daemon_continue_budget);
meta_test!(test_daemon_continue_yes);

// Meta-evolution
meta_test!(test_meta_perturb_positive);
meta_test!(test_meta_perturb_nonzero);
meta_test!(test_meta_allowed);
meta_test!(test_meta_rejected);
meta_test!(test_meta_scale_pop);
meta_test!(test_meta_scale_gens);
meta_test!(test_meta_plan);
meta_test!(test_meta_plan_deep);
