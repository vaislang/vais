# Vais Language Reference

This page describes the current gate-backed Vais source surface. Vais files use
the `.vais` extension and are compiled with `scripts/vaisc`.

## Status Model

Vais documentation uses these terms:

| Term | Meaning |
| --- | --- |
| Verified | Covered by `scripts/test-vaisc-front.sh`, `scripts/test-vaisc-parity.sh`, `scripts/test-vaisc-host.sh`, `scripts/test.sh`, or a self-host gate |
| Full engine | Compiled by the native public driver linked with the reusable self-host compiler core |
| Direct engine | Native promoted-slice LLVM path selected with `--engine direct` |
| Specified | Intended surface, not yet protected as a release claim |

Public examples should use verified syntax unless they explicitly document a
planned or experimental area.

## Program Shape

```vais
fn main() -> Int {
    return 42
}
```

- Entry point: `fn main() -> Int`.
- Source files must end in `.vais`.
- Statements do not require semicolons in ordinary source files.
- Line comments start with `#`.

## Modules, Packages, And Imports

The Phase 2 module model is specified in
[../design/MODULES.md](../design/MODULES.md). The first implemented slice
supports local dotted imports in the full engine:

```vais
import math.add

fn main() -> Int {
    return add(20, 22)
}
```

Without a package manifest, `import math.add` resolves to `math/add.vais` under
the entry file's directory. With a nearest `vais.toml`, imports resolve under
the manifest source root:

```toml
name = "demo"
version = "0.1.0"
source = "src"

[dependencies]
mathlib = "../mathlib"
```

For `source = "src"`, compile `src/main.vais`; `import math.add` resolves to
`src/math/add.vais`. A `[dependencies]` entry maps an import prefix to another
local package directory with its own `vais.toml`; for example,
`import mathlib.public` resolves to `public.vais` under the `mathlib` package's
source root when no local `mathlib/public.vais` file exists. Files loaded from
a dependency resolve plain imports under their own package source root.

Imported files are merged in deterministic import-path order before
compilation. Missing imports, duplicate top-level symbols, import cycles,
missing dependency manifests, unsafe dependency paths, and invalid manifests are
front-contract errors.

Explicit `module` and `package` declarations are reserved for later Phase 2
gates and are rejected for now. The direct engine remains single-file.

## Functions

```vais
fn add(a: Int, b: Int) -> Int {
    return a + b
}

fn main() -> Int {
    return add(20, 22)
}
```

Verified today:

- `Int` parameters and `Int` return values.
- Multiple helper functions.
- Recursive and mutually recursive `Int` functions.
- Generic marker syntax for simple `Int` helper cases, as tracked in the parity
  manifest.

The direct engine gate covers Int, Bool, Char, and Str helper calls in addition to the
full engine.

## Variables

```vais
let x = 5
let mut total = 0
total = total + x
let typed: Int = 42
```

- `let` binds an immutable value.
- `let mut` binds a mutable value.
- `let name: Int = value`, `let name: Bool = value`,
  `let name: Char = value`, and `let name: Str = value` are verified for
  scalar locals.
- Compound assignment such as `+=` is not Vais syntax.

## Types

Verified release surface:

