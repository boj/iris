//! iris-stage0-native: Standalone native IRIS bootstrap binary.
//!
//! This binary embeds all bootstrap JSON artifacts at compile time.
//! No external files needed at runtime. Compiled once from Rust,
//! then IRIS bootstraps itself from here forever.
//!
//! The Rust code is the frozen seed — like GCC's initial C compiler.
//! Everything it runs is IRIS: tokenizer, parser, lowerer, self-interpreter.

use std::collections::BTreeMap;
use std::env;
use std::process;
use std::rc::Rc;

use iris_bootstrap::mini_eval;
use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;

// Embed bootstrap JSON at compile time — no file I/O needed at runtime
const TOKENIZER_JSON: &str = include_str!("../../bootstrap/tokenizer.json");
const PARSER_JSON: &str = include_str!("../../bootstrap/parser.json");
const LOWERER_JSON: &str = include_str!("../../bootstrap/lowerer.json");
const SELF_INTERP_JSON: &str = include_str!("../../bootstrap/self_interpreter.json");

fn load_embedded(json: &str) -> SemanticGraph {
    serde_json::from_str(json).expect("embedded JSON parse error")
}

struct IrisRuntime {
    tokenizer: SemanticGraph,
    parser: SemanticGraph,
    lowerer: SemanticGraph,
    interpreter: SemanticGraph,
}

impl IrisRuntime {
    fn new() -> Self {
        Self {
            tokenizer: load_embedded(TOKENIZER_JSON),
            parser: load_embedded(PARSER_JSON),
            lowerer: load_embedded(LOWERER_JSON),
            interpreter: load_embedded(SELF_INTERP_JSON),
        }
    }

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

    fn eval_direct(&self, graph: &SemanticGraph, inputs: &[Value]) -> Value {
        let empty_reg = BTreeMap::new();
        mini_eval::evaluate_with_registry(graph, inputs, 100_000_000, &empty_reg)
            .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); })
    }

    fn compile(&self, source: &str) -> SemanticGraph {
        // Use Rust syntax module for full compilation (handles lambdas, match, etc.)
        #[cfg(feature = "syntax")]
        {
            let result = iris_bootstrap::syntax::compile(source);
            if !result.errors.is_empty() {
                for err in &result.errors {
                    eprintln!("{}", iris_bootstrap::syntax::format_error(source, err));
                }
                process::exit(1);
            }
            return result.fragments.last().unwrap().1.graph.clone();
        }
        // Fallback: IRIS pipeline (handles basic expressions)
        #[cfg(not(feature = "syntax"))]
        {
            let tokens = self.eval_direct(&self.tokenizer, &[Value::String(source.to_string())]);
            let ast = self.eval_direct(&self.parser, &[tokens, Value::String(source.to_string())]);
            let program = self.eval_direct(&self.lowerer, &[ast, Value::String(source.to_string())]);
            match program {
                Value::Program(g) => Rc::try_unwrap(g).unwrap_or_else(|rc| (*rc).clone()),
                Value::Tuple(ref fields) if !fields.is_empty() => match &fields[0] {
                    Value::Program(g) => g.as_ref().clone(),
                    _ => { eprintln!("lowerer error"); process::exit(1); }
                },
                _ => { eprintln!("lowerer error"); process::exit(1); }
            }
        }
    }
}

fn parse_args(args: &[String]) -> Vec<Value> {
    args.iter().map(|s| {
        if let Ok(n) = s.parse::<i64>() { Value::Int(n) }
        else if let Ok(f) = s.parse::<f64>() { Value::Float64(f) }
        else if s == "true" { Value::Bool(true) }
        else if s == "false" { Value::Bool(false) }
        else if s == "()" { Value::Unit }
        else { Value::String(s.clone()) }
    }).collect()
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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("iris-stage0 (native, self-hosted)\n");
        eprintln!("Usage: iris-stage0 <command> [options]\n");
        eprintln!("Commands:");
        eprintln!("  compile <source.iris> [-o output.json]");
        eprintln!("  run <source.iris> [args...]");
        eprintln!("  direct <program.json> [args...]");
        eprintln!("  version");
        process::exit(1);
    }

    let runtime = IrisRuntime::new();

    match args[1].as_str() {
        "compile" => {
            if args.len() < 3 { eprintln!("Usage: iris-stage0 compile <source.iris> [-o output.json]"); process::exit(1); }
            let source = std::fs::read_to_string(&args[2])
                .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
            let graph = runtime.compile(&source);
            let json = serde_json::to_string(&graph).unwrap();
            if args.len() >= 5 && args[3] == "-o" {
                std::fs::write(&args[4], &json).unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
                eprintln!("{} -> {} ({} bytes)", args[2], args[4], json.len());
            } else {
                println!("{}", json);
            }
        }
        "run" => {
            if args.len() < 3 { eprintln!("Usage: iris-stage0 run <source.iris> [args...]"); process::exit(1); }
            let source = std::fs::read_to_string(&args[2])
                .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
            let graph = runtime.compile(&source);
            let result = runtime.eval_program(&graph, &parse_args(&args[3..]));
            println!("{}", format_value(&result));
        }
        "direct" => {
            if args.len() < 3 { eprintln!("Usage: iris-stage0 direct <program.json> [args...]"); process::exit(1); }
            let program = mini_eval::load_graph(&args[2])
                .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
            let result = runtime.eval_program(&program, &parse_args(&args[3..]));
            println!("{}", format_value(&result));
        }
        "help" | "--help" | "-h" => {
            eprintln!("iris-stage0 (native, self-hosted)\n");
            eprintln!("Usage: iris-stage0 <command> [options]\n");
            eprintln!("Commands:");
            eprintln!("  compile <source.iris> [-o output.json]");
            eprintln!("  run <source.iris> [args...]");
            eprintln!("  direct <program.json> [args...]");
            eprintln!("  interp <interp.json> <prog.json> [args]");
            eprintln!("  version");
        }
        "interp" => {
            if args.len() < 4 { eprintln!("Usage: iris-stage0 interp <interp.json> <prog.json> [args...]"); process::exit(1); }
            let interpreter = mini_eval::load_graph(&args[2])
                .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
            let target = mini_eval::load_graph(&args[3])
                .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
            let inputs = parse_args(&args[4..]);
            let interp_inputs = vec![
                Value::Program(Rc::new(target)),
                Value::tuple(inputs),
            ];
            let empty_reg = BTreeMap::new();
            let result = mini_eval::evaluate_with_registry(&interpreter, &interp_inputs, 100_000_000, &empty_reg)
                .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
            println!("{}", format_value(&result));
        }
        "version" | "--version" | "-V" => {
            println!("iris-stage0 0.1.0 (native, self-hosted)");
            println!("Embedded: tokenizer.json, parser.json, lowerer.json, self_interpreter.json");
            println!("Runtime: mini_eval ({} LOC)", 994);
        }
        other => {
            eprintln!("Unknown command: {}", other);
            process::exit(1);
        }
    }
}
