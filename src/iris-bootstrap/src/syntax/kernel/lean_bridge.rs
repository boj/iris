//! Lean 4 IPC bridge — calls proven kernel functions via a subprocess.
//!
//! The Lean code at `lean/IrisKernel/` IS the formal proof. This module
//! communicates with a Lean server process (`iris-kernel-server`) via
//! stdin/stdout pipes, so the running code is the proven code — without
//! linking the Lean runtime into the Rust process.
//!
//! ## Wire format
//!
//! Request:  rule_id (1 byte) + payload_len (4 bytes LE) + payload bytes
//! Response: result_len (4 bytes LE) + result bytes
//!
//! Rule ID 0 = cost_leq check (result is 1 byte: 0 or 1)
//! Rule IDs 1-20 = kernel inference rules (result is encoded Judgment)
//! Rule ID 255 = shutdown

use crate::syntax::kernel::cost_checker;
use crate::syntax::kernel::theorem::{Binding, Context, Judgment};
use iris_types::cost::{CostBound, CostVar};
use iris_types::graph::{BinderId, NodeId};
use iris_types::types::TypeId;

// ---------------------------------------------------------------------------
// IPC rule IDs (must match lean/IrisKernelServer.lean dispatchRule)
// ---------------------------------------------------------------------------

#[cfg(feature = "lean-ffi")]
mod rule_ids {
    pub const RULE_COST_LEQ: u8 = 0;
    pub const RULE_ASSUME: u8 = 1;
    pub const RULE_INTRO: u8 = 2;
    pub const RULE_ELIM: u8 = 3;
    pub const RULE_REFL: u8 = 4;
    pub const RULE_SYMM: u8 = 5;
    pub const RULE_TRANS: u8 = 6;
    pub const RULE_CONGR: u8 = 7;
    #[allow(dead_code)]
    pub const RULE_TYPE_CHECK_NODE_FULL: u8 = 8;
    pub const RULE_COST_SUBSUME: u8 = 9;
    pub const RULE_COST_LEQ_RULE: u8 = 10;
    #[allow(dead_code)]
    pub const RULE_REFINE_INTRO: u8 = 11;
    #[allow(dead_code)]
    pub const RULE_REFINE_ELIM: u8 = 12;
    pub const RULE_NAT_IND: u8 = 13;
    #[allow(dead_code)]
    pub const RULE_STRUCTURAL_IND: u8 = 14;
    pub const RULE_LET_BIND: u8 = 15;
    pub const RULE_MATCH_ELIM: u8 = 16;
    pub const RULE_FOLD_RULE: u8 = 17;
    #[allow(dead_code)]
    pub const RULE_TYPE_ABST: u8 = 18;
    #[allow(dead_code)]
    pub const RULE_TYPE_APP: u8 = 19;
    pub const RULE_GUARD_RULE: u8 = 20;
    pub const RULE_SHUTDOWN: u8 = 255;
}

#[cfg(feature = "lean-ffi")]
use rule_ids::*;

// ---------------------------------------------------------------------------
// Lean kernel server process management (lean-ffi only)
// ---------------------------------------------------------------------------

#[cfg(feature = "lean-ffi")]
mod ipc {
    use std::io::{Read, Write, BufReader, BufWriter};
    use std::process::{Child, Command, Stdio};
    use std::sync::Mutex;

    /// The Lean kernel server process and its I/O handles.
    struct LeanKernelProcess {
        child: Child,
        stdin: BufWriter<std::process::ChildStdin>,
        stdout: BufReader<std::process::ChildStdout>,
    }

