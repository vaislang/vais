# 쿼리 가이드

VaisDB의 네 가지 쿼리 엔진과 하이브리드 쿼리 작성 방법을 설명합니다.

---

## SQL 쿼리

VaisDB는 표준 SQL을 지원하며 B+Tree 인덱스 기반의 관계형 쿼리 엔진을 내장합니다.

### 기본 CRUD

```sql
-- 삽입
INSERT INTO documents (title, content, author)
VALUES ('제목', '내용', '작성자');

-- 조회
SELECT id, title, author, created_at
FROM documents
WHERE author = '홍길동'
ORDER BY created_at DESC
LIMIT 10;

-- 수정
UPDATE documents
SET content = '수정된 내용', updated_at = NOW()
WHERE id = 42;

-- 삭제
DELETE FROM documents
WHERE created_at < '2024-01-01';
```

### JOIN

```sql
-- INNER JOIN: 문서와 태그 결합
SELECT d.title, t.name AS tag
FROM documents d
INNER JOIN document_tags dt ON d.id = dt.document_id
INNER JOIN tags t            ON dt.tag_id = t.id
WHERE t.name IN ('AI', '머신러닝')
ORDER BY d.title;

-- LEFT JOIN: 댓글 없는 문서도 포함
SELECT d.title, COUNT(c.id) AS comment_count
FROM documents d
LEFT JOIN comments c ON d.id = c.document_id
GROUP BY d.id, d.title
ORDER BY comment_count DESC;
```

### CTE (WITH 절)

```sql
-- 공통 테이블 표현식으로 복잡한 쿼리 분리
WITH recent_docs AS (
    SELECT id, title, content, embedding
    FROM documents
    WHERE created_at > NOW() - INTERVAL '7 days'
),
popular_tags AS (
    SELECT document_id, COUNT(*) AS tag_count
    FROM document_tags
    GROUP BY document_id
    HAVING COUNT(*) >= 3
)
SELECT r.title, p.tag_count
FROM recent_docs r
JOIN popular_tags p ON r.id = p.document_id
ORDER BY p.tag_count DESC;
```

### Window Functions

```sql
-- 저자별 순위 매기기
SELECT
    title,
    author,
    view_count,
    RANK() OVER (PARTITION BY author ORDER BY view_count DESC) AS author_rank,
    SUM(view_count) OVER (PARTITION BY author)                 AS author_total_views
FROM documents
ORDER BY author, author_rank;

-- 이동 평균
SELECT
    date,
    daily_count,
    AVG(daily_count) OVER (
        ORDER BY date
        ROWS BETWEEN 6 PRECEDING AND CURRENT ROW
    ) AS moving_avg_7d
FROM daily_document_stats;
```

### 인덱스 생성

```sql
-- 단일 컬럼 인덱스
CREATE INDEX idx_documents_author ON documents(author);

-- 복합 인덱스
CREATE INDEX idx_documents_author_date ON documents(author, created_at DESC);

-- 부분 인덱스
CREATE INDEX idx_active_documents ON documents(created_at)
WHERE status = 'active';

-- 인덱스 삭제
DROP INDEX idx_documents_author;
```

---

## VECTOR_SEARCH

HNSW(Hierarchical Navigable Small World) 인덱스를 사용한 벡터 유사도 검색입니다.

### 기본 문법

```sql
VECTOR_SEARCH(column, query_vector, top_k [, metric])
```

| 파라미터 | 설명 | 기본값 |
|---------|------|--------|
| `column` | 벡터 컬럼명 | 필수 |
| `query_vector` | 검색 기준 벡터 또는 `EMBED(text)` | 필수 |
| `top_k` | 반환할 최대 결과 수 | 필수 |
| `metric` | `cosine` \| `l2` \| `dot` | `cosine` |

반환 컬럼:
- `similarity` — 유사도 점수 (cosine/dot: 높을수록 유사, l2: 낮을수록 유사)
- `rank` — 유사도 순위 (1부터 시작)

### 예제

```sql
-- 텍스트 쿼리로 유사 문서 검색 (EMBED 함수 사용)
SELECT d.id, d.title, v.similarity
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED('트랜스포머 어텐션 메커니즘'), top_k=10) v
WHERE v.similarity > 0.75
ORDER BY v.similarity DESC;

-- 기존 문서와 유사한 문서 찾기
SELECT d2.id, d2.title, v.similarity
FROM documents d1
  JOIN documents d2 ON d1.id != d2.id
  VECTOR_SEARCH(d2.embedding, d1.embedding, top_k=5) v
WHERE d1.id = 42
ORDER BY v.similarity DESC;

-- L2 거리 기반 검색
SELECT d.title, v.similarity AS l2_distance
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED($1), top_k=5, metric='l2') v
ORDER BY v.similarity ASC;  -- L2는 낮을수록 유사
```

