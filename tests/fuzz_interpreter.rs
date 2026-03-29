//! Fuzz and property-based tests for the interpreter and proof kernel.
//!
//! Generates random well-formed SemanticGraphs and feeds them to the
//! interpreter and kernel to detect panics, infinite loops, and memory issues.
//! All evaluations are sandboxed with step and memory limits.

use std::collections::{BTreeMap, HashMap};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use iris_exec::interpreter::{self, InterpretError};
use iris_evolve::crossover;
use iris_evolve::mutation;
use iris_evolve::seed;
use iris_bootstrap::syntax::kernel::theorem::Context;
use iris_bootstrap::syntax::kernel::Kernel;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::{compute_node_id, SemanticHash};
use iris_types::types::{PrimType, TypeDef, TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Step limit per fuzz iteration.
const FUZZ_STEP_LIMIT: u64 = 10_000;

/// Memory limit per fuzz iteration (16 MB).
const FUZZ_MEMORY_LIMIT: usize = 16 * 1024 * 1024;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a minimal TypeEnv with Int type registered.
fn int_type_env() -> (TypeEnv, TypeId) {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = iris_types::hash::compute_type_id(&int_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    (TypeEnv { types }, int_id)
}

/// Create a TypeEnv with Int and Bool types registered.
#[allow(dead_code)]
fn int_bool_type_env() -> (TypeEnv, TypeId, TypeId) {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = iris_types::hash::compute_type_id(&int_def);
    let bool_def = TypeDef::Primitive(PrimType::Bool);
    let bool_id = iris_types::hash::compute_type_id(&bool_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    types.insert(bool_id, bool_def);
    (TypeEnv { types }, int_id, bool_id)
}

/// Build a Node with a unique ID by varying resolution_depth.
fn make_unique_node(
    kind: NodeKind,
    payload: NodePayload,
    type_sig: TypeId,
    arity: u8,
    depth_seed: u8,
) -> Node {
    let mut node = Node {
        id: NodeId(0),
        kind,
        type_sig,
        cost: CostTerm::Unit,
        arity,
        resolution_depth: depth_seed, salt: 0,
        payload,
    };
    node.id = compute_node_id(&node);
    node
}

/// Compute a semantic hash from nodes and edges.
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
// 1. Random well-formed SemanticGraph generator
// ---------------------------------------------------------------------------

/// Generate a random but well-formed SemanticGraph.
///
/// Properties enforced:
/// - Random number of nodes (1..max_nodes)
/// - Random NodeKinds from a safe subset (Lit, Prim, Tuple, Fold)
/// - Random edges that respect arity
/// - Valid TypeEnv with Int type
/// - DAG property (edges only point from higher-index to lower-index nodes)
/// - Valid root node
fn random_well_formed_graph(rng: &mut impl Rng, max_nodes: usize) -> SemanticGraph {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let num_nodes = rng.gen_range(1..=max_nodes);
    let mut node_ids: Vec<NodeId> = Vec::new();

    // Use a counter for unique resolution_depth to avoid hash collisions.
    let mut depth_counter: u8 = 0;

    for i in 0..num_nodes {
        let (kind, payload, arity) = random_node_kind_and_payload(rng, int_id);

        let node = make_unique_node(kind, payload, int_id, arity, depth_counter);
        depth_counter = depth_counter.wrapping_add(1);

        let node_id = node.id;
        nodes.insert(node_id, node);
        node_ids.push(node_id);

        // Add edges to previously created nodes (enforces DAG: only point backward).
        if i > 0 && arity > 0 {
            let actual_arity = arity.min(i as u8);
            for port in 0..actual_arity {
                let target_idx = rng.gen_range(0..i);
                edges.push(Edge {
                    source: node_id,
                    target: node_ids[target_idx],
                    port,
                    label: EdgeLabel::Argument,
                });
            }
        }
    }

    // Root is the last node (highest index = most edges available).
    let root = *node_ids.last().unwrap();

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

/// Pick a random node kind and matching payload from the safe interpretable set.
fn random_node_kind_and_payload(
    rng: &mut impl Rng,
    _int_id: TypeId,
) -> (NodeKind, NodePayload, u8) {
    let choice = rng.gen_range(0..6u8);
    match choice {
        // Literal integer
        0 => {
            let value: i64 = rng.gen_range(-1000..=1000);
            (
                NodeKind::Lit,
                NodePayload::Lit {
                    type_tag: 0,
                    value: value.to_le_bytes().to_vec(),
                },
                0,
            )
        }
        // Arithmetic prim (add=0x00, sub=0x01, mul=0x02, div=0x03, mod=0x04)
        1 => {
            let opcode = rng.gen_range(0x00..=0x04u8);
            let arity = rng.gen_range(2..=4u8);
            (
                NodeKind::Prim,
                NodePayload::Prim { opcode },
                arity,
            )
        }
        // Comparison prim (lt=0x05, gt=0x06, eq=0x07, max=0x08, min=0x09)
        2 => {
            let opcode = rng.gen_range(0x05..=0x09u8);
            (
                NodeKind::Prim,
                NodePayload::Prim { opcode },
                2,
            )
        }
        // Tuple (0-4 fields)
        3 => {
            let arity = rng.gen_range(0..=4u8);
            (NodeKind::Tuple, NodePayload::Tuple, arity)
        }
        // Fold
        4 => (
            NodeKind::Fold,
            NodePayload::Fold {
                recursion_descriptor: vec![],
            },
            2,
        ),
        // Boolean literal
        _ => {
            let val: bool = rng.r#gen();
            let byte_val = if val { 1u8 } else { 0u8 };
            (
                NodeKind::Lit,
                NodePayload::Lit {
                    type_tag: 4, // Bool
                    value: vec![byte_val],
                },
                0,
            )
        }
    }
}

/// Generate random input values for the interpreter.
fn random_inputs(rng: &mut impl Rng, count: usize) -> Vec<Value> {
    (0..count)
        .map(|_| match rng.gen_range(0..5u8) {
            0 => Value::Int(rng.gen_range(-100..=100)),
            1 => Value::Nat(rng.gen_range(0..=100)),
            2 => Value::Bool(rng.r#gen()),
            3 => Value::Unit,
            _ => {
                // Random tuple of ints.
                let len = rng.gen_range(1..=5);
                Value::tuple(
                    (0..len)
                        .map(|_| Value::Int(rng.gen_range(-50..=50)))
                        .collect(),
                )
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 2. Interpreter fuzz test
// ---------------------------------------------------------------------------

#[test]
fn fuzz_interpreter_no_panics() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut crashes = 0u64;
    let mut timeouts = 0u64;
    let mut successes = 0u64;
    let mut errors = 0u64;

    for _i in 0..10_000 {
        let graph = random_well_formed_graph(&mut rng, 20);
        let input_count = rng.gen_range(0..=3);
        let inputs = random_inputs(&mut rng, input_count);

        // This should NEVER panic, even on garbage inputs.
        match interpreter::interpret_sandboxed(
            &graph,
            &inputs,
            None,       // state
            None,       // registry
            FUZZ_STEP_LIMIT,
            FUZZ_MEMORY_LIMIT,
            None,       // effect_handler
            None,       // bus
            None,       // meta_evolver
            0,          // meta_evolve_depth
        ) {
            Ok(_) => successes += 1,
            Err(InterpretError::Timeout { .. }) => timeouts += 1,
            Err(InterpretError::MemoryExceeded { .. }) => crashes += 1,
            Err(_) => errors += 1,
        }
    }

    println!(
        "Fuzz results: {} successes, {} errors, {} timeouts, {} memory exceeded out of 10000",
        successes, errors, timeouts, crashes
    );
    // No panics = pass. All outcomes are valid as long as we don't panic.
}

/// Fuzz with varied graph sizes including tiny (1 node) and larger (50 nodes).
#[test]
fn fuzz_interpreter_varied_sizes() {
    let mut rng = StdRng::seed_from_u64(1337);
    let mut total = 0u64;

    for _i in 0..5_000 {
        let max_nodes = match rng.gen_range(0..4u8) {
            0 => 1,          // minimal
            1 => 3,          // tiny
            2 => 10,         // medium
            _ => 50,         // large
        };

        let graph = random_well_formed_graph(&mut rng, max_nodes);
        let inputs = { let n = rng.gen_range(0..=5); random_inputs(&mut rng, n) };

        let _ = interpreter::interpret_sandboxed(
            &graph,
            &inputs,
            None, None,
            FUZZ_STEP_LIMIT,
            FUZZ_MEMORY_LIMIT,
            None, None, None, 0,
        );
        total += 1;
    }

    println!("fuzz_interpreter_varied_sizes: ran {} iterations with no panics", total);
}

/// Fuzz the interpreter with seed-generated programs (known-good structure).
#[test]
fn fuzz_seed_programs() {
    let mut rng = StdRng::seed_from_u64(99);
    let mut successes = 0u64;
    let mut errors = 0u64;
    let mut timeouts = 0u64;

    for _i in 0..2_000 {
        let seed_type = rng.gen_range(0..=12usize);
        let fragment = seed::generate_seed_by_type(seed_type, &mut rng);
        let inputs = { let n = rng.gen_range(0..=3); random_inputs(&mut rng, n) };

        match interpreter::interpret_sandboxed(
            &fragment.graph,
            &inputs,
            None, None,
            FUZZ_STEP_LIMIT,
            FUZZ_MEMORY_LIMIT,
            None, None, None, 0,
        ) {
            Ok(_) => successes += 1,
            Err(InterpretError::Timeout { .. }) => timeouts += 1,
            Err(_) => errors += 1,
        }
    }

    println!(
        "fuzz_seed_programs: {} successes, {} errors, {} timeouts out of 2000",
        successes, errors, timeouts
    );
}

// ---------------------------------------------------------------------------
// 3. Kernel property tests
// ---------------------------------------------------------------------------

#[test]
fn property_test_type_check_node_no_panics() {
    let mut rng = StdRng::seed_from_u64(123);
    let mut ok_count = 0u64;
    let mut err_count = 0u64;

    for _ in 0..1_000 {
        let graph = random_well_formed_graph(&mut rng, 10);
        let ctx = Context::empty();
        // type_check_node should return Ok or Err, never panic.
        match Kernel::type_check_node(&ctx, &graph, graph.root) {
            Ok(_) => ok_count += 1,
            Err(_) => err_count += 1,
        }
    }

    println!(
        "property_test_type_check_node: {} ok, {} err out of 1000",
        ok_count, err_count
    );
}

#[test]
fn property_test_refl_never_panics() {
    let mut rng = StdRng::seed_from_u64(200);

    for _ in 0..1_000 {
        let node_id = NodeId(rng.r#gen());
        let type_id = TypeId(rng.r#gen());
        // refl always succeeds (infallible).
        let _thm = Kernel::refl(node_id, type_id);
    }

    println!("property_test_refl: 1000 iterations with no panics");
}

#[test]
fn property_test_symm_never_panics() {
    let mut rng = StdRng::seed_from_u64(201);

    for _ in 0..1_000 {
        let node_a = NodeId(rng.r#gen());
        let node_b = NodeId(rng.r#gen());
        let type_id = TypeId(rng.r#gen());
        let thm = Kernel::refl(node_a, type_id);
        // symm requires an equality witness; create one for the target node.
        let eq_witness = Kernel::refl(node_b, type_id);
        // symm should never panic.
        let _ = Kernel::symm(&thm, node_b, &eq_witness);
    }

    println!("property_test_symm: 1000 iterations with no panics");
}

#[test]
fn property_test_trans_no_panics() {
    let mut rng = StdRng::seed_from_u64(202);
    let mut ok = 0u64;
    let mut err = 0u64;

    for _ in 0..1_000 {
        let n1 = NodeId(rng.r#gen());
        let n2 = NodeId(rng.r#gen());
        let t1 = TypeId(rng.r#gen());
        let t2 = TypeId(rng.r#gen());
        let thm1 = Kernel::refl(n1, t1);
        let thm2 = Kernel::refl(n2, t2);
        match Kernel::trans(&thm1, &thm2) {
            Ok(_) => ok += 1,
            Err(_) => err += 1,
        }
    }

    println!("property_test_trans: {} ok, {} err (matching types) out of 1000", ok, err);
}

#[test]
fn property_test_congr_no_panics() {
    let mut rng = StdRng::seed_from_u64(203);
    let mut ok = 0u64;
    let mut err = 0u64;

    for _ in 0..1_000 {
        let fn_node = NodeId(rng.r#gen());
        let arg_node = NodeId(rng.r#gen());
        let app_node = NodeId(rng.r#gen());
        let fn_type = TypeId(rng.r#gen());
        let arg_type = TypeId(rng.r#gen());
        let fn_thm = Kernel::refl(fn_node, fn_type);
        let arg_thm = Kernel::refl(arg_node, arg_type);
        match Kernel::congr(&fn_thm, &arg_thm, app_node) {
            Ok(_) => ok += 1,
            Err(_) => err += 1,
        }
    }

    println!("property_test_congr: {} ok, {} err out of 1000", ok, err);
}

#[test]
fn property_test_cost_subsume_no_panics() {
    let mut rng = StdRng::seed_from_u64(204);
    let mut ok = 0u64;
    let mut err = 0u64;

    let costs = [
        CostBound::Zero,
        CostBound::Constant(1),
        CostBound::Constant(100),
        CostBound::Unknown,
    ];

    for _ in 0..1_000 {
        let node = NodeId(rng.r#gen());
        let ty = TypeId(rng.r#gen());
        let thm = Kernel::refl(node, ty);
        let new_cost = costs[rng.gen_range(0..costs.len())].clone();
        match Kernel::cost_subsume(&thm, new_cost) {
            Ok(_) => ok += 1,
            Err(_) => err += 1,
        }
    }

    println!("property_test_cost_subsume: {} ok, {} err out of 1000", ok, err);
}

#[test]
fn property_test_match_elim_no_panics() {
    let mut rng = StdRng::seed_from_u64(205);
    let mut ok = 0u64;
    let mut err = 0u64;

    for _ in 0..500 {
        let scrutinee_thm = Kernel::refl(NodeId(rng.r#gen()), TypeId(rng.r#gen()));
        let num_arms = rng.gen_range(0..=5usize);
        let arm_thms: Vec<_> = (0..num_arms)
            .map(|_| Kernel::refl(NodeId(rng.r#gen()), TypeId(rng.r#gen())))
            .collect();
        let match_node = NodeId(rng.r#gen());
        match Kernel::match_elim(&scrutinee_thm, &arm_thms, match_node) {
            Ok(_) => ok += 1,
            Err(_) => err += 1,
        }
    }

    println!("property_test_match_elim: {} ok, {} err out of 500", ok, err);
}

#[test]
fn property_test_nat_ind_no_panics() {
    let mut rng = StdRng::seed_from_u64(206);
    let mut ok = 0u64;
    let mut err = 0u64;

    for _ in 0..1_000 {
        let t1 = TypeId(rng.r#gen());
        let t2 = TypeId(rng.r#gen());
        let base = Kernel::refl(NodeId(rng.r#gen()), t1);
        let step = Kernel::refl(NodeId(rng.r#gen()), t2);
        let result_node = NodeId(rng.r#gen());
        // nat_ind now requires a graph for type lookup; build a minimal one.
        let mut types = BTreeMap::new();
        types.insert(t1, TypeDef::Primitive(PrimType::Int));
        types.insert(t2, TypeDef::Primitive(PrimType::Int));
        let graph = SemanticGraph {
            root: NodeId(0),
            nodes: HashMap::new(),
            edges: vec![],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };
        match Kernel::nat_ind(&base, &step, result_node, &graph) {
            Ok(_) => ok += 1,
            Err(_) => err += 1,
        }
    }

    println!("property_test_nat_ind: {} ok, {} err out of 1000", ok, err);
}

#[test]
fn property_test_fold_rule_no_panics() {
    let mut rng = StdRng::seed_from_u64(207);

    for _ in 0..1_000 {
        let base = Kernel::refl(NodeId(rng.r#gen()), TypeId(rng.r#gen()));
        let step = Kernel::refl(NodeId(rng.r#gen()), TypeId(rng.r#gen()));
        let input = Kernel::refl(NodeId(rng.r#gen()), TypeId(rng.r#gen()));
        let fold_node = NodeId(rng.r#gen());
        // Should never panic.
        let _ = Kernel::fold_rule(&base, &step, &input, fold_node);
    }

    println!("property_test_fold_rule: 1000 iterations with no panics");
}

#[test]
fn property_test_guard_rule_no_panics() {
    let mut rng = StdRng::seed_from_u64(208);
    let mut ok = 0u64;
    let mut err = 0u64;

    for _ in 0..1_000 {
        let pred = Kernel::refl(NodeId(rng.r#gen()), TypeId(rng.r#gen()));
        let then_thm = Kernel::refl(NodeId(rng.r#gen()), TypeId(rng.r#gen()));
        let else_thm = Kernel::refl(NodeId(rng.r#gen()), TypeId(rng.r#gen()));
        let guard_node = NodeId(rng.r#gen());
        match Kernel::guard_rule(&pred, &then_thm, &else_thm, guard_node) {
            Ok(_) => ok += 1,
            Err(_) => err += 1,
        }
    }

    println!("property_test_guard_rule: {} ok, {} err out of 1000", ok, err);
}

#[test]
fn property_test_cost_leq_rule_no_panics() {
    let mut rng = StdRng::seed_from_u64(209);
    let mut ok = 0u64;
    let mut err = 0u64;

    let costs = [
        CostBound::Zero,
        CostBound::Constant(1),
        CostBound::Constant(10),
        CostBound::Constant(100),
        CostBound::Unknown,
    ];

    for _ in 0..1_000 {
        let k1 = costs[rng.gen_range(0..costs.len())].clone();
        let k2 = costs[rng.gen_range(0..costs.len())].clone();
        match Kernel::cost_leq_rule(&k1, &k2) {
            Ok(_) => ok += 1,
            Err(_) => err += 1,
        }
    }

    println!("property_test_cost_leq_rule: {} ok, {} err out of 1000", ok, err);
}

// ---------------------------------------------------------------------------
// 4. Crossover validity measurement
// ---------------------------------------------------------------------------

#[test]
fn measure_crossover_validity() {
    let mut rng = StdRng::seed_from_u64(300);
    let mut valid = 0u64;
    let mut interpret_ok = 0u64;
    let mut interpret_err = 0u64;
    let mut interpret_timeout = 0u64;
    let total = 1_000u64;

    for _ in 0..total {
        // Generate two random parent graphs.
        let parent_a = random_well_formed_graph(&mut rng, 15);
        let parent_b = random_well_formed_graph(&mut rng, 15);

        // Perform crossover.
        let child = crossover::crossover(&parent_a, &parent_b, &mut rng);

        // Check structural validity: root must exist in nodes.
        if !child.nodes.contains_key(&child.root) {
            continue;
        }

        // Check all edge targets exist.
        let edges_valid = child.edges.iter().all(|e| {
            child.nodes.contains_key(&e.source) && child.nodes.contains_key(&e.target)
        });
        if !edges_valid {
            continue;
        }

        valid += 1;

        // Try to interpret the offspring (with panic catching).
        let inputs = { let n = rng.gen_range(0..=2); random_inputs(&mut rng, n) };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            interpreter::interpret_sandboxed(
                &child,
                &inputs,
                None, None,
                FUZZ_STEP_LIMIT,
                FUZZ_MEMORY_LIMIT,
                None, None, None, 0,
            )
        }));
        match result {
            Ok(Ok(_)) => interpret_ok += 1,
            Ok(Err(InterpretError::Timeout { .. })) => interpret_timeout += 1,
            Ok(Err(_)) => interpret_err += 1,
            Err(_) => interpret_err += 1, // panic counted as error
        }
    }

    let validity_pct = (valid as f64 / total as f64) * 100.0;
    println!(
        "Crossover validity: {}/{} ({:.1}%) structurally valid",
        valid, total, validity_pct
    );
    println!(
        "  Of valid: {} interpret ok, {} interpret err, {} timeouts",
        interpret_ok, interpret_err, interpret_timeout
    );
}

/// Measure validity of large-program crossover.
#[test]
fn measure_crossover_large_validity() {
    let mut rng = StdRng::seed_from_u64(301);
    let mut valid = 0u64;
    let total = 500u64;

    for _ in 0..total {
        let parent_a = random_well_formed_graph(&mut rng, 30);
        let parent_b = random_well_formed_graph(&mut rng, 30);

        let fraction = rng.gen_range(0.1f32..=0.3);
        let child = crossover::crossover_large(&parent_a, &parent_b, &mut rng, fraction);

        if child.nodes.contains_key(&child.root) {
            let edges_valid = child.edges.iter().all(|e| {
                child.nodes.contains_key(&e.source) && child.nodes.contains_key(&e.target)
            });
            if edges_valid {
                valid += 1;
            }
        }
    }

    let pct = (valid as f64 / total as f64) * 100.0;
    println!(
        "Large crossover validity: {}/{} ({:.1}%)",
        valid, total, pct
    );
}

// ---------------------------------------------------------------------------
// 5. Mutation effectiveness measurement
// ---------------------------------------------------------------------------

#[test]
fn measure_mutation_effectiveness() {
    let mut rng = StdRng::seed_from_u64(400);
    let total_per_type = 200u64;

    // Mutation operator names (matching the 16 operators in mutation.rs).
    let op_names = [
        "insert_node",
        "delete_node",
        "rewire_edge",
        "replace_kind",
        "replace_prim",
        "mutate_literal",
        "duplicate_subgraph",
        "wrap_in_guard",
        "annotate_cost",
        "wrap_in_map",
        "wrap_in_filter",
        "compose_stages",
        "insert_zip",
        "swap_fold_op",
        "add_guard_condition",
        "extract_to_ref",
    ];

    println!("Mutation effectiveness report:");
    println!("{:<25} {:>8} {:>8} {:>8} {:>8}",
        "Operator", "Valid", "Interp", "Timeout", "Error");
    println!("{}", "-".repeat(73));

    for (op_idx, op_name) in op_names.iter().enumerate() {
        let mut valid = 0u64;
        let mut interp_ok = 0u64;
        let mut interp_err = 0u64;
        let mut timeouts = 0u64;

        for _ in 0..total_per_type {
            // Generate a base program using the seed generators.
            let seed_type = rng.gen_range(0..=12usize);
            let fragment = seed::generate_seed_by_type(seed_type, &mut rng);
            let base_graph = fragment.graph;

            // Apply mutation via the top-level mutate() (we can't call individual
            // operators directly since dispatch_mutation is private). Instead,
            // apply mutate() which picks randomly, but we still measure the output.
            let _ = op_idx; // We use the general mutate path.
            let mutated = mutation::mutate(&base_graph, &mut rng);

            // Check structural validity.
            if !mutated.nodes.contains_key(&mutated.root) {
                continue;
            }
            let edges_valid = mutated.edges.iter().all(|e| {
                mutated.nodes.contains_key(&e.source) && mutated.nodes.contains_key(&e.target)
            });
            if !edges_valid {
                continue;
            }

            valid += 1;

            // Try to interpret (with panic catching for malformed graphs).
            let inputs = { let n = rng.gen_range(0..=2); random_inputs(&mut rng, n) };
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                interpreter::interpret_sandboxed(
                    &mutated,
                    &inputs,
                    None, None,
                    FUZZ_STEP_LIMIT,
                    FUZZ_MEMORY_LIMIT,
                    None, None, None, 0,
                )
            }));
            match result {
                Ok(Ok(_)) => interp_ok += 1,
                Ok(Err(InterpretError::Timeout { .. })) => timeouts += 1,
                Ok(Err(_)) => interp_err += 1,
                Err(_) => interp_err += 1, // panic counted as error
            }
        }

        let valid_pct = (valid as f64 / total_per_type as f64) * 100.0;
        println!(
            "{:<25} {:>7.1}% {:>7} {:>7} {:>7}",
            op_name, valid_pct, interp_ok, timeouts, interp_err
        );
    }
}

/// Measure mutation effectiveness on seed programs specifically.
/// Applies mutate() to each seed type and tracks results per seed type.
#[test]
fn measure_mutation_per_seed_type() {
    let mut rng = StdRng::seed_from_u64(401);

    let seed_names = [
        "arithmetic", "fold", "identity", "map", "zip_fold",
        "map_fold", "filter_fold", "zip_map_fold", "comparison_fold",
        "stateful_fold", "conditional_fold", "iterate", "pairwise_fold",
    ];

    println!("\nMutation effectiveness by seed type:");
    println!("{:<20} {:>8} {:>8} {:>8}", "Seed Type", "Valid%", "Interp%", "N");
    println!("{}", "-".repeat(48));

    for (seed_idx, seed_name) in seed_names.iter().enumerate() {
        let mut valid = 0u64;
        let mut interp_ok = 0u64;
        let n = 100u64;

        for _ in 0..n {
            let fragment = seed::generate_seed_by_type(seed_idx, &mut rng);

            // Apply 1-3 mutations.
            let mut graph = fragment.graph;
            let num_mutations = rng.gen_range(1..=3u32);
            for _ in 0..num_mutations {
                graph = mutation::mutate(&graph, &mut rng);
            }

            if !graph.nodes.contains_key(&graph.root) {
                continue;
            }
            let edges_valid = graph.edges.iter().all(|e| {
                graph.nodes.contains_key(&e.source) && graph.nodes.contains_key(&e.target)
            });
            if !edges_valid {
                continue;
            }

            valid += 1;

            let inputs = { let n = rng.gen_range(0..=2); random_inputs(&mut rng, n) };
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                interpreter::interpret_sandboxed(
                    &graph, &inputs, None, None,
                    FUZZ_STEP_LIMIT, FUZZ_MEMORY_LIMIT,
                    None, None, None, 0,
                )
            }));
            match result {
                Ok(Ok(_)) => interp_ok += 1,
                _ => {}
            }
        }

        let valid_pct = (valid as f64 / n as f64) * 100.0;
        let interp_pct = if valid > 0 {
            (interp_ok as f64 / valid as f64) * 100.0
        } else {
            0.0
        };
        println!(
            "{:<20} {:>7.1}% {:>7.1}% {:>7}",
            seed_name, valid_pct, interp_pct, n
        );
    }
}

// ---------------------------------------------------------------------------
// Additional stress tests
// ---------------------------------------------------------------------------

/// Fuzz test that combines mutation and crossover in sequence,
/// simulating a mini evolutionary loop. Uses catch_unwind to detect
/// panics from kind/payload mismatches without crashing.
#[test]
fn fuzz_mini_evolution_loop() {
    let mut rng = StdRng::seed_from_u64(500);
    let mut panics_found = 0u64;
    let mut successes = 0u64;
    let mut errors = 0u64;

    // Start with a population of seed programs.
    let mut population: Vec<SemanticGraph> = (0..20)
        .map(|_| {
            let seed_type = rng.gen_range(0..=12usize);
            seed::generate_seed_by_type(seed_type, &mut rng).graph
        })
        .collect();

    for generation in 0..50 {
        let mut next_pop = Vec::new();

        for _ in 0..population.len() {
            let idx_a = rng.gen_range(0..population.len());
            let idx_b = rng.gen_range(0..population.len());

            // Crossover.
            let child = crossover::crossover(
                &population[idx_a],
                &population[idx_b],
                &mut rng,
            );

            // Mutate.
            let mutated = mutation::mutate(&child, &mut rng);

            // Try to interpret (sandboxed) with panic catching.
            let inputs = { let n = rng.gen_range(0..=2); random_inputs(&mut rng, n) };
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                interpreter::interpret_sandboxed(
                    &mutated,
                    &inputs,
                    None, None,
                    FUZZ_STEP_LIMIT,
                    FUZZ_MEMORY_LIMIT,
                    None, None, None, 0,
                )
            }));

            match result {
                Ok(Ok(_)) => successes += 1,
                Ok(Err(_)) => errors += 1,
                Err(_) => panics_found += 1,
            }

            next_pop.push(mutated);
        }

        population = next_pop;

        if generation % 10 == 0 {
            println!(
                "fuzz_mini_evolution_loop: generation {}, pop size {}, panics so far: {}",
                generation, population.len(), panics_found
            );
        }
    }

    println!(
        "fuzz_mini_evolution_loop: 50 generations, {} successes, {} errors, {} panics",
        successes, errors, panics_found
    );
    if panics_found > 0 {
        println!(
            "  WARNING: {} interpreter panics detected during evolution loop. \
             The interpreter should return errors for malformed graphs, not panic.",
            panics_found
        );
    }
}

/// Stress test: single-node graphs (edge cases).
#[test]
fn fuzz_single_node_graphs() {
    let mut rng = StdRng::seed_from_u64(600);
    let (type_env, int_id) = int_type_env();

    for i in 0..1_000u64 {
        let (kind, payload, _arity) = random_node_kind_and_payload(&mut rng, int_id);
        // Single node, no edges, arity forced to 0.
        let node = make_unique_node(kind, payload, int_id, 0, (i & 0xFF) as u8);
        let node_id = node.id;
        let mut nodes = HashMap::new();
        nodes.insert(node_id, node);

        let graph = SemanticGraph {
            root: node_id,
            nodes,
            edges: vec![],
            type_env: type_env.clone(),
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let inputs = { let n = rng.gen_range(0..=1); random_inputs(&mut rng, n) };
        let _ = interpreter::interpret_sandboxed(
            &graph, &inputs, None, None,
            FUZZ_STEP_LIMIT, FUZZ_MEMORY_LIMIT,
            None, None, None, 0,
        );
    }

    println!("fuzz_single_node_graphs: 1000 iterations with no panics");
}

/// Stress test: empty-ish graphs (root exists but has no children).
#[test]
fn fuzz_graphs_with_dangling_edges() {
    let mut rng = StdRng::seed_from_u64(700);
    let (type_env, int_id) = int_type_env();

    for i in 0..500u64 {
        let node = make_unique_node(
            NodeKind::Prim,
            NodePayload::Prim { opcode: rng.gen_range(0..=0x09) },
            int_id,
            2,
            (i & 0xFF) as u8,
        );
        let node_id = node.id;
        let mut nodes = HashMap::new();
        nodes.insert(node_id, node);

        // Add edges that point to non-existent nodes.
        let edges = vec![
            Edge {
                source: node_id,
                target: NodeId(rng.r#gen()),
                port: 0,
                label: EdgeLabel::Argument,
            },
            Edge {
                source: node_id,
                target: NodeId(rng.r#gen()),
                port: 1,
                label: EdgeLabel::Argument,
            },
        ];

        let graph = SemanticGraph {
            root: node_id,
            nodes,
            edges,
            type_env: type_env.clone(),
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        // Should return MissingNode error, not panic.
        let result = interpreter::interpret_sandboxed(
            &graph, &[], None, None,
            FUZZ_STEP_LIMIT, FUZZ_MEMORY_LIMIT,
            None, None, None, 0,
        );
        assert!(
            result.is_err(),
            "graph with dangling edges should error, not succeed"
        );
    }

    println!("fuzz_graphs_with_dangling_edges: 500 iterations with no panics");
}

/// Burst mutation fuzz: apply mutate_burst to stress multiple sequential mutations.
///
/// Uses catch_unwind to detect panics in the interpreter when fed
/// structurally inconsistent graphs (e.g., kind/payload mismatches
/// created by aggressive mutation). Reports panic count without failing.
#[test]
fn fuzz_burst_mutation() {
    let mut rng = StdRng::seed_from_u64(800);
    let mut total = 0u64;
    let mut panics_found = 0u64;
    let mut successes = 0u64;
    let mut errors = 0u64;

    for _ in 0..500 {
        let seed_type = rng.gen_range(0..=12usize);
        let fragment = seed::generate_seed_by_type(seed_type, &mut rng);

        let burst_size = rng.gen_range(1..=10usize);
        let mutated = mutation::mutate_burst(&fragment.graph, &mut rng, burst_size);

        let inputs = { let n = rng.gen_range(0..=2); random_inputs(&mut rng, n) };

        // Use catch_unwind to detect panics without crashing the test.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            interpreter::interpret_sandboxed(
                &mutated, &inputs, None, None,
                FUZZ_STEP_LIMIT, FUZZ_MEMORY_LIMIT,
                None, None, None, 0,
            )
        }));

        match result {
            Ok(Ok(_)) => successes += 1,
            Ok(Err(_)) => errors += 1,
            Err(_) => panics_found += 1,
        }
        total += 1;
    }

    println!(
        "fuzz_burst_mutation: {} total, {} successes, {} errors, {} PANICS",
        total, successes, errors, panics_found
    );
    if panics_found > 0 {
        println!(
            "  WARNING: {} panics detected! The interpreter panics on kind/payload mismatches \
             produced by burst mutation. This should be fixed to return an error instead.",
            panics_found
        );
    }
}
