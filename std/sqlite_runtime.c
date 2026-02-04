// SQLite3 runtime support for Vais
// Provides C bindings that wrap the SQLite3 C API for use by std/sqlite.vais.
//
// Build: Link with -lsqlite3
//   clang -o your_app your_app.ll std/sqlite_runtime.c -lsqlite3
//
// Requires: sqlite3.h and libsqlite3 installed on the system.
//   macOS:   brew install sqlite3  (or use system sqlite3)
//   Linux:   apt install libsqlite3-dev  /  dnf install sqlite-devel
//   Windows: download from https://sqlite.org/download.html

#include <sqlite3.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// ============================================
// Database Operations
// ============================================

// Open a database file. Returns opaque handle (sqlite3*) cast to i64.
// Returns 0 on failure.
long __sqlite_open(const char* path) {
    if (path == NULL) return 0;

    sqlite3* db = NULL;
    int rc = sqlite3_open(path, &db);
    if (rc != SQLITE_OK) {
        if (db != NULL) {
            sqlite3_close(db);
        }
        return 0;
    }
    return (long)db;
}

// Close a database connection. Returns SQLITE_OK (0) on success.
long __sqlite_close(long handle) {
    if (handle == 0) return SQLITE_OK;
    sqlite3* db = (sqlite3*)handle;
    return (long)sqlite3_close(db);
}

// Execute a simple SQL statement (no results).
// callback is reserved for future use (pass 0 from Vais).
// Returns SQLITE_OK (0) on success, error code on failure.
long __sqlite_exec(long handle, const char* sql, long callback) {
    if (handle == 0 || sql == NULL) return SQLITE_MISUSE;
    sqlite3* db = (sqlite3*)handle;
    char* errmsg = NULL;

    // callback parameter is ignored for now (no callback support)
    int rc = sqlite3_exec(db, sql, NULL, NULL, &errmsg);
    if (errmsg != NULL) {
        sqlite3_free(errmsg);
    }
    return (long)rc;
}

// ============================================
// Prepared Statement Operations
// ============================================

// Prepare a SQL statement. Returns opaque stmt handle (sqlite3_stmt*) cast to i64.
// Returns 0 on failure.
long __sqlite_prepare(long handle, const char* sql) {
    if (handle == 0 || sql == NULL) return 0;
    sqlite3* db = (sqlite3*)handle;
    sqlite3_stmt* stmt = NULL;

    int rc = sqlite3_prepare_v2(db, sql, -1, &stmt, NULL);
    if (rc != SQLITE_OK) {
        if (stmt != NULL) {
            sqlite3_finalize(stmt);
        }
        return 0;
    }
    return (long)stmt;
}

// Bind an integer value to a prepared statement parameter.
// index is 1-based (first parameter is 1).
// Returns SQLITE_OK on success.
long __sqlite_bind_int(long stmt_handle, long index, long value) {
    if (stmt_handle == 0) return SQLITE_MISUSE;
    sqlite3_stmt* stmt = (sqlite3_stmt*)stmt_handle;
    return (long)sqlite3_bind_int64(stmt, (int)index, (sqlite3_int64)value);
}

// Bind a text value to a prepared statement parameter.
// index is 1-based. The text is copied by SQLite (SQLITE_TRANSIENT).
// Returns SQLITE_OK on success.
long __sqlite_bind_text(long stmt_handle, long index, const char* text) {
    if (stmt_handle == 0) return SQLITE_MISUSE;
    sqlite3_stmt* stmt = (sqlite3_stmt*)stmt_handle;
    if (text == NULL) {
        return (long)sqlite3_bind_null(stmt, (int)index);
    }
    return (long)sqlite3_bind_text(stmt, (int)index, text, -1, SQLITE_TRANSIENT);
}

// Bind a double value to a prepared statement parameter.
// The value is passed as the raw i64 bit pattern of a double.
// index is 1-based.
// Returns SQLITE_OK on success.
long __sqlite_bind_double(long stmt_handle, long index, long value) {
    if (stmt_handle == 0) return SQLITE_MISUSE;
    sqlite3_stmt* stmt = (sqlite3_stmt*)stmt_handle;
    double dval;
    memcpy(&dval, &value, sizeof(double));
    return (long)sqlite3_bind_double(stmt, (int)index, dval);
}

// Bind a NULL value to a prepared statement parameter.
// index is 1-based.
// Returns SQLITE_OK on success.
long __sqlite_bind_null(long stmt_handle, long index) {
    if (stmt_handle == 0) return SQLITE_MISUSE;
    sqlite3_stmt* stmt = (sqlite3_stmt*)stmt_handle;
    return (long)sqlite3_bind_null(stmt, (int)index);
}

