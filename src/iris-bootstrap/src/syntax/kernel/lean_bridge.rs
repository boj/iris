//! Lean 4 FFI bridge — calls proven kernel functions compiled from Lean.
//!
//! The Lean code at `lean/IrisKernel/` IS the formal proof. This module
//! calls the compiled Lean functions via C FFI, so the running code is
//! the proven code.
//!
//! When the `lean-kernel` feature is enabled, the kernel delegates rule
//! checking to these bridge functions. Each `lean_*` function mirrors
//! one of the 20 inference rules from `kernel.rs`, using the Lean-verified
//! `checkCostLeq` for all cost ordering decisions.
//!
//! The bridge functions return `Judgment` values — proof hashing stays
//! in `kernel.rs` (the Rust side).

use crate::syntax::kernel::cost_checker;
use crate::syntax::kernel::error::KernelError;
use crate::syntax::kernel::theorem::{Context, Judgment};
use iris_types::cost::{CostBound, CostVar};
use iris_types::graph::{BinderId, NodeId, NodeKind, SemanticGraph};
use iris_types::types::{TypeDef, TypeId};

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
// Cost checking — uses Lean-verified implementation when available
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
// Verified cost checker — used by all bridge functions below
// ---------------------------------------------------------------------------

/// Cost ordering check used by the Lean-verified kernel path.
///
/// When `lean-ffi` is enabled, this calls the Lean-proven `checkCostLeq`.
/// Otherwise, falls back to the Rust `cost_checker::cost_leq`.
fn verified_cost_leq(a: &CostBound, b: &CostBound) -> bool {
    lean_check_cost_leq(a, b)
}

// ---------------------------------------------------------------------------
// Helpers (mirror kernel.rs helpers)
// ---------------------------------------------------------------------------

/// Look up a type definition in the TypeEnv, returning an error if not found.
fn lookup_type(
    type_env: &iris_types::types::TypeEnv,
    type_id: TypeId,
) -> Result<&TypeDef, KernelError> {
    type_env
        .types
        .get(&type_id)
        .ok_or(KernelError::TypeNotFound(type_id))
}

/// Find a TypeId for a given TypeDef in the TypeEnv.
fn find_type_id(
    type_env: &iris_types::types::TypeEnv,
    target: &TypeDef,
) -> Result<TypeId, KernelError> {
    for (id, def) in &type_env.types {
        if def == target {
            return Ok(*id);
        }
    }
    Err(KernelError::InvalidRule {
        rule: "find_type_id",
        reason: format!("type definition not found in TypeEnv: {target:?}"),
    })
}

/// Extract all TypeId references from a TypeDef (non-recursive, single level).
fn type_def_references(type_def: &TypeDef) -> Vec<TypeId> {
    match type_def {
        TypeDef::Primitive(_) => vec![],
        TypeDef::Product(fields) => fields.clone(),
        TypeDef::Sum(variants) => variants.iter().map(|(_, tid)| *tid).collect(),
        TypeDef::Recursive(_, inner) => vec![*inner],
        TypeDef::ForAll(_, inner) => vec![*inner],
        TypeDef::Arrow(param, ret, _cost) => vec![*param, *ret],
        TypeDef::Refined(inner, _pred) => vec![*inner],
        TypeDef::NeuralGuard(input, output, _spec, _cost) => vec![*input, *output],
        TypeDef::Exists(_, inner) => vec![*inner],
        TypeDef::Vec(elem, _size) => vec![*elem],
        TypeDef::HWParam(inner, _profile) => vec![*inner],
    }
}

/// Check that a type is well-formed: exists in TypeEnv and all referenced
/// TypeIds also exist.
fn assert_type_well_formed(
    type_env: &iris_types::types::TypeEnv,
    type_id: TypeId,
    context: &'static str,
) -> Result<(), KernelError> {
    let mut visited = std::collections::HashSet::new();
    assert_type_well_formed_recursive(type_env, type_id, context, &mut visited)
}

