
//! Integration tests for the capability-based security system.
//!
//! Validates that:
//! - Sandboxed programs cannot perform file I/O, network, FFI, or JIT
//! - Sandboxed programs can still do pure computation
//! - Path restrictions enforce glob patterns on file operations
//! - Host restrictions enforce matching on network operations
//! - Daemon candidate sandboxing is correctly configured
//! - PermissionDenied errors are properly returned
//! - Capability annotations in .iris source are parsed correctly

use std::collections::{BTreeMap, HashMap};

use iris_exec::capabilities::{Capabilities, CapabilityGuardHandler};
use iris_exec::effect_runtime::{NoOpHandler, RuntimeEffectHandler};
use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::{EffectHandler, EffectRequest, EffectTag, Value};
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

/// Build a simple add(3, 4) graph for pure computation.
fn make_add_graph(a: i64, b: i64) -> SemanticGraph {
    let (type_env, int_id) = int_type_env();
    let mut nodes = HashMap::new();

    let lit_a = make_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: a.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let lit_b = make_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: b.to_le_bytes().to_vec(),
        },
        int_id,
        0,
    );
    let prim = make_node(NodeKind::Prim, NodePayload::Prim { opcode: 0x00 }, int_id, 2);

    let prim_id = prim.id;
    let a_id = lit_a.id;
    let b_id = lit_b.id;

    nodes.insert(a_id, lit_a);
    nodes.insert(b_id, lit_b);
    nodes.insert(prim_id, prim);

    let edges = vec![
        Edge {
            source: prim_id,
            target: a_id,
            port: 0,
            label: EdgeLabel::Argument,
        },
        Edge {
            source: prim_id,
            target: b_id,
            port: 1,
            label: EdgeLabel::Argument,
        },
    ];

    let hash = compute_hash(&nodes, &edges);
    SemanticGraph {
        root: prim_id,
        nodes,
        edges,
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash,
    }
}

// ---------------------------------------------------------------------------
// Test: sandbox blocks file write
// ---------------------------------------------------------------------------

