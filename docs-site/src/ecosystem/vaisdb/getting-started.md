# 빠른 시작

VaisDB를 설치하고 첫 번째 쿼리를 실행하는 방법을 안내합니다.

---

## 요구 사항

- Vais 컴파일러 v1.0.0 이상 (`~/.cargo/bin/vaisc`)
- 운영체제: Linux, macOS, Windows (WSL2 권장)

---

## 설치

### 소스에서 빌드

```bash
# vais-lang 모노레포 클론
git clone https://github.com/vaislang/vais-lang.git
cd vais-lang/packages/vaisdb

# 빌드
vaisc build

# 바이너리 확인
./vaisdb --version
```

### 패키지 매니저 (준비 중)

```bash
# vais 패키지 매니저 사용 (향후 지원 예정)
vpm install vaisdb
```

---

## 사용 모드

VaisDB는 두 가지 모드로 실행할 수 있습니다.

| 모드 | 설명 | 적합한 용도 |
|------|------|------------|
| 임베디드 모드 | 프로세스 내부에서 직접 DB 파일 접근 | 단일 앱, 로컬 개발 |
| TCP 서버 모드 | 독립 서버 프로세스, 다중 클라이언트 | 마이크로서비스, 공유 DB |

---

## 임베디드 모드

### 데이터베이스 열기

```vais
U vaisdb::{Database};

F main() {
    # 새 DB 생성 또는 기존 DB 열기 (.vaisdb 파일)
    db := Database::open("myapp.vaisdb")?;
    println("데이터베이스 열기 성공");
}
```

### 테이블 생성

```sql
CREATE TABLE documents (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    title   TEXT NOT NULL,
    content TEXT,
    author  TEXT,
    tags    TEXT[],
    embedding VECTOR(1536),
    created_at TIMESTAMP DEFAULT NOW()
);
```

Vais에서 실행:

```vais
U vaisdb::{Database};

F main() {
    db := Database::open("myapp.vaisdb")?;

    db.execute("
        CREATE TABLE IF NOT EXISTS documents (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            title     TEXT NOT NULL,
            content   TEXT,
            author    TEXT,
            embedding VECTOR(1536),
            created_at TIMESTAMP DEFAULT NOW()
        )
    ", [])?;

    println("테이블 생성 완료");
}
```

### 데이터 삽입

#### 일반 텍스트 삽입

```vais
U vaisdb::{Database};

F main() {
    db := Database::open("myapp.vaisdb")?;

    # 단건 삽입
    db.execute("
        INSERT INTO documents (title, content, author)
        VALUES ($1, $2, $3)
    ", ["VaisDB 소개", "VaisDB는 RAG-native 하이브리드 데이터베이스입니다.", "홍길동"])?;

    println("문서 삽입 완료");
}
```

#### 벡터와 함께 삽입

`EMBED()` 함수를 사용하면 텍스트에서 임베딩 벡터를 자동으로 생성합니다.

```vais
U vaisdb::{Database};

F main() {
    db := Database::open("myapp.vaisdb")?;

    # EMBED() 함수로 자동 임베딩 생성
    db.execute("
        INSERT INTO documents (title, content, embedding)
        VALUES ($1, $2, EMBED($2))
    ", ["벡터 검색 기초", "HNSW 알고리즘은 근사 최근접 이웃 검색에 사용됩니다."])?;

    # 사전 계산된 벡터 직접 삽입
    embedding: Vec<f32> = compute_embedding("직접 계산한 임베딩");
    db.execute("
        INSERT INTO documents (title, content, embedding)
        VALUES ($1, $2, $3)
    ", ["직접 삽입", "사전 계산 벡터 사용", embedding])?;
}
```

#### 배치 삽입

```vais
U vaisdb::{Database};

F main() {
    db := Database::open("myapp.vaisdb")?;

    # 트랜잭션으로 묶어 배치 삽입
    tx := db.begin()?;

    documents := [
        ("AI 개요", "인공지능의 기본 개념"),
        ("ML 입문", "머신러닝 알고리즘 소개"),
        ("딥러닝", "신경망 구조와 학습 방법"),
    ];

    LF (title, content) IN documents {
        tx.execute("
            INSERT INTO documents (title, content, embedding)
            VALUES ($1, $2, EMBED($2))
        ", [title, content])?;
    }

    tx.commit()?;
    println("배치 삽입 완료");
}
```

---

## TCP 서버 모드

### 서버 시작

