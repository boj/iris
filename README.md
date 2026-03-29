# IRIS

**Intelligent Runtime for Iterative Synthesis** -- a self-improving programming language and runtime where programs are typed DAGs that evolve, verify, and optimize themselves.

## What is IRIS?

IRIS is both a language and a runtime -- one system, two faces:

- **The language** is an ML-like surface syntax (`.iris` files) that humans and LLMs write. It compiles to SemanticGraph, the canonical program representation.
- **The runtime** is a four-layer execution substrate that evolves, verifies, and runs SemanticGraphs. Evolution can also produce programs directly -- no surface syntax required.

The surface syntax is a projection of the SemanticGraph, not the source of truth. Programs are content-addressed DAGs with 20 node kinds, purely functional, carrying their own type environments. The syntax is how you write them; the graph is what they are.

```
L0  Evolution        -- population search (NSGA-II + lexicase + novelty)
L1  Semantics        -- SemanticGraph (20 node kinds, BLAKE3 content-addressed)
L2  Verification     -- LCF proof kernel (20 inference rules, zero unsafe Rust)
L3  Hardware         -- tree-walker + CLCU (AVX-512)
```

## Key Properties

- **Self-improving**: Run any program with `--improve` and the runtime automatically traces function calls, builds test cases from observed I/O, evolves faster implementations via NSGA-II genetic search, gates them for correctness and performance, and hot-swaps improvements into the running program. No manual specs needed; the program improves itself from its own behavior.
- **Self-hosting**: 228 `.iris` programs (29K+ LOC) cover all system components. The bootstrap evaluator loads the meta-circular interpreter, which handles all 20 node kinds. The infrastructure uses its own type system features (ADTs, pattern matching, imports) internally. The irreducible Rust substrate is ~51K LOC across 5 crates (proof kernel + bootstrap evaluator + types + evolution engine + execution shim).
- **Verified**: The LCF proof kernel (CaCIC -- Cost-aware Calculus of Inductive Constructions) proves type safety, cost bounds, and functional properties via refinement types and contracts. The kernel's 20 inference rules are formalized in Lean 4 with 47 theorems (zero `sorry`), and a Lean FFI bridge lets the proven Lean code execute as the production kernel.
- **Recursive**: Programs improve programs. The evolution engine runs inside the language, and evolved programs can themselves evolve sub-programs. A BLAKE3 Merkle chain provides tamper-evident audit trails for all self-modifications.

## Architecture

The runtime is implemented in Rust with 5 crates. The parser, proof kernel, and syntax pipeline are embedded inside the bootstrap crate (merged from the original 14-crate workspace):

```
Permanent Rust substrate (5 crates)
  iris-types       (5,370 LOC)  -- SemanticGraph, Value, types, cost, wire format
  iris-bootstrap  (12,887 LOC)  -- bootstrap evaluator + parser + proof kernel
    +-- syntax/                 -- lexer, parser, lowerer (merged from iris-syntax)
    +-- syntax/kernel/          -- LCF proof kernel (merged from iris-kernel)
  iris-exec        (2,326 LOC)  -- execution shim, capabilities, effect runtime
  iris-evolve     (30,340 LOC)  -- evolution engine, improvement daemon
  iris-clcu-sys      (300 LOC)  -- FFI bindings to C CLCU interpreter

IRIS Programs (228 files, 29K+ LOC)
  interpreter/     -- meta-circular interpreter (all 20 node kinds)
  evolution/       -- mutation, selection, crossover, seeds, NSGA-II
  compiler/        -- 10-pass pipeline (monomorphize -> container pack)
  codec/           -- GIN-VAE encoder/decoder, HNSW index
  analyzer/        -- 10 pattern detectors for problem classification
  exec/            -- evaluator, cache, effects, daemon, message bus (uses ADTs)
  checker/         -- tier classification, obligation counting
  foundry/         -- algorithm foundry API, fragment library
  syntax/          -- tokenizer, parser, lowerer (self-hosting)
  meta/            -- auto-improve, performance gate, instrumentation
  deploy/          -- bytecode serialization, standalone binaries
  store/           -- persistence, registry
  stdlib/          -- type modules (Option, Result, Either, Ordering)
  algorithms/      -- fibonacci, gcd, factorial, quicksort
  io/              -- TCP, files, system primitives
  threading/       -- spawn/join, atomics, rwlocks
  vm/              -- bytecode compiler, VM step/run
  population/      -- population management, elitism
```

## Quick Start

```bash
# Build
cargo build --release

# Run an IRIS program
cargo run --release --bin iris -- run examples/algorithms/factorial.iris 10
# Output: 3628800

# Type-check / verify a program
cargo run --release --bin iris -- check examples/verified/bounded_add.iris
# Output: [OK] bounded_add: 5/5 obligations satisfied (score: 1.00)

# Evolve a solution from a specification
cargo run --release --bin iris -- solve spec.iris

# Run with observation-driven improvement
cargo run --release --bin iris -- run --improve examples/algorithms/factorial.iris 10
```

