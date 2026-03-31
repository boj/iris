---
title: "Reference"
description: "Formal grammar, primitive opcodes, effect tags, and type system reference."
weight: 100
---

This is the complete reference for the IRIS language: formal grammar, every primitive opcode, effect tags, and the type system.

## Grammar {#grammar}

The IRIS surface syntax is defined in `src/iris-programs/syntax/iris_parser.iris` and `src/iris-programs/syntax/iris_lowerer.iris`, with pre-compiled pipelines in `bootstrap/*.json`.

### Module Structure {#grammar-module}

```
module       ::= capability_decl* item*
item         ::= let_decl | type_decl | import_decl
```

### Capability Declarations {#grammar-capabilities}

```
capability_decl ::= 'allow' '[' cap_entry (',' cap_entry)* ']'
                  | 'deny'  '[' cap_entry (',' cap_entry)* ']'
cap_entry       ::= IDENT STRING_LIT?
```

### Let Declarations {#grammar-let}

```
let_decl     ::= 'let' ['rec'] IDENT param* [':' type] [cost_annot]
                 requires_clause* ensures_clause* '=' expr

param        ::= IDENT
cost_annot   ::= '[' 'cost' ':' cost_expr ']'
requires_clause ::= 'requires' expr
ensures_clause  ::= 'ensures' expr
```

### Type Declarations {#grammar-type-decl}

```
type_decl    ::= 'type' IDENT ['<' IDENT (',' IDENT)* '>'] '=' type
               | 'type' IDENT ['<' IDENT (',' IDENT)* '>'] '=' sum_type
               | 'type' IDENT ['<' IDENT (',' IDENT)* '>'] '=' struct_type

sum_type     ::= variant ('|' variant)*
variant      ::= UPPER_IDENT ['(' type ')']
UPPER_IDENT  ::= [A-Z] [a-zA-Z0-9_]*

struct_type  ::= '{' field (',' field)* '}'
field        ::= IDENT ':' type
```

Sum type declarations automatically bind constructors. `Red` in `type Color = Red | Green | Blue` becomes a value; `Some` in `type Option = Some(Int) | None` becomes a function `Some : Int -> Option`.

Struct type declarations define named-field product types. `type Point = { x: Int, y: Int }` creates a type where `.x` resolves to `.0` and `.y` to `.1` at compile time. Struct types are sugar over tuples.

### Import Declarations {#grammar-import}

```
import_decl  ::= 'import' STRING_LIT 'as' IDENT
               | 'import' HEX_HASH 'as' IDENT
STRING_LIT   ::= '"' [^"]* '"'
HEX_HASH     ::= '#' [0-9a-f]+
```

Path-based imports use a string literal containing a file path, resolved
relative to the importing file. Hash-based imports use a `#`-prefixed BLAKE3
hash to identify an immutable content-addressed fragment. Both forms bind the
imported module's top-level declarations (including ADT constructors) into
the importing scope.

```iris
import "stdlib/option.iris" as Opt    -- path-based
import #abc123def456 as math          -- hash-based
```

### Types {#grammar-types}

```
type         ::= 'forall' IDENT '.' type
               | type_atom '->' type
               | type_atom

type_atom    ::= '(' ')'                          -- Unit
               | '(' type (',' type)+ ')'          -- Tuple
               | '(' type ')'                      -- Parenthesized
               | '{' IDENT ':' type '|' expr '}'   -- Refinement
               | '{' field (',' field)* '}'         -- Struct type
               | IDENT '<' type (',' type)* '>'    -- Parameterized
               | IDENT                              -- Named
```

### Cost Expressions {#grammar-cost}

```
cost_expr    ::= 'Unknown' | 'Zero'
               | 'Const' '(' INT ')'
               | 'Linear' '(' IDENT ')'
               | 'NLogN' '(' IDENT ')'
               | 'Polynomial' '(' IDENT ',' INT ')'
               | 'Sum' '(' cost_expr ',' cost_expr ')'
```

