//! Progressive algorithm proving ground.
//! Start simple, scale up, find where IRIS breaks.

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
    println!("  {:30} {:>6} {:>5.0}% gen {:>5} {:>7.2}s",
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
fn progressive_algorithm_proving() {
    println!("\n{}", "=".repeat(80));
    println!("  IRIS Progressive Algorithm Proving Ground");
    println!("  Start simple. Scale up. Find the wall.");
    println!("{}\n", "=".repeat(80));

    // =====================================================================
    // TIER 1: Single operations (should all solve gen 1)
    // =====================================================================
    println!("--- TIER 1: Single Operations ---");

    solve("negate(x)", vec![
        tc(vec![Value::Int(5)], Value::Int(-5)),
        tc(vec![Value::Int(-3)], Value::Int(3)),
        tc(vec![Value::Int(0)], Value::Int(0)),
    ], 64, 200);

    solve("square(x)", vec![
        tc(vec![Value::Int(3)], Value::Int(9)),
        tc(vec![Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(-4)], Value::Int(16)),
        tc(vec![Value::Int(1)], Value::Int(1)),
    ], 64, 500);

    solve("max(a, b)", vec![
        tc(vec![Value::Int(3), Value::Int(5)], Value::Int(5)),
        tc(vec![Value::Int(10), Value::Int(2)], Value::Int(10)),
        tc(vec![Value::Int(-1), Value::Int(-5)], Value::Int(-1)),
        tc(vec![Value::Int(7), Value::Int(7)], Value::Int(7)),
    ], 64, 500);

    solve("clamp(x, 0, 10)", vec![
        tc(vec![Value::Int(5)], Value::Int(5)),
        tc(vec![Value::Int(-3)], Value::Int(0)),
        tc(vec![Value::Int(15)], Value::Int(10)),
        tc(vec![Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(10)], Value::Int(10)),
    ], 128, 1000);

    // =====================================================================
    // TIER 2: Fold-based reductions (should solve quickly)
    // =====================================================================
    println!("\n--- TIER 2: Fold Reductions ---");

    solve("sum([...])", vec![
        tc(vec![list(&[1, 2, 3])], Value::Int(6)),
        tc(vec![list(&[10])], Value::Int(10)),
        tc(vec![list(&[0, 0, 0])], Value::Int(0)),
    ], 64, 200);

    solve("product([...])", vec![
        tc(vec![list(&[2, 3, 4])], Value::Int(24)),
        tc(vec![list(&[1, 1, 1])], Value::Int(1)),
        tc(vec![list(&[5])], Value::Int(5)),
    ], 64, 200);

    solve("min([...])", vec![
        tc(vec![list(&[3, 1, 4, 1, 5])], Value::Int(1)),
        tc(vec![list(&[10])], Value::Int(10)),
        tc(vec![list(&[-5, 0, 5])], Value::Int(-5)),
    ], 64, 200);

    solve("count_elements([...])", vec![
        tc(vec![list(&[1, 2, 3])], Value::Int(3)),
        tc(vec![list(&[1])], Value::Int(1)),
        tc(vec![list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])], Value::Int(10)),
    ], 64, 500);

    // =====================================================================
    // TIER 3: Map + Fold compositions
    // =====================================================================
    println!("\n--- TIER 3: Compositions ---");

    solve("sum_of_squares([...])", vec![
        tc(vec![list(&[1, 2, 3])], Value::Int(14)),
        tc(vec![list(&[0])], Value::Int(0)),
        tc(vec![list(&[4, 3])], Value::Int(25)),
    ], 128, 1000);

    solve("dot_product(a, b)", vec![
        tc(vec![list(&[1, 2, 3]), list(&[4, 5, 6])], Value::Int(32)),
        tc(vec![list(&[1, 0]), list(&[0, 1])], Value::Int(0)),
        tc(vec![list(&[2, 3]), list(&[4, 5])], Value::Int(23)),
    ], 256, 3000);

    solve("sum_of_abs([...])", vec![
        tc(vec![list(&[1, -2, 3, -4])], Value::Int(10)),
        tc(vec![list(&[0])], Value::Int(0)),
        tc(vec![list(&[-5, -5])], Value::Int(10)),
    ], 128, 1000);

    // =====================================================================
    // TIER 4: Conditionals + dispatch
    // =====================================================================
    println!("\n--- TIER 4: Conditionals ---");

    solve("abs(x)", vec![
        tc(vec![Value::Int(5)], Value::Int(5)),
        tc(vec![Value::Int(-3)], Value::Int(3)),
        tc(vec![Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(-100)], Value::Int(100)),
    ], 128, 1000);

    solve("sign(x) -> -1/0/1", vec![
        tc(vec![Value::Int(5)], Value::Int(1)),
        tc(vec![Value::Int(-3)], Value::Int(-1)),
        tc(vec![Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(100)], Value::Int(1)),
        tc(vec![Value::Int(-1)], Value::Int(-1)),
    ], 256, 2000);

    solve("calculator(a,op,b)", vec![
        tc(vec![Value::Int(3), Value::Int(0), Value::Int(5)], Value::Int(8)),
        tc(vec![Value::Int(10), Value::Int(1), Value::Int(3)], Value::Int(7)),
        tc(vec![Value::Int(4), Value::Int(2), Value::Int(6)], Value::Int(24)),
        tc(vec![Value::Int(1), Value::Int(0), Value::Int(1)], Value::Int(2)),
        tc(vec![Value::Int(5), Value::Int(1), Value::Int(2)], Value::Int(3)),
    ], 256, 2000);

    // =====================================================================
    // TIER 5: Iterative / recursive
    // =====================================================================
    println!("\n--- TIER 5: Iteration ---");

    solve("factorial(n)", vec![
        tc(vec![Value::Int(0)], Value::Int(1)),
        tc(vec![Value::Int(1)], Value::Int(1)),
        tc(vec![Value::Int(5)], Value::Int(120)),
        tc(vec![Value::Int(3)], Value::Int(6)),
    ], 128, 1000);

    solve("fibonacci(n)", vec![
        tc(vec![Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(1)], Value::Int(1)),
        tc(vec![Value::Int(5)], Value::Int(5)),
        tc(vec![Value::Int(7)], Value::Int(13)),
    ], 256, 3000);

    solve("gcd(a, b)", vec![
        tc(vec![Value::Int(12), Value::Int(8)], Value::Int(4)),
        tc(vec![Value::Int(7), Value::Int(3)], Value::Int(1)),
        tc(vec![Value::Int(100), Value::Int(75)], Value::Int(25)),
    ], 256, 3000);

    solve("power(x, n)", vec![
        tc(vec![Value::Int(2), Value::Int(0)], Value::Int(1)),
        tc(vec![Value::Int(2), Value::Int(3)], Value::Int(8)),
        tc(vec![Value::Int(3), Value::Int(2)], Value::Int(9)),
        tc(vec![Value::Int(5), Value::Int(3)], Value::Int(125)),
    ], 128, 1000);

    // =====================================================================
    // TIER 6: Predicate-based operations
    // =====================================================================
    println!("\n--- TIER 6: Predicates ---");

    solve("all_positive([...])", vec![
        tc(vec![list(&[1, 2, 3])], Value::Int(1)),
        tc(vec![list(&[1, -1, 2])], Value::Int(0)),
        tc(vec![list(&[0, 1])], Value::Int(0)),
    ], 128, 1000);

    solve("count_positives([...])", vec![
        tc(vec![list(&[1, -2, 3, -4, 5])], Value::Int(3)),
        tc(vec![list(&[-1, -2])], Value::Int(0)),
        tc(vec![list(&[1, 2, 3])], Value::Int(3)),
    ], 128, 1000);

    solve("any_negative([...])", vec![
        tc(vec![list(&[1, 2, 3])], Value::Int(0)),
        tc(vec![list(&[1, -1, 2])], Value::Int(1)),
        tc(vec![list(&[-5])], Value::Int(1)),
    ], 128, 1000);

    // =====================================================================
    // TIER 7: Multi-input list operations
    // =====================================================================
    println!("\n--- TIER 7: Multi-Input ---");

    solve("manhattan(a, b)", vec![
        tc(vec![list(&[1, 2, 3]), list(&[4, 2, 1])], Value::Int(5)),
        tc(vec![list(&[0, 0]), list(&[3, 4])], Value::Int(7)),
        tc(vec![list(&[5]), list(&[5])], Value::Int(0)),
    ], 256, 3000);

    solve("weighted_sum(vals, weights)", vec![
        tc(vec![list(&[1, 2, 3]), list(&[10, 20, 30])], Value::Int(140)),
        tc(vec![list(&[1, 1]), list(&[1, 1])], Value::Int(2)),
        tc(vec![list(&[5]), list(&[3])], Value::Int(15)),
    ], 256, 3000);

    // =====================================================================
    // TIER 8: The hard stuff
    // =====================================================================
    println!("\n--- TIER 8: Hard Problems ---");

    solve("is_sorted([...])", vec![
        tc(vec![list(&[1, 2, 3])], Value::Int(1)),
        tc(vec![list(&[3, 1, 2])], Value::Int(0)),
        tc(vec![list(&[1])], Value::Int(1)),
        tc(vec![list(&[5, 5, 5])], Value::Int(1)),
    ], 256, 3000);

    solve("second_largest([...])", vec![
        tc(vec![list(&[3, 1, 4, 1, 5])], Value::Int(4)),
        tc(vec![list(&[5, 5, 3])], Value::Int(5)),
        tc(vec![list(&[1, 2])], Value::Int(1)),
    ], 256, 3000);

    solve("poly_eval([1,2,3], x=2)", vec![
        tc(vec![list(&[1, 2, 3]), Value::Int(2)], Value::Int(17)),
        tc(vec![list(&[5, 0, 1]), Value::Int(3)], Value::Int(14)),
        tc(vec![list(&[1]), Value::Int(99)], Value::Int(1)),
    ], 256, 3000);

    solve("linear_search([...], target)", vec![
        tc(vec![list(&[1, 3, 5, 7, 9]), Value::Int(5)], Value::Int(2)),
        tc(vec![list(&[10, 20, 30]), Value::Int(30)], Value::Int(2)),
        tc(vec![list(&[1, 2, 3]), Value::Int(4)], Value::Int(-1)),
    ], 256, 3000);

    println!("\n{}", "=".repeat(80));
    println!("  End of proving ground. Everything above the wall is what IRIS can build.");
    println!("{}\n", "=".repeat(80));
}
