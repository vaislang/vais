# Regex API Reference

> Simple regular expression matching using recursive descent

## Import

```vais
U std/regex
```

## Supported Syntax

| Pattern | Description |
|---------|-------------|
| `.` | Match any character |
| `*` | Zero or more of preceding |
| `+` | One or more of preceding |
| `?` | Zero or one of preceding |
| `^` | Start anchor |
| `$` | End anchor |
| `[abc]` | Character class |
| `[^abc]` | Negated character class |
| `[a-z]` | Character range |
| `\` | Escape next character |

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `regex_compile` | `F regex_compile(pattern: i64) -> i64` | Compile pattern to regex |
| `regex_match` | `F regex_match(regex: i64, text: i64) -> i64` | Match regex against text |
| `regex_test` | `F regex_test(pattern: i64, text: i64) -> i64` | Compile, match, and free |
| `regex_free` | `F regex_free(regex: i64) -> i64` | Free compiled regex |

## Usage

```vais
U std/regex

F main() -> i64 {
    # Quick test (compile + match + free)
    result := regex_test("^he.*o$", "hello")  # 1 (match)

    # Compiled pattern for repeated use
    re := regex_compile("[0-9]+")
    m1 := regex_match(re, "abc123")   # 1
    m2 := regex_match(re, "no nums")  # 0
    regex_free(re)
    0
}
```
