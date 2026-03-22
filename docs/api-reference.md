# IRIS API Reference

This document covers the public API of the key IRIS crates. All types and functions described here are verified against the actual source code.

---

## iris-types

Core data structures shared across all IRIS crates.

**Source:** `src/iris-types/`

### SemanticGraph (`graph.rs`)

The canonical program representation.

```rust
pub struct SemanticGraph {
    pub root: NodeId,
    pub nodes: HashMap<NodeId, Node>,
    pub edges: Vec<Edge>,
    pub type_env: TypeEnv,
    pub cost: CostBound,
    pub resolution: Resolution,
    pub hash: SemanticHash,
}
```

Methods:
- `sorted_nodes() -> Vec<(&NodeId, &Node)>` -- Iterate nodes in deterministic order (sorted by NodeId)
- `sorted_node_ids() -> Vec<NodeId>` -- Return node IDs in deterministic sorted order

### Node (`graph.rs`)

```rust
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,        // 5-bit tag, 20 variants
    pub type_sig: TypeRef,     // Index into TypeEnv
    pub cost: CostTerm,        // Per-node cost annotation
    pub arity: u8,
    pub resolution_depth: u8,
    pub salt: u64,             // Disambiguation salt
    pub payload: NodePayload,  // Kind-specific data
}
```

### NodeKind (`graph.rs`)

20 variants: `Prim`, `Apply`, `Lambda`, `Let`, `Match`, `Lit`, `Ref`, `Neural`, `Fold`, `Unfold`, `Effect`, `Tuple`, `Inject`, `Project`, `TypeAbst`, `TypeApp`, `LetRec`, `Guard`, `Rewrite`, `Extern`.

### Value (`eval.rs`)

Runtime value type with 14 variants:

```rust
pub enum Value {
    Int(i64),
    Nat(u64),
    Float64(f64),
    Float32(f32),
    Bool(bool),
    Bytes(Vec<u8>),
    Unit,
    Tuple(Vec<Value>),
    Tagged(u16, Box<Value>),
    State(StateStore),
    Graph(KnowledgeGraph),
    Program(Box<SemanticGraph>),
    Future(FutureHandle),
    String(String),
}
```

### Fragment (`fragment.rs`)

Self-contained holographic unit (the genome IS a Fragment):

```rust
pub struct Fragment {
    pub id: FragmentId,                  // BLAKE3 of (graph, boundary, type_env, imports)
    pub graph: SemanticGraph,
    pub boundary: Boundary,              // Typed inputs and outputs
    pub type_env: TypeEnv,
    pub imports: Vec<FragmentRef>,       // Dependencies by hash
    pub metadata: FragmentMeta,
    pub proof: Option<ProofReceipt>,
    pub contracts: FragmentContracts,    // requires/ensures
}
```

### TypeDef (`types.rs`)

11 variants for the type system:

```rust
pub enum TypeDef {
    Primitive(PrimType),           // Int, Nat, Float64, Float32, Bool, Bytes, Unit
    Product(Vec<TypeId>),          // Tuple
    Sum(Vec<(Tag, TypeId)>),       // Tagged union
    Recursive(BoundVar, TypeId),   // mu X. F(X)
    ForAll(BoundVar, TypeId),      // Polymorphism
    Arrow(TypeId, TypeId, CostBound),   // Function with cost
    Refined(TypeId, RefinementPredicate),  // Refinement type
    NeuralGuard(TypeId, TypeId, GuardSpec, CostBound),
    Exists(BoundVar, TypeId),      // Existential type
    Vec(TypeId, SizeTerm),         // Sized vector
    HWParam(TypeId, HardwareProfile),   // HW-parameterized
}
```

### CostBound (`cost.rs`)

13-variant algebraic cost model:

```rust
pub enum CostBound {
    Unknown, Zero, Constant(u64),
    Linear(CostVar), NLogN(CostVar), Polynomial(CostVar, u32),
    Sum(Box<CostBound>, Box<CostBound>),
    Par(Box<CostBound>, Box<CostBound>),
    Mul(Box<CostBound>, Box<CostBound>),
    Amortized(Box<CostBound>, PotentialFn),
    HWScaled(Box<CostBound>, HWParamRef),
    Sup(Vec<CostBound>),
    Inf(Vec<CostBound>),
}
```

