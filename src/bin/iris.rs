use std::env;
use std::fs;
use std::process;

#[cfg(feature = "rust-scaffolding")]
use iris_types::eval::Value;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        #[cfg(feature = "rust-scaffolding")]
        "run" => cmd_run(&args[2..]),
        #[cfg(feature = "rust-scaffolding")]
        "solve" => cmd_solve(&args[2..]),
        #[cfg(feature = "rust-scaffolding")]
        "deploy" => cmd_deploy(&args[2..]),
        #[cfg(feature = "rust-scaffolding")]
        "compile" => cmd_compile(&args[2..]),
        #[cfg(feature = "rust-scaffolding")]
        "check" => cmd_check(&args[2..]),
        #[cfg(feature = "rust-scaffolding")]
        "lint" => cmd_lint(&args[2..]),
        #[cfg(feature = "rust-scaffolding")]
        "daemon" => cmd_daemon(&args[2..]),
        #[cfg(feature = "rust-scaffolding")]
        "repl" => cmd_repl(),
        "store" => cmd_store(&args[2..]),
        "explain" => cmd_explain(&args[2..]),
        "version" | "--version" | "-V" => println!("iris 0.1.0"),
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
IRIS Language Toolkit v0.1.0

Usage: iris <command> [options]

Commands:
  run [--improve] [--watch] <file.iris> [args...]  Execute an IRIS program
    --watch                              Watch file and re-run on changes
    --improve                            Enable observation-driven improvement
    --improve-threshold 2.0              Max slowdown for gate (default: 2.0)
    --improve-min-traces 50              Min traces before evolving (default: 50)
    --improve-sample-rate 0.01           Fraction of calls to trace (default: 1%)
    --improve-budget 5                   Max seconds per evolution attempt
  solve <spec.iris>               Evolve a solution from a specification
  deploy <file.iris> -o <out>     Compile to standalone binary source
  compile <file.iris> -o <out>    AOT compile to native ELF binary
  check <file.iris>               Type-check / verify a program
  lint <file.iris>                Static analysis (unused bindings, shadowing, etc.)
  daemon [N] [options]            Run threaded self-improvement daemon
    --exec-mode continuous|interval:800  Execution mode (default: interval:800)
    --improve-threshold 2.0              Max slowdown for gate (default: 2.0)
    --max-stagnant 5                     Give up after N failed attempts (default: 5)
    --max-improve-threads 2              Concurrent improvement threads (default: 2)
    --max-cycles N                       Stop after N cycles
  store <subcommand>              Manage the fragment cache
    list                          List all cached fragments
    get <name>                    Show details of a cached fragment
    rm <name>                     Remove a cached fragment
    clear                         Clear the entire cache
    path                          Print the cache directory path
  repl                            Interactive REPL
  version                         Print version
  help                            Show this help"
    );
}

fn cmd_store(args: &[String]) {
    use iris_bootstrap::fragment_cache;
    if args.is_empty() {
        eprintln!("Usage: iris store <list|get|rm|clear|path>");
        process::exit(1);
    }
    let dir = fragment_cache::cache_dir();
    match args[0].as_str() {
        "list" => {
            let entries = fragment_cache::list_cached(&dir);
            if entries.is_empty() { println!("(no cached fragments)"); }
            else {
                println!("{:<30} {:>5}  {}", "NAME", "GEN", "HASH");
                for (name, generation, hex_id) in &entries {
                    let prefix = if hex_id.len() >= 16 { &hex_id[..16] } else { hex_id };
                    println!("{:<30} {:>5}  {}", name, generation, prefix);
                }
                println!("\n{} fragment(s)", entries.len());
            }
        }
        "get" => {
            if args.len() < 2 { eprintln!("Usage: iris store get <name>"); process::exit(1); }
            let name = &args[1];
            let entries = fragment_cache::list_cached(&dir);
            match entries.iter().find(|(n, _, _)| n == name) {
                Some((_, generation, hex_id)) => {
                    let prefix = if hex_id.len() >= 16 { &hex_id[..16] } else { hex_id.as_str() };
                    let frag_file = dir.join(format!("{}.frag", prefix));
                    let size = std::fs::metadata(&frag_file).map(|m| m.len()).unwrap_or(0);
                    println!("Name:       {}", name);
                    println!("Generation: {}", generation);
                    println!("Hash:       {}", hex_id);
                    println!("File:       {}", frag_file.display());
                    println!("Size:       {} bytes", size);
                }
                None => { eprintln!("Fragment '{}' not found.", name); process::exit(1); }
            }
        }
        "rm" => {
            if args.len() < 2 { eprintln!("Usage: iris store rm <name>"); process::exit(1); }
            if fragment_cache::remove_fragment(&dir, &args[1]) { println!("Removed '{}'.", args[1]); }
            else { eprintln!("Not found."); process::exit(1); }
        }
        "clear" => { fragment_cache::clear_cache(&dir); println!("Cache cleared."); }
        "path" => { println!("{}", dir.display()); }
        other => { eprintln!("Unknown: {}\nUsage: iris store <list|get|rm|clear|path>", other); process::exit(1); }
    }
}

// ===========================================================================
// Error explanations
// ===========================================================================

static EXPLAIN_E001: &str = "E001: Unknown identifier\n\nThe compiler could not find a variable, function, or primitive with this name\nin the current scope. The compiler suggests close matches when available.\n\nCommon causes: typo, using before defined, missing import, nonexistent primitive.";
static EXPLAIN_E002: &str = "E002: Type mismatch\n\nA value was used where a different type was expected.\n\nCommon causes: passing Int where List expected, different types from match arms,\narithmetic on non-numeric values, applying a non-function.";
static EXPLAIN_E003: &str = "E003: Non-exhaustive pattern match\n\nA match expression does not cover all possible cases.\n\nFix: add the missing pattern or a wildcard _ catch-all arm.";
static EXPLAIN_E004: &str = "E004: Division by zero\n\nInteger division or modulo by zero. Guard with: if y == 0 then 0 else x / y";
static EXPLAIN_E005: &str = "E005: Step limit exceeded\n\nThe evaluator hit its maximum step count. Common causes: infinite recursion,\nmissing base case, very large input. Default limit is 10,000,000.";
static EXPLAIN_E006: &str = "E006: Unused binding\n\nA let binding introduces a name never referenced in its body.\nFix: remove it, prefix with _ to mark intentional, or use it.";

fn cmd_explain(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: iris explain <error-code>\n");
        eprintln!("Available codes:");
        eprintln!("  E001  Unknown identifier");
        eprintln!("  E002  Type mismatch");
        eprintln!("  E003  Non-exhaustive pattern match");
        eprintln!("  E004  Division by zero");
        eprintln!("  E005  Step limit exceeded");
        eprintln!("  E006  Unused binding");
        process::exit(1);
    }
    let code = args[0].to_uppercase();
    let explanation = match code.as_str() {
        "E001" => EXPLAIN_E001, "E002" => EXPLAIN_E002, "E003" => EXPLAIN_E003,
        "E004" => EXPLAIN_E004, "E005" => EXPLAIN_E005, "E006" => EXPLAIN_E006,
        _ => { eprintln!("Unknown error code: {}. Run `iris explain` for available codes.", args[0]); process::exit(1); }
    };
    println!("{}", explanation);
}

// ===========================================================================
// Commands: run, check, repl
// ===========================================================================

