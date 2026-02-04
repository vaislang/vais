# Allocator API Reference

> Custom memory allocator traits and implementations

## Import

```vais
U std/allocator
```

## Structs

### Layout

Describes memory size and alignment requirements.

```vais
S Layout { size: i64, align: i64 }
```

| Function | Signature | Description |
|----------|-----------|-------------|
| `layout_new` | `F layout_new(size: i64, align: i64) -> Layout` | Create layout |
| `layout_for_i64` | `F layout_for_i64() -> Layout` | Layout for i64 (8, 8) |
| `layout_array` | `F layout_array(elem_size: i64, elem_align: i64, count: i64) -> Layout` | Array layout |

### Allocation

Result of an allocation with pointer and actual size.

```vais
S Allocation { ptr: i64, size: i64 }
```

## Overview

This module provides the foundation for pluggable allocators. Users can implement custom allocation strategies (pool allocators, slab allocators, etc.) using the Layout and Allocation types.

## Usage

```vais
U std/allocator

F main() -> i64 {
    layout := layout_for_i64()
    # layout.size = 8, layout.align = 8
    0
}
```
