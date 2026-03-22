use iris_exec::jit_backend;
use iris_exec::registry::FragmentRegistry;
use iris_types::eval::Value;
use std::time::Instant;

fn compile_program(src: &str) -> (iris_types::graph::SemanticGraph, FragmentRegistry) {
    let result = iris_bootstrap::syntax::compile(src);
    assert!(result.errors.is_empty());
    let mut registry = FragmentRegistry::new();
    for (_, frag, _) in &result.fragments { registry.register(frag.clone()); }
    let (_, frag, _) = result.fragments.last().unwrap();
    (frag.graph.clone(), registry)
}

#[test]
fn lto_inlining_check() {
    let (graph, reg) = compile_program("let main a b = a * b + a - b");
    let inputs = vec![Value::Int(7), Value::Int(5)];
    let n = 100_000u64;
    let _ = jit_backend::interpret_jit(&graph, &inputs, Some(&reg));

    let start = Instant::now();
    for _ in 0..n { let _ = jit_backend::interpret_jit(&graph, &inputs, Some(&reg)); }
    let full = start.elapsed().as_nanos() as f64 / n as f64;

    let start = Instant::now();
    for _ in 0..n { let _ = jit_backend::interpret_jit_single(&graph, &inputs, Some(&reg)); }
    let single = start.elapsed().as_nanos() as f64 / n as f64;

    let start = Instant::now();
    for _ in 0..n { let _ = jit_backend::call_jit_fast(&graph, &inputs); }
    let fast = start.elapsed().as_nanos() as f64 / n as f64;

    let start = Instant::now();
    for _ in 0..n { let _ = iris_bootstrap::evaluate(&graph, &[Value::Int(7), Value::Int(5)]); }
    let tree = start.elapsed().as_nanos() as f64 / n as f64;

    println!("\n=== Dispatch Performance (100K iterations) ===");
    println!("  Tree-walker:         {:>6.0} ns", tree);
    println!("  interpret_jit (Vec): {:>6.0} ns  ({:.1}x vs tree)", full, tree / full);
    println!("  interpret_jit_single:{:>6.0} ns  ({:.1}x vs tree)", single, tree / single);
    println!("  call_jit_fast:       {:>6.0} ns  ({:.1}x vs tree)", fast, tree / fast);
}
