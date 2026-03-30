# IRIS User Guide

## Getting Started

### Build from Source

IRIS requires Rust (1.75+) and Lean 4 (for the proof kernel). Build with:

```sh
cargo build --release
```

This produces the `iris` binary at `target/release/iris`. On first build, the Lean proof kernel server is compiled automatically via `lake build`.

The workspace has 2 Rust crates + the Lean 4 kernel:

| Component | Purpose |
|-----------|---------|
| `iris-types` (Rust) | Core data structures: `SemanticGraph`, `Value`, types, cost bounds, wire format |
| `iris-bootstrap` (Rust) | Bootstrap evaluator + syntax pipeline (lexer, parser, lowerer) + kernel IPC bridge |
| `lean/IrisKernel` (Lean 4) | Proof kernel: 20 inference rules, runs as IPC subprocess |

### Your First Program

Create a file `hello.iris`:

```iris
-- Compute the factorial of n
let rec factorial n : Int -> Int [cost: Linear(n)] =
  if n <= 1 then 1
  else n * factorial (n - 1)
```

Run it:

```sh
iris run hello.iris 10
# Output: 3628800
```

### Running Examples

The `src/iris-programs/` and `examples/` directories contains working `.iris` programs organized by category:

```sh
# Algorithms
iris run examples/algorithms/fibonacci.iris 10
iris run examples/algorithms/gcd.iris 12 8
iris run examples/algorithms/collatz.iris 27

# Verified programs (with contracts)
iris run examples/verified/safe_div.iris 10 3
iris run examples/verified/bounded_add.iris 100 200

# Check type safety
iris check examples/verified/bounded_add.iris
```

---

## The IRIS Language

IRIS uses an ML-like surface syntax. Comments start with `--`.

### Literals

```iris
42              -- Int
3.14            -- Float64
true            -- Bool
false           -- Bool
"hello"         -- String
()              -- Unit
```

### Let Bindings

Top-level definitions use `let`:

```iris
let double x = x * 2
```

Local bindings use `let ... in`:

```iris
let main x =
  let y = x + 1 in
  let z = y * 2 in
  z
```

### Recursive Functions

Use `let rec` for recursion:

```iris
let rec fibonacci n : Int -> Int [cost: Unknown] =
  if n <= 1 then n
  else fibonacci (n - 1) + fibonacci (n - 2)
```

### Lambdas

Anonymous functions use backslash notation:

```iris
\x -> x + 1
\x y -> x * y + 1
```

### Types and Cost Annotations

Type annotations use `:` after the parameters. Cost annotations use `[cost: ...]`:

```iris
let add x y : Int -> Int -> Int [cost: Const(1)] = x + y
let sum_to n : Int -> Int [cost: Linear(n)] = fold 0 (+) n
```

Available cost bounds:

| Cost | Meaning |
|------|---------|
| `Unknown` | No cost information (default) |
| `Zero` | Zero cost |
| `Const(n)` | Constant cost `n` |
| `Linear(v)` | Linear in variable `v` |
| `NLogN(v)` | N*log(N) in variable `v` |

### Type Expressions

```iris
Int                     -- Primitive integer
Float64                 -- 64-bit float
Bool                    -- Boolean
String                  -- UTF-8 string
()                      -- Unit type
(Int, Int)              -- Tuple
Int -> Int              -- Function
Int -> Int -> Int       -- Curried function
{x : Int | x > 0}      -- Refinement type
forall a. a -> a        -- Polymorphic type
List<Int>               -- Parameterized type
```

### Type Declarations

Define named types:

```iris
type Pair = (Int, Int)
type Point<T> = (T, T, T)
```

---

## Control Flow

### If/Then/Else

```iris
let abs x = if x >= 0 then x else 0 - x
```

### Match Expressions

Pattern matching with `match ... with`:

```iris
let describe n =
  match n with
  | 0 -> 0
  | 1 -> 1
  | _ -> 2
```

Patterns can be: wildcards (`_`), identifiers, integer literals, or boolean literals.

### Guards

Guards are runtime conditional checks that appear in the SemanticGraph as `Guard` nodes. In surface syntax, they manifest as `if/then/else` expressions. The proof kernel tracks guard costs as `Sum(predicate, Sup(then_branch, else_branch))`.

---

## Iteration

IRIS uses `fold` as the primary iteration primitive, not loops.

### fold

