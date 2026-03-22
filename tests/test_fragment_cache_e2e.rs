/// End-to-end test: generational self-improvement via fragment cache.
///
/// Simulates the full cycle:
///   Generation 0: compile source, evaluate, save original to cache
///   Generation 1: "evolve" an improved version, save to cache
///   Generation 2: load from cache on startup, verify improved version runs

use std::collections::BTreeMap;
use std::path::PathBuf;

use iris_bootstrap::fragment_cache;
use iris_bootstrap::syntax;
use iris_types::eval::Value;
use iris_types::fragment::{Fragment, FragmentId};
use iris_types::graph::SemanticGraph;
use iris_types::hash::compute_fragment_id;

fn compile_all(src: &str) -> Vec<(String, Fragment)> {
    let result = syntax::compile(src);
    assert!(result.errors.is_empty(), "compile errors: {:?}", result.errors);
    result.fragments.into_iter().map(|(n, f, _)| (n, f)).collect()
}

fn find(frags: &[(String, Fragment)], name: &str) -> Fragment {
    frags.iter().find(|(n, _)| n == name).unwrap().1.clone()
}

fn registry(frags: &[(String, Fragment)]) -> BTreeMap<FragmentId, SemanticGraph> {
    frags.iter().map(|(_, f)| (f.id, f.graph.clone())).collect()
}

fn eval(frag: &Fragment, args: &[Value], reg: &BTreeMap<FragmentId, SemanticGraph>) -> Value {
    iris_bootstrap::evaluate_with_fragments(&frag.graph, args, 10_000_000, reg).unwrap()
}

#[test]
fn generational_improvement_persists() {
    let cache_dir = PathBuf::from(std::env::temp_dir()).join("iris-e2e-gen-test");
    let _ = std::fs::remove_dir_all(&cache_dir);

    // === Generation 0: Compile and run the "pre-evolution" version ===
    let pre_src = r#"
let compute n =
  let res = fold (0, 0, 0, 0) (\state step ->
    let h = state.0 in
    let c = state.1 in
    let sum = state.2 in
    let max_h = state.3 in
    let new_h = (h * 31 + step) % 1000003 in
    let new_max = if new_h > max_h then new_h else max_h in
    (new_h, c + 1, sum + new_h, new_max)
  ) (list_range 0 n) in
  (res.0, res.1)
"#;

    let frags_gen0 = compile_all(pre_src);
    let compute_gen0 = find(&frags_gen0, "compute");
    let reg_gen0 = registry(&frags_gen0);
    let result_gen0 = eval(&compute_gen0, &[Value::Int(500)], &reg_gen0);

    println!("\n=== Generation 0 (original) ===");
    println!("  FragmentId: {}", fragment_cache::fragment_id_hex(&compute_gen0.id));
    println!("  Result: {:?}", result_gen0);

    // No cache yet
    assert!(fragment_cache::load_fragment(&cache_dir, "compute").is_none());
    assert_eq!(fragment_cache::generation(&cache_dir, "compute"), 0);

    // === Generation 1: Evolution produces an improved version ===
    // Simulate what --improve does: evolve a leaner version, save to cache.
    let post_src = r#"
let compute n =
  fold (0, 0) (\state step ->
    let new_h = (state.0 * 31 + step) % 1000003 in
    (new_h, state.1 + 1)
  ) (list_range 0 n)
"#;

    let frags_gen1 = compile_all(post_src);
    let compute_gen1 = find(&frags_gen1, "compute");
    let reg_gen1 = registry(&frags_gen1);
    let result_gen1 = eval(&compute_gen1, &[Value::Int(500)], &reg_gen1);

    // Same result, different structure
    assert_eq!(result_gen0, result_gen1, "improved version must produce same result");
    assert_ne!(compute_gen0.id, compute_gen1.id, "different structure = different hash");

    // Save improved version to cache (this is what --improve does after hot-swap)
    let hex = fragment_cache::save_fragment(&cache_dir, "compute", &compute_gen1).unwrap();
    println!("\n=== Generation 1 (evolved) ===");
    println!("  FragmentId: {}", &hex);
    println!("  Result: {:?}", result_gen1);
    println!("  Saved to cache: gen {}", fragment_cache::generation(&cache_dir, "compute"));

    assert_eq!(fragment_cache::generation(&cache_dir, "compute"), 1);

    // === Generation 2: Next run loads improved version from cache ===
    // Simulate what `iris run` does on startup: compile source, check cache.
    let frags_gen2 = compile_all(pre_src); // Compile the ORIGINAL source again
    let compute_compiled = find(&frags_gen2, "compute");

    // Without cache: would get the original (slow) version
    assert_eq!(compute_compiled.id, compute_gen0.id);

    // With cache: load the improved version
    let compute_cached = fragment_cache::load_fragment(&cache_dir, "compute")
        .expect("cache should return the gen-1 improved version");

    // Verify: cached version has the evolved hash, not the original
    assert_eq!(compute_cached.id, compute_gen1.id,
        "cache should return the improved version");
    assert_ne!(compute_cached.id, compute_gen0.id,
        "cache should NOT return the original");

    // Verify: cached version produces correct results
    let reg_cached: BTreeMap<FragmentId, SemanticGraph> =
        [(compute_cached.id, compute_cached.graph.clone())].into_iter().collect();
    let result_gen2 = eval(&compute_cached, &[Value::Int(500)], &reg_cached);
    assert_eq!(result_gen2, result_gen0, "cached version must produce same result");

    // Verify: BLAKE3 integrity survived the disk round-trip
    let recomputed_id = compute_fragment_id(&compute_cached);
    assert_eq!(recomputed_id, compute_gen1.id, "BLAKE3 integrity check passed");

    println!("\n=== Generation 2 (loaded from cache) ===");
    println!("  Compiled FragmentId: {} (original, would be slow)",
        fragment_cache::fragment_id_hex(&compute_compiled.id));
    println!("  Cached FragmentId:   {} (improved, loaded from disk)",
        fragment_cache::fragment_id_hex(&compute_cached.id));
    println!("  Result: {:?}", result_gen2);
    println!("  BLAKE3 integrity: verified");
    println!("\n  Generational self-improvement works:");
    println!("  Gen 0 -> Gen 1: evolution improved the program");
    println!("  Gen 1 -> Gen 2: improvement persisted across restart");

    // Cleanup
    let _ = std::fs::remove_dir_all(&cache_dir);
}
