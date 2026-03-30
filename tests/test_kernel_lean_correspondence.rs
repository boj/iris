//! Cross-validation test suite: Rust kernel vs Lean 4 formalization.
//!
//! This file contains 60+ tests that verify the Rust proof kernel at
//! `src/iris-kernel/src/kernel.rs` produces the same results as the Lean
//! specification at `lean/IrisKernel/Rules.lean`.
//!
//! For each of the 20 inference rules, we test:
//! 1. A positive case (valid input produces a Theorem with correct judgment)
//! 2. A negative case (invalid input is rejected with the right error)
//! 3. An edge case (boundary conditions)
//!
//! Additionally, property-based tests verify:
//! - Random graphs don't crash the checker
//! - Cost lattice transitivity and antisymmetry
//! - Proof hash determinism

use std::collections::HashMap;

use iris_bootstrap::syntax::kernel::cost_checker;
use iris_bootstrap::syntax::kernel::error::KernelError;
use iris_bootstrap::syntax::kernel::kernel::Kernel;
use iris_bootstrap::syntax::kernel::theorem::{Context, Judgment, Theorem};
use iris_types::cost::{CostBound, CostVar};
use iris_types::graph::{BinderId, NodeId, NodeKind, Resolution, SemanticGraph};
use iris_types::hash::SemanticHash;
use iris_types::types::{
    BoundVar, LIAAtom, LIAFormula, LIATerm, PrimType, Tag, TypeDef, TypeEnv, TypeId,
};

// ===========================================================================
// Helpers
// ===========================================================================

/// Build a TypeEnv from a list of (TypeId, TypeDef) pairs.
fn make_type_env(defs: Vec<(TypeId, TypeDef)>) -> TypeEnv {
    TypeEnv {
        types: defs.into_iter().collect(),
    }
}

/// Build a minimal SemanticGraph with only a TypeEnv (no nodes/edges).
fn graph_with_types(type_env: TypeEnv) -> SemanticGraph {
    SemanticGraph {
        root: NodeId(0),
        nodes: HashMap::new(),
        edges: vec![],
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

/// Build a graph that also contains specific nodes (for type_check_node tests).
fn graph_with_nodes(
    type_env: TypeEnv,
    nodes: Vec<iris_types::graph::Node>,
) -> SemanticGraph {
    let mut node_map = HashMap::new();
    let root = nodes.first().map(|n| n.id).unwrap_or(NodeId(0));
    for n in nodes {
        node_map.insert(n.id, n);
    }
    SemanticGraph {
        root,
        nodes: node_map,
        edges: vec![],
        type_env,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

/// Create a simple leaf node for testing type_check_node.
fn make_lit_node(id: NodeId, type_sig: TypeId) -> iris_types::graph::Node {
    iris_types::graph::Node {
        id,
        kind: NodeKind::Lit,
        type_sig,
        cost: iris_types::cost::CostTerm::Unit,
        arity: 0,
        resolution_depth: 0,
        salt: 0,
        payload: iris_types::graph::NodePayload::Lit {
            type_tag: 0,
            value: vec![0; 8],
        },
    }
}

/// Create a Prim node for testing type_check_node.
fn make_prim_node(id: NodeId, type_sig: TypeId) -> iris_types::graph::Node {
    iris_types::graph::Node {
        id,
        kind: NodeKind::Prim,
        type_sig,
        cost: iris_types::cost::CostTerm::Unit,
        arity: 0,
        resolution_depth: 0,
        salt: 0,
        payload: iris_types::graph::NodePayload::Prim { opcode: 0 },
    }
}

/// Create a Theorem directly (only possible within iris-kernel crate, so we
/// use Kernel::refl as a proxy to create theorems with controlled judgments).
fn make_theorem(node_id: NodeId, type_id: TypeId, cost: CostBound, ctx: Context) -> Theorem {
    // Start with refl (empty context, Zero cost), then adapt via rules.
    // For tests that need specific contexts/costs, we chain kernel rules.
    let base = Kernel::refl(node_id, type_id);
    // If ctx is empty and cost is Zero, we can use the base directly.
    if ctx == Context::empty() && cost == CostBound::Zero {
        return base;
    }
    // For non-zero costs, use cost_subsume to lift
    if ctx == Context::empty() {
        if cost_checker::cost_leq(&CostBound::Zero, &cost) {
            return Kernel::cost_subsume(&base, cost).unwrap();
        }
    }
    // For non-empty contexts, we need assume
    // Try to match the first binding
    if let Some(binding) = ctx.bindings.first() {
        if binding.type_id == type_id {
            let thm = Kernel::assume(&ctx, binding.name, node_id).unwrap();
            if cost == CostBound::Zero {
                return thm;
            }
            if cost_checker::cost_leq(&CostBound::Zero, &cost) {
                return Kernel::cost_subsume(&thm, cost).unwrap();
            }
        }
    }
    // Fallback: use refl (won't have the right context, but it's still useful
    // for many tests)
    base
}

// ===========================================================================
// Rule 1: assume — Variable rule (Var)
// Lean: Derivation.assume env Gamma name n tau (Gamma.lookup name = some tau)
// Rust: Kernel::assume(ctx, name, node_id) -> Theorem
// ===========================================================================

/// Lean correspondence: assume rule produces Gamma |- n : tau @ Zero
/// when Gamma.lookup name = some tau.
#[test]
fn rule01_assume_positive() {
    let ctx = Context::empty()
        .extend(BinderId(0), TypeId(10))
        .extend(BinderId(1), TypeId(20));
    let thm = Kernel::assume(&ctx, BinderId(1), NodeId(100)).unwrap();

    // Lean: Derivation env Gamma n tau CostBound.Zero
    assert_eq!(thm.node_id(), NodeId(100));
    assert_eq!(thm.type_ref(), TypeId(20));
    assert_eq!(*thm.cost(), CostBound::Zero);
    assert_eq!(*thm.context(), ctx);
}

/// Negative: binder not in context should fail.
/// Lean: Gamma.lookup name = none => no derivation possible.
#[test]
fn rule01_assume_negative_binder_not_found() {
    let ctx = Context::empty().extend(BinderId(0), TypeId(10));
    let err = Kernel::assume(&ctx, BinderId(99), NodeId(1)).unwrap_err();
    assert!(matches!(err, KernelError::BinderNotFound { rule: "assume", binder: BinderId(99) }));
}

/// Edge case: shadowed binder — most recent binding wins.
/// Lean: Context.lookup searches from most recent (reversed list).
#[test]
fn rule01_assume_edge_shadowing() {
    let ctx = Context::empty()
        .extend(BinderId(0), TypeId(10))
        .extend(BinderId(0), TypeId(20)); // shadows the first
    let thm = Kernel::assume(&ctx, BinderId(0), NodeId(1)).unwrap();
    // Should get TypeId(20), not TypeId(10)
    assert_eq!(thm.type_ref(), TypeId(20));
}

// ===========================================================================
// Rule 2: intro — Arrow introduction (->I)
// Lean: Derivation.intro env Gamma lam binder A B arrow_id kappa_body body
//       requires body derivation in extended context and arrow type in env
// Rust: Kernel::intro(ctx, lam_node, binder_name, binder_type, body_thm, graph)
// ===========================================================================

/// Positive: intro produces Arrow type at Zero cost.
#[test]
fn rule02_intro_positive() {
    let int_id = TypeId(1);
    let arrow_id = TypeId(2);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (arrow_id, TypeDef::Arrow(int_id, int_id, CostBound::Zero)),
    ]);
    let graph = graph_with_types(type_env);

    let ctx = Context::empty();
    let extended = ctx.extend(BinderId(0), int_id);
    let body_thm = Kernel::assume(&extended, BinderId(0), NodeId(10)).unwrap();

    let lam_thm = Kernel::intro(&ctx, NodeId(20), BinderId(0), int_id, &body_thm, &graph).unwrap();

    // Lean: Derivation env Gamma lam arrow_id CostBound.Zero
    assert_eq!(lam_thm.type_ref(), arrow_id);
    assert_eq!(*lam_thm.cost(), CostBound::Zero);
    assert_eq!(*lam_thm.context(), ctx);
}

/// Negative: body theorem has wrong context.
#[test]
fn rule02_intro_negative_context_mismatch() {
    let int_id = TypeId(1);
    let arrow_id = TypeId(2);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (arrow_id, TypeDef::Arrow(int_id, int_id, CostBound::Zero)),
    ]);
    let graph = graph_with_types(type_env);

    // Body theorem is in empty context, not extended
    let body_thm = Kernel::refl(NodeId(10), int_id);
    let ctx = Context::empty();

    let err = Kernel::intro(&ctx, NodeId(20), BinderId(0), int_id, &body_thm, &graph).unwrap_err();
    assert!(matches!(err, KernelError::ContextMismatch { rule: "intro" }));
}

/// Edge case: arrow type not registered in TypeEnv.
#[test]
fn rule02_intro_edge_arrow_not_in_env() {
    let int_id = TypeId(1);
    let bool_id = TypeId(3);
    // TypeEnv has Int and Bool but no Arrow(Int, Int, Zero)
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (bool_id, TypeDef::Primitive(PrimType::Bool)),
    ]);
    let graph = graph_with_types(type_env);

    let ctx = Context::empty();
    let extended = ctx.extend(BinderId(0), int_id);
    let body_thm = Kernel::assume(&extended, BinderId(0), NodeId(10)).unwrap();

    let err = Kernel::intro(&ctx, NodeId(20), BinderId(0), int_id, &body_thm, &graph).unwrap_err();
    assert!(matches!(err, KernelError::InvalidRule { rule: "find_type_id", .. }));
}

