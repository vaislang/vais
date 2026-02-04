# HTTP Server API Reference

> Advanced HTTP server framework with routing, middleware, and static files

## Import

```vais
U std/http_server
```

## Overview

The HTTP server module provides a full-featured web framework built on top of `std/http` and `std/net`. It includes:

- Path parameter matching (e.g., `/users/:id`)
- Query string parsing
- Middleware chain (logging, CORS, auth)
- Static file serving with automatic MIME detection
- Request/Response builder pattern
- HTTPS/TLS support

## Constants

### Capacity Limits

| Constant | Value | Description |
|----------|-------|-------------|
| `MAX_PATH_SEGMENTS` | 32 | Maximum path segments in route pattern |
| `MAX_PARAMS` | 16 | Maximum path parameters per route |
| `MAX_MIDDLEWARES` | 32 | Maximum middleware handlers |
| `MAX_QUERY_PARAMS` | 32 | Maximum query parameters per request |
| `STATIC_FILE_BUFFER` | 65536 | Static file read buffer size (64KB) |
| `MAX_ROUTES` | 256 | Maximum routes per application |

### MIME Types

| Constant | Value | MIME Type |
|----------|-------|-----------|
| `MIME_HTML` | 1 | text/html |
| `MIME_CSS` | 2 | text/css |
| `MIME_JS` | 3 | application/javascript |
| `MIME_JSON` | 4 | application/json |
| `MIME_TEXT` | 5 | text/plain |
| `MIME_PNG` | 6 | image/png |
| `MIME_JPG` | 7 | image/jpeg |
| `MIME_GIF` | 8 | image/gif |
| `MIME_SVG` | 9 | image/svg+xml |
| `MIME_ICO` | 10 | image/x-icon |
| `MIME_WASM` | 11 | application/wasm |
| `MIME_OCTET` | 12 | application/octet-stream |

### HTTP Methods

| Constant | Value | Method |
|----------|-------|--------|
| `METHOD_GET` | 1 | GET |
| `METHOD_POST` | 2 | POST |
| `METHOD_PUT` | 3 | PUT |
| `METHOD_DELETE` | 4 | DELETE |
| `METHOD_PATCH` | 5 | PATCH |
| `METHOD_HEAD` | 6 | HEAD |
| `METHOD_OPTIONS` | 7 | OPTIONS |

### Route Segment Types

| Constant | Value | Type |
|----------|-------|------|
| `SEG_LITERAL` | 0 | Exact match (e.g., "users") |
| `SEG_PARAM` | 1 | Parameter capture (e.g., ":id") |
| `SEG_WILDCARD` | 2 | Wildcard match (e.g., "*") |

## Structs

### PathParam

Represents a single path parameter captured from a route pattern.

```vais
S PathParam {
    name: str,
    value: str
}
```

### PathParams

Collection of path parameters extracted from a matched route.

```vais
S PathParams {
    items: i64,      # Pointer to array of PathParam
    count: i64,
    capacity: i64
}
```

**Methods:**

- `new() -> PathParams` - Create empty path parameters collection
- `add(&self, name: str, value: str) -> i64` - Add parameter (returns 0 on success, -1 if full)
- `get(&self, name: str) -> str?` - Get parameter value by name
- `len(&self) -> i64` - Get parameter count
- `clear(&self) -> i64` - Clear all parameters
- `drop(&self) -> i64` - Free memory

### QueryParam

Represents a single query string parameter.

```vais
S QueryParam {
    key: str,
    value: str
}
```

### QueryParams

Collection of query string parameters parsed from the request URL.

```vais
S QueryParams {
    items: i64,      # Pointer to array of QueryParam
    count: i64,
    capacity: i64
}
```

**Methods:**

- `new() -> QueryParams` - Create empty query parameters collection
- `parse(query: str) -> QueryParams` - Parse query string (e.g., "key1=val1&key2=val2")
- `add(&self, key: str, value: str) -> i64` - Add parameter
- `get(&self, key: str) -> str?` - Get parameter value by key
- `has(&self, key: str) -> i64` - Check if key exists (returns 1 if present, 0 if not)
- `len(&self) -> i64` - Get parameter count
- `drop(&self) -> i64` - Free memory

### RequestCtx

Enhanced request context with path parameters, query parameters, and helper methods.

```vais
S RequestCtx {
    method: i64,
    path: str,
    full_path: str,     # Original path including query string
    version: str,
    headers: i64,       # Pointer to Headers struct
    header_count: i64,
    header_capacity: i64,
    body: i64,
    body_len: i64,
    params: PathParams,
    query: QueryParams
}
```

