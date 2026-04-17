# Database Integration

vais-server integrates natively with vaisdb. `QueryBuilder` sends queries directly over the vaisdb wire protocol without an ORM translation layer.

```
Handler (vais-server)
    ↓  QueryBuilder.build() → SQL / VECTOR_SEARCH / GRAPH_TRAVERSE / FULLTEXT_MATCH
    ↓  DbConnection.execute(sql)
vaisdb (Vector + Graph + SQL + Full-text)
```

---

## DbConnection — Database Connection

### Embedded Mode (File-based)

```vais
U db/connection

config := DbConfig.embedded("./data/myapp.vaisdb")
M DbConnection.connect(config) {
    Ok(conn) => {
        println("Connected: {conn.to_string()}")
    },
    Err(e) => {
        println("Connection failed: {e.message}")
        R 1
    },
}
```

### TCP Mode (Remote Server)

```vais
config := DbConfig.tcp("127.0.0.1", 7300)
M DbConnection.connect(config) {
    Ok(conn) => { /* ... */ },
    Err(e)   => { /* ... */ },
}
```

### DbConfig Fields

| Factory Method | Mode | Required Parameters |
|---------------|------|---------------------|
| `DbConfig.embedded(path)` | `DbMode.Embedded` | `db_path` |
| `DbConfig.tcp(host, port)` | `DbMode.Tcp` | `host`, `port` |

The default timeout is 5000ms.

### Executing Queries

```vais
sql := "SELECT id, name FROM users WHERE id = 1"
M conn.execute(sql) {
    Ok(result) => {
        println("Row count: {result.row_count()}")
        I i = 0; i < result.row_count(); i = i + 1 {
            row := result.rows.get(i)
            println("  {row.get()}")
        }
    },
    Err(e) => {
        println("Query failed: {e.message}")
    },
}
```

`QueryResult` struct:

| Field | Type | Meaning |
|-------|------|---------|
| `rows` | `Vec<Row>` | List of result rows |
| `affected_rows` | `i64` | Number of rows affected by INSERT/UPDATE/DELETE |
| `columns` | `Vec<str>` | List of column names |

---

## ConnectionPool — Connection Pool

In production, use `ConnectionPool` instead of opening a new connection for every request.

### Creating a Pool

```vais
U db/connection
U db/pool

db_config   := DbConfig.embedded("./data/myapp.vaisdb")
pool_config := PoolConfig.default()   # min=2, max=10, idle_timeout=30s

M ConnectionPool.new(db_config, pool_config) {
    Ok(mut pool) => {
        # Use the pool
        stats := pool.stats()
        println("{stats.to_string()}")
    },
    Err(e) => {
        println("Pool creation failed: {e.message}")
        R 1
    },
}
```

### Acquiring and Releasing Connections

```vais
M pool.acquire() {
    Ok(conn) => {
        # Execute query
        M conn.execute("SELECT 1") {
            Ok(_)  => { println("Health check passed") },
            Err(e) => { println("Error: {e.message}") },
        }
        # Connection must be released when done
        pool.release(conn)
    },
    Err(e) => {
        println("Pool exhausted: {e.message}")
    },
}
```

### PoolConfig Parameters

```vais
pool_config := PoolConfig.new(
    2,      # min_connections — connections opened at startup
    20,     # max_connections — maximum number of connections
    60000,  # idle_timeout_ms — idle connection timeout (ms)
)
```

### PoolStats

```vais
stats := pool.stats()
# PoolStats { active=3, idle=7, total=10 }
println(stats.to_string())
```

| Field | Meaning |
|-------|---------|
| `active` | Connections currently in use |
| `idle` | Connections waiting in the pool |
| `total` | active + idle |

### Health Check

```vais
pool.health_check()   # sends SELECT 1 ping to idle connections and replaces dead ones
```

---

## QueryBuilder — Query Builder

`QueryBuilder` supports SQL, vector search, graph traversal, and full-text search through a single fluent API.

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

If no columns are specified, `*` is used.

```vais
sql := QueryBuilder.new()
    .select("products")
    .where_clause("price < 10000")
    .build()
# SELECT * FROM products WHERE price < 10000
```

### Combining Multiple WHERE Conditions

Multiple calls to `.where_clause()` are combined with AND.

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

### Transactions

```vais
begin_sql  := QueryBuilder.new().begin_transaction().build()  # "BEGIN"
commit_sql := QueryBuilder.new().commit().build()             # "COMMIT"
rb_sql     := QueryBuilder.new().rollback().build()           # "ROLLBACK"

M conn.execute(begin_sql) {
    Ok(_)  => {},
    Err(e) => { R Err(e) },
}
# ... execute DML queries ...
M conn.execute(commit_sql) {
    Ok(_)  => { println("Transaction committed") },
    Err(e) => {
        conn.execute(rb_sql)
        println("Commit failed, rolled back: {e.message}")
    },
}
```

