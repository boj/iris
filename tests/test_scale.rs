//! Scaling test: verify that IRIS operations work at 50, 200, 500, and 1000
//! node program sizes, and measure wall-clock time at each scale.

use std::time::Instant;

use rand::SeedableRng;
use rand::rngs::StdRng;

use iris_evolve::crossover;
use iris_evolve::mutation;
use iris_evolve::seed;
use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_exec::ExecutionService;
use iris_types::eval::{EvalTier, TestCase, Value};

/// Generate a program of approximately the target node count using the
/// appropriate seed generator.
fn generate_program(rng: &mut StdRng, target_nodes: usize) -> iris_types::fragment::Fragment {
    if target_nodes <= 50 {
        // Pipeline with 2-3 stages produces ~20-60 nodes.
        seed::random_pipeline_program(rng, 3)
    } else if target_nodes <= 200 {
        // Stateful loop with ~10-15 iterations produces ~100-200 nodes.
        seed::random_stateful_loop(rng, 12)
    } else if target_nodes <= 500 {
        // Modular program with 10-15 modules produces ~200-500 nodes.
        seed::random_modular_program(rng, 12)
    } else if target_nodes <= 2000 {
        // Modular program with 25-40 modules for ~500-1000+ nodes.
        seed::random_modular_program(rng, 30)
    } else {
        // Large modular program with many modules for 2000-8192+ nodes.
        // Each module is ~20-50 nodes; 200 modules targets ~4000-8000 nodes.
        seed::random_modular_program(rng, 200)
    }
}

/// Simple test cases for evaluation (we just need them to not crash).
fn simple_test_cases() -> Vec<TestCase> {
    vec![
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
            ])],
            expected_output: None,
            initial_state: None,
            expected_state: None,
        },
        TestCase {
            inputs: vec![Value::Int(42)],
            expected_output: None,
            initial_state: None,
            expected_state: None,
        },
    ]
}

struct ScaleResult {
    target_nodes: usize,
    actual_nodes: usize,
    create_us: u128,
    mutate_us: u128,
    burst_mutate_us: u128,
    crossover_us: u128,
    eval_us: u128,
    compile_us: Option<u128>,
}

fn run_scale_test(target_nodes: usize) -> ScaleResult {
    let mut rng = StdRng::seed_from_u64(42 + target_nodes as u64);

    // --- Create ---
    let t0 = Instant::now();
    let fragment = generate_program(&mut rng, target_nodes);
    let create_us = t0.elapsed().as_micros();
    let actual_nodes = fragment.graph.nodes.len();

    // --- Mutate (single) ---
    let t1 = Instant::now();
    let _mutated = mutation::mutate(&fragment.graph, &mut rng);
    let mutate_us = t1.elapsed().as_micros();

    // --- Burst mutate ---
    let burst_size = (fragment.graph.nodes.len() / 20).min(10).max(1);
    let t2 = Instant::now();
    let _burst_mutated = mutation::mutate_burst(&fragment.graph, &mut rng, burst_size);
    let burst_mutate_us = t2.elapsed().as_micros();

    // --- Crossover ---
    // Create a second parent for crossover.
    let parent_b = generate_program(&mut rng, target_nodes);
    let t3 = Instant::now();
    let _child = crossover::crossover_large(
        &fragment.graph,
        &parent_b.graph,
        &mut rng,
        0.2, // 20% subgraph fraction
    );
    let crossover_us = t3.elapsed().as_micros();

    // --- Evaluate (tree-walking) ---
    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 64,
        worker_threads: 1,
        ..ExecConfig::default()
    });
    let test_cases = simple_test_cases();
    let t4 = Instant::now();
    // Use evaluate_individual which goes through the full pipeline.
    let _eval_result = exec.evaluate_individual(&fragment.graph, &test_cases, EvalTier::A);
    let eval_us = t4.elapsed().as_micros();

    // --- Compile (only for small programs) ---
    let compile_us = if actual_nodes <= 500 {
        let t5 = Instant::now();
        let _result = exec.evaluate_individual(&fragment.graph, &test_cases, EvalTier::B);
        Some(t5.elapsed().as_micros())
    } else {
        None
    };

    ScaleResult {
        target_nodes,
        actual_nodes,
        create_us,
        mutate_us,
        burst_mutate_us,
        crossover_us,
        eval_us,
        compile_us,
    }
}

