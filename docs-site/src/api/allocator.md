# Allocator API Reference

> Custom memory allocator traits and implementations for flexible allocation strategies

## Import

```vais
U std/allocator
```

## Overview

The allocator module provides foundation types and multiple allocation strategies:
- **Global allocator** - Wraps system malloc/free with alignment
- **Bump allocator** - Fast linear allocation, bulk reset
- **Pool allocator** - Fixed-size blocks with O(1) alloc/free
- **Free list allocator** - General-purpose with first-fit allocation
- **Stack allocator** - LIFO allocation with efficient pop

All allocators support explicit memory management with pointer-based state mutation.

## Structs

### Layout

Describes memory requirements with size and alignment.

```vais
S Layout {
    size: i64,        # Required size in bytes
    align: i64        # Required alignment (must be power of 2)
}
```

#### Layout Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `layout_new` | `F layout_new(size: i64, align: i64) -> Layout` | Create a custom layout |
| `layout_for_i64` | `F layout_for_i64() -> Layout` | Layout for i64 (size=8, align=8) |
| `layout_for_i32` | `F layout_for_i32() -> Layout` | Layout for i32 (size=4, align=4) |
| `layout_for_ptr` | `F layout_for_ptr() -> Layout` | Layout for pointer (size=8, align=8) |
| `layout_array` | `F layout_array(elem_size: i64, elem_align: i64, count: i64) -> Layout` | Layout for array of elements |
| `layout_extend` | `F layout_extend(base: Layout, ext: Layout) -> Layout` | Extend layout with another layout |

### Allocation

Result of an allocation operation with pointer and actual size.

```vais
S Allocation {
    ptr: i64,         # Pointer to allocated memory (0 if failed)
    size: i64         # Actual allocated size (may be >= requested)
}
```

## Global Allocator

Wraps system malloc/free with alignment support.

| Function | Signature | Description |
|----------|-----------|-------------|
| `global_alloc` | `F global_alloc(layout: Layout) -> Allocation` | Allocate aligned memory |
| `global_alloc_zeroed` | `F global_alloc_zeroed(layout: Layout) -> Allocation` | Allocate zeroed memory |
| `global_dealloc` | `F global_dealloc(ptr: i64, layout: Layout) -> ()` | Free allocated memory |
| `global_realloc` | `F global_realloc(ptr: i64, old_layout: Layout, new_layout: Layout) -> Allocation` | Reallocate memory |

## BumpAllocator

A simple bump allocator that allocates linearly from a buffer. Extremely fast allocation, but can only free all at once.

