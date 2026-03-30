//! End-to-end tests for the iris-stage0 bootstrap binary.
//!
//! iris-stage0 uses the IRIS-written compilation pipeline (no Rust syntax).
//! Tests cover:
//!   - direct: evaluate pre-compiled JSON graphs
//!   - interp: meta-circular evaluation through the IRIS interpreter
//!   - compile/run: compile .iris source via the IRIS pipeline
//!   - rebuild: regenerate bootstrap JSON artifacts
//!   - consistency: all execution paths produce the same result

use std::path::PathBuf;
use std::process::Command;

extern crate serde_json;

fn stage0() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let frozen = manifest.join("bootstrap/iris-stage0");
    if frozen.exists() {
        return frozen;
    }
    let built = manifest.join("target/release/iris-stage0");
    if built.exists() {
        return built;
    }
    panic!(
        "iris-stage0 binary not found. Run:\n  \
         cargo build --release --bin iris-stage0 && \
         cp target/release/iris-stage0 bootstrap/"
    );
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run iris-stage0 with args, assert success, return stdout.
fn stage0_ok(args: &[&str]) -> String {
    let out = Command::new(stage0())
        .args(args)
        .output()
        .expect("failed to execute iris-stage0");
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        panic!(
            "iris-stage0 {} failed (exit {}):\n{}",
            args.join(" "),
            out.status,
            stderr
        );
    }
    String::from_utf8(out.stdout).expect("stdout is utf8").trim().to_string()
}

/// Compile an .iris source file to JSON using the Rust syntax pipeline,
/// returning the path to the JSON file. This bypasses the IRIS pipeline
/// for tests that need known-good compilation.
fn compile_with_rust(source: &str, name: &str) -> PathBuf {
    let src_path = std::env::temp_dir().join(format!("iris_stage0_test_{}.iris", name));
    let json_path = std::env::temp_dir().join(format!("iris_stage0_test_{}.json", name));
    std::fs::write(&src_path, source).expect("write temp iris file");

    // Use iris-compile (Rust syntax) to produce known-good JSON
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let compile_bin = manifest.join("target/release/iris-compile");
    if !compile_bin.exists() {
        panic!("iris-compile not found. Run: cargo build --release --features syntax --bin iris-compile");
    }
    let out = Command::new(&compile_bin)
        .args([src_path.to_str().unwrap(), "-o", json_path.to_str().unwrap()])
        .output()
        .expect("failed to run iris-compile");
    if !out.status.success() {
        panic!("iris-compile failed: {}", String::from_utf8_lossy(&out.stderr));
    }
    json_path
}

// ===========================================================================
// Basic commands
// ===========================================================================

#[test]
fn test_version() {
    let out = stage0_ok(&["version"]);
    assert!(out.contains("iris-stage0"), "version output: {}", out);
    assert!(out.contains("self-hosted"), "should say self-hosted: {}", out);
}

#[test]
fn test_help() {
    let out = Command::new(stage0())
        .args(["help"])
        .output()
        .expect("run help");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("compile"), "help mentions compile");
    assert!(stderr.contains("run"), "help mentions run");
    assert!(stderr.contains("direct"), "help mentions direct");
    assert!(stderr.contains("interp"), "help mentions interp");
}

// ===========================================================================
// direct: evaluate pre-compiled JSON programs
// ===========================================================================

#[test]
fn test_direct_constant() {
    let json = compile_with_rust("let c = 42", "direct_const");
    assert_eq!(stage0_ok(&["direct", json.to_str().unwrap()]), "42");
    let _ = std::fs::remove_file(&json);
}

#[test]
fn test_direct_with_args() {
    let json = compile_with_rust("let main x : Int -> Int = x + 1", "direct_args");
    assert_eq!(stage0_ok(&["direct", json.to_str().unwrap(), "99"]), "100");
    let _ = std::fs::remove_file(&json);
}

