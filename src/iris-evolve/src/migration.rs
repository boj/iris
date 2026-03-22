//! Migration between demes (SPEC Section 6.8).
//!
//! Ring topology migration: every 5 generations, the top-2 individuals from
//! each deme migrate to the next deme in a ring, replacing the worst 2
//! individuals in the destination.

use crate::population::Deme;

// ---------------------------------------------------------------------------
// Ring migration
// ---------------------------------------------------------------------------

/// Perform ring migration: top-2 from each deme migrate to the next.
///
/// Every 5 generations, the top-2 individuals (by Pareto rank + crowding
/// distance) from deme[i] replace the worst-2 in deme[(i+1) % N].
pub fn migrate_ring(demes: &mut [Deme], generation: u64) {
    if demes.len() < 2 || !should_migrate(generation) {
        return;
    }

    let n = demes.len();

    // Collect migrants from each deme: top-2 by rank + crowding distance.
    let mut all_migrants = Vec::with_capacity(n);
    for deme in demes.iter() {
        let migrants = select_top_k(&deme.individuals, 2);
        all_migrants.push(migrants);
    }

    // For each deme, receive migrants from the previous deme.
    for i in 0..n {
        let source_idx = i; // source deme
        let dest_idx = (i + 1) % n; // destination deme

        let migrants = &all_migrants[source_idx];
        if migrants.is_empty() {
            continue;
        }

        // Replace the worst individuals in the destination deme.
        replace_worst(&mut demes[dest_idx].individuals, migrants);
    }
}

/// Check if migration should occur this generation (every 5 generations).
pub fn should_migrate(generation: u64) -> bool {
    generation > 0 && generation % 5 == 0
}

/// Select the top-k individuals by Pareto rank (lower is better), then
/// crowding distance (higher is better).
fn select_top_k(
    individuals: &[crate::individual::Individual],
    k: usize,
) -> Vec<crate::individual::Individual> {
    if individuals.is_empty() {
        return vec![];
    }

    let mut indices: Vec<usize> = (0..individuals.len()).collect();
    indices.sort_by(|&a, &b| {
        let rank_cmp = individuals[a].pareto_rank.cmp(&individuals[b].pareto_rank);
        if rank_cmp != std::cmp::Ordering::Equal {
            return rank_cmp;
        }
        // Higher crowding distance is better.
        individuals[b]
            .crowding_distance
            .partial_cmp(&individuals[a].crowding_distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    indices
        .into_iter()
        .take(k)
        .map(|i| individuals[i].clone())
        .collect()
}

/// Replace the worst individuals in a population with migrants.
///
/// Worst = highest Pareto rank, then lowest crowding distance.
fn replace_worst(
    population: &mut Vec<crate::individual::Individual>,
    migrants: &[crate::individual::Individual],
) {
    if population.is_empty() || migrants.is_empty() {
        return;
    }

    // Sort population by quality (worst first).
    let mut indices: Vec<usize> = (0..population.len()).collect();
    indices.sort_by(|&a, &b| {
        // Worst first: highest rank first.
        let rank_cmp = population[b].pareto_rank.cmp(&population[a].pareto_rank);
        if rank_cmp != std::cmp::Ordering::Equal {
            return rank_cmp;
        }
        // Then lowest crowding distance.
        population[a]
            .crowding_distance
            .partial_cmp(&population[b].crowding_distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Replace the worst `migrants.len()` individuals.
    let to_replace = migrants.len().min(population.len());
    for (mi, &worst_idx) in indices.iter().take(to_replace).enumerate() {
        if mi < migrants.len() {
            population[worst_idx] = migrants[mi].clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::individual::{Fitness, Individual};
    use crate::phase::iris_evolve_test_helpers::make_dummy_fragment;
    use crate::population::Phase;

    fn make_ind(rank: usize, crowding: f32, fitness_sum: f32) -> Individual {
        let fragment = make_dummy_fragment();
        let mut ind = Individual::new(fragment);
        ind.pareto_rank = rank;
        ind.crowding_distance = crowding;
        ind.fitness = Fitness {
            values: [fitness_sum / 4.0, fitness_sum / 4.0, fitness_sum / 4.0, fitness_sum / 4.0, 0.0],
        };
        ind.meta.pareto_rank = rank as u16;
        ind
    }

    #[test]
    fn test_should_migrate() {
        assert!(!should_migrate(0));
        assert!(!should_migrate(1));
        assert!(should_migrate(5));
        assert!(should_migrate(10));
        assert!(!should_migrate(7));
    }

    #[test]
    fn test_select_top_k() {
        let pop = vec![
            make_ind(2, 0.5, 0.4),
            make_ind(0, 0.9, 0.8),
            make_ind(0, 0.3, 0.7),
            make_ind(1, 0.5, 0.6),
        ];
        let top2 = select_top_k(&pop, 2);
        assert_eq!(top2.len(), 2);
        // Both should be rank 0.
        assert_eq!(top2[0].pareto_rank, 0);
        assert_eq!(top2[1].pareto_rank, 0);
        // First should have higher crowding distance.
        assert!(top2[0].crowding_distance >= top2[1].crowding_distance);
    }

    #[test]
    fn test_replace_worst() {
        let mut pop = vec![
            make_ind(0, 0.9, 0.8), // best
            make_ind(1, 0.5, 0.4), // mid
            make_ind(3, 0.1, 0.1), // worst
        ];
        let migrants = vec![make_ind(0, 0.8, 0.9)];

        replace_worst(&mut pop, &migrants);

        // The worst individual (rank 3) should be replaced.
        assert_eq!(pop.len(), 3);
        // No individual should have rank 3 anymore.
        assert!(!pop.iter().any(|i| i.pareto_rank == 3));
    }

    #[test]
    fn test_migrate_ring_two_demes() {
        let seed_fn = |_: usize| make_dummy_fragment();
        let mut deme_a = Deme::initialize(4, |i| seed_fn(i));
        let mut deme_b = Deme::initialize(4, |i| seed_fn(i));

        // Give deme_a some ranked individuals.
        for (i, ind) in deme_a.individuals.iter_mut().enumerate() {
            ind.pareto_rank = i;
            ind.crowding_distance = 1.0 - i as f32 * 0.2;
            ind.fitness = Fitness {
                values: [1.0 - i as f32 * 0.2, 1.0 - i as f32 * 0.2, 1.0 - i as f32 * 0.2, 1.0 - i as f32 * 0.2, 0.0],
            };
            ind.meta.pareto_rank = i as u16;
        }

        // Give deme_b worse individuals.
        for ind in deme_b.individuals.iter_mut() {
            ind.pareto_rank = 5;
            ind.crowding_distance = 0.1;
            ind.fitness = Fitness {
                values: [0.1, 0.1, 0.1, 0.1, 0.0],
            };
            ind.meta.pareto_rank = 5;
        }

        let mut demes = vec![deme_a, deme_b];
        migrate_ring(&mut demes, 5);

        // After migration, deme_b should have some good individuals from deme_a.
        let best_in_b = demes[1]
            .individuals
            .iter()
            .map(|i| i.fitness.values.iter().sum::<f32>())
            .fold(0.0f32, f32::max);
        assert!(best_in_b > 0.5, "Migration should bring good individuals to deme_b");
    }
}