| Type | Notes |
| --- | --- |
| `Int` | Primary scalar type |
| `Bool` | Produced by comparisons and boolean expressions; verified for local annotations, helper parameters, and helper returns |
| `Str` | String literals, local annotations, reassignment, helper parameters/returns, length, index, and equality |
| `Char` | Single-byte character literals, equality, annotations, helper parameters, and helper returns as Int-compatible scalar values |
| `List<Int>` | Empty/list literal, list/element assignment, `push`, `len`, `is_empty`, `last`, `pop`, index, `sum` |
| `List<Str>` | Full-engine local `push`, local index read, and argv-based `proc_run` host arguments |
| `List<Struct>` | Direct-engine `[]`, `list()`, list literal, list/element assignment, `push`, `len`, `is_empty`, `last`, `pop`, index, field read/write, parameter reference, return value |
| `Map<Int,Int>` | Local `{}`, assignment copy, parameter reference/mutation, `insert`, `get(key, default)`, `get_opt(key)`, `contains`, and `len` |
| `Map<Int,Bool>` | Local `{}`, assignment copy, parameter reference/mutation, `insert`, `get(key, default)`, `contains`, and `len` |
| `Map<Int,Char>` | Local `{}`, assignment copy, `insert`, `get(key, default)`, `contains`, and `len` |
| `Option<Int>` | `Some(Int)`/`None`, helper returns, struct/local storage, statement-form `match`, expression-match binding, and local-binding `?` propagation |
| `Result<Int,Int>` | `Ok(Int)`/`Err(Int)`, helper returns, statement-form `match`, expression-match binding, and local-binding `?` propagation |
| Simple `struct` | Literal construction, field access, and local field write |
| Small `enum` | Payload-free enum/match, small recursive `Int` payload enum/match, and single-field struct payload enum/match |

Specified or partial areas are tracked in [../../std/PRELUDE.md](../../std/PRELUDE.md)
and `tools/vaisc-parity.tsv`.

## Expressions And Operators

```vais
return (a + b) * 2
return n % 10
return a == b
return a < b and b < c
```

Verified operators:

- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Boolean words: `and`, `or`, `not`
- Bit helpers: `bitnot(x)`, `bitand(a, b)`, `bitor(a, b)`, `bitxor(a, b)`,
  `shl(x, n)`, `shr(x, n)`
- Conversion call: `Int(x)` in verified examples
- Parsing helpers: `parse_uint(s)` and `parse_int(s)` for `Str`

Vais does not use Rust spellings such as `&&`, `||`, `!x`, `x as Int`, or
`Enum::Variant`.

## Control Flow

```vais
if n <= 1 {
    return 1
} else if n == 2 {
    return 2
} else {
    return n * fact(n - 1)
}

let mut i = 0
let mut sum = 0
while i < n {
    sum = sum + i
    i = i + 1
}

for j in 0..n {
    sum = sum + j
}

for k in 0..=n {
    sum = sum + k
}

for k in 0..10 {
    if k == 3 { continue }
    if k == 6 { break }
    sum = sum + k
}
```

The direct engine gate covers `if`, `while`, range `for`, `break`/`continue`
inside loops, local `let`, assignment, helper
calls, `return`, simple inline `if { return ... }`, `Bool`/`Char`/`Str` scalar helper
signatures, `Str` literals/length/index/equality, `Char` literal equality and
annotations,
simple Int-field struct
locals, struct parameter/return helpers, and `List<Int>` local operations plus
parameter reference and return value ABI, local `Map<Int,Int>`,
`Map<Int,Bool>`, and `Map<Int,Char>` construction and lookup/update helpers,
`Map<Int,Int>` and `Map<Int,Bool>` parameter reference/mutation, plus `List<Struct>` construction with
`[]`, `list()`, list literals, list/element assignment, `push`, `len`, index,
field read/write, parameter reference, and return value ABI.

Verified today:

- `if`, `else if`, `else`
- `while`
- `for name in start..end` and `for name in start..=end`
- `break` and `continue` inside `while` and range `for` loops
- Early `return`

## Structs

```vais
struct Box {
    value: Int,
}

fn main() -> Int {
    let b = Box { value: 42 }
    return b.value
}
```

Struct helper values are also gate-backed:

```vais
fn make_box(value: Int) -> Box {
    return Box { value: value }
}

fn read_box(b: Box) -> Int {
    return b.value
}
```

Verified today:

- Simple struct declarations with `Int` fields.
- Struct literals.
- Field access.
- Field write for direct-engine local struct values.
- Struct parameters and return values in direct-engine helper functions.
- Selected struct/list combinations through self-host gates.

