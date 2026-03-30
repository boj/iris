use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::cost::{CostBound, CostTerm};
use crate::fragment::FragmentId;
use crate::guard::{BlobRef, GuardSpec};
use crate::hash::SemanticHash;
use crate::types::{BoundVar, DecreaseWitness, TypeEnv, TypeRef};

// ---------------------------------------------------------------------------
// NodeId
// ---------------------------------------------------------------------------

/// 64-bit content-addressed node identity (BLAKE3 truncated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

// ---------------------------------------------------------------------------
// BinderId
// ---------------------------------------------------------------------------

/// Binder identifier for lambda/let-rec nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BinderId(pub u32);

// ---------------------------------------------------------------------------
// RewriteRuleId
// ---------------------------------------------------------------------------

/// 256-bit identity of a verified rewrite rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RewriteRuleId(pub [u8; 32]);

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    /// 5-bit tag (20 active variants).
    pub kind: NodeKind,
    /// Index into TypeEnv.
    pub type_sig: TypeRef,
    /// Per-node cost annotation.
    pub cost: CostTerm,
    pub arity: u8,
    pub resolution_depth: u8,
    /// Disambiguation salt for content-addressed deduplication.
    /// Nodes with identical structure but different graph positions get
    /// different salts so they hash to distinct NodeIds.
    #[serde(default)]
    pub salt: u64,
    pub payload: NodePayload,
}

// ---------------------------------------------------------------------------
// NodeKind (5-bit tag, 20 active variants)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum NodeKind {
    Prim = 0x00,
    Apply = 0x01,
    Lambda = 0x02,
    Let = 0x03,
    Match = 0x04,
    Lit = 0x05,
    Ref = 0x06,
    Neural = 0x07,
    Fold = 0x08,
    Unfold = 0x09,
    Effect = 0x0A,
    Tuple = 0x0B,
    Inject = 0x0C,
    Project = 0x0D,
    TypeAbst = 0x0E,
    TypeApp = 0x0F,
    LetRec = 0x10,
    Guard = 0x11,
    Rewrite = 0x12,
    Extern = 0x13,
}

// ---------------------------------------------------------------------------
// NodePayload (one variant per NodeKind)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodePayload {
    /// Prim: opcode from the ~50 primitive table.
    Prim { opcode: u8 },
    /// Apply: no payload; arity determines edges.
    Apply,
    /// Lambda: binder + captured variable count.
    Lambda {
        binder: BinderId,
        captured_count: u32,
    },
    /// Let: no payload.
    Let,
    /// Match: arm count + pattern sketches.
    Match {
        arm_count: u16,
        arm_patterns: Vec<u8>,
    },
    /// Lit: type tag + value bytes.
    Lit { type_tag: u8, value: Vec<u8> },
    /// Ref: reference to another fragment.
    Ref { fragment_id: FragmentId },
    /// Neural: guard spec ref + weight blob ref.
    Neural {
        guard_spec: GuardSpec,
        weight_blob: BlobRef,
    },
    /// Fold: recursion structure descriptor.
    Fold { recursion_descriptor: Vec<u8> },
    /// Unfold: recursion structure descriptor.
    Unfold { recursion_descriptor: Vec<u8> },
    /// Effect: effect tag.
    Effect { effect_tag: u8 },
    /// Tuple: field count from arity — no extra payload.
    Tuple,
    /// Inject: tag index into a sum type.
    Inject { tag_index: u16 },
    /// Project: field index into a product type.
    Project { field_index: u16 },
    /// TypeAbst: bound variable identifier.
    TypeAbst { bound_var_id: BoundVar },
    /// TypeApp: type argument.
    TypeApp { type_arg: TypeRef },
    /// LetRec: guarded recursive binding.
    LetRec {
        binder: BinderId,
        decrease: DecreaseWitness,
    },
    /// Guard: runtime guard node references.
    Guard {
        predicate_node: NodeId,
        body_node: NodeId,
        fallback_node: NodeId,
    },
    /// Rewrite: verified rewrite application.
    Rewrite {
        rule_id: RewriteRuleId,
        body: NodeId,
    },
    /// Extern: external function reference.
    Extern {
        name: [u8; 32],
        type_sig: TypeRef,
    },
}

// ---------------------------------------------------------------------------
// Edge
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
    pub port: u8,
    pub label: EdgeLabel,
}

// ---------------------------------------------------------------------------
// EdgeLabel (4-bit)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum EdgeLabel {
    Argument = 0,
    Scrutinee = 1,
    Binding = 2,
    Continuation = 3,
    Decrease = 4,
}

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resolution {
    /// Depth 0: high-level intent.
    Intent,
    /// Depth 1: architectural shape.
    Architecture,
    /// Depth 2: full implementation detail.
    Implementation,
}

// ---------------------------------------------------------------------------
// SemanticGraph
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticGraph {
    pub root: NodeId,
    pub nodes: HashMap<NodeId, Node>,
    pub edges: Vec<Edge>,
    pub type_env: TypeEnv,
    /// Non-optional; `Unknown` if unannotated.
    pub cost: CostBound,
    pub resolution: Resolution,
    /// 256-bit BLAKE3 behavioral hash.
    pub hash: SemanticHash,
}

impl SemanticGraph {
    /// Iterate over nodes in deterministic (sorted by NodeId) order.
    ///
    /// HashMap does not guarantee iteration order. Use this method when
    /// deterministic ordering is required (e.g., content-addressing, hashing,
    /// serialization).
    pub fn sorted_nodes(&self) -> Vec<(&NodeId, &Node)> {
        let mut pairs: Vec<(&NodeId, &Node)> = self.nodes.iter().collect();
        pairs.sort_by_key(|(id, _)| *id);
        pairs
    }

    /// Return node IDs in deterministic sorted order.
    pub fn sorted_node_ids(&self) -> Vec<NodeId> {
        let mut ids: Vec<_> = self.nodes.keys().copied().collect();
        ids.sort();
        ids
    }
}
