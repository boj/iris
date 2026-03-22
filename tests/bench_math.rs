//! Comprehensive math library benchmarks for IRIS.
//!
//! Evolves mathematical functions and measures their correctness and
//! performance against known hand-written Rust implementations.
//!
//! For each problem:
//!   1. Defines test cases with known correct answers
//!   2. Evolves a solution using IRIS
//!   3. Measures wall-clock time for evolution
//!   4. Measures wall-clock time for executing the evolved solution on 1000 inputs
//!   5. Compares execution time to a hand-written Rust equivalent
//!   6. Prints a benchmark report

use std::time::{Duration, Instant};

use iris_evolve::config::{EvolutionConfig, PhaseThresholds, ProblemSpec};
use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_exec::ExecutionService;
use iris_types::eval::{EvalTier, TestCase, Value};

// =========================================================================
// Benchmark infrastructure
// =========================================================================

/// Result of benchmarking a single math problem.
struct BenchResult {
    name: &'static str,
    evolved: bool,
    correctness: f32,
    generations_run: usize,
    evolve_time: Duration,
    iris_exec_time: Option<Duration>,
    rust_exec_time: Duration,
    num_nodes: usize,
    num_edges: usize,
    failure_reason: Option<String>,
}

impl BenchResult {
    fn ratio(&self) -> Option<f64> {
        self.iris_exec_time.map(|iris| {
            let iris_us = iris.as_secs_f64() * 1_000_000.0;
            let rust_us = self.rust_exec_time.as_secs_f64() * 1_000_000.0;
            if rust_us < 0.001 {
                iris_us / 0.001
            } else {
                iris_us / rust_us
            }
        })
    }

    fn status_str(&self) -> String {
        if self.evolved {
            "YES".to_string()
        } else if self.correctness > 0.0 {
            format!("NO ({:.0}%)", self.correctness * 100.0)
        } else {
            "NO (0%)".to_string()
        }
    }
}

