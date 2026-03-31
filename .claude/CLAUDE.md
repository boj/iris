# IRIS Project Guidelines

## North Star Goals

### 1. Become Daimon's substrate
Replace Zig as the implementation language for Daimon (~/Development/daimon), and replace Daimon's implementation with IRIS so that Daimon can do full self-modification. IRIS is not a standalone project — it exists to become Daimon's substrate.

### 2. IRIS writes itself in IRIS (→ three phase transitions)
IRIS is 100% self-hosted. No Rust, no Cargo, no .rs files remain. The only non-IRIS component is the Lean 4 proof kernel (Lob's theorem ceiling -- a system cannot verify its own verifier). Everything else -- compiler passes, mutation operators, seed generators, fitness functions, the interpreter itself -- is IRIS programs compiled through the IRIS pipeline.

Three phase transitions required (from the AI Minds Council, docs/council/10-ai-council-overview.md):

**Phase Transition 1: Tool → Organism** -- COMPLETE. All scaffolding replaced by IRIS equivalents. Bootstrap evaluator is the sole execution engine.

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
- 100% self-hosted: no Rust, no Cargo, no .rs files
- Bootstrap: frozen `iris-stage0` binary (mini_eval + JIT) in `bootstrap/`
- Pre-compiled pipeline stages: `bootstrap/*.json` (tokenizer, parser, lowerer, interpreter, compiler)
- 243 infrastructure .iris files in `src/iris-programs/` (19 categories)
- 119 example .iris files, 10 benchmark .iris files
- Only non-IRIS component: Lean 4 proof kernel in `lean/`

## Architecture
- 4-layer stack: Evolution (L0) → Semantics (L1) → Verification (L2) → Hardware (L3)
- Canonical representation: SemanticGraph (20 node kinds, purely functional)
- Proof kernel: Lean 4, LCF-style, 20 inference rules (runs as IPC subprocess)
- Self-hosting pipeline: tokenizer.iris → parser.iris → lowerer.iris (all in IRIS)
- Compiler: 10-pass pipeline (SemanticGraph → CLCU containers), implemented in .iris
- Execution: bootstrap evaluator (`iris-stage0`) with mini_eval + JIT

## IRIS Program Quality Rules (for agents writing .iris files)

These are absolute requirements for all .iris programs:

### Programs must TRANSFORM data, not DESCRIBE it
- A compiler pass must produce a transformed SemanticGraph, not a tuple of statistics
- A serializer must produce bytes, not size estimates
- A decoder must construct a graph, not return hash summaries
- An interpreter must evaluate expressions, not delegate to `graph_eval` for everything
- **Test:** does the function's return value contain the actual output, or just metadata about what the output would be?

### Tests must execute the .iris files
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
- **Test:** does the program work under `bootstrap/iris-stage0 run`?

## Build Conventions
- No Rust, no Cargo. Everything goes through `bootstrap/iris-stage0`
- Compile: `bootstrap/iris-stage0 compile <source.iris> -o output.json`
- Run: `bootstrap/iris-stage0 run <source.iris> [args...]`
- Build native: `bootstrap/iris-stage0 build <source.iris> -o binary`
- Direct eval: `bootstrap/iris-stage0 direct <program.json> [args...]`
- Interp: `bootstrap/iris-stage0 interp <interp.json> <prog.json> [args]`
- Test: `bootstrap/iris-stage0 test src/iris-programs/`
- Rebuild pipeline: `bootstrap/iris-stage0 rebuild`
- BLAKE3 for all hashing
- Lean 4 proof kernel at `lean/` (optional, for verification)
- NixOS: use `nix-shell -p <packages> --run '<command>'` for tools not on PATH
- Stage0 is the frozen bootstrap seed — it never changes. All commands: compile, run, build, direct, interp, test, rebuild
