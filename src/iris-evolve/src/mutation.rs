use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU8, Ordering};

use rand::Rng;

use iris_types::cost::{CostBound, CostTerm};
use iris_types::fragment::FragmentId;
use iris_types::graph::{Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, SemanticGraph};
use iris_types::hash::{compute_node_id, SemanticHash};
use iris_types::types::TypeId;

// ---------------------------------------------------------------------------
// Mutation operator weights — updated with structural operators
// ---------------------------------------------------------------------------

/// Cumulative weight thresholds for operator selection.
///
/// Strategy: structural mutations get ~45% weight to enable composition
/// discovery. Core mutations (insert/delete/rewire/replace/literal) handle
/// fine-tuning. swap_fold_op is the most impactful: it does the critical
/// correlated base+op change that solves factorial/product/min/max.
///
/// Weight budget allocation:
/// - Core mutations (insert/delete/rewire/replace/literal): ~45%
/// - Structural mutations (map/filter/zip/compose/fold-op/guard): ~55%
const WEIGHT_THRESHOLDS: [(u8, f64); 16] = [
    ( 0, 0.10),  // insert_node        0.10
    ( 1, 0.20),  // delete_node         0.10
    ( 2, 0.28),  // rewire_edge         0.08
    ( 3, 0.31),  // replace_kind        0.03
    ( 4, 0.37),  // replace_prim        0.06
    ( 5, 0.44),  // mutate_literal      0.07
    ( 6, 0.45),  // duplicate_subgraph  0.01
    ( 7, 0.46),  // wrap_in_guard       0.01
    ( 8, 0.47),  // annotate_cost       0.01
    // --- Structural mutation operators (53% total) ---
    ( 9, 0.56),  // wrap_in_map         0.09
    (10, 0.65),  // wrap_in_filter      0.09
    (11, 0.72),  // compose_stages      0.07
    (12, 0.79),  // insert_zip          0.07
    (13, 0.92),  // swap_fold_op        0.13
    (14, 0.97),  // add_guard_condition 0.05
    (15, 1.00),  // extract_to_ref      0.03
];

// ---------------------------------------------------------------------------
// Custom weights for self-improvement (thread-local override)
// ---------------------------------------------------------------------------

thread_local! {
    /// Thread-local override for mutation operator weights.
    /// When set, `mutate()` uses these cumulative thresholds instead of
    /// the hardcoded `WEIGHT_THRESHOLDS`.
    static CUSTOM_WEIGHTS: RefCell<Option<Vec<(u8, f64)>>> = RefCell::new(None);
}

/// Install custom mutation weights for the current thread.
///
/// The weights should be cumulative thresholds: `[(operator_id, cumulative_threshold)]`.
/// Used by the self-improvement module to evaluate alternative weight distributions.
///
/// # Validation
/// Returns an error string if:
/// - Any threshold is negative or > 1.0 (out of range).
/// - The sequence is not monotonically non-decreasing.
/// - The final threshold is not in (0.0, 1.0] (must sum to a positive total).
///
/// On validation failure the thread-local weights are left unchanged.
pub fn set_custom_weights(weights: &[(u8, f64)]) -> Result<(), String> {
    if weights.is_empty() {
        return Err("custom weights must be non-empty".into());
    }

    let mut prev: f64 = 0.0;
    for (i, &(_op, threshold)) in weights.iter().enumerate() {
        if threshold < 0.0 {
            return Err(format!(
                "custom weights[{}]: threshold {} is negative",
                i, threshold
            ));
        }
        if threshold > 1.0 + 1e-9 {
            return Err(format!(
                "custom weights[{}]: threshold {} exceeds 1.0",
                i, threshold
            ));
        }
        if threshold < prev - 1e-9 {
            return Err(format!(
                "custom weights[{}]: threshold {} is not monotonically non-decreasing (prev {})",
                i, threshold, prev
            ));
        }
        prev = threshold;
    }

    // Final threshold must be positive so at least one operator is reachable.
    let final_threshold = weights.last().map(|&(_, t)| t).unwrap_or(0.0);
    if final_threshold <= 0.0 {
        return Err("custom weights: final cumulative threshold must be > 0.0".into());
    }

    CUSTOM_WEIGHTS.with(|w| {
        *w.borrow_mut() = Some(weights.to_vec());
    });
    Ok(())
}

/// Clear custom mutation weights, reverting to the hardcoded defaults.
pub fn clear_custom_weights() {
    CUSTOM_WEIGHTS.with(|w| {
        *w.borrow_mut() = None;
    });
}

// ---------------------------------------------------------------------------
// RAII guard for custom weights
// ---------------------------------------------------------------------------

/// RAII guard that restores the previous thread-local mutation weights on drop.
///
/// Using this guard instead of calling `set_custom_weights` / `clear_custom_weights`
/// directly ensures that weights are always restored even if the calling code
/// panics, leaving the thread in a consistent state.
///
/// # Example
/// ```ignore
/// {
///     let _guard = CustomWeightsGuard::new(&my_weights)?;
///     // ... run evolution with custom weights ...
/// } // weights automatically restored here
/// ```
pub struct CustomWeightsGuard {
    /// The weights that were active before this guard was created.
    previous: Option<Vec<(u8, f64)>>,
}

impl CustomWeightsGuard {
    /// Install `weights` as the current thread-local weights.
    ///
    /// Validates the weights before installing; returns an error if invalid.
    /// The previous weights (if any) are saved and will be restored on drop.
    pub fn new(weights: &[(u8, f64)]) -> Result<Self, String> {
        // Save the current weights before replacing them.
        let previous = CUSTOM_WEIGHTS.with(|w| w.borrow().clone());

        // Validate and install new weights.
        set_custom_weights(weights)?;

        Ok(Self { previous })
    }
}

impl Drop for CustomWeightsGuard {
    fn drop(&mut self) {
        // Restore whatever was active before this guard was created.
        CUSTOM_WEIGHTS.with(|w| {
            *w.borrow_mut() = self.previous.take();
        });
    }
}

// ---------------------------------------------------------------------------
// Program size limit
// ---------------------------------------------------------------------------

/// Maximum nodes allowed in a single individual after mutation.
///
/// Prevents runaway memory and CPU usage from unbounded graph growth.
/// Mutation operators that would exceed this limit return the parent unchanged.
pub const MAX_NODES_PER_INDIVIDUAL: usize = 1000;

// ---------------------------------------------------------------------------
// Top-level mutate
// ---------------------------------------------------------------------------

/// Apply a random mutation operator to the graph.
///
/// If custom weights have been installed via `set_custom_weights()`, those
/// are used instead of the hardcoded defaults. This enables the
/// self-improvement module to evaluate different weight distributions.
pub fn mutate(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let roll: f64 = rng.r#gen();
    let mut op = 15u8;

    // Check for thread-local custom weights first.
    let used_custom = CUSTOM_WEIGHTS.with(|w| {
        let guard = w.borrow();
        if let Some(custom) = guard.as_ref() {
            for &(id, threshold) in custom {
                if roll < threshold {
                    op = id;
                    return true;
                }
            }
            // If we fell through all thresholds, use last operator.
            op = custom.last().map(|&(id, _)| id).unwrap_or(15);
            true
        } else {
            false
        }
    });

    if !used_custom {
        for &(id, threshold) in &WEIGHT_THRESHOLDS {
            if roll < threshold {
                op = id;
                break;
            }
        }
    }

    dispatch_mutation(op, graph, rng)
}

/// Apply a random mutation operator, returning both the mutated graph and the
/// operator index that was selected. This enables causal attribution — tracking
/// which operators produce improvements.
pub fn mutate_tracked(graph: &SemanticGraph, rng: &mut impl Rng) -> (SemanticGraph, u8) {
    let roll: f64 = rng.r#gen();
    let mut op = 15u8;

    let used_custom = CUSTOM_WEIGHTS.with(|w| {
        let guard = w.borrow();
        if let Some(custom) = guard.as_ref() {
            for &(id, threshold) in custom {
                if roll < threshold {
                    op = id;
                    return true;
                }
            }
            op = custom.last().map(|&(id, _)| id).unwrap_or(15);
            true
        } else {
            false
        }
    });

    if !used_custom {
        for &(id, threshold) in &WEIGHT_THRESHOLDS {
            if roll < threshold {
                op = id;
                break;
            }
        }
    }

    (dispatch_mutation(op, graph, rng), op)
}

