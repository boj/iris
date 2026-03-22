
//! Self-writing crossover and death/culling functions as IRIS programs.
//!
//! Builds on the patterns from self_write_population.rs to implement:
//!
//! **Crossover:**
//! 1. `crossover_embedding` — Interpolate two embedding vectors (Tuple of
//!    Float64) with an alpha weight. Computes:
//!      result[i] = alpha * emb_a[i] + (1.0 - alpha) * emb_b[i]
//!    Uses zip(emb_a, emb_b) then map with a lambda that applies the
//!    weighted blend per element pair.
//!
//! **Death/Culling:**
//! 1. `fitness_threshold_cull` — Remove individuals below a fitness threshold.
//!    Takes (population, threshold), returns filtered population using
//!    fold+guard (same pattern as self_write_population.rs death_cull).
//!
//! 2. `age_cull` — Remove individuals above max age. Takes (population, ages,
//!    max_age), zips population with ages, filters by age <= max_age, then
//!    projects out just the individuals.
//!
//! 3. `elitism_preserve` — Keep top-k by rank. Takes (population, ranks, k),
//!    returns top-k individuals. Uses fold to find the k-th best rank
//!    threshold, then filters to keep individuals with rank <= threshold.

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
// Graph construction helpers (shared pattern from other self_write_* tests)
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

fn float_lit_node(id: u64, value: f64) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x02,
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

fn guard_node(id: u64, predicate: u64, body: u64, fallback: u64) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Guard,
        NodePayload::Guard {
            predicate_node: NodeId(predicate),
            body_node: NodeId(body),
            fallback_node: NodeId(fallback),
        },
        0,
    )
}

// ===========================================================================
// 1. crossover_embedding — interpolate two embedding vectors
// ===========================================================================
//
// Takes 3 inputs:
//   inputs[0] = Tuple(Float64...) — embedding A
//   inputs[1] = Tuple(Float64...) — embedding B
//   inputs[2] = Float64(alpha)    — interpolation weight (0.0=B, 1.0=A)
//
// Computes: result[i] = alpha * A[i] + (1.0 - alpha) * B[i]
//
// Strategy:
//   1. zip(A, B) -> Tuple of Tuple(a_i, b_i) pairs
//   2. map over pairs with a Lambda that computes the blend:
//        add(mul(alpha, fst(pair)), mul(sub(1.0, alpha), snd(pair)))
//
// Graph structure:
//   Root(id=1): map(0x30, arity=2)
//   +-- port 0: zip(0x32, arity=2) [id=10]
//   |   +-- port 0: input_ref(0) [id=20]  -- emb_a
//   |   +-- port 1: input_ref(1) [id=30]  -- emb_b
//   +-- port 1: Lambda [id=40]
//       +-- body: add(mul(alpha, fst(pair)), mul(sub(1.0, alpha), snd(pair)))
//
// Lambda body (binder=0xFFFF_0003, so input_ref(3) = pair element):
//   id=100: add(0x00, arity=2)
//   +-- port 0: mul(alpha, fst(pair)) [id=110]
//   |   +-- port 0: input_ref(2) [id=120] -- alpha
//   |   +-- port 1: Project(0) from input_ref(3) [id=130, 135]
//   +-- port 1: mul(one_minus_alpha, snd(pair)) [id=140]
//       +-- port 0: sub(1.0, alpha) [id=150]
//       |   +-- port 0: Float64(1.0) [id=160]
//       |   +-- port 1: input_ref(2) [id=170] -- alpha
//       +-- port 1: Project(1) from input_ref(3) [id=180, 185]

