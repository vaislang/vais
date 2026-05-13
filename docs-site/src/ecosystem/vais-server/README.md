# vais-server

vais-server는 **Vais 언어**로 작성된 Express/Axum 스타일의 백엔드 API 프레임워크입니다. FFI 없이 순수 Vais 코드만으로 구현되어 있으며, VAIS 풀스택 생태계의 HTTP 계층을 담당합니다.

```
vais-web  (프론트엔드 + SSR)
    ↕  HTTP / WebSocket
vais-server  (백엔드 API 프레임워크)   ← 이 패키지
    ↕  네이티브 쿼리 API
vaisdb  (벡터 + 그래프 + 관계형 + 전문검색 데이터베이스)
```

---

## 특징

### Express/Axum 스타일 API

라우트 등록 방식은 Express.js를 따릅니다. 핸들러는 이름 문자열로 등록되며 런타임이 심볼릭 디스패치로 실제 함수를 연결합니다.

```vais
app.get("/users",      "handle_list_users")
app.post("/users",     "handle_create_user")
app.put("/users/:id",  "handle_update_user")
app.delete("/users/:id", "handle_delete_user")
```

라우트 그룹은 Axum의 `Router::nest`에 대응하는 `app.group("/prefix")` API로 구성합니다.

```vais
api := mut app.group("/api/v1")
api.get("/posts",     "handle_list_posts")
api.post("/posts",    "handle_create_post")
```

### 최소 코어 — App + Router + Middleware

세 가지 기본 요소로 모든 동작을 조합합니다.

| 구성 요소 | 역할 |
|-----------|------|
| `App` | 라우트와 미들웨어를 등록하고 서버를 시작 |
| `Router` | RadixTree 기반 O(log n) URL 매칭 |
| `Pipeline` | before/after 대칭 미들웨어 파이프라인 |

### 내장 인증 (Built-in Auth)

별도 라이브러리 없이 바로 사용할 수 있는 인증 모듈을 제공합니다.

- **JWT** — HS256 서명, TokenPair(access + refresh), 클레임 검증
- **OAuth 2.0** — 인증 코드 플로우, CSRF state 관리
- **Session** — 서버 사이드 세션 스토어 (TTL 지원)
- **Password** — bcrypt 스타일 해시 및 검증

### 멀티 프로토콜

단일 서버 인스턴스에서 여러 프로토콜을 동시에 처리합니다.

- **REST** — HTTP/1.1 기반 CRUD API, 페이지네이션 헬퍼
- **WebSocket** — RFC 6455, Room 기반 브로드캐스트
- **GraphQL** — 스키마 인트로스펙션, 리졸버 디스패치
- **gRPC** — 서비스 디스크립터, 프레이밍
- **OpenAPI** — 3.0 문서 자동 생성

### vaisdb 네이티브 통합

ORM 변환 계층 없이 `QueryBuilder`가 vaisdb 와이어 프로토콜로 직접 쿼리를 전송합니다. SQL, 벡터 검색, 그래프 탐색, 전문 검색을 하나의 유창한 API로 처리합니다.

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

### Pure Vais — FFI 없음

프레임워크 자체는 FFI 호출을 전혀 사용하지 않습니다. 외부 런타임 함수(`current_time_ms`, `str_len` 등)는 `X F` 선언으로 `vaisc` 링커가 해석합니다. 모든 의존성은 Vais 표준 라이브러리(`std/`) 에서 가져옵니다.

| 임포트 | 사용처 |
|--------|--------|
| `std/async_http` | HTTP/1.1 파싱 |
| `std/http_server` | TCP 연결 수락 루프 |
| `std/websocket` | RFC 6455 프레이밍 |
| `std/vec` | 동적 배열 |
| `std/option` | Optional 값 |

---

## 프로젝트 구조

```
vais-server/
├── src/
│   ├── main.vais          # 진입점
│   ├── core/              # App, Config, Context, Error
│   ├── http/              # HttpMethod, HttpStatus, Request, Response
│   ├── router/            # RadixTree, Router, RouteGroup
│   ├── middleware/        # Pipeline, CORS, Logger, RateLimit, ...
│   ├── auth/              # JWT, OAuth2, Session, Guard, Password
│   ├── ws/                # WebSocket 메시지, 핸들러, Room
│   ├── db/                # DbConnection, Pool, QueryBuilder, Migrator
│   ├── api/               # REST, GraphQL, gRPC, OpenAPI
│   └── util/              # JSON, Validation, Env
├── tests/                 # 모듈별 단위 테스트 + 통합 테스트
└── examples/              # hello.vais, rest_api.vais, chat.vais, fullstack.vais
```

---

## 빌드 및 실행

```sh
# 빌드
vaisc build src/main.vais -o vais-server

# 실행
./vais-server

# 테스트 전체 실행
vaisc test tests/

# 특정 테스트 파일 실행
vaisc test tests/router/test_router.vais
```

---

## 다음 단계

- [빠른 시작](./getting-started.md) — Hello World 서버를 5분 안에 실행
- [라우팅 가이드](./routing.md) — RadixTree 라우터, 라우트 그룹, 미들웨어 파이프라인
- [데이터베이스 통합](./database.md) — vaisdb 연결, ConnectionPool, QueryBuilder, 마이그레이션
