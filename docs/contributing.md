# Contributing to IRIS

This guide explains how to extend IRIS with new primitives, effects, compiler passes, and programs.

---

## Project Structure

```
src/
  iris-types/         Core data structures (SemanticGraph, Value, types, cost, wire)
  iris-bootstrap/     Bootstrap evaluator + syntax pipeline + proof kernel
    src/syntax/       Lexer, parser, AST, lowering (merged from iris-syntax)
    src/syntax/kernel/ LCF proof kernel (merged from iris-kernel)
  iris-exec/          Execution shim (capabilities, effect runtime, service)
  iris-evolve/        Evolution (NSGA-II, mutation, crossover, self-improvement)
  iris-clcu-sys/      FFI bindings to C CLCU interpreter
  bin/
    iris.rs           CLI tool (run, solve, check, daemon, repl)
src/iris-programs/    Core .iris programs (18 categories)
examples/             Demo .iris programs + Rust examples
tests/                Integration tests (115 files)
```

---

## How to Add a New Primitive

Primitives are named functions that map to opcodes. They are resolved at parse/lower time and executed by the interpreter.

### 1. Register the name in `prim.rs`

Edit `src/iris-bootstrap/src/syntax/prim.rs`:

```rust
pub fn resolve_primitive(name: &str) -> Option<(u8, u8)> {
    match name {
        // ... existing entries ...
        "my_new_prim" => Some((0xF0, 2)),  // (opcode, arity)
        _ => None,
    }
}
```

Choose an unused opcode. Current ranges:
- `0x00-0x09`: Arithmetic
- `0x10-0x15`: Bitwise
- `0x20-0x25`: Comparison
- `0x30-0x32`: Higher-order (map, filter, zip)
- `0x40-0x44`: Conversion
- `0x55`: State
- `0x80-0x8D`: Graph introspection
- `0xA0`: Meta-evolution
- `0xB0-0xC0`: String operations
- `0xC1-0xCD`: List/collection operations
- `0xD2`: Data access (tuple_get)
- `0xD8-0xE3`: Math
- `0xE4-0xE8`: Time, bytes

### 2. Implement in the interpreter

Edit `src/iris-exec/src/interpreter.rs`. Find the `eval_prim` function (or equivalent opcode dispatch) and add a case:

```rust
0xF0 => {
    // my_new_prim(a, b)
    let a = args[0].clone();
    let b = args[1].clone();
    match (&a, &b) {
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x.wrapping_add(*y))),
        _ => Err(InterpretError::TypeError(
            format!("my_new_prim expects (Int, Int), got ({:?}, {:?})", a, b),
        )),
    }
}
```

### 3. Add to the bytecode compiler (optional)

If the primitive should be supported by the bytecode VM, add it to `src/iris-exec/src/compile_bytecode.rs` and `src/iris-exec/src/vm.rs`.

### 4. Add to the LSP completions (optional)

Edit `src/iris-exec/src/service.rs` or `src/iris-exec/src/effect_runtime.rs` to include the new primitive in the completion list so editors can autocomplete it.

### 5. Write a test

Add a test in `tests/` or as a unit test in the relevant crate:

```rust
#[test]
fn test_my_new_prim() {
    let source = "let main x y = my_new_prim x y";
    let result = iris_syntax::compile(source);
    assert!(result.errors.is_empty());
    let (_, fragment, _) = &result.fragments[0];
    let output = iris_exec::interpreter::interpret(&fragment.graph, &[Value::Int(3), Value::Int(4)], None);
    assert_eq!(output.unwrap().0, vec![Value::Int(7)]);
}
```

---

## How to Add a New Effect

Effects are I/O operations categorized by `EffectTag`.

### 1. Add the tag to `EffectTag`

Edit `src/iris-types/src/eval.rs`:

```rust
pub enum EffectTag {
    // ... existing variants ...

    /// My new effect: (args) -> result.
    MyNewEffect,

    Custom(u8),
}
```

Update `from_u8()` and `to_u8()` with the new opcode:

```rust
impl EffectTag {
    pub fn from_u8(tag: u8) -> Self {
        match tag {
            // ... existing entries ...
            0x30 => Self::MyNewEffect,
            other => Self::Custom(other),
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            // ... existing entries ...
            Self::MyNewEffect => 0x30,
            Self::Custom(v) => v,
        }
    }
}
```

### 2. Add a surface-syntax name

Edit `src/iris-bootstrap/src/syntax/lower.rs` to recognize the effect name and emit an `Effect` node with the correct tag. Effect functions are resolved during lowering by checking for known names that map to effect opcodes.

### 3. Implement the handler

Edit `src/iris-exec/src/effects.rs` to handle the new effect in `RealHandler`:

```rust
EffectTag::MyNewEffect => {
    let arg = &request.args[0];
    // Perform the actual I/O
    let result = do_something(arg)?;
    Ok(result)
}
```

### 4. Update capability system

If the effect should be controllable by capabilities, add it to the default sets in `src/iris-exec/src/capabilities.rs`.

### 5. Add the LSP completion

Update the LSP to include the new effect name.

### 6. Write a test

Add a test in `tests/test_effects.rs` or `tests/test_io_primitives.rs`.

---

## How to Add a Compiler Pass

The compilation pipeline types are defined in `src/iris-types/src/compiler_ir.rs`. The pipeline behavior is implemented in IRIS programs under `src/iris-programs/compiler/`.

### Current Pipeline

