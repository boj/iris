//! Observation-driven trace collection for self-improvement.
//!
//! The trace system records function call samples (inputs, output, latency)
//! from a running program. These traces become implicit test cases that the
//! evolution engine uses to synthesize faster replacements.

use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::collections::BTreeMap;

use crate::eval::{Value, TestCase};
use crate::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// TraceEntry — a single observed function call
// ---------------------------------------------------------------------------

/// One sampled function call: what went in, what came out, how long it took.
#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub inputs: Vec<Value>,
    pub output: Value,
    pub latency_ns: u64,
}

// ---------------------------------------------------------------------------
// TraceBuffer — per-function ring buffer of observations
// ---------------------------------------------------------------------------

/// Fixed-size ring buffer of trace entries for a single function.
#[derive(Debug)]
pub struct TraceBuffer {
    entries: Vec<Option<TraceEntry>>,
    write_idx: usize,
    count: u64,
    capacity: usize,
}

impl TraceBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: (0..capacity).map(|_| None).collect(),
            write_idx: 0,
            count: 0,
            capacity,
        }
    }

    pub fn push(&mut self, entry: TraceEntry) {
        self.entries[self.write_idx] = Some(entry);
        self.write_idx = (self.write_idx + 1) % self.capacity;
        self.count += 1;
    }

    pub fn len(&self) -> u64 {
        self.count
    }

    /// Snapshot all present entries (oldest first).
    pub fn snapshot(&self) -> Vec<TraceEntry> {
        let filled = std::cmp::min(self.count as usize, self.capacity);
        let mut out = Vec::with_capacity(filled);
        // Start from the oldest entry in the ring.
        let start = if self.count as usize > self.capacity {
            self.write_idx
        } else {
            0
        };
        for i in 0..filled {
            let idx = (start + i) % self.capacity;
            if let Some(e) = &self.entries[idx] {
                out.push(e.clone());
            }
        }
        out
    }

    /// Average latency in nanoseconds across buffered entries.
    pub fn avg_latency_ns(&self) -> u64 {
        let entries = self.snapshot();
        if entries.is_empty() {
            return 0;
        }
        let sum: u64 = entries.iter().map(|e| e.latency_ns).sum();
        sum / entries.len() as u64
    }
}

// ---------------------------------------------------------------------------
// FunctionId — identifies a traceable function
// ---------------------------------------------------------------------------

/// Identifies a function in the running program. Uses the fragment name
/// (from the source `let` binding) since that's what users see.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionId(pub String);

// ---------------------------------------------------------------------------
// TraceCollector — the central collection point
// ---------------------------------------------------------------------------

/// Collects function-call traces across the running program.
/// Thread-safe: shared between the evaluator and the improvement daemon.
pub struct TraceCollector {
    /// Per-function trace buffers.
    buffers: Mutex<BTreeMap<FunctionId, TraceBuffer>>,
    /// Sampling: only trace 1-in-N calls (N = 1/sample_rate).
    sample_counter: AtomicU64,
    sample_interval: u64,
    /// Ring buffer capacity per function.
    buffer_capacity: usize,
    /// Master enable flag.
    enabled: AtomicBool,
}

impl TraceCollector {
    pub fn new(sample_rate: f64, buffer_capacity: usize) -> Self {
        let interval = if sample_rate <= 0.0 {
            u64::MAX
        } else {
            (1.0 / sample_rate) as u64
        };
        Self {
            buffers: Mutex::new(BTreeMap::new()),
            sample_counter: AtomicU64::new(0),
            sample_interval: interval,
            buffer_capacity,
            enabled: AtomicBool::new(true),
        }
    }

    /// Default configuration: 1% sampling, 200 entries per function.
    pub fn default_config() -> Self {
        Self::new(0.01, 200)
    }

    /// Should this call be sampled? Increments the counter atomically.
    pub fn should_sample(&self) -> bool {
        if !self.enabled.load(Ordering::Relaxed) {
            return false;
        }
        let count = self.sample_counter.fetch_add(1, Ordering::Relaxed);
        count % self.sample_interval == 0
    }

    /// Record a traced function call.
    pub fn record(&self, fn_id: FunctionId, entry: TraceEntry) {
        if let Ok(mut buffers) = self.buffers.lock() {
            buffers
                .entry(fn_id)
                .or_insert_with(|| TraceBuffer::new(self.buffer_capacity))
                .push(entry);
        }
    }

