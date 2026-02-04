# Postgres API Reference

> PostgreSQL client library built on libpq

**Dependencies:** `-lpq`

## Overview

The Postgres module provides a complete PostgreSQL database client with:
- Connection management with connection strings
- Simple SQL queries and parameterized queries
- Prepared statements for performance
- Transactions (begin/commit/rollback)
- Result set access with type conversion (int, text, float, bool)
- Connection status and error reporting
- Parameter builder helper for safe query construction

## Import

```vais
U std/postgres
```

## Compilation

Requires linking against libpq:

```bash
vaisc myapp.vais && clang myapp.ll std/postgres_runtime.c -lpq -o myapp
```

## Constants

### Connection Status

| Constant | Value | Description |
|----------|-------|-------------|
| `PG_CONNECTION_OK` | 0 | Connection successful |
| `PG_CONNECTION_BAD` | 1 | Connection failed |

### Result Status

| Constant | Value | Description |
|----------|-------|-------------|
| `PG_RESULT_EMPTY_QUERY` | 0 | Empty query string |
| `PG_RESULT_COMMAND_OK` | 1 | Command executed successfully (no rows) |
| `PG_RESULT_TUPLES_OK` | 2 | Query returned rows |
| `PG_RESULT_BAD_RESPONSE` | 5 | Server sent bad response |
| `PG_RESULT_FATAL_ERROR` | 7 | Fatal error occurred |

### Configuration

| Constant | Value | Description |
|----------|-------|-------------|
| `PG_DEFAULT_PORT` | 5432 | Default PostgreSQL port |
| `PG_CONNINFO_MAX` | 1024 | Maximum connection string size |
| `PG_MAX_PARAMS` | 64 | Maximum query parameters |

## PgResult

### PgResult Struct

```vais
S PgResult {
    handle: i64,       # PGresult* from libpq
    row_count: i64,    # Number of rows
    col_count: i64     # Number of columns
}
```

### PgResult Methods

#### from_handle

```vais
F from_handle(h: i64) -> PgResult
```

Wrap a raw libpq PGresult pointer.

#### is_valid

```vais
F is_valid(&self) -> i64
```

Check if the result is valid (not null). Returns 1 if valid, 0 otherwise.

#### is_ok

```vais
F is_ok(&self) -> i64
```

Check if the query was successful (COMMAND_OK or TUPLES_OK). Returns 1 if successful, 0 otherwise.

#### status

```vais
F status(&self) -> i64
```

Get the result status code (one of PG_RESULT_* constants).

#### rows

```vais
F rows(&self) -> i64
```

Get number of rows in the result set.

#### cols

```vais
F cols(&self) -> i64
```

Get number of columns in the result set.

#### get_text

```vais
F get_text(&self, row: i64, col: i64) -> str
```

Get a value as text (raw string). Returns empty string if out of bounds.

#### get_int

```vais
F get_int(&self, row: i64, col: i64) -> i64
```

Get a value as integer. Returns 0 if NULL or out of bounds.

#### get_float

```vais
F get_float(&self, row: i64, col: i64) -> f64
```

Get a value as float. Returns 0.0 if NULL or out of bounds.

#### get_bool

```vais
F get_bool(&self, row: i64, col: i64) -> i64
```

Get a value as boolean. Returns 1 for "t"/"true"/"1", 0 otherwise.

#### is_null

```vais
F is_null(&self, row: i64, col: i64) -> i64
```

Check if a value is NULL. Returns 1 if NULL, 0 otherwise.

#### clear

```vais
F clear(&self) -> i64
```

Free the result resources.

#### drop

```vais
F drop(&self) -> i64
```

Alias for clear (RAII pattern).

## PgConnection

### PgConnection Struct

```vais
S PgConnection {
    handle: i64,        # PGconn* from libpq
    host: str,
    port: i64,
    dbname: str,
    user: str,
    is_connected: i64   # 1 if connected, 0 otherwise
}
```

### PgConnection Methods

#### connect

```vais
F connect(conninfo: str) -> PgConnection
```

Connect using a full connection string (e.g., "host=localhost port=5432 dbname=mydb user=myuser password=secret").

#### connect_params

```vais
F connect_params(host: str, port: i64, dbname: str, user: str, password: str) -> PgConnection
```

Connect using individual parameters.

#### is_connected

```vais
F is_connected(&self) -> i64
```

Check if connection is active and OK. Returns 1 if connected, 0 otherwise.

#### status

```vais
F status(&self) -> i64
```

Get connection status code (PG_CONNECTION_OK or PG_CONNECTION_BAD).

#### error_message

```vais
F error_message(&self) -> str
```

Get error message from the connection.

#### exec

```vais
F exec(&self, sql: str) -> i64
```

Execute a simple SQL command (no results expected). Returns 1 on success, 0 on failure.

#### query

```vais
F query(&self, sql: str) -> PgResult
```

Execute a SQL query and return results.

#### exec_params

```vais
F exec_params(&self, sql: str, nparams: i64, params: i64) -> i64
```

Execute a parameterized query (no results expected). `params` is a pointer to an array of string pointers. Returns 1 on success, 0 on failure.

#### query_params

```vais
F query_params(&self, sql: str, nparams: i64, params: i64) -> PgResult
```

Execute a parameterized query and return results.

#### prepare

```vais
F prepare(&self, name: str, sql: str, nparams: i64) -> i64
```

Prepare a named statement. Returns 1 on success, 0 on failure.

#### exec_prepared

```vais
F exec_prepared(&self, name: str, nparams: i64, params: i64) -> i64
```

Execute a prepared statement (no results expected). Returns 1 on success, 0 on failure.

#### query_prepared