## Enums And Match

Payload-free enum:

```vais
enum Color { Red, Green, Blue }

fn number(c: Color) -> Int {
    match c {
        Color.Red => return 1,
        Color.Green => return 2,
        Color.Blue => return 3,
    }
}
```

Small payload enum:

```vais
enum Node { Lit(Int), Add(Node, Node), Mul(Node, Node) }

fn eval(n: Node) -> Int {
    match n {
        Lit(v) => return v,
        Add(a, b) => return eval(a) + eval(b),
        Mul(a, b) => return eval(a) * eval(b),
    }
}
```

Verified today:

- Payload-free enum tags with `Enum.Tag` spelling.
- Simple return-arm and expression-arm `match`.
- Int `match` with literal patterns and `_` catch-all, as covered by
  `examples/e55_match_wildcard.vais`.
- Payload-free enum `match` with `_` catch-all, as covered by
  `examples/e90_enum_wildcard.vais`.
- Small recursive `Int` payload enum/match lowering used by the parity corpus.
- Multi-field `Int` payload enum expression-arm lowering, as covered by
  `examples/e02_enum_payload.vais`.
- Payload-free enum values stored in simple struct fields and matched through
  field access, as covered by `examples/e24_struct_enum_field.vais`.
- Single-field struct payload enum values constructed from a struct literal and
  matched through payload field access, as covered by
  `examples/e64_enum_struct_payload.vais`.

Broader payload shapes, including multi-field struct payloads, are not a release
claim yet.

## Lists

```vais
fn main() -> Int {
    let xs: List<Int> = []
    xs.push(10)
    xs.push(20)
    xs.push(30)
    let tail = xs.pop()
    return xs.len() + xs[1] + xs.is_empty() + xs.last() + tail - 30
}
```

Function parameters can also receive a local `List<Int>`:

```vais
fn fill(out: List<Int>, n: Int) -> Int {
    out.push(n)
    out.push(n + 2)
    return out.len()
}

fn main() -> Int {
    let xs: List<Int> = []
    let count = fill(xs, 20)
    return count * 10 + xs[1]
}
```

Inline list values are valid return values and call arguments in the direct
engine:

```vais
fn make(n: Int) -> List<Int> {
    return [n, n + 2]
}

fn score(xs: List<Int>) -> Int {
    return xs.sum()
}

fn main() -> Int {
    return score([20, 22])
}
```

List-returning helper calls can be passed directly to `List<Int>` parameters in
ordinary statements and `if`/`else if` conditions:

```vais
fn make(n: Int) -> List<Int> {
    return [n, n + 1]
}

fn score(xs: List<Int>) -> Int {
    return xs.sum()
}

fn main() -> Int {
    if score([1, 1]) == 99 {
        return 1
    } else if score(make(20)) == 41 {
        return 42
    }
    return 0
}
```

Declared struct values can also be stored in local direct-engine lists:

```vais
struct Box {
    value: Int,
}

fn make(v: Int) -> List<Box> {
    return [Box { value: v }, Box { value: v + 1 }]
}

fn score(xs: List<Box>) -> Int {
    let first = xs[0]
    let tail = xs.pop()
    return first.value + xs.len() + tail.value
}

fn main() -> Int {
    let xs: List<Box> = []
    xs.push(Box { value: 20 })
    let ys: List<Box> = make(21)
    xs = [Box { value: 19 }]
    xs[0] = Box { value: 20 }
    xs[0].value = xs[0].value + 1
    return xs[0].value + ys[1].value + score([Box { value: 0 }, Box { value: 0 }]) - 1
}
```

Verified today:

- Empty `List<Int>` with an explicit type.
- Integer list literals such as `[10, 20, 30]`.
- Inline `[]`, `list()`, and integer list literals as `List<Int>` return values
  and call arguments in the direct engine.
- `List<Int>`-returning helper calls used directly as `List<Int>` call arguments
  in statement contexts plus `if`, `else if`, and `while` conditions.
