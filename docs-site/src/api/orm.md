# ORM API Reference

> Lightweight object-relational mapping with SQL query builder

## Overview

The ORM module provides a lightweight ORM for Vais with:
- Schema and column definition with types, constraints, and defaults
- Fluent QueryBuilder for SELECT, INSERT, UPDATE, DELETE
- SQL injection prevention via runtime escaping
- WHERE clause building with AND/OR operators
- ORDER BY, LIMIT, OFFSET support
- Migration up/down support for schema versioning
- Compatible with SQLite and PostgreSQL

## Import

```vais
U std/orm
```

## Constants

### Column Types

| Constant | Value | SQL Type | Description |
|----------|-------|----------|-------------|
| `COL_INTEGER` | 1 | INTEGER | Integer column |
| `COL_TEXT` | 2 | TEXT | Text/string column |
| `COL_REAL` | 3 | REAL | Floating point column |
| `COL_BLOB` | 4 | BLOB | Binary data column |
| `COL_BOOLEAN` | 5 | BOOLEAN | Boolean column |

### Query Types

| Constant | Value | Description |
|----------|-------|-------------|
| `QUERY_SELECT` | 1 | SELECT query |
| `QUERY_INSERT` | 2 | INSERT query |
| `QUERY_UPDATE` | 3 | UPDATE query |
| `QUERY_DELETE` | 4 | DELETE query |

### Buffer Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `SQL_BUFFER_SIZE` | 4096 | SQL query buffer size |
| `SQL_MAX_COLUMNS` | 64 | Maximum columns per table |
| `SQL_MAX_WHERE` | 32 | Maximum WHERE clauses |
| `SQL_MAX_VALUES` | 64 | Maximum values in INSERT |

## Column

### Column Struct

```vais
S Column {
    name: str,
    col_type: i64,
    is_primary: i64,
    is_nullable: i64,
    default_value: str
}
```

### Column Methods

#### new

```vais
F new(name: str, col_type: i64) -> Column
```

Create a new column with specified name and type.

#### integer / text / real / blob / boolean

```vais
F integer(name: str) -> Column
F text(name: str) -> Column
F real(name: str) -> Column
F blob(name: str) -> Column
F boolean(name: str) -> Column
```

Create columns of specific types.

#### type_str

```vais
F type_str(&self) -> str
```

Get SQL type string for this column type.

#### write_def

```vais
F write_def(&self, buf: i64, pos: i64) -> i64
```

Write column definition SQL to buffer at position, returns new position.

## Schema

### Schema Struct

```vais
S Schema {
    table_name: str,
    columns: i64,        # Pointer to array of Column data
    column_count: i64
}
```

### Schema Methods

#### new

```vais
F new(table_name: str) -> Schema
```

Create a new schema for a table.

#### add_column

```vais
F add_column(&self, name: str, col_type: i64) -> Schema
```

Add a column to the schema. Returns self for chaining.

#### primary_key

```vais
F primary_key(&self) -> Schema
```

Mark the last added column as primary key. Returns self for chaining.

#### nullable

```vais
F nullable(&self) -> Schema
```

Mark the last added column as nullable. Returns self for chaining.

#### with_default

```vais
F with_default(&self, val: str) -> Schema
```

Set default value on the last added column. Returns self for chaining.

#### col_type_str

```vais
F col_type_str(col_type: i64) -> str
```

Get column type string from type ID (static method).

#### create_table

```vais
F create_table(&self) -> str
```

Generate CREATE TABLE SQL statement.

#### drop_table

```vais
F drop_table(&self) -> str
```

Generate DROP TABLE SQL statement.

#### len

```vais
F len(&self) -> i64
```

Get column count.

#### drop

```vais
F drop(&self) -> i64
```

Free memory.

## WhereClause

### WhereClause Struct

```vais
S WhereClause {
    items: i64,        # Pointer to array: [connector, column, operator, value]
    count: i64,
    capacity: i64
}
```

### WhereClause Methods

#### new

```vais
F new() -> WhereClause
```

Create a new WHERE clause builder.

#### add

```vais
F add(&self, connector: str, column: str, operator: str, value: str) -> i64
```

Add a WHERE condition. `connector` is "AND", "OR", or "" (for first). Returns 0 on success, -1 on capacity exceeded.

#### write_to

```vais
F write_to(&self, buf: i64, pos: i64) -> i64
```

Write WHERE clause to buffer, returns new position.

#### drop

```vais
F drop(&self) -> i64
```

Free memory.

## QueryBuilder

### QueryBuilder Struct

```vais
S QueryBuilder {
    query_type: i64,
    table: str,
    columns: i64,        # Pointer to column name array
    column_count: i64,
    values: i64,         # Pointer to value array (for INSERT/UPDATE)
    value_count: i64,
    where_clause: WhereClause,
    order_col: str,
    order_dir: str,
    limit_val: i64,
    offset_val: i64
}
```

