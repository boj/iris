---
title: "Language Guide"
description: "IRIS syntax, types, operators, and constructs."
weight: 30
---

IRIS uses an ML-like surface syntax that compiles to SemanticGraph, the canonical program representation.

## Basics {#basics}

### Comments {#comments}

```iris
-- This is a line comment
```

### Let Bindings {#let-bindings}

```iris
let x = 42 in x + 1

-- With type annotation and cost
let double n : Int -> Int [cost: Unit] =
  n * 2
```

### Recursive Functions {#recursive-functions}

```iris
let rec factorial n : Int -> Int [cost: Linear(n)] =
  if n <= 1 then 1
  else n * factorial (n - 1)
```

### Mutual Recursion {#mutual-recursion}

Use `and` to define mutually recursive functions:

```iris
let rec even n =
  if n == 0 then true else odd (n - 1)
and odd n =
  if n == 0 then false else even (n - 1)
```

### Lambdas {#lambdas}

```iris
let add = \x y -> x + y

-- Used with higher-order functions
fold 0 (\acc x -> acc + x) xs
```

### Comparing Styles {#styles}

`let..in`, `|>`, and lambdas are interchangeable ways to express the same computation. Choose whichever reads best.

**Goal:** take a list, keep positives, double them, sum the result.

With `let..in` (name each intermediate step):

```iris
let process xs : List Int -> Int =
  let positives = filter (\x -> x > 0) xs in
  let doubled = map (\x -> x * 2) positives in
  fold 0 (+) doubled
```

With `|>` (pipeline, no intermediate names):

```iris
let process xs : List Int -> Int =
  xs |> filter (\x -> x > 0)
     |> map (\x -> x * 2)
     |> fold 0 (+)
```

With nested application (fully inline):

```iris
let process xs : List Int -> Int =
  fold 0 (+) (map (\x -> x * 2) (filter (\x -> x > 0) xs))
```

All three compile to the same SemanticGraph. `let..in` is clearest when you need to reuse an intermediate value or give it a meaningful name. `|>` is clearest for linear data pipelines. Nested application works for short expressions but gets hard to read as chains grow.

## Types {#types}

### Primitive Types {#primitive-types}

| Type | Description |
|------|-------------|
| `Int` | 64-bit signed integer |
| `Nat` | Non-negative integer |
| `Float64` | 64-bit floating point |
| `Float32` | 32-bit floating point |
| `Bool` | Boolean (`true` / `false`) |
| `String` | UTF-8 string (via `Bytes`) |
| `Bytes` | Raw byte sequence |
| `Unit` | Unit type (no value) |

### Composite Types {#composite-types}

**Tuples** are the universal container. At runtime, all sequences are `Tuple(Vec<Value>)` -- a dynamically-sized vector. There is no separate list type. `fold`, `map`, `filter`, and all collection operations work on tuples:

```iris
let point = (1, 2, 3)             -- 3 elements
let xs = (10, 20, 30, 40, 50)    -- 5 elements, same type
let pair = ("hello", 42)          -- mixed types are fine
let empty = ()                    -- 0 elements (unit)

-- Access by index
let x = point.0                   -- 1
let y = point.1                   -- 2

-- Collection ops work on any tuple
let total = fold 0 (+) xs         -- 150
let big = filter (\x -> x > 20) xs  -- (30, 40, 50)

-- Tuples grow and shrink dynamically
let extended = list_append xs (60, 70)  -- (10, 20, 30, 40, 50, 60, 70)
let first3 = list_take xs 3             -- (10, 20, 30)
```

Operations like `list_append`, `filter`, and `list_take` return new tuples of different lengths.

### Type System {#type-system-note}

Type annotations are optional but progressively unlock stronger guarantees. See the [Type System](/learn/type-system/) page for the full specification.

- Refinement types: `{x: Int | x > 0}`
- Contracts: `requires` / `ensures` clauses
- Cost annotations: `[cost: Linear(n)]`
- Exhaustive pattern matching for sum types

### Cost Annotations {#cost-annotations}

Every function signature can carry a cost term:

```iris
let sum xs : Int -> Int [cost: Linear(xs)] = fold 0 (+) xs
let sort xs : Int -> Int [cost: NLogN(xs)] = ...
let double n : Int -> Int [cost: Const(1)] = n * 2
```

Available cost terms:

| Term | Meaning |
|------|---------|
| `Zero` | No cost (compile-time only) |
| `Const(n)` | Fixed cost of n steps |
| `Linear(v)` | O(n) in variable v |
| `NLogN(v)` | O(n log n) in variable v |
| `Polynomial(v, d)` | O(n^d) in variable v |
| `Unknown` | Cost not declared (default) |

See [Type System: Cost Annotations](/learn/type-system/#costs) for the full cost lattice.

### Contracts {#contracts}

Functions can declare pre/post-conditions:

```iris
let safe_div a b : Int -> Int -> Int
  requires b != 0
  ensures result >= 0
  = if b == 0 then 0 else a / b
```

See [Type System: Contracts](/learn/type-system/#contracts) for details.

## Control Flow {#control-flow}

### Conditionals {#conditionals}

```iris
if n <= 1 then 1
else n * factorial (n - 1)
```

### Pattern Matching {#pattern-matching}

Match on integers, booleans, or **named constructors** from sum types:

```iris
-- Integer patterns
match expr with
  | 0 -> "zero"
  | 1 -> "one"
  | _ -> "other"

-- ADT constructor patterns
type Option = Some(Int) | None
let unwrap x =
    match x with
      | Some(v) -> v       -- binds inner value to v
      | None -> 0          -- bare constructor match

-- Enum patterns (no payload)
type Color = Red | Green | Blue
let is_red c =
    match c with
      | Red -> true
      | _ -> false
```

Constructor names must start with an uppercase letter. The checker verifies exhaustiveness; missing variants produce a compile error.

#### Tuple Patterns {#tuple-patterns}

Destructure tuples directly in match arms:

```iris
match pair with
  | (a, b) -> a + b

match triple with
  | (x, _, z) -> x * z    -- _ ignores a position
```

#### Guard Clauses {#guard-clauses}

Add conditions to match arms with `when`:

```iris
let classify n =
  match n with
    | n when n > 0 -> "positive"
    | n when n < 0 -> "negative"
    | _ -> "zero"
```

#### Match Inside Lambdas {#match-in-lambda}

Pattern matching works inside lambda bodies, including fold callbacks. This enables powerful idioms like classifying and accumulating over collections:

```iris
type MaybeTag = KnownTag(Int) | UnknownTag

-- Count known tags in a collection using match inside fold
let count_known tags =
  fold 0
    (\acc t ->
      match t with
      | KnownTag(_) -> acc + 1
      | UnknownTag -> acc)
    tags

-- Partition a list of Results by matching inside fold
type Result = Ok(Int) | Err(Int)
let sum_oks results =
  fold 0
    (\acc r ->
      match r with
      | Ok(v) -> acc + v
      | Err(_) -> acc)
    results
```

This composes naturally with other lambda features: you can nest `if`/`else` inside match arms, or chain multiple matches.

### Type Aliases {#type-aliases}

Type aliases introduce a new name for an existing type. They are structural, not nominal -- the alias and the original type are interchangeable:

```iris
type UserId = Int
type Point = (Float64, Float64)
type Handler = String -> Result<String, String>
```

### Algebraic Data Types {#algebraic-data-types}

Sum types let you define custom data types with named alternatives. Declare them with `type`, separating variants with `|`:

```iris
-- Simple enum (no payloads)
type Color = Red | Green | Blue

-- Optional value
type Option = Some(Int) | None

-- Error handling
type Result = Ok(Int) | Err(Int)

-- State machine
type State = Idle | Running(Int) | Paused(Int) | Done(Int)
```

#### Parametric Types {#parametric-types}

Type parameters make ADTs generic over any type:

```iris
type Option<T> = Some(T) | None
type Result<T, E> = Ok(T) | Err(E)
type List<T> = Cons(T, List<T>) | Nil
```

Use them with concrete types:

```iris
let x : Option<Int> = Some 42
let y : Result<String, Int> = Ok "hello"
```

#### Multi-Arity Constructors {#multi-arity-constructors}

Constructors can carry multiple fields:

```iris
type Pair = MkPair(Int, String)
type Rect = MkRect(Float64, Float64, Float64, Float64)

let p = MkPair 1 "hello"
let r = MkRect 0.0 0.0 10.0 20.0
```

**Constructors are automatically bound as functions:**

```iris
let x = Some 42          -- Some : Int -> Option
let y = None              -- None : Option (bare constructor)
let c = Green             -- Green : Color
let s = Running 0         -- Running : Int -> State
```

**Destructure with pattern matching:**

```iris
-- Option: unwrap with a default
let unwrap_or x default_val =
    match x with
      | Some(v) -> v
      | None -> default_val

-- Result: propagate errors
let and_then r f =
    match r with
      | Ok(v) -> f v
      | Err(e) -> Err e

-- Result: transform success values
let map_ok r f =
    match r with
      | Ok(v) -> Ok (f v)
      | Err(e) -> Err e

-- State machine: advance one step
let step state =
    match state with
      | Idle -> Running 0
      | Running(n) -> if n >= 100 then Done n else Running (n + 1)
      | Paused(n) -> Running n
      | Done(n) -> Done n
```

**Design rules:**
- Variant names must start with uppercase (`Some`, `None`, `Red`)
- Tags are assigned by declaration order (first variant = tag 0)
- Bare variants (no parentheses) inject `Unit` internally
- The wildcard `_` pattern matches any variant
- Constructors are scoped to the module where the type is declared

See [examples/algebraic-types/](https://github.com/boj/iris/tree/main/examples/algebraic-types) for working examples: Option, Result, linked lists, and state machines.

### Struct Types {#struct-types}

Struct types (records) give named fields to tuples. They are **sugar over tuples**: `{ x = 3, y = 4 }` compiles to `(3, 4)`, and field access `.x` resolves to `.0` at compile time.

**Define a record type:**

```iris
type Point = { x: Int, y: Int }
type Color = { r: Int, g: Int, b: Int }
```

**Construct with named fields:**

```iris
let origin : Point = { x = 0, y = 0 }
let red : Color = { r = 255, g = 0, b = 0 }
```

**Access fields by name:**

```iris
let px = origin.x     -- resolves to .0 at compile time
let py = origin.y     -- resolves to .1
let g_val = red.g     -- 0
```

Positional access still works on record types: `origin.0` is equivalent to `origin.x`.

**Use in functions:**

```iris
let add_points : Point -> Point -> Point = \a -> \b ->
  { x = a.x + b.x, y = a.y + b.y }

let distance_sq : Point -> Int = \p ->
  p.x * p.x + p.y * p.y
```

**Design rules:**
- Record types are defined with `type Name = { field: Type, ... }`
- Record literals use `{ field = expr, ... }`
- Fields are assigned positions by declaration order (first field = `.0`)
- Field access `.name` is resolved to positional `.0`, `.1`, etc. at compile time
- Positional access (`.0`, `.1`) still works on struct values

### Typeclasses {#typeclasses}

Typeclasses define shared interfaces that types can implement. Methods are dispatched via dictionary passing.

**Declare a typeclass:**

```iris
class Eq<A> where
  eq : A -> A -> Bool
```

**Implement for a type:**

```iris
instance Eq<Int> where
  eq = \a b -> a == b

instance Eq<String> where
  eq = \a b -> string_eq a b
```

**Use with explicit dictionary passing:**

```iris
let all_equal dict xs =
  fold true (\acc x -> acc && dict.eq (list_head xs) x) xs
```

### Fold (Catamorphism) {#fold}

Fold is the primary iteration construct, replacing loops:

```iris
-- Sum a list
fold 0 (+) xs

-- Count elements
fold 0 (\acc x -> acc + 1) xs

-- Map via fold
fold () (\acc x -> list_append acc (f x)) xs
```

### Unfold (Anamorphism) {#unfold}

Unfold generates sequences from a seed value. It's the dual of fold -- fold consumes a structure, unfold produces one.

```iris
-- Generate a range: seed=0, step produces (element, next_state)
-- Stops when the predicate returns true or the budget is exhausted
unfold (\n -> if n >= 10 then None else Some (n, n + 1)) 0
-- produces (0, 1, 2, 3, 4, 5, 6, 7, 8, 9)
```

The eager `unfold` materializes elements into a tuple with a hard cap of 1,000 elements. `list_range` caps at 10,000. For unbounded sequences, use lazy lists instead (see below).

For simple ranges, `list_range` is more convenient:

```iris
let xs = list_range 0 100    -- (0, 1, 2, ..., 99)
```

### Lazy Lists (Infinite Streams) {#lazy-lists}

Lazy infinite lists use **thunks** -- suspended computations that produce one element at a time without materializing the entire sequence in memory.

#### `lazy_unfold` -- Create a lazy stream {#lazy-unfold}

`lazy_unfold` takes a step function and a seed, returning a `Thunk`. The step function receives the current state and returns `(element, next_state)` to produce an element, or `()` to signal end of stream.

```iris
-- Natural numbers: 0, 1, 2, 3, ...
let naturals n = lazy_unfold (\s -> (s, s + 1)) n

-- Fibonacci sequence: 0, 1, 1, 2, 3, 5, 8, ...
let fibs = lazy_unfold (\s -> (s.0, (s.1, s.0 + s.1))) (0, 1)

-- Powers of 2: 1, 2, 4, 8, 16, ...
let powers_of_2 = lazy_unfold (\s -> (s, s * 2)) 1

-- Finite stream: countdown from n to 1
let countdown n = lazy_unfold (\s -> if s > 0 then (s, s - 1) else ()) n

-- Constant stream: repeat a value forever
let repeat x = lazy_unfold (\s -> (s, s)) x
```

Creating a `lazy_unfold` is O(1) -- it returns a `Thunk` immediately without computing any elements.

#### `lazy_take` -- Materialize elements {#lazy-take}

`lazy_take n stream` forces the first `n` elements from a lazy stream, returning them as a tuple. If the stream ends before `n` elements, you get however many were produced.

```iris
lazy_take 10 (naturals 0)        -- (0, 1, 2, 3, 4, 5, 6, 7, 8, 9)
lazy_take 10 fibs                -- (0, 1, 1, 2, 3, 5, 8, 13, 21, 34)
lazy_take 8 powers_of_2          -- (1, 2, 4, 8, 16, 32, 64, 128)
lazy_take 1000 (countdown 5)     -- (5, 4, 3, 2, 1)  -- stream ends at 5 elements
```

#### `lazy_map` -- Transform a lazy stream {#lazy-map}

`lazy_map f stream` applies a function to each element lazily, producing a new lazy stream. No elements are computed until the result is forced with `lazy_take` or `fold`.

```iris
-- Even numbers: 0, 2, 4, 6, 8, ...
let evens = lazy_map (\x -> x * 2) (naturals 0)
lazy_take 5 evens                -- (0, 2, 4, 6, 8)

-- Squares: 0, 1, 4, 9, 16, ...
let squares = lazy_map (\x -> x * x) (naturals 0)
lazy_take 5 squares              -- (0, 1, 4, 9, 16)
```

#### `thunk_force` -- Manual stepping {#thunk-force}

`thunk_force` advances a thunk by one step, returning `(element, next_thunk)`. This is the low-level primitive that `lazy_take` is built on.

```iris
let stream = naturals 0
let step1 = thunk_force stream    -- (0, <thunk>)
let step2 = thunk_force step1.1   -- (1, <thunk>)
let step3 = thunk_force step2.1   -- (2, <thunk>)
```

#### Composing with `fold` {#lazy-fold}

`fold` is thunk-aware -- when given a lazy stream (after `lazy_take`), it processes the materialized elements normally:

```iris
-- Sum first 100 natural numbers = 5050
fold 0 (+) (lazy_take 100 (naturals 1))

-- Sum of squares 0² + 1² + ... + 9² = 285
fold 0 (+) (lazy_take 10 (lazy_map (\x -> x * x) (naturals 0)))
```

#### Pipeline style {#lazy-pipeline}

Lazy streams compose naturally with `|>`:

```iris
let sum_of_even_squares n =
  naturals 0
  |> lazy_map (\x -> x * x)
  |> lazy_map (\x -> x * 2)
  |> lazy_take n
  |> fold 0 (+)
```

### Pipe Operator {#pipe}

```iris
xs |> filter (\x -> x > 0) |> map (\x -> x * 2)
```

## Operators {#operators}

### Arithmetic {#arithmetic}
`+`, `-`, `*`, `/`, `%`, `neg`, `abs`, `min`, `max`, `pow`

### Comparison {#comparison}
`==`, `!=`, `<`, `>`, `<=`, `>=`

### Logic {#logic}
`&&`, `||`, `!`

`&&` and `||` are **short-circuit**: `a && b` desugars to `if a then b else false`, and `a || b` desugars to `if a then true else b`. The right operand is only evaluated when needed.

### Bitwise {#bitwise}
`and`, `or`, `xor`, `not`, `shl`, `shr`, `rotl`, `rotr`, `popcount`, `clz`

## Graph Introspection {#graph-introspection}

Programs can inspect and modify their own representation. Every program is a typed DAG -- these opcodes let you walk, edit, and evaluate that DAG at runtime:

```iris
-- Capture your own program as a value
let me = self_graph ()

-- Walk the graph: get the root, then inspect it
let root = graph_get_root me              -- NodeId
let kind = graph_get_kind me root         -- 0x00=Prim, 0x02=Lambda, etc.

-- Modify: change the root's opcode from add (0x00) to mul (0x02)
let modified = graph_set_prim_op me root 0x02

-- Build new structure: add a Lit node (kind=5) with arity 0
let new_node = graph_add_node_rt me 5 0
graph_connect me root new_node 0          -- wire it as child at slot 0

-- Evaluate the modified graph with inputs (10, 20)
let result = graph_eval modified (10, 20) -- returns 200 if mul
```

This is how programs improve themselves -- by manipulating their own graph representation.

## Effects and I/O {#effects}

Side effects are explicit and controlled via the Effect node kind. 44 effect tags are provided, organized into categories:

```iris
-- File operations
let fd = file_open "data.txt" "r"
let contents = file_read_bytes fd 4096
file_close fd

-- TCP networking
let conn = tcp_connect "example.com" 80
tcp_write conn request
let response = tcp_read conn 65536
tcp_close conn

-- Time and environment
let now = clock_ns
let home = env_get "HOME"

-- Threading
let handle = thread_spawn my_program
let result = thread_join handle

-- JIT compilation (requires --features jit)
let code_bytes = flatten_code (x86_mov_imm64 0 42) in
let fn_ptr = mmap_exec code_bytes in
let result = call_native fn_ptr ()
```

The effect system covers: print/log, file I/O, TCP networking, environment, time, random, threading/atomics, JIT (MmapExec/CallNative), FFI, and custom user effects. See the [full reference](/learn/reference/#effects).

## Capability-Based Security {#capabilities}

Control what effects a program can perform:

```iris
allow [FileRead, FileWrite "/tmp/*", TcpConnect "api.*"]
deny [ThreadSpawn, MmapExec, NetworkListen]
```

Capabilities are checked at runtime before executing Effect nodes. This prevents privilege escalation in evolved code. The JIT effects (`MmapExec`, `CallNative`) are denied by default in sandboxed contexts.

## Imports {#imports}

Two forms of import exist: **path-based** imports for local files and
**hash-based** imports for content-addressed fragments.

### Path-Based Imports {#path-imports}

Import a file by its path, relative to the importing file:

```iris
import "stdlib/option.iris" as Opt
import "stdlib/result.iris" as Res
import "../utils/helpers.iris" as H
```

All top-level `let` and `type` declarations from the imported file become
available in the importing scope. Constructor bindings from imported ADT types
are propagated automatically. After importing `option.iris`, you can use
`Some` and `None` directly.

```iris
import "stdlib/option.iris" as Opt

-- Constructors Some and None are in scope
let x = Some 42
let y = None

-- Functions from the module are available
let val = unwrap_or x 0     -- 42
let mapped = map y (\v -> v + 1)  -- None
```

**Path resolution:** paths are resolved relative to the file that contains the
`import` declaration. If `src/main.iris` imports `"../stdlib/option.iris"`, it
resolves to `stdlib/option.iris` relative to the project root.

**Cycle detection:** circular imports are detected and produce a clear error.
If `a.iris` imports `b.iris` and `b.iris` imports `a.iris`, the compiler will
report the cycle rather than looping forever.

### Hash-Based Imports {#hash-imports}

Content-addressed imports identify an immutable fragment by its BLAKE3 hash:

```iris
import #abc123def456 as math
import #789fead00123 as list
```

The hash refers to the BLAKE3 content hash of the target fragment. Use
`iris store list` or your package registry to look up hash values for
published fragments.

Closures work correctly across import boundaries. You can pass lambdas to imported higher-order functions like `map`, `and_then`, and `filter`:

```iris
import "stdlib/option.iris" as Opt

let doubled = Opt.map (Some 42) (\x -> x * 2)        -- Some(84)
let checked = Opt.and_then (Some 10) (\x ->
  if x > 5 then Some (x * 3) else None)              -- Some(30)
```

## Standard Library {#standard-library}

The standard library provides type modules (Option, Result, Either, Ordering), collections, math, strings, file I/O, HTTP, threading, lazy lists, and monads. See the [Standard Library](/learn/stdlib/) page for complete documentation.

## Keywords {#keywords}

`let`, `rec`, `in`, `val`, `type`, `import`, `as`, `match`, `with`, `when`, `if`, `then`, `else`, `and`, `forall`, `true`, `false`, `requires`, `ensures`, `allow`, `deny`, `class`, `instance`, `where`