#[test]
fn test_direct_arithmetic() {
    let json = compile_with_rust("let main x : Int -> Int = x * x + 1", "direct_arith");
    assert_eq!(stage0_ok(&["direct", json.to_str().unwrap(), "7"]), "50");
    let _ = std::fs::remove_file(&json);
}

#[test]
fn test_direct_two_args() {
    let json = compile_with_rust("let main x y : Int -> Int -> Int = x + y", "direct_two");
    assert_eq!(stage0_ok(&["direct", json.to_str().unwrap(), "13", "29"]), "42");
    let _ = std::fs::remove_file(&json);
}

#[test]
fn test_direct_conditional() {
    let json = compile_with_rust("let main x : Int -> Int = if x > 0 then x else 0 - x", "direct_cond");
    assert_eq!(stage0_ok(&["direct", json.to_str().unwrap(), "-5"]), "5");
    assert_eq!(stage0_ok(&["direct", json.to_str().unwrap(), "3"]), "3");
    let _ = std::fs::remove_file(&json);
}

#[test]
fn test_direct_let_binding() {
    let json = compile_with_rust("let main x : Int -> Int = let y = x + 1 in y * y", "direct_let");
    assert_eq!(stage0_ok(&["direct", json.to_str().unwrap(), "4"]), "25");
    let _ = std::fs::remove_file(&json);
}

#[test]
fn test_direct_nested_conditional() {
    let json = compile_with_rust(
        "let main x : Int -> Int = if x < 0 then 0 - 1 else if x == 0 then 0 else 1",
        "direct_nested_cond",
    );
    assert_eq!(stage0_ok(&["direct", json.to_str().unwrap(), "-10"]), "-1");
    assert_eq!(stage0_ok(&["direct", json.to_str().unwrap(), "0"]), "0");
    assert_eq!(stage0_ok(&["direct", json.to_str().unwrap(), "5"]), "1");
    let _ = std::fs::remove_file(&json);
}

// ===========================================================================
// interp: meta-circular interpreter
// ===========================================================================

#[test]
fn test_interp_simple() {
    let json = compile_with_rust("let main x : Int -> Int = x + x", "interp_simple");
    let interp_path = project_root().join("bootstrap/interpreter.json");
    let result = stage0_ok(&["interp", interp_path.to_str().unwrap(), json.to_str().unwrap(), "21"]);
    assert_eq!(result, "42");
    let _ = std::fs::remove_file(&json);
}

#[test]
fn test_interp_arithmetic() {
    let json = compile_with_rust("let main x : Int -> Int = (x + 3) * (x - 1)", "interp_arith");
    let interp_path = project_root().join("bootstrap/interpreter.json");
    let result = stage0_ok(&["interp", interp_path.to_str().unwrap(), json.to_str().unwrap(), "10"]);
    assert_eq!(result, "117");
    let _ = std::fs::remove_file(&json);
}

// ===========================================================================
// Consistency: direct and interp produce the same result
// ===========================================================================

#[test]
fn test_direct_interp_consistent() {
    let json = compile_with_rust("let main x : Int -> Int = (x + 3) * (x - 1)", "consistent");
    let interp_path = project_root().join("bootstrap/interpreter.json");

    let direct_result = stage0_ok(&["direct", json.to_str().unwrap(), "10"]);
    let interp_result = stage0_ok(&["interp", interp_path.to_str().unwrap(), json.to_str().unwrap(), "10"]);

    assert_eq!(direct_result, "117", "direct: (10+3)*(10-1) = 117");
    assert_eq!(interp_result, "117", "interp: (10+3)*(10-1) = 117");
    let _ = std::fs::remove_file(&json);
}

// ===========================================================================
// Self-hosted compilation: compile .iris source via the IRIS pipeline
// ===========================================================================