See the [Type System](/learn/type-system/) page for how types, costs, contracts, and effects interact.

### Expressions {#grammar-expr}

Precedence (lowest to highest):

1. `let ... in`, `if ... then ... else`, `match ... with`, `\params -> body`
2. Pipe: `|>`
3. Logical OR: `||`
4. Logical AND: `&&`
5. Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
6. Addition: `+`, `-`
7. Multiplication: `*`, `/`, `%`
8. Unary: `-` (negation), `!` (logical not)
9. Application: `f x y` (juxtaposition)
10. Postfix: `.0`, `.1`, ... (tuple access)
11. Atoms: literals, variables, parenthesized, tuples

```
expr         ::= 'let' IDENT '=' expr 'in' expr
               | 'if' expr 'then' expr 'else' expr
               | 'match' expr 'with' match_arm+
               | '\' IDENT+ '->' expr
               | pipe_expr

pipe_expr    ::= or_expr ('|>' or_expr)*
or_expr      ::= and_expr ('||' and_expr)*
and_expr     ::= cmp_expr ('&&' cmp_expr)*
cmp_expr     ::= add_expr (cmp_op add_expr)?
add_expr     ::= mul_expr (('+' | '-') mul_expr)*
mul_expr     ::= unary_expr (('*' | '/' | '%') unary_expr)*
unary_expr   ::= '-' unary_expr | '!' unary_expr | app_expr
app_expr     ::= postfix_expr atom_expr*
postfix_expr ::= atom_expr ('.' (INT | IDENT))*

atom_expr    ::= INT | FLOAT | STRING | 'true' | 'false' | IDENT
               | '(' ')'                    -- Unit
               | '(' op ')'                 -- Operator section
               | '(' expr (',' expr)+ ')'   -- Tuple
               | '(' expr ')'               -- Parenthesized
               | '{' field_init (',' field_init)* '}'  -- Struct literal

field_init   ::= IDENT '=' expr
```

### Match Arms {#grammar-match}

```
match_arm    ::= '|' pattern '->' pipe_expr
pattern      ::= '_' | IDENT | INT | 'true' | 'false'
               | UPPER_IDENT ['(' pattern ')']    -- Constructor pattern
```

Constructor patterns like `Some(v)` or `None` destructure sum type values bound by `type` declarations. Patterns are tried top-to-bottom; the first matching arm is evaluated.

Match expressions work inside any expression context, including lambda bodies. This is essential for fold callbacks and higher-order functions:

```iris
let result = fold xs 0 (\acc x ->
  match classify x with
  | Positive(n) -> acc + n
  | Negative(n) -> acc - n
  | Zero -> acc)
```

### Operator Sections {#grammar-operator-sections}

```iris
(+)   -- \a b -> a + b
(*)   -- \a b -> a * b
(==)  -- \a b -> a == b
```

## Lexical Elements {#lexical}

### Keywords {#keywords}

```
let  rec  in  val  type  import  as
match  with  if  then  else
forall  true  false
requires  ensures
allow  deny
```

### Operators {#operators}

```
+  -  *  /  %            -- Arithmetic
==  !=  <  >  <=  >=     -- Comparison
&&  ||  !                -- Logical
|>                       -- Pipe
->                       -- Arrow (types and lambdas)
\                        -- Lambda
.                        -- Tuple access
```

### Literals {#literals}

| Form | Type | Examples |
|------|------|---------|
| Decimal integer | `Int` | `0`, `42`, `-7` |
| Decimal float | `Float64` | `3.14`, `0.5`, `1e10` |
| Double-quoted string | `String` | `"hello"`, `"foo\nbar"` |
| Hex hash | `FragmentId` | `#abc123def456` |
| Boolean | `Bool` | `true`, `false` |

## Primitive Opcodes {#opcodes}

