//! Phase detection for the evolutionary loop (SPEC Section 6.6).
//!
//! Three phases are detected from fitness improvement dynamics, population
//! diversity, and stagnation counters. Each phase adjusts mutation rate,
//! crossover rate, and tournament size.

use std::collections::VecDeque;

use crate::individual::Individual;
use crate::population::Phase;

// ---------------------------------------------------------------------------
// PhaseDetector
// ---------------------------------------------------------------------------

/// Detects the current evolutionary phase from population metrics.
pub struct PhaseDetector {
    /// Rolling window of per-generation best fitness improvement.
    improvement_history: VecDeque<f32>,
    /// Rolling window of per-generation population diversity.
    diversity_history: VecDeque<f32>,
    /// Number of consecutive generations without significant improvement.
    pub stagnation_counter: u32,
    /// Window size for smoothing.
    window_size: usize,
    /// Previous generation's best fitness (sum of objectives).
    prev_best: f32,
}

/// Phase-specific evolutionary parameters.
#[derive(Debug, Clone, Copy)]
pub struct PhaseParams {
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub tournament_size: usize,
}

impl PhaseDetector {
    /// Create a new phase detector with the given smoothing window size.
    pub fn new(window_size: usize) -> Self {
        Self {
            improvement_history: VecDeque::with_capacity(window_size),
            diversity_history: VecDeque::with_capacity(window_size),
            stagnation_counter: 0,
            window_size: window_size.max(1),
            prev_best: 0.0,
        }
    }

    /// Detect the current evolutionary phase from population state.
    ///
    /// Rules (from SPEC Section 6.6):
    /// - Exploration: improvement rate > 0.05 OR first 20 generations
    /// - Exploitation: improvement rate < 0.01 AND diversity < 0.3
    /// - SteadyState: everything else
    /// - Stagnation counter > 50: force Exploration (inject diversity)
    pub fn detect(&mut self, population: &[Individual], generation: u64) -> Phase {
        if population.is_empty() {
            return Phase::Exploration;
        }

        // Compute current best fitness (sum of objectives).
        let current_best = population
            .iter()
            .map(|i| i.fitness.values.iter().sum::<f32>())
            .fold(0.0f32, f32::max);

        // Compute improvement from previous generation.
        let improvement = (current_best - self.prev_best).max(0.0);
        self.prev_best = current_best;

        // Push improvement into rolling window.
        if self.improvement_history.len() >= self.window_size {
            self.improvement_history.pop_front();
        }
        self.improvement_history.push_back(improvement);

        // Compute diversity: average pairwise fitness distance (simplified).
        let diversity = compute_diversity(population);
        if self.diversity_history.len() >= self.window_size {
            self.diversity_history.pop_front();
        }
        self.diversity_history.push_back(diversity);

        // Compute smoothed improvement rate.
        let improvement_rate = if self.improvement_history.is_empty() {
            0.0
        } else {
            self.improvement_history.iter().sum::<f32>()
                / self.improvement_history.len() as f32
        };

        let avg_diversity = if self.diversity_history.is_empty() {
            1.0
        } else {
            self.diversity_history.iter().sum::<f32>()
                / self.diversity_history.len() as f32
        };

        // Stagnation counter > 80: force Exploration (more patience for
        // larger populations that may need longer to find structural changes).
        if self.stagnation_counter > 80 {
            self.stagnation_counter = 0;
            return Phase::Exploration;
        }

        // First 20 generations: always Exploration.
        if generation < 20 {
            self.stagnation_counter = 0;
            return Phase::Exploration;
        }

        // Apply rules.
        if improvement_rate > 0.05 {
            self.stagnation_counter = 0;
            Phase::Exploration
        } else if improvement_rate < 0.01 && avg_diversity < 0.3 {
            self.stagnation_counter += 1;
            Phase::Exploitation
        } else {
            if improvement_rate < 0.001 {
                self.stagnation_counter += 1;
            } else {
                self.stagnation_counter = 0;
            }
            Phase::SteadyState
        }
    }
}