    impl Drop for LeanKernelProcess {
        fn drop(&mut self) {
            // Send shutdown command (rule 255) — best effort
            let _ = self.stdin.write_all(&[super::RULE_SHUTDOWN]);
            let _ = self.stdin.write_all(&0u32.to_le_bytes());
            let _ = self.stdin.flush();
            // Wait briefly for clean exit, then kill
            match self.child.try_wait() {
                Ok(Some(_)) => {} // already exited
                _ => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                }
            }
        }
    }

    static LEAN_PROCESS: std::sync::OnceLock<Mutex<Option<LeanKernelProcess>>> =
        std::sync::OnceLock::new();

    /// Find the Lean kernel server binary.
    fn find_server_binary() -> Option<std::path::PathBuf> {
        // 1. Environment variable override
        if let Ok(path) = std::env::var("IRIS_KERNEL_SERVER") {
            let p = std::path::PathBuf::from(path);
            if p.exists() {
                return Some(p);
            }
        }

        // 2. Relative to CARGO_MANIFEST_DIR (build-time)
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let p = std::path::PathBuf::from(&manifest_dir)
                .join("../../lean/.lake/build/bin/iris-kernel-server");
            if p.exists() {
                return Some(p);
            }
        }

        // 3. Relative to the current executable
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                // Try: <exe_dir>/../lean/.lake/build/bin/iris-kernel-server
                let p = dir.join("../lean/.lake/build/bin/iris-kernel-server");
                if p.exists() {
                    return Some(p);
                }
            }
        }

        // 4. Relative to workspace root (common in dev)
        for base in &[".", ".."] {
            let p = std::path::PathBuf::from(base)
                .join("lean/.lake/build/bin/iris-kernel-server");
            if p.exists() {
                return Some(p);
            }
        }

        // 5. Search up from CWD to find lean/ directory
        if let Ok(cwd) = std::env::current_dir() {
            let mut dir = cwd.as_path();
            loop {
                let p = dir.join("lean/.lake/build/bin/iris-kernel-server");
                if p.exists() {
                    return Some(p);
                }
                match dir.parent() {
                    Some(parent) => dir = parent,
                    None => break,
                }
            }
        }

        None
    }

    /// Spawn the Lean kernel server process.
    fn spawn_server() -> Result<LeanKernelProcess, String> {
        let binary = find_server_binary()
            .ok_or_else(|| "Could not find iris-kernel-server binary. Set IRIS_KERNEL_SERVER env var or run `lake build iris-kernel-server` in the lean/ directory.".to_string())?;

        let mut child = Command::new(&binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to spawn iris-kernel-server at {:?}: {}", binary, e))?;

        let child_stdin = child.stdin.take()
            .ok_or_else(|| "Failed to capture stdin of kernel server".to_string())?;
        let child_stdout = child.stdout.take()
            .ok_or_else(|| "Failed to capture stdout of kernel server".to_string())?;

        let mut stdout = BufReader::new(child_stdout);

        // Wait for the ready signal: 4 bytes "IRIS" magic
        let mut magic = [0u8; 4];
        stdout.read_exact(&mut magic)
            .map_err(|e| format!("Failed to read ready signal from kernel server: {}", e))?;
        if &magic != b"IRIS" {
            return Err(format!(
                "Invalid ready signal from kernel server: {:?} (expected b\"IRIS\")", magic
            ));
        }

        Ok(LeanKernelProcess {
            child,
            stdin: BufWriter::new(child_stdin),
            stdout,
        })
    }

    /// Send a request to the Lean kernel server and get the response bytes.
    /// Returns None if the server is unavailable or returns an error.
    pub fn call_rule(rule_id: u8, payload: &[u8]) -> Option<Vec<u8>> {
        let mutex = LEAN_PROCESS.get_or_init(|| {
            let proc = spawn_server().ok();
            Mutex::new(proc)
        });

        let mut guard = mutex.lock().ok()?;
        let proc = guard.as_mut()?;

        // Write request: rule_id + payload_len(u32 LE) + payload
        if proc.stdin.write_all(&[rule_id]).is_err() {
            *guard = None;
            return None;
        }
        if proc.stdin.write_all(&(payload.len() as u32).to_le_bytes()).is_err() {
            *guard = None;
            return None;
        }
        if proc.stdin.write_all(payload).is_err() {
            *guard = None;
            return None;
        }
        if proc.stdin.flush().is_err() {
            *guard = None;
            return None;
        }

        // Read response: result_len(u32 LE) + result bytes
        let mut len_buf = [0u8; 4];
        if proc.stdout.read_exact(&mut len_buf).is_err() {
            *guard = None;
            return None;
        }
        let len = u32::from_le_bytes(len_buf) as usize;

        // Sanity check: don't allocate more than 16 MiB
        if len > 16 * 1024 * 1024 {
            *guard = None;
            return None;
        }

        let mut result = vec![0u8; len];
        if proc.stdout.read_exact(&mut result).is_err() {
            *guard = None;
            return None;
        }

        Some(result)
    }
}

