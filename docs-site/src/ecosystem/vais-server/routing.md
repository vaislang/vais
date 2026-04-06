# 라우팅 가이드

vais-server의 라우팅 시스템은 세 가지 핵심 구성 요소로 이루어집니다.

- **RadixTree 라우터** — O(log n) URL 매칭, 경로 파라미터 추출
- **RouteGroup** — 프리픽스 스코프 서브 라우터
- **Pipeline** — before/after 대칭 미들웨어 파이프라인

---

## RadixTree 라우터

내부적으로 각 HTTP 메서드마다 독립적인 RadixTree를 유지합니다. 요청이 들어오면 해당 메서드의 트리에서 경로를 매칭하고 핸들러 ID와 경로 파라미터를 반환합니다.

### 라우트 등록

`App`은 HTTP 메서드별 편의 메서드를 제공합니다.

```vais
app.get("/users",           "handle_list_users")
app.post("/users",          "handle_create_user")
app.get("/users/:id",       "handle_get_user")
app.put("/users/:id",       "handle_update_user")
app.patch("/users/:id",     "handle_patch_user")
app.delete("/users/:id",    "handle_delete_user")
app.ws("/ws/chat",          "handle_ws_chat")
```

두 번째 인자는 핸들러 함수의 **이름 문자열**입니다. 런타임이 이 이름을 기반으로 실제 함수를 디스패치합니다(심볼릭 디스패치).

### 경로 파라미터

`:param` 형식으로 동적 세그먼트를 선언합니다.

```vais
app.get("/articles/:slug/comments/:comment_id", "handle_get_comment")
```

핸들러 내에서 `ctx.path_params`로 파라미터 값을 읽습니다.

```vais
F handle_get_user(ctx: Context) -> Response {
    # ctx.path_params — "id=<value>" 형식 문자열
    id := ctx.path_params

    pairs := Vec.new()
    pairs.push("id")
    pairs.push(id)
    pairs.push("name")
    pairs.push("Alice")
    ctx.json(200, json_encode(pairs))
}
```

### 매칭 결과

라우터는 세 가지 상태를 반환합니다.

| 상태 | 의미 |
|------|------|
| `RouteMatchStatus.Found` | 경로와 메서드 모두 매칭됨 |
| `RouteMatchStatus.NotFound` | 어떤 메서드로도 매칭되지 않음 → 404 |
| `RouteMatchStatus.MethodNotAllowed` | 경로는 매칭되지만 메서드가 다름 → 405 |

---

## 라우트 그룹

`app.group("/prefix")`는 지정한 프리픽스가 붙은 서브 라우터를 반환합니다. 관련 라우트를 논리적으로 묶고 싶을 때 사용합니다.

```vais
F main() -> i64 {
    config := ServerConfig.default()
    app    := mut App.new(config)

    app.use("logger")
    app.use("cors")

    # /api/v1 그룹
    v1 := mut app.group("/api/v1")

    v1.get("/users",        "handle_list_users")
    v1.get("/users/:id",    "handle_get_user")
    v1.post("/users",       "handle_create_user")
    v1.put("/users/:id",    "handle_update_user")
    v1.delete("/users/:id", "handle_delete_user")

    # 그룹 라우트를 메인 앱으로 병합
    I i = 0; i < v1.route_count(); i = i + 1 {
        r := v1.routes.get(i)
        app._add_route(r.method, r.path, r.handler_id)
    }

    M app.listen(":8080") {
        Ok(_) => {},
        Err(e) => { println("오류: {e.message}") R 1 },
    }
    0
}
```

그룹을 중첩할 수도 있습니다.

```vais
admin := mut app.group("/admin")
users := mut admin.group("/users")   # 실제 프리픽스: /admin/users
```

> **참고**: `app.group()`은 새로운 `App` 인스턴스를 반환합니다. 등록한 라우트를 메인 앱에서 사용하려면 `_add_route`로 명시적으로 병합해야 합니다.

---

## 핸들러

모든 핸들러는 동일한 시그니처를 따릅니다.

```vais
F <handler_name>(ctx: Context) -> Response {
    # ...
}
```

### Context 필드

| 필드 | 타입 | 내용 |
|------|------|------|
| `ctx.method` | `str` | HTTP 메서드 (`"GET"`, `"POST"` 등) |
| `ctx.path` | `str` | 요청 경로 (`"/users/42"`) |
| `ctx.path_params` | `str` | 경로 파라미터 (`"id=42"`) |
| `ctx.query_params` | `str` | 쿼리 문자열 파라미터 |
| `ctx.body` | `str` | 요청 바디 |
| `ctx.request_id` | `str` | 요청 고유 ID |
| `ctx.state` | `str` | 미들웨어가 전달한 상태 |

