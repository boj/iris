//! iris-stage0-next: JIT-based IRIS stage0 binary.
//!
//! Uses tagged i64 values and JIT compilation for native-speed execution.
//! The self-interpreter and compilation pipeline run through the JIT runtime
//! with Rust helper functions for complex operations (strings, graphs, I/O).
//!
//! All bootstrap artifacts are embedded at compile time.

use std::collections::BTreeMap;
use std::env;
use std::process;
use std::rc::Rc;

use iris_bootstrap::jit;
use iris_bootstrap::mini_eval;
use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;

const TOKENIZER_JSON: &str = include_str!("../../bootstrap/tokenizer.json");
const PARSER_JSON: &str = include_str!("../../bootstrap/parser.json");
const LOWERER_JSON: &str = include_str!("../../bootstrap/lowerer.json");
const SELF_INTERP_JSON: &str = include_str!("../../bootstrap/self_interpreter.json");

fn load_embedded(json: &str) -> SemanticGraph {
    serde_json::from_str(json).expect("embedded JSON parse error")
}

struct JitRuntime {
    tokenizer: SemanticGraph,
    parser: SemanticGraph,
    lowerer: SemanticGraph,
    interpreter: SemanticGraph,
}

impl JitRuntime {
    fn new() -> Self {
        Self {
            tokenizer: load_embedded(TOKENIZER_JSON),
            parser: load_embedded(PARSER_JSON),
            lowerer: load_embedded(LOWERER_JSON),
            interpreter: load_embedded(SELF_INTERP_JSON),
        }
    }

    /// Evaluate a compiled program through the JIT self-interpreter.
    fn eval_program(&self, graph: &SemanticGraph, inputs: &[Value]) -> Value {
        // Build env as tagged i64 values
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

        // Pack inputs as tagged i64 for the self-interpreter
        let me_tagged = jit::pack(Value::Program(Rc::new(self.interpreter.clone())));
        let prog_tagged = jit::pack(Value::Program(Rc::new(graph.clone())));
        let env_tagged = jit::pack(env);

        // Call the self-interpreter through the JIT runtime
        let result_tagged = jit::rt_self_eval(me_tagged, prog_tagged, env_tagged);

        // Unpack result
        let result = jit::unpack(result_tagged);

        // Free tagged heap values
        jit::free_tagged(me_tagged);
        jit::free_tagged(prog_tagged);
        jit::free_tagged(env_tagged);
        jit::free_tagged(result_tagged);

        result
    }

    /// Evaluate a compiler stage directly via mini_eval (fastest for pipeline stages).
    fn eval_direct(&self, graph: &SemanticGraph, inputs: &[Value]) -> Value {
        let empty_reg = BTreeMap::new();
        mini_eval::evaluate_with_registry(graph, inputs, 100_000_000, &empty_reg)
            .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); })
    }

    /// Compile source using the IRIS pipeline.
    fn compile(&self, source: &str) -> SemanticGraph {
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
        eprintln!("iris-stage0-next: JIT-based IRIS runtime\n");
        eprintln!("Usage: iris-stage0-next <command> [options]\n");
        eprintln!("Commands:");
        eprintln!("  run <source.iris> [args...]");
        eprintln!("  compile <source.iris> [-o output.json]");
        eprintln!("  direct <program.json> [args...]");
        eprintln!("  version");
        process::exit(1);
    }

    let runtime = JitRuntime::new();

    match args[1].as_str() {
        "run" => {
            if args.len() < 3 { eprintln!("Usage: run <source.iris> [args...]"); process::exit(1); }
            let source = std::fs::read_to_string(&args[2])
                .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
            let graph = runtime.compile(&source);
            let result = runtime.eval_program(&graph, &parse_args(&args[3..]));
            println!("{}", format_value(&result));
        }
        "compile" => {
            if args.len() < 3 { eprintln!("Usage: compile <source.iris> [-o output.json]"); process::exit(1); }
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
        "direct" => {
            if args.len() < 3 { eprintln!("Usage: direct <program.json> [args...]"); process::exit(1); }
            let program = mini_eval::load_graph(&args[2])
                .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
            let result = runtime.eval_program(&program, &parse_args(&args[3..]));
            println!("{}", format_value(&result));
        }
        "help" | "--help" | "-h" => {
            eprintln!("iris-stage0-next: JIT-based IRIS runtime\n");
            eprintln!("Usage: iris-stage0-next <command> [options]\n");
            eprintln!("Commands:");
            eprintln!("  run <source.iris> [args...]");
            eprintln!("  compile <source.iris> [-o output.json]");
            eprintln!("  direct <program.json> [args...]");
            eprintln!("  interp <interp.json> <prog.json> [args]");
            eprintln!("  version");
        }
        "interp" => {
            if args.len() < 4 { eprintln!("Usage: interp <interp.json> <prog.json> [args...]"); process::exit(1); }
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
            println!("iris-stage0 0.1.0 (JIT-based, self-hosted)");
            println!("Runtime: jit.rs (tagged i64) + mini_eval (994 LOC) fallback");
        }
        other => {
            eprintln!("Unknown command: {}", other);
            process::exit(1);
        }
    }
}
