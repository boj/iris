//! Tests for src/iris-programs/foundry/ — problem solving and library management.
//!
//! Tests the pure, testable functions across the foundry .iris files:
//! - latency_tiers.iris: tier classification, budgets, and thresholds
//! - fragment_library.iris: type hashing and library sizing
//! - bootstrap_problems.iris: test data definitions
//! - solve.iris: tier budgets and serialization helpers
//! - foundry.iris: tier constants, population/generation maps, stubs

use iris_exec::effect_runtime::RuntimeEffectHandler;
use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// Helpers (matching test_exec_iris_programs.rs pattern)
// ---------------------------------------------------------------------------

struct CompiledFile {
    fragments: Vec<(String, SemanticGraph)>,
    registry: FragmentRegistry,
}

fn compile_iris(path: &str) -> CompiledFile {
    let src =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {}: {}", path, e));
    let result = iris_bootstrap::syntax::compile(&src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!(
                "{}: {}",
                path,
                iris_bootstrap::syntax::format_error(&src, err)
            );
        }
        panic!(
            "{} failed to compile with {} errors",
            path,
            result.errors.len()
        );
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
    CompiledFile {
        fragments,
        registry,
    }
}

fn run(f: &CompiledFile, name: &str, inputs: &[Value]) -> Value {
    let graph = &f
        .fragments
        .iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("fragment '{}' not found", name))
        .1;
    let handler = RuntimeEffectHandler::new();
    let (outputs, _) = interpreter::interpret_with_effects(
        graph,
        inputs,
        None,
        Some(&f.registry),
        10_000_000,
        Some(&handler),
    )
    .expect("evaluation should succeed");
    assert!(!outputs.is_empty(), "should produce output");
    outputs.into_iter().next().unwrap()
}

fn run_int(f: &CompiledFile, name: &str, inputs: &[Value]) -> i64 {
    match run(f, name, inputs) {
        Value::Int(v) => v,
        o => panic!("expected Int, got {:?}", o),
    }
}

fn run_tuple4_ints(f: &CompiledFile, name: &str, inputs: &[Value]) -> (i64, i64, i64, i64) {
    match run(f, name, inputs) {
        Value::Tuple(t) => {
            assert!(
                t.len() >= 4,
                "expected tuple of 4+ elements, got {}",
                t.len()
            );
            let a = match &t[0] {
                Value::Int(v) => *v,
                o => panic!("t[0]: {:?}", o),
            };
            let b = match &t[1] {
                Value::Int(v) => *v,
                o => panic!("t[1]: {:?}", o),
            };
            let c = match &t[2] {
                Value::Int(v) => *v,
                o => panic!("t[2]: {:?}", o),
            };
            let d = match &t[3] {
                Value::Int(v) => *v,
                o => panic!("t[3]: {:?}", o),
            };
            (a, b, c, d)
        }
        o => panic!("expected Tuple, got {:?}", o),
    }
}

fn run_tuple2_ints(f: &CompiledFile, name: &str, inputs: &[Value]) -> (i64, i64) {
    match run(f, name, inputs) {
        Value::Tuple(t) => {
            assert!(
                t.len() >= 2,
                "expected tuple of 2+ elements, got {}",
                t.len()
            );
            let a = match &t[0] {
                Value::Int(v) => *v,
                o => panic!("t[0]: {:?}", o),
            };
            let b = match &t[1] {
                Value::Int(v) => *v,
                o => panic!("t[1]: {:?}", o),
            };
            (a, b)
        }
        o => panic!("expected Tuple, got {:?}", o),
    }
}

fn run_tuple3_ints(f: &CompiledFile, name: &str, inputs: &[Value]) -> (i64, i64, i64) {
    match run(f, name, inputs) {
        Value::Tuple(t) => {
            assert!(
                t.len() >= 3,
                "expected tuple of 3+ elements, got {}",
                t.len()
            );
            let a = match &t[0] {
                Value::Int(v) => *v,
                o => panic!("t[0]: {:?}", o),
            };
            let b = match &t[1] {
                Value::Int(v) => *v,
                o => panic!("t[1]: {:?}", o),
            };
            let c = match &t[2] {
                Value::Int(v) => *v,
                o => panic!("t[2]: {:?}", o),
            };
            (a, b, c)
        }
        o => panic!("expected Tuple, got {:?}", o),
    }
}

