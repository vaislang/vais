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

Representative gate-backed examples are `examples/module_basic/main.vais`,
`examples/package_basic/src/main.vais`, and
`examples/dependency_basic/app/src/main.vais`.

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
- Generic marker syntax for simple `Int` helper cases and generic identity
  helpers applied to struct literals, as tracked in the parity manifest.

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
| `List<Int>` | Empty/list literal, typed non-empty local literal, inline call argument, borrowed helper parameter, list/element assignment, `push`, `len`, `is_empty`, `last`, `pop`, index, `sum`, and local `List<List<Int>>` literal double-index reads |
| `List<Str>` | Full-engine local `push`, local index read, and argv-based `proc_run` host arguments |
| `List<Struct>` | Direct-engine `[]`, `list()`, list literal, list/element assignment, `push`, `len`, `is_empty`, `last`, `pop`, index, field read/write, parameter reference, return value |
| `Map<Int,Int>` | Local `{}`, local/parameter/return-call assignment copy, parameter reference/mutation, return-value local initialization, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, and `len` |
| `Map<Int,Bool>` | Local `{}`, local/parameter/return-call assignment copy, parameter reference/mutation, return-value local initialization, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, and `len` |
| `Map<Int,Char>` | Local `{}`, local/parameter/return-call assignment copy, parameter reference/mutation, return-value local initialization, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, and `len` |
| `Map<Str,Int>` | Local `{}`, local/parameter/return-call assignment copy, parameter reference/mutation, return-value local initialization, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, and `len` |
| `Map<Str,Bool>` | Local `{}`, local/parameter/return-call assignment copy, parameter reference/mutation, return-value local initialization, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, and `len` |
| `Map<Str,Char>` | Local `{}`, local/parameter/return-call assignment copy, parameter reference/mutation, return-value local initialization, `insert`, `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, and `len` |
| `Option<Int>` | `Some(Int)`/`None`, helper returns, struct/local storage, statement-form `match`, single-line and multiline expression-match binding, local-binding `?` propagation, and nested match through an enum payload |
| `Result<Int,Int>` | `Ok(Int)`/`Err(Int)`, helper returns, statement-form `match`, expression-match binding, and local-binding `?` propagation |
| `(Int, Int)` tuple | Function return and local destructuring slice lowered through generated structs |
| Simple `struct` | Literal construction, field access, local field write, single-field nested struct read/write, helper parameters, helper returns, helper-return assignment, generic marker syntax used with `Int` values, generic identity helpers applied to struct literals, simple `impl` method return chains, and simple `trait` plus `impl Trait for Struct` method calls |
| Small `enum` | Payload-free enum/match, payload enum wildcard match, small recursive `Int` payload enum/match, single-field struct payload enum/match, and single `Option<Int>` payload enum with a nested Option match arm |

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
- Bool expression locals and Bool-returning helper predicates are covered by
  `examples/e10_bool_logic.vais` and `examples/e36_bool_predicate.vais`.
- Bit helpers: `bitnot(x)`, `bitand(a, b)`, `bitor(a, b)`, `bitxor(a, b)`,
  `shl(x, n)`, `shr(x, n)`
- Conversion calls: `Int(x)` and `Str(x)` for Int-compatible scalar values in
  verified examples
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

The direct engine gate covers `if`, `while`, range `for`, named `List<Int>`
for-each loops, `break`/`continue` inside loops, local `let`, assignment, helper
calls, `return`, simple inline `if { return ... }`, `Bool`/`Char`/`Str` scalar helper
signatures, `Str` literals/length/index/equality, `Char` literal equality and
annotations,
simple Int-field struct
locals, struct parameter/return helpers, and `List<Int>` local operations plus
parameter reference and return value ABI, local `Map<Int,Int>`,
`Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, and `Map<Str,Char>` construction and lookup/update helpers,
`Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, and `Map<Str,Bool>` return-value local initialization,
`Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, and `Map<Str,Char>` parameter reference/mutation, plus `List<Struct>` construction with
`[]`, `list()`, list literals, list/element assignment, `push`, `len`, index,
field read/write, parameter reference, and return value ABI.

Verified today:

- `if`, `else if`, `else`
- `while`
- `for name in start..end` and `for name in start..=end`
- `for name in xs` over gate-backed integer collections
- `break` and `continue` inside `while` and range `for` loops
- Early `return`

