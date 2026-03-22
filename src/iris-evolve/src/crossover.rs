use std::collections::{BTreeSet, BTreeMap};

use rand::Rng;

use iris_types::graph::{Edge, EdgeLabel, NodeId, SemanticGraph};
use iris_types::hash::{compute_node_id, SemanticHash};

use crate::mutation::MAX_NODES_PER_INDIVIDUAL;

// ---------------------------------------------------------------------------
// Gen1 crossover: subgraph exchange
// ---------------------------------------------------------------------------

/// Perform crossover by exchanging subgraphs between two parents.
///
/// Gen1 implementation: no embedding interpolation (requires codec, Gen2).
/// Instead, pick a random subgraph from each parent and swap them.
pub fn crossover(
    parent_a: &SemanticGraph,
    parent_b: &SemanticGraph,
    rng: &mut impl Rng,
) -> SemanticGraph {
    // Pick a random pivot node from each parent.
    let ids_a: Vec<NodeId> = parent_a.nodes.keys().copied().collect();
    let ids_b: Vec<NodeId> = parent_b.nodes.keys().copied().collect();

    if ids_a.is_empty() {
        return parent_b.clone();
    }
    if ids_b.is_empty() {
        return parent_a.clone();
    }

    // Start with parent_a as the base.
    let mut child = parent_a.clone();

    // Extract a small subgraph from parent_b (pivot + 1-hop successors).
    let pivot_b = ids_b[rng.gen_range(0..ids_b.len())];
    let donor_nodes = collect_subgraph(parent_b, pivot_b, 2);

    // Pick a splice point in parent_a (a node to attach the donor subgraph to).
    let splice_point = ids_a[rng.gen_range(0..ids_a.len())];

    // Transplant donor nodes into the child with remapped IDs to avoid
    // collisions. We perturb salt to produce fresh hashes.
    let mut id_remap: BTreeMap<NodeId, NodeId> = BTreeMap::new();

    for &donor_id in &donor_nodes {
        if let Some(orig) = parent_b.nodes.get(&donor_id) {
            let mut node = orig.clone();
            // Perturb to get a fresh ID that won't collide.
            node.salt = node.salt.wrapping_add(1);
            node.id = compute_node_id(&node);
            id_remap.insert(donor_id, node.id);
            child.nodes.insert(node.id, node);
        }
    }

    // Copy edges internal to the donor subgraph.
    for edge in &parent_b.edges {
        if let (Some(&new_src), Some(&new_tgt)) =
            (id_remap.get(&edge.source), id_remap.get(&edge.target))
        {
            child.edges.push(Edge {
                source: new_src,
                target: new_tgt,
                port: edge.port,
                label: edge.label,
            });
        }
    }

    // Connect the donor subgraph root to the splice point.
    if let Some(&donor_root) = id_remap.get(&pivot_b) {
        child.edges.push(Edge {
            source: splice_point,
            target: donor_root,
            port: 0,
            label: EdgeLabel::Argument,
        });
    }

    // Merge type environments: add any types from parent_b not already present.
    for (tid, tdef) in &parent_b.type_env.types {
        child.type_env.types.entry(*tid).or_insert_with(|| tdef.clone());
    }

    // Remove dangling edges before rehashing.
    remove_dangling_edges(&mut child);

    // Rehash the child graph.
    rehash(&mut child);

    // Reject if the child exceeds the program size limit — return parent_a unchanged.
    if child.nodes.len() > MAX_NODES_PER_INDIVIDUAL {
        return parent_a.clone();
    }

    child
}

// ---------------------------------------------------------------------------
// Large-program crossover: BFS-based subgraph exchange
// ---------------------------------------------------------------------------

