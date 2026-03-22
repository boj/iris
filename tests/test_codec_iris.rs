
//! Test harness for iris-codec .iris programs.
//!
//! Loads each codec .iris file, compiles it via iris_bootstrap::syntax::compile(),
//! registers all fragments in a FragmentRegistry, then evaluates every
//! `test_` binding through the Rust interpreter with the registry.
//! Asserts that each test returns a truthy (positive) value.

use iris_exec::interpreter;
use iris_exec::registry::FragmentRegistry;
use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compile IRIS source, register all fragments, return named graphs + registry.
fn compile_with_registry(src: &str) -> (Vec<(String, SemanticGraph)>, FragmentRegistry) {
    let result = iris_bootstrap::syntax::compile(src);
    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(src, err));
        }
        panic!(
            "compilation failed with {} errors:\n{}",
            result.errors.len(),
            result.errors.iter().map(|e| iris_bootstrap::syntax::format_error(src, e)).collect::<Vec<_>>().join("\n")
        );
    }

    let mut registry = FragmentRegistry::new();
    for (_, frag, _) in &result.fragments {
        registry.register(frag.clone());
    }

    let named: Vec<_> = result
        .fragments
        .into_iter()
        .map(|(name, frag, _)| (name, frag.graph))
        .collect();

    (named, registry)
}

/// Evaluate a SemanticGraph with no inputs and return the result as i64.
fn eval_no_args(graph: &SemanticGraph, registry: &FragmentRegistry) -> Result<i64, String> {
    match interpreter::interpret_with_registry(graph, &[], None, Some(registry)) {
        Ok((outputs, _)) => {
            if let Some(val) = outputs.first() {
                match val {
                    Value::Int(n) => Ok(*n),
                    Value::Bool(true) => Ok(1),
                    Value::Bool(false) => Ok(0),
                    _ => Err(format!("unexpected result type: {:?}", val)),
                }
            } else {
                Err("no output".to_string())
            }
        }
        Err(e) => Err(format!("evaluation error: {:?}", e)),
    }
}

/// Load a codec .iris file, compile it, run all test_ bindings, and
/// assert each one returns a positive value.
fn run_codec_tests(filename: &str) {
    let path = format!("src/iris-programs/codec/{}", filename);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path, e));
    let (fragments, registry) = compile_with_registry(&source);

    let test_frags: Vec<_> = fragments
        .iter()
        .filter(|(name, _)| name.starts_with("test_"))
        .collect();

    assert!(
        !test_frags.is_empty(),
        "{}: no test_ bindings found",
        filename
    );

    let mut passed = 0;
    let mut failed = Vec::new();

    for (name, graph) in &test_frags {
        match eval_no_args(graph, &registry) {
            Ok(n) if n > 0 => {
                passed += 1;
            }
            Ok(n) => {
                failed.push(format!("  {} returned {} (expected > 0)", name, n));
            }
            Err(e) => {
                failed.push(format!("  {} error: {}", name, e));
            }
        }
    }

    if !failed.is_empty() {
        panic!(
            "{}: {}/{} tests failed:\n{}",
            filename,
            failed.len(),
            test_frags.len(),
            failed.join("\n")
        );
    }

    println!(
        "{}: all {}/{} tests passed",
        filename, passed, test_frags.len()
    );
}

// ---------------------------------------------------------------------------
// Tests for each codec .iris file
// ---------------------------------------------------------------------------

#[test]
fn test_embedding_iris() {
    run_codec_tests("embedding.iris");
}

#[test]
fn test_crossover_iris() {
    run_codec_tests("crossover.iris");
}

#[test]
fn test_neural_codec_iris() {
    run_codec_tests("neural_codec.iris");
}

#[test]
fn test_training_data_iris() {
    run_codec_tests("training_data.iris");
}

#[test]
fn test_cosine_similarity_iris() {
    run_codec_tests("cosine_similarity.iris");
}

#[test]
fn test_extract_features_iris() {
    run_codec_tests("extract_features.iris");
}

#[test]
fn test_node_histogram_iris() {
    run_codec_tests("node_histogram.iris");
}

#[test]
fn test_hnsw_index_iris() {
    run_codec_tests("hnsw_index.iris");
}

#[test]
fn test_gin_encoder_iris() {
    run_codec_tests("gin_encoder.iris");
}

#[test]
fn test_graph_decoder_iris() {
    run_codec_tests("graph_decoder.iris");
}

#[test]
fn test_feature_codec_iris() {
    run_codec_tests("feature_codec.iris");
}

#[test]
fn test_structural_repair_iris() {
    run_codec_tests("structural_repair.iris");
}

#[test]
fn test_repository_iris() {
    run_codec_tests("repository.iris");
}
