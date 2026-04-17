# AI 바이브코딩 5분 가이드

AI(Claude, Cursor, Copilot 등)로 Vais 코드를 생성하려는 개발자를 위한 최단 경로입니다.
5분 안에 "언어 + 생태계 전체 그림 + 첫 실행"까지 도달하는 것이 목표입니다.

---

## 1. 설치 한 줄

```bash
curl -fsSL https://vaislang.dev/install.sh | sh
vaisc --version    # v0.1.0
```

Windows PowerShell: `iwr https://vaislang.dev/install.ps1 | iex`
Homebrew: `brew install vaislang/vais/vais`

---

## 2. AI에게 Vais를 "알려주기"

AI가 Vais 문법과 생태계를 모르면 잘못된 코드를 생성합니다.
Vais는 **`llms.txt` 표준**을 따르는 두 개의 컨텍스트 파일을 제공합니다.

| 파일 | 용도 | 크기 |
|------|------|------|
| [llms.txt](https://vaislang.dev/llms.txt) | 큐레이션 인덱스 (링크 + 핵심 규칙) | 작음 |
| [llms-full.txt](https://vaislang.dev/llms-full.txt) | 전체 문서 concat — 드래그로 drop-in | 큼 |

**사용법**: `llms-full.txt`를 Cursor의 프로젝트 컨텍스트나 Claude 대화창에 첨부하면
AI가 Vais 문법/표준 라이브러리/VaisX/VaisDB/vais-server 전체를 즉시 이해합니다.

---

## 3. 언어 핵심 5분

### 단일 문자 키워드 (AI 토큰 절감 목적)

| 키 | 의미 | 키 | 의미 |
|----|------|----|------|
| `F` | function | `W` | trait |
| `S` | struct | `X` | impl |
| `E` | enum / else | `P` | pub |
| `I` | if | `D` | defer |
| `L` | loop | `A` | async |
| `M` | match | `Y` | await |
| `R` | return | `N` | extern |
| `T` | type alias | `G` | global |
| `U` | use (import) | `O` | union |

### 연산자

- `:=` 바인딩, `:= mut` 가변 바인딩
- `?` try / ternary, `!` unwrap
- `|>` pipe, `..` range, `@` self-recursion
- `#` 주석, `{expr}` 문자열 보간

### 타입 변환 — **엄격 규칙** (AI가 가장 자주 틀림)

```vais
# ❌ 틀림 — 암시적 coercion은 금지
x: i64 := 1.5      # error: float → int 암시 금지

# ✅ 맞음 — as 명시적 변환
x: i64 := 1.5 as i64
flag: bool := 1 as bool
f: f64 := (42 as f64)
```

**허용되는 암시적 변환**: 정수 widening (`i8→i64`), float 리터럴 추론 (`f32↔f64`).
**금지**: `bool↔i64`, `int↔float`, `str↔i64`, 정수 narrowing (`i64→i32`).

---

## 4. Hello World + 컴파일

```vais
# hello.vais
F main() -> i64 {
    puts("Hello, Vais!")
    0
}
```

```bash
vaisc build hello.vais    # → ./hello
./hello                    # Hello, Vais!
vaisc run hello.vais      # 빌드 + 실행
```

---

## 5. 실전 예제 3개

### 예제 1: CLI 도구 — FizzBuzz

```vais
F fizzbuzz(n: i64) -> i64 {
    LF i:1..n+1 {
        I i % 15 == 0      { puts("FizzBuzz") }
        EL I i % 3 == 0    { puts("Fizz") }
        EL I i % 5 == 0    { puts("Buzz") }
        EL                 { puts("{i}") }
    }
    0
}

F main() -> i64 {
    puts("=== FizzBuzz 1..20 ===")
    fizzbuzz(20)
}
```

핵심: `LF` loop, `I/EL` if-else-if 체인, 문자열 보간 `{i}`.

### 예제 2: HTTP 서버 (vais-server)

```vais
U core/app::{App, ServerConfig}
U core/context::{Context, Response}

F handle_hello(ctx: Context) -> Response {
    ctx.text(200, "Hello from vais-server!")
}

F handle_user(ctx: Context) -> Response {
    id := ctx.param("id")
    ctx.json(200, "{\"id\": \"{id}\", \"name\": \"Alice\"}")
}

F main() -> i64 {
    app := mut App.new(ServerConfig.default())
    app.get("/",          "handle_hello")
    app.get("/users/:id", "handle_user")
    app.listen(":8080")
    0
}
```

자세히: [vais-server 가이드](../ecosystem/vais-server/README.md)

### 예제 3: VaisDB — 하이브리드 쿼리

```vais
U vaisdb::{Database}

F main() -> i64 {
    db := Database.open("myapp.vaisdb")?

    # SQL + 벡터 검색을 한 트랜잭션에서
    db.execute("CREATE TABLE documents (id i64, title str, embedding vector(768))")?
    db.execute("INSERT INTO documents VALUES (1, 'Vais intro', @v1)")?

    results := db.query("
        SELECT id, title
        FROM documents
        VECTOR_SEARCH(embedding, @query, top_k=5)
        WHERE title LIKE '%Vais%'
    ")?

    LF row in results {
        puts("{row.id}: {row.title}")
    }
    0
}
```

자세히: [VaisDB 쿼리 가이드](../ecosystem/vaisdb/queries.md)

---

## 6. 풀스택 한 장 그림

```
┌──────────── VaisX (프론트엔드) ────────────┐
│ .vaisx 컴포넌트, < 3KB 런타임, SSR/SSG      │
│ $state / $derived / $effect                │
└────────────────────┬───────────────────────┘
                     │ HTTP / WebSocket
┌────────────────────▼───────────────────────┐
│ vais-server (백엔드)                        │
│ Express/Axum 스타일 라우팅 + 미들웨어        │
│ 내장 JWT 인증                               │
└────────────────────┬───────────────────────┘
                     │ 네이티브 쿼리 API
┌────────────────────▼───────────────────────┐
│ VaisDB (데이터베이스)                        │
│ Vector + Graph + SQL + Full-text            │
│ 단일 `.vaisdb` 파일, 단일 트랜잭션            │
└────────────────────────────────────────────┘
```

---

## 7. AI에게 효과적으로 시키기 — 프롬프트 팁

- **컨텍스트 제공**: 대화 시작 시 `llms.txt` 링크를 알려주거나 `llms-full.txt`를 첨부.
- **타입 변환 주의 요청**: "Vais는 엄격한 타입 변환을 요구한다. 암시적 coercion 금지, `as`로 명시." 한 줄 추가.
- **제거된 키워드 주의**: `weak`, `clone`, `consume`은 제거됨. AI가 이들을 사용하려 하면 거부.
- **실험 기능 금지**: `lazy`, `force`, HKT, `impl Trait` (dyn이 아닌 것)은 아직 제한적. 프로덕션 코드에서 제외.
- **생태계 import**: VaisDB는 `U vaisdb::{Database}`, vais-server는 `U core/app::{App}` 형식.

---

## 8. 다음 단계

- **언어 깊게**: [언어 명세](../language/language-spec.md)
- **실전 튜토리얼**: [CLI 도구](../tutorials/cli-tool.md), [HTTP 서버](../tutorials/http-server.md), [WebSocket 채팅](../tutorials/websocket-chat.md)
- **생태계 상세**: [생태계 개요](../ecosystem/README.md)
- **플레이그라운드**: [play.vaislang.dev](https://play.vaislang.dev) (40+ 예제)
- **GitHub**: [vaislang/vais](https://github.com/vaislang/vais)
