# URL API Reference

> URL parsing, validation, and string conversion

## Import

```vais
U std/url
```

## Struct

```vais
S Url {
    scheme: String,    # Protocol (http, https, etc.)
    username: String,  # Optional username
    password: String,  # Optional password
    host: String,      # Hostname or IP
    port: i64,         # Port (0 if not specified)
    path: String,      # Path component
    query: String,     # Query string (without '?')
    fragment: String   # Fragment (without '#')
}
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> Url` | Create empty URL |
| `drop` | `F drop(&self) -> i64` | Free all memory |
| `to_string` | `F to_string(&self) -> String` | Convert to string |
| `full_path` | `F full_path(&self) -> String` | Get full path (path + query + fragment) |
| `default_port` | `F default_port(&self) -> i64` | Get default port for scheme |
| `effective_port` | `F effective_port(&self) -> i64` | Get effective port (specified or default) |
| `is_secure` | `F is_secure(&self) -> i64` | Check if scheme is secure (https, ftps, wss) |

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `url_parse` | `F url_parse(s: String) -> Option` | Parse URL from String |
| `url_parse_cstr` | `F url_parse_cstr(cstr: i64) -> Option` | Parse URL from C string |
| `url_to_string` | `F url_to_string(url: Url) -> i64` | Convert to C string (caller must free) |
| `url_encode` | `F url_encode(s: String) -> String` | URL-encode string (percent-encoding) |
| `url_decode` | `F url_decode(s: String) -> String` | URL-decode string (percent-decoding) |

## Usage

```vais
U std/url

F main() -> i64 {
    u := url_parse("https://example.com:8080/api?q=test#section")
    # u.scheme = "https", u.host = "example.com", u.port = 8080
    u.drop()
    0
}
```
