
//! Bootstrap evaluator integration tests.
//!
//! Proves the bootstrap chain works:
//!   Rust bootstrap evaluator -> IRIS interpreter -> target program
//!
//! Step 1: Compile full_interpreter.iris to SemanticGraph
//! Step 2: Load it into the bootstrap evaluator
//! Step 3: Run it on simple target programs
//! Step 4: Verify outputs match the Rust interpreter
//! Step 5: Measure overhead

use std::collections::HashMap;
use std::time::Instant;

use iris_bootstrap::{bootstrap_eval, evaluate, save_graph};
use iris_exec::interpreter;
use iris_types::cost::{CostBound, CostTerm};
use iris_types::eval::Value;
use iris_types::graph::{
    Edge, EdgeLabel, Node, NodeId, NodeKind, NodePayload, Resolution, SemanticGraph,
};
use iris_types::hash::SemanticHash;
use iris_types::types::{TypeEnv, TypeId};

// ---------------------------------------------------------------------------
// Helpers: build simple target programs as SemanticGraphs
// ---------------------------------------------------------------------------

fn make_graph(nodes: HashMap<NodeId, Node>, edges: Vec<Edge>, root: u64) -> SemanticGraph {
    SemanticGraph {
        root: NodeId(root),
        nodes,
        edges,
        type_env: TypeEnv {
            types: std::collections::BTreeMap::new(),
        },
        cost: CostBound::Unknown,
        resolution: Resolution::Implementation,
        hash: SemanticHash([0; 32]),
    }
}

fn int_lit(id: u64, value: i64) -> (NodeId, Node) {
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind: NodeKind::Lit,
            type_sig: TypeId(0),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0x00,
                value: value.to_le_bytes().to_vec(),
            },
        },
    )
}

fn input_ref(id: u64, index: u8) -> (NodeId, Node) {
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind: NodeKind::Lit,
            type_sig: TypeId(0),
            cost: CostTerm::Unit,
            arity: 0,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Lit {
                type_tag: 0xFF,
                value: vec![index],
            },
        },
    )
}

fn prim_node(id: u64, opcode: u8, arity: u8) -> (NodeId, Node) {
    (
        NodeId(id),
        Node {
            id: NodeId(id),
            kind: NodeKind::Prim,
            type_sig: TypeId(0),
            cost: CostTerm::Unit,
            arity,
            resolution_depth: 0,
            salt: 0,
            payload: NodePayload::Prim { opcode },
        },
    )
}

fn edge(source: u64, target: u64, port: u8, label: EdgeLabel) -> Edge {
    Edge {
        source: NodeId(source),
        target: NodeId(target),
        port,
        label,
    }
}

/// Build a simple "add two inputs" program: add(input0, input1)
fn make_add_program() -> SemanticGraph {
    let nodes = HashMap::from([
        input_ref(1, 0),
        input_ref(2, 1),
        prim_node(3, 0x00, 2), // add
    ]);
    let edges = vec![
        edge(3, 1, 0, EdgeLabel::Argument),
        edge(3, 2, 1, EdgeLabel::Argument),
    ];
    make_graph(nodes, edges, 3)
}

/// Build a "multiply two inputs" program: mul(input0, input1)
fn make_mul_program() -> SemanticGraph {
    let nodes = HashMap::from([
        input_ref(1, 0),
        input_ref(2, 1),
        prim_node(3, 0x02, 2), // mul
    ]);
    let edges = vec![
        edge(3, 1, 0, EdgeLabel::Argument),
        edge(3, 2, 1, EdgeLabel::Argument),
    ];
    make_graph(nodes, edges, 3)
}

/// Build a constant program: lit(42)
fn make_const_program(value: i64) -> SemanticGraph {
    let nodes = HashMap::from([int_lit(1, value)]);
    make_graph(nodes, vec![], 1)
}

/// Build a program: sub(mul(input0, input0), input1)  i.e. x*x - y
fn make_square_minus_program() -> SemanticGraph {
    let nodes = HashMap::from([
        input_ref(1, 0),
        input_ref(2, 0), // second ref to input 0
        input_ref(3, 1),
        prim_node(4, 0x02, 2), // mul
        prim_node(5, 0x01, 2), // sub
    ]);
    let edges = vec![
        edge(4, 1, 0, EdgeLabel::Argument),
        edge(4, 2, 1, EdgeLabel::Argument),
        edge(5, 4, 0, EdgeLabel::Argument),
        edge(5, 3, 1, EdgeLabel::Argument),
    ];
    make_graph(nodes, edges, 5)
}

