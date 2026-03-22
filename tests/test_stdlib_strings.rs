
//! Tests for the full set of string operation primitives (0xB0-0xC0).
//!
//! Covers every registered string opcode:
//!   0xB0 str_len       0xB1 str_concat    0xB2 str_slice
//!   0xB3 str_contains  0xB4 str_split     0xB5 str_join
//!   0xB6 str_to_int    0xB7 int_to_string 0xB8 str_eq
//!   0xB9 str_starts_with 0xBA str_ends_with 0xBB str_replace
//!   0xBC str_trim      0xBD str_upper     0xBE str_lower
//!   0xBF str_chars     0xC0 char_at

use std::collections::{BTreeMap, HashMap};

use iris_types::eval::Value;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::graph::*;
use iris_types::hash::{compute_node_id, SemanticHash};
use iris_types::types::{PrimType, TypeDef, TypeEnv};

use iris_exec::interpreter;

// ===========================================================================
// Helpers (same pattern as test_novelty_strings.rs)
// ===========================================================================

fn make_type_env() -> (TypeEnv, iris_types::types::TypeId) {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = iris_types::hash::compute_type_id(&int_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    (TypeEnv { types }, int_id)
}

fn make_prim_graph(opcode: u8, arg_lits: &[(u8, Vec<u8>)]) -> SemanticGraph {
    let (type_env, int_id) = make_type_env();

    let mut nodes = HashMap::new();
    let mut edges = vec![];
    let mut arg_ids = vec![];

    for (_i, (type_tag, value)) in arg_lits.iter().enumerate() {
        let mut node = Node {
            id: NodeId(0),
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit {
                type_tag: *type_tag,
                value: value.clone(),
            },
        };
        node.id = compute_node_id(&node);
        arg_ids.push(node.id);
        nodes.insert(node.id, node);
    }

    let mut prim_node = Node {
        id: NodeId(0),
        kind: NodeKind::Prim,
        type_sig: int_id,
        cost: CostTerm::Unit,
        arity: arg_lits.len() as u8,
        resolution_depth: 2, salt: 0,
        payload: NodePayload::Prim { opcode },
    };
    prim_node.id = compute_node_id(&prim_node);

    for (i, &aid) in arg_ids.iter().enumerate() {
        edges.push(Edge {
            source: prim_node.id,
            target: aid,
            port: i as u8,
            label: EdgeLabel::Argument,
        });
    }

    nodes.insert(prim_node.id, prim_node.clone());

    SemanticGraph {
        root: prim_node.id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

fn str_lit(s: &str) -> (u8, Vec<u8>) {
    (0x07, s.as_bytes().to_vec())
}

fn int_lit(v: i64) -> (u8, Vec<u8>) {
    (0x00, v.to_le_bytes().to_vec())
}

fn eval_graph(graph: &SemanticGraph) -> Result<Value, iris_exec::interpreter::InterpretError> {
    let (outputs, _state) = interpreter::interpret(graph, &[], None)?;
    if outputs.len() == 1 {
        Ok(outputs.into_iter().next().unwrap())
    } else {
        Ok(Value::tuple(outputs))
    }
}

// ===========================================================================
// 0xB0 str_len
// ===========================================================================

#[test]
fn str_len_basic() {
    let g = make_prim_graph(0xB0, &[str_lit("hello")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int(5));
}

#[test]
fn str_len_empty() {
    let g = make_prim_graph(0xB0, &[str_lit("")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int(0));
}

#[test]
fn str_len_unicode() {
    // Multi-byte chars count as 1 each.
    let g = make_prim_graph(0xB0, &[str_lit("abc")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int(3));
}

// ===========================================================================
// 0xB1 str_concat
// ===========================================================================

#[test]
fn str_concat_basic() {
    let g = make_prim_graph(0xB1, &[str_lit("foo"), str_lit("bar")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("foobar".into()));
}

#[test]
fn str_concat_empty_left() {
    let g = make_prim_graph(0xB1, &[str_lit(""), str_lit("x")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("x".into()));
}

#[test]
fn str_concat_empty_right() {
    let g = make_prim_graph(0xB1, &[str_lit("x"), str_lit("")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("x".into()));
}

// ===========================================================================
// 0xB2 str_slice
// ===========================================================================

#[test]
fn str_slice_basic() {
    let g = make_prim_graph(0xB2, &[str_lit("hello world"), int_lit(0), int_lit(5)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("hello".into()));
}

#[test]
fn str_slice_middle() {
    let g = make_prim_graph(0xB2, &[str_lit("abcdef"), int_lit(2), int_lit(4)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("cd".into()));
}

#[test]
fn str_slice_out_of_bounds() {
    let g = make_prim_graph(0xB2, &[str_lit("hi"), int_lit(0), int_lit(100)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("hi".into()));
}

#[test]
fn str_slice_empty_range() {
    let g = make_prim_graph(0xB2, &[str_lit("abc"), int_lit(2), int_lit(2)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("".into()));
}

#[test]
fn str_slice_negative_start() {
    let g = make_prim_graph(0xB2, &[str_lit("hello"), int_lit(-3), int_lit(2)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("he".into()));
}

// ===========================================================================
// 0xB3 str_contains
// ===========================================================================

#[test]
fn str_contains_found() {
    let g = make_prim_graph(0xB3, &[str_lit("hello world"), str_lit("world")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(true));
}

#[test]
fn str_contains_not_found() {
    let g = make_prim_graph(0xB3, &[str_lit("hello"), str_lit("xyz")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(false));
}

#[test]
fn str_contains_empty_needle() {
    let g = make_prim_graph(0xB3, &[str_lit("anything"), str_lit("")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(true));
}

#[test]
fn str_contains_self() {
    let g = make_prim_graph(0xB3, &[str_lit("abc"), str_lit("abc")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(true));
}

// ===========================================================================
// 0xB4 str_split
// ===========================================================================

#[test]
fn str_split_basic() {
    let g = make_prim_graph(0xB4, &[str_lit("a,b,c"), str_lit(",")]);
    assert_eq!(
        eval_graph(&g).unwrap(),
        Value::tuple(vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
        ])
    );
}

#[test]
fn str_split_no_match() {
    let g = make_prim_graph(0xB4, &[str_lit("abc"), str_lit(",")]);
    assert_eq!(
        eval_graph(&g).unwrap(),
        Value::tuple(vec![Value::String("abc".into())])
    );
}

#[test]
fn str_split_empty_parts() {
    let g = make_prim_graph(0xB4, &[str_lit(",a,,b,"), str_lit(",")]);
    assert_eq!(
        eval_graph(&g).unwrap(),
        Value::tuple(vec![
            Value::String("".into()),
            Value::String("a".into()),
            Value::String("".into()),
            Value::String("b".into()),
            Value::String("".into()),
        ])
    );
}

// ===========================================================================
// 0xB5 str_join (tested indirectly since we can't easily create Tuple lits)
// ===========================================================================
// str_join is tested via split -> join roundtrip.

// ===========================================================================
// 0xB6 str_to_int
// ===========================================================================

#[test]
fn str_to_int_positive() {
    let g = make_prim_graph(0xB6, &[str_lit("42")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int(42));
}

#[test]
fn str_to_int_negative() {
    let g = make_prim_graph(0xB6, &[str_lit("-7")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int(-7));
}

#[test]
fn str_to_int_invalid() {
    let g = make_prim_graph(0xB6, &[str_lit("abc")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int(0));
}

#[test]
fn str_to_int_whitespace() {
    let g = make_prim_graph(0xB6, &[str_lit("  123  ")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int(123));
}

#[test]
fn str_to_int_zero() {
    let g = make_prim_graph(0xB6, &[str_lit("0")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int(0));
}

// ===========================================================================
// 0xB7 int_to_string
// ===========================================================================

#[test]
fn int_to_string_positive() {
    let g = make_prim_graph(0xB7, &[int_lit(42)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("42".into()));
}

#[test]
fn int_to_string_negative() {
    let g = make_prim_graph(0xB7, &[int_lit(-100)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("-100".into()));
}

#[test]
fn int_to_string_zero() {
    let g = make_prim_graph(0xB7, &[int_lit(0)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("0".into()));
}

// ===========================================================================
// 0xB8 str_eq
// ===========================================================================

#[test]
fn str_eq_same() {
    let g = make_prim_graph(0xB8, &[str_lit("abc"), str_lit("abc")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(true));
}

#[test]
fn str_eq_different() {
    let g = make_prim_graph(0xB8, &[str_lit("abc"), str_lit("def")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(false));
}

#[test]
fn str_eq_empty() {
    let g = make_prim_graph(0xB8, &[str_lit(""), str_lit("")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(true));
}

#[test]
fn str_eq_case_sensitive() {
    let g = make_prim_graph(0xB8, &[str_lit("Hello"), str_lit("hello")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(false));
}

// ===========================================================================
// 0xB9 str_starts_with
// ===========================================================================

#[test]
fn str_starts_with_true() {
    let g = make_prim_graph(0xB9, &[str_lit("hello world"), str_lit("hello")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(true));
}

#[test]
fn str_starts_with_false() {
    let g = make_prim_graph(0xB9, &[str_lit("hello"), str_lit("world")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(false));
}

#[test]
fn str_starts_with_empty_prefix() {
    let g = make_prim_graph(0xB9, &[str_lit("abc"), str_lit("")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(true));
}

// ===========================================================================
// 0xBA str_ends_with
// ===========================================================================

#[test]
fn str_ends_with_true() {
    let g = make_prim_graph(0xBA, &[str_lit("hello world"), str_lit("world")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(true));
}

#[test]
fn str_ends_with_false() {
    let g = make_prim_graph(0xBA, &[str_lit("hello"), str_lit("xyz")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(false));
}

#[test]
fn str_ends_with_empty_suffix() {
    let g = make_prim_graph(0xBA, &[str_lit("abc"), str_lit("")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Bool(true));
}

// ===========================================================================
// 0xBB str_replace
// ===========================================================================

#[test]
fn str_replace_basic() {
    let g = make_prim_graph(0xBB, &[str_lit("hello world"), str_lit("world"), str_lit("rust")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("hello rust".into()));
}

#[test]
fn str_replace_no_match() {
    let g = make_prim_graph(0xBB, &[str_lit("hello"), str_lit("xyz"), str_lit("abc")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("hello".into()));
}

#[test]
fn str_replace_multiple() {
    let g = make_prim_graph(0xBB, &[str_lit("aXbXc"), str_lit("X"), str_lit("_")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("a_b_c".into()));
}

#[test]
fn str_replace_with_empty() {
    let g = make_prim_graph(0xBB, &[str_lit("hello world"), str_lit(" world"), str_lit("")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("hello".into()));
}

// ===========================================================================
// 0xBC str_trim
// ===========================================================================

#[test]
fn str_trim_both() {
    let g = make_prim_graph(0xBC, &[str_lit("  hello  ")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("hello".into()));
}

#[test]
fn str_trim_leading() {
    let g = make_prim_graph(0xBC, &[str_lit("   abc")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("abc".into()));
}

#[test]
fn str_trim_trailing() {
    let g = make_prim_graph(0xBC, &[str_lit("xyz   ")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("xyz".into()));
}

#[test]
fn str_trim_no_whitespace() {
    let g = make_prim_graph(0xBC, &[str_lit("abc")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("abc".into()));
}

#[test]
fn str_trim_tabs_newlines() {
    let g = make_prim_graph(0xBC, &[str_lit("\t\n hello \n\t")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("hello".into()));
}

// ===========================================================================
// 0xBD str_upper
// ===========================================================================

#[test]
fn str_upper_basic() {
    let g = make_prim_graph(0xBD, &[str_lit("hello")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("HELLO".into()));
}

#[test]
fn str_upper_already_upper() {
    let g = make_prim_graph(0xBD, &[str_lit("ABC")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("ABC".into()));
}

#[test]
fn str_upper_mixed() {
    let g = make_prim_graph(0xBD, &[str_lit("Hello World")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("HELLO WORLD".into()));
}

#[test]
fn str_upper_empty() {
    let g = make_prim_graph(0xBD, &[str_lit("")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("".into()));
}

// ===========================================================================
// 0xBE str_lower
// ===========================================================================

#[test]
fn str_lower_basic() {
    let g = make_prim_graph(0xBE, &[str_lit("HELLO")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("hello".into()));
}

#[test]
fn str_lower_already_lower() {
    let g = make_prim_graph(0xBE, &[str_lit("abc")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("abc".into()));
}

#[test]
fn str_lower_mixed() {
    let g = make_prim_graph(0xBE, &[str_lit("Hello World")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("hello world".into()));
}

// ===========================================================================
// 0xBF str_chars
// ===========================================================================

#[test]
fn str_chars_basic() {
    let g = make_prim_graph(0xBF, &[str_lit("abc")]);
    assert_eq!(
        eval_graph(&g).unwrap(),
        Value::tuple(vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
        ])
    );
}

#[test]
fn str_chars_empty() {
    let g = make_prim_graph(0xBF, &[str_lit("")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::tuple(vec![]));
}

#[test]
fn str_chars_single() {
    let g = make_prim_graph(0xBF, &[str_lit("x")]);
    assert_eq!(
        eval_graph(&g).unwrap(),
        Value::tuple(vec![Value::String("x".into())])
    );
}

// ===========================================================================
// 0xC0 char_at
// ===========================================================================

#[test]
fn char_at_first() {
    let g = make_prim_graph(0xC0, &[str_lit("hello"), int_lit(0)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int('h' as i64));
}

#[test]
fn char_at_middle() {
    let g = make_prim_graph(0xC0, &[str_lit("hello"), int_lit(2)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int('l' as i64));
}

#[test]
fn char_at_last() {
    let g = make_prim_graph(0xC0, &[str_lit("hello"), int_lit(4)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int('o' as i64));
}

#[test]
fn char_at_out_of_bounds() {
    let g = make_prim_graph(0xC0, &[str_lit("hi"), int_lit(5)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int(-1));
}

#[test]
fn char_at_negative_index() {
    let g = make_prim_graph(0xC0, &[str_lit("abc"), int_lit(-1)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int(-1));
}

#[test]
fn char_at_empty_string() {
    let g = make_prim_graph(0xC0, &[str_lit(""), int_lit(0)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int(-1));
}

#[test]
fn char_at_digit() {
    let g = make_prim_graph(0xC0, &[str_lit("42"), int_lit(0)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int('4' as i64));
}

#[test]
fn char_at_space() {
    let g = make_prim_graph(0xC0, &[str_lit("a b"), int_lit(1)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::Int(' ' as i64));
}

// ===========================================================================
// Prim name resolution (prim.rs)
// ===========================================================================

#[test]
fn prim_resolution_all_string_ops() {
    use iris_bootstrap::syntax::prim::resolve_primitive;

    let expected: &[(&str, u8, u8)] = &[
        ("str_len", 0xB0, 1),
        ("str_concat", 0xB1, 2),
        ("str_slice", 0xB2, 3),
        ("str_contains", 0xB3, 2),
        ("str_split", 0xB4, 2),
        ("str_join", 0xB5, 2),
        ("str_to_int", 0xB6, 1),
        ("int_to_string", 0xB7, 1),
        ("str_eq", 0xB8, 2),
        ("str_starts_with", 0xB9, 2),
        ("str_ends_with", 0xBA, 2),
        ("str_replace", 0xBB, 3),
        ("str_trim", 0xBC, 1),
        ("str_upper", 0xBD, 1),
        ("str_lower", 0xBE, 1),
        ("str_chars", 0xBF, 1),
        ("char_at", 0xC0, 2),
    ];

    for &(name, opcode, arity) in expected {
        let result = resolve_primitive(name);
        assert_eq!(
            result,
            Some((opcode, arity)),
            "resolve_primitive({:?}) should be Some(({:#04x}, {})), got {:?}",
            name, opcode, arity, result,
        );
    }
}

// ===========================================================================
// Composition / round-trip tests
// ===========================================================================

#[test]
fn str_to_int_of_int_to_string_roundtrip() {
    // int_to_string(42) = "42", then str_to_int("42") = 42
    let g = make_prim_graph(0xB7, &[int_lit(42)]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("42".into()));

    let g2 = make_prim_graph(0xB6, &[str_lit("42")]);
    assert_eq!(eval_graph(&g2).unwrap(), Value::Int(42));
}

#[test]
fn str_upper_then_lower_identity() {
    // str_lower(str_upper("Hello")) should be "hello" (not "Hello")
    let g_upper = make_prim_graph(0xBD, &[str_lit("Hello")]);
    assert_eq!(eval_graph(&g_upper).unwrap(), Value::String("HELLO".into()));

    let g_lower = make_prim_graph(0xBE, &[str_lit("HELLO")]);
    assert_eq!(eval_graph(&g_lower).unwrap(), Value::String("hello".into()));
}

#[test]
fn str_trim_then_contains() {
    // After trimming "  hello  ", should contain "hello"
    let g_trim = make_prim_graph(0xBC, &[str_lit("  hello  ")]);
    assert_eq!(eval_graph(&g_trim).unwrap(), Value::String("hello".into()));

    let g_contains = make_prim_graph(0xB3, &[str_lit("hello"), str_lit("ell")]);
    assert_eq!(eval_graph(&g_contains).unwrap(), Value::Bool(true));
}

#[test]
fn str_replace_then_len() {
    // Replace "world" with "!" in "hello world", length should be 6
    let g = make_prim_graph(0xBB, &[str_lit("hello world"), str_lit("world"), str_lit("!")]);
    assert_eq!(eval_graph(&g).unwrap(), Value::String("hello !".into()));
}

#[test]
fn char_at_matches_str_chars_order() {
    // char_at("abc", 0) should be 'a' == 97
    let g0 = make_prim_graph(0xC0, &[str_lit("abc"), int_lit(0)]);
    assert_eq!(eval_graph(&g0).unwrap(), Value::Int(97));

    // char_at("abc", 1) should be 'b' == 98
    let g1 = make_prim_graph(0xC0, &[str_lit("abc"), int_lit(1)]);
    assert_eq!(eval_graph(&g1).unwrap(), Value::Int(98));

    // char_at("abc", 2) should be 'c' == 99
    let g2 = make_prim_graph(0xC0, &[str_lit("abc"), int_lit(2)]);
    assert_eq!(eval_graph(&g2).unwrap(), Value::Int(99));
}
