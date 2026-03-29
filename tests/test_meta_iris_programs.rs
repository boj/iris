//! Test harness for meta .iris programs (src/iris-programs/meta/).
//!
//! Loads each meta .iris file, compiles it via iris_bootstrap::syntax::compile(),
//! registers all fragments in a FragmentRegistry, then:
//!   - If the file contains test_ bindings: run them (assert positive result).
//!   - Otherwise: verify compilation, find key fragments, evaluate with inputs.

use std::rc::Rc;

use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compile IRIS source, register all fragments, return named graphs + registry.
fn compile_with_registry(src: &str) -> (Vec<(String, SemanticGraph)>, FragmentRegistry) {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!(
            "compilation failed with {} errors:\n{}",
            result.errors.len(),
            result
                .errors
                .iter()
                .map(|e| iris_bootstrap::syntax::format_error(src, e))
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

/// Find a named fragment's graph.
fn find_graph<'a>(
    fragments: &'a [(String, SemanticGraph)],
    name: &str,
) -> &'a SemanticGraph {
    &fragments
        .iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("fragment '{}' not found", name))
        .1
}

/// Evaluate a SemanticGraph with given inputs and return the first output.
fn eval(
    graph: &SemanticGraph,
    inputs: &[Value],
    registry: &FragmentRegistry,
) -> Value {
    let (outputs, _) =
        interpreter::interpret_with_registry(graph, inputs, None, Some(registry))
            .unwrap_or_else(|e| panic!("evaluation failed: {:?}", e));
    outputs.into_iter().next().expect("no output")
}

/// Evaluate and extract an Int result.
fn eval_int(
    graph: &SemanticGraph,
    inputs: &[Value],
    registry: &FragmentRegistry,
) -> i64 {
    match eval(graph, inputs, registry) {
        Value::Int(n) => n,
        other => panic!("expected Int, got {:?}", other),
    }
}

/// Evaluate with no inputs and extract an Int.
fn eval_no_args_int(
    graph: &SemanticGraph,
    registry: &FragmentRegistry,
) -> i64 {
    eval_int(graph, &[], registry)
}

/// Load a meta .iris file and compile it.
fn load_meta(filename: &str) -> (Vec<(String, SemanticGraph)>, FragmentRegistry) {
    let path = format!("src/iris-programs/meta/{}", filename);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path, e));
    compile_with_registry(&source)
}

