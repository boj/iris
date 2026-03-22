/// Content-Addressed Evolution Lifecycle
///
/// Demonstrates the full IRIS lifecycle in 6 stages:
///   1. Compile  -> .iris source to Fragment with BLAKE3 FragmentId
///   2. Content-address -> deterministic hashing, structural sensitivity
///   3. Benchmark -> pre-evolution vs post-evolution speedup
///   4. Serialize -> wire format to disk, files named by hash
///   5. Load     -> deserialize, verify hash integrity
///   6. Hot-swap -> replace fragment by hash, evaluate improved version

use std::collections::BTreeMap;
use std::time::Instant;

use iris_bootstrap::syntax;
use iris_types::eval::Value;
use iris_types::fragment::{Fragment, FragmentId};
use iris_types::graph::SemanticGraph;
use iris_types::hash::{compute_fragment_id, compute_node_id};
use iris_types::wire;

const LIFECYCLE_SRC: &str = include_str!("../examples/lifecycle/content-addressed-evolution.iris");

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn compile_program(src: &str) -> Vec<(String, Fragment)> {
    let result = syntax::compile(src);
    assert!(result.errors.is_empty(), "compile errors: {:?}", result.errors);
    result.fragments.into_iter().map(|(name, frag, _)| (name, frag)).collect()
}

fn find_fragment(frags: &[(String, Fragment)], name: &str) -> Fragment {
    frags.iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("fragment '{}' not found", name))
        .1.clone()
}

fn build_registry(frags: &[(String, Fragment)]) -> BTreeMap<FragmentId, SemanticGraph> {
    frags.iter().map(|(_, f)| (f.id, f.graph.clone())).collect()
}

fn eval_frag(
    frag: &Fragment,
    args: &[Value],
    registry: &BTreeMap<FragmentId, SemanticGraph>,
) -> Value {
    iris_bootstrap::evaluate_with_fragments(&frag.graph, args, 50_000_000, registry)
        .expect("evaluation failed")
}

fn fragment_id_hex(id: &FragmentId) -> String {
    id.0.iter().map(|b| format!("{:02x}", b)).collect()
}

fn bench_ns(frag: &Fragment, args: &[Value], registry: &BTreeMap<FragmentId, SemanticGraph>) -> f64 {
    let warmup = 3;
    let iters = 10;
    for _ in 0..warmup { let _ = eval_frag(frag, args, registry); }
    let start = Instant::now();
    for _ in 0..iters { let _ = eval_frag(frag, args, registry); }
    start.elapsed().as_nanos() as f64 / iters as f64
}

// ---------------------------------------------------------------------------
// Stage 1: Compile
// ---------------------------------------------------------------------------

#[test]
fn stage1_compile_and_hash() {
    let frags = compile_program(LIFECYCLE_SRC);
    let pre_int = find_fragment(&frags, "hash_pre");
    let post_int = find_fragment(&frags, "hash_post");
    let pre_float = find_fragment(&frags, "ema_pre");
    let post_float = find_fragment(&frags, "ema_post");

    println!("\n=== Stage 1: Compile & Hash ===");
    println!("  hash_pre  FragmentId: {}  ({} nodes)", fragment_id_hex(&pre_int.id), pre_int.graph.nodes.len());
    println!("  hash_post FragmentId: {}  ({} nodes)", fragment_id_hex(&post_int.id), post_int.graph.nodes.len());
    println!("  ema_pre   FragmentId: {}  ({} nodes)", fragment_id_hex(&pre_float.id), pre_float.graph.nodes.len());
    println!("  ema_post  FragmentId: {}  ({} nodes)", fragment_id_hex(&post_float.id), post_float.graph.nodes.len());

    // Different structure -> different FragmentId
    assert_ne!(pre_int.id, post_int.id);
    assert_ne!(pre_float.id, post_float.id);
    // Pre-evolution has more nodes (extra state tracking)
    assert!(pre_int.graph.nodes.len() > post_int.graph.nodes.len(),
        "pre-evolution should have more nodes than post-evolution");
    assert!(pre_float.graph.nodes.len() > post_float.graph.nodes.len(),
        "pre-evolution should have more nodes than post-evolution");
}

// ---------------------------------------------------------------------------
// Stage 2: Content Addressing
// ---------------------------------------------------------------------------

#[test]
fn stage2_deterministic_hashing() {
    let frags1 = compile_program(LIFECYCLE_SRC);
    let frags2 = compile_program(LIFECYCLE_SRC);

    let pre1 = find_fragment(&frags1, "hash_pre");
    let pre2 = find_fragment(&frags2, "hash_pre");

    println!("\n=== Stage 2: Content Addressing ===");
    println!("  Compile #1: {}", fragment_id_hex(&pre1.id));
    println!("  Compile #2: {}", fragment_id_hex(&pre2.id));

    assert_eq!(pre1.id, pre2.id,
        "same source must produce same BLAKE3 FragmentId (idempotent)");

    let recomputed = compute_fragment_id(&pre1);
    println!("  Recomputed:  {}", fragment_id_hex(&recomputed));
    assert_eq!(pre1.id, recomputed);
}

