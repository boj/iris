//! iris-native: Compile IRIS source to native x86-64 ELF binary.
//! Uses aot_compile.iris + elf_wrapper.iris with full fragment registries.

use std::collections::BTreeMap;
use std::rc::Rc;
use iris_types::eval::Value;
use iris_types::fragment::FragmentId;
use iris_types::graph::SemanticGraph;

fn compile_with_registry(source: &str) -> (SemanticGraph, BTreeMap<FragmentId, SemanticGraph>) {
    let result = iris_bootstrap::syntax::compile(source);
    if !result.errors.is_empty() {
        for e in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(source, e));
        }
        std::process::exit(1);
    }
    let mut registry = BTreeMap::new();
    let mut main_graph = None;
    for (name, frag, _) in &result.fragments {
        registry.insert(frag.id, frag.graph.clone());
        main_graph = Some((name.clone(), frag.graph.clone()));
    }
    let (_, graph) = main_graph.unwrap();
    (graph, registry)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: iris-native <source.iris> [-o output] [--args N]");
        std::process::exit(1);
    }

    let source_path = &args[1];
    let mut output_path = String::from("a.out");
    let mut n_args = 0i64;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-o" if i + 1 < args.len() => { output_path = args[i + 1].clone(); i += 2; }
            "--args" if i + 1 < args.len() => { n_args = args[i + 1].parse().unwrap_or(0); i += 2; }
            _ => { i += 1; }
        }
    }

    // Compile target program
    let target_source = std::fs::read_to_string(source_path)
        .unwrap_or_else(|e| { eprintln!("error: {}", e); std::process::exit(1); });
    let target_result = iris_bootstrap::syntax::compile(&target_source);
    if !target_result.errors.is_empty() {
        for e in &target_result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(&target_source, e));
        }
        std::process::exit(1);
    }
    let target = target_result.fragments.last().unwrap().1.graph.clone();

    // Compile aot_compile.iris
    eprintln!("Compiling aot_compile.iris...");
    let aot_source = std::fs::read_to_string("src/iris-programs/compiler/aot_compile.iris").unwrap();
    let (aot, aot_reg) = compile_with_registry(&aot_source);

    // Compile elf_wrapper.iris
    eprintln!("Compiling elf_wrapper.iris...");
    let elf_source = std::fs::read_to_string("src/iris-programs/compiler/elf_wrapper.iris").unwrap();
    let (elf_wrap, elf_reg) = compile_with_registry(&elf_source);

    // Step 1: AOT compile
    eprintln!("AOT compiling {}...", source_path);
    let code = iris_bootstrap::evaluate_with_registry(
        &aot, &[Value::Program(Rc::new(target))],
        50_000_000, &aot_reg,
    ).unwrap_or_else(|e| { eprintln!("aot_compile failed: {}", e); std::process::exit(1); });

    if let Value::Bytes(ref b) = code {
        eprintln!("Generated {} bytes of x86-64 machine code", b.len());
    }

    // Step 2: ELF wrap
    let elf = iris_bootstrap::evaluate_with_registry(
        &elf_wrap, &[code, Value::Int(n_args)],
        10_000_000, &elf_reg,
    ).unwrap_or_else(|e| { eprintln!("elf_wrap failed: {}", e); std::process::exit(1); });

    if let Value::Bytes(ref b) = elf {
        std::fs::write(&output_path, b)
            .unwrap_or_else(|e| { eprintln!("write error: {}", e); std::process::exit(1); });
        // Make executable
        std::process::Command::new("chmod").args(["+x", &output_path]).output().ok();
        eprintln!("Wrote {} ({} bytes)", output_path, b.len());
    } else {
        eprintln!("elf_wrap returned unexpected: {:?}", elf);
        std::process::exit(1);
    }
}
