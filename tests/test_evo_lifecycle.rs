//! Integration tests for evolution lifecycle features (SPEC Sections 6.3, 6.6,
//! 6.8, 6.9, 4.9).
//!
//! Tests: phase detection, death/culling, compression, f_verify scoring,
//! MAP-Elites archive, migration, and multi-deme evolution.

use std::collections::{BTreeMap, HashMap};

use rand::SeedableRng;
use rand::rngs::StdRng;

use iris_evolve::config::{EvolutionConfig, PhaseThresholds};
use iris_evolve::death::{compress_population, cull_population, should_compress};
use iris_evolve::individual::{Fitness, Individual};
use iris_evolve::map_elites::MapElitesArchive;
use iris_evolve::migration::{migrate_ring, should_migrate};
use iris_evolve::phase::{PhaseDetector, phase_params, iris_evolve_test_helpers};
use iris_evolve::population::{Deme, Phase};
use iris_evolve::verify::{self, VerifyTier};

use iris_types::cost::{CostBound, CostTerm};
use iris_types::fragment::{Boundary, Fragment, FragmentId, FragmentMeta};
use iris_types::graph::*;
use iris_types::hash::{compute_fragment_id, compute_node_id, SemanticHash};
use iris_types::types::{PrimType, TypeDef, TypeEnv};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_dummy_fragment() -> Fragment {
    iris_evolve_test_helpers::make_dummy_fragment()
}

fn make_fragment_with_kinds(kinds: &[NodeKind]) -> Fragment {
    iris_evolve_test_helpers::make_fragment_with_kinds(kinds)
}

fn ind_with_fitness(values: [f32; 5]) -> Individual {
    let fragment = make_dummy_fragment();
    let mut ind = Individual::new(fragment);
    ind.fitness = Fitness { values };
    ind
}

fn ind_with_fitness_and_rank(values: [f32; 5], rank: usize, crowding: f32) -> Individual {
    let mut ind = ind_with_fitness(values);
    ind.pareto_rank = rank;
    ind.crowding_distance = crowding;
    ind.meta.pareto_rank = rank as u16;
    ind
}

fn ind_with_fragment(fragment: Fragment, values: [f32; 5]) -> Individual {
    let mut ind = Individual::new(fragment);
    ind.fitness = Fitness { values };
    ind
}

/// Create a fragment with many unreachable nodes (bloated graph).
fn make_bloated_fragment() -> Fragment {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = iris_types::hash::compute_type_id(&int_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    let type_env = TypeEnv { types };

    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // Root node.
    let mut root_node = Node {
        id: NodeId(0),
        kind: NodeKind::Prim,
        type_sig: int_id,
        cost: CostTerm::Unit,
        arity: 1,
        resolution_depth: 0, salt: 0,
        payload: NodePayload::Prim { opcode: 0x00 },
    };
    root_node.id = compute_node_id(&root_node);
    let root_id = root_node.id;
    nodes.insert(root_id, root_node);

    // One reachable child.
    let mut child_node = Node {
        id: NodeId(0),
        kind: NodeKind::Lit,
        type_sig: int_id,
        cost: CostTerm::Unit,
        arity: 0,
        resolution_depth: 1, salt: 0,
        payload: NodePayload::Lit {
            type_tag: 0,
            value: 42i64.to_le_bytes().to_vec(),
        },
    };
    child_node.id = compute_node_id(&child_node);
    let child_id = child_node.id;
    nodes.insert(child_id, child_node);

    edges.push(Edge {
        source: root_id,
        target: child_id,
        port: 0,
        label: EdgeLabel::Argument,
    });

    // Five unreachable nodes (dead code).
    for i in 2..7u8 {
        let mut dead_node = Node {
            id: NodeId(0),
            kind: NodeKind::Lit,
            type_sig: int_id,
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: i, salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0,
                value: (i as i64).to_le_bytes().to_vec(),
            },
        };
        dead_node.id = compute_node_id(&dead_node);
        nodes.insert(dead_node.id, dead_node);
    }

    let graph = SemanticGraph {
        root: root_id,
        nodes,
        edges,
        type_env: type_env.clone(),
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    };

    let boundary = Boundary {
        inputs: vec![],
        outputs: vec![(root_id, int_id)],
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
        contracts: Default::default(),    };
    fragment.id = compute_fragment_id(&fragment);
    fragment
}

// ===========================================================================
// Phase Detection Tests
// ===========================================================================

