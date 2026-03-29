
//! Integration test: run actual evolution using IRIS-written components
//! instead of Rust. Prove they work end-to-end, not just in isolation.

use iris_types::eval::*;
use iris_types::graph::*;
use iris_types::cost::*;
use iris_types::types::*;
use iris_types::hash::*;
use iris_types::fragment::*;
use iris_exec::service::*;
use iris_exec::interpreter;
use iris_evolve::config::*;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

fn int_type_env() -> (TypeEnv, TypeId) {
    let int_def = TypeDef::Primitive(PrimType::Int);
    let int_id = compute_type_id(&int_def);
    let mut types = BTreeMap::new();
    types.insert(int_id, int_def);
    (TypeEnv { types }, int_id)
}

fn make_node(kind: NodeKind, payload: NodePayload, int_id: TypeId, arity: u8) -> Node {
    let mut n = Node {
        id: NodeId(0), kind, type_sig: int_id,
        cost: CostTerm::Unit, arity, resolution_depth: 0, salt: 0, payload,
    };
    n.id = compute_node_id(&n);
    n
}

fn make_program(opcode: u8) -> SemanticGraph {
    let (te, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let i0 = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0xFF, value: vec![0] }, int_id, 0);
    let i1 = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0xFF, value: vec![1] }, int_id, 0);
    let op = make_node(NodeKind::Prim, NodePayload::Prim { opcode }, int_id, 2);
    let i0id = i0.id; let i1id = i1.id; let opid = op.id;
    nodes.insert(i0id, i0);
    nodes.insert(i1id, i1);
    nodes.insert(opid, op);
    let edges = vec![
        Edge { source: opid, target: i0id, port: 0, label: EdgeLabel::Argument },
        Edge { source: opid, target: i1id, port: 1, label: EdgeLabel::Argument },
    ];
    SemanticGraph {
        root: opid, nodes, edges, type_env: te,
        cost: CostBound::Unknown, resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

/// Run the IRIS replace_prim component on a program
fn iris_replace_prim(program: &SemanticGraph, new_opcode: u8) -> Option<SemanticGraph> {
    let (te, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // InputRef(0) = the program to modify
    let inp0 = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0xFF, value: vec![0] }, int_id, 0);
    let inp0_id = inp0.id; nodes.insert(inp0_id, inp0);

    // graph_get_root(input[0]) — get root node ID (opcode 0x8A)
    let root_op = make_node(NodeKind::Prim, NodePayload::Prim { opcode: 0x8A }, int_id, 1);
    let root_id = root_op.id; nodes.insert(root_id, root_op);
    edges.push(Edge { source: root_id, target: inp0_id, port: 0, label: EdgeLabel::Argument });

    // InputRef(1) = new opcode
    let inp1 = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0xFF, value: vec![1] }, int_id, 0);
    let inp1_id = inp1.id; nodes.insert(inp1_id, inp1);

    // GraphSetPrimOp(program, root_id, new_opcode)
    let set_op = make_node(NodeKind::Prim, NodePayload::Prim { opcode: 0x84 }, int_id, 3);
    let set_id = set_op.id; nodes.insert(set_id, set_op);
    edges.push(Edge { source: set_id, target: inp0_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: set_id, target: root_id, port: 1, label: EdgeLabel::Argument });
    edges.push(Edge { source: set_id, target: inp1_id, port: 2, label: EdgeLabel::Argument });

    let iris_program = SemanticGraph {
        root: set_id, nodes, edges, type_env: te,
        cost: CostBound::Unknown, resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    };

    let inputs = vec![
        Value::Program(Rc::new(program.clone())),
        Value::Int(new_opcode as i64),
    ];

    match interpreter::interpret(&iris_program, &inputs, None) {
        Ok((outputs, _)) => {
            match outputs.into_iter().next() {
                Some(Value::Program(p)) => Some(Rc::try_unwrap(p).unwrap_or_else(|rc| (*rc).clone())),
                Some(Value::Tuple(elems)) => {
                    if let Some(Value::Program(p)) = elems.first() {
                        Some((**p).clone())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        Err(_) => None,
    }
}

/// Run the IRIS fitness evaluator on a program
fn iris_evaluate(program: &SemanticGraph, input: &[Value], expected: i64) -> bool {
    let (te, int_id) = int_type_env();
    let mut nodes = HashMap::new();
    let mut edges = Vec::new();

    // graph_eval(program, inputs)
    let inp0 = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0xFF, value: vec![0] }, int_id, 0);
    let inp0_id = inp0.id; nodes.insert(inp0_id, inp0);
    let inp1 = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0xFF, value: vec![1] }, int_id, 0);
    let inp1_id = inp1.id; nodes.insert(inp1_id, inp1);
    let eval_node = make_node(NodeKind::Prim, NodePayload::Prim { opcode: 0x89 }, int_id, 2);
    let eval_id = eval_node.id; nodes.insert(eval_id, eval_node);
    edges.push(Edge { source: eval_id, target: inp0_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: eval_id, target: inp1_id, port: 1, label: EdgeLabel::Argument });

    // eq(result, expected)
    let exp_lit = make_node(NodeKind::Lit, NodePayload::Lit { type_tag: 0, value: expected.to_le_bytes().to_vec() }, int_id, 0);
    let exp_id = exp_lit.id; nodes.insert(exp_id, exp_lit);
    let eq_node = make_node(NodeKind::Prim, NodePayload::Prim { opcode: 0x20 }, int_id, 2);
    let eq_id = eq_node.id; nodes.insert(eq_id, eq_node);
    edges.push(Edge { source: eq_id, target: eval_id, port: 0, label: EdgeLabel::Argument });
    edges.push(Edge { source: eq_id, target: exp_id, port: 1, label: EdgeLabel::Argument });

    let iris_eval = SemanticGraph {
        root: eq_id, nodes, edges, type_env: te,
        cost: CostBound::Unknown, resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    };

    let inputs = vec![
        Value::Program(Rc::new(program.clone())),
        Value::tuple(input.to_vec()),
    ];

    match interpreter::interpret(&iris_eval, &inputs, None) {
        Ok((outputs, _)) => matches!(outputs.first(), Some(Value::Bool(true)) | Some(Value::Int(1))),
        Err(_) => false,
    }
}

