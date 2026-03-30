---
title: "Type System"
description: "IRIS type system: gradual typing, refinements, effects, costs, and contracts."
weight: 50
---

IRIS has a **gradual type system** that ranges from fully dynamic (no annotations) to fully verified (refinement types with contracts).  All programs in the standard distribution pass `compile_checked`, the mandatory type-check pass that runs at compile time.

## At a Glance {#overview}

| Layer | What it checks | Annotation needed? |
|-------|---------------|-------------------|
| **Tier 0** | Types of literals, primitives, tuples, projections | Optional (inferred for simple expressions) |
| **Tier 1** | Function types, let bindings, pattern matching, guards | Type signatures on functions |
| **Tier 2** | Polymorphism, contracts (`requires`/`ensures`), effects | Explicit annotations |
| **Tier 3** | Exhaustive patterns, effect verification, cost enforcement | Full annotations |

The checker is **gradual**: unannotated code still compiles and runs.  Annotations add progressively more guarantees.

### Cross-import type safety {#cross-import}

Higher-order functions work correctly across import boundaries.  Type annotations in imported modules are verified by the checker, and closures passed to imported functions carry their source graph so the evaluator resolves bindings correctly.  This means you can define a generic combinator in one module, import it in another, and pass locally-defined lambdas to it without losing type safety or closure semantics.

---

## Primitive Types {#primitives}

```iris
Int         -- 64-bit signed integer (default)
Nat         -- Non-negative integer
Bool        -- True (1) or false (0)
Float64     -- IEEE 754 double-precision float
Float32     -- IEEE 754 single-precision float
String      -- UTF-8 string (alias for Bytes in type context)
Bytes       -- Raw byte sequence
Unit        -- The unit type (single value)
Program     -- A SemanticGraph handle
```

Unannotated parameters default to `Int`.

---

## Function Types {#functions}

Function types use the arrow `->`, which is right-associative:

```iris
-- A function from Int to Bool
let is_positive x : Int -> Bool = x > 0

-- A curried two-argument function
-- Int -> Int -> Int  means  Int -> (Int -> Int)
let add x y : Int -> Int -> Int = x + y

-- Tuples as parameters
let swap pair : (Int, Int) -> (Int, Int) = (pair.1, pair.0)
```

### Arrow with Cost {#arrow-cost}

Internally, every arrow carries a cost bound: `A → B @ cost`.  When you write `Int -> Int`, the cost defaults to `Unknown`.  Explicit costs go in brackets:

```iris
let double n : Int -> Int [cost: Const(1)] = n * 2
```

---

## Composite Types {#composite}

### Tuples (Product Types) {#tuples}

```iris
-- Tuple type annotation
let origin : (Int, Int) = (0, 0)

-- Projection via .0, .1, etc.
let first pair : (Int, Int) -> Int = pair.0

-- Nested tuples
let triple : (Int, (Bool, String)) = (42, (1, "hello"))
```

### Sum Types (Algebraic Data Types) {#sums}

Sum types represent a choice between alternatives.  Declare them with `type`, using `|` to separate variants:

```iris
type Option = Some(Int) | None
type Color = Red | Green | Blue
type Result = Ok(Int) | Err(Int)
```

Variant payloads can be any type, including records and functions:

```iris
-- Inline anonymous record as payload
type Shape = Circle(Float) | Rect({ width: Float, height: Float }) | Empty

-- Or reference a named struct
type Dimensions = { width: Float, height: Float }
type Shape = Circle(Float) | Rect(Dimensions) | Empty

type Handler = OnEvent(Int -> Int) | Noop
```

Variants with payloads use parentheses; bare variants carry no data (Unit payload internally).  Constructors are automatically bound as functions:

```iris
let x = Some 42          -- Inject tag 0 with payload 42
let y = None              -- Inject tag 1 with Unit
let c = Green             -- Inject tag 1 (declaration order)

let s = Rect { width = 10.0, height = 5.0 }
let h = OnEvent (\x -> x + 1)
```

**Pattern matching** destructures sum values with named constructors:

```iris
let unwrap_or x default_val =
    match x with
      | Some(v) -> v            -- binds inner value to v
      | None -> default_val     -- bare constructor, no binding
```

The checker verifies **exhaustiveness**: every constructor of a `Sum` type must be covered, unless a wildcard `_` pattern is present.  Missing variants produce a compile error.

### Patterns in Practice {#patterns-in-practice}

