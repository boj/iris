---
title: "Getting Started"
description: "Install IRIS and run your first program."
weight: 10
---

## Install {#install}

IRIS requires Rust and Lean 4:

- **Rust** 1.75+ ([rustup](https://rustup.rs/))
- **Lean 4** ([elan](https://github.com/leanprover/elan) or `nix-shell -p lean4` on NixOS)

The proof kernel is written in Lean 4 and compiled to a native binary. Rust's build script invokes `lake build` automatically on first build.

```bash
# Clone the repository
git clone https://github.com/boj/iris.git
cd iris

# Build (auto-compiles the Lean kernel server on first run)
cargo build --release
```

The `iris` binary will be at `target/release/iris`.

## Run a Program {#run}

A library of example programs is included. Try running the factorial example:

```bash
cargo run --release --bin iris -- run examples/algorithms/factorial.iris 10
# Output: 3628800
```

Or Fibonacci:

```bash
cargo run --release --bin iris -- run examples/algorithms/fibonacci.iris 10
# Output: 55
```

## Write Your First Program {#first-program}

Create a file called `hello.iris`:

```iris
-- Greatest common divisor
let rec gcd a b : Int -> Int -> Int [cost: Unknown] =
  if b == 0 then a
  else gcd b (a % b)
```

Run it:

```bash
cargo run --release --bin iris -- run hello.iris 48 18
# Output: 6
```

## Define Custom Types {#custom-types}

Algebraic data types (sum types with named constructors and pattern matching) are first-class:

```iris
type Result = Ok(Int) | Err(Int)

let safe_divide a b : Int -> Int -> Int =
    if b == 0 then Err 0
    else Ok (a / b)

let show_result r : Int -> Int =
    match r with
      | Ok(v) -> v
      | Err(e) -> 0 - 1
```

You can also pattern match on imported types. Here, `Opt.map` applies a lambda to an `Option` value, and we destructure the result:

```iris
import "stdlib/option.iris" as Opt

-- Use imported HOFs with local lambdas
let doubled = Opt.map (Some(21)) (\x -> x * 2)    -- Some(42)

-- Pattern match on results
match doubled with
| Some(v) -> v    -- 42
| None -> 0
```

See [examples/algebraic-types/](https://github.com/boj/iris/tree/main/examples/algebraic-types) for Option, Result, linked lists, and state machines.

## Import Standard Library Modules {#imports}

Path-based imports let you pull in standard library modules to
work with common patterns like optional values and error handling:

```iris
import "stdlib/option.iris" as Opt
import "stdlib/result.iris" as Res

let safe_head xs : Tuple -> Option =
  if list_len xs == 0 then None
  else Some (list_nth xs 0)

let val = unwrap_or (safe_head (1, 2, 3)) 0  -- 1
```

Paths are resolved relative to the importing file. All top-level declarations
and constructors from the imported file become available in scope.

Higher-order functions like `map`, `filter`, and `and_then` work correctly across import boundaries. You can pass lambdas to functions defined in imported modules and pattern match on the results they return.

## Interactive REPL {#repl}

```bash
cargo run --release --bin iris -- repl
```

## Type-Check a Program {#check}

Verify a program's correctness obligations:

```bash
cargo run --release --bin iris -- check examples/algorithms/factorial.iris
# Output: [OK] factorial: 5/5 obligations satisfied (score: 1.00)
```

## Evolve a Solution {#evolve}

Provide a specification and let the solver evolve a solution:

```bash
cargo run --release --bin iris -- solve spec.iris
```

## Observation-Driven Improvement {#improve}

Run any program with `--improve` and the runtime automatically traces function calls, builds test cases from observed I/O, evolves faster implementations, and hot-swaps them in:

```bash
cargo run --release --bin iris -- run --improve examples/algorithms/factorial.iris 10
```

No manual specs needed. The program improves itself from its own behavior. See [Evolution & Improvement](/learn/daemon/) for options and details.

## Build Options {#build-options}

```bash
# Default build (evaluator + Lean kernel)
cargo build --release

# With syntax pipeline (parser, lowerer, type checker)
cargo build --release --features syntax

# Run tests
cargo test --features syntax
```

| Feature | What it enables |
|---------|-----------------|
| `syntax` | Parser, lowerer, type checker, kernel correspondence tests |

## What's Next {#next}

- Read the [Language Guide](/learn/language/) for full syntax reference
- Explore the [Standard Library](/learn/stdlib/)
- Understand the [Architecture](/learn/architecture/)