/// Apply a specific mutation operator by index.
fn dispatch_mutation(op: u8, graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let result = match op {
        0 => insert_node(graph, rng),
        1 => delete_node(graph, rng),
        2 => rewire_edge(graph, rng),
        3 => replace_kind(graph, rng),
        4 => replace_prim(graph, rng),
        5 => mutate_literal(graph, rng),
        6 => duplicate_subgraph(graph, rng),
        7 => wrap_in_guard(graph, rng),
        8 => annotate_cost(graph, rng),
        // Structural mutation operators
        9 => wrap_in_map(graph, rng),
        10 => wrap_in_filter(graph, rng),
        11 => compose_stages(graph, rng),
        12 => insert_zip(graph, rng),
        13 => swap_fold_op(graph, rng),
        14 => add_guard_condition(graph, rng),
        _ => extract_to_ref(graph, rng),
    };

    // Reject mutations that exceed the program size limit — return parent unchanged.
    if result.nodes.len() > MAX_NODES_PER_INDIVIDUAL {
        return graph.clone();
    }

    result
}

// ---------------------------------------------------------------------------
// Proof-guided mutation
// ---------------------------------------------------------------------------

use iris_bootstrap::syntax::kernel::checker::{MutationHint, ProofFailureDiagnosis};
use iris_types::eval::{TestCase, Value};

/// Apply a mutation guided by proof failure diagnoses.
///
/// Picks a random diagnosis and applies its suggested mutation hint.
/// This gives evolution a targeted way to fix type errors, missing cost
/// annotations, and tier violations rather than relying on blind random
/// mutation.
///
/// Falls back to standard `mutate()` if no diagnoses are available or
/// the hint cannot be applied.
pub fn proof_guided_mutation(
    graph: &SemanticGraph,
    diagnoses: &[ProofFailureDiagnosis],
    rng: &mut impl Rng,
) -> SemanticGraph {
    if diagnoses.is_empty() {
        return mutate(graph, rng);
    }

    // Pick a random diagnosis.
    let diag = &diagnoses[rng.gen_range(0..diagnoses.len())];

    // If the failing node doesn't exist in the graph, fall back.
    if !graph.nodes.contains_key(&diag.node_id) {
        return mutate(graph, rng);
    }

    match &diag.suggestion {
        Some(MutationHint::WrapInGuard) => {
            // Use the existing wrap_in_guard operator, targeting the failing node.
            wrap_in_guard_at(graph, diag.node_id)
        }
        Some(MutationHint::AddCostAnnotation) => {
            // Annotate the failing node with a cost bound.
            annotate_cost_at(graph, diag.node_id, rng)
        }
        Some(MutationHint::FixTypeSignature(expected, _actual)) => {
            // Change the failing node's type_sig to the expected type.
            fix_type_sig(graph, diag.node_id, *expected)
        }
        Some(MutationHint::AddTerminationCheck) => {
            // Wrap the failing fold/unfold in a guard for termination.
            wrap_in_guard_at(graph, diag.node_id)
        }
        Some(MutationHint::DowngradeTier(kind)) => {
            // Replace the tier-violating node with a simpler construct.
            downgrade_node(graph, diag.node_id, *kind, rng)
        }
        None => mutate(graph, rng),
    }
}

/// Generate a test case from a proof failure counterexample.
///
/// When a refinement type check fails and produces a counterexample (a variable
/// assignment that violates the refinement predicate), this function converts
/// it into a `TestCase` that targets the failure. The counterexample values
/// become test inputs, focusing evolutionary selection pressure on the edge
/// cases that break programs (CDGP pattern).
///
/// Returns `None` if the diagnosis has no counterexample or if the counterexample
/// cannot be meaningfully converted to a test case.
pub fn counterexample_to_test_case(
    diagnosis: &ProofFailureDiagnosis,
    _graph: &SemanticGraph,
) -> Option<TestCase> {
    let counterexample = diagnosis.counterexample.as_ref()?;

    if counterexample.is_empty() {
        return None;
    }

    // Convert the counterexample variable assignments into test case inputs.
    // Sort by BoundVar index so the ordering is deterministic.
    let mut entries: Vec<_> = counterexample.iter().collect();
    entries.sort_by_key(|(bv, _)| bv.0);

    let inputs: Vec<Value> = entries.iter().map(|(_, val)| Value::Int(**val)).collect();

    // The expected output is None: the test case is used to observe what the
    // program produces on this input. If the program crashes or produces a
    // wrong result, that failure drives selection against it.
    //
    // When the diagnosis includes a FixTypeSignature hint, we know the type
    // mismatch but not the correct value, so we leave expected_output as None.
    // This makes the test case a "negative example" -- it exposes the bug
    // without asserting a specific correct answer.
    Some(TestCase {
        inputs,
        expected_output: None,
        initial_state: None,
        expected_state: None,
    })
}

