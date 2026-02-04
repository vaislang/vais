# String API Reference

> Heap-allocated dynamic string with length and capacity tracking

## Import

```vais
U std/string
```

## Overview

The `String` type provides a dynamically-sized, heap-allocated string that automatically grows as needed. Strings are null-terminated and use ASCII character encoding. The module also depends on `std/option` for safe error handling.

## Dependencies

```vais
U std/option
```

## Struct

```vais
S String {
    data: i64,   # Pointer to char array (i8*)
    len: i64,    # Current length (excluding null terminator)
    cap: i64     # Allocated capacity
}
```

## Methods

### with_capacity

```vais
F with_capacity(capacity: i64) -> String
```

Create an empty string with the specified initial capacity.

**Parameters:**
- `capacity`: Initial capacity to allocate

**Returns:** New empty `String`

**Example:**
```vais
s := String.with_capacity(100)
```

---

### len

```vais
F len(&self) -> i64
```

Get the current length of the string (excluding null terminator).

**Returns:** The string length

---

### capacity

```vais
F capacity(&self) -> i64
```

Get the allocated capacity.

**Returns:** The current capacity

---

### is_empty

```vais
F is_empty(&self) -> i64
```

Check if the string is empty.

**Returns:** `1` if empty, `0` otherwise

---

### char_at

```vais
F char_at(&self, index: i64) -> i64
```

Get the ASCII character at the specified index. Returns 0 for out-of-bounds access.

**Parameters:**
- `index`: The index to access

**Returns:** ASCII value of character, or `0` if out of bounds

**Example:**
```vais
s := str_from("hello")
c := s.char_at(0)  # 104 ('h')
```

---

### char_at_opt

```vais
F char_at_opt(&self, index: i64) -> Option<i64>
```

Safe character access using Option type.

**Parameters:**
- `index`: The index to access

**Returns:** `Some(char)` if index is valid, `None` otherwise

**Example:**
```vais
s := str_from("test")
M s.char_at_opt(0) {
    Some(c) => { puts_char(c) }
    None => { puts("Out of bounds") }
}
```

---

### push_char

```vais
F push_char(&self, c: i64) -> i64
```

Append a character to the end of the string. Automatically grows capacity if needed.

**Parameters:**
- `c`: ASCII value of character to append

**Returns:** New length

**Example:**
```vais
s := str_from("hello")
s.push_char(33)  # Append '!'
```

---

### grow

```vais
F grow(&self) -> i64
```

Grow the string's capacity (doubles it, or sets to 16 if smaller).

**Returns:** The new capacity

---

### clear

```vais
F clear(&self) -> i64
```

Clear the string contents (sets length to 0).

**Returns:** `0`

---

### print

```vais
F print(&self) -> i64
```

Print the string to stdout.

**Returns:** Result from `puts_ptr`

---

### drop

```vais
F drop(&self) -> i64
```

Free the string's memory.

**Returns:** `0`

## Free Functions

### str_from

```vais
F str_from(s: i64) -> String
```

Create a `String` from a C string literal (null-terminated).

**Parameters:**
- `s`: Pointer to null-terminated string

**Returns:** New `String` containing a copy of the data

**Example:**
```vais
s := str_from("Hello, world!")
```

---

### str_concat

```vais
F str_concat(a: String, b: String) -> String
```

Concatenate two strings into a new string.

**Parameters:**
- `a`: First string
- `b`: Second string

**Returns:** New `String` containing concatenated result

**Example:**
```vais
a := str_from("Hello")
b := str_from(" world")
c := str_concat(a, b)  # "Hello world"
```

---

### str_substring

```vais
F str_substring(s: String, start: i64, end: i64) -> String
```

Extract a substring from start index (inclusive) to end index (exclusive).

**Parameters:**
- `s`: Source string
- `start`: Start index (clamped to 0)
- `end`: End index (clamped to length)

**Returns:** New `String` containing substring

**Example:**
```vais
s := str_from("hello world")
sub := str_substring(s, 0, 5)  # "hello"
```

---

### str_contains_char

```vais
F str_contains_char(s: String, c: i64) -> i64
```

Check if the string contains the specified character.

**Parameters:**
- `s`: String to search
- `c`: ASCII value of character

**Returns:** `1` if found, `0` otherwise

**Example:**
```vais
s := str_from("hello")
I str_contains_char(s, 101) {  # 'e'
    puts("Contains 'e'")
}
```

---

### str_eq

```vais
F str_eq(a: String, b: String) -> i64
```

Compare two strings for equality.

**Parameters:**
- `a`: First string
- `b`: Second string

**Returns:** `1` if equal, `0` otherwise

**Example:**
```vais
a := str_from("test")
b := str_from("test")
I str_eq(a, b) {
    puts("Strings are equal")
}
```

## Usage Examples

### Basic String Operations

```vais
U std/string

F main() -> i64 {
    # Create from literal
    s := str_from("Hello")

    # Append characters
    s.push_char(32)   # space
    s.push_char(119)  # 'w'
    s.push_char(111)  # 'o'
    s.push_char(114)  # 'r'
    s.push_char(108)  # 'l'
    s.push_char(100)  # 'd'
    s.push_char(33)   # '!'

    # Print result
    s.print()  # "Hello world!"

    # Clean up
    s.drop()
    0
}
```

### String Concatenation

```vais
U std/string

F main() -> i64 {
    first := str_from("Hello")
    second := str_from(" ")
    third := str_from("world!")

    temp := str_concat(first, second)
    result := str_concat(temp, third)

    result.print()  # "Hello world!"

    # Free all strings
    first.drop()
    second.drop()
    third.drop()
    temp.drop()
    result.drop()
    0
}
```

### Substring Extraction

```vais
U std/string

F main() -> i64 {
    text := str_from("The quick brown fox")

    # Extract "quick"
    word := str_substring(text, 4, 9)
    word.print()

    text.drop()
    word.drop()
    0
}
```

### Character Access

```vais
U std/string

F main() -> i64 {
    s := str_from("hello")

    # Iterate through characters
    i := 0
    L {
        I i >= s.len() {
            B 0
        }
        c := s.char_at(i)
        puts_char(c)
        i = i + 1
    }

    s.drop()
    0
}
```

### String Comparison

```vais
U std/string

F main() -> i64 {
    a := str_from("test")
    b := str_from("test")
    c := str_from("different")

    I str_eq(a, b) {
        puts("a equals b")  # This prints
    }

    I str_eq(a, c) {
        puts("a equals c")  # This doesn't print
    }

    a.drop()
    b.drop()
    c.drop()
    0
}
```