// ===========================================================================
// Rule 3: elim — Arrow elimination (->E / modus ponens)
// Lean: Derivation.elim env Gamma f a app A B arrow_id kf ka kb
//       cost = Sum(ka, Sum(kf, kb))
// Rust: Kernel::elim(fn_thm, arg_thm, app_node, graph)
// ===========================================================================

/// Positive: application produces return type with combined cost.
#[test]
fn rule03_elim_positive() {
    let int_id = TypeId(1);
    let bool_id = TypeId(3);
    let arrow_id = TypeId(2);
    let body_cost = CostBound::Constant(5);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (bool_id, TypeDef::Primitive(PrimType::Bool)),
        (arrow_id, TypeDef::Arrow(int_id, bool_id, body_cost.clone())),
    ]);
    let graph = graph_with_types(type_env);

    // fn_thm: f : Arrow(Int, Bool, Constant(5)) @ Zero
    let fn_thm = Kernel::refl(NodeId(1), arrow_id);
    // arg_thm: a : Int @ Zero
    let arg_thm = Kernel::refl(NodeId(2), int_id);

    let app_thm = Kernel::elim(&fn_thm, &arg_thm, NodeId(3), &graph).unwrap();

    // Lean: app : B @ Sum(ka, Sum(kf, kb))
    // ka=Zero, kf=Zero, kb=Constant(5)
    // => Sum(Zero, Sum(Zero, Constant(5)))
    assert_eq!(app_thm.type_ref(), bool_id);
    let expected_cost = CostBound::Sum(
        Box::new(CostBound::Zero),
        Box::new(CostBound::Sum(
            Box::new(CostBound::Zero),
            Box::new(CostBound::Constant(5)),
        )),
    );
    assert_eq!(*app_thm.cost(), expected_cost);
}

/// Negative: argument type mismatch (arg is Bool, function expects Int).
#[test]
fn rule03_elim_negative_type_mismatch() {
    let int_id = TypeId(1);
    let bool_id = TypeId(3);
    let arrow_id = TypeId(2);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (bool_id, TypeDef::Primitive(PrimType::Bool)),
        (arrow_id, TypeDef::Arrow(int_id, int_id, CostBound::Zero)),
    ]);
    let graph = graph_with_types(type_env);

    let fn_thm = Kernel::refl(NodeId(1), arrow_id);
    let arg_thm = Kernel::refl(NodeId(2), bool_id); // Wrong type!

    let err = Kernel::elim(&fn_thm, &arg_thm, NodeId(3), &graph).unwrap_err();
    assert!(matches!(err, KernelError::TypeMismatch { expected, actual, .. }
        if expected == int_id && actual == bool_id));
}

/// Edge case: fn_thm type is not an Arrow.
#[test]
fn rule03_elim_edge_not_arrow() {
    let int_id = TypeId(1);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
    ]);
    let graph = graph_with_types(type_env);

    let fn_thm = Kernel::refl(NodeId(1), int_id); // Int, not Arrow
    let arg_thm = Kernel::refl(NodeId(2), int_id);

    let err = Kernel::elim(&fn_thm, &arg_thm, NodeId(3), &graph).unwrap_err();
    assert!(matches!(err, KernelError::UnexpectedTypeDef { expected: "Arrow", .. }));
}

// ===========================================================================
// Rule 4: refl — Reflexivity
// Lean: Derivation.refl env Gamma n tau (unconditional, any context)
// Rust: Kernel::refl(node_id, type_id) -> Theorem (infallible)
// ===========================================================================

/// Positive: refl produces theorem at Zero cost, empty context.
#[test]
fn rule04_refl_positive() {
    let thm = Kernel::refl(NodeId(42), TypeId(99));

    // Lean: Derivation env Gamma n tau CostBound.Zero
    assert_eq!(thm.node_id(), NodeId(42));
    assert_eq!(thm.type_ref(), TypeId(99));
    assert_eq!(*thm.cost(), CostBound::Zero);
    assert_eq!(*thm.context(), Context::empty());
}

/// Negative: refl is infallible in Rust (no preconditions). Verify it doesn't
/// panic with extreme inputs.
#[test]
fn rule04_refl_negative_extreme_ids() {
    let thm = Kernel::refl(NodeId(u64::MAX), TypeId(u64::MAX));
    assert_eq!(thm.node_id(), NodeId(u64::MAX));
    assert_eq!(thm.type_ref(), TypeId(u64::MAX));
}

/// Edge case: refl with NodeId(0) and TypeId(0) (the "dummy" values used by
/// cost_leq_rule in both Lean and Rust).
#[test]
fn rule04_refl_edge_zero_ids() {
    let thm = Kernel::refl(NodeId(0), TypeId(0));
    assert_eq!(thm.node_id(), NodeId(0));
    assert_eq!(thm.type_ref(), TypeId(0));
    assert_eq!(*thm.cost(), CostBound::Zero);
}

// ===========================================================================
// Rule 5: symm — Symmetry of equality
// Lean: Derivation.symm env Gamma a b tau kappa (derives b from a's derivation)
// Rust: Kernel::symm(thm, other_node) -> Result<Theorem>
// ===========================================================================

/// Positive: symm transfers judgment to a different node with equality witness.
#[test]
fn rule05_symm_positive() {
    let thm_a = Kernel::refl(NodeId(1), TypeId(10));
    let eq_witness = Kernel::refl(NodeId(2), TypeId(10));
    let thm_b = Kernel::symm(&thm_a, NodeId(2), &eq_witness).unwrap();

    // Lean: same type and cost, different node
    assert_eq!(thm_b.node_id(), NodeId(2));
    assert_eq!(thm_b.type_ref(), TypeId(10)); // same type
    assert_eq!(*thm_b.cost(), CostBound::Zero); // same cost
}

/// Negative: symm rejects witness with wrong node_id.
#[test]
fn rule05_symm_negative_wrong_witness() {
    let ctx = Context::empty().extend(BinderId(0), TypeId(10));
    let thm_a = Kernel::assume(&ctx, BinderId(0), NodeId(1)).unwrap();
    // Witness is about NodeId(3), not NodeId(2).
    let bad_witness = Kernel::refl(NodeId(3), TypeId(10));
    let err = Kernel::symm(&thm_a, NodeId(2), &bad_witness).unwrap_err();
    assert!(matches!(err, iris_bootstrap::syntax::kernel::error::KernelError::NotEqual { .. }));
}

/// Positive: symm preserves context, type, cost with valid witness.
#[test]
fn rule05_symm_preserves_cost() {
    let ctx = Context::empty().extend(BinderId(0), TypeId(10));
    let thm_a = Kernel::assume(&ctx, BinderId(0), NodeId(1)).unwrap();
    let eq_witness = Kernel::refl(NodeId(2), TypeId(10));
    let thm_b = Kernel::symm(&thm_a, NodeId(2), &eq_witness).unwrap();

    // Verify context, type, and cost are all preserved
    assert_eq!(*thm_b.context(), ctx);
    assert_eq!(thm_b.type_ref(), TypeId(10));
    assert_eq!(*thm_b.cost(), CostBound::Zero);
    // Only node_id changes
    assert_eq!(thm_b.node_id(), NodeId(2));
}

/// Edge case: symm back to the same node (a = symm(a)).
#[test]
fn rule05_symm_edge_same_node() {
    let thm = Kernel::refl(NodeId(1), TypeId(10));
    let eq_witness = Kernel::refl(NodeId(1), TypeId(10));
    let thm_sym = Kernel::symm(&thm, NodeId(1), &eq_witness).unwrap();
    assert_eq!(thm_sym.node_id(), thm.node_id());
    assert_eq!(thm_sym.type_ref(), thm.type_ref());
}

// ===========================================================================
// Rule 6: trans — Transitivity of equality
// Lean: Derivation.trans env Gamma a b tau k1 k2
//       chains a:tau@k1 and b:tau@k2 => a:tau@k2
// Rust: Kernel::trans(thm1, thm2) -> Result<Theorem>
// ===========================================================================

/// Positive: trans chains two theorems with the same type.
#[test]
fn rule06_trans_positive() {
    let thm1 = Kernel::refl(NodeId(1), TypeId(10));
    let thm2 = Kernel::cost_subsume(
        &Kernel::refl(NodeId(2), TypeId(10)),
        CostBound::Constant(5),
    ).unwrap();

    let result = Kernel::trans(&thm1, &thm2).unwrap();

    // Lean: result has node of thm1, cost of thm2
    assert_eq!(result.node_id(), NodeId(1)); // from thm1
    assert_eq!(result.type_ref(), TypeId(10)); // same type
    assert_eq!(*result.cost(), CostBound::Constant(5)); // from thm2
}

/// Negative: trans fails when types don't match.
#[test]
fn rule06_trans_negative_type_mismatch() {
    let thm1 = Kernel::refl(NodeId(1), TypeId(10));
    let thm2 = Kernel::refl(NodeId(2), TypeId(20));

    let err = Kernel::trans(&thm1, &thm2).unwrap_err();
    assert!(matches!(err, KernelError::TypeMismatch {
        expected: TypeId(10),
        actual: TypeId(20),
        context: "trans",
    }));
}

/// Edge case: trans with identical theorems.
#[test]
fn rule06_trans_edge_identical() {
    let thm = Kernel::refl(NodeId(1), TypeId(10));
    let result = Kernel::trans(&thm, &thm).unwrap();
    assert_eq!(result.node_id(), NodeId(1));
    assert_eq!(result.type_ref(), TypeId(10));
    assert_eq!(*result.cost(), CostBound::Zero);
}