/// Wrap a specific node in a Guard (targeted version of wrap_in_guard).
fn wrap_in_guard_at(graph: &SemanticGraph, target_id: NodeId) -> SemanticGraph {
    let mut g = graph.clone();

    let type_sig = g
        .nodes
        .get(&target_id)
        .map(|n| n.type_sig)
        .unwrap_or(TypeId(0));

    // Create a trivial predicate node (Lit true).
    let mut pred_node = Node {
        id: NodeId(0),
        kind: NodeKind::Lit,
        type_sig,
        cost: CostTerm::Unit,
        arity: 0,
        resolution_depth: MUTATION_UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed),
        salt: 0,
        payload: NodePayload::Lit {
            type_tag: 4, // Bool
            value: vec![1],
        },
    };
    pred_node.id = compute_node_id(&pred_node);

    // Create a fallback node (Lit 0).
    let mut fallback_node = Node {
        id: NodeId(0),
        kind: NodeKind::Lit,
        type_sig,
        cost: CostTerm::Unit,
        arity: 0,
        resolution_depth: MUTATION_UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed),
        salt: 0,
        payload: NodePayload::Lit {
            type_tag: 0,
            value: vec![0],
        },
    };
    fallback_node.id = compute_node_id(&fallback_node);

    let pred_id = pred_node.id;
    let fallback_id = fallback_node.id;

    // Create the Guard node.
    let mut guard_node = Node {
        id: NodeId(0),
        kind: NodeKind::Guard,
        type_sig,
        cost: CostTerm::Unit,
        arity: 3,
        resolution_depth: MUTATION_UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed),
        salt: 0,
        payload: NodePayload::Guard {
            predicate_node: pred_id,
            body_node: target_id,
            fallback_node: fallback_id,
        },
    };
    guard_node.id = compute_node_id(&guard_node);
    let guard_id = guard_node.id;

    g.nodes.insert(pred_id, pred_node);
    g.nodes.insert(fallback_id, fallback_node);
    g.nodes.insert(guard_id, guard_node);

    // Add edges: guard -> pred, guard -> body, guard -> fallback.
    g.edges.push(Edge {
        source: guard_id,
        target: pred_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    g.edges.push(Edge {
        source: guard_id,
        target: target_id,
        port: 1,
        label: EdgeLabel::Continuation,
    });
    g.edges.push(Edge {
        source: guard_id,
        target: fallback_id,
        port: 2,
        label: EdgeLabel::Continuation,
    });

    // If the target was the root, promote the guard to root.
    if g.root == target_id {
        g.root = guard_id;
    } else {
        // Rewire edges that pointed to the target to point to the guard.
        for edge in &mut g.edges {
            if edge.target == target_id && edge.source != guard_id {
                edge.target = guard_id;
            }
        }
    }

    rehash_graph(&mut g);
    g
}

/// Annotate a specific node with a cost bound.
fn annotate_cost_at(graph: &SemanticGraph, target_id: NodeId, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();
    if let Some(node) = g.nodes.get_mut(&target_id) {
        let cost = match rng.gen_range(0..3u8) {
            0 => CostBound::Zero,
            1 => CostBound::Constant(1),
            _ => CostBound::Constant(rng.gen_range(1..=100)),
        };
        node.cost = CostTerm::Annotated(cost);
    }
    rehash_graph(&mut g);
    g
}

/// Fix a node's type signature to the expected type.
fn fix_type_sig(graph: &SemanticGraph, target_id: NodeId, expected: TypeId) -> SemanticGraph {
    let mut g = graph.clone();
    if let Some(node) = g.nodes.get_mut(&target_id) {
        // Only change the type_sig if the expected type exists in the env.
        if g.type_env.types.contains_key(&expected) {
            node.type_sig = expected;
            // Recompute node ID since type_sig changed.
            node.id = compute_node_id(node);
            // Update any edges and maps that referenced the old ID.
            let old_id = target_id;
            let new_id = node.id;
            if old_id != new_id {
                let node_data = g.nodes.remove(&old_id).unwrap();
                g.nodes.insert(new_id, node_data);
                // Update edges.
                for edge in &mut g.edges {
                    if edge.source == old_id {
                        edge.source = new_id;
                    }
                    if edge.target == old_id {
                        edge.target = new_id;
                    }
                }
                // Update root.
                if g.root == old_id {
                    g.root = new_id;
                }
            }
        }
    }
    rehash_graph(&mut g);
    g
}

/// Downgrade a tier-violating node to a simpler construct.
fn downgrade_node(
    graph: &SemanticGraph,
    target_id: NodeId,
    kind: NodeKind,
    rng: &mut impl Rng,
) -> SemanticGraph {
    let mut g = graph.clone();
    match kind {
        // Replace Fold/Unfold/LetRec with a simple Prim apply.
        NodeKind::Fold | NodeKind::Unfold | NodeKind::LetRec => {
            if let Some(node) = g.nodes.get_mut(&target_id) {
                let type_sig = node.type_sig;
                let new_node = make_unique_node(
                    NodeKind::Prim,
                    NodePayload::Prim { opcode: 0x00 },
                    type_sig,
                    0,
                );
                let old_id = target_id;
                let new_id = new_node.id;
                g.nodes.remove(&old_id);
                g.nodes.insert(new_id, new_node);
                // Remove edges from old node.
                g.edges.retain(|e| e.source != old_id);
                // Rewire edges targeting old node.
                for edge in &mut g.edges {
                    if edge.target == old_id {
                        edge.target = new_id;
                    }
                }
                if g.root == old_id {
                    g.root = new_id;
                }
            }
        }
        // For Neural, replace with a Lit node.
        NodeKind::Neural => {
            if let Some(node) = g.nodes.get_mut(&target_id) {
                let type_sig = node.type_sig;
                let new_node = make_unique_node(
                    NodeKind::Lit,
                    NodePayload::Lit {
                        type_tag: 0,
                        value: vec![0, 0, 0, 0],
                    },
                    type_sig,
                    0,
                );
                let old_id = target_id;
                let new_id = new_node.id;
                g.nodes.remove(&old_id);
                g.nodes.insert(new_id, new_node);
                g.edges.retain(|e| e.source != old_id);
                for edge in &mut g.edges {
                    if edge.target == old_id {
                        edge.target = new_id;
                    }
                }
                if g.root == old_id {
                    g.root = new_id;
                }
            }
        }
        // For TypeAbst, replace with identity (remove type abstraction layer).
        _ => {
            // Fall back to standard mutation for other kinds.
            return mutate(&g, rng);
        }
    }
    rehash_graph(&mut g);
    g
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect node IDs into a Vec for random indexing.
fn node_ids(graph: &SemanticGraph) -> Vec<NodeId> {
    graph.nodes.keys().copied().collect()
}

/// Pick a random node ID from the graph. Returns None if graph is empty.
fn random_node_id(graph: &SemanticGraph, rng: &mut impl Rng) -> Option<NodeId> {
    let ids = node_ids(graph);
    if ids.is_empty() {
        return None;
    }
    Some(ids[rng.gen_range(0..ids.len())])
}

/// Create a fresh Prim node with a random opcode (arithmetic or higher-order).
fn make_random_prim_node(rng: &mut impl Rng, type_sig: TypeId) -> Node {
    let (opcode, arity) = random_prim_opcode(rng);
    let mut node = Node {
        id: NodeId(0), // placeholder
        kind: NodeKind::Prim,
        type_sig,
        cost: CostTerm::Unit,
        arity,
        resolution_depth: 2, salt: 0,
        payload: NodePayload::Prim { opcode },
    };
    node.id = compute_node_id(&node);
    node
}

/// Pick a random Prim opcode. 80% arithmetic (0x00-0x09), 20% higher-order
/// (map/zip/concat/filter/take/drop/reverse/flat_map).
fn random_prim_opcode(rng: &mut impl Rng) -> (u8, u8) {
    if rng.gen_range(0..5u8) < 4 {
        // Arithmetic range.
        (rng.gen_range(0x00..=0x09u8), 2)
    } else {
        // Higher-order range.
        let ho: &[(u8, u8)] = &[
            (0x30, 2), // map:      collection + function
            (0x31, 2), // filter:   collection + predicate
            (0x32, 2), // zip:      two collections
            (0x33, 2), // take:     collection + count
            (0x34, 2), // drop:     collection + count
            (0x35, 2), // concat:   two collections
            (0x36, 1), // reverse:  one collection
            (0x39, 2), // flat_map: collection + function
        ];
        let &(opcode, arity) = &ho[rng.gen_range(0..ho.len())];
        (opcode, arity)
    }
}

/// Recompute the semantic hash of the graph (simplified: hash of all node IDs
/// and edges).
fn rehash_graph(graph: &mut SemanticGraph) {
    let mut hasher = blake3::Hasher::new();
    for (nid, _) in &graph.nodes {
        hasher.update(&nid.0.to_le_bytes());
    }
    for edge in &graph.edges {
        hasher.update(&edge.source.0.to_le_bytes());
        hasher.update(&edge.target.0.to_le_bytes());
        hasher.update(&[edge.port, edge.label as u8]);
    }
    graph.hash = SemanticHash(*hasher.finalize().as_bytes());
}

// ---------------------------------------------------------------------------
// Mutation operators
// ---------------------------------------------------------------------------

/// Insert a new Prim node connected to random edges (weight 0.25).
pub fn insert_node(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();

    // Pick a type signature from an existing node, or use a default.
    let type_sig = g
        .nodes
        .values()
        .next()
        .map(|n| n.type_sig)
        .unwrap_or(TypeId(0));

    let new_node = make_random_prim_node(rng, type_sig);
    let new_id = new_node.id;

    // Connect the new node: pick a random existing node as source, another as
    // target. The new node is inserted between them.
    let ids = node_ids(&g);
    if ids.len() >= 2 {
        let src = ids[rng.gen_range(0..ids.len())];
        let tgt_idx = rng.gen_range(0..ids.len());
        let tgt = ids[tgt_idx];

        g.edges.push(Edge {
            source: src,
            target: new_id,
            port: 0,
            label: EdgeLabel::Argument,
        });
        g.edges.push(Edge {
            source: new_id,
            target: tgt,
            port: 0,
            label: EdgeLabel::Argument,
        });
    } else if !ids.is_empty() {
        // Single node: connect new node to it
        g.edges.push(Edge {
            source: ids[0],
            target: new_id,
            port: 0,
            label: EdgeLabel::Argument,
        });
    }

    g.nodes.insert(new_id, new_node);
    rehash_graph(&mut g);
    g
}

/// Delete a non-root node and reconnect its edges (weight 0.15).
///
/// Protects structural composition nodes (Map/Filter/Zip Prim nodes and Fold
/// nodes) from deletion 75% of the time to preserve hard-won compositions.
fn delete_node(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();
    let ids = node_ids(&g);

    // Never delete the root or if only one node remains.
    let candidates: Vec<NodeId> = ids.into_iter().filter(|&id| id != g.root).collect();
    if candidates.is_empty() {
        return g;
    }

    let victim = candidates[rng.gen_range(0..candidates.len())];

    // Protect structural composition nodes: 75% chance to skip deletion
    // of Fold nodes or higher-order Prim nodes (map/filter/zip).
    if let Some(node) = g.nodes.get(&victim) {
        let is_structural = match node.kind {
            NodeKind::Fold => true,
            NodeKind::Prim => {
                if let NodePayload::Prim { opcode } = &node.payload {
                    (0x30..=0x3F).contains(opcode)
                } else {
                    false
                }
            }
            _ => false,
        };
        if is_structural && rng.gen_range(0..4u8) < 3 {
            // 75% chance: skip deletion, return unmodified.
            return g;
        }
    }

    // Find nodes that feed into victim, and nodes victim feeds into.
    let sources: Vec<NodeId> = g
        .edges
        .iter()
        .filter(|e| e.target == victim)
        .map(|e| e.source)
        .collect();
    let targets: Vec<NodeId> = g
        .edges
        .iter()
        .filter(|e| e.source == victim)
        .map(|e| e.target)
        .collect();

    // Remove all edges involving victim.
    g.edges.retain(|e| e.source != victim && e.target != victim);

    // Reconnect: for each source, connect to each target (or at least one).
    if !sources.is_empty() && !targets.is_empty() {
        for &src in &sources {
            for &tgt in &targets {
                g.edges.push(Edge {
                    source: src,
                    target: tgt,
                    port: 0,
                    label: EdgeLabel::Argument,
                });
            }
        }
    }

    g.nodes.remove(&victim);
    rehash_graph(&mut g);
    g
}

/// Rewire an edge's source or target (weight 0.15).
fn rewire_edge(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();
    if g.edges.is_empty() {
        return g;
    }

    let ids = node_ids(&g);
    if ids.is_empty() {
        return g;
    }

    let edge_idx = rng.gen_range(0..g.edges.len());
    let new_target = ids[rng.gen_range(0..ids.len())];

    if rng.r#gen::<bool>() {
        g.edges[edge_idx].target = new_target;
    } else {
        g.edges[edge_idx].source = new_target;
    }

    rehash_graph(&mut g);
    g
}

/// Change a node's NodeKind to a compatible kind (weight 0.10).
fn replace_kind(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();
    let target = match random_node_id(&g, rng) {
        Some(id) => id,
        None => return g,
    };

    // Only replace with kinds that have simple (or no) payloads.
    let choice = rng.gen_range(0..4u8);
    let (new_kind, new_payload) = match choice {
        0 => (NodeKind::Prim, NodePayload::Prim { opcode: rng.gen_range(0x00..=0x09) }),
        1 => (NodeKind::Apply, NodePayload::Apply),
        2 => (NodeKind::Let, NodePayload::Let),
        _ => (NodeKind::Tuple, NodePayload::Tuple),
    };

    if let Some(node) = g.nodes.get_mut(&target) {
        node.kind = new_kind;
        node.payload = new_payload;
        node.id = compute_node_id(node);
    }

    // Re-insert with new ID if it changed.
    if let Some(node) = g.nodes.remove(&target) {
        let new_id = node.id;
        // Update edges that referenced old ID.
        for edge in &mut g.edges {
            if edge.source == target {
                edge.source = new_id;
            }
            if edge.target == target {
                edge.target = new_id;
            }
        }
        if g.root == target {
            g.root = new_id;
        }
        g.nodes.insert(new_id, node);
    }

    rehash_graph(&mut g);
    g
}

/// Change a Prim node's opcode (weight 0.10).
fn replace_prim(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();

    // Find Prim nodes.
    let prim_ids: Vec<NodeId> = g
        .nodes
        .iter()
        .filter(|(_, n)| n.kind == NodeKind::Prim)
        .map(|(id, _)| *id)
        .collect();

    if prim_ids.is_empty() {
        return g;
    }

    let target = prim_ids[rng.gen_range(0..prim_ids.len())];

    if let Some(node) = g.nodes.get_mut(&target) {
        let (opcode, arity) = random_prim_opcode(rng);
        node.payload = NodePayload::Prim { opcode };
        node.arity = arity;
        node.id = compute_node_id(node);
    }

    // Re-insert with potentially new ID.
    if let Some(node) = g.nodes.remove(&target) {
        let new_id = node.id;
        for edge in &mut g.edges {
            if edge.source == target {
                edge.source = new_id;
            }
            if edge.target == target {
                edge.target = new_id;
            }
        }
        if g.root == target {
            g.root = new_id;
        }
        g.nodes.insert(new_id, node);
    }

    rehash_graph(&mut g);
    g
}

/// Duplicate a small subgraph (weight 0.05).
fn duplicate_subgraph(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();
    let start = match random_node_id(&g, rng) {
        Some(id) => id,
        None => return g,
    };

    // Collect a small subgraph: the start node + direct successors (1-hop).
    let successors: Vec<NodeId> = g
        .edges
        .iter()
        .filter(|e| e.source == start)
        .map(|e| e.target)
        .filter(|id| g.nodes.contains_key(id))
        .collect();

    let mut id_map: BTreeMap<NodeId, NodeId> = BTreeMap::new();

    // Clone start node with perturbed hash.
    if let Some(orig) = g.nodes.get(&start) {
        let mut dup = orig.clone();
        // Perturb: change salt to produce a different hash.
        dup.salt = dup.salt.wrapping_add(1);
        dup.id = compute_node_id(&dup);
        id_map.insert(start, dup.id);
        g.nodes.insert(dup.id, dup);
    }

    // Clone each successor.
    for &succ_id in &successors {
        if let Some(orig) = graph.nodes.get(&succ_id) {
            let mut dup = orig.clone();
            dup.salt = dup.salt.wrapping_add(1);
            dup.id = compute_node_id(&dup);
            id_map.insert(succ_id, dup.id);
            g.nodes.insert(dup.id, dup);
        }
    }

    // Clone edges within the subgraph.
    let mut new_edges = vec![];
    for edge in &graph.edges {
        if let (Some(&new_src), Some(&new_tgt)) =
            (id_map.get(&edge.source), id_map.get(&edge.target))
        {
            new_edges.push(Edge {
                source: new_src,
                target: new_tgt,
                port: edge.port,
                label: edge.label,
            });
        }
    }
    g.edges.extend(new_edges);

    // Connect duplicated subgraph root to a random existing node.
    if let Some(&dup_root) = id_map.get(&start) {
        let ids = node_ids(&g);
        if !ids.is_empty() {
            let connect_to = ids[rng.gen_range(0..ids.len())];
            g.edges.push(Edge {
                source: dup_root,
                target: connect_to,
                port: 0,
                label: EdgeLabel::Argument,
            });
        }
    }

    rehash_graph(&mut g);
    g
}

/// Wrap a node in a Guard (predicate, body, fallback) (weight 0.05).
fn wrap_in_guard(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();
    let body_id = match random_node_id(&g, rng) {
        Some(id) => id,
        None => return g,
    };

    let type_sig = g
        .nodes
        .get(&body_id)
        .map(|n| n.type_sig)
        .unwrap_or(TypeId(0));

    // Create a trivial predicate node (Lit true).
    let mut pred_node = Node {
        id: NodeId(0),
        kind: NodeKind::Lit,
        type_sig,
        cost: CostTerm::Unit,
        arity: 0,
        resolution_depth: 2, salt: 0,
        payload: NodePayload::Lit {
            type_tag: 4, // Bool
            value: vec![1],
        },
    };
    pred_node.id = compute_node_id(&pred_node);

    // Create a fallback node (Lit 0).
    let mut fallback_node = Node {
        id: NodeId(0),
        kind: NodeKind::Lit,
        type_sig,
        cost: CostTerm::Unit,
        arity: 0,
        resolution_depth: 2, salt: 0,
        payload: NodePayload::Lit {
            type_tag: 0, // Int
            value: vec![0],
        },
    };
    fallback_node.id = compute_node_id(&fallback_node);

    // Create the Guard node.
    let mut guard_node = Node {
        id: NodeId(0),
        kind: NodeKind::Guard,
        type_sig,
        cost: CostTerm::Unit,
        arity: 3,
        resolution_depth: 2, salt: 0,
        payload: NodePayload::Guard {
            predicate_node: pred_node.id,
            body_node: body_id,
            fallback_node: fallback_node.id,
        },
    };
    guard_node.id = compute_node_id(&guard_node);

    let guard_id = guard_node.id;

    g.nodes.insert(pred_node.id, pred_node);
    g.nodes.insert(fallback_node.id, fallback_node);
    g.nodes.insert(guard_id, guard_node);

    // Redirect edges that targeted body_id to target the guard instead.
    for edge in &mut g.edges {
        if edge.target == body_id {
            edge.target = guard_id;
        }
    }

    // Add edges from guard to its children.
    g.edges.push(Edge {
        source: guard_id,
        target: body_id,
        port: 1,
        label: EdgeLabel::Continuation,
    });

    if g.root == body_id {
        g.root = guard_id;
    }

    rehash_graph(&mut g);
    g
}

/// Mutate a Lit node's value (weight 0.10).
fn mutate_literal(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();

    // Find Lit nodes.
    let lit_ids: Vec<NodeId> = g
        .nodes
        .iter()
        .filter(|(_, n)| n.kind == NodeKind::Lit)
        .map(|(id, _)| *id)
        .collect();

    if lit_ids.is_empty() {
        return g;
    }

    let target = lit_ids[rng.gen_range(0..lit_ids.len())];

    if let Some(node) = g.nodes.get_mut(&target) {
        if let NodePayload::Lit { type_tag, value } = &mut node.payload {
            match *type_tag {
                0x00 => {
                    // Int: perturb by a small delta or replace entirely.
                    if value.len() == 8 {
                        let current = i64::from_le_bytes(value[..8].try_into().unwrap());
                        let new_val = if rng.r#gen::<bool>() {
                            // Small perturbation.
                            current.wrapping_add(rng.gen_range(-5..=5))
                        } else {
                            // Random replacement.
                            rng.gen_range(-100..=100)
                        };
                        *value = new_val.to_le_bytes().to_vec();
                    }
                }
                0x04 => {
                    // Bool: flip.
                    if value.len() == 1 {
                        value[0] = 1 - value[0];
                    }
                }
                _ => {} // Other types: no-op for now.
            }
        }
        node.id = compute_node_id(node);
    }

    // Re-insert with potentially new ID.
    if let Some(node) = g.nodes.remove(&target) {
        let new_id = node.id;
        for edge in &mut g.edges {
            if edge.source == target {
                edge.source = new_id;
            }
            if edge.target == target {
                edge.target = new_id;
            }
        }
        if g.root == target {
            g.root = new_id;
        }
        g.nodes.insert(new_id, node);
    }

    rehash_graph(&mut g);
    g
}

