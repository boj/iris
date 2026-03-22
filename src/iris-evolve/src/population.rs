use rand::Rng;

use iris_exec::ExecutionService;
use iris_exec::message_bus::MessageBus;
use iris_types::eval::{EvalTier, TestCase, Value};
use iris_types::fragment::Fragment;

use crate::config::EvolutionConfig;
use crate::crossover;
use crate::ecosystem::FragmentEcosystem;
use crate::individual::{Fitness, Individual};
use crate::iris_runtime::IrisRuntime;
use crate::lexicase;
use crate::mutation;
use crate::novelty::NoveltyArchive;
use crate::nsga2;
use crate::phase::{PhaseDetector, phase_params};
use crate::resource::ResourceAllocator;
use crate::stigmergy::StigmergicField;
use crate::verify;

// ---------------------------------------------------------------------------
// Phase
// ---------------------------------------------------------------------------

/// Evolutionary phase, detected from fitness improvement dynamics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Phase {
    /// High diversity, rapid improvement — favor exploration.
    Exploration,
    /// Moderate improvement — balanced search.
    SteadyState,
    /// Low improvement, convergence — favor exploitation of elites.
    Exploitation,
}

// ---------------------------------------------------------------------------
// Deme
// ---------------------------------------------------------------------------

/// Tracks mutation locations for stigmergic analysis.
#[derive(Debug, Clone, Default)]
pub struct MutationTracker {
    /// Maps (node_index_within_graph) -> number of times mutated.
    pub mutation_counts: std::collections::HashMap<u64, usize>,
    /// Maps (node_index_within_graph) -> number of times that mutation improved fitness.
    pub improvement_counts: std::collections::HashMap<u64, usize>,
}

/// A single deme (sub-population) of individuals.
pub struct Deme {
    /// Current population.
    pub individuals: Vec<Individual>,
    /// Current generation counter.
    pub generation: usize,
    /// Current evolutionary phase.
    pub phase: Phase,
    /// Phase detector with rolling windows and stagnation tracking.
    phase_detector: PhaseDetector,
    /// Novelty archive for behavioral diversity search.
    novelty_archive: NoveltyArchive,
    /// Stigmergic field: shared signal grid in embedding space (SPEC 6.7).
    pub(crate) stigmergic_field: StigmergicField,
    /// Codec for computing embeddings (used by stigmergic field).
    /// 
    codec: iris_types::codec::FeatureCodec,
    /// Optional IRIS runtime for IRIS-mode evolution.
    /// When set, mutation uses IRIS programs instead of Rust functions.
    iris_runtime: Option<std::sync::Arc<IrisRuntime>>,
    /// Message bus for inter-program communication within the deme.
    /// Programs can send/receive values through named channels.
    pub(crate) message_bus: MessageBus,
    /// Resource allocator for budget-based evaluation (Phase Transition 2).
    pub(crate) resource_allocator: Option<ResourceAllocator>,
    /// Fragment ecosystem tracker (Phase Transition 2).
    pub(crate) ecosystem: FragmentEcosystem,
    /// Mutation location tracker for stigmergic mutation bias.
    #[allow(dead_code)]
    pub(crate) mutation_tracker: MutationTracker,
    /// Per-generation operator application records: (operator_id, fitness_delta).
    /// Populated after evaluation by comparing offspring fitness to parent fitness.
    /// Cleared after each `step()` call; consumers should drain via
    /// `take_operator_records()` before the next step.
    pub(crate) operator_records: Vec<(u8, f64)>,
    /// Pending operator attributions: (offspring_fragment_id, operator_id, parent_fitness_sum).
    /// Set during `reproduce()`, resolved after `evaluate_all()` in the next step.
    pending_operator_attributions: Vec<(iris_types::fragment::FragmentId, u8, f32)>,
}

impl Deme {
    /// Create a new deme from seed fragments.
    pub fn initialize<F>(size: usize, seed_fn: F) -> Self
    where
        F: FnMut(usize) -> Fragment,
    {
        Self::initialize_with_novelty_k(size, seed_fn, 15)
    }

    /// Create a new deme with a specific novelty k-nearest parameter.
    pub fn initialize_with_novelty_k<F>(size: usize, mut seed_fn: F, novelty_k: usize) -> Self
    where
        F: FnMut(usize) -> Fragment,
    {
        let individuals = (0..size)
            .map(|i| Individual::new(seed_fn(i)))
            .collect();

        // Initialize the message bus with a default "deme" channel for
        // intra-deme communication during evolution.
        let mut bus = MessageBus::new();
        bus.create_channel("deme", 256);
        bus.create_channel("fitness_signal", 256);

        Self {
            individuals,
            generation: 0,
            phase: Phase::Exploration,
            phase_detector: PhaseDetector::new(20),
            novelty_archive: NoveltyArchive::new(novelty_k),
            stigmergic_field: StigmergicField::new(0.5),
            codec: iris_types::codec::FeatureCodec::new(),
            iris_runtime: None,
            message_bus: bus,
            resource_allocator: None,
            ecosystem: FragmentEcosystem::new(3),
            mutation_tracker: MutationTracker::default(),
            operator_records: Vec::new(),
            pending_operator_attributions: Vec::new(),
        }
    }

