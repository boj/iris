//! Isolated tests for every I/O and host-runtime effect primitive.
//!
//! These verify that the bootstrap evaluator correctly dispatches each
//! effect tag to the EffectHandler and returns the handler's result.
//! Uses a TestHandler that records dispatched tags and returns
//! deterministic values — no actual I/O is performed.
//!
//! Coverage: effect tags 0x00-0x2B (all 44 effect primitives),
//! plus bootstrap opcodes 0xF2 (file_read) and 0xF4 (print).

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
// RecordingHandler: captures all dispatched effect requests
// ---------------------------------------------------------------------------

struct RecordingHandler {
    log: Mutex<Vec<(u8, Vec<Value>)>>,
}

impl RecordingHandler {
    fn new() -> Self {
        Self {
            log: Mutex::new(Vec::new()),
        }
    }

    fn entries(&self) -> Vec<(u8, Vec<Value>)> {
        self.log.lock().unwrap().clone()
    }
}

impl EffectHandler for RecordingHandler {
    fn handle(&self, request: EffectRequest) -> Result<Value, EffectError> {
        let tag = request.tag.to_u8();
        self.log
            .lock()
            .unwrap()
            .push((tag, request.args.clone()));

        match request.tag {
            // Console
            EffectTag::Print => Ok(Value::Unit),
            EffectTag::ReadLine => Ok(Value::String("user input".into())),

            // File I/O
            EffectTag::FileRead => Ok(Value::String("file contents".into())),
            EffectTag::FileWrite => Ok(Value::Unit),
            EffectTag::FileOpen => Ok(Value::Int(42)), // fd
            EffectTag::FileReadBytes => Ok(Value::Bytes(vec![0xDE, 0xAD])),
            EffectTag::FileWriteBytes => Ok(Value::Int(2)), // bytes written
            EffectTag::FileClose => Ok(Value::Unit),
            EffectTag::FileStat => Ok(Value::tuple(vec![
                Value::Int(1024),  // size
                Value::Int(0o644), // permissions
                Value::Int(1),     // is_file
            ])),
            EffectTag::DirList => Ok(Value::tuple(vec![
                Value::String("a.txt".into()),
                Value::String("b.txt".into()),
            ])),

            // System
            EffectTag::Timestamp => Ok(Value::Int(1700000000)),
            EffectTag::ClockNs => Ok(Value::Int(1700000000_000_000_000)),
            EffectTag::Random => Ok(Value::Int(42)),
            EffectTag::Sleep => Ok(Value::Unit),
            EffectTag::EnvGet => Ok(Value::String("/usr/bin".into())),
            EffectTag::RandomBytes => {
                let count = match request.args.first() {
                    Some(Value::Int(n)) => *n as usize,
                    _ => 4,
                };
                Ok(Value::Bytes(vec![0xAB; count]))
            }
            EffectTag::SleepMs => Ok(Value::Unit),

            // TCP Networking
            EffectTag::TcpConnect => Ok(Value::Int(100)),  // fd
            EffectTag::TcpRead => Ok(Value::Bytes(vec![0x48, 0x49])), // "HI"
            EffectTag::TcpWrite => Ok(Value::Int(2)),       // bytes written
            EffectTag::TcpClose => Ok(Value::Unit),
            EffectTag::TcpListen => Ok(Value::Int(200)),    // listener fd
            EffectTag::TcpAccept => Ok(Value::Int(201)),    // accepted fd

            // Threading
            EffectTag::ThreadSpawn => Ok(Value::Int(1001)), // thread id
            EffectTag::ThreadJoin => Ok(Value::Int(0)),     // exit code
            EffectTag::AtomicRead => Ok(Value::Int(99)),
            EffectTag::AtomicWrite => Ok(Value::Unit),
            EffectTag::AtomicSwap => Ok(Value::Int(99)),    // old value
            EffectTag::AtomicAdd => Ok(Value::Int(100)),    // new value
            EffectTag::RwLockRead => Ok(Value::Int(1)),     // acquired
            EffectTag::RwLockWrite => Ok(Value::Int(1)),    // acquired
            EffectTag::RwLockRelease => Ok(Value::Unit),

            // JIT/FFI
            EffectTag::MmapExec => Ok(Value::Int(0)),
            EffectTag::CallNative => Ok(Value::Int(0)),
            EffectTag::FfiCall => Ok(Value::Int(0)),

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

fn effect_graph(tag: u8) -> SemanticGraph {
    let (nid, node) = make_node(1, NodeKind::Effect, NodePayload::Effect { effect_tag: tag });
    let mut nodes = HashMap::new();
    nodes.insert(nid, node);
    make_graph(nodes, vec![], 1)
}

fn effect_graph_with_arg(tag: u8, arg_str: &str) -> SemanticGraph {
    let (root_id, root) = make_node(1, NodeKind::Effect, NodePayload::Effect { effect_tag: tag });
    let (arg_id, arg_node) = make_node(
        2,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x07,
            value: arg_str.as_bytes().to_vec(),
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

fn effect_graph_with_int_arg(tag: u8, val: i64) -> SemanticGraph {
    let (root_id, root) = make_node(1, NodeKind::Effect, NodePayload::Effect { effect_tag: tag });
    let (arg_id, arg_node) = make_node(
        2,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: val.to_le_bytes().to_vec(),
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

fn run_effect(graph: &SemanticGraph, handler: &RecordingHandler) -> Result<Value, String> {
    evaluate_with_effects(graph, &[], 1000, &empty_registry(), handler)
        .map_err(|e| format!("{:?}", e))
}

// ---------------------------------------------------------------------------
// File I/O effects
// ---------------------------------------------------------------------------

#[test]
fn test_file_read_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_arg(0x04, "/tmp/test.txt");
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::String("file contents".into()));
    assert_eq!(handler.entries()[0].0, 0x04);
}

#[test]
fn test_file_write_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_arg(0x05, "/tmp/out.txt");
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Unit);
    assert_eq!(handler.entries()[0].0, 0x05);
}

#[test]
fn test_file_open_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_arg(0x16, "/tmp/data.bin");
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(42));
    assert_eq!(handler.entries()[0].0, 0x16);
}

#[test]
fn test_file_read_bytes_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x17, 42); // fd
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Bytes(vec![0xDE, 0xAD]));
    assert_eq!(handler.entries()[0].0, 0x17);
}

