//! Edge-case tests for CapabilityGuardHandler wiring and RuntimeEffectHandler.
//!
//! Verifies that:
//! - RuntimeEffectHandler handles file open/read/write/close lifecycle
//! - RuntimeEffectHandler returns errors for invalid handles
//! - RuntimeEffectHandler implements atomic state operations
//! - RuntimeEffectHandler handles EnvGet, FileStat, DirList, SleepMs
//! - CapabilityGuardHandler wraps &dyn EffectHandler (blanket impl)
//! - interpret_with_capabilities auto-creates handler and wraps with guard
//! - IrisExecutionService enforces sandbox capabilities on eval
//! - Error mapping extracts PermissionDenied from bootstrap TypeError

use std::collections::{BTreeMap, HashMap};

use iris_exec::capabilities::{Capabilities, CapabilityGuardHandler};
use iris_exec::effect_runtime::{LoggingHandler, NoOpHandler, RuntimeEffectHandler};
use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::{EffectError, EffectHandler, EffectRequest, EffectTag, Value};
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

fn make_node(kind: NodeKind, payload: NodePayload, type_sig: TypeId, arity: u8) -> Node {
    let mut node = Node {
        id: NodeId(0),
        kind,
        type_sig,
        cost: CostTerm::Unit,
        arity,
        resolution_depth: 2,
        salt: 0,
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

fn make_effect_graph(effect_tag: u8) -> SemanticGraph {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let effect_node = make_node(
        NodeKind::Effect,
        NodePayload::Effect { effect_tag },
        int_id,
        0,
    );
    let root_id = effect_node.id;
    nodes.insert(root_id, effect_node);
    let hash = compute_hash(&nodes, &[]);
    SemanticGraph {
        root: root_id,
        nodes,
        edges: vec![],
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

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
        let (type_tag, payload_bytes) = match val {
            Value::Int(n) => (0x00, n.to_le_bytes().to_vec()),
            Value::String(s) => (0x07, s.as_bytes().to_vec()),
            _ => (0xFF, vec![]),
        };
        let lit_node = make_node(
            NodeKind::Lit,
            NodePayload::Lit {
                type_tag,
                value: payload_bytes,
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

// ===========================================================================
// RuntimeEffectHandler: file handle lifecycle
// ===========================================================================

#[test]
fn runtime_file_open_read_close_lifecycle() {
    let handler = RuntimeEffectHandler::new();

    // Write a temp file.
    let path = "/tmp/iris_test_rt_lifecycle.txt";
    std::fs::write(path, b"hello iris").unwrap();

    // FileOpen(path, 0=read)
    let handle = handler
        .handle(EffectRequest {
            tag: EffectTag::FileOpen,
            args: vec![Value::String(path.into()), Value::Int(0)],
        })
        .unwrap();
    let handle_id = match handle {
        Value::Int(h) => h,
        other => panic!("expected Int handle, got {:?}", other),
    };
    assert!(handle_id > 0);

    // FileReadBytes(handle, 1024)
    let data = handler
        .handle(EffectRequest {
            tag: EffectTag::FileReadBytes,
            args: vec![Value::Int(handle_id), Value::Int(1024)],
        })
        .unwrap();
    match data {
        Value::Bytes(b) => assert_eq!(b, b"hello iris"),
        other => panic!("expected Bytes, got {:?}", other),
    }

    // FileClose(handle)
    let result = handler
        .handle(EffectRequest {
            tag: EffectTag::FileClose,
            args: vec![Value::Int(handle_id)],
        })
        .unwrap();
    assert_eq!(result, Value::Unit);

    // Reading from closed handle should error.
    let err = handler
        .handle(EffectRequest {
            tag: EffectTag::FileReadBytes,
            args: vec![Value::Int(handle_id), Value::Int(10)],
        })
        .unwrap_err();
    assert!(err.message.contains("invalid handle"));

    std::fs::remove_file(path).unwrap();
}

#[test]
fn runtime_file_write_bytes_lifecycle() {
    let handler = RuntimeEffectHandler::new();
    let path = "/tmp/iris_test_rt_write.txt";

    // FileOpen(path, 1=write)
    let handle = handler
        .handle(EffectRequest {
            tag: EffectTag::FileOpen,
            args: vec![Value::String(path.into()), Value::Int(1)],
        })
        .unwrap();
    let handle_id = match handle {
        Value::Int(h) => h,
        _ => panic!("expected handle"),
    };

    // FileWriteBytes(handle, data)
    let written = handler
        .handle(EffectRequest {
            tag: EffectTag::FileWriteBytes,
            args: vec![
                Value::Int(handle_id),
                Value::Bytes(b"test data".to_vec()),
            ],
        })
        .unwrap();
    match written {
        Value::Int(n) => assert_eq!(n, 9),
        other => panic!("expected Int bytes written, got {:?}", other),
    }

    handler
        .handle(EffectRequest {
            tag: EffectTag::FileClose,
            args: vec![Value::Int(handle_id)],
        })
        .unwrap();

    assert_eq!(std::fs::read_to_string(path).unwrap(), "test data");
    std::fs::remove_file(path).unwrap();
}

// ===========================================================================
// RuntimeEffectHandler: invalid handle errors
// ===========================================================================

#[test]
fn runtime_invalid_file_handle_errors() {
    let handler = RuntimeEffectHandler::new();

    let err = handler
        .handle(EffectRequest {
            tag: EffectTag::FileReadBytes,
            args: vec![Value::Int(9999), Value::Int(10)],
        })
        .unwrap_err();
    assert!(err.message.contains("invalid handle"));
    assert_eq!(err.tag, EffectTag::FileReadBytes);
}

#[test]
fn runtime_invalid_tcp_handle_errors() {
    let handler = RuntimeEffectHandler::new();

    let err = handler
        .handle(EffectRequest {
            tag: EffectTag::TcpRead,
            args: vec![Value::Int(9999), Value::Int(10)],
        })
        .unwrap_err();
    assert!(err.message.contains("invalid handle"));
}

// ===========================================================================
// RuntimeEffectHandler: FileStat, DirList, EnvGet
// ===========================================================================

#[test]
fn runtime_file_stat() {
    let handler = RuntimeEffectHandler::new();
    let path = "/tmp/iris_test_stat.txt";
    std::fs::write(path, b"12345").unwrap();

    let result = handler
        .handle(EffectRequest {
            tag: EffectTag::FileStat,
            args: vec![Value::String(path.into())],
        })
        .unwrap();

    match result {
        Value::Tuple(fields) => {
            assert_eq!(fields.len(), 3);
            match &fields[0] {
                Value::Int(size) => assert_eq!(*size, 5),
                other => panic!("expected size Int, got {:?}", other),
            }
            match &fields[2] {
                Value::Int(is_dir) => assert_eq!(*is_dir, 0),
                other => panic!("expected is_dir Int, got {:?}", other),
            }
        }
        other => panic!("expected Tuple, got {:?}", other),
    }

    std::fs::remove_file(path).unwrap();
}

#[test]
fn runtime_dir_list() {
    let handler = RuntimeEffectHandler::new();
    let dir = "/tmp/iris_test_dirlist";
    let _ = std::fs::create_dir_all(dir);
    std::fs::write(format!("{}/a.txt", dir), b"").unwrap();
    std::fs::write(format!("{}/b.txt", dir), b"").unwrap();

    let result = handler
        .handle(EffectRequest {
            tag: EffectTag::DirList,
            args: vec![Value::String(dir.into())],
        })
        .unwrap();

    match result {
        Value::Tuple(entries) => {
            let names: Vec<String> = entries
                .into_iter()
                .map(|v| match v {
                    Value::String(s) => s,
                    _ => panic!("expected String entry"),
                })
                .collect();
            assert!(names.contains(&"a.txt".to_string()));
            assert!(names.contains(&"b.txt".to_string()));
        }
        other => panic!("expected Tuple, got {:?}", other),
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn runtime_env_get_existing_var() {
    let handler = RuntimeEffectHandler::new();
    // HOME should always be set.
    let result = handler
        .handle(EffectRequest {
            tag: EffectTag::EnvGet,
            args: vec![Value::String("HOME".into())],
        })
        .unwrap();

    match result {
        Value::String(s) => assert!(!s.is_empty(), "HOME should be non-empty"),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn runtime_env_get_missing_var() {
    let handler = RuntimeEffectHandler::new();
    let result = handler
        .handle(EffectRequest {
            tag: EffectTag::EnvGet,
            args: vec![Value::String("IRIS_NONEXISTENT_VAR_12345".into())],
        })
        .unwrap();
    assert_eq!(result, Value::Unit);
}

// ===========================================================================
// RuntimeEffectHandler: atomic state
// ===========================================================================

#[test]
fn runtime_atomic_read_write() {
    let handler = RuntimeEffectHandler::new();

    // Read nonexistent key → Unit.
    let val = handler
        .handle(EffectRequest {
            tag: EffectTag::AtomicRead,
            args: vec![Value::String("counter".into())],
        })
        .unwrap();
    assert_eq!(val, Value::Unit);

    // Write key.
    handler
        .handle(EffectRequest {
            tag: EffectTag::AtomicWrite,
            args: vec![Value::String("counter".into()), Value::Int(42)],
        })
        .unwrap();

    // Read back.
    let val = handler
        .handle(EffectRequest {
            tag: EffectTag::AtomicRead,
            args: vec![Value::String("counter".into())],
        })
        .unwrap();
    assert_eq!(val, Value::Int(42));
}

#[test]
fn runtime_atomic_swap() {
    let handler = RuntimeEffectHandler::new();

    handler
        .handle(EffectRequest {
            tag: EffectTag::AtomicWrite,
            args: vec![Value::String("key".into()), Value::Int(10)],
        })
        .unwrap();

    let old = handler
        .handle(EffectRequest {
            tag: EffectTag::AtomicSwap,
            args: vec![Value::String("key".into()), Value::Int(20)],
        })
        .unwrap();
    assert_eq!(old, Value::Int(10));

    let current = handler
        .handle(EffectRequest {
            tag: EffectTag::AtomicRead,
            args: vec![Value::String("key".into())],
        })
        .unwrap();
    assert_eq!(current, Value::Int(20));
}

#[test]
fn runtime_atomic_add() {
    let handler = RuntimeEffectHandler::new();

    handler
        .handle(EffectRequest {
            tag: EffectTag::AtomicWrite,
            args: vec![Value::String("acc".into()), Value::Int(100)],
        })
        .unwrap();

    let old = handler
        .handle(EffectRequest {
            tag: EffectTag::AtomicAdd,
            args: vec![Value::String("acc".into()), Value::Int(5)],
        })
        .unwrap();
    assert_eq!(old, Value::Int(100));

    let current = handler
        .handle(EffectRequest {
            tag: EffectTag::AtomicRead,
            args: vec![Value::String("acc".into())],
        })
        .unwrap();
    assert_eq!(current, Value::Int(105));
}

// ===========================================================================
// RuntimeEffectHandler: SleepMs (minimal — just verify it doesn't error)
// ===========================================================================

#[test]
fn runtime_sleep_ms_returns_unit() {
    let handler = RuntimeEffectHandler::new();
    let result = handler
        .handle(EffectRequest {
            tag: EffectTag::SleepMs,
            args: vec![Value::Int(1)], // 1ms
        })
        .unwrap();
    assert_eq!(result, Value::Unit);
}

// ===========================================================================
// RuntimeEffectHandler: RandomBytes
// ===========================================================================

#[test]
fn runtime_random_bytes_correct_length() {
    let handler = RuntimeEffectHandler::new();
    let result = handler
        .handle(EffectRequest {
            tag: EffectTag::RandomBytes,
            args: vec![Value::Int(32)],
        })
        .unwrap();
    match result {
        Value::Bytes(b) => assert_eq!(b.len(), 32),
        other => panic!("expected Bytes, got {:?}", other),
    }
}

// ===========================================================================
// RuntimeEffectHandler: stubbed effects return errors (not panics)
// ===========================================================================

#[test]
fn runtime_stubbed_effects_return_errors() {
    let handler = RuntimeEffectHandler::new();

    for tag in &[
        EffectTag::HttpGet,
        EffectTag::HttpPost,
        EffectTag::DbQuery,
        EffectTag::DbExecute,
        EffectTag::SendMessage,
        EffectTag::RecvMessage,
        EffectTag::ThreadSpawn,
        EffectTag::ThreadJoin,
        EffectTag::MmapExec,
        EffectTag::CallNative,
        EffectTag::FfiCall,
    ] {
        let result = handler.handle(EffectRequest {
            tag: *tag,
            args: vec![],
        });
        assert!(
            result.is_err(),
            "stubbed effect {:?} should return Err, got {:?}",
            tag,
            result
        );
    }
}

// ===========================================================================
// CapabilityGuardHandler wraps &dyn EffectHandler (blanket impl)
// ===========================================================================

#[test]
fn guard_wraps_dyn_ref() {
    let runtime = RuntimeEffectHandler::new();
    let dyn_ref: &dyn EffectHandler = &runtime;
    let caps = Capabilities::unrestricted();

    // This tests the blanket impl: &T: EffectHandler where T: EffectHandler.
    let guard = CapabilityGuardHandler::new(dyn_ref, caps);

    let result = guard.handle(EffectRequest {
        tag: EffectTag::Timestamp,
        args: vec![],
    });
    assert!(result.is_ok());
    match result.unwrap() {
        Value::Int(ms) => assert!(ms > 1_577_836_800_000),
        other => panic!("expected Int, got {:?}", other),
    }
}

// ===========================================================================
// interpret_with_capabilities: auto-creates handler, wraps with guard
// ===========================================================================

#[test]
fn interpret_with_caps_blocks_disallowed_effect() {
    // Build a graph that does FileWrite (0x05).
    let graph = make_effect_graph(EffectTag::FileWrite.to_u8());
    let caps = Capabilities::sandbox(); // sandbox doesn't allow FileWrite

    let result = interpreter::interpret_with_capabilities(
        &graph,
        &[],
        None,
        None,
        None, // no inner handler — should auto-create RuntimeEffectHandler
        None,
        None,
        0,
        caps,
    );

    match result {
        Err(interpreter::InterpretError::PermissionDenied { effect }) => {
            assert_eq!(effect, EffectTag::FileWrite);
        }
        other => panic!("expected PermissionDenied for FileWrite, got {:?}", other),
    }
}

#[test]
fn interpret_with_caps_allows_timestamp_in_sandbox() {
    // Sandbox allows Timestamp.
    let graph = make_effect_graph(EffectTag::Timestamp.to_u8());
    let caps = Capabilities::sandbox();

    let (outputs, _state) = interpreter::interpret_with_capabilities(
        &graph,
        &[],
        None,
        None,
        None,
        None,
        None,
        0,
        caps,
    )
    .expect("sandbox should allow Timestamp");

    match &outputs[0] {
        Value::Int(ms) => assert!(*ms > 1_577_836_800_000),
        other => panic!("expected Int, got {:?}", other),
    }
}

#[test]
fn interpret_with_caps_wraps_provided_handler() {
    // Provide a LoggingHandler as inner handler. The guard should wrap it.
    let logger = LoggingHandler::new();
    let caps = Capabilities::sandbox(); // allows Timestamp

    let graph = make_effect_graph(EffectTag::Timestamp.to_u8());
    let result = interpreter::interpret_with_capabilities(
        &graph,
        &[],
        None,
        None,
        Some(&logger),
        None,
        None,
        0,
        caps,
    );

    assert!(result.is_ok());
    // The logger should have captured the request (it was the inner handler).
    assert_eq!(logger.len(), 1);
    assert_eq!(logger.requests()[0].tag, EffectTag::Timestamp);
}

// ===========================================================================
// Error mapping: PermissionDenied extraction
// ===========================================================================

#[test]
fn error_mapping_extracts_permission_denied() {
    // When CapabilityGuardHandler blocks an effect, bootstrap wraps it as
    // TypeError("effect 0xNN failed: ... permission denied ...").
    // The interpreter should map that back to PermissionDenied.
    let graph = make_effect_graph(EffectTag::TcpConnect.to_u8());
    let caps = Capabilities::sandbox(); // blocks TCP

    let result = interpreter::interpret_with_capabilities(
        &graph,
        &[],
        None,
        None,
        None,
        None,
        None,
        0,
        caps,
    );

    match result {
        Err(interpreter::InterpretError::PermissionDenied { effect }) => {
            assert_eq!(effect, EffectTag::TcpConnect);
        }
        other => panic!("expected PermissionDenied for TcpConnect, got {:?}", other),
    }
}

#[test]
fn error_mapping_effect_failed_for_non_permission_errors() {
    // When the RuntimeEffectHandler returns an error that isn't permission-
    // related, it should map to EffectFailed (not PermissionDenied).
    let graph = make_effect_graph(EffectTag::HttpGet.to_u8());
    let caps = Capabilities::unrestricted(); // allows everything

    let result = interpreter::interpret_with_capabilities(
        &graph,
        &[],
        None,
        None,
        None,
        None,
        None,
        0,
        caps,
    );

    match result {
        Err(interpreter::InterpretError::EffectFailed { tag, message }) => {
            assert!(
                tag.contains("0x02") || tag.contains("HttpGet"),
                "tag should reference HttpGet: {}",
                tag
            );
            assert!(
                message.contains("not implemented"),
                "should say not implemented: {}",
                message
            );
        }
        other => panic!("expected EffectFailed for HttpGet, got {:?}", other),
    }
}

// ===========================================================================
// IrisExecutionService enforces capabilities
// ===========================================================================

#[test]
fn exec_service_sandbox_blocks_file_effects() {
    use iris_exec::service::{ExecConfig, IrisExecutionService};
    use iris_exec::ExecutionService;
    use iris_types::eval::{EvalTier, TestCase};

    // Default ExecConfig uses sandbox capabilities.
    let exec = IrisExecutionService::new(ExecConfig::default());

    // A program that tries FileWrite should fail.
    let graph = make_effect_graph(EffectTag::FileWrite.to_u8());
    let tests = vec![TestCase {
        inputs: vec![],
        expected_output: None,
        initial_state: None, expected_state: None,
    }];

    let result = exec.evaluate_individual(&graph, &tests, EvalTier::A);
    // The service catches errors and returns empty outputs (not a hard error).
    match result {
        Ok(eval_result) => {
            // Service wraps errors: outputs should be empty for the failed case.
            assert!(
                eval_result.outputs[0].is_empty(),
                "FileWrite should have failed, producing empty output"
            );
        }
        Err(_) => {} // Also acceptable if it propagates the error.
    }
}

#[test]
fn exec_service_with_custom_caps_allows_timestamp() {
    use iris_exec::service::{ExecConfig, IrisExecutionService, SandboxConfig};
    use iris_exec::ExecutionService;
    use iris_types::eval::{EvalTier, TestCase};

    let exec = IrisExecutionService::with_capabilities(Capabilities::sandbox());

    let graph = make_effect_graph(EffectTag::Timestamp.to_u8());
    let tests = vec![TestCase {
        inputs: vec![],
        expected_output: None,
        initial_state: None, expected_state: None,
    }];

    let result = exec
        .evaluate_individual(&graph, &tests, EvalTier::A)
        .expect("Timestamp should succeed in sandbox");

    assert!(!result.outputs[0].is_empty(), "should have output");
    match &result.outputs[0][0] {
        Value::Int(ms) => assert!(*ms > 1_577_836_800_000),
        other => panic!("expected Int, got {:?}", other),
    }
}

// ===========================================================================
// CapabilityGuardHandler: env var restriction edge case
// ===========================================================================

#[test]
fn guard_blocks_env_get_without_env_permission() {
    let handler = RuntimeEffectHandler::new();
    let mut caps = Capabilities::sandbox();
    caps.allowed_effects.insert(EffectTag::EnvGet); // allow the tag...
    // ...but allowed_env_vars is empty → should still block.
    let guard = CapabilityGuardHandler::new(&handler, caps);

    let result = guard.handle(EffectRequest {
        tag: EffectTag::EnvGet,
        args: vec![Value::String("HOME".into())],
    });

    match result {
        Err(e) => assert!(
            e.message.contains("permission denied") || e.message.contains("not allowed"),
            "should deny: {}",
            e.message
        ),
        Ok(_) => panic!("should have denied EnvGet without env var permission"),
    }
}

#[test]
fn guard_allows_env_get_with_wildcard() {
    let handler = RuntimeEffectHandler::new();
    let mut caps = Capabilities::sandbox();
    caps.allowed_effects.insert(EffectTag::EnvGet);
    caps.allowed_env_vars = vec!["*".to_string()];
    let guard = CapabilityGuardHandler::new(&handler, caps);

    let result = guard.handle(EffectRequest {
        tag: EffectTag::EnvGet,
        args: vec![Value::String("HOME".into())],
    });

    assert!(result.is_ok(), "should allow EnvGet with wildcard");
}