/// Add a cost annotation to a node (weight 0.05).
fn annotate_cost(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();
    let target = match random_node_id(&g, rng) {
        Some(id) => id,
        None => return g,
    };

    if let Some(node) = g.nodes.get_mut(&target) {
        let choice = rng.gen_range(0..3u8);
        let cost = match choice {
            0 => CostBound::Zero,
            1 => CostBound::Constant(1),
            _ => CostBound::Constant(rng.gen_range(1..=100)),
        };
        node.cost = CostTerm::Annotated(cost);
    }

    rehash_graph(&mut g);
    g
}

// ---------------------------------------------------------------------------
// Structural mutation helpers
// ---------------------------------------------------------------------------

/// Global counter for generating unique nodes within mutations. Varies
/// `resolution_depth` so that structurally identical nodes get distinct
/// content-addressed IDs.
static MUTATION_UNIQUE_COUNTER: AtomicU8 = AtomicU8::new(100);

/// Build a Node with a unique ID by varying `resolution_depth`.
fn make_unique_node(
    kind: NodeKind,
    payload: NodePayload,
    type_sig: TypeId,
    arity: u8,
) -> Node {
    let depth = MUTATION_UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut node = Node {
        id: NodeId(0),
        kind,
        type_sig,
        cost: CostTerm::Unit,
        arity,
        resolution_depth: depth, salt: 0,
        payload,
    };
    node.id = compute_node_id(&node);
    node
}

