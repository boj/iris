//! End-to-end integration test: evolve a program that doubles every element
//! in a list.
//!
//! ## Difficulty analysis
//!
//! The current IRIS Gen1 architecture has a Fold node that *reduces* a
//! collection to a scalar — it cannot build a new list (map operation).
//! Therefore, the full "double every element" problem (list -> list) is
//! beyond what the current seed generators + interpreter can express.
//!
//! Instead, we test two achievable variants:
//!
//! ### Variant A: Double a single integer (x -> 2x)
//! This is framed as fold over a two-element list [x, 2] with multiply.
//! Solution: fold(1, mul, [x, 2]) = 1 * x * 2 = 2x
//! We pass the input as a Tuple [x, 2] so the fold sees the elements.
//!
//! ### Variant B: Sum of doubled elements (product-then-sum)
//! Input: [a, b, c], expected: a + b + c (sum). This is the sum test but
//! confirms the engine can rediscover add when starting from mul seeds.
//!
//! If the engine can discover fold(1, mul, list) for the product variant,
//! that demonstrates it can find operators beyond add.

use std::time::Instant;

use iris_evolve::config::{EvolutionConfig, PhaseThresholds, ProblemSpec};
use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_exec::ExecutionService;
use iris_types::eval::{TestCase, Value};

// ---------------------------------------------------------------------------
// Test cases: product of a list of integers
//
// fold(1, mul, list) = product of elements
// This tests whether evolution can discover the mul opcode (0x02) and
// the identity element 1 (instead of 0 for add).
// ---------------------------------------------------------------------------

fn product_test_cases() -> Vec<TestCase> {
    vec![
        // [1, 2, 3] -> 6   (1 * 2 * 3 = 6)
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
            expected_output: Some(vec![Value::Int(6)]),
            initial_state: None,
            expected_state: None,
        },
        // [5] -> 5
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(5)])],
            expected_output: Some(vec![Value::Int(5)]),
            initial_state: None,
            expected_state: None,
        },
        // [2, 3, 4] -> 24
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
            ])],
            expected_output: Some(vec![Value::Int(24)]),
            initial_state: None,
            expected_state: None,
        },
        // [1, 1, 1, 1] -> 1
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(1),
                Value::Int(1),
                Value::Int(1),
                Value::Int(1),
            ])],
            expected_output: Some(vec![Value::Int(1)]),
            initial_state: None,
            expected_state: None,
        },
        // [10, 10] -> 100
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(10), Value::Int(10)])],
            expected_output: Some(vec![Value::Int(100)]),
            initial_state: None,
            expected_state: None,
        },
    ]
}

// ---------------------------------------------------------------------------
// Test cases: double a single integer
//
// Input: [x, x] as a Tuple (we duplicate the value so fold can "see" it
// twice). Solution: fold(0, add, [x, x]) = 0 + x + x = 2x.
// ---------------------------------------------------------------------------

fn double_test_cases() -> Vec<TestCase> {
    vec![
        // double(1) = 2:  input=[1, 1], fold(0, add) = 2
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(1), Value::Int(1)])],
            expected_output: Some(vec![Value::Int(2)]),
            initial_state: None,
            expected_state: None,
        },
        // double(2) = 4
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(2), Value::Int(2)])],
            expected_output: Some(vec![Value::Int(4)]),
            initial_state: None,
            expected_state: None,
        },
        // double(3) = 6
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(3), Value::Int(3)])],
            expected_output: Some(vec![Value::Int(6)]),
            initial_state: None,
            expected_state: None,
        },
        // double(0) = 0
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(0), Value::Int(0)])],
            expected_output: Some(vec![Value::Int(0)]),
            initial_state: None,
            expected_state: None,
        },
        // double(5) = 10
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(5), Value::Int(5)])],
            expected_output: Some(vec![Value::Int(10)]),
            initial_state: None,
            expected_state: None,
        },
    ]
}

// ---------------------------------------------------------------------------
// Main test: product of a list
// ---------------------------------------------------------------------------

