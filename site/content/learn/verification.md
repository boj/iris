---
title: "Verification"
description: "How IRIS verifies programs: proof kernel, refinement types, cost analysis, and Lean formalization."
weight: 60
---

IRIS includes a verification system that lets programs carry machine-checked proofs of correctness. Verification is optional -- programs work without proofs -- but the infrastructure exists to verify properties ranging from type safety to cost bounds.

This page explains what verification means in IRIS, how it works, what it can prove, and where the boundaries are.

## Why Verify? {#why}

Programs can modify themselves. Evolution breeds new code. The `--improve` flag hot-swaps faster implementations at runtime. In a system where code changes autonomously, you need a way to know that changes are safe.

The verification system serves three purposes:

1. **Safety gate for evolution** -- the daemon won't deploy a replacement that fails verification
2. **Gradient for search** -- graded verification scores let evolution follow a gradient toward provable programs, not just binary pass/fail
3. **Trust boundary** -- the proof kernel is the only component that cannot be replaced by evolved code

## The Proof Kernel {#kernel}

The kernel is the trusted computing base, written in **Lean 4** and running as a separate IPC subprocess (`iris-kernel-server`). All 20 inference rules execute in Lean — the running code is the formal proof. Lean's type system guarantees that every judgment was derived by a valid sequence of rule applications.

The Rust side wraps Lean's results in opaque `Theorem` values (with BLAKE3 proof hashes for audit trails) but never evaluates inference rules itself. The only code that can construct `Theorem` values lives in the kernel bridge. No external code can forge a proof.

### Why Lean? {#why-lean}

In a self-improving system, the kernel is the one component that must never be wrong — everything else (mutations, evolutions, deployments) is gated through it. Rust gives memory safety but not logical correctness. Lean 4 is both a proof assistant and a compiled language: the inference rules are executable functions with machine-checked correspondence proofs linking them to their formal specification. If it compiles, the proofs hold.

### Properties {#kernel-properties}

- 20 inference rules implemented in Lean 4 with correspondence proofs
- Runs as a native subprocess via stdin/stdout IPC (~microsecond latency per rule)
- Every theorem records: context, node, type, cost bound, and a cryptographic hash (BLAKE3) of the derivation chain
- No `unsafe` Rust in the kernel path — communication is via `std::process` and `std::io`

### The 20 Rules {#rules}

The rules form a sound fragment of System F with refinements, cost annotations, and structural recursion.

**Core lambda calculus (rules 1--7, 15):**

| Rule | Name | What it proves |
|------|------|----------------|
| 1 | `assume` | Variable lookup: if x:A is in the context, then x has type A |
| 2 | `intro` | Lambda introduction: given a proof of the body, produce a function type |
| 3 | `elim` | Application: given f:A->B and a:A, produce f(a):B |
| 4 | `refl` | Reflexivity: any node equals itself |
| 5 | `symm` | Symmetry: if a = b then b = a |
| 6 | `trans` | Transitivity: if a = b and b = c then a = c |
| 7 | `congr` | Congruence: if f = g and a = b then f(a) = g(b) |
| 15 | `let_bind` | Let binding: compose the costs of the bound expression and body |

**Pattern matching and control flow (rules 16, 20):**

| Rule | Name | What it proves |
|------|------|----------------|
| 16 | `match_elim` | All match arms agree on type; cost is scrutinee + max(arm costs) |
| 20 | `guard_rule` | Conditional: both branches must agree on type |

**Induction (rules 13--14, 17):**

| Rule | Name | What it proves |
|------|------|----------------|
| 13 | `nat_ind` | Natural number induction: P(0) and P(n)->P(n+1) implies P for all n |
| 14 | `structural_ind` | Structural induction: one case per constructor proves the property for all values |
| 17 | `fold_rule` | Catamorphism: cost is input + base + (step cost * input expression cost) |

**Polymorphism (rules 18--19):**

| Rule | Name | What it proves |
|------|------|----------------|
| 18 | `type_abst` | ForAll introduction (System F) |
| 19 | `type_app` | ForAll elimination with well-formedness check on substituted types |

**Cost and refinement (rules 8--12):**

