use serde::{Deserialize, Serialize};

use crate::graph::{NodeId, SemanticGraph};
use crate::proof::ProofReceipt;
use crate::types::{LIAFormula, TypeEnv, TypeRef};

// ---------------------------------------------------------------------------
// FragmentId
// ---------------------------------------------------------------------------

/// 256-bit BLAKE3 hash identifying a fragment (program/genome).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FragmentId(pub [u8; 32]);

/// Alias for referencing other fragments by hash.
pub type FragmentRef = FragmentId;

// ---------------------------------------------------------------------------
// Fragment
// ---------------------------------------------------------------------------

/// A self-contained holographic unit. The genome IS a Fragment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fragment {
    /// BLAKE3 of (graph, boundary, type_env, imports).
    pub id: FragmentId,
    pub graph: SemanticGraph,
    /// Typed inputs and outputs.
    pub boundary: Boundary,
    /// Complete, self-contained type environment.
    pub type_env: TypeEnv,
    /// Dependencies by hash.
    pub imports: Vec<FragmentRef>,
    pub metadata: FragmentMeta,
    /// Does NOT affect FragmentId.
    pub proof: Option<ProofReceipt>,
    /// Contract annotations (requires/ensures) — does NOT affect FragmentId.
    #[serde(default)]
    pub contracts: FragmentContracts,
}

// ---------------------------------------------------------------------------
// FragmentContracts
// ---------------------------------------------------------------------------

/// Pre/post-condition contracts for a fragment.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FragmentContracts {
    /// Preconditions: LIA formulas over input variables.
    pub requires: Vec<LIAFormula>,
    /// Postconditions: LIA formulas over input variables + result variable.
    /// By convention, BoundVar(0xFFFF) represents the result value.
    pub ensures: Vec<LIAFormula>,
}

// ---------------------------------------------------------------------------
// Boundary
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Boundary {
    pub inputs: Vec<(NodeId, TypeRef)>,
    pub outputs: Vec<(NodeId, TypeRef)>,
}

// ---------------------------------------------------------------------------
// FragmentMeta
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FragmentMeta {
    /// Human-readable name (optional).
    pub name: Option<String>,
    /// Creation timestamp (Unix epoch seconds).
    pub created_at: u64,
    /// Generation number in which this fragment was produced.
    pub generation: u64,
    /// Lineage hash for tracking ancestry.
    pub lineage_hash: u32,
}
