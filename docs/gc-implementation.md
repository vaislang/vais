# Garbage Collection Implementation

This document describes the implementation of optional garbage collection in Vais.

## Architecture

### Components

1. **vais-gc** (Rust crate)
   - Core GC implementation
   - Mark-and-Sweep algorithm
   - C FFI exports

2. **std/gc.vais** (Vais module)
   - Vais API for GC functions
   - High-level wrappers
   - Helper utilities

3. **Codegen Integration**
   - GC-aware allocation
   - Automatic root registration
   - Runtime function calls

4. **CLI Support**
   - `--gc` flag to enable GC mode
   - `--gc-threshold` to configure collection threshold

## Design Decisions

### Why Mark-and-Sweep?

1. **Simplicity**: Easy to understand and implement
2. **Reliability**: Well-tested algorithm
3. **No cyclic issues**: Handles cycles naturally
4. **Low complexity**: Suitable for prototypes and REPL

### Conservative vs. Precise

We use **conservative scanning** because:

1. No need to modify LLVM IR generation extensively
2. Works with existing allocations
3. Minimal changes to type system
4. Good enough for scripting/REPL use cases

Tradeoffs:

- May retain some garbage (false positives)
- Acceptable for non-production environments

## Memory Layout

### GC Object Header

```rust
struct GcObjectHeader {
    size: usize,        // Object size (excluding header)
    marked: bool,       // Mark bit for GC
    ref_count: usize,   // Optional reference counting
    type_id: u32,       // Type identifier (for debugging)
}
```

Total header size: 24 bytes (on 64-bit systems)

### Object Layout

```
[Header: 24 bytes][User Data: N bytes]
^                 ^
|                 |
|                 +-- Returned to user
+-- Managed by GC
```

## Allocation Flow

### Without GC

```vais
ptr := malloc(100)
# Use ptr
free(ptr)
```

LLVM IR:
```llvm
%1 = call i8* @malloc(i64 100)
# ...
call void @free(i8* %1)
```

### With GC

```vais
ptr := gc_alloc(100, 0)
# Use ptr
# No free() needed - GC handles it
```

LLVM IR:
```llvm
%1 = call i8* @vais_gc_alloc(i64 100, i32 0)
# ...
# Automatic collection when threshold reached
```

## Collection Algorithm

### Mark Phase

```rust
fn mark(&mut self) {
    // Clear all marks
    for obj in self.objects.values_mut() {
        obj.header.marked = false;
    }

    // Mark from roots
    for root in &self.roots {
        self.mark_object(root.ptr);
    }
}
```

### Sweep Phase

```rust
fn sweep(&mut self) {
    let mut to_remove = Vec::new();

    for (ptr, obj) in &self.objects {
        if !obj.header.marked {
            to_remove.push(*ptr);
            self.bytes_allocated -= obj.header.size;
            self.objects_freed += 1;
        }
    }

    for ptr in to_remove {
        self.objects.remove(&ptr);
    }
}
```

## Root Management

### Automatic Roots

In GC mode, local variables are automatically registered as roots:

```vais
#[gc]
F example() -> i64 {
    x := gc_alloc(100, 0)  # Automatically registered as root
    # ...
    0  # x automatically unregistered on return
}
```

### Manual Roots

For global data or long-lived pointers:

```vais
V global_ptr: i64 = 0

F init() -> i64 {
    global_ptr = gc_alloc(1000, 0)
    gc_add_root(global_ptr)  # Prevent collection
    0
}

F cleanup() -> i64 {
    gc_remove_root(global_ptr)  # Allow collection
    0
}
```

## Configuration

### Threshold

Controls when automatic collection triggers:

```vais
gc_set_threshold(2097152)  # 2 MB
```

When `bytes_allocated >= threshold`, GC automatically runs.

### Default Settings

- Threshold: 1 MB (1048576 bytes)
- Conservative scanning: Enabled
- Stack scanning: Not yet implemented (manual roots only)

## Performance Characteristics

### Time Complexity

- **Allocation**: O(1)
- **Collection**: O(n) where n = number of objects
- **Mark**: O(m) where m = number of reachable objects

### Space Complexity

- **Header overhead**: 24 bytes per object
- **Root set**: O(r) where r = number of roots
- **Total**: O(n + r)

### Benchmark Results

From `gc_test.vais`:

```
Stress test: 100 objects, 256 bytes each
- Allocations: 100
- Collections: ~6
- Time: ~5ms
- Memory: ~25KB
```

## Integration Examples

### Example 1: Simple Allocation

```vais
U std/gc

F main() -> i64 {
    gc_init()

    # Allocate
    ptr := gc_alloc(1024, 1)

    # GC stats
    printf("Allocated: %ld bytes\n", gc_bytes_allocated())

    0
}
```

### Example 2: Vector with GC

```vais
S GcVec {
    data: i64,  # GC-managed pointer
    len: i64,
    cap: i64
}

X GcVec {
    F new() -> GcVec {
        data_ptr := gc_alloc(32, 100)
        gc_add_root(data_ptr)
        GcVec { data: data_ptr, len: 0, cap: 4 }
    }

    F drop(&self) -> i64 {
        gc_remove_root(self.data)
        0
    }
}
```

### Example 3: Automatic Collection

```vais
F process_large_data() -> i64 {
    gc_set_threshold(4096)  # Low threshold

    i := 0
    L i < 100 {
        # Each allocation might trigger GC
        temp := gc_alloc(1024, 0)
        # Process temp
        i = i + 1
    }
    # Unreferenced objects automatically collected
    0
}
```

## Testing

### Unit Tests

```bash
cargo test -p vais-gc
```

Tests cover:
- Basic allocation
- Collection
- Root preservation
- Threshold behavior
- Stress scenarios

### Integration Tests

```bash
vaisc build examples/gc_test.vais --gc && ./examples/gc_test
vaisc build examples/gc_vec_test.vais --gc && ./examples/gc_vec_test
```

## Future Work

### Short Term

- [ ] Stack scanning for automatic root detection
- [ ] Improved type information
- [ ] Finalizers for cleanup

### Medium Term

- [ ] Incremental collection
- [ ] Write barriers for generational GC
- [ ] Parallel marking

### Long Term

- [ ] Concurrent GC
- [ ] Compaction
- [ ] Thread-local heaps

## References

1. "The Garbage Collection Handbook" by Jones et al.
2. Boehm-Demers-Weiser Conservative GC
3. LLVM Stack Map documentation
4. Rust's memory model

## See Also

- `/crates/vais-gc/README.md` - GC crate documentation
- `/std/gc.vais` - Vais GC API
- `/examples/gc_test.vais` - Comprehensive tests