// =========================================================================
// latency_tiers.iris — ALL 7 functions
// =========================================================================

#[test]
fn test_tier_budget_instant() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    let config = Value::tuple(vec![Value::Int(100), Value::Int(200), Value::Int(30)]);
    assert_eq!(
        run_tuple4_ints(&f, "tier_budget", &[Value::Int(0), config]),
        (0, 0, 100, 1)
    );
}

#[test]
fn test_tier_budget_fast() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    let config = Value::tuple(vec![Value::Int(100), Value::Int(200), Value::Int(30)]);
    assert_eq!(
        run_tuple4_ints(&f, "tier_budget", &[Value::Int(1), config]),
        (16, 50, 10000, 1)
    );
}

#[test]
fn test_tier_budget_standard() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    let config = Value::tuple(vec![Value::Int(100), Value::Int(200), Value::Int(30)]);
    assert_eq!(
        run_tuple4_ints(&f, "tier_budget", &[Value::Int(2), config]),
        (100, 200, 30000, 1)
    );
}

#[test]
fn test_tier_budget_deep() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    let config = Value::tuple(vec![Value::Int(100), Value::Int(200), Value::Int(30)]);
    assert_eq!(
        run_tuple4_ints(&f, "tier_budget", &[Value::Int(3), config]),
        (200, 1000, 300000, 4)
    );
}

#[test]
fn test_classify_tier_has_library_small() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(
        run_int(&f, "classify_tier", &[Value::Int(3), Value::Int(5), Value::Int(1)]),
        0
    );
}

#[test]
fn test_classify_tier_no_library_small() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(
        run_int(&f, "classify_tier", &[Value::Int(3), Value::Int(5), Value::Int(0)]),
        1
    );
}

#[test]
fn test_classify_tier_medium() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(
        run_int(&f, "classify_tier", &[Value::Int(10), Value::Int(50), Value::Int(0)]),
        2
    );
}

#[test]
fn test_classify_tier_large() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(
        run_int(&f, "classify_tier", &[Value::Int(25), Value::Int(200), Value::Int(0)]),
        3
    );
}

#[test]
fn test_tier_allows_evolution_instant() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(run_int(&f, "tier_allows_evolution", &[Value::Int(0)]), 0);
}

#[test]
fn test_tier_allows_evolution_fast() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(run_int(&f, "tier_allows_evolution", &[Value::Int(1)]), 1);
}

#[test]
fn test_tier_is_multi_deme_deep() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(run_int(&f, "tier_is_multi_deme", &[Value::Int(3)]), 1);
}

#[test]
fn test_tier_is_multi_deme_standard() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(run_int(&f, "tier_is_multi_deme", &[Value::Int(2)]), 0);
}

#[test]
fn test_tier_novelty_weight_deep() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(run_int(&f, "tier_novelty_weight", &[Value::Int(3)]), 2);
}

#[test]
fn test_tier_novelty_weight_fast() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(run_int(&f, "tier_novelty_weight", &[Value::Int(1)]), 0);
}

#[test]
fn test_meets_tier_threshold_instant_pass() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(
        run_int(&f, "meets_tier_threshold", &[Value::Int(0), Value::Int(99)]),
        1
    );
}

#[test]
fn test_meets_tier_threshold_instant_fail() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(
        run_int(&f, "meets_tier_threshold", &[Value::Int(0), Value::Int(90)]),
        0
    );
}

#[test]
fn test_meets_tier_threshold_standard_pass() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(
        run_int(&f, "meets_tier_threshold", &[Value::Int(2), Value::Int(90)]),
        1
    );
}

#[test]
fn test_meets_tier_threshold_standard_fail() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(
        run_int(&f, "meets_tier_threshold", &[Value::Int(2), Value::Int(80)]),
        0
    );
}

#[test]
fn test_meets_tier_threshold_deep_pass() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(
        run_int(&f, "meets_tier_threshold", &[Value::Int(3), Value::Int(50)]),
        1
    );
}

#[test]
fn test_meets_tier_threshold_deep_fail() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(
        run_int(&f, "meets_tier_threshold", &[Value::Int(3), Value::Int(40)]),
        0
    );
}

#[test]
fn test_effective_tier_downgrade_on_library_hit() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(
        run_int(&f, "effective_tier", &[Value::Int(2), Value::Int(1)]),
        0
    );
}

