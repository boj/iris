
//! Profiling benchmark: runs each Benchmarks Game program at increasing input
//! sizes to identify where time is spent and which benchmarks are slowest.

use std::time::Instant;
use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::eval::Value;

fn compile_and_run(src: &str, entry_fn: &str, inputs: &[Value], label: &str) -> f64 {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        eprintln!("{}: {} compile errors", label, result.errors.len());
        return 0.0;
    }
    let mut registry = FragmentRegistry::new();
    let mut frags = Vec::new();
    for (n, frag, _) in result.fragments {
        registry.register(frag.clone());
        frags.push((n, frag));
    }
    let graph = &frags.iter().find(|(n, _)| n == entry_fn).unwrap().1.graph;
    let start = Instant::now();
    match interpreter::interpret_with_step_limit(graph, inputs, None, Some(&registry), 500_000_000) {
        Ok(_) => {}
        Err(e) => eprintln!("{}: {:?}", label, e),
    }
    start.elapsed().as_secs_f64() * 1000.0
}

#[test]
fn profile_all_benchmarks() {
    println!("\n==================================================================");
    println!("  IRIS Benchmark Profiling — Scaling Analysis");
    println!("==================================================================\n");

    // N-Body: Float64-heavy, fold over range
    println!("--- N-Body (Float64 math, fold iteration) ---");
    let src = include_str!("../benchmark/n-body/n-body.iris");
    for n in [0, 10, 50, 100] {
        let ms = compile_and_run(src, "run", &[Value::Int(n)], &format!("n-body N={}", n));
        println!("  N={:>5}: {:>8.1}ms", n, ms);
    }

    // Fannkuch-redux: list manipulation
    println!("\n--- Fannkuch-Redux (list ops: take/drop/append/nth) ---");
    let src = include_str!("../benchmark/fannkuch-redux/fannkuch-redux.iris");
    for perm in [
        vec![3, 2, 1],
        vec![4, 3, 2, 1],
        vec![5, 4, 3, 2, 1],
        vec![6, 5, 4, 3, 2, 1],
    ] {
        let n = perm.len();
        let input = Value::tuple(perm.iter().map(|&x| Value::Int(x)).collect());
        let ms = compile_and_run(src, "count_flips", &[input, Value::Int(30)], "fannkuch");
        println!("  N={}: {:>8.1}ms", n, ms);
    }

    // Binary trees: allocation + map/fold
    println!("\n--- Binary Trees (allocation, map/fold) ---");
    let src = include_str!("../benchmark/binary-trees/binary-trees.iris");
    for d in [2, 4, 6, 8, 10] {
        let ms = compile_and_run(src, "bench", &[Value::Int(d)], &format!("binary-trees d={}", d));
        println!("  depth={:>2}: {:>8.1}ms  (nodes={})", d, ms, (1 << (d + 1)) - 1);
    }

    // FASTA: string building via fold
    println!("\n--- FASTA (string building, LCG, fold) ---");
    let src = include_str!("../benchmark/fasta/fasta.iris");
    for n in [10, 50, 100, 200, 500] {
        let ms = compile_and_run(src, "fasta_dna", &[Value::Int(n)], &format!("fasta N={}", n));
        println!("  N={:>5}: {:>8.1}ms", n, ms);
    }

    // Reverse complement: string manipulation
    println!("\n--- Reverse Complement (string ops: slice, eq, concat) ---");
    let src = include_str!("../benchmark/reverse-complement/reverse-complement.iris");
    for len in [10, 50, 100, 200, 500] {
        let dna: String = "ACGT".chars().cycle().take(len).collect();
        let ms = compile_and_run(src, "reverse_complement", &[Value::String(dna)], &format!("revcomp N={}", len));
        println!("  N={:>5}: {:>8.1}ms", len, ms);
    }

    // K-nucleotide: map operations
    println!("\n--- K-Nucleotide (map insert/get/size, string slicing) ---");
    let src = include_str!("../benchmark/k-nucleotide/k-nucleotide.iris");
    for len in [20, 50, 100, 200] {
        let dna: String = "ACGTACGT".chars().cycle().take(len).collect();
        let ms = compile_and_run(src, "bench", &[Value::String(dna), Value::Int(2)], &format!("k-nuc N={}", len));
        println!("  N={:>5} k=2: {:>8.1}ms", len, ms);
    }

    // Pidigits: integer arithmetic
    println!("\n--- Pi Digits (integer arithmetic, pow, fold) ---");
    let src = include_str!("../benchmark/pidigits/pidigits.iris");
    for n in [5, 10, 15] {
        let ms = compile_and_run(src, "bench", &[Value::Int(n)], &format!("pidigits N={}", n));
        println!("  N={:>2}: {:>8.1}ms", n, ms);
    }

    // Regex-redux: string replacement
    println!("\n--- Regex-Redux (string replace, pattern counting) ---");
    let src = include_str!("../benchmark/regex-redux/regex-redux.iris");
    for len in [20, 50, 100, 200] {
        // Generate DNA with some IUB codes mixed in
        let dna: String = "ACGTBNDK".chars().cycle().take(len).collect();
        let ms = compile_and_run(src, "bench", &[Value::String(dna)], &format!("regex-redux N={}", len));
        println!("  N={:>5}: {:>8.1}ms", len, ms);
    }

    // Thread ring: pure fold iteration
    println!("\n--- Thread Ring (fold iteration, integer ops) ---");
    let src = include_str!("../benchmark/thread-ring/thread-ring.iris");
    for token in [100, 1000, 5000, 10000, 50000] {
        let ms = compile_and_run(src, "bench_standard", &[Value::Int(token)], &format!("thread-ring token={}", token));
        println!("  token={:>6}: {:>8.1}ms", token, ms);
    }

    println!("\n==================================================================");
    println!("  Summary: Cost Model Analysis");
    println!("==================================================================\n");

    // Measure per-step cost for different operation types
    println!("--- Per-step costs (microseconds per fold iteration) ---");

    // Pure integer fold
    let src_int = "let f n = fold 0 (\\acc i -> acc + i * 2 + 1) (list_range 0 n)";
    let bench_int = iris_bootstrap::syntax::compile(src_int);
    let mut reg = FragmentRegistry::new();
    let mut frags = Vec::new();
    for (n, frag, _) in bench_int.fragments {
        reg.register(frag.clone());
        frags.push((n, frag));
    }
    let g = &frags.iter().find(|(n, _)| n == "f").unwrap().1.graph;
    let n_iters = 10000;
    let start = Instant::now();
    let _ = interpreter::interpret_with_step_limit(g, &[Value::Int(n_iters)], None, Some(&reg), 500_000_000);
    let int_us = start.elapsed().as_nanos() as f64 / n_iters as f64 / 1000.0;
    println!("  Integer fold (add+mul):     {:>6.2}us/iter", int_us);

    // Float64 fold
    let src_float = "let f n = fold 0.0 (\\acc i -> acc + int_to_float i * 2.5 + 1.1) (list_range 0 n)";
    let bench_f = iris_bootstrap::syntax::compile(src_float);
    let mut reg = FragmentRegistry::new();
    let mut frags = Vec::new();
    for (n, frag, _) in bench_f.fragments {
        reg.register(frag.clone());
        frags.push((n, frag));
    }
    let g = &frags.iter().find(|(n, _)| n == "f").unwrap().1.graph;
    let start = Instant::now();
    let _ = interpreter::interpret_with_step_limit(g, &[Value::Int(n_iters)], None, Some(&reg), 500_000_000);
    let float_us = start.elapsed().as_nanos() as f64 / n_iters as f64 / 1000.0;
    println!("  Float64 fold (add+mul):     {:>6.2}us/iter", float_us);

    // String concat fold
    let src_str = "let f n = fold \"\" (\\acc i -> str_concat acc \"x\") (list_range 0 n)";
    let bench_s = iris_bootstrap::syntax::compile(src_str);
    let mut reg = FragmentRegistry::new();
    let mut frags = Vec::new();
    for (n, frag, _) in bench_s.fragments {
        reg.register(frag.clone());
        frags.push((n, frag));
    }
    let g = &frags.iter().find(|(n, _)| n == "f").unwrap().1.graph;
    let n_str = 1000;
    let start = Instant::now();
    let _ = interpreter::interpret_with_step_limit(g, &[Value::Int(n_str)], None, Some(&reg), 500_000_000);
    let str_us = start.elapsed().as_nanos() as f64 / n_str as f64 / 1000.0;
    println!("  String concat fold:         {:>6.2}us/iter", str_us);

    // Tuple access fold (list_nth)
    let src_tup = "let f xs n = fold 0 (\\acc i -> acc + list_nth xs (i % 10)) (list_range 0 n)";
    let bench_t = iris_bootstrap::syntax::compile(src_tup);
    let mut reg = FragmentRegistry::new();
    let mut frags = Vec::new();
    for (n, frag, _) in bench_t.fragments {
        reg.register(frag.clone());
        frags.push((n, frag));
    }
    let g = &frags.iter().find(|(n, _)| n == "f").unwrap().1.graph;
    let xs = Value::tuple((0..10).map(|i| Value::Int(i)).collect());
    let start = Instant::now();
    let _ = interpreter::interpret_with_step_limit(g, &[xs, Value::Int(n_iters)], None, Some(&reg), 500_000_000);
    let tup_us = start.elapsed().as_nanos() as f64 / n_iters as f64 / 1000.0;
    println!("  Tuple access fold (list_nth):{:>6.2}us/iter", tup_us);

    // Cross-fragment call fold
    let src_xfrag = "let double x = x * 2\nlet f n = fold 0 (\\acc i -> acc + double i) (list_range 0 n)";
    let bench_x = iris_bootstrap::syntax::compile(src_xfrag);
    let mut reg = FragmentRegistry::new();
    let mut frags = Vec::new();
    for (n, frag, _) in bench_x.fragments {
        reg.register(frag.clone());
        frags.push((n, frag));
    }
    let g = &frags.iter().find(|(n, _)| n == "f").unwrap().1.graph;
    let start = Instant::now();
    let _ = interpreter::interpret_with_step_limit(g, &[Value::Int(n_iters)], None, Some(&reg), 500_000_000);
    let xfrag_us = start.elapsed().as_nanos() as f64 / n_iters as f64 / 1000.0;
    println!("  Cross-fragment call fold:   {:>6.2}us/iter", xfrag_us);

    println!();
}
