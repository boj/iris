use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};

use rand::Rng;

use iris_types::cost::{CostBound, CostTerm};
use iris_types::fragment::{Boundary, Fragment, FragmentId, FragmentMeta};
use iris_types::graph::{Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph};
use iris_types::hash::{compute_fragment_id, compute_node_id, SemanticHash};
use iris_types::types::{PrimType, TypeDef, TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Custom seed strategy (thread-local override for self-improvement)
// ---------------------------------------------------------------------------

/// Serializable seed strategy weights for thread-local override.
#[derive(Clone)]
struct SeedStrategyWeights {
    /// Cumulative thresholds: (seed_type_index, cumulative_weight).
    thresholds: Vec<(usize, f64)>,
}

thread_local! {
    /// Thread-local override for seed type selection.
    static CUSTOM_SEED_STRATEGY: RefCell<Option<SeedStrategyWeights>> = RefCell::new(None);
}

/// Install custom seed strategy weights for the current thread.
///
/// Called by the self-improvement module to evaluate alternative seed
/// distributions.
pub fn set_custom_seed_strategy(strategy: &crate::self_improve::SeedStrategy) {
    let thresholds = strategy.to_cumulative_thresholds();
    CUSTOM_SEED_STRATEGY.with(|s| {
        *s.borrow_mut() = Some(SeedStrategyWeights { thresholds });
    });
}

/// Clear custom seed strategy, reverting to the hardcoded distribution.
pub fn clear_custom_seed_strategy() {
    CUSTOM_SEED_STRATEGY.with(|s| {
        *s.borrow_mut() = None;
    });
}

/// Select a seed type index using the custom strategy, if installed.
///
/// Returns `Some(index)` if a custom strategy is active, `None` otherwise.
/// The index maps to the `SEED_TYPE_NAMES` array in `self_improve.rs`:
///   0=arithmetic, 1=fold, 2=identity, 3=map, 4=zip_fold, 5=map_fold,
///   6=filter_fold, 7=zip_map_fold, 8=comparison_fold, 9=stateful_fold,
///   10=conditional_fold, 11=iterate, 12=pairwise_fold
pub fn custom_seed_type(rng: &mut impl Rng) -> Option<usize> {
    CUSTOM_SEED_STRATEGY.with(|s| {
        let guard = s.borrow();
        guard.as_ref().map(|strategy| {
            let roll: f64 = rng.r#gen();
            for &(idx, threshold) in &strategy.thresholds {
                if roll < threshold {
                    return idx;
                }
            }
            strategy
                .thresholds
                .last()
                .map(|&(idx, _)| idx)
                .unwrap_or(0)
        })
    })
}

/// Generate a seed fragment based on seed type index.
///
/// Maps the index from `custom_seed_type` to the appropriate seed generator.
pub fn generate_seed_by_type(seed_type: usize, rng: &mut impl Rng) -> Fragment {
    match seed_type {
        0 => random_arithmetic_program(rng, 2, 2),
        1 => random_fold_program(rng),
        2 => identity_program(),
        3 => random_map_program(rng),
        4 => random_zip_fold_program(rng),
        5 => random_map_fold_program(rng),
        6 => random_filter_fold_program(rng),
        7 => random_zip_map_fold_program(rng),
        8 => random_comparison_fold_program(rng),
        9 => random_stateful_fold_program(rng),
        10 => random_conditional_fold_program(rng),
        11 => random_iterate_program(rng),
        12 => random_pairwise_fold_program(rng),
        _ => random_map_cmp_fold_program(rng),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a minimal TypeEnv with Int type registered.
fn int_type_env() -> (TypeEnv, TypeId) {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = iris_types::hash::compute_type_id(&int_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    (TypeEnv { types }, int_id)
}

/// Build a Node with auto-computed ID.
fn make_node(kind: NodeKind, payload: NodePayload, type_sig: TypeId, arity: u8) -> Node {
    let mut node = Node {
        id: NodeId(0),
        kind,
        type_sig,
        cost: CostTerm::Unit,
        arity,
        resolution_depth: 2, salt: 0,
        payload,
    };
    node.id = compute_node_id(&node);
    node
}

/// Global counter for generating unique nodes. Varies `resolution_depth` so
/// that structurally identical nodes get distinct content-addressed IDs.
use std::sync::atomic::{AtomicU8, Ordering};
static UNIQUE_COUNTER: AtomicU8 = AtomicU8::new(0);

/// Build a Node with a unique ID by varying `resolution_depth`.
///
/// Used by large seed generators where many structurally similar nodes would
/// otherwise hash-collide in the BTreeMap.
fn make_unique_node(kind: NodeKind, payload: NodePayload, type_sig: TypeId, arity: u8) -> Node {
    let depth = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
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

/// Compute semantic hash from nodes and edges.
fn compute_hash(nodes: &HashMap<NodeId, Node>, edges: &[Edge]) -> SemanticHash {
    let mut hasher = blake3::Hasher::new();
    let mut sorted_nids: Vec<_> = nodes.keys().collect();
    sorted_nids.sort();
    for nid in sorted_nids {
        hasher.update(&nid.0.to_le_bytes());
    }
    for edge in edges {
        hasher.update(&edge.source.0.to_le_bytes());
        hasher.update(&edge.target.0.to_le_bytes());
        hasher.update(&[edge.port, edge.label as u8]);
    }
    SemanticHash(*hasher.finalize().as_bytes())
}

/// Wrap a SemanticGraph in a Fragment.
fn graph_to_fragment(graph: SemanticGraph) -> Fragment {
    let boundary = Boundary {
        inputs: vec![],
        outputs: vec![(graph.root, graph.nodes[&graph.root].type_sig)],
    };
    let type_env = graph.type_env.clone();

    let mut fragment = Fragment {
        id: FragmentId([0; 32]),
        graph,
        boundary,
        type_env,
        imports: vec![],
        metadata: FragmentMeta {
            name: None,
            created_at: 0,
            generation: 0,
            lineage_hash: 0,
        },
        proof: None,
        contracts: Default::default(),    };
    fragment.id = compute_fragment_id(&fragment);
    fragment
}

// ---------------------------------------------------------------------------
// Seed generators
// ---------------------------------------------------------------------------

/// Generate a random arithmetic program tree of the given depth and width.
///
/// Produces a tree of Prim nodes (arithmetic opcodes 0x00-0x09) with Lit
/// leaves.
pub fn random_arithmetic_program(rng: &mut impl Rng, depth: usize, width: usize) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let root = build_arith_tree(rng, int_id, depth, width, &mut nodes, &mut edges);

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

/// Recursively build an arithmetic tree.
fn build_arith_tree(
    rng: &mut impl Rng,
    int_id: TypeId,
    depth: usize,
    width: usize,
    nodes: &mut HashMap<NodeId, Node>,
    edges: &mut Vec<Edge>,
) -> NodeId {
    if depth == 0 {
        // Leaf: literal integer.
        let value = rng.gen_range(-10i64..=10);
        let node = make_node(
            NodeKind::Lit,
            NodePayload::Lit {
                type_tag: 0, // Int
                value: value.to_le_bytes().to_vec(),
            },
            int_id,
            0,
        );
        let id = node.id;
        nodes.insert(id, node);
        return id;
    }

    // Internal: arithmetic Prim node.
    let opcode = rng.gen_range(0x00..=0x04u8); // add, sub, mul, div, mod
    let arity = width.min(4).max(2) as u8;
    let node = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode },
        int_id,
        arity,
    );
    let parent_id = node.id;
    nodes.insert(parent_id, node);

    for port in 0..arity {
        let child = build_arith_tree(rng, int_id, depth - 1, width, nodes, edges);
        edges.push(Edge {
            source: parent_id,
            target: child,
            port,
            label: EdgeLabel::Argument,
        });
    }

    parent_id
}

/// Generate a simple fold program: fold (+) 0 over a list.
pub fn random_fold_program(rng: &mut impl Rng) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Base case: random literal (evolution must discover that 0 is correct
    // for sum problems).
    let base_value = rng.gen_range(-5i64..=5);
    let base_node = make_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: base_value.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    // Step function: random arithmetic op (sub, mul, or max — NOT add,
    // so evolution must discover the correct opcode for sum problems).
    let step_opcode = [0x01u8, 0x02, 0x08][rng.gen_range(0..3)];
    let step_node = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: step_opcode },
        int_id,
        2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // Fold node
    let fold_node = make_node(
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![],
        },
        int_id,
        2,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);

    // Edges: fold -> base (port 0), fold -> step (port 1)
    edges.push(Edge {
        source: fold_id,
        target: base_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: fold_id,
        target: step_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

/// Generate a higher-order pipeline seed: map(collection, op).
///
/// Produces a program shaped like:
///   Prim(map) → [input_collection, Prim(random_arith_op)]
///
/// This gives evolution a starting point for discovering map-based
/// transformations and multi-phase algorithms like dot product.
pub fn random_map_program(rng: &mut impl Rng) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // The collection comes from positional input 0 at runtime,
    // but we need a placeholder node so the graph is well-formed.
    // Use a Tuple literal as the placeholder (empty — will be
    // overridden by the input binding in the interpreter).
    let placeholder_node = make_node(
        NodeKind::Tuple,
        NodePayload::Tuple,
        int_id,
        0,
    );
    let placeholder_id = placeholder_node.id;
    nodes.insert(placeholder_id, placeholder_node);

    // Step function: random arithmetic op.
    let step_opcode = [0x00u8, 0x01, 0x02][rng.gen_range(0..3)];
    let step_node = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: step_opcode },
        int_id,
        2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    // Map node.
    let map_node = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 }, // map
        int_id,
        2,
    );
    let map_id = map_node.id;
    nodes.insert(map_id, map_node);

    // Edges: map -> placeholder_collection (port 0), map -> step_fn (port 1)
    edges.push(Edge {
        source: map_id,
        target: placeholder_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: map_id,
        target: step_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: map_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

/// Generate a zip+fold pipeline seed.
///
/// Produces a program shaped like:
///   Fold(base=Lit(0), step=Prim(add))
///     with the collection being: Map(Zip(input0, input1), Prim(mul))
///
/// This is close to a dot product skeleton and gives evolution the
/// building blocks to discover zip-map-fold compositions.
pub fn random_zip_fold_program(rng: &mut impl Rng) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Two placeholder tuples for the zip inputs.
    let placeholder_a = make_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let placeholder_b = make_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let pa_id = placeholder_a.id;
    // Since both placeholders are identical (same content-hash), we need to
    // differentiate them.  Use a Lit(0) for the second input.
    let placeholder_b_node = make_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: 0i64.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let pb_id = placeholder_b_node.id;
    nodes.insert(pa_id, placeholder_a);
    nodes.insert(pb_id, placeholder_b_node);

    // Zip node.
    let zip_node = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x32 }, // zip
        int_id,
        2,
    );
    let zip_id = zip_node.id;
    nodes.insert(zip_id, zip_node);
    edges.push(Edge {
        source: zip_id,
        target: pa_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: zip_id,
        target: pb_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    // Mul step for map.
    let mul_opcode = [0x00u8, 0x02][rng.gen_range(0..2)]; // add or mul
    let mul_step = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: mul_opcode },
        int_id,
        2,
    );
    let mul_id = mul_step.id;
    nodes.insert(mul_id, mul_step);

    // Map node: map(zipped, mul)
    let map_node = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 }, // map
        int_id,
        2,
    );
    let map_id = map_node.id;
    nodes.insert(map_id, map_node);
    edges.push(Edge {
        source: map_id,
        target: zip_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: map_id,
        target: mul_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    // Fold base: random small literal.
    let base_value = rng.gen_range(-3i64..=3);
    let base_node = make_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: base_value.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    // Fold step: random arithmetic op.
    let fold_step_opcode = [0x00u8, 0x01, 0x02][rng.gen_range(0..3)];
    let fold_step = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: fold_step_opcode },
        int_id,
        2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    // Fold node.
    let fold_node = make_node(
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![],
        },
        int_id,
        2,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge {
        source: fold_id,
        target: base_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: fold_id,
        target: fold_step_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    let _ = placeholder_b; // suppress unused warning

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

/// Recursively build an arithmetic tree with unique node IDs.
///
/// Same as `build_arith_tree` but uses `make_unique_node` to guarantee
/// distinct NodeIds even for structurally identical sub-trees.
fn build_unique_arith_tree(
    rng: &mut impl Rng,
    int_id: TypeId,
    depth: usize,
    width: usize,
    nodes: &mut HashMap<NodeId, Node>,
    edges: &mut Vec<Edge>,
) -> NodeId {
    if depth == 0 {
        let value = rng.gen_range(-10i64..=10);
        let node = make_unique_node(
            NodeKind::Lit,
            NodePayload::Lit {
                type_tag: 0,
                value: value.to_le_bytes().to_vec(),
            },
            int_id,
            0,
        );
        let id = node.id;
        nodes.insert(id, node);
        return id;
    }

    let opcode = rng.gen_range(0x00..=0x04u8);
    let arity = width.min(4).max(2) as u8;
    let node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode },
        int_id,
        arity,
    );
    let parent_id = node.id;
    nodes.insert(parent_id, node);

    for port in 0..arity {
        let child = build_unique_arith_tree(rng, int_id, depth - 1, width, nodes, edges);
        edges.push(Edge {
            source: parent_id,
            target: child,
            port,
            label: EdgeLabel::Argument,
        });
    }

    parent_id
}

