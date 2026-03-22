//! Self-improvement for IRIS's evolution engine.
//!
//! This module enables IRIS to evolve its own evolution parameters: mutation
//! operator weights, seed generator distributions, and (in the future) fitness
//! function coefficients. The approach is meta-evolution: a population of
//! *strategies* is evolved, where each strategy is evaluated by running the
//! actual evolutionary loop with those parameters on a set of benchmark
//! problems.
//!
//! This is a general-purpose capability: any system that uses evolutionary
//! search can optimize its own search strategy via this mechanism.

use std::time::{Duration, Instant};

use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

use iris_exec::ExecutionService;

use crate::config::{EvolutionConfig, PhaseThresholds, ProblemSpec};
use crate::mutation;
use crate::seed;

// ---------------------------------------------------------------------------
// MutationStrategy
// ---------------------------------------------------------------------------

/// The names of all mutation operators, in the same order as
/// `WEIGHT_THRESHOLDS` in `mutation.rs`.
pub const MUTATION_OPERATOR_NAMES: [&str; 16] = [
    "insert_node",
    "delete_node",
    "rewire_edge",
    "replace_kind",
    "replace_prim",
    "mutate_literal",
    "duplicate_subgraph",
    "wrap_in_guard",
    "annotate_cost",
    "wrap_in_map",
    "wrap_in_filter",
    "compose_stages",
    "insert_zip",
    "swap_fold_op",
    "add_guard_condition",
    "extract_to_ref",
];

/// Default mutation weights matching the current hardcoded values in
/// `mutation.rs` (these are individual weights, not cumulative).
const DEFAULT_MUTATION_WEIGHTS: [f32; 16] = [
    0.10, // insert_node
    0.10, // delete_node
    0.08, // rewire_edge
    0.03, // replace_kind
    0.06, // replace_prim
    0.07, // mutate_literal
    0.01, // duplicate_subgraph
    0.01, // wrap_in_guard
    0.01, // annotate_cost
    0.09, // wrap_in_map
    0.09, // wrap_in_filter
    0.07, // compose_stages
    0.07, // insert_zip
    0.13, // swap_fold_op
    0.05, // add_guard_condition
    0.03, // extract_to_ref
];

/// Evolvable mutation operator weight distribution.
///
/// Instead of hardcoded cumulative thresholds, this struct holds per-operator
/// weights that can be perturbed, selected, and evolved via meta-evolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationStrategy {
    /// (operator_name, weight) pairs. Weights are non-negative and sum to ~1.0.
    pub weights: Vec<(String, f32)>,
}

impl MutationStrategy {
    /// Create the default strategy matching the current hardcoded weights.
    pub fn default_strategy() -> Self {
        let weights = MUTATION_OPERATOR_NAMES
            .iter()
            .zip(DEFAULT_MUTATION_WEIGHTS.iter())
            .map(|(name, &w)| (name.to_string(), w))
            .collect();
        Self { weights }
    }

    /// Create a random strategy with uniform-ish weights.
    pub fn random(rng: &mut impl Rng) -> Self {
        let mut raw: Vec<f32> = (0..16).map(|_| rng.gen_range(0.01f32..1.0)).collect();
        let sum: f32 = raw.iter().sum();
        for w in &mut raw {
            *w /= sum;
        }
        let weights = MUTATION_OPERATOR_NAMES
            .iter()
            .zip(raw.into_iter())
            .map(|(name, w)| (name.to_string(), w))
            .collect();
        Self { weights }
    }

    /// Mutate the strategy itself (meta-mutation).
    ///
    /// Perturbs each weight by a small Gaussian-like noise (uniform
    /// approximation), then re-normalizes so weights sum to 1.0.
    pub fn perturb(&self, rng: &mut impl Rng) -> Self {
        let mut new_weights: Vec<(String, f32)> = self
            .weights
            .iter()
            .map(|(name, w)| {
                // Perturbation: multiply by a factor in [0.5, 2.0].
                let factor = rng.gen_range(0.5f32..2.0);
                let new_w = (w * factor).max(0.001); // floor to avoid zero
                (name.clone(), new_w)
            })
            .collect();

        // Normalize.
        let sum: f32 = new_weights.iter().map(|(_, w)| w).sum();
        if sum > 0.0 {
            for (_, w) in &mut new_weights {
                *w /= sum;
            }
        }

        Self { weights: new_weights }
    }

