# Vec API Reference

> Dynamic array (growable) for storing elements of type T

## Overview

`Vec<T>` is a generic dynamic array that automatically grows as needed. It provides efficient indexed access and append operations.

## Struct

### Vec<T>

```vais
S Vec<T> {
    data: i64,      # Pointer to element array
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

Create a new empty Vec with the specified initial capacity.

**Parameters:**
- `capacity`: Initial capacity to allocate

**Returns:** A new `Vec<T>` with the specified capacity

**Example:**
```vais
v := Vec::with_capacity(100)
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

Grow the vector's capacity (typically doubles it).

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
# Create a vector
v := Vec::with_capacity(10)

# Add elements
v.push(1)
v.push(2)
v.push(3)

# Access elements
x := v.get(0)  # x = 1

# Check length
L v.len() > 0 {
    # Vector is not empty
}
```

### Using Option Type

```vais
v := vec_new()
v.push(42)

# Safe access with Option
M v.get_opt(0) {
    Some(val) => { # Use val }
    None => { # Handle error }
}
```
