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

### 24. Dead primitive registrations -- opcode conflicts in prim.rs

**Severity: HIGH** -- 5 primitives have Rust implementations but are NOT callable.

The following primitives have working Rust code (`prim_*` methods in `lib.rs`) but
their opcodes were overwritten by `buf_new/buf_push/buf_finish` and `sort_by`:

| Primitive | Claimed opcode | Actually dispatches | Status |
|-----------|---------------|--------------------|----|
| `str_from_chars` | 0xD3 | `buf_new` | Dead code |
| `is_unit` | 0xD4 | `buf_push` | Dead code |
| `type_of` | 0xD5 | `buf_finish` | Dead code |
| `str_index_of` | 0xD6 | `Value::Unit` (reserved) | Dead code |
| `map_contains_key` | 0xCF | `sort_by` | Dead code |

These names are NOT in `prim.rs` and will fail to resolve at compile time. Several
.iris example files reference them (`lisp.iris`, `a_star.iris`, `huffman.iris`,
`caesar_cipher.iris`, `rot13.iris`) and will not actually run.

**Fix:** Assign new non-conflicting opcodes in `prim.rs` and add dispatch entries
in `lib.rs`. Suggested range: 0xE0-0xE8 (currently unused).

---

### 25. Test verification needed for rewritten .iris examples

**Severity: MEDIUM** -- 20 .iris files were rewritten to close FIXMEs but not tested.

The following files had significant logic changes (not just comment edits) and need
`cargo test` verification:

- `binary_search_tree.iris` -- full rewrite to State-backed BST
- `trie.iris` -- full rewrite to State-backed trie
- `sudoku_solver.iris` -- grid changed from tuple to State map
- `map_reduce.iris` -- grouping changed to State
- `anagram.iris` -- grouping changed to State
- `relational.iris` -- sort changed to `sort_by`
- `arithmetic_parser.iris` -- removed dispatch workaround, using `let rec...and`
- `lisp.iris` -- eval/apply rewritten as mutual recursion
- `edit_distance.iris` -- DP recurrence fixed + `str_chars` variant added
- `fizzbuzz.iris`, `roman_numerals.iris`, `morse_code.iris`, `visitor.iris` -- output format changes

---

### 26. Test coverage gaps in src/iris-programs/

**Severity: LOW** -- many core .iris modules lack test entries.

`tests/fixtures/iris-testing/` has test files for some modules but not all.
Missing test coverage for: `compiler/`, `foundry/`, `store/`, `syntax/` (beyond
the Rust-level parser tests), `stdlib/` (individual module tests), `mutation/`
operators, `population/` management.

---

## Resolved Issues

### Fixed in Rust (true primitives -- cross opaque Value/String boundary)

| # | Issue | Fix | Opcode |
|---|-------|-----|--------|
| 11 | `&&`/`||` not short-circuit (crashes on guarded conditions) | Lowerer transforms to `if/then/else` Guards | n/a |
| 14 | Lazy stream primitives not dispatched in bootstrap | Implemented `lazy_unfold`, `lazy_take`, `lazy_map`, `thunk_force` | 0xE9-0xEC |

**Note:** Issues #1, #2, #16, #21 were previously listed here but are affected by
the opcode conflict described in issue #24. The Rust implementations exist but are
not wired into `prim.rs`. See issue #24 for details.

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
