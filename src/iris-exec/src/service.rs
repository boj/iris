//! Service module — IrisExecutionService backed by iris-bootstrap.

use std::sync::Arc;

use iris_types::eval::{EvalError, EvalResult, EvalTier, StateStore, TestCase, Value};
use iris_types::fragment::FragmentId;
use iris_types::graph::SemanticGraph;

use crate::capabilities::Capabilities;
use crate::effect_runtime::RuntimeEffectHandler;
use crate::interpreter;
use crate::registry::FragmentRegistry;
use crate::stats::CacheStats;
use crate::ExecutionService;

#[derive(Debug, Clone, Copy)]
pub struct SandboxConfig {
    pub memory_limit_bytes: usize,
    pub step_limit: u64,
    pub timeout_ms: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            memory_limit_bytes: interpreter::DEFAULT_MEMORY_LIMIT,
            step_limit: interpreter::MAX_STEPS,
            timeout_ms: 5_000,
        }
    }
}

impl SandboxConfig {
    pub fn enumerate() -> Self {
        Self {
            memory_limit_bytes: interpreter::ENUMERATE_MEMORY_LIMIT,
            step_limit: 10_000,
            timeout_ms: 1_000,
        }
    }

    pub fn tier_bc() -> Self {
        Self {
            memory_limit_bytes: interpreter::DEFAULT_MEMORY_LIMIT,
            step_limit: interpreter::MAX_STEPS,
            timeout_ms: 30_000,
        }
    }
}

pub struct ExecConfig {
    pub cache_capacity: usize,
    pub worker_threads: usize,
    pub sandbox: SandboxConfig,
    pub capabilities: Capabilities,
}

impl Default for ExecConfig {
    fn default() -> Self {
        Self {
            cache_capacity: 1024,
            worker_threads: std::thread::available_parallelism()
                .map(|n| n.get().min(28))
                .unwrap_or(4),
            sandbox: SandboxConfig::default(),
            capabilities: Capabilities::sandbox(),
        }
    }
}

pub struct IrisExecutionService {
    registry: Arc<FragmentRegistry>,
    sandbox: SandboxConfig,
    capabilities: Capabilities,
    effect_handler: Arc<RuntimeEffectHandler>,
}

impl IrisExecutionService {
    pub fn new(config: ExecConfig) -> Self {
        Self {
            registry: Arc::new(FragmentRegistry::new()),
            sandbox: config.sandbox,
            capabilities: config.capabilities,
            effect_handler: RuntimeEffectHandler::shared(),
        }
    }

    pub fn with_registry(config: ExecConfig, registry: FragmentRegistry) -> Self {
        Self {
            registry: Arc::new(registry),
            sandbox: config.sandbox,
            capabilities: config.capabilities,
            effect_handler: RuntimeEffectHandler::shared(),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(ExecConfig::default())
    }

    pub fn with_capabilities(capabilities: Capabilities) -> Self {
        Self::new(ExecConfig {
            capabilities,
            ..ExecConfig::default()
        })
    }

    pub fn registry(&self) -> &FragmentRegistry {
        &self.registry
    }

    fn graph_id(graph: &SemanticGraph) -> FragmentId {
        FragmentId(graph.hash.0)
    }

    fn eval_one(
        &self,
        graph: &SemanticGraph,
        test_inputs: &[TestCase],
        _tier: EvalTier,
    ) -> Result<EvalResult, EvalError> {
        let graph_hash = Self::graph_id(graph);
        let step_limit = self.sandbox.step_limit * (1 + graph.nodes.len() as u64 / 100);
        let start = std::time::Instant::now();

        let mut outputs: Vec<Vec<Value>> = Vec::with_capacity(test_inputs.len());
        let mut final_states: Vec<StateStore> = Vec::with_capacity(test_inputs.len());

        // Wrap the runtime effect handler with capability enforcement.
        let guarded = crate::capabilities::CapabilityGuardHandler::new(
            self.effect_handler.as_ref(),
            self.capabilities.clone(),
        );

        for tc in test_inputs {
            let mut init_state = tc.initial_state.clone().unwrap_or_default();
            match interpreter::interpret_sandboxed(
                graph,
                &tc.inputs,
                Some(&mut init_state),
                if self.registry.is_empty() { None } else { Some(self.registry.as_ref()) },
                step_limit,
                self.sandbox.memory_limit_bytes,
                Some(&guarded),
                None,
                None,
                0,
            ) {
                Ok((vals, state)) => {
                    outputs.push(vals);
                    final_states.push(state);
                }
                Err(e) => {
                    eprintln!("[iris-exec] interpret_sandboxed error: {}", e);
                    outputs.push(vec![]);
                    final_states.push(StateStore::new());
                }
            }
        }

        let wall_time_ns = start.elapsed().as_nanos() as u64;
        let correctness = compute_correctness(&outputs, &final_states, test_inputs);

        Ok(EvalResult {
            outputs,
            outputs_hash: 0,
            correctness_score: correctness,
            per_case_scores: vec![],
            wall_time_ns,
            compile_time_ns: 0,
            counters: None,
            tier_executed: EvalTier::A,
            cache_hit: false,
            graph_hash,
        })
    }
}

fn compute_correctness(
    outputs: &[Vec<Value>],
    _states: &[StateStore],
    test_cases: &[TestCase],
) -> f32 {
    if test_cases.is_empty() {
        return 0.0;
    }

    let mut total = 0.0f32;
    for (i, tc) in test_cases.iter().enumerate() {
        if let Some(expected) = &tc.expected_output {
            if let Some(actual) = outputs.get(i) {
                if actual == expected {
                    total += 1.0;
                } else {
                    let mut case_score = 0.0f32;
                    let n = expected.len().max(actual.len()).max(1);
                    for j in 0..expected.len().min(actual.len()) {
                        if actual[j] == expected[j] {
                            case_score += 1.0 / n as f32;
                        }
                    }
                    total += case_score;
                }
            }
        }
    }
    total / test_cases.len() as f32
}

impl ExecutionService for IrisExecutionService {
    fn evaluate_individual(
        &self,
        program: &SemanticGraph,
        test_inputs: &[TestCase],
        tier: EvalTier,
    ) -> Result<EvalResult, EvalError> {
        self.eval_one(program, test_inputs, tier)
    }

    fn evaluate_batch(
        &self,
        programs: &[SemanticGraph],
        test_inputs: &[TestCase],
        tier: EvalTier,
    ) -> Result<Vec<EvalResult>, EvalError> {
        let mut results = Vec::with_capacity(programs.len());
        for graph in programs {
            results.push(self.eval_one(graph, test_inputs, tier)?);
        }
        Ok(results)
    }

    fn evict_cache(&self, _graph_ids: &[FragmentId]) {}

    fn cache_stats(&self) -> CacheStats {
        CacheStats {
            hits: 0,
            misses: 0,
            evictions: 0,
            current_entries: 0,
            max_entries: 0,
        }
    }
}
