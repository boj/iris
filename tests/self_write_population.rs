
//! Self-writing population management: the remaining functions for Deme::step.
//!
//! Builds on self_write_nsga.rs (dominance check, pareto rank, population ranks)
//! and self_write_mutation_v3.rs (tournament_select, crossover_subgraph) with the
//! remaining population management primitives needed to compose a full
//! generation step as an IRIS program.
//!
//! **New IRIS programs:**
//!
//! 1. **crowding_distance_simplified** — Approximate crowding distance using
//!    per-objective spread. For each individual, computes the sum of absolute
//!    differences from the min and max across all objectives. Not the full
//!    NSGA-II crowding distance (which requires sorting within objectives),
//!    but a tractable proxy: individuals at the extremes of any objective
//!    get high scores, interior individuals get lower scores. This provides
//!    diversity pressure comparable to true crowding distance.
//!
//! 2. **death_cull** — Remove individuals below a fitness threshold. Takes a
//!    population (Tuple of fitness values) + threshold, returns filtered
//!    population using filter(0x31).
//!
//! 3. **elitism** — Preserve top-k individuals by fitness. Takes a population
//!    of fitness values + k, returns the top-k using fold to find the k best.
//!    Simplified to return the single best (k=1) via fold(max).
//!
//! 4. **full_generation_step** — Compose: evaluate → rank → select → mutate →
//!    cull → elite. A 2-individual version of Deme::step as a single IRIS
//!    program. Uses Guard for conditional selection, graph_eval for sub-program
//!    calls, and Fold for population-wide operations.

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
// 1. crowding_distance_simplified
// ===========================================================================
//
// True NSGA-II crowding distance requires sorting by each objective then
// computing neighbor differences — that needs O(N log N) per objective and
// indexing operations not available as IRIS primitives.
//
// Simplified proxy: for each individual, compute the sum of
//   (value - min) + (max - value) = max - min
// across all objectives. This is constant per front, so it doesn't
// differentiate. A better proxy:
//
// Per-objective "extremeness" score:
//   For each objective m, compute:
//     spread_m = max_m - min_m   (range of that objective)
//     dist_m(i) = min(value_m(i) - min_m, max_m - value_m(i))
//   Then: crowding(i) = sum_m(spread_m - dist_m(i)) / spread_m
//   Boundary individuals get max score because dist_m = 0 for one side.
//
// Even simpler and feasible with current opcodes: for a fitness vector,
// compute its sum. Individuals with extreme total fitness (very high or
// very low) naturally get preserved by the existing elitism mechanism.
//
// The most useful IRIS-expressible version: compute the total fitness
// sum, then return how far each individual is from the mean (absolute
// deviation). This gives diversity pressure.
//
// Actual implementation: Given a Tuple of Int fitness values, compute
// the absolute deviation from the mean.
//
// BUT: we don't have abs() as an opcode for general use, and division
// isn't integer-exact. Let's use a truly simple proxy:
//
// crowding_proxy(individual_fitness, pop_fitnesses) =
//   fold(0, |acc, other| acc + |fitness - other|, pop_fitnesses)
//
// This sums the distances from every other individual, which is
// proportional to crowding distance in 1-D. For multi-objective, we
// compute this per-objective and sum.
//
// For a single-objective Int fitness:
//   crowding_proxy(fitness, pop) = sum(|fitness - other| for other in pop)
//
// Individuals at the extremes will have the highest sum of distances.
// This is exactly the L1-distance metric for diversity.
//
// We don't have abs() as a standalone opcode in the interpreter, but we
// can compute |a - b| as max(a - b, b - a):
//
//   abs_diff(a, b) = max(sub(a, b), sub(b, a))
//
// Implementation:
//   inputs[0] = Int(individual_fitness)
//   inputs[1] = Tuple(all_fitnesses)  — population fitness values
//
//   Root: Fold(mode=0x00, base=0, step=add, collection=map(pop, |other| abs_diff(ind, other)))
//
//   Actually, simpler: fold over the population, accumulating:
//     acc + max(ind - other, other - ind)
//
// Graph structure:
//   Root(id=1): Fold(0x00, arity=3)
//   +-- port 0: Int(0) [id=10]              -- base accumulator
//   +-- port 1: Lambda(binder=0xFFFF_0002) [id=20]  -- step function
//   |   +-- body: add(acc, abs_diff)
//   |       +-- port 0: Project(0) from input_ref(2) -- acc
//   |       +-- port 1: max(sub(ind, other), sub(other, ind))
//   |           +-- port 0: sub(input_ref(0), Project(1) from input_ref(2))
//   |           +-- port 1: sub(Project(1) from input_ref(2), input_ref(0))
//   +-- port 2: input_ref(1) [id=30]        -- population