ADTs replace sentinel values with types the compiler can check. Instead of returning `-1` for "not found" and hoping callers remember to check, use a sum type:

```iris
type Result = Found(Int) | NotFound

let lookup items key : List -> Int -> Result =
  fold NotFound (\found i ->
    if (list_nth items i) == key then Found i else found)
    (list_len items)

-- Callers must handle both cases -- forgetting NotFound is a compile error
let use_result items key =
  match (lookup items key) with
  | Found(idx) -> idx
  | NotFound -> 0
```

Multi-variant ADTs work well for operations that can fail in different ways:

```iris
type ParseResult = Ok(Int) | InvalidFormat | OutOfRange(Int)

let handle r = match r with
  | Ok(val)        -> val
  | InvalidFormat   -> 0
  | OutOfRange(val) -> val
```

### Struct Types (Record Types) {#struct-types}

Struct types give named fields to tuples. They are **syntactic sugar over product types**: the compiler translates record definitions and field accesses to positional tuple operations at compile time.

```iris
type Point = { x: Int, y: Int }

let origin : Point = { x = 0, y = 0 }  -- compiles to (0, 0)
let px = origin.x                       -- resolves to origin.0
```

**Field resolution:** `.x` on a `Point` resolves to `.0`, `.y` to `.1`, based on declaration order. The type system tracks field names during checking, but the runtime sees only tuples and positional projections.

```iris
type Color = { r: Int, g: Int, b: Int }
let red : Color = { r = 255, g = 0, b = 0 }
let g_val = red.g   -- resolves to red.1 → 0
```

Because structs are tuples, positional access still works: `origin.0` is equivalent to `origin.x`. Functions that accept tuples accept struct values transparently.

```iris
let add_points : Point -> Point -> Point = \a -> \b ->
  { x = a.x + b.x, y = a.y + b.y }
```

### Record Composition {#record-composition}

Records can be composed with `/` to merge fields from multiple record types:

```iris
type Contact = { email: String, phone: String }
type Address = { street: String, city: String }

-- Compose named record types
type User = { name: String } / Contact / Address
-- equivalent to { name: String, email: String, phone: String, street: String, city: String }

let u : User = { name = "Alice", email = "a@b", phone = "555", street = "Main", city = "NYC" }
let e = u.email
```

Both inline records and named types can appear on either side of `/`. Duplicate field names across composed records are a compile error:

```iris
type A = { x: Int }
type B = { x: Int }
type C = A / B   -- Error: duplicate field 'x' in record composition
```

### Recursive Types {#recursive}

```
μ X. F(X)
```

Recursive types use the fixed-point combinator.  In practice, recursive data (lists, trees) is encoded via `fold`/`unfold` over the graph representation.

---

## Polymorphism (System F) {#polymorphism}

**Parametric polymorphism** is supported via `forall`:

```iris
let identity x : forall a. a -> a = x
```

### How it works

The checker introduces and eliminates `ForAll` types:

- **Introduction** (`type_abst`, rule 18): If `body : T`, then `body : ∀X. T`
- **Elimination** (`type_app`, rule 19): If `f : ∀X. T`, then `f [S] : T[S/X]`

Soundness requires that the instantiation type is **well-formed**: it must exist in the type environment with all sub-types transitively present.

---

## Refinement Types {#refinements}

Refinement types constrain values with **Linear Integer Arithmetic (LIA)** predicates:

```iris
-- A positive integer
-- Syntax: {variable : BaseType | Predicate}
let check_positive x : {n: Int | n > 0} -> Bool = 1
```

### Supported predicates

| Predicate | Meaning |
|-----------|---------|
| `x == y`  | Equality |
| `x < y`, `x <= y` | Ordering |
| `x > y`, `x >= y` | Ordering (desugared to flipped `<` / `<=`) |
| `x != y`  | Disequality (`Not(Eq(...))`) |
| `p && q`  | Conjunction |
| `p \|\| q` | Disjunction |
| `!p`      | Negation |

### LIA terms

Predicates operate over linear integer arithmetic:

| Term | Meaning |
|------|---------|
| `42` | Integer constant |
| `x`  | Variable reference |
| `a + b` | Addition |
| `a - b` | Subtraction (desugared to `a + (-b)`) |
| `k * x` | Scalar multiplication (linear only) |
| `result` | Special variable for the function's return value |

The solver uses **property-based testing** with 1000 random inputs to check that `requires ⟹ ensures` holds.

---

## Contracts {#contracts}

