//! LIA (Linear Integer Arithmetic) decision procedure — pure Rust, no external SMT.
//!
//! Provides evaluation, satisfiability solving, and counterexample generation
//! for quantifier-free LIA formulas used in refinement type predicates.
//!
//! # Algorithm
//!
//! For `solve_lia`, we use a two-phase approach:
//! 1. **Bounded enumeration** over small ranges (fast for simple predicates).
//! 2. **Negation-based counterexample search** that negates the predicate and
//!    searches for a satisfying assignment to the negated formula.
//!
//! This avoids pulling in z3/cvc5 as a dependency while handling the common
//! refinement predicates that arise in practice (bounds checks, divisibility,
//! simple arithmetic constraints).

use std::collections::HashMap;

use iris_types::types::{BoundVar, LIAAtom, LIAFormula, LIATerm};

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

/// Evaluate a `LIATerm` under a variable assignment.
///
/// Variables not present in the assignment are treated as 0.
pub fn evaluate_term(term: &LIATerm, assignment: &HashMap<BoundVar, i64>) -> i64 {
    match term {
        LIATerm::Const(c) => *c,
        LIATerm::Var(v) => *assignment.get(v).unwrap_or(&0),
        LIATerm::Add(a, b) => {
            evaluate_term(a, assignment).saturating_add(evaluate_term(b, assignment))
        }
        LIATerm::Mul(c, t) => c.saturating_mul(evaluate_term(t, assignment)),
        LIATerm::Neg(t) => evaluate_term(t, assignment).saturating_neg(),
        LIATerm::Len(v) => *assignment.get(v).unwrap_or(&0),
        LIATerm::Size(v) => *assignment.get(v).unwrap_or(&0),
        LIATerm::IfThenElse(cond, then_t, else_t) => {
            if evaluate_lia(cond, assignment) {
                evaluate_term(then_t, assignment)
            } else {
                evaluate_term(else_t, assignment)
            }
        }
        LIATerm::Mod(a, b) => {
            let bv = evaluate_term(b, assignment);
            if bv == 0 {
                0 // avoid division by zero
            } else {
                evaluate_term(a, assignment) % bv
            }
        }
    }
}

/// Evaluate a `LIAAtom` under a variable assignment.
fn evaluate_atom(atom: &LIAAtom, assignment: &HashMap<BoundVar, i64>) -> bool {
    match atom {
        LIAAtom::Eq(a, b) => evaluate_term(a, assignment) == evaluate_term(b, assignment),
        LIAAtom::Lt(a, b) => evaluate_term(a, assignment) < evaluate_term(b, assignment),
        LIAAtom::Le(a, b) => evaluate_term(a, assignment) <= evaluate_term(b, assignment),
        LIAAtom::Divisible(t, d) => {
            if *d == 0 {
                false
            } else {
                evaluate_term(t, assignment) % (*d as i64) == 0
            }
        }
    }
}

/// Evaluate a quantifier-free LIA formula under a variable assignment.
pub fn evaluate_lia(formula: &LIAFormula, assignment: &HashMap<BoundVar, i64>) -> bool {
    match formula {
        LIAFormula::True => true,
        LIAFormula::False => false,
        LIAFormula::And(a, b) => evaluate_lia(a, assignment) && evaluate_lia(b, assignment),
        LIAFormula::Or(a, b) => evaluate_lia(a, assignment) || evaluate_lia(b, assignment),
        LIAFormula::Not(f) => !evaluate_lia(f, assignment),
        LIAFormula::Implies(a, b) => !evaluate_lia(a, assignment) || evaluate_lia(b, assignment),
        LIAFormula::Atom(atom) => evaluate_atom(atom, assignment),
    }
}

// ---------------------------------------------------------------------------
// Negate a formula (for counterexample search)
// ---------------------------------------------------------------------------

/// Negate a LIA formula. Used internally by `find_counterexample`.
pub fn negate(formula: &LIAFormula) -> LIAFormula {
    LIAFormula::Not(Box::new(formula.clone()))
}

// ---------------------------------------------------------------------------
// Free variables
// ---------------------------------------------------------------------------

