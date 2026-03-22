
//! Test harness for iris interpreter .iris programs.
//!
//! Loads each interpreter .iris file from src/iris-programs/interpreter/,
//! compiles it via iris_bootstrap::syntax::compile(), registers all fragments
//! in a FragmentRegistry, then evaluates key entry points through the Rust
//! interpreter with the registry. Asserts correct output for each.

use std::collections::{BTreeMap, HashMap};

use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
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

/// Load an interpreter .iris file and compile it.
fn load_interpreter_file(filename: &str) -> (Vec<(String, SemanticGraph)>, FragmentRegistry) {
    let path = format!("src/iris-programs/interpreter/{}", filename);
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {}: {}", path, e));
    compile_with_registry(&source)
}

/// Find a named fragment or panic.
fn find_fragment<'a>(
    fragments: &'a [(String, SemanticGraph)],
    name: &str,
) -> &'a SemanticGraph {
    fragments
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, g)| g)
        .unwrap_or_else(|| {
            let names: Vec<&str> = fragments.iter().map(|(n, _)| n.as_str()).collect();
            panic!("fragment '{}' not found in {:?}", name, names)
        })
}

/// Evaluate a graph with inputs and return the output values.
fn eval_with_inputs(
    graph: &SemanticGraph,
    inputs: &[Value],
    registry: &FragmentRegistry,
) -> Vec<Value> {
    let (outputs, _) = interpreter::interpret_with_registry(graph, inputs, None, Some(registry))
        .unwrap_or_else(|e| panic!("interpret failed: {:?}", e));
    outputs
}

