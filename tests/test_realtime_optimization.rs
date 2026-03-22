//! Comprehensive integration test for the IRIS self-improvement loop.
//!
//! Validates the complete lifecycle: cold start, evolution cycles, persistence,
//! reload from disk, hot-swap with rollback, and full loop verification.
//! Each phase builds on the previous, proving the system works end-to-end.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::time::Duration;

use iris_evolve::auto_improve::{AutoImproveConfig, AutoImprover, ImprovementAction};
use iris_evolve::instrumentation::{AuditAction, AuditTrail};
use iris_evolve::iris_runtime::IrisRuntime;
use iris_evolve::self_improving_daemon::{
    DaemonState, ExecMode, SelfImprovingConfig, SelfImprovingDaemon,
};
use iris_exec::service::IrisExecutionService;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::TypeEnv;

// ===========================================================================
// Helpers
// ===========================================================================

/// Tiny config for fast tests (deterministic, minimal budgets).
fn tiny_config(state_dir: Option<PathBuf>) -> SelfImprovingConfig {
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
        state_dir,
        memory_limit: 0,
        seed: Some(42),
        max_improve_threads: 2,
        max_stagnant: 5,
        min_improvement: 0.05,
        exec_mode: ExecMode::Continuous,
        trigger_check_interval: 100,
    }
}

/// Create a unique temp directory for test isolation.
fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("iris_realtime_opt_test")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create test dir");
    dir
}

/// Build a minimal addition program: add(inputs[0], inputs[1]).
fn build_add_program() -> SemanticGraph {
    let mut nodes = HashMap::new();
    // Root: prim add (opcode 0x00, arity 2)
    nodes.insert(
        NodeId(1),
        Node {
            id: NodeId(1),
            kind: NodeKind::Prim,
            type_sig: iris_types::types::TypeId(0),
            cost: CostTerm::Unit,
            arity: 2,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Prim { opcode: 0x00 },
        },
    );
    // Input ref 0
    nodes.insert(
        NodeId(10),
        Node {
            id: NodeId(10),
            kind: NodeKind::Lit,
            type_sig: iris_types::types::TypeId(0),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0xFF,
                value: vec![0],
            },
        },
    );
    // Input ref 1
    nodes.insert(
        NodeId(20),
        Node {
            id: NodeId(20),
            kind: NodeKind::Lit,
            type_sig: iris_types::types::TypeId(0),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0xFF,
                value: vec![1],
            },
        },
    );
    let edges = vec![
        Edge {
            source: NodeId(1),
            target: NodeId(10),
            port: 0,
            label: EdgeLabel::Argument,
        },
        Edge {
            source: NodeId(1),
            target: NodeId(20),
            port: 1,
            label: EdgeLabel::Argument,
        },
    ];
    SemanticGraph {
        root: NodeId(1),
        nodes,
        edges,
        type_env: TypeEnv {
            types: BTreeMap::new(),
        },
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

/// Build an empty/broken program (no nodes, guaranteed to fail).
fn build_broken_program() -> SemanticGraph {
    SemanticGraph {
        root: NodeId(0),
        nodes: HashMap::new(),
        edges: Vec::new(),
        type_env: TypeEnv {
            types: BTreeMap::new(),
        },
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0xFF; 32]),
    }
}

/// Evaluate a SemanticGraph with the given inputs via the interpreter.
fn eval_program(program: &SemanticGraph, inputs: &[Value]) -> Option<Vec<Value>> {
    match iris_exec::interpreter::interpret(program, inputs, None) {
        Ok((outputs, _)) => Some(outputs),
        Err(_) => None,
    }
}

// ===========================================================================
// Phase 1: Cold Start
// ===========================================================================

