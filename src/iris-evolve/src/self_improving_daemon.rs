//! Self-improving daemon: wires together the IrisDaemon, AutoImprover,
//! SelfInspector, IrisRuntime, and persistence into a single continuously
//! running process that profiles, evolves, gates, deploys, inspects, and
//! persists IRIS components.
//!
//! Architecture:
//! ```text
//! ThreadedDaemon (criteria-driven improvement)
//!   +-- ImprovementPool (bounded thread pool for background evolution)
//!   +-- ComponentMetrics (lock-free per-component metrics)
//!   +-- ImproveTrigger (criteria that trigger improvement)
//!   +-- StagnationDetector (per-component convergence tracking)
//!   +-- ConvergenceDetector (system-wide local maximum detection)
//!   +-- AutoImprover (profile -> evolve -> gate -> deploy)
//!   +-- SelfInspector (detect regressions -> auto-correct -> audit)
//!   +-- IrisRuntime (hot-swap deployed components)
//!   +-- Persistence (save/restore state to disk as JSON)
//! ```

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use iris_exec::service::IrisExecutionService;
use iris_types::graph::{NodePayload, SemanticGraph};

use crate::auto_improve::{
    AutoImproveConfig, AutoImprover, ImprovementAction, ImprovementEvent,
};
use crate::improvement_tracker::ImprovementTracker;
use crate::instrumentation::{
    AuditAction, CorrectionResult, InspectionResult, SelfInspector,
};
use crate::iris_runtime::IrisRuntime;

// ---------------------------------------------------------------------------
// ExecMode
// ---------------------------------------------------------------------------

/// Execution mode for the daemon's main loop.
#[derive(Debug, Clone)]
pub enum ExecMode {
    /// Sleep for a fixed interval between cycles.
    FixedInterval(Duration),
    /// Run continuously with no sleep (for benchmarks / testing).
    Continuous,
}

impl Default for ExecMode {
    fn default() -> Self {
        ExecMode::FixedInterval(Duration::from_millis(800))
    }
}

// ---------------------------------------------------------------------------
// ComponentMetrics
// ---------------------------------------------------------------------------

/// Per-component metrics collection.
///
/// Uses `Mutex<HashMap<...>>` for thread safety (no external dependency).
/// Global counters use atomics for lock-free access.
pub struct ComponentMetrics {
    /// Per-component sliding window of last 100 latencies.
    latencies: Mutex<HashMap<String, VecDeque<Duration>>>,
    /// Per-component correctness rate (exponential moving average).
    correctness: Mutex<HashMap<String, f64>>,
    /// Global counters.
    pub total_executions: AtomicU64,
    pub total_improvements: AtomicU64,
    pub total_rollbacks: AtomicU64,
}

const LATENCY_WINDOW_SIZE: usize = 100;

impl ComponentMetrics {
    /// Create empty metrics.
    pub fn new() -> Self {
        Self {
            latencies: Mutex::new(HashMap::new()),
            correctness: Mutex::new(HashMap::new()),
            total_executions: AtomicU64::new(0),
            total_improvements: AtomicU64::new(0),
            total_rollbacks: AtomicU64::new(0),
        }
    }

    /// Record a latency sample for a component.
    pub fn record_latency(&self, component: &str, duration: Duration) {
        let mut map = self.latencies.lock().unwrap();
        let window = map
            .entry(component.to_string())
            .or_insert_with(|| VecDeque::with_capacity(LATENCY_WINDOW_SIZE + 1));
        window.push_back(duration);
        if window.len() > LATENCY_WINDOW_SIZE {
            window.pop_front();
        }
    }

    /// Record a correctness observation (exponential moving average, alpha=0.1).
    pub fn record_correctness(&self, component: &str, correct: bool) {
        let alpha = 0.1;
        let value = if correct { 1.0 } else { 0.0 };
        let mut map = self.correctness.lock().unwrap();
        let rate = map.entry(component.to_string()).or_insert(1.0);
        *rate = alpha * value + (1.0 - alpha) * *rate;
    }

    /// Get the p99 latency for a component.
    pub fn p99_latency(&self, component: &str) -> Option<Duration> {
        let map = self.latencies.lock().unwrap();
        let window = map.get(component)?;
        if window.is_empty() {
            return None;
        }
        let mut sorted: Vec<Duration> = window.iter().copied().collect();
        sorted.sort();
        let idx = ((sorted.len() as f64) * 0.99).ceil() as usize;
        let idx = idx.min(sorted.len()).saturating_sub(1);
        Some(sorted[idx])
    }

    /// Get the mean latency for a component.
    pub fn mean_latency(&self, component: &str) -> Option<Duration> {
        let map = self.latencies.lock().unwrap();
        let window = map.get(component)?;
        if window.is_empty() {
            return None;
        }
        let total: Duration = window.iter().sum();
        Some(total / window.len() as u32)
    }

    /// Get the correctness rate for a component.
    pub fn correctness_rate(&self, component: &str) -> Option<f64> {
        let map = self.correctness.lock().unwrap();
        map.get(component).copied()
    }

    /// Get all component names that have recorded latencies.
    pub fn component_names(&self) -> Vec<String> {
        let map = self.latencies.lock().unwrap();
        map.keys().cloned().collect()
    }

    /// Check if a component has enough samples for meaningful statistics.
    pub fn has_sufficient_samples(&self, component: &str, min_samples: usize) -> bool {
        let map = self.latencies.lock().unwrap();
        map.get(component)
            .map(|w| w.len() >= min_samples)
            .unwrap_or(false)
    }
}

impl Default for ComponentMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ImproveTrigger
// ---------------------------------------------------------------------------

