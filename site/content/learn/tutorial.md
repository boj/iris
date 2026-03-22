---
title: "Tutorial"
description: "Build a program from scratch: write, verify, evolve, deploy, and improve automatically."
weight: 20
---

This tutorial walks through writing a program, verifying it, evolving alternatives, deploying it, and running it with observation-driven improvement.

> **Note:** This tutorial uses the short form `iris` to invoke the CLI. If you haven't installed IRIS to your PATH, substitute `cargo run --release --bin iris --` for `iris` in all commands below. See [Getting Started](/learn/get-started/) for build instructions.

## Step 1: Write a Specification {#spec}

Create a file `programs/my_service/spec.iris` that defines what we want: a function that computes the sum of integers from 0 to n-1.

```iris
-- Sum a list of integers.
-- test: 0 -> 0
-- test: 1 -> 0
-- test: 2 -> 1
-- test: 3 -> 3
-- test: 4 -> 6
-- test: 5 -> 10
-- test: 10 -> 45
let sum_to n : Int -> Int [cost: Linear(n)] = fold 0 (+) n
```

The `-- test: <inputs> -> <output>` comments define the specification. Each line provides input(s) and the expected output, separated by `->`. Multiple inputs are comma-separated.

## Step 2: Write the Program {#write}

Create `programs/my_service/service.iris`:

```iris
-- A service that computes various reductions over integer ranges.

-- Sum of integers from 0 to n-1
let sum_to n : Int -> Int [cost: Linear(n)]
  requires n >= 0
  ensures result >= 0
  = fold 0 (+) n

-- Product of integers from 1 to n (factorial)
let product_to n : Int -> Int [cost: Linear(n)]
  requires n >= 0
  = fold 1 (\acc i -> acc * (i + 1)) n

-- Maximum of integers from 0 to n-1
let max_of n : Int -> Int [cost: Linear(n)]
  requires n >= 1
  = fold 0 (\acc i -> if i > acc then i else acc) n

-- Define an operation type instead of using integer codes
type Operation = Sum | Product | Max

-- Entry point: dispatch based on named operation
let main op n : Int -> Int -> Int [cost: Linear(n)] =
  match op with
  | Sum -> sum_to n
  | Product -> product_to n
  | Max -> max_of n
  | _ -> 0
```

Each variant of `Operation` is a named constructor. The compiler verifies exhaustiveness, so if you add a new variant, you must handle it.

### Using Struct Types {#tutorial-structs}

Struct types give named fields to tuples, improving readability without runtime cost:

```iris
-- Define a struct for service configuration
type Config = { operation: Operation, count: Int }

-- Construct with named fields
let default_config : Config = { operation = Sum, count = 10 }

-- Access fields by name
let result = main default_config.operation default_config.count
```

Structs are sugar over tuples: `{ operation = Sum, count = 10 }` compiles to `(Sum, 10)`, and `.operation` resolves to `.0` at compile time.

## Step 3: Verify {#verify}

Run the type checker to verify correctness:

```bash
iris check programs/my_service/service.iris
```

Expected output:

```
[OK] sum_to: 5/5 obligations satisfied (score: 1.00)
[OK] product_to: 4/4 obligations satisfied (score: 1.00)
[OK] max_of: 4/4 obligations satisfied (score: 1.00)
[OK] main: 6/6 obligations satisfied (score: 1.00)
All 4 definitions verified.
```

## Step 4: Test {#test}

Run the service via `main`, which dispatches on the operation type:

```bash
iris run programs/my_service/service.iris Sum 10    # main Sum 10 -> sum_to 10 = 45
iris run programs/my_service/service.iris Product 5 # main Product 5  -> product_to 5 = 120
iris run programs/my_service/service.iris Max 10    # main Max 10 -> max_of 10 = 9
```

Or call the individual functions directly:

```bash
iris run programs/my_service/sum_to.iris 10       # 45
iris run programs/my_service/product_to.iris 5    # 120
iris run programs/my_service/max_of.iris 10       # 9
```

## Step 5: Evolve an Alternative {#evolve}

Create a specification file with test cases:

```iris
-- test: 0 -> 0
-- test: 1 -> 0
-- test: 2 -> 1
-- test: 3 -> 3
-- test: 4 -> 6
-- test: 5 -> 10
-- test: 10 -> 45
-- test: 100 -> 4950
```

Save as `programs/my_service/evolve_spec.iris` and run:

```bash
iris solve programs/my_service/evolve_spec.iris
```

The solver generates candidates, scores them on correctness, performance, and verifiability, and reports the best:

```
Evolving solution from 8 test cases...
Evolution complete: 12 generations in 0.34s
Best fitness: correctness=1.0000, performance=0.9800, verifiability=0.9000
Best program: 3 nodes, 2 edges
```

## Step 6: Deploy {#deploy}

### As Deployable Rust Source {#deploy-rust}

```bash
iris deploy programs/my_service/service.iris -o my_service.rs
rustc --edition 2021 -O my_service.rs -o my_service
```

This generates a self-contained Rust file with the bootstrap evaluator embedded.

### As a Shared Library {#deploy-shared}

The `deploy_shared_lib` API generates a `.so` that exports:

```c
int64_t iris_invoke(int64_t* inputs, size_t num_inputs, int64_t* output);
```

## Step 7: Run with Observation-Driven Improvement {#improve}

The `--improve` flag enables automatic optimization: the runtime traces function calls, builds test cases from observed I/O, evolves faster implementations, and hot-swaps them in.

```bash
iris run --improve --improve-threshold 2.0 myservice.iris
```

