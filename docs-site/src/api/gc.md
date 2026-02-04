# GC API Reference

> Optional garbage collector with mark-and-sweep and generational collection strategies

## Import

```vais
U std/gc
```

## Overview

The GC module provides two garbage collection implementations:
1. **Basic GC** - Simple mark-and-sweep collector for general use
2. **Generational GC** - Two-generation collector optimized for short-lived objects

Both collectors provide automatic memory management with explicit root registration and configurable collection thresholds.

## Basic GC API

### Core Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `gc_init` | `F gc_init() -> i64` | Initialize GC (call once at start) |
| `gc_alloc` | `F gc_alloc(size: i64, type_id: i64) -> i64` | Allocate GC-managed memory |
| `gc_alloc_simple` | `F gc_alloc_simple(size: i64) -> i64` | Allocate with default type_id (0) |
| `gc_add_root` | `F gc_add_root(ptr: i64) -> i64` | Register a root pointer |
| `gc_remove_root` | `F gc_remove_root(ptr: i64) -> i64` | Unregister a root pointer |
| `gc_collect` | `F gc_collect() -> i64` | Force mark-and-sweep collection |

### Statistics Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `gc_bytes_allocated` | `F gc_bytes_allocated() -> i64` | Total bytes currently allocated |
| `gc_objects_count` | `F gc_objects_count() -> i64` | Number of live objects |
| `gc_collections` | `F gc_collections() -> i64` | Number of collections performed |
| `gc_set_threshold` | `F gc_set_threshold(threshold: i64) -> i64` | Set collection threshold (default: 1MB) |
| `gc_print_stats` | `F gc_print_stats() -> i64` | Print statistics to stdout |

### GcStats Struct

Statistics snapshot for basic GC.

```vais
S GcStats {
    bytes_allocated: i64,
    objects_count: i64,
    collections: i64,
    threshold: i64
}
```

| Function | Signature | Description |
|----------|-----------|-------------|
| `gc_stats` | `F gc_stats() -> GcStats` | Get statistics as struct |

### GcRootGuard Struct

RAII-style automatic root registration/unregistration.

```vais
S GcRootGuard {
    ptr: i64
}
```

#### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(ptr: i64) -> GcRootGuard` | Create guard and register root |
| `drop` | `F drop(&self) -> i64` | Unregister root automatically |

### Scoped GC Helpers

| Function | Signature | Description |
|----------|-----------|-------------|
| `gc_scope_begin` | `F gc_scope_begin() -> i64` | Mark scope start (returns current bytes) |
| `gc_scope_end` | `F gc_scope_end(start_bytes: i64) -> i64` | End scope and force collection |

## Generational GC API

### Core Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `gen_gc_init` | `F gen_gc_init() -> i64` | Initialize generational GC |
| `gen_gc_alloc` | `F gen_gc_alloc(size: i64, type_id: i64) -> i64` | Allocate in young generation |
| `gen_gc_add_root` | `F gen_gc_add_root(ptr: i64) -> i64` | Register root pointer |
| `gen_gc_remove_root` | `F gen_gc_remove_root(ptr: i64) -> i64` | Unregister root pointer |
| `gen_gc_write_barrier` | `F gen_gc_write_barrier(source: i64, old_target: i64, new_target: i64) -> i64` | Notify GC of pointer modification |

### Collection Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `gen_gc_collect_minor` | `F gen_gc_collect_minor() -> i64` | Collect young generation only (fast) |
| `gen_gc_collect_major` | `F gen_gc_collect_major() -> i64` | Collect both generations (thorough) |
| `gen_gc_collect_full` | `F gen_gc_collect_full() -> i64` | Minor + major collection |

