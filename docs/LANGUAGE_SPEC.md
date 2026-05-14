# Vais Language Specification

Version: 0.1.0 source baseline

This document is the compiler-facing language contract for the current Vais
source baseline. It is not a product-complete v1.0 declaration.

## Public Baseline

The current public baseline is evidence-scoped:

- Core compiler and promoted runtime gates are certified by named tests.
- DB/server/web behavior is claimed only where a named gate covers it.
- Browser-only playground compile/execute is not claimed as complete.
- Older phase reports and archived blog posts are historical evidence, not the
  active language contract.

## Lexical Structure

Comments start with `#` and continue to the end of the line.

```vais
# A comment
fn add(a: i64, b: i64) -> i64 = a + b
```

Identifiers start with a letter or underscore, followed by letters, digits, or
underscores.

```text
[a-zA-Z_][a-zA-Z0-9_]*
```

## Keywords

Canonical declaration and module keywords:

| Keyword | Meaning |
| --- | --- |
| `fn` | Function declaration |
| `struct` | Struct declaration |
| `enum` | Enum declaration |
| `trait` | Trait declaration |
| `impl` | Inherent or trait implementation |
| `type` | Type alias |
| `use` | Import |
| `pub` | Public visibility |

Current compact control and system forms:

| Keyword | Meaning |
| --- | --- |
| `I` | If expression |
| `else` | Else branch |
| `match` | Pattern match |
| `L` | General loop |
| `LF` | Range or foreach loop |
| `LW` | While-style loop |
| `B` | Break |
| `C` | Continue |
| `D` | Defer |
| `A` | Async function marker |
| `Y` | Await/yield shorthand |
| `N` | Extern declaration |
| `G` | Global declaration |
| `O` | Union declaration |

Early compact declaration/import aliases are not canonical in public examples.
Some may remain parser compatibility surfaces, but new documentation and tests
should use the canonical forms above.

## Primitive Types

```vais
i8 i16 i32 i64 i128
u8 u16 u32 u64 u128
f32 f64
bool
str
()
```

## Compound Types

```vais
*T              # Typed raw pointer
&T              # Shared reference
&mut T          # Mutable reference
&[T]            # Shared slice
&mut [T]        # Mutable slice
[T]             # Array-like value surface
(A, B, C)       # Tuple
Option<T>
Result<T, E>
Vec<T>
```

Low-level memory APIs such as `malloc`, `free`, `load_i64`, and `store_i64`
expose raw addresses as `i64`. Typed pointers (`*T`) are a separate type
surface and do not implicitly unify with raw integer addresses.

## Type Conversion Rules

Vais keeps structural types strict, while numeric primitives may adapt to the
expected numeric context. These rules are part of the language type contract,
not a backend fallback.

Implicit conversions:

- Integer numeric unification across integer widths and signedness.
- Integer/float numeric promotion when the expected type is numeric.
- Float literal inference between `f32` and `f64`.

Prohibited implicit conversions:

- `bool <-> i64`: use `flag as i64` or `n != 0`.
- `str <-> i64`: use string parsing/formatting helpers or explicit raw pointer
  interop helpers.
- `*T <-> i64`: typed pointer values do not implicitly unify with raw integer
  addresses.
- Structural type changes between unrelated structs, enums, tuples, references,
  slices, and pointers.

```vais
# Allowed numeric adaptation
x: i8 = 10
y: i64 = x
z: f64 = 3

# Explicit conversion boundary
flag: bool = true
n: i64 = flag as i64

# Compile errors
bad_bool: i64 = true
bad_str: i64 = "hello"
```

Protection tests:

- `crates/vaisc/tests/e2e/phase158_type_strict.rs`
- `crates/vais-types/tests/inference_unification_tests.rs`

## Functions

Functions may use expression bodies or block bodies. The last block expression
is the return value unless an explicit `return` exits early.

```vais
fn add(a: i64, b: i64) -> i64 = a + b

fn abs(x: i64) -> i64 {
    I x < 0 { 0 - x } else { x }
}

fn checked_div(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 {
        return Err("division by zero")
    }
    Ok(a / b)
}
```

The self-recursion operator `@` calls the current function.

```vais
fn fib(n: i64) -> i64 = I n < 2 { n } else { @(n - 1) + @(n - 2) }
```

## Variables

```vais
x := 42
y: i32 = 7
counter := mut 0
counter = counter + 1
```

## Control Flow

`I`, `match`, and block forms are expressions when all arms produce compatible
types.

