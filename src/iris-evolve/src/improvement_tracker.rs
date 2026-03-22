//! Recursive improvement tracking and compounding metrics (Phase Transition 3).
//!
//! Tracks whether self-improvement is COMPOUNDING — each cycle faster than the
//! last, with the rate of improvement itself increasing. This is the key metric
//! for Phase Transition 3: Slow -> Recursive.
//!
//! Provides:
//! - Per-component improvement rate (linear regression slope of latency)
//! - Acceleration (slope of improvement rate over time — is the rate increasing?)
//! - Causal attribution: which mutation operators cause improvements
//! - Problems-per-hour tracking
//! - Adaptive operator weighting based on empirical contribution

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::self_improve::MUTATION_OPERATOR_NAMES;

// ---------------------------------------------------------------------------
// OperatorStats
// ---------------------------------------------------------------------------

/// Causal attribution statistics for a single mutation operator.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperatorStats {
    /// Total number of times this operator was applied.
    pub times_used: u64,
    /// Number of times this operator led to a fitness improvement.
    pub improvements_caused: u64,
    /// Sum of fitness deltas across all improvements caused by this operator.
    pub total_fitness_delta: f64,
    /// Average improvement per successful application (total_delta / improvements_caused).
    pub avg_improvement: f64,
}

impl OperatorStats {
    /// Record an application of this operator.
    ///
    /// `fitness_delta` is the change in fitness compared to the parent.
    /// Positive means the offspring is better; zero or negative means no improvement.
    pub fn record(&mut self, fitness_delta: f64) {
        self.times_used += 1;
        if fitness_delta > 0.0 {
            self.improvements_caused += 1;
            self.total_fitness_delta += fitness_delta;
            self.avg_improvement = if self.improvements_caused > 0 {
                self.total_fitness_delta / self.improvements_caused as f64
            } else {
                0.0
            };
        }
    }

    /// Success rate: fraction of applications that caused improvement.
    pub fn success_rate(&self) -> f64 {
        if self.times_used == 0 {
            0.0
        } else {
            self.improvements_caused as f64 / self.times_used as f64
        }
    }
}

// ---------------------------------------------------------------------------
// ImprovementTracker
// ---------------------------------------------------------------------------

/// Tracks recursive improvement metrics for Phase Transition 3.
///
/// Measures whether self-improvement is compounding by computing:
/// - Improvement rate per component (slope of latency reduction over cycles)
/// - Acceleration (slope of improvement rate — is the rate itself increasing?)
/// - Causal attribution: which operators contribute most to improvements
/// - Problems solved per unit time
#[derive(Debug, Clone)]
pub struct ImprovementTracker {
    /// Per-component measurement history: Vec of (timestamp_ms, latency_ns, correctness).
    history: HashMap<String, Vec<(u64, u64, f64)>>,

    /// Per-component improvement rate (latency delta per cycle, computed via linear regression).
    /// Negative values mean latency is decreasing (good).
    improvement_rate: HashMap<String, f64>,

    /// Per-component acceleration: rate of change of improvement_rate.
    /// Negative values mean improvement is accelerating (latency dropping faster and faster).
    acceleration: HashMap<String, f64>,

    /// Cumulative problems solved over time: (timestamp_ms, cumulative_count).
    problems_solved_over_time: Vec<(u64, usize)>,

    /// Per-operator causal attribution.
    operator_contributions: HashMap<String, OperatorStats>,

    /// Sliding window size for improvement rate computation.
    window_size: usize,

    /// History of improvement rates for acceleration computation.
    /// Per-component: Vec of (timestamp_ms, improvement_rate).
    rate_history: HashMap<String, Vec<(u64, f64)>>,
}