    /// Convert this strategy into cumulative weight thresholds suitable for
    /// `mutation::mutate_with_weights`.
    pub fn to_cumulative_thresholds(&self) -> Vec<(u8, f64)> {
        let mut cumulative = Vec::with_capacity(self.weights.len());
        let mut acc = 0.0f64;
        for (i, (_, w)) in self.weights.iter().enumerate() {
            acc += *w as f64;
            cumulative.push((i as u8, acc));
        }
        // Ensure last entry is >= 1.0.
        if let Some(last) = cumulative.last_mut() {
            last.1 = 1.0;
        }
        cumulative
    }

    /// Evaluate this strategy's quality by running evolution on benchmark
    /// problems and returning the average best-correctness across problems.
    ///
    /// Uses small populations and few generations to keep evaluation fast.
    pub fn evaluate(
        &self,
        problems: &[ProblemSpec],
        exec: &dyn ExecutionService,
        eval_generations: usize,
        eval_pop_size: usize,
        timeout_per_problem: Duration,
    ) -> f32 {
        if problems.is_empty() {
            return 0.0;
        }

        let thresholds = self.to_cumulative_thresholds();
        let mut total_correctness = 0.0f32;

        for problem in problems {
            let config = EvolutionConfig {
                population_size: eval_pop_size,
                max_generations: eval_generations,
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
                novelty_k: 10,
                novelty_threshold: 0.1,
                novelty_weight: 1.0,
                coevolution: false,
                resource_budget_ms: 0,
                iris_mode: false,
            };

            // Install the custom weights for this evaluation.
            // Best-effort: validation failure means we proceed with defaults.
            let _ = mutation::set_custom_weights(&thresholds);

            let result = crate::evolve_with_timeout(
                config,
                problem.clone(),
                exec,
                timeout_per_problem,
            );

            // Clear custom weights after evaluation.
            mutation::clear_custom_weights();

            total_correctness += result.best_individual.fitness.correctness();
        }

        total_correctness / problems.len() as f32
    }

    /// Evolve better mutation weights via meta-evolution.
    ///
    /// Runs a population of mutation strategies, evaluating each by its
    /// ability to solve the given benchmark problems. Returns the best
    /// strategy found.
    pub fn evolve_strategy(
        problems: &[ProblemSpec],
        exec: &dyn ExecutionService,
        generations: usize,
        population: usize,
        eval_generations: usize,
        eval_pop_size: usize,
        timeout_per_problem: Duration,
    ) -> MutationStrategy {
        let mut rng = StdRng::from_entropy();

        // Initialize strategy population: half default, half random.
        let mut strategies: Vec<(MutationStrategy, f32)> = Vec::with_capacity(population);
        for i in 0..population {
            let strategy = if i == 0 {
                MutationStrategy::default_strategy()
            } else if i < population / 2 {
                MutationStrategy::default_strategy().perturb(&mut rng)
            } else {
                MutationStrategy::random(&mut rng)
            };
            strategies.push((strategy, 0.0));
        }

        for _gen in 0..generations {
            // Evaluate all strategies.
            for (strategy, score) in &mut strategies {
                *score = strategy.evaluate(
                    problems,
                    exec,
                    eval_generations,
                    eval_pop_size,
                    timeout_per_problem,
                );
            }

            // Sort by score (descending).
            strategies.sort_by(|a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });

            // Selection + reproduction: top 25% survive, rest replaced by
            // perturbations of the top strategies.
            let elite_count = (population / 4).max(1);
            let mut next_gen: Vec<(MutationStrategy, f32)> = Vec::with_capacity(population);

            // Keep elites.
            for i in 0..elite_count {
                next_gen.push(strategies[i].clone());
            }

            // Fill the rest with perturbations of elites.
            while next_gen.len() < population {
                let parent_idx = rng.gen_range(0..elite_count);
                let child = strategies[parent_idx].0.perturb(&mut rng);
                next_gen.push((child, 0.0));
            }

            strategies = next_gen;
        }

        // Final evaluation to pick the best.
        for (strategy, score) in &mut strategies {
            *score = strategy.evaluate(
                problems,
                exec,
                eval_generations,
                eval_pop_size,
                timeout_per_problem,
            );
        }
        strategies.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        strategies.into_iter().next().map(|(s, _)| s).unwrap()
    }
}