#[test]
fn stage2_node_level_hashing() {
    let frags = compile_program(LIFECYCLE_SRC);
    let post = find_fragment(&frags, "hash_post");

    println!("\n=== Stage 2: Node-Level BLAKE3 ===");
    let mut shown = 0;
    for (id, node) in &post.graph.nodes {
        let computed = compute_node_id(node);
        if shown < 5 {
            println!("  Node {:?}: kind={:?}, computed_id={:?}",
                id, node.kind, computed);
            shown += 1;
        }
    }
    println!("  ... ({} nodes total)", post.graph.nodes.len());
}

// ---------------------------------------------------------------------------
// Stage 3: Benchmark (pre vs post evolution)
// ---------------------------------------------------------------------------

#[test]
fn stage3_benchmark() {
    let frags = compile_program(LIFECYCLE_SRC);
    let pre_int = find_fragment(&frags, "hash_pre");
    let post_int = find_fragment(&frags, "hash_post");
    let pre_float = find_fragment(&frags, "ema_pre");
    let post_float = find_fragment(&frags, "ema_post");
    let registry = build_registry(&frags);
    let n = 50_000i64;
    let args = [Value::Int(n)];

    // Correctness: pre and post produce the same (hash, count) result
    let result_pre = eval_frag(&pre_int, &args, &registry);
    let result_post = eval_frag(&post_int, &args, &registry);
    assert_eq!(result_pre, result_post,
        "pre and post evolution must produce the same result");

    // Float correctness: compare EMA values (allow small float tolerance)
    let result_pre_f = eval_frag(&pre_float, &args, &registry);
    let result_post_f = eval_frag(&post_float, &args, &registry);
    if let (Value::Tuple(a), Value::Tuple(b)) = (&result_pre_f, &result_post_f) {
        if let (Value::Float64(ema_a), Value::Float64(ema_b)) = (&a[0], &b[0]) {
            assert!((ema_a - ema_b).abs() < 1e-6,
                "EMA values should match: {} vs {}", ema_a, ema_b);
        }
    }

    // Benchmark all four
    let pre_int_ns = bench_ns(&pre_int, &args, &registry);
    let post_int_ns = bench_ns(&post_int, &args, &registry);
    let pre_float_ns = bench_ns(&pre_float, &args, &registry);
    let post_float_ns = bench_ns(&post_float, &args, &registry);

    let int_speedup = pre_int_ns / post_int_ns;
    let float_speedup = pre_float_ns / post_float_ns;

    println!("\n=== Stage 3: Pre-Evolution vs Post-Evolution (N={}) ===", n);
    println!();
    println!("  Integer fold (GP native codegen):");
    println!("    Pre-evolution:  {:>7.1} ns/step  ({:.2} ms)  [4-elem state, extra ops]",
        pre_int_ns / n as f64, pre_int_ns / 1_000_000.0);
    println!("    Post-evolution: {:>7.1} ns/step  ({:.2} ms)  [2-elem state, minimal ops]",
        post_int_ns / n as f64, post_int_ns / 1_000_000.0);
    println!("    Speedup: {:.2}x", int_speedup);
    println!();
    println!("  Float64 fold (AVX native codegen):");
    println!("    Pre-evolution:  {:>7.1} ns/step  ({:.2} ms)  [6-elem state, statistics]",
        pre_float_ns / n as f64, pre_float_ns / 1_000_000.0);
    println!("    Post-evolution: {:>7.1} ns/step  ({:.2} ms)  [2-elem state, just EMA]",
        post_float_ns / n as f64, post_float_ns / 1_000_000.0);
    println!("    Speedup: {:.2}x", float_speedup);
}

// ---------------------------------------------------------------------------
// Stage 4: Serialize & Save
// ---------------------------------------------------------------------------