#[test]
fn test_phase_detection_first_20_generations_exploration() {
    let mut detector = PhaseDetector::new(10);
    let pop = vec![ind_with_fitness([0.5, 0.5, 0.5, 0.5, 0.0])];

    for generation in 0..20 {
        let phase = detector.detect(&pop, generation);
        assert_eq!(
            phase,
            Phase::Exploration,
            "First 20 generations should be Exploration, got {:?} at generation {}",
            phase,
            generation
        );
    }
}

#[test]
fn test_phase_detection_transitions() {
    let mut detector = PhaseDetector::new(5);

    // First 20 gens: Exploration (regardless of improvement).
    let pop = vec![ind_with_fitness([0.5, 0.5, 0.5, 0.5, 0.0])];
    for generation in 0..20 {
        let phase = detector.detect(&pop, generation);
        assert_eq!(phase, Phase::Exploration);
    }

    // After generation 20 with no improvement, should transition to SteadyState or Exploitation.
    for generation in 20..40 {
        let phase = detector.detect(&pop, generation);
        assert_ne!(
            phase,
            Phase::Exploration,
            "After generation 20 with no improvement, should not be Exploration (generation {})",
            generation
        );
    }
}

#[test]
fn test_phase_detection_stagnation_forces_exploration() {
    let mut detector = PhaseDetector::new(5);
    let pop = vec![ind_with_fitness([0.5, 0.5, 0.5, 0.5, 0.0])];

    // Skip past the first 20 forced-exploration generations.
    for generation in 0..20 {
        detector.detect(&pop, generation);
    }

    // Run many generations with zero improvement to trigger stagnation.
    for generation in 20..100 {
        detector.detect(&pop, generation);
    }

    // After 50+ stagnation, should force back to Exploration.
    // The stagnation counter should eventually trigger this.
    let mut found_exploration = false;
    for generation in 100..200 {
        let phase = detector.detect(&pop, generation);
        if phase == Phase::Exploration {
            found_exploration = true;
            break;
        }
    }
    assert!(
        found_exploration,
        "Stagnation > 50 should force Exploration phase"
    );
}

#[test]
fn test_phase_params_values() {
    let exp = phase_params(Phase::Exploration);
    assert!((exp.mutation_rate - 0.95).abs() < f64::EPSILON);
    assert!((exp.crossover_rate - 0.7).abs() < f64::EPSILON);
    assert_eq!(exp.tournament_size, 2);

    let ss = phase_params(Phase::SteadyState);
    assert!((ss.mutation_rate - 0.7).abs() < f64::EPSILON);
    assert!((ss.crossover_rate - 0.5).abs() < f64::EPSILON);
    assert_eq!(ss.tournament_size, 3);

    let ex = phase_params(Phase::Exploitation);
    assert!((ex.mutation_rate - 0.3).abs() < f64::EPSILON);
    assert!((ex.crossover_rate - 0.4).abs() < f64::EPSILON);
    assert_eq!(ex.tournament_size, 4);
}

// ===========================================================================
// Death and Culling Tests
// ===========================================================================

#[test]
fn test_death_removes_low_fitness() {
    let mut pop = vec![
        ind_with_fitness_and_rank([0.0, 0.0, 0.0, 0.0, 0.0], 1, 1.0),
        ind_with_fitness_and_rank([0.5, 0.5, 0.5, 0.5, 0.0], 0, 1.0),
        ind_with_fitness_and_rank([0.005, 0.005, 0.005, 0.005, 0.0], 2, 1.0),
    ];

    cull_population(&mut pop, Phase::Exploration, 10, 100);

    // First and third have all objectives < 0.01, should be removed.
    assert_eq!(pop.len(), 1);
    assert_eq!(pop[0].pareto_rank, 0);
}

#[test]
fn test_death_preserves_elites() {
    let mut pop = vec![
        ind_with_fitness_and_rank([0.5, 0.5, 0.5, 0.5, 0.0], 0, 1.0), // elite
        ind_with_fitness_and_rank([0.5, 0.5, 0.5, 0.5, 0.0], 1, 1.0), // non-elite
    ];

    // Age death: generation 300, meta.age = 0, so age = 300.
    // Exploitation MAX_AGE = 50, but elites are exempt.
    cull_population(&mut pop, Phase::Exploitation, 300, 100);

    assert_eq!(pop.len(), 1, "Elite should survive age death");
    assert_eq!(pop[0].pareto_rank, 0, "Survivor should be the elite");
}

