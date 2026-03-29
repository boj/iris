/-!
# IRIS Proof Kernel — Core Types

Lean 4 formalization of the types from:
- `src/iris-repr/src/graph.rs`     — NodeId, NodeKind
- `src/iris-repr/src/types.rs`     — TypeId, TypeDef, PrimType, BoundVar, Tag
- `src/iris-repr/src/cost.rs`      — CostBound, CostVar
- `src/iris-kernel/src/theorem.rs` — Binding, Context, Judgment

Every inductive mirrors the Rust enum it corresponds to, with the same
variant names (modulo Lean naming conventions).
-/

namespace IrisKernel

-- ===========================================================================
-- NodeId — 64-bit content-addressed node identity (BLAKE3 truncated)
-- Rust: `pub struct NodeId(pub u64);`
-- ===========================================================================

/-- Opaque 64-bit content-addressed node identity. -/
structure NodeId where
  val : Nat
  deriving DecidableEq, Repr, BEq, Hashable

instance : Inhabited NodeId := ⟨⟨0⟩⟩

-- ===========================================================================
-- BinderId — binder identifier for lambda/let-rec nodes
-- Rust: `pub struct BinderId(pub u32);`
-- ===========================================================================

/-- Binder identifier for lambda/let-rec nodes. -/
structure BinderId where
  val : Nat
  deriving DecidableEq, Repr, BEq, Hashable

-- ===========================================================================
-- TypeId — 64-bit content-addressed type identity
-- Rust: `pub struct TypeId(pub u64);`
-- ===========================================================================

/-- 64-bit content-addressed type identity. -/
structure TypeId where
  val : Nat
  deriving DecidableEq, Repr, BEq, Hashable

instance : Inhabited TypeId := ⟨⟨0⟩⟩

-- ===========================================================================
-- BoundVar — de Bruijn-style bound variable identifier
-- Rust: `pub struct BoundVar(pub u32);`
-- ===========================================================================

/-- De Bruijn-style bound variable identifier. -/
structure BoundVar where
  val : Nat
  deriving DecidableEq, Repr, BEq, Hashable

-- ===========================================================================
-- Tag — sum-type variant tag
-- Rust: `pub struct Tag(pub u16);`
-- ===========================================================================

/-- Tag for sum-type variants. -/
structure Tag where
  val : Nat
  deriving DecidableEq, Repr, BEq, Hashable

-- ===========================================================================
-- CostVar — variable in cost expressions
-- Rust: `pub struct CostVar(pub u32);`
-- ===========================================================================

/-- Variable in cost expressions. -/
structure CostVar where
  val : Nat
  deriving DecidableEq, Repr, BEq, Hashable

-- ===========================================================================
-- NodeKind — 5-bit tag, 20 active variants
-- Rust: `pub enum NodeKind { Prim, Apply, Lambda, ... }`
-- ===========================================================================

/-- The 20 node kinds in the IRIS semantic graph.
    Mirrors `NodeKind` in `src/iris-repr/src/graph.rs`. -/
inductive NodeKind : Type where
  | Prim      : NodeKind  -- 0x00
  | Apply     : NodeKind  -- 0x01
  | Lambda    : NodeKind  -- 0x02
  | Let       : NodeKind  -- 0x03
  | Match     : NodeKind  -- 0x04
  | Lit       : NodeKind  -- 0x05
  | Ref       : NodeKind  -- 0x06
  | Neural    : NodeKind  -- 0x07
  | Fold      : NodeKind  -- 0x08
  | Unfold    : NodeKind  -- 0x09
  | Effect    : NodeKind  -- 0x0A
  | Tuple     : NodeKind  -- 0x0B
  | Inject    : NodeKind  -- 0x0C
  | Project   : NodeKind  -- 0x0D
  | TypeAbst  : NodeKind  -- 0x0E
  | TypeApp   : NodeKind  -- 0x0F
  | LetRec    : NodeKind  -- 0x10
  | Guard     : NodeKind  -- 0x11
  | Rewrite   : NodeKind  -- 0x12
  | Extern    : NodeKind  -- 0x13
  deriving DecidableEq, Repr

