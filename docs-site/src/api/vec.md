# Vec API Reference

> Dynamic array (growable) for storing elements of type T

## Import

```vais
U std/vec
```

## Overview

`Vec<T>` is a generic dynamic array that automatically grows as needed. It provides efficient indexed access and append operations. Each element occupies 8 bytes (i64) in memory. The module depends on `std/option` for safe error handling.

## Dependencies

```vais
U std/option
```

## Struct

### `Vec<T>`

```vais
S Vec<T> {
    data: i64,      # Pointer to element array (all pointers are i64)
    len: i64,       # Current number of elements
    cap: i64        # Allocated capacity
}
```

A dynamically-sized array that stores elements contiguously in memory.

**Fields:**
- `data`: Pointer to the allocated memory
- `len`: Current number of elements in the vector
- `cap`: Total capacity before reallocation is needed

## Methods

### with_capacity

```vais
F with_capacity(capacity: i64) -> Vec<T>
```

Create a new empty Vec with the specified initial capacity. Allocates `capacity * 8` bytes.

**Parameters:**
- `capacity`: Initial capacity to allocate (number of elements)

**Returns:** A new `Vec<T>` with the specified capacity

**Example:**
```vais
v := Vec.with_capacity(100)
```

---

### len

```vais
F len(&self) -> i64
```

Get the number of elements in the vector.

**Returns:** The current length

---

### capacity

```vais
F capacity(&self) -> i64
```

Get the allocated capacity of the vector.

**Returns:** The total capacity

---

### is_empty

```vais
F is_empty(&self) -> i64
```

Check if the vector is empty.

**Returns:** `1` if empty, `0` otherwise

---

### get

```vais
F get(&self, index: i64) -> T
```

Get element at the specified index. Returns `0` if index is out of bounds.

**Parameters:**
- `index`: The index to access

**Returns:** The element at the index, or `0` if out of bounds

---

### get_opt

```vais
F get_opt(&self, index: i64) -> Option<T>
```

Get element at index using Option type for safer access.

**Parameters:**
- `index`: The index to access

**Returns:** `Some(value)` if index is valid, `None` otherwise

---

### set

```vais
F set(&self, index: i64, value: T) -> i64
```

Set element at the specified index.

**Parameters:**
- `index`: The index to modify
- `value`: The value to set

**Returns:** `1` if successful, `0` if index out of bounds

---

### push

```vais
F push(&self, value: T) -> i64
```

Push an element to the end of the vector. Automatically grows capacity if needed.

**Parameters:**
- `value`: The value to append

**Returns:** The new length

---

### pop

```vais
F pop(&self) -> T
```

Pop and return the last element. Returns `0` if vector is empty.

**Returns:** The last element, or `0` if empty

---

### pop_opt

```vais
F pop_opt(&self) -> Option<T>
```

Pop element using Option type for safer access.

**Returns:** `Some(value)` if vec is not empty, `None` otherwise

---

### grow

```vais
F grow(&self) -> i64
```

Grow the vector's capacity. Doubles the current capacity, or sets it to 8 if less than 8. Called automatically by `push` when needed.

**Returns:** The new capacity

---

### clear

```vais
F clear(&self) -> i64
```

Clear all elements (sets length to 0).

**Returns:** `0`

---

### drop

```vais
F drop(&self) -> i64
```

Free the vector's memory.

**Returns:** `0`

## Functions

### vec_new

```vais
F vec_new() -> Vec<i64>
```

Create a new `Vec<i64>` with initial capacity of 8.

**Returns:** A new `Vec<i64>`

**Example:**
```vais
v := vec_new()
v.push(42)
v.push(100)
```

## Usage Examples

### Basic Usage

```vais
U std/vec

F main() -> i64 {
    # Create a vector
    v := Vec.with_capacity(10)

    # Add elements
    v.push(1)
    v.push(2)
    v.push(3)

    # Access elements
    x := v.get(0)  # x = 1
    y := v.get(2)  # y = 3

    # Check length
    I v.len() > 0 {
        puts("Vector is not empty")
    }

    # Clean up
    v.drop()
    0
}
```

### Using vec_new Helper

```vais
U std/vec

F main() -> i64 {
    # Create Vec<i64> with default capacity
    v := vec_new()

    v.push(10)
    v.push(20)
    v.push(30)

    # Iterate through elements
    i := 0
    L {
        I i >= v.len() {
            B 0
        }
        val := v.get(i)
        puts_i64(val)
        i = i + 1
    }

    v.drop()
    0
}
```

### Using Option Type for Safe Access

```vais
U std/vec
U std/option

F main() -> i64 {
    v := vec_new()
    v.push(42)

    # Safe access with Option
    M v.get_opt(0) {
        Some(val) => { puts_i64(val) }
        None => { puts("Out of bounds") }
    }

    # Out of bounds access
    M v.get_opt(10) {
        Some(val) => { puts_i64(val) }
        None => { puts("Index too large") }  # This prints
    }

    v.drop()
    0
}
```

### Stack Operations

```vais
U std/vec

F main() -> i64 {
    v := vec_new()

    # Push elements (like a stack)
    v.push(1)
    v.push(2)
    v.push(3)

    # Pop elements in reverse order
    L {
        I v.is_empty() {
            B 0
        }
        val := v.pop()
        puts_i64(val)  # Prints: 3, 2, 1
    }

    v.drop()
    0
}
```

### Safe Pop with Option

```vais
U std/vec
U std/option

F main() -> i64 {
    v := vec_new()
    v.push(100)

    # Pop safely
    M v.pop_opt() {
        Some(val) => { puts_i64(val) }  # Prints 100
        None => { puts("Empty") }
    }

    # Pop from empty vector
    M v.pop_opt() {
        Some(val) => { puts_i64(val) }
        None => { puts("Empty") }  # This prints
    }

    v.drop()
    0
}
```

### Modifying Elements

```vais
U std/vec

F main() -> i64 {
    v := vec_new()

    # Add elements
    v.push(10)
    v.push(20)
    v.push(30)

    # Modify an element
    v.set(1, 99)

    # Verify
    puts_i64(v.get(1))  # Prints 99

    v.drop()
    0
}
```

### Clearing and Reusing

```vais
U std/vec

F main() -> i64 {
    v := vec_new()

    # First use
    v.push(1)
    v.push(2)
    puts_i64(v.len())  # 2

    # Clear
    v.clear()
    puts_i64(v.len())  # 0

    # Reuse
    v.push(10)
    v.push(20)
    puts_i64(v.len())  # 2

    v.drop()
    0
}
```