// ---------------------------------------------------------------------------
// Wire format encoding (must match lean/IrisKernel/FFI.lean decodeCostBound)
// ---------------------------------------------------------------------------

pub fn encode_cost_bound(cost: &CostBound, buf: &mut Vec<u8>) {
    match cost {
        CostBound::Unknown => buf.push(0x00),
        CostBound::Zero => buf.push(0x01),
        CostBound::Constant(k) => {
            buf.push(0x02);
            buf.extend_from_slice(&(*k as u64).to_le_bytes());
        }
        CostBound::Linear(CostVar(v)) => {
            buf.push(0x03);
            buf.extend_from_slice(&(*v as u32).to_le_bytes());
        }
        CostBound::NLogN(CostVar(v)) => {
            buf.push(0x04);
            buf.extend_from_slice(&(*v as u32).to_le_bytes());
        }
        CostBound::Polynomial(CostVar(v), deg) => {
            buf.push(0x05);
            buf.extend_from_slice(&(*v as u32).to_le_bytes());
            buf.extend_from_slice(&(*deg as u32).to_le_bytes());
        }
        CostBound::Sum(a, b) => {
            buf.push(0x06);
            encode_cost_bound(a, buf);
            encode_cost_bound(b, buf);
        }
        CostBound::Par(a, b) => {
            buf.push(0x07);
            encode_cost_bound(a, buf);
            encode_cost_bound(b, buf);
        }
        CostBound::Mul(a, b) => {
            buf.push(0x08);
            encode_cost_bound(a, buf);
            encode_cost_bound(b, buf);
        }
        CostBound::Sup(costs) => {
            buf.push(0x09);
            buf.extend_from_slice(&(costs.len() as u16).to_le_bytes());
            for c in costs {
                encode_cost_bound(c, buf);
            }
        }
        CostBound::Inf(costs) => {
            buf.push(0x0A);
            buf.extend_from_slice(&(costs.len() as u16).to_le_bytes());
            for c in costs {
                encode_cost_bound(c, buf);
            }
        }
        CostBound::Amortized(inner, _) => {
            buf.push(0x0B);
            encode_cost_bound(inner, buf);
        }
        CostBound::HWScaled(inner, _) => {
            buf.push(0x0C);
            encode_cost_bound(inner, buf);
        }
    }
}

// ---------------------------------------------------------------------------
// Wire format encoding — kernel types
// (must match lean/IrisKernel/FFI.lean decoders)
// ---------------------------------------------------------------------------

/// Encode a NodeId as u64 LE.
pub fn encode_node_id(node: NodeId, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&node.0.to_le_bytes());
}

/// Encode a TypeId as u64 LE.
pub fn encode_type_id(ty: TypeId, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&ty.0.to_le_bytes());
}

/// Encode a BinderId as u32 LE.
pub fn encode_binder_id(binder: BinderId, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&binder.0.to_le_bytes());
}

/// Encode a Context: count (u16 LE), then count x (BinderId + TypeId) pairs.
pub fn encode_context(ctx: &Context, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&(ctx.bindings.len() as u16).to_le_bytes());
    for binding in &ctx.bindings {
        encode_binder_id(binding.name, buf);
        encode_type_id(binding.type_id, buf);
    }
}

/// Encode a Judgment: Context + NodeId + TypeId + CostBound.
pub fn encode_judgment(j: &Judgment, buf: &mut Vec<u8>) {
    encode_context(&j.context, buf);
    encode_node_id(j.node_id, buf);
    encode_type_id(j.type_ref, buf);
    encode_cost_bound(&j.cost, buf);
}

// ---------------------------------------------------------------------------
// Wire format decoding — kernel types
// (must match lean/IrisKernel/FFI.lean encoders)
// ---------------------------------------------------------------------------

/// Decode a NodeId (u64 LE) from bytes at offset. Returns (NodeId, new_offset).
pub fn decode_node_id(data: &[u8], offset: usize) -> Option<(NodeId, usize)> {
    if offset + 8 > data.len() { return None; }
    let val = u64::from_le_bytes(data[offset..offset+8].try_into().ok()?);
    Some((NodeId(val), offset + 8))
}

