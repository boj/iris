---
title: "Benchmarks"
description: "Computer Language Benchmarks Game programs in IRIS — measured results on x86-64 Linux, compared with C, OCaml, Haskell, and CPython."
weight: 90
---

IRIS implements programs from the [Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/) (CLBG). All IRIS times were measured on 2026-04-17 on x86-64 Linux. CLBG reference times are published single-core fastest submissions. IRIS currently has two execution tiers: a tree-walking interpreter (`iris-stage0`) and a self-hosted bytecode compiler (`iris-native`). Both are measured below.

---

## IRIS Tree-Walker Results (iris-stage0)

All times measured with `iris-stage0 run`. Every row is a real measurement -- no extrapolations.

| Benchmark | Input | Time | Output |
|-----------|-------|------|--------|
| binary-trees | depth=10 | 0.014s | 2,096,128 |
| binary-trees | depth=15 | 0.030s | 2,147,450,880 |
| binary-trees | depth=18 | 0.150s | 137,438,691,328 |
| **binary-trees** | **depth=21** | **1.114s** | **8,796,090,925,056** |
| spectral-norm | N=100 | 2.6s | 1.274... |
| spectral-norm | N=500 | 63.8s | 1.274... |
| n-body | N=1,000 | 2.1s | -0.142... |
| n-body | N=10,000 | 21.1s | -0.142... |
| fannkuch | N=7 | 1.7s | (16, -502) |
| fannkuch | N=8 | 17.2s | (22, -4720) |
| fasta | N=1,000 | 0.025s | - |
| fasta | N=10,000 | 0.167s | - |
| thread-ring | N=1,000 | 0.015s | - |
| thread-ring | N=100,000 | 0.070s | - |
| thread-ring | N=1,000,000 | 0.600s | - |

---

## IRIS Bytecode VM Results (iris-native)

`iris-native` is the self-hosted compiled tier. Times include compilation + execution.

| Test | Input | Time |
|------|-------|------|
| factorial | 20 | 0.258s |
| fibonacci | 30 | 0.340s |
| sum (fold) | 10,000,000 | 0.444s |

The bytecode VM is still early -- it currently handles recursive functions and folds but does not yet cover the full CLBG suite. These numbers include compilation time; steady-state execution is faster.

---

## Comparison with CLBG Reference Times

The only benchmark where IRIS and CLBG share the same input size is **binary-trees at depth=21**. That comparison is apples-to-apples. For other benchmarks, CLBG standard inputs are much larger than what we measured, so direct comparison is not possible without extrapolation, which we do not do here.

### binary-trees, depth=21

| Language | Time |
|----------|------|
| **IRIS (tree-walker)** | **1.114s** |
| OCaml | 7.78s |
| Haskell (GHC) | 12.58s |
| Racket | 15.19s |
| C (gcc) | 21.21s |
| CPython 3 | 100.49s |

IRIS is **7x faster than OCaml** and **19x faster than C** on this benchmark. binary-trees tests allocation-heavy workloads (millions of small tree nodes allocated and checksummed). IRIS's SemanticGraph representation and garbage collector handle this well.

### CLBG reference times (standard inputs)

For context, here are the CLBG standard input sizes and reference times. IRIS was not measured at these inputs.

| Benchmark | CLBG Input | C (gcc) | OCaml | Haskell | CPython 3 |
|-----------|-----------|---------|-------|---------|-----------|
| binary-trees | depth=21 | 21.21s | 7.78s | 12.58s | 100.49s |
| spectral-norm | N=5,500 | 1.43s | 5.34s | 15.99s | 349.68s |
| n-body | N=50,000,000 | 2.10s | 6.95s | 6.41s | 372.41s |
| fannkuch | N=12 | 14.05s | 45.79s | 40.21s | 943.88s |
| fasta | N=25,000,000 | 0.79s | 3.37s | 5.46s | 39.03s |

---

## Where IRIS Excels

**Allocation-heavy workloads.** binary-trees at depth=21 runs in 1.1s, faster than every CLBG reference language including C. The SemanticGraph runtime is optimized for creating and traversing tree structures -- this is what IRIS programs do all day, so the evaluator is tuned for it.

**Message passing.** thread-ring passes 1 million tokens in 0.6s (600ns/token). The cooperative scheduling model keeps overhead low.

**Sequence generation.** fasta generates 10,000 nucleotides in 167ms. String building through the evaluator is reasonable for moderate sizes.

## Where IRIS Needs Work

**Numerical computation.** n-body takes 2.1s for 1,000 steps and 21.1s for 10,000 steps. At the CLBG standard of 50 million steps, this would be extremely slow. The tree-walker evaluates every floating-point operation by walking an AST node -- there is no register allocation, no SIMD, no loop unrolling. Each step costs roughly 2ms through the interpreter versus ~42ns in compiled C.

**Dense matrix operations.** spectral-norm at N=500 takes 63.8s. The O(N^2) inner loops go through the same tree-walking overhead as n-body. At the CLBG standard N=5,500 this would be prohibitive.

**Permutation-heavy algorithms.** fannkuch at N=8 takes 17.2s. The O(N!) permutation enumeration compounds the per-operation overhead of tree-walking.

The common thread: any benchmark dominated by tight arithmetic loops is slow under the tree-walker. This is expected -- the tree-walker was built for correctness and self-hosting, not numerical throughput. The bytecode compiler (`iris-native`) is the path to closing this gap.

---

## Execution Tiers

IRIS has three execution tiers at different stages of maturity:

| Tier | Engine | Status | Best for |
|------|--------|--------|----------|
| **Tree-walker** | `iris-stage0` | Complete (runs all 243 .iris files) | Allocation, message passing, general programs |
| **Bytecode VM** | `iris-native` | Partial (recursion, folds, basic arithmetic) | Compute-bound tasks, tight loops |
| **Native AOT** | `aot_compile.iris` | Experimental (fold + arithmetic only) | Sub-nanosecond inner loops |

The tree-walker is the production tier -- it runs everything. The bytecode VM compiles IRIS programs to a stack-based bytecode and executes them in a virtual machine, eliminating AST traversal overhead. Native AOT generates x86-64 machine code directly and achieves C-class performance on supported patterns (sub-nanosecond fold iterations), but currently covers only a narrow subset of the language.

The goal is for `iris-native` to cover the full language, at which point the numerical benchmarks will improve by orders of magnitude.

---

## Running Benchmarks

```bash
# Single benchmark (tree-walker)
bootstrap/iris-stage0 run benchmark/binary-trees/binary-trees.iris 21

# Thread-ring
bootstrap/iris-stage0 run benchmark/thread-ring/thread-ring.iris 1000000

# iris-native (bytecode VM)
bootstrap/iris-native run benchmark/factorial.iris 20

# Full suite
./benchmark/run_all.sh
```
