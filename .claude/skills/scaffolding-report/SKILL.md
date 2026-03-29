---
name: scaffolding-report
description: Audit how much Rust/C scaffolding has been replaced by IRIS equivalents. Use when the user asks about self-hosting progress, scaffolding replacement status, or "how much work until 100% IRIS".
argument-hint: "[crate-name] (optional — audit a single crate instead of all)"
effort: max
---

# Scaffolding Replacement Report

Audit every Rust scaffolding crate to determine what percentage of its **functional behaviors** have been replaced by working, tested IRIS equivalents.

## Important rules

- **Do NOT measure by lines of code.** IRIS programs are SemanticGraphs, not text. LOC is meaningless for them.
- Measure by **functional coverage**: which capabilities/behaviors are implemented in IRIS and passing tests.
- A `.iris` program that returns statistics or counts instead of actually transforming a graph is a **specification**, not a replacement. Note the difference clearly.
- An untested `.iris` program gets flagged — without tests, behavioral equivalence is unverified.

## Scope

If `$ARGUMENTS` names a specific crate (e.g., `iris-evolve`), audit only that crate. Otherwise audit all scaffolding crates.

**Permanent (skip these — not replacement targets):**
- `iris-kernel` — proof kernel (Lob's theorem ceiling)
- `iris-bootstrap` — substrate below Lob ceiling (runs the IRIS programs)
- `iris-clcu` / `iris-clcu-sys` — C hardware layer / FFI bindings

**Scaffolding crates to audit:**
- `iris-syntax` → `src/iris-programs/syntax/` + `bootstrap/*.json`
- `iris-evolve` → `src/iris-programs/evolution/`, `src/iris-programs/seeds/`, `src/iris-programs/analyzer/`, `src/iris-programs/meta/`, `src/iris-programs/population/`, `src/iris-programs/mutation/`
- `iris-exec` (rust-scaffolding gated heavy impl) → `src/iris-programs/interpreter/`, `src/iris-programs/jit/`, `src/iris-programs/vm/`, `src/iris-programs/exec/`
- `iris-codec` → `src/iris-programs/codec/`
- `iris-repr` → `src/iris-programs/repr/`
- `iris-compiler` → `src/iris-programs/compiler/`
- `iris-deploy` → `src/iris-programs/deploy/`
- `iris-lsp` → (check if any equivalent exists)

## Methodology

For each crate, spawn parallel agents (one per crate or group) to:

1. **Read the Rust source** — enumerate every public function, trait impl, and behavior the crate provides
2. **Read the corresponding `.iris` files** — determine which behaviors have IRIS equivalents
3. **Check if the IRIS programs actually transform data** or just return statistics/counts (specification vs replacement)
4. **Check test coverage** — search `tests/` for any test that loads and executes the `.iris` files as programs (not tests of the Rust crate itself)
5. **Identify gaps** — what behaviors have no IRIS equivalent at all

## Output format

### Per-crate summary table

| Crate | Functional Coverage | Tested | Status |
|-------|-------------------|--------|--------|
| iris-syntax | X% | Y/Z behaviors | Brief status |
| ... | ... | ... | ... |

### Per-crate detail

For each crate, list:
- **Replaced**: behaviors with working, tested IRIS equivalents
- **Specified but not replacing**: `.iris` programs that model logic but don't produce real output (return stats, delegate to Rust primitives, etc.)
- **Missing**: behaviors with no IRIS equivalent at all
- **Untested**: `.iris` programs that exist but have no test coverage

### Systemic gaps

Call out any cross-cutting issues blocking replacement (e.g., missing graph mutation primitives, untested programs, delegation to Rust `graph_eval`).

### Critical path

What are the highest-leverage next steps to increase coverage?
