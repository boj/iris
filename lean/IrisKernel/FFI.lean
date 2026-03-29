import IrisKernel.Types
import IrisKernel.Eval
import IrisKernel.Kernel

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
  | 0x0B => do
    let (inner, c) ← decodeCostBound c
    some (CostBound.Amortized inner, c)
  | 0x0C => do
    let (inner, c) ← decodeCostBound c
    some (CostBound.HWScaled inner, c)
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
-- Kernel type decoders
-- ===========================================================================

/-- Decode a NodeId (u64 LE). -/
def decodeNodeId (c : Cursor) : Option (NodeId × Cursor) := do
  let (v, c) ← c.readUInt64LE
  some (⟨v.toNat⟩, c)

/-- Decode a TypeId (u64 LE). -/
def decodeTypeId (c : Cursor) : Option (TypeId × Cursor) := do
  let (v, c) ← c.readUInt64LE
  some (⟨v.toNat⟩, c)

/-- Decode a BinderId (u32 LE). -/
def decodeBinderId (c : Cursor) : Option (BinderId × Cursor) := do
  let (v, c) ← c.readUInt32LE
  some (⟨v.toNat⟩, c)

/-- Decode a CostVar (u32 LE). -/
def decodeCostVar (c : Cursor) : Option (CostVar × Cursor) := do
  let (v, c) ← c.readUInt32LE
  some (⟨v.toNat⟩, c)

/-- Decode a Context: count (u16 LE), then count × (BinderId + TypeId) pairs. -/
def decodeContext (c : Cursor) : Option (Context × Cursor) := do
  let (count, c) ← c.readUInt16LE
  decodeBindings c count.toNat []
where
  decodeBindings (c : Cursor) (n : Nat) (acc : List Binding) : Option (Context × Cursor) :=
    match n with
    | 0 => some (⟨acc.reverse⟩, c)
    | n + 1 => do
      let (name, c) ← decodeBinderId c
      let (ty, c) ← decodeTypeId c
      decodeBindings c n ({ name := name, type_id := ty } :: acc)

/-- Decode a PrimType tag. -/
def decodePrimType (c : Cursor) : Option (PrimType × Cursor) := do
  let (tag, c) ← c.readUInt8
  match tag.toNat with
  | 0 => some (PrimType.Int, c)
  | 1 => some (PrimType.Nat, c)
  | 2 => some (PrimType.Float64, c)
  | 3 => some (PrimType.Float32, c)
  | 4 => some (PrimType.Bool, c)
  | 5 => some (PrimType.Bytes, c)
  | 6 => some (PrimType.Unit, c)
  | _ => none

/-- Decode a Tag (u16 LE). -/
def decodeTag (c : Cursor) : Option (Tag × Cursor) := do
  let (v, c) ← c.readUInt16LE
  some (⟨v.toNat⟩, c)

/-- Decode a BoundVar (u32 LE). -/
def decodeBoundVar (c : Cursor) : Option (BoundVar × Cursor) := do
  let (v, c) ← c.readUInt32LE
  some (⟨v.toNat⟩, c)

/-- Decode a TypeDef from a byte cursor.
    Format: tag byte, then variant-specific payload. -/
partial def decodeTypeDef (c : Cursor) : Option (TypeDef × Cursor) := do
  let (tag, c) ← c.readUInt8
  match tag.toNat with
  | 0x00 => do  -- Primitive
    let (pt, c) ← decodePrimType c
    some (TypeDef.Primitive pt, c)
  | 0x01 => do  -- Product: count (u16), then TypeIds
    let (count, c) ← c.readUInt16LE
    let (fields, c) ← decodeTypeIdList c count.toNat []
    some (TypeDef.Product fields, c)
  | 0x02 => do  -- Sum: count (u16), then (Tag, TypeId) pairs
    let (count, c) ← c.readUInt16LE
    let (variants, c) ← decodeTagTypeIdList c count.toNat []
    some (TypeDef.Sum variants, c)
  | 0x03 => do  -- Recursive
    let (bv, c) ← decodeBoundVar c
    let (inner, c) ← decodeTypeId c
    some (TypeDef.Recursive bv inner, c)
  | 0x04 => do  -- ForAll
    let (bv, c) ← decodeBoundVar c
    let (inner, c) ← decodeTypeId c
    some (TypeDef.ForAll bv inner, c)
  | 0x05 => do  -- Arrow
    let (param, c) ← decodeTypeId c
    let (ret, c) ← decodeTypeId c
    let (cost, c) ← decodeCostBound c
    some (TypeDef.Arrow param ret cost, c)
  | 0x06 => do  -- Refined
    let (inner, c) ← decodeTypeId c
    some (TypeDef.Refined inner, c)
  | 0x07 => do  -- NeuralGuard
    let (inp, c) ← decodeTypeId c
    let (out, c) ← decodeTypeId c
    let (cost, c) ← decodeCostBound c
    some (TypeDef.NeuralGuard inp out cost, c)
  | 0x08 => do  -- Exists
    let (bv, c) ← decodeBoundVar c
    let (inner, c) ← decodeTypeId c
    some (TypeDef.Exists bv inner, c)
  | 0x09 => do  -- Vec
    let (elem, c) ← decodeTypeId c
    let (size, c) ← c.readUInt64LE
    some (TypeDef.Vec elem size.toNat, c)
  | 0x0A => do  -- HWParam
    let (inner, c) ← decodeTypeId c
    some (TypeDef.HWParam inner, c)
  | _ => none
