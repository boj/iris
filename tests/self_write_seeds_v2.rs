
//! Self-writing seed generation v2: IRIS programs that construct all 13 seed types.
//!
//! Extends self_write_seeds.rs (which covers fold and add/arithmetic generators)
//! with generators for the remaining 11 seed types:
//!
//!   3. identity         — Lambda(Lit(0)) with Binding edge
//!   4. map              — Map(placeholder, add)
//!   5. map_fold         — Fold(0, add, Map(placeholder, mul))
//!   6. filter_fold      — Fold(0, add, Filter(placeholder, gt))
//!   7. zip_map_fold     — Fold(0, add, Map(Zip(a, b), mul))
//!   8. comparison_fold  — Fold(MIN, max) or Fold(MAX, min)
//!   9. stateful_fold    — Fold(0, add) plain sum
//!  10. conditional_fold — Fold(0, add, Filter(placeholder, gt)) sum-of-filtered
//!  11. pairwise_fold    — Map(Zip(input, Drop(input,1)), sub) pairwise diffs
//!
//! Each generator uses the "embedded template" pattern:
//!   - self_graph (0x80) captures the generator's own graph (with template nodes)
//!   - graph_replace_subtree (0x88) re-roots onto the template root
//!   - graph_connect (0x86) chains to wire edges between template nodes
//!
//! Additionally two generators demonstrate multi-input targets:
//!  12. arithmetic (mul variant) — mul(input(0), input(1))
//!  13. sub variant              — sub(input(0), input(1))

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Graph construction helpers
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
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

fn int_lit_node(id: u64, value: i64) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: value.to_le_bytes().to_vec(),
        },
        0,
    )
}

fn input_ref_node(id: u64, index: u8) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0xFF,
            value: vec![index],
        },
        0,
    )
}

fn prim_node(id: u64, opcode: u8, arity: u8) -> (NodeId, Node) {
    make_node(id, NodeKind::Prim, NodePayload::Prim { opcode }, arity)
}

fn fold_node(id: u64, arity: u8) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![],
        },
        arity,
    )
}

fn lambda_node(id: u64) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lambda,
        NodePayload::Lambda {
            binder: iris_types::graph::BinderId(0),
            captured_count: 0,
        },
        1,
    )
}

fn tuple_node(id: u64, arity: u8) -> (NodeId, Node) {
    make_node(id, NodeKind::Tuple, NodePayload::Tuple, arity)
}

// ---------------------------------------------------------------------------
// Helper: build the replace_subtree core (self_graph -> replace -> re-root)
//
// This is the common 4-node core for all generators:
//   core_id:     graph_replace_subtree(0x88, arity=4)
//   core_id+1:   self_graph(0x80, arity=0)
//   core_id+2:   lit(builder_root_id)
//   core_id+3:   self_graph(0x80, arity=0)
//   core_id+4:   lit(template_root_id)
//
// Returns (core_node_id, edges_to_add)
// ---------------------------------------------------------------------------
fn build_replace_core(
    nodes: &mut HashMap<NodeId, Node>,
    core_id: u64,
    builder_root_id: i64,
    template_root_id: i64,
) -> (u64, Vec<Edge>) {
    let (nid, node) = prim_node(core_id, 0x88, 4);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(core_id + 1, 0x80, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(core_id + 2, builder_root_id);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(core_id + 3, 0x80, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(core_id + 4, template_root_id);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(core_id, core_id + 1, 0, EdgeLabel::Argument),
        make_edge(core_id, core_id + 2, 1, EdgeLabel::Argument),
        make_edge(core_id, core_id + 3, 2, EdgeLabel::Argument),
        make_edge(core_id, core_id + 4, 3, EdgeLabel::Argument),
    ];

    (core_id, edges)
}

// ---------------------------------------------------------------------------
// Helper: build a graph_connect layer
//
// connect_id:   graph_connect(0x86, arity=4)
// connect_id+1: lit(source_node_id)
// connect_id+2: lit(target_node_id)
// connect_id+3: lit(port)
//
// inner_id: the node providing port 0 (the program to connect into)
// ---------------------------------------------------------------------------
fn build_connect_layer(
    nodes: &mut HashMap<NodeId, Node>,
    connect_id: u64,
    inner_id: u64,
    source_node_id: i64,
    target_node_id: i64,
    port: i64,
) -> (u64, Vec<Edge>) {
    let (nid, node) = prim_node(connect_id, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(connect_id + 1, source_node_id);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(connect_id + 2, target_node_id);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(connect_id + 3, port);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(connect_id, inner_id, 0, EdgeLabel::Argument),
        make_edge(connect_id, connect_id + 1, 1, EdgeLabel::Argument),
        make_edge(connect_id, connect_id + 2, 2, EdgeLabel::Argument),
        make_edge(connect_id, connect_id + 3, 3, EdgeLabel::Argument),
    ];

    (connect_id, edges)
}

// ---------------------------------------------------------------------------
// Helper: run generator, extract Program
// ---------------------------------------------------------------------------
fn run_generator(graph: &SemanticGraph) -> SemanticGraph {
    let (outputs, _) = interpreter::interpret(graph, &[], None).unwrap();
    assert_eq!(outputs.len(), 1, "generator should return exactly one value");
    match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Helper: count reachable nodes from root
// ---------------------------------------------------------------------------
fn reachable_nodes(graph: &SemanticGraph) -> std::collections::BTreeSet<NodeId> {
    let mut reachable = std::collections::BTreeSet::new();
    let mut worklist = vec![graph.root];
    while let Some(nid) = worklist.pop() {
        if reachable.insert(nid) {
            for edge in &graph.edges {
                if edge.source == nid && !reachable.contains(&edge.target) {
                    worklist.push(edge.target);
                }
            }
        }
    }
    reachable
}

// ===========================================================================
// Generator 1: Identity — Lambda(Lit(0))
//
// Target program:
//   root=Lambda(1000, arity=1, binder=0, captured_count=0)
//     └─ port 0 (Binding): Lit(1010, type_tag=0, value=0)
//
// Builder:
//   Root(id=1): graph_connect(0x86) — lambda→lit, port 0, label Binding
//     ├─0: graph_replace_subtree core (ids 10-14)
//     ├─1: lit(1000)  — source
//     ├─2: lit(1010)  — target
//     └─3: lit(0)     — port
//
// Template nodes (no edges):
//   id=1000: Lambda(binder=0, captured_count=0, arity=1)
//   id=1010: Lit(0)
// ===========================================================================

fn build_identity_seed_generator() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut all_edges = Vec::new();

    // Builder root: graph_connect for the one edge
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);

    // Replace core at ids 10-14
    let (core_id, core_edges) = build_replace_core(&mut nodes, 10, 1, 1000);
    all_edges.extend(core_edges);

    // Connect args for root
    let (nid, node) = int_lit_node(20, 1000); // source: lambda
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(21, 1010); // target: lit(0)
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(22, 0);    // port 0
    nodes.insert(nid, node);

    all_edges.push(make_edge(1, core_id, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 20, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 21, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 22, 3, EdgeLabel::Argument));

    // Template nodes
    let (nid, node) = lambda_node(1000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(1010, 0);
    nodes.insert(nid, node);

    make_graph(nodes, all_edges, 1)
}

// ===========================================================================
// Generator 2: Mul(input(0), input(1))
//
// Target: Prim(mul=0x02, arity=2) → [input_ref(0), input_ref(1)]
//
// Same pattern as the add generator but with mul opcode.
// ===========================================================================

fn build_mul_seed_generator() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut all_edges = Vec::new();

    // Outer: graph_connect (mul→input_ref(1), port 1)
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);

    // Inner: graph_connect (mul→input_ref(0), port 0)
    let (nid, node) = prim_node(2, 0x86, 4);
    nodes.insert(nid, node);

    // Replace core at ids 10-14
    let (core_id, core_edges) = build_replace_core(&mut nodes, 10, 1, 3000);
    all_edges.extend(core_edges);

    // Inner connect args
    let (nid, node) = int_lit_node(30, 3000);  // source: mul
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(31, 3010);  // target: input_ref(0)
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(32, 0);     // port 0
    nodes.insert(nid, node);

    // Outer connect args
    let (nid, node) = int_lit_node(40, 3000);  // source: mul
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(41, 3020);  // target: input_ref(1)
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(42, 1);     // port 1
    nodes.insert(nid, node);

    // Outer edges
    all_edges.push(make_edge(1, 2, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 40, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 41, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 42, 3, EdgeLabel::Argument));

    // Inner edges
    all_edges.push(make_edge(2, core_id, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 30, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 31, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 32, 3, EdgeLabel::Argument));

    // Template nodes
    let (nid, node) = prim_node(3000, 0x02, 2); // mul
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(3010, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(3020, 1);
    nodes.insert(nid, node);

    make_graph(nodes, all_edges, 1)
}

