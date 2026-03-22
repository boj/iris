# IRIS Proof Kernel — Lean 4 Formalization

Lean 4 formalization of the IRIS proof kernel's 20 inference rules and their metatheoretic properties.

## What's formalized

### Core types (`IrisKernel/Types.lean`)

| Lean type | Rust source | Description |
|-----------|-------------|-------------|
| `NodeId` | `iris-repr/src/graph.rs::NodeId` | 64-bit content-addressed node identity |
| `BinderId` | `iris-repr/src/graph.rs::BinderId` | Binder identifier for lambda/let-rec |
| `TypeId` | `iris-repr/src/types.rs::TypeId` | 64-bit content-addressed type identity |
| `BoundVar` | `iris-repr/src/types.rs::BoundVar` | De Bruijn bound variable index |
| `Tag` | `iris-repr/src/types.rs::Tag` | Sum-type variant tag |
| `CostVar` | `iris-repr/src/cost.rs::CostVar` | Variable in cost expressions |
| `NodeKind` | `iris-repr/src/graph.rs::NodeKind` | 20-variant node kind enum |
| `CostBound` | `iris-repr/src/cost.rs::CostBound` | 11 cost bound variants (omits `Amortized`, `HWScaled`) |
| `PrimType` | `iris-repr/src/types.rs::PrimType` | 7 primitive type tags |
| `TypeDef` | `iris-repr/src/types.rs::TypeDef` | 11 type definition variants |
| `TypeEnv` | `iris-repr/src/types.rs::TypeEnv` | Type environment (list of (TypeId, TypeDef)) |
| `Binding` | `iris-kernel/src/theorem.rs::Binding` | Context binding (name + type) |
| `Context` | `iris-kernel/src/theorem.rs::Context` | Typing context (list of bindings) |
| `Judgment` | `iris-kernel/src/theorem.rs::Judgment` | Typing judgment `Γ ⊢ e : τ @ κ` |
| `CostLeq` | `iris-kernel/src/cost_checker.rs::cost_leq` | Cost partial order as inductive Prop |

### Inference rules (`IrisKernel/Rules.lean`)

All 20 rules from `iris-kernel/src/kernel.rs` are formalized as constructors of an inductive `Derivation` type:

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

## Correspondence with Rust code

| Lean file | Rust source |
|-----------|-------------|
| `IrisKernel/Types.lean` | `iris-repr/src/graph.rs`, `iris-repr/src/types.rs`, `iris-repr/src/cost.rs`, `iris-kernel/src/theorem.rs` |
| `IrisKernel/Rules.lean` | `iris-kernel/src/kernel.rs` (the 20 `Kernel::*` methods) |
| `IrisKernel/Properties.lean` | Metatheory (no direct Rust counterpart) |
| `IrisKernel/Consistency.lean` | Metatheory (no direct Rust counterpart) |

### Simplifications from the Rust implementation

1. **`CostBound`** — Omits `Amortized` (requires opaque `PotentialFn`) and `HWScaled` (requires opaque `HWParamRef`). These are runtime concepts not relevant to the type-theoretic formalization.

2. **`TypeDef.Refined`** — The refinement predicate (`LIAFormula`) is abstracted away. The Lean formalization tracks that a refinement type wraps a base type, but does not model the predicate logic.

3. **`TypeDef.NeuralGuard`** — Omits the `GuardSpec` field (opaque runtime blob). Retains input type, output type, and cost bound.

4. **`TypeDef.HWParam`** — Omits the `HardwareProfile` field. Retains the inner type.

5. **`TypeDef.Vec`** — Uses `Nat` for size instead of `SizeTerm` (which involves bound variables and arithmetic).

6. **`TypeEnv`** — Uses `List (TypeId × TypeDef)` instead of `BTreeMap`. Semantically equivalent for finite maps.

7. **`Theorem`** — The LCF-style opaque `Theorem` type with `proof_hash` is replaced by the inductive `Derivation` Prop. In the Lean formalization, a value of type `Derivation env Γ n τ κ` IS the proof — there's no separate "proof hash" because Lean's type system enforces the LCF invariant structurally.

## How to build

Requires [Lean 4](https://leanprover.github.io/lean4/doc/setup.html) and [Lake](https://github.com/leanprover/lean4/tree/master/src/lake) (ships with Lean 4).

```bash
cd lean/
lake build
```

The build will report any `sorry` markers. These are documented above and in the source files.

## Architecture notes

The Lean formalization replaces the LCF architecture (opaque `Theorem` type + trusted kernel module) with a natural deduction-style inductive `Derivation` type. This is a standard approach in proof assistants:

- In Rust (LCF style): only `kernel.rs` can construct `Theorem` values; the `pub(crate)` visibility enforces this.
- In Lean: `Derivation` is an inductive Prop whose constructors ARE the inference rules. A value of type `Derivation env Γ n τ κ` can only be built by applying these constructors — Lean's kernel enforces this.

Both approaches achieve the same guarantee: every proven judgment was derived by a valid sequence of inference rule applications.
