import IrisKernel.Types
import IrisKernel.Eval

/-!
# IRIS Proof Kernel — Executable Inference Rules

Executable `def` versions of all 20 inference rules from
`src/iris-kernel/src/kernel.rs`. Each function returns `Option Judgment`
and checks preconditions computationally.

These functions take Judgment inputs (not Theorem — proof hashing stays
in Rust). The Rust side wraps the result in a Theorem with proof_hash
computed via BLAKE3.
-/

namespace IrisKernel.Kernel

-- ===========================================================================
-- 1. assume: Γ, x:A ⊢ x : A @ Zero
-- ===========================================================================

/-- Assumption rule: if binder `name` has type `τ` in context `ctx`,
    produce `ctx ⊢ nodeId : τ @ Zero`. -/
def assume_ (ctx : Context) (name : BinderId) (nodeId : NodeId) : Option Judgment :=
  match ctx.lookup name with
  | some τ => some { context := ctx, node_id := nodeId, type_ref := τ, cost := CostBound.Zero }
  | none => none

-- ===========================================================================
-- 2. intro: if Γ,x:A ⊢ body : B @ κ then Γ ⊢ λ : Arrow(A,B,κ) @ Zero
-- ===========================================================================

/-- Arrow introduction. Given a body judgment in the extended context and
    the arrow type id that must exist as Arrow(A, B, κ_body) in the env,
    produce a judgment for the lambda at arrow type with zero cost. -/
def intro (env : TypeEnv) (ctx : Context) (lamNode : NodeId)
    (binderName : BinderId) (binderType : TypeId)
    (bodyJudgment : Judgment) (arrowId : TypeId) : Option Judgment := do
  -- Check the body's context is ctx extended with the binder
  let extended := ctx.extend binderName binderType
  if bodyJudgment.context != extended then none
  else do
    -- Check the arrow type exists and matches
    let arrowDef ← env.lookup arrowId
    match arrowDef with
    | TypeDef.Arrow paramTy retTy bodyCost =>
      if paramTy != binderType then none
      else if retTy != bodyJudgment.type_ref then none
      else if !(costBoundBEq bodyCost bodyJudgment.cost) then none
      else some { context := ctx, node_id := lamNode, type_ref := arrowId, cost := CostBound.Zero }
    | _ => none

-- ===========================================================================
-- 3. elim: function application / modus ponens
-- ===========================================================================

/-- Arrow elimination. Given fn and arg judgments, produce application
    judgment: Γ ⊢ app : B @ Sum(κ_a, Sum(κ_f, κ_body)). -/
def elim (env : TypeEnv) (fnJ argJ : Judgment) (appNode : NodeId) : Option Judgment := do
  -- Contexts must match
  if fnJ.context != argJ.context then none
  else do
    -- fn must have an Arrow type
    let fnTypeDef ← env.lookup fnJ.type_ref
    match fnTypeDef with
    | TypeDef.Arrow paramTy retTy bodyCost =>
      -- Arg type must match param type
      if argJ.type_ref != paramTy then none
      else
        let totalCost := CostBound.Sum argJ.cost (CostBound.Sum fnJ.cost bodyCost)
        some { context := fnJ.context, node_id := appNode, type_ref := retTy, cost := totalCost }
    | _ => none

-- ===========================================================================
-- 4. refl: reflexivity of equality
-- ===========================================================================

/-- Reflexivity: any node equals itself at zero cost. -/
def refl_ (ctx : Context) (nodeId : NodeId) (typeId : TypeId) : Judgment :=
  { context := ctx, node_id := nodeId, type_ref := typeId, cost := CostBound.Zero }

-- ===========================================================================
-- 5. symm: symmetry of equality
-- ===========================================================================

/-- Symmetry: given a judgment about node `a` and an equality witness about
    node `other` with same type, produce a judgment about `other`. -/
def symm_ (thm : Judgment) (otherNode : NodeId) (eqWitness : Judgment) : Option Judgment :=
  -- Equality witness must be about the target node
  if eqWitness.node_id != otherNode then none
  -- Equality witness must have the same type
  else if eqWitness.type_ref != thm.type_ref then none
  else some { context := thm.context, node_id := otherNode, type_ref := thm.type_ref, cost := thm.cost }

-- ===========================================================================
-- 6. trans: transitivity of equality
-- ===========================================================================

/-- Transitivity: chain two judgments with the same type.
    Result keeps node of first, cost of second. -/
def trans_ (thm1 thm2 : Judgment) : Option Judgment :=
  if thm1.type_ref != thm2.type_ref then none
  else some { context := thm1.context, node_id := thm1.node_id, type_ref := thm2.type_ref, cost := thm2.cost }

-- ===========================================================================
-- 7. congr: congruence
-- ===========================================================================

