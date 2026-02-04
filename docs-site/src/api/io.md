# IO API Reference

> Standard input/output operations for reading from stdin

## Import

```vais
U std/io
```

## Overview

The IO module provides safe input operations with built-in buffer overflow protection. All input functions validate buffer parameters and enforce size limits to prevent memory corruption. The module depends on `std/option` for optional value handling.

## Constants

| Name | Value | Description |
|------|-------|-------------|
| `INPUT_BUFFER_SIZE` | 1024 | Default buffer size for input operations |

## Core Functions

### read_line

```vais
F read_line(buffer: i64, max_len: i64) -> i64
```

Read a line from stdin into a buffer, removing trailing newline.

**Parameters:**
- `buffer`: Destination buffer (must be pre-allocated)
- `max_len`: Maximum length including null terminator (1..=1048576)

**Returns:** Pointer to buffer on success, `0` on EOF/error

**Safety:** Caller must ensure buffer has at least `max_len` bytes allocated. The function validates `max_len` and clamps values above 1048576.

---

### read_i64

```vais
F read_i64() -> i64
```

Read an integer from stdin.

**Returns:** The parsed integer value, or `0` if invalid input

**Example:**
```vais
num := read_i64()
```

---

### read_f64

```vais
F read_f64() -> f64
```

Read a floating-point number from stdin.

**Returns:** The parsed float value, or `0.0` if invalid input

**Example:**
```vais
price := read_f64()
```

---

### read_char

```vais
F read_char() -> i64
```

Read a single character from stdin.

**Returns:** The character as `i64`, or `-1` on EOF

---

### prompt_line

```vais
F prompt_line(prompt: i64, buffer: i64, max_len: i64) -> i64
```

Print a prompt and read a line from stdin.

**Parameters:**
- `prompt`: Null-terminated string to display
- `buffer`: Destination buffer
- `max_len`: Maximum length

**Returns:** Pointer to buffer on success, `0` on error

**Example:**
```vais
buf := malloc(256)
prompt_line("Enter your name: ", buf, 256)
free(buf)
```

---

### prompt_i64

```vais
F prompt_i64(prompt: i64) -> i64
```

Print a prompt and read an integer.

**Parameters:**
- `prompt`: Null-terminated string to display

**Returns:** The integer value

**Example:**
```vais
age := prompt_i64("Enter age: ")
```

---

### prompt_f64

```vais
F prompt_f64(prompt: i64) -> f64
```

Print a prompt and read a float.

**Parameters:**
- `prompt`: Null-terminated string to display

**Returns:** The float value

**Example:**
```vais
height := prompt_f64("Enter height (m): ")
```

## Extern C Functions

The following C library functions are available for advanced use:

| Function | Signature | Description |
|----------|-----------|-------------|
| `get_stdin` | `X F get_stdin() -> i64` | Get stdin file handle |
| `fgets_ptr` | `X F fgets_ptr(buffer: i64, size: i64, stream: i64) -> i64` | Read line from stream |
| `atol_ptr` | `X F atol_ptr(s: i64) -> i64` | Convert string to i64 |
| `atof_ptr` | `X F atof_ptr(s: i64) -> f64` | Convert string to f64 |
| `getchar` | `X F getchar() -> i64` | Read single character |

## Usage Examples

### Basic Input

```vais
U std/io

F main() -> i64 {
    # Read a number
    age := prompt_i64("Enter your age: ")

    # Read a float
    gpa := prompt_f64("Enter GPA: ")

    # Read a character
    c := read_char()

    0
}
```

### Reading Lines

```vais
U std/io

F main() -> i64 {
    buffer := malloc(256)

    # Read a line with prompt
    result := prompt_line("Enter text: ", buffer, 256)

    I result != 0 {
        # Successfully read input
        puts_ptr(buffer)
    }

    free(buffer)
    0
}
```

### Input Validation Loop

```vais
U std/io

F main() -> i64 {
    buf := malloc(100)
    valid := 0

    L valid == 0 {
        prompt_line("Enter positive number: ", buf, 100)
        num := atol_ptr(buf)
        I num > 0 {
            valid = 1
        }
    }

    free(buf)
    0
}
```

## Safety Notes

- All buffers passed to `read_line` and `prompt_line` must be pre-allocated with at least `max_len` bytes
- The `max_len` parameter is validated and clamped to the range 1..=1048576
- Trailing newlines are automatically removed from input
- Memory allocated by these functions must be freed by the caller
- String-based functions (currently disabled) require importing `std/string`