// ===========================================================================
// Generator 3: Sub(input(0), input(1))
// ===========================================================================

fn build_sub_seed_generator() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut all_edges = Vec::new();

    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(2, 0x86, 4);
    nodes.insert(nid, node);

    let (core_id, core_edges) = build_replace_core(&mut nodes, 10, 1, 3100);
    all_edges.extend(core_edges);

    let (nid, node) = int_lit_node(30, 3100);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(31, 3110);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(32, 0);
    nodes.insert(nid, node);

    let (nid, node) = int_lit_node(40, 3100);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(41, 3120);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(42, 1);
    nodes.insert(nid, node);

    all_edges.push(make_edge(1, 2, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 40, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 41, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 42, 3, EdgeLabel::Argument));

    all_edges.push(make_edge(2, core_id, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 30, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 31, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 32, 3, EdgeLabel::Argument));

    let (nid, node) = prim_node(3100, 0x01, 2); // sub
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(3110, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(3120, 1);
    nodes.insert(nid, node);

    make_graph(nodes, all_edges, 1)
}

// ===========================================================================
// Generator 4: Map(placeholder_tuple, add)
//
// Target: Prim(map=0x30, arity=2)
//   ├─ port 0: Tuple(arity=0) — placeholder
//   └─ port 1: Prim(add=0x00, arity=2) — step function
//
// Template nodes: map(4000), tuple(4010), prim_add(4020)
// Edges: map→tuple port 0, map→add port 1
// ===========================================================================

