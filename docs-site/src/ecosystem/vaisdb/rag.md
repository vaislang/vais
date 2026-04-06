# RAG 기능

VaisDB는 RAG(Retrieval-Augmented Generation) 파이프라인을 데이터베이스 레벨에서 기본 지원합니다.
외부 라이브러리 없이 시맨틱 청킹, 임베딩 관리, 컨텍스트 보존, 에이전트 메모리를 처리합니다.

---

## RAG 개요

기존 RAG 파이프라인은 여러 외부 컴포넌트가 필요합니다.

```
기존 RAG:
  문서 → [청킹 라이브러리] → [임베딩 API] → [벡터 DB]
                                            + [관계형 DB]
                                            + [검색 엔진]
                          ↓
                        LLM

VaisDB RAG:
  문서 → [VaisDB]  →  LLM
         ├ 시맨틱 청킹 (내장)
         ├ 임베딩 관리 (내장)
         ├ 벡터 + 그래프 + SQL + 전문 검색 (내장)
         └ 컨텍스트 보존 (내장)
```

---

## 시맨틱 청킹

VaisDB는 의미 경계를 기준으로 문서를 자동 분할합니다. 단순 길이 기반 청킹이 아닌, 문장과 절의 의미 구조를 분석하여 청크 경계를 결정합니다.

### SEMANTIC_CHUNK 함수

```sql
SEMANTIC_CHUNK(content [, options])
```

| 옵션 | 설명 | 기본값 |
|------|------|--------|
| `max_tokens` | 청크당 최대 토큰 수 | `512` |
| `overlap_tokens` | 청크 간 오버랩 토큰 수 | `64` |
| `preserve_sentences` | 문장 경계 유지 여부 | `true` |
| `language` | 언어 코드 | `auto` |

### 예제

```sql
-- 문서를 시맨틱 청크로 분할하여 삽입
INSERT INTO document_chunks (document_id, chunk_index, content, embedding)
SELECT
    $1 AS document_id,
    ROW_NUMBER() OVER () - 1 AS chunk_index,
    chunk.text AS content,
    EMBED(chunk.text) AS embedding
FROM SEMANTIC_CHUNK(
    '긴 문서 내용...',
    max_tokens=512,
    overlap_tokens=64,
    preserve_sentences=true
) AS chunk;
```

Vais에서 사용:

```vais
U vaisdb::{Database};

F ingest_document(db: &Database, doc_id: i64, title: str, content: str) {
    tx := db.begin()?;

    # 원본 문서 저장
    tx.execute("
        INSERT INTO documents (id, title, full_content)
        VALUES ($1, $2, $3)
    ", [doc_id, title, content])?;

    # 시맨틱 청킹 + 임베딩 자동 생성
    tx.execute("
        INSERT INTO document_chunks
            (document_id, chunk_index, content, embedding)
        SELECT
            $1,
            ROW_NUMBER() OVER () - 1,
            chunk.text,
            EMBED(chunk.text)
        FROM SEMANTIC_CHUNK($2,
            max_tokens=512,
            overlap_tokens=64
        ) AS chunk
    ", [doc_id, content])?;

    tx.commit()?;
    println("문서 '{title}' 청킹 완료");
}
```

### 청크 관계 그래프

시맨틱 청킹 후 청크 간 순서 관계와 의미 유사도 관계를 그래프로 자동 구성합니다.

```sql
-- 청크 테이블
CREATE TABLE document_chunks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER REFERENCES documents(id),
    chunk_index INTEGER NOT NULL,
    content     TEXT NOT NULL,
    embedding   VECTOR(1536),
    token_count INTEGER
);

-- 청크 관계 엣지 테이블
CREATE TABLE chunk_edges (
    src_chunk_id INTEGER REFERENCES document_chunks(id),
    dst_chunk_id INTEGER REFERENCES document_chunks(id),
    edge_type    TEXT NOT NULL,  -- 'next', 'prev', 'semantic_similar'
    weight       FLOAT DEFAULT 1.0
);
```

청크 관계 자동 구성:

```vais
U vaisdb::{Database};

F build_chunk_graph(db: &Database, document_id: i64) {
    # 1. 순서 관계 (next/prev) 생성
    db.execute("
        INSERT INTO chunk_edges (src_chunk_id, dst_chunk_id, edge_type, weight)
        SELECT
            c1.id, c2.id, 'next', 1.0
        FROM document_chunks c1
        JOIN document_chunks c2
            ON c1.document_id = c2.document_id
           AND c2.chunk_index = c1.chunk_index + 1
        WHERE c1.document_id = $1
    ", [document_id])?;

    # 2. 의미 유사 관계 (유사도 0.8 이상인 청크 간 연결)
    db.execute("
        INSERT INTO chunk_edges (src_chunk_id, dst_chunk_id, edge_type, weight)
        SELECT c1.id, v.chunk_id, 'semantic_similar', v.similarity
        FROM document_chunks c1
          VECTOR_SEARCH(
              (SELECT embedding FROM document_chunks WHERE id = c1.id),
              c1.embedding, top_k=5
          ) v
          JOIN document_chunks c2 ON v.chunk_id = c2.id
        WHERE c1.document_id = $1
          AND v.similarity > 0.8
          AND c1.id != v.chunk_id
    ", [document_id])?;
}
```

---

## 임베딩 관리

### EMBED 함수

`EMBED()` 함수는 텍스트에서 벡터 임베딩을 자동으로 생성합니다.

```sql
-- 텍스트를 즉시 임베딩으로 변환
SELECT EMBED('검색할 텍스트');

-- 삽입 시 자동 임베딩
INSERT INTO documents (title, content, embedding)
VALUES ('제목', '내용', EMBED('내용'));

-- 쿼리 벡터 생성
SELECT d.title, v.similarity
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED('검색 쿼리'), top_k=10) v;
```

### 임베딩 모델 설정

```sql
-- 기본 임베딩 모델 설정
SET embedding.model = 'text-embedding-3-small';
SET embedding.dimensions = 1536;
SET embedding.api_key = 'sk-...';

-- 또는 로컬 모델 사용
SET embedding.model = 'local:nomic-embed-text-v1.5';
SET embedding.local_path = '/models/nomic-embed-text-v1.5.gguf';
```

Vais 설정:

```vais
U vaisdb::{Database, EmbeddingConfig};

F main() {
    config := EmbeddingConfig {
        model: "text-embedding-3-small",
        dimensions: 1536,
        api_key: std::env::get("OPENAI_API_KEY"),
        batch_size: 100,      # 배치 임베딩 크기
        cache_enabled: true,  # 동일 텍스트 임베딩 캐싱
    };

    db := Database::open_with_config("knowledge.vaisdb", config)?;
}
```

### 임베딩 갱신

문서 내용이 변경되면 임베딩을 재계산해야 합니다.

```sql
-- 특정 문서의 임베딩 갱신
UPDATE documents
SET embedding = EMBED(content)
WHERE id = 42;

-- 임베딩이 없는 문서 일괄 갱신
UPDATE documents
SET embedding = EMBED(content)
WHERE embedding IS NULL;
```

---

## RAG_SEARCH

`RAG_SEARCH`는 VaisDB의 최상위 RAG 검색 함수입니다. 벡터 검색, 그래프 컨텍스트 탐색, 전문 검색을 결합하여 LLM에 전달할 최적의 컨텍스트를 구성합니다.

### 기본 문법

```sql
RAG_SEARCH(query, options)
```

| 옵션 | 설명 | 기본값 |
|------|------|--------|
| `top_k` | 반환할 청크 수 | `10` |
| `expand_context` | 그래프로 주변 청크 포함 | `true` |
| `context_depth` | 컨텍스트 확장 깊이 | `1` |
| `rerank` | 결과 재랭킹 여부 | `true` |
| `min_score` | 최소 관련성 점수 | `0.5` |

### 예제

```sql
-- 기본 RAG 검색
SELECT
    chunk_id,
    document_title,
    content,
    relevance_score,
    context_type      -- 'primary' | 'context'
FROM RAG_SEARCH(
    '트랜스포머의 어텐션 메커니즘은 어떻게 작동하나요?',
    top_k=5,
    expand_context=true,
    context_depth=1
);
```

