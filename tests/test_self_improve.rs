//! Tests for IRIS self-improvement: evolving mutation weights, seed strategies,
//! and the self-improvement loop itself.
//!
//! All tests use tiny populations and few generations to stay fast.

use iris_evolve::config::ProblemSpec;
use iris_evolve::self_improve::{MutationStrategy, SeedStrategy, MUTATION_OPERATOR_NAMES};
use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_types::eval::{TestCase, Value};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Simple sum problem for benchmarking.
fn sum_problem() -> ProblemSpec {
    ProblemSpec {
        test_cases: vec![
            TestCase {
                inputs: vec![Value::tuple(vec![
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                ])],
                expected_output: Some(vec![Value::Int(6)]),
                initial_state: None,
                expected_state: None,
            },
            TestCase {
                inputs: vec![Value::tuple(vec![Value::Int(10)])],
                expected_output: Some(vec![Value::Int(10)]),
                initial_state: None,
                expected_state: None,
            },
        ],
        description: "Sum of integers".to_string(),
        target_cost: None,
    }
}

// ---------------------------------------------------------------------------
// MutationStrategy tests
// ---------------------------------------------------------------------------

#[test]
fn mutation_strategy_default_produces_valid_weights() {
    let strategy = MutationStrategy::default_strategy();

    // Should have exactly 16 operators.
    assert_eq!(strategy.weights.len(), 16);

    // All weights should be positive.
    for (name, w) in &strategy.weights {
        assert!(*w > 0.0, "Weight for {} should be positive, got {}", name, w);
    }

    // Weights should sum to approximately 1.0.
    let sum: f32 = strategy.weights.iter().map(|(_, w)| w).sum();
    assert!(
        (sum - 1.0).abs() < 0.01,
        "Weights should sum to ~1.0, got {}",
        sum
    );

    // Operator names should match the known set.
    for (i, (name, _)) in strategy.weights.iter().enumerate() {
        assert_eq!(
            name.as_str(),
            MUTATION_OPERATOR_NAMES[i],
            "Operator name mismatch at index {}",
            i
        );
    }
}

#[test]
fn mutation_strategy_perturb_produces_different_weights() {
    let mut rng = StdRng::seed_from_u64(42);
    let original = MutationStrategy::default_strategy();
    let perturbed = original.perturb(&mut rng);

    // Should have same number of operators.
    assert_eq!(perturbed.weights.len(), original.weights.len());

    // Weights should still sum to ~1.0.
    let sum: f32 = perturbed.weights.iter().map(|(_, w)| w).sum();
    assert!(
        (sum - 1.0).abs() < 0.01,
        "Perturbed weights should sum to ~1.0, got {}",
        sum
    );

    // At least some weights should have changed.
    let mut any_different = false;
    for ((_, w1), (_, w2)) in original.weights.iter().zip(perturbed.weights.iter()) {
        if (w1 - w2).abs() > 0.001 {
            any_different = true;
            break;
        }
    }
    assert!(any_different, "Perturbed strategy should differ from original");

    // All weights should still be positive.
    for (name, w) in &perturbed.weights {
        assert!(
            *w > 0.0,
            "Perturbed weight for {} should be positive, got {}",
            name, w
        );
    }
}

#[test]
fn mutation_strategy_cumulative_thresholds_are_valid() {
    let strategy = MutationStrategy::default_strategy();
    let thresholds = strategy.to_cumulative_thresholds();

    // Should have 16 entries.
    assert_eq!(thresholds.len(), 16);

    // Should be monotonically increasing.
    for i in 1..thresholds.len() {
        assert!(
            thresholds[i].1 >= thresholds[i - 1].1,
            "Cumulative thresholds should be monotonically increasing"
        );
    }

    // Last threshold should be 1.0.
    assert!(
        (thresholds.last().unwrap().1 - 1.0).abs() < 0.001,
        "Last cumulative threshold should be 1.0"
    );
}

#[test]
fn mutation_strategy_random_produces_valid_weights() {
    let mut rng = StdRng::seed_from_u64(123);
    let strategy = MutationStrategy::random(&mut rng);

    assert_eq!(strategy.weights.len(), 16);

    let sum: f32 = strategy.weights.iter().map(|(_, w)| w).sum();
    assert!(
        (sum - 1.0).abs() < 0.01,
        "Random strategy weights should sum to ~1.0, got {}",
        sum
    );

    for (_, w) in &strategy.weights {
        assert!(*w > 0.0, "All weights should be positive");
    }
}

// ---------------------------------------------------------------------------
// SeedStrategy tests
// ---------------------------------------------------------------------------

#[test]
fn seed_strategy_default_produces_valid_weights() {
    let strategy = SeedStrategy::default_strategy();

    // Should have 13 seed types.
    assert_eq!(strategy.seed_weights.len(), 13);

    // All weights should be positive.
    for (name, w) in &strategy.seed_weights {
        assert!(*w > 0.0, "Seed weight for {} should be positive", name);
    }

    // Weights should sum to approximately 1.0 (proportional to slot coverage
    // in the modulo-30 distribution; the 13 types cover 26 of 30 slots).
    let sum: f32 = strategy.seed_weights.iter().map(|(_, w)| w).sum();
    assert!(
        sum > 0.5 && sum <= 1.01,
        "Seed weights should sum to a reasonable value, got {}",
        sum
    );
}

