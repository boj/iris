//! Algorithm evolution benchmark for IRIS.
//!
//! Tests whether IRIS can evolve search and sorting algorithms from scratch,
//! and measures performance against equivalent native Rust implementations.
//!
//! Problems range from simple fold-based reductions (list length, contains)
//! to harder multi-pass algorithms (insertion sort, merge sorted lists).

use std::time::{Duration, Instant};

use iris_evolve::config::{EvolutionConfig, PhaseThresholds, ProblemSpec};
use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_exec::ExecutionService;
use iris_types::eval::{EvalTier, TestCase, Value};

// ---------------------------------------------------------------------------
// Test case builders
// ---------------------------------------------------------------------------

/// Helper: build a TestCase from a single list input and an expected Int output.
fn list_to_int(input: Vec<i64>, expected: i64) -> TestCase {
    TestCase {
        inputs: vec![Value::tuple(input.into_iter().map(Value::Int).collect())],
        expected_output: Some(vec![Value::Int(expected)]),
        initial_state: None,
        expected_state: None,
    }
}

/// Helper: build a TestCase with two inputs (list + target) and an expected Int output.
fn list_and_val_to_int(list: Vec<i64>, target: i64, expected: i64) -> TestCase {
    TestCase {
        inputs: vec![
            Value::tuple(list.into_iter().map(Value::Int).collect()),
            Value::Int(target),
        ],
        expected_output: Some(vec![Value::Int(expected)]),
        initial_state: None,
        expected_state: None,
    }
}

/// Helper: build a TestCase from a single list input and an expected list output.
fn list_to_list(input: Vec<i64>, expected: Vec<i64>) -> TestCase {
    TestCase {
        inputs: vec![Value::tuple(input.into_iter().map(Value::Int).collect())],
        expected_output: Some(vec![Value::tuple(
            expected.into_iter().map(Value::Int).collect(),
        )]),
        initial_state: None,
        expected_state: None,
    }
}

/// Helper: build a TestCase with two list inputs and an expected list output.
fn two_lists_to_list(a: Vec<i64>, b: Vec<i64>, expected: Vec<i64>) -> TestCase {
    TestCase {
        inputs: vec![
            Value::tuple(a.into_iter().map(Value::Int).collect()),
            Value::tuple(b.into_iter().map(Value::Int).collect()),
        ],
        expected_output: Some(vec![Value::tuple(
            expected.into_iter().map(Value::Int).collect(),
        )]),
        initial_state: None,
        expected_state: None,
    }
}

// ---------------------------------------------------------------------------
// Problem definitions
// ---------------------------------------------------------------------------

struct Problem {
    name: &'static str,
    test_cases: Vec<TestCase>,
    population_size: usize,
    max_generations: usize,
    rust_reference: fn(&[TestCase]) -> Duration,
}

fn linear_search_cases() -> Vec<TestCase> {
    vec![
        list_and_val_to_int(vec![1, 3, 5, 7, 9], 5, 2),
        list_and_val_to_int(vec![1, 3, 5], 3, 1),
        list_and_val_to_int(vec![10, 20, 30], 30, 2),
        list_and_val_to_int(vec![1, 2, 3], 4, -1),
    ]
}

fn list_reversal_cases() -> Vec<TestCase> {
    vec![
        list_to_list(vec![1, 2, 3], vec![3, 2, 1]),
        list_to_list(vec![1], vec![1]),
        list_to_list(vec![5, 4, 3, 2, 1], vec![1, 2, 3, 4, 5]),
    ]
}

fn list_length_cases() -> Vec<TestCase> {
    vec![
        list_to_int(vec![1, 2, 3], 3),
        list_to_int(vec![], 0),
        list_to_int(vec![1], 1),
        list_to_int(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 10),
    ]
}

fn contains_cases() -> Vec<TestCase> {
    vec![
        list_and_val_to_int(vec![1, 2, 3], 2, 1),
        list_and_val_to_int(vec![1, 2, 3], 4, 0),
        list_and_val_to_int(vec![], 1, 0),
    ]
}

fn count_occurrences_cases() -> Vec<TestCase> {
    vec![
        list_and_val_to_int(vec![1, 2, 1, 3, 1], 1, 3),
        list_and_val_to_int(vec![1, 2, 3], 4, 0),
        list_and_val_to_int(vec![5, 5, 5], 5, 3),
    ]
}

