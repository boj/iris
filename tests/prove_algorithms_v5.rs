//! IRIS Complexity Scaling v5 — programs that modify programs.
//!
//! v1: 27/27 basic algorithms. v2: 25/25 advanced. v3: 52/52 systems-level.
//! v4: programs that process programs. Now: the same operations IRIS uses
//! internally for mutation — can IRIS evolve the tools it needs to modify itself?

use iris_types::eval::*;
use iris_exec::service::*;
use iris_evolve::*;
use iris_evolve::config::*;

fn solve(name: &str, test_cases: Vec<TestCase>, pop: usize, max_gens: usize) -> (f32, usize, f64) {
    let spec = ProblemSpec {
        test_cases,
        description: name.to_string(),
        target_cost: None,
    };
    let config = EvolutionConfig {
        population_size: pop,
        max_generations: max_gens,
        num_demes: 1,
        ..EvolutionConfig::default()
    };
    let exec = IrisExecutionService::new(ExecConfig::default());
    let result = evolve(config, spec, &exec);
    let c = result.best_individual.fitness.correctness();
    let solved = if c >= 0.99 { "SOLVED" } else { "FAILED" };
    println!("  {:50} {:>6} {:>5.0}% gen {:>5} {:>7.2}s",
             name, solved, c * 100.0, result.generations_run, result.total_time.as_secs_f64());
    (c, result.generations_run, result.total_time.as_secs_f64())
}

fn tc(inputs: Vec<Value>, expected: Value) -> TestCase {
    TestCase {
        inputs,
        expected_output: Some(vec![expected]),
        initial_state: None,
        expected_state: None,
    }
}

fn list(vals: &[i64]) -> Value {
    Value::tuple(vals.iter().map(|&v| Value::Int(v)).collect())
}

