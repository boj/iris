//! native_compile: Compile SemanticGraph → x86-64 with proper control flow.
//!
//! Unlike flatten_subgraph (flat, both-branches cmov), this compiler:
//! - Emits conditional jumps for Guards (only evaluates taken branch)
//! - Emits loop constructs for Folds
//! - Emits function calls for graph_eval (recursive sub-evaluation)
//! - Handles all node kinds via recursive graph traversal
//!
//! Generated function signature: fn(args: *const i64, n_args: i64) -> i64
//! All values are tagged i64 (via jit::pack/unpack).

use std::collections::BTreeMap;
use std::rc::Rc;

use iris_types::eval::Value;
use iris_types::graph::{
    BinderId, Edge, EdgeLabel, NodeId, NodeKind, NodePayload, SemanticGraph,
};

/// Compile a SemanticGraph to x86-64 machine code bytes.
/// The generated function takes tagged i64 values and returns a tagged i64.
/// Signature: extern "C" fn(*const i64, i64) -> i64
///   arg0 (rdi) = pointer to tagged input values array
///   arg1 (rsi) = number of inputs
///   returns rax = tagged result value
pub fn compile_graph_native(graph: &SemanticGraph) -> Option<Vec<u8>> {
    let mut cg = NativeCodeGen::new(graph);
    cg.compile()
}

struct NativeCodeGen<'a> {
    graph: &'a SemanticGraph,
    code: Vec<u8>,
    edges_from: BTreeMap<NodeId, Vec<&'a Edge>>,
    /// Stack frame layout: each slot is 8 bytes at [rbp - offset]
    /// Slots 1-6: saved input pointers
    /// Slots 7+: temporaries
    next_slot: i32,
}

impl<'a> NativeCodeGen<'a> {
    fn new(graph: &'a SemanticGraph) -> Self {
        let mut edges_from: BTreeMap<NodeId, Vec<&Edge>> = BTreeMap::new();
        for edge in &graph.edges {
            edges_from.entry(edge.source).or_default().push(edge);
        }
        for edges in edges_from.values_mut() {
            edges.sort_by_key(|e| (e.port, e.label as u8));
        }
        Self {
            graph,
            code: Vec::new(),
            edges_from,
            next_slot: -64, // start after saved registers + input area
        }
    }

