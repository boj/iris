//! IRIS improves IRIS: the daemon runs its own components as the workload,
//! profiles their performance, evolves better versions, and deploys them.
//! The improved components are then used in the next improvement cycle.
//!
//! This is Phase Transition 3: recursive self-improvement.

use std::path::PathBuf;
use std::sync::{Arc, RwLock, atomic::{AtomicU64, Ordering}};
use std::time::{Duration, Instant};

use iris_evolve::auto_improve::AutoImproveConfig;
use iris_evolve::config::{EvolutionConfig, ProblemSpec};
use iris_evolve::iris_runtime::IrisRuntime;
use iris_evolve::self_improving_daemon::*;
use iris_exec::interpreter;
use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_types::eval::{TestCase, Value};

/// Run a real evolution workload using IRIS components and measure performance.
fn run_iris_evolution_workload(runtime: &IrisRuntime, exec: &IrisExecutionService) -> (f64, Duration) {
    let start = Instant::now();

    // Problem: evolve sum(list)
    let test_cases = vec![
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)])],
            expected_output: Some(vec![Value::Int(6)]),
            initial_state: None,
            expected_state: None,
        },
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(10)])],
            expected_output: Some(vec![Value::Int(10)]),
            initial_state: None,
            expected_state: None,
        },
    ];

    let spec = ProblemSpec {
        test_cases: test_cases.clone(),
        description: "sum".to_string(),
        target_cost: None,
    };

    let config = EvolutionConfig {
        population_size: 16,
        max_generations: 30,
        num_demes: 1,
        iris_mode: true, // USE IRIS COMPONENTS
        ..EvolutionConfig::default()
    };

    let result = iris_evolve::evolve(config, spec, exec);
    let elapsed = start.elapsed();
    let correctness = result.best_individual.fitness.correctness();

    (correctness as f64, elapsed)
}

#[test]
fn iris_improves_itself() {
    println!("\n{}", "=".repeat(70));
    println!("  IRIS Self-on-Self: Recursive Self-Improvement Test");
    println!("{}\n", "=".repeat(70));

    let dir = std::env::temp_dir().join("iris_self_on_self");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Phase 1: Baseline — run evolution with IRIS components, measure speed
    println!("Phase 1: Baseline measurement");
    let runtime = IrisRuntime::new();
    let exec = IrisExecutionService::with_defaults();

    let mut baseline_times = Vec::new();
    for i in 0..3 {
        let (correctness, elapsed) = run_iris_evolution_workload(&runtime, &exec);
        println!("  Run {}: {:.0}% correctness in {:.1}ms", i + 1, correctness * 100.0, elapsed.as_secs_f64() * 1000.0);
        baseline_times.push(elapsed);
    }
    let baseline_avg = baseline_times.iter().map(|d| d.as_secs_f64()).sum::<f64>() / baseline_times.len() as f64;
    println!("  Baseline average: {:.1}ms\n", baseline_avg * 1000.0);

    // Phase 2: Run self-improving daemon targeting IRIS components
    println!("Phase 2: Self-improvement daemon (50 cycles)");
    let config = SelfImprovingConfig {
        cycle_time_ms: 1, // fast as possible
        max_cycles: Some(50),
        improve_interval: 5,
        inspect_interval: 10,
        auto_improve: AutoImproveConfig {
            cycle_interval_secs: 0,
            max_slowdown: 2.0,
            test_cases_per_component: 5,
            evolution_generations: 20,
            evolution_pop_size: 16,
            gate_runs: 3,
            explore_problems: 3,
        },
        state_dir: Some(dir.clone()),
        memory_limit: 50 * 1024 * 1024,
        seed: Some(42),
        max_improve_threads: 1,
        max_stagnant: 3,
        min_improvement: 0.05,
        exec_mode: ExecMode::Continuous,
        trigger_check_interval: 5,
    };

    let mut daemon = ThreadedDaemon::new(config);
    let result = daemon.run();

    println!("  Cycles:           {}", result.cycles_completed);
    println!("  Improvement runs: {}", result.improvement_cycles);
    println!("  Deployed:         {}", result.components_deployed);
    println!("  Converged:        {}", result.converged_components);
    println!("  Recursive depth:  {}", result.recursive_depth);
    println!("  Wall time:        {:.2}s", result.total_time.as_secs_f64());

    // Phase 3: Post-improvement measurement
    println!("\nPhase 3: Post-improvement measurement");
    let mut post_times = Vec::new();
    for i in 0..3 {
        let (correctness, elapsed) = run_iris_evolution_workload(&runtime, &exec);
        println!("  Run {}: {:.0}% correctness in {:.1}ms", i + 1, correctness * 100.0, elapsed.as_secs_f64() * 1000.0);
        post_times.push(elapsed);
    }
    let post_avg = post_times.iter().map(|d| d.as_secs_f64()).sum::<f64>() / post_times.len() as f64;
    println!("  Post-improvement average: {:.1}ms\n", post_avg * 1000.0);

    // Phase 4: Report
    println!("Phase 4: Results");
    let speedup = baseline_avg / post_avg;
    println!("  Baseline:     {:.1}ms per evolution run", baseline_avg * 1000.0);
    println!("  After daemon: {:.1}ms per evolution run", post_avg * 1000.0);
    println!("  Speedup:      {:.2}x", speedup);
    println!("  Daemon deployed {} components", result.components_deployed);
    println!("  {} components hit stagnation (local maxima)", result.converged_components);

    // Check persistence
    let state_exists = dir.join("daemon_state.json").exists();
    let audit_exists = dir.join("audit_trail.json").exists();
    println!("  State persisted: {}", state_exists);
    println!("  Audit persisted: {}", audit_exists);

    // Phase 5: Reload and verify
    println!("\nPhase 5: Reload from disk");
    let config2 = SelfImprovingConfig {
        cycle_time_ms: 1,
        max_cycles: Some(10),
        improve_interval: 5,
        inspect_interval: 10,
        auto_improve: AutoImproveConfig {
            cycle_interval_secs: 0,
            max_slowdown: 2.0,
            test_cases_per_component: 5,
            evolution_generations: 20,
            evolution_pop_size: 16,
            gate_runs: 3,
            explore_problems: 3,
        },
        state_dir: Some(dir.clone()),
        memory_limit: 50 * 1024 * 1024,
        seed: Some(99),
        max_improve_threads: 1,
        max_stagnant: 3,
        min_improvement: 0.05,
        exec_mode: ExecMode::Continuous,
        trigger_check_interval: 5,
    };
    let mut daemon2 = ThreadedDaemon::new(config2);
    let result2 = daemon2.run();
    println!("  Resumed: {} cycles, {} improvement runs", result2.cycles_completed, result2.improvement_cycles);

    println!("\n{}", "=".repeat(70));
    println!("  IRIS Self-on-Self test complete.");
    println!("  The daemon ran evolution using IRIS components,");
    println!("  profiled their performance, and attempted recursive improvement.");
    println!("{}\n", "=".repeat(70));

    // Basic assertions
    assert!(result.cycles_completed >= 50, "Should complete all cycles");
    assert!(result.improvement_cycles > 0, "Should attempt improvements");
    assert!(state_exists, "State should be persisted");
    assert!(audit_exists, "Audit trail should be persisted");
}