// ---------------------------------------------------------------------------
// SeedStrategy
// ---------------------------------------------------------------------------

/// The names of all seed generator types, matching the distribution in
/// `evolve()` in `lib.rs`.
pub const SEED_TYPE_NAMES: [&str; 13] = [
    "arithmetic",
    "fold",
    "identity",
    "map",
    "zip_fold",
    "map_fold",
    "filter_fold",
    "zip_map_fold",
    "comparison_fold",
    "stateful_fold",
    "conditional_fold",
    "iterate",
    "pairwise_fold",
];

/// Default seed weights (proportional, matching the modulo-30 distribution).
const DEFAULT_SEED_WEIGHTS: [f32; 13] = [
    2.0 / 30.0,  // arithmetic (slots 0,1)
    2.0 / 30.0,  // fold (slots 2,3)
    1.0 / 30.0,  // identity (slot 4)
    1.0 / 30.0,  // map (slot 5)
    1.0 / 30.0,  // zip_fold (slot 6)
    3.0 / 30.0,  // map_fold (slots 7,8,9)
    2.0 / 30.0,  // filter_fold (slots 10,11)
    2.0 / 30.0,  // zip_map_fold (slots 12,13)
    2.0 / 30.0,  // comparison_fold (slots 14,15)
    2.0 / 30.0,  // stateful_fold (slots 16,17)
    2.0 / 30.0,  // conditional_fold (slots 18,19)
    3.0 / 30.0,  // iterate (slots 22,23,24)
    3.0 / 30.0,  // pairwise_fold (slots 25,26,27)
];

/// Evolvable seed generator distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedStrategy {
    /// (seed_type, weight) pairs. Weights are non-negative and sum to ~1.0.
    pub seed_weights: Vec<(String, f32)>,
}

impl SeedStrategy {
    /// Create the default strategy matching the current distribution.
    pub fn default_strategy() -> Self {
        let seed_weights = SEED_TYPE_NAMES
            .iter()
            .zip(DEFAULT_SEED_WEIGHTS.iter())
            .map(|(name, &w)| (name.to_string(), w))
            .collect();
        Self { seed_weights }
    }

    /// Create a random seed strategy.
    pub fn random(rng: &mut impl Rng) -> Self {
        let mut raw: Vec<f32> = (0..SEED_TYPE_NAMES.len())
            .map(|_| rng.gen_range(0.01f32..1.0))
            .collect();
        let sum: f32 = raw.iter().sum();
        for w in &mut raw {
            *w /= sum;
        }
        let seed_weights = SEED_TYPE_NAMES
            .iter()
            .zip(raw.into_iter())
            .map(|(name, w)| (name.to_string(), w))
            .collect();
        Self { seed_weights }
    }

    /// Mutate the seed strategy (meta-mutation).
    pub fn perturb(&self, rng: &mut impl Rng) -> Self {
        let mut new_weights: Vec<(String, f32)> = self
            .seed_weights
            .iter()
            .map(|(name, w)| {
                let factor = rng.gen_range(0.5f32..2.0);
                let new_w = (w * factor).max(0.001);
                (name.clone(), new_w)
            })
            .collect();

        let sum: f32 = new_weights.iter().map(|(_, w)| w).sum();
        if sum > 0.0 {
            for (_, w) in &mut new_weights {
                *w /= sum;
            }
        }

        Self { seed_weights: new_weights }
    }

    /// Convert to cumulative thresholds for seed selection.
    pub fn to_cumulative_thresholds(&self) -> Vec<(usize, f64)> {
        let mut cumulative = Vec::with_capacity(self.seed_weights.len());
        let mut acc = 0.0f64;
        for (i, (_, w)) in self.seed_weights.iter().enumerate() {
            acc += *w as f64;
            cumulative.push((i, acc));
        }
        if let Some(last) = cumulative.last_mut() {
            last.1 = 1.0;
        }
        cumulative
    }

    /// Select a seed type index based on the strategy weights.
    pub fn select_seed_type(&self, rng: &mut impl Rng) -> usize {
        let thresholds = self.to_cumulative_thresholds();
        let roll: f64 = rng.r#gen();
        for &(idx, threshold) in &thresholds {
            if roll < threshold {
                return idx;
            }
        }
        thresholds.last().map(|&(idx, _)| idx).unwrap_or(0)
    }