#[test]
#[ignore] // slow: IRIS tree-walking on 395-line tokenizer (~30s)
fn test_compile_tokenizer_source() {
    let root = project_root();
    let tokenizer_src = root.join("src/iris-programs/syntax/tokenizer.iris");
    let out_path = std::env::temp_dir().join("iris_stage0_test_tokenizer.json");

    stage0_ok(&["compile", tokenizer_src.to_str().unwrap(), "-o", out_path.to_str().unwrap()]);

    let json = std::fs::read_to_string(&out_path).expect("read compiled tokenizer");
    assert!(json.len() > 10_000, "tokenizer JSON is substantial: {} bytes", json.len());

    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(parsed.get("root").is_some(), "has root");
    assert!(parsed.get("nodes").is_some(), "has nodes");

    let _ = std::fs::remove_file(&out_path);
}

#[test]
#[ignore] // slow: IRIS tree-walking on 1064-line parser (~60s)
fn test_compile_parser_source() {
    let root = project_root();
    let parser_src = root.join("src/iris-programs/syntax/iris_parser.iris");
    let out_path = std::env::temp_dir().join("iris_stage0_test_parser.json");

    stage0_ok(&["compile", parser_src.to_str().unwrap(), "-o", out_path.to_str().unwrap()]);

    let json = std::fs::read_to_string(&out_path).expect("read compiled parser");
    assert!(json.len() > 10_000, "parser JSON is substantial: {} bytes", json.len());

    let _ = std::fs::remove_file(&out_path);
}

#[test]
#[ignore] // slow: IRIS tree-walking on large lowerer (~60s)
fn test_compile_lowerer_source() {
    let root = project_root();
    let lowerer_src = root.join("src/iris-programs/syntax/iris_lowerer.iris");
    let out_path = std::env::temp_dir().join("iris_stage0_test_lowerer.json");

    stage0_ok(&["compile", lowerer_src.to_str().unwrap(), "-o", out_path.to_str().unwrap()]);

    let json = std::fs::read_to_string(&out_path).expect("read compiled lowerer");
    assert!(json.len() > 10_000, "lowerer JSON is substantial: {} bytes", json.len());

    let _ = std::fs::remove_file(&out_path);
}

#[test]
#[ignore] // slow: IRIS tree-walking on interpreter (~30s)
fn test_compile_interpreter_source() {
    let root = project_root();
    let interp_src = root.join("src/iris-programs/interpreter/full_interpreter.iris");
    let out_path = std::env::temp_dir().join("iris_stage0_test_interp_compiled.json");

    stage0_ok(&["compile", interp_src.to_str().unwrap(), "-o", out_path.to_str().unwrap()]);

    let json = std::fs::read_to_string(&out_path).expect("read compiled interpreter");
    assert!(json.len() > 10_000, "interpreter JSON is substantial: {} bytes", json.len());

    let _ = std::fs::remove_file(&out_path);
}

// ===========================================================================
// Tokenizer smoke test: verify the IRIS tokenizer produces tokens
// ===========================================================================

#[test]
fn test_tokenizer_produces_output() {
    let root = project_root();
    let tok_path = root.join("bootstrap/tokenizer.json");

    // The tokenizer should produce a tuple of token tuples
    // Note: only simple sources work; operator handling has pre-existing issues
    let result = stage0_ok(&["direct", tok_path.to_str().unwrap(), "let c = 42"]);
    assert!(result.contains("("), "tokenizer produces structured output: {}", result);
    // Should contain at least 4 tokens: let, c, =, 42
    assert!(result.matches("(").count() >= 4, "at least 4 tokens: {}", result);
}

// ===========================================================================
// rebuild: regenerate bootstrap JSON from IRIS source
// ===========================================================================

