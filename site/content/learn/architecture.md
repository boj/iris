---
title: "Architecture"
description: "The four-layer IRIS architecture: evolution, semantics, verification, and hardware."
weight: 80
---

IRIS is a four-layer stack. Each layer operates on a single canonical representation: the SemanticGraph.

```
L0  Evolution        -- population search (NSGA-II + lexicase + novelty)
L1  Semantics        -- SemanticGraph (20 node kinds, BLAKE3 content-addressed)
L2  Verification     -- LCF proof kernel (20 inference rules, zero unsafe Rust)
L3  Hardware         -- bootstrap evaluator + JIT (W^X x86-64) + CLCU (AVX-512)
```

## SemanticGraph (L1) {#semanticgraph}

The SemanticGraph is the canonical program representation. Every program -- whether written by hand, compiled from `.iris` files, or bred by evolution -- is a typed DAG with 20 node kinds.

### Node Kinds {#node-kinds}

| Tag | Kind | Purpose |
|-----|------|---------|
| `0x00` | Prim | Primitive operation (~50 opcodes) |
| `0x01` | Apply | Function application |
| `0x02` | Lambda | Abstraction (closure) |
| `0x03` | Let | Local binding |
| `0x04` | Match | Pattern matching |
| `0x05` | Lit | Literal value |
| `0x06` | Ref | Cross-fragment call |
| `0x07` | Neural | Neural computation layer |
| `0x08` | Fold | Structural recursion (catamorphism) |
| `0x09` | Unfold | Corecursion (anamorphism) |
| `0x0A` | Effect | Side effect descriptor |
| `0x0B` | Tuple | Product constructor |
| `0x0C` | Inject | Sum constructor |
| `0x0D` | Project | Product elimination |
| `0x0E` | TypeAbst | Type abstraction |
| `0x0F` | TypeApp | Type application |
| `0x10` | LetRec | Recursive binding (guarded) |
| `0x11` | Guard | Runtime guard (conditional) |
| `0x12` | Rewrite | Verified rewrite |
| `0x13` | Extern | External function |

Each node carries a `NodeId` (64-bit BLAKE3 truncated), a type reference, a cost term, an arity, and operation-specific payload. Nodes are content-addressed -- identical subgraphs share the same ID.

### How ADTs Compile {#adt-compilation}

When you write `type Option = Some(Int) | None`, the compiler:

1. Creates a `TypeDef::Sum([(Tag(0), Int), (Tag(1), Unit)])` in the type environment
2. Binds `Some` as a constructor that emits `Inject { tag: 0 }` nodes
3. Binds `None` as a bare constructor emitting `Inject { tag: 1 }` with Unit payload
4. Compiles `match` arms with constructor patterns to `Match` nodes with tag-based dispatch
5. Arms that bind inner values (e.g., `Some(v)`) are wrapped in `Lambda` nodes so the evaluator can pass the payload

## Evolution (L0) {#evolution}

Programs are evolved through multi-objective genetic search.

### Mutation Operators {#mutation-operators}

16 mutation operators transform program graphs:

| ID | Operator | Description |
|----|----------|-------------|
| 0 | `insert_node` | Add a new node to the graph |
| 1 | `delete_node` | Remove a node |
| 2 | `rewire_edge` | Rewire edges between nodes |
| 3 | `replace_kind` | Change a node's kind tag |
| 4 | `replace_prim` | Change a node's opcode |
| 5 | `mutate_literal` | Modify a literal value |
| 6 | `duplicate_subgraph` | Clone a subgraph |
| 7 | `wrap_in_guard` | Add a conditional guard |
| 8 | `annotate_cost` | Add cost annotation |
| 9 | `wrap_in_map` | Wrap in map operation |
| 10 | `wrap_in_filter` | Wrap in filter operation |
| 11 | `compose_stages` | Compose pipeline stages |
| 12 | `insert_zip` | Add zip operation |
| 13 | `swap_fold_op` | Change fold accumulator |
| 14 | `add_guard_condition` | Add guard predicate |
| 15 | `extract_to_ref` | Extract to cross-fragment ref |

### Selection Strategies {#selection}

- **Tournament selection** -- binary tournament on fitness
- **Lexicase selection** -- sequential filtering on individual test cases
- **NSGA-II** -- multi-objective Pareto optimization
- **Novelty search** -- reward behavioral novelty over raw fitness

### Population Management {#population}

- **Pareto ranking** -- assign non-dominated rank
- **Crowding distance** -- maintain population diversity
- **Elitism** -- preserve top individuals across generations
- **Death culling** -- remove unfit individuals

