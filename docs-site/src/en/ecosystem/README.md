# Vais Ecosystem

A full-stack ecosystem workbench built on top of the Vais language.

Current public claims are gate-backed: VaisDB package `261/261` and runtime
`34/34`, vais-server runtime `20/20`, vais-web runtime `61/77`, unit
`390/390`, package `3272/3272`, full-build `24/24`, cross-package schema
`15/15`, and multi-domain product schema `9/9`. These are not
product-complete v1 claims.

## Architecture

```
┌─────────────────────────────────────────────┐
│                   Client                     │
└─────────────────────┬───────────────────────┘
                      │ HTTP / WebSocket
┌─────────────────────▼───────────────────────┐
│  VaisX (vais-web)                           │
│  Compile-time reactive frontend framework    │
│  runtime 61/77 · unit 390/390 · full-build 24/24 │
└─────────────────────┬───────────────────────┘
                      │ HTTP / WebSocket
┌─────────────────────▼───────────────────────┐
│  vais-server                                │
│  Express/Axum-style backend API framework   │
│  middleware pipeline · runtime smoke 20/20  │
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
| [VaisX](./vais-web/README.md) | Frontend framework workbench | Runtime 61/77, unit 390/390, full-build 24/24 |
| [VaisDB](./vaisdb/README.md) | Hybrid database workbench | Package 261/261, runtime 34/34 |
| [vais-server](./vais-server/README.md) | Backend framework workbench | Runtime smoke 20/20 |

## Full-stack example

```vais
# === Frontend (VaisX) ===
# app/page.vaisx
# <script>
todos := $state([])

A fn load() -> Vec<Todo> {
    fetch("/api/todos").json()
}
# </script>
# <template>
#   @each todos as todo {
#     <li>{todo.text}</li>
#   }
# </template>

# === Backend (vais-server) ===
use core/app
use db/query

fn handle_todos(ctx: Context) -> Response {
    sql := QueryBuilder.new()
        .select("todos")
        .order_by("id", SortDirection.Asc)
        .build()
    ctx.json(200, db.execute(sql))
}

fn main() -> i64 {
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