Representative gate-backed examples include `examples/e06_for_sum.vais`,
`examples/e12_exclusive_range.vais`, `examples/e13_nested_for.vais`,
`examples/e25_for_filter_sum.vais`, `examples/e57_break.vais`,
`examples/e58_continue.vais`, and `examples/e65_loop_break_acc.vais`.

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

Struct helper parameters and return values are also gate-backed:

```vais
fn make_box(value: Int) -> Box {
    return Box { value: value }
}

fn read_box(b: Box) -> Int {
    return b.value
}
```

Generic marker syntax on simple structs is gate-backed when used with `Int`
values:

```vais
struct Box<T> { val: T }

fn main() -> Int {
    let b = Box { val: 7 }
    return b.val
}
```

Generic identity helpers are also gate-backed when they pass through a struct
literal directly:

```vais
struct Pair { a: Int, b: Int }
fn first_field(p: Pair) -> Int { return p.a }
fn apply<T>(x: T) -> T { return x }

fn main() -> Int {
    let p = apply(Pair { a: 7, b: 2 })
    return first_field(p)
}
```

Verified today:

- Simple struct declarations with `Int` fields.
- Struct literals.
- Field access.
- Field write for direct-engine local struct values.
- Single-field nested struct literals, reads, and writes in the full self-host
  path, as covered by `examples/e01_nested_struct.vais` and
  `examples/e32_nested_field_mut.vais`.
- Twenty-field flat struct literals and field reads on the full self-host path,
  as covered by `examples/e51_index_ast.vais`.
- Struct parameters and return values in full self-host and direct-engine helper
  functions, including assignment from struct-returning calls.
- `pub` modifiers on struct declarations, struct fields, and helper functions
  are accepted as source-level visibility markers. The current compiler treats
  them as metadata rather than a separate ABI/export mechanism, as covered by
  `examples/d5run.vais`.
- Generic marker syntax on simple structs used with `Int` values and generic
  identity helpers applied directly to struct literals, as covered by
  `examples/e63_generic_struct_def.vais` and
  `examples/e46_generic_struct.vais`.
- Simple `impl Struct { fn method(self, ...) ... }` methods used in a return
  expression chain, as covered by `examples/e09_struct_method.vais`.
- Simple `trait` declarations paired with `impl Trait for Struct` methods, when
  the method call is used in a return expression, as covered by
  `examples/e78_trait_impl_for.vais`.
- Selected struct/list combinations through self-host gates.

Representative struct helper examples include `examples/e17_struct_return.vais`,
`examples/e28_struct_rebuild.vais`, `examples/e37_struct_area.vais`,
`examples/e41_recursion_struct.vais`, `examples/e54_inventory.vais`, and
`examples/e62_struct_multi_return.vais`. Promoted struct-method examples are
`examples/e09_struct_method.vais` and `examples/e78_trait_impl_for.vais`.

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
- Payload enum `match` with `_` catch-all, as covered by
  `examples/e120_enum_payload_wildcard.vais`.
- Small recursive `Int` payload enum/match lowering used by the parity corpus.
- Multi-field `Int` payload enum expression-arm lowering, as covered by
  `examples/e02_enum_payload.vais`.
- Payload-free enum values stored in simple struct fields and matched through
  field access, as covered by `examples/e24_struct_enum_field.vais`.
- Single-field struct payload enum values constructed from a struct literal and
  matched through payload field access, as covered by
  `examples/e64_enum_struct_payload.vais`.
- A single `Option<Int>` enum payload matched through a nested Option match arm,
  as covered by `examples/e79_nested_match.vais`.

Broader payload shapes, including multi-field struct payloads and arbitrary
nested enum payloads, are not a release claim yet.

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

- Empty `List<Int>` with an explicit type, typed non-empty local `List<Int>`
  literals such as `let xs: List<Int> = [10, 20, 30]`, and inline `List<Int>`
  literals passed to `List<Int>` parameters, as covered by
  `examples/e82_list_literal_direct_arg.vais` and `examples/d4b.vais`.
- Borrowed `&List<Int>` helper parameters in the full self-host path, as
  covered by `examples/e15_list_recursion.vais` and
  `examples/e68_binary_search.vais`.
- Integer list literals such as `[10, 20, 30]`.
- Computed index reads over integer list literals, as covered by
  `examples/e61_array_index_expr.vais`, and simple array-backed state machines,
  as covered by `examples/e52_state_machine.vais`.
