//! Performance parity gate for self-writing bootstrap.
//!
//! An IRIS-evolved component replaces its Rust equivalent ONLY IF:
//! 1. Correctness: 100% match on all test cases
//! 2. Performance: within `max_slowdown` wall-clock time of Rust version
//! 3. The component passes this gate on N consecutive runs

use std::time::Instant;

use iris_types::eval::Value;
use iris_types::graph::SemanticGraph;

/// Result of a performance comparison between Rust and IRIS implementations.
#[derive(Debug, Clone)]
pub struct PerformanceGateResult {
    pub component_name: String,
    pub correctness: f32,           // 0.0 - 1.0
    pub rust_time_ns: u64,
    pub iris_time_ns: u64,
    pub slowdown: f64,              // iris_time / rust_time
    pub passes_gate: bool,
    pub max_allowed_slowdown: f64,
    pub test_count: usize,
}

impl PerformanceGateResult {
    pub fn summary(&self) -> String {
        format!(
            "{}: correctness={:.0}% slowdown={:.1}x (limit {:.1}x) → {}",
            self.component_name,
            self.correctness * 100.0,
            self.slowdown,
            self.max_allowed_slowdown,
            if self.passes_gate { "PASS" } else { "FAIL" },
        )
    }
}

/// Compare an IRIS component against its Rust equivalent.
///
/// `rust_fn`: the Rust implementation (takes inputs, returns outputs)
/// `iris_program`: the IRIS program that should do the same thing
/// `test_inputs`: list of input vectors to test on
/// `max_slowdown`: maximum allowed slowdown (e.g., 2.0 = IRIS can be 2x slower)
/// `runs`: number of repetitions for timing accuracy
pub fn performance_gate<F>(
    component_name: &str,
    rust_fn: F,
    iris_program: &SemanticGraph,
    test_inputs: &[Vec<Value>],
    max_slowdown: f64,
    runs: usize,
) -> PerformanceGateResult
where
    F: Fn(&[Value]) -> Vec<Value>,
{
    let mut correct = 0usize;
    let mut total = 0usize;

    // Correctness check: compare outputs
    for inputs in test_inputs {
        let rust_output = rust_fn(inputs);
        let iris_output = run_iris(iris_program, inputs);
        total += 1;
        if outputs_match(&rust_output, &iris_output) {
            correct += 1;
        }
    }

    let correctness = if total > 0 { correct as f32 / total as f32 } else { 0.0 };

    // Performance check: time both over multiple runs
    let rust_start = Instant::now();
    for _ in 0..runs {
        for inputs in test_inputs {
            let _ = rust_fn(inputs);
        }
    }
    let rust_time = rust_start.elapsed();

    let iris_start = Instant::now();
    for _ in 0..runs {
        for inputs in test_inputs {
            let _ = run_iris(iris_program, inputs);
        }
    }
    let iris_time = iris_start.elapsed();

    let rust_ns = rust_time.as_nanos() as u64;
    let iris_ns = iris_time.as_nanos() as u64;
    let slowdown = if rust_ns > 0 {
        iris_ns as f64 / rust_ns as f64
    } else {
        1.0
    };

    let passes = correctness >= 0.999 && slowdown <= max_slowdown;

    PerformanceGateResult {
        component_name: component_name.to_string(),
        correctness,
        rust_time_ns: rust_ns,
        iris_time_ns: iris_ns,
        slowdown,
        passes_gate: passes,
        max_allowed_slowdown: max_slowdown,
        test_count: total,
    }
}

/// Run an IRIS program on inputs via the interpreter with compute-only sandbox.
///
/// Uses `Capabilities::sandbox()` to restrict the program to pure computation
/// with no I/O, no FFI, no network access, and no thread spawning.  This
/// prevents evolved candidates from performing side-effects during gate
/// evaluation, which could corrupt host state or game fitness measurements.
fn run_iris(program: &SemanticGraph, inputs: &[Value]) -> Vec<Value> {
    match iris_exec_interpret(program, inputs) {
        Ok((outputs, _)) => outputs,
        Err(_) => vec![],
    }
}

fn iris_exec_interpret(
    graph: &SemanticGraph,
    inputs: &[Value],
) -> Result<(Vec<Value>, iris_types::eval::StateStore), iris_exec::interpreter::InterpretError> {
    // Sandbox: computation only, no I/O, no FFI, no network.
    let caps = iris_exec::capabilities::Capabilities::sandbox();
    iris_exec::interpreter::interpret_with_capabilities(
        graph, inputs, None, None, None, None, None, 0, caps,
    )
}

fn outputs_match(a: &[Value], b: &[Value]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(x, y)| x == y)
}

/// Deployment decision: should this IRIS component replace its Rust equivalent?
pub fn should_deploy(gate_results: &[PerformanceGateResult]) -> bool {
    // Must pass ALL runs
    gate_results.iter().all(|r| r.passes_gate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_result_summary_format() {
        let result = PerformanceGateResult {
            component_name: "test".to_string(),
            correctness: 1.0,
            rust_time_ns: 1000,
            iris_time_ns: 1500,
            slowdown: 1.5,
            passes_gate: true,
            max_allowed_slowdown: 2.0,
            test_count: 10,
        };
        let s = result.summary();
        assert!(s.contains("PASS"));
        assert!(s.contains("1.5x"));
    }

    #[test]
    fn should_deploy_requires_all_pass() {
        let pass = PerformanceGateResult {
            component_name: "a".into(), correctness: 1.0,
            rust_time_ns: 100, iris_time_ns: 150, slowdown: 1.5,
            passes_gate: true, max_allowed_slowdown: 2.0, test_count: 5,
        };
        let fail = PerformanceGateResult {
            component_name: "a".into(), correctness: 0.8,
            rust_time_ns: 100, iris_time_ns: 500, slowdown: 5.0,
            passes_gate: false, max_allowed_slowdown: 2.0, test_count: 5,
        };
        assert!(should_deploy(&[pass.clone(), pass.clone()]));
        assert!(!should_deploy(&[pass, fail]));
    }
}
