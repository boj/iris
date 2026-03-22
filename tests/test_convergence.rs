//! Evolution convergence experiments.
//!
//! Runs each evolution problem multiple times with different RNG seeds,
//! outputting CSV data for analysis.
//!
//! Run: cargo test --test test_convergence -- --nocapture --ignored
//! Output: prints CSV to stdout (pipe to file)

use std::time::Instant;

use iris_evolve::config::{EvolutionConfig, PhaseThresholds, ProblemSpec};
use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_exec::ExecutionService;
use iris_types::eval::{TestCase, Value};

// ---------------------------------------------------------------------------
// Problem definitions
// ---------------------------------------------------------------------------

fn sum_cases() -> Vec<TestCase> {
    vec![
        TestCase { inputs: vec![Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)])], expected_output: Some(vec![Value::Int(6)]), initial_state: None, expected_state: None },
        TestCase { inputs: vec![Value::tuple(vec![Value::Int(10)])], expected_output: Some(vec![Value::Int(10)]), initial_state: None, expected_state: None },
        TestCase { inputs: vec![Value::tuple(vec![])], expected_output: Some(vec![Value::Int(0)]), initial_state: None, expected_state: None },
        TestCase { inputs: vec![Value::tuple(vec![Value::Int(1), Value::Int(1), Value::Int(1), Value::Int(1), Value::Int(1)])], expected_output: Some(vec![Value::Int(5)]), initial_state: None, expected_state: None },
        TestCase { inputs: vec![Value::tuple(vec![Value::Int(100), Value::Int(-50)])], expected_output: Some(vec![Value::Int(50)]), initial_state: None, expected_state: None },
        TestCase { inputs: vec![Value::tuple(vec![Value::Int(0), Value::Int(0), Value::Int(0)])], expected_output: Some(vec![Value::Int(0)]), initial_state: None, expected_state: None },
    ]
}

fn max_cases() -> Vec<TestCase> {
    vec![
        TestCase { inputs: vec![Value::tuple(vec![Value::Int(3), Value::Int(1), Value::Int(2)])], expected_output: Some(vec![Value::Int(3)]), initial_state: None, expected_state: None },
        TestCase { inputs: vec![Value::tuple(vec![Value::Int(5)])], expected_output: Some(vec![Value::Int(5)]), initial_state: None, expected_state: None },
        TestCase { inputs: vec![Value::tuple(vec![Value::Int(-1), Value::Int(-5), Value::Int(-2)])], expected_output: Some(vec![Value::Int(-1)]), initial_state: None, expected_state: None },
        TestCase { inputs: vec![Value::tuple(vec![Value::Int(10), Value::Int(20), Value::Int(15)])], expected_output: Some(vec![Value::Int(20)]), initial_state: None, expected_state: None },
        TestCase { inputs: vec![Value::tuple(vec![Value::Int(0), Value::Int(0)])], expected_output: Some(vec![Value::Int(0)]), initial_state: None, expected_state: None },
    ]
}

fn double_cases() -> Vec<TestCase> {
    vec![
        TestCase { inputs: vec![Value::Int(1)], expected_output: Some(vec![Value::Int(2)]), initial_state: None, expected_state: None },
        TestCase { inputs: vec![Value::Int(5)], expected_output: Some(vec![Value::Int(10)]), initial_state: None, expected_state: None },
        TestCase { inputs: vec![Value::Int(0)], expected_output: Some(vec![Value::Int(0)]), initial_state: None, expected_state: None },
        TestCase { inputs: vec![Value::Int(-3)], expected_output: Some(vec![Value::Int(-6)]), initial_state: None, expected_state: None },
        TestCase { inputs: vec![Value::Int(100)], expected_output: Some(vec![Value::Int(200)]), initial_state: None, expected_state: None },
    ]
}

// ---------------------------------------------------------------------------
// Experiment runner
// ---------------------------------------------------------------------------

struct Problem {
    name: &'static str,
    cases: Vec<TestCase>,
    description: &'static str,
    population: usize,
    generations: usize,
    mutation_rate: f64,
}

