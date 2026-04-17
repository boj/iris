---
title: "IRIS Programming Language"
---

# A functional language that compiles itself.

IRIS is a self-hosted functional programming language with algebraic data types, pattern matching, and native x86-64 compilation. The entire toolchain -- tokenizer, parser, compiler, code generator -- is written in IRIS.

```iris
type Tree = Leaf(Int) | Node(Tree, Tree)

let rec depth t : Tree -> Int =
    match t with
      | Leaf(_) -> 1
      | Node(left, right) ->
          let l = depth left in
          let r = depth right in
          1 + if l > r then l else r

let rec sum_tree t : Tree -> Int =
    match t with
      | Leaf(n) -> n
      | Node(left, right) -> sum_tree left + sum_tree right
```

## Features

**Self-hosted toolchain.** The IRIS compiler is written in IRIS. The tokenizer, parser, AST compiler, and native code generator are all `.iris` source files compiled by `iris-native` -- which was itself built from those same files. Run `bootstrap/build-native-self` to rebuild the compiler from source.

**Algebraic data types and pattern matching.** Define sum types with constructors and destructure them with exhaustive pattern matching. Option, Result, linked lists, state machines -- the usual functional toolkit, with no special syntax for built-in types.

**Native x86-64 compilation.** IRIS compiles to native machine code. No interpreter overhead for production use. The compiler handles recursion, higher-order functions, closures, let bindings, and tail calls.

**Modules and imports.** Organize code across files with path-based imports. `import "stdlib/option.iris" as Opt` brings a module into scope with qualified access. A standard library provides Option, Result, collections, math, and I/O.

## Quick start

```bash
git clone https://github.com/boj/iris.git && cd iris
export PATH="$PWD/bootstrap:$PATH"

# Run a program
iris-native examples/algorithms/factorial.iris 12
# 479001600

# Multi-file project with imports
iris-build run my_project/main.iris

# Rebuild the compiler from its own source
bootstrap/build-native-self
```

<div class="cta">

[Get started -->](/learn/get-started/) · [Language guide -->](/learn/language/) · [Standard library -->](/learn/stdlib/) · [GitHub](https://github.com/boj/iris)

</div>
