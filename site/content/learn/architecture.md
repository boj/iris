---
title: "Architecture"
description: "How IRIS programs are compiled to native x86-64 and executed."
weight: 80
---

IRIS programs are compiled from source to bytecodes and executed by a native
x86-64 VM. The entire compilation pipeline is written in IRIS and compiles
itself -- a verified fixed point.

## Compilation Pipeline {#pipeline}

```
source.iris
    |
    v
tokenizer.iris      Lexes source into tokens
    |
    v
iris_parser.iris    Parses tokens into an AST
    |
    v
ast_compile.iris    Compiles AST to flat bytecodes
    |
    v
native_vm.iris      Hand-assembled x86-64 VM executes bytecodes
```

Each stage is a `.iris` program. The tokenizer, parser, and AST compiler are
compiled to bytecodes by the same AST compiler, then executed by the same
native VM. This is how the compiler compiles itself.

### Tokenizer

`src/iris-programs/syntax/tokenizer.iris` -- Lexes source text into a token
stream. Recognizes keywords (`let`, `if`, `then`, `else`, `match`, `with`,
`import`, `as`, `type`, `rec`), operators, identifiers, integers, strings, and
comments (`--`).

### Parser

`src/iris-programs/syntax/iris_parser.iris` -- Recursive-descent parser that
produces an AST. Handles `let`/`let rec` bindings, lambda expressions, `if`/`then`/`else`,
`match`/`with` pattern matching, function application, infix operators, tuples,
imports, and ADT definitions.

### AST Compiler

`src/iris-programs/compiler/ast_compile_single.iris` -- Compiles an AST to a
flat tuple of bytecodes. Handles multi-binding modules by wrapping prior
declarations as `Let`/`in` bindings. Supports recursive functions, lambda
inlining, multi-parameter functions, and scope tracking.

## Native VM {#native-vm}

`src/iris-programs/compiler/native_vm.iris` (~960 lines) -- A bytecode
interpreter written as hand-assembled x86-64 machine code, emitted as byte
literals from IRIS code.

### Registers

| Register | Role |
|----------|------|
| `r12` | Program counter (index into bytecode) |
| `r13` | Value stack pointer (grows downward) |
| `r14` | Bytecode tuple pointer |
| `r15` | Heap bump allocator |
| `rbx` | Locals array pointer |

### Opcodes

The VM implements 30+ opcodes:

| Code | Name | Code | Name |
|------|------|------|------|
| 0 | `HALT` | 16 | `JMP` |
| 1 | `PUSH` | 17 | `JZ` |
| 2 | `ADD` | 18 | `MAKE_TUPLE` |
| 3 | `SUB` | 19 | `TUPLE_GET` |
| 4 | `MUL` | 21 | `TUPLE_LEN` |
| 5 | `DIV` | 22 | `LIST_APPEND` |
| 6 | `MOD` | 23 | `BITAND` |
| 7 | `NEG` | 24 | `SHR` |
| 8 | `EQ` | 25 | `FOLD_BEGIN` |
| 9 | `LT` | 26 | `FOLD_END` |
| 10 | `GT` | 27 | `LIST_RANGE` |
| 11 | `NE` | 29 | `PUSH_STR_PTR` |
| 12 | `LE` | 30 | `STR_LEN` |
| 13 | `GE` | 31 | `CHAR_AT` |
| 14 | `LOAD` | 32 | `STR_CONCAT` |
| 15 | `STORE` | 33 | `STR_SLICE` |
| | | 34 | `LIST_CONCAT` |
| | | 39 | `FILE_READ` |
| | | 40 | `DEBUG_PRINT` |

The dispatch loop loads an opcode from `bytecode[r12]`, walks a
compare-and-jump chain, executes the handler, and jumps back to the loop top.

### Memory Layout

The VM uses a fixed stack frame:

- **Value stack** (`[rbp-256..rbp-512]`) -- operand stack, grows downward
- **Locals** (`[rbp-768..rbp-512]`) -- 32 local variable slots (8 bytes each)
- **Scratch slots** (`[rbp-136..rbp-176]`) -- temporary storage for string/file ops
- **Heap** -- bump-allocated via `r15`, used for tuples and strings

Tuples and strings share a tagged-pointer format. Strings use tag `1` with
bytes packed after an 8-byte header.

## Self-Hosting {#self-hosting}

The compiler compiles itself through this loop:

1. `iris-native` loads the tokenizer, parser, and AST compiler as `.iris` source
2. Each stage is compiled to bytecodes by `ast_compile_single.iris`
3. The bytecodes are executed by `native_vm.iris` (which is itself compiled the same way)
4. The output is a new `iris-native` binary

The `bootstrap/build-native-self` script automates this. The only non-IRIS
dependency is the ELF stub template (frozen x86 machine code for the startup
sequence). Everything else -- tokenization, parsing, compilation, VM execution
-- is IRIS compiling IRIS.

### Bootstrap Chain

```
iris-stage0 (frozen seed)
    |  compiles + runs
    v
iris-native (self-hosted compiler + VM)
    |  compiles itself
    v
iris-native' (reproduced binary -- fixed point)
```

`iris-stage0` is the frozen bootstrap binary. It contains a tree-walking
evaluator and is used only to bootstrap the first `iris-native`. After that,
`iris-native` can reproduce itself.

## SemanticGraph {#semanticgraph}

The SemanticGraph is the canonical program representation used by `iris-stage0`.
It is a typed DAG with 20 node kinds:

| Tag | Kind | Tag | Kind |
|-----|------|-----|------|
| 0x00 | Prim | 0x0A | Effect |
| 0x01 | Apply | 0x0B | Tuple |
| 0x02 | Lambda | 0x0C | Inject |
| 0x03 | Let | 0x0D | Project |
| 0x04 | Match | 0x0E | TypeAbst |
| 0x05 | Lit | 0x0F | TypeApp |
| 0x06 | Ref | 0x10 | LetRec |
| 0x07 | Neural | 0x11 | Guard |
| 0x08 | Fold | 0x12 | Rewrite |
| 0x09 | Unfold | 0x13 | Extern |

Nodes are content-addressed via BLAKE3-truncated 64-bit IDs. Identical
subgraphs share the same ID automatically.

The native compilation pipeline (`iris-native`) works with bytecodes directly
and does not use SemanticGraph at runtime. SemanticGraph remains the format for
`iris-stage0` commands (`compile`, `run`, `direct`, `interp`).

## Four-Layer Model {#layers}

IRIS has additional capabilities organized into four layers:

```
L0  Evolution      Population search, mutation, selection
L1  Semantics      SemanticGraph (20 node kinds, BLAKE3 content-addressed)
L2  Verification   LCF proof kernel (20 inference rules, Lean 4)
L3  Hardware       Native x86-64 VM, iris-stage0 evaluator
```

**L0 -- Evolution.** Programs can be evolved through multi-objective genetic
search (NSGA-II, lexicase selection, novelty search) with 16 mutation operators.
See `src/iris-programs/evolution/`.

**L2 -- Verification.** An LCF-style proof kernel implements CaCIC (Cost-aware
Calculus of Inductive Constructions) with 20 inference rules, formalized in Lean
4. Runs as an IPC subprocess. See [Verification](/learn/verification/).

**L3 -- Hardware.** The execution layer: `native_vm.iris` for compiled programs,
`iris-stage0` for interpreted evaluation, and effect dispatch for I/O.
