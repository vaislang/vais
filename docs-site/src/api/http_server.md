# HTTP Server API Reference

> HTTP server framework with routing, middleware, and static files

## Import

```vais
U std/http_server
```

## Features

- Path parameter matching (e.g., `/users/:id`)
- Middleware chain (logging, CORS, auth)
- Static file serving with MIME types
- Request/Response builder pattern
- Query string parsing

## Constants

| Name | Value | Description |
|------|-------|-------------|
| `MIME_HTML` | 1 | text/html |
| `MIME_CSS` | 2 | text/css |
| `MIME_JS` | 3 | application/javascript |
| `MIME_JSON` | 4 | application/json |

## Key Functions

| Function | Description |
|----------|-------------|
| `server_new(port)` | Create HTTP server |
| `server_route(srv, method, path, handler)` | Register route |
| `server_middleware(srv, middleware_fn)` | Add middleware |
| `server_static(srv, url_prefix, dir_path)` | Serve static files |
| `server_start(srv)` | Start listening |

## Usage

```vais
U std/http_server

F handle_index(req: i64, resp: i64) -> i64 {
    # Build response
    0
}

F main() -> i64 {
    srv := server_new(8080)
    server_route(srv, CLIENT_GET, "/", handle_index)
    server_start(srv)
    0
}
```