#[test]
fn test_sandbox_blocks_file_write() {
    let graph = make_effect_graph(EffectTag::FileWrite.to_u8());
    let caps = Capabilities::sandbox();

    let result = interpreter::interpret_with_capabilities(
        &graph,
        &[],
        None,
        None,
        Some(&NoOpHandler),
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

// ---------------------------------------------------------------------------
// Test: sandbox blocks TCP connect
// ---------------------------------------------------------------------------

#[test]
fn test_sandbox_blocks_tcp() {
    let graph = make_effect_graph(EffectTag::TcpConnect.to_u8());
    let caps = Capabilities::sandbox();

    let result = interpreter::interpret_with_capabilities(
        &graph,
        &[],
        None,
        None,
        Some(&NoOpHandler),
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

// ---------------------------------------------------------------------------
// Test: sandbox blocks EnvGet (not in allowed effects)
// ---------------------------------------------------------------------------

#[test]
fn test_sandbox_blocks_env_get() {
    let graph = make_effect_graph(EffectTag::EnvGet.to_u8());
    let caps = Capabilities::sandbox();

    // EnvGet is not in the sandbox allowed list.
    assert!(!caps.is_allowed(EffectTag::EnvGet));

    let result = interpreter::interpret_with_capabilities(
        &graph,
        &[],
        None,
        None,
        Some(&NoOpHandler),
        None,
        None,
        0,
        caps,
    );

    match result {
        Err(interpreter::InterpretError::PermissionDenied { effect }) => {
            assert_eq!(effect, EffectTag::EnvGet);
        }
        other => panic!("expected PermissionDenied for EnvGet, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test: sandbox blocks FFI (FfiCall effect via can_ffi check)
// ---------------------------------------------------------------------------

#[test]
fn test_sandbox_blocks_ffi_call() {
    // FfiCall (0x2B) is blocked by both: (a) can_ffi: false structural check,
    // and (b) FfiCall not being in allowed_effects.
    let graph = make_effect_graph(EffectTag::FfiCall.to_u8());
    let caps = Capabilities::sandbox();

    assert!(!caps.can_ffi);
    assert!(!caps.is_allowed(EffectTag::FfiCall));

    let result = interpreter::interpret_with_capabilities(
        &graph,
        &[],
        None,
        None,
        Some(&NoOpHandler),
        None,
        None,
        0,
        caps,
    );

    match result {
        Err(interpreter::InterpretError::PermissionDenied { effect }) => {
            assert_eq!(effect, EffectTag::FfiCall);
        }
        other => panic!("expected PermissionDenied for FfiCall, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test: unrestricted caps include FfiCall
// ---------------------------------------------------------------------------

#[test]
fn test_unrestricted_includes_ffi() {
    let caps = Capabilities::unrestricted();
    assert!(caps.can_ffi);
    assert!(caps.is_allowed(EffectTag::FfiCall));
    assert!(caps.is_allowed(EffectTag::CallNative));
}

// ---------------------------------------------------------------------------
// Test: sandbox blocks mmap_exec (JIT)
// ---------------------------------------------------------------------------

#[test]
fn test_sandbox_blocks_mmap_exec() {
    let graph = make_effect_graph(EffectTag::MmapExec.to_u8());
    let caps = Capabilities::sandbox();

    assert!(!caps.can_mmap_exec);
    assert!(!caps.is_allowed(EffectTag::MmapExec));

    let result = interpreter::interpret_with_capabilities(
        &graph,
        &[],
        None,
        None,
        Some(&NoOpHandler),
        None,
        None,
        0,
        caps,
    );

    match result {
        Err(interpreter::InterpretError::PermissionDenied { effect }) => {
            assert_eq!(effect, EffectTag::MmapExec);
        }
        other => panic!("expected PermissionDenied for MmapExec, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test: sandbox allows pure computation
// ---------------------------------------------------------------------------

#[test]
fn test_sandbox_allows_pure_computation() {
    let graph = make_add_graph(17, 25);
    let caps = Capabilities::sandbox();

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

    let (outputs, _state) = result.expect("pure computation should succeed in sandbox");
    assert_eq!(outputs, vec![Value::Int(42)]);
}

// ---------------------------------------------------------------------------
// Test: file path restriction via CapabilityGuardHandler
// ---------------------------------------------------------------------------

#[test]
fn test_capability_file_path_restriction() {
    let caps = Capabilities::io_restricted(&["/tmp/*"], &[]);

    // A handler that would succeed for any file op — the guard should block it.
    let guard = CapabilityGuardHandler::new(NoOpHandler, caps);

    // Allowed path: /tmp/data.txt
    let allowed_req = EffectRequest {
        tag: EffectTag::FileRead,
        args: vec![Value::String("/tmp/data.txt".to_string())],
    };
    let result = guard.handle(allowed_req);
    assert!(result.is_ok(), "should allow /tmp/data.txt");

    // Denied path: /etc/passwd
    let denied_req = EffectRequest {
        tag: EffectTag::FileRead,
        args: vec![Value::String("/etc/passwd".to_string())],
    };
    let err = guard.handle(denied_req).unwrap_err();
    assert_eq!(err.tag, EffectTag::FileRead);
    assert!(err.message.contains("permission denied"));
    assert!(err.message.contains("/etc/passwd"));
}

// ---------------------------------------------------------------------------
// Test: network host restriction via CapabilityGuardHandler
// ---------------------------------------------------------------------------

#[test]
fn test_capability_network_host_restriction() {
    let caps = Capabilities::io_restricted(&[], &["api.example.com", "*.internal.net"]);

    let guard = CapabilityGuardHandler::new(NoOpHandler, caps);

    // Allowed host
    let allowed_req = EffectRequest {
        tag: EffectTag::TcpConnect,
        args: vec![
            Value::String("api.example.com".to_string()),
            Value::Int(443),
        ],
    };
    let result = guard.handle(allowed_req);
    assert!(result.is_ok(), "should allow api.example.com");

    // Allowed wildcard host
    let wildcard_req = EffectRequest {
        tag: EffectTag::TcpConnect,
        args: vec![
            Value::String("foo.internal.net".to_string()),
            Value::Int(8080),
        ],
    };
    let result = guard.handle(wildcard_req);
    assert!(result.is_ok(), "should allow foo.internal.net via *.internal.net");

    // Denied host
    let denied_req = EffectRequest {
        tag: EffectTag::TcpConnect,
        args: vec![
            Value::String("evil.com".to_string()),
            Value::Int(80),
        ],
    };
    let err = guard.handle(denied_req).unwrap_err();
    assert_eq!(err.tag, EffectTag::TcpConnect);
    assert!(err.message.contains("permission denied"));
    assert!(err.message.contains("evil.com"));
}

// ---------------------------------------------------------------------------
// Test: daemon runs candidates sandboxed
// ---------------------------------------------------------------------------

#[test]
fn test_daemon_runs_candidates_sandboxed() {
    // Simulate what the daemon does: run an evolved candidate with
    // daemon_candidate() capabilities.
    let caps = Capabilities::daemon_candidate();

    // Verify the daemon sandbox properties.
    assert!(!caps.can_spawn_threads, "daemon sandbox must not allow threads");
    assert!(!caps.can_ffi, "daemon sandbox must not allow FFI");
    assert!(!caps.can_mmap_exec, "daemon sandbox must not allow JIT");
    assert_eq!(caps.max_memory, 10 * 1024 * 1024, "daemon sandbox must limit to 10MB");
    assert_eq!(caps.max_steps, 1_000_000, "daemon sandbox must limit to 1M steps");

    // Network must be blocked.
    assert!(!caps.is_allowed(EffectTag::TcpConnect), "daemon sandbox must block TCP");
    assert!(!caps.is_allowed(EffectTag::HttpGet), "daemon sandbox must block HTTP");

    // File ops are allowed but only to /tmp.
    assert!(caps.is_allowed(EffectTag::FileRead));
    assert!(caps.is_path_allowed("/tmp/candidate_output.txt"));
    assert!(!caps.is_path_allowed("/etc/shadow"));
    assert!(!caps.is_path_allowed("/home/user/.ssh/id_rsa"));

    // Pure computation should succeed within the sandbox.
    let graph = make_add_graph(100, 200);
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
    let (outputs, _) = result.expect("pure math should work in daemon sandbox");
    assert_eq!(outputs, vec![Value::Int(300)]);
}

// ---------------------------------------------------------------------------
// Test: PermissionDenied error type
// ---------------------------------------------------------------------------

#[test]
fn test_permission_denied_error() {
    let graph = make_effect_graph(EffectTag::DbQuery.to_u8());
    let caps = Capabilities::sandbox();

    let result = interpreter::interpret_with_capabilities(
        &graph,
        &[],
        None,
        None,
        Some(&NoOpHandler),
        None,
        None,
        0,
        caps,
    );

    match result {
        Err(ref err @ interpreter::InterpretError::PermissionDenied { effect }) => {
            assert_eq!(effect, EffectTag::DbQuery);
            // Verify the error message is informative.
            let msg = format!("{}", err);
            assert!(msg.contains("permission denied"));
            assert!(msg.contains("DbQuery"));
        }
        other => panic!("expected PermissionDenied, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test: thread spawn blocked by can_spawn_threads
// ---------------------------------------------------------------------------

#[test]
fn test_sandbox_blocks_thread_spawn() {
    let graph = make_effect_graph(EffectTag::ThreadSpawn.to_u8());
    let caps = Capabilities::sandbox();

    assert!(!caps.can_spawn_threads);

    let result = interpreter::interpret_with_capabilities(
        &graph,
        &[],
        None,
        None,
        Some(&NoOpHandler),
        None,
        None,
        0,
        caps,
    );

    match result {
        Err(interpreter::InterpretError::PermissionDenied { effect }) => {
            assert_eq!(effect, EffectTag::ThreadSpawn);
        }
        other => panic!("expected PermissionDenied for ThreadSpawn, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test: unrestricted capabilities allow everything
// ---------------------------------------------------------------------------

#[test]
fn test_unrestricted_allows_all_effects() {
    // An effect graph that does Timestamp (a benign effect).
    let graph = make_effect_graph(EffectTag::Timestamp.to_u8());
    let caps = Capabilities::unrestricted();

    let result = interpreter::interpret_with_capabilities(
        &graph,
        &[],
        None,
        None,
        Some(&RuntimeEffectHandler::new()),
        None,
        None,
        0,
        caps,
    );

    let (outputs, _) = result.expect("unrestricted should allow Timestamp");
    match &outputs[0] {
        Value::Int(ms) => assert!(*ms > 0, "Timestamp should return a positive value"),
        other => panic!("expected Int, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Test: CapabilityGuardHandler blocks disallowed effect tags
// ---------------------------------------------------------------------------

#[test]
fn test_capability_guard_blocks_tag() {
    let caps = Capabilities::sandbox(); // No file ops allowed.
    let guard = CapabilityGuardHandler::new(NoOpHandler, caps);

    let req = EffectRequest {
        tag: EffectTag::FileWrite,
        args: vec![
            Value::String("/tmp/test.txt".to_string()),
            Value::String("data".to_string()),
        ],
    };

    let err = guard.handle(req).unwrap_err();
    assert_eq!(err.tag, EffectTag::FileWrite);
    assert!(err.message.contains("permission denied"));
}

// ---------------------------------------------------------------------------
// Test: CapabilityGuardHandler allows permitted effect tags
// ---------------------------------------------------------------------------

#[test]
fn test_capability_guard_allows_permitted() {
    let caps = Capabilities::sandbox(); // Print is allowed.
    let guard = CapabilityGuardHandler::new(NoOpHandler, caps);

    let req = EffectRequest {
        tag: EffectTag::Print,
        args: vec![Value::String("hello".to_string())],
    };

    let result = guard.handle(req);
    assert!(result.is_ok(), "Print should be allowed in sandbox");
}

// ---------------------------------------------------------------------------
// Test: .iris capability annotation parsing
// ---------------------------------------------------------------------------

#[test]
fn test_iris_capability_parsing() {
    let source = r#"
allow [FileRead, FileWrite "/tmp/*"]
deny [TcpConnect, ThreadSpawn, MmapExec]

let main x = x + 1
"#;

    let module = iris_bootstrap::syntax::parse(source).expect("should parse capability annotations");

    let caps = module.capabilities.expect("should have capabilities");
    assert_eq!(caps.allow.len(), 2);
    assert_eq!(caps.allow[0].effect_name, "FileRead");
    assert_eq!(caps.allow[0].argument, None);
    assert_eq!(caps.allow[1].effect_name, "FileWrite");
    assert_eq!(caps.allow[1].argument, Some("/tmp/*".to_string()));

    assert_eq!(caps.deny.len(), 3);
    assert_eq!(caps.deny[0].effect_name, "TcpConnect");
    assert_eq!(caps.deny[1].effect_name, "ThreadSpawn");
    assert_eq!(caps.deny[2].effect_name, "MmapExec");

    // The module should still have the let declaration.
    assert_eq!(module.items.len(), 1);
}

// ---------------------------------------------------------------------------
// Test: .iris module without capabilities parses normally
// ---------------------------------------------------------------------------

#[test]
fn test_iris_no_capabilities() {
    let source = r#"
let double x = x * 2
"#;

    let module = iris_bootstrap::syntax::parse(source).expect("should parse without capabilities");
    assert!(module.capabilities.is_none());
    assert_eq!(module.items.len(), 1);
}

// ---------------------------------------------------------------------------
// Test: .iris deny-only capability declaration
// ---------------------------------------------------------------------------

#[test]
fn test_iris_deny_only_capabilities() {
    let source = r#"
deny [MmapExec, ThreadSpawn]

let f x = x
"#;

    let module = iris_bootstrap::syntax::parse(source).expect("should parse deny-only capabilities");
    let caps = module.capabilities.expect("should have capabilities");
    assert!(caps.allow.is_empty());
    assert_eq!(caps.deny.len(), 2);
    assert_eq!(caps.deny[0].effect_name, "MmapExec");
    assert_eq!(caps.deny[1].effect_name, "ThreadSpawn");
}

// ---------------------------------------------------------------------------
// Test: glob pattern edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_glob_edge_cases() {
    let caps = Capabilities::io_restricted(
        &["/tmp/**", "/home/user/safe/*"],
        &[],
    );

    // /tmp/** should match nested paths (parent must exist for canonicalization).
    // Use paths whose parents actually exist on the filesystem.
    assert!(caps.is_path_allowed("/tmp/file.txt"));

    // Create a real nested dir for the ** test.
    let test_dir = "/tmp/iris_glob_test_nested";
    let _ = std::fs::create_dir_all(test_dir);
    assert!(caps.is_path_allowed(&format!("{}/d.txt", test_dir)));
    let _ = std::fs::remove_dir_all(test_dir);

    // Paths outside allowed patterns should be denied.
    assert!(!caps.is_path_allowed("/var/log/syslog"));
}

// ---------------------------------------------------------------------------
// Test: path traversal via .. is blocked
// ---------------------------------------------------------------------------

#[test]
fn test_path_traversal_blocked() {
    let caps = Capabilities::io_restricted(&["/tmp/*"], &[]);

    // Direct path is allowed.
    assert!(caps.is_path_allowed("/tmp/ok.txt"));

    // Path traversal via .. must be blocked even if it resolves inside /tmp.
    assert!(!caps.is_path_allowed("/tmp/../etc/passwd"));
    assert!(!caps.is_path_allowed("/tmp/sub/../../etc/shadow"));
}

// ---------------------------------------------------------------------------
// Test: FileReadBytes/FileWriteBytes use handles, not paths
// ---------------------------------------------------------------------------

#[test]
fn test_file_bytes_uses_handles_not_paths() {
    // FileReadBytes/FileWriteBytes operate on integer file handles, not paths.
    // Path enforcement happens at FileOpen time; byte ops inherit the handle's
    // permissions. This test verifies the security fix: byte ops with string
    // args are NOT path-checked (they'd be bogus anyway since real args are
    // integer handles).
    let caps = Capabilities::io_restricted(&["/tmp/*"], &[]);
    let guard = CapabilityGuardHandler::new(NoOpHandler, caps);

    // FileReadBytes is allowed by io_restricted caps, and handle-based
    // args bypass the path check (correctly).
    let req = EffectRequest {
        tag: EffectTag::FileReadBytes,
        args: vec![Value::Int(42)], // integer handle, not a path
    };
    // NoOpHandler returns Unit — the guard passes it through because
    // FileReadBytes is in the allowed set and isn't path-checked.
    assert!(guard.handle(req).is_ok());

    // FileOpen to a disallowed path SHOULD still be blocked.
    let denied_open = EffectRequest {
        tag: EffectTag::FileOpen,
        args: vec![Value::String("/etc/passwd".to_string())],
    };
    let err = guard.handle(denied_open).unwrap_err();
    assert_eq!(err.tag, EffectTag::FileOpen);
    assert!(err.message.contains("permission denied"));
}

// ---------------------------------------------------------------------------
// Test: TcpListen host restriction
// ---------------------------------------------------------------------------

#[test]
fn test_tcp_listen_host_restriction() {
    let caps = Capabilities::io_restricted(&[], &["localhost"]);
    let guard = CapabilityGuardHandler::new(NoOpHandler, caps);

    // TcpListen on allowed interface.
    let allowed_req = EffectRequest {
        tag: EffectTag::TcpListen,
        args: vec![Value::String("localhost".to_string()), Value::Int(8080)],
    };
    assert!(guard.handle(allowed_req).is_ok());

    // TcpListen on disallowed interface.
    let denied_req = EffectRequest {
        tag: EffectTag::TcpListen,
        args: vec![Value::String("0.0.0.0".to_string()), Value::Int(8080)],
    };
    let err = guard.handle(denied_req).unwrap_err();
    assert_eq!(err.tag, EffectTag::TcpListen);
    assert!(err.message.contains("permission denied"));
}

// ---------------------------------------------------------------------------
// Test: EnvGet restricted by allowed_env_vars
// ---------------------------------------------------------------------------

#[test]
fn test_env_var_restriction() {
    let caps = Capabilities {
        allowed_env_vars: vec!["HOME".to_string(), "PATH".to_string()],
        ..Capabilities::io_restricted(&[], &[])
    };
    // Allow EnvGet in the effect set.
    let mut caps = caps;
    caps.allowed_effects.insert(EffectTag::EnvGet);

    assert!(caps.is_env_var_allowed("HOME"));
    assert!(caps.is_env_var_allowed("PATH"));
    assert!(!caps.is_env_var_allowed("SECRET_KEY"));
    assert!(!caps.is_env_var_allowed("AWS_SECRET_ACCESS_KEY"));

    let guard = CapabilityGuardHandler::new(NoOpHandler, caps);

    // Allowed env var.
    let allowed_req = EffectRequest {
        tag: EffectTag::EnvGet,
        args: vec![Value::String("HOME".to_string())],
    };
    assert!(guard.handle(allowed_req).is_ok());

    // Denied env var.
    let denied_req = EffectRequest {
        tag: EffectTag::EnvGet,
        args: vec![Value::String("SECRET_KEY".to_string())],
    };
    let err = guard.handle(denied_req).unwrap_err();
    assert_eq!(err.tag, EffectTag::EnvGet);
    assert!(err.message.contains("permission denied"));
}

// ---------------------------------------------------------------------------
// Test: CallNative requires can_ffi
// ---------------------------------------------------------------------------

#[test]
fn test_call_native_requires_can_ffi() {
    let mut caps = Capabilities::unrestricted();
    caps.can_ffi = false;
    // CallNative requires can_ffi (not just can_mmap_exec).
    assert!(!caps.is_allowed(EffectTag::CallNative));

    // With can_ffi but no can_mmap_exec, also blocked.
    let mut caps2 = Capabilities::unrestricted();
    caps2.can_mmap_exec = false;
    assert!(!caps2.is_allowed(EffectTag::CallNative));
}

// ---------------------------------------------------------------------------
// Test: sandbox blocks channels
// ---------------------------------------------------------------------------

#[test]
fn test_sandbox_blocks_channels() {
    let caps = Capabilities::sandbox();
    assert!(!caps.can_use_channels, "sandbox must block channel operations");

    // Unrestricted should allow channels.
    let caps_unr = Capabilities::unrestricted();
    assert!(caps_unr.can_use_channels);
}
