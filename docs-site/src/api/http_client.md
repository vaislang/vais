# HTTP Client API Reference

> Full-featured HTTP client library with request building, response handling, connection pooling, and TLS support

## Import

```vais
U std/http_client
```

## Overview

The HTTP Client module provides a complete HTTP/HTTPS client implementation with support for:
- All standard HTTP methods (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
- URL parsing and validation
- Header management (set, add, remove, get)
- Request/response body handling
- JSON, form, and text content types
- Authentication helpers (Bearer token, Basic auth)
- Connection pooling and keep-alive
- Automatic redirect following
- Timeout configuration
- TLS/HTTPS with SNI and certificate verification

## Constants

### HTTP Methods

| Constant | Value | Description |
|----------|-------|-------------|
| `CLIENT_GET` | 1 | GET method |
| `CLIENT_POST` | 2 | POST method |
| `CLIENT_PUT` | 3 | PUT method |
| `CLIENT_DELETE` | 4 | DELETE method |
| `CLIENT_PATCH` | 5 | PATCH method |
| `CLIENT_HEAD` | 6 | HEAD method |
| `CLIENT_OPTIONS` | 7 | OPTIONS method |

### Buffer Sizes

| Constant | Value | Description |
|----------|-------|-------------|
| `CLIENT_MAX_HEADERS` | 8192 | Maximum header buffer size |
| `CLIENT_MAX_BODY` | 1048576 | Maximum body size (1MB) |
| `CLIENT_RECV_CHUNK` | 4096 | Receive buffer chunk size |
| `CLIENT_MAX_URL_LEN` | 2048 | Maximum URL length |

### Default Configuration

| Constant | Value | Description |
|----------|-------|-------------|
| `CLIENT_DEFAULT_TIMEOUT` | 30000 | Default timeout (30 seconds) |
| `CLIENT_DEFAULT_MAX_REDIRECTS` | 10 | Maximum redirects to follow |
| `CLIENT_DEFAULT_PORT_HTTP` | 80 | Default HTTP port |
| `CLIENT_DEFAULT_PORT_HTTPS` | 443 | Default HTTPS port |

### Connection Pool

| Constant | Value | Description |
|----------|-------|-------------|
| `POOL_MAX_CONNECTIONS` | 16 | Maximum pooled connections |
| `POOL_ENTRY_SIZE` | 32 | Pool entry size in bytes |

### Error Codes

| Constant | Value | Description |
|----------|-------|-------------|
| `CLIENT_ERR_NONE` | 0 | No error |
| `CLIENT_ERR_DNS` | -1 | DNS resolution failed |
| `CLIENT_ERR_CONNECT` | -2 | Connection failed |
| `CLIENT_ERR_SEND` | -3 | Send failed |
| `CLIENT_ERR_RECV` | -4 | Receive failed |
| `CLIENT_ERR_TIMEOUT` | -5 | Request timed out |
| `CLIENT_ERR_PARSE` | -6 | Response parse error |
| `CLIENT_ERR_TOO_MANY_REDIRECTS` | -7 | Too many redirects |
| `CLIENT_ERR_INVALID_URL` | -8 | Invalid URL |
| `CLIENT_ERR_TLS_INIT` | -9 | TLS initialization failed |
| `CLIENT_ERR_TLS_HANDSHAKE` | -10 | TLS handshake failed |

## Structs

### Url

URL component parser and builder.

| Field | Type | Description |
|-------|------|-------------|
| `scheme` | `str` | Protocol ("http" or "https") |
| `host` | `str` | Hostname or IP address |
| `port` | `i64` | Port number (default: 80/443) |
| `path` | `str` | Path component |
| `query` | `str` | Query string (without ?) |
| `raw` | `str` | Original URL string |

**Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `parse` | `F parse(raw_url: str) -> Url` | Parse URL string into components |
| `is_https` | `F is_https(&self) -> i64` | Check if URL uses HTTPS (1=yes, 0=no) |
| `request_path` | `F request_path(&self) -> str` | Get path with query string |
| `host_header` | `F host_header(&self) -> str` | Get Host header value (host:port) |

### HttpRequest

HTTP request builder with chainable methods.

| Field | Type | Description |
|-------|------|-------------|
| `method` | `i64` | HTTP method constant |
| `url` | `Url` | Parsed URL |
| `headers` | `i64` | Pointer to header array |
| `header_count` | `i64` | Number of headers |
| `header_capacity` | `i64` | Header array capacity |
| `body` | `i64` | Pointer to body data |
| `body_len` | `i64` | Body length in bytes |
| `timeout_ms` | `i64` | Request timeout in milliseconds |

**Constructors:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(method: i64, url_str: str) -> HttpRequest` | Create request with method and URL |
| `get` | `F get(url: str) -> HttpRequest` | Create GET request |
| `post` | `F post(url: str) -> HttpRequest` | Create POST request |
| `put` | `F put(url: str) -> HttpRequest` | Create PUT request |
| `delete` | `F delete(url: str) -> HttpRequest` | Create DELETE request |
| `patch` | `F patch(url: str) -> HttpRequest` | Create PATCH request |
| `head` | `F head(url: str) -> HttpRequest` | Create HEAD request |
| `options` | `F options(url: str) -> HttpRequest` | Create OPTIONS request |

**Header Management:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `add_header` | `F add_header(&self, name: str, value: str) -> HttpRequest` | Add header (allows duplicates) |
| `set_header` | `F set_header(&self, name: str, value: str) -> HttpRequest` | Set/replace header |
| `remove_header` | `F remove_header(&self, name: str) -> HttpRequest` | Remove header by name |
| `get_header` | `F get_header(&self, name: str) -> str?` | Get header value (Option) |

**Body Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `with_body` | `F with_body(&self, data: i64, len: i64) -> HttpRequest` | Set raw body data |
| `with_text` | `F with_text(&self, text: str) -> HttpRequest` | Set text body (text/plain) |
| `with_json` | `F with_json(&self, json_str: str) -> HttpRequest` | Set JSON body (application/json) |
| `with_form` | `F with_form(&self, form_data: str) -> HttpRequest` | Set form body (application/x-www-form-urlencoded) |

**Configuration:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `with_timeout` | `F with_timeout(&self, ms: i64) -> HttpRequest` | Set request timeout |
| `with_bearer_token` | `F with_bearer_token(&self, token: str) -> HttpRequest` | Set Bearer token authentication |
| `with_basic_auth` | `F with_basic_auth(&self, encoded: str) -> HttpRequest` | Set Basic authentication |

**Other Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `method_str` | `F method_str(&self) -> str` | Get method as string ("GET", "POST", etc.) |
| `serialize` | `F serialize(&self, buffer: i64, buffer_size: i64) -> i64` | Serialize request to buffer |
| `drop` | `F drop(&self) -> i64` | Free allocated memory |

### HttpResponse

Parsed HTTP response with status, headers, and body.

| Field | Type | Description |
|-------|------|-------------|
| `status` | `i64` | HTTP status code (200, 404, etc.) |
| `status_text` | `str` | Status text ("OK", "Not Found", etc.) |
| `version` | `str` | HTTP version ("HTTP/1.1") |
| `headers` | `i64` | Pointer to header array |
| `header_count` | `i64` | Number of headers |
| `header_capacity` | `i64` | Header array capacity |
| `body` | `i64` | Pointer to body data |
| `body_len` | `i64` | Body length in bytes |
| `error_code` | `i64` | Error code (0 on success) |

**Constructors:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `error` | `F error(code: i64) -> HttpResponse` | Create error response |

**Status Checking:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `is_ok` | `F is_ok(&self) -> i64` | Check if no transport error (1=ok, 0=error) |
| `is_success` | `F is_success(&self) -> i64` | Check if status is 2xx |
| `is_redirect` | `F is_redirect(&self) -> i64` | Check if status is 3xx |
| `is_client_error` | `F is_client_error(&self) -> i64` | Check if status is 4xx |
| `is_server_error` | `F is_server_error(&self) -> i64` | Check if status is 5xx |

**Content Access:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `body_text` | `F body_text(&self) -> str` | Get body as string |
| `get_header` | `F get_header(&self, name: str) -> str?` | Get header value (case-insensitive) |
| `has_header` | `F has_header(&self, name: str) -> i64` | Check if header exists |
| `content_type` | `F content_type(&self) -> str?` | Get Content-Type header |
| `content_length` | `F content_length(&self) -> i64` | Get Content-Length (-1 if not present) |
| `location` | `F location(&self) -> str?` | Get Location header (for redirects) |

**Error Handling:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `error_text` | `F error_text(&self) -> str` | Get error description string |
| `drop` | `F drop(&self) -> i64` | Free allocated memory |

### ConnectionPool

Connection pool for keep-alive connection reuse.

| Field | Type | Description |
|-------|------|-------------|
| `entries` | `i64` | Pointer to pool entry array |
| `count` | `i64` | Number of active entries |
| `capacity` | `i64` | Maximum capacity |

**Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> ConnectionPool` | Create new connection pool |
| `get` | `F get(&self, host: str, port: i64) -> i64` | Get cached connection fd (-1 if none) |
| `put` | `F put(&self, host: str, port: i64, fd: i64) -> i64` | Return connection to pool |
| `close_all` | `F close_all(&self) -> i64` | Close all pooled connections |
| `drop` | `F drop(&self) -> i64` | Free pool memory |

### HttpClient

Configurable HTTP client with connection pooling and redirect support.

| Field | Type | Description |
|-------|------|-------------|
| `timeout_ms` | `i64` | Default timeout in milliseconds |
| `follow_redirects` | `i64` | Enable redirect following (1=on, 0=off) |
| `max_redirects` | `i64` | Maximum redirects to follow |
| `keep_alive` | `i64` | Enable keep-alive (1=on, 0=off) |
| `pool` | `ConnectionPool` | Connection pool |
| `default_headers` | `i64` | Pointer to default header array |
| `default_header_count` | `i64` | Number of default headers |
| `default_header_capacity` | `i64` | Default header capacity |

**Constructor:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> HttpClient` | Create HTTP client with defaults |

**Configuration:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `timeout` | `F timeout(&self, ms: i64) -> HttpClient` | Set default timeout |
| `follow_redirects` | `F follow_redirects(&self, follow: i64) -> HttpClient` | Enable/disable redirects |
| `max_redirects` | `F max_redirects(&self, max: i64) -> HttpClient` | Set maximum redirects |
| `keep_alive` | `F keep_alive(&self, enabled: i64) -> HttpClient` | Enable/disable keep-alive |
| `default_header` | `F default_header(&self, name: str, value: str) -> HttpClient` | Add default header |

**Request Execution:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `send` | `F send(&self, request: &HttpRequest) -> HttpResponse` | Execute HTTP request |

**Convenience Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `get` | `F get(&self, url: str) -> HttpResponse` | Send GET request |
| `post_json` | `F post_json(&self, url: str, json_body: str) -> HttpResponse` | Send POST with JSON body |
| `post_form` | `F post_form(&self, url: str, form_data: str) -> HttpResponse` | Send POST with form data |
| `post_text` | `F post_text(&self, url: str, text: str) -> HttpResponse` | Send POST with text body |
| `put_json` | `F put_json(&self, url: str, json_body: str) -> HttpResponse` | Send PUT with JSON body |
| `patch_json` | `F patch_json(&self, url: str, json_body: str) -> HttpResponse` | Send PATCH with JSON body |
| `delete` | `F delete(&self, url: str) -> HttpResponse` | Send DELETE request |
| `head` | `F head(&self, url: str) -> HttpResponse` | Send HEAD request |

**Cleanup:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `close` | `F close(&self) -> i64` | Close all pooled connections |
| `drop` | `F drop(&self) -> i64` | Free all memory |

## JSON Helper Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `json_kv` | `F json_kv(key: str, value: str) -> str` | Build JSON object with one key-value pair |
| `json_kv2` | `F json_kv2(k1: str, v1: str, k2: str, v2: str) -> str` | Build JSON object with two key-value pairs |
| `json_kv_int` | `F json_kv_int(key: str, value: i64) -> str` | Build JSON object with integer value |

## Top-Level Convenience Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `http_client` | `F http_client() -> HttpClient` | Create new HTTP client |
| `http_get` | `F http_get(url: str) -> HttpResponse` | Simple GET request (one-shot) |
| `http_post` | `F http_post(url: str, json_body: str) -> HttpResponse` | Simple POST with JSON (one-shot) |
| `http_put` | `F http_put(url: str, json_body: str) -> HttpResponse` | Simple PUT with JSON (one-shot) |
| `http_delete` | `F http_delete(url: str) -> HttpResponse` | Simple DELETE request (one-shot) |
| `http_patch` | `F http_patch(url: str, json_body: str) -> HttpResponse` | Simple PATCH with JSON (one-shot) |

## Usage Examples

### Simple GET Request

```vais
U std/http_client

F main() -> i64 {
    resp := http_get("https://api.example.com/users")

    I resp.is_success() == 1 {
        body := resp.body_text()
        __println(body)
        __println("Status: ", resp.status)
    } E {
        __println("Error: ", resp.error_text())
    }

    resp.drop()
    0
}
```

### POST JSON Request

```vais
U std/http_client

F main() -> i64 {
    json := json_kv2("name", "Alice", "email", "alice@example.com")
    resp := http_post("https://api.example.com/users", json)

    I resp.is_success() == 1 {
        __println("Created: ", resp.body_text())
    }

    resp.drop()
    0
}
```

### Custom Request with Headers

```vais
U std/http_client

F main() -> i64 {
    req := HttpRequest::get("https://api.example.com/protected")
        .with_bearer_token("eyJhbGc...")
        .set_header("X-Custom-Header", "value")
        .with_timeout(10000)  # 10 seconds

    client := HttpClient::new()
    resp := client.send(&req)

    I resp.is_success() == 1 {
        __println("Response: ", resp.body_text())
    }

    req.drop()
    resp.drop()
    client.drop()
    0
}
```

### Reusable Client with Connection Pooling

```vais
U std/http_client

F main() -> i64 {
    client := HttpClient::new()
        .timeout(5000)
        .follow_redirects(1)
        .keep_alive(1)
        .default_header("User-Agent", "MyApp/1.0")

    # First request (creates connection)
    resp1 := client.get("http://example.com/api/users")
    __println("Status: ", resp1.status)
    resp1.drop()

    # Second request (reuses connection)
    resp2 := client.get("http://example.com/api/posts")
    __println("Status: ", resp2.status)
    resp2.drop()

    client.drop()
    0
}
```

### PUT Request with JSON

```vais
U std/http_client

F main() -> i64 {
    json := json_kv2("name", "Bob", "status", "active")

    req := HttpRequest::put("https://api.example.com/users/123")
        .with_json(json)
        .with_bearer_token("token123")

    client := HttpClient::new()
    resp := client.send(&req)

    I resp.is_success() == 1 {
        __println("Updated successfully")
    } E {
        __println("Error: ", resp.status, " - ", resp.error_text())
    }

    req.drop()
    resp.drop()
    client.drop()
    0
}
```

### Handling Response Headers

```vais
U std/http_client

F main() -> i64 {
    resp := http_get("https://api.example.com/data")

    # Get specific headers
    content_type := resp.content_type()
    M content_type {
        Some(ct) => __println("Content-Type: ", ct),
        None => __println("No Content-Type header")
    }

    content_len := resp.content_length()
    I content_len >= 0 {
        __println("Content-Length: ", content_len)
    }

    # Check if header exists
    I resp.has_header("X-Custom-Header") == 1 {
        custom := resp.get_header("X-Custom-Header")
        M custom {
            Some(val) => __println("Custom: ", val),
            None => 0
        }
    }

    resp.drop()
    0
}
```

### Basic Authentication

```vais
U std/http_client

F main() -> i64 {
    # Encode "user:password" in base64
    encoded := "dXNlcjpwYXNzd29yZA=="

    req := HttpRequest::get("https://api.example.com/secure")
        .with_basic_auth(encoded)

    client := HttpClient::new()
    resp := client.send(&req)

    I resp.status == 200 {
        __println("Authenticated: ", resp.body_text())
    } E I resp.status == 401 {
        __println("Authentication failed")
    }

    req.drop()
    resp.drop()
    client.drop()
    0
}
```

### Redirect Handling

```vais
U std/http_client

F main() -> i64 {
    client := HttpClient::new()
        .follow_redirects(1)
        .max_redirects(5)

    resp := client.get("http://example.com/redirect")

    I resp.is_success() == 1 {
        __println("Final response: ", resp.body_text())
    } E I resp.error_code == CLIENT_ERR_TOO_MANY_REDIRECTS {
        __println("Too many redirects")
    }

    resp.drop()
    client.drop()
    0
}
```

### Error Handling

```vais
U std/http_client

F main() -> i64 {
    resp := http_get("https://invalid-domain-xyz.com")

    I resp.is_ok() == 0 {
        M resp.error_code {
            CLIENT_ERR_DNS => __println("DNS resolution failed"),
            CLIENT_ERR_CONNECT => __println("Connection failed"),
            CLIENT_ERR_TIMEOUT => __println("Request timed out"),
            CLIENT_ERR_TLS_INIT => __println("TLS initialization failed"),
            CLIENT_ERR_TLS_HANDSHAKE => __println("TLS handshake failed"),
            _ => __println("Unknown error: ", resp.error_text())
        }
    } E {
        __println("Status: ", resp.status)
    }

    resp.drop()
    0
}
```

### Form Data POST

```vais
U std/http_client

F main() -> i64 {
    form := "username=alice&password=secret123&remember=true"

    req := HttpRequest::post("https://example.com/login")
        .with_form(form)

    client := HttpClient::new()
    resp := client.send(&req)

    I resp.is_redirect() == 1 {
        location := resp.location()
        M location {
            Some(url) => __println("Redirect to: ", url),
            None => __println("No location header")
        }
    }

    req.drop()
    resp.drop()
    client.drop()
    0
}
```

## Notes

- HTTPS requires TLS support compiled into the runtime
- Connection pooling is only available for plain HTTP connections (not HTTPS)
- Default headers are applied to all requests sent through the client
- Redirects automatically convert POST/PUT/PATCH to GET for 301/302/303 status codes
- The client automatically adds default headers: User-Agent, Accept, Connection
- All strings returned by the API should be freed by the caller when appropriate
- Use `drop()` methods to free allocated memory for requests, responses, and clients
