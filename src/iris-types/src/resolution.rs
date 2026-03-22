//! Multi-Resolution Encoding (SPEC Section 10.3).
//!
//! Three resolution levels let programs be viewed at different abstraction
//! depths: Intent (depth 0), Architecture (depth 1), Implementation (depth 2).
//!
//! Nodes carry a `resolution_depth: u8` field. Viewing at level R shows only
//! nodes with depth <= R; hidden nodes are collapsed into summary Ref nodes
//! that preserve boundary types.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use crate::cost::CostTerm;
use crate::fragment::FragmentId;
use crate::graph::{
    Edge, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use crate::hash::SemanticHash;

// ---------------------------------------------------------------------------
// Resolution → depth mapping
// ---------------------------------------------------------------------------

/// Convert a `Resolution` enum variant to its numeric depth threshold.
fn resolution_depth(level: Resolution) -> u8 {
    match level {
        Resolution::Intent => 0,
        Resolution::Architecture => 1,
        Resolution::Implementation => 2,
    }
}

// ---------------------------------------------------------------------------
// resolve()
// ---------------------------------------------------------------------------

/// View a graph at a specific resolution level.
///
/// Nodes with `resolution_depth > level` are collapsed into summary `Ref`
/// nodes that preserve the original node's type signature (boundary type
/// preservation invariant).
pub fn resolve(graph: &SemanticGraph, level: Resolution) -> SemanticGraph {
    let max_depth = resolution_depth(level);

    // Partition nodes into visible and hidden sets.
    let mut visible: BTreeSet<NodeId> = BTreeSet::new();
    let mut hidden: BTreeSet<NodeId> = BTreeSet::new();

    for (&id, node) in &graph.nodes {
        if node.resolution_depth <= max_depth {
            visible.insert(id);
        } else {
            hidden.insert(id);
        }
    }

    // Build the resolved node map: visible nodes are kept as-is,
    // hidden nodes that are direct targets of visible parents get
    // replaced with summary Ref nodes.
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();

    // Copy all visible nodes.
    for &id in &visible {
        if let Some(node) = graph.nodes.get(&id) {
            nodes.insert(id, node.clone());
        }
    }

    // For each edge from a visible node to a hidden node, create a
    // summary Ref node preserving the hidden node's type signature.
    let mut summary_nodes: HashMap<NodeId, Node> = HashMap::new();
    for edge in &graph.edges {
        if visible.contains(&edge.source) && hidden.contains(&edge.target) {
            if !summary_nodes.contains_key(&edge.target) {
                if let Some(hidden_node) = graph.nodes.get(&edge.target) {
                    let summary = Node {
                        id: hidden_node.id,
                        kind: NodeKind::Ref,
                        type_sig: hidden_node.type_sig,
                        cost: CostTerm::Unit,
                        arity: 0,
                        resolution_depth: max_depth, salt: 0,
                        payload: NodePayload::Ref {
                            fragment_id: FragmentId([0; 32]),
                        },
                    };
                    summary_nodes.insert(hidden_node.id, summary);
                }
            }
        }
    }

    // Merge summary nodes into the result.
    for (id, node) in summary_nodes {
        nodes.insert(id, node);
    }

    // Keep only edges where both endpoints are in the resolved node set.
    let node_ids: BTreeSet<NodeId> = nodes.keys().copied().collect();
    let edges: Vec<Edge> = graph
        .edges
        .iter()
        .filter(|e| node_ids.contains(&e.source) && node_ids.contains(&e.target))
        .cloned()
        .collect();

    // Ensure root is present; if root was hidden, the graph is empty at this
    // resolution (degenerate case — return minimal graph).
    let root = if nodes.contains_key(&graph.root) {
        graph.root
    } else if let Some(&first) = nodes.keys().next() {
        first
    } else {
        // All nodes hidden at this depth — return single-node summary.
        let root_node = if let Some(orig_root) = graph.nodes.get(&graph.root) {
            Node {
                id: orig_root.id,
                kind: NodeKind::Ref,
                type_sig: orig_root.type_sig,
                cost: CostTerm::Unit,
                arity: 0,
                resolution_depth: 0, salt: 0,
                payload: NodePayload::Ref {
                    fragment_id: FragmentId([0; 32]),
                },
            }
        } else {
            return graph.clone();
        };
        let root_id = root_node.id;
        nodes.insert(root_id, root_node);
        root_id
    };

    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: graph.type_env.clone(),
        cost: graph.cost.clone(),
        resolution: level,
        hash: SemanticHash([0; 32]),
    }
}

