# Language Specification

Complete reference for the Vais programming language.

## Overview

Vais is a systems programming language designed for:

- **Token efficiency** - Single-character keywords minimize AI token usage
- **Type safety** - Strong static typing with full inference
- **Native performance** - LLVM-based compilation to native code
- **Modern features** - Generics, traits, async/await, pattern matching

## Keywords

Vais uses single-character keywords for maximum efficiency:

| Keyword | Meaning | Example |
|---------|---------|---------|
| `F` | Function | `F add(a: i64, b: i64) -> i64 { a + b }` |
| `S` | Struct | `S Point { x: f64, y: f64 }` |
| `E` | Enum/Else | `E Color { Red, Green, Blue }` / `E { fallback }` |
| `I` | If | `I x > 0 { "positive" }` |
| `L` | Loop | `L i := 0; i < 10; i += 1 { ... }` |
| `M` | Match | `M x { 1 => "one", _ => "other" }` |
| `R` | Return | `R 42` |
| `B` | Break | `B` |
| `C` | Continue | `C` |
| `W` | Trait | `W Printable { F print(self) }` |
| `X` | Impl | `X Point: Printable { ... }` |
| `U` | Use/Import | `U std/io` |
| `P` | Public | `P F public_fn() {}` |
| `T` | Type alias | `T Int = i64` |
| `A` | Async | `A F fetch() -> str { ... }` |
| `Y` | Await | `result := Y fetch()` |
| `N` | Extern | `N F malloc(size: i64) -> i64` |
| `G` | Global | `G counter: i64 = 0` |
| `D` | Defer | `D cleanup()` |
| `O` | Union | `O Data { i: i64, f: f64 }` |

## Operators

### Special Operators

| Operator | Meaning | Example |
|----------|---------|---------|
| `@` | Self-recursion | `@(n-1) + @(n-2)` |
| `:=` | Variable binding | `x := 5` |
| `:= mut` | Mutable binding | `x := mut 0` |
| `?` | Try (error propagation) | `result?` |
| `!` | Unwrap | `result!` |
| `\|>` | Pipe | `x \|> f \|> g` |
| `..` | Range | `1..10` |

### Arithmetic Operators

```vais
+ - * / %        # Basic arithmetic
+= -= *= /= %=   # Compound assignment
```

### Comparison Operators

```vais
== != < > <= >=
```

### Logical Operators

```vais
&& ||  # Logical AND, OR
!      # Logical NOT
```

### Bitwise Operators

```vais
& | ^ << >>      # AND, OR, XOR, left shift, right shift
```

## Types

### Primitive Types

```vais
# Integers
i8 i16 i32 i64 i128
u8 u16 u32 u64 u128

# Floating point
f32 f64

# Boolean
bool

# String
str
```

### Compound Types

```vais
# Array
[i64; 10]         # Fixed-size array of 10 i64s

# Slice
&[i64]            # Immutable slice
&mut [i64]        # Mutable slice

# Tuple
(i64, f64, str)

# Pointer
*i64              # Raw pointer
```

### Generic Types

```vais
Vec<T>            # Generic vector
HashMap<K, V>     # Generic hash map
Option<T>         # Optional value
Result<T, E>      # Result with error
```

## Variable Declaration

```vais
# Type inference
x := 42                 # i64 inferred
y := 3.14               # f64 inferred

# Explicit type
count: i64 = 100

# Mutable
counter := mut 0
counter = counter + 1

# Multiple declarations
a := 1
b := 2
c := 3
```

## Functions

### Basic Function

```vais
F add(a: i64, b: i64) -> i64 {
    a + b
}
```

### No Return Value

```vais
F greet(name: str) {
    puts("Hello, ")
    puts(name)
}
```

### Self-Recursion

```vais
F factorial(n: i64) -> i64 {
    I n <= 1 { R 1 }
    n * @(n - 1)
}
```

### Generic Functions

```vais
F identity<T>(x: T) -> T {
    x
}

F max<T>(a: T, b: T) -> T {
    I a > b { a } E { b }
}
```

## Control Flow

### If Expression

```vais
# Simple if
I x > 0 {
    puts("positive")
}

# If-else
I x > 0 {
    puts("positive")
} E {
    puts("negative or zero")
}

# If as expression
sign := I x > 0 { 1 } E I x < 0 { -1 } E { 0 }
```

### Match Expression

```vais
M x {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "many"
}
```

