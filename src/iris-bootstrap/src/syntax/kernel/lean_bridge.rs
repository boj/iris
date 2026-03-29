//! Lean 4 FFI bridge — calls proven kernel functions compiled from Lean.
//!
//! The Lean code at `lean/IrisKernel/` IS the formal proof. This module
//! calls the compiled Lean functions via C FFI, so the running code is
//! the proven code. There is no Rust fallback.

use crate::syntax::kernel::theorem::{Binding, Context, Judgment};
use iris_types::cost::{CostBound, CostVar};
use iris_types::graph::{BinderId, NodeId};
use iris_types::types::TypeId;

// ---------------------------------------------------------------------------
// Lean runtime initialization
// ---------------------------------------------------------------------------

unsafe extern "C" {
    fn iris_lean_init();
    fn iris_lean_is_initialized() -> i32;
    fn iris_check_cost_leq_bytes(data: *const u8, len: usize) -> u8;
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

static LEAN_INITIALIZED: std::sync::Once = std::sync::Once::new();

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

pub fn encode_node_id(node: NodeId, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&node.0.to_le_bytes());
}

pub fn encode_type_id(ty: TypeId, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&ty.0.to_le_bytes());
}

pub fn encode_binder_id(binder: BinderId, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&binder.0.to_le_bytes());
}

pub fn encode_context(ctx: &Context, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&(ctx.bindings.len() as u16).to_le_bytes());
    for binding in &ctx.bindings {
        encode_binder_id(binding.name, buf);
        encode_type_id(binding.type_id, buf);
    }
}

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

pub fn decode_node_id(data: &[u8], offset: usize) -> Option<(NodeId, usize)> {
    if offset + 8 > data.len() { return None; }
    let val = u64::from_le_bytes(data[offset..offset+8].try_into().ok()?);
    Some((NodeId(val), offset + 8))
}

pub fn decode_type_id(data: &[u8], offset: usize) -> Option<(TypeId, usize)> {
    if offset + 8 > data.len() { return None; }
    let val = u64::from_le_bytes(data[offset..offset+8].try_into().ok()?);
    Some((TypeId(val), offset + 8))
}

pub fn decode_binder_id(data: &[u8], offset: usize) -> Option<(BinderId, usize)> {
    if offset + 4 > data.len() { return None; }
    let val = u32::from_le_bytes(data[offset..offset+4].try_into().ok()?);
    Some((BinderId(val), offset + 4))
}

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

pub fn decode_judgment(data: &[u8], offset: usize) -> Option<(Judgment, usize)> {
    let (context, pos) = decode_context(data, offset)?;
    let (node_id, pos) = decode_node_id(data, pos)?;
    let (type_ref, pos) = decode_type_id(data, pos)?;
    let (cost, pos) = decode_cost_bound(data, pos)?;
    Some((Judgment { context, node_id, type_ref, cost }, pos))
}

pub fn decode_lean_result(data: &[u8]) -> Option<Judgment> {
    if data.is_empty() { return None; }
    if data[0] != 1 { return None; }
    let (j, _) = decode_judgment(data, 1)?;
    Some(j)
}

// ---------------------------------------------------------------------------
// Generic FFI call helper
// ---------------------------------------------------------------------------

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
// Public API — all calls go through Lean FFI
// ---------------------------------------------------------------------------

pub fn lean_check_cost_leq(a: &CostBound, b: &CostBound) -> bool {
    ensure_lean_initialized();
    let mut buf = Vec::with_capacity(64);
    encode_cost_bound(a, &mut buf);
    encode_cost_bound(b, &mut buf);
    unsafe { iris_check_cost_leq_bytes(buf.as_ptr(), buf.len()) == 1 }
}

/// Rule 1: assume
pub fn lean_assume(ctx: &Context, name: BinderId, node_id: NodeId) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(64);
    encode_context(ctx, &mut buf);
    encode_binder_id(name, &mut buf);
    encode_node_id(node_id, &mut buf);
    call_lean_rule(iris_kernel_assume_bytes, &buf)
}

/// Rule 2: intro
pub fn lean_intro(
    ctx: &Context, lam_node: NodeId, binder_name: BinderId, binder_type: TypeId,
    body_judgment: &Judgment, arrow_id: TypeId,
    type_env_bytes: &[u8],
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

/// Rule 3: elim
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

/// Rule 4: refl
pub fn lean_refl(ctx: &Context, node_id: NodeId, type_id: TypeId) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(64);
    encode_context(ctx, &mut buf);
    encode_node_id(node_id, &mut buf);
    encode_type_id(type_id, &mut buf);
    call_lean_rule(iris_kernel_refl_bytes, &buf)
}

