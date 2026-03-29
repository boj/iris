//! CS Fundamentals Benchmark Suite for IRIS
//!
//! Tests data structure operations, predicate patterns, and classic CS problems
//! to establish what the evolutionary system can and cannot discover.
//!
//! Organized in three difficulty tiers:
//!   Tier 1 (Easy): fold/map/filter patterns
//!   Tier 2 (Moderate): composition of primitives
//!   Tier 3 (Hard): novel structure discovery

use std::time::Instant;

use iris_evolve::config::{EvolutionConfig, PhaseThresholds, ProblemSpec};
use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_exec::ExecutionService;
use iris_types::eval::{EvalTier, TestCase, Value};

// ---------------------------------------------------------------------------
// Test case builders
// ---------------------------------------------------------------------------

fn tc(inputs: Vec<Value>, expected: Vec<Value>) -> TestCase {
    TestCase {
        inputs,
        expected_output: Some(expected),
        initial_state: None,
        expected_state: None,
    }
}

fn int(v: i64) -> Value {
    Value::Int(v)
}

fn tup(vs: Vec<Value>) -> Value {
    Value::tuple(vs)
}

fn ints(vs: &[i64]) -> Value {
    tup(vs.iter().map(|&v| int(v)).collect())
}

// ---------------------------------------------------------------------------
// Problem definitions
// ---------------------------------------------------------------------------

struct Problem {
    name: &'static str,
    tier: u8,
    test_cases: Vec<TestCase>,
    population_size: usize,
    max_generations: usize,
}

fn tier1_problems() -> Vec<Problem> {
    vec![
        // 1. All positive: check if all elements > 0
        Problem {
            name: "All positive",
            tier: 1,
            test_cases: vec![
                tc(vec![ints(&[1, 2, 3])], vec![int(1)]),
                tc(vec![ints(&[1, -1, 2])], vec![int(0)]),
                tc(vec![ints(&[])], vec![int(1)]),
                tc(vec![ints(&[5])], vec![int(1)]),
                tc(vec![ints(&[0])], vec![int(0)]),
                tc(vec![ints(&[1, 2, 3, 4, 5])], vec![int(1)]),
            ],
            population_size: 256,
            max_generations: 2000,
        },
        // 2. Any negative: check if any element < 0
        Problem {
            name: "Any negative",
            tier: 1,
            test_cases: vec![
                tc(vec![ints(&[1, 2, 3])], vec![int(0)]),
                tc(vec![ints(&[1, -1, 2])], vec![int(1)]),
                tc(vec![ints(&[])], vec![int(0)]),
                tc(vec![ints(&[-5])], vec![int(1)]),
                tc(vec![ints(&[0, 0, 0])], vec![int(0)]),
                tc(vec![ints(&[-1, -2, -3])], vec![int(1)]),
            ],
            population_size: 256,
            max_generations: 2000,
        },
        // 3. Sum of even numbers: filter even + sum
        Problem {
            name: "Sum of evens",
            tier: 1,
            test_cases: vec![
                tc(vec![ints(&[1, 2, 3, 4, 5, 6])], vec![int(12)]),
                tc(vec![ints(&[1, 3, 5])], vec![int(0)]),
                tc(vec![ints(&[2, 4])], vec![int(6)]),
                tc(vec![ints(&[])], vec![int(0)]),
                tc(vec![ints(&[10])], vec![int(10)]),
                tc(vec![ints(&[7])], vec![int(0)]),
            ],
            population_size: 256,
            max_generations: 2000,
        },
        // 4. Count positives: count elements > 0
        Problem {
            name: "Count positives",
            tier: 1,
            test_cases: vec![
                tc(vec![ints(&[1, -2, 3, -4, 5])], vec![int(3)]),
                tc(vec![ints(&[-1, -2])], vec![int(0)]),
                tc(vec![ints(&[])], vec![int(0)]),
                tc(vec![ints(&[1, 2, 3])], vec![int(3)]),
                tc(vec![ints(&[0, 0, 0])], vec![int(0)]),
                tc(vec![ints(&[1])], vec![int(1)]),
            ],
            population_size: 256,
            max_generations: 2000,
        },
        // 5. Flatten: concat nested tuples — input is a tuple of two tuples
        Problem {
            name: "Flatten",
            tier: 1,
            test_cases: vec![
                tc(
                    vec![tup(vec![ints(&[1, 2]), ints(&[3, 4])])],
                    vec![ints(&[1, 2, 3, 4])],
                ),
                tc(
                    vec![tup(vec![ints(&[]), ints(&[1])])],
                    vec![ints(&[1])],
                ),
                tc(
                    vec![tup(vec![ints(&[5, 6, 7]), ints(&[])])],
                    vec![ints(&[5, 6, 7])],
                ),
                tc(
                    vec![tup(vec![ints(&[1]), ints(&[2])])],
                    vec![ints(&[1, 2])],
                ),
                tc(
                    vec![tup(vec![ints(&[]), ints(&[])])],
                    vec![ints(&[])],
                ),
            ],
            population_size: 256,
            max_generations: 2000,
        },
    ]
}

