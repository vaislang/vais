# Vais Language Specification

Version: 0.0.1

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
11. [Traits and Implementations](#traits-and-implementations)
12. [Pattern Matching](#pattern-matching)
13. [Generics](#generics)
14. [Module System](#module-system)
15. [Async/Await](#asyncawait)
16. [Memory Management](#memory-management)

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
| `C` | Continue | Continue to next loop iteration |

Note: Constants are defined with the `C` keyword followed by identifier, type, and value (see [Constants](#constants)).

### Multi-letter Keywords

- `mut` - Mutable variable/reference
- `self` - Instance reference
- `Self` - Type reference in impl
- `true`, `false` - Boolean literals
- `spawn` - Spawn async task
- `await` - Await async result
- `weak` - Weak reference
- `clone` - Clone operation

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
| `?` | Ternary conditional | `x > 0 ? 1 : -1` |
| `.` | Member access | `point.x` |
| `::` | Path separator | `std::math::PI` |
| `->` | Function return type | `F add()->i64` |
| `=>` | Match arm separator | `0 => "zero"` |
| `..` | Range (exclusive) | `0..10` |
| `..=` | Range (inclusive) | `0..=10` |

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

### Block Expressions

Blocks are expressions that return the value of their last expression:

```vais
{
    x := 10
    y := 20
    x + y    # Returns 30
}
```

---

## Statements

### Variable Declaration

```vais
# Type-inferred
x := 10

# Explicit type
y: i64 = 20

# Mutable (future)
mut z := 30
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

### Match with Guards (future)

```vais
M value {
    x if x > 0 => "positive",
    x if x < 0 => "negative",
    _ => "zero"
}
```

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

    # Chain async calls
    data := fetch_data("example.com").await

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

## Best Practices

1. **Use type inference** with `:=` when the type is obvious
2. **Use expression forms** for simple functions: `F add(a:i64,b:i64)->i64 = a+b`
3. **Use self-recursion `@`** for cleaner recursive functions
4. **Pattern match** instead of nested if-else chains
5. **Leverage generics** to reduce code duplication
6. **Import only needed modules** to keep token count low
7. **Use match exhaustiveness** to catch all cases

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

---

## Grammar Summary

```
Program      ::= Item*
Item         ::= Function | Struct | Enum | Trait | Impl | Use | Const

Function     ::= 'F' Ident TypeParams? '(' Params? ')' '->' Type ('=' Expr | Block)
               | 'A' 'F' Ident TypeParams? '(' Params? ')' '->' Type Block
               | 'X' 'F' Ident '(' Params? ')' '->' Type

Struct       ::= 'S' Ident TypeParams? '{' Fields '}'
Enum         ::= 'E' Ident TypeParams? '{' Variants '}'
Trait        ::= 'W' Ident TypeParams? '{' TraitItems '}'
Impl         ::= 'X' Ident TypeParams? (':' Trait)? '{' ImplItems '}'
Use          ::= 'U' Path
Const        ::= 'C' Ident ':' Type '=' Expr

Expr         ::= Literal
               | Ident
               | BinaryExpr
               | UnaryExpr
               | CallExpr
               | FieldExpr
               | IndexExpr
               | TernaryExpr
               | IfExpr
               | LoopExpr
               | MatchExpr
               | BlockExpr
               | '@' '(' Args ')'

Stmt         ::= 'R' Expr?
               | 'B'
               | 'C'
               | Let
               | Expr

Type         ::= PrimitiveType
               | Ident TypeArgs?
               | '*' Type
               | '[' Type ']'
```

---

## Conclusion

Vais is designed to be a minimal yet powerful systems programming language optimized for AI code generation. Its single-letter keywords, expression-oriented design, and self-recursion operator make it highly token-efficient while maintaining the expressiveness needed for complex systems programming tasks.

For more examples, see the `/examples` directory. For standard library documentation, see `STDLIB.md`.
