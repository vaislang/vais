# Vais Syntax Guide

This guide covers the complete syntax of Vais.

## Table of Contents

- [Comments](#comments)
- [Data Types](#data-types)
- [Variables](#variables)
- [Operators](#operators)
- [Functions](#functions)
- [Control Flow](#control-flow)
- [Collections](#collections)
- [Collection Operations](#collection-operations)
- [Modules](#modules)
- [FFI](#ffi)
- [Best Practices](#best-practices)

---

## Comments

```vais
// Single-line comment

/*
   Multi-line
   comment
*/
```

---

## Data Types

### Primitives

| Type | Example | Description |
|------|---------|-------------|
| `Int` | `42`, `-17`, `0` | 64-bit signed integer |
| `Float` | `3.14`, `-0.5`, `1.0e10` | 64-bit floating point |
| `Bool` | `true`, `false` | Boolean |
| `String` | `"hello"`, `'world'` | UTF-8 string |
| `Nil` | `nil` | Null/None value |

### Collections

| Type | Example | Description |
|------|---------|-------------|
| `Array` | `[1, 2, 3]` | Ordered list |
| `Map` | `{a: 1, b: 2}` | Key-value pairs |
| `Range` | `1..10` | Integer range (exclusive end) |

### Type Annotations (Optional)

```vais
// Function with type annotations
add(a: Int, b: Int): Int = a + b

// Parameter types
greet(name: String) = "Hello, " ++ name
```

---

## Variables

### Declaration

```vais
// Immutable by default
x = 10
name = "Vais"
numbers = [1, 2, 3]
```

### Destructuring

```vais
// Array destructuring
[first, second, ...rest] = [1, 2, 3, 4, 5]
// first = 1, second = 2, rest = [3, 4, 5]

// Struct destructuring
{name, age} = {name: "Alice", age: 30}
```

---

## Operators

### Arithmetic

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `5 + 3` = `8` |
| `-` | Subtraction | `5 - 3` = `2` |
| `*` | Multiplication | `5 * 3` = `15` |
| `/` | Division | `10 / 3` = `3` |
| `%` | Modulo | `10 % 3` = `1` |
| `-` | Negation (unary) | `-5` |

### Comparison

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `5 == 5` = `true` |
| `!=` | Not equal | `5 != 3` = `true` |
| `<` | Less than | `3 < 5` = `true` |
| `>` | Greater than | `5 > 3` = `true` |
| `<=` | Less or equal | `5 <= 5` = `true` |
| `>=` | Greater or equal | `5 >= 3` = `true` |

### Logical

| Operator | Description | Example |
|----------|-------------|---------|
| `&&` | Logical AND | `true && false` = `false` |
| `\|\|` | Logical OR | `true \|\| false` = `true` |
| `!` | Logical NOT | `!true` = `false` |

### String and Collection

| Operator | Description | Example |
|----------|-------------|---------|
| `++` | Concatenation | `"a" ++ "b"` = `"ab"` |
| `#` | Length | `#[1,2,3]` = `3` |

---

## Functions

### Basic Definition

```vais
// Single expression (no braces needed)
add(a, b) = a + b

// Call
result = add(10, 20)  // 30
```

### Self-Recursion with `$`

The `$` operator calls the current function recursively:

```vais
// Factorial
factorial(n) = n < 2 ? 1 : n * $(n - 1)

factorial(5)  // 120

// Fibonacci
fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)

fib(10)  // 55
```

### Lambda Expressions

```vais
// Lambda with explicit parameter
double = (x) => x * 2

// Lambda placeholder (_)
triple = _ * 3

// Use in collection operations
[1, 2, 3].@(_ * 2)        // [2, 4, 6]
[1, 2, 3].@((x) => x * 2) // same result
```

### Higher-Order Functions

```vais
// Function returning function
make_adder(n) = (x) => x + n

add5 = make_adder(5)
add5(10)  // 15

// Function as parameter
apply(f, x) = f(x)
apply(_ * 2, 10)  // 20
```

### Public Functions

```vais
// Exported function (visible to other modules)
pub add(a, b) = a + b

// Private function (default)
helper(x) = x * 2
```

---

## Control Flow

### Ternary Expression

```vais
// condition ? then : else
max(a, b) = a > b ? a : b

// Nested ternary
sign(n) = n > 0 ? "positive" : n < 0 ? "negative" : "zero"
```

### If Expression

```vais
// if-then-else expression
result = if x > 0 then "positive" else "non-positive"

// Multi-line (for complex logic)
classify(n) = if n > 0 then
    "positive"
else if n < 0 then
    "negative"
else
    "zero"
```

---

## Collections

### Arrays

```vais
// Creation
empty = []
numbers = [1, 2, 3, 4, 5]
mixed = [1, "two", true, [3, 4]]

// Access
first = numbers[0]      // 1
last = numbers[-1]      // 5 (negative index)

// Range creation
one_to_ten = [1..11]    // [1, 2, 3, ..., 10]
```

### Maps/Structs

```vais
// Creation
person = {name: "Alice", age: 30}

// Access
person.name      // "Alice"
person["name"]   // "Alice"

// Nested
data = {
    user: {name: "Bob", id: 123},
    items: [1, 2, 3]
}
data.user.name   // "Bob"
```

### Ranges

```vais
// Exclusive range (end not included)
1..5     // [1, 2, 3, 4]

// Use with collection operations
[1..6].@(_ * 2)   // [2, 4, 6, 8, 10]
```

---

## Collection Operations

Vais provides concise operators for common functional operations.

### Map `.@`

Transform each element:

```vais
[1, 2, 3].@(_ * 2)           // [2, 4, 6]
[1, 2, 3].@((x) => x + 1)    // [2, 3, 4]

// Nested access
users.@(_.name)              // Extract names
```

### Filter `.?`

Keep elements matching predicate:

```vais
[1, 2, 3, 4, 5].?(_ > 2)         // [3, 4, 5]
[1, 2, 3, 4, 5].?(_ % 2 == 0)    // [2, 4]

// Filter objects
users.?(_.age >= 18)             // Adult users
```

### Reduce `./`

Fold elements into single value:

```vais
// Sum: ./+
[1, 2, 3, 4, 5]./+(0, _ + _)     // 15

// Product: ./*
[1, 2, 3, 4, 5]./*(1, _ * _)     // 120

// Custom reduce
[1, 2, 3]./(10, _ + _)           // 10 + 1 + 2 + 3 = 16
```

### Chaining Operations

```vais
// Filter then map
[1..10].?(_ % 2 == 0).@(_ * _)   // [4, 16, 36, 64]

// Complex pipeline
data
    .?(_.active)          // Filter active items
    .@(_.value)           // Extract values
    ./+(0, _ + _)         // Sum them
```

---

## Modules

### Importing

```vais
// Import specific functions
use math.{sin, cos, pi}

// Import with alias
use json.parse as json_parse

// Import all (not recommended)
use utils.*
```

### Exporting

```vais
// Public function (exported)
pub add(a, b) = a + b

pub calculate(x, y) = {
    sum: add(x, y),
    diff: x - y
}

// Private helper (not exported)
helper(x) = x * 2
```

### Module File Structure

```
my-project/
├── vais.toml
├── src/
│   ├── main.vais
│   └── utils.vais
└── lib/
    └── math.vais
```

---

## FFI

### Foreign Function Interface

Declare external C functions:

```vais
ffi "c" {
    fn abs(n: i32) -> i32
    fn sqrt(x: f64) -> f64
    fn pow(base: f64, exp: f64) -> f64
    fn strlen(s: cstr) -> i64
    fn getenv(key: cstr) -> cstr
}
```

### FFI Types

| Vais Type | C Type | Description |
|-----------|--------|-------------|
| `i8`, `i16`, `i32`, `i64` | `int8_t`, etc. | Signed integers |
| `u8`, `u16`, `u32`, `u64` | `uint8_t`, etc. | Unsigned integers |
| `f32` | `float` | 32-bit float |
| `f64` | `double` | 64-bit float |
| `bool` | `bool` | Boolean |
| `cstr` | `char*` | C string |
| `ptr` | `void*` | Generic pointer |
| `void` | `void` | No return value |

### Using FFI Functions

```vais
ffi "c" {
    fn abs(n: i32) -> i32
    fn sqrt(x: f64) -> f64
}

print(abs(-42))      // 42
print(sqrt(16.0))    // 4.0
```

---

## Best Practices

1. **Use `$` for recursion** - More concise than repeating the function name
2. **Use `_` in lambdas** - Cleaner for simple transformations
3. **Chain operations** - Compose `.@`, `.?`, `./` for readable pipelines
4. **Prefer ternary** - For simple conditionals
5. **Type annotations** - Add for complex functions (optional but helpful)
6. **Module organization** - Keep related functions together

### Complete Example

```vais
// Module imports
use std.io.{read_file, write_file}

// Public functions
pub factorial(n) = n < 2 ? 1 : n * $(n - 1)

pub fibonacci(n) = n < 2 ? n : $(n - 1) + $(n - 2)

pub quicksort(arr) =
    #arr <= 1 ? arr :
    let pivot = arr[0] in
    let less = arr[1..].?((_ < pivot)) in
    let greater = arr[1..].?((_ >= pivot)) in
    $(less) ++ [pivot] ++ $(greater)

// Main program
numbers = [5, 2, 8, 1, 9, 3]
sorted = quicksort(numbers)

print("Original:", numbers)
print("Sorted:", sorted)
print("Factorials:", [1..6].@(factorial(_)))
print("Fibonacci:", [0..10].@(fibonacci(_)))
```

---

## Related Documentation

- [Getting Started](./getting-started.md) - Installation and setup
- [API Reference](./api.md) - Built-in functions
- [Examples](./examples.md) - More code examples