fn tier2_problems() -> Vec<Problem> {
    vec![
        // 6. Running sum: prefix sums
        Problem {
            name: "Running sum",
            tier: 2,
            test_cases: vec![
                tc(vec![ints(&[1, 2, 3, 4])], vec![ints(&[1, 3, 6, 10])]),
                tc(vec![ints(&[5])], vec![ints(&[5])]),
                tc(vec![ints(&[])], vec![ints(&[])]),
                tc(vec![ints(&[1, 1, 1])], vec![ints(&[1, 2, 3])]),
                tc(vec![ints(&[10, -5, 3])], vec![ints(&[10, 5, 8])]),
            ],
            population_size: 256,
            max_generations: 2000,
        },
        // 7. Pairwise differences: consecutive deltas
        Problem {
            name: "Pairwise diffs",
            tier: 2,
            test_cases: vec![
                tc(vec![ints(&[1, 3, 6, 10])], vec![ints(&[2, 3, 4])]),
                tc(vec![ints(&[5, 5, 5])], vec![ints(&[0, 0])]),
                tc(vec![ints(&[1, 2])], vec![ints(&[1])]),
                tc(vec![ints(&[10, 7, 3])], vec![ints(&[-3, -4])]),
                tc(vec![ints(&[1])], vec![ints(&[])]),
            ],
            population_size: 256,
            max_generations: 2000,
        },
        // 8. Second largest
        Problem {
            name: "Second largest",
            tier: 2,
            test_cases: vec![
                tc(vec![ints(&[3, 1, 4, 1, 5])], vec![int(4)]),
                tc(vec![ints(&[5, 5, 3])], vec![int(5)]),
                tc(vec![ints(&[1, 2])], vec![int(1)]),
                tc(vec![ints(&[10, 20, 30])], vec![int(20)]),
                tc(vec![ints(&[7, 7, 7])], vec![int(7)]),
                tc(vec![ints(&[1, 100])], vec![int(1)]),
            ],
            population_size: 256,
            max_generations: 2000,
        },
        // 9. Is sorted: check ascending order
        Problem {
            name: "Is sorted",
            tier: 2,
            test_cases: vec![
                tc(vec![ints(&[1, 2, 3])], vec![int(1)]),
                tc(vec![ints(&[3, 1, 2])], vec![int(0)]),
                tc(vec![ints(&[1])], vec![int(1)]),
                tc(vec![ints(&[])], vec![int(1)]),
                tc(vec![ints(&[1, 1, 2])], vec![int(1)]),
                tc(vec![ints(&[5, 4])], vec![int(0)]),
            ],
            population_size: 256,
            max_generations: 2000,
        },
        // 10. Weighted sum: sum(i * a[i])
        Problem {
            name: "Weighted sum",
            tier: 2,
            test_cases: vec![
                // 0*10 + 1*20 + 2*30 = 80
                tc(vec![ints(&[10, 20, 30])], vec![int(80)]),
                // 0*1 + 1*2 + 2*3 = 8
                tc(vec![ints(&[1, 2, 3])], vec![int(8)]),
                tc(vec![ints(&[5])], vec![int(0)]),
                tc(vec![ints(&[])], vec![int(0)]),
                // 0*0 + 1*1 + 2*2 + 3*3 = 14
                tc(vec![ints(&[0, 1, 2, 3])], vec![int(14)]),
            ],
            population_size: 256,
            max_generations: 2000,
        },
    ]
}

