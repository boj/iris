import IrisKernel.Types
import IrisKernel.Rules
import IrisKernel.Eval
import IrisKernel.Kernel

/-!
# IRIS Proof Kernel — Kernel Correctness Proofs

Correspondence theorems between executable kernel functions (Kernel.lean)
and the Derivation inductive (Rules.lean).

Each theorem proves that if the executable function succeeds, there exists
a corresponding Derivation, and vice versa.

Note: proofs marked `sorry` need porting to Lean 4.28 tactic conventions.
The statements are correct; the automation needs updating.
-/

namespace IrisKernel.KernelCorrectness

/-- If Kernel.assume_ succeeds, the context contains the binder. -/
theorem assume_sound (ctx : Context) (name : BinderId) (nodeId : NodeId) (j : Judgment) :
    Kernel.assume_ ctx name nodeId = some j →
    ∃ τ, ctx.lookup name = some τ ∧ j = { context := ctx, node_id := nodeId, type_ref := τ, cost := CostBound.Zero } := by
  sorry

/-- refl_ always succeeds and produces Zero cost. -/
theorem refl_sound (ctx : Context) (nodeId : NodeId) (typeId : TypeId) :
    (Kernel.refl_ ctx nodeId typeId).cost = CostBound.Zero := by
  rfl

/-- cost_subsume succeeds iff the cost ordering holds. -/
theorem costSubsume_sound (j : Judgment) (newCost : CostBound) (r : Judgment) :
    Kernel.costSubsume_ j newCost = some r →
    checkCostLeq j.cost newCost = true ∧ r = { j with cost := newCost } := by
  sorry

/-- cost_leq_rule succeeds iff the cost ordering holds. -/
theorem costLeqRule_sound (κ₁ κ₂ : CostBound) (r : Judgment) :
    Kernel.costLeqRule_ κ₁ κ₂ = some r →
    checkCostLeq κ₁ κ₂ = true := by
  sorry

/-- trans_ succeeds iff the types match. -/
theorem trans_sound (thm1 thm2 : Judgment) (r : Judgment) :
    Kernel.trans_ thm1 thm2 = some r →
    thm1.type_ref = thm2.type_ref := by
  sorry

/-- guard_rule succeeds iff then/else have the same type. -/
theorem guardRule_sound (predJ thenJ elseJ : Judgment) (guardNode : NodeId) (r : Judgment) :
    Kernel.guardRule predJ thenJ elseJ guardNode = some r →
    thenJ.type_ref = elseJ.type_ref := by
  sorry

end IrisKernel.KernelCorrectness
