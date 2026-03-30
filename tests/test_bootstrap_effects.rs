//! Tests for the bootstrap's EffectHandler integration.
//!
//! Verifies that `evaluate_with_effects` correctly dispatches Effect nodes
//! through an EffectHandler, enabling IRIS programs to perform I/O,
//! threading, JIT, and FFI via the bootstrap's opcode dispatch.

use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

use iris_bootstrap::evaluate_with_effects;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::{EffectError, EffectHandler, EffectRequest, EffectTag, Value};
use iris_types::fragment::FragmentId;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Test EffectHandler: captures all effect requests for inspection
// ---------------------------------------------------------------------------

struct TestHandler {
    log: Mutex<Vec<(u8, Vec<Value>)>>,
}

impl TestHandler {
    fn new() -> Self {
        Self {
            log: Mutex::new(Vec::new()),
        }
    }

    fn entries(&self) -> Vec<(u8, Vec<Value>)> {
        self.log.lock().unwrap().clone()
    }
}

impl EffectHandler for TestHandler {
    fn handle(&self, request: EffectRequest) -> Result<Value, EffectError> {
        let tag = request.tag.to_u8();
        self.log
            .lock()
            .unwrap()
            .push((tag, request.args.clone()));

        match request.tag {
            EffectTag::Print => Ok(Value::Unit),
            EffectTag::Timestamp => Ok(Value::Int(1234567890)),
            EffectTag::ClockNs => Ok(Value::Int(1234567890_000_000_000)),
            EffectTag::Random => Ok(Value::Int(42)),
            EffectTag::RandomBytes => {
                let count = match request.args.first() {
                    Some(Value::Int(n)) => *n as usize,
                    _ => 4,
                };
                Ok(Value::Bytes(vec![0xAB; count]))
            }
            EffectTag::FileRead => Ok(Value::String("file contents".to_string())),
            EffectTag::FileWrite => Ok(Value::Unit),
            EffectTag::SleepMs => Ok(Value::Unit),
            _ => Ok(Value::Int(0)),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_node(id: u64, kind: NodeKind, payload: NodePayload) -> (NodeId, Node) {
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind,
            type_sig: TypeId(0),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload,
        },
    )
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

fn make_effect_graph(effect_tag: u8) -> SemanticGraph {
    let (nid, node) = make_node(1, NodeKind::Effect, NodePayload::Effect { effect_tag });
    let mut nodes = HashMap::new();
    nodes.insert(nid, node);
    make_graph(nodes, vec![], 1)
}

fn make_effect_graph_with_string_arg(effect_tag: u8, arg: &str) -> SemanticGraph {
    let (root_id, root) = make_node(1, NodeKind::Effect, NodePayload::Effect { effect_tag });
    let (arg_id, arg_node) = make_node(
        2,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x07,
            value: arg.as_bytes().to_vec(),
        },
    );
    let mut nodes = HashMap::new();
    nodes.insert(root_id, root);
    nodes.insert(arg_id, arg_node);
    make_graph(
        nodes,
        vec![Edge {
            source: NodeId(1),
            target: NodeId(2),
            port: 0,
            label: EdgeLabel::Argument,
        }],
        1,
    )
}

fn empty_registry() -> BTreeMap<FragmentId, SemanticGraph> {
    BTreeMap::new()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_effect_print_dispatched_through_handler() {
    let handler = TestHandler::new();
    let graph = make_effect_graph_with_string_arg(0x00, "hello");
    let registry = empty_registry();

    let result = evaluate_with_effects(&graph, &[], 1000, &registry, &handler);
    assert!(result.is_ok(), "print effect should succeed: {:?}", result);
    assert_eq!(result.unwrap(), Value::Unit);

    let entries = handler.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, 0x00);
    assert_eq!(entries[0].1, vec![Value::String("hello".to_string())]);
}

#[test]
fn test_effect_timestamp_dispatched_through_handler() {
    let handler = TestHandler::new();
    let graph = make_effect_graph(0x09);
    let registry = empty_registry();

    let result = evaluate_with_effects(&graph, &[], 1000, &registry, &handler);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(1234567890));

    let entries = handler.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, 0x09);
}