#[test]
fn iris_components_in_real_evolution() {
    println!("\n{}", "=".repeat(80));
    println!("  IRIS Components in Production: Integration Test");
    println!("  Using IRIS-written components to run actual evolution");
    println!("{}\n", "=".repeat(80));

    // =====================================================================
    // Test 1: IRIS mutation transforms a real program
    // =====================================================================
    println!("--- Test 1: IRIS mutation on real programs ---");

    let sub_program = make_program(0x01); // sub(a, b)
    let result = interpreter::interpret(&sub_program, &[Value::Int(10), Value::Int(3)], None);
    println!("  sub(10, 3) = {:?}", result.as_ref().ok().map(|(o,_)| o.first()));

    // Use IRIS replace_prim to change sub → add
    let mutated = iris_replace_prim(&sub_program, 0x00).expect("IRIS mutation failed");
    let result2 = interpreter::interpret(&mutated, &[Value::Int(10), Value::Int(3)], None);
    println!("  After IRIS mutation (sub→add): add(10, 3) = {:?}", result2.as_ref().ok().map(|(o,_)| o.first()));

    match result2 {
        Ok((ref outputs, _)) if outputs.first() == Some(&Value::Int(13)) => {
            println!("  ✓ IRIS mutation produced correct program");
        }
        _ => println!("  ✗ IRIS mutation produced wrong result"),
    }

    // =====================================================================
    // Test 2: IRIS fitness evaluator scores programs correctly
    // =====================================================================
    println!("\n--- Test 2: IRIS fitness evaluator ---");

    let add_program = make_program(0x00); // add(a, b)
    let pass = iris_evaluate(&add_program, &[Value::Int(3), Value::Int(5)], 8);
    println!("  add(3,5)==8? {} (expected true)", pass);

    let fail = iris_evaluate(&add_program, &[Value::Int(3), Value::Int(5)], 9);
    println!("  add(3,5)==9? {} (expected false)", fail);

    assert!(pass, "Correct test case should pass");
    assert!(!fail, "Wrong expected should fail");
    println!("  ✓ IRIS evaluator scores correctly");

    // =====================================================================
    // Test 3: Manual evolution loop using IRIS components
    // =====================================================================
    println!("\n--- Test 3: Manual evolution loop with IRIS components ---");

    // Goal: evolve add(a,b) from sub(a,b) by trying different opcodes
    // This is a tiny evolution: population of 1, mutate, evaluate, keep if better

    let target_inputs = vec![
        (vec![Value::Int(3), Value::Int(5)], 8i64),
        (vec![Value::Int(10), Value::Int(1)], 11),
        (vec![Value::Int(0), Value::Int(0)], 0),
    ];

    let mut current = make_program(0x01); // start with sub
    let mut current_score = 0;

    // Score the initial program
    for (inputs, expected) in &target_inputs {
        if iris_evaluate(&current, inputs, *expected) {
            current_score += 1;
        }
    }
    println!("  Gen 0: score = {}/{} (sub program)", current_score, target_inputs.len());

    // Try mutations: change opcode to 0x00 (add), 0x02 (mul), 0x07 (min), 0x08 (max)
    for &opcode in &[0x00u8, 0x02, 0x07, 0x08] {
        if let Some(mutant) = iris_replace_prim(&current, opcode) {
            let mut score = 0;
            for (inputs, expected) in &target_inputs {
                if iris_evaluate(&mutant, inputs, *expected) {
                    score += 1;
                }
            }
            let op_name = match opcode { 0x00 => "add", 0x02 => "mul", 0x07 => "min", 0x08 => "max", _ => "?" };
            println!("  Mutant ({}): score = {}/{}", op_name, score, target_inputs.len());

            if score > current_score {
                current = mutant;
                current_score = score;
                println!("  → Accepted! New best.");
            }
        }
    }

    println!("\n  Final score: {}/{}", current_score, target_inputs.len());
    if current_score == target_inputs.len() {
        println!("  ✓ IRIS components evolved a perfect program!");
    } else {
        println!("  ✗ Evolution incomplete (best: {}/{})", current_score, target_inputs.len());
    }

    // Verify the final program works
    let final_result = interpreter::interpret(&current, &[Value::Int(3), Value::Int(5)], None);
    println!("  Final program: f(3, 5) = {:?}", final_result.as_ref().ok().map(|(o,_)| o.first()));

    assert_eq!(current_score, target_inputs.len(), "Should find perfect solution");

    // =====================================================================
    // Test 4: Multi-generation evolution with IRIS components
    // =====================================================================
    println!("\n--- Test 4: Multi-generation hill climbing ---");

    // Start from mul, evolve toward add using IRIS mutation + IRIS evaluation
    let mut program = make_program(0x02); // start with mul
    let opcodes_to_try = [0x00u8, 0x01, 0x02, 0x07, 0x08]; // add, sub, mul, min, max

    for generation in 0..5 {
        let mut best_score = 0;
        let mut best_program = program.clone();

        // Evaluate current
        for (inputs, expected) in &target_inputs {
            if iris_evaluate(&program, inputs, *expected) {
                best_score += 1;
            }
        }

        // Try each mutation
        for &opcode in &opcodes_to_try {
            if let Some(mutant) = iris_replace_prim(&program, opcode) {
                let mut score = 0;
                for (inputs, expected) in &target_inputs {
                    if iris_evaluate(&mutant, inputs, *expected) {
                        score += 1;
                    }
                }
                if score > best_score {
                    best_score = score;
                    best_program = mutant;
                }
            }
        }

        program = best_program;
        println!("  Gen {}: score = {}/{}", generation, best_score, target_inputs.len());

        if best_score == target_inputs.len() {
            println!("  ✓ Perfect solution found at generation {}!", generation);
            break;
        }
    }

    let check = interpreter::interpret(&program, &[Value::Int(7), Value::Int(3)], None);
    println!("  Evolved program: f(7, 3) = {:?}", check.as_ref().ok().map(|(o,_)| o.first()));
    assert_eq!(check.ok().map(|(o,_)| o), Some(vec![Value::Int(10)]));

    println!("\n{}", "=".repeat(80));
    println!("  ALL TESTS PASSED");
    println!("  IRIS components successfully:");
    println!("    - Mutated programs (IRIS replace_prim)");
    println!("    - Evaluated programs (IRIS fitness evaluator via graph_eval)");
    println!("    - Ran a complete evolution loop (mutate → evaluate → select)");
    println!("    - Evolved mul(a,b) → add(a,b) in 1 generation");
    println!("  IRIS can evolve programs using programs written in IRIS.");
    println!("{}\n", "=".repeat(80));
}