/-- Congruence: if function and argument are equal, application is equal.
    Result type from fn, cost is Sum(κ_f, κ_a). -/
def congr_ (fnJ argJ : Judgment) (appNode : NodeId) : Option Judgment :=
  if fnJ.context != argJ.context then none
  else
    let totalCost := CostBound.Sum fnJ.cost argJ.cost
    some { context := fnJ.context, node_id := appNode, type_ref := fnJ.type_ref, cost := totalCost }

-- ===========================================================================
-- 8. type_check_node: type-check a node against its annotation
-- ===========================================================================

/-- Type-check a node: verify the type is well-formed and assign cost
    based on node kind. -/
def typeCheckNode_ (env : TypeEnv) (ctx : Context) (nodeId : NodeId)
    (kind : NodeKind) (typeSig : TypeId) : Option Judgment :=
  if checkTypeWellFormed env typeSig then
    let cost := match kind with
      | NodeKind.Lit     => CostBound.Zero
      | NodeKind.Ref     => CostBound.Zero
      | NodeKind.Lambda  => CostBound.Zero
      | NodeKind.Prim    => CostBound.Constant 1
      | NodeKind.Tuple   => CostBound.Zero
      | NodeKind.Project => CostBound.Constant 1
      | NodeKind.Inject  => CostBound.Zero
      | NodeKind.TypeAbst => CostBound.Zero
      | NodeKind.TypeApp  => CostBound.Zero
      | _                => CostBound.Unknown
    some { context := ctx, node_id := nodeId, type_ref := typeSig, cost := cost }
  else
    none

-- ===========================================================================
-- 9. cost_subsume: weaken a cost bound
-- ===========================================================================

/-- Cost subsumption: if j proves e : T @ κ₁ and κ₁ ≤ κ₂,
    produce e : T @ κ₂. -/
def costSubsume_ (j : Judgment) (newCost : CostBound) : Option Judgment :=
  if checkCostLeq j.cost newCost then
    some { j with cost := newCost }
  else
    none

-- ===========================================================================
-- 10. cost_leq_rule: verify cost ordering
-- ===========================================================================

/-- Cost ordering witness: verify κ₁ ≤ κ₂ and produce a witness judgment. -/
def costLeqRule_ (κ₁ κ₂ : CostBound) : Option Judgment :=
  if checkCostLeq κ₁ κ₂ then
    some { context := Context.empty, node_id := ⟨0⟩, type_ref := ⟨0⟩, cost := κ₂ }
  else
    none

-- ===========================================================================
-- 11. refine_intro: refinement type introduction
-- ===========================================================================

/-- Refinement introduction: given base judgment and predicate witness,
    produce judgment at refined type. -/
def refineIntro (env : TypeEnv) (baseJ predJ : Judgment) (refinedTypeId : TypeId) : Option Judgment := do
  let refinedDef ← env.lookup refinedTypeId
  match refinedDef with
  | TypeDef.Refined innerType =>
    -- Base type must match
    if baseJ.type_ref != innerType then none
    -- Predicate witness must be about the same node
    else if predJ.node_id != baseJ.node_id then none
    else some { context := baseJ.context, node_id := baseJ.node_id, type_ref := refinedTypeId, cost := baseJ.cost }
  | _ => none

-- ===========================================================================
-- 12. refine_elim: refinement type elimination
-- ===========================================================================

/-- Refinement elimination: extract base type from refined type. -/
def refineElim (env : TypeEnv) (j : Judgment) : Option Judgment := do
  let typeDef ← env.lookup j.type_ref
  match typeDef with
  | TypeDef.Refined innerType =>
    some { j with type_ref := innerType }
  | _ => none

-- ===========================================================================
-- 13. nat_ind: natural number induction
-- ===========================================================================

/-- Natural number induction: base + step ⟹ ∀n.P(n).
    Cost is Sum(κ_base, κ_step). -/
def natInd (baseJ stepJ : Judgment) (resultNode : NodeId) : Option Judgment :=
  -- Both must have the same result type
  if baseJ.type_ref != stepJ.type_ref then none
  else
    let cost := CostBound.Sum baseJ.cost stepJ.cost
    some { context := baseJ.context, node_id := resultNode, type_ref := baseJ.type_ref, cost := cost }

-- ===========================================================================
-- 14. structural_ind: structural induction over an ADT
-- ===========================================================================

/-- Structural induction over a sum type.
    All cases must produce the same result type.
    Cost is Sup of all case costs. -/