The `run` command loads the pre-compiled tokenizer, parser, and lowerer from
`bootstrap/*.json`, then evaluates the resulting SemanticGraph with the bootstrap
tree-walker. No feature flags are needed.

### Tests

```bash
# Run core tests (397+ pass across 9 suites)
cargo test --features rust-scaffolding --test test_typecheck_all \
  --test test_capability_wiring_iris --test test_exec_iris_programs \
  --test test_syntax --test test_syntax_scaffolding_gap \
  --test test_verification_complete --test test_checker_iris \
  --test test_improve --test test_improve_e2e

# Run integration tests
cargo test --features rust-scaffolding --test test_effects --test test_security \
  --test test_bootstrap_effects --test test_capability_wiring_iris

# Type-check all 228 .iris files
cargo test --features rust-scaffolding --test test_typecheck_all
```

## Observation-Driven Improvement

```bash
iris run --improve myprogram.iris 42
```

The `--improve` flag enables automatic runtime optimization:

1. **Traces** function calls at a configurable sampling rate (default 1%), recording `(inputs, output, latency)` per function
2. **Builds** test cases from observed I/O, no manual specs needed
3. **Evolves** faster implementations using NSGA-II genetic search (budget: 5s)
4. **Gates** candidates through equivalence (identical outputs on all traces) and performance (≤2× slowdown)
5. **Hot-swaps** improvements atomically into the running program

```
[improve] daemon started: min_traces=50, threshold=2.0x, budget=5s
42
[improve] attempting compute (73 test cases, avg 124.3µs)
[improve] ✓ deployed compute (124.3µs → 68.1µs, 45% faster)
```

The full pipeline (trace, synthesize, evolve, gate, swap) completes in under 100ms for simple functions. 18 tests (13 unit + 5 end-to-end) verify every stage. See [Evolution & Improvement](site/content/learn/daemon.md) for options and benchmarks.

Programs can also evolve sub-programs at runtime with `evolve_subprogram`, or generate programs from specs with `iris solve`.

## Performance

The bootstrap evaluator is a tree-walking interpreter. 10 programs from the [Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/) are implemented in `benchmark/`.

### Micro-Benchmarks (Criterion, release mode)

| Operation | Time |
|-----------|------|
| Literal evaluation | 66 ns |
| `add(3,5)` | 207 ns |
| Fold iteration (integer) | 0.55 µs/step |
| Fold iteration (float64) | 0.59 µs/step |
| Cross-fragment call | 0.83 µs/step |
| JIT `add(3,5)` | 60 ns (3.5× faster) |

### Benchmarks Game (tree-walker, release mode)

| Program | Input | Time | Per-unit cost |
|---------|-------|------|--------------|
| binary-trees | depth=14 | ~13 ms | ~0.4 µs/node |
| n-body | N=100 | 262.8 ms | 2.6 ms/step |
| fasta | N=500 | 3.5 ms | 7.0 µs/char |
| thread-ring | token=50K | 52.0 ms | 1.04 µs/token |
| k-nucleotide | N=200 | 0.7 ms | 3.5 µs/base |
| pidigits | N=15 | 0.2 ms | n/a |

### Comparison to Other Languages (CLBG standard inputs)

| Benchmark | Input | IRIS (est.) | CPython 3 | Haskell | OCaml |
|-----------|-------|-------------|-----------|---------|-------|
| binary-trees | depth=21 | **~1.7 s** | ~100 s | ~2.2 s | ~3.5 s |
| fasta | N=25M | ~175 s | ~40 s | ~1.1 s | ~1.8 s |
| thread-ring | N=50M | ~52 s | ~10 s | ~1.0 s | ~1.5 s |

Binary-trees is the strongest result: recursive allocation is the tree-walker's sweet spot, **~59× faster than CPython**. Loop-heavy benchmarks (fasta, thread-ring) are 4–5× slower than CPython because fold dispatch (0.55 µs/step) is more expensive than bytecode dispatch (~0.05 µs/step). See the full [Benchmarks](site/content/learn/benchmarks.md) page for analysis.

## The Language

Programs are `.iris` files with an ML-like surface syntax. The syntax compiles to SemanticGraph -- it's a human-friendly way to construct typed DAGs, not a traditional source language. Evolved programs that have no textual origin can be decompiled back to `.iris` (best-effort, may be lossy for complex DAG structures).

