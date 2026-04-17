# Query Guide

A guide to VaisDB's four query engines and how to write hybrid queries.

---

## SQL Queries

VaisDB supports standard SQL and includes a built-in relational query engine backed by a B+Tree index.

### Basic CRUD

```sql
-- Insert
INSERT INTO documents (title, content, author)
VALUES ('Title', 'Content', 'Author');

-- Select
SELECT id, title, author, created_at
FROM documents
WHERE author = 'John Doe'
ORDER BY created_at DESC
LIMIT 10;

-- Update
UPDATE documents
SET content = 'Updated content', updated_at = NOW()
WHERE id = 42;

-- Delete
DELETE FROM documents
WHERE created_at < '2024-01-01';
```

### JOIN

```sql
-- INNER JOIN: combine documents with their tags
SELECT d.title, t.name AS tag
FROM documents d
INNER JOIN document_tags dt ON d.id = dt.document_id
INNER JOIN tags t            ON dt.tag_id = t.id
WHERE t.name IN ('AI', 'machine learning')
ORDER BY d.title;

-- LEFT JOIN: include documents with no comments
SELECT d.title, COUNT(c.id) AS comment_count
FROM documents d
LEFT JOIN comments c ON d.id = c.document_id
GROUP BY d.id, d.title
ORDER BY comment_count DESC;
```

### CTE (WITH Clause)

```sql
-- Decompose complex queries using common table expressions
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
-- Rank documents per author
SELECT
    title,
    author,
    view_count,
    RANK() OVER (PARTITION BY author ORDER BY view_count DESC) AS author_rank,
    SUM(view_count) OVER (PARTITION BY author)                 AS author_total_views
FROM documents
ORDER BY author, author_rank;

-- Moving average
SELECT
    date,
    daily_count,
    AVG(daily_count) OVER (
        ORDER BY date
        ROWS BETWEEN 6 PRECEDING AND CURRENT ROW
    ) AS moving_avg_7d
FROM daily_document_stats;
```

### Creating Indexes

```sql
-- Single-column index
CREATE INDEX idx_documents_author ON documents(author);

-- Composite index
CREATE INDEX idx_documents_author_date ON documents(author, created_at DESC);

-- Partial index
CREATE INDEX idx_active_documents ON documents(created_at)
WHERE status = 'active';

-- Drop an index
DROP INDEX idx_documents_author;
```

---

## VECTOR_SEARCH

Vector similarity search using an HNSW (Hierarchical Navigable Small World) index.

### Syntax

```sql
VECTOR_SEARCH(column, query_vector, top_k [, metric])
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `column` | Vector column name | required |
| `query_vector` | Reference vector or `EMBED(text)` | required |
| `top_k` | Maximum number of results to return | required |
| `metric` | `cosine` \| `l2` \| `dot` | `cosine` |

Returned columns:
- `similarity` — similarity score (cosine/dot: higher is more similar; l2: lower is more similar)
- `rank` — similarity rank (starting from 1)

### Examples

```sql
-- Find similar documents from a text query (using EMBED)
SELECT d.id, d.title, v.similarity
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED('transformer attention mechanism'), top_k=10) v
WHERE v.similarity > 0.75
ORDER BY v.similarity DESC;

-- Find documents similar to an existing document
SELECT d2.id, d2.title, v.similarity
FROM documents d1
  JOIN documents d2 ON d1.id != d2.id
  VECTOR_SEARCH(d2.embedding, d1.embedding, top_k=5) v
WHERE d1.id = 42
ORDER BY v.similarity DESC;

-- L2 distance-based search
SELECT d.title, v.similarity AS l2_distance
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED($1), top_k=5, metric='l2') v
ORDER BY v.similarity ASC;  -- lower L2 = more similar
```

### HNSW Index Configuration

```sql
-- Create a vector index
CREATE VECTOR INDEX idx_doc_embedding
ON documents(embedding)
USING hnsw
WITH (
    m = 16,           -- max connections per layer (default 16)
    ef_construction = 200,  -- search depth during index build (default 200)
    ef_search = 50    -- search depth during query (default 50)
);

