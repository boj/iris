import IrisKernel.Types

/-!
# IRIS Proof Kernel — Executable Evaluation Functions

Computable (`def`) versions of the key kernel decision procedures.
These are the functions that will be compiled and exported via FFI
so the running code IS the formal proof.

Every function here mirrors a Rust function in the kernel's TCB:
- `checkCostLeq`   ↔ `cost_checker::cost_leq`
- `evalLIAFormula`  ↔ `lia_solver::evaluate_lia`
- `typeCheckNode`   ↔ (the `type_check_node` rule's well-formedness check)
-/

namespace IrisKernel

-- ===========================================================================
-- CostBound comparison — executable decision procedure
-- ===========================================================================

/-- Check if a CostBound is "at least linear" in the growth hierarchy.
    Mirrors `is_at_least_linear` in `src/iris-kernel/src/cost_checker.rs`. -/
def isAtLeastLinear : CostBound → Bool
  | CostBound.Linear _       => true
  | CostBound.NLogN _        => true
  | CostBound.Polynomial _ _ => true
  | _                        => false

/-- BEq instance for CostVar. -/
instance : BEq CostVar := ⟨fun a b => a.val == b.val⟩

-- We need structural equality on CostBound for the checker.
-- costBoundBEq and costBoundListBEq are mutually recursive because
-- Sup/Inf contain List CostBound.
mutual
  def costBoundBEq : CostBound → CostBound → Bool
    | CostBound.Unknown, CostBound.Unknown => true
    | CostBound.Zero, CostBound.Zero => true
    | CostBound.Constant k1, CostBound.Constant k2 => k1 == k2
    | CostBound.Linear v1, CostBound.Linear v2 => v1 == v2
    | CostBound.NLogN v1, CostBound.NLogN v2 => v1 == v2
    | CostBound.Polynomial v1 d1, CostBound.Polynomial v2 d2 => v1 == v2 && d1 == d2
    | CostBound.Sum a1 a2, CostBound.Sum b1 b2 => costBoundBEq a1 b1 && costBoundBEq a2 b2
    | CostBound.Par a1 a2, CostBound.Par b1 b2 => costBoundBEq a1 b1 && costBoundBEq a2 b2
    | CostBound.Mul a1 a2, CostBound.Mul b1 b2 => costBoundBEq a1 b1 && costBoundBEq a2 b2
    | CostBound.Sup vs1, CostBound.Sup vs2 => costBoundListBEq vs1 vs2
    | CostBound.Inf vs1, CostBound.Inf vs2 => costBoundListBEq vs1 vs2
    | CostBound.Amortized a, CostBound.Amortized b => costBoundBEq a b
    | CostBound.HWScaled a, CostBound.HWScaled b => costBoundBEq a b
    | _, _ => false

  def costBoundListBEq : List CostBound → List CostBound → Bool
    | [], [] => true
    | x :: xs, y :: ys => costBoundBEq x y && costBoundListBEq xs ys
    | _, _ => false
end

instance : BEq CostBound := ⟨costBoundBEq⟩

/-- Check if `a ≤ b` in the cost partial order.

    This is a conservative decision procedure: if we cannot determine
    the relationship, we return `false` (reject rather than accept
    uncertain orderings).

    Mirrors `cost_leq` in `src/iris-kernel/src/cost_checker.rs` exactly. -/
partial def checkCostLeq (a b : CostBound) : Bool :=
  -- Identical costs are always ordered.
  if costBoundBEq a b then true
  -- Unknown absorbs everything on the right.
  else match b with
  | CostBound.Unknown => true
  | _ =>
    -- Unknown on the left: only ≤ Unknown (handled by equality above).
    match a with
    | CostBound.Unknown => false
    | _ => checkCostLeqCore a b

where
  /-- Core comparison after handling Unknown and equality. -/
  checkCostLeqCore (a b : CostBound) : Bool :=
    match a, b with
    -- Zero is bottom
    | CostBound.Zero, _ => true

    -- Constant ordering
    | CostBound.Constant _, CostBound.Zero => false
    | CostBound.Constant k1, CostBound.Constant k2 => k1 ≤ k2
    | CostBound.Constant _, _ => isAtLeastLinear b

    -- Linear ordering
    | CostBound.Linear _, CostBound.Zero => false
    | CostBound.Linear _, CostBound.Constant _ => false
    | CostBound.Linear v1, CostBound.Linear v2 => v1 == v2
    | CostBound.Linear v1, CostBound.NLogN v2 => v1 == v2
    | CostBound.Linear v1, CostBound.Polynomial v2 _ => v1 == v2

    -- NLogN ordering
    | CostBound.NLogN _, CostBound.Zero => false
    | CostBound.NLogN _, CostBound.Constant _ => false
    | CostBound.NLogN _, CostBound.Linear _ => false
    | CostBound.NLogN v1, CostBound.NLogN v2 => v1 == v2
    | CostBound.NLogN v1, CostBound.Polynomial v2 d => v1 == v2 && d ≥ 2

    -- Polynomial ordering
    | CostBound.Polynomial .., CostBound.Zero => false
    | CostBound.Polynomial .., CostBound.Constant _ => false
    | CostBound.Polynomial .., CostBound.Linear _ => false
    | CostBound.Polynomial .., CostBound.NLogN _ => false
    | CostBound.Polynomial v1 d1, CostBound.Polynomial v2 d2 => v1 == v2 && d1 ≤ d2

    -- Composite: pointwise comparison
    | CostBound.Sum a1 a2, CostBound.Sum b1 b2 =>
      checkCostLeq a1 b1 && checkCostLeq a2 b2
    | CostBound.Par a1 a2, CostBound.Par b1 b2 =>
      checkCostLeq a1 b1 && checkCostLeq a2 b2
    | CostBound.Mul a1 a2, CostBound.Mul b1 b2 =>
      checkCostLeq a1 b1 && checkCostLeq a2 b2

    -- Sup on left: all branches must be ≤ b
    | CostBound.Sup branches, _ => branches.all (checkCostLeq · b)

    -- Sup on right: a ≤ at least one branch
    | _, CostBound.Sup branches => branches.any (checkCostLeq a ·)

    -- Inf on right: a ≤ all branches
    | _, CostBound.Inf branches => branches.all (checkCostLeq a ·)

    -- Inf on left: any branch ≤ b
    | CostBound.Inf branches, _ => branches.any (checkCostLeq · b)

    -- Amortized: conservative — compare via inner cost
    | CostBound.Amortized inner, _ => checkCostLeq inner b
    | _, CostBound.Amortized inner => checkCostLeq a inner

    -- HWScaled: conservative — compare via inner cost
    | CostBound.HWScaled inner, _ => checkCostLeq inner b
    | _, CostBound.HWScaled inner => checkCostLeq a inner

    -- Lift simple costs into Sum when one component is Zero
    | _, CostBound.Sum b1 b2 =>
      (costBoundBEq b2 CostBound.Zero && checkCostLeq a b1) ||
      (costBoundBEq b1 CostBound.Zero && checkCostLeq a b2)
    | _, CostBound.Par b1 b2 =>
      (costBoundBEq b2 CostBound.Zero && checkCostLeq a b1) ||
      (costBoundBEq b1 CostBound.Zero && checkCostLeq a b2)

    -- Cannot determine ordering
    | _, _ => false

-- ===========================================================================
-- LIA types — mutually recursive inductives
-- ===========================================================================

mutual
  /-- LIA term (Linear Integer Arithmetic).
      Mirrors `LIATerm` in `src/iris-repr/src/types.rs`.
      Mutually defined with LIAFormula because IfThenElse contains a formula
      as its condition. -/
  inductive LIATerm : Type where
    | Var       : Nat → LIATerm
    | Const     : Int → LIATerm
    | Add       : LIATerm → LIATerm → LIATerm
    | Mul       : Int → LIATerm → LIATerm
    | Neg       : LIATerm → LIATerm
    | Len       : Nat → LIATerm
    | Size      : Nat → LIATerm
    | IfThenElse : LIAFormula → LIATerm → LIATerm → LIATerm
    | Mod       : LIATerm → LIATerm → LIATerm

  /-- LIA formula (quantifier-free).
      Mirrors `LIAFormula` in `src/iris-repr/src/types.rs`. -/
  inductive LIAFormula : Type where
    | True    : LIAFormula
    | False   : LIAFormula
    | And     : LIAFormula → LIAFormula → LIAFormula
    | Or      : LIAFormula → LIAFormula → LIAFormula
    | Not     : LIAFormula → LIAFormula
    | Implies : LIAFormula → LIAFormula → LIAFormula
    | Atom    : LIAAtom → LIAFormula

  /-- LIA atom (atomic predicate). -/
  inductive LIAAtom : Type where
    | Eq        : LIATerm → LIATerm → LIAAtom
    | Lt        : LIATerm → LIATerm → LIAAtom
    | Le        : LIATerm → LIATerm → LIAAtom
    | Divisible : LIATerm → Nat → LIAAtom
end

/-- Variable environment for LIA evaluation: var id → value. -/
def LIAEnv := List (Nat × Int)

namespace LIAEnv

/-- Look up a variable in the environment. Returns 0 if not found
    (matching the Rust `unwrap_or(&0)` convention). -/
def lookup (env : LIAEnv) (var : Nat) : Int :=
  match env with
  | []          => 0
  | (k, v) :: rest => if k == var then v else lookup rest var

end LIAEnv

-- LIA evaluation: three mutually recursive partial functions.
mutual
  /-- Evaluate a LIA term under a variable assignment.
      Mirrors `evaluate_term` in `src/iris-kernel/src/lia_solver.rs`. -/
  partial def evalLIATerm (t : LIATerm) (env : LIAEnv) : Int :=
    match t with
    | .Var v         => env.lookup v
    | .Const c       => c
    | .Add a b       => evalLIATerm a env + evalLIATerm b env
    | .Mul c t       => c * evalLIATerm t env
    | .Neg t         => -(evalLIATerm t env)
    | .Len v         => env.lookup v
    | .Size v        => env.lookup v
    | .IfThenElse cond thenT elseT =>
      if evalLIAFormula cond env then evalLIATerm thenT env
      else evalLIATerm elseT env
    | .Mod a b       =>
      let bv := evalLIATerm b env
      if bv == 0 then 0
      else evalLIATerm a env % bv

  /-- Evaluate a LIA atom under a variable assignment. -/
  partial def evalLIAAtom (atom : LIAAtom) (env : LIAEnv) : Bool :=
    match atom with
    | .Eq a b     => evalLIATerm a env == evalLIATerm b env
    | .Lt a b     => evalLIATerm a env < evalLIATerm b env
    | .Le a b     => evalLIATerm a env ≤ evalLIATerm b env
    | .Divisible t d =>
      if d == 0 then false
      else evalLIATerm t env % (d : Int) == 0

  /-- Evaluate a quantifier-free LIA formula under a variable assignment.
      Mirrors `evaluate_lia` in `src/iris-kernel/src/lia_solver.rs`. -/
  partial def evalLIAFormula (f : LIAFormula) (env : LIAEnv) : Bool :=
    match f with
    | .True      => true
    | .False     => false
    | .And a b   => evalLIAFormula a env && evalLIAFormula b env
    | .Or a b    => evalLIAFormula a env || evalLIAFormula b env
    | .Not f     => !evalLIAFormula f env
    | .Implies a b => !evalLIAFormula a env || evalLIAFormula b env
    | .Atom atom => evalLIAAtom atom env
end

-- ===========================================================================
-- Type-checking: well-formedness check (executable)
-- ===========================================================================

/-- Check that a TypeId is well-formed in the type environment.
    Executable version of `TypeWellFormed`.

    Returns `true` iff:
    1. The TypeId has a definition in the environment
    2. All TypeIds referenced by that definition also exist

    Mirrors `assert_type_well_formed` in `src/iris-kernel/src/kernel.rs`. -/
def checkTypeWellFormed (env : TypeEnv) (id : TypeId) : Bool :=
  match env.lookup id with
  | none => false
  | some td =>
    let refs := typeDefReferences td
    refs.all (env.contains ·)

/-- Executable type-check for a single node.

    Given a node kind, a type signature (TypeId), and a type environment,
    verify the type is well-formed and return the judgment.

    This corresponds to the `type_check_node` rule (#8) in the kernel.
    The rule trusts the node's annotation but verifies that the type
    exists and is well-formed.

    Returns `some judgment` on success, `none` on failure. -/
def typeCheckNode (kind : NodeKind) (typeSig : TypeId) (env : TypeEnv)
    (ctx : Context) (nodeId : NodeId) : Option Judgment :=
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
    some {
      context  := ctx
      node_id  := nodeId
      type_ref := typeSig
      cost     := cost
    }
  else
    none

-- ===========================================================================
-- Cost-leq rule (#10) — executable
-- ===========================================================================

/-- Check cost ordering and produce a witness judgment.
    Mirrors `Kernel::cost_leq_rule` in kernel.rs. -/
def costLeqRule (κ₁ κ₂ : CostBound) : Option Judgment :=
  if checkCostLeq κ₁ κ₂ then
    some {
      context  := Context.empty
      node_id  := ⟨0⟩
      type_ref := ⟨0⟩
      cost     := κ₂
    }
  else
    none

-- ===========================================================================
-- Cost subsumption rule (#9) — executable
-- ===========================================================================

/-- Cost subsumption: weaken a cost bound on a judgment. -/
def costSubsume (j : Judgment) (κ₂ : CostBound) : Option Judgment :=
  if checkCostLeq j.cost κ₂ then
    some { j with cost := κ₂ }
  else
    none

end IrisKernel
