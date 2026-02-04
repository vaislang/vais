// Logging runtime support for Vais
// Provides structured logging with levels, JSON output, spans, and file I/O
// Thread-safe logging using pthread mutex
//
// Features:
// - Log levels: TRACE, DEBUG, INFO, WARN, ERROR
// - Output targets: stdout, stderr, file
// - Formats: text (human-readable), JSON
// - Structured fields (key-value pairs)
// - Span-based tracing with unique IDs
// - ISO8601 timestamps
// - Thread-safe with mutex locking

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <pthread.h>
#include <stdarg.h>
#include <unistd.h>
#include <sys/time.h>

// ============================================
// Constants
// ============================================

#define LOG_LEVEL_TRACE 0
#define LOG_LEVEL_DEBUG 1
#define LOG_LEVEL_INFO  2
#define LOG_LEVEL_WARN  3
#define LOG_LEVEL_ERROR 4

#define LOG_OUTPUT_STDOUT 0
#define LOG_OUTPUT_STDERR 1
#define LOG_OUTPUT_FILE   2

#define LOG_FORMAT_TEXT 0
#define LOG_FORMAT_JSON 1

#define LOG_OK                  0
#define LOG_ERR_INIT           -1
#define LOG_ERR_FILE           -2
#define LOG_ERR_INVALID_LEVEL  -3
#define LOG_ERR_INVALID_OUTPUT -4
#define LOG_ERR_INVALID_FORMAT -5
#define LOG_ERR_SPAN           -6
#define LOG_ERR_WRITE          -7

#define MAX_SPANS 1024
#define MAX_LOG_LINE 4096
#define MAX_TRACE_ID 32

// ============================================
// Global Logger State
// ============================================

typedef struct {
    long level;           // Current log level
    long output;          // Output target (stdout/stderr/file)
    long format;          // Output format (text/json)
    FILE* file;           // File handle (if output is LOG_OUTPUT_FILE)
    char file_path[256];  // Log file path
    pthread_mutex_t lock; // Thread-safety lock
    int initialized;      // 1 if initialized
} Logger;

static Logger global_logger = {
    .level = LOG_LEVEL_INFO,
    .output = LOG_OUTPUT_STDOUT,
    .format = LOG_FORMAT_TEXT,
    .file = NULL,
    .file_path = "",
    .lock = PTHREAD_MUTEX_INITIALIZER,
    .initialized = 0
};

// ============================================
// Span Tracking
// ============================================

typedef struct {
    long span_id;
    char name[128];
    char trace_id[MAX_TRACE_ID];
    int active;
} Span;

static Span spans[MAX_SPANS];
static long next_span_id = 1;
static pthread_mutex_t span_lock = PTHREAD_MUTEX_INITIALIZER;

// ============================================
// Helper Functions
// ============================================

// Get current timestamp in ISO8601 format
static void get_timestamp_iso8601(char* buf, size_t size) {
    struct timeval tv;
    gettimeofday(&tv, NULL);

    struct tm* tm_info = gmtime(&tv.tv_sec);
    strftime(buf, size, "%Y-%m-%dT%H:%M:%S", tm_info);

    // Append milliseconds
    char ms_buf[8];
    snprintf(ms_buf, sizeof(ms_buf), ".%03dZ", (int)(tv.tv_usec / 1000));
    strncat(buf, ms_buf, size - strlen(buf) - 1);
}

// Generate a unique trace ID (simple counter-based for now)
// NOTE: Must be called while holding span_lock
static void generate_trace_id(char* buf, size_t size) {
    static unsigned long trace_counter = 0;
    unsigned long id = ++trace_counter;
    snprintf(buf, size, "trace-%016lx", id);
}

// Get log level name
static const char* level_name(long level) {
    switch (level) {
        case LOG_LEVEL_TRACE: return "TRACE";
        case LOG_LEVEL_DEBUG: return "DEBUG";
        case LOG_LEVEL_INFO:  return "INFO";
        case LOG_LEVEL_WARN:  return "WARN";
        case LOG_LEVEL_ERROR: return "ERROR";
        default: return "UNKNOWN";
    }
}

