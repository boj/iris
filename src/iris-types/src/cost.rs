use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::types::LIAFormula;

// ---------------------------------------------------------------------------
// CostVar
// ---------------------------------------------------------------------------

/// Variable in cost expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CostVar(pub u32);

// ---------------------------------------------------------------------------
// HWParamRef
// ---------------------------------------------------------------------------

/// BLAKE3 hash of a `HardwareProfile`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HWParamRef(pub [u8; 32]);

// ---------------------------------------------------------------------------
// PotentialFn (placeholder for amortized analysis)
// ---------------------------------------------------------------------------

/// Potential function for amortized cost analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PotentialFn {
    pub description: String,
}

// ---------------------------------------------------------------------------
// CostAxiom
// ---------------------------------------------------------------------------

/// An axiom relating hardware parameters to cost.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostAxiom {
    pub name: String,
    pub formula: LIAFormula,
}

// ---------------------------------------------------------------------------
// CostBound (14 variants)
// ---------------------------------------------------------------------------

/// Cost that is hardware-independent by default.
///
/// Most variants describe universal, substrate-independent cost bounds.
/// `HWScaled` is only used when deploying to specific hardware — it wraps
/// a universal inner bound with a hardware profile reference.  All other
/// variants are universal.
///
/// Use [`universalize_cost`] to strip hardware-specific annotations,
/// yielding a purely abstract cost bound suitable for cross-platform
/// reasoning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CostBound {
    Unknown,
    Zero,
    Constant(u64),
    Linear(CostVar),
    NLogN(CostVar),
    Polynomial(CostVar, u32),
    Sum(Box<CostBound>, Box<CostBound>),
    Par(Box<CostBound>, Box<CostBound>),
    Mul(Box<CostBound>, Box<CostBound>),
    Amortized(Box<CostBound>, PotentialFn),
    HWScaled(Box<CostBound>, HWParamRef),
    Sup(Vec<CostBound>),
    Inf(Vec<CostBound>),
}

// ---------------------------------------------------------------------------
// universalize_cost
// ---------------------------------------------------------------------------

/// Strip all hardware-specific cost annotations from a `CostBound`.
///
/// `HWScaled(inner, _)` is replaced by the universalized `inner`.
/// All other variants are recursed structurally.  The result is a purely
/// abstract cost bound that contains no `HWParamRef` references.
pub fn universalize_cost(cost: &CostBound) -> CostBound {
    match cost {
        CostBound::HWScaled(inner, _) => universalize_cost(inner),
        CostBound::Sum(a, b) => CostBound::Sum(
            Box::new(universalize_cost(a)),
            Box::new(universalize_cost(b)),
        ),
        CostBound::Par(a, b) => CostBound::Par(
            Box::new(universalize_cost(a)),
            Box::new(universalize_cost(b)),
        ),
        CostBound::Mul(a, b) => CostBound::Mul(
            Box::new(universalize_cost(a)),
            Box::new(universalize_cost(b)),
        ),
        CostBound::Amortized(inner, pf) => CostBound::Amortized(
            Box::new(universalize_cost(inner)),
            pf.clone(),
        ),
        CostBound::Sup(bounds) => {
            CostBound::Sup(bounds.iter().map(universalize_cost).collect())
        }
        CostBound::Inf(bounds) => {
            CostBound::Inf(bounds.iter().map(universalize_cost).collect())
        }
        // Leaf variants — no hardware content.
        other => other.clone(),
    }
}

// ---------------------------------------------------------------------------
// CostTerm (per-node)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CostTerm {
    Unit,
    Inherited,
    Annotated(CostBound),
}

// ---------------------------------------------------------------------------
// HardwareProfile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub name: String,
    /// e.g., cache_line_bytes: 64
    pub params: BTreeMap<String, u64>,
    pub constraints: Vec<LIAFormula>,
    pub axioms: Vec<CostAxiom>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_hw_ref() -> HWParamRef {
        HWParamRef([0xAB; 32])
    }

    #[test]
    fn universalize_strips_hw_scaled() {
        let inner = CostBound::Linear(CostVar(0));
        let scaled = CostBound::HWScaled(Box::new(inner.clone()), dummy_hw_ref());
        assert_eq!(universalize_cost(&scaled), inner);
    }

    #[test]
    fn universalize_strips_nested_hw_scaled() {
        let leaf = CostBound::Constant(42);
        let scaled = CostBound::HWScaled(Box::new(leaf.clone()), dummy_hw_ref());
        let double_scaled = CostBound::HWScaled(Box::new(scaled), dummy_hw_ref());
        assert_eq!(universalize_cost(&double_scaled), leaf);
    }

    #[test]
    fn universalize_recurses_through_sum() {
        let a = CostBound::HWScaled(
            Box::new(CostBound::Constant(1)),
            dummy_hw_ref(),
        );
        let b = CostBound::Linear(CostVar(0));
        let sum = CostBound::Sum(Box::new(a), Box::new(b.clone()));

        let result = universalize_cost(&sum);
        assert_eq!(
            result,
            CostBound::Sum(
                Box::new(CostBound::Constant(1)),
                Box::new(b),
            )
        );
    }

    #[test]
    fn universalize_recurses_through_sup_inf() {
        let bounds = vec![
            CostBound::HWScaled(Box::new(CostBound::Zero), dummy_hw_ref()),
            CostBound::Constant(10),
        ];
        let sup = CostBound::Sup(bounds);
        let result = universalize_cost(&sup);
        assert_eq!(
            result,
            CostBound::Sup(vec![CostBound::Zero, CostBound::Constant(10)])
        );
    }

    #[test]
    fn universalize_preserves_non_hw_cost() {
        let cost = CostBound::NLogN(CostVar(1));
        assert_eq!(universalize_cost(&cost), cost);
    }

    #[test]
    fn universalize_preserves_amortized() {
        let pf = PotentialFn {
            description: "test".to_string(),
        };
        let inner = CostBound::HWScaled(
            Box::new(CostBound::Constant(5)),
            dummy_hw_ref(),
        );
        let amortized = CostBound::Amortized(Box::new(inner), pf.clone());
        let result = universalize_cost(&amortized);
        assert_eq!(
            result,
            CostBound::Amortized(Box::new(CostBound::Constant(5)), pf)
        );
    }
}
