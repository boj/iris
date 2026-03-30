//! Compiler error types.

use std::fmt;

/// Errors that can occur during compilation.
#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    /// Feature not yet implemented for Gen1.
    Unsupported(String),
    /// Internal compiler error (invariant violation).
    Internal(String),
    /// The input graph is malformed.
    InvalidGraph(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Unsupported(msg) => write!(f, "unsupported: {}", msg),
            CompileError::Internal(msg) => write!(f, "internal compiler error: {}", msg),
            CompileError::InvalidGraph(msg) => write!(f, "invalid graph: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {}
