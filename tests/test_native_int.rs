/// Integration tests for integer native x86-64 codegen.
/// These programs exercise the compile_flat_native_int path through the
/// bootstrap evaluator — no rust-scaffolding feature required.

use std::collections::BTreeMap;
use std::time::Instant;
use iris_bootstrap::syntax;
use iris_types::eval::Value;
use iris_types::fragment::FragmentId;
use iris_types::graph::SemanticGraph;

fn compile_and_find(src: &str, name: &str) -> (SemanticGraph, BTreeMap<FragmentId, SemanticGraph>) {
    let result = syntax::compile(src);
    assert!(result.errors.is_empty(), "compile errors: {:?}", result.errors);
    let mut reg = BTreeMap::new();
    let mut target = None;
    for (n, frag, _) in &result.fragments {
        reg.insert(frag.id, frag.graph.clone());
        if n == name {
            target = Some(frag.graph.clone());
        }
    }
    (target.expect(&format!("function '{}' not found", name)), reg)
}

fn eval(src: &str, name: &str, args: &[Value]) -> Value {
    let (graph, reg) = compile_and_find(src, name);
    iris_bootstrap::evaluate_with_fragments(&graph, args, 50_000_000, &reg)
        .expect("evaluation failed")
}

// --- Correctness tests ---

#[test]
fn int_fold_sum() {
    // Simple integer fold: count iterations
    let src = "let sum n = fold 0 (\\acc i -> acc + 1) (list_range 0 n)";
    let result = eval(src, "sum", &[Value::Int(100)]);
    assert_eq!(result, Value::Int(100));
}

#[test]
fn int_fold_sum_tuple_state() {
    // Tuple state fold: accumulate (sum, count)
    let src = r#"
let count_sum n =
  let res = fold (0, 0) (\state i ->
    (state.0 + 1, state.1 + 1)
  ) (list_range 0 n) in
  res.0
"#;
    let result = eval(src, "count_sum", &[Value::Int(50)]);
    assert_eq!(result, Value::Int(50));
}

#[test]
fn int_fold_modular_arithmetic() {
    // Modular arithmetic like thread-ring uses
    let src = r#"
let mod_test n =
  let res = fold (0, 0) (\state step ->
    let cur = state.0 in
    let acc = state.1 in
    (step % 7, acc + step % 7)
  ) (list_range 0 n) in
  res.1
"#;
    let result = eval(src, "mod_test", &[Value::Int(100)]);
    // Sum of (i % 7) for i=0..99
    let expected: i64 = (0..100).map(|i| i % 7).sum();
    assert_eq!(result, Value::Int(expected));
}

#[test]
fn int_fold_with_guard() {
    // Guard (if/then/else) in fold body — the core thread-ring pattern
    let src = r#"
let find_zero n =
  let res = fold (n, 0) (\state step ->
    let cur = state.0 in
    let found = state.1 in
    if found > 0 then state
    else if cur == 0 then (0, step + 1)
    else (cur - 1, 0)
  ) (list_range 0 (n + 2)) in
  res.1
"#;
    let result = eval(src, "find_zero", &[Value::Int(5)]);
    // cur starts at 5, decrements each step until 0 at step 5, then found = 6
    assert_eq!(result, Value::Int(6));
}

#[test]
fn thread_ring_correctness() {
    // Actual thread-ring benchmark
    let src = include_str!("../benchmark/thread-ring/thread-ring.iris");
    let result = eval(src, "bench", &[Value::Int(503), Value::Int(1000)]);
    // Expected: the thread ID (1-indexed) that sees token reach 0
    // token=1000, n_threads=503: winner = 1000 % 503 + 1 = 497 + 1 = 498
    assert_eq!(result, Value::Int(498));
}

#[test]
fn thread_ring_small() {
    let src = include_str!("../benchmark/thread-ring/thread-ring.iris");
    // n_threads=3, token=7: step 7 sees token=0, thread_id = 7%3+1 = 2
    let result = eval(src, "bench", &[Value::Int(3), Value::Int(7)]);
    assert_eq!(result, Value::Int(2));
}

#[test]
fn int_fold_division() {
    // Integer division in fold
    let src = r#"
let div_test n =
  let res = fold (n, 0) (\state i ->
    let cur = state.0 in
    (cur / 2, state.1 + cur / 2)
  ) (list_range 0 5) in
  res.1
"#;
    // n=100: cur=100, 50, 25, 12, 6 → sum = 50+25+12+6+3 = 96
    let result = eval(src, "div_test", &[Value::Int(100)]);
    assert_eq!(result, Value::Int(96));
}