impl ImprovementTracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        Self::with_window(20)
    }

    /// Create a tracker with a specific sliding window size.
    pub fn with_window(window_size: usize) -> Self {
        let mut operator_contributions = HashMap::new();
        for name in &MUTATION_OPERATOR_NAMES {
            operator_contributions.insert(name.to_string(), OperatorStats::default());
        }

        Self {
            history: HashMap::new(),
            improvement_rate: HashMap::new(),
            acceleration: HashMap::new(),
            problems_solved_over_time: Vec::new(),
            operator_contributions,
            window_size: window_size.max(3), // minimum 3 points for regression
            rate_history: HashMap::new(),
        }
    }

    /// Record a measurement for a component.
    ///
    /// `timestamp_ms` — wall-clock time in milliseconds since some epoch.
    /// `latency_ns` — execution latency in nanoseconds.
    /// `correctness` — correctness score in [0.0, 1.0].
    pub fn record_measurement(
        &mut self,
        component: &str,
        timestamp_ms: u64,
        latency_ns: u64,
        correctness: f64,
    ) {
        let entries = self
            .history
            .entry(component.to_string())
            .or_insert_with(Vec::new);
        entries.push((timestamp_ms, latency_ns, correctness));

        // Recompute improvement rate for this component.
        self.recompute_rate(component, timestamp_ms);
    }

    /// Record a problem being solved.
    pub fn record_problem_solved(&mut self, timestamp_ms: u64) {
        let current_count = self
            .problems_solved_over_time
            .last()
            .map(|(_, c)| *c)
            .unwrap_or(0);
        self.problems_solved_over_time
            .push((timestamp_ms, current_count + 1));
    }

    /// Record a mutation operator application and its fitness effect.
    ///
    /// `operator_id` is the index (0..15) into MUTATION_OPERATOR_NAMES.
    /// `fitness_delta` is (offspring_fitness - parent_fitness).
    pub fn record_operator_application(&mut self, operator_id: u8, fitness_delta: f64) {
        let name = operator_name(operator_id);
        let stats = self
            .operator_contributions
            .entry(name.to_string())
            .or_insert_with(OperatorStats::default);
        stats.record(fitness_delta);
    }

    /// Get the improvement rate for a component.
    ///
    /// Returns the slope of latency over recent measurements (via linear
    /// regression). A negative value means latency is decreasing (good).
    /// Returns `None` if insufficient data.
    pub fn improvement_rate(&self, component: &str) -> Option<f64> {
        self.improvement_rate.get(component).copied()
    }

    /// Get the acceleration for a component.
    ///
    /// Acceleration is the slope of the improvement rate over time.
    /// A negative value means latency reduction is speeding up (compounding).
    /// Returns `None` if insufficient data.
    pub fn acceleration(&self, component: &str) -> Option<f64> {
        self.acceleration.get(component).copied()
    }

    /// Whether improvement is compounding for any component.
    ///
    /// Compounding means the rate of latency reduction is itself increasing
    /// (acceleration < 0 when measuring latency, since lower is better).
    /// We report compounding when acceleration is negative (latency dropping
    /// faster and faster).
    pub fn is_compounding(&self) -> bool {
        self.acceleration.values().any(|&a| a < -1e-9)
    }

    /// Compute the problems-per-hour rate.
    ///
    /// Uses the first and last entries in the solved-over-time history.
    pub fn problems_per_hour(&self) -> f64 {
        if self.problems_solved_over_time.len() < 2 {
            return 0.0;
        }

        let (first_ts, first_count) = self.problems_solved_over_time.first().unwrap();
        let (last_ts, last_count) = self.problems_solved_over_time.last().unwrap();

        let elapsed_ms = last_ts.saturating_sub(*first_ts);
        if elapsed_ms == 0 {
            return 0.0;
        }

        let problems_delta = last_count.saturating_sub(*first_count) as f64;
        let hours = elapsed_ms as f64 / (3_600_000.0); // ms -> hours
        problems_delta / hours
    }

    /// Get all operator contributions.
    pub fn operator_contributions(&self) -> &HashMap<String, OperatorStats> {
        &self.operator_contributions
    }

    /// Get operator contributions as a sorted Vec (by total_fitness_delta, descending).
    pub fn operator_contributions_sorted(&self) -> Vec<(String, OperatorStats)> {
        let mut entries: Vec<_> = self
            .operator_contributions
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        entries.sort_by(|a, b| {
            b.1.total_fitness_delta
                .partial_cmp(&a.1.total_fitness_delta)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries
    }

    /// Compute adaptive operator weights based on empirical contribution.
    ///
    /// Returns normalized weights [0..1] for each operator, indexed by operator ID.
    /// Operators that cause more improvements get higher weight.
    /// Uses a blend of success rate and total fitness delta for robustness.
    ///
    /// `baseline_weight` is the minimum weight floor (prevents operators from
    /// being completely eliminated before they have enough samples).
    /// `min_samples` is the minimum number of uses before an operator's
    /// empirical data is trusted (below this, baseline weight is used).
    pub fn adaptive_weights(
        &self,
        baseline_weight: f64,
        min_samples: u64,
    ) -> Vec<(String, f64)> {
        let mut raw_weights: Vec<(String, f64)> = Vec::with_capacity(MUTATION_OPERATOR_NAMES.len());

        for name in &MUTATION_OPERATOR_NAMES {
            let stats = self.operator_contributions.get(*name);
            let weight = match stats {
                Some(s) if s.times_used >= min_samples => {
                    // Blend success rate (0..1) with average improvement magnitude.
                    // Success rate drives exploration of effective operators.
                    // Average improvement drives exploitation of high-delta operators.
                    let success_rate = s.success_rate();
                    let avg_imp = s.avg_improvement.min(1.0); // cap at 1.0
                    let empirical = 0.6 * success_rate + 0.4 * avg_imp;
                    // Blend with baseline: 70% empirical, 30% baseline.
                    0.7 * empirical + 0.3 * baseline_weight
                }
                _ => baseline_weight,
            };
            raw_weights.push((name.to_string(), weight.max(baseline_weight * 0.1)));
        }

        // Normalize so weights sum to 1.0.
        let sum: f64 = raw_weights.iter().map(|(_, w)| w).sum();
        if sum > 0.0 {
            for (_, w) in &mut raw_weights {
                *w /= sum;
            }
        }

        raw_weights
    }

    /// Install adaptive weights into the mutation system's thread-local
    /// override. This creates a feedback loop: measure -> weight -> improve ->
    /// measure.
    ///
    /// Returns `true` if weights were installed (i.e., there was enough data
    /// to compute non-uniform weights).
    pub fn install_adaptive_weights(&self, min_samples: u64) -> bool {
        let weights = self.adaptive_weights(1.0 / MUTATION_OPERATOR_NAMES.len() as f64, min_samples);

        // Check if weights are meaningfully non-uniform (at least one operator
        // has been used enough to shift weights).
        let any_empirical = self.operator_contributions.values().any(|s| s.times_used >= min_samples);
        if !any_empirical {
            return false;
        }

        // Convert to cumulative thresholds for mutation::set_custom_weights.
        let mut cumulative: Vec<(u8, f64)> = Vec::with_capacity(weights.len());
        let mut acc = 0.0f64;
        for (i, (_, w)) in weights.iter().enumerate() {
            acc += w;
            cumulative.push((i as u8, acc));
        }
        // Ensure last entry reaches 1.0.
        if let Some(last) = cumulative.last_mut() {
            last.1 = 1.0;
        }

        // Best-effort: if validation fails (shouldn't happen with well-formed
        // weight vectors), leave current weights unchanged.
        let _ = crate::mutation::set_custom_weights(&cumulative);
        true
    }

    /// Clear any installed adaptive weights, reverting to defaults.
    pub fn clear_adaptive_weights(&self) {
        crate::mutation::clear_custom_weights();
    }

    /// Get the measurement history for a component.
    pub fn history(&self, component: &str) -> Option<&Vec<(u64, u64, f64)>> {
        self.history.get(component)
    }

    /// Get the problems-solved-over-time history.
    pub fn problems_solved_history(&self) -> &[(u64, usize)] {
        &self.problems_solved_over_time
    }

    /// Total number of problems solved.
    pub fn total_problems_solved(&self) -> usize {
        self.problems_solved_over_time
            .last()
            .map(|(_, c)| *c)
            .unwrap_or(0)
    }

    /// Get all component names that have recorded history.
    pub fn component_names(&self) -> Vec<String> {
        self.history.keys().cloned().collect()
    }

    /// Format a human-readable summary of the current state.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push("=== Improvement Tracker Summary ===".to_string());

        // Per-component rates.
        for (component, rate) in &self.improvement_rate {
            let accel = self.acceleration.get(component).copied().unwrap_or(0.0);
            let status = if accel < -1e-9 {
                "COMPOUNDING"
            } else if accel > 1e-9 {
                "DECELERATING"
            } else {
                "STEADY"
            };
            lines.push(format!(
                "  {}: rate={:.6} ns/cycle, accel={:.6} ns/cycle^2 ({})",
                component, rate, accel, status,
            ));
        }

        // Problems per hour.
        lines.push(format!(
            "  Problems/hour: {:.1}",
            self.problems_per_hour()
        ));

        // Top operators.
        let sorted = self.operator_contributions_sorted();
        lines.push("  Top operators:".to_string());
        for (name, stats) in sorted.iter().take(5) {
            if stats.times_used > 0 {
                lines.push(format!(
                    "    {}: used={}, improved={}, delta={:.4}, success={:.1}%",
                    name,
                    stats.times_used,
                    stats.improvements_caused,
                    stats.total_fitness_delta,
                    stats.success_rate() * 100.0,
                ));
            }
        }

        // Compounding status.
        if self.is_compounding() {
            lines.push("  STATUS: COMPOUNDING (improvement rate accelerating)".to_string());
        } else {
            lines.push("  STATUS: Not yet compounding".to_string());
        }

        lines.join("\n")
    }

    // -----------------------------------------------------------------------
    // Internal: recompute improvement rate via linear regression
    // -----------------------------------------------------------------------

    fn recompute_rate(&mut self, component: &str, current_timestamp_ms: u64) {
        let entries = match self.history.get(component) {
            Some(e) => e,
            None => return,
        };

        // Use the last `window_size` entries for regression.
        let n = entries.len().min(self.window_size);
        if n < 3 {
            // Not enough data for meaningful regression.
            return;
        }

        let window = &entries[entries.len() - n..];

        // Linear regression: y = latency_ns, x = index (0..n-1).
        // We use index-based x rather than timestamp to get "per-cycle" slope.
        let slope = linear_regression_slope(window.iter().enumerate().map(|(i, &(_, lat, _))| {
            (i as f64, lat as f64)
        }));

        let _old_rate = self.improvement_rate.get(component).copied();
        self.improvement_rate
            .insert(component.to_string(), slope);

        // Record rate in rate_history for acceleration computation.
        let rate_entries = self
            .rate_history
            .entry(component.to_string())
            .or_insert_with(Vec::new);
        rate_entries.push((current_timestamp_ms, slope));

        // Compute acceleration: slope of improvement_rate over time.
        let rate_n = rate_entries.len().min(self.window_size);
        if rate_n >= 3 {
            let rate_window = &rate_entries[rate_entries.len() - rate_n..];
            let accel = linear_regression_slope(
                rate_window
                    .iter()
                    .enumerate()
                    .map(|(i, &(_, r))| (i as f64, r)),
            );
            self.acceleration.insert(component.to_string(), accel);
        }
    }
}

