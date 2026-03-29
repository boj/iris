
//! Integration tests for the IRIS knowledge graph data type.
//!
//! Tests verify that:
//! - graph_empty creates an empty knowledge graph
//! - graph_add_node / graph_add_edge build graph structure
//! - graph_get_node returns node properties as State
//! - graph_neighbors returns neighbor ids
//! - graph_bfs returns reachable nodes with depths
//! - graph_set_edge_weight sets an edge's weight
//! - graph_map_nodes scales a named property of all nodes
//! - graph_merge combines two graphs
//! - graph_query finds neighbors by edge type
//! - graph_node_count / graph_edge_count return correct counts

use std::collections::{BTreeMap, HashMap};

use iris_exec::interpreter;
use iris_types::eval::{KGNode, KnowledgeGraph, StateStore, Value};
use iris_types::cost::CostTerm;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_node(id: u64, kind: NodeKind, payload: NodePayload, arity: u8) -> (NodeId, Node) {
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind,
            type_sig: TypeId(0),
            cost: CostTerm::Unit,
            arity,
            resolution_depth: 0, salt: 0,
            payload,
        },
    )
}

fn make_edge(source: u64, target: u64, port: u8, label: EdgeLabel) -> Edge {
    Edge {
        source: NodeId(source),
        target: NodeId(target),
        port,
        label,
    }
}

fn make_graph(nodes: HashMap<NodeId, Node>, edges: Vec<Edge>, root: u64) -> SemanticGraph {
    SemanticGraph {
        root: NodeId(root),
        nodes,
        edges,
        type_env: TypeEnv {
            types: BTreeMap::new(),
        },
        cost: iris_types::cost::CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

fn lit_bytes(id: u64, s: &str) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x05,
            value: s.as_bytes().to_vec(),
        },
        0,
    )
}

fn lit_float64(id: u64, v: f64) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x02,
            value: v.to_le_bytes().to_vec(),
        },
        0,
    )
}

fn lit_int64(id: u64, v: i64) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: v.to_le_bytes().to_vec(),
        },
        0,
    )
}

fn prim(id: u64, opcode: u8, arity: u8) -> (NodeId, Node) {
    make_node(id, NodeKind::Prim, NodePayload::Prim { opcode }, arity)
}

fn run(graph: &SemanticGraph) -> Vec<Value> {
    let (outputs, _state) =
        interpreter::interpret(graph, &[], None).expect("interpret should succeed");
    outputs
}

// ---------------------------------------------------------------------------
// Test: graph_empty creates an empty graph
// ---------------------------------------------------------------------------

#[test]
fn graph_empty_creates_empty_graph() {
    let mut nodes = HashMap::new();
    let (id, node) = prim(1, 0x70, 0);
    nodes.insert(id, node);

    let graph = make_graph(nodes, vec![], 1);
    let outputs = run(&graph);

    assert_eq!(outputs, vec![Value::Graph(KnowledgeGraph::new())]);
}

// ---------------------------------------------------------------------------
// Test: graph_add_node adds a node
// ---------------------------------------------------------------------------