### Crossover {#crossover}

The Graph-Embedding Codec (GIN-VAE) enables crossover by:

1. Encoding parent graphs into embedding vectors
2. Interpolating in embedding space
3. Decoding back to valid graphs
4. Running a 10-phase structural repair pipeline

The GIN-VAE is trained on a 1,045-program corpus (all `.iris` programs, evolution Pareto fronts, self-write mutation programs, bootstrap interpreter, and seed generators). The LearnedCodec achieves 0.012 reconstruction loss.

### Improvement Tracking (PT3) {#improvement-tracking}

The evolution system tracks improvement dynamics for each component:

- **Improvement rate** -- linear regression over sliding-window latency measurements
- **Acceleration** -- second derivative of latency; negative acceleration means improvements are compounding
- **Causal operator attribution** -- per-operator statistics tracking success rate, times used, and average fitness delta across 16 mutation operators
- **Adaptive weighting** -- mutation operator selection weights adjusted proportional to observed success rate and improvement magnitude

When the tracker detects compounding improvements (acceleration < 0), the daemon increases investment in that component's evolution. When deceleration is detected, it recognizes a plateau and redirects resources.

## Verification (L2) {#verification}

The proof kernel is the trusted core. It implements CaCIC (Cost-aware Calculus of Inductive Constructions) with 20 inference rules.

See the [Verification](/learn/verification/) page for a deep dive into the proof kernel, refinement types, the checker, and the Lean formalization.

### Properties {#verification-properties}

- **LCF-style** -- proofs can only be constructed through the kernel's API; no external code can forge a `Theorem` value
- **Refinement types** -- predicates on types: `{x: Int | x > 0}`
- **Cost annotations** -- asymptotic bounds as evolution fitness objectives
- **Lean 4 formalization** -- the 20 rules are mirrored in Lean 4 with consistency proofs

## Execution (L3) {#execution}

Programs run through the bootstrap evaluator with optional JIT compilation and effect dispatch:

### Bootstrap Evaluator {#evaluator}

The bootstrap evaluator handles all 20 node kinds directly. It dispatches primitive operations (50+ opcodes), manages closures, and evaluates fold/unfold recursion. The meta-circular interpreter (`full_interpreter.iris`) can also dispatch on node kind tags via graph introspection.

**Effect Dispatch:** Opcode `0xA1` (`perform_effect`) dispatches all 44 effect tags through an `EffectHandler`. The `RuntimeEffectHandler` implements real I/O (files, TCP, environment, time, random). The `CapabilityGuardHandler` wraps any handler and enforces capability restrictions before each effect call.

**Performance:** ~200 ns per arithmetic operation (interpreted).

### JIT Compiler (x86-64) {#jit}

The JIT generates x86-64 machine code, compiles it via the `mmap_exec` effect (W^X memory mapping), and invokes compiled functions via `call_native` (System V AMD64 ABI, up to 6 i64 arguments).

**W^X enforcement:** Pages are allocated read-write, code bytes are copied in, then `mprotect` flips them to read-execute. Pages are never simultaneously writable and executable. Each region is limited to 1 MiB. All regions are `munmap`'d on cleanup.

**Performance:** JIT `add` runs in ~64 ns (3.1× faster than interpreted). Feature-gated behind `--features jit` and capability-gated (sandboxes block `MmapExec`/`CallNative`).

### CLCU (AVX-512) {#clcu}

Cache-Line Compute Units: 64-byte containers designed for 16-lane SIMD execution. Includes adaptive prefetch and branch predictor hints.

## Self-Improvement Daemon {#daemon}

The daemon continuously profiles running components, evolves faster replacements, and hot-swaps them in. See [Evolution & Improvement](/learn/daemon/) for the full pipeline.

Key properties:
- Performance gate: replacements must be 100% correct and within 2x slowdown
- BLAKE3 Merkle audit chain for tamper-evident modification history
- Stagnation detection: stops investing in components that plateau
- State persists to disk; improvements survive restarts

## Crate Map {#crates}

The Rust runtime is organized into 5 crates:

| Crate | Purpose |
|-------|---------|
| `iris-types` | SemanticGraph, types, values, wire format |
| `iris-bootstrap` | Bootstrap evaluator + syntax pipeline + LCF proof kernel |
| `iris-exec` | Execution shim, capabilities, effect runtime |
| `iris-evolve` | Evolution engine, improvement pipeline |
| `iris-clcu-sys` | FFI bindings to C CLCU (AVX-512) |