#[test]
#[ignore] // slow: rebuilds all 4 JSON artifacts via IRIS tree-walking (~3min)
fn test_rebuild_produces_valid_json() {
    let root = project_root();
    let bootstrap = root.join("bootstrap");

    let before: Vec<u64> = ["tokenizer.json", "parser.json", "lowerer.json", "interpreter.json"]
        .iter()
        .map(|f| std::fs::metadata(bootstrap.join(f)).map(|m| m.len()).unwrap_or(0))
        .collect();

    stage0_ok(&["rebuild", root.to_str().unwrap()]);

    for (i, name) in ["tokenizer.json", "parser.json", "lowerer.json", "interpreter.json"]
        .iter()
        .enumerate()
    {
        let path = bootstrap.join(name);
        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read rebuilt {}: {}", name, e));

        assert!(json.contains("\"root\""), "{} has root", name);
        assert!(json.contains("\"nodes\""), "{} has nodes", name);

        let size = json.len() as u64;
        assert!(size > 1000, "{} has substance: {} bytes", name, size);

        if before[i] > 0 {
            let ratio = size as f64 / before[i] as f64;
            assert!(
                (0.5..2.0).contains(&ratio),
                "{} size changed dramatically: {} -> {} (ratio {})",
                name, before[i], size, ratio
            );
        }
    }

    // Restore original JSONs (rebuild may produce slightly different serialization)
    let _ = Command::new("git")
        .args(["checkout", "bootstrap/tokenizer.json", "bootstrap/parser.json",
               "bootstrap/lowerer.json", "bootstrap/interpreter.json"])
        .current_dir(&root)
        .output();
}

// ===========================================================================
// Self-interpreter tests
// ===========================================================================

/// Run a program through the IRIS self-interpreter.
fn self_interpret(source: &str, name: &str, inputs: &[iris_types::eval::Value]) -> iris_types::eval::Value {
    use iris_types::eval::Value;

    let json = compile_with_rust(source, name);
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let si = iris_bootstrap::load_graph(
        manifest.join("bootstrap/self_interpreter.json").to_str().unwrap()
    ).unwrap();
    let target = iris_bootstrap::load_graph(json.to_str().unwrap()).unwrap();

    // Build env: BinderId(0xFFFF_0000 + i) = inputs[i]
    let env_entries: Vec<Value> = inputs.iter().enumerate()
        .map(|(i, v)| Value::tuple(vec![Value::Int(0xFFFF_0000u32 as i64 + i as i64), v.clone()]))
        .collect();
    let env = if env_entries.is_empty() { Value::Range(0, 0) } else { Value::tuple(env_entries) };

    let result = iris_bootstrap::evaluate_with_limit(
        &si,
        &[Value::Program(std::rc::Rc::new(si.clone())), Value::Program(std::rc::Rc::new(target)), env],
        50_000_000,
    );

    let _ = std::fs::remove_file(&json);
    result.unwrap_or_else(|e| panic!("self-interpreter failed on {}: {}", name, e))
}

#[test]
fn test_si_constant() {
    assert_eq!(self_interpret("let main = 42", "si_const", &[]),
        iris_types::eval::Value::Int(42));
}

#[test]
fn test_si_identity() {
    assert_eq!(self_interpret("let f x = x", "si_id", &[iris_types::eval::Value::Int(7)]),
        iris_types::eval::Value::Int(7));
}

#[test]
fn test_si_addition() {
    assert_eq!(self_interpret("let f x y = x + y", "si_add",
        &[iris_types::eval::Value::Int(3), iris_types::eval::Value::Int(5)]),
        iris_types::eval::Value::Int(8));
}

#[test]
fn test_si_guard() {
    assert_eq!(self_interpret("let f x = if x > 0 then x else 0 - x", "si_guard",
        &[iris_types::eval::Value::Int(-5)]),
        iris_types::eval::Value::Int(5));
}

#[test]
fn test_si_let_binding() {
    assert_eq!(self_interpret("let f x = let y = x + 1 in y * y", "si_let",
        &[iris_types::eval::Value::Int(4)]),
        iris_types::eval::Value::Int(25));
}

#[test]
fn test_si_nested_expr() {
    assert_eq!(self_interpret("let f x = (x + 1) * (x - 1)", "si_nested",
        &[iris_types::eval::Value::Int(5)]),
        iris_types::eval::Value::Int(24));
}

// ===========================================================================
// mini_eval tests: verify the minimal evaluator runs self_interpreter.json
// ===========================================================================