where
  decodeTypeIdList (c : Cursor) (n : Nat) (acc : List TypeId) : Option (List TypeId × Cursor) :=
    match n with
    | 0 => some (acc.reverse, c)
    | n + 1 => do
      let (tid, c) ← decodeTypeId c
      decodeTypeIdList c n (tid :: acc)
  decodeTagTypeIdList (c : Cursor) (n : Nat) (acc : List (Tag × TypeId)) : Option (List (Tag × TypeId) × Cursor) :=
    match n with
    | 0 => some (acc.reverse, c)
    | n + 1 => do
      let (tag, c) ← decodeTag c
      let (tid, c) ← decodeTypeId c
      decodeTagTypeIdList c n ((tag, tid) :: acc)

/-- Decode a TypeEnv: count (u16 LE), then count × (TypeId, TypeDef) pairs. -/
partial def decodeTypeEnv (c : Cursor) : Option (TypeEnv × Cursor) := do
  let (count, c) ← c.readUInt16LE
  decodeEntries c count.toNat []
where
  decodeEntries (c : Cursor) (n : Nat) (acc : TypeEnv) : Option (TypeEnv × Cursor) :=
    match n with
    | 0 => some (acc.reverse, c)
    | n + 1 => do
      let (tid, c) ← decodeTypeId c
      let (tdef, c) ← decodeTypeDef c
      decodeEntries c n ((tid, tdef) :: acc)

/-- Decode a Judgment: Context + NodeId + TypeId + CostBound. -/
partial def decodeJudgment (c : Cursor) : Option (Judgment × Cursor) := do
  let (ctx, c) ← decodeContext c
  let (nid, c) ← decodeNodeId c
  let (tid, c) ← decodeTypeId c
  let (cost, c) ← decodeCostBound c
  some ({ context := ctx, node_id := nid, type_ref := tid, cost := cost }, c)

-- ===========================================================================
-- Kernel type encoders (for returning results to Rust)
-- ===========================================================================

namespace Cursor

/-- Write a UInt8 into a ByteArray. -/
def writeUInt8 (arr : ByteArray) (v : UInt8) : ByteArray :=
  arr.push v

/-- Write a UInt16 LE into a ByteArray. -/
def writeUInt16LE (arr : ByteArray) (v : UInt16) : ByteArray :=
  arr.push (v.toUInt8) |>.push ((v >>> 8).toUInt8)

/-- Write a UInt32 LE into a ByteArray. -/
def writeUInt32LE (arr : ByteArray) (v : UInt32) : ByteArray :=
  arr.push (v.toUInt8) |>.push ((v >>> 8).toUInt8) |>.push ((v >>> 16).toUInt8) |>.push ((v >>> 24).toUInt8)

/-- Write a UInt64 LE into a ByteArray. -/
def writeUInt64LE (arr : ByteArray) (v : UInt64) : ByteArray :=
  arr.push (v.toUInt8) |>.push ((v >>> 8).toUInt8) |>.push ((v >>> 16).toUInt8) |>.push ((v >>> 24).toUInt8) |>.push ((v >>> 32).toUInt8) |>.push ((v >>> 40).toUInt8) |>.push ((v >>> 48).toUInt8) |>.push ((v >>> 56).toUInt8)

