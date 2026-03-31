---
title: "Getting Started"
description: "Install IRIS and run your first program."
weight: 10
---

## Install {#install}

IRIS is fully self-hosted. The frozen bootstrap binary (`iris-stage0`) is included in the repository -- no build step required.

Optionally, **Lean 4** ([elan](https://github.com/leanprover/elan) or `nix-shell -p lean4` on NixOS) is needed if you want to rebuild the proof kernel.

```bash
# Clone the repository
git clone https://github.com/boj/iris.git
cd iris

# Add iris-stage0 to your PATH
export PATH="$PWD/bootstrap:$PATH"
```

The `iris-stage0` binary is at `bootstrap/iris-stage0`.

## Run a Program {#run}

A library of example programs is included. Try running the factorial example:

```bash
iris-stage0 run examples/algorithms/factorial.iris 10
# Output: 3628800
```

Or Fibonacci:

```bash
iris-stage0 run examples/algorithms/fibonacci.iris 10
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
iris-stage0 run hello.iris 48 18
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
iris-stage0 repl
```

## Type-Check a Program {#check}

Verify a program's correctness obligations:

```bash
iris-stage0 check examples/algorithms/factorial.iris
# Output: [OK] factorial: 5/5 obligations satisfied (score: 1.00)
```

## Evolve a Solution {#evolve}

Provide a specification and let the solver evolve a solution:

```bash
iris-stage0 run solve_spec.iris
```

## Observation-Driven Improvement {#improve}

Run any program with `--improve` and the runtime automatically traces function calls, builds test cases from observed I/O, evolves faster implementations, and hot-swaps them in:

```bash
iris-stage0 run --improve examples/algorithms/factorial.iris 10
```

No manual specs needed. The program improves itself from its own behavior. See [Evolution & Improvement](/learn/daemon/) for options and details.

## CLI Commands {#cli-commands}

`iris-stage0` is the self-hosted IRIS binary. It supports the following commands:

```bash
# Compile a program to SemanticGraph
iris-stage0 compile <file.iris>

# Run a program
iris-stage0 run <file.iris> [args...]

# Build a native binary
iris-stage0 build <file.iris>

# Run tests
iris-stage0 test

# Rebuild the bootstrap binary
iris-stage0 rebuild
```

## What's Next {#next}

- Read the [Language Guide](/learn/language/) for full syntax reference
- Explore the [Standard Library](/learn/stdlib/)
- Understand the [Architecture](/learn/architecture/)