#[test]
fn test_effective_tier_no_library_hit() {
    let f = compile_iris("src/iris-programs/foundry/latency_tiers.iris");
    assert_eq!(
        run_int(&f, "effective_tier", &[Value::Int(2), Value::Int(0)]),
        2
    );
}

// =========================================================================
// fragment_library.iris — type hashing and library size
// =========================================================================

#[test]
fn test_type_sig_hash() {
    let f = compile_iris("src/iris-programs/foundry/fragment_library.iris");
    // bitxor (2 * 65537) 5 = bitxor 131074 5 = 131079
    assert_eq!(
        run_int(&f, "type_sig_hash", &[Value::Int(2), Value::Int(5)]),
        131079
    );
}

// =========================================================================
// bootstrap_problems.iris — test data definitions
// =========================================================================

#[test]
fn test_add_tests_is_tuple_of_4() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    match run(&f, "add_tests", &[]) {
        Value::Tuple(t) => assert_eq!(t.len(), 4, "add_tests should have 4 test cases"),
        o => panic!("expected Tuple, got {:?}", o),
    }
}

#[test]
fn test_sub_tests_is_tuple_of_4() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    match run(&f, "sub_tests", &[]) {
        Value::Tuple(t) => assert_eq!(t.len(), 4, "sub_tests should have 4 test cases"),
        o => panic!("expected Tuple, got {:?}", o),
    }
}

#[test]
fn test_mul_tests_is_tuple_of_4() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    match run(&f, "mul_tests", &[]) {
        Value::Tuple(t) => assert_eq!(t.len(), 4, "mul_tests should have 4 test cases"),
        o => panic!("expected Tuple, got {:?}", o),
    }
}

#[test]
fn test_abs_tests_is_tuple_of_4() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    match run(&f, "abs_tests", &[]) {
        Value::Tuple(t) => assert_eq!(t.len(), 4, "abs_tests should have 4 test cases"),
        o => panic!("expected Tuple, got {:?}", o),
    }
}

#[test]
fn test_max_tests_is_tuple_of_4() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    match run(&f, "max_tests", &[]) {
        Value::Tuple(t) => assert_eq!(t.len(), 4, "max_tests should have 4 test cases"),
        o => panic!("expected Tuple, got {:?}", o),
    }
}

#[test]
fn test_min_tests_is_tuple_of_4() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    match run(&f, "min_tests", &[]) {
        Value::Tuple(t) => assert_eq!(t.len(), 4, "min_tests should have 4 test cases"),
        o => panic!("expected Tuple, got {:?}", o),
    }
}

#[test]
fn test_double_tests_is_tuple_of_4() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    match run(&f, "double_tests", &[]) {
        Value::Tuple(t) => assert_eq!(t.len(), 4, "double_tests should have 4 test cases"),
        o => panic!("expected Tuple, got {:?}", o),
    }
}

#[test]
fn test_factorial_tests_is_tuple_of_4() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    match run(&f, "factorial_tests", &[]) {
        Value::Tuple(t) => assert_eq!(t.len(), 4, "factorial_tests should have 4 test cases"),
        o => panic!("expected Tuple, got {:?}", o),
    }
}

#[test]
fn test_fibonacci_tests_is_tuple_of_4() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    match run(&f, "fibonacci_tests", &[]) {
        Value::Tuple(t) => assert_eq!(t.len(), 4, "fibonacci_tests should have 4 test cases"),
        o => panic!("expected Tuple, got {:?}", o),
    }
}

#[test]
fn test_gcd_tests_is_tuple_of_4() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    match run(&f, "gcd_tests", &[]) {
        Value::Tuple(t) => assert_eq!(t.len(), 4, "gcd_tests should have 4 test cases"),
        o => panic!("expected Tuple, got {:?}", o),
    }
}

#[test]
fn test_power_tests_is_tuple_of_4() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    match run(&f, "power_tests", &[]) {
        Value::Tuple(t) => assert_eq!(t.len(), 4, "power_tests should have 4 test cases"),
        o => panic!("expected Tuple, got {:?}", o),
    }
}

// =========================================================================
// solve.iris — tier budgets (pure parts)
// =========================================================================

#[test]
fn test_solve_tier_budget_instant() {
    let f = compile_iris("src/iris-programs/foundry/solve.iris");
    assert_eq!(
        run_tuple2_ints(&f, "tier_budget", &[Value::Int(0), Value::Int(32), Value::Int(100)]),
        (0, 0)
    );
}

