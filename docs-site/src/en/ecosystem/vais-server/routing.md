# Routing Guide

The routing system in vais-server is composed of three core components.

- **RadixTree router** — O(log n) URL matching, path parameter extraction
- **RouteGroup** — prefix-scoped sub-router
- **Pipeline** — symmetric before/after middleware pipeline

---

## RadixTree Router

Internally, an independent RadixTree is maintained for each HTTP method. When a request arrives, the path is matched against the tree for that method and the handler ID along with path parameters are returned.

### Route Registration

`App` provides convenience methods per HTTP method.

```vais
app.get("/users",           "handle_list_users")
app.post("/users",          "handle_create_user")
app.get("/users/:id",       "handle_get_user")
app.put("/users/:id",       "handle_update_user")
app.patch("/users/:id",     "handle_patch_user")
app.delete("/users/:id",    "handle_delete_user")
app.ws("/ws/chat",          "handle_ws_chat")
```

The second argument is the **name string** of the handler function. The runtime dispatches to the actual function based on this name (symbolic dispatch).

### Path Parameters

Declare dynamic segments using the `:param` format.

```vais
app.get("/articles/:slug/comments/:comment_id", "handle_get_comment")
```

Read parameter values inside the handler via `ctx.path_params`.

```vais
F handle_get_user(ctx: Context) -> Response {
    # ctx.path_params — "id=<value>" format string
    id := ctx.path_params

    pairs := Vec.new()
    pairs.push("id")
    pairs.push(id)
    pairs.push("name")
    pairs.push("Alice")
    ctx.json(200, json_encode(pairs))
}
```

### Match Results

The router returns one of three states.

| State | Meaning |
|-------|---------|
| `RouteMatchStatus.Found` | Both path and method matched |
| `RouteMatchStatus.NotFound` | No match for any method → 404 |
| `RouteMatchStatus.MethodNotAllowed` | Path matched but method differs → 405 |

---

## Route Groups

`app.group("/prefix")` returns a sub-router with the given prefix applied. Use it to logically group related routes.

```vais
F main() -> i64 {
    config := ServerConfig.default()
    app    := mut App.new(config)

    app.use("logger")
    app.use("cors")

    # /api/v1 group
    v1 := mut app.group("/api/v1")

    v1.get("/users",        "handle_list_users")
    v1.get("/users/:id",    "handle_get_user")
    v1.post("/users",       "handle_create_user")
    v1.put("/users/:id",    "handle_update_user")
    v1.delete("/users/:id", "handle_delete_user")

    # Merge group routes into the main app
    I i = 0; i < v1.route_count(); i = i + 1 {
        r := v1.routes.get(i)
        app._add_route(r.method, r.path, r.handler_id)
    }

    M app.listen(":8080") {
        Ok(_) => {},
        Err(e) => { println("Error: {e.message}") R 1 },
    }
    0
}
```

Groups can also be nested.

```vais
admin := mut app.group("/admin")
users := mut admin.group("/users")   # effective prefix: /admin/users
```

> **Note**: `app.group()` returns a new `App` instance. To use the registered routes in the main app, they must be explicitly merged via `_add_route`.

---

## Handlers

All handlers follow the same signature.

```vais
F <handler_name>(ctx: Context) -> Response {
    # ...
}
```

### Context Fields

| Field | Type | Content |
|-------|------|---------|
| `ctx.method` | `str` | HTTP method (`"GET"`, `"POST"`, etc.) |
| `ctx.path` | `str` | Request path (`"/users/42"`) |
| `ctx.path_params` | `str` | Path parameters (`"id=42"`) |
| `ctx.query_params` | `str` | Query string parameters |
| `ctx.body` | `str` | Request body |
| `ctx.request_id` | `str` | Unique request ID |
| `ctx.state` | `str` | State passed by middleware |

### Response Builder

```vais
# JSON response
F handle_list(ctx: Context) -> Response {
    ctx.json(200, "[{\"id\":1,\"name\":\"Alice\"}]")
}

# Error response
F handle_not_found(ctx: Context) -> Response {
    pairs := Vec.new()
    pairs.push("error")
    pairs.push("Resource not found.")
    ctx.json(404, json_encode(pairs))
}

# Created (201)
F handle_create(ctx: Context) -> Response {
    pairs := Vec.new()
    pairs.push("id")
    pairs.push("99")
    pairs.push("status")
    pairs.push("created")
    ctx.json(201, json_encode(pairs))
}

# No Content (204)
F handle_delete(ctx: Context) -> Response {
    ctx.status(204)
}

# Redirect
F handle_old_path(ctx: Context) -> Response {
    ctx.redirect("/new/path")
}
```

### Setting Response Headers

```vais
F handle_with_header(ctx: Context) -> Response {
    ctx2 := ctx.set_header("X-Request-Id", "abc-123")
    ctx2.json(200, "{\"ok\":true}")
}
```

