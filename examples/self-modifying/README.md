# Self-Modifying Program (Autopoiesis Demo)

Demonstrates IRIS's core capability: programs that **reify**, **inspect**,
**modify**, and **re-evaluate** their own semantic graph representation.

## How it works

1. `self_graph` (opcode 0x80) captures the running program as a first-class `Program` value
2. `graph_get_root` reads the root node ID
3. `graph_get_prim_op` / `graph_set_prim_op` inspect and modify the operation
4. `graph_eval` evaluates the modified graph with new inputs

## Key functions

- `add_two a b` — a simple addition that we reify and transform
- `read_root_opcode prog` — inspect a program's root operation
- `replace_root_opcode prog new_op` — swap the root operation
- `self_modify_demo a b new_op` — full pipeline: reify → inspect → modify → eval
- `self_modify_swap a b` — concrete demo: transforms add → mul

## Example

```
self_modify_swap(5, 3) = (0, 2, 15)
                          │  │  └─ result of 5 * 3 (modified program)
                          │  └─── new opcode (mul = 2)
                          └────── old opcode (add = 0)
```

This is the foundation of IRIS's autopoiesis: programs that modify themselves
at the graph level, creating the substrate for self-evolving software.

## Running

```bash
cargo test --test test_examples self_modifying -- --nocapture
```