- `for x in xs { ... }` over integer collections, as covered by
  `examples/e25_for_filter_sum.vais`. The full self-host path covers fixed
  integer arrays, typed non-empty local `List<Int>` literals, inline
  `List<Int>` literal call arguments, and scalar `List<Int>` locals/parameters;
  the native direct engine covers named local or parameter `List<Int>` values.
- Inline `[]`, `list()`, and integer list literals as `List<Int>` return values
  in the direct engine, and inline integer list literals as `List<Int>` call
  arguments in both the full self-host path and direct engine.
- Local `List<List<Int>>` literals with constant row selection and `Int` column
  indexing, as covered by `examples/e77_nested_list.vais`.
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
`last()`, `pop()`, index, `sum()`, non-capturing `map(|x| expr)`,
`filter(|x| predicate).sum()`, named `for x in xs` iteration, and function calls where `List<Int>` parameters are
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

List method forms outside the verified non-capturing `List<Int>.map(...)` and
`List<Int>.filter(...).sum()` slices, and method forms outside the verified
simple struct `impl`/`impl Trait for Struct` return-expression slices, are not
release-surface claims yet.
Broader `List<List<T>>` mutation, parameter passing, returns, and dynamic row
selection are not release-surface claims yet.

## Maps

`Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
`Map<Str,Bool>`, and `Map<Str,Char>` have verified slices in the full self-host
compiler path and native direct engine. All six support `{}`, assignment copy,
`insert`,
`get(key, default)`, `remove`, `clear`, `get_opt(key)`, `contains`, and `len`.
`Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
`Map<Str,Bool>`, and `Map<Str,Char>` support function return values that
initialize an explicitly annotated local. They also support function parameters
by reference; a callee can mutate the caller-visible Map with `insert`,
`remove`, `clear`, or same-type assignment. A Map parameter can be the source or
target of `target = source`, and assignment copies contents instead of creating
an alias.

Verified example:

```vais
fn main() -> Int {
    let scores: Map<Int,Int> = {}
    scores.insert(4, 38)
    scores.insert(4, 40)
    scores.insert(9, 2)
    scores.remove(5)
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
fn make_scores() -> Map<Int,Int> {
    let scores: Map<Int,Int> = {}
    scores.insert(4, 40)
    scores.insert(9, 2)
    return scores
}

fn main() -> Int {
    let scores: Map<Int,Int> = make_scores()
    if scores.contains(4) and scores.contains(9) {
        return scores.get(4, 0) + scores.get(9, 0)
    }
    return 0
}
```

```vais
fn make_names() -> Map<Str,Int> {
    let scores: Map<Str,Int> = {}
    scores.insert("red", 40)
    scores.insert("blue", 2)
    return scores
}

fn main() -> Int {
    let scores: Map<Str,Int> = make_names()
    let copy: Map<Str,Int> = {}
    copy = scores
    scores.remove("blue")
    if copy.contains("blue") and scores.contains("red") and not scores.contains("blue") {
        return scores.get("red", 0) + copy.get("blue", 0)
    }
    return 0
}
```

```vais
fn make_flags() -> Map<Int,Bool> {
    let flags: Map<Int,Bool> = {}
    flags.insert(4, true)
    flags.insert(9, false)
    return flags
}

fn main() -> Int {
    let flags: Map<Int,Bool> = make_flags()
    if flags.get(4, false) and not flags.get(9, true) and flags.contains(4) and flags.len() == 2 {
        return 42
    }
    return 0
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
    let flags: Map<Int,Bool> = {}
    flags.insert(4, true)
    let yes_value = match flags.get_opt(4) { Some(v) => v, None => 0 }

    let letters: Map<Int,Char> = {}
    letters.insert(4, 'A')
    let letter_value = match letters.get_opt(4) { Some(v) => v, None => 58 }

    return yes_value * 20 + letter_value - 43
}
```

```vais
fn main() -> Int {
    let scores: Map<Int,Int> = {}
    scores.insert(4, 1)
    scores.clear()
    scores.insert(9, 40)
    if not scores.contains(4) and scores.get(9, 0) == 40 and scores.len() == 1 {
        return 42
    }
    return 0
}
```

```vais
fn make_letters() -> Map<Int,Char> {
    let letters: Map<Int,Char> = {}
    letters.insert(4, 'A')
    letters.insert(9, 'B')
    return letters
}

fn main() -> Int {
    let letters: Map<Int,Char> = make_letters()
    if letters.get(4, 'Z') == 'A' and letters.get(9, 'Z') == 'B' and letters.contains(4) and letters.len() == 2 {
        return 42
    }
    return 0
}
```

