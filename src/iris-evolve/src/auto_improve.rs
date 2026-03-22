//! Autonomous self-improvement daemon for IRIS.
//!
//! `AutoImprover` is a continuously running process that profiles its own
//! components, evolves IRIS replacements for them, and deploys improvements
//! through the performance gate. The cycle is:
//!
//! 1. **Profile**: measure each component's current speed, find the slowest.
//! 2. **Extract**: generate test cases from the slowest component.
//! 3. **Evolve**: try to breed a faster IRIS replacement.
//! 4. **Gate**: check performance (100% correct, <2x slowdown).
//! 5. **Deploy**: register the IRIS component if it passes.
//! 6. **Explore**: if nothing to optimize, try solving a new random problem.

use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

use iris_exec::ExecutionService;
use iris_types::eval::{TestCase, Value};
use iris_types::graph::SemanticGraph;

use crate::config::{EvolutionConfig, ProblemSpec};
use crate::corpus::generate_problem_specs;
use crate::performance_gate::{self, PerformanceGateResult};

// ---------------------------------------------------------------------------
// AutoImproveConfig
// ---------------------------------------------------------------------------

/// Configuration for the autonomous self-improvement daemon.
#[derive(Debug, Clone)]
pub struct AutoImproveConfig {
    /// Seconds to sleep between improvement cycles.
    pub cycle_interval_secs: u64,
    /// Maximum slowdown allowed for IRIS replacements (default: 2.0x).
    pub max_slowdown: f64,
    /// Number of test cases to generate per component.
    pub test_cases_per_component: usize,
    /// Evolution budget (generations) for replacement candidates.
    pub evolution_generations: usize,
    /// Evolution population size for replacement candidates.
    pub evolution_pop_size: usize,
    /// Number of timing runs for performance gate.
    pub gate_runs: usize,
    /// Maximum number of problems to explore when idle.
    pub explore_problems: usize,
}

impl Default for AutoImproveConfig {
    fn default() -> Self {
        Self {
            cycle_interval_secs: 10,
            max_slowdown: 2.0,
            test_cases_per_component: 10,
            evolution_generations: 50,
            evolution_pop_size: 32,
            gate_runs: 3,
            explore_problems: 5,
        }
    }
}

// ---------------------------------------------------------------------------
// DeployedComponent
// ---------------------------------------------------------------------------

/// An IRIS program that has been deployed to replace a Rust component.
#[derive(Debug, Clone)]
pub struct DeployedComponent {
    pub name: String,
    pub iris_program: SemanticGraph,
    pub slowdown: f64,
    pub deployed_at: u64, // generation (cycle count)
}

// ---------------------------------------------------------------------------
// ImprovementAction / ImprovementEvent
// ---------------------------------------------------------------------------

/// What happened during an improvement cycle.
#[derive(Debug, Clone)]
pub enum ImprovementAction {
    /// Profiled components, found the slowest one.
    Profiled { slowest: String, time_ns: u64 },
    /// Evolved a candidate replacement.
    Evolved { candidate_slowdown: f64 },
    /// Successfully deployed an IRIS replacement.
    Deployed { slowdown: f64 },
    /// Attempted improvement but failed.
    Failed { reason: String },
    /// Explored a new capability (no component to optimize).
    Explored { new_capability: String, solve_rate: f32 },
}

/// A single event in the improvement history.
#[derive(Debug, Clone)]
pub struct ImprovementEvent {
    pub component: String,
    pub action: ImprovementAction,
    pub timestamp: u64, // cycle count
}

impl ImprovementEvent {
    /// Human-readable summary of this event.
    pub fn summary(&self) -> String {
        match &self.action {
            ImprovementAction::Profiled { slowest, time_ns } => {
                format!(
                    "cycle {}: profiled components, slowest='{}' ({} ns)",
                    self.timestamp, slowest, time_ns
                )
            }
            ImprovementAction::Evolved { candidate_slowdown } => {
                format!(
                    "cycle {}: evolved candidate for '{}' (slowdown={:.1}x)",
                    self.timestamp, self.component, candidate_slowdown
                )
            }
            ImprovementAction::Deployed { slowdown } => {
                format!(
                    "cycle {}: DEPLOYED '{}' (slowdown={:.1}x)",
                    self.timestamp, self.component, slowdown
                )
            }
            ImprovementAction::Failed { reason } => {
                format!(
                    "cycle {}: failed to improve '{}': {}",
                    self.timestamp, self.component, reason
                )
            }
            ImprovementAction::Explored {
                new_capability,
                solve_rate,
            } => {
                format!(
                    "cycle {}: explored '{}' (solve_rate={:.1}%)",
                    self.timestamp,
                    new_capability,
                    solve_rate * 100.0
                )
            }
        }
    }
}

