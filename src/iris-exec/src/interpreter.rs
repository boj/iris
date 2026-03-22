//! Interpreter module — thin shims delegating to `iris-bootstrap`.

use std::fmt;

use iris_types::eval::{
    EffectHandler, EffectTag, StateStore, Value,
};
use iris_types::graph::{
    EdgeLabel, NodeId, SemanticGraph,
};

use crate::capabilities::Capabilities;
use crate::message_bus::MessageBus;
use crate::registry::FragmentRegistry;
use crate::MetaEvolver;

#[derive(Debug, Clone)]
pub enum InterpretError {
    MissingNode(NodeId),
    MissingEdge {
        source: NodeId,
        port: u8,
        label: EdgeLabel,
    },
    TypeError(String),
    DivisionByZero,
    UnknownOpcode(u8),
    Unsupported(String),
    RecursionLimit { depth: u32, limit: u32 },
    Timeout { steps: u64, limit: u64 },
    MalformedLiteral { type_tag: u8, len: usize },
    EffectFailed { tag: String, message: String },
    SelfEvalDepthExceeded { depth: u32, limit: u32 },
    MetaEvolveDepthExceeded { depth: u32, limit: u32 },
    MetaEvolveFailed(String),
    MemoryExceeded { used: usize, limit: usize },
    PermissionDenied { effect: EffectTag },
}

impl fmt::Display for InterpretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingNode(id) => write!(f, "missing node: {:?}", id),
            Self::MissingEdge { source, port, label } => {
                write!(f, "missing edge from {:?} port {} label {:?}", source, port, label)
            }
            Self::TypeError(msg) => write!(f, "type error: {}", msg),
            Self::DivisionByZero => write!(f, "division by zero"),
            Self::UnknownOpcode(op) => write!(f, "unknown opcode: 0x{:02x}", op),
            Self::Unsupported(what) => write!(f, "unsupported: {}", what),
            Self::RecursionLimit { depth, limit } => {
                write!(f, "recursion depth {} exceeded limit {}", depth, limit)
            }
            Self::Timeout { steps, limit } => {
                write!(f, "step count {} exceeded limit {}", steps, limit)
            }
            Self::MalformedLiteral { type_tag, len } => {
                write!(f, "malformed literal: type_tag=0x{:02x}, payload len={}", type_tag, len)
            }
            Self::EffectFailed { tag, message } => {
                write!(f, "effect {} failed: {}", tag, message)
            }
            Self::SelfEvalDepthExceeded { depth, limit } => {
                write!(f, "self-eval depth {} exceeded limit {}", depth, limit)
            }
            Self::MetaEvolveDepthExceeded { depth, limit } => {
                write!(f, "meta-evolve depth {} exceeded limit {}", depth, limit)
            }
            Self::MetaEvolveFailed(msg) => {
                write!(f, "meta-evolve failed: {}", msg)
            }
            Self::MemoryExceeded { used, limit } => {
                write!(f, "memory limit exceeded: {} bytes used, limit {} bytes", used, limit)
            }
            Self::PermissionDenied { effect } => {
                write!(f, "permission denied: effect {:?} not allowed by capabilities", effect)
            }
        }
    }
}

impl std::error::Error for InterpretError {}

pub const MAX_STEPS: u64 = 10_000_000;
pub const DEFAULT_MEMORY_LIMIT: usize = 256 * 1024 * 1024;
pub const ENUMERATE_MEMORY_LIMIT: usize = 16 * 1024 * 1024;

