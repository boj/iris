//! Death and compression for population management (SPEC Section 6.9).
//!
//! Death conditions remove low-fitness, aged, or overcrowded individuals.
//! Compression eliminates dead code from individual graphs every 50 generations.

use std::collections::BTreeSet;

use crate::individual::Individual;
use crate::population::Phase;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum age by phase (in generations).
fn max_age_for_phase(phase: Phase) -> u64 {
    match phase {
        Phase::Exploration => 200,
        Phase::SteadyState => 100,
        Phase::Exploitation => 50,
    }
}

// ---------------------------------------------------------------------------
// Culling
// ---------------------------------------------------------------------------

/// Remove individuals that meet death conditions.
///
/// Death conditions (SPEC Section 6.9):
/// - Fitness < 0.01 on ALL 4 objectives: immediate death
/// - Age > MAX_AGE (phase-dependent): age death
/// - Crowding: if population exceeds target, kill most crowded (lowest
///   crowding distance)
/// - Elites (pareto_rank == 0) are exempt from age and crowding death
pub fn cull_population(
    population: &mut Vec<Individual>,
    phase: Phase,
    generation: u64,
    target_size: usize,
) {
    // 1. Immediate death: fitness < 0.01 on all 4 objectives.
    //    Skeleton-protected individuals are exempt.
    population.retain(|ind| {
        if ind.skeleton_protected_until > generation {
            return true; // skeleton-protected: exempt from immediate death
        }
        !ind.fitness.values.iter().all(|&v| v < 0.01)
    });

    // 2. Age death: remove non-elite individuals older than MAX_AGE.
    //    Skeleton-protected individuals are exempt.
    let max_age = max_age_for_phase(phase);
    population.retain(|ind| {
        if ind.pareto_rank == 0 || ind.skeleton_protected_until > generation {
            return true; // elites and protected skeletons exempt
        }
        let age = generation.saturating_sub(ind.meta.age as u64);
        age <= max_age
    });

    // 3. Crowding death: if population still exceeds target, kill lowest
    //    crowding distance individuals (non-elite only).
    if population.len() > target_size {
        // Sort non-elites by crowding distance (ascending = most crowded first).
        // Keep elites unconditionally.
        let mut elite_indices = Vec::new();
        let mut non_elite_indices = Vec::new();

        for (i, ind) in population.iter().enumerate() {
            if ind.pareto_rank == 0 || ind.skeleton_protected_until > generation {
                elite_indices.push(i);
            } else {
                non_elite_indices.push(i);
            }
        }

        // Sort non-elites by crowding distance ascending (kill lowest first).
        non_elite_indices.sort_by(|&a, &b| {
            population[a]
                .crowding_distance
                .partial_cmp(&population[b].crowding_distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Determine how many non-elites to keep.
        let to_keep = target_size.saturating_sub(elite_indices.len());
        let keep_non_elites = to_keep.min(non_elite_indices.len());

        // Collect indices to keep.
        let mut keep_set: BTreeSet<usize> = BTreeSet::new();
        for &i in &elite_indices {
            keep_set.insert(i);
        }
        // Keep the non-elites with the highest crowding distance.
        for &i in non_elite_indices.iter().rev().take(keep_non_elites) {
            keep_set.insert(i);
        }

        // Rebuild population with only the kept individuals.
        let mut kept = Vec::with_capacity(keep_set.len());
        for (i, ind) in population.drain(..).enumerate() {
            if keep_set.contains(&i) {
                kept.push(ind);
            }
        }
        *population = kept;
    }
}

// ---------------------------------------------------------------------------
// Compression
// ---------------------------------------------------------------------------

/// Compress population by running dead code elimination on each individual's
/// graph (every 50 generations per SPEC Section 6.9).
///
/// Returns the number of nodes removed across all individuals.
pub fn compress_population(population: &mut Vec<Individual>) -> usize {
    let mut total_removed = 0;
    for ind in population.iter_mut() {
        total_removed += compress_individual(ind);
    }
    total_removed
}

/// Run dead code elimination on a single individual's graph.
///
/// Removes nodes unreachable from the root via edges, then recalculates
/// the graph hash.
fn compress_individual(ind: &mut Individual) -> usize {
    let graph = &mut ind.fragment.graph;
    let root = graph.root;

    // BFS from root to find reachable nodes.
    let mut reachable = BTreeSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(root);
    reachable.insert(root);

    while let Some(node_id) = queue.pop_front() {
        for edge in &graph.edges {
            if edge.source == node_id && !reachable.contains(&edge.target) {
                if graph.nodes.contains_key(&edge.target) {
                    reachable.insert(edge.target);
                    queue.push_back(edge.target);
                }
            }
        }
    }

    let original_count = graph.nodes.len();

    // Remove unreachable nodes.
    graph.nodes.retain(|id, _| reachable.contains(id));

    // Remove edges involving unreachable nodes.
    graph
        .edges
        .retain(|e| reachable.contains(&e.source) && reachable.contains(&e.target));

    // Rehash the graph.
    rehash_graph(graph);

    original_count - graph.nodes.len()
}

/// Recompute the semantic hash of a graph.
fn rehash_graph(graph: &mut iris_types::graph::SemanticGraph) {
    let mut hasher = blake3::Hasher::new();
    for (nid, _) in &graph.nodes {
        hasher.update(&nid.0.to_le_bytes());
    }
    for edge in &graph.edges {
        hasher.update(&edge.source.0.to_le_bytes());
        hasher.update(&edge.target.0.to_le_bytes());
        hasher.update(&[edge.port, edge.label as u8]);
    }
    graph.hash = iris_types::hash::SemanticHash(*hasher.finalize().as_bytes());
}

/// Check whether it's time to run compression (every 50 generations).
pub fn should_compress(generation: u64) -> bool {
    generation > 0 && generation % 50 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::individual::{Fitness, Individual};
    use crate::phase::iris_evolve_test_helpers::make_dummy_fragment;

    fn ind_with_fitness(values: [f32; 5], rank: usize, crowding: f32) -> Individual {
        let fragment = make_dummy_fragment();
        let mut ind = Individual::new(fragment);
        ind.fitness = Fitness { values };
        ind.pareto_rank = rank;
        ind.crowding_distance = crowding;
        ind.meta.pareto_rank = rank as u16;
        ind
    }

    #[test]
    fn test_cull_low_fitness() {
        let mut pop = vec![
            ind_with_fitness([0.0, 0.0, 0.0, 0.0, 0.0], 1, 1.0),
            ind_with_fitness([0.5, 0.5, 0.5, 0.5, 0.0], 0, 1.0),
            ind_with_fitness([0.005, 0.005, 0.005, 0.005, 0.0], 2, 1.0),
        ];
        cull_population(&mut pop, Phase::Exploration, 10, 100);
        // First and third should be removed (all objectives < 0.01).
        assert_eq!(pop.len(), 1);
        assert_eq!(pop[0].pareto_rank, 0);
    }

    #[test]
    fn test_elite_exempt_from_age_death() {
        let mut pop = vec![
            ind_with_fitness([0.5, 0.5, 0.5, 0.5, 0.0], 0, 1.0), // elite
            ind_with_fitness([0.5, 0.5, 0.5, 0.5, 0.0], 1, 1.0), // non-elite
        ];
        // Set age to 0, generation to 300 (age = 300).
        // Exploitation MAX_AGE = 50, so non-elite should die.
        cull_population(&mut pop, Phase::Exploitation, 300, 100);
        assert_eq!(pop.len(), 1);
        assert_eq!(pop[0].pareto_rank, 0);
    }

    #[test]
    fn test_crowding_death() {
        let mut pop = vec![
            ind_with_fitness([0.5, 0.5, 0.5, 0.5, 0.0], 1, 0.1), // most crowded
            ind_with_fitness([0.5, 0.5, 0.5, 0.5, 0.0], 1, 0.5),
            ind_with_fitness([0.5, 0.5, 0.5, 0.5, 0.0], 1, 0.9), // least crowded
            ind_with_fitness([0.5, 0.5, 0.5, 0.5, 0.0], 0, 0.1), // elite
        ];
        cull_population(&mut pop, Phase::Exploration, 0, 2);
        // Should keep the elite + highest crowding non-elite.
        assert_eq!(pop.len(), 2);
    }
}