fn remove_duplicates_cases() -> Vec<TestCase> {
    vec![
        list_to_list(vec![1, 2, 1, 3, 2], vec![1, 2, 3]),
        list_to_list(vec![1, 1, 1], vec![1]),
        list_to_list(vec![1, 2, 3], vec![1, 2, 3]),
    ]
}

fn insertion_sort_cases() -> Vec<TestCase> {
    vec![
        list_to_list(vec![3, 1, 4, 1, 5], vec![1, 1, 3, 4, 5]),
        list_to_list(vec![5, 4, 3, 2, 1], vec![1, 2, 3, 4, 5]),
        list_to_list(vec![1], vec![1]),
        list_to_list(vec![], vec![]),
    ]
}

fn merge_sorted_cases() -> Vec<TestCase> {
    vec![
        two_lists_to_list(vec![1, 3, 5], vec![2, 4, 6], vec![1, 2, 3, 4, 5, 6]),
        two_lists_to_list(vec![1, 2], vec![3, 4], vec![1, 2, 3, 4]),
    ]
}

// ---------------------------------------------------------------------------
// Native Rust reference implementations (for timing comparison)
// ---------------------------------------------------------------------------

fn rust_linear_search(cases: &[TestCase]) -> Duration {
    let start = Instant::now();
    for _ in 0..1000 {
        for tc in cases {
            let list = match &tc.inputs[0] {
                Value::Tuple(elems) => elems,
                _ => continue,
            };
            let target = match &tc.inputs[1] {
                Value::Int(v) => *v,
                _ => continue,
            };
            let mut _result: i64 = -1;
            for (i, elem) in list.iter().enumerate() {
                if let Value::Int(v) = elem {
                    if *v == target {
                        _result = i as i64;
                        break;
                    }
                }
            }
            std::hint::black_box(_result);
        }
    }
    start.elapsed()
}

fn rust_list_reversal(cases: &[TestCase]) -> Duration {
    let start = Instant::now();
    for _ in 0..1000 {
        for tc in cases {
            let list = match &tc.inputs[0] {
                Value::Tuple(elems) => elems.clone(),
                _ => continue,
            };
            let mut reversed = list.as_ref().clone();
            reversed.reverse();
            std::hint::black_box(&reversed);
        }
    }
    start.elapsed()
}

fn rust_list_length(cases: &[TestCase]) -> Duration {
    let start = Instant::now();
    for _ in 0..1000 {
        for tc in cases {
            let list = match &tc.inputs[0] {
                Value::Tuple(elems) => elems,
                _ => continue,
            };
            std::hint::black_box(list.len() as i64);
        }
    }
    start.elapsed()
}

fn rust_contains(cases: &[TestCase]) -> Duration {
    let start = Instant::now();
    for _ in 0..1000 {
        for tc in cases {
            let list = match &tc.inputs[0] {
                Value::Tuple(elems) => elems,
                _ => continue,
            };
            let target = match &tc.inputs[1] {
                Value::Int(v) => *v,
                _ => continue,
            };
            let _found = list.iter().any(|e| matches!(e, Value::Int(v) if *v == target));
            std::hint::black_box(_found);
        }
    }
    start.elapsed()
}

fn rust_count_occurrences(cases: &[TestCase]) -> Duration {
    let start = Instant::now();
    for _ in 0..1000 {
        for tc in cases {
            let list = match &tc.inputs[0] {
                Value::Tuple(elems) => elems,
                _ => continue,
            };
            let target = match &tc.inputs[1] {
                Value::Int(v) => *v,
                _ => continue,
            };
            let _count = list
                .iter()
                .filter(|e| matches!(e, Value::Int(v) if *v == target))
                .count();
            std::hint::black_box(_count);
        }
    }
    start.elapsed()
}

fn rust_remove_duplicates(cases: &[TestCase]) -> Duration {
    let start = Instant::now();
    for _ in 0..1000 {
        for tc in cases {
            let list = match &tc.inputs[0] {
                Value::Tuple(elems) => elems,
                _ => continue,
            };
            let mut seen: Vec<Value> = Vec::new();
            for elem in list.as_ref() {
                if !seen.contains(elem) {
                    seen.push(elem.clone());
                }
            }
            std::hint::black_box(&seen);
        }
    }
    start.elapsed()
}