| Rule | Name | What it proves |
|------|------|----------------|
| 8 | `type_check_node` | Trust a node's content-addressed type annotation |
| 9 | `cost_subsume` | Weaken a cost bound: if actual cost <= declared cost, accept |
| 10 | `cost_leq_rule` | Produce a witness that one cost bound is <= another |
| 11 | `refine_intro` | If e:T and P(e) holds, then e:{x:T\|P} |
| 12 | `refine_elim` | If e:{x:T\|P}, extract both e:T and the predicate P |

### Why LCF? {#lcf}

The LCF architecture means the kernel is the single point of trust. The checker, the evolution engine, and user code all produce proofs by calling kernel rules -- they cannot bypass them. If the 20 rules are sound, then every `Theorem` value in the system is sound, regardless of what untrusted code produced it.

## Refinement Types {#refinement-types}

The proof kernel supports refinement types via `refine_intro` and `refine_elim` rules, and the LIA (Linear Integer Arithmetic) solver can check predicates. However, refinement type annotations in `.iris` surface syntax are parsed but **not lowered** to the SemanticGraph -- the lowerer discards them.

Refinement types work at the kernel API level when proofs are constructed programmatically (e.g., by the checker or evolution engine), but not from surface syntax today.

```iris
-- Parsed but not lowered -- serves as documentation
let safe_div x y : Int -> {y : Int | y != 0} -> Int = x / y
```

The kernel's `refine_intro` rule requires an independent proof that the predicate holds -- you can't introduce a refinement without evidence. `refine_elim` extracts the base type, discarding the predicate. This prevents circular reasoning.

### Supported Predicates {#predicates}

The LIA solver handles:

- **Comparisons**: `x = y`, `x < y`, `x <= y`, `x != y`
- **Arithmetic**: constants, variables, addition, scalar multiplication, negation, modulo
- **Logic**: and, or, not, implies
- **Divisibility**: `x % d = 0`

When the solver can't decide a predicate, it falls back to property-based testing (random sampling with counterexample generation).

## Example Programs {#examples}

IRIS ships with annotated programs in `examples/`. The `requires`/`ensures` annotations are lowered to LIA formulas and verified at compile time by the LIA solver. They do not affect runtime execution (no runtime contract checking), but the compiler reports violations as errors.

What `iris check` actually verifies is the **graph structure**: node types agree across edges, match arms are consistent, guards have compatible branches, cost annotations are consistent with the kernel's proven costs, and `requires`/`ensures` contracts are verified via property-based testing (LIA solver + random sampling).

### Absolute value {#example-abs}

```iris
let abs x : Int -> Int
  requires x >= -1000000 && x <= 1000000
  ensures result >= 0
  = if x >= 0 then x else 0 - x
```

The checker verifies that both guard branches produce values of the same type, that the graph is well-formed, and tests the `requires`/`ensures` contracts via the LIA solver.

### Safe division {#example-div}

```iris
let safe_div x y : Int -> Int -> Int
  requires y != 0
  = x / y
```

The `requires y != 0` is verified by the LIA solver at compile time. At runtime, division by zero will still error if unchecked code calls `safe_div` with `y = 0`.

### Bounded addition {#example-add}

```iris
let bounded_add x y : Int -> Int -> Int
  requires x >= -500000 && x <= 500000
  requires y >= -500000 && y <= 500000
  ensures result >= -1000000
  ensures result <= 1000000
  ensures result == x + y
  = x + y
```

### Clamping {#example-clamp}

```iris
let clamp x lo hi : Int -> Int -> Int -> Int
  requires lo <= hi
  ensures result >= lo
  ensures result <= hi
  = if x < lo then lo else if x > hi then hi else x
```

## The Checker {#checker}

The checker is untrusted code that drives the kernel. It walks a program's graph bottom-up, applying kernel rules at each node, and collects the results.  Two checkers exist: the **strict checker** (binary pass/fail for proofs) and the **graded checker** (partial credit for evolution).

### How `compile_checked` works {#compile-checked}

Every `.iris` file goes through this pipeline:

1. **Parse** the source into AST (tokenizer → parser)
2. **Lower** to SemanticGraph (type expressions → TypeDefs, contracts → LIA formulas)
3. **Classify tier**: scan node kinds to determine Tier 0/1/2
4. **Propagate contexts**: top-down BFS assigns typing contexts to each node
5. **Type-check**: bottom-up topo-sort, apply kernel rules at each node
6. **Verify contracts**: run LIA solver on `requires`/`ensures` clauses
7. **Collect effects**: scan for Effect nodes, report effect sets