#[test]
fn test_solve_tier_budget_fast() {
    let f = compile_iris("src/iris-programs/foundry/solve.iris");
    assert_eq!(
        run_tuple2_ints(&f, "tier_budget", &[Value::Int(1), Value::Int(32), Value::Int(100)]),
        (16, 50)
    );
}

#[test]
fn test_solve_tier_budget_standard() {
    let f = compile_iris("src/iris-programs/foundry/solve.iris");
    assert_eq!(
        run_tuple2_ints(&f, "tier_budget", &[Value::Int(2), Value::Int(32), Value::Int(100)]),
        (32, 100)
    );
}

#[test]
fn test_solve_tier_budget_deep() {
    let f = compile_iris("src/iris-programs/foundry/solve.iris");
    assert_eq!(
        run_tuple2_ints(&f, "tier_budget", &[Value::Int(3), Value::Int(32), Value::Int(100)]),
        (64, 500)
    );
}

// =========================================================================
// foundry.iris — pure functions only (skip foundry_solve: uses clock_ns)
// =========================================================================

#[test]
fn test_tier_instant_constant() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(run_int(&f, "tier_instant", &[]), 0);
}

#[test]
fn test_tier_fast_constant() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(run_int(&f, "tier_fast", &[]), 1);
}

#[test]
fn test_tier_standard_constant() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(run_int(&f, "tier_standard", &[]), 2);
}

#[test]
fn test_tier_deep_constant() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(run_int(&f, "tier_deep", &[]), 3);
}

#[test]
fn test_tier_max_generations_instant() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(run_int(&f, "tier_max_generations", &[Value::Int(0)]), 0);
}

#[test]
fn test_tier_max_generations_fast() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(run_int(&f, "tier_max_generations", &[Value::Int(1)]), 100);
}

#[test]
fn test_tier_max_generations_standard() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(run_int(&f, "tier_max_generations", &[Value::Int(2)]), 1000);
}

#[test]
fn test_tier_max_generations_deep() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(
        run_int(&f, "tier_max_generations", &[Value::Int(3)]),
        10000
    );
}

#[test]
fn test_tier_population_size_instant() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(run_int(&f, "tier_population_size", &[Value::Int(0)]), 0);
}

#[test]
fn test_tier_population_size_fast() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(run_int(&f, "tier_population_size", &[Value::Int(1)]), 32);
}

#[test]
fn test_tier_population_size_standard() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(run_int(&f, "tier_population_size", &[Value::Int(2)]), 128);
}

#[test]
fn test_tier_population_size_deep() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(run_int(&f, "tier_population_size", &[Value::Int(3)]), 512);
}

#[test]
fn test_mutate_candidate() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(run_int(&f, "mutate_candidate", &[Value::Int(5)]), 6);
}

#[test]
fn test_evaluate_candidate_zero_spec() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    assert_eq!(
        run_int(&f, "evaluate_candidate", &[Value::Int(0), Value::Int(0)]),
        0
    );
}

#[test]
fn test_evaluate_candidate_nonzero_spec() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    // Build a dummy candidate (just pass Int 0 — graph_eval will fail gracefully)
    // Use empty spec to avoid needing a real program
    let spec = Value::tuple(vec![]);
    assert_eq!(
        run_int(&f, "evaluate_candidate", &[Value::Int(0), spec]),
        0, // 0 test cases → 0 fitness
    );
}

#[test]
fn test_library_search_with_tuple() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    // Library is now Tuple of (program, fitness) pairs
    let library = Value::tuple(vec![
        Value::tuple(vec![Value::Int(1), Value::Int(300)]),
        Value::tuple(vec![Value::Int(2), Value::Int(700)]),
        Value::tuple(vec![Value::Int(3), Value::Int(500)]),
    ]);
    let result = run_tuple3_ints(&f, "library_search", &[library, Value::Int(0)]);
    assert_eq!(result.0, 3, "count should be 3 (all entries visited)");
    assert_eq!(result.2, 700, "best fitness should be 700");
}