**Methods:**

- `from_raw(method, path, version, header_items, header_count, header_capacity, body, body_len) -> RequestCtx` - Create from raw request data
- `get_header(&self, name: str) -> str?` - Get header value by name (case-insensitive)
- `method_str(&self) -> str` - Get method as string ("GET", "POST", etc.)
- `body_text(&self) -> str` - Get body as string
- `param(&self, name: str) -> str?` - Get path parameter value
- `query_param(&self, key: str) -> str?` - Get query parameter value
- `content_type(&self) -> str?` - Get Content-Type header
- `is_json(&self) -> i64` - Check if request is JSON (returns 1 if true)

### ResponseBuilder

Fluent builder for HTTP responses with method chaining.

```vais
S ResponseBuilder {
    status: i64,
    status_text: str,
    headers: i64,       # Pointer to header array
    header_count: i64,
    header_capacity: i64,
    body: i64,
    body_len: i64
}
```

**Static Methods:**

- `new(status: i64) -> ResponseBuilder` - Create response with status code
- `ok() -> ResponseBuilder` - 200 OK response
- `created() -> ResponseBuilder` - 201 Created response
- `no_content() -> ResponseBuilder` - 204 No Content response
- `bad_request() -> ResponseBuilder` - 400 Bad Request response
- `unauthorized() -> ResponseBuilder` - 401 Unauthorized response
- `forbidden() -> ResponseBuilder` - 403 Forbidden response
- `not_found() -> ResponseBuilder` - 404 Not Found response
- `method_not_allowed() -> ResponseBuilder` - 405 Method Not Allowed response
- `internal_error() -> ResponseBuilder` - 500 Internal Server Error response
- `redirect(url: str) -> ResponseBuilder` - 302 redirect to URL
- `redirect_permanent(url: str) -> ResponseBuilder` - 301 permanent redirect

**Instance Methods:**

- `header(&self, name: str, value: str) -> ResponseBuilder` - Add header (chainable)
- `text(&self, content: str) -> ResponseBuilder` - Set text body with Content-Type
- `json(&self, content: str) -> ResponseBuilder` - Set JSON body with Content-Type
- `html(&self, content: str) -> ResponseBuilder` - Set HTML body with Content-Type
- `body(&self, data: i64, len: i64) -> ResponseBuilder` - Set raw body
- `content_type(&self, ct: str) -> ResponseBuilder` - Set Content-Type header
- `cors(&self, origin: str) -> ResponseBuilder` - Add CORS headers
- `cache(&self, max_age: i64) -> ResponseBuilder` - Set Cache-Control max-age
- `serialize(&self, buffer: i64, buffer_size: i64) -> i64` - Serialize to buffer for sending

### RoutePattern

Internal struct representing a parsed route pattern with segments.

```vais
S RoutePattern {
    method: i64,
    pattern: str,
    handler_ptr: i64,
    seg_types: i64,    # Array of segment types
    seg_values: i64,   # Array of segment string pointers
    seg_count: i64
}
```

**Methods:**

- `parse(method: i64, pattern: str, handler: i64) -> RoutePattern` - Parse pattern into segments
- `matches(&self, path: str, params: &PathParams) -> i64` - Match path against pattern (returns 1 if matched)

### Middleware

Represents a middleware handler with priority.

```vais
S Middleware {
    name: str,
    handler_ptr: i64,   # Function pointer
    priority: i64       # Lower = runs first
}
```

### MiddlewareChain

Collection of middleware handlers executed in priority order.

```vais
S MiddlewareChain {
    items: i64,      # Array of Middleware
    count: i64,
    capacity: i64
}
```

**Methods:**

- `new() -> MiddlewareChain` - Create empty middleware chain
- `add(&self, name: str, handler: i64, priority: i64) -> MiddlewareChain` - Add middleware with priority
- `execute(&self, ctx: &RequestCtx, response: ResponseBuilder) -> ResponseBuilder` - Execute all middleware
- `len(&self) -> i64` - Get middleware count

### StaticFiles

Static file server with MIME type detection and security checks.

```vais
S StaticFiles {
    root_dir: str,
    prefix: str,       # URL prefix (e.g., "/static")
    index_file: str    # Default index file (e.g., "index.html")
}
```

**Methods:**

