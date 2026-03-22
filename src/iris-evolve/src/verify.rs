//! f_verify scoring — graded proof credit + heuristic fallback (SPEC Section 4.9).
//!
//! ## Graded verification (primary)
//!
//! `compute_f_verify_graded` runs the proof kernel's graded type checker
//! on the graph. Instead of binary pass/fail, it returns a score in [0,1]
//! proportional to the fraction of proof obligations satisfied, plus
//! diagnostic information about failures for proof-guided mutation.
//!
//! ## Heuristic fallback
//!
//! The original `compute_f_verify` heuristic (based on graph structure)
//! is retained for contexts where the kernel is not available.

use iris_bootstrap::syntax::kernel::checker;
use iris_types::graph::{NodeKind, SemanticGraph};
use iris_types::proof::VerifyTier as KernelVerifyTier;

// Re-export graded types so iris-evolve consumers don't need iris-kernel directly.
pub use iris_bootstrap::syntax::kernel::checker::{MutationHint, ProofFailureDiagnosis, VerificationReport};

// ---------------------------------------------------------------------------
// Graded proof-credit scoring
// ---------------------------------------------------------------------------

/// Compute f_verify using graded proof credit from the kernel's type checker.
///
/// Returns a score in [0.0, 1.0] proportional to satisfied proof obligations,
/// plus a vector of failure diagnoses for proof-guided mutation.
///
/// The tier is chosen automatically: Tier 1 if the graph contains Fold/LetRec
/// (since those need induction), otherwise Tier 0.
pub fn compute_f_verify_graded(
    graph: &SemanticGraph,
) -> (f32, Vec<ProofFailureDiagnosis>) {
    // Choose the appropriate kernel tier based on graph contents.
    let tier = classify_kernel_tier(graph);
    let report = checker::type_check_graded(graph, tier);

    // Scale the raw score by the tier bonus: Tier 0 gets up to 0.3,
    // Tier 1 gets up to 0.6, reflecting that more complex graphs are
    // worth more when fully verified.
    let tier_ceiling = match tier {
        KernelVerifyTier::Tier0 => 0.3,
        KernelVerifyTier::Tier1 => 0.6,
        KernelVerifyTier::Tier2 => 0.9,
        KernelVerifyTier::Tier3 => 1.0,
    };

    let score = report.score * tier_ceiling;

    let diagnoses: Vec<ProofFailureDiagnosis> = report
        .failed
        .iter()
        .map(|(nid, err)| checker::diagnose(*nid, err, graph))
        .collect();

    (score, diagnoses)
}

/// Map graph structure to a kernel `VerifyTier` for graded checking.
fn classify_kernel_tier(graph: &SemanticGraph) -> KernelVerifyTier {
    let mut has_fold = false;
    let mut has_letrec = false;
    let mut has_neural = false;

    for node in graph.nodes.values() {
        match node.kind {
            NodeKind::Fold | NodeKind::Unfold => has_fold = true,
            NodeKind::LetRec => has_letrec = true,
            NodeKind::Neural => has_neural = true,
            _ => {}
        }
    }

    if has_neural {
        // Neural requires Tier 2+; use Tier 2 for graded checking.
        KernelVerifyTier::Tier2
    } else if has_fold || has_letrec {
        KernelVerifyTier::Tier1
    } else {
        KernelVerifyTier::Tier0
    }
}

// ---------------------------------------------------------------------------
// Heuristic verification tier scoring (original)
// ---------------------------------------------------------------------------

/// Verification tier determined by graph structure heuristic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerifyTier {
    /// No verification possible — contains complex constructs.
    Unverified,
    /// Tier 0: no Fold/LetRec/Neural/Extern — fully automatic, < 10ms.
    Tier0,
    /// Tier 1: has Fold but no Neural — mostly automatic, < 1s.
    Tier1,
}

/// Determine the verification tier for a graph using a lightweight heuristic.
///
/// Gen3 heuristic rules:
/// - If the graph has no Fold/LetRec/Neural/Extern -> Tier 0 -> f_verify = 0.3
/// - If the graph has Fold but no Neural -> Tier 1 -> f_verify = 0.6
/// - Otherwise -> unverified -> f_verify = 0.0
pub fn classify_verify_tier(graph: &SemanticGraph) -> VerifyTier {
    let mut has_fold = false;
    let mut has_letrec = false;
    let mut has_neural = false;
    let mut has_extern = false;

    for node in graph.nodes.values() {
        match node.kind {
            NodeKind::Fold | NodeKind::Unfold => has_fold = true,
            NodeKind::LetRec => has_letrec = true,
            NodeKind::Neural => has_neural = true,
            NodeKind::Extern => has_extern = true,
            _ => {}
        }
    }

    if !has_fold && !has_letrec && !has_neural && !has_extern {
        VerifyTier::Tier0
    } else if has_fold && !has_neural {
        VerifyTier::Tier1
    } else {
        VerifyTier::Unverified
    }
}

/// Convert a verification tier to an f_verify score (SPEC Section 4.9).
///
/// ```text
/// Tier 0   -> f_verify = 0.3
/// Tier 1   -> f_verify = 0.6
/// Tier 2   -> f_verify = 0.9    (not used in Gen3 heuristic)
/// Tier 3   -> f_verify = 1.0    (not used in Gen3 heuristic)
/// Unverified -> f_verify = 0.0
/// ```
pub fn tier_to_f_verify(tier: VerifyTier) -> f32 {
    match tier {
        VerifyTier::Tier0 => 0.3,
        VerifyTier::Tier1 => 0.6,
        VerifyTier::Unverified => 0.0,
    }
}

