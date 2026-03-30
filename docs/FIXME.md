# FIXME: IRIS Language Issues

Issues discovered while porting ~60 examples from other programming languages to IRIS.

---

## Open Issues

### 19. No `let rec` across multiple top-level bindings

**Severity: MEDIUM** -- top-level functions can only self-recurse.

`let rec f x = ... f (x - 1) ...` works, but a top-level function `f` cannot call
a later-defined top-level function `g` that calls `f`. Each `let` binding is
independent. Even serial dependencies like `f calls g, g calls h` require all
three to be in the same `let rec` scope if any of them recurse.

**Workaround:** Nest everything in a single deep `let rec` block, or use
`compile_source` to dynamically link modules.

---

### 20. Immutable tuple performance cliff for grid/matrix algorithms

**Severity: HIGH** for Daimon readiness -- most real-world algorithms need mutable arrays.

Every `list_set` (the IRIS stdlib version) creates 3 intermediate tuples per call.
For an 81-element Sudoku board solved via backtracking (~1000 placements), that's
~243,000 intermediate tuple allocations. For A* on a 100x100 grid, the cost is
prohibitive.

This is not a bug but a fundamental performance characteristic of immutable tuples.

**Workaround (PARTIALLY RESOLVED):** Use `State` (BTreeMap via `state_empty` /
`map_insert` / `map_get`) for sparse updates, accepting O(log n) per access instead
of O(1). This avoids the intermediate allocation cascade for large mutable arrays.

**Suggested fix (long-term):** Add a persistent vector (like Clojure's) as a Value
variant, giving O(log32 n) update and O(log32 n) access. Or add transient/mutable
tuple support within a linear scope (like Haskell's ST monad).

---

## Resolved Issues

### Fixed in Rust (true primitives -- cross opaque Value/String boundary)

| # | Issue | Fix | Opcode |
|---|-------|-----|--------|
| 1 | No `str_from_chars` (char codes to String) | Added `str_from_chars` | 0xD3 |
| 2 | `map_get` returns Unit, can't distinguish missing keys | Added `is_unit` + `map_contains_key` | 0xD4, 0xCF |
| 11 | `&&`/`||` not short-circuit (crashes on guarded conditions) | Lowerer transforms to `if/then/else` Guards | n/a |
| 14 | Lazy stream primitives not dispatched in bootstrap | Implemented `lazy_unfold`, `lazy_take`, `lazy_map`, `thunk_force` | 0xE9-0xEC |
| 16 | No runtime type predicates (`is_tuple`, `is_int`, etc.) | Added `type_of` returning int tag | 0xD5 |
| 21 | No `str_index_of` for substring search | Added `str_index_of` | 0xD6 |

### Fixed in IRIS (stdlib functions in `src/iris-programs/stdlib/list_ops.iris`)

| # | Issue | Fix |
|---|-------|-----|
| 5 | DP row updates can't reference current row's earlier cells | `scan_left` -- fold collecting all intermediate values |
| 8 | No `list_set` for element replacement | `list_set` -- composed from `list_take`/`list_append`/`list_drop`/`list_concat` |
| 12 | `fold` can't break early | `fold_while` -- uses `(continue?, acc)` done-flag pattern |
| 22 | `fold_while` API confusing | Clarified docstring and examples |

### Not Issues (already work correctly)

| # | Reported Issue | Reality |
|---|----------------|---------|
| 3 | Tuples don't nest | They do. `((1,2), (3,4))` works correctly. |
| 4 | `int_to_string` missing | Already exists (opcode 0xB7). `map_get` also accepts Int keys. |
| 6 | No negative literal syntax | Parser handles `-expr` via `parse_unary_expr()`. |
| 7 | `filter` Bool vs Int ambiguity | Accepts both: `Bool(true)` and `Int(nonzero)` both keep. |
| 9 | `\|>` with lambdas broken | Works fine. `x \|> (\y -> y+1)` desugars to `(\y -> y+1) x`. |
| 10 | Singleton tuple `(val,)` ambiguous | Parser handles trailing comma explicitly. |
| 23 | `pow` only integer exponents | Already polymorphic: uses `f64::powf` when either arg is Float. |

### Resolved with fix/workaround

| # | Issue | Resolution |
|---|-------|------------|
| 15 | No mutual recursion (`and` keyword) | **RESOLVED.** `let rec f = ... and g = ...` syntax now works; lowers to a joint fixpoint. Recursive descent parsers and eval/apply pairs can be expressed idiomatically. |
| 17 | `list_sort` only sorts by first element / integer comparison | **RESOLVED.** `sort_by comparator_fn list` primitive added. Comparator receives `(a, b)` tuple, returns negative/0/positive. Used in `a_star.iris` and `relational.iris`. |
| 18 | String processing is O(n) per character access | **RESOLVED.** Convert string to tuple of char codes upfront with `str_chars`, then use `char_at` on the tuple for O(1) indexed access per character. `str_chars` + `char_at` exist and work correctly. |

### Acceptable Limitations

| # | Issue | Notes |
|---|-------|-------|
| 13 | Church encoding needs rank-2 types | Programs execute correctly; type annotations must be simplified. Bootstrap type checker is intentionally minimal. |
