# IRIS Language Reference

## Grammar

The IRIS surface syntax is an ML-like language. This section describes the complete grammar as implemented in `src/iris-bootstrap/src/syntax/parser.rs` and `src/iris-bootstrap/src/syntax/lexer.rs`.

### Module Structure

```
module       ::= capability_decl* item*
item         ::= let_decl | type_decl | import_decl
```

### Capability Declarations

```
capability_decl ::= 'allow' '[' cap_entry (',' cap_entry)* ']'
                  | 'deny'  '[' cap_entry (',' cap_entry)* ']'
cap_entry       ::= IDENT STRING_LIT?
```

Example:
```iris
allow [FileRead, FileWrite "/tmp/*"]
deny [TcpConnect, ThreadSpawn, MmapExec]
```

### Let Declarations

```
let_decl     ::= 'let' ['rec'] IDENT param* [':' type] [cost_annot]
                 requires_clause* ensures_clause* '=' expr

param        ::= IDENT
cost_annot   ::= '[' 'cost' ':' cost_expr ']'
requires_clause ::= 'requires' expr
ensures_clause  ::= 'ensures' expr
```

### Type Declarations

```
type_decl    ::= 'type' IDENT ['<' IDENT (',' IDENT)* '>'] '=' type
```

### Import Declarations

```
import_decl  ::= 'import' HEX_HASH 'as' IDENT
```

### Types

```
type         ::= 'forall' IDENT '.' type
               | type_atom '->' type
               | type_atom

type_atom    ::= '(' ')'                          -- Unit
               | '(' type (',' type)+ ')'          -- Tuple
               | '(' type ')'                      -- Parenthesized
               | '{' IDENT ':' type '|' expr '}'   -- Refinement
               | IDENT '<' type (',' type)* '>'    -- Parameterized
               | IDENT                              -- Named
```

### Cost Expressions

```
cost_expr    ::= 'Unknown' | 'Zero' | 'Unit'
               | 'Const' '(' INT ')'
               | 'Linear' '(' IDENT ')'
               | 'NLogN' '(' IDENT ')'
```

### Expressions

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
postfix_expr ::= atom_expr ('.' INT)*

atom_expr    ::= INT | FLOAT | STRING | 'true' | 'false' | IDENT
               | '(' ')'                    -- Unit
               | '(' op ')'                 -- Operator section
               | '(' expr (',' expr)+ ')'   -- Tuple
               | '(' expr ')'               -- Parenthesized
```

### Match Arms

```
match_arm    ::= '|' pattern '->' pipe_expr
pattern      ::= '_' | IDENT | INT | 'true' | 'false'
```

### Operator Sections

Operator sections wrap binary operators as first-class functions:

```iris
(+)   -- \a b -> a + b
(*)   -- \a b -> a * b
(==)  -- \a b -> a == b
```

### Comments

```
-- This is a line comment (extends to end of line)
```

---

## Lexical Elements

### Keywords (Reserved Words)

```
let  rec  in  val  type  import  as
match  with  if  then  else
forall  true  false
requires  ensures
allow  deny
```

### Operators

```
+  -  *  /  %            -- Arithmetic
==  !=  <  >  <=  >=     -- Comparison
&&  ||  !                -- Logical
|>                       -- Pipe
->                       -- Arrow (types and lambdas)
\                        -- Lambda
.                        -- Tuple access
```

### Punctuation

```
(  )  {  }  [  ]  ,  :  =  _  |
```

### Literals

| Form | Type | Examples |
|------|------|---------|
| Decimal integer | `Int` | `0`, `42`, `-7` |
| Decimal float | `Float64` | `3.14`, `0.5`, `1e10` |
| Double-quoted string | `String` | `"hello"`, `"foo\nbar"` |
| Hex hash | `FragmentId` | `#abc123def456` |
| Boolean | `Bool` | `true`, `false` |

---

## Primitive Operations

