---
title: "IRIS Programming Language"
---

# A language that evolves itself.

IRIS is a self-improving programming language where programs are typed DAGs that can inspect, modify, and optimize themselves at runtime. An evolution engine breeds better implementations. A proof kernel verifies they're correct.

```iris
-- Evolve a sorting function from a specification
let rec sort xs : Tuple -> Tuple [cost: NLogN(xs)]
  requires list_len xs >= 0
  ensures result == sorted(xs)
  = if list_len xs <= 1 then xs
    else let pivot = list_nth xs 0 in
         let less = filter (\x -> x < pivot) xs in
         let greater = filter (\x -> x >= pivot) (list_drop xs 1) in
         list_concat (sort less) (list_append (sort greater) pivot)
```

## Why IRIS?

**Programs as values.** Every IRIS program is a content-addressed SemanticGraph, a typed DAG with 20 node kinds. Programs can reify themselves with `self_graph`, modify their own structure, and evaluate the result. This isn't metaprogramming bolted on. It's the foundation.

**Evolution, not just compilation.** The NSGA-II evolution engine breeds program variants across multiple objectives: correctness, performance, and code size. 16 mutation operators transform graphs. Phase-adaptive selection balances exploration and exploitation.

**Verified by construction.** An LCF-style proof kernel with 20 inference rules checks every candidate. Refinement types express preconditions (`requires`) and postconditions (`ensures`). Algebraic data types with exhaustive pattern matching. A Lean 4 formalization proves the kernel sound.

**Modules and imports.** Path-based imports (`import "stdlib/option.iris" as Opt`) let you compose programs from reusable modules. A standard library (Option, Result, Either, Ordering, math, collections, I/O) provides common patterns out of the box. Content-addressed hash imports ensure reproducible builds.

## The four layers

```
L0  Evolution      NSGA-II search: 16 mutation ops, lexicase + novelty selection
L1  Semantics      SemanticGraph: 20 node kinds, BLAKE3 content-addressed DAGs
L2  Verification   LCF proof kernel: 20 rules, refinement types, cost analysis
L3  Hardware       Tree-walker → flat eval → native AVX x86-64 + CLCU (AVX-512)
```

## Get started

```bash
git clone https://github.com/boj/iris.git && cd iris

# Run a program
iris-stage0 run examples/algorithms/fibonacci.iris

# Evolve a solution from test cases
iris-stage0 run solve_spec.iris

# Run with observation-driven improvement
iris-stage0 run --improve examples/algorithms/factorial.iris 10
```

<div class="cta">

[Install →](/learn/get-started/) · [Learn the language →](/learn/language/) · [Architecture →](/learn/architecture/) · [GitHub](https://github.com/boj/iris)

</div>
