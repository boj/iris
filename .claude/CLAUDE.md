# IRIS Project Guidelines

## North Star Goals

### 1. Become Daimon's substrate
Replace Zig as the implementation language for Daimon (~/Development/daimon), and replace Daimon's implementation with IRIS so that Daimon can do full self-modification. IRIS is not a standalone project — it exists to become Daimon's substrate.

### 2. IRIS writes itself in IRIS (→ three phase transitions)
All Rust scaffolding has been replaced by .iris programs. The only permanent Rust is the proof kernel (iris-types + iris-bootstrap, Lob's theorem ceiling). Everything else -- mutation operators, seed generators, compiler passes, fitness functions, the interpreter itself -- is IRIS programs that IRIS bred.

Three phase transitions required (from the AI Minds Council, docs/council/10-ai-council-overview.md):

**Phase Transition 1: Tool → Organism** -- COMPLETE. All Rust components replaced by IRIS equivalents. Bootstrap evaluator is the sole execution engine.

**Phase Transition 2: Individual → Ecology** -- Programs must interact, compete, cooperate, and form emergent structures larger than any individual. Not just message passing -- an ecology where programs REACT to each other.

**Phase Transition 3: Slow → Recursive** -- Self-improvement must compound, each cycle faster than the last, with the interpreter itself as the first target for self-optimization.

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
- Fully self-hosted: all scaffolding (iris-exec, iris-evolve) deleted
- Permanent Rust: iris-types (types + graph), iris-bootstrap (evaluator + proof kernel + syntax)
- Everything else is .iris programs: evolution, mutation, crossover, fitness, compiler passes, codec, repr, deploy, LSP

## Architecture
- 4-layer stack: Evolution (L0) → Semantics (L1) → Verification (L2) → Hardware (L3)
- Canonical representation: SemanticGraph (20 node kinds, purely functional)
- Proof kernel: LCF-style, 20 inference rules, zero unsafe Rust
- Compiler: 10-pass pipeline (SemanticGraph → CLCU containers), implemented in .iris
- Execution: bootstrap evaluator (iris-bootstrap) with Rc<SemanticGraph> copy-on-write

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
- `graph_add_node_rt pg 0` creates a Prim node, then `graph_set_prim_op pg node opcode` sets the opcode
- Never pass `()` as a node ID — use `graph_get_root pg` explicitly
- **Test:** does the program work under `cargo test` (default build)?

## Build Conventions
- Rust workspace: iris-types + iris-bootstrap (2 crates)
- CLCU hardware layer at iris-clcu/ (AVX-512 container runtime, not yet integrated)
- BLAKE3 for all hashing
- Zero unsafe in proof kernel
- NixOS: use `nix-shell -p <packages> --run '<command>'` for tools not on PATH
- Default build: `cargo build` (evaluator + types)
- With parser/lowerer: `cargo build --features syntax`
- Stage0 binary: `cargo build --release --features syntax --bin iris-stage0 && cp target/release/iris-stage0 bootstrap/`
- Stage0 is the frozen bootstrap seed — all commands: compile, run, direct, interp, test, rebuild