```vais
fn classify(n: i64) -> str {
    I n < 0 { "negative" }
    else I n == 0 { "zero" }
    else { "positive" }
}

fn color_code(c: Color) -> i64 {
    match c {
        Red => 1,
        Green => 2,
        Blue => 3,
    }
}
```

Loops use compact control forms.

```vais
fn sum_to(n: i64) -> i64 {
    total := mut 0
    LF i: 0..n {
        total = total + i
    }
    total
}

fn first_even(limit: i64) -> i64 {
    i := mut 0
    LW i < limit {
        I i % 2 == 0 { return i }
        i = i + 1
    }
    -1
}
```

## Structs

```vais
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn manhattan(&self) -> f64 {
        self.x + self.y
    }
}
```

Struct values have named fields. Generic structs are specialized where the
compiler has concrete type arguments.

## Enums

```vais
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Enum tags are declaration-order indexes starting at `0`. Pattern matching uses
the same declaration-order tags; it must not hardcode variant meaning from a
variant name except through the registered enum definition.

## Enum Runtime Layout

The LLVM backend uses one canonical runtime layout for each enum declaration:

- Unit-only enum: `%Enum = type { i32 }`
- Payload enum: `%Enum = type { i32, { i64, ... } }`

The first field is always an `i32` tag. Payload fields are stored in a nested
payload struct. `Option<T>` and `Result<T, E>` use the same canonical erased
layout for the current runtime surface:

```llvm
%Option = type { i32, { i64 } }
%Result = type { i32, { i64 } }
```

The compiler must not mix this layout with anonymous enum literals such as
`{ i8, i64 }`. Constructor, call, match, `?`, and `!` lowering all use the same
registered layout.

Protection tests:

- `crates/vais-codegen/tests/inkwell_enum_layout_tests.rs`
- `crates/vais-codegen/tests/text_enum_layout_tests.rs`
- `crates/vaisc/tests/e2e/phase90_enums.rs`
- `crates/vaisc/tests/e2e/phase41_error_handling.rs`

## Pattern Matching

```vais
fn unwrap_or<T>(opt: Option<T>, default: T) -> T {
    match opt {
        Some(v) => v,
        None => default,
    }
}

fn describe(n: i64) -> str {
    match n {
        0 => "zero",
        1 => "one",
        _ => "many",
    }
}
```

## Error Handling

`Result<T, E>` and `Option<T>` are explicit enum types. The try operator `?`
propagates the error/empty branch. The unwrap operator `!` extracts the payload
or panics.

```vais
fn parse_and_double(s: str) -> Result<i64, str> {
    n := parse_i64(s)?
    Ok(n * 2)
}

fn get_or_zero(opt: Option<i64>) -> i64 = opt!
```

## Generics

```vais
fn identity<T>(x: T) -> T = x

struct Pair<A, B> {
    first: A,
    second: B,
}
```

Generic functions and structs are specialized on concrete type arguments where
the compiler can prove the instantiation. Runtime-erased surfaces must be
specified explicitly, not inferred from backend convenience.

## Traits And Impl

```vais
trait Printable {
    fn print(&self) -> i64
}

struct Counter { value: i64 }

impl Counter: Printable {
    fn print(&self) -> i64 {
        print_i64(self.value)
        0
    }
}
```

## References And Lifetimes

Reference-to-value conversion is explicit. A `&T` does not silently become `T`;
use dereference at the boundary.

```vais
fn read(x: &i64) -> i64 = *x
```

Lifetime elision is permitted only on the tested surface. Ambiguous reference
returns require explicit lifetime information and must fail at type-check time
rather than falling through to backend errors.

## Async, Extern, And Globals

```vais
A fn fetch(id: i64) -> Result<str, str> {
    Ok("data")
}

N fn malloc(size: i64) -> i64
N fn free(ptr: i64) -> i64

G counter: i64 = 0
```

Async and non-native targets are implementation surfaces unless a named gate
certifies a narrower claim.

## Built-Ins

Common built-ins include:

```vais
puts(s: str) -> i64
print_i64(x: i64) -> i64
malloc(size: i64) -> i64
free(ptr: i64) -> i64
load_i64(ptr: i64) -> i64
store_i64(ptr: i64, value: i64) -> i64
sizeof(T) -> i64
```

Raw memory built-ins operate on `i64` addresses. Code using typed pointers must
cross that boundary explicitly.

## Compatibility Rule

If a syntax form is accepted by the parser but not listed here as canonical or
current compact syntax, it is a compatibility surface. New public docs,
playground examples, and compiler gate names should not promote compatibility
syntax as the current language contract.