-- ===========================================================================
-- CostBound — 13 variants (omitting HWScaled for the pure formalization)
-- Rust: `pub enum CostBound { Unknown, Zero, Constant(u64), ... }`
--
-- ===========================================================================

/-- Cost bounds for resource usage analysis.
    Mirrors `CostBound` in `src/iris-repr/src/cost.rs`.

    `Amortized` wraps an inner cost bound (the potential function is opaque
    and not relevant to the metatheory — only the inner bound matters for
    ordering).

    `HWScaled` wraps an inner cost bound (the hardware profile ref is opaque
    and not relevant to the metatheory — only the inner bound matters for
    ordering). -/
inductive CostBound : Type where
  | Unknown    : CostBound
  | Zero       : CostBound
  | Constant   : Nat → CostBound
  | Linear     : CostVar → CostBound
  | NLogN      : CostVar → CostBound
  | Polynomial : CostVar → Nat → CostBound
  | Sum        : CostBound → CostBound → CostBound
  | Par        : CostBound → CostBound → CostBound
  | Mul        : CostBound → CostBound → CostBound
  | Sup        : List CostBound → CostBound
  | Inf        : List CostBound → CostBound
  | Amortized  : CostBound → CostBound        -- inner bound (potential fn is opaque)
  | HWScaled   : CostBound → CostBound        -- inner bound (hw profile ref is opaque)

-- DecidableEq and Repr cannot be auto-derived for CostBound because it
-- contains List CostBound (nested inductive). We provide BEq in Eval.lean
-- and use the inductive CostLeq relation for proofs instead.

-- ===========================================================================
-- PrimType — primitive type tags
-- Rust: `pub enum PrimType { Int, Nat, Float64, Float32, Bool, Bytes, Unit }`
-- ===========================================================================

/-- Primitive type tags. Mirrors `PrimType` in `src/iris-repr/src/types.rs`. -/
inductive PrimType : Type where
  | Int     : PrimType
  | Nat     : PrimType
  | Float64 : PrimType
  | Float32 : PrimType
  | Bool    : PrimType
  | Bytes   : PrimType
  | Unit    : PrimType
  deriving DecidableEq, Repr

-- ===========================================================================
-- TypeDef — 11 variants (type definitions in the TypeEnv)
-- Rust: `pub enum TypeDef { Primitive, Product, Sum, ... }`
--
-- We simplify NeuralGuard and HWParam by omitting opaque runtime data.
-- The formalization retains their type structure.
-- ===========================================================================

/-- Type definitions in the IRIS type environment.
    Mirrors `TypeDef` in `src/iris-repr/src/types.rs`.

    Simplifications from the Rust version:
    - `NeuralGuard`: omits `GuardSpec` (opaque blob), retains input/output types + cost
    - `HWParam`: omits `HardwareProfile` (opaque blob), retains inner type
    - `Refined`: uses a simplified predicate representation
    - `Vec`: uses `Nat` for size instead of `SizeTerm` -/
inductive TypeDef : Type where
  | Primitive    : PrimType → TypeDef
  | Product      : List TypeId → TypeDef
  | Sum          : List (Tag × TypeId) → TypeDef
  | Recursive    : BoundVar → TypeId → TypeDef
  | ForAll       : BoundVar → TypeId → TypeDef
  | Arrow        : TypeId → TypeId → CostBound → TypeDef
  | Refined      : TypeId → TypeDef   -- base type (predicate abstracted away)
  | NeuralGuard  : TypeId → TypeId → CostBound → TypeDef  -- input, output, cost
  | Exists       : BoundVar → TypeId → TypeDef
  | Vec          : TypeId → Nat → TypeDef
  | HWParam      : TypeId → TypeDef   -- inner type (hardware profile abstracted away)

-- DecidableEq/Repr cannot be auto-derived because TypeDef contains CostBound
-- (which has nested List CostBound). We do not need DecidableEq for TypeDef
-- in the executable code path — type lookup uses TypeId equality instead.