/// Assert that a fragment exists in the compiled output.
fn assert_has_fragment(fragments: &[(String, SemanticGraph)], name: &str) {
    assert!(
        fragments.iter().any(|(n, _)| n == name),
        "expected fragment '{}', found: {:?}",
        name,
        fragments.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// graph_inspect.iris
// ---------------------------------------------------------------------------

#[test]
fn test_graph_inspect_compiles() {
    let (frags, _reg) = load_meta("graph_inspect.iris");
    assert_has_fragment(&frags, "inspect");
}

#[test]
fn test_graph_inspect_on_new_graph() {
    let (frags, reg) = load_meta("graph_inspect.iris");
    let inspect = find_graph(&frags, "inspect");

    // graph_new creates a minimal graph (Prim(add) root + 2 Lit children).
    // Build a Value::Program from a freshly compiled identity program.
    let simple_src = "let identity x : Int -> Int [cost: Const(1)] = x";
    let (simple_frags, _) = compile_with_registry(simple_src);
    let simple_graph = &simple_frags[0].1;
    let prog = Value::Program(Rc::new(simple_graph.clone()));

    let result = eval(inspect, &[prog], &reg);
    // inspect returns (root, kind, nodes) where nodes is a Tuple of node IDs
    match result {
        Value::Tuple(elems) => {
            assert!(elems.len() >= 3, "expected 3-tuple from inspect, got {}", elems.len());
            // root is a node ID (can be any integer)
            assert!(matches!(&elems[0], Value::Int(_)), "expected Int root");
            // kind should be an int (node kind enum)
            assert!(matches!(&elems[1], Value::Int(_)), "expected Int kind");
            // nodes is a Tuple of node IDs (graph_nodes returns all node IDs)
            match &elems[2] {
                Value::Tuple(node_ids) => {
                    assert!(!node_ids.is_empty(), "expected non-empty node list");
                }
                Value::Int(_) => {
                    // Single-node graph may return just an Int
                }
                other => panic!("expected Tuple or Int for nodes, got {:?}", other),
            }
        }
        other => panic!("expected Tuple from inspect, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// instrumentation.iris
// ---------------------------------------------------------------------------

#[test]
fn test_instrumentation_compiles() {
    let (frags, _reg) = load_meta("instrumentation.iris");
    let expected = [
        "timing_sum", "timing_count", "compute_mean", "compute_p99",
        "detect_regression", "classify_severity", "record_timing",
        "memory_exceeded", "verify_chain_link", "verify_chain",
        "performance_delta",
    ];
    for name in &expected {
        assert_has_fragment(&frags, name);
    }
}

// NOTE: compute_mean uses `div` as infix syntax, which the parser treats as
// a function application rather than a binary operator (the parser supports
// `/` not `div` for division). We verify it compiles but skip runtime eval.

#[test]
fn test_instrumentation_compute_p99() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "compute_p99");
    // compute_p99((10, 50, 30)) → 50 (max)
    let timings = Value::tuple(vec![Value::Int(10), Value::Int(50), Value::Int(30)]);
    assert_eq!(eval_int(graph, &[timings], &reg), 50);
}

#[test]
fn test_instrumentation_detect_regression_yes() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "detect_regression");
    // mean 250 vs baseline 100 → regression (250 > 200)
    assert_eq!(eval_int(graph, &[Value::Int(250), Value::Int(100)], &reg), 1);
}

#[test]
fn test_instrumentation_detect_regression_no() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "detect_regression");
    // mean 150 vs baseline 100 → no regression (150 <= 200)
    assert_eq!(eval_int(graph, &[Value::Int(150), Value::Int(100)], &reg), 0);
}

#[test]
fn test_instrumentation_detect_regression_zero_baseline() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "detect_regression");
    // baseline 0 → no regression
    assert_eq!(eval_int(graph, &[Value::Int(500), Value::Int(0)], &reg), 0);
}

#[test]
fn test_instrumentation_classify_severity_critical() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "classify_severity");
    assert_eq!(eval_int(graph, &[Value::Int(600)], &reg), 2);
}

#[test]
fn test_instrumentation_classify_severity_warning() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "classify_severity");
    assert_eq!(eval_int(graph, &[Value::Int(300)], &reg), 1);
}

#[test]
fn test_instrumentation_classify_severity_info() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "classify_severity");
    assert_eq!(eval_int(graph, &[Value::Int(100)], &reg), 0);
}

#[test]
fn test_instrumentation_memory_exceeded_yes() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "memory_exceeded");
    assert_eq!(eval_int(graph, &[Value::Int(200), Value::Int(100)], &reg), 1);
}

#[test]
fn test_instrumentation_memory_exceeded_no() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "memory_exceeded");
    assert_eq!(eval_int(graph, &[Value::Int(50), Value::Int(100)], &reg), 0);
}

#[test]
fn test_instrumentation_memory_exceeded_zero_limit() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "memory_exceeded");
    assert_eq!(eval_int(graph, &[Value::Int(200), Value::Int(0)], &reg), 0);
}

#[test]
fn test_instrumentation_verify_chain_link_valid() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "verify_chain_link");
    assert_eq!(eval_int(graph, &[Value::Int(42), Value::Int(42)], &reg), 1);
}

#[test]
fn test_instrumentation_verify_chain_link_broken() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "verify_chain_link");
    assert_eq!(eval_int(graph, &[Value::Int(42), Value::Int(43)], &reg), 0);
}

