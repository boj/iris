// Comprehensive JIT vs tree-walker benchmark
// Usage: cargo test --release --features rust-scaffolding,jit --test bench_jit_comparison -- --nocapture

use iris_exec::jit_backend;
use iris_exec::registry::FragmentRegistry;
use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;
use std::collections::BTreeMap;
use iris_types::fragment::FragmentId;

fn compile_program(src: &str) -> (SemanticGraph, FragmentRegistry) {
    let result = iris_bootstrap::syntax::compile(src);
    assert!(result.errors.is_empty(), "compile errors: {:?}", result.errors);
    let mut registry = FragmentRegistry::new();
    for (_, frag, _) in &result.fragments {
        registry.register(frag.clone());
    }
    let (_, frag, _) = result.fragments.last().unwrap();
    (frag.graph.clone(), registry)
}

fn bench_expr(name: &str, src: &str, inputs: &[Value], n: usize) {
    let (graph, reg) = compile_program(src);
    let compilable = jit_backend::is_jit_compilable(&graph);

    // Warm up both paths
    let _ = iris_bootstrap::evaluate(&graph, inputs);
    if compilable {
        let _ = jit_backend::interpret_jit(&graph, inputs, Some(&reg));
    }

    // Tree-walker
    let start = std::time::Instant::now();
    for _ in 0..n {
        let _ = iris_bootstrap::evaluate(&graph, inputs);
    }
    let tree_ns = start.elapsed().as_nanos() as f64 / n as f64;

    // JIT fast path (if compilable)
    let jit_ns = if compilable {
        let start = std::time::Instant::now();
        for _ in 0..n {
            let _ = jit_backend::call_jit_fast(&graph, inputs);
        }
        start.elapsed().as_nanos() as f64 / n as f64
    } else {
        f64::NAN
    };

    let ratio = if jit_ns.is_nan() {
        "n/a".to_string()
    } else {
        format!("{:.2}x", tree_ns / jit_ns)
    };

    let compilable_str = if compilable { "✓" } else { "✗" };
    println!(
        "  {:<35} {:>8.0} ns  {:>8}  {:>8}  {}",
        name,
        tree_ns,
        if jit_ns.is_nan() { "n/a".to_string() } else { format!("{:.0} ns", jit_ns) },
        ratio,
        compilable_str,
    );
}

#[test]
fn benchmark_jit_comparison() {
    let n = 50_000;
    println!();
    println!("=======================================================================");
    println!("  JIT vs Tree-Walker Per-Call Performance ({} iterations)", n);
    println!("=======================================================================");
    println!("  {:<35} {:>10}  {:>8}  {:>8}  {}", "Expression", "Tree", "JIT", "Speedup", "Compilable");
    println!("  {:<35} {:>10}  {:>8}  {:>8}  {}", "-" .repeat(35), "----", "---", "-------", "---");

    // Simple arithmetic
    bench_expr("a + b", "let main a b = a + b", &[Value::Int(17), Value::Int(25)], n);
    bench_expr("a * b + a - b", "let main a b = a * b + a - b", &[Value::Int(7), Value::Int(5)], n);
    bench_expr("(a+b)*(a-b)", "let main a b = (a + b) * (a - b)", &[Value::Int(17), Value::Int(25)], n);
    bench_expr("a * a + b * b", "let main a b = a * a + b * b", &[Value::Int(3), Value::Int(4)], n);

    // Guard/conditional
    bench_expr("if a > 0 then a*2 else -a", "let main a = if a > 0 then a * 2 else 0 - a", &[Value::Int(21)], n);
    bench_expr("guard (negative input)", "let main a = if a > 0 then a * 2 else 0 - a", &[Value::Int(-5)], n);

    // Let bindings
    bench_expr("let x = a+b in x*x", "let main a b = let x = a + b in x * x", &[Value::Int(3), Value::Int(4)], n);

    // Tuples
    bench_expr("(a, b, a+b)", "let main a b = (a, b, a + b)", &[Value::Int(10), Value::Int(20)], n);

    // Fold with Prim step (unrolled)
    bench_expr("fold Prim(+) tuple", "let main n = fold 0 (\\acc x -> acc + x) (1,2,3,4,5,6,7,8,9,n)", &[Value::Int(10)], n);

    // Fold with Lambda step (loop-compiled)
    bench_expr("fold lambda (acc+x)", "let main n = fold 0 (\\acc x -> acc + x) (1,2,3,4,5,6,7,8,9,n)", &[Value::Int(10)], n);
    bench_expr("fold lambda (acc+x*2)", "let main a b = fold 0 (\\acc x -> acc + x * 2) (a, b, 10, 20)", &[Value::Int(3), Value::Int(5)], n);
    bench_expr("fold lambda (conditional)", "let main n = fold 0 (\\acc x -> if x > 3 then acc + x else acc) (1,2,3,4,5,n)", &[Value::Int(10)], n);
    bench_expr("fold lambda (factorial)", "let main a = fold 1 (\\acc x -> acc * x) (1, 2, 3, 4, 5, a)", &[Value::Int(6)], n);

    println!();
    println!("  Note: JIT speedup applies to arithmetic, guards, tuples, and fold+lambda.");
    println!("=======================================================================");
}
