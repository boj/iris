
//! Integration tests for the IRIS algebraic effect system.
//!
//! Tests verify that:
//! - Effect nodes invoke the handler and return its result
//! - LoggingHandler captures effect requests for inspection
//! - NoOpHandler returns Unit for everything
//! - RuntimeEffectHandler performs actual I/O (Timestamp, Random)
//! - SandboxedHandler enforces allow-lists
//! - Effect results flow back into the computation graph

use std::collections::{BTreeMap, HashMap};

use iris_exec::effect_runtime::{LoggingHandler, NoOpHandler, RuntimeEffectHandler};
use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::{EffectTag, Value};
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::{compute_node_id, SemanticHash};
use iris_types::types::{PrimType, TypeDef, TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Helpers (same pattern as builtins.rs)
// ---------------------------------------------------------------------------

fn int_type_env() -> (TypeEnv, TypeId) {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = iris_types::hash::compute_type_id(&int_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    (TypeEnv { types }, int_id)
}

fn make_node(kind: NodeKind, payload: NodePayload, type_sig: TypeId, arity: u8) -> Node {
    let mut node = Node {
        id: NodeId(0),
        kind,
        type_sig,
        cost: CostTerm::Unit,
        arity,
        resolution_depth: 2, salt: 0,
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

/// Build a minimal graph with a single Effect node (no arguments).
fn make_effect_graph(effect_tag: u8) -> SemanticGraph {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let edges = Vec::new();

    let effect_node = make_node(
        NodeKind::Effect,
        NodePayload::Effect { effect_tag },
        int_id,
        0,
    );
    let root_id = effect_node.id;
    nodes.insert(root_id, effect_node);

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

/// Build a graph with an Effect node that has argument edges pointing to
/// literal nodes.
fn make_effect_graph_with_args(effect_tag: u8, lit_values: &[Value]) -> SemanticGraph {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    let effect_node = make_node(
        NodeKind::Effect,
        NodePayload::Effect { effect_tag },
        int_id,
        lit_values.len() as u8,
    );
    let root_id = effect_node.id;
    nodes.insert(root_id, effect_node);

    for (i, val) in lit_values.iter().enumerate() {
        let (type_tag, value_bytes) = match val {
            Value::Int(v) => (0x00u8, v.to_le_bytes().to_vec()),
            Value::Bytes(b) => (0x05u8, b.clone()),
            _ => (0x06u8, vec![]), // Unit
        };
        let lit_node = make_node(
            NodeKind::Lit,
            NodePayload::Lit {
                type_tag,
                value: value_bytes,
            },
            int_id,
            0,
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

/// Build a graph: Prim(add, Effect(Timestamp), Lit(1))
/// This tests that the effect result flows back into computation.
fn make_effect_plus_lit_graph() -> SemanticGraph {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Effect(Timestamp) — tag 0x09
    let effect_node = make_node(
        NodeKind::Effect,
        NodePayload::Effect { effect_tag: 0x09 },
        int_id,
        0,
    );
    let effect_id = effect_node.id;
    nodes.insert(effect_id, effect_node);

    // Lit(1)
    let lit_node = make_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: 1i64.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let lit_id = lit_node.id;
    nodes.insert(lit_id, lit_node);

    // Prim(add) — opcode 0x00
    let add_node = make_node(
        NodeKind::Prim,
        NodePayload::Prim { opcode: 0x00 },
        int_id,
        2,
    );
    let add_id = add_node.id;
    nodes.insert(add_id, add_node);

    // add -> effect (port 0)
    edges.push(Edge {
        source: add_id,
        target: effect_id,
        port: 0,
        label: EdgeLabel::Argument,
    });
    // add -> lit (port 1)
    edges.push(Edge {
        source: add_id,
        target: lit_id,
        port: 1,
        label: EdgeLabel::Argument,
    });

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: add_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_print_effect_with_logging_handler() {
    // Build a graph: Effect(Print, Lit("hello"))
    let graph = make_effect_graph_with_args(
        0x00, // Print
        &[Value::Bytes(b"hello".to_vec())],
    );

    let handler = LoggingHandler::new();
    let (outputs, _state) = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        Some(&handler),
    )
    .expect("interpret should succeed");

    // Print returns Unit.
    assert_eq!(outputs, vec![Value::Unit]);

    // The handler should have captured one Print request.
    let entries = handler.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].tag, EffectTag::Print);
    assert_eq!(entries[0].args, vec![Value::Bytes(b"hello".to_vec())]);
}

#[test]
fn test_timestamp_effect_returns_reasonable_value() {
    let graph = make_effect_graph(0x09); // Timestamp

    let handler = RuntimeEffectHandler::new();
    let (outputs, _state) = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        Some(&handler),
    )
    .expect("interpret should succeed");

    // Timestamp returns an Int (Unix ms).
    assert_eq!(outputs.len(), 1);
    match &outputs[0] {
        Value::Int(ms) => {
            // Should be a reasonable timestamp (after year 2020 = ~1577836800000 ms).
            assert!(
                *ms > 1_577_836_800_000,
                "timestamp {} is unreasonably small",
                ms
            );
        }
        other => panic!("expected Int, got {:?}", other),
    }
}

#[test]
fn test_random_effect_returns_different_values() {
    let graph = make_effect_graph(0x0A); // Random

    let handler = RuntimeEffectHandler::new();

    let (out1, _) = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        Some(&handler),
    )
    .expect("first call should succeed");

    // Brief pause so the nanos-based seed differs.
    std::thread::sleep(std::time::Duration::from_millis(1));

    let (out2, _) = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        Some(&handler),
    )
    .expect("second call should succeed");

    // Both should be Int values.
    assert!(matches!(&out1[0], Value::Int(_)));
    assert!(matches!(&out2[0], Value::Int(_)));

    // They should differ (with overwhelming probability).
    assert_ne!(
        out1[0], out2[0],
        "two random calls should produce different values"
    );
}

#[test]
fn test_noop_handler_returns_unit() {
    let graph = make_effect_graph(0x09); // Timestamp

    let handler = NoOpHandler;
    let (outputs, _state) = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        Some(&handler),
    )
    .expect("interpret should succeed");

    // NoOpHandler always returns Unit, even for Timestamp.
    assert_eq!(outputs, vec![Value::Unit]);
}

#[test]
fn test_noop_handler_returns_unit_for_all_tags() {
    for tag in 0x00..=0x0D {
        let graph = make_effect_graph(tag);
        let handler = NoOpHandler;
        let (outputs, _state) = interpreter::interpret_with_effects(
            &graph,
            &[],
            None,
            None,
            100_000,
            Some(&handler),
        )
        .expect(&format!("interpret should succeed for tag 0x{:02x}", tag));
        assert_eq!(
            outputs,
            vec![Value::Unit],
            "NoOpHandler should return Unit for tag 0x{:02x}",
            tag
        );
    }
}

#[test]
fn test_effect_result_flows_into_computation() {
    // Graph: add(Effect(Timestamp), Lit(1))
    // With RuntimeEffectHandler, Effect(Timestamp) returns an Int.
    // The add node adds 1 to the timestamp.
    let graph = make_effect_plus_lit_graph();

    let handler = RuntimeEffectHandler::new();
    let (outputs, _state) = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        Some(&handler),
    )
    .expect("interpret should succeed");

    assert_eq!(outputs.len(), 1);
    match &outputs[0] {
        Value::Int(val) => {
            // The result should be timestamp + 1, so it must be > timestamp alone.
            assert!(
                *val > 1_577_836_800_000,
                "result {} should be a timestamp + 1",
                val
            );
        }
        other => panic!("expected Int, got {:?}", other),
    }
}

#[test]
fn test_no_handler_fallback_handles_timestamp() {
    // When no handler is provided, the bootstrap evaluator's built-in
    // fallback handles Timestamp (returns real time as Int) and Print.
    let graph = make_effect_graph(0x09); // Timestamp

    let (outputs, _state) = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        None, // no handler — bootstrap fallback handles timestamp
    )
    .expect("interpret should succeed");

    match &outputs[0] {
        Value::Int(ms) => assert!(*ms > 1_577_836_800_000, "timestamp should be a real epoch ms"),
        other => panic!("expected Int from bootstrap timestamp fallback, got {:?}", other),
    }
}

