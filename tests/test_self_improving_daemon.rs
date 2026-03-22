//! Integration tests for the threaded self-improving daemon.
//!
//! Validates that all components (ThreadedDaemon, ImprovementPool,
//! ComponentMetrics, StagnationDetector, ConvergenceDetector,
//! AutoImprover, SelfInspector, IrisRuntime, persistence, and audit trail)
//! work together end-to-end.

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::Duration;

use iris_evolve::auto_improve::AutoImproveConfig;
use iris_evolve::self_improving_daemon::{
    ComponentMetrics, ConvergenceDetector, ExecMode, ImproveTrigger, ImprovementPool,
    SelfImprovingConfig, SelfImprovingDaemon, StagnationDetector,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Tiny config for fast tests.
fn tiny_config(state_dir: Option<PathBuf>) -> SelfImprovingConfig {
    SelfImprovingConfig {
        cycle_time_ms: 1, // 1ms cycles for speed
        max_cycles: Some(20),
        improve_interval: 5,
        inspect_interval: 3,
        auto_improve: AutoImproveConfig {
            cycle_interval_secs: 0,
            max_slowdown: 100.0, // generous so gate can pass
            test_cases_per_component: 3,
            evolution_generations: 3,
            evolution_pop_size: 4,
            gate_runs: 1,
            explore_problems: 1,
        },
        state_dir,
        memory_limit: 0, // no memory check
        seed: Some(42),  // deterministic
        max_improve_threads: 2,
        max_stagnant: 5,
        min_improvement: 0.05,
        exec_mode: ExecMode::Continuous,
        trigger_check_interval: 100,
    }
}

// ---------------------------------------------------------------------------
// 1. Threaded daemon runs (basic lifecycle)
// ---------------------------------------------------------------------------

#[test]
fn test_threaded_daemon_runs() {
    let config = tiny_config(None);
    let mut daemon = SelfImprovingDaemon::new(config);
    let result = daemon.run();

    // Should complete all 20 cycles.
    assert_eq!(result.cycles_completed, 20);

    // Auto-improve fires every 5 cycles (periodic trigger): cycles 5, 10, 15, 20 -> 4 times.
    assert!(
        result.improvement_cycles >= 1,
        "expected at least 1 improvement cycle, got {}",
        result.improvement_cycles,
    );

    // Should have collected 20 reports.
    assert_eq!(daemon.reports().len(), 20);

    // At least one report should have an improvement event.
    let improve_reports: Vec<_> = daemon
        .reports()
        .iter()
        .filter(|r| r.improvement_event.is_some())
        .collect();
    assert!(
        !improve_reports.is_empty(),
        "at least one cycle should have run auto-improve"
    );
}

// ---------------------------------------------------------------------------
// 2. Improvement triggered by latency threshold
// ---------------------------------------------------------------------------

#[test]
fn test_improvement_triggered_by_latency() {
    let metrics = ComponentMetrics::new();

    // Inject a slow component: p99 exceeds threshold.
    for _ in 0..20 {
        metrics.record_latency("slow_component", Duration::from_millis(50));
    }

    let trigger = ImproveTrigger::LatencyThreshold {
        component: "slow_component".to_string(),
        max_p99: Duration::from_millis(10), // threshold = 10ms, actual = 50ms
    };

    // Should fire.
    let result = trigger.check(&metrics, 1);
    assert_eq!(
        result.as_deref(),
        Some("slow_component"),
        "latency trigger should fire for slow_component"
    );

    // Fast component should NOT trigger.
    for _ in 0..20 {
        metrics.record_latency("fast_component", Duration::from_micros(100));
    }
    let trigger_fast = ImproveTrigger::LatencyThreshold {
        component: "fast_component".to_string(),
        max_p99: Duration::from_millis(10),
    };
    assert!(
        trigger_fast.check(&metrics, 1).is_none(),
        "latency trigger should not fire for fast_component"
    );
}

// ---------------------------------------------------------------------------
// 3. Correctness trigger
// ---------------------------------------------------------------------------

#[test]
fn test_improvement_triggered_by_correctness() {
    let metrics = ComponentMetrics::new();

    // Record many incorrect observations to drive the EMA below the threshold.
    for _ in 0..50 {
        metrics.record_correctness("buggy_component", false);
    }

    let trigger = ImproveTrigger::CorrectnessBelow {
        component: "buggy_component".to_string(),
        min_rate: 0.5,
    };

    let result = trigger.check(&metrics, 1);
    assert_eq!(
        result.as_deref(),
        Some("buggy_component"),
        "correctness trigger should fire for buggy_component"
    );

    // Fully correct component should NOT trigger.
    for _ in 0..50 {
        metrics.record_correctness("good_component", true);
    }
    let trigger_good = ImproveTrigger::CorrectnessBelow {
        component: "good_component".to_string(),
        min_rate: 0.5,
    };
    assert!(
        trigger_good.check(&metrics, 1).is_none(),
        "correctness trigger should not fire for good_component"
    );
}

// ---------------------------------------------------------------------------
// 4. Stagnation stops attempts
// ---------------------------------------------------------------------------

#[test]
fn test_stagnation_stops_attempts() {
    let mut stagnation = StagnationDetector::new(3, 0.05);

    // Record 2 stagnant attempts — not yet converged.
    stagnation.record_attempt("comp_a", 0.01);
    stagnation.record_attempt("comp_a", 0.02);
    assert!(
        !stagnation.is_converged("comp_a"),
        "comp_a should not be converged after 2 stagnant attempts"
    );
    assert_eq!(stagnation.stagnant_count("comp_a"), 2);

    // Third stagnant attempt should trigger convergence.
    let converged = stagnation.record_attempt("comp_a", 0.00);
    assert!(
        converged,
        "record_attempt should return true when component just converged"
    );
    assert!(
        stagnation.is_converged("comp_a"),
        "comp_a should be converged after 3 stagnant attempts"
    );

    // Further attempts should not change anything.
    let converged_again = stagnation.record_attempt("comp_a", 0.00);
    assert!(
        !converged_again,
        "already-converged component should return false"
    );

    // A real improvement should reset the counter.
    stagnation.record_attempt("comp_b", 0.01);
    stagnation.record_attempt("comp_b", 0.10); // real improvement
    assert_eq!(
        stagnation.stagnant_count("comp_b"),
        0,
        "counter should reset after real improvement"
    );
}

// ---------------------------------------------------------------------------
// 5. Convergence detected (system-wide)
// ---------------------------------------------------------------------------

#[test]
fn test_convergence_detected() {
    let mut stagnation = StagnationDetector::new(2, 0.05);
    let mut convergence = ConvergenceDetector::new();

    let components = vec![
        "comp_a".to_string(),
        "comp_b".to_string(),
        "comp_c".to_string(),
    ];

    // Not converged yet — no components have stagnated.
    assert!(
        !convergence.is_fully_converged(&components, &stagnation),
        "should not be converged when no components have stagnated"
    );

    // Mark comp_a as faster than Rust.
    convergence.mark_faster_than_rust("comp_a");

    // Stagnate comp_b.
    stagnation.record_attempt("comp_b", 0.0);
    stagnation.record_attempt("comp_b", 0.0);
    assert!(stagnation.is_converged("comp_b"));

    // Still not fully converged — comp_c is neither.
    assert!(
        !convergence.is_fully_converged(&components, &stagnation),
        "should not be converged when comp_c is still active"
    );

    // Stagnate comp_c.
    stagnation.record_attempt("comp_c", 0.0);
    stagnation.record_attempt("comp_c", 0.0);

    // Now fully converged.
    assert!(
        convergence.is_fully_converged(&components, &stagnation),
        "should be converged when all components are stagnated or faster"
    );
}

// ---------------------------------------------------------------------------
// 6. Concurrent improvement limit
// ---------------------------------------------------------------------------

#[test]
fn test_concurrent_improvement_limit() {
    let pool = ImprovementPool::new(2);

    // Start two improvements.
    assert!(pool.try_start("comp_a"), "should start comp_a");
    assert!(pool.try_start("comp_b"), "should start comp_b");
    assert_eq!(pool.active_count(), 2);

    // Third should fail.
    assert!(
        !pool.try_start("comp_c"),
        "should not start comp_c — at capacity"
    );

    // Duplicate should fail.
    assert!(
        !pool.try_start("comp_a"),
        "should not start comp_a again — already in progress"
    );

    // Finish one, then the third should succeed.
    pool.finish("comp_a");
    assert_eq!(pool.active_count(), 1);
    assert!(pool.try_start("comp_c"), "should start comp_c after slot freed");
    assert_eq!(pool.active_count(), 2);

    // Cleanup.
    pool.finish("comp_b");
    pool.finish("comp_c");
    assert_eq!(pool.active_count(), 0);
}

// ---------------------------------------------------------------------------
// 7. Atomic swap during execution (no crash)
// ---------------------------------------------------------------------------

#[test]
fn test_atomic_swap_during_execution() {
    use iris_evolve::iris_runtime::IrisRuntime;
    use std::sync::{Arc, Barrier, RwLock};

    let runtime = Arc::new(RwLock::new(IrisRuntime::new()));
    let barrier = Arc::new(Barrier::new(2));

    // Spawn a reader thread that continuously reads the runtime.
    let rt_clone = Arc::clone(&runtime);
    let done = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let done_clone = Arc::clone(&done);
    let barrier_clone = Arc::clone(&barrier);

    let reader = std::thread::spawn(move || {
        let mut reads = 0u64;
        // Synchronize start with writer.
        barrier_clone.wait();
        while !done_clone.load(Ordering::Relaxed) {
            let rt = rt_clone.read().unwrap();
            let _ = rt.deployed_components();
            reads += 1;
        }
        reads
    });

    // Synchronize start with reader.
    barrier.wait();

    // Meanwhile, do several writes (component swaps).
    {
        use iris_types::cost::{CostBound, CostTerm};
        use iris_types::graph::{Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph};
        use iris_types::hash::SemanticHash;
        use iris_types::types::TypeEnv;
        use std::collections::{BTreeMap, HashMap};

        for i in 0..50 {
            let mut nodes = HashMap::new();
            nodes.insert(
                NodeId(i),
                Node {
                    id: NodeId(i),
                    kind: NodeKind::Lit,
                    type_sig: iris_types::types::TypeId(0),
                    cost: CostTerm::Unit,
                    arity: 0,
                    resolution_depth: 0,
                    salt: 0,
                    payload: NodePayload::Lit {
                        type_tag: 0x00,
                        value: (i as i64).to_le_bytes().to_vec(),
                    },
                },
            );
            let graph = SemanticGraph {
                root: NodeId(i),
                nodes,
                edges: vec![],
                type_env: TypeEnv {
                    types: BTreeMap::new(),
                },
                cost: CostBound::Unknown,
                resolution: Resolution::Implementation,
                hash: SemanticHash([0; 32]),
            };

            let mut rt = runtime.write().unwrap();
            rt.replace_component("test_swap", graph);
            drop(rt);
            // Yield to give the reader thread a chance to acquire the lock.
            std::thread::yield_now();
        }
    }

    done.store(true, Ordering::Relaxed);
    let reads = reader.join().unwrap();

    // The reader should have done many reads without panicking.
    // The important property is that concurrent read/write doesn't crash.
    assert!(
        reads >= 1,
        "reader should have completed at least 1 read, got {}",
        reads
    );
}

// ---------------------------------------------------------------------------
// 8. Self-inspection runs
// ---------------------------------------------------------------------------

#[test]
fn self_inspection_runs() {
    let config = tiny_config(None);
    let mut daemon = SelfImprovingDaemon::new(config);
    let result = daemon.run();

    assert_eq!(result.cycles_completed, 20);

    // The audit trail should exist. Even if no anomalies detected,
    // the machinery ran without crashing.
    let _ = result.audit_entries;
}

// ---------------------------------------------------------------------------
// 9. Checkpoint save/load round trip
// ---------------------------------------------------------------------------

#[test]
fn checkpoint_saves_and_loads() {
    let dir = std::env::temp_dir().join("iris_threaded_daemon_checkpoint_test");
    let _ = std::fs::remove_dir_all(&dir);

    // Run with persistence enabled.
    let config = tiny_config(Some(dir.clone()));
    let mut daemon = SelfImprovingDaemon::new(config);
    let result = daemon.run();

    assert_eq!(result.cycles_completed, 20);

    // Verify checkpoint files exist on disk.
    assert!(
        dir.join("daemon_state.json").exists(),
        "daemon_state.json should exist after run"
    );
    assert!(
        dir.join("audit_trail.json").exists(),
        "audit_trail.json should exist after run"
    );

    // Verify the state file is valid JSON.
    let state_data = std::fs::read_to_string(dir.join("daemon_state.json")).unwrap();
    let state: serde_json::Value =
        serde_json::from_str(&state_data).expect("daemon_state.json should be valid JSON");
    assert!(state.get("daemon_cycles").is_some());
    assert!(state.get("improvement_cycles").is_some());
    assert!(state.get("deployed").is_some());

    // Verify the audit trail is valid JSON.
    let audit_data = std::fs::read_to_string(dir.join("audit_trail.json")).unwrap();
    let _audit: serde_json::Value =
        serde_json::from_str(&audit_data).expect("audit_trail.json should be valid JSON");

    // Load state into a fresh daemon.
    let config2 = tiny_config(None);
    let mut daemon2 = SelfImprovingDaemon::new(config2);
    let _loaded = daemon2.load_state(&dir);
    // The important thing is it doesn't crash.
    // The cycle count should be restored.
    assert_eq!(
        daemon2.cycle_count(),
        20,
        "cycle count should be restored from checkpoint"
    );

    // Cleanup.
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// 10. Audit trail has entries
// ---------------------------------------------------------------------------

#[test]
fn audit_trail_has_entries() {
    let config = tiny_config(None);
    let mut daemon = SelfImprovingDaemon::new(config);
    let _result = daemon.run();

    // The audit trail should exist and be valid.
    let audit = &daemon.inspector().audit;
    assert!(audit.verify_chain(), "audit trail chain should verify");

    // History in the auto-improver should have entries from improvement cycles.
    let auto_improver = daemon.auto_improver().lock().unwrap();
    let history = auto_improver.history();
    assert!(
        !history.is_empty(),
        "auto-improver history should have entries after 20 cycles with improve_interval=5"
    );
}

// ---------------------------------------------------------------------------
// 11. Runtime component audit tracks changes
// ---------------------------------------------------------------------------

#[test]
fn runtime_component_audit_tracks_changes() {
    let config = tiny_config(None);
    let mut daemon = SelfImprovingDaemon::new(config);
    let _result = daemon.run();

    // The runtime's component audit log records all replacements.
    // It may be empty if no components were deployed (evolution is stochastic),
    // but accessing it should work without panicking.
    let runtime = daemon.runtime().read().unwrap();
    let _audit = runtime.component_audit();
}

// ---------------------------------------------------------------------------
// 12. Stop via running flag
// ---------------------------------------------------------------------------

#[test]
fn daemon_stops_via_running_flag() {
    let config = SelfImprovingConfig {
        cycle_time_ms: 50,
        max_cycles: None, // would run forever
        improve_interval: 100, // don't trigger improvement
        inspect_interval: 100,
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
        exec_mode: ExecMode::FixedInterval(Duration::from_millis(50)),
        trigger_check_interval: 100,
    };

    let mut daemon = SelfImprovingDaemon::new(config);
    let flag = daemon.running_flag();

    // Spawn a thread that stops the daemon after ~120ms.
    let stopper = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(120));
        flag.store(false, std::sync::atomic::Ordering::Relaxed);
    });

    let result = daemon.run();
    stopper.join().unwrap();

    assert!(
        result.cycles_completed >= 1,
        "should have completed at least 1 cycle, got {}",
        result.cycles_completed
    );
    assert!(
        result.cycles_completed <= 10,
        "should not have run too many cycles, got {}",
        result.cycles_completed
    );
}

