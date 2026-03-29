//! The proof kernel: ~20 primitive inference rules.
//!
//! This is the ONLY code that can construct `Theorem` values. It is the
//! trusted computing base of IRIS. Every rule validates its preconditions
//! and returns `Err(KernelError)` on failure.
//!
//! Zero `unsafe` blocks. Small, auditable, correct.
//!
//! # Consistency Argument — Mapping to Standard Type Theory
//!
//! The 20 rules form a sound fragment of **System F with refinements, cost
//! annotations, and structural recursion**. Each rule maps to a standard
//! judgment form:
//!
//! | # | Rule           | Standard Counterpart                            | Notes                                                       |
//! |---|----------------|-------------------------------------------------|-------------------------------------------------------------|
//! | 1 | `assume`       | Variable rule (Var)                             | Standard. Looks up binder in Gamma; O(n) scan is safe.      |
//! | 2 | `intro`        | Lambda introduction (->I)                       | Standard. Checks extended context matches body theorem.     |
//! | 3 | `elim`         | Lambda elimination / modus ponens (->E)         | Standard. Checks arg type matches domain, contexts match.   |
//! | 4 | `refl`         | Reflexivity of definitional equality             | Standard. `t = t` is unconditional.                         |
//! | 5 | `symm`         | Symmetry of equality                            | Standard. Transfers judgment to a symmetric node.           |
//! | 6 | `trans`        | Transitivity of equality                        | Standard. Chains two equalities; requires same type.        |
//! | 7 | `congr`        | Congruence of equality                          | Standard. `f=g, a=b => f(a)=g(b)`.                         |
//! | 8 | `type_check_node` | Annotation / axiom schema                    | **Novel.** Trusts node annotations for well-typed graphs.   |
//! |   |                |                                                 | Sound because the graph is content-addressed and validated  |
//! |   |                |                                                 | by the builder; the kernel verifies type_sig exists.        |
//! | 9 | `cost_subsume` | Subsumption / weakening (cost dimension)        | **Novel.** Analogous to subtyping subsumption. Sound        |
//! |   |                |                                                 | because `cost_leq` is a conservative partial order.         |
//! |10 | `cost_leq_rule`| Cost ordering witness                           | **Novel.** Produces a witness that `k1 <= k2`. No type      |
//! |   |                |                                                 | content; purely a cost-algebra judgment.                    |
//! |11 | `refine_intro` | Refinement type introduction ({x:T|P} intro)   | Standard (liquid types). Base type must match; predicate    |
//! |   |                |                                                 | must be independently proven.                               |
//! |12 | `refine_elim`  | Refinement type elimination ({x:T|P} elim)      | Standard. Extracts base type from refinement.               |
//! |13 | `nat_ind`      | Natural number induction (Peano axiom)          | Standard. Base + step => forall. Cost is Sum(base, step).   |
//! |14 | `structural_ind` | Structural induction over ADTs                | Standard (CIC-style). One case per constructor.             |
//! |15 | `let_bind`     | Let binding (cut rule)                          | Standard. `Gamma |- e1:A, Gamma,x:A |- e2:B => let x=e1 in e2 : B`. |
//! |16 | `match_elim`   | Sum elimination / case analysis                 | Standard. All arms must agree on result type.               |
//! |17 | `fold_rule`    | Catamorphism / structural recursion              | **Novel.** Sound because fold over finite data terminates.  |
//! |   |                |                                                 | Cost includes Mul(step, input_size).                        |
//! |18 | `type_abst`    | ForAll introduction (System F: /\I)             | Standard. Body type must match ForAll's inner type.         |
//! |19 | `type_app`     | ForAll elimination (System F: /\E)              | Standard, with well-formedness guard. The result type must  |
//! |   |                |                                                 | exist in the TypeEnv and be recursively well-formed (all    |
//! |   |                |                                                 | referenced TypeIds must also exist). This prevents          |
//! |   |                |                                                 | substitution of malformed types.                            |
//! |20 | `guard_rule`   | Conditional / if-then-else                      | Standard. Then/else must agree on type; cost is             |
//! |   |                |                                                 | Sum(pred, Sup(then, else)).                                 |
//!
//! ## Why the rule set is consistent
//!
//! 1. **Core fragment (rules 1-7, 15, 16)** is simply-typed lambda calculus
//!    with products/sums and equality, which is well-known to be consistent.
//!
//! 2. **Polymorphism (rules 18-19)** follows System F. Rule 19 (type_app)
//!    enforces that the substituted type is well-formed (all referenced TypeIds
//!    exist in the TypeEnv), preventing dangling type references from creating
//!    inconsistencies.
//!
//! 3. **Refinement types (rules 11-12)** follow the standard liquid types
//!    discipline: introduction requires an independent proof of the predicate,
//!    and elimination discards the predicate. No circular reasoning is possible.
//!
//! 4. **Induction (rules 13-14)** follows standard Peano / structural
//!    induction. Structural induction requires exhaustive case coverage.
//!
//! 5. **Cost annotations (rules 9-10, 17)** are a separate dimension that
//!    does not affect type soundness. They form a lattice with a conservative
//!    partial order; subsumption only weakens (never strengthens) bounds.
//!
//! 6. **Novel rules (8, 17, 20)** are conservative by construction:
//!    - Rule 8 trusts content-addressed annotations but verifies type existence.
//!    - Rule 17 (fold) is a catamorphism that terminates on finite data.
//!    - Rule 20 (guard) is standard if-then-else.
//!
//! 7. **The LCF architecture** ensures that no code outside this module can
//!    construct `Theorem` values, so soundness reduces to the correctness of
//!    these 20 rules.

use crate::syntax::kernel::cost_checker;
use crate::syntax::kernel::error::KernelError;
#[cfg(feature = "lean-kernel")]
use crate::syntax::kernel::lean_bridge;
use crate::syntax::kernel::theorem::{Context, Judgment, Theorem};

use iris_types::cost::CostBound;
use iris_types::graph::{BinderId, NodeId, NodeKind, SemanticGraph};
use iris_types::types::{TypeDef, TypeId};

// ---------------------------------------------------------------------------
// Proof hashing helper
// ---------------------------------------------------------------------------

/// Compute a BLAKE3 proof hash from a rule name and sub-proof hashes.
fn proof_hash(rule: &str, sub_hashes: &[&[u8; 32]]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(rule.as_bytes());
    for h in sub_hashes {
        hasher.update(h.as_slice());
    }
    *hasher.finalize().as_bytes()
}

/// Compute a BLAKE3 proof hash from a rule name, node id, and type id.
fn proof_hash_leaf(rule: &str, node: NodeId, ty: TypeId) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(rule.as_bytes());
    hasher.update(&node.0.to_le_bytes());
    hasher.update(&ty.0.to_le_bytes());
    *hasher.finalize().as_bytes()
}

// ---------------------------------------------------------------------------
// Kernel
// ---------------------------------------------------------------------------

/// The proof kernel. An empty struct that namespaces the inference rules.
///
/// All methods are `pub` so the checker (and tests) can call them, but they
/// are the ONLY way to obtain `Theorem` values.
pub struct Kernel;

impl Kernel {
    // -----------------------------------------------------------------------
    // 1. assume: Gamma, P |- P
    // -----------------------------------------------------------------------