/// Find all Fold nodes in the graph.
fn find_fold_nodes(graph: &SemanticGraph) -> Vec<NodeId> {
    graph
        .nodes
        .iter()
        .filter(|(_, n)| n.kind == NodeKind::Fold)
        .map(|(id, _)| *id)
        .collect()
}

/// Get the argument targets of a node, sorted by port.
fn argument_targets(graph: &SemanticGraph, node_id: NodeId) -> Vec<(u8, NodeId)> {
    let mut args: Vec<(u8, NodeId)> = graph
        .edges
        .iter()
        .filter(|e| e.source == node_id && e.label == EdgeLabel::Argument)
        .map(|e| (e.port, e.target))
        .collect();
    args.sort_by_key(|(port, _)| *port);
    args
}

/// Helper to re-insert a node after its ID changes, updating all edges.
fn reinsert_node(g: &mut SemanticGraph, old_id: NodeId, node: Node) {
    let new_id = node.id;
    for edge in &mut g.edges {
        if edge.source == old_id {
            edge.source = new_id;
        }
        if edge.target == old_id {
            edge.target = new_id;
        }
    }
    if g.root == old_id {
        g.root = new_id;
    }
    g.nodes.insert(new_id, node);
}

// ---------------------------------------------------------------------------
// Structural mutation operator 1: wrap_in_map (weight 0.08)
// ---------------------------------------------------------------------------