// ---------------------------------------------------------------------------
// Large seed generators (scaling support)
// ---------------------------------------------------------------------------

/// Generate a pipeline of map/filter/fold stages.
///
/// Each stage is a higher-order operation (map, filter, or fold) with an
/// arithmetic body of `body_depth` levels. At `stages=5` with default body
/// depth, produces ~50-250 nodes.
pub fn random_pipeline_program(rng: &mut impl Rng, stages: usize) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Start with a placeholder tuple (the input collection).
    let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let mut current_root = placeholder.id;
    nodes.insert(placeholder.id, placeholder);

    for stage_idx in 0..stages {
        // Each stage is one of: map, filter, fold.
        let stage_kind = rng.gen_range(0..3u8);

        // Build the body function as a small arithmetic tree (2-3 levels deep).
        let body_depth = rng.gen_range(2..=3);
        let body_width = rng.gen_range(2..=3);
        let body_root = build_unique_arith_tree(rng, int_id, body_depth, body_width, &mut nodes, &mut edges);

        match stage_kind {
            0 => {
                // Map stage.
                let map_node = make_unique_node(
                    NodeKind::Prim,
                    NodePayload::Prim { opcode: 0x30 },
                    int_id,
                    2,
                );
                let map_id = map_node.id;
                nodes.insert(map_id, map_node);
                edges.push(Edge {
                    source: map_id,
                    target: current_root,
                    port: 0,
                    label: EdgeLabel::Argument,
                });
                edges.push(Edge {
                    source: map_id,
                    target: body_root,
                    port: 1,
                    label: EdgeLabel::Argument,
                });
                current_root = map_id;
            }
            1 => {
                // Filter stage (uses a comparison as predicate).
                let filter_node = make_unique_node(
                    NodeKind::Prim,
                    NodePayload::Prim { opcode: 0x31 },
                    int_id,
                    2,
                );
                let filter_id = filter_node.id;
                nodes.insert(filter_id, filter_node);
                edges.push(Edge {
                    source: filter_id,
                    target: current_root,
                    port: 0,
                    label: EdgeLabel::Argument,
                });
                edges.push(Edge {
                    source: filter_id,
                    target: body_root,
                    port: 1,
                    label: EdgeLabel::Argument,
                });
                current_root = filter_id;
            }
            _ => {
                // Fold stage.
                let base_value = rng.gen_range(-5i64..=5);
                let base_node = make_unique_node(
                    NodeKind::Lit,
                    NodePayload::Lit {
                        type_tag: 0,
                        value: base_value.to_le_bytes().to_vec(),
                    },
                    int_id,
                    0,
                );
                let base_id = base_node.id;
                nodes.insert(base_id, base_node);

                let fold_node = make_unique_node(
                    NodeKind::Fold,
                    NodePayload::Fold {
                        recursion_descriptor: vec![],
                    },
                    int_id,
                    2,
                );
                let fold_id = fold_node.id;
                nodes.insert(fold_id, fold_node);
                edges.push(Edge {
                    source: fold_id,
                    target: base_id,
                    port: 0,
                    label: EdgeLabel::Argument,
                });
                edges.push(Edge {
                    source: fold_id,
                    target: body_root,
                    port: 1,
                    label: EdgeLabel::Argument,
                });
                current_root = fold_id;

                // If not the last stage, wrap result back into a tuple so
                // subsequent pipeline stages have a collection to process.
                if stage_idx < stages - 1 {
                    let wrap_node = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 1);
                    let wrap_id = wrap_node.id;
                    nodes.insert(wrap_id, wrap_node);
                    edges.push(Edge {
                        source: wrap_id,
                        target: current_root,
                        port: 0,
                        label: EdgeLabel::Argument,
                    });
                    current_root = wrap_id;
                }
            }
        }
    }

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: current_root,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

