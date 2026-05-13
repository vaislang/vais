# vais-server

vais-server is an Express/Axum-style backend API framework written in the **Vais language**. It is implemented entirely in pure Vais code without FFI, and serves as the HTTP layer of the VAIS full-stack ecosystem.

```
vais-web  (frontend + SSR)
    ↕  HTTP / WebSocket
vais-server  (backend API framework)   ← this package
    ↕  native query API
vaisdb  (vector + graph + relational + full-text search database)
```

---

## Features

### Express/Axum-style API

Route registration follows the Express.js convention. Handlers are registered by name string, and the runtime connects the actual function via symbolic dispatch.

```vais
app.get("/users",      "handle_list_users")
app.post("/users",     "handle_create_user")
app.put("/users/:id",  "handle_update_user")
app.delete("/users/:id", "handle_delete_user")
```

Route groups are organized with the `app.group("/prefix")` API, which corresponds to Axum's `Router::nest`.

```vais
api := mut app.group("/api/v1")
api.get("/posts",     "handle_list_posts")
api.post("/posts",    "handle_create_post")
```

### Minimal Core — App + Router + Middleware

All behavior is composed from three fundamental building blocks.

| Component | Role |
|-----------|------|
| `App` | Registers routes and middleware, starts the server |
| `Router` | RadixTree-based O(log n) URL matching |
| `Pipeline` | Symmetric before/after middleware pipeline |

### Built-in Authentication

An authentication module ready to use without additional libraries.

- **JWT** — HS256 signing, TokenPair (access + refresh), claim validation
- **OAuth 2.0** — authorization code flow, CSRF state management
- **Session** — server-side session store (TTL support)
- **Password** — bcrypt-style hashing and verification

### Multi-Protocol

Handles multiple protocols simultaneously in a single server instance.

- **REST** — HTTP/1.1-based CRUD API, pagination helpers
- **WebSocket** — RFC 6455, room-based broadcasting
- **GraphQL** — schema introspection, resolver dispatch
- **gRPC** — service descriptor, framing
- **OpenAPI** — 3.0 document auto-generation

### Native vaisdb Integration

`QueryBuilder` sends queries directly over the vaisdb wire protocol without an ORM translation layer. SQL, vector search, graph traversal, and full-text search are all handled through a single fluent API.

```vais
sql := QueryBuilder.new()
    .select("documents")
    .column("id")
    .column("title")
    .where_clause("published = 1")
    .order_by("created_at", SortDirection.Desc)
    .limit(20)
    .build()
```

### Pure Vais — Without FFI

The framework itself makes no FFI calls. External runtime functions (`current_time_ms`, `str_len`, etc.) are declared with `X F` and resolved by the `vaisc` linker. All dependencies come from the Vais standard library (`std/`).

| Import | Used for |
|--------|----------|
| `std/async_http` | HTTP/1.1 parsing |
| `std/http_server` | TCP connection accept loop |
| `std/websocket` | RFC 6455 framing |
| `std/vec` | Dynamic arrays |
| `std/option` | Optional values |

---

## Project Structure

```
vais-server/
├── src/
│   ├── main.vais          # entry point
│   ├── core/              # App, Config, Context, Error
│   ├── http/              # HttpMethod, HttpStatus, Request, Response
│   ├── router/            # RadixTree, Router, RouteGroup
│   ├── middleware/        # Pipeline, CORS, Logger, RateLimit, ...
│   ├── auth/              # JWT, OAuth2, Session, Guard, Password
│   ├── ws/                # WebSocket messages, handlers, Room
│   ├── db/                # DbConnection, Pool, QueryBuilder, Migrator
│   ├── api/               # REST, GraphQL, gRPC, OpenAPI
│   └── util/              # JSON, Validation, Env
├── tests/                 # per-module unit tests + integration tests
└── examples/              # hello.vais, rest_api.vais, chat.vais, fullstack.vais
```

---

## Build and Run

```sh
# Build
vaisc build src/main.vais -o vais-server

# Run
./vais-server

# Run all tests
vaisc test tests/

# Run a specific test file
vaisc test tests/router/test_router.vais
```

---

## Next Steps

- [Getting Started](./getting-started.md) — Run a Hello World server in 5 minutes
- [Routing Guide](./routing.md) — RadixTree router, route groups, middleware pipeline
- [Database Integration](./database.md) — vaisdb connection, ConnectionPool, QueryBuilder, migrations