All primitives are named functions resolved by the IRIS lowerer (`src/iris-programs/syntax/iris_lowerer.iris`). Each maps to a `(opcode, arity)` pair.

### Arithmetic (0x00--0x09) {#op-arithmetic}

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `add` | 0x00 | 2 | `a + b` |
| `sub` | 0x01 | 2 | `a - b` |
| `mul` | 0x02 | 2 | `a * b` |
| `div` | 0x03 | 2 | `a / b` (integer division, errors on zero) |
| `mod` | 0x04 | 2 | `a % b` |
| `neg` | 0x05 | 1 | `-a` |
| `abs` | 0x06 | 1 | `\|a\|` |
| `min` | 0x07 | 2 | `min(a, b)` |
| `max` | 0x08 | 2 | `max(a, b)` |
| `pow` | 0x09 | 2 | `a^b` |

### Bitwise (0x10--0x15) {#op-bitwise}

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `bitand` | 0x10 | 2 | Bitwise AND |
| `bitor` | 0x11 | 2 | Bitwise OR |
| `bitxor` | 0x12 | 2 | Bitwise XOR |
| `bitnot` | 0x13 | 1 | Bitwise NOT |
| `shl` | 0x14 | 2 | Shift left |
| `shr` | 0x15 | 2 | Shift right |

### Comparison (0x20--0x25) {#op-comparison}

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `eq` | 0x20 | 2 | `a == b` |
| `ne` | 0x21 | 2 | `a != b` |
| `lt` | 0x22 | 2 | `a < b` |
| `gt` | 0x23 | 2 | `a > b` |
| `le` | 0x24 | 2 | `a <= b` |
| `ge` | 0x25 | 2 | `a >= b` |

### Higher-Order (0x30--0x32) {#op-higher-order}

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `map` | 0x30 | 2 | Apply function to each element |
| `filter` | 0x31 | 2 | Keep elements satisfying predicate |
| `zip` | 0x32 | 2 | Pair elements from two collections |

### Conversion (0x40--0x44) {#op-conversion}

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `int_to_float` | 0x40 | 1 | Int to Float64 |
| `float_to_int` | 0x41 | 1 | Float64 to Int (truncate) |
| `bool_to_int` | 0x44 | 1 | Bool to Int (false=0, true=1) |

### Graph Introspection (0x80--0x8D) {#op-graph}

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `self_graph` | 0x80 | 0 | Capture own SemanticGraph as a `Program` value |
| `graph_nodes` | 0x81 | 1 | List all node IDs in a graph |
| `graph_get_kind` | 0x82 | 2 | Get the NodeKind of a node |
| `graph_get_prim_op` | 0x83 | 2 | Get the opcode of a Prim node |
| `graph_set_prim_op` | 0x84 | 3 | Set the opcode of a Prim node (returns modified graph) |
| `graph_add_node_rt` | 0x85 | 2 | Add a new node at runtime |
| `graph_connect` | 0x86 | 4 | Add an edge between two nodes |
| `graph_disconnect` | 0x87 | 3 | Remove an edge |
| `graph_replace_subtree` | 0x88 | 3 | Replace a subtree in the graph |
| `graph_eval` | 0x89 | 2 | Evaluate a graph with given inputs |
| `graph_get_root` | 0x8A | 1 | Get the root NodeId of a graph |
| `graph_add_guard_rt` | 0x8B | 4 | Add a Guard node at runtime |
| `graph_add_ref_rt` | 0x8C | 2 | Add a Ref node at runtime |
| `graph_set_cost` | 0x8D | 3 | Set cost annotation on a node |

### Meta-Evolution (0xA0) {#op-meta}

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `evolve_subprogram` | 0xA0 | 3 | Breed a sub-program satisfying given test cases |