/// Get the phase-specific evolutionary parameters.
pub fn phase_params(phase: Phase) -> PhaseParams {
    match phase {
        Phase::Exploration => PhaseParams {
            mutation_rate: 0.95,
            crossover_rate: 0.7,
            tournament_size: 2,
        },
        Phase::SteadyState => PhaseParams {
            mutation_rate: 0.7,
            crossover_rate: 0.5,
            tournament_size: 3,
        },
        Phase::Exploitation => PhaseParams {
            mutation_rate: 0.3,
            crossover_rate: 0.4,
            tournament_size: 4,
        },
    }
}

/// Compute population diversity as the average variance of fitness objectives.
///
/// Returns a value in [0, 1] where higher means more diverse.
pub fn compute_diversity(population: &[Individual]) -> f32 {
    if population.len() < 2 {
        return 1.0;
    }

    let n = population.len() as f32;

    // Compute variance per objective, then average.
    let mut total_variance = 0.0f32;
    for obj in 0..crate::individual::NUM_OBJECTIVES {
        let mean = population.iter().map(|i| i.fitness.values[obj]).sum::<f32>() / n;
        let variance = population
            .iter()
            .map(|i| {
                let diff = i.fitness.values[obj] - mean;
                diff * diff
            })
            .sum::<f32>()
            / n;
        total_variance += variance;
    }

    // Normalize: max theoretical variance per objective is 0.25 (for [0,1] range),
    // so max total is 1.0.
    (total_variance / 4.0 * 4.0).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::individual::{Fitness, Individual};

    fn dummy_individual(fitness_values: [f32; 5]) -> Individual {
        let fragment = iris_evolve_test_helpers::make_dummy_fragment();
        let mut ind = Individual::new(fragment);
        ind.fitness = Fitness { values: fitness_values };
        ind
    }

    #[test]
    fn test_first_20_generations_exploration() {
        let mut detector = PhaseDetector::new(10);
        let pop = vec![dummy_individual([0.5, 0.5, 0.5, 0.5, 0.0])];
        for generation in 0..20 {
            let phase = detector.detect(&pop, generation);
            assert_eq!(phase, Phase::Exploration);
        }
    }

    #[test]
    fn test_phase_params() {
        let p = phase_params(Phase::Exploration);
        assert!((p.mutation_rate - 0.95).abs() < f64::EPSILON);
        assert_eq!(p.tournament_size, 2);

        let p = phase_params(Phase::SteadyState);
        assert!((p.mutation_rate - 0.7).abs() < f64::EPSILON);
        assert_eq!(p.tournament_size, 3);

        let p = phase_params(Phase::Exploitation);
        assert!((p.mutation_rate - 0.3).abs() < f64::EPSILON);
        assert_eq!(p.tournament_size, 4);
    }
}

// Test helpers module — available within the crate only.
#[doc(hidden)]
pub mod iris_evolve_test_helpers {
    use iris_types::fragment::{Boundary, Fragment, FragmentId, FragmentMeta};
    use iris_types::graph::*;
    use iris_types::cost::{CostBound, CostTerm};
    use iris_types::hash::{compute_fragment_id, compute_node_id, SemanticHash};
    use iris_types::types::{PrimType, TypeDef, TypeEnv};
    use std::collections::{BTreeMap, HashMap};

    /// Create a minimal fragment for testing.
    pub fn make_dummy_fragment() -> Fragment {
        let int_def = TypeDef::Primitive(PrimType::Int);
        let int_id = iris_types::hash::compute_type_id(&int_def);
        let mut types = BTreeMap::new();
        types.insert(int_id, int_def);
        let type_env = TypeEnv { types };

        let mut node = Node {
            id: NodeId(0),
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 2, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0,
                value: 42i64.to_le_bytes().to_vec(),
            },
        };
        node.id = compute_node_id(&node);