/// Decode a TypeId (u64 LE) from bytes at offset. Returns (TypeId, new_offset).
pub fn decode_type_id(data: &[u8], offset: usize) -> Option<(TypeId, usize)> {
    if offset + 8 > data.len() { return None; }
    let val = u64::from_le_bytes(data[offset..offset+8].try_into().ok()?);
    Some((TypeId(val), offset + 8))
}

/// Decode a BinderId (u32 LE) from bytes at offset.
pub fn decode_binder_id(data: &[u8], offset: usize) -> Option<(BinderId, usize)> {
    if offset + 4 > data.len() { return None; }
    let val = u32::from_le_bytes(data[offset..offset+4].try_into().ok()?);
    Some((BinderId(val), offset + 4))
}

/// Decode a Context from bytes at offset.
pub fn decode_context(data: &[u8], offset: usize) -> Option<(Context, usize)> {
    if offset + 2 > data.len() { return None; }
    let count = u16::from_le_bytes(data[offset..offset+2].try_into().ok()?) as usize;
    let mut pos = offset + 2;
    let mut bindings = Vec::with_capacity(count);
    for _ in 0..count {
        let (name, p) = decode_binder_id(data, pos)?;
        let (type_id, p) = decode_type_id(data, p)?;
        bindings.push(Binding { name, type_id });
        pos = p;
    }
    Some((Context { bindings }, pos))
}

/// Decode a CostBound from bytes at offset. Returns (CostBound, new_offset).
pub fn decode_cost_bound(data: &[u8], offset: usize) -> Option<(CostBound, usize)> {
    if offset >= data.len() { return None; }
    let tag = data[offset];
    let pos = offset + 1;
    match tag {
        0x00 => Some((CostBound::Unknown, pos)),
        0x01 => Some((CostBound::Zero, pos)),
        0x02 => {
            if pos + 8 > data.len() { return None; }
            let k = u64::from_le_bytes(data[pos..pos+8].try_into().ok()?);
            Some((CostBound::Constant(k), pos + 8))
        }
        0x03 => {
            if pos + 4 > data.len() { return None; }
            let v = u32::from_le_bytes(data[pos..pos+4].try_into().ok()?);
            Some((CostBound::Linear(CostVar(v)), pos + 4))
        }
        0x04 => {
            if pos + 4 > data.len() { return None; }
            let v = u32::from_le_bytes(data[pos..pos+4].try_into().ok()?);
            Some((CostBound::NLogN(CostVar(v)), pos + 4))
        }
        0x05 => {
            if pos + 8 > data.len() { return None; }
            let v = u32::from_le_bytes(data[pos..pos+4].try_into().ok()?);
            let d = u32::from_le_bytes(data[pos+4..pos+8].try_into().ok()?);
            Some((CostBound::Polynomial(CostVar(v), d), pos + 8))
        }
        0x06 => {
            let (a, pos) = decode_cost_bound(data, pos)?;
            let (b, pos) = decode_cost_bound(data, pos)?;
            Some((CostBound::Sum(Box::new(a), Box::new(b)), pos))
        }
        0x07 => {
            let (a, pos) = decode_cost_bound(data, pos)?;
            let (b, pos) = decode_cost_bound(data, pos)?;
            Some((CostBound::Par(Box::new(a), Box::new(b)), pos))
        }
        0x08 => {
            let (a, pos) = decode_cost_bound(data, pos)?;
            let (b, pos) = decode_cost_bound(data, pos)?;
            Some((CostBound::Mul(Box::new(a), Box::new(b)), pos))
        }
        0x09 => {
            if pos + 2 > data.len() { return None; }
            let count = u16::from_le_bytes(data[pos..pos+2].try_into().ok()?) as usize;
            let mut p = pos + 2;
            let mut vs = Vec::with_capacity(count);
            for _ in 0..count {
                let (v, np) = decode_cost_bound(data, p)?;
                vs.push(v);
                p = np;
            }
            Some((CostBound::Sup(vs), p))
        }
        0x0A => {
            if pos + 2 > data.len() { return None; }
            let count = u16::from_le_bytes(data[pos..pos+2].try_into().ok()?) as usize;
            let mut p = pos + 2;
            let mut vs = Vec::with_capacity(count);
            for _ in 0..count {
                let (v, np) = decode_cost_bound(data, p)?;
                vs.push(v);
                p = np;
            }
            Some((CostBound::Inf(vs), p))
        }
        0x0B => {
            let (inner, pos) = decode_cost_bound(data, pos)?;
            Some((CostBound::Amortized(Box::new(inner), iris_types::cost::PotentialFn { description: String::new() }), pos))
        }
        0x0C => {
            let (inner, pos) = decode_cost_bound(data, pos)?;
            Some((CostBound::HWScaled(Box::new(inner), iris_types::cost::HWParamRef([0; 32])), pos))
        }
        _ => None,
    }
}