#[cfg(feature = "rust-scaffolding")]
fn cmd_run(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: iris run [--improve] [--watch] [--backend auto|tree|jit|aot|clcu] <file.iris> [args...]");
        process::exit(1);
    }

    // Parse flags before the file path.
    let mut improve_mode = false;
    let mut watch_mode = false;
    let mut improve_threshold = 2.0f64;
    let mut improve_min_traces = 50u64;
    let mut improve_sample_rate = 0.01f64;
    let mut improve_budget = 5u64;
    let mut backend = "tree".to_string();
    let mut positional_start = 0usize;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--watch" => { watch_mode = true; i += 1; }
            "--improve" => { improve_mode = true; i += 1; }
            "--improve-threshold" => {
                i += 1;
                improve_threshold = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(2.0);
                i += 1;
            }
            "--improve-min-traces" => {
                i += 1;
                improve_min_traces = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(50);
                i += 1;
            }
            "--improve-sample-rate" => {
                i += 1;
                improve_sample_rate = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(0.01);
                i += 1;
            }
            "--improve-budget" => {
                i += 1;
                improve_budget = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(5);
                i += 1;
            }
            "--backend" => {
                i += 1;
                backend = args.get(i).map(|s| s.to_string()).unwrap_or_else(|| "tree".to_string());
                i += 1;
            }
            _ => { positional_start = i; break; }
        }
    }

    let remaining = &args[positional_start..];
    if remaining.is_empty() {
        eprintln!("Usage: iris run [--improve] [--watch] <file.iris> [args...]");
        process::exit(1);
    }

    let path = &remaining[0];
    let cli_args = &remaining[1..];

    // Run once (or in a loop if --watch).
    run_once(path, cli_args, &backend, improve_mode, improve_threshold,
             improve_min_traces, improve_sample_rate, improve_budget);

    if watch_mode {
        use std::time::{Duration, SystemTime};

        let poll_interval = Duration::from_millis(500);
        let mut last_mtime = fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        eprintln!("[watch] watching '{}' for changes (Ctrl-C to stop)", path);

        loop {
            std::thread::sleep(poll_interval);
            let current_mtime = match fs::metadata(path).and_then(|m| m.modified()) {
                Ok(t) => t,
                Err(_) => continue,
            };
            if current_mtime != last_mtime {
                last_mtime = current_mtime;
                // Clear screen (ANSI escape).
                eprint!("\x1b[2J\x1b[H");
                eprintln!("[watch] file changed, re-running...\n");
                run_once(path, cli_args, &backend, improve_mode, improve_threshold,
                         improve_min_traces, improve_sample_rate, improve_budget);
            }
        }
    }
}

/// Load, compile, and execute a single .iris file.
#[cfg(feature = "rust-scaffolding")]
fn run_once(
    path: &str,
    cli_args: &[String],
    backend: &str,
    improve_mode: bool,
    improve_threshold: f64,
    improve_min_traces: u64,
    improve_sample_rate: f64,
    improve_budget: u64,
) {
    use iris_exec::registry::FragmentRegistry;

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Cannot read '{}': {}", path, e);
            return;
        }
    };

    // Parse source to AST
    let module = match iris_bootstrap::syntax::parse(&source) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", iris_bootstrap::syntax::format_error(&source, &e));
            return;
        }
    };

    // Lower AST to SemanticGraph fragments
    let mut result = iris_bootstrap::syntax::lower::compile_module(&module);
    if !result.errors.is_empty() {
        for e in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(&source, e));
        }
        return;
    }

    if result.fragments.is_empty() {
        eprintln!("No definitions found in '{}'", path);
        return;
    }

    // Load cached improved fragments (from previous --improve runs).
    // If the cache has a newer version of a function, use it instead of
    // the freshly compiled one. This is what makes generational
    // self-improvement persist across runs.
    let cache_dir = iris_bootstrap::fragment_cache::cache_dir();
    let mut cached_count = 0;
    for (fname, frag, _smap) in result.fragments.iter_mut() {
        if let Some(cached) = iris_bootstrap::fragment_cache::load_fragment(&cache_dir, fname) {
            let cache_gen = iris_bootstrap::fragment_cache::generation(&cache_dir, fname);
            eprintln!("[cache] loaded improved '{}' (gen {}, {})",
                fname,
                cache_gen,
                &iris_bootstrap::fragment_cache::fragment_id_hex(&cached.id)[..16]);
            *frag = cached;
            cached_count += 1;
        }
    }
    if cached_count > 0 {
        eprintln!("[cache] {} function(s) loaded from {}", cached_count, cache_dir.display());
    }

    // Find the "main" fragment, or use the last one defined
    let (name, fragment, _source_map) = result
        .fragments
        .iter()
        .find(|(name, _, _)| name == "main")
        .unwrap_or_else(|| result.fragments.last().unwrap());

    let graph = &fragment.graph;

    // Build a fragment registry so Ref nodes (from let rec) can resolve
    let mut registry = FragmentRegistry::new();
    for (_, frag, _) in &result.fragments {
        registry.register(frag.clone());
    }

    // Parse CLI arguments as interpreter inputs
    let inputs: Vec<Value> = cli_args
        .iter()
        .map(|a| {
            if let Ok(v) = a.parse::<i64>() {
                Value::Int(v)
            } else if let Ok(v) = a.parse::<f64>() {
                Value::Float64(v)
            } else if a == "true" {
                Value::Bool(true)
            } else if a == "false" {
                Value::Bool(false)
            } else {
                Value::String(a.clone())
            }
        })
        .collect();

    // Resolve "auto" backend: try JIT first (if compilable), else tree-walker
    let resolved_backend = if backend == "auto" {
        if iris_exec::jit_backend::is_jit_compilable(graph) {
            "jit"
        } else {
            "tree"
        }
    } else {
        backend
    };

    if improve_mode {
        run_with_improvement(
            name, graph, &inputs, &result, registry,
            improve_threshold, improve_min_traces,
            improve_sample_rate, improve_budget,
        );
    } else if resolved_backend == "jit" || resolved_backend == "aot" {
        // JIT/AOT: try fast native path first, fall back to tree-walker
        if let Some(result) = iris_exec::jit_backend::call_jit_fast(graph, &inputs) {
            println!("{}", format_value(&result));
        } else {
            match iris_exec::jit_backend::interpret_jit(graph, &inputs, Some(&registry)) {
                Ok(outputs) => {
                    for output in &outputs {
                        println!("{}", format_value(output));
                    }
                }
                Err(e) => {
                    eprintln!("Runtime error in '{}': {}", name, e);
                }
            }
        }
    } else if resolved_backend == "clcu" {
        #[cfg(feature = "clcu")]
        {
            if let Some(result) = iris_exec::jit_backend::call_jit_fast(graph, &inputs) {
                println!("{}", format_value(&result));
            } else {
                match iris_exec::jit_backend::interpret_jit(graph, &inputs, Some(&registry)) {
                    Ok(outputs) => {
                        for output in &outputs {
                            println!("{}", format_value(output));
                        }
                    }
                    Err(e) => {
                        eprintln!("Runtime error in '{}': {}", name, e);
                    }
                }
            }
        }
        #[cfg(not(feature = "clcu"))]
        {
            eprintln!("CLCU backend requires --features clcu");
        }
    } else {
        // Standard tree-walker execution
        match iris_exec::interpreter::interpret_with_registry(graph, &inputs, None, Some(&registry)) {
            Ok((outputs, _state)) => {
                for output in &outputs {
                    println!("{}", format_value(output));
                }
            }
            Err(e) => {
                eprintln!("Runtime error in '{}': {}", name, e);
            }
        }
    }
}

