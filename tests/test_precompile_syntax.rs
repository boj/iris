
//! Pre-compile the tokenizer, parser, and lowerer IRIS programs to JSON SemanticGraphs.
//!
//! These JSON files live in bootstrap/ and are used by the non-scaffolding CLI
//! to run IRIS programs without depending on iris-syntax at runtime.

use std::fs;

fn compile_and_save(source_path: &str, output_path: &str) {
    let source = fs::read_to_string(source_path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {}", source_path, e));

    let result = iris_bootstrap::syntax::compile(&source);
    if !result.errors.is_empty() {
        for e in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(&source, e));
        }
        panic!("{} had {} compile errors", source_path, result.errors.len());
    }
    assert!(
        !result.fragments.is_empty(),
        "No fragments from {}",
        source_path
    );

    // The entry-point function is the last fragment
    let graph = &result.fragments.last().unwrap().1.graph;
    iris_bootstrap::save_graph(graph, output_path)
        .unwrap_or_else(|e| panic!("Cannot save {}: {}", output_path, e));

    // Verify round-trip
    let loaded = iris_bootstrap::load_graph(output_path)
        .unwrap_or_else(|e| panic!("Cannot reload {}: {}", output_path, e));
    assert_eq!(loaded.root, graph.root);
    assert_eq!(loaded.nodes.len(), graph.nodes.len());
}

#[test]
fn precompile_tokenizer_json() {
    let src = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/iris-programs/syntax/tokenizer.iris"
    );
    let out = concat!(env!("CARGO_MANIFEST_DIR"), "/bootstrap/tokenizer.json");
    compile_and_save(src, out);
}

#[test]
fn precompile_parser_json() {
    let src = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/iris-programs/syntax/iris_parser.iris"
    );
    let out = concat!(env!("CARGO_MANIFEST_DIR"), "/bootstrap/parser.json");
    compile_and_save(src, out);
}

#[test]
fn precompile_lowerer_json() {
    let src = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/iris-programs/syntax/iris_lowerer.iris"
    );
    let out = concat!(env!("CARGO_MANIFEST_DIR"), "/bootstrap/lowerer.json");
    compile_and_save(src, out);
}