impl fmt::Display for ImprovementEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

// ---------------------------------------------------------------------------
// AutoImproveStatus
// ---------------------------------------------------------------------------

/// Snapshot of the auto-improver's current state.
#[derive(Debug, Clone)]
pub struct AutoImproveStatus {
    pub components_profiled: usize,
    pub components_deployed: usize,
    pub total_improvement_cycles: usize,
    pub capabilities_explored: usize,
    pub deployed_components: Vec<String>,
}

// ---------------------------------------------------------------------------
// AutoImprover
// ---------------------------------------------------------------------------

/// The autonomous self-improvement engine.
///
/// Profiles registered components, evolves IRIS replacements, and deploys
/// them through the performance gate. Components that could be replaced
/// are registered with their Rust timing baselines.
pub struct AutoImprover {
    /// Components that could be replaced (name -> Rust timing baseline).
    rust_baselines: BTreeMap<String, Duration>,
    /// Currently deployed IRIS replacements.
    deployed: BTreeMap<String, DeployedComponent>,
    /// Problems for testing components.
    #[allow(dead_code)]
    test_problems: Vec<ProblemSpec>,
    /// History of improvement events.
    history: Vec<ImprovementEvent>,
    /// Current cycle count.
    cycle_count: u64,
    /// Configuration.
    config: AutoImproveConfig,
    /// RNG for generating test cases and problems.
    rng: StdRng,
}

impl AutoImprover {
    /// Create a new auto-improver with default configuration.
    pub fn new() -> Self {
        Self::with_config(AutoImproveConfig::default())
    }

    /// Create a new auto-improver with the given configuration.
    pub fn with_config(config: AutoImproveConfig) -> Self {
        let mut rng = StdRng::from_entropy();
        let test_problems = generate_problem_specs(&mut rng, config.explore_problems);
        Self {
            rust_baselines: BTreeMap::new(),
            deployed: BTreeMap::new(),
            test_problems,
            history: Vec::new(),
            cycle_count: 0,
            config,
            rng,
        }
    }

    /// Create with a deterministic seed (for testing).
    pub fn with_seed(config: AutoImproveConfig, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let test_problems = generate_problem_specs(&mut rng, config.explore_problems);
        Self {
            rust_baselines: BTreeMap::new(),
            deployed: BTreeMap::new(),
            test_problems,
            history: Vec::new(),
            cycle_count: 0,
            config,
            rng,
        }
    }

    /// Register a component with its Rust baseline timing.
    pub fn register_component(&mut self, name: &str, baseline: Duration) {
        self.rust_baselines.insert(name.to_string(), baseline);
    }

    /// Get a reference to the deployed components.
    pub fn deployed(&self) -> &BTreeMap<String, DeployedComponent> {
        &self.deployed
    }

    /// Get the improvement history.
    pub fn history(&self) -> &[ImprovementEvent] {
        &self.history
    }