/// Compile the IRIS interpreter from source.
fn compile_interpreter() -> SemanticGraph {
    let source = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/iris-programs/interpreter/full_interpreter.iris")
    ).expect("failed to read full_interpreter.iris");
    let result = iris_bootstrap::syntax::compile(&source);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(&source, err));
        }
        panic!("failed to compile full_interpreter.iris: {} errors", result.errors.len());
    }
    assert!(!result.fragments.is_empty(), "no fragments from full_interpreter.iris");
    result.fragments[0].1.graph.clone()
}

/// Compile a program from IRIS surface syntax.
fn compile_program(source: &str) -> SemanticGraph {
    let result = iris_bootstrap::syntax::compile(source);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(source, err));
        }
        panic!("compilation failed");
    }
    result.fragments[0].1.graph.clone()
}

// ---------------------------------------------------------------------------
// Step 1: Bootstrap evaluator directly evaluates simple programs
// ---------------------------------------------------------------------------

#[test]
fn bootstrap_direct_const() {
    let g = make_const_program(42);
    let result = evaluate(&g, &[]).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn bootstrap_direct_add() {
    let g = make_add_program();
    let result = evaluate(&g, &[Value::Int(3), Value::Int(5)]).unwrap();
    assert_eq!(result, Value::Int(8));
}

#[test]
fn bootstrap_direct_mul() {
    let g = make_mul_program();
    let result = evaluate(&g, &[Value::Int(4), Value::Int(7)]).unwrap();
    assert_eq!(result, Value::Int(28));
}

#[test]
fn bootstrap_direct_square_minus() {
    let g = make_square_minus_program();
    let result = evaluate(&g, &[Value::Int(5), Value::Int(3)]).unwrap();
    assert_eq!(result, Value::Int(22)); // 5*5 - 3
}

// ---------------------------------------------------------------------------
// Step 2: Bootstrap evaluator runs the compiled IRIS interpreter
// ---------------------------------------------------------------------------

#[test]
fn bootstrap_iris_interpreter_add() {
    let interpreter = compile_interpreter();
    let target = make_add_program();
    let result = bootstrap_eval(&interpreter, &target, &[Value::Int(3), Value::Int(5)]).unwrap();
    assert_eq!(result, Value::Int(8));
    println!("Bootstrap chain: add(3, 5) = {:?}", result);
}

#[test]
fn bootstrap_iris_interpreter_const() {
    let interpreter = compile_interpreter();
    let target = make_const_program(99);
    let result = bootstrap_eval(&interpreter, &target, &[]).unwrap();
    assert_eq!(result, Value::Int(99));
    println!("Bootstrap chain: const(99) = {:?}", result);
}

#[test]
fn bootstrap_iris_interpreter_mul() {
    let interpreter = compile_interpreter();
    let target = make_mul_program();
    let result = bootstrap_eval(&interpreter, &target, &[Value::Int(6), Value::Int(7)]).unwrap();
    assert_eq!(result, Value::Int(42));
    println!("Bootstrap chain: mul(6, 7) = {:?}", result);
}

// ---------------------------------------------------------------------------
// Step 3: Test with more complex programs compiled from IRIS syntax
// ---------------------------------------------------------------------------

#[test]
fn bootstrap_iris_interpreter_compiled_add() {
    let interpreter = compile_interpreter();
    let target = compile_program("let add2 x y = x + y");
    let result = bootstrap_eval(&interpreter, &target, &[Value::Int(10), Value::Int(20)]).unwrap();
    assert_eq!(result, Value::Int(30));
    println!("Bootstrap chain: compiled add(10, 20) = {:?}", result);
}

#[test]
fn bootstrap_iris_interpreter_compiled_guard() {
    let interpreter = compile_interpreter();
    let target = compile_program("let abs_val x = if x < 0 then -x else x");

    let result = bootstrap_eval(&interpreter, &target, &[Value::Int(-7)]).unwrap();
    assert_eq!(result, Value::Int(7));

    let result = bootstrap_eval(&interpreter, &target, &[Value::Int(5)]).unwrap();
    assert_eq!(result, Value::Int(5));

    println!("Bootstrap chain: abs(-7) = 7, abs(5) = 5");
}

#[test]
fn bootstrap_iris_interpreter_compiled_fold() {
    let interpreter = compile_interpreter();
    let target = compile_program("let sum xs = fold 0 (+) xs");
    let input = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)]);
    let result = bootstrap_eval(&interpreter, &target, &[input]).unwrap();
    assert_eq!(result, Value::Int(10));
    println!("Bootstrap chain: fold sum [1,2,3,4] = {:?}", result);
}