### HNSW 인덱스 설정

```sql
-- 벡터 인덱스 생성
CREATE VECTOR INDEX idx_doc_embedding
ON documents(embedding)
USING hnsw
WITH (
    m = 16,           -- 각 레이어의 최대 연결 수 (기본 16)
    ef_construction = 200,  -- 인덱스 빌드 시 탐색 깊이 (기본 200)
    ef_search = 50    -- 검색 시 탐색 깊이 (기본 50)
);

-- 인덱스 재구성
REINDEX VECTOR INDEX idx_doc_embedding;
```

### Vais에서 사용

```vais
U vaisdb::{Database};

F semantic_search(db: &Database, query: str, threshold: f32) {
    results := db.query("
        SELECT d.title, d.content, v.similarity
        FROM documents d
          VECTOR_SEARCH(d.embedding, EMBED($1), top_k=20) v
        WHERE v.similarity > $2
        ORDER BY v.similarity DESC
    ", [query, threshold])?;

    LF row IN results {
        println("[{row.similarity:.3f}] {row.title}");
    }
}
```

---

## GRAPH_TRAVERSE

Property Graph 모델 기반의 그래프 탐색 쿼리입니다.

### 기본 문법

```sql
GRAPH_TRAVERSE(start_id, direction, depth [, edge_type [, weight_column]])
```

| 파라미터 | 설명 | 기본값 |
|---------|------|--------|
| `start_id` | 시작 노드 ID | 필수 |
| `direction` | `outbound` \| `inbound` \| `any` | 필수 |
| `depth` | 탐색 최대 깊이 | 필수 |
| `edge_type` | 특정 엣지 타입 필터 | 전체 |
| `weight_column` | 가중치 컬럼명 (최단 경로용) | 없음 |

반환 컬럼:
- `node_id` — 탐색된 노드 ID
- `depth` — 시작 노드로부터의 깊이
- `path` — 경로 (노드 ID 배열)
- `relevance` — 경로 기반 관련성 점수

### 그래프 데이터 구조

```sql
-- 노드 테이블
CREATE TABLE knowledge_nodes (
    id         INTEGER PRIMARY KEY,
    label      TEXT NOT NULL,    -- 노드 타입 (Concept, Entity, Document 등)
    name       TEXT NOT NULL,
    properties JSONB
);

-- 엣지 테이블
CREATE TABLE knowledge_edges (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    src_id      INTEGER REFERENCES knowledge_nodes(id),
    dst_id      INTEGER REFERENCES knowledge_nodes(id),
    edge_type   TEXT NOT NULL,   -- 관계 타입 (references, related_to, part_of 등)
    weight      FLOAT DEFAULT 1.0,
    properties  JSONB
);
```

### 예제

```sql
-- 특정 노드에서 2단계 깊이까지 참조 관계 탐색
SELECT n.name, g.depth, g.path
FROM knowledge_nodes n
  GRAPH_TRAVERSE(42, direction='outbound', depth=2, edge_type='references') g
WHERE g.node_id = n.id
ORDER BY g.depth, n.name;

-- 양방향 탐색으로 관련 개념 찾기
SELECT n.name, n.label, g.depth
FROM knowledge_nodes n
  GRAPH_TRAVERSE(100, direction='any', depth=3) g
WHERE g.node_id = n.id
  AND n.label = 'Concept'
ORDER BY g.depth;

-- 두 노드 간 최단 경로
SELECT g.path, g.depth
FROM knowledge_nodes src
  GRAPH_TRAVERSE(src.id, direction='outbound', depth=10) g
WHERE src.id = 10
  AND g.node_id = 50
ORDER BY g.depth
LIMIT 1;
```

### Vais에서 그래프 데이터 삽입

```vais
U vaisdb::{Database};

F build_knowledge_graph(db: &Database) {
    # 노드 삽입
    db.execute("
        INSERT INTO knowledge_nodes (id, label, name)
        VALUES
            (1, 'Concept', 'AI'),
            (2, 'Concept', '머신러닝'),
            (3, 'Concept', '딥러닝'),
            (4, 'Concept', '트랜스포머')
    ", [])?;

    # 엣지 삽입 (관계 정의)
    db.execute("
        INSERT INTO knowledge_edges (src_id, dst_id, edge_type, weight)
        VALUES
            (1, 2, 'includes', 0.9),
            (2, 3, 'includes', 0.85),
            (3, 4, 'evolved_to', 0.95)
    ", [])?;
}
```

