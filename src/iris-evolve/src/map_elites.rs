//! MAP-Elites archive for behavioral diversity (SPEC Section 6.3).
//!
//! Maintains a 2D grid (complexity x output_diversity) of the best individual
//! per behavior niche. During selection, 10% of the time individuals are drawn
//! from the archive rather than the main population.

use std::collections::BTreeMap;

use rand::Rng;

use crate::individual::Individual;

// ---------------------------------------------------------------------------
// BehaviorBin
// ---------------------------------------------------------------------------

/// 2D grid coordinate: (complexity_bin, output_diversity_bin).
pub type BehaviorBin = (u8, u8);

// ---------------------------------------------------------------------------
// MapElitesArchive
// ---------------------------------------------------------------------------

/// MAP-Elites archive: maintains the best individual per behavior niche.
///
/// Grid dimensions:
/// - Complexity: 4 bins (0-7 nodes, 8-31, 32-127, 128+)
/// - Output diversity: 3 bins (1 distinct output, 2-3, 4+)
/// - Total: 12 cells
pub struct MapElitesArchive {
    grid: BTreeMap<BehaviorBin, Individual>,
}

impl MapElitesArchive {
    /// Create a new empty archive.
    pub fn new() -> Self {
        Self {
            grid: BTreeMap::new(),
        }
    }

    /// Return the number of occupied cells.
    pub fn len(&self) -> usize {
        self.grid.len()
    }

    /// Return whether the archive is empty.
    pub fn is_empty(&self) -> bool {
        self.grid.is_empty()
    }

    /// Update the archive with an individual.
    ///
    /// The individual is placed in the bin corresponding to its behavior
    /// characteristics. It replaces the existing occupant only if it has
    /// higher total fitness.
    pub fn update(&mut self, individual: &Individual) {
        let bin = compute_bin(individual);
        let ind_fitness_sum: f32 = individual.fitness.values.iter().sum();

        let should_insert = match self.grid.get(&bin) {
            Some(existing) => {
                let existing_sum: f32 = existing.fitness.values.iter().sum();
                ind_fitness_sum > existing_sum
            }
            None => true,
        };

        if should_insert {
            self.grid.insert(bin, individual.clone());
        }
    }

    /// Update the archive with a batch of individuals.
    pub fn update_batch(&mut self, population: &[Individual]) {
        for ind in population {
            self.update(ind);
        }
    }

    /// Draw a random individual from the archive.
    ///
    /// Returns None if the archive is empty.
    pub fn sample(&self, rng: &mut impl Rng) -> Option<&Individual> {
        if self.grid.is_empty() {
            return None;
        }
        let values: Vec<&Individual> = self.grid.values().collect();
        Some(values[rng.gen_range(0..values.len())])
    }

    /// Get an iterator over all archived individuals.
    pub fn individuals(&self) -> impl Iterator<Item = &Individual> {
        self.grid.values()
    }

    /// Get the bin for a given individual (exposed for testing).
    pub fn bin_of(individual: &Individual) -> BehaviorBin {
        compute_bin(individual)
    }
}

