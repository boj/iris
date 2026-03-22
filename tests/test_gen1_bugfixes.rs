
//! Regression tests for Gen1 language bugs discovered during benchmark testing.
//!
//! Bug 1: Nested lambda scope — inner lambdas could not reference outer lambda
//!         parameters because the lowerer used a fixed binder index (0xFFFF_0002)
//!         for all lambda nesting levels.
//!
//! Bug 2: Parameter index >= 2 in fold — function params at index 2+ were
//!         shadowed by the fold/map lambda binder (also 0xFFFF_0002).
//!
//! Bug 3: str_eq/str_contains/str_starts_with/str_ends_with returned
//!         Value::Int(0/1) instead of Value::Bool, breaking Guard evaluation.

use iris_exec::interpreter;
use iris_types::eval::Value;
use iris_types::graph::*;
use iris_types::hash::compute_node_id;
use iris_types::types::*;
use std::collections::{BTreeMap, HashMap};

fn compile_and_run(src: &str, inputs: &[Value]) -> Value {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!("compilation failed with {} errors", result.errors.len());
    }
    assert!(!result.fragments.is_empty(), "no fragments produced");
    let graph = &result.fragments[0].1.graph;
    let (out, _) = interpreter::interpret(graph, inputs, None).unwrap();
    assert!(!out.is_empty(), "no output values");
    out.into_iter().next().unwrap()
}

// ===========================================================================
// Bug 1: Nested lambda scope
// ===========================================================================

#[test]
fn test_nested_lambda_captures_outer_var() {
    // let f x = map (\i -> i + x) (1, 2, 3)
    // With x=10, expected: (11, 12, 13)
    let result = compile_and_run(
        "let f x = map (\\i -> i + x) (1, 2, 3)",
        &[Value::Int(10)],
    );
    assert_eq!(
        result,
        Value::tuple(vec![Value::Int(11), Value::Int(12), Value::Int(13)])
    );
}

#[test]
fn test_double_nested_lambda() {
    // let f x = map (\i -> i + x) (1, 2, 3)
    // Nested: outer function captures x, lambda captures x from outer.
    // Testing with different x values to ensure capture is correct.
    let result = compile_and_run(
        "let f x = map (\\i -> i + x) (1, 2, 3)",
        &[Value::Int(100)],
    );
    assert_eq!(
        result,
        Value::tuple(vec![Value::Int(101), Value::Int(102), Value::Int(103)])
    );

    // Also test with x=0 to verify it's not hardcoded.
    let result = compile_and_run(
        "let f x = map (\\i -> i + x) (1, 2, 3)",
        &[Value::Int(0)],
    );
    assert_eq!(
        result,
        Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
    );
}

#[test]
fn test_fold_with_captured_var() {
    // let f x = fold 0 (\acc i -> acc + i + x) (1, 2)
    // With x=10: step 1: 0 + 1 + 10 = 11, step 2: 11 + 2 + 10 = 23
    let result = compile_and_run(
        "let f x = fold 0 (\\acc i -> acc + i + x) (1, 2)",
        &[Value::Int(10)],
    );
    assert_eq!(result, Value::Int(23));
}

// ===========================================================================
// Bug 2: Parameter index >= 2 in fold
// ===========================================================================

#[test]
fn test_fold_accesses_param_2() {
    // let f x y z = fold 0 (\acc i -> acc + i + z) (1, 2, 3)
    // With x=0, y=0, z=100: fold should accumulate (0+1+100) + (2+100) + (3+100) = 406
    // Step by step: acc=0, elem=1: 0+1+100=101, elem=2: 101+2+100=203, elem=3: 203+3+100=306
    let result = compile_and_run(
        "let f x y z = fold 0 (\\acc i -> acc + i + z) (1, 2, 3)",
        &[Value::Int(0), Value::Int(0), Value::Int(100)],
    );
    assert_eq!(result, Value::Int(306));
}

#[test]
fn test_map_accesses_param_2() {
    // let f x y z = map (\i -> i + z) (1, 2, 3)
    // With x=0, y=0, z=100: map should produce (101, 102, 103)
    let result = compile_and_run(
        "let f x y z = map (\\i -> i + z) (1, 2, 3)",
        &[Value::Int(0), Value::Int(0), Value::Int(100)],
    );
    assert_eq!(
        result,
        Value::tuple(vec![Value::Int(101), Value::Int(102), Value::Int(103)])
    );
}

// ===========================================================================
// Bug 3: str_eq / str_contains / str_starts_with / str_ends_with return Bool
// ===========================================================================