// ===========================================================================
// Rule 7: congr — Congruence
// Lean: Derivation.congr env Gamma f a app tau sigma kf ka
//       result: app : tau @ Sum(kf, ka)
// Rust: Kernel::congr(fn_thm, arg_thm, app_node) -> Result<Theorem>
// ===========================================================================

/// Positive: congr produces Sum cost from function and argument.
#[test]
fn rule07_congr_positive() {
    let fn_thm = Kernel::refl(NodeId(1), TypeId(10));
    let arg_thm = Kernel::refl(NodeId(2), TypeId(20));

    let app_thm = Kernel::congr(&fn_thm, &arg_thm, NodeId(3)).unwrap();

    // Lean: app : tau @ Sum(kf, ka)
    assert_eq!(app_thm.node_id(), NodeId(3));
    assert_eq!(app_thm.type_ref(), TypeId(10)); // function's type
    let expected_cost = CostBound::Sum(
        Box::new(CostBound::Zero),
        Box::new(CostBound::Zero),
    );
    assert_eq!(*app_thm.cost(), expected_cost);
}

/// Negative: congr fails when contexts don't match.
#[test]
fn rule07_congr_negative_context_mismatch() {
    let ctx1 = Context::empty().extend(BinderId(0), TypeId(10));
    let ctx2 = Context::empty().extend(BinderId(1), TypeId(20));

    let fn_thm = Kernel::assume(&ctx1, BinderId(0), NodeId(1)).unwrap();
    let arg_thm = Kernel::assume(&ctx2, BinderId(1), NodeId(2)).unwrap();

    let err = Kernel::congr(&fn_thm, &arg_thm, NodeId(3)).unwrap_err();
    assert!(matches!(err, KernelError::ContextMismatch { rule: "congr" }));
}

/// Edge case: congr with non-zero costs.
#[test]
fn rule07_congr_edge_nonzero_costs() {
    let fn_thm = Kernel::cost_subsume(
        &Kernel::refl(NodeId(1), TypeId(10)),
        CostBound::Constant(3),
    ).unwrap();
    let arg_thm = Kernel::cost_subsume(
        &Kernel::refl(NodeId(2), TypeId(20)),
        CostBound::Constant(7),
    ).unwrap();

    let app_thm = Kernel::congr(&fn_thm, &arg_thm, NodeId(3)).unwrap();

    let expected_cost = CostBound::Sum(
        Box::new(CostBound::Constant(3)),
        Box::new(CostBound::Constant(7)),
    );
    assert_eq!(*app_thm.cost(), expected_cost);
}

// ===========================================================================
// Rule 8: type_check_node — Annotation / axiom schema
// Lean: Derivation.type_check_node env Gamma n tau kappa (TypeWellFormed env tau)
// Rust: Kernel::type_check_node(ctx, graph, node_id) -> Result<Theorem>
// ===========================================================================

/// Positive: Lit node gets type from its annotation at Zero cost.
#[test]
fn rule08_type_check_node_positive_lit() {
    let int_id = TypeId(1);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
    ]);
    let node = make_lit_node(NodeId(1), int_id);
    let graph = graph_with_nodes(type_env, vec![node]);

    let ctx = Context::empty();
    let thm = Kernel::type_check_node(&ctx, &graph, NodeId(1)).unwrap();

    assert_eq!(thm.type_ref(), int_id);
    assert_eq!(*thm.cost(), CostBound::Zero); // Lit nodes: zero cost
}

/// Negative: node references a type not in TypeEnv.
#[test]
fn rule08_type_check_node_negative_type_not_found() {
    let bogus_type_id = TypeId(999);
    let type_env = make_type_env(vec![]); // empty
    let node = make_lit_node(NodeId(1), bogus_type_id);
    let graph = graph_with_nodes(type_env, vec![node]);

    let ctx = Context::empty();
    let err = Kernel::type_check_node(&ctx, &graph, NodeId(1)).unwrap_err();
    assert!(matches!(err, KernelError::TypeNotFound(TypeId(999))));
}

/// Edge case: Prim node gets Constant(1) cost (differs from Lit's Zero).
#[test]
fn rule08_type_check_node_edge_prim_cost() {
    let int_id = TypeId(1);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
    ]);
    let node = make_prim_node(NodeId(1), int_id);
    let graph = graph_with_nodes(type_env, vec![node]);

    let ctx = Context::empty();
    let thm = Kernel::type_check_node(&ctx, &graph, NodeId(1)).unwrap();

    assert_eq!(thm.type_ref(), int_id);
    assert_eq!(*thm.cost(), CostBound::Constant(1));
}

// ===========================================================================
// Rule 9: cost_subsume — Cost subsumption
// Lean: Derivation.cost_subsume env Gamma n tau k1 k2 (k1 <= k2)
// Rust: Kernel::cost_subsume(thm, new_cost) -> Result<Theorem>
// ===========================================================================

/// Positive: weaken Zero to Constant(100).
#[test]
fn rule09_cost_subsume_positive() {
    let thm = Kernel::refl(NodeId(1), TypeId(10));
    let weakened = Kernel::cost_subsume(&thm, CostBound::Constant(100)).unwrap();

    assert_eq!(weakened.type_ref(), TypeId(10)); // type preserved
    assert_eq!(weakened.node_id(), NodeId(1)); // node preserved
    assert_eq!(*weakened.cost(), CostBound::Constant(100));
}

/// Negative: strengthening is rejected (Constant -> Zero).
#[test]
fn rule09_cost_subsume_negative_strengthening() {
    let thm = Kernel::cost_subsume(
        &Kernel::refl(NodeId(1), TypeId(10)),
        CostBound::Constant(5),
    ).unwrap();

    let err = Kernel::cost_subsume(&thm, CostBound::Zero).unwrap_err();
    assert!(matches!(err, KernelError::CostViolation { .. }));
}

/// Edge case: weaken to Unknown (anything <= Unknown).
#[test]
fn rule09_cost_subsume_edge_to_unknown() {
    let n = CostVar(0);
    let thm = Kernel::cost_subsume(
        &Kernel::refl(NodeId(1), TypeId(10)),
        CostBound::Polynomial(n, 3),
    ).unwrap();

    // Unknown is no longer a valid upper bound (soundness fix).
    let result = Kernel::cost_subsume(&thm, CostBound::Unknown);
    assert!(result.is_err(), "cost_subsume to Unknown should fail");
}

// ===========================================================================
// Rule 10: cost_leq_rule — Cost ordering witness
// Lean: Derivation.cost_leq_rule env Gamma k1 k2 (CostLeq k1 k2)
//       produces derivation about dummy node 0 with cost k2
// Rust: Kernel::cost_leq_rule(k1, k2) -> Result<Theorem>
// ===========================================================================

/// Positive: Zero <= Constant(10).
#[test]
fn rule10_cost_leq_rule_positive() {
    let thm = Kernel::cost_leq_rule(&CostBound::Zero, &CostBound::Constant(10)).unwrap();

    // Lean: dummy node 0, type 0, cost k2
    assert_eq!(thm.node_id(), NodeId(0));
    assert_eq!(thm.type_ref(), TypeId(0));
    assert_eq!(*thm.cost(), CostBound::Constant(10));
}

/// Negative: Linear(n) is NOT <= Constant(k).
#[test]
fn rule10_cost_leq_rule_negative() {
    let n = CostVar(0);
    let err = Kernel::cost_leq_rule(
        &CostBound::Linear(n),
        &CostBound::Constant(100),
    ).unwrap_err();
    assert!(matches!(err, KernelError::CostViolation { .. }));
}

/// Edge case: all lattice edges verified.
/// Lean: Zero <= Constant <= Linear <= NLogN <= Polynomial (same variable).
#[test]
fn rule10_cost_leq_rule_edge_full_lattice() {
    let n = CostVar(0);
    // Zero <= Constant
    assert!(Kernel::cost_leq_rule(&CostBound::Zero, &CostBound::Constant(1)).is_ok());
    // Constant <= Linear
    assert!(Kernel::cost_leq_rule(&CostBound::Constant(100), &CostBound::Linear(n)).is_ok());
    // Linear <= NLogN
    assert!(Kernel::cost_leq_rule(&CostBound::Linear(n), &CostBound::NLogN(n)).is_ok());
    // NLogN <= Polynomial(n, 2)
    assert!(Kernel::cost_leq_rule(&CostBound::NLogN(n), &CostBound::Polynomial(n, 2)).is_ok());
    // Polynomial(n, 2) <= Polynomial(n, 3)
    assert!(Kernel::cost_leq_rule(&CostBound::Polynomial(n, 2), &CostBound::Polynomial(n, 3)).is_ok());
    // Unknown is no longer a valid upper bound (soundness fix).
    assert!(Kernel::cost_leq_rule(&CostBound::Polynomial(n, 10), &CostBound::Unknown).is_err());
}

// ===========================================================================
// Rule 11: refine_intro — Refinement type introduction
// Lean: Derivation.refine_intro env Gamma n base refined kappa
//       requires base derivation + predicate derivation + Refined(base) in env
// Rust: Kernel::refine_intro(base_thm, pred_holds, refined_type_id, graph)
// ===========================================================================