Fold over a range (0 to n-1):

```iris
-- Sum from 0 to n-1
let sum_to n = fold 0 (+) n

-- With explicit lambda:
let sum_to n = fold 0 (\acc i -> acc + i) n
```

### Operator Sections

Use `(+)`, `(*)`, etc. as first-class functions:

```iris
let sum xs = fold 0 (+) xs
let product xs = fold 1 (*) xs
```

### map, filter, zip

Higher-order operations on collections (Tuples):

```iris
-- map a function over elements
let doubled = map (\x -> x * 2) xs

-- filter elements by predicate
let positives = filter (\x -> x > 0) xs

-- zip two collections
let paired = zip xs ys
```

### unfold (Corecursion)

Unfold generates data from a seed:

```iris
-- Generate a sequence from a seed value
-- unfold seed step_function
```

---

## Functions

### Application by Juxtaposition

Functions are applied by juxtaposition (no parentheses needed):

```iris
let result = add 3 5          -- add(3, 5) = 8
let result = factorial 10     -- factorial(10)
```

### Pipes

The pipe operator `|>` threads a value through a chain:

```iris
let result = 10 |> double |> add 1
-- equivalent to: add 1 (double 10)
```

### Tuples

Construct tuples with `(a, b, c)` and access elements with `.0`, `.1`, etc.:

```iris
let pair = (3, 4)
let first = pair.0       -- 3
let second = pair.1      -- 4
```

---

## Effects: I/O, Threading, FFI

Effects are performed through named built-in functions that map to `EffectTag` opcodes.

### Console I/O

```iris
let main = print "hello world"
let main = let line = read_line () in print line
```

### File I/O

```iris
let main path =
  let h = file_open path 0 in       -- 0 = read mode
  let data = file_read_bytes h 4096 in
  let _ = file_close h in
  print data
```

File modes: `0` = read, `1` = write (create/truncate), `2` = append.

### Networking (TCP)

```iris
let main port =
  let listener = tcp_listen port in
  let conn = tcp_accept listener in
  let data = tcp_read conn 1024 in
  let _ = tcp_write conn data in
  tcp_close conn
```

### HTTP (Built from TCP Primitives)

```iris
let http_get host port path =
  let conn = tcp_connect host port in
  let req = str_concat "GET " (str_concat path " HTTP/1.0\r\n\r\n") in
  let _ = tcp_write conn req in
  let response = tcp_read conn 65536 in
  let _ = tcp_close conn in
  response
```

### Threading

```iris
let main x =
  let prog = self_graph () in
  let handle = thread_spawn prog in
  thread_join handle
```

Atomic operations: `atomic_read`, `atomic_write`, `atomic_swap`, `atomic_add`.
Reader-writer locks: `rwlock_read`, `rwlock_write`, `rwlock_release`.

### FFI (Foreign Function Interface)

Call C functions via `ffi_call`:

```iris
let getpid = ffi_call "libc" "getpid" ()
let time = ffi_call "libc" "time" (0)
let result = ffi_call "/path/to/lib.so" "my_function" (arg1, arg2)
```

> **Note:** `mmap_exec` and `call_native` are blocked by default capabilities. These require explicit `allow [ MmapExec ]` capability declarations and should only be used in trusted, verified contexts.

---

## Contracts: requires/ensures

Preconditions and postconditions are declared with `requires` and `ensures`:

```iris
let safe_div x y : Int -> Int -> Int
  requires y != 0
  = x / y

let bounded_add x y : Int -> Int -> Int
  requires x >= -500000 && x <= 500000
  requires y >= -500000 && y <= 500000
  ensures result >= -1000000
  ensures result <= 1000000
  ensures result == x + y
  = x + y

let abs x : Int -> Int
  requires x >= -1000000 && x <= 1000000
  ensures result >= 0
  = if x >= 0 then x else 0 - x
```

Contracts are checked by the proof kernel and the `iris check` command.

---

## Module Capabilities: allow/deny

Control what effects a module can perform:

```iris
allow [FileRead, FileWrite "/tmp/*"]
deny [TcpConnect, ThreadSpawn, MmapExec]

let main path =
  let h = file_open path 0 in
  file_read_bytes h 4096
```

Capability entries can include path/host arguments for fine-grained control. The runtime enforces these restrictions and returns `PermissionDenied` for disallowed effects.

---

## Imports

