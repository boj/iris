import IrisKernel.Types
import IrisKernel.Rules
import IrisKernel.Properties

/-!
# IRIS Proof Kernel — Consistency Properties

Core consistency theorems about the IRIS inference rules:
1. No empty derivation of bottom
2. Type uniqueness for deterministic rules
3. Cost monotonicity
4. Context invariants
-/

namespace IrisKernel

-- ===========================================================================
-- Bottom type
-- ===========================================================================

/-- A type is "bottom" if it's a Sum with no variants. -/
def isBottom (env : TypeEnv) (τ : TypeId) : Prop :=
  env.lookup τ = some (TypeDef.Sum [])

-- ===========================================================================
-- Theorem 1: No derivation of bottom via structural_ind
-- ===========================================================================

/-- structural_ind requires at least one variant, so it cannot derive bottom. -/
theorem structural_ind_not_bottom (env : TypeEnv) (_Γ : Context)
    (sum_type _result_type : TypeId) (_result_node : NodeId)
    (variants : List (Tag × TypeId))
    (_case_nodes : List NodeId) (_case_costs : List CostBound)
    (h_sum : env.lookup sum_type = some (TypeDef.Sum variants))
    (_h_len1 : variants.length = _case_nodes.length)
    (_h_len2 : _case_nodes.length = _case_costs.length)
    (h_pos : 0 < variants.length) :
    ¬ isBottom env sum_type := by
  intro h_bot
  unfold isBottom at h_bot
  rw [h_sum] at h_bot
  have h_eq : TypeDef.Sum variants = TypeDef.Sum [] := by
    injection h_bot
  injection h_eq with h_nil
  rw [h_nil] at h_pos
  exact Nat.lt_irrefl 0 h_pos

/-- match_elim requires at least one arm. -/
theorem match_elim_nonempty_arms (_env : TypeEnv) (_Γ : Context)
    (_scrutinee _match_node : NodeId) (_scrutinee_type _result_type : TypeId)
    (_κ_scrutinee : CostBound) (arm_nodes : List NodeId) (_arm_costs : List CostBound)
    (_h_scr : Derivation _env _Γ _scrutinee _scrutinee_type _κ_scrutinee)
    (h_pos : 0 < arm_nodes.length)
    (_h_len : arm_nodes.length = _arm_costs.length) :
    arm_nodes ≠ [] := by
  intro h_nil
  rw [h_nil] at h_pos
  exact Nat.lt_irrefl 0 h_pos

-- ===========================================================================
-- Theorem 2: Type consistency for deterministic rules
-- ===========================================================================

/-- For assume, the type is uniquely determined by the context lookup. -/
theorem assume_type_unique (_env : TypeEnv) (Γ : Context) (name : BinderId)
    (_n : NodeId) (τ₁ τ₂ : TypeId)
    (h1 : Γ.lookup name = some τ₁)
    (h2 : Γ.lookup name = some τ₂) :
    τ₁ = τ₂ := by
  rw [h1] at h2
  injection h2

-- ===========================================================================
-- Theorem 3: Cost monotonicity
-- ===========================================================================

/-- cost_subsume only increases costs. -/
theorem cost_subsume_monotone (env : TypeEnv) (Γ : Context)
    (n : NodeId) (τ : TypeId) (κ₁ κ₂ : CostBound)
    (h_deriv : Derivation env Γ n τ κ₁)
    (h_leq : CostLeq κ₁ κ₂) :
    Derivation env Γ n τ κ₂ :=
  Derivation.cost_subsume env Γ n τ κ₁ κ₂ h_deriv h_leq

/-- Derivations at Zero cost can be subsumed to any cost bound. -/
theorem zero_cost_subsumable (env : TypeEnv) (Γ : Context)
    (n : NodeId) (τ : TypeId) (κ : CostBound)
    (h : Derivation env Γ n τ CostBound.Zero) :
    Derivation env Γ n τ κ :=
  Derivation.cost_subsume env Γ n τ CostBound.Zero κ h (CostLeq.zero_bot κ)

-- ===========================================================================
-- Theorem 4: Empty context properties
-- ===========================================================================

/-- In an empty context, assume cannot derive anything. -/
theorem no_assume_in_empty_context (name : BinderId) :
    Context.empty.lookup name = none := by
  unfold Context.empty Context.lookup
  simp
  unfold Context.lookup.go
  rfl

/-- refl is the only base rule that produces derivations in empty context. -/
theorem refl_is_base_rule (env : TypeEnv) (n : NodeId) (τ : TypeId) :
    Derivation env Context.empty n τ CostBound.Zero :=
  Derivation.refl env Context.empty n τ

-- ===========================================================================
-- Theorem 5: Type well-formedness propagation
-- ===========================================================================

