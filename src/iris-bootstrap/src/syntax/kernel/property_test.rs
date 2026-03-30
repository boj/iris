//! Property-based testing as a probabilistic verification tier.
//!
//! This module provides infrastructure for testing functional correctness
//! properties by generating random inputs and checking that the property
//! holds for all of them. This is not a proof, but provides high confidence
//! when the LIA solver cannot handle a predicate (e.g., nonlinear arithmetic).
//!
//! # Usage
//!
//! Properties are expressed as LIA formulas over bound variables. The
//! property tester generates random assignments and evaluates the formula.
//!
//! For testing program behavior (running an actual SemanticGraph), use the
//! integration-level `property_test_program` function which depends on
//! `iris-exec`.

use std::collections::HashMap;

use iris_types::types::{BoundVar, LIAFormula};

use crate::syntax::kernel::lia_solver;

/// Result of a property test run.
#[derive(Debug, Clone)]
pub struct PropertyTestResult {
    /// Number of test cases run.
    pub num_tests: usize,
    /// Number of tests that passed.
    pub passed: usize,
    /// Number of tests that failed.
    pub failed: usize,
    /// First counterexample found (if any).
    pub counterexample: Option<HashMap<BoundVar, i64>>,
    /// Whether all tests passed.
    pub success: bool,
}

/// A property to test: a LIA formula over named variables with value ranges.
#[derive(Debug, Clone)]
pub struct Property {
    /// The formula that should hold for all inputs.
    pub formula: LIAFormula,
    /// The variables to generate inputs for.
    pub vars: Vec<BoundVar>,
    /// Value ranges for each variable (lo, hi inclusive).
    pub ranges: Vec<(i64, i64)>,
}

/// Run a property test: generate `num_tests` random assignments and check
/// that `property.formula` holds for all of them.
///
/// Uses a deterministic PRNG seeded from the formula's Debug representation
/// mixed with the variable list and `num_tests` so results are reproducible
/// yet distinct across different calls with the same formula.
///
/// # Security note
///
/// The seed is enriched beyond just `blake3(formula)` to reduce the risk of
/// a malicious formula whose Debug representation is crafted to produce a
/// known seed with known weaknesses. The variable list and test count are
/// mixed in as additional entropy.
pub fn property_test(property: &Property, num_tests: usize) -> PropertyTestResult {
    let mut passed = 0;
    let mut counterexample = None;

    // Simple deterministic PRNG (xorshift64)
    let mut seed: u64 = {
        let mut hasher = blake3::Hasher::new();
        // Mix formula structure (same as before for reproducibility baseline).
        hasher.update(format!("{:?}", property.formula).as_bytes());
        // Mix in the variable list so the same formula over different vars
        // produces different seeds.
        for var in &property.vars {
            hasher.update(&var.0.to_le_bytes());
        }
        // Mix in num_tests so callers requesting different test counts do not
        // always explore the same initial prefix of the input space.
        hasher.update(&(num_tests as u64).to_le_bytes());
        // Mix in a compile-time constant (the number of range pairs provided)
        // to differentiate properties with different range configurations.
        hasher.update(&(property.ranges.len() as u64).to_le_bytes());
        let hash = hasher.finalize();
        let bytes: [u8; 8] = hash.as_bytes()[..8].try_into().unwrap();
        u64::from_le_bytes(bytes)
    };
    if seed == 0 {
        seed = 0xDEAD_BEEF_CAFE_BABE;
    }

    for _ in 0..num_tests {
        let mut assignment = HashMap::new();
        for (i, var) in property.vars.iter().enumerate() {
            let (lo, hi) = if i < property.ranges.len() {
                property.ranges[i]
            } else {
                (-100, 100)
            };
            let range_size = (hi - lo + 1).max(1) as u64;
            // xorshift64
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let val = lo + (seed % range_size) as i64;
            assignment.insert(*var, val);
        }

        if !lia_solver::evaluate_lia(&property.formula, &assignment) {
            if counterexample.is_none() {
                counterexample = Some(assignment);
            }
        } else {
            passed += 1;
        }
    }

    let failed = num_tests - passed;
    PropertyTestResult {
        num_tests,
        passed,
        failed,
        counterexample,
        success: failed == 0,
    }
}

/// Quick check: test a property with default settings (1000 tests, wide ranges).
pub fn quick_check(formula: &LIAFormula, vars: &[BoundVar]) -> PropertyTestResult {
    let ranges = vars.iter().map(|_| (-1000, 1000)).collect();
    let property = Property {
        formula: formula.clone(),
        vars: vars.to_vec(),
        ranges,
    };
    property_test(&property, 1000)
}

