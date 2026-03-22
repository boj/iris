//! Cost algebra checker.
//!
//! Implements the partial order on `CostBound` values. This is called BY the
//! kernel, so correctness here is trust-critical.
//!
//! Partial order:
//!   Zero <= Constant(k) <= Linear(n) <= NLogN(n) <= Polynomial(n, d)
//!
//! Composite forms:
//!   Sum(a, b) <= Sum(a', b')  when a <= a' and b <= b'
//!   Par(a, b) <= Par(a', b')  when a <= a' and b <= b'
//!   Mul(a, b) <= Mul(a', b')  when a <= a' and b <= b'
//!   Sup(vs)   <= x            when ALL v in vs satisfy v <= x
//!   x         <= Inf(vs)      when x <= ALL v in vs (pessimistic: any v)

use iris_types::cost::CostBound;

/// Returns `true` if `a <= b` in the cost partial order.
///
/// This is a conservative check: if we cannot determine the relationship,
/// we return `false` (i.e., we reject rather than accept uncertain orderings).
pub fn cost_leq(a: &CostBound, b: &CostBound) -> bool {
    // Identical costs are always ordered.
    if a == b {
        return true;
    }

    // Unknown: only Unknown <= Unknown is valid (handled by equality above).
    // Programs cannot claim Unknown cost to bypass the cost system.
    // Unknown <= x is false (cannot subsume an unknown cost).
    // x <= Unknown is also false (Unknown is not a valid upper bound).
    if matches!(a, CostBound::Unknown) || matches!(b, CostBound::Unknown) {
        return false;
    }

    // Try to evaluate composite costs to constants before structural matching.
    // This handles cases like Constant(5) <= Sum(Constant(3), Constant(2)).
    if let Some(va) = eval_constant(a) {
        if let Some(vb) = eval_constant(b) {
            return va <= vb;
        }
    }

    match (a, b) {
        // --- Base cases in the lattice ---
        (CostBound::Zero, _) => true,

        (CostBound::Constant(_), CostBound::Zero) => false,
        (CostBound::Constant(k1), CostBound::Constant(k2)) => k1 <= k2,
        (CostBound::Constant(_), _) => {
            // Constant <= Linear, NLogN, Polynomial, Unknown
            is_at_least_linear(b)
        }

        (CostBound::Linear(_), CostBound::Zero | CostBound::Constant(_)) => false,
        (CostBound::Linear(v1), CostBound::Linear(v2)) => v1 == v2,
        (CostBound::Linear(v1), CostBound::NLogN(v2)) => v1 == v2,
        (CostBound::Linear(v1), CostBound::Polynomial(v2, _)) => v1 == v2,

        (CostBound::NLogN(_), CostBound::Zero | CostBound::Constant(_) | CostBound::Linear(_)) => {
            false
        }
        (CostBound::NLogN(v1), CostBound::NLogN(v2)) => v1 == v2,
        (CostBound::NLogN(v1), CostBound::Polynomial(v2, d)) => v1 == v2 && *d >= 2,

        (
            CostBound::Polynomial(..),
            CostBound::Zero
            | CostBound::Constant(_)
            | CostBound::Linear(_)
            | CostBound::NLogN(_),
        ) => false,
        (CostBound::Polynomial(v1, d1), CostBound::Polynomial(v2, d2)) => {
            v1 == v2 && d1 <= d2
        }

        // --- Composite forms ---
        (CostBound::Sum(a1, a2), CostBound::Sum(b1, b2)) => {
            cost_leq(a1, b1) && cost_leq(a2, b2)
        }
        (CostBound::Par(a1, a2), CostBound::Par(b1, b2)) => {
            cost_leq(a1, b1) && cost_leq(a2, b2)
        }
        (CostBound::Mul(a1, a2), CostBound::Mul(b1, b2)) => {
            cost_leq(a1, b1) && cost_leq(a2, b2)
        }

        // Sup: a Sup is <= b if ALL branches are <= b.
        (CostBound::Sup(branches), _) => branches.iter().all(|branch| cost_leq(branch, b)),

        // Anything <= Sup if it's <= at least one branch.
        (_, CostBound::Sup(branches)) => branches.iter().any(|branch| cost_leq(a, branch)),

        // Inf: a <= Inf(branches) if a <= ALL branches.
        (_, CostBound::Inf(branches)) => branches.iter().all(|branch| cost_leq(a, branch)),

        // Inf(branches) <= b if ANY branch <= b.
        (CostBound::Inf(branches), _) => branches.iter().any(|branch| cost_leq(branch, b)),

        // Amortized: conservative — treat as the underlying cost.
        (CostBound::Amortized(inner, _), _) => cost_leq(inner, b),
        (_, CostBound::Amortized(inner, _)) => cost_leq(a, inner),

        // HWScaled: can only compare same-profile scaled costs.
        (CostBound::HWScaled(inner_a, ref_a), CostBound::HWScaled(inner_b, ref_b)) => {
            ref_a == ref_b && cost_leq(inner_a, inner_b)
        }

        // Evaluate composite costs: try to reduce Sum/Mul of constants
        // to a single constant for comparison.
        (CostBound::Sum(a1, a2), _) => {
            // Try to evaluate Sum(a1, a2) if both are constants.
            if let (Some(v1), Some(v2)) = (eval_constant(a1), eval_constant(a2)) {
                cost_leq(&CostBound::Constant(v1.saturating_add(v2)), b)
            } else {
                // Conservative: a <= b if both components <= b
                // (since Sum(a1,a2) >= max(a1,a2) but we can't always evaluate)
                false
            }
        }
        (_, CostBound::Sum(b1, b2)) => {
            // Try to evaluate Sum(b1, b2) if both are constants.
            if let (Some(v1), Some(v2)) = (eval_constant(b1), eval_constant(b2)) {
                cost_leq(a, &CostBound::Constant(v1.saturating_add(v2)))
            } else {
                // a <= Sum(b1, b2) when one component is Zero.
                (matches!(b2.as_ref(), CostBound::Zero) && cost_leq(a, b1))
                    || (matches!(b1.as_ref(), CostBound::Zero) && cost_leq(a, b2))
            }
        }
        (CostBound::Mul(a1, a2), _) => {
            if let (Some(v1), Some(v2)) = (eval_constant(a1), eval_constant(a2)) {
                cost_leq(&CostBound::Constant(v1.saturating_mul(v2)), b)
            } else {
                false
            }
        }
        (_, CostBound::Mul(b1, b2)) => {
            if let (Some(v1), Some(v2)) = (eval_constant(b1), eval_constant(b2)) {
                cost_leq(a, &CostBound::Constant(v1.saturating_mul(v2)))
            } else {
                false
            }
        }
        (_, CostBound::Par(b1, b2)) => {
            (matches!(b2.as_ref(), CostBound::Zero) && cost_leq(a, b1))
                || (matches!(b1.as_ref(), CostBound::Zero) && cost_leq(a, b2))
        }

        // Fall through: cannot determine ordering.
        _ => false,
    }
}

