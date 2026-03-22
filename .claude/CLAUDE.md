# IRIS Project Guidelines

## North Star Goals

### 1. Become Daimon's substrate
Replace Zig as the implementation language for Daimon (~/Development/daimon), and replace Daimon's implementation with IRIS so that Daimon can do full self-modification. IRIS is not a standalone project — it exists to become Daimon's substrate.

### 2. IRIS writes itself in IRIS (→ three phase transitions)
IRIS should progressively replace its own Rust/C implementation with IRIS-evolved equivalents. The Rust and C are scaffolding — used to bootstrap the first generation, then discarded as IRIS replaces each component with an evolved, verified equivalent. The only permanent human-authored code is the proof kernel (~5K lines Rust, Löb's theorem ceiling). Everything else — mutation operators, seed generators, compiler passes, fitness functions, the interpreter itself — should eventually be IRIS programs that IRIS bred.

Three phase transitions required (from the AI Minds Council, docs/council/10-ai-council-overview.md):

**Phase Transition 1: Tool → Organism** — IRIS must run continuously, maintain itself, and produce its own components (autopoiesis). Every Rust component needs an IRIS-evolved equivalent.

**Phase Transition 2: Individual → Ecology** — Programs must interact, compete, cooperate, and form emergent structures larger than any individual. Not just message passing — an ecology where programs REACT to each other.

**Phase Transition 3: Slow → Recursive** — Self-improvement must compound, each cycle faster than the last, with the interpreter itself as the first target for self-optimization.

## What Daimon Needs From IRIS
- 360K+ LOC equivalent capability (79 cognitive modules, 85 sensory plugins, 7 daemons)
- Stateful computation (knowledge graph that mutates over time)
- Real I/O effects (network, files, PostgreSQL, Unix sockets, Discord)
- Multi-program composition (modules that interact via message passing)
- Continuous self-modification (weights, graph edges, predictions updated online)
- Real-time constraints (800ms cognitive cycles)
- ~50MB RAM footprint
- Formal verification of cognitive invariants

## Current State
- Gen1 implementation complete (~11K LOC Rust + C)
- Can evolve simple programs (sum, max, etc.)
- Gen2 in progress: CLCU execution bridge, perf counters, harder test problems
- iris-exec (22,829 LOC) gated behind `rust-scaffolding` feature; default build uses iris-bootstrap shims

## Architecture
- 4-layer stack: Evolution (L0) → Semantics (L1) → Verification (L2) → Hardware (L3)
- Canonical representation: SemanticGraph (20 node kinds, purely functional)
- Proof kernel: LCF-style, 20 inference rules, zero unsafe Rust
- Compiler: 10-pass pipeline (SemanticGraph → CLCU containers)
- Execution: bootstrap evaluator (default) or full interpreter/JIT/VM (`rust-scaffolding`)
- `iris-exec` split: always-available thin API (1,696 LOC) + gated heavy impl (22,829 LOC)

## IRIS Program Quality Rules (for agents writing .iris files)

When writing IRIS programs that replace Rust scaffolding, these are absolute requirements:

### Programs must TRANSFORM data, not DESCRIBE it
- A compiler pass must produce a transformed SemanticGraph, not a tuple of statistics
- A serializer must produce bytes, not size estimates
- A decoder must construct a graph, not return hash summaries
- An interpreter must evaluate expressions, not delegate to `graph_eval` for everything
- **Test:** does the function's return value contain the actual output, or just metadata about what the output would be?

### Tests must execute the .iris files
- Rust tests that construct SemanticGraphs in Rust and test Rust APIs do NOT test .iris programs
- Tests must: load the .iris file → compile it → evaluate it through the bootstrap evaluator or interpreter → assert on the result
- A test that returns `= 1` unconditionally is a stub, not a test
- **Test:** remove the .iris file — does the test still pass? If yes, it's not testing the .iris file

### Every defined function must be reachable
- If a dispatch table routes to 4 of 16 operators, the other 12 are dead code
- If a helper function is defined but never called from the main entry point, it's dead code
- **Test:** can you trace a call path from the program's entry point to every function?

### Bootstrap compatibility
- `graph_add_node_rt` behaves differently in bootstrap (arg = node kind) vs rust-scaffolding (arg = always Prim, arg = opcode)
- Portable pattern: `graph_add_node_rt pg 0` (creates Prim) then `graph_set_prim_op pg node opcode` (sets opcode)
- Never pass `()` as a node ID — use `graph_get_root pg` explicitly
- **Test:** does the program work under `cargo test` (default build) without `--features rust-scaffolding`?

## Build Conventions
- Rust workspace at project root
- C library at iris-clcu/
- BLAKE3 for all hashing
- Zero unsafe in proof kernel
- NixOS: use `nix-shell -p <packages> --run '<command>'` for tools not on PATH
- Default build (`cargo build`): uses iris-bootstrap shims, no heavy Rust interpreter
- Full build (`cargo build --features rust-scaffolding`): full interpreter, JIT, VM, effects
