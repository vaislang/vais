# WebSocket API Reference

> WebSocket protocol implementation (RFC 6455)

## Overview

The WebSocket module provides a full-featured WebSocket implementation with:
- Server-side WebSocket upgrade handshake from HTTP
- Client and server connection management
- Frame encoding/decoding (text, binary, ping, pong, close)
- Masking/unmasking per RFC 6455 specification
- Close handshake and connection state management
- Echo server pattern for simple WebSocket servers

## Import

```vais
U std/websocket
```

## Constants

### WebSocket Opcodes

| Constant | Value | Description |
|----------|-------|-------------|
| `WS_OPCODE_CONTINUATION` | 0 | Continuation frame |
| `WS_OPCODE_TEXT` | 1 | Text frame |
| `WS_OPCODE_BINARY` | 2 | Binary frame |
| `WS_OPCODE_CLOSE` | 8 | Close frame |
| `WS_OPCODE_PING` | 9 | Ping frame |
| `WS_OPCODE_PONG` | 10 | Pong frame |

### WebSocket Close Status Codes

| Constant | Value | Description |
|----------|-------|-------------|
| `WS_CLOSE_NORMAL` | 1000 | Normal closure |
| `WS_CLOSE_GOING_AWAY` | 1001 | Endpoint going away |
| `WS_CLOSE_PROTOCOL_ERROR` | 1002 | Protocol error |
| `WS_CLOSE_UNSUPPORTED` | 1003 | Unsupported data type |
| `WS_CLOSE_NO_STATUS` | 1005 | No status code received |
| `WS_CLOSE_ABNORMAL` | 1006 | Abnormal closure |
| `WS_CLOSE_INVALID_DATA` | 1007 | Invalid frame payload data |
| `WS_CLOSE_POLICY` | 1008 | Policy violation |
| `WS_CLOSE_TOO_LARGE` | 1009 | Message too large |
| `WS_CLOSE_EXTENSION` | 1010 | Missing extension |
| `WS_CLOSE_UNEXPECTED` | 1011 | Unexpected condition |

### Connection States

| Constant | Value | Description |
|----------|-------|-------------|
| `WS_STATE_CONNECTING` | 0 | Connection in progress |
| `WS_STATE_OPEN` | 1 | Connection open |
| `WS_STATE_CLOSING` | 2 | Connection closing |
| `WS_STATE_CLOSED` | 3 | Connection closed |

### Buffer Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `WS_MAX_FRAME_SIZE` | 65536 | Maximum frame payload size (64KB) |
| `WS_HEADER_BUFFER` | 8192 | Buffer for HTTP upgrade headers |
| `WS_RECV_BUFFER` | 4096 | TCP receive chunk size |
| `WS_SEND_BUFFER` | 65550 | Send buffer (payload + 14 bytes frame overhead) |

## WsFrame

### WsFrame Struct

```vais
S WsFrame {
    opcode: i64,
    payload: i64,        # Pointer to payload data
    payload_len: i64,
    is_final: i64,       # 1 if FIN bit set
    is_masked: i64       # 1 if MASK bit set
}
```

### WsFrame Methods

#### new

```vais
F new(opcode: i64, payload: i64, payload_len: i64) -> WsFrame
```

Create a new WebSocket frame with custom opcode and payload.

#### text

```vais
F text(msg: str) -> WsFrame
```

Create a text frame from a string message.

#### binary

```vais
F binary(data: i64, len: i64) -> WsFrame
```

Create a binary frame from raw data.

#### ping

```vais
F ping() -> WsFrame
```

Create a ping frame with no payload.

#### pong

```vais
F pong() -> WsFrame
```

Create a pong frame (response to ping).

#### close

```vais
F close(status_code: i64) -> WsFrame
```

Create a close frame with a status code.

#### is_control

```vais
F is_control(&self) -> i64
```

Check if this is a control frame (opcode >= 8). Returns 1 if true, 0 otherwise.

#### is_text

```vais
F is_text(&self) -> i64
```

Check if this is a text frame. Returns 1 if true, 0 otherwise.

#### is_binary

```vais
F is_binary(&self) -> i64
```

Check if this is a binary frame. Returns 1 if true, 0 otherwise.

#### is_close

```vais
F is_close(&self) -> i64
```

Check if this is a close frame. Returns 1 if true, 0 otherwise.

#### is_ping

```vais
F is_ping(&self) -> i64
```

Check if this is a ping frame. Returns 1 if true, 0 otherwise.

#### is_pong

```vais
F is_pong(&self) -> i64
```

Check if this is a pong frame. Returns 1 if true, 0 otherwise.

#### payload_text

```vais
F payload_text(&self) -> str
```

Get payload as string (for text frames). Returns empty string if no payload.

#### close_code

```vais
F close_code(&self) -> i64
```

Get close status code from close frame. Returns `WS_CLOSE_NO_STATUS` if not a close frame.

#### drop

```vais
F drop(&self) -> i64
```

Free payload memory.

## WsConnection

### WsConnection Struct

```vais
S WsConnection {
    fd: i64,             # TCP socket file descriptor
    is_server: i64,      # 1 if server-side, 0 if client-side
    state: i64,          # Connection state (WS_STATE_*)
    mask_key: i64        # Mask key for client->server frames
}
```

### WsConnection Methods

#### from_server

