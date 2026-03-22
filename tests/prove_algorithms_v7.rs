//! IRIS Complexity Scaling v7 — algorithms IRIS needs to replace its own internals.
//!
//! v1-v3: basic through systems-level (tiers 1-21).
//! v4-v5: programs that process/mutate programs (tiers 22-27).
//! v6: direct self-modification opcode tests.
//! Now: sorting, graph ops, encoding, and simple interpreters — the algorithmic
//! substrate IRIS needs to evolve its own mutation, selection, and compilation passes.

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
fn complexity_scaling_v7() {
    println!("\n{}", "=".repeat(90));
    println!("  IRIS Complexity Scaling v7 — Algorithms for Self-Writing");
    println!("  Sorting, graphs, encoding, interpreters: the tools IRIS needs internally.");
    println!("{}\n", "=".repeat(90));

    let mut solved_count = 0usize;
    let mut total_count = 0usize;
    let mut total_time = 0.0f64;

    // =====================================================================
    // TIER 28: Sorting (the classic test)
    // IRIS needs to sort candidates by fitness, order mutation targets,
    // and maintain sorted populations. Can it evolve sorting primitives?
    // =====================================================================
    println!("--- TIER 28: Sorting ---");

    // bubble_sort_pass: one pass of bubble sort (pairwise swap if out of order)
    // [3,1,4,1,5] -> compare (3,1)->swap, (3,4)->ok, (4,1)->swap, (4,5)->ok -> [1,3,1,4,5]
    total_count += 1;
    let (c, _, t) = solve("bubble_sort_pass([3,1,4,1,5]) -> [1,3,1,4,5]", vec![
        tc(vec![list(&[3, 1, 4, 1, 5])], list(&[1, 3, 1, 4, 5])),
        tc(vec![list(&[2, 1])], list(&[1, 2])),
        tc(vec![list(&[1, 2, 3])], list(&[1, 2, 3])),
        tc(vec![list(&[5, 3, 1])], list(&[3, 1, 5])),
        tc(vec![list(&[4, 2])], list(&[2, 4])),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // is_permutation: do two lists contain the same elements?
    // Uses sorted comparison: same elements in any order -> 1
    total_count += 1;
    let (c, _, t) = solve("is_permutation([1,3,2], [2,1,3]) -> 1", vec![
        tc(vec![list(&[1, 3, 2]), list(&[2, 1, 3])], Value::Int(1)),
        tc(vec![list(&[1, 2, 3]), list(&[1, 2, 3])], Value::Int(1)),
        tc(vec![list(&[1, 2, 3]), list(&[1, 2, 4])], Value::Int(0)),
        tc(vec![list(&[5, 5]), list(&[5, 5])], Value::Int(1)),
        tc(vec![list(&[1, 2]), list(&[2, 2])], Value::Int(0)),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // insertion_position: where to insert a value to keep a sorted list sorted
    // [1,3,5,7], 4 -> index 2 (insert before 5)
    total_count += 1;
    let (c, _, t) = solve("insertion_position([1,3,5,7], 4) -> 2", vec![
        tc(vec![list(&[1, 3, 5, 7]), Value::Int(4)], Value::Int(2)),
        tc(vec![list(&[1, 3, 5, 7]), Value::Int(0)], Value::Int(0)),
        tc(vec![list(&[1, 3, 5, 7]), Value::Int(8)], Value::Int(4)),
        tc(vec![list(&[10, 20, 30]), Value::Int(15)], Value::Int(1)),
        tc(vec![list(&[10, 20, 30]), Value::Int(25)], Value::Int(2)),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // =====================================================================
    // TIER 29: Graph-like operations (on flat adjacency lists)
    // IRIS's semantic graphs are the core data structure. Programs that
    // reason about edges, nodes, and connectivity are essential for
    // self-modification: adding edges, rewiring, checking structure.
    // =====================================================================
    println!("\n--- TIER 29: Graph Operations ---");

    // out_degree: count edges from a given node in a flat edge list
    // Edge list [0,1, 0,2, 1,2] means edges 0->1, 0->2, 1->2
    // out_degree(edges, node=0) = 2 (edges 0->1, 0->2)
    total_count += 1;
    let (c, _, t) = solve("out_degree([0,1,0,2,1,2], node=0) -> 2", vec![
        tc(vec![list(&[0, 1, 0, 2, 1, 2]), Value::Int(0)], Value::Int(2)),
        tc(vec![list(&[0, 1, 0, 2, 1, 2]), Value::Int(1)], Value::Int(1)),
        tc(vec![list(&[0, 1, 0, 2, 1, 2]), Value::Int(2)], Value::Int(0)),
        tc(vec![list(&[1, 0, 2, 0]), Value::Int(1)], Value::Int(1)),
        tc(vec![list(&[1, 0, 2, 0]), Value::Int(2)], Value::Int(1)),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // has_edge: does the flat edge list contain edge (src, dst)?
    total_count += 1;
    let (c, _, t) = solve("has_edge([0,1,0,2,1,2], 0, 2) -> 1", vec![
        tc(vec![list(&[0, 1, 0, 2, 1, 2]), Value::Int(0), Value::Int(2)], Value::Int(1)),
        tc(vec![list(&[0, 1, 0, 2, 1, 2]), Value::Int(0), Value::Int(1)], Value::Int(1)),
        tc(vec![list(&[0, 1, 0, 2, 1, 2]), Value::Int(2), Value::Int(0)], Value::Int(0)),
        tc(vec![list(&[0, 1, 0, 2, 1, 2]), Value::Int(1), Value::Int(2)], Value::Int(1)),
        tc(vec![list(&[1, 0]), Value::Int(0), Value::Int(1)], Value::Int(0)),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // node_count_from_edges: max node ID + 1 in a flat edge list
    // [0,1, 1,2, 2,3] -> max is 3, so 4 nodes
    total_count += 1;
    let (c, _, t) = solve("node_count([0,1,1,2,2,3]) -> 4", vec![
        tc(vec![list(&[0, 1, 1, 2, 2, 3])], Value::Int(4)),
        tc(vec![list(&[0, 1])], Value::Int(2)),
        tc(vec![list(&[0, 5])], Value::Int(6)),
        tc(vec![list(&[3, 2, 1, 0])], Value::Int(4)),
        tc(vec![list(&[0, 0])], Value::Int(1)),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // =====================================================================
    // TIER 30: Encoder/decoder patterns
    // IRIS needs to serialize and deserialize programs. Run-length encoding
    // is a compression primitive — programs that can encode and decode
    // demonstrate the bidirectional transformation capability needed for
    // the codec layer.
    // =====================================================================
    println!("\n--- TIER 30: Encoder/Decoder ---");

    // encode_rle: run-length encode a list as [value, count, value, count, ...]
    // [1,1,2,2,2,3] -> [1,2,2,3,3,1]
    total_count += 1;
    let (c, _, t) = solve("encode_rle([1,1,2,2,2,3]) -> [1,2,2,3,3,1]", vec![
        tc(vec![list(&[1, 1, 2, 2, 2, 3])], list(&[1, 2, 2, 3, 3, 1])),
        tc(vec![list(&[5, 5, 5])], list(&[5, 3])),
        tc(vec![list(&[1, 2, 3])], list(&[1, 1, 2, 1, 3, 1])),
        tc(vec![list(&[7])], list(&[7, 1])),
        tc(vec![list(&[4, 4])], list(&[4, 2])),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // decode_rle: inverse of encode_rle
    // [1,2,2,3,3,1] -> [1,1,2,2,2,3]
    total_count += 1;
    let (c, _, t) = solve("decode_rle([1,2,2,3,3,1]) -> [1,1,2,2,2,3]", vec![
        tc(vec![list(&[1, 2, 2, 3, 3, 1])], list(&[1, 1, 2, 2, 2, 3])),
        tc(vec![list(&[5, 3])], list(&[5, 5, 5])),
        tc(vec![list(&[1, 1, 2, 1, 3, 1])], list(&[1, 2, 3])),
        tc(vec![list(&[7, 1])], list(&[7])),
        tc(vec![list(&[4, 2])], list(&[4, 4])),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // =====================================================================
    // TIER 31: Simple interpreters (the meta-level)
    // The ultimate test: can IRIS evolve a program that interprets other
    // programs? This is the kernel of self-writing — a program that takes
    // an opcode sequence and executes it. We use reverse Polish notation
    // with opcodes: 0=add, 1=sub, 2=mul. Values are operands.
    // =====================================================================
    println!("\n--- TIER 31: Simple Interpreters (RPN) ---");

    // eval_rpn: evaluate a 3-element RPN expression [a, b, op]
    // where op: 0=add, 1=sub, 2=mul
    // [3, 5, 0] -> 3 + 5 = 8
    total_count += 1;
    let (c, _, t) = solve("eval_rpn([3,5,0]) -> 8 (add)", vec![
        tc(vec![list(&[3, 5, 0])], Value::Int(8)),
        tc(vec![list(&[1, 2, 0])], Value::Int(3)),
        tc(vec![list(&[10, 7, 0])], Value::Int(17)),
        tc(vec![list(&[0, 0, 0])], Value::Int(0)),
        tc(vec![list(&[100, 1, 0])], Value::Int(101)),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // eval_rpn with subtraction
    // [10, 3, 1] -> 10 - 3 = 7
    total_count += 1;
    let (c, _, t) = solve("eval_rpn([10,3,1]) -> 7 (sub)", vec![
        tc(vec![list(&[10, 3, 1])], Value::Int(7)),
        tc(vec![list(&[5, 2, 1])], Value::Int(3)),
        tc(vec![list(&[20, 20, 1])], Value::Int(0)),
        tc(vec![list(&[100, 1, 1])], Value::Int(99)),
        tc(vec![list(&[0, 5, 1])], Value::Int(-5)),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // eval_rpn with multiplication
    // [4, 6, 2] -> 4 * 6 = 24
    total_count += 1;
    let (c, _, t) = solve("eval_rpn([4,6,2]) -> 24 (mul)", vec![
        tc(vec![list(&[4, 6, 2])], Value::Int(24)),
        tc(vec![list(&[3, 3, 2])], Value::Int(9)),
        tc(vec![list(&[7, 0, 2])], Value::Int(0)),
        tc(vec![list(&[1, 100, 2])], Value::Int(100)),
        tc(vec![list(&[5, 5, 2])], Value::Int(25)),
    ], 256, 3000);
    if c >= 0.99 { solved_count += 1; }
    total_time += t;

    // =====================================================================
    // Summary
    // =====================================================================
    println!("\n{}", "=".repeat(90));
    println!(
        "  v7 Results: {}/{} solved in {:.1}s",
        solved_count, total_count, total_time
    );
    println!("  Tiers 28-31: sorting, graphs, encoding, interpreters");
    println!("{}\n", "=".repeat(90));
}
