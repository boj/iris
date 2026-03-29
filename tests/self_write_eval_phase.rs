
//! Self-write: evaluator completion and phase detection as IRIS programs.
//!
//! **Evaluator** — builds on self_write_fitness.rs (single-case at 1.5x,
//! multi-case at 2.9x) by adding:
//!   1. `compute_correctness`: IRIS program that compares a program's outputs
//!      to expected outputs and returns a [0.0, 1.0] score.
//!   2. `evaluate`: IRIS program that runs a program on all test cases, then
//!      computes the aggregate correctness score (passed / total).
//!
//! **Phase detection** — PhaseDetector logic as IRIS:
//!   1. `improvement_tracker`: fold over fitness history, compute rate of
//!      change (sum of improvements / window size).
//!   2. `phase_classifier`: if improvement > threshold → Exploration (Int 0),
//!      if < low_threshold AND diversity < 0.3 → Exploitation (Int 2),
//!      else SteadyState (Int 1).

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    BinderId, Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
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

// ---------------------------------------------------------------------------
// Target program: add(input0, input1)
// ---------------------------------------------------------------------------

fn make_add_program() -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = prim_node(1, 0x00, 2);
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

// ---------------------------------------------------------------------------
// 1. compute_correctness — IRIS program
// ---------------------------------------------------------------------------
//
// Takes 2 inputs:
//   inputs[0] = Value::Int(actual_output)
//   inputs[1] = Value::Int(expected_output)
//
// Returns Float64 score in [0.0, 1.0]:
//   if actual == expected → 1.0
//   else → 1.0 / (1.0 + |actual - expected| / max(1, |expected|))^2
//
// Graph structure (using Guard for the if/else):
//
//   Root(id=1): Guard
//     predicate: eq(input0, input1)                   [id=10]
//     body: float_lit(1.0)                            [id=20]
//     fallback: div(1.0, add(1.0, relative_error²))   [id=30..90]
//
// Where relative_error = |actual - expected| / max(1, |expected|)
// and we approximate (1/(1+e))^2 as 1/(1+e)^2 = 1/((1+e)*(1+e))