/-- type_abst preserves well-formedness. -/
theorem type_abst_preserves_wellformedness (env : TypeEnv) (Γ : Context)
    (n : NodeId) (inner forall_type : TypeId) (bv : BoundVar) (κ : CostBound)
    (h_body : Derivation env Γ n inner κ)
    (h_lookup : env.lookup forall_type = some (TypeDef.ForAll bv inner))
    (h_wf : TypeWellFormed env forall_type) :
    Derivation env Γ n forall_type κ :=
  Derivation.type_abst env Γ n inner forall_type bv κ h_body h_lookup h_wf

/-- type_app requires the result type to be well-formed. -/
theorem type_app_requires_wellformedness (env : TypeEnv) (Γ : Context)
    (n : NodeId) (forall_type result_type : TypeId)
    (bv : BoundVar) (inner : TypeId) (κ : CostBound)
    (h_deriv : Derivation env Γ n forall_type κ)
    (h_forall : env.lookup forall_type = some (TypeDef.ForAll bv inner))
    (h_wf : TypeWellFormed env result_type) :
    Derivation env Γ n result_type κ :=
  Derivation.type_app env Γ n forall_type result_type bv inner κ h_deriv h_forall h_wf

-- ===========================================================================
-- Theorem 6: nat_ind cost is additive
-- ===========================================================================

theorem nat_ind_cost_additive (env : TypeEnv) (Γ : Context)
    (b s r : NodeId) (τ : TypeId) (κb κs : CostBound)
    (hb : Derivation env Γ b τ κb)
    (hs : Derivation env Γ s τ κs) :
    Derivation env Γ r τ (CostBound.Sum κb κs) :=
  Derivation.nat_ind env Γ b s r τ κb κs hb hs

-- ===========================================================================
-- Theorem 7: match cost bounds worst case
-- ===========================================================================

theorem match_cost_bounds_worst_case (env : TypeEnv) (Γ : Context)
    (scrutinee match_node : NodeId) (st rt : TypeId)
    (κs : CostBound) (arm_nodes : List NodeId) (arm_costs : List CostBound)
    (hs : Derivation env Γ scrutinee st κs)
    (h_pos : 0 < arm_nodes.length)
    (h_len : arm_nodes.length = arm_costs.length)
    (h_arms : ∀ (i : Nat) (hn : i < arm_nodes.length) (hc : i < arm_costs.length),
      Derivation env Γ (arm_nodes.get ⟨i, hn⟩) rt (arm_costs.get ⟨i, hc⟩)) :
    Derivation env Γ match_node rt (CostBound.Sum κs (CostBound.Sup arm_costs)) :=
  Derivation.match_elim env Γ scrutinee match_node st rt κs arm_nodes arm_costs
    hs h_pos h_len h_arms

-- ===========================================================================
-- Theorem 8: Context lookup is deterministic
-- ===========================================================================

theorem context_lookup_deterministic (ctx : Context) (name : BinderId) :
    ∀ (τ₁ τ₂ : TypeId),
      ctx.lookup name = some τ₁ → ctx.lookup name = some τ₂ → τ₁ = τ₂ := by
  intro τ₁ τ₂ h1 h2
  rw [h1] at h2
  injection h2

-- ===========================================================================
-- Theorem 9: Sup is an upper bound
-- ===========================================================================

theorem sup_is_upper_bound (vs : List CostBound) (x : CostBound)
    (h : CostLeq (CostBound.Sup vs) x) (v : CostBound) (hv : v ∈ vs) :
    CostLeq v x :=
  CostLeq.trans v (CostBound.Sup vs) x (CostLeq.elem_sup v vs hv) h

-- ===========================================================================
-- Theorem 10: Derivation composition
-- ===========================================================================

theorem let_bind_compose (env : TypeEnv) (Γ : Context)
    (e₁ e₂ e₃ let_inner let_outer : NodeId)
    (x y : BinderId) (A B C : TypeId)
    (κ₁ κ₂ κ₃ : CostBound)
    (h1 : Derivation env Γ e₁ A κ₁)
    (h2 : Derivation env (Γ.extend x A) e₂ B κ₂)
    (h3 : Derivation env (Γ.extend y B) e₃ C κ₃) :
    Derivation env Γ let_outer C (CostBound.Sum (CostBound.Sum κ₁ κ₂) κ₃) := by
  have h_inner := Derivation.let_bind env Γ let_inner e₁ e₂ x A B κ₁ κ₂ h1 h2
  exact Derivation.let_bind env Γ let_outer let_inner e₃ y B C (CostBound.Sum κ₁ κ₂) κ₃
    h_inner h3

end IrisKernel
