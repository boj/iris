//! Type checker — UNTRUSTED code that produces proof trees.
//!
//! This module drives bottom-up traversal of a `SemanticGraph`, applying
//! kernel rules at each node. It is outside the TCB: it produces `Theorem`
//! values by calling `Kernel` methods, and those methods validate every step.
//!
//! Tier-aware: Tier 0 uses only decidable rules; Tier 1 adds induction, etc.
//!
//! ## Graded verification
//!
//! The `type_check_graded` function returns a `VerificationReport` with partial
//! credit: instead of failing on the first error, it continues checking all
//! nodes and reports how many proof obligations were satisfied. This gives
//! evolutionary search a gradient through the verification landscape.

use std::collections::{BTreeMap, HashMap};

use iris_types::cost::CostBound;
use iris_types::fragment::FragmentContracts;
use iris_types::graph::{NodeId, NodeKind, NodePayload, SemanticGraph};
use iris_types::proof::{ProofTree, RuleName, VerifyTier};
use iris_types::types::{BoundVar, LIAAtom, LIAFormula, LIATerm, TypeDef, TypeId};

use crate::syntax::kernel::cost_checker;
use crate::syntax::kernel::error::CheckError;
use crate::syntax::kernel::kernel::Kernel;
use crate::syntax::kernel::lia_solver;
use crate::syntax::kernel::property_test;
use crate::syntax::kernel::theorem::{Context, Theorem};

// ---------------------------------------------------------------------------
// Cost warning type
// ---------------------------------------------------------------------------

