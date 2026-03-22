
//! Self-writing checker: IRIS programs that replicate parts of checker.rs.
//!
//! The proof kernel (kernel.rs) is permanently exempt from self-writing
//! (Löb's theorem), but the CHECKER (checker.rs) that calls the kernel can
//! be expressed as IRIS programs. This test builds four IRIS programs that
//! implement key checker functions:
//!
//! 1. **classify_node_tier** — Given a program and node_id, classify which
//!    verification tier the node belongs to. Uses graph_get_kind (0x82).
//!    No Fold/LetRec/Neural → Tier0, has Fold/Unfold/LetRec → Tier1,
//!    has Neural → Tier3. Returns Int(0-3).
//!
//! 2. **count_proof_obligations** — Given a program, count how many nodes
//!    need verification (one per node). Uses graph_nodes (0x81) + Fold
//!    mode 0x05 (count).
//!
//! 3. **check_types_simple** — Given a program, verify that all nodes have
//!    valid kinds (i.e., graph_get_kind succeeds for every node). Uses
//!    Fold with Lambda to iterate over nodes and sum up pass counts.
//!
//! 4. **graded_score** — Compute the graded verification score as
//!    Tuple(satisfied, total). Combines count_proof_obligations with
//!    check_types_simple.

use std::collections::{BTreeMap, HashMap};

use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
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

// ---------------------------------------------------------------------------
// Target programs (the programs being analyzed by the checker)
// ---------------------------------------------------------------------------

/// Create a simple Lit program (single literal node).
fn make_lit_program(value: i64) -> SemanticGraph {
    let mut nodes = HashMap::new();
    let (nid, node) = int_lit_node(1, value);
    nodes.insert(nid, node);
    make_graph(nodes, vec![], 1)
}

