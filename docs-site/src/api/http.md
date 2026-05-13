# HTTP API Reference

> HTTP client and server implementation built on TCP networking

> **Implementation:** Requires C runtime (`http_runtime.c`). Provides HTTP/1.1 protocol constants and types used by `http_client` and `http_server` modules.

## Overview

The HTTP module provides an HTTP/1.1 implementation surface with:
- HTTP client for making requests (GET, POST, PUT, DELETE, etc.)
- HTTP server with routing support
- Request and response objects
- Header management
- Multiple HTTP methods and status codes

## Constants

### HTTP Methods

| Constant | Value | Description |
|----------|-------|-------------|
| `HTTP_GET` | 1 | GET method |
| `HTTP_POST` | 2 | POST method |
| `HTTP_PUT` | 3 | PUT method |
| `HTTP_DELETE` | 4 | DELETE method |
| `HTTP_PATCH` | 5 | PATCH method |
| `HTTP_HEAD` | 6 | HEAD method |
| `HTTP_OPTIONS` | 7 | OPTIONS method |

### HTTP Status Codes

| Constant | Value | Description |
|----------|-------|-------------|
| `HTTP_OK` | 200 | Success |
| `HTTP_CREATED` | 201 | Resource created |
| `HTTP_ACCEPTED` | 202 | Request accepted for processing |
| `HTTP_NO_CONTENT` | 204 | No content |
| `HTTP_MOVED_PERMANENTLY` | 301 | Resource moved permanently |
| `HTTP_FOUND` | 302 | Resource found at different URI |
| `HTTP_NOT_MODIFIED` | 304 | Resource not modified |
| `HTTP_BAD_REQUEST` | 400 | Bad request |
| `HTTP_UNAUTHORIZED` | 401 | Unauthorized |
| `HTTP_FORBIDDEN` | 403 | Forbidden |
| `HTTP_NOT_FOUND` | 404 | Not found |
| `HTTP_METHOD_NOT_ALLOWED` | 405 | HTTP method not allowed |
| `HTTP_CONFLICT` | 409 | Request conflict |
| `HTTP_INTERNAL_ERROR` | 500 | Internal server error |
| `HTTP_NOT_IMPLEMENTED` | 501 | Not implemented |
| `HTTP_BAD_GATEWAY` | 502 | Bad gateway |
| `HTTP_SERVICE_UNAVAILABLE` | 503 | Service unavailable |

### Buffer Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `HTTP_MAX_HEADER_SIZE` | 8192 | Maximum size for HTTP headers (8KB) |
| `HTTP_MAX_BODY_SIZE` | 1048576 | Maximum size for HTTP body (1MB) |
| `HTTP_DEFAULT_BUFFER` | 4096 | Default buffer size for reading (4KB) |

## Request

### Request Struct

```vais
struct Request {
    method: i64,
    path: str,
    version: str,
    headers: Headers,
    body: i64,
    body_len: i64
}
```

### Request Methods

#### new

```vais
fn new(method: i64, path: str) -> Request
```

Create a new HTTP request.

#### get / post / put / delete

```vais
fn get(path: str) -> Request
fn post(path: str) -> Request
fn put(path: str) -> Request
fn delete(path: str) -> Request
```

Create a request with the specified HTTP method.

#### with_body

```vais
fn with_body(&self, data: i64, len: i64) -> Request
```

Set the request body.

#### with_json

```vais
fn with_json(&self, json_str: str) -> Request
```

Set JSON body and Content-Type header.

#### header

```vais
fn header(&self, name: str, value: str) -> Request
```

Add a header to the request.

## Response

### Response Struct

```vais
struct Response {
    status: i64,
    status_text: str,
    version: str,
    headers: Headers,
    body: i64,
    body_len: i64
}
```

### Response Methods

#### new / ok / not_found / bad_request / internal_error

