//! `iris-evolve` — Layer 0 evolutionary substrate for IRIS.
//!
//! Breeds programs via multi-objective evolution (NSGA-II) operating on
//! `SemanticGraph` representations. Gen3 scope: multi-deme with migration,
//! MAP-Elites archive, phase detection, death/compression, f_verify scoring.
//!
//! Phase Transition 2 (Individual -> Ecology): coevolution of programs and
//! test cases, resource competition, and fragment ecosystem tracking.

pub mod analyzer;
pub mod attention;
pub mod auto_improve;
pub mod checkpoint;
pub mod coevolution;
pub mod component_bridge;
pub mod config;
pub mod corpus;
pub mod crossover;
pub mod death;
pub mod ecosystem;
pub mod enumerate;
pub mod evolve_seeds;
pub mod improvement_tracker;
pub mod individual;
pub mod instrumentation;
pub mod iris_runtime;
pub mod lexicase;
pub mod map_elites;
pub mod meta_evolver;
pub mod migration;
pub mod mutation;
pub mod novelty;
pub mod nsga2;
pub mod phase;
pub mod population;
pub mod resource;
pub mod result;
pub mod seed;
pub mod self_improve;
pub mod self_improving_daemon;
pub mod stigmergy;
pub mod verify;

use std::time::{Duration, Instant};

use rand::SeedableRng;
use rand::rngs::StdRng;

use iris_exec::ExecutionService;

use crate::config::{EvolutionConfig, ProblemSpec};
use crate::death::{compress_population, cull_population, should_compress};
use crate::individual::Fitness;
use crate::map_elites::MapElitesArchive;
use crate::migration::{migrate_ring, should_migrate};
use crate::population::Deme;
use crate::result::{EvolutionResult, GenerationSnapshot};
use crate::self_improve::ImprovementResult;