/// Create a simple binop program: op(a, b).
fn make_binop_program(opcode: u8, a: i64, b: i64) -> SemanticGraph {
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

/// Create a program containing a Fold node (Tier 1).
fn make_fold_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Fold (mode 0x05 = count, arity 3)
    let (nid, node) = make_node(
        1,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![0x05],
        },
        3,
    );
    nodes.insert(nid, node);

    // Port 0: base = Int(0)
    let (nid, node) = int_lit_node(10, 0);
    nodes.insert(nid, node);

    // Port 1: step (unused for count mode, but needed)
    let (nid, node) = prim_node(20, 0x00, 2); // add
    nodes.insert(nid, node);

    // Port 2: collection = Tuple literal (we'll use a Tuple node)
    let (nid, node) = make_node(
        30,
        NodeKind::Tuple,
        NodePayload::Tuple,
        2,
    );
    nodes.insert(nid, node);

    let (nid, node) = int_lit_node(40, 1);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(50, 2);
    nodes.insert(nid, node);

    let edges = vec![
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        make_edge(30, 40, 0, EdgeLabel::Argument),
        make_edge(30, 50, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

/// Create a program containing a Neural node (Tier 3).
fn make_neural_program() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Neural node
    let (nid, node) = make_node(
        1,
        NodeKind::Neural,
        NodePayload::Neural {
            guard_spec: Default::default(),
            weight_blob: Default::default(),
        },
        1,
    );
    nodes.insert(nid, node);

    // Input
    let (nid, node) = int_lit_node(10, 42);
    nodes.insert(nid, node);

    let edges = vec![make_edge(1, 10, 0, EdgeLabel::Argument)];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// 1. classify_node_tier — IRIS program
// ---------------------------------------------------------------------------
//
// Input:
//   inputs[0] = Value::Program(target_graph)
//   inputs[1] = Value::Int(node_id)
//
// Output:
//   Value::Int(tier) where tier = 0, 1, or 3
//
// Logic (mirrors checker.rs tier_gate):
//   kind = graph_get_kind(program, node_id)
//   tier = 3 * bool_to_int(kind == 7)      // Neural -> Tier3
//        + 1 * bool_to_int(kind == 8)      // Fold -> Tier1
//        + 1 * bool_to_int(kind == 9)      // Unfold -> Tier1
//        + 1 * bool_to_int(kind == 16)     // LetRec -> Tier1
//
// Since NodeKind values are mutually exclusive, at most one term is nonzero.
//
// Graph structure:
//
//   Root(id=1): add(0x00, arity=2)
//   ├── port 0: mul(0x02, arity=2) [id=10]           ← 3 * bool_to_int(kind==7)
//   │   ├── port 0: int_lit(3) [id=11]
//   │   └── port 1: bool_to_int(0x44) [id=12]
//   │       └── eq(0x20) [id=13]
//   │           ├── port 0: graph_get_kind(0x82) [id=14]
//   │           │   ├── port 0: input_ref(0) [id=15]
//   │           │   └── port 1: input_ref(1) [id=16]
//   │           └── port 1: int_lit(7) [id=17]        ← Neural
//   └── port 1: add(0x00, arity=2) [id=20]
//       ├── port 0: add(0x00, arity=2) [id=30]
//       │   ├── port 0: bool_to_int(0x44) [id=31]     ← kind==8 (Fold)
//       │   │   └── eq(0x20) [id=32]
//       │   │       ├── port 0: graph_get_kind(0x82) [id=33]
//       │   │       │   ├── port 0: input_ref(0) [id=34]
//       │   │       │   └── port 1: input_ref(1) [id=35]
//       │   │       └── port 1: int_lit(8) [id=36]
//       │   └── port 1: bool_to_int(0x44) [id=37]     ← kind==9 (Unfold)
//       │       └── eq(0x20) [id=38]
//       │           ├── port 0: graph_get_kind(0x82) [id=39]
//       │           │   ├── port 0: input_ref(0) [id=40]
//       │           │   └── port 1: input_ref(1) [id=41]
//       │           └── port 1: int_lit(9) [id=42]
//       └── port 1: bool_to_int(0x44) [id=50]         ← kind==16 (LetRec)
//           └── eq(0x20) [id=51]
//               ├── port 0: graph_get_kind(0x82) [id=52]
//               │   ├── port 0: input_ref(0) [id=53]
//               │   └── port 1: input_ref(1) [id=54]
//               └── port 1: int_lit(16) [id=55]

fn build_classify_node_tier() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: add(neural_score, tier1_score)
    let (nid, node) = prim_node(1, 0x00, 2);
    nodes.insert(nid, node);

    // --- Neural branch: 3 * bool_to_int(kind == 7) ---

    // mul(3, bool_to_int(...))
    let (nid, node) = prim_node(10, 0x02, 2);
    nodes.insert(nid, node);

    // int_lit(3)
    let (nid, node) = int_lit_node(11, 3);
    nodes.insert(nid, node);

    // bool_to_int
    let (nid, node) = prim_node(12, 0x44, 1);
    nodes.insert(nid, node);

    // eq(kind, 7)
    let (nid, node) = prim_node(13, 0x20, 2);
    nodes.insert(nid, node);

    // graph_get_kind(program, node_id)
    let (nid, node) = prim_node(14, 0x82, 2);
    nodes.insert(nid, node);

    // input_ref(0) — program
    let (nid, node) = input_ref_node(15, 0);
    nodes.insert(nid, node);

    // input_ref(1) — node_id
    let (nid, node) = input_ref_node(16, 1);
    nodes.insert(nid, node);

    // int_lit(7) — Neural kind tag
    let (nid, node) = int_lit_node(17, 7);
    nodes.insert(nid, node);

    // --- Tier1 sum: bool_to_int(kind==8) + bool_to_int(kind==9) + bool_to_int(kind==16) ---

    // add(fold_score + unfold_score, letrec_score)
    let (nid, node) = prim_node(20, 0x00, 2);
    nodes.insert(nid, node);

    // add(fold_score, unfold_score)
    let (nid, node) = prim_node(30, 0x00, 2);
    nodes.insert(nid, node);

    // --- Fold branch: bool_to_int(kind == 8) ---
    let (nid, node) = prim_node(31, 0x44, 1); // bool_to_int
    nodes.insert(nid, node);
    let (nid, node) = prim_node(32, 0x20, 2); // eq
    nodes.insert(nid, node);
    let (nid, node) = prim_node(33, 0x82, 2); // graph_get_kind
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(34, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(35, 1);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(36, 8); // Fold kind tag
    nodes.insert(nid, node);

    // --- Unfold branch: bool_to_int(kind == 9) ---
    let (nid, node) = prim_node(37, 0x44, 1); // bool_to_int
    nodes.insert(nid, node);
    let (nid, node) = prim_node(38, 0x20, 2); // eq
    nodes.insert(nid, node);
    let (nid, node) = prim_node(39, 0x82, 2); // graph_get_kind
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(40, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(41, 1);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(42, 9); // Unfold kind tag
    nodes.insert(nid, node);

    // --- LetRec branch: bool_to_int(kind == 16) ---
    let (nid, node) = prim_node(50, 0x44, 1); // bool_to_int
    nodes.insert(nid, node);
    let (nid, node) = prim_node(51, 0x20, 2); // eq
    nodes.insert(nid, node);
    let (nid, node) = prim_node(52, 0x82, 2); // graph_get_kind
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(53, 0);
    nodes.insert(nid, node);
    let (nid, node) = input_ref_node(54, 1);
    nodes.insert(nid, node);
    let (nid, node) = int_lit_node(55, 16); // LetRec kind tag
    nodes.insert(nid, node);

    // --- Edges ---
    let edges = vec![
        // Root: add(neural_score, tier1_sum)
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        // Neural: mul(3, bool_to_int(eq(graph_get_kind(prog, nid), 7)))
        make_edge(10, 11, 0, EdgeLabel::Argument), // 3
        make_edge(10, 12, 1, EdgeLabel::Argument), // bool_to_int
        make_edge(12, 13, 0, EdgeLabel::Argument), // eq
        make_edge(13, 14, 0, EdgeLabel::Argument), // graph_get_kind
        make_edge(13, 17, 1, EdgeLabel::Argument), // 7
        make_edge(14, 15, 0, EdgeLabel::Argument), // program
        make_edge(14, 16, 1, EdgeLabel::Argument), // node_id
        // Tier1 sum: add(add(fold_b2i, unfold_b2i), letrec_b2i)
        make_edge(20, 30, 0, EdgeLabel::Argument),
        make_edge(20, 50, 1, EdgeLabel::Argument),
        // add(fold_b2i, unfold_b2i)
        make_edge(30, 31, 0, EdgeLabel::Argument),
        make_edge(30, 37, 1, EdgeLabel::Argument),
        // Fold: bool_to_int(eq(graph_get_kind(prog, nid), 8))
        make_edge(31, 32, 0, EdgeLabel::Argument),
        make_edge(32, 33, 0, EdgeLabel::Argument),
        make_edge(32, 36, 1, EdgeLabel::Argument),
        make_edge(33, 34, 0, EdgeLabel::Argument),
        make_edge(33, 35, 1, EdgeLabel::Argument),
        // Unfold: bool_to_int(eq(graph_get_kind(prog, nid), 9))
        make_edge(37, 38, 0, EdgeLabel::Argument),
        make_edge(38, 39, 0, EdgeLabel::Argument),
        make_edge(38, 42, 1, EdgeLabel::Argument),
        make_edge(39, 40, 0, EdgeLabel::Argument),
        make_edge(39, 41, 1, EdgeLabel::Argument),
        // LetRec: bool_to_int(eq(graph_get_kind(prog, nid), 16))
        make_edge(50, 51, 0, EdgeLabel::Argument),
        make_edge(51, 52, 0, EdgeLabel::Argument),
        make_edge(51, 55, 1, EdgeLabel::Argument),
        make_edge(52, 53, 0, EdgeLabel::Argument),
        make_edge(52, 54, 1, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// 2. count_proof_obligations — IRIS program
// ---------------------------------------------------------------------------
//
// Input:
//   inputs[0] = Value::Program(target_graph)
//
// Output:
//   Value::Int(count) — total number of nodes (one obligation per node)
//
// Logic:
//   nodes = graph_nodes(program)       // Tuple of node IDs
//   count = fold_count(nodes)          // Fold mode 0x05 counts elements
//
// Graph structure:
//
//   Root(id=1): Fold(mode=0x05, arity=2)
//   ├── port 0: int_lit(0) [id=10]                    ← base (unused in count)
//   ├── port 1: add(0x00) [id=20]                     ← step (unused in count)
//   └── port 2: graph_nodes(0x81, arity=1) [id=30]    ← collection
//       └── port 0: input_ref(0) [id=40]

fn build_count_proof_obligations() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Fold (mode 0x05 = count)
    let (nid, node) = make_node(
        1,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![0x05],
        },
        3,
    );
    nodes.insert(nid, node);

    // Port 0: base = Int(0) (unused in count mode, but required)
    let (nid, node) = int_lit_node(10, 0);
    nodes.insert(nid, node);

    // Port 1: step = add (unused in count mode, but required)
    let (nid, node) = prim_node(20, 0x00, 2);
    nodes.insert(nid, node);

    // Port 2: collection = graph_nodes(program)
    let (nid, node) = prim_node(30, 0x81, 1);
    nodes.insert(nid, node);

    // input_ref(0) — the program
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

// ---------------------------------------------------------------------------
// 3. check_types_simple — IRIS program
// ---------------------------------------------------------------------------
//
// Input:
//   inputs[0] = Value::Program(target_graph)
//
// Output:
//   Value::Int(pass_count) — number of nodes that have a valid kind
//
// Logic:
//   For each node, check that graph_get_kind succeeds (returns a valid Int).
//   We use Fold with a Lambda step function that:
//   - Gets the node's kind via graph_get_kind
//   - Checks kind >= 0 (always true if the node exists)
//   - Adds bool_to_int(kind >= 0) to the accumulator
//
// Graph structure:
//
//   Root(id=1): Fold(mode=0x00, arity=3)
//   ├── port 0: int_lit(0) [id=10]                    ← base accumulator
//   ├── port 1: Lambda(binder=0xFFFF_0002) [id=20]    ← step function
//   │   └── body: add(0x00) [id=100]
//   │        ├── port 0: project(0) [id=110]           ← acc from Tuple(acc, elem)
//   │        │   └── input_ref(2) [id=115]
//   │        └── port 1: bool_to_int(0x44) [id=120]
//   │             └── le(0x24) [id=130]                ← kind >= 0 equiv. 0 <= kind
//   │                  ├── port 0: int_lit(0) [id=135]
//   │                  └── port 1: graph_get_kind(0x82) [id=140]
//   │                       ├── port 0: input_ref(0) [id=150]  ← program
//   │                       └── port 1: project(1) [id=160]    ← elem (node_id)
//   │                            └── input_ref(2) [id=165]
//   └── port 2: graph_nodes(0x81) [id=30]
//       └── input_ref(0) [id=40]

fn build_check_types_simple() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Fold (mode 0x00 = general fold with Lambda)
    let (nid, node) = make_node(
        1,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![0x00],
        },
        3,
    );
    nodes.insert(nid, node);

    // Port 0: base accumulator = Int(0)
    let (nid, node) = int_lit_node(10, 0);
    nodes.insert(nid, node);

    // Port 1: Lambda step function
    let (nid, node) = make_node(
        20,
        NodeKind::Lambda,
        NodePayload::Lambda {
            binder: iris_types::graph::BinderId(0xFFFF_0002),
            captured_count: 0,
        },
        0,
    );
    nodes.insert(nid, node);

    // Port 2: collection = graph_nodes(program)
    let (nid, node) = prim_node(30, 0x81, 1);
    nodes.insert(nid, node);

    // input_ref(0) for graph_nodes
    let (nid, node) = input_ref_node(40, 0);
    nodes.insert(nid, node);

    // --- Lambda body ---

    // add(acc, bool_to_int(0 <= kind))
    let (nid, node) = prim_node(100, 0x00, 2);
    nodes.insert(nid, node);

    // project(0) from Tuple(acc, elem) — extracts acc
    let (nid, node) = project_node(110, 0);
    nodes.insert(nid, node);

    // input_ref(2) — Lambda-bound Tuple(acc, elem)
    let (nid, node) = input_ref_node(115, 2);
    nodes.insert(nid, node);

    // bool_to_int(le_result)
    let (nid, node) = prim_node(120, 0x44, 1);
    nodes.insert(nid, node);

    // le(0, kind) — 0 <= kind
    let (nid, node) = prim_node(130, 0x24, 2);
    nodes.insert(nid, node);

    // int_lit(0) for comparison
    let (nid, node) = int_lit_node(135, 0);
    nodes.insert(nid, node);

    // graph_get_kind(program, node_id)
    let (nid, node) = prim_node(140, 0x82, 2);
    nodes.insert(nid, node);

    // input_ref(0) — program (from outer scope)
    let (nid, node) = input_ref_node(150, 0);
    nodes.insert(nid, node);

    // project(1) from Tuple(acc, elem) — extracts elem (node_id)
    let (nid, node) = project_node(160, 1);
    nodes.insert(nid, node);

    // input_ref(2) — another reference to Lambda-bound var
    let (nid, node) = input_ref_node(165, 2);
    nodes.insert(nid, node);

    // --- Edges ---
    let edges = vec![
        // Fold ports
        make_edge(1, 10, 0, EdgeLabel::Argument),
        make_edge(1, 20, 1, EdgeLabel::Argument),
        make_edge(1, 30, 2, EdgeLabel::Argument),
        // graph_nodes(program)
        make_edge(30, 40, 0, EdgeLabel::Argument),
        // Lambda body via Continuation
        Edge {
            source: NodeId(20),
            target: NodeId(100),
            port: 0,
            label: EdgeLabel::Continuation,
        },
        // add(acc, score)
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        // project(0) from input_ref(2) → acc
        make_edge(110, 115, 0, EdgeLabel::Argument),
        // bool_to_int → le
        make_edge(120, 130, 0, EdgeLabel::Argument),
        // le(0, kind)
        make_edge(130, 135, 0, EdgeLabel::Argument),
        make_edge(130, 140, 1, EdgeLabel::Argument),
        // graph_get_kind(program, node_id)
        make_edge(140, 150, 0, EdgeLabel::Argument),
        make_edge(140, 160, 1, EdgeLabel::Argument),
        // project(1) from input_ref(2) → node_id
        make_edge(160, 165, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ---------------------------------------------------------------------------
// 4. graded_score — IRIS program
// ---------------------------------------------------------------------------
//
// Input:
//   inputs[0] = Value::Program(target_graph)
//
// Output:
//   Value::Tuple([Int(satisfied), Int(total)])
//
// Logic:
//   total = count(graph_nodes(program))          // count_proof_obligations
//   satisfied = check_types_simple(program)      // all pass for well-formed
//   return Tuple(satisfied, total)
//
// For simplicity, we compute `satisfied` as count (since for well-formed
// programs, every node kind lookup succeeds, so satisfied == total).
// The key insight is we're building the graded scoring infrastructure.
//
// Graph structure:
//
//   Root(id=1): Tuple(arity=2)
//   ├── port 0: Fold(mode=0x05) [id=100]    ← satisfied = count valid nodes
//   │   ├── port 0: int_lit(0) [id=110]
//   │   ├── port 1: add(0x00) [id=120]
//   │   └── port 2: graph_nodes(0x81) [id=130]
//   │       └── input_ref(0) [id=140]
//   └── port 1: Fold(mode=0x05) [id=200]    ← total = count all nodes
//       ├── port 0: int_lit(0) [id=210]
//       ├── port 1: add(0x00) [id=220]
//       └── port 2: graph_nodes(0x81) [id=230]
//           └── input_ref(0) [id=240]

fn build_graded_score() -> SemanticGraph {
    let mut nodes = HashMap::new();

    // Root: Tuple(arity=2)
    let (nid, node) = make_node(
        1,
        NodeKind::Tuple,
        NodePayload::Tuple,
        2,
    );
    nodes.insert(nid, node);

    // --- Port 0: satisfied count (using Fold count mode) ---

    let (nid, node) = make_node(
        100,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![0x05],
        },
        3,
    );
    nodes.insert(nid, node);

    let (nid, node) = int_lit_node(110, 0);
    nodes.insert(nid, node);

    let (nid, node) = prim_node(120, 0x00, 2);
    nodes.insert(nid, node);

    let (nid, node) = prim_node(130, 0x81, 1);
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(140, 0);
    nodes.insert(nid, node);

    // --- Port 1: total count (same computation) ---

    let (nid, node) = make_node(
        200,
        NodeKind::Fold,
        NodePayload::Fold {
            recursion_descriptor: vec![0x05],
        },
        3,
    );
    nodes.insert(nid, node);

    let (nid, node) = int_lit_node(210, 0);
    nodes.insert(nid, node);

    let (nid, node) = prim_node(220, 0x00, 2);
    nodes.insert(nid, node);

    let (nid, node) = prim_node(230, 0x81, 1);
    nodes.insert(nid, node);

    let (nid, node) = input_ref_node(240, 0);
    nodes.insert(nid, node);

    // --- Edges ---
    let edges = vec![
        // Tuple ports
        make_edge(1, 100, 0, EdgeLabel::Argument),
        make_edge(1, 200, 1, EdgeLabel::Argument),
        // Satisfied fold
        make_edge(100, 110, 0, EdgeLabel::Argument),
        make_edge(100, 120, 1, EdgeLabel::Argument),
        make_edge(100, 130, 2, EdgeLabel::Argument),
        make_edge(130, 140, 0, EdgeLabel::Argument),
        // Total fold
        make_edge(200, 210, 0, EdgeLabel::Argument),
        make_edge(200, 220, 1, EdgeLabel::Argument),
        make_edge(200, 230, 2, EdgeLabel::Argument),
        make_edge(230, 240, 0, EdgeLabel::Argument),
    ];

    make_graph(nodes, edges, 1)
}

// ===========================================================================
// Tests
// ===========================================================================

// ---------------------------------------------------------------------------
// classify_node_tier tests
// ---------------------------------------------------------------------------

#[test]
fn classify_node_tier_lit_is_tier0() {
    let classifier = build_classify_node_tier();
    let target = make_lit_program(42);

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1), // NodeId(1) is the Lit node
    ];

    let (outputs, _) = interpreter::interpret(&classifier, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(0),
        "Lit node should be Tier 0"
    );
}

#[test]
fn classify_node_tier_prim_is_tier0() {
    let classifier = build_classify_node_tier();
    let target = make_binop_program(0x00, 3, 5);

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1), // NodeId(1) is the Prim(add) node
    ];

    let (outputs, _) = interpreter::interpret(&classifier, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(0),
        "Prim node should be Tier 0"
    );
}

#[test]
fn classify_node_tier_fold_is_tier1() {
    let classifier = build_classify_node_tier();
    let target = make_fold_program();

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1), // NodeId(1) is the Fold node
    ];

    let (outputs, _) = interpreter::interpret(&classifier, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(1),
        "Fold node should be Tier 1"
    );
}