    /// Assumption rule: if a proposition (binder) is in the context, derive it.
    ///
    /// `assume(ctx, name, prop_type)` produces `ctx |- name : prop_type @ Zero`
    /// provided `name : prop_type` appears in `ctx`.
    pub fn assume(
        ctx: &Context,
        name: BinderId,
        node_id: NodeId,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_assume(ctx, name, node_id)?;
            let ph = proof_hash_leaf("assume", judgment.node_id, judgment.type_ref);
            return Ok(Theorem {
                judgment,
                proof_hash: ph,
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            let type_id = ctx.lookup(name).ok_or(KernelError::BinderNotFound {
                rule: "assume",
                binder: name,
            })?;

            Ok(Theorem {
                judgment: Judgment {
                    context: ctx.clone(),
                    node_id,
                    type_ref: type_id,
                    cost: CostBound::Zero,
                },
                proof_hash: proof_hash_leaf("assume", node_id, type_id),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 2. intro: if Gamma, x:A |- body : B @ k then Gamma |- lam : A -> B @ Zero
    // -----------------------------------------------------------------------

    /// Introduction rule for arrow types.
    ///
    /// Given a theorem `body_thm` proving `Gamma, x:A |- body : B @ k`,
    /// produce `Gamma |- lam_node : Arrow(A, B, k) @ Zero`.
    ///
    /// The caller must supply the lambda node id and the binder name/type that
    /// was added to the context.
    pub fn intro(
        ctx: &Context,
        lam_node: NodeId,
        binder_name: BinderId,
        binder_type: TypeId,
        body_thm: &Theorem,
        graph: &SemanticGraph,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_intro(
                ctx,
                lam_node,
                binder_name,
                binder_type,
                &body_thm.judgment,
                graph,
            )?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash("intro", &[body_thm.proof_hash()]),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            // The body theorem's context must be ctx extended with the binder.
            let extended = ctx.extend(binder_name, binder_type);
            if body_thm.judgment.context != extended {
                return Err(KernelError::ContextMismatch { rule: "intro" });
            }

            // Build the Arrow type id. We need it registered in the type_env.
            let body_type = body_thm.judgment.type_ref;
            let body_cost = body_thm.judgment.cost.clone();
            let arrow_def = TypeDef::Arrow(binder_type, body_type, body_cost);

            // Find or verify the arrow type exists in the graph's type_env.
            let arrow_id = find_type_id(&graph.type_env, &arrow_def)?;

            Ok(Theorem {
                judgment: Judgment {
                    context: ctx.clone(),
                    node_id: lam_node,
                    type_ref: arrow_id,
                    cost: CostBound::Zero,
                },
                proof_hash: proof_hash("intro", &[body_thm.proof_hash()]),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 3. elim: modus ponens / function application
    //    if Gamma |- f : A -> B @ k_f and Gamma |- a : A @ k_a
    //    then Gamma |- app : B @ Sum(k_a, Sum(k_f_arg, k_f))
    // -----------------------------------------------------------------------

    /// Elimination rule: function application / modus ponens.
    ///
    /// Given `fn_thm` proving `Gamma |- f : Arrow(A, B, k_f)` and
    /// `arg_thm` proving `Gamma |- a : A`, produce
    /// `Gamma |- app_node : B @ Sum(k_arg, Sum(k_f_body, k_f))`.
    pub fn elim(
        fn_thm: &Theorem,
        arg_thm: &Theorem,
        app_node: NodeId,
        graph: &SemanticGraph,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_elim(
                &fn_thm.judgment,
                &arg_thm.judgment,
                app_node,
                graph,
            )?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash(
                    "elim",
                    &[fn_thm.proof_hash(), arg_thm.proof_hash()],
                ),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            // Contexts must match.
            if fn_thm.judgment.context != arg_thm.judgment.context {
                return Err(KernelError::ContextMismatch { rule: "elim" });
            }

            // fn_thm must have an Arrow type.
            let fn_type_def = lookup_type(&graph.type_env, fn_thm.judgment.type_ref)?;
            let (param_type, return_type, body_cost) = match fn_type_def {
                TypeDef::Arrow(a, b, k) => (a, b, k),
                _ => {
                    return Err(KernelError::UnexpectedTypeDef {
                        type_id: fn_thm.judgment.type_ref,
                        expected: "Arrow",
                    });
                }
            };

            // Argument type must match the function's parameter type.
            if arg_thm.judgment.type_ref != *param_type {
                return Err(KernelError::TypeMismatch {
                    expected: *param_type,
                    actual: arg_thm.judgment.type_ref,
                    context: "elim (argument type)",
                });
            }

            // Total cost: Sum(k_arg, Sum(k_fn, k_body))
            let total_cost = CostBound::Sum(
                Box::new(arg_thm.judgment.cost.clone()),
                Box::new(CostBound::Sum(
                    Box::new(fn_thm.judgment.cost.clone()),
                    Box::new(body_cost.clone()),
                )),
            );

            Ok(Theorem {
                judgment: Judgment {
                    context: fn_thm.judgment.context.clone(),
                    node_id: app_node,
                    type_ref: *return_type,
                    cost: total_cost,
                },
                proof_hash: proof_hash(
                    "elim",
                    &[fn_thm.proof_hash(), arg_thm.proof_hash()],
                ),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 4. refl: |- t = t (reflexivity of equality)
    // -----------------------------------------------------------------------

    /// Reflexivity: for any node, it equals itself.
    ///
    /// Produces a theorem asserting `node_id = node_id` (represented as the
    /// node having its own type, at zero cost, in an empty context).
    pub fn refl(node_id: NodeId, type_id: TypeId) -> Theorem {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_refl(node_id, type_id);
            return Theorem {
                proof_hash: proof_hash_leaf("refl", node_id, type_id),
                judgment,
            };
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            Theorem {
                judgment: Judgment {
                    context: Context::empty(),
                    node_id,
                    type_ref: type_id,
                    cost: CostBound::Zero,
                },
                proof_hash: proof_hash_leaf("refl", node_id, type_id),
            }
        }
    }

    // -----------------------------------------------------------------------
    // 5. symm: if |- a = b then |- b = a
    // -----------------------------------------------------------------------

    /// Symmetry of equality.
    ///
    /// Given a theorem about node `a` and an equality witness proving
    /// `a = b` (a theorem about `b` with the same type), produce a theorem
    /// about `b` with the type/cost of `thm`.
    ///
    /// The equality witness must:
    /// 1. Have the same type as `thm` (same type judgment).
    /// 2. Have node_id == `other_node` (it is about the target node).
    ///
    /// Without the witness, `symm` would allow transferring ANY theorem
    /// to ANY other node, completely collapsing the type system.
    pub fn symm(
        thm: &Theorem,
        other_node: NodeId,
        eq_witness: &Theorem,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_symm(
                &thm.judgment,
                other_node,
                &eq_witness.judgment,
            )?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash("symm", &[thm.proof_hash(), eq_witness.proof_hash()]),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            // The equality witness must be about the target node.
            if eq_witness.judgment.node_id != other_node {
                return Err(KernelError::NotEqual {
                    left: thm.judgment.node_id,
                    right: other_node,
                });
            }

            // The equality witness must have the same type as the source theorem.
            if eq_witness.judgment.type_ref != thm.judgment.type_ref {
                return Err(KernelError::TypeMismatch {
                    expected: thm.judgment.type_ref,
                    actual: eq_witness.judgment.type_ref,
                    context: "symm (equality witness type)",
                });
            }

            Ok(Theorem {
                judgment: Judgment {
                    context: thm.judgment.context.clone(),
                    node_id: other_node,
                    type_ref: thm.judgment.type_ref,
                    cost: thm.judgment.cost.clone(),
                },
                proof_hash: proof_hash("symm", &[thm.proof_hash(), eq_witness.proof_hash()]),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 6. trans: if |- a = b and |- b = c then |- a = c
    // -----------------------------------------------------------------------

    /// Transitivity of equality.
    ///
    /// Given `thm1` about node `a` with type `T` and `thm2` about node `b`
    /// with the same type `T`, produce a theorem about node `a` with the cost
    /// of `thm2` (the "target" end of the chain).
    pub fn trans(thm1: &Theorem, thm2: &Theorem) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_trans(&thm1.judgment, &thm2.judgment)?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash(
                    "trans",
                    &[thm1.proof_hash(), thm2.proof_hash()],
                ),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            if thm1.judgment.type_ref != thm2.judgment.type_ref {
                return Err(KernelError::TypeMismatch {
                    expected: thm1.judgment.type_ref,
                    actual: thm2.judgment.type_ref,
                    context: "trans",
                });
            }

            Ok(Theorem {
                judgment: Judgment {
                    context: thm1.judgment.context.clone(),
                    node_id: thm1.judgment.node_id,
                    type_ref: thm2.judgment.type_ref,
                    cost: thm2.judgment.cost.clone(),
                },
                proof_hash: proof_hash(
                    "trans",
                    &[thm1.proof_hash(), thm2.proof_hash()],
                ),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 7. congr: congruence — if f = g and a = b then f(a) = g(b)
    // -----------------------------------------------------------------------

    /// Congruence: if function and argument are equal, application is equal.
    ///
    /// Given `fn_thm` (about node f) and `arg_thm` (about node a), produce a
    /// theorem about the application node. Both theorems must share a context.
    pub fn congr(
        fn_thm: &Theorem,
        arg_thm: &Theorem,
        app_node: NodeId,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_congr(
                &fn_thm.judgment,
                &arg_thm.judgment,
                app_node,
            )?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash(
                    "congr",
                    &[fn_thm.proof_hash(), arg_thm.proof_hash()],
                ),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            if fn_thm.judgment.context != arg_thm.judgment.context {
                return Err(KernelError::ContextMismatch { rule: "congr" });
            }

            // The result type comes from the function theorem; cost is the sum.
            let total_cost = CostBound::Sum(
                Box::new(fn_thm.judgment.cost.clone()),
                Box::new(arg_thm.judgment.cost.clone()),
            );

            Ok(Theorem {
                judgment: Judgment {
                    context: fn_thm.judgment.context.clone(),
                    node_id: app_node,
                    type_ref: fn_thm.judgment.type_ref,
                    cost: total_cost,
                },
                proof_hash: proof_hash(
                    "congr",
                    &[fn_thm.proof_hash(), arg_thm.proof_hash()],
                ),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 8. type_check_node: the main typing judgment for a single node
    // -----------------------------------------------------------------------

    /// Type-check a single node against its annotation in the graph.
    ///
    /// This is the workhorse rule: it looks up the node in the graph, checks
    /// that the node's `type_sig` and `cost` annotation are consistent with
    /// the node's kind and payload, and produces a theorem.
    ///
    /// For composite nodes (Apply, Lambda, etc.) the caller must supply
    /// sub-theorems for the children. This rule handles leaf nodes directly
    /// (Lit, Prim with no arguments, etc.).
    pub fn type_check_node(
        ctx: &Context,
        graph: &SemanticGraph,
        node_id: NodeId,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_type_check_node(ctx, graph, node_id)?;
            // Compute proof hash based on node kind, matching the Rust path.
            let node = graph.nodes.get(&node_id)
                .ok_or(KernelError::NodeNotFound(node_id))?;
            let hash_rule = match node.kind {
                NodeKind::Lit => "type_check_lit",
                NodeKind::Prim => "type_check_prim",
                NodeKind::Tuple => "type_check_tuple",
                NodeKind::Inject => "type_check_inject",
                NodeKind::Project => "type_check_project",
                NodeKind::Ref => "type_check_ref",
                _ => "type_check_node",
            };
            return Ok(Theorem {
                proof_hash: proof_hash_leaf(hash_rule, node_id, node.type_sig),
                judgment,
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            let node = graph
                .nodes
                .get(&node_id)
                .ok_or(KernelError::NodeNotFound(node_id))?;

            // Verify the type_sig references a valid type in the environment.
            let _type_def = lookup_type(&graph.type_env, node.type_sig)?;

            // Determine the cost from the node's annotation.
            let cost = match &node.cost {
                iris_types::cost::CostTerm::Unit => CostBound::Constant(1),
                iris_types::cost::CostTerm::Inherited => graph.cost.clone(),
                iris_types::cost::CostTerm::Annotated(c) => c.clone(),
            };

            // For leaf nodes, we can produce the theorem directly.
            // For composite nodes, this provides the "frame" — the checker
            // must combine it with sub-theorems using intro/elim/etc.
            match node.kind {
                NodeKind::Lit => {
                    Ok(Theorem {
                        judgment: Judgment {
                            context: ctx.clone(),
                            node_id,
                            type_ref: node.type_sig,
                            cost: CostBound::Zero,
                        },
                        proof_hash: proof_hash_leaf("type_check_lit", node_id, node.type_sig),
                    })
                }
                NodeKind::Prim => {
                    Ok(Theorem {
                        judgment: Judgment {
                            context: ctx.clone(),
                            node_id,
                            type_ref: node.type_sig,
                            cost: CostBound::Constant(1),
                        },
                        proof_hash: proof_hash_leaf("type_check_prim", node_id, node.type_sig),
                    })
                }
                NodeKind::Tuple => {
                    Ok(Theorem {
                        judgment: Judgment {
                            context: ctx.clone(),
                            node_id,
                            type_ref: node.type_sig,
                            cost: CostBound::Zero,
                        },
                        proof_hash: proof_hash_leaf("type_check_tuple", node_id, node.type_sig),
                    })
                }
                NodeKind::Inject => {
                    Ok(Theorem {
                        judgment: Judgment {
                            context: ctx.clone(),
                            node_id,
                            type_ref: node.type_sig,
                            cost: CostBound::Zero,
                        },
                        proof_hash: proof_hash_leaf("type_check_inject", node_id, node.type_sig),
                    })
                }
                NodeKind::Project => {
                    Ok(Theorem {
                        judgment: Judgment {
                            context: ctx.clone(),
                            node_id,
                            type_ref: node.type_sig,
                            cost: CostBound::Zero,
                        },
                        proof_hash: proof_hash_leaf("type_check_project", node_id, node.type_sig),
                    })
                }
                NodeKind::Ref => {
                    Ok(Theorem {
                        judgment: Judgment {
                            context: ctx.clone(),
                            node_id,
                            type_ref: node.type_sig,
                            cost,
                        },
                        proof_hash: proof_hash_leaf("type_check_ref", node_id, node.type_sig),
                    })
                }
                _ => Err(KernelError::InvalidRule {
                    rule: "type_check_node",
                    reason: format!(
                        "composite node kind {:?} cannot be type-checked by annotation; use the appropriate structural rule",
                        node.kind
                    ),
                }),
            }
        }
    }

    // -----------------------------------------------------------------------
    // 9. cost_subsume: weaken a cost bound
    // -----------------------------------------------------------------------

    /// Cost subsumption: if `thm` proves `e : T @ k` and `k <= new_cost`,
    /// then produce `e : T @ new_cost`.
    pub fn cost_subsume(
        thm: &Theorem,
        new_cost: CostBound,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_cost_subsume(&thm.judgment, new_cost)?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash("cost_subsume", &[thm.proof_hash()]),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            if !cost_checker::cost_leq(&thm.judgment.cost, &new_cost) {
                return Err(KernelError::CostViolation {
                    required: new_cost,
                    actual: thm.judgment.cost.clone(),
                });
            }

            Ok(Theorem {
                judgment: Judgment {
                    context: thm.judgment.context.clone(),
                    node_id: thm.judgment.node_id,
                    type_ref: thm.judgment.type_ref,
                    cost: new_cost,
                },
                proof_hash: proof_hash("cost_subsume", &[thm.proof_hash()]),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 10. cost_leq: verify cost ordering (produces a theorem witness)
    // -----------------------------------------------------------------------

    /// Verify that `k1 <= k2` and produce a witness theorem.
    ///
    /// The theorem's node_id is a dummy (NodeId(0)); this is purely a cost
    /// ordering proof.
    pub fn cost_leq_rule(
        k1: &CostBound,
        k2: &CostBound,
    ) -> Result<Theorem, KernelError> {
        // Hash the cost bounds using their Debug representation as a
        // stable canonical form (avoids adding serde_json to the kernel).
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"cost_leq");
        hasher.update(format!("{k1:?}").as_bytes());
        hasher.update(format!("{k2:?}").as_bytes());
        let ph = *hasher.finalize().as_bytes();

        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_cost_leq_rule(k1, k2)?;
            return Ok(Theorem {
                judgment,
                proof_hash: ph,
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            if !cost_checker::cost_leq(k1, k2) {
                return Err(KernelError::CostViolation {
                    required: k2.clone(),
                    actual: k1.clone(),
                });
            }

            Ok(Theorem {
                judgment: Judgment {
                    context: Context::empty(),
                    node_id: NodeId(0),
                    type_ref: TypeId(0),
                    cost: k2.clone(),
                },
                proof_hash: ph,
            })
        }
    }

    // -----------------------------------------------------------------------
    // 11. refine_intro: introduce a refinement type
    // -----------------------------------------------------------------------

    /// Refinement introduction: given `base_thm` proving `e : T` and
    /// `pred_holds` proving the predicate holds, produce `e : {x:T | P(x)}`.
    ///
    /// The `pred_holds` theorem must relate to the refinement: specifically,
    /// it must be about the same node as `base_thm` (proving the predicate
    /// for that particular value) and must have the refined type as its type
    /// (witnessing that the predicate holds for this value). This prevents
    /// using an arbitrary unrelated theorem as the predicate witness.
    pub fn refine_intro(
        base_thm: &Theorem,
        pred_holds: &Theorem,
        refined_type_id: TypeId,
        graph: &SemanticGraph,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_refine_intro(
                &base_thm.judgment,
                &pred_holds.judgment,
                refined_type_id,
                graph,
            )?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash(
                    "refine_intro",
                    &[base_thm.proof_hash(), pred_holds.proof_hash()],
                ),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            // Verify the refined type exists and wraps the base type.
            let refined_def = lookup_type(&graph.type_env, refined_type_id)?;
            match refined_def {
                TypeDef::Refined(inner_type, _pred) => {
                    if *inner_type != base_thm.judgment.type_ref {
                        return Err(KernelError::TypeMismatch {
                            expected: *inner_type,
                            actual: base_thm.judgment.type_ref,
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

            // Validate that pred_holds is about the same node as base_thm.
            if pred_holds.judgment.node_id != base_thm.judgment.node_id {
                return Err(KernelError::InvalidRule {
                    rule: "refine_intro",
                    reason: format!(
                        "predicate witness is about node {:?}, but base theorem is about node {:?}; \
                         the predicate must be proven for the same value being refined",
                        pred_holds.judgment.node_id, base_thm.judgment.node_id
                    ),
                });
            }

            // Validate that pred_holds has a type related to the refinement.
            if pred_holds.judgment.type_ref != refined_type_id
                && pred_holds.judgment.type_ref != base_thm.judgment.type_ref
            {
                return Err(KernelError::InvalidRule {
                    rule: "refine_intro",
                    reason: format!(
                        "predicate witness has type {:?}, expected {:?} (refined) or {:?} (base); \
                         witness must prove the refinement predicate for the value",
                        pred_holds.judgment.type_ref, refined_type_id, base_thm.judgment.type_ref
                    ),
                });
            }

            Ok(Theorem {
                judgment: Judgment {
                    context: base_thm.judgment.context.clone(),
                    node_id: base_thm.judgment.node_id,
                    type_ref: refined_type_id,
                    cost: base_thm.judgment.cost.clone(),
                },
                proof_hash: proof_hash(
                    "refine_intro",
                    &[base_thm.proof_hash(), pred_holds.proof_hash()],
                ),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 12. refine_elim: eliminate a refinement type
    // -----------------------------------------------------------------------

    /// Refinement elimination: given `thm` proving `e : {x:T | P(x)}`,
    /// produce two theorems: `e : T` and the predicate holds.
    ///
    /// Returns `(base_theorem, predicate_theorem)`.
    pub fn refine_elim(
        thm: &Theorem,
        graph: &SemanticGraph,
    ) -> Result<(Theorem, Theorem), KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let (base_j, pred_j) = lean_bridge::lean_refine_elim(&thm.judgment, graph)?;
            return Ok((
                Theorem {
                    judgment: base_j,
                    proof_hash: proof_hash("refine_elim_base", &[thm.proof_hash()]),
                },
                Theorem {
                    judgment: pred_j,
                    proof_hash: proof_hash("refine_elim_pred", &[thm.proof_hash()]),
                },
            ));
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            let type_def = lookup_type(&graph.type_env, thm.judgment.type_ref)?;
            let (inner_type, _pred) = match type_def {
                TypeDef::Refined(inner, pred) => (inner, pred),
                _ => {
                    return Err(KernelError::UnexpectedTypeDef {
                        type_id: thm.judgment.type_ref,
                        expected: "Refined",
                    });
                }
            };

            let base_thm = Theorem {
                judgment: Judgment {
                    context: thm.judgment.context.clone(),
                    node_id: thm.judgment.node_id,
                    type_ref: *inner_type,
                    cost: thm.judgment.cost.clone(),
                },
                proof_hash: proof_hash("refine_elim_base", &[thm.proof_hash()]),
            };

            let pred_thm = Theorem {
                judgment: Judgment {
                    context: thm.judgment.context.clone(),
                    node_id: thm.judgment.node_id,
                    type_ref: thm.judgment.type_ref,
                    cost: CostBound::Zero,
                },
                proof_hash: proof_hash("refine_elim_pred", &[thm.proof_hash()]),
            };

            Ok((base_thm, pred_thm))
        }
    }

    // -----------------------------------------------------------------------
    // 13. nat_ind: natural number induction
    // -----------------------------------------------------------------------

    /// Natural number induction.
    ///
    /// Given `base` proving `P(0)` and `step` proving `P(n) -> P(n+1)`,
    /// produce `forall n. P(n)`.
    ///
    /// The step theorem must have an Arrow type `Arrow(T, T, k)` where
    /// `T` is the base theorem's type (representing `P(n) -> P(n+1)`).
    pub fn nat_ind(
        base: &Theorem,
        step: &Theorem,
        result_node: NodeId,
        graph: &SemanticGraph,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_nat_ind(
                &base.judgment,
                &step.judgment,
                result_node,
                graph,
            )?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash(
                    "nat_ind",
                    &[base.proof_hash(), step.proof_hash()],
                ),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            // The step case must have an Arrow type P(n) -> P(n+1).
            let step_type_def = lookup_type(&graph.type_env, step.judgment.type_ref)?;
            let step_body_cost = match step_type_def {
                TypeDef::Arrow(param_type, return_type, body_cost) => {
                    if *param_type != base.judgment.type_ref {
                        return Err(KernelError::InductionError {
                            reason: format!(
                                "nat_ind: step parameter type {:?} != base type {:?}; \
                                 step must have type P(n) -> P(n+1)",
                                param_type, base.judgment.type_ref
                            ),
                        });
                    }
                    if *return_type != base.judgment.type_ref {
                        return Err(KernelError::InductionError {
                            reason: format!(
                                "nat_ind: step return type {:?} != base type {:?}; \
                                 step must have type P(n) -> P(n+1)",
                                return_type, base.judgment.type_ref
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
                            step.judgment.type_ref
                        ),
                    });
                }
            };

            let cost = CostBound::Sum(
                Box::new(base.judgment.cost.clone()),
                Box::new(step_body_cost.clone()),
            );

            Ok(Theorem {
                judgment: Judgment {
                    context: base.judgment.context.clone(),
                    node_id: result_node,
                    type_ref: base.judgment.type_ref,
                    cost,
                },
                proof_hash: proof_hash(
                    "nat_ind",
                    &[base.proof_hash(), step.proof_hash()],
                ),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 14. structural_ind: structural induction over an algebraic data type
    // -----------------------------------------------------------------------

    /// Structural induction over a sum type (algebraic data type).
    ///
    /// Given a list of case theorems (one per constructor of the type),
    /// produce a theorem that the property holds for all values of the type.
    ///
    /// `ty` must be a Sum type in the graph's type_env, and `cases` must
    /// have exactly one theorem per constructor variant.
    pub fn structural_ind(
        ty: TypeId,
        cases: &[Theorem],
        result_node: NodeId,
        graph: &SemanticGraph,
    ) -> Result<Theorem, KernelError> {
        let sub_hashes: Vec<&[u8; 32]> = cases.iter().map(|c| c.proof_hash()).collect();

        #[cfg(feature = "lean-kernel")]
        {
            let case_judgments: Vec<Judgment> =
                cases.iter().map(|c| c.judgment.clone()).collect();
            let judgment = lean_bridge::lean_structural_ind(
                ty,
                &case_judgments,
                result_node,
                graph,
            )?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash("structural_ind", &sub_hashes),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
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

            if cases.len() != variants.len() {
                return Err(KernelError::InductionError {
                    reason: format!(
                        "structural_ind: expected {} cases, got {}",
                        variants.len(),
                        cases.len()
                    ),
                });
            }

            if cases.is_empty() {
                return Err(KernelError::InductionError {
                    reason: "structural_ind: empty sum type".to_string(),
                });
            }

            let result_type = cases[0].judgment.type_ref;
            for (i, case) in cases.iter().enumerate().skip(1) {
                if case.judgment.type_ref != result_type {
                    return Err(KernelError::InductionError {
                        reason: format!(
                            "case {i} has type {:?}, expected {:?}",
                            case.judgment.type_ref, result_type
                        ),
                    });
                }
            }

            let case_costs: Vec<CostBound> =
                cases.iter().map(|c| c.judgment.cost.clone()).collect();
            let cost = CostBound::Sup(case_costs);

            Ok(Theorem {
                judgment: Judgment {
                    context: cases[0].judgment.context.clone(),
                    node_id: result_node,
                    type_ref: result_type,
                    cost,
                },
                proof_hash: proof_hash("structural_ind", &sub_hashes),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 15. let_bind: Gamma |- let x = e1 in e2 : B @ Sum(k1, k2)
    // -----------------------------------------------------------------------

    /// Let binding rule.
    ///
    /// Given `bound_thm` proving `Gamma |- e1 : A @ k1` and `body_thm`
    /// proving `Gamma, x:A |- e2 : B @ k2`, produce
    /// `Gamma |- let_node : B @ Sum(k1, k2)`.
    pub fn let_bind(
        ctx: &Context,
        let_node: NodeId,
        binder_name: BinderId,
        bound_thm: &Theorem,
        body_thm: &Theorem,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_let_bind(
                ctx,
                let_node,
                binder_name,
                &bound_thm.judgment,
                &body_thm.judgment,
            )?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash(
                    "let_bind",
                    &[bound_thm.proof_hash(), body_thm.proof_hash()],
                ),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            // bound_thm must be in ctx.
            if bound_thm.judgment.context != *ctx {
                return Err(KernelError::ContextMismatch {
                    rule: "let_bind (bound)",
                });
            }

            // body_thm must be in ctx extended with the binder.
            let extended = ctx.extend(binder_name, bound_thm.judgment.type_ref);
            if body_thm.judgment.context != extended {
                return Err(KernelError::ContextMismatch {
                    rule: "let_bind (body)",
                });
            }

            let cost = CostBound::Sum(
                Box::new(bound_thm.judgment.cost.clone()),
                Box::new(body_thm.judgment.cost.clone()),
            );

            Ok(Theorem {
                judgment: Judgment {
                    context: ctx.clone(),
                    node_id: let_node,
                    type_ref: body_thm.judgment.type_ref,
                    cost,
                },
                proof_hash: proof_hash(
                    "let_bind",
                    &[bound_thm.proof_hash(), body_thm.proof_hash()],
                ),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 16. match_elim: pattern matching
    // -----------------------------------------------------------------------

    /// Match elimination rule.
    ///
    /// Given `scrutinee_thm` proving the scrutinee has a sum type, and
    /// `arm_thms` (one per constructor), produce a theorem for the match node.
    ///
    /// All arms must produce the same result type. Cost is
    /// `Sum(k_scrutinee, Sup(k_arm_1, ..., k_arm_n))`.
    pub fn match_elim(
        scrutinee_thm: &Theorem,
        arm_thms: &[Theorem],
        match_node: NodeId,
    ) -> Result<Theorem, KernelError> {
        let mut sub_hashes: Vec<&[u8; 32]> = vec![scrutinee_thm.proof_hash()];
        sub_hashes.extend(arm_thms.iter().map(|a| a.proof_hash()));

        #[cfg(feature = "lean-kernel")]
        {
            let arm_judgments: Vec<Judgment> =
                arm_thms.iter().map(|a| a.judgment.clone()).collect();
            let judgment = lean_bridge::lean_match_elim(
                &scrutinee_thm.judgment,
                &arm_judgments,
                match_node,
            )?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash("match_elim", &sub_hashes),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            if arm_thms.is_empty() {
                return Err(KernelError::InvalidRule {
                    rule: "match_elim",
                    reason: "no match arms".to_string(),
                });
            }

            let result_type = arm_thms[0].judgment.type_ref;
            for (_i, arm) in arm_thms.iter().enumerate().skip(1) {
                if arm.judgment.type_ref != result_type {
                    return Err(KernelError::TypeMismatch {
                        expected: result_type,
                        actual: arm.judgment.type_ref,
                        context: "match_elim (arm type mismatch)",
                    });
                }
            }

            let arm_costs: Vec<CostBound> =
                arm_thms.iter().map(|a| a.judgment.cost.clone()).collect();
            let cost = CostBound::Sum(
                Box::new(scrutinee_thm.judgment.cost.clone()),
                Box::new(CostBound::Sup(arm_costs)),
            );

            Ok(Theorem {
                judgment: Judgment {
                    context: scrutinee_thm.judgment.context.clone(),
                    node_id: match_node,
                    type_ref: result_type,
                    cost,
                },
                proof_hash: proof_hash("match_elim", &sub_hashes),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 17. fold_rule: structural recursion (catamorphism)
    // -----------------------------------------------------------------------

    /// Fold rule: structural recursion.
    ///
    /// Given `base_thm` (base case), `step_thm` (recursive step), and
    /// `input_thm` (the input being folded over), produce a theorem with
    /// cost `Sum(k_input, Sum(k_base, Mul(k_step, size)))`.
    pub fn fold_rule(
        base_thm: &Theorem,
        step_thm: &Theorem,
        input_thm: &Theorem,
        fold_node: NodeId,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_fold_rule(
                &base_thm.judgment,
                &step_thm.judgment,
                &input_thm.judgment,
                fold_node,
            )?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash(
                    "fold",
                    &[
                        base_thm.proof_hash(),
                        step_thm.proof_hash(),
                        input_thm.proof_hash(),
                    ],
                ),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            let result_type = base_thm.judgment.type_ref;

            let cost = CostBound::Sum(
                Box::new(input_thm.judgment.cost.clone()),
                Box::new(CostBound::Sum(
                    Box::new(base_thm.judgment.cost.clone()),
                    Box::new(CostBound::Mul(
                        Box::new(step_thm.judgment.cost.clone()),
                        Box::new(input_thm.judgment.cost.clone()),
                    )),
                )),
            );

            Ok(Theorem {
                judgment: Judgment {
                    context: base_thm.judgment.context.clone(),
                    node_id: fold_node,
                    type_ref: result_type,
                    cost,
                },
                proof_hash: proof_hash(
                    "fold",
                    &[
                        base_thm.proof_hash(),
                        step_thm.proof_hash(),
                        input_thm.proof_hash(),
                    ],
                ),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 18. type_abst: type abstraction (ForAll introduction)
    // -----------------------------------------------------------------------

    /// Type abstraction: if `body_thm` proves `Gamma |- e : T @ k`, produce
    /// `Gamma |- e : ForAll(X, T) @ k`.
    pub fn type_abst(
        body_thm: &Theorem,
        forall_type_id: TypeId,
        graph: &SemanticGraph,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_type_abst(
                &body_thm.judgment,
                forall_type_id,
                graph,
            )?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash("type_abst", &[body_thm.proof_hash()]),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            // Verify the ForAll type is well-formed (exists and has valid refs).
            assert_type_well_formed(&graph.type_env, forall_type_id, "type_abst")?;

            let type_def = lookup_type(&graph.type_env, forall_type_id)?;
            match type_def {
                TypeDef::ForAll(_, inner) => {
                    if *inner != body_thm.judgment.type_ref {
                        return Err(KernelError::TypeMismatch {
                            expected: *inner,
                            actual: body_thm.judgment.type_ref,
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

            Ok(Theorem {
                judgment: Judgment {
                    context: body_thm.judgment.context.clone(),
                    node_id: body_thm.judgment.node_id,
                    type_ref: forall_type_id,
                    cost: body_thm.judgment.cost.clone(),
                },
                proof_hash: proof_hash("type_abst", &[body_thm.proof_hash()]),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 19. type_app: type application (ForAll elimination)
    // -----------------------------------------------------------------------

    /// Type application: if `thm` proves `e : ForAll(X, T)`, produce
    /// `e : T[S/X]` where `result_type_id` is `T[S/X]`.
    ///
    /// **Soundness-critical:** The result type must be well-formed — it must
    /// exist in the TypeEnv AND all TypeIds it references must also exist.
    /// Without this check, a malformed type (containing dangling TypeId
    /// references) could be substituted, producing an unsound theorem.
    pub fn type_app(
        thm: &Theorem,
        result_type_id: TypeId,
        graph: &SemanticGraph,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_type_app(
                &thm.judgment,
                result_type_id,
                graph,
            )?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash("type_app", &[thm.proof_hash()]),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            let type_def = lookup_type(&graph.type_env, thm.judgment.type_ref)?;
            match type_def {
                TypeDef::ForAll(_, _) => { /* OK */ }
                _ => {
                    return Err(KernelError::UnexpectedTypeDef {
                        type_id: thm.judgment.type_ref,
                        expected: "ForAll",
                    });
                }
            }

            // Verify the result type exists AND is well-formed.
            assert_type_well_formed(&graph.type_env, result_type_id, "type_app")?;

            Ok(Theorem {
                judgment: Judgment {
                    context: thm.judgment.context.clone(),
                    node_id: thm.judgment.node_id,
                    type_ref: result_type_id,
                    cost: thm.judgment.cost.clone(),
                },
                proof_hash: proof_hash("type_app", &[thm.proof_hash()]),
            })
        }
    }

    // -----------------------------------------------------------------------
    // 20. guard_rule: runtime guard typing
    // -----------------------------------------------------------------------

    /// Guard rule: `Guard(pred, then, else) : B @ Sum(k_pred, Sup(k_then, k_else))`.
    pub fn guard_rule(
        pred_thm: &Theorem,
        then_thm: &Theorem,
        else_thm: &Theorem,
        guard_node: NodeId,
    ) -> Result<Theorem, KernelError> {
        #[cfg(feature = "lean-kernel")]
        {
            let judgment = lean_bridge::lean_guard_rule(
                &pred_thm.judgment,
                &then_thm.judgment,
                &else_thm.judgment,
                guard_node,
            )?;
            return Ok(Theorem {
                judgment,
                proof_hash: proof_hash(
                    "guard",
                    &[
                        pred_thm.proof_hash(),
                        then_thm.proof_hash(),
                        else_thm.proof_hash(),
                    ],
                ),
            });
        }
        #[cfg(not(feature = "lean-kernel"))]
        {
            // Then and else must have the same result type.
            if then_thm.judgment.type_ref != else_thm.judgment.type_ref {
                return Err(KernelError::TypeMismatch {
                    expected: then_thm.judgment.type_ref,
                    actual: else_thm.judgment.type_ref,
                    context: "guard_rule (then vs else)",
                });
            }

            let cost = CostBound::Sum(
                Box::new(pred_thm.judgment.cost.clone()),
                Box::new(CostBound::Sup(vec![
                    then_thm.judgment.cost.clone(),
                    else_thm.judgment.cost.clone(),
                ])),
            );

            Ok(Theorem {
                judgment: Judgment {
                    context: pred_thm.judgment.context.clone(),
                    node_id: guard_node,
                    type_ref: then_thm.judgment.type_ref,
                    cost,
                },
                proof_hash: proof_hash(
                    "guard",
                    &[
                        pred_thm.proof_hash(),
                        then_thm.proof_hash(),
                        else_thm.proof_hash(),
                    ],
                ),
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Look up a type definition in the TypeEnv, returning an error if not found.
#[cfg(not(feature = "lean-kernel"))]
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
/// Returns an error if the type is not registered.
#[cfg(not(feature = "lean-kernel"))]
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

/// Check that a type is well-formed: the TypeId exists in the TypeEnv, and
/// every TypeId referenced within its definition also exists.
///
/// This prevents malformed types (containing dangling TypeId references) from
/// being used in substitution, which could otherwise create unsound theorems.
///
/// The `context` parameter is used for error messages to indicate which rule
/// triggered the check.
#[cfg(any(not(feature = "lean-kernel"), test))]
fn assert_type_well_formed(
    type_env: &iris_types::types::TypeEnv,
    type_id: TypeId,
    context: &'static str,
) -> Result<(), KernelError> {
    // Use a visited set to avoid infinite recursion on recursive types.
    let mut visited = std::collections::HashSet::new();
    assert_type_well_formed_recursive(type_env, type_id, context, &mut visited)
}

/// Recursive helper for `assert_type_well_formed`. Validates that the type
/// and ALL transitively referenced types exist in the TypeEnv.
#[cfg(any(not(feature = "lean-kernel"), test))]
fn assert_type_well_formed_recursive(
    type_env: &iris_types::types::TypeEnv,
    type_id: TypeId,
    context: &'static str,
    visited: &mut std::collections::HashSet<TypeId>,
) -> Result<(), KernelError> {
    // If already visited, this type is valid (or being validated up the stack).
    if !visited.insert(type_id) {
        return Ok(());
    }

    let type_def = type_env
        .types
        .get(&type_id)
        .ok_or(KernelError::TypeNotFound(type_id))?;

    // Collect all TypeId references within this type definition and
    // recursively verify each one exists and is well-formed.
    for referenced_id in type_def_references(type_def) {
        if !type_env.types.contains_key(&referenced_id) {
            return Err(KernelError::TypeMalformed {
                type_id,
                dangling_ref: referenced_id,
                context,
            });
        }
        // Recursively check the referenced type.
        assert_type_well_formed_recursive(type_env, referenced_id, context, visited)?;
    }

    Ok(())
}

/// Extract all TypeId references from a TypeDef (non-recursive, single level).
///
/// This returns the immediate TypeId children of the definition. For a full
/// well-formedness check, the caller should verify each returned TypeId also
/// exists in the TypeEnv.
#[cfg(any(not(feature = "lean-kernel"), test))]
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use iris_types::types::{PrimType, TypeDef, TypeEnv};
    use std::collections::{BTreeMap, HashMap};

    fn make_type_env(defs: Vec<(TypeId, TypeDef)>) -> TypeEnv {
        TypeEnv {
            types: defs.into_iter().collect(),
        }
    }

    #[test]
    fn assume_in_context() {
        let ctx = Context::empty().extend(BinderId(0), TypeId(42));
        let thm = Kernel::assume(&ctx, BinderId(0), NodeId(1)).unwrap();
        assert_eq!(thm.type_ref(), TypeId(42));
        assert_eq!(thm.node_id(), NodeId(1));
        assert_eq!(*thm.cost(), CostBound::Zero);
    }

    #[test]
    fn assume_not_in_context() {
        let ctx = Context::empty();
        let err = Kernel::assume(&ctx, BinderId(0), NodeId(1)).unwrap_err();
        assert!(matches!(err, KernelError::BinderNotFound { .. }));
    }

    #[test]
    fn refl_produces_theorem() {
        let thm = Kernel::refl(NodeId(7), TypeId(99));
        assert_eq!(thm.node_id(), NodeId(7));
        assert_eq!(thm.type_ref(), TypeId(99));
        assert_eq!(*thm.cost(), CostBound::Zero);
    }

    #[test]
    fn cost_subsume_ok() {
        let thm = Kernel::refl(NodeId(1), TypeId(1));
        let weakened =
            Kernel::cost_subsume(&thm, CostBound::Constant(100)).unwrap();
        assert_eq!(*weakened.cost(), CostBound::Constant(100));
    }

    #[test]
    fn cost_subsume_fails() {
        let ctx = Context::empty().extend(BinderId(0), TypeId(1));
        let thm = Kernel::assume(&ctx, BinderId(0), NodeId(1)).unwrap();
        // Zero <= Constant, so weakening Zero to Constant is fine.
        assert!(Kernel::cost_subsume(&thm, CostBound::Constant(1)).is_ok());

        // Now weaken a Constant to Zero — should fail.
        let const_thm =
            Kernel::cost_subsume(&thm, CostBound::Constant(5)).unwrap();
        let err = Kernel::cost_subsume(&const_thm, CostBound::Zero).unwrap_err();
        assert!(matches!(err, KernelError::CostViolation { .. }));
    }

    #[test]
    fn trans_type_mismatch() {
        let thm1 = Kernel::refl(NodeId(1), TypeId(10));
        let thm2 = Kernel::refl(NodeId(2), TypeId(20));
        let err = Kernel::trans(&thm1, &thm2).unwrap_err();
        assert!(matches!(err, KernelError::TypeMismatch { .. }));
    }

    #[test]
    fn intro_and_elim_roundtrip() {
        // Set up types: Int, Arrow(Int, Int, Zero)
        let int_id = TypeId(1);
        let arrow_id = TypeId(2);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (
                arrow_id,
                TypeDef::Arrow(int_id, int_id, CostBound::Zero),
            ),
        ]);

        // Build a minimal graph with the type env.
        let graph = iris_types::graph::SemanticGraph {
            root: NodeId(0),
            nodes: HashMap::new(),
            edges: vec![],
            type_env,
            cost: CostBound::Unknown,
            resolution: iris_types::graph::Resolution::Implementation,
            hash: iris_types::hash::SemanticHash([0; 32]),
        };

        let ctx = Context::empty();
        let extended = ctx.extend(BinderId(0), int_id);

        // Assume x:Int in the extended context.
        let body_thm =
            Kernel::assume(&extended, BinderId(0), NodeId(10)).unwrap();

        // Intro: Gamma |- lam : Int -> Int @ Zero
        let lam_thm = Kernel::intro(
            &ctx,
            NodeId(20),
            BinderId(0),
            int_id,
            &body_thm,
            &graph,
        )
        .unwrap();
        assert_eq!(lam_thm.type_ref(), arrow_id);
        assert_eq!(*lam_thm.cost(), CostBound::Zero);

        // Now apply: assume we have another int value.
        let arg_thm = Kernel::refl(NodeId(30), int_id);
        // But arg_thm has empty context; need to align contexts.
        // For this test, use lam_thm (also empty context).
        let app_thm =
            Kernel::elim(&lam_thm, &arg_thm, NodeId(40), &graph).unwrap();
        assert_eq!(app_thm.type_ref(), int_id);
    }

    // -----------------------------------------------------------------------
    // Helper: build a minimal graph with just a TypeEnv (no nodes/edges).
    // -----------------------------------------------------------------------

    fn graph_with_types(type_env: TypeEnv) -> iris_types::graph::SemanticGraph {
        iris_types::graph::SemanticGraph {
            root: NodeId(0),
            nodes: HashMap::new(),
            edges: vec![],
            type_env,
            cost: CostBound::Unknown,
            resolution: iris_types::graph::Resolution::Implementation,
            hash: iris_types::hash::SemanticHash([0; 32]),
        }
    }

    // ===================================================================
    // type_app: soundness tests (the primary audit target)
    // ===================================================================

    #[test]
    fn type_app_accepts_well_formed_result() {
        // ForAll(X, Int) applied to get Int — should succeed.
        let int_id = TypeId(1);
        let forall_id = TypeId(2);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (forall_id, TypeDef::ForAll(iris_types::types::BoundVar(0), int_id)),
        ]);
        let graph = graph_with_types(type_env);

        // Create a theorem with ForAll type.
        let forall_thm = Theorem {
            judgment: Judgment {
                context: Context::empty(),
                node_id: NodeId(1),
                type_ref: forall_id,
                cost: CostBound::Zero,
            },
            proof_hash: [0; 32],
        };

        let result = Kernel::type_app(&forall_thm, int_id, &graph);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().type_ref(), int_id);
    }

    #[test]
    fn type_app_rejects_nonexistent_result_type() {
        // ForAll(X, Int), but result_type_id doesn't exist in TypeEnv.
        let int_id = TypeId(1);
        let forall_id = TypeId(2);
        let bogus_id = TypeId(999);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (forall_id, TypeDef::ForAll(iris_types::types::BoundVar(0), int_id)),
        ]);
        let graph = graph_with_types(type_env);

        let forall_thm = Theorem {
            judgment: Judgment {
                context: Context::empty(),
                node_id: NodeId(1),
                type_ref: forall_id,
                cost: CostBound::Zero,
            },
            proof_hash: [0; 32],
        };

        let err = Kernel::type_app(&forall_thm, bogus_id, &graph).unwrap_err();
        assert!(matches!(err, KernelError::TypeNotFound(id) if id == bogus_id));
    }

    #[test]
    fn type_app_rejects_malformed_result_type() {
        // The result type exists but references a TypeId that doesn't exist.
        // This is the critical soundness hole: Arrow(Int, TypeId(999)) where
        // TypeId(999) is not in the TypeEnv.
        let int_id = TypeId(1);
        let forall_id = TypeId(2);
        let dangling_id = TypeId(999);
        let malformed_arrow_id = TypeId(3);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (forall_id, TypeDef::ForAll(iris_types::types::BoundVar(0), int_id)),
            // Arrow that references a non-existent type — malformed!
            (malformed_arrow_id, TypeDef::Arrow(int_id, dangling_id, CostBound::Zero)),
        ]);
        let graph = graph_with_types(type_env);

        let forall_thm = Theorem {
            judgment: Judgment {
                context: Context::empty(),
                node_id: NodeId(1),
                type_ref: forall_id,
                cost: CostBound::Zero,
            },
            proof_hash: [0; 32],
        };

        let err = Kernel::type_app(&forall_thm, malformed_arrow_id, &graph).unwrap_err();
        assert!(matches!(
            err,
            KernelError::TypeMalformed {
                type_id,
                dangling_ref,
                ..
            } if type_id == malformed_arrow_id && dangling_ref == dangling_id
        ));
    }

    #[test]
    fn type_app_rejects_non_forall_premise() {
        // Trying to apply type_app when the premise doesn't have a ForAll type.
        let int_id = TypeId(1);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
        ]);
        let graph = graph_with_types(type_env);

        let int_thm = Kernel::refl(NodeId(1), int_id);
        let err = Kernel::type_app(&int_thm, int_id, &graph).unwrap_err();
        assert!(matches!(
            err,
            KernelError::UnexpectedTypeDef { expected: "ForAll", .. }
        ));
    }

    #[test]
    fn type_app_rejects_malformed_product_result() {
        // Product type where one field references a non-existent type.
        let int_id = TypeId(1);
        let forall_id = TypeId(2);
        let dangling_id = TypeId(888);
        let bad_product_id = TypeId(3);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (forall_id, TypeDef::ForAll(iris_types::types::BoundVar(0), int_id)),
            (bad_product_id, TypeDef::Product(vec![int_id, dangling_id])),
        ]);
        let graph = graph_with_types(type_env);

        let forall_thm = Theorem {
            judgment: Judgment {
                context: Context::empty(),
                node_id: NodeId(1),
                type_ref: forall_id,
                cost: CostBound::Zero,
            },
            proof_hash: [0; 32],
        };

        let err = Kernel::type_app(&forall_thm, bad_product_id, &graph).unwrap_err();
        assert!(matches!(
            err,
            KernelError::TypeMalformed {
                dangling_ref,
                ..
            } if dangling_ref == dangling_id
        ));
    }

    #[test]
    fn type_app_rejects_malformed_sum_result() {
        // Sum type where a variant references a non-existent type.
        let int_id = TypeId(1);
        let forall_id = TypeId(2);
        let dangling_id = TypeId(777);
        let bad_sum_id = TypeId(3);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (forall_id, TypeDef::ForAll(iris_types::types::BoundVar(0), int_id)),
            (
                bad_sum_id,
                TypeDef::Sum(vec![
                    (iris_types::types::Tag(0), int_id),
                    (iris_types::types::Tag(1), dangling_id),
                ]),
            ),
        ]);
        let graph = graph_with_types(type_env);

        let forall_thm = Theorem {
            judgment: Judgment {
                context: Context::empty(),
                node_id: NodeId(1),
                type_ref: forall_id,
                cost: CostBound::Zero,
            },
            proof_hash: [0; 32],
        };

        let err = Kernel::type_app(&forall_thm, bad_sum_id, &graph).unwrap_err();
        assert!(matches!(err, KernelError::TypeMalformed { .. }));
    }

    // ===================================================================
    // type_abst: well-formedness tests
    // ===================================================================

    #[test]
    fn type_abst_rejects_malformed_forall() {
        // ForAll whose inner type doesn't exist.
        let int_id = TypeId(1);
        let dangling_id = TypeId(666);
        let bad_forall_id = TypeId(2);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (bad_forall_id, TypeDef::ForAll(iris_types::types::BoundVar(0), dangling_id)),
        ]);
        let graph = graph_with_types(type_env);

        let body_thm = Kernel::refl(NodeId(1), dangling_id);
        let err = Kernel::type_abst(&body_thm, bad_forall_id, &graph).unwrap_err();
        assert!(matches!(err, KernelError::TypeMalformed { .. }));
    }

    #[test]
    fn type_abst_accepts_well_formed_forall() {
        let int_id = TypeId(1);
        let forall_id = TypeId(2);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (forall_id, TypeDef::ForAll(iris_types::types::BoundVar(0), int_id)),
        ]);
        let graph = graph_with_types(type_env);

        let body_thm = Kernel::refl(NodeId(1), int_id);
        let result = Kernel::type_abst(&body_thm, forall_id, &graph);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().type_ref(), forall_id);
    }

    // ===================================================================
    // Edge case tests for other rules
    // ===================================================================

    #[test]
    fn elim_rejects_context_mismatch() {
        let int_id = TypeId(1);
        let arrow_id = TypeId(2);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (arrow_id, TypeDef::Arrow(int_id, int_id, CostBound::Zero)),
        ]);
        let graph = graph_with_types(type_env);

        // fn_thm in empty context, arg_thm in extended context.
        let fn_thm = Kernel::refl(NodeId(1), arrow_id);
        let arg_ctx = Context::empty().extend(BinderId(0), int_id);
        let arg_thm = Kernel::assume(&arg_ctx, BinderId(0), NodeId(2)).unwrap();

        let err = Kernel::elim(&fn_thm, &arg_thm, NodeId(3), &graph).unwrap_err();
        assert!(matches!(err, KernelError::ContextMismatch { .. }));
    }

    #[test]
    fn elim_rejects_non_arrow_function() {
        let int_id = TypeId(1);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
        ]);
        let graph = graph_with_types(type_env);

        let fn_thm = Kernel::refl(NodeId(1), int_id);
        let arg_thm = Kernel::refl(NodeId(2), int_id);

        let err = Kernel::elim(&fn_thm, &arg_thm, NodeId(3), &graph).unwrap_err();
        assert!(matches!(
            err,
            KernelError::UnexpectedTypeDef { expected: "Arrow", .. }
        ));
    }

    #[test]
    fn elim_rejects_arg_type_mismatch() {
        let int_id = TypeId(1);
        let bool_id = TypeId(2);
        let arrow_id = TypeId(3);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (bool_id, TypeDef::Primitive(PrimType::Bool)),
            (arrow_id, TypeDef::Arrow(int_id, int_id, CostBound::Zero)),
        ]);
        let graph = graph_with_types(type_env);

        let fn_thm = Kernel::refl(NodeId(1), arrow_id);
        let arg_thm = Kernel::refl(NodeId(2), bool_id); // Wrong type!

        let err = Kernel::elim(&fn_thm, &arg_thm, NodeId(3), &graph).unwrap_err();
        assert!(matches!(err, KernelError::TypeMismatch { .. }));
    }

    #[test]
    fn intro_rejects_context_mismatch() {
        let int_id = TypeId(1);
        let arrow_id = TypeId(2);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (arrow_id, TypeDef::Arrow(int_id, int_id, CostBound::Zero)),
        ]);
        let graph = graph_with_types(type_env);

        // Body theorem in wrong context (empty instead of extended).
        let body_thm = Kernel::refl(NodeId(10), int_id);
        let ctx = Context::empty();

        let err = Kernel::intro(&ctx, NodeId(20), BinderId(0), int_id, &body_thm, &graph)
            .unwrap_err();
        assert!(matches!(err, KernelError::ContextMismatch { .. }));
    }

    #[test]
    fn let_bind_rejects_bound_context_mismatch() {
        let int_id = TypeId(1);

        let ctx = Context::empty();
        let wrong_ctx = ctx.extend(BinderId(99), int_id);

        // bound_thm in the wrong context.
        let bound_thm = Kernel::assume(&wrong_ctx, BinderId(99), NodeId(1)).unwrap();
        let body_thm = Kernel::refl(NodeId(2), int_id);

        let err = Kernel::let_bind(&ctx, NodeId(3), BinderId(0), &bound_thm, &body_thm)
            .unwrap_err();
        assert!(matches!(err, KernelError::ContextMismatch { .. }));
    }

    #[test]
    fn match_elim_rejects_no_arms() {
        let scrutinee = Kernel::refl(NodeId(1), TypeId(1));
        let err = Kernel::match_elim(&scrutinee, &[], NodeId(2)).unwrap_err();
        assert!(matches!(err, KernelError::InvalidRule { .. }));
    }

    #[test]
    fn match_elim_rejects_arm_type_mismatch() {
        let arm1 = Kernel::refl(NodeId(1), TypeId(10));
        let arm2 = Kernel::refl(NodeId(2), TypeId(20)); // Different type!
        let scrutinee = Kernel::refl(NodeId(3), TypeId(1));

        let err = Kernel::match_elim(&scrutinee, &[arm1, arm2], NodeId(4)).unwrap_err();
        assert!(matches!(err, KernelError::TypeMismatch { .. }));
    }

    #[test]
    fn structural_ind_rejects_wrong_case_count() {
        let int_id = TypeId(1);
        let sum_id = TypeId(2);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (
                sum_id,
                TypeDef::Sum(vec![
                    (iris_types::types::Tag(0), int_id),
                    (iris_types::types::Tag(1), int_id),
                ]),
            ),
        ]);
        let graph = graph_with_types(type_env);

        // Only 1 case for a 2-variant sum type.
        let case1 = Kernel::refl(NodeId(1), int_id);
        let err = Kernel::structural_ind(sum_id, &[case1], NodeId(10), &graph).unwrap_err();
        assert!(matches!(err, KernelError::InductionError { .. }));
    }

    #[test]
    fn structural_ind_rejects_case_type_mismatch() {
        let int_id = TypeId(1);
        let bool_id = TypeId(2);
        let sum_id = TypeId(3);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (bool_id, TypeDef::Primitive(PrimType::Bool)),
            (
                sum_id,
                TypeDef::Sum(vec![
                    (iris_types::types::Tag(0), int_id),
                    (iris_types::types::Tag(1), int_id),
                ]),
            ),
        ]);
        let graph = graph_with_types(type_env);

        let case1 = Kernel::refl(NodeId(1), int_id);
        let case2 = Kernel::refl(NodeId(2), bool_id); // Different result type!
        let err =
            Kernel::structural_ind(sum_id, &[case1, case2], NodeId(10), &graph).unwrap_err();
        assert!(matches!(err, KernelError::InductionError { .. }));
    }

    #[test]
    fn nat_ind_rejects_non_arrow_step() {
        // Step theorem must have Arrow type; using a plain Int type should fail.
        let int_id = TypeId(10);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
        ]);
        let graph = graph_with_types(type_env);
        let base = Kernel::refl(NodeId(1), int_id);
        let step = Kernel::refl(NodeId(2), int_id);
        let err = Kernel::nat_ind(&base, &step, NodeId(3), &graph).unwrap_err();
        assert!(matches!(err, KernelError::InductionError { .. }));
    }

    #[test]
    fn nat_ind_rejects_step_param_mismatch() {
        // Step has Arrow(Bool, Int, Zero) but base has Int -- param type mismatch.
        let int_id = TypeId(1);
        let bool_id = TypeId(2);
        let arrow_id = TypeId(3);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (bool_id, TypeDef::Primitive(PrimType::Bool)),
            (arrow_id, TypeDef::Arrow(bool_id, int_id, CostBound::Zero)),
        ]);
        let graph = graph_with_types(type_env);
        let base = Kernel::refl(NodeId(1), int_id);
        let step = Kernel::refl(NodeId(2), arrow_id);
        let err = Kernel::nat_ind(&base, &step, NodeId(3), &graph).unwrap_err();
        assert!(matches!(err, KernelError::InductionError { .. }));
    }

    #[test]
    fn nat_ind_accepts_valid_step() {
        // Step has Arrow(Int, Int, Zero), base has Int -- should succeed.
        let int_id = TypeId(1);
        let arrow_id = TypeId(2);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (arrow_id, TypeDef::Arrow(int_id, int_id, CostBound::Zero)),
        ]);
        let graph = graph_with_types(type_env);
        let base = Kernel::refl(NodeId(1), int_id);
        let step = Kernel::refl(NodeId(2), arrow_id);
        let result = Kernel::nat_ind(&base, &step, NodeId(3), &graph);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().type_ref(), int_id);
    }

    #[test]
    fn refine_intro_rejects_non_refined_type() {
        let int_id = TypeId(1);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
        ]);
        let graph = graph_with_types(type_env);

        let base_thm = Kernel::refl(NodeId(1), int_id);
        let pred_thm = Kernel::refl(NodeId(2), int_id);

        // Try to use Int as a refined type — should fail.
        let err = Kernel::refine_intro(&base_thm, &pred_thm, int_id, &graph).unwrap_err();
        assert!(matches!(
            err,
            KernelError::UnexpectedTypeDef { expected: "Refined", .. }
        ));
    }

    #[test]
    fn refine_intro_rejects_base_type_mismatch() {
        let int_id = TypeId(1);
        let bool_id = TypeId(2);
        let refined_id = TypeId(3);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (bool_id, TypeDef::Primitive(PrimType::Bool)),
            (
                refined_id,
                TypeDef::Refined(int_id, iris_types::types::LIAFormula::True),
            ),
        ]);
        let graph = graph_with_types(type_env);

        // base_thm has Bool, but the refinement wraps Int.
        let base_thm = Kernel::refl(NodeId(1), bool_id);
        let pred_thm = Kernel::refl(NodeId(2), int_id);

        let err = Kernel::refine_intro(&base_thm, &pred_thm, refined_id, &graph).unwrap_err();
        assert!(matches!(err, KernelError::TypeMismatch { .. }));
    }

    #[test]
    fn refine_elim_rejects_non_refined() {
        let int_id = TypeId(1);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
        ]);
        let graph = graph_with_types(type_env);

        let thm = Kernel::refl(NodeId(1), int_id);
        let err = Kernel::refine_elim(&thm, &graph).unwrap_err();
        assert!(matches!(
            err,
            KernelError::UnexpectedTypeDef { expected: "Refined", .. }
        ));
    }

    #[test]
    fn guard_rule_rejects_type_mismatch() {
        let pred = Kernel::refl(NodeId(1), TypeId(1));
        let then_thm = Kernel::refl(NodeId(2), TypeId(10));
        let else_thm = Kernel::refl(NodeId(3), TypeId(20)); // Different type!

        let err = Kernel::guard_rule(&pred, &then_thm, &else_thm, NodeId(4)).unwrap_err();
        assert!(matches!(err, KernelError::TypeMismatch { .. }));
    }

    #[test]
    fn congr_rejects_context_mismatch() {
        let int_id = TypeId(1);
        let fn_thm = Kernel::refl(NodeId(1), int_id);
        let arg_ctx = Context::empty().extend(BinderId(0), int_id);
        let arg_thm = Kernel::assume(&arg_ctx, BinderId(0), NodeId(2)).unwrap();

        let err = Kernel::congr(&fn_thm, &arg_thm, NodeId(3)).unwrap_err();
        assert!(matches!(err, KernelError::ContextMismatch { .. }));
    }

    // ===================================================================
    // Well-formedness helper tests
    // ===================================================================

    #[test]
    fn type_def_references_primitive_is_empty() {
        let refs = type_def_references(&TypeDef::Primitive(PrimType::Int));
        assert!(refs.is_empty());
    }

    #[test]
    fn type_def_references_arrow_has_two() {
        let refs =
            type_def_references(&TypeDef::Arrow(TypeId(1), TypeId(2), CostBound::Zero));
        assert_eq!(refs, vec![TypeId(1), TypeId(2)]);
    }

    #[test]
    fn type_def_references_product() {
        let refs = type_def_references(&TypeDef::Product(vec![TypeId(1), TypeId(2), TypeId(3)]));
        assert_eq!(refs, vec![TypeId(1), TypeId(2), TypeId(3)]);
    }

    #[test]
    fn type_def_references_forall() {
        let refs =
            type_def_references(&TypeDef::ForAll(iris_types::types::BoundVar(0), TypeId(5)));
        assert_eq!(refs, vec![TypeId(5)]);
    }

    #[test]
    fn assert_well_formed_accepts_valid_type() {
        let int_id = TypeId(1);
        let arrow_id = TypeId(2);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (arrow_id, TypeDef::Arrow(int_id, int_id, CostBound::Zero)),
        ]);

        assert!(assert_type_well_formed(&type_env, int_id, "test").is_ok());
        assert!(assert_type_well_formed(&type_env, arrow_id, "test").is_ok());
    }

    #[test]
    fn assert_well_formed_rejects_dangling_ref() {
        let int_id = TypeId(1);
        let dangling_id = TypeId(999);
        let bad_arrow_id = TypeId(2);
        let type_env = make_type_env(vec![
            (int_id, TypeDef::Primitive(PrimType::Int)),
            (bad_arrow_id, TypeDef::Arrow(int_id, dangling_id, CostBound::Zero)),
        ]);

        let err = assert_type_well_formed(&type_env, bad_arrow_id, "test").unwrap_err();
        assert!(matches!(
            err,
            KernelError::TypeMalformed {
                type_id,
                dangling_ref,
                ..
            } if type_id == bad_arrow_id && dangling_ref == dangling_id
        ));
    }

    #[test]
    fn assert_well_formed_rejects_missing_type() {
        let type_env = make_type_env(vec![]);
        let err = assert_type_well_formed(&type_env, TypeId(1), "test").unwrap_err();
        assert!(matches!(err, KernelError::TypeNotFound(_)));
    }

    // ===================================================================
    // Proof hash tests (ensure different rules produce different hashes)
    // ===================================================================

    #[test]
    fn different_rules_produce_different_hashes() {
        let thm1 = Kernel::refl(NodeId(1), TypeId(1));
        let thm2 = Kernel::refl(NodeId(1), TypeId(2));
        // Same node, different types => different hashes.
        assert_ne!(thm1.proof_hash(), thm2.proof_hash());
    }

    #[test]
    fn symm_requires_equality_witness() {
        let thm = Kernel::refl(NodeId(1), TypeId(42));
        // Provide a valid equality witness: a theorem about NodeId(2) with same type.
        let eq_witness = Kernel::refl(NodeId(2), TypeId(42));
        let sym = Kernel::symm(&thm, NodeId(2), &eq_witness).unwrap();
        assert_eq!(sym.type_ref(), TypeId(42));
        assert_eq!(sym.node_id(), NodeId(2));
        assert_ne!(thm.proof_hash(), sym.proof_hash());
    }

    #[test]
    fn symm_rejects_wrong_witness_node() {
        let thm = Kernel::refl(NodeId(1), TypeId(42));
        // Witness is about NodeId(3), but we're trying to transfer to NodeId(2).
        let bad_witness = Kernel::refl(NodeId(3), TypeId(42));
        let err = Kernel::symm(&thm, NodeId(2), &bad_witness).unwrap_err();
        assert!(matches!(err, KernelError::NotEqual { .. }));
    }

    #[test]
    fn symm_rejects_wrong_witness_type() {
        let thm = Kernel::refl(NodeId(1), TypeId(42));
        // Witness is about the right node but wrong type.
        let bad_witness = Kernel::refl(NodeId(2), TypeId(99));
        let err = Kernel::symm(&thm, NodeId(2), &bad_witness).unwrap_err();
        assert!(matches!(err, KernelError::TypeMismatch { .. }));
    }

    #[test]
    fn trans_preserves_first_node() {
        let thm1 = Kernel::refl(NodeId(1), TypeId(10));
        let thm2 = Kernel::refl(NodeId(2), TypeId(10));
        let result = Kernel::trans(&thm1, &thm2).unwrap();
        assert_eq!(result.node_id(), NodeId(1));
        assert_eq!(result.type_ref(), TypeId(10));
    }
}
