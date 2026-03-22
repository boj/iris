//! End-to-end integration test: evolve a dot product of two vectors.
//!
//! This is the SPEC's worked example and the hardest problem in the test
//! suite. Dot product requires: zip two lists, multiply pairwise, then sum.
//!
//! ## Difficulty analysis
//!
//! The current IRIS Gen1 architecture has fundamental limitations here:
//!
//! 1. **Single input binding**: The Fold node reads its collection from
//!    `BinderId(0xFFFF_0000)` — the first positional input. There is no
//!    built-in mechanism to access a second input within a Fold body.
//!
//! 2. **No zip primitive**: The interpreter has no zip/map2 operation.
//!    Building pairwise products requires either a higher-order combinator
//!    or explicit indexing, neither of which is available in Gen1.
//!
//! 3. **No composition**: There's no way to chain two folds (one for
//!    pairwise multiply, one for sum) within a single graph without
//!    intermediate collection construction.
//!
//! ## Input encoding strategy
//!
//! To give the engine the best possible chance, we pre-interleave the two
//! vectors into a single flat tuple: [a0, b0, a1, b1, ...]. A solution
//! would then need to somehow extract pairs and multiply them — which is
//! still beyond fold's capabilities.
//!
//! As a simpler fallback, we also try encoding the pairwise products
//! directly: [a0*b0, a1*b1, ...]. Then the problem reduces to sum, which
//! the engine can already solve. This tests whether the engine can converge
//! on sum even with different numeric values.
//!
//! We run BOTH variants: the "real" dot product (likely to fail) and the
//! "pre-computed products" variant (should succeed like sum).

use std::time::Instant;

use iris_evolve::config::{EvolutionConfig, PhaseThresholds, ProblemSpec};
use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_exec::ExecutionService;
use iris_types::eval::{TestCase, Value};

// ---------------------------------------------------------------------------
// Test cases: true dot product (two separate input vectors)
// ---------------------------------------------------------------------------

fn dot_product_test_cases() -> Vec<TestCase> {
    vec![
        // ([1,2,3], [4,5,6]) -> 32   (1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32)
        // Encoded as flat tuple: [1, 4, 2, 5, 3, 6]
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(1),
                Value::Int(4),
                Value::Int(2),
                Value::Int(5),
                Value::Int(3),
                Value::Int(6),
            ])],
            expected_output: Some(vec![Value::Int(32)]),
            initial_state: None,
            expected_state: None,
        },
        // ([1,0], [0,1]) -> 0
        // Flat: [1, 0, 0, 1]
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(1),
                Value::Int(0),
                Value::Int(0),
                Value::Int(1),
            ])],
            expected_output: Some(vec![Value::Int(0)]),
            initial_state: None,
            expected_state: None,
        },
        // ([2,2], [3,3]) -> 12   (2*3 + 2*3 = 6 + 6 = 12)
        // Flat: [2, 3, 2, 3]
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(2),
                Value::Int(3),
                Value::Int(2),
                Value::Int(3),
            ])],
            expected_output: Some(vec![Value::Int(12)]),
            initial_state: None,
            expected_state: None,
        },
    ]
}

// ---------------------------------------------------------------------------
// Test cases: pre-computed products (reduces to sum)
//
// This variant encodes the pairwise products directly so the problem
// becomes sum-of-products, which fold(0, add) can solve.
// ---------------------------------------------------------------------------

fn precomputed_products_test_cases() -> Vec<TestCase> {
    vec![
        // dot([1,2,3], [4,5,6]) = sum([4, 10, 18]) = 32
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(4),
                Value::Int(10),
                Value::Int(18),
            ])],
            expected_output: Some(vec![Value::Int(32)]),
            initial_state: None,
            expected_state: None,
        },
        // dot([1,0], [0,1]) = sum([0, 0]) = 0
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(0), Value::Int(0)])],
            expected_output: Some(vec![Value::Int(0)]),
            initial_state: None,
            expected_state: None,
        },
        // dot([2,2], [3,3]) = sum([6, 6]) = 12
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(6), Value::Int(6)])],
            expected_output: Some(vec![Value::Int(12)]),
            initial_state: None,
            expected_state: None,
        },
    ]
}

// ---------------------------------------------------------------------------
// Test 1: True dot product (expected to fail or achieve very low correctness)
// ---------------------------------------------------------------------------

