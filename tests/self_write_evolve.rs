
//! Core evolution loop as an IRIS program.
//!
//! This test builds and runs the step closest to "IRIS runs itself": an IRIS
//! program that performs one generation of evolution. The loop composes the
//! self-written fitness evaluator and mutation program into a single
//! evaluate-select-mutate-replace cycle.
//!
//! The evolution loop takes a population (Tuple of Programs), test cases, and
//! sub-programs (fitness evaluator, mutator) as inputs. It evaluates each
//! candidate, selects the best, mutates it, and replaces the worst.
//!
//! To keep graph depth manageable (MAX_SELF_EVAL_DEPTH = 4), we use a
//! population of 2 programs and simple comparison rather than full NSGA-II.
//! The essential cycle is: evaluate -> select best -> mutate -> insert back.

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
// Graph construction helpers (shared with other self_write_* tests)
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
        0, // Guard has no argument edges; references are in the payload
    )
}

// ---------------------------------------------------------------------------
// Target programs: candidate individuals for evolution
// ---------------------------------------------------------------------------

/// Build a program that computes `input[0] op input[1]`.
/// Uses input_ref nodes for dynamic inputs.
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

// ---------------------------------------------------------------------------
// Multi-case fitness evaluator (from self_write_fitness.rs)
// ---------------------------------------------------------------------------

/// Multi-case evaluator: given a program and test cases, returns the count
/// of passing tests.
///
/// Takes 2 inputs:
///   inputs[0] = Value::Program(target_program)
///   inputs[1] = Value::Tuple(test_cases) where each is Tuple(Tuple(inputs...), Int(expected))
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

// ---------------------------------------------------------------------------
// Mutation program (from self_write_mutation.rs)
// ---------------------------------------------------------------------------

/// Build an IRIS mutation program that changes the root Prim node's opcode.
///
/// Takes 2 inputs:
///   inputs[0] = Value::Program(target_graph)
///   inputs[1] = Value::Int(new_opcode)
///
/// Returns the modified program.
///
/// Uses graph_get_root(0x8A) to find the root node ID, which is always the
/// Prim root for our binop programs. This is robust across mutations because
/// graph_set_prim_op updates the root pointer when the root node changes.
///
/// Graph structure:
///   Root(id=1): graph_set_prim_op(0x84, arity=3)
///   ├── port 0: input_ref(0)             → Program       [id=10]
///   ├── port 1: graph_get_root(0x8A)     → root node ID  [id=20]
///   │           └── input_ref(0)                          [id=30]
///   └── port 2: input_ref(1)             → new opcode    [id=40]
fn build_mutation_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: graph_set_prim_op (opcode 0x84, 3 args)
    let (nid, node) = prim_node(1, 0x84, 3);
    nodes.insert(nid, node);

    // Port 0: the Program input
    let (nid, node) = input_ref_node(10, 0);
    nodes.insert(nid, node);

    // Port 1: graph_get_root(0x8A) to find the root node ID
    let (nid, node) = prim_node(20, 0x8A, 1);
    nodes.insert(nid, node);

    // input_ref(0) for graph_get_root arg
    let (nid, node) = input_ref_node(30, 0);
    nodes.insert(nid, node);

    // Port 2: the new opcode input
    let (nid, node) = input_ref_node(40, 1);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),  // program
        make_edge(1, 20, 1, EdgeLabel::Argument),  // root node ID
        make_edge(1, 40, 2, EdgeLabel::Argument),  // new opcode
        make_edge(20, 30, 0, EdgeLabel::Argument),  // graph_get_root(program)
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Evolution loop: one generation (IRIS program)
// ---------------------------------------------------------------------------
//
// Takes 4 inputs:
//   inputs[0] = Value::Tuple(programs)          — population (2 Programs)
//   inputs[1] = Value::Tuple(test_cases)        — test case pairs
//   inputs[2] = Value::Program(fitness_eval)    — multi-case evaluator
//   inputs[3] = Value::Program(mutator)         — mutation program
//
// Steps:
//   1. Evaluate fitness of program[0] and program[1]
//   2. Compare: if score0 >= score1, best=prog0, worst=prog1
//                                     else best=prog1, worst=prog0
//   3. Mutate best: change opcode to 0x00 (add) — a fixed mutation for now
//   4. Return Tuple(best, mutated_best) — replacing worst with mutant
//
// Graph structure:
//
//   Root(id=1): Tuple(arity=2)                  — new population
//   ├── port 0: Guard                           — best program (unchanged)
//   │   ├── predicate: ge(score0, score1)
//   │   ├── body: Project(0) from programs      — prog0
//   │   └── fallback: Project(1) from programs  — prog1
//   └── port 1: graph_eval(mutator, Tuple(best_prog, Int(0x00)))
//       ├── best_prog from Guard (same as port 0)
//       └── new opcode = Int(0x00) (add)
//
// Where score0 = graph_eval(fitness_eval, Tuple(prog0, test_cases))
//       score1 = graph_eval(fitness_eval, Tuple(prog1, test_cases))

