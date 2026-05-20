// PostgreSQL runtime support for Vais
// Wraps libpq (PostgreSQL C client library) for the std/postgres.vais module.
//
// Build requirements:
//   - libpq development headers (libpq-fe.h)
//   - Link with: -lpq
//
// Example compilation:
//   vaisc myapp.vais
//   clang myapp.ll std/postgres_runtime.c -lpq -o myapp
//
// On macOS with Homebrew:
//   clang myapp.ll std/postgres_runtime.c -I$(pg_config --includedir) -L$(pg_config --libdir) -lpq -o myapp
//
// On Ubuntu/Debian:
//   apt install libpq-dev
//   clang myapp.ll std/postgres_runtime.c -lpq -o myapp

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <libpq-fe.h>

// std/postgres.vais contains connection-string and parameter helpers that
// reference these stdlib helpers. Provide weak postgres-local definitions so
// postgres-only builds link, while builds that also link a stronger runtime
// helper (for example std/http) avoid duplicate-symbol failures.
__attribute__((weak)) long __str_copy_to(long dst, const char* src) {
    if (dst == 0 || src == NULL) return 0;
    size_t len = strlen(src);
    memcpy((void*)dst, src, len);
    return (long)len;
}

__attribute__((weak)) long __i64_to_str(long dst, long value) {
    if (dst == 0) return 0;
    int written = sprintf((char*)dst, "%ld", value);
    return (long)written;
}

// ============================================
// Connection Management
// ============================================

// Connect to PostgreSQL using a connection info string.
// conninfo: libpq connection string, e.g.,
//   "host=localhost port=5432 dbname=mydb user=myuser password=secret"
// Returns: opaque handle (PGconn*) cast to i64, or 0 on failure.
long __pg_connect(const char* conninfo) {
    if (conninfo == NULL) return 0;
    PGconn* conn = PQconnectdb(conninfo);
    if (conn == NULL) return 0;
    return (long)conn;
}

// Close the connection and free resources.
// handle: PGconn* cast to i64
// Returns: 0
long __pg_finish(long handle) {
    if (handle != 0) {
        PQfinish((PGconn*)handle);
    }
    return 0;
}

// ============================================
// Query Execution
// ============================================

// Execute a simple SQL command (no parameters).
// handle: PGconn*
// sql: SQL string
// Returns: PGresult* cast to i64, or 0 on failure.
long __pg_exec(long handle, const char* sql) {
    if (handle == 0 || sql == NULL) return 0;
    PGresult* res = PQexec((PGconn*)handle, sql);
    return (long)res;
}

// Execute a parameterized SQL command.
// handle: PGconn*
// sql: SQL with $1, $2, ... placeholders
// nparams: number of parameters
// param_values: pointer to array of C string pointers (const char**)
// Returns: PGresult* cast to i64, or 0 on failure.
long __pg_exec_params(long handle, const char* sql, long nparams, long param_values) {
    if (handle == 0 || sql == NULL) return 0;
    if (nparams < 0) nparams = 0;

    const char** values = NULL;
    if (nparams > 0 && param_values != 0) {
        values = (const char**)param_values;
    }

    PGresult* res = PQexecParams(
        (PGconn*)handle,
        sql,
        (int)nparams,
        NULL,           // paramTypes: let server infer types
        values,
        NULL,           // paramLengths: not needed for text format
        NULL,           // paramFormats: NULL = all text
        0               // resultFormat: 0 = text
    );
    return (long)res;
}

// Prepare a named statement.
// handle: PGconn*
// name: statement name (empty string "" for unnamed)
// sql: SQL with $1, $2, ... placeholders
// nparams: number of parameters
// Returns: PGresult* cast to i64, or 0 on failure.
long __pg_prepare(long handle, const char* name, const char* sql, long nparams) {
    if (handle == 0 || sql == NULL) return 0;
    if (name == NULL) name = "";
    if (nparams < 0) nparams = 0;

    PGresult* res = PQprepare(
        (PGconn*)handle,
        name,
        sql,
        (int)nparams,
        NULL            // paramTypes: let server infer types
    );
    return (long)res;
}

// Execute a previously prepared statement.
// handle: PGconn*
// name: statement name
// nparams: number of parameters
// param_values: pointer to array of C string pointers (const char**)
// Returns: PGresult* cast to i64, or 0 on failure.
long __pg_exec_prepared(long handle, const char* name, long nparams, long param_values) {
    if (handle == 0 || name == NULL) return 0;
    if (nparams < 0) nparams = 0;

    const char** values = NULL;
    if (nparams > 0 && param_values != 0) {
        values = (const char**)param_values;
    }

    PGresult* res = PQexecPrepared(
        (PGconn*)handle,
        name,
        (int)nparams,
        values,
        NULL,           // paramLengths: not needed for text format
        NULL,           // paramFormats: NULL = all text
        0               // resultFormat: 0 = text
    );
    return (long)res;
}

// ============================================
// Result Inspection
// ============================================

// Get the number of rows in a result.
// result: PGresult*
// Returns: number of rows, or 0 if result is NULL.
long __pg_ntuples(long result) {
    if (result == 0) return 0;
    return (long)PQntuples((PGresult*)result);
}

// Get the number of columns in a result.
// result: PGresult*
// Returns: number of columns, or 0 if result is NULL.
long __pg_nfields(long result) {
    if (result == 0) return 0;
    return (long)PQnfields((PGresult*)result);
}

