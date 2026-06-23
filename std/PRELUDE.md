# Vais Prelude Status

This file records prelude-like APIs and whether they are currently verified by a
gate. "Verified" means protected by a value or compiler gate. "Specified" means
intended but not a public release claim yet.

## Output

| API | Status |
| --- | --- |
| `print(EXPR)` | Verified |
| `putchar(Int)` | Verified |

## Host Files, Paths, And Processes

The Phase 3 host API is documented in
[../docs/design/HOST_IO.md](../docs/design/HOST_IO.md). `fs_exists`,
`fs_read_text`, `fs_write_text`, and `fs_mkdirs` are verified for the full
engine, the first path and string helpers are verified, and `proc_argc() ->
Int`, `proc_arg(index: Int) -> Str`, `proc_run(argv: List<Str>) -> Int`, and
`proc_run_env(argv: List<Str>, env: List<Str>) -> Int`,
`proc_capture_stdout(argv: List<Str>) -> Str`,
`proc_capture_stderr(argv: List<Str>) -> Str`, and
`proc_capture_to(argv: List<Str>, stdout_path: Str, stderr_path: Str) -> Int`
are verified through the same host gate. `proc_argc` and `proc_arg` are
verified for `vaisc run -- ...` and for binaries produced by `vaisc build`;
full in-memory status/stdout/stderr capture is specified for a later gate.

| API | Status |
| --- | --- |
| `fs_exists(path: Str) -> Bool` | Verified |
| `fs_read_text(path: Str) -> Str` | Verified |
| `fs_write_text(path: Str, text: Str) -> Int` | Verified |
| `fs_mkdirs(path: Str) -> Int` | Verified |
| `fs_remove(path: Str) -> Int` | Verified |
| `fs_cwd() -> Str` | Verified |
| `fs_temp_dir() -> Str` | Verified |
| `path_join(base: Str, child: Str) -> Str` | Verified |
| `path_basename(path: Str) -> Str` | Verified |
| `path_dirname(path: Str) -> Str` | Verified |
| `str_concat(left: Str, right: Str) -> Str` | Verified |
| `str_slice(text: Str, start: Int, len: Int) -> Str` | Verified |
| `str_byte(value: Int) -> Str` | Verified |
| `proc_argc() -> Int` | Verified |
| `proc_arg(index: Int) -> Str` | Verified |
| `proc_run(argv: List<Str>) -> Int` | Verified |
| `proc_run_env(argv: List<Str>, env: List<Str>) -> Int` | Verified |
| `proc_capture_stdout(argv: List<Str>) -> Str` | Verified |
| `proc_capture_stderr(argv: List<Str>) -> Str` | Verified |
| `proc_capture_to(argv: List<Str>, stdout_path: Str, stderr_path: Str) -> Int` | Verified |
| `proc_capture(argv: List<Str>) -> ProcessResult` | Specified |

## Collections

| API | Status |
| --- | --- |
| `[1, 2, 3]` | Verified for Int lists |
| `List<Int>` | Verified |
| `List<Str>` | Partial; verified for local `push`, local index read, and host process arguments |
| `List<T>` | Partial |
| `Map<Int,Int>` | Verified for local values, local assignment copy, parameter reference/mutation, return-value local initialization, `remove`, `clear`, and `get_opt` |
| `Map<Int,Bool>` | Verified for local values, local assignment copy, parameter reference/mutation, return-value local initialization, `remove`, `clear`, and `get_opt` |
| `Map<Int,Char>` | Verified for local values, local assignment copy, parameter reference/mutation, return-value local initialization, `remove`, `clear`, and `get_opt` |
| `Map<Str,Int>` | Verified for local values, local assignment copy, parameter reference/mutation, `insert`, `remove`, `clear`, `get`, `get_opt`, `contains`, and `len`; return values are not verified |
| `Map<K,V>` | Design-specified beyond the verified concrete local Map slices; not verified |
| `Option<Int>` | Verified for `Some(Int)`, `None`, helper returns, struct/local storage, statement `match`, expression-match binding, and local-binding `?` propagation |
| `Option<T>` | Specified beyond the `Option<Int>` slice |
| `Result<Int,Int>` | Verified for `Ok(Int)`, `Err(Int)`, helper returns, statement `match`, expression-match binding, and local-binding `?` propagation |
| `Result<T,E>` | Specified beyond the `Result<Int,Int>` slice |
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

### Map Slice

The verified Map surface is deliberately small:

| API | Verified behavior |
| --- | --- |
| `let m: Map<Int,Int> = {}` / `let m: Map<Int,Bool> = {}` / `let m: Map<Int,Char> = {}` / `let m: Map<Str,Int> = {}` | Construct an empty local map |
| `fn f(m: Map<Int,Int>) -> Int` / `fn f(m: Map<Int,Bool>) -> Int` / `fn f(m: Map<Int,Char>) -> Int` / `fn f(m: Map<Str,Int>) -> Int` | Pass a Map by reference so the callee can read or mutate the caller-visible map |
| `fn make() -> Map<Int,Int>` / `let m: Map<Int,Int> = make()` / `fn make() -> Map<Int,Bool>` / `let m: Map<Int,Bool> = make()` / `fn make() -> Map<Int,Char>` / `let m: Map<Int,Char> = make()` | Return a verified concrete Map into caller-owned local storage |
| `target = source` | Copy one local map into another local map with the same concrete type |
| `m.insert(key, value)` | Insert or replace a value by key; verified keys are `Int` for `Map<Int,V>` and `Str` for `Map<Str,Int>` |
| `m.remove(key)` | Remove a key if present; missing keys are ignored |
| `m.clear()` | Remove all keys and allow the map to be reused |
| `m.get(key, default)` | Return the value for `key`, or `default` when absent |
| `m.get_opt(key)` | Return `Some(value)` for a present key or `None` when absent on `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, or `Map<Str,Int>` |
| `m.contains(key)` | Return whether `key` is present |
| `m.len()` | Return the number of present keys |

This slice is currently available through the full self-host compiler path and
`scripts/vaisc --engine direct`.
The slice does not include generic key/value lowering, `Map<Str,Int>` return
values, generic Map return values, iteration,
`Result`, hashing controls, or map literals with entries.
Unverified generic Map function parameters, unverified return values, and non-local assignment sources are
rejected by front diagnostics.
Future Map ABI and generic expansion rules are design-specified in
`docs/design/MAP_ABI.md`; they are not verified prelude APIs until compiler
gates cover them.

## Types And Conversion

| API | Status |
| --- | --- |
| `Int` | Verified |
| `Int8`..`Int128` | Specified |
| `UInt8`..`UInt128` | Specified |
| `F32`, `F64` | Specified |
| `Bool` | Verified for comparisons, boolean literals, local annotations, helper parameters, helper returns, and unary `not` |
| `Str` | Verified for literals, local annotations, scalar helper parameters and returns, length, index, equality, reassignment, and host-backed construction helpers |
| `Char` | Verified for single-byte literals, equality, annotations, helper parameters, and helper returns as Int-compatible scalar values |
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
| `a == b`, `a != b` | Verified for `Str` in the full self-host path and native direct engine |
| `str_concat(left, right)` | Verified |
| `str_slice(text, start, len)` | Verified; invalid ranges trap |
| `str_byte(value)` | Verified; values outside `0..255` trap |
| byte-classification helpers such as `is_digit(c: Int) -> Bool` | Verified pattern |
| single-byte `Char` literals such as `'A'` | Verified for equality, `Char` locals, helper parameters, and helper returns |
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
