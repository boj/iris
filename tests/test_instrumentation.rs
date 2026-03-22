//! Integration tests for IRIS performance instrumentation and audit trail.

use std::time::Duration;

use iris_evolve::instrumentation::{
    AnomalyKind, AuditAction, AuditTrail, CorrectionResult, InspectionResult, SelfInspector,
    Severity, Telemetry, TimingWindow,
};

// ---------------------------------------------------------------------------
// TimingWindow
// ---------------------------------------------------------------------------

#[test]
fn timing_window_records_correctly() {
    let baseline = Duration::from_micros(100);
    let mut w = TimingWindow::new(baseline);

    assert_eq!(w.recent.len(), 0);
    assert_eq!(w.baseline, baseline);

    w.record(Duration::from_micros(90));
    w.record(Duration::from_micros(110));
    w.record(Duration::from_micros(100));

    assert_eq!(w.recent.len(), 3);
    // Mean should be 100us.
    assert_eq!(w.mean, Duration::from_micros(100));
    assert!(!w.regression_detected);
}

#[test]
fn timing_window_detects_regression_at_2x() {
    let baseline = Duration::from_micros(100);
    let mut w = TimingWindow::new(baseline);

    // All measurements are 3x baseline -> regression.
    for _ in 0..10 {
        w.record(Duration::from_micros(300));
    }

    assert!(w.regression_detected);

    // Verify mean is ~300us.
    let mean_us = w.mean.as_micros();
    assert!(mean_us >= 290 && mean_us <= 310, "mean was {mean_us}us");
}

#[test]
fn timing_window_no_regression_below_threshold() {
    let baseline = Duration::from_micros(100);
    let mut w = TimingWindow::new(baseline);

    // 1.8x baseline -> should NOT flag.
    for _ in 0..10 {
        w.record(Duration::from_micros(180));
    }

    assert!(!w.regression_detected);
}

#[test]
fn timing_window_p99_computed() {
    let baseline = Duration::from_micros(100);
    let mut w = TimingWindow::new(baseline);

    for i in 1..=100u64 {
        w.record(Duration::from_micros(i));
    }

    // P99 of 1..=100 should be 99 or 100.
    let p99_us = w.p99.as_micros();
    assert!(p99_us >= 99, "p99 was {p99_us}us");
}

// ---------------------------------------------------------------------------
// AuditTrail chain integrity
// ---------------------------------------------------------------------------

#[test]
fn audit_trail_chain_integrity_passes() {
    let mut trail = AuditTrail::new();

    trail.record(AuditAction::ComponentDeployed {
        name: "eval".into(),
        slowdown: 1.3,
    });
    trail.record(AuditAction::PerformanceImproved {
        component: "eval".into(),
        before_ns: 5000,
        after_ns: 3000,
    });
    trail.record(AuditAction::CorrectnessVerified {
        component: "eval".into(),
        test_count: 50,
        pass_rate: 1.0,
    });

    assert!(trail.verify_chain());
    assert_eq!(trail.len(), 3);
}

#[test]
fn audit_trail_detects_tampering_modified_entry() {
    let mut trail = AuditTrail::new();

    trail.record(AuditAction::ComponentDeployed {
        name: "a".into(),
        slowdown: 1.0,
    });
    trail.record(AuditAction::ComponentDeployed {
        name: "b".into(),
        slowdown: 1.5,
    });

    assert!(trail.verify_chain());

    // Tamper: change the first entry's performance delta.
    trail.entries_mut()[0].performance_delta = 42.0;

    assert!(!trail.verify_chain());
}

#[test]
fn audit_trail_detects_tampering_broken_chain() {
    let mut trail = AuditTrail::new();

    trail.record(AuditAction::SelfImprovement {
        strategy: "evo".into(),
        improvement: 0.1,
    });
    trail.record(AuditAction::SelfImprovement {
        strategy: "evo".into(),
        improvement: 0.2,
    });

    assert!(trail.verify_chain());

    // Tamper: corrupt the prev_hash link on the second entry.
    trail.entries_mut()[1].prev_hash = [0xFFu8; 32];

    assert!(!trail.verify_chain());
}

// ---------------------------------------------------------------------------
// AuditEntry hashing is deterministic
// ---------------------------------------------------------------------------

#[test]
fn audit_entry_hashing_deterministic() {
    let action = AuditAction::EvolutionGeneration {
        generation: 7,
        best_fitness: 0.95,
        population_size: 64,
    };

    let mut t1 = AuditTrail::new();
    let mut t2 = AuditTrail::new();

    t1.record(action.clone());
    t2.record(action);

    assert_eq!(t1.entries()[0].entry_hash, t2.entries()[0].entry_hash);
}

// ---------------------------------------------------------------------------
// Save / Load round-trip
// ---------------------------------------------------------------------------

