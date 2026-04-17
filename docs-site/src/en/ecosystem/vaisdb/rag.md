# RAG Features

VaisDB provides first-class support for RAG (Retrieval-Augmented Generation) pipelines at the database level.
Semantic chunking, embedding management, context preservation, and agent memory are all handled without any external libraries.

---

## RAG Overview

A traditional RAG pipeline requires multiple external components.

```
Traditional RAG:
  Document → [Chunking library] → [Embedding API] → [Vector DB]
                                                   + [Relational DB]
                                                   + [Search engine]
                               ↓
                             LLM

VaisDB RAG:
  Document → [VaisDB]  →  LLM
             ├ Semantic chunking (built-in)
             ├ Embedding management (built-in)
             ├ Vector + Graph + SQL + Full-text search (built-in)
             └ Context preservation (built-in)
```

---

## Semantic Chunking

VaisDB automatically splits documents at meaningful boundaries. Rather than simple length-based chunking, it analyzes the semantic structure of sentences and clauses to determine chunk boundaries.

### SEMANTIC_CHUNK Function

```sql
SEMANTIC_CHUNK(content [, options])
```

| Option | Description | Default |
|--------|-------------|---------|
| `max_tokens` | Maximum tokens per chunk | `512` |
| `overlap_tokens` | Number of overlapping tokens between chunks | `64` |
| `preserve_sentences` | Whether to respect sentence boundaries | `true` |
| `language` | Language code | `auto` |

### Examples

```sql
-- Split a document into semantic chunks and insert them
INSERT INTO document_chunks (document_id, chunk_index, content, embedding)
SELECT
    $1 AS document_id,
    ROW_NUMBER() OVER () - 1 AS chunk_index,
    chunk.text AS content,
    EMBED(chunk.text) AS embedding
FROM SEMANTIC_CHUNK(
    'Long document content...',
    max_tokens=512,
    overlap_tokens=64,
    preserve_sentences=true
) AS chunk;
```

Usage from Vais:

```vais
U vaisdb::{Database};

F ingest_document(db: &Database, doc_id: i64, title: str, content: str) {
    tx := db.begin()?;

    # Store the original document
    tx.execute("
        INSERT INTO documents (id, title, full_content)
        VALUES ($1, $2, $3)
    ", [doc_id, title, content])?;

    # Semantic chunking + automatic embedding generation
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
    println("Document '{title}' chunked successfully");
}
```

### Chunk Relationship Graph

After semantic chunking, sequential and semantic similarity relationships between chunks are automatically built as a graph.

```sql
-- Chunk table
CREATE TABLE document_chunks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER REFERENCES documents(id),
    chunk_index INTEGER NOT NULL,
    content     TEXT NOT NULL,
    embedding   VECTOR(1536),
    token_count INTEGER
);

-- Chunk relationship edge table
CREATE TABLE chunk_edges (
    src_chunk_id INTEGER REFERENCES document_chunks(id),
    dst_chunk_id INTEGER REFERENCES document_chunks(id),
    edge_type    TEXT NOT NULL,  -- 'next', 'prev', 'semantic_similar'
    weight       FLOAT DEFAULT 1.0
);
```

Automatically building chunk relationships:

```vais
U vaisdb::{Database};

F build_chunk_graph(db: &Database, document_id: i64) {
    # 1. Build sequential relationships (next/prev)
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

    # 2. Build semantic similarity relationships (connect chunks with similarity >= 0.8)
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

## Embedding Management

### EMBED Function

The `EMBED()` function automatically generates a vector embedding from text.

```sql
-- Convert text to an embedding immediately
SELECT EMBED('text to search');

-- Automatic embedding on insert
INSERT INTO documents (title, content, embedding)
VALUES ('Title', 'Content', EMBED('Content'));

-- Generate a query vector
SELECT d.title, v.similarity
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED('search query'), top_k=10) v;
```

### Embedding Model Configuration

```sql
-- Set the default embedding model
SET embedding.model = 'text-embedding-3-small';
SET embedding.dimensions = 1536;
SET embedding.api_key = 'sk-...';

-- Or use a local model
SET embedding.model = 'local:nomic-embed-text-v1.5';
SET embedding.local_path = '/models/nomic-embed-text-v1.5.gguf';
```

Vais configuration:

```vais
U vaisdb::{Database, EmbeddingConfig};

F main() {
    config := EmbeddingConfig {
        model: "text-embedding-3-small",
        dimensions: 1536,
        api_key: std::env::get("OPENAI_API_KEY"),
        batch_size: 100,      # batch embedding size
        cache_enabled: true,  # cache embeddings for identical text
    };

    db := Database::open_with_config("knowledge.vaisdb", config)?;
}
```

### Refreshing Embeddings

When document content changes, embeddings must be recomputed.

```sql
-- Refresh the embedding for a specific document
UPDATE documents
SET embedding = EMBED(content)
WHERE id = 42;

-- Batch refresh for documents missing embeddings
UPDATE documents
SET embedding = EMBED(content)
WHERE embedding IS NULL;
```

---

## RAG_SEARCH

`RAG_SEARCH` is VaisDB's top-level RAG search function. It combines vector search, graph-based context traversal, and full-text search to assemble the optimal context for delivery to an LLM.

### Syntax

```sql
RAG_SEARCH(query, options)
```

| Option | Description | Default |
|--------|-------------|---------|
| `top_k` | Number of chunks to return | `10` |
| `expand_context` | Include surrounding chunks via graph | `true` |
| `context_depth` | Context expansion depth | `1` |
| `rerank` | Whether to rerank results | `true` |
| `min_score` | Minimum relevance score | `0.5` |

### Examples

```sql
-- Basic RAG search
SELECT
    chunk_id,
    document_title,
    content,
    relevance_score,
    context_type      -- 'primary' | 'context'
