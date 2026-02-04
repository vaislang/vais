# FFI User Guide

## Introduction

This guide will help you use the Vais Foreign Function Interface (FFI) to call C and C++ code from your Vais programs.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic FFI](#basic-ffi)
3. [Type Mappings](#type-mappings)
4. [Working with Structs](#working-with-structs)
5. [Pointers and Memory](#pointers-and-memory)
6. [Callbacks](#callbacks)
7. [Linking Libraries](#linking-libraries)
8. [Using Bindgen](#using-bindgen)
9. [C++ Interop](#c-interop)
10. [Best Practices](#best-practices)

## Getting Started

### Your First FFI Call

Let's call the C `strlen` function:

```vais
# Declare the external function
extern F strlen(s: *i8) -> u64

F main() -> i64 {
    text := "Hello, FFI!"
    length := strlen(text)
    printf("String length: %llu\n", length)
    0
}
```

Compile and run:
```bash
vaisc build example.vais
./example
# Output: String length: 11
```

## Basic FFI

### Declaring External Functions

Use the `extern` keyword:

```vais
extern F function_name(param1: Type1, param2: Type2) -> ReturnType
```

Example with multiple parameters:
```vais
extern F memcpy(dest: *void, src: *void, n: u64) -> *void

F main() -> i64 {
    src := "Hello"
    dest := malloc(6)
    memcpy(dest, src, 6)
    printf("%s\n", dest)
    free(dest)
    0
}
```

### Variadic Functions

Functions like `printf` that take variable arguments:

```vais
extern F printf(format: *i8, ...) -> i64
extern F sprintf(buffer: *i8, format: *i8, ...) -> i64

F main() -> i64 {
    printf("Integer: %d, Float: %f, String: %s\n", 42, 3.14, "test")
    0
}
```

## Type Mappings

### Primitive Types

| Vais Type | C Type | Size |
|-----------|--------|------|
| i8 | int8_t | 1 byte |
| i16 | int16_t | 2 bytes |
| i32 | int32_t | 4 bytes |
| i64 | int64_t | 8 bytes |
| u8 | uint8_t | 1 byte |
| u16 | uint16_t | 2 bytes |
| u32 | uint32_t | 4 bytes |
| u64 | uint64_t | 8 bytes |
| f32 | float | 4 bytes |
| f64 | double | 8 bytes |
| bool | bool | 1 byte |
| *T | T* | 8 bytes (64-bit) |

### Special Types

- `*i8` - C string (char*)
- `*void` - Generic pointer (void*)
- `fn(...)` - Function pointer

## Working with Structs

### Compatible Struct Layout

Vais structs are compatible with C structs:

**C header (point.h)**:
```c
struct Point {
    double x;
    double y;
};

double distance(struct Point* p1, struct Point* p2);
```

**Vais code**:
```vais
S Point {
    x: f64,
    y: f64,
}

extern F distance(p1: *Point, p2: *Point) -> f64

F main() -> i64 {
    p1 := Point { x: 0.0, y: 0.0 }
    p2 := Point { x: 3.0, y: 4.0 }
    d := distance(&p1, &p2)
    printf("Distance: %f\n", d)
    0
}
```

### Nested Structs

```vais
S Vec2 {
    x: f64,
    y: f64,
}

S Entity {
    position: Vec2,
    velocity: Vec2,
    id: i32,
}

extern F update_entity(e: *Entity, dt: f64) -> void
```

## Pointers and Memory

### Allocating Memory

Use C's memory allocation functions:

```vais
extern F malloc(size: u64) -> *void
extern F free(ptr: *void) -> void
extern F calloc(count: u64, size: u64) -> *void
extern F realloc(ptr: *void, size: u64) -> *void

F allocate_array(size: i64) -> *i64 {
    malloc(size * 8)  # 8 bytes per i64
}

F main() -> i64 {
    arr := allocate_array(10)
    # Use array...
    free(arr)
    0
}
```

### Null Pointer Checks

Always check for null:

```vais
F safe_malloc(size: u64) -> *void {
    ptr := malloc(size)
    I ptr == null {
        printf("ERROR: Out of memory!\n")
        exit(1)
    }
    ptr
}
```

### Pointer Arithmetic

```vais
F iterate_array(arr: *i64, len: i64) -> i64 {
    i := 0
    L i < len {
        # Access element
        value := *(arr + i)
        printf("%d\n", value)
        i = i + 1
    }
    0
}
```

## Callbacks

### Passing Vais Functions to C

Example with `qsort`:

```vais
# Comparison function
F compare_ints(a: *void, b: *void) -> i32 {
    x := *(a as *i32)
    y := *(b as *i32)
    I x < y { R -1 }
    I x > y { R 1 }
    0
}

extern F qsort(base: *void, num: u64, size: u64,
               compar: fn(*void, *void) -> i32) -> void

F main() -> i64 {
    arr := [5, 2, 8, 1, 9]
    qsort(&arr, 5, 4, compare_ints)

    # Array is now sorted
    i := 0
    L i < 5 {
        printf("%d ", arr[i])
        i = i + 1
    }
    0
}
```

### Function Pointers

```vais
# Type alias for clarity
type Callback = fn(i32) -> void

extern F register_callback(cb: Callback) -> void

F my_callback(value: i32) -> void {
    printf("Callback called with: %d\n", value)
}

F main() -> i64 {
    register_callback(my_callback)
    0
}
```

## Linking Libraries

### Standard Libraries

Link with common libraries:

```bash
# Math library
vaisc build program.vais -l m

# Threads
vaisc build program.vais -l pthread

# Multiple libraries
vaisc build program.vais -l m -l pthread -l ssl
```

### Custom Libraries

Specify library search paths:

```bash
vaisc build program.vais -L /usr/local/lib -l mylib
```

### Static vs Dynamic

```bash
# Prefer static linking
vaisc build program.vais -l mylib --static

# Force dynamic linking
vaisc build program.vais -l mylib --dynamic
```

## Using Bindgen

### Automatic Binding Generation

Generate Vais bindings from C headers:

```bash
vaisc bindgen mylib.h -o mylib.vais
```

### Example

**C header (graphics.h)**:
```c
typedef struct {
    int x, y, width, height;
} Rectangle;

void draw_rectangle(Rectangle* rect);
int get_screen_width(void);
```

**Generate bindings**:
```bash
vaisc bindgen graphics.h -o graphics_bindings.vais
```

**Generated code**:
```vais
S Rectangle {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

extern F draw_rectangle(rect: *Rectangle) -> void
extern F get_screen_width() -> i32
```

**Use in your code**:
```vais
U graphics_bindings

F main() -> i64 {
    rect := Rectangle { x: 10, y: 20, width: 100, height: 50 }
    draw_rectangle(&rect)
    width := get_screen_width()
    printf("Screen width: %d\n", width)
    0
}
```

See `crates/vais-bindgen/README.md` for advanced options.

## C++ Interop

### Calling C++ Functions

Use bindgen for automatic name mangling:

```bash
vaisc bindgen mycpp.hpp -o mycpp.vais --cpp
```

Or manually specify mangled names:

```vais
# C++ function: void process(int x);
extern F _Z7processi(x: i32) -> void

F main() -> i64 {
    _Z7processi(42)
    0
}
```

See `crates/vais-bindgen/CPP_SUPPORT.md` for details.

## Best Practices

### 1. Check Return Values

```vais
F safe_file_open(path: *i8) -> *void {
    file := fopen(path, "r")
    I file == null {
        printf("ERROR: Could not open %s\n", path)
        exit(1)
    }
    file
}
```

### 2. Always Free Allocated Memory

```vais
F process_data() -> i64 {
    buffer := malloc(1024)
    I buffer == null {
        R -1
    }

    # Process data...

    free(buffer)  # Don't forget!
    0
}
```

### 3. Use Safe Wrappers

```vais
# Unsafe direct FFI
extern F strcpy(dest: *i8, src: *i8) -> *i8

# Safe wrapper
F safe_strcpy(dest: *i8, src: *i8, max_len: u64) -> i64 {
    I strlen(src) >= max_len {
        R -1  # Error: source too long
    }
    strcpy(dest, src)
    0
}
```

### 4. Document FFI Functions

```vais
# Opens a file for reading.
# Returns null on error.
# Caller must call fclose() when done.
extern F fopen(path: *i8, mode: *i8) -> *void
```

### 5. Handle Platform Differences

```vais
#[unix]
extern F getpid() -> i32

#[windows]
extern F GetCurrentProcessId() -> u32

F get_process_id() -> i64 {
    #[unix]
    R getpid() as i64

    #[windows]
    R GetCurrentProcessId() as i64
}
```

## Common Patterns

### File I/O

```vais
extern F fopen(path: *i8, mode: *i8) -> *void
extern F fread(ptr: *void, size: u64, count: u64, file: *void) -> u64
extern F fclose(file: *void) -> i64

F read_file(path: *i8) -> i64 {
    file := fopen(path, "r")
    I file == null {
        R -1
    }

    buffer := malloc(1024)
    bytes_read := fread(buffer, 1, 1024, file)
    printf("Read %llu bytes\n", bytes_read)

    free(buffer)
    fclose(file)
    0
}
```

### Networking

```vais
extern F socket(domain: i32, type: i32, protocol: i32) -> i32
extern F bind(sockfd: i32, addr: *void, addrlen: u32) -> i32
extern F listen(sockfd: i32, backlog: i32) -> i32
extern F accept(sockfd: i32, addr: *void, addrlen: *u32) -> i32

F create_server(port: i32) -> i32 {
    sock := socket(2, 1, 0)  # AF_INET, SOCK_STREAM
    I sock < 0 {
        R -1
    }
    # Setup and bind...
    sock
}
```

## Troubleshooting

### Undefined Symbol Errors

If you get linker errors:
```bash
# Add library path
vaisc build program.vais -L /path/to/libs -l mylib

# Check if library is installed
pkg-config --libs mylib
```

### Type Mismatch Errors

Ensure types match exactly:
```vais
# Wrong: i32 instead of i64
extern F my_func(x: i32) -> i64

# Correct
extern F my_func(x: i64) -> i64
```

### Segmentation Faults

Common causes:
- Null pointer dereference
- Buffer overflow
- Freeing memory twice
- Using freed memory

Always validate pointers and bounds.

## Further Reading

- **FFI Features**: `FFI_FEATURES.md`
- **Implementation**: `FFI_IMPLEMENTATION_SUMMARY.md`
- **Bindgen**: `crates/vais-bindgen/README.md`
- **C++ Support**: `crates/vais-bindgen/CPP_SUPPORT.md`

## Examples

See the `examples/` directory for complete examples:
- `examples/ffi_basic.vais` - Basic FFI calls
- `examples/ffi_struct.vais` - Struct passing
- `examples/ffi_callback.vais` - Callback functions
- `examples/filesystem_ffi.vais` - File I/O

## Getting Help

If you encounter issues:
1. Check that types match between C and Vais
2. Verify library is linked correctly
3. Test C code separately first
4. Use a debugger (gdb, lldb) to trace issues
5. Consult the documentation

Happy coding with FFI!
