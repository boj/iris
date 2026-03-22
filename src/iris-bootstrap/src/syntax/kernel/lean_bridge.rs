//! Lean 4 FFI bridge — calls proven kernel functions compiled from Lean.
//!
//! The Lean code at `lean/IrisKernel/` IS the formal proof. This module
//! calls the compiled Lean functions via C FFI, so the running code is
//! the proven code.

use crate::syntax::kernel::cost_checker;
use iris_types::cost::{CostBound, CostVar};

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
        CostBound::Sup(costs) => {
            buf.push(0x07);
            buf.extend_from_slice(&(costs.len() as u32).to_le_bytes());
            for c in costs {
                encode_cost_bound(c, buf);
            }
        }
        CostBound::Inf(costs) => {
            buf.push(0x08);
            buf.extend_from_slice(&(costs.len() as u32).to_le_bytes());
            for c in costs {
                encode_cost_bound(c, buf);
            }
        }
        CostBound::Par(a, b) => {
            buf.push(0x09);
            encode_cost_bound(a, buf);
            encode_cost_bound(b, buf);
        }
        CostBound::Mul(a, b) => {
            buf.push(0x0A);
            encode_cost_bound(a, buf);
            encode_cost_bound(b, buf);
        }
        // KNOWN GAP: `Amortized` and `HWScaled` are not in the Lean
        // formalization (lean/IrisKernel/FFI.lean decodeCostBound). Lean never
        // sees these variants — they are silently approximated by their inner
        // cost bound. This means:
        //   - `Amortized(O(n), proof)` is checked as `O(n)` (ignoring amort.)
        //   - `HWScaled(O(n), hw)` is checked as `O(n)` (ignoring hw factor)
        //
        // Impact: cost-ordering proofs for these bounds are under-constrained.
        // A future fix should extend the Lean formalization to handle these
        // variants explicitly, or map them to conservative upper bounds.
        CostBound::Amortized(inner, _) => encode_cost_bound(inner, buf),
        CostBound::HWScaled(inner, _) => encode_cost_bound(inner, buf),
    }
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
