---
name: verify-iris
description: Verify that .iris programs are real functional replacements, not shortcuts. Run after any agent writes .iris code. Checks for the 8 known shortcut patterns.
argument-hint: "[worktree-path or branch-name]"
effort: max
---

# Verify IRIS Program Quality

Audit .iris programs for the 8 known shortcut patterns that agents consistently produce. This skill should be run after any agent completes work on .iris files.

## Target

If `$ARGUMENTS` specifies a worktree path or branch, audit that. Otherwise audit the current working directory's changed .iris files (via `git diff --name-only`).

## The 8 Shortcut Checks

For EVERY modified or new .iris file, spawn an agent to verify:

### Check 1: Statistics vs Transformation
Read the function's return expression. Does it return:
- A tuple of counts like `(total, changed, delta)` → FAIL
- A transformed graph (result of graph_add_node_rt / graph_connect / graph_replace_subtree) → PASS
- A computed value that IS the output (not metadata about the output) → PASS

### Check 2: Size Estimation vs Real Encoding
For serialization/encoding functions: does the function produce actual byte values, or just compute how many bytes there would be? Functions named `*_size`, `*_estimate`, `*_count` that don't also produce output → FAIL.

### Check 3: Delegation vs Implementation
For interpreter/evaluator functions: does every branch end with `graph_eval program inputs`? If so, the IRIS program isn't implementing anything — it's just a routing table back to Rust. Check what percentage of code paths are native IRIS vs graph_eval delegation.

### Check 4: Tests Execute .iris Files
For every test file in `tests/`:
- Does it use `include_str!("../src/iris-programs/...")` or `std::fs::read_to_string` to load .iris source?
- Does it call `iris_syntax::compile()` or `iris_syntax::parse()` on the loaded source?
- Does it evaluate the result through bootstrap_eval or interpreter::interpret?
- If the test only constructs SemanticGraphs in Rust → FAIL (it tests Rust, not IRIS)

### Check 5: Stub Tests
Search for `= 1` or `= true` or `= 0 + 1` patterns in .iris test bindings. Any test that unconditionally returns a constant → FAIL.

### Check 6: Reachable Functions
Trace the call graph from each program's entry point (the last `let` binding or the function the tests call). Are all other functions reachable? Defined-but-uncalled functions → FAIL.

### Check 7: Bootstrap Compatibility
Search for:
- `graph_add_node_rt pg <non-zero>` where the non-zero arg is an opcode not a kind → FAIL (broken in bootstrap)
- `graph_set_prim_op ... ()` → FAIL (TypeError in bootstrap)
- Any function that only works under `--features rust-scaffolding` without documenting it → FAIL

### Check 8: Fake Scale
For population/collection management: does the data structure hold multiple items? Or does a "population" hold one individual, a "multi-deme" track one score, an "archive" store one entry? Single-item masquerading as collection → FAIL.

## Output

Report per-file results:

| File | C1 | C2 | C3 | C4 | C5 | C6 | C7 | C8 | Verdict |
|------|----|----|----|----|----|----|----|----|---------|

Then list precise fixes needed for each FAIL.