/// Decode a Judgment from bytes at offset.
pub fn decode_judgment(data: &[u8], offset: usize) -> Option<(Judgment, usize)> {
    let (context, pos) = decode_context(data, offset)?;
    let (node_id, pos) = decode_node_id(data, pos)?;
    let (type_ref, pos) = decode_type_id(data, pos)?;
    let (cost, pos) = decode_cost_bound(data, pos)?;
    Some((Judgment { context, node_id, type_ref, cost }, pos))
}

/// Decode a lean kernel result: byte 0 = success(1)/failure(0), then Judgment or error.
pub fn decode_lean_result(data: &[u8]) -> Option<Judgment> {
    if data.is_empty() { return None; }
    if data[0] != 1 { return None; }
    let (j, _) = decode_judgment(data, 1)?;
    Some(j)
}

// ---------------------------------------------------------------------------
// Generic IPC call helper (lean-ffi only)
// ---------------------------------------------------------------------------

/// Call a Lean kernel rule via IPC. Serializes input, sends to the server,
/// deserializes the result Judgment.
#[cfg(feature = "lean-ffi")]
fn call_lean_rule(rule_id: u8, buf: &[u8]) -> Option<Judgment> {
    let result = ipc::call_rule(rule_id, buf)?;
    decode_lean_result(&result)
}

// ---------------------------------------------------------------------------
// Public API — calls Lean when linked, falls back to Rust otherwise
// ---------------------------------------------------------------------------

/// Check if cost bound `a` is less than or equal to `b`.
///
/// When the Lean kernel server is available (`lean-ffi` feature), this calls
/// the formally proven `checkCostLeq` function via IPC. Otherwise falls back
/// to the Rust implementation.
#[cfg(feature = "lean-ffi")]
pub fn lean_check_cost_leq(a: &CostBound, b: &CostBound) -> bool {
    let mut buf = Vec::with_capacity(64);
    encode_cost_bound(a, &mut buf);
    encode_cost_bound(b, &mut buf);

    match ipc::call_rule(RULE_COST_LEQ, &buf) {
        Some(result) if !result.is_empty() => result[0] == 1,
        _ => {
            // Fallback to Rust if IPC fails
            cost_checker::cost_leq(a, b)
        }
    }
}

#[cfg(not(feature = "lean-ffi"))]
pub fn lean_check_cost_leq(a: &CostBound, b: &CostBound) -> bool {
    // Fallback to Rust implementation
    cost_checker::cost_leq(a, b)
}

// ---------------------------------------------------------------------------
// Public Lean kernel rule functions (lean-ffi feature)
// ---------------------------------------------------------------------------

/// Lean kernel rule 1: assume
#[cfg(feature = "lean-ffi")]
pub fn lean_assume(ctx: &Context, name: BinderId, node_id: NodeId) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(64);
    encode_context(ctx, &mut buf);
    encode_binder_id(name, &mut buf);
    encode_node_id(node_id, &mut buf);
    call_lean_rule(RULE_ASSUME, &buf)
}

/// Lean kernel rule 2: intro
#[cfg(feature = "lean-ffi")]
pub fn lean_intro(
    ctx: &Context, lam_node: NodeId, binder_name: BinderId, binder_type: TypeId,
    body_judgment: &Judgment, arrow_id: TypeId,
    type_env_bytes: &[u8], // pre-encoded TypeEnv
) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(type_env_bytes);
    encode_context(ctx, &mut buf);
    encode_node_id(lam_node, &mut buf);
    encode_binder_id(binder_name, &mut buf);
    encode_type_id(binder_type, &mut buf);
    encode_judgment(body_judgment, &mut buf);
    encode_type_id(arrow_id, &mut buf);
    call_lean_rule(RULE_INTRO, &buf)
}