/// Standard evolution config for all benchmark problems.
fn bench_config() -> EvolutionConfig {
    EvolutionConfig {
        population_size: 256,
        max_generations: 2000,
        mutation_rate: 0.8,
        crossover_rate: 0.5,
        tournament_size: 3,
        phase_thresholds: PhaseThresholds {
            exploration_min_improvement: 0.005,
            stagnation_window: 20,
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
    }
}

/// Hard problem config with larger budget.
fn bench_config_hard() -> EvolutionConfig {
    EvolutionConfig {
        population_size: 512,
        max_generations: 3000,
        mutation_rate: 0.8,
        crossover_rate: 0.5,
        tournament_size: 3,
        phase_thresholds: PhaseThresholds {
            exploration_min_improvement: 0.005,
            stagnation_window: 20,
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
    }
}

/// Run evolution for a problem and return the benchmark result.
/// Retries up to MAX_ATTEMPTS times, taking the best result.
fn run_bench(
    name: &'static str,
    test_cases: Vec<TestCase>,
    exec: &IrisExecutionService,
    bench_inputs_fn: fn() -> Vec<Vec<Value>>,
    rust_baseline_fn: fn(&[Value]) -> Value,
) -> BenchResult {
    run_bench_with_config(name, test_cases, exec, bench_inputs_fn, rust_baseline_fn, bench_config())
}

fn run_bench_hard(
    name: &'static str,
    test_cases: Vec<TestCase>,
    exec: &IrisExecutionService,
    bench_inputs_fn: fn() -> Vec<Vec<Value>>,
    rust_baseline_fn: fn(&[Value]) -> Value,
) -> BenchResult {
    run_bench_with_config(name, test_cases, exec, bench_inputs_fn, rust_baseline_fn, bench_config_hard())
}

const MAX_ATTEMPTS: usize = 3;

fn run_bench_with_config(
    name: &'static str,
    test_cases: Vec<TestCase>,
    exec: &IrisExecutionService,
    bench_inputs_fn: fn() -> Vec<Vec<Value>>,
    rust_baseline_fn: fn(&[Value]) -> Value,
    config: EvolutionConfig,
) -> BenchResult {
    let mut best_result = None;
    let mut best_correctness = -1.0f32;
    let mut total_evolve_time = Duration::ZERO;
    let mut total_generations = 0usize;

    for attempt in 0..MAX_ATTEMPTS {
        let spec = ProblemSpec {
            test_cases: test_cases.clone(),
            description: name.to_string(),
            target_cost: None,
        };

        let evolve_start = Instant::now();
        let result = iris_evolve::evolve(config.clone(), spec, exec);
        let attempt_time = evolve_start.elapsed();
        total_evolve_time += attempt_time;
        total_generations += result.generations_run;

        let correctness = result.best_individual.fitness.correctness();
        if correctness > best_correctness {
            best_correctness = correctness;
            best_result = Some(result);
        }

        // Early exit if solved
        if best_correctness >= 1.0 {
            if attempt > 0 {
                println!("       (solved on attempt {})", attempt + 1);
            }
            break;
        }
    }

    let result = best_result.unwrap();
    let evolve_time = total_evolve_time;

    let best = &result.best_individual;
    let correctness = best.fitness.correctness();
    let evolved = correctness >= 1.0;
    let generations_run = result.generations_run;
    let num_nodes = best.fragment.graph.nodes.len();
    let num_edges = best.fragment.graph.edges.len();

    // 2. Prepare 1000 random benchmark inputs
    let bench_inputs = bench_inputs_fn();

    // 3. Measure Rust baseline on the 1000 inputs
    let rust_start = Instant::now();
    for inputs in &bench_inputs {
        std::hint::black_box(rust_baseline_fn(inputs));
    }
    let rust_exec_time = rust_start.elapsed();

    // 4. Measure IRIS execution on the 1000 inputs (only if evolved successfully)
    let (iris_exec_time, failure_reason) = if evolved {
        let iris_start = Instant::now();
        for inputs in &bench_inputs {
            let tc = TestCase {
                inputs: inputs.clone(),
                expected_output: None,
                initial_state: None,
                expected_state: None,
            };
            let _ = exec.evaluate_individual(
                &best.fragment.graph,
                &[tc],
                EvalTier::A,
            );
        }
        let iris_time = iris_start.elapsed();
        (Some(iris_time), None)
    } else {
        // Diagnose why it failed: evaluate on test cases and see what happened
        let mut pass_count = 0;
        let mut fail_details = Vec::new();
        for tc in &test_cases {
            match exec.evaluate_individual(
                &best.fragment.graph,
                &[tc.clone()],
                EvalTier::A,
            ) {
                Ok(er) => {
                    if let Some(expected) = &tc.expected_output {
                        if !er.outputs.is_empty() && !er.outputs[0].is_empty() && &er.outputs[0] == expected {
                            pass_count += 1;
                        } else {
                            let actual = if er.outputs.is_empty() || er.outputs[0].is_empty() {
                                "EMPTY".to_string()
                            } else {
                                format_values(&er.outputs[0])
                            };
                            let exp = format_values(expected);
                            fail_details.push(format!("expected={} got={}", exp, actual));
                        }
                    }
                }
                Err(e) => {
                    fail_details.push(format!("eval error: {:?}", e));
                }
            }
        }
        let reason = if fail_details.is_empty() {
            format!("Passed {}/{} test cases", pass_count, test_cases.len())
        } else {
            let shown: Vec<&str> = fail_details.iter().take(3).map(|s| s.as_str()).collect();
            format!(
                "Passed {}/{}: {}{}",
                pass_count,
                test_cases.len(),
                shown.join("; "),
                if fail_details.len() > 3 { "..." } else { "" }
            )
        };
        (None, Some(reason))
    };

    BenchResult {
        name,
        evolved,
        correctness,
        generations_run,
        evolve_time,
        iris_exec_time,
        rust_exec_time,
        num_nodes,
        num_edges,
        failure_reason,
    }
}

// =========================================================================
// Problem 1: Factorial
// =========================================================================

fn factorial_test_cases() -> Vec<TestCase> {
    let cases: Vec<(i64, i64)> = vec![
        (0, 1), (1, 1), (2, 2), (3, 6), (4, 24), (5, 120), (6, 720), (7, 5040),
    ];
    cases
        .into_iter()
        .map(|(n, expected)| {
            // Input as a tuple [1, 2, ..., n] so fold can iterate over it
            let list: Vec<Value> = (1..=n).map(Value::Int).collect();
            TestCase {
                inputs: vec![Value::Tuple(list)],
                expected_output: Some(vec![Value::Int(expected)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect()
}

fn factorial_bench_inputs() -> Vec<Vec<Value>> {
    (0..1000)
        .map(|i| {
            let n = (i % 8) as i64;
            let list: Vec<Value> = (1..=n).map(Value::Int).collect();
            vec![Value::Tuple(list)]
        })
        .collect()
}

fn factorial_rust(inputs: &[Value]) -> Value {
    if let Value::Tuple(elems) = &inputs[0] {
        let n = elems.len() as i64;
        let result: i64 = (1..=n).product();
        Value::Int(result)
    } else {
        Value::Int(0)
    }
}

// =========================================================================
// Problem 2: Fibonacci
// =========================================================================

fn fibonacci_test_cases() -> Vec<TestCase> {
    let cases: Vec<(i64, i64)> = vec![
        (0, 0), (1, 1), (2, 1), (3, 2), (4, 3), (5, 5), (6, 8), (7, 13), (8, 21),
    ];
    cases
        .into_iter()
        .map(|(n, expected)| {
            // Fibonacci is hard for fold-based evolution. We represent the input
            // as a list of n elements (the values don't matter — the length encodes n).
            let list: Vec<Value> = (0..n).map(Value::Int).collect();
            TestCase {
                inputs: vec![Value::Tuple(list)],
                expected_output: Some(vec![Value::Int(expected)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect()
}

fn fibonacci_bench_inputs() -> Vec<Vec<Value>> {
    (0..1000)
        .map(|i| {
            let n = (i % 9) as i64;
            let list: Vec<Value> = (0..n).map(Value::Int).collect();
            vec![Value::Tuple(list)]
        })
        .collect()
}

fn fibonacci_rust(inputs: &[Value]) -> Value {
    if let Value::Tuple(elems) = &inputs[0] {
        let n = elems.len();
        if n == 0 {
            return Value::Int(0);
        }
        let (mut a, mut b) = (0i64, 1i64);
        for _ in 1..n {
            let tmp = a + b;
            a = b;
            b = tmp;
        }
        Value::Int(b)
    } else {
        Value::Int(0)
    }
}

// =========================================================================
// Problem 3: Power (x^n)
// =========================================================================

fn power_test_cases() -> Vec<TestCase> {
    let cases: Vec<(i64, i64, i64)> = vec![
        (2, 0, 1), (2, 1, 2), (2, 3, 8), (3, 2, 9), (2, 10, 1024), (5, 3, 125),
    ];
    cases
        .into_iter()
        .map(|(x, n, expected)| {
            // Represent as a list of n copies of x: fold(1, mul, [x, x, ..., x])
            let list: Vec<Value> = (0..n).map(|_| Value::Int(x)).collect();
            TestCase {
                inputs: vec![Value::Tuple(list)],
                expected_output: Some(vec![Value::Int(expected)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect()
}

fn power_bench_inputs() -> Vec<Vec<Value>> {
    (0..1000)
        .map(|i| {
            let x = (i % 5 + 1) as i64;
            let n = (i % 6) as i64;
            let list: Vec<Value> = (0..n).map(|_| Value::Int(x)).collect();
            vec![Value::Tuple(list)]
        })
        .collect()
}

fn power_rust(inputs: &[Value]) -> Value {
    if let Value::Tuple(elems) = &inputs[0] {
        if elems.is_empty() {
            return Value::Int(1);
        }
        let mut result = 1i64;
        for e in elems {
            if let Value::Int(x) = e {
                result *= x;
            }
        }
        Value::Int(result)
    } else {
        Value::Int(0)
    }
}

// =========================================================================
// Problem 4: GCD
// =========================================================================

fn gcd_test_cases() -> Vec<TestCase> {
    let cases: Vec<(i64, i64, i64)> = vec![
        (12, 8, 4), (7, 3, 1), (100, 75, 25), (17, 17, 17), (0, 5, 5), (48, 18, 6),
    ];
    cases
        .into_iter()
        .map(|(a, b, expected)| {
            TestCase {
                inputs: vec![Value::tuple(vec![Value::Int(a), Value::Int(b)])],
                expected_output: Some(vec![Value::Int(expected)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect()
}

fn gcd_bench_inputs() -> Vec<Vec<Value>> {
    (0..1000)
        .map(|i| {
            let a = ((i * 7 + 3) % 100 + 1) as i64;
            let b = ((i * 13 + 5) % 100 + 1) as i64;
            vec![Value::tuple(vec![Value::Int(a), Value::Int(b)])]
        })
        .collect()
}

fn gcd_rust(inputs: &[Value]) -> Value {
    if let Value::Tuple(elems) = &inputs[0] {
        if let (Some(Value::Int(av)), Some(Value::Int(bv))) = (elems.first(), elems.get(1)) {
            let mut a = av.abs();
            let mut b = bv.abs();
            while b != 0 {
                let t = b;
                b = a % b;
                a = t;
            }
            Value::Int(a)
        } else {
            Value::Int(0)
        }
    } else {
        Value::Int(0)
    }
}

// =========================================================================
// Problem 5: Sum of squares
// =========================================================================

fn sum_of_squares_test_cases() -> Vec<TestCase> {
    let cases: Vec<(i64, i64)> = vec![
        (1, 1), (2, 5), (3, 14), (4, 30), (5, 55), (10, 385),
    ];
    cases
        .into_iter()
        .map(|(n, expected)| {
            let list: Vec<Value> = (1..=n).map(Value::Int).collect();
            TestCase {
                inputs: vec![Value::Tuple(list)],
                expected_output: Some(vec![Value::Int(expected)]),
                initial_state: None,
                expected_state: None,
            }
        })
        .collect()
}

fn sum_of_squares_bench_inputs() -> Vec<Vec<Value>> {
    (0..1000)
        .map(|i| {
            let n = (i % 10 + 1) as i64;
            let list: Vec<Value> = (1..=n).map(Value::Int).collect();
            vec![Value::Tuple(list)]
        })
        .collect()
}

fn sum_of_squares_rust(inputs: &[Value]) -> Value {
    if let Value::Tuple(elems) = &inputs[0] {
        let result: i64 = elems
            .iter()
            .map(|e| {
                if let Value::Int(x) = e { x * x } else { 0 }
            })
            .sum();
        Value::Int(result)
    } else {
        Value::Int(0)
    }
}

// =========================================================================
// Problem 6: Dot product
// =========================================================================

fn dot_product_test_cases() -> Vec<TestCase> {
    vec![
        // [1,2,3] . [4,5,6] = 32
        TestCase {
            inputs: vec![
                Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
                Value::tuple(vec![Value::Int(4), Value::Int(5), Value::Int(6)]),
            ],
            expected_output: Some(vec![Value::Int(32)]),
            initial_state: None,
            expected_state: None,
        },
        // [1,0] . [0,1] = 0
        TestCase {
            inputs: vec![
                Value::tuple(vec![Value::Int(1), Value::Int(0)]),
                Value::tuple(vec![Value::Int(0), Value::Int(1)]),
            ],
            expected_output: Some(vec![Value::Int(0)]),
            initial_state: None,
            expected_state: None,
        },
        // [2,3] . [4,5] = 23
        TestCase {
            inputs: vec![
                Value::tuple(vec![Value::Int(2), Value::Int(3)]),
                Value::tuple(vec![Value::Int(4), Value::Int(5)]),
            ],
            expected_output: Some(vec![Value::Int(23)]),
            initial_state: None,
            expected_state: None,
        },
    ]
}

fn dot_product_bench_inputs() -> Vec<Vec<Value>> {
    (0..1000)
        .map(|i| {
            let len = (i % 5 + 2) as i64;
            let a: Vec<Value> = (0..len).map(|j| Value::Int(j + 1)).collect();
            let b: Vec<Value> = (0..len).map(|j| Value::Int(j * 2 + 1)).collect();
            vec![Value::Tuple(a), Value::Tuple(b)]
        })
        .collect()
}

fn dot_product_rust(inputs: &[Value]) -> Value {
    if let (Value::Tuple(a), Value::Tuple(b)) = (&inputs[0], &inputs[1]) {
        let result: i64 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| {
                let xi = if let Value::Int(v) = x { *v } else { 0 };
                let yi = if let Value::Int(v) = y { *v } else { 0 };
                xi * yi
            })
            .sum();
        Value::Int(result)
    } else {
        Value::Int(0)
    }
}

// =========================================================================
// Problem 7: Manhattan distance
// =========================================================================

fn manhattan_test_cases() -> Vec<TestCase> {
    vec![
        // |1-4| + |2-2| + |3-1| = 3 + 0 + 2 = 5
        TestCase {
            inputs: vec![
                Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
                Value::tuple(vec![Value::Int(4), Value::Int(2), Value::Int(1)]),
            ],
            expected_output: Some(vec![Value::Int(5)]),
            initial_state: None,
            expected_state: None,
        },
        // |0-3| + |0-4| = 7
        TestCase {
            inputs: vec![
                Value::tuple(vec![Value::Int(0), Value::Int(0)]),
                Value::tuple(vec![Value::Int(3), Value::Int(4)]),
            ],
            expected_output: Some(vec![Value::Int(7)]),
            initial_state: None,
            expected_state: None,
        },
        // |5-5| = 0
        TestCase {
            inputs: vec![
                Value::tuple(vec![Value::Int(5)]),
                Value::tuple(vec![Value::Int(5)]),
            ],
            expected_output: Some(vec![Value::Int(0)]),
            initial_state: None,
            expected_state: None,
        },
    ]
}

fn manhattan_bench_inputs() -> Vec<Vec<Value>> {
    (0..1000)
        .map(|i| {
            let len = (i % 5 + 1) as i64;
            let a: Vec<Value> = (0..len).map(|j| Value::Int(j * 3)).collect();
            let b: Vec<Value> = (0..len).map(|j| Value::Int(j * 3 + (i as i64 % 7))).collect();
            vec![Value::Tuple(a), Value::Tuple(b)]
        })
        .collect()
}

fn manhattan_rust(inputs: &[Value]) -> Value {
    if let (Value::Tuple(a), Value::Tuple(b)) = (&inputs[0], &inputs[1]) {
        let result: i64 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| {
                let xi = if let Value::Int(v) = x { *v } else { 0 };
                let yi = if let Value::Int(v) = y { *v } else { 0 };
                (xi - yi).abs()
            })
            .sum();
        Value::Int(result)
    } else {
        Value::Int(0)
    }
}

// =========================================================================
// Problem 8: Min of list
// =========================================================================

fn min_of_list_test_cases() -> Vec<TestCase> {
    vec![
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(3), Value::Int(1), Value::Int(4), Value::Int(1), Value::Int(5),
            ])],
            expected_output: Some(vec![Value::Int(1)]),
            initial_state: None,
            expected_state: None,
        },
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(10)])],
            expected_output: Some(vec![Value::Int(10)]),
            initial_state: None,
            expected_state: None,
        },
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(7), Value::Int(7), Value::Int(7),
            ])],
            expected_output: Some(vec![Value::Int(7)]),
            initial_state: None,
            expected_state: None,
        },
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(-5), Value::Int(0), Value::Int(5),
            ])],
            expected_output: Some(vec![Value::Int(-5)]),
            initial_state: None,
            expected_state: None,
        },
    ]
}

fn min_of_list_bench_inputs() -> Vec<Vec<Value>> {
    (0..1000)
        .map(|i| {
            let len = (i % 10 + 1) as i64;
            let list: Vec<Value> = (0..len)
                .map(|j| Value::Int((j * 17 + i as i64 * 3) % 100 - 50))
                .collect();
            vec![Value::Tuple(list)]
        })
        .collect()
}

fn min_of_list_rust(inputs: &[Value]) -> Value {
    if let Value::Tuple(elems) = &inputs[0] {
        let result = elems
            .iter()
            .map(|e| if let Value::Int(v) = e { *v } else { i64::MAX })
            .min()
            .unwrap_or(0);
        Value::Int(result)
    } else {
        Value::Int(0)
    }
}

// =========================================================================
// Display helpers
// =========================================================================

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

fn format_duration(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms >= 1000.0 {
        format!("{:.1}s", d.as_secs_f64())
    } else if ms >= 1.0 {
        format!("{:.0}ms", ms)
    } else {
        let us = d.as_secs_f64() * 1_000_000.0;
        format!("{:.0}us", us)
    }
}

// =========================================================================
// Main benchmark test
// =========================================================================

#[test]
fn bench_math_library() {
    println!();
    println!("========================================");
    println!("IRIS Math Library Benchmark");
    println!("========================================");
    println!();
    println!("Config: population=256/512, max_generations=2000/3000, up to 3 attempts");
    println!("Execution benchmark: 1000 inputs per problem");
    println!();

    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 1024,
        worker_threads: 4,
        ..ExecConfig::default()
    });

    let mut results: Vec<BenchResult> = Vec::new();

    // -- Problem 1: Factorial --
    println!("[1/8] Evolving Factorial...");
    results.push(run_bench(
        "Factorial",
        factorial_test_cases(),
        &exec,
        factorial_bench_inputs,
        factorial_rust,
    ));
    println!("       -> {} (correctness: {:.0}%, {} gens, {})",
        results.last().unwrap().status_str(),
        results.last().unwrap().correctness * 100.0,
        results.last().unwrap().generations_run,
        format_duration(results.last().unwrap().evolve_time),
    );

    // -- Problem 2: Fibonacci --
    println!("[2/8] Evolving Fibonacci...");
    results.push(run_bench_hard(
        "Fibonacci",
        fibonacci_test_cases(),
        &exec,
        fibonacci_bench_inputs,
        fibonacci_rust,
    ));
    println!("       -> {} (correctness: {:.0}%, {} gens, {})",
        results.last().unwrap().status_str(),
        results.last().unwrap().correctness * 100.0,
        results.last().unwrap().generations_run,
        format_duration(results.last().unwrap().evolve_time),
    );

    // -- Problem 3: Power --
    println!("[3/8] Evolving Power...");
    results.push(run_bench(
        "Power",
        power_test_cases(),
        &exec,
        power_bench_inputs,
        power_rust,
    ));
    println!("       -> {} (correctness: {:.0}%, {} gens, {})",
        results.last().unwrap().status_str(),
        results.last().unwrap().correctness * 100.0,
        results.last().unwrap().generations_run,
        format_duration(results.last().unwrap().evolve_time),
    );

    // -- Problem 4: GCD --
    println!("[4/8] Evolving GCD...");
    results.push(run_bench_hard(
        "GCD",
        gcd_test_cases(),
        &exec,
        gcd_bench_inputs,
        gcd_rust,
    ));
    println!("       -> {} (correctness: {:.0}%, {} gens, {})",
        results.last().unwrap().status_str(),
        results.last().unwrap().correctness * 100.0,
        results.last().unwrap().generations_run,
        format_duration(results.last().unwrap().evolve_time),
    );

    // -- Problem 5: Sum of squares --
    println!("[5/8] Evolving Sum of squares...");
    results.push(run_bench(
        "Sum of squares",
        sum_of_squares_test_cases(),
        &exec,
        sum_of_squares_bench_inputs,
        sum_of_squares_rust,
    ));
    println!("       -> {} (correctness: {:.0}%, {} gens, {})",
        results.last().unwrap().status_str(),
        results.last().unwrap().correctness * 100.0,
        results.last().unwrap().generations_run,
        format_duration(results.last().unwrap().evolve_time),
    );

    // -- Problem 6: Dot product --
    println!("[6/8] Evolving Dot product...");
    results.push(run_bench_hard(
        "Dot product",
        dot_product_test_cases(),
        &exec,
        dot_product_bench_inputs,
        dot_product_rust,
    ));
    println!("       -> {} (correctness: {:.0}%, {} gens, {})",
        results.last().unwrap().status_str(),
        results.last().unwrap().correctness * 100.0,
        results.last().unwrap().generations_run,
        format_duration(results.last().unwrap().evolve_time),
    );

    // -- Problem 7: Manhattan distance --
    println!("[7/8] Evolving Manhattan distance...");
    results.push(run_bench_hard(
        "Manhattan dist",
        manhattan_test_cases(),
        &exec,
        manhattan_bench_inputs,
        manhattan_rust,
    ));
    println!("       -> {} (correctness: {:.0}%, {} gens, {})",
        results.last().unwrap().status_str(),
        results.last().unwrap().correctness * 100.0,
        results.last().unwrap().generations_run,
        format_duration(results.last().unwrap().evolve_time),
    );

    // -- Problem 8: Min of list --
    println!("[8/8] Evolving Min of list...");
    results.push(run_bench(
        "Min of list",
        min_of_list_test_cases(),
        &exec,
        min_of_list_bench_inputs,
        min_of_list_rust,
    ));
    println!("       -> {} (correctness: {:.0}%, {} gens, {})",
        results.last().unwrap().status_str(),
        results.last().unwrap().correctness * 100.0,
        results.last().unwrap().generations_run,
        format_duration(results.last().unwrap().evolve_time),
    );

    // =====================================================================
    // Print summary report
    // =====================================================================

    println!();
    println!("========================================");
    println!("IRIS Math Library Benchmark Results");
    println!("========================================");
    println!();
    println!(
        "{:<16} | {:<12} | {:>4} | {:>11} | {:>14} | {:>14} | {:>6}",
        "Problem", "Evolved?", "Gens", "Evolve Time", "IRIS Exec (1K)", "Rust Exec (1K)", "Ratio"
    );
    println!(
        "{:-<16}-+-{:-<12}-+-{:-<4}-+-{:-<11}-+-{:-<14}-+-{:-<14}-+-{:-<6}",
        "", "", "", "", "", "", ""
    );

    for r in &results {
        let iris_str = match r.iris_exec_time {
            Some(d) => format_duration(d),
            None => "N/A".to_string(),
        };
        let ratio_str = match r.ratio() {
            Some(ratio) => format!("{:.0}x", ratio),
            None => "N/A".to_string(),
        };
        println!(
            "{:<16} | {:<12} | {:>4} | {:>11} | {:>14} | {:>14} | {:>6}",
            r.name,
            r.status_str(),
            r.generations_run,
            format_duration(r.evolve_time),
            iris_str,
            format_duration(r.rust_exec_time),
            ratio_str,
        );
    }

    println!();

    // Detail section: failure analysis
    let failures: Vec<&BenchResult> = results.iter().filter(|r| !r.evolved).collect();
    if !failures.is_empty() {
        println!("========================================");
        println!("Failure Analysis");
        println!("========================================");
        println!();
        for r in &failures {
            println!(
                "  {} (correctness: {:.0}%, {} nodes, {} edges)",
                r.name,
                r.correctness * 100.0,
                r.num_nodes,
                r.num_edges,
            );
            if let Some(reason) = &r.failure_reason {
                println!("    Diagnosis: {}", reason);
            }
            println!();
        }
    }

    // Summary statistics
    let num_solved = results.iter().filter(|r| r.evolved).count();
    let total_evolve_time: Duration = results.iter().map(|r| r.evolve_time).sum();

    println!("========================================");
    println!("Summary");
    println!("========================================");
    println!();
    println!("  Problems solved:   {}/{}", num_solved, results.len());
    println!("  Total evolve time: {}", format_duration(total_evolve_time));

    if num_solved > 0 {
        let ratios: Vec<f64> = results.iter().filter_map(|r| r.ratio()).collect();
        let avg_ratio: f64 = ratios.iter().sum::<f64>() / ratios.len() as f64;
        let min_ratio = ratios.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_ratio = ratios.iter().cloned().fold(0.0f64, f64::max);

        println!("  Avg IRIS/Rust ratio: {:.0}x", avg_ratio);
        println!("  Min ratio:           {:.0}x", min_ratio);
        println!("  Max ratio:           {:.0}x", max_ratio);
        println!();
        println!("  Note: IRIS uses a tree-walking interpreter. The ratio measures");
        println!("  the overhead of interpretation vs native compiled Rust code.");
        println!("  Tier B/C (CLCU hardware) would close this gap significantly.");
    }

    println!();

    // Assertions: the test passes if evolution ran without panicking.
    // We require at least *some* problems to be solvable.
    assert!(
        num_solved >= 1,
        "At least one math problem should be solvable by IRIS evolution"
    );
}