/// Find a Fold node and insert a Map node between its input collection and the
/// Fold. The Map applies a random arithmetic op to each element before folding.
///
/// Before: `Fold(base, step, collection)`
/// After:  `Fold(base, step, Map(collection, random_op))`
///
/// Creates programs like sum-of-squares: fold(0, add, map(input, mul))
fn wrap_in_map(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();
    let fold_ids = find_fold_nodes(&g);
    if fold_ids.is_empty() {
        return g;
    }

    let fold_id = fold_ids[rng.gen_range(0..fold_ids.len())];
    let type_sig = g.nodes.get(&fold_id).map(|n| n.type_sig).unwrap_or(TypeId(0));
    let args = argument_targets(&g, fold_id);

    // Determine the collection source: port 2 if present, else create a
    // placeholder Tuple (the interpreter falls back to positional input 0).
    let (collection_id, had_port2) = if let Some(&(_, col_id)) = args.iter().find(|(p, _)| *p == 2) {
        (col_id, true)
    } else {
        // No port 2: create a placeholder Tuple node.
        let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, type_sig, 0);
        let pid = placeholder.id;
        g.nodes.insert(pid, placeholder);
        (pid, false)
    };

    // Map function: random arithmetic op.
    let map_ops: [u8; 5] = [0x00, 0x02, 0x05, 0x06, 0x08]; // add, mul, neg, abs, max
    let map_opcode = map_ops[rng.gen_range(0..map_ops.len())];
    let map_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: map_opcode },
        type_sig,
        2,
    );
    let map_step_id = map_step.id;
    g.nodes.insert(map_step_id, map_step);

    // Map node: Prim with opcode 0x30.
    let map_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 },
        type_sig,
        2,
    );
    let map_id = map_node.id;
    g.nodes.insert(map_id, map_node);

    // Wire Map: port 0 = collection, port 1 = function.
    g.edges.push(Edge {
        source: map_id,
        target: collection_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    g.edges.push(Edge {
        source: map_id,
        target: map_step_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    // Rewire Fold's port 2 to point to the Map node instead.
    if had_port2 {
        for edge in &mut g.edges {
            if edge.source == fold_id && edge.port == 2 && edge.label == EdgeLabel::Argument {
                edge.target = map_id;
                break;
            }
        }
    } else {
        // Add a new port 2 edge from Fold to Map.
        g.edges.push(Edge {
            source: fold_id,
            target: map_id,
            port: 2,
            label: EdgeLabel::Argument,
        });
        // Update fold arity to 3 if needed.
        if let Some(fold_node) = g.nodes.get_mut(&fold_id) {
            if fold_node.arity < 3 {
                fold_node.arity = 3;
                fold_node.id = compute_node_id(fold_node);
                let node = g.nodes.remove(&fold_id).unwrap();
                reinsert_node(&mut g, fold_id, node);
            }
        }
    }

    rehash_graph(&mut g);
    g
}

// ---------------------------------------------------------------------------
// Structural mutation operator 2: wrap_in_filter (weight 0.08)
// ---------------------------------------------------------------------------

/// Find a Fold node and insert a Filter node between its input collection and
/// the Fold. The filter uses a random comparison against 0.
///
/// Before: `Fold(base, step, collection)`
/// After:  `Fold(base, step, Filter(collection, cmp_op))`
///
/// Creates programs like sum-of-positives: fold(0, add, filter(input, gt))
fn wrap_in_filter(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();
    let fold_ids = find_fold_nodes(&g);
    if fold_ids.is_empty() {
        return g;
    }

    let fold_id = fold_ids[rng.gen_range(0..fold_ids.len())];
    let type_sig = g.nodes.get(&fold_id).map(|n| n.type_sig).unwrap_or(TypeId(0));
    let args = argument_targets(&g, fold_id);

    // Determine the collection source.
    let (collection_id, had_port2) = if let Some(&(_, col_id)) = args.iter().find(|(p, _)| *p == 2) {
        (col_id, true)
    } else {
        let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, type_sig, 0);
        let pid = placeholder.id;
        g.nodes.insert(pid, placeholder);
        (pid, false)
    };

    // Filter predicate: random comparison opcode (0x20-0x25).
    // The interpreter compares each element against 0.
    let cmp_ops: [u8; 6] = [0x20, 0x21, 0x22, 0x23, 0x24, 0x25]; // eq, ne, lt, gt, le, ge
    let cmp_opcode = cmp_ops[rng.gen_range(0..cmp_ops.len())];
    let filter_pred = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: cmp_opcode },
        type_sig,
        2,
    );
    let filter_pred_id = filter_pred.id;
    g.nodes.insert(filter_pred_id, filter_pred);

    // Filter node: Prim with opcode 0x31.
    let filter_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x31 },
        type_sig,
        2,
    );
    let filter_id = filter_node.id;
    g.nodes.insert(filter_id, filter_node);

    // Wire Filter: port 0 = collection, port 1 = predicate.
    g.edges.push(Edge {
        source: filter_id,
        target: collection_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    g.edges.push(Edge {
        source: filter_id,
        target: filter_pred_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    // Rewire Fold's port 2 to point to the Filter node.
    if had_port2 {
        for edge in &mut g.edges {
            if edge.source == fold_id && edge.port == 2 && edge.label == EdgeLabel::Argument {
                edge.target = filter_id;
                break;
            }
        }
    } else {
        g.edges.push(Edge {
            source: fold_id,
            target: filter_id,
            port: 2,
            label: EdgeLabel::Argument,
        });
        if let Some(fold_node) = g.nodes.get_mut(&fold_id) {
            if fold_node.arity < 3 {
                fold_node.arity = 3;
                fold_node.id = compute_node_id(fold_node);
                let node = g.nodes.remove(&fold_id).unwrap();
                reinsert_node(&mut g, fold_id, node);
            }
        }
    }

    rehash_graph(&mut g);
    g
}

// ---------------------------------------------------------------------------
// Structural mutation operator 3: compose_stages (weight 0.06)
// ---------------------------------------------------------------------------

/// Take the current program's output and pipe it into a new Fold or Map stage.
///
/// Before: program produces output O (from root)
/// After:  `Fold(base, step, O)` or `Map(O, op)` becomes the new root
///
/// This chains stages: map -> fold, fold -> fold, etc.
fn compose_stages(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();
    let old_root = g.root;
    let type_sig = g.nodes.get(&old_root).map(|n| n.type_sig).unwrap_or(TypeId(0));

    if rng.r#gen::<bool>() {
        // Add a new Fold stage over the existing output.
        let fold_ops: [u8; 4] = [0x00, 0x02, 0x07, 0x08]; // add, mul, min, max
        let fold_opcode = fold_ops[rng.gen_range(0..fold_ops.len())];

        // Choose a semantically-matched base value.
        let base_value = match fold_opcode {
            0x00 => 0i64,   // add -> 0
            0x02 => 1,      // mul -> 1
            0x07 => i64::MAX, // min -> MAX
            0x08 => i64::MIN, // max -> MIN
            _ => 0,
        };

        let base_node = make_unique_node(
            NodeKind::Lit,
            NodePayload::Lit {
                type_tag: 0,
                value: base_value.to_le_bytes().to_vec(),
            },
            type_sig,
            0,
        );
        let base_id = base_node.id;
        g.nodes.insert(base_id, base_node);

        let step_node = make_unique_node(
            NodeKind::Prim,
            NodePayload::Prim { opcode: fold_opcode },
            type_sig,
            2,
        );
        let step_id = step_node.id;
        g.nodes.insert(step_id, step_node);

        let fold_node = make_unique_node(
            NodeKind::Fold,
            NodePayload::Fold { recursion_descriptor: vec![] },
            type_sig,
            3,
        );
        let fold_id = fold_node.id;
        g.nodes.insert(fold_id, fold_node);

        g.edges.push(Edge { source: fold_id, target: base_id, port: 0, label: EdgeLabel::Argument });
        g.edges.push(Edge { source: fold_id, target: step_id, port: 1, label: EdgeLabel::Argument });
        g.edges.push(Edge { source: fold_id, target: old_root, port: 2, label: EdgeLabel::Argument });

        g.root = fold_id;
    } else {
        // Add a new Map stage over the existing output.
        let map_ops: [u8; 5] = [0x00, 0x02, 0x05, 0x06, 0x08];
        let map_opcode = map_ops[rng.gen_range(0..map_ops.len())];

        let map_step = make_unique_node(
            NodeKind::Prim,
            NodePayload::Prim { opcode: map_opcode },
            type_sig,
            2,
        );
        let map_step_id = map_step.id;
        g.nodes.insert(map_step_id, map_step);

        let map_node = make_unique_node(
            NodeKind::Prim,
            NodePayload::Prim { opcode: 0x30 },
            type_sig,
            2,
        );
        let map_id = map_node.id;
        g.nodes.insert(map_id, map_node);

        g.edges.push(Edge { source: map_id, target: old_root, port: 0, label: EdgeLabel::Argument });
        g.edges.push(Edge { source: map_id, target: map_step_id, port: 1, label: EdgeLabel::Argument });

        g.root = map_id;
    }

    rehash_graph(&mut g);
    g
}

// ---------------------------------------------------------------------------
// Structural mutation operator 4: insert_zip (weight 0.06)
// ---------------------------------------------------------------------------

/// For programs with a Fold node, insert a Zip that pairs two inputs before
/// processing, then map a binary op over the pairs.
///
/// Before: `Fold(base, step, input_a)` (ignoring second input)
/// After:  `Fold(base, step, Map(Zip(input_a, input_b), pair_op))`
///
/// Creates dot product: fold(0, add, map(zip(a, b), mul))
fn insert_zip(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();
    let fold_ids = find_fold_nodes(&g);
    if fold_ids.is_empty() {
        return g;
    }

    let fold_id = fold_ids[rng.gen_range(0..fold_ids.len())];
    let type_sig = g.nodes.get(&fold_id).map(|n| n.type_sig).unwrap_or(TypeId(0));

    // Create two placeholder Tuple nodes for input_a and input_b.
    // The interpreter resolves empty Tuples to positional inputs
    // (BinderId(0xFFFF_0000) and BinderId(0xFFFF_0001)).
    let placeholder_a = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, type_sig, 0);
    let pa_id = placeholder_a.id;
    g.nodes.insert(pa_id, placeholder_a);

    let placeholder_b = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, type_sig, 0);
    let pb_id = placeholder_b.id;
    g.nodes.insert(pb_id, placeholder_b);

    // Zip node: Prim(0x32) with two collection arguments.
    let zip_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x32 },
        type_sig,
        2,
    );
    let zip_id = zip_node.id;
    g.nodes.insert(zip_id, zip_node);

    g.edges.push(Edge { source: zip_id, target: pa_id, port: 0, label: EdgeLabel::Argument });
    g.edges.push(Edge { source: zip_id, target: pb_id, port: 1, label: EdgeLabel::Argument });

    // Map node over the zipped pairs: map(zip_output, binary_op).
    let pair_ops: [u8; 4] = [0x00, 0x01, 0x02, 0x08]; // add, sub, mul, max
    let pair_opcode = pair_ops[rng.gen_range(0..pair_ops.len())];
    let pair_op = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: pair_opcode },
        type_sig,
        2,
    );
    let pair_op_id = pair_op.id;
    g.nodes.insert(pair_op_id, pair_op);

    let map_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 },
        type_sig,
        2,
    );
    let map_id = map_node.id;
    g.nodes.insert(map_id, map_node);

    g.edges.push(Edge { source: map_id, target: zip_id, port: 0, label: EdgeLabel::Argument });
    g.edges.push(Edge { source: map_id, target: pair_op_id, port: 1, label: EdgeLabel::Argument });

    // Rewire fold's port 2 (or add one) to point to the Map node.
    let args = argument_targets(&g, fold_id);
    let had_port2 = args.iter().any(|(p, _)| *p == 2);

    if had_port2 {
        for edge in &mut g.edges {
            if edge.source == fold_id && edge.port == 2 && edge.label == EdgeLabel::Argument {
                edge.target = map_id;
                break;
            }
        }
    } else {
        g.edges.push(Edge {
            source: fold_id,
            target: map_id,
            port: 2,
            label: EdgeLabel::Argument,
        });
        if let Some(fold_node) = g.nodes.get_mut(&fold_id) {
            if fold_node.arity < 3 {
                fold_node.arity = 3;
                fold_node.id = compute_node_id(fold_node);
                let node = g.nodes.remove(&fold_id).unwrap();
                reinsert_node(&mut g, fold_id, node);
            }
        }
    }

    rehash_graph(&mut g);
    g
}

// ---------------------------------------------------------------------------
// Structural mutation operator 5: swap_fold_op (weight 0.08)
// ---------------------------------------------------------------------------

