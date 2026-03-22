//! Dynamic resource allocation for evolutionary problems.
//!
//! The "attention economy" — problems that show improvement get more compute
//! (larger populations, more generations). Stagnating problems get diversity
//! injection or early abandonment. This replaces fixed population/generation
//! counts with an adaptive budget that responds to the fitness trajectory.

// ---------------------------------------------------------------------------
// AttentionBudget
// ---------------------------------------------------------------------------

/// Bounds on how much compute the adaptive evolver can allocate.
#[derive(Debug, Clone)]
pub struct AttentionBudget {
    /// Total wall-clock budget in milliseconds.
    pub total_budget_ms: u64,
    /// Minimum population size (floor).
    pub min_population: usize,
    /// Maximum population size (ceiling).
    pub max_population: usize,
    /// Minimum generations to run (floor).
    pub min_generations: usize,
    /// Maximum generations to run (ceiling).
    pub max_generations: usize,
}

impl Default for AttentionBudget {
    fn default() -> Self {
        Self {
            total_budget_ms: 60_000, // 60 seconds
            min_population: 16,
            max_population: 1024,
            min_generations: 50,
            max_generations: 10_000,
        }
    }
}

// ---------------------------------------------------------------------------
// ResourceDecision
// ---------------------------------------------------------------------------

/// Decision about how to allocate resources for the next generation.
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceDecision {
    /// Problem is improving — invest more resources.
    Invest {
        /// Multiply population size by this factor.
        population_multiplier: f32,
        /// Add this many extra generations to the budget.
        extra_generations: usize,
    },
    /// Problem is slowly improving — maintain current resources.
    Maintain,
    /// Problem is stagnating — inject random individuals to break out.
    InjectDiversity {
        /// Fraction of population to replace with random individuals.
        random_fraction: f32,
    },
    /// Problem has stagnated beyond recovery — stop early.
    Abandon,
}

// ---------------------------------------------------------------------------
// AdaptiveEvolver
// ---------------------------------------------------------------------------

/// Tracks improvement trajectory and decides resource allocation per generation.
///
/// Each generation, `allocate_next()` is called with the current best fitness.
/// Based on the improvement rate, the evolver decides whether to invest more
/// compute, maintain the status quo, inject diversity, or abandon the problem.
pub struct AdaptiveEvolver {
    /// Resource bounds.
    budget: AttentionBudget,
    /// Exponentially smoothed improvement rate (fitness delta per generation).
    improvement_rate: f32,
    /// Best fitness observed last generation (sum of objectives).
    last_best_fitness: f32,
    /// Consecutive generations without significant improvement.
    stagnation_count: u32,
    /// Current effective population size.
    current_population: usize,
    /// Total generations allocated so far (including extras from Invest).
    allocated_generations: usize,
    /// Smoothing factor for exponential moving average of improvement.
    smoothing: f32,
}

impl AdaptiveEvolver {
    /// Create a new adaptive evolver with the given budget.
    pub fn new(budget: AttentionBudget) -> Self {
        let initial_pop = budget.min_population;
        let initial_gens = budget.min_generations;
        Self {
            budget,
            improvement_rate: 0.0,
            last_best_fitness: 0.0,
            stagnation_count: 0,
            current_population: initial_pop,
            allocated_generations: initial_gens,
            smoothing: 0.3,
        }
    }

    /// Decide how many resources to allocate for the NEXT generation
    /// based on the improvement trajectory.
    ///
    /// `current_best` is the sum of fitness objectives for the best
    /// individual in the current generation.
    pub fn allocate_next(&mut self, current_best: f32) -> ResourceDecision {
        let improvement = current_best - self.last_best_fitness;
        self.last_best_fitness = current_best;

        // Exponential moving average of improvement rate.
        self.improvement_rate =
            self.smoothing * improvement + (1.0 - self.smoothing) * self.improvement_rate;

        if improvement > 0.01 {
            // Improving — invest more.
            self.stagnation_count = 0;

            // Grow population (clamped to budget ceiling).
            let new_pop = ((self.current_population as f32 * 1.5) as usize)
                .min(self.budget.max_population);
            self.current_population = new_pop;

            // Extend the generation budget.
            self.allocated_generations = (self.allocated_generations + 100)
                .min(self.budget.max_generations);

            ResourceDecision::Invest {
                population_multiplier: 1.5,
                extra_generations: 100,
            }
        } else if improvement > 0.001 {
            // Slow improvement — maintain current allocation.
            self.stagnation_count = 0;
            ResourceDecision::Maintain
        } else {
            // Stagnating — either inject diversity or give up.
            self.stagnation_count += 1;

            if self.stagnation_count > 50 {
                ResourceDecision::Abandon
            } else {
                ResourceDecision::InjectDiversity {
                    random_fraction: 0.3,
                }
            }
        }
    }

    /// Current effective population size.
    pub fn current_population(&self) -> usize {
        self.current_population
    }

    /// Total generations allocated so far.
    pub fn allocated_generations(&self) -> usize {
        self.allocated_generations
    }

    /// Number of consecutive stagnating generations.
    pub fn stagnation_count(&self) -> u32 {
        self.stagnation_count
    }