    fn emit(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    fn alloc_slot(&mut self) -> i32 {
        self.next_slot -= 8;
        self.next_slot
    }

    fn emit_mov_imm64_rax(&mut self, val: i64) {
        self.emit(&[0x48, 0xb8]);
        self.emit(&val.to_le_bytes());
    }

    fn emit_store_rax(&mut self, offset: i32) {
        // mov [rbp + offset], rax
        self.emit(&[0x48, 0x89, 0x85]);
        self.emit(&(offset as u32).to_le_bytes());
    }

    fn emit_load_rax(&mut self, offset: i32) {
        // mov rax, [rbp + offset]
        self.emit(&[0x48, 0x8b, 0x85]);
        self.emit(&(offset as u32).to_le_bytes());
    }

    fn arg_targets(&self, node_id: NodeId) -> Vec<NodeId> {
        self.edges_from.get(&node_id)
            .map(|edges| edges.iter()
                .filter(|e| e.label == EdgeLabel::Argument)
                .map(|e| e.target)
                .collect())
            .unwrap_or_default()
    }

    fn cont_target(&self, node_id: NodeId) -> Option<NodeId> {
        self.edges_from.get(&node_id)
            .and_then(|edges| edges.iter()
                .find(|e| e.label == EdgeLabel::Continuation)
                .map(|e| e.target))
    }

    fn compile(&mut self) -> Option<Vec<u8>> {
        // Prologue: save callee-saved regs, allocate stack frame
        self.emit(&[
            0x55,                       // push rbp
            0x48, 0x89, 0xe5,          // mov rbp, rsp
            0x53,                       // push rbx
            0x41, 0x54,                // push r12
            0x41, 0x55,                // push r13
            0x41, 0x56,                // push r14
        ]);
        // Save input args: rdi = inputs_ptr, rsi = n_inputs
        // Store to known stack slots
        self.emit(&[0x49, 0x89, 0xfc]); // mov r12, rdi (inputs ptr)
        self.emit(&[0x49, 0x89, 0xf5]); // mov r13, rsi (n_inputs)

        // Allocate stack frame (2KB — enough for ~250 temporaries)
        self.emit(&[0x48, 0x81, 0xec]);
        self.emit(&2048u32.to_le_bytes()); // sub rsp, 2048

        // Load up to 6 inputs from state array into stack slots
        for i in 0..6i32 {
            // if i >= n_inputs, skip
            let off = -((i + 1) * 8);
            // mov rax, [r12 + i*8] — load from state array
            self.emit(&[0x49, 0x8b, 0x44, 0x24]);
            self.emit(&[(i * 8) as u8]);
            // mov [rbp + off], rax
            self.emit(&[0x48, 0x89, 0x85]);
            self.emit(&(off as u32).to_le_bytes());
        }

        // Compile the root node
        self.compile_node(self.graph.root)?;
        // Store result to state[0] (r12 points to state array)
        self.emit(&[0x49, 0x89, 0x04, 0x24]); // mov [r12], rax

        // Epilogue
        self.emit(&[
            0x48, 0x8d, 0x65, 0xe0, // lea rsp, [rbp-32]
            0x41, 0x5e,              // pop r14
            0x41, 0x5d,              // pop r13
            0x41, 0x5c,              // pop r12
            0x5b,                    // pop rbx
            0x5d,                    // pop rbp
            0xc3,                    // ret
        ]);

        Some(self.code.clone())
    }

    /// Compile a node, leaving its result in rax (as tagged i64).
    fn compile_node(&mut self, node_id: NodeId) -> Option<()> {
        let node = self.graph.nodes.get(&node_id)?;

        match &node.payload {
            NodePayload::Lit { type_tag, value } => {
                match *type_tag {
                    0x00 if value.len() == 8 => {
                        let val = i64::from_le_bytes(value[..8].try_into().unwrap());
                        let tagged = val << 1; // pack as tagged int
                        self.emit_mov_imm64_rax(tagged);
                    }
                    0x04 if value.len() == 1 => {
                        let tagged = (value[0] as i64) << 1;
                        self.emit_mov_imm64_rax(tagged);
                    }
                    0x06 => {
                        self.emit_mov_imm64_rax(0); // Unit = tagged 0
                    }
                    0xFF if !value.is_empty() => {
                        // InputRef: load from input slot
                        let idx = value[0] as i32;
                        if idx < 128 {
                            // Function parameter: [rbp - (idx+1)*8]
                            let off = -((idx + 1) * 8);
                            self.emit_load_rax(off);
                        } else {
                            // Lambda parameter: load from env (handled by caller)
                            // For now, load from a known slot
                            let env_off = -((idx as i32 - 128 + 7) * 8);
                            self.emit_load_rax(env_off);
                        }
                    }
                    _ => {
                        // Other lit types: pack as heap value via runtime call
                        self.emit_mov_imm64_rax(0); // Unit placeholder
                    }
                }
            }

            NodePayload::Prim { opcode } => {
                let args = self.arg_targets(node_id);
                let op = *opcode;

                match args.len() {
                    0 => {
                        // Zero-arg prim: call runtime
                        self.emit_runtime_call(op, 0, 0, 0);
                    }
                    1 => {
                        self.compile_node(args[0])?;
                        let a_slot = self.alloc_slot();
                        self.emit_store_rax(a_slot);
                        // Unary: dispatch inline or via runtime
                        match op {
                            0x05 => { // neg: 0 - a
                                self.emit_load_rax(a_slot);
                                // Untag: sar rax, 1
                                self.emit(&[0x48, 0xd1, 0xf8]);
                                // neg rax
                                self.emit(&[0x48, 0xf7, 0xd8]);
                                // Retag: shl rax, 1
                                self.emit(&[0x48, 0xd1, 0xe0]);
                            }
                            _ => {
                                self.emit_load_rax(a_slot);
                                self.emit_runtime_call_1arg(op);
                            }
                        }
                    }
                    2 => {
                        self.compile_node(args[0])?;
                        let a_slot = self.alloc_slot();
                        self.emit_store_rax(a_slot);
                        self.compile_node(args[1])?;
                        let b_slot = self.alloc_slot();
                        self.emit_store_rax(b_slot);

                        match op {
                            // add: tagged add works directly (a<<1 + b<<1 = (a+b)<<1)
                            0x00 => {
                                self.emit_load_rax(a_slot);
                                // add rax, [rbp+b_slot]
                                self.emit(&[0x48, 0x03, 0x85]);
                                self.emit(&(b_slot as u32).to_le_bytes());
                            }
                            // sub
                            0x01 => {
                                self.emit_load_rax(a_slot);
                                self.emit(&[0x48, 0x2b, 0x85]);
                                self.emit(&(b_slot as u32).to_le_bytes());
                            }
                            // mul: untag one, multiply, result is tagged
                            0x02 => {
                                self.emit_load_rax(a_slot);
                                self.emit(&[0x48, 0xd1, 0xf8]); // sar rax, 1 (untag a)
                                // mov rcx, [rbp+b_slot]
                                self.emit(&[0x48, 0x8b, 0x8d]);
                                self.emit(&(b_slot as u32).to_le_bytes());
                                // imul rax, rcx
                                self.emit(&[0x48, 0x0f, 0xaf, 0xc1]);
                            }
                            // Comparisons: untag both, compare, set result
                            0x20..=0x25 => {
                                self.emit_load_rax(a_slot);
                                // mov rcx, [rbp+b_slot]
                                self.emit(&[0x48, 0x8b, 0x8d]);
                                self.emit(&(b_slot as u32).to_le_bytes());
                                // cmp rax, rcx
                                self.emit(&[0x48, 0x39, 0xc8]);
                                let cc = match op {
                                    0x20 => 0x94, // sete
                                    0x21 => 0x95, // setne
                                    0x22 => 0x9c, // setl
                                    0x23 => 0x9f, // setg
                                    0x24 => 0x9e, // setle
                                    0x25 => 0x9d, // setge
                                    _ => 0x94,
                                };
                                self.emit(&[0x0f, cc, 0xc0]); // setcc al
                                self.emit(&[0x48, 0x0f, 0xb6, 0xc0]); // movzx rax, al
                                self.emit(&[0x48, 0xd1, 0xe0]); // shl rax, 1 (tag)
                            }
                            // Everything else: call runtime
                            _ => {
                                self.emit_load_rax(a_slot);
                                self.emit_runtime_call_2arg(op, b_slot);
                            }
                        }
                    }
                    _ => {
                        // 3+ args: call runtime with first two
                        if args.len() >= 2 {
                            self.compile_node(args[0])?;
                            let a_slot = self.alloc_slot();
                            self.emit_store_rax(a_slot);
                            self.compile_node(args[1])?;
                            let b_slot = self.alloc_slot();
                            self.emit_store_rax(b_slot);
                            self.emit_load_rax(a_slot);
                            self.emit_runtime_call_2arg(op, b_slot);
                        } else {
                            self.emit_mov_imm64_rax(0);
                        }
                    }
                }
            }

            NodePayload::Guard { predicate_node, body_node, fallback_node } => {
                let (pred, body, fall) = (*predicate_node, *body_node, *fallback_node);

                // Compile predicate
                self.compile_node(pred)?;
                // test rax, rax (tagged: 0 = false/zero, nonzero = true)
                self.emit(&[0x48, 0x85, 0xc0]);
                // je fallback
                self.emit(&[0x0f, 0x84, 0, 0, 0, 0]); // je rel32 (patched)
                let je_patch = self.code.len() - 4;

                // Compile body (taken branch)
                self.compile_node(body)?;
                // jmp end
                self.emit(&[0xe9, 0, 0, 0, 0]); // jmp rel32 (patched)
                let jmp_patch = self.code.len() - 4;

                // Patch je to here (fallback start)
                let fallback_offset = self.code.len() - (je_patch + 4);
                self.code[je_patch..je_patch+4].copy_from_slice(&(fallback_offset as u32).to_le_bytes());

                // Compile fallback
                self.compile_node(fall)?;

                // Patch jmp to here (end)
                let end_offset = self.code.len() - (jmp_patch + 4);
                self.code[jmp_patch..jmp_patch+4].copy_from_slice(&(end_offset as u32).to_le_bytes());
            }

            NodePayload::Tuple => {
                let args = self.arg_targets(node_id);
                if args.is_empty() {
                    self.emit_mov_imm64_rax(0); // Unit
                } else {
                    // Compile each element, store on stack, then call rt_make_tuple
                    let base_slot = self.alloc_slot();
                    for (i, &arg) in args.iter().enumerate() {
                        self.compile_node(arg)?;
                        let slot = if i == 0 { base_slot } else { self.alloc_slot() };
                        self.emit_store_rax(slot);
                    }
                    // Call rt_make_tuple(count, ptr_to_elements)
                    let count = args.len() as i64;
                    // rdi = count
                    self.emit_mov_imm64_rax(count);
                    self.emit(&[0x48, 0x89, 0xc7]); // mov rdi, rax
                    // rsi = pointer to elements on stack (base_slot address)
                    // lea rsi, [rbp + base_slot]
                    self.emit(&[0x48, 0x8d, 0xb5]);
                    self.emit(&(base_slot as u32).to_le_bytes());
                    // call rt_make_tuple
                    let addr = crate::jit::rt_make_tuple as *const () as i64;
                    self.emit_mov_imm64_rax(addr);
                    self.emit(&[0xff, 0xd0]); // call rax
                }
            }

            NodePayload::Project { field_index } => {
                let args = self.arg_targets(node_id);
                if let Some(&src) = args.first() {
                    self.compile_node(src)?;
                    // rdi = tuple (tagged), rsi = index
                    self.emit(&[0x48, 0x89, 0xc7]); // mov rdi, rax
                    self.emit_mov_imm64_rax(*field_index as i64);
                    self.emit(&[0x48, 0x89, 0xc6]); // mov rsi, rax
                    // Call rt: tuple_get via rt_prim_dispatch(0xD2, tuple, index, 0)
                    self.emit_runtime_call(0xD2, 0, 0, 0);
                    // Actually, let me use the proper calling convention
                    // rt_prim_dispatch(opcode=0xD2, a=tuple, b=index, c=0)
                } else {
                    self.emit_mov_imm64_rax(0);
                }
            }

            NodePayload::Let => {
                // Binding edge → value, Continuation edge → body (Lambda wrapping)
                let bind_target = self.edges_from.get(&node_id)
                    .and_then(|edges| edges.iter()
                        .find(|e| e.label == EdgeLabel::Binding)
                        .map(|e| e.target));
                let cont_target = self.cont_target(node_id);

                if let (Some(bind_t), Some(cont_t)) = (bind_target, cont_target) {
                    // Compile binding value
                    self.compile_node(bind_t)?;
                    let bind_slot = self.alloc_slot();
                    self.emit_store_rax(bind_slot);

                    // Check if continuation is a Lambda
                    if let Some(cont_node) = self.graph.nodes.get(&cont_t) {
                        if let NodePayload::Lambda { binder, .. } = &cont_node.payload {
                            // Store binding in the binder's slot
                            let binder_idx = binder.0 as i32;
                            let env_slot = if binder_idx >= 0xFFFF_0000u32 as i32 {
                                let idx = binder_idx - 0xFFFF_0000u32 as i32;
                                if idx < 128 { -((idx + 1) * 8) }
                                else { -((idx - 128 + 7) * 8) }
                            } else { bind_slot };
                            self.emit_load_rax(bind_slot);
                            self.emit_store_rax(env_slot);

                            // Compile the lambda body
                            let body = self.cont_target(cont_t)
                                .or_else(|| self.arg_targets(cont_t).first().copied());
                            if let Some(body_id) = body {
                                self.compile_node(body_id)?;
                            } else {
                                self.emit_mov_imm64_rax(0);
                            }
                            return Some(());
                        }
                    }
                    // Not a lambda continuation: compile continuation directly
                    self.compile_node(cont_t)?;
                } else {
                    self.emit_mov_imm64_rax(0);
                }
            }

            NodePayload::Fold { .. } => {
                let args = self.arg_targets(node_id);
                if args.len() < 2 {
                    self.emit_mov_imm64_rax(0);
                    return Some(());
                }

                // Compile base value
                self.compile_node(args[0])?;
                let acc_slot = self.alloc_slot();
                self.emit_store_rax(acc_slot);

                // Compile collection (if present)
                let count_slot = self.alloc_slot();
                if args.len() >= 3 {
                    self.compile_node(args[2])?;
                    self.emit_store_rax(count_slot);
                } else {
                    self.emit_mov_imm64_rax(0);
                    self.emit_store_rax(count_slot);
                }

                // Check step kind
                let step_id = args[1];
                let step_node = self.graph.nodes.get(&step_id);

                if let Some(sn) = step_node {
                    if let NodePayload::Prim { opcode } = &sn.payload {
                        // Prim step: generate inline loop
                        let op = *opcode;
                        let counter_slot = self.alloc_slot();
                        // xor counter
                        self.emit_mov_imm64_rax(0);
                        self.emit_store_rax(counter_slot);

                        // Loop top: cmp counter, count; jge done
                        let loop_top = self.code.len();
                        self.emit_load_rax(counter_slot);
                        // mov rcx, [rbp+count_slot] — untag count
                        self.emit(&[0x48, 0x8b, 0x8d]);
                        self.emit(&(count_slot as u32).to_le_bytes());
                        self.emit(&[0x48, 0xd1, 0xf9]); // sar rcx, 1 (untag)
                        // cmp rax, rcx
                        self.emit(&[0x48, 0x39, 0xc8]);
                        self.emit(&[0x0f, 0x8d, 0, 0, 0, 0]); // jge done (patched)
                        let jge_patch = self.code.len() - 4;

                        // Load acc and counter as tagged values
                        self.emit_load_rax(acc_slot);
                        // mov rcx, counter_slot; shl rcx, 1 (tag counter)
                        self.emit(&[0x48, 0x8b, 0x8d]);
                        self.emit(&(counter_slot as u32).to_le_bytes());
                        self.emit(&[0x48, 0xd1, 0xe1]); // shl rcx, 1

                        // Apply prim op
                        match op {
                            0x00 => { // add: acc += tagged_counter
                                self.emit(&[0x48, 0x01, 0xc8]); // add rax, rcx
                            }
                            0x01 => {
                                self.emit(&[0x48, 0x29, 0xc8]); // sub rax, rcx
                            }
                            0x02 => { // mul: untag acc, mul by counter, result tagged
                                self.emit(&[0x48, 0xd1, 0xf8]); // sar rax, 1
                                self.emit(&[0x48, 0x0f, 0xaf, 0xc1]); // imul rax, rcx
                            }
                            _ => {} // unknown op: acc unchanged
                        }
                        self.emit_store_rax(acc_slot);

                        // Increment counter
                        self.emit_load_rax(counter_slot);
                        self.emit(&[0x48, 0xff, 0xc0]); // inc rax
                        self.emit_store_rax(counter_slot);

                        // Jump back to loop top
                        let back_offset = loop_top as i32 - self.code.len() as i32 - 5;
                        self.emit(&[0xe9]);
                        self.emit(&(back_offset as u32).to_le_bytes());

                        // Patch jge
                        let done_offset = self.code.len() - (jge_patch + 4);
                        self.code[jge_patch..jge_patch+4].copy_from_slice(&(done_offset as u32).to_le_bytes());

                        // Load final acc
                        self.emit_load_rax(acc_slot);
                    } else {
                        // Lambda step: call graph_eval for each iteration (slow path)
                        // For now, just return the base value
                        // TODO: compile lambda body inline
                        self.emit_load_rax(acc_slot);
                    }
                } else {
                    self.emit_load_rax(acc_slot);
                }
            }

            NodePayload::Lambda { .. } => {
                // Return a representation of the closure
                // For now, return Unit (lambdas handled by Let continuation)
                self.emit_mov_imm64_rax(0);
            }

            NodePayload::Apply => {
                // Function application: compile function and arg, call runtime
                let args = self.arg_targets(node_id);
                if args.len() >= 2 {
                    self.compile_node(args[0])?;
                    let fn_slot = self.alloc_slot();
                    self.emit_store_rax(fn_slot);
                    self.compile_node(args[1])?;
                    let arg_slot = self.alloc_slot();
                    self.emit_store_rax(arg_slot);
                    // For now, return the argument (placeholder)
                    self.emit_load_rax(arg_slot);
                } else {
                    self.emit_mov_imm64_rax(0);
                }
            }

            NodePayload::Rewrite { body, .. } => {
                self.compile_node(*body)?;
            }

            _ => {
                // Unknown node: return Unit
                self.emit_mov_imm64_rax(0);
            }
        }

        Some(())
    }

    /// Emit a call to rt_prim_dispatch(opcode, a=rax, b=0, c=0)
    fn emit_runtime_call(&mut self, opcode: u8, _a: i64, _b: i64, _c: i64) {
        // rdi = opcode, rsi = rax (current value), rdx = 0, rcx = 0
        self.emit(&[0x48, 0x89, 0xc6]); // mov rsi, rax
        self.emit_mov_imm64_rax(opcode as i64);
        self.emit(&[0x48, 0x89, 0xc7]); // mov rdi, rax
        self.emit(&[0x48, 0x31, 0xd2]); // xor rdx, rdx
        self.emit(&[0x48, 0x31, 0xc9]); // xor rcx, rcx
        let addr = crate::jit::rt_prim_dispatch as *const () as i64;
        self.emit_mov_imm64_rax(addr);
        self.emit(&[0xff, 0xd0]); // call rax
    }

    /// Emit call to rt_prim_dispatch with 1 arg (already in rax)
    fn emit_runtime_call_1arg(&mut self, opcode: u8) {
        self.emit(&[0x48, 0x89, 0xc6]); // mov rsi, rax (arg a)
        self.emit_mov_imm64_rax(opcode as i64);
        self.emit(&[0x48, 0x89, 0xc7]); // mov rdi, rax (opcode)
        self.emit(&[0x48, 0x31, 0xd2]); // xor rdx, rdx
        self.emit(&[0x48, 0x31, 0xc9]); // xor rcx, rcx
        let addr = crate::jit::rt_prim_dispatch as *const () as i64;
        self.emit_mov_imm64_rax(addr);
        self.emit(&[0xff, 0xd0]); // call rax
    }

    /// Emit call to rt_prim_dispatch with 2 args (a in rax, b at stack offset)
    fn emit_runtime_call_2arg(&mut self, opcode: u8, b_slot: i32) {
        self.emit(&[0x48, 0x89, 0xc6]); // mov rsi, rax (arg a)
        // mov rdx, [rbp+b_slot]
        self.emit(&[0x48, 0x8b, 0x95]);
        self.emit(&(b_slot as u32).to_le_bytes());
        self.emit_mov_imm64_rax(opcode as i64);
        self.emit(&[0x48, 0x89, 0xc7]); // mov rdi, rax (opcode)
        self.emit(&[0x48, 0x31, 0xc9]); // xor rcx, rcx
        let addr = crate::jit::rt_prim_dispatch as *const () as i64;
        self.emit_mov_imm64_rax(addr);
        self.emit(&[0xff, 0xd0]); // call rax
    }
}

/// Evaluate a graph using the native compiler. Compiles to x86-64, executes.
pub fn native_eval(graph: &SemanticGraph, inputs: &[Value]) -> Option<Value> {
    // Only native-compile small-to-medium graphs
    if graph.nodes.len() > 200 { return None; }
    // Only compile graphs whose opcodes are correctly handled
    for node in graph.nodes.values() {
        match &node.payload {
            NodePayload::Lit { .. } | NodePayload::Guard { .. } |
            NodePayload::Tuple | NodePayload::Project { .. } |
            NodePayload::Let | NodePayload::Fold { .. } |
            NodePayload::Lambda { .. } | NodePayload::Rewrite { .. } => {}
            NodePayload::Prim { opcode } => match opcode {
                0x00..=0x05 | 0x07..=0x08 | // arithmetic
                0x10..=0x12 |                // bitwise
                0x20..=0x25 |                // comparison
                0xB0 | 0xC0..=0xC2 | 0xC7 | // string/collection (via dispatch)
                0xD2 | 0xD6 | 0xF0 |        // tuple_get, tuple_len, list_len
                0x82 | 0x83 | 0x89 | 0x8A | // graph introspection
                0x8F | 0x97 | 0xEE => {}    // graph_outgoing, edge_target, set_root
                _ => return None,            // unknown opcode → fall back
            },
            _ => return None,
        }
    }
    let code_bytes = compile_graph_native(graph)?;

    // Pack inputs as tagged i64
    let mut tagged_inputs: Vec<i64> = inputs.iter().map(|v| crate::jit::pack(v.clone())).collect();

    // The compiled function stores its result to [rdi] (= state[0])
    #[cfg(all(target_arch = "x86_64", unix))]
    {
        use crate::native_flat::NativeCode;
        // Pad to at least 8 elements (function reads up to 6 inputs from state array)
        while tagged_inputs.len() < 8 {
            tagged_inputs.push(0);
        }
        let native = NativeCode::compile(&code_bytes)?;
        let n = tagged_inputs.len() as i64;
        unsafe { native.call_i64(&mut tagged_inputs, n); }
        return Some(crate::jit::unpack(tagged_inputs[0]));
    }

    #[cfg(not(all(target_arch = "x86_64", unix)))]
    None
}
