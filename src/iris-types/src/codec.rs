//! Codec trait types extracted from iris-codec.
//!
//! Contains the `GraphEmbeddingCodec` trait, embedding types, and error types
//! needed by consumers (iris-evolve) without pulling in the full codec crate.

use serde::{Deserialize, Serialize};

use crate::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// Embedding tier
// ---------------------------------------------------------------------------

/// Matryoshka-compatible embedding dimensionality tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmbeddingTier {
    /// 64-dimensional embedding (Gen1 feature codec).
    Tier0 = 64,
    /// 256-dimensional embedding (Gen2 GIN-VAE).
    Tier1 = 256,
    /// 768-dimensional embedding (Gen3 full model).
    Tier2 = 768,
}

// ---------------------------------------------------------------------------
// EmbeddingVector
// ---------------------------------------------------------------------------

/// Fixed-size embedding vector produced by the codec encoder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingVector {
    /// Embedding dimensions (length matches `tier`).
    pub dims: Vec<f32>,
    /// Which tier this embedding belongs to.
    pub tier: EmbeddingTier,
}

impl EmbeddingVector {
    pub fn new(dims: Vec<f32>, tier: EmbeddingTier) -> Self {
        debug_assert_eq!(dims.len(), tier as usize);
        Self { dims, tier }
    }

    pub fn cosine_similarity(&self, other: &EmbeddingVector) -> f32 {
        assert_eq!(self.dims.len(), other.dims.len());
        let dot: f32 = self.dims.iter().zip(&other.dims).map(|(a, b)| a * b).sum();
        let norm_a: f32 = self.dims.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.dims.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a < 1e-12 || norm_b < 1e-12 {
            return 0.0;
        }
        dot / (norm_a * norm_b)
    }
}

// ---------------------------------------------------------------------------
// RepairFailure
// ---------------------------------------------------------------------------

/// Error returned when `structural_repair` cannot salvage a graph.
#[derive(Debug, Clone, PartialEq)]
pub struct RepairFailure {
    pub phase: u8,
    pub reason: String,
}

impl std::fmt::Display for RepairFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "structural repair failed at phase {}: {}",
            self.phase, self.reason
        )
    }
}

impl std::error::Error for RepairFailure {}

// ---------------------------------------------------------------------------
// GraphEmbeddingCodec trait
// ---------------------------------------------------------------------------

/// Bidirectional mapping between SemanticGraphs and embedding vectors.
pub trait GraphEmbeddingCodec: Send + Sync {
    fn encode(&self, graph: &SemanticGraph) -> EmbeddingVector;
    fn decode(&self, embedding: &[f32]) -> SemanticGraph;
    fn structural_repair(&self, graph: SemanticGraph) -> Result<SemanticGraph, RepairFailure>;
    fn interpolate(&self, a: &[f32], b: &[f32], alpha: f32) -> Vec<f32>;
}

// ---------------------------------------------------------------------------
// FeatureCodec — Gen1 deterministic codec
// ---------------------------------------------------------------------------

use std::collections::BTreeMap;
use crate::graph::{NodeId, NodePayload, SemanticGraph as SG, NodeKind, Edge, EdgeLabel, Node, Resolution};
use crate::hash::SemanticHash;
use crate::cost::{CostBound, CostTerm};
use crate::types::TypeEnv;

const TIER0_DIMS: usize = 64;

/// Gen1 deterministic codec: extracts hand-crafted features from SemanticGraphs.
pub struct FeatureCodec;

impl FeatureCodec {
    pub fn new() -> Self { Self }
}

impl Default for FeatureCodec {
    fn default() -> Self { Self::new() }
}

impl GraphEmbeddingCodec for FeatureCodec {
    fn encode(&self, graph: &SG) -> EmbeddingVector {
        let mut dims = vec![0.0f32; TIER0_DIMS];
        let total = graph.nodes.len().max(1) as f32;

        // Node kind histogram (20 bins)
        for node in graph.nodes.values() {
            let idx = node.kind as u8 as usize;
            if idx < 20 { dims[idx] = dims[idx] + 1.0 / total; }
        }

        // Prim opcode histogram (16 bins, offset 20)
        let mut total_prims = 0u32;
        for node in graph.nodes.values() {
            if let NodePayload::Prim { opcode } = &node.payload {
                let bin = match *opcode {
                    0x00..=0x09 => *opcode as usize,
                    0x0A..=0x0F => 10, 0x10..=0x1F => 11,
                    0x20..=0x27 => 12, 0x28..=0x2F => 13,
                    0x30..=0x3F => 14, _ => 15,
                };
                if bin < 16 { dims[20 + bin] += 1.0; }
                total_prims += 1;
            }
        }
        let tp = total_prims.max(1) as f32;
        for i in 20..36 { dims[i] /= tp; }

        // Shape features (5 dims, offset 36)
        dims[36] = graph.nodes.len() as f32 / 100.0;
        dims[37] = graph.edges.len() as f32 / 100.0;
        dims[38] = graph.edges.len() as f32 / total;
        // Cost features (offset 41-63 zeroed for now)

        EmbeddingVector::new(dims, EmbeddingTier::Tier0)
    }

    fn decode(&self, embedding: &[f32]) -> SG {
        // Minimal decode: return a single-node graph
        let root = NodeId(1);
        let node = Node {
            id: root,
            kind: NodeKind::Lit,
            type_sig: crate::types::TypeId(0),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Lit { type_tag: 0x00, value: vec![0] },
        };
        SG {
            root,
            nodes: std::iter::once((root, node)).collect(),
            edges: vec![],
            type_env: TypeEnv { types: BTreeMap::new() },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        }
    }

    fn structural_repair(&self, graph: SG) -> Result<SG, RepairFailure> {
        // Minimal repair: validate root exists
        if graph.nodes.contains_key(&graph.root) {
            Ok(graph)
        } else {
            Err(RepairFailure { phase: 1, reason: "root not in nodes".into() })
        }
    }

    fn interpolate(&self, a: &[f32], b: &[f32], alpha: f32) -> Vec<f32> {
        assert_eq!(a.len(), b.len());
        a.iter().zip(b).map(|(&va, &vb)| alpha * va + (1.0 - alpha) * vb).collect()
    }
}
