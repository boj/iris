//! IRIS Language Server Protocol (LSP) implementation.
//!
//! Synchronous JSON-RPC over stdio. Zero async dependencies.
//! Provides: diagnostics (compile errors), hover (types), completion (keywords + primitives).

use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

use iris_bootstrap::syntax;
use iris_types::fragment::Fragment;
use iris_types::graph::NodeId;
use iris_types::types::{TypeDef, TypeId, PrimType};

type SourceMap = BTreeMap<NodeId, iris_bootstrap::syntax::error::Span>;

struct DocState {
    text: String,
    fragments: Vec<(String, Fragment, SourceMap)>,
    errors: Vec<iris_bootstrap::syntax::error::SyntaxError>,
}

struct Server {
    documents: BTreeMap<String, DocState>,
    initialized: bool,
}

impl Server {
    fn new() -> Self {
        Server { documents: BTreeMap::new(), initialized: false }
    }

    fn compile_doc(&mut self, uri: &str, text: String) {
        let result = syntax::compile(&text);
        let errors = result.errors.clone();
        let fragments = result.fragments;
        self.documents.insert(uri.to_string(), DocState {
            text,
            fragments: fragments.into_iter().map(|(n, f, s)| (n, f, s)).collect(),
            errors,
        });
    }

    fn handle_message(&mut self, msg: &serde_json::Value) -> Option<serde_json::Value> {
        let method = msg.get("method")?.as_str()?;
        let id = msg.get("id");
        let params = msg.get("params").cloned().unwrap_or(serde_json::Value::Null);

        match method {
            "initialize" => {
                self.initialized = true;
                let result = serde_json::json!({
                    "capabilities": {
                        "textDocumentSync": 1,
                        "hoverProvider": true,
                        "completionProvider": {
                            "triggerCharacters": ["."]
                        }
                    },
                    "serverInfo": {
                        "name": "iris-lsp",
                        "version": "0.1.0"
                    }
                });
                id.map(|id| response(id.clone(), result))
            }
            "initialized" => None,
            "shutdown" => {
                id.map(|id| response(id.clone(), serde_json::Value::Null))
            }
            "exit" => std::process::exit(0),
            "textDocument/didOpen" => {
                if let Some(td) = params.get("textDocument") {
                    let uri = td.get("uri").and_then(|v| v.as_str()).unwrap_or("");
                    let text = td.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    self.compile_doc(uri, text.to_string());
                    self.publish_diagnostics(uri);
                }
                None
            }
            "textDocument/didChange" => {
                if let Some(td) = params.get("textDocument") {
                    let uri = td.get("uri").and_then(|v| v.as_str()).unwrap_or("");
                    // Full sync: take the last content change
                    if let Some(changes) = params.get("contentChanges").and_then(|v| v.as_array()) {
                        if let Some(last) = changes.last() {
                            let text = last.get("text").and_then(|v| v.as_str()).unwrap_or("");
                            self.compile_doc(uri, text.to_string());
                            self.publish_diagnostics(uri);
                        }
                    }
                }
                None
            }
            "textDocument/didClose" => {
                if let Some(td) = params.get("textDocument") {
                    let uri = td.get("uri").and_then(|v| v.as_str()).unwrap_or("");
                    self.documents.remove(uri);
                }
                None
            }
            "textDocument/hover" => {
                let result = self.handle_hover(&params);
                id.map(|id| response(id.clone(), result))
            }
            "textDocument/completion" => {
                let result = self.handle_completion();
                id.map(|id| response(id.clone(), result))
            }
            _ => {
                // Unknown method — return null for requests, ignore notifications
                id.map(|id| response(id.clone(), serde_json::Value::Null))
            }
        }
    }

    fn handle_hover(&self, params: &serde_json::Value) -> serde_json::Value {
        let uri = params.pointer("/textDocument/uri").and_then(|v| v.as_str()).unwrap_or("");
        let line = params.pointer("/position/line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let character = params.pointer("/position/character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return serde_json::Value::Null,
        };

        let offset = lsp_position_to_offset(&doc.text, line, character);

        // Find the node at this offset across all fragments
        for (name, fragment, source_map) in &doc.fragments {
            if let Some(node_id) = node_at_offset(offset, source_map) {
                if let Some(node) = fragment.graph.nodes.get(&node_id) {
                    let type_str = format_type_id(&node.type_sig, &fragment.graph.type_env);
                    let kind_str = format!("{:?}", node.kind);
                    let hover_text = format!("**{}** `{}`\n\nKind: {}", name, type_str, kind_str);
                    return serde_json::json!({
                        "contents": {
                            "kind": "markdown",
                            "value": hover_text
                        }
                    });
                }
            }
        }

        serde_json::Value::Null
    }

    fn handle_completion(&self) -> serde_json::Value {
        let mut items: Vec<serde_json::Value> = Vec::new();

        // Keywords
        for kw in &["let", "rec", "in", "type", "import", "as", "match", "with",
                     "if", "then", "else", "when", "where", "forall", "true", "false",
                     "requires", "ensures", "allow", "deny", "fold", "fold_until",
                     "unfold", "class", "instance", "and"] {
            items.push(serde_json::json!({
                "label": kw,
                "kind": 14, // Keyword
            }));
        }

        // Primitives
        for name in iris_bootstrap::syntax::prim::primitive_names() {
            items.push(serde_json::json!({
                "label": name,
                "kind": 3, // Function
            }));
        }

        // Effects
        for eff in &["print", "read_line", "file_open", "file_read_bytes", "file_write_bytes",
                      "file_close", "file_stat", "dir_list", "tcp_connect", "tcp_read",
                      "tcp_write", "tcp_close", "tcp_listen", "tcp_accept", "env_get",
                      "clock_ns", "sleep_ms", "random_bytes", "thread_spawn", "thread_join",
                      "atomic_read", "atomic_write", "atomic_swap", "ffi_call"] {
            items.push(serde_json::json!({
                "label": eff,
                "kind": 3, // Function
                "detail": "effect",
            }));
        }

        serde_json::json!(items)
    }

