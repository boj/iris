use serde::{Deserialize, Serialize};

use crate::cost::CostBound;
use crate::fragment::{Fragment, FragmentId};
use crate::graph::{Node, NodeId};
use crate::types::{LIAAtom, LIAFormula, LIATerm, SizeTerm, TypeDef, TypeId};

// ---------------------------------------------------------------------------
// SemanticHash
// ---------------------------------------------------------------------------

/// 256-bit BLAKE3 behavioral fingerprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SemanticHash(pub [u8; 32]);

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Feed a `u64` into the hasher in little-endian form.
fn hash_u64(hasher: &mut blake3::Hasher, val: u64) {
    hasher.update(&val.to_le_bytes());
}

/// Feed a byte-slice length-prefixed into the hasher.
fn hash_bytes(hasher: &mut blake3::Hasher, data: &[u8]) {
    hash_u64(hasher, data.len() as u64);
    hasher.update(data);
}

/// Truncate a BLAKE3 hash to 64 bits (first 8 bytes, little-endian).
///
/// # Birthday-Attack Risk
///
/// A 64-bit hash space has a 50% collision probability at ~2^32 items
/// (~4 billion). For `NodeId` this is safe for typical graph sizes, but
/// workloads approaching 2^32 distinct nodes or types risk accidental
/// collisions. If IRIS graphs ever reach this scale, `NodeId`/`TypeId` should
/// be widened to 128 bits.
///
/// The `debug_assert!` in callers (`compute_node_id`, `compute_type_id`)
/// can be used to instrument collision detection in development builds.
fn truncate_to_u64(hash: blake3::Hash) -> u64 {
    let bytes: [u8; 8] = hash.as_bytes()[..8]
        .try_into()
        .expect("slice is 8 bytes");
    u64::from_le_bytes(bytes)
}

/// Global approximate count of allocated NodeIds in debug builds.
/// Used to warn when approaching birthday-attack thresholds.
#[cfg(debug_assertions)]
static NODE_ID_COUNT: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Global approximate count of allocated TypeIds in debug builds.
#[cfg(debug_assertions)]
static TYPE_ID_COUNT: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Warn threshold: 2^30 = ~1B. Below the 2^32 birthday boundary
/// but high enough not to fire during normal evolution runs.
const BIRTHDAY_WARN_THRESHOLD: u64 = 1 << 30;

/// Whether we've already printed the birthday-attack warning.
#[cfg(debug_assertions)]
static BIRTHDAY_WARNED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

// ---------------------------------------------------------------------------
// Hashing utilities
// ---------------------------------------------------------------------------

