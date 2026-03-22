import IrisKernel.Types
import IrisKernel.Eval

/-!
# IRIS Proof Kernel — FFI Exports

C-callable functions exported via Lean's `@[export]` attribute.
These are the entry points that the Rust kernel calls through
its FFI bridge.

## Wire format

CostBound and LIA types are encoded as flat byte arrays. See the
tag encoding documentation in the Rust bridge (`lean_bridge.rs`).
-/

namespace IrisKernel.FFI

-- ===========================================================================
-- ByteArray reader: a simple cursor over a byte array
-- ===========================================================================

/-- A read cursor: position in a ByteArray. -/
structure Cursor where
  data : ByteArray
  pos  : Nat

namespace Cursor

def create (data : ByteArray) : Cursor := ⟨data, 0⟩

def remaining (c : Cursor) : Nat := c.data.size - c.pos

def readUInt8 (c : Cursor) : Option (UInt8 × Cursor) :=
  if c.pos < c.data.size then
    some (c.data.get! c.pos, { c with pos := c.pos + 1 })
  else
    none

def readUInt16LE (c : Cursor) : Option (UInt16 × Cursor) :=
  if c.pos + 1 < c.data.size then
    let lo := c.data.get! c.pos
    let hi := c.data.get! (c.pos + 1)
    let val := lo.toUInt16 ||| (hi.toUInt16 <<< 8)
    some (val, { c with pos := c.pos + 2 })
  else
    none

def readUInt32LE (c : Cursor) : Option (UInt32 × Cursor) :=
  if c.pos + 3 < c.data.size then
    let b0 := c.data.get! c.pos
    let b1 := c.data.get! (c.pos + 1)
    let b2 := c.data.get! (c.pos + 2)
    let b3 := c.data.get! (c.pos + 3)
    let val := b0.toUInt32 ||| (b1.toUInt32 <<< 8) |||
               (b2.toUInt32 <<< 16) ||| (b3.toUInt32 <<< 24)
    some (val, { c with pos := c.pos + 4 })
  else
    none

def readUInt64LE (c : Cursor) : Option (UInt64 × Cursor) :=
  if c.pos + 7 < c.data.size then
    let b0 := (c.data.get! c.pos).toUInt64
    let b1 := (c.data.get! (c.pos + 1)).toUInt64
    let b2 := (c.data.get! (c.pos + 2)).toUInt64
    let b3 := (c.data.get! (c.pos + 3)).toUInt64
    let b4 := (c.data.get! (c.pos + 4)).toUInt64
    let b5 := (c.data.get! (c.pos + 5)).toUInt64
    let b6 := (c.data.get! (c.pos + 6)).toUInt64
    let b7 := (c.data.get! (c.pos + 7)).toUInt64
    let val := b0 ||| (b1 <<< 8) ||| (b2 <<< 16) ||| (b3 <<< 24) |||
               (b4 <<< 32) ||| (b5 <<< 40) ||| (b6 <<< 48) ||| (b7 <<< 56)
    some (val, { c with pos := c.pos + 8 })
  else
    none