### Statistics Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `gen_gc_young_objects` | `F gen_gc_young_objects() -> i64` | Number of young generation objects |
| `gen_gc_old_objects` | `F gen_gc_old_objects() -> i64` | Number of old generation objects |
| `gen_gc_minor_collections` | `F gen_gc_minor_collections() -> i64` | Number of minor GCs performed |
| `gen_gc_major_collections` | `F gen_gc_major_collections() -> i64` | Number of major GCs performed |
| `gen_gc_total_promoted` | `F gen_gc_total_promoted() -> i64` | Total objects promoted to old gen |
| `gen_gc_print_stats` | `F gen_gc_print_stats() -> i64` | Print statistics to stdout |

### Configuration Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `gen_gc_set_young_threshold` | `F gen_gc_set_young_threshold(threshold: i64) -> i64` | Set young gen threshold (default: 256KB) |
| `gen_gc_set_old_threshold` | `F gen_gc_set_old_threshold(threshold: i64) -> i64` | Set old gen threshold (default: 4MB) |
| `gen_gc_set_promotion_age` | `F gen_gc_set_promotion_age(age: i64) -> i64` | Set promotion age (default: 3) |

### Tuning Presets

| Function | Signature | Description |
|----------|-----------|-------------|
| `gen_gc_tune_low_latency` | `F gen_gc_tune_low_latency() -> i64` | Optimize for low pause times |
| `gen_gc_tune_throughput` | `F gen_gc_tune_throughput() -> i64` | Optimize for throughput |
| `gen_gc_tune_balanced` | `F gen_gc_tune_balanced() -> i64` | Balanced default settings |

#### Preset Details

| Preset | Young Threshold | Old Threshold | Promotion Age | Use Case |
|--------|----------------|---------------|---------------|----------|
| Low Latency | 64KB | 2MB | 2 | Interactive apps, real-time |
| Throughput | 1MB | 16MB | 5 | Batch processing, high allocation rate |
| Balanced | 256KB | 4MB | 3 | General-purpose applications |

### GenGcStats Struct

Statistics snapshot for generational GC.

```vais
S GenGcStats {
    young_objects: i64,
    old_objects: i64,
    minor_collections: i64,
    major_collections: i64,
    total_promoted: i64
}
```

| Function | Signature | Description |
|----------|-----------|-------------|
| `gen_gc_stats` | `F gen_gc_stats() -> GenGcStats` | Get generational GC statistics |

## Usage Examples

### Basic GC

```vais
U std/gc

F main() -> i64 {
    gc_init()

    # Allocate GC-managed memory
    ptr := gc_alloc_simple(64)
    gc_add_root(ptr)

    # Use the memory
    # ...

    # Force collection
    gc_collect()

    # Print statistics
    gc_print_stats()

    # Remove root when done
    gc_remove_root(ptr)
    0
}
```

### GC Statistics

```vais
U std/gc

F main() -> i64 {
    gc_init()

    ptr1 := gc_alloc_simple(100)
    ptr2 := gc_alloc_simple(200)
    gc_add_root(ptr1)
    gc_add_root(ptr2)

    stats := gc_stats()
    # stats.bytes_allocated
    # stats.objects_count
    # stats.collections

    gc_collect()

    gc_remove_root(ptr1)
    gc_remove_root(ptr2)
    0
}
```

### GC Root Guard (RAII)

```vais
U std/gc

F main() -> i64 {
    gc_init()

    ptr := gc_alloc_simple(128)
    guard := GcRootGuard::new(ptr)

    # Use ptr...
    # guard automatically unregisters when it goes out of scope

    guard.drop()  # Explicit cleanup
    0
}
```

### GC Scoped Blocks

```vais
U std/gc

F main() -> i64 {
    gc_init()

    start := gc_scope_begin()

    # Temporary allocations
    temp1 := gc_alloc_simple(50)
    temp2 := gc_alloc_simple(75)

    # Force collection at scope exit
    gc_scope_end(start)

    0
}
```

### Custom Threshold

```vais
U std/gc

F main() -> i64 {
    gc_init()

    # Set to 4MB threshold (more collections)
    gc_set_threshold(4194304)

    # Allocations...
    ptr := gc_alloc(1024, 1)  # With type_id
    gc_add_root(ptr)

    gc_remove_root(ptr)
    0
}
```