// ---------------------------------------------------------------------------
// assign_resolution_depths()
// ---------------------------------------------------------------------------

/// Assign resolution depths heuristically to an unresolved graph.
///
/// - Root + boundary nodes -> depth 0 (intent)
/// - Direct children of root -> depth 1 (architecture)
/// - Everything else -> depth 2 (implementation)
pub fn assign_resolution_depths(graph: &mut SemanticGraph) {
    if graph.nodes.is_empty() {
        return;
    }

    // Build adjacency list (source -> targets).
    let mut children: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();
    for edge in &graph.edges {
        children.entry(edge.source).or_default().push(edge.target);
    }

    // BFS from root: level 0 = root, level 1 = direct children, level 2+ = rest.
    let mut visited: BTreeSet<NodeId> = BTreeSet::new();
    let mut queue: VecDeque<(NodeId, u8)> = VecDeque::new();

    // Root gets depth 0.
    queue.push_back((graph.root, 0));
    visited.insert(graph.root);

    while let Some((node_id, bfs_depth)) = queue.pop_front() {
        let resolution = match bfs_depth {
            0 => 0, // intent
            1 => 1, // architecture
            _ => 2, // implementation
        };

        if let Some(node) = graph.nodes.get_mut(&node_id) {
            node.resolution_depth = resolution;
        }

        if let Some(kids) = children.get(&node_id) {
            for &child_id in kids {
                if visited.insert(child_id) {
                    queue.push_back((child_id, bfs_depth + 1));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// resolve_mixed()
// ---------------------------------------------------------------------------

/// Get a mixed-resolution view: specific subtrees at different depths.
///
/// Nodes listed in `overrides` use the specified resolution; all other nodes
/// use the graph's current `resolution` field as the default level.
pub fn resolve_mixed(
    graph: &SemanticGraph,
    overrides: &BTreeMap<NodeId, Resolution>,
) -> SemanticGraph {
    let default_depth = resolution_depth(graph.resolution);

    // For each node, determine whether it is visible based on its effective
    // resolution threshold.
    let mut visible: BTreeSet<NodeId> = BTreeSet::new();
    let mut hidden: BTreeSet<NodeId> = BTreeSet::new();

    for (&id, node) in &graph.nodes {
        let max_depth = if let Some(&res) = overrides.get(&id) {
            resolution_depth(res)
        } else {
            default_depth
        };

        if node.resolution_depth <= max_depth {
            visible.insert(id);
        } else {
            hidden.insert(id);
        }
    }

    // Build result nodes.
    let mut nodes: HashMap<NodeId, Node> = HashMap::new();

    for &id in &visible {
        if let Some(node) = graph.nodes.get(&id) {
            nodes.insert(id, node.clone());
        }
    }

    // Summary Ref nodes for hidden targets of visible sources.
    for edge in &graph.edges {
        if visible.contains(&edge.source) && hidden.contains(&edge.target) {
            if !nodes.contains_key(&edge.target) {
                if let Some(hidden_node) = graph.nodes.get(&edge.target) {
                    let summary = Node {
                        id: hidden_node.id,
                        kind: NodeKind::Ref,
                        type_sig: hidden_node.type_sig,
                        cost: CostTerm::Unit,
                        arity: 0,
                        resolution_depth: hidden_node.resolution_depth, salt: 0,
                        payload: NodePayload::Ref {
                            fragment_id: FragmentId([0; 32]),
                        },
                    };
                    nodes.insert(hidden_node.id, summary);
                }
            }
        }
    }

    let node_ids: BTreeSet<NodeId> = nodes.keys().copied().collect();
    let edges: Vec<Edge> = graph
        .edges
        .iter()
        .filter(|e| node_ids.contains(&e.source) && node_ids.contains(&e.target))
        .cloned()
        .collect();

    let root = if nodes.contains_key(&graph.root) {
        graph.root
    } else if let Some(&first) = nodes.keys().next() {
        first
    } else {
        graph.root
    };

    SemanticGraph {
        root,
        nodes,
        edges,
        type_env: graph.type_env.clone(),
        cost: graph.cost.clone(),
        resolution: graph.resolution,
        hash: SemanticHash([0; 32]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cost::CostBound;
    use crate::graph::EdgeLabel;
    use crate::hash::compute_node_id;
    use crate::types::{PrimType, TypeDef, TypeEnv};

    fn make_three_level_graph() -> SemanticGraph {
        let int_def = TypeDef::Primitive(PrimType::Int);
        let int_id = crate::hash::compute_type_id(&int_def);
        let mut types = BTreeMap::new();
        types.insert(int_id, int_def);

        let mut root = Node {
            id: NodeId(0),
            kind: NodeKind::Prim,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 1,
            resolution_depth: 0, salt: 0,
            payload: NodePayload::Prim { opcode: 0x00 },
        };
        root.id = compute_node_id(&root);

        let mut arch = Node {
            id: NodeId(0),
            kind: NodeKind::Prim,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 1,
            resolution_depth: 1, salt: 0,
            payload: NodePayload::Prim { opcode: 0x01 },
        };
        arch.id = compute_node_id(&arch);

        let mut impl_node = Node {
            id: NodeId(0),
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0,
                value: 42i64.to_le_bytes().to_vec(),
            },
        };
        impl_node.id = compute_node_id(&impl_node);

        let root_id = root.id;
        let arch_id = arch.id;
        let impl_id = impl_node.id;

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root);
        nodes.insert(arch_id, arch);
        nodes.insert(impl_id, impl_node);

        let edges = vec![
            Edge {
                source: root_id,
                target: arch_id,
                port: 0,
                label: EdgeLabel::Argument,
            },
            Edge {
                source: arch_id,
                target: impl_id,
                port: 0,
                label: EdgeLabel::Argument,
            },
        ];

        SemanticGraph {
            root: root_id,
            nodes,
            edges,
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        }
    }

    #[test]
    fn resolve_intent_shows_root_only() {
        let graph = make_three_level_graph();
        let resolved = resolve(&graph, Resolution::Intent);

        // Root (depth 0) visible, arch (depth 1) becomes summary Ref,
        // impl (depth 2) is fully hidden.
        assert!(resolved.nodes.contains_key(&graph.root));
        // Should have root + summary ref for arch = 2 nodes.
        assert_eq!(resolved.nodes.len(), 2);
    }

    #[test]
    fn resolve_implementation_shows_all() {
        let graph = make_three_level_graph();
        let resolved = resolve(&graph, Resolution::Implementation);
        assert_eq!(resolved.nodes.len(), graph.nodes.len());
    }

    #[test]
    fn boundary_type_preservation() {
        let graph = make_three_level_graph();
        let r0 = resolve(&graph, Resolution::Intent);
        let r2 = resolve(&graph, Resolution::Implementation);

        // Root node's type_sig must be identical across resolutions.
        let root_type_r0 = r0.nodes.get(&graph.root).unwrap().type_sig;
        let root_type_r2 = r2.nodes.get(&graph.root).unwrap().type_sig;
        assert_eq!(root_type_r0, root_type_r2);
    }

    #[test]
    fn assign_depths_correct() {
        let int_def = TypeDef::Primitive(PrimType::Int);
        let int_id = crate::hash::compute_type_id(&int_def);
        let mut types = BTreeMap::new();
        types.insert(int_id, int_def);

        // Build a 3-node chain: root -> child -> grandchild, all depth 0.
        let mut root = Node {
            id: NodeId(0),
            kind: NodeKind::Prim,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 1,
            resolution_depth: 0, salt: 0,
            payload: NodePayload::Prim { opcode: 0x00 },
        };
        root.id = compute_node_id(&root);

        // Use different opcodes so nodes get unique IDs.
        let mut child = Node {
            id: NodeId(0),
            kind: NodeKind::Prim,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 1,
            resolution_depth: 0, salt: 0,
            payload: NodePayload::Prim { opcode: 0x01 },
        };
        child.id = compute_node_id(&child);

        let mut grandchild = Node {
            id: NodeId(0),
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0,
                value: 99i64.to_le_bytes().to_vec(),
            },
        };
        grandchild.id = compute_node_id(&grandchild);

        let root_id = root.id;
        let child_id = child.id;
        let gc_id = grandchild.id;

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root);
        nodes.insert(child_id, child);
        nodes.insert(gc_id, grandchild);

        let edges = vec![
            Edge {
                source: root_id,
                target: child_id,
                port: 0,
                label: EdgeLabel::Argument,
            },
            Edge {
                source: child_id,
                target: gc_id,
                port: 0,
                label: EdgeLabel::Argument,
            },
        ];

        let mut graph = SemanticGraph {
            root: root_id,
            nodes,
            edges,
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        assign_resolution_depths(&mut graph);

        assert_eq!(graph.nodes[&root_id].resolution_depth, 0);
        assert_eq!(graph.nodes[&child_id].resolution_depth, 1);
        assert_eq!(graph.nodes[&gc_id].resolution_depth, 2);
    }
}
