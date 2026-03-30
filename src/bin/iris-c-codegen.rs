//! iris-c-codegen: Compile a serialized SemanticGraph (interpreter.json) into
//! a C function that implements the IRIS interpreter.
//!
//! Usage:
//!   iris-c-codegen <input.json>
//!
//! Writes the generated C source to stdout.  The emitted function has the
//! signature:
//!
//!     iris_value_t *iris_interpret(iris_value_t *program, iris_value_t *inputs);
//!
//! It calls into the C runtime defined by iris_rt.h.

use std::collections::HashMap;
use std::env;
use std::fmt::Write as FmtWrite;
use std::process;

use iris_bootstrap::load_graph;
use iris_types::graph::{EdgeLabel, NodeId, NodeKind, NodePayload, SemanticGraph};

// ---------------------------------------------------------------------------
// Opcode → C function name
// ---------------------------------------------------------------------------

fn opcode_to_c_fn(opcode: u8) -> &'static str {
    match opcode {
        // Arithmetic
        0x00 => "iris_add",
        0x01 => "iris_sub",
        0x02 => "iris_mul",
        0x03 => "iris_div",
        0x04 => "iris_mod",
        0x05 => "iris_neg",
        0x06 => "iris_abs",
        0x07 => "iris_min",
        0x08 => "iris_max",
        0x09 => "iris_pow",
        // Bitwise
        0x10 => "iris_bitand",
        0x11 => "iris_bitor",
        0x12 => "iris_bitxor",
        0x13 => "iris_bitnot",
        0x14 => "iris_shl",
        0x15 => "iris_shr",
        // Comparison
        0x20 => "iris_eq",
        0x21 => "iris_ne",
        0x22 => "iris_lt",
        0x23 => "iris_gt",
        0x24 => "iris_le",
        0x25 => "iris_ge",
        // Higher-order
        0x30 => "iris_map",
        0x31 => "iris_filter",
        0x32 => "iris_zip",
        // Conversion
        0x40 => "iris_int_to_float",
        0x41 => "iris_float_to_int",
        0x42 => "iris_float_to_bits",
        0x43 => "iris_bits_to_float",
        0x44 => "iris_bool_to_int",
        // State
        0x50 => "iris_state_get",
        0x51 => "iris_state_set",
        0x55 => "iris_state_empty",
        // Graph cost/type/edges
        0x60 => "iris_graph_get_node_cost",
        0x61 => "iris_graph_set_node_type",
        0x62 => "iris_graph_get_node_type",
        0x63 => "iris_graph_edges",
        0x64 => "iris_graph_get_arity",
        0x65 => "iris_graph_get_depth",
        0x66 => "iris_graph_get_lit_type_tag",
        // Knowledge graph
        0x70 => "iris_kg_empty",
        0x71 => "iris_kg_add_node",
        0x72 => "iris_kg_add_edge",
        0x73 => "iris_kg_get_node",
        0x75 => "iris_kg_neighbors",
        0x76 => "iris_kg_set_edge_weight",
        0x77 => "iris_kg_map_nodes",
        0x78 => "iris_kg_merge",
        0x79 => "iris_kg_query_by_edge_type",
        0x7A => "iris_kg_node_count",
        0x7B => "iris_kg_edge_count",
        // Graph introspection
        0x80 => "iris_self_graph",
        0x81 => "iris_graph_nodes",
        0x82 => "iris_graph_get_kind",
        0x83 => "iris_graph_get_prim_op",
        0x84 => "iris_graph_set_prim_op",
        0x85 => "iris_graph_add_node_rt",
        0x86 => "iris_graph_connect",
        0x87 => "iris_graph_disconnect",
        0x88 => "iris_graph_replace_subtree",
        0x89 => "iris_graph_eval",
        0x8A => "iris_graph_get_root",
        0x8B => "iris_graph_add_guard_rt",
        0x8C => "iris_graph_add_ref_rt",
        0x8D => "iris_graph_set_cost",
        0x8E => "iris_graph_get_lit_value",
        0x8F => "iris_graph_outgoing",
        // Parallel
        0x90 => "iris_par_eval",
        0x91 => "iris_par_map",
        0x92 => "iris_par_fold",
        0x93 => "iris_spawn",
        0x94 => "iris_await_future",
        0x95 => "iris_par_zip_with",
        0x96 => "iris_graph_edge_count",
        0x97 => "iris_graph_edge_target",
        0x98 => "iris_graph_get_binder",
        0x99 => "iris_graph_eval_env",
        0x9A => "iris_graph_get_tag",
        0x9B => "iris_graph_get_field_index",
        0x9C => "iris_value_get_tag",
        0x9D => "iris_value_get_payload",
        0x9E => "iris_value_make_tagged",
        0x9F => "iris_graph_get_effect_tag",
        // Evolve / effects
        0xA0 => "iris_evolve_subprogram",
        0xA1 => "iris_perform_effect",
        0xA2 => "iris_graph_eval_ref",
        0xA3 => "iris_compile_source_json",
        // String
        0xB0 => "iris_str_len",
        0xB1 => "iris_str_concat",
        0xB2 => "iris_str_slice",
        0xB3 => "iris_str_contains",
        0xB4 => "iris_str_split",
        0xB5 => "iris_str_join",
        0xB6 => "iris_str_to_int",
        0xB7 => "iris_int_to_string",
        0xB8 => "iris_str_eq",
        0xB9 => "iris_str_starts_with",
        0xBA => "iris_str_ends_with",
        0xBB => "iris_str_replace",
        0xBC => "iris_str_trim",
        0xBD => "iris_str_upper",
        0xBE => "iris_str_lower",
        0xBF => "iris_str_chars",
        0xC0 => "iris_char_at",
        // List
        0xC1 => "iris_list_append",
        0xC2 => "iris_list_nth",
        0xC3 => "iris_list_take",
        0xC4 => "iris_list_drop",
        0xC5 => "iris_list_sort",
        0xC6 => "iris_list_dedup",
        0xC7 => "iris_list_range",
        0xC8 => "iris_map_insert",
        0xC9 => "iris_map_get",
        0xCA => "iris_map_remove",
        0xCB => "iris_map_keys",
        0xCC => "iris_map_values",
        0xCD => "iris_map_size",
        0xCE => "iris_list_concat",
        0xCF => "iris_sort_by",
        // Tuple
        0xD2 => "iris_tuple_get",
        0xD3 => "iris_buf_new",
        0xD4 => "iris_buf_push",
        0xD5 => "iris_buf_finish",
        0xD6 => "iris_tuple_len",
        // Math
        0xD8 => "iris_math_sqrt",
        0xD9 => "iris_math_log",
        0xDA => "iris_math_exp",
        0xDB => "iris_math_sin",
        0xDC => "iris_math_cos",
        0xDD => "iris_math_floor",
        0xDE => "iris_math_ceil",
        0xDF => "iris_math_round",
        0xE0 => "iris_math_pi",
        0xE1 => "iris_math_e",
        0xE2 => "iris_random_int",
        0xE3 => "iris_random_float",
        // Bytes
        0xE6 => "iris_bytes_from_ints",
        0xE7 => "iris_bytes_concat",
        0xE8 => "iris_bytes_len",
        // Lazy
        0xE9 => "iris_lazy_unfold",
        0xEA => "iris_thunk_force",
        0xEB => "iris_lazy_take",
        0xEC => "iris_lazy_map",
        // Graph new/root
        0xED => "iris_graph_new",
        0xEE => "iris_graph_set_root",
        0xEF => "iris_graph_set_lit_value",
        // List len
        0xF0 => "iris_list_len",
        0xF1 => "iris_graph_set_field_index",
        // I/O
        0xF2 => "iris_file_read",
        0xF3 => "iris_compile_source",
        0xF4 => "iris_debug_print",
        0xF5 => "iris_module_eval",
        0xF6 => "iris_compile_test_file",
        0xF7 => "iris_module_test_count",
        0xF8 => "iris_module_eval_test",
        _ => "iris_unknown_op",
    }
}

