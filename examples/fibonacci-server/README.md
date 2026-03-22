# Fibonacci Server

A TCP server that accepts a number and returns the Nth fibonacci number.

## How it works

Uses IRIS's `unfold` primitive for efficient iterative fibonacci computation.
`unfold (0, 1) (+) n` generates the fibonacci sequence by starting with
seed pair (0, 1) and repeatedly applying addition to produce the next pair.

The server listens on a port, reads a number from the client, computes
the fibonacci value, and sends back the result as a string.

## Running

```bash
iris run examples/fibonacci-server/fib-server.iris
iris run examples/fibonacci-server/fib-test.iris
```

## Primitives used

- `unfold` - corecursive sequence generation (fibonacci via addition)
- `list_nth` - extract element from generated sequence
- `tcp_listen` / `tcp_accept` / `tcp_read` / `tcp_write` - TCP I/O
- `str_to_int` / `int_to_string` - number conversion
