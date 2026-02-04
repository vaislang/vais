# String API Reference

> Heap-allocated dynamic string with length and capacity tracking

## Import

```vais
U std/string
```

## Struct

```vais
S String {
    data: i64,   # Pointer to char array
    len: i64,    # Current length
    cap: i64     # Allocated capacity
}
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `with_capacity` | `F with_capacity(capacity: i64) -> String` | Create empty string with capacity |
| `len` | `F len(&self) -> i64` | Get current length |
| `capacity` | `F capacity(&self) -> i64` | Get allocated capacity |
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `char_at` | `F char_at(&self, index: i64) -> i64` | Get ASCII char at index |
| `char_at_opt` | `F char_at_opt(&self, index: i64) -> Option<i64>` | Safe char access with Option |
| `push_char` | `F push_char(&self, c: i64) -> i64` | Append a character |
| `clear` | `F clear(&self) -> i64` | Clear the string |
| `print` | `F print(&self) -> i64` | Print to stdout |
| `drop` | `F drop(&self) -> i64` | Free memory |

## Free Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `str_from` | `F str_from(s: i64) -> String` | Create String from C string literal |
| `str_concat` | `F str_concat(a: String, b: String) -> String` | Concatenate two strings |
| `str_substring` | `F str_substring(s: String, start: i64, end: i64) -> String` | Extract substring |
| `str_contains_char` | `F str_contains_char(s: String, c: i64) -> i64` | Check if contains char |
| `str_eq` | `F str_eq(a: String, b: String) -> i64` | Compare two strings |

## Usage

```vais
U std/string

F main() -> i64 {
    s := str_from("Hello")
    s.push_char(33)  # '!'
    s.print()        # prints "Hello!"
    s.drop()
    0
}
```
