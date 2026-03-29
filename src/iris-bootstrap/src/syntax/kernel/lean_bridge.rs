//! Lean 4 FFI bridge — calls proven kernel functions compiled from Lean.
//!
//! The Lean code at `lean/IrisKernel/` IS the formal proof. This module
//! calls the compiled Lean functions via C FFI, so the running code is
//! the proven code.

use crate::syntax::kernel::cost_checker;
use crate::syntax::kernel::theorem::{Binding, Context, Judgment};
use iris_types::cost::{CostBound, CostVar};
use iris_types::graph::{BinderId, NodeId};
use iris_types::types::TypeId;

// ---------------------------------------------------------------------------
// Lean runtime initialization
// ---------------------------------------------------------------------------

#[cfg(feature = "lean-ffi")]
unsafe extern "C" {
    // From lean_shim.c — handles all Lean runtime initialization
    fn iris_lean_init();
    fn iris_lean_is_initialized() -> i32;

    // From our C shim (lean_shim.c) — handles Lean object creation internally
    fn iris_check_cost_leq_bytes(data: *const u8, len: usize) -> u8;

    // Free bytes returned by kernel rule wrappers
    fn iris_lean_free_bytes(ptr: *mut u8);

    // Kernel rule wrappers (each takes ByteArray, returns ByteArray)
    fn iris_kernel_assume_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_intro_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_elim_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_refl_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_symm_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_trans_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_congr_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_type_check_node_full_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_cost_subsume_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_cost_leq_rule_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_refine_intro_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_refine_elim_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_nat_ind_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_structural_ind_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_let_bind_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_match_elim_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_fold_rule_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_type_abst_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_type_app_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
    fn iris_kernel_guard_rule_bytes(data: *const u8, len: usize, out_data: *mut *mut u8, out_len: *mut usize) -> i32;
}

#[cfg(feature = "lean-ffi")]
static LEAN_INITIALIZED: std::sync::Once = std::sync::Once::new();

#[cfg(feature = "lean-ffi")]
fn ensure_lean_initialized() {
    LEAN_INITIALIZED.call_once(|| {
        unsafe { iris_lean_init(); }
    });
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
// Public API — calls Lean when linked, falls back to Rust otherwise
// ---------------------------------------------------------------------------

/// Check if cost bound `a` is less than or equal to `b`.
///
/// When linked with the Lean library, this calls the formally proven
/// `checkCostLeq` function. Otherwise falls back to the Rust implementation.
#[cfg(feature = "lean-ffi")]
pub fn lean_check_cost_leq(a: &CostBound, b: &CostBound) -> bool {
    ensure_lean_initialized();

    let mut buf = Vec::with_capacity(64);
    encode_cost_bound(a, &mut buf);
    encode_cost_bound(b, &mut buf);

    unsafe {
        let result = iris_check_cost_leq_bytes(buf.as_ptr(), buf.len());
        result == 1
    }
}

#[cfg(not(feature = "lean-ffi"))]
pub fn lean_check_cost_leq(a: &CostBound, b: &CostBound) -> bool {
    // Fallback to Rust implementation
    cost_checker::cost_leq(a, b)
}

// ---------------------------------------------------------------------------
// Generic FFI call helper (lean-ffi only)
// ---------------------------------------------------------------------------

/// Call a Lean kernel rule via FFI. Serializes input, calls the C wrapper,
/// deserializes the result Judgment.
#[cfg(feature = "lean-ffi")]
fn call_lean_rule(
    rule_fn: unsafe extern "C" fn(*const u8, usize, *mut *mut u8, *mut usize) -> i32,
    buf: &[u8],
) -> Option<Judgment> {
    ensure_lean_initialized();
    let mut out_data: *mut u8 = std::ptr::null_mut();
    let mut out_len: usize = 0;
    let rc = unsafe { rule_fn(buf.as_ptr(), buf.len(), &mut out_data, &mut out_len) };
    if rc != 0 || out_data.is_null() || out_len == 0 {
        return None;
    }
    let result_bytes = unsafe { std::slice::from_raw_parts(out_data, out_len) };
    let result = decode_lean_result(result_bytes);
    unsafe { iris_lean_free_bytes(out_data); }
    result
}

// ---------------------------------------------------------------------------
// Public Lean kernel rule functions (lean-kernel feature)
// ---------------------------------------------------------------------------

/// Lean kernel rule 1: assume
#[cfg(feature = "lean-ffi")]
pub fn lean_assume(ctx: &Context, name: BinderId, node_id: NodeId) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(64);
    encode_context(ctx, &mut buf);
    encode_binder_id(name, &mut buf);
    encode_node_id(node_id, &mut buf);
    call_lean_rule(iris_kernel_assume_bytes, &buf)
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
    call_lean_rule(iris_kernel_intro_bytes, &buf)
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
    call_lean_rule(iris_kernel_elim_bytes, &buf)
}

