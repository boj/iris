---
title: "Standard Library"
description: "Built-in modules for math, collections, strings, I/O, and more."
weight: 40
---

IRIS ships with a standard library of 23 modules covering common operations. All are implemented as `.iris` files built on primitive opcodes.

## Math {#math}

`stdlib/math.iris` -- Trigonometric, logarithmic, and utility functions built on opcodes `0xD8`--`0xE3`.

```iris
-- Trigonometric
sin x    cos x    tan x

-- Logarithmic
log x    log10 x    log2 x    exp x

-- Utilities
sqrt x    pow x y    floor x    ceil x    round x
clamp x lo hi    lerp a b t    sigmoid x    tanh x

-- Distance
dist2d x1 y1 x2 y2

-- Constants
pi    e

-- Random
rand_int min max    rand_float

-- Conversions
to_radians deg    to_degrees rad
```

## Collections {#collections}

`stdlib/list_ops.iris` -- List operations built on fold and list primitives.

```iris
-- Access
head xs    tail xs    last xs    init xs    nth xs i

-- Transform
append xs ys    concat xss    reverse xs    flatten xss

-- Slice
take xs n    drop xs n

-- Ordering
sort xs    dedup xs

-- Generate
range start end

-- Aggregate
length xs    list_sum xs    list_product xs

-- Mutation (returns new tuple)
list_set xs i val     -- Replace element at index

-- Higher-order
list_bind xs f        -- Flatmap (monadic bind for lists)
list_return val       -- Wrap value in singleton list
scan_left init f xs   -- Fold collecting all intermediate values
fold_while init f xs  -- Fold with early termination via (continue?, acc) pairs
```

## Maps {#maps}

`stdlib/map_ops.iris` -- Key-value operations.

```iris
map_new              -- Create empty map
map_insert m k v     -- Insert key-value pair
map_get m k          -- Lookup by key
map_remove m k       -- Remove a key
map_contains m k     -- Check key existence (stdlib)
map_contains_key m k -- Check key existence (primitive, works with Int keys)
map_keys m           -- List all keys
map_values m         -- List all values
map_size m           -- Number of entries
map_merge m1 m2      -- Merge two maps
```

## Sets {#sets}

`stdlib/set_ops.iris` -- Sets implemented as sorted, deduplicated tuples. Not primitive opcodes -- built from `list_sort`, `list_dedup`, `filter`, and `fold`.

```iris
set_empty            -- Empty set: ()
set_from_list xs     -- Create set from list (sort + dedup)
set_insert s x       -- Add element
set_contains s x     -- Check membership (returns 1 or 0)
set_remove s x       -- Remove element
set_union a b        -- Union
set_intersect a b    -- Intersection
set_diff a b         -- Difference (elements in a but not b)
set_size s           -- Cardinality
set_is_subset a b    -- All elements of a are in b (returns 1 or 0)
```

## Strings {#strings}

`stdlib/string_ops.iris` + `stdlib/string_utils.iris` -- 17 string primitives plus utilities.

```iris
-- Core
str_len s    str_slice s start end    str_concat a b
str_contains s sub    str_split s delim    str_replace s old new
str_chars s    str_trim s

-- Utilities
reverse s    repeat s n    pad_left s n c    pad_right s n c
is_empty s    is_blank s    index_of s sub    count_occurrences s sub
str_starts_with s prefix    str_ends_with s suffix
str_to_upper s    str_to_lower s

-- Conversion (primitives)
str_from_chars codes      -- Tuple of Int char codes -> String
str_index_of s sub        -- Find substring position (returns -1 if not found)
int_to_string n           -- Int -> String
str_to_int s              -- String -> Int
```

## File I/O {#file-io}

`stdlib/file_ops.iris` -- File and directory operations.

```iris
let fd = file_open path mode    -- mode: "r", "w", "a"
let data = file_read_bytes fd n
file_write_bytes fd data
file_close fd
let info = file_stat path       -- Returns (size, modified_time)
let entries = dir_list path
```

## Paths {#paths}

`stdlib/path_ops.iris` -- Path manipulation.

