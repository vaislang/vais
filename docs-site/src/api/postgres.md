# Postgres API Reference

> PostgreSQL client library built on libpq

**Dependencies:** `-lpq`

## Import

```vais
U std/postgres
```

## Features

- Connection management (connect/disconnect)
- Simple and parameterized queries
- Prepared statements
- Transactions (begin/commit/rollback)
- Row/column access (int, text, float, bool)
- Connection string building

## Constants

| Name | Value | Description |
|------|-------|-------------|
| `PG_CONNECTION_OK` | 0 | Connection successful |
| `PG_CONNECTION_BAD` | 1 | Connection failed |
| `PG_RESULT_TUPLES_OK` | 2 | Query returned rows |
| `PG_RESULT_COMMAND_OK` | 1 | Command executed |

## Key Functions

| Function | Description |
|----------|-------------|
| `pg_connect(conninfo)` | Connect to database |
| `pg_disconnect(conn)` | Disconnect |
| `pg_exec(conn, sql)` | Execute query |
| `pg_prepare(conn, name, sql)` | Prepare statement |
| `pg_exec_prepared(conn, name, params)` | Execute prepared |
| `pg_ntuples(result)` | Number of rows |
| `pg_nfields(result)` | Number of columns |
| `pg_get_value(result, row, col)` | Get value as text |
| `pg_begin(conn)` | Begin transaction |
| `pg_commit(conn)` | Commit transaction |
| `pg_rollback(conn)` | Rollback transaction |

## Usage

```vais
U std/postgres

F main() -> i64 {
    conn := pg_connect("host=localhost dbname=mydb")
    result := pg_exec(conn, "SELECT * FROM users")
    rows := pg_ntuples(result)
    pg_disconnect(conn)
    0
}
```
