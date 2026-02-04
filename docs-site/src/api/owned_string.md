# OwnedString API Reference

> Owned heap-allocated string with length tracking and lifecycle management

## Import

```vais
U std/owned_string
```

## Struct

```vais
S OwnedString { data: i64, len: i64, cap: i64 }
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `with_capacity` | `F with_capacity(capacity: i64) -> OwnedString` | Create empty with capacity |
| `from_str` | `F from_str(s: i64) -> OwnedString` | Copy from C string |
| `len` | `F len(&self) -> i64` | Get length |
| `capacity` | `F capacity(&self) -> i64` | Get capacity |
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `as_ptr` | `F as_ptr(&self) -> i64` | Get raw pointer |
| `char_at` | `F char_at(&self, index: i64) -> i64` | Get character at index (ASCII) |
| `push_char` | `F push_char(&self, c: i64) -> i64` | Append character |
| `push_str` | `F push_str(&self, s: i64) -> i64` | Append C string |
| `ensure_capacity` | `F ensure_capacity(&self, needed: i64) -> i64` | Ensure at least needed bytes |
| `grow` | `F grow(&self) -> i64` | Double capacity |
| `eq` | `F eq(&self, other: OwnedString) -> i64` | Compare with another OwnedString |
| `eq_str` | `F eq_str(&self, s: i64) -> i64` | Compare with C string |
| `clone` | `F clone(&self) -> OwnedString` | Deep copy |
| `clear` | `F clear(&self) -> i64` | Clear contents |
| `print` | `F print(&self) -> i64` | Print the string |
| `drop` | `F drop(&self) -> i64` | Free memory |

## Free Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `owned_str_memcmp` | `F owned_str_memcmp(a: i64, b: i64, len: i64) -> i64` | Compare bytes (1 if equal, 0 if not) |
| `owned_str_concat` | `F owned_str_concat(a: OwnedString, b: OwnedString) -> OwnedString` | Concatenate two strings |
| `owned_str` | `F owned_str(s: i64) -> OwnedString` | Convenience wrapper for from_str |

## Overview

Similar to `String` but designed for use in database engines and internal APIs where string ownership tracking is important. Automatically manages its buffer and supports safe conversion to/from raw `i8*` pointers.

## Usage

```vais
U std/owned_string

F main() -> i64 {
    s := OwnedString.from_str("Hello")
    s.push_str(", World!")

    # Comparison
    s2 := OwnedString.from_str("Hello, World!")
    I s.eq(s2) {
        s.print()
    }

    # Clone
    s3 := s.clone()

    s.drop()
    s2.drop()
    s3.drop()
    0
}
```
