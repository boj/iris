import IrisKernel.Types

/-!
# IRIS Proof Kernel — Inference Rules

Natural deduction-style formalization of the 20 inference rules from
`src/iris-kernel/src/kernel.rs`.

The rules are defined as an inductive type `Derivation` whose constructors
correspond one-to-one with the Rust `Kernel` methods:

| #  | Constructor      | Rust method        | Type-theoretic counterpart            |
|----|------------------|--------------------|---------------------------------------|
|  1 | assume           | Kernel::assume     | Variable rule (Var)                   |
|  2 | intro            | Kernel::intro      | Arrow introduction (→I)               |
|  3 | elim             | Kernel::elim       | Arrow elimination / modus ponens (→E) |
|  4 | refl             | Kernel::refl       | Reflexivity of equality               |
|  5 | symm             | Kernel::symm       | Symmetry of equality                  |
|  6 | trans            | Kernel::trans      | Transitivity of equality              |
|  7 | congr            | Kernel::congr      | Congruence of equality                |
|  8 | type_check_node  | Kernel::type_check_node | Annotation / axiom schema        |
|  9 | cost_subsume     | Kernel::cost_subsume | Cost subsumption                    |
| 10 | cost_leq_rule    | Kernel::cost_leq_rule | Cost ordering witness              |
| 11 | refine_intro     | Kernel::refine_intro | Refinement type introduction        |
| 12 | refine_elim      | Kernel::refine_elim  | Refinement type elimination         |
| 13 | nat_ind          | Kernel::nat_ind      | Natural number induction            |
| 14 | structural_ind   | Kernel::structural_ind | Structural induction over ADTs    |
| 15 | let_bind         | Kernel::let_bind     | Let binding (cut rule)              |
| 16 | match_elim       | Kernel::match_elim   | Sum elimination / case analysis     |
| 17 | fold_rule        | Kernel::fold_rule    | Catamorphism / structural recursion |
| 18 | type_abst        | Kernel::type_abst    | ForAll introduction (∀I)            |
| 19 | type_app         | Kernel::type_app     | ForAll elimination (∀E)             |
| 20 | guard_rule       | Kernel::guard_rule   | Conditional / if-then-else          |
-/

namespace IrisKernel

-- ===========================================================================
-- Sup cost helper
-- ===========================================================================

/-- Compute the supremum of a list of costs. -/
def costSup : List CostBound → CostBound
  | []  => CostBound.Zero
  | [c] => c
  | cs  => CostBound.Sup cs

-- ===========================================================================
-- Derivation — the 20 inference rules as an inductive Prop
-- ===========================================================================

/-- `Derivation env Γ n τ κ` asserts that in type environment `env` and
    typing context `Γ`, node `n` has type `τ` with cost bound `κ`.

    Written `env; Γ ⊢ n : τ @ κ`.

    Each constructor corresponds to exactly one inference rule of the
    IRIS proof kernel. -/
