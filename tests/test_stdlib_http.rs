
//! Tests for the IRIS stdlib HTTP client/server modules.
//!
//! Unit tests verify pure string building/parsing functions via the
//! graph-level primitives (str_concat, str_split, str_len, etc.).
//! Integration tests spin up real TCP listeners and exercise the full
//! client-server round-trip via the RuntimeEffectHandler effect system.

use iris_exec::effect_runtime::RuntimeEffectHandler;
use iris_exec::interpreter;
use iris_types::eval::{EffectHandler, EffectRequest, EffectTag, Value};
use iris_types::graph::SemanticGraph;

use std::collections::{BTreeMap, HashMap};
use std::io::{Read, Write};
use std::net::TcpListener;

use iris_types::cost::{CostBound, CostTerm};
use iris_types::graph::{Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution};
use iris_types::hash::{compute_node_id, SemanticHash};
use iris_types::types::{PrimType, TypeDef, TypeEnv, TypeId};

// ===========================================================================
// Helpers — graph building
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

// ---------------------------------------------------------------------------
// TCP effect helpers
// ---------------------------------------------------------------------------

fn tcp_connect(handler: &RuntimeEffectHandler, host: &str, port: i64) -> i64 {
    let req = EffectRequest {
        tag: EffectTag::TcpConnect,
        args: vec![Value::String(host.to_string()), Value::Int(port)],
    };
    match handler.handle(req).unwrap() {
        Value::Int(h) => h,
        other => panic!("tcp_connect: expected Int, got {:?}", other),
    }
}

fn tcp_write_bytes(handler: &RuntimeEffectHandler, conn: i64, data: &[u8]) -> i64 {
    let req = EffectRequest {
        tag: EffectTag::TcpWrite,
        args: vec![Value::Int(conn), Value::Bytes(data.to_vec())],
    };
    match handler.handle(req).unwrap() {
        Value::Int(n) => n,
        other => panic!("tcp_write: expected Int, got {:?}", other),
    }
}

fn tcp_read_bytes(handler: &RuntimeEffectHandler, conn: i64, max: i64) -> Vec<u8> {
    let req = EffectRequest {
        tag: EffectTag::TcpRead,
        args: vec![Value::Int(conn), Value::Int(max)],
    };
    match handler.handle(req).unwrap() {
        Value::Bytes(b) => b,
        other => panic!("tcp_read: expected Bytes, got {:?}", other),
    }
}

fn tcp_close(handler: &RuntimeEffectHandler, conn: i64) {
    let req = EffectRequest {
        tag: EffectTag::TcpClose,
        args: vec![Value::Int(conn)],
    };
    handler.handle(req).unwrap();
}

/// Bind a TcpListener to port 0 (OS-assigned) and return (listener, port).
fn bind_random_port() -> (TcpListener, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    (listener, port)
}

// ===========================================================================
// Unit tests: build_request
// ===========================================================================