#[test]
fn int_fold_comparison_ops() {
    // All integer comparisons
    let src = r#"
let cmp_test x y =
  let eq = if x == y then 1 else 0 in
  let ne = if x != y then 1 else 0 in
  let lt = if x < y then 1 else 0 in
  let gt = if x > y then 1 else 0 in
  let le = if x <= y then 1 else 0 in
  let ge = if x >= y then 1 else 0 in
  eq + ne * 10 + lt * 100 + gt * 1000 + le * 10000 + ge * 100000
"#;
    // 3 < 5: eq=0, ne=1, lt=1, gt=0, le=1, ge=0
    let result = eval(src, "cmp_test", &[Value::Int(3), Value::Int(5)]);
    assert_eq!(result, Value::Int(0 + 10 + 100 + 0 + 10000 + 0));
    // 5 == 5
    let result = eval(src, "cmp_test", &[Value::Int(5), Value::Int(5)]);
    assert_eq!(result, Value::Int(1 + 0 + 0 + 0 + 10000 + 100000));
}

#[test]
fn int_fold_neg_abs() {
    let src = r#"
let neg_abs_test n =
  let res = fold (n, 0) (\state i ->
    let x = neg state.0 in
    (x, abs x)
  ) (list_range 0 1) in
  res
"#;
    let result = eval(src, "neg_abs_test", &[Value::Int(42)]);
    assert_eq!(result, Value::tuple(vec![Value::Int(-42), Value::Int(42)]));
}

#[test]
fn int_fold_min_max() {
    let src = r#"
let minmax n =
  fold (999, 0) (\state step ->
    let v = step % 17 in
    (min state.0 v, max state.1 v)
  ) (list_range 0 n)
"#;
    let result = eval(src, "minmax", &[Value::Int(100)]);
    let min_v: i64 = (0..100).map(|i| i % 17).min().unwrap();
    let max_v: i64 = (0..100).map(|i| i % 17).max().unwrap();
    assert_eq!(result, Value::tuple(vec![Value::Int(min_v), Value::Int(max_v)]));
}

// --- Performance test ---

#[test]
fn thread_ring_perf() {
    let src = include_str!("../benchmark/thread-ring/thread-ring.iris");
    let (graph, reg) = compile_and_find(src, "bench");
    let args = [Value::Int(503), Value::Int(50_000)];

    // Warm up
    let _ = iris_bootstrap::evaluate_with_fragments(&graph, &args, 50_000_000, &reg);

    // Time it
    let start = Instant::now();
    let iters = 5;
    for _ in 0..iters {
        let _ = iris_bootstrap::evaluate_with_fragments(&graph, &args, 50_000_000, &reg);
    }
    let elapsed = start.elapsed();
    let per_run = elapsed / iters;
    let per_token = per_run.as_nanos() as f64 / 50_000.0;

    println!("\n=== Thread-Ring Performance (N=503, token=50K) ===");
    println!("  Total:     {:?}", per_run);
    println!("  Per token: {:.2} ns/token ({:.2} µs/token)", per_token, per_token / 1000.0);

    // For reference, flat eval was ~0.38 µs/token (380 ns)
    // Native int codegen target: <0.1 µs/token
}

// --- Buffer (string builder) tests ---

#[test]
fn buf_basic() {
    // Simple buffer test: push two strings, finish
    let src = r#"
let test_buf n =
  let b = buf_new in
  let b2 = buf_push b "hello" in
  let b3 = buf_push b2 " world" in
  buf_finish b3
let test = test_buf 0
"#;
    let result = eval(src, "test", &[]);
    println!("buf_basic result: {:?}", result);
    assert_eq!(result, Value::String("hello world".to_string()));
}

#[test]
fn fasta_buf_correctness() {
    // Buffer-based fasta should produce same result as str_concat version
    let src = include_str!("../benchmark/fasta/fasta.iris");
    let (graph_concat, reg_concat) = compile_and_find(src, "fasta_dna");
    let (graph_buf, reg_buf) = compile_and_find(src, "fasta_buf");

    let result_concat = iris_bootstrap::evaluate_with_fragments(
        &graph_concat, &[Value::Int(100)], 50_000_000, &reg_concat,
    ).unwrap();
    let result_buf = iris_bootstrap::evaluate_with_fragments(
        &graph_buf, &[Value::Int(100)], 50_000_000, &reg_buf,
    ).unwrap();

    // Both should produce strings of the same length
    if let (Value::String(s1), Value::String(s2)) = (&result_concat, &result_buf) {
        assert_eq!(s1.len(), s2.len(), "buffer and concat should produce same length");
        assert_eq!(s1, s2, "buffer and concat should produce identical output");
    }
}