- `new(root_dir: str, prefix: str) -> StaticFiles` - Create static file server
- `with_index(&self, index: str) -> StaticFiles` - Set custom index file
- `mime_type(path: str) -> str` - Determine MIME type from file extension
- `is_safe_path(path: str) -> i64` - Check for path traversal attacks (returns 1 if safe)
- `serve(&self, req_path: str) -> ResponseBuilder` - Serve file from request path

### App

Main HTTP server application with routing, middleware, and static files.

```vais
S App {
    host: str,
    port: i64,
    routes: i64,        # Array of RoutePattern pointers
    route_count: i64,
    route_capacity: i64,
    middleware: MiddlewareChain,
    static_files: i64,  # Pointer to StaticFiles (0 if none)
    running: i64,
    tls_ctx: i64,       # TLS context handle (0 = plain HTTP)
    tls_enabled: i64    # 1 if HTTPS mode, 0 for HTTP
}
```

**Static Methods:**

- `new(port: i64) -> App` - Create app listening on port (binds to 0.0.0.0)
- `bind(host: str, port: i64) -> App` - Create app with specific host and port

**Instance Methods:**

- `with_tls(&self, cert_path: str, key_path: str) -> App` - Enable HTTPS with certificate and key
- `route(&self, method: i64, pattern: str, handler: i64) -> App` - Add route with pattern matching
- `get(&self, pattern: str, handler: i64) -> App` - Register GET route
- `post(&self, pattern: str, handler: i64) -> App` - Register POST route
- `put(&self, pattern: str, handler: i64) -> App` - Register PUT route
- `delete(&self, pattern: str, handler: i64) -> App` - Register DELETE route
- `patch(&self, pattern: str, handler: i64) -> App` - Register PATCH route
- `use_middleware(&self, name: str, handler: i64) -> App` - Add middleware
- `use_middleware_with_priority(&self, name: str, handler: i64, priority: i64) -> App` - Add middleware with priority
- `serve_static(&self, prefix: str, root_dir: str) -> App` - Enable static file serving
- `run(&self) -> i64` - Run server (blocking)
- `stop(&self) -> i64` - Stop server
- `find_and_handle(&self, ctx: &RequestCtx) -> ResponseBuilder` - Find matching route and execute handler

## Convenience Functions

### App Creation

```vais
F app(port: i64) -> App
```

Create a new HTTP server app on the specified port.

### Response Builders

```vais
F response(status: i64) -> ResponseBuilder
F ok() -> ResponseBuilder
F created() -> ResponseBuilder
F not_found() -> ResponseBuilder
F bad_request() -> ResponseBuilder
F internal_error() -> ResponseBuilder
```

Quick response builder functions.

### Response Helpers

```vais
F json_response(data: str) -> ResponseBuilder
F html_response(content: str) -> ResponseBuilder
F text_response(content: str) -> ResponseBuilder
```

Create common response types with appropriate Content-Type headers.

## Middleware Functions

### CORS Middleware

```vais
F cors_middleware(origin: str) -> i64
```

Create CORS middleware handler for specific origin.

### Logging Middleware

```vais
F logging_middleware_handler(ctx: &RequestCtx, response: ResponseBuilder) -> ResponseBuilder
```

Log request details (METHOD PATH -> STATUS).

### Default CORS Handler

```vais
F default_cors_handler(ctx: &RequestCtx, response: ResponseBuilder) -> ResponseBuilder
```

Add CORS headers allowing all origins (*).

## Usage Examples

### Basic Server

```vais
U std/http_server

F handle_index(ctx: &RequestCtx) -> ResponseBuilder {
    ResponseBuilder::ok().html("<h1>Hello, World!</h1>")
}

F main() -> i64 {
    app := App::new(8080)
        .get("/", handle_index as i64)

    app.run()
}
```

### Path Parameters

```vais
F handle_user(ctx: &RequestCtx) -> ResponseBuilder {
    user_id := ctx.param("id")
    M user_id {
        Some(id) => {
            json := __str_concat3("{\"user_id\": \"", id, "\"}")
            ResponseBuilder::ok().json(json)
        },
        None => ResponseBuilder::bad_request().json("{\"error\": \"Missing ID\"}")
    }
}

F main() -> i64 {
    app := App::new(8080)
        .get("/users/:id", handle_user as i64)

    app.run()
}
```

### Query Parameters