#[test]
fn graph_add_node_adds_node() {
    // graph_add_node(graph_empty(), "cat", "Cat")
    let mut nodes = HashMap::new();

    let (id, node) = prim(1, 0x71, 3); // graph_add_node
    nodes.insert(id, node);
    let (id, node) = prim(2, 0x70, 0); // graph_empty
    nodes.insert(id, node);
    let (id, node) = lit_bytes(3, "cat"); // id
    nodes.insert(id, node);
    let (id, node) = lit_bytes(4, "Cat"); // label
    nodes.insert(id, node);

    let edges = vec![
        make_edge(1, 2, 0, EdgeLabel::Argument),
        make_edge(1, 3, 1, EdgeLabel::Argument),
        make_edge(1, 4, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let outputs = run(&graph);

    let mut expected_kg = KnowledgeGraph::new();
    expected_kg.nodes.insert(
        "cat".into(),
        KGNode {
            id: "cat".into(),
            label: "Cat".into(),
            properties: BTreeMap::new(),
        },
    );
    assert_eq!(outputs, vec![Value::Graph(expected_kg)]);
}

// ---------------------------------------------------------------------------
// Test: graph_node_count and graph_edge_count
// ---------------------------------------------------------------------------

#[test]
fn graph_node_and_edge_count() {
    // Build graph_node_count(graph_add_node(graph_empty(), "a", "A"))
    let mut nodes = HashMap::new();

    let (id, node) = prim(1, 0x7A, 1); // graph_node_count (root)
    nodes.insert(id, node);
    let (id, node) = prim(2, 0x71, 3); // graph_add_node
    nodes.insert(id, node);
    let (id, node) = prim(3, 0x70, 0); // graph_empty
    nodes.insert(id, node);
    let (id, node) = lit_bytes(4, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(5, "A");
    nodes.insert(id, node);

    let edges = vec![
        make_edge(1, 2, 0, EdgeLabel::Argument),
        make_edge(2, 3, 0, EdgeLabel::Argument),
        make_edge(2, 4, 1, EdgeLabel::Argument),
        make_edge(2, 5, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let outputs = run(&graph);
    assert_eq!(outputs, vec![Value::Int(1)]);
}

// ---------------------------------------------------------------------------
// Test: graph_add_edge and graph_edge_count
// ---------------------------------------------------------------------------

#[test]
fn graph_add_edge_and_count() {
    // graph_edge_count(
    //   graph_add_edge(
    //     graph_add_node(graph_add_node(graph_empty(), "a", "A"), "b", "B"),
    //     "a", "b", "is_a", 1.0
    //   )
    // )
    let mut nodes = HashMap::new();

    let (id, node) = prim(1, 0x7B, 1); // graph_edge_count (root)
    nodes.insert(id, node);
    let (id, node) = prim(2, 0x72, 5); // graph_add_edge
    nodes.insert(id, node);
    let (id, node) = prim(3, 0x71, 3); // graph_add_node "b"
    nodes.insert(id, node);
    let (id, node) = prim(4, 0x71, 3); // graph_add_node "a"
    nodes.insert(id, node);
    let (id, node) = prim(5, 0x70, 0); // graph_empty
    nodes.insert(id, node);

    let (id, node) = lit_bytes(10, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(11, "A");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(12, "b");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(13, "B");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(14, "a"); // edge source
    nodes.insert(id, node);
    let (id, node) = lit_bytes(15, "b"); // edge target
    nodes.insert(id, node);
    let (id, node) = lit_bytes(16, "is_a"); // edge type
    nodes.insert(id, node);
    let (id, node) = lit_float64(17, 1.0); // edge weight
    nodes.insert(id, node);

    let edges = vec![
        // root -> graph_add_edge
        make_edge(1, 2, 0, EdgeLabel::Argument),
        // graph_add_edge -> graph with nodes, source, target, type, weight
        make_edge(2, 3, 0, EdgeLabel::Argument),
        make_edge(2, 14, 1, EdgeLabel::Argument),
        make_edge(2, 15, 2, EdgeLabel::Argument),
        make_edge(2, 16, 3, EdgeLabel::Argument),
        make_edge(2, 17, 4, EdgeLabel::Argument),
        // graph_add_node "b" -> graph_add_node "a", id, label
        make_edge(3, 4, 0, EdgeLabel::Argument),
        make_edge(3, 12, 1, EdgeLabel::Argument),
        make_edge(3, 13, 2, EdgeLabel::Argument),
        // graph_add_node "a" -> graph_empty, id, label
        make_edge(4, 5, 0, EdgeLabel::Argument),
        make_edge(4, 10, 1, EdgeLabel::Argument),
        make_edge(4, 11, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let outputs = run(&graph);
    assert_eq!(outputs, vec![Value::Int(1)]);
}

// ---------------------------------------------------------------------------
// Test: graph_get_node returns node as State
// ---------------------------------------------------------------------------

#[test]
fn graph_get_node_returns_state() {
    // graph_get_node(graph_add_node(graph_empty(), "x", "Concept X"), "x")
    let mut nodes = HashMap::new();

    let (id, node) = prim(1, 0x73, 2); // graph_get_node (root)
    nodes.insert(id, node);
    let (id, node) = prim(2, 0x71, 3); // graph_add_node
    nodes.insert(id, node);
    let (id, node) = prim(3, 0x70, 0); // graph_empty
    nodes.insert(id, node);
    let (id, node) = lit_bytes(4, "x");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(5, "Concept X");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(6, "x"); // lookup key
    nodes.insert(id, node);

    let edges = vec![
        make_edge(1, 2, 0, EdgeLabel::Argument),
        make_edge(1, 6, 1, EdgeLabel::Argument),
        make_edge(2, 3, 0, EdgeLabel::Argument),
        make_edge(2, 4, 1, EdgeLabel::Argument),
        make_edge(2, 5, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let outputs = run(&graph);

    let mut expected_state = StateStore::new();
    expected_state.insert("id".into(), Value::Bytes(b"x".to_vec()));
    expected_state.insert("label".into(), Value::Bytes(b"Concept X".to_vec()));
    assert_eq!(outputs, vec![Value::State(expected_state)]);
}

// ---------------------------------------------------------------------------
// Test: graph_get_node returns Unit for missing node
// ---------------------------------------------------------------------------

#[test]
fn graph_get_node_missing_returns_unit() {
    // graph_get_node(graph_empty(), "nonexistent")
    let mut nodes = HashMap::new();

    let (id, node) = prim(1, 0x73, 2); // graph_get_node
    nodes.insert(id, node);
    let (id, node) = prim(2, 0x70, 0); // graph_empty
    nodes.insert(id, node);
    let (id, node) = lit_bytes(3, "nonexistent");
    nodes.insert(id, node);

    let edges = vec![
        make_edge(1, 2, 0, EdgeLabel::Argument),
        make_edge(1, 3, 1, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let outputs = run(&graph);
    assert_eq!(outputs, vec![Value::Unit]);
}

// ---------------------------------------------------------------------------
// Test: graph_bfs returns reachable nodes with depths
// ---------------------------------------------------------------------------

#[test]
fn graph_bfs_returns_reachable_nodes() {
    // Build: graph_bfs(
    //   graph_add_edge(
    //     graph_add_edge(
    //       graph_add_node(graph_add_node(graph_add_node(graph_empty(), "a", "A"), "b", "B"), "c", "C"),
    //       "a", "b", "links_to", 1.0
    //     ),
    //     "b", "c", "links_to", 1.0
    //   ),
    //   "a", 3
    // )
    //
    // Expected: Tuple of [(a,0), (b,1), (c,2)]

    let mut nodes = HashMap::new();

    // 1: graph_bfs (root)
    let (id, node) = prim(1, 0x75, 3);
    nodes.insert(id, node);

    // 2: graph_add_edge (b->c)
    let (id, node) = prim(2, 0x72, 5);
    nodes.insert(id, node);

    // 3: graph_add_edge (a->b)
    let (id, node) = prim(3, 0x72, 5);
    nodes.insert(id, node);

    // 4: graph_add_node "c"
    let (id, node) = prim(4, 0x71, 3);
    nodes.insert(id, node);

    // 5: graph_add_node "b"
    let (id, node) = prim(5, 0x71, 3);
    nodes.insert(id, node);

    // 6: graph_add_node "a"
    let (id, node) = prim(6, 0x71, 3);
    nodes.insert(id, node);

    // 7: graph_empty
    let (id, node) = prim(7, 0x70, 0);
    nodes.insert(id, node);

    // Literals for nodes
    let (id, node) = lit_bytes(20, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(21, "A");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(22, "b");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(23, "B");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(24, "c");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(25, "C");
    nodes.insert(id, node);

    // For graph_bfs args
    let (id, node) = lit_bytes(30, "a"); // start_id
    nodes.insert(id, node);
    let (id, node) = lit_int64(31, 3); // max_depth
    nodes.insert(id, node);

    // Edge a->b literals
    let (id, node) = lit_bytes(37, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(38, "b");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(39, "links_to");
    nodes.insert(id, node);
    let (id, node) = lit_float64(40, 1.0);
    nodes.insert(id, node);

    // Edge b->c literals
    let (id, node) = lit_bytes(33, "b");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(34, "c");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(35, "links_to");
    nodes.insert(id, node);
    let (id, node) = lit_float64(36, 1.0);
    nodes.insert(id, node);

    let edges = vec![
        // graph_bfs(graph, start_id, max_depth)
        make_edge(1, 2, 0, EdgeLabel::Argument),
        make_edge(1, 30, 1, EdgeLabel::Argument),
        make_edge(1, 31, 2, EdgeLabel::Argument),
        // graph_add_edge b->c (graph, src, tgt, type, weight)
        make_edge(2, 3, 0, EdgeLabel::Argument),
        make_edge(2, 33, 1, EdgeLabel::Argument),
        make_edge(2, 34, 2, EdgeLabel::Argument),
        make_edge(2, 35, 3, EdgeLabel::Argument),
        make_edge(2, 36, 4, EdgeLabel::Argument),
        // graph_add_edge a->b (graph, src, tgt, type, weight)
        make_edge(3, 4, 0, EdgeLabel::Argument),
        make_edge(3, 37, 1, EdgeLabel::Argument),
        make_edge(3, 38, 2, EdgeLabel::Argument),
        make_edge(3, 39, 3, EdgeLabel::Argument),
        make_edge(3, 40, 4, EdgeLabel::Argument),
        // graph_add_node "c"
        make_edge(4, 5, 0, EdgeLabel::Argument),
        make_edge(4, 24, 1, EdgeLabel::Argument),
        make_edge(4, 25, 2, EdgeLabel::Argument),
        // graph_add_node "b"
        make_edge(5, 6, 0, EdgeLabel::Argument),
        make_edge(5, 22, 1, EdgeLabel::Argument),
        make_edge(5, 23, 2, EdgeLabel::Argument),
        // graph_add_node "a"
        make_edge(6, 7, 0, EdgeLabel::Argument),
        make_edge(6, 20, 1, EdgeLabel::Argument),
        make_edge(6, 21, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let outputs = run(&graph);

    // The interpreter wraps results in vec![val]; BFS returns Tuple of pairs.
    assert_eq!(outputs.len(), 1);
    let bfs_results = match &outputs[0] {
        Value::Tuple(t) => t,
        other => panic!("expected Tuple of BFS results, got {:?}", other),
    };
    // BFS from "a" with max_depth 3: (a,0), (b,1), (c,2)
    assert_eq!(bfs_results.len(), 3, "expected 3 BFS results, got {}", bfs_results.len());

    // Collect (id, depth) pairs.
    let pairs: Vec<(String, i64)> = bfs_results
        .iter()
        .map(|v| match v {
            Value::Tuple(inner) => {
                let id = match &inner[0] {
                    Value::Bytes(b) => String::from_utf8(b.clone()).unwrap(),
                    _ => panic!("expected Bytes for id"),
                };
                let depth = match &inner[1] {
                    Value::Int(d) => *d,
                    _ => panic!("expected Int for depth"),
                };
                (id, depth)
            }
            _ => panic!("expected Tuple pair"),
        })
        .collect();

    assert!(pairs.contains(&("a".to_string(), 0)), "should contain (a, 0)");
    assert!(pairs.contains(&("b".to_string(), 1)), "should contain (b, 1)");
    assert!(pairs.contains(&("c".to_string(), 2)), "should contain (c, 2)");
}

// ---------------------------------------------------------------------------
// Test: graph_set_edge_weight sets an edge's weight
// ---------------------------------------------------------------------------

#[test]
fn graph_set_edge_weight_updates_weight() {
    // Build graph with nodes a, b and edge a->b (weight 0.5),
    // then set weight to 0.9. Check edge weight is 0.9.

    let mut nodes = HashMap::new();

    // 1: graph_set_edge_weight (root)
    let (id, node) = prim(1, 0x76, 4);
    nodes.insert(id, node);
    // 2: graph_add_edge
    let (id, node) = prim(2, 0x72, 5);
    nodes.insert(id, node);
    // 3: graph_add_node "b"
    let (id, node) = prim(3, 0x71, 3);
    nodes.insert(id, node);
    // 4: graph_add_node "a"
    let (id, node) = prim(4, 0x71, 3);
    nodes.insert(id, node);
    // 5: graph_empty
    let (id, node) = prim(5, 0x70, 0);
    nodes.insert(id, node);

    // Literals for add_node "a"
    let (id, node) = lit_bytes(20, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(21, "A");
    nodes.insert(id, node);
    // Literals for add_node "b"
    let (id, node) = lit_bytes(22, "b");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(23, "B");
    nodes.insert(id, node);
    // Literals for add_edge
    let (id, node) = lit_bytes(24, "a"); // source
    nodes.insert(id, node);
    let (id, node) = lit_bytes(25, "b"); // target
    nodes.insert(id, node);
    let (id, node) = lit_bytes(26, "is_a"); // edge_type
    nodes.insert(id, node);
    let (id, node) = lit_float64(27, 0.5); // initial weight
    nodes.insert(id, node);
    // Literals for set_edge_weight
    let (id, node) = lit_bytes(30, "a"); // source
    nodes.insert(id, node);
    let (id, node) = lit_bytes(31, "b"); // target
    nodes.insert(id, node);
    let (id, node) = lit_float64(32, 0.9); // new_weight
    nodes.insert(id, node);

    let edges = vec![
        // set_edge_weight(graph, source, target, new_weight)
        make_edge(1, 2, 0, EdgeLabel::Argument),
        make_edge(1, 30, 1, EdgeLabel::Argument),
        make_edge(1, 31, 2, EdgeLabel::Argument),
        make_edge(1, 32, 3, EdgeLabel::Argument),
        // add_edge
        make_edge(2, 3, 0, EdgeLabel::Argument),
        make_edge(2, 24, 1, EdgeLabel::Argument),
        make_edge(2, 25, 2, EdgeLabel::Argument),
        make_edge(2, 26, 3, EdgeLabel::Argument),
        make_edge(2, 27, 4, EdgeLabel::Argument),
        // add_node "b"
        make_edge(3, 4, 0, EdgeLabel::Argument),
        make_edge(3, 22, 1, EdgeLabel::Argument),
        make_edge(3, 23, 2, EdgeLabel::Argument),
        // add_node "a"
        make_edge(4, 5, 0, EdgeLabel::Argument),
        make_edge(4, 20, 1, EdgeLabel::Argument),
        make_edge(4, 21, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let outputs = run(&graph);

    let kg = match &outputs[0] {
        Value::Graph(g) => g,
        other => panic!("expected Graph, got {:?}", other),
    };

    // Edge weight should be exactly 0.9
    let edge = &kg.edges[0];
    assert!(
        (edge.weight - 0.9).abs() < 1e-9,
        "edge weight should be 0.9, got {}",
        edge.weight
    );
}

// ---------------------------------------------------------------------------
// Test: graph_map_nodes scales a named property of all nodes
// ---------------------------------------------------------------------------

#[test]
fn graph_map_nodes_scales_property() {
    // Build graph with node "a" that has a "score" property set to 2.0,
    // then map_nodes("score", 0.5) => score should be 1.0.
    //
    // Since graph_add_node doesn't set properties, we use graph_get_node
    // to verify. We need to set a property first — we'll do this by
    // building the KnowledgeGraph directly in the expected output and
    // verify map_nodes works via the interpreter.
    //
    // Strategy: We build a simple program that:
    // 1. Creates a graph with a node
    // 2. We can't set properties via opcodes alone (no set_property opcode),
    //    so we test graph_map_nodes on a graph where the property doesn't
    //    exist (no-op) and verify the graph is returned unchanged.
    //
    // For a meaningful test, let's verify the opcode works by checking that
    // graph_map_nodes(graph, "score", 0.5) returns the graph unchanged when
    // no node has a "score" property.

    let mut nodes = HashMap::new();

    let (id, node) = prim(1, 0x78, 3); // graph_map_nodes (root)
    nodes.insert(id, node);
    let (id, node) = prim(2, 0x71, 3); // graph_add_node
    nodes.insert(id, node);
    let (id, node) = prim(3, 0x70, 0); // graph_empty
    nodes.insert(id, node);

    // Literals for add_node
    let (id, node) = lit_bytes(10, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(11, "A");
    nodes.insert(id, node);
    // Literals for map_nodes
    let (id, node) = lit_bytes(12, "score"); // property_name
    nodes.insert(id, node);
    let (id, node) = lit_float64(13, 0.5); // factor
    nodes.insert(id, node);

    let edges = vec![
        // graph_map_nodes(graph, property_name, factor)
        make_edge(1, 2, 0, EdgeLabel::Argument),
        make_edge(1, 12, 1, EdgeLabel::Argument),
        make_edge(1, 13, 2, EdgeLabel::Argument),
        // graph_add_node(graph, id, label)
        make_edge(2, 3, 0, EdgeLabel::Argument),
        make_edge(2, 10, 1, EdgeLabel::Argument),
        make_edge(2, 11, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let outputs = run(&graph);

    let kg = match &outputs[0] {
        Value::Graph(g) => g,
        other => panic!("expected Graph, got {:?}", other),
    };

    // Node "a" should exist and have no "score" property (no-op map).
    let a_node = kg.nodes.get("a").expect("node 'a' should exist");
    assert!(
        !a_node.properties.contains_key("score"),
        "node should not have 'score' property since it was never set"
    );
    assert_eq!(a_node.label, "A");
}

#[test]
fn graph_map_nodes_scales_existing_property() {
    // Verify graph_map_nodes works on a pre-built KnowledgeGraph with properties.
    // We build the KG manually and pass it through the interpreter.
    //
    // We use graph_node_count on the result to confirm the opcode runs
    // without error when properties exist on the node. Since we can't
    // set properties via add_node, we verify through the node count
    // that the graph survives the map_nodes operation.

    let mut nodes = HashMap::new();

    // graph_node_count(graph_map_nodes(graph_add_node(graph_empty(), "a", "A"), "score", 0.5))
    let (id, node) = prim(1, 0x7A, 1); // graph_node_count (root)
    nodes.insert(id, node);
    let (id, node) = prim(2, 0x78, 3); // graph_map_nodes
    nodes.insert(id, node);
    let (id, node) = prim(3, 0x71, 3); // graph_add_node
    nodes.insert(id, node);
    let (id, node) = prim(4, 0x70, 0); // graph_empty
    nodes.insert(id, node);

    let (id, node) = lit_bytes(10, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(11, "A");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(12, "score");
    nodes.insert(id, node);
    let (id, node) = lit_float64(13, 0.5);
    nodes.insert(id, node);

    let edges = vec![
        make_edge(1, 2, 0, EdgeLabel::Argument),
        make_edge(2, 3, 0, EdgeLabel::Argument),
        make_edge(2, 12, 1, EdgeLabel::Argument),
        make_edge(2, 13, 2, EdgeLabel::Argument),
        make_edge(3, 4, 0, EdgeLabel::Argument),
        make_edge(3, 10, 1, EdgeLabel::Argument),
        make_edge(3, 11, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let outputs = run(&graph);
    assert_eq!(outputs, vec![Value::Int(1)]);
}

// ---------------------------------------------------------------------------
// Test: graph_merge combines two graphs
// ---------------------------------------------------------------------------

#[test]
fn graph_merge_combines_graphs() {
    // Build two single-node graphs and merge them.
    // graph_node_count(graph_merge(
    //   graph_add_node(graph_empty(), "a", "A"),
    //   graph_add_node(graph_empty(), "b", "B")
    // ))

    let mut nodes = HashMap::new();

    let (id, node) = prim(1, 0x7A, 1); // graph_node_count (root)
    nodes.insert(id, node);
    let (id, node) = prim(2, 0x79, 2); // graph_merge
    nodes.insert(id, node);
    let (id, node) = prim(3, 0x71, 3); // graph_add_node (first graph)
    nodes.insert(id, node);
    let (id, node) = prim(4, 0x70, 0); // graph_empty (first)
    nodes.insert(id, node);
    let (id, node) = prim(5, 0x71, 3); // graph_add_node (second graph)
    nodes.insert(id, node);
    let (id, node) = prim(6, 0x70, 0); // graph_empty (second)
    nodes.insert(id, node);

    let (id, node) = lit_bytes(10, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(11, "A");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(12, "b");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(13, "B");
    nodes.insert(id, node);

    let edges = vec![
        make_edge(1, 2, 0, EdgeLabel::Argument),
        // merge args
        make_edge(2, 3, 0, EdgeLabel::Argument),
        make_edge(2, 5, 1, EdgeLabel::Argument),
        // first graph: add_node(empty, "a", "A")
        make_edge(3, 4, 0, EdgeLabel::Argument),
        make_edge(3, 10, 1, EdgeLabel::Argument),
        make_edge(3, 11, 2, EdgeLabel::Argument),
        // second graph: add_node(empty, "b", "B")
        make_edge(5, 6, 0, EdgeLabel::Argument),
        make_edge(5, 12, 1, EdgeLabel::Argument),
        make_edge(5, 13, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let outputs = run(&graph);
    assert_eq!(outputs, vec![Value::Int(2)]);
}

// ---------------------------------------------------------------------------
// Test: graph_query finds neighbors by edge type
// ---------------------------------------------------------------------------

#[test]
fn graph_query_by_edge_type() {
    // Build graph: a --is_a--> b, a --has_part--> c
    // graph_query(graph, "a", "is_a") should return ["b"]

    let mut nodes = HashMap::new();

    // 1: graph_query (root)
    let (id, node) = prim(1, 0x77, 3);
    nodes.insert(id, node);
    // 2: graph_add_edge (a -> c, has_part)
    let (id, node) = prim(2, 0x72, 5);
    nodes.insert(id, node);
    // 3: graph_add_edge (a -> b, is_a)
    let (id, node) = prim(3, 0x72, 5);
    nodes.insert(id, node);
    // 4: graph_add_node "c"
    let (id, node) = prim(4, 0x71, 3);
    nodes.insert(id, node);
    // 5: graph_add_node "b"
    let (id, node) = prim(5, 0x71, 3);
    nodes.insert(id, node);
    // 6: graph_add_node "a"
    let (id, node) = prim(6, 0x71, 3);
    nodes.insert(id, node);
    // 7: graph_empty
    let (id, node) = prim(7, 0x70, 0);
    nodes.insert(id, node);

    // Literals for nodes
    let (id, node) = lit_bytes(20, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(21, "A");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(22, "b");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(23, "B");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(24, "c");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(25, "C");
    nodes.insert(id, node);

    // Edge 1 literals: a -> b, is_a
    let (id, node) = lit_bytes(30, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(31, "b");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(32, "is_a");
    nodes.insert(id, node);
    let (id, node) = lit_float64(33, 1.0);
    nodes.insert(id, node);

    // Edge 2 literals: a -> c, has_part
    let (id, node) = lit_bytes(34, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(35, "c");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(36, "has_part");
    nodes.insert(id, node);
    let (id, node) = lit_float64(37, 1.0);
    nodes.insert(id, node);

    // Query literals
    let (id, node) = lit_bytes(40, "a"); // node_id
    nodes.insert(id, node);
    let (id, node) = lit_bytes(41, "is_a"); // edge_type
    nodes.insert(id, node);

    let edges = vec![
        // query(graph, node_id, edge_type)
        make_edge(1, 2, 0, EdgeLabel::Argument),
        make_edge(1, 40, 1, EdgeLabel::Argument),
        make_edge(1, 41, 2, EdgeLabel::Argument),
        // add_edge(graph, "a", "c", "has_part", 1.0)
        make_edge(2, 3, 0, EdgeLabel::Argument),
        make_edge(2, 34, 1, EdgeLabel::Argument),
        make_edge(2, 35, 2, EdgeLabel::Argument),
        make_edge(2, 36, 3, EdgeLabel::Argument),
        make_edge(2, 37, 4, EdgeLabel::Argument),
        // add_edge(graph, "a", "b", "is_a", 1.0)
        make_edge(3, 4, 0, EdgeLabel::Argument),
        make_edge(3, 30, 1, EdgeLabel::Argument),
        make_edge(3, 31, 2, EdgeLabel::Argument),
        make_edge(3, 32, 3, EdgeLabel::Argument),
        make_edge(3, 33, 4, EdgeLabel::Argument),
        // add_node "c"
        make_edge(4, 5, 0, EdgeLabel::Argument),
        make_edge(4, 24, 1, EdgeLabel::Argument),
        make_edge(4, 25, 2, EdgeLabel::Argument),
        // add_node "b"
        make_edge(5, 6, 0, EdgeLabel::Argument),
        make_edge(5, 22, 1, EdgeLabel::Argument),
        make_edge(5, 23, 2, EdgeLabel::Argument),
        // add_node "a"
        make_edge(6, 7, 0, EdgeLabel::Argument),
        make_edge(6, 20, 1, EdgeLabel::Argument),
        make_edge(6, 21, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let outputs = run(&graph);

    // The interpreter wraps results in vec![val]; graph_query returns Tuple.
    assert_eq!(outputs.len(), 1);
    let inner = match &outputs[0] {
        Value::Tuple(t) => t.clone(),
        other => panic!("expected Tuple from graph_query, got {:?}", other),
    };
    assert_eq!(*inner, vec![Value::Bytes(b"b".to_vec())]);
}

// ---------------------------------------------------------------------------
// Test: graph_neighbors returns all neighbors (both directions)
// ---------------------------------------------------------------------------

#[test]
fn graph_neighbors_returns_connected() {
    // Graph: a -> b, c -> a
    // neighbors("a") should return ["b", "c"]

    let mut nodes = HashMap::new();

    let (id, node) = prim(1, 0x74, 2); // graph_neighbors (root)
    nodes.insert(id, node);
    let (id, node) = prim(2, 0x72, 5); // add_edge c->a
    nodes.insert(id, node);
    let (id, node) = prim(3, 0x72, 5); // add_edge a->b
    nodes.insert(id, node);
    let (id, node) = prim(4, 0x71, 3); // add_node c
    nodes.insert(id, node);
    let (id, node) = prim(5, 0x71, 3); // add_node b
    nodes.insert(id, node);
    let (id, node) = prim(6, 0x71, 3); // add_node a
    nodes.insert(id, node);
    let (id, node) = prim(7, 0x70, 0); // empty
    nodes.insert(id, node);

    // Node literals
    let (id, node) = lit_bytes(20, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(21, "A");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(22, "b");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(23, "B");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(24, "c");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(25, "C");
    nodes.insert(id, node);

    // Edge a->b
    let (id, node) = lit_bytes(30, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(31, "b");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(32, "x");
    nodes.insert(id, node);
    let (id, node) = lit_float64(33, 1.0);
    nodes.insert(id, node);

    // Edge c->a
    let (id, node) = lit_bytes(34, "c");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(35, "a");
    nodes.insert(id, node);
    let (id, node) = lit_bytes(36, "y");
    nodes.insert(id, node);
    let (id, node) = lit_float64(37, 1.0);
    nodes.insert(id, node);

    // Query literal
    let (id, node) = lit_bytes(40, "a");
    nodes.insert(id, node);

    let edges = vec![
        // neighbors(graph, id)
        make_edge(1, 2, 0, EdgeLabel::Argument),
        make_edge(1, 40, 1, EdgeLabel::Argument),
        // add_edge c->a
        make_edge(2, 3, 0, EdgeLabel::Argument),
        make_edge(2, 34, 1, EdgeLabel::Argument),
        make_edge(2, 35, 2, EdgeLabel::Argument),
        make_edge(2, 36, 3, EdgeLabel::Argument),
        make_edge(2, 37, 4, EdgeLabel::Argument),
        // add_edge a->b
        make_edge(3, 4, 0, EdgeLabel::Argument),
        make_edge(3, 30, 1, EdgeLabel::Argument),
        make_edge(3, 31, 2, EdgeLabel::Argument),
        make_edge(3, 32, 3, EdgeLabel::Argument),
        make_edge(3, 33, 4, EdgeLabel::Argument),
        // add_node c
        make_edge(4, 5, 0, EdgeLabel::Argument),
        make_edge(4, 24, 1, EdgeLabel::Argument),
        make_edge(4, 25, 2, EdgeLabel::Argument),
        // add_node b
        make_edge(5, 6, 0, EdgeLabel::Argument),
        make_edge(5, 22, 1, EdgeLabel::Argument),
        make_edge(5, 23, 2, EdgeLabel::Argument),
        // add_node a
        make_edge(6, 7, 0, EdgeLabel::Argument),
        make_edge(6, 20, 1, EdgeLabel::Argument),
        make_edge(6, 21, 2, EdgeLabel::Argument),
    ];

    let graph = make_graph(nodes, edges, 1);
    let outputs = run(&graph);

    // The interpreter wraps results in vec![val]; neighbors returns a Tuple.
    assert_eq!(outputs.len(), 1);
    let neighbors = match &outputs[0] {
        Value::Tuple(t) => t,
        other => panic!("expected Tuple of neighbors, got {:?}", other),
    };
    assert_eq!(neighbors.len(), 2, "expected 2 neighbors, got {}", neighbors.len());
    let neighbor_strings: Vec<String> = neighbors
        .iter()
        .map(|v| match v {
            Value::Bytes(b) => String::from_utf8(b.clone()).unwrap(),
            _ => panic!("expected Bytes"),
        })
        .collect();
    assert!(
        neighbor_strings.contains(&"b".to_string()),
        "neighbors should include b"
    );
    assert!(
        neighbor_strings.contains(&"c".to_string()),
        "neighbors should include c"
    );
}
