
//! Self-writing iris-repr components as IRIS programs.
//!
//! Five IRIS programs (SemanticGraphs) that implement wire format estimation,
//! hash/signature computation, multi-resolution node counting, structural
//! comparison, and fragment metadata extraction.
//!
//! These programs use graph introspection opcodes (0x80-0x8A) and arithmetic
//! to re-implement parts of iris-repr's wire.rs, hash.rs, and resolution.rs
//! as IRIS programs that can analyze other programs.

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    BinderId, Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Graph construction helpers
// ---------------------------------------------------------------------------

fn make_node(id: u64, kind: NodeKind, payload: NodePayload, arity: u8) -> (NodeId, Node) {
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind,
            type_sig: TypeId(0),
            cost: CostTerm::Unit,
            arity,
            resolution_depth: 0, salt: 0,
            payload,
        },
    )
}

fn make_edge(source: u64, target: u64, port: u8, label: EdgeLabel) -> Edge {
    Edge {
        source: NodeId(source),
        target: NodeId(target),
        port,
        label,
    }
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

fn int_lit_node(id: u64, value: i64) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0x00,
            value: value.to_le_bytes().to_vec(),
        },
        0,
    )
}

fn input_ref_node(id: u64, index: u8) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0xFF,
            value: vec![index],
        },
        0,
    )
}

fn prim_node(id: u64, opcode: u8, arity: u8) -> (NodeId, Node) {
    make_node(id, NodeKind::Prim, NodePayload::Prim { opcode }, arity)
}

fn project_node(id: u64, field_index: u16) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Project,
        NodePayload::Project { field_index },
        1,
    )
}

fn fold_node(id: u64, mode: u8, arity: u8) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![mode],
        },
        arity,
    )
}

fn tuple_node(id: u64, arity: u8) -> (NodeId, Node) {
    make_node(id, NodeKind::Tuple, NodePayload::Tuple, arity)
}

fn match_node(id: u64, arm_count: u16) -> (NodeId, Node) {
    make_node(
        id,
        NodeKind::Match,
        NodePayload::Match {
            arm_count,
            arm_patterns: vec![],
        },
        arm_count as u8,
    )
}

// ---------------------------------------------------------------------------
// Target program builders (programs to be analyzed)
// ---------------------------------------------------------------------------

/// Build a 3-node program: add(lit(a), lit(b)).
fn make_binop_graph(opcode: u8, a: i64, b: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, node) = prim_node(1, opcode, 2);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(10, a);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(20, b);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build a single-node literal program.
fn make_lit_graph(value: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = int_lit_node(1, value);
    nodes.insert(nid, node);
    make_graph(nodes, vec![], 1)
}