```vais
F handle_search(ctx: &RequestCtx) -> ResponseBuilder {
    query := ctx.query_param("q")
    page := ctx.query_param("page")

    M query {
        Some(q) => {
            # Search with query parameter
            ResponseBuilder::ok().json("{\"results\": []}")
        },
        None => ResponseBuilder::bad_request().json("{\"error\": \"Missing query\"}")
    }
}

F main() -> i64 {
    app := App::new(8080)
        .get("/search", handle_search as i64)

    app.run()
}
```

### POST with JSON Body

```vais
F handle_create_user(ctx: &RequestCtx) -> ResponseBuilder {
    I ctx.is_json() != 1 {
        R ResponseBuilder::bad_request().json("{\"error\": \"Content-Type must be application/json\"}")
    }

    body := ctx.body_text()
    # Parse and process JSON body
    ResponseBuilder::created().json("{\"id\": 123}")
}

F main() -> i64 {
    app := App::new(8080)
        .post("/users", handle_create_user as i64)

    app.run()
}
```

### Middleware

```vais
F auth_middleware(ctx: &RequestCtx, response: ResponseBuilder) -> ResponseBuilder {
    auth_header := ctx.get_header("Authorization")
    M auth_header {
        Some(token) => {
            # Validate token
            response
        },
        None => ResponseBuilder::unauthorized().json("{\"error\": \"Unauthorized\"}")
    }
}

F main() -> i64 {
    app := App::new(8080)
        .use_middleware("auth", auth_middleware as i64)
        .get("/protected", handle_protected as i64)

    app.run()
}
```

### Static Files

```vais
F main() -> i64 {
    app := App::new(8080)
        .serve_static("/static", "./public")
        .get("/", handle_index as i64)

    app.run()
}
```

Static files at `./public/style.css` will be served at `/static/style.css` with automatic MIME type detection.

### HTTPS Server

```vais
F main() -> i64 {
    app := App::new(8443)
        .with_tls("cert.pem", "key.pem")
        .get("/", handle_index as i64)

    app.run()
}
```

### Multiple Routes and Methods

```vais
F main() -> i64 {
    app := App::new(8080)
        .get("/", handle_index as i64)
        .get("/users", handle_list_users as i64)
        .get("/users/:id", handle_get_user as i64)
        .post("/users", handle_create_user as i64)
        .put("/users/:id", handle_update_user as i64)
        .delete("/users/:id", handle_delete_user as i64)
        .serve_static("/static", "./public")
        .use_middleware("logging", logging_middleware_handler as i64)
        .use_middleware("cors", default_cors_handler as i64)

    app.run()
}
```

### CORS Configuration

```vais
F main() -> i64 {
    app := App::new(8080)
        .get("/api/data", handle_data as i64)
        .use_middleware("cors", default_cors_handler as i64)

    app.run()
}
```

### Custom Response Headers

```vais
F handle_download(ctx: &RequestCtx) -> ResponseBuilder {
    file_data := __read_file("report.pdf" as i64)
    file_size := __file_size("report.pdf" as i64)

    ResponseBuilder::ok()
        .header("Content-Type", "application/pdf")
        .header("Content-Disposition", "attachment; filename=\"report.pdf\"")
        .body(file_data, file_size)
}
```

### Redirects

```vais
F handle_old_url(ctx: &RequestCtx) -> ResponseBuilder {
    ResponseBuilder::redirect_permanent("/new-url")
}

F handle_login_redirect(ctx: &RequestCtx) -> ResponseBuilder {
    ResponseBuilder::redirect("/login?redirect=/dashboard")
}
```

### Cache Control

```vais
F handle_static_asset(ctx: &RequestCtx) -> ResponseBuilder {
    ResponseBuilder::ok()
        .content_type("application/javascript")
        .cache(3600)  # Cache for 1 hour
        .body(asset_data, asset_len)
}
```

## Security Features

### Path Traversal Protection

The static file server automatically checks for `..` in paths to prevent directory traversal attacks.

### HTTPS/TLS Support

Full TLS 1.2+ support via `with_tls()` method. Requires certificate and private key files.

### CORS Support

Built-in CORS middleware for cross-origin resource sharing.

## Performance Notes

- Uses zero-copy serialization where possible
- Pre-parsed route patterns for efficient matching
- 64KB buffer for static file serving
- Connection: close header for simplicity (HTTP/1.1)
- Supports up to 256 routes per application
- Middleware chain executed in priority order

## See Also

- [HTTP API](./http.md) - Lower-level HTTP client/server
- [Net API](./net.md) - TCP/IP networking primitives
- [File API](./file.md) - File system operations