/// Lean kernel rule 3: elim
#[cfg(feature = "lean-ffi")]
pub fn lean_elim(
    fn_judgment: &Judgment, arg_judgment: &Judgment, app_node: NodeId,
    type_env_bytes: &[u8],
) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(type_env_bytes);
    encode_judgment(fn_judgment, &mut buf);
    encode_judgment(arg_judgment, &mut buf);
    encode_node_id(app_node, &mut buf);
    call_lean_rule(RULE_ELIM, &buf)
}

/// Lean kernel rule 4: refl
#[cfg(feature = "lean-ffi")]
pub fn lean_refl(ctx: &Context, node_id: NodeId, type_id: TypeId) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(64);
    encode_context(ctx, &mut buf);
    encode_node_id(node_id, &mut buf);
    encode_type_id(type_id, &mut buf);
    call_lean_rule(RULE_REFL, &buf)
}

/// Lean kernel rule 5: symm
#[cfg(feature = "lean-ffi")]
pub fn lean_symm(thm: &Judgment, other_node: NodeId, eq_witness: &Judgment) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(thm, &mut buf);
    encode_node_id(other_node, &mut buf);
    encode_judgment(eq_witness, &mut buf);
    call_lean_rule(RULE_SYMM, &buf)
}

/// Lean kernel rule 6: trans
#[cfg(feature = "lean-ffi")]
pub fn lean_trans(thm1: &Judgment, thm2: &Judgment) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(thm1, &mut buf);
    encode_judgment(thm2, &mut buf);
    call_lean_rule(RULE_TRANS, &buf)
}

/// Lean kernel rule 7: congr
#[cfg(feature = "lean-ffi")]
pub fn lean_congr(fn_j: &Judgment, arg_j: &Judgment, app_node: NodeId) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(fn_j, &mut buf);
    encode_judgment(arg_j, &mut buf);
    encode_node_id(app_node, &mut buf);
    call_lean_rule(RULE_CONGR, &buf)
}

/// Lean kernel rule 9: cost_subsume
#[cfg(feature = "lean-ffi")]
pub fn lean_cost_subsume(j: &Judgment, new_cost: &CostBound) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(j, &mut buf);
    encode_cost_bound(new_cost, &mut buf);
    call_lean_rule(RULE_COST_SUBSUME, &buf)
}

/// Lean kernel rule 10: cost_leq_rule
#[cfg(feature = "lean-ffi")]
pub fn lean_cost_leq_rule(k1: &CostBound, k2: &CostBound) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(64);
    encode_cost_bound(k1, &mut buf);
    encode_cost_bound(k2, &mut buf);
    call_lean_rule(RULE_COST_LEQ_RULE, &buf)
}

/// Lean kernel rule 13: nat_ind
#[cfg(feature = "lean-ffi")]
pub fn lean_nat_ind(base_j: &Judgment, step_j: &Judgment, result_node: NodeId) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(base_j, &mut buf);
    encode_judgment(step_j, &mut buf);
    encode_node_id(result_node, &mut buf);
    call_lean_rule(RULE_NAT_IND, &buf)
}

/// Lean kernel rule 15: let_bind
#[cfg(feature = "lean-ffi")]
pub fn lean_let_bind(
    ctx: &Context, let_node: NodeId, binder_name: BinderId,
    bound_j: &Judgment, body_j: &Judgment,
) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(256);
    encode_context(ctx, &mut buf);
    encode_node_id(let_node, &mut buf);
    encode_binder_id(binder_name, &mut buf);
    encode_judgment(bound_j, &mut buf);
    encode_judgment(body_j, &mut buf);
    call_lean_rule(RULE_LET_BIND, &buf)
}

/// Lean kernel rule 16: match_elim
#[cfg(feature = "lean-ffi")]
pub fn lean_match_elim(
    scrutinee_j: &Judgment, arm_js: &[Judgment], match_node: NodeId,
) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(256);
    encode_judgment(scrutinee_j, &mut buf);
    buf.extend_from_slice(&(arm_js.len() as u16).to_le_bytes());
    for arm in arm_js {
        encode_judgment(arm, &mut buf);
    }
    encode_node_id(match_node, &mut buf);
    call_lean_rule(RULE_MATCH_ELIM, &buf)
}