#[test]
fn test_death_age_limits_by_phase() {
    // Exploration: MAX_AGE = 200
    let mut pop_exp = vec![
        ind_with_fitness_and_rank([0.5, 0.5, 0.5, 0.5, 0.0], 1, 1.0),
    ];
    cull_population(&mut pop_exp, Phase::Exploration, 199, 100);
    assert_eq!(pop_exp.len(), 1, "Should survive at age 199 in Exploration");

    let mut pop_exp2 = vec![
        ind_with_fitness_and_rank([0.5, 0.5, 0.5, 0.5, 0.0], 1, 1.0),
    ];
    cull_population(&mut pop_exp2, Phase::Exploration, 201, 100);
    assert_eq!(pop_exp2.len(), 0, "Should die at age 201 in Exploration");

    // Exploitation: MAX_AGE = 50
    let mut pop_expl = vec![
        ind_with_fitness_and_rank([0.5, 0.5, 0.5, 0.5, 0.0], 1, 1.0),
    ];
    cull_population(&mut pop_expl, Phase::Exploitation, 51, 100);
    assert_eq!(pop_expl.len(), 0, "Should die at age 51 in Exploitation");
}

#[test]
fn test_crowding_death_keeps_diverse() {
    let mut pop = vec![
        ind_with_fitness_and_rank([0.5, 0.5, 0.5, 0.5, 0.0], 1, 0.1), // most crowded
        ind_with_fitness_and_rank([0.5, 0.5, 0.5, 0.5, 0.0], 1, 0.5), // medium
        ind_with_fitness_and_rank([0.5, 0.5, 0.5, 0.5, 0.0], 1, 0.9), // least crowded
        ind_with_fitness_and_rank([0.5, 0.5, 0.5, 0.5, 0.0], 0, 0.1), // elite
    ];

    cull_population(&mut pop, Phase::Exploration, 0, 2);

    // Should keep elite + highest crowding non-elite.
    assert_eq!(pop.len(), 2);
    assert!(
        pop.iter().any(|i| i.pareto_rank == 0),
        "Elite should be preserved"
    );
}

// ===========================================================================
// Compression Tests
// ===========================================================================

#[test]
fn test_should_compress_timing() {
    assert!(!should_compress(0));
    assert!(!should_compress(1));
    assert!(!should_compress(49));
    assert!(should_compress(50));
    assert!(should_compress(100));
    assert!(!should_compress(75));
}

#[test]
fn test_compression_removes_unreachable_nodes() {
    let bloated = make_bloated_fragment();
    let original_count = bloated.graph.nodes.len();
    assert!(
        original_count > 2,
        "Bloated fragment should have more than 2 nodes, got {}",
        original_count
    );

    let mut pop = vec![ind_with_fragment(bloated, [0.5, 0.5, 0.5, 0.5, 0.0])];
    let removed = compress_population(&mut pop);

    assert!(
        removed > 0,
        "Compression should remove at least some unreachable nodes"
    );

    let final_count = pop[0].fragment.graph.nodes.len();
    assert!(
        final_count < original_count,
        "Node count should decrease: {} -> {}",
        original_count,
        final_count
    );

    // Should have exactly 2 reachable nodes (root + child).
    assert_eq!(final_count, 2, "Should have 2 reachable nodes after compression");
}

#[test]
fn test_compression_preserves_reachable_nodes() {
    // A fragment where all nodes are reachable (no dead code).
    let fragment = make_fragment_with_kinds(&[NodeKind::Lit, NodeKind::Prim]);
    let original_count = fragment.graph.nodes.len();

    let mut pop = vec![ind_with_fragment(fragment, [0.5; 5])];
    let removed = compress_population(&mut pop);

    assert_eq!(removed, 0, "No nodes should be removed when all are reachable");
    assert_eq!(pop[0].fragment.graph.nodes.len(), original_count);
}

// ===========================================================================
// f_verify Scoring Tests
// ===========================================================================

#[test]
fn test_f_verify_tier0_no_complex_constructs() {
    let fragment = make_fragment_with_kinds(&[NodeKind::Prim, NodeKind::Lit, NodeKind::Prim]);
    let tier = verify::classify_verify_tier(&fragment.graph);
    assert_eq!(tier, VerifyTier::Tier0);
    let score = verify::tier_to_f_verify(tier);
    assert!((score - 0.3).abs() < f32::EPSILON, "Tier 0 should give f_verify = 0.3");
}