// JSON string escape (minimal: escape quotes, backslashes, newlines)
static void json_escape(const char* src, char* dst, size_t dst_size) {
    size_t j = 0;
    for (size_t i = 0; src[i] != '\0' && j < dst_size - 2; i++) {
        if (src[i] == '"' || src[i] == '\\') {
            dst[j++] = '\\';
            dst[j++] = src[i];
        } else if (src[i] == '\n') {
            dst[j++] = '\\';
            dst[j++] = 'n';
        } else if (src[i] == '\r') {
            dst[j++] = '\\';
            dst[j++] = 'r';
        } else if (src[i] == '\t') {
            dst[j++] = '\\';
            dst[j++] = 't';
        } else {
            dst[j++] = src[i];
        }
    }
    dst[j] = '\0';
}

// Get output FILE* based on current configuration
static FILE* get_output_file(void) {
    if (global_logger.output == LOG_OUTPUT_STDERR) {
        return stderr;
    } else if (global_logger.output == LOG_OUTPUT_FILE && global_logger.file != NULL) {
        return global_logger.file;
    } else {
        return stdout;
    }
}

// ============================================
// Logger Initialization
// ============================================

long __log_init(long level) {
    pthread_mutex_lock(&global_logger.lock);

    if (level < LOG_LEVEL_TRACE || level > LOG_LEVEL_ERROR) {
        pthread_mutex_unlock(&global_logger.lock);
        return LOG_ERR_INVALID_LEVEL;
    }

    global_logger.level = level;
    global_logger.initialized = 1;

    pthread_mutex_unlock(&global_logger.lock);
    return LOG_OK;
}

long __log_set_level(long level) {
    pthread_mutex_lock(&global_logger.lock);

    if (level >= LOG_LEVEL_TRACE && level <= LOG_LEVEL_ERROR) {
        global_logger.level = level;
    }

    pthread_mutex_unlock(&global_logger.lock);
    return 0;
}

long __log_set_output(long target) {
    pthread_mutex_lock(&global_logger.lock);

    if (target >= LOG_OUTPUT_STDOUT && target <= LOG_OUTPUT_FILE) {
        global_logger.output = target;
    }

    pthread_mutex_unlock(&global_logger.lock);
    return 0;
}

long __log_set_file(const char* path) {
    if (path == NULL) return LOG_ERR_FILE;

    pthread_mutex_lock(&global_logger.lock);

    // Close existing file if open
    if (global_logger.file != NULL) {
        fclose(global_logger.file);
        global_logger.file = NULL;
    }

    // Open new file (append mode)
    global_logger.file = fopen(path, "a");
    if (global_logger.file == NULL) {
        pthread_mutex_unlock(&global_logger.lock);
        return LOG_ERR_FILE;
    }

    // Store path
    strncpy(global_logger.file_path, path, sizeof(global_logger.file_path) - 1);
    global_logger.file_path[sizeof(global_logger.file_path) - 1] = '\0';

    // Set line buffering
    setvbuf(global_logger.file, NULL, _IOLBF, 0);

    pthread_mutex_unlock(&global_logger.lock);
    return LOG_OK;
}

long __log_set_format(long format) {
    pthread_mutex_lock(&global_logger.lock);

    if (format >= LOG_FORMAT_TEXT && format <= LOG_FORMAT_JSON) {
        global_logger.format = format;
    }

    pthread_mutex_unlock(&global_logger.lock);
    return 0;
}

// ============================================
// Core Logging Functions
// ============================================

long __log_write(long level, const char* msg) {
    if (msg == NULL) return 0;

    pthread_mutex_lock(&global_logger.lock);

    // Check if message should be logged
    if (level < global_logger.level) {
        pthread_mutex_unlock(&global_logger.lock);
        return 0;
    }

    char timestamp[32];
    get_timestamp_iso8601(timestamp, sizeof(timestamp));

    FILE* out = get_output_file();

    if (global_logger.format == LOG_FORMAT_JSON) {
        // JSON format
        char escaped_msg[MAX_LOG_LINE];
        json_escape(msg, escaped_msg, sizeof(escaped_msg));

        fprintf(out, "{\"timestamp\":\"%s\",\"level\":\"%s\",\"msg\":\"%s\"}\n",
                timestamp, level_name(level), escaped_msg);
    } else {
        // Text format
        fprintf(out, "[%s] %s: %s\n", timestamp, level_name(level), msg);
    }

    fflush(out);

    pthread_mutex_unlock(&global_logger.lock);
    return 0;
}