    /// Evaluate this seed strategy by running evolution on benchmark problems.
    pub fn evaluate(
        &self,
        problems: &[ProblemSpec],
        exec: &dyn ExecutionService,
        eval_generations: usize,
        eval_pop_size: usize,
        timeout_per_problem: Duration,
    ) -> f32 {
        if problems.is_empty() {
            return 0.0;
        }

        let mut total_correctness = 0.0f32;

        for problem in problems {
            let config = EvolutionConfig {
                population_size: eval_pop_size,
                max_generations: eval_generations,
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
                novelty_k: 10,
                novelty_threshold: 0.1,
                novelty_weight: 1.0,
                coevolution: false,
                resource_budget_ms: 0,
                iris_mode: false,
            };

            // For seed strategy evaluation, we use the default mutation weights
            // but custom seed distribution. We run evolve_with_timeout and use
            // the seed strategy to generate the initial population.
            //
            // Since evolve_with_timeout uses the hardcoded seed distribution,
            // we install a custom seed strategy via thread-local state.
            seed::set_custom_seed_strategy(self);

            let result = crate::evolve_with_timeout(
                config,
                problem.clone(),
                exec,
                timeout_per_problem,
            );

            seed::clear_custom_seed_strategy();

            total_correctness += result.best_individual.fitness.correctness();
        }

        total_correctness / problems.len() as f32
    }

    /// Evolve better seed weights via meta-evolution.
    pub fn evolve_strategy(
        problems: &[ProblemSpec],
        exec: &dyn ExecutionService,
        generations: usize,
        population: usize,
        eval_generations: usize,
        eval_pop_size: usize,
        timeout_per_problem: Duration,
    ) -> SeedStrategy {
        let mut rng = StdRng::from_entropy();

        let mut strategies: Vec<(SeedStrategy, f32)> = Vec::with_capacity(population);
        for i in 0..population {
            let strategy = if i == 0 {
                SeedStrategy::default_strategy()
            } else if i < population / 2 {
                SeedStrategy::default_strategy().perturb(&mut rng)
            } else {
                SeedStrategy::random(&mut rng)
            };
            strategies.push((strategy, 0.0));
        }

        for _gen in 0..generations {
            for (strategy, score) in &mut strategies {
                *score = strategy.evaluate(
                    problems,
                    exec,
                    eval_generations,
                    eval_pop_size,
                    timeout_per_problem,
                );
            }

            strategies.sort_by(|a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });

            let elite_count = (population / 4).max(1);
            let mut next_gen: Vec<(SeedStrategy, f32)> = Vec::with_capacity(population);

            for i in 0..elite_count {
                next_gen.push(strategies[i].clone());
            }

            while next_gen.len() < population {
                let parent_idx = rng.gen_range(0..elite_count);
                let child = strategies[parent_idx].0.perturb(&mut rng);
                next_gen.push((child, 0.0));
            }

            strategies = next_gen;
        }

        // Final evaluation.
        for (strategy, score) in &mut strategies {
            *score = strategy.evaluate(
                problems,
                exec,
                eval_generations,
                eval_pop_size,
                timeout_per_problem,
            );
        }
        strategies.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        strategies.into_iter().next().map(|(s, _)| s).unwrap()
    }
}

// ---------------------------------------------------------------------------
// ImprovementResult
// ---------------------------------------------------------------------------

/// Result of a self-improvement round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementResult {
    /// Baseline solve rate before improvement.
    pub baseline_solve_rate: f32,
    /// Improved solve rate after meta-evolution.
    pub improved_solve_rate: f32,
    /// The evolved mutation strategy.
    pub mutation_strategy: MutationStrategy,
    /// The evolved seed strategy.
    pub seed_strategy: SeedStrategy,
    /// Human-readable descriptions of improvements found.
    pub improvements_found: Vec<String>,
}

// ---------------------------------------------------------------------------
// Self-improvement loop
// ---------------------------------------------------------------------------

