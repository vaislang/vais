# Regex API Reference

> Simple regular expression matching using recursive descent

## Import

```vais
U std/regex
```

## Overview

The regex module provides a lightweight regular expression engine using recursive descent matching. It supports common regex features including character classes, quantifiers, and anchors. Patterns are compiled into an internal node-based representation for efficient matching.

## Supported Syntax

| Pattern | Description |
|---------|-------------|
| `.` | Match any character |
| `*` | Zero or more of preceding |
| `+` | One or more of preceding |
| `?` | Zero or one of preceding |
| `^` | Start anchor (must be at beginning) |
| `$` | End anchor (must be at end) |
| `[abc]` | Character class (match a, b, or c) |
| `[^abc]` | Negated character class (match any except a, b, c) |
| `[a-z]` | Character range (match lowercase letters) |
| `\` | Escape next character |

## Functions

### regex_compile

```vais
F regex_compile(pattern: i64) -> i64
```

Compile a regular expression pattern into an internal representation.

**Parameters:**
- `pattern`: Pointer to null-terminated pattern string

**Returns:** Pointer to compiled regex (internal node structure)

**Example:**
```vais
re := regex_compile("a+b*")
```

---

### regex_match

```vais
F regex_match(regex: i64, text: i64) -> i64
```

Match a compiled regex against text. Returns 1 if the pattern matches anywhere in the text.

**Parameters:**
- `regex`: Compiled regex from `regex_compile`
- `text`: Pointer to null-terminated text string

**Returns:** `1` if match found, `0` otherwise

**Example:**
```vais
re := regex_compile("[0-9]+")
result := regex_match(re, "abc123def")  # 1
```

---

### regex_test

```vais
F regex_test(pattern: i64, text: i64) -> i64
```

Convenience function that compiles pattern, matches text, and frees the regex. Use this for one-time matches.

**Parameters:**
- `pattern`: Pointer to null-terminated pattern string
- `text`: Pointer to null-terminated text string

**Returns:** `1` if match found, `0` otherwise

**Example:**
```vais
I regex_test("^hello", "hello world") {
    puts("Match found!")
}
```

---

### regex_free

```vais
F regex_free(regex: i64) -> i64
```

Free a compiled regex and all associated memory.

**Parameters:**
- `regex`: Compiled regex to free

**Returns:** `0`

**Example:**
```vais
re := regex_compile("test")
# ... use regex ...
regex_free(re)
```

## Usage Examples

### Basic Matching

```vais
U std/regex

F main() -> i64 {
    # Quick test (compile + match + free)
    result := regex_test("^he.*o$", "hello")  # 1 (match)

    # No match
    result2 := regex_test("^he.*o$", "hi there")  # 0

    0
}
```

### Reusing Compiled Patterns

```vais
U std/regex

F main() -> i64 {
    # Compile once, use multiple times
    re := regex_compile("[0-9]+")

    m1 := regex_match(re, "abc123")    # 1 (contains digits)
    m2 := regex_match(re, "no nums")   # 0 (no digits)
    m3 := regex_match(re, "42")        # 1 (all digits)

    regex_free(re)
    0
}
```

### Character Classes

```vais
U std/regex

F main() -> i64 {
    # Match vowels
    I regex_test("[aeiou]", "hello") {
        puts("Contains vowel")
    }

    # Match non-digits
    I regex_test("[^0-9]+", "abc") {
        puts("No digits found")
    }

    # Match ranges
    I regex_test("[a-z][A-Z]", "aB") {
        puts("Lowercase followed by uppercase")
    }

    0
}
```

### Quantifiers

```vais
U std/regex

F main() -> i64 {
    # * = zero or more
    regex_test("ab*c", "ac")      # 1 (zero b's)
    regex_test("ab*c", "abc")     # 1 (one b)
    regex_test("ab*c", "abbbbc")  # 1 (many b's)

    # + = one or more
    regex_test("ab+c", "ac")      # 0 (needs at least one b)
    regex_test("ab+c", "abc")     # 1

    # ? = zero or one
    regex_test("ab?c", "ac")      # 1 (zero b's)
    regex_test("ab?c", "abc")     # 1 (one b)
    regex_test("ab?c", "abbc")    # 0 (too many b's)

    0
}
```

### Anchors

```vais
U std/regex

F main() -> i64 {
    # ^ = start of string
    regex_test("^hello", "hello world")  # 1
    regex_test("^hello", "say hello")    # 0

    # $ = end of string
    regex_test("world$", "hello world")  # 1
    regex_test("world$", "world peace")  # 0

    # Both anchors = exact match
    regex_test("^test$", "test")         # 1
    regex_test("^test$", "testing")      # 0

    0
}
```