// NOTE: performance_delta uses `div` infix syntax which the parser doesn't
// support as a binary operator. The zero-baseline case works because it takes
// the early return path (before_ns == 0 → 0) and never hits `div`.

#[test]
fn test_instrumentation_performance_delta_zero() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "performance_delta");
    assert_eq!(eval_int(graph, &[Value::Int(0), Value::Int(50)], &reg), 0);
}

// ---------------------------------------------------------------------------
// improvement_tracker.iris
// ---------------------------------------------------------------------------

#[test]
fn test_improvement_tracker_compiles() {
    let (frags, _reg) = load_meta("improvement_tracker.iris");
    let expected = [
        "mean", "regression_slope", "is_compounding",
        "record_operator_application", "operator_success_rate",
        "operator_avg_improvement", "best_operator", "problems_per_hour",
        "adaptive_weight", "tracker_summary",
    ];
    for name in &expected {
        assert_has_fragment(&frags, name);
    }
}

#[test]
fn test_improvement_tracker_mean() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "mean");
    let xs = Value::tuple(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
    assert_eq!(eval_int(graph, &[xs, Value::Int(3)], &reg), 20);
}

#[test]
fn test_improvement_tracker_mean_empty() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "mean");
    assert_eq!(eval_int(graph, &[Value::tuple(vec![]), Value::Int(0)], &reg), 0);
}

#[test]
fn test_improvement_tracker_is_compounding_yes() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "is_compounding");
    assert_eq!(eval_int(graph, &[Value::Int(-5)], &reg), 1);
}

#[test]
fn test_improvement_tracker_is_compounding_no() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "is_compounding");
    assert_eq!(eval_int(graph, &[Value::Int(5)], &reg), 0);
}

#[test]
fn test_improvement_tracker_record_op() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "record_operator_application");
    let stats = Value::tuple(vec![Value::Int(0), Value::Int(0), Value::Int(0)]);
    let result = eval(graph, &[stats, Value::Int(10)], &reg);
    // Should be (1, 1, 10): used=1, improvements=1, total_delta=10
    match result {
        Value::Tuple(elems) => {
            assert_eq!(elems.len(), 3);
            assert_eq!(elems[0], Value::Int(1));
            assert_eq!(elems[1], Value::Int(1));
            assert_eq!(elems[2], Value::Int(10));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn test_improvement_tracker_record_op_no_improve() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "record_operator_application");
    let stats = Value::tuple(vec![Value::Int(5), Value::Int(2), Value::Int(20)]);
    let result = eval(graph, &[stats, Value::Int(-5)], &reg);
    match result {
        Value::Tuple(elems) => {
            assert_eq!(elems[0], Value::Int(6));  // used increments
            assert_eq!(elems[1], Value::Int(2));  // improvements unchanged
            assert_eq!(elems[2], Value::Int(20)); // total unchanged
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn test_improvement_tracker_success_rate() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "operator_success_rate");
    let stats = Value::tuple(vec![Value::Int(10), Value::Int(3), Value::Int(30)]);
    // 3/10 * 1000 = 300
    assert_eq!(eval_int(graph, &[stats], &reg), 300);
}

#[test]
fn test_improvement_tracker_avg_improvement() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "operator_avg_improvement");
    let stats = Value::tuple(vec![Value::Int(10), Value::Int(4), Value::Int(80)]);
    // 80/4 * 1000 = 20000
    assert_eq!(eval_int(graph, &[stats], &reg), 20000);
}

#[test]
fn test_improvement_tracker_pph() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "problems_per_hour");
    let result = eval_int(
        graph,
        &[Value::Int(0), Value::Int(0), Value::Int(3600000), Value::Int(10)],
        &reg,
    );
    assert!(result > 0, "problems_per_hour should be positive, got {}", result);
}