def structuralInd (env : TypeEnv) (sumType : TypeId) (cases : List Judgment)
    (resultNode : NodeId) : Option Judgment := do
  -- Sum type must exist
  let sumDef ← env.lookup sumType
  match sumDef with
  | TypeDef.Sum variants =>
    -- Must have exactly one case per variant
    if cases.length != variants.length then none
    else if cases.isEmpty then none
    else do
      -- All cases must have the same result type
      let resultType := cases.head!.type_ref
      if cases.any (fun c => c.type_ref != resultType) then none
      else
        let caseCosts := cases.map (fun c => c.cost)
        let cost := CostBound.Sup caseCosts
        some { context := cases.head!.context, node_id := resultNode, type_ref := resultType, cost := cost }
  | _ => none

-- ===========================================================================
-- 15. let_bind: let binding (cut rule)
-- ===========================================================================

/-- Let binding: Γ ⊢ e₁:A @ κ₁, Γ,x:A ⊢ e₂:B @ κ₂
    ⟹ Γ ⊢ let_node : B @ Sum(κ₁, κ₂). -/
def letBind (ctx : Context) (letNode : NodeId) (binderName : BinderId)
    (boundJ bodyJ : Judgment) : Option Judgment :=
  -- bound must be in ctx
  if boundJ.context != ctx then none
  else
    -- body must be in ctx extended with the binder
    let extended := ctx.extend binderName boundJ.type_ref
    if bodyJ.context != extended then none
    else
      let cost := CostBound.Sum boundJ.cost bodyJ.cost
      some { context := ctx, node_id := letNode, type_ref := bodyJ.type_ref, cost := cost }

-- ===========================================================================
-- 16. match_elim: pattern matching / sum elimination
-- ===========================================================================

/-- Match elimination: scrutinee + arms ⟹ match node.
    All arms must produce the same result type.
    Cost is Sum(κ_scrutinee, Sup(arm costs)). -/
def matchElim (scrutineeJ : Judgment) (armJs : List Judgment)
    (matchNode : NodeId) : Option Judgment :=
  if armJs.isEmpty then none
  else
    let resultType := armJs.head!.type_ref
    if armJs.any (fun a => a.type_ref != resultType) then none
    else
      let armCosts := armJs.map (fun a => a.cost)
      let cost := CostBound.Sum scrutineeJ.cost (CostBound.Sup armCosts)
      some { context := scrutineeJ.context, node_id := matchNode, type_ref := resultType, cost := cost }

-- ===========================================================================
-- 17. fold_rule: structural recursion (catamorphism)
-- ===========================================================================

/-- Fold rule: base + step + input ⟹ fold node.
    Cost: Sum(κ_input, Sum(κ_base, Mul(κ_step, κ_input))). -/
def foldRule (baseJ stepJ inputJ : Judgment) (foldNode : NodeId) : Option Judgment :=
  let resultType := baseJ.type_ref
  let cost := CostBound.Sum inputJ.cost (CostBound.Sum baseJ.cost (CostBound.Mul stepJ.cost inputJ.cost))
  some { context := baseJ.context, node_id := foldNode, type_ref := resultType, cost := cost }

-- ===========================================================================
-- 18. type_abst: type abstraction (ForAll introduction)
-- ===========================================================================

/-- Type abstraction: if body has type `inner`, produce ForAll(bv, inner). -/
def typeAbst (env : TypeEnv) (bodyJ : Judgment) (forallTypeId : TypeId) : Option Judgment := do
  -- Check well-formedness
  if !checkTypeWellFormed env forallTypeId then none
  else do
    let typeDef ← env.lookup forallTypeId
    match typeDef with
    | TypeDef.ForAll _bv inner =>
      if inner != bodyJ.type_ref then none
      else some { bodyJ with type_ref := forallTypeId }
    | _ => none

-- ===========================================================================
-- 19. type_app: type application (ForAll elimination)
-- ===========================================================================

/-- Type application: if thm proves ForAll(X, T), produce T[S/X].
    Result type must be well-formed. -/
def typeApp (env : TypeEnv) (j : Judgment) (resultTypeId : TypeId) : Option Judgment := do
  let typeDef ← env.lookup j.type_ref
  match typeDef with
  | TypeDef.ForAll _ _ =>
    if !checkTypeWellFormed env resultTypeId then none
    else some { j with type_ref := resultTypeId }
  | _ => none

-- ===========================================================================
-- 20. guard_rule: conditional / if-then-else
-- ===========================================================================

/-- Guard rule: pred + then + else ⟹ guard node.
    Then and else must have the same result type.
    Cost: Sum(κ_pred, Sup([κ_then, κ_else])). -/
def guardRule (predJ thenJ elseJ : Judgment) (guardNode : NodeId) : Option Judgment :=
  if thenJ.type_ref != elseJ.type_ref then none
  else
    let cost := CostBound.Sum predJ.cost (CostBound.Sup [thenJ.cost, elseJ.cost])
    some { context := predJ.context, node_id := guardNode, type_ref := thenJ.type_ref, cost := cost }

end IrisKernel.Kernel
