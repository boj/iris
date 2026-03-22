//! IRIS Complexity Scaling v3 — pushing toward self-writing capability.
//! v1: 27/27 basic algorithms. v2: 25/25 advanced. Now: systems-level programs.

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
    TestCase { inputs, expected_output: Some(vec![expected]), initial_state: None, expected_state: None }
}

fn list(vals: &[i64]) -> Value {
    Value::tuple(vals.iter().map(|&v| Value::Int(v)).collect())
}

fn stc(inputs: Vec<Value>, expected: Value, init_state: StateStore, exp_state: StateStore) -> TestCase {
    TestCase { inputs, expected_output: Some(vec![expected]), initial_state: Some(init_state), expected_state: Some(exp_state) }
}

use std::collections::{BTreeMap, HashMap};
type StateStore = BTreeMap<String, Value>;

fn state(pairs: &[(&str, i64)]) -> StateStore {
    pairs.iter().map(|(k, v)| (k.to_string(), Value::Int(*v))).collect()
}

#[test]
fn complexity_scaling_v3() {
    println!("\n{}", "=".repeat(90));
    println!("  IRIS Complexity Scaling v3 — Toward Self-Writing");
    println!("  52/52 algorithms solved. Now: programs that process programs.");
    println!("{}\n", "=".repeat(90));

    // =====================================================================
    // TIER 15: Data structure operations
    // =====================================================================
    println!("--- TIER 15: Data Structures ---");

    solve("stack_push([1,2], 3) → [1,2,3]", vec![
        tc(vec![list(&[1, 2]), Value::Int(3)], list(&[1, 2, 3])),
        tc(vec![list(&[]), Value::Int(5)], list(&[5])),
        tc(vec![list(&[10]), Value::Int(20)], list(&[10, 20])),
    ], 256, 3000);

    solve("last_element([1,2,3]) → 3", vec![
        tc(vec![list(&[1, 2, 3])], Value::Int(3)),
        tc(vec![list(&[10])], Value::Int(10)),
        tc(vec![list(&[5, 10, 15, 20])], Value::Int(20)),
    ], 256, 3000);

    solve("init([1,2,3]) → [1,2]", vec![
        tc(vec![list(&[1, 2, 3])], list(&[1, 2])),
        tc(vec![list(&[10, 20])], list(&[10])),
        tc(vec![list(&[5])], list(&[])),
    ], 256, 3000);

    solve("repeat(x, n) → [x, x, ..n times]", vec![
        tc(vec![Value::Int(7), Value::Int(3)], list(&[7, 7, 7])),
        tc(vec![Value::Int(0), Value::Int(4)], list(&[0, 0, 0, 0])),
        tc(vec![Value::Int(1), Value::Int(1)], list(&[1])),
    ], 256, 3000);

    solve("range(n) → [0, 1, ..n-1]", vec![
        tc(vec![Value::Int(5)], list(&[0, 1, 2, 3, 4])),
        tc(vec![Value::Int(1)], list(&[0])),
        tc(vec![Value::Int(3)], list(&[0, 1, 2])),
    ], 256, 3000);

    // =====================================================================
    // TIER 16: String-like operations (on integer lists as "strings")
    // =====================================================================
    println!("\n--- TIER 16: String-Like ---");

    solve("starts_with([1,2,3,4], [1,2]) → 1", vec![
        tc(vec![list(&[1, 2, 3, 4]), list(&[1, 2])], Value::Int(1)),
        tc(vec![list(&[1, 2, 3]), list(&[2, 3])], Value::Int(0)),
        tc(vec![list(&[5]), list(&[5])], Value::Int(1)),
        tc(vec![list(&[1, 2]), list(&[1, 2, 3])], Value::Int(0)),
    ], 256, 3000);

    solve("hamming_distance([1,0,1], [1,1,0]) → 2", vec![
        tc(vec![list(&[1, 0, 1, 1]), list(&[1, 1, 0, 1])], Value::Int(2)),
        tc(vec![list(&[1, 1, 1]), list(&[1, 1, 1])], Value::Int(0)),
        tc(vec![list(&[0, 0]), list(&[1, 1])], Value::Int(2)),
    ], 256, 3000);

    solve("run_length_count([1,1,2,2,2,3]) → 3", vec![
        tc(vec![list(&[1, 1, 2, 2, 2, 3])], Value::Int(3)),
        tc(vec![list(&[1, 2, 3])], Value::Int(3)),
        tc(vec![list(&[5, 5, 5])], Value::Int(1)),
        tc(vec![list(&[1])], Value::Int(1)),
    ], 256, 3000);

    // =====================================================================
    // TIER 17: Higher-order function synthesis
    // =====================================================================
    println!("\n--- TIER 17: Higher-Order ---");

    solve("apply_twice(f, x) where f=+3", vec![
        // f(x) = x + 3, apply_twice = f(f(x)) = x + 6
        tc(vec![Value::Int(0)], Value::Int(6)),
        tc(vec![Value::Int(10)], Value::Int(16)),
        tc(vec![Value::Int(-3)], Value::Int(3)),
    ], 256, 3000);

    solve("compose(f, g, x) where f=*2, g=+1", vec![
        // f(g(x)) = (x+1)*2
        tc(vec![Value::Int(0)], Value::Int(2)),
        tc(vec![Value::Int(3)], Value::Int(8)),
        tc(vec![Value::Int(10)], Value::Int(22)),
    ], 256, 3000);

    // =====================================================================
    // TIER 18: Matrix-like operations (flat representation)
    // =====================================================================
    println!("\n--- TIER 18: Matrix Ops ---");

    solve("mat_trace_2x2([a,b,c,d]) → a+d", vec![
        tc(vec![list(&[1, 2, 3, 4])], Value::Int(5)),
        tc(vec![list(&[5, 0, 0, 5])], Value::Int(10)),
        tc(vec![list(&[0, 0, 0, 0])], Value::Int(0)),
    ], 256, 3000);

    solve("mat_det_2x2([a,b,c,d]) → a*d - b*c", vec![
        tc(vec![list(&[1, 2, 3, 4])], Value::Int(-2)),
        tc(vec![list(&[2, 0, 0, 3])], Value::Int(6)),
        tc(vec![list(&[1, 0, 0, 1])], Value::Int(1)),
        tc(vec![list(&[3, 1, 5, 2])], Value::Int(1)),
    ], 256, 3000);

    solve("mat_vec_mul_2x2([a,b,c,d], [x,y])", vec![
        // [a*x+b*y, c*x+d*y]
        tc(vec![list(&[1, 0, 0, 1]), list(&[3, 4])], list(&[3, 4])),
        tc(vec![list(&[2, 0, 0, 2]), list(&[5, 10])], list(&[10, 20])),
        tc(vec![list(&[1, 2, 3, 4]), list(&[1, 1])], list(&[3, 7])),
    ], 256, 3000);

    // =====================================================================
    // TIER 19: Stateful computations
    // =====================================================================
    println!("\n--- TIER 19: Stateful ---");

    solve("counter_increment(state) → state+1", vec![
        stc(vec![], Value::Unit, state(&[("count", 0)]), state(&[("count", 1)])),
        stc(vec![], Value::Unit, state(&[("count", 5)]), state(&[("count", 6)])),
        stc(vec![], Value::Unit, state(&[("count", -1)]), state(&[("count", 0)])),
    ], 256, 3000);

    solve("accumulate(state, x) → state.sum + x", vec![
        stc(vec![Value::Int(10)], Value::Int(10), state(&[("sum", 0)]), state(&[("sum", 10)])),
        stc(vec![Value::Int(5)], Value::Int(15), state(&[("sum", 10)]), state(&[("sum", 15)])),
        stc(vec![Value::Int(-3)], Value::Int(12), state(&[("sum", 15)]), state(&[("sum", 12)])),
    ], 256, 3000);

    // =====================================================================
    // TIER 20: Multi-step algorithms
    // =====================================================================
    println!("\n--- TIER 20: Multi-Step Algorithms ---");

    solve("moving_avg_2([1,3,5,7]) → [2,4,6]", vec![
        tc(vec![list(&[1, 3, 5, 7])], list(&[2, 4, 6])),
        tc(vec![list(&[10, 20])], list(&[15])),
        tc(vec![list(&[0, 0, 0])], list(&[0, 0])),
    ], 256, 3000);

    solve("median_of_3(a, b, c)", vec![
        tc(vec![Value::Int(1), Value::Int(3), Value::Int(2)], Value::Int(2)),
        tc(vec![Value::Int(5), Value::Int(5), Value::Int(5)], Value::Int(5)),
        tc(vec![Value::Int(10), Value::Int(1), Value::Int(7)], Value::Int(7)),
        tc(vec![Value::Int(3), Value::Int(1), Value::Int(2)], Value::Int(2)),
    ], 256, 3000);

    solve("argmax([3,1,4,1,5]) → 4", vec![
        tc(vec![list(&[3, 1, 4, 1, 5])], Value::Int(4)),
        tc(vec![list(&[10])], Value::Int(0)),
        tc(vec![list(&[1, 2, 3])], Value::Int(2)),
        tc(vec![list(&[5, 5, 5])], Value::Int(0)),
    ], 256, 3000);

    solve("top_2_sum([3,1,4,1,5]) → 9", vec![
        // sum of two largest elements
        tc(vec![list(&[3, 1, 4, 1, 5])], Value::Int(9)),
        tc(vec![list(&[10, 20])], Value::Int(30)),
        tc(vec![list(&[1, 1, 1])], Value::Int(2)),
    ], 256, 3000);

    solve("euclidean_dist_sq(a, b)", vec![
        // sum of (a[i]-b[i])^2
        tc(vec![list(&[0, 0]), list(&[3, 4])], Value::Int(25)),
        tc(vec![list(&[1, 1, 1]), list(&[1, 1, 1])], Value::Int(0)),
        tc(vec![list(&[0]), list(&[5])], Value::Int(25)),
    ], 256, 3000);

    // =====================================================================
    // TIER 21: Programs about programs (meta-level)
    // =====================================================================
    println!("\n--- TIER 21: Meta-Level (programs about numbers that represent programs) ---");

    solve("count_ops(opcode_list) → count non-zero", vec![
        tc(vec![list(&[1, 0, 2, 0, 3])], Value::Int(3)),
        tc(vec![list(&[0, 0, 0])], Value::Int(0)),
        tc(vec![list(&[5, 5, 5])], Value::Int(3)),
    ], 128, 1000);

    solve("checksum(data) → sum mod 256", vec![
        tc(vec![list(&[100, 200, 50])], Value::Int(94)),  // 350 mod 256
        tc(vec![list(&[0, 0])], Value::Int(0)),
        tc(vec![list(&[128, 128])], Value::Int(0)),  // 256 mod 256
        tc(vec![list(&[1, 2, 3])], Value::Int(6)),
    ], 256, 3000);

    solve("xor_fold([a,b,c,...]) → a^b^c^...", vec![
        tc(vec![list(&[5, 3])], Value::Int(6)),       // 0101 ^ 0011 = 0110
        tc(vec![list(&[7, 7])], Value::Int(0)),       // x ^ x = 0
        tc(vec![list(&[0, 0, 0])], Value::Int(0)),
        tc(vec![list(&[1, 2, 4])], Value::Int(7)),    // 001 ^ 010 ^ 100 = 111
    ], 128, 1000);

    println!("\n{}", "=".repeat(90));
    println!("  End of v3 proving ground.");
    println!("{}\n", "=".repeat(90));
}