long __log_with_field(long level, const char* msg, const char* key, const char* value) {
    if (msg == NULL || key == NULL || value == NULL) return 0;

    pthread_mutex_lock(&global_logger.lock);

    if (level < global_logger.level) {
        pthread_mutex_unlock(&global_logger.lock);
        return 0;
    }

    char timestamp[32];
    get_timestamp_iso8601(timestamp, sizeof(timestamp));

    FILE* out = get_output_file();

    if (global_logger.format == LOG_FORMAT_JSON) {
        // JSON format with field
        char escaped_msg[MAX_LOG_LINE];
        char escaped_key[256];
        char escaped_value[MAX_LOG_LINE];

        json_escape(msg, escaped_msg, sizeof(escaped_msg));
        json_escape(key, escaped_key, sizeof(escaped_key));
        json_escape(value, escaped_value, sizeof(escaped_value));

        fprintf(out, "{\"timestamp\":\"%s\",\"level\":\"%s\",\"msg\":\"%s\",\"%s\":\"%s\"}\n",
                timestamp, level_name(level), escaped_msg, escaped_key, escaped_value);
    } else {
        // Text format with field
        fprintf(out, "[%s] %s: %s [%s=%s]\n",
                timestamp, level_name(level), msg, key, value);
    }

    fflush(out);

    pthread_mutex_unlock(&global_logger.lock);
    return 0;
}

long __log_with_fields(long level, const char* msg, const char* fields) {
    if (msg == NULL || fields == NULL) return 0;

    pthread_mutex_lock(&global_logger.lock);

    if (level < global_logger.level) {
        pthread_mutex_unlock(&global_logger.lock);
        return 0;
    }

    char timestamp[32];
    get_timestamp_iso8601(timestamp, sizeof(timestamp));

    FILE* out = get_output_file();

    if (global_logger.format == LOG_FORMAT_JSON) {
        // JSON format with multiple fields
        char escaped_msg[MAX_LOG_LINE];
        json_escape(msg, escaped_msg, sizeof(escaped_msg));

        fprintf(out, "{\"timestamp\":\"%s\",\"level\":\"%s\",\"msg\":\"%s\"",
                timestamp, level_name(level), escaped_msg);

        // Parse fields (format: "key1=value1,key2=value2,...")
        char fields_copy[MAX_LOG_LINE];
        strncpy(fields_copy, fields, sizeof(fields_copy) - 1);
        fields_copy[sizeof(fields_copy) - 1] = '\0';

        char* token = strtok(fields_copy, ",");
        while (token != NULL) {
            // Split by '='
            char* eq = strchr(token, '=');
            if (eq != NULL) {
                *eq = '\0';
                const char* key = token;
                const char* value = eq + 1;

                char escaped_key[256];
                char escaped_value[MAX_LOG_LINE];
                json_escape(key, escaped_key, sizeof(escaped_key));
                json_escape(value, escaped_value, sizeof(escaped_value));

                fprintf(out, ",\"%s\":\"%s\"", escaped_key, escaped_value);
            }
            token = strtok(NULL, ",");
        }

        fprintf(out, "}\n");
    } else {
        // Text format with fields
        fprintf(out, "[%s] %s: %s [%s]\n",
                timestamp, level_name(level), msg, fields);
    }

    fflush(out);

    pthread_mutex_unlock(&global_logger.lock);
    return 0;
}

// ============================================
// Span Tracing
// ============================================