/// Run a program through the IRIS self-interpreter using mini_eval (not the full evaluator).
fn mini_self_interpret(source: &str, name: &str, inputs: &[iris_types::eval::Value]) -> iris_types::eval::Value {
    use iris_types::eval::Value;

    let json = compile_with_rust(source, name);
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let si = iris_bootstrap::mini_eval::load_graph(
        manifest.join("bootstrap/self_interpreter.json").to_str().unwrap()
    ).unwrap();
    let target = iris_bootstrap::mini_eval::load_graph(json.to_str().unwrap()).unwrap();

    let env_entries: Vec<Value> = inputs.iter().enumerate()
        .map(|(i, v)| Value::tuple(vec![Value::Int(0xFFFF_0000u32 as i64 + i as i64), v.clone()]))
        .collect();
    let env = if env_entries.is_empty() { Value::Range(0, 0) } else { Value::tuple(env_entries) };

    let empty_reg = std::collections::BTreeMap::new();
    let result = iris_bootstrap::mini_eval::evaluate_with_registry(
        &si,
        &[Value::Program(std::rc::Rc::new(si.clone())), Value::Program(std::rc::Rc::new(target)), env],
        50_000_000,
        &empty_reg,
    );

    let _ = std::fs::remove_file(&json);
    result.unwrap_or_else(|e| panic!("mini_eval failed on {}: {}", name, e))
}

#[test]
fn test_mini_constant() {
    assert_eq!(mini_self_interpret("let main = 42", "mini_const", &[]),
        iris_types::eval::Value::Int(42));
}

#[test]
fn test_mini_identity() {
    assert_eq!(mini_self_interpret("let f x = x", "mini_id", &[iris_types::eval::Value::Int(7)]),
        iris_types::eval::Value::Int(7));
}

#[test]
fn test_mini_addition() {
    assert_eq!(mini_self_interpret("let f x y = x + y", "mini_add",
        &[iris_types::eval::Value::Int(3), iris_types::eval::Value::Int(5)]),
        iris_types::eval::Value::Int(8));
}

#[test]
fn test_mini_guard() {
    assert_eq!(mini_self_interpret("let f x = if x > 0 then x else 0 - x", "mini_guard",
        &[iris_types::eval::Value::Int(-5)]),
        iris_types::eval::Value::Int(5));
}

#[test]
fn test_mini_let_binding() {
    assert_eq!(mini_self_interpret("let f x = let y = x + 1 in y * y", "mini_let",
        &[iris_types::eval::Value::Int(4)]),
        iris_types::eval::Value::Int(25));
}

#[test]
fn test_mini_nested_expr() {
    assert_eq!(mini_self_interpret("let f x = (x + 1) * (x - 1)", "mini_nested",
        &[iris_types::eval::Value::Int(5)]),
        iris_types::eval::Value::Int(24));
}

#[test]
fn test_mini_compile_constant() {
    use iris_types::eval::Value;
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bd = manifest.join("bootstrap");
    let load = |name: &str| iris_bootstrap::mini_eval::load_graph(bd.join(name).to_str().unwrap()).unwrap();
    let tok = load("tokenizer.json");
    let par = load("parser.json");
    let low = load("lowerer.json");
    let si = load("self_interpreter.json");
    let empty_reg = std::collections::BTreeMap::new();
    let source = "let c = 42";
    let tokens = iris_bootstrap::mini_eval::evaluate_with_registry(
        &tok, &[Value::String(source.into())], 5_000_000, &empty_reg).expect("tokenize");
    let ast = iris_bootstrap::mini_eval::evaluate_with_registry(
        &par, &[tokens, Value::String(source.into())], 50_000_000, &empty_reg).expect("parse");
    let program = iris_bootstrap::mini_eval::evaluate_with_registry(
        &low, &[ast, Value::String(source.into())], 50_000_000, &empty_reg).expect("lower");
    let graph = match program {
        Value::Program(g) => std::rc::Rc::try_unwrap(g).unwrap_or_else(|rc| (*rc).clone()),
        Value::Tuple(ref f) if !f.is_empty() => match &f[0] {
            Value::Program(g) => g.as_ref().clone(), _ => panic!("unexpected"),
        }, _ => panic!("unexpected"),
    };
    let env = Value::Range(0, 0);
    let result = iris_bootstrap::mini_eval::evaluate_with_registry(
        &si, &[Value::Program(std::rc::Rc::new(si.clone())), Value::Program(std::rc::Rc::new(graph)), env],
        10_000_000, &empty_reg).expect("eval");
    assert_eq!(result, Value::Int(42));
}