- `xs.push(value)`.
- `xs.len()`.
- `xs.is_empty()`.
- `xs.last()`.
- `xs.pop()`.
- `xs[index]`.
- `xs.sum()`.
- Assigning `[]`, `list()`, list literals, local list values, and returned-list
  values to a matching direct-engine list local or list parameter.
- Assigning values to matching direct-engine list elements with `xs[index] = value`.
- Runtime trap behavior for out-of-range `xs[index]`, empty `xs.last()`, and
  empty `xs.pop()` in the full self-host path and native direct engine.
- Passing a local `List<Int>` to a `List<Int>` parameter.
- Returning `List<Int>` from helper functions.
- `List<Str>` locals with `push` and index reads in the full self-host path,
  including host process arguments and Vais-authored text tools.
- `List<Struct>` values with an explicit type, `[]`, `list()`, list literals,
  list/element assignment, `push`, `len`, `is_empty`, `last`, `pop`, index, field reads/writes, parameter
  references, return values, inline call arguments, and returned-list call
  arguments in the direct engine.

Invalid list access is a runtime trap. This covers negative indexes,
out-of-range indexes, `last()` on an empty list, and `pop()` on an empty list.
`pop()` checks before mutating the list length.

The direct engine gate covers `List<Int>` values created with `[]`, `list()`, or
small integer list literals, plus `push`, `len`/`len()`, `is_empty()`,
`last()`, `pop()`, index, `sum()`, and function calls where `List<Int>` parameters are
local list names or inline list values. It also covers `List<Int>`-returning helper calls passed
directly to `List<Int>` parameters in `return`, `let`, list-literal item, `push`, assignment
statements, `if`, `else if`, and `while` conditions. In the direct engine
native ABI, `List<Int>`
parameters are passed by reference, so `push` and `pop` on a local-list
parameter mutate the caller's local list, and assigning a new list to a list
parameter replaces the caller's local list. `push`, `pop`, or assignment on an
inline or returned-list temporary mutates only that temporary value. `List<Int>` return values are
returned by value. The
same parameter-reference and return-by-value ABI applies to `List<Struct>` for
declared structs, including inline list arguments and `List<Struct>`-returning
helper calls passed directly to `List<Struct>` parameters in statement contexts
plus `if`, `else if`, and `while` conditions. Direct list elements can be assigned with
`xs[index] = value`, and indexed `List<Struct>` element fields can be assigned
with `xs[index].field = value`; both work through list parameters.
`List<Struct>.last()` and `List<Struct>.pop()` can be bound to a struct local
before reading fields.
`sum()` on `List<Struct>` is not a direct-engine release claim.

Methods such as `map`, `filter`, and arbitrary user-defined methods are not
release-surface claims yet.

## Maps

`Map<Int,Int>`, `Map<Int,Bool>`, and `Map<Int,Char>` have verified
local-value slices in the full self-host compiler path and native direct engine.
All three support `{}`, assignment copy, `insert`, `get(key, default)`,
`contains`, and `len`.
`Map<Int,Int>` also supports `get_opt(key) -> Option<Int>`. `Map<Int,Int>` and
`Map<Int,Bool>` support function parameters by reference; a callee can mutate
the caller-visible Map with `insert`.

Verified example:

```vais
fn main() -> Int {
    let scores: Map<Int,Int> = {}
    scores.insert(4, 38)
    scores.insert(4, 40)
    scores.insert(9, 2)
    let copy: Map<Int,Int> = {}
    copy = scores
    scores.insert(4, 1)
    let found = copy.get(4, 0)
    let missing = scores.get(5, 0 - 1)
    let maybe = match scores.get_opt(9) { Some(v) => v, None => 0 }
    return found + missing + maybe + scores.contains(4) + scores.contains(5) + scores.len()
}
```