/// Run a program with the observation-driven improvement daemon.
#[cfg(feature = "rust-scaffolding")]
fn run_with_improvement(
    name: &str,
    graph: &iris_types::graph::SemanticGraph,
    inputs: &[Value],
    compile_result: &iris_bootstrap::syntax::lower::CompileResult,
    registry: iris_exec::registry::FragmentRegistry,
    threshold: f64,
    min_traces: u64,
    sample_rate: f64,
    evolution_budget_secs: u64,
) {
    use std::sync::{Arc, atomic::AtomicBool};
    use std::collections::BTreeMap;
    use iris_types::trace::{TraceCollector, ImprovementConfig, FunctionId, ImprovementResult};
    use iris_exec::improve::{LiveRegistry, evaluate_and_trace};

    // Build the named graph map and live registry.
    let mut named_graphs = BTreeMap::new();
    for (fname, frag, _) in &compile_result.fragments {
        named_graphs.insert(fname.clone(), frag.graph.clone());
    }
    let live_registry = Arc::new(LiveRegistry::new(named_graphs, registry));
    let collector = Arc::new(TraceCollector::new(sample_rate, 200));

    let config = ImprovementConfig {
        min_traces,
        max_slowdown: threshold,
        evolution_budget_secs,
        population_size: 32,
        max_generations: 200,
        max_test_cases: 20,
        scan_interval_secs: 5,
    };

    // Start the improvement daemon thread.
    let stop = Arc::new(AtomicBool::new(false));
    let daemon_stop = Arc::clone(&stop);
    let daemon_collector = Arc::clone(&collector);
    let daemon_registry = Arc::clone(&live_registry);
    let daemon_config = config.clone();

    let daemon_handle = std::thread::spawn(move || {
        improvement_daemon_loop(
            daemon_config,
            daemon_collector,
            daemon_registry,
            daemon_stop,
        );
    });

    // Execute the main program (with tracing).
    let reg_snapshot = live_registry.snapshot_registry();
    match evaluate_and_trace(name, graph, inputs, &reg_snapshot, &collector) {
        Ok((outputs, _state)) => {
            for output in &outputs {
                println!("{}", format_value(output));
            }
        }
        Err(e) => {
            eprintln!("Runtime error in '{}': {}", name, e);
        }
    }

    // Stop the daemon.
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = daemon_handle.join();

    // Report improvements.
    let improvements = live_registry.improvements();
    if !improvements.is_empty() {
        eprintln!("\n[improve] {} improvement(s) deployed:", improvements.len());
        for imp in &improvements {
            eprintln!(
                "  {} — {:.1}µs → {:.1}µs",
                imp.fn_id.0,
                imp.old_latency_ns as f64 / 1000.0,
                imp.new_latency_ns.unwrap_or(0) as f64 / 1000.0,
            );
        }
    }
}

/// The improvement daemon loop (runs in a background thread).
#[cfg(feature = "rust-scaffolding")]
fn improvement_daemon_loop(
    config: iris_types::trace::ImprovementConfig,
    collector: std::sync::Arc<iris_types::trace::TraceCollector>,
    live_registry: std::sync::Arc<iris_exec::improve::LiveRegistry>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    use std::time::{Duration, Instant};
    use iris_types::trace::{FunctionId, ImprovementResult};

    eprintln!(
        "[improve] daemon started: min_traces={}, threshold={:.1}x, budget={}s",
        config.min_traces, config.max_slowdown, config.evolution_budget_secs,
    );

    let scan_interval = Duration::from_secs(config.scan_interval_secs);
    let mut attempts = 0u64;
    let mut deployed = 0u64;

    while !stop.load(std::sync::atomic::Ordering::Relaxed) {
        std::thread::sleep(scan_interval);

        let ready = collector.ready_functions(config.min_traces);
        if ready.is_empty() {
            continue;
        }

        for fn_id in &ready {
            if stop.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            let _current_graph = match live_registry.get_graph(&fn_id.0) {
                Some(g) => g,
                None => continue,
            };

            let test_cases = collector.build_test_cases(fn_id, config.max_test_cases);
            if test_cases.len() < 3 {
                continue;
            }

            let old_latency = collector.avg_latency_ns(fn_id);
            attempts += 1;

            eprintln!(
                "[improve] attempting {} ({} test cases, avg {:.1}µs)",
                fn_id.0, test_cases.len(), old_latency as f64 / 1000.0,
            );

            // Evolve a replacement.
            let graph_map = live_registry.snapshot_graph_map();
            let exec_service = iris_exec::service::IrisExecutionService::with_defaults();

            let evo_config = iris_evolve::config::EvolutionConfig {
                population_size: config.population_size,
                max_generations: config.max_generations,
                ..Default::default()
            };

            let spec = iris_evolve::config::ProblemSpec {
                test_cases: test_cases.clone(),
                description: format!("improve {}", fn_id.0),
                target_cost: None,
            };

            let evo_result = iris_evolve::evolve(evo_config, spec, &exec_service);

            if evo_result.best_individual.fitness.correctness() < 1.0 {
                eprintln!("[improve] ✗ {} — no correct candidate found", fn_id.0);
                continue;
            }

            let candidate = &evo_result.best_individual.fragment.graph;

            // Equivalence gate: test against ALL traces.
            let all_cases = collector.build_test_cases(fn_id, 200);
            let mut passes = true;
            for tc in &all_cases {
                let expected = match &tc.expected_output {
                    Some(v) => v,
                    None => continue,
                };
                match iris_bootstrap::evaluate_with_registry(candidate, &tc.inputs, 100_000, &graph_map) {
                    Ok(val) => {
                        if vec![val] != *expected {
                            passes = false;
                            break;
                        }
                    }
                    Err(_) => { passes = false; break; }
                }
            }

            if !passes {
                eprintln!("[improve] ✗ {} — failed equivalence gate", fn_id.0);
                continue;
            }

            // Performance gate: measure candidate latency.
            let mut total_ns = 0u64;
            let mut count = 0u64;
            for tc in all_cases.iter().take(20) {
                let start = Instant::now();
                let _ = iris_bootstrap::evaluate_with_registry(candidate, &tc.inputs, 100_000, &graph_map);
                total_ns += start.elapsed().as_nanos() as u64;
                count += 1;
            }
            let new_latency = if count > 0 { total_ns / count } else { 0 };
            let max_allowed = (old_latency as f64 * config.max_slowdown) as u64;

            if new_latency > max_allowed && old_latency > 0 {
                eprintln!(
                    "[improve] ✗ {} — too slow ({:.1}µs > {:.1}µs limit)",
                    fn_id.0, new_latency as f64 / 1000.0, max_allowed as f64 / 1000.0,
                );
                continue;
            }

            // Deploy!
            let result = ImprovementResult {
                fn_id: fn_id.clone(),
                success: true,
                old_latency_ns: old_latency,
                new_latency_ns: Some(new_latency),
                test_cases_used: test_cases.len(),
                generations_run: config.max_generations,
            };

            live_registry.swap(&fn_id.0, candidate.clone(), result);
            deployed += 1;

            // Persist the improved fragment to the cache so it survives restarts.
            // This is what makes generational self-improvement work.
            let improved_frag = evo_result.best_individual.fragment.clone();
            let cache_dir = iris_bootstrap::fragment_cache::cache_dir();
            match iris_bootstrap::fragment_cache::save_fragment(&cache_dir, &fn_id.0, &improved_frag) {
                Ok(hex) => {
                    let cache_gen = iris_bootstrap::fragment_cache::generation(&cache_dir, &fn_id.0);
                    eprintln!(
                        "[cache] saved '{}' gen {} -> {}",
                        fn_id.0, cache_gen, &hex[..16],
                    );
                }
                Err(e) => eprintln!("[cache] failed to save '{}': {}", fn_id.0, e),
            }

            eprintln!(
                "[improve] ✓ deployed {} ({:.1}µs → {:.1}µs)",
                fn_id.0,
                old_latency as f64 / 1000.0,
                new_latency as f64 / 1000.0,
            );
        }
    }

    eprintln!(
        "[improve] daemon stopped: {} attempts, {} deployed",
        attempts, deployed,
    );
}

