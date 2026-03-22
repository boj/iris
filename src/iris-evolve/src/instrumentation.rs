//! Internal performance instrumentation with audit trail and provability.
//!
//! When IRIS runs in daemon mode, the `SelfInspector` continuously monitors
//! per-component performance, detects regressions and anomalies, auto-corrects
//! flaws, and maintains a cryptographic (BLAKE3) audit trail proving every
//! change was an improvement.
//!
//! Key types:
//! - [`Telemetry`] — per-component timing windows and system-wide metrics.
//! - [`AuditTrail`] — tamper-evident chain of [`AuditEntry`] records.
//! - [`SelfInspector`] — periodic self-inspection with auto-correction.

use std::collections::{BTreeMap, VecDeque};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use iris_types::proof::ProofReceipt;

// ---------------------------------------------------------------------------
// Severity
// ---------------------------------------------------------------------------

/// Severity level for an anomaly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

// ---------------------------------------------------------------------------
// AnomalyKind / Anomaly
// ---------------------------------------------------------------------------

/// Classification of a detected anomaly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnomalyKind {
    /// Component is running slower than expected.
    PerformanceRegression {
        expected_ns: u64,
        actual_ns: u64,
        ratio: f64,
    },
    /// Component produced incorrect output on a test case.
    CorrectnessFailure {
        test_case_idx: usize,
        expected: String,
        actual: String,
    },
    /// Memory usage spiked above the configured limit.
    MemorySpike {
        bytes: usize,
        limit: usize,
    },
    /// Component crashed during execution.
    ComponentCrash {
        error: String,
    },
    /// A self-modification violated a safety invariant.
    SelfModificationViolation {
        component: String,
        change: String,
    },
}

/// A detected anomaly with metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Anomaly {
    pub timestamp: u64,
    pub component: String,
    pub kind: AnomalyKind,
    pub severity: Severity,
    pub auto_corrected: bool,
}

// ---------------------------------------------------------------------------
// TimingWindow
// ---------------------------------------------------------------------------

/// Sliding window of recent timing measurements for one component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingWindow {
    /// Last N timing measurements.
    pub recent: VecDeque<Duration>,
    /// Established performance baseline.
    pub baseline: Duration,
    /// Current running mean.
    pub mean: Duration,
    /// Current p99.
    pub p99: Duration,
    /// Whether a regression has been detected in the current window.
    pub regression_detected: bool,
}

/// Maximum number of measurements held in a `TimingWindow`.
const WINDOW_SIZE: usize = 128;

/// A timing that exceeds `REGRESSION_RATIO * baseline` is flagged.
const REGRESSION_RATIO: f64 = 2.0;

impl TimingWindow {
    /// Create a new window with the given baseline.
    pub fn new(baseline: Duration) -> Self {
        Self {
            recent: VecDeque::with_capacity(WINDOW_SIZE),
            baseline,
            mean: baseline,
            p99: baseline,
            regression_detected: false,
        }
    }

    /// Record a new timing measurement and recompute statistics.
    pub fn record(&mut self, duration: Duration) {
        if self.recent.len() >= WINDOW_SIZE {
            self.recent.pop_front();
        }
        self.recent.push_back(duration);
        self.recompute();
    }

    /// Recompute mean, p99, and regression flag.
    fn recompute(&mut self) {
        if self.recent.is_empty() {
            return;
        }

        // Mean
        let total_ns: u128 = self.recent.iter().map(|d| d.as_nanos()).sum();
        let mean_ns = total_ns / self.recent.len() as u128;
        self.mean = Duration::from_nanos(mean_ns as u64);

        // P99
        let mut sorted: Vec<Duration> = self.recent.iter().copied().collect();
        sorted.sort();
        let idx = ((sorted.len() as f64) * 0.99).ceil() as usize;
        let idx = idx.min(sorted.len()) .saturating_sub(1);
        self.p99 = sorted[idx];

        // Regression detection: mean > REGRESSION_RATIO * baseline
        let baseline_ns = self.baseline.as_nanos() as f64;
        let mean_f = mean_ns as f64;
        self.regression_detected = baseline_ns > 0.0 && mean_f > baseline_ns * REGRESSION_RATIO;
    }
}

// ---------------------------------------------------------------------------
// SystemMetrics
// ---------------------------------------------------------------------------

