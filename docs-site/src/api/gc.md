# GC API Reference

> Optional garbage collector with mark-and-sweep collection

## Import

```vais
U std/gc
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `gc_init` | `F gc_init() -> i64` | Initialize GC (call once at start) |
| `gc_alloc` | `F gc_alloc(size: i64, type_id: i64) -> i64` | Allocate GC-managed memory |
| `gc_alloc_simple` | `F gc_alloc_simple(size: i64) -> i64` | Allocate with default type_id |
| `gc_add_root` | `F gc_add_root(ptr: i64) -> i64` | Register a root pointer |
| `gc_remove_root` | `F gc_remove_root(ptr: i64) -> i64` | Unregister a root pointer |
| `gc_collect` | `F gc_collect() -> i64` | Force mark-and-sweep collection |
| `gc_bytes_allocated` | `F gc_bytes_allocated() -> i64` | Total bytes allocated |
| `gc_objects_count` | `F gc_objects_count() -> i64` | Number of live objects |
| `gc_collections` | `F gc_collections() -> i64` | Number of collections run |
| `gc_set_threshold` | `F gc_set_threshold(threshold: i64) -> i64` | Set collection threshold |
| `gc_print_stats` | `F gc_print_stats() -> i64` | Print GC statistics |

## Usage

```vais
U std/gc

F main() -> i64 {
    gc_init()
    ptr := gc_alloc_simple(64)
    gc_add_root(ptr)
    # ... use ptr ...
    gc_collect()
    gc_print_stats()
    0
}
```