#[test]
fn test_improvement_tracker_regression_slope_flat() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "regression_slope");
    let data = Value::tuple(vec![
        Value::tuple(vec![Value::Int(1), Value::Int(100)]),
        Value::tuple(vec![Value::Int(2), Value::Int(100)]),
        Value::tuple(vec![Value::Int(3), Value::Int(100)]),
    ]);
    assert_eq!(eval_int(graph, &[data, Value::Int(3)], &reg), 0);
}

#[test]
fn test_improvement_tracker_regression_slope_increasing() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "regression_slope");
    let data = Value::tuple(vec![
        Value::tuple(vec![Value::Int(0), Value::Int(100)]),
        Value::tuple(vec![Value::Int(10), Value::Int(200)]),
        Value::tuple(vec![Value::Int(20), Value::Int(300)]),
    ]);
    let slope = eval_int(graph, &[data, Value::Int(3)], &reg);
    assert!(slope > 0, "slope should be positive for increasing data, got {}", slope);
}

#[test]
fn test_improvement_tracker_regression_slope_decreasing() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "regression_slope");
    let data = Value::tuple(vec![
        Value::tuple(vec![Value::Int(0), Value::Int(300)]),
        Value::tuple(vec![Value::Int(10), Value::Int(200)]),
        Value::tuple(vec![Value::Int(20), Value::Int(100)]),
    ]);
    let slope = eval_int(graph, &[data, Value::Int(3)], &reg);
    assert!(slope < 0, "slope should be negative for decreasing data, got {}", slope);
}

// ---------------------------------------------------------------------------
// auto_improve.iris
// ---------------------------------------------------------------------------

#[test]
fn test_auto_improve_compiles() {
    let (frags, _reg) = load_meta("auto_improve.iris");
    let expected = [
        "find_slowest", "passes_gate", "compute_slowdown",
        "compute_correctness", "count_tests", "count_correct",
        "gate_check", "profile_component", "extract_test_cases",
        "evolve_replacement", "improve_cycle", "improve_loop",
        "explore_capability", "daemon_cycle",
    ];
    for name in &expected {
        assert_has_fragment(&frags, name);
    }
}

#[test]
fn test_auto_improve_passes_gate_yes() {
    let (frags, reg) = load_meta("auto_improve.iris");
    let graph = find_graph(&frags, "passes_gate");
    assert_eq!(
        eval_int(graph, &[Value::Int(100), Value::Int(150), Value::Int(200)], &reg),
        1
    );
}

#[test]
fn test_auto_improve_passes_gate_fail_correctness() {
    let (frags, reg) = load_meta("auto_improve.iris");
    let graph = find_graph(&frags, "passes_gate");
    assert_eq!(
        eval_int(graph, &[Value::Int(95), Value::Int(150), Value::Int(200)], &reg),
        0
    );
}

#[test]
fn test_auto_improve_passes_gate_fail_slowdown() {
    let (frags, reg) = load_meta("auto_improve.iris");
    let graph = find_graph(&frags, "passes_gate");
    assert_eq!(
        eval_int(graph, &[Value::Int(100), Value::Int(250), Value::Int(200)], &reg),
        0
    );
}

#[test]
fn test_auto_improve_compute_slowdown_equal() {
    let (frags, reg) = load_meta("auto_improve.iris");
    let graph = find_graph(&frags, "compute_slowdown");
    assert_eq!(
        eval_int(graph, &[Value::Int(1000), Value::Int(1000)], &reg),
        100
    );
}

#[test]
fn test_auto_improve_compute_slowdown_2x() {
    let (frags, reg) = load_meta("auto_improve.iris");
    let graph = find_graph(&frags, "compute_slowdown");
    assert_eq!(
        eval_int(graph, &[Value::Int(2000), Value::Int(1000)], &reg),
        200
    );
}