```iris
-- Factorial with recursion and cost annotation
let rec factorial n : Int -> Int [cost: Linear(n)] =
  if n <= 1 then 1
  else n * factorial (n - 1)

-- Sum a list using fold
let sum xs : List Int -> Int [cost: Linear(xs)] =
  fold 0 (+) xs

-- Self-modifying: replace an operator in a program
let mutate program new_op : Program -> Int -> Program =
  let root = graph_get_root program in
  graph_set_prim_op program root new_op
```

### Algebraic Data Types

Define custom types with named constructors and pattern match on them:

```iris
type Result = Ok(Int) | Err(Int)

let safe_divide a b : Int -> Int -> Int =
    if b == 0 then Err 0
    else Ok (a / b)

let unwrap_or r default_val : Int -> Int -> Int =
    match r with
      | Ok(v) -> v
      | Err(e) -> default_val
```

Sum types support: payloads (`Some(Int)`), bare enums (`Red | Green | Blue`), exhaustiveness checking, and wildcard patterns. See [examples/algebraic-types/](examples/algebraic-types/) for Option, Result, linked lists, and state machines.

### Imports and Standard Library

Path-based and content-addressed imports are supported. Higher-order functions work across module boundaries -- closures carry their source graph for correct cross-import evaluation:

```iris
import "stdlib/option.iris" as Opt
import "stdlib/result.iris" as Res

-- Map over an Option with a local lambda
let doubled = Opt.map (Some(21)) (\x -> x * 2)      -- Some(42)

-- Chain operations monadically
let chained = Opt.and_then (Some(10)) (\x ->
  if x > 5 then Some(x * 3) else None)               -- Some(30)

-- Filter with a predicate
let filtered = Opt.filter (Some(7)) (\x -> x > 10)   -- None

-- Error handling with Result
let safe = Res.and_then (Ok(10)) (\x ->
  if x > 0 then Ok(x * 2) else Err(0 - 1))           -- Ok(20)
```

The standard library provides 4 type modules (Option, Result, Either, Ordering) with higher-order functions (map, and\_then, filter, unwrap\_or\_else, zip\_with), plus modules for math, collections, strings, file I/O, HTTP, threading, and lazy lists.

Inline ADTs can be defined without imports for module-internal use:

```iris
type DispatchResult = DispatchOk(Int) | EffectUnsupported | EffectDenied | DispatchErr(Int)

let handle result : DispatchResult -> Int =
  match result with
  | DispatchOk(v)  -> v
  | EffectDenied   -> 0 - 1
  | _              -> 0
```

Pattern matching works inside lambda bodies (fold callbacks, map functions), enabling type-safe ADT processing in higher-order contexts.

### Struct Types

Struct types (records) give named fields to tuples. They are sugar over product types; field access resolves to positional projection at compile time:

```iris
type Point = { x: Int, y: Int }

let origin : Point = { x = 0, y = 0 }    -- compiles to (0, 0)
let px = origin.x                         -- resolves to origin.0

let add_points : Point -> Point -> Point = \a -> \b ->
  { x = a.x + b.x, y = a.y + b.y }
```

### Available Primitives

| Category | Primitives |
|----------|-----------|
| Arithmetic | `+`, `-`, `*`, `/`, `%`, `neg`, `abs`, `min`, `max`, `pow` |
| Comparison | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| Logic | `and`, `or`, `not` |
| Graph | `self_graph`, `graph_nodes`, `graph_get_kind`, `graph_get_prim_op`, `graph_set_prim_op`, `graph_add_node_rt`, `graph_connect`, `graph_disconnect`, `graph_replace_subtree`, `graph_eval`, `graph_get_root`, `graph_add_guard_rt`, `graph_add_ref_rt`, `graph_set_cost` |
| I/O | `tcp_connect`, `tcp_read`, `tcp_write`, `tcp_close`, `tcp_listen`, `tcp_accept`, `file_open`, `file_read_bytes`, `file_write_bytes`, `file_close`, `file_stat`, `dir_list`, `env_get`, `clock_ns`, `random_bytes`, `sleep_ms` |
| Threading | `thread_spawn`, `thread_join`, `atomic_read`, `atomic_write`, `atomic_swap`, `atomic_add`, `rwlock_read`, `rwlock_write`, `rwlock_release` |
| Collections | `fold`, `map`, `filter`, `zip`, `concat`, `reverse`, `len` |
| Effects | `perform_effect` (generic effect dispatch via opcode 0xA1) |

## The Runtime

Once compiled to SemanticGraph, programs are evaluated by the bootstrap tree-walker. The evaluator handles all 20 node kinds, dispatches effects through a capability-guarded handler, and supports self-evaluation via `graph_eval`.

### Execution Tiers

| Tier | Backend | Description |
|------|---------|-------------|
| A | Tree-walking interpreter | Fast evaluation, no overhead |
| CLCU | AVX-512 containers | Hardware-accelerated via C CLCU library |

### Effect System