```bash
# 기본 포트(5432)로 서버 시작
./vaisdb server --db knowledge.vaisdb

# 포트 및 옵션 지정
./vaisdb server \
  --db knowledge.vaisdb \
  --port 5432 \
  --host 0.0.0.0 \
  --max-connections 100 \
  --tls-cert /path/to/cert.pem \
  --tls-key /path/to/key.pem
```

### 클라이언트로 접속

```vais
U vaisdb::{Client};

F main() {
    # TCP 클라이언트로 원격 서버에 접속
    client := Client::connect("vaisdb://localhost:5432/knowledge")?;

    result := client.query("SELECT COUNT(*) FROM documents", [])?;
    println("문서 수: {result[0][0]}");

    client.close();
}
```

### 서버 설정 파일 (vaisdb.toml)

```toml
[server]
host = "0.0.0.0"
port = 5432
max_connections = 100

[database]
path = "knowledge.vaisdb"
page_size = 4096
buffer_pool_size = "512MB"
wal_sync_mode = "full"       # full | normal | off

[security]
auth = "password"            # password | cert | none
tls = true
tls_cert = "/path/to/cert.pem"
tls_key  = "/path/to/key.pem"

[embeddings]
model = "text-embedding-3-small"
dimensions = 1536
api_key_env = "OPENAI_API_KEY"
```

---

## 첫 번째 전체 예제

아래는 문서를 삽입하고 벡터 유사도 검색을 수행하는 완전한 예제입니다.

```vais
U vaisdb::{Database, Row};

F main() {
    # 1. 데이터베이스 열기
    db := Database::open("demo.vaisdb")?;

    # 2. 스키마 생성
    db.execute("
        CREATE TABLE IF NOT EXISTS articles (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            title     TEXT NOT NULL,
            body      TEXT NOT NULL,
            embedding VECTOR(1536)
        )
    ", [])?;

    # 3. 샘플 데이터 삽입
    samples := [
        ("Vais 언어 소개",     "Vais는 AI 최적화 시스템 프로그래밍 언어입니다."),
        ("VaisDB 아키텍처",    "VaisDB는 4개 엔진을 단일 ACID 트랜잭션으로 통합합니다."),
        ("HNSW 알고리즘",      "계층형 탐색 가능 소세계 그래프 기반 근사 최근접 이웃 알고리즘."),
        ("BM25 랭킹",         "전문 검색에서 문서 관련성을 측정하는 확률 기반 모델."),
        ("RAG 파이프라인",     "검색 증강 생성은 외부 지식을 LLM에 주입하는 기법입니다."),
    ];

    tx := db.begin()?;
    LF (title, body) IN samples {
        tx.execute("
            INSERT INTO articles (title, body, embedding)
            VALUES ($1, $2, EMBED($2))
        ", [title, body])?;
    }
    tx.commit()?;
    println("샘플 데이터 삽입 완료 ({samples.len()}건)");

    # 4. 벡터 유사도 검색
    println("\n--- 벡터 검색: '근사 이웃 탐색 알고리즘' ---");
    results := db.query("
        SELECT a.title, v.similarity
        FROM articles a
          VECTOR_SEARCH(a.embedding, EMBED($1), top_k=3) v
        ORDER BY v.similarity DESC
    ", ["근사 이웃 탐색 알고리즘"])?;

    LF row IN results {
        println("  [{row.similarity:.3f}] {row.title}");
    }

    # 5. 전문 검색
    println("\n--- 전문 검색: 'ACID 트랜잭션' ---");
    ft_results := db.query("
        SELECT a.title, ft.score
        FROM articles a
          FULLTEXT_MATCH(a.body, $1) ft
        ORDER BY ft.score DESC
        LIMIT 3
    ", ["ACID 트랜잭션"])?;

    LF row IN ft_results {
        println("  [{row.score:.3f}] {row.title}");
    }
}
```

예상 출력:

```
샘플 데이터 삽입 완료 (5건)

--- 벡터 검색: '근사 이웃 탐색 알고리즘' ---
  [0.932] HNSW 알고리즘
  [0.781] VaisDB 아키텍처
  [0.654] RAG 파이프라인

--- 전문 검색: 'ACID 트랜잭션' ---
  [1.240] VaisDB 아키텍처
  [0.850] Vais 언어 소개
```

---

## 다음 단계

- [쿼리 가이드](./queries.md) — SQL, VECTOR_SEARCH, GRAPH_TRAVERSE, FULLTEXT_MATCH 상세 설명
- [RAG 기능](./rag.md) — 시맨틱 청킹, RAG_SEARCH, 에이전트 메모리
