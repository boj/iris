---
title: "Evolution & Improvement"
weight: 70
description: "Evolve programs from specifications, synthesize functions at runtime, and improve running programs through observation-driven content-addressed evolution."
---

## Overview

Three ways to evolve and improve programs:

1. **`iris run --improve`**: observe a running program and evolve faster implementations automatically
2. **`iris solve`**: give it test cases, get back a synthesized program
3. **`evolve_subprogram`**: call the evolution engine from within running code

Every evolved version is content-addressed via BLAKE3 hashing, serialized to a binary wire format, and stored on disk by hash. See [Content-Addressed Lifecycle](#lifecycle) below for the full walkthrough.

---

## Observation-Driven Improvement

Run any program with `--improve` and the runtime automatically observes function calls, builds test cases from real I/O, evolves faster implementations, and hot-swaps them in, no manual specs needed.

```bash
iris run --improve myprogram.iris 42
```

```
[improve] daemon started: min_traces=50, threshold=2.0x, budget=5s
42
[improve] attempting compute (73 test cases, avg 124.3µs)
[improve] ✓ deployed compute (124.3µs → 68.1µs, 45% faster)

[improve] 1 improvement(s) deployed:
  compute -- 124.3µs → 68.1µs
```

The program runs and produces output normally. The improvement daemon works in a background thread, evolving replacements as traces accumulate.

### Pipeline

The improvement pipeline has five stages:

1. **Trace**: sample function calls at a configurable rate (default 1%), recording `(inputs, output, latency_ns)` in per-function ring buffers (200 entries max)
2. **Synthesize**: when enough traces accumulate (default: 50 calls), deduplicate by input and build test cases automatically
3. **Evolve**: run NSGA-II genetic search against the test cases, scored on correctness, performance, and program size (budget: 5s default)
4. **Gate**: reject candidates that fail either gate:
   - *Equivalence gate*: must produce identical outputs on **all** accumulated traces, not just the evolution subset
   - *Performance gate*: latency must not exceed `threshold × original_latency` (default: 2x)
5. **Swap**: atomically replace the function in the live registry; the program continues with the improved version immediately

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--improve` | off | Enable observation-driven improvement |
| `--improve-threshold` | `2.0` | Max slowdown factor for the performance gate |
| `--improve-min-traces` | `50` | Min observed calls before attempting improvement |
| `--improve-sample-rate` | `0.01` | Fraction of calls to trace (1%) |
| `--improve-budget` | `5` | Max wall-clock seconds per evolution attempt |

### How Tracing Works

The tracer uses an atomic counter for deterministic sampling: every N-th call is recorded (where N = `1/sample_rate`). This avoids locks on the hot path. Each function gets its own ring buffer, so high-frequency functions don't crowd out low-frequency ones.

Traces are never used to change program semantics. They only provide test cases for the evolution engine. If evolution fails or the gate rejects a candidate, nothing changes, and the original function continues running.

---

## Evolving from a Specification

Write a spec as a list of input -> output test cases:

```iris
-- spec.iris
-- test: 0 -> 0
-- test: 1 -> 1
-- test: 2 -> 3
-- test: 3 -> 6
-- test: 4 -> 10
-- test: 5 -> 15
-- test: 10 -> 55
-- test: 100 -> 5050
```

Run the solver:

```bash
iris solve spec.iris
```

```
Evolving solution from 8 test cases...
Evolution complete: 12 generations in 0.34s
Best fitness: correctness=1.0000, performance=0.9800, verifiability=0.9000
Best program: 3 nodes, 2 edges
```

The solver generates candidates using genetic search (NSGA-II), scores them on correctness, performance, and verifiability, and returns the best.

### Spec Format

Each `-- test:` comment defines an input -> output pair. Multiple inputs use tuple syntax:

```iris
-- Single input
-- test: 5 -> 25

-- Multiple inputs
-- test: (3, 4) -> 7
```

---

## Meta-Evolution from Running Code

Programs can evolve sub-programs at runtime using the `evolve_subprogram` opcode (0xA0):

```iris
-- Build test cases as (input, expected_output) pairs
let tc1 = ((1, 1), 2)
let tc2 = ((2, 2), 4)
let tc3 = ((3, 3), 6)
let tc4 = ((0, 0), 0)
let tc5 = ((5, 5), 10)
let test_cases = (tc1, tc2, tc3, tc4, tc5)

-- Evolve a program that satisfies these test cases
let evolved = evolve_subprogram test_cases 200

-- Use the evolved program
let result = graph_eval evolved (4, 4)    -- expects 8
```

### Constraints

| Limit | Value | Reason |
|-------|-------|--------|
| Max wall-clock time | 5 seconds | Prevent runaway evolution |
| Max nesting depth | 1 | Evolved programs cannot recursively evolve |
| Population size | 32 | Smaller for responsiveness |

---

<a name="lifecycle"></a>
## Content-Addressed Lifecycle {#lifecycle}

Every compiled function, every evolved candidate, and every hot-swapped replacement is content-addressed via BLAKE3. This section walks through the full lifecycle with a concrete example.

### The Example Program

Four fold-based computations: pre-evolution and post-evolution versions of both integer and float workloads. The pre-evolution versions carry extra state and do redundant work. Evolution discovers that the extra computation is unnecessary and produces leaner versions.

```iris
-- Pre-evolution: 4-element state, tracks hash + count + sum + max
let hash_pre n =
  let res = fold (0, 0, 0, 0) (\state step ->
    let h = state.0 in
    let c = state.1 in
    let sum = state.2 in
    let max_h = state.3 in
    let new_h = (h * 31 + step) % 1000003 in
    let new_sum = sum + new_h in
    let new_max = if new_h > max_h then new_h else max_h in
    (new_h, c + 1, new_sum, new_max)
  ) (list_range 0 n) in
  (res.0, res.1)

-- Post-evolution: 2-element state, just hash + count
let hash_post n =
  fold (0, 0) (\state step ->
    let new_h = (state.0 * 31 + step) % 1000003 in
    (new_h, state.1 + 1)
  ) (list_range 0 n)
```

Both produce the same `(hash, count)` result, but the post-evolution version has half the state, fewer operations per step, and less register pressure. The evaluator automatically selects the fastest execution path:

| Function | Values | State | Codegen | Registers |
|----------|--------|-------|---------|-----------|
| `hash_pre` | Int | 4-tuple | GP native | 4 loop-carried + intermediates |
| `hash_post` | Int | 2-tuple | GP native | 2 loop-carried + intermediates |
| `ema_pre` | Float64 | 6-tuple | AVX native | 6 loop-carried xmm + intermediates |
| `ema_post` | Float64 | 2-tuple | AVX native | 2 loop-carried xmm + intermediates |

### BLAKE3 Hash Chain {#blake3}

When IRIS compiles source code, each function becomes a `Fragment` containing a `SemanticGraph` (typed DAG). Every node, graph, and fragment gets a BLAKE3 hash:

```
Node     -> NodeId      (64-bit truncated BLAKE3 of kind + type + payload)
Graph    -> SemanticHash (256-bit BLAKE3 of all node hashes)
Fragment -> FragmentId   (256-bit BLAKE3 of graph hash + boundary + types + imports)
```

This is **content addressing**: the hash is determined entirely by structure. Two structurally identical programs always get the same hash, regardless of when or where they're compiled.

```
hash_pre   -> 7ba901d171b1da9c429c5c1f0e0b912f...  (30 nodes, 4-elem state)
hash_post  -> f6e776a5a5bd2eac75f2d46f6361b396...  (17 nodes, 2-elem state)
ema_pre    -> ...                                    (37 nodes, 6-elem state)
ema_post   -> ...                                    (23 nodes, 2-elem state)
```

The pre and post versions produce the **same result** for all inputs, but their SemanticGraphs differ structurally (fewer nodes, smaller state tuples), so they get **different BLAKE3 hashes**.

**Properties:**
- **Deterministic**: compiling the same source twice produces the same FragmentId
- **Sensitive**: any structural change produces a different hash
- **Proof-independent**: proofs are excluded from FragmentId, so verification doesn't change identity

### Pre-Evolution vs Post-Evolution Benchmark {#codegen-benchmark}

Measured at N=50,000 iterations. Both versions compile to native x86-64 machine code, but the post-evolution versions have fewer operations per step and smaller tuple state (less register pressure):

| Function | ns/step | ms total | State | Speedup |
|----------|---------|----------|-------|---------|
| `hash_pre` (integer) | ~556 | ~28 | 4-elem, extra ops | -- |
| **`hash_post`** (integer) | **~200** | **~10** | **2-elem, minimal** | **2.8x** |
| `ema_pre` (Float64) | ~728 | ~36 | 6-elem, statistics | -- |
| **`ema_post`** (Float64) | **~223** | **~11** | **2-elem, just EMA** | **3.3x** |

Integer folds compile to GP register machine code (add, imul, idiv, cmp, cmov). Float folds compile to AVX instructions (vaddsd, vmulsd, vsubsd). The speedup comes from evolution eliminating dead state (unused statistics) and redundant operations, reducing both the number of machine instructions per step and the number of registers needed for loop-carried state.

For comparison, the tree-walker (no native codegen) runs at ~2,000-5,000 ns/step for these programs.

### Serialize & Store {#store}

Fragments serialize to a compact binary wire format (magic `IRIS`, little-endian, deterministic) and are stored on disk named by their BLAKE3 hash:

```
fragments/
  7ba901d171b1da9c.frag   (1,930 bytes)  <- hash_pre  (30 nodes)
  f6e776a5a5bd2eac.frag   (1,311 bytes)  <- hash_post (17 nodes, 32% smaller)
```

The post-evolution version is **32% smaller on disk** -- fewer nodes means fewer bytes to serialize, store, and transfer. Loading a fragment from disk and recomputing its BLAKE3 hash produces the **same FragmentId** as the original:

```
Original:  7ba901d171b1da9c429c5c1f0e0b912f...
Loaded:    7ba901d171b1da9c429c5c1f0e0b912f...  (match)
```

This gives you:
- **Deduplication**: structurally identical programs share one file
- **Integrity**: load a fragment and confirm its hash matches the filename
- **Caching**: skip recompilation if the hash already exists
- **Distribution**: share fragments by hash with integrity guaranteed by BLAKE3

### Hot-Swap {#hot-swap}

When evolution produces an improved version, the name-addressed registry swaps atomically:

```
1. "hasher" -> hash_pre   (FragmentId: 7ba901d1..., ~556 ns/step)
   hasher(500) = (492458, 500)

2. Hot-swap: evolution produced a 2.8x faster version

3. "hasher" -> hash_post  (FragmentId: f6e776a5..., ~200 ns/step)
   hasher(500) = (492458, 500)   <- same result, 2.8x faster
```

Both versions coexist on disk, addressed by their BLAKE3 hashes. The old version is never deleted. This enables rollback, A/B testing, audit trails, and gradual migration.

### The Full Cycle

Putting it all together, the evolution lifecycle is:

1. **Observe**: trace function calls to collect input/output pairs
2. **Evolve**: generate candidate replacements via NSGA-II
3. **Verify**: equivalence gate + performance gate (within 2x)
4. **Hash**: each candidate gets a unique BLAKE3 FragmentId
5. **Store**: serialize to `fragments/{hash}.frag` in the persistent cache
6. **Swap**: atomically replace the live function
7. **Restart**: next `iris run` loads the improved version from cache automatically
8. **Repeat**: generation N+1 builds on generation N's improvements

### Persistent Fragment Cache {#cache}

Improvements persist across process restarts via a content-addressed fragment cache at `~/.iris/fragments/` (or `$IRIS_HOME/fragments/`):

```
~/.iris/fragments/
  manifest.json                  # name -> {fragment_id, generation, improved_at}
  861059f04935ef6e.frag          # gen 0: original (pre-evolution)
  24c8b7c66a95e4fc.frag          # gen 1: evolved replacement
```

The manifest maps function names to their current best BLAKE3 FragmentId and generation number. Each `.frag` file is the wire-format serialization of a Fragment, named by its content hash.

**On startup** (`iris run`), the runtime checks the cache for each compiled function. If a cached improved version exists, it loads it (with BLAKE3 integrity verification) and uses it instead of the freshly compiled original:

```
[cache] loaded improved 'compute' (gen 1, 24c8b7c66a95e4fc)
[cache] 1 function(s) loaded from /home/user/.iris/fragments
```

**After improvement** (`iris run --improve`), when the daemon hot-swaps an improved function, it saves the evolved Fragment to the cache and increments the generation:

```
[cache] saved 'compute' gen 1 -> 24c8b7c66a95e4fc
[improve] deployed compute (556ns -> 200ns, 2.8x faster)
```

**Generational improvement** works because each run starts from the cache, not from source:

```
Gen 0: compile source     -> FragmentId 861059f0... (original)
Gen 1: evolve + save      -> FragmentId 24c8b7c6... (2.8x faster)
Gen 2: load from cache    -> starts from 24c8b7c6, evolves further
Gen 3: load from cache    -> starts from gen 2's best, evolves further
...
```

Old versions are never deleted. Both the original and every improved version coexist on disk, addressed by their BLAKE3 hashes. This enables rollback, A/B testing, and full audit trails.

### Running the Example {#running}

**Step 1: Run the program**

Execute the lifecycle example. The evaluator automatically compiles fold bodies to native x86-64 (GP registers for integer, AVX for float):

```bash
iris run examples/lifecycle/content-addressed-evolution.iris 1000
```

```
(653622, 1000)
```

**Step 2: Run with observation-driven improvement**

Enable the improvement daemon. It traces calls, evolves faster candidates, gates them, hot-swaps, and saves to the persistent cache:

```bash
iris run --improve examples/lifecycle/content-addressed-evolution.iris 50000
```

```
[improve] daemon started: min_traces=50, threshold=2.0x, budget=5s
(492458, 50000)
[cache] saved 'compute' gen 1 -> 24c8b7c66a95e4fc
[improve] deployed compute (556.2ns -> 200.4ns, 2.78x faster)
```

**Step 3: Run again (loads improved version from cache)**

The next run automatically picks up the evolved version:

```bash
iris run examples/lifecycle/content-addressed-evolution.iris 50000
```

```
[cache] loaded improved 'compute' (gen 1, 24c8b7c66a95e4fc)
[cache] 1 function(s) loaded from ~/.iris/fragments
(492458, 50000)
```

Same result, but now running the 2.8x faster evolved version. No `--improve` flag needed; the cache loads automatically.

**Step 4: Run the full lifecycle test**

The integration test demonstrates all 6 stages with printed output:

```bash
cargo test --release --no-default-features --features syntax \
  --test test_content_addressed_lifecycle -- --nocapture
```

```
=== Stage 3: Pre-Evolution vs Post-Evolution (N=50000) ===

  Integer fold (GP native codegen):
    Pre-evolution:    556.2 ns/step  (27.81 ms)  [4-elem state, extra ops]
    Post-evolution:   200.4 ns/step  (10.02 ms)  [2-elem state, minimal ops]
    Speedup: 2.78x

  Float64 fold (AVX native codegen):
    Pre-evolution:    727.7 ns/step  (36.38 ms)  [6-elem state, statistics]
    Post-evolution:   222.5 ns/step  (11.13 ms)  [2-elem state, just EMA]
    Speedup: 3.27x
```

**Step 5: Run the generational persistence test**

The end-to-end test proves improvements survive across restarts:

```bash
cargo test --release --no-default-features --features syntax \
  --test test_fragment_cache_e2e -- --nocapture
```

```
=== Generation 0 (original) ===
  FragmentId: 861059f04935ef6e...
  Result: (492458, 500)

=== Generation 1 (evolved) ===
  FragmentId: 24c8b7c66a95e4fc...
  Result: (492458, 500)
  Saved to cache: gen 1

=== Generation 2 (loaded from cache) ===
  Compiled FragmentId: 861059f0... (original, would be slow)
  Cached FragmentId:   24c8b7c6... (improved, loaded from disk)
  Result: (492458, 500)
  BLAKE3 integrity: verified
```

**Step 6: Continuous improvement with the daemon**

For long-running services, the daemon runs continuously, with each improvement persisted to the cache:

```bash
iris daemon 100 --max-cycles 10
```

### Source Code

| File | Purpose |
|------|---------|
| [`examples/lifecycle/content-addressed-evolution.iris`](https://github.com/boj/iris/blob/main/examples/lifecycle/content-addressed-evolution.iris) | Pre/post evolution programs |
| [`tests/test_content_addressed_lifecycle.rs`](https://github.com/boj/iris/blob/main/tests/test_content_addressed_lifecycle.rs) | 6-stage lifecycle test |
| [`tests/test_fragment_cache_e2e.rs`](https://github.com/boj/iris/blob/main/tests/test_fragment_cache_e2e.rs) | Generational persistence test |
| [`src/iris-bootstrap/src/fragment_cache.rs`](https://github.com/boj/iris/blob/main/src/iris-bootstrap/src/fragment_cache.rs) | Fragment cache implementation |
