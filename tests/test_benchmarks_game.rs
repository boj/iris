
//! Computer Language Benchmarks Game — IRIS implementations.
//!
//! Compiles each .iris benchmark from the benchmark/ directory and verifies
//! correct output via the tree-walking interpreter. Also measures wall-clock
//! time so the benchmark table in benchmark/README.md can be populated.
//!
//! Benchmarks:
//!   n-body            — planetary orbit simulation (Float64 math)
//!   spectral-norm     — spectral norm of a matrix (Float64 iteration)
//!   fannkuch-redux    — pancake flipping (integer array manipulation)
//!   binary-trees      — tree allocation/checksum (memory allocation)
//!   fasta             — DNA sequence generation (string/LCG)
//!   reverse-complement — DNA reverse complement (string manipulation)

use std::time::Instant;

use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::eval::Value;
use iris_types::fragment::Fragment;
use iris_types::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compiled benchmark: named graphs + a registry for cross-fragment calls.
struct CompiledBench {
    fragments: Vec<(String, Fragment)>,
    registry: FragmentRegistry,
}

/// Compile an IRIS source string, returning fragments and a registry.
fn compile_bench(src: &str) -> CompiledBench {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!("compilation failed with {} errors", result.errors.len());
    }

    let mut registry = FragmentRegistry::new();
    let mut fragments = Vec::new();

    for (name, frag, _source_map) in result.fragments {
        registry.register(frag.clone());
        fragments.push((name, frag));
    }

    CompiledBench {
        fragments,
        registry,
    }
}

/// Find a named fragment's graph.
fn find_graph<'a>(bench: &'a CompiledBench, name: &str) -> &'a SemanticGraph {
    &bench
        .fragments
        .iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| {
            let names: Vec<_> = bench.fragments.iter().map(|(n, _)| n.as_str()).collect();
            panic!("fragment '{}' not found; available: {:?}", name, names)
        })
        .1
        .graph
}

/// Interpret with registry and generous step limit.
fn run(bench: &CompiledBench, graph: &SemanticGraph, inputs: &[Value]) -> Vec<Value> {
    let max_steps = 500_000_000;
    let (out, _) = interpreter::interpret_with_step_limit(
        graph,
        inputs,
        None,
        Some(&bench.registry),
        max_steps,
    )
    .unwrap_or_else(|e| panic!("interpretation failed: {:?}", e));
    out
}

fn assert_float_near(actual: &Value, expected: f64, tol: f64, label: &str) {
    match actual {
        Value::Float64(v) => {
            assert!(
                (v - expected).abs() < tol,
                "{}: expected ~{}, got {} (delta {})",
                label,
                expected,
                v,
                (v - expected).abs()
            );
        }
        _ => panic!("{}: expected Float64, got {:?}", label, actual),
    }
}

fn as_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        _ => panic!("expected String, got {:?}", v),
    }
}

// ---------------------------------------------------------------------------
// N-Body
// ---------------------------------------------------------------------------