    /// Get all function IDs that have accumulated enough traces.
    pub fn ready_functions(&self, min_traces: u64) -> Vec<FunctionId> {
        if let Ok(buffers) = self.buffers.lock() {
            buffers
                .iter()
                .filter(|(_, buf)| buf.len() >= min_traces)
                .map(|(id, _)| id.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Build test cases from observed traces for a function.
    /// Deduplicates by input and selects up to `max_cases` diverse entries.
    pub fn build_test_cases(&self, fn_id: &FunctionId, max_cases: usize) -> Vec<TestCase> {
        let snapshot = if let Ok(buffers) = self.buffers.lock() {
            buffers.get(fn_id).map(|b| b.snapshot()).unwrap_or_default()
        } else {
            return Vec::new();
        };

        // Deduplicate by input (keep the first occurrence of each unique input set).
        let mut seen: Vec<Vec<Value>> = Vec::new();
        let mut unique: Vec<&TraceEntry> = Vec::new();
        for entry in &snapshot {
            if !seen.contains(&entry.inputs) {
                seen.push(entry.inputs.clone());
                unique.push(entry);
            }
        }

        // Take up to max_cases, spreading across the range.
        let step = if unique.len() <= max_cases {
            1
        } else {
            unique.len() / max_cases
        };

        unique
            .iter()
            .step_by(step)
            .take(max_cases)
            .map(|e| TestCase {
                inputs: e.inputs.clone(),
                expected_output: Some(vec![e.output.clone()]),
                initial_state: None,
                expected_state: None,
            })
            .collect()
    }

    /// Get average latency for a function in nanoseconds.
    pub fn avg_latency_ns(&self, fn_id: &FunctionId) -> u64 {
        if let Ok(buffers) = self.buffers.lock() {
            buffers.get(fn_id).map(|b| b.avg_latency_ns()).unwrap_or(0)
        } else {
            0
        }
    }

    /// Snapshot trace count per function (for monitoring/logging).
    pub fn stats(&self) -> BTreeMap<FunctionId, u64> {
        if let Ok(buffers) = self.buffers.lock() {
            buffers.iter().map(|(id, buf)| (id.clone(), buf.len())).collect()
        } else {
            BTreeMap::new()
        }
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }
}

impl std::fmt::Debug for TraceCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TraceCollector")
            .field("sample_interval", &self.sample_interval)
            .field("buffer_capacity", &self.buffer_capacity)
            .field("enabled", &self.enabled.load(Ordering::Relaxed))
            .finish()
    }
}

// ---------------------------------------------------------------------------
// ImprovementConfig — settings for the improvement daemon
// ---------------------------------------------------------------------------

/// Configuration for the observation-driven improvement daemon.
#[derive(Debug, Clone)]
pub struct ImprovementConfig {
    /// Minimum observed calls before attempting improvement.
    pub min_traces: u64,
    /// Max slowdown factor for the performance gate (e.g. 2.0 = can be 2x slower).
    pub max_slowdown: f64,
    /// Max wall-clock seconds per evolution attempt.
    pub evolution_budget_secs: u64,
    /// Population size for evolution.
    pub population_size: usize,
    /// Max generations per evolution attempt.
    pub max_generations: usize,
    /// Max test cases to derive from traces.
    pub max_test_cases: usize,
    /// How often (in seconds) to scan for improvable functions.
    pub scan_interval_secs: u64,
}

impl Default for ImprovementConfig {
    fn default() -> Self {
        Self {
            min_traces: 50,
            max_slowdown: 2.0,
            evolution_budget_secs: 5,
            population_size: 32,
            max_generations: 200,
            max_test_cases: 20,
            scan_interval_secs: 5,
        }
    }
}

// ---------------------------------------------------------------------------
// ImprovementResult — what the daemon reports
// ---------------------------------------------------------------------------

/// Outcome of one improvement attempt.
#[derive(Debug, Clone)]
pub struct ImprovementResult {
    pub fn_id: FunctionId,
    pub success: bool,
    pub old_latency_ns: u64,
    pub new_latency_ns: Option<u64>,
    pub test_cases_used: usize,
    pub generations_run: usize,
}