fn tier3_problems() -> Vec<Problem> {
    vec![
        // 11. Two sum: find two indices that sum to target
        //     Returns a tuple of two indices (multiple valid answers accepted)
        //     We only check one valid answer per test case.
        Problem {
            name: "Two sum",
            tier: 3,
            test_cases: vec![
                // [1,3,5,7], target=8 -> (0,3) or (1,2)
                tc(
                    vec![ints(&[1, 3, 5, 7]), int(8)],
                    vec![tup(vec![int(0), int(3)])],
                ),
                // [2,7,11,15], target=9 -> (0,1)
                tc(
                    vec![ints(&[2, 7, 11, 15]), int(9)],
                    vec![tup(vec![int(0), int(1)])],
                ),
                // [1,2,3,4], target=7 -> (2,3)
                tc(
                    vec![ints(&[1, 2, 3, 4]), int(7)],
                    vec![tup(vec![int(2), int(3)])],
                ),
            ],
            population_size: 512,
            max_generations: 3000,
        },
        // 12. Matrix multiply 2x2: flat [a,b,c,d] * flat [e,f,g,h]
        //     [[a,b],[c,d]] * [[e,f],[g,h]] = [[ae+bg, af+bh],[ce+dg, cf+dh]]
        Problem {
            name: "Matrix mul 2x2",
            tier: 3,
            test_cases: vec![
                // Identity * Identity = Identity
                // [1,0,0,1] * [1,0,0,1] = [1,0,0,1]
                tc(
                    vec![ints(&[1, 0, 0, 1]), ints(&[1, 0, 0, 1])],
                    vec![ints(&[1, 0, 0, 1])],
                ),
                // [1,2,3,4] * [1,0,0,1] = [1,2,3,4]
                tc(
                    vec![ints(&[1, 2, 3, 4]), ints(&[1, 0, 0, 1])],
                    vec![ints(&[1, 2, 3, 4])],
                ),
                // [2,0,0,2] * [3,0,0,3] = [6,0,0,6]
                tc(
                    vec![ints(&[2, 0, 0, 2]), ints(&[3, 0, 0, 3])],
                    vec![ints(&[6, 0, 0, 6])],
                ),
                // [1,2,3,4] * [5,6,7,8] = [19,22,43,50]
                tc(
                    vec![ints(&[1, 2, 3, 4]), ints(&[5, 6, 7, 8])],
                    vec![ints(&[19, 22, 43, 50])],
                ),
            ],
            population_size: 512,
            max_generations: 3000,
        },
        // 13. Polynomial evaluation: a[0] + a[1]*x + a[2]*x^2 + ...
        Problem {
            name: "Poly eval",
            tier: 3,
            test_cases: vec![
                // [1, 2, 3], x=2 -> 1 + 2*2 + 3*4 = 17
                tc(vec![ints(&[1, 2, 3]), int(2)], vec![int(17)]),
                // [5], x=10 -> 5
                tc(vec![ints(&[5]), int(10)], vec![int(5)]),
                // [0, 1], x=7 -> 7
                tc(vec![ints(&[0, 1]), int(7)], vec![int(7)]),
                // [1, 0, 1], x=3 -> 1 + 0 + 9 = 10
                tc(vec![ints(&[1, 0, 1]), int(3)], vec![int(10)]),
                // [2, 3, 1], x=1 -> 2 + 3 + 1 = 6
                tc(vec![ints(&[2, 3, 1]), int(1)], vec![int(6)]),
            ],
            population_size: 512,
            max_generations: 3000,
        },
        // 14. Moving average: window-3 average (integer division)
        Problem {
            name: "Moving avg w3",
            tier: 3,
            test_cases: vec![
                // [1,2,3,4,5] -> [(1+2+3)/3, (2+3+4)/3, (3+4+5)/3] = [2,3,4]
                tc(vec![ints(&[1, 2, 3, 4, 5])], vec![ints(&[2, 3, 4])]),
                // [3,3,3,3] -> [3, 3]
                tc(vec![ints(&[3, 3, 3, 3])], vec![ints(&[3, 3])]),
                // [10,20,30] -> [20]
                tc(vec![ints(&[10, 20, 30])], vec![ints(&[20])]),
                // [6,12,18,24] -> [12, 18]
                tc(vec![ints(&[6, 12, 18, 24])], vec![ints(&[12, 18])]),
            ],
            population_size: 512,
            max_generations: 3000,
        },
    ]
}