    /// Take and clear the collected operator records from the last generation.
    ///
    /// Returns `(operator_id, fitness_delta)` pairs for each mutation that was
    /// applied during `reproduce()`. Used by `ImprovementTracker` for causal
    /// attribution.
    pub fn take_operator_records(&mut self) -> Vec<(u8, f64)> {
        std::mem::take(&mut self.operator_records)
    }

    /// Set the IRIS runtime for IRIS-mode evolution.
    pub fn set_iris_runtime(&mut self, runtime: std::sync::Arc<IrisRuntime>) {
        self.iris_runtime = Some(runtime);
    }

    /// Enable resource competition with the given total budget in milliseconds.
    /// Top 25% of population gets 2x evaluation budget, bottom 25% gets 0.5x.
    pub fn enable_resource_competition(&mut self, total_budget_ms: u64) {
        self.resource_allocator = Some(ResourceAllocator::new(total_budget_ms));
    }

    /// Set the keystone threshold for fragment ecosystem tracking.
    pub fn set_keystone_threshold(&mut self, threshold: usize) {
        self.ecosystem = FragmentEcosystem::new(threshold);
    }

    /// Get a reference to the message bus.
    pub fn message_bus(&self) -> &MessageBus {
        &self.message_bus
    }

    /// Get a mutable reference to the message bus.
    pub fn message_bus_mut(&mut self) -> &mut MessageBus {
        &mut self.message_bus
    }

    /// Get a reference to the stigmergic field.
    pub fn stigmergic_field(&self) -> &StigmergicField {
        &self.stigmergic_field
    }

    /// Get a reference to the fragment ecosystem.
    pub fn ecosystem(&self) -> &FragmentEcosystem {
        &self.ecosystem
    }

    /// Get a reference to the resource allocator (if enabled).
    pub fn resource_allocator(&self) -> Option<&ResourceAllocator> {
        self.resource_allocator.as_ref()
    }

    /// Run one generation: evaluate, rank, select, reproduce.
    ///
    /// When ecology features are enabled:
    /// - Resource competition: top/bottom performers get different eval budgets
    /// - Stigmergic mutation bias: mutations are biased toward high-pheromone locations
    /// - Message bus: programs can communicate within the deme
    /// - Ecosystem tracking: keystone fragments are protected from death
    pub fn step(
        &mut self,
        exec: &dyn ExecutionService,
        test_cases: &[TestCase],
        config: &EvolutionConfig,
        rng: &mut impl Rng,
    ) {
        // 0. Resolve pending operator attributions from the previous
        //    generation's reproduce() call. Now that the offspring have been
        //    evaluated (they are in self.individuals), we can compute the
        //    fitness delta: offspring_fitness - parent_fitness.
        self.resolve_operator_attributions();

        // 1. Evaluate all individuals via ExecutionService (Tier A).
        //    With resource competition, top/bottom performers get different
        //    numbers of test cases evaluated.
        self.evaluate_with_resource_competition(exec, test_cases, config);

        // 1b. Re-resolve attributions now that this generation's offspring
        //     have been evaluated (covers the case where offspring from the
        //     immediately preceding reproduce() are in the current population).
        self.resolve_operator_attributions();

        // 2. NSGA-II ranking: non-dominated sort + crowding distance.
        self.rank();

        // 2b. Resource allocation: compute budgets based on Pareto rank.
        if let Some(ref mut alloc) = self.resource_allocator {
            alloc.allocate(&self.individuals);
        }

        // 3. Phase detection using the new PhaseDetector.
        self.phase = self.phase_detector.detect(&self.individuals, self.generation as u64);

        // 3b. Stigmergic field: deposit signals for all evaluated individuals.
        self.deposit_stigmergic_signals();

        // 3c. Decay stigmergic signals and prune negligible voxels.
        self.stigmergic_field.decay();
        if self.generation % 10 == 0 {
            self.stigmergic_field.prune(0.001);
        }

        // 3d. Message bus: broadcast top-performer fitness to the bus
        //     so programs can react to the fitness landscape.
        self.broadcast_fitness_signals();

        // 3e. Fragment ecosystem tracking every 5 generations.
        if self.generation % 5 == 0 {
            self.ecosystem.update(&self.individuals, self.generation as u64);
        }

        // 4. Reproduce: selection + crossover + mutation with phase-aware params.
        //    Mutation is biased by stigmergic field signals.
        let offspring = self.reproduce(config, rng);

        // 5. Elitism: preserve top elites from current population.
        // Scale with population size: at least 2, up to 5% of pop.
        let num_elites = (config.population_size / 20).max(2).min(16);
        let mut next_gen = self.select_elites(num_elites);

        // 5b. Preserve skeleton-protected individuals that aren't already
        // in the elite set (they may have low initial fitness but need
        // time to be properly evaluated).
        let current_gen = self.generation as u64;
        let elite_ids: Vec<_> = next_gen.iter().map(|i| i.fragment.id).collect();
        for ind in &self.individuals {
            if ind.skeleton_protected_until > current_gen
                && !elite_ids.contains(&ind.fragment.id)
            {
                next_gen.push(ind.clone());
            }
        }

        // 5c. Protect keystone fragments from being lost.
        //     If any individual carries a keystone fragment that isn't in the
        //     next generation, preserve that individual.
        let keystone_ids = self.ecosystem.keystones();
        if !keystone_ids.is_empty() {
            let next_gen_ids: std::collections::HashSet<_> =
                next_gen.iter().map(|i| i.fragment.id).collect();
            for ind in &self.individuals {
                // Check if this individual's fragment is a keystone.
                if keystone_ids.contains(&ind.fragment.id)
                    && !next_gen_ids.contains(&ind.fragment.id)
                {
                    next_gen.push(ind.clone());
                }
                // Also check if it imports a keystone and is the only carrier.
                for &import_id in &ind.fragment.imports {
                    if keystone_ids.contains(&import_id)
                        && !next_gen.iter().any(|ng| ng.fragment.imports.contains(&import_id))
                    {
                        next_gen.push(ind.clone());
                        break;
                    }
                }
            }
        }

        next_gen.extend(offspring);

        // Trim to population size, but never remove protected individuals or keystones.
        if next_gen.len() > config.population_size {
            // Partition: protected individuals stay, others may be trimmed.
            let (mut protected, mut unprotected): (Vec<_>, Vec<_>) = next_gen
                .into_iter()
                .partition(|ind| {
                    ind.skeleton_protected_until > current_gen
                        || keystone_ids.contains(&ind.fragment.id)
                });

            // Sort unprotected by fitness (descending) to keep the best.
            unprotected.sort_by(|a, b| {
                let sa: f32 = b.fitness.values.iter().sum();
                let sb: f32 = a.fitness.values.iter().sum();
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            });

            let remaining_slots = config.population_size.saturating_sub(protected.len());
            unprotected.truncate(remaining_slots);
            protected.extend(unprotected);
            next_gen = protected;
        }

        self.individuals = next_gen;
        self.generation += 1;
    }