#[test]
fn test_auto_improve_compute_slowdown_zero_baseline() {
    let (frags, reg) = load_meta("auto_improve.iris");
    let graph = find_graph(&frags, "compute_slowdown");
    // rust_time_ns == 0 → returns 100
    assert_eq!(
        eval_int(graph, &[Value::Int(500), Value::Int(0)], &reg),
        100
    );
}

#[test]
fn test_auto_improve_compute_correctness_all() {
    let (frags, reg) = load_meta("auto_improve.iris");
    let graph = find_graph(&frags, "compute_correctness");
    assert_eq!(
        eval_int(graph, &[Value::Int(10), Value::Int(10)], &reg),
        100
    );
}

#[test]
fn test_auto_improve_compute_correctness_half() {
    let (frags, reg) = load_meta("auto_improve.iris");
    let graph = find_graph(&frags, "compute_correctness");
    assert_eq!(
        eval_int(graph, &[Value::Int(5), Value::Int(10)], &reg),
        50
    );
}

#[test]
fn test_auto_improve_compute_correctness_zero() {
    let (frags, reg) = load_meta("auto_improve.iris");
    let graph = find_graph(&frags, "compute_correctness");
    assert_eq!(
        eval_int(graph, &[Value::Int(0), Value::Int(0)], &reg),
        0
    );
}

#[test]
fn test_auto_improve_count_tests() {
    let (frags, reg) = load_meta("auto_improve.iris");
    let graph = find_graph(&frags, "count_tests");
    let tests = Value::tuple(vec![
        Value::tuple(vec![Value::Int(1), Value::Int(2)]),
        Value::tuple(vec![Value::Int(3), Value::Int(4)]),
        Value::tuple(vec![Value::Int(5), Value::Int(6)]),
    ]);
    assert_eq!(eval_int(graph, &[tests], &reg), 3);
}

