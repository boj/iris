
//! Test harness for iris-checker .iris programs.
//!
//! Loads each checker .iris file, compiles it via iris_bootstrap::syntax::compile(),
//! registers all fragments in a FragmentRegistry, then evaluates the entry
//! fragment with appropriate inputs and asserts expected outputs.

use std::collections::HashMap;

use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::*;
use iris_types::guard::{BlobRef, GuardSpec};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compile IRIS source, register all fragments, return named graphs + registry.
fn compile_with_registry(src: &str) -> (Vec<(String, SemanticGraph)>, FragmentRegistry) {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!(
            "compilation failed with {} errors:\n{}",
            result.errors.len(),
            result
                .errors
                .iter()
                .map(|e| iris_bootstrap::syntax::format_error(src, e))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    let mut registry = FragmentRegistry::new();
    for (_, frag, _) in &result.fragments {
        registry.register(frag.clone());
    }

    let named: Vec<_> = result
        .fragments
        .into_iter()
        .map(|(name, frag, _)| (name, frag.graph))
        .collect();

    (named, registry)
}

/// Find a named fragment in the compiled output.
fn find_fragment<'a>(
    fragments: &'a [(String, SemanticGraph)],
    name: &str,
) -> &'a SemanticGraph {
    &fragments
        .iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| {
            let names: Vec<_> = fragments.iter().map(|(n, _)| n.as_str()).collect();
            panic!(
                "fragment '{}' not found; available: {:?}",
                name, names
            )
        })
        .1
}

/// Evaluate a SemanticGraph with given inputs and return outputs.
fn eval(
    graph: &SemanticGraph,
    inputs: &[Value],
    registry: &FragmentRegistry,
) -> Vec<Value> {
    interpreter::interpret_with_registry(graph, inputs, None, Some(registry))
        .unwrap_or_else(|e| panic!("evaluation error: {:?}", e))
        .0
}

/// Evaluate and return the first output as i64.
fn eval_int(
    graph: &SemanticGraph,
    inputs: &[Value],
    registry: &FragmentRegistry,
) -> i64 {
    let outputs = eval(graph, inputs, registry);
    match outputs.first() {
        Some(Value::Int(n)) => *n,
        Some(Value::Bool(b)) => if *b { 1 } else { 0 },
        other => panic!("expected Int output, got: {:?}", other),
    }
}

/// Load a checker .iris file source.
fn load_checker_source(filename: &str) -> String {
    let path = format!("src/iris-programs/checker/{}", filename);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path, e))
}

// ---------------------------------------------------------------------------
// Graph construction helpers
// ---------------------------------------------------------------------------

fn int_lit(id: u64, value: i64) -> (NodeId, Node) {
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind: NodeKind::Lit,
            type_sig: TypeId(0),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0x00,
                value: value.to_le_bytes().to_vec(),
            },
        },
    )
}

fn prim_node(id: u64, opcode: u8, arity: u8) -> (NodeId, Node) {
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind: NodeKind::Prim,
            type_sig: TypeId(0),
            cost: CostTerm::Unit,
            arity,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Prim { opcode },
        },
    )
}

fn fold_node(id: u64) -> (NodeId, Node) {
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind: NodeKind::Fold,
            type_sig: TypeId(0),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Fold {
                recursion_descriptor: vec![],
            },
        },
    )
}

fn neural_node(id: u64) -> (NodeId, Node) {
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind: NodeKind::Neural,
            type_sig: TypeId(0),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Neural {
                guard_spec: GuardSpec::default(),
                weight_blob: BlobRef { hash: [0; 32], size: 0 },
            },
        },
    )
}