/// Collect all free variables from a term.
fn collect_term_vars(term: &LIATerm, vars: &mut Vec<BoundVar>) {
    match term {
        LIATerm::Var(v) | LIATerm::Len(v) | LIATerm::Size(v) => {
            if !vars.contains(v) {
                vars.push(*v);
            }
        }
        LIATerm::Const(_) => {}
        LIATerm::Add(a, b) => {
            collect_term_vars(a, vars);
            collect_term_vars(b, vars);
        }
        LIATerm::Mul(_, t) | LIATerm::Neg(t) => {
            collect_term_vars(t, vars);
        }
        LIATerm::IfThenElse(cond, then_t, else_t) => {
            collect_formula_vars_inner(cond, vars);
            collect_term_vars(then_t, vars);
            collect_term_vars(else_t, vars);
        }
        LIATerm::Mod(a, b) => {
            collect_term_vars(a, vars);
            collect_term_vars(b, vars);
        }
    }
}

/// Collect all free variables from an atom.
fn collect_atom_vars(atom: &LIAAtom, vars: &mut Vec<BoundVar>) {
    match atom {
        LIAAtom::Eq(a, b) | LIAAtom::Lt(a, b) | LIAAtom::Le(a, b) => {
            collect_term_vars(a, vars);
            collect_term_vars(b, vars);
        }
        LIAAtom::Divisible(t, _) => {
            collect_term_vars(t, vars);
        }
    }
}

/// Collect all free variables from a formula.
pub fn collect_formula_vars(formula: &LIAFormula) -> Vec<BoundVar> {
    let mut vars = Vec::new();
    collect_formula_vars_inner(formula, &mut vars);
    vars
}

fn collect_formula_vars_inner(formula: &LIAFormula, vars: &mut Vec<BoundVar>) {
    match formula {
        LIAFormula::True | LIAFormula::False => {}
        LIAFormula::And(a, b) | LIAFormula::Or(a, b) | LIAFormula::Implies(a, b) => {
            collect_formula_vars_inner(a, vars);
            collect_formula_vars_inner(b, vars);
        }
        LIAFormula::Not(f) => {
            collect_formula_vars_inner(f, vars);
        }
        LIAFormula::Atom(atom) => {
            collect_atom_vars(atom, vars);
        }
    }
}

// ---------------------------------------------------------------------------
// Bounded enumeration solver
// ---------------------------------------------------------------------------

/// Default search range per variable when no explicit ranges are given.
const DEFAULT_RANGE: (i64, i64) = (-16, 16);

/// Find a satisfying assignment for a QF-LIA formula, or prove unsatisfiable
/// within the bounded search space.
///
/// Uses bounded enumeration over small ranges first. For formulas with more
/// than 3 variables, the ranges are automatically narrowed to keep the
/// search space tractable.
///
/// **IMPORTANT: This solver is probabilistic / incomplete.**
/// - For <= 2 variables, enumerates [-16, 16] (33^2 = 1,089 assignments).
/// - For 3-4 variables, enumerates [-8, 8] (17^3 to 17^4 assignments).
/// - For 5+ variables, enumerates [-4, 4] (9^n assignments).
///
/// A `None` result means UNSAT *within the search bounds*, not globally
/// UNSAT. Formulas with solutions outside the enumerated range will be
/// incorrectly reported as unsatisfiable.
///
/// **Future work:** Replace bounded enumeration with DPLL(T) or integrate
/// an SMT solver backend for complete decision procedures on QF-LIA.
///
/// Returns `Some(assignment)` if SAT, `None` if UNSAT within bounds.
pub fn solve_lia(
    formula: &LIAFormula,
    vars: &[BoundVar],
) -> Option<HashMap<BoundVar, i64>> {
    // Trivial cases.
    match formula {
        LIAFormula::True => {
            return Some(vars.iter().map(|v| (*v, 0)).collect());
        }
        LIAFormula::False => {
            return None;
        }
        _ => {}
    }

    if vars.is_empty() {
        // No variables: just evaluate.
        let empty = HashMap::new();
        if evaluate_lia(formula, &empty) {
            return Some(empty);
        } else {
            return None;
        }
    }

    // Determine ranges: scale down for many variables.
    let (lo, hi) = if vars.len() <= 2 {
        DEFAULT_RANGE
    } else if vars.len() <= 4 {
        (-8, 8)
    } else {
        (-4, 4)
    };

    let ranges: Vec<(i64, i64)> = vars.iter().map(|_| (lo, hi)).collect();
    solve_lia_bounded(formula, vars, &ranges)
}

