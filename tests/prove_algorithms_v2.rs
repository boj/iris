//! Advanced algorithm proving ground — beyond the basics.
//! Everything in v1 (27/27) is solved. Now: harder problems.

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
    println!("  {:40} {:>6} {:>5.0}% gen {:>5} {:>7.2}s",
             name, solved, c * 100.0, result.generations_run, result.total_time.as_secs_f64());
    (c, result.generations_run, result.total_time.as_secs_f64())
}

fn tc(inputs: Vec<Value>, expected: Value) -> TestCase {
    TestCase { inputs, expected_output: Some(vec![expected]), initial_state: None, expected_state: None }
}

fn list(vals: &[i64]) -> Value {
    Value::tuple(vals.iter().map(|&v| Value::Int(v)).collect())
}

#[test]
fn advanced_algorithm_proving() {
    println!("\n{}", "=".repeat(85));
    println!("  IRIS Advanced Algorithm Proving Ground v2");
    println!("  27/27 basic algorithms solved. Now: harder stuff.");
    println!("{}\n", "=".repeat(85));

    // =====================================================================
    // TIER 9: Mathematical functions
    // =====================================================================
    println!("--- TIER 9: Math Functions ---");

    solve("average([...])", vec![
        tc(vec![list(&[2, 4, 6])], Value::Int(4)),
        tc(vec![list(&[10])], Value::Int(10)),
        tc(vec![list(&[1, 2, 3, 4, 5])], Value::Int(3)),
        tc(vec![list(&[0, 0, 0, 0])], Value::Int(0)),
    ], 256, 3000);

    solve("range_span(list) = max - min", vec![
        tc(vec![list(&[1, 5, 3])], Value::Int(4)),
        tc(vec![list(&[10, 10, 10])], Value::Int(0)),
        tc(vec![list(&[-5, 5])], Value::Int(10)),
        tc(vec![list(&[7])], Value::Int(0)),
    ], 256, 3000);

    solve("sum_first_n(list, n)", vec![
        tc(vec![list(&[10, 20, 30, 40, 50]), Value::Int(3)], Value::Int(60)),
        tc(vec![list(&[1, 2, 3]), Value::Int(1)], Value::Int(1)),
        tc(vec![list(&[5, 5, 5, 5]), Value::Int(4)], Value::Int(20)),
        tc(vec![list(&[100]), Value::Int(1)], Value::Int(100)),
    ], 256, 3000);

    solve("is_even(x)", vec![
        tc(vec![Value::Int(4)], Value::Int(1)),
        tc(vec![Value::Int(7)], Value::Int(0)),
        tc(vec![Value::Int(0)], Value::Int(1)),
        tc(vec![Value::Int(-2)], Value::Int(1)),
        tc(vec![Value::Int(1)], Value::Int(0)),
    ], 128, 1000);

    solve("collatz_step(x)", vec![
        // if even: x/2, if odd: 3x+1
        tc(vec![Value::Int(4)], Value::Int(2)),
        tc(vec![Value::Int(7)], Value::Int(22)),
        tc(vec![Value::Int(1)], Value::Int(4)),
        tc(vec![Value::Int(10)], Value::Int(5)),
        tc(vec![Value::Int(3)], Value::Int(10)),
    ], 256, 3000);

    // =====================================================================
    // TIER 10: List transformations
    // =====================================================================
    println!("\n--- TIER 10: List Transformations ---");

    solve("double_each([...])", vec![
        tc(vec![list(&[1, 2, 3])], list(&[2, 4, 6])),
        tc(vec![list(&[0])], list(&[0])),
        tc(vec![list(&[5, 10])], list(&[10, 20])),
    ], 256, 3000);

    solve("square_each([...])", vec![
        tc(vec![list(&[1, 2, 3])], list(&[1, 4, 9])),
        tc(vec![list(&[0, 5])], list(&[0, 25])),
        tc(vec![list(&[-3])], list(&[9])),
    ], 256, 3000);

    solve("negate_each([...])", vec![
        tc(vec![list(&[1, -2, 3])], list(&[-1, 2, -3])),
        tc(vec![list(&[0])], list(&[0])),
    ], 128, 1000);

    solve("add_constant([...], c)", vec![
        tc(vec![list(&[1, 2, 3]), Value::Int(10)], list(&[11, 12, 13])),
        tc(vec![list(&[0, 0]), Value::Int(5)], list(&[5, 5])),
        tc(vec![list(&[10]), Value::Int(-3)], list(&[7])),
    ], 256, 3000);

    solve("pairwise_add(a, b)", vec![
        tc(vec![list(&[1, 2, 3]), list(&[4, 5, 6])], list(&[5, 7, 9])),
        tc(vec![list(&[0, 0]), list(&[0, 0])], list(&[0, 0])),
        tc(vec![list(&[10]), list(&[-10])], list(&[0])),
    ], 256, 3000);

    // =====================================================================
    // TIER 11: Multi-step computations
    // =====================================================================
    println!("\n--- TIER 11: Multi-Step ---");

    solve("normalize(list) = each / sum", vec![
        // Integer division — floor
        tc(vec![list(&[2, 4, 6])], list(&[0, 0, 0])),  // 2/12=0, 4/12=0, 6/12=0
        tc(vec![list(&[10, 20, 70])], list(&[0, 0, 0])),  // all < 1 with int div
        // Better test: multiply by 100 first for percentage
        tc(vec![list(&[1, 1, 1])], list(&[0, 0, 0])),
    ], 256, 3000);

    solve("cumulative_sum([...])", vec![
        tc(vec![list(&[1, 2, 3, 4])], list(&[1, 3, 6, 10])),
        tc(vec![list(&[5])], list(&[5])),
        tc(vec![list(&[1, 1, 1])], list(&[1, 2, 3])),
    ], 256, 3000);

    solve("differences([...])", vec![
        // consecutive differences: [a,b,c] → [b-a, c-b]
        tc(vec![list(&[1, 3, 6, 10])], list(&[2, 3, 4])),
        tc(vec![list(&[5, 5, 5])], list(&[0, 0])),
        tc(vec![list(&[10, 7])], list(&[-3])),
    ], 256, 3000);

    solve("inner_product_plus_bias(a, b, bias)", vec![
        // dot(a,b) + bias — a basic neuron
        tc(vec![list(&[1, 2, 3]), list(&[4, 5, 6]), Value::Int(10)], Value::Int(42)),
        tc(vec![list(&[1, 0]), list(&[0, 1]), Value::Int(0)], Value::Int(0)),
        tc(vec![list(&[2]), list(&[3]), Value::Int(1)], Value::Int(7)),
    ], 256, 3000);

    // =====================================================================
    // TIER 12: Conditional list operations
    // =====================================================================
    println!("\n--- TIER 12: Conditional List Ops ---");

    solve("filter_positives([...])", vec![
        tc(vec![list(&[1, -2, 3, -4, 5])], list(&[1, 3, 5])),
        tc(vec![list(&[-1, -2])], list(&[])),
        tc(vec![list(&[1, 2, 3])], list(&[1, 2, 3])),
    ], 256, 3000);

    solve("count_zeros([...])", vec![
        tc(vec![list(&[0, 1, 0, 2, 0])], Value::Int(3)),
        tc(vec![list(&[1, 2, 3])], Value::Int(0)),
        tc(vec![list(&[0])], Value::Int(1)),
    ], 128, 1000);

    solve("has_duplicate_adj([...])", vec![
        // any adjacent pair equal?
        tc(vec![list(&[1, 2, 2, 3])], Value::Int(1)),
        tc(vec![list(&[1, 2, 3])], Value::Int(0)),
        tc(vec![list(&[5, 5])], Value::Int(1)),
        tc(vec![list(&[1])], Value::Int(0)),
    ], 256, 3000);

    solve("count_greater_than(list, threshold)", vec![
        tc(vec![list(&[1, 5, 3, 7, 2]), Value::Int(3)], Value::Int(2)),
        tc(vec![list(&[10, 20, 30]), Value::Int(0)], Value::Int(3)),
        tc(vec![list(&[1, 2]), Value::Int(5)], Value::Int(0)),
    ], 256, 3000);

    // =====================================================================
    // TIER 13: Two-pass algorithms
    // =====================================================================
    println!("\n--- TIER 13: Two-Pass ---");

    solve("mean_deviation([...])", vec![
        // sum(abs(x - mean)) for each x
        // [1,2,3] mean=2 → |1-2|+|2-2|+|3-2| = 2
        tc(vec![list(&[1, 2, 3])], Value::Int(2)),
        tc(vec![list(&[5, 5, 5])], Value::Int(0)),
        tc(vec![list(&[0, 10])], Value::Int(10)),
    ], 256, 3000);

    solve("variance_numerator([...])", vec![
        // sum((x - mean)^2) — numerator of variance
        // [1,2,3] mean=2 → 1+0+1 = 2
        tc(vec![list(&[1, 2, 3])], Value::Int(2)),
        tc(vec![list(&[5, 5, 5])], Value::Int(0)),
        tc(vec![list(&[0, 4])], Value::Int(8)),
    ], 256, 3000);

    solve("distance_from_max([...])", vec![
        // max - each element
        // [1,5,3] → [4,0,2]
        tc(vec![list(&[1, 5, 3])], list(&[4, 0, 2])),
        tc(vec![list(&[3, 3, 3])], list(&[0, 0, 0])),
        tc(vec![list(&[10])], list(&[0])),
    ], 256, 3000);

    // =====================================================================
    // TIER 14: Classic CS problems
    // =====================================================================
    println!("\n--- TIER 14: Classic CS ---");

    solve("binary_to_decimal([1,0,1]→5)", vec![
        tc(vec![list(&[1, 0, 1])], Value::Int(5)),
        tc(vec![list(&[1, 1, 0])], Value::Int(6)),
        tc(vec![list(&[1])], Value::Int(1)),
        tc(vec![list(&[0])], Value::Int(0)),
        tc(vec![list(&[1, 0, 0, 0])], Value::Int(8)),
    ], 256, 3000);

    solve("digit_sum(n)", vec![
        // sum of digits: 123 → 6, 99 → 18
        // With integer ops: repeated mod 10 + div 10
        tc(vec![Value::Int(123)], Value::Int(6)),
        tc(vec![Value::Int(99)], Value::Int(18)),
        tc(vec![Value::Int(5)], Value::Int(5)),
        tc(vec![Value::Int(100)], Value::Int(1)),
    ], 256, 5000);

    solve("count_divisors(n)", vec![
        tc(vec![Value::Int(6)], Value::Int(4)),    // 1,2,3,6
        tc(vec![Value::Int(7)], Value::Int(2)),    // 1,7 (prime)
        tc(vec![Value::Int(12)], Value::Int(6)),   // 1,2,3,4,6,12
        tc(vec![Value::Int(1)], Value::Int(1)),
    ], 256, 5000);

    solve("is_prime(n)", vec![
        tc(vec![Value::Int(7)], Value::Int(1)),
        tc(vec![Value::Int(4)], Value::Int(0)),
        tc(vec![Value::Int(2)], Value::Int(1)),
        tc(vec![Value::Int(1)], Value::Int(0)),
        tc(vec![Value::Int(13)], Value::Int(1)),
        tc(vec![Value::Int(9)], Value::Int(0)),
    ], 256, 5000);

    println!("\n{}", "=".repeat(85));
    println!("  End of advanced proving ground.");
    println!("{}\n", "=".repeat(85));
}
