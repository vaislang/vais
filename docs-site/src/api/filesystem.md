# Filesystem API Reference

> POSIX filesystem operations (mkdir, rename, stat, etc.)

## Import

```vais
U std/filesystem
```

## Functions

### Directory Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `fs_mkdir` | `F fs_mkdir(path: str, mode: i64) -> i64` | Create directory |
| `fs_rmdir` | `F fs_rmdir(path: str) -> i64` | Remove directory |
| `fs_chdir` | `F fs_chdir(path: str) -> i64` | Change directory |
| `fs_getcwd` | `F fs_getcwd() -> str` | Get current working directory |

### File Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `fs_unlink` | `F fs_unlink(path: str) -> i64` | Delete file |
| `fs_rename` | `F fs_rename(old_path: str, new_path: str) -> i64` | Rename file |

### Metadata

| Function | Signature | Description |
|----------|-----------|-------------|
| `fs_file_size` | `F fs_file_size(path: str) -> i64` | Get file size in bytes |
| `fs_mtime` | `F fs_mtime(path: str) -> i64` | Get modification time (Unix timestamp) |

## Usage

```vais
U std/filesystem

F main() -> i64 {
    # Directory operations
    fs_mkdir("output", 755)
    cwd := fs_getcwd()

    # File operations
    size := fs_file_size("data.txt")
    mtime := fs_mtime("data.txt")
    fs_rename("old.txt", "new.txt")

    # Cleanup
    fs_unlink("temp.txt")
    fs_rmdir("output")
    0
}
```
