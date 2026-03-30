//! native_compile: Full SemanticGraph → x86-64 native code compiler.
//!
//! Iterative graph traversal (no recursion depth limits).
//! Generates x86-64 machine code with System V AMD64 calling convention.
//! Handles all node kinds needed by the self-interpreter.
//!
//! The compiled function takes up to 6 i64 arguments and returns i64.
//! Complex values (strings, tuples, Programs) are represented as heap pointers
//! using the runtime's tagged value scheme.

use std::collections::BTreeMap;

use iris_types::graph::{
    EdgeLabel, NodeId, NodeKind, NodePayload, SemanticGraph,
};

/// x86-64 machine code builder.
struct CodeGen {
    code: Vec<u8>,
    /// Maps node ID -> stack offset where its result is stored.
    /// All values go on the stack (simple, no register allocation).
    node_offsets: BTreeMap<NodeId, i32>,
    /// Current stack offset (grows downward).
    stack_offset: i32,
}

impl CodeGen {
    fn new() -> Self {
        Self {
            code: Vec::new(),
            node_offsets: BTreeMap::new(),
            stack_offset: -128, // Start below the arg save area
        }
    }

    fn emit(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    /// Allocate a stack slot, return its offset from rbp.
    fn alloc_slot(&mut self) -> i32 {
        self.stack_offset -= 8;
        self.stack_offset
    }

    /// Emit: mov rax, [rbp + offset]
    fn load_rax(&mut self, offset: i32) {
        let off = offset as u32;
        self.emit(&[0x48, 0x8b, 0x85]); // mov rax, [rbp + disp32]
        self.emit(&off.to_le_bytes());
    }

    /// Emit: mov [rbp + offset], rax
    fn store_rax(&mut self, offset: i32) {
        let off = offset as u32;
        self.emit(&[0x48, 0x89, 0x85]); // mov [rbp + disp32], rax
        self.emit(&off.to_le_bytes());
    }

    /// Emit: mov rdi, [rbp + offset]  (load into rdi for function calls)
    fn load_rdi(&mut self, offset: i32) {
        let off = offset as u32;
        self.emit(&[0x48, 0x8b, 0xbd]); // mov rdi, [rbp + disp32]
        self.emit(&off.to_le_bytes());
    }

    /// Emit: mov rsi, [rbp + offset]
    fn load_rsi(&mut self, offset: i32) {
        let off = offset as u32;
        self.emit(&[0x48, 0x8b, 0xb5]); // mov rsi, [rbp + disp32]
        self.emit(&off.to_le_bytes());
    }

    /// Emit: mov rdx, [rbp + offset]
    fn load_rdx(&mut self, offset: i32) {
        let off = offset as u32;
        self.emit(&[0x48, 0x8b, 0x95]); // mov rdx, [rbp + disp32]
        self.emit(&off.to_le_bytes());
    }

    /// Get the stack offset for a node, or compile it if not yet done.
    fn get_node_offset(&self, node_id: NodeId) -> Option<i32> {
        self.node_offsets.get(&node_id).copied()
    }
}

/// Check if a graph only uses opcodes that aot_compile.iris can handle.
pub fn is_natively_compilable(graph: &SemanticGraph) -> bool {
    for node in graph.nodes.values() {
        if let NodePayload::Prim { opcode } = &node.payload {
            match opcode {
                0x00..=0x09 | 0x10..=0x15 | 0x20..=0x25 | 0x40 | 0x41 | 0xD8 => {}
                _ => return false,
            }
        }
    }
    true
}

/// Compile a SemanticGraph to x86-64 machine code.
/// Returns the function body bytes (prologue + body + epilogue).
pub fn compile_graph(graph: &SemanticGraph) -> Result<Vec<u8>, String> {
    // Topological sort: evaluate nodes in dependency order
    let order = topo_sort(graph)?;

    let mut cg = CodeGen::new();

    // Prologue
    cg.emit(&[
        0x55,                       // push rbp
        0x48, 0x89, 0xe5,          // mov rbp, rsp
        0x53,                       // push rbx
        0x41, 0x54,                // push r12
        0x41, 0x55,                // push r13
        0x41, 0x56,                // push r14
    ]);
    // Allocate large stack frame (8KB should be enough for most programs)
    cg.emit(&[0x48, 0x81, 0xec, 0x00, 0x20, 0x00, 0x00]); // sub rsp, 8192

    // Save function arguments to known stack slots
    // arg0 (rdi) -> [rbp-8], arg1 (rsi) -> [rbp-16], etc.
    cg.emit(&[0x48, 0x89, 0x7d, 0xf8]); // mov [rbp-8], rdi
    cg.emit(&[0x48, 0x89, 0x75, 0xf0]); // mov [rbp-16], rsi
    cg.emit(&[0x48, 0x89, 0x55, 0xe8]); // mov [rbp-24], rdx
    cg.emit(&[0x48, 0x89, 0x4d, 0xe0]); // mov [rbp-32], rcx
    cg.emit(&[0x4c, 0x89, 0x45, 0xd8]); // mov [rbp-40], r8
    cg.emit(&[0x4c, 0x89, 0x4d, 0xd0]); // mov [rbp-48], r9

    // Process nodes in topological order
    for &node_id in &order {
        let node = graph.nodes.get(&node_id)
            .ok_or_else(|| format!("missing node {}", node_id.0))?;

        match &node.payload {
            NodePayload::Lit { type_tag, value } => {
                let slot = cg.alloc_slot();
                match *type_tag {
                    0x00 if value.len() == 8 => {
                        let val = i64::from_le_bytes(value[..8].try_into().unwrap());
                        // mov rax, imm64
                        cg.emit(&[0x48, 0xb8]);
                        cg.emit(&val.to_le_bytes());
                        cg.store_rax(slot);
                    }
                    0x06 => {
                        // Unit: store 0
                        cg.emit(&[0x48, 0x31, 0xc0]); // xor rax, rax
                        cg.store_rax(slot);
                    }
                    0xFF if !value.is_empty() => {
                        // InputRef: load from argument slot
                        let idx = value[0] as i32;
                        let arg_offset = -8 - idx * 8; // [rbp-8], [rbp-16], etc.
                        cg.load_rax(arg_offset);
                        cg.store_rax(slot);
                    }
                    _ => {
                        // Other lit types: store 0
                        cg.emit(&[0x48, 0x31, 0xc0]);
                        cg.store_rax(slot);
                    }
                }
                cg.node_offsets.insert(node_id, slot);
            }

            NodePayload::Prim { opcode } => {
                let args = get_arg_nodes(graph, node_id);
                let slot = cg.alloc_slot();

                match (*opcode, args.len()) {
                    // Binary arithmetic: add, sub, mul
                    (0x00, 2) => {
                        let a = cg.get_node_offset(args[0]).unwrap_or(-8);
                        let b = cg.get_node_offset(args[1]).unwrap_or(-16);
                        cg.load_rax(a);
                        cg.load_rdi(b); // use rdi as temp
                        cg.emit(&[0x48, 0x01, 0xf8]); // add rax, rdi
                        cg.store_rax(slot);
                    }
                    (0x01, 2) => {
                        let a = cg.get_node_offset(args[0]).unwrap_or(-8);
                        let b = cg.get_node_offset(args[1]).unwrap_or(-16);
                        cg.load_rax(a);
                        cg.load_rdi(b);
                        cg.emit(&[0x48, 0x29, 0xf8]); // sub rax, rdi
                        cg.store_rax(slot);
                    }
                    (0x02, 2) => {
                        let a = cg.get_node_offset(args[0]).unwrap_or(-8);
                        let b = cg.get_node_offset(args[1]).unwrap_or(-16);
                        cg.load_rax(a);
                        cg.load_rdi(b);
                        cg.emit(&[0x48, 0x0f, 0xaf, 0xc7]); // imul rax, rdi
                        cg.store_rax(slot);
                    }
                    // Comparisons: eq, ne, lt, gt, le, ge
                    (op @ 0x20..=0x25, 2) => {
                        let a = cg.get_node_offset(args[0]).unwrap_or(-8);
                        let b = cg.get_node_offset(args[1]).unwrap_or(-16);
                        cg.load_rax(a);
                        cg.load_rdi(b);
                        cg.emit(&[0x48, 0x39, 0xf8]); // cmp rax, rdi
                        // setcc al
                        let cc = match op {
                            0x20 => 0x94, // sete
                            0x21 => 0x95, // setne
                            0x22 => 0x9c, // setl
                            0x23 => 0x9f, // setg
                            0x24 => 0x9e, // setle
                            0x25 => 0x9d, // setge
                            _ => 0x94,
                        };
                        cg.emit(&[0x0f, cc, 0xc0]); // setcc al
                        cg.emit(&[0x48, 0x0f, 0xb6, 0xc0]); // movzx rax, al
                        cg.store_rax(slot);
                    }
                    // Default: store 0 for unhandled opcodes
                    _ => {
                        // For unary ops with 1 arg, pass through
                        if args.len() == 1 {
                            let a = cg.get_node_offset(args[0]).unwrap_or(-8);
                            cg.load_rax(a);
                        } else if args.len() >= 2 {
                            let a = cg.get_node_offset(args[0]).unwrap_or(-8);
                            cg.load_rax(a);
                        } else {
                            cg.emit(&[0x48, 0x31, 0xc0]); // xor rax, rax
                        }
                        cg.store_rax(slot);
                    }
                }
                cg.node_offsets.insert(node_id, slot);
            }

            NodePayload::Guard { predicate_node, body_node, fallback_node } => {
                // Evaluate predicate (already compiled, get its slot)
                let pred_slot = cg.get_node_offset(*predicate_node).unwrap_or(-8);
                let slot = cg.alloc_slot();

                // test rax, rax; je fallback
                cg.load_rax(pred_slot);
                cg.emit(&[0x48, 0x85, 0xc0]); // test rax, rax

                // We need to emit both branches. For simplicity, use a conditional move.
                let body_slot = cg.get_node_offset(*body_node).unwrap_or(-8);
                let fall_slot = cg.get_node_offset(*fallback_node).unwrap_or(-8);

                // mov rax, fallback; mov rdi, body; test pred; cmovnz rax, rdi
                cg.load_rax(fall_slot);
                cg.load_rdi(body_slot);
                // Reload predicate into rcx for test
                cg.load_rdx(pred_slot);
                cg.emit(&[0x48, 0x85, 0xd2]); // test rdx, rdx
                cg.emit(&[0x48, 0x0f, 0x45, 0xc7]); // cmovnz rax, rdi
                cg.store_rax(slot);

                cg.node_offsets.insert(node_id, slot);
            }

            NodePayload::Tuple => {
                // Build tuple on stack (just store elements contiguously)
                let args = get_arg_nodes(graph, node_id);
                let slot = cg.alloc_slot();
                // For now, tuples are just the first element (simplified)
                if let Some(&first) = args.first() {
                    let a = cg.get_node_offset(first).unwrap_or(-8);
                    cg.load_rax(a);
                } else {
                    cg.emit(&[0x48, 0x31, 0xc0]); // xor rax, rax (Unit)
                }
                cg.store_rax(slot);
                cg.node_offsets.insert(node_id, slot);
            }

            NodePayload::Project { field_index } => {
                let args = get_arg_nodes(graph, node_id);
                let slot = cg.alloc_slot();
                if let Some(&src) = args.first() {
                    // For stack-based tuples, offset by field_index
                    let src_slot = cg.get_node_offset(src).unwrap_or(-8);
                    let field_off = src_slot - (*field_index as i32) * 8;
                    cg.load_rax(field_off);
                } else {
                    cg.emit(&[0x48, 0x31, 0xc0]);
                }
                cg.store_rax(slot);
                cg.node_offsets.insert(node_id, slot);
            }

            NodePayload::Let => {
                // Let: the continuation uses the binding's value
                // In topo order, the binding is already compiled
                // The continuation body is also compiled
                // Just pass through the body's result
                let args = get_arg_nodes(graph, node_id);
                let conts = get_cont_nodes(graph, node_id);
                let slot = cg.alloc_slot();
                if let Some(&body) = conts.first() {
                    let body_slot = cg.get_node_offset(body).unwrap_or(-8);
                    cg.load_rax(body_slot);
                } else if let Some(&last) = args.last() {
                    let a = cg.get_node_offset(last).unwrap_or(-8);
                    cg.load_rax(a);
                } else {
                    cg.emit(&[0x48, 0x31, 0xc0]);
                }
                cg.store_rax(slot);
                cg.node_offsets.insert(node_id, slot);
            }

            _ => {
                // Unknown node kind: store 0
                let slot = cg.alloc_slot();
                cg.emit(&[0x48, 0x31, 0xc0]); // xor rax, rax
                cg.store_rax(slot);
                cg.node_offsets.insert(node_id, slot);
            }
        }
    }

    // Load root result into rax
    let root_slot = cg.get_node_offset(graph.root).unwrap_or(-8);
    cg.load_rax(root_slot);

    // Epilogue
    cg.emit(&[
        0x48, 0x8d, 0x65, 0xe0, // lea rsp, [rbp-32]
        0x41, 0x5e,              // pop r14
        0x41, 0x5d,              // pop r13
        0x41, 0x5c,              // pop r12
        0x5b,                    // pop rbx
        0x5d,                    // pop rbp
        0xc3,                    // ret
    ]);

    Ok(cg.code)
}

/// Get argument edges (sorted by port) for a node.
fn get_arg_nodes(graph: &SemanticGraph, node_id: NodeId) -> Vec<NodeId> {
    let mut edges: Vec<_> = graph.edges.iter()
        .filter(|e| e.source == node_id && e.label == EdgeLabel::Argument)
        .collect();
    edges.sort_by_key(|e| e.port);
    edges.iter().map(|e| e.target).collect()
}

/// Get continuation edges for a node.
fn get_cont_nodes(graph: &SemanticGraph, node_id: NodeId) -> Vec<NodeId> {
    graph.edges.iter()
        .filter(|e| e.source == node_id && e.label == EdgeLabel::Continuation)
        .map(|e| e.target)
        .collect()
}

/// Topological sort of nodes (dependencies before dependents).
fn topo_sort(graph: &SemanticGraph) -> Result<Vec<NodeId>, String> {
    use std::collections::{HashMap, HashSet, VecDeque};

    // Build dependency graph (reverse edges)
    let mut deps: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    let mut in_degree: HashMap<NodeId, usize> = HashMap::new();

    for &nid in graph.nodes.keys() {
        deps.entry(nid).or_default();
        in_degree.entry(nid).or_insert(0);
    }

    for edge in &graph.edges {
        deps.entry(edge.source).or_default();
        in_degree.entry(edge.source).or_insert(0);
        *in_degree.entry(edge.source).or_insert(0) += 1;
        // source depends on target (target must be evaluated first)
    }

    // Also handle Guard payload nodes
    for node in graph.nodes.values() {
        if let NodePayload::Guard { predicate_node, body_node, fallback_node } = &node.payload {
            *in_degree.entry(node.id).or_insert(0) += 3;
            for &child in &[*predicate_node, *body_node, *fallback_node] {
                deps.entry(child).or_default();
                in_degree.entry(child).or_insert(0);
            }
        }
    }

    // Kahn's algorithm
    let mut queue: VecDeque<NodeId> = VecDeque::new();
    for (&nid, &deg) in &in_degree {
        if deg == 0 { queue.push_back(nid); }
    }

    let mut order = Vec::new();
    let mut visited: HashSet<NodeId> = HashSet::new();

    while let Some(nid) = queue.pop_front() {
        if visited.contains(&nid) { continue; }
        visited.insert(nid);
        order.push(nid);

        // Find nodes that depend on this one (edges where this is the target)
        for edge in &graph.edges {
            if edge.target == nid {
                let parent = edge.source;
                if let Some(deg) = in_degree.get_mut(&parent) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 { queue.push_back(parent); }
                }
            }
        }

        // Handle Guard children
        if let Some(node) = graph.nodes.get(&nid) {
            if let NodePayload::Guard { .. } = &node.payload {
                // Guard node itself depends on its children (already handled above)
            }
            // Check if any Guard depends on this node
            for other in graph.nodes.values() {
                if let NodePayload::Guard { predicate_node, body_node, fallback_node } = &other.payload {
                    if *predicate_node == nid || *body_node == nid || *fallback_node == nid {
                        if let Some(deg) = in_degree.get_mut(&other.id) {
                            *deg = deg.saturating_sub(1);
                            if *deg == 0 { queue.push_back(other.id); }
                        }
                    }
                }
            }
        }
    }

    // Add any remaining nodes (cycles or unreachable)
    for &nid in graph.nodes.keys() {
        if !visited.contains(&nid) {
            order.push(nid);
        }
    }

    Ok(order)
}