Vais에서 RAG 파이프라인 구성:

```vais
U vaisdb::{Database, RagResult};

S RagContext {
    chunks:  Vec<str>,
    sources: Vec<str>,
    scores:  Vec<f32>,
}

F build_rag_context(db: &Database, question: str) -> Result<RagContext, str> {
    results := db.query("
        SELECT
            dc.content,
            d.title AS source,
            r.relevance_score
        FROM RAG_SEARCH($1,
            top_k=8,
            expand_context=true,
            context_depth=1,
            rerank=true
        ) r
        JOIN document_chunks dc ON r.chunk_id = dc.id
        JOIN documents d        ON dc.document_id = d.id
        ORDER BY r.relevance_score DESC
    ", [question])?;

    ctx := RagContext {
        chunks:  results.map(|r| r.content),
        sources: results.map(|r| r.source),
        scores:  results.map(|r| r.relevance_score),
    };

    R Ok(ctx);
}

F answer_question(db: &Database, llm: &LlmClient, question: str) -> str {
    ctx := build_rag_context(db, question)?;

    # LLM에 컨텍스트와 함께 질문 전달
    prompt := build_prompt(question, &ctx);
    llm.complete(prompt)?
}
```

---

## 컨텍스트 확장

RAG 검색에서 직접 매칭된 청크만 반환하면 문맥이 끊길 수 있습니다. VaisDB는 그래프를 활용하여 관련 청크를 자동으로 포함합니다.

### 컨텍스트 윈도우 확장

```sql
-- 매칭된 청크와 전/후 청크를 함께 반환
WITH matched_chunks AS (
    SELECT dc.id, dc.document_id, dc.chunk_index, v.similarity
    FROM document_chunks dc
      VECTOR_SEARCH(dc.embedding, EMBED($1), top_k=5) v
    WHERE v.similarity > 0.65
),
context_chunks AS (
    -- 매칭 청크 포함
    SELECT mc.id, mc.similarity, 'primary' AS chunk_type
    FROM matched_chunks mc
    UNION
    -- 앞뒤 청크 포함 (컨텍스트 확장)
    SELECT dc.id, mc.similarity * 0.8, 'context' AS chunk_type
    FROM document_chunks dc
    JOIN matched_chunks mc
        ON dc.document_id = mc.document_id
       AND ABS(dc.chunk_index - mc.chunk_index) = 1
)
SELECT dc.content, cc.similarity, cc.chunk_type
FROM context_chunks cc
JOIN document_chunks dc ON cc.id = dc.id
ORDER BY dc.document_id, dc.chunk_index;
```

### 그래프 기반 컨텍스트 탐색

```sql
-- 의미적으로 연결된 청크까지 포함
WITH primary_chunks AS (
    SELECT dc.id, v.similarity
    FROM document_chunks dc
      VECTOR_SEARCH(dc.embedding, EMBED($1), top_k=5) v
),
extended_chunks AS (
    SELECT pc.id, pc.similarity, 'primary' AS type
    FROM primary_chunks pc
    UNION
    -- 그래프로 연결된 관련 청크
    SELECT g.node_id AS id, pc.similarity * g.relevance, 'graph_context' AS type
    FROM primary_chunks pc
      GRAPH_TRAVERSE(pc.id, direction='any', depth=1,
                     edge_type='semantic_similar') g
)
SELECT dc.content, ec.similarity, ec.type
FROM extended_chunks ec
JOIN document_chunks dc ON ec.id = dc.id
ORDER BY ec.similarity DESC;
```

---

## 에이전트 메모리

VaisDB는 AI 에이전트의 장단기 메모리를 관리하는 전용 기능을 제공합니다.

### 메모리 스키마