/// Crossover for large programs using BFS-based subgraph extraction.
///
/// Instead of a fixed 2-hop neighborhood, extracts a subgraph that is
/// approximately `target_subgraph_fraction` (e.g. 0.1-0.3) of the donor
/// parent. This lets crossover exchange meaningful chunks of large programs.
pub fn crossover_large(
    parent_a: &SemanticGraph,
    parent_b: &SemanticGraph,
    rng: &mut impl Rng,
    target_subgraph_fraction: f32,
) -> SemanticGraph {
    let ids_a: Vec<NodeId> = parent_a.nodes.keys().copied().collect();
    let ids_b: Vec<NodeId> = parent_b.nodes.keys().copied().collect();

    if ids_a.is_empty() {
        return parent_b.clone();
    }
    if ids_b.is_empty() {
        return parent_a.clone();
    }

    // Start with parent_a as the base.
    let mut child = parent_a.clone();

    // Determine target donor subgraph size.
    let target_size = ((ids_b.len() as f32 * target_subgraph_fraction).ceil() as usize).max(1);

    // BFS from a random node in parent_b to collect a subgraph of the target size.
    let pivot_b = ids_b[rng.gen_range(0..ids_b.len())];
    let donor_nodes = collect_subgraph_bfs(parent_b, pivot_b, target_size);

    // Pick a splice point in parent_a.
    let splice_point = ids_a[rng.gen_range(0..ids_a.len())];

    // Transplant donor nodes with remapped IDs.
    let mut id_remap: BTreeMap<NodeId, NodeId> = BTreeMap::new();

    for &donor_id in &donor_nodes {
        if let Some(orig) = parent_b.nodes.get(&donor_id) {
            let mut node = orig.clone();
            node.salt = node.salt.wrapping_add(1);
            node.id = compute_node_id(&node);
            id_remap.insert(donor_id, node.id);
            child.nodes.insert(node.id, node);
        }
    }

    // Copy edges internal to the donor subgraph.
    let donor_set: BTreeSet<NodeId> = donor_nodes.iter().copied().collect();
    for edge in &parent_b.edges {
        if donor_set.contains(&edge.source) && donor_set.contains(&edge.target) {
            if let (Some(&new_src), Some(&new_tgt)) =
                (id_remap.get(&edge.source), id_remap.get(&edge.target))
            {
                child.edges.push(Edge {
                    source: new_src,
                    target: new_tgt,
                    port: edge.port,
                    label: edge.label,
                });
            }
        }
    }

    // Connect the donor subgraph root to the splice point.
    if let Some(&donor_root) = id_remap.get(&pivot_b) {
        child.edges.push(Edge {
            source: splice_point,
            target: donor_root,
            port: 0,
            label: EdgeLabel::Argument,
        });
    }

    // Merge type environments.
    for (tid, tdef) in &parent_b.type_env.types {
        child.type_env.types.entry(*tid).or_insert_with(|| tdef.clone());
    }

    // Remove dangling edges before rehashing.
    remove_dangling_edges(&mut child);

    rehash(&mut child);

    // Reject if the child exceeds the program size limit — return parent_a unchanged.
    if child.nodes.len() > MAX_NODES_PER_INDIVIDUAL {
        return parent_a.clone();
    }

    child
}

/// Collect node IDs via BFS from `start`, expanding until `target_size` nodes
/// are collected or the reachable subgraph is exhausted.
fn collect_subgraph_bfs(
    graph: &SemanticGraph,
    start: NodeId,
    target_size: usize,
) -> Vec<NodeId> {
    let mut visited = BTreeSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start);
    visited.insert(start);

    while let Some(node) = queue.pop_front() {
        if visited.len() >= target_size {
            break;
        }
        for edge in &graph.edges {
            if edge.source == node && graph.nodes.contains_key(&edge.target) {
                if visited.insert(edge.target) {
                    queue.push_back(edge.target);
                    if visited.len() >= target_size {
                        break;
                    }
                }
            }
        }
    }

    visited.into_iter().collect()
}

/// Collect node IDs reachable from `start` within `max_depth` hops.
fn collect_subgraph(graph: &SemanticGraph, start: NodeId, max_depth: usize) -> Vec<NodeId> {
    let mut visited = BTreeSet::new();
    let mut frontier = vec![(start, 0usize)];

    while let Some((node, depth)) = frontier.pop() {
        if depth > max_depth || !visited.insert(node) {
            continue;
        }
        for edge in &graph.edges {
            if edge.source == node && graph.nodes.contains_key(&edge.target) {
                frontier.push((edge.target, depth + 1));
            }
        }
    }

    visited.into_iter().collect()
}

/// Remove edges whose source or target nodes no longer exist in the graph.
///
/// Crossover splices donor nodes into a child graph; any donor edges whose
/// counterparts were not transplanted become dangling and must be pruned to
/// maintain graph integrity.
fn remove_dangling_edges(graph: &mut SemanticGraph) {
    graph.edges.retain(|e| {
        graph.nodes.contains_key(&e.source) && graph.nodes.contains_key(&e.target)
    });
}

/// Recompute the graph's semantic hash.
fn rehash(graph: &mut SemanticGraph) {
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
// Codec-based crossover (Gen1 embedding integration)
// ---------------------------------------------------------------------------

/// Crossover via the embedding codec: encode -> interpolate -> decode -> repair.
///
/// Falls back to subgraph crossover on codec failure.
///
/// 
pub fn crossover_via_codec(
    codec: &dyn iris_types::codec::GraphEmbeddingCodec,
    parent_a: &SemanticGraph,
    parent_b: &SemanticGraph,
    alpha: f32,
    rng: &mut impl Rng,
) -> SemanticGraph {
    // Encode parents, interpolate, decode, repair
    let emb_a = codec.encode(parent_a);
    let emb_b = codec.encode(parent_b);
    let blended = codec.interpolate(&emb_a.dims, &emb_b.dims, alpha);
    let candidate = codec.decode(&blended);
    match codec.structural_repair(candidate) {
        Ok(child) => child,
        Err(_) => crossover(parent_a, parent_b, rng),
    }
}