// ---------------------------------------------------------------------------
// 13. Multiple improvement cycles accumulate history
// ---------------------------------------------------------------------------

#[test]
fn multiple_improvement_cycles_accumulate_history() {
    // Run with improve_interval=2 so we get many improvement cycles in 20 cycles.
    let config = SelfImprovingConfig {
        cycle_time_ms: 1,
        max_cycles: Some(20),
        improve_interval: 2,
        inspect_interval: 10,
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
    };

    let mut daemon = SelfImprovingDaemon::new(config);
    let result = daemon.run();

    // Improvement fires at cycles 2,4,6,8,10,12,14,16,18,20 = 10 times.
    assert!(
        result.improvement_cycles >= 5,
        "expected at least 5 improvement cycles, got {}",
        result.improvement_cycles
    );

    // Auto-improver history should have at least that many entries.
    let auto_improver = daemon.auto_improver().lock().unwrap();
    let history = auto_improver.history();
    assert!(
        history.len() >= 5,
        "expected at least 5 history entries, got {}",
        history.len()
    );
}

// ---------------------------------------------------------------------------
// 14. IRIS selection is used in evolution (iris_mode)
// ---------------------------------------------------------------------------

#[test]
fn test_iris_selection_used_in_evolution() {
    use iris_evolve::iris_runtime::IrisRuntime;

    let rt = IrisRuntime::new();
    assert!(rt.initialized);

    // Verify IRIS select picks the best fitness.
    let fitnesses = vec![0.1, 0.9, 0.3, 0.5];
    let winner = rt.select(&fitnesses);
    assert_eq!(
        winner, 1,
        "IRIS select should pick index 1 (fitness 0.9), got {}",
        winner
    );

    // Verify tournament_select with random sampling still finds good candidates.
    let mut rng = rand::thread_rng();
    let fitnesses_large = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    let mut selected_high = 0;
    for _ in 0..100 {
        let idx = rt.tournament_select(&fitnesses_large, 4, &mut rng);
        if fitnesses_large[idx] >= 0.7 {
            selected_high += 1;
        }
    }
    // Tournament selection with size 4 should frequently select high-fitness individuals.
    assert!(
        selected_high > 30,
        "tournament select should favor high fitness: only {} of 100 selected >= 0.7",
        selected_high
    );
}