#[test]
fn test_library_add() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");
    // library_add returns (fragment, (fitness, library))
    match run(&f, "library_add", &[Value::Int(10), Value::Int(42), Value::Int(99)]) {
        Value::Tuple(t) => {
            assert_eq!(t.len(), 2);
            assert_eq!(t[0], Value::Int(42), "first element = fragment");
            match &t[1] {
                Value::Tuple(inner) => {
                    assert_eq!(inner.len(), 2);
                    assert_eq!(inner[0], Value::Int(99), "fitness");
                    assert_eq!(inner[1], Value::Int(10), "original library");
                }
                o => panic!("expected inner Tuple, got {:?}", o),
            }
        }
        o => panic!("expected Tuple, got {:?}", o),
    }
}

// ===========================================================================
// Integration tests: real compiled programs through foundry pipelines
// ===========================================================================

/// Compile a single IRIS function and return its SemanticGraph.
fn compile_single_function(src: &str) -> SemanticGraph {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!("compile_single_function failed: {} errors", result.errors.len());
    }
    assert!(!result.fragments.is_empty(), "no fragments compiled");
    result.fragments.into_iter().next().unwrap().1.graph
}

// --- bootstrap_problems.iris: eval_on_problem with real compiled programs ---

#[test]
fn test_integration_eval_on_problem_add_all_pass() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    let add_graph = compile_single_function("let add x y : Int -> Int -> Int = x + y");
    let candidate = Value::Program(Box::new(add_graph));
    let test_cases = Value::tuple(vec![
        Value::tuple(vec![Value::tuple(vec![Value::Int(1), Value::Int(2)]), Value::Int(3)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(0), Value::Int(0)]), Value::Int(0)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(-1), Value::Int(1)]), Value::Int(0)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(100), Value::Int(200)]), Value::Int(300)]),
    ]);
    let v = run_int(&f, "eval_on_problem", &[candidate, test_cases]);
    assert_eq!(v, 4, "all 4 add test cases should pass");
}

#[test]
fn test_integration_eval_on_problem_mul_partial_pass() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    let mul_graph = compile_single_function("let mul x y : Int -> Int -> Int = x * y");
    let candidate = Value::Program(Box::new(mul_graph));
    // mul(1,2)=2≠3, mul(0,0)=0==0✓, mul(-1,1)=-1≠0, mul(100,200)=20000≠300
    let test_cases = Value::tuple(vec![
        Value::tuple(vec![Value::tuple(vec![Value::Int(1), Value::Int(2)]), Value::Int(3)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(0), Value::Int(0)]), Value::Int(0)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(-1), Value::Int(1)]), Value::Int(0)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(100), Value::Int(200)]), Value::Int(300)]),
    ]);
    let v = run_int(&f, "eval_on_problem", &[candidate, test_cases]);
    assert_eq!(v, 1, "only (0,0)→0 should pass for mul against add tests");
}

#[test]
fn test_integration_eval_on_problem_double_dedicated() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    let dbl_graph = compile_single_function("let double x : Int -> Int = x * 2");
    let candidate = Value::Program(Box::new(dbl_graph));
    let test_cases = Value::tuple(vec![
        Value::tuple(vec![Value::tuple(vec![Value::Int(3)]), Value::Int(6)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(0)]), Value::Int(0)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(-5)]), Value::Int(-10)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(50)]), Value::Int(100)]),
    ]);
    let v = run_int(&f, "eval_on_problem", &[candidate, test_cases]);
    assert_eq!(v, 4, "all 4 double test cases should pass");
}

#[test]
fn test_integration_eval_on_problem_zero_pass() {
    let f = compile_iris("src/iris-programs/foundry/bootstrap_problems.iris");
    let add_graph = compile_single_function("let add x y : Int -> Int -> Int = x + y");
    let candidate = Value::Program(Box::new(add_graph));
    let test_cases = Value::tuple(vec![
        Value::tuple(vec![Value::tuple(vec![Value::Int(1), Value::Int(2)]), Value::Int(999)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(3), Value::Int(4)]), Value::Int(999)]),
    ]);
    let v = run_int(&f, "eval_on_problem", &[candidate, test_cases]);
    assert_eq!(v, 0, "no test cases should pass with wrong expected values");
}

// --- fragment_library.iris: type hashing and library size with real graphs ---

#[test]
fn test_integration_type_sig_hash_unary_fn() {
    let f = compile_iris("src/iris-programs/foundry/fragment_library.iris");
    // type_sig_hash(1, 0) = bitxor(65537, 0) = 65537
    assert_eq!(
        run_int(&f, "type_sig_hash", &[Value::Int(1), Value::Int(0)]),
        65537
    );
}