/// Compute f_verify for a graph in one step.
pub fn compute_f_verify(graph: &SemanticGraph) -> f32 {
    tier_to_f_verify(classify_verify_tier(graph))
}

/// Convert a verification tier to the u8 value stored in IndividualMeta.
pub fn tier_to_u8(tier: VerifyTier) -> u8 {
    match tier {
        VerifyTier::Unverified => 0xFF, // sentinel for unverified
        VerifyTier::Tier0 => 0,
        VerifyTier::Tier1 => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase::iris_evolve_test_helpers::make_fragment_with_kinds;

    #[test]
    fn test_tier0_simple_graph() {
        let fragment = make_fragment_with_kinds(&[NodeKind::Prim, NodeKind::Lit, NodeKind::Prim]);
        let tier = classify_verify_tier(&fragment.graph);
        assert_eq!(tier, VerifyTier::Tier0);
        assert!((tier_to_f_verify(tier) - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_tier1_fold_no_neural() {
        let fragment = make_fragment_with_kinds(&[NodeKind::Lit, NodeKind::Fold]);
        let tier = classify_verify_tier(&fragment.graph);
        assert_eq!(tier, VerifyTier::Tier1);
        assert!((tier_to_f_verify(tier) - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_unverified_neural() {
        let fragment = make_fragment_with_kinds(&[NodeKind::Lit, NodeKind::Neural]);
        let tier = classify_verify_tier(&fragment.graph);
        assert_eq!(tier, VerifyTier::Unverified);
        assert!((tier_to_f_verify(tier) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_unverified_fold_and_neural() {
        let fragment =
            make_fragment_with_kinds(&[NodeKind::Lit, NodeKind::Fold, NodeKind::Neural]);
        let tier = classify_verify_tier(&fragment.graph);
        assert_eq!(tier, VerifyTier::Unverified);
    }

    #[test]
    fn test_tier0_with_lambda_tuple() {
        let fragment = make_fragment_with_kinds(&[
            NodeKind::Lit,
            NodeKind::Lambda,
            NodeKind::Tuple,
            NodeKind::Prim,
        ]);
        let tier = classify_verify_tier(&fragment.graph);
        assert_eq!(tier, VerifyTier::Tier0);
    }

    // -----------------------------------------------------------------------
    // Graded verification tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_graded_single_lit_scores_1() {
        // A single well-typed Lit node should get score 1.0 (scaled by tier).
        let fragment = make_fragment_with_kinds(&[NodeKind::Lit]);
        let (score, diagnoses) = compute_f_verify_graded(&fragment.graph);
        // Tier 0 ceiling is 0.3, score should be 0.3 * 1.0 = 0.3.
        assert!((score - 0.3).abs() < f32::EPSILON, "expected ~0.3, got {score}");
        assert!(diagnoses.is_empty(), "no failures expected");
    }

    #[test]
    fn test_graded_multiple_lits_all_pass() {
        let fragment = make_fragment_with_kinds(&[NodeKind::Lit, NodeKind::Lit, NodeKind::Lit]);
        let (score, diagnoses) = compute_f_verify_graded(&fragment.graph);
        // All nodes pass -> score = tier_ceiling * 1.0 = 0.3
        assert!(score > 0.0, "should have positive score");
        assert!(diagnoses.is_empty(), "no failures expected");
    }

    #[test]
    fn test_graded_fold_at_tier1_passes() {
        // A Fold node at Tier 1 should pass (not tier-gated).
        let fragment = make_fragment_with_kinds(&[NodeKind::Lit, NodeKind::Fold]);
        let (score, diagnoses) = compute_f_verify_graded(&fragment.graph);
        // Tier 1 ceiling is 0.6, should get partial or full credit.
        assert!(score > 0.0, "should have positive score, got {score}");
        // The fold may or may not pass type_check_node, but the Lit should.
        // At minimum 1/2 nodes pass.
    }

    #[test]
    fn test_graded_diagnoses_contain_failing_nodes() {
        // Neural at Tier 2 — may fail type_check_node, producing diagnoses.
        let fragment = make_fragment_with_kinds(&[NodeKind::Lit, NodeKind::Neural]);
        let (_score, diagnoses) = compute_f_verify_graded(&fragment.graph);
        // We don't assert exact failure count since Neural may or may not pass
        // the kernel check, but the API should return valid data.
        for diag in &diagnoses {
            // Each diagnosis should have a valid node_id.
            assert!(
                fragment.graph.nodes.contains_key(&diag.node_id),
                "diagnosis node_id should be in graph"
            );
        }
    }

    #[test]
    fn test_graded_empty_graph_scores_1() {
        use std::collections::{BTreeMap, HashMap};
        use iris_types::cost::CostBound;
        use iris_types::graph::{NodeId, Resolution, SemanticGraph};
        use iris_types::hash::SemanticHash;
        use iris_types::types::TypeEnv;

        let graph = SemanticGraph {
            root: NodeId(0),
            nodes: HashMap::new(),
            edges: vec![],
            type_env: TypeEnv {
                types: BTreeMap::new(),
            },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };
        let (score, diagnoses) = compute_f_verify_graded(&graph);
        // Empty graph -> score 1.0 * tier_ceiling
        assert!(
            (score - 0.3).abs() < f32::EPSILON,
            "empty graph should score tier_ceiling, got {score}"
        );
        assert!(diagnoses.is_empty());
    }
}