FROM RAG_SEARCH(
    'How does the transformer attention mechanism work?',
    top_k=5,
    expand_context=true,
    context_depth=1
);
```

Assembling a RAG pipeline from Vais:

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

    # Pass the context and question to the LLM
    prompt := build_prompt(question, &ctx);
    llm.complete(prompt)?
}
```

---

## Context Expansion

Returning only the directly matched chunks from a RAG search can break context continuity. VaisDB uses the graph to automatically include related chunks.

### Context Window Expansion

```sql
-- Return matched chunks along with their preceding and following chunks
WITH matched_chunks AS (
    SELECT dc.id, dc.document_id, dc.chunk_index, v.similarity
    FROM document_chunks dc
      VECTOR_SEARCH(dc.embedding, EMBED($1), top_k=5) v
    WHERE v.similarity > 0.65
),
context_chunks AS (
    -- Include matched chunks
    SELECT mc.id, mc.similarity, 'primary' AS chunk_type
    FROM matched_chunks mc
    UNION
    -- Include adjacent chunks (context expansion)
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

### Graph-Based Context Traversal

```sql
-- Include semantically connected chunks as well
WITH primary_chunks AS (
    SELECT dc.id, v.similarity
    FROM document_chunks dc
      VECTOR_SEARCH(dc.embedding, EMBED($1), top_k=5) v
),
extended_chunks AS (
    SELECT pc.id, pc.similarity, 'primary' AS type
    FROM primary_chunks pc
    UNION
    -- Graph-connected related chunks
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

## Agent Memory

VaisDB provides dedicated features for managing short-term and long-term memory for AI agents.

### Memory Schema

```sql
-- Agent memory table
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

### Storing and Retrieving Memory

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
    # Combine semantic similarity + importance + recency
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

    # Update access count
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

### Memory Consolidation

Summarize older episodic memories into semantic memories.

```vais
U vaisdb::{Database};

F consolidate_episodic_memories(
    db:       &Database,
    agent_id: str,
    llm:      &LlmClient
) {
    # Retrieve episodic memories from the past 7 days
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

    # Generate a summary with the LLM
    episode_texts := episodes.map(|e| e.content).join("\n");
    summary := llm.summarize(episode_texts)?;

    tx := db.begin()?;

    # Store as semantic memory
    tx.execute("
        INSERT INTO agent_memory
            (agent_id, memory_type, content, embedding, importance)
        VALUES ($1, 'semantic', $2, EMBED($2), 0.8)
    ", [agent_id, summary])?;

    # Expire the consolidated episodic memories
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

## Complete RAG Pipeline Example

Below is a full RAG pipeline from document ingestion through question answering.

```vais
U vaisdb::{Database};
U std::env;

# --- Step 1: Document ingestion and indexing ---

F index_document(db: &Database, title: str, content: str) {
    tx := db.begin()?;

    # Store the original document
    doc_id := tx.execute_returning_id("
        INSERT INTO documents (title, full_content, embedding)
        VALUES ($1, $2, EMBED($2))
        RETURNING id
    ", [title, content])?;

    # Semantic chunking + automatic embedding generation
    tx.execute("
        INSERT INTO document_chunks
            (document_id, chunk_index, content, embedding, token_count)
        SELECT $1, ROW_NUMBER() OVER () - 1, chunk.text, EMBED(chunk.text), chunk.tokens
        FROM SEMANTIC_CHUNK($2, max_tokens=512, overlap_tokens=64) AS chunk
    ", [doc_id, content])?;

    # Build chunk relationship graph
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
    println("Indexing complete: {title}");
}

# --- Step 2: RAG retrieval ---

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

# --- Step 3: Prompt construction + LLM call ---

F answer(db: &Database, question: str) -> str {
    chunks := rag_retrieve(db, question);

    context := chunks.map(|c| "Source: {c.source}\n{c.content}").join("\n\n---\n\n");

    prompt := "Answer the question based on the following documents.\n\n{context}\n\nQuestion: {question}\nAnswer:";

    # LLM call (e.g., vais-server LLM client)
    llm_call(prompt)
}

F main() {
    db := Database::open("knowledge.vaisdb")?;

    # Index documents
    index_document(&db, "Introducing VaisDB",  "VaisDB is a RAG-native hybrid database...");
    index_document(&db, "The HNSW Algorithm",  "HNSW is a hierarchical navigable small world graph...");
    index_document(&db, "BM25 Ranking",        "BM25 is a probabilistic full-text search ranking model...");

    # Question answering
    answer := answer(&db, "How does vector search work in VaisDB?");
    println("Answer:\n{answer}");
}
```

---

## Operational Tips

### Monitor Chunk Quality

```sql
-- Check chunk size distribution
SELECT
    AVG(token_count) AS avg_tokens,
    MIN(token_count) AS min_tokens,
    MAX(token_count) AS max_tokens,
    COUNT(*)         AS total_chunks
FROM document_chunks;

-- Find chunks missing embeddings
SELECT COUNT(*) AS missing_embeddings
FROM document_chunks
WHERE embedding IS NULL;
```

### Optimize Storage with VACUUM

```sql
-- Reclaim space from deleted chunks
VACUUM document_chunks;

-- Rebuild the vector index
REINDEX VECTOR INDEX idx_chunks_embedding;
```

---

## Next Steps

- [Query Guide](./queries.md) — Detailed reference for SQL, VECTOR_SEARCH, GRAPH_TRAVERSE, FULLTEXT_MATCH
- [VaisDB Overview](./README.md) — Back to architecture and core features
