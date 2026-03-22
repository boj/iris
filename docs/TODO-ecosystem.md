# IRIS Ecosystem TODO

Gap analysis based on comparison with Haskell and Rust ecosystems (2026-03-28).
Updated with progress from the same day.

## Critical (blocks real-world use)

- [x] **Parametric types** — `Option<T>`, `Result<T, E>`, generic ADTs via monomorphization
- [ ] **Working LSP** — `iris-lsp` is a 5-line stub. Need hover types, go-to-definition, inline errors. (Deferred to last)
- [x] **Wire up CLI stubs** — `iris solve` now invokes evolution engine with `-- test:` annotations
- [x] **Package manifest** — Design spec at `docs/package-manifest-spec.md` (iris.toml, decentralized, BLAKE3-based)
- [x] **`iris store` command** — `iris store list/get/rm/clear/path`

## High Impact (transforms the experience)

- [x] **Deep pattern matching** — Tuple patterns `(a, b)`, guard clauses `when`, multi-arity constructors `Pair(Int, Int)`
- [ ] **Typeclasses/traits** — Design doc at `docs/typeclass-design.md` (dictionary-passing approach). Implementation pending.
- [x] **Formatter** — `iris fmt` skeleton at `syntax/formatter.rs`
- [x] **Linter** — `iris lint` with 5 rules (L001-L005: unused bindings, shadows, complexity, missing types, constant folds)
- [x] **Stateful REPL** — rustyline, `:type`, `:list`, `:clear`, `:load`, history
- [x] **Error message suggestions** — Did-you-mean via Levenshtein, `iris explain E001-E006`
- [x] **Sort with comparator** — `sort_by` primitive (opcode 0xCF)

## Medium Impact (polish and depth)

- [x] **JSON full support** — `json_full.iris` with tagged-tuple encoding (in progress via agent)
- [x] **Property-based testing** — `quickcheck.iris` with LCG random gen + shrinking (in progress via agent)
- [x] **Async/concurrency stdlib** — `async_ops.iris`: parallel, race, timeout, channel, parallel_map/fold
- [x] **Watch mode** — `iris run --watch` polls mtime every 500ms
- [x] **Debugger/profiler** — `debug.iris`: time_it, bench, trace, counted, compare_impls
- [x] **Mutual recursion** — `let rec f ... and g ...` with `and` keyword
- [x] **Type aliases** — `type UserId = Int` (structural, recursive expansion)
- [x] **Multi-arity constructors** — `Pair(Int, Int)` in type decls and patterns

## Documentation

- [x] **Error catalog** — `iris explain E001-E006`
- [x] **Contribution guide** — `docs/CONTRIBUTING.md`
- [x] **Fix `iris store` doc gap** — `iris store` command now exists
- [x] **Per-primitive API reference** — 139 primitives documented at `site/content/learn/primitives.md`
- [ ] **Interactive examples/playground** — Static Markdown only today

## Remaining

- [ ] **Working LSP** — Deferred. Needs `tower-lsp` dependency and full language server.
- [ ] **Typeclasses implementation** — Design done. Dictionary-passing compilation needed.
- [ ] **Interactive playground** — Web-based REPL/sandbox.

## What IRIS Already Does Better

Not TODOs -- genuine differentiators to preserve and build on:

- Content-addressed everything via BLAKE3 (FragmentId dedup/caching/distribution)
- Self-improvement built in (--improve daemon, evolution engine, performance gates, persistent fragment cache)
- Native codegen from high-level language (AVX + GP integer register allocation)
- Verification as first-class (refinement types, LCF proof kernel, graded tiers)
- Programs as data (graph introspection, graph_eval, self-modification with static types)