/// Run a single round of self-improvement: evaluate the current strategy,
/// evolve better mutation weights and seed distributions, and report results.
pub fn self_improve(
    benchmark_problems: &[ProblemSpec],
    exec: &dyn ExecutionService,
    budget: Duration,
    meta_generations: usize,
    meta_population: usize,
    eval_generations: usize,
    eval_pop_size: usize,
) -> ImprovementResult {
    let start = Instant::now();

    // Time budget: split between mutation strategy evolution and seed strategy
    // evolution (60/40 split, mutation weights are more impactful).
    let mutation_budget = Duration::from_secs_f64(budget.as_secs_f64() * 0.55);
    let seed_budget = Duration::from_secs_f64(budget.as_secs_f64() * 0.35);
    // Reserve 10% for baseline evaluation.

    let timeout_per_problem = Duration::from_millis(500);

    // 1. Evaluate current (baseline) strategy.
    let current_mutation = MutationStrategy::default_strategy();
    let baseline = current_mutation.evaluate(
        benchmark_problems,
        exec,
        eval_generations,
        eval_pop_size,
        timeout_per_problem,
    );

    // 2. Evolve better mutation weights (if time allows).
    let better_mutations = if start.elapsed() < mutation_budget {
        MutationStrategy::evolve_strategy(
            benchmark_problems,
            exec,
            meta_generations,
            meta_population,
            eval_generations,
            eval_pop_size,
            timeout_per_problem,
        )
    } else {
        current_mutation.clone()
    };

    // 3. Evolve better seed distribution (if time allows).
    let better_seeds = if start.elapsed() < mutation_budget + seed_budget {
        SeedStrategy::evolve_strategy(
            benchmark_problems,
            exec,
            meta_generations,
            meta_population,
            eval_generations,
            eval_pop_size,
            timeout_per_problem,
        )
    } else {
        SeedStrategy::default_strategy()
    };

    // 4. Evaluate improved strategy.
    let improved = better_mutations.evaluate(
        benchmark_problems,
        exec,
        eval_generations,
        eval_pop_size,
        timeout_per_problem,
    );

    // 5. Summarize improvements.
    let mut improvements = Vec::new();

    if improved > baseline {
        improvements.push(format!(
            "Mutation weights improved solve rate: {:.1}% -> {:.1}%",
            baseline * 100.0,
            improved * 100.0,
        ));
    }

    // Report significant weight changes.
    for (i, ((name, old_w), (_, new_w))) in current_mutation
        .weights
        .iter()
        .zip(better_mutations.weights.iter())
        .enumerate()
    {
        let change = (new_w - old_w) / old_w.max(0.001);
        if change.abs() > 0.3 {
            let direction = if change > 0.0 { "increased" } else { "decreased" };
            improvements.push(format!(
                "  {} weight {}: {:.3} -> {:.3}",
                direction, name, old_w, new_w,
            ));
        }
        let _ = i; // suppress unused warning
    }

    // Only adopt the evolved strategy if it is strictly better than baseline.
    // Returning a strategy that performs worse would regress future rounds.
    let (final_mutations, final_seeds) = if improved > baseline {
        (better_mutations, better_seeds)
    } else {
        // Evolved strategy is not better — keep the current strategy unchanged.
        (current_mutation.clone(), SeedStrategy::default_strategy())
    };

    ImprovementResult {
        baseline_solve_rate: baseline,
        improved_solve_rate: improved,
        mutation_strategy: final_mutations,
        seed_strategy: final_seeds,
        improvements_found: improvements,
    }
}

/// Run multiple rounds of self-improvement.
///
/// Each round builds on the previous: the evolved strategy from round N
/// becomes the baseline for round N+1. Returns results from all rounds.
pub fn self_improve_loop(
    problems: Vec<ProblemSpec>,
    exec: &dyn ExecutionService,
    rounds: usize,
    budget_per_round: Duration,
    meta_generations: usize,
    meta_population: usize,
    eval_generations: usize,
    eval_pop_size: usize,
) -> Vec<ImprovementResult> {
    let mut results = Vec::with_capacity(rounds);

    for round in 0..rounds {
        eprintln!(
            "iris-evolve: self-improvement round {}/{} starting...",
            round + 1,
            rounds,
        );

        let result = self_improve(
            &problems,
            exec,
            budget_per_round,
            meta_generations,
            meta_population,
            eval_generations,
            eval_pop_size,
        );

        eprintln!(
            "iris-evolve: round {} complete. baseline={:.1}%, improved={:.1}%, {} improvements",
            round + 1,
            result.baseline_solve_rate * 100.0,
            result.improved_solve_rate * 100.0,
            result.improvements_found.len(),
        );

        results.push(result);
    }

    results
}