Free function:
- `universalize_cost(cost: &CostBound) -> CostBound` -- Strip all hardware-specific annotations

### EffectTag (`eval.rs`)

Categorizes I/O effects. 43 named variants plus `Custom(u8)`.

Key trait:
```rust
pub trait EffectHandler: Send + Sync {
    fn handle(&self, request: EffectRequest) -> Result<Value, EffectError>;
}
```

### Hashing (`hash.rs`)

- `compute_node_id(node: &Node) -> NodeId` -- BLAKE3 hash truncated to 64 bits
- `compute_type_id(td: &TypeDef) -> TypeId` -- BLAKE3 hash truncated to 64 bits
- `compute_fragment_id(fragment: &Fragment) -> FragmentId` -- Full 256-bit BLAKE3

---

## iris-exec

Execution service: interpreter, VM, JIT, effects, capabilities.

**Source:** `src/iris-exec/`

### ExecutionService trait (`lib.rs`)

```rust
pub trait ExecutionService {
    fn evaluate_individual(
        &self,
        program: &SemanticGraph,
        test_inputs: &[TestCase],
        tier: EvalTier,
    ) -> Result<EvalResult, EvalError>;

    fn evaluate_batch(
        &self,
        programs: &[SemanticGraph],
        test_inputs: &[TestCase],
        tier: EvalTier,
    ) -> Result<Vec<EvalResult>, EvalError>;

    fn evict_cache(&self, graph_ids: &[FragmentId]);
    fn cache_stats(&self) -> CacheStats;
}
```

### MetaEvolver trait (`lib.rs`)

```rust
pub trait MetaEvolver: Send + Sync {
    fn evolve_subprogram(
        &self,
        test_cases: Vec<TestCase>,
        max_generations: usize,
        meta_depth: u32,
    ) -> Result<SemanticGraph, String>;
}
```

### Interpreter (`interpreter.rs`)

The tree-walking interpreter. Main entry point:

```rust
pub fn interpret(
    graph: &SemanticGraph,
    inputs: &[Value],
    effect_handler: Option<&dyn EffectHandler>,
) -> Result<(Vec<Value>, StateStore), InterpretError>;

pub fn interpret_with_registry(
    graph: &SemanticGraph,
    inputs: &[Value],
    effect_handler: Option<&dyn EffectHandler>,
    registry: Option<&FragmentRegistry>,
) -> Result<(Vec<Value>, StateStore), InterpretError>;
```

The interpreter handles all 20 node kinds, effects, self-modification (`graph_eval`), meta-evolution (`evolve_subprogram`), threading, FFI, and capabilities.

### Bytecode VM (`vm.rs`)

Stack-machine VM that executes compiled bytecode:

```rust
impl VM {
    pub fn execute(
        bytecode: &Bytecode,
        args: &[Value],
        max_steps: u64,
        memory_limit: usize,
    ) -> Result<Value, VMError>;

    pub fn execute_with_graph(
        bytecode: &Bytecode,
        args: &[Value],
        max_steps: u64,
        memory_limit: usize,
        source_graph: &SemanticGraph,
    ) -> Result<Value, VMError>;
}
```

Produces identical results to the tree-walker; exists purely as a performance optimization.

### JIT Compiler (`jit.rs`)

x86-64 only. Compiles bytecode to native machine code:

```rust
pub fn jit_compile(bytecode: &Bytecode) -> Result<JitCode, JitError>;
pub fn jit_execute(code: &JitCode, inputs: &[Value]) -> Result<Value, JitError>;
```

Uses raw `mmap`/`mprotect` syscalls (no libc dependency). Supports arithmetic, comparison, FoldPrim, FoldLambda, MapPrim, FilterPrim, Call/Return.

### IrisExecutionService (`service.rs`)

Concrete implementation of `ExecutionService`:

```rust
pub struct IrisExecutionService { ... }

impl IrisExecutionService {
    pub fn new(config: ExecConfig) -> Self;
    pub fn with_defaults() -> Self;
}
```

Configuration via `SandboxConfig`:

```rust
pub struct SandboxConfig {
    pub memory_limit_bytes: usize,  // Default: 256 MB
    pub step_limit: u64,            // Default: 100,000
    pub timeout_ms: u64,            // Default: 5,000 ms
}
```