```
SemanticGraph
  -> Pass 1:  Monomorphization ............ MonoGraph
  -> Pass 2:  Defunctionalization ......... FirstOrderGraph
  -> Pass 3:  Match Lowering .............. PredicatedGraph
  -> Pass 4:  Fold/Recursion Lowering ..... LoopGraph
  -> Pass 5:  Effect Lowering ............. TrampolineGraph
  -> Pass 6:  Neural Lowering ............. PrimitiveGraph
  -> Pass 7:  Data Layout ................. LayoutGraph
  -> Pass 8:  Instruction Selection ....... MicroOpSequence
  -> Pass 9:  Register Allocation ......... AllocatedSequence
  -> Pass 10: Container Packing ........... CLCUChain
```

### Adding a New Pass

1. Define the new IR types in `src/iris-types/src/compiler_ir.rs`
2. Create the transformation as an IRIS program in `src/iris-programs/compiler/`
3. Wire it into the pipeline between the appropriate existing passes
4. Write tests

---

## How to Write .iris Programs

### Program Structure

Every `.iris` file is a module containing top-level declarations:

```iris
-- Optional capability declarations
allow [FileRead]
deny [TcpConnect]

-- Type declarations
type Pair = (Int, Int)

-- Imports
import #abc123 as utils

-- Function declarations
let my_function x y : Int -> Int -> Int [cost: Const(1)] =
  x + y

-- Recursive functions
let rec factorial n : Int -> Int [cost: Linear(n)] =
  if n <= 1 then 1
  else n * factorial (n - 1)

-- Entry point (resolved by name "main" or last declaration)
let main x = factorial x
```

### Conventions

- Use `--` comments to document every function
- Annotate types and costs where possible
- Add `requires`/`ensures` contracts for safety-critical functions
- Use `fold` for iteration instead of explicit recursion where possible
- Use descriptive names following existing patterns in `src/iris-programs/`
- Organize related functions into category directories

### Test Cases in Comments

For use with `iris solve`:

```iris
-- test: 5 -> 120
-- test: 0 -> 1
-- test: 1 -> 1
-- test: 10 -> 3628800
```

Format: `-- test: input1, input2, ... -> output1, output2, ...`

---

## Testing Requirements

### Functional Tests

Every new feature needs functional tests. Add them as `[[test]]` entries in the root `Cargo.toml`:

```toml
[[test]]
name = "test_my_feature"
path = "tests/test_my_feature.rs"
```

### Integration Tests

Test end-to-end behavior by compiling `.iris` source, lowering to SemanticGraph, and evaluating:

```rust
#[test]
fn test_my_program() {
    let source = std::fs::read_to_string("src/iris-programs/my_category/my_program.iris").unwrap();
    let result = iris_syntax::compile(&source);
    assert!(result.errors.is_empty(), "Compile errors: {:?}", result.errors);

    let (_, fragment, _) = &result.fragments[0];
    let output = iris_exec::interpreter::interpret(
        &fragment.graph,
        &[Value::Int(42)],
        None,
    ).unwrap();

    assert_eq!(output.0, vec![Value::Int(expected)]);
}
```

### Benchmark Tests

Performance-sensitive changes should include benchmarks. The project has three benchmark suites:

- `tests/bench_math.rs` -- Mathematical operations
- `tests/bench_algorithms.rs` -- Algorithm performance
- `tests/bench_cs.rs` -- Computer science fundamentals

Run benchmarks:

```sh
cargo test --release bench_math -- --nocapture
cargo test --release bench_algorithms -- --nocapture
cargo test --release bench_cs -- --nocapture
```

### Verification Tests

For changes to the kernel or type system, add proof tests:

```sh
cargo test --release prove_algorithms -- --nocapture
```

---

## Code Style and Conventions

### Rust

- Follow the project's existing patterns over personal preferences
- Zero `unsafe` in the proof kernel (`iris-bootstrap::syntax::kernel`) -- this is an absolute rule
- Use `blake3` for all hashing
- Use `serde` with `Serialize`/`Deserialize` for serializable types
- Match the existing `#[derive]` patterns for new types
- Keep the proof kernel small and auditable
- Use `BTreeMap` for deterministic iteration order (important for content-addressing)

### IRIS Programs

- Comments: `-- Single line comment`
- Naming: `snake_case` for functions and variables
- Type annotations are encouraged but optional
- Cost annotations help the optimizer and verifier
- Contracts (`requires`/`ensures`) are encouraged for public APIs
- Keep functions small and composable
- Prefer `fold` over explicit recursion

### Git

- Tests must pass before merging
- Run `cargo test --release` (full suite) before submitting changes
- Include benchmark deltas for performance-sensitive changes

---

## Key Design Principles

1. **Programs are data.** SemanticGraph is the canonical representation. Programs can inspect and modify their own graph.

2. **Content-addressing everywhere.** Every node, type, and fragment is identified by its BLAKE3 hash. This enables deduplication, caching, and deterministic behavior.

3. **The proof kernel is sacred.** Only `iris-bootstrap/src/syntax/kernel/kernel.rs` can construct `Theorem` values. It contains zero `unsafe` Rust and will never be auto-modified by IRIS.

4. **Effects are descriptions, not actions.** The interpreter constructs `EffectRequest` values and yields them to an `EffectHandler`. This keeps the computation graph pure.

5. **Graceful degradation.** Higher execution tiers (VM, JIT) fall back to the tree-walking interpreter for unsupported node kinds. Nothing crashes; it just runs slower.

6. **Self-improvement is general.** The `self_improve` module optimizes mutation weights and strategies. Any system using evolutionary search can benefit from this, not just IRIS-specific problems.

7. **Test behavior, not implementation.** Tests verify that programs produce correct outputs, not that they use specific internal representations.