fn from_bootstrap(e: iris_bootstrap::BootstrapError) -> InterpretError {
    match e {
        iris_bootstrap::BootstrapError::MissingNode(id) => InterpretError::MissingNode(id),
        iris_bootstrap::BootstrapError::MissingEdge { source, port, label } => {
            InterpretError::MissingEdge { source, port, label }
        }
        iris_bootstrap::BootstrapError::TypeError(ref msg)
            if msg.contains("permission denied") =>
        {
            // Bootstrap wraps CapabilityGuardHandler errors as
            // TypeError("effect 0xNN failed: ... permission denied ...").
            // Extract the effect tag and map to PermissionDenied.
            let tag = extract_effect_tag_from_error(msg);
            InterpretError::PermissionDenied { effect: tag }
        }
        iris_bootstrap::BootstrapError::TypeError(ref msg)
            if msg.starts_with("effect 0x") && msg.contains("failed:") =>
        {
            // Other effect failures that aren't permission-denied.
            let tag_str = msg.split_whitespace().nth(1).unwrap_or("0x00");
            let tag_msg = msg.splitn(2, "failed: ").nth(1).unwrap_or(msg);
            InterpretError::EffectFailed {
                tag: tag_str.to_string(),
                message: tag_msg.to_string(),
            }
        }
        iris_bootstrap::BootstrapError::TypeError(msg) => InterpretError::TypeError(msg),
        iris_bootstrap::BootstrapError::DivisionByZero => InterpretError::DivisionByZero,
        iris_bootstrap::BootstrapError::UnknownOpcode(op) => InterpretError::UnknownOpcode(op),
        iris_bootstrap::BootstrapError::Unsupported(what) => InterpretError::Unsupported(what),
        iris_bootstrap::BootstrapError::RecursionLimit { depth, limit } => {
            InterpretError::RecursionLimit { depth, limit }
        }
        iris_bootstrap::BootstrapError::Timeout { steps, limit } => {
            InterpretError::Timeout { steps, limit }
        }
        iris_bootstrap::BootstrapError::MalformedLiteral { type_tag, len } => {
            InterpretError::MalformedLiteral { type_tag, len }
        }
    }
}

/// Extract an EffectTag from a bootstrap error message like
/// "effect 0x05 failed: ... permission denied: effect FileWrite ..."
fn extract_effect_tag_from_error(msg: &str) -> EffectTag {
    // Try to find "effect 0xNN" at the start.
    if let Some(hex) = msg.strip_prefix("effect 0x") {
        if let Some(hex_str) = hex.get(..2) {
            if let Ok(tag_byte) = u8::from_str_radix(hex_str, 16) {
                return EffectTag::from_u8(tag_byte);
            }
        }
    }
    // Fallback: look for known tag names in the message.
    for (name, tag) in &[
        ("FileWrite", EffectTag::FileWrite),
        ("FileRead", EffectTag::FileRead),
        ("TcpConnect", EffectTag::TcpConnect),
        ("ThreadSpawn", EffectTag::ThreadSpawn),
        ("MmapExec", EffectTag::MmapExec),
        ("FfiCall", EffectTag::FfiCall),
        ("EnvGet", EffectTag::EnvGet),
    ] {
        if msg.contains(name) {
            return *tag;
        }
    }
    EffectTag::Custom(0xFF)
}

pub fn interpret(
    graph: &SemanticGraph,
    inputs: &[Value],
    state: Option<&mut StateStore>,
) -> Result<(Vec<Value>, StateStore), InterpretError> {
    interpret_with_registry(graph, inputs, state, None)
}

pub fn interpret_with_registry(
    graph: &SemanticGraph,
    inputs: &[Value],
    state: Option<&mut StateStore>,
    _registry: Option<&FragmentRegistry>,
) -> Result<(Vec<Value>, StateStore), InterpretError> {
    let max_steps = MAX_STEPS * (1 + graph.nodes.len() as u64 / 100);
    interpret_with_step_limit(graph, inputs, state, _registry, max_steps)
}

pub fn interpret_with_step_limit(
    graph: &SemanticGraph,
    inputs: &[Value],
    state: Option<&mut StateStore>,
    _registry: Option<&FragmentRegistry>,
    max_steps: u64,
) -> Result<(Vec<Value>, StateStore), InterpretError> {
    interpret_with_effects(graph, inputs, state, _registry, max_steps, None)
}

pub fn interpret_with_effects(
    graph: &SemanticGraph,
    inputs: &[Value],
    state: Option<&mut StateStore>,
    _registry: Option<&FragmentRegistry>,
    max_steps: u64,
    _effect_handler: Option<&dyn EffectHandler>,
) -> Result<(Vec<Value>, StateStore), InterpretError> {
    interpret_with_bus(graph, inputs, state, _registry, max_steps, _effect_handler, None)
}