fn make_graph(nodes: HashMap<NodeId, Node>, edges: Vec<Edge>, root: u64) -> SemanticGraph {
    SemanticGraph {
        root: NodeId(root),
        nodes,
        edges,
        type_env: TypeEnv {
            types: std::collections::BTreeMap::new(),
        },
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

/// Build a simple 3-node graph: Prim(add, opcode=0x00) + 2 Lits.
fn make_test_graph_3nodes() -> SemanticGraph {
    let nodes = HashMap::from([
        int_lit(1, 10),
        int_lit(2, 20),
        prim_node(3, 0x00, 2),
    ]);
    let edges = vec![
        Edge {
            source: NodeId(3),
            target: NodeId(1),
            port: 0,
            label: EdgeLabel::Argument,
        },
        Edge {
            source: NodeId(3),
            target: NodeId(2),
            port: 1,
            label: EdgeLabel::Argument,
        },
    ];
    make_graph(nodes, edges, 3)
}

/// Build a graph with a Neural node (kind 7) for tier testing.
fn make_neural_graph() -> SemanticGraph {
    let nodes = HashMap::from([neural_node(1)]);
    make_graph(nodes, vec![], 1)
}

/// Build a graph with a Fold node (kind 8) for tier testing.
fn make_fold_graph() -> SemanticGraph {
    let nodes = HashMap::from([fold_node(1)]);
    make_graph(nodes, vec![], 1)
}

/// Build a graph with a Lit node (kind 5) for tier testing.
fn make_lit_graph() -> SemanticGraph {
    let nodes = HashMap::from([int_lit(1, 42)]);
    make_graph(nodes, vec![], 1)
}

// ---------------------------------------------------------------------------
// Tests: obligation_count.iris
// ---------------------------------------------------------------------------

#[test]
fn test_obligation_count_3nodes() {
    let src = load_checker_source("obligation_count.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "obligation_count");

    let test_program = make_test_graph_3nodes();
    let result = eval_int(graph, &[Value::Program(Box::new(test_program))], &registry);
    assert_eq!(result, 3, "3-node graph should have 3 obligations");
}

#[test]
fn test_obligation_count_1node() {
    let src = load_checker_source("obligation_count.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "obligation_count");

    let test_program = make_lit_graph();
    let result = eval_int(graph, &[Value::Program(Box::new(test_program))], &registry);
    assert_eq!(result, 1, "1-node graph should have 1 obligation");
}

// ---------------------------------------------------------------------------
// Tests: tier_classify.iris
// ---------------------------------------------------------------------------

#[test]
fn test_tier_classify_lit_is_tier0() {
    let src = load_checker_source("tier_classify.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "tier_classify");

    // Lit node (kind 5) → tier 0
    let test_program = make_lit_graph();
    let result = eval_int(
        graph,
        &[Value::Program(Box::new(test_program)), Value::Int(1)],
        &registry,
    );
    assert_eq!(result, 0, "Lit node should be tier 0");
}

#[test]
fn test_tier_classify_prim_is_tier0() {
    let src = load_checker_source("tier_classify.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "tier_classify");

    // Prim node (kind 0) → tier 0
    let nodes = HashMap::from([prim_node(1, 0x00, 2)]);
    let test_program = make_graph(nodes, vec![], 1);
    let result = eval_int(
        graph,
        &[Value::Program(Box::new(test_program)), Value::Int(1)],
        &registry,
    );
    assert_eq!(result, 0, "Prim node should be tier 0");
}

#[test]
fn test_tier_classify_fold_is_tier1() {
    let src = load_checker_source("tier_classify.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "tier_classify");

    // Fold node (kind 8) → tier 1
    let test_program = make_fold_graph();
    let result = eval_int(
        graph,
        &[Value::Program(Box::new(test_program)), Value::Int(1)],
        &registry,
    );
    assert_eq!(result, 1, "Fold node should be tier 1");
}

#[test]
fn test_tier_classify_neural_is_tier3() {
    let src = load_checker_source("tier_classify.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "tier_classify");

    // Neural node (kind 7) → tier 3
    let test_program = make_neural_graph();
    let result = eval_int(
        graph,
        &[Value::Program(Box::new(test_program)), Value::Int(1)],
        &registry,
    );
    assert_eq!(result, 3, "Neural node should be tier 3");
}

// ---------------------------------------------------------------------------
// Tests: type_check.iris
// ---------------------------------------------------------------------------

#[test]
fn test_type_check_3nodes() {
    let src = load_checker_source("type_check.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "type_check");

    // Well-formed 3-node graph: all nodes have valid kinds (>= 0)
    let test_program = make_test_graph_3nodes();
    let result = eval_int(graph, &[Value::Program(Box::new(test_program))], &registry);
    assert_eq!(result, 3, "all 3 nodes should pass type check");
}

#[test]
fn test_type_check_1node() {
    let src = load_checker_source("type_check.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "type_check");

    let test_program = make_lit_graph();
    let result = eval_int(graph, &[Value::Program(Box::new(test_program))], &registry);
    assert_eq!(result, 1, "1-node graph should have 1 passing node");
}

// ---------------------------------------------------------------------------
// Tests: cost_check.iris
// ---------------------------------------------------------------------------

#[test]
fn test_cost_kind_zero() {
    let src = load_checker_source("cost_check.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "cost_kind");

    // cost == 0 → kind 0 (Zero)
    assert_eq!(eval_int(graph, &[Value::Int(0)], &registry), 0);
}

#[test]
fn test_cost_kind_constant() {
    let src = load_checker_source("cost_check.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "cost_kind");

    // cost < 100 → kind 1 (Constant)
    assert_eq!(eval_int(graph, &[Value::Int(50)], &registry), 1);
}

#[test]
fn test_cost_kind_linear() {
    let src = load_checker_source("cost_check.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "cost_kind");

    // 100 <= cost < 10000 → kind 2 (Linear)
    assert_eq!(eval_int(graph, &[Value::Int(500)], &registry), 2);
}

#[test]
fn test_cost_kind_nlogn() {
    let src = load_checker_source("cost_check.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "cost_kind");

    // 10000 <= cost < 100000 → kind 3 (NLogN)
    assert_eq!(eval_int(graph, &[Value::Int(50000)], &registry), 3);
}

#[test]
fn test_cost_kind_polynomial() {
    let src = load_checker_source("cost_check.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "cost_kind");

    // 100000 <= cost < 1000000 → kind 4 (Polynomial)
    assert_eq!(eval_int(graph, &[Value::Int(500000)], &registry), 4);
}

#[test]
fn test_cost_kind_unknown() {
    let src = load_checker_source("cost_check.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "cost_kind");

    // cost >= 1000000 → kind 5 (Unknown)
    assert_eq!(eval_int(graph, &[Value::Int(2000000)], &registry), 5);
}

#[test]
fn test_cost_leq_base_ordering() {
    let src = load_checker_source("cost_check.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "cost_leq_base");

    // Same kind: 2 <= 2 → 1
    assert_eq!(eval_int(graph, &[Value::Int(2), Value::Int(2)], &registry), 1);
    // Lower kind: 1 <= 3 → 1
    assert_eq!(eval_int(graph, &[Value::Int(1), Value::Int(3)], &registry), 1);
    // Higher kind: 4 <= 2 → 0
    assert_eq!(eval_int(graph, &[Value::Int(4), Value::Int(2)], &registry), 0);
}

#[test]
fn test_is_at_least_linear() {
    let src = load_checker_source("cost_check.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "is_at_least_linear");

    assert_eq!(eval_int(graph, &[Value::Int(0)], &registry), 0);
    assert_eq!(eval_int(graph, &[Value::Int(1)], &registry), 0);
    assert_eq!(eval_int(graph, &[Value::Int(2)], &registry), 1);
    assert_eq!(eval_int(graph, &[Value::Int(3)], &registry), 1);
}

#[test]
fn test_cost_sum() {
    let src = load_checker_source("cost_check.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "cost_sum");

    // Sum returns the higher kind
    assert_eq!(eval_int(graph, &[Value::Int(1), Value::Int(3)], &registry), 3);
    assert_eq!(eval_int(graph, &[Value::Int(4), Value::Int(2)], &registry), 4);
    assert_eq!(eval_int(graph, &[Value::Int(2), Value::Int(2)], &registry), 2);
}

#[test]
fn test_cost_mul() {
    let src = load_checker_source("cost_check.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "cost_mul");

    // Zero * anything = Zero
    assert_eq!(eval_int(graph, &[Value::Int(0), Value::Int(3)], &registry), 0);
    assert_eq!(eval_int(graph, &[Value::Int(2), Value::Int(0)], &registry), 0);
    // Unknown * anything = Unknown
    assert_eq!(eval_int(graph, &[Value::Int(5), Value::Int(2)], &registry), 5);
    // Const * X = X
    assert_eq!(eval_int(graph, &[Value::Int(1), Value::Int(3)], &registry), 3);
    // Linear * Linear = Polynomial (2 + 2 = 4)
    assert_eq!(eval_int(graph, &[Value::Int(2), Value::Int(2)], &registry), 4);
}

#[test]
fn test_verify_cost() {
    let src = load_checker_source("cost_check.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "verify_cost");

    // Small cost within large budget → pass
    assert_eq!(eval_int(graph, &[Value::Int(50), Value::Int(500)], &registry), 1);
    // Large cost within small budget → fail
    assert_eq!(eval_int(graph, &[Value::Int(500000), Value::Int(50)], &registry), 0);
    // Same kind → pass
    assert_eq!(eval_int(graph, &[Value::Int(200), Value::Int(300)], &registry), 1);
}

// ---------------------------------------------------------------------------
// Tests: lia_solver.iris
// ---------------------------------------------------------------------------

#[test]
fn test_eval_linear_term() {
    let src = load_checker_source("lia_solver.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "eval_linear_term");

    // 3 * 5 + 7 = 22
    assert_eq!(
        eval_int(graph, &[Value::Int(3), Value::Int(5), Value::Int(7)], &registry),
        22
    );
    // 0 * 10 + 0 = 0
    assert_eq!(
        eval_int(graph, &[Value::Int(0), Value::Int(10), Value::Int(0)], &registry),
        0
    );
}

#[test]
fn test_eval_eq() {
    let src = load_checker_source("lia_solver.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "eval_eq");

    assert_eq!(eval_int(graph, &[Value::Int(5), Value::Int(5)], &registry), 1);
    assert_eq!(eval_int(graph, &[Value::Int(5), Value::Int(3)], &registry), 0);
}

#[test]
fn test_eval_lt() {
    let src = load_checker_source("lia_solver.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "eval_lt");

    assert_eq!(eval_int(graph, &[Value::Int(3), Value::Int(5)], &registry), 1);
    assert_eq!(eval_int(graph, &[Value::Int(5), Value::Int(3)], &registry), 0);
    assert_eq!(eval_int(graph, &[Value::Int(5), Value::Int(5)], &registry), 0);
}

#[test]
fn test_eval_le() {
    let src = load_checker_source("lia_solver.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "eval_le");

    assert_eq!(eval_int(graph, &[Value::Int(3), Value::Int(5)], &registry), 1);
    assert_eq!(eval_int(graph, &[Value::Int(5), Value::Int(5)], &registry), 1);
    assert_eq!(eval_int(graph, &[Value::Int(6), Value::Int(5)], &registry), 0);
}

#[test]
fn test_eval_divisible() {
    let src = load_checker_source("lia_solver.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "eval_divisible");

    // 12 divisible by 3 → 1
    assert_eq!(eval_int(graph, &[Value::Int(12), Value::Int(3)], &registry), 1);
    // 13 not divisible by 3 → 0
    assert_eq!(eval_int(graph, &[Value::Int(13), Value::Int(3)], &registry), 0);
    // Divisible by 0 → 0
    assert_eq!(eval_int(graph, &[Value::Int(12), Value::Int(0)], &registry), 0);
}

#[test]
fn test_eval_and() {
    let src = load_checker_source("lia_solver.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "eval_and");

    assert_eq!(eval_int(graph, &[Value::Int(1), Value::Int(1)], &registry), 1);
    assert_eq!(eval_int(graph, &[Value::Int(1), Value::Int(0)], &registry), 0);
    assert_eq!(eval_int(graph, &[Value::Int(0), Value::Int(1)], &registry), 0);
}

#[test]
fn test_eval_or() {
    let src = load_checker_source("lia_solver.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "eval_or");

    assert_eq!(eval_int(graph, &[Value::Int(0), Value::Int(0)], &registry), 0);
    assert_eq!(eval_int(graph, &[Value::Int(1), Value::Int(0)], &registry), 1);
    assert_eq!(eval_int(graph, &[Value::Int(0), Value::Int(1)], &registry), 1);
}

#[test]
fn test_eval_not() {
    let src = load_checker_source("lia_solver.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "eval_not");

    assert_eq!(eval_int(graph, &[Value::Int(1)], &registry), 0);
    assert_eq!(eval_int(graph, &[Value::Int(0)], &registry), 1);
}

#[test]
fn test_eval_implies() {
    let src = load_checker_source("lia_solver.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "eval_implies");

    // T → T = T
    assert_eq!(eval_int(graph, &[Value::Int(1), Value::Int(1)], &registry), 1);
    // T → F = F
    assert_eq!(eval_int(graph, &[Value::Int(1), Value::Int(0)], &registry), 0);
    // F → anything = T
    assert_eq!(eval_int(graph, &[Value::Int(0), Value::Int(0)], &registry), 1);
    assert_eq!(eval_int(graph, &[Value::Int(0), Value::Int(1)], &registry), 1);
}

#[test]
fn test_check_bounds() {
    let src = load_checker_source("lia_solver.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "check_bounds");

    // 5 within [1, 10] → 1
    assert_eq!(
        eval_int(graph, &[Value::Int(5), Value::Int(1), Value::Int(10)], &registry),
        1
    );
    // 0 not within [1, 10] → 0
    assert_eq!(
        eval_int(graph, &[Value::Int(0), Value::Int(1), Value::Int(10)], &registry),
        0
    );
    // Boundary: 1 within [1, 10] → 1
    assert_eq!(
        eval_int(graph, &[Value::Int(1), Value::Int(1), Value::Int(10)], &registry),
        1
    );
}

// ---------------------------------------------------------------------------
// Tests: zk_verify.iris
// ---------------------------------------------------------------------------

#[test]
fn test_fiat_shamir_challenge() {
    let src = load_checker_source("zk_verify.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "fiat_shamir_challenge");

    // Deterministic: same inputs → same output
    let r1 = eval_int(graph, &[Value::Int(42), Value::Int(7)], &registry);
    let r2 = eval_int(graph, &[Value::Int(42), Value::Int(7)], &registry);
    assert_eq!(r1, r2, "fiat_shamir_challenge should be deterministic");
    // Result must be non-negative
    assert!(r1 >= 0, "challenge should be non-negative");
}

#[test]
fn test_verify_merkle_path_leaf_equals_root() {
    let src = load_checker_source("zk_verify.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "verify_merkle_path");

    // path_length 0, leaf == root → valid
    assert_eq!(
        eval_int(graph, &[Value::Int(123), Value::Int(123), Value::Int(0)], &registry),
        1
    );
    // path_length 0, leaf != root → invalid
    assert_eq!(
        eval_int(graph, &[Value::Int(123), Value::Int(456), Value::Int(0)], &registry),
        0
    );
}

#[test]
fn test_verify_merkle_path_depth() {
    let src = load_checker_source("zk_verify.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "verify_merkle_path");

    // Valid path_length (within max_depth 32) → valid (delegated to Rust)
    assert_eq!(
        eval_int(graph, &[Value::Int(100), Value::Int(200), Value::Int(5)], &registry),
        1
    );
    // path_length > 32 → invalid
    assert_eq!(
        eval_int(graph, &[Value::Int(100), Value::Int(200), Value::Int(33)], &registry),
        0
    );
}

#[test]
fn test_check_challenge_count() {
    let src = load_checker_source("zk_verify.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "check_challenge_count");

    assert_eq!(eval_int(graph, &[Value::Int(128), Value::Int(128)], &registry), 1);
    assert_eq!(eval_int(graph, &[Value::Int(128), Value::Int(64)], &registry), 0);
}

#[test]
fn test_verify_public_inputs() {
    let src = load_checker_source("zk_verify.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "verify_public_inputs");

    // Compute the expected hash using the same algorithm
    let program_hash = 10;
    let spec_hash = 20;
    let cost_hash = 30;
    let combined = program_hash + spec_hash * 31 + cost_hash * 961;
    // fiat_shamir_challenge(combined, 0)
    let mixed = combined * 31;
    let h = mixed * 2654435761_i64;
    let expected = if h < 0 { -h } else { h };

    // Matching hash → 1
    assert_eq!(
        eval_int(
            graph,
            &[Value::Int(program_hash), Value::Int(spec_hash), Value::Int(cost_hash), Value::Int(expected)],
            &registry,
        ),
        1
    );
    // Wrong hash → 0
    assert_eq!(
        eval_int(
            graph,
            &[Value::Int(program_hash), Value::Int(spec_hash), Value::Int(cost_hash), Value::Int(9999)],
            &registry,
        ),
        0
    );
}

#[test]
fn test_verify_zk_proof_valid() {
    let src = load_checker_source("zk_verify.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "verify_zk_proof");

    // proof_tree_size=4, security_param=128, root_hash=100, leaf_hash=100
    // depth for size 4 = 2, merkle_path with path_length 2 and any hashes → valid (delegated)
    // challenge_count: security_param == security_param → valid
    let outputs = eval(
        graph,
        &[Value::Int(4), Value::Int(128), Value::Int(100), Value::Int(50)],
        &registry,
    );
    // Returns (is_valid, challenge_count, merkle_depth)
    match outputs.first() {
        Some(Value::Tuple(tuple)) => {
            assert_eq!(tuple.len(), 3, "expected 3-tuple output");
            // is_valid should be 1 (merkle path_length=2 > 0 and non-leaf → 1, challenge_ok=1)
            assert_eq!(tuple[0], Value::Int(1), "proof should be valid");
            // challenge_count = security_param
            assert_eq!(tuple[1], Value::Int(128));
            // depth for size 4 = 2
            assert_eq!(tuple[2], Value::Int(2));
        }
        other => panic!("expected Tuple output, got: {:?}", other),
    }
}

#[test]
fn test_verify_zk_proof_single_node() {
    let src = load_checker_source("zk_verify.iris");
    let (fragments, registry) = compile_with_registry(&src);
    let graph = find_fragment(&fragments, "verify_zk_proof");

    // proof_tree_size=1, leaf == root (both 42) → depth 0, merkle check: leaf==root → 1
    let outputs = eval(
        graph,
        &[Value::Int(1), Value::Int(64), Value::Int(42), Value::Int(42)],
        &registry,
    );
    match outputs.first() {
        Some(Value::Tuple(tuple)) => {
            assert_eq!(tuple[0], Value::Int(1), "single-node proof with matching hashes should be valid");
            assert_eq!(tuple[1], Value::Int(64));
            assert_eq!(tuple[2], Value::Int(0), "depth should be 0 for size 1");
        }
        other => panic!("expected Tuple output, got: {:?}", other),
    }
}