#[test]
fn test_backward_compat_interpret() {
    // The original `interpret()` function (no handler parameter) should
    // still work — Timestamp returns real time via bootstrap fallback.
    let graph = make_effect_graph(0x09);

    let (outputs, _state) =
        interpreter::interpret(&graph, &[], None).expect("interpret should succeed");

    match &outputs[0] {
        Value::Int(ms) => assert!(*ms > 1_577_836_800_000),
        other => panic!("expected Int, got {:?}", other),
    }
}

#[test]
fn test_logging_handler_captures_multiple_effects() {
    // Build two separate effect graphs isn't practical in a single graph,
    // so we test by calling twice and checking that entries accumulate.
    let graph1 = make_effect_graph_with_args(0x00, &[Value::Int(42)]);
    let graph2 = make_effect_graph_with_args(0x0B, &[Value::Int(99)]);

    let handler = LoggingHandler::new();

    interpreter::interpret_with_effects(&graph1, &[], None, None, 100_000, Some(&handler))
        .expect("first effect should succeed");
    interpreter::interpret_with_effects(&graph2, &[], None, None, 100_000, Some(&handler))
        .expect("second effect should succeed");

    let entries = handler.entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].tag, EffectTag::Print);
    assert_eq!(entries[0].args, vec![Value::Int(42)]);
    assert_eq!(entries[1].tag, EffectTag::Log);
    assert_eq!(entries[1].args, vec![Value::Int(99)]);
}