/// Positive: introduce a refinement type from a base type and predicate proof.
#[test]
fn rule11_refine_intro_positive() {
    let int_id = TypeId(1);
    let refined_id = TypeId(2);
    let pred = LIAFormula::Atom(LIAAtom::Le(
        LIATerm::Const(0),
        LIATerm::Var(BoundVar(0)),
    ));
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (refined_id, TypeDef::Refined(int_id, pred)),
    ]);
    let graph = graph_with_types(type_env);

    let base_thm = Kernel::refl(NodeId(1), int_id);
    // Predicate witness must be about the same node as base_thm.
    let pred_holds = Kernel::refl(NodeId(1), int_id);

    let result = Kernel::refine_intro(&base_thm, &pred_holds, refined_id, &graph).unwrap();
    assert_eq!(result.type_ref(), refined_id);
    assert_eq!(result.node_id(), NodeId(1));
}

/// Negative: base type doesn't match refined inner type.
#[test]
fn rule11_refine_intro_negative_type_mismatch() {
    let int_id = TypeId(1);
    let bool_id = TypeId(3);
    let refined_id = TypeId(2);
    let pred = LIAFormula::True;
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (bool_id, TypeDef::Primitive(PrimType::Bool)),
        (refined_id, TypeDef::Refined(int_id, pred)),
    ]);
    let graph = graph_with_types(type_env);

    let base_thm = Kernel::refl(NodeId(1), bool_id); // Wrong! Refined wraps Int
    let pred_holds = Kernel::refl(NodeId(2), int_id);

    let err = Kernel::refine_intro(&base_thm, &pred_holds, refined_id, &graph).unwrap_err();
    assert!(matches!(err, KernelError::TypeMismatch { .. }));
}

/// Edge case: refined type is not actually Refined in TypeEnv.
#[test]
fn rule11_refine_intro_edge_not_refined() {
    let int_id = TypeId(1);
    let not_refined_id = TypeId(2);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (not_refined_id, TypeDef::Primitive(PrimType::Bool)), // Not Refined!
    ]);
    let graph = graph_with_types(type_env);

    let base_thm = Kernel::refl(NodeId(1), int_id);
    let pred_holds = Kernel::refl(NodeId(2), int_id);

    let err = Kernel::refine_intro(&base_thm, &pred_holds, not_refined_id, &graph).unwrap_err();
    assert!(matches!(err, KernelError::UnexpectedTypeDef { expected: "Refined", .. }));
}

// ===========================================================================
// Rule 12: refine_elim — Refinement type elimination
// Lean: Derivation.refine_elim env Gamma n base refined kappa
//       extracts base type from refinement
// Rust: Kernel::refine_elim(thm, graph) -> Result<(Theorem, Theorem)>
// ===========================================================================

/// Positive: extract base type from a refined type.
#[test]
fn rule12_refine_elim_positive() {
    let int_id = TypeId(1);
    let refined_id = TypeId(2);
    let pred = LIAFormula::True;
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (refined_id, TypeDef::Refined(int_id, pred)),
    ]);
    let graph = graph_with_types(type_env);

    let refined_thm = Kernel::refl(NodeId(1), refined_id);
    let (base_thm, pred_thm) = Kernel::refine_elim(&refined_thm, &graph).unwrap();

    // Lean: base_thm has base_type, same node, same cost
    assert_eq!(base_thm.type_ref(), int_id);
    assert_eq!(base_thm.node_id(), NodeId(1));
    assert_eq!(*base_thm.cost(), CostBound::Zero);

    // pred_thm witnesses the predicate
    assert_eq!(pred_thm.node_id(), NodeId(1));
}

/// Negative: theorem's type is not a Refined type.
#[test]
fn rule12_refine_elim_negative_not_refined() {
    let int_id = TypeId(1);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
    ]);
    let graph = graph_with_types(type_env);

    let thm = Kernel::refl(NodeId(1), int_id);
    let err = Kernel::refine_elim(&thm, &graph).unwrap_err();
    assert!(matches!(err, KernelError::UnexpectedTypeDef { expected: "Refined", .. }));
}

/// Edge case: refine_elim preserves cost from input theorem.
#[test]
fn rule12_refine_elim_edge_preserves_cost() {
    let int_id = TypeId(1);
    let refined_id = TypeId(2);
    let pred = LIAFormula::True;
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (refined_id, TypeDef::Refined(int_id, pred)),
    ]);
    let graph = graph_with_types(type_env);

    let refined_thm = Kernel::cost_subsume(
        &Kernel::refl(NodeId(1), refined_id),
        CostBound::Constant(42),
    ).unwrap();

    let (base_thm, _) = Kernel::refine_elim(&refined_thm, &graph).unwrap();
    assert_eq!(*base_thm.cost(), CostBound::Constant(42)); // cost preserved
}

// ===========================================================================
// Rule 13: nat_ind — Natural number induction
// Lean: Derivation.nat_ind env Gamma base step result tau kb ks
//       cost = Sum(kb, ks)
// Rust: Kernel::nat_ind(base, step, result_node) -> Result<Theorem>
// ===========================================================================

/// Positive: base and step (Arrow type) produce nat_ind theorem.
#[test]
fn rule13_nat_ind_positive() {
    let base_type_id = TypeId(10);
    let arrow_type_id = TypeId(11);
    let type_env = TypeEnv {
        types: vec![
            (base_type_id, TypeDef::Primitive(PrimType::Int)),
            (arrow_type_id, TypeDef::Arrow(base_type_id, base_type_id, CostBound::Constant(5))),
        ].into_iter().collect(),
    };
    let graph = graph_with_types(type_env);

    let base = Kernel::refl(NodeId(1), base_type_id);
    let step = Kernel::refl(NodeId(2), arrow_type_id);

    let result = Kernel::nat_ind(&base, &step, NodeId(3), &graph).unwrap();

    // Lean: cost = Sum(kb, step_body_cost)
    assert_eq!(result.type_ref(), base_type_id);
    assert_eq!(result.node_id(), NodeId(3));
    let expected_cost = CostBound::Sum(
        Box::new(CostBound::Zero),           // base cost
        Box::new(CostBound::Constant(5)),     // step body cost from Arrow
    );
    assert_eq!(*result.cost(), expected_cost);
}

/// Negative: step has non-Arrow type.
#[test]
fn rule13_nat_ind_negative_type_mismatch() {
    let base_type_id = TypeId(10);
    let other_type_id = TypeId(20);
    let type_env = TypeEnv {
        types: vec![
            (base_type_id, TypeDef::Primitive(PrimType::Int)),
            (other_type_id, TypeDef::Primitive(PrimType::Bool)),
        ].into_iter().collect(),
    };
    let graph = graph_with_types(type_env);

    let base = Kernel::refl(NodeId(1), base_type_id);
    let step = Kernel::refl(NodeId(2), other_type_id);

    let err = Kernel::nat_ind(&base, &step, NodeId(3), &graph).unwrap_err();
    assert!(matches!(err, KernelError::InductionError { .. }));
}

/// Edge case: both base and step have Zero cost (step Arrow with Zero body cost).
#[test]
fn rule13_nat_ind_edge_zero_costs() {
    let base_type_id = TypeId(10);
    let arrow_type_id = TypeId(11);
    let type_env = TypeEnv {
        types: vec![
            (base_type_id, TypeDef::Primitive(PrimType::Int)),
            (arrow_type_id, TypeDef::Arrow(base_type_id, base_type_id, CostBound::Zero)),
        ].into_iter().collect(),
    };
    let graph = graph_with_types(type_env);

    let base = Kernel::refl(NodeId(1), base_type_id);
    let step = Kernel::refl(NodeId(2), arrow_type_id);

    let result = Kernel::nat_ind(&base, &step, NodeId(3), &graph).unwrap();
    let expected_cost = CostBound::Sum(
        Box::new(CostBound::Zero),
        Box::new(CostBound::Zero),
    );
    assert_eq!(*result.cost(), expected_cost);
}

// ===========================================================================
// Rule 14: structural_ind — Structural induction over ADTs
// Lean: Derivation.structural_ind env Gamma sum_type result_type result_node
//       variants case_nodes case_costs
//       cost = Sup(case_costs)
// Rust: Kernel::structural_ind(ty, cases, result_node, graph)
// ===========================================================================

/// Positive: two-variant sum type with matching cases.
#[test]
fn rule14_structural_ind_positive() {
    let int_id = TypeId(1);
    let bool_id = TypeId(3);
    let sum_id = TypeId(4);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (bool_id, TypeDef::Primitive(PrimType::Bool)),
        (sum_id, TypeDef::Sum(vec![(Tag(0), int_id), (Tag(1), bool_id)])),
    ]);
    let graph = graph_with_types(type_env);

    let case1 = Kernel::refl(NodeId(1), int_id);
    let case2 = Kernel::refl(NodeId(2), int_id); // both produce Int

    let result = Kernel::structural_ind(sum_id, &[case1, case2], NodeId(3), &graph).unwrap();

    assert_eq!(result.type_ref(), int_id);
    assert_eq!(result.node_id(), NodeId(3));
    // Lean: cost = Sup(case_costs)
    assert_eq!(*result.cost(), CostBound::Sup(vec![CostBound::Zero, CostBound::Zero]));
}

/// Negative: wrong number of cases (fewer than variants).
#[test]
fn rule14_structural_ind_negative_missing_case() {
    let int_id = TypeId(1);
    let bool_id = TypeId(3);
    let sum_id = TypeId(4);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (bool_id, TypeDef::Primitive(PrimType::Bool)),
        (sum_id, TypeDef::Sum(vec![(Tag(0), int_id), (Tag(1), bool_id)])),
    ]);
    let graph = graph_with_types(type_env);

    let case1 = Kernel::refl(NodeId(1), int_id);
    // Only 1 case for 2-variant sum

    let err = Kernel::structural_ind(sum_id, &[case1], NodeId(3), &graph).unwrap_err();
    assert!(matches!(err, KernelError::InductionError { .. }));
}