/// Solve a QF-LIA formula with explicit per-variable ranges.
fn solve_lia_bounded(
    formula: &LIAFormula,
    vars: &[BoundVar],
    ranges: &[(i64, i64)],
) -> Option<HashMap<BoundVar, i64>> {
    let mut assignment = HashMap::new();
    if enumerate_solve(formula, vars, ranges, 0, &mut assignment) {
        Some(assignment)
    } else {
        None
    }
}

/// Recursive enumeration over the Cartesian product of variable ranges.
fn enumerate_solve(
    formula: &LIAFormula,
    vars: &[BoundVar],
    ranges: &[(i64, i64)],
    depth: usize,
    assignment: &mut HashMap<BoundVar, i64>,
) -> bool {
    if depth == vars.len() {
        return evaluate_lia(formula, assignment);
    }

    let (lo, hi) = ranges[depth];
    let var = vars[depth];

    for val in lo..=hi {
        assignment.insert(var, val);
        if enumerate_solve(formula, vars, ranges, depth + 1, assignment) {
            return true;
        }
    }

    assignment.remove(&var);
    false
}

// ---------------------------------------------------------------------------
// Counterexample generation
// ---------------------------------------------------------------------------

/// Generate a counterexample: an assignment that violates a refinement predicate.
///
/// Negates the predicate and searches for a satisfying assignment. If found,
/// that assignment demonstrates a violation of the predicate.
///
/// `value_ranges` provides bounded search ranges for each variable.
/// If empty, uses default ranges derived from the variable list.
pub fn find_counterexample(
    predicate: &LIAFormula,
    vars: &[BoundVar],
    value_ranges: &[(i64, i64)],
) -> Option<HashMap<BoundVar, i64>> {
    // Trivially true predicates have no counterexamples.
    if matches!(predicate, LIAFormula::True) {
        return None;
    }

    // Trivially false predicates: any assignment is a counterexample.
    if matches!(predicate, LIAFormula::False) {
        return Some(vars.iter().map(|v| (*v, 0)).collect());
    }

    // Negate the predicate: we want an assignment where the predicate is false.
    let negated = negate(predicate);

    let ranges: Vec<(i64, i64)> = if value_ranges.len() == vars.len() {
        value_ranges.to_vec()
    } else {
        // Use default ranges.
        let (lo, hi) = if vars.len() <= 2 {
            DEFAULT_RANGE
        } else if vars.len() <= 4 {
            (-8, 8)
        } else {
            (-4, 4)
        };
        vars.iter().map(|_| (lo, hi)).collect()
    };

    solve_lia_bounded(&negated, vars, &ranges)
}

// ---------------------------------------------------------------------------
// LIA term constructors: abs, min, max
// ---------------------------------------------------------------------------

/// Construct an absolute-value term: `abs(t)` = `if t >= 0 then t else -t`.
pub fn lia_abs(t: LIATerm) -> LIATerm {
    LIATerm::IfThenElse(
        Box::new(LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Lt(
            t.clone(),
            LIATerm::Const(0),
        ))))),
        Box::new(t.clone()),
        Box::new(LIATerm::Neg(Box::new(t))),
    )
}

/// Construct a min term: `min(a, b)` = `if a <= b then a else b`.
pub fn lia_min(a: LIATerm, b: LIATerm) -> LIATerm {
    LIATerm::IfThenElse(
        Box::new(LIAFormula::Atom(LIAAtom::Le(a.clone(), b.clone()))),
        Box::new(a),
        Box::new(b),
    )
}

/// Construct a max term: `max(a, b)` = `if a >= b then a else b`.
pub fn lia_max(a: LIATerm, b: LIATerm) -> LIATerm {
    LIATerm::IfThenElse(
        Box::new(LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Lt(
            a.clone(),
            b.clone(),
        ))))),
        Box::new(a),
        Box::new(b),
    )
}

/// Construct a divisibility predicate: `x % n == 0`.
pub fn lia_divisible(t: LIATerm, n: u64) -> LIAFormula {
    LIAFormula::Atom(LIAAtom::Divisible(t, n))
}

