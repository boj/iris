# FIXME: IRIS Language Issues

Issues discovered while porting ~60 examples from other programming languages to IRIS.

---

## Open Issues

### 15. No mutual recursion (`and` keyword)

**Severity: HIGH** -- blocks idiomatic parser/interpreter design.

IRIS has `let rec` for self-recursion but no `and` keyword for mutually recursive
functions. This is a significant limitation for:
- Recursive descent parsers (expr calls term calls factor calls expr)
- Mutually recursive type-checking passes
- Interpreter eval/apply pairs

**Workaround:** Combine mutually recursive functions into a single function with a
`mode` integer parameter that dispatches internally. This works but produces unreadable
code and defeats the purpose of named functions.

**Suggested fix:** Support `let rec f = ... and g = ...` syntax like OCaml/Haskell,
lowering to a joint Fold/LetRec node group that shares a fixpoint.

---

### 17. `list_sort` only sorts by first element / integer comparison

**Severity: MEDIUM** -- blocks custom sort orders and multi-key sorting.

`list_sort` sorts tuples by coercing elements to integers. For tuples-of-tuples
(e.g., `((3, "c"), (1, "a"))`), it compares the first integer element only. There
is no way to provide a custom comparator.

**Workaround:** Prepend a sort key, sort, then strip the key (as done in
`examples/database/relational.iris` `rel_order_by`).

**Suggested fix:** Add `list_sort_by key_fn list` primitive that extracts sort keys
via a user function, or `list_sort_cmp cmp_fn list` that takes a comparator.

---

### 18. String processing is O(n) per character access

**Severity: MEDIUM** -- makes parsers and string algorithms very slow.

`char_at s i` iterates through `i` UTF-8 characters to reach position `i`.
`str_slice s i j` similarly does O(i) work just to find the start. This makes
any character-by-character string parser O(n^2) on the input length.

**Workaround:** Convert string to a tuple of char codes upfront with `str_chars`,
then work on the tuple. Costs O(n) memory for the intermediate tuple.

**Suggested fix:** Add `str_to_bytes s` returning a `Bytes` value with O(1) indexed
access, or make `char_at` O(1) by caching a byte-offset table internally.

---

### 19. No `let rec` across multiple top-level bindings

**Severity: MEDIUM** -- top-level functions can only self-recurse.

`let rec f x = ... f (x - 1) ...` works, but a top-level function `f` cannot call
a later-defined top-level function `g` that calls `f`. Each `let` binding is
independent. Even serial dependencies like `f calls g, g calls h` require all
three to be in the same `let rec` scope if any of them recurse.

Same root cause as #15. Both require parser/lowerer changes to support multi-binding
fixpoints.

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

**Workaround:** Use `State` (BTreeMap) for sparse updates, accepting O(log n) per
access instead of O(1).

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

### Acceptable Limitations

| # | Issue | Notes |
|---|-------|-------|
| 13 | Church encoding needs rank-2 types | Programs execute correctly; type annotations must be simplified. Bootstrap type checker is intentionally minimal. |
