
//! Integration tests for data structure primitives: list, map, and set operations.
//!
//! List primitives (0xC1-0xC7) operate on Value::Tuple.
//! Map primitives (0xC8-0xCD) operate on Value::State (BTreeMap<String, Value>).
//! Set operations are built on sorted+deduped lists.

use iris_exec::interpreter;
use iris_types::eval::{StateStore, Value};
use iris_types::graph::SemanticGraph;

fn compile_and_get_graph(src: &str) -> SemanticGraph {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!("compilation failed with {} errors", result.errors.len());
    }
    assert!(!result.fragments.is_empty(), "no fragments produced");
    result.fragments[0].1.graph.clone()
}

fn run(src: &str, inputs: &[Value]) -> Value {
    let g = compile_and_get_graph(src);
    let (out, _) = interpreter::interpret(&g, inputs, None).unwrap();
    assert_eq!(out.len(), 1, "expected single output value");
    out.into_iter().next().unwrap()
}

// ===========================================================================
// List primitives
// ===========================================================================

#[test]
fn test_list_concat_basic() {
    let out = run(
        "let f xs ys = list_concat xs ys",
        &[
            Value::tuple(vec![Value::Int(1), Value::Int(2)]),
            Value::tuple(vec![Value::Int(3), Value::Int(4)]),
        ],
    );
    assert_eq!(
        out,
        Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)])
    );
}

#[test]
fn test_list_concat_empty() {
    let out = run(
        "let f xs ys = list_concat xs ys",
        &[
            Value::tuple(vec![]),
            Value::tuple(vec![Value::Int(1)]),
        ],
    );
    assert_eq!(out, Value::tuple(vec![Value::Int(1)]));
}

#[test]
fn test_list_concat_both_empty() {
    let out = run(
        "let f xs ys = list_concat xs ys",
        &[Value::tuple(vec![]), Value::tuple(vec![])],
    );
    assert_eq!(out, Value::tuple(vec![]));
}

#[test]
fn test_list_nth_valid() {
    let out = run(
        "let f xs i = list_nth xs i",
        &[
            Value::tuple(vec![Value::Int(10), Value::Int(20), Value::Int(30)]),
            Value::Int(1),
        ],
    );
    assert_eq!(out, Value::Int(20));
}

#[test]
fn test_list_nth_first() {
    let out = run(
        "let f xs i = list_nth xs i",
        &[
            Value::tuple(vec![Value::Int(10), Value::Int(20)]),
            Value::Int(0),
        ],
    );
    assert_eq!(out, Value::Int(10));
}

#[test]
fn test_list_nth_out_of_bounds() {
    let out = run(
        "let f xs i = list_nth xs i",
        &[
            Value::tuple(vec![Value::Int(10)]),
            Value::Int(5),
        ],
    );
    assert_eq!(out, Value::Unit);
}

#[test]
fn test_list_nth_negative() {
    let out = run(
        "let f xs i = list_nth xs i",
        &[
            Value::tuple(vec![Value::Int(10)]),
            Value::Int(-1),
        ],
    );
    assert_eq!(out, Value::Unit);
}

#[test]
fn test_list_take() {
    let out = run(
        "let f xs n = list_take xs n",
        &[
            Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]),
            Value::Int(2),
        ],
    );
    assert_eq!(out, Value::tuple(vec![Value::Int(1), Value::Int(2)]));
}

#[test]
fn test_list_take_more_than_length() {
    let out = run(
        "let f xs n = list_take xs n",
        &[
            Value::tuple(vec![Value::Int(1), Value::Int(2)]),
            Value::Int(10),
        ],
    );
    assert_eq!(out, Value::tuple(vec![Value::Int(1), Value::Int(2)]));
}

#[test]
fn test_list_take_zero() {
    let out = run(
        "let f xs n = list_take xs n",
        &[
            Value::tuple(vec![Value::Int(1), Value::Int(2)]),
            Value::Int(0),
        ],
    );
    assert_eq!(out, Value::tuple(vec![]));
}

#[test]
fn test_list_take_negative() {
    let out = run(
        "let f xs n = list_take xs n",
        &[
            Value::tuple(vec![Value::Int(1), Value::Int(2)]),
            Value::Int(-3),
        ],
    );
    assert_eq!(out, Value::tuple(vec![]));
}

#[test]
fn test_list_drop() {
    let out = run(
        "let f xs n = list_drop xs n",
        &[
            Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]),
            Value::Int(2),
        ],
    );
    assert_eq!(out, Value::tuple(vec![Value::Int(3), Value::Int(4)]));
}

#[test]
fn test_list_drop_more_than_length() {
    let out = run(
        "let f xs n = list_drop xs n",
        &[
            Value::tuple(vec![Value::Int(1)]),
            Value::Int(10),
        ],
    );
    assert_eq!(out, Value::tuple(vec![]));
}

