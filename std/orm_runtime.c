// ORM runtime support for Vais
// Provides SQL string building and escaping utilities
// for the std/orm.vais standard library module.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// ============================================
// SQL String Escaping
// ============================================

// Escape a string for safe SQL usage (prevents SQL injection)
// Doubles single quotes: ' -> ''
// Returns a newly allocated escaped string
const char* __sql_escape(const char* input) {
    if (input == NULL) return "";

    size_t input_len = strlen(input);
    // Worst case: every char is a single quote -> double the length
    size_t max_len = input_len * 2 + 1;
    char* result = (char*)malloc(max_len);
    if (result == NULL) return "";

    size_t j = 0;
    for (size_t i = 0; i < input_len; i++) {
        if (input[i] == '\'') {
            result[j++] = '\'';
            result[j++] = '\'';
        } else if (input[i] == '\\') {
            result[j++] = '\\';
            result[j++] = '\\';
        } else if (input[i] == '\0') {
            break;
        } else {
            result[j++] = input[i];
        }
    }
    result[j] = '\0';
    return result;
}

// Quote and escape a string for SQL: wraps in single quotes
// e.g., "hello" -> "'hello'", "it's" -> "'it''s'"
const char* __sql_quote(const char* input) {
    if (input == NULL) return "''";

    const char* escaped = __sql_escape(input);
    size_t escaped_len = strlen(escaped);
    size_t total = escaped_len + 3; // quote + escaped + quote + null
    char* result = (char*)malloc(total);
    if (result == NULL) return "''";

    result[0] = '\'';
    memcpy(result + 1, escaped, escaped_len);
    result[escaped_len + 1] = '\'';
    result[escaped_len + 2] = '\0';

    // Free the intermediate escaped string if it was allocated
    if (escaped != input && escaped[0] != '\0') {
        free((void*)escaped);
    }

    return result;
}

// ============================================
// SQL String Builder Helpers
// ============================================

// Internal: append string to buffer, returns new position
static size_t sb_append(char* buf, size_t pos, const char* str) {
    if (str == NULL) return pos;
    size_t len = strlen(str);
    memcpy(buf + pos, str, len);
    return pos + len;
}

// Internal: append character to buffer, returns new position
static size_t sb_append_char(char* buf, size_t pos, char ch) {
    buf[pos] = ch;
    return pos + 1;
}

// Internal: append long to buffer, returns new position
static size_t sb_append_long(char* buf, size_t pos, long value) {
    int written = sprintf(buf + pos, "%ld", value);
    return pos + (size_t)written;
}

// ============================================
// SQL Statement Builders
// ============================================

// Build a SELECT statement
// table: table name
// columns: comma-separated column names (or "*")
// where_clause: WHERE conditions (empty string for none)
// order: ORDER BY clause (empty string for none)
// limit_val: LIMIT value (-1 for no limit)
// Returns a newly allocated SQL string
const char* __sql_build_select(const char* table, const char* columns,
                                const char* where_clause, const char* order,
                                long limit_val) {
    if (table == NULL) return "";

    size_t buf_size = 4096;
    char* buf = (char*)malloc(buf_size);
    if (buf == NULL) return "";

    size_t pos = 0;
    pos = sb_append(buf, pos, "SELECT ");

    if (columns != NULL && strlen(columns) > 0) {
        pos = sb_append(buf, pos, columns);
    } else {
        pos = sb_append_char(buf, pos, '*');
    }

    pos = sb_append(buf, pos, " FROM ");
    pos = sb_append(buf, pos, table);

    if (where_clause != NULL && strlen(where_clause) > 0) {
        pos = sb_append(buf, pos, " WHERE ");
        pos = sb_append(buf, pos, where_clause);
    }

    if (order != NULL && strlen(order) > 0) {
        pos = sb_append(buf, pos, " ORDER BY ");
        pos = sb_append(buf, pos, order);
    }

    if (limit_val >= 0) {
        pos = sb_append(buf, pos, " LIMIT ");
        pos = sb_append_long(buf, pos, limit_val);
    }

    pos = sb_append_char(buf, pos, ';');
    buf[pos] = '\0';

    return buf;
}