    /// Evaluate individuals with resource competition.
    ///
    /// When a resource allocator is active, individuals get different numbers
    /// of test cases based on their fitness rank from the previous generation:
    /// - Top 25%: evaluated on ALL test cases (2x base = all)
    /// - Middle 50%: evaluated on a random subset (base = ~66%)
    /// - Bottom 25%: evaluated on a smaller subset (0.5x base = ~33%)
    ///
    /// This creates selection pressure beyond pure fitness: better programs
    /// get richer fitness signals, which helps them improve faster.
    fn evaluate_with_resource_competition(
        &mut self,
        exec: &dyn ExecutionService,
        test_cases: &[TestCase],
        config: &EvolutionConfig,
    ) {
        if self.resource_allocator.is_none() || test_cases.len() <= 2 {
            // No resource competition: evaluate all on all test cases.
            self.evaluate_all(exec, test_cases, config);
            return;
        }

        // Sort individuals by pareto_rank to determine tiers.
        let n = self.individuals.len();
        if n == 0 {
            return;
        }

        let mut ranked_indices: Vec<(usize, usize)> = self
            .individuals
            .iter()
            .enumerate()
            .map(|(i, ind)| (i, ind.pareto_rank))
            .collect();
        ranked_indices.sort_by_key(|&(_, rank)| rank);

        // Compute the number of test cases for each tier.
        let total_tests = test_cases.len();
        let full_budget = total_tests; // top 25%
        let base_budget = (total_tests * 2 + 2) / 3; // middle 50% (~66%)
        let half_budget = (total_tests + 2) / 3; // bottom 25% (~33%, at least 1)
        let half_budget = half_budget.max(1);

        // Assign test case subsets per individual based on position.
        let mut per_individual_tests: Vec<Vec<TestCase>> = vec![vec![]; n];

        for (position, &(idx, _rank)) in ranked_indices.iter().enumerate() {
            let rank_fraction = position as f64 / n as f64;
            let num_tests = if rank_fraction < 0.25 {
                full_budget
            } else if rank_fraction > 0.75 {
                half_budget
            } else {
                base_budget
            };

            // For reduced budgets, select a deterministic subset based on
            // individual index so it's reproducible.
            if num_tests >= total_tests {
                per_individual_tests[idx] = test_cases.to_vec();
            } else {
                // Use a stride-based selection for reproducibility.
                let stride = total_tests as f64 / num_tests as f64;
                let mut subset = Vec::with_capacity(num_tests);
                for j in 0..num_tests {
                    let tc_idx = ((j as f64 * stride) as usize).min(total_tests - 1);
                    subset.push(test_cases[tc_idx].clone());
                }
                per_individual_tests[idx] = subset;
            }
        }

        // Track which test case indices each individual was evaluated on,
        // so we can correctly populate per_case_scores for lexicase selection.
        let mut per_individual_tc_indices: Vec<Vec<usize>> = vec![vec![]; n];
        for (position, &(idx, _rank)) in ranked_indices.iter().enumerate() {
            let rank_fraction = position as f64 / n as f64;
            let num_tests = if rank_fraction < 0.25 {
                full_budget
            } else if rank_fraction > 0.75 {
                half_budget
            } else {
                base_budget
            };
            if num_tests >= total_tests {
                per_individual_tc_indices[idx] = (0..total_tests).collect();
            } else {
                let stride = total_tests as f64 / num_tests as f64;
                per_individual_tc_indices[idx] = (0..num_tests)
                    .map(|j| ((j as f64 * stride) as usize).min(total_tests - 1))
                    .collect();
            }
        }

        // Evaluate each individual on its assigned subset.
        let mut behaviors: Vec<Option<crate::novelty::BehaviorDescriptor>> =
            vec![None; n];

        for (i, ind) in self.individuals.iter_mut().enumerate() {
            let ind_tests = &per_individual_tests[i];
            let tc_indices = &per_individual_tc_indices[i];
            match exec.evaluate_individual(
                &ind.fragment.graph,
                ind_tests,
                EvalTier::A,
            ) {
                Ok(result) => {
                    let (fitness, diagnoses) =
                        compute_fitness_with_diagnoses(&result, &ind.fragment.graph);
                    ind.fitness = fitness;
                    ind.proof_failure_count = diagnoses.len();
                    // Pad per_case_scores to full test case length.
                    // Place each score at the correct test case index;
                    // unevaluated test cases get 0.0.
                    let mut full_scores = vec![0.0f32; total_tests];
                    for (j, &tc_idx) in tc_indices.iter().enumerate() {
                        if j < result.per_case_scores.len() && tc_idx < total_tests {
                            full_scores[tc_idx] = result.per_case_scores[j];
                        }
                    }
                    ind.per_case_scores = full_scores;
                    let tier = verify::classify_verify_tier(&ind.fragment.graph);
                    ind.meta.verify_tier = verify::tier_to_u8(tier);
                    behaviors[i] =
                        Some(NoveltyArchive::behavior_from_results(&result.outputs));
                }
                Err(_) => {
                    ind.fitness = Fitness::ZERO;
                    ind.per_case_scores = vec![0.0; total_tests];
                }
            }
        }

        // Compute novelty scores and update fitness objective 4 (novelty).
        let novelty_threshold = config.novelty_threshold;
        let novelty_weight = config.novelty_weight;

        for (i, ind) in self.individuals.iter_mut().enumerate() {
            if let Some(behavior) = &behaviors[i] {
                let raw_novelty = self.novelty_archive.novelty_score(behavior);
                let f_novelty = (raw_novelty * novelty_weight).clamp(0.0, 1.0);
                ind.fitness.values[4] = f_novelty;
            }
        }

        for behavior in behaviors.into_iter().flatten() {
            self.novelty_archive.add(behavior, novelty_threshold);
        }
    }