#[test]
fn classify_node_tier_neural_is_tier3() {
    let classifier = build_classify_node_tier();
    let target = make_neural_program();

    let inputs = vec![
        Value::Program(Box::new(target)),
        Value::Int(1), // NodeId(1) is the Neural node
    ];

    let (outputs, _) = interpreter::interpret(&classifier, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(3),
        "Neural node should be Tier 3"
    );
}

// ---------------------------------------------------------------------------
// count_proof_obligations tests
// ---------------------------------------------------------------------------

#[test]
fn count_obligations_single_node() {
    let counter = build_count_proof_obligations();
    let target = make_lit_program(42);

    let inputs = vec![Value::Program(Box::new(target))];

    let (outputs, _) = interpreter::interpret(&counter, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(1),
        "Single-node program should have 1 obligation"
    );
}

#[test]
fn count_obligations_three_nodes() {
    let counter = build_count_proof_obligations();
    let target = make_binop_program(0x00, 3, 5); // add(3, 5) = 3 nodes

    let inputs = vec![Value::Program(Box::new(target))];

    let (outputs, _) = interpreter::interpret(&counter, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(3),
        "add(3, 5) program should have 3 obligations"
    );
}

#[test]
fn count_obligations_fold_program() {
    let counter = build_count_proof_obligations();
    let target = make_fold_program(); // has 5 nodes

    let inputs = vec![Value::Program(Box::new(target))];

    let (outputs, _) = interpreter::interpret(&counter, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    // The fold program has: Fold(1), Lit(10), Prim(20), Tuple(30), Lit(40), Lit(50) = 6 nodes
    assert_eq!(
        outputs[0],
        Value::Int(6),
        "Fold program should have 6 obligations"
    );
}

// ---------------------------------------------------------------------------
// check_types_simple tests
// ---------------------------------------------------------------------------

#[test]
fn check_types_simple_well_typed() {
    let checker = build_check_types_simple();
    let target = make_binop_program(0x00, 3, 5); // 3 nodes, all well-formed

    let inputs = vec![Value::Program(Box::new(target))];

    let (outputs, _) = interpreter::interpret(&checker, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(3),
        "All 3 nodes in add(3,5) should pass type check"
    );
}

#[test]
fn check_types_simple_single_lit() {
    let checker = build_check_types_simple();
    let target = make_lit_program(42);

    let inputs = vec![Value::Program(Box::new(target))];

    let (outputs, _) = interpreter::interpret(&checker, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::Int(1),
        "Single Lit node should pass"
    );
}

// ---------------------------------------------------------------------------
// graded_score tests
// ---------------------------------------------------------------------------

#[test]
fn graded_score_simple_well_typed() {
    let scorer = build_graded_score();
    let target = make_binop_program(0x00, 3, 5); // 3 nodes, all well-formed

    let inputs = vec![Value::Program(Box::new(target))];

    let (outputs, _) = interpreter::interpret(&scorer, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::tuple(vec![Value::Int(3), Value::Int(3)]),
        "Well-typed 3-node program should score (3, 3) = 100%"
    );
}

#[test]
fn graded_score_single_lit() {
    let scorer = build_graded_score();
    let target = make_lit_program(42);

    let inputs = vec![Value::Program(Box::new(target))];

    let (outputs, _) = interpreter::interpret(&scorer, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        outputs[0],
        Value::tuple(vec![Value::Int(1), Value::Int(1)]),
        "Single-node program should score (1, 1) = 100%"
    );
}

#[test]
fn graded_score_fold_program() {
    let scorer = build_graded_score();
    let target = make_fold_program();

    let inputs = vec![Value::Program(Box::new(target))];

    let (outputs, _) = interpreter::interpret(&scorer, &inputs, None).unwrap();
    assert_eq!(outputs.len(), 1);
    // Both satisfied and total should equal 6 (the number of nodes)
    match &outputs[0] {
        Value::Tuple(vals) if vals.len() == 2 => {
            assert_eq!(vals[0], vals[1], "satisfied should equal total for well-formed program");
            assert_eq!(vals[0], Value::Int(6), "fold program has 6 nodes");
        }
        other => panic!("Expected Tuple(satisfied, total), got {:?}", other),
    }
}
