---
title: "Tooling"
description: "CLI commands, REPL, LSP, and editor integration."
weight: 85
---

## iris run {#run}

Execute an IRIS program. Parses the source, compiles to SemanticGraph fragments, and evaluates the `main` binding (or the last definition if no `main` exists).

```bash
iris run [flags] <file.iris> [args...]
```

| Flag | Description |
|------|-------------|
| `--watch` | Re-run automatically when the file changes (polls every 500ms) |
| `--improve` | Enable observation-driven self-improvement (background daemon) |
| `--improve-threshold N` | Max slowdown factor for performance gate (default: 2.0) |
| `--improve-min-traces N` | Min traces before attempting evolution (default: 50) |
| `--improve-sample-rate N` | Fraction of calls to trace (default: 0.01 = 1%) |
| `--improve-budget N` | Max seconds per evolution attempt (default: 5) |
| `--backend MODE` | Execution backend: `auto`, `tree`, `jit`, `aot`, `clcu` (default: `tree`) |

**Examples:**

```bash
# Run a program with arguments
iris run examples/algorithms/factorial.iris 10
# 3628800

# Watch mode: re-runs on file save
iris run --watch myprogram.iris

# Self-improvement: evolve faster implementations at runtime
iris run --improve server.iris
# [improve] daemon started: min_traces=50, threshold=2.0x, budget=5s
# [improve] attempting compute (73 test cases, avg 124.3us)
# [improve] deployed compute (124.3us -> 68.1us, 45% faster)
```

CLI arguments are parsed as typed values: integers become `Int`, decimals become `Float64`, `true`/`false` become `Bool`, and everything else becomes `String`.

---

## iris repl {#repl}

Interactive read-eval-print loop with persistent history. Definitions accumulate across inputs so you can build up state incrementally.

```bash
iris repl
```

```
IRIS REPL v0.1.0
Type expressions or :help for commands.

iris> let x = 42
42
iris> x * 2 + 1
85
iris> let double = \n -> n * 2
iris> double x
84
```

**REPL commands:**

| Command | Description |
|---------|-------------|
| `:type <expr>` | Show the inferred type of an expression |
| `:load <file>` | Load an `.iris` file into the current session |
| `:list` | Show all defined names and their types |
| `:clear` | Reset all accumulated definitions |
| `:help` | Show available commands |
| `:quit` | Exit the REPL |

Lines ending with `\` continue on the next line for multi-line input:

```
iris> let complex = \x -> \
  ...   let y = x * 2 in \
  ...   y + 1
iris> complex 10
21
```

History is saved to `~/.iris/repl_history` between sessions.

---

## iris check {#check}

Type-check and verify a program. Each definition is verified at its auto-detected tier, and the result reports how many type obligations are satisfied.

```bash
iris check <file.iris>
```

**Example output:**

```
[OK] factorial: 3/3 obligations satisfied (score: 1.00)
[OK] fibonacci: 5/5 obligations satisfied (score: 1.00)
All 2 definitions verified.
```

On failure:

```
[FAIL] broken: 2/4 obligations satisfied (score: 0.50)
  - node NodeId(3): expected Int, found String
  - node NodeId(7): type mismatch in branch arms
```

Exits with code 1 if any definition fails verification.

---

## iris lint {#lint}

Static analysis for common issues. Operates on both the AST (name-level analysis) and compiled SemanticGraph (node-count analysis).

```bash
iris lint <file.iris>
```

**Lint rules:**

| Code | Name | Description |
|------|------|-------------|
| L001 | Unused binding | A `let` binding or function parameter is never referenced |
| L002 | Shadowed name | A binding shadows an existing name in scope |
| L003 | Large fragment | Compiled fragment exceeds 200 nodes (complexity threshold) |
| L004 | Missing type | Top-level function has no return type annotation |
| L005 | Constant fold | Fold step function ignores its accumulator parameter |

**Example output:**

```
L001: unused binding 'temp' (line 12)
L002: shadowed binding 'x' (line 18)
L004: function 'process' has no return type annotation (line 5)
L005: fold step function ignores accumulator 'acc' (line 23)