/// Criteria that trigger an improvement attempt for a specific component.
#[derive(Debug, Clone)]
pub enum ImproveTrigger {
    /// Component's p99 latency exceeds a threshold.
    LatencyThreshold {
        component: String,
        max_p99: Duration,
    },
    /// Component's correctness rate dropped below a threshold.
    CorrectnessBelow {
        component: String,
        min_rate: f64,
    },
    /// System memory usage exceeds a threshold.
    MemoryExceeded {
        max_bytes: usize,
    },
    /// Periodic fallback: trigger improvement every N executions.
    Periodic {
        interval: u64,
    },
}

impl ImproveTrigger {
    /// Check whether this trigger fires given the current metrics and execution count.
    ///
    /// Returns `Some(component_name)` if the trigger fires, identifying which
    /// component should be improved.
    pub fn check(
        &self,
        metrics: &ComponentMetrics,
        execution_count: u64,
    ) -> Option<String> {
        match self {
            ImproveTrigger::LatencyThreshold { component, max_p99 } => {
                if !metrics.has_sufficient_samples(component, 10) {
                    return None;
                }
                let p99 = metrics.p99_latency(component)?;
                if p99 > *max_p99 {
                    Some(component.clone())
                } else {
                    None
                }
            }
            ImproveTrigger::CorrectnessBelow { component, min_rate } => {
                let rate = metrics.correctness_rate(component)?;
                if rate < *min_rate {
                    Some(component.clone())
                } else {
                    None
                }
            }
            ImproveTrigger::MemoryExceeded { max_bytes } => {
                // Use a rough approximation of memory via allocated atomic counter.
                // In production this would read /proc/self/statm or similar.
                // For now, we don't have a reliable cross-platform way, so this
                // trigger is a placeholder that never fires in tests.
                let _ = max_bytes;
                None
            }
            ImproveTrigger::Periodic { interval } => {
                if *interval > 0 && execution_count % interval == 0 && execution_count > 0 {
                    // Periodic triggers don't target a specific component.
                    // Return a sentinel that tells the daemon to pick the slowest.
                    Some("__periodic__".to_string())
                } else {
                    None
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// StagnationDetector
// ---------------------------------------------------------------------------

/// Tracks consecutive improvement attempts with insufficient gain,
/// per component. When a component hits `max_stagnant`, it is marked
/// as converged and no further improvement is attempted.
pub struct StagnationDetector {
    /// Per-component: consecutive improvement attempts with < min_improvement gain.
    attempts: HashMap<String, u32>,
    /// Maximum consecutive stagnant attempts before marking converged.
    pub max_stagnant: u32,
    /// Minimum improvement ratio (e.g. 0.05 = 5%) to count as progress.
    pub min_improvement: f64,
    /// Components that have converged (no more improvement possible).
    pub converged: HashSet<String>,
}

impl StagnationDetector {
    /// Create a new detector with the given thresholds.
    pub fn new(max_stagnant: u32, min_improvement: f64) -> Self {
        Self {
            attempts: HashMap::new(),
            max_stagnant,
            min_improvement,
            converged: HashSet::new(),
        }
    }

    /// Record an improvement attempt result.
    ///
    /// `improvement_ratio` is the relative improvement (e.g. 0.10 = 10% faster).
    /// Returns `true` if the component was just marked as converged.
    pub fn record_attempt(&mut self, component: &str, improvement_ratio: f64) -> bool {
        if self.converged.contains(component) {
            return false; // already converged, nothing to do
        }

        if improvement_ratio >= self.min_improvement {
            // Real improvement — reset counter.
            self.attempts.insert(component.to_string(), 0);
            false
        } else {
            // Stagnant attempt.
            let count = self
                .attempts
                .entry(component.to_string())
                .or_insert(0);
            *count += 1;
            if *count >= self.max_stagnant {
                self.converged.insert(component.to_string());
                true
            } else {
                false
            }
        }
    }

    /// Whether a component has converged.
    pub fn is_converged(&self, component: &str) -> bool {
        self.converged.contains(component)
    }

    /// Number of consecutive stagnant attempts for a component.
    pub fn stagnant_count(&self, component: &str) -> u32 {
        self.attempts.get(component).copied().unwrap_or(0)
    }

    /// Number of converged components.
    pub fn converged_count(&self) -> usize {
        self.converged.len()
    }
}

impl Default for StagnationDetector {
    fn default() -> Self {
        Self::new(5, 0.05)
    }
}

// ---------------------------------------------------------------------------
// ConvergenceDetector
// ---------------------------------------------------------------------------

/// Detects system-wide local maximum: all components are either converged
/// or already faster than their Rust baselines.
pub struct ConvergenceDetector {
    /// Components known to be faster than Rust (no improvement needed).
    faster_than_rust: HashSet<String>,
}

impl ConvergenceDetector {
    pub fn new() -> Self {
        Self {
            faster_than_rust: HashSet::new(),
        }
    }

    /// Mark a component as already faster than Rust.
    pub fn mark_faster_than_rust(&mut self, component: &str) {
        self.faster_than_rust.insert(component.to_string());
    }

    /// Check whether ALL known components have converged or are faster than Rust.
    ///
    /// `all_components` is the full set of component names in the system.
    pub fn is_fully_converged(
        &self,
        all_components: &[String],
        stagnation: &StagnationDetector,
    ) -> bool {
        if all_components.is_empty() {
            return false; // no components = not converged, just empty
        }

        all_components.iter().all(|name| {
            stagnation.is_converged(name) || self.faster_than_rust.contains(name)
        })
    }
}

impl Default for ConvergenceDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ImprovementPool
// ---------------------------------------------------------------------------

/// Bounded thread pool for background improvement work.
///
/// Uses `std::thread::spawn` with an atomic counter to limit concurrency.
/// Tracks which components are being improved to avoid duplicates.
pub struct ImprovementPool {
    /// Maximum concurrent improvement threads.
    pub max_concurrent: usize,
    /// Currently active improvement threads.
    active: Arc<AtomicUsize>,
    /// Components currently being improved (prevents duplicate work).
    in_progress: Arc<Mutex<HashSet<String>>>,
}

impl ImprovementPool {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            active: Arc::new(AtomicUsize::new(0)),
            in_progress: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// How many improvement threads are currently active.
    pub fn active_count(&self) -> usize {
        self.active.load(Ordering::Relaxed)
    }

    /// Whether a component is currently being improved.
    pub fn is_in_progress(&self, component: &str) -> bool {
        let set = self.in_progress.lock().unwrap();
        set.contains(component)
    }

    /// Whether we can spawn another improvement thread.
    pub fn can_spawn(&self) -> bool {
        self.active.load(Ordering::Relaxed) < self.max_concurrent
    }

    /// Try to start an improvement for a component.
    ///
    /// Returns `false` if at capacity or the component is already being improved.
    /// On success, marks the component as in-progress and increments active count.
    pub fn try_start(&self, component: &str) -> bool {
        if !self.can_spawn() {
            return false;
        }

        let mut set = self.in_progress.lock().unwrap();
        if set.contains(component) {
            return false;
        }

        // Reserve the slot.
        set.insert(component.to_string());
        self.active.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// Mark an improvement as finished.
    pub fn finish(&self, component: &str) {
        let mut set = self.in_progress.lock().unwrap();
        set.remove(component);
        self.active.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get a clone of the active counter for thread use.
    pub fn active_counter(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.active)
    }

    /// Get a clone of the in-progress set for thread use.
    pub fn in_progress_set(&self) -> Arc<Mutex<HashSet<String>>> {
        Arc::clone(&self.in_progress)
    }
}

impl Default for ImprovementPool {
    fn default() -> Self {
        Self::new(2)
    }
}

// ---------------------------------------------------------------------------
// ImprovementOutcome
// ---------------------------------------------------------------------------

/// Result of a background improvement thread.
#[derive(Debug, Clone)]
pub struct ImprovementOutcome {
    pub component: String,
    pub success: bool,
    pub improvement_ratio: f64,
    pub new_program: Option<SemanticGraph>,
    pub slowdown: f64,
    pub event: ImprovementEvent,
}

// ---------------------------------------------------------------------------
// SelfImprovingConfig
// ---------------------------------------------------------------------------

/// Configuration for the self-improving daemon.
#[derive(Debug, Clone)]
pub struct SelfImprovingConfig {
    /// Target cycle time in milliseconds (default: 800ms).
    pub cycle_time_ms: u64,
    /// Maximum number of cycles. `None` = run until stopped.
    pub max_cycles: Option<u64>,
    /// Run auto-improvement every N cycles (used as periodic trigger).
    pub improve_interval: u64,
    /// Run self-inspection every N cycles.
    pub inspect_interval: u64,
    /// Configuration for the auto-improver.
    pub auto_improve: AutoImproveConfig,
    /// Directory for persisting state. `None` = no persistence.
    pub state_dir: Option<PathBuf>,
    /// Memory limit for the self-inspector (bytes). 0 = no limit.
    pub memory_limit: usize,
    /// Deterministic RNG seed for the auto-improver. `None` = random.
    pub seed: Option<u64>,
    /// Maximum concurrent improvement threads.
    pub max_improve_threads: usize,
    /// Maximum stagnant attempts before giving up on a component.
    pub max_stagnant: u32,
    /// Minimum improvement ratio to count as progress (default 0.05 = 5%).
    pub min_improvement: f64,
    /// Execution mode.
    pub exec_mode: ExecMode,
    /// Check triggers every N executions.
    pub trigger_check_interval: u64,
}

impl Default for SelfImprovingConfig {
    fn default() -> Self {
        Self {
            cycle_time_ms: 800,
            max_cycles: None,
            improve_interval: 10,
            inspect_interval: 5,
            auto_improve: AutoImproveConfig::default(),
            state_dir: None,
            memory_limit: 50 * 1024 * 1024, // 50 MB
            seed: None,
            max_improve_threads: 2,
            max_stagnant: 5,
            min_improvement: 0.05,
            exec_mode: ExecMode::default(),
            trigger_check_interval: 100,
        }
    }
}

// ---------------------------------------------------------------------------
// SelfImprovingReport
// ---------------------------------------------------------------------------

/// Report from a single self-improving cycle.
#[derive(Debug, Clone)]
pub struct SelfImprovingReport {
    /// The daemon cycle number.
    pub cycle: u64,
    /// Wall-clock time for this cycle in milliseconds.
    pub wall_time_ms: u64,
    /// If an improvement cycle ran this cycle, its event.
    pub improvement_event: Option<ImprovementEvent>,
    /// If an inspection ran this cycle, its findings.
    pub inspection_findings: Vec<String>,
    /// Whether a checkpoint was saved this cycle.
    pub checkpoint_saved: bool,
    /// Whether any improvement trigger fired this cycle.
    pub trigger_fired: Option<String>,
    /// Whether the system is fully converged.
    pub fully_converged: bool,
    /// Whether improvement is currently compounding (PT3 metric).
    pub is_compounding: bool,
}

// ---------------------------------------------------------------------------
// DaemonState (serializable for checkpoint)
// ---------------------------------------------------------------------------

/// Serializable snapshot of the daemon's deployed components and metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DaemonState {
    /// Deployed components.
    pub deployed: Vec<DeployedComponentRecord>,
    /// Total improvement cycles completed.
    pub improvement_cycles: u64,
    /// Total daemon cycles completed.
    pub daemon_cycles: u64,
}

/// A single deployed component record for serialization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeployedComponentRecord {
    pub name: String,
    pub program_json: String,
    pub slowdown: f64,
    pub deployed_at: u64,
}

// ---------------------------------------------------------------------------
// SelfImprovingResult
// ---------------------------------------------------------------------------

/// Final result after the self-improving daemon stops.
#[derive(Debug, Clone)]
pub struct SelfImprovingResult {
    /// Total daemon cycles completed.
    pub cycles_completed: u64,
    /// Total wall-clock time.
    pub total_time: Duration,
    /// Number of auto-improvement cycles run.
    pub improvement_cycles: u64,
    /// Number of components currently deployed.
    pub components_deployed: usize,
    /// Number of audit trail entries.
    pub audit_entries: usize,
    /// Recursive improvement depth: how many generations of self-improvement
    /// have been deployed (each deploy increments this).
    pub recursive_depth: u64,
    /// Number of components that have converged (stopped improving).
    pub converged_components: usize,
    /// Whether system-wide convergence was detected.
    pub fully_converged: bool,
    // --- Phase Transition 3: Recursive improvement metrics ---
    /// Improvement rate: slope of latency reduction per cycle (negative = improving).
    pub improvement_rate: f64,
    /// Acceleration: rate of change of improvement rate (negative = compounding).
    pub acceleration: f64,
    /// Whether improvement is compounding (acceleration < 0).
    pub is_compounding: bool,
    /// Total number of problems solved during the daemon's lifetime.
    pub problems_solved: usize,
    /// Top operator contributions: (name, stats) sorted by total fitness delta.
    pub operator_contributions: Vec<(String, crate::improvement_tracker::OperatorStats)>,
}

// ---------------------------------------------------------------------------
// ThreadedDaemon (replaces SelfImprovingDaemon)
// ---------------------------------------------------------------------------

/// The fully wired self-improving daemon with threading, criteria-based
/// improvement triggers, and convergence detection.
///
/// Runs a cycle loop that monitors component metrics, fires improvement
/// triggers when criteria are met, spawns background improvement threads,
/// detects stagnation per-component, and stops when the system reaches
/// a local maximum.
pub struct ThreadedDaemon {
    config: SelfImprovingConfig,
    runtime: Arc<RwLock<IrisRuntime>>,
    auto_improver: Arc<Mutex<AutoImprover>>,
    inspector: SelfInspector,
    exec: Arc<IrisExecutionService>,
    cycle_count: u64,
    running: Arc<AtomicBool>,
    reports: Vec<SelfImprovingReport>,
    recursive_depth: u64,

    // Threading
    improve_pool: ImprovementPool,

    // Metrics
    metrics: Arc<ComponentMetrics>,

    // Triggers
    improve_triggers: Vec<ImproveTrigger>,

    // Convergence
    stagnation: StagnationDetector,
    convergence: ConvergenceDetector,
    fully_converged: bool,

    // Pending results from background threads
    pending_outcomes: Arc<Mutex<Vec<ImprovementOutcome>>>,

    // Component names for convergence checking
    component_names: Vec<String>,

    // Phase Transition 3: recursive improvement tracking
    improvement_tracker: ImprovementTracker,

    // Deployment rate limiting
    /// Instant of the last successful component deployment.
    last_deploy_at: Option<Instant>,
    /// Number of deployments in the current hour window.
    deploys_this_hour: u32,
    /// Wall-clock start of the current hour window.
    hour_window_start: Instant,
}

/// Type alias for backward compatibility.
pub type SelfImprovingDaemon = ThreadedDaemon;

/// Minimum wall-clock interval between consecutive component deployments.
const MIN_DEPLOY_COOLDOWN: Duration = Duration::from_secs(60);

/// Maximum deployments allowed per hour window.
const MAX_DEPLOYS_PER_HOUR: u32 = 10;

/// Effect tag bytes that are unconditionally forbidden in deployed programs.
///
/// These allow arbitrary code execution or hardware access that cannot be
/// safely sandboxed at the graph level:
/// - 0x29: MmapExec — maps bytes as executable memory
/// - 0x2A: CallNative — calls a raw function pointer
/// - 0x2B: FfiCall — calls a foreign function via dlopen/dlsym
const FORBIDDEN_EFFECT_TAGS: &[u8] = &[0x29, 0x2A, 0x2B];

/// Validate a program for safe deployment.
///
/// Returns `Ok(())` if the program passes all safety checks, or an `Err`
/// with a description of the violation.
///
/// Current checks:
/// - No `Effect` nodes with forbidden tags (MmapExec, CallNative, FfiCall).
fn validate_program_for_deployment(program: &SemanticGraph) -> Result<(), String> {
    for node in program.nodes.values() {
        if let NodePayload::Effect { effect_tag } = &node.payload {
            if FORBIDDEN_EFFECT_TAGS.contains(effect_tag) {
                return Err(format!(
                    "program contains forbidden effect node (tag 0x{:02x}) \
                     which would allow arbitrary code execution",
                    effect_tag
                ));
            }
        }
    }
    Ok(())
}

impl ThreadedDaemon {
    /// Create a new threaded self-improving daemon.
    pub fn new(config: SelfImprovingConfig) -> Self {
        let exec = Arc::new(IrisExecutionService::with_defaults());
        let runtime = Arc::new(RwLock::new(IrisRuntime::new()));

        let auto_improver = match config.seed {
            Some(seed) => AutoImprover::with_seed(config.auto_improve.clone(), seed),
            None => AutoImprover::with_config(config.auto_improve.clone()),
        };

        let inspector = SelfInspector::new(
            Duration::from_secs(config.auto_improve.cycle_interval_secs),
            config.memory_limit,
        );

        let stagnation = StagnationDetector::new(config.max_stagnant, config.min_improvement);
        let convergence = ConvergenceDetector::new();
        let improve_pool = ImprovementPool::new(config.max_improve_threads);
        let metrics = Arc::new(ComponentMetrics::new());

        // Register default components.
        let component_names = vec![
            "mutation_insert_node".to_string(),
            "mutation_delete_node".to_string(),
            "mutation_rewire_edge".to_string(),
            "seed_arithmetic".to_string(),
            "seed_fold".to_string(),
        ];

        let mut auto_improver = auto_improver;
        let baselines = [
            ("mutation_insert_node", 50u64),
            ("mutation_delete_node", 30),
            ("mutation_rewire_edge", 45),
            ("seed_arithmetic", 20),
            ("seed_fold", 25),
        ];

        for (name, micros) in &baselines {
            auto_improver.register_component(name, Duration::from_micros(*micros));
        }

        // Build default triggers: periodic + latency thresholds for each component.
        let mut improve_triggers = Vec::new();

        // Periodic trigger as fallback.
        improve_triggers.push(ImproveTrigger::Periodic {
            interval: config.improve_interval,
        });

        // Latency triggers: trigger if p99 > 2x baseline.
        for (name, micros) in &baselines {
            improve_triggers.push(ImproveTrigger::LatencyThreshold {
                component: name.to_string(),
                max_p99: Duration::from_micros(micros * 2),
            });
        }

        // Register telemetry for the inspector.
        let auto_improver = Arc::new(Mutex::new(auto_improver));
        let now = Instant::now();
        let mut daemon = Self {
            config,
            runtime,
            auto_improver,
            inspector,
            exec,
            cycle_count: 0,
            running: Arc::new(AtomicBool::new(true)),
            reports: Vec::new(),
            recursive_depth: 0,
            improve_pool,
            metrics,
            improve_triggers,
            stagnation,
            convergence,
            fully_converged: false,
            pending_outcomes: Arc::new(Mutex::new(Vec::new())),
            component_names,
            improvement_tracker: ImprovementTracker::new(),
            last_deploy_at: None,
            deploys_this_hour: 0,
            hour_window_start: now,
        };

        // Register the same components for telemetry monitoring.
        for (name, micros) in &baselines {
            daemon
                .inspector
                .telemetry
                .register_component(name, Duration::from_micros(*micros));
        }

        daemon
    }

    /// Check whether a deployment is permitted right now under the rate limits.
    ///
    /// Returns `false` if:
    /// - Less than `MIN_DEPLOY_COOLDOWN` (60s) has elapsed since the last deployment, or
    /// - `MAX_DEPLOYS_PER_HOUR` deployments have already occurred in the current
    ///   one-hour window.
    fn can_deploy_now(&mut self) -> bool {
        let now = Instant::now();

        // Rotate the hour window if needed.
        if now.duration_since(self.hour_window_start) >= Duration::from_secs(3600) {
            self.hour_window_start = now;
            self.deploys_this_hour = 0;
        }

        // Cooldown check.
        if let Some(last) = self.last_deploy_at {
            if now.duration_since(last) < MIN_DEPLOY_COOLDOWN {
                return false;
            }
        }

        // Per-hour cap.
        if self.deploys_this_hour >= MAX_DEPLOYS_PER_HOUR {
            return false;
        }

        true
    }

    /// Record a successful deployment for rate-limiting bookkeeping.
    fn record_deploy(&mut self) {
        self.last_deploy_at = Some(Instant::now());
        self.deploys_this_hour += 1;
    }

    /// Get a clone of the running flag for external shutdown control.
    pub fn running_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.running)
    }

    /// Signal the daemon to stop.
    ///
    /// Uses `Release` ordering so that all prior stores are visible to the
    /// thread reading the flag with `Acquire`.
    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }

    /// Current cycle count.
    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }

    /// Access the auto-improver (locks the mutex).
    pub fn auto_improver(&self) -> &Arc<Mutex<AutoImprover>> {
        &self.auto_improver
    }

    /// Access the self-inspector.
    pub fn inspector(&self) -> &SelfInspector {
        &self.inspector
    }

    /// Access the runtime.
    pub fn runtime(&self) -> &Arc<RwLock<IrisRuntime>> {
        &self.runtime
    }

    /// Access collected reports.
    pub fn reports(&self) -> &[SelfImprovingReport] {
        &self.reports
    }

    /// Current recursive improvement depth.
    pub fn recursive_depth(&self) -> u64 {
        self.recursive_depth
    }

    /// Access the component metrics.
    pub fn metrics(&self) -> &Arc<ComponentMetrics> {
        &self.metrics
    }

    /// Access the stagnation detector.
    pub fn stagnation(&self) -> &StagnationDetector {
        &self.stagnation
    }

    /// Access the convergence detector.
    pub fn convergence(&self) -> &ConvergenceDetector {
        &self.convergence
    }

    /// Whether the system has fully converged.
    pub fn is_fully_converged(&self) -> bool {
        self.fully_converged
    }

    /// Access the improvement pool.
    pub fn improvement_pool(&self) -> &ImprovementPool {
        &self.improve_pool
    }