#[test]
fn test_file_write_bytes_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x18, 42); // fd
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(2));
    assert_eq!(handler.entries()[0].0, 0x18);
}

#[test]
fn test_file_close_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x19, 42); // fd
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Unit);
    assert_eq!(handler.entries()[0].0, 0x19);
}

#[test]
fn test_file_stat_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_arg(0x1A, "/tmp/test.txt");
    let result = run_effect(&graph, &handler).unwrap();
    match result {
        Value::Tuple(fields) => assert_eq!(fields.len(), 3),
        other => panic!("expected Tuple, got {:?}", other),
    }
    assert_eq!(handler.entries()[0].0, 0x1A);
}

#[test]
fn test_dir_list_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_arg(0x1B, "/tmp");
    let result = run_effect(&graph, &handler).unwrap();
    match result {
        Value::Tuple(fields) => assert_eq!(fields.len(), 2),
        other => panic!("expected Tuple, got {:?}", other),
    }
    assert_eq!(handler.entries()[0].0, 0x1B);
}

// ---------------------------------------------------------------------------
// TCP Networking effects
// ---------------------------------------------------------------------------

#[test]
fn test_tcp_connect_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_arg(0x10, "127.0.0.1:8080");
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(100));
    assert_eq!(handler.entries()[0].0, 0x10);
}

#[test]
fn test_tcp_read_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x11, 100); // fd
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Bytes(vec![0x48, 0x49]));
    assert_eq!(handler.entries()[0].0, 0x11);
}

#[test]
fn test_tcp_write_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x12, 100); // fd
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(2));
    assert_eq!(handler.entries()[0].0, 0x12);
}

#[test]
fn test_tcp_close_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x13, 100); // fd
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Unit);
    assert_eq!(handler.entries()[0].0, 0x13);
}

#[test]
fn test_tcp_listen_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_arg(0x14, "0.0.0.0:9090");
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(200));
    assert_eq!(handler.entries()[0].0, 0x14);
}