-- ===========================================================================
-- TypeEnv — content-addressed map from TypeId to TypeDef
-- Rust: `pub struct TypeEnv { pub types: BTreeMap<TypeId, TypeDef> }`
-- ===========================================================================

/-- A type environment: a finite map from TypeId to TypeDef.
    Uses `List` for simplicity; the Rust implementation uses `BTreeMap`. -/
def TypeEnv := List (TypeId × TypeDef)

namespace TypeEnv

/-- Look up a type definition by its id. -/
def lookup (env : TypeEnv) (id : TypeId) : Option TypeDef :=
  match env with
  | []          => none
  | (k, v) :: rest => if k == id then some v else lookup rest id

/-- Check whether a TypeId is present in the environment. -/
def contains (env : TypeEnv) (id : TypeId) : Bool :=
  match env with
  | []          => false
  | (k, _) :: rest => if k == id then true else contains rest id

/-- Empty type environment. -/
def empty : TypeEnv := []

/-- Extend a type environment with a new binding. -/
def extend (env : TypeEnv) (id : TypeId) (def_ : TypeDef) : TypeEnv :=
  (id, def_) :: env

end TypeEnv

-- ===========================================================================
-- Binding — a single binding in a typing context
-- Rust: `pub struct Binding { pub name: BinderId, pub type_id: TypeId }`
-- ===========================================================================

/-- A single binding in a typing context: binder name with its type. -/
structure Binding where
  name    : BinderId
  type_id : TypeId
  deriving DecidableEq, Repr, BEq

-- ===========================================================================
-- Context — an ordered list of bindings (Gamma)
-- Rust: `pub struct Context { pub bindings: Vec<Binding> }`
-- ===========================================================================

/-- A typing context: an ordered list of bindings.
    Mirrors `Context` in `src/iris-kernel/src/theorem.rs`. -/
structure Context where
  bindings : List Binding
  deriving DecidableEq, Repr, BEq

namespace Context

/-- The empty context. -/
def empty : Context := ⟨[]⟩

/-- Extend the context with a new binding at the end. -/
def extend (ctx : Context) (name : BinderId) (ty : TypeId) : Context :=
  ⟨ctx.bindings ++ [⟨name, ty⟩]⟩

/-- Look up a binder in the context (searches from most recent, i.e., end of list).

    Since `extend` appends at the end, the most recently added binding is
    found first by searching from the end (via reversal). -/
def lookup (ctx : Context) (name : BinderId) : Option TypeId :=
  go ctx.bindings.reverse
where
  go : List Binding → Option TypeId
    | []     => none
    | b :: rest => if b.name == name then some b.type_id else go rest

