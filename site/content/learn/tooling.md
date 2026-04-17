---
title: "Tooling"
description: "CLI tools for compiling, running, and building IRIS programs."
weight: 85
---

IRIS ships four tools. Two are self-hosted (written in IRIS, compiled to native x86-64). Two are bootstrap infrastructure.

## iris-native {#iris-native}

The self-hosted native compiler. Reads IRIS source, tokenizes, parses, compiles to bytecodes, and executes -- all in native machine code. No interpreter, no tree-walker.

### Compile and run {#native-run}

```bash
iris-native <source.iris> <arg>
```

The argument is passed to the program's `main` binding. It is parsed as a typed value: integers become `Int`, everything else becomes `String`.

```bash
iris-native examples/algorithms/factorial.iris 10
# 3628800

iris-native examples/classic-programs/hello_world.iris 0
# Hello, World!
```

### Compile only {#native-compile}

```bash
iris-native --compile <source.iris>
```

Outputs the compiled bytecodes to stdout. Used by `build-native-self` to compile pipeline stages.

```bash
iris-native --compile src/iris-programs/syntax/tokenizer.iris > tokenizer.bin
```

### How it works {#native-internals}

`iris-native` is a single static ELF binary (~500KB) containing:

| Section | Role |
|---------|------|
| Startup stub | x86-64 machine code: reads source file, dispatches to pipeline stages |
| Native VM | Bytecode interpreter compiled from `native_vm.iris` |
| Tokenizer | Compiled from `tokenizer.iris` |
| Parser | Compiled from `iris_parser.iris` |
| AST compiler | Compiled from `ast_compile_single.iris` |

Every section was compiled from IRIS source. The binary has no Rust, no libc, no external dependencies.

---

## iris-build {#iris-build}

Multi-file build tool. Resolves `import` declarations before compiling, so programs can span multiple files.

### Run with imports {#build-run}

```bash
iris-build run <source.iris> [args...]
```

Iteratively resolves all `import "path" as Alias` declarations (up to 8 levels deep), inlines the imported modules, then compiles and runs via `iris-native`.

```bash
iris-build run src/iris-programs/test_import.iris 0
```

### Compile with imports {#build-compile}

```bash
iris-build compile <source.iris>
```

Same import resolution, but outputs compiled bytecodes instead of executing.

### Import resolution {#import-resolution}

`iris-build` uses `resolve_imports.iris` (itself an IRIS program) to expand imports. Each pass replaces one level of `import` declarations with the contents of the referenced file. Programs with deeply nested imports may require multiple passes -- the tool runs up to 8.

---

## build-native-self {#build-native-self}

Bootstrapping script: the compiler compiles itself. Produces a new `iris-native` binary using the existing `iris-native` as the compiler.

```bash
bootstrap/build-native-self [-o output]
```

What it does:

1. Compiles the tokenizer, parser, and AST compiler from `.iris` source using `iris-native --compile`
2. Compiles `native_vm.iris` to x86-64 machine code by running it through `iris-native`
3. Compiles `emit_elf_stub.iris` to generate ELF headers and the startup stub
4. Assembles all sections into a new ELF binary

```
$ bootstrap/build-native-self
=== Self-Hosted IRIS Native Build ===
  Tokenizer...  48152 bytes
  Parser...     216408 bytes
  AST compiler... 69200 bytes
  Native VM...  11904 bytes (compiled from source)
  ELF + Stub... 908 bytes (compiled from source)

=== Self-hosted binary: bootstrap/iris-native (345672 bytes) ===
```

Every byte of the output was produced by IRIS code. The only non-IRIS input is the previous `iris-native` binary used to run the compilation.

There is also `build-native`, which uses `iris-stage0` instead of `iris-native` to compile the pipeline stages. It serves as a fallback path when `iris-native` cannot yet compile a new version of itself.

---

## iris-stage0 {#iris-stage0}

The frozen bootstrap binary. A JIT-based tree-walking evaluator (~8MB) written in Rust, now permanently frozen. It has a larger feature set than `iris-native` -- the goal is for `iris-native` to eventually replace it entirely.

### Commands {#stage0-commands}

```bash
iris-stage0 run <source.iris> [args...]        # Run via tree-walker
iris-stage0 compile <source.iris> -o out.json   # Compile to JSON SemanticGraph
iris-stage0 build <source.iris> -o binary       # Produce native binary (via stage0)
iris-stage0 direct <program.json> [args...]     # Evaluate pre-compiled JSON
iris-stage0 interp <interp.json> <prog.json>    # Run program through interpreter
iris-stage0 test <dir>                          # Run test suite
```

### When to use stage0 vs iris-native {#stage0-vs-native}

| | `iris-native` | `iris-stage0` |
|---|---|---|
| Written in | IRIS (self-hosted) | Rust (frozen) |
| Binary size | ~500KB | ~8MB |
| Multi-file imports | Via `iris-build` | Built-in |
| Recursion | Tail calls + caller-save convention | Full recursion |
| Higher-order functions | Supported | Supported |
| SemanticGraph JSON output | No | Yes |
| Test runner | No | Yes |

Use `iris-native` (via `iris-build` for multi-file programs) for normal development. Use `iris-stage0` when you need JSON output, the test runner, or features not yet implemented in `iris-native`.

---

## iris (wrapper) {#iris-wrapper}

A convenience script at `bootstrap/iris` that dispatches to the right tool:

```bash
iris run <source.iris> [args]          # Tree-walker via stage0
iris run-native <source.iris> [arg]    # Compile to native, then execute
iris build <source.iris> -o <output>   # Produce standalone native binary
iris compile <source.iris> -o <json>   # Compile to JSON SemanticGraph
iris test <dir>                        # Run test suite
iris pipeline                          # Compile all pipeline stages
iris version                           # Show version
```