    /// The smoothed improvement rate.
    pub fn improvement_rate(&self) -> f32 {
        self.improvement_rate
    }

    /// The budget bounds.
    pub fn budget(&self) -> &AttentionBudget {
        &self.budget
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn default_budget() -> AttentionBudget {
        AttentionBudget {
            total_budget_ms: 10_000,
            min_population: 16,
            max_population: 256,
            min_generations: 50,
            max_generations: 1000,
        }
    }

    #[test]
    fn improving_problem_gets_more_resources() {
        let mut evolver = AdaptiveEvolver::new(default_budget());
        assert_eq!(evolver.current_population(), 16);
        assert_eq!(evolver.allocated_generations(), 50);

        // Simulate steady improvement: each generation improves by 0.05.
        let mut fitness = 0.0;
        for _ in 0..5 {
            fitness += 0.05;
            let decision = evolver.allocate_next(fitness);
            assert!(
                matches!(decision, ResourceDecision::Invest { .. }),
                "Expected Invest for improving problem, got {:?}",
                decision,
            );
        }

        // Population should have grown from 16.
        assert!(
            evolver.current_population() > 16,
            "Population should grow: got {}",
            evolver.current_population(),
        );
        // Generations should have grown from 50.
        assert!(
            evolver.allocated_generations() > 50,
            "Generations should grow: got {}",
            evolver.allocated_generations(),
        );
    }

    #[test]
    fn population_respects_ceiling() {
        let budget = AttentionBudget {
            max_population: 32,
            ..default_budget()
        };
        let mut evolver = AdaptiveEvolver::new(budget);

        // Push population growth many times.
        let mut fitness = 0.0;
        for _ in 0..20 {
            fitness += 0.1;
            evolver.allocate_next(fitness);
        }

        assert!(
            evolver.current_population() <= 32,
            "Population must not exceed ceiling: got {}",
            evolver.current_population(),
        );
    }

    #[test]
    fn generations_respect_ceiling() {
        let budget = AttentionBudget {
            max_generations: 200,
            ..default_budget()
        };
        let mut evolver = AdaptiveEvolver::new(budget);

        let mut fitness = 0.0;
        for _ in 0..20 {
            fitness += 0.1;
            evolver.allocate_next(fitness);
        }

        assert!(
            evolver.allocated_generations() <= 200,
            "Generations must not exceed ceiling: got {}",
            evolver.allocated_generations(),
        );
    }

    #[test]
    fn slow_improvement_maintains() {
        let mut evolver = AdaptiveEvolver::new(default_budget());

        // Set a baseline fitness first.
        evolver.allocate_next(1.0);

        // Slow improvements: each step adds 0.005 (between 0.001 and 0.01).
        let mut fitness = 1.0;
        for _ in 0..5 {
            fitness += 0.005;
            let decision = evolver.allocate_next(fitness);
            assert_eq!(
                decision,
                ResourceDecision::Maintain,
                "Expected Maintain for slowly improving problem",
            );
        }
    }

    #[test]
    fn stagnating_problem_gets_diversity_injection() {
        let mut evolver = AdaptiveEvolver::new(default_budget());

        // Set a baseline.
        evolver.allocate_next(1.0);

        // No improvement at all.
        for i in 0..10 {
            let decision = evolver.allocate_next(1.0);
            assert!(
                matches!(decision, ResourceDecision::InjectDiversity { random_fraction } if random_fraction == 0.3),
                "Generation {}: expected InjectDiversity, got {:?}",
                i,
                decision,
            );
        }

        assert!(
            evolver.stagnation_count() > 0,
            "Stagnation counter should be positive",
        );
    }

    #[test]
    fn abandoned_problem_stops_early() {
        let mut evolver = AdaptiveEvolver::new(default_budget());

        // Set a baseline.
        evolver.allocate_next(1.0);

        // Stagnate for 51 generations to trigger abandonment.
        let mut last_decision = ResourceDecision::Maintain;
        for _ in 0..51 {
            last_decision = evolver.allocate_next(1.0);
        }

        assert_eq!(
            last_decision,
            ResourceDecision::Abandon,
            "Expected Abandon after 51 stagnating generations",
        );
    }

    #[test]
    fn improvement_resets_stagnation() {
        let mut evolver = AdaptiveEvolver::new(default_budget());

        // Set a baseline and stagnate for 30 generations.
        evolver.allocate_next(1.0);
        for _ in 0..30 {
            evolver.allocate_next(1.0);
        }
        assert!(evolver.stagnation_count() > 0);

        // Now improve significantly.
        let decision = evolver.allocate_next(1.1);
        assert!(
            matches!(decision, ResourceDecision::Invest { .. }),
            "Expected Invest after breaking stagnation, got {:?}",
            decision,
        );
        assert_eq!(evolver.stagnation_count(), 0, "Stagnation should reset");
    }

    #[test]
    fn initial_state_is_correct() {
        let budget = default_budget();
        let evolver = AdaptiveEvolver::new(budget.clone());

        assert_eq!(evolver.current_population(), budget.min_population);
        assert_eq!(evolver.allocated_generations(), budget.min_generations);
        assert_eq!(evolver.stagnation_count(), 0);
        assert_eq!(evolver.improvement_rate(), 0.0);
    }
}