fn run_experiment(problem: &Problem, run_id: usize) {
    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 512,
        worker_threads: 2,
        ..ExecConfig::default()
    });

    let spec = ProblemSpec {
        test_cases: problem.cases.clone(),
        description: problem.description.to_string(),
        target_cost: None,
    };

    let config = EvolutionConfig {
        population_size: problem.population,
        max_generations: problem.generations,
        mutation_rate: problem.mutation_rate,
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

    let start = Instant::now();
    let result = iris_evolve::evolve(config, spec, &exec);
    let total_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Output per-generation CSV rows
    for snap in &result.history {
        println!(
            "{},{},{},{:.6},{:.6},{:.6},{:.6},{},{:?},{:.1}",
            problem.name,
            run_id,
            snap.generation,
            snap.best_fitness.correctness(),
            snap.avg_fitness.correctness(),
            snap.best_fitness.performance(),
            snap.best_fitness.cost(),
            snap.pareto_front_size,
            snap.phase,
            total_ms,
        );
    }
}

// ---------------------------------------------------------------------------
// Tests (ignored by default — run explicitly)
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn convergence_all_problems() {
    let problems = vec![
        Problem {
            name: "sum",
            cases: sum_cases(),
            description: "Sum of list",
            population: 64,
            generations: 200,
            mutation_rate: 0.8_f64,
        },
        Problem {
            name: "max",
            cases: max_cases(),
            description: "Max of list",
            population: 64,
            generations: 300,
            mutation_rate: 0.8_f64,
        },
        Problem {
            name: "double",
            cases: double_cases(),
            description: "Double an integer",
            population: 64,
            generations: 200,
            mutation_rate: 0.8_f64,
        },
    ];

    // CSV header
    println!("problem,run,generation,best_correctness,avg_correctness,best_performance,best_cost,pareto_front,phase,total_ms");

    let runs_per_problem = 5;

    for problem in &problems {
        for run in 0..runs_per_problem {
            run_experiment(problem, run);
        }
    }
}

/// Quick single-run smoke test (not ignored)
#[test]
fn convergence_smoke_test() {
    let problem = Problem {
        name: "sum",
        cases: sum_cases(),
        description: "Sum of list",
        population: 32,
        generations: 50,
        mutation_rate: 0.8_f64,
    };

    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 256,
        worker_threads: 2,
        ..ExecConfig::default()
    });

    let spec = ProblemSpec {
        test_cases: problem.cases.clone(),
        description: problem.description.to_string(),
        target_cost: None,
    };

    let config = EvolutionConfig {
        population_size: problem.population,
        max_generations: problem.generations,
        mutation_rate: problem.mutation_rate,
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

    let start = Instant::now();
    let result = iris_evolve::evolve(config, spec, &exec);
    let elapsed = start.elapsed();

    let best = result.best_individual.fitness.correctness();
    let gens = result.history.len();

    eprintln!(
        "Smoke test: sum — best={:.1}% in {} gens, {:.0}ms",
        best * 100.0,
        gens,
        elapsed.as_secs_f64() * 1000.0,
    );

    // Should at least make some progress
    assert!(best > 0.0, "evolution should find at least a partially correct solution");
}

// ---------------------------------------------------------------------------
// Parameter sweep (ignored — run explicitly)
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn parameter_sweep() {
    let cases = double_cases();
    let populations = [32, 64, 128];
    let mutation_rates = [0.5, 0.8, 0.95];
    let tournament_sizes = [2, 3, 5];

    println!("pop,mutation_rate,tournament_size,best_correctness,generations,time_ms");

    for &pop in &populations {
        for &mut_rate in &mutation_rates {
            for &tourn in &tournament_sizes {
                let exec = IrisExecutionService::new(ExecConfig {
                    cache_capacity: 512,
                    worker_threads: 2,
                    ..ExecConfig::default()
                });

                let spec = ProblemSpec {
                    test_cases: cases.clone(),
                    description: "Double an integer".to_string(),
                    target_cost: None,
                };

                let config = EvolutionConfig {
                    population_size: pop,
                    max_generations: 200,
                    mutation_rate: mut_rate,
                    crossover_rate: 0.5,
                    tournament_size: tourn,
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

                let start = Instant::now();
                let result = iris_evolve::evolve(config, spec, &exec);
                let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

                let best = result.best_individual.fitness.correctness();
                let gens = result.history.len();

                println!(
                    "{},{},{},{:.6},{},{:.1}",
                    pop, mut_rate, tourn, best, gens, elapsed_ms
                );
            }
        }
    }
}
