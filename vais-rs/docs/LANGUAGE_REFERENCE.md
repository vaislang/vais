# Vais Language Reference

Complete reference for the Vais programming language.

## Table of Contents

1. [Syntax Overview](#syntax-overview)
2. [Types](#types)
3. [Functions](#functions)
4. [Operators](#operators)
5. [Collections](#collections)
6. [Control Flow](#control-flow)
7. [Pattern Matching](#pattern-matching)
8. [Modules](#modules)
9. [Error Handling](#error-handling)
10. [Async/Concurrency](#asyncconcurrency)

---

## Syntax Overview

### Comments

```vais
// Single-line comment
/* Multi-line
   comment */
```

### Variables

```vais
x = 42           // Immutable binding
name = "Vais"    // String
active = true    // Boolean
```

### Literals

```vais
// Numbers
42        // Integer
3.14      // Float
0xFF      // Hex
0b1010    // Binary

// Strings
"hello"   // String
'c'       // Character (planned)

// Collections
[1, 2, 3]           // Array
#{1, 2, 3}          // Set (converted to Array)
{a: 1, b: 2}        // Struct
(1, "two", 3.0)     // Tuple
```

---

## Types

### Primitive Types

| Type | Description | Examples |
|------|-------------|----------|
| `Int` | 64-bit integer | `42`, `-1`, `0xFF` |
| `Float` | 64-bit float | `3.14`, `-0.5`, `1e10` |
| `Bool` | Boolean | `true`, `false` |
| `String` | UTF-8 string | `"hello"` |
| `Void` | No value | (unit type) |

### Collection Types

| Type | Description | Example |
|------|-------------|---------|
| `[T]` | Array of T | `[Int]`, `[String]` |
| `#{T}` | Set of T | `#{Int}` |
| `{K: V}` | Map from K to V | `{String: Int}` |
| `(T1, T2, ...)` | Tuple | `(Int, String)` |

### Special Types

| Type | Description |
|------|-------------|
| `?T` | Optional T (may be none) |
| `!T` | Result T (may be error) |
| `Future<T>` | Async result |
| `Channel<T>` | Communication channel |

### Type Annotations

```vais
// Function with types
add(a: Int, b: Int) -> Int = a + b

// Generic function
identity<T>(x: T) -> T = x

// Complex types
process(data: [Int], fn: Int -> Int) -> [Int] = data.@(fn)
```

---

## Functions

### Basic Functions

```vais
// Simple function
square(x) = x * x

// Multiple parameters
add(a, b) = a + b

// With type annotations
multiply(x: Int, y: Int) -> Int = x * y
```

### Block Body

```vais
// Statements separated by semicolons
calculate(x, y) = {
    sum = x + y;
    product = x * y;
    sum + product   // Last expression is the return value
}
```

### Self-Recursion ($)

```vais
// $ refers to the enclosing function
factorial(n) = n < 2 ? 1 : n * $(n - 1)
fibonacci(n) = n < 2 ? n : $(n-1) + $(n-2)

// Tail-recursive
sum_to(n, acc) = n == 0 ? acc : $(n - 1, acc + n)
```

### Lambda Expressions

```vais
// Shorthand with _ (placeholder) - recommended
numbers.@(_ * 2)      // Double each element
numbers.?(_ > 0)      // Filter positive numbers
numbers.@(_ * _)      // Square each element

// Arrow syntax (in collection operations)
numbers.@((x) => x * 2)

// Pipe lambda syntax (inline only)
[1, 2, 3].@(|x| x * 2)
```

### Async Functions

```vais
async fetch_data(url) = {
    response = await http_get(url)
    await json_parse(response.body)
}
```

---

## Operators

### Arithmetic

| Operator | Description |
|----------|-------------|
| `+` | Addition |
| `-` | Subtraction |
| `*` | Multiplication |
| `/` | Division |
| `%` | Modulo |
| `^` | Power |

### Comparison

| Operator | Description |
|----------|-------------|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less or equal |
| `>=` | Greater or equal |

### Logical

| Operator | Description |
|----------|-------------|
| `and` | Logical AND |
| `or` | Logical OR |
| `not` | Logical NOT |

### Collection Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `.@(f)` | Map | `[1,2,3].@(_ * 2)` |
| `.?(p)` | Filter | `[1,2,3].?(_ > 1)` |
| `./+` | Sum reduce | `[1,2,3]./+` → 6 |
| `./*` | Product reduce | `[1,2,3]./*` → 6 |
| `./min` | Min reduce | `[1,2,3]./min` → 1 |
| `./max` | Max reduce | `[1,2,3]./max` → 3 |
| `./and` | Logical AND reduce | `[true,true]./and` |
| `./or` | Logical OR reduce | `[false,true]./or` |
| `elem @ arr` | Contains | `2 @ [1,2,3]` → true |

### Chaining Operations

```vais
// Chain collection operators instead of pipeline
data.@(process).@(format).@(output)

// Example: double -> filter > 10 -> sum
[1, 2, 3, 4, 5].@(_ * 2).?(_ > 5)./+  // 24
```

### String

| Operator | Description |
|----------|-------------|
| `++` | Concatenation |

---

## Collections

### Arrays

```vais
nums = [1, 2, 3, 4, 5]

// Access
first = nums[0]
last = nums[-1]

// Operations
len(nums)           // 5
first(nums)         // 1
last(nums)          // 5
reverse(nums)       // [5, 4, 3, 2, 1]
concat(nums, [6,7]) // [1, 2, 3, 4, 5, 6, 7]
```

### Sets

```vais
s = #{1, 2, 3}

// Operations
contains(s, 2)      // true
union(s, #{4, 5})   // #{1, 2, 3, 4, 5}
intersect(s, #{2})  // #{2}
```

### Structs

```vais
// Struct literal (use identifiers as keys, not strings)
person = {name: "Vais", version: "0.1"}

// Access with dot notation
person.name         // "Vais"
person.version      // "0.1"

// Nested structs
config = {
    server: {host: "localhost", port: 8080},
    debug: true
}
config.server.host  // "localhost"
```

---

## Control Flow

### Conditionals

```vais
// Ternary operator (primary conditional)
result = condition ? value_if_true : value_if_false

// If-then-else expression (alternative syntax)
result = if condition then value_if_true else value_if_false

// Nested ternary
classify(n) = n < 0 ? "negative" : n == 0 ? "zero" : "positive"

// If-then-else in function
classify(n) = if n > 0 then "positive" else "non-positive"

// In functions
max(a, b) = a > b ? a : b
abs(n) = n < 0 ? -n : n
```

### For Loop

```vais
// Iterate over array
for i in [1, 2, 3] {
    print(i)
}

// With range
for n in range(1, 10) {
    print(n)
}
```

### Pipeline Operator

```vais
// Pipeline: pass value to function
double(x) = x * 2
triple(x) = x * 3

5 |> double           // 10
5 |> double |> triple // 30

// Chained processing
value |> process |> format |> output
```

### Iteration (Functional Style)

```vais
// Use collection operators instead of loops
// Map over range
range(1, 10).@(print(_))

// Process each item
items.@(process(_))

// Sum with reduce
range(1, 100)./+  // Sum 1 to 99

// Recursive iteration
count_down(n) = n <= 0 ? "done" : { print(n); $(n - 1) }
```

---

## Pattern Matching

### Match Expression

```vais
result = match value {
    0 => "zero",
    1 => "one",
    n if n < 0 => "negative",
    n if n > 100 => "large: " ++ str(n),
    _ => "other"
}
```

### Tuple Destructuring

```vais
// Destructure tuple in let binding
let (a, b) = (1, 2) : a + b    // 3

// Multiple elements
let (x, y, z) = (10, 20, 30) : x * y * z  // 6000

// Nested in expressions
result = let (first, second) = get_pair() : first + second
```

---

## Modules

### Import

```vais
// Import entire module
use math

// Use functions directly after import
let x = add(1, 2)

// Or use qualified calls
let y = math.mul(3, 4)
```

### Module File Structure

```
project/
├── main.vais
└── lib/
    └── math.vais     // use lib.math
```

### Module Definition (math.vais)

```vais
// lib/math.vais
double(x) = x * 2
triple(x) = x * 3
add(a, b) = a + b
```

### Built-in Functions

```vais
// Math functions are available globally
print(sin(0))       // 0
print(cos(0))       // 1
print(sqrt(16))     // 4
print(abs(-5))      // 5
```

---

## Error Handling

### Try-Catch

```vais
// Basic try-catch
result = try {
    risky_operation()
} catch e {
    "Error: " ++ e
}

// Error propagation
let caught = try {
    err("something went wrong")
} catch e {
    "Caught: " ++ e
}

// Safe execution (returns value if no error)
let safe = try { 42 } catch e { 0 }
```

### Error Function

```vais
// Raise an error
err("error message")

// In conditional
validate(x) = x < 0 ? err("negative not allowed") : x
```

### Validation Pattern

```vais
// Use ternary operator for validation
safe_divide(a, b) = b == 0 ? err("Division by zero") : a / b

// Pattern matching for error handling
handle_result(x) = match x {
    0 => err("zero not allowed"),
    n if n < 0 => err("negative"),
    _ => x * 2
}
```

---

## Async/Concurrency (Planned)

> Note: Full async/await is planned for a future version.

### Async Functions (Planned)

```vais
// Planned syntax:
// async fetch(url) = {
//     response = await http_get(url)
//     response.body
// }
```

### Current Status

The `async` keyword is recognized but full async/await runtime is not yet implemented.

---

## Standard Library

### Available Built-in Functions

```vais
// I/O
print(...)          // Print values
println(...)        // Print with newline

// Math
sqrt(n), pow(a, b), abs(n)
sin(x), cos(x), tan(x)
floor(x), ceil(x), round(x)

// String
len(s), upper(s), lower(s)
split(s, delim), join(arr, delim)
contains(s, sub), replace(s, old, new)
str(value)          // Convert to string

// Array
len(arr), range(start, end)
first(arr), last(arr), reverse(arr)
```