/// Edge case: case types don't all agree.
#[test]
fn rule14_structural_ind_edge_type_disagreement() {
    let int_id = TypeId(1);
    let bool_id = TypeId(3);
    let sum_id = TypeId(4);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (bool_id, TypeDef::Primitive(PrimType::Bool)),
        (sum_id, TypeDef::Sum(vec![(Tag(0), int_id), (Tag(1), bool_id)])),
    ]);
    let graph = graph_with_types(type_env);

    let case1 = Kernel::refl(NodeId(1), int_id);
    let case2 = Kernel::refl(NodeId(2), bool_id); // different result type!

    let err = Kernel::structural_ind(sum_id, &[case1, case2], NodeId(3), &graph).unwrap_err();
    assert!(matches!(err, KernelError::InductionError { .. }));
}

// ===========================================================================
// Rule 15: let_bind — Let binding (cut rule)
// Lean: Derivation.let_bind env Gamma let bound body binder A B k1 k2
//       cost = Sum(k1, k2)
// Rust: Kernel::let_bind(ctx, let_node, binder, bound_thm, body_thm)
// ===========================================================================

/// Positive: let binding with matching contexts.
#[test]
fn rule15_let_bind_positive() {
    let ctx = Context::empty();
    let int_id = TypeId(1);
    let bool_id = TypeId(2);

    // bound_thm: Gamma |- e1 : Int @ Zero
    let bound_thm = Kernel::refl(NodeId(1), int_id);
    // body_thm: Gamma, x:Int |- e2 : Bool @ Zero
    let extended = ctx.extend(BinderId(0), int_id);
    let body_thm = Kernel::assume(&extended, BinderId(0), NodeId(2)).unwrap();
    // body_thm has type Int, not Bool. Let's use refl for Bool in extended context.
    // Actually, we need to construct a theorem in the extended context.
    // We can use symm to change the type reference — but that doesn't work either.
    // For a proper test, use assume which gives us Int.
    // Let's just test with same type.
    let bound_thm2 = Kernel::refl(NodeId(1), int_id);
    let body_thm2 = Kernel::assume(&extended, BinderId(0), NodeId(2)).unwrap();

    let result = Kernel::let_bind(&ctx, NodeId(3), BinderId(0), &bound_thm2, &body_thm2).unwrap();

    // Lean: cost = Sum(k1, k2)
    assert_eq!(result.type_ref(), int_id); // body's type
    assert_eq!(result.node_id(), NodeId(3));
    let expected_cost = CostBound::Sum(
        Box::new(CostBound::Zero), // bound cost
        Box::new(CostBound::Zero), // body cost
    );
    assert_eq!(*result.cost(), expected_cost);
    assert_eq!(*result.context(), ctx);
}

/// Negative: bound theorem has wrong context.
#[test]
fn rule15_let_bind_negative_bound_context_mismatch() {
    let ctx = Context::empty();
    let int_id = TypeId(1);

    // bound_thm is in a different context
    let wrong_ctx = Context::empty().extend(BinderId(99), TypeId(99));
    let bound_thm = Kernel::assume(&wrong_ctx, BinderId(99), NodeId(1)).unwrap();
    let extended = ctx.extend(BinderId(0), TypeId(99));
    let body_thm = Kernel::assume(&extended, BinderId(0), NodeId(2)).unwrap();

    let err = Kernel::let_bind(&ctx, NodeId(3), BinderId(0), &bound_thm, &body_thm).unwrap_err();
    assert!(matches!(err, KernelError::ContextMismatch { .. }));
}

/// Edge case: body theorem has wrong extended context.
#[test]
fn rule15_let_bind_edge_body_context_mismatch() {
    let ctx = Context::empty();
    let int_id = TypeId(1);

    let bound_thm = Kernel::refl(NodeId(1), int_id);
    // Body in wrong extended context (different binder type)
    let wrong_extended = ctx.extend(BinderId(0), TypeId(99));
    let body_thm = Kernel::assume(&wrong_extended, BinderId(0), NodeId(2)).unwrap();

    let err = Kernel::let_bind(&ctx, NodeId(3), BinderId(0), &bound_thm, &body_thm).unwrap_err();
    assert!(matches!(err, KernelError::ContextMismatch { .. }));
}

// ===========================================================================
// Rule 16: match_elim — Sum elimination / case analysis
// Lean: Derivation.match_elim env Gamma scrutinee match scr_type res_type
//       ks arm_nodes arm_costs
//       cost = Sum(ks, Sup(arm_costs))
// Rust: Kernel::match_elim(scrutinee_thm, arm_thms, match_node)
// ===========================================================================

/// Positive: match with two arms agreeing on result type.
#[test]
fn rule16_match_elim_positive() {
    let scrutinee = Kernel::refl(NodeId(1), TypeId(10));
    let arm1 = Kernel::refl(NodeId(2), TypeId(20));
    let arm2 = Kernel::refl(NodeId(3), TypeId(20));

    let result = Kernel::match_elim(&scrutinee, &[arm1, arm2], NodeId(4)).unwrap();

    assert_eq!(result.type_ref(), TypeId(20));
    assert_eq!(result.node_id(), NodeId(4));
    // Lean: Sum(ks, Sup(arm_costs))
    let expected_cost = CostBound::Sum(
        Box::new(CostBound::Zero), // scrutinee cost
        Box::new(CostBound::Sup(vec![CostBound::Zero, CostBound::Zero])),
    );
    assert_eq!(*result.cost(), expected_cost);
}

/// Negative: arm types disagree.
#[test]
fn rule16_match_elim_negative_type_disagreement() {
    let scrutinee = Kernel::refl(NodeId(1), TypeId(10));
    let arm1 = Kernel::refl(NodeId(2), TypeId(20));
    let arm2 = Kernel::refl(NodeId(3), TypeId(30)); // different!

    let err = Kernel::match_elim(&scrutinee, &[arm1, arm2], NodeId(4)).unwrap_err();
    // The lean bridge returns None for arm type mismatches, which the
    // kernel wraps as InvalidRule.
    assert!(matches!(err, KernelError::InvalidRule { .. }));
}

/// Edge case: empty arms list.
#[test]
fn rule16_match_elim_edge_empty_arms() {
    let scrutinee = Kernel::refl(NodeId(1), TypeId(10));

    let err = Kernel::match_elim(&scrutinee, &[], NodeId(4)).unwrap_err();
    assert!(matches!(err, KernelError::InvalidRule { rule: "match_elim", .. }));
}

// ===========================================================================
// Rule 17: fold_rule — Catamorphism / structural recursion
// Lean: Derivation.fold_rule env Gamma base step input fold
//       result_type input_type step_type kb ks ki
//       cost = Sum(ki, Sum(kb, Mul(ks, ki)))
// Rust: Kernel::fold_rule(base_thm, step_thm, input_thm, fold_node)
// ===========================================================================

/// Positive: fold produces correct cost formula.
#[test]
fn rule17_fold_rule_positive() {
    let base_thm = Kernel::refl(NodeId(1), TypeId(10)); // result type
    let step_thm = Kernel::cost_subsume(
        &Kernel::refl(NodeId(2), TypeId(20)),
        CostBound::Constant(3),
    ).unwrap();
    let input_thm = Kernel::cost_subsume(
        &Kernel::refl(NodeId(3), TypeId(30)),
        CostBound::Constant(5),
    ).unwrap();

    let result = Kernel::fold_rule(&base_thm, &step_thm, &input_thm, NodeId(4)).unwrap();

    assert_eq!(result.type_ref(), TypeId(10)); // base's type is result type
    // Lean: Sum(ki, Sum(kb, Mul(ks, ki)))
    let expected_cost = CostBound::Sum(
        Box::new(CostBound::Constant(5)), // ki
        Box::new(CostBound::Sum(
            Box::new(CostBound::Zero), // kb
            Box::new(CostBound::Mul(
                Box::new(CostBound::Constant(3)), // ks
                Box::new(CostBound::Constant(5)), // ki
            )),
        )),
    );
    assert_eq!(*result.cost(), expected_cost);
}

/// Negative: fold_rule is currently infallible in Rust (no type checks on
/// step vs result). Verify it still produces correct structure with mismatched
/// types (the Lean formalization requires step_type as a separate parameter).
#[test]
fn rule17_fold_rule_negative_result_type_from_base() {
    let base_thm = Kernel::refl(NodeId(1), TypeId(10));
    let step_thm = Kernel::refl(NodeId(2), TypeId(20)); // different type
    let input_thm = Kernel::refl(NodeId(3), TypeId(30));

    // Rust fold_rule doesn't check step type — result type always comes from base
    let result = Kernel::fold_rule(&base_thm, &step_thm, &input_thm, NodeId(4)).unwrap();
    assert_eq!(result.type_ref(), TypeId(10)); // always base's type
}

/// Edge case: all Zero costs.
#[test]
fn rule17_fold_rule_edge_zero_costs() {
    let base = Kernel::refl(NodeId(1), TypeId(10));
    let step = Kernel::refl(NodeId(2), TypeId(20));
    let input = Kernel::refl(NodeId(3), TypeId(30));

    let result = Kernel::fold_rule(&base, &step, &input, NodeId(4)).unwrap();

    let expected_cost = CostBound::Sum(
        Box::new(CostBound::Zero),
        Box::new(CostBound::Sum(
            Box::new(CostBound::Zero),
            Box::new(CostBound::Mul(
                Box::new(CostBound::Zero),
                Box::new(CostBound::Zero),
            )),
        )),
    );
    assert_eq!(*result.cost(), expected_cost);
}