// ---------------------------------------------------------------------------
// Build edge lookup: source → [(port, target)]
// ---------------------------------------------------------------------------

fn build_edge_map(graph: &SemanticGraph) -> HashMap<NodeId, Vec<(u8, NodeId)>> {
    let mut map: HashMap<NodeId, Vec<(u8, NodeId)>> = HashMap::new();
    for edge in &graph.edges {
        if edge.label == EdgeLabel::Argument {
            map.entry(edge.source)
                .or_default()
                .push((edge.port, edge.target));
        }
    }
    // Sort each node's arguments by port
    for args in map.values_mut() {
        args.sort_by_key(|(port, _)| *port);
    }
    map
}

// ---------------------------------------------------------------------------
// Code generator
// ---------------------------------------------------------------------------

struct CodeGen<'a> {
    graph: &'a SemanticGraph,
    edge_map: HashMap<NodeId, Vec<(u8, NodeId)>>,
    out: String,
    indent: usize,
}

impl<'a> CodeGen<'a> {
    fn new(graph: &'a SemanticGraph) -> Self {
        let edge_map = build_edge_map(graph);
        Self {
            graph,
            edge_map,
            out: String::new(),
            indent: 1,
        }
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    /// Generate C code for the entire graph, writing the function body.
    fn generate(&mut self) {
        self.emit_header();
        self.emit_node(self.graph.root);
        self.emit_footer();
    }

    fn emit_header(&mut self) {
        self.out.push_str("/* Generated from bootstrap/interpreter.json */\n");
        self.out.push_str("/* Do not edit — regenerate with iris-c-codegen */\n\n");
        self.out.push_str("#include \"iris_rt.h\"\n\n");
        self.out.push_str(
            "iris_value_t *iris_interpret(iris_value_t *program, iris_value_t *inputs) {\n",
        );
    }

    fn emit_footer(&mut self) {
        self.out.push_str("}\n");
    }

    /// Emit a node as a statement (for Guard) or return-expression.
    fn emit_node(&mut self, node_id: NodeId) {
        let node = match self.graph.nodes.get(&node_id) {
            Some(n) => n,
            None => {
                let ind = self.indent_str();
                writeln!(
                    self.out,
                    "{}/* ERROR: node {} not found */",
                    ind, node_id.0
                )
                .unwrap();
                writeln!(self.out, "{}return iris_unit();", ind).unwrap();
                return;
            }
        };

        match &node.payload {
            NodePayload::Guard {
                predicate_node,
                body_node,
                fallback_node,
            } => {
                let pred_id = *predicate_node;
                let body_id = *body_node;
                let fallback_id = *fallback_node;
                let ind = self.indent_str();
                let pred_expr = self.node_expr(pred_id);
                writeln!(
                    self.out,
                    "{}/* Node {} (Guard) */",
                    ind, node_id.0
                )
                .unwrap();
                writeln!(
                    self.out,
                    "{}if (iris_is_truthy({})) {{",
                    ind, pred_expr
                )
                .unwrap();
                self.indent += 1;
                self.emit_node(body_id);
                self.indent -= 1;
                writeln!(self.out, "{}}} else {{", ind).unwrap();
                self.indent += 1;
                self.emit_node(fallback_id);
                self.indent -= 1;
                writeln!(self.out, "{}}}", ind).unwrap();
            }
            _ => {
                let ind = self.indent_str();
                let expr = self.node_expr(node_id);
                writeln!(self.out, "{}return {};", ind, expr).unwrap();
            }
        }
    }

    /// Generate C expression for a node (inline).  Guard nodes inside
    /// expressions are wrapped in a ternary operator.
    fn node_expr(&self, node_id: NodeId) -> String {
        let node = match self.graph.nodes.get(&node_id) {
            Some(n) => n,
            None => return format!("/* missing node {} */ iris_unit()", node_id.0),
        };

        match &node.payload {
            NodePayload::Lit { type_tag, value } => self.lit_expr(*type_tag, value),

            NodePayload::Prim { opcode } => {
                let fname = opcode_to_c_fn(*opcode);
                let args = self.edge_map.get(&node_id);
                match args {
                    Some(arg_edges) => {
                        let arg_exprs: Vec<String> = arg_edges
                            .iter()
                            .map(|(_, target)| self.node_expr(*target))
                            .collect();
                        format!("{}({})", fname, arg_exprs.join(", "))
                    }
                    None => {
                        // Zero-argument primitive
                        format!("{}()", fname)
                    }
                }
            }

            NodePayload::Guard {
                predicate_node,
                body_node,
                fallback_node,
            } => {
                // Guard used as sub-expression: emit as ternary
                let pred = self.node_expr(*predicate_node);
                let body = self.node_expr(*body_node);
                let fallback = self.node_expr(*fallback_node);
                format!(
                    "(iris_is_truthy({}) ? {} : {})",
                    pred, body, fallback
                )
            }

            _ => {
                format!(
                    "/* unsupported node kind {:?} {} */ iris_unit()",
                    node.kind, node_id.0
                )
            }
        }
    }

    /// Generate C expression for a Lit node.
    fn lit_expr(&self, type_tag: u8, value: &[u8]) -> String {
        if type_tag == 0xFF {
            // Input reference
            let index = value_to_i64(value);
            match index {
                0 => "program".to_string(),
                1 => "inputs".to_string(),
                _ => format!("iris_tuple_get(inputs, iris_int({}))", index - 1),
            }
        } else if type_tag == 0 {
            // Integer literal
            let val = value_to_i64(value);
            format!("iris_int({})", val)
        } else {
            // String literal (or other types) — emit as string
            let s = String::from_utf8_lossy(value);
            let escaped = escape_c_string(&s);
            format!("iris_string(\"{}\")", escaped)
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Interpret a little-endian byte slice as a signed 64-bit integer.
fn value_to_i64(bytes: &[u8]) -> i64 {
    let mut buf = [0u8; 8];
    let len = bytes.len().min(8);
    buf[..len].copy_from_slice(&bytes[..len]);
    i64::from_le_bytes(buf)
}

/// Escape a string for use in a C string literal.
fn escape_c_string(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if c.is_ascii_graphic() || c == ' ' => out.push(c),
            c => {
                // Emit as hex escape for non-printable
                for b in c.to_string().as_bytes() {
                    write!(out, "\\x{:02x}", b).unwrap();
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: iris-c-codegen <input.json>");
        process::exit(1);
    }

    let graph = load_graph(&args[1]).unwrap_or_else(|e| {
        eprintln!("Failed to load graph: {}", e);
        process::exit(1);
    });

    // Validate: only Lit, Prim, Guard nodes
    for (id, node) in &graph.nodes {
        match node.kind {
            NodeKind::Lit | NodeKind::Prim | NodeKind::Guard => {}
            other => {
                eprintln!(
                    "Warning: node {} has unsupported kind {:?} — will emit iris_unit()",
                    id.0, other
                );
            }
        }
    }

    let mut codegen = CodeGen::new(&graph);
    codegen.generate();
    print!("{}", codegen.out);
}