// ---------------------------------------------------------------------------
// Step 4: Verify bootstrap matches Rust interpreter
// ---------------------------------------------------------------------------

#[test]
fn bootstrap_matches_rust_interpreter() {
    let interpreter = compile_interpreter();

    let test_cases: Vec<(&str, Vec<Value>, Value)> = vec![
        ("let id x = x", vec![Value::Int(42)], Value::Int(42)),
        ("let add2 x y = x + y", vec![Value::Int(3), Value::Int(4)], Value::Int(7)),
        ("let neg x = -x", vec![Value::Int(10)], Value::Int(-10)),
        ("let double x = x * 2", vec![Value::Int(5)], Value::Int(10)),
    ];

    for (source, inputs, expected) in &test_cases {
        let target = compile_program(source);

        // Rust interpreter
        let (rust_out, _) = interpreter::interpret(&target, inputs, None).unwrap();
        assert_eq!(rust_out, vec![expected.clone()], "Rust interpreter failed for: {}", source);

        // Bootstrap chain
        let bootstrap_out = bootstrap_eval(&interpreter, &target, inputs).unwrap();
        assert_eq!(bootstrap_out, *expected, "Bootstrap chain failed for: {}", source);

        println!("{}: Rust={:?}, Bootstrap={:?} -- MATCH", source, rust_out[0], bootstrap_out);
    }
}

// ---------------------------------------------------------------------------
// Step 5: Pre-compile interpreter to JSON (for the CLI binary)
// ---------------------------------------------------------------------------

#[test]
fn precompile_interpreter_json() {
    let interpreter = compile_interpreter();
    let json_path = concat!(env!("CARGO_MANIFEST_DIR"), "/bootstrap/interpreter.json");
    save_graph(&interpreter, json_path).expect("failed to save interpreter.json");

    // Verify we can load it back.
    let loaded = iris_bootstrap::load_graph(json_path).expect("failed to reload interpreter.json");
    assert_eq!(loaded.root, interpreter.root);
    assert_eq!(loaded.nodes.len(), interpreter.nodes.len());
    println!("Pre-compiled interpreter to {} ({} nodes, {} edges)",
        json_path, loaded.nodes.len(), loaded.edges.len());
}

// ---------------------------------------------------------------------------
// Step 6: Measure overhead
// ---------------------------------------------------------------------------

#[test]
fn measure_bootstrap_overhead() {
    let interpreter = compile_interpreter();
    let target = make_add_program();
    let inputs = [Value::Int(3), Value::Int(5)];

    // Warm up
    for _ in 0..10 {
        let _ = interpreter::interpret(&target, &inputs, None);
        let _ = bootstrap_eval(&interpreter, &target, &inputs);
    }

    // Measure Rust interpreter directly
    let iterations = 1000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = interpreter::interpret(&target, &inputs, None);
    }
    let rust_time = start.elapsed();

    // Measure bootstrap chain
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = bootstrap_eval(&interpreter, &target, &inputs);
    }
    let bootstrap_time = start.elapsed();

    // Measure bootstrap evaluator directly (no meta-circular layer)
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = evaluate(&target, &inputs);
    }
    let direct_time = start.elapsed();

    let rust_us = rust_time.as_micros() as f64 / iterations as f64;
    let bootstrap_us = bootstrap_time.as_micros() as f64 / iterations as f64;
    let direct_us = direct_time.as_micros() as f64 / iterations as f64;
    let overhead_ratio = bootstrap_us / rust_us;
    let direct_ratio = direct_us / rust_us;

    println!("\n========================================");
    println!("  Bootstrap Overhead Measurement");
    println!("  (add(3, 5) = 8, {} iterations)", iterations);
    println!("========================================");
    println!("  Rust interpreter:   {:.1} us", rust_us);
    println!("  Bootstrap direct:   {:.1} us ({:.1}x)", direct_us, direct_ratio);
    println!("  Bootstrap -> IRIS:  {:.1} us ({:.1}x)", bootstrap_us, overhead_ratio);
    println!("========================================\n");

    // Sanity check: all produce the same result.
    let (rust_result, _) = interpreter::interpret(&target, &inputs, None).unwrap();
    let bootstrap_result = bootstrap_eval(&interpreter, &target, &inputs).unwrap();
    let direct_result = evaluate(&target, &inputs).unwrap();
    assert_eq!(rust_result, vec![Value::Int(8)]);
    assert_eq!(bootstrap_result, Value::Int(8));
    assert_eq!(direct_result, Value::Int(8));
}
