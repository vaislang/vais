# File API Reference

> File I/O with memory-mapped files and advisory locks

## Import

```vais
U std/file
```

## Structs

### File

```vais
S File { handle: i64, mode: i64 }
```

### MappedFile

```vais
S MappedFile { addr: i64, len: i64 }
```

### FileLock

```vais
S FileLock { fd: i64, locked: i64 }
```

## File Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `open_read` | `F open_read(path: i64) -> File` | Open for reading |
| `open_write` | `F open_write(path: i64) -> File` | Open for writing (creates/truncates) |
| `open_append` | `F open_append(path: i64) -> File` | Open for appending |
| `is_open` | `F is_open(&self) -> i64` | Check if file is open |
| `read_byte` | `F read_byte(&self) -> i64` | Read single byte (-1 on EOF) |
| `read` | `F read(&self, buffer: i64, count: i64) -> i64` | Read bytes into buffer |
| `read_line` | `F read_line(&self, buffer: i64, max_len: i64) -> i64` | Read a line |
| `write_byte` | `F write_byte(&self, byte: i64) -> i64` | Write single byte |
| `write` | `F write(&self, buffer: i64, count: i64) -> i64` | Write bytes |
| `write_str` | `F write_str(&self, str: i64) -> i64` | Write null-terminated string |
| `flush` | `F flush(&self) -> i64` | Flush buffer |
| `sync` | `F sync(&self) -> i64` | Fsync data + metadata |
| `seek` | `F seek(&self, offset: i64, origin: i64) -> i64` | Seek (0=start, 1=current, 2=end) |
| `tell` | `F tell(&self) -> i64` | Get current position |
| `eof` | `F eof(&self) -> i64` | Check end-of-file |
| `close` | `F close(&self) -> i64` | Close the file |

## Free Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `file_read_all` | `F file_read_all(path: i64, size_out: i64) -> i64` | Read entire file |
| `file_write_all` | `F file_write_all(path: i64, buffer: i64, size: i64) -> i64` | Write buffer to file |
| `file_append` | `F file_append(path: i64, buffer: i64, size: i64) -> i64` | Append to file |
| `file_exists` | `F file_exists(path: i64) -> i64` | Check if file exists |

## Usage

```vais
U std/file

F main() -> i64 {
    f := File.open_write("output.txt")
    f.write_str("Hello, file!")
    f.close()

    exists := file_exists("output.txt")  # 1
    0
}
```
