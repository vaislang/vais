# Async HTTP API Reference

> Non-blocking HTTP/1.1 server and client built on async networking

## Import

```vais
U std/async_http
```

## Overview

The `async_http` module provides a complete async HTTP/1.1 framework with routing, middleware support, request/response parsing, connection pooling, and a client API. It is designed for building non-blocking web services.

## Constants

### HTTP Methods

| Constant | Value |
|----------|-------|
| `ASYNC_HTTP_GET` | 1 |
| `ASYNC_HTTP_POST` | 2 |
| `ASYNC_HTTP_PUT` | 3 |
| `ASYNC_HTTP_DELETE` | 4 |
| `ASYNC_HTTP_PATCH` | 5 |
| `ASYNC_HTTP_HEAD` | 6 |
| `ASYNC_HTTP_OPTIONS` | 7 |

### HTTP Status Codes

| Constant | Value |
|----------|-------|
| `STATUS_OK` | 200 |
| `STATUS_CREATED` | 201 |
| `STATUS_NO_CONTENT` | 204 |
| `STATUS_NOT_FOUND` | 404 |
| `STATUS_INTERNAL_ERROR` | 500 |

### Content Types

| Constant | Value | MIME |
|----------|-------|------|
| `CT_TEXT` | 1 | text/plain |
| `CT_HTML` | 2 | text/html |
| `CT_JSON` | 3 | application/json |
| `CT_FORM` | 4 | application/x-www-form-urlencoded |
| `CT_OCTET` | 5 | application/octet-stream |

## Structs

### `AsyncHttpRequest`

Parsed HTTP request from a client.

**Fields:** `method`, `path`, `query`, `version`, `headers`, `header_count`, `body`, `body_len`, `content_type`, `keep_alive`

**Methods:**
- `new() -> AsyncHttpRequest` -- Create an empty request
- `method_str(&self) -> str` -- Get HTTP method as string
- `is_get(&self) -> i64` -- Check if GET request
- `is_post(&self) -> i64` -- Check if POST request
- `has_body(&self) -> i64` -- Check if request has a body
- `get_header(&self, name: str) -> str` -- Get header value by name

### `AsyncHttpResponse`

HTTP response built by a handler.

**Fields:** `status`, `headers`, `header_count`, `body`, `body_len`, `content_type`

**Methods:**
- `new(status: i64) -> AsyncHttpResponse` -- Create with status code
- `ok() -> AsyncHttpResponse` -- Create a 200 OK response
- `not_found() -> AsyncHttpResponse` -- Create a 404 response
- `bad_request() -> AsyncHttpResponse` -- Create a 400 response
- `internal_error() -> AsyncHttpResponse` -- Create a 500 response
- `with_text(&mut self, text: str) -> i64` -- Set text body
- `with_html(&mut self, html: str) -> i64` -- Set HTML body
- `with_json(&mut self, json: str) -> i64` -- Set JSON body
- `add_header(&mut self, name: str, value: str) -> i64` -- Add a response header
- `status_text(&self) -> str` -- Get status text (e.g., "OK")
- `content_type_str(&self) -> str` -- Get MIME type string

### `Router`

Collection of routes mapping method+path to handler functions.

**Methods:**
- `new() -> Router` -- Create an empty router
- `get(&mut self, path: str, handler: i64) -> i64` -- Add GET route
- `post(&mut self, path: str, handler: i64) -> i64` -- Add POST route
- `put(&mut self, path: str, handler: i64) -> i64` -- Add PUT route
- `delete(&mut self, path: str, handler: i64) -> i64` -- Add DELETE route
- `find_handler(&self, method: i64, path: str) -> i64` -- Find handler for request

### `AsyncHttpServer`

HTTP server with routing and middleware.

**Methods:**
- `new(host: str, port: i64) -> AsyncHttpServer` -- Create a server
- `route_get/post/put/delete(&mut self, path: str, handler: i64)` -- Add routes
- `use_middleware(&mut self, mw: Middleware) -> i64` -- Add middleware
- `bind(&mut self) -> i64` -- Start listening
- `poll_once(&mut self) -> i64` -- Process one request
- `run_iterations(&mut self, max_iter: i64) -> i64` -- Run accept loop
- `stop(&mut self) -> i64` -- Stop the server
- `stats_requests/stats_connections(&self) -> i64` -- Get statistics

### `AsyncHttpClient`

HTTP client for making requests.

**Methods:**
- `new() -> AsyncHttpClient` -- Create a client
- `with_base_url(url: str) -> AsyncHttpClient` -- Create with base URL
- `get(&self, path: str) -> AsyncHttpResponse` -- GET request
- `post(&self, path: str, body: str) -> AsyncHttpResponse` -- POST request
- `put(&self, path: str, body: str) -> AsyncHttpResponse` -- PUT request
- `delete(&self, path: str) -> AsyncHttpResponse` -- DELETE request
- `post_json(&self, path: str, json: str) -> AsyncHttpResponse` -- POST JSON
- `set_timeout(&mut self, ms: i64) -> i64` -- Set timeout
- `add_default_header(&mut self, name: str, value: str) -> i64` -- Add default header

### `ConnectionPool`

Reusable connection pool for managing file descriptors.

**Methods:**
- `new(capacity: i64) -> ConnectionPool`
- `acquire(&mut self) -> i64` -- Get a connection fd
- `release(&mut self, fd: i64) -> i64` -- Return a connection
- `available(&self) -> i64` -- Count of available connections

## Convenience Functions

```vais
F async_http_server(host: str, port: i64) -> AsyncHttpServer
F async_http_client() -> AsyncHttpClient
F async_http_client_for(url: str) -> AsyncHttpClient
F text_response(status: i64, text: str) -> AsyncHttpResponse
F json_response(status: i64, json: str) -> AsyncHttpResponse
F html_response(status: i64, html: str) -> AsyncHttpResponse
F connection_pool(capacity: i64) -> ConnectionPool
F logging_middleware() -> Middleware
F cors_middleware() -> Middleware
```

## Example

```vais
U std/async_http

F main() {
    server := mut async_http_server("127.0.0.1", 8080)
    server.route_get("/health", health_handler)
    server.bind()
    server.run_iterations(100)
    server.stop()
}
```
