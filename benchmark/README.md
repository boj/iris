# Computer Language Benchmarks Game

10 benchmark implementations from the
[Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/),
written in `.iris` and evaluated through IRIS's execution backends.

## Execution Backends

IRIS supports three execution backends, selectable via `--backend`:

| Backend | Flag | Description |
|---------|------|-------------|
| **Tree-walker** | `--backend tree` | Default. Interprets SemanticGraph nodes directly. |
| **JIT (x86-64)** | `--backend jit` | Compiles to native x86-64 via pure IRIS AOT compiler. |
| **Auto** | `--backend auto` | Uses JIT if the graph is compilable, otherwise tree-walker. |
| **CLCU** | `--backend clcu` | Vectorized C runtime (container-based). Experimental. |

The JIT backend compiles simple expressions (arithmetic, guards, let bindings)
to native machine code. Complex programs (fold, lambda, apply, match) fall back
to the tree-walker automatically.

## Benchmarks

| Benchmark | Description | Status |
|-----------|-------------|--------|
| n-body | Planetary orbit simulation (2-body) | Full |
| spectral-norm | Spectral norm of infinite matrix | Partial (Rust-orchestrated) |
| fannkuch-redux | Pancake flipping (permutation puzzle) | Full (flip counting) |
| binary-trees | Tree allocation + checksum | Full |
| fasta | DNA sequence generation (LCG) | Full |
| reverse-complement | DNA reverse complement | Full |
| k-nucleotide | DNA subsequence frequencies | Full |
| pidigits | Compute pi digits (Machin formula) | Full (i64 precision) |
| regex-redux | DNA pattern matching/replacement | Simplified (no regex) |
| thread-ring | Token passing in a ring | Simulated (pure fold) |

## Results

All times measured in `--release` mode. The tree-walking interpreter evaluates
SemanticGraph nodes directly. JIT compiles to native x86-64 via the pure IRIS
AOT compiler (`aot_compile.iris`).

### Scaling (tree-walker vs JIT)

All CLBG programs use fold/lambda/apply and fall back to tree-walker under JIT.
Both backends produce identical results — the ~2ms JIT overhead is from the
AOT compilability check on first invocation.

| Benchmark | Input | Tree (ms) | JIT (ms) | Notes |
|-----------|-------|-----------|----------|-------|
| binary-trees | depth=6 | 4 | 6 | ✓ identical |
| binary-trees | depth=10 | 4 | 6 | ✓ identical |
| binary-trees | depth=12 | 6 | 8 | ✓ identical |
| binary-trees | depth=14 | 15 | 16 | ✓ identical |
| fasta | N=100 | 5 | 6 | ✓ identical |
| fasta | N=500 | 8 | 9 | ✓ identical |
| fasta | N=1000 | 12 | 13 | ✓ identical |
| pidigits | N=10 | 4 | 5 | ✓ identical |
| pidigits | N=15 | 4 | 6 | ✓ identical |
| pidigits | N=27 | 4 | 6 | ✓ identical |
| thread-ring | token=1K | 4 | 7 | ✓ identical |
| thread-ring | token=5K | 9 | 12 | ✓ identical |

### JIT vs Tree-Walker: Per-Call Performance (50,000 iterations)

For expressions the JIT *can* compile (arithmetic, guards, let bindings),
measured inside a single process to eliminate startup overhead:

| Expression | Tree (ns/call) | JIT (ns/call) | Speedup | Compilable |
|------------|----------------|---------------|---------|------------|
| `a + b` | 261 | 25 | **10.4×** | ✓ |
| `a * b + a - b` | 559 | 22 | **26.0×** | ✓ |
| `(a+b)*(a-b)` | 573 | 20 | **28.0×** | ✓ |
| `a * a + b * b` | 547 | 20 | **26.7×** | ✓ |
| `if a > 0 then a*2 else -a` | 463 | 20 | **22.6×** | ✓ |
| `let x = a+b in x*x` | 492 | 20 | **24.4×** | ✓ |
| `(a, b, a+b)` | 454 | — | — | ✗ |
| `fold 0..9 (acc+i)` | 4,629 | — | — | ✗ |

The JIT achieves **10–28× speedup** over the tree-walker for compilable
expressions. The 20ns JIT call time includes: RwLock read (cache lookup),
Value→i64 conversion, native function call via transmuted pointer, and
Value::Int construction for the result.

### Why the JIT Wins

The optimized dispatch path (hot path only, ~20ns):
1. RwLock read on HashMap by SemanticHash → get cached function pointer
2. Convert `&[Value]` inputs to `[i64; 6]` stack array (no heap allocation)
3. Direct `transmute` to native function pointer → call
4. Return `Value::Int(result)` (48 bytes on stack)