pub use crate::meta_evolver::IrisMetaEvolver;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run the evolutionary loop.
///
/// Evolves a population of programs to satisfy `spec`, evaluating each
/// generation via `exec`. Returns the best individual and Pareto front.
///
/// Launches bottom-up enumeration in a background thread for programs
/// under ~8 nodes. If enumeration finds a perfect solution before
/// evolution does, the enumerated solution is used.
///
/// If `config.num_demes > 1`, creates multiple demes with ring migration.
pub fn evolve(
    config: EvolutionConfig,
    spec: ProblemSpec,
    exec: &dyn ExecutionService,
) -> EvolutionResult {
    let start = Instant::now();
    let mut rng = StdRng::from_entropy();

    // Launch bottom-up enumeration in a background thread.
    // For small programs (<=8 nodes), exhaustive enumeration with type
    // pruning is faster and guaranteed to find solutions.
    let enum_test_cases = spec.test_cases.clone();
    let enum_handle = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || enumerate::enumerate_solution(&enum_test_cases, 6))
        .expect("failed to spawn enumeration thread");

    // Analyze test cases and generate ALL matching skeletons.
    let analyzer_skeletons = analyzer::build_all_skeletons(&spec.test_cases);

    // Pre-evaluate skeletons: if any is already perfect, return immediately.
    for skeleton in &analyzer_skeletons {
        if let Ok(result) = exec.evaluate_individual(
            &skeleton.graph,
            &spec.test_cases,
            iris_types::eval::EvalTier::A,
        ) {
            if result.correctness_score >= 0.999 {
                // Perfect skeleton — no evolution needed.
                let mut best = individual::Individual::new(skeleton.clone());
                best.fitness.values[0] = result.correctness_score.clamp(0.0, 1.0);
                return EvolutionResult {
                    best_individual: best,
                    pareto_front: vec![],
                    generations_run: 0,
                    total_time: start.elapsed(),
                    history: vec![],
                };
            }
        }
    }

    let num_demes = config.num_demes.max(1);
    let pop_size = config.population_size;
    let skeleton_protection_generations: u64 = 10;

    // Initialize demes.
    // Weight distribution: ~15% each for the 6 new composition seeds,
    // ~10% for the 3 original seeds (arithmetic, fold, identity).
    // Total: 6*15 + 3*10 = 120 -> use modulo 20 for fair distribution.
    let novelty_k = config.novelty_k;
    let skeletons_ref = &analyzer_skeletons;
    let mut demes: Vec<Deme> = (0..num_demes)
        .map(|_| {
            Deme::initialize_with_novelty_k(pop_size, |i| {
                // Inject ALL analyzer skeletons as the first N seeds.
                // Each skeleton gets multiple copies for robustness.
                if !skeletons_ref.is_empty() {
                    let num_skeletons = skeletons_ref.len();
                    let copies_per = 4usize.max(8 / num_skeletons);
                    let total_skeleton_slots = num_skeletons * copies_per;
                    if i < total_skeleton_slots {
                        return skeletons_ref[i % num_skeletons].clone();
                    }
                }
                // Check for custom seed strategy (installed by self-improvement).
                if let Some(seed_type) = seed::custom_seed_type(&mut rng) {
                    return seed::generate_seed_by_type(seed_type, &mut rng);
                }
                // Default hardcoded distribution.
                match i % 30 {
                    // Original seeds (~7% each)
                    0 | 1 => seed::random_arithmetic_program(&mut rng, 2, 2),
                    2 | 3 => seed::random_fold_program(&mut rng),
                    4 => seed::identity_program(),
                    // Existing higher-order seeds (~3% each)
                    5 => seed::random_map_program(&mut rng),
                    6 => seed::random_zip_fold_program(&mut rng),
                    // Composition seeds
                    7 | 8 | 9 => seed::random_map_fold_program(&mut rng),
                    10 | 11 => seed::random_filter_fold_program(&mut rng),
                    12 | 13 => seed::random_zip_map_fold_program(&mut rng),
                    14 | 15 => seed::random_comparison_fold_program(&mut rng),
                    16 | 17 => seed::random_stateful_fold_program(&mut rng),
                    18 | 19 => seed::random_conditional_fold_program(&mut rng),
                    // Map-comparison+fold seeds
                    20 | 21 => seed::random_map_cmp_fold_program(&mut rng),
                    // Iterative/unfold seeds (for Fibonacci, GCD, etc.)
                    22 | 23 | 24 => seed::random_iterate_program(&mut rng),
                    // Pairwise fold seeds (for is-sorted, pairwise diffs)
                    25 | 26 | 27 => seed::random_pairwise_fold_program(&mut rng),
                    // Additional map+cmp+fold for predicate coverage
                    _ => seed::random_map_cmp_fold_program(&mut rng),
                }
            }, novelty_k)
        })
        .collect();

    // Mark skeleton-seeded individuals as protected for N generations.
    for deme in &mut demes {
        if !skeletons_ref.is_empty() {
            let num_skeletons = skeletons_ref.len();
            let copies_per = 4usize.max(8 / num_skeletons);
            let total_skeleton_slots = num_skeletons * copies_per;
            let limit = total_skeleton_slots.min(deme.individuals.len());
            for ind in deme.individuals.iter_mut().take(limit) {
                ind.skeleton_protected_until = skeleton_protection_generations;
            }
        }
    }

    // IRIS mode: create IrisRuntime and wire into demes.
    if config.iris_mode {
        eprintln!("Running in IRIS mode \u{2014} mutation, selection, and evaluation powered by IRIS programs");
        let iris_rt = std::sync::Arc::new(iris_runtime::IrisRuntime::new());
        for deme in &mut demes {
            deme.set_iris_runtime(iris_rt.clone());
        }
    }

    // Resource competition: enable on all demes when configured.
    if config.resource_budget_ms > 0 {
        for deme in &mut demes {
            deme.enable_resource_competition(config.resource_budget_ms);
        }
    }

    // MAP-Elites archive (shared across demes).
    let mut archive = MapElitesArchive::new();

    let mut history = Vec::with_capacity(config.max_generations);

    // Main evolutionary loop.
    for generation in 0..config.max_generations {
        // Step each deme.
        for deme in &mut demes {
            deme.step(exec, &spec.test_cases, &config, &mut rng);

            // Death/cull after each step.
            cull_population(
                &mut deme.individuals,
                deme.phase,
                generation as u64,
                config.population_size,
            );

            // Compression every 50 generations.
            if should_compress(generation as u64) {
                compress_population(&mut deme.individuals);
            }

            // Update MAP-Elites archive.
            archive.update_batch(&deme.individuals);
        }

        // Migration between demes (if multi-deme).
        if num_demes > 1 && should_migrate(generation as u64) {
            migrate_ring(&mut demes, generation as u64);
        }

        // Record snapshot from the first deme (primary).
        let primary = &demes[0];
        let best_fitness = primary
            .best_individual()
            .map(|i| i.fitness)
            .unwrap_or(Fitness::ZERO);

        let avg_fitness = compute_avg_fitness(&primary.individuals);
        let front_size = primary.pareto_front().len();

        history.push(GenerationSnapshot {
            generation,
            best_fitness,
            avg_fitness,
            pareto_front_size: front_size,
            phase: primary.phase,
        });

        // Early termination: perfect correctness achieved in any deme.
        if demes.iter().any(|d| {
            d.best_individual()
                .map(|i| i.fitness.correctness() >= 1.0)
                .unwrap_or(false)
        }) {
            break;
        }
    }

    // Find the best individual across all demes.
    let mut best = demes
        .iter()
        .filter_map(|d| d.best_individual())
        .max_by(|a, b| {
            let sa: f32 = a.fitness.values.iter().sum();
            let sb: f32 = b.fitness.values.iter().sum();
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .unwrap_or_else(|| demes[0].individuals[0].clone());

    // Check if bottom-up enumeration found a solution.
    if let Ok(Some(enum_fragment)) = enum_handle.join() {
        // If evolution didn't achieve perfect correctness, use the
        // enumerated solution instead.
        if best.fitness.correctness() < 1.0 {
            best = individual::Individual::new(enum_fragment);
            best.fitness.values[0] = 1.0_f32.clamp(0.0, 1.0); // perfect correctness
        }
    }

    // Clamp all fitness values to [0.0, 1.0] before returning.
    for v in best.fitness.values.iter_mut() {
        *v = v.clamp(0.0, 1.0);
    }

    // Pareto front from primary deme.
    let pareto_front: Vec<_> = demes[0]
        .pareto_front()
        .into_iter()
        .cloned()
        .collect();

    EvolutionResult {
        best_individual: best,
        pareto_front,
        generations_run: demes[0].generation,
        total_time: start.elapsed(),
        history,
    }
}

/// Run the evolutionary loop with adaptive resource allocation.
///
/// Instead of fixed population size and generation count, this uses an
/// `AdaptiveEvolver` to dynamically allocate compute based on the fitness
/// improvement trajectory:
///
/// - **Improving problems** get larger populations and more generations.
/// - **Slowly improving problems** maintain current resources.
/// - **Stagnating problems** get diversity injection (30% random replacement).
/// - **Hopelessly stagnated problems** are abandoned early.
///
/// This is the "attention economy" — compute flows to where it does the
/// most good.
pub fn evolve_adaptive(
    config: EvolutionConfig,
    spec: ProblemSpec,
    exec: &dyn ExecutionService,
    budget: attention::AttentionBudget,
) -> EvolutionResult {
    let start = Instant::now();
    let mut rng = StdRng::from_entropy();

    // Launch bottom-up enumeration in a background thread.
    let enum_test_cases = spec.test_cases.clone();
    let enum_handle = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || enumerate::enumerate_solution(&enum_test_cases, 6))
        .expect("failed to spawn enumeration thread");

    // Analyze test cases and generate ALL matching skeletons.
    let adaptive_skeletons = analyzer::build_all_skeletons(&spec.test_cases);

    // Pre-evaluate skeletons: if any is already perfect, return immediately.
    for skeleton in &adaptive_skeletons {
        if let Ok(result) = exec.evaluate_individual(
            &skeleton.graph,
            &spec.test_cases,
            iris_types::eval::EvalTier::A,
        ) {
            if result.correctness_score >= 0.999 {
                let mut best = individual::Individual::new(skeleton.clone());
                best.fitness.values[0] = result.correctness_score.clamp(0.0, 1.0);
                return EvolutionResult {
                    best_individual: best,
                    pareto_front: vec![],
                    generations_run: 0,
                    total_time: start.elapsed(),
                    history: vec![],
                };
            }
        }
    }

    let mut adaptive = attention::AdaptiveEvolver::new(budget);
    let initial_pop_size = adaptive.current_population();
    let novelty_k = config.novelty_k;
    let adaptive_protection_gens: u64 = 10;

    // Initialize a single deme with the adaptive population size.
    let adaptive_skeletons_ref = &adaptive_skeletons;
    let mut deme = Deme::initialize_with_novelty_k(initial_pop_size, |i| {
        if !adaptive_skeletons_ref.is_empty() {
            let num_skeletons = adaptive_skeletons_ref.len();
            let copies_per = 4usize.max(8 / num_skeletons);
            let total = num_skeletons * copies_per;
            if i < total {
                return adaptive_skeletons_ref[i % num_skeletons].clone();
            }
        }
        if let Some(seed_type) = seed::custom_seed_type(&mut rng) {
            return seed::generate_seed_by_type(seed_type, &mut rng);
        }
        match i % 30 {
            0 | 1 => seed::random_arithmetic_program(&mut rng, 2, 2),
            2 | 3 => seed::random_fold_program(&mut rng),
            4 => seed::identity_program(),
            5 => seed::random_map_program(&mut rng),
            6 => seed::random_zip_fold_program(&mut rng),
            7 | 8 | 9 => seed::random_map_fold_program(&mut rng),
            10 | 11 => seed::random_filter_fold_program(&mut rng),
            12 | 13 => seed::random_zip_map_fold_program(&mut rng),
            14 | 15 => seed::random_comparison_fold_program(&mut rng),
            16 | 17 => seed::random_stateful_fold_program(&mut rng),
            18 | 19 => seed::random_conditional_fold_program(&mut rng),
            20 | 21 => seed::random_map_cmp_fold_program(&mut rng),
            22 | 23 | 24 => seed::random_iterate_program(&mut rng),
            25 | 26 | 27 => seed::random_pairwise_fold_program(&mut rng),
            _ => seed::random_map_cmp_fold_program(&mut rng),
        }
    }, novelty_k);

    // Mark skeleton-seeded individuals as protected.
    if !adaptive_skeletons_ref.is_empty() {
        let num_skeletons = adaptive_skeletons_ref.len();
        let copies_per = 4usize.max(8 / num_skeletons);
        let total = num_skeletons * copies_per;
        let limit = total.min(deme.individuals.len());
        for ind in deme.individuals.iter_mut().take(limit) {
            ind.skeleton_protected_until = adaptive_protection_gens;
        }
    }

    let mut archive = MapElitesArchive::new();
    let mut history = Vec::new();

    let mut generation = 0usize;
    loop {
        // Check if we've exceeded the adaptive generation budget.
        if generation >= adaptive.allocated_generations() {
            break;
        }

        // Check wall-clock timeout.
        if start.elapsed() >= Duration::from_millis(adaptive.budget().total_budget_ms) {
            break;
        }

        // Create a config snapshot with the current adaptive population size.
        let step_config = EvolutionConfig {
            population_size: adaptive.current_population(),
            ..config.clone()
        };

        // Step the deme.
        deme.step(exec, &spec.test_cases, &step_config, &mut rng);

        // Death/cull.
        cull_population(
            &mut deme.individuals,
            deme.phase,
            generation as u64,
            adaptive.current_population(),
        );

        // Compression every 50 generations.
        if should_compress(generation as u64) {
            compress_population(&mut deme.individuals);
        }

        // Update MAP-Elites archive.
        archive.update_batch(&deme.individuals);

        // Compute current best fitness (sum of objectives).
        let best_fitness = deme
            .best_individual()
            .map(|i| i.fitness)
            .unwrap_or(individual::Fitness::ZERO);

        let current_best_sum: f32 = best_fitness.values.iter().sum();

        // Record snapshot.
        let avg_fitness = compute_avg_fitness(&deme.individuals);
        let front_size = deme.pareto_front().len();

        history.push(GenerationSnapshot {
            generation,
            best_fitness,
            avg_fitness,
            pareto_front_size: front_size,
            phase: deme.phase,
        });

        // Early termination: perfect correctness.
        if best_fitness.correctness() >= 1.0 {
            break;
        }

        // Adaptive resource allocation decision.
        let decision = adaptive.allocate_next(current_best_sum);

        match decision {
            attention::ResourceDecision::Invest { .. } => {
                // Population growth is handled inside AdaptiveEvolver.
                // We may need to add more individuals to reach the new size.
                let target = adaptive.current_population();
                while deme.individuals.len() < target {
                    let new_fragment = seed::random_arithmetic_program(&mut rng, 2, 2);
                    deme.individuals.push(individual::Individual::new(new_fragment));
                }
            }
            attention::ResourceDecision::Maintain => {
                // Nothing to do — keep going.
            }
            attention::ResourceDecision::InjectDiversity { random_fraction } => {
                // Replace `random_fraction` of the population with fresh random individuals.
                let num_replace = (deme.individuals.len() as f32 * random_fraction) as usize;
                let num_replace = num_replace.max(1).min(deme.individuals.len());

                // Sort by fitness (ascending) and replace the worst individuals.
                deme.individuals.sort_by(|a, b| {
                    let sa: f32 = a.fitness.values.iter().sum();
                    let sb: f32 = b.fitness.values.iter().sum();
                    sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
                });

                for i in 0..num_replace {
                    let new_fragment = match i % 6 {
                        0 => seed::random_arithmetic_program(&mut rng, 2, 2),
                        1 => seed::random_fold_program(&mut rng),
                        2 => seed::random_map_fold_program(&mut rng),
                        3 => seed::random_iterate_program(&mut rng),
                        4 => seed::random_pairwise_fold_program(&mut rng),
                        _ => seed::random_map_cmp_fold_program(&mut rng),
                    };
                    deme.individuals[i] = individual::Individual::new(new_fragment);
                }
            }
            attention::ResourceDecision::Abandon => {
                // Problem is hopelessly stagnated — stop early.
                break;
            }
        }

        generation += 1;
    }

    // Find the best individual.
    let mut best = deme
        .best_individual()
        .cloned()
        .unwrap_or_else(|| deme.individuals[0].clone());

    // Check if bottom-up enumeration found a solution.
    if let Ok(Some(enum_fragment)) = enum_handle.join() {
        if best.fitness.correctness() < 1.0 {
            best = individual::Individual::new(enum_fragment);
            best.fitness.values[0] = 1.0;
        }
    }

    let pareto_front: Vec<_> = deme
        .pareto_front()
        .into_iter()
        .cloned()
        .collect();

    EvolutionResult {
        best_individual: best,
        pareto_front,
        generations_run: generation,
        total_time: start.elapsed(),
        history,
    }
}