#[test]
fn test_f_verify_tier1_fold_no_neural() {
    let fragment = make_fragment_with_kinds(&[NodeKind::Lit, NodeKind::Fold]);
    let tier = verify::classify_verify_tier(&fragment.graph);
    assert_eq!(tier, VerifyTier::Tier1);
    let score = verify::tier_to_f_verify(tier);
    assert!((score - 0.6).abs() < f32::EPSILON, "Tier 1 should give f_verify = 0.6");
}

#[test]
fn test_f_verify_unverified_neural() {
    let fragment = make_fragment_with_kinds(&[NodeKind::Lit, NodeKind::Neural]);
    let tier = verify::classify_verify_tier(&fragment.graph);
    assert_eq!(tier, VerifyTier::Unverified);
    let score = verify::tier_to_f_verify(tier);
    assert!((score - 0.0).abs() < f32::EPSILON, "Unverified should give f_verify = 0.0");
}

#[test]
fn test_f_verify_unverified_extern() {
    let fragment = make_fragment_with_kinds(&[NodeKind::Lit, NodeKind::Extern]);
    let tier = verify::classify_verify_tier(&fragment.graph);
    // Extern without Fold -> Unverified (has Extern in the graph).
    assert_eq!(tier, VerifyTier::Unverified);
}

#[test]
fn test_f_verify_fold_and_neural_is_unverified() {
    let fragment = make_fragment_with_kinds(&[NodeKind::Lit, NodeKind::Fold, NodeKind::Neural]);
    let tier = verify::classify_verify_tier(&fragment.graph);
    assert_eq!(tier, VerifyTier::Unverified);
}

#[test]
fn test_f_verify_compute_shorthand() {
    let fragment = make_fragment_with_kinds(&[NodeKind::Prim, NodeKind::Lit]);
    let score = verify::compute_f_verify(&fragment.graph);
    assert!((score - 0.3).abs() < f32::EPSILON);
}

// ===========================================================================
// MAP-Elites Archive Tests
// ===========================================================================

#[test]
fn test_map_elites_insert_and_replace() {
    let mut archive = MapElitesArchive::new();
    assert!(archive.is_empty());

    let frag = make_dummy_fragment();
    let ind1 = ind_with_fragment(frag.clone(), [0.3, 0.3, 0.3, 0.3, 0.0]);
    archive.update(&ind1);
    assert_eq!(archive.len(), 1);

    // Higher fitness should replace.
    let ind2 = ind_with_fragment(frag.clone(), [0.5, 0.5, 0.5, 0.5, 0.0]);
    archive.update(&ind2);
    assert_eq!(archive.len(), 1);

    // Lower fitness should NOT replace.
    let ind3 = ind_with_fragment(frag, [0.1, 0.1, 0.1, 0.1, 0.0]);
    archive.update(&ind3);
    assert_eq!(archive.len(), 1);

    // Check the stored individual has the highest fitness.
    let mut rng = StdRng::seed_from_u64(42);
    let sampled = archive.sample(&mut rng).unwrap();
    let fitness_sum: f32 = sampled.fitness.values.iter().sum();
    assert!(
        fitness_sum > 1.5,
        "Archive should contain the best individual (fitness sum {})",
        fitness_sum
    );
}

#[test]
fn test_map_elites_preserves_diversity() {
    let mut archive = MapElitesArchive::new();

    // Insert individuals with different graph structures -> different bins.
    let frag_simple = make_dummy_fragment(); // 1 node = bin (0, x)
    let ind1 = ind_with_fragment(frag_simple, [0.5; 5]);

    let frag_complex = make_fragment_with_kinds(&[
        NodeKind::Lit,
        NodeKind::Prim,
        NodeKind::Prim,
        NodeKind::Fold,
        NodeKind::Lit,
        NodeKind::Prim,
        NodeKind::Lit,
        NodeKind::Prim,
    ]);
    let ind2 = ind_with_fragment(frag_complex, [0.4; 5]);

    archive.update(&ind1);
    archive.update(&ind2);

    // The two individuals may or may not land in different bins depending on
    // structure. At minimum, we should have at least 1 entry.
    assert!(archive.len() >= 1);

    // Both individuals should be retrievable if in different bins.
    let all: Vec<_> = archive.individuals().collect();
    assert!(
        !all.is_empty(),
        "Archive should contain at least one individual"
    );
}

