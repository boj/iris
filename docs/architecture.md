# IRIS Architecture Guide

## Overview

IRIS is a self-improving programming language designed around evolutionary program synthesis. Programs are represented as content-addressed semantic graphs, evolved via multi-objective optimization, verified by an LCF-style proof kernel, and executed through a tiered evaluator pipeline: tree-walker → flat evaluator → native x86-64 AVX codegen, all with capability-guarded effects.

The system is implemented as a Rust workspace (5 crates, ~51K LOC) with a C library (`iris-clcu/`) for vectorized execution, and 256 `.iris` programs (34K+ LOC) covering all system components.

---

## The 4-Layer Stack

```
+---------------------------------------------+
|  L0: Evolution                               |
|  (iris-evolve)                               |
|  NSGA-II, mutation, crossover, selection,    |
|  multi-deme migration, novelty search,       |
|  coevolution, self-improvement               |
+---------------------------------------------+
|  L1: Semantics                               |
|  (iris-types, iris-bootstrap::syntax)        |
|  SemanticGraph, TypeEnv, CostBound,          |
|  Fragment, Value, EffectTag                  |
+---------------------------------------------+
|  L2: Verification                            |
|  (iris-bootstrap::syntax::kernel)            |
|  LCF proof kernel, 20 inference rules,       |
|  type checker, cost checker, LIA solver,     |
|  ZK proofs                                   |
+---------------------------------------------+
|  L3: Hardware Materialization                 |
|  (iris-exec, iris-clcu-sys)                  |
|  Tree-walker, flat eval, native x86-64 AVX,  |
|  capability-guarded effects, CLCU (AVX-512)  |
+---------------------------------------------+
```

### L0: Evolution (iris-evolve)

The evolutionary substrate breeds programs to satisfy specifications. Key components:

- **NSGA-II**: Multi-objective optimization balancing correctness, performance, and verifiability
- **Multi-deme**: Multiple populations with ring migration every 5 generations
- **Mutation operators**: 16 operators including insert_node, delete_node, rewire_edge, replace_prim, wrap_in_map, swap_fold_op, etc.
- **Crossover**: Embedding-based crossover for semantic preservation
- **Selection**: Tournament selection and lexicase selection (per-test-case scoring)
- **Novelty search**: k-nearest-neighbor behavioral novelty to escape local optima
- **Coevolution**: Programs and test cases co-evolve (arms race)
- **Resource competition**: Programs compete for evaluation time based on fitness rank
- **Phase detection**: Tracks evolutionary phase transitions
- **Bottom-up enumeration**: Exhaustive search for small programs (<=8 nodes)
- **Test case analysis**: Detects problem patterns (sum, max, linear, product, etc.) and generates matching seed skeletons

### L1: Semantics (iris-types, iris-bootstrap::syntax)

The canonical representation layer. All programs, types, and values are defined here.

- **SemanticGraph**: The universal program representation (20 node kinds, edges with labels)
- **Fragment**: Self-contained holographic unit with graph, boundary, type env, imports, and proof
- **TypeEnv**: Content-addressed map of type definitions (11 TypeDef variants)
- **CostBound**: 13-variant algebraic cost model
- **Value**: 14-variant runtime value type
- **EffectTag**: 44 categorized I/O effects with handler trait
- **Surface syntax**: ML-like language (iris-bootstrap::syntax) that parses and lowers to SemanticGraph
- **Wire format**: Binary serialization for fragments and bundles

### L2: Verification (iris-bootstrap::syntax::kernel)

The trusted computing base. Only this code can construct `Theorem` values.

- **20 inference rules**: Variable, lambda intro/elim, reflexivity, symmetry, transitivity, congruence, type_check_node, cost_subsume, refinement intro/elim, natural induction, structural induction, let binding, match elimination, fold rule, type abstraction/application, guard rule
- **LCF architecture**: `Theorem` is an opaque type; only `Kernel` methods can construct it
- **Zero unsafe Rust**: The entire proof kernel contains no unsafe blocks
- **Cost checker**: Conservative partial order on `CostBound`
- **LIA solver**: Quantifier-free linear integer arithmetic for refinement types
- **Graded verification**: Partial credit scoring instead of pass/fail
- **ZK proofs**: Zero-knowledge proof generation and verification for program properties

