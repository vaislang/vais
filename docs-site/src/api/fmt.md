# Fmt API Reference

> String formatting, number-to-string conversion, and format builders

## Import

```vais
U std/fmt
```

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `FMT_BUF_SIZE` | 256 | Default format buffer size |
| `ALIGN_LEFT` | 0 | Left alignment |
| `ALIGN_CENTER` | 1 | Center alignment |
| `ALIGN_RIGHT` | 2 | Right alignment |

## Number Conversion Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `itoa` | `F itoa(value: i64) -> i64` | Integer to decimal string |
| `itoa_hex` | `F itoa_hex(value: i64) -> i64` | Integer to hex string |
| `itoa_bin` | `F itoa_bin(value: i64) -> i64` | Integer to binary string |
| `itoa_oct` | `F itoa_oct(value: i64) -> i64` | Integer to octal string |
| `format_int` | `F format_int(value: i64) -> i64` | Alias for itoa |
| `format_hex` | `F format_hex(value: i64) -> i64` | Alias for itoa_hex |
| `format_bin` | `F format_bin(value: i64) -> i64` | Alias for itoa_bin |
| `format_oct` | `F format_oct(value: i64) -> i64` | Alias for itoa_oct |
| `strlen` | `F strlen(s: i64) -> i64` | Get string length |
| `strcpy` | `F strcpy(dst: i64, src: i64) -> i64` | Copy string to buffer |
| `fmt_builder` | `F fmt_builder() -> FormatBuilder` | Create new format builder |

## FormatBuilder

Incremental string builder for formatted output.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> FormatBuilder` | Create with default capacity |
| `with_capacity` | `F with_capacity(cap: i64) -> FormatBuilder` | Create with capacity |
| `write_char` | `F write_char(&self, ch: i64) -> i64` | Write single character |
| `write_str` | `F write_str(&self, s: i64) -> i64` | Write string |
| `write_int` | `F write_int(&self, value: i64) -> i64` | Write integer |
| `write_hex` | `F write_hex(&self, value: i64) -> i64` | Write hex integer |
| `write_newline` | `F write_newline(&self) -> i64` | Write newline |
| `write_repeat` | `F write_repeat(&self, ch: i64, count: i64) -> i64` | Write char N times |
| `write_padded_int` | `F write_padded_int(&self, value: i64, width: i64, align: i64, pad: i64) -> i64` | Padded integer |
| `len` | `F len(&self) -> i64` | Get current length |
| `finish` | `F finish(&self) -> i64` | Finalize, return string pointer |
| `reset` | `F reset(&self) -> i64` | Reset for reuse |
| `cleanup` | `F cleanup(&self) -> i64` | Free buffer |

## DebugStruct

Helper for structured debug output.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(name: i64) -> DebugStruct` | Create with struct name |
| `field` | `F field(&self, name: i64, value: i64) -> i64` | Add a field |
| `finish` | `F finish(&self) -> i64` | Finalize output |

## Usage

```vais
U std/fmt

F main() -> i64 {
    fb := FormatBuilder::new()
    fb.write_str("Count: ")
    fb.write_int(42)
    fb.write_newline()
    result := fb.finish()
    puts_ptr(result)
    fb.cleanup()
    0
}
```
