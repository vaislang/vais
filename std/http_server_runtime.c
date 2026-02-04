// HTTP Server Framework runtime support for Vais
// Provides additional runtime functions for std/http_server.vais
// including path matching, string utilities, file I/O, and middleware support.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <ctype.h>
#include <time.h>

// ============================================
// String Utilities
// ============================================

// Check if string starts with prefix, returns 1 if true
long __str_starts_with(const char* s, const char* prefix) {
    if (s == NULL || prefix == NULL) return 0;
    size_t prefix_len = strlen(prefix);
    if (strlen(s) < prefix_len) return 0;
    return strncmp(s, prefix, prefix_len) == 0 ? 1 : 0;
}

// Check if string starts with prefix of given length
long __str_starts_with_n(const char* s, const char* prefix, long n) {
    if (s == NULL || prefix == NULL || n <= 0) return 0;
    if ((long)strlen(s) < n) return 0;
    return strncmp(s, prefix, (size_t)n) == 0 ? 1 : 0;
}

// Extract substring (caller must free)
const char* __substr(const char* s, long start, long len) {
    if (s == NULL || start < 0 || len <= 0) {
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }
    size_t slen = strlen(s);
    if ((size_t)start >= slen) {
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }
    if ((size_t)(start + len) > slen) {
        len = (long)(slen - (size_t)start);
    }
    char* result = (char*)malloc((size_t)len + 1);
    if (result) {
        memcpy(result, s + start, (size_t)len);
        result[len] = '\0';
    }
    return result;
}

// Concatenate three strings (caller must free)
const char* __str_concat3(const char* a, const char* b, const char* c) {
    if (a == NULL) a = "";
    if (b == NULL) b = "";
    if (c == NULL) c = "";
    size_t la = strlen(a), lb = strlen(b), lc = strlen(c);
    char* result = (char*)malloc(la + lb + lc + 1);
    if (result) {
        memcpy(result, a, la);
        memcpy(result + la, b, lb);
        memcpy(result + la + lb, c, lc);
        result[la + lb + lc] = '\0';
    }
    return result;
}

// Load byte at offset from pointer
long __load_byte_at(long ptr, long offset) {
    if (ptr == 0) return 0;
    return (long)(unsigned char)((const char*)ptr)[offset];
}

// ============================================
// Status Code to Text
// ============================================

const char* __status_to_text(long status) {
    switch (status) {
        case 200: return "OK";
        case 201: return "Created";
        case 202: return "Accepted";
        case 204: return "No Content";
        case 301: return "Moved Permanently";
        case 302: return "Found";
        case 304: return "Not Modified";
        case 400: return "Bad Request";
        case 401: return "Unauthorized";
        case 403: return "Forbidden";
        case 404: return "Not Found";
        case 405: return "Method Not Allowed";
        case 409: return "Conflict";
        case 500: return "Internal Server Error";
        case 501: return "Not Implemented";
        case 502: return "Bad Gateway";
        case 503: return "Service Unavailable";
        default: return "Unknown";
    }
}

// ============================================
// File Operations
// ============================================

// Read entire file into memory, returns pointer to data (caller must free)
// Returns 0 on error
long __read_file(long path) {
    const char* filepath = (const char*)path;
    if (filepath == NULL) return 0;

    FILE* f = fopen(filepath, "rb");
    if (f == NULL) return 0;

    // Get file size
    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);

    if (size <= 0) {
        fclose(f);
        return 0;
    }

    char* buffer = (char*)malloc((size_t)size + 1);
    if (buffer == NULL) {
        fclose(f);
        return 0;
    }

    size_t read = fread(buffer, 1, (size_t)size, f);
    fclose(f);

    if ((long)read != size) {
        free(buffer);
        return 0;
    }

    buffer[size] = '\0';
    return (long)buffer;
}

// Get file size, returns -1 on error
long __file_size(long path) {
    const char* filepath = (const char*)path;
    if (filepath == NULL) return -1;

    struct stat st;
    if (stat(filepath, &st) != 0) return -1;
    return (long)st.st_size;
}

// ============================================
// Path Matching
// ============================================

// Match a request path against pre-parsed route segments.
// seg_types: array of i64 (0=literal, 1=param, 2=wildcard)
// seg_values: array of string pointers
// seg_count: number of segments
// path: the request path to match
// params_items: pointer to PathParams.items
// params_count: pointer to PathParams.count
// Returns 1 if matched, 0 if not.

// PathParams struct layout:
// offset 0: items (ptr)
// offset 8: count (i64)
// offset 16: capacity (i64)