### Generational GC Basic

```vais
U std/gc

F main() -> i64 {
    gen_gc_init()

    # Allocate in young generation
    ptr := gen_gc_alloc(256, 0)
    gen_gc_add_root(ptr)

    # Fast minor collection
    gen_gc_collect_minor()

    # Objects surviving multiple minor GCs get promoted to old gen
    gen_gc_collect_minor()
    gen_gc_collect_minor()
    gen_gc_collect_minor()

    # Thorough major collection
    gen_gc_collect_major()

    gen_gc_remove_root(ptr)
    0
}
```

### Write Barrier

```vais
U std/gc

S Node {
    value: i64,
    next: i64  # Pointer to another Node
}

F main() -> i64 {
    gen_gc_init()

    # Allocate two nodes
    node1 := gen_gc_alloc(16, 1) as &Node
    node2 := gen_gc_alloc(16, 1) as &Node
    gen_gc_add_root(node1 as i64)
    gen_gc_add_root(node2 as i64)

    # If node1 is in old gen and we're modifying it to point to node2
    old_next := node1.next
    node1.next = node2 as i64
    gen_gc_write_barrier(node1 as i64, old_next, node2 as i64)

    gen_gc_remove_root(node1 as i64)
    gen_gc_remove_root(node2 as i64)
    0
}
```

### Generational GC Statistics

```vais
U std/gc

F main() -> i64 {
    gen_gc_init()

    # Allocate objects
    i := 0
    L i < 100 {
        ptr := gen_gc_alloc(50, 0)
        gen_gc_add_root(ptr)
        i = i + 1
    }

    # Check statistics
    stats := gen_gc_stats()
    # stats.young_objects
    # stats.old_objects
    # stats.minor_collections
    # stats.major_collections
    # stats.total_promoted

    gen_gc_print_stats()
    0
}
```

### Low-Latency Tuning

```vais
U std/gc

F main() -> i64 {
    gen_gc_init()

    # Optimize for low pause times
    gen_gc_tune_low_latency()

    # Small young gen = frequent fast minor GCs
    # Small promotion age = quick promotion to old gen
    # Suitable for interactive applications

    ptr := gen_gc_alloc(100, 0)
    gen_gc_add_root(ptr)
    # ...
    gen_gc_remove_root(ptr)
    0
}
```

### Throughput Tuning

```vais
U std/gc

F main() -> i64 {
    gen_gc_init()

    # Optimize for throughput
    gen_gc_tune_throughput()

    # Large young gen = fewer minor GCs
    # High promotion age = keep objects in young gen longer
    # Suitable for batch processing, high allocation rates

    i := 0
    L i < 10000 {
        ptr := gen_gc_alloc(200, 0)
        gen_gc_add_root(ptr)
        i = i + 1
    }

    0
}
```

### Custom Generational Configuration

```vais
U std/gc

F main() -> i64 {
    gen_gc_init()

    # Custom tuning
    gen_gc_set_young_threshold(524288)    # 512KB
    gen_gc_set_old_threshold(8388608)     # 8MB
    gen_gc_set_promotion_age(4)           # Promote after 4 minor GCs

    # Your application logic
    ptr := gen_gc_alloc(1024, 0)
    gen_gc_add_root(ptr)

    gen_gc_collect_full()
    gen_gc_remove_root(ptr)
    0
}
```

### Full Collection Cycle

```vais
U std/gc

F main() -> i64 {
    gen_gc_init()

    # Phase 1: Allocate many short-lived objects
    i := 0
    L i < 1000 {
        temp := gen_gc_alloc(50, 0)
        # temp not rooted, will be collected
        i = i + 1
    }

    # Phase 2: Minor GC to clean up young gen
    gen_gc_collect_minor()

    # Phase 3: Allocate long-lived object
    persistent := gen_gc_alloc(200, 1)
    gen_gc_add_root(persistent)

    # Phase 4: Several minor GCs to promote persistent object
    gen_gc_collect_minor()
    gen_gc_collect_minor()
    gen_gc_collect_minor()
    gen_gc_collect_minor()

    # Phase 5: Major GC to clean old generation
    gen_gc_collect_major()

    # Check final state
    stats := gen_gc_stats()
    gen_gc_print_stats()

    gen_gc_remove_root(persistent)
    0
}
```