    /// Access the improvement tracker (Phase Transition 3).
    pub fn improvement_tracker(&self) -> &ImprovementTracker {
        &self.improvement_tracker
    }

    /// Mutable access to the improvement tracker.
    pub fn improvement_tracker_mut(&mut self) -> &mut ImprovementTracker {
        &mut self.improvement_tracker
    }

    /// Load previously deployed components from a state directory.
    ///
    /// Returns the number of components loaded.
    pub fn load_state(&mut self, state_dir: &Path) -> usize {
        let state_path = state_dir.join("daemon_state.json");
        let data = match std::fs::read_to_string(&state_path) {
            Ok(d) => d,
            Err(_) => return 0,
        };
        let state: DaemonState = match serde_json::from_str(&data) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let mut loaded = 0;
        let mut auto_improver = self.auto_improver.lock().unwrap();
        let mut runtime = self.runtime.write().unwrap();
        for record in &state.deployed {
            match serde_json::from_str::<SemanticGraph>(&record.program_json) {
                Ok(program) => {
                    runtime.replace_component(&record.name, program.clone());
                    auto_improver.deploy(&record.name, program, record.slowdown);
                    loaded += 1;
                }
                Err(_) => {
                    // Skip components that fail to deserialize.
                }
            }
        }
        self.cycle_count = state.daemon_cycles;
        loaded
    }

