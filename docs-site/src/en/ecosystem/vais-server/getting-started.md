# Getting Started

This guide walks you through running your first HTTP server with vais-server, covering basic routing and JSON responses.

---

## Prerequisites

- The `vaisc` compiler must be installed.
- The vais-server package must be located at `packages/vais-server/`.

---

## Hello World Server

### 1. Write the Source File

Create `src/main.vais` and enter the following content.

```vais
use core/app
use core/config
use core/context

C PORT: u16 = 8080

# GET / handler — plain text response
fn handle_hello(ctx: Context) -> Response {
    ctx.text(200, "Hello, World!")
}

fn main() -> i64 {
    config := ServerConfig.default()
    app    := mut App.new(config)

    app.get("/", "handle_hello")

    addr := ":{PORT}"
    println("Server starting: {addr}")

    match app.listen(addr) {
        Ok(_) => {
            println("Server shut down cleanly.")
        },
        Err(e) => {
            println("Failed to start server: {e.message}")
            return 1
        },
    }
    0
}
```

### 2. Build and Run

```sh
vaisc build src/main.vais -o hello-server
./hello-server
```

```
Server starting: :8080
```

### 3. Verify the Request

```sh
curl http://localhost:8080/
# Hello, World!
```

---

## Understanding the Code Structure

### Imports

```vais
use core/app      # App struct — route/middleware registration, listen()
use core/config   # ServerConfig — server configuration
use core/context  # Context — request context, Response builder
```

The `U` keyword is for module imports. Paths are relative to `src/`.

### Handler Signature

```vais
fn handle_hello(ctx: Context) -> Response {
    ctx.text(200, "Hello, World!")
}
```

All handlers have the form `F(ctx: Context) -> Response`. The `ctx` carries the request method, path, path parameters, query parameters, and body.

### Route Registration — Symbolic Dispatch

```vais
app.get("/", "handle_hello")
```

The second argument is the function name as a **string**. Because the current version of Vais does not support first-class function pointers, the runtime dispatches to the actual function by name.

### Handling Results

```vais
match app.listen(addr) {
    Ok(_)  => { println("Clean shutdown") },
    Err(e) => { println("Error: {e.message}") return 1 },
}
```

`M` is the match expression. `app.listen()` returns `Result<(), VaisServerError>`, so it must always be handled.

---

## JSON API Server

An example that returns JSON responses in addition to plain text.

```vais
use core/app
use core/config
use core/context
use src/util/json

C PORT: u16 = 8080

# GET /ping — JSON health check
fn handle_ping(ctx: Context) -> Response {
    pairs := Vec.new()
    pairs.push("status")
    pairs.push("ok")
    pairs.push("version")
    pairs.push("1.0.0")
    ctx.json(200, json_encode(pairs))
}

# GET /hello/:name — using path parameters
fn handle_greet(ctx: Context) -> Response {
    name := ctx.path_params   # "name=<value>" format
    pairs := Vec.new()
    pairs.push("message")
    pairs.push("Hello, " + name + "!")
    ctx.json(200, json_encode(pairs))
}

fn main() -> i64 {
    config := ServerConfig.default()
    app    := mut App.new(config)

    app.get("/ping",         "handle_ping")
    app.get("/hello/:name",  "handle_greet")

    println("JSON API server starting: :{PORT}")
    match app.listen(":{PORT}") {
        Ok(_) => {},
        Err(e) => {
            println("Error: {e.message}")
            return 1
        },
    }
    0
}
```

```sh
curl http://localhost:8080/ping
# {"status":"ok","version":"1.0.0"}

curl http://localhost:8080/hello/Alice
# {"message":"Hello, Alice!"}
```

### Context Response Builder

| Method | Content-Type | Purpose |
|--------|-------------|---------|
| `ctx.text(status, body)` | `text/plain; charset=utf-8` | Plain text |
| `ctx.json(status, body)` | `application/json; charset=utf-8` | JSON |
| `ctx.html(status, body)` | `text/html; charset=utf-8` | HTML |
| `ctx.redirect(url)` | — | 302 redirect |
| `ctx.status(code)` | — | Status code with no body |

---

## Adding Middleware

```vais
app.use("logger")   # request/response logging
app.use("cors")     # automatic CORS headers
```

The value passed to `app.use()` is the middleware name as a string. Built-in middleware includes `logger`, `cors`, `rate_limit`, `compress`, and `recovery`.

Middleware before hooks run in registration order; after hooks run in reverse order.

```vais
fn main() -> i64 {
    config := ServerConfig.default()
    app    := mut App.new(config)

    # Register global middleware (before routes)
    app.use("logger")
    app.use("cors")

    app.get("/", "handle_hello")

    match app.listen(":{PORT}") {
        Ok(_) => {},
        Err(e) => { println("Error: {e.message}") return 1 },
    }
    0
}
```

---

## ServerConfig

`ServerConfig.default()` is a local in-process helper: port `8080`, host
`0.0.0.0`, env `dev`, max connections `1000`, read/write timeout `5000`, and
log level `info`. For local examples, construct `ServerConfig` directly.

```vais
config := ServerConfig {
    port:             8080,
    host:             "0.0.0.0",
    env:              "dev",
    max_connections:  1000,
    read_timeout_ms:  5000,
    write_timeout_ms: 5000,
    log_level:        "info",
}
app := mut App.new(config)
```

This is not environment-variable loading, config-file loading, CLI precedence,
socket binding proof, TLS/proxy/DNS behavior, or a production deployment
contract.

---

## Next Steps

- [Routing Guide](./routing.md) — RadixTree router, route groups, and middleware pipeline in detail
- [Database Integration](./database.md) — vaisdb connection, QueryBuilder, and migrations
