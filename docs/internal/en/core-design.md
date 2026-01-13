# Vais Core Language Design Principles

**Version:** 1.0.0
**Date:** 2026-01-12

---

## Overview

This document defines the design principles for the Vais core language.
The core is the part that **should never change**, so it must be designed carefully.

---

## Core Philosophy

### 1. Minimal but Complete

```
Included in core:
✅ Basic types (i, f, s, b, [T], {K:V}, ?T)
✅ Basic operators (+, -, *, /, %, ==, !=, <, >, etc.)
✅ Control flow (conditional, loop expressions)
✅ Function definition/calls
✅ Module system basics
✅ Error handling basics

NOT included in core:
❌ File I/O (std.io)
❌ Networking (std.net)
❌ Data formats (std.json)
❌ Advanced data structures (std.collections)
❌ Async (std.async)
```

### 2. Orthogonal Design

Each feature should be independent and composable.

```vais
# Everything is an expression
result = if cond { a } else { b }  # Conditional is expression
value = { let x = 1; x + 2 }       # Block is expression

# Operators work consistently
[1,2,3].@(_*2)      # Map over array
"abc".@(_.up)       # Map over string (each character)
{a:1,b:2}.@(_*2)    # Map over map values
```

### 3. Explicit over Implicit

```vais
# Types are specified or inferable
add(a, b) = a + b           # OK: inferred from usage
add(a:i, b:i) = a + b       # OK: explicit types

# Side effects are explicit
fn pure_fn(x) = x * 2       # Pure function
fn io_fn(path) = !{         # ! = has side effects
    read_file(path)
}
```

### 4. Predictable Performance

```vais
# Costs should be clear
arr.@(_*2)           # O(n) - linear
arr.sort             # O(n log n) - clear
arr.?(_>0).@(_*2)    # O(n) - can be optimized to single pass

# No hidden costs
# (Unlike Python where `in` is O(n) for list, O(1) for set)
```

---

## Type System

### Primitive Types

| Type | Full Name | Description | Example |
|------|-----------|-------------|---------|
| `i` | int | 64-bit signed integer | `42`, `-1` |
| `i32` | int32 | 32-bit signed integer | `42i32` |
| `i64` | int64 | 64-bit signed integer | `42i64` |
| `f` | float | 64-bit float | `3.14` |
| `f32` | float32 | 32-bit float | `3.14f32` |
| `s` | string | UTF-8 string | `"hello"` |
| `b` | bool | boolean | `true`, `false` |
| `void` | void | no value | - |

### Compound Types

| Type | Description | Example |
|------|-------------|---------|
| `[T]` | array of T | `[1, 2, 3]` |
| `{K:V}` | map from K to V | `{"a": 1, "b": 2}` |
| `(T1, T2, ...)` | tuple | `(1, "a", true)` |
| `?T` | optional T | `some(42)`, `nil` |
| `!T` | result (T or error) | `ok(42)`, `err("failed")` |

### User-Defined Types

```vais
# Struct
type User = {
    name: s,
    age: i,
    email: ?s
}

# Enum
type Status =
    | Pending
    | Active(since: i)
    | Inactive(reason: s)

# Type Alias
type UserId = i
type UserMap = {s: User}
```

### Generics

```vais
# Generic function
fn first<T>(arr: [T]) -> ?T = arr.nth(0)

# Generic type
type Result<T, E> =
    | Ok(value: T)
    | Err(error: E)

# Constraints
fn sum<T: Numeric>(arr: [T]) -> T = arr./+
```

### Traits

```vais
# Trait definition
trait Eq {
    fn eq(self, other: Self) -> b
    fn neq(self, other: Self) -> b = !self.eq(other)
}

trait Ord: Eq {
    fn cmp(self, other: Self) -> i  # -1, 0, 1
    fn lt(self, other: Self) -> b = self.cmp(other) < 0
    fn gt(self, other: Self) -> b = self.cmp(other) > 0
}

trait Mappable<T> {
    fn map<U>(self, f: T -> U) -> Self<U>
}

# Trait implementation
impl Eq for User {
    fn eq(self, other) = self.name == other.name
}
```

---

## Expressions

