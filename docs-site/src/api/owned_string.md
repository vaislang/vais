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
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `push_char` | `F push_char(&self, c: i64) -> i64` | Append character |
| `push_str` | `F push_str(&self, s: i64) -> i64` | Append C string |
| `as_ptr` | `F as_ptr(&self) -> i64` | Get raw pointer |
| `clear` | `F clear(&self) -> i64` | Clear contents |
| `drop` | `F drop(&self) -> i64` | Free memory |

## Overview

Similar to `String` but designed for use in database engines and internal APIs where string ownership tracking is important. Automatically manages its buffer and supports safe conversion to/from raw `i8*` pointers.

## Usage

```vais
U std/owned_string

F main() -> i64 {
    s := OwnedString::from_str("Hello")
    s.push_str(", World!")
    ptr := s.as_ptr()
    s.drop()
    0
}
```
