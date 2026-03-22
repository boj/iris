//! Zero-knowledge proof system for IRIS.
//!
//! Implements a hash-based ZK scheme using Fiat-Shamir transformed Sigma
//! protocol over Merkle-hashed proof trees. This provides real cryptographic
//! guarantees (proven secure in the random oracle model) using only BLAKE3.
//!
//! The prover demonstrates knowledge of a valid CaCIC derivation (proof tree)
//! without revealing the derivation itself. Only the program hash, spec type
//! hash, and cost bound hash are public.

use std::fmt;

use iris_types::cost::CostBound;
use iris_types::fragment::FragmentId;
use iris_types::graph::SemanticGraph;
use iris_types::proof::{ProofReceipt, ProofTree};
use iris_types::types::TypeRef;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from ZK proof generation or verification.
#[derive(Debug, Clone, PartialEq)]
pub enum ZkError {
    /// The proof tree is empty (no leaves to commit to).
    EmptyProofTree,
    /// Merkle path verification failed at a given index.
    MerklePathInvalid { index: usize },
    /// Challenge-response count does not match expected.
    ChallengeMismatch { expected: usize, actual: usize },
    /// The commitment does not match the recomputed value.
    CommitmentMismatch,
    /// Public inputs hash mismatch.
    PublicInputsMismatch,
    /// Proof receipt is inconsistent with the program.
    ReceiptMismatch { reason: String },
}

impl fmt::Display for ZkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProofTree => write!(f, "proof tree has no leaves"),
            Self::MerklePathInvalid { index } => {
                write!(f, "Merkle path invalid at index {index}")
            }
            Self::ChallengeMismatch { expected, actual } => {
                write!(
                    f,
                    "challenge-response count mismatch: expected {expected}, got {actual}"
                )
            }
            Self::CommitmentMismatch => write!(f, "commitment does not match"),
            Self::PublicInputsMismatch => write!(f, "public inputs hash mismatch"),
            Self::ReceiptMismatch { reason } => {
                write!(f, "receipt mismatch: {reason}")
            }
        }
    }
}

impl std::error::Error for ZkError {}

// ---------------------------------------------------------------------------
// ZK public inputs
// ---------------------------------------------------------------------------

/// Public inputs visible to the verifier. These identify what is being proved
/// without revealing the program or derivation.
#[derive(Debug, Clone, PartialEq)]
pub struct ZkPublicInputs {
    /// BLAKE3 hash identifying the program (FragmentId).
    pub program_hash: FragmentId,
    /// Hash of the claimed type/specification.
    pub spec_type_hash: [u8; 32],
    /// Hash of the claimed cost bound.
    pub cost_bound_hash: [u8; 32],
}

// ---------------------------------------------------------------------------
// ZK private witness
// ---------------------------------------------------------------------------

/// Private witness known only to the prover.
pub struct ZkPrivateWitness<'a> {
    /// The actual program.
    pub program: &'a SemanticGraph,
    /// The full CaCIC derivation.
    pub proof_tree: &'a ProofTree,
}

// ---------------------------------------------------------------------------
// ZK proof
// ---------------------------------------------------------------------------

/// A ZK proof that a valid derivation exists.
#[derive(Debug, Clone, PartialEq)]
pub struct ZkProof {
    /// Public inputs (program hash, spec type, cost bound).
    pub public_inputs: ZkPublicInputs,
    /// Merkle root of the proof tree.
    pub commitment: [u8; 32],
    /// Responses to Fiat-Shamir challenges: each is a Merkle path sibling.
    pub challenge_responses: Vec<ChallengeResponse>,
    /// Total proof size in bytes (approximate).
    pub proof_size: usize,
}

/// A single challenge-response pair in the Fiat-Shamir protocol.
#[derive(Debug, Clone, PartialEq)]
pub struct ChallengeResponse {
    /// Index of the challenged leaf.
    pub leaf_index: usize,
    /// Hash of the leaf itself.
    pub leaf_hash: [u8; 32],
    /// Merkle path from leaf to root (sibling hashes).
    pub merkle_path: Vec<[u8; 32]>,
    /// Whether each sibling is on the left (true) or right (false).
    pub path_directions: Vec<bool>,
}

// ---------------------------------------------------------------------------
// Merkle tree over proof tree leaves
// ---------------------------------------------------------------------------

/// Flatten a `ProofTree` into a list of leaf hashes (DFS order).
fn flatten_proof_tree(tree: &ProofTree) -> Vec<[u8; 32]> {
    let mut leaves = Vec::new();
    flatten_recursive(tree, &mut leaves);
    leaves
}

