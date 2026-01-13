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
#{1, 2, 3}          // Set
{"a": 1, "b": 2}    // Map
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
// Full syntax
double = (x) => x * 2

// Pipe lambda syntax
add = |x, y| x + y

// Shorthand with _ (placeholder)
numbers.@(_ * 2)      // Same as: numbers.@(x => x * 2)
numbers.?(_ > 0)      // Same as: numbers.?(x => x > 0)
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

### Pipeline

```vais
// |> passes left side as argument to right side
data |> process |> format |> output

// Equivalent to:
output(format(process(data)))
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

### Maps

```vais
m = {"name": "Vais", "version": "0.1"}

// Access
m["name"]           // "Vais"
m.name              // "Vais" (dot notation)

// Operations
keys(m)             // ["name", "version"]
values(m)           // ["Vais", "0.1"]
```

---

## Control Flow

### Conditionals

```vais
// Ternary operator
result = condition ? value_if_true : value_if_false

// If-then-else
result = if x > 0 then "positive" else "non-positive"

// If block
result = if x > 0 {
    "positive"
} else if x < 0 {
    "negative"
} else {
    "zero"
}
```

### Loops

```vais
// For loop
for i in range(1, 10) {
    print(i)
}

// For with collection
for item in items {
    process(item)
}

// While loop
while condition {
    do_something()
}
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

### Destructuring

```vais
// Tuple destructuring
(a, b) = (1, 2)

// Array destructuring
[first, second, ...rest] = [1, 2, 3, 4, 5]

// Struct destructuring
{name, age} = person
```

---

## Modules

### Import

```vais
// Import entire module
use math

// Import specific items
use math::{sin, cos, PI}

// Import with alias
use math as m
```

### Export

```vais
// Public function
pub add(a, b) = a + b

// Private (default)
helper(x) = x * 2
```

---

## Error Handling

### Try-Catch

```vais
result = try {
    risky_operation()
} catch e {
    handle_error(e)
}
```

### Result Type

```vais
safe_divide(a, b) = 
    if b == 0 
    then Err("Division by zero") 
    else Ok(a / b)

// Using the result
match safe_divide(10, 2) {
    Ok(value) => print(value),
    Err(msg) => print("Error: " ++ msg)
}
```

### Optional Chaining

```vais
// ? operator for early return on error
value = get_data()?
processed = process(value)?
result = format(processed)?
```

---

## Async/Concurrency

### Async Functions

```vais
async fetch(url) = {
    response = await http_get(url)
    response.body
}
```

### Channels

```vais
// Create channel
ch = channel(10)  // buffered channel

// Send
ch <- value

// Receive
value = <- ch
```

### Spawn

```vais
// Spawn concurrent task
handle = spawn async_task(args)

// Wait for result
result = await handle
```

### Parallel Operations

```vais
// Parallel map
results = data.||@(expensive_operation)

// Parallel filter
filtered = data.||?(slow_predicate)
```

---

## Standard Library

See [STDLIB.md](./STDLIB.md) for the complete standard library reference.