### IrisDaemon (`daemon.rs`)

Continuous execution engine:

```rust
pub struct IrisDaemon { ... }
pub struct DaemonConfig {
    pub cycle_time_ms: u64,         // Default: 800
    pub max_cycles: Option<u64>,
    pub programs: Vec<(String, SemanticGraph)>,
    pub channels: Vec<(String, usize)>,
    pub enable_evolution: bool,
    pub on_cycle: Option<Box<dyn Fn(CycleReport) + Send>>,
}
```

### Effect Handlers (`effects.rs`)

Built-in handlers:

| Handler | Purpose |
|---------|---------|
| `NoOpHandler` | Returns Unit for everything (safe for evolution) |
| `LoggingHandler` | Captures effect requests into inspectable log |
| `RealHandler` | Performs actual I/O |
| `SandboxedHandler` | Wraps handler with allow/deny lists |
| `CompositeHandler` | Chains multiple handlers |

### Capabilities (`capabilities.rs`)

```rust
pub struct Capabilities {
    pub allowed_effects: HashSet<EffectTag>,
    pub max_memory: usize,
    pub max_steps: u64,
    pub max_wall_time: Duration,
    pub allowed_paths: Vec<String>,    // Filesystem glob patterns
    pub allowed_hosts: Vec<String>,    // Network hosts
    pub can_spawn_threads: bool,
    pub can_ffi: bool,
    pub can_mmap_exec: bool,
}

impl Capabilities {
    pub fn unrestricted() -> Self;
    pub fn sandboxed() -> Self;
    pub fn none() -> Self;
}
```

### Bytecode Compiler (`compile_bytecode.rs`)

```rust
pub fn compile_to_bytecode(graph: &SemanticGraph) -> Result<Bytecode, CompileError>;
```

Supports: Lit, Prim, Fold, Tuple, Guard, Project, Match, Inject, TypeAbst, TypeApp, Rewrite. Falls back to tree-walker for unsupported kinds.

### FragmentRegistry (`registry.rs`)

Registry for resolving cross-fragment `Ref` nodes:

```rust
pub struct FragmentRegistry { ... }

impl FragmentRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, fragment: Fragment);
    pub fn lookup(&self, id: &FragmentId) -> Option<&Fragment>;
}
```

### MessageBus (`message_bus.rs`)

IPC message passing between programs:

```rust
pub struct MessageBus { ... }

impl MessageBus {
    pub fn new() -> Self;
    pub fn create_channel(&self, name: &str, capacity: usize);
    pub fn send(&self, channel: &str, value: Value) -> Result<(), String>;
    pub fn recv(&self, channel: &str) -> Result<Value, String>;
}
```

---

## iris-evolve

Multi-objective evolutionary engine.

**Source:** `src/iris-evolve/`

### evolve() (`lib.rs`)

Main entry point for evolution:

```rust
pub fn evolve(
    config: EvolutionConfig,
    spec: ProblemSpec,
    exec: &dyn ExecutionService,
) -> EvolutionResult;
```

Internally:
1. Launches bottom-up enumeration in a background thread
2. Analyzes test cases and generates matching seed skeletons
3. Initializes multi-deme population
4. Runs NSGA-II generations with mutation, crossover, selection
5. Applies migration, death/compression, novelty scoring
6. Returns best individual and Pareto front

### EvolutionConfig (`config.rs`)

```rust
pub struct EvolutionConfig {
    pub population_size: usize,        // Default: 64
    pub max_generations: usize,        // Default: 1000
    pub mutation_rate: f64,            // Default: [0, 1]
    pub crossover_rate: f64,           // Default: [0, 1]
    pub tournament_size: usize,
    pub phase_thresholds: PhaseThresholds,
    pub target_generation_time_ms: u64,
    pub num_demes: usize,              // Default: 1
    pub novelty_k: usize,             // Default: 15
    pub novelty_threshold: f32,        // Default: 0.1
    pub novelty_weight: f32,           // Default: 1.0
    pub coevolution: bool,             // Default: false
    pub resource_budget_ms: u64,       // Default: 0 (disabled)
    pub iris_mode: bool,               // Default: false
}
```

### ProblemSpec (`config.rs`)

