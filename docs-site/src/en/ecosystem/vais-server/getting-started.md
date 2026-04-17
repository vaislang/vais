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
U core/app
U core/config
U core/context

C PORT: u16 = 8080

# GET / handler — plain text response
F handle_hello(ctx: Context) -> Response {
    ctx.text(200, "Hello, World!")
}

F main() -> i64 {
    config := ServerConfig.default()
    app    := mut App.new(config)

    app.get("/", "handle_hello")

    addr := ":{PORT}"
    println("Server starting: {addr}")

    M app.listen(addr) {
        Ok(_) => {
            println("Server shut down cleanly.")
        },
        Err(e) => {
            println("Failed to start server: {e.message}")
            R 1
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
U core/app      # App struct — route/middleware registration, listen()
U core/config   # ServerConfig — server configuration
U core/context  # Context — request context, Response builder
```

The `U` keyword is for module imports. Paths are relative to `src/`.

### Handler Signature

```vais
F handle_hello(ctx: Context) -> Response {
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
M app.listen(addr) {
    Ok(_)  => { println("Clean shutdown") },
    Err(e) => { println("Error: {e.message}") R 1 },
}
```

`M` is the match expression. `app.listen()` returns `Result<(), VaisServerError>`, so it must always be handled.

---

## JSON API Server

An example that returns JSON responses in addition to plain text.

```vais
U core/app
U core/config
U core/context
U src/util/json

C PORT: u16 = 8080

# GET /ping — JSON health check
F handle_ping(ctx: Context) -> Response {
    pairs := Vec.new()
    pairs.push("status")
    pairs.push("ok")
    pairs.push("version")
    pairs.push("1.0.0")
    ctx.json(200, json_encode(pairs))
}

# GET /hello/:name — using path parameters
F handle_greet(ctx: Context) -> Response {
    name := ctx.path_params   # "name=<value>" format
    pairs := Vec.new()
    pairs.push("message")
    pairs.push("Hello, " + name + "!")
    ctx.json(200, json_encode(pairs))
}

F main() -> i64 {
    config := ServerConfig.default()
    app    := mut App.new(config)

    app.get("/ping",         "handle_ping")
    app.get("/hello/:name",  "handle_greet")

    println("JSON API server starting: :{PORT}")
    M app.listen(":{PORT}") {
        Ok(_) => {},
        Err(e) => {
            println("Error: {e.message}")
            R 1
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
F main() -> i64 {
    config := ServerConfig.default()
    app    := mut App.new(config)

    # Register global middleware (before routes)
    app.use("logger")
    app.use("cors")

    app.get("/", "handle_hello")

    M app.listen(":{PORT}") {
        Ok(_) => {},
        Err(e) => { println("Error: {e.message}") R 1 },
    }
    0
}
```

---

## ServerConfig

`ServerConfig.default()` uses default settings. For fine-grained configuration, construct `ServerConfig` directly.

```vais
config := ServerConfig {
    host:         "0.0.0.0",
    port:         8080,
    max_body_size: 1048576,   # 1 MB
    timeout_ms:   30000,
}
app := mut App.new(config)
```

---

## Next Steps

- [Routing Guide](./routing.md) — RadixTree router, route groups, and middleware pipeline in detail
- [Database Integration](./database.md) — vaisdb connection, QueryBuilder, and migrations
