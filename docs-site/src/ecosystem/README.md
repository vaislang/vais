# Vais 생태계

Vais 언어를 기반으로 구축된 풀스택 생태계입니다.

## 아키텍처

```
┌─────────────────────────────────────────────┐
│                  클라이언트                    │
└─────────────────────┬───────────────────────┘
                      │ HTTP / WebSocket
┌─────────────────────▼───────────────────────┐
│  VaisX (vais-web)                           │
│  컴파일 타임 반응성 프론트엔드 프레임워크         │
│  < 3KB 런타임 · SSR/SSG · 파일 기반 라우팅      │
└─────────────────────┬───────────────────────┘
                      │ HTTP / WebSocket
┌─────────────────────▼───────────────────────┐
│  vais-server                                │
│  Express/Axum 스타일 백엔드 API 프레임워크       │
│  미들웨어 파이프라인 · REST/GraphQL/gRPC        │
└─────────────────────┬───────────────────────┘
                      │ Native Query API
┌─────────────────────▼───────────────────────┐
│  VaisDB                                     │
│  RAG-native 하이브리드 데이터베이스              │
│  Vector + Graph + SQL + Full-text            │
└─────────────────────────────────────────────┘
```

## 패키지 요약

| 패키지 | 설명 | 주요 특징 |
|--------|------|----------|
| [VaisX](./vais-web/README.md) | 프론트엔드 프레임워크 | 컴파일 타임 반응성, < 3KB, SSR/SSG |
| [VaisDB](./vaisdb/README.md) | 하이브리드 데이터베이스 | 4엔진 통합, ACID, RAG-native |
| [vais-server](./vais-server/README.md) | 백엔드 프레임워크 | 미들웨어, 멀티 프로토콜, vaisdb 통합 |

## 풀스택 예제

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

## 시작하기

- [VaisX 빠른 시작](./vais-web/getting-started.md)
- [VaisDB 빠른 시작](./vaisdb/getting-started.md)
- [vais-server 빠른 시작](./vais-server/getting-started.md)

## 소스 코드

모든 생태계 패키지는 [vaislang/vais-lang](https://github.com/vaislang/vais-lang) 모노레포에서 관리됩니다.