    /// Broadcast fitness signals to the message bus.
    ///
    /// This enables inter-program awareness: programs in the deme can
    /// "sense" the fitness landscape through the bus. The top performer's
    /// fitness is broadcast, plus aggregate statistics.
    fn broadcast_fitness_signals(&mut self) {
        // Send the best fitness sum to the fitness_signal channel.
        if let Some(best) = self.best_individual() {
            let fitness_sum: f32 = best.fitness.values.iter().sum();
            // Drop errors (channel might be full, which is fine).
            let _ = self.message_bus.send(
                "fitness_signal",
                Value::Int((fitness_sum * 1000.0) as i64),
            );
        }

        // Send population size.
        let _ = self.message_bus.send(
            "deme",
            Value::Int(self.individuals.len() as i64),
        );
    }

    /// Evaluate all individuals, computing fitness from EvalResult.
    /// Integrates f_verify scoring (SPEC Section 4.9) and novelty search.
    fn evaluate_all(
        &mut self,
        exec: &dyn ExecutionService,
        test_cases: &[TestCase],
        config: &EvolutionConfig,
    ) {
        let graphs: Vec<_> = self
            .individuals
            .iter()
            .map(|ind| ind.fragment.graph.clone())
            .collect();

        let results = exec.evaluate_batch(&graphs, test_cases, EvalTier::A);

        // Collect behavior descriptors for novelty computation.
        let mut behaviors: Vec<Option<crate::novelty::BehaviorDescriptor>> =
            vec![None; self.individuals.len()];

        match results {
            Ok(eval_results) => {
                for (i, (ind, result)) in self
                    .individuals
                    .iter_mut()
                    .zip(eval_results.iter())
                    .enumerate()
                {
                    let (fitness, diagnoses) =
                        compute_fitness_with_diagnoses(result, &ind.fragment.graph);
                    ind.fitness = fitness;
                    ind.proof_failure_count = diagnoses.len();
                    // Store per-test-case scores for lexicase selection.
                    ind.per_case_scores = result.per_case_scores.clone();
                    // Update verify tier in metadata.
                    let tier = verify::classify_verify_tier(&ind.fragment.graph);
                    ind.meta.verify_tier = verify::tier_to_u8(tier);
                    // Compute behavior descriptor from outputs.
                    behaviors[i] =
                        Some(NoveltyArchive::behavior_from_results(&result.outputs));
                }
            }
            Err(_) => {
                // On batch failure, try individual evaluation.
                for (i, ind) in self.individuals.iter_mut().enumerate() {
                    match exec.evaluate_individual(
                        &ind.fragment.graph,
                        test_cases,
                        EvalTier::A,
                    ) {
                        Ok(result) => {
                            let (fitness, diagnoses) =
                                compute_fitness_with_diagnoses(&result, &ind.fragment.graph);
                            ind.fitness = fitness;
                            ind.proof_failure_count = diagnoses.len();
                            ind.per_case_scores = result.per_case_scores.clone();
                            let tier = verify::classify_verify_tier(&ind.fragment.graph);
                            ind.meta.verify_tier = verify::tier_to_u8(tier);
                            behaviors[i] =
                                Some(NoveltyArchive::behavior_from_results(&result.outputs));
                        }
                        Err(_) => {
                            ind.fitness = Fitness::ZERO;
                            ind.per_case_scores = vec![0.0; test_cases.len()];
                        }
                    }
                }
            }
        }

        // Compute novelty scores and update fitness objective 4 (novelty).
        let novelty_threshold = config.novelty_threshold;
        let novelty_weight = config.novelty_weight;

        for (i, ind) in self.individuals.iter_mut().enumerate() {
            if let Some(behavior) = &behaviors[i] {
                let raw_novelty = self.novelty_archive.novelty_score(behavior);
                // Normalize novelty to [0, 1] and apply weight.
                let f_novelty = (raw_novelty * novelty_weight).clamp(0.0, 1.0);
                ind.fitness.values[4] = f_novelty;
            }
        }

        // Add novel behaviors to the archive.
        for behavior in behaviors.into_iter().flatten() {
            self.novelty_archive.add(behavior, novelty_threshold);
        }
    }