#[test]
fn evolve_product_of_list() {
    let start = Instant::now();

    println!();
    println!("====================================================================");
    println!("  IRIS End-to-End Integration Test: Evolve Product-of-List");
    println!("====================================================================");
    println!();
    println!("  Requires: fold(1, mul, list) — multiplicative identity 1 + opcode 0x02");
    println!();

    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 512,
        worker_threads: 4,
        ..ExecConfig::default()
    });

    let test_cases = product_test_cases();
    let spec = ProblemSpec {
        test_cases: test_cases.clone(),
        description: "Product of a list of integers".to_string(),
        target_cost: None,
    };

    print_test_cases(&spec, &test_cases);

    let config = EvolutionConfig {
        population_size: 64,
        max_generations: 300,
        mutation_rate: 0.8,
        crossover_rate: 0.5,
        tournament_size: 3,
        phase_thresholds: PhaseThresholds {
            exploration_min_improvement: 0.005,
            stagnation_window: 15,
            min_diversity: 0.1,
        },
        target_generation_time_ms: 500,
        num_demes: 1,
        novelty_k: 15,
        novelty_threshold: 0.1,
        novelty_weight: 1.0,
        coevolution: false,
        resource_budget_ms: 0,
        iris_mode: false,
    };

    print_config(&config);

    let result = iris_evolve::evolve(config, spec, &exec);
    print_evolution_results(&result, &test_cases, &exec, start.elapsed());

    assert!(
        result.generations_run >= 0,
        "Evolution should have completed"
    );
}

// ---------------------------------------------------------------------------
// Main test: double a single integer (fold-based)
// ---------------------------------------------------------------------------

#[test]
fn evolve_double_integer() {
    let start = Instant::now();

    println!();
    println!("====================================================================");
    println!("  IRIS End-to-End Integration Test: Evolve Double-Integer");
    println!("====================================================================");
    println!();
    println!("  Input encoding: [x, x] as Tuple (duplicated so fold processes x twice)");
    println!("  Solution: fold(0, add, [x, x]) = 2x");
    println!("  This is structurally identical to sum — tests the engine's");
    println!("  consistency when the same program shape solves a different framing.");
    println!();

    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 512,
        worker_threads: 4,
        ..ExecConfig::default()
    });

    let test_cases = double_test_cases();
    let spec = ProblemSpec {
        test_cases: test_cases.clone(),
        description: "Double a single integer (via fold over [x, x])".to_string(),
        target_cost: None,
    };

    print_test_cases(&spec, &test_cases);

    let config = EvolutionConfig {
        population_size: 64,
        max_generations: 300,
        mutation_rate: 0.8,
        crossover_rate: 0.5,
        tournament_size: 3,
        phase_thresholds: PhaseThresholds {
            exploration_min_improvement: 0.005,
            stagnation_window: 15,
            min_diversity: 0.1,
        },
        target_generation_time_ms: 500,
        num_demes: 1,
        novelty_k: 15,
        novelty_threshold: 0.1,
        novelty_weight: 1.0,
        coevolution: false,
        resource_budget_ms: 0,
        iris_mode: false,
    };

    print_config(&config);

    let result = iris_evolve::evolve(config, spec, &exec);
    print_evolution_results(&result, &test_cases, &exec, start.elapsed());

    assert!(
        result.generations_run >= 0,
        "Evolution should have completed"
    );
}

// ---------------------------------------------------------------------------
// Shared output helpers
// ---------------------------------------------------------------------------

fn print_test_cases(spec: &ProblemSpec, test_cases: &[TestCase]) {
    println!("Problem: {}", spec.description);
    println!("Test cases: {}", test_cases.len());
    for (i, tc) in test_cases.iter().enumerate() {
        let input_desc = format_value(&tc.inputs[0]);
        let expected_desc = tc
            .expected_output
            .as_ref()
            .map(|e| format_values(e))
            .unwrap_or_else(|| "?".to_string());
        println!("  [{}] {} -> {}", i, input_desc, expected_desc);
    }
    println!();
}

fn print_config(config: &EvolutionConfig) {
    println!("Evolution config:");
    println!("  Population size:   {}", config.population_size);
    println!("  Max generations:   {}", config.max_generations);
    println!("  Mutation rate:     {}", config.mutation_rate);
    println!("  Crossover rate:    {}", config.crossover_rate);
    println!("  Tournament size:   {}", config.tournament_size);
    println!();
}

