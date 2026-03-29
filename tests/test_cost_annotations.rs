//! Tests that cost annotations are actually verified against proven costs.
//!
//! The checker must reject programs whose declared cost is tighter than what
//! the kernel can prove, and accept programs whose declared cost is correct
//! or loose.
//!
//! NOTE: The kernel's cost model tracks *expression evaluation cost*, not
//! data-dependent complexity.  A fold over a variable `n` has near-Zero
//! proven cost because the input expression `n` costs Zero to evaluate.
//! The iteration count proxy uses the *cost of the input expression*, not
//! the runtime data size.  This means `[cost: Const(1)]` on a fold over a
//! bare variable is currently accepted.  Data-dependent cost tracking would
//! require a separate Size() analysis.

use iris_bootstrap::syntax::kernel::checker::{type_check, type_check_graded, VerificationReport};
use iris_types::cost::CostBound;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn compile_and_check(src: &str) -> Vec<(String, VerificationReport)> {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!(
            "compilation failed with {} errors",
            result.errors.len(),
        );
    }
    result
        .fragments
        .into_iter()
        .map(|(name, fragment, _smap)| {
            let tier = iris_bootstrap::syntax::classify_tier(&fragment.graph);
            let report = type_check_graded(&fragment.graph, tier);
            (name, report)
        })
        .collect()
}

fn get_report<'a>(results: &'a [(String, VerificationReport)], name: &str) -> &'a VerificationReport {
    &results
        .iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| {
            let names: Vec<_> = results.iter().map(|(n, _)| n.as_str()).collect();
            panic!("fragment '{}' not found; available: {:?}", name, names)
        })
        .1
}

fn compile_and_get_proven_cost(src: &str) -> CostBound {
    let result = iris_bootstrap::syntax::compile(src);
    assert!(result.errors.is_empty(), "compile errors: {:?}", result.errors);
    let (_, fragment, _) = &result.fragments[0];
    let tier = iris_bootstrap::syntax::classify_tier(&fragment.graph);
    match type_check(&fragment.graph, tier) {
        Ok((_, root_thm)) => root_thm.cost().clone(),
        Err(e) => panic!("type_check failed: {:?}", e),
    }
}

// ---------------------------------------------------------------------------
// Tests: correct annotations pass
// ---------------------------------------------------------------------------

#[test]
fn correct_const_cost_produces_no_warnings() {
    let src = r#"
let double n : Int -> Int [cost: Const(1)] = n * 2
"#;
    let results = compile_and_check(src);
    let report = get_report(&results, "double");
    assert!(
        report.cost_warnings.is_empty(),
        "expected no cost warnings for Const(1) on a simple multiplication, got: {:?}",
        report.cost_warnings,
    );
    assert!(report.failed.is_empty(), "expected no failures: {:?}", report.failed);
}

#[test]
fn correct_linear_cost_on_fold_produces_no_warnings() {
    let src = r#"
let sum_to n : Int -> Int [cost: Linear(n)] = fold 0 (+) n
"#;
    let results = compile_and_check(src);
    let report = get_report(&results, "sum_to");
    assert!(
        report.cost_warnings.is_empty(),
        "expected no cost warnings for Linear(n) on a fold, got: {:?}",
        report.cost_warnings,
    );
    assert!(report.failed.is_empty(), "expected no failures: {:?}", report.failed);
}

#[test]
fn unknown_cost_produces_no_warnings() {
    let src = r#"
let add a b : Int -> Int -> Int = a + b
"#;
    let results = compile_and_check(src);
    let report = get_report(&results, "add");
    assert!(
        report.cost_warnings.is_empty(),
        "expected no cost warnings for unannotated function, got: {:?}",
        report.cost_warnings,
    );
}

// ---------------------------------------------------------------------------
// Tests: loose (overestimating) annotations pass
// ---------------------------------------------------------------------------

#[test]
fn overestimated_cost_is_accepted() {
    // Declaring Linear(n) on a constant-time function is fine — the proven
    // cost (Const) is <= Linear, so the annotation is valid (just loose).
    let src = r#"
let double n : Int -> Int [cost: Linear(n)] = n * 2
"#;
    let results = compile_and_check(src);
    let report = get_report(&results, "double");
    assert!(
        report.cost_warnings.is_empty(),
        "overestimated cost should not produce warnings, got: {:?}",
        report.cost_warnings,
    );
}