    /// Perform NSGA-II ranking on the current population.
    fn rank(&mut self) {
        let fitnesses: Vec<Fitness> = self.individuals.iter().map(|i| i.fitness).collect();

        // Non-dominated sort.
        let fronts = nsga2::non_dominated_sort(&fitnesses);

        // Assign Pareto ranks and crowding distances.
        let mut ranks = vec![0usize; self.individuals.len()];
        let mut crowd = vec![0.0f32; self.individuals.len()];

        for (rank, front) in fronts.iter().enumerate() {
            let distances = nsga2::crowding_distance(front, &fitnesses);
            for (i, &idx) in front.iter().enumerate() {
                ranks[idx] = rank;
                crowd[idx] = distances[i];
            }
        }

        for (i, ind) in self.individuals.iter_mut().enumerate() {
            ind.pareto_rank = ranks[i];
            ind.crowding_distance = crowd[i];
            ind.meta.pareto_rank = ranks[i] as u16;
        }
    }

    /// Selection + crossover + mutation to produce offspring.
    ///
    /// Uses epsilon-lexicase selection for parent selection (preserves
    /// specialists on individual test cases) with phase-aware mutation
    /// and crossover rates.
    ///
    /// Stigmergic mutation bias: when the stigmergic field has signal,
    /// with 30% probability the mutation target node is biased toward
    /// high-pheromone locations (nodes whose embeddings are in high-fitness
    /// regions of the field).
    fn reproduce(&mut self, config: &EvolutionConfig, rng: &mut impl Rng) -> Vec<Individual> {
        let pop_size = config.population_size;
        let num_elites = (pop_size / 20).max(2).min(16);
        let target_offspring = pop_size.saturating_sub(num_elites); // account for elites

        // Get phase-specific parameters.
        let params = phase_params(self.phase);

        // Check if per-case scores are available for lexicase selection.
        let has_per_case = self
            .individuals
            .iter()
            .all(|i| !i.per_case_scores.is_empty());

        // Fallback to tournament if per-case scores not yet available
        // (e.g., generation 0 before first evaluation).
        let ranks: Vec<usize> = self.individuals.iter().map(|i| i.pareto_rank).collect();
        let crowds: Vec<f32> = self
            .individuals
            .iter()
            .map(|i| i.crowding_distance)
            .collect();

        // Precompute fitness sums for IRIS selection.
        let fitness_sums: Vec<f32> = self
            .individuals
            .iter()
            .map(|i| i.fitness.values.iter().sum())
            .collect();

        // Precompute stigmergic readings for all individuals for mutation bias.
        let stigmergic_readings: Vec<crate::stigmergy::SignalReading> = self
            .individuals
            .iter()
            .map(|ind| self.stigmergic_reading(ind))
            .collect();

        let mut offspring = Vec::with_capacity(target_offspring);
        // Track which mutation operator was used for the current offspring.
        // Set to Some((op_id, parent_idx)) when mutate_tracked is used,
        // cleared after each offspring is created.
        let mut current_op: Option<(u8, usize)> = None;

        for _ in 0..target_offspring {
            // --- Selection ---
            // Stigmergic selection bias: with 20% probability, prefer
            // parents from high-fitness stigmergic regions.
            let stigmergy_select = self.stigmergic_field.voxel_count() > 0
                && rng.r#gen::<f64>() < 0.20;
            let stigmergy_select = false;

            let parent_a_idx = if stigmergy_select {
                unreachable!()
            } else if let Some(ref iris_rt) = self.iris_runtime {
                iris_rt.tournament_select(&fitness_sums, params.tournament_size, rng)
            } else if has_per_case {
                lexicase::lexicase_select_downsampled(&self.individuals, rng, 0.15)
            } else {
                nsga2::tournament_select(&ranks, &crowds, params.tournament_size, rng)
            };

            // --- Crossover ---
            let child_graph = if rng.r#gen::<f64>() < params.crossover_rate {
                let parent_b_idx = if let Some(ref iris_rt) = self.iris_runtime {
                    iris_rt.tournament_select(&fitness_sums, params.tournament_size, rng)
                } else if has_per_case {
                    lexicase::lexicase_select_downsampled(&self.individuals, rng, 0.15)
                } else {
                    nsga2::tournament_select(&ranks, &crowds, params.tournament_size, rng)
                };

                // When IRIS runtime is available, try IRIS crossover first.
                // Falls back to Rust crossover on failure.
                if let Some(ref iris_rt) = self.iris_runtime {
                    match iris_rt.crossover(
                        &self.individuals[parent_a_idx].fragment.graph,
                        &self.individuals[parent_b_idx].fragment.graph,
                        rng,
                    ) {
                        Some((offspring_a, _offspring_b)) => offspring_a,
                        None => crossover::crossover(
                            &self.individuals[parent_a_idx].fragment.graph,
                            &self.individuals[parent_b_idx].fragment.graph,
                            rng,
                        ),
                    }
                } else {
                    crossover::crossover(
                        &self.individuals[parent_a_idx].fragment.graph,
                        &self.individuals[parent_b_idx].fragment.graph,
                        rng,
                    )
                }
            } else {
                // Clone parent.
                self.individuals[parent_a_idx].fragment.graph.clone()
            };

            // --- Mutation ---
            // When IRIS runtime is available (iris_mode), use IRIS programs
            // for mutation. Otherwise, use Rust functions.
            // With 15% probability, use proof-guided mutation if the parent
            // has proof failures — this exploits the graded verification
            // gradient to fix specific type errors.
            //
            // Stigmergic mutation bias: with 30% probability when
            // stigmergic signals exist, apply multiple mutations to the
            // graph, biased toward nodes in high-fitness regions.
            let final_graph = if rng.r#gen::<f64>() < params.mutation_rate {
                // Check if stigmergic bias should apply.
                let use_stigmergy = self.stigmergic_field.voxel_count() > 0
                    && rng.r#gen::<f64>() < 0.30;
                let use_stigmergy = false;

                if use_stigmergy {
                    unreachable!()
                } else if let Some(ref iris_rt) = self.iris_runtime {
                    // IRIS mode: use IRIS programs for mutation.
                    let parent = &self.individuals[parent_a_idx];
                    if parent.proof_failure_count > 0 && rng.r#gen::<f64>() < 0.15 {
                        let (_score, diagnoses) =
                            verify::compute_f_verify_graded(&child_graph);
                        if !diagnoses.is_empty() {
                            mutation::proof_guided_mutation(&child_graph, &diagnoses, rng)
                        } else {
                            iris_rt.mutate(&child_graph, rng)
                        }
                    } else {
                        iris_rt.mutate(&child_graph, rng)
                    }
                } else {
                    // Standard Rust mode — use mutate_tracked for causal attribution.
                    let parent = &self.individuals[parent_a_idx];
                    if parent.proof_failure_count > 0 && rng.r#gen::<f64>() < 0.15 {
                        let (_score, diagnoses) =
                            verify::compute_f_verify_graded(&child_graph);
                        if !diagnoses.is_empty() {
                            mutation::proof_guided_mutation(&child_graph, &diagnoses, rng)
                        } else {
                            let (graph, op_id) = mutation::mutate_tracked(&child_graph, rng);
                            current_op = Some((op_id, parent_a_idx));
                            graph
                        }
                    } else {
                        let (graph, op_id) = mutation::mutate_tracked(&child_graph, rng);
                        current_op = Some((op_id, parent_a_idx));
                        graph
                    }
                }
            } else {
                child_graph
            };

            // Wrap in a new Fragment.
            let fragment = graph_to_fragment(final_graph, self.generation as u64);
            let ind = Individual::new(fragment);
            // If this offspring was produced by a tracked mutation operator,
            // record the pending attribution for later resolution.
            if let Some((op_id, parent_idx)) = current_op.take() {
                let parent_fitness_sum: f32 =
                    self.individuals[parent_idx].fitness.values.iter().sum();
                self.pending_operator_attributions.push((
                    ind.fragment.id,
                    op_id,
                    parent_fitness_sum,
                ));
            }
            offspring.push(ind);
        }

        offspring
    }

    /// Resolve pending operator attributions after offspring have been evaluated.
    ///
    /// Looks up each pending offspring by fragment ID in the current population,
    /// computes the fitness delta (offspring - parent), and records it as an
    /// operator record for the ImprovementTracker.
    fn resolve_operator_attributions(&mut self) {
        if self.pending_operator_attributions.is_empty() {
            return;
        }

        // Build a lookup from fragment ID to fitness sum for current population.
        let fitness_by_id: std::collections::HashMap<iris_types::fragment::FragmentId, f32> = self
            .individuals
            .iter()
            .map(|ind| {
                let sum: f32 = ind.fitness.values.iter().sum();
                (ind.fragment.id, sum)
            })
            .collect();

        // Resolve each pending attribution.
        let pending = std::mem::take(&mut self.pending_operator_attributions);
        for (frag_id, op_id, parent_fitness_sum) in pending {
            if let Some(&offspring_fitness_sum) = fitness_by_id.get(&frag_id) {
                // Fitness delta: positive means improvement.
                let delta = (offspring_fitness_sum - parent_fitness_sum) as f64;
                self.operator_records.push((op_id, delta));
            }
            // If the offspring isn't in the current population (it was culled
            // or trimmed), we skip it — that mutation didn't survive selection,
            // which is itself informative (no improvement recorded).
        }
    }

    /// Select a parent biased by stigmergic signal quality.
    ///
    /// Higher avg_fitness in the stigmergic field at an individual's
    /// embedding location makes it more likely to be selected.
    ///
    /// 
    fn stigmergy_biased_select(
        &self,
        readings: &[crate::stigmergy::SignalReading],
        fitness_sums: &[f32],
        rng: &mut impl Rng,
    ) -> usize {
        if self.individuals.is_empty() {
            return 0;
        }

        // Compute selection weights: blend fitness sum with stigmergic reading.
        // Weight = fitness_sum * (1 + stigmergic_avg_fitness) * (1 - failure_rate)
        let weights: Vec<f64> = self
            .individuals
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let fs = fitness_sums[i] as f64;
                let reading = &readings[i];
                let stigmergic_bonus = 1.0 + reading.avg_fitness as f64;
                let failure_penalty = 1.0 - reading.failure_rate as f64;
                (fs * stigmergic_bonus * failure_penalty).max(0.001) // floor to avoid zero weight
            })
            .collect();

        let total: f64 = weights.iter().sum();
        if total <= 0.0 {
            return rng.gen_range(0..self.individuals.len());
        }

        // Roulette wheel selection.
        let mut pick = rng.r#gen::<f64>() * total;
        for (i, &w) in weights.iter().enumerate() {
            pick -= w;
            if pick <= 0.0 {
                return i;
            }
        }

        self.individuals.len() - 1
    }

    /// Apply stigmergic-biased mutation.
    ///
    /// Mutates the graph, checks if the result lands in a better region
    /// of the stigmergic field, and if not, tries again (up to 3 attempts).
    /// This biases mutation toward changes that move the program's embedding
    /// toward high-fitness, low-failure regions.
    ///
    /// 
    fn stigmergy_biased_mutate(
        &self,
        graph: &iris_types::graph::SemanticGraph,
        parent_reading: &crate::stigmergy::SignalReading,
        rng: &mut impl Rng,
    ) -> iris_types::graph::SemanticGraph {
        use iris_types::codec::GraphEmbeddingCodec;

        let parent_quality = parent_reading.avg_fitness - parent_reading.failure_rate * 0.5;

        let mut best_mutant = mutation::mutate(graph, rng);
        let best_quality = parent_quality;

        for _attempt in 0..3 {
            let candidate = mutation::mutate(graph, rng);
            let embedding = self.codec.encode(&candidate);
            let reading = self.stigmergic_field.read(&embedding.dims);
            let quality = reading.avg_fitness - reading.failure_rate * 0.5;

            if quality > best_quality {
                best_mutant = candidate;
                // Found an improvement — use it.
                break;
            }
        }

        best_mutant
    }

    /// Select top-k individuals by Pareto rank + crowding distance (elites).
    fn select_elites(&self, k: usize) -> Vec<Individual> {
        let mut sorted: Vec<usize> = (0..self.individuals.len()).collect();
        sorted.sort_by(|&a, &b| {
            let rank_cmp = self.individuals[a]
                .pareto_rank
                .cmp(&self.individuals[b].pareto_rank);
            if rank_cmp != std::cmp::Ordering::Equal {
                return rank_cmp;
            }
            // Higher crowding distance is better.
            self.individuals[b]
                .crowding_distance
                .partial_cmp(&self.individuals[a].crowding_distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        sorted
            .into_iter()
            .take(k)
            .map(|i| self.individuals[i].clone())
            .collect()
    }

    /// Deposit stigmergic signals for all individuals in the current population.
    ///
    /// Each individual's graph is encoded via the feature codec, and its
    /// fitness (sum of objectives) is deposited into the stigmergic field.
    /// Failed individuals (zero correctness) deposit failure signals.
    ///
    /// 
    fn deposit_stigmergic_signals(&mut self) {
        use iris_types::codec::GraphEmbeddingCodec;

        let generation = self.generation as u64;
        for ind in &self.individuals {
            let embedding = self.codec.encode(&ind.fragment.graph);
            let fitness_sum: f32 = ind.fitness.values.iter().sum();
            let failed = ind.fitness.correctness() <= 0.0;
            self.stigmergic_field.deposit(&embedding.dims, fitness_sum, failed, generation);
        }
    }

    /// Compute the stigmergic gradient at an individual's embedding location.
    ///
    /// Returns a direction vector in embedding space pointing toward
    /// high-fitness regions and away from high-failure regions.
    ///
    /// 
    pub fn stigmergic_gradient(&self, ind: &Individual) -> Vec<f32> {
        use iris_types::codec::GraphEmbeddingCodec;

        let embedding = self.codec.encode(&ind.fragment.graph);
        self.stigmergic_field.gradient(&embedding.dims)
    }

    /// Read the stigmergic signal at an individual's embedding location.
    ///
    /// 
    pub fn stigmergic_reading(&self, ind: &Individual) -> crate::stigmergy::SignalReading {
        use iris_types::codec::GraphEmbeddingCodec;

        let embedding = self.codec.encode(&ind.fragment.graph);
        self.stigmergic_field.read(&embedding.dims)
    }

    /// Extract the current Pareto front (rank 0 individuals).
    pub fn pareto_front(&self) -> Vec<&Individual> {
        self.individuals
            .iter()
            .filter(|i| i.pareto_rank == 0)
            .collect()
    }

    /// Best individual by sum of fitness components.
    pub fn best_individual(&self) -> Option<&Individual> {
        self.individuals
            .iter()
            .max_by(|a, b| {
                let sa: f32 = a.fitness.values.iter().sum();
                let sb: f32 = b.fitness.values.iter().sum();
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Use component-based mutation instead of hardcoded operators.
    ///
    /// If the `ComponentRegistry` has mutation components registered, one
    /// is selected at random and applied via the `BridgeRegistry`. If no
    /// components are registered, falls back to the standard Rust
    /// `mutation::mutate` function.
    pub fn mutate_with_components(
        &self,
        graph: &iris_types::graph::SemanticGraph,
        components: &iris_types::component::ComponentRegistry,
        bridge: &crate::component_bridge::BridgeRegistry,
        rng: &mut rand::rngs::StdRng,
    ) -> iris_types::graph::SemanticGraph {
        if components.mutations.is_empty() {
            // Fallback: no IRIS components registered, use Rust operators.
            return mutation::mutate(graph, rng);
        }

        // Select a random mutation component.
        let idx = rng.gen_range(0..components.mutations.len());
        let component = &components.mutations[idx];

        // Apply via bridge; fall back to Rust on error.
        match bridge.apply_mutation(component, graph, rng) {
            Ok(mutated) => mutated,
            Err(_) => mutation::mutate(graph, rng),
        }
    }

    /// Extract counterexample-driven test cases from the current population.
    ///
    /// Runs graded verification on each individual that has proof failures,
    /// extracts counterexamples from the diagnoses, and converts them into
    /// new test cases. These test cases target the specific edge cases that
    /// break programs (CDGP pattern: counterexample-driven genetic programming).
    ///
    /// Returns a deduplicated set of new test cases (up to `max_cases`).
    pub fn extract_counterexample_test_cases(&self, max_cases: usize) -> Vec<TestCase> {
        let mut new_cases = Vec::new();

        for ind in &self.individuals {
            if ind.proof_failure_count == 0 {
                continue;
            }

            // Run graded verification to get diagnoses with counterexamples.
            let (_score, diagnoses) = verify::compute_f_verify_graded(&ind.fragment.graph);

            for diag in &diagnoses {
                if new_cases.len() >= max_cases {
                    break;
                }

                if let Some(test_case) =
                    mutation::counterexample_to_test_case(diag, &ind.fragment.graph)
                {
                    // Deduplicate: skip if we already have a test case with the same inputs.
                    let already_exists = new_cases
                        .iter()
                        .any(|existing: &TestCase| existing.inputs == test_case.inputs);
                    if !already_exists {
                        new_cases.push(test_case);
                    }
                }
            }

            if new_cases.len() >= max_cases {
                break;
            }
        }

        new_cases
    }
}

// ---------------------------------------------------------------------------
// Fitness computation
// ---------------------------------------------------------------------------

/// Compute fitness from an EvalResult, using graded proof-credit for f_verify.
///
/// Returns the Fitness and optionally the proof failure diagnoses (for
/// proof-guided mutation).
fn compute_fitness(
    result: &iris_types::eval::EvalResult,
    graph: &iris_types::graph::SemanticGraph,
) -> Fitness {
    // f_correct: directly from correctness_score.
    let f_correct = result.correctness_score.clamp(0.0, 1.0);

    // f_perf: baseline_time / actual_time, clamped. Use 1ms as baseline.
    let baseline_ns = 1_000_000u64; // 1ms
    let f_perf = if result.wall_time_ns > 0 {
        (baseline_ns as f32 / result.wall_time_ns as f32).clamp(0.0, 1.0)
    } else {
        1.0
    };

    // f_verify: graded proof credit from the kernel type checker.
    // Falls back to the heuristic value if graded is lower (shouldn't
    // happen in practice, but ensures monotonicity during transition).
    let (f_verify_graded, _diagnoses) = verify::compute_f_verify_graded(graph);
    let f_verify_heuristic = verify::compute_f_verify(graph);
    let f_verify = f_verify_graded.max(f_verify_heuristic);

    // f_cost: inverse of compile time as a rough proxy.
    let f_cost = if result.compile_time_ns > 0 {
        (100_000.0 / result.compile_time_ns as f32).clamp(0.0, 1.0)
    } else {
        1.0
    };

    Fitness {
        values: [f_correct, f_perf, f_verify, f_cost, 0.0],
    }
}

/// Compute fitness with full graded verification, returning diagnoses
/// for proof-guided mutation.
fn compute_fitness_with_diagnoses(
    result: &iris_types::eval::EvalResult,
    graph: &iris_types::graph::SemanticGraph,
) -> (Fitness, Vec<verify::ProofFailureDiagnosis>) {
    let fitness = compute_fitness(result, graph);
    let (_score, diagnoses) = verify::compute_f_verify_graded(graph);
    (fitness, diagnoses)
}

// ---------------------------------------------------------------------------
// Fragment construction helper
// ---------------------------------------------------------------------------

/// Wrap a SemanticGraph into a Fragment.
fn graph_to_fragment(graph: iris_types::graph::SemanticGraph, generation: u64) -> iris_types::fragment::Fragment {
    use iris_types::fragment::{Boundary, FragmentMeta};
    use iris_types::hash::compute_fragment_id;

    let root = graph.root;
    let type_sig = graph
        .nodes
        .get(&root)
        .map(|n| n.type_sig)
        .unwrap_or(iris_types::types::TypeId(0));

    let boundary = Boundary {
        inputs: vec![],
        outputs: vec![(root, type_sig)],
    };
    let type_env = graph.type_env.clone();

    let mut fragment = iris_types::fragment::Fragment {
        id: iris_types::fragment::FragmentId([0; 32]),
        graph,
        boundary,
        type_env,
        imports: vec![],
        metadata: FragmentMeta {
            name: None,
            created_at: 0,
            generation,
            lineage_hash: 0,
        },
        proof: None,
        contracts: Default::default(),    };
    fragment.id = compute_fragment_id(&fragment);
    fragment
}