/// Generate a stateful loop program: a Fold with state_get/state_set in the body.
///
/// Simulates Daimon's cognitive loop pattern. Each iteration body is 10-20
/// nodes, using state opcodes (0x50-0x55) to read and write state.
pub fn random_stateful_loop(rng: &mut impl Rng, iterations: usize) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Build a body function that reads state, computes, and writes state.
    // The body is replicated `iterations` times as nested Fold iterations
    // (via a deeper arithmetic body).

    // State key for the loop counter.
    let key_bytes = b"counter\0".to_vec();
    let key_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x05, // Bytes
            value: key_bytes,
        },
        int_id,
        0,
    );
    let key_id = key_node.id;
    nodes.insert(key_id, key_node);

    // Build the loop body: state_get -> arithmetic -> state_set
    // Repeat this pattern `iterations` times chained together.
    let mut current_body_root = {
        // Initial state_get
        let state_get = make_unique_node(
            NodeKind::Prim,
            NodePayload::Prim { opcode: 0x50 }, // state_get
            int_id,
            1,
        );
        let get_id = state_get.id;
        nodes.insert(get_id, state_get);
        edges.push(Edge {
            source: get_id,
            target: key_id,
            port: 0,
            label: EdgeLabel::Argument,
        });
        get_id
    };

    for iter in 0..iterations {
        // Arithmetic computation on the current value.
        let body_depth = rng.gen_range(1..=2);
        let body_width = 2;
        let arith_root = build_unique_arith_tree(rng, int_id, body_depth, body_width, &mut nodes, &mut edges);

        // Connect arithmetic to previous result.
        let combine_opcode = [0x00u8, 0x01, 0x02][rng.gen_range(0..3)];
        let combine_node = make_unique_node(
            NodeKind::Prim,
            NodePayload::Prim { opcode: combine_opcode },
            int_id,
            2,
        );
        let combine_id = combine_node.id;
        nodes.insert(combine_id, combine_node);
        edges.push(Edge {
            source: combine_id,
            target: current_body_root,
            port: 0,
            label: EdgeLabel::Argument,
        });
        edges.push(Edge {
            source: combine_id,
            target: arith_root,
            port: 1,
            label: EdgeLabel::Argument,
        });

        // State key for this iteration (differentiated to avoid hash collisions).
        let iter_key_bytes = format!("state_{}\0", iter).into_bytes();
        let iter_key_node = make_unique_node(
            NodeKind::Lit,
            NodePayload::Lit {
                type_tag: 0x05,
                value: iter_key_bytes,
            },
            int_id,
            0,
        );
        let iter_key_id = iter_key_node.id;
        nodes.insert(iter_key_id, iter_key_node);

        // state_set with computed value.
        let state_set = make_unique_node(
            NodeKind::Prim,
            NodePayload::Prim { opcode: 0x51 }, // state_set
            int_id,
            2,
        );
        let set_id = state_set.id;
        nodes.insert(set_id, state_set);
        edges.push(Edge {
            source: set_id,
            target: iter_key_id,
            port: 0,
            label: EdgeLabel::Argument,
        });
        edges.push(Edge {
            source: set_id,
            target: combine_id,
            port: 1,
            label: EdgeLabel::Argument,
        });

        current_body_root = set_id;
    }

    // Wrap everything in a Fold as the outer driver.
    let base_value = rng.gen_range(0i64..=10);
    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: base_value.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![],
        },
        int_id,
        2,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge {
        source: fold_id,
        target: base_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: fold_id,
        target: current_body_root,
        port: 1,
        label: EdgeLabel::Argument,
    });

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

/// Generate a modular program composed of multiple sub-fragments connected
/// via internal references.
///
/// Creates `num_modules` sub-graphs (each 20-50 nodes), then composes them
/// by connecting each module's output to the next module's input. This
/// produces programs in the 100-1000+ node range depending on `num_modules`.
pub fn random_modular_program(rng: &mut impl Rng, num_modules: usize) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let mut module_roots: Vec<NodeId> = Vec::with_capacity(num_modules);

    for _module_idx in 0..num_modules {
        // Each module is a small arithmetic + higher-order sub-graph.
        let module_depth = rng.gen_range(2..=4);
        let module_width = rng.gen_range(2..=3);

        // Build the core arithmetic tree for this module.
        let arith_root = build_unique_arith_tree(
            rng, int_id, module_depth, module_width, &mut nodes, &mut edges,
        );

        // Optionally wrap in a higher-order operation.
        let wrap_kind = rng.gen_range(0..4u8);
        let module_root = match wrap_kind {
            0 => {
                // Wrap in map.
                let map_node = make_unique_node(
                    NodeKind::Prim,
                    NodePayload::Prim { opcode: 0x30 },
                    int_id,
                    2,
                );
                let map_id = map_node.id;
                nodes.insert(map_id, map_node);

                // If we have a previous module, use its output as collection.
                if let Some(&prev_root) = module_roots.last() {
                    edges.push(Edge {
                        source: map_id,
                        target: prev_root,
                        port: 0,
                        label: EdgeLabel::Argument,
                    });
                } else {
                    // First module: use a placeholder.
                    let ph = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
                    let ph_id = ph.id;
                    nodes.insert(ph_id, ph);
                    edges.push(Edge {
                        source: map_id,
                        target: ph_id,
                        port: 0,
                        label: EdgeLabel::Argument,
                    });
                }
                edges.push(Edge {
                    source: map_id,
                    target: arith_root,
                    port: 1,
                    label: EdgeLabel::Argument,
                });
                map_id
            }
            1 => {
                // Wrap in fold.
                let base_val = rng.gen_range(-5i64..=5);
                let base = make_unique_node(
                    NodeKind::Lit,
                    NodePayload::Lit {
                        type_tag: 0,
                        value: base_val.to_le_bytes().to_vec(),
                    },
                    int_id,
                    0,
                );
                let base_id = base.id;
                nodes.insert(base_id, base);

                let fold = make_unique_node(
                    NodeKind::Fold,
                    NodePayload::Fold {
                        recursion_descriptor: vec![],
                    },
                    int_id,
                    2,
                );
                let fold_id = fold.id;
                nodes.insert(fold_id, fold);
                edges.push(Edge {
                    source: fold_id,
                    target: base_id,
                    port: 0,
                    label: EdgeLabel::Argument,
                });
                edges.push(Edge {
                    source: fold_id,
                    target: arith_root,
                    port: 1,
                    label: EdgeLabel::Argument,
                });
                fold_id
            }
            _ => {
                // Use the arithmetic tree directly; connect to previous module
                // if available.
                if let Some(&prev_root) = module_roots.last() {
                    let combine_opcode = [0x00u8, 0x01, 0x02][rng.gen_range(0..3)];
                    let combine = make_unique_node(
                        NodeKind::Prim,
                        NodePayload::Prim { opcode: combine_opcode },
                        int_id,
                        2,
                    );
                    let combine_id = combine.id;
                    nodes.insert(combine_id, combine);
                    edges.push(Edge {
                        source: combine_id,
                        target: prev_root,
                        port: 0,
                        label: EdgeLabel::Argument,
                    });
                    edges.push(Edge {
                        source: combine_id,
                        target: arith_root,
                        port: 1,
                        label: EdgeLabel::Argument,
                    });
                    combine_id
                } else {
                    arith_root
                }
            }
        };

        module_roots.push(module_root);
    }

    // The program root is the last module's root.
    let root = *module_roots.last().unwrap_or(&NodeId(0));

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

/// Generate the identity function: Lambda(x, Ref(x)).
pub fn identity_program() -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();

    // Lambda node
    let lambda_node = make_node(
        NodeKind::Lambda,
        NodePayload::Lambda {
            binder: iris_types::graph::BinderId(0),
            captured_count: 0,
        },
        int_id,
        1,
    );
    let lambda_id = lambda_node.id;
    nodes.insert(lambda_id, lambda_node);

    // Lit as the "body" (identity returns its argument). In a real system
    // this would be a variable reference; for Gen1 seeding we use a simple
    // Lit(0) placeholder that the evolutionary process can mutate.
    let body_node = make_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: 0i64.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let body_id = body_node.id;
    nodes.insert(body_id, body_node);

    let edges = vec![Edge {
        source: lambda_id,
        target: body_id,
        port: 0,
        label: EdgeLabel::Binding,
    }];

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: lambda_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Zero,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

// ---------------------------------------------------------------------------
// Multi-stage composition seed generators
//
// These seeds use the fold port-2 collection override: when a fold node has
// a third argument edge (port 2), the fold uses that evaluated result as
// the collection instead of positional input 0. This enables composition
// patterns like map+fold and filter+fold.
// ---------------------------------------------------------------------------

/// Generate a map+fold composition: fold over a mapped collection.
///
/// Structure: Fold(base, fold_op, collection=Map(input, map_op))
/// Seeds programs like sum-of-squares: fold(0, add, map(input, mul))
pub fn random_map_fold_program(rng: &mut impl Rng) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Placeholder for the input collection (will be overridden by input[0]).
    let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let placeholder_id = placeholder.id;
    nodes.insert(placeholder_id, placeholder);

    // Map operation: random op applied to each element.
    // For binary ops on non-pair elements, map applies op(elem, elem).
    // e.g., mul(x, x) = x^2.
    let map_ops: [u8; 5] = [0x00, 0x02, 0x05, 0x06, 0x08]; // add, mul, neg, abs, max
    let map_opcode = map_ops[rng.gen_range(0..map_ops.len())];
    let map_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: map_opcode },
        int_id,
        2,
    );
    let map_step_id = map_step.id;
    nodes.insert(map_step_id, map_step);

    // Map node: map(placeholder, map_op)
    let map_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 }, // map
        int_id,
        2,
    );
    let map_id = map_node.id;
    nodes.insert(map_id, map_node);
    edges.push(Edge {
        source: map_id,
        target: placeholder_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: map_id,
        target: map_step_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    // Fold base.
    let bases: [i64; 3] = [-1, 0, 1];
    let base_value = bases[rng.gen_range(0..bases.len())];
    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: base_value.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    // Fold step op.
    let fold_ops: [u8; 4] = [0x00, 0x02, 0x07, 0x08]; // add, mul, min, max
    let fold_opcode = fold_ops[rng.gen_range(0..fold_ops.len())];
    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: fold_opcode },
        int_id,
        2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    // Fold node with 3 argument edges: base(0), step(1), collection(2).
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![],
        },
        int_id,
        3, // arity 3 for collection override
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge {
        source: fold_id,
        target: base_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: fold_id,
        target: fold_step_id,
        port: 1,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: fold_id,
        target: map_id,
        port: 2,
        label: EdgeLabel::Argument,
    });

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