All primitives are named functions resolved by `src/iris-bootstrap/src/syntax/prim.rs`. Each primitive maps to a `(opcode, arity)` pair.

### Arithmetic (0x00-0x09)

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `add` | 0x00 | 2 | `a + b` |
| `sub` | 0x01 | 2 | `a - b` |
| `mul` | 0x02 | 2 | `a * b` |
| `div` | 0x03 | 2 | `a / b` (integer division, errors on zero) |
| `mod` | 0x04 | 2 | `a % b` |
| `neg` | 0x05 | 1 | `-a` |
| `abs` | 0x06 | 1 | `|a|` |
| `min` | 0x07 | 2 | `min(a, b)` |
| `max` | 0x08 | 2 | `max(a, b)` |
| `pow` | 0x09 | 2 | `a^b` |

### Bitwise (0x10-0x15)

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `bitand` | 0x10 | 2 | Bitwise AND |
| `bitor` | 0x11 | 2 | Bitwise OR |
| `bitxor` | 0x12 | 2 | Bitwise XOR |
| `bitnot` | 0x13 | 1 | Bitwise NOT |
| `shl` | 0x14 | 2 | Shift left |
| `shr` | 0x15 | 2 | Shift right |

### Comparison (0x20-0x25)

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `eq` | 0x20 | 2 | `a == b` |
| `ne` | 0x21 | 2 | `a != b` |
| `lt` | 0x22 | 2 | `a < b` |
| `gt` | 0x23 | 2 | `a > b` |
| `le` | 0x24 | 2 | `a <= b` |
| `ge` | 0x25 | 2 | `a >= b` |

### Higher-Order (0x30-0x32)

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `map` | 0x30 | 2 | Apply function to each element |
| `filter` | 0x31 | 2 | Keep elements satisfying predicate |
| `zip` | 0x32 | 2 | Pair elements from two collections |

### Conversion (0x40-0x44)

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `int_to_float` | 0x40 | 1 | Int to Float64 |
| `float_to_int` | 0x41 | 1 | Float64 to Int (truncate) |
| `bool_to_int` | 0x44 | 1 | Bool to Int (false=0, true=1) |

### State (0x55)

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `state_empty` | 0x55 | 0 | Create empty `StateStore` |

### Graph Introspection (0x80-0x8D)

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
| `graph_eval` / `sg_eval` | 0x89 | 2 | Evaluate a graph with given inputs |
| `graph_get_root` | 0x8A | 1 | Get the root NodeId of a graph |
| `graph_add_guard_rt` | 0x8B | 4 | Add a Guard node at runtime |
| `graph_add_ref_rt` | 0x8C | 2 | Add a Ref node at runtime |
| `graph_set_cost` | 0x8D | 3 | Set cost annotation on a node |

### Meta-Evolution (0xA0)

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `evolve_subprogram` | 0xA0 | 3 | Breed a sub-program satisfying given test cases |

### String Operations (0xB0-0xC0)

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

### List/Collection Operations (0xC1-0xCD)

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

### Data Access (0xD2)

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `tuple_get` | 0xD2 | 2 | Get field from tuple by string key or int index |

### Math (0xD8-0xE3)

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

### Time (0xE4-0xE5)

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `time_format` | 0xE4 | 2 | Format timestamp |
| `time_parse` | 0xE5 | 2 | Parse time string |

### Bytes (0xE6-0xE8)

| Name | Opcode | Arity | Semantics |
|------|--------|-------|-----------|
| `bytes_from_ints` | 0xE6 | 1 | Convert Tuple of ints to Bytes |
| `bytes_concat` | 0xE7 | 2 | Concatenate two Bytes |
| `bytes_len` | 0xE8 | 1 | Length of Bytes |

---

## Effect Tags

Effects are performed through `Effect` nodes in the SemanticGraph. In surface syntax, effect functions like `print`, `tcp_connect`, etc. are resolved to these tags. Defined in `src/iris-types/src/eval.rs`.

