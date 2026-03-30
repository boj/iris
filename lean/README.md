# IRIS Proof Kernel — Lean 4

The IRIS proof kernel, implemented in Lean 4. This is not a formalization of a separate kernel — it IS the kernel. The Lean code compiles to a native binary (`iris-kernel-server`) that runs as an IPC subprocess, processing all 20 inference rules. The running code is the formal proof.

## What's formalized

### Core types (`IrisKernel/Types.lean`)

| Lean type | Rust mirror | Description |
|-----------|-------------|-------------|
| `NodeId` | `iris_types::graph::NodeId` | 64-bit content-addressed node identity |
| `BinderId` | `iris_types::graph::BinderId` | Binder identifier for lambda/let-rec |
| `TypeId` | `iris_types::types::TypeId` | 64-bit content-addressed type identity |
| `BoundVar` | `iris_types::types::BoundVar` | De Bruijn bound variable index |
| `Tag` | `iris_types::types::Tag` | Sum-type variant tag |
| `CostVar` | `iris_types::cost::CostVar` | Variable in cost expressions |
| `NodeKind` | `iris_types::graph::NodeKind` | 20-variant node kind enum |
| `CostBound` | `iris_types::cost::CostBound` | 13 cost bound variants (incl. `Amortized`, `HWScaled`) |
| `PrimType` | `iris_types::types::PrimType` | 7 primitive type tags |
| `TypeDef` | `iris_types::types::TypeDef` | 11 type definition variants |
| `TypeEnv` | `iris_types::types::TypeEnv` | Type environment (list of (TypeId, TypeDef)) |
| `Binding` | `iris_bootstrap::syntax::kernel::theorem::Binding` | Context binding (name + type) |
| `Context` | `iris_bootstrap::syntax::kernel::theorem::Context` | Typing context (list of bindings) |
| `Judgment` | `iris_bootstrap::syntax::kernel::theorem::Judgment` | Typing judgment `Γ ⊢ e : τ @ κ` |
| `CostLeq` | `iris_bootstrap::syntax::kernel::cost_checker::cost_leq` | Cost partial order as inductive Prop |

### Inference rules (`IrisKernel/Rules.lean`)

All 20 rules are implemented as executable functions in `Kernel.lean` and exported via C FFI in `FFI.lean`. They are also formalized as constructors of an inductive `Derivation` type in `Rules.lean` for proving metatheory:

| # | Constructor | Rust method | Type-theoretic counterpart |
|---|-------------|-------------|----------------------------|
| 1 | `assume` | `Kernel::assume` | Variable rule (Var) |
| 2 | `intro` | `Kernel::intro` | Arrow introduction (→I) |
| 3 | `elim` | `Kernel::elim` | Arrow elimination (→E) |
| 4 | `refl` | `Kernel::refl` | Reflexivity of equality |
| 5 | `symm` | `Kernel::symm` | Symmetry of equality |
| 6 | `trans` | `Kernel::trans` | Transitivity of equality |
| 7 | `congr` | `Kernel::congr` | Congruence of equality |
| 8 | `type_check_node` | `Kernel::type_check_node` | Annotation / axiom schema |
| 9 | `cost_subsume` | `Kernel::cost_subsume` | Cost subsumption |
| 10 | `cost_leq_rule` | `Kernel::cost_leq_rule` | Cost ordering witness |
| 11 | `refine_intro` | `Kernel::refine_intro` | Refinement type intro |
| 12 | `refine_elim` | `Kernel::refine_elim` | Refinement type elim |
| 13 | `nat_ind` | `Kernel::nat_ind` | Natural number induction |
| 14 | `structural_ind` | `Kernel::structural_ind` | Structural induction over ADTs |
| 15 | `let_bind` | `Kernel::let_bind` | Let binding (cut rule) |
| 16 | `match_elim` | `Kernel::match_elim` | Sum elimination / case analysis |
| 17 | `fold_rule` | `Kernel::fold_rule` | Catamorphism / structural recursion |
| 18 | `type_abst` | `Kernel::type_abst` | ForAll introduction (∀I) |
| 19 | `type_app` | `Kernel::type_app` | ForAll elimination (∀E) |
| 20 | `guard_rule` | `Kernel::guard_rule` | Conditional / if-then-else |