### Everything is an Expression

```vais
# Conditionals are expressions
x = if a > b { a } else { b }

# Blocks are expressions
y = {
    let temp = compute()
    temp * 2
}

# Match is expression
result = match status {
    Pending => "waiting",
    Active(s) => "active since " + s.str,
    Inactive(r) => "inactive: " + r
}
```

### Operator Expressions

```vais
# Arithmetic
a + b, a - b, a * b, a / b, a % b

# Comparison
a == b, a != b, a < b, a > b, a <= b, a >= b

# Logical
a && b, a || b, !a

# Collection
arr.@expr      # map
arr.?expr      # filter
arr./op        # reduce
a @ arr        # contains (in)
a..b           # range
#arr           # length
```

### Chain Expressions

```vais
# Method chaining
users
    .?(_.active)
    .@(_.name)
    .sort
    .take(10)

# Pipe operator (alternative)
users
    |> filter(_.active)
    |> map(_.name)
    |> sort
    |> take(10)
```

### Lambda Expressions

```vais
# Implicit parameter (_)
arr.@(_*2)              # each element * 2
arr.?(_.age > 18)       # filter by age

# Explicit parameter
arr.@(x => x * 2)
arr.?((u) => u.age > 18)

# Multiple parameters
zip(a, b).@((x, y) => x + y)
```

---

## Functions

### Function Definition

```vais
# Basic
add(a, b) = a + b

# With types
add(a: i, b: i) -> i = a + b

# Multi-line
process(data) = {
    let cleaned = clean(data)
    let validated = validate(cleaned)
    transform(validated)
}

# Default parameters
greet(name, greeting = "Hello") = greeting + ", " + name

# Named parameters (at call site)
create_user(name: "John", age: 30)
```

### Recursion

```vais
# Named recursion
fact(n) = if n < 2 { 1 } else { n * fact(n-1) }

# Self recursion with $
fact(n) = n < 2 ? 1 : n * $(n-1)

# Tail recursion (optimized)
fact(n, acc = 1) = n < 2 ? acc : $(n-1, n*acc)
```

### Higher-Order Functions

```vais
# Function as parameter
apply(f, x) = f(x)
apply(_*2, 5)  # 10

# Function as return value
multiplier(n) = (x) => x * n
double = multiplier(2)
double(5)  # 10

# Composition
compose(f, g) = (x) => f(g(x))
```

---

## Control Flow

### Conditional

```vais
# If expression
result = if cond { then_val } else { else_val }

# Ternary (shorthand)
result = cond ? then_val : else_val

# Chained
category =
    age < 13 ? "child" :
    age < 20 ? "teen" :
    age < 60 ? "adult" :
    "senior"
```

### Pattern Matching

```vais
# Match expression
match value {
    0 => "zero",
    1 => "one",
    n if n < 0 => "negative",
    _ => "other"
}

# Destructuring
match user {
    {name, age} if age >= 18 => "adult: " + name,
    {name, _} => "minor: " + name
}

# Option matching
match maybe_value {
    some(v) => process(v),
    nil => default_value
}
```

### Iteration

```vais
# For each (expression)
for x in arr { process(x) }

# With index
for (i, x) in arr.enum {
    print(i.str + ": " + x.str)
}

# Collection operations (preferred)
arr.@(process)           # map
arr.?(predicate)         # filter
arr./(acc, x => ...)     # fold
```

---

## Error Handling

### Option Type (?T)

```vais
# Creating
maybe_val: ?i = some(42)
nothing: ?i = nil

# Using
val = maybe_val ? default    # unwrap with default
val = maybe_val!             # unwrap or panic

# Chaining
result = maybe_val
    .@(_*2)           # map over option
    .?(_.> 0)         # filter option
    ? 0               # default if nil
```

### Result Type (!T)

```vais
# Creating
success: !i = ok(42)
failure: !i = err("something went wrong")

# Using with ?
fn read_data(path) -> !s = {
    content = read_file(path)?    # propagate error
    parsed = parse(content)?      # propagate error
    ok(parsed)
}

# Pattern matching
match result {
    ok(v) => process(v),
    err(e) => handle_error(e)
}
```