#[test]
fn test_integration_type_sig_hash_nullary() {
    let f = compile_iris("src/iris-programs/foundry/fragment_library.iris");
    // type_sig_hash(0, 5) = bitxor(0, 5) = 5
    assert_eq!(
        run_int(&f, "type_sig_hash", &[Value::Int(0), Value::Int(5)]),
        5
    );
}

#[test]
fn test_integration_library_size_real_graph() {
    let f = compile_iris("src/iris-programs/foundry/fragment_library.iris");
    let graph = compile_single_function("let id x : Int -> Int = x");
    let lib = Value::Program(Box::new(graph));
    let size = run_int(&f, "library_size", &[lib]);
    assert!(size > 0, "library_size should be positive for a real compiled graph");
}

// ===========================================================================
// Integration: fragment_library register → search round-trip
// ===========================================================================

#[test]
fn test_integration_library_register_search_roundtrip() {
    let f = compile_iris("src/iris-programs/foundry/fragment_library.iris");
    // Create a fresh library graph
    let lib_src = "let lib_seed x = x";
    let lib_graph = compile_single_function(lib_src);
    let library = Value::Program(Box::new(lib_graph));

    // Use a small hash that fits in u8 (graph_set_prim_op truncates to u8)
    let type_hash: i64 = 42;

    // Register a fragment with that hash
    let updated_lib = run(&f, "register", &[library, Value::Int(0), Value::Int(type_hash)]);
    assert!(matches!(updated_lib, Value::Program(_)), "register should return a Program");

    // Search should find it
    let result = run(&f, "search_by_type", &[updated_lib.clone(), Value::Int(type_hash)]);
    match result {
        Value::Tuple(t) => {
            assert!(t.len() >= 3, "search_by_type returns (count, node, score)");
            let count = match &t[0] { Value::Int(n) => *n, _ => panic!("expected Int count") };
            assert!(count >= 1, "should find at least 1 match, got {}", count);
        }
        _ => panic!("expected Tuple from search_by_type, got {:?}", result),
    }
}

#[test]
fn test_integration_library_register_search_miss() {
    let f = compile_iris("src/iris-programs/foundry/fragment_library.iris");
    let lib_graph = compile_single_function("let seed x = x");
    let library = Value::Program(Box::new(lib_graph));

    // Register with hash 42
    let updated_lib = run(&f, "register", &[library, Value::Int(0), Value::Int(42)]);

    // Search for hash 999 — should NOT find
    let result = run(&f, "search_by_type", &[updated_lib, Value::Int(999)]);
    match result {
        Value::Tuple(t) => {
            let count = match &t[0] { Value::Int(n) => *n, _ => panic!("expected Int") };
            assert_eq!(count, 0, "should find 0 matches for wrong hash");
        }
        _ => panic!("expected Tuple"),
    }
}

#[test]
fn test_integration_library_record_hit() {
    let f = compile_iris("src/iris-programs/foundry/fragment_library.iris");
    let lib_graph = compile_single_function("let seed x = x");
    let library = Value::Program(Box::new(lib_graph));

    // Register with hash 100
    let lib = run(&f, "register", &[library, Value::Int(0), Value::Int(100)]);

    // Find the node via search
    let search = run(&f, "search_by_type", &[lib.clone(), Value::Int(100)]);
    let node_id = match &search {
        Value::Tuple(t) => t[1].clone(),
        _ => panic!("expected Tuple"),
    };

    // Record a hit — increments the opcode (used as solve count proxy)
    let updated = run(&f, "record_hit", &[lib, node_id]);
    assert!(matches!(updated, Value::Program(_)), "record_hit returns Program");
}

// ===========================================================================
// Integration: foundry evaluate_candidate with real programs
// ===========================================================================

#[test]
fn test_integration_foundry_evaluate_candidate_real_add() {
    // Compile foundry.iris which now has a proper evaluate_candidate
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");

    // Compile a real add program
    let add_graph = compile_single_function("let add x y = x + y");
    let candidate = Value::Program(Box::new(add_graph));

    // Build test cases as Tuple of (inputs, expected) pairs
    let test_cases = Value::tuple(vec![
        Value::tuple(vec![Value::tuple(vec![Value::Int(1), Value::Int(2)]), Value::Int(3)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(0), Value::Int(0)]), Value::Int(0)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(10), Value::Int(20)]), Value::Int(30)]),
    ]);

    let fitness = run_int(&f, "evaluate_candidate", &[candidate, test_cases]);
    // 3/3 passed → (3 * 1000) / 3 = 1000
    assert_eq!(fitness, 1000, "perfect add should score 1000");
}