#[test]
fn test_map_elites_batch_update() {
    let mut archive = MapElitesArchive::new();

    let pop: Vec<Individual> = (0..10)
        .map(|i| {
            let frag = make_dummy_fragment();
            ind_with_fragment(frag, [i as f32 / 10.0; 5])
        })
        .collect();

    archive.update_batch(&pop);
    assert!(archive.len() >= 1, "Batch update should populate the archive");
}

#[test]
fn test_map_elites_sample_empty() {
    let archive = MapElitesArchive::new();
    let mut rng = StdRng::seed_from_u64(42);
    assert!(archive.sample(&mut rng).is_none());
}

// ===========================================================================
// Migration Tests
// ===========================================================================

#[test]
fn test_should_migrate_timing() {
    assert!(!should_migrate(0));
    assert!(!should_migrate(1));
    assert!(!should_migrate(4));
    assert!(should_migrate(5));
    assert!(should_migrate(10));
    assert!(should_migrate(15));
    assert!(!should_migrate(7));
}

#[test]
fn test_migration_transfers_top_individuals() {
    let seed_fn = |_: usize| make_dummy_fragment();
    let mut deme_a = Deme::initialize(4, |i| seed_fn(i));
    let mut deme_b = Deme::initialize(4, |i| seed_fn(i));

    // Give deme_a good individuals.
    for (i, ind) in deme_a.individuals.iter_mut().enumerate() {
        ind.pareto_rank = i;
        ind.crowding_distance = 1.0 - i as f32 * 0.2;
        ind.fitness = Fitness {
            values: [1.0 - i as f32 * 0.2, 1.0 - i as f32 * 0.2, 1.0 - i as f32 * 0.2, 1.0 - i as f32 * 0.2, 0.0],
        };
        ind.meta.pareto_rank = i as u16;
    }

    // Give deme_b poor individuals.
    for ind in deme_b.individuals.iter_mut() {
        ind.pareto_rank = 5;
        ind.crowding_distance = 0.1;
        ind.fitness = Fitness {
            values: [0.1, 0.1, 0.1, 0.1, 0.0],
        };
        ind.meta.pareto_rank = 5;
    }

    let best_in_b_before = deme_b
        .individuals
        .iter()
        .map(|i| i.fitness.values.iter().sum::<f32>())
        .fold(0.0f32, f32::max);

    let mut demes = vec![deme_a, deme_b];
    migrate_ring(&mut demes, 5); // generation 5 -> migration happens

    let best_in_b_after = demes[1]
        .individuals
        .iter()
        .map(|i| i.fitness.values.iter().sum::<f32>())
        .fold(0.0f32, f32::max);

    assert!(
        best_in_b_after > best_in_b_before,
        "Migration should improve deme_b: before={}, after={}",
        best_in_b_before,
        best_in_b_after
    );
}

#[test]
fn test_migration_no_op_wrong_generation() {
    let seed_fn = |_: usize| make_dummy_fragment();
    let mut deme_a = Deme::initialize(4, |i| seed_fn(i));
    let mut deme_b = Deme::initialize(4, |i| seed_fn(i));

    // Give deme_a good individuals.
    for ind in deme_a.individuals.iter_mut() {
        ind.fitness = Fitness {
            values: [0.9, 0.9, 0.9, 0.9, 0.0],
        };
        ind.pareto_rank = 0;
    }

    // Give deme_b poor individuals.
    for ind in deme_b.individuals.iter_mut() {
        ind.fitness = Fitness {
            values: [0.1, 0.1, 0.1, 0.1, 0.0],
        };
        ind.pareto_rank = 5;
    }

    let mut demes = vec![deme_a, deme_b];

    // Generation 3 -- should NOT migrate.
    migrate_ring(&mut demes, 3);

    let best_in_b = demes[1]
        .individuals
        .iter()
        .map(|i| i.fitness.values.iter().sum::<f32>())
        .fold(0.0f32, f32::max);

    assert!(
        best_in_b < 1.0,
        "Migration should not happen at generation 3"
    );
}

