//! Integration tests for the component-as-program interface.
//!
//! Validates that Rust mutation operators and seed generators can be wrapped
//! as IRIS components and executed through the bridge. This is the interface
//! layer; actual IRIS-native components come later when IRIS can evolve them.

use std::collections::{BTreeMap, HashMap};

use rand::rngs::StdRng;
use rand::SeedableRng;

use iris_types::component::{ComponentRegistry, MutationComponent};
use iris_types::cost::{CostBound, CostTerm};
use iris_types::fragment::Fragment;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

use iris_evolve::component_bridge::{
    apply_mutation_component, wrap_mutation_as_component, wrap_seed_as_component, BridgeRegistry,
};

// ---------------------------------------------------------------------------
// Adapter functions
// ---------------------------------------------------------------------------
// The bridge uses concrete `fn` pointers (`fn(&SemanticGraph, &mut StdRng)`).
// The generic mutation operators take `impl Rng`, so we provide thin adapters
// that monomorphize them to `StdRng`.

/// Adapter: wraps `iris_evolve::mutation::insert_node` for StdRng.
fn insert_node_adapter(graph: &SemanticGraph, rng: &mut StdRng) -> SemanticGraph {
    iris_evolve::mutation::insert_node(graph, rng)
}

/// Adapter: wraps `iris_evolve::mutation::mutate` for StdRng.
fn mutate_adapter(graph: &SemanticGraph, rng: &mut StdRng) -> SemanticGraph {
    iris_evolve::mutation::mutate(graph, rng)
}

/// Adapter: wraps `iris_evolve::seed::random_arithmetic_program` with fixed depth/width.
fn random_arithmetic_seed(rng: &mut StdRng) -> Fragment {
    iris_evolve::seed::random_arithmetic_program(rng, 2, 2)
}

