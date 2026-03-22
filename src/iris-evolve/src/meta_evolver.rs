//! `IrisMetaEvolver` — concrete implementation of the `MetaEvolver` trait.
//!
//! Enables the `evolve_subprogram` opcode (0xA0): a running program can
//! construct test cases at runtime, invoke evolution, and receive a bred
//! sub-program as `Value::Program`.

use std::time::{Duration, Instant};

use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_exec::MetaEvolver;
use iris_types::eval::TestCase;
use iris_types::graph::SemanticGraph;

use crate::config::{EvolutionConfig, PhaseThresholds, ProblemSpec};

// ---------------------------------------------------------------------------
// IrisMetaEvolver
// ---------------------------------------------------------------------------

/// Meta-evolver that delegates to `iris_evolve::evolve()`.
///
/// Creates a fresh `IrisExecutionService` per call (no shared state with
/// the calling program's execution). Limits wall-clock time and population
/// size for responsiveness.
pub struct IrisMetaEvolver {
    /// Maximum wall-clock time per `evolve_subprogram` call.
    pub max_wall_time: Duration,
    /// Population size for sub-evolution (smaller than normal for speed).
    pub population_size: usize,
    /// Maximum meta-evolution nesting depth (0-indexed).
    /// 0 = top-level can call evolve, 1 = evolved program can call evolve,
    /// 2+ = rejected.
    pub max_depth: u32,
}

impl Default for IrisMetaEvolver {
    fn default() -> Self {
        Self {
            max_wall_time: Duration::from_secs(5),
            population_size: 32,
            // Depth 1: only the top-level program may call evolve_subprogram.
            // Deeper nesting amplifies CPU unboundedly and is rejected.
            max_depth: 1,
        }
    }
}

impl IrisMetaEvolver {
    /// Create a new meta-evolver with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a meta-evolver with custom settings.
    pub fn with_config(max_wall_time: Duration, population_size: usize, max_depth: u32) -> Self {
        Self {
            max_wall_time,
            population_size,
            max_depth,
        }
    }
}

impl MetaEvolver for IrisMetaEvolver {
    fn evolve_subprogram(
        &self,
        test_cases: Vec<TestCase>,
        max_generations: usize,
        meta_depth: u32,
    ) -> Result<SemanticGraph, String> {
        // Depth check.
        if meta_depth >= self.max_depth {
            return Err(format!(
                "meta-evolve depth {} exceeds limit {}",
                meta_depth, self.max_depth
            ));
        }

        if test_cases.is_empty() {
            return Err("no test cases provided for sub-evolution".into());
        }

        // Create a fresh execution service for the sub-evolution.
        let exec = IrisExecutionService::new(ExecConfig {
            cache_capacity: 256,
            worker_threads: 2,
            ..ExecConfig::default()
        });

        let spec = ProblemSpec {
            test_cases,
            description: format!("meta-evolved sub-program (depth {})", meta_depth),
            target_cost: None,
        };

        let config = EvolutionConfig {
            population_size: self.population_size,
            max_generations,
            mutation_rate: 0.8,
            crossover_rate: 0.5,
            tournament_size: 3,
            phase_thresholds: PhaseThresholds {
                exploration_min_improvement: 0.005,
                stagnation_window: 15,
                min_diversity: 0.1,
            },
            target_generation_time_ms: 200,
            num_demes: 1,
            novelty_k: 15,
            novelty_threshold: 0.1,
            novelty_weight: 1.0,
            coevolution: false,
            resource_budget_ms: 0,
            iris_mode: false,
        };

        // Run evolution with a wall-clock time limit.
        let start = Instant::now();
        let result = crate::evolve_with_timeout(config, spec, &exec, self.max_wall_time);

        let elapsed = start.elapsed();
        eprintln!(
            "iris-evolve: meta-evolution (depth {}) completed in {:.2?}, \
             {} generations, best correctness: {:.2}%",
            meta_depth,
            elapsed,
            result.generations_run,
            result.best_individual.fitness.correctness() * 100.0,
        );

        Ok(result.best_individual.fragment.graph.clone())
    }
}