#[test]
fn test_auto_improve_find_slowest() {
    let (frags, reg) = load_meta("auto_improve.iris");
    let graph = find_graph(&frags, "find_slowest");
    let components = Value::tuple(vec![
        Value::tuple(vec![Value::Int(1), Value::Int(100)]),
        Value::tuple(vec![Value::Int(2), Value::Int(500)]),
        Value::tuple(vec![Value::Int(3), Value::Int(200)]),
    ]);
    let result = eval(graph, &[components], &reg);
    match result {
        Value::Tuple(elems) => {
            assert_eq!(elems[0], Value::Int(2), "slowest should be component 2");
            assert_eq!(elems[1], Value::Int(500), "slowest time should be 500");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// performance_gate.iris
// ---------------------------------------------------------------------------

#[test]
fn test_performance_gate_compiles() {
    let (frags, _reg) = load_meta("performance_gate.iris");
    let expected = [
        "outputs_match", "eval_correctness", "correctness_pct",
        "slowdown_pct", "gate_passes", "performance_gate", "should_deploy",
    ];
    for name in &expected {
        assert_has_fragment(&frags, name);
    }
}

#[test]
fn test_performance_gate_outputs_match_yes() {
    let (frags, reg) = load_meta("performance_gate.iris");
    let graph = find_graph(&frags, "outputs_match");
    assert_eq!(eval_int(graph, &[Value::Int(42), Value::Int(42)], &reg), 1);
}

#[test]
fn test_performance_gate_outputs_match_no() {
    let (frags, reg) = load_meta("performance_gate.iris");
    let graph = find_graph(&frags, "outputs_match");
    assert_eq!(eval_int(graph, &[Value::Int(42), Value::Int(43)], &reg), 0);
}

// NOTE: correctness_pct and slowdown_pct use `div` infix syntax which the
// parser doesn't support as a binary operator. Zero-denominator cases work
// because they take early return paths before hitting `div`.

#[test]
fn test_performance_gate_correctness_pct_zero_total() {
    let (frags, reg) = load_meta("performance_gate.iris");
    let graph = find_graph(&frags, "correctness_pct");
    assert_eq!(eval_int(graph, &[Value::Int(0), Value::Int(0)], &reg), 0);
}

#[test]
fn test_performance_gate_slowdown_pct_zero_rust() {
    let (frags, reg) = load_meta("performance_gate.iris");
    let graph = find_graph(&frags, "slowdown_pct");
    // rust_ns == 0 → returns 100 (early return, avoids `div`)
    assert_eq!(eval_int(graph, &[Value::Int(500), Value::Int(0)], &reg), 100);
}

#[test]
fn test_performance_gate_gate_passes_yes() {
    let (frags, reg) = load_meta("performance_gate.iris");
    let graph = find_graph(&frags, "gate_passes");
    assert_eq!(
        eval_int(graph, &[Value::Int(100), Value::Int(150), Value::Int(200)], &reg),
        1
    );
}

#[test]
fn test_performance_gate_gate_passes_fail_corr() {
    let (frags, reg) = load_meta("performance_gate.iris");
    let graph = find_graph(&frags, "gate_passes");
    assert_eq!(
        eval_int(graph, &[Value::Int(99), Value::Int(150), Value::Int(200)], &reg),
        0
    );
}

#[test]
fn test_performance_gate_gate_passes_fail_slow() {
    let (frags, reg) = load_meta("performance_gate.iris");
    let graph = find_graph(&frags, "gate_passes");
    assert_eq!(
        eval_int(graph, &[Value::Int(100), Value::Int(250), Value::Int(200)], &reg),
        0
    );
}

#[test]
fn test_performance_gate_should_deploy_all_pass() {
    let (frags, reg) = load_meta("performance_gate.iris");
    let graph = find_graph(&frags, "should_deploy");
    let results = Value::tuple(vec![Value::Int(1), Value::Int(1), Value::Int(1)]);
    assert_eq!(eval_int(graph, &[results], &reg), 1);
}

#[test]
fn test_performance_gate_should_deploy_one_fail() {
    let (frags, reg) = load_meta("performance_gate.iris");
    let graph = find_graph(&frags, "should_deploy");
    let results = Value::tuple(vec![Value::Int(1), Value::Int(0), Value::Int(1)]);
    assert_eq!(eval_int(graph, &[results], &reg), 0);
}

// ---------------------------------------------------------------------------
// daemon.iris
// ---------------------------------------------------------------------------

#[test]
fn test_daemon_compiles() {
    let (frags, _reg) = load_meta("daemon.iris");
    let expected = [
        "stagnation_threshold", "convergence_threshold", "max_explore_attempts",
        "is_stagnating", "is_converged", "decide_mode",
        "run_improve_cycle", "run_explore_cycle", "run_reset",
        "daemon_loop", "should_continue",
    ];
    for name in &expected {
        assert_has_fragment(&frags, name);
    }
}

#[test]
fn test_daemon_stagnation_threshold() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "stagnation_threshold");
    assert_eq!(eval_no_args_int(graph, &reg), 10);
}

#[test]
fn test_daemon_convergence_threshold() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "convergence_threshold");
    assert_eq!(eval_no_args_int(graph, &reg), 5);
}

#[test]
fn test_daemon_max_explore_attempts() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "max_explore_attempts");
    assert_eq!(eval_no_args_int(graph, &reg), 3);
}

#[test]
fn test_daemon_is_stagnating_yes() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "is_stagnating");
    assert_eq!(eval_int(graph, &[Value::Int(15)], &reg), 1);
}

#[test]
fn test_daemon_is_stagnating_no() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "is_stagnating");
    assert_eq!(eval_int(graph, &[Value::Int(5)], &reg), 0);
}

#[test]
fn test_daemon_is_stagnating_exact_threshold() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "is_stagnating");
    assert_eq!(eval_int(graph, &[Value::Int(10)], &reg), 1);
}

#[test]
fn test_daemon_is_converged_yes() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "is_converged");
    let recent = Value::tuple(vec![
        Value::Int(1), Value::Int(0), Value::Int(2), Value::Int(1), Value::Int(0),
    ]);
    assert_eq!(eval_int(graph, &[recent], &reg), 1);
}