#[cfg(feature = "rust-scaffolding")]
fn cmd_deploy(_args: &[String]) {
    eprintln!("'iris deploy' now uses IRIS programs directly.");
    eprintln!("See: src/iris-programs/deploy/deploy_orchestrate.iris");
    eprintln!("Run: iris run src/iris-programs/deploy/deploy_orchestrate.iris");
    process::exit(1);
}

#[cfg(feature = "rust-scaffolding")]
fn cmd_compile(_args: &[String]) {
    eprintln!("'iris compile' now uses IRIS programs directly.");
    eprintln!("See: src/iris-programs/compiler/compile_pipeline.iris");
    eprintln!("     src/iris-programs/deploy/standalone.iris");
    process::exit(1);
}

#[cfg(feature = "rust-scaffolding")]
fn cmd_solve(args: &[String]) {
    let mut population_size: usize = 64;
    let mut max_generations: usize = 500;
    let mut positional: Vec<&String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--population" => {
                i += 1;
                population_size = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(64);
            }
            "--generations" => {
                i += 1;
                max_generations = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(500);
            }
            _ => positional.push(&args[i]),
        }
        i += 1;
    }

    if positional.is_empty() {
        eprintln!("Usage: iris solve [--population N] [--generations N] <spec.iris>");
        process::exit(1);
    }

    let path = positional[0];
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Cannot read '{}': {}", path, e);
        process::exit(1);
    });

    let test_cases = parse_test_comments(&source);
    if test_cases.is_empty() {
        eprintln!("No test cases found. Add lines like:");
        eprintln!("  -- test: 5 -> 25");
        eprintln!("  -- test: (3, 4) -> 7");
        process::exit(1);
    }

    let compile_result = iris_bootstrap::syntax::compile(&source);
    if !compile_result.errors.is_empty() {
        for e in &compile_result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(&source, e));
        }
        process::exit(1);
    }

    eprintln!(
        "Solving with {} test cases (population={}, generations={})...",
        test_cases.len(), population_size, max_generations,
    );

    use iris_evolve::config::{EvolutionConfig, ProblemSpec};
    use iris_exec::service::IrisExecutionService;

    let spec = ProblemSpec {
        test_cases,
        description: format!("solve {}", path),
        target_cost: None,
    };

    let exec = IrisExecutionService::with_defaults();
    let config = EvolutionConfig {
        population_size,
        max_generations,
        ..Default::default()
    };

    let result = iris_evolve::evolve(config, spec, &exec);

    let best = &result.best_individual;
    let correctness = best.fitness.correctness();
    let performance = best.fitness.performance();
    let nodes = best.fragment.graph.nodes.len();
    let edges = best.fragment.graph.edges.len();

    if correctness >= 1.0 {
        eprintln!("Solution found in {} generations ({:.1?})!", result.generations_run, result.total_time);
        eprintln!("  fitness: correctness={:.4}, performance={:.4}", correctness, performance);
        eprintln!("  graph: {} nodes, {} edges", nodes, edges);
    } else {
        eprintln!("No perfect solution found after {} generations ({:.1?}).", result.generations_run, result.total_time);
        eprintln!("  best fitness: correctness={:.4}, performance={:.4}", correctness, performance);
        eprintln!("  graph: {} nodes, {} edges", nodes, edges);
    }
}

/// Parse `-- test: input -> output` comment lines from IRIS source.
#[cfg(feature = "rust-scaffolding")]
fn parse_test_comments(source: &str) -> Vec<iris_types::eval::TestCase> {
    let mut cases = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("-- test:") {
            let rest = rest.trim();
            if let Some((input_str, output_str)) = rest.split_once("->") {
                let inputs = parse_value_list(input_str.trim());
                let outputs = parse_value_list(output_str.trim());
                cases.push(iris_types::eval::TestCase {
                    inputs,
                    expected_output: Some(outputs),
                    initial_state: None,
                    expected_state: None,
                });
            }
        }
    }
    cases
}

#[cfg(feature = "rust-scaffolding")]
fn parse_value_list(s: &str) -> Vec<Value> {
    let s = s.trim();
    if s.starts_with('(') && s.ends_with(')') {
        let inner = &s[1..s.len() - 1];
        return split_top_level(inner)
            .iter()
            .map(|part| parse_single_value(part.trim()))
            .collect();
    }
    vec![parse_single_value(s)]
}

#[cfg(feature = "rust-scaffolding")]
fn split_top_level(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => { parts.push(&s[start..i]); start = i + 1; }
            _ => {}
        }
    }
    if start < s.len() { parts.push(&s[start..]); }
    parts
}

#[cfg(feature = "rust-scaffolding")]
fn parse_single_value(s: &str) -> Value {
    let s = s.trim();
    if s == "true" { return Value::Bool(true); }
    if s == "false" { return Value::Bool(false); }
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        return Value::Bytes(s[1..s.len()-1].as_bytes().to_vec());
    }
    if s.contains('.') {
        if let Ok(f) = s.parse::<f64>() { return Value::Float64(f); }
    }
    if let Ok(i) = s.parse::<i64>() { return Value::Int(i); }
    Value::Bytes(s.as_bytes().to_vec())
}

// ---------------------------------------------------------------------------
// cmd_check: Type-check / verify a program
// ---------------------------------------------------------------------------

#[cfg(feature = "rust-scaffolding")]
fn cmd_check(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: iris check <file.iris>");
        process::exit(1);
    }

    let path = &args[0];
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Cannot read '{}': {}", path, e);
        process::exit(1);
    });

    // Parse first, then verify each fragment at its auto-detected tier.
    let result = iris_bootstrap::syntax::compile(&source);

    if !result.errors.is_empty() {
        for e in &result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(&source, e));
        }
        process::exit(1);
    }

    if result.fragments.is_empty() {
        eprintln!("No definitions found in '{}'", path);
        process::exit(1);
    }

    // Print per-fragment verification results
    let mut all_ok = true;
    for (name, fragment, _source_map) in &result.fragments {
        // Auto-detect the minimum tier needed for this fragment's nodes.
        let tier = iris_bootstrap::syntax::kernel::checker::minimum_tier(&fragment.graph);
        let report = iris_bootstrap::syntax::kernel::checker::type_check_graded(
            &fragment.graph,
            tier,
        );
        if report.failed.is_empty() {
            println!(
                "[OK] {}: {}/{} obligations satisfied (score: {:.2})",
                name, report.satisfied, report.total_obligations, report.score
            );
        } else {
            all_ok = false;
            println!(
                "[FAIL] {}: {}/{} obligations satisfied (score: {:.2})",
                name, report.satisfied, report.total_obligations, report.score
            );
            for (node_id, err) in &report.failed {
                println!("  - node {:?}: {}", node_id, err);
            }
        }
    }

    if all_ok {
        println!("All {} definitions verified.", result.fragments.len());
    } else {
        process::exit(1);
    }
}


// ---------------------------------------------------------------------------
// cmd_lint: Static analysis
// ---------------------------------------------------------------------------