The tree-walker traverses 3–10+ graph nodes per expression, doing
pattern matching on `NodeKind`, hash lookups for edges, and recursive
`eval_node` calls. Even `a + b` requires ~5 node traversals (root Prim,
left InputRef, right InputRef, opcode dispatch, result construction).

### Remaining Bottleneck: AOT Coverage

The CLBG benchmarks still fall back to tree-walker because they use
node kinds the AOT compiler can't handle yet:

| Node Kind | Used By | AOT Status |
|-----------|---------|------------|
| Lambda (2) | All fold bodies, all benchmarks | ✗ Not compilable |
| Apply (1) | Function calls | ✗ Not compilable |
| Ref (7) | Cross-fragment calls | ✗ Not compilable |
| Fold (8) | All iteration | ✓ Supported (but body is Lambda) |

Adding Lambda + Apply compilation would unlock JIT for most benchmarks.

### Comparison to Other Languages (CLBG standard inputs)

Extrapolated from measured scaling to CLBG standard input sizes, compared
against published single-threaded results from the
[Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/).

| Benchmark | Input | IRIS (est.) | CPython 3 | Haskell (GHC) | OCaml | Racket |
|-----------|-------|-------------|-----------|---------------|-------|--------|
| binary-trees | depth=21 | **~1.7 s** | ~100 s | ~2.2 s | ~3.5 s | ~5 s |
| fasta | N=25M | ~175 s | ~40 s | ~1.1 s | ~1.8 s | ~6 s |
| thread-ring | N=50M | ~52 s | ~10 s | ~1.0 s | ~1.5 s | ~3 s |
| n-body | N=50M | ~36 hrs | ~500 s | ~2.2 s | ~2.0 s | ~8 s |

**Key takeaways:**

- **binary-trees** is the standout: the tree-walker handles recursive
  allocation naturally, landing in the same range as Haskell and OCaml
  and **~59× faster than CPython**.
- **fasta** and **thread-ring** are 4–5× slower than CPython — fold iteration
  (0.55 µs/step) is more expensive than CPython's bytecode loop (~0.05 µs/step).
- **n-body** is the worst case: 14-field tuple destructuring per step makes
  it impractical at CLBG scale. Compiled languages are 10,000–100,000× faster.

### Per-Operation Costs

Measured via fold micro-benchmarks at scale:

| Operation | Cost per iteration |
|-----------|-------------------|
| Integer arithmetic (add + mul) | 0.55 µs |
| Float64 arithmetic (add + mul) | 0.59 µs |
| String concatenation | 0.43 µs |
| Tuple access (list_nth) | 0.58 µs |
| Cross-fragment function call | 0.83 µs |

### Execution Tier Comparison (50,000 iterations, in-process)

| Tier | Expression | Per call | vs Tree |
|------|------------|----------|---------|
| Tree-walker | `a + b` | 261 ns | 1.0× |
| Tree-walker | `a * b + a - b` | 559 ns | 1.0× |
| JIT (x86-64) | `a + b` | 25 ns | **10.4×** |
| JIT (x86-64) | `a * b + a - b` | 22 ns | **26.0×** |
| Tree-walker | `fold 0..9` | 4,629 ns | 1.0× |

The JIT generates native x86-64 via the pure IRIS AOT compiler and
dispatches through a direct function pointer (no effect handler overhead).
Cache lookup is 20ns via RwLock<HashMap> by SemanticHash.

### Why the Performance Spread?

The tree-walker's performance depends on the dominant operation pattern:

| Pattern | IRIS cost | CPython cost | Winner |
|---------|----------|-------------|--------|
| Recursive allocation (binary-trees) | Fast — graph node creation is cheap | Slow — GC pressure from Python objects | **IRIS** |
| Loop iteration (thread-ring, fasta) | 0.55 µs/step (fold dispatch) | 0.05 µs/step (bytecode dispatch) | CPython |
| Tuple destructuring (n-body) | 0.58 µs per field | ~0.05 µs (local variable access) | CPython |
| String concat (fasta) | 0.43 µs per concat | Similar (both O(n²)) | Tie |

## Top 3 Optimization Targets

Based on profiling, these are the highest-impact optimizations:

### 1. N-body tuple destructuring: ~10,000x slower than Python

**Problem**: Each n-body timestep destructures a 14-element tuple (`state.0`
through `state.13`), which requires 14 separate node evaluations in the
tree-walker. Combined with cross-fragment calls to `body_body_dv` and
`energy_two_body`, a single fold iteration triggers hundreds of node
evaluations.

