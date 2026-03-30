//! iris-bootstrap: Minimal binary for running IRIS programs through the
//! meta-circular interpreter.
//!
//! Usage:
//!   iris-bootstrap run <interpreter.json> <program.json> [args...]
//!   iris-bootstrap direct <program.json> [args...]
//!   iris-bootstrap test [project_root]
//!
//! The `run` command loads the compiled IRIS interpreter and uses it to
//! evaluate the target program -- this is the full bootstrap chain.
//!
//! The `direct` command evaluates the program directly with the bootstrap
//! evaluator (no meta-circular layer).
//!
//! The `test` command runs the IRIS self-hosted test suite
//! (tests/fixtures/iris-testing/run_all_tests.iris). Zero Rust test files.

use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;
use std::process;

use iris_bootstrap::{bootstrap_eval, load_graph};
use iris_types::eval::Value;
use iris_types::fragment::FragmentId;
use iris_types::graph::SemanticGraph;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "run" => cmd_run(&args[2..]),
        "direct" => cmd_direct(&args[2..]),
        "test" => cmd_test(&args[2..]),
        "help" | "--help" | "-h" => print_usage(),
        other => {
            eprintln!("Unknown command: {}", other);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  iris-bootstrap run <interpreter.json> <program.json> [int_args...]");
    eprintln!("  iris-bootstrap direct <program.json> [int_args...]");
    eprintln!("  iris-bootstrap test [project_root]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  run     Load IRIS interpreter, run target program through it");
    eprintln!("  direct  Evaluate program directly with bootstrap evaluator");
    eprintln!("  test    Run IRIS self-hosted test suite (tests/fixtures/iris-testing/run_all_tests.iris)");
}

fn parse_args(args: &[String]) -> Vec<Value> {
    args.iter()
        .map(|s| {
            if let Ok(n) = s.parse::<i64>() {
                Value::Int(n)
            } else if let Ok(f) = s.parse::<f64>() {
                Value::Float64(f)
            } else if s == "true" {
                Value::Bool(true)
            } else if s == "false" {
                Value::Bool(false)
            } else if s == "()" {
                Value::Unit
            } else {
                Value::String(s.clone())
            }
        })
        .collect()
}