### 응답 빌더

```vais
# JSON 응답
F handle_list(ctx: Context) -> Response {
    ctx.json(200, "[{\"id\":1,\"name\":\"Alice\"}]")
}

# 에러 응답
F handle_not_found(ctx: Context) -> Response {
    pairs := Vec.new()
    pairs.push("error")
    pairs.push("리소스를 찾을 수 없습니다.")
    ctx.json(404, json_encode(pairs))
}

# 생성 성공 (201 Created)
F handle_create(ctx: Context) -> Response {
    pairs := Vec.new()
    pairs.push("id")
    pairs.push("99")
    pairs.push("status")
    pairs.push("created")
    ctx.json(201, json_encode(pairs))
}

# 내용 없음 (204 No Content)
F handle_delete(ctx: Context) -> Response {
    ctx.status(204)
}

# 리다이렉트
F handle_old_path(ctx: Context) -> Response {
    ctx.redirect("/new/path")
}
```

### 응답 헤더 설정

```vais
F handle_with_header(ctx: Context) -> Response {
    ctx2 := ctx.set_header("X-Request-Id", "abc-123")
    ctx2.json(200, "{\"ok\":true}")
}
```

`set_header`는 새 `Context`를 반환합니다. 여러 헤더를 설정할 때는 체이닝합니다.

```vais
ctx2 := ctx.set_header("X-Foo", "bar").set_header("X-Baz", "qux")
```

---

## 미들웨어 파이프라인

### 파이프라인 모델

파이프라인은 Koa.js의 양파 모델을 따릅니다.

```
요청 →  before[0]  →  before[1]  →  before[2]  →  핸들러
                                                      ↓
응답 ←  after[0]   ←  after[1]   ←  after[2]   ←  handler_response
```

- `before` 훅은 등록 순서대로 실행됩니다.
- `after` 훅은 등록 역순으로 실행됩니다.
- 어떤 `before` 훅이 `BeforeResult.Respond`를 반환하면 이후 before 훅과 핸들러를 건너뜁니다. 단, 이미 실행된 미들웨어의 after 훅은 정상적으로 실행됩니다.

### 내장 미들웨어 등록

```vais
app.use("logger")      # 요청 로깅
app.use("cors")        # CORS 헤더
app.use("rate_limit")  # 속도 제한
app.use("compress")    # 응답 압축
app.use("recovery")    # 패닉 복구
```

### 미들웨어 직접 구현

커스텀 미들웨어는 `before`와 `after` 두 함수를 구현하는 구조체로 만듭니다.

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

    # before: Authorization 헤더 검증
    F before(self, ctx: Context) -> BeforeResult {
        token := ctx.query_params   # 실제로는 헤더에서 추출
        I token == "" {
            pairs := Vec.new()
            pairs.push("error")
            pairs.push("인증 토큰이 필요합니다.")
            err_response := ctx.json(401, json_encode(pairs))
            R BeforeResult.respond(err_response)
        }
        BeforeResult.next()
    }

    # after: 응답에 보안 헤더 추가
    F after(self, ctx: Context, response: Response) -> Response {
        # 실제 구현에서는 response에 헤더를 추가하여 반환
        response
    }
}
```

`BeforeResult.next()` — 다음 미들웨어/핸들러로 진행합니다.
`BeforeResult.respond(response)` — 파이프라인을 단락(short-circuit)시킵니다.

### Pipeline 내부 구조

```vais
S Pipeline {
    entries: Vec<PipelineEntry>,
    count:   i64,
}
```

- `pipeline.run_before(ctx)` → `PipelineBeforeOutput` 반환
  - `short_circuit: true`이면 `response` 필드에 조기 응답이 담겨 있습니다.
  - `short_circuit: false`이면 핸들러를 호출해야 합니다.
- `pipeline.run_after(ctx, handler_response)` → 최종 `Response` 반환

Vais는 외부 구조체를 변이하는 루프를 허용하지 않으므로, 파이프라인 내부는 재귀 헬퍼 함수(`pipeline_run_before`, `pipeline_run_after`)로 구현되어 있습니다.

---

## 전체 라우팅 예제 — CRUD REST API

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

    println("REST API 서버 시작: :{PORT} (라우트 수: {app.route_count()})")

    M app.listen(":{PORT}") {
        Ok(_) => {},
        Err(e) => { println("오류: {e.message}") R 1 },
    }
    0
}
```

---

## 다음 단계

- [데이터베이스 통합](./database.md) — QueryBuilder로 실제 데이터를 조회하고 핸들러에서 반환하는 방법