// Build an INSERT statement
// table: table name
// columns: comma-separated column names
// values: comma-separated values (should already be quoted/escaped)
// Returns a newly allocated SQL string
const char* __sql_build_insert(const char* table, const char* columns,
                                const char* values) {
    if (table == NULL) return "";

    size_t buf_size = 4096;
    char* buf = (char*)malloc(buf_size);
    if (buf == NULL) return "";

    size_t pos = 0;
    pos = sb_append(buf, pos, "INSERT INTO ");
    pos = sb_append(buf, pos, table);
    pos = sb_append(buf, pos, " (");

    if (columns != NULL) {
        pos = sb_append(buf, pos, columns);
    }

    pos = sb_append(buf, pos, ") VALUES (");

    if (values != NULL) {
        pos = sb_append(buf, pos, values);
    }

    pos = sb_append_char(buf, pos, ')');
    pos = sb_append_char(buf, pos, ';');
    buf[pos] = '\0';

    return buf;
}

// Build an UPDATE statement
// table: table name
// set_clause: SET assignments (e.g., "name = 'Alice', age = 30")
// where_clause: WHERE conditions (empty string for none)
// Returns a newly allocated SQL string
const char* __sql_build_update(const char* table, const char* set_clause,
                                const char* where_clause) {
    if (table == NULL) return "";

    size_t buf_size = 4096;
    char* buf = (char*)malloc(buf_size);
    if (buf == NULL) return "";

    size_t pos = 0;
    pos = sb_append(buf, pos, "UPDATE ");
    pos = sb_append(buf, pos, table);
    pos = sb_append(buf, pos, " SET ");

    if (set_clause != NULL) {
        pos = sb_append(buf, pos, set_clause);
    }

    if (where_clause != NULL && strlen(where_clause) > 0) {
        pos = sb_append(buf, pos, " WHERE ");
        pos = sb_append(buf, pos, where_clause);
    }

    pos = sb_append_char(buf, pos, ';');
    buf[pos] = '\0';

    return buf;
}

// Build a DELETE statement
// table: table name
// where_clause: WHERE conditions (empty string for none)
// Returns a newly allocated SQL string
const char* __sql_build_delete(const char* table, const char* where_clause) {
    if (table == NULL) return "";

    size_t buf_size = 4096;
    char* buf = (char*)malloc(buf_size);
    if (buf == NULL) return "";

    size_t pos = 0;
    pos = sb_append(buf, pos, "DELETE FROM ");
    pos = sb_append(buf, pos, table);

    if (where_clause != NULL && strlen(where_clause) > 0) {
        pos = sb_append(buf, pos, " WHERE ");
        pos = sb_append(buf, pos, where_clause);
    }

    pos = sb_append_char(buf, pos, ';');
    buf[pos] = '\0';

    return buf;
}

// Build a CREATE TABLE statement
// table: table name
// column_defs: comma-separated column definitions
//   e.g., "id INTEGER PRIMARY KEY, name TEXT NOT NULL, age INTEGER DEFAULT 0"
// Returns a newly allocated SQL string
const char* __sql_build_create_table(const char* table, const char* column_defs) {
    if (table == NULL) return "";

    size_t buf_size = 4096;
    char* buf = (char*)malloc(buf_size);
    if (buf == NULL) return "";

    size_t pos = 0;
    pos = sb_append(buf, pos, "CREATE TABLE IF NOT EXISTS ");
    pos = sb_append(buf, pos, table);
    pos = sb_append(buf, pos, " (");

    if (column_defs != NULL) {
        pos = sb_append(buf, pos, column_defs);
    }

    pos = sb_append(buf, pos, ");");
    buf[pos] = '\0';

    return buf;
}