fn cmd_run(args: &[String]) {
    if args.len() < 2 {
        eprintln!("run: expected <interpreter.json> <program.json> [args...]");
        process::exit(1);
    }

    let interpreter = match load_graph(&args[0]) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Failed to load interpreter: {}", e);
            process::exit(1);
        }
    };

    let target = match load_graph(&args[1]) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Failed to load target program: {}", e);
            process::exit(1);
        }
    };

    let inputs = parse_args(&args[2..]);

    match bootstrap_eval(&interpreter, &target, &inputs) {
        Ok(result) => println!("{:?}", result),
        Err(e) => {
            eprintln!("Execution error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_direct(args: &[String]) {
    if args.is_empty() {
        eprintln!("direct: expected <program.json> [args...]");
        process::exit(1);
    }

    let program = match load_graph(&args[0]) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Failed to load program: {}", e);
            process::exit(1);
        }
    };

    let inputs = parse_args(&args[1..]);

    match iris_bootstrap::evaluate(&program, &inputs) {
        Ok(result) => println!("{:?}", result),
        Err(e) => {
            eprintln!("Execution error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_test(args: &[String]) {
    // Determine project root.
    let root = if let Some(root_arg) = args.first() {
        PathBuf::from(root_arg)
    } else {
        // Default: detect from CARGO_MANIFEST_DIR or current directory.
        env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| env::current_dir().expect("cannot get current directory"))
    };

    let root_str = root.to_str().expect("project root is not valid UTF-8");

    // Read and compile the test runner.
    let runner_path = root.join("tests/fixtures/iris-testing/run_all_tests.iris");
    let runner_src = match std::fs::read_to_string(&runner_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read test runner: {}", e);
            process::exit(1);
        }
    };

    // Compile all bindings in the runner. Build a fragment registry so that
    // cross-binding Ref nodes resolve correctly.
    let compile_result = iris_bootstrap::syntax::compile(&runner_src);
    if !compile_result.errors.is_empty() {
        for err in &compile_result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(&runner_src, err));
        }
        eprintln!("Failed to compile test runner");
        process::exit(1);
    }

    let mut registry: BTreeMap<FragmentId, SemanticGraph> = BTreeMap::new();
    let mut run_all_graph: Option<SemanticGraph> = None;

    for (name, frag, _) in &compile_result.fragments {
        registry.insert(frag.id, frag.graph.clone());
        if name == "run_all" {
            run_all_graph = Some(frag.graph.clone());
        }
    }

    let runner = run_all_graph.unwrap_or_else(|| {
        eprintln!("run_all_tests.iris must define 'run_all'");
        process::exit(1);
    });

    eprintln!("=== IRIS Self-Hosted Test Suite ===\n");

    // Evaluate: run_all takes project root path as input.
    // Use a generous step limit -- test suite evaluates hundreds of programs.
    // module_eval sub-evaluations have their own step budgets, so the outer
    // runner only needs steps for control flow + file I/O + compilation.
    let max_steps = 100_000_000;
    let result = iris_bootstrap::evaluate_with_registry(
        &runner,
        &[Value::String(root_str.to_string())],
        max_steps,
        &registry,
    );

    match result {
        Ok(Value::Tuple(ref file_results)) => {
            let mut total_passed: i64 = 0;
            let mut total_failed: i64 = 0;

            for entry in file_results.iter() {
                if let Value::Tuple(fields) = entry {
                    if fields.len() >= 3 {
                        let path = match &fields[0] {
                            Value::String(s) => s.as_str(),
                            _ => "???",
                        };
                        let passed = match &fields[1] {
                            Value::Int(n) => *n,
                            _ => 0,
                        };
                        let failed = match &fields[2] {
                            Value::Int(n) => *n,
                            _ => 0,
                        };

                        eprintln!("  {}: {} passed, {} failed", path, passed, failed);

                        // If there were failures, re-evaluate individual tests to identify them.
                        if failed > 0 {
                            identify_failures(root_str, path);
                        }

                        total_passed += passed;
                        total_failed += failed;
                    }
                }
            }

            eprintln!(
                "\n=== IRIS Self-Test Summary ===\n  Total: {} passed, {} failed, {} tests",
                total_passed, total_failed, total_passed + total_failed,
            );

            if total_failed > 0 {
                process::exit(1);
            }
            if total_passed == 0 {
                eprintln!("No tests were found or passed");
                process::exit(1);
            }
        }
        Ok(other) => {
            eprintln!("Test runner returned unexpected value: {:?}", other);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Test runner failed: {}", e);
            process::exit(1);
        }
    }
}

/// When a test file has failures, re-evaluate individual test bindings to identify which ones fail.
fn identify_failures(root_str: &str, test_path: &str) {
    let root = PathBuf::from(root_str);
    let harness_path = root.join("tests/fixtures/iris-testing/test_harness.iris");
    let test_file_path = root.join(test_path);

    let harness = match std::fs::read_to_string(&harness_path) {
        Ok(s) => s,
        Err(_) => return,
    };
    let test_src = match std::fs::read_to_string(&test_file_path) {
        Ok(s) => s,
        Err(_) => return,
    };

    // Parse dependencies.
    let mut combined = harness;
    for line in test_src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("-- depends:") {
            let rest = trimmed.strip_prefix("-- depends:").unwrap().trim();
            for dep in rest.split(',') {
                let dep = dep.trim();
                if !dep.is_empty() {
                    if let Ok(dep_src) = std::fs::read_to_string(root.join(dep)) {
                        combined.push('\n');
                        combined.push_str(&dep_src);
                    }
                }
            }
        } else if !trimmed.starts_with("//") && !trimmed.starts_with("--") && !trimmed.is_empty() {
            break;
        }
    }
    combined.push('\n');
    combined.push_str(&test_src);

    let compile_result = iris_bootstrap::syntax::compile(&combined);
    if !compile_result.errors.is_empty() {
        return;
    }

    let mut registry: BTreeMap<FragmentId, SemanticGraph> = BTreeMap::new();
    for (_, frag, _) in &compile_result.fragments {
        registry.insert(frag.id, frag.graph.clone());
    }

    for (name, frag, _) in &compile_result.fragments {
        if name.starts_with("test_") && frag.boundary.inputs.is_empty() {
            match iris_bootstrap::evaluate_with_registry(
                &frag.graph, &[], 1_000_000, &registry,
            ) {
                Ok(Value::Int(n)) if n > 0 => {}
                Ok(v) => eprintln!("    FAIL: {} => {:?}", name, v),
                Err(e) => eprintln!("    FAIL: {} => {}", name, e),
            }
        }
    }
}