```rust
pub struct ProblemSpec {
    pub test_cases: Vec<TestCase>,
    pub description: String,
    pub target_cost: Option<CostBound>,
}
```

### IrisRuntime (`iris_runtime.rs`)

IRIS runtime for autopoietic closure (IRIS programs evolving IRIS programs):

When `iris_mode` is enabled, evolution uses IRIS programs for mutation, evaluation, and selection instead of Rust functions.

### SelfImprovingDaemon (`self_improving_daemon.rs`)

Continuous self-improvement:

```rust
pub struct SelfImprovingDaemon { ... }
pub struct SelfImprovingConfig {
    pub cycle_time_ms: u64,
    pub max_cycles: Option<u64>,
    pub improve_interval: u64,
    pub inspect_interval: u64,
    pub auto_improve: AutoImproveConfig,
    pub state_dir: Option<PathBuf>,
    pub memory_limit: usize,
    pub seed: Option<u64>,
    pub max_improve_threads: usize,
    pub max_stagnant: u32,
    pub min_improvement: f64,
    pub exec_mode: ExecMode,
    pub trigger_check_interval: u64,
}

impl SelfImprovingDaemon {
    pub fn new(config: SelfImprovingConfig) -> Self;
    pub fn run(&mut self) -> DaemonResult;
}
```

### Mutation Operators (`mutation.rs`)

16 operators with configurable weights:

| # | Operator | Weight | Description |
|---|----------|--------|-------------|
| 0 | `insert_node` | 10% | Insert a new node into the graph |
| 1 | `delete_node` | 10% | Remove a node |
| 2 | `rewire_edge` | 8% | Change an edge's target |
| 3 | `replace_kind` | 3% | Change a node's kind |
| 4 | `replace_prim` | 6% | Replace a primitive opcode |
| 5 | `mutate_literal` | 7% | Modify a literal value |
| 6 | `duplicate_subgraph` | 1% | Copy a subtree |
| 7 | `wrap_in_guard` | 1% | Add a guard node |
| 8 | `annotate_cost` | 1% | Add/change cost annotation |
| 9 | `wrap_in_map` | 9% | Wrap computation in map |
| 10 | `wrap_in_filter` | 9% | Wrap in filter |
| 11 | `compose_stages` | 7% | Compose two stages |
| 12 | `insert_zip` | 7% | Insert zip operation |
| 13 | `swap_fold_op` | 13% | Change fold's operator |
| 14 | `add_guard_condition` | 5% | Add guard condition |
| 15 | `extract_to_ref` | 3% | Extract subtree to ref |

Custom weights can be installed per-thread for self-improvement.

---

## iris-bootstrap::syntax

Surface syntax: lexer, parser, lowerer. (Merged from the former `iris-syntax` crate.)

**Source:** `src/iris-bootstrap/src/syntax/`

### parse() (`lib.rs`)

```rust
pub fn parse(source: &str) -> Result<ast::Module, SyntaxError>;
```

### compile() (`lib.rs`)

Parse + lower to SemanticGraph fragments:

```rust
pub fn compile(source: &str) -> CompileResult;

pub struct CompileResult {
    pub fragments: Vec<(String, Fragment, SourceMap)>,
    pub errors: Vec<SyntaxError>,
}
```

### compile_and_verify() (`lib.rs`)

Parse, lower, and verify:

```rust
pub fn compile_and_verify(
    source: &str,
    tier: VerifyTier,
) -> (CompileResult, String);
```

### compile_module() (`lower.rs`)

Lower an AST Module to fragments:

```rust
pub fn compile_module(module: &Module) -> CompileResult;
```

### Primitive Resolution (`prim.rs`)

```rust
pub fn resolve_primitive(name: &str) -> Option<(u8, u8)>;
// Returns (opcode, arity) for named primitives
```

---

## iris-bootstrap::syntax::kernel

LCF-style proof kernel. (Merged from the former `iris-kernel` crate.)

**Source:** `src/iris-bootstrap/src/syntax/kernel/`

### type_check() (`checker.rs`)

```rust
pub fn type_check(graph: &SemanticGraph) -> Result<Theorem, CheckError>;
```

### type_check_graded() (`checker.rs`)

Graded verification with partial credit:

```rust
pub fn type_check_graded(
    graph: &SemanticGraph,
    tier: VerifyTier,
) -> VerificationReport;

pub struct VerificationReport {
    pub total_obligations: usize,
    pub satisfied: usize,
    pub failed: Vec<(NodeId, CheckError)>,
    pub tier: VerifyTier,
    pub partial_proof: Option<ProofTree>,
    pub score: f32,   // 0.0 - 1.0
}
```

### minimum_tier() (`checker.rs`)

Auto-detect the minimum verification tier:

```rust
pub fn minimum_tier(graph: &SemanticGraph) -> VerifyTier;
```

### diagnose() (`checker.rs`)

Produce mutation hints from proof failures:

```rust
pub fn diagnose(
    graph: &SemanticGraph,
    report: &VerificationReport,
) -> Vec<ProofFailureDiagnosis>;
```

### Kernel (`kernel.rs`)

The 20 inference rules. Zero unsafe Rust:

```rust
pub struct Kernel;

impl Kernel {
    pub fn assume(...) -> Result<Theorem, KernelError>;
    pub fn intro(...) -> Result<Theorem, KernelError>;
    pub fn elim(...) -> Result<Theorem, KernelError>;
    pub fn refl(...) -> Result<Theorem, KernelError>;
    pub fn symm(...) -> Result<Theorem, KernelError>;
    pub fn trans(...) -> Result<Theorem, KernelError>;
    pub fn congr(...) -> Result<Theorem, KernelError>;
    pub fn type_check_node(...) -> Result<Theorem, KernelError>;
    pub fn cost_subsume(...) -> Result<Theorem, KernelError>;
    pub fn cost_leq_rule(...) -> Result<Theorem, KernelError>;
    pub fn refine_intro(...) -> Result<Theorem, KernelError>;
    pub fn refine_elim(...) -> Result<Theorem, KernelError>;
    pub fn nat_ind(...) -> Result<Theorem, KernelError>;
    pub fn structural_ind(...) -> Result<Theorem, KernelError>;
    pub fn let_bind(...) -> Result<Theorem, KernelError>;
    pub fn match_elim(...) -> Result<Theorem, KernelError>;
    pub fn fold_rule(...) -> Result<Theorem, KernelError>;
    pub fn type_abst(...) -> Result<Theorem, KernelError>;
    pub fn type_app(...) -> Result<Theorem, KernelError>;
    pub fn guard_rule(...) -> Result<Theorem, KernelError>;
}
```

### ZK Proofs (`zk.rs`)

```rust
pub fn generate_zk_proof(...) -> Result<ZkProof, ZkError>;
pub fn verify_zk_proof(...) -> Result<bool, ZkError>;
pub fn prove_program(...) -> Result<ZkProof, ZkError>;
pub fn verify_listing(...) -> Result<MarketVerification, ZkError>;
```

---

## iris-bootstrap

Minimal evaluator for bootstrapping.

**Source:** `src/iris-bootstrap/`

### evaluate() (`lib.rs`)

~500 LOC tree-walking evaluator supporting the subset of node kinds needed to run the IRIS interpreter written in IRIS:

```rust
pub fn evaluate(
    graph: &SemanticGraph,
    inputs: &[Value],
) -> Result<Vec<Value>, BootstrapError>;
```

Supported node kinds: Lit, Prim, Guard, Fold, Lambda/Apply, Let, Ref, Tuple.

---

## Removed Crates

The following crates were consolidated into the 5-crate workspace during the self-hosting effort. Their functionality is now provided by IRIS programs or merged into surviving crates:

| Former Crate | Status |
|-------------|--------|
| `iris-repr` | Renamed to `iris-types` |
| `iris-kernel` | Merged into `iris-bootstrap::syntax::kernel` |
| `iris-syntax` | Merged into `iris-bootstrap::syntax` |
| `iris-compiler` | Types merged into `iris-types::compiler_ir`; behavior replaced by IRIS programs |
| `iris-codec` | Types merged into `iris-types::codec`; behavior replaced by IRIS programs |
| `iris-deploy` | Replaced by IRIS programs in `src/iris-programs/deploy/` |
| `iris-store` | Replaced by IRIS programs in `src/iris-programs/store/` |
| `iris-foundry` | Replaced by IRIS programs in `src/iris-programs/foundry/` |
| `iris-lsp` | Replaced by IRIS programs |
