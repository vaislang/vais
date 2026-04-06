# 데이터베이스 통합

vais-server는 vaisdb와 네이티브로 통합됩니다. ORM 변환 계층 없이 `QueryBuilder`가 vaisdb 와이어 프로토콜로 직접 쿼리를 전송합니다.

```
핸들러 (vais-server)
    ↓  QueryBuilder.build() → SQL / VECTOR_SEARCH / GRAPH_TRAVERSE / FULLTEXT_MATCH
    ↓  DbConnection.execute(sql)
vaisdb (Vector + Graph + SQL + Full-text)
```

---

## DbConnection — 데이터베이스 연결

### 임베디드 모드 (파일 기반)

```vais
U db/connection

config := DbConfig.embedded("./data/myapp.vaisdb")
M DbConnection.connect(config) {
    Ok(conn) => {
        println("연결 성공: {conn.to_string()}")
    },
    Err(e) => {
        println("연결 실패: {e.message}")
        R 1
    },
}
```

### TCP 모드 (원격 서버)

```vais
config := DbConfig.tcp("127.0.0.1", 7300)
M DbConnection.connect(config) {
    Ok(conn) => { /* ... */ },
    Err(e)   => { /* ... */ },
}
```

### DbConfig 필드

| 팩터리 메서드 | 모드 | 필수 파라미터 |
|-------------|------|-------------|
| `DbConfig.embedded(path)` | `DbMode.Embedded` | `db_path` |
| `DbConfig.tcp(host, port)` | `DbMode.Tcp` | `host`, `port` |

기본 타임아웃은 5000ms입니다.

### 쿼리 실행

```vais
sql := "SELECT id, name FROM users WHERE id = 1"
M conn.execute(sql) {
    Ok(result) => {
        println("결과 행 수: {result.row_count()}")
        I i = 0; i < result.row_count(); i = i + 1 {
            row := result.rows.get(i)
            println("  {row.get()}")
        }
    },
    Err(e) => {
        println("쿼리 실패: {e.message}")
    },
}
```

`QueryResult` 구조체:

| 필드 | 타입 | 의미 |
|------|------|------|
| `rows` | `Vec<Row>` | 결과 행 목록 |
| `affected_rows` | `i64` | INSERT/UPDATE/DELETE 영향 행 수 |
| `columns` | `Vec<str>` | 컬럼명 목록 |

---

## ConnectionPool — 연결 풀

프로덕션 환경에서는 매 요청마다 새 연결을 여는 대신 `ConnectionPool`을 사용합니다.

### 풀 생성

```vais
U db/connection
U db/pool

db_config   := DbConfig.embedded("./data/myapp.vaisdb")
pool_config := PoolConfig.default()   # min=2, max=10, idle_timeout=30s

M ConnectionPool.new(db_config, pool_config) {
    Ok(mut pool) => {
        # 풀 사용
        stats := pool.stats()
        println("{stats.to_string()}")
    },
    Err(e) => {
        println("풀 생성 실패: {e.message}")
        R 1
    },
}
```

### 연결 획득 및 반환

```vais
M pool.acquire() {
    Ok(conn) => {
        # 쿼리 수행
        M conn.execute("SELECT 1") {
            Ok(_)  => { println("헬스 체크 성공") },
            Err(e) => { println("오류: {e.message}") },
        }
        # 반드시 연결을 반환해야 합니다
        pool.release(conn)
    },
    Err(e) => {
        println("풀 소진: {e.message}")
    },
}
```

### PoolConfig 파라미터

```vais
pool_config := PoolConfig.new(
    2,      # min_connections — 시작 시 미리 열어 두는 연결 수
    20,     # max_connections — 최대 연결 수
    60000,  # idle_timeout_ms — 유휴 연결 타임아웃 (ms)
)
```

### PoolStats

```vais
stats := pool.stats()
# PoolStats { active=3, idle=7, total=10 }
println(stats.to_string())
```

| 필드 | 의미 |
|------|------|
| `active` | 현재 사용 중인 연결 수 |
| `idle` | 대기 중인 연결 수 |
| `total` | active + idle |

### 헬스 체크

```vais
pool.health_check()   # 유휴 연결에 SELECT 1 핑을 보내고 죽은 연결을 교체
```

---

## QueryBuilder — 쿼리 빌더

`QueryBuilder`는 SQL, 벡터 검색, 그래프 탐색, 전문 검색을 하나의 유창한(fluent) API로 지원합니다.

### SELECT

```vais
U db/query

sql := QueryBuilder.new()
    .select("users")
    .column("id")
    .column("name")
    .column("email")
    .where_clause("active = 1")
    .order_by("created_at", SortDirection.Desc)
    .limit(50)
    .build()
# SELECT id, name, email FROM users WHERE active = 1 ORDER BY created_at DESC LIMIT 50
```

컬럼을 지정하지 않으면 `*`로 처리됩니다.