### QueryBuilder Methods

#### new

```vais
F new() -> QueryBuilder
```

Create a new query builder.

#### select

```vais
F select(cols: str) -> QueryBuilder
```

Create a SELECT query with specified columns (e.g., "id, name, age" or "*").

#### from

```vais
F from(&self, table: str) -> QueryBuilder
```

Set the FROM table. Returns self for chaining.

#### where_eq

```vais
F where_eq(&self, column: str, value: str) -> QueryBuilder
```

Add WHERE column = value. Returns self for chaining.

#### where_gt

```vais
F where_gt(&self, column: str, value: str) -> QueryBuilder
```

Add WHERE column > value. Returns self for chaining.

#### where_lt

```vais
F where_lt(&self, column: str, value: str) -> QueryBuilder
```

Add WHERE column < value. Returns self for chaining.

#### and_eq

```vais
F and_eq(&self, column: str, value: str) -> QueryBuilder
```

Add AND column = value. Returns self for chaining.

#### or_eq

```vais
F or_eq(&self, column: str, value: str) -> QueryBuilder
```

Add OR column = value. Returns self for chaining.

#### order_by

```vais
F order_by(&self, column: str, direction: str) -> QueryBuilder
```

Add ORDER BY clause. Direction is "ASC" or "DESC". Returns self for chaining.

#### limit

```vais
F limit(&self, n: i64) -> QueryBuilder
```

Add LIMIT clause. Returns self for chaining.

#### offset

```vais
F offset(&self, n: i64) -> QueryBuilder
```

Add OFFSET clause. Returns self for chaining.

#### insert

```vais
F insert(table: str, cols: str, vals: str) -> QueryBuilder
```

Create an INSERT query. `cols` is comma-separated column names, `vals` is comma-separated values.

#### update

```vais
F update(table: str, set_clause: str) -> QueryBuilder
```

Create an UPDATE query. `set_clause` is like "name = 'Alice', age = 30".

#### delete

```vais
F delete(table: str) -> QueryBuilder
```

Create a DELETE query.

#### build

```vais
F build(&self) -> str
```

Build the final SQL string.

#### drop

```vais
F drop(&self) -> i64
```

Free memory.

## Migration

### Migration Struct

```vais
S Migration {
    version: i64,
    name: str,
    up_sql: str,
    down_sql: str
}
```

### Migration Methods

#### new

```vais
F new(version: i64, name: str, up_sql: str, down_sql: str) -> Migration
```

Create a new migration.

#### migrate_up

```vais
F migrate_up(&self) -> str
```

Get the up migration SQL.

#### migrate_down

```vais
F migrate_down(&self) -> str
```

Get the down migration SQL.

## MigrationRunner

### MigrationRunner Struct

```vais
S MigrationRunner {
    migrations: i64,     # Pointer to array of Migration pointers
    count: i64,
    capacity: i64
}
```

### MigrationRunner Methods

#### new

```vais
F new() -> MigrationRunner
```

Create a new migration runner.

#### add

```vais
F add(&self, migration: Migration) -> MigrationRunner
```

Add a migration. Returns self for chaining.

#### migrate_up_all

```vais
F migrate_up_all(&self) -> str
```

Get all up-migration SQL statements concatenated.

#### migrate_down_all

```vais
F migrate_down_all(&self) -> str
```

Get all down-migration SQL statements (in reverse order).

#### len

```vais
F len(&self) -> i64
```

Get migration count.

#### drop

```vais
F drop(&self) -> i64
```

Free memory.

## Convenience Functions

```vais
F schema(table_name: str) -> Schema
```

Create a new schema for a table.

```vais
F select_from(columns: str, table: str) -> QueryBuilder
```

Create a SELECT query builder.

```vais
F insert_into(table: str, cols: str, vals: str) -> QueryBuilder
```

Create an INSERT query builder.

```vais
F update_table(table: str, set_clause: str) -> QueryBuilder
```

Create an UPDATE query builder.

```vais
F delete_from(table: str) -> QueryBuilder
```

Create a DELETE query builder.

```vais
F migration(version: i64, name: str, up_sql: str, down_sql: str) -> Migration
```

Create a new migration.

```vais
F migration_runner() -> MigrationRunner
```

Create a new migration runner.

## Usage Examples

### Define Schema and Create Table

```vais
U std/orm

F create_users_table() -> str {
    s := schema("users")
        .add_column("id", COL_INTEGER)
        .primary_key()
        .add_column("name", COL_TEXT)
        .add_column("email", COL_TEXT)
        .add_column("age", COL_INTEGER)
        .nullable()
        .add_column("active", COL_BOOLEAN)
        .with_default("1")

    sql := s.create_table()
    # sql = "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL, age INTEGER, active BOOLEAN NOT NULL DEFAULT 1);"

    s.drop()
    sql
}
```