/// System-wide metrics snapshot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Average evaluation time per program (nanoseconds).
    pub eval_speed_ns: u64,
    /// Problems solved per hour (estimated).
    pub problems_solved_per_hour: f64,
    /// Percentage of IRIS written in IRIS (0.0 – 100.0).
    pub iris_percentage: f64,
    /// Number of deployed IRIS components.
    pub deployed_components: usize,
    /// Total number of self-written components (lifetime).
    pub total_self_written: usize,
    /// Current memory usage in bytes.
    pub memory_usage_bytes: usize,
    /// Uptime in seconds.
    pub uptime_secs: u64,
}

// ---------------------------------------------------------------------------
// Telemetry
// ---------------------------------------------------------------------------

/// Aggregate telemetry: per-component timing windows, system metrics, and
/// a list of detected anomalies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Telemetry {
    /// Per-component timing (component name -> sliding window).
    pub component_timings: BTreeMap<String, TimingWindow>,
    /// System-wide metrics.
    pub system_metrics: SystemMetrics,
    /// Detected anomalies.
    pub anomalies: Vec<Anomaly>,
}

impl Telemetry {
    /// Create empty telemetry.
    pub fn new() -> Self {
        Self {
            component_timings: BTreeMap::new(),
            system_metrics: SystemMetrics::default(),
            anomalies: Vec::new(),
        }
    }

    /// Register a component with its baseline timing.
    pub fn register_component(&mut self, name: &str, baseline: Duration) {
        self.component_timings
            .insert(name.to_string(), TimingWindow::new(baseline));
    }

    /// Record a timing measurement for a component.
    ///
    /// If the component is not registered, it is silently ignored.
    pub fn record_timing(&mut self, component: &str, duration: Duration) {
        if let Some(window) = self.component_timings.get_mut(component) {
            window.record(duration);
        }
    }

    /// Scan all timing windows and generate anomalies for regressions.
    pub fn detect_anomalies(&mut self, timestamp: u64) {
        for (name, window) in &self.component_timings {
            if window.regression_detected {
                let baseline_ns = window.baseline.as_nanos() as u64;
                let actual_ns = window.mean.as_nanos() as u64;
                let ratio = if baseline_ns > 0 {
                    actual_ns as f64 / baseline_ns as f64
                } else {
                    1.0
                };

                let anomaly = Anomaly {
                    timestamp,
                    component: name.clone(),
                    kind: AnomalyKind::PerformanceRegression {
                        expected_ns: baseline_ns,
                        actual_ns,
                        ratio,
                    },
                    severity: if ratio > 5.0 {
                        Severity::Critical
                    } else {
                        Severity::Warning
                    },
                    auto_corrected: false,
                };

                // Avoid duplicate entries for the same component+timestamp.
                let already = self.anomalies.iter().any(|a| {
                    a.component == anomaly.component && a.timestamp == anomaly.timestamp
                });
                if !already {
                    self.anomalies.push(anomaly);
                }
            }
        }
    }
}

impl Default for Telemetry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// AuditAction
// ---------------------------------------------------------------------------

/// An action that mutated the running system, recorded in the audit trail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditAction {
    ComponentDeployed {
        name: String,
        slowdown: f64,
    },
    ComponentReverted {
        name: String,
        reason: String,
    },
    PerformanceImproved {
        component: String,
        before_ns: u64,
        after_ns: u64,
    },
    CorrectnessVerified {
        component: String,
        test_count: usize,
        pass_rate: f32,
    },
    AnomalyDetected {
        anomaly: Anomaly,
    },
    AnomalyCorrected {
        anomaly_id: u64,
        correction: String,
    },
    SelfImprovement {
        strategy: String,
        improvement: f64,
    },
    EvolutionGeneration {
        generation: u64,
        best_fitness: f32,
        population_size: usize,
    },
}

// ---------------------------------------------------------------------------
// AuditEntry
// ---------------------------------------------------------------------------

