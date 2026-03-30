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

// Embed compiler sources for the `build` command (native binary generation)
const AOT_COMPILE_SRC: &str = include_str!("../../src/iris-programs/compiler/aot_compile.iris");
const ELF_WRAPPER_SRC: &str = include_str!("../../src/iris-programs/compiler/elf_wrapper.iris");
const NATIVE_RUNTIME_SRC: &str = include_str!("../../src/iris-programs/compiler/native_runtime.iris");

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

#[cfg(feature = "syntax")]
fn compile_iris_with_registry(source: &str) -> (SemanticGraph, BTreeMap<iris_types::fragment::FragmentId, SemanticGraph>) {
    let result = iris_bootstrap::syntax::compile(source);
    if !result.errors.is_empty() {
        for e in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(source, e));
        }
        process::exit(1);
    }
    let mut registry = BTreeMap::new();
    let mut main_graph = None;
    for (name, frag, _) in &result.fragments {
        registry.insert(frag.id, frag.graph.clone());
        main_graph = Some(frag.graph.clone());
        let _ = name; // use last fragment as main
    }
    (main_graph.unwrap(), registry)
}

#[cfg(feature = "syntax")]
fn cmd_build(_runtime: &IrisRuntime, args: &[String]) {
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

    // Step 1: Compile target source
    let target_source = std::fs::read_to_string(source_path)
        .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });
    let (target, _target_reg) = compile_iris_with_registry(&target_source);

    // Step 2: Compile aot_compile.iris (the native code generator)
    eprintln!("Loading code generator...");
    let aot_result = iris_bootstrap::syntax::compile(AOT_COMPILE_SRC);
    if !aot_result.errors.is_empty() {
        for e in &aot_result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(AOT_COMPILE_SRC, e));
        }
        process::exit(1);
    }
    let mut aot_reg: BTreeMap<FragmentId, SemanticGraph> = BTreeMap::new();
    let mut aot_fn = None;
    for (name, frag, _) in &aot_result.fragments {
        aot_reg.insert(frag.id, frag.graph.clone());
        if name == "aot_compile" { aot_fn = Some(frag.graph.clone()); }
    }
    let aot_fn = aot_fn.expect("aot_compile function not found");

    // Step 3: Compile elf_wrapper.iris (the ELF assembler)
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

    // Step 4: Compile native_runtime.iris (the runtime library)
    eprintln!("Loading runtime...");
    let (rt_fn, rt_reg) = compile_iris_with_registry(NATIVE_RUNTIME_SRC);
    let runtime_bytes = iris_bootstrap::evaluate_with_registry(
        &rt_fn, &[], 1_000_000, &rt_reg,
    ).unwrap_or_else(|e| { eprintln!("runtime error: {}", e); process::exit(1); });
    if let Value::Bytes(ref b) = runtime_bytes {
        eprintln!("Runtime: {} bytes", b.len());
    }

    // Step 5: AOT compile the target via IRIS code generator
    eprintln!("Compiling {} to x86-64...", source_path);
    let code = iris_bootstrap::evaluate_with_registry(
        &aot_fn,
        &[Value::Program(Rc::new(target.clone()))],
        50_000_000,
        &aot_reg,
    ).unwrap_or_else(|e| { eprintln!("aot_compile error: {}", e); process::exit(1); });

    if let Value::Bytes(ref b) = code {
        eprintln!("Generated {} bytes of x86-64", b.len());
    }

    // Step 6: Wrap in ELF (with runtime if available)
    let elf = if let Some(ref wrap_rt) = elf_wrap_rt_fn {
        iris_bootstrap::evaluate_with_registry(
            wrap_rt,
            &[code, runtime_bytes, Value::Int(n_args)],
            10_000_000,
            &elf_reg,
        ).unwrap_or_else(|e| { eprintln!("elf_wrap_rt error: {}", e); process::exit(1); })
    } else {
        iris_bootstrap::evaluate_with_registry(
            &elf_wrap_fn.unwrap(),
            &[code, Value::Int(n_args)],
            10_000_000,
            &elf_reg,
        ).unwrap_or_else(|e| { eprintln!("elf_wrap error: {}", e); process::exit(1); })
    };

    // Step 7: Write binary
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
        eprintln!("error: ELF wrapper returned {:?}", elf);
        process::exit(1);
    }
}

#[cfg(not(feature = "syntax"))]
fn cmd_build(_runtime: &IrisRuntime, _args: &[String]) {
    eprintln!("error: build command requires syntax feature");
    process::exit(1);
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
        "build" => {
            cmd_build(&runtime, &args[2..]);
        }
        "help" | "--help" | "-h" => {
            eprintln!("iris-stage0 (native, self-hosted)\n");
            eprintln!("Usage: iris-stage0 <command> [options]\n");
            eprintln!("Commands:");
            eprintln!("  build <source.iris> [-o binary] [--args N]   Compile to native binary");
            eprintln!("  run <source.iris> [args...]                  Interpret .iris source");
            eprintln!("  compile <source.iris> [-o output.json]       Compile to JSON graph");
            eprintln!("  direct <program.json> [args...]              Evaluate JSON graph");
            eprintln!("  interp <interp.json> <prog.json> [args]      Meta-circular eval");
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
