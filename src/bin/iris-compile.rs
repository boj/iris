//! iris-compile: Compile .iris source to a JSON SemanticGraph.
//!
//! Usage:
//!   iris-compile <source.iris> [-o output.json]

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: iris-compile <source.iris> [-o output.json]");
        std::process::exit(1);
    }

    let source_path = &args[1];
    let source = fs::read_to_string(source_path)
        .unwrap_or_else(|e| { eprintln!("error: cannot read '{}': {}", source_path, e); std::process::exit(1); });

    let result = iris_bootstrap::syntax::compile(&source);

    if !result.errors.is_empty() {
        for e in &result.errors {
            eprintln!("error: {:?}", e);
        }
        std::process::exit(1);
    }

    if result.fragments.is_empty() {
        eprintln!("error: compilation produced no fragments");
        std::process::exit(1);
    }

    // Take the last fragment's graph (the main entry point).
    // Earlier fragments are helpers that get referenced via Ref nodes.
    let graph = &result.fragments.last().unwrap().1.graph;
    let json = serde_json::to_string(graph)
        .unwrap_or_else(|e| { eprintln!("error: serialization failed: {}", e); std::process::exit(1); });

    if args.len() >= 4 && args[2] == "-o" {
        fs::write(&args[3], &json)
            .unwrap_or_else(|e| { eprintln!("error: cannot write '{}': {}", args[3], e); std::process::exit(1); });
        eprintln!("Compiled {} -> {} ({} bytes)", source_path, args[3], json.len());
    } else {
        println!("{}", json);
    }
}