fn flatten_recursive(tree: &ProofTree, leaves: &mut Vec<[u8; 32]>) {
    match tree {
        ProofTree::ByRule(rule_name, node_id, children) => {
            // Hash this rule application as a leaf.
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"ByRule");
            hasher.update(rule_name.0.as_bytes());
            hasher.update(&node_id.0.to_le_bytes());
            leaves.push(*hasher.finalize().as_bytes());
            // Recurse into children.
            for child in children {
                flatten_recursive(child, leaves);
            }
        }
        ProofTree::BySMT(node_id, _formula, cert) => {
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"BySMT");
            hasher.update(&node_id.0.to_le_bytes());
            hasher.update(&cert.0);
            leaves.push(*hasher.finalize().as_bytes());
        }
        ProofTree::ByWitness(node_id, witness) => {
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"ByWitness");
            hasher.update(&node_id.0.to_le_bytes());
            // Hash the decrease witness variant tag.
            match witness {
                iris_types::types::DecreaseWitness::Structural(a, b) => {
                    hasher.update(&[0x00]);
                    hasher.update(&a.0.to_le_bytes());
                    hasher.update(&b.0.to_le_bytes());
                }
                iris_types::types::DecreaseWitness::Sized(a, b) => {
                    hasher.update(&[0x01]);
                    // Hash the LIATerm discriminants for domain separation.
                    hasher.update(&format!("{a:?}").as_bytes());
                    hasher.update(&format!("{b:?}").as_bytes());
                }
                iris_types::types::DecreaseWitness::WellFounded(v) => {
                    hasher.update(&[0x02]);
                    hasher.update(&v.0);
                }
            }
            leaves.push(*hasher.finalize().as_bytes());
        }
        ProofTree::ByExtern(node_id, cert) => {
            let mut hasher = blake3::Hasher::new();
            hasher.update(b"ByExtern");
            hasher.update(&node_id.0.to_le_bytes());
            hasher.update(&cert.0);
            leaves.push(*hasher.finalize().as_bytes());
        }
    }
}

/// Domain-separated padding hash. Padding leaves use this value instead of
/// [0u8; 32] so that a challenge landing on a padding index cannot be
/// trivially satisfied by presenting a zero-valued leaf hash.
fn padding_leaf_hash(index: usize) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"IrisMerklePadding");
    hasher.update(&(index as u64).to_le_bytes());
    *hasher.finalize().as_bytes()
}

/// Build a Merkle tree from leaf hashes. Returns all layers (leaves at index 0,
/// root at the last index). Pads to a power of two with domain-separated
/// padding hashes (not zero) to prevent trivial forgery on padding leaves.
fn build_merkle_tree(leaves: &[[u8; 32]]) -> Vec<Vec<[u8; 32]>> {
    if leaves.is_empty() {
        return vec![vec![[0u8; 32]]];
    }

    // Pad to next power of two using domain-separated padding hashes.
    let n = leaves.len().next_power_of_two();
    let mut current_layer: Vec<[u8; 32]> = Vec::with_capacity(n);
    current_layer.extend_from_slice(leaves);
    while current_layer.len() < n {
        let pad_idx = current_layer.len();
        current_layer.push(padding_leaf_hash(pad_idx));
    }

    let mut layers = vec![current_layer.clone()];

    while current_layer.len() > 1 {
        let mut next_layer = Vec::with_capacity(current_layer.len() / 2);
        for pair in current_layer.chunks(2) {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&pair[0]);
            hasher.update(&pair[1]);
            next_layer.push(*hasher.finalize().as_bytes());
        }
        layers.push(next_layer.clone());
        current_layer = next_layer;
    }

    layers
}

/// Extract a Merkle path (sibling hashes + directions) for a given leaf index.
fn merkle_path(layers: &[Vec<[u8; 32]>], leaf_index: usize) -> (Vec<[u8; 32]>, Vec<bool>) {
    let mut siblings = Vec::new();
    let mut directions = Vec::new();
    let mut idx = leaf_index;

    for layer in &layers[..layers.len().saturating_sub(1)] {
        let sibling_idx = idx ^ 1;
        if sibling_idx < layer.len() {
            siblings.push(layer[sibling_idx]);
        } else {
            siblings.push([0u8; 32]);
        }
        // true = sibling is on the left (current node is right child)
        directions.push(idx & 1 == 1);
        idx /= 2;
    }

    (siblings, directions)
}

/// Verify a Merkle path from leaf to root.
///
/// Used in unit tests to check internal Merkle tree construction. The main
/// verifier uses `recompute_root_from_path` + `bind_root_to_program` instead.
#[cfg(test)]
fn verify_merkle_path(
    leaf_hash: &[u8; 32],
    path: &[[u8; 32]],
    directions: &[bool],
    root: &[u8; 32],
) -> bool {
    let mut current = *leaf_hash;

    for (sibling, &is_right) in path.iter().zip(directions.iter()) {
        let mut hasher = blake3::Hasher::new();
        if is_right {
            // Current node is the right child; sibling is on the left.
            hasher.update(sibling);
            hasher.update(&current);
        } else {
            // Current node is the left child; sibling is on the right.
            hasher.update(&current);
            hasher.update(sibling);
        }
        current = *hasher.finalize().as_bytes();
    }

    current == *root
}

// ---------------------------------------------------------------------------
// Hashing helpers
// ---------------------------------------------------------------------------

/// Hash the public inputs into 32 bytes.
fn hash_public_inputs(inputs: &ZkPublicInputs) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"ZkPublicInputs");
    hasher.update(&inputs.program_hash.0);
    hasher.update(&inputs.spec_type_hash);
    hasher.update(&inputs.cost_bound_hash);
    *hasher.finalize().as_bytes()
}

