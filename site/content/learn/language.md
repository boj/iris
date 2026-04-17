---
title: "Language Guide"
description: "IRIS syntax, types, operators, and constructs."
weight: 30
---

IRIS is a functional language with ML-like syntax that compiles to SemanticGraph and then to native x86-64 machine code.

## Basics {#basics}

### Let bindings {#let-bindings}

`let` introduces a named value. `let..in` scopes it to an expression:

```iris
let x = 42 in x + 1   -- 43

let double n = n * 2
```

With a type annotation:

```iris
let double n : Int -> Int = n * 2
```

### Functions {#functions}

Functions are defined with `let`. Multiple parameters are curried:

```iris
let add a b = a + b
let result = add 3 4   -- 7
```

### Recursion {#recursion}

Recursive functions require `let rec`. Mutually recursive functions use `and`:

```iris
let rec factorial n =
  if n <= 1 then 1
  else n * factorial (n - 1)

let rec even n = if n == 0 then true else odd (n - 1)
and odd n = if n == 0 then false else even (n - 1)
```

### Comments {#comments}

```iris
-- This is a line comment
```

## Types {#types}

| Type | Description |
|------|-------------|
| `Int` | 64-bit signed integer |
| `String` | UTF-8 string |
| `Bytes` | Raw byte sequence |
| `Unit` | Unit type (`()`) |
| `Bool` | `true` / `false` |
| `Tuple` | Dynamically-sized ordered collection |

There is no separate list type. Tuples serve as both fixed-size records and variable-length sequences. All collection operations (`fold`, `map`, `filter`) work on tuples.

Type annotations are optional. Cost annotations declare asymptotic complexity:

```iris
let sum xs : Tuple -> Int [cost: Linear(xs)] = fold 0 (+) xs
let double n : Int -> Int [cost: Const(1)] = n * 2
```

## Pattern matching {#pattern-matching}

### Match expressions {#match}

Match on integers, constructors, or tuples:

```iris
match expr with
  | 0 -> "zero"
  | 1 -> "one"
  | _ -> "other"
```

### Algebraic data types {#algebraic-data-types}

Define sum types with `type`. Constructors start with an uppercase letter:

```iris
type Option = Some(Int) | None
type Color = Red | Green | Blue
type State = Idle | Running(Int) | Paused(Int) | Done(Int)
```

Destructure with `match`:

```iris
let unwrap_or x default_val =
    match x with
      | Some(v) -> v
      | None -> default_val

let step state =
    match state with
      | Idle -> Running 0
      | Running(n) -> if n >= 100 then Done n else Running (n + 1)
      | Paused(n) -> Running n
      | Done(n) -> Done n
```

Constructors are automatically bound as functions: `Some 42`, `Running 0`, `None`.

### Parametric types {#parametric-types}

```iris
type Option<T> = Some(T) | None
type Result<T, E> = Ok(T) | Err(E)
```

### Tuple patterns {#tuple-patterns}

```iris
match pair with
  | (a, b) -> a + b

match triple with
  | (x, _, z) -> x * z
```

### Guard clauses {#guard-clauses}

```iris
let classify n =
  match n with
    | n when n > 0 -> "positive"
    | n when n < 0 -> "negative"
    | _ -> "zero"
```

## Higher-order functions {#higher-order-functions}

### Lambdas {#lambdas}

Anonymous functions use `\` syntax:

```iris
let add = \x y -> x + y
let inc = \x -> x + 1
```

### Map, filter, fold {#map-filter-fold}

These operate on tuples (the universal collection):

```iris
map (\x -> x * 2) (1, 2, 3, 4)          -- (2, 4, 6, 8)
filter (\x -> x > 2) (1, 2, 3, 4)       -- (3, 4)
fold 0 (\acc x -> acc + x) (1, 2, 3, 4) -- 10
```

`fold` is the primary iteration construct. It replaces loops:

```iris
-- Count elements
fold 0 (\acc _ -> acc + 1) xs

