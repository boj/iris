# Tutorial: Building a Self-Improving Service

This tutorial walks through writing an IRIS program, evolving it, and deploying it as a self-improving service.

---

## Step 1: Write a Specification

Create a file `src/iris-programs/my_service/spec.iris` that defines what we want: a function that computes the sum of a list of integers.

```iris
-- Sum a list of integers.
-- test: 0 -> 0
-- test: 1 -> 0
-- test: 2 -> 1
-- test: 3 -> 3
-- test: 4 -> 6
-- test: 5 -> 10
-- test: 10 -> 45
let sum_to n : Int -> Int [cost: Linear(n)] = fold 0 (+) n
```

The `-- test: <inputs> -> <output>` comments define the specification. Each line provides input(s) and the expected output, separated by `->`. Multiple inputs are comma-separated.

---

## Step 2: Write the Program

Create `src/iris-programs/my_service/service.iris`:

```iris
-- A service that computes various reductions over integer ranges.
-- Demonstrates fold with different operators and contracts.

-- Sum of integers from 0 to n-1
let sum_to n : Int -> Int [cost: Linear(n)]
  requires n >= 0
  ensures result >= 0
  = fold 0 (+) n

-- Product of integers from 1 to n (factorial)
let product_to n : Int -> Int [cost: Linear(n)]
  requires n >= 0
  = fold 1 (\acc i -> acc * (i + 1)) n

-- Maximum of integers from 0 to n-1
let max_of n : Int -> Int [cost: Linear(n)]
  requires n >= 1
  = fold 0 (\acc i -> if i > acc then i else acc) n

-- Entry point: dispatch based on operation code
-- op=0: sum, op=1: product, op=2: max
let main op n : Int -> Int -> Int [cost: Linear(n)] =
  match op with
  | 0 -> sum_to n
  | 1 -> product_to n
  | 2 -> max_of n
  | _ -> 0
```

---

## Step 3: Verify the Program

Run the type checker to verify contracts and cost annotations:

```sh
iris check src/iris-programs/my_service/service.iris
```

Expected output:
```
[OK] sum_to: 5/5 obligations satisfied (score: 1.00)
[OK] product_to: 4/4 obligations satisfied (score: 1.00)
[OK] max_of: 4/4 obligations satisfied (score: 1.00)
[OK] main: 6/6 obligations satisfied (score: 1.00)
All 4 definitions verified.
```

---

## Step 4: Test Manually

Run the program with different inputs:

```sh
iris run src/iris-programs/my_service/service.iris 0 10    # sum of 0..9 = 45
iris run src/iris-programs/my_service/service.iris 1 5     # product of 1..5 = 120
iris run src/iris-programs/my_service/service.iris 2 10    # max of 0..9 = 9
```

---

## Step 5: Evolve an Alternative Solution

Create a specification file with test cases to evolve a solution:

```iris
-- test: 0 -> 0
-- test: 1 -> 0
-- test: 2 -> 1
-- test: 3 -> 3
-- test: 4 -> 6
-- test: 5 -> 10
-- test: 10 -> 45
-- test: 100 -> 4950
```

Save this as `src/iris-programs/my_service/evolve_spec.iris` and run:

```sh
iris solve src/iris-programs/my_service/evolve_spec.iris
```

The evolutionary engine will:

1. **Analyze** the test cases to detect the problem pattern (sum detection)
2. **Generate seed skeletons** matching the detected pattern
3. **Launch enumeration** in a background thread for small programs (<=8 nodes)
4. **Run NSGA-II** with a population of 64 individuals
5. **Score** each individual on three objectives: correctness, performance, verifiability
6. **Report** the best solution found

Example output:
```
Evolving solution from 8 test cases...
Evolution complete: 12 generations in 0.34s
Best fitness: correctness=1.0000, performance=0.9800, verifiability=0.9000
Best program: 3 nodes, 2 edges
```

---

## Step 6: Deploy

### As a Standalone Binary

```sh
iris compile src/iris-programs/my_service/service.iris -o my_service
./my_service 0 10
# Output: 45
```

This produces a native ELF64 binary (x86-64 only) with no runtime dependencies.

### As Deployable Rust Source

