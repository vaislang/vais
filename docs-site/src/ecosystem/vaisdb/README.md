# VaisDB

**RAG-Native 하이브리드 데이터베이스** — Vector + Graph + SQL + Full-Text를 단일 DB로

---

## 소개

VaisDB는 [Vais](https://github.com/vaislang/vais) 언어로 작성된 RAG-native 하이브리드 데이터베이스입니다. AI/LLM 애플리케이션에 필요한 네 가지 검색 패러다임을 하나의 데이터베이스 파일(`.vaisdb`)과 단일 트랜잭션으로 통합합니다.

---

## 왜 VaisDB인가?

현대 RAG 시스템은 여러 개의 전문 데이터베이스를 동시에 운영해야 합니다.

| 필요 기능 | 기존 솔루션 | 월 비용 (예상) |
|-----------|------------|--------------|
| 벡터 검색 | Pinecone / Milvus | $200 ~ $500 |
| 그래프 탐색 | Neo4j | $200 ~ $500 |
| 관계형 쿼리 | PostgreSQL | $200 ~ $500 |
| 전문 검색 | Elasticsearch | $500 ~ $750 |
| **합계** | **4개 DB + 동기화 로직** | **$1,100 ~ $2,250** |

4개의 커넥션, 4개의 스키마, 4개의 일관성 모델, 그리고 애플리케이션 레벨의 데이터 병합이 필요합니다.

**VaisDB는 이 모두를 하나의 데이터베이스로 대체합니다.**

```
기존:  App → Vector DB ─┐
            Graph DB  ─┤→ LLM
            RDBMS     ─┤
            Search    ─┘

VaisDB: App → VaisDB → LLM
```

---

## 핵심 기능

### 1. 4엔진 통합 하이브리드 쿼리

벡터 유사도, 그래프 탐색, SQL 조인, 전문 검색을 **단일 쿼리**에서 실행합니다.

```sql
SELECT d.title, d.content, v.similarity, g.relationship
FROM documents d
  VECTOR_SEARCH(d.embedding, @query_vector, top_k=10) v
  GRAPH_TRAVERSE(d.id, direction='outbound', depth=2) g
  FULLTEXT_MATCH(d.content, 'transformer attention') ft
WHERE d.created_at > '2025-01-01'
  AND v.similarity > 0.7
ORDER BY v.similarity * 0.4 + ft.score * 0.3 + g.relevance * 0.3
LIMIT 20;
```

### 2. ACID 트랜잭션

벡터 인덱스 업데이트, 그래프 변경, 관계형 쓰기가 **단일 트랜잭션**에서 처리됩니다.

- **WAL(Write-Ahead Log)** 기반 내구성 — 시스템 충돌에도 데이터 안전
- **fsync** 보장 — ACID D(Durability) 완전 준수
- 페이지 기반 버퍼 풀로 모든 엔진이 동일한 스토리지 계층 공유

### 3. RAG-Native 내장 기능

외부 라이브러리 없이 데이터베이스 레벨에서 RAG 작업을 처리합니다.

- **시맨틱 청킹** — 의미 경계를 기준으로 문서를 자동 분할
- **컨텍스트 보존** — 청크 간 관계를 그래프 엣지로 저장
- **팩트 검증** — 벡터 검색 결과를 SQL JOIN으로 교차 검증

### 4. 단일 파일 스토리지

SQLite처럼 데이터베이스 전체가 `.vaisdb` 단일 파일에 저장됩니다.

- 배포, 백업, 이동이 파일 복사 한 번으로 완료
- 임베디드 모드: 외부 서버 프로세스 불필요
- TCP 서버 모드: 다중 클라이언트 동시 접속 지원

---

## 아키텍처

```
┌──────────────────────────────────────────────────────────────┐
│                     하이브리드 쿼리 플래너                      │
│           (코스트 기반 옵티마이저 — 전 엔진 통합)                │
├─────────────┬─────────────┬─────────────┬────────────────────┤
│  벡터 엔진   │  그래프 엔진  │   SQL 엔진  │    전문 검색 엔진    │
│   (HNSW)   │ (Property   │  (B+Tree)  │  (Inverted Index) │
│            │   Graph)    │            │     (BM25)         │
├─────────────┴─────────────┴─────────────┴────────────────────┤
│                     통합 스토리지 엔진                          │
│              (페이지 매니저 + WAL + 버퍼 풀)                    │
├──────────────────────────────────────────────────────────────┤
│                     RAG-Native 레이어                          │
│           (시맨틱 청킹 + 컨텍스트 보존 + 임베딩 관리)             │
└──────────────────────────────────────────────────────────────┘
```

### 벡터 엔진

- **알고리즘**: HNSW(Hierarchical Navigable Small World)
- **유사도 메트릭**: cosine, L2(유클리드), dot product
- **양자화**: 메모리 효율을 위한 스칼라/제품 양자화 지원
- **인덱스**: `VECTOR_SEARCH(table, vector, k)` 함수로 접근

### 그래프 엔진

- **모델**: Property Graph — 노드와 엣지에 임의 속성 부여
- **탐색**: BFS/DFS, 최단 경로, 깊이 제한 탐색
- **쿼리**: `GRAPH_TRAVERSE(start_id, direction, depth)` 함수로 접근

### SQL 엔진

- **파서**: 표준 SQL 파싱
- **실행기**: 코스트 기반 쿼리 최적화
- **지원 기능**: JOIN, CTE(WITH 절), Window Functions, 서브쿼리

### 전문 검색 엔진

- **알고리즘**: BM25(Best Match 25) 랭킹
- **인덱스**: 역 인덱스(Inverted Index)
- **기능**: 토크나이저, 불용어 처리, 형태소 분석 연동
- **쿼리**: `FULLTEXT_MATCH(table, query)` 함수로 접근

---

## 모듈 구조

```
src/
├── storage/    # 페이지 매니저, WAL, 버퍼 풀, B+Tree
├── sql/        # SQL 파서, 실행기, 옵티마이저
├── vector/     # HNSW 인덱스, 양자화, 벡터 스토리지
├── graph/      # Property Graph, 탐색, 경로 탐색
├── fulltext/   # 역 인덱스, BM25, 토크나이저
├── planner/    # 하이브리드 쿼리 플래너, 코스트 모델, 스코어 퓨전
├── rag/        # 시맨틱 청킹, 컨텍스트 보존, RAG_SEARCH
├── server/     # TCP 서버, 와이어 프로토콜, 커넥션 풀
├── ops/        # 운영: 백업, 메트릭, VACUUM, REINDEX
├── security/   # 인증, RBAC, RLS, 암호화, TLS, 감사 로그
└── client/     # 클라이언트 라이브러리
```

---

## 빠른 예제

### Vais 코드에서 사용

```vais
U vaisdb::{Database, QueryResult};

F main() {
    db := Database::open("knowledge.vaisdb")?;

    # 문서 저장
    db.execute("
        INSERT INTO documents (title, content, embedding)
        VALUES ('AI 기초', '인공지능의 기본 개념...', EMBED($1))
    ", ["인공지능은 기계가 인간처럼 학습하고 추론하는 기술입니다."])?;

    # 하이브리드 검색
    results := db.query("
        SELECT d.title, v.similarity
        FROM documents d
          VECTOR_SEARCH(d.embedding, EMBED($1), top_k=5) v
        WHERE v.similarity > 0.7
        ORDER BY v.similarity DESC
    ", ["머신러닝 알고리즘"])?;

    LF row IN results {
        println("{row.title}: {row.similarity}");
    }
}
```

---

## 다음 단계

- [빠른 시작](./getting-started.md) — 설치 및 첫 번째 쿼리
- [쿼리 가이드](./queries.md) — SQL, VECTOR_SEARCH, GRAPH_TRAVERSE, FULLTEXT_MATCH
- [RAG 기능](./rag.md) — 시맨틱 청킹, 임베딩 관리, RAG_SEARCH

---

## 라이선스

MIT
