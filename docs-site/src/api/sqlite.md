# SQLite API Reference

> SQLite3 database bindings for Vais

**Dependencies:** `-lsqlite3`

## Import

```vais
U std/sqlite
```

## Structs

### SqliteDb

Database connection handle.

| Method | Signature | Description |
|--------|-----------|-------------|
| `open` | `F open(path: i64) -> SqliteDb` | Open database file |
| `close` | `F close(&self) -> i64` | Close connection |
| `exec` | `F exec(&self, sql: i64) -> i64` | Execute SQL statement |
| `prepare` | `F prepare(&self, sql: i64) -> SqliteStmt` | Prepare statement |
| `last_error` | `F last_error(&self) -> i64` | Get last error message |

### SqliteStmt

Prepared statement handle.

| Method | Signature | Description |
|--------|-----------|-------------|
| `step` | `F step(&self) -> i64` | Execute/advance to next row |
| `reset` | `F reset(&self) -> i64` | Reset for re-execution |
| `bind_int` | `F bind_int(&self, idx: i64, value: i64) -> i64` | Bind integer parameter |
| `bind_text` | `F bind_text(&self, idx: i64, value: i64) -> i64` | Bind text parameter |
| `column_int` | `F column_int(&self, idx: i64) -> i64` | Get integer column |
| `column_text` | `F column_text(&self, idx: i64) -> i64` | Get text column |
| `finalize` | `F finalize(&self) -> i64` | Finalize statement |

## Constants

| Name | Value | Description |
|------|-------|-------------|
| `SQLITE_OK` | 0 | Success |
| `SQLITE_ROW` | 100 | Row available |
| `SQLITE_DONE` | 101 | No more rows |

## Usage

```vais
U std/sqlite

F main() -> i64 {
    db := SqliteDb.open("test.db")
    db.exec("CREATE TABLE users (id INTEGER, name TEXT)")
    db.exec("INSERT INTO users VALUES (1, 'Alice')")
    stmt := db.prepare("SELECT name FROM users WHERE id = ?")
    stmt.bind_int(1, 1)
    I stmt.step() == SQLITE_ROW {
        name := stmt.column_text(0)
    }
    stmt.finalize()
    db.close()
    0
}
```