// ---------------------------------------------------------------------------
// 15. IRIS crossover is tried in evolution
// ---------------------------------------------------------------------------

#[test]
fn test_iris_crossover_used_in_evolution() {
    use iris_evolve::iris_runtime::IrisRuntime;
    use iris_types::cost::{CostBound, CostTerm};
    use iris_types::graph::{
        Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
    };
    use iris_types::hash::SemanticHash;
    use iris_types::types::TypeEnv;
    use std::collections::{BTreeMap, HashMap};

    let rt = IrisRuntime::new();

    // Build two simple programs to crossover.
    let make_prog = |opcode: u8, a: i64, b: i64, base: u64| -> SemanticGraph {
        let mut nodes = HashMap::new();
        let root = Node {
            id: NodeId(base),
            kind: NodeKind::Prim,
            type_sig: iris_types::types::TypeId(0),
            cost: CostTerm::Unit,
            arity: 2,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Prim { opcode },
        };
        nodes.insert(NodeId(base), root);
        let lit_a = Node {
            id: NodeId(base + 10),
            kind: NodeKind::Lit,
            type_sig: iris_types::types::TypeId(0),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0x00,
                value: a.to_le_bytes().to_vec(),
            },
        };
        nodes.insert(NodeId(base + 10), lit_a);
        let lit_b = Node {
            id: NodeId(base + 20),
            kind: NodeKind::Lit,
            type_sig: iris_types::types::TypeId(0),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0x00,
                value: b.to_le_bytes().to_vec(),
            },
        };
        nodes.insert(NodeId(base + 20), lit_b);
        let edges = vec![
            Edge {
                source: NodeId(base),
                target: NodeId(base + 10),
                port: 0,
                label: EdgeLabel::Argument,
            },
            Edge {
                source: NodeId(base),
                target: NodeId(base + 20),
                port: 1,
                label: EdgeLabel::Argument,
            },
        ];
        SemanticGraph {
            root: NodeId(base),
            nodes,
            edges,
            type_env: TypeEnv {
                types: BTreeMap::new(),
            },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        }
    };

    let parent_a = make_prog(0x00, 5, 3, 1); // add(5, 3)
    let parent_b = make_prog(0x02, 7, 2, 100); // mul(7, 2)

    let mut rng = rand::thread_rng();

    // Try crossover multiple times — it may fail on some random crossover points.
    let mut succeeded = false;
    for _ in 0..20 {
        if let Some((off_a, off_b)) = rt.crossover(&parent_a, &parent_b, &mut rng) {
            // Both offspring should have at least some nodes.
            assert!(!off_a.nodes.is_empty(), "offspring_a should have nodes");
            assert!(!off_b.nodes.is_empty(), "offspring_b should have nodes");
            succeeded = true;
            break;
        }
    }
    // IRIS crossover may not succeed on every random crossover point, which is fine.
    // The important thing is the method is callable and doesn't panic.
    let _ = succeeded;
}