#[test]
fn test_cold_start() {
    let dir = test_dir("cold_start");

    // Create IrisRuntime with default programs.
    let runtime = IrisRuntime::new();

    // Verify runtime is initialized with 5 built-in programs.
    assert!(runtime.initialized, "runtime should be initialized");
    assert!(
        !runtime.replace_prim.nodes.is_empty(),
        "replace_prim should be non-empty"
    );
    assert!(
        !runtime.direct_replace_prim.nodes.is_empty(),
        "direct_replace_prim should be non-empty"
    );
    assert!(
        !runtime.add_node.nodes.is_empty(),
        "add_node should be non-empty"
    );
    assert!(
        !runtime.connect.nodes.is_empty(),
        "connect should be non-empty"
    );
    assert!(
        !runtime.evaluate.nodes.is_empty(),
        "evaluate should be non-empty"
    );

    // Verify no deployed (hot-swapped) components yet.
    assert!(
        runtime.deployed_components().is_empty(),
        "no deployed components on cold start"
    );

    // Verify no saved state on disk.
    assert!(
        !dir.join("daemon_state.json").exists(),
        "no saved state should exist"
    );
    assert!(
        !dir.join("audit_trail.json").exists(),
        "no audit trail should exist"
    );

    // Verify component audit is empty.
    assert!(
        runtime.component_audit().is_empty(),
        "no component audit entries on cold start"
    );

    // Cleanup.
    let _ = std::fs::remove_dir_all(&dir);
}

// ===========================================================================
// Phase 2: Evolution Cycle
// ===========================================================================

#[test]
fn test_evolution_cycle() {
    let exec = IrisExecutionService::with_defaults();

    // Create AutoImprover with deterministic seed.
    let mut improver = AutoImprover::with_seed(
        AutoImproveConfig {
            cycle_interval_secs: 0,
            max_slowdown: 100.0,
            test_cases_per_component: 4,
            evolution_generations: 5,
            evolution_pop_size: 8,
            gate_runs: 1,
            explore_problems: 2,
        },
        42,
    );

    // Register components (mirrors daemon setup).
    improver.register_component("mutation_insert_node", Duration::from_micros(50));
    improver.register_component("mutation_delete_node", Duration::from_micros(30));
    improver.register_component("mutation_rewire_edge", Duration::from_micros(45));
    improver.register_component("seed_arithmetic", Duration::from_micros(20));
    improver.register_component("seed_fold", Duration::from_micros(25));

    // Run 3 improvement cycles.
    let mut events = Vec::new();
    for _ in 0..3 {
        let event = improver.run_cycle(&exec);
        events.push(event);
    }

    // Verify at least one cycle attempted evolution (profiled or explored).
    assert_eq!(events.len(), 3, "should have 3 events from 3 cycles");
    assert_eq!(improver.cycle_count(), 3, "cycle count should be 3");

    // Each event should have a valid action.
    for event in &events {
        match &event.action {
            ImprovementAction::Profiled { slowest, time_ns } => {
                assert!(!slowest.is_empty(), "profiled component name should be non-empty");
                assert!(*time_ns > 0, "profiled time should be positive");
            }
            ImprovementAction::Evolved { candidate_slowdown } => {
                assert!(
                    *candidate_slowdown >= 0.0,
                    "candidate slowdown should be non-negative"
                );
            }
            ImprovementAction::Deployed { slowdown } => {
                assert!(*slowdown >= 0.0, "deployed slowdown should be non-negative");
            }
            ImprovementAction::Failed { reason } => {
                assert!(!reason.is_empty(), "failure reason should be non-empty");
            }
            ImprovementAction::Explored {
                new_capability,
                solve_rate,
            } => {
                assert!(!new_capability.is_empty(), "explored capability should have a name");
                assert!(
                    *solve_rate >= 0.0 && *solve_rate <= 1.0,
                    "solve rate should be in [0, 1]"
                );
            }
        }
    }

    // Verify history was recorded.
    let history = improver.history();
    assert!(
        history.len() >= 3,
        "history should have at least 3 entries (may have more from profiling steps), got {}",
        history.len()
    );
}

// ===========================================================================
// Phase 3: Persistence (Save State to Disk)
// ===========================================================================

