# VaisDB

**RAG-Native Hybrid Database** — Vector + Graph + SQL + Full-Text in a single DB

---

## Introduction

VaisDB is a RAG-native hybrid database written in [Vais](https://github.com/vaislang/vais). It unifies the four search paradigms required by AI/LLM applications into a single database file (`.vaisdb`) and a single transaction model.

---

## Why VaisDB?

Modern RAG systems require running multiple specialized databases simultaneously.

| Required capability | Existing solution | Monthly cost (est.) |
|---------------------|-------------------|---------------------|
| Vector search | Pinecone / Milvus | $200 ~ $500 |
| Graph traversal | Neo4j | $200 ~ $500 |
| Relational query | PostgreSQL | $200 ~ $500 |
| Full-text search | Elasticsearch | $500 ~ $750 |
| **Total** | **4 DBs + sync logic** | **$1,100 ~ $2,250** |

Four connections, four schemas, four consistency models, and application-level data merging are all required.

**VaisDB replaces all of this with a single database.**

```
Before: App → Vector DB ─┐
             Graph DB  ─┤→ LLM
             RDBMS     ─┤
             Search    ─┘

VaisDB: App → VaisDB → LLM
```

---

## Core Features

### 1. 4-Engine Unified Hybrid Query

Vector similarity, graph traversal, SQL joins, and full-text search all execute in a **single query**.

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

### 2. ACID Transactions

Vector index updates, graph mutations, and relational writes are all handled in a **single transaction**.

- **WAL (Write-Ahead Log)**-based durability — data is safe even after a system crash
- **fsync** guarantee — full compliance with ACID Durability
- All engines share the same storage layer via a page-based buffer pool

### 3. RAG-Native Built-in Capabilities

RAG operations are handled at the database level with no external libraries required.

- **Semantic chunking** — documents are automatically split at meaningful boundaries
- **Context preservation** — relationships between chunks are stored as graph edges
- **Fact verification** — vector search results are cross-validated with SQL JOINs

### 4. Single-File Storage

Like SQLite, the entire database is stored in a single `.vaisdb` file.

- Deploy, back up, and move with a single file copy
- Embedded mode: no external server process required
- TCP server mode: supports multiple concurrent client connections

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Hybrid Query Planner                       │
│           (Cost-based optimizer — all engines unified)        │
├─────────────┬─────────────┬─────────────┬────────────────────┤
│ Vector Engine│ Graph Engine│  SQL Engine │ Full-Text Engine   │
│   (HNSW)    │ (Property   │  (B+Tree)   │ (Inverted Index)   │
│             │   Graph)    │             │     (BM25)         │
├─────────────┴─────────────┴─────────────┴────────────────────┤
│                    Unified Storage Engine                      │
│              (Page Manager + WAL + Buffer Pool)               │
├──────────────────────────────────────────────────────────────┤
│                     RAG-Native Layer                           │
│     (Semantic Chunking + Context Preservation + Embedding Mgmt)│
└──────────────────────────────────────────────────────────────┘
```

### Vector Engine

- **Algorithm**: HNSW (Hierarchical Navigable Small World)
- **Similarity metrics**: cosine, L2 (Euclidean), dot product
- **Quantization**: scalar/product quantization for memory efficiency
- **Access**: via the `VECTOR_SEARCH(table, vector, k)` function

### Graph Engine

- **Model**: Property Graph — arbitrary properties on nodes and edges
- **Traversal**: BFS/DFS, shortest path, depth-limited traversal
- **Access**: via the `GRAPH_TRAVERSE(start_id, direction, depth)` function

### SQL Engine

- **Parser**: standard SQL parsing
- **Executor**: cost-based query optimization
- **Supported features**: JOIN, CTE (WITH clause), Window Functions, subqueries

### Full-Text Search Engine

- **Algorithm**: BM25 (Best Match 25) ranking
- **Index**: Inverted Index
- **Features**: tokenizer, stopword handling, morphological analysis integration
- **Access**: via the `FULLTEXT_MATCH(table, query)` function

---

## Module Structure

```
src/
├── storage/    # Page manager, WAL, buffer pool, B+Tree
├── sql/        # SQL parser, executor, optimizer
├── vector/     # HNSW index, quantization, vector storage
├── graph/      # Property Graph, traversal, path search
├── fulltext/   # Inverted index, BM25, tokenizer
├── planner/    # Hybrid query planner, cost model, score fusion
├── rag/        # Semantic chunking, context preservation, RAG_SEARCH
├── server/     # TCP server, wire protocol, connection pool
├── ops/        # Operations: backup, metrics, VACUUM, REINDEX
├── security/   # Auth, RBAC, RLS, encryption, TLS, audit log
└── client/     # Client libraries
```

---

## Quick Example

### Usage from Vais Code

```vais
U vaisdb::{Database, QueryResult};

F main() {
    db := Database::open("knowledge.vaisdb")?;

    # Store a document
    db.execute("
        INSERT INTO documents (title, content, embedding)
        VALUES ('AI Basics', 'Foundational concepts of artificial intelligence...', EMBED($1))
    ", ["Artificial intelligence is the technology that enables machines to learn and reason like humans."])?;

    # Hybrid search
    results := db.query("
        SELECT d.title, v.similarity
        FROM documents d
          VECTOR_SEARCH(d.embedding, EMBED($1), top_k=5) v
        WHERE v.similarity > 0.7
        ORDER BY v.similarity DESC
    ", ["machine learning algorithms"])?;

    LF row IN results {
        println("{row.title}: {row.similarity}");
    }
}
```

---

## Next Steps

- [Getting Started](./getting-started.md) — Installation and your first query
- [Query Guide](./queries.md) — SQL, VECTOR_SEARCH, GRAPH_TRAVERSE, FULLTEXT_MATCH
- [RAG Features](./rag.md) — Semantic chunking, embedding management, RAG_SEARCH

---

## License

MIT
