//! Error types for the proof kernel and type checker.
//!
//! `KernelError` covers failures inside the trusted kernel (invalid rule
//! applications, precondition violations). `CheckError` wraps those and adds
//! tier-specific failures from the untrusted checker.

use iris_types::cost::CostBound;
use iris_types::graph::{BinderId, NodeId};
use iris_types::proof::VerifyTier;
use iris_types::types::TypeId;
use std::fmt;

// ---------------------------------------------------------------------------
// KernelError — trusted kernel failures
// ---------------------------------------------------------------------------

/// Errors produced by the proof kernel when an inference rule's preconditions
/// are not met.
#[derive(Clone, Debug, PartialEq)]
pub enum KernelError {
    /// Expected two types to match but they differ.
    TypeMismatch {
        expected: TypeId,
        actual: TypeId,
        context: &'static str,
    },

    /// A cost bound is not less-than-or-equal to the required bound.
    CostViolation {
        required: CostBound,
        actual: CostBound,
    },

    /// An inference rule was applied incorrectly (wrong number of premises,
    /// wrong rule for the node kind, etc.).
    InvalidRule {
        rule: &'static str,
        reason: String,
    },

    /// A referenced node was not found in the graph.
    NodeNotFound(NodeId),

    /// A referenced type was not found in the type environment.
    TypeNotFound(TypeId),

    /// The type definition does not have the expected shape (e.g., expected
    /// Arrow but found Product).
    UnexpectedTypeDef {
        type_id: TypeId,
        expected: &'static str,
    },

    /// Contexts do not match between two theorems that need to share a context.
    ContextMismatch {
        rule: &'static str,
    },

    /// A binder name was not found in the context.
    BinderNotFound {
        rule: &'static str,
        binder: BinderId,
    },

    /// Equality precondition violated (nodes are not equal).
    NotEqual {
        left: NodeId,
        right: NodeId,
    },

    /// Induction case mismatch (wrong number of cases, etc.).
    InductionError {
        reason: String,
    },

    /// A type definition references a TypeId that does not exist in the TypeEnv.
    /// This indicates a malformed type that cannot be safely used in
    /// substitution (e.g., ForAll elimination).
    TypeMalformed {
        type_id: TypeId,
        dangling_ref: TypeId,
        context: &'static str,
    },
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeMismatch {
                expected,
                actual,
                context,
            } => {
                write!(
                    f,
                    "type mismatch in {context}: expected {expected:?}, got {actual:?}"
                )
            }
            Self::CostViolation { required, actual } => {
                write!(f, "cost violation: {actual:?} does not satisfy {required:?}")
            }
            Self::InvalidRule { rule, reason } => {
                write!(f, "invalid rule application ({rule}): {reason}")
            }
            Self::NodeNotFound(id) => write!(f, "node not found: {id:?}"),
            Self::TypeNotFound(id) => write!(f, "type not found: {id:?}"),
            Self::UnexpectedTypeDef { type_id, expected } => {
                write!(f, "type {type_id:?}: expected {expected}")
            }
            Self::ContextMismatch { rule } => {
                write!(f, "context mismatch in rule {rule}")
            }
            Self::BinderNotFound { rule, binder } => {
                write!(f, "binder {binder:?} not found in context (rule {rule})")
            }
            Self::NotEqual { left, right } => {
                write!(f, "nodes not equal: {left:?} vs {right:?}")
            }
            Self::InductionError { reason } => {
                write!(f, "induction error: {reason}")
            }
            Self::TypeMalformed {
                type_id,
                dangling_ref,
                context,
            } => {
                write!(
                    f,
                    "malformed type {type_id:?} in {context}: references non-existent type {dangling_ref:?}"
                )
            }
        }
    }
}

impl std::error::Error for KernelError {}

// ---------------------------------------------------------------------------
// CheckError — untrusted checker failures
// ---------------------------------------------------------------------------

/// Errors produced by the type checker (untrusted). Wraps `KernelError` plus
/// tier-specific failures.
#[derive(Clone, Debug)]
pub enum CheckError {
    /// A kernel rule application failed.
    Kernel(KernelError),

    /// The graph contains a construct not permitted at the requested tier.
    TierViolation {
        tier: VerifyTier,
        node: NodeId,
        reason: String,
    },

    /// The graph is malformed (cycles, missing edges, etc.).
    MalformedGraph { reason: String },

    /// A node kind is not yet supported by the checker.
    Unsupported { node: NodeId, kind: String },

    /// A refinement predicate is violated by a concrete value or cannot be
    /// proven from the input constraints.
    RefinementViolation {
        node: NodeId,
        /// Human-readable description of what failed.
        reason: String,
        /// A concrete variable assignment that violates the predicate
        /// (when available from the LIA solver).
        counterexample: Option<std::collections::HashMap<iris_types::types::BoundVar, i64>>,
    },
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kernel(e) => write!(f, "kernel error: {e}"),
            Self::TierViolation { tier, node, reason } => {
                write!(f, "tier {tier:?} violation at {node:?}: {reason}")
            }
            Self::MalformedGraph { reason } => {
                write!(f, "malformed graph: {reason}")
            }
            Self::Unsupported { node, kind } => {
                write!(f, "unsupported node kind {kind} at {node:?}")
            }
            Self::RefinementViolation {
                node,
                reason,
                counterexample,
            } => {
                write!(f, "refinement violation at {node:?}: {reason}")?;
                if let Some(ce) = counterexample {
                    write!(f, " (counterexample: ")?;
                    let mut first = true;
                    for (var, val) in ce {
                        if !first {
                            write!(f, ", ")?;
                        }
                        write!(f, "{var:?}={val}")?;
                        first = false;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for CheckError {}

impl From<KernelError> for CheckError {
    fn from(e: KernelError) -> Self {
        Self::Kernel(e)
    }
}