### String Operations (0xB0--0xC0) {#op-string}

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `str_len` | 0xB0 | 1 | String length |
| `str_concat` | 0xB1 | 2 | Concatenate two strings |
| `str_slice` | 0xB2 | 3 | Substring (start, end) |
| `str_contains` | 0xB3 | 2 | Check if string contains substring |
| `str_split` | 0xB4 | 2 | Split string by delimiter |
| `str_join` | 0xB5 | 2 | Join strings with separator |
| `str_to_int` | 0xB6 | 1 | Parse string as integer |
| `int_to_string` | 0xB7 | 1 | Convert integer to string |
| `str_eq` | 0xB8 | 2 | String equality |
| `str_starts_with` | 0xB9 | 2 | Check prefix |
| `str_ends_with` | 0xBA | 2 | Check suffix |
| `str_replace` | 0xBB | 3 | Replace all occurrences |
| `str_trim` | 0xBC | 1 | Trim whitespace |
| `str_upper` | 0xBD | 1 | Convert to uppercase |
| `str_lower` | 0xBE | 1 | Convert to lowercase |
| `str_chars` | 0xBF | 1 | Split into character Tuple |
| `char_at` | 0xC0 | 2 | Get character at index |

### List/Collection Operations (0xC1--0xCD) {#op-list}

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `list_append` | 0xC1 | 2 | Concatenate two lists |
| `list_nth` | 0xC2 | 2 | Get element at index |
| `list_take` | 0xC3 | 2 | First N elements |
| `list_drop` | 0xC4 | 2 | Drop first N elements |
| `list_sort` | 0xC5 | 1 | Sort ascending |
| `list_dedup` | 0xC6 | 1 | Remove consecutive duplicates |
| `list_range` | 0xC7 | 2 | Generate range [start, end) |
| `map_insert` | 0xC8 | 3 | Insert key-value into State map |
| `map_get` | 0xC9 | 2 | Get value by key from State map |
| `map_remove` | 0xCA | 2 | Remove key from State map |
| `map_keys` | 0xCB | 1 | Get all keys as Tuple |
| `map_values` | 0xCC | 1 | Get all values as Tuple |
| `map_size` | 0xCD | 1 | Number of entries in State map |

### Data Access / Introspection (0xCF, 0xD2--0xD6) {#op-data-access}

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `map_contains_key` | 0xCF | 2 | Check if key exists in State map (returns Bool) |
| `tuple_get` | 0xD2 | 2 | Get field from tuple by string key or int index |
| `str_from_chars` | 0xD3 | 1 | Convert Tuple of char codes (Int) to String |
| `is_unit` | 0xD4 | 1 | Check if value is Unit (returns Bool) |
| `type_of` | 0xD5 | 1 | Return Int tag identifying Value variant (0=Int, 1=Float64, 2=Bool, 3=String, 4=Tuple, 5=Unit, 6=State, 7=Graph, 8=Program, 9=Thunk, 10=Bytes, 11=Range) |
| `str_index_of` | 0xD6 | 2 | Find first occurrence of substring, returns char index or -1 |

### Math (0xD8--0xE3) {#op-math}

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `math_sqrt` | 0xD8 | 1 | Square root |
| `math_log` | 0xD9 | 1 | Natural logarithm |
| `math_exp` | 0xDA | 1 | Exponential (e^x) |
| `math_sin` | 0xDB | 1 | Sine |
| `math_cos` | 0xDC | 1 | Cosine |
| `math_floor` | 0xDD | 1 | Floor |
| `math_ceil` | 0xDE | 1 | Ceiling |
| `math_round` | 0xDF | 1 | Round to nearest |
| `math_pi` | 0xE0 | 0 | Pi constant |
| `math_e` | 0xE1 | 0 | Euler's number |
| `random_int` | 0xE2 | 2 | Random integer in [min, max] |
| `random_float` | 0xE3 | 0 | Random float in [0, 1) |

