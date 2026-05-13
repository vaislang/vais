# vais-regex

NFA-based regular expression engine for pattern matching in Vais.

## Features

- **Thompson NFA**: Fast pattern matching using non-deterministic finite automaton
- **Meta-characters**: `.` (any), `*` (0+), `+` (1+), `?` (0-1), `|` (or)
- **Anchors**: `^` (start), `$` (end)
- **Character classes**: `[abc]`, `[a-z]`, `[^abc]` (negated)
- **Escapes**: `\d` (digit), `\w` (word), `\s` (whitespace), `\.` (literal)

## API

```vais
# Compile pattern to NFA
F regex_compile(pattern: i64, pattern_len: i64) -> i64

# Full match (text must match entire pattern)
F regex_match(regex: i64, text: i64, text_len: i64) -> i64

# Search for first occurrence (returns position or -1)
F regex_search(regex: i64, text: i64, text_len: i64) -> i64

# Find all matches (returns {positions_ptr, count})
F regex_find_all(regex: i64, text: i64, text_len: i64) -> i64

# Free compiled regex
F regex_free(regex: i64) -> i64
```

## Examples

```vais
# Literal match
pattern := str_to_ptr("hello")
text := str_to_ptr("hello world")
regex := regex_compile(pattern, strlen(pattern))
pos := regex_search(regex, text, strlen(text))  # Returns 0
regex_free(regex)

# Digit matching
pattern := str_to_ptr("\\d+")
text := str_to_ptr("abc123def")
regex := regex_compile(pattern, strlen(pattern))
pos := regex_search(regex, text, strlen(text))  # Returns 3
regex_free(regex)

# Character class
pattern := str_to_ptr("[aeiou]")
text := str_to_ptr("hello")
regex := regex_compile(pattern, strlen(pattern))
result := regex_find_all(regex, text, strlen(text))
count := load_i64(result + 8)  # Returns 2 (e, o)
regex_free(regex)
```

## Supported Patterns

| Pattern | Description | Example |
|---------|-------------|---------|
| `abc` | Literal text | Matches "abc" |
| `.` | Any character (except newline) | `h.llo` matches "hello" |
| `*` | Zero or more | `ab*c` matches "ac", "abc", "abbc" |
| `+` | One or more | `ab+c` matches "abc", "abbc" |
| `?` | Zero or one | `colou?r` matches "color", "colour" |
| `\|` | Alternation | `cat\|dog` matches "cat" or "dog" |
| `^` | Start anchor | `^hello` matches start of text |
| `$` | End anchor | `world$` matches end of text |
| `[abc]` | Character class | Matches 'a', 'b', or 'c' |
| `[a-z]` | Range | Matches lowercase letters |
| `[^abc]` | Negated class | Matches any except 'a', 'b', 'c' |
| `\d` | Digit | Matches [0-9] |
| `\w` | Word character | Matches [a-zA-Z0-9_] |
| `\s` | Whitespace | Matches space, tab, newline |

## Implementation

The engine uses Thompson's construction to convert regex patterns into NFAs, then simulates
the NFA using epsilon-closure and state set propagation. The implementation is pure Vais
with no external dependencies (~800 lines).

## Testing

Run tests with:
```bash
vaisc tests/test_regex.vais -o test_regex && ./test_regex
```

13 tests cover literals, meta-characters, quantifiers, character classes, anchors, and search operations.

## License

MIT
