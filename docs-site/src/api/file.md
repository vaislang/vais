# File API Reference

> File I/O with memory-mapped files and advisory locks

## Import

```vais
U std/file
```

## Constants

### Seek Origin

| Constant | Value | Description |
|----------|-------|-------------|
| `SEEK_SET` | 0 | Beginning of file |
| `SEEK_CUR` | 1 | Current position |
| `SEEK_END` | 2 | End of file |

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
| `get_mode` | `F get_mode(&self) -> i64` | Get file mode (0=closed, 1=read, 2=write, 3=append) |
| `read_byte` | `F read_byte(&self) -> i64` | Read single byte (-1 on EOF) |
| `read_byte_opt` | `F read_byte_opt(&self) -> Option` | Read byte as Option |
| `read` | `F read(&self, buffer: i64, count: i64) -> i64` | Read bytes into buffer |
| `read_line` | `F read_line(&self, buffer: i64, max_len: i64) -> i64` | Read a line |
| `read_line_result` | `F read_line_result(&self, buffer: i64, max_len: i64) -> Result` | Read line with Result |
| `write_byte` | `F write_byte(&self, byte: i64) -> i64` | Write single byte |
| `write` | `F write(&self, buffer: i64, count: i64) -> i64` | Write bytes |
| `write_str` | `F write_str(&self, str: i64) -> i64` | Write null-terminated string |
| `flush` | `F flush(&self) -> i64` | Flush buffer |
| `sync` | `F sync(&self) -> i64` | Fsync data + metadata |
| `datasync` | `F datasync(&self) -> i64` | Sync data only (no metadata) |
| `seek` | `F seek(&self, offset: i64, origin: i64) -> i64` | Seek (0=start, 1=current, 2=end) |
| `tell` | `F tell(&self) -> i64` | Get current position |
| `eof` | `F eof(&self) -> i64` | Check end-of-file |
| `close` | `F close(&self) -> i64` | Close the file |
| `drop` | `F drop(&self) -> i64` | Alias for close (RAII) |

## Free Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `file_read_all` | `F file_read_all(path: i64, size_out: i64) -> i64` | Read entire file |
| `file_write_all` | `F file_write_all(path: i64, buffer: i64, size: i64) -> i64` | Write buffer to file |
| `file_append` | `F file_append(path: i64, buffer: i64, size: i64) -> i64` | Append to file |
| `file_exists` | `F file_exists(path: i64) -> i64` | Check if file exists |
| `file_read_all_result` | `F file_read_all_result(path: i64, size_out: i64) -> Result` | Read entire file with Result |
| `file_sync` | `F file_sync(path: i64) -> i64` | Sync file to disk by path |
| `dir_sync` | `F dir_sync(path: i64) -> i64` | Sync directory metadata |

## MappedFile Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `map` | `F map(fd: i64, len: i64, prot: i64, flags: i64, offset: i64) -> MappedFile` | Map file descriptor |
| `map_read` | `F map_read(fd: i64, len: i64) -> MappedFile` | Map for reading |
| `map_readwrite` | `F map_readwrite(fd: i64, len: i64) -> MappedFile` | Map for read-write |
| `is_valid` | `F is_valid(&self) -> i64` | Check if mapping is valid |
| `unmap` | `F unmap(&self) -> i64` | Unmap memory region |
| `sync` | `F sync(&self) -> i64` | Sync to disk |
| `sync_async` | `F sync_async(&self) -> i64` | Async sync to disk |
| `advise` | `F advise(&self, advice: i64) -> i64` | Advise kernel on access pattern |
| `read_byte` | `F read_byte(&self, offset: i64) -> i64` | Read byte at offset |
| `write_byte` | `F write_byte(&self, offset: i64, val: i64) -> i64` | Write byte at offset |

## FileLock Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `from_fd` | `F from_fd(fd: i64) -> FileLock` | Create from file descriptor |
| `open` | `F open(path: i64) -> FileLock` | Open file for locking |
| `is_valid` | `F is_valid(&self) -> i64` | Check if lock handle is valid |
| `lock_shared` | `F lock_shared(&self) -> i64` | Acquire shared (read) lock |
| `lock_exclusive` | `F lock_exclusive(&self) -> i64` | Acquire exclusive (write) lock |
| `try_lock_shared` | `F try_lock_shared(&self) -> i64` | Try shared lock (non-blocking) |
| `try_lock_exclusive` | `F try_lock_exclusive(&self) -> i64` | Try exclusive lock (non-blocking) |
| `unlock` | `F unlock(&self) -> i64` | Release lock |
| `is_locked` | `F is_locked(&self) -> i64` | Check if currently locked |
| `close` | `F close(&self) -> i64` | Close file and release lock |
| `drop` | `F drop(&self) -> i64` | Alias for close (RAII) |

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