/// Rule 5: symm
pub fn lean_symm(thm: &Judgment, other_node: NodeId, eq_witness: &Judgment) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(thm, &mut buf);
    encode_node_id(other_node, &mut buf);
    encode_judgment(eq_witness, &mut buf);
    call_lean_rule(iris_kernel_symm_bytes, &buf)
}

/// Rule 6: trans
pub fn lean_trans(thm1: &Judgment, thm2: &Judgment) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(thm1, &mut buf);
    encode_judgment(thm2, &mut buf);
    call_lean_rule(iris_kernel_trans_bytes, &buf)
}

/// Rule 7: congr
pub fn lean_congr(fn_j: &Judgment, arg_j: &Judgment, app_node: NodeId) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(fn_j, &mut buf);
    encode_judgment(arg_j, &mut buf);
    encode_node_id(app_node, &mut buf);
    call_lean_rule(iris_kernel_congr_bytes, &buf)
}

/// Rule 8: type_check_node
pub fn lean_type_check_node(
    ctx: &Context, node_id: NodeId, type_env_bytes: &[u8],
) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(type_env_bytes);
    encode_context(ctx, &mut buf);
    encode_node_id(node_id, &mut buf);
    call_lean_rule(iris_kernel_type_check_node_full_bytes, &buf)
}

/// Rule 9: cost_subsume
pub fn lean_cost_subsume(j: &Judgment, new_cost: &CostBound) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(j, &mut buf);
    encode_cost_bound(new_cost, &mut buf);
    call_lean_rule(iris_kernel_cost_subsume_bytes, &buf)
}

/// Rule 10: cost_leq_rule
pub fn lean_cost_leq_rule(k1: &CostBound, k2: &CostBound) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(64);
    encode_cost_bound(k1, &mut buf);
    encode_cost_bound(k2, &mut buf);
    call_lean_rule(iris_kernel_cost_leq_rule_bytes, &buf)
}

/// Rule 11: refine_intro
pub fn lean_refine_intro(
    base_j: &Judgment, pred_j: &Judgment, refined_type: TypeId,
    type_env_bytes: &[u8],
) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(type_env_bytes);
    encode_judgment(base_j, &mut buf);
    encode_judgment(pred_j, &mut buf);
    encode_type_id(refined_type, &mut buf);
    call_lean_rule(iris_kernel_refine_intro_bytes, &buf)
}

/// Rule 12: refine_elim
pub fn lean_refine_elim(
    refined_j: &Judgment, type_env_bytes: &[u8],
) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(type_env_bytes);
    encode_judgment(refined_j, &mut buf);
    call_lean_rule(iris_kernel_refine_elim_bytes, &buf)
}

/// Rule 13: nat_ind
pub fn lean_nat_ind(base_j: &Judgment, step_j: &Judgment, result_node: NodeId) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(128);
    encode_judgment(base_j, &mut buf);
    encode_judgment(step_j, &mut buf);
    encode_node_id(result_node, &mut buf);
    call_lean_rule(iris_kernel_nat_ind_bytes, &buf)
}

/// Rule 14: structural_ind
pub fn lean_structural_ind(
    case_js: &[Judgment], result_type: TypeId, result_node: NodeId,
) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(&(case_js.len() as u16).to_le_bytes());
    for case in case_js {
        encode_judgment(case, &mut buf);
    }
    encode_type_id(result_type, &mut buf);
    encode_node_id(result_node, &mut buf);
    call_lean_rule(iris_kernel_structural_ind_bytes, &buf)
}

/// Rule 15: let_bind
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

/// Rule 16: match_elim
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

/// Rule 17: fold_rule
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

/// Rule 18: type_abst
pub fn lean_type_abst(
    body_j: &Judgment, forall_type: TypeId, forall_node: NodeId,
    type_env_bytes: &[u8],
) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(type_env_bytes);
    encode_judgment(body_j, &mut buf);
    encode_type_id(forall_type, &mut buf);
    encode_node_id(forall_node, &mut buf);
    call_lean_rule(iris_kernel_type_abst_bytes, &buf)
}

/// Rule 19: type_app
pub fn lean_type_app(
    forall_j: &Judgment, applied_type: TypeId, result_node: NodeId,
    type_env_bytes: &[u8],
) -> Option<Judgment> {
    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(type_env_bytes);
    encode_judgment(forall_j, &mut buf);
    encode_type_id(applied_type, &mut buf);
    encode_node_id(result_node, &mut buf);
    call_lean_rule(iris_kernel_type_app_bytes, &buf)
}