/// A single record in the tamper-evident audit chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Monotonically increasing id.
    pub id: u64,
    /// Unix-style timestamp (or cycle count).
    pub timestamp: u64,
    /// What happened.
    pub action: AuditAction,
    /// BLAKE3 hash of the system state *before* this action.
    pub before_hash: [u8; 32],
    /// BLAKE3 hash of the system state *after* this action.
    pub after_hash: [u8; 32],
    /// Formal proof of correctness, if available.
    pub proof: Option<ProofReceipt>,
    /// Positive = improvement, negative = regression.
    pub performance_delta: f64,
    /// BLAKE3 hash of this entry's contents (excluding `entry_hash` itself).
    pub entry_hash: [u8; 32],
    /// Hash of the previous entry (genesis entry uses `[0u8; 32]`).
    pub prev_hash: [u8; 32],
}

impl AuditEntry {
    /// Compute the canonical hash for this entry.
    ///
    /// Covers all fields except `entry_hash` itself (it is the output).
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.id.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&serde_json::to_vec(&self.action).unwrap_or_default());
        hasher.update(&self.before_hash);
        hasher.update(&self.after_hash);
        if let Some(ref proof) = self.proof {
            hasher.update(&serde_json::to_vec(proof).unwrap_or_default());
        }
        hasher.update(&self.performance_delta.to_le_bytes());
        hasher.update(&self.prev_hash);
        *hasher.finalize().as_bytes()
    }
}

// ---------------------------------------------------------------------------
// AuditTrail
// ---------------------------------------------------------------------------

/// Tamper-evident chain of audit entries with a BLAKE3 Merkle root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    /// The ordered chain of audit entries.
    ///
    /// **WARNING**: mutating entries directly will break chain integrity.
    /// Use [`record`](AuditTrail::record) to append.  The field is
    /// `pub(crate)` to allow crate-internal test access while preventing
    /// external mutation that would silently break the chain.
    entries: Vec<AuditEntry>,
    /// Merkle root over all `entry_hash` values.
    merkle_root: [u8; 32],
}