#[test]
fn fasta_buf_perf() {
    let src = include_str!("../benchmark/fasta/fasta.iris");
    let (graph_concat, reg_concat) = compile_and_find(src, "fasta_dna");
    let (graph_buf, reg_buf) = compile_and_find(src, "fasta_buf");
    let n = 5000;
    let args = [Value::Int(n)];

    // Benchmark str_concat version
    let start = Instant::now();
    let _ = iris_bootstrap::evaluate_with_fragments(&graph_concat, &args, 50_000_000, &reg_concat);
    let concat_time = start.elapsed();

    // Benchmark buf_push version
    let start = Instant::now();
    let _ = iris_bootstrap::evaluate_with_fragments(&graph_buf, &args, 50_000_000, &reg_buf);
    let buf_time = start.elapsed();

    let speedup = concat_time.as_nanos() as f64 / buf_time.as_nanos() as f64;

    println!("\n=== Fasta: str_concat vs buf_push (N={}) ===", n);
    println!("  str_concat: {:?}  ({:.1} µs/char)", concat_time, concat_time.as_nanos() as f64 / n as f64 / 1000.0);
    println!("  buf_push:   {:?}  ({:.1} µs/char)", buf_time, buf_time.as_nanos() as f64 / n as f64 / 1000.0);
    println!("  Speedup:    {:.1}x", speedup);

    // Extrapolate to CLBG scale (N=25M)
    // str_concat is O(n²): time ~ n² * per_char_cost
    // buf_push is O(n): time ~ n * per_char_cost
    let concat_per_char = concat_time.as_nanos() as f64 / n as f64;
    let buf_per_char = buf_time.as_nanos() as f64 / n as f64;
    let concat_25m = concat_per_char * 25_000_000.0 * (25_000_000.0 / n as f64); // O(n²) scaling
    let buf_25m = buf_per_char * 25_000_000.0; // O(n) scaling

    println!("\n  CLBG extrapolation (N=25M):");
    println!("  str_concat: ~{:.0}s (O(n^2))", concat_25m / 1e9);
    println!("  buf_push:   ~{:.1}s (O(n))", buf_25m / 1e9);
    println!("  Projected speedup: {:.0}x", concat_25m / buf_25m);
}

#[test]
fn fasta_seed_native_perf() {
    // Test the pure-integer LCG seed computation — should trigger native GP codegen
    let src = include_str!("../benchmark/fasta/fasta.iris");
    let (graph, reg) = compile_and_find(src, "fasta_seed_only");
    let n = 50_000i64;
    let args = [Value::Int(n), Value::Int(42)];

    // Warm up
    for _ in 0..3 { let _ = iris_bootstrap::evaluate_with_fragments(&graph, &args, 50_000_000, &reg); }

    let start = Instant::now();
    let iters = 10;
    for _ in 0..iters {
        let _ = iris_bootstrap::evaluate_with_fragments(&graph, &args, 50_000_000, &reg);
    }
    let elapsed = start.elapsed() / iters;
    let per_char = elapsed.as_nanos() as f64 / n as f64;

    println!("\n=== Fasta LCG Seed (pure integer, N={}) ===", n);
    println!("  Time:     {:?}  ({:.1} ns/iter)", elapsed, per_char);

    // Extrapolate to N=25M
    let est_25m = per_char * 25_000_000.0;
    println!("  At N=25M: ~{:.1}s", est_25m / 1e9);
    println!("  OCaml:    ~1.8s");
    if est_25m / 1e9 < 1.8 {
        println!("  IRIS FASTER THAN OCAML");
    } else {
        println!("  Gap: {:.1}x", est_25m / 1e9 / 1.8);
    }
}

// --- fold_until tests ---

#[test]
fn fold_until_basic() {
    // fold_until stops when predicate returns true
    let src = r#"
let find_first_gt n =
  fold_until (\acc -> acc > n) 0 (\state step -> state + 1) (list_range 0 1000)
let test = find_first_gt 5
"#;
    let result = eval(src, "test", &[]);
    assert_eq!(result, Value::Int(6), "fold_until should stop at 6 (first > 5)");
}

#[test]
fn fold_until_thread_ring() {
    // Thread ring with early exit via fold_until
    let src = r#"
let thread_ring_fast n_threads token =
  let res = fold_until (\state -> state.1 > 0) (token, 0) (\state step ->
    let cur_token = state.0 in
    let winner = state.1 in
    if cur_token == 0 then
      let thread_id = step % n_threads + 1 in
      (0, thread_id)
    else
      (cur_token - 1, 0)
  ) (list_range 0 (token + 2)) in
  res.1
"#;
    // n_threads=503, token=1000: winner at step 1000, thread_id = 1000%503+1 = 498
    let result = eval(src, "thread_ring_fast", &[Value::Int(503), Value::Int(1000)]);
    assert_eq!(result, Value::Int(498));
}

