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

The direct engine gate covers Int helper calls in addition to the full engine.

## Variables

```vais
let x = 5
let mut total = 0
total = total + x
let typed: Int = 42
```

- `let` binds an immutable value.
- `let mut` binds a mutable value.
- `let name: Int = value` is verified for `Int`.
- Compound assignment such as `+=` is not Vais syntax.

## Types

Verified release surface:

| Type | Notes |
| --- | --- |
| `Int` | Primary scalar type |
| `Bool` | Produced by comparisons and boolean expressions |
| `Str` | String literals and selected string operations |
| `Char` | Single-byte character literals in verified examples |
| `List<Int>` | Empty/list literal, `push`, `len`, index, `sum` |
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
calls, `return`, simple Int-field struct locals, struct parameter/return
helpers, and `List<Int>` local operations plus parameter reference and return
value ABI.
Strings and the self-host compiler tier remain full-engine territory.

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
    return xs.len() + xs[1]
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
ordinary statements:

```vais
fn make(n: Int) -> List<Int> {
    return [n, n + 1]
}

fn score(xs: List<Int>) -> Int {
    return xs.sum()
}

fn main() -> Int {
    return score(make(20))
}
```

Verified today:

- Empty `List<Int>` with an explicit type.
- Integer list literals such as `[10, 20, 30]`.
- Inline `[]`, `list()`, and integer list literals as `List<Int>` return values
  and call arguments in the direct engine.
- `List<Int>`-returning helper calls used directly as `List<Int>` call arguments
  in statement contexts.
- `xs.push(value)`.
- `xs.len()`.
- `xs[index]`.
- `xs.sum()`.
- Passing a local `List<Int>` to a `List<Int>` parameter.
- Returning `List<Int>` from helper functions.

The direct engine gate covers `List<Int>` values created with `[]`, `list()`, or
small integer list literals, plus `push`, `len`/`len()`, index, `sum()`, and
function calls where `List<Int>` parameters are local list names or inline list
values. It also covers `List<Int>`-returning helper calls passed directly to
`List<Int>` parameters in `return`, `let`, list-literal item, `push`, and
assignment statements. In the direct engine native ABI, `List<Int>` parameters
are passed by reference, so `push` on a local-list parameter mutates the caller's
local list; `push` on an inline or returned-list temporary mutates only that
temporary value. `List<Int>` return values are returned by value. Returned-list
argument hoisting inside `while` conditions and `List<Struct>` are not
direct-engine release claims yet.

Methods such as `map`, `filter`, and arbitrary user-defined methods are not
release-surface claims yet.

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

- String literals.
- String equality in parity examples.
- String length/indexing in self-host gates.
- Single-byte character literal equality in parity examples.
- `print("...{name}...")` interpolation for simple identifiers.
- `putchar(Int)`.

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