#[cfg(feature = "syntax")]
#[test]
fn test_native_compile_simple() {
    use iris_types::eval::Value;
    use iris_types::fragment::FragmentId;
    use iris_types::graph::SemanticGraph;
    use std::collections::BTreeMap;

    // Compile aot_compile.iris with full fragment registry
    let aot_source = std::fs::read_to_string("src/iris-programs/compiler/aot_compile.iris").unwrap();
    let aot_result = iris_bootstrap::syntax::compile(&aot_source);
    assert!(aot_result.errors.is_empty(), "aot_compile.iris has errors: {:?}", aot_result.errors);
    
    let mut aot_registry: BTreeMap<FragmentId, SemanticGraph> = BTreeMap::new();
    let mut aot_main = None;
    for (name, frag, _) in &aot_result.fragments {
        aot_registry.insert(frag.id, frag.graph.clone());
        if name == "aot_compile" { aot_main = Some(frag.graph.clone()); }
    }
    let aot = aot_main.expect("aot_compile function not found");

    // Compile elf_wrapper.iris
    let elf_source = std::fs::read_to_string("src/iris-programs/compiler/elf_wrapper.iris").unwrap();
    let elf_result = iris_bootstrap::syntax::compile(&elf_source);
    assert!(elf_result.errors.is_empty(), "elf_wrapper.iris has errors");
    let mut elf_registry: BTreeMap<FragmentId, SemanticGraph> = BTreeMap::new();
    let mut elf_main = None;
    for (name, frag, _) in &elf_result.fragments {
        elf_registry.insert(frag.id, frag.graph.clone());
        if name == "elf_wrap" { elf_main = Some(frag.graph.clone()); }
    }
    let elf_wrap = elf_main.expect("elf_wrap function not found");
    
    // Target: "let f x = x + 1"
    let target_json = compile_with_rust("let f x : Int -> Int = x + 1", "native_target");
    let target = iris_bootstrap::load_graph(target_json.to_str().unwrap()).unwrap();
    
    // Step 1: aot_compile(target) -> machine code bytes
    let code = iris_bootstrap::evaluate_with_registry(
        &aot,
        &[Value::Program(std::rc::Rc::new(target))],
        10_000_000,
        &aot_registry,
    ).expect("aot_compile");
    
    eprintln!("aot_compile result: {:?}", match &code {
        Value::Bytes(b) => format!("{} bytes of machine code", b.len()),
        Value::Tuple(t) => format!("tuple of {}", t.len()),
        _ => format!("{:?}", code),
    });
    
    // Step 2: elf_wrap(code, 1) -> ELF binary
    let elf = iris_bootstrap::evaluate_with_registry(
        &elf_wrap,
        &[code, Value::Int(1)],
        10_000_000,
        &elf_registry,
    ).expect("elf_wrap");
    
    if let Value::Bytes(ref b) = elf {
        eprintln!("ELF binary: {} bytes", b.len());
        let path = std::env::temp_dir().join("iris_native_test");
        std::fs::write(&path, b).unwrap();
        std::process::Command::new("chmod").args(["+x", path.to_str().unwrap()]).output().unwrap();
        let output = std::process::Command::new(path.to_str().unwrap())
            .arg("41").output().expect("run native binary");
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("stdout: {:?}, stderr: {:?}, status: {}", stdout, stderr, output.status);
        assert_eq!(stdout, "42", "native binary should compute 41+1=42");
        let _ = std::fs::remove_file(&path);
    } else {
        panic!("elf_wrap returned {:?}", elf);
    }
    
    let _ = std::fs::remove_file(&target_json);
}