-- Build a new collection
fold () (\acc x -> list_append acc (f x)) xs
```

### Partial application {#partial-application}

Functions are curried, so partial application works naturally:

```iris
let add5 = add_curried 5
let double_all = map_with (\x -> x * 2)
```

### Pipe operator {#pipe}

`|>` threads a value through a chain of functions:

```iris
xs |> filter (\x -> x > 0)
   |> map (\x -> x * 2)
   |> fold 0 (+)
```

## Operators {#operators}

### Arithmetic {#arithmetic}

`+`, `-`, `*`, `/`, `%`

Also available: `neg`, `abs`, `min`, `max`, `pow`.

### Comparison {#comparison}

`==`, `!=`, `<`, `>`, `<=`, `>=`

### Logic {#logic}

`&&`, `||`, `!`

Both `&&` and `||` are short-circuit.

### Bitwise {#bitwise}

`bitand`, `or`, `xor`, `not`, `shl`, `shr`, `rotl`, `rotr`, `popcount`, `clz`

```iris
let masked = bitand value 0xff
let shifted = shr value 8
```

## Strings {#strings}

Strings are UTF-8. Core operations:

```iris
let n = str_len "hello"                       -- 5
let c = char_at "hello" 0                     -- 104 (ASCII 'h')
let s = str_concat "hello" " world"           -- "hello world"
let sub = str_slice "hello world" 0 5         -- "hello"
let greeting = str_concat "Hello, " (str_concat name "!")
```

Convert between types:

```iris
let s = int_to_string 42                      -- "42"
let chars = str_chars "abc"                    -- tuple of char codes
```

## Tuples and lists {#tuples-and-lists}

Tuples are the universal container. There is no separate list type.

### Construction {#tuple-construction}

```iris
let point = (1, 2, 3)
let pair = ("hello", 42)       -- mixed types are fine
let empty = ()                 -- unit / empty tuple
```

### Access and manipulation {#tuple-access}

```iris
let x = point.0               -- 1 (positional access)
let y = point.1               -- 2

let n = list_len xs            -- length
let v = list_nth xs 0          -- element at index
let ys = list_append xs 99     -- append element
let zs = list_concat xs ys     -- concatenate two tuples
let first3 = list_take xs 3    -- first 3 elements
let rest = list_drop xs 2      -- drop first 2 elements
let xs = list_range 0 10       -- (0, 1, 2, ..., 9)
```

## Control flow {#control-flow}

### If/then/else {#if-then-else}

```iris
if n <= 1 then 1
else n * factorial (n - 1)
```

`if` is an expression and always returns a value.

### Fold as iteration {#fold-iteration}

`fold` replaces loops. Accumulate over any tuple:

```iris
-- Sum
fold 0 (+) xs

-- Build a string
fold "" (\acc x -> str_concat acc (int_to_string x)) xs

-- Match inside fold
fold 0 (\acc item ->
  match item with
    | Ok(v) -> acc + v
    | Err(_) -> acc
) results
```

For simple ranges, combine with `list_range`:

```iris
-- Sum 1 to 100
fold 0 (+) (list_range 1 101)
```

## Imports {#imports}

Import a file by path, relative to the importing file:

```iris
import "stdlib/option.iris" as Option
import "stdlib/result.iris" as Result
```

All top-level `let` and `type` declarations become available. Constructors from imported ADT types are propagated automatically:

```iris
import "stdlib/option.iris" as Option

let x = Some 42          -- constructor in scope
let v = unwrap_or x 0    -- function in scope
```

Circular imports are detected at compile time.

## I/O {#io}

### Print {#print}

```iris
let _ = print "hello"
let _ = debug_print value
```

### File I/O {#file-io}

```iris
let contents = file_read "data.txt"

let h = file_open "data.txt" 0      -- 0 = read-only
let bytes = file_read_bytes h 4096
let _ = file_close h
```

### Other effects {#other-effects}

TCP networking, environment variables, time, threading, and JIT compilation are available as built-in effects. See the [full reference](/learn/reference/#effects).
