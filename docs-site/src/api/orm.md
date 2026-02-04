# ORM API Reference

> Lightweight object-relational mapping with SQL query builder

## Import

```vais
U std/orm
```

## Features

- Schema/Column definition with types, primary keys, defaults
- Fluent QueryBuilder for SELECT, INSERT, UPDATE, DELETE
- SQL injection prevention via escaping
- Migration up/down support

## Column Types

| Constant | Value | SQL Type |
|----------|-------|----------|
| `COL_INTEGER` | 1 | INTEGER |
| `COL_TEXT` | 2 | TEXT |
| `COL_REAL` | 3 | REAL |
| `COL_BLOB` | 4 | BLOB |
| `COL_BOOLEAN` | 5 | BOOLEAN |

## Query Types

| Constant | Value |
|----------|-------|
| `QUERY_SELECT` | 1 |
| `QUERY_INSERT` | 2 |
| `QUERY_UPDATE` | 3 |
| `QUERY_DELETE` | 4 |

## Key Functions

| Function | Description |
|----------|-------------|
| `schema_new(name)` | Create table schema |
| `schema_add_column(schema, name, type)` | Add column |
| `schema_to_create_sql(schema)` | Generate CREATE TABLE SQL |
| `query_select(table)` | Start SELECT query |
| `query_insert(table)` | Start INSERT query |
| `query_where(query, condition)` | Add WHERE clause |
| `query_build(query)` | Build SQL string |

## Usage

```vais
U std/orm

F main() -> i64 {
    q := query_select("users")
    q := query_where(q, "age > 18")
    sql := query_build(q)
    # sql = "SELECT * FROM users WHERE age > 18"
    0
}
```
