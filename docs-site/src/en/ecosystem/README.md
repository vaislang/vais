# Vais Ecosystem

A full-stack ecosystem built on top of the Vais language.

## Architecture

```
┌─────────────────────────────────────────────┐
│                   Client                     │
└─────────────────────┬───────────────────────┘
                      │ HTTP / WebSocket
┌─────────────────────▼───────────────────────┐
│  VaisX (vais-web)                           │
│  Compile-time reactive frontend framework    │
│  < 3KB runtime · SSR/SSG · file-based routing│
└─────────────────────┬───────────────────────┘
                      │ HTTP / WebSocket
┌─────────────────────▼───────────────────────┐
│  vais-server                                │
│  Express/Axum-style backend API framework   │
│  middleware pipeline · REST/GraphQL/gRPC    │
└─────────────────────┬───────────────────────┘
                      │ Native Query API
┌─────────────────────▼───────────────────────┐
│  VaisDB                                     │
│  RAG-native hybrid database                 │
│  Vector + Graph + SQL + Full-text           │
└─────────────────────────────────────────────┘
```

## Package summary

| Package | Description | Key features |
|---------|-------------|--------------|
| [VaisX](./vais-web/README.md) | Frontend framework | Compile-time reactivity, < 3 KB, SSR/SSG |
| [VaisDB](./vaisdb/README.md) | Hybrid database | 4-engine integration, ACID, RAG-native |
| [vais-server](./vais-server/README.md) | Backend framework | Middleware, multi-protocol, vaisdb integration |

## Full-stack example

```vais
# === Frontend (VaisX) ===
# app/page.vaisx
# <script>
todos := $state([])

A F load() -> Vec<Todo> {
    fetch("/api/todos").json()
}
# </script>
# <template>
#   @each todos as todo {
#     <li>{todo.text}</li>
#   }
# </template>

# === Backend (vais-server) ===
U core/app
U db/query

F handle_todos(ctx: Context) -> Response {
    sql := QueryBuilder.new()
        .select("todos")
        .order_by("id", SortDirection.Asc)
        .build()
    ctx.json(200, db.execute(sql))
}

F main() -> i64 {
    app := mut App.new(ServerConfig.default())
    app.get("/api/todos", "handle_todos")
    app.listen(":8080")
    0
}
```

## Getting started

- [VaisX Getting Started](./vais-web/getting-started.md)
- [VaisDB Getting Started](./vaisdb/getting-started.md)
- [vais-server Getting Started](./vais-server/getting-started.md)

## Source code

All ecosystem packages are maintained in the [vaislang/vais-lang](https://github.com/vaislang/vais-lang) monorepo.
