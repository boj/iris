//! Observation-driven improvement support.
//!
//! Provides `LiveRegistry` (thread-safe graph storage with hot-swap) and
//! `evaluate_and_trace` (interpreter wrapper that samples call traces).

use std::sync::Mutex;
use std::time::Instant;

use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;
use iris_types::trace::{
    FunctionId, ImprovementResult, TraceCollector, TraceEntry,
};

use crate::registry::FragmentRegistry;

// ---------------------------------------------------------------------------
// LiveRegistry — thread-safe fragment registry with hot-swap
// ---------------------------------------------------------------------------

/// A thread-safe wrapper around named graphs + fragment registry.
/// The evaluator reads from this; the daemon writes improved graphs into it.
pub struct LiveRegistry {
    /// Name → SemanticGraph for all top-level bindings.
    graphs: Mutex<std::collections::BTreeMap<String, SemanticGraph>>,
    /// Fragment registry for Ref resolution.
    registry: Mutex<FragmentRegistry>,
    /// Log of improvements made.
    improvements: Mutex<Vec<ImprovementResult>>,
}

impl LiveRegistry {
    pub fn new(
        named_graphs: std::collections::BTreeMap<String, SemanticGraph>,
        registry: FragmentRegistry,
    ) -> Self {
        Self {
            graphs: Mutex::new(named_graphs),
            registry: Mutex::new(registry),
            improvements: Mutex::new(Vec::new()),
        }
    }

    /// Get the current graph for a named function.
    pub fn get_graph(&self, name: &str) -> Option<SemanticGraph> {
        self.graphs.lock().ok()?.get(name).cloned()
    }

    /// Get all function names.
    pub fn function_names(&self) -> Vec<String> {
        self.graphs.lock().ok()
            .map(|g| g.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get a snapshot of the registry for evaluation.
    pub fn snapshot_registry(&self) -> FragmentRegistry {
        self.registry.lock().ok().map(|r| r.clone()).unwrap_or_default()
    }

    /// Get the graph map for bootstrap evaluation.
    pub fn snapshot_graph_map(&self) -> std::collections::BTreeMap<iris_types::fragment::FragmentId, SemanticGraph> {
        self.registry.lock().ok()
            .map(|r| r.to_graph_map())
            .unwrap_or_default()
    }

    /// Hot-swap a function's graph with an improved version.
    pub fn swap(&self, name: &str, new_graph: SemanticGraph, result: ImprovementResult) {
        if let Ok(mut graphs) = self.graphs.lock() {
            graphs.insert(name.to_string(), new_graph);
        }
        if let Ok(mut log) = self.improvements.lock() {
            log.push(result);
        }
    }

    /// Get improvement log.
    pub fn improvements(&self) -> Vec<ImprovementResult> {
        self.improvements.lock().ok().map(|v| v.clone()).unwrap_or_default()
    }
}

impl std::fmt::Debug for LiveRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveRegistry").finish()
    }
}

// ---------------------------------------------------------------------------
// evaluate_and_trace — wrapper that traces a fragment evaluation
// ---------------------------------------------------------------------------

/// Evaluate a fragment and record the trace if sampling says yes.
pub fn evaluate_and_trace(
    name: &str,
    graph: &SemanticGraph,
    inputs: &[Value],
    registry: &FragmentRegistry,
    collector: &TraceCollector,
) -> Result<(Vec<Value>, iris_types::eval::StateStore), crate::interpreter::InterpretError> {
    let should_trace = collector.should_sample();
    let start = if should_trace { Some(Instant::now()) } else { None };

    let result = crate::interpreter::interpret_with_registry(
        graph, inputs, None, Some(registry),
    );

    if let (Some(start), Ok((outputs, _))) = (start, &result) {
        let elapsed_ns = start.elapsed().as_nanos() as u64;
        let output = if outputs.len() == 1 {
            outputs[0].clone()
        } else {
            Value::tuple(outputs.clone())
        };
        collector.record(
            FunctionId(name.to_string()),
            TraceEntry {
                inputs: inputs.to_vec(),
                output,
                latency_ns: elapsed_ns,
            },
        );
    }

    result
}
