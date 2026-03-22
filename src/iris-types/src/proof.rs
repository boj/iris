use serde::{Deserialize, Serialize};

use crate::cost::CostBound;
use crate::fragment::FragmentId;
use crate::graph::NodeId;
use crate::types::{DecreaseWitness, LIAFormula, TypeRef};

// ---------------------------------------------------------------------------
// RuleName
// ---------------------------------------------------------------------------

/// Name of a typing/inference rule applied in a proof step.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuleName(pub String);

// ---------------------------------------------------------------------------
// SmtCertificate
// ---------------------------------------------------------------------------

/// Opaque certificate from an SMT solver.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SmtCertificate(pub Vec<u8>);

// ---------------------------------------------------------------------------
// ExternCertificate
// ---------------------------------------------------------------------------

/// Opaque certificate from an external verification source.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExternCertificate(pub Vec<u8>);

// ---------------------------------------------------------------------------
// ProofTree
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProofTree {
    /// Derivation by a named typing/inference rule.
    ByRule(RuleName, NodeId, Vec<ProofTree>),
    /// Derivation via SMT solver with a certificate.
    BySMT(NodeId, LIAFormula, SmtCertificate),
    /// Derivation via an explicit decrease witness.
    ByWitness(NodeId, DecreaseWitness),
    /// Derivation via an external certificate.
    ByExtern(NodeId, ExternCertificate),
}

// ---------------------------------------------------------------------------
// VerifyTier (0-3)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum VerifyTier {
    /// Fully automatic, quantifier-free LIA, < 10ms.
    Tier0 = 0,
    /// Mostly automatic, bounded quantifiers, < 1s.
    Tier1 = 1,
    /// Semi-automatic, full FOL + SMT, < 60s.
    Tier2 = 2,
    /// External certificate.
    Tier3 = 3,
}

// ---------------------------------------------------------------------------
// ProofReceipt
// ---------------------------------------------------------------------------

/// ~330 byte compact proof certificate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofReceipt {
    pub graph_hash: FragmentId,
    pub type_sig: TypeRef,
    pub cost_bound: CostBound,
    pub tier: VerifyTier,
    /// BLAKE3 of the full ProofTree.
    pub proof_merkle_root: [u8; 32],
    /// Compact data for fast re-verification (up to 256 bytes).
    pub compact_witness: Vec<u8>,
}