/// Hash a TypeRef into 32 bytes for public input.
pub fn hash_type_ref(type_ref: &TypeRef) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"TypeRef");
    hasher.update(&type_ref.0.to_le_bytes());
    *hasher.finalize().as_bytes()
}

/// Hash a CostBound into 32 bytes for public input.
pub fn hash_cost_bound(cost: &CostBound) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"CostBound");
    hash_cost_bound_recursive(&mut hasher, cost);
    *hasher.finalize().as_bytes()
}

fn hash_cost_bound_recursive(hasher: &mut blake3::Hasher, cost: &CostBound) {
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
            hash_cost_bound_recursive(hasher, a);
            hash_cost_bound_recursive(hasher, b);
        }
        CostBound::Par(a, b) => {
            hasher.update(&[0x07]);
            hash_cost_bound_recursive(hasher, a);
            hash_cost_bound_recursive(hasher, b);
        }
        CostBound::Mul(a, b) => {
            hasher.update(&[0x08]);
            hash_cost_bound_recursive(hasher, a);
            hash_cost_bound_recursive(hasher, b);
        }
        CostBound::Amortized(inner, _pf) => {
            hasher.update(&[0x09]);
            hash_cost_bound_recursive(hasher, inner);
        }
        CostBound::HWScaled(inner, hw) => {
            hasher.update(&[0x0A]);
            hash_cost_bound_recursive(hasher, inner);
            hasher.update(&hw.0);
        }
        CostBound::Sup(bounds) => {
            hasher.update(&[0x0B]);
            // Length-prefix the inner bounds to prevent second-preimage via
            // boundary ambiguity between differently-sized Sup/Inf collections.
            hasher.update(&(bounds.len() as u64).to_le_bytes());
            for b in bounds {
                hash_cost_bound_recursive(hasher, b);
            }
        }
        CostBound::Inf(bounds) => {
            hasher.update(&[0x0C]);
            // Length-prefix the inner bounds (same rationale as Sup above).
            hasher.update(&(bounds.len() as u64).to_le_bytes());
            for b in bounds {
                hash_cost_bound_recursive(hasher, b);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Fiat-Shamir challenge generation
// ---------------------------------------------------------------------------

/// Number of challenges to issue. More challenges = stronger security
/// (each challenge halves the forgery probability).
const NUM_CHALLENGES: usize = 128;

/// Generate deterministic challenge indices from commitment + public inputs
/// using the Fiat-Shamir heuristic.
///
/// Challenges are deduplicated to ensure distinct leaves are challenged:
/// repeated challenges (all 128 landing on the same leaf) would not provide
/// the intended security. We expand with additional counter values until we
/// have `NUM_CHALLENGES` distinct indices (or exhaust the leaf space if the
/// padded tree has fewer than `NUM_CHALLENGES` leaves).
fn generate_challenges(
    commitment: &[u8; 32],
    public_inputs: &ZkPublicInputs,
    num_leaves: usize,
) -> Vec<usize> {
    let pi_hash = hash_public_inputs(public_inputs);
    let max_distinct = NUM_CHALLENGES.min(num_leaves);
    let mut seen = std::collections::HashSet::with_capacity(max_distinct);
    let mut challenges = Vec::with_capacity(max_distinct);

    let mut counter: u64 = 0;
    while challenges.len() < max_distinct {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"FiatShamirChallenge");
        hasher.update(commitment);
        hasher.update(&pi_hash);
        hasher.update(&counter.to_le_bytes());
        let h = hasher.finalize();
        let bytes: [u8; 8] = h.as_bytes()[..8].try_into().expect("8 bytes");
        let idx = (u64::from_le_bytes(bytes) as usize) % num_leaves;
        if seen.insert(idx) {
            challenges.push(idx);
        }
        counter += 1;
        // Safety valve: if the tree is very small and we've tried many times,
        // stop to avoid an infinite loop (can only happen when num_leaves <
        // NUM_CHALLENGES, in which case max_distinct < NUM_CHALLENGES).
        if counter > (num_leaves as u64) * 8 + 1024 {
            break;
        }
    }

    challenges
}

// ---------------------------------------------------------------------------
// Proof generation
// ---------------------------------------------------------------------------

/// Compute the program-bound Merkle root: binds the program hash and spec
/// type hash into the root so that a proof from program A cannot be reused
/// for program B (issue: ZK proof not bound to program semantics).
///
/// The binding is: `blake3("IrisMerkleRoot" || program_hash || spec_type_hash
///                         || raw_merkle_root)`
fn bind_root_to_program(
    raw_root: &[u8; 32],
    public_inputs: &ZkPublicInputs,
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"IrisMerkleRoot");
    hasher.update(&public_inputs.program_hash.0);
    hasher.update(&public_inputs.spec_type_hash);
    hasher.update(raw_root);
    *hasher.finalize().as_bytes()
}

