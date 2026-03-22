//! Resource competition for evolutionary populations.
//!
//! Programs compete for a shared computational budget. Better performers
//! get more evaluation time; poor performers get starved. This creates
//! selection pressure beyond pure fitness scoring.
//!
//! Phase Transition 2: Individual -> Ecology.

use std::collections::HashMap;

use crate::individual::Individual;

// ---------------------------------------------------------------------------
// ResourceAllocator
// ---------------------------------------------------------------------------

/// Allocates computational budget to individuals based on fitness rank.
///
/// Top 25% of performers (by Pareto rank) get 2x the base budget.
/// Bottom 25% get 0.5x. Middle 50% get 1x. This creates implicit
/// selection pressure: better programs get more evaluation time, which
/// in turn gives them richer fitness signals.
pub struct ResourceAllocator {
    /// Total budget in milliseconds across all individuals.
    pub total_budget_ms: u64,
    /// Per-individual budget allocations (individual index -> ms).
    pub allocations: HashMap<usize, u64>,
}

impl ResourceAllocator {
    /// Create a new resource allocator with the given total budget.
    pub fn new(total_budget_ms: u64) -> Self {
        Self {
            total_budget_ms,
            allocations: HashMap::new(),
        }
    }

    /// Allocate budget proportional to fitness rank.
    ///
    /// Top 25% get 2x budget. Bottom 25% get 0.5x budget.
    /// Middle 50% get 1x budget.
    ///
    /// Ranks are sorted ascending (0 = best). The rank_fraction is
    /// normalized to [0, 1] so the allocation is independent of
    /// population size.
    pub fn allocate(&mut self, population: &[Individual]) {
        let n = population.len();
        if n == 0 {
            self.allocations.clear();
            return;
        }

        let base = self.base_budget(n);

        // Sort individuals by pareto_rank to compute rank fractions.
        let mut ranked: Vec<(usize, usize)> = population
            .iter()
            .enumerate()
            .map(|(i, ind)| (i, ind.pareto_rank))
            .collect();
        ranked.sort_by_key(|&(_, rank)| rank);

        self.allocations.clear();
        for (position, &(idx, _rank)) in ranked.iter().enumerate() {
            let rank_fraction = position as f64 / n as f64;
            let multiplier = if rank_fraction < 0.25 {
                2.0
            } else if rank_fraction > 0.75 {
                0.5
            } else {
                1.0
            };
            self.allocations
                .insert(idx, (base as f64 * multiplier) as u64);
        }
    }

    /// Base budget per individual (total / population size).
    pub fn base_budget(&self, pop_size: usize) -> u64 {
        if pop_size == 0 {
            return self.total_budget_ms;
        }
        self.total_budget_ms / pop_size as u64
    }

    /// Get the budget allocated to a specific individual.
    pub fn get_budget(&self, index: usize) -> u64 {
        self.allocations.get(&index).copied().unwrap_or(0)
    }

    /// Get the total allocated budget (should be close to total_budget_ms).
    pub fn total_allocated(&self) -> u64 {
        self.allocations.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::individual::Individual;
    use crate::phase::iris_evolve_test_helpers::make_dummy_fragment;

    fn ind_with_rank(rank: usize) -> Individual {
        let fragment = make_dummy_fragment();
        let mut ind = Individual::new(fragment);
        ind.pareto_rank = rank;
        ind
    }

    #[test]
    fn allocate_distributes_budget() {
        let mut allocator = ResourceAllocator::new(1000);

        // 4 individuals with ascending ranks.
        let pop = vec![
            ind_with_rank(0), // best, top 25%
            ind_with_rank(1), // middle
            ind_with_rank(2), // middle
            ind_with_rank(3), // worst, bottom 25%
        ];

        allocator.allocate(&pop);

        // Base budget = 1000 / 4 = 250ms.
        // Fractions: 0/4=0.0, 1/4=0.25, 2/4=0.5, 3/4=0.75
        // Top 25% (position 0, fraction 0.0 < 0.25): 250 * 2.0 = 500ms
        // Middle (position 1, fraction 0.25 — not < 0.25, not > 0.75): 250 * 1.0 = 250ms
        // Middle (position 2, fraction 0.50): 250 * 1.0 = 250ms
        // Boundary (position 3, fraction 0.75 — not > 0.75): 250 * 1.0 = 250ms
        assert_eq!(allocator.get_budget(0), 500);
        assert_eq!(allocator.get_budget(1), 250);
        assert_eq!(allocator.get_budget(2), 250);
        assert_eq!(allocator.get_budget(3), 250);
    }

    #[test]
    fn allocate_empty_population() {
        let mut allocator = ResourceAllocator::new(1000);
        allocator.allocate(&[]);
        assert!(allocator.allocations.is_empty());
    }

    #[test]
    fn allocate_single_individual() {
        let mut allocator = ResourceAllocator::new(1000);
        let pop = vec![ind_with_rank(0)];
        allocator.allocate(&pop);
        // Single individual is both top and bottom 25%.
        // position=0, fraction=0.0, top 25% -> 2.0x.
        assert_eq!(allocator.get_budget(0), 2000);
    }

    #[test]
    fn top_performers_get_more_budget() {
        let mut allocator = ResourceAllocator::new(10000);

        let pop: Vec<Individual> = (0..20)
            .map(|i| ind_with_rank(i))
            .collect();

        allocator.allocate(&pop);

        let base = 10000u64 / 20;

        // Top 25% (positions 0..5) get 2x base.
        for i in 0..5 {
            assert_eq!(allocator.get_budget(i), base * 2);
        }

        // Bottom 25% (positions 15..20) get 0.5x base.
        for i in 16..20 {
            let budget = allocator.get_budget(i);
            assert_eq!(budget, base / 2);
        }

        // Middle 50% (positions 5..15) get 1x base.
        for i in 5..=15 {
            let budget = allocator.get_budget(i);
            assert_eq!(budget, base);
        }
    }
}
