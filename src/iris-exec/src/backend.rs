//! Execution backend registry.
//!
//! Enumerates the available execution backends on the current platform and
//! selects the best one.  The abstract machine (bytecode) is substrate-
//! independent; these backends are the only place where hardware assumptions
//! live.

use iris_types::abstract_machine;

// ---------------------------------------------------------------------------
// ExecutionBackend
// ---------------------------------------------------------------------------

/// The set of execution backends IRIS supports.
///
/// `TreeWalker` and `BytecodeVM` are always available (pure Rust, portable).
/// `Jit` and `Clcu` require specific hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutionBackend {
    /// Tree-walking interpreter over `SemanticGraph`.  Always available.
    TreeWalker,
    /// Stack-based VM executing compiled `Bytecode`.  Always available.
    BytecodeVM,
    /// JIT compiler emitting native code.  x86-64 only.
    Jit,
    /// CLCU chain interpreter using AVX-512.  x86-64 + AVX-512 only.
    Clcu,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return the list of backends available on the current host.
///
/// `TreeWalker` and `BytecodeVM` are unconditionally present.
/// `Jit` is present on x86-64.
/// `Clcu` is present on x86-64 with AVX-512.
pub fn available_backends() -> Vec<ExecutionBackend> {
    let caps = abstract_machine::current_machine().capabilities;
    let mut backends = vec![ExecutionBackend::TreeWalker, ExecutionBackend::BytecodeVM];

    if caps.has_jit {
        backends.push(ExecutionBackend::Jit);
    }
    if caps.has_avx512 {
        backends.push(ExecutionBackend::Clcu);
    }

    backends
}

/// Return the best backend available on the current host.
///
/// Preference order: Clcu > Jit > BytecodeVM > TreeWalker.
pub fn best_backend() -> ExecutionBackend {
    let backends = available_backends();
    if backends.contains(&ExecutionBackend::Clcu) {
        ExecutionBackend::Clcu
    } else if backends.contains(&ExecutionBackend::Jit) {
        ExecutionBackend::Jit
    } else {
        ExecutionBackend::BytecodeVM
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_backends_always_includes_universal() {
        let backends = available_backends();
        assert!(backends.contains(&ExecutionBackend::TreeWalker));
        assert!(backends.contains(&ExecutionBackend::BytecodeVM));
    }

    #[test]
    fn best_backend_is_from_available() {
        let backends = available_backends();
        let best = best_backend();
        assert!(backends.contains(&best));
    }

    #[test]
    fn available_backends_has_at_least_two() {
        assert!(available_backends().len() >= 2);
    }
}