/// Lean kernel rule 4: refl
#[cfg(feature = "lean-ffi")]
pub fn lean_refl(ctx: &Context, node_id: NodeId, type_id: TypeId) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(64);
    encode_context(ctx, &mut buf);
    encode_node_id(node_id, &mut buf);
    encode_type_id(type_id, &mut buf);
    call_lean_rule(iris_kernel_refl_bytes, &buf)
}

/// Lean kernel rule 5: symm
#[cfg(feature = "lean-ffi")]
pub fn lean_symm(thm: &Judgment, other_node: NodeId, eq_witness: &Judgment) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(thm, &mut buf);
    encode_node_id(other_node, &mut buf);
    encode_judgment(eq_witness, &mut buf);
    call_lean_rule(iris_kernel_symm_bytes, &buf)
}

/// Lean kernel rule 6: trans
#[cfg(feature = "lean-ffi")]
pub fn lean_trans(thm1: &Judgment, thm2: &Judgment) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(thm1, &mut buf);
    encode_judgment(thm2, &mut buf);
    call_lean_rule(iris_kernel_trans_bytes, &buf)
}

/// Lean kernel rule 7: congr
#[cfg(feature = "lean-ffi")]
pub fn lean_congr(fn_j: &Judgment, arg_j: &Judgment, app_node: NodeId) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(fn_j, &mut buf);
    encode_judgment(arg_j, &mut buf);
    encode_node_id(app_node, &mut buf);
    call_lean_rule(iris_kernel_congr_bytes, &buf)
}

/// Lean kernel rule 9: cost_subsume
#[cfg(feature = "lean-ffi")]
pub fn lean_cost_subsume(j: &Judgment, new_cost: &CostBound) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(j, &mut buf);
    encode_cost_bound(new_cost, &mut buf);
    call_lean_rule(iris_kernel_cost_subsume_bytes, &buf)
}

/// Lean kernel rule 10: cost_leq_rule
#[cfg(feature = "lean-ffi")]
pub fn lean_cost_leq_rule(k1: &CostBound, k2: &CostBound) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(64);
    encode_cost_bound(k1, &mut buf);
    encode_cost_bound(k2, &mut buf);
    call_lean_rule(iris_kernel_cost_leq_rule_bytes, &buf)
}

/// Lean kernel rule 13: nat_ind
#[cfg(feature = "lean-ffi")]
pub fn lean_nat_ind(base_j: &Judgment, step_j: &Judgment, result_node: NodeId) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(base_j, &mut buf);
    encode_judgment(step_j, &mut buf);
    encode_node_id(result_node, &mut buf);
    call_lean_rule(iris_kernel_nat_ind_bytes, &buf)
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
    call_lean_rule(iris_kernel_let_bind_bytes, &buf)
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
    call_lean_rule(iris_kernel_match_elim_bytes, &buf)
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
    call_lean_rule(iris_kernel_fold_rule_bytes, &buf)
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
    call_lean_rule(iris_kernel_guard_rule_bytes, &buf)
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
        // These use the Rust fallback (no lean-ffi feature)
        // Unknown is no longer a valid upper bound (soundness fix).
        assert!(!lean_check_cost_leq(&CostBound::Zero, &CostBound::Unknown));
        assert!(lean_check_cost_leq(&CostBound::Zero, &CostBound::Constant(5)));
        assert!(lean_check_cost_leq(&CostBound::Constant(3), &CostBound::Constant(5)));
        assert!(!lean_check_cost_leq(&CostBound::Constant(10), &CostBound::Constant(5)));
        assert!(lean_check_cost_leq(
            &CostBound::Constant(1),
            &CostBound::Linear(CostVar(0)),
        ));
    }
}