/// Run the evolutionary loop with coevolution and ecological dynamics.
///
/// When `config.coevolution` is enabled, this uses the `CoevolutionEngine`
/// to co-evolve test cases alongside programs. Resource allocation is
/// applied when `config.resource_budget_ms > 0`. Fragment ecosystem
/// tracking is always active.
///
/// Falls back to standard `evolve()` when coevolution is disabled.
pub fn evolve_with_ecology(
    config: EvolutionConfig,
    spec: ProblemSpec,
    exec: &dyn ExecutionService,
) -> EvolutionResult {
    if !config.coevolution {
        return evolve(config, spec, exec);
    }

    let start = Instant::now();
    let mut rng = StdRng::from_entropy();

    // Analyze test cases and generate ALL matching skeletons.
    let eco_skeletons = analyzer::build_all_skeletons(&spec.test_cases);

    // Pre-evaluate skeletons: if any is already perfect, return immediately.
    for skeleton in &eco_skeletons {
        if let Ok(result) = exec.evaluate_individual(
            &skeleton.graph,
            &spec.test_cases,
            iris_types::eval::EvalTier::A,
        ) {
            if result.correctness_score >= 0.999 {
                let mut best = individual::Individual::new(skeleton.clone());
                best.fitness.values[0] = result.correctness_score.clamp(0.0, 1.0);
                return EvolutionResult {
                    best_individual: best,
                    pareto_front: vec![],
                    generations_run: 0,
                    total_time: start.elapsed(),
                    history: vec![],
                };
            }
        }
    }

    let pop_size = config.population_size;
    let novelty_k = config.novelty_k;
    let eco_protection_gens: u64 = 10;

    // Initialize a single deme for coevolution.
    let eco_skeletons_ref = &eco_skeletons;
    let mut deme = Deme::initialize_with_novelty_k(pop_size, |i| {
        if !eco_skeletons_ref.is_empty() {
            let num_skeletons = eco_skeletons_ref.len();
            let copies_per = 4usize.max(8 / num_skeletons);
            let total = num_skeletons * copies_per;
            if i < total {
                return eco_skeletons_ref[i % num_skeletons].clone();
            }
        }
        if let Some(seed_type) = seed::custom_seed_type(&mut rng) {
            return seed::generate_seed_by_type(seed_type, &mut rng);
        }
        match i % 30 {
            0 | 1 => seed::random_arithmetic_program(&mut rng, 2, 2),
            2 | 3 => seed::random_fold_program(&mut rng),
            4 => seed::identity_program(),
            5 => seed::random_map_program(&mut rng),
            6 => seed::random_zip_fold_program(&mut rng),
            7 | 8 | 9 => seed::random_map_fold_program(&mut rng),
            10 | 11 => seed::random_filter_fold_program(&mut rng),
            12 | 13 => seed::random_zip_map_fold_program(&mut rng),
            14 | 15 => seed::random_comparison_fold_program(&mut rng),
            16 | 17 => seed::random_stateful_fold_program(&mut rng),
            18 | 19 => seed::random_conditional_fold_program(&mut rng),
            20 | 21 => seed::random_map_cmp_fold_program(&mut rng),
            22 | 23 | 24 => seed::random_iterate_program(&mut rng),
            25 | 26 | 27 => seed::random_pairwise_fold_program(&mut rng),
            _ => seed::random_map_cmp_fold_program(&mut rng),
        }
    }, novelty_k);

    // Mark skeleton-seeded individuals as protected.
    if !eco_skeletons_ref.is_empty() {
        let num_skeletons = eco_skeletons_ref.len();
        let copies_per = 4usize.max(8 / num_skeletons);
        let total = num_skeletons * copies_per;
        let limit = total.min(deme.individuals.len());
        for ind in deme.individuals.iter_mut().take(limit) {
            ind.skeleton_protected_until = eco_protection_gens;
        }
    }

    // Enable resource competition on the deme if configured.
    if config.resource_budget_ms > 0 {
        deme.enable_resource_competition(config.resource_budget_ms);
    }

    // Initialize coevolution engine with problem's test cases as seeds.
    let mut coevo = coevolution::CoevolutionEngine::new(deme, spec.test_cases.clone());

    // Fragment ecosystem tracker (used at the evolve_with_ecology level
    // for keystone-aware culling, separate from the per-deme tracker).
    let mut fragment_eco = ecosystem::FragmentEcosystem::new(3);

    // Fragment registry for ecosystem pruning.
    let mut frag_registry = iris_exec::registry::FragmentRegistry::new();

    let mut history = Vec::with_capacity(config.max_generations);

    for generation in 0..config.max_generations {
        // Coevolutionary step (this internally does evaluate + rank +
        // stigmergic deposit/decay + message bus broadcast + reproduce).
        coevo.step(exec, &config, &mut rng);

        // Fragment ecosystem tracking every 5 generations.
        if generation % 5 == 0 {
            fragment_eco.update(&coevo.program_deme.individuals, generation as u64);
        }

        // Death/cull with keystone protection.
        // Get keystones before culling so we can protect them.
        let keystones: std::collections::HashSet<iris_types::fragment::FragmentId> =
            fragment_eco.keystones().into_iter().collect();

        // Protect keystone individuals from cull by temporarily setting
        // pareto_rank = 0 (elite) so cull_population won't remove them.
        let mut keystone_originals: Vec<(usize, usize)> = Vec::new();
        for (i, ind) in coevo.program_deme.individuals.iter_mut().enumerate() {
            if keystones.contains(&ind.fragment.id) && ind.pareto_rank != 0 {
                keystone_originals.push((i, ind.pareto_rank));
                ind.pareto_rank = 0;
            }
        }

        cull_population(
            &mut coevo.program_deme.individuals,
            coevo.program_deme.phase,
            generation as u64,
            config.population_size,
        );

        // Restore original pareto ranks after cull (for those that survived).
        for (i, original_rank) in keystone_originals {
            if i < coevo.program_deme.individuals.len() {
                coevo.program_deme.individuals[i].pareto_rank = original_rank;
            }
        }

        // Compression every 50 generations.
        if should_compress(generation as u64) {
            compress_population(&mut coevo.program_deme.individuals);
        }

        // Ecosystem pruning every 50 generations: remove decaying non-keystone fragments.
        if generation > 0 && generation % 50 == 0 {
            fragment_eco.prune(&mut frag_registry, 30, generation as u64);
        }

        // Record snapshot.
        let best_fitness = coevo.program_deme
            .best_individual()
            .map(|i| i.fitness)
            .unwrap_or(Fitness::ZERO);

        let avg_fitness = compute_avg_fitness(&coevo.program_deme.individuals);
        let front_size = coevo.program_deme.pareto_front().len();

        history.push(GenerationSnapshot {
            generation,
            best_fitness,
            avg_fitness,
            pareto_front_size: front_size,
            phase: coevo.program_deme.phase,
        });

        // Early termination.
        if coevo.best_program_fitness() >= 1.0 {
            break;
        }
    }

    let best = coevo.program_deme
        .best_individual()
        .cloned()
        .unwrap_or_else(|| coevo.program_deme.individuals[0].clone());

    let pareto_front: Vec<_> = coevo.program_deme
        .pareto_front()
        .into_iter()
        .cloned()
        .collect();

    EvolutionResult {
        best_individual: best,
        pareto_front,
        generations_run: coevo.generation,
        total_time: start.elapsed(),
        history,
    }
}