**Impact**: N-body is the only benchmark that takes >1ms even at small inputs.
At N=100, it takes 263ms (2.6ms per timestep). This is ~10,000x slower than
Python because Python uses direct variable access while IRIS traverses a graph.

**Fix**: Expand AOT compiler coverage to handle fold bodies and lambda
abstractions. With fold bodies compiled to native loops, n-body per-step
cost could drop from 2.6ms to ~10µs (250× improvement). Requires: AOT
support for Apply, Lambda, Ref nodes.

### 2. Fold iteration overhead: 0.5-0.6 us per step

**Problem**: Every fold iteration requires: (a) building a `list_range`
tuple upfront (O(n) allocation), (b) evaluating the lambda body for each
element via `eval_node` recursion, (c) cloning the accumulator `Value` on
each step. For large n, the `list_range` allocation alone creates an
n-element `Vec<Value::Int>`.

**Impact**: All benchmarks that iterate (n-body, fasta, reverse-complement,
thread-ring, k-nucleotide) are bottlenecked by this ~0.55us/iter minimum.
Thread-ring at token=5000 takes 6.2ms for pure integer decrement.

**Fix**: (a) Replace `list_range` in fold with a lazy counter that doesn't
allocate the full range upfront. (b) Compile fold bodies to bytecode --
eliminate `eval_node` dispatch overhead. (c) Use `Cow<Value>` or in-place
mutation for accumulators that aren't aliased.

### 3. String building is O(n^2) due to concat-in-fold pattern

**Problem**: FASTA and reverse-complement build strings by repeated
`str_concat acc "x"` in a fold. Each concat allocates a new string of
length `len(acc) + 1`, making the total work O(n^2) for building an
n-character string.

**Impact**: FASTA N=500 takes 3.5ms (vs Python's 0.08ms = 44x slower),
and the gap widens with longer strings. Reverse-complement at N=500
takes 2.9ms.

**Fix**: (a) Add a `StringBuilder` primitive or `str_push` opcode that
appends in O(1) amortized time (like `Vec::push`). (b) Alternative:
add a `str_from_chars` primitive that takes a tuple of single-char
strings and produces a string in one allocation.

## Known Limitations

### Nested Lambda Scope

IRIS Gen1's tree-walking interpreter does not support inner lambdas that
reference outer lambda parameters. For example:

```
-- This FAILS:
map (\i -> fold 0 (\acc j -> acc + i + j) xs) ys

-- This WORKS (i is let-bound, not a lambda param):
let i = 42 in fold 0 (\acc j -> acc + i + j) xs
```

This prevents full implementation of spectral-norm (which needs nested
map-over-fold patterns) and mandelbrot (nested 2D iteration).

**Workaround**: Use cross-fragment function calls. Define the inner
computation as a separate named function and call it from the fold lambda.
However, building result vectors element-by-element requires `map` inside
fold (another nested lambda), so this only partially helps.

### Parameter Index Limitation

Function parameters at index >= 2 cannot be reliably used inside fold
lambda bodies. Parameters at indices 0 and 1 work correctly.

**Workaround**: Keep functions to at most 2 parameters, or ensure only
parameters at indices 0-1 are referenced inside fold/map lambdas.

### Integer Division (Truncation, not Floor)

IRIS integer division (`/`) truncates towards zero, not towards negative
infinity. This differs from Python's `//` operator. The pidigits benchmark
required explicit floor-division logic to work correctly.

### list_range Limit

`list_range` is limited to 10,000 elements. This caps fold iteration counts
and prevents thread-ring from running with token > 9999.

### i64 Overflow

All integer arithmetic uses fixed-width i64. The pidigits benchmark uses
the Machin formula to stay within i64 range (~15 digits), whereas the
standard Benchmarks Game version uses arbitrary-precision integers.

### Reserved Words

The following are reserved and cannot be used as variable names:
`result`, `val`, `requires`, `ensures`, `match`, `with`, `if`, `then`,
`else`, `let`, `in`, `fold`, `unfold`, `map`, `filter`, `guard`, `effect`.

## Running

### Run all benchmarks
```bash
./benchmark/run_all.sh
```

### Run a single benchmark
```bash
./benchmark/n-body/run.sh [N]
```

### Run Rust tests (verify correctness)
```bash
cargo test --release --test test_benchmarks_game -- --nocapture
```

### Run profiling benchmark
```bash
cargo test --release --test bench_profiling -- --nocapture
```
