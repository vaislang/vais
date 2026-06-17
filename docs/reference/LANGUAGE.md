# Vais Language Reference

This page describes the current gate-backed Vais source surface. Vais files use
the `.vais` extension and are compiled with `scripts/vaisc`.

## Status Model

Vais documentation uses these terms:

| Term | Meaning |
| --- | --- |
| Verified | Covered by `scripts/test-vaisc-front.sh`, `scripts/test-vaisc-parity.sh`, `scripts/test.sh`, or a self-host gate |
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

For an entry file `main.vais`, `import math.add` resolves to `math/add.vais`
under the entry file's directory. Imported files are merged in deterministic
import-path order before compilation. Missing imports, duplicate top-level
symbols, and import cycles are front-contract errors.

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

The direct engine gate covers Int, Bool, and Str helper calls in addition to the
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
- `let name: Int = value`, `let name: Bool = value`, and
  `let name: Str = value` are verified for scalar locals.
- Compound assignment such as `+=` is not Vais syntax.

## Types

Verified release surface:

| Type | Notes |
| --- | --- |
| `Int` | Primary scalar type |
| `Bool` | Produced by comparisons, boolean expressions, and helper signatures |
| `Str` | String literals, helper signatures, length, index, and equality |
| `Char` | Single-byte character literals in verified examples |
| `List<Int>` | Empty/list literal, list/element assignment, `push`, `len`, `is_empty`, `last`, `pop`, index, `sum` |
| `List<Struct>` | Direct-engine `[]`, `list()`, list literal, list/element assignment, `push`, `len`, `is_empty`, `last`, `pop`, index, field read/write, parameter reference, return value |
| `Map<Int,Int>` | Local `{}`, `insert`, `get(key, default)`, `contains`, and `len` |
| Simple `struct` | Literal construction, field access, and local field write |
| Small `enum` | Payload-free enum/match and small recursive `Int` payload enum/match |

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
```

The direct engine gate covers `if`, `while`, local `let`, assignment, helper
calls, `return`, simple inline `if { return ... }`, `Bool`/`Str` scalar helper
signatures, `Str` literals/length/index/equality, simple Int-field struct
locals, struct parameter/return helpers, and `List<Int>` local operations plus
parameter reference and return value ABI, local `Map<Int,Int>` construction and
lookup/update helpers, plus `List<Struct>` construction with
`[]`, `list()`, list literals, list/element assignment, `push`, `len`, index,
field read/write, parameter reference, and return value ABI.

Verified today:

- `if`, `else if`, `else`
- `while`
- Early `return`

`for`, `break`, and `continue` are not release-surface claims yet.

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
- Simple return-arm `match`.
- Small recursive `Int` payload enum/match lowering used by the parity corpus.

Broader payload shapes are not a release claim yet.

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

`Map<Int,Int>` has a verified local-value slice in the full self-host compiler
path and native direct engine.

Verified example:

```vais
fn main() -> Int {
    let scores: Map<Int,Int> = {}
    scores.insert(4, 38)
    scores.insert(4, 40)
    scores.insert(9, 2)
    let found = scores.get(4, 0)
    let missing = scores.get(5, 0 - 1)
    return found + missing + scores.contains(4) + scores.contains(5) + scores.len()
}
```

Verified behavior:

- Only local `Map<Int,Int>` values are supported.
- `{}` constructs an empty map when the local type is explicitly
  `Map<Int,Int>`.
- `insert(key, value)` inserts or replaces a value.
- `get(key, default)` returns `default` when the key is absent. This avoids
  publishing `Option` before `Option` itself is gate-backed.
- `contains(key)` returns whether a key is present.
- `len()` returns the number of present keys.

Not included in the current Map slice: generic key/value lowering, assignment,
deletion, iteration, entry literals, `Option`, `Result`, custom hashing, or
public ABI claims for Map parameters and return values.

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
- `Str` helper parameters, local values, and helper return values.
- `s.len()` and `s[i]` in the full self-host path and native direct engine.
- `a == b` and `a != b` for `Str` in the native direct engine and parity
  examples.
- `Bool` byte-classification helpers such as `is_digit(c: Int) -> Bool`.
- Named integer parsing helpers `parse_uint(s)` and `parse_int(s)`, as covered
  by `examples/e83_parse_helpers.vais`.
- User-defined integer parsing over `Str`, as covered by
  `examples/e70_parse_uint.vais`.
- Identifier scanning over `Str`, as covered by
  `examples/e72_identifier_scan.vais`.
- Single-byte character literal equality in parity examples.
- `print("...{name}...")` interpolation for simple identifiers.
- `putchar(Int)`.

`parse_uint` parses a leading unsigned decimal run and stops at the first
non-decimal byte. `parse_int` accepts a leading `-` and then parses the same
decimal run. Empty input and input with no leading decimal digit return `0`.

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

`tools/vais-check.py` and `scripts/vaisc` front diagnostics catch common
non-Vais spellings and print source coordinates, `help:`, and when available a
concrete `fix:`.

```bash
python3 tools/vais-check.py examples/c4.vais
```

Common corrections:

| Do not write | Write |
| --- | --- |
| `a && b` | `a and b` |
| `a || b` | `a or b` |
| `!x` | `not x` |
| `x as Int` | `Int(x)` |
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
bash scripts/test.sh
```

The exact release subset is tracked in `tools/vaisc-parity.tsv`.