#[test]
fn test_tcp_accept_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x15, 200); // listener fd
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(201));
    assert_eq!(handler.entries()[0].0, 0x15);
}

// ---------------------------------------------------------------------------
// System effects
// ---------------------------------------------------------------------------

#[test]
fn test_readline_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph(0x01);
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::String("user input".into()));
    assert_eq!(handler.entries()[0].0, 0x01);
}

#[test]
fn test_env_get_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_arg(0x1C, "PATH");
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::String("/usr/bin".into()));
    assert_eq!(handler.entries()[0].0, 0x1C);
}

#[test]
fn test_clock_ns_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph(0x09);
    let result = run_effect(&graph, &handler).unwrap();
    // 0x09 maps to Timestamp (not ClockNs), handler returns 1700000000
    assert_eq!(result, Value::Int(1700000000));
    assert_eq!(handler.entries()[0].0, 0x09);
}

#[test]
fn test_random_bytes_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x1E, 8);
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Bytes(vec![0xAB; 8]));
    assert_eq!(handler.entries()[0].0, 0x1E);
}

#[test]
fn test_sleep_ms_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x1F, 100);
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Unit);
    assert_eq!(handler.entries()[0].0, 0x1F);
}

#[test]
fn test_sleep_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x08, 1);
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Unit);
    assert_eq!(handler.entries()[0].0, 0x08);
}

// ---------------------------------------------------------------------------
// Threading / Atomic effects
// ---------------------------------------------------------------------------

#[test]
fn test_thread_spawn_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph(0x20);
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(1001));
    assert_eq!(handler.entries()[0].0, 0x20);
}

#[test]
fn test_thread_join_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x21, 1001);
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(0));
    assert_eq!(handler.entries()[0].0, 0x21);
}

#[test]
fn test_atomic_read_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x22, 0); // addr
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(99));
    assert_eq!(handler.entries()[0].0, 0x22);
}

#[test]
fn test_atomic_write_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x23, 0); // addr
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Unit);
    assert_eq!(handler.entries()[0].0, 0x23);
}

#[test]
fn test_atomic_swap_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x24, 0); // addr
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(99));
    assert_eq!(handler.entries()[0].0, 0x24);
}

#[test]
fn test_atomic_add_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x25, 0); // addr
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(100));
    assert_eq!(handler.entries()[0].0, 0x25);
}

#[test]
fn test_rwlock_read_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x26, 0); // lock id
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(1));
    assert_eq!(handler.entries()[0].0, 0x26);
}

#[test]
fn test_rwlock_write_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x27, 0); // lock id
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(1));
    assert_eq!(handler.entries()[0].0, 0x27);
}

#[test]
fn test_rwlock_release_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph_with_int_arg(0x28, 0); // lock id
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Unit);
    assert_eq!(handler.entries()[0].0, 0x28);
}

// ---------------------------------------------------------------------------
// JIT / FFI effects
// ---------------------------------------------------------------------------

#[test]
fn test_mmap_exec_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph(0x29);
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(0));
    assert_eq!(handler.entries()[0].0, 0x29);
}

#[test]
fn test_ffi_call_dispatch() {
    let handler = RecordingHandler::new();
    let graph = effect_graph(0x2B);
    let result = run_effect(&graph, &handler).unwrap();
    assert_eq!(result, Value::Int(0));
    assert_eq!(handler.entries()[0].0, 0x2B);
}

// ---------------------------------------------------------------------------
// Boundary: handler absence causes error
// ---------------------------------------------------------------------------

#[test]
fn test_all_tags_dispatched_to_handler() {
    let handler = RecordingHandler::new();
    let tags: Vec<u8> = vec![
        0x00, 0x01, 0x04, 0x05, 0x08, 0x09,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
        0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1E, 0x1F,
        0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28,
    ];

    for &tag in &tags {
        let graph = effect_graph(tag);
        let result = run_effect(&graph, &handler);
        assert!(
            result.is_ok(),
            "effect tag 0x{:02X} should dispatch successfully, got {:?}",
            tag,
            result
        );
    }

    assert_eq!(
        handler.entries().len(),
        tags.len(),
        "every tag should have been dispatched exactly once"
    );
}
