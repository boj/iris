//! The opaque `Theorem` type — the core LCF-style abstraction.
//!
//! A `Theorem` can ONLY be constructed by functions in `crate::kernel`.
//! External crates can inspect theorems (read their judgments) but never
//! fabricate one. This is enforced by making all fields `pub(crate)`.

use iris_types::cost::CostBound;
use iris_types::graph::{BinderId, NodeId};
use iris_types::types::TypeId;

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

/// A single binding in a typing context: a binder name with its type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub name: BinderId,
    pub type_id: TypeId,
}

/// A typing context: an ordered list of bindings (Gamma).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context {
    pub bindings: Vec<Binding>,
}

impl Context {
    /// Empty context.
    pub fn empty() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// Extend the context with a new binding, returning a new Context.
    pub fn extend(&self, name: BinderId, type_id: TypeId) -> Self {
        let mut bindings = self.bindings.clone();
        bindings.push(Binding { name, type_id });
        Self { bindings }
    }

    /// Look up a binder in the context (searches from most recent).
    pub fn lookup(&self, name: BinderId) -> Option<TypeId> {
        self.bindings
            .iter()
            .rev()
            .find(|b| b.name == name)
            .map(|b| b.type_id)
    }

    /// Check whether `self` is a prefix of (or equal to) `other`,
    /// meaning `other` extends `self` with additional bindings.
    pub fn is_prefix_of(&self, other: &Self) -> bool {
        if self.bindings.len() > other.bindings.len() {
            return false;
        }
        self.bindings
            .iter()
            .zip(other.bindings.iter())
            .all(|(a, b)| a == b)
    }

    /// Remove the last binding, returning (new context, removed binding).
    /// Returns `None` if the context is empty.
    pub fn pop(&self) -> Option<(Self, Binding)> {
        if self.bindings.is_empty() {
            return None;
        }
        let mut bindings = self.bindings.clone();
        let removed = bindings.pop().expect("non-empty");
        Some((Self { bindings }, removed))
    }
}

// ---------------------------------------------------------------------------
// Judgment
// ---------------------------------------------------------------------------

/// A typing judgment: Gamma |- e : tau @ kappa
///
/// Reads: "In context Gamma, node e has type tau with cost bound kappa."
#[derive(Clone, Debug, PartialEq)]
pub struct Judgment {
    pub context: Context,
    pub node_id: NodeId,
    pub type_ref: TypeId,
    pub cost: CostBound,
}

// ---------------------------------------------------------------------------
// Theorem (opaque)
// ---------------------------------------------------------------------------

/// A proven theorem. Contains a judgment and a proof hash for audit trails.
///
/// **Invariant:** This struct can only be constructed inside `crate::kernel`.
/// External crates see the fields via accessor methods but cannot create
/// `Theorem` values.
#[derive(Clone, Debug)]
pub struct Theorem {
    /// The proven judgment.
    pub(crate) judgment: Judgment,
    /// BLAKE3 hash of the proof derivation that produced this theorem.
    pub(crate) proof_hash: [u8; 32],
}

impl Theorem {
    // --- Accessors (public, read-only) ---

    /// The judgment this theorem proves.
    pub fn judgment(&self) -> &Judgment {
        &self.judgment
    }

    /// The context of the proven judgment.
    pub fn context(&self) -> &Context {
        &self.judgment.context
    }

    /// The node this theorem is about.
    pub fn node_id(&self) -> NodeId {
        self.judgment.node_id
    }

    /// The type assigned by the judgment.
    pub fn type_ref(&self) -> TypeId {
        self.judgment.type_ref
    }

    /// The cost bound established by the judgment.
    pub fn cost(&self) -> &CostBound {
        &self.judgment.cost
    }

    /// BLAKE3 hash of the proof derivation.
    pub fn proof_hash(&self) -> &[u8; 32] {
        &self.proof_hash
    }
}