    /// Get the current cycle count.
    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }

    // -----------------------------------------------------------------------
    // The improvement cycle
    // -----------------------------------------------------------------------

    /// Run a single improvement cycle.
    ///
    /// 1. Profile all components to find the slowest.
    /// 2. Extract test cases from the slowest component.
    /// 3. Evolve a replacement.
    /// 4. Gate-check the replacement.
    /// 5. Deploy if it passes.
    /// 6. If nothing to optimize, explore a new capability.
    pub fn run_cycle(&mut self, exec: &dyn ExecutionService) -> ImprovementEvent {
        self.cycle_count += 1;

        // 1. PROFILE: find the slowest component.
        if self.rust_baselines.is_empty() {
            // No components registered — go straight to exploration.
            let event = self.explore_new_capability_event(exec);
            self.history.push(event.clone());
            return event;
        }

        let (slowest_name, slowest_time) = self.profile_components();
        let profile_event = ImprovementEvent {
            component: slowest_name.clone(),
            action: ImprovementAction::Profiled {
                slowest: slowest_name.clone(),
                time_ns: slowest_time.as_nanos() as u64,
            },
            timestamp: self.cycle_count,
        };
        self.history.push(profile_event);

        // If already deployed, try to improve a different component or explore.
        if self.deployed.contains_key(&slowest_name) {
            let event = self.explore_new_capability_event(exec);
            self.history.push(event.clone());
            return event;
        }

        // 2. EXTRACT: generate test cases.
        let test_cases = self.extract_test_cases(&slowest_name);

        // 3. EVOLVE: try to evolve a replacement.
        let candidate = self.evolve_replacement(&slowest_name, &test_cases, exec);

        match candidate {
            Some(program) => {
                // 4. GATE: check performance.
                let gate_result = self.run_performance_gate(
                    &slowest_name,
                    &program,
                    &test_cases,
                );

                if gate_result.passes_gate {
                    // 5. DEPLOY.
                    let slowdown = gate_result.slowdown;
                    self.deploy(&slowest_name, program, slowdown);
                    let event = ImprovementEvent {
                        component: slowest_name,
                        action: ImprovementAction::Deployed { slowdown },
                        timestamp: self.cycle_count,
                    };
                    self.history.push(event.clone());
                    event
                } else {
                    let event = ImprovementEvent {
                        component: slowest_name,
                        action: ImprovementAction::Failed {
                            reason: format!(
                                "gate failed: correctness={:.0}% slowdown={:.1}x",
                                gate_result.correctness * 100.0,
                                gate_result.slowdown
                            ),
                        },
                        timestamp: self.cycle_count,
                    };
                    self.history.push(event.clone());
                    event
                }
            }
            None => {
                let event = ImprovementEvent {
                    component: slowest_name,
                    action: ImprovementAction::Failed {
                        reason: "evolution produced no viable candidate".to_string(),
                    },
                    timestamp: self.cycle_count,
                };
                self.history.push(event.clone());
                event
            }
        }
    }

    /// Profile all registered components, return the slowest (name, duration).
    ///
    /// When an `IrisRuntime` is provided, uses actual runtime timings from
    /// its performance tracker instead of hardcoded Rust baselines. Falls back
    /// to baselines for components with no runtime samples.
    pub fn profile_components(&self) -> (String, Duration) {
        self.rust_baselines
            .iter()
            .max_by_key(|(_, d)| *d)
            .map(|(name, d)| (name.clone(), *d))
            .unwrap_or_else(|| ("unknown".to_string(), Duration::ZERO))
    }

    /// Profile using actual IRIS runtime timings when available.
    ///
    /// Queries `iris_runtime.component_timings()` for real measurements.
    /// For components without runtime samples, falls back to Rust baselines.
    /// Returns the slowest component.
    pub fn profile_with_runtime(
        &self,
        runtime: &crate::iris_runtime::IrisRuntime,
    ) -> (String, Duration) {
        let runtime_timings = runtime.component_timings();

        if !runtime_timings.is_empty() {
            // Use actual runtime timings — they reflect real IRIS performance.
            let (name, dur) = &runtime_timings[0]; // sorted slowest first
            return (name.clone(), *dur);
        }

        // No runtime samples yet — fall back to baselines.
        self.profile_components()
    }

    /// Generate test cases by building a ProblemSpec for the component.
    ///
    /// For real use, these would be derived from the component's actual
    /// inputs/outputs. For now, we generate random arithmetic problems
    /// that exercise the core evaluation path.
    pub fn extract_test_cases(&mut self, _component_name: &str) -> Vec<TestCase> {
        let count = self.config.test_cases_per_component;
        let mut cases = Vec::with_capacity(count);
        for _ in 0..count {
            let a = self.rng.gen_range(-50i64..50);
            let b = self.rng.gen_range(-50i64..50);
            let result = a.wrapping_add(b);
            cases.push(TestCase {
                inputs: vec![Value::Int(a), Value::Int(b)],
                expected_output: Some(vec![Value::Int(result)]),
                initial_state: None,
                expected_state: None,
            });
        }
        cases
    }

    /// Evolve a replacement using IRIS evolution.
    ///
    /// Returns `Some(program)` if evolution found a candidate with non-zero
    /// correctness, `None` otherwise.
    pub fn evolve_replacement(
        &self,
        _name: &str,
        tests: &[TestCase],
        exec: &dyn ExecutionService,
    ) -> Option<SemanticGraph> {
        let spec = ProblemSpec {
            test_cases: tests.to_vec(),
            description: format!("auto_improve_replacement"),
            target_cost: None,
        };

        let config = EvolutionConfig {
            population_size: self.config.evolution_pop_size,
            max_generations: self.config.evolution_generations,
            num_demes: 1,
            ..Default::default()
        };

        let result = crate::evolve_with_timeout(
            config,
            spec,
            exec,
            Duration::from_secs(5),
        );

        // Only return candidates with non-trivial correctness.
        if result.best_individual.fitness.correctness() > 0.0 {
            Some(result.best_individual.fragment.graph.clone())
        } else {
            None
        }
    }

    /// Run the performance gate on a candidate replacement.
    fn run_performance_gate(
        &self,
        component_name: &str,
        candidate: &SemanticGraph,
        test_cases: &[TestCase],
    ) -> PerformanceGateResult {
        // Build input vectors for the gate.
        let test_inputs: Vec<Vec<Value>> = test_cases
            .iter()
            .map(|tc| tc.inputs.clone())
            .collect();

        // The "Rust function" for the gate is just the expected output lookup.
        // This simulates a perfect Rust implementation.
        let expected_outputs: Vec<Vec<Value>> = test_cases
            .iter()
            .filter_map(|tc| tc.expected_output.clone())
            .collect();

        let expected_ref = expected_outputs.clone();
        let rust_fn = move |inputs: &[Value]| -> Vec<Value> {
            // Find the matching test case by inputs and return its expected output.
            for (i, tc_inputs) in test_inputs.iter().enumerate() {
                if tc_inputs == inputs {
                    if let Some(expected) = expected_ref.get(i) {
                        return expected.clone();
                    }
                }
            }
            vec![]
        };

        // Re-collect inputs for the gate call.
        let gate_inputs: Vec<Vec<Value>> = test_cases
            .iter()
            .map(|tc| tc.inputs.clone())
            .collect();

        performance_gate::performance_gate(
            component_name,
            rust_fn,
            candidate,
            &gate_inputs,
            self.config.max_slowdown,
            self.config.gate_runs,
        )
    }

    /// Deploy an IRIS component, replacing the Rust version.
    pub fn deploy(&mut self, name: &str, program: SemanticGraph, slowdown: f64) {
        self.deployed.insert(
            name.to_string(),
            DeployedComponent {
                name: name.to_string(),
                iris_program: program,
                slowdown,
                deployed_at: self.cycle_count,
            },
        );
    }

    /// Generate a random new problem and try to solve it.
    ///
    /// Returns (problem_description, solve_rate).
    pub fn explore_new_capability(
        &mut self,
        exec: &dyn ExecutionService,
    ) -> (String, f32) {
        // Generate a fresh problem.
        let problems = generate_problem_specs(&mut self.rng, 1);
        let problem = match problems.into_iter().next() {
            Some(p) => p,
            None => {
                return ("none".to_string(), 0.0);
            }
        };

        let description = problem.description.clone();

        let config = EvolutionConfig {
            population_size: self.config.evolution_pop_size,
            max_generations: self.config.evolution_generations,
            num_demes: 1,
            ..Default::default()
        };

        let result = crate::evolve_with_timeout(
            config,
            problem,
            exec,
            Duration::from_secs(5),
        );

        let solve_rate = result.best_individual.fitness.correctness();
        (description, solve_rate)
    }

    /// Internal helper: run explore and wrap in an event.
    fn explore_new_capability_event(
        &mut self,
        exec: &dyn ExecutionService,
    ) -> ImprovementEvent {
        let (desc, rate) = self.explore_new_capability(exec);
        ImprovementEvent {
            component: "exploration".to_string(),
            action: ImprovementAction::Explored {
                new_capability: desc,
                solve_rate: rate,
            },
            timestamp: self.cycle_count,
        }
    }

    /// Summary of current state.
    pub fn status(&self) -> AutoImproveStatus {
        let capabilities_explored = self
            .history
            .iter()
            .filter(|e| matches!(e.action, ImprovementAction::Explored { .. }))
            .count();

        AutoImproveStatus {
            components_profiled: self.rust_baselines.len(),
            components_deployed: self.deployed.len(),
            total_improvement_cycles: self.cycle_count as usize,
            capabilities_explored,
            deployed_components: self.deployed.keys().cloned().collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// Daemon entry point
// ---------------------------------------------------------------------------

/// Run the self-improving daemon loop (blocks forever).
///
/// Each cycle: profile -> evolve -> gate -> deploy, then sleep.
/// This is intended to be run in its own thread or as a standalone process.
pub fn run_self_improving(config: AutoImproveConfig, exec: &dyn ExecutionService) -> ! {
    let interval = Duration::from_secs(config.cycle_interval_secs);
    let mut improver = AutoImprover::with_config(config);

    // Register some default components with synthetic baselines.
    // In a real deployment, these would be measured from actual Rust code.
    improver.register_component("mutation_insert_node", Duration::from_micros(50));
    improver.register_component("mutation_delete_node", Duration::from_micros(30));
    improver.register_component("mutation_rewire_edge", Duration::from_micros(45));
    improver.register_component("seed_arithmetic", Duration::from_micros(20));
    improver.register_component("seed_fold", Duration::from_micros(25));

    loop {
        let event = improver.run_cycle(exec);
        eprintln!("[AutoImprove] {}", event.summary());

        std::thread::sleep(interval);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_config() -> AutoImproveConfig {
        AutoImproveConfig {
            cycle_interval_secs: 0,
            max_slowdown: 100.0, // generous for testing
            test_cases_per_component: 4,
            evolution_generations: 3,
            evolution_pop_size: 8,
            gate_runs: 1,
            explore_problems: 3,
        }
    }

    #[test]
    fn auto_improver_initializes_empty() {
        let improver = AutoImprover::with_config(tiny_config());
        assert_eq!(improver.rust_baselines.len(), 0);
        assert_eq!(improver.deployed.len(), 0);
        assert_eq!(improver.cycle_count(), 0);
        assert!(improver.history().is_empty());
    }

    #[test]
    fn register_component_adds_baseline() {
        let mut improver = AutoImprover::with_config(tiny_config());
        improver.register_component("test_comp", Duration::from_micros(100));
        assert_eq!(improver.rust_baselines.len(), 1);
        assert!(improver.rust_baselines.contains_key("test_comp"));
    }

    #[test]
    fn profile_components_finds_slowest() {
        let mut improver = AutoImprover::with_config(tiny_config());
        improver.register_component("fast", Duration::from_micros(10));
        improver.register_component("slow", Duration::from_micros(100));
        improver.register_component("medium", Duration::from_micros(50));

        let (name, dur) = improver.profile_components();
        assert_eq!(name, "slow");
        assert_eq!(dur, Duration::from_micros(100));
    }

    #[test]
    fn extract_test_cases_generates_valid_cases() {
        let mut improver = AutoImprover::with_seed(tiny_config(), 42);
        let cases = improver.extract_test_cases("anything");
        assert_eq!(cases.len(), 4); // test_cases_per_component = 4
        for tc in &cases {
            assert_eq!(tc.inputs.len(), 2);
            assert!(tc.expected_output.is_some());
            let expected = tc.expected_output.as_ref().unwrap();
            assert_eq!(expected.len(), 1);
            // Verify the addition is correct.
            if let (Value::Int(a), Value::Int(b)) = (&tc.inputs[0], &tc.inputs[1]) {
                if let Value::Int(r) = &expected[0] {
                    assert_eq!(*r, a.wrapping_add(*b));
                }
            }
        }
    }

    #[test]
    fn status_reports_correct_counts() {
        let mut improver = AutoImprover::with_config(tiny_config());
        improver.register_component("a", Duration::from_micros(10));
        improver.register_component("b", Duration::from_micros(20));

        let status = improver.status();
        assert_eq!(status.components_profiled, 2);
        assert_eq!(status.components_deployed, 0);
        assert_eq!(status.total_improvement_cycles, 0);
        assert_eq!(status.capabilities_explored, 0);
        assert!(status.deployed_components.is_empty());
    }

    #[test]
    fn improvement_event_summary_formatting() {
        let event = ImprovementEvent {
            component: "test".to_string(),
            action: ImprovementAction::Deployed { slowdown: 1.5 },
            timestamp: 7,
        };
        let s = event.summary();
        assert!(s.contains("DEPLOYED"));
        assert!(s.contains("1.5x"));
        assert!(s.contains("cycle 7"));
    }
}