### Loops

```vais
# C-style loop
L i := 0; i < 10; i += 1 {
    print_i64(i)
}

# Infinite loop
L {
    I should_break { B }
}

# Loop with break and continue
L i := 0; i < 20; i += 1 {
    I i % 2 == 0 { C }  # Skip even numbers
    I i > 15 { B }      # Break at 15
    print_i64(i)
}
```

## Structs

```vais
# Define struct
S Point {
    x: f64,
    y: f64
}

# Create instance
p := Point { x: 3.0, y: 4.0 }

# Access fields
x_coord := p.x
```

### Methods

```vais
X Point {
    F distance(self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y)
    }

    F translate(self, dx: f64, dy: f64) -> Point {
        Point { x: self.x + dx, y: self.y + dy }
    }
}
```

## Enums

```vais
# Simple enum
E Color {
    Red,
    Green,
    Blue
}

# Enum with data
E Option<T> {
    Some(T),
    None
}

E Result<T, E> {
    Ok(T),
    Err(E)
}
```

## Pattern Matching

```vais
E Option<T> { Some(T), None }

F unwrap_or<T>(opt: Option<T>, default: T) -> T {
    M opt {
        Some(v) => v,
        None => default
    }
}
```

## Traits

```vais
# Define trait
W Printable {
    F print(self)
}

# Implement trait
S Person { name: str, age: i64 }

X Person: Printable {
    F print(self) {
        puts(self.name)
    }
}
```

## Generics

```vais
# Generic struct
S Box<T> {
    value: T
}

# Generic function
F swap<T>(a: T, b: T) -> (T, T) {
    (b, a)
}

# Generic trait
W Container<T> {
    F get(self) -> T
}
```

## Error Handling

### Option Type

```vais
E Option<T> { Some(T), None }

F find(arr: &[i64], target: i64) -> Option<i64> {
    # ... search logic
    Some(index)  # or None
}
```

### Result Type

```vais
E Result<T, E> { Ok(T), Err(E) }

F divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 { R Err("division by zero") }
    Ok(a / b)
}
```

### Try Operator `?`

```vais
F compute() -> Result<i64, str> {
    a := divide(10, 2)?   # Propagate error
    b := divide(a, 3)?
    Ok(b)
}
```

### Unwrap Operator `!`

```vais
result := divide(10, 2)
value := result!  # Panics if Err
```

## Module System

```vais
# Import module
U std/io
U std/vec

# Use items from module
v := Vec::new()
content := read_file("data.txt")
```

## Comments

```vais
# Single-line comment

F main() {
    x := 42  # Inline comment
}
```

## String Interpolation

```vais
name := "Alice"
age := 30

# Variable interpolation (NOT SUPPORTED - use puts)
puts("Name: ")
puts(name)

# Concatenation
msg := "Hello, " + name
```

## Built-in Functions

```vais
# I/O
puts(s: str)              # Print string
print_i64(x: i64)         # Print integer
print_f64(x: f64)         # Print float

# Memory
malloc(size: i64) -> i64  # Allocate memory
free(ptr: i64)            # Free memory

# Type operations
sizeof(T) -> i64          # Size of type
```

## Best Practices

1. **Use type inference** when the type is obvious
2. **Use explicit types** for function parameters and return values
3. **Prefer expressions** over statements (use `I` instead of `if` statements)
4. **Use `@` for recursion** instead of function name
5. **Handle errors** with `Result` and `?` operator
6. **Use pattern matching** with `M` for complex conditionals
7. **Keep functions small** and focused on single responsibility

## Examples

### Fibonacci

```vais
F fib(n: i64) -> i64 {
    I n <= 1 { R n }
    @(n-1) + @(n-2)
}
```

### Linked List

```vais
S Node<T> {
    value: T,
    next: Option<Box<Node<T>>>
}

X Node<T> {
    F new(value: T) -> Node<T> {
        Node { value: value, next: None }
    }
}
```

### Error Handling

```vais
F parse_number(s: str) -> Result<i64, str> {
    # Parsing logic
    I is_valid {
        Ok(number)
    } E {
        Err("Invalid number")
    }
}
```

## Learn More

- [Tutorial](../getting-started/tutorial.md) - Step-by-step guide
- [Standard Library](https://github.com/vaislang/vais/tree/main/std) - Built-in modules
- [Examples](https://github.com/vaislang/vais/tree/main/examples) - Code samples