// ===========================================================================
// Rule 18: type_abst — ForAll introduction (System F)
// Lean: Derivation.type_abst env Gamma n inner forall_type bv kappa
//       requires body type = inner, ForAll(bv, inner) in env, well-formed
// Rust: Kernel::type_abst(body_thm, forall_type_id, graph)
// ===========================================================================

/// Positive: introduce ForAll type.
#[test]
fn rule18_type_abst_positive() {
    let int_id = TypeId(1);
    let forall_id = TypeId(2);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (forall_id, TypeDef::ForAll(BoundVar(0), int_id)),
    ]);
    let graph = graph_with_types(type_env);

    let body_thm = Kernel::refl(NodeId(1), int_id);
    let result = Kernel::type_abst(&body_thm, forall_id, &graph).unwrap();

    assert_eq!(result.type_ref(), forall_id);
    assert_eq!(result.node_id(), NodeId(1));
    assert_eq!(*result.cost(), CostBound::Zero); // cost preserved
}

/// Negative: body type doesn't match ForAll's inner type.
#[test]
fn rule18_type_abst_negative_inner_mismatch() {
    let int_id = TypeId(1);
    let bool_id = TypeId(3);
    let forall_id = TypeId(2);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (bool_id, TypeDef::Primitive(PrimType::Bool)),
        (forall_id, TypeDef::ForAll(BoundVar(0), int_id)),
    ]);
    let graph = graph_with_types(type_env);

    let body_thm = Kernel::refl(NodeId(1), bool_id); // Bool != Int
    let err = Kernel::type_abst(&body_thm, forall_id, &graph).unwrap_err();
    assert!(matches!(err, KernelError::TypeMismatch { .. }));
}

/// Edge case: forall_type_id is not actually ForAll.
#[test]
fn rule18_type_abst_edge_not_forall() {
    let int_id = TypeId(1);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
    ]);
    let graph = graph_with_types(type_env);

    let body_thm = Kernel::refl(NodeId(1), int_id);
    let err = Kernel::type_abst(&body_thm, int_id, &graph).unwrap_err();
    assert!(matches!(err, KernelError::UnexpectedTypeDef { expected: "ForAll", .. }));
}

// ===========================================================================
// Rule 19: type_app — ForAll elimination (System F)
// Lean: Derivation.type_app env Gamma n forall_type result_type bv inner kappa
//       requires ForAll in env + result_type well-formed
// Rust: Kernel::type_app(thm, result_type_id, graph)
// ===========================================================================

/// Positive: eliminate ForAll to get the instantiated type.
#[test]
fn rule19_type_app_positive() {
    let int_id = TypeId(1);
    let forall_id = TypeId(2);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (forall_id, TypeDef::ForAll(BoundVar(0), int_id)),
    ]);
    let graph = graph_with_types(type_env);

    let forall_thm = Kernel::refl(NodeId(1), forall_id);
    let result = Kernel::type_app(&forall_thm, int_id, &graph).unwrap();

    assert_eq!(result.type_ref(), int_id);
    assert_eq!(result.node_id(), NodeId(1));
    assert_eq!(*result.cost(), CostBound::Zero);
}

/// Negative: result type doesn't exist in TypeEnv (soundness check).
#[test]
fn rule19_type_app_negative_result_not_found() {
    let int_id = TypeId(1);
    let forall_id = TypeId(2);
    let bogus_id = TypeId(999);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (forall_id, TypeDef::ForAll(BoundVar(0), int_id)),
    ]);
    let graph = graph_with_types(type_env);

    let forall_thm = Kernel::refl(NodeId(1), forall_id);
    let err = Kernel::type_app(&forall_thm, bogus_id, &graph).unwrap_err();
    assert!(matches!(err, KernelError::TypeNotFound(TypeId(999))));
}

/// Edge case: result type exists but references dangling TypeId (malformed).
/// This is the critical Rule 19 soundness check from docs/council/01-metatheory.md.
#[test]
fn rule19_type_app_edge_malformed_result_type() {
    let int_id = TypeId(1);
    let forall_id = TypeId(2);
    let malformed_id = TypeId(3);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (forall_id, TypeDef::ForAll(BoundVar(0), int_id)),
        // Arrow references TypeId(999) which doesn't exist
        (malformed_id, TypeDef::Arrow(int_id, TypeId(999), CostBound::Zero)),
    ]);
    let graph = graph_with_types(type_env);

    let forall_thm = Kernel::refl(NodeId(1), forall_id);
    let err = Kernel::type_app(&forall_thm, malformed_id, &graph).unwrap_err();
    assert!(matches!(err, KernelError::TypeMalformed { .. }));
}

// ===========================================================================
// Rule 20: guard_rule — Conditional / if-then-else
// Lean: Derivation.guard_rule env Gamma pred then else guard
//       pred_type result_type kp kt ke
//       cost = Sum(kp, Sup([kt, ke]))
// Rust: Kernel::guard_rule(pred_thm, then_thm, else_thm, guard_node)
// ===========================================================================

/// Positive: both branches have the same type.
#[test]
fn rule20_guard_rule_positive() {
    let pred_thm = Kernel::refl(NodeId(1), TypeId(10));
    let then_thm = Kernel::refl(NodeId(2), TypeId(20));
    let else_thm = Kernel::refl(NodeId(3), TypeId(20));

    let result = Kernel::guard_rule(&pred_thm, &then_thm, &else_thm, NodeId(4)).unwrap();

    assert_eq!(result.type_ref(), TypeId(20));
    assert_eq!(result.node_id(), NodeId(4));
    // Lean: Sum(kp, Sup([kt, ke]))
    let expected_cost = CostBound::Sum(
        Box::new(CostBound::Zero),
        Box::new(CostBound::Sup(vec![CostBound::Zero, CostBound::Zero])),
    );
    assert_eq!(*result.cost(), expected_cost);
}

/// Negative: then and else have different types.
#[test]
fn rule20_guard_rule_negative_branch_mismatch() {
    let pred_thm = Kernel::refl(NodeId(1), TypeId(10));
    let then_thm = Kernel::refl(NodeId(2), TypeId(20));
    let else_thm = Kernel::refl(NodeId(3), TypeId(30)); // different!

    let err = Kernel::guard_rule(&pred_thm, &then_thm, &else_thm, NodeId(4)).unwrap_err();
    assert!(matches!(err, KernelError::TypeMismatch {
        expected: TypeId(20),
        actual: TypeId(30),
        ..
    }));
}

/// Edge case: guard with non-zero costs — verify the Sum/Sup structure.
#[test]
fn rule20_guard_rule_edge_nonzero_costs() {
    let pred_thm = Kernel::cost_subsume(
        &Kernel::refl(NodeId(1), TypeId(10)),
        CostBound::Constant(2),
    ).unwrap();
    let then_thm = Kernel::cost_subsume(
        &Kernel::refl(NodeId(2), TypeId(20)),
        CostBound::Constant(3),
    ).unwrap();
    let else_thm = Kernel::cost_subsume(
        &Kernel::refl(NodeId(3), TypeId(20)),
        CostBound::Constant(7),
    ).unwrap();

    let result = Kernel::guard_rule(&pred_thm, &then_thm, &else_thm, NodeId(4)).unwrap();

    let expected_cost = CostBound::Sum(
        Box::new(CostBound::Constant(2)),
        Box::new(CostBound::Sup(vec![CostBound::Constant(3), CostBound::Constant(7)])),
    );
    assert_eq!(*result.cost(), expected_cost);
}

// ===========================================================================
// Property-based tests
// ===========================================================================

/// Generate random SemanticGraphs and verify the checker doesn't panic.
#[test]
fn property_random_graphs_checker_doesnt_crash() {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    for trial in 0..1000 {
        // Random TypeEnv
        let num_types = rng.gen_range(1..=5);
        let mut type_defs = Vec::new();
        for i in 0..num_types {
            let id = TypeId(i as u64 + 1);
            let def = match rng.gen_range(0..3) {
                0 => TypeDef::Primitive(PrimType::Int),
                1 => TypeDef::Primitive(PrimType::Bool),
                _ => TypeDef::Primitive(PrimType::Unit),
            };
            type_defs.push((id, def));
        }

        // Random nodes
        let num_nodes = rng.gen_range(1..=3);
        let mut nodes = Vec::new();
        for j in 0..num_nodes {
            let node_id = NodeId(j as u64 + 100);
            let type_sig = TypeId(rng.gen_range(1..=(num_types as u64)));
            nodes.push(make_lit_node(node_id, type_sig));
        }

        let graph = graph_with_nodes(make_type_env(type_defs), nodes);

        // Run type_check_graded and verify it doesn't panic
        let report = iris_bootstrap::syntax::kernel::type_check_graded(
            &graph,
            iris_types::proof::VerifyTier::Tier0,
        );

        // Just verify it produced a result without panicking
        assert!(
            report.total_obligations >= 0,
            "trial {trial} panicked"
        );
    }
}

