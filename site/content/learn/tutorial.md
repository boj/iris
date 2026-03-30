---
title: "Tutorial"
description: "Learn the language from first principles, then evolve and improve a program."
weight: 20
---

This tutorial teaches IRIS by building up from simple expressions to a complete program that evolves itself. No prior functional programming experience is required.

> **Note:** This tutorial uses `iris` to invoke the CLI. If you haven't installed IRIS to your PATH, substitute `cargo run --release --bin iris --` for `iris` in all commands below. See [Getting Started](/learn/get-started/) for build instructions.

---

## Part 1: The Language {#language}

### Values and Let Bindings {#values}

The simplest IRIS program is an expression:

```iris
42
```

`let` binds a name to a value. The `in` keyword introduces the expression that uses the binding:

```iris
let x = 10 in x + 1    -- 11
```

You can chain bindings. Each `let..in` introduces a name that the next expression can use:

```iris
let width = 5 in
let height = 3 in
width * height    -- 15
```

### Functions {#functions}

Define a function with `let`, listing parameters after the name:

```iris
let double n = n * 2
```

Call it by writing the function name followed by its argument:

```iris
double 5    -- 10
```

Functions can take multiple parameters:

```iris
let add a b = a + b

add 3 4    -- 7
```

### Type Annotations {#types}

Type annotations are optional, but they document what a function expects and returns. The syntax is `: InputType -> OutputType` after the parameters:

```iris
let double n : Int -> Int = n * 2
```

This says `double` takes an `Int` and returns an `Int`. Multiple parameters chain with `->`:

```iris
let add a b : Int -> Int -> Int = a + b
```

Read this left to right: takes an `Int`, takes another `Int`, returns an `Int`.

### Conditionals {#conditionals}

`if..then..else` works like you'd expect, but both branches must return a value (there are no statements, only expressions):

```iris
let abs n : Int -> Int =
  if n < 0 then 0 - n
  else n
```

### Recursion {#recursion}

Use `let rec` for functions that call themselves:

```iris
let rec factorial n : Int -> Int =
  if n <= 1 then 1
  else n * factorial (n - 1)
```

Parentheses around `(n - 1)` are needed so IRIS reads it as one argument, not as `factorial n` minus `1`.

### Tuples {#tuples}

Tuples are the universal container. They hold any number of values of any type:

```iris
let point = (1, 2, 3)
let pair = ("hello", 42)
let empty = ()
```

Access elements by position with `.0`, `.1`, etc.:

```iris
let x = point.0    -- 1
let y = point.1    -- 2
```

### Lambdas {#lambdas}

A lambda is an anonymous function. The syntax is `\param -> body`:

```iris
let add_one = \x -> x + 1

add_one 5    -- 6
```

Lambdas are most useful when passed to other functions, as you'll see next.

### Fold: Iteration Without Loops {#fold}

IRIS has no `for` or `while` loops. Instead, it has `fold`, which processes a collection one element at a time, accumulating a result.

`fold` takes three arguments:

1. An **initial value** (the accumulator starts here)
2. A **function** that takes the current accumulator and the next element, and returns the new accumulator
3. A **collection** to iterate over

```iris
-- Sum the numbers (1, 2, 3, 4, 5)
fold 0 (\acc x -> acc + x) (1, 2, 3, 4, 5)    -- 15
```

Step by step:
- Start with `acc = 0`
- Element `1`: `acc = 0 + 1 = 1`
- Element `2`: `acc = 1 + 2 = 3`
- Element `3`: `acc = 3 + 3 = 6`
- Element `4`: `acc = 6 + 4 = 10`
- Element `5`: `acc = 10 + 5 = 15`

The shorthand `(+)` wraps an operator as a function, so these are equivalent:

```iris
fold 0 (\acc x -> acc + x) xs
fold 0 (+) xs
```

You can fold over anything. Count elements, find a maximum, build a new tuple:

```iris
-- Count elements
fold 0 (\acc _ -> acc + 1) (10, 20, 30)    -- 3

-- Find the maximum
fold 0 (\acc x -> if x > acc then x else acc) (3, 7, 2, 9, 1)    -- 9
```

### Custom Types {#custom-types}

Define your own types with `type`. The simplest form is an **enum** with named alternatives separated by `|`:

```iris
type Color = Red | Green | Blue
```

Variants can carry data:

```iris
type Option = Some(Int) | None
```

`Some` is a constructor that wraps a value. `None` is a bare constructor with no data. Use `match` to inspect which variant you have:

```iris
let describe x =
  match x with
  | Some(v) -> v
  | None -> 0
```

