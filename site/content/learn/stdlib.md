---
title: "Standard Library"
description: "Built-in modules for math, collections, strings, I/O, and more."
weight: 40
---

IRIS ships with a standard library of `.iris` modules. All are implemented in IRIS. Source lives in `src/iris-programs/stdlib/`.

```iris
import "stdlib/option.iris" as Opt
import "stdlib/math.iris" as Math
```

## Data Types

| Module | Description |
|--------|-------------|
| `option.iris` | `Some(x)` / `None` with `map`, `and_then`, `unwrap_or` |
| `result.iris` | `Ok(x)` / `Err(e)` with `map`, `and_then`, `unwrap_or` |
| `either.iris` | `Left(a)` / `Right(b)` for disjoint unions |
| `ordering.iris` | `LT` / `EQ` / `GT` for comparisons |

## Collections

| Module | Description |
|--------|-------------|
| `list_ops.iris` | `reverse`, `sort`, `zip`, `take`, `drop`, `flatten` |
| `map_ops.iris` | Association-list maps: `map_get`, `map_insert`, `map_keys` |
| `set_ops.iris` | Sets via sorted lists: `set_add`, `set_member`, `set_union` |
| `lazy.iris` | Lazy lists: `lazy_unfold`, `lazy_take`, `lazy_map`, `lazy_filter` |

## Strings & Math

| Module | Description |
|--------|-------------|
| `math.iris` | `abs`, `max`, `min`, `pow`, `gcd`, `clamp`, `sign`, `is_prime` |
| `string_ops.iris` | `str_split`, `str_join`, `str_replace`, `str_trim`, `str_contains` |
| `string_utils.iris` | `str_starts_with`, `str_ends_with`, `str_pad_left`, `str_repeat` |
| `constants.iris` | `pi`, `e`, `phi`, `tau`, numeric conversion helpers |

## I/O & Network

| Module | Description |
|--------|-------------|
| `file_ops.iris` | `read_file`, `write_file`, `file_exists`, `list_dir` |
| `http_client.iris` | HTTP GET/POST with headers, body parsing |
| `http_server.iris` | HTTP server: route matching, request/response handling |
| `json.iris` | JSON parser and serializer |
| `json_full.iris` | Extended JSON with nested objects and arrays |
| `path_ops.iris` | Path manipulation: `join`, `dirname`, `basename`, `extension` |
| `time_ops.iris` | `now_ms`, `elapsed_ms`, timing utilities |

## Utilities

| Module | Description |
|--------|-------------|
| `async_ops.iris` | Async/await primitives, `spawn`, `await_result` |
| `tco.iris` | Tail-call optimization helpers for deep recursion |
| `quickcheck.iris` | Property-based testing: generators, shrinking, `check_property` |
| `debug.iris` | `debug_print`, `assert`, `trace` |
| `reader.iris` | Reader monad for configuration threading |
| `writer.iris` | Writer monad for logging and accumulation |

## Compiler Infrastructure

The self-hosted compiler pipeline lives in `src/iris-programs/`:

| Directory | Purpose |
|-----------|---------|
| `syntax/` | Tokenizer, parser, import resolver |
| `compiler/` | AST compiler, native VM, ELF generator |
| `interpreter/` | Self-interpreter and evaluator |
| `checker/` | Proof kernel client, verification |
| `evolution/` | NSGA-II engine, fitness functions |
| `mutation/` | Graph mutation operators |
| `population/` | Population management and selection |
| `repr/` | SemanticGraph, BLAKE3 hashing, serialization |
| `tools/` | REPL, package manager, build tools |