```iris
path_join a b        -- Join path components
path_dirname p       -- Directory part
path_basename p      -- File name part
path_extension p     -- File extension
path_is_absolute p   -- Check if absolute
path_normalize p     -- Normalize path
```

## Time {#time}

`stdlib/time_ops.iris` -- Time operations.

```iris
let now = clock_ns       -- Current time in nanoseconds
sleep_ms duration        -- Sleep for duration milliseconds
```

## HTTP {#http}

`stdlib/http_client.iris` + `stdlib/http_server.iris` -- HTTP built entirely on TCP primitives.

### Client {#http-client}

```iris
-- Simple GET
let response = http_get "example.com" 80 "/api/data"

-- POST with body
let response = http_post "example.com" 80 "/api" body

-- Parsed responses (status_code, headers, body)
let (status, headers, body) = http_get_parsed host port path

-- Convenience
let body = http_get_body host port path
let status = http_status raw_response
let value = http_get_header raw_response "Content-Type"
```

### Server {#http-server}

```iris
let server = tcp_listen "0.0.0.0" 8080
let conn = tcp_accept server
let request = tcp_read conn 65536
tcp_write conn response
tcp_close conn
```

## Data Access & Introspection {#data-access}

Tuple, list, and runtime type primitives.

```iris
let val = tuple_get obj key      -- Get field from tuple by key
let len = list_len xs            -- Length of a list

-- Type introspection (primitives)
is_unit val                      -- True if value is Unit
type_of val                      -- Int tag: 0=Int, 1=Float64, 2=Bool, 3=String,
                                 --          4=Tuple, 5=Unit, 6=State, 7=Graph,
                                 --          8=Program, 9=Thunk, 10=Bytes, 11=Range
```

## Threading {#threading}

Threading primitives for concurrent programs.

```iris
-- Spawn and join
let handle = thread_spawn fn
let result = thread_join handle

-- Atomics
let val = atomic_read ref
atomic_write ref val
let old = atomic_swap ref new_val
atomic_add ref delta

-- Reader-writer locks
rwlock_read lock
rwlock_write lock
rwlock_release lock
```

## Lazy Lists {#lazy-lists}

`stdlib/lazy.iris` -- Lazy infinite stream combinators built on thunk primitives.

```iris
-- Generators
naturals n           -- Infinite natural numbers starting from n
fibs                 -- Fibonacci sequence: 0, 1, 1, 2, 3, 5, 8, ...
repeat x             -- Infinite stream of constant value x
countdown n          -- Finite stream: n, n-1, ..., 1
powers_of_2          -- Powers of 2: 1, 2, 4, 8, 16, ...

-- Combinators
take n xs            -- Materialize first n elements as a tuple
lmap f xs            -- Lazy map: apply f to each element (lazy)

-- Derived
sum_first_n n        -- Sum of first n natural numbers
```

### Primitives

These built-in primitives power the lazy list module:

```iris
lazy_unfold f seed   -- Create lazy stream from step function and seed
lazy_take n stream   -- Force first n elements into a tuple
lazy_map f stream    -- Transform each element lazily
thunk_force stream   -- Step once: returns (element, next_thunk) or ()
```

## Writer Monad {#writer}

`stdlib/writer.iris` -- Computations that accumulate a log alongside a result. Useful for tracing, auditing, or collecting diagnostics without threading a log parameter.

```iris
-- Core
writer_return val          -- Wrap value with empty log: (val, ())
writer_bind wa f           -- Chain computations, concatenating logs
writer_tell msg            -- Append one message to log
writer_tell_all msgs       -- Append multiple messages

-- Accessors
writer_value wa            -- Extract just the result
writer_log wa              -- Extract just the log
writer_run wa              -- Identity (value is already (result, log))

-- Transform
writer_map wa f            -- Map function over result, keep log
writer_zip_with wa wb f    -- Combine two writers, merge results + logs
```

## Reader Monad {#reader}

`stdlib/reader.iris` -- Computations that read from a shared environment. Useful for dependency injection or configuration access without explicit parameter passing. A reader is a function `env -> result`.