/-- Weaken the context by adding a binding at the beginning.

    Unlike `extend` (which appends at the end and is found FIRST by
    `lookup`'s reversed search), `weaken` prepends at the beginning
    so the new binding is found LAST. This means `weaken` never
    shadows existing bindings, making weakening theorems provable.

    Commutation property: `weaken (extend ctx b A) e t = extend (weaken ctx e t) b A`
    (by `List.cons_append`). -/
def weaken (ctx : Context) (name : BinderId) (ty : TypeId) : Context :=
  ⟨⟨name, ty⟩ :: ctx.bindings⟩

/-- Check whether `self` is a prefix of `other`. -/
def isPrefix (self other : Context) : Bool :=
  self.bindings.isPrefixOf other.bindings

/-- Remove the last binding, returning (new context, removed binding).
    Returns `none` if the context is empty. -/
def pop (ctx : Context) : Option (Context × Binding) :=
  match ctx.bindings.reverse with
  | []     => none
  | b :: rest => some (⟨rest.reverse⟩, b)

end Context

-- ===========================================================================
-- Judgment — typing judgment: Γ ⊢ e : τ @ κ
-- Rust: `pub struct Judgment { context, node_id, type_ref, cost }`
-- ===========================================================================

/-- A typing judgment: `Γ ⊢ e : τ @ κ`.
    Reads: "In context Γ, node e has type τ with cost bound κ."
    Mirrors `Judgment` in `src/iris-kernel/src/theorem.rs`. -/
structure Judgment where
  context  : Context
  node_id  : NodeId
  type_ref : TypeId
  cost     : CostBound

instance : Inhabited CostBound := ⟨CostBound.Zero⟩
instance : Inhabited Context := ⟨⟨[]⟩⟩
instance : Inhabited Judgment := ⟨⟨default, default, default, default⟩⟩

-- ===========================================================================
-- CostLeq — the partial order on CostBound
-- Mirrors `cost_leq` in `src/iris-kernel/src/cost_checker.rs`
--
-- Defined as an inductive Prop so we can reason about it.
-- ===========================================================================

/-- `CostLeq a b` witnesses that `a ≤ b` in the cost partial order.
    Mirrors the `cost_leq` function in `src/iris-kernel/src/cost_checker.rs`.

    Key axioms of the partial order:
    - `Zero` is bottom (below everything)
    - `Unknown` is top (above everything)
    - `Zero ≤ Constant ≤ Linear ≤ NLogN ≤ Polynomial` (for same variable)
    - Composite forms (Sum, Par, Mul) are compared pointwise
    - `Sup(vs) ≤ x` iff every `v ∈ vs` satisfies `v ≤ x`
    - `x ≤ Inf(vs)` iff `x ≤ v` for every `v ∈ vs` -/
inductive CostLeq : CostBound → CostBound → Prop where
  /-- Reflexivity: `a ≤ a`. -/
  | refl : (a : CostBound) → CostLeq a a
  /-- Zero is the bottom element. -/
  | zero_bot : (b : CostBound) → CostLeq CostBound.Zero b
  /-- Unknown is the top element. -/
  | unknown_top : (a : CostBound) → CostLeq a CostBound.Unknown
  /-- Constants are ordered by value. -/
  | const_le : (k1 k2 : Nat) → k1 ≤ k2 → CostLeq (CostBound.Constant k1) (CostBound.Constant k2)
  /-- Constant ≤ Linear (same variable). -/
  | const_linear : (k : Nat) → (v : CostVar) → CostLeq (CostBound.Constant k) (CostBound.Linear v)
  /-- Constant ≤ NLogN. -/
  | const_nlogn : (k : Nat) → (v : CostVar) → CostLeq (CostBound.Constant k) (CostBound.NLogN v)
  /-- Constant ≤ Polynomial. -/
  | const_poly : (k : Nat) → (v : CostVar) → (d : Nat) →
      CostLeq (CostBound.Constant k) (CostBound.Polynomial v d)
  /-- Linear ≤ NLogN (same variable). -/
  | linear_nlogn : (v : CostVar) → CostLeq (CostBound.Linear v) (CostBound.NLogN v)
  /-- Linear ≤ Polynomial (same variable, degree ≥ 1). -/
  | linear_poly : (v : CostVar) → (d : Nat) →
      CostLeq (CostBound.Linear v) (CostBound.Polynomial v d)
  /-- NLogN ≤ Polynomial(v, d) when d ≥ 2. -/
  | nlogn_poly : (v : CostVar) → (d : Nat) → 2 ≤ d →
      CostLeq (CostBound.NLogN v) (CostBound.Polynomial v d)
  /-- Polynomial ordering: same variable, smaller degree. -/
  | poly_le : (v : CostVar) → (d1 d2 : Nat) → d1 ≤ d2 →
      CostLeq (CostBound.Polynomial v d1) (CostBound.Polynomial v d2)
  /-- Pointwise Sum comparison. -/
  | sum_le : (a1 a2 b1 b2 : CostBound) →
      CostLeq a1 b1 → CostLeq a2 b2 →
      CostLeq (CostBound.Sum a1 a2) (CostBound.Sum b1 b2)
  /-- Pointwise Par comparison. -/
  | par_le : (a1 a2 b1 b2 : CostBound) →
      CostLeq a1 b1 → CostLeq a2 b2 →
      CostLeq (CostBound.Par a1 a2) (CostBound.Par b1 b2)
  /-- Pointwise Mul comparison. -/
  | mul_le : (a1 a2 b1 b2 : CostBound) →
      CostLeq a1 b1 → CostLeq a2 b2 →
      CostLeq (CostBound.Mul a1 a2) (CostBound.Mul b1 b2)
  /-- Sup is below x if every element is below x. -/
  | sup_le : (vs : List CostBound) → (x : CostBound) →
      (∀ v, v ∈ vs → CostLeq v x) →
      CostLeq (CostBound.Sup vs) x
  /-- x is below Inf if x is below every element. -/
  | le_inf : (x : CostBound) → (vs : List CostBound) →
      (∀ v, v ∈ vs → CostLeq x v) →
      CostLeq x (CostBound.Inf vs)
  /-- Sum embedding left: a ≤ Sum(a, b) when Zero ≤ b. -/
  | sum_embed_left : (a b : CostBound) → CostLeq a (CostBound.Sum a b)
  /-- Sum embedding right: b ≤ Sum(a, b) when Zero ≤ a. -/
  | sum_embed_right : (a b : CostBound) → CostLeq b (CostBound.Sum a b)
  /-- Element of a Sup: if v ∈ vs, then v ≤ Sup(vs). -/
  | elem_sup : (v : CostBound) → (vs : List CostBound) → v ∈ vs →
      CostLeq v (CostBound.Sup vs)
  /-- Inf element: if v ∈ vs, then Inf(vs) ≤ v. -/
  | inf_elem : (v : CostBound) → (vs : List CostBound) → v ∈ vs →
      CostLeq (CostBound.Inf vs) v
  /-- Amortized is conservative: Amortized(inner) ≤ b iff inner ≤ b. -/
  | amortized_le : (inner b : CostBound) → CostLeq inner b →
      CostLeq (CostBound.Amortized inner) b
  /-- a ≤ Amortized(inner) iff a ≤ inner. -/
  | le_amortized : (a inner : CostBound) → CostLeq a inner →
      CostLeq a (CostBound.Amortized inner)
  /-- HWScaled is conservative: HWScaled(inner) ≤ b iff inner ≤ b. -/
  | hwscaled_le : (inner b : CostBound) → CostLeq inner b →
      CostLeq (CostBound.HWScaled inner) b
  /-- a ≤ HWScaled(inner) iff a ≤ inner. -/
  | le_hwscaled : (a inner : CostBound) → CostLeq a inner →
      CostLeq a (CostBound.HWScaled inner)
  /-- Transitivity: if a ≤ b and b ≤ c then a ≤ c. -/
  | trans : (a b c : CostBound) → CostLeq a b → CostLeq b c → CostLeq a c

-- ===========================================================================
-- TypeDef references and well-formedness
-- ===========================================================================

/-- Extract the immediate TypeId references from a TypeDef.
    Mirrors `type_def_references` in `src/iris-kernel/src/kernel.rs`. -/
def typeDefReferences : TypeDef → List TypeId
  | TypeDef.Primitive _         => []
  | TypeDef.Product fields      => fields
  | TypeDef.Sum variants        => variants.map Prod.snd
  | TypeDef.Recursive _ inner   => [inner]
  | TypeDef.ForAll _ inner      => [inner]
  | TypeDef.Arrow param ret _   => [param, ret]
  | TypeDef.Refined inner       => [inner]
  | TypeDef.NeuralGuard i o _   => [i, o]
  | TypeDef.Exists _ inner      => [inner]
  | TypeDef.Vec elem _          => [elem]
  | TypeDef.HWParam inner       => [inner]

/-- A TypeId is well-formed in a TypeEnv if it exists and all its
    immediate type references also exist.
    Mirrors `assert_type_well_formed` in kernel.rs. -/
def TypeWellFormed (env : TypeEnv) (id : TypeId) : Prop :=
  ∃ (td : TypeDef),
    env.lookup id = some td ∧
    ∀ ref_id, ref_id ∈ typeDefReferences td → env.contains ref_id = true

end IrisKernel
