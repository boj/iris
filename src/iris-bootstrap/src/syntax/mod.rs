//! iris-syntax: ML-like surface syntax for the IRIS language.
//!
//! Also contains the `kernel` module (the LCF-style proof kernel),
//! merged from the former `iris-kernel` crate.

pub mod ast;
pub mod diagnostic;
pub mod error;
#[allow(unsafe_code)]
pub mod kernel;
pub mod lexer;
pub mod lower;
pub mod parser;
pub mod prim;
pub mod token;

use error::SyntaxError;
use lower::CompileResult;
use std::path::Path;

pub fn parse(source: &str) -> Result<ast::Module, SyntaxError> {
    let mut lex = lexer::Lexer::new(source);
    let tokens = lex.tokenize()?;
    let mut parser = parser::Parser::new(tokens);
    parser.parse_module()
}

pub fn compile(source: &str) -> CompileResult {
    match parse(source) {
        Ok(module) => lower::compile_module(&module),
        Err(e) => CompileResult { fragments: Vec::new(), errors: vec![e], constructors: std::collections::BTreeMap::new(), adt_types: std::collections::BTreeMap::new() },
    }
}

/// Compile source with path-based import resolution.
/// `source_path` is the path of the file being compiled.
pub fn compile_file(source: &str, source_path: &Path) -> CompileResult {
    match parse(source) {
        Ok(module) => {
            let source_dir = source_path.parent();
            lower::compile_module_with_path(&module, source_dir, &mut std::collections::HashSet::new())
        }
        Err(e) => CompileResult { fragments: Vec::new(), errors: vec![e], constructors: std::collections::BTreeMap::new(), adt_types: std::collections::BTreeMap::new() },
    }
}

pub fn format_error(source: &str, err: &SyntaxError) -> String {
    error::format_error(source, err)
}

/// Compile source and verify each fragment, returning formatted diagnostics.
pub fn compile_and_verify(
    source: &str,
    tier: iris_types::proof::VerifyTier,
) -> (CompileResult, String) {
    let result = compile(source);
    let mut all_diagnostics = String::new();
    for (_, fragment, source_map) in &result.fragments {
        let report = kernel::checker::type_check_graded(&fragment.graph, tier);
        if !report.failed.is_empty() {
            all_diagnostics.push_str(&diagnostic::format_diagnostics(
                source, &report, source_map,
            ));
        }
    }
    (result, all_diagnostics)
}

/// Compile source with mandatory type checking. Returns an error string if any
/// fragment fails type verification. This is the strict pre-execution check.
pub fn compile_checked(
    source: &str,
) -> Result<CompileResult, String> {
    compile_checked_inner(compile(source), source)
}

/// Compile source from a file with mandatory type checking and import resolution.
pub fn compile_checked_file(
    source: &str,
    source_path: &Path,
) -> Result<CompileResult, String> {
    compile_checked_inner(compile_file(source, source_path), source)
}

fn compile_checked_inner(
    result: CompileResult,
    source: &str,
) -> Result<CompileResult, String> {
    if !result.errors.is_empty() {
        return Err(result.errors.iter()
            .map(|e| format!("{:?}", e))
            .collect::<Vec<_>>()
            .join("\n"));
    }
    let mut all_errors = String::new();
    for (name, fragment, source_map) in &result.fragments {
        let tier = classify_tier(&fragment.graph);
        let report = kernel::checker::type_check_graded(&fragment.graph, tier);
        if !report.failed.is_empty() {
            all_errors.push_str(&format!("Type errors in `{}`:\n", name));
            all_errors.push_str(&diagnostic::format_diagnostics(
                source, &report, source_map,
            ));
        }
        // Verify requires/ensures contracts using the LIA solver.
        let num_inputs = fragment.boundary.inputs.len();
        if let Err(e) = kernel::checker::verify_contracts(&fragment.contracts, num_inputs) {
            all_errors.push_str(&format!("Contract violation in `{}`: {}\n", name, e));
        }
        // Collect and report effect usage for transparency.
        let effects = kernel::checker::collect_graph_effects(&fragment.graph);
        if !effects.is_pure() {
            // Effect information is informational at compile_checked level;
            // verify_effects can be called with a declared EffectSet for strict checking.
            let _ = effects;
        }
    }
    if all_errors.is_empty() {
        Ok(result)
    } else {
        Err(all_errors)
    }
}

/// Classify the verification tier for a graph based on its node kinds.
pub fn classify_tier(graph: &iris_types::graph::SemanticGraph) -> iris_types::proof::VerifyTier {
    use iris_types::graph::NodeKind;
    use iris_types::proof::VerifyTier;

    let mut has_fold = false;
    let mut has_letrec = false;
    let mut has_neural = false;

    for node in graph.nodes.values() {
        match node.kind {
            NodeKind::Fold | NodeKind::Unfold => has_fold = true,
            NodeKind::LetRec => has_letrec = true,
            NodeKind::Neural => has_neural = true,
            _ => {}
        }
    }

    if has_neural {
        VerifyTier::Tier2
    } else if has_fold || has_letrec {
        VerifyTier::Tier1
    } else {
        VerifyTier::Tier0
    }
}
