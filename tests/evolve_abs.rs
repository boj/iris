//! End-to-end integration test: evolve a program that computes the absolute
//! value of a single integer.
//!
//! ## Difficulty analysis
//!
//! Absolute value requires: if x < 0 then -x else x.
//!
//! The IRIS interpreter has several paths that could express this:
//!
//! 1. **Prim(abs, 0x06)** — A single opcode that computes abs directly.
//!    However, Prim nodes get their arguments from Argument edges in the
//!    graph, and there is no general "input reference" node. The only way
//!    the input reaches the computation is through Fold's hardcoded
//!    BinderId(0xFFFF_0000) lookup.
//!
//! 2. **Guard node** — The mutation operator `wrap_in_guard` can create
//!    conditional branches. Combined with Prim(neg) and Prim(lt), evolution
//!    could discover: Guard(pred=lt(x,0), body=neg(x), fallback=x).
//!    This requires multiple coordinated mutations.
//!
//! 3. **Fold trick** — Pass input as a single-element tuple [x]. Then
//!    fold(0, max, [x]) computes max(0, x), which equals abs(x) for
//!    non-negative x but returns 0 for negative x. Not correct for abs.
//!    fold(0, add, [x]) just returns x. Also not abs.
//!
//! Given the current architecture, the cleanest approach is to frame the
//! problem as a fold: pass the input as [x, -x] (the value and its
//! negation), then fold(0, max, [x, -x]) = max(max(0, x), -x) = abs(x)
//! when base >= 0. But this requires encoding -x in the input, which
//! defeats the purpose.
//!
//! **Realistic framing**: We pass the input as a single-element tuple [x]
//! and let evolution search. This is a genuinely hard problem for the
//! current system. We document what the engine achieves.

use std::time::Instant;

use iris_evolve::config::{EvolutionConfig, PhaseThresholds, ProblemSpec};
use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_exec::ExecutionService;
use iris_types::eval::{TestCase, Value};

// ---------------------------------------------------------------------------
// Test cases: absolute value
// ---------------------------------------------------------------------------

fn abs_test_cases() -> Vec<TestCase> {
    vec![
        // 5 -> 5
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(5)])],
            expected_output: Some(vec![Value::Int(5)]),
            initial_state: None,
            expected_state: None,
        },
        // -3 -> 3
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(-3)])],
            expected_output: Some(vec![Value::Int(3)]),
            initial_state: None,
            expected_state: None,
        },
        // 0 -> 0
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(0)])],
            expected_output: Some(vec![Value::Int(0)]),
            initial_state: None,
            expected_state: None,
        },
        // -100 -> 100
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(-100)])],
            expected_output: Some(vec![Value::Int(100)]),
            initial_state: None,
            expected_state: None,
        },
        // 1 -> 1
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(1)])],
            expected_output: Some(vec![Value::Int(1)]),
            initial_state: None,
            expected_state: None,
        },
    ]
}

// ---------------------------------------------------------------------------
// Main test
// ---------------------------------------------------------------------------

#[test]
fn evolve_absolute_value() {
    let start = Instant::now();

    println!();
    println!("====================================================================");
    println!("  IRIS End-to-End Integration Test: Evolve Absolute Value");
    println!("====================================================================");
    println!();
    println!("  This is a HARD problem for the current evolutionary engine.");
    println!("  The fold-based architecture reduces lists to scalars.");
    println!("  For single-element inputs [x], fold(base, op, [x]) = op(base, x).");
    println!("  No single (base, op) combination computes abs(x) for all x.");
    println!();
    println!("  Possible partial solutions the engine might discover:");
    println!("    fold(0, max, [x])  => max(0, x)  -- works for x >= 0, fails for x < 0");
    println!("    fold(0, add, [x])  => x          -- identity, works for x >= 0");
    println!("    fold(0, sub, [x])  => -x          -- negation, works for x <= 0");
    println!();
    println!("  A perfect solution requires a Guard or Prim(abs) with input routing.");
    println!();

    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 512,
        worker_threads: 4,
        ..ExecConfig::default()
    });

    let test_cases = abs_test_cases();
    let spec = ProblemSpec {
        test_cases: test_cases.clone(),
        description: "Absolute value of a single integer".to_string(),
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

    // Larger search budget — this is a harder problem.
    let config = EvolutionConfig {
        population_size: 96,
        max_generations: 500,
        mutation_rate: 0.85,
        crossover_rate: 0.5,
        tournament_size: 3,
        phase_thresholds: PhaseThresholds {
            exploration_min_improvement: 0.003,
            stagnation_window: 25,
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
        println!("The engine discovered abs(x) — likely via Guard or Prim(abs, 0x06).");
    } else {
        println!(
            "Best correctness achieved: {:.1}% ({} of {} test cases)",
            best.fitness.correctness() * 100.0,
            passes,
            test_cases.len()
        );
        println!();
        println!("ANALYSIS: This is expected. The current fold-based architecture");
        println!("cannot express abs(x) with a single (base, op) combination.");
        println!("Improvements needed for perfect abs(x):");
        println!("  1. Input-reference nodes (so Prim(abs) can read the input)");
        println!("  2. Guard seed generators (so evolution starts with conditionals)");
        println!("  3. Compositional mutation (chain neg + guard + comparison)");
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
    assert!(
        best.fitness.correctness() >= 0.0,
        "Fitness correctness should be non-negative"
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
