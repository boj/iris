# IRIS

**Intelligent Runtime for Iterative Synthesis** -- a self-improving programming language that writes itself. Programs are typed DAGs that evolve, verify, and optimize themselves. IRIS is 100% self-hosted: every component from the compiler to the evolution engine is written in IRIS.

## What is IRIS?

IRIS is both a language and a runtime -- one system, two faces:

- **The language** is an ML-like surface syntax (`.iris` files) that humans and LLMs write. It compiles to SemanticGraph, the canonical program representation.
- **The runtime** is a four-layer execution substrate that evolves, verifies, and runs SemanticGraphs. Evolution can also produce programs directly -- no surface syntax required.

The surface syntax is a projection of the SemanticGraph, not the source of truth. Programs are content-addressed DAGs with 20 node kinds, purely functional, carrying their own type environments. The syntax is how you write them; the graph is what they are.

```
L0  Evolution        -- population search (NSGA-II + lexicase + novelty)
L1  Semantics        -- SemanticGraph (20 node kinds, BLAKE3 content-addressed)
L2  Verification     -- Lean 4 proof kernel (20 inference rules, IPC subprocess)
L3  Hardware         -- mini_eval + JIT (bootstrap evaluator)
```

## Key Properties

- **Self-hosted**: 372 `.iris` files implement the entire system. The compiler pipeline (tokenizer, parser, lowerer), the interpreter, the evolution engine, mutation operators, fitness functions, codec, deploy tooling, LSP -- all written in IRIS. The only non-IRIS component is the Lean 4 proof kernel (which must be external by Lob's theorem -- a system cannot verify its own verifier).
- **Self-improving**: Run any program with `--improve` and the runtime automatically traces function calls, builds test cases from observed I/O, evolves faster implementations via NSGA-II genetic search, gates them for correctness and performance, and hot-swaps improvements into the running program. No manual specs needed; the program improves itself from its own behavior.
- **Verified**: The proof kernel is written in Lean 4 and runs as a separate IPC process (`iris-kernel-server`). The 20 inference rules (CaCIC -- Cost-aware Calculus of Inductive Constructions) prove type safety, cost bounds, and functional properties. Lean's type system guarantees the kernel is correct -- the running code IS the formal proof.
- **Recursive**: Programs improve programs. The evolution engine runs inside the language, and evolved programs can themselves evolve sub-programs. A BLAKE3 Merkle chain provides tamper-evident audit trails for all self-modifications.

## Architecture

IRIS is fully self-hosted. The bootstrap binary (`iris-stage0`) is a frozen, pre-compiled seed that contains a mini evaluator and JIT. Everything else is `.iris` programs compiled through the IRIS pipeline.

```
Bootstrap (frozen binary)
  bootstrap/iris-stage0             -- self-hosted seed (mini_eval + JIT)
  bootstrap/*.json                  -- pre-compiled pipeline stages

Lean 4 Proof Kernel
  lean/IrisKernel/                  -- 20 inference rules (CaCIC), compiled to native
  lean/IrisKernelServer.lean        -- IPC server (stdin/stdout, auto-spawned)

IRIS Programs (243 infrastructure files, 19 categories)
  syntax/          -- tokenizer, parser, lowerer (self-hosting pipeline)
  interpreter/     -- meta-circular interpreter (all 20 node kinds)
  compiler/        -- 10-pass pipeline (monomorphize -> container pack)
  evolution/       -- mutation, selection, crossover, seeds, NSGA-II
  codec/           -- GIN-VAE encoder/decoder, HNSW index
  analyzer/        -- 10 pattern detectors for problem classification
  exec/            -- evaluator, cache, effects, daemon, message bus (uses ADTs)
  checker/         -- tier classification, obligation counting
  foundry/         -- algorithm foundry API, fragment library
  meta/            -- auto-improve, performance gate, instrumentation
  deploy/          -- bytecode serialization, standalone binaries, ELF native
  store/           -- persistence, registry
  stdlib/          -- type modules (Option, Result, Either, Ordering), I/O, collections
  mutation/        -- mutation operators for evolution
  population/      -- population management, elitism
  seeds/           -- seed program generators
  lsp/             -- language server protocol
  bootstrap/       -- bootstrap stage definitions
  repr/            -- program representation utilities

Examples (119 files, 32 categories)
  algorithms/, data-structures/, functional-patterns/, concurrency/,
  self-modifying/, verified/, games-puzzles/, simulation/, ...

Benchmarks (10 programs)
  Computer Language Benchmarks Game suite (binary-trees, n-body, fasta, ...)
```

## Quick Start

### Prerequisites

- **`bootstrap/iris-stage0`** -- the frozen bootstrap binary (included in the repo, pre-compiled)
- **Lean 4** (optional) -- only needed if you want to run the proof kernel for verification

On NixOS, Lean is found automatically via `nix-shell`. On other systems, install
from [leanprover.github.io](https://leanprover.github.io/lean4/doc/setup.html).

### Build and Run

```bash
# Compile an IRIS program to JSON (SemanticGraph)
bootstrap/iris-stage0 compile examples/algorithms/factorial.iris -o factorial.json

# Run an IRIS program directly
bootstrap/iris-stage0 run examples/algorithms/factorial.iris 10
# Output: 3628800

# Build a native binary
bootstrap/iris-stage0 build examples/algorithms/factorial.iris -o factorial

# Run a pre-compiled program
bootstrap/iris-stage0 direct factorial.json 10

# Run through the meta-circular interpreter
bootstrap/iris-stage0 interp bootstrap/interpreter.json factorial.json 10

# Run tests
bootstrap/iris-stage0 test src/iris-programs/
```

### Rebuilding the Pipeline

The IRIS pipeline is self-hosting: the tokenizer, parser, and lowerer are themselves `.iris` programs compiled through the pipeline. To rebuild:

```bash
# Rebuild all pipeline stages from source
bootstrap/iris-stage0 rebuild
```

This compiles `src/iris-programs/syntax/iris_tokenizer.iris`, `iris_parser.iris`, and `iris_lowerer.iris` through the current pipeline stages, producing updated `bootstrap/*.json` files. The stage0 binary itself is frozen and never changes.

## Observation-Driven Improvement

```bash
bootstrap/iris-stage0 run --improve myprogram.iris 42
```

The `--improve` flag enables automatic runtime optimization:

1. **Traces** function calls at a configurable sampling rate (default 1%), recording `(inputs, output, latency)` per function
2. **Builds** test cases from observed I/O, no manual specs needed
3. **Evolves** faster implementations using NSGA-II genetic search (budget: 5s)
4. **Gates** candidates through equivalence (identical outputs on all traces) and performance (<=2x slowdown)
5. **Hot-swaps** improvements atomically into the running program

```
[improve] daemon started: min_traces=50, threshold=2.0x, budget=5s
42
[improve] attempting compute (73 test cases, avg 124.3us)
[improve] deployed compute (124.3us -> 68.1us, 45% faster)
```

Programs can also evolve sub-programs at runtime with `evolve_subprogram`, or generate programs from specs with `bootstrap/iris-stage0 run src/iris-programs/evolution/solve.iris spec.iris`.

## Performance

The bootstrap evaluator combines a mini tree-walker with a JIT backend. 10 programs from the [Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/) are implemented in `benchmark/`.

### Micro-Benchmarks

| Operation | Time |
|-----------|------|
| Literal evaluation | 66 ns |
| `add(3,5)` | 207 ns |
| Fold iteration (integer) | 0.55 us/step |
| Fold iteration (float64) | 0.59 us/step |
| Cross-fragment call | 0.83 us/step |
| JIT `add(3,5)` | 60 ns (3.5x faster) |

### Benchmarks Game (release mode)

| Program | Input | Time | Per-unit cost |
|---------|-------|------|--------------|
| binary-trees | depth=14 | ~13 ms | ~0.4 us/node |
| n-body | N=100 | 262.8 ms | 2.6 ms/step |
| fasta | N=500 | 3.5 ms | 7.0 us/char |
| thread-ring | token=50K | 52.0 ms | 1.04 us/token |
| k-nucleotide | N=200 | 0.7 ms | 3.5 us/base |
| pidigits | N=15 | 0.2 ms | n/a |

### Comparison to Other Languages (CLBG standard inputs)

| Benchmark | Input | IRIS (est.) | CPython 3 | Haskell | OCaml |
|-----------|-------|-------------|-----------|---------|-------|
| binary-trees | depth=21 | **~1.7 s** | ~100 s | ~2.2 s | ~3.5 s |
| fasta | N=25M | ~175 s | ~40 s | ~1.1 s | ~1.8 s |
| thread-ring | N=50M | ~52 s | ~10 s | ~1.0 s | ~1.5 s |

Binary-trees is the strongest result: recursive allocation is the evaluator's sweet spot, **~59x faster than CPython**. Loop-heavy benchmarks (fasta, thread-ring) are 4-5x slower than CPython because fold dispatch (0.55 us/step) is more expensive than bytecode dispatch (~0.05 us/step). See the full [Benchmarks](site/content/learn/benchmarks.md) page for analysis.

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

The standard library provides type modules (Option, Result, Either, Ordering) with higher-order functions (map, and\_then, filter, unwrap\_or\_else, zip\_with), plus modules for math, collections, strings, file I/O, HTTP, threading, and lazy lists.

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

Once compiled to SemanticGraph, programs are evaluated by the bootstrap evaluator (`iris-stage0`). The evaluator handles all 20 node kinds, dispatches effects through a capability-guarded handler, and supports self-evaluation via `graph_eval`.

### Effect System

The evaluator dispatches all 44 effect tags through capability-guarded effect handling. Real I/O (files, TCP, env, time, random, atomic state) is available when capabilities are granted. By default, programs run in a sandbox that allows only pure computation effects (Print, Log, Timestamp, Random).

## Verification

Three layers of verification are provided:

### Lean 4 Proof Kernel
The proof kernel is written in Lean 4 (`lean/IrisKernel/`) and runs as a native subprocess (`iris-kernel-server`). All 20 inference rules execute in Lean -- the running code IS the formal proof. Lean's type system guarantees that every judgment was derived by a valid sequence of rule applications.

Why Lean matters: in a self-improving system, the kernel is the one thing that must never be wrong. By writing it in a language with a built-in proof checker, we get machine-verified correctness -- not "tested" correctness, not "reviewed" correctness, but mathematically proven. The kernel can't be silently broken by a mutation, an evolution, or a bug in any other component, because Lean won't compile if the proofs don't hold.

### IPC Bridge
The Lean kernel runs as a persistent subprocess, communicating over stdin/stdout pipes. The wire protocol is: `rule_id(u8) + payload_len(u32 LE) + payload` -> `result_len(u32 LE) + result`. The server is spawned automatically on first kernel call and stays alive for the process lifetime. Latency per rule call is ~microseconds -- negligible versus the millisecond-scale evolution loop.

### Metatheory
47 theorems proven in Lean with zero `sorry` in the core rules. Key results: cost lattice properties, weakening, consistency (no derive-bottom, match exhaustiveness, type uniqueness).

### BLAKE3 Audit Chain
Every self-modification (component deployment, rollback) is recorded in a tamper-evident BLAKE3 Merkle chain. Each entry's hash covers all fields; each entry's `prev_hash` links to the previous. Modifying any entry breaks the chain.

## Security Model

### Capability-Based Sandboxing

All programs run through capability-guarded effect handling, which enforces fine-grained effect permissions before any I/O reaches the OS. The default sandbox blocks:
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

## Project Structure

```
bootstrap/
  iris-stage0              Frozen self-hosted binary (mini_eval + JIT)
  tokenizer.json           Pre-compiled tokenizer stage
  parser.json              Pre-compiled parser stage
  lowerer.json             Pre-compiled lowerer stage
  interpreter.json         Pre-compiled interpreter
  compiler.json            Pre-compiled compiler
  self_interpreter.json    Pre-compiled self-interpreter
  mini_eval.json           Pre-compiled mini evaluator
  stage0-manifest.json     Manifest of all bootstrap stages

src/iris-programs/         243 .iris infrastructure files (19 categories)
  syntax/                  Self-hosting pipeline (tokenizer, parser, lowerer)
  interpreter/             Meta-circular interpreter
  compiler/                10-pass compilation pipeline
  evolution/               NSGA-II evolution engine
  mutation/                Mutation operators
  population/              Population management
  seeds/                   Seed program generators
  exec/                    Evaluator, cache, effects, daemon
  stdlib/                  Standard library (Option, Result, Either, I/O, collections)
  deploy/                  Bytecode serialization, ELF native binaries
  codec/                   GIN-VAE encoder/decoder
  analyzer/                Problem classification
  checker/                 Tier classification, obligation counting
  foundry/                 Algorithm foundry API
  meta/                    Auto-improve, performance gate
  store/                   Persistence, registry
  lsp/                     Language server protocol
  bootstrap/               Bootstrap stage definitions
  repr/                    Program representation utilities

examples/                  119 .iris example programs (32 categories)
benchmark/                 10 Computer Language Benchmarks Game programs
lean/                      Lean 4 proof kernel (IPC subprocess)
docs/                      User guide, language reference, architecture
site/                      Hugo-based documentation website (iris-lang.org)
```

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE) for the full text.

Commercial licensing available. Contact Brian Jones (bojo@bojo.wtf) for details.
