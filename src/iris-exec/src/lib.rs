//! `iris-exec` — Execution Service bridging L0 (evolution) and L3 (hardware).
//!
//! Provides thin shims delegating to `iris-bootstrap` for the IRIS
//! meta-circular interpreter + bootstrap evaluator. Core types, traits,
//! and lightweight infrastructure (registry, message bus, cache, stats)
//! are always available.

pub mod backend;
pub mod cache;
pub mod capabilities;
pub mod effect_runtime;
pub mod improve;
pub mod message_bus;
pub mod registry;
pub mod stats;

pub mod interpreter;
pub mod jit_backend;
pub mod clcu_backend;
pub mod service;

use iris_types::eval::{EvalError, EvalResult, EvalTier, TestCase};
use iris_types::fragment::FragmentId;
use iris_types::graph::SemanticGraph;

use crate::stats::CacheStats;

// ---------------------------------------------------------------------------
// ExecutionService trait (SPEC Section 8.2)
// ---------------------------------------------------------------------------

/// Orchestration interface between L0 (evolutionary substrate) and L3
/// (hardware materialization).
pub trait ExecutionService {
    /// Evaluate a single program against the given test cases at the
    /// requested evaluation tier.
    fn evaluate_individual(
        &self,
        program: &SemanticGraph,
        test_inputs: &[TestCase],
        tier: EvalTier,
    ) -> Result<EvalResult, EvalError>;

    /// Evaluate a batch of programs in parallel. Returns one `EvalResult`
    /// per program, in the same order.
    fn evaluate_batch(
        &self,
        programs: &[SemanticGraph],
        test_inputs: &[TestCase],
        tier: EvalTier,
    ) -> Result<Vec<EvalResult>, EvalError>;

    /// Evict specific entries from the compilation cache.
    fn evict_cache(&self, graph_ids: &[FragmentId]);

    /// Return a snapshot of compilation cache statistics.
    fn cache_stats(&self) -> CacheStats;
}

// ---------------------------------------------------------------------------
// MetaEvolver trait — runtime sub-evolution from within programs
// ---------------------------------------------------------------------------

/// Trait for performing meta-evolution: breeding sub-programs at runtime
/// to satisfy a caller-defined specification.
///
/// This enables the `evolve_subprogram` opcode (0xA0): a running program
/// can construct test cases, invoke the evolutionary engine, and receive
/// an evolved sub-program as a `Value::Program`.
///
/// Implementations must be `Send + Sync` for use across threads.
pub trait MetaEvolver: Send + Sync {
    /// Evolve a sub-program that satisfies the given test cases.
    ///
    /// # Arguments
    /// - `test_cases`: The specification (input/output pairs) the evolved
    ///   program must satisfy.
    /// - `max_generations`: Budget cap on evolutionary generations.
    /// - `meta_depth`: Current meta-evolution depth (0 = top-level program
    ///   calling evolve, 1 = program evolved by evolve calling evolve again).
    ///   Implementations should refuse if `meta_depth >= max_depth`.
    ///
    /// # Returns
    /// The best evolved program's `SemanticGraph`, or an error message.
    fn evolve_subprogram(
        &self,
        test_cases: Vec<TestCase>,
        max_generations: usize,
        meta_depth: u32,
    ) -> Result<SemanticGraph, String>;
}