`match` checks each pattern top to bottom and executes the first one that fits. The variable `v` inside `Some(v)` binds to whatever value was wrapped.

Here's a more complete example -- a safe division function that returns `None` instead of crashing on divide-by-zero:

```iris
type Option = Some(Int) | None

let safe_div a b : Int -> Int -> Option =
  if b == 0 then None
  else Some (a / b)

let result = safe_div 10 3    -- Some(3)
let bad = safe_div 10 0       -- None
```

### The Pipe Operator {#pipe}

The pipe `|>` passes a value as the last argument to the next function. It turns nested calls inside out into a readable left-to-right chain:

```iris
-- Without pipe (read inside out)
fold 0 (+) (filter (\x -> x > 0) xs)

-- With pipe (read left to right)
xs |> filter (\x -> x > 0) |> fold 0 (+)
```

Both compile to the same thing. Use whichever reads better.

### Imports {#imports}

Pull in other modules with `import`:

```iris
import "stdlib/option.iris" as Opt

let x = Some 42
let val = unwrap_or x 0    -- 42
```

The import makes all the module's definitions available. Paths are relative to the importing file. See the [Standard Library](/learn/stdlib/) for what's included.

### Contracts {#contracts}

`requires` and `ensures` state what must be true before and after a function runs. The verifier checks these automatically:

```iris
let safe_div a b : Int -> Int -> Int
  requires b != 0
  ensures result >= 0
  = a / b
```

`requires b != 0` means callers must guarantee `b` isn't zero. `ensures result >= 0` means the function guarantees its output is non-negative. These aren't just comments -- `iris check` proves them.

### Cost Annotations {#costs}

Cost annotations declare a function's computational complexity:

```iris
let sum_to n : Int -> Int [cost: Linear(n)] =
  fold 0 (+) (list_range 0 n)
```

`[cost: Linear(n)]` declares that this function's cost is at most O(n). The verifier checks that the proven cost (derived from the expression structure) does not exceed the declared bound. Overestimating is accepted; underestimating produces a warning or error. Available costs include `Zero`, `Const(k)`, `Linear(n)`, `NLogN(n)`, and `Polynomial(n, d)`. See the [Type System](/learn/type-system/#costs) for the full list.

---

## Part 2: Verify a Program {#verify}

Now let's use the features from Part 1 together. Create a file called `my_math.iris`:

```iris
-- Sum of integers from 0 to n-1
let sum_to n : Int -> Int [cost: Linear(n)]
  requires n >= 0
  ensures result >= 0
  = fold 0 (+) (list_range 0 n)

-- Product of integers from 1 to n (factorial)
let product_to n : Int -> Int [cost: Linear(n)]
  requires n >= 0
  = fold 1 (\acc i -> acc * (i + 1)) (list_range 0 n)

-- Maximum of integers from 0 to n-1
let max_of n : Int -> Int [cost: Linear(n)]
  requires n >= 1
  = fold 0 (\acc i -> if i > acc then i else acc) (list_range 0 n)
```

Run the verifier:

```bash
iris check my_math.iris
```

```
[OK] sum_to: 5/5 obligations satisfied (score: 1.00)
[OK] product_to: 4/4 obligations satisfied (score: 1.00)
[OK] max_of: 4/4 obligations satisfied (score: 1.00)
All 3 definitions verified.
```

Each obligation is a type check, contract, or cost annotation that the verifier checked. For cost annotations, the checker confirms that the proven expression cost does not exceed the declared bound. A score of `1.00` means all passed. If a contract or cost bound were violated, the output would tell you which obligation failed and why.

Run the program:

```bash
iris run my_math.iris 10       -- calls sum_to by default: 45
```

---

## Part 3: Evolve a Solution {#evolve}

So far you've written programs by hand. IRIS can also **evolve** programs from a specification -- you describe what the function should do with test cases, and the evolution engine breeds a program that satisfies them.

### Write a Spec {#spec}

Create a file called `sum_spec.iris` with input/output test cases:

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

Each line defines: given this input, the function should produce this output. The pattern here is the sum of integers from 0 to n-1 (same as `sum_to` from Part 2), but you don't tell the solver *how* to compute it -- only *what* the answers should be.

### Run the Solver {#solve}

```bash
iris solve sum_spec.iris
```

```
Evolving solution from 8 test cases...
Evolution complete: 12 generations in 0.34s
Best fitness: correctness=1.0000, performance=0.9800, verifiability=0.9000
Best program: 3 nodes, 2 edges
```

The solver uses an evolutionary algorithm (NSGA-II) that:

1. **Generates** a population of random candidate programs
2. **Scores** each candidate on three objectives: correctness (does it match the test cases?), performance (how fast is it?), and verifiability (can the proof kernel verify it?)
3. **Selects** the best candidates and mutates them to produce a new generation
4. **Repeats** until it finds a solution or hits the time budget

The result -- `3 nodes, 2 edges` -- is a SemanticGraph (IRIS's internal program representation). It's not text you'd read; it's a data structure the runtime executes directly.

### Multiple Inputs {#multi-input}

Specs support multiple inputs with tuple syntax:

```iris
-- test: (3, 4) -> 7
-- test: (0, 0) -> 0
-- test: (10, 5) -> 15
```

---

## Part 4: Observation-Driven Improvement {#improve}

Evolution from specs is powerful, but it requires you to write test cases up front. **Observation-driven improvement** skips that step: you run a program normally, and IRIS watches what it does, builds test cases from real behavior, evolves faster versions, and swaps them in -- all automatically.

### Run with `--improve` {#run-improve}

Take the `my_math.iris` program from Part 2 and run it with the `--improve` flag:

```bash
iris run --improve my_math.iris 10000
```

```
[improve] daemon started: min_traces=50, threshold=2.0x, budget=5s
45
[improve] attempting sum_to (73 test cases, avg 124.3us)
[improve] deployed sum_to (124.3us -> 68.1us, 45% faster)

[improve] 1 improvement(s) deployed:
  sum_to -- 124.3us -> 68.1us
```

Here's what happened:

1. The program ran normally and produced its answer (`45`)
2. In the background, a daemon **traced** function calls, recording inputs and outputs
3. Once it had enough traces (50 by default), it used them as test cases and **evolved** a faster version
4. The faster version passed two gates: it produces identical outputs on all traces (equivalence gate), and it's no more than 2x slower (performance gate)
5. The daemon **hot-swapped** the improved version into the running program

### What Gets Saved {#saved}

The improved version is saved to a persistent **fragment cache** at `~/.iris/fragments/`:

```
~/.iris/fragments/
  manifest.json            # maps function names to their best version
  861059f04935ef6e.frag    # gen 0: the original compiled version
  24c8b7c66a95e4fc.frag    # gen 1: the evolved replacement
```

Every version is identified by its **BLAKE3 hash** -- a content-addressed fingerprint derived entirely from the program's structure. Two structurally identical programs always get the same hash, regardless of when they were compiled. This means:

- **Deduplication**: identical programs share one file
- **Integrity**: loading a fragment verifies its hash matches
- **No conflicts**: different versions coexist, addressed by hash

### Rerun with the Improved Version {#rerun}

Now run the same program again, *without* `--improve`:

```bash
iris run my_math.iris 10000
```

```
[cache] loaded improved 'sum_to' (gen 1, 24c8b7c66a95e4fc)
[cache] 1 function(s) loaded from ~/.iris/fragments
45
```

Same input, same output -- but the runtime automatically loaded the evolved version from the cache. No `--improve` flag needed. The program is faster and you didn't change a line of code.

### Generational Improvement {#generations}

Each run builds on the previous best. Run with `--improve` again and the daemon starts from the already-improved version, potentially evolving something even better:

```
Gen 0: compile from source      -> original
Gen 1: evolve, save to cache    -> 2.8x faster
Gen 2: load gen 1, evolve again -> potentially faster still
Gen 3: load gen 2, evolve again -> ...
```

Old versions are never deleted. Every generation's fragment stays in the cache, enabling rollback if needed.

### Improvement Options {#options}

| Flag | Default | Description |
|------|---------|-------------|
| `--improve` | off | Enable observation-driven improvement |
| `--improve-threshold` | `2.0` | Max slowdown factor for deployment gate |
| `--improve-min-traces` | `50` | Min observed calls before evolving |
| `--improve-sample-rate` | `0.01` | Fraction of calls to trace (1%) |
| `--improve-budget` | `5` | Max seconds per evolution attempt |

---

## What's Next {#next}

You've now seen the core loop: write a program, verify it, evolve alternatives, and let the runtime improve it over time.

- [Language Guide](/learn/language/) -- full syntax reference: pattern matching, typeclasses, lazy lists, effects, capabilities
- [Standard Library](/learn/stdlib/) -- Option, Result, collections, math, file I/O, HTTP, and more
- [Type System](/learn/type-system/) -- refinement types, parametric types, cost analysis, the proof kernel
- [Evolution & Improvement](/learn/daemon/) -- the full evolution pipeline, content-addressed lifecycle, and meta-evolution from running code
- [Architecture](/learn/architecture/) -- the four-layer stack: evolution, semantics, verification, hardware