-- Rebuild the index
REINDEX VECTOR INDEX idx_doc_embedding;
```

### Usage from Vais

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

Graph traversal queries based on the Property Graph model.

### Syntax

```sql
GRAPH_TRAVERSE(start_id, direction, depth [, edge_type [, weight_column]])
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `start_id` | Starting node ID | required |
| `direction` | `outbound` \| `inbound` \| `any` | required |
| `depth` | Maximum traversal depth | required |
| `edge_type` | Filter by specific edge type | all |
| `weight_column` | Weight column name (for shortest path) | none |

Returned columns:
- `node_id` — ID of the traversed node
- `depth` — depth from the starting node
- `path` — path (array of node IDs)
- `relevance` — path-based relevance score

### Graph Data Structure

```sql
-- Node table
CREATE TABLE knowledge_nodes (
    id         INTEGER PRIMARY KEY,
    label      TEXT NOT NULL,    -- node type (Concept, Entity, Document, etc.)
    name       TEXT NOT NULL,
    properties JSONB
);

-- Edge table
CREATE TABLE knowledge_edges (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    src_id      INTEGER REFERENCES knowledge_nodes(id),
    dst_id      INTEGER REFERENCES knowledge_nodes(id),
    edge_type   TEXT NOT NULL,   -- relationship type (references, related_to, part_of, etc.)
    weight      FLOAT DEFAULT 1.0,
    properties  JSONB
);
```

### Examples

```sql
-- Traverse reference relationships up to depth 2 from a given node
SELECT n.name, g.depth, g.path
FROM knowledge_nodes n
  GRAPH_TRAVERSE(42, direction='outbound', depth=2, edge_type='references') g
WHERE g.node_id = n.id
ORDER BY g.depth, n.name;

-- Find related concepts via bidirectional traversal
SELECT n.name, n.label, g.depth
FROM knowledge_nodes n
  GRAPH_TRAVERSE(100, direction='any', depth=3) g
WHERE g.node_id = n.id
  AND n.label = 'Concept'
ORDER BY g.depth;

-- Shortest path between two nodes
SELECT g.path, g.depth
FROM knowledge_nodes src
  GRAPH_TRAVERSE(src.id, direction='outbound', depth=10) g
WHERE src.id = 10
  AND g.node_id = 50
ORDER BY g.depth
LIMIT 1;
```

### Inserting Graph Data from Vais

```vais
U vaisdb::{Database};

F build_knowledge_graph(db: &Database) {
    # Insert nodes
    db.execute("
        INSERT INTO knowledge_nodes (id, label, name)
        VALUES
            (1, 'Concept', 'AI'),
            (2, 'Concept', 'Machine Learning'),
            (3, 'Concept', 'Deep Learning'),
            (4, 'Concept', 'Transformer')
    ", [])?;

    # Insert edges (define relationships)
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

Inverted index full-text search based on the BM25 algorithm.

### Syntax

```sql
FULLTEXT_MATCH(column, query [, language [, options]])
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `column` | Text column to search | required |
| `query` | Search terms (space-delimited OR, `+` for AND, `-` for NOT) | required |
| `language` | `ko` \| `en` \| `ja` \| `auto` | `auto` |
| `options` | Additional options (JSON) | `{}` |

Returned columns:
- `score` — BM25 relevance score (higher = more relevant)
- `snippet` — text excerpt containing the search terms

### Examples

```sql
-- Basic full-text search
SELECT d.title, ft.score, ft.snippet
FROM documents d
  FULLTEXT_MATCH(d.content, 'HNSW approximate nearest neighbor') ft
ORDER BY ft.score DESC
LIMIT 10;

-- AND search (all terms must be present)
SELECT d.title, ft.score
FROM documents d
  FULLTEXT_MATCH(d.content, '+vector +search +algorithm') ft
ORDER BY ft.score DESC;

-- NOT search (exclude a specific term)
SELECT d.title, ft.score
FROM documents d
  FULLTEXT_MATCH(d.content, 'deep learning -CNN') ft
ORDER BY ft.score DESC;

-- Phrase search
SELECT d.title, ft.score
FROM documents d
  FULLTEXT_MATCH(d.content, '"transformer attention"') ft
ORDER BY ft.score DESC;
```