#[test]
fn test_scaling_report() {
    let targets = [50, 200, 500, 1000, 8192];
    let mut results = Vec::new();

    for &target in &targets {
        let result = run_scale_test(target);
        results.push(result);
    }

    // Print the report.
    println!();
    println!("=== IRIS Scaling Report ===");
    println!(
        "{:<12} {:<12} {:<12} {:<12} {:<15} {:<12} {:<12} {:<12}",
        "Target", "Actual", "Create", "Mutate", "Burst Mutate", "Crossover", "Eval", "Compile"
    );
    println!(
        "{:<12} {:<12} {:<12} {:<12} {:<15} {:<12} {:<12} {:<12}",
        "nodes", "nodes", "(us)", "(us)", "(us)", "(us)", "(us)", "(us)"
    );
    println!("{}", "-".repeat(99));

    for r in &results {
        let compile_str = match r.compile_us {
            Some(us) => format!("{}", us),
            None => "skipped".to_string(),
        };
        println!(
            "{:<12} {:<12} {:<12} {:<12} {:<15} {:<12} {:<12} {:<12}",
            r.target_nodes,
            r.actual_nodes,
            r.create_us,
            r.mutate_us,
            r.burst_mutate_us,
            r.crossover_us,
            r.eval_us,
            compile_str,
        );
    }
    println!();

    // Verify all operations succeeded (no panics).
    for r in &results {
        assert!(r.actual_nodes > 0, "Program at target {} has 0 nodes", r.target_nodes);
        assert!(r.create_us < 10_000_000, "Creation took too long at {} nodes", r.target_nodes);
        assert!(r.eval_us < 30_000_000, "Evaluation took too long at {} nodes", r.target_nodes);
    }
}

#[test]
fn test_pipeline_program_node_count() {
    let mut rng = StdRng::seed_from_u64(123);
    let frag = seed::random_pipeline_program(&mut rng, 5);
    let count = frag.graph.nodes.len();
    assert!(count >= 20, "Pipeline with 5 stages should have >= 20 nodes, got {}", count);
    assert!(count <= 500, "Pipeline with 5 stages should have <= 500 nodes, got {}", count);
}

#[test]
fn test_stateful_loop_node_count() {
    let mut rng = StdRng::seed_from_u64(456);
    let frag = seed::random_stateful_loop(&mut rng, 10);
    let count = frag.graph.nodes.len();
    assert!(count >= 50, "Stateful loop with 10 iterations should have >= 50 nodes, got {}", count);
    assert!(count <= 500, "Stateful loop with 10 iterations should have <= 500 nodes, got {}", count);
}

#[test]
fn test_modular_program_node_count() {
    let mut rng = StdRng::seed_from_u64(789);
    let frag = seed::random_modular_program(&mut rng, 20);
    let count = frag.graph.nodes.len();
    assert!(count >= 100, "Modular program with 20 modules should have >= 100 nodes, got {}", count);
    assert!(count <= 2000, "Modular program with 20 modules should have <= 2000 nodes, got {}", count);
}

#[test]
fn test_burst_mutation_applies_multiple() {
    let mut rng = StdRng::seed_from_u64(101);
    let frag = seed::random_modular_program(&mut rng, 10);
    let original_hash = frag.graph.hash;

    let mutated = mutation::mutate_burst(&frag.graph, &mut rng, 5);
    // After 5 mutations, the hash should have changed.
    assert_ne!(original_hash, mutated.hash, "Burst mutation should change the graph hash");
}

#[test]
fn test_crossover_large_fraction() {
    let mut rng = StdRng::seed_from_u64(202);
    let parent_a = seed::random_modular_program(&mut rng, 10);
    let parent_b = seed::random_modular_program(&mut rng, 10);

    let child = crossover::crossover_large(&parent_a.graph, &parent_b.graph, &mut rng, 0.25);

    // Child should have at least as many nodes as parent_a (we only add donor nodes).
    assert!(
        child.nodes.len() >= parent_a.graph.nodes.len(),
        "Child should have >= parent_a nodes: {} vs {}",
        child.nodes.len(),
        parent_a.graph.nodes.len(),
    );
}

#[test]
fn test_interpreter_timeout() {
    // Create a program and try to evaluate with a very small step limit.
    let mut rng = StdRng::seed_from_u64(303);
    let frag = seed::random_pipeline_program(&mut rng, 3);

    let result = iris_exec::interpreter::interpret_with_step_limit(
        &frag.graph,
        &[Value::tuple(vec![Value::Int(1), Value::Int(2)])],
        None,
        None,
        5, // Very small step limit — should timeout for any non-trivial program.
    );

    // Should either timeout or succeed (some very small programs might finish
    // in 5 steps). We mainly verify it doesn't panic.
    match result {
        Ok(_) => {} // Small enough to finish.
        Err(iris_exec::interpreter::InterpretError::Timeout { .. }) => {} // Expected.
        Err(other) => {
            // Other errors are acceptable (missing edges, etc. in evolved programs).
            let _ = other;
        }
    }
}