4 warning(s) in myprogram.iris
```

---

## iris solve {#solve}

Evolve a program from test specifications. Provide input-output pairs as `-- test:` comments in the source file, and the evolution engine will synthesize a correct implementation.

```bash
iris solve [flags] <spec.iris>
```

| Flag | Description |
|------|-------------|
| `--population N` | Population size (default: 64) |
| `--generations N` | Max generations (default: 500) |

**Spec file format:**

```iris
-- test: 0 -> 0
-- test: 1 -> 1
-- test: 5 -> 25
-- test: 10 -> 100
-- test: (3, 4) -> 7
```

Each `-- test:` line specifies an input-output mapping. Tuple inputs use `(a, b)` syntax.

**Example:**

```bash
iris solve square.iris
# Solving with 4 test cases (population=64, generations=500)...
# Solution found in 23 generations (0.8s)!
#   fitness: correctness=1.0000, performance=0.9200
#   graph: 3 nodes, 2 edges
```

---

## iris store {#store}

Manage the fragment cache. Improved fragments (from `--improve` or the daemon) are persisted here as content-addressed binaries.

```bash
iris store <subcommand>
```

| Subcommand | Description |
|------------|-------------|
| `list` | List all cached fragments with name, generation, and hash |
| `get <name>` | Show details of a specific cached fragment |
| `rm <name>` | Remove a cached fragment |
| `clear` | Clear the entire cache |
| `path` | Print the cache directory path |

**Examples:**

```bash
iris store list
# NAME                           GEN  HASH
# compute                          3  a1b2c3d4e5f67890
# transform                        1  f0e1d2c3b4a59687
#
# 2 fragment(s)

iris store get compute
# Name:       compute
# Generation: 3
# Hash:       a1b2c3d4e5f67890...
# File:       /home/user/.iris/cache/a1b2c3d4e5f67890.frag
# Size:       1248 bytes

iris store path
# /home/user/.iris/cache
```

---

## iris explain {#explain}

Show detailed explanations for error codes. Useful when the compiler or runtime emits a terse error.

```bash
iris explain <error-code>
```

**Available error codes:**

| Code | Summary |
|------|---------|
| E001 | Unknown identifier -- typo, missing import, or nonexistent primitive |
| E002 | Type mismatch -- wrong type in context (wrong arg type, mismatched branches) |
| E003 | Non-exhaustive pattern match -- missing cases in `match` expression |
| E004 | Division by zero -- unguarded `/` or `%` with zero divisor |
| E005 | Step limit exceeded -- infinite recursion or very large input |
| E006 | Unused binding -- `let` introduces a name that is never used |

**Example:**

```bash
iris explain E004
# E004: Division by zero
#
# Integer division or modulo by zero. Guard with: if y == 0 then 0 else x / y
```

---

## iris daemon {#daemon}

Run the continuous self-improvement daemon. Monitors the program ecosystem, evolves better implementations, and persists improvements across restarts. State is stored in `.iris-daemon/` in the current directory.

```bash
iris daemon [N] [flags]
```

| Flag | Description |
|------|-------------|
| `--exec-mode MODE` | `continuous` or `interval:N` (ms). Default: `interval:800` |
| `--improve-threshold N` | Max slowdown for performance gate (default: 2.0) |
| `--max-stagnant N` | Give up after N failed improvement attempts (default: 5) |
| `--max-improve-threads N` | Concurrent improvement threads (default: 2) |
| `--max-cycles N` | Stop after N cycles (default: unlimited) |

The positional argument `N` is a shorthand for `--max-cycles N`.

**Example:**

```bash
# Run 100 improvement cycles
iris daemon 100

# Continuous mode with custom thresholds
iris daemon --exec-mode continuous --improve-threshold 1.5 --max-improve-threads 4
```

See the [Evolution & Improvement](/learn/daemon/) guide for details on what the daemon does at each cycle.

---

## iris-lsp {#lsp}

Language server providing diagnostics, hover types, and completion for `.iris` files. Communicates via JSON-RPC over stdio (LSP protocol).

**Capabilities:**

| Feature | Description |
|---------|-------------|
| Diagnostics | Compile errors shown inline as you type |
| Hover | Show inferred types on hover |
| Completion | Keywords, primitives, and import suggestions (triggered by `.`) |

### VS Code

Add to `.vscode/settings.json`:

```json
{
  "iris.serverPath": "/path/to/iris/bootstrap/iris-lsp"
}
```

Or create a custom language configuration in `.vscode/settings.json`:

```json
{
  "[iris]": {
    "editor.tabSize": 2
  },
  "languageServerHaskell.serverExecutablePath": "/path/to/iris/bootstrap/iris-lsp"
}
```

### Neovim (nvim-lspconfig)

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

configs.iris = {
  default_config = {
    cmd = { '/path/to/iris/bootstrap/iris-lsp' },
    filetypes = { 'iris' },
    root_dir = lspconfig.util.root_pattern('.git'),
    settings = {},
  },
}

lspconfig.iris.setup{}
```

### Helix

Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "iris"
scope = "source.iris"
file-types = ["iris"]
comment-token = "--"
language-servers = ["iris-lsp"]

[language-server.iris-lsp]
command = "/path/to/iris/bootstrap/iris-lsp"
```

---

## iris fmt {#fmt}

Format IRIS source files with consistent style. Normalizes indentation, spacing, and newlines.

```bash
iris fmt <file.iris>
```

The formatter handles `let` declarations, type declarations, imports, `match` expressions, `if`/`then`/`else`, lambdas, and mutual recursion groups (`and` blocks).