```sql
-- 에이전트 메모리 테이블
CREATE TABLE agent_memory (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id     TEXT NOT NULL,
    session_id   TEXT,
    memory_type  TEXT NOT NULL,  -- 'episodic' | 'semantic' | 'procedural'
    content      TEXT NOT NULL,
    embedding    VECTOR(1536),
    importance   FLOAT DEFAULT 1.0,
    access_count INTEGER DEFAULT 0,
    created_at   TIMESTAMP DEFAULT NOW(),
    last_accessed TIMESTAMP DEFAULT NOW(),
    expires_at   TIMESTAMP
);

CREATE INDEX idx_memory_agent  ON agent_memory(agent_id);
CREATE INDEX idx_memory_type   ON agent_memory(memory_type);
CREATE VECTOR INDEX idx_memory_embedding ON agent_memory(embedding)
    USING hnsw WITH (m=16, ef_construction=200);
```

### 메모리 저장 및 검색

```vais
U vaisdb::{Database};

S AgentMemory {
    agent_id:    str,
    session_id:  str,
    memory_type: str,
    content:     str,
    importance:  f32,
}

F store_memory(db: &Database, memory: &AgentMemory) {
    db.execute("
        INSERT INTO agent_memory
            (agent_id, session_id, memory_type, content, embedding, importance)
        VALUES ($1, $2, $3, $4, EMBED($4), $5)
    ", [
        memory.agent_id,
        memory.session_id,
        memory.memory_type,
        memory.content,
        memory.importance,
    ])?;
}

F recall_memories(
    db:       &Database,
    agent_id: str,
    context:  str,
    limit:    i64
) -> Result<Vec<str>, str> {
    # 시맨틱 유사도 + 중요도 + 최근성 결합
    results := db.query("
        SELECT m.content, v.similarity, m.importance, m.last_accessed
        FROM agent_memory m
          VECTOR_SEARCH(m.embedding, EMBED($2), top_k=$3) v
        WHERE m.agent_id = $1
          AND (m.expires_at IS NULL OR m.expires_at > NOW())
        ORDER BY
            (v.similarity * 0.5 + m.importance * 0.3 +
             RECENCY_SCORE(m.last_accessed) * 0.2) DESC
        LIMIT $3
    ", [agent_id, context, limit])?;

    # 접근 카운트 업데이트
    db.execute("
        UPDATE agent_memory
        SET access_count = access_count + 1,
            last_accessed = NOW()
        WHERE agent_id = $1
          AND id IN (
              SELECT id FROM agent_memory
                VECTOR_SEARCH(embedding, EMBED($2), top_k=$3) v
              WHERE agent_id = $1
          )
    ", [agent_id, context, limit])?;

    R Ok(results.map(|r| r.content));
}
```

### 메모리 응집 (Consolidation)

오래된 에피소드 메모리를 시맨틱 메모리로 요약합니다.

```vais
U vaisdb::{Database};

F consolidate_episodic_memories(
    db:       &Database,
    agent_id: str,
    llm:      &LlmClient
) {
    # 최근 7일 에피소드 메모리 조회
    episodes := db.query("
        SELECT content, created_at
        FROM agent_memory
        WHERE agent_id = $1
          AND memory_type = 'episodic'
          AND created_at < NOW() - INTERVAL '7 days'
          AND access_count < 3
        ORDER BY created_at
    ", [agent_id])?;

    I episodes.len() == 0 { R; }

    # LLM으로 요약 생성
    episode_texts := episodes.map(|e| e.content).join("\n");
    summary := llm.summarize(episode_texts)?;

    tx := db.begin()?;

    # 시맨틱 메모리로 저장
    tx.execute("
        INSERT INTO agent_memory
            (agent_id, memory_type, content, embedding, importance)
        VALUES ($1, 'semantic', $2, EMBED($2), 0.8)
    ", [agent_id, summary])?;

    # 응집된 에피소드 메모리 만료 처리
    tx.execute("
        UPDATE agent_memory
        SET expires_at = NOW()
        WHERE agent_id = $1
          AND memory_type = 'episodic'
          AND created_at < NOW() - INTERVAL '7 days'
          AND access_count < 3
    ", [agent_id])?;

    tx.commit()?;
}
```

---

## 완전한 RAG 파이프라인 예제

아래는 문서 수집부터 질의응답까지의 전체 RAG 파이프라인입니다.