#[test]
fn test_list_drop_zero() {
    let out = run(
        "let f xs n = list_drop xs n",
        &[
            Value::tuple(vec![Value::Int(1), Value::Int(2)]),
            Value::Int(0),
        ],
    );
    assert_eq!(out, Value::tuple(vec![Value::Int(1), Value::Int(2)]));
}

#[test]
fn test_list_sort() {
    let out = run(
        "let f xs = list_sort xs",
        &[Value::tuple(vec![
            Value::Int(3),
            Value::Int(1),
            Value::Int(4),
            Value::Int(1),
            Value::Int(5),
            Value::Int(2),
        ])],
    );
    assert_eq!(
        out,
        Value::tuple(vec![
            Value::Int(1),
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ])
    );
}

#[test]
fn test_list_sort_already_sorted() {
    let out = run(
        "let f xs = list_sort xs",
        &[Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)])],
    );
    assert_eq!(out, Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
}

#[test]
fn test_list_sort_empty() {
    let out = run("let f xs = list_sort xs", &[Value::tuple(vec![])]);
    assert_eq!(out, Value::tuple(vec![]));
}

#[test]
fn test_list_sort_single() {
    let out = run("let f xs = list_sort xs", &[Value::tuple(vec![Value::Int(42)])]);
    assert_eq!(out, Value::tuple(vec![Value::Int(42)]));
}

#[test]
fn test_list_sort_negative() {
    let out = run(
        "let f xs = list_sort xs",
        &[Value::tuple(vec![Value::Int(3), Value::Int(-1), Value::Int(0)])],
    );
    assert_eq!(out, Value::tuple(vec![Value::Int(-1), Value::Int(0), Value::Int(3)]));
}

#[test]
fn test_list_dedup() {
    let out = run(
        "let f xs = list_dedup xs",
        &[Value::tuple(vec![
            Value::Int(1),
            Value::Int(1),
            Value::Int(2),
            Value::Int(2),
            Value::Int(3),
            Value::Int(1),
        ])],
    );
    // Full dedup (not just consecutive): [1,1,2,2,3,1] -> [1,2,3]
    assert_eq!(
        out,
        Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
    );
}

#[test]
fn test_list_dedup_empty() {
    let out = run("let f xs = list_dedup xs", &[Value::tuple(vec![])]);
    assert_eq!(out, Value::tuple(vec![]));
}

#[test]
fn test_list_dedup_no_dups() {
    let out = run(
        "let f xs = list_dedup xs",
        &[Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)])],
    );
    assert_eq!(out, Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
}

