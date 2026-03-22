---
title: "Tools"
description: "IRIS developer tools: CLI, LSP, and more."
layout: "single"
---

## The `iris` CLI {#cli}

The IRIS command-line tool is your primary interface. One binary handles running, checking, evolving, and deploying programs.

| Command | Description |
|---------|-------------|
| `iris run` | Execute an IRIS program |
| `iris solve` | Evolve a solution from a specification |
| `iris run --improve` | Run with observation-driven improvement |
| `iris check` | Type-check correctness obligations |
| `iris repl` | Interactive REPL |
| `iris deploy` | Generate standalone Rust source |

### Run a Program {#run}

```bash
iris run examples/algorithms/factorial.iris 10
# Output: 3628800
```

### Type-Check {#check}

Verify correctness obligations without executing:

```bash
iris check examples/algorithms/factorial.iris
# [OK] factorial: 5/5 obligations satisfied (score: 1.00)
```

### Evolve a Solution {#evolve}

Provide a specification and let IRIS breed a correct implementation:

```bash
iris solve spec.iris
```

### Interactive REPL {#repl}

```bash
iris repl
```

### Observation-Driven Improvement {#improve}

Run any program with automatic tracing, evolution, and hot-swap:

```bash
iris run --improve myprogram.iris 42
```

### Deploy {#deploy}

Generate standalone Rust source:

```bash
iris deploy examples/algorithms/factorial.iris -o factorial.rs
# Compile the output with: rustc --edition 2021 -O factorial.rs -o factorial
```

## Build from Source {#build}

IRIS is a Rust workspace. Building is straightforward:

```bash
# Clone
git clone https://github.com/boj/iris.git
cd iris

# Build (release mode recommended)
cargo build --release

# Build with JIT support (x86-64 only)
cargo build --release --features jit

# Run the test suite (2260+ tests)
cargo test --features jit -- --skip evolution --skip convergence

# Run micro-benchmarks (criterion)
cargo bench --features jit

# Run Benchmarks Game suite (10 programs)
cargo test --release --test test_benchmarks_game
```

### System Requirements {#requirements}

- Rust 1.75+
- C compiler (for CLCU library)
- x86-64 (for AVX-512 CLCU features)

### Crate Structure {#crates}

The workspace is organized into 5 focused crates:

| Crate | Description |
|-------|-------------|
| `iris-types` | SemanticGraph, types, values, wire format |
| `iris-bootstrap` | Bootstrap evaluator + syntax pipeline + LCF proof kernel |
| `iris-exec` | Execution shim: capabilities, effect runtime, service |
| `iris-evolve` | Evolution engine, improvement pipeline |
| `iris-clcu-sys` | FFI bindings to C CLCU interpreter |

## Examples {#examples}

The `examples/` directory contains 90+ programs across 20 categories, ported from classic programming language examples:

| Category | Programs | Highlights |
|----------|----------|------------|
| **algorithms/sorting** | bubble, insertion, merge, quick, selection | All major O(n^2) and O(n log n) sorts |
| **algorithms/searching** | binary search, linear search | With lower_bound variant |
| **algorithms/dynamic-programming** | edit distance, LCS, knapsack, coin change, stairs | Row-by-row DP with scan_left |
| **algorithms/graph** | DFS, BFS, topological sort, Dijkstra, A* | Adjacency list representation |
| **algorithms/math** | sieve, prime factors, Newton sqrt, matrix multiply, Pascal's triangle | |
| **algorithms/** | union-find, KMP string matching, sudoku solver | Complex data structures |
| **data-structures** | stack, queue, BST, priority queue, trie | Functional implementations |
| **string-processing** | ROT13, Caesar cipher, RLE, palindrome, anagram, Morse code | Uses str_from_chars for string output |
| **functional-patterns** | Church encoding, composition, currying, lazy streams, Y combinator, state monad, monad transformers | Writer/Reader/Either/List monads |
| **classic-programs** | FizzBuzz, hello world, Tower of Hanoi, Roman numerals, temperature converter, base converter, Brainfuck interpreter, Luhn algorithm | |
| **games-puzzles** | N-queens, Game of Life, tic-tac-toe (with minimax AI), maze solver | |
| **concurrency** | dining philosophers, pipeline, MapReduce | Concurrent patterns as simulations |
| **interpreters** | Lisp interpreter, regex matcher | S-expression eval, backtracking regex |
| **parsers** | arithmetic parser, JSON parser | Recursive descent with fold_while |
| **compression** | Huffman coding | Full encode pipeline |
| **numerical** | integration (trapezoidal, Simpson's, midpoint, Monte Carlo) | Float64 math |
| **simulation** | cellular automata (Rule 110/30), particle physics | 1D CA, N-body gravity |
| **database** | relational algebra | select, project, join, group_by, order_by, aggregates |
| **crypto** | hash functions (DJB2, FNV-1a, Jenkins) + Rabin-Karp search | |
| **design-patterns** | visitor/AST walker | eval, count, depth, pretty-print, constant folding |
| **knowledge-graph** | taxonomy reasoning | Transitive is_a, property inheritance via kg_* primitives |
| **self-modifying** | self-inspecting optimizer | Uses self_graph, graph_nodes, graph_get_kind |

```bash
# Run any example
iris run examples/algorithms/sorting/quicksort.iris
iris run examples/games-puzzles/game_of_life.iris
iris run examples/interpreters/lisp.iris
```

## Benchmarks {#benchmarks}

IRIS implements all 10 programs from the [Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/):

| Benchmark | Description | IRIS Approach |
|-----------|-------------|---------------|
| **n-body** | Planetary orbit simulation | Float64 math, fold, tuples, cross-fragment calls |
| **spectral-norm** | Spectral norm of infinite matrix | Float64, fold, list_nth |
| **fannkuch-redux** | Pancake flipping puzzle | Integer lists, fold, list_take/drop/append |
| **binary-trees** | Tree allocation + checksum | pow, map, fold, list_range |
| **fasta** | DNA sequence generation | String ops, fold, LCG random |
| **reverse-complement** | DNA reverse complement | String ops, fold, str_eq, str_slice |
| **k-nucleotide** | DNA subsequence frequencies | str_slice, fold, map_insert/get |
| **pidigits** | Compute pi digits | Integer arithmetic, Machin's formula |
| **regex-redux** | DNA pattern matching | str_contains, str_replace (no regex engine) |
| **thread-ring** | Token passing ring | fold, modular arithmetic |

### Run the benchmarks

```bash
# All 10 benchmarks
cargo test --release --test test_benchmarks_game

# Individual benchmark
iris run benchmark/n-body/n-body.iris
```