// ---------------------------------------------------------------------------
// Rust baselines for benchmarking
// ---------------------------------------------------------------------------

fn rust_all_positive(v: &[i64]) -> i64 {
    if v.iter().all(|&x| x > 0) { 1 } else { 0 }
}

fn rust_any_negative(v: &[i64]) -> i64 {
    if v.iter().any(|&x| x < 0) { 1 } else { 0 }
}

fn rust_sum_evens(v: &[i64]) -> i64 {
    v.iter().filter(|&&x| x % 2 == 0).sum()
}

fn rust_count_positives(v: &[i64]) -> i64 {
    v.iter().filter(|&&x| x > 0).count() as i64
}

fn rust_flatten(a: &[i64], b: &[i64]) -> Vec<i64> {
    let mut r = a.to_vec();
    r.extend_from_slice(b);
    r
}

fn rust_running_sum(v: &[i64]) -> Vec<i64> {
    let mut acc = 0i64;
    v.iter()
        .map(|&x| {
            acc += x;
            acc
        })
        .collect()
}

fn rust_pairwise_diffs(v: &[i64]) -> Vec<i64> {
    v.windows(2).map(|w| w[1] - w[0]).collect()
}

fn rust_second_largest(v: &[i64]) -> i64 {
    let mut sorted = v.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() - 2]
}

fn rust_is_sorted(v: &[i64]) -> i64 {
    if v.windows(2).all(|w| w[0] <= w[1]) { 1 } else { 0 }
}

fn rust_weighted_sum(v: &[i64]) -> i64 {
    v.iter().enumerate().map(|(i, &x)| i as i64 * x).sum()
}

fn rust_two_sum(v: &[i64], target: i64) -> (i64, i64) {
    for i in 0..v.len() {
        for j in (i + 1)..v.len() {
            if v[i] + v[j] == target {
                return (i as i64, j as i64);
            }
        }
    }
    (-1, -1)
}

fn rust_matmul_2x2(a: &[i64], b: &[i64]) -> Vec<i64> {
    vec![
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
    ]
}

fn rust_poly_eval(coeffs: &[i64], x: i64) -> i64 {
    let mut result = 0i64;
    let mut power = 1i64;
    for &c in coeffs {
        result += c * power;
        power *= x;
    }
    result
}

fn rust_moving_avg_3(v: &[i64]) -> Vec<i64> {
    v.windows(3).map(|w| (w[0] + w[1] + w[2]) / 3).collect()
}

// ---------------------------------------------------------------------------
// Result tracking
// ---------------------------------------------------------------------------

struct ProblemResult {
    name: &'static str,
    tier: u8,
    solved: bool,
    best_correctness: f32,
    generations_used: usize,
    evolve_time_ms: u64,
    iris_1k_us: Option<u64>,
    rust_1k_us: Option<u64>,
}

// ---------------------------------------------------------------------------
// Evolution runner
// ---------------------------------------------------------------------------

const MAX_ATTEMPTS: usize = 3;

