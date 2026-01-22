# vais-gc

Optional garbage collector for the Vais programming language.

## Overview

`vais-gc` provides automatic memory management for Vais, making it easier to write safe code in REPL and scripting environments. It uses a **Mark-and-Sweep** algorithm with conservative stack scanning.

## Features

- **Mark-and-Sweep GC**: Simple and reliable garbage collection
- **Conservative Scanning**: Automatically detects pointers on the stack
- **Configurable Threshold**: Control when GC triggers
- **C FFI**: Seamless integration with LLVM-generated code
- **Zero Runtime Overhead**: Only active when enabled

## Usage

### Enabling GC Mode

Compile with the `--gc` flag:

```bash
vaisc build my_program.vais --gc
vaisc build my_program.vais --gc --gc-threshold 2097152  # 2MB threshold
```

### In Vais Code

```vais
U std/gc

F main() -> i64 {
    # Initialize GC
    gc_init()

    # Allocate GC-managed memory
    ptr := gc_alloc(1024, 0)  # 1KB allocation

    # GC automatically frees unreachable objects

    # Force collection
    gc_collect()

    # View statistics
    gc_print_stats()

    0
}
```

### Using #[gc] Attribute (Future)

In the future, you'll be able to use the `#[gc]` attribute on functions:

```vais
#[gc]
F process_data() -> i64 {
    # All allocations in this function are GC-managed
    data := Vec.new()
    data.push(42)
    # No need to call free() - GC handles it
    0
}
```

## API Reference

### Initialization

- `gc_init()` - Initialize the GC system (call once at program start)
- `gc_set_threshold(bytes)` - Set allocation threshold for automatic collection

### Allocation

- `gc_alloc(size, type_id)` - Allocate GC-managed memory
- `gc_alloc_simple(size)` - Allocate with default type_id

### Root Management

- `gc_add_root(ptr)` - Register a pointer as GC root
- `gc_remove_root(ptr)` - Unregister a GC root

### Collection

- `gc_collect()` - Force garbage collection

### Statistics

- `gc_bytes_allocated()` - Get current bytes allocated
- `gc_objects_count()` - Get number of live objects
- `gc_collections()` - Get number of collections performed
- `gc_print_stats()` - Print GC statistics to stdout
- `gc_stats()` - Get statistics as a struct

## Implementation Details

### Mark-and-Sweep Algorithm

1. **Mark Phase**: Starting from roots, mark all reachable objects
2. **Sweep Phase**: Free all unmarked objects

### Conservative Scanning

The GC conservatively scans object data for potential pointers. Any value that looks like a valid pointer to a GC object is treated as a pointer.

### Performance

- **Allocation**: O(1) allocation time
- **Collection**: O(n) where n = number of objects
- **Memory overhead**: ~24 bytes per object (header)

### Thread Safety

Currently, the GC uses global locks and is not designed for concurrent access. Thread-local GC heaps are planned for future versions.

## Examples

See `/examples/gc_test.vais` for comprehensive tests covering:

- Basic allocation
- GC statistics
- Manual collection
- Root registration
- Threshold behavior
- Large allocations
- Memory stress testing

See `/examples/gc_vec_test.vais` for a GC-managed vector implementation.

## Testing

Run the test suite:

```bash
cargo test -p vais-gc
```

## Future Enhancements

- [ ] Incremental collection
- [ ] Generational GC
- [ ] Thread-local heaps
- [ ] Finalizers
- [ ] Weak references
- [ ] Compaction

## License

MIT
