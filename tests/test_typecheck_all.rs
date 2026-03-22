//! Batch type-check: run compile_checked on all .iris programs.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_iris_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_iris_files(&path, out);
            } else if path.extension().map_or(false, |e| e == "iris") {
                out.push(path);
            }
        }
    }
}

fn categorize_error(err: &iris_bootstrap::syntax::kernel::CheckError) -> String {
    use iris_bootstrap::syntax::kernel::CheckError;
    match err {
        CheckError::Kernel(ke) => {
            let s = format!("{:?}", ke);
            if s.starts_with("TypeMismatch") {
                "Kernel::TypeMismatch".into()
            } else if s.starts_with("CostViolation") {
                "Kernel::CostViolation".into()
            } else if s.starts_with("InvalidRule") {
                // Extract rule name
                let rule = s.split("rule: \"").nth(1)
                    .and_then(|r| r.split('"').next())
                    .unwrap_or("?");
                let reason = s.split("reason: \"").nth(1)
                    .and_then(|r| r.split('"').next())
                    .unwrap_or("?");
                format!("InvalidRule({rule}): {}", reason.chars().take(50).collect::<String>())
            } else if s.starts_with("NodeNotFound") {
                "Kernel::NodeNotFound".into()
            } else if s.starts_with("TypeNotFound") {
                "Kernel::TypeNotFound".into()
            } else if s.starts_with("UnexpectedTypeDef") {
                "Kernel::UnexpectedTypeDef".into()
            } else {
                format!("Kernel::{}", s.split(|c: char| c == '{' || c == '(').next().unwrap_or("?"))
            }
        }
        CheckError::TierViolation { .. } => "TierViolation".into(),
        CheckError::MalformedGraph { reason } => {
            if reason.contains("children not proven") {
                "children_not_proven".into()
            } else {
                format!("MalformedGraph({})", reason.chars().take(40).collect::<String>())
            }
        }
        CheckError::Unsupported { kind, .. } => format!("Unsupported({kind})"),
        CheckError::RefinementViolation { .. } => "RefinementViolation".into(),
    }
}

#[test]
fn typecheck_all_iris_programs() {
    let mut files = Vec::new();
    collect_iris_files(Path::new("src/iris-programs"), &mut files);
    files.sort();

    let mut pass = 0u32;
    let mut compile_err = 0u32;
    let mut type_err = 0u32;
    let mut type_err_files: Vec<(String, String)> = Vec::new();
    let mut compile_err_files: Vec<String> = Vec::new();
    let mut error_cats: BTreeMap<String, u32> = BTreeMap::new();

    for path in &files {
        let src = fs::read_to_string(path).unwrap();
        let name = path.strip_prefix("src/iris-programs/")
            .unwrap_or(path)
            .display()
            .to_string();

        let result = iris_bootstrap::syntax::compile(&src);
        if !result.errors.is_empty() {
            compile_err += 1;
            compile_err_files.push(name);
            continue;
        }
        let mut file_ok = true;
        for (_fn_name, fragment, _smap) in &result.fragments {
            let tier = iris_bootstrap::syntax::classify_tier(&fragment.graph);
            let report = iris_bootstrap::syntax::kernel::checker::type_check_graded(
                &fragment.graph, tier,
            );
            if !report.failed.is_empty() {
                file_ok = false;
                for (_nid, err) in &report.failed {
                    let cat = categorize_error(err);
                    *error_cats.entry(cat).or_insert(0u32) += 1;
                }
            }
        }
        if file_ok {
            pass += 1;
        } else {
            type_err += 1;
            type_err_files.push((name, String::new()));
        }
    }

    eprintln!("\n=== BATCH TYPE CHECK: {} files ===", files.len());
    eprintln!("  ✓ pass:       {pass}");
    eprintln!("  ✗ type error:  {type_err}");
    eprintln!("  ✗ parse error: {compile_err}");

    eprintln!("\nError categories:");
    for (cat, count) in &error_cats {
        eprintln!("  {cat}: {count}");
    }

    if !type_err_files.is_empty() {
        eprintln!("\nType error files:");
        for (name, _) in &type_err_files {
            eprintln!("  {name}");
        }
    }
    if !compile_err_files.is_empty() {
        eprintln!("\nParse error files:");
        for name in &compile_err_files {
            eprintln!("  {name}");
        }
    }

    let pct = (pass as f64 / files.len() as f64) * 100.0;
    eprintln!("\nPass rate: {pct:.1}%");

    assert_eq!(type_err, 0, "type errors in .iris files");
    assert_eq!(compile_err, 0, "parse errors in .iris files");
    assert_eq!(pass, files.len() as u32, "all .iris files must type-check");
}
