# Chat Protocol

A simple chat protocol over TCP. Messages are formatted as `USER:message\n`.
The server tracks connected clients using State and broadcasts messages.

## How it works

- Messages are formatted as `USER:message\n` strings
- Client connections are tracked in a State map (`client_N` -> connection handle)
- Broadcasting iterates over all client keys and writes the message to each
- Parsing splits on `:` to extract the username and message body

## Running

```bash
iris run examples/chat-protocol/chat.iris
iris run examples/chat-protocol/chat-test.iris
```

## Primitives used

- `str_concat` / `str_split` / `str_starts_with` / `str_trim` - message format
- `state_empty` / `map_insert` / `map_get` / `map_remove` - client tracking
- `map_keys` / `fold` - broadcast iteration
- `tcp_listen` / `tcp_accept` / `tcp_read` / `tcp_write` - TCP I/O