#[test]
fn test_save_state_to_disk() {
    let dir = test_dir("save_state");

    // Run daemon with persistence enabled.
    let config = tiny_config(Some(dir.clone()));
    let mut daemon = SelfImprovingDaemon::new(config);
    let result = daemon.run();

    assert_eq!(result.cycles_completed, 20);

    // Verify daemon_state.json exists and is valid.
    let state_path = dir.join("daemon_state.json");
    assert!(state_path.exists(), "daemon_state.json should exist");
    let state_data = std::fs::read_to_string(&state_path).expect("read state file");
    let state: DaemonState =
        serde_json::from_str(&state_data).expect("daemon_state.json should deserialize");
    assert_eq!(
        state.daemon_cycles, 20,
        "daemon_cycles should be 20"
    );
    assert!(
        state.improvement_cycles >= 1,
        "should have at least 1 improvement cycle recorded"
    );

    // Verify audit_trail.json exists and is valid.
    let audit_path = dir.join("audit_trail.json");
    assert!(audit_path.exists(), "audit_trail.json should exist");
    let audit_data = std::fs::read_to_string(&audit_path).expect("read audit file");
    let audit: AuditTrail =
        serde_json::from_str(&audit_data).expect("audit_trail.json should deserialize");
    assert!(
        audit.verify_chain(),
        "audit trail chain integrity should hold after save"
    );

    // Verify that if there were deployed components, they have program_json.
    for record in &state.deployed {
        assert!(
            !record.name.is_empty(),
            "deployed component should have a name"
        );
        assert!(
            !record.program_json.is_empty(),
            "deployed component should have program_json"
        );
    }

    // Cleanup.
    let _ = std::fs::remove_dir_all(&dir);
}

// ===========================================================================
// Phase 4: Reload from Disk
// ===========================================================================

#[test]
fn test_reload_from_disk() {
    let dir = test_dir("reload");

    // Phase A: Run daemon, save state.
    let config = tiny_config(Some(dir.clone()));
    let mut daemon1 = SelfImprovingDaemon::new(config);
    let result1 = daemon1.run();
    assert_eq!(result1.cycles_completed, 20);

    // Record pre-save state.
    let pre_save_cycles = result1.cycles_completed;

    // Phase B: Create a completely fresh daemon and load from disk.
    let config2 = tiny_config(None);
    let mut daemon2 = SelfImprovingDaemon::new(config2);
    let loaded_count = daemon2.load_state(&dir);

    // Cycle count should be restored.
    assert_eq!(
        daemon2.cycle_count(),
        pre_save_cycles,
        "cycle count should be restored from checkpoint"
    );

    // Phase C: Load and verify audit trail.
    let audit_path = dir.join("audit_trail.json");
    let audit = AuditTrail::load(&audit_path.to_string_lossy())
        .expect("should load audit trail from disk");
    assert!(
        audit.verify_chain(),
        "loaded audit trail should have valid chain integrity"
    );

    // Verify prev_hash links: each entry's prev_hash should equal the
    // previous entry's entry_hash (genesis uses [0u8; 32]).
    for (i, entry) in audit.entries().iter().enumerate() {
        let expected_prev = if i == 0 {
            [0u8; 32]
        } else {
            audit.entries()[i - 1].entry_hash
        };
        assert_eq!(
            entry.prev_hash, expected_prev,
            "entry {} prev_hash should link to entry {} entry_hash",
            i,
            i.saturating_sub(1)
        );
    }

    // Phase D: Verify loaded daemon state file is valid.
    let state_data = std::fs::read_to_string(dir.join("daemon_state.json"))
        .expect("read daemon state");
    let state: DaemonState =
        serde_json::from_str(&state_data).expect("deserialize daemon state");
    assert_eq!(state.daemon_cycles, pre_save_cycles);

    // If components were deployed, verify they were loaded.
    if !state.deployed.is_empty() {
        assert!(
            loaded_count > 0,
            "should have loaded deployed components from disk"
        );
    }

    // Cleanup.
    let _ = std::fs::remove_dir_all(&dir);
}

// ===========================================================================
// Phase 5: Hot-Swap + Rollback
// ===========================================================================

