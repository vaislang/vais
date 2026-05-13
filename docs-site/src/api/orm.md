# ORM API Reference

> Lightweight object-relational mapping with SQL query builder

> **Implementation:** Requires C runtime (`orm_runtime.c`). Uses SQLite or PostgreSQL as backend; the corresponding database library (`-lsqlite3` or `-lpq`) must be linked.

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
use std/orm
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
struct Column {
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
fn new(name: str, col_type: i64) -> Column
```

Create a new column with specified name and type.

#### integer / text / real / blob / boolean

```vais
fn integer(name: str) -> Column
fn text(name: str) -> Column
fn real(name: str) -> Column
fn blob(name: str) -> Column
fn boolean(name: str) -> Column
```

Create columns of specific types.

#### type_str

```vais
fn type_str(&self) -> str
```

Get SQL type string for this column type.

#### write_def

```vais
fn write_def(&self, buf: i64, pos: i64) -> i64
```

Write column definition SQL to buffer at position, returns new position.

## Schema

### Schema Struct

```vais
struct Schema {
    table_name: str,
    columns: i64,        # Pointer to array of Column data
    column_count: i64
}
```

### Schema Methods

#### new

```vais
fn new(table_name: str) -> Schema
```

Create a new schema for a table.

#### add_column

```vais
fn add_column(&self, name: str, col_type: i64) -> Schema
```

Add a column to the schema. Returns self for chaining.

#### primary_key

```vais
fn primary_key(&self) -> Schema
```

Mark the last added column as primary key. Returns self for chaining.

#### nullable

```vais
fn nullable(&self) -> Schema
```

Mark the last added column as nullable. Returns self for chaining.

#### with_default

```vais
fn with_default(&self, val: str) -> Schema
```

Set default value on the last added column. Returns self for chaining.

#### col_type_str

```vais
fn col_type_str(col_type: i64) -> str
```

Get column type string from type ID (static method).

#### create_table

```vais
fn create_table(&self) -> str
```

Generate CREATE TABLE SQL statement.

#### drop_table

```vais
fn drop_table(&self) -> str
```

Generate DROP TABLE SQL statement.

#### len

```vais
fn len(&self) -> i64
```

Get column count.

#### drop

```vais
fn drop(&self) -> i64
```

Free memory.

## WhereClause

### WhereClause Struct

```vais
struct WhereClause {
    items: i64,        # Pointer to array: [connector, column, operator, value]
    count: i64,
    capacity: i64
}
```

### WhereClause Methods

#### new

```vais
fn new() -> WhereClause
```

Create a new WHERE clause builder.

#### add

```vais
fn add(&self, connector: str, column: str, operator: str, value: str) -> i64
```

Add a WHERE condition. `connector` is "AND", "OR", or "" (for first). Returns 0 on success, -1 on capacity exceeded.

#### write_to

```vais
fn write_to(&self, buf: i64, pos: i64) -> i64
```

Write WHERE clause to buffer, returns new position.

#### drop

```vais
fn drop(&self) -> i64
```

Free memory.

## QueryBuilder

### QueryBuilder Struct

```vais
struct QueryBuilder {
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
fn new() -> QueryBuilder
```

Create a new query builder.

#### select

```vais
fn select(cols: str) -> QueryBuilder
```

Create a SELECT query with specified columns (e.g., "id, name, age" or "*").

#### from

```vais
fn from(&self, table: str) -> QueryBuilder
```

Set the FROM table. Returns self for chaining.

#### where_eq

```vais
fn where_eq(&self, column: str, value: str) -> QueryBuilder
```

Add WHERE column = value. Returns self for chaining.

#### where_gt

```vais
fn where_gt(&self, column: str, value: str) -> QueryBuilder
```

Add WHERE column > value. Returns self for chaining.

#### where_lt

```vais
fn where_lt(&self, column: str, value: str) -> QueryBuilder
```

Add WHERE column < value. Returns self for chaining.

#### and_eq

```vais
fn and_eq(&self, column: str, value: str) -> QueryBuilder
```

Add AND column = value. Returns self for chaining.

#### or_eq

```vais
fn or_eq(&self, column: str, value: str) -> QueryBuilder
```

Add OR column = value. Returns self for chaining.

#### order_by

```vais
fn order_by(&self, column: str, direction: str) -> QueryBuilder
```

Add ORDER BY clause. Direction is "ASC" or "DESC". Returns self for chaining.

#### limit

```vais
fn limit(&self, n: i64) -> QueryBuilder
```

Add LIMIT clause. Returns self for chaining.

#### offset

```vais
fn offset(&self, n: i64) -> QueryBuilder
```

Add OFFSET clause. Returns self for chaining.

#### insert

```vais
fn insert(table: str, cols: str, vals: str) -> QueryBuilder
```

Create an INSERT query. `cols` is comma-separated column names, `vals` is comma-separated values.

#### update

```vais
fn update(table: str, set_clause: str) -> QueryBuilder
```

Create an UPDATE query. `set_clause` is like "name = 'Alice', age = 30".

#### delete

```vais
fn delete(table: str) -> QueryBuilder
```

Create a DELETE query.

#### build

```vais
fn build(&self) -> str
```

Build the final SQL string.

#### drop

```vais
fn drop(&self) -> i64
```

Free memory.

## Migration

### Migration Struct

```vais
struct Migration {
    version: i64,
    name: str,
    up_sql: str,
    down_sql: str
}
```

### Migration Methods

#### new

```vais
fn new(version: i64, name: str, up_sql: str, down_sql: str) -> Migration
```

Create a new migration.

#### migrate_up

```vais
fn migrate_up(&self) -> str
```

Get the up migration SQL.

#### migrate_down

```vais
fn migrate_down(&self) -> str
```

Get the down migration SQL.

## MigrationRunner

### MigrationRunner Struct

```vais
struct MigrationRunner {
    migrations: i64,     # Pointer to array of Migration pointers
    count: i64,
    capacity: i64
}
```

### MigrationRunner Methods

#### new

```vais
fn new() -> MigrationRunner
```

Create a new migration runner.

#### add

```vais
fn add(&self, migration: Migration) -> MigrationRunner
```

Add a migration. Returns self for chaining.

#### migrate_up_all

```vais
fn migrate_up_all(&self) -> str
```

Get all up-migration SQL statements concatenated.

#### migrate_down_all

```vais
fn migrate_down_all(&self) -> str
```

Get all down-migration SQL statements (in reverse order).

#### len

```vais
fn len(&self) -> i64
```

Get migration count.

#### drop

```vais
fn drop(&self) -> i64
```

Free memory.

## Convenience Functions

```vais
fn schema(table_name: str) -> Schema
```

Create a new schema for a table.

```vais
fn select_from(columns: str, table: str) -> QueryBuilder
```

Create a SELECT query builder.

```vais
fn insert_into(table: str, cols: str, vals: str) -> QueryBuilder
```

Create an INSERT query builder.

```vais
fn update_table(table: str, set_clause: str) -> QueryBuilder
```

Create an UPDATE query builder.

```vais
fn delete_from(table: str) -> QueryBuilder
```

Create a DELETE query builder.

```vais
fn migration(version: i64, name: str, up_sql: str, down_sql: str) -> Migration
```

Create a new migration.

```vais
fn migration_runner() -> MigrationRunner
```

Create a new migration runner.

## Usage Examples

### Define Schema and Create Table

```vais
use std/orm

fn create_users_table() -> str {
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
use std/orm

fn find_active_users() -> str {
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
use std/orm

fn insert_user(name: str, email: str, age: i64) -> str {
    q := insert_into("users", "name, email, age", "Alice, alice@example.com, 30")

    sql := q.build()
    # sql = "INSERT INTO users (name, email, age) VALUES (Alice, alice@example.com, 30);"

    q.drop()
    sql
}
```

### UPDATE Query

```vais
use std/orm

fn update_user_email(user_id: i64, new_email: str) -> str {
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
use std/orm

fn delete_inactive_users() -> str {
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
use std/orm

fn find_users_complex() -> str {
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
use std/orm

fn setup_migrations() -> MigrationRunner {
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

fn run_migrations() -> i64 {
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
use std/orm
use std/postgres

fn main() -> i64 {
    # Connect to database
    conn := pg_connect("localhost", 5432, "mydb", "user", "pass")

    I conn.is_connected() == 0 {
        return -1
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
use std/orm

fn drop_users_table() -> str {
    s := schema("users")
    sql := s.drop_table()
    # sql = "DROP TABLE IF EXISTS users;"
    s.drop()
    sql
}
```