/// Generate a filter+fold composition: fold over a filtered collection.
///
/// Structure: Fold(base, fold_op, collection=Filter(input, cmp_op))
/// The filter uses comparison opcodes (0x20-0x25) which compare element
/// against 0. This seeds patterns like sum-of-positives: fold(0, add, filter(input, gt)).
pub fn random_filter_fold_program(rng: &mut impl Rng) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Placeholder for the input collection.
    let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let placeholder_id = placeholder.id;
    nodes.insert(placeholder_id, placeholder);

    // Filter predicate: comparison opcode (compares element vs 0).
    let cmp_ops: [u8; 6] = [0x20, 0x21, 0x22, 0x23, 0x24, 0x25]; // eq, ne, lt, gt, le, ge
    let cmp_opcode = cmp_ops[rng.gen_range(0..cmp_ops.len())];
    let filter_pred = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: cmp_opcode },
        int_id,
        2,
    );
    let filter_pred_id = filter_pred.id;
    nodes.insert(filter_pred_id, filter_pred);

    // Filter node: filter(placeholder, cmp_op)
    let filter_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x31 }, // filter
        int_id,
        2,
    );
    let filter_id = filter_node.id;
    nodes.insert(filter_id, filter_node);
    edges.push(Edge {
        source: filter_id,
        target: placeholder_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: filter_id,
        target: filter_pred_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    // Fold base.
    let bases: [i64; 3] = [-1, 0, 1];
    let base_value = bases[rng.gen_range(0..bases.len())];
    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: base_value.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    // Fold step op.
    let fold_ops: [u8; 4] = [0x00, 0x02, 0x07, 0x08]; // add, mul, min, max
    let fold_opcode = fold_ops[rng.gen_range(0..fold_ops.len())];
    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: fold_opcode },
        int_id,
        2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    // Fold node with collection override.
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![],
        },
        int_id,
        3,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge {
        source: fold_id,
        target: base_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: fold_id,
        target: fold_step_id,
        port: 1,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: fold_id,
        target: filter_id,
        port: 2,
        label: EdgeLabel::Argument,
    });

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

/// Generate a zip+map+fold composition.
///
/// Structure: Fold(base, fold_op, collection=Map(Zip(input_a, input_b), pair_op))
/// Seeds programs like dot product: fold(0, add, map(zip(a, b), mul))
pub fn random_zip_map_fold_program(rng: &mut impl Rng) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Two placeholder tuples for zip inputs.
    let placeholder_a = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let pa_id = placeholder_a.id;
    nodes.insert(pa_id, placeholder_a);

    let placeholder_b = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: 0i64.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let pb_id = placeholder_b.id;
    nodes.insert(pb_id, placeholder_b);

    // Zip node.
    let zip_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x32 }, // zip
        int_id,
        2,
    );
    let zip_id = zip_node.id;
    nodes.insert(zip_id, zip_node);
    edges.push(Edge {
        source: zip_id,
        target: pa_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: zip_id,
        target: pb_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    // Pair operation for map (applied to each (a_i, b_i) pair).
    let pair_ops: [u8; 5] = [0x00, 0x01, 0x02, 0x07, 0x08]; // add, sub, mul, min, max
    let pair_opcode = pair_ops[rng.gen_range(0..pair_ops.len())];
    let pair_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: pair_opcode },
        int_id,
        2,
    );
    let pair_step_id = pair_step.id;
    nodes.insert(pair_step_id, pair_step);

    // Map node: map(zipped, pair_op)
    let map_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 }, // map
        int_id,
        2,
    );
    let map_id = map_node.id;
    nodes.insert(map_id, map_node);
    edges.push(Edge {
        source: map_id,
        target: zip_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: map_id,
        target: pair_step_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    // Fold base.
    let bases: [i64; 3] = [-1, 0, 1];
    let base_value = bases[rng.gen_range(0..bases.len())];
    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: base_value.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    // Fold step.
    let fold_ops: [u8; 4] = [0x00, 0x02, 0x07, 0x08];
    let fold_opcode = fold_ops[rng.gen_range(0..fold_ops.len())];
    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: fold_opcode },
        int_id,
        2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    // Fold node with collection override (port 2 = map output).
    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![],
        },
        int_id,
        3,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge {
        source: fold_id,
        target: base_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: fold_id,
        target: fold_step_id,
        port: 1,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: fold_id,
        target: map_id,
        port: 2,
        label: EdgeLabel::Argument,
    });

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

/// Generate a comparison-based fold: fold(extreme_base, min_or_max).
///
/// Uses extreme base values so min/max fold works correctly on first element.
/// Seeds max/min discovery.
pub fn random_comparison_fold_program(rng: &mut impl Rng) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Choose min or max pattern.
    let is_max = rng.gen_bool(0.5);
    let (base_value, step_opcode) = if is_max {
        (i64::MIN, 0x08u8) // max: start from MIN
    } else {
        (i64::MAX, 0x07u8) // min: start from MAX
    };

    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0,
            value: base_value.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let step_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: step_opcode },
        int_id,
        2,
    );
    let step_id = step_node.id;
    nodes.insert(step_id, step_node);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![],
        },
        int_id,
        2,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge {
        source: fold_id,
        target: base_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    edges.push(Edge {
        source: fold_id,
        target: step_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

/// Generate a counting/length fold: fold(0, add) with a constant-1 step.
///
/// Since fold step with Prim(add) does acc + elem, we need the collection
/// to be all 1s for counting. We use map to transform input to all 1s,
/// then fold(0, add) over the mapped result.
///
/// Alternatively, just fold(0, add) over the input — seeds sum patterns.
/// This seeds list-length (when elem values are all 1) and sum patterns.
pub fn random_stateful_fold_program(rng: &mut impl Rng) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Choose pattern: sum, length (via map to 1s), or product.
    let pattern = rng.gen_range(0..3u8);

    // Track root fold id across all pattern branches.
    // Each arm sets this before the match exits; the initial None is a required
    // placeholder for the mut binding.
    #[allow(unused_assignments)]
    let mut root_fold_id: Option<NodeId> = None;

    match pattern {
        0 => {
            // Plain fold(0, add) — sum pattern.
            let base_node = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit {
                    type_tag: 0,
                    value: 0i64.to_le_bytes().to_vec(),
                },
                int_id,
                0,
            );
            let base_id = base_node.id;
            nodes.insert(base_id, base_node);

            let step_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x00 }, // add
                int_id,
                2,
            );
            let step_id = step_node.id;
            nodes.insert(step_id, step_node);

            let fold_node = make_unique_node(
                NodeKind::Fold,
                NodePayload::Fold {
                    recursion_descriptor: vec![],
                },
                int_id,
                2,
            );
            let fold_id = fold_node.id;
            nodes.insert(fold_id, fold_node);
            edges.push(Edge {
                source: fold_id,
                target: base_id,
                port: 0,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: fold_id,
                target: step_id,
                port: 1,
                label: EdgeLabel::Argument,
            });
            root_fold_id = Some(fold_id);
        }
        1 => {
            // fold(1, mul) — product pattern (factorial when input is [1..n]).
            let base_node = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit {
                    type_tag: 0,
                    value: 1i64.to_le_bytes().to_vec(),
                },
                int_id,
                0,
            );
            let base_id = base_node.id;
            nodes.insert(base_id, base_node);

            let step_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x02 }, // mul
                int_id,
                2,
            );
            let step_id = step_node.id;
            nodes.insert(step_id, step_node);

            let fold_node = make_unique_node(
                NodeKind::Fold,
                NodePayload::Fold {
                    recursion_descriptor: vec![],
                },
                int_id,
                2,
            );
            let fold_id = fold_node.id;
            nodes.insert(fold_id, fold_node);
            edges.push(Edge {
                source: fold_id,
                target: base_id,
                port: 0,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: fold_id,
                target: step_id,
                port: 1,
                label: EdgeLabel::Argument,
            });
            root_fold_id = Some(fold_id);
        }
        _ => {
            // Length: fold(0, add, map(input, div/div)) where div(x,x) = 1.
            // Map with div makes every element 1 (when element != 0).
            // Then fold(0, add) counts them.
            let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
            let placeholder_id = placeholder.id;
            nodes.insert(placeholder_id, placeholder);

            // Map step: div(elem, elem) = 1 for all non-zero elements.
            // Use 0x03 (div) so div(x,x) = 1.
            let map_step = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x03 }, // div
                int_id,
                2,
            );
            let map_step_id = map_step.id;
            nodes.insert(map_step_id, map_step);

            let map_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x30 }, // map
                int_id,
                2,
            );
            let map_id = map_node.id;
            nodes.insert(map_id, map_node);
            edges.push(Edge {
                source: map_id,
                target: placeholder_id,
                port: 0,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: map_id,
                target: map_step_id,
                port: 1,
                label: EdgeLabel::Argument,
            });

            let base_node = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit {
                    type_tag: 0,
                    value: 0i64.to_le_bytes().to_vec(),
                },
                int_id,
                0,
            );
            let base_id = base_node.id;
            nodes.insert(base_id, base_node);

            let step_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x00 }, // add
                int_id,
                2,
            );
            let step_id = step_node.id;
            nodes.insert(step_id, step_node);

            let fold_node = make_unique_node(
                NodeKind::Fold,
                NodePayload::Fold {
                    recursion_descriptor: vec![],
                },
                int_id,
                3,
            );
            let fold_id = fold_node.id;
            nodes.insert(fold_id, fold_node);
            edges.push(Edge {
                source: fold_id,
                target: base_id,
                port: 0,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: fold_id,
                target: step_id,
                port: 1,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: fold_id,
                target: map_id,
                port: 2,
                label: EdgeLabel::Argument,
            });
            root_fold_id = Some(fold_id);
        }
    }

    let fold_id = root_fold_id.unwrap();

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