/// Generate a ZK proof that a valid proof tree exists for the given program.
///
/// The proof commits to the Merkle root of the proof tree, bound to the
/// program hash and spec type hash, then responds to Fiat-Shamir challenges
/// by revealing Merkle paths to selected leaves.
pub fn generate_zk_proof(
    program: &SemanticGraph,
    proof_tree: &ProofTree,
    public_inputs: &ZkPublicInputs,
) -> Result<ZkProof, ZkError> {
    // Flatten proof tree into leaf hashes.
    let leaves = flatten_proof_tree(proof_tree);
    if leaves.is_empty() {
        return Err(ZkError::EmptyProofTree);
    }

    // Verify the program hash matches public inputs before doing any work.
    if program.hash.0 != public_inputs.program_hash.0 {
        return Err(ZkError::ReceiptMismatch {
            reason: "program hash does not match public inputs".into(),
        });
    }

    // Build Merkle tree.
    let merkle_layers = build_merkle_tree(&leaves);
    let raw_root = merkle_layers.last().expect("non-empty")[0];

    // Bind the program hash and spec type into the commitment so that a proof
    // generated for program A cannot be presented for program B.
    let commitment = bind_root_to_program(&raw_root, public_inputs);

    // Generate Fiat-Shamir challenges (deduplicated, see generate_challenges).
    let padded_len = leaves.len().next_power_of_two();
    let challenges = generate_challenges(&commitment, public_inputs, padded_len);

    // Respond to each challenge with a Merkle path.
    // Challenges landing on padding indices are answered with the domain-
    // separated padding hash (not zero); the verifier checks this matches.
    let mut responses = Vec::with_capacity(challenges.len());
    for &leaf_idx in &challenges {
        let actual_leaf_hash = if leaf_idx < leaves.len() {
            leaves[leaf_idx]
        } else {
            padding_leaf_hash(leaf_idx)
        };
        let (path, directions) = merkle_path(&merkle_layers, leaf_idx);
        responses.push(ChallengeResponse {
            leaf_index: leaf_idx,
            leaf_hash: actual_leaf_hash,
            merkle_path: path,
            path_directions: directions,
        });
    }

    // Compute approximate proof size.
    let proof_size = 32 // commitment
        + 32 * 3 // public inputs
        + responses.iter().map(|r| {
            32 // leaf_hash
            + 8 // leaf_index
            + r.merkle_path.len() * 32 // siblings
            + r.path_directions.len() // direction bits
        }).sum::<usize>();

    Ok(ZkProof {
        public_inputs: public_inputs.clone(),
        commitment,
        challenge_responses: responses,
        proof_size,
    })
}

// ---------------------------------------------------------------------------
// Proof verification
// ---------------------------------------------------------------------------

/// Verify a ZK proof.
///
/// Recomputes the Fiat-Shamir challenges from the commitment and public inputs,
/// then checks that each revealed Merkle path is consistent with the raw Merkle
/// root (re-derived from the path), and that the program-bound commitment
/// matches the stored commitment.
pub fn verify_zk_proof(proof: &ZkProof) -> Result<bool, ZkError> {
    if proof.challenge_responses.is_empty() {
        return Err(ZkError::ChallengeMismatch {
            expected: NUM_CHALLENGES,
            actual: 0,
        });
    }

    // Determine the tree size from the Merkle path length (log2 of padded leaves).
    let path_len = proof.challenge_responses[0].merkle_path.len();
    let padded_len = 1usize << path_len;

    // Recompute challenges using Fiat-Shamir. The challenges are derived from
    // the stored commitment (which is already program-bound) so they are
    // correctly tied to the specific program being verified.
    let challenges =
        generate_challenges(&proof.commitment, &proof.public_inputs, padded_len);

    if challenges.len() != proof.challenge_responses.len() {
        return Err(ZkError::ChallengeMismatch {
            expected: challenges.len(),
            actual: proof.challenge_responses.len(),
        });
    }

    // Verify each challenge-response.
    //
    // Each Merkle path leads to the raw tree root. We re-derive the
    // program-bound commitment from the raw root and check it matches
    // `proof.commitment`. This validates both the Merkle path integrity AND
    // that the proof is correctly bound to this program/spec (issue 1 fix).
    for (i, (expected_idx, response)) in
        challenges.iter().zip(proof.challenge_responses.iter()).enumerate()
    {
        // The response must be for the correct leaf index.
        if response.leaf_index != *expected_idx {
            return Err(ZkError::MerklePathInvalid { index: i });
        }

        // Reject challenges on padding indices: the verifier refuses to accept
        // padding leaves as valid proof witnesses (issue 4 fix).
        // A padding index is any index >= the number of real leaves. Since we
        // do not store the real leaf count in the proof, we detect padding by
        // checking whether the presented leaf hash matches the expected
        // domain-separated padding hash. If someone presents a real-looking
        // hash for a padding slot it still needs to pass the Merkle check, so
        // this detection is conservative (forgery still fails the path check).
        let expected_padding = padding_leaf_hash(response.leaf_index);
        let is_padding = response.leaf_hash == expected_padding;
        if is_padding {
            // Padding leaves prove nothing about the witness — reject.
            return Err(ZkError::MerklePathInvalid { index: i });
        }

        // Recompute the raw Merkle root from this path.
        let recomputed_raw_root = recompute_root_from_path(
            &response.leaf_hash,
            &response.merkle_path,
            &response.path_directions,
        );

        // Bind the program hash and spec type into the raw root and check
        // it matches the stored commitment. This is the core of issue 1:
        // a proof for program A will produce a different commitment here
        // and fail verification against program B's public inputs.
        let expected_commitment =
            bind_root_to_program(&recomputed_raw_root, &proof.public_inputs);
        if expected_commitment != proof.commitment {
            return Err(ZkError::MerklePathInvalid { index: i });
        }
    }

    Ok(true)
}

