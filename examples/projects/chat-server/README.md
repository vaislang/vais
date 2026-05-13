# WebSocket Chat Server

A real-world example of a TCP-based WebSocket chat server written in Vais, demonstrating network programming, state management, and concurrent client handling.

## Features

- **TCP Server**: Listens on port 8081
- **WebSocket Protocol**: Simplified WebSocket handshake (HTTP 101 Switching Protocols)
- **Multi-client Support**: Handles multiple simultaneous connections (up to 100 clients)
- **Message Broadcasting**: Broadcasts messages to all connected clients except sender
- **Join/Leave Notifications**: Announces when users join or leave the chat

## Project Structure

```
chat-server/
├── main.vais       # Server entry point, connection acceptance loop
├── room.vais       # Chat room management (add/remove clients, broadcast)
├── client.vais     # Client handler (handshake, message loop)
└── README.md       # This file
```

## Building

```bash
# From vais project root
cargo run --bin vaisc -- examples/projects/chat-server/main.vais -o chat-server

# Or using the compiler directly
vaisc examples/projects/chat-server/main.vais -o chat-server
```

## Running

```bash
./chat-server
```

Expected output:
```
=== Vais WebSocket Chat Server ===
Starting server on port 8081...
Server started successfully!
Waiting for client connections...
(Connect via ws://localhost:8081)
```

## Testing

### Option 1: Using `websocat` (recommended)

```bash
# Install websocat
brew install websocat  # macOS
# or: cargo install websocat

# Connect to the server
websocat ws://localhost:8081

# Type messages and press Enter to send
```

### Option 2: Using browser JavaScript

```javascript
const ws = new WebSocket('ws://localhost:8081');

ws.onopen = () => {
    console.log('Connected to chat server');
    ws.send('Hello from browser!');
};

ws.onmessage = (event) => {
    console.log('Received:', event.data);
};
```

### Option 3: Using Python

```python
import websocket

ws = websocket.create_connection('ws://localhost:8081')
print(ws.recv())  # Welcome message
ws.send('Hello from Python!')
ws.close()
```

## Architecture

### Data Structures

**ChatRoom** (24 bytes):
- `clients_ptr: i64` - Pointer to array of client file descriptors
- `clients_count: i64` - Current number of connected clients
- `max_clients: i64` - Maximum capacity (100)

### Message Flow

1. **Connection**: Client connects → TCP accept → WebSocket handshake
2. **Join**: Add to room → Send welcome → Broadcast join notification
3. **Message**: Receive → Broadcast to all others
4. **Disconnect**: Remove from room → Broadcast leave notification → Close socket

## Limitations (Simplified Implementation)

- **Single-threaded**: Handles clients sequentially (no threading/async)
- **Simplified WebSocket**: Doesn't fully implement WebSocket frame encoding/decoding
- **No TLS**: Uses plain TCP (ws://, not wss://)
- **No authentication**: No user identification or access control
- **Fixed capacity**: Maximum 100 clients hardcoded

## Production Enhancements

For a production-ready chat server, consider:

- **Threading**: Spawn threads for each client (`spawn` keyword in Vais)
- **Full WebSocket**: Implement RFC 6455 frame parsing/masking
- **TLS Support**: Add SSL/TLS for secure connections
- **User Management**: Username registration, authentication
- **Multiple Rooms**: Support for different chat channels
- **Message History**: Persist and replay recent messages
- **Rate Limiting**: Prevent spam/DoS attacks

## Related Examples

- `examples/net/tcp_server.vais` - Basic TCP server
- `examples/concurrency/thread_pool.vais` - Thread pool pattern
- `examples/projects/http-server/` - HTTP server implementation

## License

Part of the Vais language examples. See repository root for license information.