/// Generate a conditional fold: filter+fold composition for predicate patterns.
///
/// Structure: Fold(base, fold_op, collection=Filter(input, cmp_op))
/// with various base/fold_op combos for different predicate patterns:
/// - fold(0, add, filter(input, gt)) = count positives (when fold_op adds 1-mapped values)
/// - fold(1, mul, filter-inverted) = all-positive
/// - fold(0, max, filter(input, lt)) = any-negative
///
/// Also includes map+filter+fold combos for richer patterns.
pub fn random_conditional_fold_program(rng: &mut impl Rng) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let placeholder_id = placeholder.id;
    nodes.insert(placeholder_id, placeholder);

    let pattern = rng.gen_range(0..3u8);

    // Track root fold id across all pattern branches.
    // Each arm sets this before the match exits; the initial None is a required
    // placeholder for the mut binding.
    #[allow(unused_assignments)]
    let mut root_fold_id: Option<NodeId> = None;

    match pattern {
        0 => {
            // Filter + fold(0, add): count elements matching predicate.
            let cmp_ops: [u8; 4] = [0x21, 0x22, 0x23, 0x25]; // ne, lt, gt, ge
            let cmp_opcode = cmp_ops[rng.gen_range(0..cmp_ops.len())];
            let filter_pred = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: cmp_opcode },
                int_id,
                2,
            );
            let filter_pred_id = filter_pred.id;
            nodes.insert(filter_pred_id, filter_pred);

            let filter_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x31 },
                int_id,
                2,
            );
            let filter_id = filter_node.id;
            nodes.insert(filter_id, filter_node);
            edges.push(Edge {
                source: filter_id,
                target: placeholder_id,
                port: 0,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: filter_id,
                target: filter_pred_id,
                port: 1,
                label: EdgeLabel::Argument,
            });

            // Map filtered elements to 1 (via div(x,x)).
            let map_step = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x03 }, // div
                int_id,
                2,
            );
            let map_step_id = map_step.id;
            nodes.insert(map_step_id, map_step);

            let map_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x30 }, // map
                int_id,
                2,
            );
            let map_id = map_node.id;
            nodes.insert(map_id, map_node);
            edges.push(Edge {
                source: map_id,
                target: filter_id,
                port: 0,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: map_id,
                target: map_step_id,
                port: 1,
                label: EdgeLabel::Argument,
            });

            let base_node = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit {
                    type_tag: 0,
                    value: 0i64.to_le_bytes().to_vec(),
                },
                int_id,
                0,
            );
            let base_id = base_node.id;
            nodes.insert(base_id, base_node);

            let step_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x00 }, // add
                int_id,
                2,
            );
            let step_id = step_node.id;
            nodes.insert(step_id, step_node);

            let fold_node = make_unique_node(
                NodeKind::Fold,
                NodePayload::Fold {
                    recursion_descriptor: vec![],
                },
                int_id,
                3,
            );
            let fold_id = fold_node.id;
            nodes.insert(fold_id, fold_node);
            edges.push(Edge {
                source: fold_id,
                target: base_id,
                port: 0,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: fold_id,
                target: step_id,
                port: 1,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: fold_id,
                target: map_id,
                port: 2,
                label: EdgeLabel::Argument,
            });
            root_fold_id = Some(fold_id);
        }
        1 => {
            // Filter + fold: sum of filtered elements.
            let cmp_ops: [u8; 4] = [0x21, 0x22, 0x23, 0x25];
            let cmp_opcode = cmp_ops[rng.gen_range(0..cmp_ops.len())];
            let filter_pred = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: cmp_opcode },
                int_id,
                2,
            );
            let filter_pred_id = filter_pred.id;
            nodes.insert(filter_pred_id, filter_pred);

            let filter_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x31 },
                int_id,
                2,
            );
            let filter_id = filter_node.id;
            nodes.insert(filter_id, filter_node);
            edges.push(Edge {
                source: filter_id,
                target: placeholder_id,
                port: 0,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: filter_id,
                target: filter_pred_id,
                port: 1,
                label: EdgeLabel::Argument,
            });

            let base_node = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit {
                    type_tag: 0,
                    value: 0i64.to_le_bytes().to_vec(),
                },
                int_id,
                0,
            );
            let base_id = base_node.id;
            nodes.insert(base_id, base_node);

            let fold_ops: [u8; 2] = [0x00, 0x02]; // add or mul
            let fold_opcode = fold_ops[rng.gen_range(0..fold_ops.len())];
            let step_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: fold_opcode },
                int_id,
                2,
            );
            let step_id = step_node.id;
            nodes.insert(step_id, step_node);

            let fold_node = make_unique_node(
                NodeKind::Fold,
                NodePayload::Fold {
                    recursion_descriptor: vec![],
                },
                int_id,
                3,
            );
            let fold_id = fold_node.id;
            nodes.insert(fold_id, fold_node);
            edges.push(Edge {
                source: fold_id,
                target: base_id,
                port: 0,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: fold_id,
                target: step_id,
                port: 1,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: fold_id,
                target: filter_id,
                port: 2,
                label: EdgeLabel::Argument,
            });
            root_fold_id = Some(fold_id);
        }
        _ => {
            // Map + fold(0, add): transform and sum.
            // Different from map_fold_program in that it uses abs mapping.
            let map_ops: [u8; 3] = [0x05, 0x06, 0x02]; // neg, abs, mul
            let map_opcode = map_ops[rng.gen_range(0..map_ops.len())];
            let map_step = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: map_opcode },
                int_id,
                2,
            );
            let map_step_id = map_step.id;
            nodes.insert(map_step_id, map_step);

            let map_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x30 },
                int_id,
                2,
            );
            let map_id = map_node.id;
            nodes.insert(map_id, map_node);
            edges.push(Edge {
                source: map_id,
                target: placeholder_id,
                port: 0,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: map_id,
                target: map_step_id,
                port: 1,
                label: EdgeLabel::Argument,
            });

            let base_node = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit {
                    type_tag: 0,
                    value: 0i64.to_le_bytes().to_vec(),
                },
                int_id,
                0,
            );
            let base_id = base_node.id;
            nodes.insert(base_id, base_node);

            let step_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x00 }, // add
                int_id,
                2,
            );
            let step_id = step_node.id;
            nodes.insert(step_id, step_node);

            let fold_node = make_unique_node(
                NodeKind::Fold,
                NodePayload::Fold {
                    recursion_descriptor: vec![],
                },
                int_id,
                3,
            );
            let fold_id = fold_node.id;
            nodes.insert(fold_id, fold_node);
            edges.push(Edge {
                source: fold_id,
                target: base_id,
                port: 0,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: fold_id,
                target: step_id,
                port: 1,
                label: EdgeLabel::Argument,
            });
            edges.push(Edge {
                source: fold_id,
                target: map_id,
                port: 2,
                label: EdgeLabel::Argument,
            });
            root_fold_id = Some(fold_id);
        }
    }

    let fold_id = root_fold_id.unwrap();

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}

