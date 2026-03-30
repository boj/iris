//! Component-as-program interface for IRIS.
//!
//! Expresses mutation operators, seed generators, and fitness functions as
//! IRIS type signatures so they can eventually be evolved. This is the
//! foundation for autopoietic closure: the things that PRODUCE programs
//! become programs themselves.

use crate::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// ComponentKind
// ---------------------------------------------------------------------------

/// Discriminates the role a component plays in the evolution loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKind {
    /// Takes a Program, returns a modified Program.
    /// IRIS type: Arrow(Program, Program, cost_bound)
    Mutation,
    /// Takes an Int (random seed), returns a Program.
    /// IRIS type: Arrow(Int, Program, cost_bound)
    Seed,
    /// Takes a Program (+ test cases), returns fitness scores.
    /// IRIS type: Arrow(Tuple(Program, Tuple(TestCase...)), Tuple(Float...), cost_bound)
    Fitness,
}

// ---------------------------------------------------------------------------
// MutationComponent
// ---------------------------------------------------------------------------

/// A mutation operator expressed as an IRIS-addressable component.
///
/// Today the `program` field holds a placeholder `SemanticGraph` and the
/// actual work is done by a wrapped Rust function (see `component_bridge`).
/// Once IRIS can evolve its own operators, `program` will hold a real IRIS
/// program that is interpreted directly.
#[derive(Debug, Clone)]
pub struct MutationComponent {
    pub name: String,
    pub program: SemanticGraph,
}

// ---------------------------------------------------------------------------
// SeedComponent
// ---------------------------------------------------------------------------

/// A seed generator expressed as an IRIS-addressable component.
///
/// IRIS type: Arrow(Int, Program, cost_bound)
/// The Int input is a random seed; the output is a freshly generated program.
#[derive(Debug, Clone)]
pub struct SeedComponent {
    pub name: String,
    pub program: SemanticGraph,
}

// ---------------------------------------------------------------------------
// FitnessComponent
// ---------------------------------------------------------------------------

/// A fitness function expressed as an IRIS-addressable component.
///
/// IRIS type: Arrow(Tuple(Program, Tuple(TestCase...)), Tuple(Float...), cost_bound)
#[derive(Debug, Clone)]
pub struct FitnessComponent {
    pub name: String,
    pub program: SemanticGraph,
}

// ---------------------------------------------------------------------------
// ComponentRegistry
// ---------------------------------------------------------------------------

/// Registry of IRIS-as-program components.
///
/// The evolution loop queries the registry for available components. When
/// no IRIS-native components are registered, it falls back to the hardcoded
/// Rust implementations.
#[derive(Debug, Clone)]
pub struct ComponentRegistry {
    pub mutations: Vec<MutationComponent>,
    pub seeds: Vec<SeedComponent>,
    pub fitness: Vec<FitnessComponent>,
}

impl ComponentRegistry {
    /// Create an empty registry (no components registered).
    pub fn new() -> Self {
        Self {
            mutations: Vec::new(),
            seeds: Vec::new(),
            fitness: Vec::new(),
        }
    }

    /// Look up a mutation component by name.
    pub fn find_mutation(&self, name: &str) -> Option<&MutationComponent> {
        self.mutations.iter().find(|c| c.name == name)
    }

    /// Look up a seed component by name.
    pub fn find_seed(&self, name: &str) -> Option<&SeedComponent> {
        self.seeds.iter().find(|c| c.name == name)
    }

    /// Look up a fitness component by name.
    pub fn find_fitness(&self, name: &str) -> Option<&FitnessComponent> {
        self.fitness.iter().find(|c| c.name == name)
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
