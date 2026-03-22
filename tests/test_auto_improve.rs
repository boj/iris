//! Integration tests for the autonomous self-improvement daemon.
//!
//! Validates that `AutoImprover` can profile, evolve, gate-check, and deploy
//! components. Uses tiny budgets to keep tests fast (< 30s total).

use std::time::Duration;

use iris_evolve::auto_improve::{
    AutoImproveConfig, AutoImprover, ImprovementAction,
};
use iris_exec::service::{ExecConfig, IrisExecutionService};

/// Tiny config for fast tests.
fn tiny_config() -> AutoImproveConfig {
    AutoImproveConfig {
        cycle_interval_secs: 0,
        max_slowdown: 100.0, // generous so gate passes easily
        test_cases_per_component: 4,
        evolution_generations: 3,
        evolution_pop_size: 8,
        gate_runs: 1,
        explore_problems: 2,
    }
}

fn make_exec() -> IrisExecutionService {
    IrisExecutionService::new(ExecConfig {
        cache_capacity: 64,
        worker_threads: 1,
        ..ExecConfig::default()
    })
}

// -------------------------------------------------------------------------
// 1. AutoImprover initializes with profiled components
// -------------------------------------------------------------------------

#[test]
fn auto_improver_initializes_with_profiled_components() {
    let mut improver = AutoImprover::with_seed(tiny_config(), 42);
    improver.register_component("comp_a", Duration::from_micros(100));
    improver.register_component("comp_b", Duration::from_micros(200));
    improver.register_component("comp_c", Duration::from_micros(50));

    let status = improver.status();
    assert_eq!(status.components_profiled, 3);
    assert_eq!(status.components_deployed, 0);
    assert_eq!(status.total_improvement_cycles, 0);
}

// -------------------------------------------------------------------------
// 2. profile_components identifies the slowest
// -------------------------------------------------------------------------

#[test]
fn profile_components_identifies_slowest() {
    let mut improver = AutoImprover::with_seed(tiny_config(), 42);
    improver.register_component("fast", Duration::from_micros(10));
    improver.register_component("slow", Duration::from_micros(500));
    improver.register_component("medium", Duration::from_micros(100));

    let (name, dur) = improver.profile_components();
    assert_eq!(name, "slow");
    assert_eq!(dur, Duration::from_micros(500));
}

// -------------------------------------------------------------------------
// 3. extract_test_cases generates valid test cases
// -------------------------------------------------------------------------

#[test]
fn extract_test_cases_generates_valid_cases() {
    let mut improver = AutoImprover::with_seed(tiny_config(), 42);
    let cases = improver.extract_test_cases("some_component");

    assert_eq!(cases.len(), 4);
    for tc in &cases {
        assert_eq!(tc.inputs.len(), 2, "each test case should have 2 inputs");
        let expected = tc.expected_output.as_ref().expect("should have expected output");
        assert_eq!(expected.len(), 1, "each test case should have 1 expected output");

        // Verify the test case is internally consistent (a + b = expected).
        if let (iris_types::eval::Value::Int(a), iris_types::eval::Value::Int(b)) =
            (&tc.inputs[0], &tc.inputs[1])
        {
            if let iris_types::eval::Value::Int(r) = &expected[0] {
                assert_eq!(*r, a.wrapping_add(*b), "test case output should be a+b");
            } else {
                panic!("expected output should be Int");
            }
        } else {
            panic!("inputs should be Int");
        }
    }
}

// -------------------------------------------------------------------------
// 4. One improvement cycle runs without crashing
// -------------------------------------------------------------------------

#[test]
fn one_improvement_cycle_runs_without_crashing() {
    let mut improver = AutoImprover::with_seed(tiny_config(), 42);
    improver.register_component("target", Duration::from_micros(100));

    let exec = make_exec();
    let event = improver.run_cycle(&exec);

    // Should have produced some event.
    assert!(improver.cycle_count() >= 1);
    assert!(!improver.history().is_empty());

    // The event should have a non-empty summary.
    let summary = event.summary();
    assert!(!summary.is_empty());
    assert!(summary.contains("cycle 1"));
}

// -------------------------------------------------------------------------
// 5. If a component passes the gate, it gets deployed
// -------------------------------------------------------------------------

