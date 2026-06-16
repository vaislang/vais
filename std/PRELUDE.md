# Vais Prelude Status

This file records prelude-like APIs and whether they are currently verified by a
gate. "Verified" means protected by a value or compiler gate. "Specified" means
intended but not a public release claim yet.

## Output

| API | Status |
| --- | --- |
| `print(EXPR)` | Verified |
| `putchar(Int)` | Verified |

## Collections

| API | Status |
| --- | --- |
| `[1, 2, 3]` | Verified for Int lists |
| `List<Int>` | Verified |
| `List<T>` | Partial |
| `Map<K,V>` | Specified; first planned gate-backed slice is `Map<Int,Int>` |
| `v.len()` | Verified |
| `v.is_empty()` | Verified |
| `v.last()` | Verified |
| `v.pop()` | Verified |
| `v[i]` | Verified |
| `v.sum()` | Verified for Int lists |
| `v.push(x)` | Verified for Int lists and selected self-host shapes |
| `v.map(|x| ...)` | Specified |
| `v.filter(|x| BOOL)` | Specified |

Invalid list access traps at runtime. This includes negative or out-of-range
`v[i]`, `v.last()` on an empty list, and `v.pop()` on an empty list. `v.pop()`
checks before mutating the list length.

### Planned Map Slice

`Map<K,V>` is not verified yet. The first planned public slice is deliberately
small:

| API | Planned behavior |
| --- | --- |
| `let m: Map<Int,Int> = {}` | Construct an empty integer map |
| `m.insert(key, value)` | Insert or replace an integer value by integer key |
| `m.get(key, default)` | Return the value for `key`, or `default` when absent |
| `m.contains(key)` | Return whether `key` is present |
| `m.len()` | Return the number of present keys |

The first slice does not include generic key/value lowering, iteration,
deletion, `Option`, `Result`, hashing controls, or map literals with entries.

## Types And Conversion

| API | Status |
| --- | --- |
| `Int` | Verified |
| `Int8`..`Int128` | Specified |
| `UInt8`..`UInt128` | Specified |
| `F32`, `F64` | Specified |
| `Bool` | Verified for comparisons, boolean literals, and scalar helper signatures |
| `Str` | Verified for literals, scalar helper signatures, length, index, and equality |
| `Char` | Partial |
| `Int(x)` | Verified |
| `parse_uint(s)` | Verified for `Str`; parses a leading unsigned decimal run |
| `parse_int(s)` | Verified for `Str`; accepts a leading `-` before the decimal run |
| `F64(x)`, `UInt8(x)`, `Str(x)` | Specified |

## Strings

| API | Status |
| --- | --- |
| `"text"` and `` `text` `` | Verified |
| `s.len()` | Verified |
| `s[i]` | Verified |
| `a == b`, `a != b` | Verified for `Str` |
| byte-classification helpers such as `is_digit(c: Int) -> Bool` | Verified pattern |
| `parse_uint(s)`, `parse_int(s)` | Verified |

`parse_uint` and `parse_int` stop at the first non-decimal byte. Empty input or
input with no leading decimal digit returns `0`; `parse_int("-5")` returns `-5`.

## Control And Operators

| API | Status |
| --- | --- |
| `and`, `or`, `not` | Verified |
| `bitnot(x)` | Verified |
| `bitand(a,b)`, `bitor(a,b)`, `bitxor(a,b)` | Verified |
| `shl(x,n)`, `shr(x,n)` | Verified |
| `break`, `continue` | Specified |

## Current Work

New prelude entries should land with examples and value-correctness tests before
they are described as verified.
