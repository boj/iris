/// The IRIS Abstract Machine.
///
/// Programs are represented as SemanticGraphs (substrate-independent).
/// They compile to Bytecode (the abstract machine -- also substrate-independent).
/// Bytecode executes on:
///   - The Rust VM (iris-exec/vm.rs) -- portable, any platform
///   - The JIT (iris-exec/jit.rs) -- x86-64 native code
///   - The CLCU interpreter (iris-clcu) -- x86-64 AVX-512 optimized
///
/// The SemanticGraph and Bytecode contain NO hardware assumptions.
/// Hardware-specific behavior lives ONLY in execution backends.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// AbstractMachine
// ---------------------------------------------------------------------------

/// Describes the IRIS abstract machine and the capabilities of the current
/// platform.
///
/// This is a *description*, not a runtime engine. The abstract machine is
/// the bytecode instruction set itself; this struct advertises what the
/// current host can execute.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbstractMachine {
    /// Human-readable name (e.g. "IRIS-AM/1").
    pub name: String,
    /// Abstract machine version.  Bumped when the opcode set changes.
    pub version: u32,
    /// Capabilities available on the current host.
    pub capabilities: MachineCapabilities,
}

// ---------------------------------------------------------------------------
// MachineCapabilities
// ---------------------------------------------------------------------------

/// What the current host can provide beyond the universal abstract machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MachineCapabilities {
    /// Maximum operand stack depth the VM enforces.
    pub max_stack_depth: usize,
    /// Maximum number of constants in a single bytecode program.
    pub max_constants: usize,
    /// Whether a JIT backend is available on this platform.
    pub has_jit: bool,
    /// Whether the CLCU (AVX-512) backend is available.
    pub has_avx512: bool,
    /// Whether hardware performance counters are accessible.
    pub has_perf_counters: bool,
}

// ---------------------------------------------------------------------------
// current_machine
// ---------------------------------------------------------------------------

/// Return an `AbstractMachine` describing the current host.
///
/// The abstract machine version and stack/constant limits are fixed at
/// compile time.  Hardware feature flags are detected at runtime via
/// `cfg!(target_arch)` and `is_x86_feature_detected!`.
pub fn current_machine() -> AbstractMachine {
    AbstractMachine {
        name: "IRIS-AM/1".to_string(),
        version: 1,
        capabilities: MachineCapabilities {
            max_stack_depth: 1024,
            max_constants: u16::MAX as usize,
            has_jit: cfg!(target_arch = "x86_64"),
            has_avx512: detect_avx512(),
            has_perf_counters: detect_perf_counters(),
        },
    }
}

/// Detect AVX-512 support at runtime.
fn detect_avx512() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        #[cfg(target_feature = "avx512f")]
        {
            return true;
        }
        #[cfg(not(target_feature = "avx512f"))]
        {
            // Runtime detection via std.  `is_x86_feature_detected!` is a
            // std macro that reads CPUID.
            std::is_x86_feature_detected!("avx512f")
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// Detect whether hardware performance counters are likely available.
///
/// We use a simple heuristic: Linux + x86_64 usually means perf_event_open
/// is available.  Other platforms return false.
fn detect_perf_counters() -> bool {
    cfg!(all(target_os = "linux", target_arch = "x86_64"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_machine_has_valid_name_and_version() {
        let m = current_machine();
        assert_eq!(m.name, "IRIS-AM/1");
        assert_eq!(m.version, 1);
    }

    #[test]
    fn capabilities_limits_are_positive() {
        let m = current_machine();
        assert!(m.capabilities.max_stack_depth > 0);
        assert!(m.capabilities.max_constants > 0);
    }

    #[test]
    fn abstract_machine_serializes_round_trip() {
        let m = current_machine();
        let json = serde_json::to_string(&m).expect("serialize");
        let m2: AbstractMachine = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(m, m2);
    }
}
