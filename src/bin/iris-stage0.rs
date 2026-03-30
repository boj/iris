//! iris-stage0: The frozen bootstrap seed for IRIS.
//!
//! Rust is only the JSON loader + 807-line mini evaluator.
//! Everything else is IRIS: tokenizer, parser, lowerer, interpreter.
//!
//! Commands:
//!   compile <source.iris> [-o output.json]   Compile .iris source to JSON
//!   run <source.iris> [args...]              Compile + evaluate an .iris file
//!   direct <program.json> [args...]          Evaluate a pre-compiled JSON graph
//!   interp <interp.json> <prog.json> [args]  Meta-circular evaluation
//!   rebuild [project_root]                   Regenerate bootstrap JSONs from IRIS source

use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;

use iris_bootstrap::mini_eval;
use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "compile" => cmd_compile(&args[2..]),
        "run" => cmd_run(&args[2..]),
        "direct" => cmd_direct(&args[2..]),
        "interp" => cmd_interp(&args[2..]),
        "rebuild" => cmd_rebuild(&args[2..]),
        "version" | "--version" | "-V" => {
            println!("iris-stage0 0.1.0 (self-hosted, mini_eval)");
        }
        "help" | "--help" | "-h" => print_usage(),
        other => {
            eprintln!("Unknown command: {}", other);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!(
        "\
iris-stage0: IRIS bootstrap seed (self-hosted)

Usage: iris-stage0 <command> [options]

Commands:
  compile <source.iris> [-o output.json]    Compile .iris source to JSON
  run <source.iris> [args...]               Compile and evaluate an .iris file
  direct <program.json> [args...]           Evaluate a pre-compiled JSON graph
  interp <interp.json> <prog.json> [args]   Meta-circular evaluation
  rebuild [project_root]                    Regenerate bootstrap JSONs from IRIS source
  version                                   Print version"
    );
}

// ---------------------------------------------------------------------------
// IRIS runtime: everything is IRIS, Rust only does JSON + mini_eval
// ---------------------------------------------------------------------------

struct IrisRuntime {
    tokenizer: SemanticGraph,
    parser: SemanticGraph,
    lowerer: SemanticGraph,
    interpreter: SemanticGraph,
}

impl IrisRuntime {
    fn load(bootstrap_dir: &PathBuf) -> Self {
        let load = |name: &str| -> SemanticGraph {
            mini_eval::load_graph(bootstrap_dir.join(name).to_str().unwrap())
                .unwrap_or_else(|e| { eprintln!("error loading {}: {}", name, e); process::exit(1); })
        };
        Self {
            tokenizer: load("tokenizer.json"),
            parser: load("parser.json"),
            lowerer: load("lowerer.json"),
            interpreter: load("self_interpreter.json"),
        }
    }

    /// Evaluate a program via the IRIS self-interpreter (mini_eval bootstraps it).
    fn eval_program(&self, graph: &SemanticGraph, inputs: &[Value]) -> Value {
        let mut env_entries: Vec<Value> = Vec::new();
        for (i, val) in inputs.iter().enumerate() {
            env_entries.push(Value::tuple(vec![
                Value::Int(0xFFFF_0000u32 as i64 + i as i64),
                val.clone(),
            ]));
        }
        let env = if env_entries.is_empty() {
            Value::Range(0, 0)
        } else {
            Value::tuple(env_entries)
        };

        let empty_reg = BTreeMap::new();
        mini_eval::evaluate_with_registry(
            &self.interpreter,
            &[
                Value::Program(Rc::new(self.interpreter.clone())),
                Value::Program(Rc::new(graph.clone())),
                env,
            ],
            100_000_000,
            &empty_reg,
        ).unwrap_or_else(|e| { eprintln!("eval error: {}", e); process::exit(1); })
    }

    /// Evaluate a compiler stage via mini_eval directly.
    /// Compiler stages (tokenizer, parser, lowerer) are IRIS programs that
    /// mini_eval can run without the self-interpreter overhead.
    fn eval_direct(&self, graph: &SemanticGraph, inputs: &[Value]) -> Value {
        let empty_reg = BTreeMap::new();
        mini_eval::evaluate_with_registry(graph, inputs, 100_000_000, &empty_reg)
            .unwrap_or_else(|e| { eprintln!("compile stage error: {}", e); process::exit(1); })
    }

    /// Compile IRIS source to a SemanticGraph via the IRIS pipeline.
    fn compile(&self, source: &str) -> SemanticGraph {
        let tokens = self.eval_direct(&self.tokenizer, &[Value::String(source.to_string())]);
        let ast = self.eval_direct(&self.parser, &[tokens, Value::String(source.to_string())]);
        let program = self.eval_direct(&self.lowerer, &[ast, Value::String(source.to_string())]);

        match program {
            Value::Program(g) => Rc::try_unwrap(g).unwrap_or_else(|rc| (*rc).clone()),
            Value::Tuple(ref fields) if !fields.is_empty() => {
                match &fields[0] {
                    Value::Program(g) => g.as_ref().clone(),
                    _ => { eprintln!("error: lowerer returned unexpected value"); process::exit(1); }
                }
            }
            other => { eprintln!("error: lowerer returned {:?}, expected Program", other); process::exit(1); }
        }
    }
}

