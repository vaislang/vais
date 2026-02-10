# Slice Types

## Overview
Vais supports slice types `&[T]` and `&mut [T]` as fat pointers (pointer + length). Slices provide a view into contiguous sequences without ownership, enabling efficient array and vector operations.

## Syntax
- **Immutable slice**: `&[T]` — read-only view
- **Mutable slice**: `&mut [T]` — mutable view

## Creating Slices

Slices are created by range indexing into arrays or vectors:

```vais
arr := [10, 20, 30, 40, 50]
slice := arr[1..3]    # &[i64] — elements at index 1, 2

mut_arr := mut [1, 2, 3, 4]
mut_slice := mut_arr[0..2]    # &mut [i64]
```

## Indexing

### Element Access
```vais
slice := [10, 20, 30][0..3]
val := slice[0]       # Access element (bounds checked at runtime)
# val = 10
```

### Sub-slicing
```vais
slice := [1, 2, 3, 4, 5][1..4]    # [2, 3, 4]
sub := slice[1..2]                  # [3] — sub-slice
```

Range syntax:
- `slice[i..j]` — elements from index i (inclusive) to j (exclusive)
- `slice[i..]` — from index i to end
- `slice[..j]` — from start to index j
- `slice[..]` — entire slice (copy)

## Length

The `.len()` method returns the slice length as `i64`:

```vais
data := [1, 2, 3, 4, 5]
slice := data[1..4]
len := slice.len()    # Returns 3
```

## Function Parameters

Slices are commonly used for function parameters to avoid copying:

```vais
F sum(data: &[i64]) -> i64 {
    total := 0
    i := 0
    L i < data.len() {
        total = total + data[i]
        i = i + 1
    }
    total
}

F fill(data: &mut [i64], val: i64) {
    i := 0
    L i < data.len() {
        data[i] = val
        i = i + 1
    }
}

# Usage
arr := [10, 20, 30, 40]
s := sum(arr[..])           # Pass entire array as slice
fill(arr[1..3], 99)         # Fill indices 1, 2 with 99
```

## Borrowing Rules

Slices follow Vais ownership rules:
- Multiple `&[T]` slices allowed simultaneously (immutable borrows)
- Only one `&mut [T]` slice at a time (mutable borrow)
- Cannot have `&[T]` and `&mut [T]` simultaneously

```vais
arr := mut [1, 2, 3, 4]
slice1 := arr[0..2]         # &[i64] — OK
slice2 := arr[2..4]         # &[i64] — OK (non-overlapping)

mut_slice := arr[0..2]      # &mut [i64]
# slice3 := arr[1..3]       # ERROR: cannot borrow while mutably borrowed
```

## Implementation Details

### Fat Pointer Representation
Slices are implemented as fat pointers containing:
- `i8*` — pointer to first element
- `i64` — length (number of elements)

LLVM type: `{ i8*, i64 }`

### Indexing Operations
Slice indexing is compiled to:
1. `extractvalue` — extract pointer and length from fat pointer
2. `bitcast` — cast `i8*` to element type pointer
3. `getelementptr` (GEP) — compute element address
4. `load` — read element value

### Bounds Checking
All slice indexing operations include runtime bounds checks. Out-of-bounds access triggers a runtime error.

### Comparison with Arrays
| Feature | Array `[T; N]` | Slice `&[T]` |
|---------|----------------|--------------|
| Size | Compile-time constant | Runtime value |
| Indexing | Direct GEP | extractvalue + bitcast + GEP |
| Storage | Inline value | Fat pointer (16 bytes) |
| Ownership | Owned | Borrowed |

## Examples

### Iterating Over Slices
```vais
F print_all(items: &[i64]) {
    i := 0
    L i < items.len() {
        print_i64(items[i])
        i = i + 1
    }
}

data := [10, 20, 30, 40, 50]
print_all(data[1..4])    # Prints: 20, 30, 40
```

### Splitting Slices
```vais
F split_at(data: &[i64], mid: i64) -> (&[i64], &[i64]) {
    left := data[0..mid]
    right := data[mid..data.len()]
    (left, right)
}

arr := [1, 2, 3, 4, 5, 6]
(first, second) := split_at(arr[..], 3)
# first = [1, 2, 3], second = [4, 5, 6]
```

### In-place Modification
```vais
F double_values(data: &mut [i64]) {
    i := 0
    L i < data.len() {
        data[i] = data[i] * 2
        i = i + 1
    }
}

arr := mut [5, 10, 15]
double_values(arr[..])
# arr is now [10, 20, 30]
```

## See Also

- [Lifetimes & Borrow Checking](./lifetimes.md)
- [Type Inference](./type-inference.md)
- [Vec API](../api/vec.md)