// ---------------------------------------------------------------------------
// Tests: incorrect annotations produce warnings
// ---------------------------------------------------------------------------

#[test]
fn zero_cost_on_arithmetic_produces_warning() {
    // Zero cost means compile-time-only, but n * 2 requires a runtime Prim
    // operation (cost: Constant(1)).  The proven cost exceeds Zero.
    let src = r#"
let double n : Int -> Int [cost: Zero] = n * 2
"#;
    let results = compile_and_check(src);
    let report = get_report(&results, "double");
    assert!(
        !report.cost_warnings.is_empty(),
        "expected a cost warning for Zero on runtime arithmetic, but got none",
    );
    let warning = &report.cost_warnings[0];
    assert!(
        matches!(warning.declared, CostBound::Zero),
        "expected declared cost Zero, got {:?}",
        warning.declared,
    );
}

#[test]
fn zero_cost_on_nested_arithmetic_produces_warning() {
    // Multiple Prim operations chain cost: (a + b) * 2 has proven cost
    // that exceeds Zero even in the expression-cost model.
    let src = r#"
let compute a b : Int -> Int -> Int [cost: Zero] = (a + b) * 2
"#;
    let results = compile_and_check(src);
    let report = get_report(&results, "compute");
    assert!(
        !report.cost_warnings.is_empty(),
        "expected a cost warning for Zero on multi-step arithmetic, but got none",
    );
}

// ---------------------------------------------------------------------------
// Tests: proven cost is computed correctly
// ---------------------------------------------------------------------------

#[test]
fn proven_cost_of_literal_is_zero() {
    let cost = compile_and_get_proven_cost("let x : Int = 42");
    assert!(
        matches!(cost, CostBound::Zero),
        "expected literal to have Zero proven cost, got {:?}",
        cost,
    );
}

#[test]
fn proven_cost_of_arithmetic_includes_prim_cost() {
    // n * 2: Prim(mul) costs Constant(1), operands are Zero/Constant(1)
    let cost = compile_and_get_proven_cost("let double n : Int -> Int = n * 2");
    // The proven cost should be non-Zero (includes the Prim operation).
    assert!(
        !matches!(cost, CostBound::Zero),
        "expected arithmetic to have non-Zero proven cost, got {:?}",
        cost,
    );
}

#[test]
fn proven_cost_of_fold_includes_mul_step() {
    // fold 0 (+) n: fold_rule produces Sum(input_cost, Sum(base_cost, Mul(step_cost, input_cost)))
    let cost = compile_and_get_proven_cost(
        "let sum_to n : Int -> Int = fold 0 (+) n"
    );
    // The fold cost uses Mul for the step*input product.
    // Since input `n` is a bare variable (cost=Zero), the Mul evaluates
    // to Zero, but the structure should still be a Sum.
    assert!(
        matches!(cost, CostBound::Sum(..)),
        "expected fold to have Sum proven cost, got {:?}",
        cost,
    );
}

// ---------------------------------------------------------------------------
// Tests: graph-level cost is checked (not just per-node)
// ---------------------------------------------------------------------------

#[test]
fn graph_cost_annotation_is_stored() {
    let result = iris_bootstrap::syntax::compile(
        "let sum_to n : Int -> Int [cost: Linear(n)] = fold 0 (+) n"
    );
    assert!(result.errors.is_empty());
    let (_, fragment, _) = &result.fragments[0];
    assert!(
        !matches!(fragment.graph.cost, CostBound::Unknown),
        "expected graph.cost to be set to Linear, got {:?}",
        fragment.graph.cost,
    );
}

#[test]
fn root_node_gets_annotated_cost_term() {
    let result = iris_bootstrap::syntax::compile(
        "let double n : Int -> Int [cost: Const(1)] = n * 2"
    );
    assert!(result.errors.is_empty());
    let (_, fragment, _) = &result.fragments[0];
    let root = &fragment.graph.nodes[&fragment.graph.root];
    assert!(
        matches!(root.cost, iris_types::cost::CostTerm::Annotated(_)),
        "expected root node cost to be Annotated, got {:?}",
        root.cost,
    );
}

