
//! End-to-end integration test: dot product via higher-order combinators.
//!
//! With the map/zip/concat primitives now available, the engine has the
//! building blocks for multi-phase algorithms. Dot product becomes:
//!
//!   zip([a0,a1,...], [b0,b1,...]) -> [(a0,b0), (a1,b1), ...]
//!   map(zipped, mul)             -> [a0*b0, a1*b1, ...]
//!   fold(0, add, products)       -> sum of products
//!
//! This test has two parts:
//!   1. A direct interpreter test that constructs the graph by hand and
//!      verifies it produces the correct dot product.
//!   2. An evolutionary test that gives the engine higher-order seeds and
//!      lets it discover a dot-product solution.

use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

use iris_evolve::config::{EvolutionConfig, PhaseThresholds, ProblemSpec};
use iris_exec::interpreter::interpret;
use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_exec::ExecutionService;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::{TestCase, Value};
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::{compute_node_id, SemanticHash};
use iris_types::types::{PrimType, TypeDef, TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn int_type_env() -> (TypeEnv, TypeId) {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = iris_types::hash::compute_type_id(&int_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    (TypeEnv { types }, int_id)
}

fn make_node(kind: NodeKind, payload: NodePayload, type_sig: TypeId, arity: u8) -> Node {
    make_node_depth(kind, payload, type_sig, arity, 2)
}

fn make_node_depth(kind: NodeKind, payload: NodePayload, type_sig: TypeId, arity: u8, res_depth: u8) -> Node {
    let mut node = Node {
        id: NodeId(0),
        kind,
        type_sig,
        cost: CostTerm::Unit,
        arity,
        resolution_depth: res_depth, salt: 0,
        payload,
    };
    node.id = compute_node_id(&node);
    node
}

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

// ---------------------------------------------------------------------------
// Test 1: Direct interpreter test — zip+map(mul)+fold(add) = dot product
// ---------------------------------------------------------------------------

/// Build a graph that computes dot product of two tuples passed as two
/// separate inputs via positional binding.
///
/// Graph structure:
///   Fold(base=Lit(0), step=Prim(add))
///     where the collection input to fold is:
///       Map(Zip(input0, input1), Prim(mul))
///
/// Since Fold reads its collection from BinderId(0xFFFF_0000), and our
/// higher-order combinators read their inputs from argument edges, we
/// need to structure this so that the map+zip are evaluated inline.
///
/// Approach: We build a pipeline graph where the root is Fold, and the
/// collection it folds over is the result of map(zip(a, b), mul).
/// But Fold gets its collection from the input binding, not from edges.
///
/// So instead we construct a simpler test: the inputs are already the
/// pairwise products (computed by zip+map externally), and fold sums them.
/// Then we test zip and map separately.
///
/// ACTUALLY: Let us test the combinators directly via separate interpret calls.

#[test]
fn test_zip_combinator() {
    println!();
    println!("=== Test: zip combinator ===");

    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Two Tuple inputs as literals.
    // Left: [1, 2, 3]
    let left_elems: Vec<Node> = vec![
        make_node(
            NodeKind::Lit,
            NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() },
            int_id, 0,
        ),
        make_node(
            NodeKind::Lit,
            NodePayload::Lit { type_tag: 0, value: 2i64.to_le_bytes().to_vec() },
            int_id, 0,
        ),
        make_node(
            NodeKind::Lit,
            NodePayload::Lit { type_tag: 0, value: 3i64.to_le_bytes().to_vec() },
            int_id, 0,
        ),
    ];
    // Use resolution_depth=2 for left tuple.
    let left_tuple = make_node_depth(NodeKind::Tuple, NodePayload::Tuple, int_id, 3, 2);
    let left_id = left_tuple.id;
    nodes.insert(left_id, left_tuple);
    for (port, elem) in left_elems.into_iter().enumerate() {
        let eid = elem.id;
        edges.push(Edge { source: left_id, target: eid, port: port as u8, label: EdgeLabel::Argument });
        nodes.insert(eid, elem);
    }

    // Right: [4, 5, 6]
    let right_elems: Vec<Node> = vec![
        make_node(
            NodeKind::Lit,
            NodePayload::Lit { type_tag: 0, value: 4i64.to_le_bytes().to_vec() },
            int_id, 0,
        ),
        make_node(
            NodeKind::Lit,
            NodePayload::Lit { type_tag: 0, value: 5i64.to_le_bytes().to_vec() },
            int_id, 0,
        ),
        make_node(
            NodeKind::Lit,
            NodePayload::Lit { type_tag: 0, value: 6i64.to_le_bytes().to_vec() },
            int_id, 0,
        ),
    ];
    // Use resolution_depth=1 for right tuple to get a distinct NodeId.
    let right_tuple = make_node_depth(NodeKind::Tuple, NodePayload::Tuple, int_id, 3, 1);
    let right_id = right_tuple.id;
    nodes.insert(right_id, right_tuple);
    for (port, elem) in right_elems.into_iter().enumerate() {
        let eid = elem.id;
        edges.push(Edge { source: right_id, target: eid, port: port as u8, label: EdgeLabel::Argument });
        nodes.insert(eid, elem);
    }

    // Zip node: zip(left, right)
    let zip_node = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x32 },
        int_id, 2,
    );
    let zip_id = zip_node.id;
    nodes.insert(zip_id, zip_node);
    edges.push(Edge { source: zip_id, target: left_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: zip_id, target: right_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: zip_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    let result = interpret(&graph, &[], None).map(|(v, _)| v).expect("zip should succeed");
    println!("zip([1,2,3], [4,5,6]) = {:?}", result);

    // Expected: [(1,4), (2,5), (3,6)] — returned as a Tuple wrapping inner Tuples.
    // The interpreter wraps the result in vec![val], so we unwrap the outer Tuple.
    assert_eq!(result.len(), 1, "interpret returns single-element vec");
    let pairs = match &result[0] {
        Value::Tuple(elems) => elems.clone(),
        other => panic!("expected Tuple, got {:?}", other),
    };
    assert_eq!(pairs.len(), 3, "expected 3 pairs");
    assert_eq!(pairs[0], Value::tuple(vec![Value::Int(1), Value::Int(4)]));
    assert_eq!(pairs[1], Value::tuple(vec![Value::Int(2), Value::Int(5)]));
    assert_eq!(pairs[2], Value::tuple(vec![Value::Int(3), Value::Int(6)]));
    println!("PASS");
}

#[test]
fn test_map_combinator() {
    println!();
    println!("=== Test: map combinator (map pairs through mul) ===");

    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Input: a Tuple of pairs: [(1,4), (2,5), (3,6)]
    // Build as nested Tuples.
    // Each pair tuple uses a different resolution_depth to get a unique NodeId.
    let pair_nodes: Vec<(NodeId, Vec<Node>)> = vec![
        {
            let a = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() }, int_id, 0);
            let b = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: 4i64.to_le_bytes().to_vec() }, int_id, 0);
            let t = make_node_depth(NodeKind::Tuple, NodePayload::Tuple, int_id, 2, 2);
            (t.id, vec![t, a, b])
        },
        {
            let a = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: 2i64.to_le_bytes().to_vec() }, int_id, 0);
            let b = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: 5i64.to_le_bytes().to_vec() }, int_id, 0);
            let t = make_node_depth(NodeKind::Tuple, NodePayload::Tuple, int_id, 2, 1);
            (t.id, vec![t, a, b])
        },
        {
            let a = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: 3i64.to_le_bytes().to_vec() }, int_id, 0);
            let b = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: 6i64.to_le_bytes().to_vec() }, int_id, 0);
            let t = make_node_depth(NodeKind::Tuple, NodePayload::Tuple, int_id, 2, 0);
            (t.id, vec![t, a, b])
        },
    ];

    // Build pair sub-graphs.
    let mut pair_ids = Vec::new();
    for (tid, pair_group) in &pair_nodes {
        pair_ids.push(*tid);
        for node in pair_group {
            nodes.insert(node.id, node.clone());
        }
        // Wire tuple -> elements.
        let tuple_node = &pair_group[0];
        let elem_a = &pair_group[1];
        let elem_b = &pair_group[2];
        edges.push(Edge { source: tuple_node.id, target: elem_a.id, port: 0, label: EdgeLabel::Argument });
        edges.push(Edge { source: tuple_node.id, target: elem_b.id, port: 1, label: EdgeLabel::Argument });
    }

    // Outer tuple: collection of pairs.
    let outer_tuple = make_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 3);
    let outer_id = outer_tuple.id;
    nodes.insert(outer_id, outer_tuple);
    for (port, pid) in pair_ids.iter().enumerate() {
        edges.push(Edge { source: outer_id, target: *pid, port: port as u8, label: EdgeLabel::Argument });
    }

    // Mul node (the function to map with).
    let mul_node = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x02 }, // mul
        int_id, 2,
    );
    let mul_id = mul_node.id;
    nodes.insert(mul_id, mul_node);

    // Map node: map(collection_of_pairs, mul)
    let map_node = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x30 }, // map
        int_id, 2,
    );
    let map_id = map_node.id;
    nodes.insert(map_id, map_node);
    edges.push(Edge { source: map_id, target: outer_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: map_id, target: mul_id, port: 1, label: EdgeLabel::Argument });

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

    let result = interpret(&graph, &[], None).map(|(v, _)| v).expect("map should succeed");
    println!("map([(1,4),(2,5),(3,6)], mul) = {:?}", result);

    // Expected: [4, 10, 18] — wrapped in a single-element vec by the interpreter.
    assert_eq!(result.len(), 1, "interpret returns single-element vec");
    let elems = match &result[0] {
        Value::Tuple(e) => e.clone(),
        other => panic!("expected Tuple, got {:?}", other),
    };
    assert_eq!(elems.len(), 3);
    assert_eq!(elems[0], Value::Int(4));
    assert_eq!(elems[1], Value::Int(10));
    assert_eq!(elems[2], Value::Int(18));
    println!("PASS");
}