```vais
fn stamp(letters: Map<Int,Char>, key: Int) -> Int {
    letters.insert(key, 'A')
    if letters.get(key, 'Z') == 'A' and letters.len() == 1 {
        return 40
    }
    return 0
}

fn main() -> Int {
    let letters: Map<Int,Char> = {}
    let n = stamp(letters, 4)
    if letters.get(4, 'Z') == 'A' and letters.contains(4) {
        return n + letters.len() + letters.contains(4)
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

```vais
fn main() -> Int {
    let flags: Map<Str,Bool> = {}
    let red: Str = "red"
    flags.insert("red", true)
    flags.insert("blue", false)
    flags.insert(red, true)

    let copy: Map<Str,Bool> = {}
    copy = flags

    flags.remove("blue")
    flags.clear()
    flags.insert("green", true)

    let yes_value = match copy.get_opt("red") { Some(v) => v, None => 0 }
    let no_value = match copy.get_opt("blue") { Some(v) => v, None => 1 }
    let missing = match flags.get_opt("red") { Some(v) => 0, None => 5 }

    if copy.contains("blue") and not flags.contains("red") and flags.get("green", false) and flags.len() == 1 {
        return yes_value * 20 + (1 - no_value) * 10 + missing + flags.contains("green") + copy.len() * 3
    }
    return 0
}
```

```vais
fn mark(flags: Map<Str,Bool>, key: Str, value: Bool) -> Int {
    flags.insert(key, value)
    flags.insert("blue", false)
    flags.remove("blue")
    flags.insert("green", true)
    return flags.len()
}

fn main() -> Int {
    let flags: Map<Str,Bool> = {}
    let red: Str = "red"
    let n = mark(flags, red, true)
    let red_value = match flags.get_opt(red) { Some(v) => v, None => 0 }
    if flags.contains("green") and not flags.contains("blue") and flags.get("green", false) {
        return n * 20 + red_value + flags.contains("green")
    }
    return 0
}
```

```vais
fn make_flags() -> Map<Str,Bool> {
    let flags: Map<Str,Bool> = {}
    flags.insert("red", true)
    flags.insert("blue", false)
    return flags
}

fn main() -> Int {
    let flags: Map<Str,Bool> = make_flags()
    let copy: Map<Str,Bool> = {}
    copy = flags
    flags.remove("blue")
    let red_value = match flags.get_opt("red") { Some(v) => v, None => 0 }
    if copy.contains("blue") and flags.contains("red") and not flags.contains("blue") {
        return red_value * 40 + copy.len()
    }
    return 0
}
```

```vais
fn put_name(scores: Map<Str,Int>, key: Str, value: Int) -> Int {
    scores.insert(key, value)
    scores.insert("blue", 2)
    scores.remove("blue")
    return scores.len()
}

fn main() -> Int {
    let scores: Map<Str,Int> = {}
    let red: Str = "red"
    let n = put_name(scores, red, 40)
    if scores.contains("red") and not scores.contains("blue") {
        return scores.get("red", 0) + n + scores.len()
    }
    return 0
}
```

```vais
fn main() -> Int {
    let scores: Map<Str,Int> = {}
    let red: Str = "red"
    scores.insert("red", 10)
    scores.insert("blue", 20)
    scores.insert(red, 40)

    let copy: Map<Str,Int> = {}
    copy = scores

    scores.remove("blue")
    scores.clear()
    scores.insert("green", 1)

    let maybe = match copy.get_opt("red") { Some(v) => v, None => 0 }
    if copy.contains("blue") and not scores.contains("red") and scores.get("green", 0) == 1 and scores.len() == 1 {
        return maybe + copy.get("blue", 0) - 18
    }
    return 0
}
```

```vais
fn main() -> Int {
    let letters: Map<Str,Char> = {}
    let red: Str = "red"
    letters.insert("red", 'A')
    letters.insert("blue", 'B')
    letters.insert(red, 'A')

    let copy: Map<Str,Char> = {}
    copy = letters

    letters.remove("blue")
    letters.clear()
    letters.insert("green", 'C')

    let red_value = match copy.get_opt("red") { Some(v) => v, None => 0 }
    let blue_value = match copy.get_opt("blue") { Some(v) => v, None => 0 }
    let missing = match letters.get_opt("red") { Some(v) => 0, None => 5 }

    if copy.contains("blue") and not letters.contains("red") and letters.get("green", 'Z') == 'C' and letters.len() == 1 {
        return red_value - 65 + blue_value - 66 + missing + letters.contains("green") + copy.len() * 18
    }
    return 0
}
```

```vais
fn stamp(letters: Map<Str,Char>, key: Str, value: Char) -> Int {
    letters.insert(key, value)
    letters.insert("green", 'C')
    return letters.len()
}