end Cursor

/-- Encode a NodeId as u64 LE. -/
def encodeNodeId (arr : ByteArray) (n : NodeId) : ByteArray :=
  Cursor.writeUInt64LE arr n.val.toUInt64

/-- Encode a TypeId as u64 LE. -/
def encodeTypeId (arr : ByteArray) (t : TypeId) : ByteArray :=
  Cursor.writeUInt64LE arr t.val.toUInt64

/-- Encode a BinderId as u32 LE. -/
def encodeBinderId (arr : ByteArray) (b : BinderId) : ByteArray :=
  Cursor.writeUInt32LE arr b.val.toUInt32

/-- Encode a CostBound to bytes. -/
partial def encodeCostBound (arr : ByteArray) (c : CostBound) : ByteArray :=
  match c with
  | .Unknown => arr.push 0x00
  | .Zero => arr.push 0x01
  | .Constant k => Cursor.writeUInt64LE (arr.push 0x02) k.toUInt64
  | .Linear v => Cursor.writeUInt32LE (arr.push 0x03) v.val.toUInt32
  | .NLogN v => Cursor.writeUInt32LE (arr.push 0x04) v.val.toUInt32
  | .Polynomial v d => Cursor.writeUInt32LE (Cursor.writeUInt32LE (arr.push 0x05) v.val.toUInt32) d.toUInt32
  | .Sum a b => encodeCostBound (encodeCostBound (arr.push 0x06) a) b
  | .Par a b => encodeCostBound (encodeCostBound (arr.push 0x07) a) b
  | .Mul a b => encodeCostBound (encodeCostBound (arr.push 0x08) a) b
  | .Sup vs =>
    let arr := Cursor.writeUInt16LE (arr.push 0x09) vs.length.toUInt16
    vs.foldl (fun acc v => encodeCostBound acc v) arr
  | .Inf vs =>
    let arr := Cursor.writeUInt16LE (arr.push 0x0A) vs.length.toUInt16
    vs.foldl (fun acc v => encodeCostBound acc v) arr
  | .Amortized inner => encodeCostBound (arr.push 0x0B) inner
  | .HWScaled inner => encodeCostBound (arr.push 0x0C) inner

/-- Encode a Context: count (u16 LE), then bindings. -/
def encodeContext (arr : ByteArray) (ctx : Context) : ByteArray :=
  let arr := Cursor.writeUInt16LE arr ctx.bindings.length.toUInt16
  ctx.bindings.foldl (fun acc b =>
    encodeTypeId (encodeBinderId acc b.name) b.type_id) arr

/-- Encode a Judgment: Context + NodeId + TypeId + CostBound.
    Returns byte 1 (success) followed by the encoded judgment. -/
partial def encodeJudgment (arr : ByteArray) (j : Judgment) : ByteArray :=
  let arr := encodeContext arr j.context
  let arr := encodeNodeId arr j.node_id
  let arr := encodeTypeId arr j.type_ref
  encodeCostBound arr j.cost

/-- Encode a successful result: byte 1 + Judgment. -/
partial def encodeSuccess (j : Judgment) : ByteArray :=
  encodeJudgment (ByteArray.mkEmpty 64 |>.push 1) j

/-- Encode a failure result: byte 0 + error code. -/
def encodeFailure (errCode : UInt8) : ByteArray :=
  ByteArray.mkEmpty 2 |>.push 0 |>.push errCode

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

-- ===========================================================================
-- NodeKind decoder helper
-- ===========================================================================

/-- Decode a NodeKind from a byte tag (0x00-0x13). -/
def decodeNodeKind (tag : UInt8) : Option NodeKind :=
  match tag.toNat with
  | 0x00 => some NodeKind.Prim
  | 0x01 => some NodeKind.Apply
  | 0x02 => some NodeKind.Lambda
  | 0x03 => some NodeKind.Let
  | 0x04 => some NodeKind.Match
  | 0x05 => some NodeKind.Lit
  | 0x06 => some NodeKind.Ref
  | 0x07 => some NodeKind.Neural
  | 0x08 => some NodeKind.Fold
  | 0x09 => some NodeKind.Unfold
  | 0x0A => some NodeKind.Effect
  | 0x0B => some NodeKind.Tuple
  | 0x0C => some NodeKind.Inject
  | 0x0D => some NodeKind.Project
  | 0x0E => some NodeKind.TypeAbst
  | 0x0F => some NodeKind.TypeApp
  | 0x10 => some NodeKind.LetRec
  | 0x11 => some NodeKind.Guard
  | 0x12 => some NodeKind.Rewrite
  | 0x13 => some NodeKind.Extern
  | _    => none