### Standard Effects (0x00-0x0D)

| Tag | Name | Signature | Semantics |
|-----|------|-----------|-----------|
| 0x00 | `Print` | `Value -> Unit` | Output a value |
| 0x01 | `ReadLine` | `() -> Bytes` | Read a line of text |
| 0x02 | `HttpGet` | `String -> Bytes` | GET a URL |
| 0x03 | `HttpPost` | `String, Bytes -> Bytes` | POST to a URL |
| 0x04 | `FileRead` | `String -> Bytes` | Read file contents |
| 0x05 | `FileWrite` | `String, Bytes -> Unit` | Write file contents |
| 0x06 | `DbQuery` | `String -> Tuple` | Execute DB query |
| 0x07 | `DbExecute` | `String -> Int` | Execute DB mutation |
| 0x08 | `Sleep` | `Int -> Unit` | Sleep N milliseconds |
| 0x09 | `Timestamp` | `() -> Int` | Unix timestamp in ms |
| 0x0A | `Random` | `() -> Int` | Random integer |
| 0x0B | `Log` | `String -> Unit` | Log at info level |
| 0x0C | `SendMessage` | `String, Value -> Unit` | Send IPC message |
| 0x0D | `RecvMessage` | `String -> Value` | Receive IPC message |

### Raw I/O Primitives (0x10-0x1F)

| Tag | Name | Signature | Semantics |
|-----|------|-----------|-----------|
| 0x10 | `TcpConnect` | `String, Int -> Int` | Connect to TCP endpoint |
| 0x11 | `TcpRead` | `Int, Int -> Bytes` | Read from TCP connection |
| 0x12 | `TcpWrite` | `Int, Bytes -> Int` | Write to TCP connection |
| 0x13 | `TcpClose` | `Int -> Unit` | Close TCP connection |
| 0x14 | `TcpListen` | `Int -> Int` | Listen on TCP port |
| 0x15 | `TcpAccept` | `Int -> Int` | Accept TCP connection |
| 0x16 | `FileOpen` | `String, Int -> Int` | Open file (mode: 0=R, 1=W, 2=A) |
| 0x17 | `FileReadBytes` | `Int, Int -> Bytes` | Read bytes from file handle |
| 0x18 | `FileWriteBytes` | `Int, Bytes -> Int` | Write bytes to file handle |
| 0x19 | `FileClose` | `Int -> Unit` | Close file handle |
| 0x1A | `FileStat` | `String -> Tuple` | Stat a file path |
| 0x1B | `DirList` | `String -> Tuple` | List directory entries |
| 0x1C | `EnvGet` | `String -> String` | Get environment variable |
| 0x1D | `ClockNs` | `() -> Int` | Nanosecond timestamp |
| 0x1E | `RandomBytes` | `Int -> Bytes` | Generate random bytes |
| 0x1F | `SleepMs` | `Int -> Unit` | Sleep for N milliseconds |

### Threading / Atomic Primitives (0x20-0x28)

| Tag | Name | Signature | Semantics |
|-----|------|-----------|-----------|
| 0x20 | `ThreadSpawn` | `Program -> Future` | Spawn thread |
| 0x21 | `ThreadJoin` | `Future -> Value` | Join thread |
| 0x22 | `AtomicRead` | `String -> Value` | Atomic read from state |
| 0x23 | `AtomicWrite` | `String, Value -> Unit` | Atomic write to state |
| 0x24 | `AtomicSwap` | `String, Value -> Value` | Atomic swap |
| 0x25 | `AtomicAdd` | `String, Int -> Value` | Atomic add |
| 0x26 | `RwLockRead` | `String -> Value` | Reader lock acquire + read |
| 0x27 | `RwLockWrite` | `String, Value -> Unit` | Writer lock acquire + write |
| 0x28 | `RwLockRelease` | `String -> Unit` | Release lock |

### JIT Primitives (0x29-0x2A)