The bootstrap evaluator dispatches all 44 effect tags through an `EffectHandler` trait. The `RuntimeEffectHandler` implements real I/O (files, TCP, env, time, random, atomic state), and the `CapabilityGuardHandler` wraps it with capability enforcement.

Every execution path (`IrisExecutionService`, `interpret_with_capabilities`, and `interpret_sandboxed`) enforces capabilities. By default, programs run in a sandbox that allows only pure computation effects (Print, Log, Timestamp, Random).

## Verification

Three layers of verification are provided:

### LCF Proof Kernel
The kernel is ~7,345 LOC of Rust (in `src/iris-bootstrap/src/syntax/kernel/`) with zero `unsafe` blocks outside the Lean FFI bridge. It has 20 inference rules that produce opaque `Theorem` values; external code cannot forge proofs. The kernel proves type safety, cost bounds, and functional properties via refinement types (`{x : Int | x > 0}`) and contracts (`requires`/`ensures`).

### Lean 4 Formalization
The kernel's 20 rules are formalized in Lean 4 (`lean/IrisKernel/`) with 47 theorems and zero `sorry` markers. Key results include weakening (structural induction over all 20 constructors), cost lattice properties, and consistency. 85 cross-validation tests verify the Rust implementation matches the Lean specification.

### Lean FFI Bridge
A C shim + Rust bridge lets the *compiled Lean code* execute as the production kernel. Enable with `--features lean-ffi` (requires Lean 4 toolchain). Without it, the Rust implementation runs as a fallback, and both produce identical results (verified by 85 cross-validation tests).

### BLAKE3 Audit Chain
Every self-modification (component deployment, rollback) is recorded in a tamper-evident BLAKE3 Merkle chain. Each entry's hash covers all fields; each entry's `prev_hash` links to the previous. Modifying any entry breaks the chain.

## Security Model

### Capability-Based Sandboxing

All programs run through the `CapabilityGuardHandler`, which enforces fine-grained effect permissions before any I/O reaches the OS. The default sandbox blocks:
- All file operations (read, write, open, stat, dirlist)
- All network operations (TCP connect/listen/accept)
- Thread spawning
- FFI calls
- MmapExec (JIT code generation)
- Environment variable access

Only explicitly allowed effects (Print, Log, Timestamp, Random, ClockNs, RandomBytes, SleepMs) pass through.

### Capability Declarations

Programs declare the effects they need using `allow`/`deny` blocks:

```iris
allow [FileRead, TcpConnect "api.*"]
deny [MmapExec, ThreadSpawn, FfiCall]
```

Capabilities are enforced at runtime before each effect is dispatched. A program that attempts an effect not in its allow-list gets a `PermissionDenied` error.

### Path and Host Restrictions

File operations validate paths against an allow-list with glob matching (`/tmp/**`, `/home/user/data/*`). Paths are canonicalized to prevent symlink bypass attacks. Null bytes in paths are rejected. TCP operations validate host names against an allow-list with wildcard subdomain matching (`*.example.com`).

### Security Audit Status

All 6 findings from the 2026-03-24 security audit have been resolved. See [docs/security-todo.md](docs/security-todo.md) for details.

### Kernel Safety

The LCF proof kernel (at `src/iris-bootstrap/src/syntax/kernel/`) enforces zero `unsafe` blocks across all modules. The single exception is `lean_bridge`, which is explicitly `#[allow(unsafe_code)]` because it calls into Lean 4's C-compiled output via FFI.

## Project Structure

```
src/
  iris-types/          Core data structures (SemanticGraph, Value, types, cost, wire)
  iris-bootstrap/      Bootstrap evaluator + syntax pipeline + proof kernel
    src/syntax/        Lexer, parser, AST, lowering (merged from iris-syntax)
    src/syntax/kernel/ LCF proof kernel (merged from iris-kernel)
  iris-exec/           Execution shim, capabilities, effect runtime, registry
  iris-evolve/         Evolution engine, improvement daemon
  iris-clcu-sys/       FFI bindings to C CLCU interpreter
  bin/iris.rs          CLI binary (run, check, solve, daemon, repl)
src/iris-programs/    Core .iris programs (19 categories)
examples/             Demo .iris programs + Rust examples
  stdlib/             Standard library type modules (Option, Result, Either, Ordering)
benchmark/             10 Computer Language Benchmarks Game implementations
bootstrap/             Pre-compiled IRIS interpreter (JSON)
lean/                  Lean 4 kernel formalization (47 theorems, 0 sorry)
iris-clcu/             C CLCU interpreter (AVX-512 + scalar)
tests/                 45+ test files (397+ tests)
docs/                  User guide, language reference, architecture, API, contributing
site/                  Hugo-based documentation website (iris-lang.org)
```

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE) for the full text.

Commercial licensing available. Contact Brian Jones (bojo@bojo.wtf) for details.