## Generational GC Theory

### Two-Generation Model

- **Young Generation**: New objects allocated here. Fast, frequent collections.
- **Old Generation**: Long-lived objects promoted here. Slower, less frequent collections.

### Collection Types

1. **Minor GC**: Collects young generation only
   - Fast (small heap region)
   - Frequent (low threshold)
   - Promotes survivors to old generation

2. **Major GC**: Collects both generations
   - Slower (entire heap)
   - Infrequent (high threshold)
   - Thorough cleanup

3. **Full GC**: Minor followed by major
   - Most thorough
   - Use for complete cleanup

### Write Barrier

When modifying old-generation objects to point to young-generation objects, call the write barrier to maintain GC correctness.

**Required**: Old object pointing to young object
**Not required**: Young object pointing to any object, old object pointing to old object

### Promotion

Objects surviving `promotion_age` minor collections are promoted from young to old generation.

- Low promotion age: Objects promoted quickly (less young gen pressure)
- High promotion age: Objects stay in young gen longer (more thorough filtering)

## Performance Considerations

### Basic GC

- **Collection Time**: O(reachable objects)
- **Threshold**: Controls collection frequency vs pause time
- **Best for**: Simple applications, predictable allocation patterns

### Generational GC

- **Minor GC Time**: O(young objects) - typically 10-100x faster than full GC
- **Major GC Time**: O(all objects)
- **Best for**: Applications with many short-lived objects (most programs)

### Tuning Guidelines

| Scenario | Recommendation |
|----------|----------------|
| Real-time, low latency | Low-latency preset, small young gen |
| High allocation rate | Throughput preset, large young gen |
| Mixed workload | Balanced preset (default) |
| Memory constrained | Small thresholds, frequent GC |
| CPU constrained | Large thresholds, less frequent GC |

## Memory Management

- **Roots**: Must explicitly register/unregister stack and global pointers
- **Type IDs**: Optional tagging for debugging (not used for collection)
- **Thresholds**: Automatic collection triggered when threshold exceeded
- **Manual Collection**: Force collection anytime with `gc_collect()` or `gen_gc_collect_*`
- **Thread Safety**: GC is not thread-safe; use external synchronization

## Advanced Usage

### Mixing Allocators

```vais
U std/gc
U std/allocator

F main() -> i64 {
    gc_init()

    # GC for dynamic data structures
    tree_node := gc_alloc_simple(32)
    gc_add_root(tree_node)

    # Pool allocator for fixed-size temporary objects
    pool := PoolAllocator::new(64, 100)
    temp := pool.alloc()

    # Each allocator manages its own memory
    pool.drop()
    gc_remove_root(tree_node)
    0
}
```

### Hybrid Manual/GC Management

```vais
U std/gc

F main() -> i64 {
    gen_gc_init()

    # Long-lived objects: GC-managed
    global_cache := gen_gc_alloc(1024, 1)
    gen_gc_add_root(global_cache)

    # Short-lived objects: Manual malloc/free
    temp := malloc(256)
    # ... use temp ...
    free(temp)

    gen_gc_remove_root(global_cache)
    0
}
```

## Debugging Tips

- Use `gc_print_stats()` / `gen_gc_print_stats()` to monitor GC behavior
- Track `collections` count to detect over-collection
- Monitor `bytes_allocated` for memory leaks (growing despite collections)
- For generational GC, check promotion rate via `total_promoted`
- High minor collection count with low promotion = good (short-lived objects)
- High major collection count = may need larger old generation threshold
