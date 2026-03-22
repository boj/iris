//! Fragment ecosystem tracking.
//!
//! Tracks which program fragments reference others. Frequently-referenced
//! fragments are "keystone species" in the ecosystem. Unused fragments
//! decay and are eventually pruned.
//!
//! Phase Transition 2: Individual -> Ecology.

use std::collections::HashMap;

use iris_exec::registry::FragmentRegistry;
use iris_types::fragment::FragmentId;

use crate::individual::Individual;

// ---------------------------------------------------------------------------
// FragmentEcosystem
// ---------------------------------------------------------------------------

/// Tracks the ecological relationships between fragments in the population.
///
/// Fragments that are referenced by many programs are "keystone species" —
/// removing them would disrupt the ecosystem. Fragments that haven't been
/// used for many generations are "decaying" and candidates for pruning.
pub struct FragmentEcosystem {
    /// How many programs reference each fragment (by FragmentId).
    pub reference_counts: HashMap<FragmentId, usize>,
    /// Generation when each fragment was last referenced.
    pub last_used: HashMap<FragmentId, u64>,
    /// Minimum reference count to be considered a "keystone" fragment.
    pub keystone_threshold: usize,
}

impl FragmentEcosystem {
    /// Create a new fragment ecosystem with the given keystone threshold.
    pub fn new(keystone_threshold: usize) -> Self {
        Self {
            reference_counts: HashMap::new(),
            last_used: HashMap::new(),
            keystone_threshold,
        }
    }

    /// Update reference counts from the current population.
    ///
    /// Scans each individual's fragment imports and counts how many programs
    /// reference each fragment ID. Also updates the `last_used` generation
    /// timestamp for any referenced fragment.
    pub fn update(&mut self, population: &[Individual], generation: u64) {
        // Reset reference counts for this generation.
        for count in self.reference_counts.values_mut() {
            *count = 0;
        }

        for ind in population {
            // Count the individual's own fragment as "existing".
            let fid = ind.fragment.id;
            *self.reference_counts.entry(fid).or_insert(0) += 0;
            // Don't update last_used just for existing — only for being referenced.

            // Count imports (references to other fragments).
            for &import_id in &ind.fragment.imports {
                *self.reference_counts.entry(import_id).or_insert(0) += 1;
                self.last_used.insert(import_id, generation);
            }
        }
    }

    /// Get keystone fragments (reference count >= keystone_threshold).
    pub fn keystones(&self) -> Vec<FragmentId> {
        self.reference_counts
            .iter()
            .filter(|&(_, &count)| count >= self.keystone_threshold)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Get decaying fragments (not used in `max_age` generations).
    pub fn decaying(&self, max_age: u64, current_gen: u64) -> Vec<FragmentId> {
        self.last_used
            .iter()
            .filter(|&(_, &last)| current_gen.saturating_sub(last) > max_age)
            .map(|(&id, _)| id)
            .collect()
    }

    /// Prune decaying fragments from both the ecosystem tracking and
    /// the fragment registry.
    ///
    /// Removes fragments that haven't been referenced in `max_age`
    /// generations and are not keystones.
    pub fn prune(&mut self, registry: &mut FragmentRegistry, max_age: u64, current_gen: u64) {
        let decaying = self.decaying(max_age, current_gen);
        let keystones: std::collections::HashSet<FragmentId> =
            self.keystones().into_iter().collect();

        let to_remove: Vec<FragmentId> = decaying
            .into_iter()
            .filter(|id| !keystones.contains(id))
            .collect();

        for id in &to_remove {
            self.reference_counts.remove(id);
            self.last_used.remove(id);
            registry.remove(id);
        }
    }

    /// Get the reference count for a specific fragment.
    pub fn reference_count(&self, id: &FragmentId) -> usize {
        self.reference_counts.get(id).copied().unwrap_or(0)
    }

    /// Get the number of tracked fragments.
    pub fn num_tracked(&self) -> usize {
        self.reference_counts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::individual::Individual;
    use crate::phase::iris_evolve_test_helpers::make_dummy_fragment;
    use iris_types::fragment::FragmentId;

    fn make_ind_with_imports(imports: Vec<FragmentId>) -> Individual {
        let mut fragment = make_dummy_fragment();
        fragment.imports = imports;
        Individual::new(fragment)
    }

    #[test]
    fn update_counts_imports() {
        let mut eco = FragmentEcosystem::new(2);
        let shared_id = FragmentId([1; 32]);

        let pop = vec![
            make_ind_with_imports(vec![shared_id]),
            make_ind_with_imports(vec![shared_id]),
            make_ind_with_imports(vec![shared_id]),
            make_ind_with_imports(vec![]),
        ];

        eco.update(&pop, 0);
        assert_eq!(eco.reference_count(&shared_id), 3);
    }

    #[test]
    fn keystones_detected() {
        let mut eco = FragmentEcosystem::new(2);
        let popular_id = FragmentId([1; 32]);
        let rare_id = FragmentId([2; 32]);

        let pop = vec![
            make_ind_with_imports(vec![popular_id, rare_id]),
            make_ind_with_imports(vec![popular_id]),
            make_ind_with_imports(vec![popular_id]),
        ];

        eco.update(&pop, 0);

        let keystones = eco.keystones();
        assert!(keystones.contains(&popular_id));
        assert!(!keystones.contains(&rare_id));
    }

    #[test]
    fn decaying_detected() {
        let mut eco = FragmentEcosystem::new(2);
        let old_id = FragmentId([1; 32]);
        let new_id = FragmentId([2; 32]);

        // Old fragment was last used at generation 0.
        eco.last_used.insert(old_id, 0);
        // New fragment was last used at generation 100.
        eco.last_used.insert(new_id, 100);

        let decaying = eco.decaying(50, 105);
        assert!(decaying.contains(&old_id));
        assert!(!decaying.contains(&new_id));
    }

    #[test]
    fn prune_removes_non_keystone_decaying() {
        let mut eco = FragmentEcosystem::new(2);
        let mut registry = FragmentRegistry::new();

        let decaying_id = FragmentId([1; 32]);
        let keystone_id = FragmentId([2; 32]);

        // Set up: decaying_id is old and not keystone.
        eco.last_used.insert(decaying_id, 0);
        eco.reference_counts.insert(decaying_id, 0);

        // keystone_id is old but heavily referenced -> keystone.
        eco.last_used.insert(keystone_id, 0);
        eco.reference_counts.insert(keystone_id, 5);

        eco.prune(&mut registry, 50, 100);

        // decaying_id should be pruned.
        assert!(!eco.reference_counts.contains_key(&decaying_id));
        // keystone_id should be preserved.
        assert!(eco.reference_counts.contains_key(&keystone_id));
    }
}
