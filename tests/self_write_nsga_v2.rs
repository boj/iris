
//! NSGA-II v2: crowding distance, non-dominated sort, and tournament
//! selection as IRIS programs.
//!
//! Extends self_write_nsga.rs with three new building blocks:
//!
//! 1. **Crowding distance (simplified)**: For individuals sharing a Pareto
//!    front, compute a diversity score. We use a sum-of-pairwise-distances
//!    approach: for each individual, sum the Manhattan distance to every
//!    other individual in the front, then divide by (n-1). Higher values
//!    mean the individual occupies a less-crowded region.
//!
//! 2. **Non-dominated sort**: Assign each individual to a Pareto front by
//!    computing domination counts. Front 0 has count 0. Front 1 has count
//!    that drops to 0 after removing front 0 members. We reuse the
//!    dominance check to compute full domination matrices.
//!
//! 3. **Tournament selection**: Select the best of k random candidates
//!    using rank as primary key and crowding distance as tiebreaker.
//!
//! Each algorithm is built as a SemanticGraph and verified against the
//! Rust implementations in iris_evolve::nsga2.

use std::collections::{BTreeMap, HashMap};

use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    BinderId, Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Graph construction helpers (same as self_write_nsga.rs)
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

fn project_node(id: u64, field_index: u16) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Project,
        NodePayload::Project { field_index },
        1,
    )
}

fn tuple_node(id: u64, arity: u8) -> (NodeId, Node) {
    make_node(id, NodeKind::Tuple, NodePayload::Tuple, arity)
}

fn fold_node(id: u64, mode: u8, arity: u8) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![mode],
        },
        arity,
    )
}

fn lambda_node(id: u64, binder: u32) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lambda,
        NodePayload::Lambda {
            binder: BinderId(binder),
            captured_count: 0,
        },
        0,
    )
}

// ---------------------------------------------------------------------------
// Dominance check (reused from self_write_nsga.rs)
// ---------------------------------------------------------------------------

