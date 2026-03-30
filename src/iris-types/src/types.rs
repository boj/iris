use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::cost::{CostBound, HardwareProfile};

// ---------------------------------------------------------------------------
// Identity types
// ---------------------------------------------------------------------------

/// 64-bit content-addressed type identity (BLAKE3 truncated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypeId(pub u64);

/// Index into a `TypeEnv`; same width as `TypeId`.
pub type TypeRef = TypeId;

/// De Bruijn-style bound variable identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BoundVar(pub u32);

/// Tag for sum-type variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tag(pub u16);

// ---------------------------------------------------------------------------
// TypeEnv
// ---------------------------------------------------------------------------

/// Content-addressed map of type definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeEnv {
    pub types: BTreeMap<TypeId, TypeDef>,
}

// ---------------------------------------------------------------------------
// TypeDef (11 variants)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeDef {
    /// Int, Nat, Float64, Float32, Bool, Bytes, Unit
    Primitive(PrimType),
    /// Tuple / struct
    Product(Vec<TypeId>),
    /// Tagged union
    Sum(Vec<(Tag, TypeId)>),
    /// mu X. F(X)
    Recursive(BoundVar, TypeId),
    /// Polymorphism
    ForAll(BoundVar, TypeId),
    /// Function WITH cost
    Arrow(TypeId, TypeId, CostBound),
    /// Refinement type
    Refined(TypeId, RefinementPredicate),
    /// Neural computation with cost
    NeuralGuard(TypeId, TypeId, crate::guard::GuardSpec, CostBound),
    /// Existential type
    Exists(BoundVar, TypeId),
    /// Sized vector
    Vec(TypeId, SizeTerm),
    /// HW-parameterized type
    HWParam(TypeId, HardwareProfile),
}

// ---------------------------------------------------------------------------
// PrimType
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimType {
    Int,
    Nat,
    Float64,
    Float32,
    Bool,
    Bytes,
    Unit,
}

// ---------------------------------------------------------------------------
// SizeTerm
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SizeTerm {
    Const(u64),
    Var(BoundVar),
    Add(Box<SizeTerm>, Box<SizeTerm>),
    Mul(u64, Box<SizeTerm>),
}

// ---------------------------------------------------------------------------
// Refinement predicates (LIA)
// ---------------------------------------------------------------------------

pub type RefinementPredicate = LIAFormula;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LIAFormula {
    True,
    False,
    And(Box<LIAFormula>, Box<LIAFormula>),
    Or(Box<LIAFormula>, Box<LIAFormula>),
    Not(Box<LIAFormula>),
    Implies(Box<LIAFormula>, Box<LIAFormula>),
    Atom(LIAAtom),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LIAAtom {
    Eq(LIATerm, LIATerm),
    Lt(LIATerm, LIATerm),
    Le(LIATerm, LIATerm),
    Divisible(LIATerm, u64),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LIATerm {
    Var(BoundVar),
    Const(i64),
    Add(Box<LIATerm>, Box<LIATerm>),
    Mul(i64, Box<LIATerm>),
    Neg(Box<LIATerm>),
    Len(BoundVar),
    Size(BoundVar),
    /// if cond then a else b (for rewriting abs, min, max)
    IfThenElse(Box<LIAFormula>, Box<LIATerm>, Box<LIATerm>),
    /// Modulo: a % b
    Mod(Box<LIATerm>, Box<LIATerm>),
}

// ---------------------------------------------------------------------------
// DecreaseWitness
// ---------------------------------------------------------------------------

/// Identifier for a proven well-founded order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WFOrderId(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DecreaseWitness {
    /// rec_arg is a strict subterm of outer_arg
    Structural(BoundVar, BoundVar),
    /// rec_measure < outer_measure
    Sized(LIATerm, LIATerm),
    /// Proven well-founded order (Tier 2)
    WellFounded(WFOrderId),
}