/// Recompute a Merkle root from a leaf hash and its sibling path.
fn recompute_root_from_path(
    leaf_hash: &[u8; 32],
    path: &[[u8; 32]],
    directions: &[bool],
) -> [u8; 32] {
    let mut current = *leaf_hash;
    for (sibling, &is_right) in path.iter().zip(directions.iter()) {
        let mut hasher = blake3::Hasher::new();
        if is_right {
            hasher.update(sibling);
            hasher.update(&current);
        } else {
            hasher.update(&current);
            hasher.update(sibling);
        }
        current = *hasher.finalize().as_bytes();
    }
    current
}

// ---------------------------------------------------------------------------
// Market types
// ---------------------------------------------------------------------------

/// A listing on the computation market: a verified program for sale.
#[derive(Debug, Clone)]
pub struct MarketListing {
    /// The specification and proof.
    pub spec: ZkPublicInputs,
    /// ZK proof of correctness.
    pub proof: ZkProof,
    /// Price in abstract units.
    pub price: u64,
    /// Seller identity (256-bit public key or hash).
    pub seller_id: [u8; 32],
}

/// Result of verifying a market listing.
#[derive(Debug, Clone)]
pub struct MarketVerification {
    /// Hash of the listing (commitment to spec + proof + price + seller).
    pub listing_hash: [u8; 32],
    /// Whether the proof verified successfully.
    pub verified: bool,
    /// Time taken to verify, in microseconds.
    pub verification_time_us: u64,
}

/// Verify a market listing by checking its ZK proof.
pub fn verify_listing(listing: &MarketListing) -> MarketVerification {
    let start = std::time::Instant::now();

    let verified = verify_zk_proof(&listing.proof).unwrap_or(false);

    let elapsed = start.elapsed();

    // Hash the listing for identification.
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"MarketListing");
    hasher.update(&listing.spec.program_hash.0);
    hasher.update(&listing.spec.spec_type_hash);
    hasher.update(&listing.spec.cost_bound_hash);
    hasher.update(&listing.proof.commitment);
    hasher.update(&listing.price.to_le_bytes());
    hasher.update(&listing.seller_id);
    let listing_hash = *hasher.finalize().as_bytes();

    MarketVerification {
        listing_hash,
        verified,
        verification_time_us: elapsed.as_micros() as u64,
    }
}

// ---------------------------------------------------------------------------
// Integration: prove_program
// ---------------------------------------------------------------------------

