---
title: "Tools"
description: "IRIS developer tools: compiler, build system, and bootstrap."
layout: "single"
---

## iris-native {#iris-native}

The self-hosted IRIS compiler and runtime. Compiles `.iris` source files to
native x86-64 binaries through a pipeline written entirely in IRIS: tokenizer,
parser, AST compiler, and native VM.

```bash
# Compile and run a program
bootstrap/iris-native --run src/iris-programs/compiler/ast_compile_single.iris

# Compile a pipeline stage
bootstrap/iris-native --compile src/iris-programs/syntax/tokenizer.iris
```

`iris-native` is itself an x86-64 ELF binary produced by the IRIS compiler. It
can compile its own pipeline stages, making it a self-hosting compiler.

## iris-build {#iris-build}

Multi-file build tool. Resolves `import` statements, compiles dependencies, and
produces a single binary.

```bash
# Run a program with imports
bootstrap/iris-build run src/myprogram.iris

# Compile to a standalone binary
bootstrap/iris-build compile src/myprogram.iris -o output
```

Import syntax in `.iris` files:

```iris
import "stdlib/option.iris" as Opt
Opt.unwrap_or x 0
```

## iris (wrapper) {#iris-cli}

Convenience wrapper around `iris-stage0` and `iris-native`. Provides a unified
CLI for common operations.

```bash
iris run program.iris [args]          # Run via tree-walker
iris run-native program.iris [arg]    # Compile to native, then execute
iris build program.iris -o output     # Produce standalone native binary
iris pipeline                         # Compile all pipeline stages
iris compile program.iris -o out.json # Compile to JSON SemanticGraph
iris test dir/                        # Run test suite
iris version                          # Show version
```

## iris-stage0 {#iris-stage0}

The frozen bootstrap binary. Contains a tree-walking evaluator that operates on
SemanticGraphs. Used to bootstrap `iris-native` and as a fallback execution
engine.

```bash
iris-stage0 run program.iris [args]        # Execute a program
iris-stage0 compile source.iris -o out.json # Compile to SemanticGraph
iris-stage0 build source.iris -o binary    # Build native binary
iris-stage0 direct program.json [args]     # Run pre-compiled JSON
iris-stage0 interp interp.json prog.json   # Run program through interpreter
iris-stage0 test src/iris-programs/        # Run test suite
iris-stage0 rebuild                        # Rebuild bootstrap pipeline
```

`iris-stage0` is frozen -- it never changes. All improvements happen in `.iris`
files that run on top of it.

## build-native-self {#build-native-self}

Self-hosted binary builder. Compiles all pipeline stages using `iris-native`
itself, then packages them into a new `iris-native` binary.

```bash
bootstrap/build-native-self -o bootstrap/iris-native
```

This is the self-hosting loop: `iris-native` compiles the tokenizer, parser, AST
compiler, and native VM from `.iris` source, producing a new copy of itself. The
only non-IRIS dependency is the ELF stub template.

## Build from Source {#build}

IRIS is fully self-hosted. The bootstrap binary is included in the repository --
no external build tools required.

```bash
git clone https://github.com/boj/iris.git
cd iris

# Run a program
bootstrap/iris-stage0 run examples/algorithms/factorial.iris 10

# Build a native binary
bootstrap/iris build examples/algorithms/factorial.iris -o factorial
./factorial 10

# Run tests
bootstrap/iris-stage0 test src/iris-programs/
```

### Requirements {#requirements}

- x86-64 Linux
- Lean 4 (optional -- only needed for the proof kernel)

### Repository Layout {#layout}

| Path | Description |
|------|-------------|
| `bootstrap/iris-stage0` | Frozen bootstrap binary |
| `bootstrap/iris-native` | Self-hosted native compiler |
| `bootstrap/iris-build` | Multi-file build tool |
| `bootstrap/*.json` | Pre-compiled pipeline stages |
| `src/iris-programs/` | 290 `.iris` source files |
| `examples/` | 119 example programs |
| `lean/` | Lean 4 proof kernel |
