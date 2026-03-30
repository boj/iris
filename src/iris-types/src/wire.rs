//! Binary wire format for IRIS fragments and bundles (SPEC Section 14).
//!
//! Canonical, deterministic, little-endian serialization. No serde — raw byte
//! manipulation ensures bit-for-bit reproducibility. This is the encoding
//! that `compute_fragment_id` hashes over.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use crate::cost::{CostBound, CostTerm, CostVar, HWParamRef, PotentialFn};
use crate::fragment::{Boundary, Fragment, FragmentId, FragmentMeta};
use crate::graph::{
    BinderId, Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution,
    RewriteRuleId, SemanticGraph,
};
use crate::guard::{BlobRef, ErrorBound, GuardSpec};
use crate::hash::SemanticHash;
use crate::proof::{ProofReceipt, VerifyTier};
use crate::types::{
    BoundVar, DecreaseWitness, LIAAtom, LIAFormula, LIATerm, PrimType, SizeTerm, Tag,
    TypeDef, TypeEnv, TypeId, WFOrderId,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Magic bytes: "IRIS" in ASCII (big-endian so it reads naturally in hex dump).
const FRAGMENT_MAGIC: [u8; 4] = [0x49, 0x52, 0x49, 0x53];

/// Magic bytes: "IRBD" for fragment bundles.
const BUNDLE_MAGIC: [u8; 4] = [0x49, 0x52, 0x42, 0x44];

/// Current wire format version.
const WIRE_VERSION: u16 = 1;

/// Flag: proof receipt is present.
const FLAG_HAS_PROOF: u16 = 0x0001;

// ---------------------------------------------------------------------------
// WireError
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum WireError {
    /// Not enough bytes remaining for the expected field.
    UnexpectedEof {
        expected: usize,
        remaining: usize,
        context: &'static str,
    },
    /// Magic bytes do not match.
    BadMagic { got: [u8; 4] },
    /// Unsupported wire format version.
    UnsupportedVersion { got: u16 },
    /// Invalid discriminant tag for an enum variant.
    InvalidTag {
        tag: u8,
        context: &'static str,
    },
    /// A length field would cause an out-of-bounds read.
    InvalidLength {
        length: usize,
        context: &'static str,
    },
    /// Generic decode error.
    Malformed(String),
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof {
                expected,
                remaining,
                context,
            } => write!(
                f,
                "unexpected EOF in {}: need {} bytes, have {}",
                context, expected, remaining
            ),
            Self::BadMagic { got } => write!(f, "bad magic: {:02x?}", got),
            Self::UnsupportedVersion { got } => write!(f, "unsupported version: {}", got),
            Self::InvalidTag { tag, context } => {
                write!(f, "invalid tag 0x{:02x} in {}", tag, context)
            }
            Self::InvalidLength { length, context } => {
                write!(f, "invalid length {} in {}", length, context)
            }
            Self::Malformed(msg) => write!(f, "malformed: {}", msg),
        }
    }
}

impl std::error::Error for WireError {}

// ===========================================================================
// Cursor helpers
// ===========================================================================

