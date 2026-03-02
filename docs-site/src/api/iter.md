# Iter API Reference

> Iterator pattern for sequential access, based on Rust's Iterator trait

## Import

```vais
U std/iter
```

## Overview

The `iter` module defines the iterator pattern for Vais. It provides the `Iterator` trait, concrete iterator structs (`Range`, `VecIter`, `SliceIter`), and array-based adapter functions for functional-style data processing (map, filter, fold, etc.).

**Note:** Due to current compiler limitations, static methods are not supported. Use helper functions instead (e.g., construct structs directly).

## Trait

### `Iterator`

```vais
W Iterator {
    F next(&self) -> i64
}
```

The core iterator interface. Returns the next value, or `-1` when exhausted.

## Structs

### `Range`

```vais
S Range {
    current: i64,
    end: i64,
    step: i64
}
```

Iterates from `current` to `end` (exclusive) with a given step.

**Methods:**

- `has_next(&self) -> i64` -- Returns `1` if more elements remain, `0` otherwise
- `next(&self) -> i64` -- Returns next value and advances, or `-1` when done

### `VecIter`

```vais
S VecIter {
    data: i64,
    len: i64,
    index: i64
}
```

Iterates over elements of a `Vec`.

**Methods:**

- `has_next(&self) -> i64` -- Returns `1` if more elements remain
- `peek(&self) -> i64` -- Peek at current element without advancing
- `next(&self) -> i64` -- Returns next value and advances

### `SliceIter`

```vais
S SliceIter {
    ptr: i64,
    end: i64,
    elem_size: i64
}
```

Iterates over a memory slice by pointer.

**Methods:**

- `has_next(&self) -> i64` -- Returns `1` if more elements remain
- `next(&self) -> i64` -- Returns next value and advances

## Consuming Functions (Array-based)

These operate on arrays stored as `(data_ptr, len)` pairs.

### iter_sum

```vais
F iter_sum(data: i64, len: i64) -> i64
```

Sum all elements in an array.

### iter_product

```vais
F iter_product(data: i64, len: i64) -> i64
```

Product of all elements.

### iter_count

```vais
F iter_count(data: i64, len: i64) -> i64
```

Returns the number of elements (same as `len`).

### iter_min / iter_max

```vais
F iter_min(data: i64, len: i64) -> i64
F iter_max(data: i64, len: i64) -> i64
```

Find the minimum or maximum element. Returns `0` if empty.

### iter_contains

```vais
F iter_contains(data: i64, len: i64, value: i64) -> i64
```

Check if any element equals `value`. Returns `1` if found, `0` otherwise.

## Adapter Functions

These create new arrays via `malloc` and return pointers.

### iter_map

```vais
F iter_map(data: i64, len: i64, f: fn(i64) -> i64) -> i64
```

Apply function `f` to each element. Returns pointer to new array (same length).

### iter_filter

```vais
F iter_filter(data: i64, len: i64, pred: fn(i64) -> i64, out_count_ptr: i64) -> i64
```

Filter elements by predicate. Actual count is stored at `out_count_ptr`. Returns pointer to new array.

### iter_take / iter_skip

```vais
F iter_take(data: i64, len: i64, n: i64) -> i64
F iter_skip(data: i64, len: i64, n: i64) -> i64
```

Take the first `n` or skip the first `n` elements.

### iter_chain

```vais
F iter_chain(data1: i64, len1: i64, data2: i64, len2: i64) -> i64
```

Concatenate two arrays. Total length = `len1 + len2`.

### iter_zip

```vais
F iter_zip(data1: i64, len1: i64, data2: i64, len2: i64) -> i64
```

Pair elements as consecutive `(a, b)` pairs. Returns pointer to `2 * min(len1, len2)` elements.

### iter_enumerate

```vais
F iter_enumerate(data: i64, len: i64) -> i64
```

Pair each element with its index as `(index, value)`.

### iter_fold

```vais
F iter_fold(data: i64, len: i64, init: i64, f: fn(i64, i64) -> i64) -> i64
```

Reduce the array to a single value using accumulator function `f`.

### iter_any / iter_all

```vais
F iter_any(data: i64, len: i64, pred: fn(i64) -> i64) -> i64
F iter_all(data: i64, len: i64, pred: fn(i64) -> i64) -> i64
```

Check if any/all elements satisfy a predicate.

### iter_find

```vais
F iter_find(data: i64, len: i64, pred: fn(i64) -> i64) -> i64
```

Return the first element satisfying the predicate, or `-1` if not found.

### iter_position

```vais
F iter_position(data: i64, len: i64, pred: fn(i64) -> i64) -> i64
```

Return the index of the first element satisfying the predicate, or `-1`.

## Example

```vais
U std/iter

F main() {
    # Range-based for loop
    L i:0..10 { print(i) }

    # Manual iterator
    r := Range { current: 0, end: 5, step: 1 }
    L {
        v := r.next()
        I v < 0 { B }
        print(v)
    }

    # Adapter chaining
    data := iter_map(arr, len, |x: i64| x * 2)
    sum := iter_sum(data, len)
}
```
