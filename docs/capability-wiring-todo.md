# Capability Wiring

IRIS programs can declare capabilities (`allow`/`deny`) in source, and the `CapabilityGuardHandler` enforces them at runtime. The wiring from Rust capabilities → bootstrap evaluator → effect dispatch is complete.

## Current State (2026-03-25)

### What works (Rust side)
- `Capabilities` struct with 11 dimensions (effects, paths, hosts, memory, steps, time, threads, FFI, mmap, channels, env vars)
- `CapabilityGuardHandler` wraps any `EffectHandler`, checks capabilities before dispatch
- `RuntimeEffectHandler` implements all 44 effect tags with real I/O
- Preset profiles: `unrestricted()`, `sandbox()`, `io_restricted()`, `daemon_candidate()`
- Path security: null-byte rejection, `..` traversal blocking, parent canonicalization for symlink prevention
- **All execution paths enforce capabilities**: `IrisExecutionService`, `interpret_with_capabilities`, `interpret_sandboxed`
- Default sandbox blocks all I/O; only Print, Log, Timestamp, Random, ClockNs, RandomBytes, SleepMs allowed
- `PermissionDenied` errors correctly extracted from bootstrap's effect dispatch
- Capability propagation to sub-contexts (graph_eval, eval_ref, par_eval, spawn)
- 273 tests pass including 49 security/capability integration tests

### What works (IRIS side)
- Parser recognizes `allow [ TcpConnect "api.example.com" ]` and `deny [ FileWrite ]`
- Lowerer creates CapabilityDecl AST nodes (kinds 28/29)
- `src/iris-programs/exec/capabilities.iris` models the capability algebra (intersect, union, profiles)

### Remaining gap: source declarations → runtime
- Lowerer **discards** CapabilityDecl nodes, so they don't appear in the SemanticGraph
- Interpreter **ignores** any capability info in the graph, so programs run with whatever capabilities the Rust caller passes in
- No path from `.iris` source declaration → runtime enforcement

## What still needs to happen

### Phase 1: Capability declarations in SemanticGraph
- Lowerer stores `allow`/`deny` declarations in `FragmentMeta.contracts` or a new `FragmentMeta.capabilities` field
- Each fragment carries its declared capability requirements
- `SemanticGraph` serialization preserves capability declarations

### Phase 2: Interpreter reads and enforces capabilities
- Before executing a fragment, interpreter reads its declared capabilities
- Constructs a `Capabilities` struct from the declarations
- Wraps the effect handler with `CapabilityGuardHandler` using those capabilities
- Principle of least privilege: declared capabilities are an UPPER BOUND; the caller can further restrict but not expand

### Phase 3: Capability composition for multi-fragment programs
- When fragment A calls fragment B via Ref, B's capabilities = intersection(A's caps, B's declared caps)
- A parent cannot grant capabilities it doesn't have
- Capability violations at Ref boundaries produce clear errors

### Phase 4: IRIS-native capability checking
- The IRIS capabilities.iris program should be usable by the interpreter for capability algebra
- Eventually, capability checking itself runs as IRIS code, not Rust

## Files involved
- `src/iris-bootstrap/src/syntax/lower.rs`: capability declaration lowering (currently discards)
- `src/iris-types/src/fragment.rs`: `FragmentMeta`/`FragmentContracts` (needs capabilities field)
- `src/iris-exec/src/capabilities.rs`: `Capabilities` and `CapabilityGuardHandler` (wired in)
- `src/iris-exec/src/effect_runtime.rs`: `RuntimeEffectHandler` (implements real I/O)
- `src/iris-exec/src/interpreter.rs`: capability enforcement (wired in via interpret_with_capabilities)
- `src/iris-programs/syntax/iris_lowerer.iris`: self-hosted lowerer (needs to preserve cap decls)
- `src/iris-programs/exec/capabilities.iris`: IRIS capability algebra (Phase 4)