All standard library programs pass this pipeline with zero errors.

### Graded Verification {#graded}

Instead of binary pass/fail, the checker computes a score in [0.0, 1.0]. This is critical for evolution, since a program that satisfies 8/10 obligations is better than one that satisfies 2/10, even though both "fail."

The checker auto-classifies programs into tiers:

| Tier | Node kinds | What's checked |
|------|-----------|----------------|
| Tier 0 | Lit, Prim, Tuple, Apply, Lambda, Let, Guard, Match | Types, application, let bindings, patterns |
| Tier 1 | + Fold, Unfold, LetRec, TypeAbst, TypeApp | + Induction, recursion, polymorphism |
| Tier 2 | + Effect, Extern | + Effect verification, cost enforcement, contracts |

### Gradual Typing {#gradual}

The checker uses a **trust-annotation fallback**: when a structural rule can't fire (e.g., child nodes aren't proven), it trusts the node's type annotation.  This means:

- **Unannotated code compiles**: the checker trusts all annotations
- **Partial annotations work**: annotated nodes get full checking, others get trust
- **Full annotations get full proofs**: every node proven by kernel rules

Proof trees tag trusted nodes with `"trust"` so audits can distinguish fully proven from trusted theorems.

### Proof-Guided Mutation {#proof-guided}

When verification fails, the checker produces a `ProofFailureDiagnosis` with a `MutationHint`:

- `AddTerminationCheck`: missing loop termination
- `FixTypeSignature(expected, actual)`: type mismatch
- `AddCostAnnotation`: node needs a cost bound
- `WrapInGuard`: add a runtime guard (e.g., division-by-zero check)

The evolution engine uses these hints to guide mutations toward provable programs. Instead of random search, the system knows *what's wrong* and *how to fix it*.

## Cost Analysis {#cost}

Cost annotations declare the computational complexity of a function:

```iris
let sum xs : List Int -> Int [cost: Linear(xs)] = fold 0 (+) xs
```

The cost lattice forms a partial order:

```
Zero < Const(k) < Linear(n) < NLogN(n) < Polynomial(n, d)
```

Composite forms handle sequential (`Sum`), parallel (`Par`), repeated (`Mul`), and branching (`Sup` for max, `Inf` for min) costs.

The kernel's `cost_subsume` rule (rule 9) weakens cost bounds: if the proven cost is `Linear(n)`, the checker accepts a declared bound of `Polynomial(n, 2)` because `Linear(n) <= Polynomial(n, 2)` in the cost lattice. Overestimating is always safe; underestimating triggers a warning (Tier 0/1) or a hard error (Tier 2+).

