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
| `Map<K,V>` | Specified |
| `v.len()` | Verified |
| `v.is_empty()` | Verified |
| `v[i]` | Verified |
| `v.sum()` | Verified for Int lists |
| `v.push(x)` | Verified for Int lists and selected self-host shapes |
| `v.map(|x| ...)` | Specified |
| `v.filter(|x| BOOL)` | Specified |

## Types And Conversion

| API | Status |
| --- | --- |
| `Int` | Verified |
| `Int8`..`Int128` | Specified |
| `UInt8`..`UInt128` | Specified |
| `F32`, `F64` | Specified |
| `Bool`, `Str`, `Char` | Partial |
| `Int(x)` | Verified |
| `F64(x)`, `UInt8(x)`, `Str(x)` | Specified |

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