fn build_map_seed_generator() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut all_edges = Vec::new();

    // connect 2: map→add, port 1
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);

    // connect 1: map→tuple, port 0
    let (nid, node) = prim_node(2, 0x86, 4);
    nodes.insert(nid, node);

    // Replace core
    let (core_id, core_edges) = build_replace_core(&mut nodes, 10, 1, 4000);
    all_edges.extend(core_edges);

    // Connect 1 args: map→tuple, port 0
    let (nid, node) = int_lit_node(30, 4000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(31, 4010);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(32, 0);
    nodes.insert(nid, node);

    // Connect 2 args: map→add, port 1
    let (nid, node) = int_lit_node(40, 4000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(41, 4020);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(42, 1);
    nodes.insert(nid, node);

    // Outer connect
    all_edges.push(make_edge(1, 2, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 40, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 41, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 42, 3, EdgeLabel::Argument));

    // Inner connect
    all_edges.push(make_edge(2, core_id, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 30, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 31, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 32, 3, EdgeLabel::Argument));

    // Template nodes
    let (nid, node) = prim_node(4000, 0x30, 2); // map
    nodes.insert(nid, node);
    let (nid, node) = tuple_node(4010, 0);       // placeholder tuple
    nodes.insert(nid, node);
    let (nid, node) = prim_node(4020, 0x00, 2);  // add step
    nodes.insert(nid, node);

    make_graph(nodes, all_edges, 1)
}

// ===========================================================================
// Generator 5: Comparison Fold — Fold(i64::MIN, max) for finding maximum
//
// Target: Fold(arity=2)
//   ├─ port 0: Lit(i64::MIN) — base case
//   └─ port 1: Prim(max=0x08) — step function
//
// Template: fold(5000), lit_min(5010), prim_max(5020)
// ===========================================================================

fn build_comparison_fold_seed_generator() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut all_edges = Vec::new();

    // connect 2: fold→max, port 1
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);

    // connect 1: fold→lit_min, port 0
    let (nid, node) = prim_node(2, 0x86, 4);
    nodes.insert(nid, node);

    // Replace core
    let (core_id, core_edges) = build_replace_core(&mut nodes, 10, 1, 5000);
    all_edges.extend(core_edges);

    // Connect 1: fold→lit_min, port 0
    let (nid, node) = int_lit_node(30, 5000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(31, 5010);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(32, 0);
    nodes.insert(nid, node);

    // Connect 2: fold→max, port 1
    let (nid, node) = int_lit_node(40, 5000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(41, 5020);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(42, 1);
    nodes.insert(nid, node);

    all_edges.push(make_edge(1, 2, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 40, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 41, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 42, 3, EdgeLabel::Argument));

    all_edges.push(make_edge(2, core_id, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 30, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 31, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 32, 3, EdgeLabel::Argument));

    // Template nodes
    let (nid, node) = fold_node(5000, 2);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(5010, i64::MIN); // extreme base for max
    nodes.insert(nid, node);
    let (nid, node) = prim_node(5020, 0x08, 2);     // max
    nodes.insert(nid, node);

    make_graph(nodes, all_edges, 1)
}

// ===========================================================================
// Generator 6: Stateful Fold — Fold(0, add) plain sum
//
// Target: Fold(arity=2)
//   ├─ port 0: Lit(0) — base
//   └─ port 1: Prim(add=0x00) — step
// ===========================================================================

fn build_stateful_fold_seed_generator() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut all_edges = Vec::new();

    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(2, 0x86, 4);
    nodes.insert(nid, node);

    let (core_id, core_edges) = build_replace_core(&mut nodes, 10, 1, 6000);
    all_edges.extend(core_edges);

    // fold→lit(0), port 0
    let (nid, node) = int_lit_node(30, 6000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(31, 6010);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(32, 0);
    nodes.insert(nid, node);

    // fold→add, port 1
    let (nid, node) = int_lit_node(40, 6000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(41, 6020);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(42, 1);
    nodes.insert(nid, node);

    all_edges.push(make_edge(1, 2, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 40, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 41, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 42, 3, EdgeLabel::Argument));

    all_edges.push(make_edge(2, core_id, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 30, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 31, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(2, 32, 3, EdgeLabel::Argument));

    // Template
    let (nid, node) = fold_node(6000, 2);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(6010, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(6020, 0x00, 2); // add
    nodes.insert(nid, node);

    make_graph(nodes, all_edges, 1)
}

// ===========================================================================
// Generator 7: Map+Fold — Fold(0, add, Map(placeholder, mul))
//
// Target: Fold(arity=3)
//   ├─ port 0: Lit(0)
//   ├─ port 1: Prim(add=0x00)
//   └─ port 2: Map(0x30, arity=2)
//                ├─ port 0: Tuple(placeholder)
//                └─ port 1: Prim(mul=0x02)
//
// Template: fold(7000), lit_0(7010), prim_add(7020),
//           map(7030), tuple(7040), prim_mul(7050)
// Edges: fold→lit port 0, fold→add port 1, fold→map port 2,
//        map→tuple port 0, map→mul port 1
// Total: 5 connect layers
// ===========================================================================

fn build_map_fold_seed_generator() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut all_edges = Vec::new();

    // We need 5 connect layers. Build from innermost out.
    // Layer 5 (outermost, root): fold→map, port 2
    // Layer 4: fold→add, port 1
    // Layer 3: fold→lit(0), port 0
    // Layer 2: map→mul, port 1
    // Layer 1 (innermost): map→tuple, port 0

    // Replace core at ids 100-104
    let (core_id, core_edges) = build_replace_core(&mut nodes, 100, 1, 7000);
    all_edges.extend(core_edges);

    // Layer 1: map→tuple, port 0
    let (l1_id, l1_edges) = build_connect_layer(&mut nodes, 200, core_id, 7030, 7040, 0);
    all_edges.extend(l1_edges);

    // Layer 2: map→mul, port 1
    let (l2_id, l2_edges) = build_connect_layer(&mut nodes, 210, l1_id, 7030, 7050, 1);
    all_edges.extend(l2_edges);

    // Layer 3: fold→lit(0), port 0
    let (l3_id, l3_edges) = build_connect_layer(&mut nodes, 220, l2_id, 7000, 7010, 0);
    all_edges.extend(l3_edges);

    // Layer 4: fold→add, port 1
    let (l4_id, l4_edges) = build_connect_layer(&mut nodes, 230, l3_id, 7000, 7020, 1);
    all_edges.extend(l4_edges);

    // Layer 5 (root=1): fold→map, port 2
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(50, 7000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(51, 7030);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(52, 2);
    nodes.insert(nid, node);

    all_edges.push(make_edge(1, l4_id, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 50, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 51, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 52, 3, EdgeLabel::Argument));

    // Template nodes
    let (nid, node) = fold_node(7000, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(7010, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(7020, 0x00, 2); // add
    nodes.insert(nid, node);
    let (nid, node) = prim_node(7030, 0x30, 2); // map
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(7040, 0);    // program input
    nodes.insert(nid, node);
    let (nid, node) = prim_node(7050, 0x02, 2);  // mul
    nodes.insert(nid, node);

    make_graph(nodes, all_edges, 1)
}

// ===========================================================================
// Generator 8: Filter+Fold — Fold(0, add, Filter(placeholder, gt))
//
// Target: Fold(arity=3)
//   ├─ port 0: Lit(0)
//   ├─ port 1: Prim(add=0x00)
//   └─ port 2: Filter(0x31, arity=2)
//                ├─ port 0: Tuple(placeholder)
//                └─ port 1: Prim(gt=0x23)
//
// Same structure as map+fold but with filter(0x31) and gt(0x23)
// ===========================================================================

fn build_filter_fold_seed_generator() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut all_edges = Vec::new();

    let (core_id, core_edges) = build_replace_core(&mut nodes, 100, 1, 8000);
    all_edges.extend(core_edges);

    // Layer 1: filter→tuple, port 0
    let (l1_id, l1_edges) = build_connect_layer(&mut nodes, 200, core_id, 8030, 8040, 0);
    all_edges.extend(l1_edges);

    // Layer 2: filter→gt, port 1
    let (l2_id, l2_edges) = build_connect_layer(&mut nodes, 210, l1_id, 8030, 8050, 1);
    all_edges.extend(l2_edges);

    // Layer 3: fold→lit(0), port 0
    let (l3_id, l3_edges) = build_connect_layer(&mut nodes, 220, l2_id, 8000, 8010, 0);
    all_edges.extend(l3_edges);

    // Layer 4: fold→add, port 1
    let (l4_id, l4_edges) = build_connect_layer(&mut nodes, 230, l3_id, 8000, 8020, 1);
    all_edges.extend(l4_edges);

    // Root: fold→filter, port 2
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(50, 8000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(51, 8030);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(52, 2);
    nodes.insert(nid, node);

    all_edges.push(make_edge(1, l4_id, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 50, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 51, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 52, 3, EdgeLabel::Argument));

    // Template nodes
    let (nid, node) = fold_node(8000, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(8010, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(8020, 0x00, 2); // add
    nodes.insert(nid, node);
    let (nid, node) = prim_node(8030, 0x31, 2); // filter
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(8040, 0);   // program input
    nodes.insert(nid, node);
    let (nid, node) = prim_node(8050, 0x23, 2); // gt
    nodes.insert(nid, node);

    make_graph(nodes, all_edges, 1)
}

// ===========================================================================
// Generator 9: Zip+Map+Fold — Fold(0, add, Map(Zip(tuple_a, tuple_b), mul))
//
// Target: Fold(arity=3)
//   ├─ port 0: Lit(0)
//   ├─ port 1: Prim(add)
//   └─ port 2: Map(0x30)
//                ├─ port 0: Zip(0x32)
//                │            ├─ port 0: Tuple_a(placeholder)
//                │            └─ port 1: Tuple_b(Lit(0))
//                └─ port 1: Prim(mul)
//
// Template: fold(9000), lit_0(9010), add(9020),
//           map(9030), zip(9040), tuple_a(9050), lit_0_b(9060), mul(9070)
// 7 edges = 7 connect layers
// ===========================================================================

fn build_zip_map_fold_seed_generator() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut all_edges = Vec::new();

    let (core_id, core_edges) = build_replace_core(&mut nodes, 100, 1, 9000);
    all_edges.extend(core_edges);

    // Layer 1: zip→tuple_a, port 0
    let (l1_id, l1_edges) = build_connect_layer(&mut nodes, 200, core_id, 9040, 9050, 0);
    all_edges.extend(l1_edges);

    // Layer 2: zip→lit_0_b, port 1
    let (l2_id, l2_edges) = build_connect_layer(&mut nodes, 210, l1_id, 9040, 9060, 1);
    all_edges.extend(l2_edges);

    // Layer 3: map→zip, port 0
    let (l3_id, l3_edges) = build_connect_layer(&mut nodes, 220, l2_id, 9030, 9040, 0);
    all_edges.extend(l3_edges);

    // Layer 4: map→mul, port 1
    let (l4_id, l4_edges) = build_connect_layer(&mut nodes, 230, l3_id, 9030, 9070, 1);
    all_edges.extend(l4_edges);

    // Layer 5: fold→lit_0, port 0
    let (l5_id, l5_edges) = build_connect_layer(&mut nodes, 240, l4_id, 9000, 9010, 0);
    all_edges.extend(l5_edges);

    // Layer 6: fold→add, port 1
    let (l6_id, l6_edges) = build_connect_layer(&mut nodes, 250, l5_id, 9000, 9020, 1);
    all_edges.extend(l6_edges);

    // Root: fold→map, port 2
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(50, 9000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(51, 9030);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(52, 2);
    nodes.insert(nid, node);

    all_edges.push(make_edge(1, l6_id, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 50, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 51, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 52, 3, EdgeLabel::Argument));

    // Template nodes
    let (nid, node) = fold_node(9000, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(9010, 0);       // base
    nodes.insert(nid, node);
    let (nid, node) = prim_node(9020, 0x00, 2);    // add
    nodes.insert(nid, node);
    let (nid, node) = prim_node(9030, 0x30, 2);    // map
    nodes.insert(nid, node);
    let (nid, node) = prim_node(9040, 0x32, 2);    // zip
    nodes.insert(nid, node);
    let (nid, node) = tuple_node(9050, 0);          // placeholder a
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(9060, 0);        // placeholder b (Lit 0)
    nodes.insert(nid, node);
    let (nid, node) = prim_node(9070, 0x02, 2);    // mul
    nodes.insert(nid, node);

    make_graph(nodes, all_edges, 1)
}

// ===========================================================================
// Generator 10: Conditional Fold — Fold(0, add, Filter(placeholder, gt))
//
// Same structure as filter_fold but represents the "conditional fold" variant
// (sum of positive elements). Uses gt(0x23) as the filter predicate.
//
// This is structurally identical to filter_fold_seed_generator but uses
// different template IDs (to prove each generator is independent).
// ===========================================================================

fn build_conditional_fold_seed_generator() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut all_edges = Vec::new();

    let (core_id, core_edges) = build_replace_core(&mut nodes, 100, 1, 10000);
    all_edges.extend(core_edges);

    // Layer 1: filter→tuple, port 0
    let (l1_id, l1_edges) = build_connect_layer(&mut nodes, 200, core_id, 10030, 10040, 0);
    all_edges.extend(l1_edges);

    // Layer 2: filter→gt, port 1
    let (l2_id, l2_edges) = build_connect_layer(&mut nodes, 210, l1_id, 10030, 10050, 1);
    all_edges.extend(l2_edges);

    // Layer 3: fold→lit(0), port 0
    let (l3_id, l3_edges) = build_connect_layer(&mut nodes, 220, l2_id, 10000, 10010, 0);
    all_edges.extend(l3_edges);

    // Layer 4: fold→add, port 1
    let (l4_id, l4_edges) = build_connect_layer(&mut nodes, 230, l3_id, 10000, 10020, 1);
    all_edges.extend(l4_edges);

    // Root: fold→filter, port 2
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(50, 10000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(51, 10030);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(52, 2);
    nodes.insert(nid, node);

    all_edges.push(make_edge(1, l4_id, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 50, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 51, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 52, 3, EdgeLabel::Argument));

    // Template nodes
    let (nid, node) = fold_node(10000, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(10010, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(10020, 0x00, 2); // add
    nodes.insert(nid, node);
    let (nid, node) = prim_node(10030, 0x31, 2); // filter
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10040, 0);  // program input
    nodes.insert(nid, node);
    let (nid, node) = prim_node(10050, 0x23, 2); // gt
    nodes.insert(nid, node);

    make_graph(nodes, all_edges, 1)
}

// ===========================================================================
// Generator 11: Pairwise — Map(Zip(input_ref(0), Drop(input_ref(0), 1)), sub)
//
// Target: Map(0x30, arity=2)
//   ├─ port 0: Zip(0x32, arity=2)
//   │            ├─ port 0: input_ref(0)
//   │            └─ port 1: Drop(0x34, arity=2)
//   │                         ├─ port 0: input_ref(0)
//   │                         └─ port 1: Lit(1)
//   └─ port 1: Prim(sub=0x01)
//
// Template: map(11000), zip(11010), input_ref_a(11020), drop(11030),
//           input_ref_b(11040), lit_1(11050), prim_sub(11060)
// 6 edges = 6 connect layers
// ===========================================================================

fn build_pairwise_seed_generator() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let mut all_edges = Vec::new();

    let (core_id, core_edges) = build_replace_core(&mut nodes, 100, 1, 11000);
    all_edges.extend(core_edges);

    // Layer 1: drop→input_ref_b, port 0
    let (l1_id, l1_edges) = build_connect_layer(&mut nodes, 200, core_id, 11030, 11040, 0);
    all_edges.extend(l1_edges);

    // Layer 2: drop→lit_1, port 1
    let (l2_id, l2_edges) = build_connect_layer(&mut nodes, 210, l1_id, 11030, 11050, 1);
    all_edges.extend(l2_edges);

    // Layer 3: zip→input_ref_a, port 0
    let (l3_id, l3_edges) = build_connect_layer(&mut nodes, 220, l2_id, 11010, 11020, 0);
    all_edges.extend(l3_edges);

    // Layer 4: zip→drop, port 1
    let (l4_id, l4_edges) = build_connect_layer(&mut nodes, 230, l3_id, 11010, 11030, 1);
    all_edges.extend(l4_edges);

    // Layer 5: map→zip, port 0
    let (l5_id, l5_edges) = build_connect_layer(&mut nodes, 240, l4_id, 11000, 11010, 0);
    all_edges.extend(l5_edges);

    // Root: map→sub, port 1
    let (nid, node) = prim_node(1, 0x86, 4);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(50, 11000);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(51, 11060);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(52, 1);
    nodes.insert(nid, node);

    all_edges.push(make_edge(1, l5_id, 0, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 50, 1, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 51, 2, EdgeLabel::Argument));
    all_edges.push(make_edge(1, 52, 3, EdgeLabel::Argument));

    // Template nodes
    let (nid, node) = prim_node(11000, 0x30, 2);  // map
    nodes.insert(nid, node);
    let (nid, node) = prim_node(11010, 0x32, 2);  // zip
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(11020, 0);   // input_ref(0) for zip port 0
    nodes.insert(nid, node);
    let (nid, node) = prim_node(11030, 0x34, 2);  // drop
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(11040, 0);   // input_ref(0) for drop port 0
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(11050, 1);      // lit(1) for drop count
    nodes.insert(nid, node);
    let (nid, node) = prim_node(11060, 0x01, 2);  // sub
    nodes.insert(nid, node);

    make_graph(nodes, all_edges, 1)
}

// ===========================================================================
// Tests
// ===========================================================================

// --- Identity Generator ---

#[test]
fn identity_seed_generator_produces_lambda_program() {
    let generator = build_identity_seed_generator();
    let program = run_generator(&generator);

    let root_node = program.nodes.get(&program.root).expect("root should exist");
    assert_eq!(root_node.kind, NodeKind::Lambda, "root should be Lambda");

    let reachable = reachable_nodes(&program);
    assert_eq!(reachable.len(), 2, "identity should have 2 reachable nodes (Lambda + Lit)");

    let edges: Vec<&Edge> = program.edges.iter().filter(|e| e.source == program.root).collect();
    assert_eq!(edges.len(), 1, "Lambda should have 1 edge");
    assert_eq!(edges[0].port, 0, "edge should be port 0");
}

// --- Mul Generator ---

#[test]
fn mul_seed_generator_produces_mul_program() {
    let generator = build_mul_seed_generator();
    let program = run_generator(&generator);

    let root_node = program.nodes.get(&program.root).expect("root should exist");
    assert_eq!(root_node.kind, NodeKind::Prim);
    assert_eq!(root_node.payload, NodePayload::Prim { opcode: 0x02 });

    let (result, _) =
        interpreter::interpret(&program, &[Value::Int(3), Value::Int(7)], None).unwrap();
    assert_eq!(result, vec![Value::Int(21)], "mul(3, 7) should be 21");
}

#[test]
fn mul_seed_generator_handles_zero() {
    let generator = build_mul_seed_generator();
    let program = run_generator(&generator);

    let (result, _) =
        interpreter::interpret(&program, &[Value::Int(0), Value::Int(42)], None).unwrap();
    assert_eq!(result, vec![Value::Int(0)], "mul(0, 42) should be 0");
}

// --- Sub Generator ---

#[test]
fn sub_seed_generator_produces_sub_program() {
    let generator = build_sub_seed_generator();
    let program = run_generator(&generator);

    let root_node = program.nodes.get(&program.root).expect("root should exist");
    assert_eq!(root_node.kind, NodeKind::Prim);
    assert_eq!(root_node.payload, NodePayload::Prim { opcode: 0x01 });

    let (result, _) =
        interpreter::interpret(&program, &[Value::Int(10), Value::Int(3)], None).unwrap();
    assert_eq!(result, vec![Value::Int(7)], "sub(10, 3) should be 7");
}

#[test]
fn sub_seed_generator_handles_negatives() {
    let generator = build_sub_seed_generator();
    let program = run_generator(&generator);

    let (result, _) =
        interpreter::interpret(&program, &[Value::Int(5), Value::Int(8)], None).unwrap();
    assert_eq!(result, vec![Value::Int(-3)], "sub(5, 8) should be -3");
}

// --- Map Generator ---

#[test]
fn map_seed_generator_produces_map_program() {
    let generator = build_map_seed_generator();
    let program = run_generator(&generator);

    let root_node = program.nodes.get(&program.root).expect("root should exist");
    assert_eq!(root_node.kind, NodeKind::Prim);
    assert_eq!(
        root_node.payload,
        NodePayload::Prim { opcode: 0x30 },
        "root should be map (0x30)"
    );

    let reachable = reachable_nodes(&program);
    assert_eq!(reachable.len(), 3, "map program should have 3 reachable nodes");

    let edges: Vec<&Edge> = program.edges.iter().filter(|e| e.source == program.root).collect();
    assert_eq!(edges.len(), 2, "map should have 2 edges");

    let mut ports: Vec<u8> = edges.iter().map(|e| e.port).collect();
    ports.sort();
    assert_eq!(ports, vec![0, 1], "map edges should use ports 0, 1");
}

// --- Comparison Fold Generator ---

#[test]
fn comparison_fold_generator_produces_max_fold() {
    let generator = build_comparison_fold_seed_generator();
    let program = run_generator(&generator);

    let root_node = program.nodes.get(&program.root).expect("root should exist");
    assert_eq!(root_node.kind, NodeKind::Fold, "root should be Fold");

    // fold(MIN, max, [3, 1, 7, 2]) = 7
    let input = Value::tuple(vec![Value::Int(3), Value::Int(1), Value::Int(7), Value::Int(2)]);
    let (result, _) = interpreter::interpret(&program, &[input], None).unwrap();
    assert_eq!(result, vec![Value::Int(7)], "fold(MIN, max, [3,1,7,2]) should be 7");
}

#[test]
fn comparison_fold_generator_handles_single_element() {
    let generator = build_comparison_fold_seed_generator();
    let program = run_generator(&generator);

    let input = Value::tuple(vec![Value::Int(42)]);
    let (result, _) = interpreter::interpret(&program, &[input], None).unwrap();
    assert_eq!(result, vec![Value::Int(42)], "fold(MIN, max, [42]) should be 42");
}

#[test]
fn comparison_fold_generator_handles_negatives() {
    let generator = build_comparison_fold_seed_generator();
    let program = run_generator(&generator);

    let input = Value::tuple(vec![Value::Int(-5), Value::Int(-1), Value::Int(-10)]);
    let (result, _) = interpreter::interpret(&program, &[input], None).unwrap();
    assert_eq!(result, vec![Value::Int(-1)], "fold(MIN, max, [-5,-1,-10]) should be -1");
}

// --- Stateful Fold Generator ---

#[test]
fn stateful_fold_generator_produces_sum() {
    let generator = build_stateful_fold_seed_generator();
    let program = run_generator(&generator);

    let root_node = program.nodes.get(&program.root).expect("root should exist");
    assert_eq!(root_node.kind, NodeKind::Fold, "root should be Fold");

    // fold(0, add, [1, 2, 3]) = 6
    let input = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let (result, _) = interpreter::interpret(&program, &[input], None).unwrap();
    assert_eq!(result, vec![Value::Int(6)], "fold(0, add, [1,2,3]) should be 6");
}

#[test]
fn stateful_fold_generator_handles_empty() {
    let generator = build_stateful_fold_seed_generator();
    let program = run_generator(&generator);

    let input = Value::tuple(vec![]);
    let (result, _) = interpreter::interpret(&program, &[input], None).unwrap();
    assert_eq!(result, vec![Value::Int(0)], "fold(0, add, []) should be 0");
}

// --- Map+Fold Generator ---

#[test]
fn map_fold_generator_produces_correct_structure() {
    let generator = build_map_fold_seed_generator();
    let program = run_generator(&generator);

    let root_node = program.nodes.get(&program.root).expect("root should exist");
    assert_eq!(root_node.kind, NodeKind::Fold, "root should be Fold");

    let reachable = reachable_nodes(&program);
    assert_eq!(
        reachable.len(), 6,
        "map+fold should have 6 reachable nodes (fold, lit, add, map, input_ref, mul)"
    );

    // Verify 3 edges from fold root.
    let fold_edges: Vec<&Edge> = program.edges.iter().filter(|e| e.source == program.root).collect();
    assert_eq!(fold_edges.len(), 3, "Fold should have 3 edges");
    let mut ports: Vec<u8> = fold_edges.iter().map(|e| e.port).collect();
    ports.sort();
    assert_eq!(ports, vec![0, 1, 2]);
}

#[test]
fn map_fold_generator_computes_sum_of_squares() {
    let generator = build_map_fold_seed_generator();
    let program = run_generator(&generator);

    // fold(0, add, map([2, 3, 4], mul))
    // map(mul) on single elements: mul(x, x) = x^2
    // map([2,3,4], mul) = [4, 9, 16]
    // fold(0, add, [4, 9, 16]) = 29
    let input = Value::tuple(vec![Value::Int(2), Value::Int(3), Value::Int(4)]);
    let (result, _) = interpreter::interpret(&program, &[input], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(29)],
        "fold(0, add, map([2,3,4], mul)) = sum of squares = 29"
    );
}

// --- Filter+Fold Generator ---

#[test]
fn filter_fold_generator_produces_correct_structure() {
    let generator = build_filter_fold_seed_generator();
    let program = run_generator(&generator);

    let root_node = program.nodes.get(&program.root).expect("root should exist");
    assert_eq!(root_node.kind, NodeKind::Fold, "root should be Fold");

    let reachable = reachable_nodes(&program);
    assert_eq!(
        reachable.len(), 6,
        "filter+fold should have 6 reachable nodes"
    );
}

#[test]
fn filter_fold_generator_sums_positives() {
    let generator = build_filter_fold_seed_generator();
    let program = run_generator(&generator);

    // fold(0, add, filter([-3, 5, -1, 7, 0], gt))
    // gt filters elements > 0: [5, 7]
    // fold(0, add, [5, 7]) = 12
    let input = Value::tuple(vec![
        Value::Int(-3), Value::Int(5), Value::Int(-1),
        Value::Int(7), Value::Int(0),
    ]);
    let (result, _) = interpreter::interpret(&program, &[input], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(12)],
        "fold(0, add, filter([-3,5,-1,7,0], gt)) should be 12"
    );
}

#[test]
fn filter_fold_generator_handles_all_negative() {
    let generator = build_filter_fold_seed_generator();
    let program = run_generator(&generator);

    let input = Value::tuple(vec![Value::Int(-1), Value::Int(-2), Value::Int(-3)]);
    let (result, _) = interpreter::interpret(&program, &[input], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(0)],
        "fold(0, add, filter([-1,-2,-3], gt)) should be 0 (nothing passes filter)"
    );
}

// --- Zip+Map+Fold Generator ---

#[test]
fn zip_map_fold_generator_produces_correct_structure() {
    let generator = build_zip_map_fold_seed_generator();
    let program = run_generator(&generator);

    let root_node = program.nodes.get(&program.root).expect("root should exist");
    assert_eq!(root_node.kind, NodeKind::Fold, "root should be Fold");

    let reachable = reachable_nodes(&program);
    assert_eq!(
        reachable.len(), 8,
        "zip+map+fold should have 8 reachable nodes"
    );
}

// --- Conditional Fold Generator ---

#[test]
fn conditional_fold_generator_sums_positives() {
    let generator = build_conditional_fold_seed_generator();
    let program = run_generator(&generator);

    let root_node = program.nodes.get(&program.root).expect("root should exist");
    assert_eq!(root_node.kind, NodeKind::Fold, "root should be Fold");

    // Same as filter_fold: sum of positives
    let input = Value::tuple(vec![Value::Int(10), Value::Int(-5), Value::Int(3)]);
    let (result, _) = interpreter::interpret(&program, &[input], None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(13)],
        "conditional fold should sum positives: 10 + 3 = 13"
    );
}

// --- Pairwise Generator ---

#[test]
fn pairwise_generator_produces_correct_structure() {
    let generator = build_pairwise_seed_generator();
    let program = run_generator(&generator);

    let root_node = program.nodes.get(&program.root).expect("root should exist");
    assert_eq!(root_node.kind, NodeKind::Prim, "root should be Prim (map)");
    assert_eq!(
        root_node.payload,
        NodePayload::Prim { opcode: 0x30 },
        "root should be map (0x30)"
    );

    let reachable = reachable_nodes(&program);
    assert_eq!(
        reachable.len(), 7,
        "pairwise should have 7 reachable nodes"
    );
}

// --- Cross-generator: all generators produce valid Programs ---

#[test]
fn all_generators_produce_programs() {
    let generators: Vec<(&str, SemanticGraph)> = vec![
        ("identity", build_identity_seed_generator()),
        ("mul", build_mul_seed_generator()),
        ("sub", build_sub_seed_generator()),
        ("map", build_map_seed_generator()),
        ("comparison_fold", build_comparison_fold_seed_generator()),
        ("stateful_fold", build_stateful_fold_seed_generator()),
        ("map_fold", build_map_fold_seed_generator()),
        ("filter_fold", build_filter_fold_seed_generator()),
        ("zip_map_fold", build_zip_map_fold_seed_generator()),
        ("conditional_fold", build_conditional_fold_seed_generator()),
        ("pairwise", build_pairwise_seed_generator()),
    ];

    for (name, generator) in &generators {
        let (outputs, _) = interpreter::interpret(generator, &[], None)
            .unwrap_or_else(|e| panic!("{} generator failed: {:?}", name, e));
        assert_eq!(outputs.len(), 1, "{} should return 1 value", name);
        match &outputs[0] {
            Value::Program(_) => {}
            other => panic!("{} should return Program, got {:?}", name, other),
        }
    }
}

// --- Structural: generated programs have correct edge counts ---

#[test]
fn generated_programs_have_correct_root_edge_counts() {
    let cases: Vec<(&str, SemanticGraph, usize)> = vec![
        ("identity", build_identity_seed_generator(), 1),
        ("mul", build_mul_seed_generator(), 2),
        ("sub", build_sub_seed_generator(), 2),
        ("map", build_map_seed_generator(), 2),
        ("comparison_fold", build_comparison_fold_seed_generator(), 2),
        ("stateful_fold", build_stateful_fold_seed_generator(), 2),
        ("map_fold", build_map_fold_seed_generator(), 3),
        ("filter_fold", build_filter_fold_seed_generator(), 3),
        ("zip_map_fold", build_zip_map_fold_seed_generator(), 3),
        ("conditional_fold", build_conditional_fold_seed_generator(), 3),
    ];

    for (name, generator, expected_edges) in cases {
        let program = run_generator(&generator);
        let root_edges: Vec<&Edge> = program
            .edges
            .iter()
            .filter(|e| e.source == program.root)
            .collect();
        assert_eq!(
            root_edges.len(),
            expected_edges,
            "{} root should have {} edges, got {}",
            name,
            expected_edges,
            root_edges.len()
        );
    }
}

// --- Run generated programs via graph_eval (0x89) ---

#[test]
fn generated_mul_program_runs_via_graph_eval() {
    let generator = build_mul_seed_generator();
    let (outputs, _) = interpreter::interpret(&generator, &[], None).unwrap();
    let mul_program = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    let mut nodes = HashMap::new();
    let (nid, node) = prim_node(1, 0x89, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];

    let eval_program = make_graph(nodes, edges, 1);

    let inputs = vec![
        Value::Program(Rc::new(mul_program)),
        Value::tuple(vec![Value::Int(6), Value::Int(7)]),
    ];
    let (result, _) = interpreter::interpret(&eval_program, &inputs, None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(42)],
        "graph_eval(mul_program, (6, 7)) should be 42"
    );
}

#[test]
fn generated_stateful_fold_runs_via_graph_eval() {
    let generator = build_stateful_fold_seed_generator();
    let (outputs, _) = interpreter::interpret(&generator, &[], None).unwrap();
    let fold_program = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        other => panic!("expected Program, got {:?}", other),
    };

    let mut nodes = HashMap::new();
    let (nid, node) = prim_node(1, 0x89, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];

    let eval_program = make_graph(nodes, edges, 1);

    let list = Value::tuple(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
    let wrapped = Value::tuple(vec![list]);
    let inputs = vec![Value::Program(Rc::new(fold_program)), wrapped];
    let (result, _) = interpreter::interpret(&eval_program, &inputs, None).unwrap();
    assert_eq!(
        result,
        vec![Value::Int(60)],
        "graph_eval(sum_fold, [10,20,30]) should be 60"
    );
}

// --- Summary test ---

#[test]
fn self_write_seeds_v2_summary() {
    eprintln!();
    eprintln!("=== Self-Write Seed Generators v2: 11 generators ===");
    eprintln!();
    eprintln!("  [1/11]  identity          -- Lambda(Lit(0))");
    eprintln!("  [2/11]  mul               -- Prim(mul, input(0), input(1))");
    eprintln!("  [3/11]  sub               -- Prim(sub, input(0), input(1))");
    eprintln!("  [4/11]  map               -- Map(placeholder, add)");
    eprintln!("  [5/11]  comparison_fold    -- Fold(MIN, max) for finding maximum");
    eprintln!("  [6/11]  stateful_fold      -- Fold(0, add) plain sum");
    eprintln!("  [7/11]  map_fold           -- Fold(0, add, Map(placeholder, mul))");
    eprintln!("  [8/11]  filter_fold        -- Fold(0, add, Filter(placeholder, gt))");
    eprintln!("  [9/11]  zip_map_fold       -- Fold(0, add, Map(Zip(a,b), mul))");
    eprintln!("  [10/11] conditional_fold   -- Fold(0, add, Filter(placeholder, gt))");
    eprintln!("  [11/11] pairwise           -- Map(Zip(input, Drop(input,1)), sub)");
    eprintln!();
    eprintln!("Combined with self_write_seeds.rs (fold + add), all 13 seed types covered:");
    eprintln!("  arithmetic(add/mul/sub), fold, identity, map, zip_fold(zip+map+fold),");
    eprintln!("  map_fold, filter_fold, comparison_fold, stateful_fold,");
    eprintln!("  conditional_fold, pairwise");
    eprintln!();
    eprintln!("Pattern: embedded template nodes + self_graph + replace_subtree + connect");
}