/// Adapter: wraps `iris_evolve::seed::identity_program`.
fn identity_seed(_rng: &mut StdRng) -> Fragment {
    iris_evolve::seed::identity_program()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal valid SemanticGraph for testing.
fn make_test_graph() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // A simple graph: one Lit node (root) and one Prim node connected to it.
    let lit_node = Node {
        id: NodeId(1),
        kind: NodeKind::Lit,
        type_sig: TypeId(0),
        cost: CostTerm::Unit,
        arity: 0,
        resolution_depth: 0, salt: 0,
        payload: NodePayload::Lit {
            type_tag: 0,
            value: 42i64.to_le_bytes().to_vec(),
        },
    };

    let lit2_node = Node {
        id: NodeId(2),
        kind: NodeKind::Lit,
        type_sig: TypeId(0),
        cost: CostTerm::Unit,
        arity: 0,
        resolution_depth: 0, salt: 0,
        payload: NodePayload::Lit {
            type_tag: 0,
            value: 7i64.to_le_bytes().to_vec(),
        },
    };

    let prim_node = Node {
        id: NodeId(3),
        kind: NodeKind::Prim,
        type_sig: TypeId(0),
        cost: CostTerm::Unit,
        arity: 2,
        resolution_depth: 0, salt: 0,
        payload: NodePayload::Prim { opcode: 0x00 }, // add
    };

    nodes.insert(NodeId(1), lit_node);
    nodes.insert(NodeId(2), lit2_node);
    nodes.insert(NodeId(3), prim_node);

    let edges = vec![
        Edge {
            source: NodeId(3),
            target: NodeId(1),
            port: 0,
            label: EdgeLabel::Argument,
        },
        Edge {
            source: NodeId(3),
            target: NodeId(2),
            port: 1,
            label: EdgeLabel::Argument,
        },
    ];

    SemanticGraph {
        root: NodeId(3),
        nodes,
        edges,
        type_env: TypeEnv {
            types: BTreeMap::new(),
        },
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Wrap insert_node as a MutationComponent, apply it, verify it modifies
/// the graph (adds at least one node).
#[test]
fn test_wrap_insert_node_as_mutation_component() {
    let (component, bridge) = wrap_mutation_as_component("insert_node", insert_node_adapter);
    assert_eq!(component.name, "insert_node");

    let graph = make_test_graph();
    let original_node_count = graph.nodes.len();

    let mut rng = StdRng::seed_from_u64(42);
    let mutated = apply_mutation_component(&bridge, &component, &graph, &mut rng).unwrap();

    // insert_node adds exactly one node.
    assert_eq!(
        mutated.nodes.len(),
        original_node_count + 1,
        "insert_node should add exactly one node"
    );
    // The mutated graph should have more edges than the original.
    assert!(
        mutated.edges.len() > graph.edges.len(),
        "insert_node should add edges to connect the new node"
    );
}

/// Wrap random_arithmetic_program as a SeedComponent, apply it, verify it
/// produces a valid program.
#[test]
fn test_wrap_random_arithmetic_as_seed_component() {
    let (component, bridge) = wrap_seed_as_component("random_arithmetic", random_arithmetic_seed);
    assert_eq!(component.name, "random_arithmetic");

    let mut rng = StdRng::seed_from_u64(99);
    let fragment = bridge.apply_seed(&component, &mut rng).unwrap();

    // A valid fragment should have a non-empty graph.
    assert!(
        !fragment.graph.nodes.is_empty(),
        "seed should produce a graph with at least one node"
    );
    // The root should be present in the node map.
    assert!(
        fragment.graph.nodes.contains_key(&fragment.graph.root),
        "root node should exist in the graph"
    );
}

/// ComponentRegistry holds multiple components and can select by name.
#[test]
fn test_component_registry_lookup() {
    let mut registry = ComponentRegistry::new();

    // Register multiple mutation components.
    let (comp_insert, _) = wrap_mutation_as_component("insert_node", insert_node_adapter);
    let (comp_mutate, _) = wrap_mutation_as_component("mutate", mutate_adapter);

    registry.mutations.push(comp_insert);
    registry.mutations.push(comp_mutate);

    // Register seed components.
    let (seed_arith, _) = wrap_seed_as_component("random_arithmetic", random_arithmetic_seed);
    let (seed_identity, _) = wrap_seed_as_component("identity", identity_seed);

    registry.seeds.push(seed_arith);
    registry.seeds.push(seed_identity);

    // Lookup by name.
    assert!(registry.find_mutation("insert_node").is_some());
    assert!(registry.find_mutation("mutate").is_some());
    assert!(registry.find_mutation("nonexistent").is_none());

    assert!(registry.find_seed("random_arithmetic").is_some());
    assert!(registry.find_seed("identity").is_some());
    assert!(registry.find_seed("nonexistent").is_none());

    assert!(registry.find_fitness("anything").is_none());

    // Verify counts.
    assert_eq!(registry.mutations.len(), 2);
    assert_eq!(registry.seeds.len(), 2);
    assert_eq!(registry.fitness.len(), 0);
}

/// apply_mutation_component works with wrapped Rust functions and produces
/// valid graphs.
#[test]
fn test_apply_mutation_component_with_bridge() {
    let mut bridge = BridgeRegistry::new();

    let comp_insert = bridge.register_mutation("insert_node", insert_node_adapter);
    let comp_mutate = bridge.register_mutation("mutate", mutate_adapter);

    let graph = make_test_graph();
    let mut rng = StdRng::seed_from_u64(123);

    // Apply insert_node.
    let result1 = bridge
        .apply_mutation(&comp_insert, &graph, &mut rng)
        .unwrap();
    assert!(
        result1.nodes.len() > graph.nodes.len(),
        "insert_node should grow the graph"
    );

    // Apply generic mutate.
    let result2 = bridge
        .apply_mutation(&comp_mutate, &graph, &mut rng)
        .unwrap();
    // mutate applies a random operator; the result should be a valid graph
    // (may or may not change size, but should not be empty).
    assert!(
        !result2.nodes.is_empty(),
        "mutate should produce a non-empty graph"
    );
}

/// An unregistered component name returns an error.
#[test]
fn test_apply_unregistered_component_errors() {
    let bridge = BridgeRegistry::new();
    let fake_component = MutationComponent {
        name: "nonexistent".to_string(),
        program: make_test_graph(),
    };

    let graph = make_test_graph();
    let mut rng = StdRng::seed_from_u64(1);

    let result = bridge.apply_mutation(&fake_component, &graph, &mut rng);
    assert!(result.is_err(), "unregistered component should error");
}

/// Deme::mutate_with_components falls back to Rust mutation when the
/// registry is empty.
#[test]
fn test_deme_mutate_with_empty_registry_falls_back() {
    let registry = ComponentRegistry::new();
    let bridge = BridgeRegistry::new();

    // Create a minimal deme with one individual.
    let fragment = iris_evolve::seed::identity_program();
    let deme = iris_evolve::population::Deme::initialize(1, |_| fragment.clone());

    let graph = make_test_graph();
    let mut rng = StdRng::seed_from_u64(77);

    // Should not panic; falls back to mutation::mutate.
    let result = deme.mutate_with_components(&graph, &registry, &bridge, &mut rng);
    assert!(
        !result.nodes.is_empty(),
        "fallback mutation should produce a non-empty graph"
    );
}

/// Deme::mutate_with_components uses registered components when available.
#[test]
fn test_deme_mutate_with_registered_components() {
    let mut registry = ComponentRegistry::new();
    let mut bridge = BridgeRegistry::new();

    let comp = bridge.register_mutation("insert_node", insert_node_adapter);
    registry.mutations.push(comp);

    let fragment = iris_evolve::seed::identity_program();
    let deme = iris_evolve::population::Deme::initialize(1, |_| fragment.clone());

    let graph = make_test_graph();
    let original_count = graph.nodes.len();
    let mut rng = StdRng::seed_from_u64(55);

    let result = deme.mutate_with_components(&graph, &registry, &bridge, &mut rng);
    // insert_node is the only registered mutation, so it must be selected.
    assert_eq!(
        result.nodes.len(),
        original_count + 1,
        "should use the registered insert_node component"
    );
}
