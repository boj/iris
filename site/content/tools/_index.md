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
| `iris deploy` | Generate a standalone native binary |

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

Generate a standalone native binary:

```bash
iris-stage0 build examples/algorithms/factorial.iris -o factorial
./factorial 10
# Output: 3628800
```

## Build from Source {#build}

IRIS is fully self-hosted. The frozen bootstrap binary (`iris-stage0`) is included in the repository -- no external build tools required.

```bash
# Clone
git clone https://github.com/boj/iris.git
cd iris

# Add iris-stage0 to your PATH
export PATH="$PWD/bootstrap:$PATH"

# Run a program
iris-stage0 run examples/algorithms/factorial.iris 10

# Compile a program
iris-stage0 compile myprogram.iris

# Build a native binary
iris-stage0 build myprogram.iris

# Run the test suite
iris-stage0 test

# Rebuild the bootstrap binary (self-hosted)
iris-stage0 rebuild
```

### System Requirements {#requirements}

- x86-64 Linux (for the pre-built bootstrap binary)
- Lean 4 (optional, for rebuilding the proof kernel)

### Component Structure {#components}

IRIS is fully self-hosted with the Lean 4 proof kernel as an IPC subprocess:

| Component | Description |
|-----------|-------------|
| `bootstrap/iris-stage0` | Frozen self-hosted binary: compiler, evaluator, all CLI commands |
| `bootstrap/*.json` | Pre-compiled pipeline (tokenizer, parser, lowerer) |
| `lean/IrisKernel` (Lean 4) | Proof kernel: 20 inference rules, IPC server |
| `src/iris-programs/` | 372 `.iris` programs: stdlib, compiler passes, evolution, LSP, deploy |

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
iris-stage0 test benchmarks/

# Individual benchmark
iris-stage0 run benchmark/n-body/n-body.iris
```