---

## FULLTEXT_MATCH

BM25 알고리즘 기반 역 인덱스 전문 검색입니다.

### 기본 문법

```sql
FULLTEXT_MATCH(column, query [, language [, options]])
```

| 파라미터 | 설명 | 기본값 |
|---------|------|--------|
| `column` | 검색 대상 텍스트 컬럼 | 필수 |
| `query` | 검색어 (공백 구분 OR, `+`는 AND, `-`는 NOT) | 필수 |
| `language` | `ko` \| `en` \| `ja` \| `auto` | `auto` |
| `options` | 추가 옵션 (JSON) | `{}` |

반환 컬럼:
- `score` — BM25 관련성 점수 (높을수록 관련성 높음)
- `snippet` — 검색어가 포함된 텍스트 발췌

### 예제

```sql
-- 기본 전문 검색
SELECT d.title, ft.score, ft.snippet
FROM documents d
  FULLTEXT_MATCH(d.content, 'HNSW 근사 최근접 이웃') ft
ORDER BY ft.score DESC
LIMIT 10;

-- AND 검색 (모든 단어 포함)
SELECT d.title, ft.score
FROM documents d
  FULLTEXT_MATCH(d.content, '+벡터 +검색 +알고리즘') ft
ORDER BY ft.score DESC;

-- NOT 검색 (특정 단어 제외)
SELECT d.title, ft.score
FROM documents d
  FULLTEXT_MATCH(d.content, '딥러닝 -CNN') ft
ORDER BY ft.score DESC;

-- 구문 검색
SELECT d.title, ft.score
FROM documents d
  FULLTEXT_MATCH(d.content, '"트랜스포머 어텐션"') ft
ORDER BY ft.score DESC;
```

### 전문 검색 인덱스 생성

```sql
-- 전문 검색 인덱스 생성
CREATE FULLTEXT INDEX idx_documents_content
ON documents(content)
WITH (
    language = 'ko',
    tokenizer = 'ngram',  -- ngram | whitespace | mecab
    ngram_size = 2,
    stopwords = true
);

-- 다중 컬럼 인덱스
CREATE FULLTEXT INDEX idx_documents_full
ON documents(title, content)
WITH (
    language = 'auto',
    boost = '{"title": 2.0, "content": 1.0}'  -- 제목에 가중치 2배
);
```

---

## 하이브리드 쿼리

VaisDB의 핵심 기능: 여러 검색 엔진을 **단일 쿼리**에서 결합합니다.

### 벡터 + SQL 필터링

```sql
-- SQL 필터로 범위 한정 후 벡터 검색
SELECT d.title, d.author, v.similarity
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED($1), top_k=20) v
WHERE d.created_at BETWEEN '2025-01-01' AND '2025-12-31'
  AND d.category = 'AI'
  AND v.similarity > 0.65
ORDER BY v.similarity DESC
LIMIT 10;
```

### 벡터 + 전문 검색 (RRF 퓨전)

```sql
-- Reciprocal Rank Fusion으로 두 검색 결과 결합
SELECT
    d.title,
    v.similarity,
    ft.score,
    -- 가중 점수 계산
    (v.similarity * 0.6 + NORMALIZE(ft.score) * 0.4) AS hybrid_score
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED($1), top_k=50) v
  FULLTEXT_MATCH(d.content, $1) ft
ORDER BY hybrid_score DESC
LIMIT 10;
```

### 벡터 + 그래프 + SQL (완전 하이브리드)

```sql
-- 세 엔진을 모두 활용하는 지식 그래프 검색
SELECT
    d.title,
    d.content,
    v.similarity     AS vector_score,
    ft.score         AS text_score,
    g.depth          AS graph_depth,
    g.relevance      AS graph_score,
    -- 가중 종합 점수
    (v.similarity * 0.4 + NORMALIZE(ft.score) * 0.3 + g.relevance * 0.3) AS final_score
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED($1), top_k=30) v
  FULLTEXT_MATCH(d.content, $1) ft
  GRAPH_TRAVERSE(d.id, direction='outbound', depth=2, edge_type='references') g
WHERE d.status = 'published'
  AND v.similarity > 0.5
ORDER BY final_score DESC
LIMIT 20;
```