#[test]
fn test_build_request_get() {
    // Build HTTP request string using str_concat primitive chain.
    // Replicate: "GET /api/data HTTP/1.1\r\nHost: example.com\r\n..."
    // Pass CRLF as input since \r is not supported in IRIS string literals.
    let src = r#"
let f method path host body crlf =
    let request_line = str_concat method (str_concat " " (str_concat path (str_concat " HTTP/1.1" crlf))) in
    let host_header = str_concat "Host: " (str_concat host crlf) in
    let content_length = str_concat "Content-Length: " (str_concat (int_to_string (str_len body)) crlf) in
    let connection = str_concat "Connection: close" crlf in
    let headers = str_concat host_header (str_concat content_length connection) in
    str_concat request_line (str_concat headers (str_concat crlf body))
"#;
    let result = run(
        src,
        &[
            Value::String("GET".into()),
            Value::String("/api/data".into()),
            Value::String("example.com".into()),
            Value::String("".into()),
            Value::String("\r\n".into()),
        ],
    );
    match result {
        Value::String(s) => {
            assert!(
                s.starts_with("GET /api/data HTTP/1.1\r\n"),
                "bad request line: {}",
                s.replace('\r', "\\r").replace('\n', "\\n")
            );
            assert!(s.contains("Host: example.com\r\n"), "missing Host header");
            assert!(s.contains("Content-Length: 0\r\n"), "missing Content-Length");
            assert!(s.contains("Connection: close\r\n"), "missing Connection header");
            assert!(s.contains("\r\n\r\n"), "missing header-body separator");
        }
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn test_build_request_post() {
    let src = r#"
let f method path host body crlf =
    let request_line = str_concat method (str_concat " " (str_concat path (str_concat " HTTP/1.1" crlf))) in
    let host_header = str_concat "Host: " (str_concat host crlf) in
    let content_length = str_concat "Content-Length: " (str_concat (int_to_string (str_len body)) crlf) in
    let connection = str_concat "Connection: close" crlf in
    let headers = str_concat host_header (str_concat content_length connection) in
    str_concat request_line (str_concat headers (str_concat crlf body))
"#;
    let body = r#"{"key":"value"}"#;
    let result = run(
        src,
        &[
            Value::String("POST".into()),
            Value::String("/submit".into()),
            Value::String("localhost".into()),
            Value::String(body.into()),
            Value::String("\r\n".into()),
        ],
    );
    match result {
        Value::String(s) => {
            assert!(s.starts_with("POST /submit HTTP/1.1\r\n"), "bad request line");
            assert!(s.contains("Host: localhost\r\n"), "missing Host");
            assert!(
                s.contains(&format!("Content-Length: {}\r\n", body.len())),
                "wrong Content-Length"
            );
            assert!(s.ends_with(body), "body not at end");
        }
        other => panic!("expected String, got {:?}", other),
    }
}

// ===========================================================================
// Unit tests: parse_response — via str_split, str_to_int, list_nth prims
// ===========================================================================

#[test]
fn test_parse_response_200() {
    // Parse HTTP response: extract status code.
    // Pass the raw response and the separator as inputs.
    let src = r#"
let f raw sep =
    let parts = str_split raw sep in
    let header_block = list_nth parts 0 in
    let header_lines = str_split header_block "\n" in
    let status_line = list_nth header_lines 0 in
    let status_parts = str_split status_line " " in
    if list_len status_parts > 1
        then str_to_int (list_nth status_parts 1)
        else 0
"#;
    let raw = "HTTP/1.1 200 OK\r\n\r\nHello";
    let result = run(
        src,
        &[Value::String(raw.into()), Value::String("\r\n\r\n".into())],
    );
    assert_eq!(result, Value::Int(200));
}

#[test]
fn test_parse_response_404() {
    let src = r#"
let f raw sep =
    let parts = str_split raw sep in
    let header_block = list_nth parts 0 in
    let header_lines = str_split header_block "\n" in
    let status_line = list_nth header_lines 0 in
    let status_parts = str_split status_line " " in
    if list_len status_parts > 1
        then str_to_int (list_nth status_parts 1)
        else 0
"#;
    let raw = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\n404 Not Found";
    let result = run(
        src,
        &[Value::String(raw.into()), Value::String("\r\n\r\n".into())],
    );
    assert_eq!(result, Value::Int(404));
}

#[test]
fn test_parse_response_with_headers() {
    // Extract the header block (everything before separator).
    let src = r#"
let f raw sep =
    let parts = str_split raw sep in
    list_nth parts 0
"#;
    let raw = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nX-Custom: foo\r\n\r\n{\"ok\":true}";
    let result = run(
        src,
        &[Value::String(raw.into()), Value::String("\r\n\r\n".into())],
    );
    match result {
        Value::String(s) => {
            assert!(s.contains("Content-Type: application/json"), "missing Content-Type");
            assert!(s.contains("X-Custom: foo"), "missing X-Custom header");
        }
        other => panic!("expected String, got {:?}", other),
    }
}

// ===========================================================================
// Unit tests: build_response
// ===========================================================================

#[test]
fn test_build_response_ok() {
    // Build HTTP response via str_concat. Pass CRLF as input.
    let src = r#"
let f status_code status_text content_type body crlf =
    let status_line = str_concat "HTTP/1.1 " (str_concat (int_to_string status_code) (str_concat " " (str_concat status_text crlf))) in
    let ct_header = str_concat "Content-Type: " (str_concat content_type crlf) in
    let cl_header = str_concat "Content-Length: " (str_concat (int_to_string (str_len body)) crlf) in
    let conn_header = str_concat "Connection: close" crlf in
    str_concat status_line (str_concat ct_header (str_concat cl_header (str_concat conn_header (str_concat crlf body))))
"#;
    let result = run(
        src,
        &[
            Value::Int(200),
            Value::String("OK".into()),
            Value::String("text/plain".into()),
            Value::String("Hello World".into()),
            Value::String("\r\n".into()),
        ],
    );
    match result {
        Value::String(s) => {
            assert!(s.starts_with("HTTP/1.1 200 OK\r\n"), "bad status line");
            assert!(s.contains("Content-Type: text/plain\r\n"), "missing CT");
            assert!(s.contains("Content-Length: 11\r\n"), "bad CL");
            assert!(s.contains("Connection: close\r\n"), "missing Connection");
            assert!(s.ends_with("\r\n\r\nHello World"), "bad body");
        }
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn test_build_response_json() {
    let src = r#"
let f body crlf =
    let status_line = str_concat "HTTP/1.1 200 OK" crlf in
    let ct_header = str_concat "Content-Type: application/json" crlf in
    let cl_header = str_concat "Content-Length: " (str_concat (int_to_string (str_len body)) crlf) in
    let conn_header = str_concat "Connection: close" crlf in
    str_concat status_line (str_concat ct_header (str_concat cl_header (str_concat conn_header (str_concat crlf body))))
"#;
    let body = r#"{"status":"ok"}"#;
    let result = run(
        src,
        &[Value::String(body.into()), Value::String("\r\n".into())],
    );
    match result {
        Value::String(s) => {
            assert!(s.contains("Content-Type: application/json\r\n"), "missing CT");
            assert!(
                s.contains(&format!("Content-Length: {}\r\n", body.len())),
                "wrong CL"
            );
            assert!(s.ends_with(body), "body mismatch");
        }
        other => panic!("expected String, got {:?}", other),
    }
}

// ===========================================================================
// Integration: HTTP client-server round-trip via RuntimeEffectHandler TCP
// ===========================================================================

#[test]
fn test_http_client_server_roundtrip() {
    let (listener, port) = bind_random_port();

    let server_thread = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).unwrap();
        let request = String::from_utf8_lossy(&buf[..n]).to_string();

        assert!(
            request.starts_with("GET / HTTP/1.1"),
            "unexpected request: {}",
            request
        );

        let response = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nHello";
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    });

    let handler = RuntimeEffectHandler::new();
    let conn = tcp_connect(&handler, "127.0.0.1", port as i64);

    let request = "GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    tcp_write_bytes(&handler, conn, request.as_bytes());

    let response_bytes = tcp_read_bytes(&handler, conn, 4096);
    let response = String::from_utf8(response_bytes).unwrap();

    assert!(response.contains("200 OK"), "missing 200: {}", response);
    assert!(response.ends_with("Hello"), "missing body: {}", response);

    tcp_close(&handler, conn);
    server_thread.join().unwrap();
}

#[test]
fn test_http_post_echo() {
    let (listener, port) = bind_random_port();

    let server_thread = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).unwrap();
        let request = String::from_utf8_lossy(&buf[..n]).to_string();

        let body = request.split("\r\n\r\n").nth(1).unwrap_or("").to_string();

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    });

    let handler = RuntimeEffectHandler::new();
    let conn = tcp_connect(&handler, "127.0.0.1", port as i64);

    let post_body = r#"{"msg":"ping"}"#;
    let request = format!(
        "POST /echo HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        post_body.len(),
        post_body
    );
    tcp_write_bytes(&handler, conn, request.as_bytes());

    let response_bytes = tcp_read_bytes(&handler, conn, 4096);
    let response = String::from_utf8(response_bytes).unwrap();

    assert!(response.contains("200 OK"), "missing 200: {}", response);
    assert!(
        response.ends_with(post_body),
        "body not echoed: {}",
        response
    );

    tcp_close(&handler, conn);
    server_thread.join().unwrap();
}

#[test]
fn test_http_multiple_requests() {
    let (listener, port) = bind_random_port();

    let server_thread = std::thread::spawn(move || {
        for i in 0..3 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let _n = stream.read(&mut buf).unwrap();

            let body = format!("response-{}", i);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    });

    let handler = RuntimeEffectHandler::new();

    for i in 0..3 {
        let conn = tcp_connect(&handler, "127.0.0.1", port as i64);
        let request = format!(
            "GET /page/{} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
            i
        );
        tcp_write_bytes(&handler, conn, request.as_bytes());

        let response_bytes = tcp_read_bytes(&handler, conn, 4096);
        let response = String::from_utf8(response_bytes).unwrap();

        let expected_body = format!("response-{}", i);
        assert!(
            response.ends_with(&expected_body),
            "request {}: expected body '{}', got: {}",
            i,
            expected_body,
            response
        );

        tcp_close(&handler, conn);
    }

    server_thread.join().unwrap();
}
