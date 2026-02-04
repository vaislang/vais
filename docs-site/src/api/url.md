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

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `url_parse` | `F url_parse(s: i64) -> Url` | Parse URL from C string |
| `url_to_string` | `F url_to_string(url: Url) -> String` | Convert to string |

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