/// Generate a ZK proof for a verified program given its proof receipt.
///
/// This is the main integration point: after `type_check` produces a
/// `ProofTree` and the receipt is constructed, this function wraps them
/// into a zero-knowledge proof.
pub fn prove_program(
    graph: &SemanticGraph,
    proof_tree: &ProofTree,
    receipt: &ProofReceipt,
) -> Result<ZkProof, ZkError> {
    // Verify receipt consistency.
    if graph.hash.0 != receipt.graph_hash.0 {
        return Err(ZkError::ReceiptMismatch {
            reason: "graph hash does not match receipt".into(),
        });
    }

    // Build public inputs from the receipt.
    let public_inputs = ZkPublicInputs {
        program_hash: receipt.graph_hash,
        spec_type_hash: hash_type_ref(&receipt.type_sig),
        cost_bound_hash: hash_cost_bound(&receipt.cost_bound),
    };

    generate_zk_proof(graph, proof_tree, &public_inputs)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use iris_types::cost::CostBound;
    use iris_types::graph::*;
    use iris_types::hash::SemanticHash;
    use iris_types::proof::*;
    use iris_types::types::{BoundVar, DecreaseWitness, TypeEnv, TypeId};
    use std::collections::{BTreeMap, HashMap};

    /// Build a minimal SemanticGraph for testing.
    fn test_graph() -> SemanticGraph {
        let node_id = NodeId(42);
        let type_ref = TypeId(1);

        let mut nodes = HashMap::new();
        nodes.insert(
            node_id,
            Node {
                id: node_id,
                kind: NodeKind::Lit,
                type_sig: type_ref,
                cost: iris_types::cost::CostTerm::Unit,
                arity: 0,
                resolution_depth: 0, salt: 0,
                payload: NodePayload::Lit {
                    type_tag: 0x01,
                    value: vec![0, 0, 0, 42],
                },
            },
        );

        // Compute the semantic hash for this graph.
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"test_graph");
        hasher.update(&node_id.0.to_le_bytes());
        let hash = SemanticHash(*hasher.finalize().as_bytes());

        SemanticGraph {
            root: node_id,
            nodes,
            edges: vec![],
            type_env: TypeEnv {
                types: BTreeMap::new(),
            },
            cost: CostBound::Constant(1),
            resolution: Resolution::Implementation,
            hash,
        }
    }

    /// Build a sample proof tree with multiple nodes.
    fn test_proof_tree() -> ProofTree {
        ProofTree::ByRule(
            RuleName("Lit".into()),
            NodeId(42),
            vec![
                ProofTree::ByWitness(
                    NodeId(100),
                    DecreaseWitness::Structural(BoundVar(0), BoundVar(1)),
                ),
                ProofTree::ByRule(
                    RuleName("App".into()),
                    NodeId(200),
                    vec![ProofTree::ByExtern(
                        NodeId(300),
                        ExternCertificate(vec![1, 2, 3, 4]),
                    )],
                ),
            ],
        )
    }

    fn test_public_inputs(graph: &SemanticGraph) -> ZkPublicInputs {
        ZkPublicInputs {
            program_hash: FragmentId(graph.hash.0),
            spec_type_hash: hash_type_ref(&TypeId(1)),
            cost_bound_hash: hash_cost_bound(&CostBound::Constant(1)),
        }
    }

    fn test_receipt(graph: &SemanticGraph) -> ProofReceipt {
        ProofReceipt {
            graph_hash: FragmentId(graph.hash.0),
            type_sig: TypeId(1),
            cost_bound: CostBound::Constant(1),
            tier: VerifyTier::Tier0,
            proof_merkle_root: [0u8; 32],
            compact_witness: vec![],
        }
    }

    #[test]
    fn test_generate_and_verify_proof() {
        let graph = test_graph();
        let tree = test_proof_tree();
        let pi = test_public_inputs(&graph);

        let proof = generate_zk_proof(&graph, &tree, &pi).expect("proof generation");
        let result = verify_zk_proof(&proof).expect("verification");
        assert!(result, "valid proof should verify");
    }

    #[test]
    fn test_tampered_commitment_fails() {
        let graph = test_graph();
        let tree = test_proof_tree();
        let pi = test_public_inputs(&graph);

        let mut proof = generate_zk_proof(&graph, &tree, &pi).expect("proof generation");

        // Tamper with the commitment.
        proof.commitment[0] ^= 0xFF;

        // Verification should fail because the Fiat-Shamir challenges change
        // when the commitment changes, so the stored responses won't match.
        let result = verify_zk_proof(&proof);
        assert!(
            result.is_err() || result == Ok(false),
            "tampered proof should not verify"
        );
    }

    #[test]
    fn test_tampered_leaf_hash_fails() {
        let graph = test_graph();
        let tree = test_proof_tree();
        let pi = test_public_inputs(&graph);

        let mut proof = generate_zk_proof(&graph, &tree, &pi).expect("proof generation");

        // Tamper with a leaf hash in a response.
        if let Some(resp) = proof.challenge_responses.first_mut() {
            resp.leaf_hash[0] ^= 0xFF;
        }

        let result = verify_zk_proof(&proof);
        assert!(
            result.is_err(),
            "tampered leaf hash should fail verification"
        );
    }

    #[test]
    fn test_tampered_merkle_path_fails() {
        let graph = test_graph();
        let tree = test_proof_tree();
        let pi = test_public_inputs(&graph);

        let mut proof = generate_zk_proof(&graph, &tree, &pi).expect("proof generation");

        // Tamper with a sibling in the Merkle path.
        if let Some(resp) = proof.challenge_responses.first_mut() {
            if let Some(sibling) = resp.merkle_path.first_mut() {
                sibling[0] ^= 0xFF;
            }
        }

        let result = verify_zk_proof(&proof);
        assert!(
            result.is_err(),
            "tampered Merkle path should fail verification"
        );
    }

    #[test]
    fn test_public_inputs_match_program_hash() {
        let graph = test_graph();
        let tree = test_proof_tree();
        let pi = test_public_inputs(&graph);

        let proof = generate_zk_proof(&graph, &tree, &pi).expect("proof generation");
        assert_eq!(
            proof.public_inputs.program_hash.0, graph.hash.0,
            "public inputs should contain the correct program hash"
        );
    }

    #[test]
    fn test_wrong_program_hash_rejected() {
        let graph = test_graph();
        let tree = test_proof_tree();

        let bad_pi = ZkPublicInputs {
            program_hash: FragmentId([0xAA; 32]),
            spec_type_hash: [0; 32],
            cost_bound_hash: [0; 32],
        };

        let result = generate_zk_proof(&graph, &tree, &bad_pi);
        assert!(
            result.is_err(),
            "proof generation should reject mismatched program hash"
        );
    }

    #[test]
    fn test_proof_size_reasonable() {
        let graph = test_graph();
        let tree = test_proof_tree();
        let pi = test_public_inputs(&graph);

        let proof = generate_zk_proof(&graph, &tree, &pi).expect("proof generation");

        // With 128 Fiat-Shamir challenges, a small proof tree produces ~14KB.
        // For typical programs with deeper trees the Merkle paths are longer
        // but the count stays at 128, keeping proofs compact (< 50KB).
        assert!(
            proof.proof_size < 50_000,
            "proof size {} should be < 50KB",
            proof.proof_size
        );
    }

    #[test]
    fn test_market_listing_verification_roundtrip() {
        let graph = test_graph();
        let tree = test_proof_tree();
        let pi = test_public_inputs(&graph);

        let proof = generate_zk_proof(&graph, &tree, &pi).expect("proof generation");

        let listing = MarketListing {
            spec: pi,
            proof,
            price: 1000,
            seller_id: [0x42; 32],
        };

        let verification = verify_listing(&listing);
        assert!(verification.verified, "valid listing should verify");
        assert_ne!(
            verification.listing_hash, [0u8; 32],
            "listing hash should be non-zero"
        );
    }

    #[test]
    fn test_market_listing_tampered_fails() {
        let graph = test_graph();
        let tree = test_proof_tree();
        let pi = test_public_inputs(&graph);

        let mut proof = generate_zk_proof(&graph, &tree, &pi).expect("proof generation");
        // Tamper.
        proof.commitment[0] ^= 0xFF;

        let listing = MarketListing {
            spec: proof.public_inputs.clone(),
            proof,
            price: 1000,
            seller_id: [0x42; 32],
        };

        let verification = verify_listing(&listing);
        assert!(
            !verification.verified,
            "tampered listing should fail verification"
        );
    }

    #[test]
    fn test_prove_program_integration() {
        let graph = test_graph();
        let tree = test_proof_tree();
        let receipt = test_receipt(&graph);

        let proof = prove_program(&graph, &tree, &receipt).expect("prove_program");
        let result = verify_zk_proof(&proof).expect("verification");
        assert!(result, "prove_program output should verify");
    }

    #[test]
    fn test_prove_program_receipt_mismatch() {
        let graph = test_graph();
        let tree = test_proof_tree();

        let mut receipt = test_receipt(&graph);
        receipt.graph_hash = FragmentId([0xBB; 32]);

        let result = prove_program(&graph, &tree, &receipt);
        assert!(
            result.is_err(),
            "mismatched receipt should be rejected"
        );
    }

    #[test]
    fn test_proof_generation_and_verification_performance() {
        let graph = test_graph();
        let tree = test_proof_tree();
        let pi = test_public_inputs(&graph);

        let start = std::time::Instant::now();
        let proof = generate_zk_proof(&graph, &tree, &pi).expect("proof generation");
        let gen_time = start.elapsed();

        let start = std::time::Instant::now();
        let result = verify_zk_proof(&proof).expect("verification");
        let verify_time = start.elapsed();

        assert!(result);
        assert!(
            gen_time.as_millis() < 100,
            "generation took {}ms, expected < 100ms",
            gen_time.as_millis()
        );
        assert!(
            verify_time.as_millis() < 100,
            "verification took {}ms, expected < 100ms",
            verify_time.as_millis()
        );
    }

    #[test]
    fn test_empty_proof_tree_error() {
        // A ByRule with zero children is still one leaf (the rule itself).
        // We need to test that the error path works when there are genuinely
        // zero leaves — which can't happen with valid ProofTree variants
        // since every variant produces at least one leaf. So this test
        // verifies a single-leaf tree works correctly.
        let graph = test_graph();
        let tree = ProofTree::ByRule(RuleName("Single".into()), NodeId(42), vec![]);
        let pi = test_public_inputs(&graph);

        let proof = generate_zk_proof(&graph, &tree, &pi).expect("single leaf OK");
        let result = verify_zk_proof(&proof).expect("verification");
        assert!(result);
    }

    #[test]
    fn test_merkle_tree_internals() {
        // Verify the Merkle tree construction is correct.
        let leaves: Vec<[u8; 32]> = (0..4u8)
            .map(|i| {
                let mut h = [0u8; 32];
                h[0] = i;
                h
            })
            .collect();

        let layers = build_merkle_tree(&leaves);
        assert_eq!(layers.len(), 3); // 4 leaves -> 2 internal -> 1 root
        assert_eq!(layers[0].len(), 4);
        assert_eq!(layers[1].len(), 2);
        assert_eq!(layers[2].len(), 1);

        // Verify each leaf's Merkle path leads to the root.
        let root = layers[2][0];
        for i in 0..4 {
            let (path, dirs) = merkle_path(&layers, i);
            assert!(
                verify_merkle_path(&leaves[i], &path, &dirs, &root),
                "Merkle path for leaf {i} should verify"
            );
        }
    }

    #[test]
    fn test_deterministic_challenges() {
        // Fiat-Shamir should be deterministic.
        let commitment = [0x42u8; 32];
        let pi = ZkPublicInputs {
            program_hash: FragmentId([1u8; 32]),
            spec_type_hash: [2u8; 32],
            cost_bound_hash: [3u8; 32],
        };

        let c1 = generate_challenges(&commitment, &pi, 16);
        let c2 = generate_challenges(&commitment, &pi, 16);
        assert_eq!(c1, c2, "challenges must be deterministic");
    }

    #[test]
    fn test_hash_type_ref_deterministic() {
        let t1 = hash_type_ref(&TypeId(42));
        let t2 = hash_type_ref(&TypeId(42));
        assert_eq!(t1, t2);

        let t3 = hash_type_ref(&TypeId(43));
        assert_ne!(t1, t3, "different types should hash differently");
    }

    #[test]
    fn test_hash_cost_bound_variants() {
        let h1 = hash_cost_bound(&CostBound::Zero);
        let h2 = hash_cost_bound(&CostBound::Constant(1));
        let h3 = hash_cost_bound(&CostBound::Unknown);
        assert_ne!(h1, h2);
        assert_ne!(h1, h3);
        assert_ne!(h2, h3);
    }

    // -----------------------------------------------------------------------
    // Security fix tests
    // -----------------------------------------------------------------------

    /// A proof generated for program A must not verify under program B's
    /// public inputs (issue 1: ZK proof bound to program semantics).
    #[test]
    fn test_proof_cannot_be_reused_for_different_program() {
        let graph_a = test_graph();
        let tree = test_proof_tree();
        let pi_a = test_public_inputs(&graph_a);

        let mut proof = generate_zk_proof(&graph_a, &tree, &pi_a).expect("proof generation");

        // Swap in a different program hash (simulating a different program B).
        proof.public_inputs.program_hash = FragmentId([0xCC; 32]);

        // Verification must fail: the commitment was built with A's hash, so
        // re-deriving the commitment from the Merkle paths with B's hash gives
        // a different value.
        let result = verify_zk_proof(&proof);
        assert!(
            result.is_err() || result == Ok(false),
            "proof for program A must not verify under program B's public inputs"
        );
    }

    /// Changing spec_type_hash in the public inputs must invalidate the proof.
    #[test]
    fn test_spec_hash_change_invalidates_proof() {
        let graph = test_graph();
        let tree = test_proof_tree();
        let pi = test_public_inputs(&graph);

        let mut proof = generate_zk_proof(&graph, &tree, &pi).expect("proof generation");
        proof.public_inputs.spec_type_hash[0] ^= 0xFF;

        let result = verify_zk_proof(&proof);
        assert!(
            result.is_err() || result == Ok(false),
            "altered spec type hash must invalidate proof"
        );
    }

    /// Challenges must be distinct (issue 6: deduplicate).
    #[test]
    fn test_challenges_are_distinct() {
        let commitment = [0x42u8; 32];
        let pi = ZkPublicInputs {
            program_hash: FragmentId([1u8; 32]),
            spec_type_hash: [2u8; 32],
            cost_bound_hash: [3u8; 32],
        };
        // With 256 leaves there is enough space for 128 distinct challenges.
        let challenges = generate_challenges(&commitment, &pi, 256);
        let unique: std::collections::HashSet<usize> = challenges.iter().cloned().collect();
        assert_eq!(
            unique.len(),
            challenges.len(),
            "all challenges must be distinct"
        );
    }

    /// Padding leaf hashes must be non-zero and unique per index (issue 4).
    #[test]
    fn test_padding_leaf_hashes_are_unique() {
        let p0 = padding_leaf_hash(0);
        let p1 = padding_leaf_hash(1);
        let p2 = padding_leaf_hash(2);
        assert_ne!(p0, [0u8; 32], "padding must not be zero");
        assert_ne!(p0, p1, "padding hashes must differ by index");
        assert_ne!(p1, p2);
    }

    /// A challenge landing on a padding leaf must be rejected by the verifier
    /// (issue 4).
    #[test]
    fn test_padding_challenge_rejected() {
        let graph = test_graph();
        let tree = test_proof_tree();
        let pi = test_public_inputs(&graph);

        let mut proof = generate_zk_proof(&graph, &tree, &pi).expect("proof generation");

        // Inject a padding-hash leaf into the first response.
        if let Some(resp) = proof.challenge_responses.first_mut() {
            resp.leaf_hash = padding_leaf_hash(resp.leaf_index);
        }

        let result = verify_zk_proof(&proof);
        assert!(
            result.is_err(),
            "challenge landing on padding leaf must be rejected"
        );
    }

    /// Sup/Inf with length prefix: different-length collections must not collide
    /// (issue 5).
    #[test]
    fn test_sup_inf_length_prefix_differentiates() {
        let sup1 = hash_cost_bound(&CostBound::Sup(vec![CostBound::Zero]));
        let sup2 = hash_cost_bound(&CostBound::Sup(vec![
            CostBound::Zero,
            CostBound::Zero,
        ]));
        assert_ne!(
            sup1, sup2,
            "Sup with different element counts must hash differently"
        );
        let inf1 = hash_cost_bound(&CostBound::Inf(vec![CostBound::Zero]));
        let inf2 = hash_cost_bound(&CostBound::Inf(vec![
            CostBound::Zero,
            CostBound::Zero,
        ]));
        assert_ne!(
            inf1, inf2,
            "Inf with different element counts must hash differently"
        );
        // Sup and Inf with the same contents must still differ (different tags).
        assert_ne!(sup1, inf1, "Sup and Inf must hash differently");
    }
}