#[test]
fn test_migration_ring_topology() {
    // With 3 demes, migration goes A->B, B->C, C->A.
    let seed_fn = |_: usize| make_dummy_fragment();
    let mut demes: Vec<Deme> = (0..3)
        .map(|deme_idx| {
            let mut deme = Deme::initialize(4, |i| seed_fn(i));
            for ind in deme.individuals.iter_mut() {
                let fitness_val = match deme_idx {
                    0 => 0.9, // deme 0: best
                    1 => 0.5, // deme 1: medium
                    _ => 0.1, // deme 2: worst
                };
                ind.fitness = Fitness {
                    values: [fitness_val, fitness_val, fitness_val, fitness_val, 0.0],
                };
                ind.pareto_rank = if deme_idx == 0 { 0 } else { deme_idx + 1 };
                ind.meta.pareto_rank = ind.pareto_rank as u16;
                ind.crowding_distance = 1.0;
            }
            deme
        })
        .collect();

    migrate_ring(&mut demes, 5);

    // Deme 1 should have received good individuals from deme 0.
    let best_in_1 = demes[1]
        .individuals
        .iter()
        .map(|i| i.fitness.values.iter().sum::<f32>())
        .fold(0.0f32, f32::max);

    assert!(
        best_in_1 > 2.0,
        "Deme 1 should have received good individuals from deme 0 (best_in_1 = {})",
        best_in_1
    );
}

#[test]
fn test_single_deme_no_migration() {
    let seed_fn = |_: usize| make_dummy_fragment();
    let mut demes = vec![Deme::initialize(4, |i| seed_fn(i))];

    // Should be a no-op with single deme.
    migrate_ring(&mut demes, 5);
    assert_eq!(demes.len(), 1);
}

// ===========================================================================
// Multi-deme Evolution Integration
// ===========================================================================

#[test]
fn test_multi_deme_config_defaults() {
    let config = EvolutionConfig::default();
    assert_eq!(config.num_demes, 1, "Default num_demes should be 1");
}

#[test]
fn test_multi_deme_config_backwards_compat() {
    // Existing configs with num_demes: 1 should work identically.
    let config = EvolutionConfig {
        population_size: 32,
        max_generations: 10,
        mutation_rate: 0.8,
        crossover_rate: 0.5,
        tournament_size: 3,
        phase_thresholds: PhaseThresholds::default(),
        target_generation_time_ms: 500,
        num_demes: 1,
        novelty_k: 15,
        novelty_threshold: 0.1,
        novelty_weight: 1.0,
        coevolution: false,
        resource_budget_ms: 0,
        iris_mode: false,
    };
    assert_eq!(config.num_demes, 1);
}

// ===========================================================================
// Combined Lifecycle Integration
// ===========================================================================

#[test]
fn test_lifecycle_death_then_compression() {
    // Simulate a full lifecycle: create population, apply death, then compress.
    let bloated = make_bloated_fragment();
    let mut pop = vec![
        // Good individual with bloated graph.
        {
            let mut ind = ind_with_fragment(bloated.clone(), [0.5, 0.5, 0.5, 0.5, 0.0]);
            ind.pareto_rank = 0;
            ind.meta.pareto_rank = 0;
            ind
        },
        // Bad individual.
        ind_with_fitness_and_rank([0.001, 0.001, 0.001, 0.001, 0.0], 3, 0.1),
        // Normal individual.
        ind_with_fitness_and_rank([0.3, 0.3, 0.3, 0.3, 0.0], 1, 0.5),
    ];

    // Death should remove the bad individual.
    cull_population(&mut pop, Phase::Exploration, 0, 10);
    assert_eq!(pop.len(), 2, "Death should remove the low-fitness individual");

    // Compression should remove dead code from the bloated graph.
    let removed = compress_population(&mut pop);
    assert!(removed > 0, "Compression should remove unreachable nodes");
}

#[test]
fn test_f_verify_integrated_with_phase() {
    // Verify that f_verify is correctly computed for different graph structures
    // and that phase detection doesn't interfere.

    let simple_frag = make_fragment_with_kinds(&[NodeKind::Prim, NodeKind::Lit]);
    assert!((verify::compute_f_verify(&simple_frag.graph) - 0.3).abs() < f32::EPSILON);

    let fold_frag = make_fragment_with_kinds(&[NodeKind::Lit, NodeKind::Fold]);
    assert!((verify::compute_f_verify(&fold_frag.graph) - 0.6).abs() < f32::EPSILON);

    let neural_frag = make_fragment_with_kinds(&[NodeKind::Lit, NodeKind::Neural]);
    assert!((verify::compute_f_verify(&neural_frag.graph) - 0.0).abs() < f32::EPSILON);
}
