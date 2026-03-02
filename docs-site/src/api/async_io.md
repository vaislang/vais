# Async I/O API Reference

> Asynchronous file operations with buffering support

## Import

```vais
U std/async_io
```

## Overview

The `async_io` module provides asynchronous file I/O with buffered reading and writing. It wraps POSIX file operations (`open`, `read`, `write`, `close`, `lseek`) and provides `AsyncFile`, `AsyncFileReader`, and `AsyncFileWriter` for structured file access.

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `ASYNC_READ` | 0 | Open for reading |
| `ASYNC_WRITE` | 577 | Open for writing (create/truncate) |
| `ASYNC_APPEND` | 521 | Open for appending (create) |
| `ASYNC_BUF_SIZE` | 4096 | Default buffer size |
| `ASYNC_LINE_BUF_SIZE` | 1024 | Line buffer size |

## Structs

### `AsyncFile`

```vais
S AsyncFile {
    fd: i64,
    path: str,
    mode: i64,
    is_open: i64
}
```

Async file handle.

**Methods:**

- `open(path: str, mode: i64) -> AsyncFile` -- Open a file (0=read, 1=write, 2=append)
- `read(@, buf: i64, len: i64) -> i64` -- Read data into buffer
- `write(@, buf: i64, len: i64) -> i64` -- Write data from buffer
- `close(@) -> i64` -- Close the file
- `read_all(@) -> str` -- Read entire file content as string
- `write_all(@, data: str) -> i64` -- Write entire string to file

### `AsyncFileReader`

```vais
S AsyncFileReader {
    file: AsyncFile,
    buffer: i64,
    buf_pos: i64,
    buf_len: i64
}
```

Buffered line reader for async files.

**Methods:**

- `new(file: AsyncFile) -> AsyncFileReader` -- Create a reader
- `read_line(@) -> str` -- Read next line (newline-delimited)
- `has_next(@) -> i64` -- Check if more data is available
- `close(@) -> i64` -- Clean up and close

### `AsyncFileWriter`

```vais
S AsyncFileWriter {
    file: AsyncFile,
    buffer: i64,
    buf_pos: i64,
    capacity: i64
}
```

Buffered writer for async files.

**Methods:**

- `new(file: AsyncFile) -> AsyncFileWriter` -- Create a writer
- `write(@, data: str) -> i64` -- Write data to buffer
- `flush(@) -> i64` -- Flush buffer to file
- `close(@) -> i64` -- Flush, clean up, and close

## Helper Functions

### async_read_file

```vais
F async_read_file(path: str) -> str
```

Read an entire file and return its content as a string. Returns `""` on error.

### async_write_file

```vais
F async_write_file(path: str, content: str) -> i64
```

Write a string to a file (creates/overwrites). Returns bytes written or `-1` on error.

### async_copy_file

```vais
F async_copy_file(src: str, dst: str) -> i64
```

Copy a file from `src` to `dst`. Returns total bytes copied or `-1` on error.

## Example

```vais
U std/async_io

F main() {
    # Write a file
    async_write_file("/tmp/hello.txt", "Hello, World!")

    # Read it back
    content := async_read_file("/tmp/hello.txt")

    # Buffered reading
    file := AsyncFile.open("/tmp/hello.txt", 0)
    reader := AsyncFileReader.new(file)
    line := reader.read_line()
    reader.close()
}
```
