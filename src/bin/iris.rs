use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
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
  store <subcommand>              Manage the fragment cache
    list                          List all cached fragments
    get <name>                    Show details of a cached fragment
    rm <name>                     Remove a cached fragment
    clear                         Clear the entire cache
    path                          Print the cache directory path
  explain <error-code>            Explain a compiler error code
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
                    let size = fs::metadata(&frag_file).map(|m| m.len()).unwrap_or(0);
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