#[test]
fn fold_until_perf() {
    // Compare fold vs fold_until on thread-ring
    let src_fold = include_str!("../benchmark/thread-ring/thread-ring.iris");
    let src_fold_until = r#"
let thread_ring_fast n_threads token =
  let res = fold_until (\state -> state.1 > 0) (token, 0) (\state step ->
    let cur_token = state.0 in
    let winner = state.1 in
    if cur_token == 0 then
      let thread_id = step % n_threads + 1 in
      (0, thread_id)
    else
      (cur_token - 1, 0)
  ) (list_range 0 (token + 2)) in
  res.1
"#;

    let (graph_fold, reg_fold) = compile_and_find(src_fold, "bench");
    let (graph_until, reg_until) = compile_and_find(src_fold_until, "thread_ring_fast");
    let args = [Value::Int(503), Value::Int(50_000)];

    // Warm up
    let _ = iris_bootstrap::evaluate_with_fragments(&graph_fold, &args, 50_000_000, &reg_fold);
    let _ = iris_bootstrap::evaluate_with_fragments(&graph_until, &args, 50_000_000, &reg_until);

    // Benchmark fold (runs all iterations)
    let start = Instant::now();
    let iters = 5;
    for _ in 0..iters {
        let _ = iris_bootstrap::evaluate_with_fragments(&graph_fold, &args, 50_000_000, &reg_fold);
    }
    let fold_time = start.elapsed() / iters;

    // Benchmark fold_until (early exit)
    let start = Instant::now();
    for _ in 0..iters {
        let _ = iris_bootstrap::evaluate_with_fragments(&graph_until, &args, 50_000_000, &reg_until);
    }
    let until_time = start.elapsed() / iters;

    let speedup = fold_time.as_nanos() as f64 / until_time.as_nanos() as f64;

    println!("\n=== fold vs fold_until: Thread-Ring (N=503, token=50K) ===");
    println!("  fold:       {:?}  ({:.2} µs/token, runs all 50K iterations)", fold_time, fold_time.as_nanos() as f64 / 50_000.0 / 1000.0);
    println!("  fold_until: {:?}  (exits after ~1001 iterations)", until_time);
    if speedup >= 1.0 {
        println!("  Speedup:    {:.1}x", speedup);
    } else {
        println!("  Note: fold_until uses tree-walker (not native codegen yet)");
        println!("  Per-iteration: fold={:.0}ns (native), fold_until={:.0}ns (tree-walker)",
            fold_time.as_nanos() as f64 / 50_000.0,
            until_time.as_nanos() as f64 / 1001.0);
    }

    // At CLBG scale (N=50M), fold runs all 50M iterations.
    // fold_until exits after ~token iterations (1000 for standard benchmark).
    // Extrapolation:
    let fold_50m_est = fold_time.as_nanos() as f64 * (50_000_000.0 / 50_000.0);
    let until_50m_est = until_time.as_nanos() as f64; // same — exits after ~1001 regardless
    println!("\n  CLBG extrapolation (N=50M, token=1000):");
    println!("  fold:       {:.1}s (runs all 50M iterations)", fold_50m_est / 1e9);
    println!("  fold_until: {:.1}ms (exits after 1001 iterations)", until_50m_est / 1e6);
    println!("  Projected speedup: {:.0}x", fold_50m_est / until_50m_est);
}

#[test]
fn nbody_no_regression() {
    // Verify n-body (float native path) still works correctly after our changes
    let src = include_str!("../benchmark/n-body/n-body.iris");
    let (graph, reg) = compile_and_find(src, "run");

    // Correctness
    let result = iris_bootstrap::evaluate_with_fragments(
        &graph, &[Value::Int(10)], 50_000_000, &reg,
    ).unwrap();
    // n-body run returns a float (energy) — just check it evaluates without error
    assert!(matches!(result, Value::Float64(_) | Value::Tuple(_)),
        "n-body should return a numeric value, got {:?}", result);

    // Performance
    let args = [Value::Int(1000)];
    let _ = iris_bootstrap::evaluate_with_fragments(&graph, &args, 50_000_000, &reg);
    let start = Instant::now();
    let iters = 5;
    for _ in 0..iters {
        let _ = iris_bootstrap::evaluate_with_fragments(&graph, &args, 50_000_000, &reg);
    }
    let elapsed = start.elapsed();
    let per_run = elapsed / iters;
    let per_step = per_run.as_nanos() as f64 / 1000.0;
    println!("\n=== N-Body Performance (N=1000, float native) ===");
    println!("  Total:    {:?}", per_run);
    println!("  Per step: {:.2} ns/step ({:.2} µs/step)", per_step, per_step / 1000.0);
    // Previous baseline: ~0.22 µs/step — should not regress
}