```vais
sql := QueryBuilder.new()
    .select("products")
    .where_clause("price < 10000")
    .build()
# SELECT * FROM products WHERE price < 10000
```

### WHERE 조건 다중 결합

`.where_clause()`를 여러 번 호출하면 AND로 결합됩니다.

```vais
sql := QueryBuilder.new()
    .select("orders")
    .column("id")
    .column("total")
    .where_clause("status = 'shipped'")
    .where_clause("total > 50000")
    .build()
# SELECT id, total FROM orders WHERE status = 'shipped' AND total > 50000
```

### JOIN

```vais
sql := QueryBuilder.new()
    .select("posts")
    .column("posts.id")
    .column("posts.title")
    .column("users.name")
    .join("users", "posts.user_id = users.id")
    .where_clause("posts.published = 1")
    .build()
# SELECT posts.id, posts.title, users.name FROM posts
#   JOIN users ON posts.user_id = users.id WHERE posts.published = 1
```

### INSERT

```vais
fields := Vec.new()
fields.push("name")
fields.push("email")
fields.push("created_at")

sql := QueryBuilder.new()
    .insert("users", fields)
    .build()
# INSERT INTO users (name, email, created_at) VALUES (?, ?, ?)
```

### UPDATE

```vais
fields := Vec.new()
fields.push("name")
fields.push("email")

sql := QueryBuilder.new()
    .update("users", fields)
    .where_clause("id = 42")
    .build()
# UPDATE users SET name = ?, email = ? WHERE id = 42
```

### DELETE

```vais
sql := QueryBuilder.new()
    .delete("users")
    .where_clause("id = 42")
    .build()
# DELETE FROM users WHERE id = 42
```

### 트랜잭션

```vais
begin_sql  := QueryBuilder.new().begin_transaction().build()  # "BEGIN"
commit_sql := QueryBuilder.new().commit().build()             # "COMMIT"
rb_sql     := QueryBuilder.new().rollback().build()           # "ROLLBACK"

M conn.execute(begin_sql) {
    Ok(_)  => {},
    Err(e) => { R Err(e) },
}
# ... DML 쿼리 실행 ...
M conn.execute(commit_sql) {
    Ok(_)  => { println("트랜잭션 커밋 성공") },
    Err(e) => {
        conn.execute(rb_sql)
        println("커밋 실패, 롤백: {e.message}")
    },
}
```

---

## vaisdb 하이브리드 쿼리

### 벡터 검색 (VECTOR_SEARCH)

```vais
# 임베딩 벡터로 유사 문서 top-10 검색
query_vec := "[0.12, 0.87, 0.34, 0.56]"

sql := QueryBuilder.new()
    .select("documents")
    .column("id")
    .column("title")
    .column("content")
    .vector_search("embeddings", query_vec, 10)
    .build()
# SELECT id, title, content FROM documents
#   WHERE VECTOR_SEARCH(embeddings, [0.12, 0.87, 0.34, 0.56], 10)
```

### 그래프 탐색 (GRAPH_TRAVERSE)

```vais
# 노드 "user-42"에서 outbound 방향으로 깊이 3까지 탐색
sql := QueryBuilder.new()
    .column("id")
    .column("label")
    .graph_traverse("user-42", 3, "outbound")
    .build()
# SELECT id, label FROM GRAPH_TRAVERSE('user-42', 3, 'outbound')
```

방향 옵션: `"outbound"`, `"inbound"`, `"any"`

### 전문 검색 (FULLTEXT_MATCH)

```vais
sql := QueryBuilder.new()
    .select("articles")
    .column("id")
    .column("title")
    .column("body")
    .fulltext_match("body", "vais-server 라우팅")
    .limit(20)
    .build()
# SELECT id, title, body FROM articles
#   WHERE FULLTEXT_MATCH(body, 'vais-server 라우팅') LIMIT 20
```

---

## 마이그레이션 (Migrator)

`Migrator`는 버전 기반 스키마 마이그레이션을 관리합니다. 내부적으로 `__vaisdb_migrations` 테이블로 적용 이력을 추적합니다.

### 마이그레이션 정의 및 실행

```vais
U db/connection
U db/migrate

F run_migrations(conn: DbConnection) -> Result<i64, VaisDbError> {
    migrator_result := Migrator.new(conn)
    M migrator_result {
        Err(e) => { R Err(e) },
        Ok(mut migrator) => {
            # 버전 1 — users 테이블 생성
            m1 := Migration.new(
                1,
                "create_users",
                "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT UNIQUE NOT NULL, created_at TEXT NOT NULL)",
                "DROP TABLE IF EXISTS users"
            )
            migrator.add_migration(m1)

            # 버전 2 — posts 테이블 생성
            m2 := Migration.new(
                2,
                "create_posts",
                "CREATE TABLE IF NOT EXISTS posts (id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, title TEXT NOT NULL, body TEXT, published INTEGER DEFAULT 0)",
                "DROP TABLE IF EXISTS posts"
            )
            migrator.add_migration(m2)

            # 버전 3 — posts에 인덱스 추가
            m3 := Migration.new(
                3,
                "add_posts_user_index",
                "CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id)",
                "DROP INDEX IF EXISTS idx_posts_user_id"
            )
            migrator.add_migration(m3)

            # 미적용 마이그레이션을 순서대로 실행
            migrator.run_up()
        },
    }
}
```