### Lazy Lists (0xE9--0xEC) {#op-lazy}

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `lazy_unfold` | 0xE9 | 2 | Create lazy stream from step function and seed; returns `Thunk` |
| `thunk_force` | 0xEA | 1 | Force one step: returns `(element, next_thunk)` or `Unit` |
| `lazy_take` | 0xEB | 2 | Materialize first N elements from a `Thunk` into a `Tuple` |
| `lazy_map` | 0xEC | 2 | Lazily apply function to each element of a `Thunk` stream |

### Time & Bytes (0xE4--0xE8) {#op-time-bytes}

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `time_format` | 0xE4 | 2 | Format timestamp |
| `time_parse` | 0xE5 | 2 | Parse time string |
| `bytes_from_ints` | 0xE6 | 1 | Convert Tuple of ints to Bytes |
| `bytes_concat` | 0xE7 | 2 | Concatenate two Bytes |
| `bytes_len` | 0xE8 | 1 | Length of Bytes |

## Effect Tags {#effects}

Effects are performed through `Effect` nodes. In surface syntax, effect functions resolve to these tags.

### Standard Effects (0x00--0x0D) {#effect-standard}

| Tag | Name | Signature |
|-----|------|-----------|
| 0x00 | `Print` | `Value -> Unit` |
| 0x01 | `ReadLine` | `() -> Bytes` |
| 0x02 | `HttpGet` | `String -> Bytes` |
| 0x03 | `HttpPost` | `String, Bytes -> Bytes` |
| 0x04 | `FileRead` | `String -> Bytes` |
| 0x05 | `FileWrite` | `String, Bytes -> Unit` |
| 0x06 | `DbQuery` | `String -> Tuple` |
| 0x07 | `DbExecute` | `String -> Int` |
| 0x08 | `Sleep` | `Int -> Unit` |
| 0x09 | `Timestamp` | `() -> Int` |
| 0x0A | `Random` | `() -> Int` |
| 0x0B | `Log` | `String -> Unit` |
| 0x0C | `SendMessage` | `String, Value -> Unit` |
| 0x0D | `RecvMessage` | `String -> Value` |

### Raw I/O (0x10--0x1F) {#effect-io}

| Tag | Name | Signature |
|-----|------|-----------|
| 0x10 | `TcpConnect` | `String, Int -> Int` |
| 0x11 | `TcpRead` | `Int, Int -> Bytes` |
| 0x12 | `TcpWrite` | `Int, Bytes -> Int` |
| 0x13 | `TcpClose` | `Int -> Unit` |
| 0x14 | `TcpListen` | `Int -> Int` |
| 0x15 | `TcpAccept` | `Int -> Int` |
| 0x16 | `FileOpen` | `String, Int -> Int` |
| 0x17 | `FileReadBytes` | `Int, Int -> Bytes` |
| 0x18 | `FileWriteBytes` | `Int, Bytes -> Int` |
| 0x19 | `FileClose` | `Int -> Unit` |
| 0x1A | `FileStat` | `String -> Tuple` |
| 0x1B | `DirList` | `String -> Tuple` |
| 0x1C | `EnvGet` | `String -> String` |
| 0x1D | `ClockNs` | `() -> Int` |
| 0x1E | `RandomBytes` | `Int -> Bytes` |
| 0x1F | `SleepMs` | `Int -> Unit` |

### Threading & Atomics (0x20--0x28) {#effect-threading}

| Tag | Name | Signature |
|-----|------|-----------|
| 0x20 | `ThreadSpawn` | `Program -> Future` |
| 0x21 | `ThreadJoin` | `Future -> Value` |
| 0x22 | `AtomicRead` | `String -> Value` |
| 0x23 | `AtomicWrite` | `String, Value -> Unit` |
| 0x24 | `AtomicSwap` | `String, Value -> Value` |
| 0x25 | `AtomicAdd` | `String, Int -> Value` |
| 0x26 | `RwLockRead` | `String -> Value` |
| 0x27 | `RwLockWrite` | `String, Value -> Unit` |
| 0x28 | `RwLockRelease` | `String -> Unit` |