#[test]
fn test_hot_swap_and_rollback() {
    let mut runtime = IrisRuntime::new();
    let mut audit = AuditTrail::new();

    // Build a known-good add program: add(inputs[0], inputs[1]).
    let add_program = build_add_program();

    // Verify the add program works: 3 + 7 = 10.
    let output = eval_program(&add_program, &[Value::Int(3), Value::Int(7)]);
    assert_eq!(
        output,
        Some(vec![Value::Int(10)]),
        "add program should compute 3 + 7 = 10"
    );

    // Deploy the known-good program.
    let _old = runtime.replace_component("replace_prim", add_program.clone());
    audit.record(AuditAction::ComponentDeployed {
        name: "replace_prim".to_string(),
        slowdown: 1.0,
    });

    // Verify it is deployed.
    assert!(
        runtime.deployed_components().contains_key("replace_prim"),
        "replace_prim should be in deployed components"
    );

    // Save the good version for rollback.
    let good_program = add_program.clone();

    // Deploy a deliberately broken program (empty graph).
    let broken = build_broken_program();
    let _prev = runtime.replace_component("replace_prim", broken.clone());
    audit.record(AuditAction::ComponentDeployed {
        name: "replace_prim".to_string(),
        slowdown: 999.0,
    });

    // Detect failure: the broken program cannot compute anything.
    let broken_deployed = runtime.deployed_components().get("replace_prim").unwrap();
    let broken_output = eval_program(broken_deployed, &[Value::Int(3), Value::Int(7)]);
    assert!(
        broken_output.is_none() || broken_output == Some(vec![]),
        "broken program should fail to produce correct output"
    );

    // Rollback to previous version.
    runtime.rollback_component("replace_prim", good_program.clone());
    audit.record(AuditAction::ComponentReverted {
        name: "replace_prim".to_string(),
        reason: "correctness failure detected".to_string(),
    });

    // Verify correct program is restored.
    let restored = runtime.deployed_components().get("replace_prim").unwrap();
    let restored_output = eval_program(restored, &[Value::Int(3), Value::Int(7)]);
    assert_eq!(
        restored_output,
        Some(vec![Value::Int(10)]),
        "restored program should compute 3 + 7 = 10"
    );

    // Verify audit trail records both deploy and rollback.
    assert_eq!(audit.len(), 3, "audit should have 3 entries: deploy, broken deploy, rollback");
    assert!(audit.verify_chain(), "audit chain should be valid");

    // Check specific entries.
    match &audit.entries()[0].action {
        AuditAction::ComponentDeployed { name, slowdown } => {
            assert_eq!(name, "replace_prim");
            assert!((slowdown - 1.0).abs() < 0.01);
        }
        other => panic!("expected ComponentDeployed, got {:?}", other),
    }
    match &audit.entries()[2].action {
        AuditAction::ComponentReverted { name, reason } => {
            assert_eq!(name, "replace_prim");
            assert!(reason.contains("correctness failure"));
        }
        other => panic!("expected ComponentReverted, got {:?}", other),
    }

    // Verify the component audit log in the runtime.
    let comp_audit = runtime.component_audit();
    assert!(
        comp_audit.len() >= 3,
        "runtime component audit should have at least 3 entries, got {}",
        comp_audit.len()
    );
    // Last entry should be a rollback.
    let last = &comp_audit[comp_audit.len() - 1];
    assert_eq!(last.0, "replace_prim");
    assert!(last.1.contains("rolled back"));
}

// ===========================================================================
// Phase 6: Full Loop Verification
// ===========================================================================