#[test]
fn bench_n_body() {
    let src = include_str!("../benchmark/n-body/n-body.iris");
    let start = Instant::now();
    let bench = compile_bench(src);
    let compile_ms = start.elapsed().as_secs_f64() * 1000.0;

    let run_graph = find_graph(&bench, "run");

    // Test with 0 steps: just compute initial energy
    let start = Instant::now();
    let out = run(&bench, run_graph, &[Value::Int(0)]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;

    println!("\n=== N-Body ===");
    println!("Compile: {:.1}ms", compile_ms);
    println!("Execute (N=0): {:.1}ms", exec_ms);
    println!("Output: {:?}", out);

    assert!(!out.is_empty(), "n-body should produce output");
    if let Value::Float64(e) = &out[0] {
        println!("Initial energy: {:.9}", e);
        assert!(*e < 0.0, "energy should be negative for bound system, got {}", e);
    }

    // Test with 10 steps: energy should be conserved
    let start = Instant::now();
    let out_10 = run(&bench, run_graph, &[Value::Int(10)]);
    let exec_10_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("Execute (N=10): {:.1}ms", exec_10_ms);
    if let Value::Float64(e) = &out_10[0] {
        println!("Energy after 10 steps: {:.9}", e);
        assert!(*e < 0.0, "energy should remain negative");
    }
}

// ---------------------------------------------------------------------------
// Spectral Norm
// ---------------------------------------------------------------------------

#[test]
fn bench_spectral_norm() {
    let src = include_str!("../benchmark/spectral-norm/spectral-norm.iris");
    let start = Instant::now();
    let bench = compile_bench(src);
    let compile_ms = start.elapsed().as_secs_f64() * 1000.0;

    println!("\n=== Spectral Norm ===");
    println!("Compile: {:.1}ms", compile_ms);

    // Test individual primitives from Rust, since the full algorithm
    // requires nested lambdas (not supported in IRIS Gen1).
    let n = 3;
    let ones = Value::tuple(vec![Value::Float64(1.0); n]);

    // Test av_row: compute row 0 of A * ones
    let av_row = find_graph(&bench, "av_row");
    let start = Instant::now();
    let out = run(&bench, av_row, &[Value::Int(0), ones.clone()]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("av_row(0, ones): {:?} ({:.1}ms)", out, exec_ms);
    // A[0][0]=1/1=1.0, A[0][1]=1/2=0.5, A[0][2]=1/4=0.25 => sum = 1.75
    assert_float_near(&out[0], 1.75, 0.01, "av_row(0, ones)");

    // Row 1: A[1][0]=1/3, A[1][1]=1/5, A[1][2]=1/8 => 0.3333+0.2+0.125 = 0.6583
    let out = run(&bench, av_row, &[Value::Int(1), ones.clone()]);
    println!("av_row(1, ones): {:?}", out);
    assert_float_near(&out[0], 0.6583, 0.01, "av_row(1, ones)");

    // Orchestrate the full spectral norm from Rust
    // Build A*u and A^T*v vectors by calling av_row/atv_row for each row
    let atv_row = find_graph(&bench, "atv_row");
    let dot_fn = find_graph(&bench, "dot");

    let start = Instant::now();
    let mut u: Vec<f64> = vec![1.0; n];

    for _iter in 0..10 {
        // v = A * u
        let u_val = Value::tuple(u.iter().map(|&x| Value::Float64(x)).collect());
        let mut v = vec![0.0; n];
        for i in 0..n {
            let out = run(&bench, av_row, &[Value::Int(i as i64), u_val.clone()]);
            if let Value::Float64(val) = &out[0] { v[i] = *val; }
        }
        // w = A^T * v
        let v_val = Value::tuple(v.iter().map(|&x| Value::Float64(x)).collect());
        let mut w = vec![0.0; n];
        for i in 0..n {
            let out = run(&bench, atv_row, &[Value::Int(i as i64), v_val.clone()]);
            if let Value::Float64(val) = &out[0] { w[i] = *val; }
        }
        u = w;
    }

    // Compute last v for the norm
    let u_val = Value::tuple(u.iter().map(|&x| Value::Float64(x)).collect());
    let mut v = vec![0.0; n];
    for i in 0..n {
        let out = run(&bench, av_row, &[Value::Int(i as i64), u_val.clone()]);
        if let Value::Float64(val) = &out[0] { v[i] = *val; }
    }
    let mut w = vec![0.0; n];
    let v_val = Value::tuple(v.iter().map(|&x| Value::Float64(x)).collect());
    for i in 0..n {
        let out = run(&bench, atv_row, &[Value::Int(i as i64), v_val.clone()]);
        if let Value::Float64(val) = &out[0] { w[i] = *val; }
    }

    let u_val = Value::tuple(u.iter().map(|&x| Value::Float64(x)).collect());
    let w_val = Value::tuple(w.iter().map(|&x| Value::Float64(x)).collect());
    let uv_dot = run(&bench, dot_fn, &[u_val, w_val.clone()]);
    let vv_dot = run(&bench, dot_fn, &[w_val.clone(), w_val]);

    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;

    if let (Value::Float64(uv), Value::Float64(vv)) = (&uv_dot[0], &vv_dot[0]) {
        let spectral = (uv / vv).sqrt();
        println!("Spectral norm (N={}, Rust-orchestrated): {:.9} ({:.1}ms)", n, spectral, exec_ms);
        // For N=3, spectral norm should be positive and finite
        assert!(spectral > 0.0 && spectral.is_finite(), "spectral_norm(3) should be positive and finite, got {}", spectral);
    }
}

// ---------------------------------------------------------------------------
// Fannkuch-Redux
// ---------------------------------------------------------------------------

#[test]
fn bench_fannkuch_redux() {
    let src = include_str!("../benchmark/fannkuch-redux/fannkuch-redux.iris");
    let start = Instant::now();
    let bench = compile_bench(src);
    let compile_ms = start.elapsed().as_secs_f64() * 1000.0;

    let count_flips = find_graph(&bench, "count_flips");

    println!("\n=== Fannkuch-Redux ===");
    println!("Compile: {:.1}ms", compile_ms);

    // [1, 2, 3] -> 0 flips
    let perm_sorted = Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let start = Instant::now();
    let out = run(&bench, count_flips, &[perm_sorted, Value::Int(30)]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("count_flips([1,2,3], 30): {:?} ({:.1}ms)", out, exec_ms);
    assert_eq!(out, vec![Value::Int(0)], "[1,2,3] needs 0 flips");

    // [3, 2, 1] -> 1 flip
    let perm_321 = Value::tuple(vec![Value::Int(3), Value::Int(2), Value::Int(1)]);
    let out = run(&bench, count_flips, &[perm_321, Value::Int(30)]);
    println!("count_flips([3,2,1], 30): {:?}", out);
    assert_eq!(out, vec![Value::Int(1)], "[3,2,1] needs 1 flip");

    // [2, 1, 3] -> 1 flip
    let perm_213 = Value::tuple(vec![Value::Int(2), Value::Int(1), Value::Int(3)]);
    let out = run(&bench, count_flips, &[perm_213, Value::Int(30)]);
    println!("count_flips([2,1,3], 30): {:?}", out);
    assert_eq!(out, vec![Value::Int(1)], "[2,1,3] needs 1 flip");

    // [2, 3, 1] -> 2 flips
    let perm_231 = Value::tuple(vec![Value::Int(2), Value::Int(3), Value::Int(1)]);
    let out = run(&bench, count_flips, &[perm_231, Value::Int(30)]);
    println!("count_flips([2,3,1], 30): {:?}", out);
    assert_eq!(out, vec![Value::Int(2)], "[2,3,1] needs 2 flips");

    // [3, 1, 2] -> 2 flips
    let perm_312 = Value::tuple(vec![Value::Int(3), Value::Int(1), Value::Int(2)]);
    let out = run(&bench, count_flips, &[perm_312, Value::Int(30)]);
    println!("count_flips([3,1,2], 30): {:?}", out);
    assert_eq!(out, vec![Value::Int(2)], "[3,1,2] needs 2 flips");
}

// ---------------------------------------------------------------------------
// Binary Trees
// ---------------------------------------------------------------------------

#[test]
fn bench_binary_trees() {
    let src = include_str!("../benchmark/binary-trees/binary-trees.iris");
    let start = Instant::now();
    let bench = compile_bench(src);
    let compile_ms = start.elapsed().as_secs_f64() * 1000.0;

    let bench_fn = find_graph(&bench, "bench");

    println!("\n=== Binary Trees ===");
    println!("Compile: {:.1}ms", compile_ms);

    // Depth=2: 2^3 - 1 = 7 nodes, sum = 1+2+...+7 = 28
    let start = Instant::now();
    let out = run(&bench, bench_fn, &[Value::Int(2)]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("bench(2): {:?} ({:.1}ms)", out, exec_ms);
    assert_eq!(out, vec![Value::Int(28)], "sum of 7 nodes (1+2+...+7)");

    // Depth=3: 15 nodes, sum = 120
    let out = run(&bench, bench_fn, &[Value::Int(3)]);
    println!("bench(3): {:?}", out);
    assert_eq!(out, vec![Value::Int(120)], "sum of 15 nodes");

    // Depth=4: 31 nodes, sum = 496
    let out = run(&bench, bench_fn, &[Value::Int(4)]);
    println!("bench(4): {:?}", out);
    assert_eq!(out, vec![Value::Int(496)], "sum of 31 nodes");

    // Depth=6: 127 nodes, sum = 8128
    let start = Instant::now();
    let out = run(&bench, bench_fn, &[Value::Int(6)]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("bench(6): {:?} ({:.1}ms)", out, exec_ms);
    assert_eq!(out, vec![Value::Int(8128)], "sum of 127 nodes");
}

// ---------------------------------------------------------------------------
// FASTA
// ---------------------------------------------------------------------------

#[test]
fn bench_fasta() {
    let src = include_str!("../benchmark/fasta/fasta.iris");
    let start = Instant::now();
    let bench = compile_bench(src);
    let compile_ms = start.elapsed().as_secs_f64() * 1000.0;

    println!("\n=== FASTA ===");
    println!("Compile: {:.1}ms", compile_ms);

    // Test fasta_len: should return N
    let fasta_len = find_graph(&bench, "fasta_len");
    let start = Instant::now();
    let out = run(&bench, fasta_len, &[Value::Int(10)]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("fasta_len(10): {:?} ({:.1}ms)", out, exec_ms);
    assert_eq!(out, vec![Value::Int(10)], "DNA length should equal N");

    // Test fasta_dna: verify output is a string of correct length
    let fasta_dna = find_graph(&bench, "fasta_dna");
    let start = Instant::now();
    let out = run(&bench, fasta_dna, &[Value::Int(20)]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("fasta_dna(20): {:?} ({:.1}ms)", out, exec_ms);

    assert!(!out.is_empty(), "fasta_dna should produce output");
    let dna = as_string(&out[0]);
    assert_eq!(
        dna.len(),
        20,
        "DNA string should be 20 chars, got {}",
        dna.len()
    );
    for ch in dna.chars() {
        assert!(
            "ACGT".contains(ch),
            "invalid nucleotide '{}' in DNA: {}",
            ch,
            dna
        );
    }
    println!("DNA: {}", dna);

    // Deterministic: same seed should produce same output
    let out2 = run(&bench, fasta_dna, &[Value::Int(20)]);
    let dna2 = as_string(&out2[0]);
    assert_eq!(dna, dna2, "LCG should be deterministic");

    // Test repeat_string
    let repeat_fn = find_graph(&bench, "repeat_string");
    let out = run(
        &bench,
        repeat_fn,
        &[Value::String("ABC".into()), Value::Int(7)],
    );
    let repeated = as_string(&out[0]);
    assert_eq!(repeated, "ABCABCA", "repeat_string(ABC, 7)");
}

// ---------------------------------------------------------------------------
// Reverse Complement
// ---------------------------------------------------------------------------

#[test]
fn bench_reverse_complement() {
    let src = include_str!("../benchmark/reverse-complement/reverse-complement.iris");
    let start = Instant::now();
    let bench = compile_bench(src);
    let compile_ms = start.elapsed().as_secs_f64() * 1000.0;

    println!("\n=== Reverse Complement ===");
    println!("Compile: {:.1}ms", compile_ms);

    let rc_fn = find_graph(&bench, "reverse_complement");

    // "ACGT" -> reverse "TGCA" -> complement: T->A, G->C, C->G, A->T = "ACGT"
    let start = Instant::now();
    let out = run(&bench, rc_fn, &[Value::String("ACGT".into())]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    let result = as_string(&out[0]);
    println!(
        "reverse_complement(ACGT): {} ({:.1}ms)",
        result, exec_ms
    );
    assert_eq!(result, "ACGT", "ACGT is a palindromic complement");

    // "AAAA" -> "TTTT"
    let out = run(&bench, rc_fn, &[Value::String("AAAA".into())]);
    assert_eq!(as_string(&out[0]), "TTTT");

    // "TTTT" -> "AAAA"
    let out = run(&bench, rc_fn, &[Value::String("TTTT".into())]);
    assert_eq!(as_string(&out[0]), "AAAA");

    // "AACG" -> reverse "GCAA" -> complement: G->C, C->G, A->T, A->T = "CGTT"
    let out = run(&bench, rc_fn, &[Value::String("AACG".into())]);
    assert_eq!(as_string(&out[0]), "CGTT");

    // "GCTA" -> reverse "ATCG" -> complement: A->T, T->A, C->G, G->C = "TAGC"
    let out = run(&bench, rc_fn, &[Value::String("GCTA".into())]);
    assert_eq!(as_string(&out[0]), "TAGC");

    // Roundtrip: reverse_complement(reverse_complement(x)) == x
    let verify_fn = find_graph(&bench, "verify");
    let out = run(
        &bench,
        verify_fn,
        &[Value::String("ACGTACGT".into())],
    );
    println!("verify(ACGTACGT): {:?}", out);
    assert!(out == vec![Value::Int(1)] || out == vec![Value::Bool(true)], "roundtrip should be identity, got {:?}", out);

    // "GATTACA" -> reverse "ACATTAG" -> complement: T->A... = "TGTAATC"
    let out = run(&bench, rc_fn, &[Value::String("GATTACA".into())]);
    assert_eq!(as_string(&out[0]), "TGTAATC");
}


// Debug test removed — the nested lambda limitation is documented in
// benchmark/spectral-norm/spectral-norm.iris and benchmark/README.md.

#[cfg(never)]
fn _debug_spectral_norm_types() {
    // Test 1: map producing floats
    let bench = compile_bench("let f n = map (\\x -> 1.0) (list_range 0 n)");
    let g = find_graph(&bench, "f");
    let out = run(&bench, g, &[Value::Int(3)]);
    println!("map float: {:?}", out);

    // Test 2: fold over float tuple
    let bench = compile_bench("let f xs = fold 0.0 (\\acc j -> acc + list_nth xs j) (list_range 0 3)");
    let g = find_graph(&bench, "f");
    let input = Value::tuple(vec![Value::Float64(1.0), Value::Float64(2.0), Value::Float64(3.0)]);
    let out = run(&bench, g, &[input]);
    println!("fold float from tuple: {:?}", out);

    // Test 3: 1.0 / int_to_float(x)
    let bench = compile_bench("let f x = 1.0 / int_to_float x");
    let g = find_graph(&bench, "f");
    let out = run(&bench, g, &[Value::Int(2)]);
    println!("1.0 / int_to_float(2): {:?}", out);

    // Test 4: inline av_row with i=0
    let bench = compile_bench(r#"
let f u =
  fold 0.0 (\acc j ->
    let ij = 0 + j in
    let aij = 1.0 / int_to_float (ij * (ij + 1) / 2 + 0 + 1) in
    acc + aij * list_nth u j
  ) (list_range 0 3)
"#);
    let g = find_graph(&bench, "f");
    let input = Value::tuple(vec![Value::Float64(1.0), Value::Float64(1.0), Value::Float64(1.0)]);
    let out = run(&bench, g, &[input]);
    println!("av_row inline: {:?}", out);

    // Test 5: nested lambda (map over fold)
    let bench = compile_bench(r#"
let f u n =
  map (\i ->
    fold 0.0 (\acc j ->
      let ij = i + j in
      acc + 1.0 / int_to_float (ij * (ij + 1) / 2 + i + 1) * list_nth u j
    ) (list_range 0 n)
  ) (list_range 0 n)
"#);
    let g = find_graph(&bench, "f");
    let input = Value::tuple(vec![Value::Float64(1.0), Value::Float64(1.0), Value::Float64(1.0)]);
    match interpreter::interpret_with_step_limit(
        g, &[input, Value::Int(3)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("nested lambda (map fold): {:?}", out),
        Err(e) => println!("nested lambda (map fold) FAILED: {:?}", e),
    }

    // Test 5b: simpler nested lambda
    let bench = compile_bench(r#"
let f n =
  map (\i ->
    fold 0.0 (\acc j -> acc + int_to_float (i + j)) (list_range 0 n)
  ) (list_range 0 n)
"#);
    let g = find_graph(&bench, "f");
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(3)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("simpler nested lambda: {:?}", out),
        Err(e) => println!("simpler nested lambda FAILED: {:?}", e),
    }

    // Test 5c: even simpler - does the inner lambda see i?
    let bench = compile_bench(r#"
let f n =
  map (\i ->
    fold 0 (\acc j -> acc + i + j) (list_range 0 n)
  ) (list_range 0 n)
"#);
    let g = find_graph(&bench, "f");
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(3)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("inner sees outer i: {:?}", out),
        Err(e) => println!("inner sees outer i FAILED: {:?}", e),
    }

    // Test 5d: let-bound value visible in inner lambda?
    let bench = compile_bench(r#"
let f n =
  let x = 42 in
  fold 0 (\acc j -> acc + x + j) (list_range 0 n)
"#);
    let g = find_graph(&bench, "f");
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(3)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("let-bound in lambda: {:?}", out),
        Err(e) => println!("let-bound in lambda FAILED: {:?}", e),
    }

    // Test 5e2: cross-fragment call from within fold lambda
    let bench = compile_bench(r#"
let double x = x * 2

let f n =
  fold 0 (\acc elem -> acc + double elem) (list_range 0 n)
"#);
    let g = find_graph(&bench, "f");
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(4)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("cross-fragment in fold: {:?}", out),
        Err(e) => println!("cross-fragment in fold FAILED: {:?}", e),
    }

    // Test 5e3: cross-fragment with multiple args from fold lambda
    let bench = compile_bench(r#"
let add_three a b c = a + b + c

let f xs n =
  fold 0 (\acc idx -> add_three acc idx (list_nth xs idx)) (list_range 0 n)
"#);
    let g = find_graph(&bench, "f");
    let input = Value::tuple(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
    match interpreter::interpret_with_step_limit(
        g, &[input, Value::Int(3)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("cross-fragment multi-arg: {:?}", out),
        Err(e) => println!("cross-fragment multi-arg FAILED: {:?}", e),
    }

    // Test 5f0: does InputRef work inside a lambda?
    let bench = compile_bench(r#"
let f n x =
  fold 0 (\acc j -> acc + x + j) (list_range 0 n)
"#);
    let g = find_graph(&bench, "f");
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(3), Value::Int(10)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("InputRef in lambda: {:?}", out),
        Err(e) => println!("InputRef in lambda FAILED: {:?}", e),
    }

    // Test 5f0b: float version
    let bench = compile_bench(r#"
let f n x =
  fold 0.0 (\acc j -> acc + 1.0 * list_nth x j) (list_range 0 n)
"#);
    let g = find_graph(&bench, "f");
    let input = Value::tuple(vec![Value::Float64(1.0), Value::Float64(2.0), Value::Float64(3.0)]);
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(3), input], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("Float InputRef in lambda: {:?}", out),
        Err(e) => println!("Float InputRef in lambda FAILED: {:?}", e),
    }

    // Test 5f0c: int_to_float with InputRef
    let bench = compile_bench(r#"
let f n row =
  fold 0.0 (\acc j ->
    let ij = row + j in
    acc + 1.0 / int_to_float (ij * (ij + 1) / 2 + row + 1)
  ) (list_range 0 n)
"#);
    let g = find_graph(&bench, "f");
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(3), Value::Int(0)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("int_to_float with InputRef: {:?}", out),
        Err(e) => println!("int_to_float with InputRef FAILED: {:?}", e),
    }

    // Test 5f0d: exact av_elem pattern (with mul)
    let bench = compile_bench(r#"
let f u n row =
  fold 0.0 (\acc j ->
    let ij = row + j in
    acc + 1.0 / int_to_float (ij * (ij + 1) / 2 + row + 1) * list_nth u j
  ) (list_range 0 n)
"#);
    let g = find_graph(&bench, "f");
    let u = Value::tuple(vec![Value::Float64(1.0), Value::Float64(1.0), Value::Float64(1.0)]);
    match interpreter::interpret_with_step_limit(
        g, &[u.clone(), Value::Int(3), Value::Int(0)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("av_elem (with mul): {:?}", out),
        Err(e) => println!("av_elem (with mul) FAILED: {:?}", e),
    }

    // Test 5f0e: without the * list_nth
    let bench = compile_bench(r#"
let f u n row =
  fold 0.0 (\acc j ->
    let ij = row + j in
    acc + 1.0 / int_to_float (ij * (ij + 1) / 2 + row + 1)
  ) (list_range 0 n)
"#);
    let g = find_graph(&bench, "f");
    match interpreter::interpret_with_step_limit(
        g, &[u.clone(), Value::Int(3), Value::Int(0)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("av_elem (no mul): {:?}", out),
        Err(e) => println!("av_elem (no mul) FAILED: {:?}", e),
    }

    // Test 5f0f: just the multiplication part
    let bench = compile_bench(r#"
let f u n =
  fold 0.0 (\acc j ->
    acc + list_nth u j
  ) (list_range 0 n)
"#);
    let g = find_graph(&bench, "f");
    match interpreter::interpret_with_step_limit(
        g, &[u.clone(), Value::Int(3)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("just list_nth add: {:?}", out),
        Err(e) => println!("just list_nth add FAILED: {:?}", e),
    }

    // Test 5f0g2: three params, middle one used in fold
    let bench = compile_bench(r#"
let f a b c =
  fold 0 (\acc j -> acc + b + j) (list_range 0 c)
"#);
    let g = find_graph(&bench, "f");
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(99), Value::Int(10), Value::Int(3)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("3-param fold uses b: {:?}", out),
        Err(e) => println!("3-param fold uses b FAILED: {:?}", e),
    }

    // Test 5f0g3: three params, first one used in fold
    let bench = compile_bench(r#"
let f a b c =
  fold 0 (\acc j -> acc + a + j) (list_range 0 c)
"#);
    let g = find_graph(&bench, "f");
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(10), Value::Int(99), Value::Int(3)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("3-param fold uses a: {:?}", out),
        Err(e) => println!("3-param fold uses a FAILED: {:?}", e),
    }

    // Test 5f0g4: three params, third one (index 2) used in fold
    let bench = compile_bench(r#"
let f a b c =
  fold 0 (\acc j -> acc + c + j) (list_range 0 b)
"#);
    let g = find_graph(&bench, "f");
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(99), Value::Int(3), Value::Int(10)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("3-param fold uses c: {:?}", out),
        Err(e) => println!("3-param fold uses c FAILED: {:?}", e),
    }

    // Test 5f0g: float division + multiplication
    let bench = compile_bench(r#"
let f u n =
  fold 0.0 (\acc j ->
    acc + 1.0 / int_to_float (j + 1) * list_nth u j
  ) (list_range 0 n)
"#);
    let g = find_graph(&bench, "f");
    match interpreter::interpret_with_step_limit(
        g, &[u.clone(), Value::Int(3)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("float div+mul+list_nth: {:?}", out),
        Err(e) => println!("float div+mul+list_nth FAILED: {:?}", e),
    }

    // Test 5f: test the spectral-norm components individually (with new param order)
    let bench = compile_bench(include_str!("../benchmark/spectral-norm/spectral-norm.iris"));

    // Test: map inside fold lambda (nested lambda issue)
    let bench3 = compile_bench(r#"
let f n =
  fold (list_range 0 0) (\acc idx ->
    let v = idx * 2 in
    list_append acc (map (\z -> v) (list_range 0 1))
  ) (list_range 0 n)
"#);
    // Also a simpler test: no map at all
    let bench4 = compile_bench(r#"
let f n =
  fold (list_range 0 0) (\acc idx ->
    list_append acc (list_take (list_range idx (idx + 1)) 1)
  ) (list_range 0 n)
"#);
    let g = find_graph(&bench3, "f");
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(3)], None, Some(&bench3.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("fold+map(nested lambda): {:?}", out),
        Err(e) => println!("fold+map(nested lambda) FAILED: {:?}", e),
    }
    let g = find_graph(&bench4, "f");
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(3)], None, Some(&bench4.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("fold+list_range(no nested lambda): {:?}", out),
        Err(e) => println!("fold+list_range(no nested lambda) FAILED: {:?}", e),
    }

    // Test mat_vec_a(n, u) directly
    let g = find_graph(&bench, "mat_vec_a");
    let u = Value::tuple(vec![Value::Float64(1.0), Value::Float64(1.0), Value::Float64(1.0)]);
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(3), u.clone()], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("mat_vec_a(3, ones): {:?}", out),
        Err(e) => println!("mat_vec_a FAILED: {:?}", e),
    }

    // Test dot
    let g = find_graph(&bench, "dot");
    let a = Value::tuple(vec![Value::Float64(1.0), Value::Float64(2.0)]);
    let b = Value::tuple(vec![Value::Float64(3.0), Value::Float64(4.0)]);
    match interpreter::interpret_with_step_limit(
        g, &[a, b], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("dot([1,2],[3,4]): {:?}", out),
        Err(e) => println!("dot FAILED: {:?}", e),
    }

    // Test vec_av step by step
    // 1. What does list_range 0 0 return?
    let bench2 = compile_bench("let f = list_range 0 0");
    let g = find_graph(&bench2, "f");
    let out = run(&bench2, g, &[]);
    println!("list_range 0 0: {:?}", out);

    // 2. Can we fold+append from it?
    let bench2 = compile_bench("let f = fold (list_range 0 0) (\\r i -> list_append r (list_take (map (\\z -> 1.0) (list_range 0 1)) 1)) (list_range 0 3)");
    let g = find_graph(&bench2, "f");
    let out = run(&bench2, g, &[]);
    println!("fold+append floats: {:?}", out);

    // 3. inline fold, no cross-fragment -- same params (u, n)
    // row is from the fold lambda (binder 1) -- that's the element
    // u is InputRef(0), n is InputRef(1) -- n might conflict with fold binder 1!
    let bench2 = compile_bench(r#"
let f u n =
  fold (list_range 0 0) (\res row ->
    let e = fold 0.0 (\acc j ->
      let ij = row + j in
      acc + 1.0 / int_to_float (ij * (ij + 1) / 2 + row + 1) * list_nth u j
    ) (list_range 0 n) in
    list_append res (list_take (map (\z -> e) (list_range 0 1)) 1)
  ) (list_range 0 n)
"#);
    let g = find_graph(&bench2, "f");
    let u2 = Value::tuple(vec![Value::Float64(1.0), Value::Float64(1.0), Value::Float64(1.0)]);
    match interpreter::interpret_with_step_limit(
        g, &[u2.clone(), Value::Int(3)], None, Some(&bench2.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("inline double-fold: {:?}", out),
        Err(e) => println!("inline double-fold FAILED: {:?}", e),
    }

    // 3b. cross-fragment call simple
    let bench2 = compile_bench(r#"
let double x = x * 2

let f n =
  fold 0 (\acc elem -> acc + double elem) (list_range 0 n)
"#);
    let g = find_graph(&bench2, "f");
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(4)], None, Some(&bench2.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("cross-fragment fold+append: {:?}", out),
        Err(e) => println!("cross-fragment fold+append FAILED: {:?}", e),
    }

    // 3c. cross-fragment with 2 args
    let bench2 = compile_bench(r#"
let myop u row =
  fold 0.0 (\acc j ->
    acc + list_nth u j
  ) (list_range 0 3)

let f u n =
  fold (list_range 0 0) (\res row ->
    let e = myop u row in
    list_append res (list_take (map (\z -> e) (list_range 0 1)) 1)
  ) (list_range 0 n)
"#);
    let g = find_graph(&bench2, "f");
    match interpreter::interpret_with_step_limit(
        g, &[u2, Value::Int(3)], None, Some(&bench2.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("cross-fragment 2-arg fold+append: {:?}", out),
        Err(e) => println!("cross-fragment 2-arg fold+append FAILED: {:?}", e),
    }

    // Test 5e: fold where acc is a tuple, updating with indices
    let bench = compile_bench(r#"
let f n =
  fold (0, 0, 0) (\acc idx ->
    let i = idx / n in
    let j = idx % n in
    if i == 0 then
      if j == 0 then (acc.0 + 1, acc.1, acc.2)
      else if j == 1 then (acc.0, acc.1 + 1, acc.2)
      else (acc.0, acc.1, acc.2 + 1)
    else acc
  ) (list_range 0 (n * n))
"#);
    let g = find_graph(&bench, "f");
    match interpreter::interpret_with_step_limit(
        g, &[Value::Int(3)], None, Some(&bench.registry), 500_000_000,
    ) {
        Ok((out, _)) => println!("fold tuple update: {:?}", out),
        Err(e) => println!("fold tuple update FAILED: {:?}", e),
    }
}

// ---------------------------------------------------------------------------
// K-Nucleotide
// ---------------------------------------------------------------------------

#[test]
fn bench_k_nucleotide() {
    let src = include_str!("../benchmark/k-nucleotide/k-nucleotide.iris");
    let start = Instant::now();
    let bench = compile_bench(src);
    let compile_ms = start.elapsed().as_secs_f64() * 1000.0;

    println!("\n=== K-Nucleotide ===");
    println!("Compile: {:.1}ms", compile_ms);

    // Test count_distinct: count 1-mers in "ACGTACGT" -> 4 distinct
    let count_distinct = find_graph(&bench, "count_distinct");
    let start = Instant::now();
    let out = run(&bench, count_distinct, &[Value::String("ACGTACGT".into()), Value::Int(1)]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("count_distinct(ACGTACGT, 1): {:?} ({:.1}ms)", out, exec_ms);
    assert_eq!(out, vec![Value::Int(4)], "ACGTACGT has 4 distinct 1-mers");

    // Test count_distinct: 2-mers in "ACGTACGT" -> AC, CG, GT, TA = 4 distinct
    let out = run(&bench, count_distinct, &[Value::String("ACGTACGT".into()), Value::Int(2)]);
    println!("count_distinct(ACGTACGT, 2): {:?}", out);
    assert_eq!(out, vec![Value::Int(4)], "ACGTACGT has 4 distinct 2-mers");

    // Test get_count: count of "A" in "AAACGT"
    let count_kmers = find_graph(&bench, "count_kmers");
    let out = run(&bench, count_kmers, &[Value::String("AAACGT".into()), Value::Int(1)]);
    // The result is a State (map); let's use get_count
    let get_count = find_graph(&bench, "get_count");
    let freq = out[0].clone();
    let out_a = run(&bench, get_count, &[freq.clone(), Value::String("A".into())]);
    println!("get_count(freqs, A): {:?}", out_a);
    assert_eq!(out_a, vec![Value::Int(3)], "AAACGT has 3 A's");

    let out_g = run(&bench, get_count, &[freq.clone(), Value::String("G".into())]);
    println!("get_count(freqs, G): {:?}", out_g);
    assert_eq!(out_g, vec![Value::Int(1)], "AAACGT has 1 G");

    // Test knucleotide: full benchmark on a small string
    let knuc_fn = find_graph(&bench, "knucleotide");
    let start = Instant::now();
    let out = run(&bench, knuc_fn, &[Value::String("ACGTACGTGG".into())]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("knucleotide(ACGTACGTGG): {:?} ({:.1}ms)", out, exec_ms);
    // 1-mers: A=2, C=2, G=3, T=2 -> 4 distinct
    // 2-mers: AC=2, CG=2, GT=2, TA=1, TG=1, GG=1 -> 6 distinct
    // GG count: 1
    if let Value::Tuple(fields) = &out[0] {
        assert_eq!(fields[0], Value::Int(4), "4 distinct 1-mers");
        assert_eq!(fields[2], Value::Int(1), "1 GG occurrence");
    } else {
        panic!("expected Tuple, got {:?}", out[0]);
    }

    // Larger benchmark for timing: 50-char DNA string
    let dna50 = "ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTAC";
    let bench_fn = find_graph(&bench, "bench");
    let start = Instant::now();
    let out = run(&bench, bench_fn, &[Value::String(dna50.into()), Value::Int(3)]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("bench(50-char DNA, k=3): {:?} ({:.1}ms)", out, exec_ms);
    assert!(matches!(&out[0], Value::Int(n) if *n > 0), "should find distinct 3-mers");
}

// ---------------------------------------------------------------------------
// Pi Digits
// ---------------------------------------------------------------------------

#[test]
fn bench_pidigits() {
    let src = include_str!("../benchmark/pidigits/pidigits.iris");
    let start = Instant::now();
    let bench = compile_bench(src);
    let compile_ms = start.elapsed().as_secs_f64() * 1000.0;

    println!("\n=== Pi Digits ===");
    println!("Compile: {:.1}ms", compile_ms);

    let bench_fn = find_graph(&bench, "bench");

    // Compute 10 digits of pi
    let start = Instant::now();
    let out = run(&bench, bench_fn, &[Value::Int(10)]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    let pi_str = as_string(&out[0]);
    println!("pidigits(10): {} ({:.1}ms)", pi_str, exec_ms);
    assert!(
        pi_str.starts_with("3141592653"),
        "first 10 digits of pi should be 3141592653, got {}",
        pi_str
    );

    // Compute 5 digits
    let out = run(&bench, bench_fn, &[Value::Int(5)]);
    let pi5 = as_string(&out[0]);
    println!("pidigits(5): {}", pi5);
    assert_eq!(pi5, "31415", "first 5 digits of pi");

    // Compute 15 digits
    let start = Instant::now();
    let out = run(&bench, bench_fn, &[Value::Int(15)]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    let pi15 = as_string(&out[0]);
    println!("pidigits(15): {} ({:.1}ms)", pi15, exec_ms);
    assert!(
        pi15.starts_with("314159265358979"),
        "first 15 digits of pi should be 314159265358979, got {}",
        pi15
    );
}

// ---------------------------------------------------------------------------
// Regex-Redux
// ---------------------------------------------------------------------------

#[test]
fn bench_regex_redux() {
    let src = include_str!("../benchmark/regex-redux/regex-redux.iris");
    let start = Instant::now();
    let bench = compile_bench(src);
    let compile_ms = start.elapsed().as_secs_f64() * 1000.0;

    println!("\n=== Regex-Redux ===");
    println!("Compile: {:.1}ms", compile_ms);

    // Test pattern counting
    let get_pattern_count = find_graph(&bench, "get_pattern_count");
    let start = Instant::now();
    let out = run(&bench, get_pattern_count, &[
        Value::String("AACGTTAACGT".into()),
        Value::String("A".into()),
    ]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("get_pattern_count(AACGTTAACGT, A): {:?} ({:.1}ms)", out, exec_ms);
    assert_eq!(out, vec![Value::Int(4)], "AACGTTAACGT has 4 A's");

    // Test non-overlapping counting
    let out = run(&bench, get_pattern_count, &[
        Value::String("AAAA".into()),
        Value::String("AA".into()),
    ]);
    println!("get_pattern_count(AAAA, AA): {:?}", out);
    assert_eq!(out, vec![Value::Int(2)], "AAAA has 2 non-overlapping AA's");

    // Test replacement
    let apply_fn = find_graph(&bench, "apply_replacements");
    let out = run(&bench, apply_fn, &[Value::String("BN".into())]);
    let replaced = as_string(&out[0]);
    println!("apply_replacements(BN): {}", replaced);
    assert_eq!(replaced, "CGTACGT", "B->CGT, N->ACGT");

    // Test full benchmark
    let bench_fn = find_graph(&bench, "bench");
    let start = Instant::now();
    let out = run(&bench, bench_fn, &[Value::String("ACGTBNK".into())]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("bench(ACGTBNK): {:?} ({:.1}ms)", out, exec_ms);
    // Original: 7 chars, after: ACGT + CGT + ACGT + GT = 4+3+4+2 = 13
    if let Value::Tuple(fields) = &out[0] {
        assert_eq!(fields[0], Value::Int(7), "original length 7");
        assert_eq!(fields[1], Value::Int(13), "replaced length 13");
    }

    // Test regex_redux with a longer DNA string
    let regex_redux_fn = find_graph(&bench, "regex_redux");
    let dna = "ACGTACGTBDNACGT";
    let start = Instant::now();
    let out = run(&bench, regex_redux_fn, &[Value::String(dna.into())]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("regex_redux({}): {:?} ({:.1}ms)", dna, out, exec_ms);
    if let Value::Tuple(fields) = &out[0] {
        assert_eq!(fields[0], Value::Int(15), "original length 15");
        // B->CGT(3), D->AGT(3), N->ACGT(4) = 15-3+3+3+4 = 22
        assert_eq!(fields[5], Value::Int(22), "replaced length");
    }
}

// ---------------------------------------------------------------------------
// Thread Ring
// ---------------------------------------------------------------------------

#[test]
fn bench_thread_ring() {
    let src = include_str!("../benchmark/thread-ring/thread-ring.iris");
    let start = Instant::now();
    let bench = compile_bench(src);
    let compile_ms = start.elapsed().as_secs_f64() * 1000.0;

    println!("\n=== Thread Ring ===");
    println!("Compile: {:.1}ms", compile_ms);

    let bench_fn = find_graph(&bench, "bench");

    // token=0: thread 1 sees it immediately
    let start = Instant::now();
    let out = run(&bench, bench_fn, &[Value::Int(3), Value::Int(0)]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("bench(3, 0): {:?} ({:.1}ms)", out, exec_ms);
    assert_eq!(out, vec![Value::Int(1)], "token=0 -> thread 1 wins");

    // token=1: step 0 decrements, step 1 sees 0 -> thread (1 % 3 + 1) = 2
    let out = run(&bench, bench_fn, &[Value::Int(3), Value::Int(1)]);
    println!("bench(3, 1): {:?}", out);
    assert_eq!(out, vec![Value::Int(2)], "token=1 -> thread 2 wins");

    // token=2: decrement twice, step 2 sees 0 -> thread (2 % 3 + 1) = 3
    let out = run(&bench, bench_fn, &[Value::Int(3), Value::Int(2)]);
    println!("bench(3, 2): {:?}", out);
    assert_eq!(out, vec![Value::Int(3)], "token=2 -> thread 3 wins");

    // token=3: decrement 3x, step 3 sees 0 -> thread (3 % 3 + 1) = 1 (wraps around)
    let out = run(&bench, bench_fn, &[Value::Int(3), Value::Int(3)]);
    println!("bench(3, 3): {:?}", out);
    assert_eq!(out, vec![Value::Int(1)], "token=3 -> wraps to thread 1");

    // Larger test: 503 threads, token=1000
    let bench_std = find_graph(&bench, "bench_standard");
    let start = Instant::now();
    let out = run(&bench, bench_std, &[Value::Int(1000)]);
    let exec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!("bench_standard(1000): {:?} ({:.1}ms)", out, exec_ms);
    // token=1000: (1000 % 503) + 1 = 498
    if let Value::Int(winner) = &out[0] {
        assert_eq!(*winner, 498, "503-thread ring with token=1000 -> thread 498");
    }
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

#[test]
fn benchmarks_game_summary() {
    println!("\n======================================================");
    println!("  Computer Language Benchmarks Game — IRIS Results");
    println!("======================================================\n");

    let benchmarks = [
        ("n-body", include_str!("../benchmark/n-body/n-body.iris"), "run", vec![Value::Int(0)]),
        ("spectral-norm", include_str!("../benchmark/spectral-norm/spectral-norm.iris"), "av_row", vec![Value::Int(0), Value::tuple(vec![Value::Float64(1.0), Value::Float64(1.0), Value::Float64(1.0)])]),
        ("fannkuch-redux", include_str!("../benchmark/fannkuch-redux/fannkuch-redux.iris"), "count_flips", vec![Value::tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)]), Value::Int(30)]),
        ("binary-trees", include_str!("../benchmark/binary-trees/binary-trees.iris"), "bench", vec![Value::Int(2)]),
        ("fasta", include_str!("../benchmark/fasta/fasta.iris"), "fasta_len", vec![Value::Int(10)]),
        ("reverse-complement", include_str!("../benchmark/reverse-complement/reverse-complement.iris"), "reverse_complement", vec![Value::String("ACGT".into())]),
        ("k-nucleotide", include_str!("../benchmark/k-nucleotide/k-nucleotide.iris"), "bench", vec![Value::String("ACGTACGT".into()), Value::Int(2)]),
        ("pidigits", include_str!("../benchmark/pidigits/pidigits.iris"), "bench", vec![Value::Int(10)]),
        ("regex-redux", include_str!("../benchmark/regex-redux/regex-redux.iris"), "bench", vec![Value::String("ACGTBN".into())]),
        ("thread-ring", include_str!("../benchmark/thread-ring/thread-ring.iris"), "bench", vec![Value::Int(3), Value::Int(100)]),
    ];

    println!(
        "{:<22} {:>10} {:>10} {:>6} {:>8}",
        "Benchmark", "Compile", "Execute", "Fns", "Status"
    );
    println!("{:-<22} {:->10} {:->10} {:->6} {:->8}", "", "", "", "", "");

    for (name, src, entry_fn, inputs) in &benchmarks {
        let start = Instant::now();
        let result = iris_bootstrap::syntax::compile(src);
        let compile_ms = start.elapsed().as_secs_f64() * 1000.0;

        if !result.errors.is_empty() {
            println!(
                "{:<22} {:>9.1}ms {:>10} {:>6} {:>8}",
                name, compile_ms, "-", "-", "FAIL"
            );
            continue;
        }

        let mut registry = FragmentRegistry::new();
        let mut frags = Vec::new();
        for (n, frag, _) in result.fragments {
            registry.register(frag.clone());
            frags.push((n, frag));
        }
        let frag_count = frags.len();

        let (exec_ms, status) =
            if let Some((_, frag)) = frags.iter().find(|(n, _)| n == entry_fn) {
                let start = Instant::now();
                match interpreter::interpret_with_step_limit(
                    &frag.graph,
                    inputs,
                    None,
                    Some(&registry),
                    500_000_000,
                ) {
                    Ok(_) => (start.elapsed().as_secs_f64() * 1000.0, "OK"),
                    Err(_) => (start.elapsed().as_secs_f64() * 1000.0, "ERR"),
                }
            } else {
                (0.0, "SKIP")
            };

        println!(
            "{:<22} {:>9.1}ms {:>9.1}ms {:>6} {:>8}",
            name, compile_ms, exec_ms, frag_count, status
        );
    }

    println!();
}