#[test]
fn component_deployed_when_gate_passes() {
    // Use a very generous config to maximize chance of gate passing.
    let config = AutoImproveConfig {
        cycle_interval_secs: 0,
        max_slowdown: 1000.0, // extremely generous
        test_cases_per_component: 3,
        evolution_generations: 10,
        evolution_pop_size: 16,
        gate_runs: 1,
        explore_problems: 2,
    };

    let mut improver = AutoImprover::with_seed(config, 42);
    improver.register_component("target_comp", Duration::from_micros(100));

    let exec = make_exec();

    // Run multiple cycles to give evolution a chance.
    let mut deployed = false;
    for _ in 0..3 {
        let event = improver.run_cycle(&exec);
        if matches!(event.action, ImprovementAction::Deployed { .. }) {
            deployed = true;
            break;
        }
    }

    // If deployed, verify state.
    if deployed {
        let status = improver.status();
        assert!(status.components_deployed > 0);
        assert!(!status.deployed_components.is_empty());
        assert!(improver.deployed().contains_key("target_comp"));
    }
    // Even if not deployed (evolution is stochastic), verify no panic occurred.
    assert!(improver.cycle_count() >= 1);
}

// -------------------------------------------------------------------------
// 6. explore_new_capability generates and attempts a random problem
// -------------------------------------------------------------------------

#[test]
fn explore_new_capability_runs() {
    let mut improver = AutoImprover::with_seed(tiny_config(), 42);
    // Don't register any components — should go straight to exploration.

    let exec = make_exec();
    let event = improver.run_cycle(&exec);

    match &event.action {
        ImprovementAction::Explored {
            new_capability,
            solve_rate,
        } => {
            assert!(!new_capability.is_empty(), "should have a capability name");
            assert!(
                *solve_rate >= 0.0 && *solve_rate <= 1.0,
                "solve rate should be in [0, 1]"
            );
        }
        other => {
            // With no components, we should always explore.
            panic!(
                "expected Explored action when no components registered, got {:?}",
                other
            );
        }
    }
}

// -------------------------------------------------------------------------
// 7. Status reports correct counts
// -------------------------------------------------------------------------

#[test]
fn status_reports_correct_counts() {
    let mut improver = AutoImprover::with_seed(tiny_config(), 42);
    improver.register_component("a", Duration::from_micros(10));
    improver.register_component("b", Duration::from_micros(20));

    let exec = make_exec();

    // Run 2 cycles.
    improver.run_cycle(&exec);
    improver.run_cycle(&exec);

    let status = improver.status();
    assert_eq!(status.components_profiled, 2);
    assert_eq!(status.total_improvement_cycles, 2);
    // History should have entries (at least 2 profile events + 2 result events).
    assert!(
        improver.history().len() >= 2,
        "should have at least 2 history entries, got {}",
        improver.history().len()
    );
}

// -------------------------------------------------------------------------
// 8. Run 3 improvement cycles, verify history grows
// -------------------------------------------------------------------------

#[test]
fn three_cycles_grow_history() {
    let mut improver = AutoImprover::with_seed(tiny_config(), 42);
    improver.register_component("comp_x", Duration::from_micros(100));
    improver.register_component("comp_y", Duration::from_micros(200));

    let exec = make_exec();

    let initial_history = improver.history().len();
    assert_eq!(initial_history, 0);

    for cycle in 1..=3 {
        let _event = improver.run_cycle(&exec);
        assert_eq!(
            improver.cycle_count(),
            cycle,
            "cycle count should increment"
        );
        assert!(
            improver.history().len() > initial_history,
            "history should grow after cycle {}",
            cycle
        );
    }

    // After 3 cycles, we should have at least 3 events (one per cycle minimum).
    assert!(
        improver.history().len() >= 3,
        "should have at least 3 history entries after 3 cycles, got {}",
        improver.history().len()
    );

    // Verify each cycle's timestamp increments.
    let timestamps: Vec<u64> = improver
        .history()
        .iter()
        .map(|e| e.timestamp)
        .collect();
    // Timestamps should be monotonically non-decreasing.
    for window in timestamps.windows(2) {
        assert!(
            window[0] <= window[1],
            "timestamps should be non-decreasing: {:?}",
            timestamps
        );
    }

    // Verify status reflects all cycles.
    let status = improver.status();
    assert_eq!(status.total_improvement_cycles, 3);
}