#[test]
fn unannotated_function_root_stays_unit() {
    let result = iris_bootstrap::syntax::compile(
        "let add a b : Int -> Int -> Int = a + b"
    );
    assert!(result.errors.is_empty());
    let (_, fragment, _) = &result.fragments[0];
    let root = &fragment.graph.nodes[&fragment.graph.root];
    assert!(
        matches!(root.cost, iris_types::cost::CostTerm::Unit),
        "expected root node cost to be Unit for unannotated function, got {:?}",
        root.cost,
    );
}

// ---------------------------------------------------------------------------
// Tests: graph-level cost check catches mismatches
// ---------------------------------------------------------------------------

#[test]
fn graph_level_check_catches_zero_on_prim() {
    // The graph-level check compares graph.cost (Zero) against the root
    // theorem's proven cost (non-Zero due to Prim operations).
    let src = r#"
let add a b : Int -> Int -> Int [cost: Zero] = a + b
"#;
    let results = compile_and_check(src);
    let report = get_report(&results, "add");
    assert!(
        !report.cost_warnings.is_empty(),
        "expected graph-level cost warning for Zero on addition",
    );
}

// ---------------------------------------------------------------------------
// Tests: multiple functions with mixed correctness
// ---------------------------------------------------------------------------

#[test]
fn mixed_correct_and_incorrect_annotations() {
    let src = r#"
let good n : Int -> Int [cost: Const(1)] = n * 2
let bad n : Int -> Int [cost: Zero] = n * 2
"#;
    let results = compile_and_check(src);

    let good_report = get_report(&results, "good");
    assert!(
        good_report.cost_warnings.is_empty(),
        "good function should have no cost warnings: {:?}",
        good_report.cost_warnings,
    );

    let bad_report = get_report(&results, "bad");
    assert!(
        !bad_report.cost_warnings.is_empty(),
        "bad function should have a cost warning",
    );
}

// ---------------------------------------------------------------------------
// Tests: existing programs still pass (regression guard)
// ---------------------------------------------------------------------------

#[test]
fn factorial_with_linear_cost_passes() {
    let src = r#"
let rec factorial n : Int -> Int [cost: Linear(n)] =
  if n <= 1 then 1
  else n * factorial (n - 1)
"#;
    let results = compile_and_check(src);
    let report = get_report(&results, "factorial");
    assert!(report.failed.is_empty(), "factorial should not fail: {:?}", report.failed);
}

#[test]
fn simple_arithmetic_with_const_passes() {
    let src = r#"
let square n : Int -> Int [cost: Const(1)] = n * n
"#;
    let results = compile_and_check(src);
    let report = get_report(&results, "square");
    assert!(
        report.cost_warnings.is_empty(),
        "simple arithmetic with Const(1) should not warn: {:?}",
        report.cost_warnings,
    );
    assert!(report.failed.is_empty());
}

#[test]
fn if_else_with_const_passes() {
    let src = r#"
let abs n : Int -> Int [cost: Const(1)] =
  if n < 0 then 0 - n else n
"#;
    let results = compile_and_check(src);
    let report = get_report(&results, "abs");
    assert!(report.failed.is_empty(), "abs should not fail: {:?}", report.failed);
}

// ---------------------------------------------------------------------------
// Tests: the per-node annotated cost term is checked
// ---------------------------------------------------------------------------

#[test]
fn annotated_root_cost_is_checked_by_per_node_check() {
    // When the root has CostTerm::Annotated(Zero) but the kernel proves
    // a non-Zero cost, the per-node check should flag it.
    let src = r#"
let double n : Int -> Int [cost: Zero] = n * 2
"#;
    let result = iris_bootstrap::syntax::compile(src);
    assert!(result.errors.is_empty());
    let (_, fragment, _) = &result.fragments[0];

    // Verify the root was annotated
    let root = &fragment.graph.nodes[&fragment.graph.root];
    assert!(
        matches!(root.cost, iris_types::cost::CostTerm::Annotated(CostBound::Zero)),
        "expected root Annotated(Zero), got {:?}",
        root.cost,
    );

    // Verify the strict checker's proven cost is non-Zero
    let tier = iris_bootstrap::syntax::classify_tier(&fragment.graph);
    let (_, root_thm) = type_check(&fragment.graph, tier).expect("type_check should succeed");
    assert!(
        !matches!(root_thm.cost(), CostBound::Zero),
        "proven cost should be non-Zero for arithmetic, got {:?}",
        root_thm.cost(),
    );
}