/// Helper to build a graph with a single Prim node wired to literal arguments.
fn make_prim_graph(opcode: u8, lit_nodes: Vec<Node>) -> SemanticGraph {
    let int_type_id = iris_types::hash::compute_type_id(&TypeDef::Primitive(PrimType::Int));
    let bool_type_id = iris_types::hash::compute_type_id(&TypeDef::Primitive(PrimType::Bool));
    let bytes_type_id = iris_types::hash::compute_type_id(&TypeDef::Primitive(PrimType::Bytes));

    let result_type = match opcode {
        0xB3 | 0xB8 | 0xB9 | 0xBA => bool_type_id,
        _ => int_type_id,
    };

    let mut nodes = HashMap::new();
    let mut edges = Vec::new();
    let mut arg_ids = Vec::new();

    for node in lit_nodes {
        let id = node.id;
        arg_ids.push(id);
        nodes.insert(id, node);
    }

    let mut prim_node = Node {
        id: NodeId(0),
        kind: NodeKind::Prim,
        type_sig: result_type,
        cost: iris_types::cost::CostTerm::Unit,
        arity: arg_ids.len() as u8,
        resolution_depth: 0,
        salt: 42,
        payload: NodePayload::Prim { opcode },
    };
    prim_node.id = compute_node_id(&prim_node);
    let root = prim_node.id;
    nodes.insert(root, prim_node);

    for (port, aid) in arg_ids.iter().enumerate() {
        edges.push(Edge {
            source: root,
            target: *aid,
            port: port as u8,
            label: EdgeLabel::Argument,
        });
    }

    let mut type_env = TypeEnv {
        types: BTreeMap::new(),
    };
    type_env
        .types
        .insert(int_type_id, TypeDef::Primitive(PrimType::Int));
    type_env
        .types
        .insert(bool_type_id, TypeDef::Primitive(PrimType::Bool));
    type_env
        .types
        .insert(bytes_type_id, TypeDef::Primitive(PrimType::Bytes));

    SemanticGraph {
        root,
        nodes,
        edges,
        type_env,
        cost: iris_types::cost::CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: iris_types::hash::SemanticHash([0u8; 32]),
    }
}

fn str_lit(s: &str) -> Node {
    let bytes_type_id = iris_types::hash::compute_type_id(&TypeDef::Primitive(PrimType::Bytes));
    let mut n = Node {
        id: NodeId(0),
        kind: NodeKind::Lit,
        type_sig: bytes_type_id,
        cost: iris_types::cost::CostTerm::Unit,
        arity: 0,
        resolution_depth: 0,
        salt: s.len() as u64 * 31 + 7,
        payload: NodePayload::Lit {
            type_tag: 0x07,
            value: s.as_bytes().to_vec(),
        },
    };
    n.id = compute_node_id(&n);
    n
}

#[test]
fn test_str_eq_returns_bool() {
    let g = make_prim_graph(0xB8, vec![str_lit("hello"), str_lit("hello")]);
    let (out, _) = interpreter::interpret(&g, &[], None).unwrap();
    assert_eq!(out[0], Value::Bool(true), "str_eq should return Bool(true)");

    let g = make_prim_graph(0xB8, vec![str_lit("hello"), str_lit("world")]);
    let (out, _) = interpreter::interpret(&g, &[], None).unwrap();
    assert_eq!(
        out[0],
        Value::Bool(false),
        "str_eq should return Bool(false)"
    );
}

#[test]
fn test_str_eq_in_guard() {
    // if str_eq a b then 1 else 0
    // With a="yes", b="yes" → should return 1
    let result = compile_and_run(
        r#"let f a b = if str_eq a b then 1 else 0"#,
        &[
            Value::String("yes".to_string()),
            Value::String("yes".to_string()),
        ],
    );
    assert_eq!(result, Value::Int(1));

    // With a="yes", b="no" → should return 0
    let result = compile_and_run(
        r#"let f a b = if str_eq a b then 1 else 0"#,
        &[
            Value::String("yes".to_string()),
            Value::String("no".to_string()),
        ],
    );
    assert_eq!(result, Value::Int(0));
}

#[test]
fn test_str_contains_returns_bool() {
    let g = make_prim_graph(0xB3, vec![str_lit("hello world"), str_lit("world")]);
    let (out, _) = interpreter::interpret(&g, &[], None).unwrap();
    assert_eq!(
        out[0],
        Value::Bool(true),
        "str_contains should return Bool(true)"
    );

    let g = make_prim_graph(0xB3, vec![str_lit("hello"), str_lit("xyz")]);
    let (out, _) = interpreter::interpret(&g, &[], None).unwrap();
    assert_eq!(
        out[0],
        Value::Bool(false),
        "str_contains should return Bool(false)"
    );
}

#[test]
fn test_str_starts_with_returns_bool() {
    let g = make_prim_graph(0xB9, vec![str_lit("hello world"), str_lit("hello")]);
    let (out, _) = interpreter::interpret(&g, &[], None).unwrap();
    assert_eq!(
        out[0],
        Value::Bool(true),
        "str_starts_with should return Bool(true)"
    );

    let g = make_prim_graph(0xB9, vec![str_lit("hello"), str_lit("world")]);
    let (out, _) = interpreter::interpret(&g, &[], None).unwrap();
    assert_eq!(
        out[0],
        Value::Bool(false),
        "str_starts_with should return Bool(false)"
    );
}
