# Log API Reference

> Structured logging and error tracing library with multiple output formats and span-based tracing

## Overview

The Log module provides production-grade logging with:
- Multiple log levels (TRACE, DEBUG, INFO, WARN, ERROR)
- Structured key-value fields for rich context
- JSON and text output formats
- File, stdout, stderr output targets
- Span-based tracing with unique IDs for request tracking
- Thread-safe logging via C runtime
- ISO8601 timestamps

## Constants

### Log Levels

| Constant | Value | Description |
|----------|-------|-------------|
| `LOG_LEVEL_TRACE` | 0 | Verbose tracing |
| `LOG_LEVEL_DEBUG` | 1 | Debug information |
| `LOG_LEVEL_INFO` | 2 | Informational messages |
| `LOG_LEVEL_WARN` | 3 | Warning messages |
| `LOG_LEVEL_ERROR` | 4 | Error messages |

### Output Targets

| Constant | Value | Description |
|----------|-------|-------------|
| `LOG_OUTPUT_STDOUT` | 0 | Standard output |
| `LOG_OUTPUT_STDERR` | 1 | Standard error |
| `LOG_OUTPUT_FILE` | 2 | Log file |

### Output Formats

| Constant | Value | Description |
|----------|-------|-------------|
| `LOG_FORMAT_TEXT` | 0 | Human-readable text |
| `LOG_FORMAT_JSON` | 1 | JSON format |

### Error Codes

| Constant | Value | Description |
|----------|-------|-------------|
| `LOG_OK` | 0 | Success |
| `LOG_ERR_INIT` | -1 | Initialization failed |
| `LOG_ERR_FILE` | -2 | File error |
| `LOG_ERR_INVALID_LEVEL` | -3 | Invalid log level |
| `LOG_ERR_WRITE` | -7 | Write error |

## Initialization Functions

### log_init

```vais
F log_init(level: i64) -> i64
```

Initialize the global logger with a specified level.

**Parameters:**
- `level`: Minimum log level to display

**Returns:** `LOG_OK` on success, error code on failure

---

### log_set_level

```vais
F log_set_level(level: i64) -> i64
```

Set the global log level.

---

### log_set_output

```vais
F log_set_output(target: i64) -> i64
```

Set the output target (stdout/stderr/file).

---

### log_set_file

```vais
F log_set_file(path: str) -> i64
```

Set the log file path (use before setting output to `LOG_OUTPUT_FILE`).

---

### log_set_format

```vais
F log_set_format(format: i64) -> i64
```

Set the output format (text or JSON).

## Basic Logging Functions

### log_trace / log_debug / log_info / log_warn / log_error

```vais
F log_trace(msg: str) -> i64
F log_debug(msg: str) -> i64
F log_info(msg: str) -> i64
F log_warn(msg: str) -> i64
F log_error(msg: str) -> i64
```

Log a message at the specified level.

**Parameters:**
- `msg`: Message to log

**Returns:** `0` on success

## Structured Logging

### log_with_field

```vais
F log_with_field(level: i64, msg: str, key: str, value: str) -> i64
```

Log a message with a single structured field.

**Parameters:**
- `level`: Log level
- `msg`: Message
- `key`: Field name
- `value`: Field value

**Example:**
```vais
log_with_field(LOG_LEVEL_INFO, "User login", "user_id", "12345")
```

---

### log_with_fields

```vais
F log_with_fields(level: i64, msg: str, fields: str) -> i64
```

Log a message with multiple structured fields (comma-separated key=value pairs).

**Parameters:**
- `level`: Log level
- `msg`: Message
- `fields`: Comma-separated key=value pairs

**Example:**
```vais
log_with_fields(LOG_LEVEL_ERROR, "Request failed", "status=500,method=GET,path=/api/users")
```

## Span-Based Tracing

### span_start

```vais
F span_start(name: str) -> i64
```

Start a new span for request tracing.

**Parameters:**
- `name`: Span name

**Returns:** Unique span ID (positive integer) or negative error code

---

### span_end

```vais
F span_end(span_id: i64) -> i64
```

End a span and clean up its state.

**Parameters:**
- `span_id`: Span ID to end

**Returns:** `LOG_OK` on success

---

### span_log

```vais
F span_log(span_id: i64, level: i64, msg: str) -> i64
```

Log a message within a span (automatically includes span's trace_id).

**Parameters:**
- `span_id`: Span ID
- `level`: Log level
- `msg`: Message

---

### span_log_field

```vais
F span_log_field(span_id: i64, level: i64, msg: str, key: str, value: str) -> i64
```

Log a message with a field within a span.

---

### span_trace_id

```vais
F span_trace_id(span_id: i64) -> str
```

Get the trace ID for a span.

**Parameters:**
- `span_id`: Span ID

**Returns:** Trace ID string or empty string if span not found

## Usage Examples

### Basic Logging

```vais
# Initialize logger
log_init(LOG_LEVEL_INFO)

# Log messages
log_info("Server started")
log_warn("Low memory")
log_error("Failed to connect")
```

### Structured Logging

```vais
log_init(LOG_LEVEL_INFO)

# Log with single field
log_with_field(LOG_LEVEL_INFO, "User login", "user_id", "12345")

# Log with multiple fields
log_with_fields(LOG_LEVEL_ERROR, "Request failed", "status=500,url=/api")
```

### JSON Format

```vais
log_init(LOG_LEVEL_INFO)
log_set_format(LOG_FORMAT_JSON)

log_info("Request completed")
# Output: {"timestamp":"2026-02-04T10:30:00Z","level":"INFO","msg":"Request completed"}
```

### File Output

```vais
log_init(LOG_LEVEL_DEBUG)
log_set_output(LOG_OUTPUT_FILE)
log_set_file("/var/log/app.log")

log_debug("Debug info written to file")
```

### Span Tracing

```vais
# Start a span for request tracking
span_id := span_start("handle_request")

# Log within the span (includes trace_id)
span_log(span_id, LOG_LEVEL_INFO, "Processing")

# Do work...
span_log(span_id, LOG_LEVEL_DEBUG, "Step completed")

# End the span
span_end(span_id)
```