/// Generate an Unfold-based iteration program with tuple state.
///
/// Structure: Project(0, Last(Unfold(seed_tuple, step_op, n_steps_from_input)))
///
/// This seeds iterative algorithms that can't be expressed as fold-over-collection:
/// - Fibonacci: unfold((0,1), add, n) -> emit a, state = (b, a+b) -> take last
/// - GCD: unfold((a,b), mod, term=b==0) -> take first element of final state
/// - Collatz: unfold(n, step, term=n==1) -> count steps
///
/// The Unfold node's Prim step on tuple state (a,b) does:
///   add: (a,b) -> (b, a+b)   [Fibonacci]
///   mod: (a,b) -> (b, a%b)   [GCD]
///   sub: (a,b) -> (b, a-b)
///   mul: (a,b) -> (b, a*b)
pub fn random_iterate_program(rng: &mut impl Rng) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let pattern = rng.gen_range(0..4u8);

    match pattern {
        0 => {
            // Fibonacci pattern: unfold((0,1), add, input_length)
            // The input is a list whose length encodes n.
            // We use the Unfold with a length-bounded iteration.
            // Final result = last emitted element.

            // Seed tuple (0, 1)
            let lit_0 = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
                int_id, 0,
            );
            let lit_1 = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
                int_id, 0,
            );
            let lit_0_id = lit_0.id;
            let lit_1_id = lit_1.id;
            nodes.insert(lit_0_id, lit_0);
            nodes.insert(lit_1_id, lit_1);

            let seed_tuple = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 2);
            let seed_id = seed_tuple.id;
            nodes.insert(seed_id, seed_tuple);
            edges.push(Edge { source: seed_id, target: lit_0_id, port: 0, label: EdgeLabel::Argument });
            edges.push(Edge { source: seed_id, target: lit_1_id, port: 1, label: EdgeLabel::Argument });

            // Step: add (for Fibonacci: (a,b) -> (b, a+b))
            let step_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x00 }, // add
                int_id, 2,
            );
            let step_id = step_node.id;
            nodes.insert(step_id, step_node);

            // Input placeholder (list whose length = n)
            let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
            let placeholder_id = placeholder.id;
            nodes.insert(placeholder_id, placeholder);

            // Unfold with 4 args: seed, step, (no termination), n_steps=input
            let unfold_node = make_unique_node(
                NodeKind::Unfold,
                NodePayload::Unfold { recursion_descriptor: vec![] },
                int_id, 4,
            );
            let unfold_id = unfold_node.id;
            nodes.insert(unfold_id, unfold_node);
            edges.push(Edge { source: unfold_id, target: seed_id, port: 0, label: EdgeLabel::Argument });
            edges.push(Edge { source: unfold_id, target: step_id, port: 1, label: EdgeLabel::Argument });
            // Port 2: no termination predicate, use a dummy Lit(0) that won't match Bool(true)
            let no_term = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
                int_id, 0,
            );
            let no_term_id = no_term.id;
            nodes.insert(no_term_id, no_term);
            edges.push(Edge { source: unfold_id, target: no_term_id, port: 2, label: EdgeLabel::Argument });
            edges.push(Edge { source: unfold_id, target: placeholder_id, port: 3, label: EdgeLabel::Argument });

            // Take last element: fold(0, max_index) over the unfold result.
            // Actually, the simpler approach: use fold(0, add) to just sum... no.
            // For Fibonacci, unfold((0,1), add, n) emits [0, 1, 1, 2, 3, 5, ...] (the a values).
            // fib(0)=0 (list len 0, unfold produces []), fib(1)=1 (list len 1, unfold produces [0]).
            // Wait, we need the LAST element of the unfold output.
            // The unfold emits `a` at each step, so for n steps starting from (0,1):
            //   step 0: emit 0, state becomes (1, 0+1) = (1,1)
            //   step 1: emit 1, state becomes (1, 1+1) = (1,2)
            //   step 2: emit 1, state becomes (2, 1+2) = (2,3)
            //   step 3: emit 2, state becomes (3, 2+3) = (3,5)
            //   step 4: emit 3, state becomes (5, 3+5) = (5,8)
            //   step 5: emit 5, ...
            // So unfold n times gives elements [fib(0), fib(1), ..., fib(n-1)].
            // For fib(n), we need n+1 steps and take the last, or n steps and take the last.
            // Actually fib(0)=0: input is list of len 0, unfold 0 times -> empty -> return 0.
            // fib(1)=1: input is list of len 1, unfold 1 time -> [0] -> last is 0, but expected 1.
            // Hmm, need to adjust. Let's use seed (1,0) instead of (0,1) so:
            //   unfold 1 time from (1,0): emit 1, state=(0, 1+0)=(0,1) -> [1], last=1. fib(1)=1? No, fib(1) should be 1. OK.
            //   unfold 2 times from (1,0): emit 1, state=(0,1); emit 0, state=(1,0+1)=(1,1) -> [1,0], last=0. fib(2)=1. Wrong.
            // This doesn't quite work with just last element.
            //
            // Better approach: use fold(0, add) over the unfold output to get the last emitted value.
            // Wait, that sums everything.
            //
            // Actually the simplest: unfold((0,1), add, n) produces n elements.
            // The n-th Fibonacci is the (n+1)-th emitted element. But the benchmark uses
            // list length = n, so for fib(n), we get n elements from unfold.
            // Elements are [0, 1, 1, 2, 3, 5, ..., fib(n-1)].
            // We want fib(n), not fib(n-1).
            //
            // Solution: Use seed (0,1), iterate n+1 times, take last. But n+1 is hard to express.
            // Alternative: After unfold, fold with op that keeps last value.
            // fold(base=0, step=second_arg, unfold_result) -- but we don't have a "second_arg" op.
            //
            // Simplest: the Unfold can be bounded by input length. After the loop, the
            // FINAL state has the answer. We need state[0] after n iterations.
            // From (0,1) after n iterations:
            //   n=0: state=(0,1), state[0]=0=fib(0) ✓
            //   n=1: state=(1,1), state[0]=1=fib(1) ✓
            //   n=2: state=(1,2), state[0]=1=fib(2) ✓
            //   n=3: state=(2,3), state[0]=2=fib(3) ✓
            //   n=4: state=(3,5), state[0]=3=fib(4) ✓
            //   n=5: state=(5,8), state[0]=5=fib(5) ✓
            // Yes! So we want the FINAL STATE, not the emitted elements.
            // But unfold returns the emitted elements as a Tuple. We need a variant that
            // returns the final state, or we use the elements differently.
            //
            // The emitted elements are the `a` values at each step:
            //   [0, 1, 1, 2, 3, ...] for n steps.
            // The n-th emitted element (0-indexed) is fib(n).
            // So the LAST emitted element from n steps is fib(n-1).
            // For fib(n), we need the (n+1)-th emitted element.
            //
            // But the input list has length n, so we run n steps and get fib(n-1) as last.
            // That's off by one. We need n+1 steps.
            //
            // Alternative: use seed (1, 1) and the step adds to get next fib.
            // From (1,1): emit 1, (1,2); emit 1, (2,3); emit 2, (3,5); emit 3, (5,8); emit 5, ...
            // After n steps from (1,1): elements are [1, 1, 2, 3, 5, ...] = [fib(1), fib(2), ...]
            // Last element from n steps = fib(n). ✓ (for n>=1)
            // For n=0: empty list. We need fib(0)=0.
            //
            // Solution: take fold(0, max, unfold_result). If unfold is empty, returns 0.
            // For n>=1, max of [1, 1, 2, 3, 5, ...] works for small n but not generally.
            //
            // Better: just use fold to keep the last element.
            // fold(initial=0, step=??, collection) where step ignores acc and returns elem.
            // In the interpreter, fold with Prim step does acc = op(acc, elem).
            // If we use sub: acc = acc - elem, that's wrong.
            // If we use a special approach...
            //
            // Actually, the simplest correct encoding:
            // For Fibonacci, the seed IS the Unfold node but we add the Fold(0, add) pattern
            // with just the right elements. The evolution can mutate from there.
            // Let me just use a "take-last" approach via a custom fold:
            // fold(0, sub, unfold_result): final acc = 0 - e0 - e1 - ... not right.
            //
            // OK, the cleanest solution for the enumerator: build a purpose-built
            // Fibonacci graph using the unfold + Project approach. For the SEED generator,
            // we just need to get close enough for evolution to refine.
            //
            // For seeds: use Unfold((1,1), add, input_len) and take fold(0, max, result).
            // This gives fib(n) = max([1,1,2,3,5,...]) for small n<=5.
            // Not exact for large n, but evolution can refine.
            // Actually fold(0, add, [1,1,2,3,5]) = 12, not 5. Max gives 5 ✓ for fib(5)!
            // But fib(4)=3, and max([1,1,2,3])=3 ✓. fib(3)=2, max([1,1,2])=2 ✓.
            // fib(2)=1, max([1,1])=1 ✓. fib(1)=1, max([1])=1 ✓. fib(0)=0, max([])=0 ✓ (base).
            // Wait, fib(6)=8. Unfold 6 steps from (1,1): [1,1,2,3,5,8]. max=8 ✓!
            // fib(7)=13. Unfold 7 steps: [1,1,2,3,5,8,13]. max=13 ✓!
            // THIS WORKS because Fibonacci is monotonically increasing for n>=1.
            // So fold(0, max, unfold((1,1), add, input_len)) = fib(n)!

            // Use fold(base=0, max, unfold_result)
            let fold_base = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
                int_id, 0,
            );
            let fold_base_id = fold_base.id;
            nodes.insert(fold_base_id, fold_base);

            let fold_step = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x08 }, // max
                int_id, 2,
            );
            let fold_step_id = fold_step.id;
            nodes.insert(fold_step_id, fold_step);

            let fold_node = make_unique_node(
                NodeKind::Fold,
                NodePayload::Fold { recursion_descriptor: vec![] },
                int_id, 3,
            );
            let fold_id = fold_node.id;
            nodes.insert(fold_id, fold_node);
            edges.push(Edge { source: fold_id, target: fold_base_id, port: 0, label: EdgeLabel::Argument });
            edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
            edges.push(Edge { source: fold_id, target: unfold_id, port: 2, label: EdgeLabel::Argument });

            let hash = compute_hash(&nodes, &edges);
            let graph = SemanticGraph {
                root: fold_id,
                nodes,
                edges,
                type_env,
                cost: CostBound::Unknown,
                resolution: Resolution::Implementation,
                hash,
            };
            return graph_to_fragment(graph);
        }
        1 => {
            // GCD pattern: unfold((input[0], input[1]), mod, term=b==0)
            // Then project field 0 of the final state.
            // Since unfold returns emitted elements (the a values), the last a before
            // termination is the GCD.
            //
            // GCD(12,8): (12,8) -> emit 12, (8, 12%8=4) -> emit 8, (4, 8%4=0) -> term! -> [12,8]
            // We want 4 (the final state's a). But we emitted [12, 8]. The answer is in the
            // step AFTER the last emission. Hmm.
            //
            // Actually: the termination check happens BEFORE emission. So:
            //   iter 0: check term on (12,8), b=8 != 0, continue. emit 12. state=(8, 12%8=4).
            //   iter 1: check term on (8,4), b=4 != 0, continue. emit 8. state=(4, 8%4=0).
            //   iter 2: check term on (4,0), b=0 == 0, STOP. elements=[12, 8].
            // The GCD is 4, which is state[0] = 4. But we returned elements [12, 8].
            // We can get it via fold(0, max, [12, 8]) = 12, which is wrong.
            // We need the LAST state, not elements.
            //
            // Alternative approach: Don't use termination. Use mod until b becomes 0,
            // then the sequence of a values converges. But mod(4, 0) = division by zero!
            //
            // Solution: use a fixed number of iterations (say, the sum of inputs as bound).
            // Or use the input itself as the bound (length of a dummy list).
            //
            // Actually for GCD, the test cases pass inputs as Tuple([a, b]).
            // The collection has length 2. If we use that as bound, we only get 2 iterations.
            // That's not enough.
            //
            // Better solution for GCD: the Unfold with termination approach but we need
            // to return the FINAL STATE, not the emitted elements.
            // Let me add support for that in the interpreter: when the recursion_descriptor
            // contains a specific marker byte, return the final state instead of elements.
            //
            // For now, let's use a different approach for GCD seeds: just create
            // unfold + fold(0, ?, result) and let evolution discover the right combination.
            // Use various step ops.

            // Seed: Tuple from input (the input is already a tuple of (a, b))
            let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
            let placeholder_id = placeholder.id;
            nodes.insert(placeholder_id, placeholder);

            // Step: mod
            let step_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x04 }, // mod
                int_id, 2,
            );
            let step_id = step_node.id;
            nodes.insert(step_id, step_node);

            // Termination: eq (b == 0)
            let term_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x20 }, // eq
                int_id, 2,
            );
            let term_id = term_node.id;
            nodes.insert(term_id, term_node);

            // Unfold with 3 args: seed=input, step=mod, term=eq
            let unfold_node = make_unique_node(
                NodeKind::Unfold,
                NodePayload::Unfold { recursion_descriptor: vec![0x01] }, // marker: return final state
                int_id, 3,
            );
            let unfold_id = unfold_node.id;
            nodes.insert(unfold_id, unfold_node);
            edges.push(Edge { source: unfold_id, target: placeholder_id, port: 0, label: EdgeLabel::Argument });
            edges.push(Edge { source: unfold_id, target: step_id, port: 1, label: EdgeLabel::Argument });
            edges.push(Edge { source: unfold_id, target: term_id, port: 2, label: EdgeLabel::Argument });

            // Project field 0 of the result to get the GCD
            let project_node = make_unique_node(
                NodeKind::Project,
                NodePayload::Project { field_index: 0 },
                int_id, 1,
            );
            let project_id = project_node.id;
            nodes.insert(project_id, project_node);
            edges.push(Edge { source: project_id, target: unfold_id, port: 0, label: EdgeLabel::Argument });

            let hash = compute_hash(&nodes, &edges);
            let graph = SemanticGraph {
                root: project_id,
                nodes,
                edges,
                type_env,
                cost: CostBound::Unknown,
                resolution: Resolution::Implementation,
                hash,
            };
            return graph_to_fragment(graph);
        }
        2 => {
            // Generic unfold iteration: unfold(seed, random_op, input_bound)
            // Seeds for discovery of various iterative patterns.
            let seeds: [(i64, i64); 4] = [(0, 1), (1, 0), (1, 1), (0, 0)];
            let (s0, s1) = seeds[rng.gen_range(0..seeds.len())];

            let lit_s0 = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit { type_tag: 0, value: s0.to_le_bytes().to_vec() },
                int_id, 0,
            );
            let lit_s1 = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit { type_tag: 0, value: s1.to_le_bytes().to_vec() },
                int_id, 0,
            );
            let s0_id = lit_s0.id;
            let s1_id = lit_s1.id;
            nodes.insert(s0_id, lit_s0);
            nodes.insert(s1_id, lit_s1);

            let seed_tuple = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 2);
            let seed_id = seed_tuple.id;
            nodes.insert(seed_id, seed_tuple);
            edges.push(Edge { source: seed_id, target: s0_id, port: 0, label: EdgeLabel::Argument });
            edges.push(Edge { source: seed_id, target: s1_id, port: 1, label: EdgeLabel::Argument });

            let step_ops: [u8; 5] = [0x00, 0x01, 0x02, 0x04, 0x08]; // add, sub, mul, mod, max
            let step_opcode = step_ops[rng.gen_range(0..step_ops.len())];
            let step_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: step_opcode },
                int_id, 2,
            );
            let step_id = step_node.id;
            nodes.insert(step_id, step_node);

            let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
            let placeholder_id = placeholder.id;
            nodes.insert(placeholder_id, placeholder);

            // Unfold bounded by input length
            let unfold_node = make_unique_node(
                NodeKind::Unfold,
                NodePayload::Unfold { recursion_descriptor: vec![] },
                int_id, 4,
            );
            let unfold_id = unfold_node.id;
            nodes.insert(unfold_id, unfold_node);
            edges.push(Edge { source: unfold_id, target: seed_id, port: 0, label: EdgeLabel::Argument });
            edges.push(Edge { source: unfold_id, target: step_id, port: 1, label: EdgeLabel::Argument });
            // No termination predicate
            let no_term = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
                int_id, 0,
            );
            let no_term_id = no_term.id;
            nodes.insert(no_term_id, no_term);
            edges.push(Edge { source: unfold_id, target: no_term_id, port: 2, label: EdgeLabel::Argument });
            edges.push(Edge { source: unfold_id, target: placeholder_id, port: 3, label: EdgeLabel::Argument });

            // Reduce with fold
            let fold_ops: [u8; 4] = [0x00, 0x02, 0x07, 0x08];
            let fold_opcode = fold_ops[rng.gen_range(0..fold_ops.len())];
            let fold_base_val: i64 = [0, 1, i64::MIN][rng.gen_range(0..3)];
            let fold_base = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit { type_tag: 0, value: fold_base_val.to_le_bytes().to_vec() },
                int_id, 0,
            );
            let fold_base_id = fold_base.id;
            nodes.insert(fold_base_id, fold_base);

            let fold_step = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: fold_opcode },
                int_id, 2,
            );
            let fold_step_id = fold_step.id;
            nodes.insert(fold_step_id, fold_step);

            let fold_node = make_unique_node(
                NodeKind::Fold,
                NodePayload::Fold { recursion_descriptor: vec![] },
                int_id, 3,
            );
            let fold_id = fold_node.id;
            nodes.insert(fold_id, fold_node);
            edges.push(Edge { source: fold_id, target: fold_base_id, port: 0, label: EdgeLabel::Argument });
            edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
            edges.push(Edge { source: fold_id, target: unfold_id, port: 2, label: EdgeLabel::Argument });

            let hash = compute_hash(&nodes, &edges);
            let graph = SemanticGraph {
                root: fold_id,
                nodes,
                edges,
                type_env,
                cost: CostBound::Unknown,
                resolution: Resolution::Implementation,
                hash,
            };
            return graph_to_fragment(graph);
        }
        _ => {
            // Unfold with termination predicate (various ops)
            let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
            let placeholder_id = placeholder.id;
            nodes.insert(placeholder_id, placeholder);

            let step_ops: [u8; 3] = [0x00, 0x01, 0x04]; // add, sub, mod
            let step_opcode = step_ops[rng.gen_range(0..step_ops.len())];
            let step_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: step_opcode },
                int_id, 2,
            );
            let step_id = step_node.id;
            nodes.insert(step_id, step_node);

            let cmp_ops: [u8; 3] = [0x20, 0x22, 0x24]; // eq, lt, le
            let term_opcode = cmp_ops[rng.gen_range(0..cmp_ops.len())];
            let term_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: term_opcode },
                int_id, 2,
            );
            let term_id = term_node.id;
            nodes.insert(term_id, term_node);

            let unfold_node = make_unique_node(
                NodeKind::Unfold,
                NodePayload::Unfold { recursion_descriptor: vec![0x01] }, // return final state
                int_id, 3,
            );
            let unfold_id = unfold_node.id;
            nodes.insert(unfold_id, unfold_node);
            edges.push(Edge { source: unfold_id, target: placeholder_id, port: 0, label: EdgeLabel::Argument });
            edges.push(Edge { source: unfold_id, target: step_id, port: 1, label: EdgeLabel::Argument });
            edges.push(Edge { source: unfold_id, target: term_id, port: 2, label: EdgeLabel::Argument });

            // Project field 0
            let project_node = make_unique_node(
                NodeKind::Project,
                NodePayload::Project { field_index: 0 },
                int_id, 1,
            );
            let project_id = project_node.id;
            nodes.insert(project_id, project_node);
            edges.push(Edge { source: project_id, target: unfold_id, port: 0, label: EdgeLabel::Argument });

            let hash = compute_hash(&nodes, &edges);
            let graph = SemanticGraph {
                root: project_id,
                nodes,
                edges,
                type_env,
                cost: CostBound::Unknown,
                resolution: Resolution::Implementation,
                hash,
            };
            return graph_to_fragment(graph);
        }
    }
}