inductive Derivation : TypeEnv → Context → NodeId → TypeId → CostBound → Prop where

  -- -----------------------------------------------------------------------
  -- 1. assume: Γ, x:A ⊢ x : A @ Zero
  -- Rust: Kernel::assume
  -- -----------------------------------------------------------------------
  /-- If binder `name` has type `τ` in context `Γ`, then
      `Γ ⊢ n : τ @ Zero`. -/
  | assume :
      (env : TypeEnv) →
      (Γ : Context) →
      (name : BinderId) →
      (n : NodeId) →
      (τ : TypeId) →
      Γ.lookup name = some τ →
      Derivation env Γ n τ CostBound.Zero

  -- -----------------------------------------------------------------------
  -- 2. intro: if Γ,x:A ⊢ body : B @ κ then Γ ⊢ λ : Arrow(A,B,κ) @ Zero
  -- Rust: Kernel::intro
  -- -----------------------------------------------------------------------
  /-- Arrow introduction. Given a derivation of the body in the extended
      context, produce a derivation for the lambda at arrow type. -/
  | intro :
      (env : TypeEnv) →
      (Γ : Context) →
      (lam : NodeId) →
      (binder : BinderId) →
      (A B arrow_id : TypeId) →
      (κ_body : CostBound) →
      (body : NodeId) →
      -- The body is typed in the extended context
      Derivation env (Γ.extend binder A) body B κ_body →
      -- The arrow type Arrow(A, B, κ_body) exists in the type env
      env.lookup arrow_id = some (TypeDef.Arrow A B κ_body) →
      Derivation env Γ lam arrow_id CostBound.Zero

  -- -----------------------------------------------------------------------
  -- 3. elim: if Γ ⊢ f : Arrow(A,B,κ_b) @ κ_f and Γ ⊢ a : A @ κ_a
  --    then Γ ⊢ app : B @ Sum(κ_a, Sum(κ_f, κ_b))
  -- Rust: Kernel::elim
  -- -----------------------------------------------------------------------
  /-- Arrow elimination (function application / modus ponens). -/
  | elim :
      (env : TypeEnv) →
      (Γ : Context) →
      (f a app : NodeId) →
      (A B arrow_id : TypeId) →
      (κ_f κ_a κ_body : CostBound) →
      -- f has arrow type
      Derivation env Γ f arrow_id κ_f →
      -- The arrow type is Arrow(A, B, κ_body)
      env.lookup arrow_id = some (TypeDef.Arrow A B κ_body) →
      -- a has the parameter type A
      Derivation env Γ a A κ_a →
      -- Result: app has type B with combined cost
      Derivation env Γ app B (CostBound.Sum κ_a (CostBound.Sum κ_f κ_body))

  -- -----------------------------------------------------------------------
  -- 4. refl: Γ ⊢ n : τ @ Zero (reflexivity)
  -- Rust: Kernel::refl
  -- -----------------------------------------------------------------------
  /-- Reflexivity: any node equals itself. Produces a theorem at zero cost.
      Works in any context (the Rust kernel constructs the theorem from the
      node's annotation without inspecting the context). -/
  | refl :
      (env : TypeEnv) →
      (Γ : Context) →
      (n : NodeId) →
      (τ : TypeId) →
      Derivation env Γ n τ CostBound.Zero

  -- -----------------------------------------------------------------------
  -- 5. symm: if Γ ⊢ a : τ @ κ then Γ ⊢ b : τ @ κ
  -- Rust: Kernel::symm
  -- -----------------------------------------------------------------------
  /-- Symmetry of equality. Given a derivation for node `a`, produce
      one for node `b` with the same type and cost. -/
  | symm :
      (env : TypeEnv) →
      (Γ : Context) →
      (a b : NodeId) →
      (τ : TypeId) →
      (κ : CostBound) →
      Derivation env Γ a τ κ →
      Derivation env Γ b τ κ

  -- -----------------------------------------------------------------------
  -- 6. trans: if Γ ⊢ a : τ @ κ₁ and Γ ⊢ b : τ @ κ₂ then Γ ⊢ a : τ @ κ₂
  -- Rust: Kernel::trans
  -- -----------------------------------------------------------------------
  /-- Transitivity of equality. Chains two derivations about the same type.
      The result keeps the node of the first and the cost of the second. -/
  | trans :
      (env : TypeEnv) →
      (Γ : Context) →
      (a b : NodeId) →
      (τ : TypeId) →
      (κ₁ κ₂ : CostBound) →
      Derivation env Γ a τ κ₁ →
      Derivation env Γ b τ κ₂ →
      Derivation env Γ a τ κ₂

  -- -----------------------------------------------------------------------
  -- 7. congr: if Γ ⊢ f : τ @ κ_f and Γ ⊢ a : σ @ κ_a
  --    then Γ ⊢ app : τ @ Sum(κ_f, κ_a)
  -- Rust: Kernel::congr
  -- -----------------------------------------------------------------------
  /-- Congruence: if function and argument are equal, application is equal.
      Result type comes from the function; cost is the sum. -/
  | congr :
      (env : TypeEnv) →
      (Γ : Context) →
      (f a app : NodeId) →
      (τ σ : TypeId) →
      (κ_f κ_a : CostBound) →
      Derivation env Γ f τ κ_f →
      Derivation env Γ a σ κ_a →
      Derivation env Γ app τ (CostBound.Sum κ_f κ_a)

  -- -----------------------------------------------------------------------
  -- 8. type_check_node: trust the node's annotation
  -- Rust: Kernel::type_check_node
  --
  -- This is the "axiom schema" rule. In Rust, it reads the node from the
  -- graph and trusts its type_sig. We model this by requiring the type
  -- to be well-formed in the type environment.
  -- -----------------------------------------------------------------------
  /-- Type-check a node against its annotation. The type must exist and be
      well-formed in the type environment. Cost depends on the node kind. -/
  | type_check_node :
      (env : TypeEnv) →
      (Γ : Context) →
      (n : NodeId) →
      (τ : TypeId) →
      (κ : CostBound) →
      -- The type is well-formed in the environment
      TypeWellFormed env τ →
      Derivation env Γ n τ κ

  -- -----------------------------------------------------------------------
  -- 9. cost_subsume: if Γ ⊢ n : τ @ κ₁ and κ₁ ≤ κ₂ then Γ ⊢ n : τ @ κ₂
  -- Rust: Kernel::cost_subsume
  -- -----------------------------------------------------------------------
  /-- Cost subsumption: weaken a cost bound. -/
  | cost_subsume :
      (env : TypeEnv) →
      (Γ : Context) →
      (n : NodeId) →
      (τ : TypeId) →
      (κ₁ κ₂ : CostBound) →
      Derivation env Γ n τ κ₁ →
      CostLeq κ₁ κ₂ →
      Derivation env Γ n τ κ₂

  -- -----------------------------------------------------------------------
  -- 10. cost_leq_rule: witness that κ₁ ≤ κ₂
  -- Rust: Kernel::cost_leq_rule
  --
  -- In Rust this produces a "dummy" theorem (NodeId(0), TypeId(0)).
  -- We model it as a derivation about the dummy node with cost κ₂.
  -- -----------------------------------------------------------------------
  /-- Cost ordering witness. Produces a derivation that κ₁ ≤ κ₂ holds,
      represented as a derivation about dummy node 0 with cost κ₂. -/
  | cost_leq_rule :
      (env : TypeEnv) →
      (Γ : Context) →
      (κ₁ κ₂ : CostBound) →
      CostLeq κ₁ κ₂ →
      Derivation env Γ ⟨0⟩ ⟨0⟩ κ₂

  -- -----------------------------------------------------------------------
  -- 11. refine_intro: if Γ ⊢ n : A @ κ and predicate holds
  --     then Γ ⊢ n : {x:A|P} @ κ
  -- Rust: Kernel::refine_intro
  -- -----------------------------------------------------------------------
  /-- Refinement type introduction. Given a derivation for the base type
      and a derivation witnessing that the predicate holds, produce a
      derivation at the refined type. -/
  | refine_intro :
      (env : TypeEnv) →
      (Γ : Context) →
      (n : NodeId) →
      (base_type refined_type : TypeId) →
      (κ : CostBound) →
      -- n has the base type
      Derivation env Γ n base_type κ →
      -- The refined type wraps the base type
      env.lookup refined_type = some (TypeDef.Refined base_type) →
      -- Predicate holds (witnessed by a separate derivation)
      (pred_node : NodeId) →
      (pred_type : TypeId) →
      (κ_pred : CostBound) →
      Derivation env Γ pred_node pred_type κ_pred →
      Derivation env Γ n refined_type κ

  -- -----------------------------------------------------------------------
  -- 12. refine_elim: if Γ ⊢ n : {x:A|P} @ κ then Γ ⊢ n : A @ κ
  -- Rust: Kernel::refine_elim
  -- -----------------------------------------------------------------------
  /-- Refinement type elimination. Extract the base type from a
      refinement type. -/
  | refine_elim :
      (env : TypeEnv) →
      (Γ : Context) →
      (n : NodeId) →
      (base_type refined_type : TypeId) →
      (κ : CostBound) →
      Derivation env Γ n refined_type κ →
      env.lookup refined_type = some (TypeDef.Refined base_type) →
      Derivation env Γ n base_type κ

  -- -----------------------------------------------------------------------
  -- 13. nat_ind: base + step ⟹ ∀n.P(n)
  -- Rust: Kernel::nat_ind
  -- -----------------------------------------------------------------------
  /-- Natural number induction. Given base case and inductive step (both
      producing the same type), derive the property for all naturals.
      Cost is Sum(κ_base, κ_step). -/
  | nat_ind :
      (env : TypeEnv) →
      (Γ : Context) →
      (base_node step_node result_node : NodeId) →
      (τ : TypeId) →
      (κ_base κ_step : CostBound) →
      Derivation env Γ base_node τ κ_base →
      Derivation env Γ step_node τ κ_step →
      Derivation env Γ result_node τ (CostBound.Sum κ_base κ_step)

  -- -----------------------------------------------------------------------
  -- 14. structural_ind: one case per constructor of a Sum type
  -- Rust: Kernel::structural_ind
  -- -----------------------------------------------------------------------
  /-- Structural induction over an algebraic data type (Sum type).
      Given one derivation per constructor variant, all producing the same
      result type, derive the result with cost Sup(case costs).

      `case_nodes`, `case_costs` are parallel lists — one per variant.
      `variants` is the list of (Tag, TypeId) from the Sum type. -/
  | structural_ind :
      (env : TypeEnv) →
      (Γ : Context) →
      (sum_type result_type : TypeId) →
      (result_node : NodeId) →
      (variants : List (Tag × TypeId)) →
      (case_nodes : List NodeId) →
      (case_costs : List CostBound) →
      -- The sum type has these variants
      env.lookup sum_type = some (TypeDef.Sum variants) →
      -- One case per variant
      variants.length = case_nodes.length →
      case_nodes.length = case_costs.length →
      -- Non-empty
      0 < variants.length →
      -- Each case derives the result type
      (∀ (i : Nat) (hn : i < case_nodes.length) (hc : i < case_costs.length),
        Derivation env Γ (case_nodes.get ⟨i, hn⟩) result_type (case_costs.get ⟨i, hc⟩)) →
      -- Result: cost is Sup of all case costs
      Derivation env Γ result_node result_type (CostBound.Sup case_costs)

  -- -----------------------------------------------------------------------
  -- 15. let_bind: Γ ⊢ e₁:A @ κ₁, Γ,x:A ⊢ e₂:B @ κ₂
  --     ⟹ Γ ⊢ let x=e₁ in e₂ : B @ Sum(κ₁, κ₂)
  -- Rust: Kernel::let_bind
  -- -----------------------------------------------------------------------
  /-- Let binding (cut rule). -/
  | let_bind :
      (env : TypeEnv) →
      (Γ : Context) →
      (let_node bound body : NodeId) →
      (binder : BinderId) →
      (A B : TypeId) →
      (κ₁ κ₂ : CostBound) →
      -- e₁ has type A in Γ
      Derivation env Γ bound A κ₁ →
      -- e₂ has type B in Γ extended with x:A
      Derivation env (Γ.extend binder A) body B κ₂ →
      Derivation env Γ let_node B (CostBound.Sum κ₁ κ₂)

  -- -----------------------------------------------------------------------
  -- 16. match_elim: Sum elimination / case analysis
  -- Rust: Kernel::match_elim
  -- -----------------------------------------------------------------------
  /-- Match elimination. Given a scrutinee derivation and one arm per
      constructor (all producing the same result type), derive the match
      with cost Sum(κ_scrutinee, Sup(arm costs)). -/
  | match_elim :
      (env : TypeEnv) →
      (Γ : Context) →
      (scrutinee match_node : NodeId) →
      (scrutinee_type result_type : TypeId) →
      (κ_scrutinee : CostBound) →
      (arm_nodes : List NodeId) →
      (arm_costs : List CostBound) →
      -- Scrutinee derivation
      Derivation env Γ scrutinee scrutinee_type κ_scrutinee →
      -- Non-empty arms
      0 < arm_nodes.length →
      -- Same number of arms and costs
      arm_nodes.length = arm_costs.length →
      -- Each arm derives the result type
      (∀ (i : Nat) (hn : i < arm_nodes.length) (hc : i < arm_costs.length),
        Derivation env Γ (arm_nodes.get ⟨i, hn⟩) result_type (arm_costs.get ⟨i, hc⟩)) →
      Derivation env Γ match_node result_type
        (CostBound.Sum κ_scrutinee (CostBound.Sup arm_costs))

  -- -----------------------------------------------------------------------
  -- 17. fold_rule: structural recursion (catamorphism)
  -- Rust: Kernel::fold_rule
  -- Cost: Sum(κ_input, Sum(κ_base, Mul(κ_step, κ_input)))
  -- -----------------------------------------------------------------------
  /-- Fold rule (catamorphism / structural recursion).
      Cost accounts for input traversal, base case, and step applied
      proportionally to input size. -/
  | fold_rule :
      (env : TypeEnv) →
      (Γ : Context) →
      (base_node step_node input_node fold_node : NodeId) →
      (result_type input_type step_type : TypeId) →
      (κ_base κ_step κ_input : CostBound) →
      Derivation env Γ base_node result_type κ_base →
      Derivation env Γ step_node step_type κ_step →
      Derivation env Γ input_node input_type κ_input →
      Derivation env Γ fold_node result_type
        (CostBound.Sum κ_input (CostBound.Sum κ_base (CostBound.Mul κ_step κ_input)))

  -- -----------------------------------------------------------------------
  -- 18. type_abst: if Γ ⊢ n : T @ κ then Γ ⊢ n : ∀X.T @ κ
  -- Rust: Kernel::type_abst
  -- -----------------------------------------------------------------------
  /-- ForAll introduction (type abstraction). -/
  | type_abst :
      (env : TypeEnv) →
      (Γ : Context) →
      (n : NodeId) →
      (inner forall_type : TypeId) →
      (bv : BoundVar) →
      (κ : CostBound) →
      Derivation env Γ n inner κ →
      -- ForAll type exists and wraps the inner type
      env.lookup forall_type = some (TypeDef.ForAll bv inner) →
      -- ForAll type is well-formed
      TypeWellFormed env forall_type →
      Derivation env Γ n forall_type κ

  -- -----------------------------------------------------------------------
  -- 19. type_app: if Γ ⊢ n : ∀X.T @ κ then Γ ⊢ n : T[S/X] @ κ
  -- Rust: Kernel::type_app
  -- -----------------------------------------------------------------------
  /-- ForAll elimination (type application).
      The result type must be well-formed (all referenced TypeIds exist). -/
  | type_app :
      (env : TypeEnv) →
      (Γ : Context) →
      (n : NodeId) →
      (forall_type result_type : TypeId) →
      (bv : BoundVar) →
      (inner : TypeId) →
      (κ : CostBound) →
      Derivation env Γ n forall_type κ →
      env.lookup forall_type = some (TypeDef.ForAll bv inner) →
      -- Result type is well-formed (critical soundness check)
      TypeWellFormed env result_type →
      Derivation env Γ n result_type κ

  -- -----------------------------------------------------------------------
  -- 20. guard_rule: Guard(pred, then, else) : B @ Sum(κ_pred, Sup(κ_then, κ_else))
  -- Rust: Kernel::guard_rule
  -- -----------------------------------------------------------------------
  /-- Guard rule (conditional / if-then-else).
      Then and else branches must have the same result type. -/
  | guard_rule :
      (env : TypeEnv) →
      (Γ : Context) →
      (pred_node then_node else_node guard_node : NodeId) →
      (pred_type result_type : TypeId) →
      (κ_pred κ_then κ_else : CostBound) →
      Derivation env Γ pred_node pred_type κ_pred →
      Derivation env Γ then_node result_type κ_then →
      Derivation env Γ else_node result_type κ_else →
      Derivation env Γ guard_node result_type
        (CostBound.Sum κ_pred (CostBound.Sup [κ_then, κ_else]))

end IrisKernel