/// Compute a 64-bit `NodeId` from a node's canonical representation.
///
/// Uses BLAKE3 of the node fields (excluding the `id` field itself),
/// truncated to the first 8 bytes.
pub fn compute_node_id(node: &Node) -> NodeId {
    let mut hasher = blake3::Hasher::new();

    // Kind tag
    hasher.update(&[node.kind as u8]);

    // Type signature
    hash_u64(&mut hasher, node.type_sig.0);

    // Arity + resolution depth
    hasher.update(&[node.arity, node.resolution_depth]);

    // Salt (disambiguation for structurally-identical nodes)
    if node.salt != 0 {
        hash_u64(&mut hasher, node.salt);
    }

    // Cost term discriminant (lightweight; full cost is in the graph)
    match &node.cost {
        crate::cost::CostTerm::Unit => hasher.update(&[0]),
        crate::cost::CostTerm::Inherited => hasher.update(&[1]),
        crate::cost::CostTerm::Annotated(_) => hasher.update(&[2]),
    };

    // Payload discriminant + key fields
    match &node.payload {
        crate::graph::NodePayload::Prim { opcode } => {
            hasher.update(&[0x00, *opcode]);
        }
        crate::graph::NodePayload::Apply => {
            hasher.update(&[0x01]);
        }
        crate::graph::NodePayload::Lambda {
            binder,
            captured_count,
        } => {
            hasher.update(&[0x02]);
            hasher.update(&binder.0.to_le_bytes());
            hasher.update(&captured_count.to_le_bytes());
        }
        crate::graph::NodePayload::Let => {
            hasher.update(&[0x03]);
        }
        crate::graph::NodePayload::Match {
            arm_count,
            arm_patterns,
        } => {
            hasher.update(&[0x04]);
            hasher.update(&arm_count.to_le_bytes());
            hash_bytes(&mut hasher, arm_patterns);
        }
        crate::graph::NodePayload::Lit { type_tag, value } => {
            hasher.update(&[0x05, *type_tag]);
            hash_bytes(&mut hasher, value);
        }
        crate::graph::NodePayload::Ref { fragment_id } => {
            hasher.update(&[0x06]);
            hasher.update(&fragment_id.0);
        }
        crate::graph::NodePayload::Neural { weight_blob, .. } => {
            hasher.update(&[0x07]);
            hasher.update(&weight_blob.hash);
        }
        crate::graph::NodePayload::Fold {
            recursion_descriptor,
        } => {
            hasher.update(&[0x08]);
            hash_bytes(&mut hasher, recursion_descriptor);
        }
        crate::graph::NodePayload::Unfold {
            recursion_descriptor,
        } => {
            hasher.update(&[0x09]);
            hash_bytes(&mut hasher, recursion_descriptor);
        }
        crate::graph::NodePayload::Effect { effect_tag } => {
            hasher.update(&[0x0A, *effect_tag]);
        }
        crate::graph::NodePayload::Tuple => {
            hasher.update(&[0x0B]);
        }
        crate::graph::NodePayload::Inject { tag_index } => {
            hasher.update(&[0x0C]);
            hasher.update(&tag_index.to_le_bytes());
        }
        crate::graph::NodePayload::Project { field_index } => {
            hasher.update(&[0x0D]);
            hasher.update(&field_index.to_le_bytes());
        }
        crate::graph::NodePayload::TypeAbst { bound_var_id } => {
            hasher.update(&[0x0E]);
            hasher.update(&bound_var_id.0.to_le_bytes());
        }
        crate::graph::NodePayload::TypeApp { type_arg } => {
            hasher.update(&[0x0F]);
            hash_u64(&mut hasher, type_arg.0);
        }
        crate::graph::NodePayload::LetRec { binder, .. } => {
            hasher.update(&[0x10]);
            hasher.update(&binder.0.to_le_bytes());
        }
        crate::graph::NodePayload::Guard {
            predicate_node,
            body_node,
            fallback_node,
        } => {
            hasher.update(&[0x11]);
            hash_u64(&mut hasher, predicate_node.0);
            hash_u64(&mut hasher, body_node.0);
            hash_u64(&mut hasher, fallback_node.0);
        }
        crate::graph::NodePayload::Rewrite { rule_id, body } => {
            hasher.update(&[0x12]);
            hasher.update(&rule_id.0);
            hash_u64(&mut hasher, body.0);
        }
        crate::graph::NodePayload::Extern { name, type_sig } => {
            hasher.update(&[0x13]);
            hasher.update(name);
            hash_u64(&mut hasher, type_sig.0);
        }
    }

    let id = truncate_to_u64(hasher.finalize());

    // Birthday-attack guard: warn once in debug builds when the allocated
    // NodeId count approaches the collision threshold.
    #[cfg(debug_assertions)]
    {
        let count = NODE_ID_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if count >= BIRTHDAY_WARN_THRESHOLD
            && !BIRTHDAY_WARNED.swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            eprintln!(
                "WARNING: NodeId count ({count}) approaching birthday-attack threshold. \
                 Consider widening NodeId from u64 to u128 if this graph will grow further."
            );
        }
    }

    NodeId(id)
}