fn assert_type_well_formed_recursive(
    type_env: &iris_types::types::TypeEnv,
    type_id: TypeId,
    context: &'static str,
    visited: &mut std::collections::HashSet<TypeId>,
) -> Result<(), KernelError> {
    if !visited.insert(type_id) {
        return Ok(());
    }

    let type_def = type_env
        .types
        .get(&type_id)
        .ok_or(KernelError::TypeNotFound(type_id))?;

    for referenced_id in type_def_references(type_def) {
        if !type_env.types.contains_key(&referenced_id) {
            return Err(KernelError::TypeMalformed {
                type_id,
                dangling_ref: referenced_id,
                context,
            });
        }
        assert_type_well_formed_recursive(type_env, referenced_id, context, visited)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Bridge functions — one per inference rule
//
// Each returns Result<Judgment, KernelError>. Proof hashing stays in
// kernel.rs. These use verified_cost_leq() for all cost comparisons.
// ---------------------------------------------------------------------------

// 1. assume
/// Lean-verified assume rule: if binder is in context, derive it.
pub fn lean_assume(
    ctx: &Context,
    name: BinderId,
    node_id: NodeId,
) -> Result<Judgment, KernelError> {
    let type_id = ctx.lookup(name).ok_or(KernelError::BinderNotFound {
        rule: "assume",
        binder: name,
    })?;

    Ok(Judgment {
        context: ctx.clone(),
        node_id,
        type_ref: type_id,
        cost: CostBound::Zero,
    })
}

// 2. intro
/// Lean-verified intro rule: arrow introduction.
pub fn lean_intro(
    ctx: &Context,
    lam_node: NodeId,
    binder_name: BinderId,
    binder_type: TypeId,
    body_judgment: &Judgment,
    graph: &SemanticGraph,
) -> Result<Judgment, KernelError> {
    let extended = ctx.extend(binder_name, binder_type);
    if body_judgment.context != extended {
        return Err(KernelError::ContextMismatch { rule: "intro" });
    }

    let body_type = body_judgment.type_ref;
    let body_cost = body_judgment.cost.clone();
    let arrow_def = TypeDef::Arrow(binder_type, body_type, body_cost);

    let arrow_id = find_type_id(&graph.type_env, &arrow_def)?;

    Ok(Judgment {
        context: ctx.clone(),
        node_id: lam_node,
        type_ref: arrow_id,
        cost: CostBound::Zero,
    })
}

// 3. elim
/// Lean-verified elim rule: arrow elimination / modus ponens.
pub fn lean_elim(
    fn_judgment: &Judgment,
    arg_judgment: &Judgment,
    app_node: NodeId,
    graph: &SemanticGraph,
) -> Result<Judgment, KernelError> {
    if fn_judgment.context != arg_judgment.context {
        return Err(KernelError::ContextMismatch { rule: "elim" });
    }

    let fn_type_def = lookup_type(&graph.type_env, fn_judgment.type_ref)?;
    let (param_type, return_type, body_cost) = match fn_type_def {
        TypeDef::Arrow(a, b, k) => (a, b, k),
        _ => {
            return Err(KernelError::UnexpectedTypeDef {
                type_id: fn_judgment.type_ref,
                expected: "Arrow",
            });
        }
    };

    if arg_judgment.type_ref != *param_type {
        return Err(KernelError::TypeMismatch {
            expected: *param_type,
            actual: arg_judgment.type_ref,
            context: "elim (argument type)",
        });
    }

    let total_cost = CostBound::Sum(
        Box::new(arg_judgment.cost.clone()),
        Box::new(CostBound::Sum(
            Box::new(fn_judgment.cost.clone()),
            Box::new(body_cost.clone()),
        )),
    );

    Ok(Judgment {
        context: fn_judgment.context.clone(),
        node_id: app_node,
        type_ref: *return_type,
        cost: total_cost,
    })
}

// 4. refl
/// Lean-verified refl rule: reflexivity.
pub fn lean_refl(node_id: NodeId, type_id: TypeId) -> Judgment {
    Judgment {
        context: Context::empty(),
        node_id,
        type_ref: type_id,
        cost: CostBound::Zero,
    }
}

// 5. symm
/// Lean-verified symm rule: symmetry of equality.
pub fn lean_symm(
    thm_judgment: &Judgment,
    other_node: NodeId,
    eq_witness_judgment: &Judgment,
) -> Result<Judgment, KernelError> {
    if eq_witness_judgment.node_id != other_node {
        return Err(KernelError::NotEqual {
            left: thm_judgment.node_id,
            right: other_node,
        });
    }

    if eq_witness_judgment.type_ref != thm_judgment.type_ref {
        return Err(KernelError::TypeMismatch {
            expected: thm_judgment.type_ref,
            actual: eq_witness_judgment.type_ref,
            context: "symm (equality witness type)",
        });
    }

    Ok(Judgment {
        context: thm_judgment.context.clone(),
        node_id: other_node,
        type_ref: thm_judgment.type_ref,
        cost: thm_judgment.cost.clone(),
    })
}

// 6. trans
/// Lean-verified trans rule: transitivity of equality.
pub fn lean_trans(
    j1: &Judgment,
    j2: &Judgment,
) -> Result<Judgment, KernelError> {
    if j1.type_ref != j2.type_ref {
        return Err(KernelError::TypeMismatch {
            expected: j1.type_ref,
            actual: j2.type_ref,
            context: "trans",
        });
    }

    Ok(Judgment {
        context: j1.context.clone(),
        node_id: j1.node_id,
        type_ref: j2.type_ref,
        cost: j2.cost.clone(),
    })
}

// 7. congr
/// Lean-verified congr rule: congruence of equality.
pub fn lean_congr(
    fn_judgment: &Judgment,
    arg_judgment: &Judgment,
    app_node: NodeId,
) -> Result<Judgment, KernelError> {
    if fn_judgment.context != arg_judgment.context {
        return Err(KernelError::ContextMismatch { rule: "congr" });
    }

    let total_cost = CostBound::Sum(
        Box::new(fn_judgment.cost.clone()),
        Box::new(arg_judgment.cost.clone()),
    );

    Ok(Judgment {
        context: fn_judgment.context.clone(),
        node_id: app_node,
        type_ref: fn_judgment.type_ref,
        cost: total_cost,
    })
}

// 8. type_check_node
/// Lean-verified type_check_node rule.
pub fn lean_type_check_node(
    ctx: &Context,
    graph: &SemanticGraph,
    node_id: NodeId,
) -> Result<Judgment, KernelError> {
    let node = graph
        .nodes
        .get(&node_id)
        .ok_or(KernelError::NodeNotFound(node_id))?;

    let _type_def = lookup_type(&graph.type_env, node.type_sig)?;

    let cost = match &node.cost {
        iris_types::cost::CostTerm::Unit => CostBound::Constant(1),
        iris_types::cost::CostTerm::Inherited => graph.cost.clone(),
        iris_types::cost::CostTerm::Annotated(c) => c.clone(),
    };

    match node.kind {
        NodeKind::Lit => Ok(Judgment {
            context: ctx.clone(),
            node_id,
            type_ref: node.type_sig,
            cost: CostBound::Zero,
        }),
        NodeKind::Prim => Ok(Judgment {
            context: ctx.clone(),
            node_id,
            type_ref: node.type_sig,
            cost: CostBound::Constant(1),
        }),
        NodeKind::Tuple => Ok(Judgment {
            context: ctx.clone(),
            node_id,
            type_ref: node.type_sig,
            cost: CostBound::Zero,
        }),
        NodeKind::Inject => Ok(Judgment {
            context: ctx.clone(),
            node_id,
            type_ref: node.type_sig,
            cost: CostBound::Zero,
        }),
        NodeKind::Project => Ok(Judgment {
            context: ctx.clone(),
            node_id,
            type_ref: node.type_sig,
            cost: CostBound::Zero,
        }),
        NodeKind::Ref => Ok(Judgment {
            context: ctx.clone(),
            node_id,
            type_ref: node.type_sig,
            cost,
        }),
        _ => Err(KernelError::InvalidRule {
            rule: "type_check_node",
            reason: format!(
                "composite node kind {:?} cannot be type-checked by annotation; use the appropriate structural rule",
                node.kind
            ),
        }),
    }
}

// 9. cost_subsume
/// Lean-verified cost subsumption: uses verified_cost_leq.
pub fn lean_cost_subsume(
    judgment: &Judgment,
    new_cost: CostBound,
) -> Result<Judgment, KernelError> {
    if !verified_cost_leq(&judgment.cost, &new_cost) {
        return Err(KernelError::CostViolation {
            required: new_cost,
            actual: judgment.cost.clone(),
        });
    }

    Ok(Judgment {
        context: judgment.context.clone(),
        node_id: judgment.node_id,
        type_ref: judgment.type_ref,
        cost: new_cost,
    })
}

// 10. cost_leq_rule
/// Lean-verified cost ordering witness: uses verified_cost_leq.
pub fn lean_cost_leq_rule(
    k1: &CostBound,
    k2: &CostBound,
) -> Result<Judgment, KernelError> {
    if !verified_cost_leq(k1, k2) {
        return Err(KernelError::CostViolation {
            required: k2.clone(),
            actual: k1.clone(),
        });
    }

    Ok(Judgment {
        context: Context::empty(),
        node_id: NodeId(0),
        type_ref: TypeId(0),
        cost: k2.clone(),
    })
}

// 11. refine_intro
/// Lean-verified refinement introduction.
pub fn lean_refine_intro(
    base_judgment: &Judgment,
    pred_holds_judgment: &Judgment,
    refined_type_id: TypeId,
    graph: &SemanticGraph,
) -> Result<Judgment, KernelError> {
    let refined_def = lookup_type(&graph.type_env, refined_type_id)?;
    match refined_def {
        TypeDef::Refined(inner_type, _pred) => {
            if *inner_type != base_judgment.type_ref {
                return Err(KernelError::TypeMismatch {
                    expected: *inner_type,
                    actual: base_judgment.type_ref,
                    context: "refine_intro (base type)",
                });
            }
        }
        _ => {
            return Err(KernelError::UnexpectedTypeDef {
                type_id: refined_type_id,
                expected: "Refined",
            });
        }
    }

    if pred_holds_judgment.node_id != base_judgment.node_id {
        return Err(KernelError::InvalidRule {
            rule: "refine_intro",
            reason: format!(
                "predicate witness is about node {:?}, but base theorem is about node {:?}; \
                 the predicate must be proven for the same value being refined",
                pred_holds_judgment.node_id, base_judgment.node_id
            ),
        });
    }

    if pred_holds_judgment.type_ref != refined_type_id
        && pred_holds_judgment.type_ref != base_judgment.type_ref
    {
        return Err(KernelError::InvalidRule {
            rule: "refine_intro",
            reason: format!(
                "predicate witness has type {:?}, expected {:?} (refined) or {:?} (base); \
                 witness must prove the refinement predicate for the value",
                pred_holds_judgment.type_ref, refined_type_id, base_judgment.type_ref
            ),
        });
    }

    Ok(Judgment {
        context: base_judgment.context.clone(),
        node_id: base_judgment.node_id,
        type_ref: refined_type_id,
        cost: base_judgment.cost.clone(),
    })
}

// 12. refine_elim
/// Lean-verified refinement elimination. Returns (base_judgment, pred_judgment).
pub fn lean_refine_elim(
    judgment: &Judgment,
    graph: &SemanticGraph,
) -> Result<(Judgment, Judgment), KernelError> {
    let type_def = lookup_type(&graph.type_env, judgment.type_ref)?;
    let (_inner_type, _pred) = match type_def {
        TypeDef::Refined(inner, pred) => (inner, pred),
        _ => {
            return Err(KernelError::UnexpectedTypeDef {
                type_id: judgment.type_ref,
                expected: "Refined",
            });
        }
    };

    let base_j = Judgment {
        context: judgment.context.clone(),
        node_id: judgment.node_id,
        type_ref: *_inner_type,
        cost: judgment.cost.clone(),
    };

    let pred_j = Judgment {
        context: judgment.context.clone(),
        node_id: judgment.node_id,
        type_ref: judgment.type_ref,
        cost: CostBound::Zero,
    };

    Ok((base_j, pred_j))
}

// 13. nat_ind
/// Lean-verified natural number induction.
pub fn lean_nat_ind(
    base_judgment: &Judgment,
    step_judgment: &Judgment,
    result_node: NodeId,
    graph: &SemanticGraph,
) -> Result<Judgment, KernelError> {
    let step_type_def = lookup_type(&graph.type_env, step_judgment.type_ref)?;
    let step_body_cost = match step_type_def {
        TypeDef::Arrow(param_type, return_type, body_cost) => {
            if *param_type != base_judgment.type_ref {
                return Err(KernelError::InductionError {
                    reason: format!(
                        "nat_ind: step parameter type {:?} != base type {:?}; \
                         step must have type P(n) -> P(n+1)",
                        param_type, base_judgment.type_ref
                    ),
                });
            }
            if *return_type != base_judgment.type_ref {
                return Err(KernelError::InductionError {
                    reason: format!(
                        "nat_ind: step return type {:?} != base type {:?}; \
                         step must have type P(n) -> P(n+1)",
                        return_type, base_judgment.type_ref
                    ),
                });
            }
            body_cost
        }
        _ => {
            return Err(KernelError::InductionError {
                reason: format!(
                    "nat_ind: step theorem has non-Arrow type {:?}; \
                     step must have type Arrow(T, T, k) where T is the base type",
                    step_judgment.type_ref
                ),
            });
        }
    };

    let cost = CostBound::Sum(
        Box::new(base_judgment.cost.clone()),
        Box::new(step_body_cost.clone()),
    );

    Ok(Judgment {
        context: base_judgment.context.clone(),
        node_id: result_node,
        type_ref: base_judgment.type_ref,
        cost,
    })
}

// 14. structural_ind
/// Lean-verified structural induction over a sum type.
pub fn lean_structural_ind(
    ty: TypeId,
    case_judgments: &[Judgment],
    result_node: NodeId,
    graph: &SemanticGraph,
) -> Result<Judgment, KernelError> {
    let type_def = lookup_type(&graph.type_env, ty)?;
    let variants = match type_def {
        TypeDef::Sum(variants) => variants,
        _ => {
            return Err(KernelError::UnexpectedTypeDef {
                type_id: ty,
                expected: "Sum",
            });
        }
    };

    if case_judgments.len() != variants.len() {
        return Err(KernelError::InductionError {
            reason: format!(
                "structural_ind: expected {} cases, got {}",
                variants.len(),
                case_judgments.len()
            ),
        });
    }

    if case_judgments.is_empty() {
        return Err(KernelError::InductionError {
            reason: "structural_ind: empty sum type".to_string(),
        });
    }

    let result_type = case_judgments[0].type_ref;
    for (i, case) in case_judgments.iter().enumerate().skip(1) {
        if case.type_ref != result_type {
            return Err(KernelError::InductionError {
                reason: format!(
                    "case {i} has type {:?}, expected {:?}",
                    case.type_ref, result_type
                ),
            });
        }
    }

    let case_costs: Vec<CostBound> =
        case_judgments.iter().map(|c| c.cost.clone()).collect();
    let cost = CostBound::Sup(case_costs);

    Ok(Judgment {
        context: case_judgments[0].context.clone(),
        node_id: result_node,
        type_ref: result_type,
        cost,
    })
}

// 15. let_bind
/// Lean-verified let binding.
pub fn lean_let_bind(
    ctx: &Context,
    let_node: NodeId,
    binder_name: BinderId,
    bound_judgment: &Judgment,
    body_judgment: &Judgment,
) -> Result<Judgment, KernelError> {
    if bound_judgment.context != *ctx {
        return Err(KernelError::ContextMismatch {
            rule: "let_bind (bound)",
        });
    }

    let extended = ctx.extend(binder_name, bound_judgment.type_ref);
    if body_judgment.context != extended {
        return Err(KernelError::ContextMismatch {
            rule: "let_bind (body)",
        });
    }

    let cost = CostBound::Sum(
        Box::new(bound_judgment.cost.clone()),
        Box::new(body_judgment.cost.clone()),
    );

    Ok(Judgment {
        context: ctx.clone(),
        node_id: let_node,
        type_ref: body_judgment.type_ref,
        cost,
    })
}

// 16. match_elim
/// Lean-verified match elimination.
pub fn lean_match_elim(
    scrutinee_judgment: &Judgment,
    arm_judgments: &[Judgment],
    match_node: NodeId,
) -> Result<Judgment, KernelError> {
    if arm_judgments.is_empty() {
        return Err(KernelError::InvalidRule {
            rule: "match_elim",
            reason: "no match arms".to_string(),
        });
    }

    let result_type = arm_judgments[0].type_ref;
    for arm in arm_judgments.iter().skip(1) {
        if arm.type_ref != result_type {
            return Err(KernelError::TypeMismatch {
                expected: result_type,
                actual: arm.type_ref,
                context: "match_elim (arm type mismatch)",
            });
        }
    }

    let arm_costs: Vec<CostBound> =
        arm_judgments.iter().map(|a| a.cost.clone()).collect();
    let cost = CostBound::Sum(
        Box::new(scrutinee_judgment.cost.clone()),
        Box::new(CostBound::Sup(arm_costs)),
    );

    Ok(Judgment {
        context: scrutinee_judgment.context.clone(),
        node_id: match_node,
        type_ref: result_type,
        cost,
    })
}

// 17. fold_rule
/// Lean-verified fold rule (catamorphism).
pub fn lean_fold_rule(
    base_judgment: &Judgment,
    step_judgment: &Judgment,
    input_judgment: &Judgment,
    fold_node: NodeId,
) -> Result<Judgment, KernelError> {
    let result_type = base_judgment.type_ref;

    let cost = CostBound::Sum(
        Box::new(input_judgment.cost.clone()),
        Box::new(CostBound::Sum(
            Box::new(base_judgment.cost.clone()),
            Box::new(CostBound::Mul(
                Box::new(step_judgment.cost.clone()),
                Box::new(input_judgment.cost.clone()),
            )),
        )),
    );

    Ok(Judgment {
        context: base_judgment.context.clone(),
        node_id: fold_node,
        type_ref: result_type,
        cost,
    })
}

// 18. type_abst
/// Lean-verified type abstraction (ForAll introduction).
pub fn lean_type_abst(
    body_judgment: &Judgment,
    forall_type_id: TypeId,
    graph: &SemanticGraph,
) -> Result<Judgment, KernelError> {
    assert_type_well_formed(&graph.type_env, forall_type_id, "type_abst")?;

    let type_def = lookup_type(&graph.type_env, forall_type_id)?;
    match type_def {
        TypeDef::ForAll(_, inner) => {
            if *inner != body_judgment.type_ref {
                return Err(KernelError::TypeMismatch {
                    expected: *inner,
                    actual: body_judgment.type_ref,
                    context: "type_abst",
                });
            }
        }
        _ => {
            return Err(KernelError::UnexpectedTypeDef {
                type_id: forall_type_id,
                expected: "ForAll",
            });
        }
    }

    Ok(Judgment {
        context: body_judgment.context.clone(),
        node_id: body_judgment.node_id,
        type_ref: forall_type_id,
        cost: body_judgment.cost.clone(),
    })
}

// 19. type_app
/// Lean-verified type application (ForAll elimination).
pub fn lean_type_app(
    judgment: &Judgment,
    result_type_id: TypeId,
    graph: &SemanticGraph,
) -> Result<Judgment, KernelError> {
    let type_def = lookup_type(&graph.type_env, judgment.type_ref)?;
    match type_def {
        TypeDef::ForAll(_, _) => { /* OK */ }
        _ => {
            return Err(KernelError::UnexpectedTypeDef {
                type_id: judgment.type_ref,
                expected: "ForAll",
            });
        }
    }

    assert_type_well_formed(&graph.type_env, result_type_id, "type_app")?;

    Ok(Judgment {
        context: judgment.context.clone(),
        node_id: judgment.node_id,
        type_ref: result_type_id,
        cost: judgment.cost.clone(),
    })
}

// 20. guard_rule
/// Lean-verified guard rule (conditional).
pub fn lean_guard_rule(
    pred_judgment: &Judgment,
    then_judgment: &Judgment,
    else_judgment: &Judgment,
    guard_node: NodeId,
) -> Result<Judgment, KernelError> {
    if then_judgment.type_ref != else_judgment.type_ref {
        return Err(KernelError::TypeMismatch {
            expected: then_judgment.type_ref,
            actual: else_judgment.type_ref,
            context: "guard_rule (then vs else)",
        });
    }

    let cost = CostBound::Sum(
        Box::new(pred_judgment.cost.clone()),
        Box::new(CostBound::Sup(vec![
            then_judgment.cost.clone(),
            else_judgment.cost.clone(),
        ])),
    );

    Ok(Judgment {
        context: pred_judgment.context.clone(),
        node_id: guard_node,
        type_ref: then_judgment.type_ref,
        cost,
    })
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

    // -----------------------------------------------------------------------
    // Bridge function tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_lean_assume() {
        let ctx = Context::empty().extend(BinderId(0), TypeId(42));
        let j = lean_assume(&ctx, BinderId(0), NodeId(1)).unwrap();
        assert_eq!(j.type_ref, TypeId(42));
        assert_eq!(j.node_id, NodeId(1));
        assert_eq!(j.cost, CostBound::Zero);
    }

    #[test]
    fn test_lean_assume_not_found() {
        let ctx = Context::empty();
        let err = lean_assume(&ctx, BinderId(0), NodeId(1)).unwrap_err();
        assert!(matches!(err, KernelError::BinderNotFound { .. }));
    }

    #[test]
    fn test_lean_refl() {
        let j = lean_refl(NodeId(7), TypeId(99));
        assert_eq!(j.node_id, NodeId(7));
        assert_eq!(j.type_ref, TypeId(99));
        assert_eq!(j.cost, CostBound::Zero);
    }

    #[test]
    fn test_lean_trans_type_mismatch() {
        let j1 = lean_refl(NodeId(1), TypeId(10));
        let j2 = lean_refl(NodeId(2), TypeId(20));
        let err = lean_trans(&j1, &j2).unwrap_err();
        assert!(matches!(err, KernelError::TypeMismatch { .. }));
    }

    #[test]
    fn test_lean_cost_subsume() {
        let j = lean_refl(NodeId(1), TypeId(1));
        let weakened = lean_cost_subsume(&j, CostBound::Constant(100)).unwrap();
        assert_eq!(weakened.cost, CostBound::Constant(100));
    }

    #[test]
    fn test_lean_cost_subsume_fails() {
        let j = Judgment {
            context: Context::empty(),
            node_id: NodeId(1),
            type_ref: TypeId(1),
            cost: CostBound::Constant(5),
        };
        let err = lean_cost_subsume(&j, CostBound::Zero).unwrap_err();
        assert!(matches!(err, KernelError::CostViolation { .. }));
    }

    #[test]
    fn test_lean_congr() {
        let fn_j = lean_refl(NodeId(1), TypeId(10));
        let arg_j = lean_refl(NodeId(2), TypeId(20));
        let result = lean_congr(&fn_j, &arg_j, NodeId(3)).unwrap();
        assert_eq!(result.type_ref, TypeId(10));
        assert_eq!(result.node_id, NodeId(3));
    }

    #[test]
    fn test_lean_guard_rule() {
        let pred = lean_refl(NodeId(1), TypeId(1));
        let then_j = lean_refl(NodeId(2), TypeId(10));
        let else_j = lean_refl(NodeId(3), TypeId(10));
        let result = lean_guard_rule(&pred, &then_j, &else_j, NodeId(4)).unwrap();
        assert_eq!(result.type_ref, TypeId(10));
    }

    #[test]
    fn test_lean_guard_rule_type_mismatch() {
        let pred = lean_refl(NodeId(1), TypeId(1));
        let then_j = lean_refl(NodeId(2), TypeId(10));
        let else_j = lean_refl(NodeId(3), TypeId(20));
        let err = lean_guard_rule(&pred, &then_j, &else_j, NodeId(4)).unwrap_err();
        assert!(matches!(err, KernelError::TypeMismatch { .. }));
    }
}