### 하이브리드 쿼리 — Vais 함수로 캡슐화

```vais
U vaisdb::{Database, QueryResult};

S HybridSearchConfig {
    vector_weight: f32,
    text_weight:   f32,
    graph_weight:  f32,
    top_k:         i64,
    min_similarity: f32,
}

F hybrid_search(
    db:     &Database,
    query:  str,
    config: &HybridSearchConfig
) -> Result<QueryResult, str> {
    db.query("
        SELECT
            d.id,
            d.title,
            d.content,
            v.similarity,
            ft.score,
            g.relevance,
            (v.similarity * $2 + NORMALIZE(ft.score) * $3 + g.relevance * $4)
                AS final_score
        FROM documents d
          VECTOR_SEARCH(d.embedding, EMBED($1), top_k=$5) v
          FULLTEXT_MATCH(d.content, $1) ft
          GRAPH_TRAVERSE(d.id, direction='outbound', depth=2) g
        WHERE v.similarity > $6
        ORDER BY final_score DESC
        LIMIT $5
    ", [
        query,
        config.vector_weight,
        config.text_weight,
        config.graph_weight,
        config.top_k,
        config.min_similarity,
    ])
}

F main() {
    db := Database::open("knowledge.vaisdb")?;

    config := HybridSearchConfig {
        vector_weight:  0.4,
        text_weight:    0.35,
        graph_weight:   0.25,
        top_k:          10,
        min_similarity: 0.5,
    };

    results := hybrid_search(&db, "트랜스포머 어텐션 메커니즘", &config)?;

    LF row IN results {
        println("[{row.final_score:.3f}] {row.title}");
        println("  벡터: {row.similarity:.3f} | 전문: {row.score:.3f} | 그래프: {row.relevance:.3f}");
    }
}
```

---

## 트랜잭션

모든 엔진 작업을 단일 ACID 트랜잭션으로 묶을 수 있습니다.

```vais
U vaisdb::{Database};

F atomic_document_insert(db: &Database, title: str, content: str) {
    tx := db.begin()?;

    # 1. 문서 삽입 (SQL 엔진)
    doc_id := tx.execute_returning_id("
        INSERT INTO documents (title, content, embedding)
        VALUES ($1, $2, EMBED($2))
    ", [title, content])?;

    # 2. 지식 그래프 노드 추가 (그래프 엔진)
    tx.execute("
        INSERT INTO knowledge_nodes (id, label, name)
        VALUES ($1, 'Document', $2)
    ", [doc_id, title])?;

    # 3. 관련 문서와 연결 (그래프 엔진)
    related := tx.query("
        SELECT d.id
        FROM documents d
          VECTOR_SEARCH(d.embedding, EMBED($1), top_k=3) v
        WHERE v.similarity > 0.8 AND d.id != $2
    ", [content, doc_id])?;

    LF row IN related {
        tx.execute("
            INSERT INTO knowledge_edges (src_id, dst_id, edge_type)
            VALUES ($1, $2, 'related_to')
        ", [doc_id, row.id])?;
    }

    # 모두 성공하면 커밋, 실패하면 자동 롤백
    tx.commit()?;
    println("문서 '{title}' 원자적 삽입 완료");
}
```

---

## 성능 팁

### EXPLAIN으로 쿼리 분석

```sql
EXPLAIN ANALYZE
SELECT d.title, v.similarity
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED('검색어'), top_k=10) v
ORDER BY v.similarity DESC;
```

### 주요 최적화 포인트

1. **벡터 인덱스 파라미터**: `ef_search` 값을 늘리면 정확도 향상, 속도 저하
2. **SQL 필터 우선 적용**: 벡터 검색 전 SQL WHERE 절로 후보 축소
3. **배치 임베딩**: 여러 문서 삽입 시 트랜잭션 묶기
4. **버퍼 풀 크기**: 자주 접근하는 페이지는 메모리에 캐싱됨

```sql
-- 힌트로 실행 계획 강제 지정
SELECT /*+ VECTOR_FIRST */ d.title, v.similarity
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED($1), top_k=10) v
WHERE d.category = 'AI';
```

---

## 다음 단계

- [RAG 기능](./rag.md) — RAG_SEARCH, 시맨틱 청킹, 에이전트 메모리
- [빠른 시작](./getting-started.md) — 설치 및 기본 예제로 돌아가기
