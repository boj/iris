//! IRIS Complexity Scaling v4 — programs that process programs.
//!
//! v1: 27/27 basic algorithms. v2: 25/25 advanced. v3: 52/52 systems-level.
//! Now: using integer lists as stand-ins for program representations,
//! these problems test the pattern "programs that reason about sequences
//! of operations" — steps toward IRIS writing itself.

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
fn complexity_scaling_v4() {
    println!("\n{}", "=".repeat(90));
    println!("  IRIS Complexity Scaling v4 — Programs That Process Programs");
    println!("  Integer lists as stand-ins for program representation.");
    println!("{}\n", "=".repeat(90));

    let mut solved_count = 0usize;
    let mut total_count = 0usize;
    let mut total_time = 0.0f64;

    // =====================================================================
    // TIER 22: Simple program analysis
    // Using integer lists as opcode sequences
    // =====================================================================
    println!("--- TIER 22: Simple Program Analysis ---");

    // count_nodes: count elements in a program-as-list
    // (Equivalent to count_elements, but framed as program analysis)
    total_count += 1;
    let (c, _, t) = solve("count_nodes(program) — count opcodes", vec![
        tc(vec![list(&[0x00, 0x01, 0x02])], Value::Int(3)),
        tc(vec![list(&[0x08])], Value::Int(1)),
        tc(vec![list(&[0x00, 0x00, 0x01, 0x02, 0x08])], Value::Int(5)),
        tc(vec![list(&[0x03, 0x04])], Value::Int(2)),
    ], 128, 2000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // max_opcode: find the largest opcode value in the list
    total_count += 1;
    let (c, _, t) = solve("max_opcode(program) — largest opcode", vec![
        tc(vec![list(&[3, 1, 4, 1, 5])], Value::Int(5)),
        tc(vec![list(&[8, 2, 0])], Value::Int(8)),
        tc(vec![list(&[1, 1, 1])], Value::Int(1)),
        tc(vec![list(&[0, 7, 3, 7])], Value::Int(7)),
    ], 128, 2000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // has_fold: check if opcode 0x08 (Fold) is in the list -> 1 or 0
    total_count += 1;
    let (c, _, t) = solve("has_fold(program) — contains opcode 8?", vec![
        tc(vec![list(&[0, 1, 8, 3])], Value::Int(1)),
        tc(vec![list(&[0, 1, 2, 3])], Value::Int(0)),
        tc(vec![list(&[8])], Value::Int(1)),
        tc(vec![list(&[7, 9, 0])], Value::Int(0)),
        tc(vec![list(&[1, 8, 1])], Value::Int(1)),
    ], 128, 2000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // =====================================================================
    // TIER 23: Program transformation
    // =====================================================================
    println!("\n--- TIER 23: Program Transformation ---");

    // increment_all_opcodes: add 1 to each opcode (map(+1))
    total_count += 1;
    let (c, _, t) = solve("increment_opcodes(program) — add 1 to each", vec![
        tc(vec![list(&[0, 1, 2])], list(&[1, 2, 3])),
        tc(vec![list(&[5, 10])], list(&[6, 11])),
        tc(vec![list(&[0, 0, 0])], list(&[1, 1, 1])),
        tc(vec![list(&[7])], list(&[8])),
    ], 128, 2000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // replace_opcode: replace all occurrences of `old` with `new`
    total_count += 1;
    let (c, _, t) = solve("replace_opcode(prog, old, new) — substitute", vec![
        tc(vec![list(&[1, 2, 1, 3]), Value::Int(1), Value::Int(9)], list(&[9, 2, 9, 3])),
        tc(vec![list(&[5, 5, 5]), Value::Int(5), Value::Int(0)], list(&[0, 0, 0])),
        tc(vec![list(&[1, 2, 3]), Value::Int(4), Value::Int(9)], list(&[1, 2, 3])),
        tc(vec![list(&[0, 1, 0]), Value::Int(0), Value::Int(7)], list(&[7, 1, 7])),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // remove_nops: filter out zeros (filter(!=0))
    total_count += 1;
    let (c, _, t) = solve("remove_nops(program) — filter out zeros", vec![
        tc(vec![list(&[1, 0, 2, 0, 3])], list(&[1, 2, 3])),
        tc(vec![list(&[0, 0, 0])], list(&[])),
        tc(vec![list(&[5, 3, 1])], list(&[5, 3, 1])),
        tc(vec![list(&[0, 7, 0])], list(&[7])),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // =====================================================================
    // TIER 24: Simple code generation
    // =====================================================================
    println!("\n--- TIER 24: Simple Code Generation ---");

    // make_add_program: no input, returns [0, 0, 1]
    // Represents: add(input0, input1)
    total_count += 1;
    let (c, _, t) = solve("make_add_program() -> [0, 0, 1]", vec![
        tc(vec![Value::Int(0)], list(&[0, 0, 1])),
        tc(vec![Value::Int(1)], list(&[0, 0, 1])),
        tc(vec![Value::Int(99)], list(&[0, 0, 1])),
    ], 128, 2000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // make_fold_sum: no input, returns [8, 0, 0]
    // Represents: fold(0, add)
    total_count += 1;
    let (c, _, t) = solve("make_fold_sum() -> [8, 0, 0]", vec![
        tc(vec![Value::Int(0)], list(&[8, 0, 0])),
        tc(vec![Value::Int(1)], list(&[8, 0, 0])),
        tc(vec![Value::Int(42)], list(&[8, 0, 0])),
    ], 128, 2000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // =====================================================================
    // Summary
    // =====================================================================
    println!("\n{}", "=".repeat(90));
    println!(
        "  v4 Results: {}/{} solved in {:.1}s",
        solved_count, total_count, total_time
    );
    println!("{}\n", "=".repeat(90));
}