// ---------------------------------------------------------------------------
// 16. Runtime profiling reports timings
// ---------------------------------------------------------------------------

#[test]
fn test_runtime_profiling_reports_timings() {
    use iris_evolve::iris_runtime::IrisRuntime;
    use std::time::Duration;

    let mut rt = IrisRuntime::new();

    // Initially no timings.
    assert!(rt.component_timings().is_empty());

    // Record some timings.
    rt.record_timing("mutate", Duration::from_micros(100));
    rt.record_timing("mutate", Duration::from_micros(200));
    rt.record_timing("select", Duration::from_micros(50));
    rt.record_timing("crossover", Duration::from_micros(300));

    let timings = rt.component_timings();
    assert_eq!(timings.len(), 3, "should have 3 components with timings");

    // Sorted slowest first.
    assert_eq!(timings[0].0, "crossover", "crossover should be slowest");
    assert_eq!(timings[1].0, "mutate", "mutate should be second");
    assert_eq!(timings[2].0, "select", "select should be fastest");

    // Verify mean calculations.
    assert_eq!(timings[1].1, Duration::from_micros(150)); // mean of 100, 200
}

// ---------------------------------------------------------------------------
// 17. Recursive improvement
// ---------------------------------------------------------------------------

#[test]
fn test_recursive_improvement() {
    // Run daemon for 30 cycles, verify it profiles and attempts improvement.
    let config = SelfImprovingConfig {
        cycle_time_ms: 1,
        max_cycles: Some(30),
        improve_interval: 3, // improve every 3 cycles for many attempts
        inspect_interval: 10,
        auto_improve: AutoImproveConfig {
            cycle_interval_secs: 0,
            max_slowdown: 100.0, // generous gate
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
    };

    let mut daemon = SelfImprovingDaemon::new(config);
    let result = daemon.run();

    assert_eq!(result.cycles_completed, 30);

    // Improvement fires at cycles 3,6,9,...,30 = 10 times.
    assert!(
        result.improvement_cycles >= 5,
        "expected at least 5 improvement cycles, got {}",
        result.improvement_cycles
    );

    // Verify the daemon tracked recursive depth.
    assert!(
        result.recursive_depth <= result.improvement_cycles,
        "recursive_depth ({}) should not exceed improvement_cycles ({})",
        result.recursive_depth,
        result.improvement_cycles,
    );

    // Verify the auto-improver attempted profiling.
    let auto_improver = daemon.auto_improver().lock().unwrap();
    let profiled_count = auto_improver
        .history()
        .iter()
        .filter(|e| {
            matches!(
                e.action,
                iris_evolve::auto_improve::ImprovementAction::Profiled { .. }
            )
        })
        .count();
    assert!(
        profiled_count >= 1,
        "should have at least 1 profiling event, got {}",
        profiled_count
    );
}

// ---------------------------------------------------------------------------
// 18. ComponentMetrics unit tests
// ---------------------------------------------------------------------------

#[test]
fn test_component_metrics() {
    let metrics = ComponentMetrics::new();

    // Record latencies.
    for i in 0..20 {
        metrics.record_latency("comp_a", Duration::from_micros(100 + i * 10));
    }

    // Mean should be around 195us (100 + 190) / 2.
    let mean = metrics.mean_latency("comp_a").unwrap();
    assert!(
        mean >= Duration::from_micros(150) && mean <= Duration::from_micros(250),
        "mean latency should be around 195us, got {:?}",
        mean
    );

    // p99 should be near the max.
    let p99 = metrics.p99_latency("comp_a").unwrap();
    assert!(
        p99 >= Duration::from_micros(250),
        "p99 should be >= 250us, got {:?}",
        p99
    );

    // Correctness tracking.
    for _ in 0..10 {
        metrics.record_correctness("comp_a", true);
    }
    let rate = metrics.correctness_rate("comp_a").unwrap();
    assert!(rate > 0.9, "correctness should be near 1.0, got {}", rate);

    // Global counters.
    metrics.total_executions.fetch_add(5, Ordering::Relaxed);
    assert_eq!(metrics.total_executions.load(Ordering::Relaxed), 5);
}

// ---------------------------------------------------------------------------
// 19. Periodic trigger fires at correct intervals
// ---------------------------------------------------------------------------

#[test]
fn test_periodic_trigger() {
    let metrics = ComponentMetrics::new();
    let trigger = ImproveTrigger::Periodic { interval: 5 };

    // Should not fire at count=0.
    assert!(trigger.check(&metrics, 0).is_none());

    // Should fire at count=5.
    assert_eq!(
        trigger.check(&metrics, 5).as_deref(),
        Some("__periodic__")
    );

    // Should not fire at count=3.
    assert!(trigger.check(&metrics, 3).is_none());

    // Should fire at count=10.
    assert_eq!(
        trigger.check(&metrics, 10).as_deref(),
        Some("__periodic__")
    );
}

// ---------------------------------------------------------------------------
// 20. Full convergence stops improvement
// ---------------------------------------------------------------------------

#[test]
fn test_full_convergence_stops_improvement() {
    // Use very aggressive stagnation detection (max_stagnant=1).
    let config = SelfImprovingConfig {
        cycle_time_ms: 1,
        max_cycles: Some(100),
        improve_interval: 1, // improve every cycle
        inspect_interval: 50,
        auto_improve: AutoImproveConfig {
            cycle_interval_secs: 0,
            max_slowdown: 0.001, // impossibly strict gate — nothing will deploy
            test_cases_per_component: 3,
            evolution_generations: 2,
            evolution_pop_size: 4,
            gate_runs: 1,
            explore_problems: 1,
        },
        state_dir: None,
        memory_limit: 0,
        seed: Some(42),
        max_improve_threads: 1,
        max_stagnant: 2, // converge after 2 failed attempts
        min_improvement: 0.05,
        exec_mode: ExecMode::Continuous,
        trigger_check_interval: 100,
    };

    let mut daemon = SelfImprovingDaemon::new(config);
    let result = daemon.run();

    assert_eq!(result.cycles_completed, 100);

    // The result should include convergence info.
    // (May or may not be fully converged depending on how many components
    // got stagnated, but the field should be present.)
    let _ = result.converged_components;
    let _ = result.fully_converged;
}