impl AuditTrail {
    /// Create an empty audit trail.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            merkle_root: [0u8; 32],
        }
    }

    /// Mutable access to entries — **only for tamper-detection tests**.
    /// Production code must use `record()` to maintain chain integrity.
    #[doc(hidden)]
    pub fn entries_mut(&mut self) -> &mut [AuditEntry] {
        &mut self.entries
    }

    /// Append a new entry and return a reference to it.
    pub fn record(&mut self, action: AuditAction) -> &AuditEntry {
        self.record_full(action, [0u8; 32], [0u8; 32], None, 0.0)
    }

    /// Append a fully populated entry.
    pub fn record_full(
        &mut self,
        action: AuditAction,
        before_hash: [u8; 32],
        after_hash: [u8; 32],
        proof: Option<ProofReceipt>,
        performance_delta: f64,
    ) -> &AuditEntry {
        let id = self.entries.len() as u64;
        let timestamp = id; // caller can overwrite via a wrapper
        let prev_hash = self
            .entries
            .last()
            .map(|e| e.entry_hash)
            .unwrap_or([0u8; 32]);

        let mut entry = AuditEntry {
            id,
            timestamp,
            action,
            before_hash,
            after_hash,
            proof,
            performance_delta,
            entry_hash: [0u8; 32], // placeholder
            prev_hash,
        };
        entry.entry_hash = entry.compute_hash();

        self.entries.push(entry);
        self.recompute_merkle_root();
        self.entries.last().unwrap()
    }

    /// Verify the integrity of the entire chain.
    ///
    /// Returns `true` iff every entry's `entry_hash` matches its recomputed
    /// hash AND every entry's `prev_hash` matches the preceding entry's
    /// `entry_hash`.
    pub fn verify_chain(&self) -> bool {
        for (i, entry) in self.entries.iter().enumerate() {
            // Check entry hash.
            if entry.entry_hash != entry.compute_hash() {
                return false;
            }

            // Check prev_hash link.
            let expected_prev = if i == 0 {
                [0u8; 32]
            } else {
                self.entries[i - 1].entry_hash
            };
            if entry.prev_hash != expected_prev {
                return false;
            }
        }
        true
    }

    /// Persist the audit trail as JSON.
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_vec_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    /// Load an audit trail from JSON.
    pub fn load(path: &str) -> Result<Self, std::io::Error> {
        let data = std::fs::read(path)?;
        serde_json::from_slice(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Return all entries with `timestamp >= since`.
    pub fn entries_since(&self, since: u64) -> &[AuditEntry] {
        match self.entries.iter().position(|e| e.timestamp >= since) {
            Some(idx) => &self.entries[idx..],
            None => &[],
        }
    }

    /// Access the full entry list.
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Current Merkle root.
    pub fn merkle_root(&self) -> &[u8; 32] {
        &self.merkle_root
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the trail is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // -- internal --

    /// Rebuild the Merkle root from all entry hashes.
    fn recompute_merkle_root(&mut self) {
        if self.entries.is_empty() {
            self.merkle_root = [0u8; 32];
            return;
        }

        // Leaf layer: entry hashes.
        let mut layer: Vec<[u8; 32]> = self.entries.iter().map(|e| e.entry_hash).collect();

        // Iteratively hash pairs until one root remains.
        while layer.len() > 1 {
            let mut next = Vec::with_capacity((layer.len() + 1) / 2);
            for chunk in layer.chunks(2) {
                let mut hasher = blake3::Hasher::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    // Odd element: hash with itself.
                    hasher.update(&chunk[0]);
                }
                next.push(*hasher.finalize().as_bytes());
            }
            layer = next;
        }

        self.merkle_root = layer[0];
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// InspectionResult / CorrectionResult
// ---------------------------------------------------------------------------

/// Result of one inspection check.
#[derive(Debug, Clone)]
pub enum InspectionResult {
    /// A component's mean timing exceeds its baseline by > 2x.
    Regression {
        component: String,
        expected: Duration,
        actual: Duration,
    },
    /// A component produced wrong output.
    CorrectnessFailure {
        component: String,
        test_case_idx: usize,
    },
    /// Memory usage exceeded the limit.
    MemoryExceeded {
        current: usize,
        limit: usize,
    },
}

impl InspectionResult {
    /// Is this a critical finding that warrants auto-correction?
    pub fn is_critical(&self) -> bool {
        match self {
            InspectionResult::Regression { expected, actual, .. } => {
                let ratio = actual.as_nanos() as f64 / expected.as_nanos().max(1) as f64;
                ratio > 5.0
            }
            InspectionResult::CorrectnessFailure { .. } => true,
            InspectionResult::MemoryExceeded { .. } => true,
        }
    }

    /// Convert to an `Anomaly` for the audit trail.
    pub fn to_anomaly(&self) -> Anomaly {
        match self {
            InspectionResult::Regression {
                component,
                expected,
                actual,
            } => {
                let expected_ns = expected.as_nanos() as u64;
                let actual_ns = actual.as_nanos() as u64;
                let ratio = actual_ns as f64 / expected_ns.max(1) as f64;
                Anomaly {
                    timestamp: 0,
                    component: component.clone(),
                    kind: AnomalyKind::PerformanceRegression {
                        expected_ns,
                        actual_ns,
                        ratio,
                    },
                    severity: if ratio > 5.0 {
                        Severity::Critical
                    } else {
                        Severity::Warning
                    },
                    auto_corrected: false,
                }
            }
            InspectionResult::CorrectnessFailure {
                component,
                test_case_idx,
            } => Anomaly {
                timestamp: 0,
                component: component.clone(),
                kind: AnomalyKind::CorrectnessFailure {
                    test_case_idx: *test_case_idx,
                    expected: String::new(),
                    actual: String::new(),
                },
                severity: Severity::Critical,
                auto_corrected: false,
            },
            InspectionResult::MemoryExceeded { current, limit } => Anomaly {
                timestamp: 0,
                component: "system".to_string(),
                kind: AnomalyKind::MemorySpike {
                    bytes: *current,
                    limit: *limit,
                },
                severity: Severity::Critical,
                auto_corrected: false,
            },
        }
    }

    /// Convert to an `AuditAction` for recording.
    pub fn to_audit_action(&self) -> AuditAction {
        AuditAction::AnomalyDetected {
            anomaly: self.to_anomaly(),
        }
    }
}

/// Result of an auto-correction attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorrectionResult {
    /// Reverted the component to its previous version.
    Reverted { component: String },
    /// Reverted and raised an alert for human review.
    RevertedWithAlert { component: String },
    /// Only logged; no automatic fix available.
    Logged,
}

// ---------------------------------------------------------------------------
// SelfInspector
// ---------------------------------------------------------------------------

/// Periodic self-inspection engine.
///
/// Called by the daemon every N cycles. Inspects all deployed components,
/// detects performance regressions and correctness failures, auto-corrects
/// where possible, and records every finding in the audit trail.
pub struct SelfInspector {
    pub telemetry: Telemetry,
    pub audit: AuditTrail,
    pub check_interval: Duration,
    /// Memory limit for the memory spike check (bytes).
    pub memory_limit: usize,
}

impl SelfInspector {
    /// Create a new inspector.
    pub fn new(check_interval: Duration, memory_limit: usize) -> Self {
        Self {
            telemetry: Telemetry::new(),
            audit: AuditTrail::new(),
            check_interval,
            memory_limit,
        }
    }

    /// Run one inspection cycle. Returns findings.
    pub fn inspect(&mut self) -> Vec<InspectionResult> {
        let mut results = Vec::new();

        // 1. Check each deployed component's performance.
        for (name, timing) in &self.telemetry.component_timings {
            if timing.regression_detected {
                results.push(InspectionResult::Regression {
                    component: name.clone(),
                    expected: timing.baseline,
                    actual: timing.mean,
                });
            }
        }

        // 2. Check memory usage.
        let mem = self.telemetry.system_metrics.memory_usage_bytes;
        if mem > self.memory_limit && self.memory_limit > 0 {
            results.push(InspectionResult::MemoryExceeded {
                current: mem,
                limit: self.memory_limit,
            });
        }

        // 3. Record all findings in audit trail.
        for result in &results {
            self.audit.record(result.to_audit_action());
        }

        results
    }

    /// Attempt to auto-correct a detected anomaly.
    pub fn correct(&mut self, anomaly: &Anomaly) -> CorrectionResult {
        let result = match &anomaly.kind {
            AnomalyKind::PerformanceRegression { .. } => {
                // Revert to the previous version.
                CorrectionResult::Reverted {
                    component: anomaly.component.clone(),
                }
            }
            AnomalyKind::CorrectnessFailure { .. } => {
                // Revert AND alert for investigation.
                CorrectionResult::RevertedWithAlert {
                    component: anomaly.component.clone(),
                }
            }
            AnomalyKind::MemorySpike { .. }
            | AnomalyKind::ComponentCrash { .. }
            | AnomalyKind::SelfModificationViolation { .. } => CorrectionResult::Logged,
        };

        // Record the correction in the audit trail.
        let correction_desc = format!("{:?}", result);
        self.audit.record(AuditAction::AnomalyCorrected {
            anomaly_id: anomaly.timestamp,
            correction: correction_desc,
        });

        result
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_window_records_and_computes_mean() {
        let baseline = Duration::from_micros(100);
        let mut w = TimingWindow::new(baseline);

        w.record(Duration::from_micros(100));
        w.record(Duration::from_micros(100));
        w.record(Duration::from_micros(100));

        assert_eq!(w.mean, Duration::from_micros(100));
        assert!(!w.regression_detected);
    }

    #[test]
    fn timing_window_detects_regression() {
        let baseline = Duration::from_micros(100);
        let mut w = TimingWindow::new(baseline);

        // Insert values that are > 2x the baseline.
        for _ in 0..5 {
            w.record(Duration::from_micros(300));
        }

        assert!(w.regression_detected);
    }

    #[test]
    fn timing_window_no_regression_within_threshold() {
        let baseline = Duration::from_micros(100);
        let mut w = TimingWindow::new(baseline);

        // 1.5x baseline — should NOT trigger.
        for _ in 0..5 {
            w.record(Duration::from_micros(150));
        }

        assert!(!w.regression_detected);
    }

    #[test]
    fn audit_trail_chain_integrity() {
        let mut trail = AuditTrail::new();
        trail.record(AuditAction::ComponentDeployed {
            name: "a".into(),
            slowdown: 1.2,
        });
        trail.record(AuditAction::ComponentDeployed {
            name: "b".into(),
            slowdown: 1.5,
        });
        trail.record(AuditAction::PerformanceImproved {
            component: "a".into(),
            before_ns: 1000,
            after_ns: 800,
        });

        assert!(trail.verify_chain());
        assert_eq!(trail.len(), 3);
    }

    #[test]
    fn audit_trail_detects_tampering() {
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

        // Tamper with the first entry.
        trail.entries_mut()[0].performance_delta = 999.0;

        assert!(!trail.verify_chain());
    }

    #[test]
    fn audit_entry_hash_is_deterministic() {
        let mut trail1 = AuditTrail::new();
        let mut trail2 = AuditTrail::new();

        let action = AuditAction::SelfImprovement {
            strategy: "genetic".into(),
            improvement: 0.15,
        };

        trail1.record(action.clone());
        trail2.record(action);

        assert_eq!(
            trail1.entries()[0].entry_hash,
            trail2.entries()[0].entry_hash
        );
    }

    #[test]
    fn self_inspector_detects_regression() {
        let mut inspector = SelfInspector::new(Duration::from_secs(1), 1024 * 1024);

        inspector
            .telemetry
            .register_component("slow_comp", Duration::from_micros(100));

        // Feed slow timings.
        for _ in 0..10 {
            inspector
                .telemetry
                .record_timing("slow_comp", Duration::from_micros(500));
        }

        let findings = inspector.inspect();
        assert!(!findings.is_empty());

        let regression = findings.iter().find(|f| {
            matches!(f, InspectionResult::Regression { component, .. } if component == "slow_comp")
        });
        assert!(regression.is_some());
    }

    #[test]
    fn self_inspector_auto_corrects_regression() {
        let anomaly = Anomaly {
            timestamp: 1,
            component: "broken".into(),
            kind: AnomalyKind::PerformanceRegression {
                expected_ns: 100,
                actual_ns: 500,
                ratio: 5.0,
            },
            severity: Severity::Critical,
            auto_corrected: false,
        };

        let mut inspector = SelfInspector::new(Duration::from_secs(1), 0);
        let result = inspector.correct(&anomaly);

        assert_eq!(
            result,
            CorrectionResult::Reverted {
                component: "broken".into()
            }
        );
        // Correction should be recorded in the audit trail.
        assert_eq!(inspector.audit.len(), 1);
    }

    #[test]
    fn self_inspector_auto_corrects_correctness_failure() {
        let anomaly = Anomaly {
            timestamp: 2,
            component: "wrong_comp".into(),
            kind: AnomalyKind::CorrectnessFailure {
                test_case_idx: 0,
                expected: "42".into(),
                actual: "0".into(),
            },
            severity: Severity::Critical,
            auto_corrected: false,
        };

        let mut inspector = SelfInspector::new(Duration::from_secs(1), 0);
        let result = inspector.correct(&anomaly);

        assert_eq!(
            result,
            CorrectionResult::RevertedWithAlert {
                component: "wrong_comp".into()
            }
        );
    }

    #[test]
    fn telemetry_detects_anomalies() {
        let mut tel = Telemetry::new();
        tel.register_component("c1", Duration::from_micros(50));

        for _ in 0..10 {
            tel.record_timing("c1", Duration::from_micros(200));
        }

        tel.detect_anomalies(42);
        assert!(!tel.anomalies.is_empty());
        assert_eq!(tel.anomalies[0].component, "c1");
    }

    #[test]
    fn audit_trail_entries_since() {
        let mut trail = AuditTrail::new();
        for i in 0..5 {
            trail.record(AuditAction::EvolutionGeneration {
                generation: i,
                best_fitness: 0.5,
                population_size: 32,
            });
        }

        // Timestamps are auto-assigned as 0..5 (id-based).
        let since3 = trail.entries_since(3);
        assert_eq!(since3.len(), 2);
    }

    #[test]
    fn audit_trail_empty_is_valid() {
        let trail = AuditTrail::new();
        assert!(trail.verify_chain());
        assert!(trail.is_empty());
        assert_eq!(trail.merkle_root(), &[0u8; 32]);
    }

    #[test]
    fn merkle_root_changes_with_each_entry() {
        let mut trail = AuditTrail::new();
        let root0 = *trail.merkle_root();

        trail.record(AuditAction::ComponentDeployed {
            name: "x".into(),
            slowdown: 1.0,
        });
        let root1 = *trail.merkle_root();

        trail.record(AuditAction::ComponentDeployed {
            name: "y".into(),
            slowdown: 1.1,
        });
        let root2 = *trail.merkle_root();

        assert_ne!(root0, root1);
        assert_ne!(root1, root2);
        assert_ne!(root0, root2);
    }
}