/// Generate a pairwise fold program: fold over zip(input, drop(input, 1)).
///
/// Seeds pairwise comparison patterns:
/// - Is sorted: fold(1, mul, map(zip(input, drop(input,1)), le))
/// - Pairwise differences: map(zip(input, drop(input,1)), sub)
pub fn random_pairwise_fold_program(rng: &mut impl Rng) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Input placeholder
    let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let placeholder_id = placeholder.id;
    nodes.insert(placeholder_id, placeholder);

    // drop(input, 1)
    let lit_1 = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let lit_1_id = lit_1.id;
    nodes.insert(lit_1_id, lit_1);

    let drop_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x34 }, // drop
        int_id, 2,
    );
    let drop_id = drop_node.id;
    nodes.insert(drop_id, drop_node);
    edges.push(Edge { source: drop_id, target: placeholder_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: drop_id, target: lit_1_id, port: 1, label: EdgeLabel::Argument });

    // zip(input, drop(input, 1))
    let zip_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x32 }, // zip
        int_id, 2,
    );
    let zip_id = zip_node.id;
    nodes.insert(zip_id, zip_node);
    edges.push(Edge { source: zip_id, target: placeholder_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: zip_id, target: drop_id, port: 1, label: EdgeLabel::Argument });

    let pattern = rng.gen_range(0..3u8);

    match pattern {
        0 => {
            // Is-sorted: fold(1, mul, map(zip(input, drop(input,1)), le))
            // map each (a,b) pair to le(a,b), then fold with mul (all must be 1/true)
            let cmp_ops: [u8; 2] = [0x24, 0x22]; // le, lt
            let cmp_opcode = cmp_ops[rng.gen_range(0..cmp_ops.len())];
            let map_step = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: cmp_opcode },
                int_id, 2,
            );
            let map_step_id = map_step.id;
            nodes.insert(map_step_id, map_step);

            let map_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x30 }, // map
                int_id, 2,
            );
            let map_id = map_node.id;
            nodes.insert(map_id, map_node);
            edges.push(Edge { source: map_id, target: zip_id, port: 0, label: EdgeLabel::Argument });
            edges.push(Edge { source: map_id, target: map_step_id, port: 1, label: EdgeLabel::Argument });

            let fold_base = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
                int_id, 0,
            );
            let fold_base_id = fold_base.id;
            nodes.insert(fold_base_id, fold_base);

            // fold with mul (for "all true" pattern) or min
            let fold_ops: [u8; 2] = [0x02, 0x07]; // mul, min
            let fold_opcode = fold_ops[rng.gen_range(0..fold_ops.len())];
            let fold_step = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: fold_opcode },
                int_id, 2,
            );
            let fold_step_id = fold_step.id;
            nodes.insert(fold_step_id, fold_step);

            let fold_node = make_unique_node(
                NodeKind::Fold,
                NodePayload::Fold { recursion_descriptor: vec![] },
                int_id, 3,
            );
            let fold_id = fold_node.id;
            nodes.insert(fold_id, fold_node);
            edges.push(Edge { source: fold_id, target: fold_base_id, port: 0, label: EdgeLabel::Argument });
            edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
            edges.push(Edge { source: fold_id, target: map_id, port: 2, label: EdgeLabel::Argument });

            let hash = compute_hash(&nodes, &edges);
            let graph = SemanticGraph {
                root: fold_id,
                nodes,
                edges,
                type_env,
                cost: CostBound::Unknown,
                resolution: Resolution::Implementation,
                hash,
            };
            return graph_to_fragment(graph);
        }
        1 => {
            // Pairwise differences: map(zip(input, drop(input,1)), sub)
            let map_step = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x01 }, // sub
                int_id, 2,
            );
            let map_step_id = map_step.id;
            nodes.insert(map_step_id, map_step);

            let map_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x30 }, // map
                int_id, 2,
            );
            let map_id = map_node.id;
            nodes.insert(map_id, map_node);
            edges.push(Edge { source: map_id, target: zip_id, port: 0, label: EdgeLabel::Argument });
            edges.push(Edge { source: map_id, target: map_step_id, port: 1, label: EdgeLabel::Argument });

            let hash = compute_hash(&nodes, &edges);
            let graph = SemanticGraph {
                root: map_id,
                nodes,
                edges,
                type_env,
                cost: CostBound::Unknown,
                resolution: Resolution::Implementation,
                hash,
            };
            return graph_to_fragment(graph);
        }
        _ => {
            // Generic pairwise fold: fold(base, op, map(zipped_pairs, pair_op))
            let pair_ops: [u8; 4] = [0x00, 0x01, 0x02, 0x08]; // add, sub, mul, max
            let pair_opcode = pair_ops[rng.gen_range(0..pair_ops.len())];
            let map_step = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: pair_opcode },
                int_id, 2,
            );
            let map_step_id = map_step.id;
            nodes.insert(map_step_id, map_step);

            let map_node = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: 0x30 }, // map
                int_id, 2,
            );
            let map_id = map_node.id;
            nodes.insert(map_id, map_node);
            edges.push(Edge { source: map_id, target: zip_id, port: 0, label: EdgeLabel::Argument });
            edges.push(Edge { source: map_id, target: map_step_id, port: 1, label: EdgeLabel::Argument });

            let fold_bases: [i64; 3] = [0, 1, i64::MIN];
            let fold_base_val = fold_bases[rng.gen_range(0..fold_bases.len())];
            let fold_base = make_unique_node(
                NodeKind::Lit,
                NodePayload::Lit { type_tag: 0, value: fold_base_val.to_le_bytes().to_vec() },
                int_id, 0,
            );
            let fold_base_id = fold_base.id;
            nodes.insert(fold_base_id, fold_base);

            let fold_ops: [u8; 4] = [0x00, 0x02, 0x07, 0x08];
            let fold_opcode = fold_ops[rng.gen_range(0..fold_ops.len())];
            let fold_step = make_unique_node(
                NodeKind::Prim,
                NodePayload::Prim { opcode: fold_opcode },
                int_id, 2,
            );
            let fold_step_id = fold_step.id;
            nodes.insert(fold_step_id, fold_step);

            let fold_node = make_unique_node(
                NodeKind::Fold,
                NodePayload::Fold { recursion_descriptor: vec![] },
                int_id, 3,
            );
            let fold_id = fold_node.id;
            nodes.insert(fold_id, fold_node);
            edges.push(Edge { source: fold_id, target: fold_base_id, port: 0, label: EdgeLabel::Argument });
            edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
            edges.push(Edge { source: fold_id, target: map_id, port: 2, label: EdgeLabel::Argument });

            let hash = compute_hash(&nodes, &edges);
            let graph = SemanticGraph {
                root: fold_id,
                nodes,
                edges,
                type_env,
                cost: CostBound::Unknown,
                resolution: Resolution::Implementation,
                hash,
            };
            return graph_to_fragment(graph);
        }
    }
}

