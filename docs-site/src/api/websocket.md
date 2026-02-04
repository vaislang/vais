# WebSocket API Reference

> WebSocket protocol implementation (RFC 6455)

## Import

```vais
U std/websocket
```

## Features

- Server-side WebSocket upgrade handshake
- Frame encoding/decoding (text, binary, ping, pong, close)
- Masking/unmasking per RFC 6455
- Close handshake

## Opcodes

| Constant | Value | Description |
|----------|-------|-------------|
| `WS_OPCODE_TEXT` | 1 | Text frame |
| `WS_OPCODE_BINARY` | 2 | Binary frame |
| `WS_OPCODE_CLOSE` | 8 | Close frame |
| `WS_OPCODE_PING` | 9 | Ping frame |
| `WS_OPCODE_PONG` | 10 | Pong frame |

## Close Codes

| Constant | Value | Description |
|----------|-------|-------------|
| `WS_CLOSE_NORMAL` | 1000 | Normal closure |
| `WS_CLOSE_GOING_AWAY` | 1001 | Endpoint going away |
| `WS_CLOSE_PROTOCOL_ERROR` | 1002 | Protocol error |

## Key Functions

| Function | Description |
|----------|-------------|
| `ws_accept(tcp_fd)` | Perform WebSocket handshake |
| `ws_send_text(conn, data, len)` | Send text frame |
| `ws_send_binary(conn, data, len)` | Send binary frame |
| `ws_recv(conn, buffer, len)` | Receive frame |
| `ws_close(conn, code)` | Send close frame |

## Usage

```vais
U std/websocket

# Server-side WebSocket handler
F handle_ws(tcp_fd: i64) -> i64 {
    conn := ws_accept(tcp_fd)
    buf := malloc(4096)
    n := ws_recv(conn, buf, 4096)
    ws_send_text(conn, buf, n)  # Echo back
    ws_close(conn, WS_CLOSE_NORMAL)
    0
}
```