pub fn interpret_with_bus(
    graph: &SemanticGraph,
    inputs: &[Value],
    state: Option<&mut StateStore>,
    _registry: Option<&FragmentRegistry>,
    max_steps: u64,
    _effect_handler: Option<&dyn EffectHandler>,
    _bus: Option<&mut MessageBus>,
) -> Result<(Vec<Value>, StateStore), InterpretError> {
    interpret_with_meta_evolver(
        graph, inputs, state, _registry, max_steps,
        _effect_handler, _bus, None, 0,
    )
}

pub fn interpret_with_meta_evolver(
    graph: &SemanticGraph,
    inputs: &[Value],
    state: Option<&mut StateStore>,
    _registry: Option<&FragmentRegistry>,
    max_steps: u64,
    _effect_handler: Option<&dyn EffectHandler>,
    _bus: Option<&mut MessageBus>,
    _meta_evolver: Option<&dyn MetaEvolver>,
    _meta_evolve_depth: u32,
) -> Result<(Vec<Value>, StateStore), InterpretError> {
    interpret_sandboxed(
        graph, inputs, state, _registry, max_steps, 0,
        _effect_handler, _bus, _meta_evolver, _meta_evolve_depth,
    )
}

pub fn interpret_sandboxed(
    graph: &SemanticGraph,
    inputs: &[Value],
    state: Option<&mut StateStore>,
    _registry: Option<&FragmentRegistry>,
    max_steps: u64,
    _memory_limit: usize,
    effect_handler: Option<&dyn EffectHandler>,
    _bus: Option<&mut MessageBus>,
    _meta_evolver: Option<&dyn MetaEvolver>,
    _meta_evolve_depth: u32,
) -> Result<(Vec<Value>, StateStore), InterpretError> {
    let final_state = state.map(|s| std::mem::take(s)).unwrap_or_default();

    let graph_map = _registry
        .map(|r| r.to_graph_map())
        .unwrap_or_default();

    // Build evolve callback if a MetaEvolver is provided
    let evolve_closure;
    let evolve_fn: Option<&iris_bootstrap::EvolveFn<'_>> = match _meta_evolver {
        Some(evolver) => {
            let depth = _meta_evolve_depth;
            evolve_closure = move |test_cases: Vec<(Vec<Value>, Value)>, max_gens: usize, _inner_depth: u32| {
                use iris_types::eval::TestCase;
                let tc: Vec<TestCase> = test_cases.into_iter().map(|(inputs, expected)| {
                    TestCase {
                        inputs,
                        expected_output: Some(vec![expected]),
                        initial_state: None,
                        expected_state: None,
                    }
                }).collect();
                evolver.evolve_subprogram(tc, max_gens, depth)
            };
            Some(&evolve_closure as &iris_bootstrap::EvolveFn<'_>)
        }
        None => None,
    };

    let val = iris_bootstrap::evaluate_with_evolver(
        graph, inputs, max_steps, &graph_map,
        effect_handler, evolve_fn,
    ).map_err(from_bootstrap)?;

    Ok((vec![val], final_state))
}

pub fn interpret_with_capabilities(
    graph: &SemanticGraph,
    inputs: &[Value],
    state: Option<&mut StateStore>,
    _registry: Option<&FragmentRegistry>,
    effect_handler: Option<&dyn EffectHandler>,
    _bus: Option<&mut MessageBus>,
    _meta_evolver: Option<&dyn MetaEvolver>,
    _meta_evolve_depth: u32,
    capabilities: Capabilities,
) -> Result<(Vec<Value>, StateStore), InterpretError> {
    let max_steps = if capabilities.max_steps > 0 {
        capabilities.max_steps
    } else {
        MAX_STEPS * (1 + graph.nodes.len() as u64 / 100)
    };

    // Create the capability-guarded handler. If no inner handler is provided,
    // use a RuntimeEffectHandler that performs real I/O.
    let runtime_handler;
    let inner: &dyn EffectHandler = match effect_handler {
        Some(h) => h,
        None => {
            runtime_handler = crate::effect_runtime::RuntimeEffectHandler::new();
            &runtime_handler
        }
    };
    let guarded = crate::capabilities::CapabilityGuardHandler::new(inner, capabilities);
    interpret_sandboxed(
        graph, inputs, state, _registry, max_steps, 0,
        Some(&guarded), _bus, _meta_evolver, _meta_evolve_depth,
    )
}