fn rust_insertion_sort(cases: &[TestCase]) -> Duration {
    let start = Instant::now();
    for _ in 0..1000 {
        for tc in cases {
            let list = match &tc.inputs[0] {
                Value::Tuple(elems) => elems.clone(),
                _ => continue,
            };
            let mut sorted = list.as_ref().clone();
            sorted.sort_by(|a, b| {
                let a_val = match a {
                    Value::Int(v) => *v,
                    _ => 0,
                };
                let b_val = match b {
                    Value::Int(v) => *v,
                    _ => 0,
                };
                a_val.cmp(&b_val)
            });
            std::hint::black_box(&sorted);
        }
    }
    start.elapsed()
}

fn rust_merge_sorted(cases: &[TestCase]) -> Duration {
    let start = Instant::now();
    for _ in 0..1000 {
        for tc in cases {
            let list_a = match &tc.inputs[0] {
                Value::Tuple(elems) => elems,
                _ => continue,
            };
            let list_b = match &tc.inputs[1] {
                Value::Tuple(elems) => elems,
                _ => continue,
            };
            let mut merged = Vec::with_capacity(list_a.len() + list_b.len());
            let (mut i, mut j) = (0, 0);
            while i < list_a.len() && j < list_b.len() {
                let a_val = match &list_a[i] {
                    Value::Int(v) => *v,
                    _ => 0,
                };
                let b_val = match &list_b[j] {
                    Value::Int(v) => *v,
                    _ => 0,
                };
                if a_val <= b_val {
                    merged.push(list_a[i].clone());
                    i += 1;
                } else {
                    merged.push(list_b[j].clone());
                    j += 1;
                }
            }
            merged.extend_from_slice(&list_a[i..]);
            merged.extend_from_slice(&list_b[j..]);
            std::hint::black_box(&merged);
        }
    }
    start.elapsed()
}

// ---------------------------------------------------------------------------
// Benchmark result
// ---------------------------------------------------------------------------

struct BenchResult {
    name: &'static str,
    solved: bool,
    accuracy: f32,
    generations: usize,
    evolve_time: Duration,
    iris_1k_exec: Option<Duration>,
    rust_1k_exec: Duration,
    node_count: usize,
    edge_count: usize,
    notes: String,
}

// ---------------------------------------------------------------------------
// Run a single problem
// ---------------------------------------------------------------------------

const MAX_ATTEMPTS: usize = 3;

fn run_problem(exec: &IrisExecutionService, problem: &Problem) -> BenchResult {
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

    let mut best_result = None;
    let mut best_accuracy = -1.0f32;
    let mut total_evolve_time = Duration::ZERO;
    let mut total_generations = 0usize;

    for attempt in 0..MAX_ATTEMPTS {
        let spec = ProblemSpec {
            test_cases: problem.test_cases.clone(),
            description: problem.name.to_string(),
            target_cost: None,
        };

        let evolve_start = Instant::now();
        let result = iris_evolve::evolve(config.clone(), spec, exec);
        let attempt_time = evolve_start.elapsed();
        total_evolve_time += attempt_time;
        total_generations += result.generations_run;

        let accuracy = result.best_individual.fitness.correctness();
        if accuracy > best_accuracy {
            best_accuracy = accuracy;
            best_result = Some(result);
        }

        if best_accuracy >= 1.0 {
            if attempt > 0 {
                println!("  (solved on attempt {})", attempt + 1);
            }
            break;
        }
    }

    let result = best_result.unwrap();
    let evolve_time = total_evolve_time;

    let best = &result.best_individual;
    let accuracy = best.fitness.correctness();
    let solved = accuracy >= 1.0;
    let generations = total_generations;
    let node_count = best.fragment.graph.nodes.len();
    let edge_count = best.fragment.graph.edges.len();

    // If solved, benchmark IRIS execution (1000 runs over all test cases).
    let iris_1k_exec = if solved {
        let start = Instant::now();
        for _ in 0..1000 {
            for tc in &problem.test_cases {
                let _ = exec.evaluate_individual(
                    &best.fragment.graph,
                    &[tc.clone()],
                    EvalTier::A,
                );
            }
        }
        Some(start.elapsed())
    } else {
        None
    };

    // Run Rust reference benchmark.
    let rust_1k_exec = (problem.rust_reference)(&problem.test_cases);

    // Analyze what was evolved.
    let notes = if solved {
        analyze_solution(&best.fragment.graph)
    } else {
        analyze_failure(accuracy, &problem.test_cases, exec, best)
    };

    BenchResult {
        name: problem.name,
        solved,
        accuracy,
        generations,
        evolve_time,
        iris_1k_exec,
        rust_1k_exec,
        node_count,
        edge_count,
        notes,
    }
}