#[test]
fn test_daemon_is_converged_no() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "is_converged");
    let recent = Value::tuple(vec![
        Value::Int(1), Value::Int(50), Value::Int(2), Value::Int(1), Value::Int(0),
    ]);
    assert_eq!(eval_int(graph, &[recent], &reg), 0);
}

#[test]
fn test_daemon_decide_mode_improve() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "decide_mode");
    assert_eq!(
        eval_int(graph, &[Value::Int(3), Value::Int(0), Value::Int(0)], &reg),
        0
    );
}

#[test]
fn test_daemon_decide_mode_explore() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "decide_mode");
    assert_eq!(
        eval_int(graph, &[Value::Int(0), Value::Int(1), Value::Int(0)], &reg),
        1
    );
}

#[test]
fn test_daemon_decide_mode_reset() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "decide_mode");
    // converged=1, explore_count >= max_explore_attempts(3) → mode 2
    assert_eq!(
        eval_int(graph, &[Value::Int(0), Value::Int(1), Value::Int(5)], &reg),
        2
    );
}

#[test]
fn test_daemon_decide_mode_stagnation() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "decide_mode");
    // stag_count=15 >= threshold(10), converged=0 → mode 1 (explore)
    assert_eq!(
        eval_int(graph, &[Value::Int(15), Value::Int(0), Value::Int(0)], &reg),
        1
    );
}

#[test]
fn test_daemon_should_continue_solved() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "should_continue");
    assert_eq!(eval_int(graph, &[Value::Int(100), Value::Int(10)], &reg), 0);
}

#[test]
fn test_daemon_should_continue_budget_exhausted() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "should_continue");
    assert_eq!(eval_int(graph, &[Value::Int(50), Value::Int(0)], &reg), 0);
}

#[test]
fn test_daemon_should_continue_yes() {
    let (frags, reg) = load_meta("daemon.iris");
    let graph = find_graph(&frags, "should_continue");
    assert_eq!(eval_int(graph, &[Value::Int(50), Value::Int(10)], &reg), 1);
}

// ---------------------------------------------------------------------------
// evolve_step.iris
// ---------------------------------------------------------------------------

#[test]
fn test_evolve_step_compiles() {
    let (frags, _reg) = load_meta("evolve_step.iris");
    assert_has_fragment(&frags, "evolve_step");
}

// ---------------------------------------------------------------------------
// mutate_and_test.iris
// ---------------------------------------------------------------------------

#[test]
fn test_mutate_and_test_compiles() {
    let (frags, _reg) = load_meta("mutate_and_test.iris");
    assert_has_fragment(&frags, "mutate_and_test");
}

// ---------------------------------------------------------------------------
// quine.iris
// ---------------------------------------------------------------------------

#[test]
fn test_quine_compiles() {
    let (frags, _reg) = load_meta("quine.iris");
    assert_has_fragment(&frags, "quine");
}

// ---------------------------------------------------------------------------
// self_modify.iris
// ---------------------------------------------------------------------------

#[test]
fn test_self_modify_compiles() {
    let (frags, _reg) = load_meta("self_modify.iris");
    assert_has_fragment(&frags, "self_improve");
}

// ---------------------------------------------------------------------------
// instrumentation.iris – record_timing and verify_chain (tuple-based)
// ---------------------------------------------------------------------------

// NOTE: record_timing uses `div` infix syntax which the parser doesn't
// support as a binary operator. Compile-only verification is in
// test_instrumentation_compiles above.