### JIT & FFI (0x29--0x2B) {#effect-jit-ffi}

| Tag | Name | Signature |
|-----|------|-----------|
| 0x29 | `MmapExec` | `Bytes -> Int` |
| 0x2A | `CallNative` | `Int, Tuple -> Int` |
| 0x2B | `FfiCall` | `String, String, Tuple -> Value` |

## Type System {#type-system}

### Primitive Types {#type-primitives}

| Type | Description |
|------|-------------|
| `Int` | 64-bit signed integer |
| `Nat` | 64-bit unsigned natural number |
| `Float64` | 64-bit IEEE 754 floating point |
| `Float32` | 32-bit IEEE 754 floating point |
| `Bool` | Boolean (true/false) |
| `String` | UTF-8 string |
| `Bytes` | Byte vector |
| `Unit` | Unit type (no value) |

### Composite Types {#type-composite}

| TypeDef | Description |
|---------|-------------|
| `Product(Vec<TypeId>)` | Tuple / struct |
| `Sum(Vec<(Tag, TypeId)>)` | Tagged union |
| `Arrow(TypeId, TypeId, CostBound)` | Function with cost annotation |
| `Recursive(BoundVar, TypeId)` | `mu X. F(X)` recursive type |
| `ForAll(BoundVar, TypeId)` | Polymorphic type |
| `Refined(TypeId, RefinementPredicate)` | Refinement type `{x: T \| P}` |
| `NeuralGuard(TypeId, TypeId, GuardSpec, CostBound)` | Neural computation type |
| `Exists(BoundVar, TypeId)` | Existential type |
| `Vec(TypeId, SizeTerm)` | Sized vector |
| `HWParam(TypeId, HardwareProfile)` | Hardware-parameterized type |

### Algebraic Data Types {#type-adts}

Sum types are declared with `type` and define constructors that produce tagged values:

```
type_decl   ::= 'type' IDENT '=' constructor ('|' constructor)*
constructor ::= IDENT '(' type ')' | IDENT
```

Nullary constructors like `None` or `Red` are values. Constructors with a payload like `Some(Int)` are functions of type `Int -> Option`. Constructors are first-class: they can be passed to higher-order functions, returned from expressions, and used across import boundaries.

```iris
type Option = Some(Int) | None
type Color  = Red | Green | Blue

let colors = (Red, Green, Blue)
let wrapped = map (1, 2, 3) Some   -- (Some(1), Some(2), Some(3))
```

### Struct Types {#type-structs}

Struct types are sugar over product types. A struct declaration assigns field names to tuple positions:

```
struct_type  ::= '{' field (',' field)* '}'
field        ::= IDENT ':' type
```

At compile time, struct construction `{ x = 3, y = 4 }` becomes `Tuple(3, 4)`, and field access `.x` resolves to positional `.0`. The runtime never sees field names, only tuples and integer projections.

```iris
type Point = { x: Int, y: Int }
let p : Point = { x = 10, y = 20 }   -- Tuple(10, 20)
let sum = p.x + p.y                   -- p.0 + p.1 → 30
```

### Runtime Values {#type-values}

| Variant | Description |
|---------|-------------|
| `Int(i64)` | Integer |
| `Nat(u64)` | Natural number |
| `Float64(f64)` | 64-bit float |
| `Float32(f32)` | 32-bit float |
| `Bool(bool)` | Boolean |
| `String(String)` | UTF-8 string |
| `Bytes(Vec<u8>)` | Byte vector |
| `Unit` | Unit value |
| `Tuple(Vec<Value>)` | Tuple / list |
| `Tagged(u16, Box<Value>)` | Tagged variant |
| `State(StateStore)` | Key-value map |
| `Program(Box<SemanticGraph>)` | Reified program (for self-modification) |
| `Thunk(Arc<SemanticGraph>, Box<Value>)` | Lazy stream (step function + state) |
| `Future(FutureHandle)` | Handle to concurrent computation |