-- ===========================================================================
-- Helper: wrap result
-- ===========================================================================

/-- Wrap a kernel result into the FFI return format. -/
partial def wrapResult (result : Option Judgment) : ByteArray :=
  match result with
  | some j => encodeSuccess j
  | none => encodeFailure 1

-- ===========================================================================
-- Exported FFI functions — 20 kernel rules
-- ===========================================================================

/-- Rule 1: assume — decode Context + BinderId + NodeId. -/
@[export iris_kernel_assume]
partial def assumeFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeContext c with
  | none => encodeFailure 2
  | some (ctx, c) =>
    match decodeBinderId c with
    | none => encodeFailure 2
    | some (name, c) =>
      match decodeNodeId c with
      | none => encodeFailure 2
      | some (nodeId, _) => wrapResult (Kernel.assume_ ctx name nodeId)

/-- Rule 2: intro — decode TypeEnv + Context + NodeId + BinderId + TypeId + Judgment + TypeId. -/
@[export iris_kernel_intro]
partial def introFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeTypeEnv c with
  | none => encodeFailure 2
  | some (env, c) =>
    match decodeContext c with
    | none => encodeFailure 2
    | some (ctx, c) =>
      match decodeNodeId c with
      | none => encodeFailure 2
      | some (lamNode, c) =>
        match decodeBinderId c with
        | none => encodeFailure 2
        | some (binderName, c) =>
          match decodeTypeId c with
          | none => encodeFailure 2
          | some (binderType, c) =>
            match decodeJudgment c with
            | none => encodeFailure 2
            | some (bodyJ, c) =>
              match decodeTypeId c with
              | none => encodeFailure 2
              | some (arrowId, _) =>
                wrapResult (Kernel.intro env ctx lamNode binderName binderType bodyJ arrowId)

/-- Rule 3: elim — decode TypeEnv + Judgment(fn) + Judgment(arg) + NodeId. -/
@[export iris_kernel_elim]
partial def elimFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeTypeEnv c with
  | none => encodeFailure 2
  | some (env, c) =>
    match decodeJudgment c with
    | none => encodeFailure 2
    | some (fnJ, c) =>
      match decodeJudgment c with
      | none => encodeFailure 2
      | some (argJ, c) =>
        match decodeNodeId c with
        | none => encodeFailure 2
        | some (appNode, _) => wrapResult (Kernel.elim env fnJ argJ appNode)

/-- Rule 4: refl — decode Context + NodeId + TypeId. -/
@[export iris_kernel_refl]
partial def reflFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeContext c with
  | none => encodeFailure 2
  | some (ctx, c) =>
    match decodeNodeId c with
    | none => encodeFailure 2
    | some (nodeId, c) =>
      match decodeTypeId c with
      | none => encodeFailure 2
      | some (typeId, _) => encodeSuccess (Kernel.refl_ ctx nodeId typeId)

/-- Rule 5: symm — decode Judgment(thm) + NodeId + Judgment(eqWitness). -/
@[export iris_kernel_symm]
partial def symmFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeJudgment c with
  | none => encodeFailure 2
  | some (thm, c) =>
    match decodeNodeId c with
    | none => encodeFailure 2
    | some (otherNode, c) =>
      match decodeJudgment c with
      | none => encodeFailure 2
      | some (eqWitness, _) => wrapResult (Kernel.symm_ thm otherNode eqWitness)

/-- Rule 6: trans — decode Judgment(thm1) + Judgment(thm2). -/
@[export iris_kernel_trans]
partial def transFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeJudgment c with
  | none => encodeFailure 2
  | some (thm1, c) =>
    match decodeJudgment c with
    | none => encodeFailure 2
    | some (thm2, _) => wrapResult (Kernel.trans_ thm1 thm2)