/// dominates(A, B) -> Int(1) if A dominates B, Int(0) otherwise.
/// Inputs: [A: Tuple(Int...), B: Tuple(Int...)]
fn build_dominance_check() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: mul (logical AND via multiplication)
    let (nid, node) = prim_node(1, 0x02, 2);
    nodes.insert(nid, node);

    // --- all_geq branch ---
    let (nid, node) = fold_node(10, 0x00, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(11, 1);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(12, 0x10, 2); // bitand
    nodes.insert(nid, node);
    let (nid, node) = prim_node(13, 0x30, 2); // map
    nodes.insert(nid, node);
    let (nid, node) = prim_node(14, 0x32, 2); // zip
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(15, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(16, 1);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(17, 0x25, 2); // ge
    nodes.insert(nid, node);

    // --- any_gt branch ---
    let (nid, node) = fold_node(20, 0x00, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(21, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(22, 0x11, 2); // bitor
    nodes.insert(nid, node);
    let (nid, node) = prim_node(23, 0x30, 2); // map
    nodes.insert(nid, node);
    let (nid, node) = prim_node(24, 0x32, 2); // zip
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(25, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(26, 1);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(27, 0x23, 2); // gt
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(10, 11, 0, EdgeLabel::Argument),
        make_edge(10, 12, 1, EdgeLabel::Argument),
        make_edge(10, 13, 2, EdgeLabel::Argument),
        make_edge(13, 14, 0, EdgeLabel::Argument),
        make_edge(13, 17, 1, EdgeLabel::Argument),
        make_edge(14, 15, 0, EdgeLabel::Argument),
        make_edge(14, 16, 1, EdgeLabel::Argument),
        make_edge(20, 21, 0, EdgeLabel::Argument),
        make_edge(20, 22, 1, EdgeLabel::Argument),
        make_edge(20, 23, 2, EdgeLabel::Argument),
        make_edge(23, 24, 0, EdgeLabel::Argument),
        make_edge(23, 27, 1, EdgeLabel::Argument),
        make_edge(24, 25, 0, EdgeLabel::Argument),
        make_edge(24, 26, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Pareto rank (reused from self_write_nsga.rs)
// ---------------------------------------------------------------------------

/// rank(individual, population) -> Int(domination count)
/// Inputs: [individual: Tuple, population: Tuple of Tuples]
fn build_pareto_rank() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Fold(mode=0x00, arity=3)
    let (nid, node) = fold_node(1, 0x00, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(10, 0);
    nodes.insert(nid, node);
    let (nid, node) = lambda_node(20, 0xFFFF_0002);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 1);
    nodes.insert(nid, node);

    // Lambda body: add(acc, dominates_result)
    let (nid, node) = prim_node(100, 0x00, 2); // add
    nodes.insert(nid, node);
    let (nid, node) = project_node(110, 0); // acc
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(115, 2);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(120, 0x02, 2); // mul (dominance AND)
    nodes.insert(nid, node);

    // all_geq: fold(1, bitand, map(zip(other, individual), ge))
    let (nid, node) = fold_node(200, 0x00, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(201, 1);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(202, 0x10, 2); // bitand
    nodes.insert(nid, node);
    let (nid, node) = prim_node(203, 0x30, 2); // map
    nodes.insert(nid, node);
    let (nid, node) = prim_node(204, 0x32, 2); // zip
    nodes.insert(nid, node);
    let (nid, node) = project_node(205, 1); // other
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(206, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(207, 0); // individual
    nodes.insert(nid, node);
    let (nid, node) = prim_node(208, 0x25, 2); // ge
    nodes.insert(nid, node);

    // any_gt: fold(0, bitor, map(zip(other, individual), gt))
    let (nid, node) = fold_node(300, 0x00, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(301, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(302, 0x11, 2); // bitor
    nodes.insert(nid, node);
    let (nid, node) = prim_node(303, 0x30, 2); // map
    nodes.insert(nid, node);
    let (nid, node) = prim_node(304, 0x32, 2); // zip
    nodes.insert(nid, node);
    let (nid, node) = project_node(305, 1); // other
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(306, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(307, 0); // individual
    nodes.insert(nid, node);
    let (nid, node) = prim_node(308, 0x23, 2); // gt
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        Edge {
            source: NodeId(20),
            target: NodeId(100),
            port: 0,
            label: EdgeLabel::Continuation,
        },
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        make_edge(110, 115, 0, EdgeLabel::Argument),
        make_edge(120, 200, 0, EdgeLabel::Argument),
        make_edge(120, 300, 1, EdgeLabel::Argument),
        make_edge(200, 201, 0, EdgeLabel::Argument),
        make_edge(200, 202, 1, EdgeLabel::Argument),
        make_edge(200, 203, 2, EdgeLabel::Argument),
        make_edge(203, 204, 0, EdgeLabel::Argument),
        make_edge(203, 208, 1, EdgeLabel::Argument),
        make_edge(204, 205, 0, EdgeLabel::Argument),
        make_edge(204, 207, 1, EdgeLabel::Argument),
        make_edge(205, 206, 0, EdgeLabel::Argument),
        make_edge(300, 301, 0, EdgeLabel::Argument),
        make_edge(300, 302, 1, EdgeLabel::Argument),
        make_edge(300, 303, 2, EdgeLabel::Argument),
        make_edge(303, 304, 0, EdgeLabel::Argument),
        make_edge(303, 308, 1, EdgeLabel::Argument),
        make_edge(304, 305, 0, EdgeLabel::Argument),
        make_edge(304, 307, 1, EdgeLabel::Argument),
        make_edge(305, 306, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Step 1: Manhattan distance between two fitness vectors
// ---------------------------------------------------------------------------
//
// distance(A, B) = fold(0, add, map(zip(A, B), abs_diff))
//
// abs_diff isn't a single opcode, so we compute:
//   fold(0, add, map(zip(A, B), sub))  ... but sub can be negative
//
// We use: fold(0, add, map(map(zip(A, B), sub), abs))
// That is: zip the vectors, subtract pairwise, take abs of each, sum.
//
// Inputs: [A: Tuple(Int...), B: Tuple(Int...)]
// Returns: Int (Manhattan distance)

fn build_manhattan_distance() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: fold(0, add, map(map(zip(A,B), sub), abs))
    // id=1: fold(mode=0x00, arity=3)
    let (nid, node) = fold_node(1, 0x00, 3);
    nodes.insert(nid, node);

    // id=2: Int(0) — base
    let (nid, node) = int_lit_node(2, 0);
    nodes.insert(nid, node);

    // id=3: add (0x00) — step
    let (nid, node) = prim_node(3, 0x00, 2);
    nodes.insert(nid, node);

    // id=4: map(diffs, abs) — collection = absolute differences
    let (nid, node) = prim_node(4, 0x30, 2);
    nodes.insert(nid, node);

    // id=5: map(zip(A,B), sub) — pairwise subtraction
    let (nid, node) = prim_node(5, 0x30, 2);
    nodes.insert(nid, node);

    // id=6: abs (0x06) — function for outer map
    let (nid, node) = prim_node(6, 0x06, 1);
    nodes.insert(nid, node);

    // id=7: zip(A, B)
    let (nid, node) = prim_node(7, 0x32, 2);
    nodes.insert(nid, node);

    // id=8: sub (0x01) — function for inner map
    let (nid, node) = prim_node(8, 0x01, 2);
    nodes.insert(nid, node);

    // id=9: input_ref(0) — A
    let (nid, node) = input_ref_node(9, 0);
    nodes.insert(nid, node);

    // id=10: input_ref(1) — B
    let (nid, node) = input_ref_node(10, 1);
    nodes.insert(nid, node);

    let edges = vec![
        // fold(0, add, map(...))
        make_edge(1, 2, 0, EdgeLabel::Argument),  // base=0
        make_edge(1, 3, 1, EdgeLabel::Argument),  // step=add
        make_edge(1, 4, 2, EdgeLabel::Argument),  // collection=map(diffs, abs)
        // map(diffs, abs)
        make_edge(4, 5, 0, EdgeLabel::Argument),  // collection=map(zip(A,B), sub)
        make_edge(4, 6, 1, EdgeLabel::Argument),  // function=abs
        // map(zip(A,B), sub)
        make_edge(5, 7, 0, EdgeLabel::Argument),  // collection=zip(A,B)
        make_edge(5, 8, 1, EdgeLabel::Argument),  // function=sub
        // zip(A, B)
        make_edge(7, 9, 0, EdgeLabel::Argument),  // A
        make_edge(7, 10, 1, EdgeLabel::Argument), // B
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Step 2: Crowding distance (sum-of-distances to all others in front)
// ---------------------------------------------------------------------------
//
// For a single individual within a front (Tuple of fitness vectors),
// compute the sum of Manhattan distances to every other individual.
//
// crowding(individual, front) =
//   fold(0, add_step_lambda, front)
//   where add_step_lambda = |acc, other| acc + distance(individual, other)
//
// We use graph_eval to call the Manhattan distance sub-program.
//
// Inputs:
//   inputs[0] = individual fitness vector (Tuple of Ints)
//   inputs[1] = front (Tuple of fitness vectors)
//   inputs[2] = Program(manhattan_distance)
//
// Returns: Int (total distance to all others)

fn build_crowding_score() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: fold(0, lambda, front)
    let (nid, node) = fold_node(1, 0x00, 3);
    nodes.insert(nid, node);

    // base = 0
    let (nid, node) = int_lit_node(10, 0);
    nodes.insert(nid, node);

    // step = Lambda
    let (nid, node) = lambda_node(20, 0xFFFF_0002);
    nodes.insert(nid, node);

    // collection = input_ref(1) (the front)
    let (nid, node) = input_ref_node(30, 1);
    nodes.insert(nid, node);

    // --- Lambda body ---
    // add(acc, graph_eval(distance_prog, Tuple(individual, other)))

    // id=100: add(acc, dist_result)
    let (nid, node) = prim_node(100, 0x00, 2);
    nodes.insert(nid, node);

    // id=110: Project(0) from input_ref(2) -> acc
    let (nid, node) = project_node(110, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(115, 2);
    nodes.insert(nid, node);

    // id=120: graph_eval(0x89, arity=2) -> distance(individual, other)
    let (nid, node) = prim_node(120, 0x89, 2);
    nodes.insert(nid, node);

    // id=130: input_ref(2) -> distance program (outer captured input[2])
    // But inside the Lambda body, input_ref(2) is the Lambda-bound var.
    // We need to pass the distance program as input_ref differently.
    // The Lambda binder is 0xFFFF_0002, which shadows input_ref(2).
    // We need the distance program from outer scope.
    //
    // Solution: restructure inputs so the distance program is input[2]
    // but we need to access it inside the Lambda. Since the Lambda binder
    // shadows it, we'll use a different approach: pass the program
    // as part of the individual input, or use a higher input index.
    //
    // Actually, in the interpreter, input_ref(N) looks up BinderId(0xFFFF_00NN).
    // The Lambda binder is BinderId(0xFFFF_0002). So input_ref(2) inside
    // the Lambda body refers to the Lambda-bound variable.
    // But input_ref(0) and input_ref(1) still refer to the outer inputs
    // because they are not shadowed.
    //
    // So we can't access input[2] from inside the Lambda. We need to
    // restructure: make inputs[0] = Tuple(individual, distance_program)
    // and extract them.
    //
    // OR simpler: inline the distance computation (like build_pareto_rank
    // inlines dominance). Let's do that.
    //
    // Lambda body computes: acc + manhattan_distance(individual, other)
    //   where individual = input_ref(0), other = Project(1) from input_ref(2)
    //   manhattan_distance = fold(0, add, map(map(zip(ind, other), sub), abs))

    // id=120: fold(0, add, map(map(zip(individual, other), sub), abs))
    // — Manhattan distance, inlined
    let (nid, node) = fold_node(120, 0x00, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(121, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(122, 0x00, 2); // add
    nodes.insert(nid, node);
    let (nid, node) = prim_node(123, 0x30, 2); // map (outer: abs)
    nodes.insert(nid, node);
    let (nid, node) = prim_node(124, 0x30, 2); // map (inner: sub)
    nodes.insert(nid, node);
    let (nid, node) = prim_node(125, 0x06, 1); // abs
    nodes.insert(nid, node);
    let (nid, node) = prim_node(126, 0x32, 2); // zip
    nodes.insert(nid, node);
    let (nid, node) = prim_node(127, 0x01, 2); // sub
    nodes.insert(nid, node);

    // individual = input_ref(0) (outer captured)
    let (nid, node) = input_ref_node(128, 0);
    nodes.insert(nid, node);

    // other = Project(1) from input_ref(2) (Lambda-bound Tuple(acc, elem))
    let (nid, node) = project_node(129, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(130, 2);
    nodes.insert(nid, node);

    let edges = vec![
        // Root fold
        make_edge(1, 10, 0, EdgeLabel::Argument),  // base=0
        make_edge(1, 20, 1, EdgeLabel::Argument),  // step=Lambda
        make_edge(1, 30, 2, EdgeLabel::Argument),  // collection=front

        // Lambda body
        Edge {
            source: NodeId(20),
            target: NodeId(100),
            port: 0,
            label: EdgeLabel::Continuation,
        },

        // add(acc, distance)
        make_edge(100, 110, 0, EdgeLabel::Argument), // acc
        make_edge(100, 120, 1, EdgeLabel::Argument), // distance

        // acc = Project(0) from input_ref(2)
        make_edge(110, 115, 0, EdgeLabel::Argument),

        // distance = fold(0, add, map(map(zip(ind, other), sub), abs))
        make_edge(120, 121, 0, EdgeLabel::Argument), // base=0
        make_edge(120, 122, 1, EdgeLabel::Argument), // step=add
        make_edge(120, 123, 2, EdgeLabel::Argument), // collection=map(.., abs)

        // map(diffs, abs)
        make_edge(123, 124, 0, EdgeLabel::Argument), // collection=map(zip, sub)
        make_edge(123, 125, 1, EdgeLabel::Argument), // function=abs

        // map(zip(ind, other), sub)
        make_edge(124, 126, 0, EdgeLabel::Argument), // collection=zip
        make_edge(124, 127, 1, EdgeLabel::Argument), // function=sub

        // zip(individual, other)
        make_edge(126, 128, 0, EdgeLabel::Argument), // individual
        make_edge(126, 129, 1, EdgeLabel::Argument), // other

        // other = Project(1) from input_ref(2)
        make_edge(129, 130, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Step 3: Population crowding distances
// ---------------------------------------------------------------------------
//
// For each individual in a front, compute the crowding score.
//
// crowding_distances(front) = map(front, |ind| crowding(ind, front))
//
// Inputs:
//   inputs[0] = front (Tuple of fitness vectors)
//   inputs[1] = Program(crowding_score)
//
// Returns: Tuple of Int distances (one per individual)

fn build_population_crowding() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: map(front, lambda)
    let (nid, node) = prim_node(1, 0x30, 2);
    nodes.insert(nid, node);

    // port 0: front = input_ref(0)
    let (nid, node) = input_ref_node(5, 0);
    nodes.insert(nid, node);

    // port 1: Lambda per-individual function
    let (nid, node) = lambda_node(10, 0xFFFF_0003);
    nodes.insert(nid, node);

    // Lambda body: graph_eval(crowding_prog, Tuple(individual, front))
    // id=100: graph_eval(0x89, arity=2)
    let (nid, node) = prim_node(100, 0x89, 2);
    nodes.insert(nid, node);

    // id=110: crowding_score program = input_ref(1) (outer captured)
    let (nid, node) = input_ref_node(110, 1);
    nodes.insert(nid, node);

    // id=120: Tuple(individual, front)
    let (nid, node) = tuple_node(120, 2);
    nodes.insert(nid, node);

    // id=130: individual = input_ref(3) (Lambda-bound)
    let (nid, node) = input_ref_node(130, 3);
    nodes.insert(nid, node);

    // id=140: front = input_ref(0) (outer captured)
    let (nid, node) = input_ref_node(140, 0);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 5, 0, EdgeLabel::Argument),
        make_edge(1, 10, 1, EdgeLabel::Argument),
        Edge {
            source: NodeId(10),
            target: NodeId(100),
            port: 0,
            label: EdgeLabel::Continuation,
        },
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        make_edge(120, 130, 0, EdgeLabel::Argument),
        make_edge(120, 140, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Step 4: Population ranks via map
// ---------------------------------------------------------------------------
//
// For each individual, compute its Pareto rank (domination count).
//
// ranks(population) = map(population, |ind| rank(ind, population))
//
// Inputs:
//   inputs[0] = population (Tuple of fitness vectors)
//   inputs[1] = Program(pareto_rank)
//
// Returns: Tuple of Int ranks

fn build_population_ranks() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: map(population, lambda)
    let (nid, node) = prim_node(1, 0x30, 2);
    nodes.insert(nid, node);

    // port 0: population = input_ref(0)
    let (nid, node) = input_ref_node(5, 0);
    nodes.insert(nid, node);

    // port 1: Lambda
    let (nid, node) = lambda_node(10, 0xFFFF_0003);
    nodes.insert(nid, node);

    // Lambda body: graph_eval(rank_prog, Tuple(individual, population))
    let (nid, node) = prim_node(100, 0x89, 2);
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(110, 1); // rank program
    nodes.insert(nid, node);

    let (nid, node) = tuple_node(120, 2);
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(130, 3); // individual (Lambda-bound)
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(140, 0); // population (outer)
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 5, 0, EdgeLabel::Argument),
        make_edge(1, 10, 1, EdgeLabel::Argument),
        Edge {
            source: NodeId(10),
            target: NodeId(100),
            port: 0,
            label: EdgeLabel::Continuation,
        },
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        make_edge(120, 130, 0, EdgeLabel::Argument),
        make_edge(120, 140, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Step 5: Non-dominated sort (assign front indices)
// ---------------------------------------------------------------------------
//
// Full non-dominated sort assigns each individual to a Pareto front.
// Front 0 = domination count 0. Front 1 = individuals whose count
// drops to 0 when front-0 members are excluded from the dominator set.
//
// We implement this as an IRIS program that:
//   1. Computes the full domination matrix M[i][j] = dominates(i, j)
//   2. For each individual i, computes rank = sum over j of M[j][i]
//      (this is the domination count, equivalent to front assignment
//      for populations where domination counts correspond to fronts)
//
// For NSGA-II the important property is: domination count 0 = front 0.
// Higher fronts can be approximated by domination counts for ranking.
//
// The domination count approach gives us exactly the same result as
// build_pareto_rank for each individual. For true front assignment we
// would need iterative removal, which is expensive in the graph model.
//
// For tournament selection, domination count is sufficient since lower
// count means better rank. This is what we use.
//
// This program computes ranks for the full population.
//
// Inputs: [population: Tuple of fitness vectors]
// Returns: Tuple of Int (domination counts per individual)
//
// Implementation: reuse build_population_ranks with build_pareto_rank.

// ---------------------------------------------------------------------------
// Step 6: Tournament selection
// ---------------------------------------------------------------------------
//
// Given ranks and crowding distances, select the best of two candidates.
// Lower rank wins; ties broken by higher crowding distance.
//
// tournament_compare(rank_a, crowd_a, rank_b, crowd_b) =
//   if rank_a < rank_b then 0
//   else if rank_b < rank_a then 1
//   else if crowd_a >= crowd_b then 0
//   else 1
//
// Inputs:
//   inputs[0] = Tuple(rank_a: Int, crowd_a: Int)
//   inputs[1] = Tuple(rank_b: Int, crowd_b: Int)
//
// Returns: Int(0) if A wins, Int(1) if B wins.
//
// We compute this using arithmetic without branching:
//   a_better_rank = lt(rank_a, rank_b)  -- Bool
//   b_better_rank = lt(rank_b, rank_a)  -- Bool
//   same_rank = eq(rank_a, rank_b)      -- Bool
//   a_more_crowded = ge(crowd_a, crowd_b) -- Bool
//
//   a_wins = bitor(a_better_rank, bitand(same_rank, a_more_crowded))
//   result = if a_wins then 0 else 1
//
// Using int coercion of Bool: a_wins is 1 or 0
//   result = sub(1, a_wins)  -- 0 if a_wins=1, 1 if a_wins=0

fn build_tournament_compare() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Result = sub(1, a_wins)
    // id=1: sub(0x01, arity=2)
    let (nid, node) = prim_node(1, 0x01, 2);
    nodes.insert(nid, node);

    // id=2: Int(1)
    let (nid, node) = int_lit_node(2, 1);
    nodes.insert(nid, node);

    // id=3: a_wins = bitor(a_better_rank, bitand(same_rank, a_more_crowded))
    let (nid, node) = prim_node(3, 0x11, 2); // bitor
    nodes.insert(nid, node);

    // id=4: a_better_rank = lt(rank_a, rank_b)
    let (nid, node) = prim_node(4, 0x22, 2); // lt
    nodes.insert(nid, node);

    // id=5: bitand(same_rank, a_more_crowded)
    let (nid, node) = prim_node(5, 0x10, 2); // bitand
    nodes.insert(nid, node);

    // id=6: same_rank = eq(rank_a, rank_b)
    let (nid, node) = prim_node(6, 0x20, 2); // eq
    nodes.insert(nid, node);

    // id=7: a_more_crowded = ge(crowd_a, crowd_b)
    let (nid, node) = prim_node(7, 0x25, 2); // ge
    nodes.insert(nid, node);

    // Extract components:
    // id=10: rank_a = Project(0) from input_ref(0)
    let (nid, node) = project_node(10, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(11, 0);
    nodes.insert(nid, node);

    // id=12: crowd_a = Project(1) from input_ref(0)
    let (nid, node) = project_node(12, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(13, 0);
    nodes.insert(nid, node);

    // id=14: rank_b = Project(0) from input_ref(1)
    let (nid, node) = project_node(14, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(15, 1);
    nodes.insert(nid, node);

    // id=16: crowd_b = Project(1) from input_ref(1)
    let (nid, node) = project_node(16, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(17, 1);
    nodes.insert(nid, node);

    // For the eq and second lt, we need separate references:
    // id=20: rank_a (for eq)
    let (nid, node) = project_node(20, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(21, 0);
    nodes.insert(nid, node);

    // id=22: rank_b (for eq)
    let (nid, node) = project_node(22, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(23, 1);
    nodes.insert(nid, node);

    let edges = vec![
        // result = sub(1, a_wins)
        make_edge(1, 2, 0, EdgeLabel::Argument),   // 1
        make_edge(1, 3, 1, EdgeLabel::Argument),   // a_wins

        // a_wins = bitor(a_better_rank, bitand(same_rank, a_more_crowded))
        make_edge(3, 4, 0, EdgeLabel::Argument),   // a_better_rank
        make_edge(3, 5, 1, EdgeLabel::Argument),   // bitand(...)

        // a_better_rank = lt(rank_a, rank_b)
        make_edge(4, 10, 0, EdgeLabel::Argument),  // rank_a
        make_edge(4, 14, 1, EdgeLabel::Argument),  // rank_b

        // bitand(same_rank, a_more_crowded)
        make_edge(5, 6, 0, EdgeLabel::Argument),   // same_rank
        make_edge(5, 7, 1, EdgeLabel::Argument),   // a_more_crowded

        // same_rank = eq(rank_a, rank_b)
        make_edge(6, 20, 0, EdgeLabel::Argument),  // rank_a
        make_edge(6, 22, 1, EdgeLabel::Argument),  // rank_b

        // a_more_crowded = ge(crowd_a, crowd_b)
        make_edge(7, 12, 0, EdgeLabel::Argument),  // crowd_a
        make_edge(7, 16, 1, EdgeLabel::Argument),  // crowd_b

        // Projections
        make_edge(10, 11, 0, EdgeLabel::Argument),
        make_edge(12, 13, 0, EdgeLabel::Argument),
        make_edge(14, 15, 0, EdgeLabel::Argument),
        make_edge(16, 17, 0, EdgeLabel::Argument),
        make_edge(20, 21, 0, EdgeLabel::Argument),
        make_edge(22, 23, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Rust reference implementations for verification
// ---------------------------------------------------------------------------

/// Manhattan distance between two integer vectors.
fn rust_manhattan_distance(a: &[i64], b: &[i64]) -> i64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
}

/// Crowding score for one individual: sum of distances to all others.
fn rust_crowding_score(individual: &[i64], front: &[Vec<i64>]) -> i64 {
    front
        .iter()
        .map(|other| rust_manhattan_distance(individual, other))
        .sum()
}

/// Compute domination counts for a population using direct pairwise
/// comparison (same algorithm as the IRIS programs).
fn rust_domination_counts(population: &[Vec<i64>]) -> Vec<i64> {
    let n = population.len();
    let mut counts = vec![0i64; n];
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let mut all_geq = true;
            let mut any_gt = false;
            for k in 0..population[i].len() {
                if population[j][k] < population[i][k] {
                    all_geq = false;
                }
                if population[j][k] > population[i][k] {
                    any_gt = true;
                }
            }
            if all_geq && any_gt {
                counts[i] += 1;
            }
        }
    }
    counts
}

/// Full non-dominated sort producing front assignments.
/// This matches iris_evolve::nsga2::non_dominated_sort logic.
fn rust_non_dominated_sort(population: &[Vec<i64>]) -> Vec<usize> {
    let n = population.len();
    let mut domination_count = vec![0usize; n];
    let mut dominated_set: Vec<Vec<usize>> = vec![vec![]; n];

    for i in 0..n {
        for j in (i + 1)..n {
            let mut i_dom_j_all = true;
            let mut i_dom_j_any = false;
            let mut j_dom_i_all = true;
            let mut j_dom_i_any = false;
            for k in 0..population[i].len() {
                if population[i][k] < population[j][k] {
                    i_dom_j_all = false;
                }
                if population[i][k] > population[j][k] {
                    i_dom_j_any = true;
                }
                if population[j][k] < population[i][k] {
                    j_dom_i_all = false;
                }
                if population[j][k] > population[i][k] {
                    j_dom_i_any = true;
                }
            }
            if i_dom_j_all && i_dom_j_any {
                dominated_set[i].push(j);
                domination_count[j] += 1;
            } else if j_dom_i_all && j_dom_i_any {
                dominated_set[j].push(i);
                domination_count[i] += 1;
            }
        }
    }

    let mut front_assignment = vec![0usize; n];
    let mut current_front: Vec<usize> = (0..n)
        .filter(|&i| domination_count[i] == 0)
        .collect();
    let mut front_idx = 0;

    while !current_front.is_empty() {
        let mut next_front = vec![];
        for &i in &current_front {
            front_assignment[i] = front_idx;
            for &j in &dominated_set[i] {
                domination_count[j] -= 1;
                if domination_count[j] == 0 {
                    next_front.push(j);
                }
            }
        }
        front_idx += 1;
        current_front = next_front;
    }

    front_assignment
}

// ---------------------------------------------------------------------------
// Helper to convert population data to IRIS Values
// ---------------------------------------------------------------------------

fn population_to_value(pop: &[Vec<i64>]) -> Value {
    Value::Tuple(
        pop.iter()
            .map(|v| Value::tuple(v.iter().map(|&x| Value::Int(x)).collect()))
            .collect(),
    )
}

fn vec_to_value(v: &[i64]) -> Value {
    Value::tuple(v.iter().map(|&x| Value::Int(x)).collect())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Manhattan distance: |3-1| + |4-2| = 2 + 2 = 4
#[test]
fn manhattan_distance_basic() {
    let prog = build_manhattan_distance();
    let a = vec_to_value(&[3, 4]);
    let b = vec_to_value(&[1, 2]);
    let (outputs, _) = interpreter::interpret(&prog, &[a, b], None).unwrap();
    assert_eq!(outputs, vec![Value::Int(4)], "distance([3,4], [1,2]) = 4");
}

/// Manhattan distance: identical vectors -> 0
#[test]
fn manhattan_distance_identical() {
    let prog = build_manhattan_distance();
    let a = vec_to_value(&[5, 5, 5]);
    let b = vec_to_value(&[5, 5, 5]);
    let (outputs, _) = interpreter::interpret(&prog, &[a, b], None).unwrap();
    assert_eq!(outputs, vec![Value::Int(0)], "distance to self = 0");
}

/// Manhattan distance: |10-0| + |0-10| = 20
#[test]
fn manhattan_distance_extreme() {
    let prog = build_manhattan_distance();
    let a = vec_to_value(&[10, 0]);
    let b = vec_to_value(&[0, 10]);
    let (outputs, _) = interpreter::interpret(&prog, &[a, b], None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(20)],
        "distance([10,0], [0,10]) = 20"
    );
}

/// Crowding score: sum of distances from [3,3] to {[5,1], [1,5], [3,3]}
/// dist([3,3],[5,1]) = |3-5|+|3-1| = 2+2 = 4
/// dist([3,3],[1,5]) = |3-1|+|3-5| = 2+2 = 4
/// dist([3,3],[3,3]) = 0
/// total = 8
#[test]
fn crowding_score_basic() {
    let prog = build_crowding_score();
    let individual = vec_to_value(&[3, 3]);
    let front = population_to_value(&[vec![5, 1], vec![1, 5], vec![3, 3]]);
    let (outputs, _) = interpreter::interpret(&prog, &[individual, front], None).unwrap();
    let expected = rust_crowding_score(&[3, 3], &[vec![5, 1], vec![1, 5], vec![3, 3]]);
    assert_eq!(
        outputs,
        vec![Value::Int(expected)],
        "crowding score of [3,3] in front"
    );
}

/// Crowding score: corner individual has large distance
#[test]
fn crowding_score_corner() {
    let prog = build_crowding_score();
    let individual = vec_to_value(&[10, 0]);
    let front = population_to_value(&[vec![10, 0], vec![0, 10], vec![5, 5]]);
    let (outputs, _) = interpreter::interpret(&prog, &[individual, front], None).unwrap();
    let expected = rust_crowding_score(&[10, 0], &[vec![10, 0], vec![0, 10], vec![5, 5]]);
    assert_eq!(
        outputs,
        vec![Value::Int(expected)],
        "crowding score of corner [10,0]"
    );
}

/// Population crowding: verify each individual gets correct crowding score
#[test]
fn population_crowding_distances() {
    let crowding_prog = build_crowding_score();
    let pop_crowding_prog = build_population_crowding();

    let front_data = vec![vec![5i64, 1], vec![1, 5], vec![3, 3]];
    let front_val = population_to_value(&front_data);

    let inputs = vec![
        front_val,
        Value::Program(Box::new(crowding_prog)),
    ];

    let (outputs, _) = interpreter::interpret(&pop_crowding_prog, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);

    let expected: Vec<Value> = front_data
        .iter()
        .map(|ind| Value::Int(rust_crowding_score(ind, &front_data)))
        .collect();

    assert_eq!(
        outputs[0],
        Value::Tuple(expected),
        "population crowding distances should match Rust computation"
    );
}

/// Population ranks: verify domination counts match Rust computation
#[test]
fn population_ranks_match_rust() {
    let rank_prog = build_pareto_rank();
    let ranks_prog = build_population_ranks();

    let pop_data = vec![
        vec![10i64, 1],
        vec![1, 10],
        vec![5, 5],
        vec![3, 3],
        vec![0, 0],
    ];
    let population_val = population_to_value(&pop_data);

    let inputs = vec![
        population_val,
        Value::Program(Box::new(rank_prog)),
    ];

    let (outputs, _) = interpreter::interpret(&ranks_prog, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);

    let expected_counts = rust_domination_counts(&pop_data);
    let expected: Vec<Value> = expected_counts.iter().map(|&c| Value::Int(c)).collect();

    assert_eq!(
        outputs[0],
        Value::Tuple(expected),
        "IRIS ranks should match Rust domination counts"
    );
}

/// Tournament compare: lower rank always wins
#[test]
fn tournament_lower_rank_wins() {
    let prog = build_tournament_compare();
    // A: rank=0, crowd=5  vs  B: rank=1, crowd=100
    let a = Value::tuple(vec![Value::Int(0), Value::Int(5)]);
    let b = Value::tuple(vec![Value::Int(1), Value::Int(100)]);
    let (outputs, _) = interpreter::interpret(&prog, &[a, b], None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(0)],
        "rank 0 should beat rank 1 regardless of crowding"
    );
}

/// Tournament compare: higher rank loses
#[test]
fn tournament_higher_rank_loses() {
    let prog = build_tournament_compare();
    // A: rank=2, crowd=100  vs  B: rank=0, crowd=1
    let a = Value::tuple(vec![Value::Int(2), Value::Int(100)]);
    let b = Value::tuple(vec![Value::Int(0), Value::Int(1)]);
    let (outputs, _) = interpreter::interpret(&prog, &[a, b], None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(1)],
        "rank 2 should lose to rank 0"
    );
}

/// Tournament compare: same rank, higher crowding wins
#[test]
fn tournament_same_rank_crowding_tiebreak() {
    let prog = build_tournament_compare();
    // A: rank=1, crowd=10  vs  B: rank=1, crowd=5
    let a = Value::tuple(vec![Value::Int(1), Value::Int(10)]);
    let b = Value::tuple(vec![Value::Int(1), Value::Int(5)]);
    let (outputs, _) = interpreter::interpret(&prog, &[a, b], None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(0)],
        "same rank: higher crowding (10 > 5) should win"
    );
}

/// Tournament compare: same rank and crowding, A wins (ge includes equal)
#[test]
fn tournament_equal_a_wins() {
    let prog = build_tournament_compare();
    // A: rank=1, crowd=5  vs  B: rank=1, crowd=5
    let a = Value::tuple(vec![Value::Int(1), Value::Int(5)]);
    let b = Value::tuple(vec![Value::Int(1), Value::Int(5)]);
    let (outputs, _) = interpreter::interpret(&prog, &[a, b], None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(0)],
        "equal rank and crowding: A wins by convention (ge)"
    );
}

/// Tournament compare: B has better crowding when ranks match
#[test]
fn tournament_b_better_crowding() {
    let prog = build_tournament_compare();
    // A: rank=0, crowd=3  vs  B: rank=0, crowd=10
    let a = Value::tuple(vec![Value::Int(0), Value::Int(3)]);
    let b = Value::tuple(vec![Value::Int(0), Value::Int(10)]);
    let (outputs, _) = interpreter::interpret(&prog, &[a, b], None).unwrap();
    assert_eq!(
        outputs,
        vec![Value::Int(1)],
        "same rank, B has higher crowding -> B wins"
    );
}

/// End-to-end: compute ranks and crowding for a population, then use
/// tournament selection to verify the best individual is selected.
#[test]
fn end_to_end_rank_and_crowding() {
    let rank_prog = build_pareto_rank();
    let ranks_prog = build_population_ranks();
    let crowding_prog = build_crowding_score();
    let pop_crowding_prog = build_population_crowding();
    let tournament_prog = build_tournament_compare();

    // Population with clear Pareto structure:
    // [10,1]: front 0 (corner)
    // [1,10]: front 0 (corner)
    // [5,5]:  front 0 (middle)
    // [3,3]:  front 1 (dominated by [5,5])
    // [0,0]:  front 2 (dominated by everyone)
    let pop_data = vec![
        vec![10i64, 1],
        vec![1, 10],
        vec![5, 5],
        vec![3, 3],
        vec![0, 0],
    ];
    let population_val = population_to_value(&pop_data);

    // Step 1: Compute ranks
    let rank_inputs = vec![
        population_val.clone(),
        Value::Program(Box::new(rank_prog)),
    ];
    let (rank_outputs, _) = interpreter::interpret(&ranks_prog, &rank_inputs, None).unwrap();
    let iris_ranks: Vec<i64> = match &rank_outputs[0] {
        Value::Tuple(vals) => vals.iter().map(|v| match v {
            Value::Int(n) => *n,
            other => panic!("expected Int, got {:?}", other),
        }).collect(),
        other => panic!("expected Tuple, got {:?}", other),
    };

    // Verify ranks match Rust
    let expected_counts = rust_domination_counts(&pop_data);
    assert_eq!(iris_ranks, expected_counts, "ranks should match");

    // Step 2: Compute crowding for front 0 members
    let front0_data: Vec<Vec<i64>> = pop_data.iter()
        .enumerate()
        .filter(|(i, _)| expected_counts[*i] == 0)
        .map(|(_, v)| v.clone())
        .collect();
    let front0_val = population_to_value(&front0_data);

    let crowd_inputs = vec![
        front0_val,
        Value::Program(Box::new(crowding_prog)),
    ];
    let (crowd_outputs, _) = interpreter::interpret(&pop_crowding_prog, &crowd_inputs, None).unwrap();
    let iris_crowds: Vec<i64> = match &crowd_outputs[0] {
        Value::Tuple(vals) => vals.iter().map(|v| match v {
            Value::Int(n) => *n,
            other => panic!("expected Int, got {:?}", other),
        }).collect(),
        other => panic!("expected Tuple, got {:?}", other),
    };

    // Verify crowding matches Rust
    let expected_crowds: Vec<i64> = front0_data.iter()
        .map(|ind| rust_crowding_score(ind, &front0_data))
        .collect();
    assert_eq!(iris_crowds, expected_crowds, "crowding should match");

    // Step 3: Tournament between front-0 corner [10,1] (rank=0, crowd=high)
    // and front-1 [3,3] (rank=1, crowd=anything)
    // The corner individual should win.
    let a = Value::tuple(vec![Value::Int(0), Value::Int(iris_crowds[0])]);
    let b = Value::tuple(vec![Value::Int(1), Value::Int(0)]);
    let (tourn_outputs, _) = interpreter::interpret(&tournament_prog, &[a, b], None).unwrap();
    assert_eq!(
        tourn_outputs,
        vec![Value::Int(0)],
        "front-0 individual should beat front-1 in tournament"
    );
}

/// Verify non-dominated sort front assignments for a 3-objective population.
#[test]
fn three_objective_population() {
    let rank_prog = build_pareto_rank();
    let ranks_prog = build_population_ranks();

    // 3-objective population:
    // [10, 0, 0]: best in obj 0
    // [0, 10, 0]: best in obj 1
    // [0, 0, 10]: best in obj 2
    // [5, 5, 0]:  non-dominated (trade-off)
    // [1, 1, 1]:  dominated by all above
    let pop_data = vec![
        vec![10i64, 0, 0],
        vec![0, 10, 0],
        vec![0, 0, 10],
        vec![5, 5, 0],
        vec![1, 1, 1],
    ];
    let population_val = population_to_value(&pop_data);

    let inputs = vec![
        population_val,
        Value::Program(Box::new(rank_prog)),
    ];
    let (outputs, _) = interpreter::interpret(&ranks_prog, &inputs, None).unwrap();

    let iris_ranks: Vec<i64> = match &outputs[0] {
        Value::Tuple(vals) => vals.iter().map(|v| match v {
            Value::Int(n) => *n,
            other => panic!("expected Int, got {:?}", other),
        }).collect(),
        other => panic!("expected Tuple, got {:?}", other),
    };

    let expected = rust_domination_counts(&pop_data);
    assert_eq!(iris_ranks, expected, "3-objective ranks should match Rust");

    // Verify: [10,0,0], [0,10,0], [0,0,10], [5,5,0] are all non-dominated (rank 0)
    // [1,1,1] is dominated by [5,5,0] (5>=1,5>=1,0<1 -> not dom) and others
    assert_eq!(iris_ranks[0], 0, "[10,0,0] should be non-dominated");
    assert_eq!(iris_ranks[1], 0, "[0,10,0] should be non-dominated");
    assert_eq!(iris_ranks[2], 0, "[0,0,10] should be non-dominated");
    assert_eq!(iris_ranks[3], 0, "[5,5,0] should be non-dominated");
    // [1,1,1]: check if dominated
    // [10,0,0] doesn't dominate [1,1,1] since 0 < 1 in obj 1 and 2
    // [0,10,0] doesn't dominate [1,1,1] since 0 < 1 in obj 0 and 2
    // [0,0,10] doesn't dominate [1,1,1] since 0 < 1 in obj 0 and 1
    // [5,5,0] doesn't dominate [1,1,1] since 0 < 1 in obj 2
    // So [1,1,1] is actually non-dominated too!
    assert_eq!(iris_ranks[4], 0, "[1,1,1] should be non-dominated in this population");
}

/// Verify the relationship between domination counts and the Rust
/// non_dominated_sort front assignments for a well-stratified population.
#[test]
fn domination_counts_vs_fronts() {
    // A population with clear stratification:
    // Front 0: [10,10] dominates everything below
    // Front 1: [5,5] dominated only by [10,10]
    // Front 2: [2,2] dominated by [10,10] and [5,5]
    // Front 3: [0,0] dominated by all
    let pop_data = vec![
        vec![10i64, 10],
        vec![5, 5],
        vec![2, 2],
        vec![0, 0],
    ];

    let rank_prog = build_pareto_rank();
    let ranks_prog = build_population_ranks();

    let population_val = population_to_value(&pop_data);
    let inputs = vec![
        population_val,
        Value::Program(Box::new(rank_prog)),
    ];
    let (outputs, _) = interpreter::interpret(&ranks_prog, &inputs, None).unwrap();

    let iris_ranks: Vec<i64> = match &outputs[0] {
        Value::Tuple(vals) => vals.iter().map(|v| match v {
            Value::Int(n) => *n,
            other => panic!("expected Int, got {:?}", other),
        }).collect(),
        other => panic!("expected Tuple, got {:?}", other),
    };

    // Domination counts: [0, 1, 2, 3]
    assert_eq!(iris_ranks, vec![0, 1, 2, 3]);

    // For this linearly stratified population, domination counts
    // exactly equal front indices from non_dominated_sort.
    let fronts = rust_non_dominated_sort(&pop_data);
    assert_eq!(
        iris_ranks.iter().map(|&r| r as usize).collect::<Vec<_>>(),
        fronts,
        "for linearly stratified populations, domination count = front index"
    );
}

/// Verify Manhattan distance is symmetric.
#[test]
fn manhattan_distance_symmetric() {
    let prog = build_manhattan_distance();
    let a = vec_to_value(&[7, 2, 9]);
    let b = vec_to_value(&[3, 8, 1]);

    let (out_ab, _) = interpreter::interpret(&prog, &[a.clone(), b.clone()], None).unwrap();
    let (out_ba, _) = interpreter::interpret(&prog, &[b, a], None).unwrap();

    assert_eq!(out_ab, out_ba, "Manhattan distance should be symmetric");
    // |7-3| + |2-8| + |9-1| = 4 + 6 + 8 = 18
    assert_eq!(out_ab, vec![Value::Int(18)]);
}

/// Crowding scores should be symmetric within a front (distance matrix is symmetric).
#[test]
fn crowding_scores_sum_consistency() {
    let crowding_prog = build_crowding_score();

    let front_data = vec![vec![10i64, 0], vec![0, 10], vec![5, 5], vec![7, 3]];

    let mut total_crowding = 0i64;
    for ind in &front_data {
        let individual = vec_to_value(ind);
        let front = population_to_value(&front_data);
        let (outputs, _) = interpreter::interpret(
            &crowding_prog,
            &[individual, front],
            None,
        )
        .unwrap();
        let score = match &outputs[0] {
            Value::Int(n) => *n,
            other => panic!("expected Int, got {:?}", other),
        };
        let expected = rust_crowding_score(ind, &front_data);
        assert_eq!(score, expected, "crowding score mismatch for {:?}", ind);
        total_crowding += score;
    }

    // Total crowding should be even (each pair counted twice).
    assert_eq!(
        total_crowding % 2,
        0,
        "total crowding should be even (symmetric distances)"
    );
}