/// Run the evolutionary loop with a wall-clock timeout.
///
/// Identical to `evolve()` but additionally checks elapsed time after each
/// generation and terminates early if the timeout is exceeded. Used by
/// meta-evolution to keep sub-evolution responsive.
pub fn evolve_with_timeout(
    config: EvolutionConfig,
    spec: ProblemSpec,
    exec: &dyn ExecutionService,
    timeout: std::time::Duration,
) -> EvolutionResult {
    let start = Instant::now();
    let mut rng = StdRng::from_entropy();

    // Launch enumeration in background.
    let enum_test_cases = spec.test_cases.clone();
    let enum_handle = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || enumerate::enumerate_solution(&enum_test_cases, 6))
        .expect("failed to spawn enumeration thread");

    // Analyze test cases and generate ALL matching skeletons.
    let timeout_skeletons = analyzer::build_all_skeletons(&spec.test_cases);

    // Pre-evaluate skeletons: if any is already perfect, return immediately.
    for skeleton in &timeout_skeletons {
        if let Ok(result) = exec.evaluate_individual(
            &skeleton.graph,
            &spec.test_cases,
            iris_types::eval::EvalTier::A,
        ) {
            if result.correctness_score >= 0.999 {
                let mut best = individual::Individual::new(skeleton.clone());
                best.fitness.values[0] = result.correctness_score.clamp(0.0, 1.0);
                return EvolutionResult {
                    best_individual: best,
                    pareto_front: vec![],
                    generations_run: 0,
                    total_time: start.elapsed(),
                    history: vec![],
                };
            }
        }
    }

    let num_demes = config.num_demes.max(1);
    let pop_size = config.population_size;
    let timeout_protection_gens: u64 = 10;

    let novelty_k = config.novelty_k;
    let timeout_skeletons_ref = &timeout_skeletons;
    let mut demes: Vec<Deme> = (0..num_demes)
        .map(|_| {
            Deme::initialize_with_novelty_k(pop_size, |i| {
                // Inject ALL analyzer skeletons as the first N seeds.
                if !timeout_skeletons_ref.is_empty() {
                    let num_skeletons = timeout_skeletons_ref.len();
                    let copies_per = 4usize.max(8 / num_skeletons);
                    let total = num_skeletons * copies_per;
                    if i < total {
                        return timeout_skeletons_ref[i % num_skeletons].clone();
                    }
                }
                // Check for custom seed strategy (installed by self-improvement).
                if let Some(seed_type) = seed::custom_seed_type(&mut rng) {
                    return seed::generate_seed_by_type(seed_type, &mut rng);
                }
                // Default hardcoded distribution.
                match i % 30 {
                    0 | 1 => seed::random_arithmetic_program(&mut rng, 2, 2),
                    2 | 3 => seed::random_fold_program(&mut rng),
                    4 => seed::identity_program(),
                    5 => seed::random_map_program(&mut rng),
                    6 => seed::random_zip_fold_program(&mut rng),
                    7 | 8 | 9 => seed::random_map_fold_program(&mut rng),
                    10 | 11 => seed::random_filter_fold_program(&mut rng),
                    12 | 13 => seed::random_zip_map_fold_program(&mut rng),
                    14 | 15 => seed::random_comparison_fold_program(&mut rng),
                    16 | 17 => seed::random_stateful_fold_program(&mut rng),
                    18 | 19 => seed::random_conditional_fold_program(&mut rng),
                    20 | 21 => seed::random_map_cmp_fold_program(&mut rng),
                    22 | 23 | 24 => seed::random_iterate_program(&mut rng),
                    25 | 26 | 27 => seed::random_pairwise_fold_program(&mut rng),
                    _ => seed::random_map_cmp_fold_program(&mut rng),
                }
            }, novelty_k)
        })
        .collect();

    // Mark skeleton-seeded individuals as protected.
    for deme in &mut demes {
        if !timeout_skeletons_ref.is_empty() {
            let num_skeletons = timeout_skeletons_ref.len();
            let copies_per = 4usize.max(8 / num_skeletons);
            let total = num_skeletons * copies_per;
            let limit = total.min(deme.individuals.len());
            for ind in deme.individuals.iter_mut().take(limit) {
                ind.skeleton_protected_until = timeout_protection_gens;
            }
        }
    }

    let mut archive = MapElitesArchive::new();
    let mut history = Vec::with_capacity(config.max_generations);

    for generation in 0..config.max_generations {
        for deme in &mut demes {
            deme.step(exec, &spec.test_cases, &config, &mut rng);

            cull_population(
                &mut deme.individuals,
                deme.phase,
                generation as u64,
                config.population_size,
            );

            if should_compress(generation as u64) {
                compress_population(&mut deme.individuals);
            }

            archive.update_batch(&deme.individuals);
        }

        if num_demes > 1 && should_migrate(generation as u64) {
            migrate_ring(&mut demes, generation as u64);
        }

        let primary = &demes[0];
        let best_fitness = primary
            .best_individual()
            .map(|i| i.fitness)
            .unwrap_or(Fitness::ZERO);

        let avg_fitness = compute_avg_fitness(&primary.individuals);
        let front_size = primary.pareto_front().len();

        history.push(GenerationSnapshot {
            generation,
            best_fitness,
            avg_fitness,
            pareto_front_size: front_size,
            phase: primary.phase,
        });

        // Early termination: perfect correctness achieved.
        if demes.iter().any(|d| {
            d.best_individual()
                .map(|i| i.fitness.correctness() >= 1.0)
                .unwrap_or(false)
        }) {
            break;
        }

        // Wall-clock timeout.
        if start.elapsed() >= timeout {
            break;
        }
    }

    let mut best = demes
        .iter()
        .filter_map(|d| d.best_individual())
        .max_by(|a, b| {
            let sa: f32 = a.fitness.values.iter().sum();
            let sb: f32 = b.fitness.values.iter().sum();
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .unwrap_or_else(|| demes[0].individuals[0].clone());

    // Check if enumeration found a solution.
    if let Ok(Some(enum_fragment)) = enum_handle.join() {
        if best.fitness.correctness() < 1.0 {
            best = individual::Individual::new(enum_fragment);
            best.fitness.values[0] = 1.0;
        }
    }

    let pareto_front: Vec<_> = demes[0]
        .pareto_front()
        .into_iter()
        .cloned()
        .collect();

    EvolutionResult {
        best_individual: best,
        pareto_front,
        generations_run: demes[0].generation,
        total_time: start.elapsed(),
        history,
    }
}

/// Compute average fitness across a population.
fn compute_avg_fitness(individuals: &[individual::Individual]) -> Fitness {
    if individuals.is_empty() {
        return Fitness::ZERO;
    }

    let n = individuals.len() as f32;
    let mut sum = [0.0f32; individual::NUM_OBJECTIVES];
    for ind in individuals {
        for (i, &v) in ind.fitness.values.iter().enumerate() {
            sum[i] += v;
        }
    }

    let mut values = [0.0f32; individual::NUM_OBJECTIVES];
    for i in 0..individual::NUM_OBJECTIVES {
        values[i] = sum[i] / n;
    }

    Fitness { values }
}

// ---------------------------------------------------------------------------
// Self-improvement API
// ---------------------------------------------------------------------------

/// Run self-improvement: evolve better evolution parameters.
///
/// This is IRIS improving its own ability to evolve programs. It runs
/// meta-evolution on mutation operator weights and seed generator distributions,
/// evaluating each candidate strategy by its effectiveness at solving the
/// given benchmark problems.
///
/// # Arguments
/// - `problems`: Benchmark problems used to evaluate strategy quality.
/// - `exec`: Execution service for evaluating evolved programs.
/// - `rounds`: Number of self-improvement rounds to run.
///
/// # Returns
/// A `Vec<ImprovementResult>`, one per round, showing the baseline and
/// improved solve rates along with the evolved strategies.
///
/// Each round uses small populations (8) and few generations (5) for strategy
/// evaluation, and tiny sub-evolution runs (population 8, 10 generations) to
/// keep the whole process fast.
pub fn self_improve_loop(
    problems: Vec<ProblemSpec>,
    exec: &dyn ExecutionService,
    rounds: usize,
) -> Vec<ImprovementResult> {
    self_improve::self_improve_loop(
        problems,
        exec,
        rounds,
        Duration::from_secs(30),   // budget per round
        5,                         // meta-evolution generations
        8,                         // meta-evolution population
        10,                        // sub-evolution generations per eval
        8,                         // sub-evolution population per eval
    )
}
pub mod performance_gate;