    /// Save current state to disk.
    pub fn save_state(&self, state_dir: &Path) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(state_dir)?;

        // Restrict state directory to owner-only access (0700) to prevent
        // other users on the system from reading or modifying daemon state.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o700);
            let _ = std::fs::set_permissions(state_dir, perms);
        }

        let auto_improver = self.auto_improver.lock().unwrap();
        let mut deployed = Vec::new();
        for (name, component) in auto_improver.deployed() {
            let program_json =
                serde_json::to_string(&component.iris_program).unwrap_or_default();
            deployed.push(DeployedComponentRecord {
                name: name.clone(),
                program_json,
                slowdown: component.slowdown,
                deployed_at: component.deployed_at,
            });
        }

        let state = DaemonState {
            deployed,
            improvement_cycles: auto_improver.cycle_count(),
            daemon_cycles: self.cycle_count,
        };

        let json = serde_json::to_string_pretty(&state)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Atomic write: write to .tmp, then rename.
        let state_path = state_dir.join("daemon_state.json");
        let tmp_path = state_dir.join("daemon_state.json.tmp");
        std::fs::write(&tmp_path, json.as_bytes())?;
        std::fs::rename(&tmp_path, &state_path)?;

        // Save the audit trail atomically (write-tmp-rename).
        let audit_path = state_dir.join("audit_trail.json");
        let audit_tmp_path = state_dir.join("audit_trail.json.tmp");
        {
            let audit_json = serde_json::to_vec_pretty(&self.inspector.audit)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            std::fs::write(&audit_tmp_path, &audit_json)?;
            std::fs::rename(&audit_tmp_path, &audit_path)?;
        }

        Ok(())
    }

    /// Check all improvement triggers and return the first one that fires
    /// for a component that isn't already being improved and hasn't converged.
    fn check_triggers(&self) -> Option<String> {
        for trigger in &self.improve_triggers {
            if let Some(component) = trigger.check(&self.metrics, self.cycle_count) {
                // Skip components that have converged.
                if component != "__periodic__" && self.stagnation.is_converged(&component) {
                    continue;
                }
                // Skip components currently being improved.
                if component != "__periodic__" && self.improve_pool.is_in_progress(&component) {
                    continue;
                }
                return Some(component);
            }
        }
        None
    }

    /// Spawn a background improvement thread for a component.
    ///
    /// The thread takes a snapshot of the current state, runs evolution,
    /// gates the result, and pushes the outcome to the pending queue.
    fn spawn_improvement(&self, component: String) {
        if !self.improve_pool.try_start(&component) {
            return;
        }

        let auto_improver = Arc::clone(&self.auto_improver);
        let exec = Arc::clone(&self.exec);
        let pending = Arc::clone(&self.pending_outcomes);
        let active = self.improve_pool.active_counter();
        let in_progress = self.improve_pool.in_progress_set();
        let component_clone = component.clone();

        std::thread::spawn(move || {
            // Run the improvement cycle synchronously in this thread.
            let event = {
                let mut improver = auto_improver.lock().unwrap();
                improver.run_cycle(&*exec)
            };

            let (success, improvement_ratio, new_program, slowdown) = match &event.action {
                ImprovementAction::Deployed { slowdown } => {
                    let program = {
                        let improver = auto_improver.lock().unwrap();
                        improver
                            .deployed()
                            .get(&event.component)
                            .map(|d| d.iris_program.clone())
                    };
                    (true, 1.0 / slowdown.max(0.001), program, *slowdown)
                }
                ImprovementAction::Failed { .. } => (false, 0.0, None, 0.0),
                ImprovementAction::Profiled { .. } => (false, 0.0, None, 0.0),
                ImprovementAction::Evolved { candidate_slowdown } => {
                    (false, 0.0, None, *candidate_slowdown)
                }
                ImprovementAction::Explored { solve_rate, .. } => {
                    (false, *solve_rate as f64, None, 0.0)
                }
            };

            let outcome = ImprovementOutcome {
                component: component_clone.clone(),
                success,
                improvement_ratio,
                new_program,
                slowdown,
                event,
            };

            // Push outcome to pending queue.
            {
                let mut outcomes = pending.lock().unwrap();
                outcomes.push(outcome);
            }

            // Release the slot.
            {
                let mut set = in_progress.lock().unwrap();
                set.remove(&component_clone);
            }
            active.fetch_sub(1, Ordering::Relaxed);
        });
    }

    /// Process pending improvement outcomes from background threads.
    ///
    /// Atomically swaps improved programs into the runtime, records in
    /// stagnation detector and audit trail.
    fn process_pending_outcomes(&mut self) -> Vec<ImprovementEvent> {
        let outcomes: Vec<ImprovementOutcome> = {
            let mut pending = self.pending_outcomes.lock().unwrap();
            std::mem::take(&mut *pending)
        };

        let mut events = Vec::new();

        for outcome in outcomes {
            // Record in stagnation detector.
            let just_converged = self.stagnation.record_attempt(
                &outcome.component,
                outcome.improvement_ratio,
            );

            if just_converged {
                eprintln!(
                    "[ThreadedDaemon] Component '{}' has converged after {} stagnant attempts",
                    outcome.component, self.config.max_stagnant
                );
            }

            // Feed improvement data into the tracker (Phase Transition 3).
            {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);

                // Record latency measurement from the improvement outcome.
                if let Some(mean_latency) = self.metrics.mean_latency(&outcome.component) {
                    self.improvement_tracker.record_measurement(
                        &outcome.component,
                        now_ms,
                        mean_latency.as_nanos() as u64,
                        self.metrics
                            .correctness_rate(&outcome.component)
                            .unwrap_or(1.0),
                    );
                }

                // If the outcome indicates a solved problem, record it.
                if outcome.success {
                    self.improvement_tracker.record_problem_solved(now_ms);
                }
            }

            if outcome.success {
                if let Some(ref program) = outcome.new_program {
                    // Validate the program against the safe-effects allowlist.
                    if let Err(reason) = validate_program_for_deployment(program) {
                        eprintln!(
                            "[ThreadedDaemon] rejected deployment of '{}': {}",
                            outcome.component, reason
                        );
                    } else if self.can_deploy_now() {
                        // Rate-limit deployments: enforce cooldown + hourly cap.
                        // Atomically swap into runtime.
                        {
                            let mut runtime = self.runtime.write().unwrap();
                            runtime.replace_component(&outcome.component, program.clone());
                        }

                        self.record_deploy();
                        self.recursive_depth += 1;
                        self.metrics.total_improvements.fetch_add(1, Ordering::Relaxed);

                        // Record in audit trail.
                        self.inspector.audit.record(AuditAction::ComponentDeployed {
                            name: outcome.component.clone(),
                            slowdown: outcome.slowdown,
                        });
                    } else {
                        eprintln!(
                            "[ThreadedDaemon] deployment of '{}' rate-limited (cooldown or hourly cap)",
                            outcome.component
                        );
                    }
                }
            } else {
                // Failed attempt — still recorded in stagnation detector above.
            }

            events.push(outcome.event);
        }

        events
    }

    /// Run the daemon loop.
    ///
    /// Each cycle:
    /// 1. Process pending improvement outcomes from background threads.
    /// 2. Check improvement triggers and spawn background threads.
    /// 3. Every `inspect_interval` cycles: run self-inspection.
    /// 4. If state_dir is configured: save checkpoint periodically.
    /// 5. Check for system-wide convergence.
    /// 6. Sleep for the remaining cycle budget (if FixedInterval mode).
    pub fn run(&mut self) -> SelfImprovingResult {
        let run_start = Instant::now();
        let target_duration = match &self.config.exec_mode {
            ExecMode::FixedInterval(d) => *d,
            ExecMode::Continuous => Duration::ZERO,
        };
        // Use cycle_time_ms as the target if exec_mode wasn't explicitly set
        // and cycle_time_ms is non-zero (backward compat).
        let target_duration = if target_duration == Duration::ZERO
            && self.config.cycle_time_ms > 0
            && !matches!(self.config.exec_mode, ExecMode::Continuous)
        {
            Duration::from_millis(self.config.cycle_time_ms)
        } else {
            target_duration
        };

        // Try loading state on startup.
        if let Some(ref state_dir) = self.config.state_dir.clone() {
            let loaded = self.load_state(state_dir);
            if loaded > 0 {
                eprintln!(
                    "[ThreadedDaemon] Loaded {} components from checkpoint",
                    loaded
                );
            }
        }

        while self.running.load(Ordering::Acquire) {
            let cycle_start = Instant::now();
            self.cycle_count += 1;
            self.metrics.total_executions.fetch_add(1, Ordering::Relaxed);

            let mut report = SelfImprovingReport {
                cycle: self.cycle_count,
                wall_time_ms: 0,
                improvement_event: None,
                inspection_findings: Vec::new(),
                checkpoint_saved: false,
                trigger_fired: None,
                fully_converged: self.fully_converged,
                is_compounding: self.improvement_tracker.is_compounding(),
            };

            // 1. Process pending outcomes from background improvement threads.
            let completed_events = self.process_pending_outcomes();
            if let Some(event) = completed_events.into_iter().last() {
                report.improvement_event = Some(event);
            }

            // 2. Check triggers and spawn improvement threads (if not converged).
            if !self.fully_converged {
                if let Some(component) = self.check_triggers() {
                    report.trigger_fired = Some(component.clone());

                    if component == "__periodic__" {
                        // Periodic: run synchronously like the old daemon.
                        let event = {
                            let mut auto_improver = self.auto_improver.lock().unwrap();
                            auto_improver.run_cycle(&*self.exec)
                        };

                        // If deployed, hot-swap (subject to safety checks and rate limits).
                        if let ImprovementAction::Deployed { slowdown } = &event.action {
                            let component_name = &event.component;
                            let program = {
                                let auto_improver = self.auto_improver.lock().unwrap();
                                auto_improver
                                    .deployed()
                                    .get(component_name)
                                    .map(|d| d.iris_program.clone())
                            };
                            if let Some(ref program) = program {
                                // Validate safe-effects allowlist.
                                if let Err(reason) = validate_program_for_deployment(program) {
                                    eprintln!(
                                        "[ThreadedDaemon] rejected periodic deployment of '{}': {}",
                                        component_name, reason
                                    );
                                } else if self.can_deploy_now() {
                                    {
                                        let mut runtime = self.runtime.write().unwrap();
                                        runtime.replace_component(component_name, program.clone());
                                    }
                                    self.record_deploy();
                                    self.recursive_depth += 1;
                                    self.metrics
                                        .total_improvements
                                        .fetch_add(1, Ordering::Relaxed);
                                    self.inspector.audit.record(AuditAction::ComponentDeployed {
                                        name: component_name.clone(),
                                        slowdown: *slowdown,
                                    });
                                } else {
                                    eprintln!(
                                        "[ThreadedDaemon] periodic deployment of '{}' rate-limited",
                                        component_name
                                    );
                                }
                            }
                        }

                        // Record stagnation.
                        let ratio = match &event.action {
                            ImprovementAction::Deployed { slowdown } => {
                                1.0 / slowdown.max(0.001)
                            }
                            _ => 0.0,
                        };
                        self.stagnation.record_attempt(&event.component, ratio);

                        report.improvement_event = Some(event);
                    } else if !self.stagnation.is_converged(&component) {
                        // Criteria-based: spawn background thread.
                        self.spawn_improvement(component);
                    }
                }
            }

            // 3. Run self-inspection if interval matches.
            if self.cycle_count % self.config.inspect_interval == 0 {
                let findings = self.inspector.inspect();
                for finding in &findings {
                    let desc = match finding {
                        InspectionResult::Regression {
                            component,
                            expected,
                            actual,
                        } => {
                            format!(
                                "regression in '{}': expected {:?}, actual {:?}",
                                component, expected, actual
                            )
                        }
                        InspectionResult::CorrectnessFailure {
                            component,
                            test_case_idx,
                        } => {
                            format!(
                                "correctness failure in '{}' at test case {}",
                                component, test_case_idx
                            )
                        }
                        InspectionResult::MemoryExceeded { current, limit } => {
                            format!("memory exceeded: {} / {} bytes", current, limit)
                        }
                    };
                    report.inspection_findings.push(desc);

                    // Auto-correct critical findings.
                    if finding.is_critical() {
                        let anomaly = finding.to_anomaly();
                        let correction = self.inspector.correct(&anomaly);
                        if let CorrectionResult::Reverted { ref component }
                        | CorrectionResult::RevertedWithAlert { ref component } = correction
                        {
                            let mut runtime = self.runtime.write().unwrap();
                            runtime.remove_component(component);
                            self.metrics.total_rollbacks.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }

            // 4. Save checkpoint if configured (every improve_interval cycles).
            if self.config.state_dir.is_some()
                && self.cycle_count % self.config.improve_interval == 0
            {
                if let Some(ref state_dir) = self.config.state_dir.clone() {
                    if self.save_state(state_dir).is_ok() {
                        report.checkpoint_saved = true;
                    }
                }
            }

            // 5. Check system-wide convergence.
            if !self.fully_converged {
                let converged = self.convergence.is_fully_converged(
                    &self.component_names,
                    &self.stagnation,
                );
                if converged {
                    self.fully_converged = true;
                    eprintln!(
                        "[ThreadedDaemon] System has reached local maximum — \
                         all {} components converged. Stopping improvement.",
                        self.component_names.len()
                    );
                }
            }
            report.fully_converged = self.fully_converged;

            // Record timing.
            let elapsed = cycle_start.elapsed();
            report.wall_time_ms = elapsed.as_millis() as u64;
            self.reports.push(report);

            // Sleep for remaining cycle budget (if FixedInterval mode).
            if target_duration > Duration::ZERO && elapsed < target_duration {
                let remaining = target_duration - elapsed;
                if remaining > Duration::ZERO {
                    std::thread::sleep(remaining);
                }
            }

            // Check termination.
            if let Some(max) = self.config.max_cycles {
                if self.cycle_count >= max {
                    break;
                }
            }
        }

        // Wait for any active improvement threads to finish (with timeout).
        let wait_start = Instant::now();
        while self.improve_pool.active_count() > 0 && wait_start.elapsed() < Duration::from_secs(5)
        {
            std::thread::sleep(Duration::from_millis(10));
        }

        // Process any final outcomes.
        self.process_pending_outcomes();

        // Save final snapshot on shutdown.
        if let Some(ref state_dir) = self.config.state_dir.clone() {
            let _ = self.save_state(state_dir);
        }

        // Compute aggregate improvement rate and acceleration from the tracker.
        let mut avg_rate = 0.0_f64;
        let mut avg_accel = 0.0_f64;
        let mut component_count = 0usize;
        for name in self.improvement_tracker.component_names() {
            if let Some(rate) = self.improvement_tracker.improvement_rate(&name) {
                avg_rate += rate;
                component_count += 1;
            }
            if let Some(accel) = self.improvement_tracker.acceleration(&name) {
                avg_accel += accel;
            }
        }
        if component_count > 0 {
            avg_rate /= component_count as f64;
            avg_accel /= component_count as f64;
        }

        // Log final improvement tracker summary.
        eprintln!(
            "[ThreadedDaemon] Improvement rate: {:.6}/cycle, Acceleration: {:.6}/cycle^2 {}",
            avg_rate,
            avg_accel,
            if self.improvement_tracker.is_compounding() {
                "(COMPOUNDING)"
            } else if avg_accel > 1e-9 {
                "(DECELERATING — approaching local maximum)"
            } else {
                "(STEADY)"
            },
        );

        let auto_improver = self.auto_improver.lock().unwrap();
        SelfImprovingResult {
            cycles_completed: self.cycle_count,
            total_time: run_start.elapsed(),
            improvement_cycles: auto_improver.cycle_count(),
            components_deployed: auto_improver.deployed().len(),
            audit_entries: self.inspector.audit.len(),
            recursive_depth: self.recursive_depth,
            converged_components: self.stagnation.converged_count(),
            fully_converged: self.fully_converged,
            improvement_rate: avg_rate,
            acceleration: avg_accel,
            is_compounding: self.improvement_tracker.is_compounding(),
            problems_solved: self.improvement_tracker.total_problems_solved(),
            operator_contributions: self.improvement_tracker.operator_contributions_sorted(),
        }
    }
}