```vais
fn put(scores: Map<Int,Int>, key: Int, value: Int) -> Int {
    scores.insert(key, value)
    return scores.len()
}

fn main() -> Int {
    let scores: Map<Int,Int> = {}
    let n = put(scores, 4, 40)
    return scores.get(4, 0) + n + scores.contains(4)
}
```

```vais
fn mark(flags: Map<Int,Bool>, key: Int) -> Int {
    flags.insert(key, true)
    if flags.get(key, false) and flags.len() == 1 {
        return 40
    }
    return 0
}

fn main() -> Int {
    let flags: Map<Int,Bool> = {}
    let n = mark(flags, 4)
    if flags.get(4, false) and flags.contains(4) {
        return n + flags.len() + flags.contains(4)
    }
    return 0
}
```

```vais
fn main() -> Int {
    let flags: Map<Int,Bool> = {}
    let copy: Map<Int,Bool> = {}
    flags.insert(4, true)
    flags.insert(5, false)
    copy = flags
    if copy.get(4, false) and not copy.get(5, true) and copy.contains(4) {
        return 42
    }
    return 0
}
```

```vais
fn main() -> Int {
    let letters: Map<Int,Char> = {}
    let copy: Map<Int,Char> = {}
    letters.insert(4, 'A')
    letters.insert(5, 'B')
    copy = letters
    letters.insert(4, 'Z')
    if copy.get(4, 'Z') == 'A' and copy.get(5, 'Z') == 'B' and copy.contains(4) {
        return 42
    }
    return 0
}
```

Verified behavior:

- Local `Map<Int,Int>`, `Map<Int,Bool>`, and `Map<Int,Char>` values are
  supported. `Map<Int,Int>` and `Map<Int,Bool>` can also be passed as function
  parameters by reference.
- `{}` constructs an empty map when the local type is explicitly one of the
  verified concrete Map types.
- `target = source` copies one local Map into another local with the same
  concrete Map type; later mutation of either map does not alias the other.
- `insert(key, value)` inserts or replaces a value.
- `get(key, default)` returns `default` when the key is absent.
- `get_opt(key)` is verified only for `Map<Int,Int>` and returns `Some(value)`
  when the key is present and `None` when absent, as covered by
  `examples/e94_map_get_opt.vais`.
- `contains(key)` returns whether a key is present.
- `len()` returns the number of present keys.

Not included in the current Map slice: `Map<Int,Bool>.get_opt`,
`Map<Int,Char>.get_opt`,
generic key/value lowering, deletion, iteration, entry literals,
broader Map APIs that return `Option`, `Result`,
custom hashing, `Map<Int,Char>` parameters, or public ABI claims for Map
return values. Unverified `Map<Int,Char>` parameters, return values, and
non-local assignment sources are rejected by front diagnostics instead of being
treated as part of the release surface.
The future Map ABI and generic expansion contract is specified in
`docs/design/MAP_ABI.md`, but no broader Map behavior is verified until it has
compiler gates.

## Option And Result

`Option<Int>` has a first verified slice in the full compiler path:

```vais
fn find(x: Int) -> Option<Int> {
    if x > 0 { return Some(x * 2) }
    return None
}

fn main() -> Int {
    match find(21) {
        Some(v) => return v,
        None => return 0,
    }
}
```

Verified behavior:

- `Some(Int)` and `None` constructors.
- `Option<Int>` helper return values.
- `Option<Int>` stored in a simple struct field and matched through field
  access, as covered by `examples/e40_option_in_struct.vais`.
- Statement-form `match` arms that return from the current function.
- Expression-form `let x = match ...` bindings for `Option<Int>`, as covered by
  `examples/e23_option_flow.vais`.
- Local-binding `?` propagation for `Option<Int>` helper calls: `None` returns
  from the current `Option<Int>` function, and `Some` binds its payload, as
  covered by `examples/e93_option_question.vais`.

`Result<Int,Int>` has the same first statement-match shape:

```vais
fn div(a: Int, b: Int) -> Result<Int, Int> {
    if b == 0 { return Err(0) }
    return Ok(a / b)
}

fn main() -> Int {
    match div(21, 3) {
        Ok(v) => return v,
        Err(e) => return e,
    }
}
```

It also supports expression-form `let x = match ...` bindings for
`Result<Int,Int>`, as covered by `examples/e91_result_flow.vais`.
The `?` operator is verified for `Result<Int,Int>` helper calls assigned to a
local: an `Err` value returns from the current `Result<Int,Int>` function, and an
`Ok` value binds its payload, as covered by `examples/e39_error_propagate.vais`
and `examples/e92_result_question_success.vais`.

Not included yet: generic `Option<T>` or `Result<T,E>`, broader expression-form
`match` beyond the gate-backed `Option<Int>` and `Result<Int,Int>` binding
shapes, `?` beyond the gate-backed `Option<Int>` and `Result<Int,Int>`
local-binding shapes, broader Map APIs that return `Option`, direct-engine
Option/Result-specific claims beyond `Map<Int,Int>.get_opt` match bindings, and
nested option/result payloads. Unsupported generic `Option`/`Result` forms are
rejected by front diagnostics instead of being treated as verified language.

## Strings, Characters, And Output

```vais
fn main() -> Int {
    let x = 42
    print("the answer is {x}")
    putchar(33)
    return 0
}
```

Verified today:

- String literals with `"` or backtick delimiters.
- `Str` helper parameters, local values, reassignment, and helper return values,
  as covered by `examples/e89_str_type.vais`.
- `s.len()` and `s[i]` in the full self-host path and native direct engine.
- `a == b` and `a != b` for `Str` in the full self-host path, native direct
  engine, and parity examples.
- `str_concat(left, right)`, `str_slice(text, start, len)`, and
  `str_byte(value)` in the full engine host runtime.
- `Bool` byte-classification helpers such as `is_digit(c: Int) -> Bool`.
- Explicit `Bool` locals, helper parameters, helper returns, and unary `not`,
  as covered by `examples/e88_bool_type.vais`.
- Named integer parsing helpers `parse_uint(s)` and `parse_int(s)`, as covered
  by `examples/e83_parse_helpers.vais`.
- User-defined integer parsing over `Str`, as covered by
  `examples/e70_parse_uint.vais`.
- Identifier scanning over `Str`, as covered by
  `examples/e72_identifier_scan.vais`.
- Single-byte character literal equality plus `Char` locals, helper
  parameters, and helper returns as Int-compatible scalar values in the front,
  native direct, full, and parity gates, as covered by
  `examples/e85_char_type.vais`.
- `print("...{name}...")` interpolation for simple identifiers.
- `putchar(Int)`.

`parse_uint` parses a leading unsigned decimal run and stops at the first
non-decimal byte. `parse_int` accepts a leading `-` and then parses the same
decimal run. Empty input and input with no leading decimal digit return `0`.

## Host Files, Paths, And Processes

Phase 3 host APIs start with verified full-engine file, path, and argv-based
process intrinsics. The current and planned broader surface is:

```vais
fs_exists(path: Str) -> Bool
fs_read_text(path: Str) -> Str
fs_write_text(path: Str, text: Str) -> Int
fs_mkdirs(path: Str) -> Int
fs_remove(path: Str) -> Int
fs_cwd() -> Str
fs_temp_dir() -> Str
path_join(base: Str, child: Str) -> Str
path_basename(path: Str) -> Str
path_dirname(path: Str) -> Str
str_concat(left: Str, right: Str) -> Str
str_slice(text: Str, start: Int, len: Int) -> Str
str_byte(value: Int) -> Str
proc_run(argv: List<Str>) -> Int
proc_run_env(argv: List<Str>, env: List<Str>) -> Int
proc_capture_stdout(argv: List<Str>) -> Str
proc_capture_stderr(argv: List<Str>) -> Str
proc_capture_to(argv: List<Str>, stdout_path: Str, stderr_path: Str) -> Int
proc_capture(argv: List<Str>) -> ProcessResult
```