Contracts are pre/post-conditions on functions, checked at compile time:

```iris
let safe_div a b : Int -> Int -> Int
  requires b != 0
  ensures result >= 0
  = if b == 0 then 0 else a / b
```

### Syntax

```
let name params : Type
  requires <predicate>
  requires <predicate>     -- multiple requires allowed
  ensures <predicate>
  ensures <predicate>      -- multiple ensures allowed
  = body
```

### How verification works

1. The lowerer converts `requires` and `ensures` expressions to `LIAFormula` values
2. At compile time, `verify_contracts()` runs the LIA solver
3. The solver generates 1000 random assignments for all bound variables
4. For each assignment: if `requires` is satisfied, `ensures` must also hold
5. If a counterexample is found, compilation fails with the violating assignment

### Example: bounded addition

```iris
let bounded_add a b : Int -> Int -> Int
  requires a >= 0
  requires b >= 0
  requires a + b <= 1000
  ensures result >= 0
  ensures result <= 1000
  = a + b
```

---

## Cost Annotations {#costs}

Every function can declare its asymptotic complexity:

```iris
let lookup key : Int -> Int [cost: Const(1)]     = ...
let sum xs : Int -> Int [cost: Linear(xs)]        = ...
let sort xs : Int -> Int [cost: NLogN(xs)]        = ...
let matrix_mul m : Int -> Int [cost: Polynomial(m, 3)] = ...
```

### Cost bounds

| Annotation | Complexity | Description |
|-----------|-----------|-------------|
| `Zero` | O(0) | Instantaneous (literal, projection) |
| `Const(k)` | O(1) | Constant with bound `k` |
| `Linear(v)` | O(n) | Linear in variable `v` |
| `NLogN(v)` | O(n log n) | Linearithmic |
| `Polynomial(v, d)` | O(n^d) | Polynomial of degree `d` |
| `Unknown` | ? | No cost analysis (default) |

### Cost lattice

Costs form a **partial order** used by the checker:

```
Zero ≤ Const(k) ≤ Linear(v) ≤ NLogN(v) ≤ Polynomial(v, d)
```

Composite costs combine via:
- `Sum(k₁, k₂)`: sequential composition
- `Sup(k₁, k₂)`: branching (max of branches)
- `Mul(k₁, k₂)`: iteration (loop body × iterations)

The kernel's **cost subsumption** rule (rule 9) allows weakening: if you prove `f : T @ Linear(n)`, you can weaken to `f : T @ Polynomial(n, 2)`.

### Cost of kernel rules

| Rule | Cost produced |
|------|--------------|
| Literal | `Zero` |
| Primitive op | `Const(1)` |
| Lambda intro | `Zero` (binding is free) |
| Application | `Sum(k_arg, k_fn, k_body)` |
| Let binding | `Sum(k_bound, k_body)` |
| Guard (if/else) | `Sum(k_pred, Sup(k_then, k_else))` |
| Fold | `Sum(k_input, k_base, Mul(k_step, k_input))` |
| Match | `Sum(k_scrutinee, Sup(k_arms...))` |

Note: `k_input` in the fold rule is the cost of **evaluating the input expression**, not the runtime element count. A fold over a bare variable has `k_input = Zero` because variables are free to evaluate. This means the kernel's cost model tracks expression structure, not data-dependent complexity. Overestimated annotations (e.g., `[cost: Linear(n)]` on a fold whose proven cost is near-Zero) are accepted because `proven <= declared` holds.

---

## Effect Typing {#effects}

Side effects are tracked through **effect sets**.  Every function's actual effects are collected from its `Effect` nodes and can be verified against a declared set.

### Effect categories

| Category | Tags | Examples |
|----------|------|---------|
| **I/O** | 0x00–0x0D | `Print`, `ReadLine`, `FileRead`, `FileWrite`, `HttpGet` |
| **Raw I/O** | 0x10–0x1F | `TcpConnect`, `TcpRead`, `FileOpen`, `EnvGet`, `SleepMs` |
| **Threading** | 0x20–0x28 | `ThreadSpawn`, `ThreadJoin`, `AtomicRead`, `RwLockWrite` |
| **JIT/FFI** | 0x29–0x2B | `MmapExec`, `CallNative`, `FfiCall` |
| **User-defined** | 0x2C–0xFF | Custom effects |

### Pure functions

A function with no `Effect` nodes has the empty effect set; it is **pure**.  The checker reports effect information at compile time:

```
Fragment `sort`: pure (no effects)
Fragment `read_file`: effects = {FileOpen, FileReadBytes, FileClose}
```

### Capability restriction

Effects can be restricted using `allow`/`deny` annotations.  See the [capability-based security](/learn/language/#capabilities) section.

---

## The Proof Kernel {#kernel}

Type checking is driven by an **LCF-style proof kernel** with 20 inference rules.  The kernel is the only code that can construct `Theorem` values. The checker (outside the trusted base) calls kernel methods and assembles proof trees.

### The 20 rules

| # | Rule | Judgment |
|---|------|---------|
| 1 | `assume` | Γ, P ⊢ P |
| 2 | `intro` | Γ, x:A ⊢ e:B ⟹ Γ ⊢ λx.e : A→B |
| 3 | `elim` | Γ ⊢ f:A→B, Γ ⊢ a:A ⟹ Γ ⊢ f a : B |
| 4 | `refl` | e = e |
| 5 | `symm` | a = b ⟹ b = a |
| 6 | `trans` | a = b, b = c ⟹ a = c |
| 7 | `congr` | f:T, a:T ⟹ f a : T |
| 8 | `type_check_node` | Leaf node → theorem from annotation |
| 9 | `cost_subsume` | e:T@k₁, k₁≤k₂ ⟹ e:T@k₂ |
| 10 | `cost_leq_rule` | Witness that k₁ ≤ k₂ |
| 11 | `refine_intro` | e:T, P(e) holds ⟹ e:{x:T \| P(x)} |
| 12 | `refine_elim` | e:{x:T \| P(x)} ⟹ e:T |
| 13 | `nat_ind` | P(0), ∀n.P(n)→P(n+1) ⟹ ∀n.P(n) |
| 14 | `structural_ind` | One proof per constructor ⟹ all |
| 15 | `let_bind` | Γ ⊢ e₁:A, Γ,x:A ⊢ e₂:B ⟹ let x=e₁ in e₂ : B |
| 16 | `match_elim` | scrutinee:Sum, all arms:T ⟹ match:T |
| 17 | `fold_rule` | base:A, step:A→B→A, input ⟹ fold:A |
| 18 | `type_abst` | e:T ⟹ e:∀X.T |
| 19 | `type_app` | e:∀X.T ⟹ e:T[S/X] |
| 20 | `guard_rule` | pred:Bool, then:T, else:T ⟹ if:T |

Every rule produces a **BLAKE3 proof hash**, an audit trail that can be replayed.

### Gradual typing

The graded checker uses a **trust-annotation fallback**: when a structural rule cannot fire (e.g., a child node isn't proven yet), the checker trusts the node's type annotation and produces a theorem tagged `"trust"`.  This keeps the system progressive: partially annotated code still type-checks, and adding annotations strictly increases the strength of guarantees.

---

## Verification Tiers {#tiers}

The checker automatically classifies each function into a verification tier based on the node kinds in its graph:

### Tier 0: Decidable core

**Node kinds**: Lit, Prim, Tuple, Inject, Project, Ref, Apply, Lambda, Let, Guard, Match

**What's checked**:
- Type annotations match actual types
- Function application is well-typed (argument matches parameter)
- Let bindings propagate types correctly
- Pattern matching covers scrutinee type

### Tier 1: Induction and polymorphism

**Additional node kinds**: Fold, Unfold, LetRec, TypeAbst, TypeApp

**What's checked** (in addition to Tier 0):
- Fold/unfold are well-typed catamorphisms/anamorphisms
- Recursive functions (let rec) have consistent types
- Polymorphic abstraction/application are sound

### Tier 2: Effects and modules

**Additional node kinds**: Effect, Extern

**What's checked** (in addition to Tier 1):
- Effect sets are subsets of declared effects
- Cost annotations are enforced (violations become hard errors)
- Contract verification (requires ⟹ ensures)

### Automatic classification

```iris
-- Tier 0: no fold, no recursion, no effects
let add x y : Int -> Int -> Int [cost: Const(1)] = x + y

-- Tier 1: uses fold
let sum xs : Int -> Int [cost: Linear(xs)] =
  fold 0 (\acc x -> acc + x) xs

-- Tier 2: uses effects
let greet name : String -> Int [cost: Const(1)] =
  print (str_concat "Hello, " name)
```

---

## Exhaustive Pattern Matching {#exhaustiveness}

When matching on a `Sum` type, the checker verifies that **every constructor** is covered:

```iris
-- Given: type Color = Red | Green | Blue (tags 0, 1, 2)

-- ✓ Exhaustive: all tags covered
match color
  case 0 -> "red"
  case 1 -> "green"
  case 2 -> "blue"

-- ✓ Exhaustive: wildcard covers the rest
match color
  case 0 -> "red"
  case _ -> "other"

-- ✗ Non-exhaustive: missing tag 2
match color
  case 0 -> "red"
  case 1 -> "green"
  -- Error: non-exhaustive patterns -- missing tags: [2]
```

For `Bool` values, the checker expects tags 0 and 1 to be covered.

### Match in Higher-Order Contexts {#match-lambda}

Match expressions work correctly inside lambda bodies, including callbacks passed to `fold`.  This enables type-safe ADT processing in higher-order contexts:

```iris
type DispatchResult = DispatchOk(Int) | EffectUnsupported | EffectDenied | DispatchErr(Int)

let count_successes results n : Tuple -> Int -> Int =
  fold 0
    (\acc _i ->
      if _i >= n then acc
      else
        match (list_nth results _i) with
        | DispatchOk(_) -> acc + 1
        | EffectUnsupported -> acc
        | EffectDenied -> acc
        | DispatchErr(_) -> acc)
    n
```

The checker verifies exhaustiveness for the match inside the lambda just as it would at the top level, so missing a variant like `DispatchErr(_)` is still a compile error.

---

## Type Inference {#inference}

**Bottom-up type inference** runs after lowering:

1. **Literal nodes** get their type from the value (`42` → `Int`, `"hello"` → `String`)
2. **Primitive operations** get types from their opcode signature
3. **Tuple constructors** get `Product` types from their children
4. **Apply nodes** propagate: if `f : A → B` and `arg : A`, then `f arg : B`
5. **Guard nodes** (if/else) get the type of their `then` branch
6. **Let nodes** get the type of their body
7. **Lambda nodes** get `Arrow(param_type, body_type, cost)`
8. **Fold nodes** get the type of their base case

### What's inferred vs. what needs annotation

| Construct | Annotation needed? |
|-----------|-------------------|
| Integer literals | No (always `Int`) |
| String literals | No (always `String`/`Bytes`) |
| Boolean expressions | No (always `Bool`) |
| Simple let bindings | No (inferred from value) |
| Function parameters | **Recommended** (defaults to `Int`) |
| Function return types | **Recommended** (inferred but explicit is clearer) |
| Cost bounds | Optional (defaults to `Unknown`) |
| Contracts | Optional (no contracts means no verification) |
| Polymorphic functions | **Required** (must annotate `forall`) |

---

## Full Example {#full-example}

Putting it all together: a verified, cost-annotated function with contracts:

```iris
-- Absolute value with full type safety annotations
let int_abs x : Int -> Int [cost: Const(1)]
  requires x > -1000000
  ensures result >= 0
  = if x >= 0 then x else 0 - x

-- Binary search with logarithmic cost
let rec binary_search arr target lo hi
  : Int -> Int -> Int -> Int -> Int [cost: NLogN(arr)]
  = if lo > hi then 0 - 1
    else
      let mid = lo + (hi - lo) / 2 in
      let mid_val = list_nth arr mid in
      if mid_val == target then mid
      else if mid_val < target
        then binary_search arr target (mid + 1) hi
        else binary_search arr target lo (mid - 1)

-- Population fitness evaluation with linear cost and effects
let evaluate_population pop test_cases
  : Int -> Int -> Int [cost: Linear(pop)]
  = fold 0 (\total individual ->
      let score = fold 0 (\passed tc ->
        let result = graph_eval individual tc.0 in
        if result == tc.1 then passed + 1 else passed)
        test_cases
      in total + score)
    pop
```

---

## Design Principles {#design}

1. **Gradual**: No annotation required to run code.  Add annotations for more guarantees.
2. **Sound kernel**: The 20-rule LCF kernel is the only code that constructs proofs.  Everything else is untrusted.
3. **Content-addressed**: Types are identified by BLAKE3 hash.  Two structurally identical types are the same type.
4. **Cost-aware**: Every arrow carries a cost bound.  The kernel tracks cost through every rule.
5. **Effect-tracked**: Side effects are first-class.  Pure functions have empty effect sets.
6. **Proof-auditable**: Every theorem carries a BLAKE3 proof hash that can be replayed for verification.
