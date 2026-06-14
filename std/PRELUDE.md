# Vais Prelude

This file records the prelude surface that Vais source can use without imports.

## Output

| API | Status |
| --- | --- |
| `print(EXPR)` | verified |

## Collections

| API | Status |
| --- | --- |
| `[1, 2, 3]` | verified |
| `List<T>` | verified |
| `Map<K,V>` | specified |
| `v.len()` | verified |
| `v[i]` | verified |
| `v.sum()` | verified |
| `v.push(x)` | verified |
| `v.map(|x| ...)` | specified |
| `v.filter(|x| BOOL)` | specified |

## Types And Conversion

| API | Status |
| --- | --- |
| `Int`, `Int8`..`Int128` | verified |
| `UInt8`..`UInt128` | specified |
| `F32`, `F64` | specified |
| `Bool`, `Str`, `Char` | specified |
| `Int(x)`, `F64(x)`, `UInt8(x)` | verified |
| `Str(x)` | specified |

## Control And Operators

| API | Status |
| --- | --- |
| `and`, `or`, `not` | verified |
| `bitnot(x)` | verified |
| `bitand(a,b)`, `bitor(a,b)`, `bitxor(a,b)` | verified |
| `shl(x,n)`, `shr(x,n)` | verified |
| `break`, `continue` | specified |

## Current Work

The compiler must own these APIs directly as the native/full runtime grows. New prelude entries should land with examples and value-correctness tests.