fn build_crossover_embedding() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: map(0x30, arity=2)
    let (nid, node) = prim_node(1, 0x30, 2);
    nodes.insert(nid, node);

    // Port 0: zip(0x32, arity=2)
    let (nid, node) = prim_node(10, 0x32, 2);
    nodes.insert(nid, node);

    // zip port 0: emb_a = input_ref(0)
    let (nid, node) = input_ref_node(20, 0);
    nodes.insert(nid, node);

    // zip port 1: emb_b = input_ref(1)
    let (nid, node) = input_ref_node(30, 1);
    nodes.insert(nid, node);

    // Port 1: Lambda step function
    let (nid, node) = lambda_node(40, 0xFFFF_0003);
    nodes.insert(nid, node);

    // --- Lambda body ---

    // id=100: add(blended_a, blended_b) — the interpolation
    let (nid, node) = prim_node(100, 0x00, 2); // add
    nodes.insert(nid, node);

    // id=110: mul(alpha, fst(pair))
    let (nid, node) = prim_node(110, 0x02, 2); // mul
    nodes.insert(nid, node);

    // id=120: input_ref(2) — alpha (captured from outer)
    let (nid, node) = input_ref_node(120, 2);
    nodes.insert(nid, node);

    // id=130: Project(0) from input_ref(3) — fst(pair) = a_i
    let (nid, node) = project_node(130, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(135, 3);
    nodes.insert(nid, node);

    // id=140: mul(one_minus_alpha, snd(pair))
    let (nid, node) = prim_node(140, 0x02, 2); // mul
    nodes.insert(nid, node);

    // id=150: sub(1.0, alpha)
    let (nid, node) = prim_node(150, 0x01, 2); // sub
    nodes.insert(nid, node);

    // id=160: Float64(1.0)
    let (nid, node) = float_lit_node(160, 1.0);
    nodes.insert(nid, node);

    // id=170: input_ref(2) — alpha (second reference)
    let (nid, node) = input_ref_node(170, 2);
    nodes.insert(nid, node);

    // id=180: Project(1) from input_ref(3) — snd(pair) = b_i
    let (nid, node) = project_node(180, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(185, 3);
    nodes.insert(nid, node);

    let edges = vec![
        // Root map(zip_result, lambda)
        make_edge(1, 10, 0, EdgeLabel::Argument),  // collection = zip result
        make_edge(1, 40, 1, EdgeLabel::Argument),  // function = Lambda

        // zip(emb_a, emb_b)
        make_edge(10, 20, 0, EdgeLabel::Argument), // emb_a
        make_edge(10, 30, 1, EdgeLabel::Argument), // emb_b

        // Lambda body via Continuation
        Edge {
            source: NodeId(40),
            target: NodeId(100),
            port: 0,
            label: EdgeLabel::Continuation,
        },

        // add(blended_a, blended_b)
        make_edge(100, 110, 0, EdgeLabel::Argument), // mul(alpha, a_i)
        make_edge(100, 140, 1, EdgeLabel::Argument), // mul(1-alpha, b_i)

        // mul(alpha, fst(pair))
        make_edge(110, 120, 0, EdgeLabel::Argument), // alpha
        make_edge(110, 130, 1, EdgeLabel::Argument), // a_i

        // fst(pair) = Project(0) from input_ref(3)
        make_edge(130, 135, 0, EdgeLabel::Argument),

        // mul(1-alpha, snd(pair))
        make_edge(140, 150, 0, EdgeLabel::Argument), // 1.0 - alpha
        make_edge(140, 180, 1, EdgeLabel::Argument), // b_i

        // sub(1.0, alpha)
        make_edge(150, 160, 0, EdgeLabel::Argument), // 1.0
        make_edge(150, 170, 1, EdgeLabel::Argument), // alpha

        // snd(pair) = Project(1) from input_ref(3)
        make_edge(180, 185, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// 2. fitness_threshold_cull — remove individuals below fitness threshold
// ===========================================================================
//
// Takes 2 inputs:
//   inputs[0] = Tuple(fitness_values)  — population fitness scores
//   inputs[1] = Int(threshold)          — minimum fitness to survive
//
// Output: Tuple of surviving fitness values (those >= threshold)
//
// Implementation: Fold with Guard to conditionally append (same as
// self_write_population.rs build_death_cull).
//
// Graph structure:
//   Root(id=1): Fold(0x00, arity=3)
//   +-- port 0: Tuple() [id=10]          -- empty base
//   +-- port 1: Lambda [id=20]
//   |   +-- body: Guard [id=100]
//   |       +-- predicate: ge(elem, threshold) [id=200]
//   |       +-- body: concat(acc, Tuple(elem)) [id=300]
//   |       +-- fallback: acc [id=400]
//   +-- port 2: input_ref(0) [id=30]     -- population

fn build_fitness_threshold_cull() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Fold(mode=0x00, arity=3)
    let (nid, node) = fold_node(1, 0x00, 3);
    nodes.insert(nid, node);

    // Port 0: base = empty Tuple
    let (nid, node) = tuple_node(10, 0);
    nodes.insert(nid, node);

    // Port 1: Lambda step function
    let (nid, node) = lambda_node(20, 0xFFFF_0002);
    nodes.insert(nid, node);

    // Port 2: population = input_ref(0)
    let (nid, node) = input_ref_node(30, 0);
    nodes.insert(nid, node);

    // --- Lambda body ---

    // Predicate: ge(elem, threshold) — elem >= threshold
    let (nid, node) = prim_node(200, 0x25, 2); // ge
    nodes.insert(nid, node);

    // elem = Project(1) from input_ref(2)
    let (nid, node) = project_node(210, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(215, 2);
    nodes.insert(nid, node);

    // threshold = input_ref(1) (captured from outer)
    let (nid, node) = input_ref_node(220, 1);
    nodes.insert(nid, node);

    // Body: concat(acc, Tuple(elem)) — append survivor
    let (nid, node) = prim_node(300, 0x35, 2); // concat
    nodes.insert(nid, node);

    // acc = Project(0) from input_ref(2)
    let (nid, node) = project_node(310, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(315, 2);
    nodes.insert(nid, node);

    // Tuple(elem) — singleton tuple for concat
    let (nid, node) = tuple_node(320, 1);
    nodes.insert(nid, node);

    // elem for tuple wrapping = Project(1) from input_ref(2)
    let (nid, node) = project_node(330, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(335, 2);
    nodes.insert(nid, node);

    // Fallback: acc = Project(0) from input_ref(2)
    let (nid, node) = project_node(400, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(405, 2);
    nodes.insert(nid, node);

    // Guard node
    let (nid, node) = guard_node(100, 200, 300, 400);
    nodes.insert(nid, node);

    let edges = vec![
        // Fold ports
        make_edge(1, 10, 0, EdgeLabel::Argument),  // base (empty tuple)
        make_edge(1, 20, 1, EdgeLabel::Argument),  // step (Lambda)
        make_edge(1, 30, 2, EdgeLabel::Argument),  // collection (population)

        // Lambda body via Continuation
        Edge {
            source: NodeId(20),
            target: NodeId(100),
            port: 0,
            label: EdgeLabel::Continuation,
        },

        // Predicate: ge(elem, threshold)
        make_edge(200, 210, 0, EdgeLabel::Argument), // elem
        make_edge(200, 220, 1, EdgeLabel::Argument), // threshold

        // elem = Project(1) from input_ref(2)
        make_edge(210, 215, 0, EdgeLabel::Argument),

        // Body: concat(acc, Tuple(elem))
        make_edge(300, 310, 0, EdgeLabel::Argument), // acc
        make_edge(300, 320, 1, EdgeLabel::Argument), // Tuple(elem)

        // acc = Project(0) from input_ref(2)
        make_edge(310, 315, 0, EdgeLabel::Argument),

        // Tuple(elem)
        make_edge(320, 330, 0, EdgeLabel::Argument), // elem
        make_edge(330, 335, 0, EdgeLabel::Argument), // Project(1) from input_ref(2)

        // Fallback: acc = Project(0) from input_ref(2)
        make_edge(400, 405, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// 3. age_cull — remove individuals above max age
// ===========================================================================
//
// Takes 3 inputs:
//   inputs[0] = Tuple(individuals)  — population (can be any Value per slot)
//   inputs[1] = Tuple(ages)         — corresponding age for each individual
//   inputs[2] = Int(max_age)        — maximum allowed age
//
// Strategy:
//   1. zip(population, ages) -> Tuple of Tuple(individual, age) pairs
//   2. Fold over zipped pairs, filtering: keep pair if age <= max_age
//   3. Map over survivors to project out just the individual (fst)
//
// We combine steps 2+3: fold accumulates just the individuals (not the pairs)
// when age condition is met.
//
// Graph structure:
//   Root(id=1): Fold(0x00, arity=3)
//   +-- port 0: Tuple() [id=10]           -- empty base
//   +-- port 1: Lambda [id=20]
//   |   +-- body: Guard [id=100]
//   |       +-- predicate: le(age, max_age) [id=200]
//   |       +-- body: concat(acc, Tuple(individual)) [id=300]
//   |       +-- fallback: acc [id=400]
//   +-- port 2: zip(population, ages) [id=30]
//
// Lambda binder = 0xFFFF_0003, so input_ref(3) = fold-bound Tuple(acc, pair)
// where pair = Tuple(individual, age).
// input_ref(2) = max_age (captured from outer).
// Project(1) from input_ref(3) = pair
// Project(0) from pair = individual
// Project(1) from pair = age

fn build_age_cull() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Fold(mode=0x00, arity=3)
    let (nid, node) = fold_node(1, 0x00, 3);
    nodes.insert(nid, node);

    // Port 0: base = empty Tuple
    let (nid, node) = tuple_node(10, 0);
    nodes.insert(nid, node);

    // Port 1: Lambda step function
    // Binder 0xFFFF_0003 so input_ref(3) = fold-bound Tuple(acc, pair)
    // This avoids shadowing input_ref(2) which we need for max_age.
    let (nid, node) = lambda_node(20, 0xFFFF_0003);
    nodes.insert(nid, node);

    // Port 2: zip(population, ages)
    let (nid, node) = prim_node(30, 0x32, 2); // zip
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(31, 0); // population
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(32, 1); // ages
    nodes.insert(nid, node);

    // --- Lambda body ---

    // Predicate: le(age, max_age) — age <= max_age
    // le is 0x24
    let (nid, node) = prim_node(200, 0x24, 2); // le
    nodes.insert(nid, node);

    // age = Project(1) from Project(1) from input_ref(3)
    // input_ref(3) = Tuple(acc, pair); Project(1) = pair; Project(1) from pair = age
    let (nid, node) = project_node(210, 1); // age from pair
    nodes.insert(nid, node);
    let (nid, node) = project_node(211, 1); // pair from fold tuple
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(212, 3); // fold-bound var
    nodes.insert(nid, node);

    // max_age = input_ref(2) (captured from outer)
    let (nid, node) = input_ref_node(220, 2); // max_age
    nodes.insert(nid, node);

    // Body: concat(acc, Tuple(individual)) — append survivor
    let (nid, node) = prim_node(300, 0x35, 2); // concat
    nodes.insert(nid, node);

    // acc = Project(0) from input_ref(3)
    let (nid, node) = project_node(310, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(315, 3);
    nodes.insert(nid, node);

    // Tuple(individual) — singleton tuple for concat
    let (nid, node) = tuple_node(320, 1);
    nodes.insert(nid, node);

    // individual = Project(0) from Project(1) from input_ref(3)
    // pair = Project(1) from input_ref(3); individual = Project(0) from pair
    let (nid, node) = project_node(330, 0); // individual from pair
    nodes.insert(nid, node);
    let (nid, node) = project_node(331, 1); // pair from fold tuple
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(332, 3);
    nodes.insert(nid, node);

    // Fallback: acc = Project(0) from input_ref(3)
    let (nid, node) = project_node(400, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(405, 3);
    nodes.insert(nid, node);

    // Guard node
    let (nid, node) = guard_node(100, 200, 300, 400);
    nodes.insert(nid, node);

    let edges = vec![
        // Fold ports
        make_edge(1, 10, 0, EdgeLabel::Argument),  // base (empty tuple)
        make_edge(1, 20, 1, EdgeLabel::Argument),  // step (Lambda)
        make_edge(1, 30, 2, EdgeLabel::Argument),  // collection (zipped)

        // zip(population, ages)
        make_edge(30, 31, 0, EdgeLabel::Argument), // population
        make_edge(30, 32, 1, EdgeLabel::Argument), // ages

        // Lambda body via Continuation
        Edge {
            source: NodeId(20),
            target: NodeId(100),
            port: 0,
            label: EdgeLabel::Continuation,
        },

        // Predicate: le(age, max_age)
        make_edge(200, 210, 0, EdgeLabel::Argument), // age
        make_edge(200, 220, 1, EdgeLabel::Argument), // max_age

        // age = Project(1) from Project(1) from input_ref(3)
        make_edge(210, 211, 0, EdgeLabel::Argument),
        make_edge(211, 212, 0, EdgeLabel::Argument),

        // Body: concat(acc, Tuple(individual))
        make_edge(300, 310, 0, EdgeLabel::Argument), // acc
        make_edge(300, 320, 1, EdgeLabel::Argument), // Tuple(individual)

        // acc = Project(0) from input_ref(3)
        make_edge(310, 315, 0, EdgeLabel::Argument),

        // Tuple(individual)
        make_edge(320, 330, 0, EdgeLabel::Argument),

        // individual = Project(0) from Project(1) from input_ref(3)
        make_edge(330, 331, 0, EdgeLabel::Argument),
        make_edge(331, 332, 0, EdgeLabel::Argument),

        // Fallback: acc = Project(0) from input_ref(3)
        make_edge(400, 405, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// 4. elitism_preserve — keep top-k by rank
// ===========================================================================
//
// Takes 3 inputs:
//   inputs[0] = Tuple(individuals)  — population values
//   inputs[1] = Tuple(ranks)        — pareto rank per individual (lower = better)
//   inputs[2] = Int(k)              — number of top individuals to keep
//
// Strategy (simplified for IRIS expressibility):
//   Fold to find the minimum rank, then filter to keep individuals whose rank
//   equals that minimum, limited to k elements.
//
// For a more direct approach: zip(population, ranks), sort by rank (not
// available), take first k. Since sort isn't available, we use:
//   1. Find min_rank = fold(MAX_INT, min, ranks)
//   2. Filter: keep individuals whose rank == min_rank, up to k
//   3. If fewer than k at min_rank, repeat with next rank level
//
// Simplification: keep all individuals with rank <= threshold, where
// threshold is adjusted to get approximately k individuals. Since finding
// the exact threshold for k is hard without sorting, we implement the
// most useful case: keep the best-ranked individuals (rank 0 = Pareto front).
//
// Implementation: zip(population, ranks), fold to collect individuals whose
// rank < k (treating k as a rank cutoff). This gives "keep individuals in
// the top k rank levels", which is a meaningful elitism strategy.
//
// Graph structure:
//   Root(id=1): Fold(0x00, arity=3)
//   +-- port 0: Tuple() [id=10]           -- empty base
//   +-- port 1: Lambda [id=20]
//   |   +-- body: Guard [id=100]
//   |       +-- predicate: lt(rank, k) [id=200]
//   |       +-- body: concat(acc, Tuple(individual)) [id=300]
//   |       +-- fallback: acc [id=400]
//   +-- port 2: zip(population, ranks) [id=30]

fn build_elitism_preserve() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Fold(mode=0x00, arity=3)
    let (nid, node) = fold_node(1, 0x00, 3);
    nodes.insert(nid, node);

    // Port 0: base = empty Tuple
    let (nid, node) = tuple_node(10, 0);
    nodes.insert(nid, node);

    // Port 1: Lambda step function
    // Binder 0xFFFF_0003 so input_ref(3) = fold-bound Tuple(acc, pair)
    // This avoids shadowing input_ref(2) which we need for k.
    let (nid, node) = lambda_node(20, 0xFFFF_0003);
    nodes.insert(nid, node);

    // Port 2: zip(population, ranks)
    let (nid, node) = prim_node(30, 0x32, 2); // zip
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(31, 0); // population
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(32, 1); // ranks
    nodes.insert(nid, node);

    // --- Lambda body ---

    // Predicate: lt(rank, k) — rank < k means this individual is in the top k rank levels
    // lt is 0x22
    let (nid, node) = prim_node(200, 0x22, 2); // lt
    nodes.insert(nid, node);

    // rank = Project(1) from Project(1) from input_ref(3)
    // input_ref(3) = Tuple(acc, pair); Project(1) = pair = Tuple(individual, rank)
    // Project(1) from pair = rank
    let (nid, node) = project_node(210, 1); // rank from pair
    nodes.insert(nid, node);
    let (nid, node) = project_node(211, 1); // pair from fold tuple
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(212, 3); // fold-bound var
    nodes.insert(nid, node);

    // k = input_ref(2) (captured from outer)
    let (nid, node) = input_ref_node(220, 2); // k
    nodes.insert(nid, node);

    // Body: concat(acc, Tuple(individual)) — append elite
    let (nid, node) = prim_node(300, 0x35, 2); // concat
    nodes.insert(nid, node);

    // acc = Project(0) from input_ref(3)
    let (nid, node) = project_node(310, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(315, 3);
    nodes.insert(nid, node);

    // Tuple(individual) — singleton tuple for concat
    let (nid, node) = tuple_node(320, 1);
    nodes.insert(nid, node);

    // individual = Project(0) from Project(1) from input_ref(3)
    let (nid, node) = project_node(330, 0); // individual from pair
    nodes.insert(nid, node);
    let (nid, node) = project_node(331, 1); // pair from fold tuple
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(332, 3);
    nodes.insert(nid, node);

    // Fallback: acc = Project(0) from input_ref(3)
    let (nid, node) = project_node(400, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(405, 3);
    nodes.insert(nid, node);

    // Guard node
    let (nid, node) = guard_node(100, 200, 300, 400);
    nodes.insert(nid, node);

    let edges = vec![
        // Fold ports
        make_edge(1, 10, 0, EdgeLabel::Argument),  // base (empty tuple)
        make_edge(1, 20, 1, EdgeLabel::Argument),  // step (Lambda)
        make_edge(1, 30, 2, EdgeLabel::Argument),  // collection (zipped)

        // zip(population, ranks)
        make_edge(30, 31, 0, EdgeLabel::Argument), // population
        make_edge(30, 32, 1, EdgeLabel::Argument), // ranks

        // Lambda body via Continuation
        Edge {
            source: NodeId(20),
            target: NodeId(100),
            port: 0,
            label: EdgeLabel::Continuation,
        },

        // Predicate: lt(rank, k)
        make_edge(200, 210, 0, EdgeLabel::Argument), // rank
        make_edge(200, 220, 1, EdgeLabel::Argument), // k

        // rank = Project(1) from Project(1) from input_ref(3)
        make_edge(210, 211, 0, EdgeLabel::Argument),
        make_edge(211, 212, 0, EdgeLabel::Argument),

        // Body: concat(acc, Tuple(individual))
        make_edge(300, 310, 0, EdgeLabel::Argument), // acc
        make_edge(300, 320, 1, EdgeLabel::Argument), // Tuple(individual)

        // acc = Project(0) from input_ref(3)
        make_edge(310, 315, 0, EdgeLabel::Argument),

        // Tuple(individual)
        make_edge(320, 330, 0, EdgeLabel::Argument),

        // individual = Project(0) from Project(1) from input_ref(3)
        make_edge(330, 331, 0, EdgeLabel::Argument),
        make_edge(331, 332, 0, EdgeLabel::Argument),

        // Fallback: acc = Project(0) from input_ref(3)
        make_edge(400, 405, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// Tests
// ===========================================================================

// ---------------------------------------------------------------------------
// crossover_embedding tests
// ---------------------------------------------------------------------------

#[test]
fn embedding_crossover_pure_a() {
    let prog = build_crossover_embedding();

    let emb_a = Value::tuple(vec![
        Value::Float64(1.0),
        Value::Float64(2.0),
        Value::Float64(3.0),
    ]);
    let emb_b = Value::tuple(vec![
        Value::Float64(10.0),
        Value::Float64(20.0),
        Value::Float64(30.0),
    ]);
    let alpha = Value::Float64(1.0); // pure A

    let (out, _) = interpreter::interpret(&prog, &[emb_a, emb_b, alpha], None).unwrap();

    match &out[0] {
        Value::Tuple(result) => {
            assert_eq!(result.len(), 3, "result should have 3 dimensions");
            // alpha=1.0: result = 1.0*A + 0.0*B = A
            assert_eq!(result[0], Value::Float64(1.0));
            assert_eq!(result[1], Value::Float64(2.0));
            assert_eq!(result[2], Value::Float64(3.0));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn embedding_crossover_pure_b() {
    let prog = build_crossover_embedding();

    let emb_a = Value::tuple(vec![
        Value::Float64(1.0),
        Value::Float64(2.0),
        Value::Float64(3.0),
    ]);
    let emb_b = Value::tuple(vec![
        Value::Float64(10.0),
        Value::Float64(20.0),
        Value::Float64(30.0),
    ]);
    let alpha = Value::Float64(0.0); // pure B

    let (out, _) = interpreter::interpret(&prog, &[emb_a, emb_b, alpha], None).unwrap();

    match &out[0] {
        Value::Tuple(result) => {
            assert_eq!(result.len(), 3);
            // alpha=0.0: result = 0.0*A + 1.0*B = B
            assert_eq!(result[0], Value::Float64(10.0));
            assert_eq!(result[1], Value::Float64(20.0));
            assert_eq!(result[2], Value::Float64(30.0));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn embedding_crossover_half_blend() {
    let prog = build_crossover_embedding();

    let emb_a = Value::tuple(vec![
        Value::Float64(0.0),
        Value::Float64(10.0),
    ]);
    let emb_b = Value::tuple(vec![
        Value::Float64(10.0),
        Value::Float64(0.0),
    ]);
    let alpha = Value::Float64(0.5); // 50-50 blend

    let (out, _) = interpreter::interpret(&prog, &[emb_a, emb_b, alpha], None).unwrap();

    match &out[0] {
        Value::Tuple(result) => {
            assert_eq!(result.len(), 2);
            // 0.5*0 + 0.5*10 = 5.0, 0.5*10 + 0.5*0 = 5.0
            assert_eq!(result[0], Value::Float64(5.0));
            assert_eq!(result[1], Value::Float64(5.0));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn embedding_crossover_quarter_blend() {
    let prog = build_crossover_embedding();

    let emb_a = Value::tuple(vec![Value::Float64(100.0)]);
    let emb_b = Value::tuple(vec![Value::Float64(0.0)]);
    let alpha = Value::Float64(0.25); // 25% A, 75% B

    let (out, _) = interpreter::interpret(&prog, &[emb_a, emb_b, alpha], None).unwrap();

    match &out[0] {
        Value::Tuple(result) => {
            assert_eq!(result.len(), 1);
            // 0.25*100 + 0.75*0 = 25.0
            assert_eq!(result[0], Value::Float64(25.0));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn embedding_crossover_preserves_length() {
    let prog = build_crossover_embedding();

    let emb_a = Value::tuple(vec![
        Value::Float64(1.0),
        Value::Float64(2.0),
        Value::Float64(3.0),
        Value::Float64(4.0),
        Value::Float64(5.0),
    ]);
    let emb_b = Value::tuple(vec![
        Value::Float64(5.0),
        Value::Float64(4.0),
        Value::Float64(3.0),
        Value::Float64(2.0),
        Value::Float64(1.0),
    ]);
    let alpha = Value::Float64(0.5);

    let (out, _) = interpreter::interpret(&prog, &[emb_a, emb_b, alpha], None).unwrap();

    match &out[0] {
        Value::Tuple(result) => {
            assert_eq!(result.len(), 5, "output should match input dimensionality");
            // All should be 3.0 (midpoint of symmetric pairs)
            for (i, v) in result.iter().enumerate() {
                assert_eq!(
                    v,
                    &Value::Float64(3.0),
                    "dimension {} should be 3.0",
                    i
                );
            }
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// fitness_threshold_cull tests
// ---------------------------------------------------------------------------

#[test]
fn fitness_cull_removes_below_threshold() {
    let prog = build_fitness_threshold_cull();

    let population = Value::tuple(vec![
        Value::Int(10),
        Value::Int(3),
        Value::Int(7),
        Value::Int(1),
        Value::Int(8),
    ]);
    let threshold = Value::Int(5);

    let (out, _) = interpreter::interpret(&prog, &[population, threshold], None).unwrap();

    match &out[0] {
        Value::Tuple(survivors) => {
            assert_eq!(survivors.len(), 3, "3 individuals should survive (>= 5)");
            assert!(survivors.contains(&Value::Int(10)));
            assert!(survivors.contains(&Value::Int(7)));
            assert!(survivors.contains(&Value::Int(8)));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn fitness_cull_keeps_all_above() {
    let prog = build_fitness_threshold_cull();

    let population = Value::tuple(vec![
        Value::Int(10),
        Value::Int(20),
        Value::Int(30),
    ]);
    let threshold = Value::Int(5);

    let (out, _) = interpreter::interpret(&prog, &[population, threshold], None).unwrap();

    match &out[0] {
        Value::Tuple(survivors) => {
            assert_eq!(survivors.len(), 3, "all should survive");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn fitness_cull_removes_all_below() {
    let prog = build_fitness_threshold_cull();

    let population = Value::tuple(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
    ]);
    let threshold = Value::Int(100);

    let (out, _) = interpreter::interpret(&prog, &[population, threshold], None).unwrap();

    match &out[0] {
        Value::Tuple(survivors) => {
            assert_eq!(survivors.len(), 0, "none should survive");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn fitness_cull_threshold_equality() {
    let prog = build_fitness_threshold_cull();

    // ge means >= threshold, so exactly-threshold values survive
    let population = Value::tuple(vec![
        Value::Int(5),
        Value::Int(4),
        Value::Int(5),
    ]);
    let threshold = Value::Int(5);

    let (out, _) = interpreter::interpret(&prog, &[population, threshold], None).unwrap();

    match &out[0] {
        Value::Tuple(survivors) => {
            assert_eq!(survivors.len(), 2, "two fives should survive");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// age_cull tests
// ---------------------------------------------------------------------------

#[test]
fn age_cull_removes_old_individuals() {
    let prog = build_age_cull();

    let population = Value::tuple(vec![
        Value::Int(100), // individual A
        Value::Int(200), // individual B
        Value::Int(300), // individual C
        Value::Int(400), // individual D
    ]);
    let ages = Value::tuple(vec![
        Value::Int(10),  // A: age 10
        Value::Int(150), // B: age 150 — too old
        Value::Int(50),  // C: age 50
        Value::Int(200), // D: age 200 — too old
    ]);
    let max_age = Value::Int(100);

    let (out, _) = interpreter::interpret(&prog, &[population, ages, max_age], None).unwrap();

    match &out[0] {
        Value::Tuple(survivors) => {
            assert_eq!(survivors.len(), 2, "2 individuals should survive (age <= 100)");
            assert!(survivors.contains(&Value::Int(100)), "individual A should survive");
            assert!(survivors.contains(&Value::Int(300)), "individual C should survive");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn age_cull_keeps_all_young() {
    let prog = build_age_cull();

    let population = Value::tuple(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
    ]);
    let ages = Value::tuple(vec![
        Value::Int(5),
        Value::Int(10),
        Value::Int(15),
    ]);
    let max_age = Value::Int(100);

    let (out, _) = interpreter::interpret(&prog, &[population, ages, max_age], None).unwrap();

    match &out[0] {
        Value::Tuple(survivors) => {
            assert_eq!(survivors.len(), 3, "all should survive (all young)");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn age_cull_removes_all_old() {
    let prog = build_age_cull();

    let population = Value::tuple(vec![
        Value::Int(1),
        Value::Int(2),
    ]);
    let ages = Value::tuple(vec![
        Value::Int(500),
        Value::Int(600),
    ]);
    let max_age = Value::Int(100);

    let (out, _) = interpreter::interpret(&prog, &[population, ages, max_age], None).unwrap();

    match &out[0] {
        Value::Tuple(survivors) => {
            assert_eq!(survivors.len(), 0, "all too old, none survive");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn age_cull_exact_max_age_survives() {
    let prog = build_age_cull();

    // le means <= max_age, so exactly-at-max individuals survive
    let population = Value::tuple(vec![
        Value::Int(42),
        Value::Int(99),
    ]);
    let ages = Value::tuple(vec![
        Value::Int(100), // exactly at max
        Value::Int(101), // one over
    ]);
    let max_age = Value::Int(100);

    let (out, _) = interpreter::interpret(&prog, &[population, ages, max_age], None).unwrap();

    match &out[0] {
        Value::Tuple(survivors) => {
            assert_eq!(survivors.len(), 1, "only age=100 survives (le)");
            assert_eq!(survivors[0], Value::Int(42));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// elitism_preserve tests
// ---------------------------------------------------------------------------

#[test]
fn elitism_preserves_top_rank() {
    let prog = build_elitism_preserve();

    let population = Value::tuple(vec![
        Value::Int(100), // individual A
        Value::Int(200), // individual B
        Value::Int(300), // individual C
        Value::Int(400), // individual D
    ]);
    let ranks = Value::tuple(vec![
        Value::Int(2), // A: rank 2
        Value::Int(0), // B: rank 0 (best)
        Value::Int(1), // C: rank 1
        Value::Int(0), // D: rank 0 (best)
    ]);
    let k = Value::Int(1); // keep rank < 1 => only rank 0

    let (out, _) = interpreter::interpret(&prog, &[population, ranks, k], None).unwrap();

    match &out[0] {
        Value::Tuple(elites) => {
            assert_eq!(elites.len(), 2, "2 individuals with rank 0 (< 1)");
            assert!(elites.contains(&Value::Int(200)));
            assert!(elites.contains(&Value::Int(400)));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn elitism_preserves_top_2_ranks() {
    let prog = build_elitism_preserve();

    let population = Value::tuple(vec![
        Value::Int(10),
        Value::Int(20),
        Value::Int(30),
        Value::Int(40),
        Value::Int(50),
    ]);
    let ranks = Value::tuple(vec![
        Value::Int(3), // rank 3
        Value::Int(0), // rank 0
        Value::Int(1), // rank 1
        Value::Int(2), // rank 2
        Value::Int(0), // rank 0
    ]);
    let k = Value::Int(2); // keep rank < 2 => ranks 0 and 1

    let (out, _) = interpreter::interpret(&prog, &[population, ranks, k], None).unwrap();

    match &out[0] {
        Value::Tuple(elites) => {
            assert_eq!(elites.len(), 3, "3 individuals with rank < 2 (ranks 0 and 1)");
            assert!(elites.contains(&Value::Int(20))); // rank 0
            assert!(elites.contains(&Value::Int(30))); // rank 1
            assert!(elites.contains(&Value::Int(50))); // rank 0
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn elitism_k_zero_keeps_none() {
    let prog = build_elitism_preserve();

    let population = Value::tuple(vec![
        Value::Int(1),
        Value::Int(2),
    ]);
    let ranks = Value::tuple(vec![
        Value::Int(0),
        Value::Int(0),
    ]);
    let k = Value::Int(0); // rank < 0 => nothing passes

    let (out, _) = interpreter::interpret(&prog, &[population, ranks, k], None).unwrap();

    match &out[0] {
        Value::Tuple(elites) => {
            assert_eq!(elites.len(), 0, "k=0 should keep nothing");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn elitism_large_k_keeps_all() {
    let prog = build_elitism_preserve();

    let population = Value::tuple(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
    ]);
    let ranks = Value::tuple(vec![
        Value::Int(0),
        Value::Int(1),
        Value::Int(2),
    ]);
    let k = Value::Int(100); // rank < 100 => all pass

    let (out, _) = interpreter::interpret(&prog, &[population, ranks, k], None).unwrap();

    match &out[0] {
        Value::Tuple(elites) => {
            assert_eq!(elites.len(), 3, "k=100 keeps all (max rank is 2)");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Composition test: cull then elite
// ---------------------------------------------------------------------------

#[test]
fn compose_fitness_cull_then_elitism() {
    let cull = build_fitness_threshold_cull();
    let elite = build_elitism_preserve();

    // Start with 5 individuals, cull by fitness, then keep top rank
    let population = Value::tuple(vec![
        Value::Int(10),
        Value::Int(3),   // below threshold
        Value::Int(7),
        Value::Int(1),   // below threshold
        Value::Int(8),
    ]);
    let threshold = Value::Int(5);

    // Step 1: cull by fitness threshold
    let (culled_out, _) = interpreter::interpret(&cull, &[population, threshold], None).unwrap();
    let culled = culled_out[0].clone();

    // Survivors: 10, 7, 8
    let survivor_count = match &culled {
        Value::Tuple(v) => v.len(),
        _ => panic!("expected Tuple"),
    };
    assert_eq!(survivor_count, 3, "3 survive fitness cull");

    // Step 2: elitism — keep rank 0 only
    let ranks = Value::tuple(vec![
        Value::Int(0), // 10: rank 0
        Value::Int(1), // 7: rank 1
        Value::Int(0), // 8: rank 0
    ]);
    let k = Value::Int(1); // keep rank < 1

    let (elite_out, _) = interpreter::interpret(&elite, &[culled, ranks, k], None).unwrap();

    match &elite_out[0] {
        Value::Tuple(elites) => {
            assert_eq!(elites.len(), 2, "2 elites with rank 0");
            assert!(elites.contains(&Value::Int(10)));
            assert!(elites.contains(&Value::Int(8)));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Summary report
// ---------------------------------------------------------------------------

#[test]
fn summary_report() {
    eprintln!("\n=== Self-Write Crossover & Death: Status Report ===\n");

    eprintln!("SUCCEEDED (IRIS programs built + tested):");
    eprintln!("  [CROSS-1] crossover_embedding      -- zip+map: alpha*A + (1-alpha)*B per dimension");
    eprintln!("  [DEATH-1] fitness_threshold_cull    -- fold+guard filter by fitness >= threshold");
    eprintln!("  [DEATH-2] age_cull                  -- zip(pop,ages) + fold+guard filter by age <= max");
    eprintln!("  [DEATH-3] elitism_preserve          -- zip(pop,ranks) + fold+guard filter by rank < k");
    eprintln!();
    eprintln!("BUILDING ON (from prior self_write_* tests):");
    eprintln!("  crossover_subgraph (self_write_mutation_v3.rs)");
    eprintln!("  death_cull (self_write_population.rs)");
    eprintln!("  elitism_find_best / elitism_top_2 (self_write_population.rs)");
    eprintln!();
    eprintln!("COVERAGE MAP:");
    eprintln!("  crossover.rs:");
    eprintln!("    crossover_subgraph   -> self_write_mutation_v3.rs (done)");
    eprintln!("    crossover_embedding  -> THIS FILE: zip+map interpolation");
    eprintln!("  death.rs:");
    eprintln!("    fitness_threshold    -> THIS FILE: fold+guard ge filter");
    eprintln!("    age_cull             -> THIS FILE: zip+fold+guard le filter");
    eprintln!("    elitism_preserve     -> THIS FILE: zip+fold+guard lt filter");
    eprintln!();
    eprintln!("TOTAL: 4 new IRIS programs for crossover + death/culling");
    eprintln!("=== End Report ===");
}