#[test]
fn test_sandboxed_handler_allows_permitted_effects() {
    let graph = make_effect_graph(0x09); // Timestamp

    // Allow only Timestamp via CapabilityGuardHandler.
    let mut caps = iris_exec::capabilities::Capabilities::sandbox();
    caps.allowed_effects.clear();
    caps.allowed_effects.insert(EffectTag::Timestamp);
    let inner = RuntimeEffectHandler::new();
    let handler = iris_exec::capabilities::CapabilityGuardHandler::new(&inner, caps);
    let (outputs, _state) = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        Some(&handler),
    )
    .expect("interpret should succeed");

    match &outputs[0] {
        Value::Int(ms) => assert!(*ms > 1_577_836_800_000),
        other => panic!("expected Int, got {:?}", other),
    }
}

#[test]
fn test_sandboxed_handler_blocks_unpermitted_effects() {
    let graph = make_effect_graph(0x0A); // Random

    // Allow only Timestamp — Random is blocked.
    let mut caps = iris_exec::capabilities::Capabilities::sandbox();
    caps.allowed_effects.clear();
    caps.allowed_effects.insert(EffectTag::Timestamp);
    let inner = RuntimeEffectHandler::new();
    let handler = iris_exec::capabilities::CapabilityGuardHandler::new(&inner, caps);
    let result = interpreter::interpret_with_effects(
        &graph,
        &[],
        None,
        None,
        100_000,
        Some(&handler),
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    // Bootstrap wraps handler errors as TypeError("effect 0x.. failed: ...").
    let err_str = format!("{}", err);
    assert!(
        err_str.contains("permission denied") || err_str.contains("not allowed"),
        "error should mention permissions: {}",
        err_str
    );
}

#[test]
fn test_effect_tag_round_trip() {
    // Verify that EffectTag round-trips through u8.
    let tags = vec![
        EffectTag::Print,
        EffectTag::ReadLine,
        EffectTag::HttpGet,
        EffectTag::HttpPost,
        EffectTag::FileRead,
        EffectTag::FileWrite,
        EffectTag::DbQuery,
        EffectTag::DbExecute,
        EffectTag::Sleep,
        EffectTag::Timestamp,
        EffectTag::Random,
        EffectTag::Log,
        EffectTag::SendMessage,
        EffectTag::RecvMessage,
        EffectTag::Custom(0x42),
    ];

    for tag in tags {
        let byte = tag.to_u8();
        let recovered = EffectTag::from_u8(byte);
        assert_eq!(tag, recovered, "round-trip failed for {:?}", tag);
    }
}

#[test]
fn test_logging_handler_clear() {
    let graph = make_effect_graph(0x00);
    let handler = LoggingHandler::new();

    interpreter::interpret_with_effects(&graph, &[], None, None, 100_000, Some(&handler))
        .unwrap();
    assert_eq!(handler.entries().len(), 1);

    handler.clear();
    assert_eq!(handler.entries().len(), 0);
}
