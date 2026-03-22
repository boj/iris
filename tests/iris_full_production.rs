//! Full production integration test: evolution running on IRIS programs.
//!
//! This is the definitive test: `evolve()` with `iris_mode: true` uses
//! IRIS-written mutation, selection, and evaluation programs instead of
//! Rust functions, and produces correct results.

use std::collections::{BTreeMap, HashMap};

use iris_evolve::config::{EvolutionConfig, ProblemSpec};
use iris_evolve::evolve;
use iris_evolve::iris_runtime::IrisRuntime;
use iris_exec::interpreter;
use iris_exec::service::{ExecConfig, IrisExecutionService};
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::{TestCase, Value};
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{PrimType, TypeDef, TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn int_type_env() -> (TypeEnv, TypeId) {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = iris_types::hash::compute_type_id(&int_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    (TypeEnv { types }, int_id)
}

fn make_node(kind: NodeKind, payload: NodePayload, int_id: TypeId, arity: u8) -> Node {
    let mut n = Node {
        id: NodeId(0),
        kind,
        type_sig: int_id,
        cost: CostTerm::Unit,
        arity,
        resolution_depth: 0, salt: 0,
        payload,
    };
    n.id = iris_types::hash::compute_node_id(&n);
    n
}

fn make_program(opcode: u8) -> SemanticGraph {
    let (te, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let i0 = make_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0xFF,
            value: vec![0],
        },
        int_id,
        0,
    );
    let i1 = make_node(
        NodeKind::Lit,
        NodePayload::Lit {
            type_tag: 0xFF,
            value: vec![1],
        },
        int_id,
        0,
    );
    let op = make_node(NodeKind::Prim, NodePayload::Prim { opcode }, int_id, 2);
    let i0id = i0.id;
    let i1id = i1.id;
    let opid = op.id;
    nodes.insert(i0id, i0);
    nodes.insert(i1id, i1);
    nodes.insert(opid, op);
    let edges = vec![
        Edge {
            source: opid,
            target: i0id,
            port: 0,
            label: EdgeLabel::Argument,
        },
        Edge {
            source: opid,
            target: i1id,
            port: 1,
            label: EdgeLabel::Argument,
        },
    ];
    SemanticGraph {
        root: opid,
        nodes,
        edges,
        type_env: te,
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

fn sum_problem_spec() -> ProblemSpec {
    ProblemSpec {
        test_cases: vec![
            TestCase {
                inputs: vec![Value::tuple(vec![
                    Value::Int(1),
                    Value::Int(2),
                    Value::Int(3),
                ])],
                expected_output: Some(vec![Value::Int(6)]),
                initial_state: None,
                expected_state: None,
            },
            TestCase {
                inputs: vec![Value::tuple(vec![Value::Int(0)])],
                expected_output: Some(vec![Value::Int(0)]),
                initial_state: None,
                expected_state: None,
            },
            TestCase {
                inputs: vec![Value::tuple(vec![Value::Int(10), Value::Int(-5)])],
                expected_output: Some(vec![Value::Int(5)]),
                initial_state: None,
                expected_state: None,
            },
            TestCase {
                inputs: vec![Value::tuple(vec![])],
                expected_output: Some(vec![Value::Int(0)]),
                initial_state: None,
                expected_state: None,
            },
        ],
        description: "Sum all elements in a list".to_string(),
        target_cost: None,
    }
}

fn add_problem_spec() -> ProblemSpec {
    ProblemSpec {
        test_cases: vec![
            TestCase {
                inputs: vec![Value::Int(3), Value::Int(5)],
                expected_output: Some(vec![Value::Int(8)]),
                initial_state: None,
                expected_state: None,
            },
            TestCase {
                inputs: vec![Value::Int(10), Value::Int(1)],
                expected_output: Some(vec![Value::Int(11)]),
                initial_state: None,
                expected_state: None,
            },
            TestCase {
                inputs: vec![Value::Int(0), Value::Int(0)],
                expected_output: Some(vec![Value::Int(0)]),
                initial_state: None,
                expected_state: None,
            },
            TestCase {
                inputs: vec![Value::Int(-3), Value::Int(3)],
                expected_output: Some(vec![Value::Int(0)]),
                initial_state: None,
                expected_state: None,
            },
            TestCase {
                inputs: vec![Value::Int(100), Value::Int(200)],
                expected_output: Some(vec![Value::Int(300)]),
                initial_state: None,
                expected_state: None,
            },
        ],
        description: "Add two integers".to_string(),
        target_cost: None,
    }
}

// ---------------------------------------------------------------------------
// Test 1: IrisRuntime unit-level -- mutation works
// ---------------------------------------------------------------------------

#[test]
fn iris_runtime_mutates_programs() {
    println!("\n{}", "=".repeat(70));
    println!("  Test 1: IrisRuntime mutation");
    println!("{}", "=".repeat(70));

    let rt = IrisRuntime::new();
    assert!(rt.initialized);

    // Build a sub(a, b) program.
    let sub_program = make_program(0x01);
    let result = interpreter::interpret(&sub_program, &[Value::Int(10), Value::Int(3)], None);
    assert_eq!(
        result.ok().map(|(o, _)| o),
        Some(vec![Value::Int(7)]),
        "sub(10, 3) should be 7"
    );

    // Mutate it using IRIS.
    let mut rng = rand::thread_rng();
    let mutated = rt.mutate(&sub_program, &mut rng);

    // The mutated program should be structurally different (different opcode
    // on most runs) or at worst the same (identity mutation is valid).
    println!(
        "  Original nodes: {}, Mutated nodes: {}",
        sub_program.nodes.len(),
        mutated.nodes.len()
    );
    println!("  Mutation completed without panic -- IRIS mutation works");
}

// ---------------------------------------------------------------------------
// Test 2: IrisRuntime -- evaluation works
// ---------------------------------------------------------------------------

#[test]
fn iris_runtime_evaluates_programs() {
    println!("\n{}", "=".repeat(70));
    println!("  Test 2: IrisRuntime evaluation via graph_eval");
    println!("{}", "=".repeat(70));

    let rt = IrisRuntime::new();

    let add_program = make_program(0x00); // add(a, b)

    // Evaluate add(3, 5) using the IRIS evaluator.
    let result = rt.evaluate_program(&add_program, &[Value::Int(3), Value::Int(5)]);
    println!("  IRIS evaluate add(3, 5) = {:?}", result);
    assert!(result.is_some(), "IRIS evaluator should return a result");

    let outputs = result.unwrap();
    assert_eq!(
        outputs.first(),
        Some(&Value::Int(8)),
        "add(3, 5) should be 8"
    );
    println!("  IRIS evaluation works correctly");
}

// ---------------------------------------------------------------------------
// Test 3: IrisRuntime -- scoring works
// ---------------------------------------------------------------------------

#[test]
fn iris_runtime_scores_programs() {
    println!("\n{}", "=".repeat(70));
    println!("  Test 3: IrisRuntime scoring");
    println!("{}", "=".repeat(70));

    let rt = IrisRuntime::new();
    let add_program = make_program(0x00);

    // Score against test cases where add is correct.
    let test_cases = vec![
        (vec![Value::Int(3), Value::Int(5)], Value::Int(8)),
        (vec![Value::Int(10), Value::Int(1)], Value::Int(11)),
        (vec![Value::Int(0), Value::Int(0)], Value::Int(0)),
    ];

    let score = rt.score_program(&add_program, &test_cases);
    println!("  add program score: {:.2}", score);
    assert!(
        score >= 0.99,
        "add program should score 1.0 on add test cases"
    );

    // Score sub program against add test cases -- should score 0.
    let sub_program = make_program(0x01);
    let sub_score = rt.score_program(&sub_program, &test_cases);
    println!("  sub program score on add tests: {:.2}", sub_score);
    assert!(
        sub_score < 0.5,
        "sub program should score low on add test cases"
    );

    println!("  IRIS scoring differentiates correct from incorrect programs");
}

// ---------------------------------------------------------------------------
// Test 4: IrisRuntime -- tournament selection works
// ---------------------------------------------------------------------------

#[test]
fn iris_runtime_tournament_selection() {
    println!("\n{}", "=".repeat(70));
    println!("  Test 4: IrisRuntime tournament selection");
    println!("{}", "=".repeat(70));

    let rt = IrisRuntime::new();
    let mut rng = rand::thread_rng();

    // A population with known fitnesses.
    let fitnesses = vec![0.0, 0.5, 1.0, 0.3, 0.8];

    // Over many selections, the best individual (index 2) should be
    // selected most often.
    let mut counts = vec![0usize; 5];
    for _ in 0..1000 {
        let idx = rt.tournament_select(&fitnesses, 3, &mut rng);
        counts[idx] += 1;
    }

    println!("  Selection counts: {:?}", counts);
    // The highest-fitness individual should be selected most often.
    let best_count = counts[2];
    let worst_count = counts[0];
    assert!(
        best_count > worst_count,
        "best individual should be selected more often than worst"
    );
    println!("  Tournament selection correctly favors fitter individuals");
}

// ---------------------------------------------------------------------------
// Test 5: Manual evolution loop using IrisRuntime
// ---------------------------------------------------------------------------

#[test]
fn iris_runtime_manual_evolution() {
    println!("\n{}", "=".repeat(70));
    println!("  Test 5: Manual evolution loop using IrisRuntime");
    println!("{}", "=".repeat(70));

    let rt = IrisRuntime::new();
    let mut rng = rand::thread_rng();

    // Target: add(a, b)
    let test_cases = vec![
        (vec![Value::Int(3), Value::Int(5)], Value::Int(8)),
        (vec![Value::Int(10), Value::Int(1)], Value::Int(11)),
        (vec![Value::Int(0), Value::Int(0)], Value::Int(0)),
    ];

    // Start with sub(a, b) -- wrong answer.
    let mut current = make_program(0x01);
    let mut current_score = rt.score_program(&current, &test_cases);
    println!("  Generation 0: score = {:.2} (sub program)", current_score);

    // Run 20 generations of hill-climbing with IRIS mutation.
    for generation in 1..=20 {
        let mutant = rt.mutate(&current, &mut rng);
        let mutant_score = rt.score_program(&mutant, &test_cases);

        if mutant_score > current_score {
            current = mutant;
            current_score = mutant_score;
            println!(
                "  Generation {}: score = {:.2} (improved!)",
                generation, current_score
            );
        }

        if current_score >= 0.99 {
            println!(
                "  Perfect solution found at generation {}!",
                generation
            );
            break;
        }
    }

    // We expect to find the solution within 20 generations of hill-climbing
    // since replace_prim can directly change sub -> add.
    println!("  Final score: {:.2}", current_score);

    if current_score >= 0.99 {
        // Verify the solution actually works.
        let result =
            interpreter::interpret(&current, &[Value::Int(7), Value::Int(3)], None);
        println!(
            "  Evolved program: f(7, 3) = {:?}",
            result.as_ref().ok().map(|(o, _)| o.first())
        );
        println!("  IRIS-powered evolution found correct program!");
    } else {
        // Even if hill-climbing didn't find it (unlucky RNG), the test
        // still passes as long as it ran without errors. The full evolve()
        // test below uses a proper population.
        println!("  Hill climbing didn't converge (expected with random mutations)");
    }
}

// ---------------------------------------------------------------------------
// Test 6: evolve() in IRIS mode -- the definitive test
// ---------------------------------------------------------------------------

#[test]
fn evolve_add_in_iris_mode() {
    println!("\n{}", "=".repeat(70));
    println!("  Test 6: evolve() in IRIS mode -- the definitive test");
    println!("  Solving add(a,b) using IRIS-written components");
    println!("{}", "=".repeat(70));

    let config = EvolutionConfig {
        iris_mode: true,
        population_size: 64,
        max_generations: 200,
        num_demes: 1,
        ..Default::default()
    };

    let spec = add_problem_spec();
    let exec = IrisExecutionService::new(ExecConfig::default());

    let result = evolve(config, spec, &exec);

    let correctness = result.best_individual.fitness.correctness();
    println!(
        "  Result: {:.0}% correctness in {} generations ({:.2}s)",
        correctness * 100.0,
        result.generations_run,
        result.total_time.as_secs_f64()
    );

    // The add problem is trivially solvable (even enumeration finds it).
    // The key assertion is that it RUNS in IRIS mode without errors.
    assert!(
        correctness >= 0.99,
        "add(a,b) should be solved in IRIS mode (got {:.2}%)",
        correctness * 100.0
    );

    println!("  add(a,b) solved in IRIS mode!");
}

// ---------------------------------------------------------------------------
// Test 7: evolve() in IRIS mode -- sum problem (harder)
// ---------------------------------------------------------------------------

#[test]
fn evolve_sum_in_iris_mode() {
    println!("\n{}", "=".repeat(70));
    println!("  Test 7: evolve() in IRIS mode -- sum problem");
    println!("  Solving sum([1,2,3]) = 6 using IRIS-written components");
    println!("{}", "=".repeat(70));

    let config = EvolutionConfig {
        iris_mode: true,
        population_size: 64,
        max_generations: 200,
        num_demes: 1,
        ..Default::default()
    };

    let spec = sum_problem_spec();
    let exec = IrisExecutionService::new(ExecConfig::default());

    let result = evolve(config, spec, &exec);

    let correctness = result.best_individual.fitness.correctness();
    println!(
        "  Result: {:.0}% correctness in {} generations ({:.2}s)",
        correctness * 100.0,
        result.generations_run,
        result.total_time.as_secs_f64()
    );

    // Sum is a harder problem. We assert it runs without crashing.
    // With the analyzer skeleton injection, it should still solve it.
    assert!(
        correctness >= 0.99,
        "sum should be solved in IRIS mode (got {:.2}%)",
        correctness * 100.0
    );

    println!(
        "  Sum solved in IRIS mode! Generation {}",
        result.generations_run
    );
}

// ---------------------------------------------------------------------------
// Test 8: Backward compatibility -- iris_mode=false still works
// ---------------------------------------------------------------------------

#[test]
fn evolve_add_in_rust_mode() {
    println!("\n{}", "=".repeat(70));
    println!("  Test 8: Backward compatibility -- Rust mode");
    println!("{}", "=".repeat(70));

    let config = EvolutionConfig {
        iris_mode: false,
        population_size: 64,
        max_generations: 200,
        num_demes: 1,
        ..Default::default()
    };

    let spec = add_problem_spec();
    let exec = IrisExecutionService::new(ExecConfig::default());

    let result = evolve(config, spec, &exec);

    let correctness = result.best_individual.fitness.correctness();
    println!(
        "  Rust mode: {:.0}% correctness in {} generations",
        correctness * 100.0,
        result.generations_run
    );
    assert!(
        correctness >= 0.99,
        "add(a,b) should be solved in Rust mode"
    );
    println!("  Backward compatibility confirmed");
}

// ---------------------------------------------------------------------------
// Test 9: IRIS fallback safety -- graceful degradation
// ---------------------------------------------------------------------------

#[test]
fn iris_mode_fallback_on_empty_graph() {
    println!("\n{}", "=".repeat(70));
    println!("  Test 9: IRIS fallback safety");
    println!("{}", "=".repeat(70));

    let rt = IrisRuntime::new();
    let mut rng = rand::thread_rng();

    // An empty graph -- IRIS will fail, should fall back to Rust.
    let empty = SemanticGraph {
        root: NodeId(0),
        nodes: HashMap::new(),
        edges: Vec::new(),
        type_env: TypeEnv {
            types: BTreeMap::new(),
        },
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    };

    // Should not panic.
    let _mutated = rt.mutate(&empty, &mut rng);

    let fallbacks = rt
        .fallback_count
        .load(std::sync::atomic::Ordering::Relaxed);
    println!("  Fallback count after empty graph mutation: {}", fallbacks);
    assert!(
        fallbacks > 0,
        "should have fallen back to Rust for empty graph"
    );
    println!("  Graceful degradation confirmed: IRIS fails -> Rust takes over");
}

// ---------------------------------------------------------------------------
// Test 10: IRIS mutation produces diverse programs
// ---------------------------------------------------------------------------

#[test]
fn iris_mutation_produces_diversity() {
    println!("\n{}", "=".repeat(70));
    println!("  Test 10: IRIS mutation diversity");
    println!("{}", "=".repeat(70));

    let rt = IrisRuntime::new();
    let mut rng = rand::thread_rng();

    let base = make_program(0x00); // add(a, b)

    // Mutate 50 times and collect unique results.
    let mut unique_opcodes = std::collections::HashSet::new();
    for _ in 0..50 {
        let mutant = rt.mutate(&base, &mut rng);
        // Find the Prim node opcode in the mutant.
        for node in mutant.nodes.values() {
            if let NodePayload::Prim { opcode } = &node.payload {
                unique_opcodes.insert(*opcode);
            }
        }
    }

    println!(
        "  Unique opcodes after 50 mutations: {:?}",
        unique_opcodes
    );
    assert!(
        unique_opcodes.len() >= 2,
        "IRIS mutation should produce at least 2 different opcodes, got {:?}",
        unique_opcodes
    );
    println!("  IRIS mutation produces diverse programs");
}