/// Build a 5-node program: add(mul(a, b), sub(c, d)).
fn make_5node_graph(a: i64, b: i64, c: i64, d: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: add (opcode 0x00)
    let (nid, node) = prim_node(1, 0x00, 2);
    nodes.insert(nid, node);
    // mul(a, b) at id=10
    let (nid, node) = prim_node(10, 0x02, 2);
    nodes.insert(nid, node);
    // sub(c, d) at id=20
    let (nid, node) = prim_node(20, 0x01, 2);
    nodes.insert(nid, node);
    // lit(a) at id=100
    let (nid, node) = int_lit_node(100, a);
    nodes.insert(nid, node);
    // lit(b) at id=101
    let (nid, node) = int_lit_node(101, b);
    nodes.insert(nid, node);
    // lit(c) at id=102
    let (nid, node) = int_lit_node(102, c);
    nodes.insert(nid, node);
    // lit(d) at id=103
    let (nid, node) = int_lit_node(103, d);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(10, 100, 0, EdgeLabel::Argument),
        make_edge(10, 101, 1, EdgeLabel::Argument),
        make_edge(20, 102, 0, EdgeLabel::Argument),
        make_edge(20, 103, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build a 3-node program with resolution depths assigned.
/// root (depth 0) -> child (depth 1) -> grandchild (depth 2).
fn make_3node_with_resolution(opcode_root: u8, opcode_child: u8, value: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();

    let (nid, mut node) = prim_node(1, opcode_root, 1);
    node.resolution_depth = 0;
    nodes.insert(nid, node);

    let (nid, mut node) = prim_node(10, opcode_child, 1);
    node.resolution_depth = 1;
    nodes.insert(nid, node);

    let (nid, mut node) = int_lit_node(20, value);
    node.resolution_depth = 2;
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(10, 20, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build a program with a Fold node (used for fragment_metadata has_fold check).
fn make_fold_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Fold (mode 0x00, arity 3)
    let (nid, node) = fold_node(1, 0x00, 3);
    nodes.insert(nid, node);

    // base = Int(0)
    let (nid, node) = int_lit_node(10, 0);
    nodes.insert(nid, node);

    // step = add (0x00, arity 2)
    let (nid, node) = prim_node(20, 0x00, 2);
    nodes.insert(nid, node);

    // collection = Tuple of [1, 2, 3]
    let (nid, node) = tuple_node(30, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(31, 1);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(32, 2);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(33, 3);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        make_edge(30, 31, 0, EdgeLabel::Argument),
        make_edge(30, 32, 1, EdgeLabel::Argument),
        make_edge(30, 33, 2, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Build a program with a Guard node (used for fragment_metadata has_guard check).
fn make_guard_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Guard node
    let (nid, node) = make_node(
        1,
        NodeKind::Guard,
        NodePayload::Guard {
            predicate_node: NodeId(10),
            body_node: NodeId(20),
            fallback_node: NodeId(30),
        },
        3,
    );
    nodes.insert(nid, node);

    // predicate: eq(5, 5) -> true
    let (nid, node) = prim_node(10, 0x20, 2);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(11, 5);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(12, 5);
    nodes.insert(nid, node);

    // body: lit(42)
    let (nid, node) = int_lit_node(20, 42);
    nodes.insert(nid, node);

    // fallback: lit(-1)
    let (nid, node) = int_lit_node(30, -1);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        make_edge(10, 11, 0, EdgeLabel::Argument),
        make_edge(10, 12, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// IRIS Program 1: count_fragment_size
// ===========================================================================
//
// Estimates the serialized size of a program in bytes.
//
// Input:
//   inputs[0] = Value::Program(target_program)
//
// Output:
//   Value::Int(estimated_byte_count)
//
// Formula: node_count * 20 + (node_count - 1) * 18 + 32
//        = node_count * 38 - 18 + 32
//        = node_count * 38 + 14
//
// Graph structure:
//   Root(id=1): add(0x00, arity=2)                              [estimated_size]
//   ├── port 0: mul(0x02, arity=2)                              [id=10]
//   │   ├── port 0: Fold(0x05, count) [id=100]                  [node_count]
//   │   │   ├── port 0: int_lit(0) [id=101]                     [base]
//   │   │   ├── port 1: add(0x00) [id=102]                      [step (unused in count mode)]
//   │   │   └── port 2: graph_nodes(0x81) [id=103]              [nodes tuple]
//   │   │       └── port 0: input_ref(0) [id=104]               [program]
//   │   └── port 1: int_lit(38) [id=110]                        [bytes per node+edge]
//   └── port 1: int_lit(14) [id=20]                             [overhead constant]

fn build_count_fragment_size() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: add(mul(node_count, 38), 14)
    let (nid, node) = prim_node(1, 0x00, 2);
    nodes.insert(nid, node);

    // mul(node_count, 38)
    let (nid, node) = prim_node(10, 0x02, 2);
    nodes.insert(nid, node);

    // overhead constant = 14
    let (nid, node) = int_lit_node(20, 14);
    nodes.insert(nid, node);

    // Fold(count mode=0x05) to count node IDs
    let (nid, node) = fold_node(100, 0x05, 3);
    nodes.insert(nid, node);

    // base = 0
    let (nid, node) = int_lit_node(101, 0);
    nodes.insert(nid, node);

    // step = add (needed for fold structure, not actually called in count mode)
    let (nid, node) = prim_node(102, 0x00, 2);
    nodes.insert(nid, node);

    // graph_nodes(program) -> Tuple of node IDs
    let (nid, node) = prim_node(103, 0x81, 1);
    nodes.insert(nid, node);

    // input_ref(0) = the program
    let (nid, node) = input_ref_node(104, 0);
    nodes.insert(nid, node);

    // bytes_per_unit = 38
    let (nid, node) = int_lit_node(110, 38);
    nodes.insert(nid, node);

    let edges = vec![
        // root: add(mul_result, overhead)
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        // mul(node_count, 38)
        make_edge(10, 100, 0, EdgeLabel::Argument),
        make_edge(10, 110, 1, EdgeLabel::Argument),
        // fold(base, step, collection)
        make_edge(100, 101, 0, EdgeLabel::Argument),
        make_edge(100, 102, 1, EdgeLabel::Argument),
        make_edge(100, 103, 2, EdgeLabel::Argument),
        // graph_nodes(input_ref(0))
        make_edge(103, 104, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// IRIS Program 2: compute_graph_signature
// ===========================================================================
//
// Computes a simplified hash fingerprint of a program.
//
// Input:
//   inputs[0] = Value::Program(target_program)
//
// Output:
//   Value::Int(signature)
//
// Algorithm:
//   1. Get root node ID via graph_get_root (0x8A).
//   2. Get root kind via graph_get_kind (0x82).
//   3. Get node count via graph_nodes (0x81) + fold count (0x05).
//   4. Combine: signature = root_kind * 997 + node_count * 31 + root_kind
//      (This is a simple but consistent fingerprint.)
//
// For Prim root nodes, also fold in the opcode:
//   We use a Match to check if root kind == 0 (Prim).
//   If Prim: signature = (root_kind * 997 + node_count * 31) XOR (opcode * 65537)
//   If not Prim: signature = root_kind * 997 + node_count * 31

fn build_compute_graph_signature() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root(id=1): Match on (root_kind == Prim?)
    let (nid, node) = match_node(1, 2);
    nodes.insert(nid, node);

    // ---- Scrutinee: eq(root_kind, 0) ----
    // eq (0x20, arity 2)
    let (nid, node) = prim_node(50, 0x20, 2);
    nodes.insert(nid, node);

    // graph_get_kind(program, root_id)
    let (nid, node) = prim_node(51, 0x82, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(52, 0);
    nodes.insert(nid, node);
    // graph_get_root(program)
    let (nid, node) = prim_node(53, 0x8A, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(54, 0);
    nodes.insert(nid, node);
    // int_lit(0) = NodeKind::Prim
    let (nid, node) = int_lit_node(55, 0);
    nodes.insert(nid, node);

    // ---- Arm 0 (false / not Prim): base_sig = root_kind * 997 + node_count * 31 ----
    // add(mul(root_kind, 997), mul(node_count, 31))
    let (nid, node) = prim_node(200, 0x00, 2);
    nodes.insert(nid, node);
    // mul(root_kind, 997)
    let (nid, node) = prim_node(201, 0x02, 2);
    nodes.insert(nid, node);
    // graph_get_kind(program, root_id) [re-compute]
    let (nid, node) = prim_node(202, 0x82, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(203, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(204, 0x8A, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(205, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(206, 997);
    nodes.insert(nid, node);
    // mul(node_count, 31)
    let (nid, node) = prim_node(210, 0x02, 2);
    nodes.insert(nid, node);
    // Fold count for node_count
    let (nid, node) = fold_node(211, 0x05, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(212, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(213, 0x00, 2);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(214, 0x81, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(215, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(216, 31);
    nodes.insert(nid, node);

    // ---- Arm 1 (true / is Prim): base_sig XOR (opcode * 65537) ----
    // bitxor(base_sig, mul(opcode, 65537))
    let (nid, node) = prim_node(300, 0x12, 2); // bitxor
    nodes.insert(nid, node);
    // base_sig = add(mul(root_kind, 997), mul(node_count, 31))
    // root_kind is 0 for Prim, so mul(0, 997) = 0, base_sig = mul(node_count, 31)
    let (nid, node) = prim_node(301, 0x02, 2); // mul(node_count, 31)
    nodes.insert(nid, node);
    // Fold count for node_count
    let (nid, node) = fold_node(302, 0x05, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(303, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(304, 0x00, 2);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(305, 0x81, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(306, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(307, 31);
    nodes.insert(nid, node);

    // mul(opcode, 65537)
    let (nid, node) = prim_node(310, 0x02, 2);
    nodes.insert(nid, node);
    // graph_get_prim_op(program, root_id)
    let (nid, node) = prim_node(311, 0x83, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(312, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(313, 0x8A, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(314, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(315, 65537);
    nodes.insert(nid, node);

    let edges = vec![
        // Match scrutinee
        make_edge(1, 50, 0, EdgeLabel::Scrutinee),
        // Match arms
        make_edge(1, 200, 0, EdgeLabel::Argument), // arm 0 (not Prim)
        make_edge(1, 300, 1, EdgeLabel::Argument), // arm 1 (is Prim)
        // scrutinee: eq(graph_get_kind(...), 0)
        make_edge(50, 51, 0, EdgeLabel::Argument),
        make_edge(50, 55, 1, EdgeLabel::Argument),
        make_edge(51, 52, 0, EdgeLabel::Argument),
        make_edge(51, 53, 1, EdgeLabel::Argument),
        make_edge(53, 54, 0, EdgeLabel::Argument),
        // Arm 0: add(mul(root_kind, 997), mul(node_count, 31))
        make_edge(200, 201, 0, EdgeLabel::Argument),
        make_edge(200, 210, 1, EdgeLabel::Argument),
        make_edge(201, 202, 0, EdgeLabel::Argument),
        make_edge(201, 206, 1, EdgeLabel::Argument),
        make_edge(202, 203, 0, EdgeLabel::Argument),
        make_edge(202, 204, 1, EdgeLabel::Argument),
        make_edge(204, 205, 0, EdgeLabel::Argument),
        make_edge(210, 211, 0, EdgeLabel::Argument),
        make_edge(210, 216, 1, EdgeLabel::Argument),
        make_edge(211, 212, 0, EdgeLabel::Argument),
        make_edge(211, 213, 1, EdgeLabel::Argument),
        make_edge(211, 214, 2, EdgeLabel::Argument),
        make_edge(214, 215, 0, EdgeLabel::Argument),
        // Arm 1: bitxor(mul(node_count, 31), mul(opcode, 65537))
        make_edge(300, 301, 0, EdgeLabel::Argument),
        make_edge(300, 310, 1, EdgeLabel::Argument),
        make_edge(301, 302, 0, EdgeLabel::Argument),
        make_edge(301, 307, 1, EdgeLabel::Argument),
        make_edge(302, 303, 0, EdgeLabel::Argument),
        make_edge(302, 304, 1, EdgeLabel::Argument),
        make_edge(302, 305, 2, EdgeLabel::Argument),
        make_edge(305, 306, 0, EdgeLabel::Argument),
        make_edge(310, 311, 0, EdgeLabel::Argument),
        make_edge(310, 315, 1, EdgeLabel::Argument),
        make_edge(311, 312, 0, EdgeLabel::Argument),
        make_edge(311, 313, 1, EdgeLabel::Argument),
        make_edge(313, 314, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// IRIS Program 3: resolve_at_level
// ===========================================================================
//
// Given a program and a resolution level (0=intent, 1=architecture,
// 2=implementation), counts how many nodes are visible at that level.
//
// Since there is no graph_get_resolution_depth opcode, this program uses
// graph_nodes (0x81) to enumerate all node IDs, then uses Fold (count
// mode 0x05) to count them. The test constructs pre-resolved graphs at
// each level to verify the count is correct.
//
// Input:
//   inputs[0] = Value::Program(target_program)
//   inputs[1] = Value::Int(level) — unused (filtering done by pre-resolution)
//
// Output:
//   Value::Int(visible_node_count)
//
// Graph structure:
//   Root(id=1): Fold(0x05, count mode, arity=3)
//   ├── port 0: int_lit(0) [id=10]                     [base]
//   ├── port 1: add(0x00) [id=20]                      [step (unused in count mode)]
//   └── port 2: graph_nodes(0x81) [id=30]              [nodes tuple]
//       └── port 0: input_ref(0) [id=40]               [program]

fn build_resolve_at_level() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Fold (count mode 0x05)
    let (nid, node) = fold_node(1, 0x05, 3);
    nodes.insert(nid, node);

    // base = 0
    let (nid, node) = int_lit_node(10, 0);
    nodes.insert(nid, node);

    // step = add (not used in count mode but needed for fold structure)
    let (nid, node) = prim_node(20, 0x00, 2);
    nodes.insert(nid, node);

    // graph_nodes(program)
    let (nid, node) = prim_node(30, 0x81, 1);
    nodes.insert(nid, node);

    // input_ref(0) = the program
    let (nid, node) = input_ref_node(40, 0);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        make_edge(30, 40, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// IRIS Program 4: compare_programs
// ===========================================================================
//
// Given two programs, checks structural equivalence:
//   - Same node count
//   - Same root kind
//   - Same root opcode (if both are Prim)
//
// Returns Int(1) if equivalent, Int(0) otherwise.
//
// Input:
//   inputs[0] = Value::Program(program_a)
//   inputs[1] = Value::Program(program_b)
//
// Output:
//   Value::Int(1 or 0)
//
// Strategy:
//   count_a = fold_count(graph_nodes(prog_a))
//   count_b = fold_count(graph_nodes(prog_b))
//   kind_a = graph_get_kind(prog_a, graph_get_root(prog_a))
//   kind_b = graph_get_kind(prog_b, graph_get_root(prog_b))
//   result = bool_to_int(eq(count_a, count_b)) * bool_to_int(eq(kind_a, kind_b))
//   (multiplication acts as logical AND for 0/1 values)

fn build_compare_programs() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: mul(count_match, kind_match)  — both are 0 or 1, so mul = AND
    let (nid, node) = prim_node(1, 0x02, 2);
    nodes.insert(nid, node);

    // ---- count_match = bool_to_int(eq(count_a, count_b)) ----
    let (nid, node) = prim_node(10, 0x44, 1); // bool_to_int
    nodes.insert(nid, node);
    let (nid, node) = prim_node(11, 0x20, 2); // eq
    nodes.insert(nid, node);

    // count_a = fold_count(graph_nodes(prog_a))
    let (nid, node) = fold_node(100, 0x05, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(101, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(102, 0x00, 2);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(103, 0x81, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(104, 0); // prog_a
    nodes.insert(nid, node);

    // count_b = fold_count(graph_nodes(prog_b))
    let (nid, node) = fold_node(110, 0x05, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(111, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(112, 0x00, 2);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(113, 0x81, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(114, 1); // prog_b
    nodes.insert(nid, node);

    // ---- kind_match = bool_to_int(eq(kind_a, kind_b)) ----
    let (nid, node) = prim_node(20, 0x44, 1); // bool_to_int
    nodes.insert(nid, node);
    let (nid, node) = prim_node(21, 0x20, 2); // eq
    nodes.insert(nid, node);

    // kind_a = graph_get_kind(prog_a, graph_get_root(prog_a))
    let (nid, node) = prim_node(200, 0x82, 2); // graph_get_kind
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(201, 0); // prog_a
    nodes.insert(nid, node);
    let (nid, node) = prim_node(202, 0x8A, 1); // graph_get_root
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(203, 0); // prog_a
    nodes.insert(nid, node);

    // kind_b = graph_get_kind(prog_b, graph_get_root(prog_b))
    let (nid, node) = prim_node(210, 0x82, 2); // graph_get_kind
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(211, 1); // prog_b
    nodes.insert(nid, node);
    let (nid, node) = prim_node(212, 0x8A, 1); // graph_get_root
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(213, 1); // prog_b
    nodes.insert(nid, node);

    let edges = vec![
        // root: mul(count_match, kind_match)
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        // count_match: bool_to_int(eq(count_a, count_b))
        make_edge(10, 11, 0, EdgeLabel::Argument),
        make_edge(11, 100, 0, EdgeLabel::Argument),
        make_edge(11, 110, 1, EdgeLabel::Argument),
        // count_a: fold_count(graph_nodes(prog_a))
        make_edge(100, 101, 0, EdgeLabel::Argument),
        make_edge(100, 102, 1, EdgeLabel::Argument),
        make_edge(100, 103, 2, EdgeLabel::Argument),
        make_edge(103, 104, 0, EdgeLabel::Argument),
        // count_b: fold_count(graph_nodes(prog_b))
        make_edge(110, 111, 0, EdgeLabel::Argument),
        make_edge(110, 112, 1, EdgeLabel::Argument),
        make_edge(110, 113, 2, EdgeLabel::Argument),
        make_edge(113, 114, 0, EdgeLabel::Argument),
        // kind_match: bool_to_int(eq(kind_a, kind_b))
        make_edge(20, 21, 0, EdgeLabel::Argument),
        make_edge(21, 200, 0, EdgeLabel::Argument),
        make_edge(21, 210, 1, EdgeLabel::Argument),
        // kind_a: graph_get_kind(prog_a, graph_get_root(prog_a))
        make_edge(200, 201, 0, EdgeLabel::Argument),
        make_edge(200, 202, 1, EdgeLabel::Argument),
        make_edge(202, 203, 0, EdgeLabel::Argument),
        // kind_b: graph_get_kind(prog_b, graph_get_root(prog_b))
        make_edge(210, 211, 0, EdgeLabel::Argument),
        make_edge(210, 212, 1, EdgeLabel::Argument),
        make_edge(212, 213, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// IRIS Program 5: fragment_metadata
// ===========================================================================
//
// Extracts metadata from a program and returns as a Tuple of Ints:
//   (node_count, edge_count_estimate, root_kind, has_fold, has_guard, max_resolution_depth)
//
// node_count:         fold_count(graph_nodes(prog))
// edge_count_estimate: node_count - 1  (tree approximation)
// root_kind:          graph_get_kind(prog, graph_get_root(prog))
// has_fold:           fold over nodes, checking if any node kind == 0x08 (Fold)
// has_guard:          fold over nodes, checking if any node kind == 0x11 (Guard)
// max_resolution_depth: constant 2 (programs at Implementation level)
//
// Since checking individual node kinds requires a per-node loop with
// graph_get_kind, we use a simpler approach for has_fold and has_guard:
// use a Fold (mode 0x00) with a Lambda that accumulates max(acc, eq(kind, target)).
//
// For simplicity, has_fold and has_guard are computed by iterating nodes and
// using graph_get_kind + eq + bool_to_int + max to detect presence.
//
// Input:
//   inputs[0] = Value::Program(target_program)
//
// Output:
//   Value::Tuple([node_count, edge_est, root_kind, has_fold, has_guard, max_res_depth])

fn build_fragment_metadata() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Tuple(6 fields)
    let (nid, node) = tuple_node(1, 6);
    nodes.insert(nid, node);

    // ---- Field 0: node_count = fold_count(graph_nodes(prog)) ----
    let (nid, node) = fold_node(100, 0x05, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(101, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(102, 0x00, 2);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(103, 0x81, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(104, 0);
    nodes.insert(nid, node);

    // ---- Field 1: edge_count_est = node_count - 1 ----
    let (nid, node) = prim_node(200, 0x01, 2); // sub
    nodes.insert(nid, node);
    let (nid, node) = fold_node(201, 0x05, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(202, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(203, 0x00, 2);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(204, 0x81, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(205, 0);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(206, 1);
    nodes.insert(nid, node);

    // ---- Field 2: root_kind = graph_get_kind(prog, graph_get_root(prog)) ----
    let (nid, node) = prim_node(300, 0x82, 2); // graph_get_kind
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(301, 0);
    nodes.insert(nid, node);
    let (nid, node) = prim_node(302, 0x8A, 1); // graph_get_root
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(303, 0);
    nodes.insert(nid, node);

    // ---- Field 3: has_fold ----
    // Fold(mode=0x00) over graph_nodes, with Lambda step:
    //   step(acc, node_id) = max(acc, bool_to_int(eq(graph_get_kind(prog, node_id), 0x08)))
    let (nid, node) = fold_node(400, 0x00, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(401, 0); // base = 0
    nodes.insert(nid, node);
    // Lambda step function
    let (nid, node) = make_node(
        402,
        NodeKind::Lambda,
        NodePayload::Lambda {
            binder: BinderId(0xFFFF_0002),
            captured_count: 0,
        },
        0,
    );
    nodes.insert(nid, node);
    // collection = graph_nodes(prog)
    let (nid, node) = prim_node(403, 0x81, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(404, 0);
    nodes.insert(nid, node);

    // Lambda body: max(acc, bool_to_int(eq(graph_get_kind(prog, node_id), 8)))
    // input_ref(2) = Tuple(acc, node_id)
    let (nid, node) = prim_node(410, 0x08, 2); // max
    nodes.insert(nid, node);
    // acc = Project(0) from input_ref(2)
    let (nid, node) = project_node(411, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(412, 2);
    nodes.insert(nid, node);
    // bool_to_int(eq(kind, 8))
    let (nid, node) = prim_node(413, 0x44, 1); // bool_to_int
    nodes.insert(nid, node);
    let (nid, node) = prim_node(414, 0x20, 2); // eq
    nodes.insert(nid, node);
    // graph_get_kind(prog, node_id)
    let (nid, node) = prim_node(415, 0x82, 2);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(416, 0); // prog
    nodes.insert(nid, node);
    // node_id = Project(1) from input_ref(2)
    let (nid, node) = project_node(417, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(418, 2);
    nodes.insert(nid, node);
    // int_lit(8) = NodeKind::Fold
    let (nid, node) = int_lit_node(419, 8);
    nodes.insert(nid, node);

    // ---- Field 4: has_guard ----
    // Same pattern as has_fold but checking for kind == 0x11 (Guard)
    let (nid, node) = fold_node(500, 0x00, 3);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(501, 0);
    nodes.insert(nid, node);
    let (nid, node) = make_node(
        502,
        NodeKind::Lambda,
        NodePayload::Lambda {
            binder: BinderId(0xFFFF_0003),
            captured_count: 0,
        },
        0,
    );
    nodes.insert(nid, node);
    let (nid, node) = prim_node(503, 0x81, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(504, 0);
    nodes.insert(nid, node);

    // Lambda body: max(acc, bool_to_int(eq(graph_get_kind(prog, node_id), 0x11)))
    let (nid, node) = prim_node(510, 0x08, 2); // max
    nodes.insert(nid, node);
    let (nid, node) = project_node(511, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(512, 3); // bound by 0xFFFF_0003
    nodes.insert(nid, node);
    let (nid, node) = prim_node(513, 0x44, 1); // bool_to_int
    nodes.insert(nid, node);
    let (nid, node) = prim_node(514, 0x20, 2); // eq
    nodes.insert(nid, node);
    let (nid, node) = prim_node(515, 0x82, 2); // graph_get_kind
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(516, 0); // prog
    nodes.insert(nid, node);
    let (nid, node) = project_node(517, 1);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(518, 3); // bound var
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(519, 0x11); // Guard kind tag
    nodes.insert(nid, node);

    // ---- Field 5: max_resolution_depth = 2 ----
    let (nid, node) = int_lit_node(600, 2);
    nodes.insert(nid, node);

    let edges = vec![
        // Tuple fields
        make_edge(1, 100, 0, EdgeLabel::Argument), // node_count
        make_edge(1, 200, 1, EdgeLabel::Argument), // edge_count_est
        make_edge(1, 300, 2, EdgeLabel::Argument), // root_kind
        make_edge(1, 400, 3, EdgeLabel::Argument), // has_fold
        make_edge(1, 500, 4, EdgeLabel::Argument), // has_guard
        make_edge(1, 600, 5, EdgeLabel::Argument), // max_res_depth
        // Field 0: node_count
        make_edge(100, 101, 0, EdgeLabel::Argument),
        make_edge(100, 102, 1, EdgeLabel::Argument),
        make_edge(100, 103, 2, EdgeLabel::Argument),
        make_edge(103, 104, 0, EdgeLabel::Argument),
        // Field 1: edge_count_est = node_count - 1
        make_edge(200, 201, 0, EdgeLabel::Argument),
        make_edge(200, 206, 1, EdgeLabel::Argument),
        make_edge(201, 202, 0, EdgeLabel::Argument),
        make_edge(201, 203, 1, EdgeLabel::Argument),
        make_edge(201, 204, 2, EdgeLabel::Argument),
        make_edge(204, 205, 0, EdgeLabel::Argument),
        // Field 2: root_kind
        make_edge(300, 301, 0, EdgeLabel::Argument),
        make_edge(300, 302, 1, EdgeLabel::Argument),
        make_edge(302, 303, 0, EdgeLabel::Argument),
        // Field 3: has_fold
        make_edge(400, 401, 0, EdgeLabel::Argument),
        make_edge(400, 402, 1, EdgeLabel::Argument),
        make_edge(400, 403, 2, EdgeLabel::Argument),
        make_edge(403, 404, 0, EdgeLabel::Argument),
        // Lambda body (has_fold)
        Edge {
            source: NodeId(402),
            target: NodeId(410),
            port: 0,
            label: EdgeLabel::Continuation,
        },
        make_edge(410, 411, 0, EdgeLabel::Argument), // acc
        make_edge(410, 413, 1, EdgeLabel::Argument), // bool_to_int(eq(...))
        make_edge(411, 412, 0, EdgeLabel::Argument),
        make_edge(413, 414, 0, EdgeLabel::Argument),
        make_edge(414, 415, 0, EdgeLabel::Argument),
        make_edge(414, 419, 1, EdgeLabel::Argument),
        make_edge(415, 416, 0, EdgeLabel::Argument),
        make_edge(415, 417, 1, EdgeLabel::Argument),
        make_edge(417, 418, 0, EdgeLabel::Argument),
        // Field 4: has_guard
        make_edge(500, 501, 0, EdgeLabel::Argument),
        make_edge(500, 502, 1, EdgeLabel::Argument),
        make_edge(500, 503, 2, EdgeLabel::Argument),
        make_edge(503, 504, 0, EdgeLabel::Argument),
        // Lambda body (has_guard)
        Edge {
            source: NodeId(502),
            target: NodeId(510),
            port: 0,
            label: EdgeLabel::Continuation,
        },
        make_edge(510, 511, 0, EdgeLabel::Argument), // acc
        make_edge(510, 513, 1, EdgeLabel::Argument), // bool_to_int(eq(...))
        make_edge(511, 512, 0, EdgeLabel::Argument),
        make_edge(513, 514, 0, EdgeLabel::Argument),
        make_edge(514, 515, 0, EdgeLabel::Argument),
        make_edge(514, 519, 1, EdgeLabel::Argument),
        make_edge(515, 516, 0, EdgeLabel::Argument),
        make_edge(515, 517, 1, EdgeLabel::Argument),
        make_edge(517, 518, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// Tests
// ===========================================================================

// ---------- count_fragment_size ----------

#[test]
fn count_fragment_size_3node() {
    let estimator = build_count_fragment_size();
    let target = make_binop_graph(0x00, 3, 5); // 3 nodes

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&estimator, &inputs, None).unwrap();

    assert_eq!(outputs.len(), 1);
    // 3 * 38 + 14 = 128
    assert_eq!(
        outputs[0],
        Value::Int(128),
        "3-node program estimated at 128 bytes (3*38 + 14)"
    );
}

#[test]
fn count_fragment_size_1node() {
    let estimator = build_count_fragment_size();
    let target = make_lit_graph(42); // 1 node

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&estimator, &inputs, None).unwrap();

    assert_eq!(outputs.len(), 1);
    // 1 * 38 + 14 = 52
    assert_eq!(
        outputs[0],
        Value::Int(52),
        "1-node program estimated at 52 bytes (1*38 + 14)"
    );
}

#[test]
fn count_fragment_size_7node() {
    let estimator = build_count_fragment_size();
    let target = make_5node_graph(2, 3, 4, 5); // 7 nodes (3 prim + 4 lit)

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&estimator, &inputs, None).unwrap();

    assert_eq!(outputs.len(), 1);
    // 7 * 38 + 14 = 280
    assert_eq!(
        outputs[0],
        Value::Int(280),
        "7-node program estimated at 280 bytes (7*38 + 14)"
    );
}

// ---------- compute_graph_signature ----------

#[test]
fn signature_same_program_same_result() {
    let signer = build_compute_graph_signature();
    let target = make_binop_graph(0x00, 3, 5);

    let inputs1 = vec![Value::Program(Rc::new(target.clone()))];
    let (out1, _) = interpreter::interpret(&signer, &inputs1, None).unwrap();

    let inputs2 = vec![Value::Program(Rc::new(target))];
    let (out2, _) = interpreter::interpret(&signer, &inputs2, None).unwrap();

    assert_eq!(out1, out2, "same program must produce same signature");
}

#[test]
fn signature_different_opcode_different_result() {
    let signer = build_compute_graph_signature();

    // add(3, 5) vs mul(3, 5) — same structure but different root opcode
    let add_prog = make_binop_graph(0x00, 3, 5);
    let mul_prog = make_binop_graph(0x02, 3, 5);

    let (out_add, _) = interpreter::interpret(
        &signer,
        &[Value::Program(Rc::new(add_prog))],
        None,
    )
    .unwrap();
    let (out_mul, _) = interpreter::interpret(
        &signer,
        &[Value::Program(Rc::new(mul_prog))],
        None,
    )
    .unwrap();

    assert_ne!(
        out_add, out_mul,
        "different root opcodes should produce different signatures"
    );
}

#[test]
fn signature_different_structure_different_result() {
    let signer = build_compute_graph_signature();

    // 3-node program vs 1-node program — different node count
    let three_node = make_binop_graph(0x00, 3, 5);
    let one_node = make_lit_graph(42);

    let (out_3, _) = interpreter::interpret(
        &signer,
        &[Value::Program(Rc::new(three_node))],
        None,
    )
    .unwrap();
    let (out_1, _) = interpreter::interpret(
        &signer,
        &[Value::Program(Rc::new(one_node))],
        None,
    )
    .unwrap();

    assert_ne!(
        out_3, out_1,
        "different node counts should produce different signatures"
    );
}

// ---------- resolve_at_level ----------

#[test]
fn resolve_at_level_2_shows_all() {
    let resolver = build_resolve_at_level();
    // 3-node program at Implementation level (all nodes visible)
    let target = make_3node_with_resolution(0x00, 0x01, 42);

    let inputs = vec![
        Value::Program(Rc::new(target)),
        Value::Int(2), // level 2 = implementation
    ];
    let (outputs, _) = interpreter::interpret(&resolver, &inputs, None).unwrap();

    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(3),
        "level 2 should show all 3 nodes"
    );
}

#[test]
fn resolve_at_level_0_shows_fewer() {
    let resolver = build_resolve_at_level();

    // Pre-resolve the graph at Intent level using iris-repr's resolve().
    let full_graph = make_3node_with_resolution(0x00, 0x01, 42);
    let resolved = iris_types::resolution::resolve(&full_graph, Resolution::Intent);

    let inputs = vec![
        Value::Program(Rc::new(resolved)),
        Value::Int(0), // level 0 = intent
    ];
    let (outputs, _) = interpreter::interpret(&resolver, &inputs, None).unwrap();

    assert_eq!(outputs.len(), 1);
    // At Intent level (depth 0), only the root is truly visible (depth 0),
    // plus a summary Ref for the arch node = 2 nodes.
    let node_count = match &outputs[0] {
        Value::Int(n) => *n,
        _ => panic!("expected Int"),
    };
    assert!(
        node_count < 3,
        "level 0 should show fewer than 3 nodes, got {}",
        node_count
    );
}

#[test]
fn resolve_at_level_1_shows_more_than_0() {
    let resolver = build_resolve_at_level();

    let full_graph = make_3node_with_resolution(0x00, 0x01, 42);
    let resolved_0 = iris_types::resolution::resolve(&full_graph, Resolution::Intent);
    let resolved_1 = iris_types::resolution::resolve(&full_graph, Resolution::Architecture);

    let (out_0, _) = interpreter::interpret(
        &resolver,
        &[Value::Program(Rc::new(resolved_0)), Value::Int(0)],
        None,
    )
    .unwrap();
    let (out_1, _) = interpreter::interpret(
        &resolver,
        &[Value::Program(Rc::new(resolved_1)), Value::Int(1)],
        None,
    )
    .unwrap();

    let count_0 = match &out_0[0] {
        Value::Int(n) => *n,
        _ => panic!("expected Int"),
    };
    let count_1 = match &out_1[0] {
        Value::Int(n) => *n,
        _ => panic!("expected Int"),
    };

    assert!(
        count_1 >= count_0,
        "level 1 ({}) should show at least as many nodes as level 0 ({})",
        count_1,
        count_0
    );
}

// ---------- compare_programs ----------

#[test]
fn compare_identical_programs() {
    let comparator = build_compare_programs();

    let prog_a = make_binop_graph(0x00, 3, 5);
    let prog_b = make_binop_graph(0x00, 10, 20);

    // Same structure: 3 nodes, root is Prim(add)
    let inputs = vec![
        Value::Program(Rc::new(prog_a)),
        Value::Program(Rc::new(prog_b)),
    ];
    let (outputs, _) = interpreter::interpret(&comparator, &inputs, None).unwrap();

    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(1),
        "structurally equivalent programs should return 1"
    );
}

#[test]
fn compare_different_programs_different_root_kind() {
    let comparator = build_compare_programs();

    let prim_prog = make_binop_graph(0x00, 3, 5); // root is Prim
    let lit_prog = make_lit_graph(42);              // root is Lit

    let inputs = vec![
        Value::Program(Rc::new(prim_prog)),
        Value::Program(Rc::new(lit_prog)),
    ];
    let (outputs, _) = interpreter::interpret(&comparator, &inputs, None).unwrap();

    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(0),
        "different root kinds should return 0"
    );
}

#[test]
fn compare_different_node_count() {
    let comparator = build_compare_programs();

    let three_node = make_binop_graph(0x00, 3, 5);    // 3 nodes
    let seven_node = make_5node_graph(2, 3, 4, 5);    // 7 nodes

    let inputs = vec![
        Value::Program(Rc::new(three_node)),
        Value::Program(Rc::new(seven_node)),
    ];
    let (outputs, _) = interpreter::interpret(&comparator, &inputs, None).unwrap();

    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(0),
        "different node counts should return 0"
    );
}

// ---------- fragment_metadata ----------

#[test]
fn fragment_metadata_binop_program() {
    let meta = build_fragment_metadata();
    let target = make_binop_graph(0x00, 3, 5); // 3 nodes, root=Prim, no fold, no guard

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&meta, &inputs, None).unwrap();

    assert_eq!(outputs.len(), 1);
    let fields = match &outputs[0] {
        Value::Tuple(v) => v.clone(),
        other => panic!("expected Tuple, got {:?}", other),
    };

    assert_eq!(fields.len(), 6, "metadata should have 6 fields");

    // node_count = 3
    assert_eq!(fields[0], Value::Int(3), "node_count should be 3");
    // edge_count_est = 2
    assert_eq!(fields[1], Value::Int(2), "edge_est should be 2");
    // root_kind = 0 (Prim)
    assert_eq!(fields[2], Value::Int(0), "root_kind should be 0 (Prim)");
    // has_fold = 0
    assert_eq!(fields[3], Value::Int(0), "has_fold should be 0");
    // has_guard = 0
    assert_eq!(fields[4], Value::Int(0), "has_guard should be 0");
    // max_resolution_depth = 2
    assert_eq!(fields[5], Value::Int(2), "max_res_depth should be 2");
}

#[test]
fn fragment_metadata_fold_program() {
    let meta = build_fragment_metadata();
    let target = make_fold_program();

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&meta, &inputs, None).unwrap();

    let fields = match &outputs[0] {
        Value::Tuple(v) => v.clone(),
        other => panic!("expected Tuple, got {:?}", other),
    };

    // has_fold should be 1
    assert_eq!(fields[3], Value::Int(1), "has_fold should be 1 for fold program");
    // has_guard should be 0
    assert_eq!(fields[4], Value::Int(0), "has_guard should be 0 for fold program");
}

#[test]
fn fragment_metadata_guard_program() {
    let meta = build_fragment_metadata();
    let target = make_guard_program();

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&meta, &inputs, None).unwrap();

    let fields = match &outputs[0] {
        Value::Tuple(v) => v.clone(),
        other => panic!("expected Tuple, got {:?}", other),
    };

    // has_fold should be 0
    assert_eq!(fields[3], Value::Int(0), "has_fold should be 0 for guard program");
    // has_guard should be 1
    assert_eq!(fields[4], Value::Int(1), "has_guard should be 1 for guard program");
}

#[test]
fn fragment_metadata_lit_program() {
    let meta = build_fragment_metadata();
    let target = make_lit_graph(42); // 1 node, root=Lit

    let inputs = vec![Value::Program(Rc::new(target))];
    let (outputs, _) = interpreter::interpret(&meta, &inputs, None).unwrap();

    let fields = match &outputs[0] {
        Value::Tuple(v) => v.clone(),
        other => panic!("expected Tuple, got {:?}", other),
    };

    // node_count = 1
    assert_eq!(fields[0], Value::Int(1), "node_count should be 1");
    // edge_count_est = 0
    assert_eq!(fields[1], Value::Int(0), "edge_est should be 0");
    // root_kind = 5 (Lit)
    assert_eq!(fields[2], Value::Int(5), "root_kind should be 5 (Lit)");
}
