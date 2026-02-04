# Arena API Reference

> Region-based memory allocator - fast batch allocation, single free

## Import

```vais
U std/arena
```

## Struct

```vais
S Arena {
    chunks: i64,       # Chunk pointer list
    chunk_count: i64,
    chunk_size: i64,   # Size per chunk (default 64KB)
    current: i64,      # Current chunk
    offset: i64        # Offset in current chunk
}
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> Arena` | Create with 64KB chunks |
| `with_chunk_size` | `F with_chunk_size(size: i64) -> Arena` | Custom chunk size |
| `alloc` | `F alloc(&self, size: i64) -> i64` | Allocate bytes |
| `alloc_zeroed` | `F alloc_zeroed(&self, size: i64) -> i64` | Allocate zero-initialized bytes |
| `alloc_array` | `F alloc_array(&self, count: i64, item_size: i64) -> i64` | Allocate array |
| `grow` | `F grow(&self) -> i64` | Grow arena by adding new chunk |
| `total_allocated` | `F total_allocated(&self) -> i64` | Get total allocated bytes |
| `total_capacity` | `F total_capacity(&self) -> i64` | Get total capacity |
| `reset` | `F reset(&self) -> i64` | Reset (reuse memory) |
| `drop` | `F drop(&self) -> i64` | Free all chunks |

## Free Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `arena_new` | `F arena_new() -> Arena` | Create arena (convenience) |
| `arena_alloc` | `F arena_alloc(arena: Arena, size: i64) -> i64` | Allocate (convenience) |
| `arena_reset` | `F arena_reset(arena: Arena) -> i64` | Reset (convenience) |
| `arena_drop` | `F arena_drop(arena: Arena) -> i64` | Drop (convenience) |

## Usage

```vais
U std/arena

F main() -> i64 {
    a := Arena::new()
    ptr1 := a.alloc(64)   # Fast allocation
    ptr2 := a.alloc(128)
    a.drop()               # Frees everything at once
    0
}
```