| Tag | Name | Signature | Semantics |
|-----|------|-----------|-----------|
| 0x29 | `MmapExec` | `Bytes -> Int` | Map bytes as executable (W^X) |
| 0x2A | `CallNative` | `Int, Tuple -> Int` | Call native function pointer |

### FFI (0x2B)

| Tag | Name | Signature | Semantics |
|-----|------|-----------|-----------|
| 0x2B | `FfiCall` | `String, String, Tuple -> Value` | Call foreign function via dlopen/dlsym |

---

## Type System

### Primitive Types

Defined in `src/iris-types/src/types.rs`:

| Type | Description |
|------|-------------|
| `Int` | 64-bit signed integer |
| `Nat` | 64-bit unsigned natural number |
| `Float64` | 64-bit IEEE 754 floating point |
| `Float32` | 32-bit IEEE 754 floating point |
| `Bool` | Boolean (true/false) |
| `Bytes` | Byte vector |
| `Unit` | Unit type (no value) |

### Composite Types

| TypeDef Variant | Description |
|-----------------|-------------|
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

### Runtime Values

Defined in `src/iris-types/src/eval.rs`:

| Value Variant | Description |
|---------------|-------------|
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
| `State(StateStore)` | Key-value map (BTreeMap<String, Value>) |
| `Graph(KnowledgeGraph)` | Knowledge graph |
| `Program(Box<SemanticGraph>)` | Reified program (for self-modification) |
| `Future(FutureHandle)` | Handle to concurrent computation |

---

## Cost Bounds

The cost system tracks computational complexity. Defined in `src/iris-types/src/cost.rs`.

### CostBound Variants

| Variant | Meaning |
|---------|---------|
| `Unknown` | No cost information |
| `Zero` | Zero cost |
| `Constant(u64)` | Constant time |
| `Linear(CostVar)` | O(n) in the variable |
| `NLogN(CostVar)` | O(n log n) |
| `Polynomial(CostVar, u32)` | O(n^d) |
| `Sum(CostBound, CostBound)` | Sequential composition |
| `Par(CostBound, CostBound)` | Parallel composition (max) |
| `Mul(CostBound, CostBound)` | Product |
| `Amortized(CostBound, PotentialFn)` | Amortized cost |
| `HWScaled(CostBound, HWParamRef)` | Hardware-specific scaling |
| `Sup(Vec<CostBound>)` | Supremum (upper bound of several) |
| `Inf(Vec<CostBound>)` | Infimum (lower bound of several) |

---

## Verification Tiers

Defined in `src/iris-types/src/proof.rs`:

| Tier | Name | Budget | Description |
|------|------|--------|-------------|
| Tier 0 | Automatic | < 10ms | Quantifier-free LIA, fully decidable |
| Tier 1 | Mostly automatic | < 1s | Bounded quantifiers, induction |
| Tier 2 | Semi-automatic | < 60s | Full FOL + SMT |
| Tier 3 | External | Unbounded | External certificate (user-provided proof) |

The `iris check` command auto-detects the minimum tier needed for each fragment's nodes.

---

## Binary Operator Opcodes

Operators in expressions map to these opcodes (from `src/iris-bootstrap/src/syntax/ast.rs`):

| Operator | Opcode | Symbol |
|----------|--------|--------|
| Add | 0x00 | `+` |
| Sub | 0x01 | `-` |
| Mul | 0x02 | `*` |
| Div | 0x03 | `/` |
| Mod | 0x04 | `%` |
| And | 0x10 | `&&` |
| Or | 0x11 | `\|\|` |
| Eq | 0x20 | `==` |
| Ne | 0x21 | `!=` |
| Lt | 0x22 | `<` |
| Gt | 0x23 | `>` |
| Le | 0x24 | `<=` |
| Ge | 0x25 | `>=` |

### Unary Operator Opcodes

| Operator | Opcode | Symbol |
|----------|--------|--------|
| Neg | 0x05 | `-` (prefix) |
| Not | 0x13 | `!` |