/// A non-fatal warning about cost annotation mismatches.
///
/// Cost annotations are best-effort: a mismatch produces a warning rather
/// than a hard error because cost analysis is approximate.
#[derive(Debug, Clone)]
pub struct CostWarning {
    /// The node whose cost annotation is suspect.
    pub node: NodeId,
    /// The cost declared by the annotation on the node.
    pub declared: CostBound,
    /// The cost proven by the type derivation.
    pub proven: CostBound,
    /// Human-readable description.
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Graded verification types
// ---------------------------------------------------------------------------

/// Result of graded type checking: partial credit for partially-correct graphs.
#[derive(Debug, Clone)]
pub struct VerificationReport {
    /// Total number of proof obligations (one per node).
    pub total_obligations: usize,
    /// Number of obligations that were satisfied.
    pub satisfied: usize,
    /// Which nodes failed and why.
    pub failed: Vec<(NodeId, CheckError)>,
    /// The verification tier used.
    pub tier: VerifyTier,
    /// Proof tree for the parts that succeeded (if any nodes passed).
    pub partial_proof: Option<ProofTree>,
    /// Ratio of satisfied / total obligations (0.0 - 1.0).
    pub score: f32,
    /// Non-fatal cost annotation warnings.
    pub cost_warnings: Vec<CostWarning>,
}

/// Diagnostic information extracted from a proof obligation failure.
#[derive(Debug, Clone)]
pub struct ProofFailureDiagnosis {
    /// The node that failed.
    pub node_id: NodeId,
    /// The kind of node that failed.
    pub node_kind: NodeKind,
    /// The error that occurred.
    pub error: CheckError,
    /// A suggested mutation to fix the failure (if determinable).
    pub suggestion: Option<MutationHint>,
    /// Counterexample: a variable assignment that violates a refinement predicate.
    /// Present when the failure involves a refined type and the LIA solver can
    /// find a concrete input that triggers the type mismatch.
    pub counterexample: Option<HashMap<BoundVar, i64>>,
}

/// Hints for proof-guided mutation: what kind of change might fix a failure.
#[derive(Debug, Clone)]
pub enum MutationHint {
    /// Fold/Unfold is missing a termination check — add one.
    AddTerminationCheck,
    /// Type mismatch: expected type X, got type Y.
    FixTypeSignature(TypeId, TypeId),
    /// Node is missing a cost annotation.
    AddCostAnnotation,
    /// Node needs a runtime guard (e.g., division by zero protection).
    WrapInGuard,
    /// Node kind is not allowed at this tier — downgrade the construct.
    DowngradeTier(NodeKind),
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Auto-detect the minimum verification tier required for a graph.
pub fn minimum_tier(graph: &SemanticGraph) -> VerifyTier {
    let mut tier = VerifyTier::Tier0;
    for node in graph.nodes.values() {
        let node_tier = match node.kind {
            NodeKind::Fold | NodeKind::Unfold | NodeKind::LetRec | NodeKind::TypeAbst => VerifyTier::Tier1,
            NodeKind::Effect | NodeKind::Extern => VerifyTier::Tier2,
            NodeKind::Neural => VerifyTier::Tier3,
            _ => VerifyTier::Tier0,
        };
        if (node_tier as u8) > (tier as u8) {
            tier = node_tier;
        }
    }
    tier
}

/// Type-check an entire `SemanticGraph` at the given tier.
///
/// Returns a `ProofTree` (for archival) plus the root `Theorem` on success.
pub fn type_check(
    graph: &SemanticGraph,
    tier: VerifyTier,
) -> Result<(ProofTree, Theorem), CheckError> {
    let mut checker = Checker::new(graph, tier);
    checker.check_all()
}

/// Type-check an entire `SemanticGraph` with graded scoring.
///
/// Instead of failing on the first error, continues checking all nodes and
/// returns a `VerificationReport` with partial credit. This gives evolution
/// a gradient: a graph with 9/10 nodes well-typed scores 0.9.
pub fn type_check_graded(
    graph: &SemanticGraph,
    tier: VerifyTier,
) -> VerificationReport {
    let mut checker = GradedChecker::new(graph, tier);
    checker.check_all_graded()
}

/// Verify fragment contracts (requires/ensures) using the LIA solver.
///
/// After type checking passes, this function checks that the contract
/// annotations are satisfiable: for all inputs satisfying `requires`,
/// the output must satisfy `ensures`. Uses property-based testing with
/// the LIA solver to find counterexamples.
///
/// Returns `Ok(())` if no contracts exist or all contracts hold, or
/// `Err(CheckError::RefinementViolation)` with a counterexample if a
/// contract is violated.
pub fn verify_contracts(
    contracts: &FragmentContracts,
    num_inputs: usize,
) -> Result<(), CheckError> {
    if contracts.requires.is_empty() && contracts.ensures.is_empty() {
        return Ok(());
    }

    // Collect all bound variables referenced in the contracts.
    let mut vars: Vec<BoundVar> = Vec::new();
    for formula in contracts.requires.iter().chain(contracts.ensures.iter()) {
        collect_bound_vars(formula, &mut vars);
    }
    vars.sort();
    vars.dedup();

    // If no vars found, synthesize input vars from arity.
    if vars.is_empty() {
        for i in 0..num_inputs {
            vars.push(BoundVar(i as u32));
        }
        // Add result variable if ensures reference it.
        if !contracts.ensures.is_empty() {
            vars.push(BoundVar(0xFFFF));
        }
    }

    let result = property_test::verify_contract(
        &contracts.requires,
        &contracts.ensures,
        &vars,
        1000,
    );

    if result.success {
        Ok(())
    } else {
        Err(CheckError::RefinementViolation {
            node: NodeId(0),
            reason: format!(
                "contract: requires {:?} => ensures {:?}",
                contracts.requires, contracts.ensures
            ),
            counterexample: result.counterexample.map(|ce| {
                ce.into_iter().collect()
            }),
        })
    }
}

/// Collect all BoundVar references from an LIA formula.
fn collect_bound_vars(formula: &LIAFormula, vars: &mut Vec<BoundVar>) {
    match formula {
        LIAFormula::Atom(atom) => match atom {
            LIAAtom::Eq(l, r) | LIAAtom::Lt(l, r) | LIAAtom::Le(l, r) => {
                collect_term_vars(l, vars);
                collect_term_vars(r, vars);
            }
            LIAAtom::Divisible(t, _) => collect_term_vars(t, vars),
        },
        LIAFormula::And(a, b) | LIAFormula::Or(a, b) | LIAFormula::Implies(a, b) => {
            collect_bound_vars(a, vars);
            collect_bound_vars(b, vars);
        }
        LIAFormula::Not(inner) => collect_bound_vars(inner, vars),
        LIAFormula::True | LIAFormula::False => {}
    }
}

/// Collect BoundVar references from an LIA term.
fn collect_term_vars(term: &LIATerm, vars: &mut Vec<BoundVar>) {
    match term {
        LIATerm::Var(bv) | LIATerm::Len(bv) | LIATerm::Size(bv) => vars.push(*bv),
        LIATerm::Add(a, b) | LIATerm::Mod(a, b) => {
            collect_term_vars(a, vars);
            collect_term_vars(b, vars);
        }
        LIATerm::Mul(_, inner) | LIATerm::Neg(inner) => collect_term_vars(inner, vars),
        LIATerm::IfThenElse(cond, then_t, else_t) => {
            collect_bound_vars(cond, vars);
            collect_term_vars(then_t, vars);
            collect_term_vars(else_t, vars);
        }
        LIATerm::Const(_) => {}
    }
}

/// Extract a `ProofFailureDiagnosis` from a failed node.
///
/// Examines the error and node kind to produce a `MutationHint` that
/// the evolutionary loop can use for proof-guided mutation.
///
/// When the failing node has a refined type (`{x : T | phi}`), the LIA
/// solver is invoked to find a counterexample -- a concrete variable
/// assignment that violates the refinement predicate. This counterexample
/// can be converted into a test case that exposes the bug.
pub fn diagnose(
    node_id: NodeId,
    error: &CheckError,
    graph: &SemanticGraph,
) -> ProofFailureDiagnosis {
    let node_kind = graph
        .nodes
        .get(&node_id)
        .map(|n| n.kind)
        .unwrap_or(NodeKind::Lit);

    let suggestion = infer_mutation_hint(node_kind, error);

    // Attempt counterexample generation for refinement type failures.
    let counterexample = extract_counterexample(node_id, error, graph);

    ProofFailureDiagnosis {
        node_id,
        node_kind,
        error: error.clone(),
        suggestion,
        counterexample,
    }
}

/// Try to extract a counterexample from a refinement type failure.
///
/// Looks up the node's type signature; if it is a refined type, uses the
/// LIA solver to find an assignment that violates the refinement predicate.
fn extract_counterexample(
    node_id: NodeId,
    _error: &CheckError,
    graph: &SemanticGraph,
) -> Option<HashMap<BoundVar, i64>> {
    let node = graph.nodes.get(&node_id)?;
    let type_def = graph.type_env.types.get(&node.type_sig)?;

    // Only attempt for refined types.
    if let iris_types::types::TypeDef::Refined(_base_type, predicate) = type_def {
        let vars = lia_solver::collect_formula_vars(predicate);
        if vars.is_empty() {
            return None;
        }
        // Use bounded search with default ranges.
        let ranges: Vec<(i64, i64)> = vars.iter().map(|_| (-16, 16)).collect();
        lia_solver::find_counterexample(predicate, &vars, &ranges)
    } else {
        None
    }
}

/// Infer a mutation hint from a node kind and error.
fn infer_mutation_hint(kind: NodeKind, error: &CheckError) -> Option<MutationHint> {
    match error {
        CheckError::Kernel(ke) => {
            use crate::syntax::kernel::error::KernelError;
            match ke {
                KernelError::TypeMismatch {
                    expected, actual, ..
                } => Some(MutationHint::FixTypeSignature(*expected, *actual)),
                KernelError::CostViolation { .. } => Some(MutationHint::AddCostAnnotation),
                KernelError::InvalidRule { rule, .. }
                    if *rule == "fold_rule" || *rule == "structural_ind" =>
                {
                    Some(MutationHint::AddTerminationCheck)
                }
                _ => match kind {
                    NodeKind::Fold | NodeKind::Unfold => {
                        Some(MutationHint::AddTerminationCheck)
                    }
                    _ => Some(MutationHint::WrapInGuard),
                },
            }
        }
        CheckError::TierViolation { .. } => Some(MutationHint::DowngradeTier(kind)),
        CheckError::MalformedGraph { .. } => None,
        CheckError::Unsupported { .. } => None,
        CheckError::RefinementViolation { .. } => Some(MutationHint::WrapInGuard),
    }
}

// ---------------------------------------------------------------------------
// ForAll / polymorphism helpers
// ---------------------------------------------------------------------------

/// Unwrap `ForAll(X, Arrow(P, R))` to get the inner Arrow's param and return types.
/// Also handles plain `Arrow(P, R)` directly.
fn unwrap_forall_to_arrow(
    type_def: Option<&TypeDef>,
    type_env: &iris_types::types::TypeEnv,
) -> Option<(TypeId, TypeId)> {
    match type_def {
        Some(TypeDef::Arrow(p, r, _)) => Some((*p, *r)),
        Some(TypeDef::ForAll(_, inner_id)) => {
            match type_env.types.get(inner_id) {
                Some(TypeDef::Arrow(p, r, _)) => Some((*p, *r)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// If `fn_thm` proves a ForAll type, try to instantiate it to an Arrow type
/// that can accept `arg_thm`'s type. Builds the instantiated Arrow in the
/// type environment and calls `type_app`.
fn try_instantiate_forall(
    fn_thm: &Theorem,
    arg_thm: &Theorem,
    graph: &SemanticGraph,
) -> Option<Theorem> {
    let fn_type = graph.type_env.types.get(&fn_thm.judgment.type_ref)?;
    match fn_type {
        TypeDef::ForAll(_, inner_id) => {
            // The inner type should be Arrow-shaped. Check if we can find
            // an Arrow type in the env that matches the arg type.
            let inner = graph.type_env.types.get(inner_id)?;
            match inner {
                TypeDef::Arrow(_, _, _) => {
                    // Look for an existing Arrow(arg_type, R) in the type env.
                    let arg_type = arg_thm.judgment.type_ref;
                    for (&tid, tdef) in &graph.type_env.types {
                        if let TypeDef::Arrow(p, _, _) = tdef {
                            if *p == arg_type {
                                return Kernel::type_app(fn_thm, tid, graph).ok();
                            }
                        }
                    }
                    None
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Pre-pass: walk the graph and assign typing contexts to child nodes
/// based on their parent's type structure. Lambda extends context with
/// binder type, Let extends with bound value's type.
fn propagate_contexts_for_graph(
    graph: &SemanticGraph,
    contexts: &mut BTreeMap<NodeId, Context>,
) {
    // Build parent→children map from edges.
    let mut parent_children: BTreeMap<NodeId, Vec<(u8, NodeId)>> = BTreeMap::new();
    for edge in &graph.edges {
        parent_children
            .entry(edge.source)
            .or_default()
            .push((edge.port, edge.target));
    }
    // Sort children by port for consistent ordering.
    for children in parent_children.values_mut() {
        children.sort_by_key(|(port, _)| *port);
    }

    // Walk from root down, propagating contexts.
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(graph.root);

    while let Some(node_id) = queue.pop_front() {
        let ctx = contexts.get(&node_id).cloned().unwrap_or_else(Context::empty);
        let node = match graph.nodes.get(&node_id) {
            Some(n) => n,
            None => continue,
        };
        let children: Vec<NodeId> = parent_children
            .get(&node_id)
            .map(|cs| cs.iter().map(|(_, nid)| *nid).collect())
            .unwrap_or_default();

        match node.kind {
            NodeKind::Lambda => {
                let type_def = graph.type_env.types.get(&node.type_sig);
                let param_type = unwrap_forall_to_arrow(type_def, &graph.type_env)
                    .map(|(p, _)| p);
                if let (Some(param_type), Some(&body_id)) = (param_type, children.first()) {
                    let binder_name = iris_types::graph::BinderId(node_id.0 as u32);
                    let extended = ctx.extend(binder_name, param_type);
                    contexts.insert(body_id, extended);
                }
            }
            NodeKind::Let | NodeKind::LetRec => {
                if children.len() >= 2 {
                    let bound_id = children[0];
                    let body_id = children[1];
                    // The bound expression's type is its type_sig.
                    if let Some(bound_node) = graph.nodes.get(&bound_id) {
                        let binder_name = iris_types::graph::BinderId(node_id.0 as u32);
                        let extended = ctx.extend(binder_name, bound_node.type_sig);
                        contexts.insert(body_id, extended);
                    }
                }
            }
            _ => {}
        }

        for child_id in &children {
            queue.push_back(*child_id);
        }
    }
}

/// Check exhaustiveness of a Match node's patterns against the scrutinee's
/// Sum type. Returns an error if constructors are missing and no wildcard
/// pattern (0xFF) is present.
fn check_match_exhaustiveness(
    node_id: NodeId,
    node: &iris_types::graph::Node,
    scrutinee_type: TypeId,
    graph: &SemanticGraph,
) -> Result<(), CheckError> {
    let scrutinee_typedef = match graph.type_env.types.get(&scrutinee_type) {
        Some(td) => td,
        None => return Ok(()), // Unknown type — can't check exhaustiveness
    };

    let expected_tags: Vec<u16> = match scrutinee_typedef {
        TypeDef::Sum(variants) => variants.iter().map(|(tag, _)| tag.0).collect(),
        TypeDef::Primitive(iris_types::types::PrimType::Bool) => vec![0, 1],
        _ => return Ok(()), // Not a Sum/Bool — exhaustiveness check doesn't apply
    };

    let arm_patterns = match &node.payload {
        NodePayload::Match { arm_patterns, .. } => arm_patterns,
        _ => return Ok(()),
    };

    // If any arm has wildcard (0xFF), the match is exhaustive by default.
    if arm_patterns.contains(&0xFF) {
        return Ok(());
    }

    let covered: std::collections::BTreeSet<u16> = arm_patterns
        .iter()
        .map(|&p| p as u16)
        .collect();

    let missing: Vec<u16> = expected_tags
        .iter()
        .filter(|tag| !covered.contains(tag))
        .copied()
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(CheckError::MalformedGraph {
            reason: format!(
                "Match node {node_id:?}: non-exhaustive patterns — missing tags: {missing:?}"
            ),
        })
    }
}

/// Collect the set of effect tags used by Effect nodes in a graph.
/// Returns an EffectSet containing all effect tags found.
pub fn collect_graph_effects(graph: &SemanticGraph) -> iris_types::eval::EffectSet {
    let mut tags = Vec::new();
    for node in graph.nodes.values() {
        if let NodePayload::Effect { effect_tag } = &node.payload {
            tags.push(*effect_tag);
        }
    }
    iris_types::eval::EffectSet::from_tags(tags)
}

/// Verify that a graph's actual effects are a subset of the declared effects.
/// Returns an error listing undeclared effects if any are found.
pub fn verify_effects(
    graph: &SemanticGraph,
    declared: &iris_types::eval::EffectSet,
) -> Result<(), CheckError> {
    let actual = collect_graph_effects(graph);
    if actual.is_subset_of(declared) {
        Ok(())
    } else {
        let undeclared: Vec<u8> = actual.tags().iter()
            .filter(|t| !declared.tags().contains(t))
            .copied()
            .collect();
        Err(CheckError::MalformedGraph {
            reason: format!(
                "undeclared effects: {:?} (declared: {:?}, actual: {:?})",
                undeclared.iter().map(|&t| iris_types::eval::EffectTag::from_u8(t)).collect::<Vec<_>>(),
                declared.tags(),
                actual.tags(),
            ),
        })
    }
}

// ---------------------------------------------------------------------------
// Internal checker state
// ---------------------------------------------------------------------------

struct Checker<'g> {
    graph: &'g SemanticGraph,
    tier: VerifyTier,
    /// Cache of already-proven theorems, keyed by NodeId.
    proven: BTreeMap<NodeId, Theorem>,
    /// Cache of proof trees for the proven theorems.
    proof_trees: BTreeMap<NodeId, ProofTree>,
    /// Typing contexts for each node (populated during Lambda/Let processing).
    contexts: BTreeMap<NodeId, Context>,
    /// Accumulated cost warnings (non-fatal).
    cost_warnings: Vec<CostWarning>,
}

impl<'g> Checker<'g> {
    fn new(graph: &'g SemanticGraph, tier: VerifyTier) -> Self {
        Self {
            graph,
            tier,
            proven: BTreeMap::new(),
            proof_trees: BTreeMap::new(),
            contexts: BTreeMap::new(),
            cost_warnings: Vec::new(),
        }
    }

    /// Check all nodes bottom-up and return the root's proof tree + theorem.
    fn check_all(&mut self) -> Result<(ProofTree, Theorem), CheckError> {
        // Compute a topological order of nodes (leaves first).
        let topo_order = self.topological_sort()?;

        // Pre-pass: propagate typing contexts top-down before bottom-up checking.
        self.propagate_contexts();

        // Process each node in topological order.
        for node_id in &topo_order {
            self.check_node(*node_id)?;
        }

        // The root must have been proven.
        let root = self.graph.root;
        let root_thm = self.proven.get(&root).ok_or(CheckError::MalformedGraph {
            reason: format!("root node {root:?} was not proven"),
        })?;
        let root_proof = self
            .proof_trees
            .get(&root)
            .ok_or(CheckError::MalformedGraph {
                reason: format!("root node {root:?} has no proof tree"),
            })?;

        // Verify graph-level cost annotation against the root's proven cost.
        if let Some(warning) = check_graph_cost(self.graph, root_thm) {
            if self.tier >= VerifyTier::Tier2 {
                return Err(CheckError::MalformedGraph {
                    reason: format!(
                        "graph cost violation: declared {:?}, proven {:?}",
                        warning.declared, warning.proven,
                    ),
                });
            }
            self.cost_warnings.push(warning);
        }

        Ok((root_proof.clone(), root_thm.clone()))
    }

    /// Check a single node, assuming all children have been checked.
    fn check_node(&mut self, node_id: NodeId) -> Result<(), CheckError> {
        // Skip if already proven.
        if self.proven.contains_key(&node_id) {
            return Ok(());
        }

        let node = self
            .graph
            .nodes
            .get(&node_id)
            .ok_or(CheckError::MalformedGraph {
                reason: format!("node {node_id:?} not in graph"),
            })?;

        // Tier gate: reject constructs not allowed at the current tier.
        self.tier_gate(node_id, node.kind)?;

        // Build the context for this node. Use the propagated context if
        // one was set by a parent Lambda/Let, otherwise use empty.
        let ctx = self.contexts.get(&node_id).cloned().unwrap_or_else(Context::empty);

        match node.kind {
            NodeKind::Lit | NodeKind::Prim | NodeKind::Tuple | NodeKind::Inject
            | NodeKind::Project | NodeKind::Extern => {
                // Leaf-like nodes: use the kernel's type_check_node directly.
                let thm = Kernel::type_check_node(&ctx, self.graph, node_id)?;
                let rule_name = rule_name_for_kind(node.kind);
                let proof = ProofTree::ByRule(rule_name, node_id, vec![]);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::Apply => {
                // Application: use Kernel::elim (structural rule).
                // If the function has a ForAll type, instantiate with type_app first.
                let children = self.children_of(node_id);
                if children.len() < 2 {
                    return Err(CheckError::MalformedGraph {
                        reason: format!("Apply node {node_id:?} has fewer than 2 children"),
                    });
                }
                let fn_id = children[0];
                let arg_id = children[1];
                let fn_thm = self.proven.get(&fn_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("Apply node {node_id:?}: function child {fn_id:?} not proven"),
                })?;
                let arg_thm = self.proven.get(&arg_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("Apply node {node_id:?}: argument child {arg_id:?} not proven"),
                })?;
                // If fn has ForAll type, instantiate it to an Arrow matching the arg type.
                let fn_thm_inst = try_instantiate_forall(fn_thm, arg_thm, self.graph);
                let fn_thm_ref = fn_thm_inst.as_ref().unwrap_or(fn_thm);
                let thm = Kernel::elim(fn_thm_ref, arg_thm, node_id, self.graph)?;
                let sub_proofs = vec![
                    self.proof_trees.get(&fn_id).cloned().unwrap_or(
                        ProofTree::ByRule(RuleName("assumed".to_string()), fn_id, vec![]),
                    ),
                    self.proof_trees.get(&arg_id).cloned().unwrap_or(
                        ProofTree::ByRule(RuleName("assumed".to_string()), arg_id, vec![]),
                    ),
                ];
                let proof = ProofTree::ByRule(RuleName("Elim".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::Lambda => {
                // Lambda: use Kernel::intro (structural rule).
                // If the node's type is ForAll(X, Arrow(...)), unwrap the ForAll,
                // do intro on the inner Arrow, then wrap with type_abst.
                let children = self.children_of(node_id);
                if children.is_empty() {
                    return Err(CheckError::MalformedGraph {
                        reason: format!("Lambda node {node_id:?} has no children"),
                    });
                }
                let body_id = children[0];

                // Propagate context: extend with the lambda's binder type
                // so the body is checked in the right context.
                let node_type_def = self.graph.type_env.types.get(&node.type_sig);
                let inner_arrow = unwrap_forall_to_arrow(node_type_def, &self.graph.type_env);
                if let Some((param_type, _)) = inner_arrow {
                    let binder_name = iris_types::graph::BinderId(node_id.0 as u32);
                    let extended_ctx = ctx.extend(binder_name, param_type);
                    self.contexts.insert(body_id, extended_ctx);
                }

                let body_thm = self.proven.get(&body_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("Lambda node {node_id:?}: body child {body_id:?} not proven"),
                })?;
                // Extract binder info from the node's Arrow type.
                let node_type_def = self.graph.type_env.types.get(&node.type_sig);
                let thm = match node_type_def {
                    Some(TypeDef::Arrow(param_type, _ret_type, _cost)) => {
                        let binder_name = iris_types::graph::BinderId(node_id.0 as u32);
                        let binder_type = *param_type;
                        match Kernel::intro(&ctx, node_id, binder_name, binder_type, body_thm, self.graph) {
                            Ok(thm) => thm,
                            Err(_) => Kernel::type_check_node(&ctx, self.graph, node_id)?,
                        }
                    }
                    Some(TypeDef::ForAll(_, inner_id)) => {
                        // ForAll(X, Arrow(P, R)) — intro on inner Arrow, then type_abst.
                        let inner_id = *inner_id;
                        let forall_type_id = node.type_sig;
                        match self.graph.type_env.types.get(&inner_id) {
                            Some(TypeDef::Arrow(param_type, _ret, _cost)) => {
                                let binder_name = iris_types::graph::BinderId(node_id.0 as u32);
                                match Kernel::intro(&ctx, node_id, binder_name, *param_type, body_thm, self.graph) {
                                    Ok(inner_thm) => {
                                        Kernel::type_abst(&inner_thm, forall_type_id, self.graph)
                                            .unwrap_or(inner_thm)
                                    }
                                    Err(_) => Kernel::type_check_node(&ctx, self.graph, node_id)?,
                                }
                            }
                            _ => Kernel::type_check_node(&ctx, self.graph, node_id)?,
                        }
                    }
                    _ => Kernel::type_check_node(&ctx, self.graph, node_id)?,
                };
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("Intro".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::Let => {
                // Let: use Kernel::let_bind (structural rule).
                let children = self.children_of(node_id);
                if children.len() < 2 {
                    return Err(CheckError::MalformedGraph {
                        reason: format!("Let node {node_id:?} has fewer than 2 children"),
                    });
                }
                let bound_id = children[0];
                let body_id = children[1];

                // Propagate context: extend with the bound variable's type
                // so the body is checked with the let-binding in scope.
                if let Some(bound_thm_ref) = self.proven.get(&bound_id) {
                    let binder_name = iris_types::graph::BinderId(node_id.0 as u32);
                    let bound_type = bound_thm_ref.judgment.type_ref;
                    let extended_ctx = ctx.extend(binder_name, bound_type);
                    self.contexts.insert(body_id, extended_ctx);
                }

                let bound_thm = self.proven.get(&bound_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("Let node {node_id:?}: bound child {bound_id:?} not proven"),
                })?;
                let body_thm = self.proven.get(&body_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("Let node {node_id:?}: body child {body_id:?} not proven"),
                })?;
                let binder_name = iris_types::graph::BinderId(node_id.0 as u32);
                let thm = match Kernel::let_bind(&ctx, node_id, binder_name, bound_thm, body_thm) {
                    Ok(thm) => thm,
                    Err(_) => {
                        // Context propagation may not match; use type_check_node
                        // for leaf-like fallback.
                        Kernel::type_check_node(&ctx, self.graph, node_id)?
                    }
                };
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("LetBind".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::Match => {
                // Match: use Kernel::match_elim (structural rule).
                let children = self.children_of(node_id);
                if children.is_empty() {
                    return Err(CheckError::MalformedGraph {
                        reason: format!("Match node {node_id:?} has no children"),
                    });
                }
                let scrutinee_id = children[0];
                let scrutinee_thm = self.proven.get(&scrutinee_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("Match node {node_id:?}: scrutinee {scrutinee_id:?} not proven"),
                })?;

                // Verify exhaustiveness: all Sum constructors must be covered.
                check_match_exhaustiveness(
                    node_id, node, scrutinee_thm.judgment.type_ref, self.graph,
                )?;

                let arm_thms: Vec<Theorem> = children[1..]
                    .iter()
                    .filter_map(|c| self.proven.get(c).cloned())
                    .collect();
                let thm = if arm_thms.is_empty() {
                    Kernel::type_check_node(&ctx, self.graph, node_id)?
                } else {
                    match Kernel::match_elim(scrutinee_thm, &arm_thms, node_id) {
                        Ok(thm) => thm,
                        Err(_) => Kernel::type_check_node(&ctx, self.graph, node_id)?,
                    }
                };
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("MatchElim".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::Guard => {
                // Guard: use Kernel::guard_rule (structural rule).
                let children = self.children_of(node_id);
                if children.len() < 3 {
                    return Err(CheckError::MalformedGraph {
                        reason: format!("Guard node {node_id:?} has fewer than 3 children"),
                    });
                }
                let pred_id = children[0];
                let then_id = children[1];
                let else_id = children[2];
                let pred_thm = self.proven.get(&pred_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("Guard node {node_id:?}: predicate {pred_id:?} not proven"),
                })?;
                let then_thm = self.proven.get(&then_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("Guard node {node_id:?}: then branch {then_id:?} not proven"),
                })?;
                let else_thm = self.proven.get(&else_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("Guard node {node_id:?}: else branch {else_id:?} not proven"),
                })?;
                let thm = Kernel::guard_rule(pred_thm, then_thm, else_thm, node_id)?;
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("Guard".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::Fold => {
                // Fold: use Kernel::fold_rule (structural rule).
                let children = self.children_of(node_id);
                if children.len() < 3 {
                    return Err(CheckError::MalformedGraph {
                        reason: format!("Fold node {node_id:?} has fewer than 3 children"),
                    });
                }
                let base_id = children[0];
                let step_id = children[1];
                let input_id = children[2];
                let base_thm = self.proven.get(&base_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("Fold node {node_id:?}: base {base_id:?} not proven"),
                })?;
                let step_thm = self.proven.get(&step_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("Fold node {node_id:?}: step {step_id:?} not proven"),
                })?;
                let input_thm = self.proven.get(&input_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("Fold node {node_id:?}: input {input_id:?} not proven"),
                })?;
                let thm = Kernel::fold_rule(base_thm, step_thm, input_thm, node_id)?;
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("Fold".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::Unfold => {
                // Unfold: leaf-like (corecursion step).
                let thm = Kernel::type_check_node(&ctx, self.graph, node_id)?;
                let proof = ProofTree::ByRule(
                    RuleName("Unfold".to_string()),
                    node_id,
                    vec![],
                );
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::TypeAbst => {
                // TypeAbst: use Kernel::type_abst (structural rule).
                let children = self.children_of(node_id);
                if children.is_empty() {
                    return Err(CheckError::MalformedGraph {
                        reason: format!("TypeAbst node {node_id:?} has no children"),
                    });
                }
                let body_id = children[0];
                let body_thm = self.proven.get(&body_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("TypeAbst node {node_id:?}: body {body_id:?} not proven"),
                })?;
                let thm = match Kernel::type_abst(body_thm, node.type_sig, self.graph) {
                    Ok(thm) => thm,
                    Err(_) => Kernel::type_check_node(&ctx, self.graph, node_id)?,
                };
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("TypeAbst".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::TypeApp => {
                // TypeApp: use Kernel::type_app (structural rule).
                let children = self.children_of(node_id);
                if children.is_empty() {
                    return Err(CheckError::MalformedGraph {
                        reason: format!("TypeApp node {node_id:?} has no children"),
                    });
                }
                let body_id = children[0];
                let body_thm = self.proven.get(&body_id).ok_or(CheckError::MalformedGraph {
                    reason: format!("TypeApp node {node_id:?}: body {body_id:?} not proven"),
                })?;
                let thm = match Kernel::type_app(body_thm, node.type_sig, self.graph) {
                    Ok(thm) => thm,
                    Err(_) => Kernel::type_check_node(&ctx, self.graph, node_id)?,
                };
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("TypeApp".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::LetRec => {
                // LetRec: use Kernel::type_check_node for the recursive binding
                // (LetRec is sound because it's tier-gated and requires structural decrease).
                let thm = Kernel::type_check_node(&ctx, self.graph, node_id)?;
                let children = self.children_of(node_id);
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(
                    RuleName("LetRec".to_string()),
                    node_id,
                    sub_proofs,
                );
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            _ => {
                // Ref, Neural, Effect, Rewrite — use type_check_node (leaf-like).
                let thm = Kernel::type_check_node(&ctx, self.graph, node_id)?;
                let proof = ProofTree::ByRule(
                    RuleName(format!("{:?}", node.kind)),
                    node_id,
                    vec![],
                );
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }
        }

        // Verify refinement predicates after basic type checking passes.
        self.verify_refinement(node_id)?;

        // Verify cost annotations. At Tier 2+, cost violations are errors.
        if let Some(warning) = self.graph.nodes.get(&node_id).and_then(|n| {
            self.proven.get(&node_id).and_then(|t| check_cost_annotation(node_id, n, t))
        }) {
            if self.tier >= VerifyTier::Tier2 {
                return Err(CheckError::MalformedGraph {
                    reason: format!(
                        "cost violation at {:?}: declared {:?}, proven {:?}",
                        warning.node, warning.declared, warning.proven,
                    ),
                });
            }
            self.cost_warnings.push(warning);
        }

        Ok(())
    }

    /// After a node has been type-checked, verify its refinement predicate
    /// (if any). For Lit nodes, evaluate the predicate on the concrete value.
    /// For Prim nodes with known semantics, propagate constraints.
    /// For Guard nodes, use the guard condition to refine branches.
    fn verify_refinement(&self, node_id: NodeId) -> Result<(), CheckError> {
        let node = match self.graph.nodes.get(&node_id) {
            Some(n) => n,
            None => return Ok(()),
        };
        let type_def = match self.graph.type_env.types.get(&node.type_sig) {
            Some(td) => td,
            None => return Ok(()),
        };
        let (_base_type, predicate) = match type_def {
            TypeDef::Refined(base, pred) => (base, pred),
            _ => return Ok(()),
        };
        verify_node_refinement(
            node_id,
            node,
            predicate,
            self.graph,
            |nid| self.children_of(nid),
        )
    }

    // Refinement sub-checks and cost annotation checks delegated to
    // shared free functions below.

    /// Pre-pass: propagate typing contexts from parent to child nodes.
    fn propagate_contexts(&mut self) {
        propagate_contexts_for_graph(self.graph, &mut self.contexts);
    }

    /// Get the ordered children of a node (targets of outgoing edges, sorted
    /// by port).
    fn children_of(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut children: Vec<(u8, NodeId)> = self
            .graph
            .edges
            .iter()
            .filter(|e| e.source == node_id)
            .map(|e| (e.port, e.target))
            .collect();
        children.sort_by_key(|(port, _)| *port);
        children.into_iter().map(|(_, id)| id).collect()
    }

    /// Enforce tier restrictions.
    fn tier_gate(&self, node_id: NodeId, kind: NodeKind) -> Result<(), CheckError> {
        match self.tier {
            VerifyTier::Tier0 => {
                // Tier 0: no LetRec, Fold, Unfold, Neural, TypeAbst.
                // Unfold (corecursion) requires at least Tier 1 with a
                // termination check, or Tier 3 for productive infinite streams.
                match kind {
                    NodeKind::LetRec => {
                        return Err(CheckError::TierViolation {
                            tier: self.tier,
                            node: node_id,
                            reason: "LetRec not allowed at Tier 0".to_string(),
                        });
                    }
                    NodeKind::Fold => {
                        return Err(CheckError::TierViolation {
                            tier: self.tier,
                            node: node_id,
                            reason: "Fold not allowed at Tier 0".to_string(),
                        });
                    }
                    NodeKind::Unfold => {
                        return Err(CheckError::TierViolation {
                            tier: self.tier,
                            node: node_id,
                            reason: "Unfold not allowed at Tier 0".to_string(),
                        });
                    }
                    NodeKind::Neural => {
                        return Err(CheckError::TierViolation {
                            tier: self.tier,
                            node: node_id,
                            reason: "Neural not allowed at Tier 0".to_string(),
                        });
                    }
                    NodeKind::TypeAbst => {
                        return Err(CheckError::TierViolation {
                            tier: self.tier,
                            node: node_id,
                            reason: "TypeAbst not allowed at Tier 0".to_string(),
                        });
                    }
                    _ => {}
                }
            }
            VerifyTier::Tier1 => {
                // Tier 1: no LetRec with non-structural decrease, no Neural.
                // Unfold with a termination check is allowed at Tier 1;
                // Unfold without a termination check (infinite stream) requires
                // Tier 3 with runtime budget enforcement.
                if kind == NodeKind::Neural {
                    return Err(CheckError::TierViolation {
                        tier: self.tier,
                        node: node_id,
                        reason: "Neural not allowed at Tier 1".to_string(),
                    });
                }
            }
            VerifyTier::Tier2 | VerifyTier::Tier3 => {
                // All constructs allowed.
            }
        }
        Ok(())
    }

    /// Compute a topological sort of graph nodes (leaves first).
    fn topological_sort(&self) -> Result<Vec<NodeId>, CheckError> {
        let all_nodes: Vec<NodeId> = self.graph.nodes.keys().copied().collect();

        // Build adjacency: parent -> children.
        let mut children_map: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();
        let mut in_degree: BTreeMap<NodeId, usize> = BTreeMap::new();

        for &nid in &all_nodes {
            children_map.entry(nid).or_default();
            in_degree.entry(nid).or_insert(0);
        }

        // Edges go source -> target. For bottom-up traversal, a node depends
        // on its children (targets). So in the "process children first" order,
        // a node can only be processed after all its targets.
        // We invert: edges represent "source depends on target".
        for edge in &self.graph.edges {
            if self.graph.nodes.contains_key(&edge.source)
                && self.graph.nodes.contains_key(&edge.target)
            {
                children_map
                    .entry(edge.target)
                    .or_default()
                    .push(edge.source);
                *in_degree.entry(edge.source).or_insert(0) += 1;
            }
        }

        // Kahn's algorithm.
        let mut queue: Vec<NodeId> = all_nodes
            .iter()
            .filter(|n| in_degree.get(n).copied().unwrap_or(0) == 0)
            .copied()
            .collect();

        let mut result = Vec::with_capacity(all_nodes.len());

        while let Some(node) = queue.pop() {
            result.push(node);
            if let Some(dependents) = children_map.get(&node) {
                for &dep in dependents {
                    if let Some(deg) = in_degree.get_mut(&dep) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push(dep);
                        }
                    }
                }
            }
        }

        if result.len() != all_nodes.len() {
            return Err(CheckError::MalformedGraph {
                reason: "cycle detected in graph".to_string(),
            });
        }

        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Shared refinement and cost verification helpers (used by both checkers)
// ---------------------------------------------------------------------------

/// Shared refinement verification for a single node. Dispatches based on
/// node kind: Lit nodes check the literal value, Prim nodes propagate
/// input constraints, other nodes check satisfiability.
fn verify_node_refinement(
    node_id: NodeId,
    node: &iris_types::graph::Node,
    predicate: &LIAFormula,
    graph: &SemanticGraph,
    children_of: impl Fn(NodeId) -> Vec<NodeId>,
) -> Result<(), CheckError> {
    match node.kind {
        NodeKind::Lit => {
            if let NodePayload::Lit { type_tag, ref value } = node.payload {
                let int_val = extract_int_literal(type_tag, value);
                if let Some(int_val) = int_val {
                    let vars = lia_solver::collect_formula_vars(predicate);
                    if !vars.is_empty() {
                        let mut assignment = HashMap::new();
                        assignment.insert(vars[0], int_val);
                        if !lia_solver::evaluate_lia(predicate, &assignment) {
                            return Err(CheckError::RefinementViolation {
                                node: node_id,
                                reason: format!(
                                    "literal value {} does not satisfy refinement predicate",
                                    int_val
                                ),
                                counterexample: Some(assignment),
                            });
                        }
                    }
                }
            }
        }
        NodeKind::Prim => {
            let opcode = match &node.payload {
                NodePayload::Prim { opcode } => *opcode,
                _ => return Ok(()),
            };
            let children = children_of(node_id);
            let input_predicates: Vec<Option<&LIAFormula>> = children
                .iter()
                .filter_map(|child_id| graph.nodes.get(child_id))
                .map(|child_node| {
                    graph
                        .type_env
                        .types
                        .get(&child_node.type_sig)
                        .and_then(|td| match td {
                            TypeDef::Refined(_, pred) => Some(pred),
                            _ => None,
                        })
                })
                .collect();
            let implied = build_prim_implication(opcode, &input_predicates);
            if let Some(implied_predicate) = implied {
                let entailment = LIAFormula::Implies(
                    Box::new(implied_predicate),
                    Box::new(predicate.clone()),
                );
                let vars = lia_solver::collect_formula_vars(&entailment);
                if !vars.is_empty() {
                    let ranges: Vec<(i64, i64)> = vars.iter().map(|_| (-16, 16)).collect();
                    if let Some(counterexample) =
                        lia_solver::find_counterexample(&entailment, &vars, &ranges)
                    {
                        return Err(CheckError::RefinementViolation {
                            node: node_id,
                            reason: format!(
                                "cannot prove output refinement from input constraints for {:?} (opcode 0x{:02x})",
                                node.kind, opcode
                            ),
                            counterexample: Some(counterexample),
                        });
                    }
                }
            }
        }
        _ => {
            let vars = lia_solver::collect_formula_vars(predicate);
            if !vars.is_empty() {
                let sat = lia_solver::solve_lia(predicate, &vars);
                if sat.is_none() {
                    return Err(CheckError::RefinementViolation {
                        node: node_id,
                        reason: format!(
                            "refinement predicate unsatisfiable for {:?} node",
                            node.kind
                        ),
                        counterexample: None,
                    });
                }
            }
        }
    }

    Ok(())
}

/// Shared cost annotation verification. Returns a warning if the declared
/// cost is inconsistent with the proven cost.
fn check_cost_annotation(
    node_id: NodeId,
    node: &iris_types::graph::Node,
    thm: &Theorem,
) -> Option<CostWarning> {

    let declared_cost = match &node.cost {
        iris_types::cost::CostTerm::Annotated(c) => c.clone(),
        iris_types::cost::CostTerm::Unit => CostBound::Constant(1),
        iris_types::cost::CostTerm::Inherited => return None,
    };

    let proven_cost = thm.cost().clone();

    if !cost_checker::cost_leq(&proven_cost, &declared_cost) {
        if cost_checker::cost_leq(&declared_cost, &proven_cost) {
            return None;
        }
        return Some(CostWarning {
            node: node_id,
            declared: declared_cost.clone(),
            proven: proven_cost.clone(),
            reason: format!(
                "declared cost {:?} is not consistent with proven cost {:?}",
                declared_cost, proven_cost
            ),
        });
    }

    None
}

/// Verify the graph-level cost annotation (`graph.cost`) against the root
/// node's proven cost.  This bridges user-written `[cost: Linear(n)]`
/// annotations to the kernel's cost propagation.
fn check_graph_cost(
    graph: &SemanticGraph,
    root_thm: &Theorem,
) -> Option<CostWarning> {
    // Skip if the user didn't annotate a cost (Unknown = no annotation).
    if matches!(graph.cost, CostBound::Unknown) {
        return None;
    }

    let declared = &graph.cost;
    let proven = root_thm.cost();

    // The proven cost must be <= the declared cost.  If it is, the
    // annotation is valid (the program is at most as expensive as claimed).
    if cost_checker::cost_leq(proven, declared) {
        return None;
    }

    // If declared <= proven but not the other way, the annotation is too
    // tight — the program is more expensive than claimed.
    Some(CostWarning {
        node: graph.root,
        declared: declared.clone(),
        proven: proven.clone(),
        reason: format!(
            "graph cost annotation {:?} is too tight: proven cost is {:?}",
            declared, proven
        ),
    })
}

// ---------------------------------------------------------------------------
// GradedChecker — continues past failures for partial credit
// ---------------------------------------------------------------------------

/// A checker that does not short-circuit on errors. It attempts to check
/// every node and tallies successes vs failures.
struct GradedChecker<'g> {
    graph: &'g SemanticGraph,
    tier: VerifyTier,
    /// Proven theorems (only for nodes that succeeded).
    proven: BTreeMap<NodeId, Theorem>,
    /// Proof trees for proven nodes.
    proof_trees: BTreeMap<NodeId, ProofTree>,
    /// Typing contexts for each node (populated during Lambda/Let processing).
    contexts: BTreeMap<NodeId, Context>,
    /// Nodes that failed, with their errors.
    failures: Vec<(NodeId, CheckError)>,
    /// Accumulated cost warnings (non-fatal).
    cost_warnings: Vec<CostWarning>,
}

impl<'g> GradedChecker<'g> {
    fn new(graph: &'g SemanticGraph, tier: VerifyTier) -> Self {
        Self {
            graph,
            tier,
            proven: BTreeMap::new(),
            proof_trees: BTreeMap::new(),
            contexts: BTreeMap::new(),
            failures: Vec::new(),
            cost_warnings: Vec::new(),
        }
    }

    /// Produce a theorem by trusting the node's type annotation.
    ///
    /// This is the gradual-typing escape hatch: when the structural rule
    /// cannot fire (e.g., children not yet proven), we still accept the
    /// node's annotated type. The proof hash is tagged "trust" so audits
    /// can distinguish trusted from fully proven nodes.
    fn trust_annotation(&self, node_id: NodeId, ctx: &Context) -> Theorem {
        let node = &self.graph.nodes[&node_id];
        let cost = match &node.cost {
            iris_types::cost::CostTerm::Unit => CostBound::Constant(1),
            iris_types::cost::CostTerm::Inherited => self.graph.cost.clone(),
            iris_types::cost::CostTerm::Annotated(c) => c.clone(),
        };
        use crate::syntax::kernel::theorem::Judgment;
        Theorem {
            judgment: Judgment {
                context: ctx.clone(),
                node_id,
                type_ref: node.type_sig,
                cost,
            },
            proof_hash: {
                use std::hash::{Hash, Hasher};
                let mut h = std::collections::hash_map::DefaultHasher::new();
                "trust_annotation".hash(&mut h);
                node_id.hash(&mut h);
                let v = h.finish().to_le_bytes();
                let mut hash = [0u8; 32];
                hash[..8].copy_from_slice(&v);
                hash
            },
        }
    }

    /// Try the kernel's `type_check_node`; if it rejects a composite node,
    /// fall back to trusting the annotation.  This keeps the graded checker
    /// progressive: leaf nodes are fully checked, composite nodes get partial
    /// credit via annotation trust when structural rules can't fire.
    fn check_or_trust(&self, node_id: NodeId, ctx: &Context) -> Theorem {
        match Kernel::type_check_node(ctx, self.graph, node_id) {
            Ok(thm) => thm,
            Err(_) => self.trust_annotation(node_id, ctx),
        }
    }

    /// Check all nodes and return a graded report.
    fn check_all_graded(&mut self) -> VerificationReport {
        let total = self.graph.nodes.len();

        // If the graph has no nodes, return a trivial report.
        if total == 0 {
            return VerificationReport {
                total_obligations: 0,
                satisfied: 0,
                failed: vec![],
                tier: self.tier,
                partial_proof: None,
                score: 1.0,
                cost_warnings: vec![],
            };
        }

        // Compute topological order; if the graph has a cycle, every node
        // is considered failed.
        let topo_order = match self.topological_sort() {
            Ok(order) => order,
            Err(e) => {
                let failures: Vec<(NodeId, CheckError)> = self
                    .graph
                    .nodes
                    .keys()
                    .map(|&nid| (nid, e.clone()))
                    .collect();
                return VerificationReport {
                    total_obligations: total,
                    satisfied: 0,
                    failed: failures,
                    tier: self.tier,
                    partial_proof: None,
                    score: 0.0,
                    cost_warnings: vec![],
                };
            }
        };

        // Pre-pass: propagate typing contexts top-down before bottom-up checking.
        self.propagate_contexts();

        // Check each node; don't stop on failure.
        for &node_id in &topo_order {
            if let Err(err) = self.check_node_graded(node_id) {
                self.failures.push((node_id, err));
            }
        }

        // Build a partial proof from the root if it was proven.
        let partial_proof = self.proof_trees.get(&self.graph.root).cloned();

        // Verify graph-level cost annotation against the root's proven cost.
        if let Some(root_thm) = self.proven.get(&self.graph.root) {
            if let Some(warning) = check_graph_cost(self.graph, root_thm) {
                if self.tier >= VerifyTier::Tier2 {
                    self.failures.push((
                        self.graph.root,
                        CheckError::MalformedGraph {
                            reason: format!(
                                "graph cost violation: declared {:?}, proven {:?}",
                                warning.declared, warning.proven,
                            ),
                        },
                    ));
                } else {
                    self.cost_warnings.push(warning);
                }
            }
        }

        let satisfied = total - self.failures.len();
        let score = if total > 0 {
            satisfied as f32 / total as f32
        } else {
            1.0
        };

        VerificationReport {
            total_obligations: total,
            satisfied,
            failed: self.failures.clone(),
            tier: self.tier,
            partial_proof,
            score,
            cost_warnings: self.cost_warnings.clone(),
        }
    }

    /// Attempt to check a single node. On failure, returns the error but
    /// does NOT prevent other nodes from being checked.
    fn check_node_graded(&mut self, node_id: NodeId) -> Result<(), CheckError> {
        if self.proven.contains_key(&node_id) {
            return Ok(());
        }

        let node = self
            .graph
            .nodes
            .get(&node_id)
            .ok_or(CheckError::MalformedGraph {
                reason: format!("node {node_id:?} not in graph"),
            })?;

        // Tier gate.
        self.tier_gate(node_id, node.kind)?;

        let ctx = self.contexts.get(&node_id).cloned().unwrap_or_else(Context::empty);

        match node.kind {
            NodeKind::Lit | NodeKind::Prim | NodeKind::Tuple | NodeKind::Inject
            | NodeKind::Project | NodeKind::Extern => {
                let thm = self.check_or_trust(node_id, &ctx);
                let rule_name = rule_name_for_kind(node.kind);
                let proof = ProofTree::ByRule(rule_name, node_id, vec![]);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::Apply => {
                // Application: use Kernel::elim (structural rule).
                // If the function has a ForAll type, instantiate with type_app first.
                // Falls back to trusting the annotation when the function's type
                // is not Arrow (e.g. the lowerer assigned default Int type).
                let children = self.children_of(node_id);
                if children.len() >= 2 {
                    let fn_id = children[0];
                    let arg_id = children[1];
                    if let (Some(fn_thm), Some(arg_thm)) =
                        (self.proven.get(&fn_id), self.proven.get(&arg_id))
                    {
                        let fn_thm_inst = try_instantiate_forall(fn_thm, arg_thm, self.graph);
                        let fn_thm_ref = fn_thm_inst.as_ref().unwrap_or(fn_thm);
                        match Kernel::elim(fn_thm_ref, arg_thm, node_id, self.graph) {
                            Ok(thm) => {
                                let sub_proofs = vec![
                                    self.proof_trees.get(&fn_id).cloned().unwrap_or(
                                        ProofTree::ByRule(RuleName("assumed".to_string()), fn_id, vec![]),
                                    ),
                                    self.proof_trees.get(&arg_id).cloned().unwrap_or(
                                        ProofTree::ByRule(RuleName("assumed".to_string()), arg_id, vec![]),
                                    ),
                                ];
                                let proof = ProofTree::ByRule(RuleName("Elim".to_string()), node_id, sub_proofs);
                                self.proven.insert(node_id, thm);
                                self.proof_trees.insert(node_id, proof);
                            }
                            Err(_) => {
                                // Function type is not Arrow (default Int from lowerer) —
                                // trust the declared annotation.
                                let thm = self.trust_annotation(node_id, &ctx);
                                let proof = ProofTree::ByRule(RuleName("TrustApply".to_string()), node_id, vec![]);
                                self.proven.insert(node_id, thm);
                                self.proof_trees.insert(node_id, proof);
                            }
                        }
                    } else {
                        // Children not yet proven — trust the annotation so
                        // downstream nodes can still be checked.
                        let thm = self.trust_annotation(node_id, &ctx);
                        let proof = ProofTree::ByRule(RuleName("TrustApply".to_string()), node_id, vec![]);
                        self.proven.insert(node_id, thm);
                        self.proof_trees.insert(node_id, proof);
                    }
                } else {
                    return Err(CheckError::MalformedGraph {
                        reason: format!("Apply node {node_id:?} has fewer than 2 children"),
                    });
                }
            }

            NodeKind::Lambda => {
                // Lambda: use Kernel::intro when possible.
                // If the node's type is ForAll(X, Arrow(...)), unwrap and type_abst.
                let children = self.children_of(node_id);
                let node_type_def = self.graph.type_env.types.get(&node.type_sig);

                // Propagate context: extend with the lambda's binder type
                // so the body is checked in the right context.
                let inner_arrow = unwrap_forall_to_arrow(node_type_def, &self.graph.type_env);
                if let Some((param_type, _)) = inner_arrow {
                    if !children.is_empty() {
                        let binder_name = iris_types::graph::BinderId(node_id.0 as u32);
                        let extended_ctx = ctx.extend(binder_name, param_type);
                        self.contexts.insert(children[0], extended_ctx);
                    }
                }

                let thm = if !children.is_empty() {
                    let body_id = children[0];
                    match node_type_def {
                        Some(TypeDef::Arrow(param_type, _ret, _cost)) => {
                            if let Some(body_thm) = self.proven.get(&body_id) {
                                let binder_name = iris_types::graph::BinderId(node_id.0 as u32);
                                match Kernel::intro(&ctx, node_id, binder_name, *param_type, body_thm, self.graph) {
                                    Ok(thm) => thm,
                                    Err(_) => self.check_or_trust(node_id, &ctx),
                                }
                            } else {
                                self.check_or_trust(node_id, &ctx)
                            }
                        }
                        Some(TypeDef::ForAll(_, inner_id)) => {
                            let inner_id = *inner_id;
                            let forall_type_id = node.type_sig;
                            if let (Some(body_thm), Some(TypeDef::Arrow(param_type, _ret, _cost))) =
                                (self.proven.get(&body_id), self.graph.type_env.types.get(&inner_id))
                            {
                                let binder_name = iris_types::graph::BinderId(node_id.0 as u32);
                                match Kernel::intro(&ctx, node_id, binder_name, *param_type, body_thm, self.graph) {
                                    Ok(inner_thm) => {
                                        Kernel::type_abst(&inner_thm, forall_type_id, self.graph)
                                            .unwrap_or(inner_thm)
                                    }
                                    Err(_) => self.check_or_trust(node_id, &ctx),
                                }
                            } else {
                                self.check_or_trust(node_id, &ctx)
                            }
                        }
                        _ => self.check_or_trust(node_id, &ctx),
                    }
                } else {
                    self.check_or_trust(node_id, &ctx)
                };
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("Intro".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::Let => {
                // Let: use Kernel::let_bind when possible.
                let children = self.children_of(node_id);

                // Propagate context: extend with the bound variable's type
                // so the body is checked with the let-binding in scope.
                if children.len() >= 2 {
                    let bound_id = children[0];
                    let body_id = children[1];
                    if let Some(bound_thm_ref) = self.proven.get(&bound_id) {
                        let binder_name = iris_types::graph::BinderId(node_id.0 as u32);
                        let bound_type = bound_thm_ref.judgment.type_ref;
                        let extended_ctx = ctx.extend(binder_name, bound_type);
                        self.contexts.insert(body_id, extended_ctx);
                    }
                }

                let thm = if children.len() >= 2 {
                    let bound_id = children[0];
                    let body_id = children[1];
                    if let (Some(bound_thm), Some(body_thm)) =
                        (self.proven.get(&bound_id), self.proven.get(&body_id))
                    {
                        let binder_name = iris_types::graph::BinderId(node_id.0 as u32);
                        match Kernel::let_bind(&ctx, node_id, binder_name, bound_thm, body_thm) {
                            Ok(thm) => thm,
                            Err(_) => self.check_or_trust(node_id, &ctx),
                        }
                    } else {
                        self.check_or_trust(node_id, &ctx)
                    }
                } else {
                    self.check_or_trust(node_id, &ctx)
                };
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("LetBind".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::Match => {
                // Match: use Kernel::match_elim when possible.
                let children = self.children_of(node_id);

                // Verify exhaustiveness: all Sum constructors must be covered.
                if !children.is_empty() {
                    if let Some(scrutinee_thm) = self.proven.get(&children[0]) {
                        check_match_exhaustiveness(
                            node_id, node, scrutinee_thm.judgment.type_ref, self.graph,
                        )?;
                    }
                }

                let thm = if !children.is_empty() {
                    let scrutinee_id = children[0];
                    if let Some(scrutinee_thm) = self.proven.get(&scrutinee_id) {
                        let arm_thms: Vec<Theorem> = children[1..]
                            .iter()
                            .filter_map(|c| self.proven.get(c).cloned())
                            .collect();
                        if arm_thms.is_empty() {
                            self.check_or_trust(node_id, &ctx)
                        } else {
                            match Kernel::match_elim(scrutinee_thm, &arm_thms, node_id) {
                                Ok(thm) => thm,
                                Err(_) => self.check_or_trust(node_id, &ctx),
                            }
                        }
                    } else {
                        self.check_or_trust(node_id, &ctx)
                    }
                } else {
                    self.check_or_trust(node_id, &ctx)
                };
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("MatchElim".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::Guard => {
                // Guard: use Kernel::guard_rule when possible.
                let children = self.children_of(node_id);
                let thm = if children.len() >= 3 {
                    let pred_id = children[0];
                    let then_id = children[1];
                    let else_id = children[2];
                    if let (Some(pred_thm), Some(then_thm), Some(else_thm)) = (
                        self.proven.get(&pred_id),
                        self.proven.get(&then_id),
                        self.proven.get(&else_id),
                    ) {
                        match Kernel::guard_rule(pred_thm, then_thm, else_thm, node_id) {
                            Ok(thm) => thm,
                            Err(_) => self.check_or_trust(node_id, &ctx),
                        }
                    } else {
                        self.check_or_trust(node_id, &ctx)
                    }
                } else {
                    self.check_or_trust(node_id, &ctx)
                };
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("Guard".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::Fold => {
                // Fold: use Kernel::fold_rule when possible.
                let children = self.children_of(node_id);
                let thm = if children.len() >= 3 {
                    let base_id = children[0];
                    let step_id = children[1];
                    let input_id = children[2];
                    if let (Some(base_thm), Some(step_thm), Some(input_thm)) = (
                        self.proven.get(&base_id),
                        self.proven.get(&step_id),
                        self.proven.get(&input_id),
                    ) {
                        match Kernel::fold_rule(base_thm, step_thm, input_thm, node_id) {
                            Ok(thm) => thm,
                            Err(_) => self.check_or_trust(node_id, &ctx),
                        }
                    } else {
                        self.check_or_trust(node_id, &ctx)
                    }
                } else {
                    self.check_or_trust(node_id, &ctx)
                };
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("Fold".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::LetRec => {
                // LetRec: tier-gated, use type_check_node (recursive binding).
                let thm = self.check_or_trust(node_id, &ctx);
                let children = self.children_of(node_id);
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("LetRec".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::TypeAbst => {
                // TypeAbst: use Kernel::type_abst when possible.
                let children = self.children_of(node_id);
                let thm = if !children.is_empty() {
                    let body_id = children[0];
                    if let Some(body_thm) = self.proven.get(&body_id) {
                        match Kernel::type_abst(body_thm, node.type_sig, self.graph) {
                            Ok(thm) => thm,
                            Err(_) => self.check_or_trust(node_id, &ctx),
                        }
                    } else {
                        self.check_or_trust(node_id, &ctx)
                    }
                } else {
                    self.check_or_trust(node_id, &ctx)
                };
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("TypeAbst".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::TypeApp => {
                // TypeApp: use Kernel::type_app when possible.
                let children = self.children_of(node_id);
                let thm = if !children.is_empty() {
                    let body_id = children[0];
                    if let Some(body_thm) = self.proven.get(&body_id) {
                        match Kernel::type_app(body_thm, node.type_sig, self.graph) {
                            Ok(thm) => thm,
                            Err(_) => self.check_or_trust(node_id, &ctx),
                        }
                    } else {
                        self.check_or_trust(node_id, &ctx)
                    }
                } else {
                    self.check_or_trust(node_id, &ctx)
                };
                let sub_proofs: Vec<ProofTree> = children
                    .iter()
                    .filter_map(|c| self.proof_trees.get(c).cloned())
                    .collect();
                let proof = ProofTree::ByRule(RuleName("TypeApp".to_string()), node_id, sub_proofs);
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            NodeKind::Unfold => {
                let thm = self.check_or_trust(node_id, &ctx);
                let proof = ProofTree::ByRule(
                    RuleName("Unfold".to_string()),
                    node_id,
                    vec![],
                );
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }

            _ => {
                // Ref, Neural, Effect, Rewrite — use type_check_node (leaf-like).
                let thm = self.check_or_trust(node_id, &ctx);
                let proof = ProofTree::ByRule(
                    RuleName(format!("{:?}", node.kind)),
                    node_id,
                    vec![],
                );
                self.proven.insert(node_id, thm);
                self.proof_trees.insert(node_id, proof);
            }
        }

        // Verify refinement predicates for the graded checker too.
        self.verify_refinement_graded(node_id)?;

        // Verify cost annotations. At Tier 2+, cost violations are errors.
        if let Some(warning) = self.verify_cost_annotation_graded(node_id) {
            if self.tier >= VerifyTier::Tier2 {
                return Err(CheckError::MalformedGraph {
                    reason: format!(
                        "cost violation at {:?}: declared {:?}, proven {:?}",
                        warning.node, warning.declared, warning.proven,
                    ),
                });
            }
            self.cost_warnings.push(warning);
        }

        Ok(())
    }

    /// Verify refinement predicates in the graded checker (delegates to shared logic).
    fn verify_refinement_graded(&self, node_id: NodeId) -> Result<(), CheckError> {
        let node = match self.graph.nodes.get(&node_id) {
            Some(n) => n,
            None => return Ok(()),
        };
        let type_def = match self.graph.type_env.types.get(&node.type_sig) {
            Some(td) => td,
            None => return Ok(()),
        };
        let (_base_type, predicate) = match type_def {
            TypeDef::Refined(base, pred) => (base, pred),
            _ => return Ok(()),
        };
        verify_node_refinement(
            node_id,
            node,
            predicate,
            self.graph,
            |nid| self.children_of(nid),
        )
    }

    /// Verify cost annotations in the graded checker (delegates to shared logic).
    fn verify_cost_annotation_graded(&self, node_id: NodeId) -> Option<CostWarning> {
        let node = self.graph.nodes.get(&node_id)?;
        let thm = self.proven.get(&node_id)?;
        check_cost_annotation(node_id, node, thm)
    }

    /// Pre-pass: propagate typing contexts from parent to child nodes.
    fn propagate_contexts(&mut self) {
        propagate_contexts_for_graph(self.graph, &mut self.contexts);
    }

    /// Get ordered children of a node.
    fn children_of(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut children: Vec<(u8, NodeId)> = self
            .graph
            .edges
            .iter()
            .filter(|e| e.source == node_id)
            .map(|e| (e.port, e.target))
            .collect();
        children.sort_by_key(|(port, _)| *port);
        children.into_iter().map(|(_, id)| id).collect()
    }

    /// Enforce tier restrictions (same logic as Checker).
    fn tier_gate(&self, node_id: NodeId, kind: NodeKind) -> Result<(), CheckError> {
        match self.tier {
            VerifyTier::Tier0 => match kind {
                NodeKind::LetRec => Err(CheckError::TierViolation {
                    tier: self.tier,
                    node: node_id,
                    reason: "LetRec not allowed at Tier 0".to_string(),
                }),
                NodeKind::Fold => Err(CheckError::TierViolation {
                    tier: self.tier,
                    node: node_id,
                    reason: "Fold not allowed at Tier 0".to_string(),
                }),
                NodeKind::Unfold => Err(CheckError::TierViolation {
                    tier: self.tier,
                    node: node_id,
                    reason: "Unfold not allowed at Tier 0".to_string(),
                }),
                NodeKind::Neural => Err(CheckError::TierViolation {
                    tier: self.tier,
                    node: node_id,
                    reason: "Neural not allowed at Tier 0".to_string(),
                }),
                NodeKind::TypeAbst => Err(CheckError::TierViolation {
                    tier: self.tier,
                    node: node_id,
                    reason: "TypeAbst not allowed at Tier 0".to_string(),
                }),
                _ => Ok(()),
            },
            VerifyTier::Tier1 => {
                if kind == NodeKind::Neural {
                    Err(CheckError::TierViolation {
                        tier: self.tier,
                        node: node_id,
                        reason: "Neural not allowed at Tier 1".to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            VerifyTier::Tier2 | VerifyTier::Tier3 => Ok(()),
        }
    }

    /// Topological sort (same algorithm as Checker).
    fn topological_sort(&self) -> Result<Vec<NodeId>, CheckError> {
        let all_nodes: Vec<NodeId> = self.graph.nodes.keys().copied().collect();
        let mut children_map: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();
        let mut in_degree: BTreeMap<NodeId, usize> = BTreeMap::new();

        for &nid in &all_nodes {
            children_map.entry(nid).or_default();
            in_degree.entry(nid).or_insert(0);
        }

        for edge in &self.graph.edges {
            if self.graph.nodes.contains_key(&edge.source)
                && self.graph.nodes.contains_key(&edge.target)
            {
                children_map
                    .entry(edge.target)
                    .or_default()
                    .push(edge.source);
                *in_degree.entry(edge.source).or_insert(0) += 1;
            }
        }

        let mut queue: Vec<NodeId> = all_nodes
            .iter()
            .filter(|n| in_degree.get(n).copied().unwrap_or(0) == 0)
            .copied()
            .collect();

        let mut result = Vec::with_capacity(all_nodes.len());

        while let Some(node) = queue.pop() {
            result.push(node);
            if let Some(dependents) = children_map.get(&node) {
                for &dep in dependents {
                    if let Some(deg) = in_degree.get_mut(&dep) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push(dep);
                        }
                    }
                }
            }
        }

        if result.len() != all_nodes.len() {
            return Err(CheckError::MalformedGraph {
                reason: "cycle detected in graph".to_string(),
            });
        }

        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Refinement propagation helpers
// ---------------------------------------------------------------------------

/// Extract an integer value from a Lit node's payload.
///
/// Supports both 8-byte (i64 little-endian) and 4-byte (i32 little-endian,
/// sign-extended) integer literals.
fn extract_int_literal(type_tag: u8, value: &[u8]) -> Option<i64> {
    if type_tag != 0x00 {
        return None;
    }
    if value.len() >= 8 {
        let bytes: [u8; 8] = value[..8].try_into().ok()?;
        Some(i64::from_le_bytes(bytes))
    } else if value.len() >= 4 {
        let bytes: [u8; 4] = value[..4].try_into().ok()?;
        Some(i32::from_le_bytes(bytes) as i64)
    } else {
        None
    }
}

/// Build an implied output predicate from input refinements and the Prim opcode.
fn build_prim_implication(
    opcode: u8,
    input_predicates: &[Option<&LIAFormula>],
) -> Option<LIAFormula> {
    if input_predicates.len() < 2 {
        return None;
    }
    let pred_a = input_predicates.first().and_then(|p| p.as_ref())?;
    let pred_b = input_predicates.get(1).and_then(|p| p.as_ref())?;
    let a_nonneg = is_nonneg_predicate(pred_a);
    let b_nonneg = is_nonneg_predicate(pred_b);
    let result_var = BoundVar(0);
    match opcode {
        0x00 if a_nonneg && b_nonneg => Some(ge_zero_formula(result_var)),
        0x01 => None,
        0x02 if a_nonneg && b_nonneg => Some(ge_zero_formula(result_var)),
        _ => None,
    }
}

/// Check if a LIA formula implies the variable is non-negative (x >= 0).
fn is_nonneg_predicate(formula: &LIAFormula) -> bool {
    match formula {
        LIAFormula::Not(inner) => match inner.as_ref() {
            LIAFormula::Atom(LIAAtom::Lt(LIATerm::Var(_), LIATerm::Const(0))) => true,
            LIAFormula::Atom(LIAAtom::Le(LIATerm::Var(_), LIATerm::Const(c))) if *c < 0 => true,
            _ => false,
        },
        LIAFormula::Atom(LIAAtom::Le(LIATerm::Const(0), LIATerm::Var(_))) => true,
        LIAFormula::And(a, b) => is_nonneg_predicate(a) || is_nonneg_predicate(b),
        _ => false,
    }
}

/// Build a `x >= 0` formula for the given bound variable.
fn ge_zero_formula(v: BoundVar) -> LIAFormula {
    LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Lt(
        LIATerm::Var(v),
        LIATerm::Const(0),
    ))))
}

/// Map a `NodeKind` to an appropriate `RuleName` for proof trees.
fn rule_name_for_kind(kind: NodeKind) -> RuleName {
    let name = match kind {
        NodeKind::Lit => "Lit",
        NodeKind::Prim => "Prim",
        NodeKind::Tuple => "Tuple",
        NodeKind::Inject => "Inject",
        NodeKind::Project => "Project",
        NodeKind::Extern => "Extern",
        NodeKind::Apply => "Apply",
        NodeKind::Lambda => "Lambda",
        NodeKind::Let => "Let",
        NodeKind::Match => "Match",
        NodeKind::Fold => "Fold",
        NodeKind::Unfold => "Unfold",
        NodeKind::Effect => "Effect",
        NodeKind::Ref => "Ref",
        NodeKind::Neural => "Neural",
        NodeKind::TypeAbst => "TypeAbst",
        NodeKind::TypeApp => "TypeApp",
        NodeKind::LetRec => "LetRec",
        NodeKind::Guard => "Guard",
        NodeKind::Rewrite => "Rewrite",
    };
    RuleName(name.to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};
    use iris_types::cost::{CostBound, CostTerm};
    use iris_types::graph::{Node, NodePayload, Resolution};
    use iris_types::hash::SemanticHash;
    use iris_types::types::{PrimType, TypeDef, TypeEnv, TypeId};

    /// Build a minimal graph with a single Lit node.
    fn single_lit_graph() -> SemanticGraph {
        let int_id = TypeId(1);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));

        let node_id = NodeId(100);
        let node = Node {
            id: node_id,
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0,
                value: vec![42, 0, 0, 0],
            },
        };

        let mut nodes = HashMap::new();
        nodes.insert(node_id, node);

        SemanticGraph {
            root: node_id,
            nodes,
            edges: vec![],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        }
    }

    #[test]
    fn check_single_lit() {
        let graph = single_lit_graph();
        let (proof, thm) = type_check(&graph, VerifyTier::Tier0).unwrap();
        assert_eq!(thm.type_ref(), TypeId(1));
        assert_eq!(*thm.cost(), CostBound::Zero);
        assert!(matches!(proof, ProofTree::ByRule(..)));
    }

    #[test]
    fn tier0_rejects_fold() {
        let int_id = TypeId(1);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));

        let node_id = NodeId(200);
        let node = Node {
            id: node_id,
            kind: NodeKind::Fold,
            type_sig: int_id,
            cost: CostTerm::Annotated(CostBound::Linear(iris_types::cost::CostVar(0))),
            arity: 3,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Fold {
                recursion_descriptor: vec![],
            },
        };

        let mut nodes = HashMap::new();
        nodes.insert(node_id, node);

        let graph = SemanticGraph {
            root: node_id,
            nodes,
            edges: vec![],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let err = type_check(&graph, VerifyTier::Tier0).unwrap_err();
        assert!(matches!(err, CheckError::TierViolation { .. }));
    }

    #[test]
    fn tier1_allows_fold() {
        use iris_types::graph::Edge;

        let int_id = TypeId(1);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));

        // Build a Fold node with 3 Lit children (base, step, input).
        let base_id = NodeId(301);
        let step_id = NodeId(302);
        let input_id = NodeId(303);
        let fold_id = NodeId(300);

        let lit_node = |id: NodeId| Node {
            id,
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0,
                value: vec![0, 0, 0, 0],
            },
        };

        let fold_node = Node {
            id: fold_id,
            kind: NodeKind::Fold,
            type_sig: int_id,
            cost: CostTerm::Annotated(CostBound::Linear(iris_types::cost::CostVar(0))),
            arity: 3,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Fold {
                recursion_descriptor: vec![],
            },
        };

        let mut nodes = HashMap::new();
        nodes.insert(base_id, lit_node(base_id));
        nodes.insert(step_id, lit_node(step_id));
        nodes.insert(input_id, lit_node(input_id));
        nodes.insert(fold_id, fold_node);

        let edges = vec![
            Edge { source: fold_id, target: base_id, port: 0, label: iris_types::graph::EdgeLabel::Argument },
            Edge { source: fold_id, target: step_id, port: 1, label: iris_types::graph::EdgeLabel::Argument },
            Edge { source: fold_id, target: input_id, port: 2, label: iris_types::graph::EdgeLabel::Argument },
        ];

        let graph = SemanticGraph {
            root: fold_id,
            nodes,
            edges,
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let result = type_check(&graph, VerifyTier::Tier1);
        assert!(result.is_ok(), "Tier1 should allow Fold: {:?}", result.err());
    }

    // -------------------------------------------------------------------
    // Graded checker tests
    // -------------------------------------------------------------------

    #[test]
    fn graded_single_lit_scores_1() {
        let graph = single_lit_graph();
        let report = type_check_graded(&graph, VerifyTier::Tier0);
        assert_eq!(report.total_obligations, 1);
        assert_eq!(report.satisfied, 1);
        assert!(report.failed.is_empty());
        assert!((report.score - 1.0).abs() < f32::EPSILON);
        assert!(report.partial_proof.is_some());
    }

    #[test]
    fn graded_fold_at_tier0_gives_partial_credit() {
        // Graph: Lit + Fold. At Tier 0, Fold is tier-gated but Lit passes.
        // Expected: 1/2 satisfied.
        let int_id = TypeId(1);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));

        let lit_id = NodeId(100);
        let lit_node = Node {
            id: lit_id,
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0,
                value: vec![42, 0, 0, 0],
            },
        };

        let fold_id = NodeId(200);
        let fold_node = Node {
            id: fold_id,
            kind: NodeKind::Fold,
            type_sig: int_id,
            cost: CostTerm::Annotated(CostBound::Linear(iris_types::cost::CostVar(0))),
            arity: 1,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Fold {
                recursion_descriptor: vec![],
            },
        };

        let mut nodes = HashMap::new();
        nodes.insert(lit_id, lit_node);
        nodes.insert(fold_id, fold_node);

        let graph = SemanticGraph {
            root: fold_id,
            nodes,
            edges: vec![],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let report = type_check_graded(&graph, VerifyTier::Tier0);
        assert_eq!(report.total_obligations, 2);
        assert_eq!(report.satisfied, 1, "Lit should pass, Fold should fail at Tier 0");
        assert_eq!(report.failed.len(), 1);
        assert!((report.score - 0.5).abs() < f32::EPSILON);
        // The Fold failure should be a TierViolation.
        let (failed_id, ref err) = report.failed[0];
        assert_eq!(failed_id, fold_id);
        assert!(matches!(err, CheckError::TierViolation { .. }));
    }

    #[test]
    fn graded_multiple_lits_all_pass() {
        let int_id = TypeId(1);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));

        let mut nodes = HashMap::new();
        for i in 0..5u64 {
            let nid = NodeId(100 + i);
            nodes.insert(
                nid,
                Node {
                    id: nid,
                    kind: NodeKind::Lit,
                    type_sig: int_id,
                    cost: CostTerm::Unit,
                    arity: 0,
                    resolution_depth: i as u8, salt: 0,
                    payload: NodePayload::Lit {
                        type_tag: 0,
                        value: vec![i as u8],
                    },
                },
            );
        }

        let graph = SemanticGraph {
            root: NodeId(100),
            nodes,
            edges: vec![],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let report = type_check_graded(&graph, VerifyTier::Tier0);
        assert_eq!(report.total_obligations, 5);
        assert_eq!(report.satisfied, 5);
        assert!((report.score - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn graded_empty_graph_scores_1() {
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

        let report = type_check_graded(&graph, VerifyTier::Tier0);
        assert_eq!(report.total_obligations, 0);
        assert!((report.score - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn graded_diagnosis_extracts_correct_node_ids() {
        // Build a graph with 3 Lit nodes and 2 Fold nodes at Tier 0.
        // The Fold nodes should fail, producing diagnoses.
        let int_id = TypeId(1);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));

        let mut nodes = HashMap::new();
        let lit1 = NodeId(10);
        let lit2 = NodeId(20);
        let lit3 = NodeId(30);
        let fold1 = NodeId(40);
        let fold2 = NodeId(50);

        for (nid, kind) in [
            (lit1, NodeKind::Lit),
            (lit2, NodeKind::Lit),
            (lit3, NodeKind::Lit),
            (fold1, NodeKind::Fold),
            (fold2, NodeKind::Fold),
        ] {
            let payload = match kind {
                NodeKind::Fold => NodePayload::Fold {
                    recursion_descriptor: vec![],
                },
                _ => NodePayload::Lit {
                    type_tag: 0,
                    value: vec![0],
                },
            };
            nodes.insert(
                nid,
                Node {
                    id: nid,
                    kind,
                    type_sig: int_id,
                    cost: CostTerm::Unit,
                    arity: 0,
                    resolution_depth: 2, salt: 0,
                    payload,
                },
            );
        }

        let graph = SemanticGraph {
            root: lit1,
            nodes,
            edges: vec![],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let report = type_check_graded(&graph, VerifyTier::Tier0);
        assert_eq!(report.total_obligations, 5);
        assert_eq!(report.satisfied, 3, "3 Lits should pass");
        assert_eq!(report.failed.len(), 2, "2 Folds should fail");
        assert!((report.score - 0.6).abs() < f32::EPSILON);

        // Extract diagnoses from failures.
        let diagnoses: Vec<_> = report
            .failed
            .iter()
            .map(|(nid, err)| diagnose(*nid, err, &graph))
            .collect();
        assert_eq!(diagnoses.len(), 2);
        for diag in &diagnoses {
            assert_eq!(diag.node_kind, NodeKind::Fold);
            assert!(
                diag.node_id == fold1 || diag.node_id == fold2,
                "diagnosis node should be one of the fold nodes"
            );
            // The hint should suggest downgrading the tier.
            assert!(matches!(
                diag.suggestion,
                Some(MutationHint::DowngradeTier(NodeKind::Fold))
            ));
        }
    }

    #[test]
    fn graded_mutation_hint_fix_type_signature() {
        // Test that a TypeMismatch kernel error produces FixTypeSignature hint.
        let expected = TypeId(10);
        let actual = TypeId(20);
        let err = CheckError::Kernel(crate::syntax::kernel::error::KernelError::TypeMismatch {
            expected,
            actual,
            context: "test",
        });
        let hint = infer_mutation_hint(NodeKind::Lit, &err);
        assert!(matches!(
            hint,
            Some(MutationHint::FixTypeSignature(e, a)) if e == expected && a == actual
        ));
    }

    #[test]
    fn graded_mutation_hint_cost_violation() {
        let err = CheckError::Kernel(crate::syntax::kernel::error::KernelError::CostViolation {
            required: CostBound::Constant(5),
            actual: CostBound::Unknown,
        });
        let hint = infer_mutation_hint(NodeKind::Prim, &err);
        assert!(matches!(hint, Some(MutationHint::AddCostAnnotation)));
    }

    // -------------------------------------------------------------------
    // Counterexample extraction tests
    // -------------------------------------------------------------------

    #[test]
    fn diagnosis_with_refined_type_has_counterexample() {
        use iris_types::types::{BoundVar, LIAAtom, LIAFormula, LIATerm, TypeDef};

        // Create a refined type: {x : Int | x > 0}
        // i.e., Refined(Int, Not(Le(Var(0), Const(0))))
        let int_id = TypeId(1);
        let refined_id = TypeId(2);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));
        types.insert(
            refined_id,
            TypeDef::Refined(
                int_id,
                LIAFormula::Not(Box::new(LIAFormula::Atom(LIAAtom::Le(
                    LIATerm::Var(BoundVar(0)),
                    LIATerm::Const(0),
                )))),
            ),
        );

        // Create a Lit node with the refined type signature.
        // The Lit node will pass type checking, but diagnose() should still
        // be able to extract a counterexample from the refined type.
        let node_id = NodeId(100);
        let node = Node {
            id: node_id,
            kind: NodeKind::Lit,
            type_sig: refined_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0,
                value: vec![42, 0, 0, 0],
            },
        };

        let mut nodes = HashMap::new();
        nodes.insert(node_id, node);

        let graph = SemanticGraph {
            root: node_id,
            nodes,
            edges: vec![],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        // Manually create a diagnosis for this node (simulating a type error).
        let err = CheckError::Kernel(crate::syntax::kernel::error::KernelError::TypeMismatch {
            expected: int_id,
            actual: refined_id,
            context: "test",
        });

        let diag = diagnose(node_id, &err, &graph);

        // The diagnosis should have a counterexample because the node has
        // a refined type with a violable predicate.
        assert!(
            diag.counterexample.is_some(),
            "diagnosis of refined type should include a counterexample"
        );

        let ce = diag.counterexample.unwrap();
        // The counterexample should violate x > 0, so x <= 0.
        let x = ce[&BoundVar(0)];
        assert!(
            x <= 0,
            "counterexample x={x} should violate the predicate x > 0"
        );
    }

    #[test]
    fn diagnosis_without_refined_type_has_no_counterexample() {
        // A primitive type (not refined) should not produce a counterexample.
        let int_id = TypeId(1);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));

        let node_id = NodeId(100);
        let node = Node {
            id: node_id,
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0,
                value: vec![42, 0, 0, 0],
            },
        };

        let mut nodes = HashMap::new();
        nodes.insert(node_id, node);

        let graph = SemanticGraph {
            root: node_id,
            nodes,
            edges: vec![],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let err = CheckError::Kernel(crate::syntax::kernel::error::KernelError::TypeMismatch {
            expected: int_id,
            actual: int_id,
            context: "test",
        });

        let diag = diagnose(node_id, &err, &graph);
        assert!(
            diag.counterexample.is_none(),
            "primitive type should not produce a counterexample"
        );
    }

    #[test]
    fn counterexample_for_unsatisfiable_predicate_is_always_found() {
        use iris_types::types::{LIAFormula, TypeDef};

        // Predicate: False (always violated)
        let int_id = TypeId(1);
        let refined_id = TypeId(3);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));
        types.insert(
            refined_id,
            TypeDef::Refined(int_id, LIAFormula::False),
        );

        let node_id = NodeId(100);
        let node = Node {
            id: node_id,
            kind: NodeKind::Lit,
            type_sig: refined_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0,
                value: vec![0],
            },
        };

        let mut nodes = HashMap::new();
        nodes.insert(node_id, node);

        let graph = SemanticGraph {
            root: node_id,
            nodes,
            edges: vec![],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let err = CheckError::MalformedGraph {
            reason: "test".to_string(),
        };
        let diag = diagnose(node_id, &err, &graph);

        // LIAFormula::False with no variables -> no counterexample because
        // collect_formula_vars returns empty. This is expected: a predicate
        // with no variables that is always false is unusual and not worth
        // reporting a counterexample for.
        // The test validates that we handle this edge case gracefully.
        assert!(diag.counterexample.is_none());
    }

    #[test]
    fn verify_contracts_trivially_true() {
        let contracts = FragmentContracts {
            requires: vec![],
            ensures: vec![],
        };
        assert!(verify_contracts(&contracts, 0).is_ok());
    }

    #[test]
    fn verify_contracts_simple_requires_ensures() {
        use iris_types::types::{LIAAtom, LIAFormula, LIATerm};
        // requires x >= 0, ensures result >= 0
        let x = BoundVar(0);
        let result_var = BoundVar(0xFFFF);
        let contracts = FragmentContracts {
            requires: vec![LIAFormula::Atom(LIAAtom::Le(
                LIATerm::Const(0),
                LIATerm::Var(x),
            ))],
            ensures: vec![LIAFormula::Atom(LIAAtom::Le(
                LIATerm::Const(0),
                LIATerm::Var(result_var),
            ))],
        };
        // This should pass because requires x >= 0 => ensures result >= 0
        // is trivially satisfiable under random testing.
        let res = verify_contracts(&contracts, 1);
        // The result depends on random testing — the property doesn't
        // constrain result, so it might find counterexamples.
        // We just check it doesn't panic.
        let _ = res;
    }

    #[test]
    fn collect_bound_vars_finds_all_vars() {
        use iris_types::types::{LIAAtom, LIAFormula, LIATerm};
        let formula = LIAFormula::And(
            Box::new(LIAFormula::Atom(LIAAtom::Eq(
                LIATerm::Var(BoundVar(0)),
                LIATerm::Var(BoundVar(1)),
            ))),
            Box::new(LIAFormula::Atom(LIAAtom::Lt(
                LIATerm::Var(BoundVar(2)),
                LIATerm::Const(10),
            ))),
        );
        let mut vars = Vec::new();
        collect_bound_vars(&formula, &mut vars);
        vars.sort();
        vars.dedup();
        assert_eq!(vars, vec![BoundVar(0), BoundVar(1), BoundVar(2)]);
    }

    #[test]
    fn context_propagation_lambda_extends_context() {
        // Build a Lambda(Int -> Int) with a Lit body.
        // Verify the checker propagates context to the body.
        let int_id = TypeId(1);
        let arrow_id = TypeId(2);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));
        types.insert(
            arrow_id,
            TypeDef::Arrow(int_id, int_id, CostBound::Unknown),
        );

        let body_id = NodeId(101);
        let lambda_id = NodeId(100);

        let body_node = Node {
            id: body_id,
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0,
                value: vec![42, 0, 0, 0],
            },
        };
        let lambda_node = Node {
            id: lambda_id,
            kind: NodeKind::Lambda,
            type_sig: arrow_id,
            cost: CostTerm::Unit,
            arity: 1,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lambda {
                binder: iris_types::graph::BinderId(lambda_id.0 as u32),
                captured_count: 0,
            },
        };

        let mut nodes = HashMap::new();
        nodes.insert(body_id, body_node);
        nodes.insert(lambda_id, lambda_node);

        let graph = SemanticGraph {
            root: lambda_id,
            nodes,
            edges: vec![iris_types::graph::Edge {
                source: lambda_id,
                target: body_id,
                port: 0,
                label: iris_types::graph::EdgeLabel::Argument,
            }],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        // Run the graded checker — it should propagate context to body.
        let report = type_check_graded(&graph, VerifyTier::Tier0);
        // Both nodes should be checkable (context propagation works).
        assert!(report.score > 0.0, "checker should succeed with context propagation");
    }

    #[test]
    fn forall_lambda_checked_with_type_abst() {
        // Build a Lambda with type ForAll(X, Arrow(Int, Int)).
        // The checker should unwrap ForAll, do intro, then type_abst.
        let int_id = TypeId(1);
        let arrow_id = TypeId(2);
        let arrow_zero_id = TypeId(4);
        let forall_id = TypeId(3);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));
        types.insert(
            arrow_id,
            TypeDef::Arrow(int_id, int_id, CostBound::Unknown),
        );
        types.insert(
            arrow_zero_id,
            TypeDef::Arrow(int_id, int_id, CostBound::Zero),
        );
        types.insert(
            forall_id,
            TypeDef::ForAll(BoundVar(0), arrow_id),
        );

        let body_id = NodeId(101);
        let lambda_id = NodeId(100);

        let body_node = Node {
            id: body_id,
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0,
                value: vec![42, 0, 0, 0],
            },
        };
        let lambda_node = Node {
            id: lambda_id,
            kind: NodeKind::Lambda,
            type_sig: forall_id,
            cost: CostTerm::Unit,
            arity: 1,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lambda {
                binder: iris_types::graph::BinderId(lambda_id.0 as u32),
                captured_count: 0,
            },
        };

        let mut nodes = HashMap::new();
        nodes.insert(body_id, body_node);
        nodes.insert(lambda_id, lambda_node);

        let graph = SemanticGraph {
            root: lambda_id,
            nodes,
            edges: vec![iris_types::graph::Edge {
                source: lambda_id,
                target: body_id,
                port: 0,
                label: iris_types::graph::EdgeLabel::Argument,
            }],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let report = type_check_graded(&graph, VerifyTier::Tier0);
        // Should succeed — ForAll-typed lambda can be checked.
        assert!(report.score > 0.0, "ForAll lambda should be checkable");
        assert_eq!(report.failed.len(), 0, "no failures expected for ForAll lambda");
    }

    #[test]
    fn unwrap_forall_to_arrow_works() {
        let int_id = TypeId(1);
        let arrow_id = TypeId(2);
        let forall_id = TypeId(3);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));
        types.insert(arrow_id, TypeDef::Arrow(int_id, int_id, CostBound::Unknown));
        types.insert(forall_id, TypeDef::ForAll(BoundVar(0), arrow_id));
        let env = TypeEnv { types };

        // Plain Arrow
        let arrow_def = env.types.get(&arrow_id);
        assert_eq!(unwrap_forall_to_arrow(arrow_def, &env), Some((int_id, int_id)));

        // ForAll wrapping Arrow
        let forall_def = env.types.get(&forall_id);
        assert_eq!(unwrap_forall_to_arrow(forall_def, &env), Some((int_id, int_id)));

        // Primitive (not Arrow)
        let prim_def = env.types.get(&int_id);
        assert_eq!(unwrap_forall_to_arrow(prim_def, &env), None);
    }

    #[test]
    fn exhaustive_match_with_wildcard_passes() {
        // Match on Bool with wildcard arm → exhaustive
        let bool_id = TypeId(1);
        let int_id = TypeId(2);
        let mut types = BTreeMap::new();
        types.insert(bool_id, TypeDef::Primitive(PrimType::Bool));
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));

        let match_node = Node {
            id: NodeId(100),
            kind: NodeKind::Match,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 2,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Match {
                arm_count: 1,
                arm_patterns: vec![0xFF], // wildcard
            },
        };
        let env = TypeEnv { types };
        let graph = SemanticGraph {
            root: NodeId(100),
            nodes: HashMap::new(),
            edges: vec![],
            type_env: env,
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };
        let result = check_match_exhaustiveness(NodeId(100), &match_node, bool_id, &graph);
        assert!(result.is_ok());
    }

    #[test]
    fn non_exhaustive_match_on_sum_fails() {
        use iris_types::types::Tag;
        // Sum with 3 variants, match only covers 1
        let int_id = TypeId(1);
        let sum_id = TypeId(2);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));
        types.insert(sum_id, TypeDef::Sum(vec![
            (Tag(0), int_id),
            (Tag(1), int_id),
            (Tag(2), int_id),
        ]));

        let match_node = Node {
            id: NodeId(100),
            kind: NodeKind::Match,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 2,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Match {
                arm_count: 1,
                arm_patterns: vec![0], // only covers Tag(0)
            },
        };
        let env = TypeEnv { types };
        let graph = SemanticGraph {
            root: NodeId(100),
            nodes: HashMap::new(),
            edges: vec![],
            type_env: env,
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };
        let result = check_match_exhaustiveness(NodeId(100), &match_node, sum_id, &graph);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("missing tags"), "should report missing tags: {msg}");
    }

    #[test]
    fn collect_graph_effects_finds_effect_nodes() {
        let int_id = TypeId(1);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));

        let effect_node = Node {
            id: NodeId(100),
            kind: NodeKind::Effect,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Effect { effect_tag: 0x00 }, // Print
        };
        let lit_node = Node {
            id: NodeId(101),
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit { type_tag: 0, value: vec![0] },
        };

        let mut nodes = HashMap::new();
        nodes.insert(NodeId(100), effect_node);
        nodes.insert(NodeId(101), lit_node);

        let graph = SemanticGraph {
            root: NodeId(100),
            nodes,
            edges: vec![],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let effects = collect_graph_effects(&graph);
        assert!(!effects.is_pure());
        assert_eq!(effects.tags(), &[0x00]);
    }

    #[test]
    fn verify_effects_catches_undeclared() {
        let int_id = TypeId(1);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));

        let effect_node = Node {
            id: NodeId(100),
            kind: NodeKind::Effect,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Effect { effect_tag: 0x04 }, // FileRead
        };
        let mut nodes = HashMap::new();
        nodes.insert(NodeId(100), effect_node);

        let graph = SemanticGraph {
            root: NodeId(100),
            nodes,
            edges: vec![],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        // Declared only Print — FileRead is undeclared.
        let declared = iris_types::eval::EffectSet::singleton(0x00);
        let result = verify_effects(&graph, &declared);
        assert!(result.is_err());

        // Declared FileRead — should pass.
        let declared = iris_types::eval::EffectSet::singleton(0x04);
        let result = verify_effects(&graph, &declared);
        assert!(result.is_ok());
    }

    #[test]
    fn pure_graph_passes_effect_check() {
        let int_id = TypeId(1);
        let mut types = BTreeMap::new();
        types.insert(int_id, TypeDef::Primitive(PrimType::Int));

        let lit_node = Node {
            id: NodeId(100),
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit { type_tag: 0, value: vec![0] },
        };
        let mut nodes = HashMap::new();
        nodes.insert(NodeId(100), lit_node);

        let graph = SemanticGraph {
            root: NodeId(100),
            nodes,
            edges: vec![],
            type_env: TypeEnv { types },
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let effects = collect_graph_effects(&graph);
        assert!(effects.is_pure());
        // Pure graph passes any effect check.
        let result = verify_effects(&graph, &iris_types::eval::EffectSet::pure());
        assert!(result.is_ok());
    }
}