fn main() -> Int {
    let letters: Map<Str,Char> = {}
    let red: Str = "red"
    let n = stamp(letters, red, 'A')
    if letters.get(red, 'Z') == 'A' and letters.get("green", 'Z') == 'C' {
        return n * 20 + letters.contains("green") + letters.len() - 1
    }
    return 0
}
```

```vais
fn make_letters() -> Map<Str,Char> {
    let letters: Map<Str,Char> = {}
    letters.insert("red", 'A')
    letters.insert("blue", 'B')
    return letters
}

fn main() -> Int {
    let letters: Map<Str,Char> = make_letters()
    let copy: Map<Str,Char> = {}
    copy = letters
    letters.remove("blue")
    if copy.contains("blue") and letters.contains("red") and not letters.contains("blue") {
        return letters.get("red", 'Z') - 65 + copy.get("blue", 'Z') - 66 + copy.len() * 20 + letters.len() + copy.contains("blue")
    }
    return 0
}
```

Verified behavior:

- Local `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
  `Map<Str,Bool>`, and `Map<Str,Char>` values are supported.
- `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
  `Map<Str,Bool>`, and `Map<Str,Char>` can also be passed as function
  parameters by reference.
- `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`,
  `Map<Str,Bool>`, and `Map<Str,Char>` return values can initialize an explicitly
  annotated local, copying returned contents into caller-owned storage.
- `{}` constructs an empty map when the local type is explicitly one of the
  verified concrete Map types.
- `target = source` copies a same-type Map local, same-type Map parameter, or
  same-type Map-returning call into the target Map; later mutation of either
  map does not alias the other. Parameter-source/target copies are covered by
  `examples/e116_map_param_assignment.vais` and
  `examples/e119_map_param_target_assignment.vais`, and Map-returning call
  assignment copies are covered by `examples/e117_map_return_assignment.vais`
  and `examples/e118_map_return_assignment_args.vais`.
- `insert(key, value)` inserts or replaces a value.
- `remove(key)` removes a present key if it exists; removing a missing key is a
  no-op.
- `clear()` removes all keys and allows the map to be reused, as covered by
  `examples/e106_map_clear.vais`.
- `get(key, default)` returns `default` when the key is absent.
- `get_opt(key)` returns `Some(value)` when the key is present and `None` when
  absent for `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`,
  `Map<Str,Int>`, `Map<Str,Bool>`, and `Map<Str,Char>`, as covered by `examples/e94_map_get_opt.vais`,
  `examples/e105_map_scalar_get_opt.vais`, `examples/e107_map_str_int.vais`,
  `examples/e108_map_str_int_param.vais`, and
  `examples/e109_map_str_int_return.vais`, `examples/e110_map_str_bool.vais`,
  `examples/e111_map_str_bool_param.vais`, and
  `examples/e112_map_str_bool_return.vais`, plus
  `examples/e113_map_str_char.vais`, `examples/e114_map_str_char_param.vais`,
  `examples/e115_map_str_char_return.vais`,
  `examples/e116_map_param_assignment.vais`, and
  `examples/e117_map_return_assignment.vais`, plus
  `examples/e118_map_return_assignment_args.vais`.
- `contains(key)` returns whether a key is present.
- `len()` returns the number of present keys.

Not included in the current Map slice: broader generic key/value lowering,
iteration, entry literals, broader Map APIs that return `Option`, `Result`,
custom hashing, broader `Map<Str,V>` return values, or public ABI claims for
generic Map return values.
Unverified generic Map parameters, unverified return values, and non-promoted
assignment sources are rejected by front diagnostics instead of being treated as
part of the release surface.
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
- Expression-form `let x = match ...` bindings for `Option<Int>`, including
  multiline arm blocks, as covered by `examples/e23_option_flow.vais` and
  `examples/d2.vais`.
- Local-binding `?` propagation for `Option<Int>` helper calls: `None` returns
  from the current `Option<Int>` function, and `Some` binds its payload, as
  covered by `examples/e93_option_question.vais`.
- Direct `Option<Int>` helper-return matching from `main`, as covered by
  `examples/e08_option_chain.vais`.
- `Option<Int>` carried as a single enum payload and matched in a nested
  statement arm, as covered by `examples/e79_nested_match.vais`.

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
Helper-chain `Result<Int,Int>` propagation is covered by `examples/d3run.vais`.

Not included yet: generic `Option<T>`, `Result<T,E>`, or `Map<K,V>` beyond the
verified concrete Map shapes, broader expression-form `match` beyond the
gate-backed `Option<Int>` and `Result<Int,Int>` binding shapes, `?` beyond the
gate-backed `Option<Int>` and `Result<Int,Int>` local-binding shapes, broader
Map APIs that return `Option`, direct-engine Option/Result-specific claims
beyond concrete `Map<Int,V>.get_opt` match bindings, and nested option/result
payloads. Unsupported generic `Option`, `Result`, and `Map` forms are rejected
by front diagnostics instead of being treated as verified language.

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
- Literal string length, as covered by `examples/e44_string_len.vais`.
- Word-count state-machine scans over `Str`, as covered by
  `examples/e53_word_count.vais`.
- Two-pointer `Str` scans with computed byte indexes from both ends, as covered
  by `examples/e69_palindrome_string.vais`.
- User-defined integer parsing over `Str`, as covered by
  `examples/e70_parse_uint.vais`.
- Substring search patterns over `Str` with computed byte indexes, as covered by
  `examples/e71_string_index_of.vais`.
- Identifier scanning over `Str`, as covered by
  `examples/e72_identifier_scan.vais`.
- Single-byte character literal equality plus `Char` locals, helper
  parameters, and helper returns as Int-compatible scalar values in the front,
  native direct, full, and parity gates, as covered by
  `examples/e85_char_type.vais`.
- `print("...{name}...")` interpolation for simple identifiers in the full
  self-host path and native direct engine.
- `putchar(Int)` in the full self-host path and native direct engine.

`parse_uint` parses a leading unsigned decimal run and stops at the first
non-decimal byte. `parse_int` accepts a leading `-` and then parses the same
decimal run. Empty input and input with no leading decimal digit return `0`.
`Str(x)` converts an Int-compatible scalar value into a decimal `Str`, as
covered by `examples/e73_int_to_string.vais`.

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

## Tuples

The verified tuple slice covers function return and immediate local
destructuring for `Int` tuples:

```vais
fn pair() -> (Int, Int) {
    return (3, 4)
}

