//! Integration tests for recursive improvement tracking (Phase Transition 3).
//!
//! Tests verify that the ImprovementTracker correctly computes improvement
//! rates, acceleration, operator attribution, adaptive weighting, and
//! problems-per-hour metrics. Also tests integration with the evolution
//! loop and the self-improving daemon.

use std::time::Duration;

use iris_evolve::auto_improve::AutoImproveConfig;
use iris_evolve::improvement_tracker::{ImprovementTracker, OperatorStats, operator_name};
use iris_evolve::self_improve::MUTATION_OPERATOR_NAMES;
use iris_evolve::self_improving_daemon::{
    ExecMode, SelfImprovingConfig, SelfImprovingDaemon,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Tiny daemon config for fast tests.
fn tiny_config() -> SelfImprovingConfig {
    SelfImprovingConfig {
        cycle_time_ms: 1,
        max_cycles: Some(20),
        improve_interval: 5,
        inspect_interval: 3,
        auto_improve: AutoImproveConfig {
            cycle_interval_secs: 0,
            max_slowdown: 100.0,
            test_cases_per_component: 3,
            evolution_generations: 3,
            evolution_pop_size: 4,
            gate_runs: 1,
            explore_problems: 1,
        },
        state_dir: None,
        memory_limit: 0,
        seed: Some(42),
        max_improve_threads: 2,
        max_stagnant: 5,
        min_improvement: 0.05,
        exec_mode: ExecMode::Continuous,
        trigger_check_interval: 100,
    }
}

// ---------------------------------------------------------------------------
// 1. test_improvement_rate_computed_correctly
// ---------------------------------------------------------------------------

#[test]
fn test_improvement_rate_computed_correctly() {
    let mut tracker = ImprovementTracker::with_window(10);

    // Feed data points with linearly decreasing latency:
    // latency = 10000 - 500*i  (slope should be -500 ns/cycle)
    for i in 0u64..10 {
        tracker.record_measurement("test_comp", i * 100, 10000 - 500 * i, 1.0);
    }

    let rate = tracker.improvement_rate("test_comp");
    assert!(rate.is_some(), "improvement rate should be computed with 10 data points");
    let rate_val = rate.unwrap();

    // The slope should be approximately -500 ns/cycle.
    assert!(
        (rate_val - (-500.0)).abs() < 1.0,
        "expected slope ~-500, got {:.2}",
        rate_val,
    );

    // Verify negative rate means latency is decreasing (improvement).
    assert!(rate_val < 0.0, "rate should be negative (latency decreasing)");
}

// ---------------------------------------------------------------------------
// 2. test_acceleration_positive_when_compounding
// ---------------------------------------------------------------------------

#[test]
fn test_acceleration_positive_when_compounding() {
    let mut tracker = ImprovementTracker::with_window(10);

    // Feed data with accelerating improvement: each cycle the latency drops
    // by an increasing amount.
    //
    // Latency at cycle i = 10000 - 10*i^2  (quadratic decrease)
    // First derivative (rate) gets more negative over time.
    // Second derivative (acceleration) should be negative (latency dropping
    // faster and faster).
    for i in 0u64..20 {
        let latency = 10000u64.saturating_sub(10 * i * i);
        tracker.record_measurement("accel_comp", i * 100, latency, 1.0);
    }

    let accel = tracker.acceleration("accel_comp");
    assert!(accel.is_some(), "acceleration should be computed with 20 data points");
    let accel_val = accel.unwrap();

    // Acceleration should be negative: the rate of latency reduction is
    // itself becoming more negative (i.e., improvement is speeding up).
    assert!(
        accel_val < 0.0,
        "acceleration should be negative for compounding improvement, got {:.4}",
        accel_val,
    );

    // is_compounding should be true.
    assert!(
        tracker.is_compounding(),
        "tracker should report compounding when acceleration < 0",
    );
}

// ---------------------------------------------------------------------------
// 3. test_acceleration_negative_at_plateau
// ---------------------------------------------------------------------------

#[test]
fn test_acceleration_negative_at_plateau() {
    let mut tracker = ImprovementTracker::with_window(10);

    // Feed data with decelerating improvement: improvement slows down.
    //
    // Latency at cycle i = 5000 + 1000 / (i+1)
    // The rate of latency decrease slows dramatically as i increases.
    // This should show positive acceleration (improvement rate becoming
    // less negative, approaching zero).
    for i in 0u64..20 {
        let latency = 5000 + 1000 / (i + 1);
        tracker.record_measurement("plateau_comp", i * 100, latency, 1.0);
    }

    let accel = tracker.acceleration("plateau_comp");
    assert!(accel.is_some(), "acceleration should be computed");
    let accel_val = accel.unwrap();

    // The rate of change is decelerating: the improvement rate is becoming
    // less negative (moving toward zero). This means acceleration > 0.
    assert!(
        accel_val > 0.0,
        "acceleration should be positive at plateau (improvement decelerating), got {:.6}",
        accel_val,
    );

    // is_compounding should be false (acceleration is positive, not negative).
    assert!(
        !tracker.is_compounding(),
        "tracker should NOT report compounding when improvement is decelerating",
    );
}

// ---------------------------------------------------------------------------
// 4. test_operator_attribution_tracks_correctly
// ---------------------------------------------------------------------------

#[test]
fn test_operator_attribution_tracks_correctly() {
    let mut tracker = ImprovementTracker::new();

    // Record specific operator applications.
    // replace_prim (id=4): 10 uses, 7 improvements
    for i in 0..10 {
        let delta = if i < 7 { 0.3 } else { -0.1 };
        tracker.record_operator_application(4, delta);
    }

    // insert_node (id=0): 10 uses, 2 improvements
    for i in 0..10 {
        let delta = if i < 2 { 0.1 } else { -0.2 };
        tracker.record_operator_application(0, delta);
    }

    // delete_node (id=1): 5 uses, 0 improvements
    for _ in 0..5 {
        tracker.record_operator_application(1, -0.05);
    }

    let contributions = tracker.operator_contributions();

    // Verify replace_prim stats.
    let rp = contributions.get("replace_prim").unwrap();
    assert_eq!(rp.times_used, 10, "replace_prim should have 10 uses");
    assert_eq!(rp.improvements_caused, 7, "replace_prim should have 7 improvements");
    assert!(
        (rp.total_fitness_delta - 2.1).abs() < 1e-6,
        "replace_prim total delta should be 2.1, got {}",
        rp.total_fitness_delta,
    );
    assert!(
        (rp.avg_improvement - 0.3).abs() < 1e-6,
        "replace_prim avg improvement should be 0.3, got {}",
        rp.avg_improvement,
    );

    // Verify insert_node stats.
    let in_ = contributions.get("insert_node").unwrap();
    assert_eq!(in_.times_used, 10);
    assert_eq!(in_.improvements_caused, 2);

    // Verify delete_node stats.
    let dn = contributions.get("delete_node").unwrap();
    assert_eq!(dn.times_used, 5);
    assert_eq!(dn.improvements_caused, 0);
    assert!((dn.total_fitness_delta - 0.0).abs() < 1e-10);

    // Sorted contributions should have replace_prim first.
    let sorted = tracker.operator_contributions_sorted();
    assert_eq!(sorted[0].0, "replace_prim", "replace_prim should be top contributor");
}

// ---------------------------------------------------------------------------
// 5. test_adaptive_weighting_increases_successful_operators
// ---------------------------------------------------------------------------

#[test]
fn test_adaptive_weighting_increases_successful_operators() {
    let mut tracker = ImprovementTracker::new();

    // Make swap_fold_op (id=13) very successful.
    for _ in 0..50 {
        tracker.record_operator_application(13, 0.8);
    }

    // Make insert_node (id=0) always fail.
    for _ in 0..50 {
        tracker.record_operator_application(0, -0.3);
    }

    // Make delete_node (id=1) mediocre (50% success, small improvements).
    for i in 0..50 {
        let delta = if i % 2 == 0 { 0.05 } else { -0.1 };
        tracker.record_operator_application(1, delta);
    }

    let weights = tracker.adaptive_weights(1.0 / 16.0, 10);

    let swap_fold_weight = weights.iter().find(|(n, _)| n == "swap_fold_op").unwrap().1;
    let insert_node_weight = weights.iter().find(|(n, _)| n == "insert_node").unwrap().1;
    let delete_node_weight = weights.iter().find(|(n, _)| n == "delete_node").unwrap().1;

    // swap_fold_op should have the highest weight.
    assert!(
        swap_fold_weight > delete_node_weight,
        "swap_fold_op ({:.4}) should have higher weight than delete_node ({:.4})",
        swap_fold_weight,
        delete_node_weight,
    );

    assert!(
        swap_fold_weight > insert_node_weight,
        "swap_fold_op ({:.4}) should have higher weight than insert_node ({:.4})",
        swap_fold_weight,
        insert_node_weight,
    );

    // insert_node (always fails) should have the lowest weight among these three.
    assert!(
        insert_node_weight < delete_node_weight,
        "insert_node ({:.4}) should have lower weight than delete_node ({:.4})",
        insert_node_weight,
        delete_node_weight,
    );

    // Weights should still sum to 1.0.
    let sum: f64 = weights.iter().map(|(_, w)| w).sum();
    assert!(
        (sum - 1.0).abs() < 1e-6,
        "weights should sum to 1.0, got {}",
        sum,
    );

    // Verify that the weights differ from uniform (initial) weights.
    let uniform = 1.0 / 16.0;
    let non_uniform_count = weights.iter().filter(|(_, w)| (*w - uniform).abs() > 0.001).count();
    assert!(
        non_uniform_count >= 3,
        "at least 3 weights should differ from uniform, got {}",
        non_uniform_count,
    );
}

// ---------------------------------------------------------------------------
// 6. test_problems_per_hour_increases
// ---------------------------------------------------------------------------

#[test]
fn test_problems_per_hour_increases() {
    let mut tracker = ImprovementTracker::new();

    // Simulate problems being solved at an increasing rate.
    // Phase 1: 5 problems in 1 hour (timestamps 0..3_600_000)
    for i in 0..5 {
        tracker.record_problem_solved(i * 720_000); // every 12 min
    }

    let rate_phase1 = tracker.problems_per_hour();
    assert!(
        rate_phase1 > 0.0,
        "problems per hour should be positive, got {}",
        rate_phase1,
    );

    // Phase 2: 20 more problems in the next hour (timestamps 3_600_000..7_200_000)
    for i in 0..20 {
        tracker.record_problem_solved(3_600_000 + i * 180_000); // every 3 min
    }

    let rate_phase2 = tracker.problems_per_hour();
    // Rate should increase because more problems are being solved overall.
    assert!(
        rate_phase2 > rate_phase1,
        "problems/hour should increase: phase1={:.1}, phase2={:.1}",
        rate_phase1,
        rate_phase2,
    );

    // Verify the rate is approximately correct.
    // Total: 25 problems over 2 hours = ~12.5/hour
    // But computed from first to last entry, last entry is at
    // 3_600_000 + 19*180_000 = 3_600_000 + 3_420_000 = 7_020_000
    // first entry at 0, so elapsed = 7_020_000 ms = 1.95 hours
    // problems = 24 (25 entries, delta = 24)
    // rate = 24 / 1.95 ~= 12.3
    assert!(
        rate_phase2 > 5.0 && rate_phase2 < 50.0,
        "problems/hour should be reasonable, got {}",
        rate_phase2,
    );
}

// ---------------------------------------------------------------------------
// 7. test_full_recursive_improvement_loop
// ---------------------------------------------------------------------------

#[test]
fn test_full_recursive_improvement_loop() {
    // Run the threaded daemon for 20 cycles with the improvement tracker active.
    let config = tiny_config();
    let mut daemon = SelfImprovingDaemon::new(config);
    let result = daemon.run();

    // Basic sanity: daemon ran.
    assert_eq!(result.cycles_completed, 20);

    // PT3 metrics should be present in the result.
    // improvement_rate and acceleration may be 0.0 if no components have
    // enough measurement history, but they should be valid f64 values.
    assert!(
        result.improvement_rate.is_finite(),
        "improvement_rate should be finite, got {}",
        result.improvement_rate,
    );
    assert!(
        result.acceleration.is_finite(),
        "acceleration should be finite, got {}",
        result.acceleration,
    );

    // is_compounding is a boolean, just verify it doesn't panic.
    let _ = result.is_compounding;

    // problems_solved should be non-negative.
    // (May be 0 if no problems were solved in 20 cycles.)
    assert!(result.problems_solved >= 0);

    // operator_contributions should have entries (even if all zeros).
    // The sorted list should contain entries for operators.
    assert!(
        !result.operator_contributions.is_empty(),
        "operator contributions should have at least one entry",
    );

    // Verify all 16 operators are present in contributions.
    let op_names: Vec<&str> = result
        .operator_contributions
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    for expected_op in &MUTATION_OPERATOR_NAMES {
        assert!(
            op_names.contains(expected_op),
            "operator '{}' should be in contributions",
            expected_op,
        );
    }

    // Verify the tracker is accessible from the daemon.
    let tracker = daemon.improvement_tracker();
    let summary = tracker.summary();
    assert!(
        summary.contains("Improvement Tracker Summary"),
        "summary should contain header",
    );
}

// ---------------------------------------------------------------------------
// 8. test_operator_name_mapping
// ---------------------------------------------------------------------------

#[test]
fn test_operator_name_mapping() {
    assert_eq!(operator_name(0), "insert_node");
    assert_eq!(operator_name(1), "delete_node");
    assert_eq!(operator_name(2), "rewire_edge");
    assert_eq!(operator_name(3), "replace_kind");
    assert_eq!(operator_name(4), "replace_prim");
    assert_eq!(operator_name(5), "mutate_literal");
    assert_eq!(operator_name(6), "duplicate_subgraph");
    assert_eq!(operator_name(7), "wrap_in_guard");
    assert_eq!(operator_name(8), "annotate_cost");
    assert_eq!(operator_name(9), "wrap_in_map");
    assert_eq!(operator_name(10), "wrap_in_filter");
    assert_eq!(operator_name(11), "compose_stages");
    assert_eq!(operator_name(12), "insert_zip");
    assert_eq!(operator_name(13), "swap_fold_op");
    assert_eq!(operator_name(14), "add_guard_condition");
    assert_eq!(operator_name(15), "extract_to_ref");
    assert_eq!(operator_name(255), "unknown");
}

// ---------------------------------------------------------------------------
// 9. test_operator_stats_success_rate
// ---------------------------------------------------------------------------

#[test]
fn test_operator_stats_success_rate() {
    let mut stats = OperatorStats::default();

    // Empty: success rate should be 0.
    assert!((stats.success_rate() - 0.0).abs() < 1e-10);

    // 4 improvements out of 10 uses = 40%.
    for i in 0..10 {
        let delta = if i < 4 { 0.2 } else { -0.1 };
        stats.record(delta);
    }

    assert_eq!(stats.times_used, 10);
    assert_eq!(stats.improvements_caused, 4);
    assert!(
        (stats.success_rate() - 0.4).abs() < 1e-10,
        "success rate should be 0.4, got {}",
        stats.success_rate(),
    );
    assert!(
        (stats.avg_improvement - 0.2).abs() < 1e-10,
        "avg improvement should be 0.2, got {}",
        stats.avg_improvement,
    );
}

// ---------------------------------------------------------------------------
// 10. test_install_adaptive_weights
// ---------------------------------------------------------------------------

#[test]
fn test_install_adaptive_weights() {
    let mut tracker = ImprovementTracker::new();

    // Not enough data: should not install.
    assert!(
        !tracker.install_adaptive_weights(10),
        "should not install without enough samples",
    );

    // Add enough data for one operator.
    for _ in 0..20 {
        tracker.record_operator_application(4, 0.5); // replace_prim
    }

    // Now it should install.
    assert!(
        tracker.install_adaptive_weights(10),
        "should install with enough samples",
    );

    // Clean up: clear custom weights.
    tracker.clear_adaptive_weights();
}

// ---------------------------------------------------------------------------
// 11. test_improvement_tracker_with_daemon
// ---------------------------------------------------------------------------

#[test]
fn test_improvement_tracker_with_daemon() {
    // Run a slightly longer daemon to accumulate some tracker data.
    let mut config = tiny_config();
    config.max_cycles = Some(30);
    let mut daemon = SelfImprovingDaemon::new(config);

    // Inject some latency data into metrics before running so the tracker
    // has something to measure.
    for i in 0..20 {
        daemon
            .metrics()
            .record_latency("mutation_insert_node", Duration::from_micros(50 - i));
        daemon
            .metrics()
            .record_correctness("mutation_insert_node", true);
    }

    let result = daemon.run();
    assert_eq!(result.cycles_completed, 30);

    // Check that the improvement tracker has at least one component's data.
    let tracker = daemon.improvement_tracker();
    let component_names = tracker.component_names();

    // The tracker may or may not have data depending on whether any
    // improvement outcomes occurred. Either way, the summary should work.
    let summary = tracker.summary();
    assert!(
        !summary.is_empty(),
        "tracker summary should not be empty",
    );

    // Verify the result's PT3 fields are populated.
    assert!(result.improvement_rate.is_finite());
    assert!(result.acceleration.is_finite());
}

// ---------------------------------------------------------------------------
// 12. test_multiple_component_tracking
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_component_tracking() {
    let mut tracker = ImprovementTracker::with_window(10);

    // Component A: steadily improving.
    for i in 0u64..15 {
        tracker.record_measurement("comp_a", i * 100, 5000 - i * 200, 1.0);
    }

    // Component B: stagnant (constant latency).
    for i in 0u64..15 {
        tracker.record_measurement("comp_b", i * 100, 3000, 0.9);
    }

    // Component C: getting worse (latency increasing).
    for i in 0u64..15 {
        tracker.record_measurement("comp_c", i * 100, 2000 + i * 100, 0.8);
    }

    // A should have negative improvement rate (improving).
    let rate_a = tracker.improvement_rate("comp_a").unwrap();
    assert!(rate_a < 0.0, "comp_a should be improving (rate < 0), got {}", rate_a);

    // B should have near-zero improvement rate (stagnant).
    let rate_b = tracker.improvement_rate("comp_b").unwrap();
    assert!(
        rate_b.abs() < 10.0,
        "comp_b should be stagnant (rate ~0), got {}",
        rate_b,
    );

    // C should have positive improvement rate (getting worse).
    let rate_c = tracker.improvement_rate("comp_c").unwrap();
    assert!(rate_c > 0.0, "comp_c should be worsening (rate > 0), got {}", rate_c);
}

// ---------------------------------------------------------------------------
// 13. test_mutate_tracked_returns_operator_id
// ---------------------------------------------------------------------------

#[test]
fn test_mutate_tracked_returns_operator_id() {
    use iris_types::cost::{CostBound, CostTerm};
    use iris_types::graph::{
        Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
    };
    use iris_types::hash::SemanticHash;
    use iris_types::types::TypeEnv;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use std::collections::{BTreeMap, HashMap};

    // Build a minimal graph.
    let mut nodes = HashMap::new();
    nodes.insert(
        NodeId(1),
        Node {
            id: NodeId(1),
            kind: NodeKind::Literal,
            type_sig: iris_types::types::TypeId(0),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Int(42),
        },
    );
    let graph = SemanticGraph {
        root: NodeId(1),
        nodes,
        edges: vec![],
        type_env: TypeEnv {
            types: BTreeMap::new(),
        },
        cost: CostBound::default(),
        resolution: Resolution::default(),
        hash: SemanticHash::default(),
    };

    let mut rng = StdRng::seed_from_u64(12345);

    // Call mutate_tracked multiple times and verify we get valid operator IDs.
    let mut operator_ids_seen = std::collections::HashSet::new();
    for _ in 0..100 {
        let (_mutated_graph, op_id) = iris_evolve::mutation::mutate_tracked(&graph, &mut rng);
        assert!(
            op_id <= 15,
            "operator ID should be 0..15, got {}",
            op_id,
        );
        operator_ids_seen.insert(op_id);
    }

    // With 100 trials, we should see multiple different operators.
    assert!(
        operator_ids_seen.len() >= 3,
        "should see at least 3 different operators in 100 mutations, saw {}",
        operator_ids_seen.len(),
    );
}

// ---------------------------------------------------------------------------
// 14. test_report_includes_compounding_status
// ---------------------------------------------------------------------------

#[test]
fn test_report_includes_compounding_status() {
    let config = tiny_config();
    let mut daemon = SelfImprovingDaemon::new(config);
    let _result = daemon.run();

    // All reports should have the is_compounding field.
    for report in daemon.reports() {
        // is_compounding is a bool, just verify it exists and doesn't panic.
        let _ = report.is_compounding;
    }
}