/// Rule 20: guard_rule
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
        assert_eq!(buf[0], 0x06);
        assert_eq!(buf[1], 0x01);
        assert_eq!(buf[2], 0x02);
    }

    #[test]
    fn test_encode_cost_par() {
        let mut buf = Vec::new();
        let cost = CostBound::Par(
            Box::new(CostBound::Zero),
            Box::new(CostBound::Constant(1)),
        );
        encode_cost_bound(&cost, &mut buf);
        assert_eq!(buf[0], 0x07);
    }

    #[test]
    fn test_encode_cost_mul() {
        let mut buf = Vec::new();
        let cost = CostBound::Mul(
            Box::new(CostBound::Constant(2)),
            Box::new(CostBound::Constant(3)),
        );
        encode_cost_bound(&cost, &mut buf);
        assert_eq!(buf[0], 0x08);
    }

    #[test]
    fn test_encode_decode_node_id() {
        let mut buf = Vec::new();
        encode_node_id(NodeId(42), &mut buf);
        let (decoded, _) = decode_node_id(&buf, 0).unwrap();
        assert_eq!(decoded, NodeId(42));
    }

    #[test]
    fn test_encode_decode_type_id() {
        let mut buf = Vec::new();
        encode_type_id(TypeId(99), &mut buf);
        let (decoded, _) = decode_type_id(&buf, 0).unwrap();
        assert_eq!(decoded, TypeId(99));
    }

    #[test]
    fn test_encode_decode_binder_id() {
        let mut buf = Vec::new();
        encode_binder_id(BinderId(7), &mut buf);
        let (decoded, _) = decode_binder_id(&buf, 0).unwrap();
        assert_eq!(decoded, BinderId(7));
    }

    #[test]
    fn test_encode_decode_context() {
        let ctx = Context {
            bindings: vec![
                Binding { name: BinderId(1), type_id: TypeId(10) },
                Binding { name: BinderId(2), type_id: TypeId(20) },
            ],
        };
        let mut buf = Vec::new();
        encode_context(&ctx, &mut buf);
        let (decoded, _) = decode_context(&buf, 0).unwrap();
        assert_eq!(decoded.bindings.len(), 2);
        assert_eq!(decoded.bindings[0].name, BinderId(1));
        assert_eq!(decoded.bindings[1].type_id, TypeId(20));
    }

    #[test]
    fn test_encode_decode_judgment() {
        let j = Judgment {
            context: Context { bindings: vec![] },
            node_id: NodeId(5),
            type_ref: TypeId(10),
            cost: CostBound::Zero,
        };
        let mut buf = Vec::new();
        encode_judgment(&j, &mut buf);
        let (decoded, _) = decode_judgment(&buf, 0).unwrap();
        assert_eq!(decoded.node_id, NodeId(5));
        assert_eq!(decoded.type_ref, TypeId(10));
    }

    #[test]
    fn test_decode_lean_result_success() {
        let j = Judgment {
            context: Context { bindings: vec![] },
            node_id: NodeId(1),
            type_ref: TypeId(2),
            cost: CostBound::Zero,
        };
        let mut buf = vec![1u8]; // success byte
        encode_judgment(&j, &mut buf);
        let decoded = decode_lean_result(&buf).unwrap();
        assert_eq!(decoded.node_id, NodeId(1));
    }

    #[test]
    fn test_decode_lean_result_failure() {
        let buf = vec![0u8]; // failure byte
        assert!(decode_lean_result(&buf).is_none());
    }

    #[test]
    fn test_decode_lean_result_empty() {
        assert!(decode_lean_result(&[]).is_none());
    }

    #[test]
    fn test_encode_cost_amortized() {
        let mut buf = Vec::new();
        encode_cost_bound(
            &CostBound::Amortized(
                Box::new(CostBound::Constant(10)),
                iris_types::cost::PotentialFn { description: String::new() },
            ),
            &mut buf,
        );
        assert_eq!(buf[0], 0x0B);
    }

    #[test]
    fn test_encode_cost_hwscaled() {
        let mut buf = Vec::new();
        encode_cost_bound(
            &CostBound::HWScaled(
                Box::new(CostBound::Constant(5)),
                iris_types::cost::HWParamRef([0; 32]),
            ),
            &mut buf,
        );
        assert_eq!(buf[0], 0x0C);
    }

    #[test]
    fn test_cost_round_trip() {
        let cost = CostBound::Sum(
            Box::new(CostBound::Linear(CostVar(0))),
            Box::new(CostBound::Mul(
                Box::new(CostBound::Constant(3)),
                Box::new(CostBound::NLogN(CostVar(1))),
            )),
        );
        let mut buf = Vec::new();
        encode_cost_bound(&cost, &mut buf);
        let (decoded, _) = decode_cost_bound(&buf, 0).unwrap();
        // Re-encode and compare bytes
        let mut buf2 = Vec::new();
        encode_cost_bound(&decoded, &mut buf2);
        assert_eq!(buf, buf2);
    }
}