// ---------------------------------------------------------------------------
// Type-hash helper functions (used by compute_type_id)
// ---------------------------------------------------------------------------

/// Hash a `CostBound` into the hasher for inclusion in type IDs.
///
/// This is a lightweight structural hash — not the same as the ZK cost hash —
/// used to distinguish `Arrow` types with different cost annotations.
fn hash_cost_bound_in_type(hasher: &mut blake3::Hasher, cost: &CostBound) {
    match cost {
        CostBound::Unknown => { hasher.update(&[0x00]); }
        CostBound::Zero => { hasher.update(&[0x01]); }
        CostBound::Constant(v) => {
            hasher.update(&[0x02]);
            hasher.update(&v.to_le_bytes());
        }
        CostBound::Linear(v) => {
            hasher.update(&[0x03]);
            hasher.update(&v.0.to_le_bytes());
        }
        CostBound::NLogN(v) => {
            hasher.update(&[0x04]);
            hasher.update(&v.0.to_le_bytes());
        }
        CostBound::Polynomial(v, deg) => {
            hasher.update(&[0x05]);
            hasher.update(&v.0.to_le_bytes());
            hasher.update(&deg.to_le_bytes());
        }
        CostBound::Sum(a, b) => {
            hasher.update(&[0x06]);
            hash_cost_bound_in_type(hasher, a);
            hash_cost_bound_in_type(hasher, b);
        }
        CostBound::Par(a, b) => {
            hasher.update(&[0x07]);
            hash_cost_bound_in_type(hasher, a);
            hash_cost_bound_in_type(hasher, b);
        }
        CostBound::Mul(a, b) => {
            hasher.update(&[0x08]);
            hash_cost_bound_in_type(hasher, a);
            hash_cost_bound_in_type(hasher, b);
        }
        CostBound::Amortized(inner, _) => {
            hasher.update(&[0x09]);
            hash_cost_bound_in_type(hasher, inner);
        }
        CostBound::HWScaled(inner, hw) => {
            hasher.update(&[0x0A]);
            hash_cost_bound_in_type(hasher, inner);
            hasher.update(&hw.0);
        }
        CostBound::Sup(bounds) => {
            hasher.update(&[0x0B]);
            hasher.update(&(bounds.len() as u64).to_le_bytes());
            for b in bounds {
                hash_cost_bound_in_type(hasher, b);
            }
        }
        CostBound::Inf(bounds) => {
            hasher.update(&[0x0C]);
            hasher.update(&(bounds.len() as u64).to_le_bytes());
            for b in bounds {
                hash_cost_bound_in_type(hasher, b);
            }
        }
    }
}

/// Hash a `SizeTerm` into the hasher for inclusion in `Vec` type IDs.
fn hash_size_term(hasher: &mut blake3::Hasher, size: &SizeTerm) {
    match size {
        SizeTerm::Const(v) => {
            hasher.update(&[0x00]);
            hasher.update(&v.to_le_bytes());
        }
        SizeTerm::Var(bv) => {
            hasher.update(&[0x01]);
            hasher.update(&bv.0.to_le_bytes());
        }
        SizeTerm::Add(a, b) => {
            hasher.update(&[0x02]);
            hash_size_term(hasher, a);
            hash_size_term(hasher, b);
        }
        SizeTerm::Mul(k, t) => {
            hasher.update(&[0x03]);
            hasher.update(&k.to_le_bytes());
            hash_size_term(hasher, t);
        }
    }
}

