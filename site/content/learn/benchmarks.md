---
title: "Benchmarks"
description: "Performance benchmarks with comparisons to other compiled and interpreted languages."
weight: 90
---

Performance measurements for the bootstrap evaluator, flat evaluator, and native codegen, with comparisons to compiled and interpreted languages from the [Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/). The performance target is compiled-language class (OCaml as benchmark).

## Performance Class

The bootstrap evaluator has two execution modes for fold bodies:

1. **Tree-walker**: traverses the SemanticGraph node-by-node (HashMap lookups, Vec allocations, dynamic dispatch per node).
2. **Flat evaluator**: converts the fold body DAG into a linear instruction sequence and runs it in a tight dispatch loop with no allocations per op. Activates automatically for flattenable fold closures.

| Metric | Tree-walker | Flat evaluator | Native x86-64 | Bytecode (CPython) | JIT (PyPy) |
|--------|------------|---------------|--------------|-------------------|------------|
| Dispatch cost | ~200 ns/node | ~5–10 ns/op | 0 (machine code) | ~50 ns/instruction | ~5 ns/op |
| Fold iteration (simple) | ~0.55 µs/step | ~0.05 µs/step | ~0.01 µs/step | ~0.1 µs/step | ~0.01 µs/step |
| Fold iteration (n-body) | ~2,600 µs/step | ~4.5 µs/step | **~0.22 µs/step** | ~10 µs/step | ~0.2 µs/step |
| Startup | < 5 ms | < 5 ms | < 5 ms | ~30 ms | ~200 ms |

