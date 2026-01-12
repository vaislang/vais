# AOEL Core Language Design Principles

**Version:** 1.0.0
**Date:** 2026-01-12

---

## Overview

이 문서는 AOEL 코어 언어의 설계 원칙을 정의합니다.
코어는 **절대 변경되지 않아야 할 부분**으로, 신중하게 설계되어야 합니다.

---

## Core Philosophy

### 1. Minimal but Complete

```
코어에 포함되는 것:
✅ 기본 타입 (i, f, s, b, [T], {K:V}, ?T)
✅ 기본 연산자 (+, -, *, /, %, ==, !=, <, >, etc.)
✅ 제어 흐름 (조건, 반복 표현)
✅ 함수 정의/호출
✅ 모듈 시스템 기초
✅ 에러 처리 기초

코어에 포함되지 않는 것:
❌ 파일 I/O (std.io)
❌ 네트워크 (std.net)
❌ 데이터 포맷 (std.json)
❌ 고급 자료구조 (std.collections)
❌ 비동기 (std.async)
```

### 2. Orthogonal Design

각 기능은 독립적이고, 조합 가능해야 합니다.

```aoel
# 모든 것이 표현식
result = if cond { a } else { b }  # 조건도 표현식
value = { let x = 1; x + 2 }       # 블록도 표현식

# 연산자는 일관되게 동작
[1,2,3].@(_*2)      # 배열에 map
"abc".@(_.up)       # 문자열에 map (각 문자)
{a:1,b:2}.@(_*2)    # 맵의 값에 map
```

### 3. Explicit over Implicit

```aoel
# 타입은 명시하거나 추론 가능해야 함
add(a, b) = a + b           # OK: 사용처에서 추론
add(a:i, b:i) = a + b       # OK: 명시적 타입

# 부작용은 명시적
fn pure_fn(x) = x * 2       # 순수 함수
fn io_fn(path) = !{         # ! = 부작용 있음
    read_file(path)
}
```

### 4. Predictable Performance

```aoel
# 비용이 명확해야 함
arr.@(_*2)           # O(n) - 선형
arr.sort             # O(n log n) - 명확
arr.?(_>0).@(_*2)    # O(n) - 한 번 순회로 최적화 가능

# 숨겨진 비용 없음
# (Python의 `in`이 list는 O(n), set은 O(1)인 것과 달리)
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

```aoel
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

```aoel
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

```aoel
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

```aoel
# 조건문도 표현식
x = if a > b { a } else { b }

# 블록도 표현식
y = {
    let temp = compute()
    temp * 2
}

# match도 표현식
result = match status {
    Pending => "waiting",
    Active(s) => "active since " + s.str,
    Inactive(r) => "inactive: " + r
}
```

### Operator Expressions

```aoel
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

```aoel
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

```aoel
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

```aoel
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

```aoel
# Named recursion
fact(n) = if n < 2 { 1 } else { n * fact(n-1) }

# Self recursion with $
fact(n) = n < 2 ? 1 : n * $(n-1)

# Tail recursion (optimized)
fact(n, acc = 1) = n < 2 ? acc : $(n-1, n*acc)
```

### Higher-Order Functions

```aoel
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

```aoel
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

```aoel
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

```aoel
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

```aoel
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

```aoel
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

```aoel
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

```aoel
# file: math.aoel
mod math

# Public by default: pub
pub pi = 3.14159265359

pub fn sin(x: f) -> f = ...
pub fn cos(x: f) -> f = ...

# Private (not exported)
priv fn helper() = ...
```

### Import

```aoel
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

```aoel
pub     # public (anyone can use)
priv    # private (same module only)
pub(pkg)  # package-private (same package)
```

---

## Memory Model

### Ownership (Simplified)

```aoel
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

```aoel
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

```aoel
use std.async.spawn

# Spawn task
handle = spawn {
    expensive_computation()
}

# Wait for result
result = handle.await
```

### Channels

```aoel
use std.async.{channel, send, recv}

# Create channel
(tx, rx) = channel()

# Send
spawn { tx.send(42) }

# Receive
value = rx.recv()
```

### Async/Await

```aoel
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

다음 기능은 코어에 예약되어 있지만, 초기 버전에서는 구현하지 않습니다:

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

AOEL 코어 설계 원칙:

| 원칙 | 설명 |
|------|------|
| Minimal | 필수 기능만 코어에 |
| Orthogonal | 기능들이 독립적으로 조합 |
| Explicit | 암묵적 동작 최소화 |
| Expression | 모든 것이 값을 반환 |
| Type Safe | 컴파일 타임 타입 체크 |
| Extensible | 코어 변경 없이 확장 가능 |

**코어가 안정되면, 생태계가 성장합니다.**