/-- Rule 7: congr — decode Judgment(fn) + Judgment(arg) + NodeId. -/
@[export iris_kernel_congr]
partial def congrFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeJudgment c with
  | none => encodeFailure 2
  | some (fnJ, c) =>
    match decodeJudgment c with
    | none => encodeFailure 2
    | some (argJ, c) =>
      match decodeNodeId c with
      | none => encodeFailure 2
      | some (appNode, _) => wrapResult (Kernel.congr_ fnJ argJ appNode)

/-- Rule 8: type_check_node — decode TypeEnv + Context + NodeId + NodeKind(u8) + TypeId. -/
@[export iris_kernel_type_check_node_full]
partial def typeCheckNodeFullFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeTypeEnv c with
  | none => encodeFailure 2
  | some (env, c) =>
    match decodeContext c with
    | none => encodeFailure 2
    | some (ctx, c) =>
      match decodeNodeId c with
      | none => encodeFailure 2
      | some (nodeId, c) =>
        match c.readUInt8 with
        | none => encodeFailure 2
        | some (kindTag, c) =>
          match decodeNodeKind kindTag with
          | none => encodeFailure 3
          | some kind =>
            match decodeTypeId c with
            | none => encodeFailure 2
            | some (typeSig, _) =>
              wrapResult (Kernel.typeCheckNode_ env ctx nodeId kind typeSig)

/-- Rule 9: cost_subsume — decode Judgment + CostBound. -/
@[export iris_kernel_cost_subsume]
partial def costSubsumeFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeJudgment c with
  | none => encodeFailure 2
  | some (j, c) =>
    match decodeCostBound c with
    | none => encodeFailure 2
    | some (newCost, _) => wrapResult (Kernel.costSubsume_ j newCost)

/-- Rule 10: cost_leq_rule — decode CostBound + CostBound. -/
@[export iris_kernel_cost_leq_rule]
partial def costLeqRuleFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeCostBound c with
  | none => encodeFailure 2
  | some (κ₁, c) =>
    match decodeCostBound c with
    | none => encodeFailure 2
    | some (κ₂, _) => wrapResult (Kernel.costLeqRule_ κ₁ κ₂)

/-- Rule 11: refine_intro — decode TypeEnv + Judgment(base) + Judgment(pred) + TypeId. -/
@[export iris_kernel_refine_intro]
partial def refineIntroFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeTypeEnv c with
  | none => encodeFailure 2
  | some (env, c) =>
    match decodeJudgment c with
    | none => encodeFailure 2
    | some (baseJ, c) =>
      match decodeJudgment c with
      | none => encodeFailure 2
      | some (predJ, c) =>
        match decodeTypeId c with
        | none => encodeFailure 2
        | some (refinedTypeId, _) =>
          wrapResult (Kernel.refineIntro env baseJ predJ refinedTypeId)

/-- Rule 12: refine_elim — decode TypeEnv + Judgment. -/
@[export iris_kernel_refine_elim]
partial def refineElimFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeTypeEnv c with
  | none => encodeFailure 2
  | some (env, c) =>
    match decodeJudgment c with
    | none => encodeFailure 2
    | some (j, _) => wrapResult (Kernel.refineElim env j)

/-- Rule 13: nat_ind — decode Judgment(base) + Judgment(step) + NodeId. -/
@[export iris_kernel_nat_ind]
partial def natIndFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeJudgment c with
  | none => encodeFailure 2
  | some (baseJ, c) =>
    match decodeJudgment c with
    | none => encodeFailure 2
    | some (stepJ, c) =>
      match decodeNodeId c with
      | none => encodeFailure 2
      | some (resultNode, _) => wrapResult (Kernel.natInd baseJ stepJ resultNode)

/-- Rule 14: structural_ind — decode TypeEnv + TypeId(sumType) + count(u16) + Judgment[] + NodeId. -/
@[export iris_kernel_structural_ind]
partial def structuralIndFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeTypeEnv c with
  | none => encodeFailure 2
  | some (env, c) =>
    match decodeTypeId c with
    | none => encodeFailure 2
    | some (sumType, c) =>
      match c.readUInt16LE with
      | none => encodeFailure 2
      | some (count, c) =>
        match decodeCases c count.toNat [] with
        | none => encodeFailure 2
        | some (cases, c) =>
          match decodeNodeId c with
          | none => encodeFailure 2
          | some (resultNode, _) =>
            wrapResult (Kernel.structuralInd env sumType cases resultNode)
