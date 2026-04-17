# Getting Started

A guide to installing VaisDB and running your first query.

---

## Requirements

- Vais compiler v1.0.0 or later (`~/.cargo/bin/vaisc`)
- OS: Linux, macOS, Windows (WSL2 recommended)

---

## Installation

### Build from Source

```bash
# Clone the vais-lang monorepo
git clone https://github.com/vaislang/vais-lang.git
cd vais-lang/packages/vaisdb

# Build
vaisc build

# Verify the binary
./vaisdb --version
```

### Package Manager (coming soon)

```bash
# Using the vais package manager (planned for future support)
vpm install vaisdb
```

---

## Usage Modes

VaisDB can run in two modes.

| Mode | Description | Best suited for |
|------|-------------|-----------------|
| Embedded mode | Direct DB file access within the process | Single app, local development |
| TCP server mode | Standalone server process, multiple clients | Microservices, shared DB |

---

## Embedded Mode

### Opening a Database

```vais
U vaisdb::{Database};

F main() {
    # Create a new DB or open an existing one (.vaisdb file)
    db := Database::open("myapp.vaisdb")?;
    println("Database opened successfully");
}
```

### Creating a Table

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

Running from Vais:

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

    println("Table created successfully");
}
```

### Inserting Data

#### Inserting Plain Text

```vais
U vaisdb::{Database};

F main() {
    db := Database::open("myapp.vaisdb")?;

    # Single insert
    db.execute("
        INSERT INTO documents (title, content, author)
        VALUES ($1, $2, $3)
    ", ["Introducing VaisDB", "VaisDB is a RAG-native hybrid database.", "John Doe"])?;

    println("Document inserted successfully");
}
```

#### Inserting with a Vector

The `EMBED()` function automatically generates an embedding vector from text.

```vais
U vaisdb::{Database};

F main() {
    db := Database::open("myapp.vaisdb")?;

    # Automatically generate an embedding with EMBED()
    db.execute("
        INSERT INTO documents (title, content, embedding)
        VALUES ($1, $2, EMBED($2))
    ", ["Vector Search Basics", "The HNSW algorithm is used for approximate nearest neighbor search."])?;

    # Insert a pre-computed vector directly
    embedding: Vec<f32> = compute_embedding("pre-computed embedding");
    db.execute("
        INSERT INTO documents (title, content, embedding)
        VALUES ($1, $2, $3)
    ", ["Direct Insert", "Using a pre-computed vector", embedding])?;
}
```

#### Batch Insert

```vais
U vaisdb::{Database};

F main() {
    db := Database::open("myapp.vaisdb")?;

    # Batch insert wrapped in a transaction
    tx := db.begin()?;

    documents := [
        ("AI Overview", "Fundamental concepts of artificial intelligence"),
        ("Intro to ML", "Introduction to machine learning algorithms"),
        ("Deep Learning", "Neural network architectures and training methods"),
    ];

    LF (title, content) IN documents {
        tx.execute("
            INSERT INTO documents (title, content, embedding)
            VALUES ($1, $2, EMBED($2))
        ", [title, content])?;
    }

    tx.commit()?;
    println("Batch insert complete");
}
```

---

## TCP Server Mode

### Starting the Server

```bash
# Start the server on the default port (5432)
./vaisdb server --db knowledge.vaisdb

# Specify port and options
./vaisdb server \
  --db knowledge.vaisdb \
  --port 5432 \
  --host 0.0.0.0 \
  --max-connections 100 \
  --tls-cert /path/to/cert.pem \
  --tls-key /path/to/key.pem
```

### Connecting as a Client

```vais
U vaisdb::{Client};

F main() {
    # Connect to a remote server via TCP client
    client := Client::connect("vaisdb://localhost:5432/knowledge")?;

    result := client.query("SELECT COUNT(*) FROM documents", [])?;
    println("Document count: {result[0][0]}");

    client.close();
}
```

### Server Configuration File (vaisdb.toml)

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

## Complete First Example

Below is a complete example that inserts documents and performs a vector similarity search.

```vais
U vaisdb::{Database, Row};

F main() {
    # 1. Open the database
    db := Database::open("demo.vaisdb")?;

    # 2. Create the schema
    db.execute("
        CREATE TABLE IF NOT EXISTS articles (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            title     TEXT NOT NULL,
            body      TEXT NOT NULL,
            embedding VECTOR(1536)
        )
    ", [])?;

    # 3. Insert sample data
    samples := [
        ("Introducing the Vais Language",  "Vais is an AI-optimized systems programming language."),
        ("VaisDB Architecture",             "VaisDB unifies 4 engines under a single ACID transaction."),
        ("The HNSW Algorithm",             "A hierarchical navigable small world graph-based approximate nearest neighbor algorithm."),
        ("BM25 Ranking",                   "A probabilistic model for measuring document relevance in full-text search."),
        ("RAG Pipeline",                   "Retrieval-Augmented Generation is a technique for injecting external knowledge into LLMs."),
    ];

    tx := db.begin()?;
    LF (title, body) IN samples {
        tx.execute("
            INSERT INTO articles (title, body, embedding)
            VALUES ($1, $2, EMBED($2))
        ", [title, body])?;
    }
    tx.commit()?;
    println("Sample data inserted ({samples.len()} records)");

    # 4. Vector similarity search
    println("\n--- Vector search: 'approximate nearest neighbor algorithm' ---");
    results := db.query("
        SELECT a.title, v.similarity
        FROM articles a
          VECTOR_SEARCH(a.embedding, EMBED($1), top_k=3) v
        ORDER BY v.similarity DESC
    ", ["approximate nearest neighbor algorithm"])?;

    LF row IN results {
        println("  [{row.similarity:.3f}] {row.title}");
    }

    # 5. Full-text search
    println("\n--- Full-text search: 'ACID transaction' ---");
    ft_results := db.query("
        SELECT a.title, ft.score
        FROM articles a
          FULLTEXT_MATCH(a.body, $1) ft
        ORDER BY ft.score DESC
        LIMIT 3
    ", ["ACID transaction"])?;

    LF row IN ft_results {
        println("  [{row.score:.3f}] {row.title}");
    }
}
```

Expected output:

```
Sample data inserted (5 records)

--- Vector search: 'approximate nearest neighbor algorithm' ---
  [0.932] The HNSW Algorithm
  [0.781] VaisDB Architecture
  [0.654] RAG Pipeline

--- Full-text search: 'ACID transaction' ---
  [1.240] VaisDB Architecture
  [0.850] Introducing the Vais Language
```

---

## Next Steps

- [Query Guide](./queries.md) — Detailed reference for SQL, VECTOR_SEARCH, GRAPH_TRAVERSE, FULLTEXT_MATCH
- [RAG Features](./rag.md) — Semantic chunking, RAG_SEARCH, agent memory
