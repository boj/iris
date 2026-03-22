---
title: "Primitive Reference"
description: "Complete reference for every IRIS primitive function."
weight: 95
---

This is the definitive reference for every primitive operation available in the IRIS bootstrap evaluator. Each primitive maps to a fixed opcode and arity, resolved at compile time from the name table in `src/iris-bootstrap/src/syntax/prim.rs`.

**Numeric coercion rules** apply across arithmetic, comparison, and math primitives:
- If either operand is `Float64`, the other is promoted to `Float64` before the operation.
- `Bool` values coerce to `Int` (`true` = 1, `false` = 0) for arithmetic and bitwise ops.
- `Int` arithmetic uses wrapping (two's complement) semantics on i64.

---

## Arithmetic (0x00--0x09)

### `add` {#add}
**Opcode:** `0x00` | **Arity:** 2

Addition. Supports `Int`, `Float64`, and mixed operands (promotes to `Float64`).

```iris
let result = add 3 4
```

**Returns:** `Int` if both operands are `Int`, `Float64` if either is `Float64`.
**Errors:** TypeError if operands are not numeric.

---

### `sub` {#sub}
**Opcode:** `0x01` | **Arity:** 2

Subtraction.

```iris
let result = sub 10 3
```

**Returns:** `Int` or `Float64` (same coercion rules as `add`).
**Errors:** TypeError if operands are not numeric.

---

### `mul` {#mul}
**Opcode:** `0x02` | **Arity:** 2

Multiplication.

```iris
let result = mul 6 7
```

**Returns:** `Int` or `Float64` (same coercion rules as `add`).
**Errors:** TypeError if operands are not numeric.

---

### `div` {#div}
**Opcode:** `0x03` | **Arity:** 2

Division. Integer division truncates toward zero (wrapping_div). Float division produces `Float64`.

```iris
let result = div 10 3
```

**Returns:** `Int` (truncated) or `Float64`.
**Errors:** DivisionByZero if the divisor is 0 (or 0.0). TypeError if operands are not numeric.

---

### `mod` {#mod}
**Opcode:** `0x04` | **Arity:** 2

Remainder (modulo). Uses wrapping_rem for integers, `%` for floats.

```iris
let result = mod 10 3
```

**Returns:** `Int` or `Float64`.
**Errors:** DivisionByZero if the divisor is 0. TypeError if operands are not numeric.

---

### `neg` {#neg}
**Opcode:** `0x05` | **Arity:** 1

Arithmetic negation.

```iris
let result = neg 42
```

**Returns:** `Int` or `Float64` (same type as input).
**Errors:** TypeError if operand is not `Int` or `Float64`.

---

### `abs` {#abs}
**Opcode:** `0x06` | **Arity:** 1

Absolute value.

```iris
let result = abs (neg 42)
```

**Returns:** `Int` or `Float64` (same type as input).
**Errors:** TypeError if operand is not `Int` or `Float64`.

---

### `min` {#min}
**Opcode:** `0x07` | **Arity:** 2

Minimum of two values.

```iris
let result = min 3 7
```

**Returns:** `Int` or `Float64`.
**Errors:** TypeError if operands are not numeric.

---

### `max` {#max}
**Opcode:** `0x08` | **Arity:** 2

Maximum of two values.

```iris
let result = max 3 7
```

**Returns:** `Int` or `Float64`.
**Errors:** TypeError if operands are not numeric.

---

### `pow` {#pow}
**Opcode:** `0x09` | **Arity:** 2

Exponentiation. For integers, uses binary exponentiation with wrapping multiplication. Negative integer exponents return 0. For floats, uses `powf`.

```iris
let result = pow 2 10
```

**Returns:** `Int` (if both Int) or `Float64` (if either Float64).
**Errors:** TypeError if operands are not numeric.

---

## Bitwise (0x10--0x15)

### `bitand` {#bitand}
**Opcode:** `0x10` | **Arity:** 2

Bitwise AND.

```iris
let result = bitand 0xFF 0x0F
```

**Returns:** `Int`.
**Errors:** TypeError if operands are not `Int` (or `Bool`, which coerces).

---

### `bitor` {#bitor}
**Opcode:** `0x11` | **Arity:** 2

Bitwise OR.

```iris
let result = bitor 0xF0 0x0F
```

**Returns:** `Int`.
**Errors:** TypeError if operands are not `Int`.

---

### `bitxor` {#bitxor}
**Opcode:** `0x12` | **Arity:** 2

Bitwise XOR.

```iris
let result = bitxor 0xFF 0x0F
```

**Returns:** `Int`.
**Errors:** TypeError if operands are not `Int`.

---

### `bitnot` {#bitnot}
**Opcode:** `0x13` | **Arity:** 1

Bitwise NOT (one's complement).

```iris
let result = bitnot 0
```

**Returns:** `Int`.
**Errors:** TypeError if operand is not `Int`.

---

### `shl` {#shl}
**Opcode:** `0x14` | **Arity:** 2

Left shift. Uses wrapping semantics (shift amount is masked to low 5 bits of u32).

```iris
let result = shl 1 8
```

**Returns:** `Int`.
**Errors:** TypeError if operands are not `Int`.

---

### `shr` {#shr}
**Opcode:** `0x15` | **Arity:** 2

Arithmetic right shift. Uses wrapping semantics.

```iris
let result = shr 256 4
```

**Returns:** `Int`.
**Errors:** TypeError if operands are not `Int`.

---

## Comparison (0x20--0x25)

All comparison primitives work on `Int`, `Float64`, `Bool`, `String`, `Unit`, and `Tuple` values. `Unit` compares equal to `Int(0)` and empty tuples. Returns `Int(1)` for true, `Int(0)` for false.

### `eq` {#eq}
**Opcode:** `0x20` | **Arity:** 2

Equality test.

```iris
let result = eq x y
```

**Returns:** `Int` (1 if equal, 0 otherwise).

---

### `ne` {#ne}
**Opcode:** `0x21` | **Arity:** 2

Inequality test.

```iris
let result = ne x y
```

**Returns:** `Int` (1 if not equal, 0 otherwise).

---

### `lt` {#lt}
**Opcode:** `0x22` | **Arity:** 2

Less than.

```iris
let result = lt 3 5
```

**Returns:** `Int` (1 or 0).

---

### `gt` {#gt}
**Opcode:** `0x23` | **Arity:** 2

Greater than.

```iris
let result = gt 5 3
```

**Returns:** `Int` (1 or 0).

---

### `le` {#le}
**Opcode:** `0x24` | **Arity:** 2

Less than or equal.

```iris
let result = le 3 3
```

**Returns:** `Int` (1 or 0).

---

### `ge` {#ge}
**Opcode:** `0x25` | **Arity:** 2

Greater than or equal.

```iris
let result = ge 5 3
```

**Returns:** `Int` (1 or 0).

---

## Collection Ops (0x30--0x36)

### `map` {#map}
**Opcode:** `0x30` | **Arity:** 2

Apply a function to each element of a collection. Accepts `Tuple` or `Range` collections. When the function argument is a bare Prim node (operator section), it is applied directly. For binary operator sections applied to scalar elements, the element is duplicated (e.g., `map mul xs` squares each element).

```iris
let doubled = map (\x -> mul x 2) (1, 2, 3)
```

**Returns:** `Tuple` of transformed elements.
**Errors:** TypeError if first argument is not `Tuple` or `Range`.

---

### `filter` {#filter}
**Opcode:** `0x31` | **Arity:** 2

Keep elements for which the predicate returns a truthy value.

```iris
let evens = filter (\x -> eq (mod x 2) 0) (1, 2, 3, 4)
```

**Returns:** `Tuple` of elements passing the predicate.
**Errors:** TypeError if first argument is not `Tuple`.

---

### `zip` {#zip}
**Opcode:** `0x32` | **Arity:** 2

Pair corresponding elements from two collections. Truncates to the shorter length.

```iris
let pairs = zip (1, 2, 3) (10, 20, 30)
```

**Returns:** `Tuple` of 2-element `Tuple` pairs.
**Errors:** TypeError if arguments are not `Tuple` or `Range`.

---

## Type Conversion (0x40--0x44)

### `int_to_float` {#int_to_float}
**Opcode:** `0x40` | **Arity:** 1

Convert `Int` to `Float64`.

```iris
let f = int_to_float 42
```

**Returns:** `Float64`.
**Errors:** TypeError if argument is not `Int`.

---

### `float_to_int` {#float_to_int}
**Opcode:** `0x41` | **Arity:** 1

Truncate `Float64` to `Int` (rounds toward zero).

```iris
let n = float_to_int 3.14
```

**Returns:** `Int`.
**Errors:** TypeError if argument is not `Float64`.

---

### `float_to_bits` {#float_to_bits}
**Opcode:** `0x42` | **Arity:** 1

Reinterpret the IEEE 754 bit pattern of a `Float64` as an `Int` (i64).

```iris
let bits = float_to_bits 1.0
```

**Returns:** `Int` (the raw bit pattern).
**Errors:** TypeError if argument is not `Float64`.

---

### `bits_to_float` {#bits_to_float}
**Opcode:** `0x43` | **Arity:** 1

Reinterpret an `Int` as the IEEE 754 bit pattern of a `Float64`.

```iris
let f = bits_to_float 4607182418800017408
```

**Returns:** `Float64`.
**Errors:** TypeError if argument is not `Int`.

---

### `bool_to_int` {#bool_to_int}
**Opcode:** `0x44` | **Arity:** 1

Convert `Bool` to `Int` (true = 1, false = 0). Also accepts `Int` (identity).

```iris
let n = bool_to_int true
```

**Returns:** `Int`.
**Errors:** TypeError if argument is not `Bool` or `Int`.

---

## State Ops (0x50--0x55)

### `state_get` {#state_get}
**Opcode:** `0x50` | **Arity:** 2

Look up a key in a `State` map. Alias for `map_get`.

```iris
let val = state_get my_state "key"
```

**Returns:** The stored value, or `Unit` if the key is not found.

---

### `state_set` {#state_set}
**Opcode:** `0x51` | **Arity:** 3

Insert or update a key in a `State` map. Alias for `map_insert`.

```iris
let new_state = state_set my_state "key" 42
```

**Returns:** Updated `State` map.

---

### `state_empty` {#state_empty}
**Opcode:** `0x55` | **Arity:** 0

Create an empty `State` map (BTreeMap).

```iris
let s = state_empty
```

**Returns:** Empty `State`.

---

## Graph Introspection (0x60--0x66, 0x80--0x8F, 0x96)

### `graph_get_node_cost` {#graph_get_node_cost}
**Opcode:** `0x60` | **Arity:** 2

Read a node's cost annotation.

```iris
let cost = graph_get_node_cost prog node_id
```

**Returns:** `Int` (0 = Unit, 1 = Inherited, N >= 2 = Annotated constant N).
**Errors:** TypeError if node not found.

---

### `graph_set_node_type` {#graph_set_node_type}
**Opcode:** `0x61` | **Arity:** 3

Set a node's type signature. Type tag encoding: 0 = Int, 1 = Bool, 2 = Float64, 3 = Bytes, 4 = Product, 5 = Unit.

```iris
let prog2 = graph_set_node_type prog node_id 2
```

**Returns:** Modified `Program`.

---

### `graph_get_node_type` {#graph_get_node_type}
**Opcode:** `0x62` | **Arity:** 2

Read a node's type tag.

```iris
let type_tag = graph_get_node_type prog node_id
```

**Returns:** `Int` type tag (0 = Int, 1 = Bool, 2 = Float64, 3 = Bytes, 4 = Product, 5 = Unit, -1 = unknown).

---

### `graph_edges` {#graph_edges}
**Opcode:** `0x63` | **Arity:** 1

Get all edges in a graph as a tuple of (source, target, port) triples.

```iris
let edges = graph_edges prog
```

**Returns:** `Tuple` of 3-element `Tuple` values (source_id: Int, target_id: Int, port: Int).

---

### `graph_get_arity` {#graph_get_arity}
**Opcode:** `0x64` | **Arity:** 2

Get a node's arity (number of expected inputs).

```iris
let a = graph_get_arity prog node_id
```

**Returns:** `Int`.
**Errors:** TypeError if node not found.

---

### `graph_get_depth` {#graph_get_depth}
**Opcode:** `0x65` | **Arity:** 2

Get a node's resolution depth.

```iris
let d = graph_get_depth prog node_id
```

**Returns:** `Int`.
**Errors:** TypeError if node not found.

---

### `graph_get_lit_type_tag` {#graph_get_lit_type_tag}
**Opcode:** `0x66` | **Arity:** 2

Get the type_tag of a Lit node. Returns -1 for non-Lit nodes.

```iris
let tt = graph_get_lit_type_tag prog node_id
```

**Returns:** `Int` (the literal's type_tag byte, or -1).

---

### `self_graph` {#self_graph}
**Opcode:** `0x80` | **Arity:** 0

Return the currently executing program's SemanticGraph as a `Program` value. Enables quine-like self-inspection.

```iris
let me = self_graph
```

**Returns:** `Program`.

---

### `graph_nodes` {#graph_nodes}
**Opcode:** `0x81` | **Arity:** 1

Get all node IDs in a graph, sorted.

```iris
let ids = graph_nodes prog
```

**Returns:** `Tuple` of `Int` node IDs.

---

### `graph_get_kind` {#graph_get_kind}
**Opcode:** `0x82` | **Arity:** 2

Get the NodeKind of a node as an integer.

```iris
let kind = graph_get_kind prog node_id
```

**Returns:** `Int` (NodeKind enum discriminant).
**Errors:** TypeError if node not found.

---

### `graph_get_prim_op` / `graph_get_opcode` {#graph_get_prim_op}
**Opcode:** `0x83` | **Arity:** 2

Get the opcode of a Prim node. `graph_get_opcode` is an alias.

```iris
let op = graph_get_prim_op prog node_id
```

**Returns:** `Int` (the opcode byte).
**Errors:** TypeError if the node is not a Prim node.

---

### `graph_set_prim_op` {#graph_set_prim_op}
**Opcode:** `0x84` | **Arity:** 3

Set the opcode of a Prim node. Removes the old node, updates the payload, recomputes the node ID, and updates all edges. For Effect nodes, sets the effect_tag instead.

```iris
let (prog2, new_id) = graph_set_prim_op prog node_id 0x02
```

**Returns:** `Tuple` of (modified `Program`, new node ID as `Int`).

---

### `graph_add_node_rt` {#graph_add_node_rt}
**Opcode:** `0x85` | **Arity:** 2

Add a new node to a graph at runtime. The second argument is the NodeKind integer:
- `0x00` = Prim, `0x01` = Apply, `0x02` = Lambda, `0x03` = Let, `0x04` = Match,
  `0x05` = Lit, `0x06` = Ref, `0x07` = Neural, `0x08` = Fold, `0x09` = Unfold,
  `0x0A` = Effect, `0x0B` = Tuple, `0x0C` = Inject, `0x0D` = Project,
  `0x0E` = TypeAbst, `0x0F` = TypeApp, `0x10` = LetRec, `0x11` = Guard,
  `0x12` = Rewrite, `0x13` = Extern.

An optional third argument provides an extra opcode (used for Prim nodes).

```iris
let (prog2, node_id) = graph_add_node_rt prog 0
```

**Returns:** `Tuple` of (modified `Program`, new node ID as `Int`).
**Portable pattern:** Create a Prim node with `graph_add_node_rt prog 0`, then set the opcode with `graph_set_prim_op`.

---

### `graph_connect` {#graph_connect}
**Opcode:** `0x86` | **Arity:** 4

Add an edge from source to target at a given port (Argument label).

```iris
let prog2 = graph_connect prog source_id target_id port
```

**Returns:** Modified `Program`.

---

### `graph_disconnect` {#graph_disconnect}
**Opcode:** `0x87` | **Arity:** 3

Remove all edges from source to target.

```iris
let prog2 = graph_disconnect prog source_id target_id
```

**Returns:** Modified `Program`.

---

### `graph_replace_subtree` {#graph_replace_subtree}
**Opcode:** `0x88` | **Arity:** 3

Redirect all edges pointing at `old_id` to point at `new_id` instead. If the root was `old_id`, it becomes `new_id`.

3-argument form: `(prog, old_id, new_id)` -- edge redirection only.

4-argument form: `(target_prog, target_node, source_prog, source_node)` -- copy the subtree rooted at `source_node` from `source_prog` into `target_prog`, replacing `target_node`.

```iris
let prog2 = graph_replace_subtree prog old_id new_id
```

**Returns:** Modified `Program`.

---

### `graph_eval` / `sg_eval` {#graph_eval}
**Opcode:** `0x89` | **Arity:** 2

Evaluate a SemanticGraph (Program) with the given inputs. Creates a sub-evaluator context with its own step budget. `sg_eval` is an alias.

```iris
let result = graph_eval prog (input1, input2)
```

**Returns:** The evaluation result.
**Errors:** RecursionLimit if self-eval depth exceeds 32. TypeError if first argument is not a Program.

---

### `graph_get_root` {#graph_get_root}
**Opcode:** `0x8A` | **Arity:** 1

Get the root node ID of a graph.

```iris
let root = graph_get_root prog
```

**Returns:** `Int` (node ID).

---

### `graph_add_guard_rt` {#graph_add_guard_rt}
**Opcode:** `0x8B` | **Arity:** 4

Add a Guard node to a graph with specified predicate, body, and fallback node IDs. Automatically creates edges from the guard to its children.

```iris
let (prog2, guard_id) = graph_add_guard_rt prog pred_id body_id fallback_id
```

**Returns:** `Tuple` of (modified `Program`, guard node ID as `Int`).

---

### `graph_add_ref_rt` {#graph_add_ref_rt}
**Opcode:** `0x8C` | **Arity:** 2

Add a Ref node to a graph. The integer argument is encoded as the first 8 bytes of the FragmentId.

```iris
let (prog2, ref_id) = graph_add_ref_rt prog frag_int
```

**Returns:** `Tuple` of (modified `Program`, ref node ID as `Int`).

---

### `graph_set_cost` {#graph_set_cost}
**Opcode:** `0x8D` | **Arity:** 3

Set a node's cost annotation. Values: 0 = Unit, 1 = Inherited, N >= 2 = Annotated(Constant(N)).

```iris
let prog2 = graph_set_cost prog node_id 10
```

**Returns:** Modified `Program`.

---

### `graph_get_lit_value` {#graph_get_lit_value}
**Opcode:** `0x8E` | **Arity:** 2

Read the value stored in a Lit node, decoded according to its type_tag.

```iris
let val = graph_get_lit_value prog node_id
```

**Returns:** Decoded value (`Int`, `Float64`, `Bool`, `String`, or `Unit` for unknown tags).
**Errors:** TypeError if the node is not a Lit node.

---

### `graph_outgoing` {#graph_outgoing}
**Opcode:** `0x8F` | **Arity:** 2

Get outgoing edge targets from a node, sorted by port. For Guard nodes, returns (predicate, body, fallback) from the payload directly.

```iris
let children = graph_outgoing prog node_id
```

**Returns:** `Tuple` of `Int` target node IDs.

---

### `graph_edge_count` {#graph_edge_count}
**Opcode:** `0x96` | **Arity:** 1

Count the total number of edges in a graph.

```iris
let n = graph_edge_count prog
```

**Returns:** `Int`.

---

## Knowledge Graph (0x70--0x7B)

Knowledge graph primitives operate on the `Graph(KnowledgeGraph)` value type -- a string-keyed graph with typed nodes, labeled edges, and numeric weights.

### `kg_empty` {#kg_empty}
**Opcode:** `0x70` | **Arity:** 0

Create an empty knowledge graph.

```iris
let kg = kg_empty
```

**Returns:** `Graph` (empty KnowledgeGraph).

---

### `kg_add_node` {#kg_add_node}
**Opcode:** `0x71` | **Arity:** 3

Add a node with an ID and label.

```iris
let kg2 = kg_add_node kg "node1" "Person"
```

**Returns:** Modified `Graph`.

---

### `kg_add_edge` {#kg_add_edge}
**Opcode:** `0x72` | **Arity:** 5

Add a directed edge between two nodes with a type label and weight.

```iris
let kg2 = kg_add_edge kg "alice" "bob" "knows" 1.0
```

**Returns:** Modified `Graph`.

---

### `kg_get_node` {#kg_get_node}
**Opcode:** `0x73` | **Arity:** 2

Look up a node by ID. Returns a `State` map with "id", "label", and any properties.

```iris
let node = kg_get_node kg "node1"
```

**Returns:** `State` map with node fields, or `Unit` if not found.

---

### `kg_neighbors` {#kg_neighbors}
**Opcode:** `0x75` | **Arity:** 3

BFS traversal from a start node up to a maximum depth. Returns all reachable nodes with their depths (follows outgoing edges only).

```iris
let reachable = kg_neighbors kg "start" 3
```

**Returns:** `Tuple` of (node_id: Bytes, depth: Int) pairs.

---

### `kg_set_edge_weight` {#kg_set_edge_weight}
**Opcode:** `0x76` | **Arity:** 4

Update the weight of all edges between two specific nodes.

```iris
let kg2 = kg_set_edge_weight kg "alice" "bob" 2.5
```

**Returns:** Modified `Graph`.

---

### `kg_map_nodes` {#kg_map_nodes}
**Opcode:** `0x77` | **Arity:** 2

Scale a named numeric property on all nodes by a factor. Takes (graph, property_name, factor).

Note: Despite the 2-arg arity in `prim.rs`, the implementation expects 3 arguments.

```iris
let kg2 = kg_map_nodes kg "weight" 2.0
```

**Returns:** Modified `Graph`.

---

### `kg_merge` {#kg_merge}
**Opcode:** `0x78` | **Arity:** 2

Merge two knowledge graphs. Nodes from the second graph overwrite matching IDs in the first; edges are concatenated.

```iris
let merged = kg_merge kg1 kg2
```

**Returns:** Merged `Graph`.

---

### `kg_query_by_edge_type` {#kg_query_by_edge_type}
**Opcode:** `0x79` | **Arity:** 2

Find all targets reachable from a node via edges of a specific type. Takes (graph, source_node_id, edge_type).

Note: Despite the 2-arg arity in `prim.rs`, the implementation expects 3 arguments.

```iris
let targets = kg_query_by_edge_type kg "alice" "knows"
```

**Returns:** `Tuple` of target node IDs as `Bytes`.

---

### `kg_node_count` {#kg_node_count}
**Opcode:** `0x7A` | **Arity:** 1

Count the number of nodes in a knowledge graph.

```iris
let n = kg_node_count kg
```

**Returns:** `Int`.

---

### `kg_edge_count` {#kg_edge_count}
**Opcode:** `0x7B` | **Arity:** 1

Count the number of edges in a knowledge graph.

```iris
let n = kg_edge_count kg
```

**Returns:** `Int`.

---

## Parallel Execution (0x90--0x95)

All parallel primitives currently execute sequentially in the bootstrap evaluator. Semantic parallelism will be introduced later.

### `par_eval` {#par_eval}
**Opcode:** `0x90` | **Arity:** 1

Evaluate a tuple of programs (or a single program). Each `Program` value is evaluated in a sub-context; non-Program values pass through.

```iris
let results = par_eval (prog1, prog2, prog3)
```

**Returns:** `Tuple` of results (one per program), or a single result for a single Program.

---

### `par_map` {#par_map}
**Opcode:** `0x91` | **Arity:** 2

Map a function over tuple elements. Supports operator sections (bare Prim nodes).

```iris
let doubled = par_map (1, 2, 3) (\x -> mul x 2)
```

**Returns:** `Tuple` of transformed elements.
**Errors:** TypeError if the collection is not a `Tuple`.

---

### `par_fold` {#par_fold}
**Opcode:** `0x92` | **Arity:** 3

Fold a binary function over tuple elements with an initial accumulator.

```iris
let total = par_fold (1, 2, 3) 0 add
```

**Returns:** Final accumulator value.
**Errors:** TypeError if the collection is not a `Tuple`.

---

### `spawn` {#spawn}
**Opcode:** `0x93` | **Arity:** 1

Evaluate a program synchronously (future: asynchronous). Non-Program values pass through.

```iris
let result = spawn my_prog
```

**Returns:** Evaluation result.

---

### `await_future` {#await_future}
**Opcode:** `0x94` | **Arity:** 1

Return a future's value. Currently a no-op identity since `spawn` is synchronous.

```iris
let val = await_future future
```

**Returns:** The value as-is.

---

### `par_zip_with` {#par_zip_with}
**Opcode:** `0x95` | **Arity:** 3

Apply a binary function to corresponding pairs from two tuples. Truncates to the shorter length.

```iris
let sums = par_zip_with xs ys add
```

**Returns:** `Tuple` of results.

---

## Evolution (0xA0--0xA1)

### `evolve_subprogram` {#evolve_subprogram}
**Opcode:** `0xA0` | **Arity:** 3

Trigger the evolutionary engine to synthesize a program matching test cases. Requires a MetaEvolver to be provided to the evaluator.

```iris
let prog = evolve_subprogram test_cases max_generations 0
```

**Arguments:**
1. `Tuple` of test cases, each a `Tuple(inputs_tuple, expected_output)`.
2. `Int` max generations (default 100).
3. Reserved (depth hint).

**Returns:** `Program` (the best evolved SemanticGraph).
**Errors:** TypeError if no MetaEvolver is available.

---

## String Ops (0xB0--0xBF)

### `str_len` {#str_len}
**Opcode:** `0xB0` | **Arity:** 1

Character count (not byte count) of a string.

```iris
let n = str_len "hello"
```

**Returns:** `Int`.
**Errors:** TypeError if argument is not `String`.

---

### `str_concat` {#str_concat}
**Opcode:** `0xB1` | **Arity:** 2

Concatenate two strings.

```iris
let s = str_concat "hello" " world"
```

**Returns:** `String`.
**Errors:** TypeError if either argument is not `String`.

---

### `str_slice` {#str_slice}
**Opcode:** `0xB2` | **Arity:** 3

Extract a substring by character indices `[start, end)`. Negative start clamps to 0. Negative end counts from the end.

```iris
let s = str_slice "hello world" 0 5
```

**Returns:** `String`.
**Errors:** TypeError if types are wrong.

---

### `str_contains` {#str_contains}
**Opcode:** `0xB3` | **Arity:** 2

Check if a string contains a substring.

```iris
let found = str_contains "hello world" "world"
```

**Returns:** `Bool`.
**Errors:** TypeError if arguments are not `String`.

---

### `str_split` {#str_split}
**Opcode:** `0xB4` | **Arity:** 2

Split a string by a separator.

```iris
let parts = str_split "a,b,c" ","
```

**Returns:** `Tuple` of `String` parts.
**Errors:** TypeError if arguments are not `String`.

---

### `str_join` {#str_join}
**Opcode:** `0xB5` | **Arity:** 2

Join a tuple of strings with a separator. Non-String elements are silently skipped.

```iris
let s = str_join ("a", "b", "c") ", "
```

**Returns:** `String`.
**Errors:** TypeError if first arg is not `Tuple` or second is not `String`.

---

### `str_to_int` {#str_to_int}
**Opcode:** `0xB6` | **Arity:** 1

Parse a string as an integer. Returns 0 on parse failure.

```iris
let n = str_to_int "42"
```

**Returns:** `Int`.
**Errors:** TypeError if argument is not `String`.

---

### `int_to_string` {#int_to_string}
**Opcode:** `0xB7` | **Arity:** 1

Convert an integer to its decimal string representation.

```iris
let s = int_to_string 42
```

**Returns:** `String`.
**Errors:** TypeError if argument is not `Int`.

---

### `str_eq` {#str_eq}
**Opcode:** `0xB8` | **Arity:** 2

String equality test.

```iris
let same = str_eq "abc" "abc"
```

**Returns:** `Bool`.
**Errors:** TypeError if arguments are not `String`.

---

### `str_starts_with` {#str_starts_with}
**Opcode:** `0xB9` | **Arity:** 2

Check if a string starts with a prefix.

```iris
let yes = str_starts_with "hello" "hel"
```

**Returns:** `Bool`.
**Errors:** TypeError if arguments are not `String`.

---

### `str_ends_with` {#str_ends_with}
**Opcode:** `0xBA` | **Arity:** 2

Check if a string ends with a suffix.

```iris
let yes = str_ends_with "hello" "llo"
```

**Returns:** `Bool`.
**Errors:** TypeError if arguments are not `String`.

---

### `str_replace` {#str_replace}
**Opcode:** `0xBB` | **Arity:** 3

Replace all occurrences of a substring.

```iris
let s = str_replace "hello world" "world" "iris"
```

**Returns:** `String`.
**Errors:** TypeError if arguments are not `String`.

---

### `str_trim` {#str_trim}
**Opcode:** `0xBC` | **Arity:** 1

Remove leading and trailing whitespace.

```iris
let s = str_trim "  hello  "
```

**Returns:** `String`.
**Errors:** TypeError if argument is not `String`.

---

### `str_upper` {#str_upper}
**Opcode:** `0xBD` | **Arity:** 1

Convert to uppercase.

```iris
let s = str_upper "hello"
```

**Returns:** `String`.
**Errors:** TypeError if argument is not `String`.

---

### `str_lower` {#str_lower}
**Opcode:** `0xBE` | **Arity:** 1

Convert to lowercase.

```iris
let s = str_lower "HELLO"
```

**Returns:** `String`.
**Errors:** TypeError if argument is not `String`.

---

### `str_chars` {#str_chars}
**Opcode:** `0xBF` | **Arity:** 1

Split a string into a tuple of single-character strings.

```iris
let chars = str_chars "abc"
```

**Returns:** `Tuple` of single-character `String` values.
**Errors:** TypeError if argument is not `String`.

---

## List/Map Ops (0xC0--0xCF)

### `char_at` {#char_at}
**Opcode:** `0xC0` | **Arity:** 2

Get the Unicode code point of the character at a given index. Returns -1 for out-of-bounds or negative indices.

```iris
let code = char_at "hello" 0
```

**Returns:** `Int` (Unicode code point, or -1).

---

### `list_append` {#list_append}
**Opcode:** `0xC1` | **Arity:** 2

Append a single element to the end of a tuple.

```iris
let extended = list_append (1, 2, 3) 4
```

**Returns:** `Tuple`.
**Errors:** TypeError if first argument is not `Tuple` or `Unit`.

---

### `list_nth` {#list_nth}
**Opcode:** `0xC2` | **Arity:** 2

Get the element at index N. Returns `Unit` for out-of-bounds or negative indices.

```iris
let elem = list_nth (10, 20, 30) 1
```

**Returns:** The element, or `Unit`.
**Errors:** TypeError if first arg is not `Tuple` or `Range`.

---

### `list_take` {#list_take}
**Opcode:** `0xC3` | **Arity:** 2

Take the first N elements. Clamps to available length.

```iris
let first3 = list_take (1, 2, 3, 4, 5) 3
```

**Returns:** `Tuple`.
**Errors:** TypeError if first arg is not `Tuple`.

---

### `list_drop` {#list_drop}
**Opcode:** `0xC4` | **Arity:** 2

Drop the first N elements. Clamps to available length.

```iris
let rest = list_drop (1, 2, 3, 4, 5) 2
```

**Returns:** `Tuple`.
**Errors:** TypeError if first arg is not `Tuple`.

---

### `list_sort` {#list_sort}
**Opcode:** `0xC5` | **Arity:** 1

Sort a tuple of integers in ascending order. Extracts elements coercible to Int, sorts them, and returns Int values.

```iris
let sorted = list_sort (3, 1, 4, 1, 5)
```

**Returns:** `Tuple` of sorted `Int` values.
**Errors:** TypeError if argument is not `Tuple`.

---

### `list_dedup` {#list_dedup}
**Opcode:** `0xC6` | **Arity:** 1

Remove duplicate integers from a tuple (preserves first occurrence order).

```iris
let unique = list_dedup (1, 2, 2, 3, 1)
```

**Returns:** `Tuple` of unique `Int` values.
**Errors:** TypeError if argument is not `Tuple`.

---

### `list_range` {#list_range}
**Opcode:** `0xC7` | **Arity:** 2

Create a lazy range `[start, end)`. Returns a `Range` value (not a materialized tuple). Maximum 100 million elements.

```iris
let r = list_range 0 100
```

**Returns:** `Range`.
**Errors:** TypeError if range exceeds 100M elements.

---

### `map_insert` {#map_insert}
**Opcode:** `0xC8` | **Arity:** 3

Insert a key-value pair into a `State` map. Keys are converted to strings (`Int` formatted as decimal, `String` used as-is).

```iris
let m = map_insert state_empty "name" "iris"
```

**Returns:** `State` map.

---

### `map_get` {#map_get}
**Opcode:** `0xC9` | **Arity:** 2

Look up a key in a `State` map. Returns `Unit` if not found.

```iris
let val = map_get my_map "name"
```

**Returns:** Stored value or `Unit`.

---

### `map_remove` {#map_remove}
**Opcode:** `0xCA` | **Arity:** 2

Remove a key from a `State` map.

```iris
let m2 = map_remove my_map "old_key"
```

**Returns:** Modified `State` map.

---

### `map_keys` {#map_keys}
**Opcode:** `0xCB` | **Arity:** 1

Get all keys from a `State` map (sorted, since it uses BTreeMap).

```iris
let keys = map_keys my_map
```

**Returns:** `Tuple` of `String` keys.

---

### `map_values` {#map_values}
**Opcode:** `0xCC` | **Arity:** 1

Get all values from a `State` map (sorted by key).

```iris
let vals = map_values my_map
```

**Returns:** `Tuple` of values.

---

### `map_size` {#map_size}
**Opcode:** `0xCD` | **Arity:** 1

Get the number of entries in a `State` map. Also works on `Tuple` (returns length).

```iris
let n = map_size my_map
```

**Returns:** `Int`.

---

### `list_concat` {#list_concat}
**Opcode:** `0xCE` | **Arity:** 2

Concatenate two tuples. If the second argument is a scalar, it is appended as a single element. `Unit` values are treated as empty.

```iris
let combined = list_concat (1, 2) (3, 4)
```

**Returns:** `Tuple`.
**Errors:** TypeError if first arg is not `Tuple` or `Unit`.

---

### `sort_by` {#sort_by}
**Opcode:** `0xCF` | **Arity:** 2

Sort a list using a custom comparator function. The comparator takes `(a, b)` and should return a negative `Int` if `a < b`, 0 if equal, positive if `a > b`. `Bool(true)` is treated as -1 (less-than). Uses insertion sort internally.

```iris
let sorted = sort_by (\pair -> sub (list_nth pair 0) (list_nth pair 1)) my_list
```

**Returns:** `Tuple` of sorted elements.
**Errors:** TypeError if second arg is not `Tuple`.

---

## Data Access (0xD2--0xD6)

### `tuple_get` {#tuple_get}
**Opcode:** `0xD2` | **Arity:** 2

Extract a field from a tuple by string key (associative-list lookup) or integer index.

For string keys, searches through tuple entries looking for 2-element tuples where the first element is a matching `String`.
For integer indices, returns the element at that position (returns `Int(0)` for out-of-bounds).
Also works on `Range` values.

```iris
let val = tuple_get my_assoc_list "key"
let elem = tuple_get my_tuple 2
```

**Returns:** Found value, `Unit` (for missing keys), or `Int(0)` (for out-of-bounds indices).

---

### Evaluator-internal opcodes (0xD3--0xD6)

The following opcodes are implemented in the bootstrap evaluator but do not have name bindings in `prim.rs`. They are accessible from IRIS programs when defined as user functions or through stdlib wrappers:

- `0xD3` **str_from_chars** (arity 1) -- Convert a tuple of integer char codes to a String.
- `0xD4` **is_unit** (arity 1) -- Test whether a value is Unit. Returns Bool.
- `0xD5` **type_of** (arity 1) -- Return an integer tag identifying the runtime type (0=Int, 1=Float64, 2=Bool, 3=String, 4=Tuple, 5=Unit, 6=State, 7=Graph, 8=Program, 9=Thunk, 10=Bytes, 11=Range, 12=Tagged, 13=Nat, 14=Float32, 15=Future).
- `0xD6` **str_index_of** (arity 2) -- Find first occurrence of substring; returns char index or -1.

---

## Math (0xD8--0xE3)

All math primitives accept `Int` or `Float64` (auto-coerced to f64) and return `Float64` unless otherwise noted.

### `math_sqrt` {#math_sqrt}
**Opcode:** `0xD8` | **Arity:** 1

Square root.

```iris
let r = math_sqrt 2.0
```

**Returns:** `Float64`.
**Errors:** TypeError if argument is not numeric.

---

### `math_log` {#math_log}
**Opcode:** `0xD9` | **Arity:** 1

Natural logarithm (ln).

```iris
let l = math_log 2.718281828
```

**Returns:** `Float64`.
**Errors:** TypeError if argument is not numeric.

---

### `math_exp` {#math_exp}
**Opcode:** `0xDA` | **Arity:** 1

Exponential (e^x).

```iris
let e = math_exp 1.0
```

**Returns:** `Float64`.
**Errors:** TypeError if argument is not numeric.

---

### `math_sin` {#math_sin}
**Opcode:** `0xDB` | **Arity:** 1

Sine (radians).

```iris
let s = math_sin 3.14159
```

**Returns:** `Float64`.
**Errors:** TypeError if argument is not numeric.

---

### `math_cos` {#math_cos}
**Opcode:** `0xDC` | **Arity:** 1

Cosine (radians).

```iris
let c = math_cos 0.0
```

**Returns:** `Float64`.
**Errors:** TypeError if argument is not numeric.

---

### `math_floor` {#math_floor}
**Opcode:** `0xDD` | **Arity:** 1

Floor (round toward negative infinity). Returns `Int`.

```iris
let n = math_floor 3.7
```

**Returns:** `Int`.
**Errors:** TypeError if argument is not numeric.

---

### `math_ceil` {#math_ceil}
**Opcode:** `0xDE` | **Arity:** 1

Ceiling (round toward positive infinity). Returns `Int`.

```iris
let n = math_ceil 3.2
```

**Returns:** `Int`.
**Errors:** TypeError if argument is not numeric.

---

### `math_round` {#math_round}
**Opcode:** `0xDF` | **Arity:** 1

Round to nearest integer (half-away-from-zero). Returns `Int`.

```iris
let n = math_round 3.5
```

**Returns:** `Int`.
**Errors:** TypeError if argument is not numeric.

---

### `math_pi` {#math_pi}
**Opcode:** `0xE0` | **Arity:** 0

The constant pi (3.14159...).

```iris
let pi = math_pi
```

**Returns:** `Float64`.

---

### `math_e` {#math_e}
**Opcode:** `0xE1` | **Arity:** 0

The constant e (2.71828...).

```iris
let e = math_e
```

**Returns:** `Float64`.

---

### `random_int` {#random_int}
**Opcode:** `0xE2` | **Arity:** 2

Generate a pseudo-random integer in `[min, max]` (inclusive). Uses an xorshift PRNG seeded from the evaluator's step count (deterministic within an evaluation).

```iris
let n = random_int 1 100
```

**Returns:** `Int`.
**Errors:** TypeError if arguments are not `Int`. Returns `min` if `min > max`.

---

### `random_float` {#random_float}
**Opcode:** `0xE3` | **Arity:** 0

Generate a pseudo-random float in `[0.0, 1.0)`.

```iris
let f = random_float
```

**Returns:** `Float64`.

---

## Bytes (0xE6--0xE8)

### `bytes_from_ints` {#bytes_from_ints}
**Opcode:** `0xE6` | **Arity:** 1

Convert a tuple of integers (or nested cons-list of integers) to a `Bytes` value. Each integer is truncated to a single byte (u8). `Unit` produces empty bytes.

```iris
let b = bytes_from_ints (72, 101, 108, 108, 111)
```

**Returns:** `Bytes`.
**Errors:** TypeError if argument is not `Tuple`, `Int`, or `Unit`.

---

### `bytes_concat` {#bytes_concat}
**Opcode:** `0xE7` | **Arity:** 2

Concatenate two `Bytes` values.

```iris
let combined = bytes_concat b1 b2
```

**Returns:** `Bytes`.
**Errors:** TypeError if arguments are not `Bytes`.

---

### `bytes_len` {#bytes_len}
**Opcode:** `0xE8` | **Arity:** 1

Get the byte length of a `Bytes` value.

```iris
let n = bytes_len my_bytes
```

**Returns:** `Int`.
**Errors:** TypeError if argument is not `Bytes`.

---

## Lazy Lists (0xE9--0xEC)

### `lazy_unfold` {#lazy_unfold}
**Opcode:** `0xE9` | **Arity:** 2

Create a lazy stream from a seed and step function. The step function takes a seed and returns `(element, next_seed)`.

```iris
let stream = lazy_unfold 0 (\n -> (n, add n 1))
```

**Returns:** `Thunk` (lazy stream).

---

### `thunk_force` {#thunk_force}
**Opcode:** `0xEA` | **Arity:** 1

Force (eagerly evaluate) a thunk. In the bootstrap, this is identity for non-thunk values.

```iris
let val = thunk_force my_thunk
```

**Returns:** The forced value.

---

### `lazy_take` {#lazy_take}
**Opcode:** `0xEB` | **Arity:** 2

Take the first N elements from a lazy stream by repeatedly applying the step function. Also works on `Tuple` (takes first N elements).

```iris
let first10 = lazy_take stream 10
```

**Returns:** `Tuple` of materialized elements.
**Errors:** TypeError if stream is not `Thunk` or `Tuple`.

---

### `lazy_map` {#lazy_map}
**Opcode:** `0xEC` | **Arity:** 2

Map a function over a lazy stream. In the bootstrap, this eagerly materializes `Tuple` inputs.

```iris
let doubled = lazy_map stream (\x -> mul x 2)
```

**Returns:** `Tuple` (for materialized streams) or `Thunk`.

---

## Graph Construction (0xED--0xEF)

### `graph_new` {#graph_new}
**Opcode:** `0xED` | **Arity:** 0

Create a new empty SemanticGraph with a default Prim(add) root node.

```iris
let pg = graph_new
```

**Returns:** `Program`.

---

### `graph_set_root` {#graph_set_root}
**Opcode:** `0xEE` | **Arity:** 2

Set the root node of a graph.

```iris
let pg2 = graph_set_root pg node_id
```

**Returns:** Modified `Program`.

---

### `graph_set_lit_value` {#graph_set_lit_value}
**Opcode:** `0xEF` | **Arity:** 4

Set the value of a Lit node. Removes the old node, updates the literal payload, recomputes the node ID, and updates all edges.

Type tags: `0x00` = Int (8 bytes LE), `0x04` = Bool (1 byte), `0x07` = String (UTF-8 bytes), `0xFF` = InputRef (1 byte index).

```iris
let (pg2, new_id) = graph_set_lit_value pg node_id 0x00 42
```

**Returns:** `Tuple` of (modified `Program`, new node ID as `Int`).

---

## I/O and Substrate (0xF0--0xF8)

### `list_len` {#list_len}
**Opcode:** `0xF0` | **Arity:** 1

Get the length of a `Tuple`, `Range`, or `String`. For `Range(s, e)`, returns `max(0, e - s)`. For `String`, returns byte length.

```iris
let n = list_len (1, 2, 3)
```

**Returns:** `Int`.
**Errors:** TypeError if argument is not `Tuple`, `Range`, or `String`.

---

### `graph_set_field_index` {#graph_set_field_index}
**Opcode:** `0xF1` | **Arity:** 3

Set the `field_index` on a Project node. Converts the node's payload to `Project { field_index }`, recomputes its ID, and updates edges.

```iris
let (pg2, new_id) = graph_set_field_index pg node_id 2
```

**Returns:** `Tuple` of (modified `Program`, new node ID as `Int`).

---

### `file_read` {#file_read}
**Opcode:** `0xF2` | **Arity:** 1

Read a file's contents as a string.

```iris
let contents = file_read "path/to/file.iris"
```

**Returns:** `String`.
**Errors:** TypeError if the file cannot be read.

---

### `compile_source` {#compile_source}
**Opcode:** `0xF3` | **Arity:** 1

Compile IRIS source code into a cached module. Returns a module ID and metadata about each binding. Requires the `syntax` feature.

```iris
let (module_id, entries) = compile_source "let x = 42"
```

**Returns:** `Tuple` of (module_id: `Int`, entries: `Tuple` of (name: `String`, num_inputs: `Int`)).
**Errors:** TypeError with compilation errors if the source is invalid.

---

### `debug_print` {#debug_print}
**Opcode:** `0xF4` | **Arity:** 1

Print a value to stderr. Bypasses the effect system. `String` and `Int` values are printed directly; other types use debug formatting.

```iris
let _ = debug_print "checkpoint reached"
```

**Returns:** `Unit`.

---

### `module_eval` {#module_eval}
**Opcode:** `0xF5` | **Arity:** 3

Evaluate a specific binding from a compiled module.

```iris
let result = module_eval module_id binding_index (arg1, arg2)
```

**Arguments:**
1. `Int` module_id (from `compile_source`).
2. `Int` binding_index (0-based).
3. Inputs (`Tuple`, `Unit` for no args, or a single value).

**Returns:** Evaluation result.
**Errors:** TypeError for invalid module or binding index. RecursionLimit for excessive nesting.

---

### `compile_test_file` {#compile_test_file}
**Opcode:** `0xF6` | **Arity:** 2

Compile a test file with its dependencies and test harness. Reads `-- depends:` comments from the test file to include dependency files. Requires the `syntax` feature.

```iris
let module_id = compile_test_file "/project/root" "tests/fixtures/test_foo.iris"
```

**Returns:** `Int` module_id.
**Errors:** TypeError if compilation fails or files cannot be read.

---

### `module_test_count` {#module_test_count}
**Opcode:** `0xF7` | **Arity:** 1

Count the number of test bindings (names starting with `test_` and zero inputs) in a compiled module.

```iris
let count = module_test_count module_id
```

**Returns:** `Int`.
**Errors:** TypeError for invalid module_id.

---

### `module_eval_test` {#module_eval_test}
**Opcode:** `0xF8` | **Arity:** 2

Evaluate the Nth test binding from a compiled module (0-indexed among test_ bindings with 0 inputs).

```iris
let result = module_eval_test module_id 0
```

**Returns:** Evaluation result.
**Errors:** TypeError for invalid indices. RecursionLimit for excessive nesting.

---

## Aliases

The following names are aliases for existing opcodes:

| Alias | Canonical | Opcode |
|-------|-----------|--------|
| `sg_eval` | `graph_eval` | `0x89` |
| `graph_get_opcode` | `graph_get_prim_op` | `0x83` |

---

## Opcode Summary Table

| Opcode | Name | Arity | Category |
|--------|------|-------|----------|
| `0x00` | `add` | 2 | Arithmetic |
| `0x01` | `sub` | 2 | Arithmetic |
| `0x02` | `mul` | 2 | Arithmetic |
| `0x03` | `div` | 2 | Arithmetic |
| `0x04` | `mod` | 2 | Arithmetic |
| `0x05` | `neg` | 1 | Arithmetic |
| `0x06` | `abs` | 1 | Arithmetic |
| `0x07` | `min` | 2 | Arithmetic |
| `0x08` | `max` | 2 | Arithmetic |
| `0x09` | `pow` | 2 | Arithmetic |
| `0x10` | `bitand` | 2 | Bitwise |
| `0x11` | `bitor` | 2 | Bitwise |
| `0x12` | `bitxor` | 2 | Bitwise |
| `0x13` | `bitnot` | 1 | Bitwise |
| `0x14` | `shl` | 2 | Bitwise |
| `0x15` | `shr` | 2 | Bitwise |
| `0x20` | `eq` | 2 | Comparison |
| `0x21` | `ne` | 2 | Comparison |
| `0x22` | `lt` | 2 | Comparison |
| `0x23` | `gt` | 2 | Comparison |
| `0x24` | `le` | 2 | Comparison |
| `0x25` | `ge` | 2 | Comparison |
| `0x30` | `map` | 2 | Collection |
| `0x31` | `filter` | 2 | Collection |
| `0x32` | `zip` | 2 | Collection |
| `0x40` | `int_to_float` | 1 | Type Conversion |
| `0x41` | `float_to_int` | 1 | Type Conversion |
| `0x42` | `float_to_bits` | 1 | Type Conversion |
| `0x43` | `bits_to_float` | 1 | Type Conversion |
| `0x44` | `bool_to_int` | 1 | Type Conversion |
| `0x50` | `state_get` | 2 | State |
| `0x51` | `state_set` | 3 | State |
| `0x55` | `state_empty` | 0 | State |
| `0x60` | `graph_get_node_cost` | 2 | Graph |
| `0x61` | `graph_set_node_type` | 3 | Graph |
| `0x62` | `graph_get_node_type` | 2 | Graph |
| `0x63` | `graph_edges` | 1 | Graph |
| `0x64` | `graph_get_arity` | 2 | Graph |
| `0x65` | `graph_get_depth` | 2 | Graph |
| `0x66` | `graph_get_lit_type_tag` | 2 | Graph |
| `0x70` | `kg_empty` | 0 | Knowledge Graph |
| `0x71` | `kg_add_node` | 3 | Knowledge Graph |
| `0x72` | `kg_add_edge` | 5 | Knowledge Graph |
| `0x73` | `kg_get_node` | 2 | Knowledge Graph |
| `0x75` | `kg_neighbors` | 3 | Knowledge Graph |
| `0x76` | `kg_set_edge_weight` | 4 | Knowledge Graph |
| `0x77` | `kg_map_nodes` | 2 | Knowledge Graph |
| `0x78` | `kg_merge` | 2 | Knowledge Graph |
| `0x79` | `kg_query_by_edge_type` | 2 | Knowledge Graph |
| `0x7A` | `kg_node_count` | 1 | Knowledge Graph |
| `0x7B` | `kg_edge_count` | 1 | Knowledge Graph |
| `0x80` | `self_graph` | 0 | Graph |
| `0x81` | `graph_nodes` | 1 | Graph |
| `0x82` | `graph_get_kind` | 2 | Graph |
| `0x83` | `graph_get_prim_op` | 2 | Graph |
| `0x84` | `graph_set_prim_op` | 3 | Graph |
| `0x85` | `graph_add_node_rt` | 2 | Graph |
| `0x86` | `graph_connect` | 4 | Graph |
| `0x87` | `graph_disconnect` | 3 | Graph |
| `0x88` | `graph_replace_subtree` | 3 | Graph |
| `0x89` | `graph_eval` | 2 | Graph |
| `0x8A` | `graph_get_root` | 1 | Graph |
| `0x8B` | `graph_add_guard_rt` | 4 | Graph |
| `0x8C` | `graph_add_ref_rt` | 2 | Graph |
| `0x8D` | `graph_set_cost` | 3 | Graph |
| `0x8E` | `graph_get_lit_value` | 2 | Graph |
| `0x8F` | `graph_outgoing` | 2 | Graph |
| `0x90` | `par_eval` | 1 | Parallel |
| `0x91` | `par_map` | 2 | Parallel |
| `0x92` | `par_fold` | 3 | Parallel |
| `0x93` | `spawn` | 1 | Parallel |
| `0x94` | `await_future` | 1 | Parallel |
| `0x95` | `par_zip_with` | 3 | Parallel |
| `0x96` | `graph_edge_count` | 1 | Graph |
| `0xA0` | `evolve_subprogram` | 3 | Evolution |
| `0xB0` | `str_len` | 1 | String |
| `0xB1` | `str_concat` | 2 | String |
| `0xB2` | `str_slice` | 3 | String |
| `0xB3` | `str_contains` | 2 | String |
| `0xB4` | `str_split` | 2 | String |
| `0xB5` | `str_join` | 2 | String |
| `0xB6` | `str_to_int` | 1 | String |
| `0xB7` | `int_to_string` | 1 | String |
| `0xB8` | `str_eq` | 2 | String |
| `0xB9` | `str_starts_with` | 2 | String |
| `0xBA` | `str_ends_with` | 2 | String |
| `0xBB` | `str_replace` | 3 | String |
| `0xBC` | `str_trim` | 1 | String |
| `0xBD` | `str_upper` | 1 | String |
| `0xBE` | `str_lower` | 1 | String |
| `0xBF` | `str_chars` | 1 | String |
| `0xC0` | `char_at` | 2 | List/Map |
| `0xC1` | `list_append` | 2 | List/Map |
| `0xC2` | `list_nth` | 2 | List/Map |
| `0xC3` | `list_take` | 2 | List/Map |
| `0xC4` | `list_drop` | 2 | List/Map |
| `0xC5` | `list_sort` | 1 | List/Map |
| `0xC6` | `list_dedup` | 1 | List/Map |
| `0xC7` | `list_range` | 2 | List/Map |
| `0xC8` | `map_insert` | 3 | List/Map |
| `0xC9` | `map_get` | 2 | List/Map |
| `0xCA` | `map_remove` | 2 | List/Map |
| `0xCB` | `map_keys` | 1 | List/Map |
| `0xCC` | `map_values` | 1 | List/Map |
| `0xCD` | `map_size` | 1 | List/Map |
| `0xCE` | `list_concat` | 2 | List/Map |
| `0xCF` | `sort_by` | 2 | List/Map |
| `0xD2` | `tuple_get` | 2 | Data Access |
| `0xD8` | `math_sqrt` | 1 | Math |
| `0xD9` | `math_log` | 1 | Math |
| `0xDA` | `math_exp` | 1 | Math |
| `0xDB` | `math_sin` | 1 | Math |
| `0xDC` | `math_cos` | 1 | Math |
| `0xDD` | `math_floor` | 1 | Math |
| `0xDE` | `math_ceil` | 1 | Math |
| `0xDF` | `math_round` | 1 | Math |
| `0xE0` | `math_pi` | 0 | Math |
| `0xE1` | `math_e` | 0 | Math |
| `0xE2` | `random_int` | 2 | Math |
| `0xE3` | `random_float` | 0 | Math |
| `0xE6` | `bytes_from_ints` | 1 | Bytes |
| `0xE7` | `bytes_concat` | 2 | Bytes |
| `0xE8` | `bytes_len` | 1 | Bytes |
| `0xE9` | `lazy_unfold` | 2 | Lazy |
| `0xEA` | `thunk_force` | 1 | Lazy |
| `0xEB` | `lazy_take` | 2 | Lazy |
| `0xEC` | `lazy_map` | 2 | Lazy |
| `0xED` | `graph_new` | 0 | Graph Construction |
| `0xEE` | `graph_set_root` | 2 | Graph Construction |
| `0xEF` | `graph_set_lit_value` | 4 | Graph Construction |
| `0xF0` | `list_len` | 1 | I/O and Substrate |
| `0xF1` | `graph_set_field_index` | 3 | I/O and Substrate |
| `0xF2` | `file_read` | 1 | I/O and Substrate |
| `0xF3` | `compile_source` | 1 | I/O and Substrate |
| `0xF4` | `debug_print` | 1 | I/O and Substrate |
| `0xF5` | `module_eval` | 3 | I/O and Substrate |
| `0xF6` | `compile_test_file` | 2 | I/O and Substrate |
| `0xF7` | `module_test_count` | 1 | I/O and Substrate |
| `0xF8` | `module_eval_test` | 2 | I/O and Substrate |