### Error Propagation

```vais
# ? operator
fn process() -> !Result = {
    a = step1()?        # return early if error
    b = step2(a)?       # return early if error
    ok(finalize(b))
}

# try block
fn safe_process() -> !Result = try {
    a = step1()
    b = step2(a)
    finalize(b)
}
```

---

## Module System

### Module Definition

```vais
# file: math.vais
mod math

# Public by default: pub
pub pi = 3.14159265359

pub sin(x: f) -> f = ...
pub cos(x: f) -> f = ...

# Private (not exported)
priv helper() = ...
```

### Import

```vais
# Import module
use math

# Import specific items
use math.{sin, cos, pi}

# Import with alias
use math.sin as sine

# Import all (discouraged)
use math.*
```

### Visibility

```vais
pub     # public (anyone can use)
priv    # private (same module only)
pub(pkg)  # package-private (same package)
```

---

## Memory Model

### Ownership (Simplified)

```vais
# Values are copied by default (primitives)
a = 5
b = a    # copy
a = 10   # b is still 5

# Collections are reference counted
arr1 = [1, 2, 3]
arr2 = arr1    # shared reference
arr1.push(4)   # both see [1,2,3,4]

# Explicit clone when needed
arr2 = arr1.clone()  # deep copy
```

### Mutability

```vais
# Immutable by default
x = 5
x = 10    # OK: rebinding

# Mutable binding
mut counter = 0
counter += 1    # OK: mutation

# Mutable parameters
fn increment(mut x: i) = {
    x += 1
    x
}
```

---

## Concurrency Primitives

### Spawn

```vais
use std.async.spawn

# Spawn task
handle = spawn {
    expensive_computation()
}

# Wait for result
result = handle.await
```

### Channels

```vais
use std.async.{channel, send, recv}

# Create channel
(tx, rx) = channel()

# Send
spawn { tx.send(42) }

# Receive
value = rx.recv()
```

### Async/Await

```vais
# Async function
async fn fetch_data(url: s) -> !s = {
    response = http.get(url).await?
    ok(response.body)
}

# Await
data = fetch_data("https://api.example.com").await?
```

---

## Reserved for Future

The following features are reserved in the core but not implemented in the initial version:

```
# Macros (future)
macro define! { ... }

# Compile-time computation (future)
const fn compile_time() = ...

# Effect system (future)
fn pure_fn() -> T pure = ...
fn io_fn() -> T io = ...

# Linear types (future)
fn consume(owned x: Resource) = ...
```

---

## Design Decisions Log

### Why `_` for lambda parameter?

```
Options considered:
1. x => x * 2       (verbose)
2. \x -> x * 2      (Haskell-like, unfamiliar)
3. |x| x * 2        (Rust-like)
4. _ * 2            (chosen: concise, familiar from Scala)

Decision: _ is shorter and frequently used in functional programming.
```

### Why `?:` instead of `if-else`?

```
Options considered:
1. if cond then a else b    (verbose)
2. if(cond, a, b)           (function-like)
3. cond ? a : b             (chosen: C-family familiar)

Decision: Ternary is more concise for simple cases.
         if-else block available for complex cases.
```

### Why `.@` `.?` `./` for map/filter/reduce?

```
Options considered:
1. map(arr, fn)             (function: 6 tokens)
2. arr.map(fn)              (method: 5 tokens)
3. arr >> map(fn)           (pipe: 5 tokens)
4. arr.@(fn)                (chosen: 4 tokens)

Decision: Operator form is most concise while still readable.
         Dot prefix groups them visually as collection operations.
```

### Why no classes?

```
Rationale:
- Classes mix data and behavior in ways that complicate reasoning
- Traits + structs provide same capability with clearer separation
- Functional style is more natural for AI generation
- Simpler mental model
```

---

## Summary

Vais core design principles:

| Principle | Description |
|-----------|-------------|
| Minimal | Only essential features in core |
| Orthogonal | Features combine independently |
| Explicit | Minimize implicit behavior |
| Expression | Everything returns a value |
| Type Safe | Compile-time type checking |
| Extensible | Extendable without core changes |

**Once the core is stable, the ecosystem grows.**
