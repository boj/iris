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
L2  Verification     -- Lean 4 proof kernel (20 inference rules, IPC subprocess)
L3  Hardware         -- tree-walker + CLCU (AVX-512)
```

## Key Properties

- **Self-improving**: Run any program with `--improve` and the runtime automatically traces function calls, builds test cases from observed I/O, evolves faster implementations via NSGA-II genetic search, gates them for correctness and performance, and hot-swaps improvements into the running program. No manual specs needed; the program improves itself from its own behavior.
- **Self-hosting**: 228 `.iris` programs (29K+ LOC) cover all system components. The bootstrap evaluator loads the meta-circular interpreter, which handles all 20 node kinds. The infrastructure uses its own type system features (ADTs, pattern matching, imports) internally. The irreducible Rust substrate is iris-types + iris-bootstrap (types, evaluator, syntax); everything else is `.iris` programs.
- **Verified**: The proof kernel is written in Lean 4 and runs as a separate IPC process (`iris-kernel-server`). The 20 inference rules (CaCIC -- Cost-aware Calculus of Inductive Constructions) prove type safety, cost bounds, and functional properties. Lean's type system guarantees the kernel is correct -- the running code IS the formal proof. Rust handles proof hashing (BLAKE3 audit trail) and the `Theorem` wrapper, but all judgment logic executes in Lean.
- **Recursive**: Programs improve programs. The evolution engine runs inside the language, and evolved programs can themselves evolve sub-programs. A BLAKE3 Merkle chain provides tamper-evident audit trails for all self-modifications.

## Architecture

The runtime is a 2-crate Rust workspace plus the Lean 4 kernel:

```
Permanent substrate
  iris-types                    -- SemanticGraph, Value, types, cost, wire format
  iris-bootstrap                -- bootstrap evaluator + parser + kernel bridge
    +-- syntax/                 -- lexer, parser, lowerer
    +-- syntax/kernel/          -- Lean IPC bridge, Theorem type, proof hashing
  lean/IrisKernel/              -- Lean 4 proof kernel (20 rules, compiled to native)
  lean/IrisKernelServer.lean    -- IPC server (stdin/stdout, auto-spawned by Rust)

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

### Prerequisites

- **Rust** 1.75+ (`rustup` recommended)
- **Lean 4** (`elan` recommended — `lake build` is invoked automatically)

On NixOS, Lean is found automatically via `nix-shell`. On other systems, install
from [leanprover.github.io](https://leanprover.github.io/lean4/doc/setup.html).

### Build and Run

```bash
# Build (auto-builds the Lean kernel server on first run)
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

The first `cargo build` invokes `lake build iris-kernel-server` in `lean/` to
compile the Lean proof kernel. Subsequent builds skip this step unless the Lean
sources change. The Lean server binary is spawned automatically on first kernel
call and stays alive for the process lifetime.

### Tests

```bash
# Run all tests (329+ across lib, bridge, and correspondence suites)
cargo test --features syntax

# Kernel correspondence tests (Lean IPC exercised)
cargo test --features syntax --test test_kernel_lean_correspondence

# Wire format round-trip tests
cargo test --features syntax --test test_lean_bridge
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

### Lean 4 Proof Kernel
The proof kernel is written in Lean 4 (`lean/IrisKernel/`) and runs as a native subprocess (`iris-kernel-server`). All 20 inference rules execute in Lean — the running code IS the formal proof. Lean's type system guarantees that every judgment was derived by a valid sequence of rule applications. The Rust side (`src/iris-bootstrap/src/syntax/kernel/`) wraps results in opaque `Theorem` values with BLAKE3 proof hashes for audit trails, but never evaluates inference rules itself.

Why Lean matters: in a self-improving system, the kernel is the one thing that must never be wrong. By writing it in a language with a built-in proof checker, we get machine-verified correctness — not "tested" correctness, not "reviewed" correctness, but mathematically proven. The kernel can't be silently broken by a mutation, an evolution, or a bug in the Rust code, because Lean won't compile if the proofs don't hold.

### IPC Bridge
The Lean kernel runs as a persistent subprocess, communicating over stdin/stdout pipes. The wire protocol is: `rule_id(u8) + payload_len(u32 LE) + payload` → `result_len(u32 LE) + result`. The server is spawned automatically on first kernel call and stays alive for the process lifetime. Latency per rule call is ~microseconds — negligible versus the millisecond-scale evolution loop.

### Metatheory
47 theorems proven in Lean with zero `sorry` in the core rules. Key results: cost lattice properties, weakening, consistency (no derive-bottom, match exhaustiveness, type uniqueness). 86 cross-validation tests verify correspondence between the Lean kernel and the Rust `Theorem` wrapper.

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

The proof kernel runs out-of-process in Lean, so no `unsafe` Rust is involved in kernel logic. The Rust `lean_bridge` module communicates via safe `std::process` and `std::io` — no FFI, no C shim, no `unsafe` blocks.

## Project Structure

```
src/
  iris-types/          Core data structures (SemanticGraph, Value, types, cost, wire)
  iris-bootstrap/      Bootstrap evaluator + syntax pipeline + kernel bridge
    src/syntax/        Lexer, parser, AST, lowering
    src/syntax/kernel/ Lean IPC bridge, Theorem type, checker, proof hashing
  bin/iris.rs          CLI binary (run, check, solve, daemon, repl)
lean/
  IrisKernel/          Lean 4 proof kernel (Types, Rules, Kernel, FFI, Properties)
  IrisKernelServer.lean  IPC server (stdin/stdout, dispatches 20 rules)
  lakefile.lean        Build config (produces iris-kernel-server binary)
src/iris-programs/    Core .iris programs (19 categories)
examples/             Demo .iris programs + Rust examples
  stdlib/             Standard library type modules (Option, Result, Either, Ordering)
benchmark/             10 Computer Language Benchmarks Game implementations
bootstrap/             Pre-compiled IRIS interpreter (JSON)
iris-clcu/             C CLCU interpreter (AVX-512 + scalar)
tests/                 45+ test files (329+ tests)
docs/                  User guide, language reference, architecture, API, contributing
site/                  Hugo-based documentation website (iris-lang.org)
```

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE) for the full text.

Commercial licensing available. Contact Brian Jones (bojo@bojo.wtf) for details.
