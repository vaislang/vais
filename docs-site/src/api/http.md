# HTTP API Reference

> HTTP client and server implementation built on TCP networking

## Overview

The HTTP module provides a full-featured HTTP/1.1 implementation with:
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
S Request {
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
F new(method: i64, path: str) -> Request
```

Create a new HTTP request.

#### get / post / put / delete

```vais
F get(path: str) -> Request
F post(path: str) -> Request
F put(path: str) -> Request
F delete(path: str) -> Request
```

Create a request with the specified HTTP method.

#### with_body

```vais
F with_body(&self, data: i64, len: i64) -> Request
```

Set the request body.

#### with_json

```vais
F with_json(&self, json_str: str) -> Request
```

Set JSON body and Content-Type header.

#### header

```vais
F header(&self, name: str, value: str) -> Request
```

Add a header to the request.

## Response

### Response Struct

```vais
S Response {
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
F new(status: i64) -> Response
F ok() -> Response
F not_found() -> Response
F bad_request() -> Response
F internal_error() -> Response
```

Create response objects with common status codes.

#### with_body / with_text / with_json / with_html

```vais
F with_body(&self, data: i64, len: i64) -> Response
F with_text(&self, text: str) -> Response
F with_json(&self, json_str: str) -> Response
F with_html(&self, html: str) -> Response
```

Set response body with appropriate Content-Type.

#### Status checking

```vais
F is_success(&self) -> i64
F is_redirect(&self) -> i64
F is_client_error(&self) -> i64
F is_server_error(&self) -> i64
```

Check response status category.

## Client

### Client Struct

```vais
S Client {
    timeout_ms: i64,
    follow_redirects: i64,
    max_redirects: i64
}
```

### Client Methods

#### new

```vais
F new() -> Client
```

Create a new HTTP client with default settings.

#### execute

```vais
F execute(&self, host: str, port: i64, request: &Request) -> Response?
```

Execute an HTTP request.

#### get / post

```vais
F get(&self, url: str) -> Response?
F post(&self, url: str, body: str) -> Response?
```

Convenience methods for common operations.

## Server

### Server Struct

```vais
S Server {
    host: str,
    port: i64,
    router: Router,
    running: i64
}
```

### Server Methods

#### new

```vais
F new(port: i64) -> Server
F bind(host: str, port: i64) -> Server
```

Create a new HTTP server.

#### routes

```vais
F routes(&self, router: Router) -> Server
```

Add routes to the server.

#### run

```vais
F run(&self) -> i64
```

Run the server (blocking).

## Handler Trait

### Handler Trait

```vais
W Handler {
    F handle(&self, req: &Request) -> Response
}
```

The Handler trait defines the interface for request handlers. Any type implementing this trait can be used as a route handler in the HTTP server.

## Route

### Route Struct

```vais
S Route {
    method: i64,
    path: str,
    handler_ptr: i64
}
```

Represents a single route mapping an HTTP method and path to a handler function.

## Router

### Router Struct

```vais
S Router {
    routes: i64,
    count: i64,
    capacity: i64
}
```

### Router Methods

#### new

```vais
F new() -> Router
```

Create a new router.

#### get / post / put / delete

```vais
F get(&self, path: str, handler: i64) -> Router
F post(&self, path: str, handler: i64) -> Router
F put(&self, path: str, handler: i64) -> Router
F delete(&self, path: str, handler: i64) -> Router
```

Add routes for specific HTTP methods.

## Convenience Functions

```vais
F client() -> Client
F server(port: i64) -> Server
F router() -> Router
F get(url: str) -> Response?
F post(url: str, body: str) -> Response?
```

Quick access to common operations.

## Usage Examples

### HTTP Client - GET Request

```vais
# Simple GET request
response := get("http://api.example.com/users")

M response {
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
F handle_index(req: &Request) -> Response {
    Response::ok()
        .with_html("<h1>Welcome</h1>")
}

F handle_users(req: &Request) -> Response {
    json := '{"users":["Alice","Bob","Charlie"]}'
    Response::ok()
        .with_json(json)
}
```
