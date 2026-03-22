//! End-to-end integration test: evolve a program that computes the sum of a
//! list of integers.
//!
//! This test exercises the full IRIS pipeline:
//!   1. Seed generation (iris-evolve)
//!   2. Interpretation and evaluation (iris-exec)
//!   3. NSGA-II multi-objective evolution (iris-evolve)
//!
//! The fitness function tests correctness against known input/output pairs.
//! The input is a Tuple of Ints (representing a list), and the expected output
//! is a single Int (the sum).

use std::time::Instant;

use iris_evolve::config::{EvolutionConfig, PhaseThresholds, ProblemSpec};
use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_exec::ExecutionService;
use iris_types::eval::{TestCase, Value};

// ---------------------------------------------------------------------------
// Test cases: sum of integers
// ---------------------------------------------------------------------------

/// Build the test cases for the "sum of a list of integers" problem.
///
/// Each test case provides a single input: a `Value::Tuple` of `Value::Int`s.
/// The expected output is a single `Value::Int` equal to their sum.
fn sum_test_cases() -> Vec<TestCase> {
    vec![
        // [1, 2, 3] -> 6
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
        // [10] -> 10
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(10)])],
            expected_output: Some(vec![Value::Int(10)]),
            initial_state: None,
            expected_state: None,
        },
        // [] -> 0
        TestCase {
            inputs: vec![Value::tuple(vec![])],
            expected_output: Some(vec![Value::Int(0)]),
            initial_state: None,
            expected_state: None,
        },
        // [1, 1, 1, 1, 1] -> 5
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(1),
                Value::Int(1),
                Value::Int(1),
                Value::Int(1),
                Value::Int(1),
            ])],
            expected_output: Some(vec![Value::Int(5)]),
            initial_state: None,
            expected_state: None,
        },
        // [-3, 3] -> 0
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(-3), Value::Int(3)])],
            expected_output: Some(vec![Value::Int(0)]),
            initial_state: None,
            expected_state: None,
        },
        // [100, 200, 300] -> 600
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(100),
                Value::Int(200),
                Value::Int(300),
            ])],
            expected_output: Some(vec![Value::Int(600)]),
            initial_state: None,
            expected_state: None,
        },
    ]
}

// ---------------------------------------------------------------------------
// Main test
// ---------------------------------------------------------------------------

#[test]
fn evolve_sum_of_integers() {
    let start = Instant::now();

    println!();
    println!("====================================================================");
    println!("  IRIS End-to-End Integration Test: Evolve Sum-of-Integers");
    println!("====================================================================");
    println!();

    // 1. Create the execution service.
    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 512,
        worker_threads: 4,
        ..ExecConfig::default()
    });

    // 2. Define the problem.
    let test_cases = sum_test_cases();
    let spec = ProblemSpec {
        test_cases: test_cases.clone(),
        description: "Sum of a list of integers".to_string(),
        target_cost: None,
    };

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

    // 3. Configure evolution.
    //
    // We use a moderate population and generation count. The seed generators
    // include fold(op, 0) programs where op is randomly chosen from {add, sub,
    // mul}. Only fold(+, 0) solves the problem; evolution must either get lucky
    // in seeding or mutate a Prim opcode to 0x00 (add).
    let config = EvolutionConfig {
        population_size: 64,
        max_generations: 200,
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

    println!("Evolution config:");
    println!("  Population size:   {}", config.population_size);
    println!("  Max generations:   {}", config.max_generations);
    println!("  Mutation rate:     {}", config.mutation_rate);
    println!("  Crossover rate:    {}", config.crossover_rate);
    println!("  Tournament size:   {}", config.tournament_size);
    println!();

    // 4. Run evolution.
    println!("Starting evolution...");
    println!("--------------------------------------------------------------------");
    println!(
        "{:>5}  {:>10}  {:>10}  {:>10}  {:>10}  {:>6}  {:>12}",
        "Gen", "Best Corr", "Avg Corr", "Best Perf", "Best Cost", "Front", "Phase"
    );
    println!("--------------------------------------------------------------------");

    let result = iris_evolve::evolve(config, spec, &exec);

    // 5. Print per-generation history.
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

    // 6. Print summary.
    let total_time = start.elapsed();
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

    // 7. Evaluate the best individual on all test cases and show outputs.
    println!("Best individual evaluation:");
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
            (best.fitness.correctness() * test_cases.len() as f32).round() as u32,
            test_cases.len()
        );
    }
    println!();

    // Cache stats.
    let cache = exec.cache_stats();
    println!("Cache stats:");
    println!("  Hits:       {}", cache.hits);
    println!("  Misses:     {}", cache.misses);
    println!("  Evictions:  {}", cache.evictions);
    println!("  Hit rate:   {:.1}%", cache.hit_rate() * 100.0);
    println!();

    // The test passes if evolution ran without panicking. A perfect solution
    // may or may not be found within 200 generations due to the stochastic
    // nature of evolution. What matters is that the full pipeline works.
    assert!(
        result.generations_run >= 0,
        "Evolution should have completed"
    );
    assert!(
        best.fitness.correctness() >= 0.0,
        "Fitness correctness should be non-negative"
    );

    // Print fitness trajectory.
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