impl Default for ImprovementTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Linear regression helper
// ---------------------------------------------------------------------------

/// Compute the slope of a simple linear regression y = a + b*x.
///
/// Takes an iterator of (x, y) pairs. Returns the slope `b`.
/// Uses the formula: b = (n*sum(xy) - sum(x)*sum(y)) / (n*sum(x^2) - sum(x)^2)
fn linear_regression_slope(points: impl Iterator<Item = (f64, f64)>) -> f64 {
    let mut n = 0.0_f64;
    let mut sum_x = 0.0_f64;
    let mut sum_y = 0.0_f64;
    let mut sum_xy = 0.0_f64;
    let mut sum_x2 = 0.0_f64;

    for (x, y) in points {
        n += 1.0;
        sum_x += x;
        sum_y += y;
        sum_xy += x * y;
        sum_x2 += x * x;
    }

    if n < 2.0 {
        return 0.0;
    }

    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-15 {
        return 0.0;
    }

    (n * sum_xy - sum_x * sum_y) / denom
}

// ---------------------------------------------------------------------------
// Operator name helper
// ---------------------------------------------------------------------------

/// Get the operator name for a given operator ID.
pub fn operator_name(op_id: u8) -> &'static str {
    let idx = op_id as usize;
    if idx < MUTATION_OPERATOR_NAMES.len() {
        MUTATION_OPERATOR_NAMES[idx]
    } else {
        "unknown"
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_regression_slope_positive() {
        // y = 2x: points (0,0), (1,2), (2,4), (3,6)
        let slope = linear_regression_slope(
            [(0.0, 0.0), (1.0, 2.0), (2.0, 4.0), (3.0, 6.0)]
                .iter()
                .copied(),
        );
        assert!((slope - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_linear_regression_slope_negative() {
        // y = -3x + 10: points (0,10), (1,7), (2,4), (3,1)
        let slope = linear_regression_slope(
            [(0.0, 10.0), (1.0, 7.0), (2.0, 4.0), (3.0, 1.0)]
                .iter()
                .copied(),
        );
        assert!((slope - (-3.0)).abs() < 1e-10);
    }

    #[test]
    fn test_linear_regression_slope_flat() {
        let slope = linear_regression_slope(
            [(0.0, 5.0), (1.0, 5.0), (2.0, 5.0), (3.0, 5.0)]
                .iter()
                .copied(),
        );
        assert!(slope.abs() < 1e-10);
    }

    #[test]
    fn test_operator_stats_record() {
        let mut stats = OperatorStats::default();
        stats.record(0.5); // improvement
        assert_eq!(stats.times_used, 1);
        assert_eq!(stats.improvements_caused, 1);
        assert!((stats.total_fitness_delta - 0.5).abs() < 1e-10);
        assert!((stats.avg_improvement - 0.5).abs() < 1e-10);

        stats.record(-0.1); // no improvement
        assert_eq!(stats.times_used, 2);
        assert_eq!(stats.improvements_caused, 1); // still 1
        assert!((stats.success_rate() - 0.5).abs() < 1e-10);

        stats.record(0.3); // improvement
        assert_eq!(stats.times_used, 3);
        assert_eq!(stats.improvements_caused, 2);
        assert!((stats.total_fitness_delta - 0.8).abs() < 1e-10);
        assert!((stats.avg_improvement - 0.4).abs() < 1e-10);
    }

    #[test]
    fn test_improvement_tracker_rate_decreasing_latency() {
        let mut tracker = ImprovementTracker::with_window(10);
        // Simulate latency decreasing over time: 1000, 900, 800, 700, 600
        for i in 0..5 {
            tracker.record_measurement(
                "test_comp",
                i * 100,
                1000 - i * 100,
                1.0,
            );
        }
        let rate = tracker.improvement_rate("test_comp");
        assert!(rate.is_some());
        assert!(rate.unwrap() < 0.0, "rate should be negative (latency decreasing)");
    }

    #[test]
    fn test_problems_per_hour() {
        let mut tracker = ImprovementTracker::new();
        // 10 problems in 1 hour (3,600,000 ms)
        tracker.problems_solved_over_time.push((0, 0));
        tracker.problems_solved_over_time.push((3_600_000, 10));

        let rate = tracker.problems_per_hour();
        assert!((rate - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_adaptive_weights_uniform_with_no_data() {
        let tracker = ImprovementTracker::new();
        let weights = tracker.adaptive_weights(1.0 / 16.0, 10);
        // All operators at baseline -> all equal weight
        let expected = 1.0 / MUTATION_OPERATOR_NAMES.len() as f64;
        for (_, w) in &weights {
            assert!(
                (w - expected).abs() < 0.01,
                "expected ~{}, got {}",
                expected,
                w,
            );
        }
    }

    #[test]
    fn test_adaptive_weights_favors_successful_operator() {
        let mut tracker = ImprovementTracker::new();
        // Make "replace_prim" very successful.
        for _ in 0..20 {
            tracker.record_operator_application(4, 0.5); // replace_prim
        }
        // Make "insert_node" unsuccessful.
        for _ in 0..20 {
            tracker.record_operator_application(0, -0.1); // insert_node
        }

        let weights = tracker.adaptive_weights(1.0 / 16.0, 10);
        let replace_prim_weight = weights.iter().find(|(n, _)| n == "replace_prim").unwrap().1;
        let insert_node_weight = weights.iter().find(|(n, _)| n == "insert_node").unwrap().1;
        assert!(
            replace_prim_weight > insert_node_weight,
            "replace_prim ({}) should have higher weight than insert_node ({})",
            replace_prim_weight,
            insert_node_weight,
        );
    }

    #[test]
    fn test_operator_name() {
        assert_eq!(operator_name(0), "insert_node");
        assert_eq!(operator_name(4), "replace_prim");
        assert_eq!(operator_name(13), "swap_fold_op");
        assert_eq!(operator_name(15), "extract_to_ref");
        assert_eq!(operator_name(255), "unknown");
    }
}