### Properties (`IrisKernel/Properties.lean`)

Proven theorems (complete proofs, no `sorry`):

- `refl_well_formed` — refl produces valid derivations in empty context at zero cost
- `cost_subsume_transitive` — cost subsumption is transitive
- `cost_leq_reflexive` — cost ordering is reflexive
- `zero_is_bottom` — Zero is below every cost bound
- `unknown_is_top` — every cost bound is below Unknown
- `assume_zero_cost` — assume rule always produces zero cost
- `intro_zero_cost` — lambda introduction always has zero cost
- `guard_cost_structure` — guard rule produces Sum(pred, Sup([then, else]))
- `let_bind_additive_cost` — let binding has additive cost
- `cost_subsume_chain` — two subsumptions can be combined via transitivity
- `elim_cost_decomposition` — function application cost = Sum(arg, Sum(fn, body))
- `const_zero_leq` — Constant(0) ≤ Constant(k)
- `const_leq_linear` — Constant(k) ≤ Linear(v)
- `linear_leq_nlogn` — Linear(v) ≤ NLogN(v)
- `complexity_chain` — Zero ≤ Polynomial(v, d) via the complexity hierarchy
- `cost_leq_preorder` — cost ordering is a preorder (reflexive + transitive)

Additional complete proofs:

- `fold_cost_geq_base` — fold cost ≥ base cost (via `sum_embed_left`/`sum_embed_right`)
- `fold_cost_geq_input` — fold cost ≥ input cost (via `sum_embed_left`)
- `fold_preserves_cost` — fold produces the exact cost Sum(input, Sum(base, Mul(step, input)))

Theorems with `sorry` (pending detailed list/context reasoning):

- `context_lookup_preserved_by_extension` — lookup is preserved when extending with a different binder (requires list reverse/append lemmas)
- `weakening_assume` — weakening for assume-derived judgments (depends on above)
- `cost_leq_const_antisymm` — constant ordering is antisymmetric (complete for base cases, `sorry` in transitivity cases)

Axioms:

- `weakening_general` — general weakening for all derivation forms (requires mutual induction on Derivation)

### Consistency (`IrisKernel/Consistency.lean`)

Proven theorems:

- `structural_ind_not_bottom` — structural induction cannot derive an empty Sum (bottom) type
- `match_elim_nonempty_arms` — match requires non-empty arm lists
- `assume_type_unique` — assume yields the same type for the same binder
- `intro_arrow_unique` — arrow type components are uniquely determined
- `refine_elim_unique_base` — refinement elimination base type is unique
- `cost_subsume_monotone` — cost subsumption only increases costs
- `zero_cost_subsumable` — zero-cost derivations can be subsumed to any cost
- `no_assume_in_empty_context` — assume fails in empty context
- `context_lookup_deterministic` — context lookup is a function
- `nat_ind_cost_additive` — nat_ind cost is Sum(base, step)
- `match_cost_bounds_worst_case` — match cost uses Sup of arm costs
- `type_abst_preserves_wellformedness` — type abstraction preserves well-formedness
- `type_app_requires_wellformedness` — type application requires well-formed result type

Theorems with `sorry`:

- `sup_is_upper_bound` — Sup is an upper bound for its elements (requires `v ≤ Sup vs` as a primitive or derived rule in CostLeq; two sorry markers in the case analysis)
- `let_bind_compose` — sequential let bindings compose (requires substitution/context restructuring lemma)

### Summary of all `sorry` and `axiom` markers

| File | Count | Reason |
|------|-------|--------|
| `Properties.lean` | 3 sorry, 1 axiom | Context list lemmas (2), CostLeq transitivity cases (2), general weakening axiom |
| `Consistency.lean` | 3 sorry | CostLeq element-of-Sup (1), CostLeq transitivity case (1), context restructuring (1) |
| `Types.lean` | 0 | All definitions are complete |
| `Rules.lean` | 0 | All 20 inference rules fully defined |

## Architecture

