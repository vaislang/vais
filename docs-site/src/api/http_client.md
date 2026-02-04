# HTTP Client API Reference

> Full-featured HTTP client with request/response handling

## Import

```vais
U std/http_client
```

## Features

- GET/POST/PUT/DELETE/PATCH convenience functions
- URL parsing (host, port, path, query extraction)
- Header management
- Response parsing (status, headers, body)
- Timeout support
- Connection pooling basics

## Constants

| Name | Value | Description |
|------|-------|-------------|
| `CLIENT_GET` | 1 | GET method |
| `CLIENT_POST` | 2 | POST method |
| `CLIENT_PUT` | 3 | PUT method |
| `CLIENT_DELETE` | 4 | DELETE method |
| `CLIENT_PATCH` | 5 | PATCH method |

## Key Structs

### HttpRequest

Request builder with method, URL, headers, and body.

### HttpResponse

Parsed response with status code, headers, and body.

## Key Functions

| Function | Description |
|----------|-------------|
| `http_get(url)` | Send GET request |
| `http_post(url, body, body_len)` | Send POST request |
| `http_request(method, url, body, len)` | Send custom request |

## Usage

```vais
U std/http_client

F main() -> i64 {
    resp := http_get("http://example.com/api")
    # Process response status, headers, body
    0
}
```