/// Verify cost lattice transitivity: if a<=b and b<=c then a<=c.
#[test]
fn property_cost_lattice_transitivity() {
    let n = CostVar(0);
    let costs = vec![
        CostBound::Zero,
        CostBound::Constant(1),
        CostBound::Constant(10),
        CostBound::Linear(n),
        CostBound::NLogN(n),
        CostBound::Polynomial(n, 2),
        CostBound::Polynomial(n, 3),
        CostBound::Unknown,
    ];

    for a in &costs {
        for b in &costs {
            for c in &costs {
                if cost_checker::cost_leq(a, b) && cost_checker::cost_leq(b, c) {
                    assert!(
                        cost_checker::cost_leq(a, c),
                        "transitivity violated: {a:?} <= {b:?} and {b:?} <= {c:?} but NOT {a:?} <= {c:?}"
                    );
                }
            }
        }
    }
}

/// Verify cost lattice antisymmetry: if a<=b and b<=a then a==b.
#[test]
fn property_cost_lattice_antisymmetry() {
    let n = CostVar(0);
    let costs = vec![
        CostBound::Zero,
        CostBound::Constant(1),
        CostBound::Constant(10),
        CostBound::Linear(n),
        CostBound::NLogN(n),
        CostBound::Polynomial(n, 2),
        CostBound::Polynomial(n, 3),
        CostBound::Unknown,
    ];

    for a in &costs {
        for b in &costs {
            if cost_checker::cost_leq(a, b) && cost_checker::cost_leq(b, a) {
                assert_eq!(
                    a, b,
                    "antisymmetry violated: {a:?} <= {b:?} and {b:?} <= {a:?} but {a:?} != {b:?}"
                );
            }
        }
    }
}

/// Verify cost lattice reflexivity: a <= a for all cost bounds.
#[test]
fn property_cost_lattice_reflexivity() {
    let n = CostVar(0);
    let costs = vec![
        CostBound::Zero,
        CostBound::Constant(1),
        CostBound::Constant(u64::MAX),
        CostBound::Linear(n),
        CostBound::NLogN(n),
        CostBound::Polynomial(n, 2),
        CostBound::Unknown,
        CostBound::Sum(Box::new(CostBound::Zero), Box::new(CostBound::Constant(1))),
        CostBound::Sup(vec![CostBound::Zero, CostBound::Constant(1)]),
    ];

    for c in &costs {
        assert!(
            cost_checker::cost_leq(c, c),
            "reflexivity violated: {c:?} is NOT <= itself"
        );
    }
}

/// Verify proof hash determinism: same inputs always produce same hash.
#[test]
fn property_proof_hash_deterministic() {
    for _ in 0..100 {
        let thm1 = Kernel::refl(NodeId(42), TypeId(99));
        let thm2 = Kernel::refl(NodeId(42), TypeId(99));
        assert_eq!(thm1.proof_hash(), thm2.proof_hash());
    }

    // Also verify for symm
    let base = Kernel::refl(NodeId(1), TypeId(10));
    let eq_w = Kernel::refl(NodeId(2), TypeId(10));
    let sym1 = Kernel::symm(&base, NodeId(2), &eq_w).unwrap();
    let sym2 = Kernel::symm(&base, NodeId(2), &eq_w).unwrap();
    assert_eq!(sym1.proof_hash(), sym2.proof_hash());

    // And trans
    let a = Kernel::refl(NodeId(1), TypeId(10));
    let b = Kernel::refl(NodeId(2), TypeId(10));
    let t1 = Kernel::trans(&a, &b).unwrap();
    let t2 = Kernel::trans(&a, &b).unwrap();
    assert_eq!(t1.proof_hash(), t2.proof_hash());
}

/// Verify that different rule applications produce different proof hashes.
#[test]
fn property_proof_hash_unique() {
    let thm_refl = Kernel::refl(NodeId(1), TypeId(10));
    let eq_witness = Kernel::refl(NodeId(2), TypeId(10));
    let thm_sym = Kernel::symm(&thm_refl, NodeId(2), &eq_witness).unwrap();
    assert_ne!(thm_refl.proof_hash(), thm_sym.proof_hash());

    let thm_a = Kernel::refl(NodeId(1), TypeId(10));
    let thm_b = Kernel::refl(NodeId(1), TypeId(20));
    assert_ne!(thm_a.proof_hash(), thm_b.proof_hash());
}

// ===========================================================================
// Cross-validation: Lean CostLeq constructors vs Rust cost_leq
// ===========================================================================

/// Lean CostLeq.zero_bot: Zero is bottom element (Zero <= anything except Unknown).
#[test]
fn lean_costleq_zero_bot() {
    let n = CostVar(0);
    assert!(cost_checker::cost_leq(&CostBound::Zero, &CostBound::Zero));
    assert!(cost_checker::cost_leq(&CostBound::Zero, &CostBound::Constant(0)));
    assert!(cost_checker::cost_leq(&CostBound::Zero, &CostBound::Constant(u64::MAX)));
    assert!(cost_checker::cost_leq(&CostBound::Zero, &CostBound::Linear(n)));
    assert!(cost_checker::cost_leq(&CostBound::Zero, &CostBound::NLogN(n)));
    assert!(cost_checker::cost_leq(&CostBound::Zero, &CostBound::Polynomial(n, 1)));
    // Unknown is no longer a valid upper bound (soundness fix).
    assert!(!cost_checker::cost_leq(&CostBound::Zero, &CostBound::Unknown));
}

/// Unknown only equals itself; it is NOT a top element for subsumption.
/// Programs cannot claim Unknown cost to bypass the cost system.
#[test]
fn lean_costleq_unknown_not_top() {
    let n = CostVar(0);
    // Nothing <= Unknown (except Unknown itself via equality).
    assert!(!cost_checker::cost_leq(&CostBound::Zero, &CostBound::Unknown));
    assert!(!cost_checker::cost_leq(&CostBound::Constant(u64::MAX), &CostBound::Unknown));
    assert!(!cost_checker::cost_leq(&CostBound::Linear(n), &CostBound::Unknown));
    assert!(!cost_checker::cost_leq(&CostBound::NLogN(n), &CostBound::Unknown));
    assert!(!cost_checker::cost_leq(&CostBound::Polynomial(n, 100), &CostBound::Unknown));
    // Only Unknown == Unknown is valid.
    assert!(cost_checker::cost_leq(&CostBound::Unknown, &CostBound::Unknown));
    // Unknown is NOT <= non-Unknown
    assert!(!cost_checker::cost_leq(&CostBound::Unknown, &CostBound::Constant(u64::MAX)));
}

/// Lean CostLeq.const_le: Constant ordering by value.
#[test]
fn lean_costleq_const_le() {
    assert!(cost_checker::cost_leq(&CostBound::Constant(0), &CostBound::Constant(0)));
    assert!(cost_checker::cost_leq(&CostBound::Constant(1), &CostBound::Constant(2)));
    assert!(cost_checker::cost_leq(&CostBound::Constant(5), &CostBound::Constant(100)));
    assert!(!cost_checker::cost_leq(&CostBound::Constant(10), &CostBound::Constant(5)));
}

/// Lean CostLeq.const_linear: Constant <= Linear (any constant, same var).
#[test]
fn lean_costleq_const_linear() {
    let n = CostVar(0);
    assert!(cost_checker::cost_leq(&CostBound::Constant(0), &CostBound::Linear(n)));
    assert!(cost_checker::cost_leq(&CostBound::Constant(999), &CostBound::Linear(n)));
}

/// Lean CostLeq.const_nlogn: Constant <= NLogN.
#[test]
fn lean_costleq_const_nlogn() {
    let n = CostVar(0);
    assert!(cost_checker::cost_leq(&CostBound::Constant(0), &CostBound::NLogN(n)));
    assert!(cost_checker::cost_leq(&CostBound::Constant(999), &CostBound::NLogN(n)));
}

/// Lean CostLeq.const_poly: Constant <= Polynomial.
#[test]
fn lean_costleq_const_poly() {
    let n = CostVar(0);
    assert!(cost_checker::cost_leq(&CostBound::Constant(0), &CostBound::Polynomial(n, 1)));
    assert!(cost_checker::cost_leq(&CostBound::Constant(999), &CostBound::Polynomial(n, 5)));
}

/// Lean CostLeq.linear_nlogn: Linear <= NLogN (same variable).
#[test]
fn lean_costleq_linear_nlogn() {
    let n = CostVar(0);
    assert!(cost_checker::cost_leq(&CostBound::Linear(n), &CostBound::NLogN(n)));
    // Different variables: not ordered
    assert!(!cost_checker::cost_leq(&CostBound::Linear(CostVar(0)), &CostBound::NLogN(CostVar(1))));
}

/// Lean CostLeq.linear_poly: Linear <= Polynomial (same variable).
#[test]
fn lean_costleq_linear_poly() {
    let n = CostVar(0);
    assert!(cost_checker::cost_leq(&CostBound::Linear(n), &CostBound::Polynomial(n, 1)));
    assert!(cost_checker::cost_leq(&CostBound::Linear(n), &CostBound::Polynomial(n, 5)));
}

/// Lean CostLeq.nlogn_poly: NLogN <= Polynomial(v, d) when d >= 2.
#[test]
fn lean_costleq_nlogn_poly() {
    let n = CostVar(0);
    assert!(cost_checker::cost_leq(&CostBound::NLogN(n), &CostBound::Polynomial(n, 2)));
    assert!(cost_checker::cost_leq(&CostBound::NLogN(n), &CostBound::Polynomial(n, 5)));
    // d=1: NLogN is NOT <= Polynomial(n, 1) in the Rust implementation
    // (NLogN grows faster than linear)
    assert!(!cost_checker::cost_leq(&CostBound::NLogN(n), &CostBound::Polynomial(n, 1)));
}