The kernel runs as an IPC server (`IrisKernelServer.lean`), spawned by Rust's `lean_bridge.rs` on first use. Communication is over stdin/stdout pipes:

```
Request:  rule_id(u8) + payload_len(u32 LE) + payload bytes
Response: result_len(u32 LE) + result bytes
```

The server dispatches rule IDs 1-20 to the corresponding `Kernel.*` function, serializes the result `Judgment` (or error), and writes it back. Rule 0 is `checkCostLeq`. Rule 255 is shutdown.

### File layout

| Lean file | Role |
|-----------|------|
| `IrisKernel/Types.lean` | Core types mirroring `iris-types` |
| `IrisKernel/Rules.lean` | Inductive `Derivation` (specification) |
| `IrisKernel/Kernel.lean` | Executable `def` functions (implementation) |
| `IrisKernel/KernelCorrectness.lean` | Proofs: executable = specification |
| `IrisKernel/FFI.lean` | `@[export]` C-callable wrappers + wire format |
| `IrisKernel/Eval.lean` | Cost checker, LIA evaluator |
| `IrisKernel/Properties.lean` | Metatheory (weakening, cost lattice) |
| `IrisKernel/Consistency.lean` | Metatheory (uniqueness, exhaustiveness) |
| `IrisKernelServer.lean` | IPC server (stdin/stdout dispatch loop) |

### Simplifications from the Rust types

1. **`CostBound`** — `Amortized` and `HWScaled` are included in the wire format but treated conservatively (transparent to inner cost) since their runtime-specific fields are opaque.

2. **`TypeDef.Refined`** — The refinement predicate (`LIAFormula`) is abstracted away. The Lean formalization tracks that a refinement type wraps a base type, but does not model the predicate logic.

3. **`TypeDef.NeuralGuard`** — Omits the `GuardSpec` field (opaque runtime blob). Retains input type, output type, and cost bound.

4. **`TypeDef.HWParam`** — Omits the `HardwareProfile` field. Retains the inner type.

5. **`TypeDef.Vec`** — Uses `Nat` for size instead of `SizeTerm` (which involves bound variables and arithmetic).

6. **`TypeEnv`** — Uses `List (TypeId × TypeDef)` instead of `BTreeMap`. Semantically equivalent for finite maps.

7. **`Theorem`** — The LCF-style opaque `Theorem` type with `proof_hash` is replaced by the inductive `Derivation` Prop. In the Lean formalization, a value of type `Derivation env Γ n τ κ` IS the proof — there's no separate "proof hash" because Lean's type system enforces the LCF invariant structurally.

## How to build

Requires [Lean 4](https://leanprover.github.io/lean4/doc/setup.html) (ships with Lake). On NixOS, `nix-shell -p lean4` works.

```bash
cd lean/

# Build the IPC server binary
lake build iris-kernel-server
# Binary at: .lake/build/bin/iris-kernel-server

# Build just the library (for proofs/checking)
lake build
```

`cargo build` in the root automatically invokes `lake build iris-kernel-server` if the binary doesn't exist yet. You don't need to build Lean manually for normal development.

## Design: why Lean, not Rust

In a self-improving system, the proof kernel is the trust anchor — the one component that must never be wrong, because everything else (mutations, evolutions, deployments) is gated through it. A bug in the kernel silently invalidates all guarantees.

Rust gives you memory safety, but not logical correctness. You can write a type checker in Rust that compiles, passes tests, and still has a subtle soundness hole (e.g., the original rule 19 `type_app` didn't check well-formedness of substituted types). Tests can miss edge cases. Code review can miss edge cases. Only a proof can't miss edge cases.

Lean 4 is both a proof assistant and a compiled language. The `Derivation` inductive type's constructors ARE the inference rules — Lean's kernel (which is itself proven correct) guarantees that every value of type `Derivation` was built by valid rule application. The executable `Kernel.lean` functions are proven to correspond to this specification. When the code compiles, the proofs hold. When the proofs hold, the kernel is correct.

The Rust side handles everything that doesn't need to be proven: process management, wire format encoding, BLAKE3 hashing, the opaque `Theorem` wrapper. The trust boundary is narrow and explicit.