#[cfg(feature = "rust-scaffolding")]
fn cmd_lint(args: &[String]) {
    use iris_bootstrap::syntax::ast::{Expr, Item, LetDecl, Pattern};
    use std::collections::HashSet;
    use std::path::Path;

    if args.is_empty() {
        eprintln!("Usage: iris lint <file.iris>");
        process::exit(1);
    }

    let path_str = &args[0];
    let source = fs::read_to_string(path_str).unwrap_or_else(|e| {
        eprintln!("Cannot read '{}': {}", path_str, e);
        process::exit(1);
    });

    // Parse AST for name-level analysis.
    let module = match iris_bootstrap::syntax::parse(&source) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", iris_bootstrap::syntax::format_error(&source, &e));
            process::exit(1);
        }
    };

    // Compile for graph-level analysis.
    let compile_result = iris_bootstrap::syntax::compile_file(
        &source,
        Path::new(path_str),
    );
    if !compile_result.errors.is_empty() {
        for e in &compile_result.errors {
            eprintln!("{}", iris_bootstrap::syntax::format_error(&source, e));
        }
        process::exit(1);
    }

    struct LintWarning {
        code: &'static str,
        message: String,
        line: usize,
    }

    let mut warnings: Vec<LintWarning> = Vec::new();

    // Helper: compute 1-based line number from a byte offset.
    let line_of = |offset: usize| -> usize {
        source[..offset.min(source.len())]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
            + 1
    };

    // ---------------------------------------------------------------
    // Helper: collect all free variable names in an expression.
    // ---------------------------------------------------------------
    fn free_vars(expr: &Expr, bound: &HashSet<String>, out: &mut HashSet<String>) {
        match expr {
            Expr::Var(name, _) => {
                if !bound.contains(name) {
                    out.insert(name.clone());
                }
            }
            Expr::IntLit(..) | Expr::FloatLit(..) | Expr::BoolLit(..)
            | Expr::StringLit(..) | Expr::UnitLit(..) => {}
            Expr::Tuple(elems, _) => {
                for e in elems {
                    free_vars(e, bound, out);
                }
            }
            Expr::TupleAccess(e, _, _) => free_vars(e, bound, out),
            Expr::App(f, a, _) => {
                free_vars(f, bound, out);
                free_vars(a, bound, out);
            }
            Expr::BinOp(l, _, r, _) => {
                free_vars(l, bound, out);
                free_vars(r, bound, out);
            }
            Expr::UnaryOp(_, e, _) => free_vars(e, bound, out),
            Expr::OpSection(..) => {}
            Expr::Lambda(params, body, _) => {
                let mut inner = bound.clone();
                for p in params {
                    inner.insert(p.clone());
                }
                free_vars(body, &inner, out);
            }
            Expr::Let(name, val, body, _) | Expr::LetRec(name, val, body, _) => {
                free_vars(val, bound, out);
                let mut inner = bound.clone();
                inner.insert(name.clone());
                free_vars(body, &inner, out);
            }
            Expr::If(c, t, e, _) => {
                free_vars(c, bound, out);
                free_vars(t, bound, out);
                free_vars(e, bound, out);
            }
            Expr::Match(scrut, arms, _) => {
                free_vars(scrut, bound, out);
                for arm in arms {
                    let mut inner = bound.clone();
                    pattern_binders(&arm.pattern, &mut inner);
                    if let Some(ref g) = arm.guard {
                        free_vars(g, &inner, out);
                    }
                    free_vars(&arm.body, &inner, out);
                }
            }
            Expr::Pipe(l, r, _) => {
                free_vars(l, bound, out);
                free_vars(r, bound, out);
            }
            Expr::RecordLit(fields, _) => {
                for (_, v) in fields {
                    free_vars(v, bound, out);
                }
            }
            Expr::FieldAccess(e, _, _) => free_vars(e, bound, out),
        }
    }

    fn pattern_binders(pat: &Pattern, out: &mut HashSet<String>) {
        match pat {
            Pattern::Ident(name, _) => { out.insert(name.clone()); }
            Pattern::Constructor(_, Some(inner), _) => pattern_binders(inner, out),
            Pattern::Tuple(pats, _) => {
                for p in pats {
                    pattern_binders(p, out);
                }
            }
            _ => {}
        }
    }

    // ---------------------------------------------------------------
    // Helper: check for unused bindings & shadowed names in an expr.
    // ---------------------------------------------------------------
    fn lint_expr(
        expr: &Expr,
        scope: &HashSet<String>,
        warnings: &mut Vec<LintWarning>,
        line_of: &dyn Fn(usize) -> usize,
    ) {
        match expr {
            Expr::Let(name, val, body, span) | Expr::LetRec(name, val, body, span) => {
                // L002: shadowed name
                if name != "_" && scope.contains(name) {
                    warnings.push(LintWarning {
                        code: "L002",
                        message: format!("shadowed binding '{}'", name),
                        line: line_of(span.start),
                    });
                }

                // L001: unused binding
                if name != "_" {
                    let mut body_fv = HashSet::new();
                    let bound: HashSet<String> = HashSet::new();
                    free_vars(body, &bound, &mut body_fv);
                    if !body_fv.contains(name) {
                        warnings.push(LintWarning {
                            code: "L001",
                            message: format!("unused binding '{}'", name),
                            line: line_of(span.start),
                        });
                    }
                }

                // Recurse into value and body.
                lint_expr(val, scope, warnings, line_of);
                let mut inner_scope = scope.clone();
                inner_scope.insert(name.clone());
                lint_expr(body, &inner_scope, warnings, line_of);
            }
            Expr::Lambda(params, body, _) => {
                let mut inner = scope.clone();
                for p in params {
                    inner.insert(p.clone());
                }
                lint_expr(body, &inner, warnings, line_of);
            }
            Expr::App(f, a, _) => {
                // L005: constant fold -- fold base (\acc_param ... -> body)
                // Detect pattern: App(App(App(Var("fold"), base), step_fn), coll)
                // or App(App(Var("fold"), base), step_fn) for 2-arg fold.
                // The step_fn is the interesting part.
                lint_check_fold(expr, warnings, line_of);
                lint_expr(f, scope, warnings, line_of);
                lint_expr(a, scope, warnings, line_of);
            }
            Expr::BinOp(l, _, r, _) | Expr::Pipe(l, r, _) => {
                lint_expr(l, scope, warnings, line_of);
                lint_expr(r, scope, warnings, line_of);
            }
            Expr::UnaryOp(_, e, _) | Expr::TupleAccess(e, _, _)
            | Expr::FieldAccess(e, _, _) => {
                lint_expr(e, scope, warnings, line_of);
            }
            Expr::Tuple(elems, _) => {
                for e in elems {
                    lint_expr(e, scope, warnings, line_of);
                }
            }
            Expr::If(c, t, e, _) => {
                lint_expr(c, scope, warnings, line_of);
                lint_expr(t, scope, warnings, line_of);
                lint_expr(e, scope, warnings, line_of);
            }
            Expr::Match(scrut, arms, _) => {
                lint_expr(scrut, scope, warnings, line_of);
                for arm in arms {
                    let mut inner = scope.clone();
                    pattern_binders(&arm.pattern, &mut inner);
                    if let Some(ref g) = arm.guard {
                        lint_expr(g, &inner, warnings, line_of);
                    }
                    lint_expr(&arm.body, &inner, warnings, line_of);
                }
            }
            Expr::RecordLit(fields, _) => {
                for (_, v) in fields {
                    lint_expr(v, scope, warnings, line_of);
                }
            }
            _ => {}
        }
    }

    // ---------------------------------------------------------------
    // L005: constant fold detection.
    // Matches fold applied to a lambda whose first parameter (the
    // accumulator) is never referenced in the lambda body.
    // ---------------------------------------------------------------
    fn lint_check_fold(
        expr: &Expr,
        warnings: &mut Vec<LintWarning>,
        line_of: &dyn Fn(usize) -> usize,
    ) {
        // Collect the arguments of a curried application spine rooted at "fold".
        fn collect_fold_args<'a>(e: &'a Expr) -> Option<Vec<&'a Expr>> {
            match e {
                Expr::App(f, arg, _) => {
                    if let Some(mut args) = collect_fold_args(f) {
                        args.push(arg);
                        Some(args)
                    } else {
                        None
                    }
                }
                Expr::Var(name, _) if name == "fold" => Some(Vec::new()),
                _ => None,
            }
        }

        if let Some(args) = collect_fold_args(expr) {
            // args[0] = base, args[1] = step_fn, args[2] = collection (optional)
            if args.len() >= 2 {
                if let Expr::Lambda(params, body, span) = args[1] {
                    if let Some(acc_name) = params.first() {
                        if acc_name != "_" {
                            let mut body_fv = HashSet::new();
                            let bound: HashSet<String> = HashSet::new();
                            free_vars(body, &bound, &mut body_fv);
                            if !body_fv.contains(acc_name) {
                                warnings.push(LintWarning {
                                    code: "L005",
                                    message: format!(
                                        "fold step function ignores accumulator '{}'",
                                        acc_name
                                    ),
                                    line: line_of(span.start),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------------
    // Walk top-level items.
    // ---------------------------------------------------------------

    // Collect top-level names for shadow detection in nested lets.
    let mut top_scope: HashSet<String> = HashSet::new();
    for item in &module.items {
        match item {
            Item::LetDecl(decl) => { top_scope.insert(decl.name.clone()); }
            Item::MutualRecGroup(decls) => {
                for d in decls { top_scope.insert(d.name.clone()); }
            }
            _ => {}
        }
    }

    let lint_decl = |decl: &LetDecl, warnings: &mut Vec<LintWarning>| {
        // L004: missing type annotation on top-level function.
        if decl.ret_type.is_none() && !decl.params.is_empty() {
            warnings.push(LintWarning {
                code: "L004",
                message: format!(
                    "function '{}' has no return type annotation",
                    decl.name
                ),
                line: line_of(decl.span.start),
            });
        }

        // Build scope with params for body analysis.
        let mut scope = top_scope.clone();
        for p in &decl.params {
            scope.insert(p.clone());
        }
        lint_expr(&decl.body, &scope, warnings, &line_of);

        // L001: check for unused parameters.
        if !decl.params.is_empty() {
            let mut body_fv = HashSet::new();
            let bound: HashSet<String> = HashSet::new();
            free_vars(&decl.body, &bound, &mut body_fv);
            for p in &decl.params {
                if p != "_" && !body_fv.contains(p) {
                    warnings.push(LintWarning {
                        code: "L001",
                        message: format!(
                            "unused parameter '{}' in function '{}'",
                            p, decl.name
                        ),
                        line: line_of(decl.span.start),
                    });
                }
            }
        }
    };

    for item in &module.items {
        match item {
            Item::LetDecl(decl) => lint_decl(decl, &mut warnings),
            Item::MutualRecGroup(decls) => {
                for d in decls {
                    lint_decl(d, &mut warnings);
                }
            }
            _ => {}
        }
    }

    // ---------------------------------------------------------------
    // L003: large graphs (graph-level check on compiled fragments).
    // ---------------------------------------------------------------
    for (name, fragment, source_map) in &compile_result.fragments {
        let node_count = fragment.graph.nodes.len();
        if node_count > 200 {
            // Use the root node span if available, otherwise line 1.
            let line = source_map
                .get(&fragment.graph.root)
                .map(|s| line_of(s.start))
                .unwrap_or(1);
            warnings.push(LintWarning {
                code: "L003",
                message: format!(
                    "fragment '{}' has {} nodes (threshold: 200)",
                    name, node_count
                ),
                line,
            });
        }
    }

    // ---------------------------------------------------------------
    // Print results (deduplicated).
    // ---------------------------------------------------------------
    warnings.sort_by_key(|w| (w.line, w.code));
    warnings.dedup_by(|a, b| a.code == b.code && a.line == b.line && a.message == b.message);

    if warnings.is_empty() {
        println!("No warnings. {} definition(s) checked.", compile_result.fragments.len());
    } else {
        for w in &warnings {
            println!(
                "warning[{}]: {} at line {}",
                w.code, w.message, w.line
            );
        }
        println!(
            "\n{} warning(s) in {} definition(s).",
            warnings.len(),
            compile_result.fragments.len()
        );
    }
}

// ---------------------------------------------------------------------------
// cmd_daemon: Continuous self-improvement loop
// (does NOT require iris-syntax -- works with SemanticGraphs directly)
// ---------------------------------------------------------------------------

#[cfg(feature = "rust-scaffolding")]
fn cmd_daemon(args: &[String]) {
    use iris_evolve::self_improving_daemon::ExecMode;

    let mut max_cycles: Option<u64> = None;
    let mut exec_mode = ExecMode::FixedInterval(std::time::Duration::from_millis(800));
    let mut max_slowdown = 2.0;
    let mut max_stagnant = 5u32;
    let mut max_improve_threads = 2usize;

    // Parse CLI arguments.
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--max-cycles" => {
                i += 1;
                max_cycles = args.get(i).and_then(|a| a.parse().ok());
            }
            "--exec-mode" => {
                i += 1;
                if let Some(mode_str) = args.get(i) {
                    if mode_str == "continuous" {
                        exec_mode = ExecMode::Continuous;
                    } else if let Some(ms_str) = mode_str.strip_prefix("interval:") {
                        if let Ok(ms) = ms_str.parse::<u64>() {
                            exec_mode = ExecMode::FixedInterval(
                                std::time::Duration::from_millis(ms),
                            );
                        }
                    }
                }
            }
            "--improve-threshold" => {
                i += 1;
                if let Some(val) = args.get(i).and_then(|a| a.parse().ok()) {
                    max_slowdown = val;
                }
            }
            "--max-stagnant" => {
                i += 1;
                if let Some(val) = args.get(i).and_then(|a| a.parse().ok()) {
                    max_stagnant = val;
                }
            }
            "--max-improve-threads" => {
                i += 1;
                if let Some(val) = args.get(i).and_then(|a| a.parse().ok()) {
                    max_improve_threads = val;
                }
            }
            other => {
                // Positional: try as max_cycles for backward compat.
                if max_cycles.is_none() {
                    max_cycles = other.parse().ok();
                }
            }
        }
        i += 1;
    }

    // Determine state directory for persistence.
    let state_dir = std::env::current_dir()
        .ok()
        .map(|d| d.join(".iris-daemon"));

    let config = iris_evolve::self_improving_daemon::SelfImprovingConfig {
        cycle_time_ms: 800,
        max_cycles,
        improve_interval: 10,
        inspect_interval: 5,
        auto_improve: iris_evolve::auto_improve::AutoImproveConfig {
            cycle_interval_secs: 0,
            max_slowdown,
            test_cases_per_component: 10,
            evolution_generations: 50,
            evolution_pop_size: 32,
            gate_runs: 3,
            explore_problems: 5,
        },
        state_dir,
        memory_limit: 50 * 1024 * 1024, // 50 MB
        seed: None,
        max_improve_threads,
        max_stagnant,
        min_improvement: 0.05,
        exec_mode,
        trigger_check_interval: 100,
    };

    eprintln!("Starting IRIS threaded self-improving daemon...");
    eprintln!(
        "  max_improve_threads={}, max_stagnant={}, max_slowdown={:.1}x",
        max_improve_threads, max_stagnant, max_slowdown
    );
    if let Some(max) = max_cycles {
        eprintln!("  Will run {} cycles.", max);
    } else {
        eprintln!("  Press Ctrl+C to stop.");
    }

    let mut daemon = iris_evolve::self_improving_daemon::SelfImprovingDaemon::new(config);
    let result = daemon.run();
    eprintln!(
        "Daemon stopped after {} cycles ({:.2}s): {} improvement cycles, {} deployed, \
         {} audit entries, {} converged, fully_converged={}",
        result.cycles_completed,
        result.total_time.as_secs_f64(),
        result.improvement_cycles,
        result.components_deployed,
        result.audit_entries,
        result.converged_components,
        result.fully_converged,
    );
}

// ---------------------------------------------------------------------------
// cmd_repl: Interactive read-eval-print loop
// ---------------------------------------------------------------------------

#[cfg(feature = "rust-scaffolding")]
fn format_type_ref(type_ref: &iris_types::types::TypeRef, env: &iris_types::types::TypeEnv) -> String {
    use iris_types::types::{TypeDef, PrimType};

    match env.types.get(type_ref) {
        None => format!("?{}", type_ref.0),
        Some(td) => match td {
            TypeDef::Primitive(p) => match p {
                PrimType::Int => "Int".to_string(),
                PrimType::Nat => "Nat".to_string(),
                PrimType::Float64 => "Float64".to_string(),
                PrimType::Float32 => "Float32".to_string(),
                PrimType::Bool => "Bool".to_string(),
                PrimType::Bytes => "Bytes".to_string(),
                PrimType::Unit => "()".to_string(),
            },
            TypeDef::Product(fields) => {
                let inner: Vec<String> = fields.iter().map(|f| format_type_ref(f, env)).collect();
                format!("({})", inner.join(", "))
            }
            TypeDef::Sum(variants) => {
                let inner: Vec<String> = variants
                    .iter()
                    .map(|(tag, tid)| format!("#{} {}", tag.0, format_type_ref(tid, env)))
                    .collect();
                inner.join(" | ")
            }
            TypeDef::Arrow(from, to, _cost) => {
                format!("{} -> {}", format_type_ref(from, env), format_type_ref(to, env))
            }
            TypeDef::Recursive(bv, body) => {
                format!("mu v{}. {}", bv.0, format_type_ref(body, env))
            }
            TypeDef::ForAll(bv, body) => {
                format!("forall v{}. {}", bv.0, format_type_ref(body, env))
            }
            TypeDef::Refined(base, _pred) => {
                format!("{{{} | ...}}", format_type_ref(base, env))
            }
            TypeDef::Vec(elem, _size) => {
                format!("Vec<{}>", format_type_ref(elem, env))
            }
            TypeDef::Exists(bv, body) => {
                format!("exists v{}. {}", bv.0, format_type_ref(body, env))
            }
            TypeDef::NeuralGuard(inp, out, _spec, _cost) => {
                format!("Neural({} -> {})", format_type_ref(inp, env), format_type_ref(out, env))
            }
            TypeDef::HWParam(base, _profile) => {
                format!("HW({})", format_type_ref(base, env))
            }
        },
    }
}

#[cfg(feature = "rust-scaffolding")]
fn repl_compile_and_eval(
    source: &str,
    accumulated: &[(String, iris_types::fragment::Fragment)],
) -> Result<
    (
        Vec<(String, iris_types::fragment::Fragment)>,
        Vec<iris_types::eval::Value>,
    ),
    String,
> {
    use iris_exec::registry::FragmentRegistry;

    // Build a combined source from all accumulated definitions + new input.
    let mut full_source = String::new();
    for (def_source, _) in accumulated {
        full_source.push_str(def_source);
        full_source.push('\n');
    }
    full_source.push_str(source);

    let module = match iris_bootstrap::syntax::parse(&full_source) {
        Ok(m) => m,
        Err(e) => {
            return Err(iris_bootstrap::syntax::format_error(&full_source, &e));
        }
    };

    let result = iris_bootstrap::syntax::lower::compile_module(&module);
    if !result.errors.is_empty() {
        let mut msg = String::new();
        for e in &result.errors {
            msg.push_str(&iris_bootstrap::syntax::format_error(&full_source, e));
            msg.push('\n');
        }
        return Err(msg);
    }

    if result.fragments.is_empty() {
        return Err("(no result)".to_string());
    }

    // Build the new fragments list from this compile result.
    let new_fragments: Vec<(String, iris_types::fragment::Fragment)> = result
        .fragments
        .iter()
        .map(|(name, frag, _)| (name.clone(), frag.clone()))
        .collect();

    let (_, eval_fragment, _) = result.fragments.last().unwrap();

    let mut reg = FragmentRegistry::new();
    for (_, frag, _) in &result.fragments {
        reg.register(frag.clone());
    }

    match iris_exec::interpreter::interpret_with_registry(
        &eval_fragment.graph,
        &[],
        None,
        Some(&reg),
    ) {
        Ok((outputs, _)) => Ok((new_fragments, outputs)),
        Err(e) => Err(format!("Error: {}", e)),
    }
}

#[cfg(feature = "rust-scaffolding")]
fn cmd_repl() {
    use rustyline::error::ReadlineError;
    use iris_types::fragment::Fragment;

    // Ensure ~/.iris/ directory exists for history file.
    let history_path = dirs_or_home().join("repl_history");

    let mut rl = rustyline::DefaultEditor::new().unwrap_or_else(|e| {
        eprintln!("Failed to initialize editor: {}", e);
        process::exit(1);
    });
    let _ = rl.load_history(&history_path);

    println!("IRIS REPL v0.1.0");
    println!("Type expressions or :help for commands.");
    println!();

    // Accumulated state: (source_text, compiled_fragment) pairs keyed by name.
    // We store the source text of each `let` so we can reconstruct the full
    // source on each iteration.
    let mut definitions: Vec<(String, String, Fragment)> = Vec::new(); // (name, source, fragment)

    loop {
        let readline = rl.readline("iris> ");
        let line = match readline {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C: cancel current input, continue
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D: exit
                break;
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        };

        // Multi-line support: if line ends with `\`, keep reading.
        let mut input = line.clone();
        while input.ends_with('\\') {
            input.pop(); // remove trailing backslash
            input.push('\n');
            match rl.readline("  ... ") {
                Ok(cont) => input.push_str(&cont),
                Err(ReadlineError::Interrupted) => {
                    input.clear();
                    break;
                }
                Err(_) => break,
            }
        }

        let input = input.trim().to_string();
        if input.is_empty() {
            continue;
        }

        let _ = rl.add_history_entry(&input);

        // Handle REPL commands
        if input.starts_with(':') {
            let parts: Vec<&str> = input.splitn(2, char::is_whitespace).collect();
            match parts[0] {
                ":quit" | ":q" | ":exit" => break,
                ":help" | ":h" | ":?" => {
                    println!("REPL commands:");
                    println!("  :quit, :q, :exit   Exit the REPL");
                    println!("  :type <expr>       Show the type of an expression");
                    println!("  :list              Show all defined names");
                    println!("  :clear             Reset accumulated state");
                    println!("  :load <file>       Load an .iris file into the session");
                    println!("  :help, :h, :?      Show this help");
                    println!();
                    println!("Lines ending with \\ continue on the next line.");
                    continue;
                }
                ":list" | ":ls" => {
                    if definitions.is_empty() {
                        println!("(no definitions)");
                    } else {
                        for (name, _source, frag) in &definitions {
                            let type_str = if !frag.boundary.outputs.is_empty() {
                                let (_, tref) = &frag.boundary.outputs[0];
                                format_type_ref(tref, &frag.type_env)
                            } else {
                                "?".to_string()
                            };
                            println!("  {} : {}", name, type_str);
                        }
                    }
                    continue;
                }
                ":clear" => {
                    definitions.clear();
                    println!("State cleared.");
                    continue;
                }
                ":type" | ":t" => {
                    let expr = parts.get(1).map(|s| s.trim()).unwrap_or("");
                    if expr.is_empty() {
                        eprintln!("Usage: :type <expression>");
                        continue;
                    }
                    // Compile the expression to get its type.
                    let source = if expr.starts_with("let ") {
                        expr.to_string()
                    } else {
                        format!("let it = {}", expr)
                    };
                    let accumulated: Vec<(String, Fragment)> = definitions
                        .iter()
                        .map(|(_, src, frag)| (src.clone(), frag.clone()))
                        .collect();
                    match repl_compile_source_only(&source, &accumulated) {
                        Ok(fragments) => {
                            if let Some((name, frag)) = fragments.last() {
                                if !frag.boundary.outputs.is_empty() {
                                    let (_, tref) = &frag.boundary.outputs[0];
                                    let type_str = format_type_ref(tref, &frag.type_env);
                                    println!("{} : {}", name, type_str);
                                } else if !frag.boundary.inputs.is_empty() {
                                    // Function type: show inputs -> output
                                    let inputs: Vec<String> = frag
                                        .boundary
                                        .inputs
                                        .iter()
                                        .map(|(_, tref)| format_type_ref(tref, &frag.type_env))
                                        .collect();
                                    println!("{} : {} -> ?", name, inputs.join(" -> "));
                                } else {
                                    println!("{} : (unknown)", name);
                                }
                            }
                        }
                        Err(e) => eprintln!("{}", e.trim_end()),
                    }
                    continue;
                }
                ":load" | ":l" => {
                    let path = parts.get(1).map(|s| s.trim()).unwrap_or("");
                    if path.is_empty() {
                        eprintln!("Usage: :load <file.iris>");
                        continue;
                    }
                    match fs::read_to_string(path) {
                        Ok(file_source) => {
                            // Parse and compile the file, adding all definitions.
                            let accumulated: Vec<(String, Fragment)> = definitions
                                .iter()
                                .map(|(_, src, frag)| (src.clone(), frag.clone()))
                                .collect();
                            match repl_compile_source_only(&file_source, &accumulated) {
                                Ok(fragments) => {
                                    let count = fragments.len();
                                    for (name, frag) in fragments {
                                        let source_line = format!("let {} = ...", name); // placeholder
                                        // Extract the actual source from the file for this def
                                        upsert_definition(
                                            &mut definitions,
                                            name,
                                            source_line,
                                            frag,
                                        );
                                    }
                                    // Replace stored sources with the actual file content
                                    // by re-extracting: store the whole file as individual
                                    // let declarations.
                                    repl_reload_from_file(
                                        &mut definitions,
                                        &file_source,
                                    );
                                    println!("Loaded {} definition(s) from {}", count, path);
                                }
                                Err(e) => eprintln!("{}", e.trim_end()),
                            }
                        }
                        Err(e) => eprintln!("Cannot read '{}': {}", path, e),
                    }
                    continue;
                }
                other => {
                    eprintln!("Unknown command: {}. Type :help for available commands.", other);
                    continue;
                }
            }
        }

        // Regular input: expression or let binding.
        let source = if input.starts_with("let ") {
            input.clone()
        } else {
            format!("let it = {}", input)
        };

        let accumulated: Vec<(String, Fragment)> = definitions
            .iter()
            .map(|(_, src, frag)| (src.clone(), frag.clone()))
            .collect();

        match repl_compile_and_eval(&source, &accumulated) {
            Ok((new_fragments, outputs)) => {
                // Update definitions with newly compiled fragments.
                // The new_fragments include ALL definitions (accumulated + new).
                // We only need to add/update the ones from the current input.
                let accumulated_names: std::collections::HashSet<String> = definitions
                    .iter()
                    .map(|(name, _, _)| name.clone())
                    .collect();

                for (name, frag) in &new_fragments {
                    if !accumulated_names.contains(name) || source.contains(&format!("let {} ", name)) || source.contains(&format!("let {}\n", name)) {
                        // This is a new or redefined name from the current input.
                        // Extract just the let declaration for this name from source.
                        let def_source = extract_let_source(&source, name);
                        upsert_definition(&mut definitions, name.clone(), def_source, frag.clone());
                    }
                }

                for output in &outputs {
                    println!("{}", format_value(output));
                }
            }
            Err(e) => eprintln!("{}", e.trim_end()),
        }
    }

    // Save history
    let _ = rl.save_history(&history_path);
    println!("Goodbye.");
}

/// Get the ~/.iris/ directory path, creating it if needed.
#[cfg(feature = "rust-scaffolding")]
fn dirs_or_home() -> std::path::PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = std::path::PathBuf::from(home).join(".iris");
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Extract the source text of a specific `let` binding from a multi-definition source.
#[cfg(feature = "rust-scaffolding")]
fn extract_let_source(source: &str, name: &str) -> String {
    // Try to find "let <name>" in the source and grab everything until the next
    // top-level `let` or end of string.
    let needle = format!("let {} ", name);
    let needle_eq = format!("let {}=", name);
    if let Some(start) = source.find(&needle).or_else(|| source.find(&needle_eq)) {
        let rest = &source[start..];
        // Find the next top-level `let` after this one.
        if let Some(next_let) = rest[4..].find("\nlet ") {
            rest[..next_let + 4].trim().to_string()
        } else {
            rest.trim().to_string()
        }
    } else {
        source.trim().to_string()
    }
}

/// Compile source without evaluating (for :type and :load).
#[cfg(feature = "rust-scaffolding")]
fn repl_compile_source_only(
    source: &str,
    accumulated: &[(String, iris_types::fragment::Fragment)],
) -> Result<Vec<(String, iris_types::fragment::Fragment)>, String> {
    // Build full source from accumulated definitions + new source.
    let mut full_source = String::new();
    for (def_source, _) in accumulated {
        full_source.push_str(def_source);
        full_source.push('\n');
    }
    full_source.push_str(source);

    let module = match iris_bootstrap::syntax::parse(&full_source) {
        Ok(m) => m,
        Err(e) => {
            return Err(iris_bootstrap::syntax::format_error(&full_source, &e));
        }
    };

    let result = iris_bootstrap::syntax::lower::compile_module(&module);
    if !result.errors.is_empty() {
        let mut msg = String::new();
        for e in &result.errors {
            msg.push_str(&iris_bootstrap::syntax::format_error(&full_source, e));
            msg.push('\n');
        }
        return Err(msg);
    }

    Ok(result
        .fragments
        .into_iter()
        .map(|(name, frag, _)| (name, frag))
        .collect())
}

/// Insert or replace a definition in the accumulated state.
#[cfg(feature = "rust-scaffolding")]
fn upsert_definition(
    definitions: &mut Vec<(String, String, iris_types::fragment::Fragment)>,
    name: String,
    source: String,
    fragment: iris_types::fragment::Fragment,
) {
    if let Some(existing) = definitions.iter_mut().find(|(n, _, _)| *n == name) {
        existing.1 = source;
        existing.2 = fragment;
    } else {
        definitions.push((name, source, fragment));
    }
}

/// After loading a file, re-extract proper source lines for each definition.
#[cfg(feature = "rust-scaffolding")]
fn repl_reload_from_file(
    definitions: &mut [(String, String, iris_types::fragment::Fragment)],
    file_source: &str,
) {
    for (name, source, _) in definitions.iter_mut() {
        let extracted = extract_let_source(file_source, name);
        if extracted != file_source.trim() && extracted.starts_with("let ") {
            *source = extracted;
        }
    }
}


// ---------------------------------------------------------------------------
// Value formatting
// ---------------------------------------------------------------------------

#[cfg(feature = "rust-scaffolding")]
fn format_value(val: &Value) -> String {
    match val {
        Value::Int(n) => n.to_string(),
        Value::Nat(n) => format!("{}u", n),
        Value::Float64(f) => format!("{}", f),
        Value::Float32(f) => format!("{}f32", f),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        Value::Unit => "()".to_string(),
        Value::Bytes(b) => format!("<{} bytes>", b.len()),
        Value::Tuple(elems) => {
            let inner: Vec<String> = elems.iter().map(format_value).collect();
            format!("({})", inner.join(", "))
        }
        Value::Range(s, e) => format!("[{}..{})", s, e),
        Value::Tagged(tag, inner) => format!("#{} {}", tag, format_value(inner)),
        Value::State(_) => "<state>".to_string(),
        Value::Graph(_) => "<knowledge-graph>".to_string(),
        Value::Program(g) => format!("<program: {} nodes>", g.nodes.len()),
        Value::Future(_) => "<future>".to_string(),
        Value::Thunk(sg, _) => format!("<thunk: {} nodes>", sg.nodes.len()),
    }
}