fn build_evolution_step() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // =====================================================================
    // Score computation nodes
    // =====================================================================

    // score0 = graph_eval(fitness_eval, Tuple(prog0, test_cases))
    // id=2000: graph_eval(0x89, arity=2)
    let (nid, node) = prim_node(2000, 0x89, 2);
    nodes.insert(nid, node);
    // id=2010: input_ref(2) -> fitness_eval program
    let (nid, node) = input_ref_node(2010, 2);
    nodes.insert(nid, node);
    // id=2020: Tuple(prog0, test_cases)
    let (nid, node) = tuple_node(2020, 2);
    nodes.insert(nid, node);
    // id=2030: Project(0) from input_ref(0) -> prog0
    let (nid, node) = project_node(2030, 0);
    nodes.insert(nid, node);
    // id=2035: input_ref(0) -> programs tuple
    let (nid, node) = input_ref_node(2035, 0);
    nodes.insert(nid, node);
    // id=2040: input_ref(1) -> test_cases
    let (nid, node) = input_ref_node(2040, 1);
    nodes.insert(nid, node);

    // score1 = graph_eval(fitness_eval, Tuple(prog1, test_cases))
    // id=2100: graph_eval(0x89, arity=2)
    let (nid, node) = prim_node(2100, 0x89, 2);
    nodes.insert(nid, node);
    // id=2110: input_ref(2) -> fitness_eval program
    let (nid, node) = input_ref_node(2110, 2);
    nodes.insert(nid, node);
    // id=2120: Tuple(prog1, test_cases)
    let (nid, node) = tuple_node(2120, 2);
    nodes.insert(nid, node);
    // id=2130: Project(1) from input_ref(0) -> prog1
    let (nid, node) = project_node(2130, 1);
    nodes.insert(nid, node);
    // id=2135: input_ref(0) -> programs tuple
    let (nid, node) = input_ref_node(2135, 0);
    nodes.insert(nid, node);
    // id=2140: input_ref(1) -> test_cases
    let (nid, node) = input_ref_node(2140, 1);
    nodes.insert(nid, node);

    // =====================================================================
    // Predicate: ge(score0, score1) -> Bool
    // =====================================================================

    // id=3000: ge(0x25, arity=2)
    let (nid, node) = prim_node(3000, 0x25, 2);
    nodes.insert(nid, node);

    // =====================================================================
    // Guard: select best program
    // =====================================================================
    // If score0 >= score1, best = prog0, else best = prog1.

    // Body of guard (score0 >= score1 case): Project(0) from programs -> prog0
    // id=3100: Project(0) from input_ref(0)
    let (nid, node) = project_node(3100, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(3105, 0);
    nodes.insert(nid, node);

    // Fallback of guard (score0 < score1 case): Project(1) from programs -> prog1
    // id=3200: Project(1) from input_ref(0)
    let (nid, node) = project_node(3200, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(3205, 0);
    nodes.insert(nid, node);

    // id=3010: Guard node
    let (nid, node) = guard_node(3010, 3000, 3100, 3200);
    nodes.insert(nid, node);

    // =====================================================================
    // Guard: select worst program (opposite of best)
    // =====================================================================

    // Body (score0 >= score1 case): worst = prog1
    // id=3300: Project(1) from input_ref(0)
    let (nid, node) = project_node(3300, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(3305, 0);
    nodes.insert(nid, node);

    // Fallback (score0 < score1 case): worst = prog0
    // id=3400: Project(0) from input_ref(0)
    let (nid, node) = project_node(3400, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(3405, 0);
    nodes.insert(nid, node);

    // id=3020: Guard node for worst
    let (nid, node) = guard_node(3020, 3000, 3300, 3400);
    nodes.insert(nid, node);

    // =====================================================================
    // Mutate best: graph_eval(mutator, Tuple(best_prog, Int(0x00)))
    // =====================================================================
    // The mutation changes the first Prim node's opcode to 0x00 (add).
    // This is a fixed mutation for simplicity — in a full system, the
    // opcode would be chosen randomly or from a pool.

    // id=4000: graph_eval(0x89, arity=2)
    let (nid, node) = prim_node(4000, 0x89, 2);
    nodes.insert(nid, node);
    // id=4010: input_ref(3) -> mutator program
    let (nid, node) = input_ref_node(4010, 3);
    nodes.insert(nid, node);
    // id=4020: Tuple(best_prog, Int(0x00)) — mutation args
    let (nid, node) = tuple_node(4020, 2);
    nodes.insert(nid, node);

    // id=4030: Guard for best_prog (same logic as 3010 but separate subgraph)
    // Body: Project(0) from input_ref(0) -> prog0
    let (nid, node) = project_node(4100, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(4105, 0);
    nodes.insert(nid, node);
    // Fallback: Project(1) from input_ref(0) -> prog1
    let (nid, node) = project_node(4200, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(4205, 0);
    nodes.insert(nid, node);

    // Predicate for this guard: ge(score0, score1) — separate copy
    // We need separate score computations due to the DAG nature
    // Actually, IRIS graphs are DAGs — we CAN share nodes. Let's
    // reference the same predicate node 3000. But Guard payload
    // references nodes by ID, so we can reuse 3000.
    let (nid, node) = guard_node(4030, 3000, 4100, 4200);
    nodes.insert(nid, node);

    // id=4040: Int(0x00) — the new opcode (add)
    let (nid, node) = int_lit_node(4040, 0x00);
    nodes.insert(nid, node);

    // =====================================================================
    // Root: Tuple(best_unchanged, mutant)
    // =====================================================================
    // The new population keeps the best and replaces the worst with
    // the mutant.
    //
    // Actually, we want: Tuple(best, mutant_of_best)
    // The worst gets dropped, the best stays, and a mutant child
    // of the best takes the worst's slot.

    // id=1: Root Tuple(arity=2)
    let (nid, node) = tuple_node(1, 2);
    nodes.insert(nid, node);

    // =====================================================================
    // Edges
    // =====================================================================

    let edges = vec![
        // --- Root: Tuple(best, mutant) ---
        make_edge(1, 3010, 0, EdgeLabel::Argument),   // best program (Guard)
        make_edge(1, 4000, 1, EdgeLabel::Argument),   // mutant (graph_eval)

        // --- score0 = graph_eval(fitness_eval, Tuple(prog0, test_cases)) ---
        make_edge(2000, 2010, 0, EdgeLabel::Argument), // fitness_eval program
        make_edge(2000, 2020, 1, EdgeLabel::Argument), // args tuple
        make_edge(2020, 2030, 0, EdgeLabel::Argument), // prog0
        make_edge(2020, 2040, 1, EdgeLabel::Argument), // test_cases
        make_edge(2030, 2035, 0, EdgeLabel::Argument), // Project(0) from programs

        // --- score1 = graph_eval(fitness_eval, Tuple(prog1, test_cases)) ---
        make_edge(2100, 2110, 0, EdgeLabel::Argument), // fitness_eval program
        make_edge(2100, 2120, 1, EdgeLabel::Argument), // args tuple
        make_edge(2120, 2130, 0, EdgeLabel::Argument), // prog1
        make_edge(2120, 2140, 1, EdgeLabel::Argument), // test_cases
        make_edge(2130, 2135, 0, EdgeLabel::Argument), // Project(1) from programs

        // --- Predicate: ge(score0, score1) ---
        make_edge(3000, 2000, 0, EdgeLabel::Argument), // score0
        make_edge(3000, 2100, 1, EdgeLabel::Argument), // score1

        // --- Guard 3010 body: Project(0) from programs -> prog0 ---
        make_edge(3100, 3105, 0, EdgeLabel::Argument),

        // --- Guard 3010 fallback: Project(1) from programs -> prog1 ---
        make_edge(3200, 3205, 0, EdgeLabel::Argument),

        // --- Guard 3020 (worst) body: Project(1) from programs -> prog1 ---
        make_edge(3300, 3305, 0, EdgeLabel::Argument),

        // --- Guard 3020 (worst) fallback: Project(0) from programs -> prog0 ---
        make_edge(3400, 3405, 0, EdgeLabel::Argument),

        // --- Mutate best: graph_eval(mutator, Tuple(best_prog, 0x00)) ---
        make_edge(4000, 4010, 0, EdgeLabel::Argument), // mutator program
        make_edge(4000, 4020, 1, EdgeLabel::Argument), // args tuple
        make_edge(4020, 4030, 0, EdgeLabel::Argument), // best_prog (Guard)
        make_edge(4020, 4040, 1, EdgeLabel::Argument), // new opcode (Int(0x00))

        // --- Guard 4030 body: Project(0) from programs -> prog0 ---
        make_edge(4100, 4105, 0, EdgeLabel::Argument),

        // --- Guard 4030 fallback: Project(1) from programs -> prog1 ---
        make_edge(4200, 4205, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// Extended evolution step with variable mutation opcode
// ---------------------------------------------------------------------------
//
// Same as above but takes the mutation opcode as input[4] instead of
// hardcoding 0x00. This allows testing with different mutations per
// generation.
//
// Takes 5 inputs:
//   inputs[0] = Value::Tuple(programs)
//   inputs[1] = Value::Tuple(test_cases)
//   inputs[2] = Value::Program(fitness_eval)
//   inputs[3] = Value::Program(mutator)
//   inputs[4] = Value::Int(new_opcode)

fn build_evolution_step_variable() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Score computations (same structure as fixed version)

    // score0 = graph_eval(fitness_eval, Tuple(prog0, test_cases))
    let (nid, node) = prim_node(2000, 0x89, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(2010, 2);
    nodes.insert(nid, node);
    let (nid, node) = tuple_node(2020, 2);
    nodes.insert(nid, node);
    let (nid, node) = project_node(2030, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(2035, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(2040, 1);
    nodes.insert(nid, node);

    // score1 = graph_eval(fitness_eval, Tuple(prog1, test_cases))
    let (nid, node) = prim_node(2100, 0x89, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(2110, 2);
    nodes.insert(nid, node);
    let (nid, node) = tuple_node(2120, 2);
    nodes.insert(nid, node);
    let (nid, node) = project_node(2130, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(2135, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(2140, 1);
    nodes.insert(nid, node);

    // Predicate: ge(score0, score1)
    let (nid, node) = prim_node(3000, 0x25, 2);
    nodes.insert(nid, node);

    // Guard for best: if score0 >= score1 then prog0 else prog1
    let (nid, node) = project_node(3100, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(3105, 0);
    nodes.insert(nid, node);
    let (nid, node) = project_node(3200, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(3205, 0);
    nodes.insert(nid, node);
    let (nid, node) = guard_node(3010, 3000, 3100, 3200);
    nodes.insert(nid, node);

    // Mutation: graph_eval(mutator, Tuple(best_prog, new_opcode))
    let (nid, node) = prim_node(4000, 0x89, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(4010, 3);
    nodes.insert(nid, node);
    let (nid, node) = tuple_node(4020, 2);
    nodes.insert(nid, node);

    // best_prog for mutation (duplicate Guard)
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

    // new_opcode from input[4]
    let (nid, node) = input_ref_node(4040, 4);
    nodes.insert(nid, node);

    // Root: Tuple(best, mutant)
    let (nid, node) = tuple_node(1, 2);
    nodes.insert(nid, node);

    let edges = vec![
        // Root
        make_edge(1, 3010, 0, EdgeLabel::Argument),
        make_edge(1, 4000, 1, EdgeLabel::Argument),
        // score0
        make_edge(2000, 2010, 0, EdgeLabel::Argument),
        make_edge(2000, 2020, 1, EdgeLabel::Argument),
        make_edge(2020, 2030, 0, EdgeLabel::Argument),
        make_edge(2020, 2040, 1, EdgeLabel::Argument),
        make_edge(2030, 2035, 0, EdgeLabel::Argument),
        // score1
        make_edge(2100, 2110, 0, EdgeLabel::Argument),
        make_edge(2100, 2120, 1, EdgeLabel::Argument),
        make_edge(2120, 2130, 0, EdgeLabel::Argument),
        make_edge(2120, 2140, 1, EdgeLabel::Argument),
        make_edge(2130, 2135, 0, EdgeLabel::Argument),
        // predicate
        make_edge(3000, 2000, 0, EdgeLabel::Argument),
        make_edge(3000, 2100, 1, EdgeLabel::Argument),
        // Guard 3010 edges
        make_edge(3100, 3105, 0, EdgeLabel::Argument),
        make_edge(3200, 3205, 0, EdgeLabel::Argument),
        // Mutation
        make_edge(4000, 4010, 0, EdgeLabel::Argument),
        make_edge(4000, 4020, 1, EdgeLabel::Argument),
        make_edge(4020, 4030, 0, EdgeLabel::Argument),
        make_edge(4020, 4040, 1, EdgeLabel::Argument),
        // Guard 4030 edges
        make_edge(4100, 4105, 0, EdgeLabel::Argument),
        make_edge(4200, 4205, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// 3-program evolution step using Fold for evaluation + argmin/argmax
// ---------------------------------------------------------------------------
//
// Takes 5 inputs:
//   inputs[0] = Value::Tuple(programs)          — population (N programs)
//   inputs[1] = Value::Tuple(test_cases)        — test case pairs
//   inputs[2] = Value::Program(fitness_eval)    — evaluator
//   inputs[3] = Value::Program(mutator)         — mutation program
//   inputs[4] = Value::Int(new_opcode)          — opcode for mutation
//
// Strategy: Use map to evaluate all, then Fold to find best/worst indices,
// then reconstruct the population. This is the scalable N-program version.
//
// For simplicity in the IRIS graph, we use the Rust-side loop to run the
// 2-program evolution step iteratively on pairs (tournament-style).

// ---------------------------------------------------------------------------
// Helper: build test cases for addition (target: add(a,b) = a+b)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Helper: evaluate a program's fitness on test cases (Rust-side, for verification)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Helper: extract programs from a population tuple
// ---------------------------------------------------------------------------

fn extract_programs(population: &Value) -> Vec<SemanticGraph> {
    match population {
        Value::Tuple(elems) => elems
            .iter()
            .map(|v| match v {
                Value::Program(g) => g.as_ref().clone(),
                Value::Tuple(inner) => match inner.first() {
                    Some(Value::Program(g)) => g.as_ref().clone(),
                    _ => panic!("expected Program in population, got {:?}", v),
                },
                other => panic!("expected Program in population, got {:?}", other),
            })
            .collect(),
        other => panic!("expected Tuple population, got {:?}", other),
    }
}

// ===========================================================================
// Tests
// ===========================================================================

/// Verify the multi-case evaluator works correctly (precondition check).
#[test]
fn evaluator_precondition() {
    let evaluator = build_multi_case_evaluator();
    let test_cases = make_add_test_cases();

    // add(a,b) should pass all 3 test cases
    let add_prog = make_binop_program(0x00);
    let score = evaluate_fitness(&evaluator, &add_prog, &test_cases);
    assert_eq!(score, 3, "add should pass all 3 test cases");

    // sub(a,b) should pass 0 test cases
    let sub_prog = make_binop_program(0x01);
    let score = evaluate_fitness(&evaluator, &sub_prog, &test_cases);
    assert_eq!(score, 0, "sub should pass 0 test cases for add-target");

    // mul(a,b) should pass 0 test cases
    let mul_prog = make_binop_program(0x02);
    let score = evaluate_fitness(&evaluator, &mul_prog, &test_cases);
    assert_eq!(score, 0, "mul should pass 0 test cases for add-target");
}

/// Verify the mutation program works correctly (precondition check).
#[test]
fn mutation_precondition() {
    let mutator = build_mutation_program();
    let sub_prog = make_binop_program(0x01); // sub

    let inputs = vec![
        Value::Program(Box::new(sub_prog)),
        Value::Int(0x00), // mutate to add
    ];
    let (outputs, _) = interpreter::interpret(&mutator, &inputs, None).unwrap();

    let modified = match &outputs[0] {
        Value::Program(g) => g.as_ref().clone(),
        Value::Tuple(inner) => match inner.first() {
            Some(Value::Program(g)) => g.as_ref().clone(),
            _ => panic!("expected Program, got {:?}", &outputs[0]),
        },
        other => panic!("expected Program, got {:?}", other),
    };

    // The modified program should have an add node
    let has_add = modified
        .nodes
        .values()
        .any(|n| matches!(&n.payload, NodePayload::Prim { opcode: 0x00 }));
    assert!(has_add, "mutated program should have add opcode");
}

/// Test 1: One generation with 2 programs — best gets mutated, worst replaced.
///
/// Population: [sub(a,b), mul(a,b)]
/// Target: add(a,b) = a+b
/// Both score 0. With ge, sub (prog0) is selected as "best" (score0 >= score1).
/// Mutation changes sub -> add. New population: [sub, add].
#[test]
fn one_generation_two_programs() {
    let evo_step = build_evolution_step();
    let evaluator = build_multi_case_evaluator();
    let mutator = build_mutation_program();
    let test_cases = make_add_test_cases();

    // Initial population: sub and mul
    let sub_prog = make_binop_program(0x01);
    let mul_prog = make_binop_program(0x02);

    let population = Value::tuple(vec![
        Value::Program(Box::new(sub_prog.clone())),
        Value::Program(Box::new(mul_prog.clone())),
    ]);

    let inputs = vec![
        population,
        test_cases.clone(),
        Value::Program(Box::new(evaluator.clone())),
        Value::Program(Box::new(mutator.clone())),
    ];

    let (outputs, _) = interpreter::interpret(&evo_step, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1, "should return one value (the new population tuple)");

    let new_pop = &outputs[0];
    let programs = extract_programs(new_pop);
    assert_eq!(programs.len(), 2, "population should still have 2 programs");

    // The mutant should have opcode 0x00 (add).
    // One of the two programs in the new population should be an add program.
    let has_add = programs.iter().any(|p| {
        p.nodes.values().any(|n| {
            n.kind == NodeKind::Prim && matches!(&n.payload, NodePayload::Prim { opcode: 0x00 })
        })
    });
    assert!(has_add, "new population should contain a mutant with add opcode");
}

/// Test 2: After one generation, the population has changed.
///
/// Start with [sub, mul], target is add. After one generation, at least
/// one program should be different from the initial population.
#[test]
fn population_changes_after_generation() {
    let evo_step = build_evolution_step();
    let evaluator = build_multi_case_evaluator();
    let mutator = build_mutation_program();
    let test_cases = make_add_test_cases();

    let sub_prog = make_binop_program(0x01);
    let mul_prog = make_binop_program(0x02);

    let initial_population = Value::tuple(vec![
        Value::Program(Box::new(sub_prog.clone())),
        Value::Program(Box::new(mul_prog.clone())),
    ]);

    let inputs = vec![
        initial_population.clone(),
        test_cases.clone(),
        Value::Program(Box::new(evaluator.clone())),
        Value::Program(Box::new(mutator.clone())),
    ];

    let (outputs, _) = interpreter::interpret(&evo_step, &inputs, None).unwrap();
    let new_pop = &outputs[0];

    // The new population should differ from the initial one.
    // At least the mutant slot should have changed.
    let old_programs = extract_programs(&initial_population);
    let new_programs = extract_programs(new_pop);

    // Count how many opcodes changed
    let old_opcodes: Vec<Option<u8>> = old_programs
        .iter()
        .map(|p| {
            p.nodes
                .values()
                .find(|n| n.kind == NodeKind::Prim)
                .map(|n| match &n.payload {
                    NodePayload::Prim { opcode } => *opcode,
                    _ => 0xFF,
                })
        })
        .collect();

    let new_opcodes: Vec<Option<u8>> = new_programs
        .iter()
        .map(|p| {
            p.nodes
                .values()
                .find(|n| n.kind == NodeKind::Prim)
                .map(|n| match &n.payload {
                    NodePayload::Prim { opcode } => *opcode,
                    _ => 0xFF,
                })
        })
        .collect();

    assert_ne!(
        old_opcodes, new_opcodes,
        "population opcodes should change after one generation"
    );
}

/// Test 3: Run 3 generations in sequence. Fitness improves (or at least
/// doesn't crash). Demonstrates the evolution loop can be iterated.
///
/// We use the variable-opcode version to try different mutations each
/// generation. We always try opcode 0x00 (add), which is the target.
#[test]
fn three_generations_sequential() {
    let evo_step = build_evolution_step_variable();
    let evaluator = build_multi_case_evaluator();
    let mutator = build_mutation_program();
    let test_cases = make_add_test_cases();

    // Start with sub and mul (both score 0 on add tests)
    let sub_prog = make_binop_program(0x01);
    let mul_prog = make_binop_program(0x02);

    let mut population = Value::tuple(vec![
        Value::Program(Box::new(sub_prog)),
        Value::Program(Box::new(mul_prog)),
    ]);

    let mut scores: Vec<(i64, i64)> = Vec::new();

    for generation in 0..3 {
        // Record fitness before this generation
        let programs = extract_programs(&population);
        let s0 = evaluate_fitness(&evaluator, &programs[0], &test_cases);
        let s1 = evaluate_fitness(&evaluator, &programs[1], &test_cases);
        scores.push((s0, s1));

        // Choose mutation opcode: try add (0x00) since that's the target
        let new_opcode = 0x00i64;

        let inputs = vec![
            population.clone(),
            test_cases.clone(),
            Value::Program(Box::new(evaluator.clone())),
            Value::Program(Box::new(mutator.clone())),
            Value::Int(new_opcode),
        ];

        let (outputs, _) = interpreter::interpret(&evo_step, &inputs, None)
            .unwrap_or_else(|e| panic!("generation {} failed: {:?}", generation, e));
        population = outputs[0].clone();
    }

    // Record final fitness
    let final_programs = extract_programs(&population);
    let fs0 = evaluate_fitness(&evaluator, &final_programs[0], &test_cases);
    let fs1 = evaluate_fitness(&evaluator, &final_programs[1], &test_cases);
    scores.push((fs0, fs1));

    // The best fitness should have improved from generation 0 to the end.
    let initial_best = scores[0].0.max(scores[0].1);
    let final_best = fs0.max(fs1);

    assert!(
        final_best >= initial_best,
        "fitness should not decrease: initial best = {}, final best = {}",
        initial_best,
        final_best
    );

    // After 3 generations of trying add mutation, at least one program
    // should be add (scoring 3/3).
    assert_eq!(
        final_best, 3,
        "after 3 generations, the best program should score 3/3 (perfect add)"
    );
}

/// Test 4: Verify the correct program is selected as best.
///
/// Population: [add(a,b), sub(a,b)], target is add.
/// add scores 3, sub scores 0. add should be selected as best.
/// The mutant replaces sub, and the best (add) is kept.
#[test]
fn selects_correct_best() {
    let evo_step = build_evolution_step();
    let evaluator = build_multi_case_evaluator();
    let mutator = build_mutation_program();
    let test_cases = make_add_test_cases();

    let add_prog = make_binop_program(0x00); // perfect
    let sub_prog = make_binop_program(0x01); // zero fitness

    let population = Value::tuple(vec![
        Value::Program(Box::new(add_prog.clone())),
        Value::Program(Box::new(sub_prog.clone())),
    ]);

    let inputs = vec![
        population,
        test_cases.clone(),
        Value::Program(Box::new(evaluator.clone())),
        Value::Program(Box::new(mutator.clone())),
    ];

    let (outputs, _) = interpreter::interpret(&evo_step, &inputs, None).unwrap();
    let new_programs = extract_programs(&outputs[0]);

    // The best program (add, score=3) should be in the new population.
    // Since ge(3, 0) = true, prog0 (add) is selected as best.
    // The result is Tuple(best, mutant_of_best).
    // best = add (unchanged), mutant = add mutated to add (unchanged, since
    // mutation sets opcode to 0x00 which is already add).
    //
    // Both should be add programs.
    for (i, prog) in new_programs.iter().enumerate() {
        let has_add = prog.nodes.values().any(|n| {
            matches!(&n.payload, NodePayload::Prim { opcode: 0x00 })
        });
        assert!(
            has_add,
            "program {} in new population should be add (since best was add)",
            i
        );
    }
}

/// Test 5: Reverse order — best is at index 1.
///
/// Population: [sub(a,b), add(a,b)], target is add.
/// sub scores 0, add scores 3. add (prog1) should be selected as best.
#[test]
fn selects_best_at_index_one() {
    let evo_step = build_evolution_step();
    let evaluator = build_multi_case_evaluator();
    let mutator = build_mutation_program();
    let test_cases = make_add_test_cases();

    let sub_prog = make_binop_program(0x01);
    let add_prog = make_binop_program(0x00);

    let population = Value::tuple(vec![
        Value::Program(Box::new(sub_prog.clone())),
        Value::Program(Box::new(add_prog.clone())),
    ]);

    let inputs = vec![
        population,
        test_cases.clone(),
        Value::Program(Box::new(evaluator.clone())),
        Value::Program(Box::new(mutator.clone())),
    ];

    let (outputs, _) = interpreter::interpret(&evo_step, &inputs, None).unwrap();
    let new_programs = extract_programs(&outputs[0]);

    // ge(0, 3) = false, so prog1 (add) is selected as best.
    // Result should be Tuple(add, mutant_of_add).
    // Both should be add programs.
    for (i, prog) in new_programs.iter().enumerate() {
        let has_add = prog.nodes.values().any(|n| {
            matches!(&n.payload, NodePayload::Prim { opcode: 0x00 })
        });
        assert!(
            has_add,
            "program {} in new population should be add (best was prog1=add)",
            i
        );
    }
}

/// Test 6: Three-program evolution via pairwise tournament.
///
/// Run the 2-program evolution step as a building block for larger
/// populations. Process pairs of programs, keeping winners and their
/// mutants, to evolve a population of 3.
#[test]
fn three_program_pairwise_evolution() {
    let evo_step = build_evolution_step_variable();
    let evaluator = build_multi_case_evaluator();
    let mutator = build_mutation_program();
    let test_cases = make_add_test_cases();

    // Population of 3: sub, mul, div (all score 0 on add tests)
    let mut programs = vec![
        make_binop_program(0x01), // sub
        make_binop_program(0x02), // mul
        make_binop_program(0x03), // div
    ];

    // Run 3 rounds of pairwise evolution
    for _round in 0..3 {
        // Tournament: evolve pair (prog0, prog1)
        let pair = Value::tuple(vec![
            Value::Program(Box::new(programs[0].clone())),
            Value::Program(Box::new(programs[1].clone())),
        ]);

        let inputs = vec![
            pair,
            test_cases.clone(),
            Value::Program(Box::new(evaluator.clone())),
            Value::Program(Box::new(mutator.clone())),
            Value::Int(0x00), // try add opcode
        ];

        let (outputs, _) = interpreter::interpret(&evo_step, &inputs, None).unwrap();
        let evolved_pair = extract_programs(&outputs[0]);
        programs[0] = evolved_pair[0].clone();
        programs[1] = evolved_pair[1].clone();

        // Tournament: evolve pair (prog1, prog2)
        let pair = Value::tuple(vec![
            Value::Program(Box::new(programs[1].clone())),
            Value::Program(Box::new(programs[2].clone())),
        ]);

        let inputs = vec![
            pair,
            test_cases.clone(),
            Value::Program(Box::new(evaluator.clone())),
            Value::Program(Box::new(mutator.clone())),
            Value::Int(0x00),
        ];

        let (outputs, _) = interpreter::interpret(&evo_step, &inputs, None).unwrap();
        let evolved_pair = extract_programs(&outputs[0]);
        programs[1] = evolved_pair[0].clone();
        programs[2] = evolved_pair[1].clone();
    }

    // Check that at least one program has evolved to add
    let final_scores: Vec<i64> = programs
        .iter()
        .map(|p| evaluate_fitness(&evaluator, p, &test_cases))
        .collect();

    let best_score = *final_scores.iter().max().unwrap();
    assert_eq!(
        best_score, 3,
        "after 3 rounds of pairwise evolution, best score should be 3/3 (perfect add)"
    );

    // Verify population has changed from all-zero fitness
    let has_improved = final_scores.iter().any(|&s| s > 0);
    assert!(
        has_improved,
        "at least one program should have improved fitness"
    );
}
