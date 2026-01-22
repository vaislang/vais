# GC Quick Reference

## Enable GC Mode

```bash
# Enable GC with default settings
vaisc build myprogram.vais --gc

# Enable GC with custom threshold (2MB)
vaisc build myprogram.vais --gc --gc-threshold 2097152
```

## Import GC Module

```vais
U std/gc
```

## Initialize GC

```vais
F main() -> i64 {
    gc_init()  # Call once at program start
    # ...
}
```

## Allocate Memory

```vais
# Allocate 1024 bytes with type_id 0
ptr := gc_alloc(1024, 0)

# Allocate with simple API
ptr := gc_alloc_simple(1024)
```

## Root Management

```vais
# Register a root (prevents collection)
gc_add_root(ptr)

# Unregister a root (allows collection)
gc_remove_root(ptr)
```

## Trigger Collection

```vais
# Force garbage collection
gc_collect()
```

## Configuration

```vais
# Set threshold to 4KB
gc_set_threshold(4096)
```

## Statistics

```vais
# Get current allocated bytes
bytes := gc_bytes_allocated()

# Get number of live objects
count := gc_objects_count()

# Get collection count
collections := gc_collections()

# Print all statistics
gc_print_stats()

# Get stats as struct
stats := gc_stats()
printf("Allocated: %ld bytes\n", stats.bytes_allocated)
```

## Complete Example

```vais
U std/gc

F main() -> i64 {
    # Initialize
    gc_init()
    gc_set_threshold(1048576)  # 1MB

    # Allocate
    i := 0
    L i < 10 {
        ptr := gc_alloc(1024, 0)
        # Use ptr...
        i = i + 1
    }

    # Check stats
    printf("Allocated: %ld bytes\n", gc_bytes_allocated())
    printf("Objects: %ld\n", gc_objects_count())

    # Force collection
    gc_collect()

    # Final stats
    gc_print_stats()

    0
}
```

## GC-Managed Vector Example

```vais
S GcVec {
    data: i64,  # GC-managed pointer
    len: i64,
    cap: i64
}

X GcVec {
    F new() -> GcVec {
        data_ptr := gc_alloc(32, 100)
        gc_add_root(data_ptr)  # Prevent collection
        GcVec { data: data_ptr, len: 0, cap: 4 }
    }

    F drop(&self) -> i64 {
        gc_remove_root(self.data)  # Allow collection
        0
    }
}
```

## Default Settings

- **Threshold**: 1 MB (1048576 bytes)
- **Algorithm**: Mark-and-Sweep
- **Scanning**: Conservative

## API Summary

| Function | Description | Returns |
|----------|-------------|---------|
| `gc_init()` | Initialize GC | i64 |
| `gc_alloc(size, type_id)` | Allocate memory | i64 (ptr) |
| `gc_alloc_simple(size)` | Allocate with default type | i64 (ptr) |
| `gc_add_root(ptr)` | Register root | i64 |
| `gc_remove_root(ptr)` | Unregister root | i64 |
| `gc_collect()` | Force collection | i64 |
| `gc_set_threshold(bytes)` | Set threshold | i64 |
| `gc_bytes_allocated()` | Get allocated bytes | i64 |
| `gc_objects_count()` | Get object count | i64 |
| `gc_collections()` | Get collection count | i64 |
| `gc_print_stats()` | Print statistics | i64 |
| `gc_stats()` | Get stats struct | GcStats |

## Troubleshooting

### Memory keeps growing

- Lower the threshold: `gc_set_threshold(smaller_value)`
- Force collection manually: `gc_collect()`
- Check for memory leaks (rooted objects)

### Objects collected too early

- Register as root: `gc_add_root(ptr)`
- Keep references on stack
- Check scope lifetime

### Slow performance

- Increase threshold: `gc_set_threshold(larger_value)`
- Reduce allocation frequency
- Consider manual memory management for hot paths

## See Also

- `/crates/vais-gc/README.md` - Detailed GC documentation
- `/docs/gc-implementation.md` - Implementation details
- `/examples/gc_test.vais` - Comprehensive tests
- `/examples/gc_vec_test.vais` - Vector example