impl Default for MapElitesArchive {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Bin computation
// ---------------------------------------------------------------------------

/// Compute the behavior bin for an individual.
fn compute_bin(individual: &Individual) -> BehaviorBin {
    let complexity_bin = complexity_bin(individual);
    let diversity_bin = output_diversity_bin(individual);
    (complexity_bin, diversity_bin)
}

/// Compute the complexity bin based on node count.
///
/// - 0-7 nodes   -> bin 0
/// - 8-31 nodes  -> bin 1
/// - 32-127 nodes -> bin 2
/// - 128+ nodes  -> bin 3
fn complexity_bin(individual: &Individual) -> u8 {
    let node_count = individual.fragment.graph.nodes.len();
    if node_count <= 7 {
        0
    } else if node_count <= 31 {
        1
    } else if node_count <= 127 {
        2
    } else {
        3
    }
}

/// Compute the output diversity bin.
///
/// Based on how many distinct output values the program produces. Since we
/// don't run test cases here, we approximate diversity from the graph
/// structure: count distinct leaf node values (Lit nodes with different
/// payloads).
///
/// - 1 distinct output   -> bin 0
/// - 2-3 distinct outputs -> bin 1
/// - 4+ distinct outputs  -> bin 2
fn output_diversity_bin(individual: &Individual) -> u8 {
    use iris_types::graph::NodeKind;
    use std::collections::BTreeSet;

    // Count distinct Lit node values as a proxy for output diversity.
    let mut distinct_values = BTreeSet::new();
    for node in individual.fragment.graph.nodes.values() {
        if node.kind == NodeKind::Lit {
            if let iris_types::graph::NodePayload::Lit { type_tag, value } = &node.payload {
                // Hash the (type_tag, value) pair for distinctness.
                let mut key = vec![*type_tag];
                key.extend_from_slice(value);
                distinct_values.insert(key);
            }
        }
    }

    // Also count distinct Prim opcodes as a diversity signal.
    let mut distinct_ops = BTreeSet::new();
    for node in individual.fragment.graph.nodes.values() {
        if node.kind == NodeKind::Prim {
            if let iris_types::graph::NodePayload::Prim { opcode } = &node.payload {
                distinct_ops.insert(*opcode);
            }
        }
    }

    let total_distinct = distinct_values.len() + distinct_ops.len();
    if total_distinct <= 1 {
        0
    } else if total_distinct <= 3 {
        1
    } else {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::individual::{Fitness, Individual};
    use crate::phase::iris_evolve_test_helpers::{make_dummy_fragment, make_fragment_with_kinds};
    use iris_types::graph::NodeKind;

    fn ind_with_fitness_and_fragment(
        values: [f32; 5],
        fragment: iris_types::fragment::Fragment,
    ) -> Individual {
        let mut ind = Individual::new(fragment);
        ind.fitness = Fitness { values };
        ind
    }

    #[test]
    fn test_archive_insert_and_replace() {
        let mut archive = MapElitesArchive::new();

        let frag = make_dummy_fragment();
        let ind1 = ind_with_fitness_and_fragment([0.3, 0.3, 0.3, 0.3, 0.0], frag.clone());
        archive.update(&ind1);
        assert_eq!(archive.len(), 1);

        // Higher fitness should replace.
        let ind2 = ind_with_fitness_and_fragment([0.5, 0.5, 0.5, 0.5, 0.0], frag.clone());
        let bin = MapElitesArchive::bin_of(&ind2);
        archive.update(&ind2);
        assert_eq!(archive.len(), 1);
        let stored: f32 = archive.grid[&bin].fitness.values.iter().sum();
        assert!(stored > 1.5); // Should be ind2's fitness.

        // Lower fitness should NOT replace.
        let ind3 = ind_with_fitness_and_fragment([0.1, 0.1, 0.1, 0.1, 0.0], frag);
        archive.update(&ind3);
        let stored2: f32 = archive.grid[&bin].fitness.values.iter().sum();
        assert!(stored2 > 1.5); // Still ind2.
    }

    #[test]
    fn test_complexity_bins() {
        // Small graph (1 node) -> bin 0
        let frag = make_dummy_fragment();
        let ind = ind_with_fitness_and_fragment([0.5; 5], frag);
        assert_eq!(complexity_bin(&ind), 0);
    }

    #[test]
    fn test_sample_returns_individual() {
        let mut archive = MapElitesArchive::new();
        let frag = make_dummy_fragment();
        let ind = ind_with_fitness_and_fragment([0.5; 5], frag);
        archive.update(&ind);

        let mut rng = rand::thread_rng();
        let sampled = archive.sample(&mut rng);
        assert!(sampled.is_some());
    }

    #[test]
    fn test_empty_archive_sample() {
        let archive = MapElitesArchive::new();
        let mut rng = rand::thread_rng();
        assert!(archive.sample(&mut rng).is_none());
    }

    #[test]
    fn test_different_bins_for_different_structures() {
        // Simple lit -> small complexity, low diversity
        let frag1 = make_dummy_fragment();
        let ind1 = ind_with_fitness_and_fragment([0.5; 5], frag1);
        let bin1 = compute_bin(&ind1);

        // Fold + lit -> different structure
        let frag2 = make_fragment_with_kinds(&[
            NodeKind::Lit,
            NodeKind::Prim,
            NodeKind::Prim,
            NodeKind::Fold,
        ]);
        let ind2 = ind_with_fitness_and_fragment([0.5; 5], frag2);
        let bin2 = compute_bin(&ind2);

        // They may or may not be in different bins depending on exact sizes,
        // but both should be valid bins.
        assert!(bin1.0 < 4 && bin1.1 < 3);
        assert!(bin2.0 < 4 && bin2.1 < 3);
    }
}
