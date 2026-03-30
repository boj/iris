//! Maps kernel verification errors back to surface syntax source locations.

use crate::syntax::kernel::checker::VerificationReport;
use iris_types::graph::NodeId;

use crate::syntax::error::{format_error, Span, SyntaxError};
use crate::syntax::lower::SourceMap;

/// A verification diagnostic tied to a source location.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub span: Option<Span>,
    pub node_id: NodeId,
}

/// Convert a kernel `VerificationReport` into diagnostics with source locations.
pub fn diagnose(report: &VerificationReport, source_map: &SourceMap) -> Vec<Diagnostic> {
    report
        .failed
        .iter()
        .map(|(node_id, err)| {
            let span = source_map.get(node_id).copied();
            Diagnostic {
                message: err.to_string(),
                span,
                node_id: *node_id,
            }
        })
        .collect()
}

/// Format all diagnostics from a verification report as a single string,
/// with source context (line numbers, carets) for each error.
pub fn format_diagnostics(
    source: &str,
    report: &VerificationReport,
    source_map: &SourceMap,
) -> String {
    let diags = diagnose(report, source_map);
    if diags.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for diag in &diags {
        match diag.span {
            Some(span) => {
                let err = SyntaxError::new(&diag.message, span);
                out.push_str(&format_error(source, &err));
            }
            None => {
                out.push_str(&format!(
                    "error (node {:?}): {}\n",
                    diag.node_id, diag.message
                ));
            }
        }
    }
    out
}
