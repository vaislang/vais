# Vais Bookmarks - REST API Server

A production-ready HTTP bookmark management API server built entirely in **Vais**, demonstrating real-world systems programming with the Vais language.

## Features

- **RESTful JSON API** - Complete CRUD operations for bookmark management
- **In-Memory Storage** - Fast bookmark store with search capabilities
- **HTTP/1.1 Server** - Custom TCP socket-based HTTP server implementation
- **Optional TLS/HTTPS** - Secure connections with TLS support
- **Structured Logging** - Integrated logging for debugging and monitoring
- **Response Compression** - Optional gzip compression for large responses
- **Search Functionality** - Search bookmarks by title, URL, or tags
- **Health Checks** - `/api/health` endpoint for monitoring

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check - returns service status |
| GET | `/api/bookmarks` | List all bookmarks |
| GET | `/api/bookmarks/:id` | Get a specific bookmark by ID |
| POST | `/api/bookmarks` | Create a new bookmark |
| PUT | `/api/bookmarks/:id` | Update an existing bookmark |
| DELETE | `/api/bookmarks/:id` | Delete a bookmark |
| GET | `/api/search?q=keyword` | Search bookmarks |

## Building and Running

### Prerequisites

- Vais compiler (`vaisc`)
- LLVM 17
- clang (for linking)

### Compile

```bash
cd /Users/sswoo/study/projects/vais/projects/vais-bookmarks
vaisc compile src/main.vais -o bookmark-server
```

### Run

```bash
./bookmark-server
```

The server will start on port 8080 by default.

## Usage Examples

### Health Check

```bash
curl http://localhost:8080/api/health
```

**Response:**
```json
{"status":"ok","service":"vais-bookmarks"}
```

### List All Bookmarks

```bash
curl http://localhost:8080/api/bookmarks
```

**Response:**
```json
[
  {
    "id": 1,
    "title": "Example",
    "url": "https://example.com",
    "tags": "demo,test",
    "created_at": 1234567890,
    "updated_at": 1234567890
  }
]
```

### Create a Bookmark

```bash
curl -X POST http://localhost:8080/api/bookmarks \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Vais Programming Language",
    "url": "https://vais-lang.org",
    "tags": "programming,systems,ai-optimized"
  }'
```

**Response:**
```json
{"success":true,"id":1}
```

### Get Single Bookmark

```bash
curl http://localhost:8080/api/bookmarks/1
```

**Response:**
```json
{
  "id": 1,
  "title": "Vais Programming Language",
  "url": "https://vais-lang.org",
  "tags": "programming,systems,ai-optimized",
  "created_at": 1738665600,
  "updated_at": 1738665600
}
```

### Update Bookmark

```bash
curl -X PUT http://localhost:8080/api/bookmarks/1 \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Vais - AI-Optimized Language",
    "url": "https://vais-lang.org",
    "tags": "programming,systems,ai,compiler"
  }'
```

**Response:**
```json
{"success":true,"message":"Bookmark updated"}
```

### Delete Bookmark

```bash
curl -X DELETE http://localhost:8080/api/bookmarks/1
```

**Response:**
```json
{"success":true,"message":"Bookmark deleted"}
```

### Search Bookmarks

```bash
curl "http://localhost:8080/api/search?q=vais"
```

**Response:**
```json
[
  {
    "id": 1,
    "title": "Vais Programming Language",
    "url": "https://vais-lang.org",
    "tags": "programming,systems,ai-optimized",
    "created_at": 1738665600,
    "updated_at": 1738665600
  }
]
```

## Project Structure

```
vais-bookmarks/
├── vais.toml              # Package manifest
├── README.md              # This file
└── src/
    ├── main.vais          # Server entry point
    ├── server.vais        # HTTP server implementation
    ├── handler.vais       # Request handlers
    ├── bookmark.vais      # Data model and store
    └── json_helper.vais   # JSON utilities
```

## Architecture

### Data Flow

```
Client Request
    ↓
TCP Socket (server.vais)
    ↓
HTTP Parser (server.vais)
    ↓
Router (server.vais)
    ↓
Handler (handler.vais)
    ↓
Bookmark Store (bookmark.vais)
    ↓
JSON Response (json_helper.vais)
    ↓
HTTP Response (server.vais)
    ↓
Client
```

### Key Components

1. **main.vais** - Entry point, configuration, startup
2. **server.vais** - TCP socket server, HTTP parsing, routing
3. **handler.vais** - Business logic for each endpoint
4. **bookmark.vais** - In-memory data store and bookmark operations
5. **json_helper.vais** - JSON generation and parsing utilities

## Configuration

Edit `src/main.vais` to configure:

- **Port**: Change `port := 8080` to desired port
- **TLS**: Set `use_tls := 1` to enable HTTPS
- **Logging**: Modify log levels in handlers

## Advanced Features

### TLS/HTTPS Support

Enable TLS by setting `use_tls := 1` in `main.vais`. The server will use the TLS extern functions for secure connections.

### Response Compression

Responses larger than 1KB are automatically compressed using gzip if the client supports it (via `Accept-Encoding` header).

### Structured Logging

All requests and operations are logged using the `log_info` and `log_error` extern functions for easy debugging and monitoring.

## Performance

- **In-memory storage** - No disk I/O for maximum speed
- **Zero-copy HTTP parsing** - Minimal memory allocations
- **Optional compression** - Reduces bandwidth for large responses
- **Concurrent connections** - Handles multiple clients (via accept loop)

## Limitations

- **In-memory only** - Data is lost on server restart (no persistence)
- **Single-threaded** - Handles one request at a time
- **Basic HTTP/1.1** - No HTTP/2 or WebSocket support
- **Simple JSON parsing** - No complex nested object support

## Future Enhancements

- Add SQLite persistence
- Implement multi-threading with async runtime
- Add authentication and authorization
- Support for HTTP/2 and server-sent events
- WebSocket support for real-time updates
- Rate limiting and request throttling
- Metrics and prometheus endpoints

## Vais Language Features Demonstrated

- **Structs** - `Bookmark`, `BookmarkStore`, `HttpRequest`, `HttpResponse`
- **Functions** - Pure functions with clear signatures
- **Extern declarations** - C interop for system calls
- **Pattern matching** - Request routing and method dispatch
- **Loops** - Server accept loop and data iteration
- **Type inference** - Clean, minimal type annotations
- **Single-character keywords** - `F`, `S`, `I`, `E`, `L`, `R`, `C`, `X`

## License

MIT License - See LICENSE file for details

## Contributing

Contributions welcome! This is a demonstration project for the Vais language.

## About Vais

Vais is an AI-optimized systems programming language with:
- Single-character keywords for faster AI code generation
- LLVM backend for native performance
- Full type inference
- Rust-like safety with C-like simplicity

Learn more at: https://github.com/vais-lang/vais