#[test]
fn test_integration_foundry_evaluate_candidate_partial() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");

    // Compile a double function — won't match add test cases
    let dbl_graph = compile_single_function("let dbl x y = x + x");
    let candidate = Value::Program(Box::new(dbl_graph));

    // Test cases for add
    let test_cases = Value::tuple(vec![
        Value::tuple(vec![Value::tuple(vec![Value::Int(1), Value::Int(2)]), Value::Int(3)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(0), Value::Int(0)]), Value::Int(0)]),
        Value::tuple(vec![Value::tuple(vec![Value::Int(5), Value::Int(5)]), Value::Int(10)]),
    ]);

    let fitness = run_int(&f, "evaluate_candidate", &[candidate, test_cases]);
    // dbl(1,2)=2≠3, dbl(0,0)=0=0✓, dbl(5,5)=10=10✓ → 2/3 = 666
    assert_eq!(fitness, 666, "partial match should score 666");
}

// ===========================================================================
// Integration: foundry library_search with real programs
// ===========================================================================

#[test]
fn test_integration_foundry_library_search_finds_best() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");

    let add_graph = compile_single_function("let add x y = x + y");
    let mul_graph = compile_single_function("let mul x y = x * y");

    // Library: tuple of (program, fitness) pairs
    let library = Value::tuple(vec![
        Value::tuple(vec![Value::Program(Box::new(add_graph)), Value::Int(500)]),
        Value::tuple(vec![Value::Program(Box::new(mul_graph)), Value::Int(900)]),
    ]);

    let result = run(&f, "library_search", &[library, Value::Int(0)]);
    match result {
        Value::Tuple(t) => {
            let count = match &t[0] { Value::Int(n) => *n, _ => panic!("expected Int") };
            let best_fitness = match &t[2] { Value::Int(n) => *n, _ => panic!("expected Int") };
            assert_eq!(count, 2, "should see 2 entries");
            assert_eq!(best_fitness, 900, "best fitness should be 900");
        }
        _ => panic!("expected Tuple from library_search"),
    }
}

// ===========================================================================
// Integration: foundry_solve end-to-end (Instant tier, library hit)
// ===========================================================================

#[test]
fn test_integration_foundry_solve_instant_library_hit() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");

    let add_graph = compile_single_function("let add x y = x + y");

    // Library with a high-fitness entry
    let library = Value::tuple(vec![
        Value::tuple(vec![Value::Program(Box::new(add_graph)), Value::Int(950)]),
    ]);

    // Spec doesn't matter for library search (library_search just finds best fitness)
    let spec = Value::tuple(vec![]);
    let tier = Value::Int(0); // Instant

    let result = run(&f, "foundry_solve", &[library, spec, tier]);
    match result {
        Value::Tuple(t) => {
            assert!(t.len() >= 4, "foundry_solve returns (status, prog, fitness, elapsed)");
            let status = match &t[0] { Value::Int(n) => *n, _ => panic!("expected Int") };
            let fitness = match &t[2] { Value::Int(n) => *n, _ => panic!("expected Int") };
            assert_eq!(status, 0, "status 0 = found in library");
            assert_eq!(fitness, 950, "should return library fitness");
        }
        _ => panic!("expected Tuple from foundry_solve"),
    }
}

#[test]
fn test_integration_foundry_solve_instant_no_hit() {
    let f = compile_iris("src/iris-programs/foundry/foundry.iris");

    // Empty library
    let library = Value::tuple(vec![]);
    let spec = Value::tuple(vec![]);
    let tier = Value::Int(0); // Instant

    let result = run(&f, "foundry_solve", &[library, spec, tier]);
    match result {
        Value::Tuple(t) => {
            let status = match &t[0] { Value::Int(n) => *n, _ => panic!("expected Int") };
            assert_eq!(status, -1, "status -1 = failed (no library hit, Instant tier)");
        }
        _ => panic!("expected Tuple from foundry_solve"),
    }
}
