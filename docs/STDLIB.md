# Vais Standard Library Reference

Version: 0.0.1

## Table of Contents

1. [Overview](#overview)
2. [Core Types](#core-types)
   - [Option](#option)
   - [Result](#result)
3. [Collections](#collections)
   - [Vec](#vec)
   - [HashMap](#hashmap)
   - [String](#string)
   - [Set](#set)
   - [Deque](#deque)
   - [PriorityQueue](#priorityqueue)
4. [Smart Pointers](#smart-pointers)
   - [Box](#box)
   - [Rc](#rc)
5. [Memory Management](#memory-management)
   - [Arena](#arena)
6. [I/O](#io)
   - [IO Module](#io-module)
   - [File](#file)
7. [Mathematics](#mathematics)
   - [Math](#math)
8. [Async Runtime](#async-runtime)
   - [Future](#future)
   - [Runtime](#runtime)
9. [Traits](#traits)
   - [Iterator](#iterator)
10. [Built-in Functions](#built-in-functions)

---

## Overview

The Vais standard library provides essential data structures, algorithms, and system interfaces for building applications. All standard library modules are located in the `std/` directory.

### Importing Modules

Use the `U` keyword to import modules:

```vais
U std/math        # Math functions and constants
U std/io          # Input/output operations
U std/option      # Option type
U std/vec         # Dynamic arrays
```

---

## Core Types

### Option

**Module:** `std/option.vais`

The `Option` type represents an optional value that can be either `Some(value)` or `None`.

#### Definition

```vais
E Option {
    None,
    Some(i64)
}
```

#### Methods

```vais
# Check if option contains a value
F is_some(&self) -> i64

# Check if option is None
F is_none(&self) -> i64

# Get value or return default
F unwrap_or(&self, default: i64) -> i64
```

#### Example

```vais
U std/option

F divide(a: i64, b: i64) -> Option {
    I b == 0 {
        None
    } E {
        Some(a / b)
    }
}

F main() -> i64 {
    result := divide(10, 2)
    value := result.unwrap_or(0)  # Returns 5

    error := divide(10, 0)
    default := error.unwrap_or(-1)  # Returns -1

    print_i64(value)
    0
}
```

---

### Result

**Module:** `std/result.vais`

The `Result` type represents either success (`Ok(value)`) or failure (`Err(error)`).

#### Definition

```vais
E Result {
    Ok(i64),
    Err(i64)
}
```

#### Methods

```vais
# Check if result is Ok
F is_ok(&self) -> i64

# Check if result is Err
F is_err(&self) -> i64

# Get Ok value or default
F unwrap_or(&self, default: i64) -> i64

# Get Err value or default
F err_or(&self, default: i64) -> i64

# Map the Ok value
F map(&self, f: i64) -> Result
```

#### Helper Functions

```vais
# Create Ok result
F ok(value: i64) -> Result

# Create Err result
F err(code: i64) -> Result
```

#### Error Codes

```vais
F ERR_NONE() -> i64            # 0
F ERR_INVALID() -> i64         # 1
F ERR_NOT_FOUND() -> i64       # 2
F ERR_IO() -> i64              # 3
F ERR_OVERFLOW() -> i64        # 4
F ERR_DIVIDE_BY_ZERO() -> i64  # 5
```

#### Example

```vais
U std/result

F safe_divide(a: i64, b: i64) -> Result {
    I b == 0 {
        err(ERR_DIVIDE_BY_ZERO())
    } E {
        ok(a / b)
    }
}

F main() -> i64 {
    result := safe_divide(10, 2)

    I result.is_ok() {
        value := result.unwrap_or(0)
        print_i64(value)
    } E {
        puts("Error: Division by zero")
    }

    0
}
```

---

## Collections

### Vec

**Module:** `std/vec.vais`

Dynamic array (growable vector) that stores `i64` elements.

#### Definition

```vais
S Vec {
    data: i64,      # Pointer to element array
    len: i64,       # Current number of elements
    cap: i64        # Allocated capacity
}
```

#### Methods

```vais
# Create new Vec with capacity
F with_capacity(capacity: i64) -> Vec

# Get number of elements
F len(&self) -> i64

# Get capacity
F capacity(&self) -> i64

# Check if empty
F is_empty(&self) -> i64

# Get element at index (returns 0 if out of bounds)
F get(&self, index: i64) -> i64

# Set element at index
F set(&self, index: i64, value: i64) -> i64

# Push element to end
F push(&self, value: i64) -> i64

# Pop element from end (returns 0 if empty)
F pop(&self) -> i64

# Clear all elements
F clear(&self) -> i64
```

#### Example

```vais
U std/vec

F main() -> i64 {
    # Create vector with capacity 10
    v := Vec.with_capacity(10)

    # Add elements
    v.push(10)
    v.push(20)
    v.push(30)

    # Access elements
    first := v.get(0)    # 10
    second := v.get(1)   # 20

    # Get length
    len := v.len()       # 3

    # Pop last element
    last := v.pop()      # 30

    print_i64(first)
    print_i64(len)
    0
}
```

---

### HashMap

**Module:** `std/hashmap.vais`

Hash table with `i64` keys and `i64` values. Uses separate chaining for collision resolution.

#### Definition

```vais
S HashMap {
    buckets: i64,   # Pointer to bucket array
    size: i64,      # Number of key-value pairs
    cap: i64        # Number of buckets
}
```

#### Methods

```vais
# Create new HashMap with capacity
F with_capacity(capacity: i64) -> HashMap

# Get number of entries
F len(&self) -> i64

# Get capacity (number of buckets)
F capacity(&self) -> i64

# Check if empty
F is_empty(&self) -> i64

# Insert key-value pair
F insert(&self, key: i64, value: i64) -> i64

# Get value for key (returns 0 if not found)
F get(&self, key: i64) -> i64

# Check if key exists
F contains(&self, key: i64) -> i64

# Remove key (returns old value or 0)
F remove(&self, key: i64) -> i64

# Clear all entries
F clear(&self) -> i64
```

#### Example

```vais
U std/hashmap

F main() -> i64 {
    # Create hashmap
    map := HashMap.with_capacity(16)

    # Insert key-value pairs
    map.insert(1, 100)
    map.insert(2, 200)
    map.insert(42, 999)

    # Get values
    val1 := map.get(1)      # 100
    val2 := map.get(42)     # 999
    val3 := map.get(999)    # 0 (not found)

    # Check existence
    exists := map.contains(2)  # 1 (true)

    # Remove entry
    old_val := map.remove(1)   # 100

    print_i64(val1)
    print_i64(exists)
    0
}
```

---

### String

**Module:** `std/string.vais`

Dynamic, heap-allocated string type.

#### Definition

```vais
S String {
    data: i64,      # Pointer to char array
    len: i64,       # Current length
    cap: i64        # Allocated capacity
}
```

#### Methods

```vais
# Create new String with capacity
F with_capacity(capacity: i64) -> String

# Get length of string
F len(&self) -> i64

# Get capacity
F capacity(&self) -> i64

# Check if empty
F is_empty(&self) -> i64

# Get character at index (returns 0 if out of bounds)
F char_at(&self, index: i64) -> i64

# Push character to end
F push_char(&self, c: i64) -> i64

# Grow capacity (internal)
F grow(&self) -> i64

# Get raw pointer to data
F as_ptr(&self) -> i64
```

#### Example

```vais
U std/string

F main() -> i64 {
    # Create string
    s := String.with_capacity(32)

    # Add characters
    s.push_char(72)   # 'H'
    s.push_char(101)  # 'e'
    s.push_char(108)  # 'l'
    s.push_char(108)  # 'l'
    s.push_char(111)  # 'o'

    # Get length
    len := s.len()    # 5

    # Print string
    puts_ptr(s.as_ptr())

    0
}
```

---

### Set

**Module:** `std/set.vais`

Hash-based set collection for storing unique `i64` values.

#### Definition

```vais
S Set {
    map: HashMap    # Internally uses HashMap
}
```

#### Functions

```vais
# Create new set with capacity
F set_new(capacity: i64) -> Set

# Insert value into set
F set_insert(s: Set, value: i64) -> i64

# Check if value exists
F set_contains(s: Set, value: i64) -> i64

# Remove value from set
F set_remove(s: Set, value: i64) -> i64

# Get size of set
F set_size(s: Set) -> i64

# Check if set is empty
F set_is_empty(s: Set) -> i64

# Clear all values
F set_clear(s: Set) -> i64

# Free set memory
F set_free(s: Set) -> i64
```

---

### Deque

**Module:** `std/deque.vais`

Double-ended queue (deque) with efficient insertion and removal at both ends. Implemented using a circular buffer.

#### Definition

```vais
S Deque {
    data: i64,      # Pointer to element array
    head: i64,      # Index of first element
    tail: i64,      # Index after last element
    len: i64,       # Number of elements
    cap: i64        # Capacity
}
```

#### Functions

```vais
# Create new deque with capacity
F deque_new(capacity: i64) -> Deque

# Push element to front
F deque_push_front(dq: Deque, value: i64) -> i64

# Push element to back
F deque_push_back(dq: Deque, value: i64) -> i64

# Pop element from front
F deque_pop_front(dq: Deque) -> i64

# Pop element from back
F deque_pop_back(dq: Deque) -> i64

# Get element at index
F deque_get(dq: Deque, index: i64) -> i64

# Get size of deque
F deque_size(dq: Deque) -> i64

# Check if deque is empty
F deque_is_empty(dq: Deque) -> i64

# Free deque memory
F deque_free(dq: Deque) -> i64
```

---

### PriorityQueue

**Module:** `std/priority_queue.vais`

Min-heap based priority queue where smaller values have higher priority. Provides efficient access to the minimum element.

#### Definition

```vais
S PriorityQueue {
    data: i64,      # Pointer to element array
    size: i64,      # Current number of elements
    capacity: i64   # Allocated capacity
}
```

#### Methods

```vais
# Create new PriorityQueue with capacity
F with_capacity(capacity: i64) -> PriorityQueue

# Get number of elements
F len(&self) -> i64

# Get capacity
F capacity(&self) -> i64

# Check if empty
F is_empty(&self) -> i64

# Peek at minimum element (highest priority)
# Returns 0 if empty
F peek(&self) -> i64

# Push element into priority queue
F push(&self, value: i64) -> i64

# Pop minimum element (highest priority)
# Returns 0 if empty
F pop(&self) -> i64

# Clear all elements
F clear(&self) -> i64

# Free memory
F drop(&self) -> i64
```

#### Helper Functions

```vais
# Create new PriorityQueue with default capacity (8)
F pq_new() -> PriorityQueue

# Push element
F pq_push(pq: PriorityQueue, value: i64) -> i64

# Pop minimum element
F pq_pop(pq: PriorityQueue) -> i64

# Peek at minimum element
F pq_peek(pq: PriorityQueue) -> i64

# Get size
F pq_size(pq: PriorityQueue) -> i64

# Check if empty
F pq_is_empty(pq: PriorityQueue) -> i64

# Clear all elements
F pq_clear(pq: PriorityQueue) -> i64

# Free memory
F pq_free(pq: PriorityQueue) -> i64
```

#### Example

```vais
U std/priority_queue

F main() -> i64 {
    # Create priority queue
    pq := PriorityQueue.with_capacity(10)

    # Push elements in random order
    pq.push(50)
    pq.push(30)
    pq.push(70)
    pq.push(10)
    pq.push(40)

    # Peek at minimum (highest priority)
    min := pq.peek()    # Returns 10
    print_i64(min)

    # Pop elements in ascending order
    v1 := pq.pop()      # 10
    v2 := pq.pop()      # 30
    v3 := pq.pop()      # 40

    print_i64(v1)
    print_i64(v2)
    print_i64(v3)

    # Check size
    size := pq.len()    # 2 (50 and 70 remain)
    print_i64(size)

    # Free memory
    pq.drop()

    0
}
```

#### Use Cases

- **Task Scheduling**: Process tasks by priority
- **Dijkstra's Algorithm**: Find shortest paths in graphs
- **Event Simulation**: Process events in time order
- **Huffman Coding**: Build optimal prefix codes
- **Median Maintenance**: Efficiently track median of stream

#### Implementation Notes

- Min-heap implementation (smaller values = higher priority)
- For max-heap behavior, insert negated values
- O(log n) push and pop operations
- O(1) peek operation
- Dynamically grows capacity when full

---

## Smart Pointers

### Box

**Module:** `std/box.vais`

Heap-allocated value with unique ownership (similar to Rust's `Box<T>`).

#### Definition

```vais
S Box {
    ptr: i64    # Pointer to heap-allocated value
}
```

#### Methods

```vais
# Create new Box with value
F new(value: i64) -> Box

# Create Box with custom size
F with_size(value: i64, size: i64) -> Box

# Get the inner value
F get(&self) -> i64

# Set the inner value
F set(&self, value: i64) -> i64

# Get raw pointer
F as_ptr(&self) -> i64

# Free the box (manual)
F free(&self) -> i64
```

#### Example

```vais
U std/box

F main() -> i64 {
    # Create boxed value
    b := Box.new(42)

    # Get value
    value := b.get()  # 42

    # Set new value
    b.set(100)

    # Get updated value
    new_value := b.get()  # 100

    # Manually free (in real usage, would be automatic)
    b.free()

    print_i64(value)
    0
}
```

---

### Rc

**Module:** `std/rc.vais`

Reference-counted smart pointer for shared ownership (single-threaded).

#### Definition

```vais
S Rc {
    ptr: i64    # Pointer to ref-counted value
}

S Weak {
    ptr: i64    # Weak reference (non-owning)
}
```

#### Methods

```vais
# Create new Rc with value
F new(value: i64, value_size: i64) -> Rc

# Clone Rc (increment ref count)
F clone(&self) -> Rc

# Get the inner value
F get(&self) -> i64

# Set the inner value
F set(&self, value: i64) -> i64

# Get current reference count
F ref_count(&self) -> i64

# Get raw pointer
F as_ptr(&self) -> i64

# Create weak reference
F downgrade(&self) -> Weak

# Drop Rc (decrement ref count, free if zero)
F drop(&self) -> i64
```

#### Example

```vais
U std/rc

F main() -> i64 {
    # Create reference-counted value
    rc1 := Rc.new(42, 8)

    # Clone (increment ref count)
    rc2 := rc1.clone()

    # Both point to same value
    val1 := rc1.get()  # 42
    val2 := rc2.get()  # 42

    # Ref count is 2
    count := rc1.ref_count()  # 2

    # Modify value
    rc1.set(100)
    val3 := rc2.get()  # 100 (same value)

    print_i64(count)
    0
}
```

---

## Memory Management

### Arena

**Module:** `std/arena.vais`

Fast batch memory allocator. Allocates memory in chunks, frees all at once.

#### Definition

```vais
S Arena {
    chunks: i64,        # Pointer to chunk list
    chunk_count: i64,   # Number of chunks
    chunk_size: i64,    # Size of each chunk
    current: i64,       # Current chunk pointer
    offset: i64         # Current offset in chunk
}
```

#### Constants

```vais
F ARENA_DEFAULT_CHUNK_SIZE() -> i64  # 65536 (64KB)
```

#### Methods

```vais
# Create new arena with default chunk size
F new() -> Arena

# Create arena with custom chunk size
F with_chunk_size(size: i64) -> Arena

# Allocate memory from arena
F alloc(&self, size: i64) -> i64

# Reset arena (free all allocations)
F reset(&self) -> i64

# Destroy arena (free all memory)
F destroy(&self) -> i64
```

#### Example

```vais
U std/arena

F main() -> i64 {
    # Create arena
    arena := Arena.new()

    # Allocate multiple values
    ptr1 := arena.alloc(64)
    ptr2 := arena.alloc(128)
    ptr3 := arena.alloc(256)

    # Use allocated memory
    store_i64(ptr1, 42)
    store_i64(ptr2, 100)

    value1 := load_i64(ptr1)

    # Reset arena (free all at once)
    arena.reset()

    # Destroy arena
    arena.destroy()

    print_i64(value1)
    0
}
```

---

## I/O

### IO Module

**Module:** `std/io.vais`

Input/output operations for reading from stdin.

#### Constants

```vais
C INPUT_BUFFER_SIZE: i64 = 1024
```

#### Functions

```vais
# Read a line from stdin
F read_line(buffer: i64, max_len: i64) -> i64

# Read line as String
F read_line_string(max_len: i64) -> String

# Read i64 integer from stdin
F read_i64() -> i64

# Read f64 float from stdin
F read_f64() -> f64

# Read a word (space-delimited)
F read_word() -> i64

# Read a single character
F read_char() -> i64

# Print prompt and read line
F prompt_line(prompt: i64, buffer: i64, max_len: i64) -> i64

# Print prompt and read i64
F prompt_i64(prompt: i64) -> i64

# Print prompt and read f64
F prompt_f64(prompt: i64) -> f64
```

#### Example

```vais
U std/io

F main() -> i64 {
    # Read integer
    puts("Enter a number: ")
    num := read_i64()

    # Read float
    puts("Enter a decimal: ")
    decimal := read_f64()

    # Use prompt functions
    age := prompt_i64("Enter your age: ")
    height := prompt_f64("Enter your height: ")

    puts("You entered:")
    print_i64(num)
    print_f64(decimal)
    print_i64(age)
    print_f64(height)

    0
}
```

---

### File

**Module:** `std/file.vais`

File I/O operations for reading and writing files.

#### Definition

```vais
S File {
    handle: i64,    # FILE* pointer
    mode: i64       # 0=closed, 1=read, 2=write, 3=append
}
```

#### Methods

```vais
# Open file for reading
F open_read(path: i64) -> File

# Open file for writing (creates/truncates)
F open_write(path: i64) -> File

# Open file for appending
F open_append(path: i64) -> File

# Check if file is open
F is_open(&self) -> i64

# Read from file into buffer
F read(&self, buffer: i64, size: i64) -> i64

# Write to file from buffer
F write(&self, buffer: i64, size: i64) -> i64

# Read entire file as String
F read_to_string(&self) -> String

# Write string to file
F write_string(&self, s: String) -> i64

# Close file
F close(&self) -> i64
```

#### Example

```vais
U std/file

F main() -> i64 {
    # Write to file
    file := File.open_write("output.txt")
    I file.is_open() {
        file.write("Hello, World!", 13)
        file.close()
    }

    # Read from file
    file2 := File.open_read("output.txt")
    I file2.is_open() {
        buffer := malloc(1024)
        bytes := file2.read(buffer, 1024)
        puts_ptr(buffer)
        file2.close()
        free(buffer)
    }

    0
}
```

---

## Mathematics

### Math

**Module:** `std/math.vais`

Mathematical functions and constants.

#### Constants

```vais
C PI: f64 = 3.141592653589793
C E: f64 = 2.718281828459045
C TAU: f64 = 6.283185307179586
```

#### Basic Functions

```vais
# Absolute value (i64)
F abs_i64(x: i64) -> i64

# Absolute value (f64)
F abs(x: f64) -> f64

# Minimum (i64)
F min_i64(a: i64, b: i64) -> i64

# Minimum (f64)
F min(a: f64, b: f64) -> f64

# Maximum (i64)
F max_i64(a: i64, b: i64) -> i64

# Maximum (f64)
F max(a: f64, b: f64) -> f64

# Clamp value between min and max (i64)
F clamp_i64(x: i64, min_val: i64, max_val: i64) -> i64

# Clamp value between min and max (f64)
F clamp(x: f64, min_val: f64, max_val: f64) -> f64
```

#### Advanced Math (C library functions)

```vais
# Square root
X F sqrt(x: f64) -> f64

# Power (x raised to y)
X F pow(x: f64, y: f64) -> f64

# Floor (round down)
X F floor(x: f64) -> f64

# Ceiling (round up)
X F ceil(x: f64) -> f64

# Round to nearest integer
X F round(x: f64) -> f64
```

#### Trigonometric Functions

```vais
# Sine
X F sin(x: f64) -> f64

# Cosine
X F cos(x: f64) -> f64

# Tangent
X F tan(x: f64) -> f64

# Arc sine
X F asin(x: f64) -> f64

# Arc cosine
X F acos(x: f64) -> f64

# Arc tangent
X F atan(x: f64) -> f64

# Arc tangent (two-argument form)
X F atan2(y: f64, x: f64) -> f64
```

#### Logarithmic and Exponential

```vais
# Natural logarithm (base e)
X F log(x: f64) -> f64

# Base-10 logarithm
X F log10(x: f64) -> f64

# Base-2 logarithm
X F log2(x: f64) -> f64

# Exponential (e^x)
X F exp(x: f64) -> f64
```

#### Helper Functions

```vais
# Convert degrees to radians
F deg_to_rad(degrees: f64) -> f64

# Convert radians to degrees
F rad_to_deg(radians: f64) -> f64

# Check if numbers are approximately equal
F approx_eq(a: f64, b: f64, epsilon: f64) -> i64
```

#### Example

```vais
U std/math

F main() -> i64 {
    # Use constants
    circle_circumference := 2.0 * PI * 5.0

    # Basic math
    absolute := abs(-10.5)
    maximum := max(10.0, 20.0)

    # Advanced math
    root := sqrt(16.0)      # 4.0
    power := pow(2.0, 8.0)  # 256.0

    # Trigonometry
    sine := sin(PI / 2.0)   # 1.0
    cosine := cos(0.0)      # 1.0

    # Logarithms
    ln := log(E)            # 1.0
    log_10 := log10(100.0)  # 2.0

    print_f64(root)
    print_f64(sine)
    print_f64(ln)

    0
}
```

---

## Async Runtime

### Future

**Module:** `std/future.vais`

Future type and async runtime support for stackless coroutines.

#### Definition

```vais
E Poll {
    Pending,
    Ready(i64)
}

W Future {
    F poll(&self, ctx: i64) -> Poll
}

S Context {
    waker_ptr: i64,
    runtime_ptr: i64
}
```

#### Poll Methods

```vais
# Check if ready
F is_ready(&self) -> i64

# Check if pending
F is_pending(&self) -> i64

# Unwrap ready value
F unwrap(&self) -> i64
```

#### Example

```vais
U std/future

# Async function
A F compute(x: i64) -> i64 {
    x * 2
}

F main() -> i64 {
    # Call async and await
    result := compute(21).await

    print_i64(result)  # 42
    0
}
```

---

### Runtime

**Module:** `std/runtime.vais`

Runtime support for spawning and managing async tasks.

#### Functions

```vais
# Spawn a task (returns task handle)
F spawn_task(future: i64) -> i64

# Block on a future until completion
F block_on(future: i64) -> i64

# Run the runtime executor
F run_executor() -> i64
```

---

## Traits

### Iterator

**Module:** `std/iter.vais`

Iterator trait for sequential access to collections.

#### Definition

```vais
W Iterator {
    # Get next value, returns Option
    F next(&self) -> Option
}
```

#### Methods (when implemented)

```vais
# Map over iterator values
F map(&self, f: i64) -> Iterator

# Filter iterator values
F filter(&self, predicate: i64) -> Iterator

# Collect into Vec
F collect(&self) -> Vec

# Count elements
F count(&self) -> i64

# Fold/reduce
F fold(&self, init: i64, f: i64) -> i64
```

---

## Built-in Functions

These functions are provided by the compiler runtime and available without imports:

### I/O Functions

```vais
# Print string literal with newline
F puts(s: str) -> i64

# Print C string from pointer
F puts_ptr(ptr: i64) -> i64

# Print single character (ASCII value)
F putchar(c: i64) -> i64

# Print i64 integer
F print_i64(n: i64) -> i64

# Print f64 float
F print_f64(n: f64) -> i64
```

### Memory Functions

```vais
# Allocate memory (returns pointer)
F malloc(size: i64) -> i64

# Free memory
F free(ptr: i64) -> i64

# Copy memory
F memcpy(dst: i64, src: i64, n: i64) -> i64

# Load i64 from memory
F load_i64(ptr: i64) -> i64

# Store i64 to memory
F store_i64(ptr: i64, val: i64) -> i64

# Load byte from memory
F load_byte(ptr: i64) -> i64

# Store byte to memory
F store_byte(ptr: i64, val: i64) -> i64
```

### String Functions

```vais
# Get string length (null-terminated)
F strlen(s: i64) -> i64
```

---

## Module Index

| Module | Purpose | Key Types |
|--------|---------|-----------|
| `std/option` | Optional values | `Option<T>` |
| `std/result` | Error handling | `Result<T, E>` |
| `std/vec` | Dynamic arrays | `Vec<T>` |
| `std/hashmap` | Hash tables | `HashMap<K, V>` |
| `std/string` | String handling | `String` |
| `std/set` | Hash sets | `Set<T>` |
| `std/deque` | Double-ended queue | `Deque<T>` |
| `std/priority_queue` | Priority queue (min-heap) | `PriorityQueue<T>` |
| `std/box` | Unique ownership | `Box<T>` |
| `std/rc` | Reference counting | `Rc<T>`, `Weak<T>` |
| `std/arena` | Batch allocation | `Arena` |
| `std/io` | Input operations | I/O functions |
| `std/file` | File operations | `File` |
| `std/math` | Mathematics | Math functions |
| `std/future` | Async runtime | `Future`, `Poll` |
| `std/runtime` | Task execution | Runtime functions |
| `std/iter` | Iteration | `Iterator` trait |
| `std/net` | Networking | `TcpListener`, `TcpStream`, `UdpSocket` |

---

## Usage Patterns

### Error Handling

```vais
U std/result

F safe_operation(x: i64) -> Result {
    I x < 0 {
        err(ERR_INVALID())
    } E {
        ok(x * 2)
    }
}

F main() -> i64 {
    result := safe_operation(10)
    M result {
        Ok(v) => print_i64(v),
        Err(e) => puts("Error occurred")
    }
    0
}
```

### Working with Collections

```vais
U std/vec

F main() -> i64 {
    v := Vec.with_capacity(10)

    # Add elements
    L i: 0..10 {
        v.push(i * i)
    }

    # Process elements
    L i: 0..v.len() {
        value := v.get(i)
        print_i64(value)
    }

    0
}
```

### File I/O

```vais
U std/file

F read_config(path: i64) -> i64 {
    file := File.open_read(path)
    I !file.is_open() {
        puts("Error: Could not open file")
        R -1
    }

    buffer := malloc(1024)
    bytes := file.read(buffer, 1024)
    puts_ptr(buffer)

    file.close()
    free(buffer)
    0
}
```

---

## Best Practices

1. **Use Option for nullable values** instead of special sentinel values
2. **Use Result for operations that can fail** to provide error context
3. **Prefer Vec over raw arrays** for dynamic collections
4. **Use Rc for shared ownership** when multiple references are needed
5. **Use Arena for temporary allocations** with the same lifetime
6. **Import only needed modules** to minimize dependencies
7. **Check file operations** for success before proceeding
8. **Free allocated memory** when using manual memory management

---

## Networking

**Module**: `std/net`

Network operations using BSD socket API. Supports both IPv4 and IPv6 protocols for TCP and UDP communication.

### Constants

#### Address Families
```vais
C AF_INET: i64 = 2           # IPv4
C AF_INET6: i64 = 30         # IPv6 (macOS: 30, Linux: 10)
```

#### Socket Types
```vais
C SOCK_STREAM: i64 = 1       # TCP
C SOCK_DGRAM: i64 = 2        # UDP
```

#### Socket Options
```vais
C IPPROTO_IPV6: i64 = 41     # IPv6 protocol
C IPV6_V6ONLY: i64 = 27      # IPv6-only socket option
```

### TcpListener - TCP Server Socket

#### Methods

**`TcpListener.bind(port: i64) -> TcpListener`**
Creates and binds an IPv4 TCP listener on the specified port.

```vais
listener := TcpListener.bind(8080)
I listener.is_valid() {
    println("Listening on port 8080")
}
```

**`TcpListener.bind6(port: i64) -> TcpListener`**
Creates and binds an IPv6 TCP listener on the specified port.

```vais
listener := TcpListener.bind6(8080)
I listener.is_valid() {
    println("Listening on IPv6 port 8080")
}
```

**`listener.accept() -> TcpStream`**
Accepts an incoming connection and returns a TcpStream.

**`listener.is_valid() -> i64`**
Returns 1 if listener is valid, 0 otherwise.

**`listener.close() -> i64`**
Closes the listener socket.

### TcpStream - TCP Connection

#### Methods

**`TcpStream.connect(host: *i8, port: i64) -> TcpStream`**
Connects to a remote IPv4 host.

```vais
stream := TcpStream.connect("127.0.0.1", 8080)
I stream.is_valid() {
    stream.write("Hello", 5)
}
```

**`TcpStream.connect6(host: *i8, port: i64) -> TcpStream`**
Connects to a remote IPv6 host.

```vais
stream := TcpStream.connect6("::1", 8080)
I stream.is_valid() {
    stream.write("Hello", 5)
}
```

**`stream.read(buffer: *i8, len: i64) -> i64`**
Reads data into buffer. Returns bytes read, 0 on connection close, -1 on error.

**`stream.write(data: *i8, len: i64) -> i64`**
Writes data to stream. Returns bytes written, -1 on error.

**`stream.write_all(data: *i8, len: i64) -> i64`**
Writes all data, looping until complete. Returns total bytes written.

**`stream.is_valid() -> i64`**
Returns 1 if stream is valid, 0 otherwise.

**`stream.close() -> i64`**
Closes the stream.

### UdpSocket - UDP Socket

#### Methods

**`UdpSocket.new() -> UdpSocket`**
Creates an unbound IPv4 UDP socket.

**`UdpSocket.new6() -> UdpSocket`**
Creates an unbound IPv6 UDP socket.

**`UdpSocket.bind(port: i64) -> UdpSocket`**
Creates and binds an IPv4 UDP socket to a port.

```vais
socket := UdpSocket.bind(9000)
I socket.is_valid() {
    println("UDP socket bound to port 9000")
}
```

**`UdpSocket.bind6(port: i64) -> UdpSocket`**
Creates and binds an IPv6 UDP socket to a port.

**`socket.send_to(data: *i8, len: i64, host: *i8, port: i64) -> i64`**
Sends data to an IPv4 address. Returns bytes sent, -1 on error.

**`socket.send_to6(data: *i8, len: i64, host: *i8, port: i64) -> i64`**
Sends data to an IPv6 address. Returns bytes sent, -1 on error.

**`socket.recv(buffer: *i8, len: i64) -> i64`**
Receives data without source address info.

**`socket.recv_from(buffer: *i8, len: i64, src_addr_out: *i8, src_port_out: *i64) -> i64`**
Receives data with IPv4 source address info. Buffer for src_addr_out should be at least 16 bytes.

**`socket.recv_from6(buffer: *i8, len: i64, src_addr_out: *i8, src_port_out: *i64) -> i64`**
Receives data with IPv6 source address info. Buffer for src_addr_out should be at least 46 bytes.

**`socket.is_valid() -> i64`**
Returns 1 if socket is valid, 0 otherwise.

**`socket.close() -> i64`**
Closes the socket.

### C-Style API Functions

#### TCP Functions

**`tcp_listen(port: i64) -> i64`**
Creates IPv4 TCP listener. Returns file descriptor, -1 on error.

**`tcp_listen6(port: i64) -> i64`**
Creates IPv6 TCP listener. Returns file descriptor, -1 on error.

**`tcp_connect(host: *i8, port: i64) -> i64`**
Connects to IPv4 TCP server. Returns file descriptor, -1 on error.

**`tcp_connect6(host: *i8, port: i64) -> i64`**
Connects to IPv6 TCP server. Returns file descriptor, -1 on error.

**`tcp_accept(listener_fd: i64) -> i64`**
Accepts connection. Returns client socket fd, -1 on error.

**`tcp_read(stream_fd: i64, buffer: *i8, len: i64) -> i64`**
Reads from TCP socket.

**`tcp_write(stream_fd: i64, data: *i8, len: i64) -> i64`**
Writes to TCP socket.

**`tcp_close(stream_fd: i64) -> i64`**
Closes TCP socket.

#### UDP Functions

**`udp_bind(port: i64) -> i64`**
Creates and binds IPv4 UDP socket. Returns file descriptor, -1 on error.

**`udp_bind6(port: i64) -> i64`**
Creates and binds IPv6 UDP socket. Returns file descriptor, -1 on error.

**`udp_send_to(socket_fd: i64, data: *i8, len: i64, host: *i8, port: i64) -> i64`**
Sends data via IPv4 UDP.

**`udp_send_to6(socket_fd: i64, data: *i8, len: i64, host: *i8, port: i64) -> i64`**
Sends data via IPv6 UDP.

**`udp_recv_from(socket_fd: i64, buffer: *i8, len: i64) -> i64`**
Receives data via UDP.

**`udp_close(socket_fd: i64) -> i64`**
Closes UDP socket.

### Helper Functions

**`make_sockaddr_in(host: *i8, port: i64) -> *sockaddr_in`**
Creates IPv4 sockaddr_in structure. Caller must free.

**`make_sockaddr_in6(host: *i8, port: i64) -> *sockaddr_in6`**
Creates IPv6 sockaddr_in6 structure. Caller must free.

**`make_sockaddr_any(port: i64) -> *sockaddr_in`**
Creates IPv4 sockaddr for any address (0.0.0.0).

**`make_sockaddr_any6(port: i64) -> *sockaddr_in6`**
Creates IPv6 sockaddr for any address (::).

**`is_valid_ip(host: *i8) -> i64`**
Checks if IPv4 address string is valid. Returns 1 if valid, 0 otherwise.

**`is_valid_ip6(host: *i8) -> i64`**
Checks if IPv6 address string is valid. Returns 1 if valid, 0 otherwise.

**`net_error_string(err: i64) -> *i8`**
Converts error code to string description.

### Examples

#### TCP Echo Server (IPv6)

```vais
F main() -> i64 {
    # Create IPv6 TCP listener
    listener := TcpListener.bind6(8080)

    I !listener.is_valid() {
        println("Failed to create listener")
        1
    } E {
        println("Listening on [::]:8080")

        # Accept connection
        client := listener.accept()
        I client.is_valid() {
            # Read message
            buffer := malloc(1024)
            bytes_read := client.read(buffer, 1024)

            I bytes_read > 0 {
                # Echo back
                client.write_all(buffer, bytes_read)
            }

            free(buffer)
            client.close()
        }

        listener.close()
        0
    }
}
```

#### UDP Client (IPv6)

```vais
F main() -> i64 {
    socket := UdpSocket.new6()

    I socket.is_valid() {
        msg := "Hello UDP!"
        socket.send_to6(msg, 10, "::1", 9000)
        socket.close()
    }

    0
}
```

#### Dual-Stack Server

```vais
# IPv6 sockets can accept both IPv4 and IPv6 connections by default
# IPv4 clients appear as IPv4-mapped IPv6 addresses (::ffff:x.x.x.x)
F main() -> i64 {
    listener := TcpListener.bind6(8080)  # Accepts both IPv4 and IPv6
    println("Dual-stack server listening on port 8080")
    listener.close()
    0
}
```

---

## Future Additions

The following modules are planned for future versions:

- `std/collections` - BTreeMap, additional collection types
- `std/thread` - Threading support
- `std/sync` - Synchronization primitives (Mutex, RwLock)
- `std/path` - Path manipulation
- `std/env` - Environment variables
- `std/time` - Time and duration
- `std/regex` - Regular expressions

---

For complete language reference, see `LANGUAGE_SPEC.md`. For tutorials and examples, see `TUTORIAL.md`.