fn main() -> Int {
    let (a, b) = pair()
    return a + b
}
```

This slice is covered by `examples/e59_tuple.vais`. The public driver lowers it
to generated struct storage before the self-host core receives the source.
Nested tuples, tuple parameters, tuple mutation, and non-`Int` tuple fields are
not release-surface claims yet.

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

Returned single-`Int` closures can also be passed to a single-closure
higher-order helper that applies the closure to an `Int`, as covered by
`examples/e81_closure_return_apply.vais`.

Non-capturing inline closure literals can be passed directly to the same
single-closure `Int` higher-order helper shape, as covered by
`examples/e49_closure_arg.vais`.

Local closures can capture one `Int` local and be called inside the same
function, as covered by `examples/c5.vais`.

Captured inline closure arguments, multiple captured values, multiple closure
parameters, escaping local closure values, and broader higher-order function
patterns are not release-surface claims yet.

## Diagnostics

`scripts/vais-check` and `scripts/vaisc` front diagnostics catch common
non-Vais spellings and print source coordinates and `help:`.

```bash
scripts/vais-check examples/c4.vais
```

The public checker is built from `tools/vais_check_cli.vais` and
`tools/vais_check_core.vais`. Release gates check its fixture issue counts,
coordinate/help output shape, clean-file behavior, invalid static import path
diagnostics, invalid `main` signature diagnostics, missing helper return-type
diagnostics, unsupported generic `Option<T>`, `Result<T,E>`, and non-verified
`Map<K,V>` diagnostics, and packaged command path.

Common corrections:

| Do not write | Write |
| --- | --- |
| `a && b` | `a and b` |
| `a || b` | `a or b` |
| `!x` | `not x` |
| `x as Int` | `Int(x)` |
| `use math.add` | `import math.add` |
| `import math::add` | `import math.add` |
| `Color::Red` | `Color.Red` |
| `Vec<T>` | `List<T>` |
| `String` | `Str` |
| `fn main(argc: Int) -> Int { ... }` | `fn main() -> Int { ... }` |
| `fn add(a: Int, b: Int) { ... }` | `fn add(a: Int, b: Int) -> Int { ... }` |
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