---

## vaisdb Hybrid Queries

### Vector Search (VECTOR_SEARCH)

```vais
# Find top-10 similar documents using an embedding vector
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

### Graph Traversal (GRAPH_TRAVERSE)

```vais
# Traverse from node "user-42" in the outbound direction up to depth 3
sql := QueryBuilder.new()
    .column("id")
    .column("label")
    .graph_traverse("user-42", 3, "outbound")
    .build()
# SELECT id, label FROM GRAPH_TRAVERSE('user-42', 3, 'outbound')
```

Direction options: `"outbound"`, `"inbound"`, `"any"`

### Full-Text Search (FULLTEXT_MATCH)

```vais
sql := QueryBuilder.new()
    .select("articles")
    .column("id")
    .column("title")
    .column("body")
    .fulltext_match("body", "vais-server routing")
    .limit(20)
    .build()
# SELECT id, title, body FROM articles
#   WHERE FULLTEXT_MATCH(body, 'vais-server routing') LIMIT 20
```

---

## Migrations (Migrator)

`Migrator` manages version-based schema migrations. Internally, it tracks applied migrations using the `__vaisdb_migrations` table.

### Defining and Running Migrations

```vais
U db/connection
U db/migrate

F run_migrations(conn: DbConnection) -> Result<i64, VaisDbError> {
    migrator_result := Migrator.new(conn)
    M migrator_result {
        Err(e) => { R Err(e) },
        Ok(mut migrator) => {
            # Version 1 — create users table
            m1 := Migration.new(
                1,
                "create_users",
                "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT UNIQUE NOT NULL, created_at TEXT NOT NULL)",
                "DROP TABLE IF EXISTS users"
            )
            migrator.add_migration(m1)

            # Version 2 — create posts table
            m2 := Migration.new(
                2,
                "create_posts",
                "CREATE TABLE IF NOT EXISTS posts (id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, title TEXT NOT NULL, body TEXT, published INTEGER DEFAULT 0)",
                "DROP TABLE IF EXISTS posts"
            )
            migrator.add_migration(m2)

            # Version 3 — add index on posts
            m3 := Migration.new(
                3,
                "add_posts_user_index",
                "CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id)",
                "DROP INDEX IF EXISTS idx_posts_user_id"
            )
            migrator.add_migration(m3)

            # Run pending migrations in order
            migrator.run_up()
        },
    }
}
```

### Rollback

```vais
# Roll back to version 1 (versions 2 and 3 are run in reverse order)
M migrator.run_down(1) {
    Ok(count) => { println("{count} migration(s) rolled back") },
    Err(e)    => { println("Rollback failed: {e.message}") },
}
```

### Migration Structure

```vais
m := Migration.new(
    version,   # i64 — monotonically increasing version number
    name,      # str — migration name (snake_case recommended)
    up_sql,    # str — SQL to apply
    down_sql,  # str — SQL to roll back
)
```

---

## Full Integration Example

The following shows the complete flow: connecting to the database and running migrations at server startup, then querying data with QueryBuilder inside a handler.

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

    # In a real implementation, query the DB with conn.execute(sql)
    println("  [db] {sql}")

    pairs := Vec.new()
    pairs.push("id")
    pairs.push(id)
    pairs.push("name")
    pairs.push("Alice")
    ctx.json(200, json_encode(pairs))
}

F main() -> i64 {
    # 1. Connect to DB
    db_config := DbConfig.embedded(DB_PATH)
    db := M DbConnection.connect(db_config) {
        Err(e) => {
            println("DB connection failed: {e.message}")
            R 1
        },
        Ok(conn) => { conn },
    }

    # 2. Run migrations
    M Migrator.new(db) {
        Err(e) => {
            println("Migrator init failed: {e.message}")
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
                Ok(count) => { println("{count} migration(s) applied") },
                Err(e)    => { println("Migration failed: {e.message}") R 1 },
            }
        },
    }

    # 3. Configure HTTP server
    config := ServerConfig.default()
    app    := mut App.new(config)

    app.use("logger")
    app.use("cors")
    app.get("/users/:id", "handle_get_user")

    println("Server starting: :{PORT}")
    M app.listen(":{PORT}") {
        Ok(_) => {},
        Err(e) => { println("Server error: {e.message}") R 1 },
    }
    0
}
```

---

## QueryKind Reference

| `QueryKind` | Builder Method | Example Output |
|-------------|---------------|----------------|
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

## Next Steps

- [vaisdb Documentation](../vaisdb/README.md) — Full features and schema design of the vaisdb database engine
- [Routing Guide](./routing.md) — Patterns for returning QueryBuilder results as JSON responses from handlers
