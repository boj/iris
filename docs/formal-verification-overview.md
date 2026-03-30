# IRIS Formal Verification System

## Overview

IRIS has three complementary verification systems:

1. **LCF Proof Kernel**: proves type safety and cost bounds
2. **BLAKE3 Merkle Audit Chain**: proves what happened (tamper-evident modification history)
3. **ZK Proofs**: proves "I verified this program" without revealing the program itself

## Lean 4 Proof Kernel

### Architecture

The kernel is written in Lean 4 (`lean/IrisKernel/`) and runs as an IPC subprocess. All 20 inference rules execute in Lean — the running code is the formal proof. The Rust side wraps results in opaque `Theorem` values (`pub(crate)` fields prevent forgery) with BLAKE3 proof hashes for audit trails. The LCF invariant holds: **`Theorem` values can only be constructed by the kernel bridge**, and the bridge only constructs them from Lean-validated judgments.

Each theorem proves a judgment: `Γ ⊢ e : τ @ κ`

- `Γ`: the typing context (what variables are in scope and their types)
- `e`: a node in the SemanticGraph (identified by NodeId)
- `τ`: the type (TypeId referencing a TypeDef)
- `κ`: the cost bound (CostBound: O(1), O(n), O(n log n), O(n²), Unknown, etc.)

This is CaCIC (Cost-aware Calculus of Inductive Constructions). Types encode both correctness AND computational cost.

### The 20 Inference Rules

| Rule | What it proves |
|------|---------------|
| `assume` | Introduce a hypothesis into the context |
| `intro` | Lambda introduction (if body is well-typed, lambda is too) |
| `elim` | Function application (well-typed function applied to well-typed arg) |
| `refl` | A node has its declared type (reflexivity) |
| `symm` | Type equivalence is symmetric |
| `trans` | Type equivalences compose (transitivity) |
| `congr` | If children are well-typed, parent is too (congruence) |
| `type_check_node` | Full node verification (dispatches on NodeKind) |
| `cost_subsume` | A cheaper cost bound satisfies an expensive one |
| `cost_leq_rule` | Cost ordering (O(1) ≤ O(n) ≤ O(n²)) |
| `refine_intro` | Introduce refinement type `{x : T \| φ}` |
| `refine_elim` | Use a refinement type (extract the predicate) |
| `nat_ind` | Natural number induction |
| `structural_ind` | Structural induction on algebraic data types |
| `let_bind` | Let binding (bind value, check body) |
| `match_elim` | Pattern match exhaustiveness |
| `fold_rule` | Fold with well-typed step function and base is well-typed |
| `type_abst` | Polymorphic abstraction (∀α. T) |
| `type_app` | Polymorphic instantiation (T[α := S]) |
| `guard_rule` | Conditional guard (predicate + body + fallback all well-typed) |

### Verification Tiers

Not all programs need the same level of verification:

| Tier | Node Kinds Allowed | Use Case |
|------|-------------------|----------|
| Tier 0 | Lit, Prim, Lambda, Let, Tuple, Apply, Match, Guard, Project, Inject | Simple arithmetic, basic control flow |
| Tier 1 | + Fold, Unfold, LetRec, TypeAbst | Iteration, recursion, polymorphism |
| Tier 2 | + Effect, Extern | I/O, FFI calls |
| Tier 3 | + Neural | Machine learning nodes |

The `iris check` command auto-detects the minimum tier required for each definition.

### How the Checker Works

The checker walks a SemanticGraph bottom-up (topological order), applying kernel rules to each node:

1. Leaf nodes (Lit) get `refl`: trivially well-typed
2. Prim nodes get `congr`: if all arguments are well-typed, the primitive is too
3. Fold nodes get `fold_rule`: requires well-typed base, step function, and collection
4. Guard nodes get `guard_rule`: predicate, body, and fallback must all type-check

The output is either:
- `Ok((ProofTree, Theorem))`: complete proof, the program is verified
- `VerificationReport`: partial credit (e.g., "124/124 obligations satisfied, score 1.00")

### Usage

```bash
$ iris check src/iris-programs/interpreter/full_interpreter.iris
[OK] full_interpret: 124/124 obligations satisfied (score: 1.00)
All 1 definitions verified.
```

## BLAKE3 Merkle Audit Chain

### Purpose

The audit chain records every self-modification the system makes. It proves **what happened**, not correctness, but history. This is what auditors in finance, healthcare, and defense want.

### Structure

Each entry in the chain:

```
AuditEntry {
    id:                 0                    // monotonically increasing
    timestamp:          1711180800           // when it happened
    action:             ComponentDeployed { name: "replace_prim", slowdown: 1.3 }
    before_hash:        173cade8a4fd...      // BLAKE3 of system state before
    after_hash:         a7a0f0459cc2...      // BLAKE3 of system state after
    proof:              Some(ProofReceipt)   // formal verification result
    performance_delta:  +0.23                // positive = improvement
    entry_hash:         1424cb68326a...      // BLAKE3(all fields except this one)
    prev_hash:          0000000000000000     // genesis (or previous entry's hash)
}
```

### Chain Integrity

- Each entry's `entry_hash` = `BLAKE3(id, timestamp, action, before_hash, after_hash, proof, performance_delta, prev_hash)`
- Each entry's `prev_hash` = previous entry's `entry_hash`
- The Merkle root covers all entries via pairwise BLAKE3 hashing
- `verify_chain()` recomputes every hash and checks every link
- **Modifying any field in any entry breaks the chain**

### Example

```
Entry #0: Deploy replace_prim (1.3x slowdown, +0.23 improvement)
  prev_hash:  00000000...  (genesis)
  entry_hash: 1424cb68...

Entry #1: Deploy insert_node (1.4x slowdown, +0.15 improvement)
  prev_hash:  1424cb68...  (chains to entry #0)
  entry_hash: 56663cf5...

Entry #2: Rollback insert_node (p99 regression detected)
  prev_hash:  56663cf5...  (chains to entry #1)
  entry_hash: 857b2635...
  after_hash: a7a0f045...  (matches entry #0's after_hash, rolled back!)

Merkle root: bc22e718d9aad1c3dda231a3c97aefe8a439710536109477135007520cd12138
Tamper test: modifying any entry → verify_chain() returns false
```

## ZK Proofs (Fiat-Shamir Sigma Protocol)

For programs shared with untrusted parties, the verification can be wrapped in a zero-knowledge proof. This proves "I verified this program and it passed" without revealing the program's structure.

- Uses BLAKE3 as the random oracle (Fiat-Shamir heuristic)
- Merkle tree commitments over program nodes
- Challenge-response protocol
- ProofReceipt can be attached to audit entries

## What This Does NOT Cover

### Functional correctness

The kernel proves type safety (`Int -> Int` function won't return a `Bool`) and cost bounds (`O(n)` function won't take `O(n^2)` time).

Functional correctness is now partially addressed by three mechanisms:

1. **Refinement type checking** -- The checker verifies refinement predicates `{x : Int | phi(x)}` on Lit and Prim nodes using the LIA solver. For Lit nodes, it evaluates the predicate on the concrete value. For Prim nodes, it verifies the predicate is satisfiable.

2. **Contract annotations** -- The `.iris` surface syntax supports `requires` (preconditions) and `ensures` (postconditions) on `let` declarations:
   ```iris
   let abs x : Int -> Int
     requires x >= -1000000 && x <= 1000000
     ensures result >= 0
     = if x >= 0 then x else 0 - x
   ```
   Contracts are stored on `Fragment` and can be verified via the LIA solver or property testing.

3. **Property-based testing** -- The `property_test` module provides probabilistic verification: generate random inputs and check that a property (LIA formula) holds for all of them. Not a proof, but high confidence. Functions include:
   - `property_test(property, num_tests)` -- test with custom ranges
   - `quick_check(formula, vars)` -- 1000 tests with default ranges
   - `verify_contract(requires, ensures, vars, num_tests)` -- check `requires => ensures`

4. **Extended LIA solver** -- The LIA solver now handles:
   - Absolute value: `lia_abs(t)` rewritten as `if t >= 0 then t else -t`
   - Min/max: `lia_min(a,b)`, `lia_max(a,b)` using conditional terms
   - IfThenElse terms: `LIATerm::IfThenElse(cond, then, else)` for conditional arithmetic
   - Modulo: `LIATerm::Mod(a, b)` for divisibility and remainder constraints
   - Multiplication by constants (already supported via `LIATerm::Mul`)

### Termination

- Fold with a decreasing measure is verified (structural recursion)
- General recursion (LetRec) is NOT proven terminating
- The interpreter has step limits (timeout) as a safety net

### Side effect correctness

- Effect nodes are type-checked (correct input/output types)
- The kernel does NOT verify I/O behavior (e.g., "this TCP write sends the right data")

## Future Work

1. **Full SMT integration** -- plug in Z3/CVC5 for predicates the bounded LIA solver cannot handle
2. **Dependent types** -- types that depend on runtime values
3. **Automatic contract inference** -- infer requires/ensures from test cases
4. **Formal kernel metatheory** -- prove the 20 rules consistent in Coq or Lean