        let mut nodes = HashMap::new();
        nodes.insert(node.id, node.clone());

        let graph = SemanticGraph {
            root: node.id,
            nodes,
            edges: vec![],
            type_env: type_env.clone(),
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let boundary = Boundary {
            inputs: vec![],
            outputs: vec![(node.id, int_id)],
        };

        let mut fragment = Fragment {
            id: FragmentId([0; 32]),
            graph,
            boundary,
            type_env,
            imports: vec![],
            metadata: FragmentMeta {
                name: None,
                created_at: 0,
                generation: 0,
                lineage_hash: 0,
            },
            proof: None,
            contracts: Default::default(),        };
        fragment.id = compute_fragment_id(&fragment);
        fragment
    }

    /// Create a fragment with multiple nodes of specific kinds.
    pub fn make_fragment_with_kinds(kinds: &[NodeKind]) -> Fragment {
        let int_def = TypeDef::Primitive(PrimType::Int);
        let int_id = iris_types::hash::compute_type_id(&int_def);
        let mut types = BTreeMap::new();
        types.insert(int_id, int_def);
        let type_env = TypeEnv { types };

        let mut nodes = HashMap::new();
        let mut edges = vec![];
        let mut prev_id = None;

        for (i, &kind) in kinds.iter().enumerate() {
            let payload = match kind {
                NodeKind::Prim => NodePayload::Prim { opcode: 0x00 },
                NodeKind::Lit => NodePayload::Lit {
                    type_tag: 0,
                    value: (i as i64).to_le_bytes().to_vec(),
                },
                NodeKind::Fold => NodePayload::Fold {
                    recursion_descriptor: vec![],
                },
                NodeKind::Neural => NodePayload::Neural {
                    guard_spec: iris_types::guard::GuardSpec::default(),
                    weight_blob: iris_types::guard::BlobRef::default(),
                },
                NodeKind::LetRec => NodePayload::LetRec {
                    binder: BinderId(i as u32),
                    decrease: iris_types::types::DecreaseWitness::Structural(
                        iris_types::types::BoundVar(0),
                        iris_types::types::BoundVar(1),
                    ),
                },
                NodeKind::Extern => NodePayload::Extern {
                    name: [0u8; 32],
                    type_sig: int_id,
                },
                NodeKind::Tuple => NodePayload::Tuple,
                NodeKind::Lambda => NodePayload::Lambda {
                    binder: BinderId(0),
                    captured_count: 0,
                },
                _ => NodePayload::Lit {
                    type_tag: 0,
                    value: (i as i64).to_le_bytes().to_vec(),
                },
            };

            let mut node = Node {
                id: NodeId(0),
                kind,
                type_sig: int_id,
                cost: CostTerm::Unit,
                arity: if prev_id.is_some() { 1 } else { 0 },
                resolution_depth: i as u8, salt: 0,
                payload,
            };
            node.id = compute_node_id(&node);

            if let Some(prev) = prev_id {
                edges.push(Edge {
                    source: node.id,
                    target: prev,
                    port: 0,
                    label: EdgeLabel::Argument,
                });
            }

            prev_id = Some(node.id);
            nodes.insert(node.id, node);
        }

        let root = prev_id.unwrap_or(NodeId(0));

        let graph = SemanticGraph {
            root,
            nodes,
            edges,
            type_env: type_env.clone(),
            cost: CostBound::Unknown,
            resolution: Resolution::Implementation,
            hash: SemanticHash([0; 32]),
        };

        let boundary = Boundary {
            inputs: vec![],
            outputs: vec![(root, int_id)],
        };

        let mut fragment = Fragment {
            id: FragmentId([0; 32]),
            graph,
            boundary,
            type_env,
            imports: vec![],
            metadata: FragmentMeta {
                name: None,
                created_at: 0,
                generation: 0,
                lineage_hash: 0,
            },
            proof: None,
            contracts: Default::default(),        };
        fragment.id = compute_fragment_id(&fragment);
        fragment
    }
}