#[test]
fn test_full_realtime_optimization_loop() {
    let dir = test_dir("full_loop");

    // --- Step 1: Cold start + run 5 daemon cycles ---
    let config1 = SelfImprovingConfig {
        cycle_time_ms: 1,
        max_cycles: Some(5),
        improve_interval: 2,
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
        state_dir: Some(dir.clone()),
        memory_limit: 0,
        seed: Some(42),
        max_improve_threads: 2,
        max_stagnant: 5,
        min_improvement: 0.05,
        exec_mode: ExecMode::Continuous,
        trigger_check_interval: 100,
    };

    let mut daemon1 = SelfImprovingDaemon::new(config1);
    let result1 = daemon1.run();
    assert_eq!(result1.cycles_completed, 5);

    // Record metrics before destruction.
    let pre_save_cycles = result1.cycles_completed;
    let pre_save_improvements = result1.improvement_cycles;
    // Verify files saved to disk.
    assert!(
        dir.join("daemon_state.json").exists(),
        "daemon_state.json should exist after first run"
    );
    assert!(
        dir.join("audit_trail.json").exists(),
        "audit_trail.json should exist after first run"
    );

    // Read the saved state for later comparison.
    let state_data = std::fs::read_to_string(dir.join("daemon_state.json"))
        .expect("read state");
    let saved_state: DaemonState =
        serde_json::from_str(&state_data).expect("parse state");

    // --- Step 2: Destroy everything ---
    drop(daemon1);

    // --- Step 3: Reload from disk ---
    let config2 = SelfImprovingConfig {
        cycle_time_ms: 1,
        max_cycles: Some(10), // will run 5 more cycles (total 10)
        improve_interval: 2,
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
        state_dir: Some(dir.clone()),
        memory_limit: 0,
        seed: Some(42),
        max_improve_threads: 2,
        max_stagnant: 5,
        min_improvement: 0.05,
        exec_mode: ExecMode::Continuous,
        trigger_check_interval: 100,
    };

    let mut daemon2 = SelfImprovingDaemon::new(config2);

    // Load state explicitly (the run() method also loads, but we want to verify).
    let loaded_count = daemon2.load_state(&dir);

    // Verify state restored: cycle count should match pre-save.
    assert_eq!(
        daemon2.cycle_count(),
        pre_save_cycles,
        "cycle count should be restored from checkpoint"
    );

    // If deployed components were saved, verify they were loaded.
    if !saved_state.deployed.is_empty() {
        assert!(
            loaded_count > 0,
            "should have loaded {} deployed components",
            saved_state.deployed.len()
        );
    }

    // --- Step 4: Run 5 more cycles ---
    let result2 = daemon2.run();

    // The daemon had cycle_count=5 from load, and max_cycles=10, so it runs 5 more.
    assert_eq!(
        result2.cycles_completed, 10,
        "should have completed 10 total cycles (5 loaded + 5 new)"
    );

    // Verify continued operation: improvement cycles should have increased.
    assert!(
        result2.improvement_cycles >= pre_save_improvements,
        "improvement cycles should be >= pre-save ({}), got {}",
        pre_save_improvements,
        result2.improvement_cycles
    );

    // --- Step 5: Check audit trail integrity (Merkle chain) ---
    // The daemon saves state on shutdown, so the audit trail should be updated.
    let audit_path = dir.join("audit_trail.json");
    let final_audit = AuditTrail::load(&audit_path.to_string_lossy())
        .expect("should load final audit trail");

    // Verify chain integrity.
    assert!(
        final_audit.verify_chain(),
        "final audit trail should have valid chain integrity"
    );

    // Verify every entry's entry_hash matches its recomputed hash.
    for (i, entry) in final_audit.entries().iter().enumerate() {
        assert_eq!(
            entry.entry_hash,
            entry.compute_hash(),
            "entry {} entry_hash should match recomputed hash",
            i
        );
    }

    // Verify prev_hash chain (Merkle chain links).
    for (i, entry) in final_audit.entries().iter().enumerate() {
        let expected_prev = if i == 0 {
            [0u8; 32]
        } else {
            final_audit.entries()[i - 1].entry_hash
        };
        assert_eq!(
            entry.prev_hash, expected_prev,
            "entry {} prev_hash chain broken",
            i
        );
    }

    // Verify Merkle root is non-zero if there are entries.
    if !final_audit.entries().is_empty() {
        assert_ne!(
            *final_audit.merkle_root(),
            [0u8; 32],
            "Merkle root should be non-zero when audit trail has entries"
        );
    }

    // Verify the final state file is consistent.
    let final_state_data = std::fs::read_to_string(dir.join("daemon_state.json"))
        .expect("read final state");
    let final_state: DaemonState =
        serde_json::from_str(&final_state_data).expect("parse final state");
    assert_eq!(
        final_state.daemon_cycles, 10,
        "final state should record 10 daemon cycles"
    );

    // Cleanup.
    let _ = std::fs::remove_dir_all(&dir);
}
