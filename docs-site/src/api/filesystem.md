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

### File Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `fs_unlink` | `F fs_unlink(path: str) -> i64` | Delete file |
| `fs_rename` | `F fs_rename(old_path: str, new_path: str) -> i64` | Rename file |

### Metadata

| Function | Signature | Description |
|----------|-----------|-------------|
| `fs_file_size` | `F fs_file_size(path: str) -> i64` | Get file size |
| `fs_file_exists` | `F fs_file_exists(path: str) -> i64` | Check if file exists |
| `fs_is_dir` | `F fs_is_dir(path: str) -> i64` | Check if path is directory |

## Usage

```vais
U std/filesystem

F main() -> i64 {
    fs_mkdir("output", 755)
    size := fs_file_size("data.txt")
    fs_rename("old.txt", "new.txt")
    0
}
```