long __match_path(long seg_types, long seg_values, long seg_count,
                  const char* path, long params_ptr) {
    if (path == NULL || seg_count < 0) return 0;

    const char* p = path;
    // Skip leading /
    if (*p == '/') p++;

    long seg_idx = 0;
    while (seg_idx < seg_count) {
        // No more path to match
        if (*p == '\0') return 0;

        // Extract current path segment
        const char* seg_start = p;
        while (*p != '\0' && *p != '/') p++;
        size_t seg_len = (size_t)(p - seg_start);

        if (seg_len == 0) {
            if (*p == '/') { p++; continue; }
            return 0;
        }

        // Get segment type and value
        long seg_type = *(long*)(seg_types + seg_idx * 8);
        const char* seg_val = *(const char**)(seg_values + seg_idx * 8);

        if (seg_type == 0) {
            // Literal: exact match
            if (strlen(seg_val) != seg_len || strncmp(seg_start, seg_val, seg_len) != 0) {
                return 0;
            }
        } else if (seg_type == 1) {
            // Parameter: capture value
            if (params_ptr != 0) {
                // PathParams: items at offset 0, count at offset 8
                long items = *(long*)params_ptr;
                long count = *(long*)(params_ptr + 8);

                // Store param: name (ptr) at items + count*16, value at items + count*16 + 8
                char* value = (char*)malloc(seg_len + 1);
                memcpy(value, seg_start, seg_len);
                value[seg_len] = '\0';

                *(long*)(items + count * 16) = (long)seg_val;
                *(long*)(items + count * 16 + 8) = (long)value;
                *(long*)(params_ptr + 8) = count + 1;
            }
        } else if (seg_type == 2) {
            // Wildcard: matches everything
            return 1;
        }

        seg_idx++;

        // Skip /
        if (*p == '/') p++;
    }

    // All segments matched; path should also be consumed
    return (*p == '\0') ? 1 : 0;
}

// ============================================
// Handler and Middleware Calling
// ============================================

// ResponseBuilder struct layout (must match http_server.vais):
// offset 0:  status (i64)
// offset 8:  status_text (ptr)
// offset 16: headers (ptr)
// offset 24: header_count (i64)
// offset 32: header_capacity (i64)
// offset 40: body (ptr)
// offset 48: body_len (i64)
#define RESPONSE_BUILDER_SIZE 56

// RequestCtx struct layout (must match http_server.vais):
// offset 0:  method (i64)
// offset 8:  path (ptr)
// offset 16: full_path (ptr)
// offset 24: version (ptr)
// offset 32: headers (ptr)
// offset 40: header_count (i64)
// offset 48: header_capacity (i64)
// offset 56: body (ptr)
// offset 64: body_len (i64)
// offset 72: params (PathParams: items/count/capacity = 24 bytes)
// offset 96: query (QueryParams: items/count/capacity = 24 bytes)

// App handler: fn(ctx: &RequestCtx) -> ResponseBuilder
typedef void (*app_handler_fn)(void* out, const void* ctx);

void __call_app_handler(void* out, long handler, const void* ctx) {
    if (handler == 0 || out == NULL) {
        if (out) {
            memset(out, 0, RESPONSE_BUILDER_SIZE);
            *(long*)out = 500; // status
            *(const char**)(out + 8) = "Internal Server Error";
        }
        return;
    }
    app_handler_fn fn = (app_handler_fn)handler;
    fn(out, ctx);
}

// Middleware handler: fn(ctx: &RequestCtx, response: ResponseBuilder) -> ResponseBuilder
typedef void (*middleware_fn)(void* out, const void* ctx, const void* response);

void __call_middleware(void* out, long handler, const void* ctx, const void* response) {
    if (handler == 0 || out == NULL) {
        if (out && response) {
            memcpy(out, response, RESPONSE_BUILDER_SIZE);
        }
        return;
    }
    middleware_fn fn = (middleware_fn)handler;
    fn(out, ctx, response);
}

// ============================================
// Logging
// ============================================

long __log_request(const char* method, const char* path, long status) {
    time_t now;
    time(&now);
    struct tm* tm_info = localtime(&now);
    char time_buf[64];
    strftime(time_buf, sizeof(time_buf), "%Y-%m-%d %H:%M:%S", tm_info);

    fprintf(stderr, "[%s] %s %s -> %ld\n",
            time_buf,
            method ? method : "?",
            path ? path : "?",
            status);
    return 0;
}

long __print_server_start(const char* host, long port) {
    fprintf(stderr, "Server listening on %s:%ld\n",
            host ? host : "0.0.0.0", port);
    return 0;
}

// ============================================
// CORS Handler Factory
// ============================================

// For simplicity, this returns a pointer to the default_cors_handler
// In a full implementation, you'd create a closure capturing the origin
long __make_cors_handler(long origin) {
    // Returns 0 to indicate "use default CORS behavior"
    // The Vais-side middleware handles this
    (void)origin;
    return 0;
}