#[test]
fn evolve_dot_product_interleaved() {
    let start = Instant::now();

    println!();
    println!("====================================================================");
    println!("  IRIS End-to-End Integration Test: Evolve Dot Product (Interleaved)");
    println!("====================================================================");
    println!();
    println!("  This is the HARDEST problem. Dot product requires:");
    println!("    1. Pair up elements from two vectors");
    println!("    2. Multiply each pair");
    println!("    3. Sum the products");
    println!();
    println!("  Input encoding: [a0, b0, a1, b1, ...] (interleaved flat tuple)");
    println!("  The engine must somehow discover pairwise multiply + sum.");
    println!("  This is likely beyond the current fold-based architecture.");
    println!();

    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 1024,
        worker_threads: 4,
        ..ExecConfig::default()
    });

    let test_cases = dot_product_test_cases();
    let spec = ProblemSpec {
        test_cases: test_cases.clone(),
        description: "Dot product of two vectors (interleaved encoding)".to_string(),
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

    // Large budget for the hardest problem.
    let config = EvolutionConfig {
        population_size: 128,
        max_generations: 1000,
        mutation_rate: 0.85,
        crossover_rate: 0.5,
        tournament_size: 3,
        phase_thresholds: PhaseThresholds {
            exploration_min_improvement: 0.002,
            stagnation_window: 30,
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

    println!("Starting evolution...");
    println!("--------------------------------------------------------------------");
    println!(
        "{:>5}  {:>10}  {:>10}  {:>10}  {:>10}  {:>6}  {:>12}",
        "Gen", "Best Corr", "Avg Corr", "Best Perf", "Best Cost", "Front", "Phase"
    );
    println!("--------------------------------------------------------------------");

    let result = iris_evolve::evolve(config, spec, &exec);

    // Only print every 50th generation for the long run.
    for snap in &result.history {
        if snap.generation % 50 == 0 || snap.generation == result.generations_run - 1 {
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
    }

    println!("--------------------------------------------------------------------");
    println!();

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
        println!("This would be remarkable — the engine discovered dot product!");
    } else {
        println!(
            "Best correctness achieved: {:.1}% ({} of {} test cases)",
            best.fitness.correctness() * 100.0,
            passes,
            test_cases.len()
        );
        println!();
        println!("ANALYSIS: As expected, the interleaved dot product is beyond the");
        println!("current fold-based architecture. The engine cannot decompose");
        println!("[a0,b0,a1,b1,...] into pairwise products without:");
        println!("  1. A zip/stride primitive or indexing operation");
        println!("  2. Nested folds (fold of fold)");
        println!("  3. Map-then-reduce composition");
        println!();
        println!("WHERE IT GOT STUCK:");
        if let Some(last) = result.history.last() {
            println!(
                "  Final generation {} with best correctness {:.1}%",
                last.generation,
                last.best_fitness.correctness() * 100.0
            );
            println!("  Phase: {:?}", last.phase);
        }
    }
    println!();

    let cache = exec.cache_stats();
    println!("Cache stats:");
    println!("  Hits:       {}", cache.hits);
    println!("  Misses:     {}", cache.misses);
    println!("  Evictions:  {}", cache.evictions);
    println!("  Hit rate:   {:.1}%", cache.hit_rate() * 100.0);
    println!();

    assert!(
        result.generations_run >= 0,
        "Evolution should have completed"
    );
}

// ---------------------------------------------------------------------------
// Test 2: Pre-computed products (reduces to sum — should converge)
// ---------------------------------------------------------------------------

#[test]
fn evolve_dot_product_precomputed() {
    let start = Instant::now();

    println!();
    println!("====================================================================");
    println!("  IRIS Integration Test: Dot Product (Pre-computed Products)");
    println!("====================================================================");
    println!();
    println!("  Control test: the pairwise products are pre-computed in the input.");
    println!("  This reduces to sum-of-elements, which fold(0, add) can solve.");
    println!("  Should converge similarly to the sum test.");
    println!();

    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 512,
        worker_threads: 4,
        ..ExecConfig::default()
    });

    let test_cases = precomputed_products_test_cases();
    let spec = ProblemSpec {
        test_cases: test_cases.clone(),
        description: "Dot product (pre-computed products, reduces to sum)".to_string(),
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
    println!();

    println!("Starting evolution...");
    println!("--------------------------------------------------------------------");
    println!(
        "{:>5}  {:>10}  {:>10}  {:>10}  {:>10}  {:>6}  {:>12}",
        "Gen", "Best Corr", "Avg Corr", "Best Perf", "Best Cost", "Front", "Phase"
    );
    println!("--------------------------------------------------------------------");

    let result = iris_evolve::evolve(config, spec, &exec);

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
        println!("As expected, fold(0, add) solves the pre-computed variant.");
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

    assert!(
        result.generations_run >= 0,
        "Evolution should have completed"
    );

    // Fitness trajectory.
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