#[test]
fn stage4_serialize_and_save() {
    let frags = compile_program(LIFECYCLE_SRC);
    let pre_int = find_fragment(&frags, "hash_pre");
    let post_int = find_fragment(&frags, "hash_post");

    let dir = std::env::temp_dir().join("iris-lifecycle-test");
    let _ = std::fs::create_dir_all(&dir);

    let save = |frag: &Fragment, label: &str| -> std::path::PathBuf {
        let bytes = wire::serialize_fragment(frag);
        let hex = fragment_id_hex(&frag.id);
        let path = dir.join(format!("{}.frag", &hex[..16]));
        std::fs::write(&path, &bytes).expect("write failed");
        println!("  {} {} ({} bytes) -> {}",
            label, &hex[..16], bytes.len(), path.display());
        path
    };

    println!("\n=== Stage 4: Serialize & Save ===");
    println!("  Store directory: {}", dir.display());
    let path_pre = save(&pre_int, "pre-evolution ");
    let path_post = save(&post_int, "post-evolution");

    assert!(path_pre.exists());
    assert!(path_post.exists());
    assert_ne!(
        std::fs::read(&path_pre).unwrap(),
        std::fs::read(&path_post).unwrap(),
        "different fragments must produce different wire bytes"
    );

    let pre_size = std::fs::metadata(&path_pre).unwrap().len();
    let post_size = std::fs::metadata(&path_post).unwrap().len();
    println!("  Post-evolution is {} bytes smaller ({:.0}% reduction)",
        pre_size as i64 - post_size as i64,
        (1.0 - post_size as f64 / pre_size as f64) * 100.0);

    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Stage 5: Load & Verify
// ---------------------------------------------------------------------------

#[test]
fn stage5_load_and_verify() {
    let frags = compile_program(LIFECYCLE_SRC);
    let pre_int = find_fragment(&frags, "hash_pre");
    let post_int = find_fragment(&frags, "hash_post");
    let registry = build_registry(&frags);

    let pre_bytes = wire::serialize_fragment(&pre_int);
    let post_bytes = wire::serialize_fragment(&post_int);

    let pre_loaded = wire::deserialize_fragment(&pre_bytes)
        .expect("deserialize pre failed");
    let post_loaded = wire::deserialize_fragment(&post_bytes)
        .expect("deserialize post failed");

    let pre_hash_loaded = compute_fragment_id(&pre_loaded);
    let post_hash_loaded = compute_fragment_id(&post_loaded);

    println!("\n=== Stage 5: Load & Verify ===");
    println!("  Pre original:  {}", fragment_id_hex(&pre_int.id));
    println!("  Pre loaded:    {}", fragment_id_hex(&pre_hash_loaded));
    println!("  Post original: {}", fragment_id_hex(&post_int.id));
    println!("  Post loaded:   {}", fragment_id_hex(&post_hash_loaded));

    assert_eq!(pre_int.id, pre_hash_loaded,
        "BLAKE3 hash must survive serialize/deserialize round-trip");
    assert_eq!(post_int.id, post_hash_loaded,
        "BLAKE3 hash must survive serialize/deserialize round-trip");

    // Evaluate loaded version -> same result
    let result_orig = eval_frag(&post_int, &[Value::Int(100)], &registry);
    let loaded_registry: BTreeMap<FragmentId, SemanticGraph> =
        [(post_loaded.id, post_loaded.graph.clone())].into_iter().collect();
    let result_loaded = eval_frag(&post_loaded, &[Value::Int(100)], &loaded_registry);

    println!("  hash_post(100) original: {:?}", result_orig);
    println!("  hash_post(100) loaded:   {:?}", result_loaded);
    assert_eq!(result_orig, result_loaded);
}

// ---------------------------------------------------------------------------
// Stage 6: Hot-Swap
// ---------------------------------------------------------------------------

#[test]
fn stage6_hot_swap() {
    let frags = compile_program(LIFECYCLE_SRC);
    let pre_int = find_fragment(&frags, "hash_pre");
    let post_int = find_fragment(&frags, "hash_post");
    let registry = build_registry(&frags);
    let args = [Value::Int(500)];

    // Start with pre-evolution version
    let mut live: BTreeMap<String, (FragmentId, SemanticGraph)> = BTreeMap::new();
    live.insert("hasher".into(), (pre_int.id, pre_int.graph.clone()));

    let (active_id, active_graph) = live.get("hasher").unwrap();
    let result_v1 = iris_bootstrap::evaluate_with_fragments(
        active_graph, &args, 50_000_000, &registry,
    ).unwrap();

    println!("\n=== Stage 6: Hot-Swap ===");
    println!("  v1 (pre-evolution)  FragmentId: {}...", &fragment_id_hex(active_id)[..16]);
    println!("  v1 result: {:?}", result_v1);

    // Hot-swap: evolution produced a better version
    live.insert("hasher".into(), (post_int.id, post_int.graph.clone()));

    let (active_id, active_graph) = live.get("hasher").unwrap();
    let result_v2 = iris_bootstrap::evaluate_with_fragments(
        active_graph, &args, 50_000_000, &registry,
    ).unwrap();

    println!("  v2 (post-evolution) FragmentId: {}...", &fragment_id_hex(active_id)[..16]);
    println!("  v2 result: {:?}", result_v2);

    assert_eq!(result_v1, result_v2,
        "hot-swapped version must produce the same result");
    assert_ne!(pre_int.id, post_int.id,
        "different implementations have different BLAKE3 hashes");

    println!("  Hot-swap successful: same result, different hash, fewer ops");
}