fn build_compute_correctness() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // --- Predicate: eq(input0, input1) ---
    let (nid, node) = prim_node(10, 0x20, 2);  // eq
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(11, 0);   // actual
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(12, 1);   // expected
    nodes.insert(nid, node);

    // --- Body: Float64(1.0) ---
    let (nid, node) = float_lit_node(20, 1.0);
    nodes.insert(nid, node);

    // --- Fallback: compute partial score ---
    //
    // We need: 1.0 / (1.0 + rel_err)^2
    //
    // Step 1: diff = sub(input0, input1)               [id=40]
    // Step 2: abs_diff = abs(diff)                     [id=41]
    // Step 3: abs_diff_f = int_to_float(abs_diff)      [id=42]
    // Step 4: abs_exp = abs(input1)                    [id=43]
    // Step 5: max_scale = max(abs_exp, 1)              [id=44]
    // Step 6: max_scale_f = int_to_float(max_scale)    [id=45]
    // Step 7: rel_err = div(abs_diff_f, max_scale_f)   [id=46]
    // Step 8: one_plus = add(1.0, rel_err)             [id=47]
    // Step 9: squared = mul(one_plus, one_plus)        [id=48, duplicate ref via 49]
    // Step 10: result = div(1.0, squared)              [id=30]

    // diff = sub(input0, input1)
    let (nid, node) = prim_node(40, 0x01, 2);  // sub
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(40_1, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(40_2, 1);
    nodes.insert(nid, node);

    // abs_diff = abs(diff)
    let (nid, node) = prim_node(41, 0x06, 1);  // abs
    nodes.insert(nid, node);

    // abs_diff_f = int_to_float(abs_diff)
    let (nid, node) = prim_node(42, 0x40, 1);  // int_to_float
    nodes.insert(nid, node);

    // abs_exp = abs(input1)
    let (nid, node) = prim_node(43, 0x06, 1);  // abs
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(43_1, 1);
    nodes.insert(nid, node);

    // max_scale = max(abs_exp, 1)
    let (nid, node) = prim_node(44, 0x08, 2);  // max
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(44_1, 1);
    nodes.insert(nid, node);

    // max_scale_f = int_to_float(max_scale)
    let (nid, node) = prim_node(45, 0x40, 1);  // int_to_float
    nodes.insert(nid, node);

    // rel_err = div(abs_diff_f, max_scale_f)
    let (nid, node) = prim_node(46, 0x03, 2);  // div
    nodes.insert(nid, node);

    // one_plus = add(1.0, rel_err)
    let (nid, node) = prim_node(47, 0x00, 2);  // add
    nodes.insert(nid, node);
    let (nid, node) = float_lit_node(47_1, 1.0);
    nodes.insert(nid, node);

    // squared = mul(one_plus, one_plus)
    // We need to reference node 47's result twice. We use two separate
    // references to the same computation chain. Since IRIS is a DAG, we
    // can have two edges pointing to the same target node.
    let (nid, node) = prim_node(48, 0x02, 2);  // mul
    nodes.insert(nid, node);

    // result = div(1.0, squared)
    let (nid, node) = prim_node(30, 0x03, 2);  // div
    nodes.insert(nid, node);
    let (nid, node) = float_lit_node(30_1, 1.0);
    nodes.insert(nid, node);

    // --- Root: Guard(predicate=10, body=20, fallback=30) ---
    let (nid, node) = guard_node(1, 10, 20, 30);
    nodes.insert(nid, node);

    // --- Edges ---
    let edges = vec![
        // eq(input0, input1)
        make_edge(10, 11, 0, EdgeLabel::Argument),
        make_edge(10, 12, 1, EdgeLabel::Argument),
        // sub(input0, input1)
        make_edge(40, 40_1, 0, EdgeLabel::Argument),
        make_edge(40, 40_2, 1, EdgeLabel::Argument),
        // abs(diff)
        make_edge(41, 40, 0, EdgeLabel::Argument),
        // int_to_float(abs_diff)
        make_edge(42, 41, 0, EdgeLabel::Argument),
        // abs(input1)
        make_edge(43, 43_1, 0, EdgeLabel::Argument),
        // max(abs_exp, 1)
        make_edge(44, 43, 0, EdgeLabel::Argument),
        make_edge(44, 44_1, 1, EdgeLabel::Argument),
        // int_to_float(max_scale)
        make_edge(45, 44, 0, EdgeLabel::Argument),
        // div(abs_diff_f, max_scale_f)
        make_edge(46, 42, 0, EdgeLabel::Argument),
        make_edge(46, 45, 1, EdgeLabel::Argument),
        // add(1.0, rel_err)
        make_edge(47, 47_1, 0, EdgeLabel::Argument),
        make_edge(47, 46, 1, EdgeLabel::Argument),
        // mul(one_plus, one_plus) — DAG: two edges to node 47
        make_edge(48, 47, 0, EdgeLabel::Argument),
        make_edge(48, 47, 1, EdgeLabel::Argument),
        // div(1.0, squared)
        make_edge(30, 30_1, 0, EdgeLabel::Argument),
        make_edge(30, 48, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// 2. evaluate — full IRIS evaluator program
// ---------------------------------------------------------------------------
//
// Takes 2 inputs:
//   inputs[0] = Value::Program(target_program)
//   inputs[1] = Value::Tuple(test_cases)
//                where each test_case = Tuple(Tuple(inputs...), Int(expected))
//
// Returns Float64: correctness score = passed_count / total_count.
//
// Implementation: fold over test cases to count passes (using the multi-case
// fold from self_write_fitness), then divide by total count.
//
// Structure:
//   Root(id=1): div(float(passed_count), float(total_count))
//   ├── int_to_float(passed_count)                      [id=2]
//   │   └── Fold(mode=0x00) over test_cases             [id=100..195] (same as multi-case)
//   └── int_to_float(total_count)                       [id=3]
//       └── Fold(mode=0x05) over test_cases             [id=200..210] (count mode)

fn build_evaluate() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: div(float_passed, float_total)
    let (nid, node) = prim_node(1, 0x03, 2);  // div
    nodes.insert(nid, node);

    // int_to_float(passed_count)
    let (nid, node) = prim_node(2, 0x40, 1);  // int_to_float
    nodes.insert(nid, node);

    // int_to_float(total_count)
    let (nid, node) = prim_node(3, 0x40, 1);  // int_to_float
    nodes.insert(nid, node);

    // --- passed_count: Fold(mode=0x00) counting passing test cases ---
    // This is the multi-case evaluator body from self_write_fitness.

    // Fold root
    let (nid, node) = make_node(
        100,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![0x00],
        },
        3,
    );
    nodes.insert(nid, node);

    // Base: Int(0)
    let (nid, node) = int_lit_node(101, 0);
    nodes.insert(nid, node);

    // Step: Lambda(binder=0xFFFF_0002)
    let (nid, node) = make_node(
        102,
        NodeKind::Lambda,
        NodePayload::Lambda {
            binder: BinderId(0xFFFF_0002),
            captured_count: 0,
        },
        0,
    );
    nodes.insert(nid, node);

    // Collection: input_ref(1) — test_cases
    let (nid, node) = input_ref_node(103, 1);
    nodes.insert(nid, node);

    // Lambda body: add(acc, bool_to_int(eq(graph_eval(program, tc_inputs), tc_expected)))
    // add(acc, score)
    let (nid, node) = prim_node(110, 0x00, 2);  // add
    nodes.insert(nid, node);

    // Project(0) from input_ref(2) → acc
    let (nid, node) = project_node(111, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(112, 2);
    nodes.insert(nid, node);

    // bool_to_int
    let (nid, node) = prim_node(120, 0x44, 1);
    nodes.insert(nid, node);

    // eq(graph_eval_result, expected)
    let (nid, node) = prim_node(130, 0x20, 2);  // eq
    nodes.insert(nid, node);

    // graph_eval(program, tc_inputs)
    let (nid, node) = prim_node(140, 0x89, 2);  // graph_eval
    nodes.insert(nid, node);

    // input_ref(0) — the program
    let (nid, node) = input_ref_node(150, 0);
    nodes.insert(nid, node);

    // Project(0) from tc — tc_inputs
    let (nid, node) = project_node(160, 0);
    nodes.insert(nid, node);

    // Project(1) from input_ref(2) — tc from Tuple(acc, tc)
    let (nid, node) = project_node(170, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(171, 2);
    nodes.insert(nid, node);

    // Project(1) from tc — tc_expected
    let (nid, node) = project_node(180, 1);
    nodes.insert(nid, node);

    // Project(1) from input_ref(2) — tc from Tuple(acc, tc)
    let (nid, node) = project_node(190, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(191, 2);
    nodes.insert(nid, node);

    // --- total_count: Fold(mode=0x05) = count elements ---
    let (nid, node) = make_node(
        200,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![0x05],
        },
        3,
    );
    nodes.insert(nid, node);

    // Base for count fold: Int(0)
    let (nid, node) = int_lit_node(201, 0);
    nodes.insert(nid, node);

    // Step for count fold: doesn't matter for mode 0x05 (count ignores it),
    // but we need a valid node. Use a Lit(0).
    let (nid, node) = int_lit_node(202, 0);
    nodes.insert(nid, node);

    // Collection for count fold: input_ref(1)
    let (nid, node) = input_ref_node(203, 1);
    nodes.insert(nid, node);

    // --- Edges ---
    let edges = vec![
        // Root: div(int_to_float(passed), int_to_float(total))
        make_edge(1, 2, 0, EdgeLabel::Argument),
        make_edge(1, 3, 1, EdgeLabel::Argument),
        // int_to_float(passed_count)
        make_edge(2, 100, 0, EdgeLabel::Argument),
        // int_to_float(total_count)
        make_edge(3, 200, 0, EdgeLabel::Argument),
        // Fold ports for passed_count
        make_edge(100, 101, 0, EdgeLabel::Argument),  // base
        make_edge(100, 102, 1, EdgeLabel::Argument),  // step (Lambda)
        make_edge(100, 103, 2, EdgeLabel::Argument),  // collection
        // Lambda body via Continuation
        Edge {
            source: NodeId(102),
            target: NodeId(110),
            port: 0,
            label: EdgeLabel::Continuation,
        },
        // add(acc, score)
        make_edge(110, 111, 0, EdgeLabel::Argument),  // acc
        make_edge(110, 120, 1, EdgeLabel::Argument),  // score
        // Project(0) from input_ref(2) → acc
        make_edge(111, 112, 0, EdgeLabel::Argument),
        // bool_to_int → eq
        make_edge(120, 130, 0, EdgeLabel::Argument),
        // eq(graph_eval_result, expected)
        make_edge(130, 140, 0, EdgeLabel::Argument),
        make_edge(130, 180, 1, EdgeLabel::Argument),
        // graph_eval(program, tc_inputs)
        make_edge(140, 150, 0, EdgeLabel::Argument),
        make_edge(140, 160, 1, EdgeLabel::Argument),
        // Project(0) from Project(1) from input_ref(2) → tc.inputs
        make_edge(160, 170, 0, EdgeLabel::Argument),
        make_edge(170, 171, 0, EdgeLabel::Argument),
        // Project(1) from Project(1) from input_ref(2) → tc.expected
        make_edge(180, 190, 0, EdgeLabel::Argument),
        make_edge(190, 191, 0, EdgeLabel::Argument),
        // Fold ports for total_count
        make_edge(200, 201, 0, EdgeLabel::Argument),  // base
        make_edge(200, 202, 1, EdgeLabel::Argument),  // step
        make_edge(200, 203, 2, EdgeLabel::Argument),  // collection
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// 3. improvement_tracker — IRIS program
// ---------------------------------------------------------------------------
//
// Takes 1 input:
//   inputs[0] = Value::Tuple(fitness_history)  — list of Float64 values
//
// Computes improvement rate: fold over adjacent pairs, sum improvements,
// divide by count.
//
// Implementation: since we need adjacent-pair differences, we use two
// nested operations:
//   1. Fold in mode 0x00 over the history with a Lambda that:
//      - Maintains acc = Tuple(prev_best, total_improvement, count)
//      - For each fitness value: improvement = max(0, current - prev_best)
//      - Updates: new_acc = Tuple(max(prev_best, current), total + improvement, count + 1)
//   2. Extract total_improvement / count from the final acc.
//
// Simplified approach: compute (last - first) / count for monotonic improvement,
// or better: sum all positive deltas and divide by window size.
//
// Even simpler: fold(add, 0, history) / fold(count, 0, history) gives
// the average fitness, not the improvement rate. We need differences.
//
// Simplest approach matching the Rust code: the Rust PhaseDetector just
// sums improvements and divides by window length. Each "improvement" is
// already pre-computed (max(0, current_best - prev_best)).
//
// So improvement_tracker takes a list of improvement deltas and returns
// their average: sum / count.
//
// fold(add, 0.0, history) / int_to_float(fold(count, 0, history))

fn build_improvement_tracker() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: div(sum, float_count)
    let (nid, node) = prim_node(1, 0x03, 2);  // div
    nodes.insert(nid, node);

    // sum = Fold(mode=0x00, base=0.0, step=add, collection=input(0))
    // Using direct Prim opcode in step: fold recognizes Prim node and uses
    // its opcode directly.
    let (nid, node) = make_node(
        10,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![0x00],
        },
        3,
    );
    nodes.insert(nid, node);

    // base = Float64(0.0)
    let (nid, node) = float_lit_node(11, 0.0);
    nodes.insert(nid, node);

    // step = Prim(add, 0x00) — the fold engine uses this opcode directly
    let (nid, node) = prim_node(12, 0x00, 2);
    nodes.insert(nid, node);

    // collection = input_ref(0)
    let (nid, node) = input_ref_node(13, 0);
    nodes.insert(nid, node);

    // float_count = int_to_float(count)
    let (nid, node) = prim_node(20, 0x40, 1);  // int_to_float
    nodes.insert(nid, node);

    // count = Fold(mode=0x05, base=0, step=ignored, collection=input(0))
    let (nid, node) = make_node(
        21,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![0x05],
        },
        3,
    );
    nodes.insert(nid, node);

    // base for count
    let (nid, node) = int_lit_node(22, 0);
    nodes.insert(nid, node);

    // step for count (ignored in mode 0x05)
    let (nid, node) = int_lit_node(23, 0);
    nodes.insert(nid, node);

    // collection for count
    let (nid, node) = input_ref_node(24, 0);
    nodes.insert(nid, node);

    let edges = vec![
        // div(sum, float_count)
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        // Fold for sum
        make_edge(10, 11, 0, EdgeLabel::Argument),  // base
        make_edge(10, 12, 1, EdgeLabel::Argument),  // step
        make_edge(10, 13, 2, EdgeLabel::Argument),  // collection
        // int_to_float(count)
        make_edge(20, 21, 0, EdgeLabel::Argument),
        // Fold for count
        make_edge(21, 22, 0, EdgeLabel::Argument),  // base
        make_edge(21, 23, 1, EdgeLabel::Argument),  // step
        make_edge(21, 24, 2, EdgeLabel::Argument),  // collection
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// 4. phase_classifier — IRIS program
// ---------------------------------------------------------------------------
//
// Takes 2 inputs:
//   inputs[0] = Value::Float64(improvement_rate)
//   inputs[1] = Value::Float64(diversity)
//
// Returns Int encoding the phase:
//   0 = Exploration   (improvement > 0.05)
//   2 = Exploitation  (improvement < 0.01 AND diversity < 0.3)
//   1 = SteadyState   (everything else)
//
// Uses nested Guards for the conditional chain:
//
//   Guard(improvement > 0.05,
//     body: Int(0),               → Exploration
//     fallback: Guard(improvement < 0.01 AND diversity < 0.3,
//       body: Int(2),             → Exploitation
//       fallback: Int(1)))        → SteadyState
//
// For the AND condition, we use a nested Guard:
//   Guard(improvement < 0.01,
//     body: Guard(diversity < 0.3,
//       body: Int(2),
//       fallback: Int(1)),
//     fallback: Int(1))

fn build_phase_classifier() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // --- Outer guard: improvement > 0.05 → Exploration ---
    // predicate: gt(input0, 0.05)
    let (nid, node) = prim_node(10, 0x23, 2);  // gt
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(11, 0);   // improvement_rate
    nodes.insert(nid, node);
    let (nid, node) = float_lit_node(12, 0.05);
    nodes.insert(nid, node);

    // body: Int(0) = Exploration
    let (nid, node) = int_lit_node(20, 0);
    nodes.insert(nid, node);

    // --- Inner guard 1: improvement < 0.01 ---
    let (nid, node) = prim_node(30, 0x22, 2);  // lt
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(31, 0);   // improvement_rate
    nodes.insert(nid, node);
    let (nid, node) = float_lit_node(32, 0.01);
    nodes.insert(nid, node);

    // --- Inner guard 2 (body of inner guard 1): diversity < 0.3 ---
    let (nid, node) = prim_node(40, 0x22, 2);  // lt
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(41, 1);   // diversity
    nodes.insert(nid, node);
    let (nid, node) = float_lit_node(42, 0.3);
    nodes.insert(nid, node);

    // body: Int(2) = Exploitation
    let (nid, node) = int_lit_node(50, 2);
    nodes.insert(nid, node);

    // fallback: Int(1) = SteadyState (used by both inner guards)
    let (nid, node) = int_lit_node(60, 1);
    nodes.insert(nid, node);

    // Another fallback Int(1) for inner guard 1's fallback
    let (nid, node) = int_lit_node(61, 1);
    nodes.insert(nid, node);

    // Inner guard 2: Guard(diversity < 0.3, Int(2), Int(1))
    let (nid, node) = guard_node(70, 40, 50, 60);
    nodes.insert(nid, node);

    // Inner guard 1: Guard(improvement < 0.01, inner_guard_2, Int(1))
    let (nid, node) = guard_node(80, 30, 70, 61);
    nodes.insert(nid, node);

    // Root: Guard(improvement > 0.05, Int(0), inner_guard_1)
    let (nid, node) = guard_node(1, 10, 20, 80);
    nodes.insert(nid, node);

    let edges = vec![
        // gt(input0, 0.05)
        make_edge(10, 11, 0, EdgeLabel::Argument),
        make_edge(10, 12, 1, EdgeLabel::Argument),
        // lt(input0, 0.01)
        make_edge(30, 31, 0, EdgeLabel::Argument),
        make_edge(30, 32, 1, EdgeLabel::Argument),
        // lt(input1, 0.3)
        make_edge(40, 41, 0, EdgeLabel::Argument),
        make_edge(40, 42, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// Tests
// ===========================================================================

// ---------------------------------------------------------------------------
// compute_correctness tests
// ---------------------------------------------------------------------------

#[test]
fn correctness_exact_match() {
    let graph = build_compute_correctness();
    let inputs = vec![Value::Int(42), Value::Int(42)];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    match &outputs[0] {
        Value::Float64(v) => assert!(
            (*v - 1.0).abs() < 1e-9,
            "exact match should score 1.0, got {}",
            v
        ),
        other => panic!("expected Float64, got {:?}", other),
    }
}

#[test]
fn correctness_close_value() {
    let graph = build_compute_correctness();
    // actual=10, expected=11 → relative_error = 1/11 ≈ 0.0909
    // score = 1 / (1 + 0.0909)^2 ≈ 1 / 1.19 ≈ 0.84
    let inputs = vec![Value::Int(10), Value::Int(11)];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    match &outputs[0] {
        Value::Float64(v) => {
            assert!(*v > 0.7 && *v < 1.0, "close value should score ~0.84, got {}", v);
        }
        other => panic!("expected Float64, got {:?}", other),
    }
}

#[test]
fn correctness_very_wrong() {
    let graph = build_compute_correctness();
    // actual=100, expected=1 → rel_error = 99/1 = 99
    // score = 1 / (1+99)^2 = 1/10000 = 0.0001
    let inputs = vec![Value::Int(100), Value::Int(1)];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    match &outputs[0] {
        Value::Float64(v) => {
            assert!(*v < 0.01, "very wrong value should score near 0, got {}", v);
        }
        other => panic!("expected Float64, got {:?}", other),
    }
}

#[test]
fn correctness_zero_expected() {
    // expected=0 → max(1, |0|) = 1, so scale = 1
    // actual=5 → rel_error = 5/1 = 5
    // score = 1/(1+5)^2 = 1/36 ≈ 0.028
    let graph = build_compute_correctness();
    let inputs = vec![Value::Int(5), Value::Int(0)];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    match &outputs[0] {
        Value::Float64(v) => {
            assert!(*v < 0.1 && *v > 0.0, "wrong with 0 expected, got {}", v);
        }
        other => panic!("expected Float64, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// evaluate tests
// ---------------------------------------------------------------------------

#[test]
fn evaluate_all_correct() {
    let graph = build_evaluate();
    let add_program = make_add_program();

    let test_cases = Value::tuple(vec![
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(3), Value::Int(5)]),
            Value::Int(8),
        ]),
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(10), Value::Int(20)]),
            Value::Int(30),
        ]),
    ]);

    let inputs = vec![Value::Program(Rc::new(add_program)), test_cases];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    match &outputs[0] {
        Value::Float64(v) => {
            assert!(
                (*v - 1.0).abs() < 1e-9,
                "all correct should score 1.0, got {}",
                v
            );
        }
        other => panic!("expected Float64, got {:?}", other),
    }
}

#[test]
fn evaluate_partial_correct() {
    let graph = build_evaluate();
    let add_program = make_add_program();

    // 2 out of 4 correct → 0.5
    let test_cases = Value::tuple(vec![
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(1), Value::Int(2)]),
            Value::Int(3),   // correct
        ]),
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(5), Value::Int(5)]),
            Value::Int(99),  // wrong
        ]),
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(0), Value::Int(0)]),
            Value::Int(0),   // correct
        ]),
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(7), Value::Int(3)]),
            Value::Int(99),  // wrong
        ]),
    ]);

    let inputs = vec![Value::Program(Rc::new(add_program)), test_cases];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    match &outputs[0] {
        Value::Float64(v) => {
            assert!(
                (*v - 0.5).abs() < 1e-9,
                "2/4 correct should score 0.5, got {}",
                v
            );
        }
        other => panic!("expected Float64, got {:?}", other),
    }
}