/// Hash a `LIAFormula` into the hasher for inclusion in `Refined` type IDs.
fn hash_lia_formula(hasher: &mut blake3::Hasher, formula: &LIAFormula) {
    match formula {
        LIAFormula::True => { hasher.update(&[0x00]); }
        LIAFormula::False => { hasher.update(&[0x01]); }
        LIAFormula::And(a, b) => {
            hasher.update(&[0x02]);
            hash_lia_formula(hasher, a);
            hash_lia_formula(hasher, b);
        }
        LIAFormula::Or(a, b) => {
            hasher.update(&[0x03]);
            hash_lia_formula(hasher, a);
            hash_lia_formula(hasher, b);
        }
        LIAFormula::Not(f) => {
            hasher.update(&[0x04]);
            hash_lia_formula(hasher, f);
        }
        LIAFormula::Implies(a, b) => {
            hasher.update(&[0x05]);
            hash_lia_formula(hasher, a);
            hash_lia_formula(hasher, b);
        }
        LIAFormula::Atom(atom) => {
            hasher.update(&[0x06]);
            hash_lia_atom(hasher, atom);
        }
    }
}

fn hash_lia_atom(hasher: &mut blake3::Hasher, atom: &LIAAtom) {
    match atom {
        LIAAtom::Eq(a, b) => {
            hasher.update(&[0x00]);
            hash_lia_term(hasher, a);
            hash_lia_term(hasher, b);
        }
        LIAAtom::Lt(a, b) => {
            hasher.update(&[0x01]);
            hash_lia_term(hasher, a);
            hash_lia_term(hasher, b);
        }
        LIAAtom::Le(a, b) => {
            hasher.update(&[0x02]);
            hash_lia_term(hasher, a);
            hash_lia_term(hasher, b);
        }
        LIAAtom::Divisible(t, k) => {
            hasher.update(&[0x03]);
            hash_lia_term(hasher, t);
            hasher.update(&k.to_le_bytes());
        }
    }
}

fn hash_lia_term(hasher: &mut blake3::Hasher, term: &LIATerm) {
    match term {
        LIATerm::Var(bv) => {
            hasher.update(&[0x00]);
            hasher.update(&bv.0.to_le_bytes());
        }
        LIATerm::Const(v) => {
            hasher.update(&[0x01]);
            hasher.update(&v.to_le_bytes());
        }
        LIATerm::Add(a, b) => {
            hasher.update(&[0x02]);
            hash_lia_term(hasher, a);
            hash_lia_term(hasher, b);
        }
        LIATerm::Mul(k, t) => {
            hasher.update(&[0x03]);
            hasher.update(&k.to_le_bytes());
            hash_lia_term(hasher, t);
        }
        LIATerm::Neg(t) => {
            hasher.update(&[0x04]);
            hash_lia_term(hasher, t);
        }
        LIATerm::Len(bv) => {
            hasher.update(&[0x05]);
            hasher.update(&bv.0.to_le_bytes());
        }
        LIATerm::Size(bv) => {
            hasher.update(&[0x06]);
            hasher.update(&bv.0.to_le_bytes());
        }
        LIATerm::IfThenElse(cond, a, b) => {
            hasher.update(&[0x07]);
            hash_lia_formula(hasher, cond);
            hash_lia_term(hasher, a);
            hash_lia_term(hasher, b);
        }
        LIATerm::Mod(a, b) => {
            hasher.update(&[0x08]);
            hash_lia_term(hasher, a);
            hash_lia_term(hasher, b);
        }
    }
}