#[test]
fn complexity_scaling_v5() {
    println!("\n{}", "=".repeat(90));
    println!("  IRIS Complexity Scaling v5 — Programs That Modify Programs");
    println!("  Can IRIS evolve the same operations it uses for self-modification?");
    println!("{}\n", "=".repeat(90));

    let mut solved_count = 0usize;
    let mut total_count = 0usize;
    let mut total_time = 0.0f64;

    // =====================================================================
    // TIER 25: Program introspection
    // Analyzing opcode sequences — the first step to self-awareness
    // =====================================================================
    println!("--- TIER 25: Program Introspection ---");

    // count_unique_ops: number of distinct opcodes in a program
    // E.g., [1,2,1,3,1] has 3 unique opcodes: {1,2,3}
    total_count += 1;
    let (c, _, t) = solve("count_unique_ops(program) — distinct opcodes", vec![
        tc(vec![list(&[1, 2, 1, 3, 1])], Value::Int(3)),
        tc(vec![list(&[5, 5, 5, 5])], Value::Int(1)),
        tc(vec![list(&[1, 2, 3, 4, 5])], Value::Int(5)),
        tc(vec![list(&[0, 0])], Value::Int(1)),
        tc(vec![list(&[1, 2])], Value::Int(2)),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // program_depth: longest run of non-zero values (proxy for nesting depth)
    // E.g., [1,2,0,3,4,5,0] -> longest non-zero run = 3 ([3,4,5])
    total_count += 1;
    let (c, _, t) = solve("program_depth(program) — longest non-zero run", vec![
        tc(vec![list(&[1, 2, 0, 3, 4, 5, 0])], Value::Int(3)),
        tc(vec![list(&[1, 2, 3])], Value::Int(3)),
        tc(vec![list(&[0, 0, 0])], Value::Int(0)),
        tc(vec![list(&[1, 0, 1, 0, 1])], Value::Int(1)),
        tc(vec![list(&[0, 1, 2, 0])], Value::Int(2)),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // most_common_op: mode of the list (most frequently occurring value)
    // E.g., [1,2,1,3,1] -> 1 (appears 3 times)
    total_count += 1;
    let (c, _, t) = solve("most_common_op(program) — mode", vec![
        tc(vec![list(&[1, 2, 1, 3, 1])], Value::Int(1)),
        tc(vec![list(&[5, 5, 3])], Value::Int(5)),
        tc(vec![list(&[7])], Value::Int(7)),
        tc(vec![list(&[2, 2, 3, 3, 3])], Value::Int(3)),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // =====================================================================
    // TIER 26: Program mutation
    // The actual operations mutation.rs performs on programs
    // =====================================================================
    println!("\n--- TIER 26: Program Mutation ---");

    // insert_at: insert value at index position
    // insert_at([1,2,3], 1, 99) -> [1,99,2,3]
    total_count += 1;
    let (c, _, t) = solve("insert_at(list, idx, val) — insert element", vec![
        tc(vec![list(&[1, 2, 3]), Value::Int(1), Value::Int(99)], list(&[1, 99, 2, 3])),
        tc(vec![list(&[1, 2, 3]), Value::Int(0), Value::Int(99)], list(&[99, 1, 2, 3])),
        tc(vec![list(&[1, 2, 3]), Value::Int(3), Value::Int(99)], list(&[1, 2, 3, 99])),
        tc(vec![list(&[5]), Value::Int(0), Value::Int(10)], list(&[10, 5])),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // delete_at: remove element at index
    // delete_at([1,2,3,4], 2) -> [1,2,4]
    total_count += 1;
    let (c, _, t) = solve("delete_at(list, idx) — remove element", vec![
        tc(vec![list(&[1, 2, 3, 4]), Value::Int(2)], list(&[1, 2, 4])),
        tc(vec![list(&[1, 2, 3, 4]), Value::Int(0)], list(&[2, 3, 4])),
        tc(vec![list(&[1, 2, 3, 4]), Value::Int(3)], list(&[1, 2, 3])),
        tc(vec![list(&[5, 10]), Value::Int(0)], list(&[10])),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // swap_elements: swap elements at two positions
    // swap_elements([1,2,3,4], 1, 3) -> [1,4,3,2]
    total_count += 1;
    let (c, _, t) = solve("swap_elements(list, i, j) — swap positions", vec![
        tc(vec![list(&[1, 2, 3, 4]), Value::Int(1), Value::Int(3)], list(&[1, 4, 3, 2])),
        tc(vec![list(&[1, 2, 3, 4]), Value::Int(0), Value::Int(2)], list(&[3, 2, 1, 4])),
        tc(vec![list(&[10, 20, 30]), Value::Int(0), Value::Int(1)], list(&[20, 10, 30])),
        tc(vec![list(&[5, 6]), Value::Int(0), Value::Int(1)], list(&[6, 5])),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // =====================================================================
    // TIER 27: Program composition
    // Combining programs — the building blocks of crossover
    // =====================================================================
    println!("\n--- TIER 27: Program Composition ---");

    // interleave: merge two lists alternately
    // interleave([1,2,3], [4,5,6]) -> [1,4,2,5,3,6]
    total_count += 1;
    let (c, _, t) = solve("interleave(a, b) — alternate merge", vec![
        tc(vec![list(&[1, 2, 3]), list(&[4, 5, 6])], list(&[1, 4, 2, 5, 3, 6])),
        tc(vec![list(&[1, 2]), list(&[3, 4])], list(&[1, 3, 2, 4])),
        tc(vec![list(&[10]), list(&[20])], list(&[10, 20])),
        tc(vec![list(&[1, 2, 3]), list(&[7, 8, 9])], list(&[1, 7, 2, 8, 3, 9])),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // flatten_pairs: flatten list of 2-element tuples
    // Represented as flat list: [1,2,3,4] (pairs (1,2),(3,4)) -> [1,2,3,4]
    // Actually: extract even-indexed and odd-indexed -> concat
    // Simpler: just identity on flat lists... Let's do something useful:
    // double_elements: [1,2,3] -> [1,1,2,2,3,3]
    total_count += 1;
    let (c, _, t) = solve("double_elements(list) — duplicate each", vec![
        tc(vec![list(&[1, 2, 3])], list(&[1, 1, 2, 2, 3, 3])),
        tc(vec![list(&[5])], list(&[5, 5])),
        tc(vec![list(&[1, 2])], list(&[1, 1, 2, 2])),
        tc(vec![list(&[0, 1, 0])], list(&[0, 0, 1, 1, 0, 0])),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // zip_with_index: pair each element with its index
    // [10,20,30] -> [0,10,1,20,2,30] (flattened index-value pairs)
    total_count += 1;
    let (c, _, t) = solve("zip_with_index(list) — index-value pairs", vec![
        tc(vec![list(&[10, 20, 30])], list(&[0, 10, 1, 20, 2, 30])),
        tc(vec![list(&[5])], list(&[0, 5])),
        tc(vec![list(&[7, 8])], list(&[0, 7, 1, 8])),
        tc(vec![list(&[1, 2, 3, 4])], list(&[0, 1, 1, 2, 2, 3, 3, 4])),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // =====================================================================
    // Summary
    // =====================================================================
    println!("\n{}", "=".repeat(90));
    println!(
        "  v5 Results: {}/{} solved in {:.1}s",
        solved_count, total_count, total_time
    );
    println!("{}\n", "=".repeat(90));
}
