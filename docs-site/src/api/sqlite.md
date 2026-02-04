# SQLite API Reference

> SQLite3 database bindings for Vais

**Dependencies:** `-lsqlite3`

## Import

```vais
U std/sqlite
```

## Constants

### Result Codes

| Name | Value | Description |
|------|-------|-------------|
| `SQLITE_OK` | 0 | Success |
| `SQLITE_ERROR` | 1 | Generic error |
| `SQLITE_INTERNAL` | 2 | Internal logic error |
| `SQLITE_PERM` | 3 | Access permission denied |
| `SQLITE_ABORT` | 4 | Callback requested abort |
| `SQLITE_BUSY` | 5 | Database file is locked |
| `SQLITE_LOCKED` | 6 | A table in the database is locked |
| `SQLITE_NOMEM` | 7 | Memory allocation failed |
| `SQLITE_READONLY` | 8 | Attempt to write to readonly database |
| `SQLITE_INTERRUPT` | 9 | Operation terminated |
| `SQLITE_IOERR` | 10 | Disk I/O error |
| `SQLITE_CORRUPT` | 11 | Database disk image is malformed |
| `SQLITE_NOTFOUND` | 12 | Unknown operation |
| `SQLITE_FULL` | 13 | Database is full |
| `SQLITE_CANTOPEN` | 14 | Unable to open database file |
| `SQLITE_PROTOCOL` | 15 | Database lock protocol error |
| `SQLITE_CONSTRAINT` | 19 | Constraint violation |
| `SQLITE_MISMATCH` | 20 | Data type mismatch |
| `SQLITE_MISUSE` | 21 | Library used incorrectly |
| `SQLITE_ROW` | 100 | Row available from step() |
| `SQLITE_DONE` | 101 | Statement execution complete |

### Column Type Codes

| Name | Value | Description |
|------|-------|-------------|
| `SQLITE_INTEGER` | 1 | Integer column type |
| `SQLITE_FLOAT` | 2 | Float column type |
| `SQLITE_TEXT` | 3 | Text column type |
| `SQLITE_BLOB` | 4 | Blob column type |
| `SQLITE_NULL` | 5 | NULL column type |

## Structs

### Database

Database connection handle.

| Field | Type | Description |
|-------|------|-------------|
| `handle` | `i64` | Internal SQLite handle |
| `path` | `str` | Database file path |
| `is_open` | `i64` | 1 if open, 0 if closed |

#### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `open` | `F open(path: str) -> Database` | Open database file (creates if not exists) |
| `memory` | `F memory() -> Database` | Open in-memory database |
| `is_valid` | `F is_valid(&self) -> i64` | Check if database is open (returns 1/0) |
| `close` | `F close(&self) -> i64` | Close connection, returns result code |
| `exec` | `F exec(&self, sql: str) -> i64` | Execute SQL statement, returns result code |
| `prepare` | `F prepare(&self, sql: str) -> Statement` | Prepare SQL statement for execution |
| `error_message` | `F error_message(&self) -> str` | Get last error message |
| `last_insert_id` | `F last_insert_id(&self) -> i64` | Get rowid of last inserted row |
| `changes` | `F changes(&self) -> i64` | Get number of rows changed by last statement |
| `begin` | `F begin(&self) -> i64` | Begin transaction |
| `commit` | `F commit(&self) -> i64` | Commit transaction |
| `rollback` | `F rollback(&self) -> i64` | Rollback transaction |
| `begin_immediate` | `F begin_immediate(&self) -> i64` | Begin immediate transaction (acquires write lock) |
| `create_table` | `F create_table(&self, sql: str) -> i64` | Create table from SQL statement |
| `drop_table` | `F drop_table(&self, table_name: str) -> i64` | Drop table if exists |
| `enable_wal` | `F enable_wal(&self) -> i64` | Enable Write-Ahead Logging mode |
| `enable_foreign_keys` | `F enable_foreign_keys(&self) -> i64` | Enable foreign key enforcement |
| `drop` | `F drop(&self) -> i64` | RAII cleanup (calls close) |

### Statement

Prepared statement handle.

| Field | Type | Description |
|-------|------|-------------|
| `handle` | `i64` | Internal statement handle |
| `db_handle` | `i64` | Parent database handle |
| `column_count` | `i64` | Number of columns in result set |

#### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `is_valid` | `F is_valid(&self) -> i64` | Check if statement is valid (returns 1/0) |
| `bind_int` | `F bind_int(&self, index: i64, value: i64) -> i64` | Bind integer parameter (1-indexed) |
| `bind_text` | `F bind_text(&self, index: i64, value: str) -> i64` | Bind text parameter (1-indexed) |
| `bind_double` | `F bind_double(&self, index: i64, value: i64) -> i64` | Bind double parameter (1-indexed, value as i64 bits) |
| `bind_null` | `F bind_null(&self, index: i64) -> i64` | Bind NULL parameter (1-indexed) |
| `step` | `F step(&self) -> i64` | Execute one step, returns SQLITE_ROW/SQLITE_DONE/error |
| `column_int` | `F column_int(&self, index: i64) -> i64` | Get integer column value (0-indexed) |
| `column_text` | `F column_text(&self, index: i64) -> str` | Get text column value (0-indexed) |
| `column_double` | `F column_double(&self, index: i64) -> i64` | Get double column as i64 bits (0-indexed) |
| `column_type` | `F column_type(&self, index: i64) -> i64` | Get column type (0-indexed) |
| `column_name` | `F column_name(&self, index: i64) -> str` | Get column name (0-indexed) |
| `columns` | `F columns(&self) -> i64` | Get number of columns |
| `reset` | `F reset(&self) -> i64` | Reset for re-execution (bindings not cleared) |
| `finalize` | `F finalize(&self) -> i64` | Finalize and destroy statement |
| `execute` | `F execute(&self) -> i64` | Execute to completion (no results expected) |
| `drop` | `F drop(&self) -> i64` | RAII cleanup (calls finalize) |