`fs_exists(path: Str) -> Bool`, `fs_read_text(path: Str) -> Str`,
`fs_write_text(path: Str, text: Str) -> Int`, and
`fs_mkdirs(path: Str) -> Int`, and `fs_remove(path: Str) -> Int`, plus `fs_cwd() -> Str`,
`fs_temp_dir() -> Str`, `path_join(base: Str, child: Str) -> Str`,
`path_basename(path: Str) -> Str`, `path_dirname(path: Str) -> Str`,
`str_concat(left: Str, right: Str) -> Str`, `str_slice(text: Str, start: Int,
len: Int) -> Str`, and `str_byte(value: Int) -> Str` are gate-backed by
`scripts/test-vaisc-host.sh` for `scripts/vaisc build` and `scripts/vaisc run`.
`proc_run(argv: List<Str>) -> Int` is covered by the same gate and returns the
child process exit code while inheriting stdio.
`proc_run_env(argv: List<Str>, env: List<Str>) -> Int` is also covered there
and applies child-only `KEY=value` environment overrides before exec.
`proc_capture_stdout(argv: List<Str>) -> Str` and
`proc_capture_stderr(argv: List<Str>) -> Str` are covered by the same gate and
return one captured child stream while inheriting the other.
`proc_capture_to(argv: List<Str>, stdout_path: Str, stderr_path: Str) -> Int`
is covered by the same gate and redirects stdout/stderr to explicit files while
returning the child exit code.
`fs_remove(path: Str) -> Int` removes an existing file path and also succeeds
when the path is already missing; it is covered by the same host gate.

```vais
fn main() -> Int {
    let argv: List<Str> = []
    argv.push("/bin/sh")
    argv.push("-c")
    argv.push("exit 7")
    return proc_run(argv)
}
```

`proc_capture` remains specified for a later in-memory result-struct gate.

The host API contract is maintained in
[../design/HOST_IO.md](../design/HOST_IO.md). Except for those verified file,
path, and process intrinsics, these names should not be treated as
release-surface verified until their gates are added.

## Closures

The verified closure slice is intentionally narrow:

```vais
fn adder(n: Int) -> fn(Int) -> Int {
    return |x| x + n
}

fn main() -> Int {
    let add3 = adder(3)
    return add3(4)
}
```

General closure literals and broader higher-order function patterns are not
release-surface claims yet.

## Diagnostics

`scripts/vais-check` and `scripts/vaisc` front diagnostics catch common
non-Vais spellings and print source coordinates and `help:`.

```bash
scripts/vais-check examples/c4.vais
```

The public checker is built from `tools/vais_check_cli.vais` and
`tools/vais_check_core.vais`. Release gates check its fixture issue counts,
coordinate/help output shape, clean-file behavior, and packaged command path.

Common corrections:

| Do not write | Write |
| --- | --- |
| `a && b` | `a and b` |
| `a || b` | `a or b` |
| `!x` | `not x` |
| `x as Int` | `Int(x)` |
| `use math.add` | `import math.add` |
| `pub fn f` | `fn f` |
| `Color::Red` | `Color.Red` |
| `Vec<T>` | `List<T>` |
| `String` | `Str` |
| `x += 1` | `x = x + 1` |

## Build And Test

```bash
scripts/vaisc doctor
scripts/vaisc emit-ir examples/c4.vais -o /tmp/c4.ll
scripts/vaisc build examples/c4.vais -o /tmp/c4
scripts/vaisc run examples/c4.vais

bash scripts/test-vaisc-native.sh
bash scripts/test-vaisc-install.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test-vaisc-host.sh
bash scripts/test.sh
```

The exact release subset is tracked in `tools/vaisc-parity.tsv`.