### SELECT Query

```vais
U std/orm

F find_active_users() -> str {
    q := select_from("id, name, email", "users")
        .where_eq("active", "1")
        .order_by("name", "ASC")
        .limit(10)

    sql := q.build()
    # sql = "SELECT id, name, email FROM users WHERE active = '1' ORDER BY name ASC LIMIT 10;"

    q.drop()
    sql
}
```

### INSERT Query

```vais
U std/orm

F insert_user(name: str, email: str, age: i64) -> str {
    q := insert_into("users", "name, email, age", "Alice, alice@example.com, 30")

    sql := q.build()
    # sql = "INSERT INTO users (name, email, age) VALUES (Alice, alice@example.com, 30);"

    q.drop()
    sql
}
```

### UPDATE Query

```vais
U std/orm

F update_user_email(user_id: i64, new_email: str) -> str {
    q := update_table("users", "email = 'newemail@example.com'")
        .where_eq("id", "42")

    sql := q.build()
    # sql = "UPDATE users SET email = 'newemail@example.com' WHERE id = '42';"

    q.drop()
    sql
}
```

### DELETE Query

```vais
U std/orm

F delete_inactive_users() -> str {
    q := delete_from("users")
        .where_eq("active", "0")

    sql := q.build()
    # sql = "DELETE FROM users WHERE active = '0';"

    q.drop()
    sql
}
```

### Complex WHERE Clauses

```vais
U std/orm

F find_users_complex() -> str {
    q := QueryBuilder::select("*")
        .from("users")
        .where_eq("active", "1")
        .and_eq("verified", "1")
        .where_gt("age", "18")
        .or_eq("role", "admin")
        .order_by("created_at", "DESC")
        .limit(20)
        .offset(10)

    sql := q.build()
    # sql = "SELECT * FROM users WHERE active = '1' AND verified = '1' AND age > '18' OR role = 'admin' ORDER BY created_at DESC LIMIT 20 OFFSET 10;"

    q.drop()
    sql
}
```

### Migrations

```vais
U std/orm

F setup_migrations() -> MigrationRunner {
    runner := migration_runner()

    # Migration 1: Create users table
    m1 := migration(
        1,
        "create_users",
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
        "DROP TABLE users;"
    )
    runner.add(m1)

    # Migration 2: Add email column
    m2 := migration(
        2,
        "add_email",
        "ALTER TABLE users ADD COLUMN email TEXT;",
        "ALTER TABLE users DROP COLUMN email;"
    )
    runner.add(m2)

    # Migration 3: Create posts table
    m3 := migration(
        3,
        "create_posts",
        "CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER, title TEXT, content TEXT);",
        "DROP TABLE posts;"
    )
    runner.add(m3)

    runner
}

F run_migrations() -> i64 {
    runner := setup_migrations()

    # Get all up migrations
    up_sql := runner.migrate_up_all()
    # Execute up_sql with your database connection...

    # Or get all down migrations (in reverse)
    down_sql := runner.migrate_down_all()
    # Execute down_sql to rollback...

    runner.drop()
    0
}
```

### Full Example with PostgreSQL

```vais
U std/orm
U std/postgres

F main() -> i64 {
    # Connect to database
    conn := pg_connect("localhost", 5432, "mydb", "user", "pass")

    I conn.is_connected() == 0 {
        R -1
    }

    # Create schema
    s := schema("products")
        .add_column("id", COL_INTEGER)
        .primary_key()
        .add_column("name", COL_TEXT)
        .add_column("price", COL_REAL)
        .add_column("stock", COL_INTEGER)
        .with_default("0")

    # Execute CREATE TABLE
    create_sql := s.create_table()
    conn.exec(create_sql)
    s.drop()

    # Insert data
    insert_sql := insert_into(
        "products",
        "name, price, stock",
        "Widget, 19.99, 100"
    ).build()
    conn.exec(insert_sql)

    # Query data
    select_sql := select_from("*", "products")
        .where_gt("stock", "0")
        .order_by("price", "ASC")
        .build()

    result := conn.query(select_sql)

    I result.is_ok() == 1 {
        i := 0
        L i < result.rows() {
            name := result.get_text(i, 1)
            price := result.get_float(i, 2)
            stock := result.get_int(i, 3)
            # Process data...
            i = i + 1
        }
    }

    result.clear()
    conn.disconnect()
    0
}
```

### Drop Table

```vais
U std/orm

F drop_users_table() -> str {
    s := schema("users")
    sql := s.drop_table()
    # sql = "DROP TABLE IF EXISTS users;"
    s.drop()
    sql
}
```