fn find_bootstrap_dir() -> PathBuf {
    if let Ok(dir) = env::var("IRIS_BOOTSTRAP_DIR") {
        let p = PathBuf::from(dir);
        if p.join("tokenizer.json").exists() { return p; }
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            if parent.join("tokenizer.json").exists() { return parent.to_path_buf(); }
        }
    }
    if let Ok(manifest) = env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(manifest).join("bootstrap");
        if p.join("tokenizer.json").exists() { return p; }
    }
    let p = PathBuf::from("bootstrap");
    if p.join("tokenizer.json").exists() { return p; }
    eprintln!("error: cannot find bootstrap directory");
    process::exit(1);
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

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

fn cmd_compile(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: iris-stage0 compile <source.iris> [-o output.json]");
        process::exit(1);
    }
    let source = std::fs::read_to_string(&args[0])
        .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
    let runtime = IrisRuntime::load(&find_bootstrap_dir());
    let graph = runtime.compile(&source);
    let json = serde_json::to_string(&graph)
        .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
    if args.len() >= 3 && args[1] == "-o" {
        std::fs::write(&args[2], &json)
            .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
        eprintln!("{} -> {} ({} bytes)", args[0], args[2], json.len());
    } else {
        println!("{}", json);
    }
}

fn cmd_run(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: iris-stage0 run <source.iris> [args...]");
        process::exit(1);
    }
    let source = std::fs::read_to_string(&args[0])
        .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
    let runtime = IrisRuntime::load(&find_bootstrap_dir());
    let graph = runtime.compile(&source);
    let result = runtime.eval_program(&graph, &parse_args(&args[1..]));
    println!("{}", format_value(&result));
}

fn cmd_direct(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: iris-stage0 direct <program.json> [args...]");
        process::exit(1);
    }
    let program = mini_eval::load_graph(&args[0])
        .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
    let runtime = IrisRuntime::load(&find_bootstrap_dir());
    let result = runtime.eval_program(&program, &parse_args(&args[1..]));
    println!("{}", format_value(&result));
}

fn cmd_interp(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: iris-stage0 interp <interpreter.json> <program.json> [args...]");
        process::exit(1);
    }
    let interpreter = mini_eval::load_graph(&args[0])
        .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
    let target = mini_eval::load_graph(&args[1])
        .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
    let inputs = parse_args(&args[2..]);

    // Run interpreter on target with inputs
    let interp_inputs = vec![
        Value::Program(Rc::new(target)),
        Value::tuple(inputs),
    ];
    let empty_reg = BTreeMap::new();
    let result = mini_eval::evaluate_with_registry(&interpreter, &interp_inputs, 100_000_000, &empty_reg)
        .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
    println!("{}", format_value(&result));
}

fn cmd_rebuild(args: &[String]) {
    let root = if let Some(root_arg) = args.first() {
        PathBuf::from(root_arg)
    } else {
        env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| env::current_dir().expect("cannot get current directory"))
    };
    let programs = root.join("src/iris-programs");
    let bootstrap_dir = root.join("bootstrap");
    let runtime = IrisRuntime::load(&bootstrap_dir);

    let artifacts = [
        ("syntax/tokenizer.iris", "tokenizer.json"),
        ("syntax/iris_parser.iris", "parser.json"),
        ("syntax/iris_lowerer.iris", "lowerer.json"),
        ("interpreter/full_interpreter.iris", "interpreter.json"),
    ];

    eprintln!("=== Rebuilding bootstrap JSON from IRIS source ===\n");
    for (src_rel, out_name) in artifacts {
        let source = std::fs::read_to_string(programs.join(src_rel))
            .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
        let graph = runtime.compile(&source);
        let json = serde_json::to_string(&graph)
            .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
        std::fs::write(bootstrap_dir.join(out_name), &json)
            .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
        eprintln!("  {} -> {} ({} bytes)", src_rel, out_name, json.len());
    }
    eprintln!("\n=== Done ===");
}

fn format_value(v: &Value) -> String {
    match v {
        Value::Int(n) => n.to_string(),
        Value::Float64(f) => format!("{}", f),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        Value::Unit => "()".to_string(),
        Value::Tuple(elems) => {
            let inner: Vec<String> = elems.iter().map(|e| format_value(e)).collect();
            format!("({})", inner.join(", "))
        }
        other => format!("{:?}", other),
    }
}