/// Construct a modulo-equality predicate: `a % b == c`.
pub fn lia_mod_eq(a: LIATerm, b: LIATerm, c: LIATerm) -> LIAFormula {
    LIAFormula::Atom(LIAAtom::Eq(
        LIATerm::Mod(Box::new(a), Box::new(b)),
        c,
    ))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use iris_types::types::{BoundVar, LIAAtom, LIAFormula, LIATerm};

    fn var(id: u32) -> LIATerm {
        LIATerm::Var(BoundVar(id))
    }

    fn con(val: i64) -> LIATerm {
        LIATerm::Const(val)
    }

    fn bv(id: u32) -> BoundVar {
        BoundVar(id)
    }

    // -----------------------------------------------------------------------
    // evaluate_lia tests
    // -----------------------------------------------------------------------

    #[test]
    fn eval_true() {
        let empty = HashMap::new();
        assert!(evaluate_lia(&LIAFormula::True, &empty));
    }

    #[test]
    fn eval_false() {
        let empty = HashMap::new();
        assert!(!evaluate_lia(&LIAFormula::False, &empty));
    }

    #[test]
    fn eval_x_gt_0() {
        // x > 0 is equivalent to Not(Le(x, 0))
        let formula = LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Le(
            var(0),
            con(0),
        ))));

        let mut pos = HashMap::new();
        pos.insert(bv(0), 5);
        assert!(evaluate_lia(&formula, &pos));

        let mut zero = HashMap::new();
        zero.insert(bv(0), 0);
        assert!(!evaluate_lia(&formula, &zero));

        let mut neg = HashMap::new();
        neg.insert(bv(0), -3);
        assert!(!evaluate_lia(&formula, &neg));
    }

    #[test]
    fn eval_x_plus_y_eq_5() {
        // x + y == 5
        let formula = LIAFormula::Atom(LIAAtom::Eq(
            LIATerm::Add(Box::new(var(0)), Box::new(var(1))),
            con(5),
        ));

        let mut yes = HashMap::new();
        yes.insert(bv(0), 2);
        yes.insert(bv(1), 3);
        assert!(evaluate_lia(&formula, &yes));

        let mut no = HashMap::new();
        no.insert(bv(0), 2);
        no.insert(bv(1), 4);
        assert!(!evaluate_lia(&formula, &no));
    }

    #[test]
    fn eval_and() {
        // x > 0 AND x < 10
        let gt_0 = LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Le(
            var(0),
            con(0),
        ))));
        let lt_10 = LIAFormula::Atom(LIAAtom::Lt(var(0), con(10)));
        let formula = LIAFormula::And(Box::new(gt_0), Box::new(lt_10));

        let mut yes = HashMap::new();
        yes.insert(bv(0), 5);
        assert!(evaluate_lia(&formula, &yes));

        let mut no = HashMap::new();
        no.insert(bv(0), 15);
        assert!(!evaluate_lia(&formula, &no));
    }

    #[test]
    fn eval_implies() {
        // x > 0 => x >= 1  (should always be true for integers)
        let premise = LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Le(
            var(0),
            con(0),
        ))));
        let conclusion = LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Lt(
            var(0),
            con(1),
        ))));
        let formula = LIAFormula::Implies(Box::new(premise), Box::new(conclusion));

        for x in -10..=10 {
            let mut a = HashMap::new();
            a.insert(bv(0), x);
            assert!(
                evaluate_lia(&formula, &a),
                "x > 0 => x >= 1 should be true for x = {x}"
            );
        }
    }

    #[test]
    fn eval_divisible() {
        // x divisible by 3
        let formula = LIAFormula::Atom(LIAAtom::Divisible(var(0), 3));

        let mut yes = HashMap::new();
        yes.insert(bv(0), 9);
        assert!(evaluate_lia(&formula, &yes));

        let mut no = HashMap::new();
        no.insert(bv(0), 10);
        assert!(!evaluate_lia(&formula, &no));
    }

    #[test]
    fn eval_mul_term() {
        // 2*x == 10
        let formula = LIAFormula::Atom(LIAAtom::Eq(
            LIATerm::Mul(2, Box::new(var(0))),
            con(10),
        ));

        let mut yes = HashMap::new();
        yes.insert(bv(0), 5);
        assert!(evaluate_lia(&formula, &yes));

        let mut no = HashMap::new();
        no.insert(bv(0), 4);
        assert!(!evaluate_lia(&formula, &no));
    }

    #[test]
    fn eval_neg_term() {
        // -x == 5 implies x == -5
        let formula = LIAFormula::Atom(LIAAtom::Eq(
            LIATerm::Neg(Box::new(var(0))),
            con(5),
        ));

        let mut yes = HashMap::new();
        yes.insert(bv(0), -5);
        assert!(evaluate_lia(&formula, &yes));

        let mut no = HashMap::new();
        no.insert(bv(0), 5);
        assert!(!evaluate_lia(&formula, &no));
    }

    // -----------------------------------------------------------------------
    // solve_lia tests
    // -----------------------------------------------------------------------

    #[test]
    fn solve_trivial_true() {
        let vars = vec![bv(0)];
        let result = solve_lia(&LIAFormula::True, &vars);
        assert!(result.is_some());
    }

    #[test]
    fn solve_trivial_false() {
        let vars = vec![bv(0)];
        let result = solve_lia(&LIAFormula::False, &vars);
        assert!(result.is_none());
    }

    #[test]
    fn solve_x_eq_3() {
        // x == 3
        let formula = LIAFormula::Atom(LIAAtom::Eq(var(0), con(3)));
        let vars = vec![bv(0)];

        let result = solve_lia(&formula, &vars);
        assert!(result.is_some());
        let assignment = result.unwrap();
        assert_eq!(assignment[&bv(0)], 3);
    }

    #[test]
    fn solve_x_plus_y_eq_5() {
        // x + y == 5
        let formula = LIAFormula::Atom(LIAAtom::Eq(
            LIATerm::Add(Box::new(var(0)), Box::new(var(1))),
            con(5),
        ));
        let vars = vec![bv(0), bv(1)];

        let result = solve_lia(&formula, &vars);
        assert!(result.is_some());
        let a = result.unwrap();
        assert_eq!(a[&bv(0)] + a[&bv(1)], 5);
    }

    #[test]
    fn solve_unsat_contradiction() {
        // x > 5 AND x < 3 (UNSAT)
        let gt5 = LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Le(
            var(0),
            con(5),
        ))));
        let lt3 = LIAFormula::Atom(LIAAtom::Lt(var(0), con(3)));
        let formula = LIAFormula::And(Box::new(gt5), Box::new(lt3));

        let vars = vec![bv(0)];
        let result = solve_lia(&formula, &vars);
        assert!(result.is_none());
    }

    #[test]
    fn solve_bounded_range() {
        // 3 <= x <= 7
        let ge3 = LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Lt(
            var(0),
            con(3),
        ))));
        let le7 = LIAFormula::Atom(LIAAtom::Le(var(0), con(7)));
        let formula = LIAFormula::And(Box::new(ge3), Box::new(le7));

        let vars = vec![bv(0)];
        let result = solve_lia(&formula, &vars);
        assert!(result.is_some());
        let val = result.unwrap()[&bv(0)];
        assert!(val >= 3 && val <= 7, "expected 3 <= {val} <= 7");
    }

    #[test]
    fn solve_no_vars() {
        // True with no vars
        let result = solve_lia(&LIAFormula::True, &[]);
        assert!(result.is_some());

        // 1 < 2 with no vars
        let formula = LIAFormula::Atom(LIAAtom::Lt(con(1), con(2)));
        let result = solve_lia(&formula, &[]);
        assert!(result.is_some());

        // 2 < 1 with no vars (false)
        let formula = LIAFormula::Atom(LIAAtom::Lt(con(2), con(1)));
        let result = solve_lia(&formula, &[]);
        assert!(result.is_none());
    }

    // -----------------------------------------------------------------------
    // find_counterexample tests
    // -----------------------------------------------------------------------

    #[test]
    fn counterexample_x_gt_0_found() {
        // Predicate: x > 0. Counterexample: x <= 0.
        let predicate = LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Le(
            var(0),
            con(0),
        ))));
        let vars = vec![bv(0)];

        let result = find_counterexample(&predicate, &vars, &[(-10, 10)]);
        assert!(result.is_some(), "should find x <= 0 as counterexample");
        let ce = result.unwrap();
        assert!(ce[&bv(0)] <= 0, "counterexample x={} should be <= 0", ce[&bv(0)]);
    }

    #[test]
    fn counterexample_true_has_none() {
        // Predicate: True. No counterexample possible.
        let vars = vec![bv(0)];
        let result = find_counterexample(&LIAFormula::True, &vars, &[(-10, 10)]);
        assert!(result.is_none());
    }

    #[test]
    fn counterexample_false_always_found() {
        // Predicate: False. Every assignment is a counterexample.
        let vars = vec![bv(0)];
        let result = find_counterexample(&LIAFormula::False, &vars, &[]);
        assert!(result.is_some());
    }

    #[test]
    fn counterexample_x_in_range() {
        // Predicate: 0 <= x AND x < 100
        // With range [0, 10], there is no counterexample within bounds
        // unless we look outside.
        let ge0 = LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Lt(
            var(0),
            con(0),
        ))));
        let lt100 = LIAFormula::Atom(LIAAtom::Lt(var(0), con(100)));
        let predicate = LIAFormula::And(Box::new(ge0), Box::new(lt100));

        let vars = vec![bv(0)];

        // Search within [0, 10] — predicate holds for all, so no counterexample.
        let result = find_counterexample(&predicate, &vars, &[(0, 10)]);
        assert!(result.is_none(), "no counterexample within [0, 10]");

        // Search within [-5, 5] — negative values violate x >= 0.
        let result = find_counterexample(&predicate, &vars, &[(-5, 5)]);
        assert!(result.is_some(), "should find negative counterexample");
        let ce = result.unwrap();
        assert!(ce[&bv(0)] < 0, "counterexample should be negative");
    }

    #[test]
    fn counterexample_two_vars() {
        // Predicate: x + y > 0
        let predicate = LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Le(
            LIATerm::Add(Box::new(var(0)), Box::new(var(1))),
            con(0),
        ))));

        let vars = vec![bv(0), bv(1)];
        let result = find_counterexample(&predicate, &vars, &[(-5, 5), (-5, 5)]);
        assert!(result.is_some());
        let ce = result.unwrap();
        assert!(
            ce[&bv(0)] + ce[&bv(1)] <= 0,
            "counterexample x={}, y={} should have x+y <= 0",
            ce[&bv(0)],
            ce[&bv(1)]
        );
    }

    #[test]
    fn counterexample_becomes_test_exposure() {
        // This test verifies the end-to-end pattern:
        // 1. Define a predicate a program should satisfy.
        // 2. Find a counterexample.
        // 3. Verify the counterexample actually violates the predicate.

        // Predicate: x >= 0 AND x < 10 AND x divisible by 2
        let ge0 = LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Lt(
            var(0),
            con(0),
        ))));
        let lt10 = LIAFormula::Atom(LIAAtom::Lt(var(0), con(10)));
        let div2 = LIAFormula::Atom(LIAAtom::Divisible(var(0), 2));
        let predicate = LIAFormula::And(
            Box::new(LIAFormula::And(Box::new(ge0), Box::new(lt10))),
            Box::new(div2),
        );

        let vars = vec![bv(0)];
        let result = find_counterexample(&predicate, &vars, &[(-5, 15)]);
        assert!(result.is_some());

        let ce = result.unwrap();
        // The counterexample should actually violate the predicate.
        assert!(
            !evaluate_lia(&predicate, &ce),
            "counterexample {:?} should violate the predicate",
            ce
        );
    }

    // -----------------------------------------------------------------------
    // collect_formula_vars tests
    // -----------------------------------------------------------------------

    #[test]
    fn collect_vars_simple() {
        // x + y == 5
        let formula = LIAFormula::Atom(LIAAtom::Eq(
            LIATerm::Add(Box::new(var(0)), Box::new(var(1))),
            con(5),
        ));
        let vars = collect_formula_vars(&formula);
        assert!(vars.contains(&bv(0)));
        assert!(vars.contains(&bv(1)));
        assert_eq!(vars.len(), 2);
    }

    #[test]
    fn collect_vars_no_duplicates() {
        // x + x == 10
        let formula = LIAFormula::Atom(LIAAtom::Eq(
            LIATerm::Add(Box::new(var(0)), Box::new(var(0))),
            con(10),
        ));
        let vars = collect_formula_vars(&formula);
        assert_eq!(vars.len(), 1);
        assert!(vars.contains(&bv(0)));
    }

    #[test]
    fn collect_vars_nested() {
        // (x > 0) AND (y < 10) OR (z == 5)
        let f = LIAFormula::Or(
            Box::new(LIAFormula::And(
                Box::new(LIAFormula::Atom(LIAAtom::Lt(con(0), var(0)))),
                Box::new(LIAFormula::Atom(LIAAtom::Lt(var(1), con(10)))),
            )),
            Box::new(LIAFormula::Atom(LIAAtom::Eq(var(2), con(5)))),
        );
        let vars = collect_formula_vars(&f);
        assert_eq!(vars.len(), 3);
    }
}