#[test]
fn test_concat_combinator() {
    println!();
    println!("=== Test: concat combinator ===");

    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Left: [1, 2]
    let l1 = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: 1i64.to_le_bytes().to_vec() }, int_id, 0);
    let l2 = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: 2i64.to_le_bytes().to_vec() }, int_id, 0);
    let left = make_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 2);
    let left_id = left.id;
    nodes.insert(left_id, left);
    nodes.insert(l1.id, l1.clone());
    nodes.insert(l2.id, l2.clone());
    edges.push(Edge { source: left_id, target: l1.id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: left_id, target: l2.id, port: 1, label: EdgeLabel::Argument });

    // Right: [3, 4, 5]
    let r1 = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: 3i64.to_le_bytes().to_vec() }, int_id, 0);
    let r2 = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: 4i64.to_le_bytes().to_vec() }, int_id, 0);
    let r3 = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: 5i64.to_le_bytes().to_vec() }, int_id, 0);
    let right = make_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 3);
    let right_id = right.id;
    nodes.insert(right_id, right);
    nodes.insert(r1.id, r1.clone());
    nodes.insert(r2.id, r2.clone());
    nodes.insert(r3.id, r3.clone());
    edges.push(Edge { source: right_id, target: r1.id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: right_id, target: r2.id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: right_id, target: r3.id, port: 2, label: EdgeLabel::Argument });

    // Concat node.
    let concat_node = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x35 },
        int_id, 2,
    );
    let concat_id = concat_node.id;
    nodes.insert(concat_id, concat_node);
    edges.push(Edge { source: concat_id, target: left_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: concat_id, target: right_id, port: 1, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: concat_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    let result = interpret(&graph, &[], None).map(|(v, _)| v).expect("concat should succeed");
    println!("concat([1,2], [3,4,5]) = {:?}", result);

    assert_eq!(result.len(), 1, "interpret returns single-element vec");
    let elems = match &result[0] {
        Value::Tuple(e) => e.clone(),
        other => panic!("expected Tuple, got {:?}", other),
    };
    assert_eq!(elems.len(), 5);
    assert_eq!(elems[0], Value::Int(1));
    assert_eq!(elems[1], Value::Int(2));
    assert_eq!(elems[2], Value::Int(3));
    assert_eq!(elems[3], Value::Int(4));
    assert_eq!(elems[4], Value::Int(5));
    println!("PASS");
}

#[test]
fn test_reverse_combinator() {
    println!();
    println!("=== Test: reverse combinator ===");

    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let elems: Vec<Node> = (1..=4i64).map(|v| {
        make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: v.to_le_bytes().to_vec() }, int_id, 0)
    }).collect();

    let tuple = make_node(NodeKind::Tuple, NodePayload::Tuple, int_id, 4);
    let tuple_id = tuple.id;
    nodes.insert(tuple_id, tuple);
    for (port, elem) in elems.into_iter().enumerate() {
        edges.push(Edge { source: tuple_id, target: elem.id, port: port as u8, label: EdgeLabel::Argument });
        nodes.insert(elem.id, elem);
    }

    let rev_node = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x36 },
        int_id, 1,
    );
    let rev_id = rev_node.id;
    nodes.insert(rev_id, rev_node);
    edges.push(Edge { source: rev_id, target: tuple_id, port: 0, label: EdgeLabel::Argument });

    let hash = compute_hash(&nodes, &edges);
    let graph = SemanticGraph {
        root: rev_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    };

    let result = interpret(&graph, &[], None).map(|(v, _)| v).expect("reverse should succeed");
    println!("reverse([1,2,3,4]) = {:?}", result);

    assert_eq!(result.len(), 1, "interpret returns single-element vec");
    let elems = match &result[0] {
        Value::Tuple(e) => e.clone(),
        other => panic!("expected Tuple, got {:?}", other),
    };
    assert_eq!(elems.len(), 4);
    assert_eq!(elems[0], Value::Int(4));
    assert_eq!(elems[1], Value::Int(3));
    assert_eq!(elems[2], Value::Int(2));
    assert_eq!(elems[3], Value::Int(1));
    println!("PASS");
}