/// Read cursor tracking position into a byte slice.
struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    fn read_exact(&mut self, n: usize, context: &'static str) -> Result<&'a [u8], WireError> {
        if self.remaining() < n {
            return Err(WireError::UnexpectedEof {
                expected: n,
                remaining: self.remaining(),
                context,
            });
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    fn read_u8(&mut self, context: &'static str) -> Result<u8, WireError> {
        let b = self.read_exact(1, context)?;
        Ok(b[0])
    }

    fn read_u16(&mut self, context: &'static str) -> Result<u16, WireError> {
        let b = self.read_exact(2, context)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn read_u32(&mut self, context: &'static str) -> Result<u32, WireError> {
        let b = self.read_exact(4, context)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_u64(&mut self, context: &'static str) -> Result<u64, WireError> {
        let b = self.read_exact(8, context)?;
        Ok(u64::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_i64(&mut self, context: &'static str) -> Result<i64, WireError> {
        let b = self.read_exact(8, context)?;
        Ok(i64::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_f64(&mut self, context: &'static str) -> Result<f64, WireError> {
        let b = self.read_exact(8, context)?;
        Ok(f64::from_le_bytes(b.try_into().unwrap()))
    }

    fn read_bytes32(&mut self, context: &'static str) -> Result<[u8; 32], WireError> {
        let b = self.read_exact(32, context)?;
        Ok(b.try_into().unwrap())
    }

    /// Read a length-prefixed byte vec (4-byte length prefix).
    fn read_len_prefixed_bytes(&mut self, context: &'static str) -> Result<Vec<u8>, WireError> {
        let len = self.read_u32(context)? as usize;
        let b = self.read_exact(len, context)?;
        Ok(b.to_vec())
    }

    /// Read a length-prefixed string (4-byte length prefix).
    fn read_len_prefixed_string(&mut self, context: &'static str) -> Result<String, WireError> {
        let bytes = self.read_len_prefixed_bytes(context)?;
        String::from_utf8(bytes).map_err(|e| WireError::Malformed(format!("invalid UTF-8 in {}: {}", context, e)))
    }
}

// ===========================================================================
// Write helpers
// ===========================================================================

fn write_u8(buf: &mut Vec<u8>, v: u8) {
    buf.push(v);
}

fn write_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_u64(buf: &mut Vec<u8>, v: u64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_i64(buf: &mut Vec<u8>, v: i64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_f64(buf: &mut Vec<u8>, v: f64) {
    buf.extend_from_slice(&v.to_le_bytes());
}

fn write_bytes32(buf: &mut Vec<u8>, v: &[u8; 32]) {
    buf.extend_from_slice(v);
}

/// Write length-prefixed byte vec (4-byte length prefix).
fn write_len_prefixed_bytes(buf: &mut Vec<u8>, data: &[u8]) {
    write_u32(buf, data.len() as u32);
    buf.extend_from_slice(data);
}

/// Write length-prefixed string (4-byte length prefix).
fn write_len_prefixed_string(buf: &mut Vec<u8>, s: &str) {
    write_len_prefixed_bytes(buf, s.as_bytes());
}

// ===========================================================================
// CostBound serialization
// ===========================================================================

fn serialize_cost_bound(buf: &mut Vec<u8>, cb: &CostBound) {
    match cb {
        CostBound::Unknown => write_u8(buf, 0),
        CostBound::Zero => write_u8(buf, 1),
        CostBound::Constant(v) => {
            write_u8(buf, 2);
            write_u64(buf, *v);
        }
        CostBound::Linear(cv) => {
            write_u8(buf, 3);
            write_u32(buf, cv.0);
        }
        CostBound::NLogN(cv) => {
            write_u8(buf, 4);
            write_u32(buf, cv.0);
        }
        CostBound::Polynomial(cv, exp) => {
            write_u8(buf, 5);
            write_u32(buf, cv.0);
            write_u32(buf, *exp);
        }
        CostBound::Sum(a, b) => {
            write_u8(buf, 6);
            serialize_cost_bound(buf, a);
            serialize_cost_bound(buf, b);
        }
        CostBound::Par(a, b) => {
            write_u8(buf, 7);
            serialize_cost_bound(buf, a);
            serialize_cost_bound(buf, b);
        }
        CostBound::Mul(a, b) => {
            write_u8(buf, 8);
            serialize_cost_bound(buf, a);
            serialize_cost_bound(buf, b);
        }
        CostBound::Amortized(inner, pf) => {
            write_u8(buf, 9);
            serialize_cost_bound(buf, inner);
            write_len_prefixed_string(buf, &pf.description);
        }
        CostBound::HWScaled(inner, href) => {
            write_u8(buf, 10);
            serialize_cost_bound(buf, inner);
            write_bytes32(buf, &href.0);
        }
        CostBound::Sup(items) => {
            write_u8(buf, 11);
            write_u32(buf, items.len() as u32);
            for item in items {
                serialize_cost_bound(buf, item);
            }
        }
        CostBound::Inf(items) => {
            write_u8(buf, 12);
            write_u32(buf, items.len() as u32);
            for item in items {
                serialize_cost_bound(buf, item);
            }
        }
    }
}

fn deserialize_cost_bound(cur: &mut Cursor<'_>) -> Result<CostBound, WireError> {
    let tag = cur.read_u8("CostBound tag")?;
    match tag {
        0 => Ok(CostBound::Unknown),
        1 => Ok(CostBound::Zero),
        2 => Ok(CostBound::Constant(cur.read_u64("CostBound::Constant")?)),
        3 => Ok(CostBound::Linear(CostVar(cur.read_u32("CostBound::Linear")?))),
        4 => Ok(CostBound::NLogN(CostVar(cur.read_u32("CostBound::NLogN")?))),
        5 => {
            let cv = CostVar(cur.read_u32("CostBound::Polynomial var")?);
            let exp = cur.read_u32("CostBound::Polynomial exp")?;
            Ok(CostBound::Polynomial(cv, exp))
        }
        6 => {
            let a = deserialize_cost_bound(cur)?;
            let b = deserialize_cost_bound(cur)?;
            Ok(CostBound::Sum(Box::new(a), Box::new(b)))
        }
        7 => {
            let a = deserialize_cost_bound(cur)?;
            let b = deserialize_cost_bound(cur)?;
            Ok(CostBound::Par(Box::new(a), Box::new(b)))
        }
        8 => {
            let a = deserialize_cost_bound(cur)?;
            let b = deserialize_cost_bound(cur)?;
            Ok(CostBound::Mul(Box::new(a), Box::new(b)))
        }
        9 => {
            let inner = deserialize_cost_bound(cur)?;
            let desc = cur.read_len_prefixed_string("Amortized PotentialFn")?;
            Ok(CostBound::Amortized(
                Box::new(inner),
                PotentialFn { description: desc },
            ))
        }
        10 => {
            let inner = deserialize_cost_bound(cur)?;
            let href = cur.read_bytes32("HWScaled HWParamRef")?;
            Ok(CostBound::HWScaled(Box::new(inner), HWParamRef(href)))
        }
        11 => {
            let count = cur.read_u32("Sup count")? as usize;
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(deserialize_cost_bound(cur)?);
            }
            Ok(CostBound::Sup(items))
        }
        12 => {
            let count = cur.read_u32("Inf count")? as usize;
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(deserialize_cost_bound(cur)?);
            }
            Ok(CostBound::Inf(items))
        }
        _ => Err(WireError::InvalidTag {
            tag,
            context: "CostBound",
        }),
    }
}

// ===========================================================================
// CostTerm serialization
// ===========================================================================

fn serialize_cost_term(buf: &mut Vec<u8>, ct: &CostTerm) {
    match ct {
        CostTerm::Unit => write_u8(buf, 0),
        CostTerm::Inherited => write_u8(buf, 1),
        CostTerm::Annotated(cb) => {
            write_u8(buf, 2);
            serialize_cost_bound(buf, cb);
        }
    }
}

fn deserialize_cost_term(cur: &mut Cursor<'_>) -> Result<CostTerm, WireError> {
    let tag = cur.read_u8("CostTerm tag")?;
    match tag {
        0 => Ok(CostTerm::Unit),
        1 => Ok(CostTerm::Inherited),
        2 => Ok(CostTerm::Annotated(deserialize_cost_bound(cur)?)),
        _ => Err(WireError::InvalidTag {
            tag,
            context: "CostTerm",
        }),
    }
}

// ===========================================================================
// LIA formula/term serialization (for GuardSpec, DecreaseWitness, etc.)
// ===========================================================================

fn serialize_lia_term(buf: &mut Vec<u8>, term: &LIATerm) {
    match term {
        LIATerm::Var(bv) => {
            write_u8(buf, 0);
            write_u32(buf, bv.0);
        }
        LIATerm::Const(v) => {
            write_u8(buf, 1);
            write_i64(buf, *v);
        }
        LIATerm::Add(a, b) => {
            write_u8(buf, 2);
            serialize_lia_term(buf, a);
            serialize_lia_term(buf, b);
        }
        LIATerm::Mul(c, t) => {
            write_u8(buf, 3);
            write_i64(buf, *c);
            serialize_lia_term(buf, t);
        }
        LIATerm::Neg(t) => {
            write_u8(buf, 4);
            serialize_lia_term(buf, t);
        }
        LIATerm::Len(bv) => {
            write_u8(buf, 5);
            write_u32(buf, bv.0);
        }
        LIATerm::Size(bv) => {
            write_u8(buf, 6);
            write_u32(buf, bv.0);
        }
        LIATerm::IfThenElse(cond, then_t, else_t) => {
            write_u8(buf, 7);
            serialize_lia_formula(buf, cond);
            serialize_lia_term(buf, then_t);
            serialize_lia_term(buf, else_t);
        }
        LIATerm::Mod(a, b) => {
            write_u8(buf, 8);
            serialize_lia_term(buf, a);
            serialize_lia_term(buf, b);
        }
    }
}

fn deserialize_lia_term(cur: &mut Cursor<'_>) -> Result<LIATerm, WireError> {
    let tag = cur.read_u8("LIATerm tag")?;
    match tag {
        0 => Ok(LIATerm::Var(BoundVar(cur.read_u32("LIATerm::Var")?))),
        1 => Ok(LIATerm::Const(cur.read_i64("LIATerm::Const")?)),
        2 => {
            let a = deserialize_lia_term(cur)?;
            let b = deserialize_lia_term(cur)?;
            Ok(LIATerm::Add(Box::new(a), Box::new(b)))
        }
        3 => {
            let c = cur.read_i64("LIATerm::Mul coeff")?;
            let t = deserialize_lia_term(cur)?;
            Ok(LIATerm::Mul(c, Box::new(t)))
        }
        4 => {
            let t = deserialize_lia_term(cur)?;
            Ok(LIATerm::Neg(Box::new(t)))
        }
        5 => Ok(LIATerm::Len(BoundVar(cur.read_u32("LIATerm::Len")?))),
        6 => Ok(LIATerm::Size(BoundVar(cur.read_u32("LIATerm::Size")?))),
        7 => {
            let cond = deserialize_lia_formula(cur)?;
            let then_t = deserialize_lia_term(cur)?;
            let else_t = deserialize_lia_term(cur)?;
            Ok(LIATerm::IfThenElse(Box::new(cond), Box::new(then_t), Box::new(else_t)))
        }
        8 => {
            let a = deserialize_lia_term(cur)?;
            let b = deserialize_lia_term(cur)?;
            Ok(LIATerm::Mod(Box::new(a), Box::new(b)))
        }
        _ => Err(WireError::InvalidTag {
            tag,
            context: "LIATerm",
        }),
    }
}

fn serialize_lia_atom(buf: &mut Vec<u8>, atom: &LIAAtom) {
    match atom {
        LIAAtom::Eq(a, b) => {
            write_u8(buf, 0);
            serialize_lia_term(buf, a);
            serialize_lia_term(buf, b);
        }
        LIAAtom::Lt(a, b) => {
            write_u8(buf, 1);
            serialize_lia_term(buf, a);
            serialize_lia_term(buf, b);
        }
        LIAAtom::Le(a, b) => {
            write_u8(buf, 2);
            serialize_lia_term(buf, a);
            serialize_lia_term(buf, b);
        }
        LIAAtom::Divisible(t, d) => {
            write_u8(buf, 3);
            serialize_lia_term(buf, t);
            write_u64(buf, *d);
        }
    }
}

fn deserialize_lia_atom(cur: &mut Cursor<'_>) -> Result<LIAAtom, WireError> {
    let tag = cur.read_u8("LIAAtom tag")?;
    match tag {
        0 => {
            let a = deserialize_lia_term(cur)?;
            let b = deserialize_lia_term(cur)?;
            Ok(LIAAtom::Eq(a, b))
        }
        1 => {
            let a = deserialize_lia_term(cur)?;
            let b = deserialize_lia_term(cur)?;
            Ok(LIAAtom::Lt(a, b))
        }
        2 => {
            let a = deserialize_lia_term(cur)?;
            let b = deserialize_lia_term(cur)?;
            Ok(LIAAtom::Le(a, b))
        }
        3 => {
            let t = deserialize_lia_term(cur)?;
            let d = cur.read_u64("LIAAtom::Divisible")?;
            Ok(LIAAtom::Divisible(t, d))
        }
        _ => Err(WireError::InvalidTag {
            tag,
            context: "LIAAtom",
        }),
    }
}

fn serialize_lia_formula(buf: &mut Vec<u8>, formula: &LIAFormula) {
    match formula {
        LIAFormula::True => write_u8(buf, 0),
        LIAFormula::False => write_u8(buf, 1),
        LIAFormula::And(a, b) => {
            write_u8(buf, 2);
            serialize_lia_formula(buf, a);
            serialize_lia_formula(buf, b);
        }
        LIAFormula::Or(a, b) => {
            write_u8(buf, 3);
            serialize_lia_formula(buf, a);
            serialize_lia_formula(buf, b);
        }
        LIAFormula::Not(f) => {
            write_u8(buf, 4);
            serialize_lia_formula(buf, f);
        }
        LIAFormula::Implies(a, b) => {
            write_u8(buf, 5);
            serialize_lia_formula(buf, a);
            serialize_lia_formula(buf, b);
        }
        LIAFormula::Atom(atom) => {
            write_u8(buf, 6);
            serialize_lia_atom(buf, atom);
        }
    }
}

fn deserialize_lia_formula(cur: &mut Cursor<'_>) -> Result<LIAFormula, WireError> {
    let tag = cur.read_u8("LIAFormula tag")?;
    match tag {
        0 => Ok(LIAFormula::True),
        1 => Ok(LIAFormula::False),
        2 => {
            let a = deserialize_lia_formula(cur)?;
            let b = deserialize_lia_formula(cur)?;
            Ok(LIAFormula::And(Box::new(a), Box::new(b)))
        }
        3 => {
            let a = deserialize_lia_formula(cur)?;
            let b = deserialize_lia_formula(cur)?;
            Ok(LIAFormula::Or(Box::new(a), Box::new(b)))
        }
        4 => {
            let f = deserialize_lia_formula(cur)?;
            Ok(LIAFormula::Not(Box::new(f)))
        }
        5 => {
            let a = deserialize_lia_formula(cur)?;
            let b = deserialize_lia_formula(cur)?;
            Ok(LIAFormula::Implies(Box::new(a), Box::new(b)))
        }
        6 => {
            let atom = deserialize_lia_atom(cur)?;
            Ok(LIAFormula::Atom(atom))
        }
        _ => Err(WireError::InvalidTag {
            tag,
            context: "LIAFormula",
        }),
    }
}

// ===========================================================================
// SizeTerm serialization
// ===========================================================================

fn serialize_size_term(buf: &mut Vec<u8>, st: &SizeTerm) {
    match st {
        SizeTerm::Const(v) => {
            write_u8(buf, 0);
            write_u64(buf, *v);
        }
        SizeTerm::Var(bv) => {
            write_u8(buf, 1);
            write_u32(buf, bv.0);
        }
        SizeTerm::Add(a, b) => {
            write_u8(buf, 2);
            serialize_size_term(buf, a);
            serialize_size_term(buf, b);
        }
        SizeTerm::Mul(c, t) => {
            write_u8(buf, 3);
            write_u64(buf, *c);
            serialize_size_term(buf, t);
        }
    }
}

fn deserialize_size_term(cur: &mut Cursor<'_>) -> Result<SizeTerm, WireError> {
    let tag = cur.read_u8("SizeTerm tag")?;
    match tag {
        0 => Ok(SizeTerm::Const(cur.read_u64("SizeTerm::Const")?)),
        1 => Ok(SizeTerm::Var(BoundVar(cur.read_u32("SizeTerm::Var")?))),
        2 => {
            let a = deserialize_size_term(cur)?;
            let b = deserialize_size_term(cur)?;
            Ok(SizeTerm::Add(Box::new(a), Box::new(b)))
        }
        3 => {
            let c = cur.read_u64("SizeTerm::Mul coeff")?;
            let t = deserialize_size_term(cur)?;
            Ok(SizeTerm::Mul(c, Box::new(t)))
        }
        _ => Err(WireError::InvalidTag {
            tag,
            context: "SizeTerm",
        }),
    }
}

// ===========================================================================
// DecreaseWitness serialization
// ===========================================================================

fn serialize_decrease_witness(buf: &mut Vec<u8>, dw: &DecreaseWitness) {
    match dw {
        DecreaseWitness::Structural(a, b) => {
            write_u8(buf, 0);
            write_u32(buf, a.0);
            write_u32(buf, b.0);
        }
        DecreaseWitness::Sized(a, b) => {
            write_u8(buf, 1);
            serialize_lia_term(buf, a);
            serialize_lia_term(buf, b);
        }
        DecreaseWitness::WellFounded(wfo) => {
            write_u8(buf, 2);
            write_bytes32(buf, &wfo.0);
        }
    }
}

fn deserialize_decrease_witness(cur: &mut Cursor<'_>) -> Result<DecreaseWitness, WireError> {
    let tag = cur.read_u8("DecreaseWitness tag")?;
    match tag {
        0 => {
            let a = BoundVar(cur.read_u32("DecreaseWitness::Structural a")?);
            let b = BoundVar(cur.read_u32("DecreaseWitness::Structural b")?);
            Ok(DecreaseWitness::Structural(a, b))
        }
        1 => {
            let a = deserialize_lia_term(cur)?;
            let b = deserialize_lia_term(cur)?;
            Ok(DecreaseWitness::Sized(a, b))
        }
        2 => {
            let wfo = cur.read_bytes32("DecreaseWitness::WellFounded")?;
            Ok(DecreaseWitness::WellFounded(WFOrderId(wfo)))
        }
        _ => Err(WireError::InvalidTag {
            tag,
            context: "DecreaseWitness",
        }),
    }
}

// ===========================================================================
// ErrorBound serialization
// ===========================================================================

fn serialize_error_bound(buf: &mut Vec<u8>, eb: &ErrorBound) {
    match eb {
        ErrorBound::Exact => write_u8(buf, 0),
        ErrorBound::Statistical {
            confidence,
            epsilon,
        } => {
            write_u8(buf, 1);
            write_f64(buf, *confidence);
            write_f64(buf, *epsilon);
        }
        ErrorBound::Classification { accuracy } => {
            write_u8(buf, 2);
            write_f64(buf, *accuracy);
        }
        ErrorBound::Unverified => write_u8(buf, 3),
    }
}

fn deserialize_error_bound(cur: &mut Cursor<'_>) -> Result<ErrorBound, WireError> {
    let tag = cur.read_u8("ErrorBound tag")?;
    match tag {
        0 => Ok(ErrorBound::Exact),
        1 => {
            let confidence = cur.read_f64("ErrorBound confidence")?;
            let epsilon = cur.read_f64("ErrorBound epsilon")?;
            Ok(ErrorBound::Statistical {
                confidence,
                epsilon,
            })
        }
        2 => {
            let accuracy = cur.read_f64("ErrorBound accuracy")?;
            Ok(ErrorBound::Classification { accuracy })
        }
        3 => Ok(ErrorBound::Unverified),
        _ => Err(WireError::InvalidTag {
            tag,
            context: "ErrorBound",
        }),
    }
}

// ===========================================================================
// GuardSpec serialization
// ===========================================================================

fn serialize_guard_spec(buf: &mut Vec<u8>, gs: &GuardSpec) {
    write_u64(buf, gs.input_type.0);
    write_u64(buf, gs.output_type.0);
    // preconditions
    write_u32(buf, gs.preconditions.len() as u32);
    for p in &gs.preconditions {
        serialize_lia_formula(buf, p);
    }
    // postconditions
    write_u32(buf, gs.postconditions.len() as u32);
    for p in &gs.postconditions {
        serialize_lia_formula(buf, p);
    }
    serialize_error_bound(buf, &gs.error_bound);
    // fallback
    match &gs.fallback {
        Some(fref) => {
            write_u8(buf, 1);
            write_bytes32(buf, &fref.0);
        }
        None => write_u8(buf, 0),
    }
}

fn deserialize_guard_spec(cur: &mut Cursor<'_>) -> Result<GuardSpec, WireError> {
    let input_type = TypeId(cur.read_u64("GuardSpec input_type")?);
    let output_type = TypeId(cur.read_u64("GuardSpec output_type")?);

    let pre_count = cur.read_u32("GuardSpec preconditions count")? as usize;
    let mut preconditions = Vec::with_capacity(pre_count);
    for _ in 0..pre_count {
        preconditions.push(deserialize_lia_formula(cur)?);
    }

    let post_count = cur.read_u32("GuardSpec postconditions count")? as usize;
    let mut postconditions = Vec::with_capacity(post_count);
    for _ in 0..post_count {
        postconditions.push(deserialize_lia_formula(cur)?);
    }

    let error_bound = deserialize_error_bound(cur)?;

    let has_fallback = cur.read_u8("GuardSpec fallback flag")?;
    let fallback = if has_fallback == 1 {
        Some(FragmentId(cur.read_bytes32("GuardSpec fallback")?))
    } else {
        None
    };

    Ok(GuardSpec {
        input_type,
        output_type,
        preconditions,
        postconditions,
        error_bound,
        fallback,
    })
}

// ===========================================================================
// TypeDef serialization
// ===========================================================================

fn serialize_type_def(buf: &mut Vec<u8>, td: &TypeDef) {
    match td {
        TypeDef::Primitive(p) => {
            write_u8(buf, 0);
            write_u8(buf, *p as u8);
        }
        TypeDef::Product(fields) => {
            write_u8(buf, 1);
            write_u32(buf, fields.len() as u32);
            for f in fields {
                write_u64(buf, f.0);
            }
        }
        TypeDef::Sum(variants) => {
            write_u8(buf, 2);
            write_u32(buf, variants.len() as u32);
            for (tag, tid) in variants {
                write_u16(buf, tag.0);
                write_u64(buf, tid.0);
            }
        }
        TypeDef::Recursive(bv, tid) => {
            write_u8(buf, 3);
            write_u32(buf, bv.0);
            write_u64(buf, tid.0);
        }
        TypeDef::ForAll(bv, tid) => {
            write_u8(buf, 4);
            write_u32(buf, bv.0);
            write_u64(buf, tid.0);
        }
        TypeDef::Arrow(a, b, cost) => {
            write_u8(buf, 5);
            write_u64(buf, a.0);
            write_u64(buf, b.0);
            serialize_cost_bound(buf, cost);
        }
        TypeDef::Refined(tid, pred) => {
            write_u8(buf, 6);
            write_u64(buf, tid.0);
            serialize_lia_formula(buf, pred);
        }
        TypeDef::NeuralGuard(tin, tout, guard, cost) => {
            write_u8(buf, 7);
            write_u64(buf, tin.0);
            write_u64(buf, tout.0);
            serialize_guard_spec(buf, guard);
            serialize_cost_bound(buf, cost);
        }
        TypeDef::Exists(bv, tid) => {
            write_u8(buf, 8);
            write_u32(buf, bv.0);
            write_u64(buf, tid.0);
        }
        TypeDef::Vec(tid, size) => {
            write_u8(buf, 9);
            write_u64(buf, tid.0);
            serialize_size_term(buf, size);
        }
        TypeDef::HWParam(tid, profile) => {
            write_u8(buf, 10);
            write_u64(buf, tid.0);
            serialize_hw_profile(buf, profile);
        }
    }
}

fn deserialize_type_def(cur: &mut Cursor<'_>) -> Result<TypeDef, WireError> {
    let tag = cur.read_u8("TypeDef tag")?;
    match tag {
        0 => {
            let p = cur.read_u8("PrimType")?;
            let prim = match p {
                0 => PrimType::Int,
                1 => PrimType::Nat,
                2 => PrimType::Float64,
                3 => PrimType::Float32,
                4 => PrimType::Bool,
                5 => PrimType::Bytes,
                6 => PrimType::Unit,
                _ => {
                    return Err(WireError::InvalidTag {
                        tag: p,
                        context: "PrimType",
                    })
                }
            };
            Ok(TypeDef::Primitive(prim))
        }
        1 => {
            let count = cur.read_u32("Product count")? as usize;
            let mut fields = Vec::with_capacity(count);
            for _ in 0..count {
                fields.push(TypeId(cur.read_u64("Product field")?));
            }
            Ok(TypeDef::Product(fields))
        }
        2 => {
            let count = cur.read_u32("Sum count")? as usize;
            let mut variants = Vec::with_capacity(count);
            for _ in 0..count {
                let t = Tag(cur.read_u16("Sum tag")?);
                let tid = TypeId(cur.read_u64("Sum type")?);
                variants.push((t, tid));
            }
            Ok(TypeDef::Sum(variants))
        }
        3 => {
            let bv = BoundVar(cur.read_u32("Recursive bv")?);
            let tid = TypeId(cur.read_u64("Recursive tid")?);
            Ok(TypeDef::Recursive(bv, tid))
        }
        4 => {
            let bv = BoundVar(cur.read_u32("ForAll bv")?);
            let tid = TypeId(cur.read_u64("ForAll tid")?);
            Ok(TypeDef::ForAll(bv, tid))
        }
        5 => {
            let a = TypeId(cur.read_u64("Arrow input")?);
            let b = TypeId(cur.read_u64("Arrow output")?);
            let cost = deserialize_cost_bound(cur)?;
            Ok(TypeDef::Arrow(a, b, cost))
        }
        6 => {
            let tid = TypeId(cur.read_u64("Refined tid")?);
            let pred = deserialize_lia_formula(cur)?;
            Ok(TypeDef::Refined(tid, pred))
        }
        7 => {
            let tin = TypeId(cur.read_u64("NeuralGuard in")?);
            let tout = TypeId(cur.read_u64("NeuralGuard out")?);
            let guard = deserialize_guard_spec(cur)?;
            let cost = deserialize_cost_bound(cur)?;
            Ok(TypeDef::NeuralGuard(tin, tout, guard, cost))
        }
        8 => {
            let bv = BoundVar(cur.read_u32("Exists bv")?);
            let tid = TypeId(cur.read_u64("Exists tid")?);
            Ok(TypeDef::Exists(bv, tid))
        }
        9 => {
            let tid = TypeId(cur.read_u64("Vec tid")?);
            let size = deserialize_size_term(cur)?;
            Ok(TypeDef::Vec(tid, size))
        }
        10 => {
            let tid = TypeId(cur.read_u64("HWParam tid")?);
            let profile = deserialize_hw_profile(cur)?;
            Ok(TypeDef::HWParam(tid, profile))
        }
        _ => Err(WireError::InvalidTag {
            tag,
            context: "TypeDef",
        }),
    }
}

// ===========================================================================
// HardwareProfile serialization
// ===========================================================================

fn serialize_hw_profile(buf: &mut Vec<u8>, profile: &crate::cost::HardwareProfile) {
    write_len_prefixed_string(buf, &profile.name);
    write_u32(buf, profile.params.len() as u32);
    for (k, v) in &profile.params {
        write_len_prefixed_string(buf, k);
        write_u64(buf, *v);
    }
    write_u32(buf, profile.constraints.len() as u32);
    for c in &profile.constraints {
        serialize_lia_formula(buf, c);
    }
    write_u32(buf, profile.axioms.len() as u32);
    for a in &profile.axioms {
        write_len_prefixed_string(buf, &a.name);
        serialize_lia_formula(buf, &a.formula);
    }
}

fn deserialize_hw_profile(cur: &mut Cursor<'_>) -> Result<crate::cost::HardwareProfile, WireError> {
    let name = cur.read_len_prefixed_string("HardwareProfile name")?;
    let params_count = cur.read_u32("HardwareProfile params count")? as usize;
    let mut params = BTreeMap::new();
    for _ in 0..params_count {
        let k = cur.read_len_prefixed_string("HardwareProfile param key")?;
        let v = cur.read_u64("HardwareProfile param value")?;
        params.insert(k, v);
    }
    let constraints_count = cur.read_u32("HardwareProfile constraints count")? as usize;
    let mut constraints = Vec::with_capacity(constraints_count);
    for _ in 0..constraints_count {
        constraints.push(deserialize_lia_formula(cur)?);
    }
    let axioms_count = cur.read_u32("HardwareProfile axioms count")? as usize;
    let mut axioms = Vec::with_capacity(axioms_count);
    for _ in 0..axioms_count {
        let aname = cur.read_len_prefixed_string("CostAxiom name")?;
        let formula = deserialize_lia_formula(cur)?;
        axioms.push(crate::cost::CostAxiom {
            name: aname,
            formula,
        });
    }
    Ok(crate::cost::HardwareProfile {
        name,
        params,
        constraints,
        axioms,
    })
}

// ===========================================================================
// NodePayload serialization
// ===========================================================================

fn serialize_node_payload(buf: &mut Vec<u8>, payload: &NodePayload) {
    match payload {
        NodePayload::Prim { opcode } => {
            write_u8(buf, 0x00);
            write_u8(buf, *opcode);
        }
        NodePayload::Apply => {
            write_u8(buf, 0x01);
        }
        NodePayload::Lambda {
            binder,
            captured_count,
        } => {
            write_u8(buf, 0x02);
            write_u32(buf, binder.0);
            write_u32(buf, *captured_count);
        }
        NodePayload::Let => {
            write_u8(buf, 0x03);
        }
        NodePayload::Match {
            arm_count,
            arm_patterns,
        } => {
            write_u8(buf, 0x04);
            write_u16(buf, *arm_count);
            write_len_prefixed_bytes(buf, arm_patterns);
        }
        NodePayload::Lit { type_tag, value } => {
            write_u8(buf, 0x05);
            write_u8(buf, *type_tag);
            write_len_prefixed_bytes(buf, value);
        }
        NodePayload::Ref { fragment_id } => {
            write_u8(buf, 0x06);
            write_bytes32(buf, &fragment_id.0);
        }
        NodePayload::Neural {
            guard_spec,
            weight_blob,
        } => {
            write_u8(buf, 0x07);
            serialize_guard_spec(buf, guard_spec);
            write_bytes32(buf, &weight_blob.hash);
            write_u64(buf, weight_blob.size);
        }
        NodePayload::Fold {
            recursion_descriptor,
        } => {
            write_u8(buf, 0x08);
            write_len_prefixed_bytes(buf, recursion_descriptor);
        }
        NodePayload::Unfold {
            recursion_descriptor,
        } => {
            write_u8(buf, 0x09);
            write_len_prefixed_bytes(buf, recursion_descriptor);
        }
        NodePayload::Effect { effect_tag } => {
            write_u8(buf, 0x0A);
            write_u8(buf, *effect_tag);
        }
        NodePayload::Tuple => {
            write_u8(buf, 0x0B);
        }
        NodePayload::Inject { tag_index } => {
            write_u8(buf, 0x0C);
            write_u16(buf, *tag_index);
        }
        NodePayload::Project { field_index } => {
            write_u8(buf, 0x0D);
            write_u16(buf, *field_index);
        }
        NodePayload::TypeAbst { bound_var_id } => {
            write_u8(buf, 0x0E);
            write_u32(buf, bound_var_id.0);
        }
        NodePayload::TypeApp { type_arg } => {
            write_u8(buf, 0x0F);
            write_u64(buf, type_arg.0);
        }
        NodePayload::LetRec { binder, decrease } => {
            write_u8(buf, 0x10);
            write_u32(buf, binder.0);
            serialize_decrease_witness(buf, decrease);
        }
        NodePayload::Guard {
            predicate_node,
            body_node,
            fallback_node,
        } => {
            write_u8(buf, 0x11);
            write_u64(buf, predicate_node.0);
            write_u64(buf, body_node.0);
            write_u64(buf, fallback_node.0);
        }
        NodePayload::Rewrite { rule_id, body } => {
            write_u8(buf, 0x12);
            write_bytes32(buf, &rule_id.0);
            write_u64(buf, body.0);
        }
        NodePayload::Extern { name, type_sig } => {
            write_u8(buf, 0x13);
            write_bytes32(buf, name);
            write_u64(buf, type_sig.0);
        }
    }
}

fn deserialize_node_payload(cur: &mut Cursor<'_>) -> Result<NodePayload, WireError> {
    let tag = cur.read_u8("NodePayload tag")?;
    match tag {
        0x00 => {
            let opcode = cur.read_u8("Prim opcode")?;
            Ok(NodePayload::Prim { opcode })
        }
        0x01 => Ok(NodePayload::Apply),
        0x02 => {
            let binder = BinderId(cur.read_u32("Lambda binder")?);
            let captured_count = cur.read_u32("Lambda captured_count")?;
            Ok(NodePayload::Lambda {
                binder,
                captured_count,
            })
        }
        0x03 => Ok(NodePayload::Let),
        0x04 => {
            let arm_count = cur.read_u16("Match arm_count")?;
            let arm_patterns = cur.read_len_prefixed_bytes("Match arm_patterns")?;
            Ok(NodePayload::Match {
                arm_count,
                arm_patterns,
            })
        }
        0x05 => {
            let type_tag = cur.read_u8("Lit type_tag")?;
            let value = cur.read_len_prefixed_bytes("Lit value")?;
            Ok(NodePayload::Lit { type_tag, value })
        }
        0x06 => {
            let fragment_id = FragmentId(cur.read_bytes32("Ref fragment_id")?);
            Ok(NodePayload::Ref { fragment_id })
        }
        0x07 => {
            let guard_spec = deserialize_guard_spec(cur)?;
            let hash = cur.read_bytes32("Neural weight_blob hash")?;
            let size = cur.read_u64("Neural weight_blob size")?;
            Ok(NodePayload::Neural {
                guard_spec,
                weight_blob: BlobRef { hash, size },
            })
        }
        0x08 => {
            let recursion_descriptor = cur.read_len_prefixed_bytes("Fold recursion_descriptor")?;
            Ok(NodePayload::Fold {
                recursion_descriptor,
            })
        }
        0x09 => {
            let recursion_descriptor =
                cur.read_len_prefixed_bytes("Unfold recursion_descriptor")?;
            Ok(NodePayload::Unfold {
                recursion_descriptor,
            })
        }
        0x0A => {
            let effect_tag = cur.read_u8("Effect effect_tag")?;
            Ok(NodePayload::Effect { effect_tag })
        }
        0x0B => Ok(NodePayload::Tuple),
        0x0C => {
            let tag_index = cur.read_u16("Inject tag_index")?;
            Ok(NodePayload::Inject { tag_index })
        }
        0x0D => {
            let field_index = cur.read_u16("Project field_index")?;
            Ok(NodePayload::Project { field_index })
        }
        0x0E => {
            let bound_var_id = BoundVar(cur.read_u32("TypeAbst bound_var_id")?);
            Ok(NodePayload::TypeAbst { bound_var_id })
        }
        0x0F => {
            let type_arg = TypeId(cur.read_u64("TypeApp type_arg")?);
            Ok(NodePayload::TypeApp { type_arg })
        }
        0x10 => {
            let binder = BinderId(cur.read_u32("LetRec binder")?);
            let decrease = deserialize_decrease_witness(cur)?;
            Ok(NodePayload::LetRec { binder, decrease })
        }
        0x11 => {
            let predicate_node = NodeId(cur.read_u64("Guard predicate_node")?);
            let body_node = NodeId(cur.read_u64("Guard body_node")?);
            let fallback_node = NodeId(cur.read_u64("Guard fallback_node")?);
            Ok(NodePayload::Guard {
                predicate_node,
                body_node,
                fallback_node,
            })
        }
        0x12 => {
            let rule_id = RewriteRuleId(cur.read_bytes32("Rewrite rule_id")?);
            let body = NodeId(cur.read_u64("Rewrite body")?);
            Ok(NodePayload::Rewrite { rule_id, body })
        }
        0x13 => {
            let name = cur.read_bytes32("Extern name")?;
            let type_sig = TypeId(cur.read_u64("Extern type_sig")?);
            Ok(NodePayload::Extern { name, type_sig })
        }
        _ => Err(WireError::InvalidTag {
            tag,
            context: "NodePayload",
        }),
    }
}

// ===========================================================================
// Node serialization
// ===========================================================================

fn serialize_node(buf: &mut Vec<u8>, node: &Node) {
    write_u64(buf, node.id.0);
    write_u8(buf, node.kind as u8);
    write_u64(buf, node.type_sig.0);
    write_u8(buf, node.arity);
    write_u8(buf, node.resolution_depth);
    serialize_cost_term(buf, &node.cost);

    // Payload: write into a temp buffer to get length, then write length-prefixed.
    let mut payload_buf = Vec::new();
    serialize_node_payload(&mut payload_buf, &node.payload);
    write_u16(buf, payload_buf.len() as u16);
    buf.extend_from_slice(&payload_buf);
}

fn deserialize_node(cur: &mut Cursor<'_>) -> Result<Node, WireError> {
    let id = NodeId(cur.read_u64("Node id")?);
    let kind_byte = cur.read_u8("Node kind")?;
    let kind = match kind_byte {
        0x00 => NodeKind::Prim,
        0x01 => NodeKind::Apply,
        0x02 => NodeKind::Lambda,
        0x03 => NodeKind::Let,
        0x04 => NodeKind::Match,
        0x05 => NodeKind::Lit,
        0x06 => NodeKind::Ref,
        0x07 => NodeKind::Neural,
        0x08 => NodeKind::Fold,
        0x09 => NodeKind::Unfold,
        0x0A => NodeKind::Effect,
        0x0B => NodeKind::Tuple,
        0x0C => NodeKind::Inject,
        0x0D => NodeKind::Project,
        0x0E => NodeKind::TypeAbst,
        0x0F => NodeKind::TypeApp,
        0x10 => NodeKind::LetRec,
        0x11 => NodeKind::Guard,
        0x12 => NodeKind::Rewrite,
        0x13 => NodeKind::Extern,
        _ => {
            return Err(WireError::InvalidTag {
                tag: kind_byte,
                context: "NodeKind",
            })
        }
    };
    let type_sig = TypeId(cur.read_u64("Node type_sig")?);
    let arity = cur.read_u8("Node arity")?;
    let resolution_depth = cur.read_u8("Node resolution_depth")?;
    let cost = deserialize_cost_term(cur)?;

    let payload_len = cur.read_u16("Node payload_len")? as usize;
    let payload_bytes = cur.read_exact(payload_len, "Node payload")?;
    let mut payload_cur = Cursor::new(payload_bytes);
    let payload = deserialize_node_payload(&mut payload_cur)?;

    Ok(Node {
        id,
        kind,
        type_sig,
        cost,
        arity,
        resolution_depth,
        salt: 0,
        payload,
    })
}

// ===========================================================================
// Edge serialization
// ===========================================================================

fn serialize_edge(buf: &mut Vec<u8>, edge: &Edge) {
    write_u64(buf, edge.source.0);
    write_u64(buf, edge.target.0);
    write_u8(buf, edge.port);
    write_u8(buf, edge.label as u8);
}

fn deserialize_edge(cur: &mut Cursor<'_>) -> Result<Edge, WireError> {
    let source = NodeId(cur.read_u64("Edge source")?);
    let target = NodeId(cur.read_u64("Edge target")?);
    let port = cur.read_u8("Edge port")?;
    let label_byte = cur.read_u8("Edge label")?;
    let label = match label_byte {
        0 => EdgeLabel::Argument,
        1 => EdgeLabel::Scrutinee,
        2 => EdgeLabel::Binding,
        3 => EdgeLabel::Continuation,
        4 => EdgeLabel::Decrease,
        _ => {
            return Err(WireError::InvalidTag {
                tag: label_byte,
                context: "EdgeLabel",
            })
        }
    };
    Ok(Edge {
        source,
        target,
        port,
        label,
    })
}

// ===========================================================================
// Resolution serialization
// ===========================================================================

fn serialize_resolution(buf: &mut Vec<u8>, res: &Resolution) {
    match res {
        Resolution::Intent => write_u8(buf, 0),
        Resolution::Architecture => write_u8(buf, 1),
        Resolution::Implementation => write_u8(buf, 2),
    }
}

fn deserialize_resolution(cur: &mut Cursor<'_>) -> Result<Resolution, WireError> {
    let tag = cur.read_u8("Resolution")?;
    match tag {
        0 => Ok(Resolution::Intent),
        1 => Ok(Resolution::Architecture),
        2 => Ok(Resolution::Implementation),
        _ => Err(WireError::InvalidTag {
            tag,
            context: "Resolution",
        }),
    }
}

// ===========================================================================
// ProofReceipt serialization
// ===========================================================================

fn serialize_proof_receipt(buf: &mut Vec<u8>, pr: &ProofReceipt) {
    write_bytes32(buf, &pr.graph_hash.0);
    write_u64(buf, pr.type_sig.0);
    serialize_cost_bound(buf, &pr.cost_bound);
    write_u8(buf, pr.tier as u8);
    write_bytes32(buf, &pr.proof_merkle_root);
    write_len_prefixed_bytes(buf, &pr.compact_witness);
}

fn deserialize_proof_receipt(cur: &mut Cursor<'_>) -> Result<ProofReceipt, WireError> {
    let graph_hash = FragmentId(cur.read_bytes32("ProofReceipt graph_hash")?);
    let type_sig = TypeId(cur.read_u64("ProofReceipt type_sig")?);
    let cost_bound = deserialize_cost_bound(cur)?;
    let tier_byte = cur.read_u8("ProofReceipt tier")?;
    let tier = match tier_byte {
        0 => VerifyTier::Tier0,
        1 => VerifyTier::Tier1,
        2 => VerifyTier::Tier2,
        3 => VerifyTier::Tier3,
        _ => {
            return Err(WireError::InvalidTag {
                tag: tier_byte,
                context: "VerifyTier",
            })
        }
    };
    let proof_merkle_root = cur.read_bytes32("ProofReceipt proof_merkle_root")?;
    let compact_witness = cur.read_len_prefixed_bytes("ProofReceipt compact_witness")?;
    Ok(ProofReceipt {
        graph_hash,
        type_sig,
        cost_bound,
        tier,
        proof_merkle_root,
        compact_witness,
    })
}

// ===========================================================================
// Fragment serialization (SPEC Section 14.1)
// ===========================================================================

/// Serialize a fragment to canonical binary wire format.
///
/// Layout:
/// ```text
/// Magic: 4B (0x49524953 "IRIS") | Version: 2B | Header len: 4B
/// FragmentId: 32B | Flags: 2B | Resolution: 1B
/// TypeEnv section | Node section | Edge section
/// Boundary section | Import section | SemanticHash: 32B
/// Optional: ProofReceipt section
/// Metadata section
/// ```
pub fn serialize_fragment(fragment: &Fragment) -> Vec<u8> {
    let mut buf = Vec::with_capacity(512);

    // -- Header --
    buf.extend_from_slice(&FRAGMENT_MAGIC);
    write_u16(&mut buf, WIRE_VERSION);

    // Reserve 4 bytes for header length (will be filled after header is complete).
    let header_len_offset = buf.len();
    write_u32(&mut buf, 0); // placeholder

    // FragmentId
    write_bytes32(&mut buf, &fragment.id.0);

    // Flags
    let mut flags: u16 = 0;
    if fragment.proof.is_some() {
        flags |= FLAG_HAS_PROOF;
    }
    write_u16(&mut buf, flags);

    // Resolution
    serialize_resolution(&mut buf, &fragment.graph.resolution);

    // Record header length (everything up to here).
    let header_len = buf.len() as u32;
    buf[header_len_offset..header_len_offset + 4].copy_from_slice(&header_len.to_le_bytes());

    // -- TypeEnv section --
    let type_env = &fragment.type_env;
    write_u32(&mut buf, type_env.types.len() as u32);
    for (tid, tdef) in &type_env.types {
        write_u64(&mut buf, tid.0);
        serialize_type_def(&mut buf, tdef);
    }

    // -- Node section --
    let nodes = &fragment.graph.nodes;
    write_u32(&mut buf, nodes.len() as u32);
    // Sort by NodeId for deterministic canonical serialization.
    let mut sorted_nodes: Vec<_> = nodes.iter().collect();
    sorted_nodes.sort_by_key(|(id, _)| *id);
    for (_nid, node) in sorted_nodes {
        serialize_node(&mut buf, node);
    }

    // -- Edge section --
    let edges = &fragment.graph.edges;
    write_u32(&mut buf, edges.len() as u32);
    for edge in edges {
        serialize_edge(&mut buf, edge);
    }

    // -- Boundary section --
    write_u32(&mut buf, fragment.boundary.inputs.len() as u32);
    for (nid, tref) in &fragment.boundary.inputs {
        write_u64(&mut buf, nid.0);
        write_u64(&mut buf, tref.0);
    }
    write_u32(&mut buf, fragment.boundary.outputs.len() as u32);
    for (nid, tref) in &fragment.boundary.outputs {
        write_u64(&mut buf, nid.0);
        write_u64(&mut buf, tref.0);
    }

    // -- Import section --
    write_u16(&mut buf, fragment.imports.len() as u16);
    for imp in &fragment.imports {
        write_bytes32(&mut buf, &imp.0);
    }

    // -- SemanticHash --
    write_bytes32(&mut buf, &fragment.graph.hash.0);

    // -- Root NodeId --
    write_u64(&mut buf, fragment.graph.root.0);

    // -- Graph CostBound --
    serialize_cost_bound(&mut buf, &fragment.graph.cost);

    // -- Graph TypeEnv (inline; same serialization) --
    write_u32(&mut buf, fragment.graph.type_env.types.len() as u32);
    for (tid, tdef) in &fragment.graph.type_env.types {
        write_u64(&mut buf, tid.0);
        serialize_type_def(&mut buf, tdef);
    }

    // -- Optional ProofReceipt --
    if let Some(proof) = &fragment.proof {
        serialize_proof_receipt(&mut buf, proof);
    }

    // -- Metadata --
    serialize_metadata(&mut buf, &fragment.metadata);

    buf
}

/// Deserialize a fragment from canonical binary wire format.
pub fn deserialize_fragment(bytes: &[u8]) -> Result<Fragment, WireError> {
    let mut cur = Cursor::new(bytes);

    // -- Header --
    let magic_bytes = cur.read_exact(4, "fragment magic")?;
    let magic: [u8; 4] = magic_bytes.try_into().unwrap();
    if magic != FRAGMENT_MAGIC {
        return Err(WireError::BadMagic { got: magic });
    }

    let version = cur.read_u16("fragment version")?;
    if version != WIRE_VERSION {
        return Err(WireError::UnsupportedVersion { got: version });
    }

    let _header_len = cur.read_u32("fragment header_len")?;

    let fragment_id = FragmentId(cur.read_bytes32("FragmentId")?);

    let flags = cur.read_u16("fragment flags")?;
    let has_proof = (flags & FLAG_HAS_PROOF) != 0;

    let resolution = deserialize_resolution(&mut cur)?;

    // -- TypeEnv section --
    let type_count = cur.read_u32("TypeEnv count")? as usize;
    let mut types = BTreeMap::new();
    for _ in 0..type_count {
        let tid = TypeId(cur.read_u64("TypeEnv TypeId")?);
        let tdef = deserialize_type_def(&mut cur)?;
        types.insert(tid, tdef);
    }
    let type_env = TypeEnv { types };

    // -- Node section --
    let node_count = cur.read_u32("Node count")? as usize;
    let mut nodes = HashMap::new();
    for _ in 0..node_count {
        let node = deserialize_node(&mut cur)?;
        nodes.insert(node.id, node);
    }

    // -- Edge section --
    let edge_count = cur.read_u32("Edge count")? as usize;
    let mut edges = Vec::with_capacity(edge_count);
    for _ in 0..edge_count {
        edges.push(deserialize_edge(&mut cur)?);
    }

    // -- Boundary section --
    let input_count = cur.read_u32("Boundary inputs count")? as usize;
    let mut inputs = Vec::with_capacity(input_count);
    for _ in 0..input_count {
        let nid = NodeId(cur.read_u64("Boundary input NodeId")?);
        let tref = TypeId(cur.read_u64("Boundary input TypeRef")?);
        inputs.push((nid, tref));
    }
    let output_count = cur.read_u32("Boundary outputs count")? as usize;
    let mut outputs = Vec::with_capacity(output_count);
    for _ in 0..output_count {
        let nid = NodeId(cur.read_u64("Boundary output NodeId")?);
        let tref = TypeId(cur.read_u64("Boundary output TypeRef")?);
        outputs.push((nid, tref));
    }
    let boundary = Boundary { inputs, outputs };

    // -- Import section --
    let import_count = cur.read_u16("Import count")? as usize;
    let mut imports = Vec::with_capacity(import_count);
    for _ in 0..import_count {
        imports.push(FragmentId(cur.read_bytes32("Import FragmentId")?));
    }

    // -- SemanticHash --
    let semantic_hash = SemanticHash(cur.read_bytes32("SemanticHash")?);

    // -- Root NodeId --
    let root = NodeId(cur.read_u64("Root NodeId")?);

    // -- Graph CostBound --
    let graph_cost = deserialize_cost_bound(&mut cur)?;

    // -- Graph TypeEnv --
    let graph_type_count = cur.read_u32("Graph TypeEnv count")? as usize;
    let mut graph_types = BTreeMap::new();
    for _ in 0..graph_type_count {
        let tid = TypeId(cur.read_u64("Graph TypeEnv TypeId")?);
        let tdef = deserialize_type_def(&mut cur)?;
        graph_types.insert(tid, tdef);
    }

    let graph = SemanticGraph {
        root,
        nodes,
        edges,
        type_env: TypeEnv {
            types: graph_types,
        },
        cost: graph_cost,
        resolution,
        hash: semantic_hash,
    };

    // -- Optional ProofReceipt --
    let proof = if has_proof {
        Some(deserialize_proof_receipt(&mut cur)?)
    } else {
        None
    };

    // -- Metadata --
    let metadata = deserialize_metadata(&mut cur)?;

    Ok(Fragment {
        id: fragment_id,
        graph,
        boundary,
        type_env,
        imports,
        metadata,
        proof,
        contracts: Default::default(),
    })
}

// ===========================================================================
// Metadata serialization
// ===========================================================================

fn serialize_metadata(buf: &mut Vec<u8>, meta: &FragmentMeta) {
    // name: Option<String>
    match &meta.name {
        Some(name) => {
            write_u8(buf, 1);
            write_len_prefixed_string(buf, name);
        }
        None => write_u8(buf, 0),
    }
    write_u64(buf, meta.created_at);
    write_u64(buf, meta.generation);
    write_u32(buf, meta.lineage_hash);
}

fn deserialize_metadata(cur: &mut Cursor<'_>) -> Result<FragmentMeta, WireError> {
    let has_name = cur.read_u8("FragmentMeta has_name")?;
    let name = if has_name == 1 {
        Some(cur.read_len_prefixed_string("FragmentMeta name")?)
    } else {
        None
    };
    let created_at = cur.read_u64("FragmentMeta created_at")?;
    let generation = cur.read_u64("FragmentMeta generation")?;
    let lineage_hash = cur.read_u32("FragmentMeta lineage_hash")?;
    Ok(FragmentMeta {
        name,
        created_at,
        generation,
        lineage_hash,
    })
}

// ===========================================================================
// Bundle serialization (SPEC Section 14.2)
// ===========================================================================

/// Serialize multiple fragments into the IRBD bundle format.
///
/// Layout:
/// ```text
/// Magic: 4B (0x49524244 "IRBD") | Version: 2B | Root FragmentId: 32B
/// Fragment count: 4B | Blob count: 4B | Flags: 2B
/// Fragment table: (FragmentId 32B + offset 8B + length 4B) per fragment
/// Payload section: serialized fragments concatenated
/// ```
pub fn serialize_bundle(fragments: &[Fragment]) -> Vec<u8> {
    if fragments.is_empty() {
        let mut buf = Vec::with_capacity(48);
        buf.extend_from_slice(&BUNDLE_MAGIC);
        write_u16(&mut buf, WIRE_VERSION);
        write_bytes32(&mut buf, &[0u8; 32]); // root = zero
        write_u32(&mut buf, 0); // fragment count
        write_u32(&mut buf, 0); // blob count
        write_u16(&mut buf, 0); // flags
        return buf;
    }

    // Serialize each fragment individually.
    let serialized: Vec<Vec<u8>> = fragments.iter().map(serialize_fragment).collect();

    // Compute total size for pre-allocation.
    let table_entry_size = 32 + 8 + 4; // FragmentId + offset + length
    let header_size = 4 + 2 + 32 + 4 + 4 + 2; // magic + version + root + frag_count + blob_count + flags
    let table_size = table_entry_size * fragments.len();
    let payload_size: usize = serialized.iter().map(|s| s.len()).sum();
    let total = header_size + table_size + payload_size;

    let mut buf = Vec::with_capacity(total);

    // -- Header --
    buf.extend_from_slice(&BUNDLE_MAGIC);
    write_u16(&mut buf, WIRE_VERSION);
    write_bytes32(&mut buf, &fragments[0].id.0); // root = first fragment
    write_u32(&mut buf, fragments.len() as u32);
    write_u32(&mut buf, 0); // blob count (no blobs in basic bundle)
    write_u16(&mut buf, 0); // flags

    // -- Fragment table --
    // Offsets are relative to the start of the payload section.
    let payload_base = header_size + table_size;
    let mut offset: u64 = 0;
    for (i, frag) in fragments.iter().enumerate() {
        write_bytes32(&mut buf, &frag.id.0);
        write_u64(&mut buf, payload_base as u64 + offset);
        write_u32(&mut buf, serialized[i].len() as u32);
        offset += serialized[i].len() as u64;
    }

    // -- Payload section --
    for s in &serialized {
        buf.extend_from_slice(s);
    }

    buf
}

/// Deserialize a bundle of fragments from IRBD wire format.
pub fn deserialize_bundle(bytes: &[u8]) -> Result<Vec<Fragment>, WireError> {
    let mut cur = Cursor::new(bytes);

    // -- Header --
    let magic_bytes = cur.read_exact(4, "bundle magic")?;
    let magic: [u8; 4] = magic_bytes.try_into().unwrap();
    if magic != BUNDLE_MAGIC {
        return Err(WireError::BadMagic { got: magic });
    }

    let version = cur.read_u16("bundle version")?;
    if version != WIRE_VERSION {
        return Err(WireError::UnsupportedVersion { got: version });
    }

    let _root_id = cur.read_bytes32("bundle root FragmentId")?;
    let fragment_count = cur.read_u32("bundle fragment count")? as usize;
    let _blob_count = cur.read_u32("bundle blob count")?;
    let _flags = cur.read_u16("bundle flags")?;

    // -- Fragment table --
    let mut table_entries = Vec::with_capacity(fragment_count);
    for _ in 0..fragment_count {
        let _frag_id = cur.read_bytes32("bundle table FragmentId")?;
        let offset = cur.read_u64("bundle table offset")? as usize;
        let length = cur.read_u32("bundle table length")? as usize;
        table_entries.push((offset, length));
    }

    // -- Payload section: deserialize each fragment --
    let mut fragments = Vec::with_capacity(fragment_count);
    for (offset, length) in &table_entries {
        if *offset + *length > bytes.len() {
            return Err(WireError::InvalidLength {
                length: *length,
                context: "bundle fragment payload",
            });
        }
        let frag_bytes = &bytes[*offset..*offset + *length];
        fragments.push(deserialize_fragment(frag_bytes)?);
    }

    Ok(fragments)
}

// ===========================================================================
// Unit tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_fragment() -> Fragment {
        let mut nodes = HashMap::new();

        // Lit node: 42
        let lit_node = Node {
            id: NodeId(10),
            kind: NodeKind::Lit,
            type_sig: TypeId(100),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0x00,
                value: 42i64.to_le_bytes().to_vec(),
            },
        };
        nodes.insert(NodeId(10), lit_node);

        // Lit node: 7
        let lit_node2 = Node {
            id: NodeId(20),
            kind: NodeKind::Lit,
            type_sig: TypeId(100),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0x00,
                value: 7i64.to_le_bytes().to_vec(),
            },
        };
        nodes.insert(NodeId(20), lit_node2);

        // Prim node: add
        let add_node = Node {
            id: NodeId(1),
            kind: NodeKind::Prim,
            type_sig: TypeId(200),
            cost: CostTerm::Unit,
            arity: 2,
            resolution_depth: 0, salt: 0,
            payload: NodePayload::Prim { opcode: 0x00 },
        };
        nodes.insert(NodeId(1), add_node);

        let edges = vec![
            Edge {
                source: NodeId(1),
                target: NodeId(10),
                port: 0,
                label: EdgeLabel::Argument,
            },
            Edge {
                source: NodeId(1),
                target: NodeId(20),
                port: 1,
                label: EdgeLabel::Argument,
            },
        ];

        let mut type_env_types = BTreeMap::new();
        type_env_types.insert(TypeId(100), TypeDef::Primitive(PrimType::Int));
        type_env_types.insert(
            TypeId(200),
            TypeDef::Arrow(TypeId(100), TypeId(100), CostBound::Zero),
        );

        let graph = SemanticGraph {
            root: NodeId(1),
            nodes,
            edges,
            type_env: TypeEnv {
                types: type_env_types.clone(),
            },
            cost: CostBound::Zero,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0xAB; 32]),
        };

        Fragment {
            id: FragmentId([0x01; 32]),
            graph,
            boundary: Boundary {
                inputs: vec![(NodeId(10), TypeId(100)), (NodeId(20), TypeId(100))],
                outputs: vec![(NodeId(1), TypeId(100))],
            },
            type_env: TypeEnv {
                types: type_env_types,
            },
            imports: vec![],
            metadata: FragmentMeta {
                name: Some("test_add".to_string()),
                created_at: 1700000000,
                generation: 0,
                lineage_hash: 0,
            },
            proof: None,
            contracts: Default::default(),
        }
    }

    #[test]
    fn roundtrip_fragment() {
        let fragment = make_test_fragment();
        let bytes = serialize_fragment(&fragment);
        let decoded = deserialize_fragment(&bytes).expect("deserialize should succeed");
        assert_eq!(fragment, decoded);
    }

    #[test]
    fn magic_bytes_and_version() {
        let fragment = make_test_fragment();
        let bytes = serialize_fragment(&fragment);
        assert_eq!(&bytes[0..4], &FRAGMENT_MAGIC);
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        assert_eq!(version, WIRE_VERSION);
    }

    #[test]
    fn deterministic_serialization() {
        let fragment = make_test_fragment();
        let bytes1 = serialize_fragment(&fragment);
        let bytes2 = serialize_fragment(&fragment);
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn roundtrip_bundle() {
        let f1 = make_test_fragment();
        let mut f2 = make_test_fragment();
        f2.id = FragmentId([0x02; 32]);
        f2.metadata.name = Some("test_add_2".to_string());

        let bytes = serialize_bundle(&[f1.clone(), f2.clone()]);
        let decoded = deserialize_bundle(&bytes).expect("bundle deserialize should succeed");
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0], f1);
        assert_eq!(decoded[1], f2);
    }

    #[test]
    fn bundle_magic() {
        let f1 = make_test_fragment();
        let bytes = serialize_bundle(&[f1]);
        assert_eq!(&bytes[0..4], &BUNDLE_MAGIC);
    }

    #[test]
    fn empty_bundle() {
        let bytes = serialize_bundle(&[]);
        let decoded = deserialize_bundle(&bytes).expect("empty bundle decode");
        assert_eq!(decoded.len(), 0);
    }

    #[test]
    fn bad_magic_fragment() {
        let mut bytes = serialize_fragment(&make_test_fragment());
        bytes[0] = 0xFF;
        let err = deserialize_fragment(&bytes).unwrap_err();
        assert!(matches!(err, WireError::BadMagic { .. }));
    }

    #[test]
    fn bad_version_fragment() {
        let mut bytes = serialize_fragment(&make_test_fragment());
        // Overwrite version to 99.
        bytes[4] = 99;
        bytes[5] = 0;
        let err = deserialize_fragment(&bytes).unwrap_err();
        assert!(matches!(err, WireError::UnsupportedVersion { got: 99 }));
    }
}
