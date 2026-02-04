# FFI Features

## Overview

The Vais Foreign Function Interface (FFI) enables seamless interoperability with C, C++, and other languages, allowing you to call external libraries and expose Vais functions to other languages.

## Core Features

### 1. C Function Calls

Call C functions directly from Vais code:

```vais
# Declare external C function
extern F printf(fmt: *i8, ...) -> i64

F main() -> i64 {
    printf("Hello from Vais!\n")
    0
}
```

### 2. Type Mapping

Automatic type conversion between Vais and C types:

| Vais Type | C Type |
|-----------|--------|
| i8 | int8_t |
| i16 | int16_t |
| i32 | int32_t |
| i64 | int64_t |
| u8 | uint8_t |
| u16 | uint16_t |
| u32 | uint32_t |
| u64 | uint64_t |
| f32 | float |
| f64 | double |
| *T | T* |
| bool | bool |

### 3. Struct Interop

Pass Vais structs to C functions:

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

### 4. Library Linking

Link with external libraries:

```bash
vaisc build program.vais -l m -l pthread
```

### 5. Callback Functions

Pass Vais functions to C as callbacks:

```vais
F compare(a: *i64, b: *i64) -> i64 {
    *a - *b
}

extern F qsort(base: *void, num: u64, size: u64, cmp: fn(*void, *void) -> i64) -> void

F main() -> i64 {
    arr := [5, 2, 8, 1, 9]
    qsort(&arr, 5, 8, compare)
    0
}
```

## Advanced Features

### 1. Variable Arguments

Support for variadic functions:

```vais
extern F printf(fmt: *i8, ...) -> i64
extern F fprintf(file: *void, fmt: *i8, ...) -> i64
```

### 2. Opaque Pointers

Handle external types without knowing their layout:

```vais
extern F fopen(path: *i8, mode: *i8) -> *void
extern F fclose(file: *void) -> i64

F main() -> i64 {
    f := fopen("test.txt", "r")
    # Use file...
    fclose(f)
    0
}
```

### 3. Binding Generation

Automatically generate FFI bindings from C headers:

```bash
vaisc bindgen header.h -o bindings.vais
```

See `crates/vais-bindgen/` for details.

### 4. C++ Support

Call C++ functions with name mangling support:

```vais
# C++ function: void process(int x);
extern F _Z7processi(x: i32) -> void

# Or use bindgen for automatic mangling
```

See `crates/vais-bindgen/CPP_SUPPORT.md` for details.

## Standard Library FFI

The Vais standard library uses FFI extensively:

### File I/O
```vais
U std/fs

F main() -> i64 {
    file := fs_open("data.txt", "r")
    # Uses fopen, fread, fclose internally
    fs_close(file)
    0
}
```

### Networking
```vais
U std/net

F main() -> i64 {
    sock := net_socket(AF_INET, SOCK_STREAM, 0)
    # Uses socket, bind, listen internally
    0
}
```

## Safety Considerations

### 1. Null Pointer Checks

Always validate pointers from C:

```vais
F safe_call() -> i64 {
    ptr := c_function_returning_ptr()
    I ptr == null {
        printf("Error: null pointer\n")
        R -1
    }
    # Use ptr safely
    0
}
```

### 2. Buffer Bounds

Check buffer sizes before operations:

```vais
F safe_copy(src: *i8, dst: *i8, max: u64) -> i64 {
    I strlen(src) >= max {
        R -1  # Buffer too small
    }
    strcpy(dst, src)
    0
}
```

### 3. Memory Management

Track ownership of FFI-allocated memory:

```vais
F process_data() -> i64 {
    data := malloc(1024)
    # Use data...
    free(data)  # Don't forget to free!
    0
}
```

## Platform-Specific FFI

### Windows
```vais
#[windows]
extern F GetCurrentProcessId() -> u32

F main() -> i64 {
    pid := GetCurrentProcessId()
    printf("PID: %u\n", pid)
    0
}
```

### POSIX
```vais
#[unix]
extern F getpid() -> i32

F main() -> i64 {
    pid := getpid()
    printf("PID: %d\n", pid)
    0
}
```

## Examples

See `examples/` directory for complete examples:
- `examples/ffi_basic.vais` - Basic FFI usage
- `examples/ffi_struct.vais` - Struct passing
- `examples/ffi_callback.vais` - Callback functions
- `examples/filesystem_ffi.vais` - File I/O with FFI

## Documentation

- **Implementation**: `FFI_IMPLEMENTATION_SUMMARY.md`
- **User Guide**: `FFI_GUIDE.md`
- **Bindgen**: `crates/vais-bindgen/README.md`
- **C++ Support**: `crates/vais-bindgen/CPP_SUPPORT.md`

## Limitations

1. **Name Mangling**: C++ requires manual mangling or bindgen
2. **Complex Types**: Templates and generics not directly supported
3. **Exceptions**: C++ exceptions not handled (use error codes)
4. **ABI**: Assumes C ABI for all extern functions

## Future Enhancements

- Automatic C++ name demangling
- Support for COM on Windows
- Objective-C bridge for macOS/iOS
- Java/JNI integration
- Python C API integration

## Status

FFI is fully implemented and production-ready. See `FFI_IMPLEMENTATION_SUMMARY.md` for implementation details.