/// Evaluate a graph with no inputs and return the first output as i64.
fn eval_int(graph: &SemanticGraph, inputs: &[Value], registry: &FragmentRegistry) -> i64 {
    let outputs = eval_with_inputs(graph, inputs, registry);
    match outputs.first() {
        Some(Value::Int(n)) => *n,
        Some(Value::Bool(true)) => 1,
        Some(Value::Bool(false)) => 0,
        other => panic!("expected Int output, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Graph construction helpers (for Program inputs)
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
            resolution_depth: 0,
            salt: 0,
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

fn prim_node(id: u64, opcode: u8, arity: u8) -> (NodeId, Node) {
    make_node(id, NodeKind::Prim, NodePayload::Prim { opcode }, arity)
}

/// Lit(value) -- a constant program.
fn make_lit_program(value: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = int_lit_node(1, value);
    nodes.insert(nid, node);
    make_graph(nodes, vec![], 1)
}

/// op(Lit(a), Lit(b)) -- binary prim on two constants.
fn make_binop_program(opcode: u8, a: i64, b: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = prim_node(1, opcode, 2);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(10, a);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(20, b);
    nodes.insert(nid, node);
    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];
    make_graph(nodes, edges, 1)
}

// ===========================================================================
// eval_lit.iris
// ===========================================================================

#[test]
fn test_eval_lit() {
    let (frags, registry) = load_interpreter_file("eval_lit.iris");
    let graph = find_fragment(&frags, "eval_lit");

    // Lit(42) -> 42
    let target = make_lit_program(42);
    let out = eval_with_inputs(
        graph,
        &[Value::Program(Box::new(target))],
        &registry,
    );
    assert_eq!(out[0], Value::Int(42), "eval_lit on Lit(42) should return 42");

    // Lit(0) -> 0
    let target = make_lit_program(0);
    let out = eval_with_inputs(
        graph,
        &[Value::Program(Box::new(target))],
        &registry,
    );
    assert_eq!(out[0], Value::Int(0), "eval_lit on Lit(0) should return 0");

    // Lit(-7) -> -7
    let target = make_lit_program(-7);
    let out = eval_with_inputs(
        graph,
        &[Value::Program(Box::new(target))],
        &registry,
    );
    assert_eq!(out[0], Value::Int(-7), "eval_lit on Lit(-7) should return -7");
}

// ===========================================================================
// eval_prim.iris
// ===========================================================================

#[test]
fn test_eval_prim_arith() {
    let (frags, registry) = load_interpreter_file("eval_prim.iris");
    let graph = find_fragment(&frags, "eval_prim_arith");

    // add(3, 5) = 8 (opcode 0)
    assert_eq!(
        eval_int(graph, &[Value::Int(0), Value::Int(3), Value::Int(5)], &registry),
        8,
        "add(3, 5) = 8"
    );
    // sub(10, 3) = 7 (opcode 1)
    assert_eq!(
        eval_int(graph, &[Value::Int(1), Value::Int(10), Value::Int(3)], &registry),
        7,
        "sub(10, 3) = 7"
    );
    // mul(6, 7) = 42 (opcode 2)
    assert_eq!(
        eval_int(graph, &[Value::Int(2), Value::Int(6), Value::Int(7)], &registry),
        42,
        "mul(6, 7) = 42"
    );
    // div(20, 4) = 5 (opcode 3)
    assert_eq!(
        eval_int(graph, &[Value::Int(3), Value::Int(20), Value::Int(4)], &registry),
        5,
        "div(20, 4) = 5"
    );
    // mod(17, 5) = 2 (opcode 4)
    assert_eq!(
        eval_int(graph, &[Value::Int(4), Value::Int(17), Value::Int(5)], &registry),
        2,
        "mod(17, 5) = 2"
    );
}

#[test]
fn test_eval_prim_cmp() {
    let (frags, registry) = load_interpreter_file("eval_prim.iris");
    let graph = find_fragment(&frags, "eval_prim_cmp");

    // eq(5, 5) = 1 (opcode 0x20 = 32)
    assert_eq!(
        eval_int(graph, &[Value::Int(32), Value::Int(5), Value::Int(5)], &registry),
        1,
        "eq(5, 5) = 1"
    );
    // eq(5, 3) = 0
    assert_eq!(
        eval_int(graph, &[Value::Int(32), Value::Int(5), Value::Int(3)], &registry),
        0,
        "eq(5, 3) = 0"
    );
    // lt(3, 5) = 1 (opcode 0x22 = 34)
    assert_eq!(
        eval_int(graph, &[Value::Int(34), Value::Int(3), Value::Int(5)], &registry),
        1,
        "lt(3, 5) = 1"
    );
    // gt(5, 3) = 1 (opcode 0x23 = 35)
    assert_eq!(
        eval_int(graph, &[Value::Int(35), Value::Int(5), Value::Int(3)], &registry),
        1,
        "gt(5, 3) = 1"
    );
}

// ===========================================================================
// eval_lambda.iris
// ===========================================================================

#[test]
fn test_eval_apply() {
    let (frags, registry) = load_interpreter_file("eval_lambda.iris");
    let graph = find_fragment(&frags, "eval_apply");

    // identity(7) -> 7
    assert_eq!(eval_int(graph, &[Value::Int(0), Value::Int(7)], &registry), 7);
    // double(5) -> 10
    assert_eq!(eval_int(graph, &[Value::Int(1), Value::Int(5)], &registry), 10);
    // square(4) -> 16
    assert_eq!(eval_int(graph, &[Value::Int(2), Value::Int(4)], &registry), 16);
    // negate(3) -> -3
    assert_eq!(eval_int(graph, &[Value::Int(3), Value::Int(3)], &registry), -3);
    // increment(9) -> 10
    assert_eq!(eval_int(graph, &[Value::Int(4), Value::Int(9)], &registry), 10);
}

#[test]
fn test_eval_compose() {
    let (frags, registry) = load_interpreter_file("eval_lambda.iris");
    let graph = find_fragment(&frags, "eval_compose");

    // compose(double, identity, 5) = double(identity(5)) = 10
    assert_eq!(
        eval_int(graph, &[Value::Int(1), Value::Int(0), Value::Int(5)], &registry),
        10
    );
    // compose(square, double, 3) = square(double(3)) = square(6) = 36
    assert_eq!(
        eval_int(graph, &[Value::Int(2), Value::Int(1), Value::Int(3)], &registry),
        36
    );
}

#[test]
fn test_eval_partial_apply() {
    let (frags, registry) = load_interpreter_file("eval_lambda.iris");
    let graph = find_fragment(&frags, "eval_partial_apply");

    // partial_apply(add, 10, 5) = 5 + 10 = 15
    assert_eq!(
        eval_int(graph, &[Value::Int(0), Value::Int(10), Value::Int(5)], &registry),
        15
    );
    // partial_apply(mul, 3, 7) = 7 * 3 = 21
    assert_eq!(
        eval_int(graph, &[Value::Int(2), Value::Int(3), Value::Int(7)], &registry),
        21
    );
}

// ===========================================================================
// eval_let.iris
// ===========================================================================

#[test]
fn test_eval_let() {
    let (frags, registry) = load_interpreter_file("eval_let.iris");

    // eval_let_identity(42) = 42
    let g = find_fragment(&frags, "eval_let_identity");
    assert_eq!(eval_int(g, &[Value::Int(42)], &registry), 42);

    // eval_let_double(5) = 10
    let g = find_fragment(&frags, "eval_let_double");
    assert_eq!(eval_int(g, &[Value::Int(5)], &registry), 10);

    // eval_let_add(3, 4) = 7
    let g = find_fragment(&frags, "eval_let_add");
    assert_eq!(eval_int(g, &[Value::Int(3), Value::Int(4)], &registry), 7);

    // eval_let_shadow(10, 20) = 20 (inner shadows outer)
    let g = find_fragment(&frags, "eval_let_shadow");
    assert_eq!(eval_int(g, &[Value::Int(10), Value::Int(20)], &registry), 20);

    // eval_let_poly(3) = 3*3 + 3 = 12
    let g = find_fragment(&frags, "eval_let_poly");
    assert_eq!(eval_int(g, &[Value::Int(3)], &registry), 12);
}

// ===========================================================================
// eval_tuple.iris
// ===========================================================================

#[test]
fn test_eval_tuple_pair() {
    let (frags, registry) = load_interpreter_file("eval_tuple.iris");
    let graph = find_fragment(&frags, "eval_tuple_pair");

    let out = eval_with_inputs(graph, &[Value::Int(3), Value::Int(7)], &registry);
    assert_eq!(
        out[0],
        Value::tuple(vec![Value::Int(3), Value::Int(7)]),
        "eval_tuple_pair(3, 7) = (3, 7)"
    );
}

#[test]
fn test_eval_tuple_triple() {
    let (frags, registry) = load_interpreter_file("eval_tuple.iris");
    let graph = find_fragment(&frags, "eval_tuple_triple");

    let out = eval_with_inputs(graph, &[Value::Int(1), Value::Int(2), Value::Int(3)], &registry);
    assert_eq!(
        out[0],
        Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
        "eval_tuple_triple(1, 2, 3) = (1, 2, 3)"
    );
}

#[test]
fn test_eval_tuple_fst_snd() {
    let (frags, registry) = load_interpreter_file("eval_tuple.iris");

    let fst = find_fragment(&frags, "eval_tuple_fst");
    let pair = Value::tuple(vec![Value::Int(10), Value::Int(20)]);
    assert_eq!(eval_int(fst, &[pair.clone()], &registry), 10);

    let snd = find_fragment(&frags, "eval_tuple_snd");
    assert_eq!(eval_int(snd, &[pair], &registry), 20);
}

#[test]
fn test_eval_tuple_swap() {
    let (frags, registry) = load_interpreter_file("eval_tuple.iris");
    let graph = find_fragment(&frags, "eval_tuple_swap");

    let pair = Value::tuple(vec![Value::Int(1), Value::Int(2)]);
    let out = eval_with_inputs(graph, &[pair], &registry);
    assert_eq!(
        out[0],
        Value::tuple(vec![Value::Int(2), Value::Int(1)]),
        "eval_tuple_swap((1, 2)) = (2, 1)"
    );
}

// ===========================================================================
// eval_match.iris
// ===========================================================================

#[test]
fn test_eval_match() {
    let (frags, registry) = load_interpreter_file("eval_match.iris");
    let graph = find_fragment(&frags, "eval_match");

    // scrutinee=0 -> arm0=10
    assert_eq!(
        eval_int(
            graph,
            &[Value::Int(0), Value::Int(10), Value::Int(20), Value::Int(99)],
            &registry
        ),
        10
    );
    // scrutinee=1 -> arm1=20
    assert_eq!(
        eval_int(
            graph,
            &[Value::Int(1), Value::Int(10), Value::Int(20), Value::Int(99)],
            &registry
        ),
        20
    );
    // scrutinee=5 -> default=99
    assert_eq!(
        eval_int(
            graph,
            &[Value::Int(5), Value::Int(10), Value::Int(20), Value::Int(99)],
            &registry
        ),
        99
    );
}

#[test]
fn test_eval_match_bool() {
    let (frags, registry) = load_interpreter_file("eval_match.iris");
    let graph = find_fragment(&frags, "eval_match_bool");

    // cond=0 (false) -> false_branch=100
    assert_eq!(
        eval_int(graph, &[Value::Int(0), Value::Int(100), Value::Int(200)], &registry),
        100
    );
    // cond=1 (true) -> true_branch=200
    assert_eq!(
        eval_int(graph, &[Value::Int(1), Value::Int(100), Value::Int(200)], &registry),
        200
    );
}

#[test]
fn test_eval_match_wildcard() {
    let (frags, registry) = load_interpreter_file("eval_match.iris");
    let graph = find_fragment(&frags, "eval_match_wildcard");

    // scrutinee matches known_tag -> known_result
    assert_eq!(
        eval_int(graph, &[Value::Int(5), Value::Int(5), Value::Int(99)], &registry),
        99
    );
    // scrutinee doesn't match -> returns scrutinee itself
    assert_eq!(
        eval_int(graph, &[Value::Int(7), Value::Int(5), Value::Int(99)], &registry),
        7
    );
}

// ===========================================================================
// eval_fold.iris
// ===========================================================================

#[test]
fn test_eval_fold() {
    let (frags, registry) = load_interpreter_file("eval_fold.iris");
    let graph = find_fragment(&frags, "eval_fold");

    // fold add over (1, 2, 3) with acc=0: 0 + 1 + 2 + 3 = 6
    let collection = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    assert_eq!(
        eval_int(graph, &[collection.clone(), Value::Int(0), Value::Int(0)], &registry),
        6,
        "fold add (1,2,3) acc=0 = 6"
    );

    // fold mul over (2, 3, 4) with acc=1: 1 * 2 * 3 * 4 = 24
    let collection = Value::tuple(vec![Value::Int(2), Value::Int(3), Value::Int(4)]);
    assert_eq!(
        eval_int(graph, &[collection, Value::Int(1), Value::Int(2)], &registry),
        24,
        "fold mul (2,3,4) acc=1 = 24"
    );
}

#[test]
fn test_eval_fold_pair() {
    let (frags, registry) = load_interpreter_file("eval_fold.iris");
    let graph = find_fragment(&frags, "eval_fold_pair");

    // fold add over (3, 4) with acc=0: 0 + 3 + 4 = 7
    assert_eq!(
        eval_int(
            graph,
            &[Value::Int(3), Value::Int(4), Value::Int(0), Value::Int(0)],
            &registry
        ),
        7,
        "fold_pair add (3,4) acc=0 = 7"
    );

    // fold mul over (5, 6) with acc=1: 1 * 5 * 6 = 30
    assert_eq!(
        eval_int(
            graph,
            &[Value::Int(5), Value::Int(6), Value::Int(1), Value::Int(2)],
            &registry
        ),
        30,
        "fold_pair mul (5,6) acc=1 = 30"
    );
}

// ===========================================================================
// eval_guard.iris
// ===========================================================================

#[test]
fn test_eval_guard() {
    let (frags, registry) = load_interpreter_file("eval_guard.iris");
    let graph = find_fragment(&frags, "eval_guard");

    // predicate=1 (true) -> body=42
    assert_eq!(
        eval_int(graph, &[Value::Int(1), Value::Int(42), Value::Int(99)], &registry),
        42
    );
    // predicate=0 (false) -> fallback=99
    assert_eq!(
        eval_int(graph, &[Value::Int(0), Value::Int(42), Value::Int(99)], &registry),
        99
    );
}

#[test]
fn test_eval_guard_chain() {
    let (frags, registry) = load_interpreter_file("eval_guard.iris");
    let graph = find_fragment(&frags, "eval_guard_chain");

    // p1=1 -> body1=10
    assert_eq!(
        eval_int(
            graph,
            &[Value::Int(1), Value::Int(10), Value::Int(0), Value::Int(20), Value::Int(30)],
            &registry
        ),
        10
    );
    // p1=0, p2=1 -> body2=20
    assert_eq!(
        eval_int(
            graph,
            &[Value::Int(0), Value::Int(10), Value::Int(1), Value::Int(20), Value::Int(30)],
            &registry
        ),
        20
    );
    // p1=0, p2=0 -> fallback=30
    assert_eq!(
        eval_int(
            graph,
            &[Value::Int(0), Value::Int(10), Value::Int(0), Value::Int(20), Value::Int(30)],
            &registry
        ),
        30
    );
}

#[test]
fn test_eval_guard_clamp() {
    let (frags, registry) = load_interpreter_file("eval_guard.iris");
    let graph = find_fragment(&frags, "eval_guard_clamp");

    // value=5, lo=0, hi=10 -> 5 (in range)
    assert_eq!(
        eval_int(graph, &[Value::Int(5), Value::Int(0), Value::Int(10)], &registry),
        5
    );
    // value=-3, lo=0, hi=10 -> 0 (below range)
    assert_eq!(
        eval_int(graph, &[Value::Int(-3), Value::Int(0), Value::Int(10)], &registry),
        0
    );
    // value=15, lo=0, hi=10 -> 10 (above range)
    assert_eq!(
        eval_int(graph, &[Value::Int(15), Value::Int(0), Value::Int(10)], &registry),
        10
    );
}

// ===========================================================================
// eval_project.iris
// ===========================================================================

#[test]
fn test_eval_project() {
    let (frags, registry) = load_interpreter_file("eval_project.iris");

    // fst(3, 7) = 3
    let g = find_fragment(&frags, "eval_project_fst");
    assert_eq!(eval_int(g, &[Value::Int(3), Value::Int(7)], &registry), 3);

    // snd(3, 7) = 7
    let g = find_fragment(&frags, "eval_project_snd");
    assert_eq!(eval_int(g, &[Value::Int(3), Value::Int(7)], &registry), 7);

    // mid(1, 2, 3) = 2
    let g = find_fragment(&frags, "eval_project_mid");
    assert_eq!(
        eval_int(g, &[Value::Int(1), Value::Int(2), Value::Int(3)], &registry),
        2
    );

    // sum(4, 6) = 10
    let g = find_fragment(&frags, "eval_project_sum");
    assert_eq!(eval_int(g, &[Value::Int(4), Value::Int(6)], &registry), 10);

    // nested(1, 2, 3, 4) = 3
    let g = find_fragment(&frags, "eval_project_nested");
    assert_eq!(
        eval_int(
            g,
            &[Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)],
            &registry
        ),
        3
    );
}

// ===========================================================================
// eval_inject.iris
// ===========================================================================

#[test]
fn test_eval_inject() {
    let (frags, registry) = load_interpreter_file("eval_inject.iris");

    // inject_left(42) = 42
    let g = find_fragment(&frags, "eval_inject_left");
    assert_eq!(eval_int(g, &[Value::Int(42)], &registry), 42);

    // inject_right(7) = 7
    let g = find_fragment(&frags, "eval_inject_right");
    assert_eq!(eval_int(g, &[Value::Int(7)], &registry), 7);

    // inject_match(0, 10) = 10 + 1 = 11  (Left case)
    let g = find_fragment(&frags, "eval_inject_match");
    assert_eq!(eval_int(g, &[Value::Int(0), Value::Int(10)], &registry), 11);
    // inject_match(1, 10) = 10 * 2 = 20  (Right case)
    assert_eq!(eval_int(g, &[Value::Int(1), Value::Int(10)], &registry), 20);

    // inject_roundtrip(0, 42, 99) = 42
    let g = find_fragment(&frags, "eval_inject_roundtrip");
    assert_eq!(
        eval_int(g, &[Value::Int(0), Value::Int(42), Value::Int(99)], &registry),
        42
    );
    // inject_roundtrip(1, 42, 99) = 99
    assert_eq!(
        eval_int(g, &[Value::Int(1), Value::Int(42), Value::Int(99)], &registry),
        99
    );
}

// ===========================================================================
// eval_unfold.iris
// ===========================================================================

#[test]
fn test_eval_unfold() {
    let (frags, registry) = load_interpreter_file("eval_unfold.iris");

    // unfold_count(10, 0) = 0
    let g = find_fragment(&frags, "eval_unfold_count");
    assert_eq!(eval_int(g, &[Value::Int(10), Value::Int(0)], &registry), 0);

    // unfold_count(1, 3) = 1 + 1 + 3 - 1 = 4
    assert_eq!(eval_int(g, &[Value::Int(1), Value::Int(3)], &registry), 4);

    // unfold_gcd(12, 8) = 4
    let g = find_fragment(&frags, "eval_unfold_gcd");
    assert_eq!(eval_int(g, &[Value::Int(12), Value::Int(8)], &registry), 4);

    // unfold_gcd(15, 0) = 15
    assert_eq!(eval_int(g, &[Value::Int(15), Value::Int(0)], &registry), 15);

    // unfold_power(2, 0) = 1
    let g = find_fragment(&frags, "eval_unfold_power");
    assert_eq!(eval_int(g, &[Value::Int(2), Value::Int(0)], &registry), 1);

    // unfold_power(2, 1) = 2
    assert_eq!(eval_int(g, &[Value::Int(2), Value::Int(1)], &registry), 2);

    // unfold_power(2, 3) = 8
    assert_eq!(eval_int(g, &[Value::Int(2), Value::Int(3)], &registry), 8);

    // unfold_power(3, 2) = 9
    assert_eq!(eval_int(g, &[Value::Int(3), Value::Int(2)], &registry), 9);
}

// ===========================================================================
// eval_ref.iris
// ===========================================================================

#[test]
fn test_eval_ref() {
    let (frags, registry) = load_interpreter_file("eval_ref.iris");

    // ref_call(0, 7) = identity(7) = 7
    let g = find_fragment(&frags, "eval_ref_call");
    assert_eq!(eval_int(g, &[Value::Int(0), Value::Int(7)], &registry), 7);

    // ref_call(1, 5) = double(5) = 10
    assert_eq!(eval_int(g, &[Value::Int(1), Value::Int(5)], &registry), 10);

    // ref_call(2, 4) = square(4) = 16
    assert_eq!(eval_int(g, &[Value::Int(2), Value::Int(4)], &registry), 16);

    // ref_compose(1, 2, 3) = double(square(3)) = double(9) = 18
    let g = find_fragment(&frags, "eval_ref_compose");
    assert_eq!(
        eval_int(g, &[Value::Int(1), Value::Int(2), Value::Int(3)], &registry),
        18
    );
}

// ===========================================================================
// eval_rewrite.iris
// ===========================================================================

#[test]
fn test_eval_rewrite() {
    let (frags, registry) = load_interpreter_file("eval_rewrite.iris");

    // eval_rewrite(42) = 42 (transparent)
    let g = find_fragment(&frags, "eval_rewrite");
    assert_eq!(eval_int(g, &[Value::Int(42)], &registry), 42);

    // eval_rewrite_tagged(999, 7) = 7 (tag ignored)
    let g = find_fragment(&frags, "eval_rewrite_tagged");
    assert_eq!(eval_int(g, &[Value::Int(999), Value::Int(7)], &registry), 7);

    // eval_rewrite_chain(100) = 100 (multiple rewrites transparent)
    let g = find_fragment(&frags, "eval_rewrite_chain");
    assert_eq!(eval_int(g, &[Value::Int(100)], &registry), 100);
}

// ===========================================================================
// eval_effect.iris
// ===========================================================================

#[test]
fn test_eval_effect() {
    let (frags, registry) = load_interpreter_file("eval_effect.iris");

    // eval_effect_log(42) = 42 (no-op in pure mode)
    let g = find_fragment(&frags, "eval_effect_log");
    assert_eq!(eval_int(g, &[Value::Int(42)], &registry), 42);

    // eval_effect_store_load(1, 99) = 99
    let g = find_fragment(&frags, "eval_effect_store_load");
    assert_eq!(eval_int(g, &[Value::Int(1), Value::Int(99)], &registry), 99);

    // eval_effect_random(10, 20) = 10 + (20-10)/2 = 15
    let g = find_fragment(&frags, "eval_effect_random");
    assert_eq!(eval_int(g, &[Value::Int(10), Value::Int(20)], &registry), 15);

    // eval_effect_random(5, 5) = 5 (hi == lo)
    assert_eq!(eval_int(g, &[Value::Int(5), Value::Int(5)], &registry), 5);
}

// ===========================================================================
// mini_interpreter.iris
// ===========================================================================

#[test]
fn test_mini_interpreter() {
    let (frags, registry) = load_interpreter_file("mini_interpreter.iris");
    let graph = find_fragment(&frags, "interpret");

    // Lit(42) -> 42
    let target = make_lit_program(42);
    let out = eval_with_inputs(
        graph,
        &[
            Value::Program(Box::new(target)),
            Value::tuple(vec![]),
        ],
        &registry,
    );
    assert_eq!(out[0], Value::Int(42), "mini_interpreter on Lit(42) = 42");

    // add(3, 5) -> 8
    let target = make_binop_program(0x00, 3, 5);
    let out = eval_with_inputs(
        graph,
        &[
            Value::Program(Box::new(target)),
            Value::tuple(vec![]),
        ],
        &registry,
    );
    assert_eq!(out[0], Value::Int(8), "mini_interpreter on add(3, 5) = 8");

    // mul(6, 7) -> 42
    let target = make_binop_program(0x02, 6, 7);
    let out = eval_with_inputs(
        graph,
        &[
            Value::Program(Box::new(target)),
            Value::tuple(vec![]),
        ],
        &registry,
    );
    assert_eq!(out[0], Value::Int(42), "mini_interpreter on mul(6, 7) = 42");
}

// ===========================================================================
// full_interpreter.iris
// ===========================================================================

#[test]
fn test_full_interpreter_lit() {
    let (frags, registry) = load_interpreter_file("full_interpreter.iris");
    let graph = find_fragment(&frags, "full_interpret");

    // Lit(42) -> 42
    let target = make_lit_program(42);
    let out = eval_with_inputs(
        graph,
        &[
            Value::Program(Box::new(target)),
            Value::tuple(vec![]),
        ],
        &registry,
    );
    assert_eq!(out[0], Value::Int(42), "full_interpret on Lit(42) = 42");

    // Lit(0) -> 0
    let target = make_lit_program(0);
    let out = eval_with_inputs(
        graph,
        &[
            Value::Program(Box::new(target)),
            Value::tuple(vec![]),
        ],
        &registry,
    );
    assert_eq!(out[0], Value::Int(0), "full_interpret on Lit(0) = 0");
}

#[test]
fn test_full_interpreter_arithmetic() {
    let (frags, registry) = load_interpreter_file("full_interpreter.iris");
    let graph = find_fragment(&frags, "full_interpret");

    // add(3, 5) -> 8
    let target = make_binop_program(0x00, 3, 5);
    let out = eval_with_inputs(
        graph,
        &[
            Value::Program(Box::new(target)),
            Value::tuple(vec![]),
        ],
        &registry,
    );
    assert_eq!(out[0], Value::Int(8), "full_interpret on add(3, 5) = 8");

    // sub(10, 3) -> 7
    let target = make_binop_program(0x01, 10, 3);
    let out = eval_with_inputs(
        graph,
        &[
            Value::Program(Box::new(target)),
            Value::tuple(vec![]),
        ],
        &registry,
    );
    assert_eq!(out[0], Value::Int(7), "full_interpret on sub(10, 3) = 7");

    // mul(6, 7) -> 42
    let target = make_binop_program(0x02, 6, 7);
    let out = eval_with_inputs(
        graph,
        &[
            Value::Program(Box::new(target)),
            Value::tuple(vec![]),
        ],
        &registry,
    );
    assert_eq!(out[0], Value::Int(42), "full_interpret on mul(6, 7) = 42");

    // div(20, 4) -> 5
    let target = make_binop_program(0x03, 20, 4);
    let out = eval_with_inputs(
        graph,
        &[
            Value::Program(Box::new(target)),
            Value::tuple(vec![]),
        ],
        &registry,
    );
    assert_eq!(out[0], Value::Int(5), "full_interpret on div(20, 4) = 5");
}

#[test]
fn test_full_interpreter_comparison() {
    let (frags, registry) = load_interpreter_file("full_interpreter.iris");
    let graph = find_fragment(&frags, "full_interpret");

    // eq(5, 5) -> 1  (opcode 0x20 = 32)
    let target = make_binop_program(0x20, 5, 5);
    let out = eval_with_inputs(
        graph,
        &[
            Value::Program(Box::new(target)),
            Value::tuple(vec![]),
        ],
        &registry,
    );
    match &out[0] {
        Value::Int(n) => assert_eq!(*n, 1, "full_interpret on eq(5, 5) = 1"),
        Value::Bool(b) => assert!(*b, "full_interpret on eq(5, 5) = true"),
        other => panic!("unexpected result: {:?}", other),
    }

    // lt(3, 5) -> 1  (opcode 0x22 = 34)
    let target = make_binop_program(0x22, 3, 5);
    let out = eval_with_inputs(
        graph,
        &[
            Value::Program(Box::new(target)),
            Value::tuple(vec![]),
        ],
        &registry,
    );
    match &out[0] {
        Value::Int(n) => assert_eq!(*n, 1, "full_interpret on lt(3, 5) = 1"),
        Value::Bool(b) => assert!(*b, "full_interpret on lt(3, 5) = true"),
        other => panic!("unexpected result: {:?}", other),
    }
}
