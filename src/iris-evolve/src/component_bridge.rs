//! Bridge between Rust mutation/seed functions and IRIS component interface.
//!
//! Wraps existing Rust functions as IRIS-callable components. This is the
//! transitional layer: today the components delegate to Rust code; once IRIS
//! can evolve its own operators, the `program` field will hold a real IRIS
//! program and execution will go through the interpreter instead.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use rand::rngs::StdRng;

use iris_types::component::{MutationComponent, SeedComponent};
use iris_types::cost::CostBound;
use iris_types::fragment::Fragment;
use iris_types::graph::{Resolution, SemanticGraph};
use iris_types::hash::SemanticHash;
use iris_types::types::TypeEnv;

// ---------------------------------------------------------------------------
// ComponentError
// ---------------------------------------------------------------------------

/// Error produced when applying a component fails.
#[derive(Debug, Clone)]
pub enum ComponentError {
    /// The input graph was empty or malformed.
    InvalidInput(String),
    /// The wrapped Rust function panicked or produced an invalid result.
    ExecutionFailed(String),
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "component invalid input: {}", msg),
            Self::ExecutionFailed(msg) => write!(f, "component execution failed: {}", msg),
        }
    }
}

impl std::error::Error for ComponentError {}

// ---------------------------------------------------------------------------
// Wrapped Rust function storage
// ---------------------------------------------------------------------------

/// Type-erased wrapper around a Rust mutation function.
///
/// We cannot store `fn` pointers inside `SemanticGraph`, so we maintain a
/// separate side-table keyed by component name. The `SemanticGraph` in the
/// component is a placeholder; actual dispatch goes through this table.
///
/// Thread-safety: the registry is built once at startup and read-only
/// during evolution, so no synchronization is needed.
struct MutationFnEntry {
    func: fn(&SemanticGraph, &mut StdRng) -> SemanticGraph,
}

struct SeedFnEntry {
    func: fn(&mut StdRng) -> Fragment,
}

/// Side-table mapping component names to their Rust implementations.
pub struct BridgeRegistry {
    mutation_fns: BTreeMap<String, MutationFnEntry>,
    seed_fns: BTreeMap<String, SeedFnEntry>,
}

impl BridgeRegistry {
    /// Create an empty bridge registry.
    pub fn new() -> Self {
        Self {
            mutation_fns: BTreeMap::new(),
            seed_fns: BTreeMap::new(),
        }
    }

    /// Register a Rust mutation function and return its component representation.
    pub fn register_mutation(
        &mut self,
        name: &str,
        rust_fn: fn(&SemanticGraph, &mut StdRng) -> SemanticGraph,
    ) -> MutationComponent {
        self.mutation_fns.insert(
            name.to_string(),
            MutationFnEntry { func: rust_fn },
        );
        MutationComponent {
            name: name.to_string(),
            program: placeholder_graph(),
        }
    }

    /// Register a Rust seed function and return its component representation.
    pub fn register_seed(
        &mut self,
        name: &str,
        rust_fn: fn(&mut StdRng) -> Fragment,
    ) -> SeedComponent {
        self.seed_fns.insert(
            name.to_string(),
            SeedFnEntry { func: rust_fn },
        );
        SeedComponent {
            name: name.to_string(),
            program: placeholder_graph(),
        }
    }

    /// Apply a mutation component to a program graph.
    ///
    /// Looks up the Rust function by component name and calls it. In the
    /// future, if the component's `program` field holds a real IRIS program,
    /// this will interpret that program instead.
    pub fn apply_mutation(
        &self,
        component: &MutationComponent,
        graph: &SemanticGraph,
        rng: &mut StdRng,
    ) -> Result<SemanticGraph, ComponentError> {
        let entry = self.mutation_fns.get(&component.name).ok_or_else(|| {
            ComponentError::InvalidInput(format!(
                "no Rust function registered for mutation '{}'",
                component.name
            ))
        })?;
        Ok((entry.func)(graph, rng))
    }

    /// Apply a seed component to generate a new program.
    pub fn apply_seed(
        &self,
        component: &SeedComponent,
        rng: &mut StdRng,
    ) -> Result<Fragment, ComponentError> {
        let entry = self.seed_fns.get(&component.name).ok_or_else(|| {
            ComponentError::InvalidInput(format!(
                "no Rust function registered for seed '{}'",
                component.name
            ))
        })?;
        Ok((entry.func)(rng))
    }
}

impl Default for BridgeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Convenience wrappers (standalone functions)
// ---------------------------------------------------------------------------

/// Wrap a Rust mutation function as an IRIS-callable component.
///
/// This is the bridge: Rust function <-> IRIS Program semantics.
/// Returns a `(MutationComponent, BridgeRegistry)` pair. The registry
/// holds the actual function pointer; the component holds type metadata.
pub fn wrap_mutation_as_component(
    name: &str,
    rust_fn: fn(&SemanticGraph, &mut StdRng) -> SemanticGraph,
) -> (MutationComponent, BridgeRegistry) {
    let mut bridge = BridgeRegistry::new();
    let component = bridge.register_mutation(name, rust_fn);
    (component, bridge)
}

/// Wrap a Rust seed function as an IRIS-callable component.
pub fn wrap_seed_as_component(
    name: &str,
    rust_fn: fn(&mut StdRng) -> Fragment,
) -> (SeedComponent, BridgeRegistry) {
    let mut bridge = BridgeRegistry::new();
    let component = bridge.register_seed(name, rust_fn);
    (component, bridge)
}

/// Execute an IRIS MutationComponent on a program.
///
/// Uses the bridge registry to dispatch to the underlying Rust function.
pub fn apply_mutation_component(
    bridge: &BridgeRegistry,
    component: &MutationComponent,
    program: &SemanticGraph,
    rng: &mut StdRng,
) -> Result<SemanticGraph, ComponentError> {
    bridge.apply_mutation(component, program, rng)
}

// ---------------------------------------------------------------------------
// Placeholder graph
// ---------------------------------------------------------------------------

/// Create a minimal placeholder `SemanticGraph`.
///
/// This graph represents "this component is backed by a Rust function, not
/// yet an IRIS program." It has no nodes and serves only as a type-level
/// marker. When IRIS achieves autopoietic closure, this placeholder will
/// be replaced with a real program graph.
fn placeholder_graph() -> SemanticGraph {
    SemanticGraph {
        root: iris_types::graph::NodeId(0),
        nodes: HashMap::new(),
        edges: Vec::new(),
        type_env: TypeEnv {
            types: BTreeMap::new(),
        },
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}