```iris
reader_return val env      -- Ignore env, return val
reader_ask env             -- Return the environment itself
reader_local f comp env    -- Run comp with transformed environment
reader_bind ra f env       -- Chain: run ra, pass result to f, both see same env
reader_map ra f env        -- Map function over result
reader_asks key env        -- Read a specific key from env (if env is a State map)
```

## Either Monad {#either-monad}

`stdlib/either.iris` -- Computations that can fail, with short-circuit error propagation. Encoding: `(0, value)` = success, `(1, error)` = failure.

```iris
-- Construction
either_return val          -- Wrap success: (0, val)
either_fail err            -- Wrap failure: (1, err)

-- Predicates
either_is_ok ea            -- True if success
either_is_err ea           -- True if failure

-- Chaining
either_bind ea f           -- If Ok, apply f; if Err, propagate
either_map ea f            -- Map over success value
either_map_err ea f        -- Map over error value

-- Extraction
either_unwrap_or ea def    -- Get success value or default
either_to_int ea def       -- Get success Int or default

-- Combinators
either_ap ef ea            -- Apply wrapped function to wrapped value
either_flatten eea         -- Flatten nested Either
either_or_else ea fallback -- Try ea; on failure, call fallback with error
```

## Higher-Order Primitives {#higher-order}

Built-in higher-order operations available everywhere (primitive opcodes, no
import required):

```iris
fold init f xs       -- Left fold (catamorphism)
map f xs             -- Transform each element
filter pred xs       -- Keep matching elements
zip xs ys            -- Pair elements
concat xs ys         -- Concatenate sequences
reverse xs           -- Reverse order
```

List operations (also primitive opcodes):

```iris
list_take xs n       -- First n elements
list_drop xs n       -- Skip n elements
list_sort xs         -- Sort ascending (integer comparison)
list_nth xs i        -- Element at index i
list_append xs ys    -- Concatenate two lists
list_range start end -- Generate range [start, end)
```

`flat_map` is available as `list_bind` in `stdlib/list_ops.iris`.

### fold_until {#fold-until}

`fold_until` is a built-in primitive (not an import) that folds with early exit. When the predicate returns true on the accumulator, iteration stops immediately and the current accumulator is returned.

```iris
fold_until pred acc step list
```

| Argument | Description |
|----------|-------------|
| `pred`   | `acc -> Bool` -- stop when this returns true |
| `acc`    | Initial accumulator value |
| `step`   | `acc -> elem -> acc` -- step function |
| `list`   | Collection to fold over |

Unlike `fold_while` (which still traverses all elements, skipping the step function after break), `fold_until` genuinely exits the fold loop early, which matters for large collections.

**Thread-ring example:**

```iris
-- Thread ring: pass a token around N threads until it reaches 0.
-- fold_until exits after ~token iterations instead of running all N*token.
let thread_ring_fast n_threads token =
  let res = fold_until (\state -> state.1 > 0) (token, 0) (\state step ->
    let cur_token = state.0 in
    let winner = state.1 in
    if cur_token == 0 then
      (0, (step % n_threads) + 1)
    else
      (cur_token - 1, 0)
  ) (list_range 0 (n_threads * token)) in
  res.1
```

### sort_by {#sort-by}

`sort_by` is a built-in primitive (opcode `0xCF`, arity 2) that sorts a list using a custom comparator function. The comparator receives `(a, b)` as a pair and should return a negative `Int` if `a < b`, `0` if equal, or a positive `Int` if `a > b`.

```iris
sort_by comparator list
```

```iris
-- Sort integers descending
let desc = sort_by (\pair -> sub (list_nth pair 1) (list_nth pair 0)) (3, 1, 4, 1, 5)
-- desc = (5, 4, 3, 1, 1)

-- Sort by absolute value
let by_abs = sort_by (\pair ->
  let a = if list_nth pair 0 < 0 then 0 - list_nth pair 0 else list_nth pair 0 in
  let b = if list_nth pair 1 < 0 then 0 - list_nth pair 1 else list_nth pair 1 in
  a - b
) (0 - 3, 1, 0 - 5, 2)
-- by_abs = (1, 2, -3, -5)
```