```vais
fn new(status: i64) -> Response
fn ok() -> Response
fn not_found() -> Response
fn bad_request() -> Response
fn internal_error() -> Response
```

Create response objects with common status codes.

#### with_body / with_text / with_json / with_html

```vais
fn with_body(&self, data: i64, len: i64) -> Response
fn with_text(&self, text: str) -> Response
fn with_json(&self, json_str: str) -> Response
fn with_html(&self, html: str) -> Response
```

Set response body with appropriate Content-Type.

#### Status checking

```vais
fn is_success(&self) -> i64
fn is_redirect(&self) -> i64
fn is_client_error(&self) -> i64
fn is_server_error(&self) -> i64
```

Check response status category.

## Client

### Client Struct

```vais
struct Client {
    timeout_ms: i64,
    follow_redirects: i64,
    max_redirects: i64
}
```

### Client Methods

#### new

```vais
fn new() -> Client
```

Create a new HTTP client with default settings.

#### execute

```vais
fn execute(&self, host: str, port: i64, request: &Request) -> Response?
```

Execute an HTTP request.

#### get / post

```vais
fn get(&self, url: str) -> Response?
fn post(&self, url: str, body: str) -> Response?
```

Convenience methods for common operations.

## Server

### Server Struct

```vais
struct Server {
    host: str,
    port: i64,
    router: Router,
    running: i64
}
```

### Server Methods

#### new

```vais
fn new(port: i64) -> Server
fn bind(host: str, port: i64) -> Server
```

Create a new HTTP server.

#### routes

```vais
fn routes(&self, router: Router) -> Server
```

Add routes to the server.

#### run

```vais
fn run(&self) -> i64
```

Run the server (blocking).

## Handler Trait

### Handler Trait

```vais
trait Handler {
    fn handle(&self, req: &Request) -> Response
}
```

The Handler trait defines the interface for request handlers. Any type implementing this trait can be used as a route handler in the HTTP server.

## Route

### Route Struct

```vais
struct Route {
    method: i64,
    path: str,
    handler_ptr: i64
}
```

Represents a single route mapping an HTTP method and path to a handler function.

## Router

### Router Struct

```vais
struct Router {
    routes: i64,
    count: i64,
    capacity: i64
}
```

### Router Methods

#### new

```vais
fn new() -> Router
```

Create a new router.

#### get / post / put / delete

```vais
fn get(&self, path: str, handler: i64) -> Router
fn post(&self, path: str, handler: i64) -> Router
fn put(&self, path: str, handler: i64) -> Router
fn delete(&self, path: str, handler: i64) -> Router
```

Add routes for specific HTTP methods.

## Convenience Functions

```vais
fn client() -> Client
fn server(port: i64) -> Server
fn router() -> Router
fn get(url: str) -> Response?
fn post(url: str, body: str) -> Response?
```

Quick access to common operations.

## Usage Examples

### HTTP Client - GET Request

```vais
# Simple GET request
response := get("http://api.example.com/users")

match response {
    Some(resp) => {
        I resp.is_success() == 1 {
            body := resp.body_text()
            # Process response body
        }
    }
    None => {
        # Handle error
    }
}
```

### HTTP Client - POST Request

```vais
client := Client::new()
request := Request::post("/api/users")
    .with_json('{"name":"Alice","age":30}')
    .header("Authorization", "Bearer token123")

response := client.execute("api.example.com", 80, &request)
```

### HTTP Server

```vais
# Define routes
router := router()
    .get("/", handle_index)
    .get("/users", handle_users)
    .post("/users", create_user)

# Start server
server := server(8080)
    .routes(router)
    .run()
```

### Custom Handler

```vais
fn handle_index(req: &Request) -> Response {
    Response::ok()
        .with_html("<h1>Welcome</h1>")
}

fn handle_users(req: &Request) -> Response {
    json := '{"users":["Alice","Bob","Charlie"]}'
    Response::ok()
        .with_json(json)
}
```