/// Lean kernel rule 17: fold_rule
#[cfg(feature = "lean-ffi")]
pub fn lean_fold_rule(
    base_j: &Judgment, step_j: &Judgment, input_j: &Judgment, fold_node: NodeId,
) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(256);
    encode_judgment(base_j, &mut buf);
    encode_judgment(step_j, &mut buf);
    encode_judgment(input_j, &mut buf);
    encode_node_id(fold_node, &mut buf);
    call_lean_rule(RULE_FOLD_RULE, &buf)
}

/// Lean kernel rule 20: guard_rule
#[cfg(feature = "lean-ffi")]
pub fn lean_guard_rule(
    pred_j: &Judgment, then_j: &Judgment, else_j: &Judgment, guard_node: NodeId,
) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(256);
    encode_judgment(pred_j, &mut buf);
    encode_judgment(then_j, &mut buf);
    encode_judgment(else_j, &mut buf);
    encode_node_id(guard_node, &mut buf);
    call_lean_rule(RULE_GUARD_RULE, &buf)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_cost_zero() {
        let mut buf = Vec::new();
        encode_cost_bound(&CostBound::Zero, &mut buf);
        assert_eq!(buf, vec![0x01]);
    }

    #[test]
    fn test_encode_cost_unknown() {
        let mut buf = Vec::new();
        encode_cost_bound(&CostBound::Unknown, &mut buf);
        assert_eq!(buf, vec![0x00]);
    }

    #[test]
    fn test_encode_cost_constant() {
        let mut buf = Vec::new();
        encode_cost_bound(&CostBound::Constant(42), &mut buf);
        assert_eq!(buf[0], 0x02);
        assert_eq!(u64::from_le_bytes(buf[1..9].try_into().unwrap()), 42);
    }

    #[test]
    fn test_encode_cost_sum() {
        let mut buf = Vec::new();
        let cost = CostBound::Sum(
            Box::new(CostBound::Zero),
            Box::new(CostBound::Constant(5)),
        );
        encode_cost_bound(&cost, &mut buf);
        assert_eq!(buf[0], 0x06); // Sum tag
        assert_eq!(buf[1], 0x01); // Zero tag
        assert_eq!(buf[2], 0x02); // Constant tag
    }

    #[test]
    fn test_encode_cost_par() {
        let mut buf = Vec::new();
        let cost = CostBound::Par(
            Box::new(CostBound::Zero),
            Box::new(CostBound::Constant(1)),
        );
        encode_cost_bound(&cost, &mut buf);
        assert_eq!(buf[0], 0x07); // Par tag
    }

    #[test]
    fn test_encode_cost_mul() {
        let mut buf = Vec::new();
        let cost = CostBound::Mul(
            Box::new(CostBound::Constant(2)),
            Box::new(CostBound::Constant(3)),
        );
        encode_cost_bound(&cost, &mut buf);
        assert_eq!(buf[0], 0x08); // Mul tag
    }

    #[test]
    fn test_lean_fallback_matches_rust() {
        // When lean-ffi is enabled and the server is available, this tests
        // the Lean kernel via IPC. Otherwise it tests the Rust fallback.
        // Both implementations agree on these cases:
        assert!(lean_check_cost_leq(&CostBound::Zero, &CostBound::Constant(5)));
        assert!(lean_check_cost_leq(&CostBound::Constant(3), &CostBound::Constant(5)));
        assert!(!lean_check_cost_leq(&CostBound::Constant(10), &CostBound::Constant(5)));
        assert!(lean_check_cost_leq(
            &CostBound::Constant(1),
            &CostBound::Linear(CostVar(0)),
        ));
    }

    /// Test Rust-specific cost_leq behavior for Unknown.
    /// Only runs without lean-ffi since the Lean kernel has different
    /// semantics for Unknown (it accepts Zero <= Unknown).
    #[cfg(not(feature = "lean-ffi"))]
    #[test]
    fn test_rust_unknown_not_upper_bound() {
        // Unknown is no longer a valid upper bound in Rust (soundness fix).
        assert!(!lean_check_cost_leq(&CostBound::Zero, &CostBound::Unknown));
    }
}
