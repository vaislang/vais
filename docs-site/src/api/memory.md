# Memory API Reference

> Low-level memory operations (memset, memcpy, memmove, memcmp)

## Import

```vais
U std/memory
```

## Functions

### Fill Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `mem_set` | `F mem_set(dest: i64, value: i64, n: i64) -> i64` | Fill with byte value |
| `mem_zero` | `F mem_zero(dest: i64, n: i64) -> i64` | Fill with zeros |
| `mem_fill_i64` | `F mem_fill_i64(dest: i64, value: i64, count: i64) -> i64` | Fill with i64 pattern |

### Copy Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `mem_copy` | `F mem_copy(dest: i64, src: i64, n: i64) -> i64` | Copy bytes (non-overlapping) |
| `mem_move` | `F mem_move(dest: i64, src: i64, n: i64) -> i64` | Copy bytes (overlapping safe) |
| `mem_copy_i64` | `F mem_copy_i64(dest: i64, src: i64, count: i64) -> i64` | Copy i64 values |

### Compare Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `mem_compare` | `F mem_compare(s1: i64, s2: i64, n: i64) -> i64` | Compare memory regions |
| `mem_equal` | `F mem_equal(s1: i64, s2: i64, n: i64) -> i64` | Check equality |

## Usage

```vais
U std/memory

F main() -> i64 {
    buf := malloc(256)
    mem_zero(buf, 256)
    mem_set(buf, 65, 10)  # Fill first 10 bytes with 'A'
    free(buf)
    0
}
```