```vais
F query_prepared(&self, name: str, nparams: i64, params: i64) -> PgResult
```

Execute a prepared statement and return results.

#### begin

```vais
F begin(&self) -> i64
```

Begin a transaction. Returns 1 on success, 0 on failure.

#### commit

```vais
F commit(&self) -> i64
```

Commit a transaction. Returns 1 on success, 0 on failure.

#### rollback

```vais
F rollback(&self) -> i64
```

Rollback a transaction. Returns 1 on success, 0 on failure.

#### disconnect

```vais
F disconnect(&self) -> i64
```

Disconnect from the database.

#### drop

```vais
F drop(&self) -> i64
```

Alias for disconnect (RAII pattern).

## PgParams

### PgParams Struct

```vais
S PgParams {
    values: i64,      # Pointer to array of str pointers
    count: i64,
    capacity: i64
}
```

Helper for building parameter arrays for parameterized queries.

### PgParams Methods

#### new

```vais
F new() -> PgParams
```

Create a new parameter builder.

#### add_text

```vais
F add_text(&self, value: str) -> PgParams
```

Add a string parameter. Returns self for chaining.

#### add_int

```vais
F add_int(&self, value: i64) -> PgParams
```

Add an integer parameter (converted to string). Returns self for chaining.

#### ptr

```vais
F ptr(&self) -> i64
```

Get the raw pointer to pass to query functions.

#### len

```vais
F len(&self) -> i64
```

Get the number of parameters.

#### clear

```vais
F clear(&self) -> i64
```

Free the parameter builder.

#### drop

```vais
F drop(&self) -> i64
```

Alias for clear.

## Convenience Functions

```vais
F pg_connect(host: str, port: i64, dbname: str, user: str, password: str) -> PgConnection
```

Quick connect with individual parameters.

```vais
F pg_connect_str(conninfo: str) -> PgConnection
```

Quick connect with connection string.

```vais
F pg_connect_local(dbname: str, user: str) -> PgConnection
```

Quick connect to localhost with defaults (no password).

```vais
F build_conninfo(host: str, port: i64, dbname: str, user: str, password: str) -> str
```

Build a libpq connection info string from individual parameters.

## Usage Examples

### Simple Query

```vais
U std/postgres

F main() -> i64 {
    conn := pg_connect("localhost", 5432, "mydb", "myuser", "password")

    I conn.is_connected() == 0 {
        # Handle connection error
        R -1
    }

    result := conn.query("SELECT id, name, age FROM users")

    I result.is_ok() == 1 {
        i := 0
        L i < result.rows() {
            id := result.get_int(i, 0)
            name := result.get_text(i, 1)
            age := result.get_int(i, 2)
            # Process row...
            i = i + 1
        }
    }

    result.clear()
    conn.disconnect()
    0
}
```

### Parameterized Query

```vais
U std/postgres

F find_user(conn: &PgConnection, min_age: i64) -> i64 {
    params := PgParams::new()
        .add_int(min_age)

    result := conn.query_params(
        "SELECT name FROM users WHERE age > $1",
        params.len(),
        params.ptr()
    )

    I result.is_ok() == 1 {
        i := 0
        L i < result.rows() {
            name := result.get_text(i, 0)
            # Process name...
            i = i + 1
        }
    }

    result.clear()
    params.clear()
    0
}
```

### Transactions

```vais
U std/postgres

F transfer_funds(conn: &PgConnection, from_id: i64, to_id: i64, amount: i64) -> i64 {
    I conn.begin() == 0 {
        R -1
    }

    # Debit from account
    params1 := PgParams::new().add_int(amount).add_int(from_id)
    success := conn.exec_params(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
        params1.len(),
        params1.ptr()
    )
    params1.clear()

    I success == 0 {
        conn.rollback()
        R -1
    }

    # Credit to account
    params2 := PgParams::new().add_int(amount).add_int(to_id)
    success = conn.exec_params(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
        params2.len(),
        params2.ptr()
    )
    params2.clear()

    I success == 0 {
        conn.rollback()
        R -1
    }

    conn.commit()
}
```

### Prepared Statements

```vais
U std/postgres

F batch_insert(conn: &PgConnection, names: i64, count: i64) -> i64 {
    # Prepare statement
    success := conn.prepare(
        "insert_user",
        "INSERT INTO users (name) VALUES ($1)",
        1
    )

    I success == 0 {
        R -1
    }

    # Execute multiple times
    i := 0
    L i < count {
        name := load_str(names + i * 8)  # Get name from array
        params := PgParams::new().add_text(name)

        conn.exec_prepared("insert_user", params.len(), params.ptr())
        params.clear()

        i = i + 1
    }

    0
}
```

### Connection String Building

```vais
U std/postgres

F main() -> i64 {
    # Build connection string
    conninfo := build_conninfo(
        "db.example.com",
        5432,
        "production",
        "admin",
        "secure_password"
    )

    # Connect
    conn := pg_connect_str(conninfo)

    I conn.is_connected() == 1 {
        # Use connection...
        conn.disconnect()
    } E {
        error := conn.error_message()
        # Handle error...
    }

    0
}
```

### Error Handling

```vais
U std/postgres

F safe_query(conn: &PgConnection, sql: str) -> PgResult {
    result := conn.query(sql)

    I result.is_valid() == 0 {
        # Result is null - connection error
        error := conn.error_message()
        # Log or handle error...
        R result
    }

    I result.is_ok() == 0 {
        # Query failed
        status := result.status()
        M status {
            PG_RESULT_FATAL_ERROR => {
                # Handle fatal error
            },
            PG_RESULT_BAD_RESPONSE => {
                # Handle bad response
            },
            _ => {
                # Handle other errors
            }
        }
    }

    result
}
```