```sh
iris deploy src/iris-programs/my_service/service.iris -o my_service.rs
rustc --edition 2021 -O my_service.rs -o my_service
```

This generates a self-contained Rust file with the bytecode VM embedded.

### As a Shared Library

The `deploy_shared_lib` API generates a `.so` that exports:
```c
int64_t iris_invoke(int64_t* inputs, size_t num_inputs, int64_t* output);
```

---

## Step 7: Run the Self-Improving Daemon

The daemon continuously executes programs and improves them:

```sh
iris daemon --max-cycles 100 --exec-mode interval:800 --improve-threshold 2.0
```

This starts the `SelfImprovingDaemon` which:

1. **Executes** all registered programs every 800ms (one "cognitive cycle")
2. **Profiles** component performance (latency, correctness)
3. **Detects** improvement opportunities via trigger criteria
4. **Evolves** replacement components in background threads
5. **Gates** replacements: rejects if > 2x slowdown vs. original
6. **Deploys** improvements via hot-swap
7. **Audits** every change with before/after metrics
8. **Inspects** for regressions and auto-rolls-back if detected

### Daemon Options

| Flag | Default | Description |
|------|---------|-------------|
| `--exec-mode` | `interval:800` | `continuous` or `interval:N` (ms) |
| `--improve-threshold` | `2.0` | Max slowdown factor for deployment gate |
| `--max-stagnant` | `5` | Give up after N failed improvement attempts |
| `--max-improve-threads` | `2` | Concurrent improvement threads |
| `--max-cycles` | (unlimited) | Stop after N cycles |

### Monitoring the Daemon

The daemon logs its activity to stderr:

```
Starting IRIS threaded self-improving daemon...
  max_improve_threads=2, max_stagnant=5, max_slowdown=2.0x
  Will run 100 cycles.
Daemon stopped after 100 cycles (82.45s): 8 improvement cycles, 3 deployed,
  12 audit entries, 2 converged, fully_converged=false
```

Key metrics:
- **improvement_cycles**: How many times the daemon attempted improvement
- **components_deployed**: How many improved components were accepted
- **audit_entries**: Total audit trail entries
- **converged_components**: Components that reached a local optimum
- **fully_converged**: Whether ALL components have converged

---

## Step 8: Monitor with the Audit Trail

Every deployment action is recorded in the audit trail. The audit captures:

- **Action type**: Deploy, Rollback, Inspect
- **Component name**: Which component was affected
- **Before/after metrics**: Latency, correctness score
- **Timestamp**: When the action occurred

The daemon persists state to `.iris-daemon/` in the current directory, enabling restart with preserved history.

---

## Program Patterns

### Self-Modifying Programs

IRIS programs can inspect and modify their own graph at runtime:

```iris
let self_improve test_cases =
  let program = self_graph () in           -- Capture own graph
  let score = graph_eval program test_cases in  -- Evaluate self
  let modified = graph_set_prim_op program () 0 in  -- Modify
  let new_score = graph_eval modified test_cases in  -- Evaluate modification
  if new_score > score then new_score      -- Keep better version
  else score
```

### Programs with Contracts

Use `requires` and `ensures` for verified correctness:

```iris
let bounded_add x y : Int -> Int -> Int
  requires x >= -500000 && x <= 500000
  requires y >= -500000 && y <= 500000
  ensures result >= -1000000
  ensures result <= 1000000
  ensures result == x + y
  = x + y
```

### Capability-Restricted Modules

Control what effects your module can perform:

```iris
allow [FileRead, FileWrite "/tmp/*"]
deny [TcpConnect, ThreadSpawn, MmapExec]

let process path =
  let h = file_open path 0 in
  let data = file_read_bytes h 4096 in
  let _ = file_close h in
  data
```

### Concurrent Programs

Spawn threads and synchronize:

```iris
let parallel_compute x =
  let prog = self_graph () in
  let h1 = thread_spawn prog in
  let h2 = thread_spawn prog in
  let r1 = thread_join h1 in
  let r2 = thread_join h2 in
  (r1, r2)
```

### FFI Integration

Call C functions from IRIS:

```iris
let main =
  let pid = ffi_call "libc" "getpid" () in
  let time = ffi_call "libc" "time" (0) in
  (pid, time)
```