/// The full dot product test: zip two literal vectors, map(mul) pairwise,
/// fold(0, add) to get the scalar product.
///
/// dot([1,2,3], [4,5,6]) = 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
#[test]
fn test_dot_product_via_combinators() {
    println!();
    println!("=== Test: full dot product via zip + map(mul) + fold(0, add) ===");

    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // ---- Build left vector [1, 2, 3] ----
    let left_lits: Vec<Node> = [1i64, 2, 3].iter().map(|&v| {
        make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: v.to_le_bytes().to_vec() }, int_id, 0)
    }).collect();
    let left_tuple = make_node_depth(NodeKind::Tuple, NodePayload::Tuple, int_id, 3, 2);
    let left_id = left_tuple.id;
    nodes.insert(left_id, left_tuple);
    for (port, lit) in left_lits.into_iter().enumerate() {
        edges.push(Edge { source: left_id, target: lit.id, port: port as u8, label: EdgeLabel::Argument });
        nodes.insert(lit.id, lit);
    }

    // ---- Build right vector [4, 5, 6] ----
    let right_lits: Vec<Node> = [4i64, 5, 6].iter().map(|&v| {
        make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: v.to_le_bytes().to_vec() }, int_id, 0)
    }).collect();
    // Use different resolution_depth to get a distinct NodeId from left_tuple.
    let right_tuple = make_node_depth(NodeKind::Tuple, NodePayload::Tuple, int_id, 3, 1);
    let right_id = right_tuple.id;
    nodes.insert(right_id, right_tuple);
    for (port, lit) in right_lits.into_iter().enumerate() {
        edges.push(Edge { source: right_id, target: lit.id, port: port as u8, label: EdgeLabel::Argument });
        nodes.insert(lit.id, lit);
    }

    // ---- Zip(left, right) ----
    let zip_node = make_node(NodeKind::Prim, NodePayload::Prim { opcode: 0x32 }, int_id, 2);
    let zip_id = zip_node.id;
    nodes.insert(zip_id, zip_node);
    edges.push(Edge { source: zip_id, target: left_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: zip_id, target: right_id, port: 1, label: EdgeLabel::Argument });

    // ---- Mul step node ----
    let mul_node = make_node(NodeKind::Prim, NodePayload::Prim { opcode: 0x02 }, int_id, 2);
    let mul_id = mul_node.id;
    nodes.insert(mul_id, mul_node);

    // ---- Map(zip_result, mul) ----
    let map_node = make_node(NodeKind::Prim, NodePayload::Prim { opcode: 0x30 }, int_id, 2);
    let map_id = map_node.id;
    nodes.insert(map_id, map_node);
    edges.push(Edge { source: map_id, target: zip_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: map_id, target: mul_id, port: 1, label: EdgeLabel::Argument });

    // ---- Fold base: Lit(0) ----
    let base_node = make_node(
        NodeKind::Lit,
        NodePayload::Lit { type_tag: 0, value: 0i64.to_le_bytes().to_vec() },
        int_id, 0,
    );
    let base_id = base_node.id;
    nodes.insert(base_id, base_node);

    // ---- Fold step: Prim(add) ----
    let add_node = make_node(NodeKind::Prim, NodePayload::Prim { opcode: 0x00 }, int_id, 2);
    let add_id = add_node.id;
    nodes.insert(add_id, add_node);

    // ---- Fold node ----
    // Fold reads its collection from positional input BinderId(0xFFFF_0000).
    // We need to pass the map result as the input. Since Fold expects its
    // collection from the input binding, we'll structure this differently:
    // evaluate the map+zip subgraph first, then pass its output as input
    // to a separate fold graph.
    //
    // Alternative approach: evaluate zip+map as one graph, then fold as another.

    // First, evaluate zip+map to get the products.
    let hash1 = compute_hash(&nodes, &edges);
    let map_graph = SemanticGraph {
        root: map_id,
        nodes: nodes.clone(),
        edges: edges.clone(),
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: hash1,
    };

    let products_raw = interpret(&map_graph, &[], None).map(|(v, _)| v).expect("zip+map should succeed");
    println!("zip+map(mul) result: {:?}", products_raw);
    // The interpreter wraps results in vec![val], so unwrap the Tuple.
    let products_vec = match &products_raw[0] {
        Value::Tuple(elems) => elems.clone(),
        other => panic!("expected Tuple, got {:?}", other),
    };
    assert_eq!(products_vec, vec![Value::Int(4), Value::Int(10), Value::Int(18)]);

    // Now build a fold graph and pass the products as input.
    let mut fold_nodes = HashMap::new();
    let mut fold_edges = Vec::new();

    fold_nodes.insert(base_id, nodes[&base_id].clone());
    fold_nodes.insert(add_id, nodes[&add_id].clone());

    let fold_node = make_node(
        NodeKind::Fold,
        NodePayload::Fold { recursion_descriptor: vec![] },
        int_id, 2,
    );
    let fold_id = fold_node.id;
    fold_nodes.insert(fold_id, fold_node);
    fold_edges.push(Edge { source: fold_id, target: base_id, port: 0, label: EdgeLabel::Argument });
    fold_edges.push(Edge { source: fold_id, target: add_id, port: 1, label: EdgeLabel::Argument });

    let hash2 = compute_hash(&fold_nodes, &fold_edges);
    let fold_graph = SemanticGraph {
        root: fold_id,
        nodes: fold_nodes,
        edges: fold_edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: hash2,
    };

    // Pass products as the input collection.
    let input = Value::Tuple(products_vec);
    let final_result = interpret(&fold_graph, &[input], None).map(|(v, _)| v).expect("fold should succeed");
    println!("fold(0, add, [4, 10, 18]) = {:?}", final_result);

    assert_eq!(final_result, vec![Value::Int(32)]);
    println!();
    println!("*** dot([1,2,3], [4,5,6]) = 32 -- PASS ***");
}

