//! `FragmentRegistry` — a store of known fragments for cross-fragment
//! composition via `Ref` nodes.
//!
//! The interpreter resolves `Ref` nodes by looking up fragments in this
//! registry and evaluating their graphs with the Ref node's inputs as
//! arguments.

use std::collections::BTreeMap;

use iris_types::fragment::{Fragment, FragmentId};

// ---------------------------------------------------------------------------
// FragmentRegistry
// ---------------------------------------------------------------------------

/// Registry of named/known fragments available for `Ref` resolution.
///
/// Used by the interpreter to resolve cross-fragment calls. Fragments can
/// be builtin library functions or previously-evolved programs.
#[derive(Debug, Clone)]
pub struct FragmentRegistry {
    fragments: BTreeMap<FragmentId, Fragment>,
}

impl FragmentRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            fragments: BTreeMap::new(),
        }
    }

    /// Register a fragment in the registry.
    pub fn register(&mut self, fragment: Fragment) {
        self.fragments.insert(fragment.id, fragment);
    }

    /// Look up a fragment by its ID.
    pub fn get(&self, id: &FragmentId) -> Option<&Fragment> {
        self.fragments.get(id)
    }

    /// Return the number of registered fragments.
    pub fn len(&self) -> usize {
        self.fragments.len()
    }

    /// Return true if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty()
    }

    /// Return all registered fragment IDs.
    pub fn fragment_ids(&self) -> Vec<FragmentId> {
        self.fragments.keys().copied().collect()
    }

    /// Return an iterator over all registered fragments.
    pub fn fragments(&self) -> impl Iterator<Item = &Fragment> {
        self.fragments.values()
    }

    /// Remove a fragment from the registry by its ID.
    ///
    /// Returns the removed fragment if it existed, or `None` if it was
    /// not in the registry. Used by the fragment ecosystem to prune
    /// decaying fragments.
    pub fn remove(&mut self, id: &FragmentId) -> Option<Fragment> {
        self.fragments.remove(id)
    }

    /// Convert to a BTreeMap of FragmentId -> SemanticGraph for the bootstrap evaluator.
    pub fn to_graph_map(&self) -> BTreeMap<FragmentId, iris_types::graph::SemanticGraph> {
        self.fragments
            .iter()
            .map(|(id, frag)| (*id, frag.graph.clone()))
            .collect()
    }
}

impl Default for FragmentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