/// Change a Fold's step operation AND adjust the base value to match.
///
/// Current `replace_prim` changes the opcode but not the base — this mutation
/// changes both together, which is the correlated change that evolution
/// can't discover via independent single mutations.
///
/// Matches:
/// - add -> base=0
/// - mul -> base=1
/// - max -> base=MIN_INT
/// - min -> base=MAX_INT
fn swap_fold_op(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();
    let fold_ids = find_fold_nodes(&g);
    if fold_ids.is_empty() {
        return g;
    }

    let fold_id = fold_ids[rng.gen_range(0..fold_ids.len())];
    let args = argument_targets(&g, fold_id);

    if args.len() < 2 {
        return g;
    }

    let base_id = args[0].1;
    let step_id = args[1].1;

    // Pick a new fold operation with a semantically correct base value.
    let ops: [(u8, i64); 5] = [
        (0x00, 0),          // add -> base=0
        (0x01, 0),          // sub -> base=0
        (0x02, 1),          // mul -> base=1
        (0x07, i64::MAX),   // min -> base=MAX
        (0x08, i64::MIN),   // max -> base=MIN
    ];
    let (new_opcode, new_base) = ops[rng.gen_range(0..ops.len())];

    // Replace the step function's opcode.
    if let Some(step_node) = g.nodes.remove(&step_id) {
        let mut new_step = step_node;
        new_step.payload = NodePayload::Prim { opcode: new_opcode };
        new_step.salt = MUTATION_UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed) as u64;
        new_step.id = compute_node_id(&new_step);
        reinsert_node(&mut g, step_id, new_step);
    }

    // Replace the base literal value.
    if let Some(base_node) = g.nodes.remove(&base_id) {
        let mut new_base_node = base_node;
        new_base_node.kind = NodeKind::Lit;
        new_base_node.payload = NodePayload::Lit {
            type_tag: 0,
            value: new_base.to_le_bytes().to_vec(),
        };
        new_base_node.salt = MUTATION_UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed) as u64;
        new_base_node.id = compute_node_id(&new_base_node);
        reinsert_node(&mut g, base_id, new_base_node);
    }

    rehash_graph(&mut g);
    g
}

// ---------------------------------------------------------------------------
// Structural mutation operator 6: add_guard_condition (weight 0.06)
// ---------------------------------------------------------------------------

/// Wrap a computation in a Guard node with a random comparison predicate.
///
/// Before: `expr`
/// After:  `Guard(Compare(input, constant, cmp_op), expr, fallback)`
///
/// The fallback is 0, and the predicate compares the first input against a
/// random small constant. This creates conditional logic.
fn add_guard_condition(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();
    let body_id = match random_node_id(&g, rng) {
        Some(id) => id,
        None => return g,
    };

    let type_sig = g.nodes.get(&body_id).map(|n| n.type_sig).unwrap_or(TypeId(0));

    // Create a comparison predicate: cmp_op(input_placeholder, constant).
    let cmp_ops: [u8; 4] = [0x22, 0x23, 0x24, 0x25]; // lt, gt, le, ge
    let cmp_opcode = cmp_ops[rng.gen_range(0..cmp_ops.len())];

    let cmp_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: cmp_opcode },
        type_sig,
        2,
    );
    let cmp_id = cmp_node.id;
    g.nodes.insert(cmp_id, cmp_node);

    // Constant to compare against.
    let constant_value: i64 = rng.gen_range(-10..=10);
    let const_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: constant_value.to_le_bytes().to_vec(),
        },
        type_sig,
        0,
    );
    let const_id = const_node.id;
    g.nodes.insert(const_id, const_node);

    // Input placeholder (empty Tuple resolves to positional input 0).
    let input_placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, type_sig, 0);
    let input_id = input_placeholder.id;
    g.nodes.insert(input_id, input_placeholder);

    // Wire comparison: cmp(input, constant).
    g.edges.push(Edge { source: cmp_id, target: input_id, port: 0, label: EdgeLabel::Argument });
    g.edges.push(Edge { source: cmp_id, target: const_id, port: 1, label: EdgeLabel::Argument });

    // Fallback: Lit(0).
    let fallback_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: 0i64.to_le_bytes().to_vec(),
        },
        type_sig,
        0,
    );
    let fallback_id = fallback_node.id;
    g.nodes.insert(fallback_id, fallback_node);

    // Guard node.
    let guard_node = make_unique_node(
        NodeKind::Guard,
        NodePayload::Guard {
            predicate_node: cmp_id,
            body_node: body_id,
            fallback_node: fallback_id,
        },
        type_sig,
        3,
    );
    let guard_id = guard_node.id;
    g.nodes.insert(guard_id, guard_node);

    // Redirect edges that targeted body_id to target the guard instead.
    for edge in &mut g.edges {
        if edge.target == body_id {
            edge.target = guard_id;
        }
    }

    // Add edge from guard to body.
    g.edges.push(Edge {
        source: guard_id,
        target: body_id,
        port: 1,
        label: EdgeLabel::Continuation,
    });

    if g.root == body_id {
        g.root = guard_id;
    }

    rehash_graph(&mut g);
    g
}

// ---------------------------------------------------------------------------
// Structural mutation operator 7: extract_to_ref (weight 0.03)
// ---------------------------------------------------------------------------

/// Take a non-root node and replace it with a Ref node pointing to a
/// synthetic fragment ID derived from the node's content hash.
///
/// This creates modular structure that the rest of the system can reuse.
/// The Ref won't resolve unless a matching fragment exists, but it still
/// exercises the modular composition pathway and can be useful if other
/// mutation operators later inline/replace it.
fn extract_to_ref(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let mut g = graph.clone();

    // Pick a non-root node.
    let candidates: Vec<NodeId> = g
        .nodes
        .keys()
        .copied()
        .filter(|&id| id != g.root)
        .collect();

    if candidates.is_empty() {
        return g;
    }

    let victim = candidates[rng.gen_range(0..candidates.len())];
    let type_sig = g.nodes.get(&victim).map(|n| n.type_sig).unwrap_or(TypeId(0));

    // Create a synthetic fragment ID from the victim node's hash.
    let mut frag_bytes = [0u8; 32];
    let victim_hash = victim.0.to_le_bytes();
    frag_bytes[..8].copy_from_slice(&victim_hash);
    // Add entropy so different extractions produce different fragment IDs.
    let entropy = rng.r#gen::<u64>();
    frag_bytes[8..16].copy_from_slice(&entropy.to_le_bytes());
    let fragment_id = FragmentId(frag_bytes);

    let ref_node = make_ref_node(fragment_id, type_sig);
    let new_id = ref_node.id;

    // Replace the victim with the Ref node.
    g.nodes.remove(&victim);
    g.nodes.insert(new_id, ref_node);

    // Update edges referencing the victim.
    for edge in &mut g.edges {
        if edge.source == victim {
            edge.source = new_id;
        }
        if edge.target == victim {
            edge.target = new_id;
        }
    }

    rehash_graph(&mut g);
    g
}

// ---------------------------------------------------------------------------
// Burst mutation for large programs
// ---------------------------------------------------------------------------

/// Apply multiple mutations in a single step for faster exploration of large
/// programs.
///
/// `burst_size` controls how many individual mutations are applied. For
/// auto-scaling, use `burst_size = min(10, graph.nodes.len() / 20)`.
pub fn mutate_burst(graph: &SemanticGraph, rng: &mut impl Rng, burst_size: usize) -> SemanticGraph {
    let count = burst_size.max(1);
    let mut g = graph.clone();
    for _ in 0..count {
        g = mutate(&g, rng);
    }
    g
}

/// Apply burst mutation with auto-scaled burst size based on program size.
///
/// burst_size = min(10, graph.nodes.len() / 20)
pub fn mutate_burst_auto(graph: &SemanticGraph, rng: &mut impl Rng) -> SemanticGraph {
    let burst_size = (graph.nodes.len() / 20).min(10).max(1);
    mutate_burst(graph, rng, burst_size)
}

// ---------------------------------------------------------------------------
// Ref-aware mutation
// ---------------------------------------------------------------------------

