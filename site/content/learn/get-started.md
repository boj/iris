---
title: "Getting Started"
description: "Install IRIS and run your first program."
weight: 10
---

## Install {#install}

IRIS ships as a self-contained binary. Clone the repository and add it to your PATH:

```bash
git clone https://github.com/boj/iris.git
cd iris
export PATH="$PWD/bootstrap:$PATH"
```

That gives you two commands:
- `iris-native` -- compile and run single-file programs
- `iris-build` -- compile and run multi-file programs with imports

No dependencies required. Optionally, install [Lean 4](https://github.com/leanprover/elan) to use the proof kernel.

## Hello world {#hello-world}

Create a file called `hello.iris`:

```iris
let rec factorial n : Int -> Int =
  if n <= 1 then 1
  else n * factorial (n - 1)
```

Run it with an argument:

```bash
iris-native hello.iris 10
# 3628800
```

`iris-native` compiles the source to native x86-64, finds the last top-level binding, applies the command-line arguments, and prints the result.

## Pattern matching {#pattern-matching}

IRIS has algebraic data types (sum types) with pattern matching. Create `shapes.iris`:

```iris
type Shape = Circle(Int) | Rect(Int, Int)

let area s : Shape -> Int =
    match s with
      | Circle(r) -> 3 * r * r
      | Rect(w, h) -> w * h

let describe s : Shape -> Int =
    match s with
      | Circle(_) -> 0
      | Rect(_, _) -> 1
```

Types are declared with `type Name = Constructor(fields) | ...`. Pattern matching uses `match ... with` and `|`-separated arms.

## Let bindings and higher-order functions {#let-and-hof}

Use `let ... in` for local bindings and lambdas for anonymous functions:

```iris
let rec fast_pow base exp : Int -> Int -> Int =
  if exp == 0 then 1
  else if exp % 2 == 0 then
    let half = fast_pow base (exp / 2) in
    half * half
  else base * fast_pow base (exp - 1)
```

Fold over a range with a higher-order function:

```iris
let sum_to n : Int -> Int =
  fold 0 (+) n
```

## Multi-file projects with imports {#imports}

Split code across files using `import`:

```iris
-- math.iris
let rec gcd a b : Int -> Int -> Int =
  if b == 0 then a
  else gcd b (a % b)
```

```iris
-- main.iris
import "math.iris" as Math

let result = Math.gcd 48 18
```

Run with `iris-build`, which resolves imports before compilation:

```bash
iris-build run main.iris
# 6
```

Import paths are relative to the importing file. All top-level bindings and constructors from the imported module become available under the qualified name.

## iris-native vs iris-build {#tools}

| Command | Use case |
|---------|----------|
| `iris-native <file> <args>` | Single-file programs, no imports |
| `iris-native --compile <file>` | Compile to bytecodes (JSON) |
| `iris-build run <file> [args]` | Multi-file programs with imports |
| `iris-build compile <file> -o out` | Compile multi-file to native binary |
| `bootstrap/build-native-self` | Rebuild the compiler from its own source |

`iris-native` is the core compiler binary, itself built from IRIS source. `iris-build` is a thin wrapper that resolves imports, then delegates to `iris-native`.

## What's next {#next}

- [Language guide](/learn/language/) -- full syntax reference
- [Standard library](/learn/stdlib/) -- Option, Result, collections, I/O
- [Type system](/learn/type-system/) -- refinement types and cost annotations
- [Architecture](/learn/architecture/) -- how the compiler pipeline works
- Browse [examples/](https://github.com/boj/iris/tree/main/examples) for algorithms, data structures, parsers, and more
