use serde::{Deserialize, Serialize};

use iris_types::cost::CostBound;
use iris_types::eval::TestCase;

// ---------------------------------------------------------------------------
// EvolutionConfig
// ---------------------------------------------------------------------------

/// Configuration for the evolutionary loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    /// Number of individuals per deme (default: 64).
    pub population_size: usize,
    /// Maximum number of generations to run.
    pub max_generations: usize,
    /// Probability of applying mutation to an offspring [0.0, 1.0].
    pub mutation_rate: f64,
    /// Probability of applying crossover vs cloning [0.0, 1.0].
    pub crossover_rate: f64,
    /// Tournament selection size.
    pub tournament_size: usize,
    /// Phase transition thresholds.
    pub phase_thresholds: PhaseThresholds,
    /// Target wall-clock time per generation in milliseconds.
    pub target_generation_time_ms: u64,
    /// Number of demes for multi-deme evolution (default: 1).
    /// When > 1, ring migration occurs every 5 generations.
    pub num_demes: usize,
    /// Number of nearest neighbors for novelty score computation (default: 15).
    pub novelty_k: usize,
    /// Minimum novelty score to add a behavior to the archive (default: 0.1).
    pub novelty_threshold: f32,
    /// Weight applied to novelty scores before clamping to [0, 1] (default: 1.0).
    pub novelty_weight: f32,
    /// Enable competitive coevolution of test cases (default: false).
    ///
    /// When enabled, test cases are co-evolved alongside programs. Tests
    /// that break programs are rewarded, creating an arms race that
    /// prevents fitness stagnation.
    pub coevolution: bool,
    /// Total computational budget in milliseconds for resource competition
    /// (default: 0, meaning disabled). When > 0, programs compete for
    /// evaluation time based on their fitness rank.
    pub resource_budget_ms: u64,
    /// Enable IRIS mode: use IRIS-written programs for mutation, selection,
    /// and evaluation instead of Rust functions (default: false).
    ///
    /// When true, the evolution loop creates an `IrisRuntime` and uses
    /// IRIS programs (graph_set_prim_op, graph_add_node_rt, graph_eval, etc.)
    /// for all evolutionary operators. Falls back to Rust on IRIS failure.
    #[serde(default)]
    pub iris_mode: bool,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            population_size: 64,
            max_generations: 1000,
            mutation_rate: 0.8,
            crossover_rate: 0.5,
            tournament_size: 3,
            phase_thresholds: PhaseThresholds::default(),
            target_generation_time_ms: 500,
            num_demes: 1,
            novelty_k: 15,
            novelty_threshold: 0.1,
            novelty_weight: 1.0,
            coevolution: false,
            resource_budget_ms: 0,
            iris_mode: false,
        }
    }
}

impl EvolutionConfig {
    /// Validate that all configuration values are within safe operating bounds.
    ///
    /// Returns an error string describing the first violation found, or `Ok(())`
    /// if the configuration is valid.  Call this after deserializing user-supplied
    /// configs to prevent resource amplification attacks.
    pub fn validate(&self) -> Result<(), String> {
        if self.population_size == 0 {
            return Err("population_size must be > 0".into());
        }
        if self.population_size > 10_000 {
            return Err(format!(
                "population_size {} exceeds maximum (10,000)",
                self.population_size
            ));
        }
        if self.max_generations > 100_000 {
            return Err(format!(
                "max_generations {} exceeds maximum (100,000)",
                self.max_generations
            ));
        }
        if !(0.0..=1.0).contains(&self.mutation_rate) {
            return Err(format!(
                "mutation_rate {} is not in [0.0, 1.0]",
                self.mutation_rate
            ));
        }
        if !(0.0..=1.0).contains(&self.crossover_rate) {
            return Err(format!(
                "crossover_rate {} is not in [0.0, 1.0]",
                self.crossover_rate
            ));
        }
        if self.tournament_size == 0 {
            return Err("tournament_size must be > 0".into());
        }
        if self.tournament_size > self.population_size {
            return Err(format!(
                "tournament_size {} exceeds population_size {}",
                self.tournament_size, self.population_size
            ));
        }
        if self.num_demes == 0 {
            return Err("num_demes must be > 0".into());
        }
        if self.num_demes > 64 {
            return Err(format!(
                "num_demes {} exceeds maximum (64)",
                self.num_demes
            ));
        }
        if self.novelty_k == 0 {
            return Err("novelty_k must be > 0".into());
        }
        if !(0.0..=1.0).contains(&self.novelty_threshold) {
            return Err(format!(
                "novelty_threshold {} is not in [0.0, 1.0]",
                self.novelty_threshold
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// PhaseThresholds
// ---------------------------------------------------------------------------

/// Thresholds for detecting evolutionary phase transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseThresholds {
    /// Minimum improvement rate (per generation) to stay in Exploration.
    pub exploration_min_improvement: f64,
    /// Stagnation window (generations) before switching to Exploitation.
    pub stagnation_window: usize,
    /// Minimum diversity (avg crowding distance) to stay in SteadyState.
    pub min_diversity: f32,
}

impl Default for PhaseThresholds {
    fn default() -> Self {
        Self {
            exploration_min_improvement: 0.01,
            stagnation_window: 20,
            min_diversity: 0.1,
        }
    }
}

// ---------------------------------------------------------------------------
// ProblemSpec
// ---------------------------------------------------------------------------

/// Specification of the problem to solve via evolution.
#[derive(Debug, Clone)]
pub struct ProblemSpec {
    /// Test cases for fitness evaluation.
    pub test_cases: Vec<TestCase>,
    /// Human-readable description of the target program.
    pub description: String,
    /// Optional target cost bound.
    pub target_cost: Option<CostBound>,
}