// Execute one step of a prepared statement.
// Returns:
//   SQLITE_ROW  (100) - a row of data is available
//   SQLITE_DONE (101) - statement has finished executing
//   Other error code on failure
long __sqlite_step(long stmt_handle) {
    if (stmt_handle == 0) return SQLITE_MISUSE;
    sqlite3_stmt* stmt = (sqlite3_stmt*)stmt_handle;
    return (long)sqlite3_step(stmt);
}

// ============================================
// Column Access (0-indexed)
// ============================================

// Get an integer value from a column in the current result row.
// index is 0-based (first column is 0).
long __sqlite_column_int(long stmt_handle, long index) {
    if (stmt_handle == 0) return 0;
    sqlite3_stmt* stmt = (sqlite3_stmt*)stmt_handle;
    return (long)sqlite3_column_int64(stmt, (int)index);
}

// Get a text value from a column in the current result row.
// index is 0-based. Returns a pointer to SQLite-managed memory.
// The string is valid until the next sqlite3_step() or sqlite3_finalize().
// We make a copy to ensure safety.
const char* __sqlite_column_text(long stmt_handle, long index) {
    if (stmt_handle == 0) return "";
    sqlite3_stmt* stmt = (sqlite3_stmt*)stmt_handle;
    const unsigned char* text = sqlite3_column_text(stmt, (int)index);
    if (text == NULL) return "";

    // Make a copy so the Vais side has a stable pointer
    size_t len = strlen((const char*)text);
    char* copy = (char*)malloc(len + 1);
    if (copy == NULL) return "";
    memcpy(copy, text, len + 1);
    return copy;
}

// Get a double value from a column in the current result row.
// index is 0-based. Returns the raw i64 bit pattern of the double.
long __sqlite_column_double(long stmt_handle, long index) {
    if (stmt_handle == 0) return 0;
    sqlite3_stmt* stmt = (sqlite3_stmt*)stmt_handle;
    double dval = sqlite3_column_double(stmt, (int)index);
    long result;
    memcpy(&result, &dval, sizeof(double));
    return result;
}

// Get the type code of a column in the current result row.
// index is 0-based.
// Returns: SQLITE_INTEGER (1), SQLITE_FLOAT (2), SQLITE_TEXT (3),
//          SQLITE_BLOB (4), SQLITE_NULL (5)
long __sqlite_column_type(long stmt_handle, long index) {
    if (stmt_handle == 0) return SQLITE_NULL;
    sqlite3_stmt* stmt = (sqlite3_stmt*)stmt_handle;
    return (long)sqlite3_column_type(stmt, (int)index);
}

// Get the number of columns in the result set.
long __sqlite_column_count(long stmt_handle) {
    if (stmt_handle == 0) return 0;
    sqlite3_stmt* stmt = (sqlite3_stmt*)stmt_handle;
    return (long)sqlite3_column_count(stmt);
}

// Get the name of a column in the result set.
// index is 0-based. Returns a pointer to SQLite-managed memory.
const char* __sqlite_column_name(long stmt_handle, long index) {
    if (stmt_handle == 0) return "";
    sqlite3_stmt* stmt = (sqlite3_stmt*)stmt_handle;
    const char* name = sqlite3_column_name(stmt, (int)index);
    if (name == NULL) return "";
    return name;
}

// ============================================
// Statement Lifecycle
// ============================================

// Finalize (destroy) a prepared statement. Returns SQLITE_OK on success.
long __sqlite_finalize(long stmt_handle) {
    if (stmt_handle == 0) return SQLITE_OK;
    sqlite3_stmt* stmt = (sqlite3_stmt*)stmt_handle;
    return (long)sqlite3_finalize(stmt);
}

// Reset a prepared statement for re-execution.
// Does not clear parameter bindings.
// Returns SQLITE_OK on success.
long __sqlite_reset(long stmt_handle) {
    if (stmt_handle == 0) return SQLITE_OK;
    sqlite3_stmt* stmt = (sqlite3_stmt*)stmt_handle;
    return (long)sqlite3_reset(stmt);
}

// ============================================
// Error & Info
// ============================================

// Get the most recent error message for a database connection.
// Returns a pointer to a SQLite-managed string (do not free).
const char* __sqlite_errmsg(long handle) {
    if (handle == 0) return "Database handle is NULL";
    sqlite3* db = (sqlite3*)handle;
    return sqlite3_errmsg(db);
}

// Get the rowid of the most recently inserted row.
long __sqlite_last_insert_rowid(long handle) {
    if (handle == 0) return 0;
    sqlite3* db = (sqlite3*)handle;
    return (long)sqlite3_last_insert_rowid(db);
}

// Get the number of rows changed by the most recent INSERT, UPDATE, or DELETE.
long __sqlite_changes(long handle) {
    if (handle == 0) return 0;
    sqlite3* db = (sqlite3*)handle;
    return (long)sqlite3_changes(db);
}