Uses insertion sort internally. `Bool(true)` from the comparator is treated as `-1` (less-than). See the [primitives reference](/learn/primitives/#sort_by) for full details.

### Boolean Operators

`&&` and `||` are **short-circuit**: `a && b` only evaluates `b` when `a` is
truthy, and `a || b` only evaluates `b` when `a` is falsy. This makes guarded
expressions safe:

```iris
-- Safe: list_nth is never called when i is out of bounds
i >= 0 && i < list_len xs && list_nth xs i > 0
```

## JIT Code Generation {#jit}

The JIT runtime (`jit_runtime.iris`, 205 lines) provides x86-64 code generation written entirely in the language itself:

```iris
-- x86 instruction builders (return nested byte tuples)
x86_mov_imm64 reg value   -- MOV reg, imm64 (REX.W + 0xB8)
x86_add_rr src dst         -- ADD dst, src
x86_sub_rr src dst         -- SUB dst, src
x86_imul_rr src dst        -- IMUL dst, src
x86_ret                    -- RET
x86_push reg               -- PUSH reg
x86_pop reg                -- POP reg

-- Compilation pipeline
flatten_code nested_bytes  -- Flatten to Bytes value
jit_compile code_bytes     -- mmap_exec → function handle
jit_call handle args       -- call_native → result

-- High-level constructors
jit_const_fn value         -- Compile a constant-returning function
jit_add_fn                 -- Compile an addition function
jit_mul_fn                 -- Compile a multiplication function
jit_identity_fn            -- Compile an identity function
```

Requires `--features jit`. Capability-gated: sandboxes deny `MmapExec` by default.

## Type Modules {#type-modules}

Four modules in `src/iris-programs/stdlib/` define common algebraic data types with
utility functions. Import them with [path-based imports](/learn/language/#path-imports):

```iris
import "stdlib/option.iris" as Opt
import "stdlib/result.iris" as Res
import "stdlib/either.iris" as E
import "stdlib/ordering.iris" as Ord
```

All top-level `let` and `type` declarations, including ADT constructors, are
propagated into the importing scope.

### Option {#type-option}

`stdlib/option.iris`: safe nullable values.

```iris
type Option<T> = Some(T) | None
```

**Functions:**

```iris
-- Unwrap an Option with a default value
let unwrap_or : Option<T> -> T -> T =
  \opt -> \default ->
    match opt with
    | Some(v) -> v
    | None -> default

-- Map a function over an Option
let option_map : Option<T> -> (T -> U) -> Option<U> =
  \opt -> \f ->
    match opt with
    | Some(v) -> Some(f v)
    | None -> None

-- Chain computations that may fail
let and_then : Option<T> -> (T -> Option<U>) -> Option<U> =
  \opt -> \f ->
    match opt with
    | Some(v) -> f v
    | None -> None

-- Check if an Option contains a value
let is_some : Option<T> -> Bool
-- Check if an Option is empty
let is_none : Option<T> -> Bool

-- Get value or compute from a function
let unwrap_or_else : Option<T> -> (Int -> T) -> T

-- Filter: keep Some only if predicate holds
let option_filter : Option<T> -> (T -> Bool) -> Option<T>

-- Zip two Options with a combining function
let zip_with : Option<T> -> Option<U> -> (T -> U -> V) -> Option<V>
```

**Usage:**

```iris
import "stdlib/option.iris" as Opt

let x = Some 10
let y = None

unwrap_or x 0                        -- 10
unwrap_or y 0                        -- 0
option_map x (\v -> v * 2)           -- Some(20)
and_then x (\v -> if v > 5 then Some(v) else None)  -- Some(10)
option_filter x (\v -> v > 100)     -- None
zip_with x (Some 20) (\a b -> a + b)  -- Some(30)
```



### Result {#type-result}

`stdlib/result.iris`: computations that may succeed or fail.

```iris
type Result<T, E> = Ok(T) | Err(E)
```

**Functions:**

```iris
-- Unwrap a Result with a default value
let result_unwrap_or : Result<T, E> -> T -> T =
  \res -> \default ->
    match res with
    | Ok(v) -> v
    | Err(e) -> default

-- Check if a Result is Ok / Err
let is_ok : Result<T, E> -> Bool
let is_err : Result<T, E> -> Bool

-- Map a function over the Ok value
let map_ok : Result<T, E> -> (T -> U) -> Result<U, E> =
  \res -> \f ->
    match res with
    | Ok(v) -> Ok(f v)
    | Err(e) -> Err(e)

-- Map a function over the Err value
let map_err : Result<T, E> -> (E -> F) -> Result<T, F>

-- Chain computations that may fail
let result_and_then : Result<T, E> -> (T -> Result<U, E>) -> Result<U, E>

-- Extract Ok value or return 0
let ok_or_zero : Result<Int, E> -> Int

-- Unwrap or call a function on the error
let result_unwrap_or_else : Result<T, E> -> (E -> T) -> T
```

**Usage:**

```iris
import "stdlib/result.iris" as Res

let ok = Ok 42
let err = Err 1

result_unwrap_or ok 0                -- 42
map_ok ok (\v -> v + 1)              -- Ok(43)
map_err err (\e -> e + 100)          -- Err(101)
result_and_then ok (\v -> if v > 0 then Ok(v * 2) else Err(0))  -- Ok(84)
ok_or_zero err                       -- 0
result_unwrap_or_else err (\e -> e * 10)  -- 10
```



### Either (ADT) {#type-either}

`stdlib/either.iris`: values that can be one of two types. This is the ADT
version with `Left`/`Right` constructors and pattern matching. For the monadic
version with `(0, val)`/`(1, err)` tuple encoding and `either_bind`/`either_map`,
see [Either Monad](#either-monad) above.

```iris
type Either<A, B> = Left(A) | Right(B)
```

**Functions:**

```iris
-- Apply one of two functions depending on the variant
let either : Either<A, B> -> (A -> C) -> (B -> C) -> C =
  \e -> \on_left -> \on_right ->
    match e with
    | Left(a) -> on_left a
    | Right(b) -> on_right b

-- Predicates
let is_left : Either<A, B> -> Bool
let is_right : Either<A, B> -> Bool

-- Extract with default
let from_left : Either<A, B> -> A -> A
let from_right : Either<A, B> -> B -> B

-- Map over one side
let map_left : Either<A, B> -> (A -> C) -> Either<C, B>
let map_right : Either<A, B> -> (B -> C) -> Either<A, C>

-- Swap Left and Right
let swap : Either<A, B> -> Either<B, A>
```

**Usage:**

```iris
import "stdlib/either.iris" as E

let l = Left 10
let r = Right 20

either l (\x -> x * 2) (\x -> x * 3)  -- 20
from_left l 0                           -- 10
from_right l 0                          -- 0
swap l                                  -- Right(10)
map_right r (\x -> x + 1)              -- Right(21)
```



### Ordering {#type-ordering}

`stdlib/ordering.iris`: comparison results.

```iris
type Ordering = Less | Equal | Greater
```

**Functions:**

```iris
-- Typeclasses for equality and ordering
class Eq<A> where
  eq : A -> A -> Bool

class Ord<A> requires Eq<A> where
  compare : A -> A -> Ordering

-- Built-in instances
instance Eq<Int> where eq = \a b -> a == b
instance Ord<Int> where compare = \a b -> ...

-- Compare two integers
let compare_int : Int -> Int -> Ordering =
  \a -> \b ->
    if a < b then Less
    else if a == b then Equal
    else Greater

-- Predicates
let is_lt : Ordering -> Bool
let is_eq : Ordering -> Bool
let is_gt : Ordering -> Bool
let is_le : Ordering -> Bool   -- Less or Equal
let is_ge : Ordering -> Bool   -- Greater or Equal

-- Reverse an ordering (Less <-> Greater)
let reverse : Ordering -> Ordering

-- Convert to integer: Less -> -1, Equal -> 0, Greater -> 1
let to_int : Ordering -> Int
```

**Usage:**

```iris
import "stdlib/ordering.iris" as Ord

let cmp = compare_int 3 5           -- Less
is_lt cmp                            -- true
is_ge cmp                            -- false
reverse cmp                          -- Greater
to_int (compare_int 7 7)            -- 0
to_int (compare_int 10 3)           -- 1
```



## Async/Concurrency {#async}

`stdlib/async_ops.iris` -- Structured concurrency combinators built on threading and atomic primitives (`thread_spawn`, `thread_join`, `atomic_swap`).

```iris
import "stdlib/async_ops.iris" as Async
```

**Functions:**

```iris
parallel f g                     -- Run f and g concurrently, return (result_f, result_g)
race f g                         -- Run both, return (winner_id, winner_result)
with_timeout ms f                -- Run f with timeout: (1, result) or (0, 0) on timeout
channel_new ()                   -- Create a synchronous channel: (send_fn, recv_fn)
parallel_map f xs                -- Map f over list in parallel (one thread per element)
parallel_fold init combine chunk_fn xs n_chunks
                                 -- Split list into chunks, fold each in parallel, combine
```

**Usage:**

```iris
import "stdlib/async_ops.iris" as Async

-- Run two computations concurrently
let (a, b) = parallel (\u -> expensive_calc 100) (\u -> expensive_calc 200)

-- Race two implementations, take whichever finishes first
let (winner, result) = race (\u -> algo_v1 input) (\u -> algo_v2 input)

-- Timeout after 500ms
let (ok, result) = with_timeout 500 (\u -> slow_computation 42)
if ok > 0 then result else fallback_value

-- Parallel map over a list
let results = parallel_map (\x -> x * x) (1, 2, 3, 4, 5)

-- Message passing via channel
let (send, recv) = channel_new ()
let t = thread_spawn (\u -> send 42)
let value = recv ()    -- value = 42
```

## Debug/Profiling {#debug}

`stdlib/debug.iris` -- Runtime inspection utilities built on `trace_emit` and `clock_ns` effect primitives.

```iris
import "stdlib/debug.iris" as Debug
```

**Functions:**

```iris
time_it f arg              -- Time a single call: (result, elapsed_ns)
bench f arg n              -- Benchmark n iterations: (last_result, avg_ns)
trace label value          -- Emit value to trace log, return value unchanged
trace_tuple label t n      -- Trace each of n elements with indexed labels
debug_assert msg condition -- Emit warning if condition is false, return condition
counted f                  -- Wrap f to count calls: (wrapped_fn, get_count_fn)
profile_fold init step xs  -- Measure fold timing: (result, total_ns, iterations)
compare_impls f1 f2 arg   -- Run both, compare: (result, f1_ns, f2_ns, match?)
```

**Usage:**

```iris
import "stdlib/debug.iris" as Debug

-- Time a single function call
let (result, ns) = time_it (\x -> fibonacci x) 30
-- ns = elapsed nanoseconds

-- Benchmark over 1000 iterations
let (result, avg_ns) = bench (\x -> sort_by (\p -> list_nth p 0 - list_nth p 1) x) data 1000

-- Trace values in a pipeline without changing them
let answer = input
  |> transform
  |> trace "after transform"
  |> finalize

-- Count function calls
let (wrapped_fib, get_count) = counted (\n -> fibonacci n)
let result = wrapped_fib 30
let calls = get_count ()

-- Compare two implementations
let (result, ns1, ns2, same) = compare_impls sort_v1 sort_v2 test_data
-- same = 1 if both return the same result
```

## Property Testing {#property-testing}

`stdlib/quickcheck.iris` -- Property-based testing with deterministic pseudo-random generation (LCG) and shrinking.

```iris
import "stdlib/quickcheck.iris" as QC
```

**Functions:**

```iris
qc_int_range lo hi seed   -- Random Int in [lo, hi]: (value, next_seed)
qc_bool seed              -- Random Bool (0 or 1): (value, next_seed)
qc_list_of gen n seed     -- Generate list of n values: (items, final_seed)
qc_check prop n_trials seed
                           -- Run property n_trials times: (passes, failures, smallest_failing)
qc_shrink_int n            -- Shrink Int toward 0: tuple of smaller candidates
qc_next_seed seed          -- Advance the LCG state by one step
```

All generators return `(value, next_seed)` tuples so they can be chained. The LCG uses the classic glibc formula: `next = (seed * 1103515245 + 12345) % 2147483648`.

**Usage:**

```iris
import "stdlib/quickcheck.iris" as QC

-- Generate random test data
let (val, seed1) = qc_int_range 1 100 42
let (flag, seed2) = qc_bool seed1
let (items, seed3) = qc_list_of (\s -> qc_int_range 0 50 s) 10 seed2

-- Property: addition is commutative
let prop_commutative = \x ->
  let (y, _) = qc_int_range 0 999 x in
  if x + y == y + x then 1 else 0

let (passes, failures, smallest) = qc_check prop_commutative 100 42
-- passes = 100, failures = 0, smallest = -1 (no failures)

-- Property that fails: all numbers less than 50
let prop_small = \x -> if x < 50 then 1 else 0
let (passes, failures, smallest) = qc_check prop_small 100 42
-- failures > 0, smallest = shrunk to value near 50
```

## JSON {#json}

`stdlib/json_full.iris` -- JSON construction, lookup, and serialization. Values are encoded as tagged tuples `(type_tag, payload)` where the tag identifies the JSON type.

```iris
import "stdlib/json_full.iris" as JSON
```

**Constructors:**

```iris
json_null                  -- (0, ())
json_bool b                -- (1, b) where b is 0 or 1
json_int n                 -- (2, n)
json_float f               -- (3, f)
json_string s              -- (4, s)
json_array items           -- (5, items) where items is a tuple of json values
json_object pairs          -- (6, pairs) where pairs is a tuple of (key, json_value)
```

**Operations:**

```iris
json_type jv               -- Return type tag (0-6)
json_get obj key           -- Lookup key in object, returns json_null if missing
json_stringify jv           -- Serialize to JSON string (up to 3 levels of nesting)
json_escape_string s       -- Escape \ and " for JSON output
```

**Usage:**

```iris
import "stdlib/json_full.iris" as JSON

-- Build a JSON object
let user = json_object (
  ("name", json_string "alice"),
  ("age", json_int 30),
  ("active", json_bool 1)
)

json_stringify user
-- {"name":"alice","age":30,"active":true}

-- Nested structures
let response = json_object (
  ("status", json_int 200),
  ("data", json_array (json_int 1, json_int 2, json_int 3))
)

json_stringify response
-- {"status":200,"data":[1,2,3]}

-- Lookup
let name = json_get user "name"
json_stringify name          -- "alice"
json_type name               -- 4 (string)

let missing = json_get user "email"
json_type missing            -- 0 (null)
```

## Constants {#constants}

`stdlib/constants.iris` -- Named type aliases for magic numbers used in the compiler, interpreter, and evolution system. Import this file to replace raw integer comparisons with readable names.

```iris
import "stdlib/constants.iris" as C
```

**Type aliases and constants:**

| Type | Constants | Description |
|------|-----------|-------------|
| `NodeKind` | `nk_prim` (0) .. `nk_extern` (19) | SemanticGraph node type tags (20 kinds) |
| `PrimOp` | `prim_add` (0) .. `prim_shr` (21), `prim_eq` (32) .. `prim_ge` (37) | Primitive opcode tags |
| `ValueTag` | `val_int` (0) .. `val_tagged` (7) | Runtime value type classification |
| `EvalTier` | `tier_a` (0), `tier_b` (1), `tier_c` (2) | Evaluation tier levels |
| `CostBound` | `cost_unknown` (0) .. `cost_polynomial` (5) | Complexity class annotations |
| `EvolutionPhase` | `phase_explore` (0), `phase_refine` (1), `phase_converge` (2) | Evolution stage markers |

**Usage:**

```iris
import "stdlib/constants.iris" as C

-- Check if a node is a lambda
if node_kind == nk_lambda then ...

-- Compare against known opcodes
if op == prim_add then ...
else if op == prim_mul then ...

-- Runtime type dispatch
let tag = type_of val
if tag == val_int then handle_int val
else if tag == val_string then handle_string val
else handle_other val
```

---

Closures work correctly across import boundaries -- you can pass lambdas to any imported higher-order function. See [Language Guide: Imports](/learn/language/#imports) for details.