/// Compute a 64-bit `TypeId` from a type definition's canonical representation.
///
/// Uses BLAKE3 of the type definition's discriminant and key fields,
/// truncated to the first 8 bytes.
pub fn compute_type_id(type_def: &TypeDef) -> TypeId {
    let mut hasher = blake3::Hasher::new();

    match type_def {
        TypeDef::Primitive(p) => {
            hasher.update(&[0x00, *p as u8]);
        }
        TypeDef::Product(fields) => {
            hasher.update(&[0x01]);
            for f in fields {
                hash_u64(&mut hasher, f.0);
            }
        }
        TypeDef::Sum(variants) => {
            hasher.update(&[0x02]);
            for (tag, tid) in variants {
                hasher.update(&tag.0.to_le_bytes());
                hash_u64(&mut hasher, tid.0);
            }
        }
        TypeDef::Recursive(bv, tid) => {
            hasher.update(&[0x03]);
            hasher.update(&bv.0.to_le_bytes());
            hash_u64(&mut hasher, tid.0);
        }
        TypeDef::ForAll(bv, tid) => {
            hasher.update(&[0x04]);
            hasher.update(&bv.0.to_le_bytes());
            hash_u64(&mut hasher, tid.0);
        }
        TypeDef::Arrow(a, b, cost) => {
            hasher.update(&[0x05]);
            hash_u64(&mut hasher, a.0);
            hash_u64(&mut hasher, b.0);
            // Include the cost bound so that Arrow(A→B, O(1)) ≠ Arrow(A→B, O(n)).
            hash_cost_bound_in_type(&mut hasher, cost);
        }
        TypeDef::Refined(tid, pred) => {
            hasher.update(&[0x06]);
            hash_u64(&mut hasher, tid.0);
            // Include the refinement predicate so that {x:T | P} ≠ {x:T | Q}.
            hash_lia_formula(&mut hasher, pred);
        }
        TypeDef::NeuralGuard(tin, tout, _guard, _cost) => {
            hasher.update(&[0x07]);
            hash_u64(&mut hasher, tin.0);
            hash_u64(&mut hasher, tout.0);
        }
        TypeDef::Exists(bv, tid) => {
            hasher.update(&[0x08]);
            hasher.update(&bv.0.to_le_bytes());
            hash_u64(&mut hasher, tid.0);
        }
        TypeDef::Vec(tid, size) => {
            hasher.update(&[0x09]);
            hash_u64(&mut hasher, tid.0);
            // Include the size term so that Vec<T>[n] ≠ Vec<T>[m].
            hash_size_term(&mut hasher, size);
        }
        TypeDef::HWParam(tid, _profile) => {
            hasher.update(&[0x0A]);
            hash_u64(&mut hasher, tid.0);
        }
    }

    let id = truncate_to_u64(hasher.finalize());

    // Birthday-attack guard: warn once in debug builds.
    #[cfg(debug_assertions)]
    {
        let count = TYPE_ID_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if count >= BIRTHDAY_WARN_THRESHOLD
            && !BIRTHDAY_WARNED.swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            eprintln!(
                "WARNING: TypeId count ({count}) approaching birthday-attack threshold. \
                 Consider widening TypeId from u64 to u128 if this type environment will grow further."
            );
        }
    }

    TypeId(id)
}

/// Compute a 256-bit `FragmentId` from a fragment's content-addressed fields.
///
/// Hashes (graph hash, boundary node ids, type_env type ids, imports) --
/// proof is excluded per spec.
pub fn compute_fragment_id(fragment: &Fragment) -> FragmentId {
    let mut hasher = blake3::Hasher::new();

    // Graph: use its semantic hash as a summary
    hasher.update(&fragment.graph.hash.0);

    // Boundary inputs
    hash_u64(&mut hasher, fragment.boundary.inputs.len() as u64);
    for (nid, tref) in &fragment.boundary.inputs {
        hash_u64(&mut hasher, nid.0);
        hash_u64(&mut hasher, tref.0);
    }

    // Boundary outputs
    hash_u64(&mut hasher, fragment.boundary.outputs.len() as u64);
    for (nid, tref) in &fragment.boundary.outputs {
        hash_u64(&mut hasher, nid.0);
        hash_u64(&mut hasher, tref.0);
    }

    // TypeEnv: hash all type ids in order
    hash_u64(&mut hasher, fragment.type_env.types.len() as u64);
    for tid in fragment.type_env.types.keys() {
        hash_u64(&mut hasher, tid.0);
    }

    // Imports
    hash_u64(&mut hasher, fragment.imports.len() as u64);
    for imp in &fragment.imports {
        hasher.update(&imp.0);
    }

    FragmentId(*hasher.finalize().as_bytes())
}
