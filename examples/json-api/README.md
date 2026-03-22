# JSON API Server

An HTTP server that parses JSON requests, routes by path, and returns JSON responses.

## Endpoints

- `GET /health` -- returns `{"status":"ok"}`
- `POST /add` -- body `{"a": N, "b": M}`, returns `{"result": N+M}`
- `GET /fib/N` -- returns `{"value": fib(N)}`

## How it works

The server parses raw HTTP requests using string operations (`str_split`,
`str_starts_with`, `str_slice`), dispatches to the appropriate handler
based on the URL path, and constructs HTTP responses with JSON bodies.

JSON parsing is implemented in IRIS using string operations (`str_split`,
`str_slice`, `str_to_int`, `str_trim`, `char_at`). A minimal
`parse_simple_object` function handles flat JSON objects with integer values.
JSON generation uses `int_to_string` and `str_concat`. The built-in
`json_parse`, `json_get`, `json_stringify`, and `json_array_len` primitives
have been removed from the substrate.

## Running

```bash
iris run examples/json-api/json-api.iris
iris run examples/json-api/json-api-test.iris
```

## Primitives used

- `str_split` / `str_slice` / `str_starts_with` / `str_eq` / `str_trim` / `str_to_int` / `char_at` - string ops
- `int_to_string` / `str_concat` - JSON generation
- `list_len` / `list_nth` / `tuple_get` - collection ops
- `tcp_listen` / `tcp_accept` / `tcp_read` / `tcp_write` - TCP I/O
- `unfold` - fibonacci computation