/// Apply a random mutation operator, including `insert_ref` when known
/// fragment IDs are available.
///
/// When `known_fragment_ids` is non-empty, a 5% weight is allocated to
/// `insert_ref` (replacing `annotate_cost`'s slot). Otherwise, falls back
/// to the standard `mutate`.
pub fn mutate_with_refs(
    graph: &SemanticGraph,
    rng: &mut impl Rng,
    known_fragment_ids: &[FragmentId],
) -> SemanticGraph {
    if known_fragment_ids.is_empty() {
        return mutate(graph, rng);
    }

    // Modified weight thresholds: structural operators + insert_ref
    // (same distribution as mutate() but extract_to_ref slot goes to insert_ref).
    let roll: f64 = rng.r#gen();
    if roll < 0.15 {
        insert_node(graph, rng)
    } else if roll < 0.30 {
        delete_node(graph, rng)
    } else if roll < 0.42 {
        rewire_edge(graph, rng)
    } else if roll < 0.46 {
        replace_kind(graph, rng)
    } else if roll < 0.54 {
        replace_prim(graph, rng)
    } else if roll < 0.62 {
        mutate_literal(graph, rng)
    } else if roll < 0.64 {
        duplicate_subgraph(graph, rng)
    } else if roll < 0.66 {
        wrap_in_guard(graph, rng)
    } else if roll < 0.73 {
        wrap_in_map(graph, rng)
    } else if roll < 0.80 {
        wrap_in_filter(graph, rng)
    } else if roll < 0.84 {
        compose_stages(graph, rng)
    } else if roll < 0.88 {
        insert_zip(graph, rng)
    } else if roll < 0.95 {
        swap_fold_op(graph, rng)
    } else if roll < 0.97 {
        add_guard_condition(graph, rng)
    } else {
        // 3% chance of insert_ref.
        insert_ref(graph, rng, known_fragment_ids)
    }
}

/// Replace a non-root node with a Ref node pointing to a known fragment.
///
/// The Ref node inherits the replaced node's incoming edges and keeps its
/// outgoing edges as arguments (so the referenced fragment receives the
/// same inputs the replaced subgraph would have).
pub fn insert_ref(
    graph: &SemanticGraph,
    rng: &mut impl Rng,
    known_fragment_ids: &[FragmentId],
) -> SemanticGraph {
    let mut g = graph.clone();

    if known_fragment_ids.is_empty() {
        return g;
    }

    // Pick a random non-root node to replace.
    let candidates: Vec<NodeId> = g
        .nodes
        .keys()
        .copied()
        .filter(|&id| id != g.root)
        .collect();

    if candidates.is_empty() {
        // If only root exists, replace the root itself.
        let fragment_id = known_fragment_ids[rng.gen_range(0..known_fragment_ids.len())];
        let type_sig = g
            .nodes
            .get(&g.root)
            .map(|n| n.type_sig)
            .unwrap_or(TypeId(0));

        let ref_node = make_ref_node(fragment_id, type_sig);
        let new_root = ref_node.id;

        // Redirect edges from old root to new root.
        for edge in &mut g.edges {
            if edge.source == g.root {
                edge.source = new_root;
            }
            if edge.target == g.root {
                edge.target = new_root;
            }
        }

        g.nodes.remove(&g.root);
        g.nodes.insert(new_root, ref_node);
        g.root = new_root;
        rehash_graph(&mut g);
        return g;
    }

    let victim = candidates[rng.gen_range(0..candidates.len())];
    let fragment_id = known_fragment_ids[rng.gen_range(0..known_fragment_ids.len())];

    let type_sig = g
        .nodes
        .get(&victim)
        .map(|n| n.type_sig)
        .unwrap_or(TypeId(0));

    let ref_node = make_ref_node(fragment_id, type_sig);
    let new_id = ref_node.id;

    // Replace the victim node with the Ref node.
    g.nodes.remove(&victim);
    g.nodes.insert(new_id, ref_node);

    // Update edges referencing the victim.
    for edge in &mut g.edges {
        if edge.source == victim {
            edge.source = new_id;
        }
        if edge.target == victim {
            edge.target = new_id;
        }
    }

    rehash_graph(&mut g);
    g
}

/// Create a Ref node pointing to the given fragment.
fn make_ref_node(fragment_id: FragmentId, type_sig: TypeId) -> Node {
    let mut node = Node {
        id: NodeId(0),
        kind: NodeKind::Ref,
        type_sig,
        cost: CostTerm::Unit,
        arity: 0,
        resolution_depth: 2, salt: 0,
        payload: NodePayload::Ref { fragment_id },
    };
    node.id = compute_node_id(&node);
    node
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use iris_types::graph::Resolution;
    use iris_types::hash::SemanticHash;
    use iris_types::types::{BoundVar, LIAAtom, LIAFormula, LIATerm, PrimType, TypeDef, TypeEnv};

    /// Build a minimal graph with a single Lit node using a refined type.
    fn graph_with_refined_type() -> (SemanticGraph, NodeId) {
        let int_id = TypeId(1);
        let refined_id = TypeId(2);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));
        types.insert(
            refined_id,
            TypeDef::Refined(
                int_id,
                LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Le(
                    LIATerm::Var(BoundVar(0)),
                    LIATerm::Const(0),
                )))),
            ),
        );

        let node_id = NodeId(100);
        let node = Node {
            id: node_id,
            kind: NodeKind::Lit,
            type_sig: refined_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0,
                value: vec![42, 0, 0, 0],
            },
        };

        let mut nodes = HashMap::new();
        nodes.insert(node_id, node);

        let graph = SemanticGraph {
            root: node_id,
            nodes,
            edges: vec![],
            type_env: TypeEnv { types },
            cost: iris_types::cost::CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        (graph, node_id)
    }

    #[test]
    fn counterexample_to_test_case_produces_inputs() {
        let (graph, node_id) = graph_with_refined_type();

        // Create a diagnosis with a counterexample.
        let mut counterexample = HashMap::new();
        counterexample.insert(BoundVar(0), -5i64);

        let diag = ProofFailureDiagnosis {
            node_id,
            node_kind: NodeKind::Lit,
            error: iris_bootstrap::syntax::kernel::CheckError::MalformedGraph {
                reason: "test".to_string(),
            },
            suggestion: Some(MutationHint::FixTypeSignature(TypeId(1), TypeId(2))),
            counterexample: Some(counterexample),
        };

        let test_case = counterexample_to_test_case(&diag, &graph);
        assert!(
            test_case.is_some(),
            "should produce a test case from counterexample"
        );

        let tc = test_case.unwrap();
        assert_eq!(tc.inputs.len(), 1);
        assert_eq!(tc.inputs[0], Value::Int(-5));
        // Expected output is None (negative example / observational test).
        assert!(tc.expected_output.is_none());
    }

    #[test]
    fn counterexample_to_test_case_no_counterexample() {
        let (graph, node_id) = graph_with_refined_type();

        let diag = ProofFailureDiagnosis {
            node_id,
            node_kind: NodeKind::Lit,
            error: iris_bootstrap::syntax::kernel::CheckError::MalformedGraph {
                reason: "test".to_string(),
            },
            suggestion: None,
            counterexample: None,
        };

        let test_case = counterexample_to_test_case(&diag, &graph);
        assert!(
            test_case.is_none(),
            "no counterexample should produce no test case"
        );
    }

    #[test]
    fn counterexample_to_test_case_empty_counterexample() {
        let (graph, node_id) = graph_with_refined_type();

        let diag = ProofFailureDiagnosis {
            node_id,
            node_kind: NodeKind::Lit,
            error: iris_bootstrap::syntax::kernel::CheckError::MalformedGraph {
                reason: "test".to_string(),
            },
            suggestion: None,
            counterexample: Some(HashMap::new()),
        };

        let test_case = counterexample_to_test_case(&diag, &graph);
        assert!(
            test_case.is_none(),
            "empty counterexample should produce no test case"
        );
    }

    #[test]
    fn counterexample_to_test_case_multi_var() {
        let (graph, node_id) = graph_with_refined_type();

        let mut counterexample = HashMap::new();
        counterexample.insert(BoundVar(0), 3i64);
        counterexample.insert(BoundVar(1), -7i64);

        let diag = ProofFailureDiagnosis {
            node_id,
            node_kind: NodeKind::Lit,
            error: iris_bootstrap::syntax::kernel::CheckError::MalformedGraph {
                reason: "test".to_string(),
            },
            suggestion: None,
            counterexample: Some(counterexample),
        };

        let test_case = counterexample_to_test_case(&diag, &graph);
        assert!(test_case.is_some());
        let tc = test_case.unwrap();
        // Sorted by BoundVar index: BoundVar(0)=3, BoundVar(1)=-7
        assert_eq!(tc.inputs.len(), 2);
        assert_eq!(tc.inputs[0], Value::Int(3));
        assert_eq!(tc.inputs[1], Value::Int(-7));
    }
}