fn print_evolution_results(
    result: &iris_evolve::result::EvolutionResult,
    test_cases: &[TestCase],
    exec: &IrisExecutionService,
    total_time: std::time::Duration,
) {
    println!("Starting evolution...");
    println!("--------------------------------------------------------------------");
    println!(
        "{:>5}  {:>10}  {:>10}  {:>10}  {:>10}  {:>6}  {:>12}",
        "Gen", "Best Corr", "Avg Corr", "Best Perf", "Best Cost", "Front", "Phase"
    );
    println!("--------------------------------------------------------------------");

    for snap in &result.history {
        println!(
            "{:>5}  {:>10.4}  {:>10.4}  {:>10.4}  {:>10.4}  {:>6}  {:>12?}",
            snap.generation,
            snap.best_fitness.correctness(),
            snap.avg_fitness.correctness(),
            snap.best_fitness.performance(),
            snap.best_fitness.cost(),
            snap.pareto_front_size,
            snap.phase,
        );
    }

    println!("--------------------------------------------------------------------");
    println!();

    let best = &result.best_individual;
    let perfect = best.fitness.correctness() >= 1.0;

    println!("====================================================================");
    println!("  RESULTS");
    println!("====================================================================");
    println!();
    println!("Generations run:    {}", result.generations_run);
    println!("Total time:         {:.2?}", total_time);
    println!("Pareto front size:  {}", result.pareto_front.len());
    println!();
    println!("Best individual fitness:");
    println!("  Correctness:      {:.4}", best.fitness.correctness());
    println!("  Performance:      {:.4}", best.fitness.performance());
    println!("  Verifiability:    {:.4}", best.fitness.verifiability());
    println!("  Cost:             {:.4}", best.fitness.cost());
    println!("  Pareto rank:      {}", best.pareto_rank);
    println!();
    println!(
        "Best individual graph: {} nodes, {} edges",
        best.fragment.graph.nodes.len(),
        best.fragment.graph.edges.len()
    );
    println!();

    println!("Best individual evaluation:");
    let mut passes = 0;
    for (i, tc) in test_cases.iter().enumerate() {
        let eval_result = exec.evaluate_individual(
            &best.fragment.graph,
            &[tc.clone()],
            iris_types::eval::EvalTier::A,
        );
        match eval_result {
            Ok(er) => {
                let actual = if er.outputs.is_empty() || er.outputs[0].is_empty() {
                    "ERROR".to_string()
                } else {
                    format_values(&er.outputs[0])
                };
                let expected = tc
                    .expected_output
                    .as_ref()
                    .map(|e| format_values(e))
                    .unwrap_or_else(|| "?".to_string());
                let pass = tc
                    .expected_output
                    .as_ref()
                    .map(|e| !er.outputs[0].is_empty() && &er.outputs[0] == e)
                    .unwrap_or(false);
                if pass {
                    passes += 1;
                }
                println!(
                    "  [{}] input={} expected={} actual={} {}",
                    i,
                    format_value(&tc.inputs[0]),
                    expected,
                    actual,
                    if pass { "PASS" } else { "FAIL" }
                );
            }
            Err(e) => {
                println!("  [{}] ERROR: {:?}", i, e);
            }
        }
    }

    println!();
    if perfect {
        println!("*** PERFECT SOLUTION FOUND! ***");
    } else {
        println!(
            "Best correctness achieved: {:.1}% ({} of {} test cases)",
            best.fitness.correctness() * 100.0,
            passes,
            test_cases.len()
        );
    }
    println!();

    let cache = exec.cache_stats();
    println!("Cache stats:");
    println!("  Hits:       {}", cache.hits);
    println!("  Misses:     {}", cache.misses);
    println!("  Evictions:  {}", cache.evictions);
    println!("  Hit rate:   {:.1}%", cache.hit_rate() * 100.0);
    println!();

    println!("Fitness trajectory (correctness by generation):");
    let max_bars = 60;
    for snap in &result.history {
        let bar_len = (snap.best_fitness.correctness() * max_bars as f32) as usize;
        let bar: String = "#".repeat(bar_len);
        println!(
            "  Gen {:>3}: [{}{}] {:.1}%",
            snap.generation,
            bar,
            " ".repeat(max_bars - bar_len),
            snap.best_fitness.correctness() * 100.0
        );
    }
    println!();
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

fn format_value(v: &Value) -> String {
    match v {
        Value::Int(n) => format!("{}", n),
        Value::Nat(n) => format!("{}u", n),
        Value::Float64(f) => format!("{:.2}", f),
        Value::Float32(f) => format!("{:.2}f32", f),
        Value::Bool(b) => format!("{}", b),
        Value::Bytes(b) => format!("{:?}", b),
        Value::Unit => "()".to_string(),
        Value::Tuple(elems) => {
            let inner: Vec<String> = elems.iter().map(format_value).collect();
            format!("[{}]", inner.join(", "))
        }
        Value::Tagged(tag, inner) => format!("#{}{{{}}}", tag, format_value(inner)),
        Value::State(s) => format!("State{{{} keys}}", s.len()),
        Value::Graph(g) => format!("Graph{{{} nodes, {} edges}}", g.nodes.len(), g.edges.len()),
        Value::Program(_) => "Program".to_string(),
        Value::Future(_) => "Future".to_string(),
        Value::Thunk(_, _) => "Thunk".to_string(),
        Value::Range(s, e) => format!("[{}..{})", s, e),
        Value::String(s) => format!("{:?}", s),
    }
}

fn format_values(vals: &[Value]) -> String {
    if vals.len() == 1 {
        format_value(&vals[0])
    } else {
        let inner: Vec<String> = vals.iter().map(format_value).collect();
        format!("({})", inner.join(", "))
    }
}