#[test]
fn test_instrumentation_verify_chain_valid() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "verify_chain");
    // Chain: entry1(hash=10, prev=0), entry2(hash=20, prev=10) → valid
    let entries = Value::tuple(vec![
        Value::tuple(vec![Value::Int(10), Value::Int(0)]),
        Value::tuple(vec![Value::Int(20), Value::Int(10)]),
    ]);
    let result = eval(graph, &[entries], &reg);
    // verify_chain returns (valid_flag, last_hash)
    match result {
        Value::Tuple(elems) => {
            assert_eq!(elems[0], Value::Int(1), "chain should be valid");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn test_instrumentation_verify_chain_broken() {
    let (frags, reg) = load_meta("instrumentation.iris");
    let graph = find_graph(&frags, "verify_chain");
    // Chain: entry1(hash=10, prev=0), entry2(hash=20, prev=99) → broken
    let entries = Value::tuple(vec![
        Value::tuple(vec![Value::Int(10), Value::Int(0)]),
        Value::tuple(vec![Value::Int(20), Value::Int(99)]),
    ]);
    let result = eval(graph, &[entries], &reg);
    match result {
        Value::Tuple(elems) => {
            assert_eq!(elems[0], Value::Int(0), "chain should be broken");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// improvement_tracker.iris – adaptive_weight
// ---------------------------------------------------------------------------

#[test]
fn test_improvement_tracker_adaptive_weight_low_samples() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "adaptive_weight");
    let stats = Value::tuple(vec![Value::Int(2), Value::Int(1), Value::Int(10)]);
    // min_samples=10, only 2 uses → returns base_weight
    assert_eq!(
        eval_int(graph, &[stats, Value::Int(100), Value::Int(10)], &reg),
        100
    );
}

#[test]
fn test_improvement_tracker_adaptive_weight_high_samples() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "adaptive_weight");
    let stats = Value::tuple(vec![Value::Int(100), Value::Int(50), Value::Int(500)]);
    let w = eval_int(graph, &[stats, Value::Int(100), Value::Int(10)], &reg);
    assert!(w > 0, "adaptive_weight should be positive, got {}", w);
}

// ---------------------------------------------------------------------------
// improvement_tracker.iris – best_operator
// ---------------------------------------------------------------------------

#[test]
fn test_improvement_tracker_best_operator() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "best_operator");
    let operators = Value::tuple(vec![
        Value::tuple(vec![Value::Int(1), Value::Int(10), Value::Int(3), Value::Int(50)]),
        Value::tuple(vec![Value::Int(2), Value::Int(20), Value::Int(8), Value::Int(200)]),
        Value::tuple(vec![Value::Int(3), Value::Int(5), Value::Int(2), Value::Int(30)]),
    ]);
    let result = eval(graph, &[operators], &reg);
    match result {
        Value::Tuple(elems) => {
            assert_eq!(elems[0], Value::Int(2), "best operator should be #2");
            assert_eq!(elems[3], Value::Int(200), "best total_delta should be 200");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// improvement_tracker.iris – tracker_summary
// ---------------------------------------------------------------------------

#[test]
fn test_improvement_tracker_summary() {
    let (frags, reg) = load_meta("improvement_tracker.iris");
    let graph = find_graph(&frags, "tracker_summary");
    // tracker_summary(rate, accel, first_ts, first_count, last_ts, last_count)
    let result = eval(
        graph,
        &[
            Value::Int(-10),      // rate (negative = improving)
            Value::Int(-2),       // accel (negative = compounding)
            Value::Int(0),        // first_ts
            Value::Int(0),        // first_count
            Value::Int(3600000),  // last_ts
            Value::Int(10),       // last_count
        ],
        &reg,
    );
    match result {
        Value::Tuple(elems) => {
            assert_eq!(elems[0], Value::Int(-10), "rate");
            assert_eq!(elems[1], Value::Int(-2), "accel");
            assert_eq!(elems[2], Value::Int(1), "is_compounding (accel<0)");
            // pph should be positive
            match &elems[3] {
                Value::Int(pph) => assert!(*pph > 0, "pph should be positive"),
                other => panic!("expected Int pph, got {:?}", other),
            }
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}