def readInt64LE (c : Cursor) : Option (Int × Cursor) :=
  match c.readUInt64LE with
  | some (v, c') =>
    -- Reinterpret as signed: if bit 63 is set, subtract 2^64.
    let n := v.toNat
    let signed : Int := if n ≥ 2^63 then (n : Int) - (2^64 : Int) else (n : Int)
    some (signed, c')
  | none => none

end Cursor

-- ===========================================================================
-- CostBound decoder
-- ===========================================================================

/-- Decode a CostBound from a byte cursor. -/
partial def decodeCostBound (c : Cursor) : Option (CostBound × Cursor) := do
  let (tag, c) ← c.readUInt8
  match tag.toNat with
  | 0x00 => some (CostBound.Unknown, c)
  | 0x01 => some (CostBound.Zero, c)
  | 0x02 => do
    let (k, c) ← c.readUInt64LE
    some (CostBound.Constant k.toNat, c)
  | 0x03 => do
    let (v, c) ← c.readUInt32LE
    some (CostBound.Linear ⟨v.toNat⟩, c)
  | 0x04 => do
    let (v, c) ← c.readUInt32LE
    some (CostBound.NLogN ⟨v.toNat⟩, c)
  | 0x05 => do
    let (v, c) ← c.readUInt32LE
    let (d, c) ← c.readUInt32LE
    some (CostBound.Polynomial ⟨v.toNat⟩ d.toNat, c)
  | 0x06 => do
    let (a, c) ← decodeCostBound c
    let (b, c) ← decodeCostBound c
    some (CostBound.Sum a b, c)
  | 0x07 => do
    let (a, c) ← decodeCostBound c
    let (b, c) ← decodeCostBound c
    some (CostBound.Par a b, c)
  | 0x08 => do
    let (a, c) ← decodeCostBound c
    let (b, c) ← decodeCostBound c
    some (CostBound.Mul a b, c)
  | 0x09 => do
    let (count, c) ← c.readUInt16LE
    let (vs, c) ← decodeCostBoundList c count.toNat
    some (CostBound.Sup vs, c)
  | 0x0A => do
    let (count, c) ← c.readUInt16LE
    let (vs, c) ← decodeCostBoundList c count.toNat
    some (CostBound.Inf vs, c)
  | _ => none

where
  decodeCostBoundList (c : Cursor) (n : Nat) : Option (List CostBound × Cursor) :=
    match n with
    | 0 => some ([], c)
    | n + 1 => do
      let (v, c) ← decodeCostBound c
      let (rest, c) ← decodeCostBoundList c n
      some (v :: rest, c)

-- ===========================================================================
-- LIA decoder
-- ===========================================================================

mutual
  /-- Decode a LIATerm from a byte cursor. -/
  partial def decodeLIATerm (c : Cursor) : Option (LIATerm × Cursor) := do
    let (tag, c) ← c.readUInt8
    match tag.toNat with
    | 0x00 => do
      let (v, c) ← c.readUInt32LE
      some (LIATerm.Var v.toNat, c)
    | 0x01 => do
      let (v, c) ← c.readInt64LE
      some (LIATerm.Const v, c)
    | 0x02 => do
      let (a, c) ← decodeLIATerm c
      let (b, c) ← decodeLIATerm c
      some (LIATerm.Add a b, c)
    | 0x03 => do
      let (coeff, c) ← c.readInt64LE
      let (t, c) ← decodeLIATerm c
      some (LIATerm.Mul coeff t, c)
    | 0x04 => do
      let (t, c) ← decodeLIATerm c
      some (LIATerm.Neg t, c)
    | 0x05 => do
      let (v, c) ← c.readUInt32LE
      some (LIATerm.Len v.toNat, c)
    | 0x06 => do
      let (v, c) ← c.readUInt32LE
      some (LIATerm.Size v.toNat, c)
    | 0x07 => do
      let (cond, c) ← decodeLIAFormula c
      let (a, c) ← decodeLIATerm c
      let (b, c) ← decodeLIATerm c
      some (LIATerm.IfThenElse cond a b, c)
    | 0x08 => do
      let (a, c) ← decodeLIATerm c
      let (b, c) ← decodeLIATerm c
      some (LIATerm.Mod a b, c)
    | _ => none

  /-- Decode a LIAAtom from a byte cursor. -/
  partial def decodeLIAAtom (c : Cursor) : Option (LIAAtom × Cursor) := do
    let (tag, c) ← c.readUInt8
    match tag.toNat with
    | 0x00 => do
      let (a, c) ← decodeLIATerm c
      let (b, c) ← decodeLIATerm c
      some (LIAAtom.Eq a b, c)
    | 0x01 => do
      let (a, c) ← decodeLIATerm c
      let (b, c) ← decodeLIATerm c
      some (LIAAtom.Lt a b, c)
    | 0x02 => do
      let (a, c) ← decodeLIATerm c
      let (b, c) ← decodeLIATerm c
      some (LIAAtom.Le a b, c)
    | 0x03 => do
      let (t, c) ← decodeLIATerm c
      let (d, c) ← c.readUInt64LE
      some (LIAAtom.Divisible t d.toNat, c)
    | _ => none

  /-- Decode a LIAFormula from a byte cursor. -/
  partial def decodeLIAFormula (c : Cursor) : Option (LIAFormula × Cursor) := do
    let (tag, c) ← c.readUInt8
    match tag.toNat with
    | 0x00 => some (LIAFormula.True, c)
    | 0x01 => some (LIAFormula.False, c)
    | 0x02 => do
      let (a, c) ← decodeLIAFormula c
      let (b, c) ← decodeLIAFormula c
      some (LIAFormula.And a b, c)
    | 0x03 => do
      let (a, c) ← decodeLIAFormula c
      let (b, c) ← decodeLIAFormula c
      some (LIAFormula.Or a b, c)
    | 0x04 => do
      let (f, c) ← decodeLIAFormula c
      some (LIAFormula.Not f, c)
    | 0x05 => do
      let (a, c) ← decodeLIAFormula c
      let (b, c) ← decodeLIAFormula c
      some (LIAFormula.Implies a b, c)
    | 0x06 => do
      let (atom, c) ← decodeLIAAtom c
      some (LIAFormula.Atom atom, c)
    | _ => none
end

-- ===========================================================================
-- LIA environment decoder
-- ===========================================================================

/-- Decode a LIA variable assignment from bytes.
    Format: count (UInt16 LE), then count × (var_id: UInt32 LE, value: Int64 LE). -/
partial def decodeLIAEnv (c : Cursor) : Option (LIAEnv × Cursor) := do
  let (count, c) ← c.readUInt16LE
  decodeLIAEnvEntries c count.toNat []
where
  decodeLIAEnvEntries (c : Cursor) (n : Nat) (acc : LIAEnv) : Option (LIAEnv × Cursor) :=
    match n with
    | 0 => some (acc.reverse, c)
    | n + 1 => do
      let (varId, c) ← c.readUInt32LE
      let (value, c) ← c.readInt64LE
      decodeLIAEnvEntries c n ((varId.toNat, value) :: acc)

-- ===========================================================================
-- Exported FFI functions
-- ===========================================================================

/-- Check cost ordering: returns 1 if a ≤ b, 0 otherwise.

    The Rust side encodes two CostBound values into a single ByteArray
    (a followed by b), passes it here, and gets back a UInt8.

    This is the pilot function — the first kernel decision procedure
    to be backed by Lean-verified code. -/
@[export iris_check_cost_leq]
def checkCostLeqFFI (data : @& ByteArray) : UInt8 :=
  let cursor := Cursor.create data
  match decodeCostBound cursor with
  | none => 0
  | some (a, cursor') =>
    match decodeCostBound cursor' with
    | none => 0
    | some (b, _) =>
      if checkCostLeq a b then 1 else 0

/-- Evaluate a LIA formula with a variable assignment.

    Input ByteArray format:
    - LIAFormula (encoded)
    - LIAEnv (encoded)

    Returns 1 if the formula evaluates to true, 0 otherwise. -/
@[export iris_eval_lia]
def evalLIAFFI (data : @& ByteArray) : UInt8 :=
  let cursor := Cursor.create data
  match decodeLIAFormula cursor with
  | none => 0
  | some (formula, cursor') =>
    match decodeLIAEnv cursor' with
    | none => 0
    | some (env, _) =>
      if evalLIAFormula formula env then 1 else 0

/-- Type-check a node: verify its type signature is well-formed.

    For the initial pilot, we only verify that the kind tag is valid
    (0x00-0x13 = 20 NodeKind variants). Full type environment
    deserialization will be added in a later step when more rules
    are migrated to Lean. -/
@[export iris_type_check_node]
def typeCheckNodeFFI (kindTag : UInt8) (_typeSigHi _typeSigLo : UInt64) : UInt8 :=
  if kindTag.toNat < 20 then 1 else 0

/-- Version/health check: returns a magic number to verify the library loaded. -/
@[export iris_lean_kernel_version]
def kernelVersion : UInt32 := 0x49524953  -- "IRIS" in ASCII

end IrisKernel.FFI