`set_header` returns a new `Context`. Chain calls when setting multiple headers.

```vais
ctx2 := ctx.set_header("X-Foo", "bar").set_header("X-Baz", "qux")
```

---

## Middleware Pipeline

### Pipeline Model

The middleware pipeline follows the onion model of Koa.js.

```
Request →  before[0]  →  before[1]  →  before[2]  →  handler
                                                         ↓
Response ←  after[0]   ←  after[1]   ←  after[2]   ←  handler_response
```

- `before` hooks execute in registration order.
- `after` hooks execute in reverse registration order.
- If any `before` hook returns `BeforeResult.Respond`, subsequent before hooks and the handler are skipped. However, after hooks for already-executed middleware still run normally.

### Registering Built-in Middleware

```vais
app.use("logger")      # request logging
app.use("cors")        # CORS headers
app.use("rate_limit")  # rate limiting
app.use("compress")    # response compression
app.use("recovery")    # panic recovery
```

### Implementing Custom Middleware

Custom middleware is created as a struct implementing two functions: `before` and `after`.

```vais
U middleware/pipeline
U core/context

S AuthMiddleware {
    secret: str,
}

X AuthMiddleware {
    F new(secret: str) -> AuthMiddleware {
        AuthMiddleware { secret }
    }

    # before: validate the Authorization header
    F before(self, ctx: Context) -> BeforeResult {
        token := ctx.query_params   # in practice, extract from header
        I token == "" {
            pairs := Vec.new()
            pairs.push("error")
            pairs.push("Authentication token required.")
            err_response := ctx.json(401, json_encode(pairs))
            R BeforeResult.respond(err_response)
        }
        BeforeResult.next()
    }

    # after: add security headers to the response
    F after(self, ctx: Context, response: Response) -> Response {
        # in a real implementation, add headers to response before returning
        response
    }
}
```

`BeforeResult.next()` — proceed to the next middleware/handler.
`BeforeResult.respond(response)` — short-circuit the pipeline.

### Pipeline Internal Structure

```vais
S Pipeline {
    entries: Vec<PipelineEntry>,
    count:   i64,
}
```

- `pipeline.run_before(ctx)` → returns `PipelineBeforeOutput`
  - If `short_circuit: true`, the `response` field holds the early response.
  - If `short_circuit: false`, the handler must be called.
- `pipeline.run_after(ctx, handler_response)` → returns the final `Response`

Because Vais does not allow loops that mutate external structs, the pipeline internals are implemented with recursive helper functions (`pipeline_run_before`, `pipeline_run_after`).

---

## Full Routing Example — CRUD REST API

```vais
U core/app
U core/config
U core/context
U src/util/json

C PORT: u16 = 8080

F handle_list_users(ctx: Context) -> Response {
    user := Vec.new()
    user.push("id")
    user.push("1")
    user.push("name")
    user.push("Alice")
    body := "[" + json_encode(user) + "]"
    ctx.json(200, body)
}

F handle_get_user(ctx: Context) -> Response {
    id := ctx.path_params
    pairs := Vec.new()
    pairs.push("id")
    pairs.push(id)
    pairs.push("name")
    pairs.push("Alice")
    ctx.json(200, json_encode(pairs))
}

F handle_create_user(ctx: Context) -> Response {
    pairs := Vec.new()
    pairs.push("id")
    pairs.push("2")
    pairs.push("status")
    pairs.push("created")
    ctx.json(201, json_encode(pairs))
}

F handle_update_user(ctx: Context) -> Response {
    id := ctx.path_params
    pairs := Vec.new()
    pairs.push("id")
    pairs.push(id)
    pairs.push("status")
    pairs.push("updated")
    ctx.json(200, json_encode(pairs))
}

F handle_delete_user(ctx: Context) -> Response {
    ctx.status(204)
}

F main() -> i64 {
    config := ServerConfig.default()
    app    := mut App.new(config)

    app.use("logger")
    app.use("cors")

    v1 := mut app.group("/api/v1")
    v1.get("/users",        "handle_list_users")
    v1.get("/users/:id",    "handle_get_user")
    v1.post("/users",       "handle_create_user")
    v1.put("/users/:id",    "handle_update_user")
    v1.delete("/users/:id", "handle_delete_user")

    I i = 0; i < v1.route_count(); i = i + 1 {
        r := v1.routes.get(i)
        app._add_route(r.method, r.path, r.handler_id)
    }

    println("REST API server starting: :{PORT} (routes: {app.route_count()})")

    M app.listen(":{PORT}") {
        Ok(_) => {},
        Err(e) => { println("Error: {e.message}") R 1 },
    }
    0
}
```

---

## Next Steps

- [Database Integration](./database.md) — How to query real data with QueryBuilder and return it as a JSON response from a handler