#[test]
fn evaluate_none_correct() {
    let graph = build_evaluate();
    let add_program = make_add_program();

    let test_cases = Value::tuple(vec![
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(1), Value::Int(1)]),
            Value::Int(99),  // wrong: 1+1=2
        ]),
        Value::tuple(vec![
            Value::tuple(vec![Value::Int(2), Value::Int(3)]),
            Value::Int(99),  // wrong: 2+3=5
        ]),
    ]);

    let inputs = vec![Value::Program(Rc::new(add_program)), test_cases];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    match &outputs[0] {
        Value::Float64(v) => {
            assert!(
                (*v).abs() < 1e-9,
                "0/2 correct should score 0.0, got {}",
                v
            );
        }
        other => panic!("expected Float64, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// improvement_tracker tests
// ---------------------------------------------------------------------------

#[test]
fn improvement_tracker_uniform() {
    // All improvements are 0.1 → average = 0.1
    let graph = build_improvement_tracker();
    let history = Value::tuple(vec![
        Value::Float64(0.1),
        Value::Float64(0.1),
        Value::Float64(0.1),
        Value::Float64(0.1),
        Value::Float64(0.1),
    ]);
    let inputs = vec![history];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    match &outputs[0] {
        Value::Float64(v) => {
            assert!(
                (*v - 0.1).abs() < 1e-9,
                "uniform 0.1 improvements → average 0.1, got {}",
                v
            );
        }
        other => panic!("expected Float64, got {:?}", other),
    }
}

#[test]
fn improvement_tracker_declining() {
    // Improvements: 0.5, 0.3, 0.1, 0.0 → sum=0.9, count=4, avg=0.225
    let graph = build_improvement_tracker();
    let history = Value::tuple(vec![
        Value::Float64(0.5),
        Value::Float64(0.3),
        Value::Float64(0.1),
        Value::Float64(0.0),
    ]);
    let inputs = vec![history];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    match &outputs[0] {
        Value::Float64(v) => {
            assert!(
                (*v - 0.225).abs() < 1e-9,
                "declining improvements → avg 0.225, got {}",
                v
            );
        }
        other => panic!("expected Float64, got {:?}", other),
    }
}

#[test]
fn improvement_tracker_zero() {
    // No improvement at all: all zeros → average = 0.0
    let graph = build_improvement_tracker();
    let history = Value::tuple(vec![
        Value::Float64(0.0),
        Value::Float64(0.0),
        Value::Float64(0.0),
    ]);
    let inputs = vec![history];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    match &outputs[0] {
        Value::Float64(v) => {
            assert!(
                (*v).abs() < 1e-9,
                "zero improvements → avg 0.0, got {}",
                v
            );
        }
        other => panic!("expected Float64, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// phase_classifier tests
// ---------------------------------------------------------------------------

#[test]
fn phase_high_improvement_is_exploration() {
    let graph = build_phase_classifier();
    // improvement=0.1 > 0.05 → Exploration (0)
    let inputs = vec![Value::Float64(0.1), Value::Float64(0.5)];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    assert_eq!(outputs[0], Value::Int(0), "high improvement → Exploration");
}

#[test]
fn phase_low_improvement_low_diversity_is_exploitation() {
    let graph = build_phase_classifier();
    // improvement=0.005 < 0.01, diversity=0.1 < 0.3 → Exploitation (2)
    let inputs = vec![Value::Float64(0.005), Value::Float64(0.1)];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    assert_eq!(
        outputs[0],
        Value::Int(2),
        "low improvement + low diversity → Exploitation"
    );
}

#[test]
fn phase_low_improvement_high_diversity_is_steady_state() {
    let graph = build_phase_classifier();
    // improvement=0.005 < 0.01, diversity=0.5 > 0.3 → SteadyState (1)
    let inputs = vec![Value::Float64(0.005), Value::Float64(0.5)];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    assert_eq!(
        outputs[0],
        Value::Int(1),
        "low improvement + high diversity → SteadyState"
    );
}

#[test]
fn phase_moderate_improvement_is_steady_state() {
    let graph = build_phase_classifier();
    // improvement=0.03 (between 0.01 and 0.05) → SteadyState (1)
    let inputs = vec![Value::Float64(0.03), Value::Float64(0.5)];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    assert_eq!(
        outputs[0],
        Value::Int(1),
        "moderate improvement → SteadyState"
    );
}

#[test]
fn phase_boundary_improvement_exactly_at_threshold() {
    let graph = build_phase_classifier();
    // improvement=0.05 (not > 0.05, so falls through)
    // improvement < 0.01 is false, so SteadyState
    let inputs = vec![Value::Float64(0.05), Value::Float64(0.5)];
    let (outputs, _) = interpreter::interpret(&graph, &inputs, None).unwrap();
    assert_eq!(
        outputs[0],
        Value::Int(1),
        "improvement exactly 0.05 → SteadyState (not strictly greater)"
    );
}

// ---------------------------------------------------------------------------
// Integration: improvement_tracker feeds into phase_classifier
// ---------------------------------------------------------------------------

#[test]
fn tracker_into_classifier_exploration() {
    let tracker = build_improvement_tracker();
    let classifier = build_phase_classifier();

    // History with high improvements → average = 0.2 → Exploration
    let history = Value::tuple(vec![
        Value::Float64(0.3),
        Value::Float64(0.2),
        Value::Float64(0.1),
    ]);
    let (tracker_out, _) = interpreter::interpret(&tracker, &vec![history], None).unwrap();
    let improvement_rate = tracker_out[0].clone();

    let classifier_inputs = vec![improvement_rate, Value::Float64(0.5)];
    let (phase_out, _) = interpreter::interpret(&classifier, &classifier_inputs, None).unwrap();
    assert_eq!(
        phase_out[0],
        Value::Int(0),
        "high avg improvement → Exploration"
    );
}

#[test]
fn tracker_into_classifier_exploitation() {
    let tracker = build_improvement_tracker();
    let classifier = build_phase_classifier();

    // History with near-zero improvements → average ≈ 0.002 → Exploitation (if low diversity)
    let history = Value::tuple(vec![
        Value::Float64(0.001),
        Value::Float64(0.003),
        Value::Float64(0.002),
    ]);
    let (tracker_out, _) = interpreter::interpret(&tracker, &vec![history], None).unwrap();
    let improvement_rate = tracker_out[0].clone();

    let classifier_inputs = vec![improvement_rate, Value::Float64(0.1)]; // low diversity
    let (phase_out, _) = interpreter::interpret(&classifier, &classifier_inputs, None).unwrap();
    assert_eq!(
        phase_out[0],
        Value::Int(2),
        "near-zero improvement + low diversity → Exploitation"
    );
}

#[test]
fn tracker_into_classifier_steady_state() {
    let tracker = build_improvement_tracker();
    let classifier = build_phase_classifier();

    // Moderate improvements → average ≈ 0.03 → SteadyState
    let history = Value::tuple(vec![
        Value::Float64(0.04),
        Value::Float64(0.02),
        Value::Float64(0.03),
    ]);
    let (tracker_out, _) = interpreter::interpret(&tracker, &vec![history], None).unwrap();
    let improvement_rate = tracker_out[0].clone();

    let classifier_inputs = vec![improvement_rate, Value::Float64(0.5)];
    let (phase_out, _) = interpreter::interpret(&classifier, &classifier_inputs, None).unwrap();
    assert_eq!(
        phase_out[0],
        Value::Int(1),
        "moderate improvement → SteadyState"
    );
}