// Get a field name as a C string.
// result: PGresult*
// col: column index (0-based)
// Returns: pointer owned by PGresult. It must not be freed separately and
//          becomes invalid after PQclear(result). Returns "" if invalid.
const char* __pg_fname(long result, long col) {
    if (result == 0 || col < 0) return "";
    PGresult* res = (PGresult*)result;
    if (col >= PQnfields(res)) return "";
    const char* name = PQfname(res, (int)col);
    if (name == NULL) return "";
    return name;
}

// Get a field value as a C string.
// result: PGresult*
// row: row index (0-based)
// col: column index (0-based)
// Returns: pointer to null-terminated string. The string is owned by
//          the PGresult and must not be freed separately.
//          Returns "" if result is NULL or indices are out of range.
const char* __pg_getvalue(long result, long row, long col) {
    if (result == 0) return "";
    const char* val = PQgetvalue((PGresult*)result, (int)row, (int)col);
    if (val == NULL) return "";
    return val;
}

// Allocate a malloc-owned copy of `src` (or an owned empty string when `src`
// is NULL). Returns NULL only on allocation failure.
static char* pg_owned_copy(const char* src) {
    size_t len = (src != NULL) ? strlen(src) : 0;
    char* copy = (char*)malloc(len + 1);
    if (copy == NULL) return NULL;
    if (len > 0) {
        memcpy(copy, src, len);
    }
    copy[len] = '\0';
    return copy;
}

// Get a field name as a malloc-owned C string copy.
// The returned pointer is stable after PQclear(result) and must be released
// with __pg_free_text. Invalid handles or out-of-range columns yield owned "".
const char* __pg_fname_copy(long result, long col) {
    if (result == 0 || col < 0) return pg_owned_copy(NULL);
    PGresult* res = (PGresult*)result;
    if (col >= PQnfields(res)) return pg_owned_copy(NULL);
    const char* name = PQfname(res, (int)col);
    return pg_owned_copy(name);
}

// Get a field value as a malloc-owned C string copy.
// result: PGresult*
// row: row index (0-based)
// col: column index (0-based)
//
// OWNERSHIP: the returned pointer is a malloc-owned copy that the caller owns
// and must release with __pg_free_text. Invalid handles, out-of-range indexes,
// and SQL NULL values return an owned empty string. The copy remains valid
// after PQclear(result).
const char* __pg_getvalue_copy(long result, long row, long col) {
    if (result == 0) return pg_owned_copy(NULL);
    PGresult* res = (PGresult*)result;
    if (row < 0 || col < 0) return pg_owned_copy(NULL);
    if (row >= PQntuples(res) || col >= PQnfields(res)) return pg_owned_copy(NULL);
    if (PQgetisnull(res, (int)row, (int)col)) return pg_owned_copy(NULL);
    const char* val = PQgetvalue(res, (int)row, (int)col);
    return pg_owned_copy(val);
}

// Free a string previously returned by __pg_getvalue_copy.
// Frees non-NULL pointers; a NULL pointer is a no-op. Always returns 0.
long __pg_free_text(const char* ptr) {
    if (ptr != NULL) {
        free((void*)ptr);
    }
    return 0;
}

// Check if a field value is NULL.
// result: PGresult*
// row: row index (0-based)
// col: column index (0-based)
// Returns: 1 if NULL, 0 if not NULL, 1 if result is invalid.
long __pg_getisnull(long result, long row, long col) {
    if (result == 0) return 1;
    return (long)PQgetisnull((PGresult*)result, (int)row, (int)col);
}

// Free a PGresult.
// result: PGresult*
// Returns: 0
long __pg_clear(long result) {
    if (result != 0) {
        PQclear((PGresult*)result);
    }
    return 0;
}

// ============================================
// Connection Status
// ============================================

// Get the connection status.
// handle: PGconn*
// Returns: PGconnStatusType as i64 (0 = CONNECTION_OK, 1 = CONNECTION_BAD)
long __pg_status(long handle) {
    if (handle == 0) return 1; // CONNECTION_BAD
    return (long)PQstatus((PGconn*)handle);
}

// Get the error message associated with the connection.
// handle: PGconn*
// Returns: pointer to error message string (owned by PGconn, do not free).
//          Returns "No connection" if handle is NULL.
const char* __pg_error_message(long handle) {
    if (handle == 0) return "No connection";
    const char* msg = PQerrorMessage((PGconn*)handle);
    if (msg == NULL) return "";
    return msg;
}

// Get the connection error message as a malloc-owned copy.
// The returned pointer is stable after PQfinish(conn) and must be released
// with __pg_free_text. A NULL handle yields an owned "No connection" string.
const char* __pg_error_message_copy(long handle) {
    if (handle == 0) return pg_owned_copy("No connection");
    const char* msg = PQerrorMessage((PGconn*)handle);
    return pg_owned_copy(msg);
}

// Get the result status.
// result: PGresult*
// Returns: ExecStatusType as i64
//   0 = PGRES_EMPTY_QUERY
//   1 = PGRES_COMMAND_OK
//   2 = PGRES_TUPLES_OK
//   5 = PGRES_BAD_RESPONSE
//   7 = PGRES_FATAL_ERROR
long __pg_result_status(long result) {
    if (result == 0) return 7; // PGRES_FATAL_ERROR
    return (long)PQresultStatus((PGresult*)result);
}