#[test]
fn audit_trail_save_load_roundtrip() {
    let mut trail = AuditTrail::new();
    trail.record(AuditAction::ComponentDeployed {
        name: "x".into(),
        slowdown: 1.1,
    });
    trail.record(AuditAction::AnomalyCorrected {
        anomaly_id: 0,
        correction: "reverted".into(),
    });

    let dir = std::env::temp_dir().join("iris_audit_test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("trail.json");
    let path_str = path.to_str().unwrap();

    trail.save(path_str).unwrap();
    let loaded = AuditTrail::load(path_str).unwrap();

    assert_eq!(loaded.len(), trail.len());
    assert_eq!(loaded.merkle_root(), trail.merkle_root());
    assert!(loaded.verify_chain());
    assert_eq!(loaded.entries()[0].entry_hash, trail.entries()[0].entry_hash);
    assert_eq!(loaded.entries()[1].entry_hash, trail.entries()[1].entry_hash);

    // Clean up.
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}

// ---------------------------------------------------------------------------
// SelfInspector
// ---------------------------------------------------------------------------

#[test]
fn self_inspector_detects_performance_regression() {
    let mut inspector = SelfInspector::new(Duration::from_secs(1), 1 << 30);

    inspector
        .telemetry
        .register_component("comp_a", Duration::from_micros(100));

    // Feed 5x-slow timings.
    for _ in 0..20 {
        inspector
            .telemetry
            .record_timing("comp_a", Duration::from_micros(600));
    }

    let findings = inspector.inspect();

    let has_regression = findings.iter().any(|f| {
        matches!(f, InspectionResult::Regression { component, .. } if component == "comp_a")
    });
    assert!(has_regression, "should detect regression for comp_a");

    // Audit trail should have recorded the anomaly.
    assert!(!inspector.audit.is_empty());
    assert!(inspector.audit.verify_chain());
}

#[test]
fn self_inspector_auto_corrects_by_reverting() {
    let mut inspector = SelfInspector::new(Duration::from_secs(1), 0);

    let anomaly = iris_evolve::instrumentation::Anomaly {
        timestamp: 5,
        component: "bad_comp".into(),
        kind: AnomalyKind::PerformanceRegression {
            expected_ns: 100,
            actual_ns: 1000,
            ratio: 10.0,
        },
        severity: Severity::Critical,
        auto_corrected: false,
    };

    let result = inspector.correct(&anomaly);
    assert_eq!(
        result,
        CorrectionResult::Reverted {
            component: "bad_comp".into()
        }
    );

    // Correction recorded.
    assert_eq!(inspector.audit.len(), 1);
    assert!(inspector.audit.verify_chain());
}

#[test]
fn self_inspector_correctness_failure_reverts_with_alert() {
    let mut inspector = SelfInspector::new(Duration::from_secs(1), 0);

    let anomaly = iris_evolve::instrumentation::Anomaly {
        timestamp: 10,
        component: "buggy".into(),
        kind: AnomalyKind::CorrectnessFailure {
            test_case_idx: 3,
            expected: "expected_val".into(),
            actual: "wrong_val".into(),
        },
        severity: Severity::Critical,
        auto_corrected: false,
    };

    let result = inspector.correct(&anomaly);
    assert_eq!(
        result,
        CorrectionResult::RevertedWithAlert {
            component: "buggy".into()
        }
    );
}

#[test]
fn self_inspector_memory_exceeded() {
    let mut inspector = SelfInspector::new(Duration::from_secs(1), 1024);
    inspector.telemetry.system_metrics.memory_usage_bytes = 2048;

    let findings = inspector.inspect();
    let has_mem = findings
        .iter()
        .any(|f| matches!(f, InspectionResult::MemoryExceeded { .. }));
    assert!(has_mem, "should detect memory exceeded");
}

// ---------------------------------------------------------------------------
// Telemetry anomaly detection
// ---------------------------------------------------------------------------

#[test]
fn telemetry_anomaly_detection() {
    let mut tel = Telemetry::new();
    tel.register_component("fast", Duration::from_micros(10));
    tel.register_component("slow", Duration::from_micros(10));

    // fast stays fast.
    for _ in 0..10 {
        tel.record_timing("fast", Duration::from_micros(12));
    }

    // slow gets very slow.
    for _ in 0..10 {
        tel.record_timing("slow", Duration::from_micros(100));
    }

    tel.detect_anomalies(1);

    // Only "slow" should have an anomaly.
    assert_eq!(tel.anomalies.len(), 1);
    assert_eq!(tel.anomalies[0].component, "slow");
}

// ---------------------------------------------------------------------------
// Merkle root changes with entries
// ---------------------------------------------------------------------------

#[test]
fn merkle_root_updates() {
    let mut trail = AuditTrail::new();
    let r0 = *trail.merkle_root();

    trail.record(AuditAction::ComponentDeployed {
        name: "a".into(),
        slowdown: 1.0,
    });
    let r1 = *trail.merkle_root();

    trail.record(AuditAction::ComponentDeployed {
        name: "b".into(),
        slowdown: 1.1,
    });
    let r2 = *trail.merkle_root();

    assert_ne!(r0, r1);
    assert_ne!(r1, r2);
}

// ---------------------------------------------------------------------------
// entries_since
// ---------------------------------------------------------------------------

#[test]
fn entries_since_filters_correctly() {
    let mut trail = AuditTrail::new();
    for _ in 0..10 {
        trail.record(AuditAction::ComponentDeployed {
            name: "x".into(),
            slowdown: 1.0,
        });
    }

    let since_5 = trail.entries_since(5);
    assert_eq!(since_5.len(), 5); // entries 5..9
    assert!(since_5.iter().all(|e| e.timestamp >= 5));
}