### Row

Convenience wrapper for result rows during iteration.

| Field | Type | Description |
|-------|------|-------------|
| `stmt_handle` | `i64` | Internal statement handle |
| `column_count` | `i64` | Number of columns |

#### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `from_stmt` | `F from_stmt(stmt: &Statement) -> Row` | Create Row from Statement |
| `get_int` | `F get_int(&self, index: i64) -> i64` | Get integer column value (0-indexed) |
| `get_text` | `F get_text(&self, index: i64) -> str` | Get text column value (0-indexed) |
| `get_double` | `F get_double(&self, index: i64) -> i64` | Get double column as i64 bits (0-indexed) |
| `get_type` | `F get_type(&self, index: i64) -> i64` | Get column type (0-indexed) |
| `get_name` | `F get_name(&self, index: i64) -> str` | Get column name (0-indexed) |
| `is_null` | `F is_null(&self, index: i64) -> i64` | Check if column is NULL (0-indexed) |
| `columns` | `F columns(&self) -> i64` | Get column count |

## Convenience Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `open` | `F open(path: str) -> Database` | Open database file |
| `memory` | `F memory() -> Database` | Open in-memory database |
| `exec` | `F exec(db: &Database, sql: str) -> i64` | Execute SQL statement |
| `result_code_str` | `F result_code_str(code: i64) -> str` | Convert result code to string |
| `is_ok` | `F is_ok(code: i64) -> i64` | Check if result code indicates success |
| `has_row` | `F has_row(code: i64) -> i64` | Check if step result indicates row available |
| `is_done` | `F is_done(code: i64) -> i64` | Check if step result indicates completion |

## Usage Examples

### Basic Usage

```vais
U std/sqlite

F main() -> i64 {
    db := Database::open("test.db")
    I db.is_valid() == 0 {
        R 1
    }

    db.exec("CREATE TABLE users (id INTEGER, name TEXT)")
    db.exec("INSERT INTO users VALUES (1, 'Alice')")

    stmt := db.prepare("SELECT name FROM users WHERE id = ?")
    stmt.bind_int(1, 1)

    I stmt.step() == SQLITE_ROW {
        name := stmt.column_text(0)
        # Use name...
    }

    stmt.finalize()
    db.close()
    0
}
```

### Query Iteration with Row

```vais
U std/sqlite

F main() -> i64 {
    db := Database::open("users.db")
    stmt := db.prepare("SELECT id, name FROM users")

    L stmt.step() == SQLITE_ROW {
        row := Row::from_stmt(&stmt)
        id := row.get_int(0)
        name := row.get_text(1)
        # Process row...
    }

    stmt.finalize()
    db.close()
    0
}
```

### Transaction Management

```vais
U std/sqlite

F main() -> i64 {
    db := Database::open("data.db")

    db.begin()
    rc := db.exec("INSERT INTO accounts VALUES (1, 100)")
    I rc != SQLITE_OK {
        db.rollback()
        R 1
    }

    rc = db.exec("UPDATE accounts SET balance = balance - 100 WHERE id = 2")
    I rc != SQLITE_OK {
        db.rollback()
        R 1
    }

    db.commit()
    db.close()
    0
}
```

### In-Memory Database

```vais
U std/sqlite

F main() -> i64 {
    db := Database::memory()
    db.exec("CREATE TABLE temp (id INTEGER)")
    db.exec("INSERT INTO temp VALUES (42)")

    stmt := db.prepare("SELECT id FROM temp")
    I stmt.step() == SQLITE_ROW {
        value := stmt.column_int(0)
        # value is 42
    }

    stmt.finalize()
    db.close()
    0
}
```

### Error Handling

```vais
U std/sqlite

F main() -> i64 {
    db := Database::open("test.db")
    rc := db.exec("INVALID SQL")

    I rc != SQLITE_OK {
        error_msg := db.error_message()
        code_name := result_code_str(rc)
        # Handle error...
        R 1
    }

    db.close()
    0
}
```

### Convenience Functions

```vais
U std/sqlite

F main() -> i64 {
    # Using convenience functions
    db := open("data.db")
    rc := exec(&db, "CREATE TABLE test (id INTEGER)")

    I is_ok(rc) {
        # Success
    }

    stmt := db.prepare("SELECT * FROM test")
    step_result := stmt.step()

    I has_row(step_result) {
        # Process row
    }

    I is_done(step_result) {
        # Query complete
    }

    stmt.finalize()
    db.close()
    0
}
```

## Compilation

To compile programs using SQLite:

```bash
vaisc --emit-ir your_app.vais
clang -o your_app your_app.ll std/sqlite_runtime.c -lsqlite3
```

Requires SQLite3 development headers and library installed on your system.