```vais
U vaisdb::{Database};
U std::env;

# --- 1단계: 문서 수집 및 인덱싱 ---

F index_document(db: &Database, title: str, content: str) {
    tx := db.begin()?;

    # 원본 문서 저장
    doc_id := tx.execute_returning_id("
        INSERT INTO documents (title, full_content, embedding)
        VALUES ($1, $2, EMBED($2))
        RETURNING id
    ", [title, content])?;

    # 시맨틱 청킹 + 임베딩 자동 생성
    tx.execute("
        INSERT INTO document_chunks
            (document_id, chunk_index, content, embedding, token_count)
        SELECT $1, ROW_NUMBER() OVER () - 1, chunk.text, EMBED(chunk.text), chunk.tokens
        FROM SEMANTIC_CHUNK($2, max_tokens=512, overlap_tokens=64) AS chunk
    ", [doc_id, content])?;

    # 청크 간 관계 그래프 구성
    tx.execute("
        INSERT INTO chunk_edges (src_chunk_id, dst_chunk_id, edge_type, weight)
        SELECT c1.id, c2.id, 'next', 1.0
        FROM document_chunks c1
        JOIN document_chunks c2
            ON c1.document_id = c2.document_id
           AND c2.chunk_index = c1.chunk_index + 1
        WHERE c1.document_id = $1
    ", [doc_id])?;

    tx.commit()?;
    println("인덱싱 완료: {title}");
}

# --- 2단계: RAG 검색 ---

S RagResult {
    content: str,
    source:  str,
    score:   f32,
}

F rag_retrieve(db: &Database, question: str) -> Vec<RagResult> {
    db.query("
        SELECT
            dc.content,
            d.title  AS source,
            r.relevance_score AS score
        FROM RAG_SEARCH($1,
            top_k=8,
            expand_context=true,
            context_depth=1,
            min_score=0.5
        ) r
        JOIN document_chunks dc ON r.chunk_id = dc.id
        JOIN documents d        ON dc.document_id = d.id
        ORDER BY r.relevance_score DESC
    ", [question])?.map(|row| RagResult {
        content: row.content,
        source:  row.source,
        score:   row.score,
    })
}

# --- 3단계: 프롬프트 구성 + LLM 호출 ---

F answer(db: &Database, question: str) -> str {
    chunks := rag_retrieve(db, question);

    context := chunks.map(|c| "출처: {c.source}\n{c.content}").join("\n\n---\n\n");

    prompt := "다음 문서들을 참고하여 질문에 답하세요.\n\n{context}\n\n질문: {question}\n답변:";

    # LLM 호출 (예: vais-server의 LLM 클라이언트)
    llm_call(prompt)
}

F main() {
    db := Database::open("knowledge.vaisdb")?;

    # 문서 인덱싱
    index_document(&db, "VaisDB 소개",    "VaisDB는 RAG-native 하이브리드 데이터베이스...");
    index_document(&db, "HNSW 알고리즘", "HNSW는 계층적 탐색 가능 소세계 그래프...");
    index_document(&db, "BM25 랭킹",     "BM25는 확률 기반 전문 검색 랭킹 모델...");

    # 질의응답
    answer := answer(&db, "VaisDB에서 벡터 검색은 어떻게 동작하나요?");
    println("답변:\n{answer}");
}
```

---

## 운영 팁

### 청크 품질 모니터링

```sql
-- 청크 크기 분포 확인
SELECT
    AVG(token_count) AS avg_tokens,
    MIN(token_count) AS min_tokens,
    MAX(token_count) AS max_tokens,
    COUNT(*)         AS total_chunks
FROM document_chunks;

-- 임베딩 없는 청크 확인
SELECT COUNT(*) AS missing_embeddings
FROM document_chunks
WHERE embedding IS NULL;
```

### VACUUM으로 스토리지 최적화

```sql
-- 삭제된 청크 공간 회수
VACUUM document_chunks;

-- 벡터 인덱스 재구성
REINDEX VECTOR INDEX idx_chunks_embedding;
```

---

## 다음 단계

- [쿼리 가이드](./queries.md) — SQL, VECTOR_SEARCH, GRAPH_TRAVERSE, FULLTEXT_MATCH 상세
- [VaisDB 개요](./README.md) — 아키텍처와 핵심 기능으로 돌아가기
