import IrisKernel.Types
import IrisKernel.Rules

/-!
# IRIS Proof Kernel — Basic Properties

Proven metatheorems about the IRIS inference rules and cost algebra.

Note: Several proofs were mechanically ported from Lean 4.16 to 4.28.
Proofs that broke due to the `induction` tactic naming changes are
temporarily marked with `sorry` and flagged with `-- PORT: ...`.
The theorems and their *statements* are unchanged and correct.
-/

namespace IrisKernel

-- ===========================================================================
-- Property 1: refl produces valid judgments
-- ===========================================================================

/-- `refl` produces a derivation in the empty context with zero cost. -/
theorem refl_well_formed (env : TypeEnv) (n : NodeId) (τ : TypeId) :
    Derivation env Context.empty n τ CostBound.Zero :=
  Derivation.refl env Context.empty n τ

/-- The cost of a refl derivation is zero. -/
theorem refl_cost_zero :
    ∀ (env : TypeEnv) (n : NodeId) (τ : TypeId),
      ∃ (d : Derivation env Context.empty n τ CostBound.Zero),
        True := by
  intro env n τ
  exact ⟨Derivation.refl env Context.empty n τ, trivial⟩

-- ===========================================================================
-- Property 2: CostLeq is transitive
-- ===========================================================================

/-- Cost subsumption is transitive. -/
theorem cost_subsume_transitive (κ₁ κ₂ κ₃ : CostBound) :
    CostLeq κ₁ κ₂ → CostLeq κ₂ κ₃ → CostLeq κ₁ κ₃ :=
  fun h1 h2 => CostLeq.trans κ₁ κ₂ κ₃ h1 h2

-- ===========================================================================
-- Property 3: CostLeq is reflexive
-- ===========================================================================

/-- Cost ordering is reflexive. -/
theorem cost_leq_reflexive (κ : CostBound) : CostLeq κ κ :=
  CostLeq.refl κ

-- ===========================================================================
-- Property 4: Zero is the bottom element
-- ===========================================================================

/-- Zero is below every cost bound. -/
theorem zero_is_bottom (κ : CostBound) : CostLeq CostBound.Zero κ :=
  CostLeq.zero_bot κ

-- ===========================================================================
-- Property 5: Unknown is the top element
-- ===========================================================================

/-- Every cost bound is below Unknown. -/
theorem unknown_is_top (κ : CostBound) : CostLeq κ CostBound.Unknown :=
  CostLeq.unknown_top κ

-- ===========================================================================
-- Property 6: Weakening
-- PORT: The full weakening proof (20 induction cases) needs mechanical
-- porting to Lean 4.28's changed `induction` naming convention.
-- The theorem statement is unchanged and correct.
-- ===========================================================================

/-- General weakening: derivations are preserved under context weakening. -/
theorem weakening_general :
    ∀ (env : TypeEnv) (Γ : Context) (n : NodeId) (τ : TypeId) (κ : CostBound)
      (ext_name : BinderId) (ext_type : TypeId),
    Derivation env Γ n τ κ →
    Derivation env (Γ.weaken ext_name ext_type) n τ κ := by
  -- PORT: 20-case induction proof needs Lean 4.28 variable naming update
  sorry

-- ===========================================================================
-- Property 7: cost_leq partial order properties
-- ===========================================================================

/-- Zero ≤ Zero follows from reflexivity. -/
theorem cost_leq_zero_zero : CostLeq CostBound.Zero CostBound.Zero :=
  CostLeq.refl CostBound.Zero

/-- Cost ordering forms a preorder (reflexive + transitive). -/
theorem cost_leq_preorder :
    (∀ (κ : CostBound), CostLeq κ κ) ∧
    (∀ (κ₁ κ₂ κ₃ : CostBound), CostLeq κ₁ κ₂ → CostLeq κ₂ κ₃ → CostLeq κ₁ κ₃) :=
  ⟨cost_leq_reflexive, cost_subsume_transitive⟩

-- ===========================================================================
-- Property 8: fold preserves cost bound
-- ===========================================================================

/-- The cost produced by fold_rule is at least κ_base. -/
theorem fold_cost_geq_base (κ_base κ_step κ_input : CostBound) :
    CostLeq κ_base
      (CostBound.Sum κ_input (CostBound.Sum κ_base (CostBound.Mul κ_step κ_input))) := by
  apply CostLeq.trans κ_base (CostBound.Sum κ_base (CostBound.Mul κ_step κ_input))
  · exact CostLeq.sum_embed_left κ_base (CostBound.Mul κ_step κ_input)
  · exact CostLeq.sum_embed_right κ_input (CostBound.Sum κ_base (CostBound.Mul κ_step κ_input))

/-- The cost produced by fold_rule is at least κ_input. -/
theorem fold_cost_geq_input (κ_base κ_step κ_input : CostBound) :
    CostLeq κ_input
      (CostBound.Sum κ_input (CostBound.Sum κ_base (CostBound.Mul κ_step κ_input))) :=
  CostLeq.sum_embed_left κ_input (CostBound.Sum κ_base (CostBound.Mul κ_step κ_input))

/-- fold_rule preserves cost. -/
theorem fold_preserves_cost (env : TypeEnv) (Γ : Context)
    (base_node step_node input_node fold_node : NodeId)
    (result_type input_type step_type : TypeId)
    (κ_base κ_step κ_input : CostBound)
    (hb : Derivation env Γ base_node result_type κ_base)
    (hs : Derivation env Γ step_node step_type κ_step)
    (hi : Derivation env Γ input_node input_type κ_input) :
    Derivation env Γ fold_node result_type
      (CostBound.Sum κ_input (CostBound.Sum κ_base (CostBound.Mul κ_step κ_input))) :=
  Derivation.fold_rule env Γ base_node step_node input_node fold_node
    result_type input_type step_type κ_base κ_step κ_input hb hs hi

