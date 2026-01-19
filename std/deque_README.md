# Deque Implementation for VAIS

## Overview

This directory contains a Double-ended Queue (Deque) implementation for the VAIS standard library.

## Files

- `/Users/sswoo/study/projects/vais/std/deque.vais` - Deque implementation using circular buffer
- `/Users/sswoo/study/projects/vais/examples/deque_test.vais` - Comprehensive test suite
- `/Users/sswoo/study/projects/vais/examples/deque_minimal_test.vais` - Minimal test

## Implementation Details

The Deque is implemented as a circular buffer with the following features:

### Data Structure
```vais
S Deque {
    data: i64,      # Pointer to element array
    head: i64,      # Index of first element
    tail: i64,      # Index of next insertion position at back
    len: i64,       # Current number of elements
    cap: i64        # Allocated capacity
}
```

### Methods

- `with_capacity(capacity: i64) -> Deque` - Create deque with specified capacity
- `deque_new() -> Deque` - Create deque with default capacity of 8
- `push_front(&self, value: i64) -> i64` - Add element to front (O(1))
- `push_back(&self, value: i64) -> i64` - Add element to back (O(1))
- `pop_front(&self) -> i64` - Remove and return front element (O(1))
- `pop_back(&self) -> i64` - Remove and return back element (O(1))
- `front(&self) -> i64` - Peek at front element without removing
- `back(&self) -> i64` - Peek at back element without removing
- `get(&self, index: i64) -> i64` - Get element at index (0-indexed from front)
- `set(&self, index: i64, value: i64) -> i64` - Set element at index
- `len(&self) -> i64` - Get number of elements
- `is_empty(&self) -> i64` - Check if empty (returns 1 if empty, 0 otherwise)
- `capacity(&self) -> i64` - Get current capacity
- `clear(&self) -> i64` - Remove all elements
- `drop(&self) -> i64` - Free memory

### Design Decisions

1. **Circular Buffer**: Efficient O(1) operations at both ends without shifting elements
2. **Automatic Growth**: Doubles capacity when full (min capacity: 8)
3. **Recursive Helper**: Used `deque_copy_element()` helper function to work around VAIS's immutable local variables
4. **Conditional Expressions**: Used `I cond { val1 } E { val2 }` pattern instead of mutable variable assignments
5. **No Early Returns**: Restructured functions to use if-else chains instead of break statements

## Current Limitations

Due to current VAIS compiler limitations:
- The implementation is complete but may not compile with the current compiler version
- Some std library features like the `@.method()` syntax for calling methods from within methods are not yet implemented
- Struct return values may have code generation issues

## Test Coverage

The test suite (`deque_test.vais`) includes:
1. Basic creation and empty checks
2. Push back operations
3. Push front operations
4. Pop front operations
5. Pop back operations
6. Get and set operations
7. Clear operation
8. Circular buffer wrapping behavior
9. Growth and reallocation
10. Mixed operations
11. Empty deque edge cases
12. Negative numbers

## Future Enhancements

When the VAIS compiler supports additional features:
- Generic type parameters (Deque<T> instead of i64-only)
- Iterator implementation
- Additional collection operations (contains, remove_at, etc.)
- Optimization passes for better code generation