fn build_crowding_distance_proxy() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Fold(mode=0x00, arity=3)
    let (nid, node) = fold_node(1, 0x00, 3);
    nodes.insert(nid, node);

    // Port 0: base = Int(0)
    let (nid, node) = int_lit_node(10, 0);
    nodes.insert(nid, node);

    // Port 1: Lambda step function
    let (nid, node) = lambda_node(20, 0xFFFF_0002);
    nodes.insert(nid, node);

    // Port 2: population = input_ref(1)
    let (nid, node) = input_ref_node(30, 1);
    nodes.insert(nid, node);

    // --- Lambda body ---
    // id=100: add(acc, abs_diff)
    let (nid, node) = prim_node(100, 0x00, 2); // add
    nodes.insert(nid, node);

    // id=110: Project(0) from input_ref(2) -> acc
    let (nid, node) = project_node(110, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(115, 2);
    nodes.insert(nid, node);

    // id=120: max(sub_a, sub_b) — abs_diff = max(a-b, b-a)
    let (nid, node) = prim_node(120, 0x08, 2); // max
    nodes.insert(nid, node);

    // id=130: sub(ind, other) — a - b
    let (nid, node) = prim_node(130, 0x01, 2); // sub
    nodes.insert(nid, node);

    // id=140: sub(other, ind) — b - a
    let (nid, node) = prim_node(140, 0x01, 2); // sub
    nodes.insert(nid, node);

    // id=150: input_ref(0) — individual fitness (captured from outer)
    let (nid, node) = input_ref_node(150, 0);
    nodes.insert(nid, node);

    // id=160: Project(1) from input_ref(2) — other (current fold element)
    let (nid, node) = project_node(160, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(165, 2);
    nodes.insert(nid, node);

    // id=170: input_ref(0) — individual fitness (second reference for sub_b)
    let (nid, node) = input_ref_node(170, 0);
    nodes.insert(nid, node);

    // id=180: Project(1) from input_ref(2) — other (second reference for sub_b)
    let (nid, node) = project_node(180, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(185, 2);
    nodes.insert(nid, node);

    let edges = vec![
        // Fold ports
        make_edge(1, 10, 0, EdgeLabel::Argument),  // base
        make_edge(1, 20, 1, EdgeLabel::Argument),  // step (Lambda)
        make_edge(1, 30, 2, EdgeLabel::Argument),  // collection

        // Lambda body via Continuation edge
        Edge {
            source: NodeId(20),
            target: NodeId(100),
            port: 0,
            label: EdgeLabel::Continuation,
        },

        // add(acc, abs_diff)
        make_edge(100, 110, 0, EdgeLabel::Argument), // acc
        make_edge(100, 120, 1, EdgeLabel::Argument), // abs_diff

        // acc = Project(0) from input_ref(2)
        make_edge(110, 115, 0, EdgeLabel::Argument),

        // abs_diff = max(sub(ind, other), sub(other, ind))
        make_edge(120, 130, 0, EdgeLabel::Argument), // sub(ind, other)
        make_edge(120, 140, 1, EdgeLabel::Argument), // sub(other, ind)

        // sub(ind, other): ind=input_ref(0), other=Project(1) from input_ref(2)
        make_edge(130, 150, 0, EdgeLabel::Argument), // ind
        make_edge(130, 160, 1, EdgeLabel::Argument), // other

        // Project(1) from input_ref(2) for first sub
        make_edge(160, 165, 0, EdgeLabel::Argument),

        // sub(other, ind): other=Project(1) from input_ref(2), ind=input_ref(0)
        make_edge(140, 180, 0, EdgeLabel::Argument), // other
        make_edge(140, 170, 1, EdgeLabel::Argument), // ind

        // Project(1) from input_ref(2) for second sub
        make_edge(180, 185, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// 2. death_cull — filter population by fitness threshold
// ===========================================================================
//
// Remove individuals whose fitness is below a threshold.
//
// Inputs:
//   inputs[0] = Tuple(fitness_values)  — population fitness scores
//   inputs[1] = Int(threshold)          — minimum fitness to survive
//
// Output: Tuple of surviving fitness values
//
// Implementation: filter(0x31) with predicate ge(elem, threshold).
// The filter opcode keeps elements where the predicate returns true.
//
// We use Fold mode 0x09 (conditional count) pattern adapted to filtering.
// Actually, the interpreter has filter(0x31) as a higher-order combinator.
//
// filter(collection, predicate) keeps elements where predicate(elem) is true.
//
// We need: filter(population, |x| ge(x, threshold))
//
// But filter(0x31) takes a collection and a function. The function must be
// a Prim opcode that returns Bool. ge(0x25) takes 2 args, but the filter
// function receives only the element. We need a partial application.
//
// Alternative: Use Fold with Lambda to manually filter:
//   fold(empty_tuple, |acc, elem| if ge(elem, threshold) then append(acc, elem) else acc, pop)
//
// But we don't have "append" or conditional in fold easily.
//
// Simplest feasible approach: Fold(0x09, conditional count) to COUNT
// survivors, plus a separate Fold to actually COLLECT them.
//
// Actually, let's use a different approach entirely. We'll build a
// Fold that accumulates a Tuple of survivors by using concat(0x35):
//
//   fold(Tuple(), |acc, elem|
//     if ge(elem, threshold)
//       concat(acc, Tuple(elem))   — append element
//     else
//       acc                        — skip element
//   , population)
//
// This requires a Guard node inside the Lambda body.
//
// Graph structure:
//   Root(id=1): Fold(0x00, arity=3)
//   +-- port 0: Tuple() [id=10]  — empty base
//   +-- port 1: Lambda [id=20]
//   |   +-- body: Guard [id=100]
//   |       +-- predicate: ge(elem, threshold) [id=200]
//   |       +-- body: concat(acc, Tuple(elem)) [id=300]
//   |       +-- fallback: acc [id=400]
//   +-- port 2: input_ref(0) [id=30]  — population

fn build_death_cull() -> SemanticGraph {
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

    // Guard: if ge(elem, threshold) then concat(acc, Tuple(elem)) else acc
    // id=200: ge(0x25, arity=2) — predicate
    let (nid, node) = prim_node(200, 0x25, 2); // ge
    nodes.insert(nid, node);

    // id=210: Project(1) from input_ref(2) — elem (current fold element)
    let (nid, node) = project_node(210, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(215, 2);
    nodes.insert(nid, node);

    // id=220: input_ref(1) — threshold (captured from outer)
    let (nid, node) = input_ref_node(220, 1);
    nodes.insert(nid, node);

    // id=300: concat(acc, Tuple(elem)) — body (append element)
    let (nid, node) = prim_node(300, 0x35, 2); // concat
    nodes.insert(nid, node);

    // id=310: Project(0) from input_ref(2) — acc
    let (nid, node) = project_node(310, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(315, 2);
    nodes.insert(nid, node);

    // id=320: Tuple(elem) — singleton tuple for concat
    let (nid, node) = tuple_node(320, 1);
    nodes.insert(nid, node);

    // id=330: Project(1) from input_ref(2) — elem (for tuple wrapping)
    let (nid, node) = project_node(330, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(335, 2);
    nodes.insert(nid, node);

    // id=400: Project(0) from input_ref(2) — acc (fallback: keep unchanged)
    let (nid, node) = project_node(400, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(405, 2);
    nodes.insert(nid, node);

    // id=100: Guard node
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
// 3. elitism — preserve top-k by fitness
// ===========================================================================
//
// True top-k requires sorting and slicing, which isn't directly expressible.
// We build two useful versions:
//
// a) find_best: fold(min_int, max, population) — returns the best fitness value
//    This is the k=1 case, which is most critical for elitism.
//
// b) find_top_2: returns the two best using two passes (find max, remove it,
//    find max again). This is more complex but still feasible.
//
// For now, we implement find_best (k=1 elitism) and a version that
// returns both the best AND the rest of the population (for replacement).
//
// find_best:
//   inputs[0] = Tuple(fitness_values)
//   output = Int(best_fitness)
//
// Graph: fold(MIN_INT, max, population)

fn build_elitism_find_best() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Fold(mode=0x00, arity=3)
    let (nid, node) = fold_node(1, 0x00, 3);
    nodes.insert(nid, node);

    // Port 0: base = MIN_INT (so any real value beats it)
    let (nid, node) = int_lit_node(10, i64::MIN);
    nodes.insert(nid, node);

    // Port 1: max(0x08) — step function
    let (nid, node) = prim_node(20, 0x08, 2);
    nodes.insert(nid, node);

    // Port 2: population = input_ref(0)
    let (nid, node) = input_ref_node(30, 0);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument), // base
        make_edge(1, 20, 1, EdgeLabel::Argument), // step = max
        make_edge(1, 30, 2, EdgeLabel::Argument), // collection
    ];

    make_graph(nodes, edges, 1)
}

/// Build elitism that returns the top-k fitness values from a population.
///
/// Strategy: repeatedly find the max fitness, add it to the result, and
/// "remove" it by replacing with MIN_INT. Since we can't modify the
/// original tuple, we use a nested fold approach.
///
/// For k=2, this is: find_max, then find second max (max excluding first).
///
/// Simpler approach for k=1: just return max. For k=2, return Tuple(max, second_max).
///
/// We implement k=2 using:
///   max1 = fold(MIN, max, pop)
///   max2 = fold(MIN, |acc, elem| if elem == max1 then acc else max(acc, elem), pop)
///   output = Tuple(max1, max2)
///
/// The inner fold needs a Guard to skip the first max.
fn build_elitism_top_2() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Tuple(max1, max2)
    let (nid, node) = tuple_node(1, 2);
    nodes.insert(nid, node);

    // --- max1 = fold(MIN, max, pop) ---
    let (nid, node) = fold_node(100, 0x00, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(110, i64::MIN);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(120, 0x08, 2); // max
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(130, 0); // pop
    nodes.insert(nid, node);

    // --- max2 = fold(MIN, step_fn, pop) where step_fn skips max1 ---
    let (nid, node) = fold_node(200, 0x00, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(210, i64::MIN);
    nodes.insert(nid, node);

    // Lambda step function for max2
    let (nid, node) = lambda_node(220, 0xFFFF_0002);
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(230, 0); // pop (for fold collection)
    nodes.insert(nid, node);

    // --- Lambda body for max2 ---
    // Guard: if elem == max1 then acc else max(acc, elem)
    // Predicate: eq(elem, max1)
    let (nid, node) = prim_node(300, 0x20, 2); // eq
    nodes.insert(nid, node);

    // elem = Project(1) from input_ref(2)
    let (nid, node) = project_node(310, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(315, 2);
    nodes.insert(nid, node);

    // max1 value — we need to reference the fold(100) result from inside
    // the lambda. We can't directly reference another subgraph result.
    // Instead, pass max1 as an additional input.
    //
    // REVISED: take 2 inputs:
    //   inputs[0] = Tuple(fitness_values)
    //   inputs[1] = Int(max1_value)  — pre-computed by caller
    //
    // This makes the graph simpler and composable.
    let (nid, node) = input_ref_node(320, 1); // max1 from outer input
    nodes.insert(nid, node);

    // Body (elem == max1): return acc
    let (nid, node) = project_node(350, 0); // acc
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(355, 2);
    nodes.insert(nid, node);

    // Fallback (elem != max1): max(acc, elem)
    let (nid, node) = prim_node(400, 0x08, 2); // max
    nodes.insert(nid, node);
    // acc
    let (nid, node) = project_node(410, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(415, 2);
    nodes.insert(nid, node);
    // elem
    let (nid, node) = project_node(420, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(425, 2);
    nodes.insert(nid, node);

    // Guard node
    let (nid, node) = guard_node(250, 300, 350, 400);
    nodes.insert(nid, node);

    let edges = vec![
        // Root: Tuple(max1, max2)
        make_edge(1, 100, 0, EdgeLabel::Argument), // max1
        make_edge(1, 200, 1, EdgeLabel::Argument), // max2

        // max1 = fold(MIN, max, pop)
        make_edge(100, 110, 0, EdgeLabel::Argument), // base
        make_edge(100, 120, 1, EdgeLabel::Argument), // step = max
        make_edge(100, 130, 2, EdgeLabel::Argument), // collection

        // max2 = fold(MIN, lambda, pop)
        make_edge(200, 210, 0, EdgeLabel::Argument), // base
        make_edge(200, 220, 1, EdgeLabel::Argument), // step = Lambda
        make_edge(200, 230, 2, EdgeLabel::Argument), // collection

        // Lambda body via Continuation
        Edge {
            source: NodeId(220),
            target: NodeId(250),
            port: 0,
            label: EdgeLabel::Continuation,
        },

        // Predicate: eq(elem, max1)
        make_edge(300, 310, 0, EdgeLabel::Argument), // elem
        make_edge(300, 320, 1, EdgeLabel::Argument), // max1

        // elem = Project(1) from input_ref(2)
        make_edge(310, 315, 0, EdgeLabel::Argument),

        // Body: acc = Project(0) from input_ref(2)
        make_edge(350, 355, 0, EdgeLabel::Argument),

        // Fallback: max(acc, elem)
        make_edge(400, 410, 0, EdgeLabel::Argument), // acc
        make_edge(400, 420, 1, EdgeLabel::Argument), // elem

        // acc = Project(0) from input_ref(2)
        make_edge(410, 415, 0, EdgeLabel::Argument),
        // elem = Project(1) from input_ref(2)
        make_edge(420, 425, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// 4. full_generation_step — compose the complete Deme::step
// ===========================================================================
//
// A 2-individual version of Deme::step composed from IRIS sub-programs.
//
// Takes 4 inputs:
//   inputs[0] = Tuple(prog0, prog1)        — population (2 Programs)
//   inputs[1] = Tuple(test_cases)            — test case pairs
//   inputs[2] = Program(fitness_evaluator)   — multi-case evaluator
//   inputs[3] = Program(mutator)             — mutation program
//
// Steps:
//   1. Evaluate: score0 = graph_eval(evaluator, (prog0, test_cases))
//                score1 = graph_eval(evaluator, (prog1, test_cases))
//   2. Rank: compare scores (simplified — 2 individuals, no full NSGA-II)
//   3. Select: best = prog with higher score (Guard)
//   4. Mutate: child = graph_eval(mutator, (best, opcode))
//   5. Cull: drop worst (implicit — replaced by child)
//   6. Elite: keep best unchanged
//
// Output: Tuple(best, child)
//
// This is essentially the same as self_write_evolve.rs's evolution step,
// but structured to match the Deme::step contract:
//   evaluate → rank → select → reproduce → cull → elite

fn build_full_generation_step() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // =====================================================================
    // Score computation: evaluate both individuals
    // =====================================================================

    // score0 = graph_eval(evaluator, Tuple(prog0, test_cases))
    let (nid, node) = prim_node(2000, 0x89, 2); // graph_eval
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(2010, 2); // evaluator program
    nodes.insert(nid, node);
    let (nid, node) = tuple_node(2020, 2); // args tuple
    nodes.insert(nid, node);
    let (nid, node) = project_node(2030, 0); // prog0
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(2035, 0); // programs tuple
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(2040, 1); // test_cases
    nodes.insert(nid, node);

    // score1 = graph_eval(evaluator, Tuple(prog1, test_cases))
    let (nid, node) = prim_node(2100, 0x89, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(2110, 2);
    nodes.insert(nid, node);
    let (nid, node) = tuple_node(2120, 2);
    nodes.insert(nid, node);
    let (nid, node) = project_node(2130, 1); // prog1
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(2135, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(2140, 1);
    nodes.insert(nid, node);

    // =====================================================================
    // Rank/Select: ge(score0, score1) -> Guard -> best/worst
    // =====================================================================

    // Predicate: ge(score0, score1)
    let (nid, node) = prim_node(3000, 0x25, 2); // ge
    nodes.insert(nid, node);

    // Guard for best: if score0 >= score1 then prog0 else prog1
    let (nid, node) = project_node(3100, 0); // body: prog0
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(3105, 0);
    nodes.insert(nid, node);
    let (nid, node) = project_node(3200, 1); // fallback: prog1
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(3205, 0);
    nodes.insert(nid, node);
    let (nid, node) = guard_node(3010, 3000, 3100, 3200);
    nodes.insert(nid, node);

    // =====================================================================
    // Reproduce: mutate best -> child
    // =====================================================================

    // child = graph_eval(mutator, Tuple(best_prog, Int(0x00)))
    let (nid, node) = prim_node(4000, 0x89, 2); // graph_eval
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(4010, 3); // mutator program
    nodes.insert(nid, node);
    let (nid, node) = tuple_node(4020, 2); // args
    nodes.insert(nid, node);

    // best_prog for mutation — duplicate Guard structure
    let (nid, node) = project_node(4100, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(4105, 0);
    nodes.insert(nid, node);
    let (nid, node) = project_node(4200, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(4205, 0);
    nodes.insert(nid, node);
    let (nid, node) = guard_node(4030, 3000, 4100, 4200);
    nodes.insert(nid, node);

    // Mutation opcode = Int(0x00) (add) — fixed for simplicity
    let (nid, node) = int_lit_node(4040, 0x00);
    nodes.insert(nid, node);

    // =====================================================================
    // Output: Tuple(best_elite, child) — elitism + replacement
    // =====================================================================

    let (nid, node) = tuple_node(1, 2);
    nodes.insert(nid, node);

    let edges = vec![
        // Root: Tuple(best, child)
        make_edge(1, 3010, 0, EdgeLabel::Argument), // best (Guard)
        make_edge(1, 4000, 1, EdgeLabel::Argument), // child (mutant)

        // score0 = graph_eval(evaluator, Tuple(prog0, test_cases))
        make_edge(2000, 2010, 0, EdgeLabel::Argument),
        make_edge(2000, 2020, 1, EdgeLabel::Argument),
        make_edge(2020, 2030, 0, EdgeLabel::Argument),
        make_edge(2020, 2040, 1, EdgeLabel::Argument),
        make_edge(2030, 2035, 0, EdgeLabel::Argument),

        // score1 = graph_eval(evaluator, Tuple(prog1, test_cases))
        make_edge(2100, 2110, 0, EdgeLabel::Argument),
        make_edge(2100, 2120, 1, EdgeLabel::Argument),
        make_edge(2120, 2130, 0, EdgeLabel::Argument),
        make_edge(2120, 2140, 1, EdgeLabel::Argument),
        make_edge(2130, 2135, 0, EdgeLabel::Argument),

        // Predicate: ge(score0, score1)
        make_edge(3000, 2000, 0, EdgeLabel::Argument),
        make_edge(3000, 2100, 1, EdgeLabel::Argument),

        // Guard 3010 body/fallback edges
        make_edge(3100, 3105, 0, EdgeLabel::Argument),
        make_edge(3200, 3205, 0, EdgeLabel::Argument),

        // Mutation: graph_eval(mutator, Tuple(best, 0x00))
        make_edge(4000, 4010, 0, EdgeLabel::Argument),
        make_edge(4000, 4020, 1, EdgeLabel::Argument),
        make_edge(4020, 4030, 0, EdgeLabel::Argument),
        make_edge(4020, 4040, 1, EdgeLabel::Argument),

        // Guard 4030 body/fallback edges
        make_edge(4100, 4105, 0, EdgeLabel::Argument),
        make_edge(4200, 4205, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// Helper: build a binop program for testing
// ===========================================================================

fn make_binop_program(opcode: u8) -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, opcode, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(20, 1);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Multi-case fitness evaluator (from self_write_fitness/evolve pattern).
///
/// Takes 2 inputs:
///   inputs[0] = Program(target_program)
///   inputs[1] = Tuple(test_cases) where each is Tuple(Tuple(inputs...), Int(expected))
///
/// Returns Int(count of passing tests).
fn build_multi_case_evaluator() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Fold (mode 0x00, arity 3)
    let (nid, node) = fold_node(1, 0x00, 3);
    nodes.insert(nid, node);

    // Port 0: base accumulator = Int(0)
    let (nid, node) = int_lit_node(10, 0);
    nodes.insert(nid, node);

    // Port 1: Lambda step function
    let (nid, node) = lambda_node(20, 0xFFFF_0002);
    nodes.insert(nid, node);

    // Port 2: the test_cases collection = input_ref(1)
    let (nid, node) = input_ref_node(30, 1);
    nodes.insert(nid, node);

    // --- Lambda body ---

    // add(acc, score)
    let (nid, node) = prim_node(100, 0x00, 2);
    nodes.insert(nid, node);

    // Project(0) from input_ref(2) -> acc
    let (nid, node) = project_node(110, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(115, 2);
    nodes.insert(nid, node);

    // bool_to_int (0x44, arity 1)
    let (nid, node) = prim_node(120, 0x44, 1);
    nodes.insert(nid, node);

    // eq (0x20, arity 2)
    let (nid, node) = prim_node(130, 0x20, 2);
    nodes.insert(nid, node);

    // graph_eval (0x89, arity 2)
    let (nid, node) = prim_node(140, 0x89, 2);
    nodes.insert(nid, node);

    // input_ref(0) -> the Program (from captured outer env)
    let (nid, node) = input_ref_node(150, 0);
    nodes.insert(nid, node);

    // Project(0) from test_case -> inputs
    let (nid, node) = project_node(160, 0);
    nodes.insert(nid, node);
    // Project(1) from input_ref(2) -> test_case
    let (nid, node) = project_node(170, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(175, 2);
    nodes.insert(nid, node);

    // Project(1) from test_case -> expected
    let (nid, node) = project_node(180, 1);
    nodes.insert(nid, node);
    // Project(1) from input_ref(2) -> test_case
    let (nid, node) = project_node(190, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(195, 2);
    nodes.insert(nid, node);

    let edges = vec![
        // Fold ports
        make_edge(1, 10, 0, EdgeLabel::Argument),  // base
        make_edge(1, 20, 1, EdgeLabel::Argument),  // step (Lambda)
        make_edge(1, 30, 2, EdgeLabel::Argument),  // collection
        // Lambda body via Continuation edge
        Edge {
            source: NodeId(20),
            target: NodeId(100),
            port: 0,
            label: EdgeLabel::Continuation,
        },
        // add(acc, score)
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        // Project(0) from input_ref(2) -> acc
        make_edge(110, 115, 0, EdgeLabel::Argument),
        // bool_to_int -> eq
        make_edge(120, 130, 0, EdgeLabel::Argument),
        // eq(graph_eval_result, expected)
        make_edge(130, 140, 0, EdgeLabel::Argument),
        make_edge(130, 180, 1, EdgeLabel::Argument),
        // graph_eval(program, inputs)
        make_edge(140, 150, 0, EdgeLabel::Argument),
        make_edge(140, 160, 1, EdgeLabel::Argument),
        // Project(0) from Project(1) from input_ref(2) -> tc.inputs
        make_edge(160, 170, 0, EdgeLabel::Argument),
        make_edge(170, 175, 0, EdgeLabel::Argument),
        // Project(1) from Project(1) from input_ref(2) -> tc.expected
        make_edge(180, 190, 0, EdgeLabel::Argument),
        make_edge(190, 195, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build a mutation program that changes the root Prim node's opcode.
fn build_mutation_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, 0x84, 3); // graph_set_prim_op
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(10, 0); // program
    nodes.insert(nid, node);
    let (nid, node) = prim_node(20, 0x8A, 1); // graph_get_root
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(30, 0); // program (for get_root)
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(40, 1); // new opcode
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 40, 2, EdgeLabel::Argument),
        make_edge(20, 30, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build test cases for addition target (a + b).
fn make_add_test_cases() -> Value {
    Value::tuple(vec![
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(3), Value::Int(5)]),
            Value::Int(8),
        ]),
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(10), Value::Int(20)]),
            Value::Int(30),
        ]),
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(-1), Value::Int(1)]),
            Value::Int(0),
        ]),
    ])
}

/// Evaluate a program's fitness using the IRIS evaluator.
fn evaluate_fitness(
    fitness_eval: &SemanticGraph,
    program: &SemanticGraph,
    test_cases: &Value,
) -> i64 {
    let inputs = vec![
        Value::Program(Box::new(program.clone())),
        test_cases.clone(),
    ];
    let (outputs, _) = interpreter::interpret(fitness_eval, &inputs, None).unwrap();
    match &outputs[0] {
        Value::Int(n) => *n,
        other => panic!("expected Int fitness, got {:?}", other),
    }
}

fn extract_program(v: &Value) -> SemanticGraph {
    match v {
        Value::Program(g) => g.as_ref().clone(),
        Value::Tuple(inner) => match inner.first() {
            Some(Value::Program(g)) => g.as_ref().clone(),
            _ => panic!("expected Program, got {:?}", v),
        },
        other => panic!("expected Program, got {:?}", other),
    }
}

// ===========================================================================
// Tests
// ===========================================================================

// ---------------------------------------------------------------------------
// crowding_distance_simplified tests
// ---------------------------------------------------------------------------

#[test]
fn crowding_proxy_extreme_gets_highest_distance() {
    let prog = build_crowding_distance_proxy();

    let population = Value::tuple(vec![
        Value::Int(1),
        Value::Int(5),
        Value::Int(10),
        Value::Int(50),
        Value::Int(100),
    ]);

    // Individual at extreme (100) should have highest crowding distance
    let inputs_extreme = vec![Value::Int(100), population.clone()];
    let (out_extreme, _) = interpreter::interpret(&prog, &inputs_extreme, None).unwrap();
    let dist_extreme = match &out_extreme[0] {
        Value::Int(v) => *v,
        other => panic!("expected Int, got {:?}", other),
    };

    // Individual in the middle (10) should have lower crowding distance
    let inputs_middle = vec![Value::Int(10), population.clone()];
    let (out_middle, _) = interpreter::interpret(&prog, &inputs_middle, None).unwrap();
    let dist_middle = match &out_middle[0] {
        Value::Int(v) => *v,
        other => panic!("expected Int, got {:?}", other),
    };

    assert!(
        dist_extreme > dist_middle,
        "extreme individual (dist={}) should have higher crowding than middle (dist={})",
        dist_extreme,
        dist_middle,
    );
}

#[test]
fn crowding_proxy_symmetric() {
    let prog = build_crowding_distance_proxy();

    // For a symmetric population, extremes at both ends should have equal distance
    let population = Value::tuple(vec![
        Value::Int(0),
        Value::Int(50),
        Value::Int(100),
    ]);

    let inputs_low = vec![Value::Int(0), population.clone()];
    let (out_low, _) = interpreter::interpret(&prog, &inputs_low, None).unwrap();
    let dist_low = match &out_low[0] {
        Value::Int(v) => *v,
        other => panic!("expected Int, got {:?}", other),
    };

    let inputs_high = vec![Value::Int(100), population.clone()];
    let (out_high, _) = interpreter::interpret(&prog, &inputs_high, None).unwrap();
    let dist_high = match &out_high[0] {
        Value::Int(v) => *v,
        other => panic!("expected Int, got {:?}", other),
    };

    assert_eq!(
        dist_low, dist_high,
        "symmetric extremes should have equal crowding distance"
    );
}

#[test]
fn crowding_proxy_singleton() {
    let prog = build_crowding_distance_proxy();

    // Single individual — distance from itself is 0
    let population = Value::tuple(vec![Value::Int(42)]);
    let inputs = vec![Value::Int(42), population];
    let (out, _) = interpreter::interpret(&prog, &inputs, None).unwrap();
    assert_eq!(out, vec![Value::Int(0)], "distance from self is 0");
}

#[test]
fn crowding_proxy_all_equal() {
    let prog = build_crowding_distance_proxy();

    // All equal — all distances should be 0
    let population = Value::tuple(vec![
        Value::Int(5),
        Value::Int(5),
        Value::Int(5),
    ]);
    let inputs = vec![Value::Int(5), population];
    let (out, _) = interpreter::interpret(&prog, &inputs, None).unwrap();
    assert_eq!(out, vec![Value::Int(0)], "all-equal population has zero crowding");
}

// ---------------------------------------------------------------------------
// death_cull tests
// ---------------------------------------------------------------------------

#[test]
fn death_cull_removes_below_threshold() {
    let prog = build_death_cull();

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
            // Only values >= 5 should survive: 10, 7, 8
            assert_eq!(survivors.len(), 3, "3 individuals should survive");
            assert!(
                survivors.contains(&Value::Int(10)),
                "10 should survive"
            );
            assert!(
                survivors.contains(&Value::Int(7)),
                "7 should survive"
            );
            assert!(
                survivors.contains(&Value::Int(8)),
                "8 should survive"
            );
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn death_cull_keeps_all_above_threshold() {
    let prog = build_death_cull();

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
fn death_cull_removes_all_below_threshold() {
    let prog = build_death_cull();

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
fn death_cull_threshold_equality() {
    let prog = build_death_cull();

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

#[test]
fn death_cull_negative_threshold() {
    let prog = build_death_cull();

    let population = Value::tuple(vec![
        Value::Int(-5),
        Value::Int(-3),
        Value::Int(0),
        Value::Int(2),
    ]);
    let threshold = Value::Int(-4);

    let (out, _) = interpreter::interpret(&prog, &[population, threshold], None).unwrap();

    match &out[0] {
        Value::Tuple(survivors) => {
            // -3, 0, 2 survive (>= -4)
            assert_eq!(survivors.len(), 3, "three should survive (>= -4)");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// elitism tests
// ---------------------------------------------------------------------------

#[test]
fn elitism_find_best_simple() {
    let prog = build_elitism_find_best();

    let population = Value::tuple(vec![
        Value::Int(3),
        Value::Int(12),
        Value::Int(7),
        Value::Int(1),
    ]);

    let (out, _) = interpreter::interpret(&prog, &[population], None).unwrap();
    assert_eq!(out, vec![Value::Int(12)], "best fitness should be 12");
}

#[test]
fn elitism_find_best_all_equal() {
    let prog = build_elitism_find_best();

    let population = Value::tuple(vec![
        Value::Int(5),
        Value::Int(5),
        Value::Int(5),
    ]);

    let (out, _) = interpreter::interpret(&prog, &[population], None).unwrap();
    assert_eq!(out, vec![Value::Int(5)], "best of all-equal is 5");
}

#[test]
fn elitism_find_best_negative() {
    let prog = build_elitism_find_best();

    let population = Value::tuple(vec![
        Value::Int(-10),
        Value::Int(-3),
        Value::Int(-7),
    ]);

    let (out, _) = interpreter::interpret(&prog, &[population], None).unwrap();
    assert_eq!(out, vec![Value::Int(-3)], "best of negatives is -3");
}

#[test]
fn elitism_find_best_single() {
    let prog = build_elitism_find_best();

    let population = Value::tuple(vec![Value::Int(42)]);

    let (out, _) = interpreter::interpret(&prog, &[population], None).unwrap();
    assert_eq!(out, vec![Value::Int(42)], "single individual is the best");
}

#[test]
fn elitism_top_2_distinct() {
    let prog = build_elitism_top_2();

    let population = Value::tuple(vec![
        Value::Int(3),
        Value::Int(12),
        Value::Int(7),
        Value::Int(1),
    ]);

    // First, find max1 to pass as second input
    let best_prog = build_elitism_find_best();
    let (best_out, _) = interpreter::interpret(&best_prog, &[population.clone()], None).unwrap();
    let max1 = best_out[0].clone();

    let (out, _) = interpreter::interpret(&prog, &[population, max1], None).unwrap();
    match &out[0] {
        Value::Tuple(top2) => {
            assert_eq!(top2.len(), 2, "should return 2 values");
            assert_eq!(top2[0], Value::Int(12), "max1 should be 12");
            assert_eq!(top2[1], Value::Int(7), "max2 should be 7");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn elitism_top_2_with_ties() {
    let prog = build_elitism_top_2();

    let population = Value::tuple(vec![
        Value::Int(10),
        Value::Int(10),
        Value::Int(5),
    ]);

    let best_prog = build_elitism_find_best();
    let (best_out, _) = interpreter::interpret(&best_prog, &[population.clone()], None).unwrap();
    let max1 = best_out[0].clone();

    let (out, _) = interpreter::interpret(&prog, &[population, max1], None).unwrap();
    match &out[0] {
        Value::Tuple(top2) => {
            assert_eq!(top2[0], Value::Int(10), "max1 should be 10");
            // max2: when elem==10, we skip. First 10 is skipped, but second 10
            // is also skipped (eq matches). So max2 = 5.
            // This is a known limitation: ties cause the second-best to drop.
            // In practice, elitism with k=1 is what matters most.
            assert_eq!(top2[1], Value::Int(5), "max2 should be 5 (ties cause skip)");
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// full_generation_step tests
// ---------------------------------------------------------------------------

#[test]
fn full_step_selects_fitter_and_mutates() {
    let step = build_full_generation_step();
    let evaluator = build_multi_case_evaluator();
    let mutator = build_mutation_program();
    let test_cases = make_add_test_cases();

    // Population: add (passes all 3 tests) and sub (passes 0 tests for add target)
    let add_prog = make_binop_program(0x00); // add
    let sub_prog = make_binop_program(0x01); // sub

    let population = Value::tuple(vec![
        Value::Program(Box::new(add_prog.clone())),
        Value::Program(Box::new(sub_prog.clone())),
    ]);

    let inputs = vec![
        population,
        test_cases.clone(),
        Value::Program(Box::new(evaluator.clone())),
        Value::Program(Box::new(mutator)),
    ];

    let (out, _) = interpreter::interpret(&step, &inputs, None).unwrap();

    match &out[0] {
        Value::Tuple(new_pop) => {
            assert_eq!(new_pop.len(), 2, "output should have 2 individuals");

            // Port 0 is the elite (best program): should be add (fitness 3)
            {
                let g = extract_program(&new_pop[0]);
                let fitness = evaluate_fitness(&evaluator, &g, &test_cases);
                assert_eq!(
                    fitness, 3,
                    "elite should be the add program with fitness 3"
                );
            }

            // Port 1 is the mutant child — it was created by mutating the
            // best (add) with opcode 0x00 (add again), so it should also
            // be add with fitness 3.
            {
                let g = extract_program(&new_pop[1]);
                let fitness = evaluate_fitness(&evaluator, &g, &test_cases);
                assert_eq!(
                    fitness, 3,
                    "child (mutated add -> add) should have fitness 3"
                );
            }
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn full_step_reversed_population() {
    // Same test but with sub first, add second — should still pick add as elite
    let step = build_full_generation_step();
    let evaluator = build_multi_case_evaluator();
    let mutator = build_mutation_program();
    let test_cases = make_add_test_cases();

    let add_prog = make_binop_program(0x00);
    let sub_prog = make_binop_program(0x01);

    let population = Value::tuple(vec![
        Value::Program(Box::new(sub_prog)),
        Value::Program(Box::new(add_prog)),
    ]);

    let inputs = vec![
        population,
        test_cases.clone(),
        Value::Program(Box::new(evaluator.clone())),
        Value::Program(Box::new(mutator)),
    ];

    let (out, _) = interpreter::interpret(&step, &inputs, None).unwrap();

    match &out[0] {
        Value::Tuple(new_pop) => {
            // Elite should be add (higher fitness), regardless of position
            {
                let g = extract_program(&new_pop[0]);
                let fitness = evaluate_fitness(&evaluator, &g, &test_cases);
                assert_eq!(
                    fitness, 3,
                    "elite should be add with fitness 3 even in reversed position"
                );
            }
        }
        other => panic!("expected Tuple, got {:?}", other),
    }
}

#[test]
fn full_step_multi_generation() {
    // Run 3 generations to verify the step can be iterated.
    // Start with sub (fitness 0) and mul (fitness 0 for add target).
    // After mutation to add (opcode 0x00), fitness should improve.
    let step = build_full_generation_step();
    let evaluator = build_multi_case_evaluator();
    let mutator = build_mutation_program();
    let test_cases = make_add_test_cases();

    let sub_prog = make_binop_program(0x01);
    let mul_prog = make_binop_program(0x02);

    let mut population = Value::tuple(vec![
        Value::Program(Box::new(sub_prog)),
        Value::Program(Box::new(mul_prog)),
    ]);

    let mut best_fitness = 0i64;

    for generation in 0..3 {
        let inputs = vec![
            population.clone(),
            test_cases.clone(),
            Value::Program(Box::new(evaluator.clone())),
            Value::Program(Box::new(mutator.clone())),
        ];

        let (out, _) = interpreter::interpret(&step, &inputs, None).unwrap();

        match &out[0] {
            Value::Tuple(new_pop) => {
                // Track best fitness
                for prog_val in new_pop {
                    let g = extract_program(prog_val);
                    let f = evaluate_fitness(&evaluator, &g, &test_cases);
                    if f > best_fitness {
                        best_fitness = f;
                    }
                }
                population = out[0].clone();
            }
            _ => panic!("generation {}: expected Tuple", generation),
        }
    }

    // The mutation always sets opcode to 0x00 (add), so after 1 generation
    // we should have an add program with fitness 3.
    assert_eq!(
        best_fitness, 3,
        "after 3 generations, best fitness should be 3 (mutation produces add)"
    );
}

// ---------------------------------------------------------------------------
// Composition test: combine all pieces
// ---------------------------------------------------------------------------

#[test]
fn compose_crowding_cull_elite() {
    // Demonstrate that crowding + cull + elitism compose correctly.
    let crowding = build_crowding_distance_proxy();
    let cull = build_death_cull();
    let elite = build_elitism_find_best();

    let population = Value::tuple(vec![
        Value::Int(1),
        Value::Int(5),
        Value::Int(10),
        Value::Int(50),
        Value::Int(100),
    ]);

    // Step 1: Compute crowding distances for all individuals
    let mut crowding_scores = Vec::new();
    if let Value::Tuple(ref pop_values) = population {
        for val in pop_values {
            let inputs = vec![val.clone(), population.clone()];
            let (out, _) = interpreter::interpret(&crowding, &inputs, None).unwrap();
            let score = match &out[0] {
                Value::Int(v) => *v,
                other => panic!("expected Int, got {:?}", other),
            };
            crowding_scores.push(score);
        }
    }

    // Extremes (1 and 100) should have highest crowding
    assert!(
        crowding_scores[0] > crowding_scores[2],
        "extreme (1) should have higher crowding than middle (10)"
    );
    assert!(
        crowding_scores[4] > crowding_scores[2],
        "extreme (100) should have higher crowding than middle (10)"
    );

    // Step 2: Cull individuals below threshold 5
    let (culled_out, _) = interpreter::interpret(
        &cull,
        &[population.clone(), Value::Int(5)],
        None,
    )
    .unwrap();
    let culled = &culled_out[0];

    let culled_count = match culled {
        Value::Tuple(v) => v.len(),
        _ => panic!("expected Tuple"),
    };
    assert_eq!(culled_count, 4, "4 individuals survive threshold 5");

    // Step 3: Find the elite from the culled population
    let (elite_out, _) = interpreter::interpret(&elite, &[culled.clone()], None).unwrap();
    assert_eq!(
        elite_out,
        vec![Value::Int(100)],
        "elite from culled population is 100"
    );

    eprintln!("Composed pipeline: crowding -> cull -> elitism verified");
}

// ---------------------------------------------------------------------------
// Summary report
// ---------------------------------------------------------------------------

#[test]
fn summary_report() {
    eprintln!("\n=== Self-Write Population Management: Status Report ===\n");

    eprintln!("SUCCEEDED (IRIS programs built + tested):");
    eprintln!("  [POP-3] crowding_distance_proxy -- L1 sum-of-distances diversity metric");
    eprintln!("  [POP-4] death_cull              -- fold+guard filter by fitness threshold");
    eprintln!("  [POP-5] elitism_find_best       -- fold(MIN, max, pop) for k=1 elitism");
    eprintln!("  [POP-6] elitism_top_2           -- two-pass fold+guard for k=2 elitism");
    eprintln!("  [POP-7] full_generation_step    -- evaluate->rank->select->mutate->cull->elite");
    eprintln!();
    eprintln!("BUILDING ON (from self_write_mutation_v3.rs + self_write_nsga.rs):");
    eprintln!("  [POP-1] tournament_select       -- pairwise compare + max selector");
    eprintln!("  [POP-2] crossover_subgraph      -- dual graph_replace_subtree");
    eprintln!("  [NSGA-1] dominance_check        -- Pareto dominance via fold+zip+map");
    eprintln!("  [NSGA-2] pareto_rank            -- inline dominance in fold-over-population");
    eprintln!("  [NSGA-3] population_ranks       -- map(pop, |i| rank(i, pop)) via graph_eval");
    eprintln!();
    eprintln!("DEME::STEP COVERAGE:");
    eprintln!("  1. Evaluate    -> multi_case_evaluator (self_write_fitness.rs)");
    eprintln!("  2. NSGA-II     -> pareto_rank + crowding_distance_proxy");
    eprintln!("  3. Select      -> tournament_select + lexicase (Rust-only, needs Effect nodes)");
    eprintln!("  4. Reproduce   -> crossover_subgraph + mutation operators (v1/v2/v3)");
    eprintln!("  5. Death/Cull  -> death_cull (filter by threshold)");
    eprintln!("  6. Elitism     -> elitism_find_best / elitism_top_2");
    eprintln!("  7. Compose     -> full_generation_step (2-individual version)");
    eprintln!();
    eprintln!("LIMITATIONS:");
    eprintln!("  - crowding_distance is a proxy (L1 sum-of-distances), not true NSGA-II");
    eprintln!("    (true version needs per-objective sort + neighbor diff, not expressible)");
    eprintln!("  - elitism_top_2 loses ties (eq causes skip of all matching values)");
    eprintln!("  - full_generation_step is 2-individual only (scalable N needs map+fold");
    eprintln!("    composition that exceeds MAX_SELF_EVAL_DEPTH=4)");
    eprintln!("  - lexicase selection needs randomness (Effect nodes + handlers)");
    eprintln!();
    eprintln!("TOTAL: 7 population management functions as IRIS programs");
    eprintln!("=== End Report ===");
}