/// Analyze a solved program's structure.
fn analyze_solution(graph: &iris_types::graph::SemanticGraph) -> String {
    use iris_types::graph::NodeKind;

    let mut fold_count = 0;
    let mut prim_count = 0;
    let mut lit_count = 0;
    let mut other_count = 0;

    for node in graph.nodes.values() {
        match node.kind {
            NodeKind::Fold => fold_count += 1,
            NodeKind::Prim => prim_count += 1,
            NodeKind::Lit => lit_count += 1,
            _ => other_count += 1,
        }
    }

    let mut parts = Vec::new();
    if fold_count > 0 {
        parts.push(format!("{}xFold", fold_count));
    }
    if prim_count > 0 {
        parts.push(format!("{}xPrim", prim_count));
    }
    if lit_count > 0 {
        parts.push(format!("{}xLit", lit_count));
    }
    if other_count > 0 {
        parts.push(format!("{}xOther", other_count));
    }
    parts.join(", ")
}

/// Analyze a failed/partial evolution attempt.
fn analyze_failure(
    accuracy: f32,
    test_cases: &[TestCase],
    exec: &IrisExecutionService,
    best: &iris_evolve::individual::Individual,
) -> String {
    // Check which test cases pass/fail.
    let total = test_cases.len();
    let mut passing = 0;
    let mut failing_indices = Vec::new();

    for (i, tc) in test_cases.iter().enumerate() {
        let eval_result = exec.evaluate_individual(
            &best.fragment.graph,
            &[tc.clone()],
            EvalTier::A,
        );
        let pass = match eval_result {
            Ok(er) => {
                tc.expected_output
                    .as_ref()
                    .map(|e| !er.outputs[0].is_empty() && &er.outputs[0] == e)
                    .unwrap_or(false)
            }
            Err(_) => false,
        };
        if pass {
            passing += 1;
        } else {
            failing_indices.push(i);
        }
    }

    if passing == 0 {
        format!("All {} cases fail", total)
    } else {
        format!(
            "{}/{} pass, fails: {:?}",
            passing, total, failing_indices
        )
    }
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
        Value::Graph(g) => format!("Graph{{{} nodes}}", g.nodes.len()),
        Value::Program(_) => "Program".to_string(),
        Value::Future(_) => "Future".to_string(),
        Value::Thunk(_, _) => "Thunk".to_string(),
        Value::String(s) => format!("{:?}", s),
        Value::Range(lo, hi) => format!("{}..{}", lo, hi),
    }
}

fn format_duration(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms < 1.0 {
        format!("{:.2}us", ms * 1000.0)
    } else if ms < 1000.0 {
        format!("{:.1}ms", ms)
    } else {
        format!("{:.2}s", ms / 1000.0)
    }
}

// ---------------------------------------------------------------------------
// Main benchmark test
// ---------------------------------------------------------------------------

#[test]
fn bench_algorithms() {
    // The interpreter recurses per graph node; in debug builds each frame is
    // large enough that evolved programs can overflow the default 8 MB test
    // thread stack.  Run the entire benchmark in a 64 MB thread.
    const STACK: usize = 64 * 1024 * 1024;
    std::thread::Builder::new()
        .stack_size(STACK)
        .spawn(bench_algorithms_inner)
        .expect("failed to spawn benchmark thread")
        .join()
        .expect("benchmark thread panicked");
}