fn run_evolution(
    problem: &Problem,
    exec: &IrisExecutionService,
) -> (bool, f32, usize, u64) {
    let config = EvolutionConfig {
        population_size: problem.population_size,
        max_generations: problem.max_generations,
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

    let mut best_correctness = -1.0f32;
    let mut total_elapsed_ms = 0u64;
    let mut total_gens = 0usize;

    for attempt in 0..MAX_ATTEMPTS {
        let spec = ProblemSpec {
            test_cases: problem.test_cases.clone(),
            description: problem.name.to_string(),
            target_cost: None,
        };

        let start = Instant::now();
        let result = iris_evolve::evolve(config.clone(), spec, exec);
        total_elapsed_ms += start.elapsed().as_millis() as u64;
        total_gens += result.generations_run;

        let correctness = result.best_individual.fitness.correctness();
        if correctness > best_correctness {
            best_correctness = correctness;
        }

        if best_correctness >= 1.0 {
            if attempt > 0 {
                println!("    (solved on attempt {})", attempt + 1);
            }
            break;
        }
    }

    let solved = best_correctness >= 1.0;
    (solved, best_correctness, total_gens, total_elapsed_ms)
}

// ---------------------------------------------------------------------------
// IRIS execution timing (1000 runs, median of 3)
// ---------------------------------------------------------------------------

fn time_iris_1k(
    exec: &IrisExecutionService,
    problem: &Problem,
) -> Option<u64> {
    let spec = ProblemSpec {
        test_cases: problem.test_cases.clone(),
        description: problem.name.to_string(),
        target_cost: None,
    };

    let config = EvolutionConfig {
        population_size: problem.population_size,
        max_generations: problem.max_generations,
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

    // Re-evolve to get a solved program.
    let result = iris_evolve::evolve(config, spec, exec);
    let best = &result.best_individual;
    if best.fitness.correctness() < 1.0 {
        return None;
    }

    let graph = &best.fragment.graph;
    let test_cases = &problem.test_cases;

    // 3 rounds of 1000 evaluations each, take median.
    let mut timings = Vec::with_capacity(3);
    for _ in 0..3 {
        let start = Instant::now();
        for _ in 0..1000 {
            for tc in test_cases {
                let _ = exec.evaluate_individual(graph, &[tc.clone()], EvalTier::A);
            }
        }
        timings.push(start.elapsed().as_micros() as u64);
    }
    timings.sort();
    Some(timings[1]) // median
}

// ---------------------------------------------------------------------------
// Rust baseline timing (1000 runs, median of 3)
// ---------------------------------------------------------------------------

fn time_rust_1k_us(problem_name: &str, test_cases: &[TestCase]) -> u64 {
    // Extract raw i64 vectors from test cases for Rust baselines.
    let inputs: Vec<Vec<i64>> = test_cases
        .iter()
        .map(|tc| match &tc.inputs[0] {
            Value::Tuple(elems) => elems
                .iter()
                .filter_map(|v| match v {
                    Value::Int(n) => Some(*n),
                    Value::Tuple(_) => {
                        // For flatten, the inner tuples are sub-lists
                        // We'll handle this per-problem
                        None
                    }
                    _ => None,
                })
                .collect(),
            Value::Int(n) => vec![*n],
            _ => vec![],
        })
        .collect();

    let mut timings = Vec::with_capacity(3);
    for _ in 0..3 {
        let start = Instant::now();
        for _ in 0..1000 {
            match problem_name {
                "All positive" => {
                    for inp in &inputs {
                        std::hint::black_box(rust_all_positive(inp));
                    }
                }
                "Any negative" => {
                    for inp in &inputs {
                        std::hint::black_box(rust_any_negative(inp));
                    }
                }
                "Sum of evens" => {
                    for inp in &inputs {
                        std::hint::black_box(rust_sum_evens(inp));
                    }
                }
                "Count positives" => {
                    for inp in &inputs {
                        std::hint::black_box(rust_count_positives(inp));
                    }
                }
                "Flatten" => {
                    for tc in test_cases {
                        if let Value::Tuple(parts) = &tc.inputs[0] {
                            let a: Vec<i64> = extract_ints(&parts[0]);
                            let b: Vec<i64> = extract_ints(&parts[1]);
                            std::hint::black_box(rust_flatten(&a, &b));
                        }
                    }
                }
                "Running sum" => {
                    for inp in &inputs {
                        std::hint::black_box(rust_running_sum(inp));
                    }
                }
                "Pairwise diffs" => {
                    for inp in &inputs {
                        std::hint::black_box(rust_pairwise_diffs(inp));
                    }
                }
                "Second largest" => {
                    for inp in &inputs {
                        if inp.len() >= 2 {
                            std::hint::black_box(rust_second_largest(inp));
                        }
                    }
                }
                "Is sorted" => {
                    for inp in &inputs {
                        std::hint::black_box(rust_is_sorted(inp));
                    }
                }
                "Weighted sum" => {
                    for inp in &inputs {
                        std::hint::black_box(rust_weighted_sum(inp));
                    }
                }
                "Two sum" => {
                    for tc in test_cases {
                        let arr = extract_ints(&tc.inputs[0]);
                        let target = match &tc.inputs[1] {
                            Value::Int(n) => *n,
                            _ => 0,
                        };
                        std::hint::black_box(rust_two_sum(&arr, target));
                    }
                }
                "Matrix mul 2x2" => {
                    for tc in test_cases {
                        let a = extract_ints(&tc.inputs[0]);
                        let b = extract_ints(&tc.inputs[1]);
                        std::hint::black_box(rust_matmul_2x2(&a, &b));
                    }
                }
                "Poly eval" => {
                    for tc in test_cases {
                        let coeffs = extract_ints(&tc.inputs[0]);
                        let x = match &tc.inputs[1] {
                            Value::Int(n) => *n,
                            _ => 0,
                        };
                        std::hint::black_box(rust_poly_eval(&coeffs, x));
                    }
                }
                "Moving avg w3" => {
                    for inp in &inputs {
                        std::hint::black_box(rust_moving_avg_3(inp));
                    }
                }
                _ => {}
            }
        }
        timings.push(start.elapsed().as_micros() as u64);
    }
    timings.sort();
    timings[1] // median
}

fn extract_ints(v: &Value) -> Vec<i64> {
    match v {
        Value::Tuple(elems) => elems
            .iter()
            .filter_map(|e| match e {
                Value::Int(n) => Some(*n),
                _ => None,
            })
            .collect(),
        Value::Int(n) => vec![*n],
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// Main test
// ---------------------------------------------------------------------------

#[test]
fn bench_cs_fundamentals() {
    let total_start = Instant::now();

    println!();
    println!("========================================");
    println!("IRIS CS Fundamentals Benchmark");
    println!("========================================");
    println!();

    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 1024,
        worker_threads: 4,
        ..ExecConfig::default()
    });

    let mut results: Vec<ProblemResult> = Vec::new();

    // Collect all problems.
    let t1 = tier1_problems();
    let t2 = tier2_problems();
    let t3 = tier3_problems();

    let all_problems: Vec<&Problem> = t1.iter().chain(t2.iter()).chain(t3.iter()).collect();

    for problem in &all_problems {
        println!(
            "--- Problem: {} (Tier {}) ---",
            problem.name, problem.tier
        );
        println!(
            "    Pop: {}, Max gens: {}, Test cases: {}",
            problem.population_size,
            problem.max_generations,
            problem.test_cases.len()
        );

        // Run evolution.
        let (solved, best_corr, gens, evolve_ms) = run_evolution(problem, &exec);

        println!(
            "    Solved: {}  Best correctness: {:.1}%  Gens: {}  Time: {}ms",
            if solved { "YES" } else { "NO" },
            best_corr * 100.0,
            gens,
            evolve_ms
        );

        // Time Rust baseline.
        let rust_us = time_rust_1k_us(problem.name, &problem.test_cases);
        println!("    Rust 1K baseline: {} us", rust_us);

        // If solved, time IRIS execution.
        let iris_us = if solved {
            let t = time_iris_1k(&exec, problem);
            if let Some(us) = t {
                println!("    IRIS 1K exec: {} us", us);
            } else {
                println!("    IRIS 1K exec: (re-evolution failed to solve)");
            }
            t
        } else {
            println!("    IRIS 1K exec: (not solved, skipping)");
            None
        };

        results.push(ProblemResult {
            name: problem.name,
            tier: problem.tier,
            solved,
            best_correctness: best_corr,
            generations_used: gens,
            evolve_time_ms: evolve_ms,
            iris_1k_us: iris_us,
            rust_1k_us: Some(rust_us),
        });

        println!();
    }

    // ---------------------------------------------------------------------------
    // Summary report
    // ---------------------------------------------------------------------------

    let t1_solved = results.iter().filter(|r| r.tier == 1 && r.solved).count();
    let t2_solved = results.iter().filter(|r| r.tier == 2 && r.solved).count();
    let t3_solved = results.iter().filter(|r| r.tier == 3 && r.solved).count();
    let total_solved = t1_solved + t2_solved + t3_solved;

    println!("========================================");
    println!("IRIS CS Fundamentals Benchmark");
    println!("========================================");
    println!();
    println!("Tier 1 (Easy):     {}/5 solved", t1_solved);
    println!("Tier 2 (Moderate): {}/5 solved", t2_solved);
    println!("Tier 3 (Hard):     {}/4 solved", t3_solved);
    println!();
    println!("Overall: {}/14 problems solved", total_solved);
    println!();

    // Table header.
    println!(
        "{:<22}| {:<4} | {:<6} | {:<6} | {:<5} | {:<10} | {:<11} | {:<11} | {:<8}",
        "Problem", "Tier", "Solved", "Best%", "Gens", "Evolve ms", "IRIS 1K us", "Rust 1K us", "Slowdown"
    );
    println!(
        "{:-<22}|{:-<6}|{:-<8}|{:-<8}|{:-<7}|{:-<12}|{:-<13}|{:-<13}|{:-<8}",
        "", "", "", "", "", "", "", "", ""
    );

    let mut slowdowns: Vec<f64> = Vec::new();

    for r in &results {
        let solved_str = if r.solved { "YES" } else { "NO" };
        let best_pct = format!("{:.1}%", r.best_correctness * 100.0);
        let iris_str = r
            .iris_1k_us
            .map(|v| format!("{}", v))
            .unwrap_or_else(|| "-".to_string());
        let rust_str = r
            .rust_1k_us
            .map(|v| format!("{}", v))
            .unwrap_or_else(|| "-".to_string());
        let slowdown_str = match (r.iris_1k_us, r.rust_1k_us) {
            (Some(iris), Some(rust)) if rust > 0 => {
                let s = iris as f64 / rust as f64;
                slowdowns.push(s);
                format!("{:.0}x", s)
            }
            _ => "-".to_string(),
        };

        println!(
            "{:<22}| {:<4} | {:<6} | {:<6} | {:<5} | {:<10} | {:<11} | {:<11} | {:<8}",
            r.name,
            r.tier,
            solved_str,
            best_pct,
            r.generations_used,
            r.evolve_time_ms,
            iris_str,
            rust_str,
            slowdown_str
        );
    }

    println!();

    // Execution speed summary.
    if !slowdowns.is_empty() {
        slowdowns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_idx = slowdowns.len() / 2;
        println!("Execution speed summary:");
        println!("  Median slowdown vs Rust: {:.0}x", slowdowns[median_idx]);
        println!("  Best case slowdown:  {:.0}x", slowdowns[0]);
        println!(
            "  Worst case slowdown: {:.0}x",
            slowdowns[slowdowns.len() - 1]
        );
        println!();
        println!("  Note: IRIS uses a tree-walking interpreter. The CLCU compiled path");
        println!("  (not benchmarked here) is expected to be 10-100x faster.");
    } else {
        println!("Execution speed summary: No problems solved, no timing data.");
    }

    println!();
    println!("Total benchmark time: {:.2?}", total_start.elapsed());
    println!();

    // Assertions: the test always passes (benchmarks are observational),
    // but we verify the pipeline didn't panic.
    assert!(
        results.len() == 14,
        "Expected 14 problems, got {}",
        results.len()
    );
}