#[test]
fn test_list_sort_then_dedup() {
    // sort + dedup = unique sorted (i.e., a set)
    let out = run(
        "let f xs = list_dedup (list_sort xs)",
        &[Value::tuple(vec![
            Value::Int(3),
            Value::Int(1),
            Value::Int(2),
            Value::Int(1),
            Value::Int(3),
        ])],
    );
    assert_eq!(out, Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
}

#[test]
fn test_list_range() {
    let out = run("let f a b = list_range a b", &[Value::Int(0), Value::Int(5)]);
    assert_eq!(
        out,
        Value::tuple(vec![
            Value::Int(0),
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ])
    );
}

#[test]
fn test_list_range_negative() {
    let out = run("let f a b = list_range a b", &[Value::Int(-2), Value::Int(3)]);
    assert_eq!(
        out,
        Value::tuple(vec![
            Value::Int(-2),
            Value::Int(-1),
            Value::Int(0),
            Value::Int(1),
            Value::Int(2),
        ])
    );
}

#[test]
fn test_list_range_empty() {
    let out = run("let f a b = list_range a b", &[Value::Int(5), Value::Int(5)]);
    assert_eq!(out, Value::tuple(vec![]));
}

#[test]
fn test_list_range_reversed() {
    // start > end should produce empty
    let out = run("let f a b = list_range a b", &[Value::Int(5), Value::Int(2)]);
    assert_eq!(out, Value::tuple(vec![]));
}

// ===========================================================================
// Map primitives
// ===========================================================================

#[test]
fn test_map_insert_and_get() {
    let out = run(
        "let f k v = map_get (map_insert state_empty k v) k",
        &[Value::String("key1".into()), Value::Int(42)],
    );
    assert_eq!(out, Value::Int(42));
}

#[test]
fn test_map_get_missing() {
    let out = run(
        "let f k = map_get state_empty k",
        &[Value::String("nope".into())],
    );
    assert_eq!(out, Value::Unit);
}

#[test]
fn test_map_remove() {
    let out = run(
        "let f k v = map_size (map_remove (map_insert state_empty k v) k)",
        &[Value::String("key1".into()), Value::Int(100)],
    );
    assert_eq!(out, Value::Int(0));
}

#[test]
fn test_map_keys() {
    let out = run(
        r#"let f = map_keys (map_insert (map_insert state_empty "a" 1) "b" 2)"#,
        &[],
    );
    // BTreeMap sorted order: "a", "b"
    assert_eq!(
        out,
        Value::tuple(vec![Value::String("a".into()), Value::String("b".into())])
    );
}

#[test]
fn test_map_values() {
    let out = run(
        r#"let f = map_values (map_insert (map_insert state_empty "a" 10) "b" 20)"#,
        &[],
    );
    // BTreeMap sorted by key: "a"->10, "b"->20
    assert_eq!(
        out,
        Value::tuple(vec![Value::Int(10), Value::Int(20)])
    );
}

#[test]
fn test_map_size() {
    let out = run(
        r#"let f = map_size (map_insert (map_insert state_empty "x" 1) "y" 2)"#,
        &[],
    );
    assert_eq!(out, Value::Int(2));
}

#[test]
fn test_map_size_empty() {
    let out = run("let f = map_size state_empty", &[]);
    assert_eq!(out, Value::Int(0));
}

#[test]
fn test_map_insert_overwrite() {
    let out = run(
        r#"let f = map_get (map_insert (map_insert state_empty "k" 1) "k" 2) "k""#,
        &[],
    );
    assert_eq!(out, Value::Int(2));
}

#[test]
fn test_map_remove_nonexistent() {
    let out = run(
        r#"let f = map_size (map_remove state_empty "nope")"#,
        &[],
    );
    assert_eq!(out, Value::Int(0));
}

// ===========================================================================
// Composition: list + map together
// ===========================================================================

#[test]
fn test_list_take_drop_composition() {
    // take 2 then drop 1 from [10, 20, 30, 40] -> [20]
    let out = run(
        "let f xs = list_drop (list_take xs 2) 1",
        &[Value::tuple(vec![
            Value::Int(10),
            Value::Int(20),
            Value::Int(30),
            Value::Int(40),
        ])],
    );
    assert_eq!(out, Value::tuple(vec![Value::Int(20)]));
}

#[test]
fn test_list_range_sum() {
    // Sum of range [1, 6) = 1+2+3+4+5 = 15
    let out = run("let f = fold 0 (+) (list_range 1 6)", &[]);
    assert_eq!(out, Value::Int(15));
}

#[test]
fn test_list_sort_with_map() {
    // Sort a list, then map double
    let out = run(
        "let f xs = map (\\x -> x * 2) (list_sort xs)",
        &[Value::tuple(vec![Value::Int(3), Value::Int(1), Value::Int(2)])],
    );
    assert_eq!(
        out,
        Value::tuple(vec![Value::Int(2), Value::Int(4), Value::Int(6)])
    );
}

#[test]
fn test_list_concat_then_sort() {
    let out = run(
        "let f xs ys = list_sort (list_concat xs ys)",
        &[
            Value::tuple(vec![Value::Int(5), Value::Int(3)]),
            Value::tuple(vec![Value::Int(1), Value::Int(4)]),
        ],
    );
    assert_eq!(
        out,
        Value::tuple(vec![Value::Int(1), Value::Int(3), Value::Int(4), Value::Int(5)])
    );
}

// ===========================================================================
// Set operations (sort + dedup)
// ===========================================================================

#[test]
fn test_set_from_list() {
    let out = run(
        "let f xs = list_dedup (list_sort xs)",
        &[Value::tuple(vec![
            Value::Int(3),
            Value::Int(1),
            Value::Int(2),
            Value::Int(1),
            Value::Int(3),
            Value::Int(2),
        ])],
    );
    assert_eq!(out, Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
}

#[test]
fn test_set_union() {
    // Union = dedup(sort(append(a, b)))
    let out = run(
        "let f a b = list_dedup (list_sort (list_concat a b))",
        &[
            Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
            Value::tuple(vec![Value::Int(2), Value::Int(3), Value::Int(4)]),
        ],
    );
    assert_eq!(
        out,
        Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)])
    );
}

#[test]
fn test_list_range_fold_product() {
    // Product of range [1, 5) = 1*2*3*4 = 24
    let out = run("let f = fold 1 (*) (list_range 1 5)", &[]);
    assert_eq!(out, Value::Int(24));
}

#[test]
fn test_list_nth_after_sort() {
    // Sort [5,1,3], then get index 0 (should be 1, the minimum)
    let out = run(
        "let f xs = list_nth (list_sort xs) 0",
        &[Value::tuple(vec![Value::Int(5), Value::Int(1), Value::Int(3)])],
    );
    assert_eq!(out, Value::Int(1));
}

#[test]
fn test_map_insert_get_with_int_key() {
    // Int keys are coerced to string via to_string
    let out = run(
        "let f = map_get (map_insert state_empty 42 100) 42",
        &[],
    );
    assert_eq!(out, Value::Int(100));
}

#[test]
fn test_map_multiple_operations() {
    // Insert 3 entries, remove 1, check size
    let out = run(
        r#"let f = map_size (map_remove (map_insert (map_insert (map_insert state_empty "a" 1) "b" 2) "c" 3) "b")"#,
        &[],
    );
    assert_eq!(out, Value::Int(2));
}
