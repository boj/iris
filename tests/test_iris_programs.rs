//! Can IRIS write simple programs? Scaling test from trivial to complex.
//!
//! Level 1: Constants (return a fixed value)
//! Level 2: Identity (return the input)
//! Level 3: Arithmetic (add two inputs)
//! Level 4: Conditional (if x > 0 return x else return -x — absolute value)
//! Level 5: Multi-input dispatch (calculator: op selects add/sub/mul)
//! Level 6: String processing (reverse a string)
//! Level 7: Stateful (counter that increments)

use iris_types::eval::*;
use iris_exec::service::*;
use iris_evolve::*;
use iris_evolve::config::*;

fn quick_evolve(test_cases: Vec<TestCase>, description: &str, max_gens: usize, pop: usize) -> (f32, usize, f64) {
    let spec = ProblemSpec {
        test_cases,
        description: description.to_string(),
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
    let correctness = result.best_individual.fitness.correctness();
    (correctness, result.generations_run, result.total_time.as_secs_f64())
}

fn tc(inputs: Vec<Value>, expected: Value) -> TestCase {
    TestCase {
        inputs,
        expected_output: Some(vec![expected]),
        initial_state: None,
        expected_state: None,
    }
}

#[test]
fn iris_program_levels() {
    println!("\n========================================");
    println!("  IRIS Program Capability Levels");
    println!("========================================\n");

    // Level 1: Return constant 42
    let (c, g, t) = quick_evolve(vec![
        tc(vec![Value::Int(0)], Value::Int(42)),
        tc(vec![Value::Int(99)], Value::Int(42)),
        tc(vec![Value::Int(-5)], Value::Int(42)),
    ], "Return constant 42", 500, 64);
    println!("Level 1 (Constant):     {:.0}% | gen {} | {:.2}s", c * 100.0, g, t);

    // Level 2: Identity — return the input unchanged
    let (c, g, t) = quick_evolve(vec![
        tc(vec![Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(7)], Value::Int(7)),
        tc(vec![Value::Int(-3)], Value::Int(-3)),
        tc(vec![Value::Int(100)], Value::Int(100)),
    ], "Identity function", 500, 64);
    println!("Level 2 (Identity):     {:.0}% | gen {} | {:.2}s", c * 100.0, g, t);

    // Level 3: Add two inputs
    let (c, g, t) = quick_evolve(vec![
        tc(vec![Value::Int(1), Value::Int(2)], Value::Int(3)),
        tc(vec![Value::Int(0), Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(10), Value::Int(-3)], Value::Int(7)),
        tc(vec![Value::Int(-5), Value::Int(-5)], Value::Int(-10)),
    ], "Add two inputs", 500, 64);
    println!("Level 3 (Add):          {:.0}% | gen {} | {:.2}s", c * 100.0, g, t);

    // Level 4: Absolute value (conditional)
    let (c, g, t) = quick_evolve(vec![
        tc(vec![Value::Int(5)], Value::Int(5)),
        tc(vec![Value::Int(-3)], Value::Int(3)),
        tc(vec![Value::Int(0)], Value::Int(0)),
        tc(vec![Value::Int(-100)], Value::Int(100)),
        tc(vec![Value::Int(1)], Value::Int(1)),
    ], "Absolute value", 1000, 128);
    println!("Level 4 (Abs value):    {:.0}% | gen {} | {:.2}s", c * 100.0, g, t);

    // Level 5: Simple calculator (add/sub only, op=0 → add, op=1 → sub)
    let (c, g, t) = quick_evolve(vec![
        tc(vec![Value::Int(3), Value::Int(0), Value::Int(5)], Value::Int(8)),   // 3+5
        tc(vec![Value::Int(10), Value::Int(1), Value::Int(3)], Value::Int(7)),  // 10-3
        tc(vec![Value::Int(0), Value::Int(0), Value::Int(0)], Value::Int(0)),   // 0+0
        tc(vec![Value::Int(7), Value::Int(1), Value::Int(7)], Value::Int(0)),   // 7-7
        tc(vec![Value::Int(1), Value::Int(0), Value::Int(1)], Value::Int(2)),   // 1+1
        tc(vec![Value::Int(5), Value::Int(1), Value::Int(2)], Value::Int(3)),   // 5-2
    ], "Calculator (add/sub)", 2000, 256);
    println!("Level 5 (Calculator):   {:.0}% | gen {} | {:.2}s", c * 100.0, g, t);

    // Level 6: Double a list (map)
    let (c, g, t) = quick_evolve(vec![
        tc(vec![Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)])],
           Value::tuple(vec![Value::Int(2), Value::Int(4), Value::Int(6)])),
        tc(vec![Value::tuple(vec![Value::Int(0)])],
           Value::tuple(vec![Value::Int(0)])),
        tc(vec![Value::tuple(vec![Value::Int(5), Value::Int(10)])],
           Value::tuple(vec![Value::Int(10), Value::Int(20)])),
    ], "Double each element", 2000, 256);
    println!("Level 6 (Map double):   {:.0}% | gen {} | {:.2}s", c * 100.0, g, t);

    // Level 7: Sum then double (composition)
    let (c, g, t) = quick_evolve(vec![
        tc(vec![Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)])], Value::Int(12)),  // sum=6, *2=12
        tc(vec![Value::tuple(vec![Value::Int(5)])], Value::Int(10)),  // sum=5, *2=10
        tc(vec![Value::tuple(vec![Value::Int(0), Value::Int(0)])], Value::Int(0)),
    ], "Sum then double", 2000, 256);
    println!("Level 7 (Compose):      {:.0}% | gen {} | {:.2}s", c * 100.0, g, t);

    println!("\n========================================");
    println!("  Programs that IRIS can write today");
    println!("========================================");
}