/// Try to evaluate a `CostBound` to a constant value. Returns `Some(k)` if
/// the cost can be statically reduced to `Constant(k)`, `None` otherwise.
fn eval_constant(c: &CostBound) -> Option<u64> {
    match c {
        CostBound::Zero => Some(0),
        CostBound::Constant(k) => Some(*k),
        CostBound::Sum(a, b) => {
            let va = eval_constant(a)?;
            let vb = eval_constant(b)?;
            Some(va.saturating_add(vb))
        }
        CostBound::Mul(a, b) => {
            let va = eval_constant(a)?;
            let vb = eval_constant(b)?;
            Some(va.saturating_mul(vb))
        }
        _ => None,
    }
}

/// Returns `true` if the cost bound is at least `Linear` in the lattice
/// (Linear, NLogN, or Polynomial).
fn is_at_least_linear(c: &CostBound) -> bool {
    matches!(
        c,
        CostBound::Linear(_) | CostBound::NLogN(_) | CostBound::Polynomial(_, _)
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use iris_types::cost::CostVar;

    #[test]
    fn zero_leq_everything_except_unknown() {
        let n = CostVar(0);
        assert!(cost_leq(&CostBound::Zero, &CostBound::Zero));
        assert!(cost_leq(&CostBound::Zero, &CostBound::Constant(42)));
        assert!(cost_leq(&CostBound::Zero, &CostBound::Linear(n)));
        // Unknown is NOT a valid upper bound (programs can't claim Unknown cost).
        assert!(!cost_leq(&CostBound::Zero, &CostBound::Unknown));
    }

    #[test]
    fn constant_ordering() {
        assert!(cost_leq(&CostBound::Constant(1), &CostBound::Constant(2)));
        assert!(cost_leq(&CostBound::Constant(2), &CostBound::Constant(2)));
        assert!(!cost_leq(&CostBound::Constant(3), &CostBound::Constant(2)));
    }

    #[test]
    fn lattice_order() {
        let n = CostVar(0);
        assert!(cost_leq(&CostBound::Constant(100), &CostBound::Linear(n)));
        assert!(cost_leq(&CostBound::Linear(n), &CostBound::NLogN(n)));
        assert!(cost_leq(
            &CostBound::NLogN(n),
            &CostBound::Polynomial(n, 2)
        ));
        assert!(cost_leq(
            &CostBound::Polynomial(n, 2),
            &CostBound::Polynomial(n, 3)
        ));
    }

    #[test]
    fn lattice_not_leq() {
        let n = CostVar(0);
        assert!(!cost_leq(&CostBound::Linear(n), &CostBound::Constant(100)));
        assert!(!cost_leq(&CostBound::NLogN(n), &CostBound::Linear(n)));
        assert!(!cost_leq(
            &CostBound::Polynomial(n, 3),
            &CostBound::Polynomial(n, 2)
        ));
    }

    #[test]
    fn sup_leq() {
        let n = CostVar(0);
        let sup = CostBound::Sup(vec![CostBound::Constant(5), CostBound::Constant(10)]);
        assert!(cost_leq(&sup, &CostBound::Constant(10)));
        assert!(cost_leq(&sup, &CostBound::Linear(n)));
        assert!(!cost_leq(&sup, &CostBound::Constant(5)));
    }

    #[test]
    fn unknown_only_equals_itself() {
        let n = CostVar(0);
        // Unknown is not a valid upper bound for anything except itself.
        assert!(!cost_leq(&CostBound::Linear(n), &CostBound::Unknown));
        assert!(!cost_leq(&CostBound::Unknown, &CostBound::Linear(n)));
        // Unknown == Unknown is the only valid case (handled by equality).
        assert!(cost_leq(&CostBound::Unknown, &CostBound::Unknown));
    }

    #[test]
    fn sum_of_constants_evaluates() {
        // Constant(5) <= Sum(Constant(3), Constant(2)) should now be true.
        let sum = CostBound::Sum(
            Box::new(CostBound::Constant(3)),
            Box::new(CostBound::Constant(2)),
        );
        assert!(cost_leq(&CostBound::Constant(5), &sum));
        assert!(cost_leq(&CostBound::Constant(4), &sum));
        assert!(!cost_leq(&CostBound::Constant(6), &sum));

        // Sum(Constant(2), Constant(3)) <= Constant(5) should also work.
        let sum2 = CostBound::Sum(
            Box::new(CostBound::Constant(2)),
            Box::new(CostBound::Constant(3)),
        );
        assert!(cost_leq(&sum2, &CostBound::Constant(5)));
        assert!(cost_leq(&sum2, &CostBound::Constant(6)));
        assert!(!cost_leq(&sum2, &CostBound::Constant(4)));
    }

    #[test]
    fn sum_ordering() {
        let sum_a = CostBound::Sum(
            Box::new(CostBound::Constant(1)),
            Box::new(CostBound::Constant(2)),
        );
        let sum_b = CostBound::Sum(
            Box::new(CostBound::Constant(3)),
            Box::new(CostBound::Constant(4)),
        );
        assert!(cost_leq(&sum_a, &sum_b));
        assert!(!cost_leq(&sum_b, &sum_a));
    }
}