/// Generate a map-comparison+fold seed: fold(base, fold_op, map(input, cmp_op)).
///
/// Seeds predicate-counting and predicate-checking patterns:
/// - Count positives: fold(0, add, map(input, gt))
/// - All positive:    fold(1, mul, map(input, gt))
/// - Any negative:    fold(0, max, map(input, lt))
pub fn random_map_cmp_fold_program(rng: &mut impl Rng) -> Fragment {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let placeholder = make_unique_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 0);
    let placeholder_id = placeholder.id;
    nodes.insert(placeholder_id, placeholder);

    let cmp_ops: [u8; 6] = [0x20, 0x21, 0x22, 0x23, 0x24, 0x25];
    let cmp_opcode = cmp_ops[rng.gen_range(0..cmp_ops.len())];
    let map_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: cmp_opcode },
        int_id,
        2,
    );
    let map_step_id = map_step.id;
    nodes.insert(map_step_id, map_step);

    let map_node = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 },
        int_id,
        2,
    );
    let map_id = map_node.id;
    nodes.insert(map_id, map_node);
    edges.push(Edge { source: map_id, target: placeholder_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: map_id, target: map_step_id, port: 1, label: EdgeLabel::Argument });

    let bases: [i64; 3] = [0, 1, 0];
    let base_value = bases[rng.gen_range(0..bases.len())];
    let base_node = make_unique_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: base_value.to_le_bytes().to_vec() },
        int_id,
        0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    let fold_ops: [u8; 4] = [0x00, 0x02, 0x07, 0x08];
    let fold_opcode = fold_ops[rng.gen_range(0..fold_ops.len())];
    let fold_step = make_unique_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: fold_opcode },
        int_id,
        2,
    );
    let fold_step_id = fold_step.id;
    nodes.insert(fold_step_id, fold_step);

    let fold_node = make_unique_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id,
        3,
    );
    let fold_id = fold_node.id;
    nodes.insert(fold_id, fold_node);
    edges.push(Edge { source: fold_id, target: base_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: fold_step_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: fold_id, target: map_id, port: 2, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: fold_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    graph_to_fragment(graph)
}