### Creating a Full-Text Index

```sql
-- Create a full-text index
CREATE FULLTEXT INDEX idx_documents_content
ON documents(content)
WITH (
    language = 'en',
    tokenizer = 'ngram',  -- ngram | whitespace | mecab
    ngram_size = 2,
    stopwords = true
);

-- Multi-column index
CREATE FULLTEXT INDEX idx_documents_full
ON documents(title, content)
WITH (
    language = 'auto',
    boost = '{"title": 2.0, "content": 1.0}'  -- 2x weight on title
);
```

---

## Hybrid Queries

VaisDB's core capability: combining multiple search engines in a **single query**.

### Vector + SQL Filtering

```sql
-- Narrow the candidate set with a SQL filter, then run vector search
SELECT d.title, d.author, v.similarity
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED($1), top_k=20) v
WHERE d.created_at BETWEEN '2025-01-01' AND '2025-12-31'
  AND d.category = 'AI'
  AND v.similarity > 0.65
ORDER BY v.similarity DESC
LIMIT 10;
```

### Vector + Full-Text Search (RRF Fusion)

```sql
-- Combine two search results with Reciprocal Rank Fusion
SELECT
    d.title,
    v.similarity,
    ft.score,
    -- weighted score
    (v.similarity * 0.6 + NORMALIZE(ft.score) * 0.4) AS hybrid_score
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED($1), top_k=50) v
  FULLTEXT_MATCH(d.content, $1) ft
ORDER BY hybrid_score DESC
LIMIT 10;
```

### Vector + Graph + SQL (Full Hybrid)

```sql
-- Knowledge graph search leveraging all three engines
SELECT
    d.title,
    d.content,
    v.similarity     AS vector_score,
    ft.score         AS text_score,
    g.depth          AS graph_depth,
    g.relevance      AS graph_score,
    -- weighted combined score
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

### Hybrid Query — Encapsulated as a Vais Function

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

    results := hybrid_search(&db, "transformer attention mechanism", &config)?;

    LF row IN results {
        println("[{row.final_score:.3f}] {row.title}");
        println("  vector: {row.similarity:.3f} | text: {row.score:.3f} | graph: {row.relevance:.3f}");
    }
}
```

---

## Transactions

All engine operations can be wrapped in a single ACID transaction.

```vais
U vaisdb::{Database};

F atomic_document_insert(db: &Database, title: str, content: str) {
    tx := db.begin()?;

    # 1. Insert the document (SQL engine)
    doc_id := tx.execute_returning_id("
        INSERT INTO documents (title, content, embedding)
        VALUES ($1, $2, EMBED($2))
    ", [title, content])?;

    # 2. Add a knowledge graph node (graph engine)
    tx.execute("
        INSERT INTO knowledge_nodes (id, label, name)
        VALUES ($1, 'Document', $2)
    ", [doc_id, title])?;

    # 3. Link to related documents (graph engine)
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

    # Commit on full success; auto-rollback on any failure
    tx.commit()?;
    println("Document '{title}' atomically inserted");
}
```

---

## Performance Tips

### Analyze Queries with EXPLAIN

```sql
EXPLAIN ANALYZE
SELECT d.title, v.similarity
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED('search term'), top_k=10) v
ORDER BY v.similarity DESC;
```

### Key Optimization Points

1. **Vector index parameters**: increasing `ef_search` improves accuracy at the cost of speed
2. **Apply SQL filters first**: narrow candidates with a WHERE clause before vector search
3. **Batch embeddings**: wrap multi-document inserts in a transaction
4. **Buffer pool size**: frequently accessed pages are cached in memory

```sql
-- Force an execution plan with a hint
SELECT /*+ VECTOR_FIRST */ d.title, v.similarity
FROM documents d
  VECTOR_SEARCH(d.embedding, EMBED($1), top_k=10) v
WHERE d.category = 'AI';
```

---

## Next Steps

- [RAG Features](./rag.md) — RAG_SEARCH, semantic chunking, agent memory
- [Getting Started](./getting-started.md) — Back to installation and basic examples