### L3: Hardware Materialization (iris-exec, iris-clcu-sys)

Executes programs with capability-guarded effects.

- **Tree-walking evaluator**: The bootstrap evaluator handles all 20 node kinds and dispatches 44 effect tags
- **Flat evaluator**: Flattens fold bodies into `FlatProgram` (linear op sequence), eliminating tree-walker overhead for numeric loops. ~578× faster than tree-walker for Float64 workloads
- **Native x86-64 codegen**: Compiles FlatPrograms to AVX machine code with linear-scan register allocation, loop-carried state (xmm0–xmm14), copy propagation, and Lit-aware spill eviction. Achieves **0.22 µs/step** on n-body (~11,800× vs tree-walker, ~45× faster than CPython)
- **RuntimeEffectHandler**: Implements real I/O (files, TCP, env, time, random, atomic state)
- **CapabilityGuardHandler**: Wraps effect handlers with fine-grained permission enforcement
- **Execution shim**: `iris-exec` provides `IrisExecutionService` with sandbox defaults
- **CLCU**: AVX-512 cache-line containers for vectorized computation via C library

---

## SemanticGraph: The 20 Node Kinds

The `SemanticGraph` is the canonical program representation. It is a directed graph where each node has a 5-bit kind tag, type signature, cost annotation, and kind-specific payload.

Defined in `src/iris-types/src/graph.rs`:

| Kind | Tag | Payload | Purpose |
|------|-----|---------|---------|
| `Prim` | 0x00 | `opcode: u8` | Primitive operation |
| `Apply` | 0x01 | (none) | Function application |
| `Lambda` | 0x02 | `binder, captured_count` | Function abstraction |
| `Let` | 0x03 | (none) | Local binding |
| `Match` | 0x04 | `arm_count, arm_patterns` | Pattern matching |
| `Lit` | 0x05 | `type_tag, value: Vec<u8>` | Literal constant |
| `Ref` | 0x06 | `fragment_id` | Cross-fragment reference |
| `Neural` | 0x07 | `guard_spec, weight_blob` | Neural computation |
| `Fold` | 0x08 | `recursion_descriptor` | Catamorphism (iteration) |
| `Unfold` | 0x09 | `recursion_descriptor` | Anamorphism (corecursion) |
| `Effect` | 0x0A | `effect_tag: u8` | I/O effect |
| `Tuple` | 0x0B | (none) | Product construction |
| `Inject` | 0x0C | `tag_index` | Sum type injection |
| `Project` | 0x0D | `field_index` | Product field projection |
| `TypeAbst` | 0x0E | `bound_var_id` | Type abstraction (ForAll intro) |
| `TypeApp` | 0x0F | `type_arg` | Type application (ForAll elim) |
| `LetRec` | 0x10 | `binder, decrease` | Guarded recursive binding |
| `Guard` | 0x11 | `predicate, body, fallback` | Runtime conditional |
| `Rewrite` | 0x12 | `rule_id, body` | Verified rewrite application |
| `Extern` | 0x13 | `name, type_sig` | External function reference |

### Edge Labels

Edges connect nodes with typed ports:

| Label | Value | Purpose |
|-------|-------|---------|
| `Argument` | 0 | Data flow (function arguments, operands) |
| `Scrutinee` | 1 | Value being matched/tested |
| `Binding` | 2 | Variable binding edge |
| `Continuation` | 3 | Control flow continuation |
| `Decrease` | 4 | Decreasing argument (for termination) |

### Resolution Levels

Each graph has a resolution level indicating its abstraction tier:

| Level | Description |
|-------|-------------|
| `Intent` | High-level intent (depth 0) |
| `Architecture` | Architectural shape (depth 1) |
| `Implementation` | Full implementation detail (depth 2) |

---

## The Execution Pipeline

### Source to Execution

