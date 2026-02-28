# Vais Language Specification

Version: 1.0.0

## Table of Contents

1. [Overview](#overview)
2. [Lexical Structure](#lexical-structure)
3. [Keywords](#keywords)
4. [Types](#types)
5. [Operators](#operators)
6. [Expressions](#expressions)
7. [Statements](#statements)
8. [Functions](#functions)
9. [Structs](#structs)
10. [Enums](#enums)
11. [Error Handling](#error-handling)
12. [Traits and Implementations](#traits-and-implementations)
13. [Pattern Matching](#pattern-matching)
14. [Generics](#generics)
15. [Module System](#module-system)
16. [Async/Await](#asyncawait)
17. [Iterators and Generators](#iterators-and-generators)
18. [Closures and Lambdas](#closures-and-lambdas)
19. [Memory Management](#memory-management)
20. [Built-in Functions](#built-in-functions)
21. [Constants](#constants)
22. [Package Ecosystem](#package-ecosystem)
23. [Best Practices](#best-practices)
24. [Examples](#examples)
25. [Grammar Summary](#grammar-summary)

---

## Overview

Vais is a token-efficient, AI-optimized systems programming language designed to minimize token usage in AI code generation while maintaining full systems programming capabilities. It features:

- **Single-letter keywords** for maximum token efficiency
- **Expression-oriented syntax** where everything returns a value
- **Self-recursion operator `@`** for concise recursive functions
- **LLVM-based compilation** for native performance
- **Type inference** with minimal annotations
- **Advanced features**: Generics, Traits, Async/Await, Pattern Matching

---

## Lexical Structure

### Comments

Comments start with `#` and continue to the end of the line:

```vais
# This is a comment
F add(a:i64, b:i64)->i64 = a + b  # Inline comment
```

### Whitespace

Whitespace (spaces, tabs, newlines) is ignored except when separating tokens.

### Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, or underscores:

```
[a-zA-Z_][a-zA-Z0-9_]*
```

Examples: `x`, `my_var`, `Counter`, `_private`

### Literals

**Integer Literals:**
```vais
42
1_000_000    # Underscores for readability
-42          # Negative (using unary minus operator)
```

**Float Literals:**
```vais
3.14
1.0e10
2.5e-3
1_000.5_00
```

**String Literals:**
```vais
"Hello, World!"
"Line with \"quotes\""
```

**String Interpolation:**
```vais
name := "Vais"
println("Hello, ~{name}!")           # Variable interpolation
println("Result: ~{2 + 3}")         # Expression interpolation
println("Escaped: {{not interp}}") # Escaped braces
```

**Boolean Literals:**
```vais
true
false
```

---

## Keywords

Vais uses single-letter keywords for maximum token efficiency:

| Keyword | Meaning | Usage |
|---------|---------|-------|
| `F` | Function | Define a function |
| `S` | Struct | Define a struct type |
| `E` | Enum (or Else) | Define enum type, or else branch in if |
| `I` | If | Conditional expression |
| `L` | Loop | Loop construct |
| `M` | Match | Pattern matching |
| `W` | Trait (Where) | Define a trait (interface) |
| `X` | Impl (eXtend) | Implement methods or traits |
| `T` | Type | Type alias definition |
| `U` | Use | Import/use modules |
| `P` | Pub | Public visibility |
| `A` | Async | Async function marker |
| `R` | Return | Early return from function |
| `B` | Break | Break from loop |
| `C` | Continue/Const | Continue to next loop iteration, or Const for constants |
| `D` | Defer | Deferred execution |
| `N` | Extern | Foreign function declaration |
| `G` | Global | Global variable declaration |
| `O` | Union | C-style untagged union |
| `Y` | Yield/Await | Yield value (shorthand for await) |

Note: The `C` keyword has dual meaning - `C` for continue in loops, and `C` for constants (see [Constants](#constants)). Context determines usage.

### Multi-letter Keywords

- `mut` - Mutable variable/reference
- `self` - Instance reference
- `Self` - Type reference in impl
- `true`, `false` - Boolean literals
- `spawn` - Spawn async task
- `await` - Await async result (also available as `Y` shorthand)
- `weak` - Weak reference
- `clone` - Clone operation
- `yield` - Yield value in iterator/coroutine (simplified implementation)

### Shorthand Keywords (Phase 29)

| Shorthand | Replaces | Example |
|-----------|----------|---------|
| `Y` | `await` | `result.Y` (postfix await) |

---

## Types

### Primitive Types

**Integer Types:**
- `i8`, `i16`, `i32`, `i64`, `i128` - Signed integers
- `u8`, `u16`, `u32`, `u64`, `u128` - Unsigned integers

**Floating-Point Types:**
- `f32` - 32-bit floating point
- `f64` - 64-bit floating point

**Other Types:**
- `bool` - Boolean type (`true` or `false`)
- `str` - String type

### Pointer Types

```vais
*i64        # Pointer to i64
*T          # Pointer to type T
```

### Array Types

```vais
[i64]       # Array of i64
[T]         # Array of type T
```

### Generic Types

```vais
Option<T>   # Generic Option type
Vec<T>      # Generic vector type
Pair<A, B>  # Multiple type parameters
```

---

## Operators

### Arithmetic Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `a + b` |
| `-` | Subtraction or unary negation | `a - b`, `-x` |
| `*` | Multiplication | `a * b` |
| `/` | Division | `a / b` |
| `%` | Modulo | `a % b` |

### Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `a == b` |
| `!=` | Not equal | `a != b` |
| `<` | Less than | `a < b` |
| `>` | Greater than | `a > b` |
| `<=` | Less or equal | `a <= b` |
| `>=` | Greater or equal | `a >= b` |

### Logical Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `&` | Bitwise AND | `a & b` |
| `\|` | Bitwise OR | `a \| b` |
| `^` | Bitwise XOR | `a ^ b` |
| `!` | Logical NOT | `!x` |
| `~` | Bitwise NOT | `~x` |
| `<<` | Left shift | `a << 2` |
| `>>` | Right shift | `a >> 2` |

### Assignment Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `=` | Assignment | `x = 10` |
| `:=` | Type-inferred assignment | `x := 10` |
| `+=` | Add and assign | `x += 5` |
| `-=` | Subtract and assign | `x -= 5` |
| `*=` | Multiply and assign | `x *= 2` |
| `/=` | Divide and assign | `x /= 2` |

### Special Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `@` | Self-recursion | `@(n-1)` |
| `?` | Ternary conditional or Try operator | `x > 0 ? 1 : -1` or `file.read()?` |
| `!` | Logical NOT or Unwrap operator | `!x` or `option!` |
| `.` | Member access | `point.x` |
| `::` | Path separator | `std::math::PI` |
| `->` | Function return type | `F add()->i64` |
| `=>` | Match arm separator | `0 => "zero"` |
| `..` | Range (exclusive) / Spread | `0..10`, `[..arr]` |
| `..=` | Range (inclusive) | `0..=10` |
| `\|>` | Pipe operator | `x \|> f \|> g` (equivalent to `g(f(x))`) |

**Note on `?` operator:** The `?` operator has two uses:
- **Ternary conditional**: `condition ? true_val : false_val`
- **Try operator**: `result?` - propagates errors to caller (see [Error Handling](#error-handling))

### Operator Precedence

Operators are listed from highest to lowest precedence:

| Precedence | Operators | Description |
|------------|-----------|-------------|
| 1 (highest) | `.`, `[]`, `()` | Member access, Index, Call |
| 2 | `-`, `!`, `~`, `@` | Unary operators |
| 3 | `*`, `/`, `%` | Multiplication, Division, Modulo |
| 4 | `+`, `-` | Addition, Subtraction |
| 5 | `<<`, `>>` | Bit shifts |
| 6 | `&` | Bitwise AND |
| 7 | `^` | Bitwise XOR |
| 8 | `\|` | Bitwise OR |
| 9 | `==`, `!=`, `<`, `>`, `<=`, `>=` | Comparison |
| 10 | `&&` | Logical AND |
| 11 | `\|\|` | Logical OR |
| 12 | `?:`, `\|>` | Ternary conditional, Pipe |
| 13 (lowest) | `=`, `:=`, `+=`, `-=`, `*=`, `/=` | Assignment |

**Note:** Bitwise `&` has higher precedence than comparison operators like `==`. Use parentheses to clarify: `(a == b) & (c == d)`.

---

## Expressions

Everything in Vais is an expression that returns a value.

### Literals

```vais
42          # Integer
3.14        # Float
"hello"     # String
true        # Boolean
```

### Variable References

```vais
x
my_variable
```

### Binary Expressions

```vais
a + b
x * y - z
a == b
```

### Unary Expressions

```vais
-x
!flag
~bits
```

### Function Calls

```vais
add(1, 2)
compute(x, y, z)
obj.method()
```

### Ternary Conditional

```vais
condition ? true_value : false_value
x > 0 ? x : -x    # Absolute value
```

### Array/Index Access

```vais
arr[0]
data[i * 2 + 1]
```

### Member Access

```vais
point.x
counter.value
obj.method()
```

### Self-Recursion

The `@` operator calls the current function recursively:

```vais
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)
```

Equivalent to:
```vais
F fib(n:i64)->i64 = n<2 ? n : fib(n-1) + fib(n-2)
```

### Pipe Operator

The `|>` operator passes the left-hand value as the first argument to the right-hand function:

```vais
# x |> f is equivalent to f(x)
result := 5 |> double |> add_one

# Chaining multiple transformations
F double(x: i64) -> i64 = x * 2
F add_one(x: i64) -> i64 = x + 1

F main() -> i64 = 5 |> double |> add_one  # 11
```

### String Interpolation

Embed expressions inside string literals with `~{expr}`:

```vais
name := "World"
println("Hello, ~{name}!")          # Variable
println("Sum: ~{2 + 3}")           # Expression
println("Escaped: {{braces}}")    # Literal braces with {{ }}
```

### Tuple Destructuring

Unpack tuple values into multiple variables:

```vais
(a, b) := get_pair()
(x, y, z) := (1, 2, 3)
```

### Block Expressions

Blocks are expressions that return the value of their last expression:

```vais
{
    x := 10
    y := 20
    x + y    # Returns 30
}
```

### Auto-Return

Functions in Vais automatically return the value of their last expression. No explicit `R` (return) is needed unless early return is required:

```vais
F add(a: i64, b: i64) -> i64 {
    a + b    # Automatically returned
}

F max(a: i64, b: i64) -> i64 {
    I a > b {
        a    # Each branch returns its last expression
    } E {
        b
    }
}

# Explicit R is only needed for early return
F safe_divide(a: i64, b: i64) -> i64 {
    I b == 0 {
        R 0    # Early return
    }
    a / b      # Auto-returned
}
```

This applies to all block expressions including `I`/`E`, `M`, and `L`.

---

## Statements

### Variable Declaration

```vais
# Type-inferred (immutable)
x := 10

# Explicit type
y: i64 = 20

# Mutable
z := mut 30
```

### If-Else Expression

```vais
# Single-line ternary
result := x > 0 ? 1 : -1

# Block form
I x < 0 {
    -1
} E {
    0
}

# Else-if chain
I x < 0 {
    -1
} E I x == 0 {
    0
} E {
    1
}
```

Note: `E` is used for "else" in if expressions.

### Loop Expression

```vais
# Infinite loop
L {
    # ... body
    B  # Break
}

# Range loop
L i: 0..10 {
    puts("Iteration")
}

# Array iteration (conceptual)
L item: array {
    # ... process item
}
```

### Match Expression

```vais
M value {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "other"    # Wildcard/default
}

# With variable binding
M option {
    Some(x) => x,
    None => 0
}
```

### Break and Continue

```vais
L i: 0..100 {
    I i == 50 { B }      # Break
    I i % 2 == 0 { C }   # Continue
    process(i)
}
```

### Return Statement

```vais
F compute(x: i64) -> i64 {
    I x < 0 {
        R 0    # Early return
    }
    x * 2
}
```

---

## Functions

### Function Definition

**Expression form (single expression):**
```vais
F add(a:i64, b:i64)->i64 = a + b
```

**Block form:**
```vais
F factorial(n:i64)->i64 {
    I n < 2 {
        1
    } E {
        n * @(n-1)
    }
}
```

### Parameters

```vais
F example(x: i64, y: f64, name: str) -> i64 { ... }
```

### Parameter Type Inference

Parameter types can be omitted when they can be inferred from call sites:

```vais
# Types inferred from usage
F add(a, b) = a + b

# Mixed: some explicit, some inferred
F scale(x, factor: f64) -> f64 = x * factor

# The compiler infers types from how the function is called
F main() -> i64 {
    add(1, 2)       # a: i64, b: i64 inferred
    scale(3.0, 2.0)  # x: f64 inferred
    0
}
```

### Return Types

```vais
F returns_int() -> i64 { 42 }
F returns_nothing() -> i64 { 0 }  # Convention: 0 for void
```

### Generic Functions

```vais
F identity<T>(x: T) -> T = x

F swap<A, B>(a: A, b: B) -> (B, A) {
    (b, a)
}
```

### Self-Recursion

```vais
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)
F countdown(n:i64)->i64 = n<1 ? 0 : @(n-1)
```

### External Functions

Declare C functions with `X F`:

```vais
X F puts(s: i64) -> i64
X F malloc(size: i64) -> i64
X F sqrt(x: f64) -> f64
```

---

## Structs

### Struct Definition

```vais
S Point {
    x: f64,
    y: f64
}

S Person {
    name: str,
    age: i64
}
```

### Generic Structs

```vais
S Pair<T> {
    first: T,
    second: T
}

S Container<K, V> {
    key: K,
    value: V
}
```

### Struct Instantiation

```vais
p := Point { x: 10.0, y: 20.0 }
person := Person { name: "Alice", age: 30 }
pair := Pair { first: 1, second: 2 }
```

### Field Access

```vais
x_coord := p.x
person_age := person.age
```

---

## Enums

### Enum Definition

**Simple enum:**
```vais
E Color {
    Red,
    Green,
    Blue
}
```

**Enum with data:**
```vais
E Option {
    None,
    Some(i64)
}

E Result {
    Ok(i64),
    Err(str)
}
```

### Enum Usage

```vais
color := Red
opt := Some(42)
err := Err("file not found")
```

### Enum Implementation Blocks

Enums can have methods just like structs:

```vais
E Color {
    Red,
    Green,
    Blue
}

X Color {
    F is_warm(&self) -> bool {
        M self {
            Red => true,
            Green => false,
            Blue => false,
            _ => false
        }
    }

    F to_hex(&self) -> str {
        M self {
            Red => "#FF0000",
            Green => "#00FF00",
            Blue => "#0000FF",
            _ => "#000000"
        }
    }
}

# Usage
F main() -> i64 {
    c := Red
    I c.is_warm() {
        puts("This is a warm color")
    }
    puts(c.to_hex())
    0
}
```

---

## Error Handling

Vais uses a Result/Option-based error handling system without traditional try-catch blocks. Error handling is done through the `?` (try) and `!` (unwrap) operators.

### The `?` Operator (Error Propagation)

The `?` operator is used to propagate errors to the caller. When applied to a `Result<T, E>` or `Option<T>`, it:
- Returns the inner value if `Ok(value)` or `Some(value)`
- Early-returns the error/None to the calling function if `Err(e)` or `None`

```vais
E Result<T, E> {
    Ok(T),
    Err(E)
}

F read_file(path: str) -> Result<str, str> {
    # If open fails, propagate the error immediately
    file := open(path)?

    # If read fails, propagate the error
    data := file.read()?

    # Success case
    Ok(data)
}

F process() -> Result<i64, str> {
    # The ? operator automatically propagates errors
    content := read_file("config.txt")?

    # Continue processing if no error
    Ok(parse(content))
}
```

### The `!` Operator (Unwrap)

The `!` operator forcefully extracts the value from a `Result` or `Option`. If the value is `Err` or `None`, the program will panic:

```vais
# Unwrap an Option - panics if None
value := some_option!

# Unwrap a Result - panics if Err
data := some_result!

# Example usage
F get_config() -> Option<str> {
    # ... returns Some(config) or None
}

F main() -> i64 {
    # This panics if get_config returns None
    config := get_config()!

    puts(config)
    0
}
```

### Error Type Derivation

Use `#[derive(Error)]` to automatically implement error traits:

```vais
#[derive(Error)]
E AppError {
    NotFound(str),
    Permission(str),
    Network(str)
}

F find_user(id: i64) -> Result<str, AppError> {
    I id < 0 {
        Err(NotFound("User ID cannot be negative"))
    } E {
        Ok("User data")
    }
}
```

### Standard Library Error Module

Vais provides `std/error.vais` with common error handling utilities (similar to Rust's `anyhow` and `thiserror`):

```vais
U std/error

# Error trait implementation
F handle_errors() -> Result<i64, str> {
    data := read_file("data.txt")?
    result := process(data)?
    Ok(result)
}
```

---

## Traits and Implementations

### Trait Definition

```vais
W Printable {
    F print(&self) -> i64
}

W Comparable {
    F compare(&self, other: &Self) -> i64
}
```

### Trait Implementation

```vais
S Counter {
    value: i64
}

X Counter: Printable {
    F print(&self) -> i64 {
        puts("Counter value:")
        print_i64(self.value)
        0
    }
}
```

### Method Implementation (without trait)

```vais
X Counter {
    F increment(&self) -> i64 {
        self.value + 1
    }

    F double(&self) -> i64 {
        self.value * 2
    }
}
```

### Calling Methods

```vais
c := Counter { value: 42 }
c.print()
inc := c.increment()
dbl := c.double()
```

---

## Pattern Matching

### Basic Match

```vais
F classify(n: i64) -> str {
    M n {
        0 => "zero",
        1 => "one",
        _ => "other"
    }
}
```

### Match with Binding

```vais
F describe(opt: Option) -> i64 {
    M opt {
        Some(x) => x,
        None => 0
    }
}
```

### Match with Guards

```vais
M value {
    x if x > 0 => "positive",
    x if x < 0 => "negative",
    _ => "zero"
}
```

### Pattern Alias

Pattern aliases allow you to bind a name to a matched pattern using the `@` operator. This is useful when you need both the matched value and access to the whole pattern:

```vais
# Basic pattern alias with range
F describe(n: i64) -> str {
    M n {
        x @ 1..10 => "small: ~{x}",
        x @ 10..100 => "medium: ~{x}",
        x @ 100..1000 => "large: ~{x}",
        _ => "very large"
    }
}

# Pattern alias with enum destructuring
E Option<T> {
    None,
    Some(T)
}

F process(opt: Option<i64>) -> i64 {
    M opt {
        val @ Some(x) => {
            # 'val' holds the entire Some variant
            # 'x' holds the inner value
            x * 2
        },
        None => 0
    }
}

# Pattern alias with struct destructuring
S Point {
    x: i64,
    y: i64
}

F classify_point(p: Point) -> str {
    M p {
        origin @ Point { x: 0, y: 0 } => "origin",
        pt @ Point { x, y } if x == y => "diagonal point",
        _ => "other point"
    }
}

# Nested pattern aliases
E Result<T, E> {
    Ok(T),
    Err(E)
}

F handle_result(r: Result<i64, str>) -> str {
    M r {
        success @ Ok(value) if value > 0 => "positive success",
        failure @ Err(msg) => "error: ~{msg}",
        _ => "zero or negative"
    }
}
```

The pattern alias `x @ pattern` syntax binds `x` to the matched value while also matching against `pattern`. This is particularly useful when:
- You need to refer to the matched value multiple times
- You want to combine pattern matching with guards
- You need both the whole value and destructured parts

### Destructuring

```vais
E Result {
    Ok(i64),
    Err(str)
}

M result {
    Ok(value) => value,
    Err(msg) => 0
}
```

### Wildcard Pattern

```vais
M x {
    0 => "zero",
    _ => "non-zero"   # Matches everything else
}
```

---

## Generics

### Generic Functions

```vais
F identity<T>(x: T) -> T = x

F max<T>(a: T, b: T) -> T {
    I a > b { a } E { b }
}
```

### Generic Structs

```vais
S Pair<T> {
    first: T,
    second: T
}

S Box<T> {
    value: T
}
```

### Generic Enums

```vais
E Option<T> {
    None,
    Some(T)
}

E Result<T, E> {
    Ok(T),
    Err(E)
}
```

### Generic Constraints (bounds)

```vais
# Function requiring trait bound
F print_all<T: Printable>(items: [T]) -> i64 {
    L item: items {
        item.print()
    }
    0
}
```

### Where Clauses

Where clauses provide an alternative syntax for specifying generic type constraints, especially useful when constraints are complex or numerous:

```vais
# Basic where clause
F find_max<T>(list: Vec<T>) -> T where T: Ord {
    result := mut list.get(0)
    L i: 1..list.len() {
        I list.get(i) > result {
            result = list.get(i)
        }
    }
    result
}

# Multiple bounds on a single type
F serialize<T>(val: T) -> str where T: Display, T: Clone {
    val.to_string()
}

# Multiple type parameters with bounds
F compare_and_print<T, U>(a: T, b: U) -> i64
    where T: Ord, U: Display
{
    puts(b.to_string())
    I a > a { 1 } E { 0 }
}

# Where clauses with structs
S Container<T> where T: Clone {
    value: T
}
```

Where clauses are especially useful when:
- Type constraints are complex or lengthy
- Multiple type parameters have constraints
- Constraints reference associated types
- You want to separate type parameters from their constraints for readability

---

## Module System

### Importing Modules

```vais
# Import from standard library
U std/math
U std/io

# Import custom module
U mymodule
```

### Using Imported Items

```vais
U std/math

F main() -> i64 {
    pi := PI              # Constant from math
    result := sqrt(16.0)  # Function from math
    puts("Square root: ")
    print_f64(result)
    0
}
```

### Module Paths

```vais
U std/io           # Standard library module
U std/collections  # Nested module path
```

---

## Async/Await

### Async Function Definition

```vais
A F compute(x: i64) -> i64 {
    x * 2
}

A F fetch_data(url: str) -> str {
    # ... async operations
    result
}
```

### Awaiting Async Functions

```vais
F main() -> i64 {
    # Call async function and await result
    result := compute(21).await

    # Using Y shorthand (equivalent to .await)
    result2 := compute(21).Y

    # Chain async calls
    data := fetch_data("example.com").Y

    0
}
```

### Spawning Tasks

```vais
# Spawn a task (runs concurrently)
task := spawn compute(42)

# Later, await the result
result := task.await
```

---

## Iterators and Generators

### The `yield` Keyword

The `yield` keyword is used to produce values in iterators and coroutines. In the current simplified implementation, `yield` returns a value from the iteration:

```vais
F counter(max: i64) -> i64 {
    L i: 0..max {
        yield i
    }
    0
}

F fibonacci(n: i64) -> i64 {
    a := 0
    b := 1
    L i: 0..n {
        yield a
        tmp := a + b
        a = b
        b = tmp
    }
    0
}
```

### Iterator Protocol

Vais implements an iterator protocol similar to Rust's Iterator trait. Collections can be iterated using the standard for-loop syntax:

```vais
# Iterate over a range
L i: 0..10 {
    puts("Value: ")
    print_i64(i)
}

# Iterate over an array
items := [1, 2, 3, 4, 5]
L item: items {
    print_i64(item)
}
```

### Iterator Adapters

Vais provides functional iterator adapters for transforming and filtering collections:

**`iter_map` - Transform each element:**
```vais
items := [1, 2, 3, 4, 5]
doubled := iter_map(items, |x| x * 2)
# Result: [2, 4, 6, 8, 10]
```

**`iter_filter` - Keep elements matching a predicate:**
```vais
numbers := [1, 2, 3, 4, 5, 6]
evens := iter_filter(numbers, |x| x % 2 == 0)
# Result: [2, 4, 6]
```

**`iter_take` - Take first N elements:**
```vais
data := [10, 20, 30, 40, 50]
first_three := iter_take(data, 3)
# Result: [10, 20, 30]
```

**`iter_skip` - Skip first N elements:**
```vais
data := [10, 20, 30, 40, 50]
after_two := iter_skip(data, 2)
# Result: [30, 40, 50]
```

**`iter_chain` - Concatenate two iterators:**
```vais
first := [1, 2, 3]
second := [4, 5, 6]
combined := iter_chain(first, second)
# Result: [1, 2, 3, 4, 5, 6]
```

**`iter_zip` - Combine two iterators pairwise:**
```vais
a := [1, 2, 3]
b := [10, 20, 30]
pairs := iter_zip(a, b)
# Result: [(1, 10), (2, 20), (3, 30)]
```

**`iter_enumerate` - Add indices to elements:**
```vais
items := ["a", "b", "c"]
indexed := iter_enumerate(items)
# Result: [(0, "a"), (1, "b"), (2, "c")]
```

### Chaining Iterator Adapters

Iterator adapters can be chained together for complex transformations:

```vais
numbers := [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

# Get first 5 even numbers, doubled
result := numbers
    |> iter_filter(|x| x % 2 == 0)
    |> iter_take(5)
    |> iter_map(|x| x * 2)
# Result: [4, 8, 12, 16, 20]
```

---

## Closures and Lambdas

### Basic Closures

Closures (also called lambdas or anonymous functions) are inline function definitions that can capture variables from their surrounding scope:

```vais
# Basic closure syntax
add_one := |x| x + 1

# Closure with multiple parameters
multiply := |x, y| x * y

# Closure with block body
complex := |n| {
    result := n * 2
    result + 10
}

# Using closures with iterator adapters
numbers := [1, 2, 3, 4, 5]
doubled := iter_map(numbers, |x| x * 2)
```

### Closure Capture Modes

Closures can capture variables from their surrounding scope in different ways. Vais provides explicit control over how variables are captured:

**By Value (Default):**

By default, closures capture variables by copying their values:

```vais
F main() -> i64 {
    x := 42

    # Closure captures 'x' by value (copy)
    add_x := |n| n + x

    result := add_x(10)  # Returns 52
    # Original 'x' is unchanged
    0
}
```

**Move Capture:**

The `move` keyword forces the closure to take ownership of captured variables by moving them into the closure:

```vais
F create_consumer() -> i64 {
    x := 42
    data := allocate_data()

    # Move 'x' and 'data' into the closure
    consumer := move |n| {
        # 'x' and 'data' are now owned by the closure
        result := n + x
        process(data)
        result
    }

    # Error: 'x' and 'data' have been moved
    # Cannot use them here anymore

    consumer(10)
}

# Common use case: returning closures
F make_adder(amount: i64) -> |i64| -> i64 {
    # Must use 'move' to transfer ownership
    move |x| x + amount
}

# Common use case: spawning async tasks
F spawn_worker(task_id: i64) -> i64 {
    data := load_task_data(task_id)

    # Move 'data' into the spawned task
    spawn move |()| {
        # 'data' is owned by this task
        process_task(data)
        0
    }

    0
}
```

### Capture Mode Summary

| Capture Mode | Syntax | Behavior | Use Case |
|--------------|--------|----------|----------|
| **By Value** | `\|args\| body` | Copies captured values | Default, when closure doesn't outlive scope |
| **Move** | `move \|args\| body` | Moves ownership into closure | Returning closures, spawning tasks, transferring ownership |

**Note:** By-reference capture modes (`&` and `&mut`) are part of the type system but require advanced lifetime analysis. The current implementation supports by-value and move semantics.

### Closure Examples

**Using closures with higher-order functions:**

```vais
# Filter with closure
F get_evens(numbers: [i64]) -> [i64] {
    iter_filter(numbers, |x| x % 2 == 0)
}

# Map with closure
F square_all(numbers: [i64]) -> [i64] {
    iter_map(numbers, |x| x * x)
}

# Chaining closures
F process_numbers(nums: [i64]) -> [i64] {
    nums
        |> iter_filter(|x| x > 0)
        |> iter_map(|x| x * 2)
        |> iter_take(10)
}
```

**Closures capturing multiple variables:**

```vais
F calculate(base: i64, multiplier: i64) -> i64 {
    # Closure captures both 'base' and 'multiplier'
    compute := |x| (x + base) * multiplier

    compute(5)
}
```

**Move semantics with spawned tasks:**

```vais
F parallel_process(data: Vec<i64>) -> i64 {
    L item: data {
        # Each task gets its own copy via move
        spawn move |()| {
            process_item(item)
            0
        }
    }
    0
}
```

---

## Memory Management

### Stack Allocation

Default allocation for local variables:

```vais
F main() -> i64 {
    x := 10        # Stack allocated
    p := Point { x: 1.0, y: 2.0 }  # Stack allocated
    0
}
```

### Heap Allocation

Use `malloc` and `free`:

```vais
# Allocate memory
ptr := malloc(64)

# Use memory
store_i64(ptr, 42)
value := load_i64(ptr)

# Free memory
free(ptr)
```

### Smart Pointers

**Box (unique ownership):**
```vais
U std/box

b := Box::new(42)
value := b.get()
```

**Rc (reference counting):**
```vais
U std/rc

rc := Rc::new(100)
rc2 := rc.clone()  # Increment ref count
value := rc.get()
```

### Arena Allocation

```vais
U std/arena

arena := Arena::with_capacity(1024)
ptr := arena.alloc(64)
# ... use allocated memory
arena.reset()  # Clear all allocations
```

---

## Built-in Functions

The following functions are provided by the compiler:

### I/O Functions

```vais
puts(s: str) -> i64          # Print string with newline
println(s: str) -> i64       # Print with interpolation support: println("x={x}")
print(s: str) -> i64         # Print without newline
putchar(c: i64) -> i64       # Print character
puts_ptr(ptr: i64) -> i64    # Print C string from pointer
```

### Memory Functions

```vais
malloc(size: i64) -> i64            # Allocate memory
free(ptr: i64) -> i64               # Free memory
memcpy(dst: i64, src: i64, n: i64) # Copy memory
load_i64(ptr: i64) -> i64           # Load i64 from memory
store_i64(ptr: i64, val: i64)       # Store i64 to memory
load_byte(ptr: i64) -> i64          # Load byte
store_byte(ptr: i64, val: i64)      # Store byte
```

### String Functions

```vais
strlen(s: i64) -> i64        # Get string length
```

### Utility Functions

```vais
print_i64(n: i64) -> i64     # Print integer
print_f64(n: f64) -> i64     # Print float
```

---

## Constants

Define compile-time constants with `C`:

```vais
C PI: f64 = 3.141592653589793
C MAX_SIZE: i64 = 1024
C VERSION: str = "0.0.1"
```

---

## Package Ecosystem

Vais includes a built-in package management system integrated into the `vaisc` compiler.

### Creating a New Package

Use `vaisc new` to create a new Vais project:

```bash
# Create a new package
vaisc new myproject

# Creates directory structure:
# myproject/
#   ├── Vais.toml       # Package manifest
#   ├── src/
#   │   └── main.vais   # Entry point
#   └── tests/          # Test directory
```

### Package Manifest (Vais.toml)

The `Vais.toml` file configures your package:

```toml
[package]
name = "myproject"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
edition = "2024"

[dependencies]
# Add dependencies here
# std = "1.0"

[dev-dependencies]
# Test-only dependencies

[build]
optimization = 2
target = "x86_64-unknown-linux-gnu"
```

### Running Tests

Use `vaisc test` to run all tests in your package:

```bash
# Run all tests
vaisc test

# Run specific test file
vaisc test tests/my_test.vais
```

**Writing tests:**
```vais
# tests/math_test.vais

F test_addition() -> i64 {
    result := add(2, 3)
    I result != 5 {
        puts("FAIL: expected 5, got different value")
        R 1
    }
    puts("PASS: addition works")
    0
}

F test_multiplication() -> i64 {
    result := multiply(3, 4)
    I result != 12 {
        puts("FAIL: expected 12")
        R 1
    }
    puts("PASS: multiplication works")
    0
}
```

### Package Commands

```bash
# Create new package
vaisc new <package_name>

# Build package
vaisc build

# Run package
vaisc run

# Run tests
vaisc test

# Check package validity
vaisc check

# Generate documentation
vaisc doc
```

### Package Tree and Documentation

View package structure and generate documentation:

```bash
# Show package dependency tree
vaisc pkg tree

# Generate package documentation
vaisc pkg doc
```

---

## Best Practices

1. **Use type inference** with `:=` when the type is obvious
2. **Use expression forms** for simple functions: `F add(a:i64,b:i64)->i64 = a+b`
3. **Use self-recursion `@`** for cleaner recursive functions
4. **Pattern match** instead of nested if-else chains
5. **Leverage generics** to reduce code duplication
6. **Import only needed modules** to keep token count low
7. **Use match exhaustiveness** to catch all cases
8. **Use `|>` pipe operator** for readable function chaining
9. **Use string interpolation** `println("x=~{x}")` instead of manual concatenation
10. **Omit parameter types** when they can be inferred from call sites
11. **Use `?` operator** for error propagation instead of manual match/return
12. **Use iterator adapters** (`iter_map`, `iter_filter`, etc.) for functional transformations
13. **Prefer `derive(Error)`** for custom error types to reduce boilerplate
14. **Use enum impl blocks** to add behavior to enums
15. **Structure projects** with `vaisc new` and `Vais.toml` for better organization

---

## Examples

### Hello World

```vais
F main()->i64 {
    puts("Hello, Vais!")
    0
}
```

### Fibonacci

```vais
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)

F main()->i64 = fib(10)
```

### Pattern Matching

```vais
E Option {
    None,
    Some(i64)
}

F unwrap_or(opt: Option, default: i64) -> i64 {
    M opt {
        Some(x) => x,
        None => default
    }
}
```

### Generic Struct

```vais
S Pair<T> {
    first: T,
    second: T
}

X Pair {
    F sum(&self) -> i64 {
        self.first + self.second
    }
}

F main() -> i64 {
    p := Pair { first: 10, second: 20 }
    p.sum()
}
```

### Error Handling with `?` and `!`

```vais
E FileError {
    NotFound(str),
    PermissionDenied(str)
}

F read_config(path: str) -> Result<str, FileError> {
    # Use ? to propagate errors
    file := open_file(path)?
    data := file.read_all()?
    Ok(data)
}

F main() -> i64 {
    # Use ! to unwrap (panics on error)
    config := read_config("config.txt")!
    puts(config)

    # Or handle errors with match
    result := read_config("data.txt")
    M result {
        Ok(content) => {
            puts("Success:")
            puts(content)
        },
        Err(NotFound(msg)) => {
            puts("File not found:")
            puts(msg)
        },
        Err(PermissionDenied(msg)) => {
            puts("Permission denied:")
            puts(msg)
        },
        _ => puts("Unknown error")
    }
    0
}
```

### Iterator Adapters

```vais
F main() -> i64 {
    numbers := [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

    # Chain iterator adapters
    result := numbers
        |> iter_filter(|x| x % 2 == 0)    # Keep even numbers
        |> iter_map(|x| x * x)             # Square them
        |> iter_take(3)                    # Take first 3

    # result = [4, 16, 36]

    L item: result {
        print_i64(item)
    }

    0
}
```

### Enum with Methods

```vais
#[derive(Error)]
E Color {
    Red,
    Green,
    Blue
}

X Color {
    F is_primary(&self) -> bool {
        true  # All are primary colors
    }

    F to_rgb(&self) -> (i64, i64, i64) {
        M self {
            Red => (255, 0, 0),
            Green => (0, 255, 0),
            Blue => (0, 0, 255),
            _ => (0, 0, 0)
        }
    }
}

F main() -> i64 {
    color := Red
    (r, g, b) := color.to_rgb()

    puts("RGB values:")
    print_i64(r)
    print_i64(g)
    print_i64(b)

    0
}
```

---

## Grammar Summary

The complete formal EBNF grammar is maintained at [`docs/grammar/vais.ebnf`](grammar/vais.ebnf)
(~320 productions, generated from the parser source). Ambiguity resolution rules and notation
conventions are documented in [`docs/grammar/README.md`](grammar/README.md).

Below is a condensed quick-reference:

```
Module       ::= Item*
Item         ::= Attribute* ['P'] (FunctionDef | StructDef | EnumDef | UnionDef
                 | TypeAlias | TraitAlias | UseDef | TraitDef | ImplDef
                 | MacroDef | ExternBlock | ConstDef | GlobalDef)

FunctionDef  ::= ['A'] 'F' Ident Generics? '(' Params? ')' ['->' Type] WhereClause? ('=' Expr | Block)
StructDef    ::= 'S' Ident Generics? WhereClause? '{' (Field | Method)* '}'
EnumDef      ::= 'E' Ident Generics? '{' Variant (',' Variant)* '}'
UnionDef     ::= 'O' Ident Generics? '{' Field (',' Field)* '}'
TraitDef     ::= 'W' Ident Generics? [':' TraitBounds] WhereClause? '{' (AssocType | TraitMethod)* '}'
ImplDef      ::= 'X' Generics? Type [':' Ident] WhereClause? '{' Method* '}'
ExternBlock  ::= 'N' StringLit? '{' ExternFunc* '}' | 'X' 'F' ExternFuncSig
UseDef       ::= 'U' Path ['.' ('{' Idents '}' | Ident)] [';']
ConstDef     ::= 'C' Ident ':' Type '=' Expr
GlobalDef    ::= 'G' Ident ':' Type '=' Expr
TypeAlias    ::= 'T' Ident Generics? '=' Type
TraitAlias   ::= 'T' Ident Generics? '=' TraitBound ('+' TraitBound)*
MacroDef     ::= 'macro' Ident '!' '{' MacroRule* '}'

Expr         ::= Assignment | Pipe | Ternary | LogicalOr | LogicalAnd
               | BitwiseOr | BitwiseXor | BitwiseAnd | Equality | Range
               | Comparison | Shift | Term | Factor | Unary | Postfix | Primary

Stmt         ::= 'R' Expr? | 'B' Expr? | 'C' | 'D' Expr | LetStmt | Expr

Type         ::= BaseType ['?' | '!']
BaseType     ::= NamedType | TupleType | FnType | ArrayType | MapType
               | PointerType | RefType | SliceType | DynTraitType
               | LinearType | AffineType | ImplTraitType | DependentType | FnPtrType

Pattern      ::= '_' | Ident ['@' Pattern] | Ident '(' Patterns ')' | Literal
               | '(' Patterns ')' | Pattern '..' Pattern | Pattern '|' Pattern

Closure      ::= '|' Params? '|' Expr | 'move' '|' Params? '|' Expr
```

See `docs/grammar/vais.ebnf` for the complete grammar with all 18 sections,
parser function cross-references, and operator precedence levels.

---

## Conclusion

Vais is designed to be a minimal yet powerful systems programming language optimized for AI code generation. Its single-letter keywords, expression-oriented design, and self-recursion operator make it highly token-efficient while maintaining the expressiveness needed for complex systems programming tasks.

For more examples, see the `/examples` directory. For standard library documentation, see `STDLIB.md`.