```vais
S BumpAllocator {
    buffer: i64,      # Start of buffer
    capacity: i64,    # Total capacity
    offset: i64,      # Current allocation offset
    allocated: i64    # Total bytes allocated (for stats)
}
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(capacity: i64) -> BumpAllocator` | Create allocator with capacity |
| `from_buffer` | `F from_buffer(buffer: i64, capacity: i64) -> BumpAllocator` | Create from existing buffer (doesn't own) |
| `alloc` | `F alloc(&self, layout: Layout) -> Allocation` | Allocate from bump allocator |
| `alloc_zeroed` | `F alloc_zeroed(&self, layout: Layout) -> Allocation` | Allocate zeroed memory |
| `reset` | `F reset(&self) -> i64` | Reset to beginning (frees all) |
| `remaining` | `F remaining(&self) -> i64` | Get remaining capacity |
| `total_allocated` | `F total_allocated(&self) -> i64` | Get total allocated bytes |
| `drop` | `F drop(&self) -> i64` | Free the allocator's buffer |

### Convenience Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `bump_new` | `F bump_new(capacity: i64) -> BumpAllocator` | Create a bump allocator |
| `bump_free` | `F bump_free(alloc: BumpAllocator) -> i64` | Free a bump allocator |

## PoolAllocator

A pool allocator for fixed-size objects. Very fast allocation/deallocation with no fragmentation.

```vais
S PoolAllocator {
    buffer: i64,       # Start of buffer
    capacity: i64,     # Total capacity in bytes
    block_size: i64,   # Size of each block
    free_list: i64,    # Pointer to first free block
    num_blocks: i64,   # Total number of blocks
    num_free: i64      # Number of free blocks
}
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(block_size: i64, num_blocks: i64) -> PoolAllocator` | Create pool allocator |
| `alloc` | `F alloc(&self) -> i64` | Allocate a block from pool (returns 0 if full) |
| `dealloc` | `F dealloc(&self, ptr: i64) -> i64` | Free a block back to pool |
| `num_free_blocks` | `F num_free_blocks(&self) -> i64` | Get number of free blocks |
| `num_allocated` | `F num_allocated(&self) -> i64` | Get number of allocated blocks |
| `drop` | `F drop(&self) -> i64` | Free the pool allocator's buffer |

### Convenience Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `pool_new` | `F pool_new(block_size: i64, num_blocks: i64) -> PoolAllocator` | Create a pool allocator |
| `pool_free` | `F pool_free(alloc: PoolAllocator) -> i64` | Free a pool allocator |

**Note**: Minimum block size is 8 bytes (pointer-sized) for free list implementation.

## FreeListAllocator

A general-purpose free list allocator with first-fit allocation strategy.

```vais
S FreeListAllocator {
    buffer: i64,      # Start of buffer
    capacity: i64,    # Total capacity
    free_list: i64,   # Pointer to first free block
    allocated: i64    # Total bytes currently allocated
}
```

### Block Header

```vais
S FreeBlock {
    size: i64,        # Size of this block (including header)
    next: i64         # Pointer to next free block
}
```

**Header size**: 16 bytes (size + next)

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(capacity: i64) -> FreeListAllocator` | Create free list allocator |
| `alloc` | `F alloc(&self, layout: Layout) -> Allocation` | Allocate using first-fit |
| `dealloc` | `F dealloc(&self, ptr: i64) -> i64` | Free memory back to free list |
| `total_allocated` | `F total_allocated(&self) -> i64` | Get allocated bytes |
| `remaining` | `F remaining(&self) -> i64` | Get remaining bytes (approximate) |
| `drop` | `F drop(&self) -> i64` | Free the allocator's buffer |

### Convenience Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `freelist_new` | `F freelist_new(capacity: i64) -> FreeListAllocator` | Create a free list allocator |
| `freelist_free` | `F freelist_free(alloc: FreeListAllocator) -> i64` | Free a free list allocator |

**Features**:
- First-fit allocation strategy
- Block splitting when allocation is smaller than free block
- Minimum allocation size: 32 bytes (including header)

## StackAllocator

A stack-based allocator with LIFO allocation pattern.

```vais
S StackAllocator {
    buffer: i64,      # Start of buffer
    capacity: i64,    # Total capacity
    offset: i64,      # Current stack top
    prev_offset: i64  # Previous allocation offset (for pop)
}
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(capacity: i64) -> StackAllocator` | Create stack allocator |
| `alloc` | `F alloc(&self, layout: Layout) -> Allocation` | Allocate from stack |
| `pop` | `F pop(&self) -> i64` | Pop the most recent allocation |
| `reset` | `F reset(&self) -> i64` | Reset stack to beginning |
| `remaining` | `F remaining(&self) -> i64` | Get remaining capacity |
| `drop` | `F drop(&self) -> i64` | Free the allocator's buffer |

### Convenience Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `stack_new` | `F stack_new(capacity: i64) -> StackAllocator` | Create a stack allocator |
| `stack_free` | `F stack_free(alloc: StackAllocator) -> i64` | Free a stack allocator |

**Features**:
- LIFO (last-in, first-out) allocation pattern
- 8-byte header stores previous offset for pop operation
- Efficient for temporary allocations with predictable lifetimes

## Usage Examples

### Global Allocator

```vais
U std/allocator

F main() -> i64 {
    layout := layout_for_i64()
    alloc := global_alloc(layout)

    I alloc.ptr != 0 {
        # Use the allocated memory
        global_dealloc(alloc.ptr, layout)
    }
    0
}
```

### Bump Allocator

```vais
U std/allocator

F main() -> i64 {
    bump := BumpAllocator::new(4096)  # 4KB buffer

    # Fast sequential allocations
    a1 := bump.alloc(layout_for_i64())
    a2 := bump.alloc(layout_for_i32())
    a3 := bump.alloc(layout_array(8, 8, 10))

    # Reset frees all at once
    bump.reset()

    # Reuse the allocator
    a4 := bump.alloc(layout_for_ptr())

    bump.drop()
    0
}
```

### Pool Allocator

```vais
U std/allocator

F main() -> i64 {
    # Pool of 100 blocks, each 64 bytes
    pool := PoolAllocator::new(64, 100)

    # O(1) allocation
    ptr1 := pool.alloc()
    ptr2 := pool.alloc()
    ptr3 := pool.alloc()

    # O(1) deallocation
    pool.dealloc(ptr2)

    # Check availability
    free_count := pool.num_free_blocks()  # 98

    pool.drop()
    0
}
```

### Free List Allocator

```vais
U std/allocator

F main() -> i64 {
    freelist := FreeListAllocator::new(8192)  # 8KB buffer

    # Variable-size allocations
    a1 := freelist.alloc(layout_new(100, 8))
    a2 := freelist.alloc(layout_new(500, 8))
    a3 := freelist.alloc(layout_new(200, 8))

    # Free in any order
    freelist.dealloc(a2.ptr)
    freelist.dealloc(a1.ptr)

    # Reallocate freed space
    a4 := freelist.alloc(layout_new(300, 8))

    freelist.drop()
    0
}
```

### Stack Allocator

```vais
U std/allocator

F main() -> i64 {
    stack := StackAllocator::new(2048)

    # LIFO allocation
    a1 := stack.alloc(layout_for_i64())
    a2 := stack.alloc(layout_for_i32())
    a3 := stack.alloc(layout_for_ptr())

    # Pop most recent (a3)
    stack.pop()

    # Pop next (a2)
    stack.pop()

    # Or reset all
    stack.reset()

    stack.drop()
    0
}
```

### Layout Extension

```vais
U std/allocator

F main() -> i64 {
    # Create layout for struct { i64, i32, ptr }
    base := layout_for_i64()
    layout2 := layout_extend(base, layout_for_i32())
    layout3 := layout_extend(layout2, layout_for_ptr())

    # layout3.size accounts for padding and alignment
    alloc := global_alloc(layout3)
    global_dealloc(alloc.ptr, layout3)
    0
}
```

### Zeroed Allocation

```vais
U std/allocator

F main() -> i64 {
    # Global allocator with zeroing
    alloc := global_alloc_zeroed(layout_array(8, 8, 100))

    # Bump allocator with zeroing
    bump := BumpAllocator::new(4096)
    zeroed := bump.alloc_zeroed(layout_for_i64())

    bump.drop()
    global_dealloc(alloc.ptr, layout_array(8, 8, 100))
    0
}
```

## Performance Characteristics

| Allocator | Alloc Time | Dealloc Time | Fragmentation | Best Use Case |
|-----------|------------|--------------|---------------|---------------|
| Global | O(1)* | O(1)* | Medium | General-purpose |
| Bump | O(1) | N/A | None | Temporary/arena |
| Pool | O(1) | O(1) | None | Fixed-size objects |
| FreeList | O(n) | O(1) | Low-Medium | Variable sizes |
| Stack | O(1) | O(1) | None | LIFO patterns |

*Global allocator times depend on system malloc implementation

## Memory Management

- All allocators must be explicitly freed with `drop()` or convenience functions
- `Allocation.ptr` returns 0 on allocation failure
- Allocators mutate their state through `&self` references
- `from_buffer()` creates non-owning bump allocators
- Free list allocator includes 16-byte headers for each allocation
- Stack allocator includes 8-byte headers for each allocation

## Advanced Features

### Custom Arena Pattern

```vais
U std/allocator

F main() -> i64 {
    # Create arena from large buffer
    buffer := malloc(1048576)  # 1MB
    arena := BumpAllocator::from_buffer(buffer, 1048576)

    # Use arena for temporary allocations
    # ...

    # Reset without freeing buffer
    arena.reset()

    # Manually free buffer when done
    free(buffer)
    0
}
```

### Allocator Selection

Choose allocator based on allocation pattern:
- **Bump**: Frame allocations, temporary scratch space, parsers
- **Pool**: Object pools, message queues, fixed-size nodes
- **FreeList**: Dynamic data structures, variable-size blocks
- **Stack**: Function call frames, expression evaluation
- **Global**: Long-lived objects, unknown patterns
