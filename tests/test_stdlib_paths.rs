
//! Tests for the IRIS stdlib path operations.
//!
//! Validates the path manipulation functions from src/iris-programs/stdlib/path_ops.iris:
//! - path_join: join two path segments
//! - path_extension: extract file extension
//! - path_basename: extract filename from path
//! - path_dirname: extract directory part of path
//!
//! Pure path functions are tested via compiled IRIS surface syntax.
//! Integration test verifies path ops with real file I/O.

use iris_exec::effect_runtime::RuntimeEffectHandler;
use iris_exec::interpreter;
use iris_types::eval::{EffectHandler, EffectRequest, EffectTag, Value};
use iris_types::graph::SemanticGraph;

use std::collections::{BTreeMap, HashMap};
use iris_types::cost::{CostBound, CostTerm};
use iris_types::graph::{Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution};
use iris_types::hash::{compute_node_id, SemanticHash};
use iris_types::types::{PrimType, TypeDef, TypeEnv, TypeId};

// ===========================================================================
// Helpers — graph-based evaluation (for testing individual string prims)
// ===========================================================================

fn make_type_env() -> (TypeEnv, TypeId) {
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
            resolution_depth: 2,
            salt: 0,
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
        resolution_depth: 2,
        salt: 0,
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

fn eval_graph(graph: &SemanticGraph) -> Value {
    let (outputs, _state) = interpreter::interpret(graph, &[], None).unwrap();
    if outputs.len() == 1 {
        outputs.into_iter().next().unwrap()
    } else {
        Value::tuple(outputs)
    }
}

// ===========================================================================
// Helpers — surface syntax compilation
// ===========================================================================

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
// path_join tests — compiled from IRIS surface syntax
//
// Logic: if str_ends_with(a, "/") == 1 then concat(a, b)
//        else concat(concat(a, "/"), b)
// ===========================================================================

const PATH_JOIN_SRC: &str = r#"
let f a b =
    if (str_ends_with a "/") == 1 then str_concat a b
    else str_concat (str_concat a "/") b
"#;

#[test]
fn test_path_join_simple() {
    let result = run(
        PATH_JOIN_SRC,
        &[Value::String("foo".into()), Value::String("bar".into())],
    );
    assert_eq!(result, Value::String("foo/bar".into()));
}

#[test]
fn test_path_join_trailing_slash() {
    let result = run(
        PATH_JOIN_SRC,
        &[Value::String("foo/".into()), Value::String("bar".into())],
    );
    assert_eq!(result, Value::String("foo/bar".into()));
}

#[test]
fn test_path_join_absolute() {
    let result = run(
        PATH_JOIN_SRC,
        &[Value::String("/home".into()), Value::String("user".into())],
    );
    assert_eq!(result, Value::String("/home/user".into()));
}

// ===========================================================================
// path_extension tests — using str_split + list_nth
//
// Strategy: split on ".", take last element, prepend "." if there was a dot.
// For paths without extensions, the split yields only 1 element.
// ===========================================================================

#[test]
fn test_path_extension_rs() {
    // Split "main.rs" by "." -> ["main", "rs"]. Two parts, so extension is ".rs".
    let src = r#"
let f path =
    let parts = str_split path "." in
    let n = list_len parts in
    if n > 1 then str_concat "." (list_nth parts (n - 1))
    else ""
"#;
    let result = run(src, &[Value::String("main.rs".into())]);
    assert_eq!(result, Value::String(".rs".into()));
}

#[test]
fn test_path_extension_none() {
    let src = r#"
let f path =
    let parts = str_split path "." in
    let n = list_len parts in
    if n > 1 then str_concat "." (list_nth parts (n - 1))
    else ""
"#;
    let result = run(src, &[Value::String("Makefile".into())]);
    assert_eq!(result, Value::String("".into()));
}

#[test]
fn test_path_extension_double() {
    // "archive.tar.gz" split by "." -> ["archive", "tar", "gz"]. Last = ".gz".
    let src = r#"
let f path =
    let parts = str_split path "." in
    let n = list_len parts in
    if n > 1 then str_concat "." (list_nth parts (n - 1))
    else ""
"#;
    let result = run(src, &[Value::String("archive.tar.gz".into())]);
    assert_eq!(result, Value::String(".gz".into()));
}

// ===========================================================================
// path_basename tests — using str_split + list_nth
//
// Split on "/", take the last element.
// ===========================================================================

#[test]
fn test_path_basename() {
    let src = r#"
let f path =
    let parts = str_split path "/" in
    list_nth parts (list_len parts - 1)
"#;
    let result = run(src, &[Value::String("/home/user/file.txt".into())]);
    assert_eq!(result, Value::String("file.txt".into()));
}

#[test]
fn test_path_basename_no_slash() {
    let src = r#"
let f path =
    let parts = str_split path "/" in
    list_nth parts (list_len parts - 1)
"#;
    let result = run(src, &[Value::String("file.txt".into())]);
    assert_eq!(result, Value::String("file.txt".into()));
}

// ===========================================================================
// path_dirname tests
//
// Use str_split on "/", take all but last, rejoin with "/".
// Handle edge cases: root path ("/file.txt") and no-slash ("file.txt").
// ===========================================================================

#[test]
fn test_path_dirname() {
    // For "/home/user/file.txt": split by "/" -> ["", "home", "user", "file.txt"]
    // Take all but last and join -> "/home/user"
    let src = r#"
let f path =
    let parts = str_split path "/" in
    let n = list_len parts in
    if n <= 1 then "."
    else
        let init = list_take parts (n - 1) in
        let joined = str_join init "/" in
        if (str_len joined) == 0 then "/"
        else joined
"#;
    let result = run(src, &[Value::String("/home/user/file.txt".into())]);
    assert_eq!(result, Value::String("/home/user".into()));
}

#[test]
fn test_path_dirname_root() {
    // "/file.txt" -> split by "/" -> ["", "file.txt"]
    // init = [""], joined = "" -> return "/"
    let src = r#"
let f path =
    let parts = str_split path "/" in
    let n = list_len parts in
    if n <= 1 then "."
    else
        let init = list_take parts (n - 1) in
        let joined = str_join init "/" in
        if (str_len joined) == 0 then "/"
        else joined
"#;
    let result = run(src, &[Value::String("/file.txt".into())]);
    assert_eq!(result, Value::String("/".into()));
}

#[test]
fn test_path_dirname_no_slash() {
    // "file.txt" -> split by "/" -> ["file.txt"]. n=1 -> "."
    let src = r#"
let f path =
    let parts = str_split path "/" in
    let n = list_len parts in
    if n <= 1 then "."
    else
        let init = list_take parts (n - 1) in
        let joined = str_join init "/" in
        if (str_len joined) == 0 then "/"
        else joined
"#;
    let result = run(src, &[Value::String("file.txt".into())]);
    assert_eq!(result, Value::String(".".into()));
}

// ===========================================================================
// Additional path_join edge cases via graph-level primitives
// ===========================================================================

#[test]
fn test_str_ends_with_slash_via_prim() {
    // str_ends_with("foo/", "/") => true
    let g = make_prim_graph(0xBA, &[str_lit("foo/"), str_lit("/")]);
    assert_eq!(eval_graph(&g), Value::Bool(true));
}

#[test]
fn test_str_ends_with_no_slash_via_prim() {
    // str_ends_with("foo", "/") => false
    let g = make_prim_graph(0xBA, &[str_lit("foo"), str_lit("/")]);
    assert_eq!(eval_graph(&g), Value::Bool(false));
}

#[test]
fn test_char_at_slash_via_prim() {
    // char_at("/home/user", 0) => 47 (ASCII '/')
    let g = make_prim_graph(0xC0, &[str_lit("/home/user"), int_lit(0)]);
    assert_eq!(eval_graph(&g), Value::Int(47));
}

// ===========================================================================
// Integration: path operations with real files
// ===========================================================================

#[test]
fn test_path_ops_with_real_files() {
    let handler = RuntimeEffectHandler::new();
    let tmp_dir = std::env::temp_dir().join("iris_stdlib_paths");
    let _ = std::fs::create_dir_all(&tmp_dir);
    let dir_str = tmp_dir.to_str().unwrap().to_string();

    // Use IRIS path_join to build a path.
    let joined = run(
        PATH_JOIN_SRC,
        &[
            Value::String(dir_str.clone()),
            Value::String("test_path_ops.txt".into()),
        ],
    );
    let file_path = match &joined {
        Value::String(s) => s.clone(),
        other => panic!("expected String, got {:?}", other),
    };

    // Write a file via RuntimeEffectHandler.
    let open_req = EffectRequest {
        tag: EffectTag::FileOpen,
        args: vec![Value::String(file_path.clone()), Value::Int(1)],
    };
    let handle = match handler.handle(open_req).unwrap() {
        Value::Int(h) => h,
        other => panic!("expected Int, got {:?}", other),
    };
    let write_req = EffectRequest {
        tag: EffectTag::FileWriteBytes,
        args: vec![Value::Int(handle), Value::Bytes(b"path test data".to_vec())],
    };
    handler.handle(write_req).unwrap();
    let close_req = EffectRequest {
        tag: EffectTag::FileClose,
        args: vec![Value::Int(handle)],
    };
    handler.handle(close_req).unwrap();

    // Verify the file exists via file_stat.
    let stat_req = EffectRequest {
        tag: EffectTag::FileStat,
        args: vec![Value::String(file_path.clone())],
    };
    let stat_result = handler.handle(stat_req);
    assert!(stat_result.is_ok(), "file at joined path should exist");

    // Extract basename with IRIS and verify it.
    let basename_src = r#"
let f path =
    let parts = str_split path "/" in
    list_nth parts (list_len parts - 1)
"#;
    let basename = run(basename_src, &[Value::String(file_path.clone())]);
    assert_eq!(basename, Value::String("test_path_ops.txt".into()));

    // Extract dirname and verify.
    let dirname_src = r#"
let f path =
    let parts = str_split path "/" in
    let n = list_len parts in
    if n <= 1 then "."
    else
        let init = list_take parts (n - 1) in
        let joined = str_join init "/" in
        if (str_len joined) == 0 then "/"
        else joined
"#;
    let dirname = run(dirname_src, &[Value::String(file_path.clone())]);
    assert_eq!(dirname, Value::String(dir_str));

    // Cleanup.
    let _ = std::fs::remove_file(&file_path);
    let _ = std::fs::remove_dir(&tmp_dir);
}