Import other fragments by content hash:

```iris
import #abc123def456 as my_lib
```

---

## The CLI

```
iris <command> [options]
```

### iris run

Execute an `.iris` program:

```sh
iris run examples/algorithms/factorial.iris 10
# Output: 3628800

iris run examples/algorithms/gcd.iris 12 8
# Output: 4
```

Arguments are parsed as: integers, floats, `true`/`false`, or strings.

### iris check

Type-check and verify a program:

```sh
iris check examples/verified/bounded_add.iris
# [OK] bounded_add: 5/5 obligations satisfied (score: 1.00)
# All 1 definitions verified.
```

The checker auto-detects the minimum verification tier needed and reports per-fragment results.

### iris solve

Evolve a solution from a specification with test cases. Test cases are specified as comments:

```iris
-- test: 1, 2 -> 3
-- test: 5, 5 -> 10
-- test: 0, 0 -> 0
```

```sh
iris solve spec.iris
# Evolving solution from 3 test cases...
# Evolution complete: 42 generations in 1.23s
# Best fitness: correctness=1.0000, performance=0.9500, verifiability=0.8000
```

### iris compile

AOT compile to a native ELF64 binary (x86-64 only):

```sh
iris compile examples/algorithms/factorial.iris -o factorial
./factorial 10
```

### iris deploy

Compile to a standalone Rust source file with embedded bytecode VM:

```sh
iris deploy examples/algorithms/factorial.iris -o factorial.rs
rustc --edition 2021 -O factorial.rs -o factorial
```

### iris daemon

Run the self-improving daemon:

```sh
iris daemon --max-cycles 100 --exec-mode interval:800 --improve-threshold 2.0
```

Options:
- `--exec-mode continuous|interval:N` -- Execution mode (default: `interval:800`)
- `--improve-threshold F` -- Max slowdown for deployment gate (default: `2.0`)
- `--max-stagnant N` -- Give up after N failed improvement attempts (default: `5`)
- `--max-improve-threads N` -- Concurrent improvement threads (default: `2`)
- `--max-cycles N` -- Stop after N cycles

### iris repl

Interactive read-eval-print loop:

```
$ iris repl
IRIS REPL v0.1.0
Type expressions (as let declarations), or :quit to exit.

iris> 2 + 3
5
iris> let double x = x * 2
iris> double 21
42
iris> :quit
```

---

## The Source and Examples Directories

Programs are organized by category:

| Directory | Contents |
|-----------|----------|
| `algorithms/` | Classic algorithms: fibonacci, factorial, gcd, collatz, power, sum_to |
| `verified/` | Programs with contracts: safe_div, bounded_add, abs, clamp |
| `stdlib/` | Standard library: math, string_ops, list_ops, file_ops, json, http_client/server, etc. |
| `io/` | I/O examples: cat, echo_server, http_get, file_copy, timestamp_log |
| `threading/` | Concurrency: spawn_join, parallel_map, atomic_counter, producer_consumer |
| `ffi/` | Foreign function interface: libc, libm |
| `evolution/` | Evolution components written in IRIS: mutation, crossover, fitness_eval, nsga2, etc. |
| `seeds/` | Seed program generators: fold_add, fold_mul, fold_max, map_fold |
| `mutation/` | Mutation operators: insert_node, delete_node, replace_prim, connect |
| `interpreter/` | IRIS interpreter written in IRIS: eval_prim, eval_fold, eval_guard, etc. |
| `compiler/` | Compiler passes written in IRIS: constant_fold, dead_code_elim, regalloc, etc. |
| `checker/` | Verification components: type_check, cost_check, tier_classify |
| `meta/` | Self-modification: self_modify, auto_improve, performance_gate |
| `exec/` | Execution infrastructure: daemon, evaluator, service, capabilities |
| `codec/` | Graph embedding codec: neural_codec, gin_encoder, cosine_similarity |
| `repr/` | Representation utilities: types, fragment_signature, wire_format |
| `store/` | Storage: file_store, serialize_graph, snapshot |
| `foundry/` | Algorithm foundry: solve, fragment_library, bootstrap_problems |
| `population/` | Population management: elitism, pareto_rank, crowding_distance |
| `analyzer/` | Test case analysis: detect_sum, detect_max, detect_linear, etc. |
| `syntax/` | Parser written in IRIS: lexer, tokenize, parse_expr |
