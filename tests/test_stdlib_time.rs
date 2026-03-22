
//! Tests for the IRIS stdlib time utilities.
//!
//! Validates:
//! - clock_ns returns a positive timestamp (the only true time primitive — side effect)
//! - now_ms (clock_ns / 1000000) returns a reasonable value
//! - elapsed_ms measures real time
//! - format/parse are pure IRIS using int_to_string, string_to_int, arithmetic
//!   (no opcodes — these are compositions of existing primitives)

use iris_exec::effect_runtime::RuntimeEffectHandler;
use iris_types::eval::{EffectHandler, EffectRequest, EffectTag, Value};

use std::collections::{BTreeMap, HashMap};
use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::{compute_node_id, SemanticHash};
use iris_types::types::{PrimType, TypeDef, TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn int_type_env() -> (TypeEnv, TypeId) {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = iris_types::hash::compute_type_id(&int_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    (TypeEnv { types }, int_id)
}

fn make_node_with_salt(
    kind: NodeKind,
    payload: NodePayload,
    type_sig: TypeId,
    arity: u8,
    salt: u64,
) -> Node {
    let mut node = Node {
        id: NodeId(0),
        kind,
        type_sig,
        cost: CostTerm::Unit,
        arity,
        resolution_depth: 2,
        salt,
        payload,
    };
    node.id = compute_node_id(&node);
    node
}

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

/// Build a graph: Prim(opcode, lit_values...)
fn make_prim_graph(opcode: u8, lit_values: &[Value]) -> SemanticGraph {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let prim_node = make_node_with_salt(
        NodeKind::Prim,
        NodePayload::Prim { opcode },
        int_id,
        lit_values.len() as u8,
        1,
    );
    let root_id = prim_node.id;
    nodes.insert(root_id, prim_node);

    for (i, val) in lit_values.iter().enumerate() {
        let (type_tag, value_bytes) = match val {
            Value::Int(v) => (0x00u8, v.to_le_bytes().to_vec()),
            Value::String(s) => (0x07u8, s.as_bytes().to_vec()),
            Value::Bytes(b) => (0x05u8, b.clone()),
            _ => (0x06u8, vec![]),
        };
        let lit_node = make_node_with_salt(
            NodeKind::Lit,
            NodePayload::Lit {
                type_tag,
                value: value_bytes,
            },
            int_id,
            0,
            (i + 100) as u64,
        );
        let lit_id = lit_node.id;
        nodes.insert(lit_id, lit_node);

        edges.push(Edge {
            source: root_id,
            target: lit_id,
            port: i as u8,
            label: EdgeLabel::Argument,
        });
    }

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: root_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

// ---------------------------------------------------------------------------
// 1. clock_ns returns a positive timestamp
// ---------------------------------------------------------------------------

#[test]
fn clock_ns_returns_positive() {
    let handler = RuntimeEffectHandler::new();
    let req = EffectRequest {
        tag: EffectTag::ClockNs,
        args: vec![],
    };
    let result = handler.handle(req).unwrap();
    match result {
        Value::Int(ns) => {
            assert!(ns > 0, "clock_ns should return a positive value, got {}", ns);
            // Should be after 2020-01-01 in nanoseconds
            assert!(
                ns > 1_577_836_800_000_000_000i64,
                "clock_ns should be a recent timestamp"
            );
        }
        other => panic!("expected Int from clock_ns, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 2. now_ms returns a reasonable millisecond timestamp
// ---------------------------------------------------------------------------

#[test]
fn now_ms_returns_reasonable_value() {
    let handler = RuntimeEffectHandler::new();
    let req = EffectRequest {
        tag: EffectTag::ClockNs,
        args: vec![],
    };
    let ns = match handler.handle(req).unwrap() {
        Value::Int(n) => n,
        other => panic!("expected Int, got {:?}", other),
    };
    let ms = ns / 1_000_000;

    // Should be after 2020-01-01 in milliseconds
    assert!(
        ms > 1_577_836_800_000i64,
        "now_ms should be a recent timestamp"
    );
    // Should be before 2050-01-01 in milliseconds
    assert!(
        ms < 2_524_608_000_000i64,
        "now_ms should not be too far in the future"
    );
}

// ---------------------------------------------------------------------------
// 3. elapsed_ms measures real time
// ---------------------------------------------------------------------------

#[test]
fn elapsed_ms_measures_time() {
    let handler = RuntimeEffectHandler::new();

    // Get start time
    let req1 = EffectRequest {
        tag: EffectTag::ClockNs,
        args: vec![],
    };
    let start_ns = match handler.handle(req1).unwrap() {
        Value::Int(n) => n,
        other => panic!("expected Int, got {:?}", other),
    };
    let start_ms = start_ns / 1_000_000;

    // Sleep a tiny bit (5ms)
    let sleep_req = EffectRequest {
        tag: EffectTag::SleepMs,
        args: vec![Value::Int(5)],
    };
    handler.handle(sleep_req).unwrap();

    // Get end time
    let req2 = EffectRequest {
        tag: EffectTag::ClockNs,
        args: vec![],
    };
    let end_ns = match handler.handle(req2).unwrap() {
        Value::Int(n) => n,
        other => panic!("expected Int, got {:?}", other),
    };
    let end_ms = end_ns / 1_000_000;

    let elapsed = end_ms - start_ms;
    assert!(
        elapsed >= 4,
        "elapsed should be at least 4ms after 5ms sleep, got {}ms",
        elapsed
    );
    assert!(
        elapsed < 500,
        "elapsed should be less than 500ms, got {}ms",
        elapsed
    );
}

// ---------------------------------------------------------------------------
// 4. format_ms — pure IRIS: just int_to_string
// ---------------------------------------------------------------------------

#[test]
fn time_format_pure_iris_ms() {
    // Formatting milliseconds is just int_to_string — no opcode needed.
    let src = r#"let format_ms ms = int_to_string ms"#;
    let g = iris_bootstrap::syntax::compile(src);
    assert!(g.errors.is_empty(), "compile failed: {:?}", g.errors);
    let graph = g.fragments[0].1.graph.clone();
    let (out, _) = interpreter::interpret(&graph, &[Value::Int(1234567)], None).unwrap();
    assert_eq!(out, vec![Value::String("1234567".into())]);
}

// ---------------------------------------------------------------------------
// 5. format_seconds — pure IRIS: division + int_to_string + string concat
// ---------------------------------------------------------------------------

#[test]
fn time_format_pure_iris_seconds() {
    // 2500ms → "2.500" using only arithmetic and string primitives
    let src = r#"
let format_secs ms =
    let secs = ms / 1000 in
    let frac = ms - (secs * 1000) in
    let frac_str = int_to_string frac in
    let padded = if frac < 10 then str_concat "00" frac_str
                 else if frac < 100 then str_concat "0" frac_str
                 else frac_str in
    str_concat (int_to_string secs) (str_concat "." padded)
"#;
    let g = iris_bootstrap::syntax::compile(src);
    assert!(g.errors.is_empty(), "compile failed: {:?}", g.errors);
    let graph = g.fragments[0].1.graph.clone();
    let (out, _) = interpreter::interpret(&graph, &[Value::Int(2500)], None).unwrap();
    assert_eq!(out, vec![Value::String("2.500".into())]);
}

// ---------------------------------------------------------------------------
// 6. parse_ms — pure IRIS: just string_to_int
// ---------------------------------------------------------------------------

#[test]
fn time_parse_pure_iris_ms() {
    // Parsing a millisecond string is just str_to_int — no opcode needed.
    let src = r#"let parse_ms s = str_to_int s"#;
    let g = iris_bootstrap::syntax::compile(src);
    assert!(g.errors.is_empty(), "compile failed: {:?}", g.errors);
    let graph = g.fragments[0].1.graph.clone();
    let (out, _) = interpreter::interpret(&graph, &[Value::String("42000".into())], None).unwrap();
    assert_eq!(out, vec![Value::Int(42000)]);
}

// ---------------------------------------------------------------------------
// 7. parse_seconds — pure IRIS: split on ".", parse parts, multiply
// ---------------------------------------------------------------------------

#[test]
fn time_parse_pure_iris_seconds() {
    // "1.5" → parse whole part "1" * 1000 + fractional "5" * 100 = 1500ms
    // We use str_split, str_to_int and arithmetic — all existing primitives.
    let src = r#"
let parse_secs s =
    let parts = str_split s "." in
    let whole = str_to_int (tuple_get parts 0) in
    let frac_str = tuple_get parts 1 in
    let frac_raw = str_to_int frac_str in
    let frac_len = str_len frac_str in
    let frac_ms = if frac_len == 1 then frac_raw * 100
                  else if frac_len == 2 then frac_raw * 10
                  else frac_raw in
    whole * 1000 + frac_ms
"#;
    let g = iris_bootstrap::syntax::compile(src);
    assert!(g.errors.is_empty(), "compile failed: {:?}", g.errors);
    let graph = g.fragments[0].1.graph.clone();
    let (out, _) = interpreter::interpret(&graph, &[Value::String("1.500".into())], None).unwrap();
    assert_eq!(out, vec![Value::Int(1500)]);
}

// ---------------------------------------------------------------------------
// 8. Elapsed accuracy — sleep 50ms and verify elapsed is in [40, 200]ms
// ---------------------------------------------------------------------------

#[test]
fn test_elapsed_accuracy() {
    let handler = RuntimeEffectHandler::new();

    let req1 = EffectRequest {
        tag: EffectTag::ClockNs,
        args: vec![],
    };
    let start_ns = match handler.handle(req1).unwrap() {
        Value::Int(n) => n,
        other => panic!("expected Int, got {:?}", other),
    };
    let start_ms = start_ns / 1_000_000;

    // Sleep 50ms
    let sleep_req = EffectRequest {
        tag: EffectTag::SleepMs,
        args: vec![Value::Int(50)],
    };
    handler.handle(sleep_req).unwrap();

    let req2 = EffectRequest {
        tag: EffectTag::ClockNs,
        args: vec![],
    };
    let end_ns = match handler.handle(req2).unwrap() {
        Value::Int(n) => n,
        other => panic!("expected Int, got {:?}", other),
    };
    let end_ms = end_ns / 1_000_000;
    let elapsed = end_ms - start_ms;

    assert!(
        elapsed >= 40,
        "elapsed should be >= 40ms after 50ms sleep, got {}ms",
        elapsed
    );
    assert!(
        elapsed <= 200,
        "elapsed should be <= 200ms after 50ms sleep, got {}ms",
        elapsed
    );
}

// ---------------------------------------------------------------------------
// 9. Format duration — seconds (>= 1000ms)
// ---------------------------------------------------------------------------

#[test]
fn test_format_duration_seconds() {
    // time_format 1500ms as %ms should give "1500"
    // To test the IRIS format_duration logic: 1500 >= 1000, so format as seconds.
    // secs = 1500 / 1000 = 1, frac = 500, padded = "500", result = "1.500s"
    // We compile the IRIS format_duration function.
    let src = r#"
let f ms =
    if ms >= 1000 then
        let secs = ms / 1000 in
        let frac = ms - (secs * 1000) in
        let frac_str = int_to_string frac in
        let padded = if frac < 10 then str_concat "00" frac_str
                     else if frac < 100 then str_concat "0" frac_str
                     else frac_str in
        str_concat (str_concat (int_to_string secs) (str_concat "." padded)) "s"
    else
        str_concat (int_to_string ms) "ms"
"#;
    let g = iris_bootstrap::syntax::compile(src);
    assert!(g.errors.is_empty(), "compile failed: {:?}", g.errors);
    let graph = g.fragments[0].1.graph.clone();
    let (out, _) = interpreter::interpret(&graph, &[Value::Int(1500)], None).unwrap();
    assert_eq!(out, vec![Value::String("1.500s".into())]);
}

// ---------------------------------------------------------------------------
// 10. Format duration — milliseconds (< 1000ms)
// ---------------------------------------------------------------------------

#[test]
fn test_format_duration_milliseconds() {
    let src = r#"
let f ms =
    if ms >= 1000 then
        let secs = ms / 1000 in
        let frac = ms - (secs * 1000) in
        let frac_str = int_to_string frac in
        let padded = if frac < 10 then str_concat "00" frac_str
                     else if frac < 100 then str_concat "0" frac_str
                     else frac_str in
        str_concat (str_concat (int_to_string secs) (str_concat "." padded)) "s"
    else
        str_concat (int_to_string ms) "ms"
"#;
    let g = iris_bootstrap::syntax::compile(src);
    assert!(g.errors.is_empty(), "compile failed: {:?}", g.errors);
    let graph = g.fragments[0].1.graph.clone();
    let (out, _) = interpreter::interpret(&graph, &[Value::Int(42)], None).unwrap();
    assert_eq!(out, vec![Value::String("42ms".into())]);
}

// ---------------------------------------------------------------------------
// 11. Format duration — zero
// ---------------------------------------------------------------------------

#[test]
fn test_format_duration_zero() {
    let src = r#"
let f ms =
    if ms >= 1000 then
        let secs = ms / 1000 in
        let frac = ms - (secs * 1000) in
        let frac_str = int_to_string frac in
        let padded = if frac < 10 then str_concat "00" frac_str
                     else if frac < 100 then str_concat "0" frac_str
                     else frac_str in
        str_concat (str_concat (int_to_string secs) (str_concat "." padded)) "s"
    else
        str_concat (int_to_string ms) "ms"
"#;
    let g = iris_bootstrap::syntax::compile(src);
    assert!(g.errors.is_empty(), "compile failed: {:?}", g.errors);
    let graph = g.fragments[0].1.graph.clone();
    let (out, _) = interpreter::interpret(&graph, &[Value::Int(0)], None).unwrap();
    assert_eq!(out, vec![Value::String("0ms".into())]);
}

// ---------------------------------------------------------------------------
// 12. now_increases — two consecutive clock calls, second >= first
// ---------------------------------------------------------------------------

#[test]
fn test_now_increases() {
    let handler = RuntimeEffectHandler::new();

    let req1 = EffectRequest {
        tag: EffectTag::ClockNs,
        args: vec![],
    };
    let ns1 = match handler.handle(req1).unwrap() {
        Value::Int(n) => n,
        other => panic!("expected Int, got {:?}", other),
    };

    let req2 = EffectRequest {
        tag: EffectTag::ClockNs,
        args: vec![],
    };
    let ns2 = match handler.handle(req2).unwrap() {
        Value::Int(n) => n,
        other => panic!("expected Int, got {:?}", other),
    };

    assert!(ns2 >= ns1, "second clock_ns ({}) should be >= first ({})", ns2, ns1);
}

// ---------------------------------------------------------------------------
// 13. Integration: timing pipeline — time a computation, format result
// ---------------------------------------------------------------------------

#[test]
fn test_timing_pipeline() {
    let handler = RuntimeEffectHandler::new();

    // Get start time.
    let req_start = EffectRequest {
        tag: EffectTag::ClockNs,
        args: vec![],
    };
    let start_ns = match handler.handle(req_start).unwrap() {
        Value::Int(n) => n,
        other => panic!("expected Int, got {:?}", other),
    };
    let start_ms = start_ns / 1_000_000;

    // Sleep for 10ms to simulate computation.
    let sleep_req = EffectRequest {
        tag: EffectTag::SleepMs,
        args: vec![Value::Int(10)],
    };
    handler.handle(sleep_req).unwrap();

    // Get end time.
    let req_end = EffectRequest {
        tag: EffectTag::ClockNs,
        args: vec![],
    };
    let end_ns = match handler.handle(req_end).unwrap() {
        Value::Int(n) => n,
        other => panic!("expected Int, got {:?}", other),
    };
    let end_ms = end_ns / 1_000_000;
    let elapsed_ms = end_ms - start_ms;

    // Format via the IRIS format_duration function.
    let src = r#"
let f ms =
    if ms >= 1000 then
        let secs = ms / 1000 in
        let frac = ms - (secs * 1000) in
        let frac_str = int_to_string frac in
        let padded = if frac < 10 then str_concat "00" frac_str
                     else if frac < 100 then str_concat "0" frac_str
                     else frac_str in
        str_concat (str_concat (int_to_string secs) (str_concat "." padded)) "s"
    else
        str_concat (int_to_string ms) "ms"
"#;
    let compile_result = iris_bootstrap::syntax::compile(src);
    assert!(compile_result.errors.is_empty());
    let graph = compile_result.fragments[0].1.graph.clone();
    let (out, _) = interpreter::interpret(&graph, &[Value::Int(elapsed_ms)], None).unwrap();

    match &out[0] {
        Value::String(s) => {
            // elapsed should be < 1000ms so should end with "ms".
            assert!(
                s.ends_with("ms"),
                "formatted duration should end with 'ms' for < 1s, got: {}",
                s
            );
        }
        other => panic!("expected String, got {:?}", other),
    }
}