```
.iris source file
      |
      v
  Lexer (syntax/lexer.rs)
      |  Tokenize: source -> Vec<Token>
      v
  Parser (syntax/parser.rs)
      |  Parse: tokens -> AST (Module, Expr, etc.)
      v
  Lowerer (syntax/lower.rs)
      |  Lower: AST -> Vec<(name, Fragment, SourceMap)>
      v
  SemanticGraph (graph.rs)
      |  Content-addressed, typed, with cost annotations
      v
  Execution:
      |
      +-- Bootstrap tree-walker (lib.rs)
      |     Handles all 20 node kinds, graph manipulation
      |     Dispatches effects through EffectHandler
      |     CapabilityGuardHandler enforces permissions
      |
      +-- Flat evaluator (lib.rs)
      |     Flattens fold bodies → FlatProgram (linear ops)
      |     ~578× faster than tree-walker for Float64
      |
      +-- Native x86-64 codegen (lib.rs)
      |     FlatProgram → AVX machine code (W^X mmap)
      |     Loop-carried regs, copy prop, Lit-aware eviction
      |     ~11,800× faster than tree-walker
      |
      +-- CLCU (iris-clcu)
            AVX-512 vectorized containers
```

### The Lowering Process

The lowerer (`src/iris-bootstrap/src/syntax/lower.rs`) translates each top-level `let` declaration into a `Fragment`:

1. Create a `LowerCtx` with scope stack and type environment
2. Bind function parameters as `InputRef(0)`, `InputRef(1)`, etc.
3. Recursively lower the body expression to `SemanticGraph` nodes
4. Each expression becomes one or more nodes with edges
5. Content-address every node via BLAKE3 (with salt for disambiguation)
6. Package into a `Fragment` with boundary, type env, contracts, and metadata

### Evaluation

The `IrisExecutionService` evaluates programs through the bootstrap tree-walker with capability-guarded effects:

| Component | Role |
|-----------|------|
| Bootstrap evaluator | Handles all 20 node kinds, `graph_eval` for self-evaluation |
| Flat evaluator | Flattens eligible fold bodies to `FlatProgram`; f64 specialization for numeric loops |
| Native x86-64 codegen | Compiles FlatPrograms to AVX machine code with register allocation; 0.22 µs/step on n-body |
| `RuntimeEffectHandler` | Implements 44 effect tags with real I/O |
| `CapabilityGuardHandler` | Enforces tag/path/host permissions before dispatch |
| CLCU (optional) | AVX-512 hardware acceleration for vectorizable programs |

All programs run in a sandbox by default. Effects beyond Print/Log/Timestamp/Random require explicit capabilities.

---

## The Effect System

The bootstrap evaluator dispatches all 44 effect tags through an `EffectHandler` trait:

```
IRIS program performs Effect node (e.g., FileWrite)
      |
      v
Bootstrap evaluator (eval_effect)
      |  Checks for EffectHandler
      v
CapabilityGuardHandler
      |  1. Check if tag is allowed
      |  2. Check path/host restrictions
      |  3. Check env var permissions
      v
RuntimeEffectHandler
      |  Performs real I/O (file ops, TCP, env, time, random, atomics)
      v
Result flows back into computation
```

Effects without a handler fall back to minimal built-in handling (Print to stderr, Timestamp returns real time). All other tags require an explicit handler.

---

## The Self-Improvement Loop

IRIS can improve its own components at runtime:

```
Profile component performance
      |
      v
Identify bottleneck (latency, correctness)
      |
      v
Evolve replacement via NSGA-II
      |
      v
Gate: run against original with test suite
      |  Reject if > 2x slowdown
      v
Deploy: hot-swap the improved component
      |
      v
Audit: record the change with before/after metrics
      |
      v
Inspect: detect regressions, auto-rollback if needed
```

This is implemented by the `SelfImprovingDaemon` (`src/iris-evolve/src/self_improving_daemon.rs`):

```
ThreadedDaemon (criteria-driven improvement)
  +-- ImprovementPool (bounded thread pool for background evolution)
  +-- ComponentMetrics (lock-free per-component metrics)
  +-- ImproveTrigger (criteria that trigger improvement)
  +-- StagnationDetector (per-component convergence tracking)
  +-- ConvergenceDetector (system-wide local maximum detection)
  +-- AutoImprover (profile -> evolve -> gate -> deploy)
  +-- SelfInspector (detect regressions -> auto-correct -> audit)
  +-- IrisRuntime (hot-swap deployed components)
  +-- Persistence (save/restore state to disk as JSON)
```