where
  decodeCases (c : Cursor) (n : Nat) (acc : List Judgment) : Option (List Judgment × Cursor) :=
    match n with
    | 0 => some (acc.reverse, c)
    | n + 1 => do
      let (j, c) ← decodeJudgment c
      decodeCases c n (j :: acc)

/-- Rule 15: let_bind — decode Context + NodeId + BinderId + Judgment(bound) + Judgment(body). -/
@[export iris_kernel_let_bind]
partial def letBindFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeContext c with
  | none => encodeFailure 2
  | some (ctx, c) =>
    match decodeNodeId c with
    | none => encodeFailure 2
    | some (letNode, c) =>
      match decodeBinderId c with
      | none => encodeFailure 2
      | some (binderName, c) =>
        match decodeJudgment c with
        | none => encodeFailure 2
        | some (boundJ, c) =>
          match decodeJudgment c with
          | none => encodeFailure 2
          | some (bodyJ, _) =>
            wrapResult (Kernel.letBind ctx letNode binderName boundJ bodyJ)

/-- Rule 16: match_elim — decode Judgment(scrutinee) + count(u16) + Judgment[] + NodeId. -/
@[export iris_kernel_match_elim]
partial def matchElimFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeJudgment c with
  | none => encodeFailure 2
  | some (scrutineeJ, c) =>
    match c.readUInt16LE with
    | none => encodeFailure 2
    | some (count, c) =>
      match decodeArms c count.toNat [] with
      | none => encodeFailure 2
      | some (arms, c) =>
        match decodeNodeId c with
        | none => encodeFailure 2
        | some (matchNode, _) =>
          wrapResult (Kernel.matchElim scrutineeJ arms matchNode)
where
  decodeArms (c : Cursor) (n : Nat) (acc : List Judgment) : Option (List Judgment × Cursor) :=
    match n with
    | 0 => some (acc.reverse, c)
    | n + 1 => do
      let (j, c) ← decodeJudgment c
      decodeArms c n (j :: acc)

/-- Rule 17: fold_rule — decode Judgment(base) + Judgment(step) + Judgment(input) + NodeId. -/
@[export iris_kernel_fold_rule]
partial def foldRuleFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeJudgment c with
  | none => encodeFailure 2
  | some (baseJ, c) =>
    match decodeJudgment c with
    | none => encodeFailure 2
    | some (stepJ, c) =>
      match decodeJudgment c with
      | none => encodeFailure 2
      | some (inputJ, c) =>
        match decodeNodeId c with
        | none => encodeFailure 2
        | some (foldNode, _) =>
          wrapResult (Kernel.foldRule baseJ stepJ inputJ foldNode)

/-- Rule 18: type_abst — decode TypeEnv + Judgment + TypeId. -/
@[export iris_kernel_type_abst]
partial def typeAbstFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeTypeEnv c with
  | none => encodeFailure 2
  | some (env, c) =>
    match decodeJudgment c with
    | none => encodeFailure 2
    | some (bodyJ, c) =>
      match decodeTypeId c with
      | none => encodeFailure 2
      | some (forallTypeId, _) =>
        wrapResult (Kernel.typeAbst env bodyJ forallTypeId)

/-- Rule 19: type_app — decode TypeEnv + Judgment + TypeId. -/
@[export iris_kernel_type_app]
partial def typeAppFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeTypeEnv c with
  | none => encodeFailure 2
  | some (env, c) =>
    match decodeJudgment c with
    | none => encodeFailure 2
    | some (j, c) =>
      match decodeTypeId c with
      | none => encodeFailure 2
      | some (resultTypeId, _) =>
        wrapResult (Kernel.typeApp env j resultTypeId)

/-- Rule 20: guard_rule — decode Judgment(pred) + Judgment(then) + Judgment(else) + NodeId. -/
@[export iris_kernel_guard_rule]
partial def guardRuleFFI (data : @& ByteArray) : ByteArray :=
  let c := Cursor.create data
  match decodeJudgment c with
  | none => encodeFailure 2
  | some (predJ, c) =>
    match decodeJudgment c with
    | none => encodeFailure 2
    | some (thenJ, c) =>
      match decodeJudgment c with
      | none => encodeFailure 2
      | some (elseJ, c) =>
        match decodeNodeId c with
        | none => encodeFailure 2
        | some (guardNode, _) =>
          wrapResult (Kernel.guardRule predJ thenJ elseJ guardNode)

end IrisKernel.FFI
