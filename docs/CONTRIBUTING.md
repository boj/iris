# Contributing to IRIS

## Development Setup

IRIS is built with Rust on NixOS. The workspace has three main crates:

```
src/
  iris-types/       # Core types: SemanticGraph, Value, Fragment, BLAKE3 hashing, wire format
  iris-bootstrap/   # Minimal evaluator + syntax compiler (always available)
  iris-exec/        # Full interpreter, JIT backend, effect runtime (rust-scaffolding feature)
  iris-evolve/      # Evolution engine, NSGA-II, self-improvement (rust-scaffolding feature)
```

### Building

```bash
# Default build (bootstrap evaluator + syntax compiler only)
cargo build

# Full build with interpreter, JIT, evolution engine
cargo build --features rust-scaffolding

# Run tests (bootstrap-only, no iris-exec dependency)
cargo test --release --no-default-features --features syntax --test test_native_int

# Run tests (full, requires rust-scaffolding)
cargo test --release --features rust-scaffolding
```

### NixOS Notes

Many CLI tools aren't on `$PATH` by default. Use:
```bash
nix-shell -p <package> --run '<command>'
```

## Architecture

### The Bootstrap Chain

```
.iris source
  -> syntax::parse()         # Tokenize + parse to AST
  -> syntax::lower::compile_module()  # Lower AST to SemanticGraph fragments
  -> iris_bootstrap::evaluate()       # Tree-walk the SemanticGraph
     -> flat evaluator                # If fold body is flattenable
     -> native x86-64 codegen         # If fold body is all-float or all-int
```

Each compiled function becomes a `Fragment` with a BLAKE3 `FragmentId`. The evaluator resolves cross-fragment references via a `BTreeMap<FragmentId, SemanticGraph>` registry.

### Adding a New Primitive Opcode

1. **Choose an opcode** from the available range. Check `src/iris-bootstrap/src/syntax/prim.rs` for the current allocation.

2. **Register the name** in `prim.rs`:
   ```rust
   "my_prim" => Some((0xNN, arity)),
   ```

3. **Implement in the bootstrap evaluator** (`src/iris-bootstrap/src/lib.rs`). Find the `prim_dispatch` match and add:
   ```rust
   0xNN => { /* implementation */ }
   ```

4. **Implement in the flat evaluator** if applicable (same file, `eval_flat_reuse` and `eval_flat_f64` functions).

5. **Update the reference docs** at `site/content/learn/reference.md`.

6. **Update downstream consumers**: LSP completion lists, syntax highlighting, any `.iris` programs that should use it.

### Adding a New Node Kind

Node kinds are defined in `src/iris-types/src/graph.rs` (`NodeKind` enum). Adding one requires:

1. Add variant to `NodeKind` and `NodePayload`
2. Update hash computation in `src/iris-types/src/hash.rs`
3. Update wire serialization in `src/iris-types/src/wire.rs`
4. Update the bootstrap evaluator (`eval_node` dispatch in `lib.rs`)
5. Update the flat evaluator if the node should be flattenable
6. Update the type checker in `src/iris-bootstrap/src/syntax/kernel/checker.rs`

### Adding a New Effect Tag

Effect tags are in `src/iris-types/src/eval.rs` (`EffectTag` enum). Adding one:

1. Add variant to `EffectTag` with a unique byte value
2. Register the name in `src/iris-bootstrap/src/syntax/lower.rs` (`resolve_effect_name`)
3. Implement the handler in `src/iris-exec/src/effect_runtime.rs`
4. Update `site/content/learn/reference.md`

## Code Organization

### Key Files

| File | Purpose | LOC |
|------|---------|-----|
| `src/iris-bootstrap/src/lib.rs` | Bootstrap evaluator, flat eval, native codegen | ~8K |
| `src/iris-bootstrap/src/syntax/parser.rs` | Tokenizer + recursive descent parser | ~400 |
| `src/iris-bootstrap/src/syntax/lower.rs` | AST to SemanticGraph lowering | ~1.4K |
| `src/iris-bootstrap/src/syntax/prim.rs` | Primitive name -> opcode table | ~100 |
| `src/iris-types/src/graph.rs` | SemanticGraph, Node, Edge definitions | ~300 |
| `src/iris-types/src/hash.rs` | BLAKE3 hashing (NodeId, TypeId, FragmentId) | ~500 |
| `src/iris-types/src/wire.rs` | Binary serialization format | ~1.5K |

### Testing Conventions

- **Integration tests** in `tests/` -- registered in `Cargo.toml` with `[[test]]` entries
- **Unit tests** in-module with `#[cfg(test)] mod tests`
- **IRIS self-hosted tests** in `tests/fixtures/iris-testing/` -- run via `iris-bootstrap test`
- Test functions for `.iris` programs are prefixed `test_` and take zero arguments

### Commit Style

```
feat: short description
fix: short description
perf: short description
docs: short description
chore: short description
style: short description
```

No `Co-Authored-By` trailers.

## Feature Flags

| Flag | What it enables |
|------|----------------|
| `syntax` | Parser + lowerer in iris-bootstrap |
| `rust-scaffolding` | Full interpreter, JIT, evolution, effects (includes `syntax`) |
| `jit` | JIT compilation via aot_compile.iris |
| `clcu` | CLCU/AVX-512 backend |

Default is `rust-scaffolding`. Tests that only need the bootstrap evaluator use `--no-default-features --features syntax`.