    fn publish_diagnostics(&self, uri: &str) {
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return,
        };

        let mut diagnostics: Vec<serde_json::Value> = Vec::new();

        for err in &doc.errors {
            let (start_line, start_char) = offset_to_lsp_position(&doc.text, err.span.start);
            let (end_line, end_char) = offset_to_lsp_position(&doc.text, err.span.end);
            diagnostics.push(serde_json::json!({
                "range": {
                    "start": { "line": start_line, "character": start_char },
                    "end": { "line": end_line, "character": end_char }
                },
                "severity": 1, // Error
                "source": "iris",
                "message": err.message
            }));
        }

        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": {
                "uri": uri,
                "diagnostics": diagnostics
            }
        });

        send_message(&notification);
    }
}

// --- JSON-RPC transport ---

fn read_message(stdin: &mut impl BufRead) -> io::Result<Option<serde_json::Value>> {
    let mut header = String::new();
    let mut content_length: usize = 0;

    loop {
        header.clear();
        let n = stdin.read_line(&mut header)?;
        if n == 0 { return Ok(None); } // EOF
        let trimmed = header.trim();
        if trimmed.is_empty() { break; } // End of headers
        if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") {
            content_length = len_str.parse().unwrap_or(0);
        }
    }

    if content_length == 0 { return Ok(None); }

    let mut body = vec![0u8; content_length];
    stdin.read_exact(&mut body)?;

    match serde_json::from_slice(&body) {
        Ok(v) => Ok(Some(v)),
        Err(e) => {
            eprintln!("[iris-lsp] JSON parse error: {}", e);
            Ok(None)
        }
    }
}

fn send_message(msg: &serde_json::Value) {
    let body = serde_json::to_string(msg).unwrap();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(header.as_bytes());
    let _ = out.write_all(body.as_bytes());
    let _ = out.flush();
}

fn response(id: serde_json::Value, result: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

// --- Position conversion ---

fn offset_to_lsp_position(source: &str, offset: usize) -> (u32, u32) {
    let offset = offset.min(source.len());
    let mut line = 0u32;
    let mut last_newline = 0usize;
    for (i, ch) in source.char_indices() {
        if i >= offset { break; }
        if ch == '\n' { line += 1; last_newline = i + 1; }
    }
    (line, (offset.saturating_sub(last_newline)) as u32)
}

fn lsp_position_to_offset(source: &str, line: u32, character: u32) -> usize {
    let mut current_line = 0u32;
    for (i, ch) in source.char_indices() {
        if current_line == line {
            return (i + character as usize).min(source.len());
        }
        if ch == '\n' { current_line += 1; }
    }
    source.len()
}

fn node_at_offset(offset: usize, source_map: &SourceMap) -> Option<NodeId> {
    source_map.iter()
        .filter(|(_, span)| span.start <= offset && offset < span.end)
        .min_by_key(|(_, span)| span.end - span.start)
        .map(|(id, _)| *id)
}

// --- Type display ---

fn format_type_id(type_id: &TypeId, type_env: &iris_types::types::TypeEnv) -> String {
    match type_env.types.get(type_id) {
        Some(td) => format_type_def(td, type_env),
        None => {
            if *type_id == iris_types::hash::compute_type_id(&TypeDef::Primitive(PrimType::Int)) {
                "Int".to_string()
            } else {
                format!("Type({})", type_id.0)
            }
        }
    }
}

fn format_type_def(td: &TypeDef, env: &iris_types::types::TypeEnv) -> String {
    match td {
        TypeDef::Primitive(p) => match p {
            PrimType::Int => "Int".into(),
            PrimType::Nat => "Nat".into(),
            PrimType::Bool => "Bool".into(),
            PrimType::Float64 => "Float64".into(),
            PrimType::Float32 => "Float32".into(),
            PrimType::Bytes => "Bytes".into(),
            PrimType::Unit => "()".into(),
        },
        TypeDef::Product(elems) => {
            let parts: Vec<String> = elems.iter().map(|t| format_type_id(t, env)).collect();
            format!("({})", parts.join(", "))
        }
        TypeDef::Sum(variants) => {
            let parts: Vec<String> = variants.iter()
                .map(|(tag, tid)| format!("#{}: {}", tag.0, format_type_id(tid, env)))
                .collect();
            parts.join(" | ")
        }
        TypeDef::Arrow(param, ret, _) => {
            format!("{} -> {}", format_type_id(param, env), format_type_id(ret, env))
        }
        TypeDef::ForAll(bv, body) => {
            format!("forall {}. {}", bv.0, format_type_id(body, env))
        }
        TypeDef::Refined(base, _) => {
            format!("{{ {}: ... }}", format_type_id(base, env))
        }
        _ => format!("{:?}", td),
    }
}

// --- Main ---

fn main() {
    eprintln!("[iris-lsp] starting (synchronous stdio JSON-RPC)");
    let mut server = Server::new();
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    loop {
        match read_message(&mut reader) {
            Ok(Some(msg)) => {
                if let Some(response) = server.handle_message(&msg) {
                    send_message(&response);
                }
            }
            Ok(None) => break, // EOF
            Err(e) => {
                eprintln!("[iris-lsp] read error: {}", e);
                break;
            }
        }
    }

    eprintln!("[iris-lsp] shutting down");
}