#[test]
fn test_effect_file_read_dispatched_through_handler() {
    let handler = TestHandler::new();
    let graph = make_effect_graph_with_string_arg(0x04, "/tmp/test.txt");
    let registry = empty_registry();

    let result = evaluate_with_effects(&graph, &[], 1000, &registry, &handler);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Value::String("file contents".to_string())
    );
}

#[test]
fn test_effect_clock_ns_dispatched_through_handler() {
    let handler = TestHandler::new();
    let graph = make_effect_graph(0x1D);
    let registry = empty_registry();

    let result = evaluate_with_effects(&graph, &[], 1000, &registry, &handler);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(1234567890_000_000_000));
}

#[test]
fn test_effect_random_dispatched_through_handler() {
    let handler = TestHandler::new();
    let graph = make_effect_graph(0x0A);
    let registry = empty_registry();

    let result = evaluate_with_effects(&graph, &[], 1000, &registry, &handler);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));
}

#[test]
fn test_unknown_effect_without_handler_fails() {
    let graph = make_effect_graph(0x10); // TcpConnect — not built-in
    let result = iris_bootstrap::evaluate_with_limit(&graph, &[], 1000);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("0x10"));
}

#[test]
fn test_unknown_effect_with_handler_succeeds() {
    let handler = TestHandler::new();
    let graph = make_effect_graph(0x10); // TcpConnect
    let registry = empty_registry();

    let result = evaluate_with_effects(&graph, &[], 1000, &registry, &handler);
    assert!(result.is_ok());

    let entries = handler.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, 0x10);
}

#[test]
fn test_mmap_exec_effect_dispatched() {
    let handler = TestHandler::new();
    let graph = make_effect_graph(0x29);
    let registry = empty_registry();

    let result = evaluate_with_effects(&graph, &[], 1000, &registry, &handler);
    assert!(result.is_ok());

    let entries = handler.entries();
    assert_eq!(entries[0].0, 0x29);
}

#[test]
fn test_thread_spawn_effect_dispatched() {
    let handler = TestHandler::new();
    let graph = make_effect_graph(0x20);
    let registry = empty_registry();

    let result = evaluate_with_effects(&graph, &[], 1000, &registry, &handler);
    assert!(result.is_ok());

    let entries = handler.entries();
    assert_eq!(entries[0].0, 0x20);
}

#[test]
fn test_ffi_call_effect_dispatched() {
    let handler = TestHandler::new();
    let graph = make_effect_graph(0x2B);
    let registry = empty_registry();

    let result = evaluate_with_effects(&graph, &[], 1000, &registry, &handler);
    assert!(result.is_ok());

    let entries = handler.entries();
    assert_eq!(entries[0].0, 0x2B);
}

#[test]
fn test_multiple_effects_in_sequence() {
    let handler = TestHandler::new();

    // Build: let _ = print("hi") in timestamp()
    let (root_id, root) = make_node(1, NodeKind::Let, NodePayload::Let);
    let (print_id, print_node) =
        make_node(2, NodeKind::Effect, NodePayload::Effect { effect_tag: 0x00 });
    let (ts_id, ts_node) =
        make_node(3, NodeKind::Effect, NodePayload::Effect { effect_tag: 0x09 });
    let (arg_id, arg_node) = make_node(
        4,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x07,
            value: b"hi".to_vec(),
        },
    );

    let mut nodes = HashMap::new();
    nodes.insert(root_id, root);
    nodes.insert(print_id, print_node);
    nodes.insert(ts_id, ts_node);
    nodes.insert(arg_id, arg_node);

    let graph = make_graph(
        nodes,
        vec![
            Edge {
                source: NodeId(1),
                target: NodeId(2),
                port: 0,
                label: EdgeLabel::Binding,
            },
            Edge {
                source: NodeId(1),
                target: NodeId(3),
                port: 0,
                label: EdgeLabel::Continuation,
            },
            Edge {
                source: NodeId(2),
                target: NodeId(4),
                port: 0,
                label: EdgeLabel::Argument,
            },
        ],
        1,
    );

    let registry = empty_registry();
    let result = evaluate_with_effects(&graph, &[], 1000, &registry, &handler);
    assert!(result.is_ok(), "sequence should succeed: {:?}", result);
    assert_eq!(result.unwrap(), Value::Int(1234567890));

    let entries = handler.entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].0, 0x00); // Print first
    assert_eq!(entries[1].0, 0x09); // Then Timestamp
}