/// Lean CostLeq.poly_le: Polynomial(v,d1) <= Polynomial(v,d2) when d1 <= d2.
#[test]
fn lean_costleq_poly_le() {
    let n = CostVar(0);
    assert!(cost_checker::cost_leq(&CostBound::Polynomial(n, 1), &CostBound::Polynomial(n, 1)));
    assert!(cost_checker::cost_leq(&CostBound::Polynomial(n, 2), &CostBound::Polynomial(n, 3)));
    assert!(!cost_checker::cost_leq(&CostBound::Polynomial(n, 3), &CostBound::Polynomial(n, 2)));
}

/// Lean CostLeq.sum_le: pointwise comparison of Sum.
#[test]
fn lean_costleq_sum_le() {
    let sum_a = CostBound::Sum(
        Box::new(CostBound::Constant(1)),
        Box::new(CostBound::Constant(2)),
    );
    let sum_b = CostBound::Sum(
        Box::new(CostBound::Constant(3)),
        Box::new(CostBound::Constant(4)),
    );
    assert!(cost_checker::cost_leq(&sum_a, &sum_b));
    assert!(!cost_checker::cost_leq(&sum_b, &sum_a));
}

/// Lean CostLeq.sup_le: Sup(vs) <= x iff ALL v in vs satisfy v <= x.
#[test]
fn lean_costleq_sup_le() {
    let n = CostVar(0);
    let sup = CostBound::Sup(vec![CostBound::Constant(5), CostBound::Constant(10)]);
    assert!(cost_checker::cost_leq(&sup, &CostBound::Constant(10)));
    assert!(cost_checker::cost_leq(&sup, &CostBound::Linear(n)));
    // Fails when any element exceeds the bound
    assert!(!cost_checker::cost_leq(&sup, &CostBound::Constant(5)));
}

// ===========================================================================
// End-to-end correspondence: multi-rule derivation chains
// ===========================================================================

/// Chain: intro + elim roundtrip (lambda applied to argument).
/// Lean: intro produces arrow, elim consumes it, result = return type.
#[test]
fn e2e_intro_elim_roundtrip() {
    let int_id = TypeId(1);
    let bool_id = TypeId(3);
    let arrow_id = TypeId(2);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (bool_id, TypeDef::Primitive(PrimType::Bool)),
        (arrow_id, TypeDef::Arrow(int_id, bool_id, CostBound::Zero)),
    ]);
    let graph = graph_with_types(type_env);

    // Step 1: intro
    let ctx = Context::empty();
    let extended = ctx.extend(BinderId(0), int_id);
    // Body has type Bool in extended context. Use refl in empty context, then
    // we need a theorem about Bool in extended context. We'll use assume on a
    // further-extended context, but that changes things. Instead, use symm.
    // Actually the simplest: assume BinderId(0) gives Int. We want Bool.
    // Let's just extend with Bool too.
    let extended2 = ctx.extend(BinderId(0), int_id);
    // We need a Bool theorem in extended2 context. We can't easily get one
    // without a Bool binding. Let's add one.
    let extended3 = ctx.extend(BinderId(0), int_id).extend(BinderId(1), bool_id);
    let body_thm = Kernel::assume(&extended3, BinderId(1), NodeId(10)).unwrap();
    // This won't match because intro expects ctx.extend(binder, type) == body_thm.context
    // Let's simplify: identity function Int -> Int
    let body_thm = Kernel::assume(&extended, BinderId(0), NodeId(10)).unwrap();
    let arrow_id_int = TypeId(2);
    // Redefine with Int -> Int
    let type_env2 = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (arrow_id_int, TypeDef::Arrow(int_id, int_id, CostBound::Zero)),
    ]);
    let graph2 = graph_with_types(type_env2);

    let lam_thm = Kernel::intro(&ctx, NodeId(20), BinderId(0), int_id, &body_thm, &graph2).unwrap();
    assert_eq!(lam_thm.type_ref(), arrow_id_int);

    // Step 2: elim (apply lambda to an argument)
    let arg_thm = Kernel::refl(NodeId(30), int_id);
    let app_thm = Kernel::elim(&lam_thm, &arg_thm, NodeId(40), &graph2).unwrap();
    assert_eq!(app_thm.type_ref(), int_id); // return type
}

/// Chain: refine_intro + refine_elim roundtrip.
#[test]
fn e2e_refine_roundtrip() {
    let int_id = TypeId(1);
    let refined_id = TypeId(2);
    let pred = LIAFormula::True;
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (refined_id, TypeDef::Refined(int_id, pred)),
    ]);
    let graph = graph_with_types(type_env);

    // Introduce refinement (witness must be about the same node as base).
    let base = Kernel::refl(NodeId(1), int_id);
    let witness = Kernel::refl(NodeId(1), int_id);
    let refined = Kernel::refine_intro(&base, &witness, refined_id, &graph).unwrap();
    assert_eq!(refined.type_ref(), refined_id);

    // Eliminate refinement
    let (base_back, _) = Kernel::refine_elim(&refined, &graph).unwrap();
    assert_eq!(base_back.type_ref(), int_id);
    assert_eq!(base_back.node_id(), NodeId(1));
}

/// Chain: type_abst + type_app roundtrip (ForAll intro + elim).
#[test]
fn e2e_forall_roundtrip() {
    let int_id = TypeId(1);
    let forall_id = TypeId(2);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
        (forall_id, TypeDef::ForAll(BoundVar(0), int_id)),
    ]);
    let graph = graph_with_types(type_env);

    // type_abst: Int -> ForAll(X, Int)
    let body = Kernel::refl(NodeId(1), int_id);
    let abstracted = Kernel::type_abst(&body, forall_id, &graph).unwrap();
    assert_eq!(abstracted.type_ref(), forall_id);

    // type_app: ForAll(X, Int) -> Int
    let applied = Kernel::type_app(&abstracted, int_id, &graph).unwrap();
    assert_eq!(applied.type_ref(), int_id);
}

/// Chain: cost_subsume + cost_leq_rule verify cost lattice integration.
#[test]
fn e2e_cost_chain() {
    let n = CostVar(0);

    // Start at Zero
    let thm = Kernel::refl(NodeId(1), TypeId(10));
    assert_eq!(*thm.cost(), CostBound::Zero);

    // Weaken to Constant(5)
    let thm = Kernel::cost_subsume(&thm, CostBound::Constant(5)).unwrap();
    assert_eq!(*thm.cost(), CostBound::Constant(5));

    // Weaken to Linear(n)
    let thm = Kernel::cost_subsume(&thm, CostBound::Linear(n)).unwrap();
    assert_eq!(*thm.cost(), CostBound::Linear(n));

    // Weaken to NLogN(n)
    let thm = Kernel::cost_subsume(&thm, CostBound::NLogN(n)).unwrap();
    assert_eq!(*thm.cost(), CostBound::NLogN(n));

    // Weaken to Polynomial(n, 2)
    let thm = Kernel::cost_subsume(&thm, CostBound::Polynomial(n, 2)).unwrap();
    assert_eq!(*thm.cost(), CostBound::Polynomial(n, 2));

    // Unknown is no longer a valid upper bound (soundness fix).
    let result = Kernel::cost_subsume(&thm, CostBound::Unknown);
    assert!(result.is_err(), "cost_subsume to Unknown should fail");

    // Verify each step was witnessed by cost_leq_rule
    assert!(Kernel::cost_leq_rule(&CostBound::Zero, &CostBound::Constant(5)).is_ok());
    assert!(Kernel::cost_leq_rule(&CostBound::Constant(5), &CostBound::Linear(n)).is_ok());
    assert!(Kernel::cost_leq_rule(&CostBound::Linear(n), &CostBound::NLogN(n)).is_ok());
    assert!(Kernel::cost_leq_rule(&CostBound::NLogN(n), &CostBound::Polynomial(n, 2)).is_ok());
    // Polynomial <= Unknown no longer holds.
    assert!(Kernel::cost_leq_rule(&CostBound::Polynomial(n, 2), &CostBound::Unknown).is_err());
}

/// Structural induction with empty sum type is rejected.
/// Lean: requires 0 < variants.length.
#[test]
fn structural_ind_empty_sum_rejected() {
    let sum_id = TypeId(1);
    let type_env = make_type_env(vec![
        (sum_id, TypeDef::Sum(vec![])), // empty sum
    ]);
    let graph = graph_with_types(type_env);

    let err = Kernel::structural_ind(sum_id, &[], NodeId(1), &graph).unwrap_err();
    assert!(matches!(err, KernelError::InductionError { .. }));
}

/// Verify that structural_ind type is NOT Sum is rejected.
#[test]
fn structural_ind_not_sum_rejected() {
    let int_id = TypeId(1);
    let type_env = make_type_env(vec![
        (int_id, TypeDef::Primitive(PrimType::Int)),
    ]);
    let graph = graph_with_types(type_env);

    let case = Kernel::refl(NodeId(1), int_id);
    let err = Kernel::structural_ind(int_id, &[case], NodeId(2), &graph).unwrap_err();
    assert!(matches!(err, KernelError::UnexpectedTypeDef { expected: "Sum", .. }));
}

/// Verify node not found error in type_check_node.
#[test]
fn type_check_node_not_found() {
    let type_env = make_type_env(vec![]);
    let graph = graph_with_nodes(type_env, vec![]); // no nodes
    let ctx = Context::empty();

    let err = Kernel::type_check_node(&ctx, &graph, NodeId(999)).unwrap_err();
    assert!(matches!(err, KernelError::NodeNotFound(NodeId(999))));
}