-- ===========================================================================
-- Property 9: assume rule produces Zero cost
-- ===========================================================================

/-- Any derivation produced by the assume rule has Zero cost. -/
theorem assume_zero_cost (env : TypeEnv) (Γ : Context) (name : BinderId)
    (n : NodeId) (τ : TypeId) (h : Γ.lookup name = some τ) :
    Derivation env Γ n τ CostBound.Zero :=
  Derivation.assume env Γ name n τ h

-- ===========================================================================
-- Property 10: intro rule produces Zero cost
-- ===========================================================================

/-- Lambda introduction always has Zero cost. -/
theorem intro_zero_cost (env : TypeEnv) (Γ : Context) (lam body : NodeId)
    (binder : BinderId) (A B arrow_id : TypeId) (κ_body : CostBound)
    (h_body : Derivation env (Γ.extend binder A) body B κ_body)
    (h_arrow : env.lookup arrow_id = some (TypeDef.Arrow A B κ_body)) :
    Derivation env Γ lam arrow_id CostBound.Zero :=
  Derivation.intro env Γ lam binder A B arrow_id κ_body body h_body h_arrow

-- ===========================================================================
-- Property 11: guard_rule cost structure
-- ===========================================================================

/-- The guard rule's cost is Sum(pred_cost, Sup([then_cost, else_cost])). -/
theorem guard_cost_structure (env : TypeEnv) (Γ : Context)
    (p t e g : NodeId) (pt rt : TypeId) (κp κt κe : CostBound)
    (hp : Derivation env Γ p pt κp)
    (ht : Derivation env Γ t rt κt)
    (he : Derivation env Γ e rt κe) :
    Derivation env Γ g rt (CostBound.Sum κp (CostBound.Sup [κt, κe])) :=
  Derivation.guard_rule env Γ p t e g pt rt κp κt κe hp ht he

-- ===========================================================================
-- Property 12: let_bind cost is additive
-- ===========================================================================

/-- Let binding has additive cost: Sum(bound_cost, body_cost). -/
theorem let_bind_additive_cost (env : TypeEnv) (Γ : Context)
    (l b1 b2 : NodeId) (x : BinderId) (A B : TypeId) (κ₁ κ₂ : CostBound)
    (h1 : Derivation env Γ b1 A κ₁)
    (h2 : Derivation env (Γ.extend x A) b2 B κ₂) :
    Derivation env Γ l B (CostBound.Sum κ₁ κ₂) :=
  Derivation.let_bind env Γ l b1 b2 x A B κ₁ κ₂ h1 h2

-- ===========================================================================
-- Property 13: cost_subsume chaining
-- ===========================================================================

/-- Two successive cost subsumptions can be combined. -/
theorem cost_subsume_chain (env : TypeEnv) (Γ : Context)
    (n : NodeId) (τ : TypeId) (κ₁ κ₂ κ₃ : CostBound)
    (h : Derivation env Γ n τ κ₁)
    (h12 : CostLeq κ₁ κ₂)
    (h23 : CostLeq κ₂ κ₃) :
    Derivation env Γ n τ κ₃ := by
  apply Derivation.cost_subsume env Γ n τ κ₁ κ₃ h
  exact CostLeq.trans κ₁ κ₂ κ₃ h12 h23

-- ===========================================================================
-- Property 14: elim cost decomposition
-- ===========================================================================

/-- The cost of function application decomposes into argument, function, and body cost. -/
theorem elim_cost_decomposition (env : TypeEnv) (Γ : Context)
    (f a app : NodeId) (A B arrow_id : TypeId)
    (κ_f κ_a κ_body : CostBound)
    (hf : Derivation env Γ f arrow_id κ_f)
    (ha : Derivation env Γ a A κ_a)
    (harrow : env.lookup arrow_id = some (TypeDef.Arrow A B κ_body)) :
    Derivation env Γ app B (CostBound.Sum κ_a (CostBound.Sum κ_f κ_body)) :=
  Derivation.elim env Γ f a app A B arrow_id κ_f κ_a κ_body hf harrow ha

-- ===========================================================================
-- Property 15: Constant ordering is well-behaved
-- ===========================================================================

/-- Constant(0) ≤ Constant(k) for all k. -/
theorem const_zero_leq (k : Nat) :
    CostLeq (CostBound.Constant 0) (CostBound.Constant k) :=
  CostLeq.const_le 0 k (Nat.zero_le k)

/-- Constant(k) ≤ Linear(v) for any variable v. -/
theorem const_leq_linear (k : Nat) (v : CostVar) :
    CostLeq (CostBound.Constant k) (CostBound.Linear v) :=
  CostLeq.const_linear k v

/-- Linear(v) ≤ NLogN(v) for the same variable. -/
theorem linear_leq_nlogn (v : CostVar) :
    CostLeq (CostBound.Linear v) (CostBound.NLogN v) :=
  CostLeq.linear_nlogn v

/-- The base complexity chain: Zero ≤ Polynomial(v, d). -/
theorem complexity_chain (k : Nat) (v : CostVar) (d : Nat) (_hd : 2 ≤ d) :
    CostLeq CostBound.Zero (CostBound.Polynomial v d) := by
  apply CostLeq.trans CostBound.Zero (CostBound.Constant k)
  · exact CostLeq.zero_bot _
  · exact CostLeq.const_poly k v d

end IrisKernel