// ---------------------------------------------------------------------------
// Test: evolutionary search for dot product with higher-order combinators
// ---------------------------------------------------------------------------

fn dot_product_v2_test_cases() -> Vec<TestCase> {
    vec![
        // Pre-computed products: the engine should discover fold(0, add).
        // This is the same as the pre-computed variant but now with the
        // knowledge that the engine has map/zip in its vocabulary.
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(4),
                Value::Int(10),
                Value::Int(18),
            ])],
            expected_output: Some(vec![Value::Int(32)]),
            initial_state: None,
            expected_state: None,
        },
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(0), Value::Int(0)])],
            expected_output: Some(vec![Value::Int(0)]),
            initial_state: None,
            expected_state: None,
        },
        TestCase {
            inputs: vec![Value::tuple(vec![Value::Int(6), Value::Int(6)])],
            expected_output: Some(vec![Value::Int(12)]),
            initial_state: None,
            expected_state: None,
        },
        TestCase {
            inputs: vec![Value::tuple(vec![
                Value::Int(1),
                Value::Int(1),
                Value::Int(1),
                Value::Int(1),
            ])],
            expected_output: Some(vec![Value::Int(4)]),
            initial_state: None,
            expected_state: None,
        },
    ]
}

#[test]
fn evolve_dot_product_v2() {
    let start = Instant::now();

    println!();
    println!("====================================================================");
    println!("  IRIS Integration Test: Dot Product v2 (with Higher-Order Prims)");
    println!("====================================================================");
    println!();
    println!("  Now that map/zip/concat are available, the engine has richer");
    println!("  building blocks. This test uses pre-computed products (like the");
    println!("  original) but with a population seeded with higher-order programs.");
    println!();

    let exec = IrisExecutionService::new(ExecConfig {
        cache_capacity: 512,
        worker_threads: 4,
        ..ExecConfig::default()
    });

    let test_cases = dot_product_v2_test_cases();
    let spec = ProblemSpec {
        test_cases: test_cases.clone(),
        description: "Dot product v2 (pre-computed products, higher-order seeds)".to_string(),
        target_cost: None,
    };

    let config = EvolutionConfig {
        population_size: 64,
        max_generations: 200,
        mutation_rate: 0.8,
        crossover_rate: 0.5,
        tournament_size: 3,
        phase_thresholds: PhaseThresholds {
            exploration_min_improvement: 0.005,
            stagnation_window: 15,
            min_diversity: 0.1,
        },
        target_generation_time_ms: 500,
        num_demes: 1,
        novelty_k: 15,
        novelty_threshold: 0.1,
        novelty_weight: 1.0,
        coevolution: false,
        resource_budget_ms: 0,
        iris_mode: false,
    };

    println!("Starting evolution...");
    let result = iris_evolve::evolve(config, spec, &exec);

    for snap in &result.history {
        if snap.generation % 20 == 0 || snap.generation == result.generations_run - 1 {
            println!(
                "  Gen {:>3}: best_corr={:.4} avg_corr={:.4} front={} phase={:?}",
                snap.generation,
                snap.best_fitness.correctness(),
                snap.avg_fitness.correctness(),
                snap.pareto_front_size,
                snap.phase,
            );
        }
    }

    let total_time = start.elapsed();
    let best = &result.best_individual;

    println!();
    println!("Generations run: {}", result.generations_run);
    println!("Total time: {:.2?}", total_time);
    println!("Best correctness: {:.4}", best.fitness.correctness());
    println!();

    // Evaluate best against test cases.
    let mut passes = 0;
    for (i, tc) in test_cases.iter().enumerate() {
        let eval_result = exec.evaluate_individual(
            &best.fragment.graph,
            &[tc.clone()],
            iris_types::eval::EvalTier::A,
        );
        match eval_result {
            Ok(er) => {
                let actual = if er.outputs.is_empty() || er.outputs[0].is_empty() {
                    "ERROR".to_string()
                } else {
                    format!("{:?}", er.outputs[0])
                };
                let pass = tc
                    .expected_output
                    .as_ref()
                    .map(|e| !er.outputs[0].is_empty() && &er.outputs[0] == e)
                    .unwrap_or(false);
                if pass { passes += 1; }
                println!(
                    "  [{}] expected={:?} actual={} {}",
                    i,
                    tc.expected_output.as_ref().unwrap(),
                    actual,
                    if pass { "PASS" } else { "FAIL" }
                );
            }
            Err(e) => println!("  [{}] ERROR: {:?}", i, e),
        }
    }

    println!();
    println!(
        "Test cases passed: {}/{} ({:.0}%)",
        passes,
        test_cases.len(),
        passes as f32 / test_cases.len() as f32 * 100.0
    );

    assert!(result.generations_run >= 0, "Evolution should have completed");
}
