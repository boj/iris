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

// Compiler sources for the `build` command
const AOT_COMPILE_SRC: &str = include_str!("../../src/iris-programs/compiler/aot_compile.iris");
const ELF_WRAPPER_SRC: &str = include_str!("../../src/iris-programs/compiler/elf_wrapper.iris");
const NATIVE_RUNTIME_SRC: &str = include_str!("../../src/iris-programs/compiler/native_runtime.iris");

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

#[cfg(feature = "syntax")]
fn compile_with_full_registry(source: &str) -> (SemanticGraph, BTreeMap<iris_types::fragment::FragmentId, SemanticGraph>) {
    let result = iris_bootstrap::syntax::compile(source);
    if !result.errors.is_empty() {
        for e in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(source, e));
        }
        process::exit(1);
    }
    let mut registry = BTreeMap::new();
    let mut main_graph = None;
    for (_name, frag, _) in &result.fragments {
        registry.insert(frag.id, frag.graph.clone());
        main_graph = Some(frag.graph.clone());
    }
    (main_graph.unwrap(), registry)
}

#[cfg(feature = "syntax")]
fn cmd_build(runtime: &JitRuntime, args: &[String]) {
    use iris_types::fragment::FragmentId;

    if args.is_empty() {
        eprintln!("Usage: iris-stage0 build <source.iris> [-o binary] [--args N]");
        process::exit(1);
    }

    let source_path = &args[0];
    let mut output_path = String::from("a.out");
    let mut n_args = 0i64;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" if i + 1 < args.len() => { output_path = args[i + 1].clone(); i += 2; }
            "--args" if i + 1 < args.len() => { n_args = args[i + 1].parse().unwrap_or(0); i += 2; }
            _ => { i += 1; }
        }
    }

    let target_source = std::fs::read_to_string(source_path)
        .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
    let target = runtime.compile(&target_source);

    eprintln!("Loading code generator...");
    let aot_result = iris_bootstrap::syntax::compile(AOT_COMPILE_SRC);
    let mut aot_reg: BTreeMap<FragmentId, SemanticGraph> = BTreeMap::new();
    let mut aot_fn = None;
    for (name, frag, _) in &aot_result.fragments {
        aot_reg.insert(frag.id, frag.graph.clone());
        if name == "aot_compile" { aot_fn = Some(frag.graph.clone()); }
    }
    let aot_fn = aot_fn.expect("aot_compile not found");

    eprintln!("Loading ELF assembler...");
    let elf_result = iris_bootstrap::syntax::compile(ELF_WRAPPER_SRC);
    let mut elf_reg: BTreeMap<FragmentId, SemanticGraph> = BTreeMap::new();
    let mut elf_wrap_fn = None;
    let mut elf_wrap_rt_fn = None;
    for (name, frag, _) in &elf_result.fragments {
        elf_reg.insert(frag.id, frag.graph.clone());
        if name == "elf_wrap" { elf_wrap_fn = Some(frag.graph.clone()); }
        if name == "elf_wrap_rt" { elf_wrap_rt_fn = Some(frag.graph.clone()); }
    }

    eprintln!("Loading runtime...");
    let (rt_fn, rt_reg) = compile_with_full_registry(NATIVE_RUNTIME_SRC);
    let runtime_bytes = iris_bootstrap::evaluate_with_registry(
        &rt_fn, &[], 1_000_000, &rt_reg,
    ).unwrap_or_else(|e| { eprintln!("runtime error: {}", e); process::exit(1); });
    if let Value::Bytes(ref b) = runtime_bytes {
        eprintln!("Runtime: {} bytes", b.len());
    }

    eprintln!("Compiling {} to x86-64...", source_path);
    let code = iris_bootstrap::evaluate_with_registry(
        &aot_fn,
        &[Value::Program(Rc::new(target))],
        50_000_000,
        &aot_reg,
    ).unwrap_or_else(|e| { eprintln!("aot_compile error: {}", e); process::exit(1); });

    if let Value::Bytes(ref b) = code {
        eprintln!("Generated {} bytes of x86-64", b.len());
    }

    let elf = if let Some(ref wrap_rt) = elf_wrap_rt_fn {
        iris_bootstrap::evaluate_with_registry(
            wrap_rt, &[code, runtime_bytes, Value::Int(n_args)],
            10_000_000, &elf_reg,
        ).unwrap_or_else(|e| { eprintln!("elf_wrap_rt error: {}", e); process::exit(1); })
    } else {
        iris_bootstrap::evaluate_with_registry(
            &elf_wrap_fn.unwrap(), &[code, Value::Int(n_args)],
            10_000_000, &elf_reg,
        ).unwrap_or_else(|e| { eprintln!("elf_wrap error: {}", e); process::exit(1); })
    };

    if let Value::Bytes(ref b) = elf {
        std::fs::write(&output_path, b)
            .unwrap_or_else(|e| { eprintln!("write error: {}", e); process::exit(1); });
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&output_path, std::fs::Permissions::from_mode(0o755)).ok();
        }
        eprintln!("{} -> {} ({} bytes)", source_path, output_path, b.len());
    } else {
        eprintln!("error: ELF wrapper returned unexpected value");
        process::exit(1);
    }
}

#[cfg(not(feature = "syntax"))]
fn cmd_build(_runtime: &JitRuntime, _args: &[String]) {
    eprintln!("error: build command requires syntax feature");
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("iris-stage0: JIT-based IRIS compiler + runtime\n");
        eprintln!("Usage: iris-stage0 <command> [options]\n");
        eprintln!("Commands:");
        eprintln!("  build <source.iris> [-o binary] [--args N]   Compile to native binary");
        eprintln!("  run <source.iris> [args...]                  Interpret .iris source");
        eprintln!("  compile <source.iris> [-o output.json]       Compile to JSON graph");
        eprintln!("  direct <program.json> [args...]              Evaluate JSON graph");
        eprintln!("  version");
        process::exit(1);
    }

    let runtime = JitRuntime::new();

    match args[1].as_str() {
        "build" => {
            cmd_build(&runtime, &args[2..].to_vec());
        }
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
            eprintln!("  build <source.iris> [-o binary] [--args N]");
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