The checker verifies cost annotations at two levels: **per-node** (each node's annotated cost is compared against the cost derived by its kernel rule) and **graph-level** (the function's declared `[cost: ...]` is compared against the root theorem's proven cost). At Tier 0 and Tier 1, a mismatch produces a `CostWarning`. At Tier 2+, it becomes a `CheckError` that fails compilation.

Note that the kernel's cost model tracks **expression evaluation cost**, not runtime data-dependent complexity. The `fold_rule` computes `Mul(k_step, k_input)` where `k_input` is the cost of *evaluating* the input expression, not the number of elements at runtime. A fold over a bare variable `n` has near-Zero proven cost because `n` costs Zero to evaluate. Declaring `[cost: Linear(n)]` on such a fold is accepted because the annotation overestimates the proven cost. True O(n) tracking would require a separate size analysis pass.

## Lean Formalization {#lean}

The kernel implementation (`lean/IrisKernel/Kernel.lean`) has executable functions for all 20 rules. These are proven to correspond to an inductive `Derivation` type (`Rules.lean`) that serves as the formal specification. The Lean code is both the running kernel and the formal proof.

### What's proven {#lean-done}

- All 20 inference rules as constructors of an inductive `Derivation` type (specification)
- Executable kernel functions with correspondence proofs (implementation = specification)
- The cost lattice partial order (`CostLeq`) with reflexivity, transitivity, and zero/unknown axioms
- Type well-formedness: a type is well-formed if it and all its references exist
- Key metatheorems: reflexivity produces valid judgments, cost weakening is transitive, Zero is bottom, Unknown is top

### What's not proven yet {#lean-todo}

- Full weakening lemma (context extension preserves derivations) -- partially proven, needs structural induction over all derivation forms
- Some cost algebra embedding rules (e.g., `a <= Sum(a, b)`)
- Correspondence proofs for 14 of 20 rules (6 complete: assume, refl, cost_subsume, cost_leq_rule, trans, guard_rule)

### What "proven" means {#lean-scope}

The Lean code IS the running kernel, not a separate model. When a correspondence proof says "if the executable function returns `some j`, then there exists a `Derivation` matching `j`", that's a statement about the actual code that processes your program's types. There is no gap between the proof and the implementation.

## Soundness Argument {#soundness}

The 20 rules are sound because they correspond to well-understood type-theoretic constructs:

1. **Rules 1--7, 15** = simply-typed lambda calculus (known sound since 1940s)
2. **Rules 18--19** = System F polymorphism (sound, with well-formedness guard on substitutions)
3. **Rules 11--12** = refinement types / liquid types (sound: intro requires independent evidence)
4. **Rules 13--14** = standard Peano and structural induction (sound, requires exhaustive cases)
5. **Rules 9--10, 17** = cost annotations in a separate dimension (don't affect type soundness)
6. **Rule 8** = trusts content-addressed node annotations (conservative)
7. **Rules 16, 20** = standard sum elimination and conditional (sound: all branches agree on type)

The LCF architecture ensures that even if untrusted code (the checker, evolution, user programs) tries to construct invalid proofs, the kernel rejects them.

## What Verification Does NOT Do {#limitations}

To be clear about the boundaries:

- **Verification is gradual.** Unannotated code still compiles and runs; the checker trusts annotations when structural rules can't fire. Adding annotations strictly increases guarantees.
- **Contract verification is probabilistic.** The LIA solver uses 1000 random inputs to check `requires ⟹ ensures`. This provides high confidence, not mathematical certainty.
- **Correspondence proofs are incomplete.** 6 of 20 rules have formal correspondence proofs linking the executable function to the `Derivation` spec. The remaining 14 are tested (86 cross-validation tests) but not yet formally proven in Lean.
- **Cost analysis tracks expression structure, not runtime behavior.** The kernel propagates costs through every rule, but cost bounds reflect the cost of evaluating the expression tree, not data-dependent runtime complexity. A fold over a variable has near-Zero proven cost because the variable itself is free to evaluate.

### What IS enforced

Since the 3-tier type system was implemented, all IRIS programs in the standard distribution pass `compile_checked`, the mandatory type-check that runs at compile time. This includes:

- **Type annotations** are checked against actual types (not just documentation)
- **Contracts** (`requires`/`ensures`) are verified via property-based testing
- **Effect sets** are collected and can be verified against declared bounds
- **Cost bounds** are enforced at Tier 2+ (violations are hard errors)
- **Pattern exhaustiveness** is checked for Sum types

See the [Type System](/learn/type-system/) page for the complete specification.

## Turing Completeness and the Halting Problem {#turing}

IRIS is Turing complete. `let rec` provides unbounded general recursion, `if/then/else` provides branching, and dynamic tuples provide unbounded data. There is no mandatory termination checker, so any computable function can be expressed.

**Step limits are sandbox policy, not language semantics.** The optional step counter (`max_steps`) kills runaway execution in sandboxed contexts, but this is a runtime safety net, not a theoretical restriction. Remove it and programs can diverge forever.

IRIS takes a **stratified approach** to the halting problem rather than trying to solve it:

| Tier | Mechanism | Guarantee |
|------|-----------|-----------|
| **Provably terminating** | Cost bound (`Linear(n)`) + decrease witness (`Structural`/`Sized`/`WellFounded`) | The proof kernel verifies termination claims. If the kernel accepts, the function terminates. |
| **Empirically terminating** | Step limit + evolution | The sandbox kills programs that exceed step budgets. Evolution breeds for termination within bounds. No proof, but practical confidence. |
| **Diverging** | Detected and killed | Programs that exceed all limits are killed by the sandbox. The system stays safe even when individual programs don't terminate. |

The proof kernel **does not solve the halting problem**; it verifies specific termination claims using structural or well-founded induction. This is the same approach as Coq, Agda, and Lean (termination checking on opt-in claims), except IRIS allows unchecked programs to run too.