The flat evaluator closes much of the gap with bytecode interpreters for compute-heavy fold loops. The native x86-64 backend compiles FlatPrograms to machine code with register allocation and loop-carried state: **AVX for Float64** workloads (~45x faster than CPython, matching PyPy) and **GP registers for integer** workloads (~0.33 µs/token on thread-ring). See [Native Codegen](#native-codegen), [Flat Evaluator](#flat-evaluator) and [JIT Results](#jit) below.

## Evaluator Micro-Benchmarks

Measured with [Criterion.rs](https://bheisler.github.io/criterion.rs/book/) in `--release` mode.

### Core Operations

| Benchmark | Time | Notes |
|-----------|------|-------|
| `eval_lit` | 66 ns | Single literal node |
| `eval_add(3,5)` | 207 ns | Two-input arithmetic |
| `nested_add/10` | 2.1 µs | Chain of 10 additions (~206 ns/add) |
| `nested_add/50` | 11.5 µs | Chain of 50 (~225 ns/add) |
| `nested_add/100` | 25.0 µs | Chain of 100 (~246 ns/add) |
| `nested_add/500` | 76.2 µs | Chain of 500 (~150 ns/add) |

### Fold (Structural Recursion)

| N | Time | Per-Element |
|---|------|-------------|
| 5 | 0.66 µs | 132 ns |
| 10 | 1.06 µs | 106 ns |
| 50 | 4.03 µs | 81 ns |
| 100 | 7.58 µs | 76 ns |

Per-element cost decreases with size, as setup is amortized over more iterations.

### Per-Operation Cost Model

Measured via fold micro-benchmarks at N=50,000 scale:

| Operation | Tree-walker | Flat evaluator | Notes |
|-----------|------------|---------------|-------|
| Integer arithmetic (add + mul) | 0.55 µs | ~0.05 µs | ~11× speedup |
| Float64 arithmetic (add + mul) | 0.59 µs | ~0.05 µs | ~12× speedup |
| String concatenation | 0.43 µs | N/A | Not flattenable |
| Tuple access (list_nth) | 0.58 µs | ~0.01 µs | ~58× speedup |
| Cross-fragment function call | 0.83 µs | ~0.26 µs | Ref inlining + flat |

The flat evaluator activates automatically when the fold body is flattenable (Prim, Lit, Project, Tuple, Guard nodes only). Bodies with Ref nodes are inlined first.

### Syntax Compilation

| Functions | Time | Per-Function |
|-----------|------|-------------|
| 3 | 10.2 µs | 3.4 µs |
| 50 | 316 µs | 6.3 µs |

Tokenizer → parser → lowerer pipeline. Sub-linear per-function growth.

---

<a name="native-codegen"></a>
## Native x86-64 Codegen

Two native codegen paths compile fold bodies to x86-64 machine code, selected automatically based on value types. Both share the same architecture: FlatProgram → register-allocated machine code → W^X mmap'd memory.

### Float64 Path (AVX)

For Float64-heavy fold bodies, compiles to AVX instructions with xmm registers.

| n-body Input | Tree-walker | Flat eval (f64) | Native x86-64 | Speedup (total) |
|-------------|------------|----------------|---------------|-----------------|
| N=1,000 | ~2,600 ms | ~5 ms | **~1.3 ms** | **~2,000×** |
| N=50,000 | N/A | ~225 ms | **~19 ms** | N/A |
| N=500,000 | N/A | N/A | **~140 ms** | N/A |
| N=5,000,000 | N/A | N/A | **~1.13 s** | N/A |

Per-step cost: **0.22 µs/step**, with AVX 3-operand encoding, loop-carried register allocation (xmm0-xmm14), copy propagation, and Lit-aware spill eviction.

### Integer Path (GP Registers)

For integer/boolean fold bodies, compiles to GP register instructions (add, sub, imul, idiv, cmp, cmov, etc.).

| thread-ring Input | Tree-walker | Flat eval | Native x86-64 (GP) | Speedup (total) |
|-------------------|------------|-----------|---------------------|-----------------|
| token=1K | ~1.0 ms | ~0.4 ms | **~0.3 ms** | **~3×** |
| token=5K | ~5.2 ms | ~1.9 ms | **~1.7 ms** | **~3×** |
| token=50K | ~52 ms | ~19 ms | **~16.5 ms** | **~3×** |

Per-token cost: **0.33 µs/token**, using 10 allocatable GP registers (rcx, rbx, rsi, rdi, r8-r11, r14, r15) with loop-carried state, division-by-zero safety, and Lit-aware eviction.

### How It Works (both paths)

1. **Optimize**: copy propagation eliminates PassThrough/Project-from-Tuple ops; constant state detection frees registers for invariant state elements
2. **Compile**: FlatOps → x86-64 bytes (Float64: vaddsd, vmulsd, vsqrtsd, etc.; Integer: add, imul, idiv, cmp, setcc, cmov, etc.)
3. **Register allocation**: linear-scan over allocatable registers (Float64: xmm0-xmm14; Integer: 10 GP regs); Lit-aware eviction skips spill stores for constants
4. **Loop-carried state**: state values persist in registers across iterations; parallel move at loop bottom shuffles output to state registers; state loads/stores only at loop entry/exit
5. **W^X safety**: mmap(RW) → copy code → mprotect(RX) via raw syscalls
6. **Fallback**: transparent to flat interpreter or tree-walker if any op can't be compiled

---

<a name="flat-evaluator"></a>
## Flat Evaluator

The flat evaluator converts fold body DAGs into linear `FlatOp` arrays (8 bytes per op) and executes them in a tight loop with no HashMap lookups, no Vec allocations, no dynamic dispatch per node.

| n-body Input | Tree-walker | Flat evaluator | Speedup |
|-------------|------------|---------------|---------|
| N=10 | 28.2 ms | **1.1 ms** | **25×** |
| N=50 | 130.5 ms | **1.1 ms** | **119×** |
| N=100 | 262.8 ms | **1.3 ms** | **202×** |
| N=1,000 | ~2,600 ms | **~5 ms** | **~520×** |
| N=50,000 | N/A | **~225 ms** | N/A |

The flat evaluator has two tiers:
- **Value path**: generic `Value` enum slots with pooled allocation, ~26 µs/step
- **f64 path**: raw `f64[]` slots for all-Float64 programs, **~4.5 µs/step** (auto-selected when applicable)

The n-body benchmark uses the f64 path, achieving **578× speedup** over the tree-walker. The step body compiles to 103 flat ops operating on a 14-element Float64 tuple.

### How It Works

1. **Flatten**: topological sort of the body DAG → linear instruction array
2. **Ref inlining**: cross-fragment references are expanded inline (multi-arg Lambda chains unwrapped)
3. **Execution**: tight `match` loop over the flat ops (arithmetic, project, tuple, guard) using a pre-allocated slot array
4. **Activation**: automatic for fold closures with flattenable bodies; transparent fallback to tree-walker otherwise

---

<a name="jit"></a>
## Interpreter vs JIT

| Operation | Interpreter | JIT | Speedup |
|-----------|------------|-----|---------|
| `const(42)` | 66 ns | **40 ns** | 1.7× |
| `add(3,5)` | 207 ns | **60 ns** | **3.5×** |
| `mul(6,7)` | ~207 ns | **60 ns** | 3.5× |

The JIT compiles SemanticGraphs to x86-64 machine code via the pure-IRIS code generator (`jit_runtime.iris`), maps it into W^X memory (write then flip to execute-only), and invokes it via System V AMD64 ABI. Feature-gated behind `--features jit`; sandboxes block it by default.

---

## Benchmarks Game

10 programs from the [Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/), implemented in `.iris` and evaluated through the bootstrap evaluator (flat evaluator activates automatically for eligible fold bodies).

### Scaling Results

| Benchmark | Input | Time | Description |
|-----------|-------|------|-------------|
| **n-body** | N=10 | 1.1 ms | 2-body gravitational sim (native x86-64, loop-carried regs) |
| | N=100 | 1.3 ms | ~0.22 µs/step |
| | N=1,000 | ~1.3 ms | |
| | N=50,000 | ~21 ms | |
| | N=500,000 | ~155 ms | |
| **binary-trees** | depth=6 | 0.04 ms | Tree allocation + checksum |
| | depth=8 | 0.1 ms | |
| | depth=10 | 0.5 ms | 2,047 nodes |
| | depth=14 | ~13 ms | 32,767 nodes |
| **fasta** | N=100 | 0.6 ms | DNA sequence (LCG random) |
| | N=500 | 2.8 ms | ~5.6 µs/char |
| **reverse-complement** | N=100 | 0.6 ms | DNA string reversal |
| | N=500 | 2.7 ms | ~5.4 µs/char |
| **k-nucleotide** | N=100 | 0.0 ms | k-mer frequency counting |
| | N=200 | 0.1 ms | |
| **pidigits** | N=10 | 0.1 ms | π digits (Machin formula, i64) |
| | N=15 | 0.1 ms | max ~15 digits (i64 precision) |
| **thread-ring** | token=1K | 0.3 ms | Token passing (native int GP regs) |
| | token=5K | 1.7 ms | ~0.33 µs/token |
| | token=50K | 16.5 ms | |
| **fannkuch-redux** | N=3..6 | < 0.1 ms | Pancake flipping |
| **regex-redux** | N=100 | < 0.1 ms | Pattern matching (no regex engine) |
| **spectral-norm** | 3-elem | < 0.1 ms | Partial (Rust-orchestrated) |

### Comparison to Other Languages

To compare fairly, we extrapolate from measured scaling to the standard CLBG input sizes and compare against published results. All CLBG times are single-threaded, fastest-submitted programs.

| Benchmark | CLBG Input | IRIS (est.) | CPython 3 | Haskell (GHC) | OCaml | Racket |
|-----------|-----------|-------------|-----------|---------------|-------|--------|
| binary-trees | depth=21 | **~1.7 s** | ~100 s | ~2.2 s | ~3.5 s | ~5 s |
| fasta | N=25M | ~140 s | ~40 s | ~1.1 s | ~1.8 s | ~6 s |
| thread-ring (fold) | N=50M | ~16.5 s | ~10 s | ~1.0 s | ~1.5 s | ~3 s |
| thread-ring (fold_until) | N=50M | **~0.04 s** | ~10 s | ~1.0 s | ~1.5 s | ~3 s |
| n-body | N=50M | **~11 s** | ~500 s | ~2.2 s | ~2.0 s | ~8 s |

**Reading the table:**
- **binary-trees**: the tree-walker handles recursive allocation well, **~59x faster than CPython**, 2x faster than OCaml
- **thread-ring** with `fold_until` (early exit): **~37x faster than OCaml** and **~250x faster than CPython**. The fold exits after ~1001 iterations instead of running all 50M. Without early exit (plain `fold`), ~11x behind OCaml.
- **n-body**: native AVX codegen, **~45x faster than CPython**, ~5.5x from OCaml
- **fasta**: 3-4x slower than CPython due to O(n^2) string concatenation (not flattenable)

### Why the Spread?

Performance depends on whether the fold body is flattenable:

| Pattern | Native? | IRIS Cost | CPython Cost | OCaml Cost | IRIS vs CPython | IRIS vs OCaml |
|---------|---------|----------|-------------|-----------|-----------------|---------------|
| Recursive allocation (binary-trees) | No (tree) | Fast (cheap nodes) | Slow (GC pressure) | ~3.5 s | **IRIS 59×** | **IRIS 2×** |
| Float64 fold (n-body) | **AVX xmm0-14** | 0.22 µs/step | ~10 µs/step | ~0.04 µs/step | **IRIS ~45×** | OCaml 5.5× |
| Integer fold (thread-ring) | **GP rcx,rbx,r8-15** | 0.33 µs/token | 0.20 µs/token | ~0.03 µs/token | CPython 1.7× | OCaml ~11× |
| fold_until early exit (thread-ring) | Tree-walker | ~40 µs/iter, exits after ~1K | runs all 50M | runs all 50M | **IRIS ~250×** | **IRIS ~37×** |
| String concatenation (fasta) | No (tree) | 0.43 µs/concat | O(n²) in both | O(1) Buffer | CPython 3.5× | OCaml ~78× |

The native codegen eliminates the tree-walker's per-node overhead (HashMap lookups, Vec allocations) for fold bodies. Float64 folds compile to AVX with 14 xmm registers; integer folds compile to GP code with 10 registers. Workloads that can't be flattened (string ops, recursion) still use the tree-walker.

---

## Evolution Convergence

The evolution engine (`iris-evolve`) uses NSGA-II multi-objective search with 16 mutation operators and phase-adaptive parameter control.

### Default Parameters

| Parameter | Default | Range |
|-----------|---------|-------|
| Population | 64 | 32–128 |
| Max generations | 1,000 | N/A |
| Mutation rate | 0.80 | 0.5–0.95 |
| Crossover rate | 0.50 | N/A |
| Tournament size | 3 | 2–5 |

### Convergence Characteristics

- **Simple problems** (sum, double, inc): solved in generation 0, as the initial population contains the answer
- **Medium problems** (max, abs): typically solved within 10–50 generations
- **Complex problems** (dot product, quadratic): may require 200+ generations

### Improvement Pipeline

The observation-driven improvement system (`--improve`) completes the full trace → evolve → gate → swap pipeline in under 100 ms for simple functions. See [Evolution & Improvement](/learn/daemon/) for details.

---

## Running Benchmarks

```bash
# Criterion micro-benchmarks
cargo bench --bench evaluator

# Benchmarks Game profiling (all 10 programs, scaling analysis)
cargo test --release --features rust-scaffolding --test bench_profiling -- --nocapture

# Full Benchmarks Game suite
./benchmark/run_all.sh

# Single benchmark
./benchmark/n-body/run.sh 100
```