---

## Bootstrap Chain

IRIS is designed to progressively replace its Rust implementation with IRIS programs:

```
1. Bootstrap evaluator (iris-bootstrap/src/lib.rs)
   |  ~3,733 LOC tree-walker with effect dispatch
   |  Includes syntax pipeline (parser, lowerer) and proof kernel
   v
2. IRIS interpreter (src/iris-programs/interpreter/full_interpreter.iris)
   |  The interpreter rewritten in IRIS surface syntax
   |  Can evaluate other IRIS programs (all 20 node kinds)
   v
3. IRIS programs
      All 256 programs in src/iris-programs/ run on the IRIS-written interpreter
```

The bootstrap evaluator supports all 20 node kinds, including Effect nodes with full handler dispatch. It also embeds the syntax pipeline (lexer, parser, lowerer) and the LCF proof kernel, merged from the original standalone crates.

---

## Content-Addressed Identity

Every node, type, and fragment is content-addressed via BLAKE3.

### Node Identity

```
NodeId = BLAKE3(kind, type_sig, arity, resolution_depth, salt, payload)[0..8]
```

- 64-bit truncated BLAKE3 hash
- Salt provides disambiguation for structurally identical nodes at different positions
- Computed by `compute_node_id()` in `src/iris-types/src/hash.rs`

### Fragment Identity

```
FragmentId = BLAKE3(graph, boundary, type_env, imports)
```

- 256-bit full BLAKE3 hash
- Uniquely identifies a program across the entire system
- Proof receipts and contracts do NOT affect FragmentId (they are metadata)

### Semantic Hash

```
SemanticHash = BLAKE3(behavioral fingerprint)
```

- 256-bit behavioral fingerprint stored on each SemanticGraph
- Used for deduplication and novelty search

### Type Identity

```
TypeId = BLAKE3(TypeDef)[0..8]
```

- 64-bit truncated BLAKE3 hash of the type definition
- Content-addressed deduplication in `TypeEnv`

---

## The IrisDaemon

The daemon runs programs continuously in configurable cognitive cycles:

```
cycle 0: execute all programs -> collect metrics -> sleep remainder
cycle 1: execute all programs -> collect metrics -> sleep remainder
...
cycle N: trigger improvement check
         -> profile -> evolve replacement -> gate -> deploy
cycle N+1: execute with improved components
...
```

Configuration:
- **Cycle time**: Default 800ms (matching Daimon's cognitive cycles)
- **Max cycles**: Optional limit
- **Improvement interval**: Check for improvement opportunities every N cycles
- **Exec mode**: `FixedInterval(Duration)` or `Continuous`

Cycle metrics tracked per cycle:
- Cycle number
- Wall-clock time
- Programs executed
- Outputs produced
- Overrun flag (cycle exceeded time budget)

---

## The Algorithm Foundry

The foundry (now implemented as IRIS programs in `src/iris-programs/foundry/`) provides a specification-in, verified-fragments-out pipeline:

```
ProblemSpec (test cases + constraints)
      |
      v
  Latency tier selection:
      |
      +-- Instant (<100ms): library lookup by type signature
      +-- Fast (<10s): small-population evolution from library seeds
      +-- Standard (<10min): full evolution
      +-- Deep (hours): multi-deme, novelty-driven search
      |
      v
  EvolutionResult
      |
      v
  Verified Fragment with ProofReceipt
```

---

## The Codec

The graph-embedding codec (types in `iris-types::codec`, behavior in IRIS programs) provides bidirectional mapping between SemanticGraph and embedding vectors:

- **FeatureCodec** (Gen1): Deterministic feature extraction (node kind histogram, opcode distribution, depth, arity patterns)
- **GIN-VAE** (Gen2/3): Graph Isomorphism Network variational autoencoder for learned embeddings
- **HNSW index**: Hierarchical Navigable Small World graph for nearest-neighbor search in embedding space
- **Crossover**: Interpolate between parent embeddings, decode back to graph, apply structural repair