/// Verify a contract: given preconditions and postconditions expressed as
/// LIA formulas, check that `requires => ensures` holds for random inputs.
///
/// The `result_var` is the BoundVar used for the function's return value.
/// The caller is responsible for substituting the actual result value.
pub fn verify_contract(
    requires: &[LIAFormula],
    ensures: &[LIAFormula],
    vars: &[BoundVar],
    num_tests: usize,
) -> PropertyTestResult {
    // Build: (requires_1 AND ... AND requires_n) => (ensures_1 AND ... AND ensures_n)
    let precond = if requires.is_empty() {
        LIAFormula::True
    } else {
        requires.iter().skip(1).fold(requires[0].clone(), |acc, r| {
            LIAFormula::And(Box::new(acc), Box::new(r.clone()))
        })
    };

    let postcond = if ensures.is_empty() {
        LIAFormula::True
    } else {
        ensures.iter().skip(1).fold(ensures[0].clone(), |acc, e| {
            LIAFormula::And(Box::new(acc), Box::new(e.clone()))
        })
    };

    let contract = LIAFormula::Implies(Box::new(precond), Box::new(postcond));

    let ranges = vars.iter().map(|_| (-1000, 1000)).collect();
    let property = Property {
        formula: contract,
        vars: vars.to_vec(),
        ranges,
    };
    property_test(&property, num_tests)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use iris_types::types::{BoundVar, LIAAtom, LIAFormula, LIATerm};

    fn bv(id: u32) -> BoundVar {
        BoundVar(id)
    }

    fn var(id: u32) -> LIATerm {
        LIATerm::Var(BoundVar(id))
    }

    fn con(val: i64) -> LIATerm {
        LIATerm::Const(val)
    }

    #[test]
    fn test_trivially_true() {
        let result = quick_check(&LIAFormula::True, &[bv(0)]);
        assert!(result.success);
        assert_eq!(result.passed, 1000);
    }

    #[test]
    fn test_trivially_false() {
        let result = quick_check(&LIAFormula::False, &[bv(0)]);
        assert!(!result.success);
        assert_eq!(result.passed, 0);
        assert!(result.counterexample.is_some());
    }

    #[test]
    fn test_x_squared_nonneg() {
        // x*x >= 0 — expressed as NOT(x*x < 0)
        // x*x is Mul(x, Var(x)) but LIA Mul is coefficient*term.
        // For property testing, we use the evaluation directly.
        // Actually, we cannot express x*x in LIA terms (it's quadratic).
        // Instead test: |x| >= 0 using IfThenElse.
        use crate::syntax::kernel::lia_solver::lia_abs;
        let abs_x = lia_abs(var(0));
        // abs(x) >= 0 means NOT(abs(x) < 0)
        let formula = LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Lt(
            abs_x,
            con(0),
        ))));
        let result = quick_check(&formula, &[bv(0)]);
        assert!(result.success, "abs(x) >= 0 should hold for all x");
    }

    #[test]
    fn test_add_commutative() {
        // x + y == y + x
        let formula = LIAFormula::Atom(LIAAtom::Eq(
            LIATerm::Add(Box::new(var(0)), Box::new(var(1))),
            LIATerm::Add(Box::new(var(1)), Box::new(var(0))),
        ));
        let result = quick_check(&formula, &[bv(0), bv(1)]);
        assert!(result.success, "x + y == y + x should hold for all x, y");
    }

    #[test]
    fn test_contract_abs() {
        use crate::syntax::kernel::lia_solver::lia_abs;
        // requires: -1000000 <= x <= 1000000
        // ensures: result >= 0
        // ensures: result == if x >= 0 then x else -x
        let x = var(0);
        let result_var = var(1); // BoundVar(1) represents `result`

        let req = LIAFormula::And(
            Box::new(LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Lt(
                x.clone(),
                con(-1_000_000),
            ))))),
            Box::new(LIAFormula::Atom(LIAAtom::Le(x.clone(), con(1_000_000)))),
        );

        let ens1 = LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Lt(
            result_var.clone(),
            con(0),
        ))));
        let ens2 = LIAFormula::Atom(LIAAtom::Eq(
            result_var.clone(),
            lia_abs(x.clone()),
        ));

        let result = verify_contract(
            &[req],
            &[ens1, ens2],
            &[bv(0), bv(1)],
            5000,
        );
        // This won't pass because we're generating random result values
        // that don't match abs(x). This is correct — contract verification
        // with program execution is done at the integration test level.
        // Here we just verify the infrastructure works.
        assert_eq!(result.num_tests, 5000);
    }
}