| Flag | Default | Description |
|------|---------|-------------|
| `--improve` | off | Enable observation-driven improvement |
| `--improve-threshold` | `2.0` | Max slowdown factor for deployment gate |
| `--improve-min-traces` | `50` | Min observed calls before evolving |
| `--improve-sample-rate` | `0.01` | Fraction of calls to trace (1%) |
| `--improve-budget` | `5` | Max seconds per evolution attempt |

See the [Evolution & Improvement guide](/learn/daemon/) for the full pipeline, benchmarked results, and usage patterns.

## Step 8: Import and Use Standard Library Types {#imports}

Type modules define common algebraic data types. Import
them with path-based imports to get safe error handling and structured values.

### Using Option for safe lookups {#import-option}

```iris
import "stdlib/option.iris" as Opt

-- Safe division that returns None on divide-by-zero
let safe_div a b : Int -> Int -> Option =
  if b == 0 then None
  else Some (a / b)

-- Chain safe operations
let compute a b c : Int -> Int -> Int -> Int =
  let step1 = safe_div a b in
  let step2 = and_then step1 (\v -> safe_div v c) in
  unwrap_or step2 0
```

### Using Result for error propagation {#import-result}

```iris
import "stdlib/result.iris" as Res

let parse_positive n : Int -> Result =
  if n > 0 then Ok n
  else Err n

let double_positive n : Int -> Result =
  and_then (parse_positive n) (\v -> Ok (v * 2))

unwrap_or (double_positive 5) 0    -- 10
unwrap_or (double_positive (0 - 3)) 0  -- 0
```

### Using Ordering for comparisons {#import-ordering}

```iris
import "stdlib/ordering.iris" as Ord

let clamp_to_range lo hi x : Int -> Int -> Int -> Int =
  let cmp_lo = compare x lo in
  let cmp_hi = compare x hi in
  if is_lt cmp_lo then lo
  else if is_gt cmp_hi then hi
  else x
```

### Cross-Import Higher-Order Functions {#import-hof}

Imported modules export higher-order functions that accept lambdas defined in
your module. This pattern chains computations by passing local `\`-lambdas to
imported HOFs like `Opt.and_then` and `Opt.map`:

```iris
import "stdlib/option.iris" as Opt

-- Safe division returning Option
let safe_div a b : Int -> Int -> Option =
  if b == 0 then None
  else Some(a / b)

-- Chain computations using imported HOFs with local lambdas
let compute x : Int -> Option =
  let step1 = safe_div 100 x in
  let step2 = Opt.and_then step1 (\v ->
    if v > 10 then Some(v - 10) else None) in
  Opt.map step2 (\v -> v * 2)
```

Lambdas (`\v -> ...`) defined in your module work correctly when passed to
functions imported from another module. Closures capture their defining
environment regardless of where the higher-order function lives.

Import paths are resolved relative to the importing file. All `let` and `type`
declarations, including constructors like `Some`, `None`, `Ok`, `Err`, are
brought into scope automatically. See the [Language Guide: Imports](/learn/language/#imports) for details.

## Program Patterns {#patterns}

### Self-Modifying Programs {#pattern-self-modify}

Programs can inspect and modify their own graph at runtime. Here, `test_cases` is a tuple of `(input, expected_output)` pairs that the program uses to score itself and its modifications:

```iris
-- test_cases: ((1, 1), (2, 4), (3, 9), ...)
-- Each pair is (input, expected_output).
let self_improve test_cases : Tuple -> Int =
  let program = self_graph () in           -- capture own graph
  let score = graph_eval program test_cases in
  let root = graph_get_root program in
  let modified = graph_set_prim_op program root 0x02 in  -- try mul
  let new_score = graph_eval modified test_cases in
  if new_score > score then new_score      -- keep if better
  else score
```

Running it:

```bash
iris run self_improve.iris '((1,1),(2,4),(3,9),(4,16))'
```

### Capability-Restricted Modules {#pattern-capabilities}

Control what effects your module can perform. This module can read any file and write under `/tmp/`, but cannot open network connections, spawn threads, or execute native code:

```iris
allow [FileRead, FileWrite "/tmp/*"]
deny [TcpConnect, ThreadSpawn, MmapExec]

-- Read a file and return its contents.
-- path: file path string, e.g., "/tmp/data.txt"
let process path : String -> Bytes =
  let h = file_open path "r" in
  let data = file_read_bytes h 4096 in
  let _ = file_close h in
  data
```

```bash
iris run process.iris "/tmp/data.txt"
```

### Concurrent Programs {#pattern-concurrent}

Spawn threads that each execute a program and collect their results:

```iris
-- Run two copies of a computation in parallel.
-- n: input value passed to both threads
let parallel_square n : Int -> Tuple =
  let half = n / 2 in
  let h1 = thread_spawn (self_graph ()) in  -- thread 1: runs this program
  let h2 = thread_spawn (self_graph ()) in  -- thread 2: runs this program
  let r1 = thread_join h1 in
  let r2 = thread_join h2 in
  (r1, r2)
```

```bash
iris run parallel_square.iris 42
# Output: (42, 42)  -- both threads compute the same result
```

### FFI Integration {#pattern-ffi}

Call C functions from IRIS via `ffi_call "library" "function" (args)`:

```iris
-- Call libc functions: getpid() and time(NULL)
let main : Tuple =
  let pid = ffi_call "libc" "getpid" () in    -- returns process ID
  let time = ffi_call "libc" "time" (0) in    -- returns Unix timestamp
  (pid, time)
```

```bash
iris run ffi_example.iris
# Output: (12345, 1711234567)
```
