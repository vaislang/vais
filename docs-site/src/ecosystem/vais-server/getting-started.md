# 빠른 시작

이 가이드에서는 vais-server로 첫 번째 HTTP 서버를 실행하고, 기본 라우팅과 JSON 응답까지 살펴봅니다.

---

## 전제 조건

- `vaisc` 컴파일러가 설치되어 있어야 합니다.
- vais-server 패키지가 `packages/vais-server/` 에 있어야 합니다.

---

## Hello World 서버

### 1. 소스 파일 작성

`src/main.vais` 파일을 만들고 아래 내용을 입력합니다.

```vais
U core/app
U core/config
U core/context

C PORT: u16 = 8080

# GET / 핸들러 — plain text 응답
F handle_hello(ctx: Context) -> Response {
    ctx.text(200, "Hello, World!")
}

F main() -> i64 {
    config := ServerConfig.default()
    app    := mut App.new(config)

    app.get("/", "handle_hello")

    addr := ":{PORT}"
    println("서버 시작: {addr}")

    M app.listen(addr) {
        Ok(_) => {
            println("서버가 정상 종료되었습니다.")
        },
        Err(e) => {
            println("서버 시작 실패: {e.message}")
            R 1
        },
    }
    0
}
```

### 2. 빌드 및 실행

```sh
vaisc build src/main.vais -o hello-server
./hello-server
```

```
서버 시작: :8080
```

### 3. 요청 확인

```sh
curl http://localhost:8080/
# Hello, World!
```

---

## 코드 구조 이해

### 임포트

```vais
U core/app      # App 구조체 — 라우트/미들웨어 등록, listen()
U core/config   # ServerConfig — 서버 설정
U core/context  # Context — 요청 컨텍스트, Response 빌더
```

`U` 키워드는 모듈 임포트입니다. 경로는 `src/` 기준 상대 경로입니다.

### 핸들러 시그니처

```vais
F handle_hello(ctx: Context) -> Response {
    ctx.text(200, "Hello, World!")
}
```

모든 핸들러는 `F(ctx: Context) -> Response` 형태입니다. `ctx`에는 요청 메서드, 경로, 경로 파라미터, 쿼리 파라미터, 바디가 담겨 있습니다.

### 라우트 등록 — 심볼릭 디스패치

```vais
app.get("/", "handle_hello")
```

두 번째 인자는 함수 이름 **문자열**입니다. Vais는 현재 버전에서 일급 함수 포인터를 지원하지 않으므로 런타임이 이름을 기반으로 실제 함수를 디스패치합니다.

### Result 처리

```vais
M app.listen(addr) {
    Ok(_)  => { println("정상 종료") },
    Err(e) => { println("오류: {e.message}") R 1 },
}
```

`M`은 match 표현식입니다. `app.listen()`은 `Result<(), VaisServerError>`를 반환하므로 반드시 처리해야 합니다.

---

## JSON API 서버

plain text 응답 외에 JSON 응답을 반환하는 예제입니다.

```vais
U core/app
U core/config
U core/context
U src/util/json

C PORT: u16 = 8080

# GET /ping — JSON 헬스 체크
F handle_ping(ctx: Context) -> Response {
    pairs := Vec.new()
    pairs.push("status")
    pairs.push("ok")
    pairs.push("version")
    pairs.push("1.0.0")
    ctx.json(200, json_encode(pairs))
}

# GET /hello/:name — 경로 파라미터 사용
F handle_greet(ctx: Context) -> Response {
    name := ctx.path_params   # "name=<value>" 형식
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

    println("JSON API 서버 시작: :{PORT}")
    M app.listen(":{PORT}") {
        Ok(_) => {},
        Err(e) => {
            println("오류: {e.message}")
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

### Context 응답 빌더

| 메서드 | Content-Type | 용도 |
|--------|-------------|------|
| `ctx.text(status, body)` | `text/plain; charset=utf-8` | 일반 텍스트 |
| `ctx.json(status, body)` | `application/json; charset=utf-8` | JSON |
| `ctx.html(status, body)` | `text/html; charset=utf-8` | HTML |
| `ctx.redirect(url)` | — | 302 리다이렉트 |
| `ctx.status(code)` | — | 바디 없는 상태 코드 |

---

## 미들웨어 추가

```vais
app.use("logger")   # 요청/응답 로그
app.use("cors")     # CORS 헤더 자동 추가
```

`app.use()`에 전달하는 값은 미들웨어 이름 문자열입니다. 내장 미들웨어로는 `logger`, `cors`, `rate_limit`, `compress`, `recovery`가 있습니다.

미들웨어는 등록 순서대로 before 훅이 실행되고, 역순으로 after 훅이 실행됩니다.

```vais
F main() -> i64 {
    config := ServerConfig.default()
    app    := mut App.new(config)

    # 전역 미들웨어 등록 (라우트보다 먼저)
    app.use("logger")
    app.use("cors")

    app.get("/", "handle_hello")

    M app.listen(":{PORT}") {
        Ok(_) => {},
        Err(e) => { println("오류: {e.message}") R 1 },
    }
    0
}
```

---

## ServerConfig

`ServerConfig.default()`는 기본 설정을 사용합니다. 상세 설정이 필요하다면 `ServerConfig`를 직접 구성하세요.

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

## 다음 단계

- [라우팅 가이드](./routing.md) — RadixTree 라우터, 라우트 그룹, 미들웨어 파이프라인 상세 설명
- [데이터베이스 통합](./database.md) — vaisdb 연결, QueryBuilder, 마이그레이션