### 롤백

```vais
# 버전 1까지 롤백 (버전 2, 3이 있으면 역순으로 실행)
M migrator.run_down(1) {
    Ok(count) => { println("{count}개 마이그레이션 롤백 완료") },
    Err(e)    => { println("롤백 실패: {e.message}") },
}
```

### Migration 구조

```vais
m := Migration.new(
    version,   # i64 — 단조 증가 버전 번호
    name,      # str — 마이그레이션 이름 (스네이크케이스 권장)
    up_sql,    # str — 적용 SQL
    down_sql,  # str — 롤백 SQL
)
```

---

## 전체 통합 예제

아래는 서버 시작 시 DB를 연결하고 마이그레이션을 수행한 뒤, 핸들러에서 QueryBuilder로 데이터를 조회하는 전체 흐름입니다.

```vais
U core/app
U core/config
U core/context
U db/connection
U db/migrate
U db/query
U src/util/json

C PORT:    u16 = 8080
C DB_PATH: str = "./data/app.vaisdb"

F handle_get_user(ctx: Context) -> Response {
    id := ctx.path_params

    sql := QueryBuilder.new()
        .select("users")
        .column("id")
        .column("name")
        .column("email")
        .where_clause("id = " + id)
        .build()

    # 실제 구현에서는 conn.execute(sql)로 DB 조회
    println("  [db] {sql}")

    pairs := Vec.new()
    pairs.push("id")
    pairs.push(id)
    pairs.push("name")
    pairs.push("Alice")
    ctx.json(200, json_encode(pairs))
}

F main() -> i64 {
    # 1. DB 연결
    db_config := DbConfig.embedded(DB_PATH)
    db := M DbConnection.connect(db_config) {
        Err(e) => {
            println("DB 연결 실패: {e.message}")
            R 1
        },
        Ok(conn) => { conn },
    }

    # 2. 마이그레이션 실행
    M Migrator.new(db) {
        Err(e) => {
            println("Migrator 초기화 실패: {e.message}")
            R 1
        },
        Ok(mut migrator) => {
            m1 := Migration.new(
                1, "create_users",
                "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT UNIQUE NOT NULL, created_at TEXT NOT NULL)",
                "DROP TABLE IF EXISTS users"
            )
            migrator.add_migration(m1)
            M migrator.run_up() {
                Ok(count) => { println("마이그레이션 {count}개 적용") },
                Err(e)    => { println("마이그레이션 실패: {e.message}") R 1 },
            }
        },
    }

    # 3. HTTP 서버 설정
    config := ServerConfig.default()
    app    := mut App.new(config)

    app.use("logger")
    app.use("cors")
    app.get("/users/:id", "handle_get_user")

    println("서버 시작: :{PORT}")
    M app.listen(":{PORT}") {
        Ok(_) => {},
        Err(e) => { println("서버 오류: {e.message}") R 1 },
    }
    0
}
```

---

## QueryKind 참조

| `QueryKind` | 생성 메서드 | 예시 출력 |
|-------------|-----------|---------|
| `Standard` (SELECT) | `.select(table)` | `SELECT ... FROM ...` |
| `Standard` (INSERT) | `.insert(table, fields)` | `INSERT INTO ... VALUES (...)` |
| `Standard` (UPDATE) | `.update(table, fields)` | `UPDATE ... SET ...` |
| `Standard` (DELETE) | `.delete(table)` | `DELETE FROM ...` |
| `VectorSearch` | `.vector_search(col, vec, k)` | `... WHERE VECTOR_SEARCH(...)` |
| `GraphTraverse` | `.graph_traverse(start, depth, dir)` | `... FROM GRAPH_TRAVERSE(...)` |
| `FulltextMatch` | `.fulltext_match(col, query)` | `... WHERE FULLTEXT_MATCH(...)` |
| `BeginTransaction` | `.begin_transaction()` | `BEGIN` |
| `Commit` | `.commit()` | `COMMIT` |
| `Rollback` | `.rollback()` | `ROLLBACK` |

---

## 다음 단계

- [vaisdb 문서](../vaisdb/README.md) — vaisdb 데이터베이스 엔진의 전체 기능과 스키마 설계
- [라우팅 가이드](./routing.md) — 핸들러에서 QueryBuilder 결과를 JSON 응답으로 반환하는 패턴
