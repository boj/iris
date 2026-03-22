# Echo Server

A TCP echo server that reads data from a connection and writes it back.

## How it works

The server listens on a port, accepts a single connection, and enters a
read-write loop. Whatever bytes the client sends are echoed back unchanged.
When the client disconnects (read returns 0), the server closes the connection.

## Running

```bash
iris run examples/echo-server/echo-server.iris
iris run examples/echo-server/echo-test.iris
```

## Primitives used

- `tcp_listen` - bind a port and start listening
- `tcp_accept` - accept an incoming connection
- `tcp_read` - read bytes from a connection
- `tcp_write` - write bytes to a connection
- `tcp_close` - close a connection