long __span_start(const char* name) {
    if (name == NULL) return LOG_ERR_SPAN;

    pthread_mutex_lock(&span_lock);

    // Find free span slot
    Span* span = NULL;
    for (int i = 0; i < MAX_SPANS; i++) {
        if (!spans[i].active) {
            span = &spans[i];
            break;
        }
    }

    if (span == NULL) {
        pthread_mutex_unlock(&span_lock);
        return LOG_ERR_SPAN;  // No free slots
    }

    // Initialize span
    span->span_id = next_span_id++;
    strncpy(span->name, name, sizeof(span->name) - 1);
    span->name[sizeof(span->name) - 1] = '\0';
    generate_trace_id(span->trace_id, sizeof(span->trace_id));
    span->active = 1;

    long span_id = span->span_id;

    pthread_mutex_unlock(&span_lock);

    // Log span start
    char start_msg[256];
    snprintf(start_msg, sizeof(start_msg), "Span started: %s", name);
    __log_with_field(LOG_LEVEL_DEBUG, start_msg, "trace_id", span->trace_id);

    return span_id;
}

long __span_end(long span_id) {
    pthread_mutex_lock(&span_lock);

    // Find span
    Span* span = NULL;
    for (int i = 0; i < MAX_SPANS; i++) {
        if (spans[i].active && spans[i].span_id == span_id) {
            span = &spans[i];
            break;
        }
    }

    if (span == NULL) {
        pthread_mutex_unlock(&span_lock);
        return LOG_ERR_SPAN;
    }

    // Log span end
    char end_msg[256];
    snprintf(end_msg, sizeof(end_msg), "Span ended: %s", span->name);
    __log_with_field(LOG_LEVEL_DEBUG, end_msg, "trace_id", span->trace_id);

    // Mark span as inactive
    span->active = 0;

    pthread_mutex_unlock(&span_lock);
    return LOG_OK;
}

long __span_log(long span_id, long level, const char* msg) {
    if (msg == NULL) return 0;

    pthread_mutex_lock(&span_lock);

    // Find span
    Span* span = NULL;
    for (int i = 0; i < MAX_SPANS; i++) {
        if (spans[i].active && spans[i].span_id == span_id) {
            span = &spans[i];
            break;
        }
    }

    if (span == NULL) {
        pthread_mutex_unlock(&span_lock);
        __log_write(level, msg);  // Fallback to regular log
        return 0;
    }

    // Copy trace_id before unlocking
    char trace_id[MAX_TRACE_ID];
    strncpy(trace_id, span->trace_id, sizeof(trace_id));

    pthread_mutex_unlock(&span_lock);

    // Log with trace_id
    __log_with_field(level, msg, "trace_id", trace_id);
    return 0;
}

long __span_log_field(long span_id, long level, const char* msg, const char* key, const char* value) {
    if (msg == NULL || key == NULL || value == NULL) return 0;

    pthread_mutex_lock(&span_lock);

    // Find span
    Span* span = NULL;
    for (int i = 0; i < MAX_SPANS; i++) {
        if (spans[i].active && spans[i].span_id == span_id) {
            span = &spans[i];
            break;
        }
    }

    if (span == NULL) {
        pthread_mutex_unlock(&span_lock);
        __log_with_field(level, msg, key, value);  // Fallback
        return 0;
    }

    // Copy trace_id before unlocking
    char trace_id[MAX_TRACE_ID];
    strncpy(trace_id, span->trace_id, sizeof(trace_id));

    pthread_mutex_unlock(&span_lock);

    // Log with trace_id and field
    // Build fields string: "trace_id=...,key=value"
    char fields[MAX_LOG_LINE];
    snprintf(fields, sizeof(fields), "trace_id=%s,%s=%s", trace_id, key, value);
    __log_with_fields(level, msg, fields);
    return 0;
}

const char* __span_trace_id(long span_id) {
    pthread_mutex_lock(&span_lock);

    // Find span
    Span* span = NULL;
    for (int i = 0; i < MAX_SPANS; i++) {
        if (spans[i].active && spans[i].span_id == span_id) {
            span = &spans[i];
            break;
        }
    }

    if (span == NULL) {
        pthread_mutex_unlock(&span_lock);
        // Return empty string
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty;
    }

    // Allocate and copy trace_id
    size_t len = strlen(span->trace_id);
    char* result = (char*)malloc(len + 1);
    if (result) {
        memcpy(result, span->trace_id, len);
        result[len] = '\0';
    }

    pthread_mutex_unlock(&span_lock);
    return result;
}