```vais
F from_server(fd: i64) -> WsConnection
```

Create a server-side WebSocket connection from an already-accepted TCP socket.

#### from_client

```vais
F from_client(fd: i64) -> WsConnection
```

Create a client-side WebSocket connection.

#### is_open

```vais
F is_open(&self) -> i64
```

Check if connection is open. Returns 1 if open, 0 otherwise.

#### send_frame

```vais
F send_frame(&self, frame: &WsFrame) -> i64
```

Send a WebSocket frame. Returns bytes sent on success, -1 on error.

#### send_text

```vais
F send_text(&self, msg: str) -> i64
```

Send a text message. Returns bytes sent on success, -1 on error.

#### send_binary

```vais
F send_binary(&self, data: i64, len: i64) -> i64
```

Send binary data. Returns bytes sent on success, -1 on error.

#### send_ping

```vais
F send_ping(&self) -> i64
```

Send a ping frame. Returns bytes sent on success, -1 on error.

#### send_pong

```vais
F send_pong(&self) -> i64
```

Send a pong frame (response to ping). Returns bytes sent on success, -1 on error.

#### send_close

```vais
F send_close(&self, status_code: i64) -> i64
```

Send a close frame with status code. Returns bytes sent on success, -1 on error.

#### recv_frame

```vais
F recv_frame(&self) -> WsFrame
```

Receive a WebSocket frame. Returns frame with opcode=-1 on error or connection closed. Automatically handles close handshake and responds to close frames.

#### close

```vais
F close(&self) -> i64
```

Close the WebSocket connection gracefully (sends close frame if open).

#### force_close

```vais
F force_close(&self) -> i64
```

Force close without handshake (immediately closes TCP socket).

#### drop

```vais
F drop(&self) -> i64
```

Cleanup / destructor (calls force_close).

## WsServer

### WsServer Struct

```vais
S WsServer {
    listener_fd: i64,
    port: i64,
    running: i64
}
```

### WsServer Methods

#### bind

```vais
F bind(port: i64) -> WsServer
```

Create and bind a WebSocket server on the given port.

#### is_valid

```vais
F is_valid(&self) -> i64
```

Check if server is valid (listener bound successfully). Returns 1 if valid, 0 otherwise.

#### accept

```vais
F accept(&self) -> WsConnection
```

Accept a WebSocket connection (performs HTTP upgrade handshake). Returns WsConnection with fd=-1 on error.

#### run_echo

```vais
F run_echo(&self) -> i64
```

Run an echo server loop (convenience for simple servers). Accepts one connection at a time and echoes text and binary frames back.

#### stop

```vais
F stop(&self) -> i64
```

Stop the server.

#### close

```vais
F close(&self) -> i64
```

Close the server listener.

#### drop

```vais
F drop(&self) -> i64
```

Cleanup / destructor (calls close).

## Convenience Functions

```vais
F ws_server(port: i64) -> WsServer
```

Create a WebSocket server bound to a port.

```vais
F ws_upgrade(fd: i64, buffer: i64, buffer_len: i64) -> WsConnection
```

Perform server-side handshake on an existing TCP connection (for integrating with existing HTTP servers). Returns WsConnection with fd=-1 on failure.

## Usage Examples

### WebSocket Echo Server

```vais
U std/websocket

F main() -> i64 {
    server := ws_server(8080)
    I server.is_valid() == 0 {
        R -1
    }

    # Run echo server (blocks)
    server.run_echo()

    server.close()
    0
}
```

### Custom WebSocket Server

```vais
U std/websocket

F main() -> i64 {
    server := WsServer::bind(8080)
    I server.is_valid() == 0 {
        R -1
    }

    L true {
        conn := server.accept()
        I conn.fd < 0 {
            C
        }

        # Handle connection
        L conn.is_open() == 1 {
            frame := conn.recv_frame()

            I frame.opcode == -1 {
                B  # Connection closed or error
            }

            I frame.is_text() == 1 {
                msg := frame.payload_text()
                # Process text message
                conn.send_text(msg)
            } E I frame.is_ping() == 1 {
                conn.send_pong()
            } E I frame.is_close() == 1 {
                B  # Close handled in recv_frame
            }

            frame.drop()
        }

        conn.close()
    }

    0
}
```

### WebSocket Client (Sending Messages)

```vais
U std/websocket

F send_message(tcp_fd: i64, msg: str) -> i64 {
    conn := WsConnection::from_client(tcp_fd)

    # Send text message
    conn.send_text(msg)

    # Receive response
    frame := conn.recv_frame()
    I frame.is_text() == 1 {
        response := frame.payload_text()
        # Process response
    }

    frame.drop()
    conn.close()
    0
}
```

### Integration with HTTP Server

```vais
U std/websocket
U std/http

# Upgrade HTTP connection to WebSocket
F handle_ws_upgrade(tcp_fd: i64, http_buffer: i64, buffer_len: i64) -> i64 {
    conn := ws_upgrade(tcp_fd, http_buffer, buffer_len)

    I conn.fd < 0 {
        R -1  # Upgrade failed
    }

    # Handle WebSocket connection
    L conn.is_open() == 1 {
        frame := conn.recv_frame()
        I frame.opcode == -1 { B }

        I frame.is_text() == 1 {
            conn.send_text(frame.payload_text())
        }

        frame.drop()
    }

    conn.close()
    0
}
```