#[test]
fn seed_strategy_perturb_produces_different_weights() {
    let mut rng = StdRng::seed_from_u64(99);
    let original = SeedStrategy::default_strategy();
    let perturbed = original.perturb(&mut rng);

    assert_eq!(perturbed.seed_weights.len(), original.seed_weights.len());

    let sum: f32 = perturbed.seed_weights.iter().map(|(_, w)| w).sum();
    assert!(
        (sum - 1.0).abs() < 0.01,
        "Perturbed seed weights should sum to ~1.0, got {}",
        sum
    );

    let mut any_different = false;
    for ((_, w1), (_, w2)) in original
        .seed_weights
        .iter()
        .zip(perturbed.seed_weights.iter())
    {
        if (w1 - w2).abs() > 0.001 {
            any_different = true;
            break;
        }
    }
    assert!(
        any_different,
        "Perturbed seed strategy should differ from original"
    );
}

#[test]
fn seed_strategy_select_covers_all_types() {
    let mut rng = StdRng::seed_from_u64(77);
    let strategy = SeedStrategy::default_strategy();

    let mut seen = vec![false; 13];
    for _ in 0..2000 {
        let idx = strategy.select_seed_type(&mut rng);
        assert!(idx < 13, "Seed type index out of range: {}", idx);
        seen[idx] = true;
    }

    // With 2000 samples and 13 types, all should be covered.
    for (i, &was_seen) in seen.iter().enumerate() {
        assert!(was_seen, "Seed type {} was never selected", i);
    }
}

// ---------------------------------------------------------------------------
// Mutation strategy evaluation (fast, trivial problem)
// ---------------------------------------------------------------------------

#[test]
fn mutation_strategy_evaluate_runs_without_crash() {
    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 64,
        worker_threads: 1,
        ..ExecConfig::default()
    });

    let problems = vec![sum_problem()];
    let strategy = MutationStrategy::default_strategy();

    // Tiny eval: 3 generations, pop 4, 500ms timeout.
    let score = strategy.evaluate(
        &problems,
        &exec,
        3,
        4,
        Duration::from_millis(500),
    );

    // Score should be in [0.0, 1.0].
    assert!(
        (0.0..=1.0).contains(&score),
        "Evaluation score should be in [0.0, 1.0], got {}",
        score
    );
}

// ---------------------------------------------------------------------------
// Self-improvement loop (integration test)
// ---------------------------------------------------------------------------

#[test]
fn self_improve_loop_runs_and_returns_results() {
    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 64,
        worker_threads: 1,
        ..ExecConfig::default()
    });

    let problems = vec![sum_problem()];

    // Run 1 round of self-improvement with minimal parameters.
    let results = iris_evolve::self_improve::self_improve_loop(
        problems,
        &exec,
        1,                            // 1 round
        Duration::from_secs(10),      // budget per round
        2,                            // 2 meta-evolution generations
        4,                            // 4 strategies in meta-population
        3,                            // 3 sub-evolution generations
        4,                            // 4 individuals in sub-evolution
    );

    assert_eq!(results.len(), 1, "Should return exactly 1 result for 1 round");

    let result = &results[0];
    assert!(
        result.baseline_solve_rate >= 0.0 && result.baseline_solve_rate <= 1.0,
        "Baseline solve rate should be in [0.0, 1.0]"
    );
    assert!(
        result.improved_solve_rate >= 0.0 && result.improved_solve_rate <= 1.0,
        "Improved solve rate should be in [0.0, 1.0]"
    );
    assert_eq!(
        result.mutation_strategy.weights.len(),
        16,
        "Evolved mutation strategy should have 16 operators"
    );
    assert_eq!(
        result.seed_strategy.seed_weights.len(),
        13,
        "Evolved seed strategy should have 11 seed types"
    );
}

#[test]
fn self_improve_loop_public_api_runs() {
    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 64,
        worker_threads: 1,
        ..ExecConfig::default()
    });

    let problems = vec![sum_problem()];

    // Use the public API from lib.rs.
    let results = iris_evolve::self_improve_loop(problems, &exec, 1);

    assert!(!results.is_empty(), "Should return at least one result");
    let result = &results[0];
    assert_eq!(result.mutation_strategy.weights.len(), 16);
}

// ---------------------------------------------------------------------------
// Custom weights thread-local mechanism
// ---------------------------------------------------------------------------

#[test]
fn custom_mutation_weights_are_applied_and_cleared() {
    use iris_evolve::mutation;

    // Install custom weights that heavily favor operator 0 (insert_node).
    let custom: Vec<(u8, f64)> = vec![
        (0, 0.99),
        (1, 0.991),
        (2, 0.992),
        (3, 0.993),
        (4, 0.994),
        (5, 0.995),
        (6, 0.996),
        (7, 0.997),
        (8, 0.998),
        (9, 0.9985),
        (10, 0.999),
        (11, 0.9993),
        (12, 0.9996),
        (13, 0.9998),
        (14, 0.9999),
        (15, 1.0),
    ];
    mutation::set_custom_weights(&custom);

    // Create a tiny graph and mutate it many times.
    // With 99% weight on insert_node, the graph should grow.
    let seed = iris_evolve::seed::identity_program();
    let mut rng = StdRng::seed_from_u64(42);
    let initial_nodes = seed.graph.nodes.len();

    let mut grew = false;
    for _ in 0..20 {
        let mutated = mutation::mutate(&seed.graph, &mut rng);
        if mutated.nodes.len() > initial_nodes {
            grew = true;
            break;
        }
    }

    // Clean up.
    mutation::clear_custom_weights();

    assert!(grew, "With 99% insert_node weight, graph should have grown");
}