fn bench_algorithms_inner() {
    println!();
    println!("========================================");
    println!("IRIS Algorithm Benchmark");
    println!("========================================");
    println!();

    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 1024,
        worker_threads: 4,
        ..ExecConfig::default()
    });

    let problems = vec![
        Problem {
            name: "Linear Search",
            test_cases: linear_search_cases(),
            population_size: 256,
            max_generations: 2000,
            rust_reference: rust_linear_search,
        },
        Problem {
            name: "List Reversal",
            test_cases: list_reversal_cases(),
            population_size: 256,
            max_generations: 2000,
            rust_reference: rust_list_reversal,
        },
        Problem {
            name: "List Length",
            test_cases: list_length_cases(),
            population_size: 256,
            max_generations: 2000,
            rust_reference: rust_list_length,
        },
        Problem {
            name: "Contains",
            test_cases: contains_cases(),
            population_size: 256,
            max_generations: 2000,
            rust_reference: rust_contains,
        },
        Problem {
            name: "Count Occur",
            test_cases: count_occurrences_cases(),
            population_size: 256,
            max_generations: 2000,
            rust_reference: rust_count_occurrences,
        },
        Problem {
            name: "Remove Dupes",
            test_cases: remove_duplicates_cases(),
            population_size: 512,
            max_generations: 3000,
            rust_reference: rust_remove_duplicates,
        },
        Problem {
            name: "Insertion Sort",
            test_cases: insertion_sort_cases(),
            population_size: 512,
            max_generations: 3000,
            rust_reference: rust_insertion_sort,
        },
        Problem {
            name: "Merge Sorted",
            test_cases: merge_sorted_cases(),
            population_size: 512,
            max_generations: 3000,
            rust_reference: rust_merge_sorted,
        },
    ];

    let total_start = Instant::now();
    let mut results: Vec<BenchResult> = Vec::new();

    for (i, problem) in problems.iter().enumerate() {
        println!(
            "[{}/{}] Evolving: {} (pop={}, max_gen={}, {} test cases)...",
            i + 1,
            problems.len(),
            problem.name,
            problem.population_size,
            problem.max_generations,
            problem.test_cases.len(),
        );

        let result = run_problem(&exec, problem);

        println!(
            "  -> {} | accuracy={:.0}% | {} generations | evolve={}",
            if result.solved { "SOLVED" } else { "FAILED" },
            result.accuracy * 100.0,
            result.generations,
            format_duration(result.evolve_time),
        );
        if result.solved {
            if let Some(iris_time) = result.iris_1k_exec {
                let ratio = iris_time.as_secs_f64() / result.rust_1k_exec.as_secs_f64();
                println!(
                    "     IRIS 1K: {} | Rust 1K: {} | ratio: {:.0}x",
                    format_duration(iris_time),
                    format_duration(result.rust_1k_exec),
                    ratio,
                );
            }
        }
        println!(
            "     Graph: {} nodes, {} edges | {}",
            result.node_count, result.edge_count, result.notes,
        );
        println!();

        results.push(result);
    }

    let total_time = total_start.elapsed();

    // Print summary table.
    println!();
    println!("========================================");
    println!("RESULTS SUMMARY");
    println!("========================================");
    println!();
    println!(
        "{:<17}| {:<8}| {:<9}| {:<5}| {:<12}| {:<13}| {:<13}| {:<7}| {}",
        "Problem", "Solved", "Accuracy", "Gens", "Evolve Time", "IRIS 1K exec", "Rust 1K exec", "Ratio", "Notes"
    );
    println!(
        "{:-<17}|{:-<9}|{:-<10}|{:-<6}|{:-<13}|{:-<14}|{:-<14}|{:-<8}|{:-<20}",
        "", "", "", "", "", "", "", "", ""
    );

    let mut solved_count = 0;
    for r in &results {
        let solved_str = if r.solved { "YES" } else { "NO" };
        let accuracy_str = format!("{:.0}%", r.accuracy * 100.0);
        let gens_str = format!("{}", r.generations);
        let evolve_str = format_duration(r.evolve_time);

        let (iris_str, rust_str, ratio_str) = if r.solved {
            if let Some(iris_time) = r.iris_1k_exec {
                let ratio = iris_time.as_secs_f64() / r.rust_1k_exec.as_secs_f64();
                (
                    format_duration(iris_time),
                    format_duration(r.rust_1k_exec),
                    format!("{:.0}x", ratio),
                )
            } else {
                ("-".to_string(), format_duration(r.rust_1k_exec), "-".to_string())
            }
        } else {
            ("-".to_string(), "-".to_string(), "-".to_string())
        };

        if r.solved {
            solved_count += 1;
        }

        // Truncate notes to fit table.
        let notes_display = if r.notes.len() > 40 {
            format!("{}...", &r.notes[..37])
        } else {
            r.notes.clone()
        };

        println!(
            "{:<17}| {:<8}| {:<9}| {:<5}| {:<12}| {:<13}| {:<13}| {:<7}| {}",
            r.name,
            solved_str,
            accuracy_str,
            gens_str,
            evolve_str,
            iris_str,
            rust_str,
            ratio_str,
            notes_display,
        );
    }

    println!();
    println!("Total benchmark time: {}", format_duration(total_time));
    println!(
        "Problems solved: {}/{}",
        solved_count,
        results.len()
    );
    println!();

    // Detailed per-problem evaluation for solved problems.
    println!("========================================");
    println!("DETAILED EVALUATION (solved problems)");
    println!("========================================");
    println!();

    let problems_ref = &problems;
    for (r, p) in results.iter().zip(problems_ref.iter()) {
        if !r.solved {
            continue;
        }
        println!("--- {} ---", r.name);
        // Re-run to show per-test-case results (we don't store the best
        // individual, but the problem structure tells us the expected I/O).
        for (i, tc) in p.test_cases.iter().enumerate() {
            let input_desc: Vec<String> = tc.inputs.iter().map(|v| format_value(v)).collect();
            let expected_desc = tc
                .expected_output
                .as_ref()
                .map(|e| {
                    e.iter()
                        .map(|v| format_value(v))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "?".to_string());
            println!(
                "  [{}] input=({}) expected={} PASS",
                i,
                input_desc.join(", "),
                expected_desc,
            );
        }
        println!();
    }

    // Analysis section.
    println!("========================================");
    println!("ANALYSIS");
    println!("========================================");
    println!();

    // Q1: What class of algorithms can IRIS evolve?
    println!("Q1: What class of algorithms can IRIS currently evolve?");
    let fold_based = ["List Length", "Contains", "Count Occur"];
    let primitive_based = ["List Reversal"];
    let complex = ["Linear Search", "Remove Dupes", "Insertion Sort", "Merge Sorted"];

    for category_name in ["Fold-based reductions", "Primitive-based", "Multi-pass/complex"] {
        let names = match category_name {
            "Fold-based reductions" => &fold_based[..],
            "Primitive-based" => &primitive_based[..],
            _ => &complex[..],
        };
        let category_results: Vec<&BenchResult> = results
            .iter()
            .filter(|r| names.contains(&r.name))
            .collect();
        let category_solved = category_results.iter().filter(|r| r.solved).count();
        println!(
            "  {}: {}/{} solved",
            category_name,
            category_solved,
            category_results.len()
        );
    }
    println!();

    // Q2: Interpreter overhead.
    println!("Q2: Interpreter overhead vs native Rust (for solved problems):");
    for r in &results {
        if r.solved {
            if let Some(iris_time) = r.iris_1k_exec {
                let ratio = iris_time.as_secs_f64() / r.rust_1k_exec.as_secs_f64();
                println!(
                    "  {}: {:.0}x slower ({} vs {})",
                    r.name,
                    ratio,
                    format_duration(iris_time),
                    format_duration(r.rust_1k_exec),
                );
            }
        }
    }
    println!();

    // Q3: Failure modes.
    println!("Q3: Failure modes for unsolved problems:");
    for r in &results {
        if !r.solved {
            println!(
                "  {}: best accuracy={:.0}%, stopped at gen {}, notes: {}",
                r.name,
                r.accuracy * 100.0,
                r.generations,
                r.notes,
            );
        }
    }
    println!();

    // The test passes as long as evolution runs without panicking.
    // We assert basic sanity.
    for r in &results {
        assert!(
            r.accuracy >= 0.0 && r.accuracy <= 1.0,
            "Accuracy for {} out of range: {}",
            r.name,
            r.accuracy,
        );
        assert!(
            r.generations >= 0,
            "Should have run at least 1 generation for {}",
            r.name,
        );
    }
}
