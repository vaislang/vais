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

Pattern matching provides exhaustive checking of values:

### Basic Patterns

```vais
# Literal patterns
M x {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "other"  # Wildcard - matches anything
}

# Variable binding
M value {
    x => x * 2  # Binds value to x
}
```

### Enum Patterns

```vais
E Option<T> { Some(T), None }

F unwrap_or<T>(opt: Option<T>, default: T) -> T {
    M opt {
        Some(v) => v,    # Destructure and bind inner value
        None => default
    }
}

E Result<T, E> { Ok(T), Err(E) }

F handle_result(r: Result<i64, str>) -> i64 {
    M r {
        Ok(val) => val,
        Err(msg) => {
            puts("Error: ")
            puts(msg)
            0
        }
    }
}
```

### Pattern Guards

Add additional conditions to patterns:

```vais
M value {
    x if x > 0 => "positive",
    x if x < 0 => "negative",
    _ => "zero"
}

# With enum destructuring
E Result<T, E> { Ok(T), Err(E) }

M result {
    Ok(val) if val > 100 => "large success",
    Ok(val) => "small success",
    Err(msg) => "error"
}
```

### Pattern Alias

Bind a name to the matched value using `@`:

```vais
F describe(n: i64) -> str {
    M n {
        x @ 1..10 => "small: ~{x}",
        x @ 10..100 => "medium: ~{x}",
        x @ 100..1000 => "large: ~{x}",
        _ => "very large"
    }
}

# With enum variants
E Option<T> { None, Some(T) }

F process(opt: Option<i64>) -> i64 {
    M opt {
        val @ Some(x) => x * 2,  # 'val' is the whole Some, 'x' is inner value
        None => 0
    }
}
```

## Error Handling

### The Try Operator `?`

Propagate errors to the caller:

```vais
E Result<T, E> { Ok(T), Err(E) }

F read_file(path: str) -> Result<str, str> {
    file := open(path)?        # If Err, return early
    data := file.read()?       # If Err, return early
    Ok(data)
}

F process() -> Result<i64, str> {
    content := read_file("config.txt")?  # Propagates error
    Ok(parse(content))
}
```

### The Unwrap Operator `!`

Extract value or panic:

```vais
# Unwrap Option - panics if None
value := some_option!

# Unwrap Result - panics if Err
data := some_result!

# Example
F main() {
    config := get_config()!  # Panics if None
    puts(config)
}
```

### Error Type Derivation

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

## Traits

Define shared behavior across types.

### Trait Definition

```vais
W Printable {
    F print(&self) -> i64
}

W Comparable {
    F compare(&self, other: &Self) -> i64
}

W Shape {
    F area(&self) -> f64
    F perimeter(&self) -> f64
}
```

### Trait Implementation

```vais
S Point { x: f64, y: f64 }

X Point: Printable {
    F print(&self) -> i64 {
        puts("Point(")
        print_f64(self.x)
        puts(", ")
        print_f64(self.y)
        puts(")")
        0
    }
}

S Circle { radius: f64 }

X Circle: Shape {
    F area(&self) -> f64 {
        pi := 3.14159
        pi * self.radius * self.radius
    }

    F perimeter(&self) -> f64 {
        pi := 3.14159
        2.0 * pi * self.radius
    }
}
```

### Method Implementation

Implement methods without a trait:

```vais
S Counter { value: i64 }

X Counter {
    F new() -> Counter {
        Counter { value: 0 }
    }

    F increment(&mut self) {
        self.value = self.value + 1
    }

    F get(&self) -> i64 {
        self.value
    }
}

F main() {
    c := Counter::new()
    c.increment()
    print_i64(c.get())  # Prints: 1
}
```

### Generic Traits

```vais
W Container<T> {
    F add(&mut self, item: T)
    F get(&self, index: i64) -> Option<T>
}

S Vec<T> {
    items: [T],
    len: i64
}

X Vec<T>: Container<T> {
    F add(&mut self, item: T) {
        # Add item implementation
    }

    F get(&self, index: i64) -> Option<T> {
        I index < 0 || index >= self.len {
            R None
        }
        Some(self.items[index])
    }
}
```

### Trait Bounds

```vais
# Function requiring trait bound
F print_all<T: Printable>(items: [T]) {
    L item : items {
        item.print()
    }
}

# Multiple trait bounds
F compare_and_print<T: Comparable + Printable>(a: T, b: T) {
    I a.compare(&b) > 0 {
        a.print()
    } E {
        b.print()
    }
}
```

### Where Clauses

Alternative syntax for complex constraints:

```vais
# Basic where clause
F find_max<T>(list: Vec<T>) -> T where T: Ord {
    result := mut list.get(0)!
    L i : 1..list.len() {
        I list.get(i)! > result {
            result = list.get(i)!
        }
    }
    result
}

# Multiple constraints
F process<T, U>(a: T, b: U) -> i64
where
    T: Printable + Clone,
    U: Comparable
{
    a.print()
    b.compare(&b)
}
```

### Trait Alias

Define aliases for trait combinations:

```vais
# Define trait alias
T Drawable = Printable + Shape

# Use in function signatures
F draw<T: Drawable>(obj: T) {
    obj.print()
    print_f64(obj.area())
}
```

## Generics

Write code that works with any type.

### Generic Functions

```vais
# Basic generic function
F identity<T>(x: T) -> T {
    x
}

# Multiple type parameters
F swap<T, U>(a: T, b: U) -> (U, T) {
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

## Async/Await

Vais supports asynchronous programming with the `A` (async) and `Y` (await/yield) keywords.

### Async Functions

```vais
# Mark function as async with 'A'
A F fetch_data(id: i64) -> str {
    # Async operation
    "Data loaded"
}

A F process_data(data: str) -> i64 {
    # Process data
    42
}
```

### Awaiting Results

```vais
F main() {
    # Call async function and await the result
    data := fetch_data(1).await
    puts(data)

    # Chain async calls
    result := process_data(data).await
    print_i64(result)
}
```

### Spawning Concurrent Tasks

```vais
A F task1() -> i64 {
    puts("Task 1 running")
    100
}

A F task2() -> i64 {
    puts("Task 2 running")
    200
}

F main() {
    # Spawn tasks to run concurrently
    t1 := spawn task1()
    t2 := spawn task2()

    # Await results
    r1 := t1.await
    r2 := t2.await

    total := r1 + r2
    print_i64(total)  # 300
}
```

## Memory Management

### Ownership

Vais uses an ownership system for memory safety:

```vais
F main() {
    # Ownership transfer (move)
    s1 := "Hello"
    s2 := s1         # s1 is moved to s2
    # puts(s1)       # Error: s1 no longer valid
}
```

### Borrowing

```vais
F print_value(x: &i64) {
    print_i64(*x)
}

F main() {
    n := 42
    print_value(&n)  # Borrow n
    print_i64(n)     # n still valid
}
```

### Mutable References

```vais
F increment(x: &mut i64) {
    *x = *x + 1
}

F main() {
    n := mut 5
    increment(&mut n)
    print_i64(n)     # Prints: 6
}
```

### Lifetimes

Specify explicit lifetimes for references:

```vais
F longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    I x.len() > y.len() { x } E { y }
}
```

## Slice Types

Work with contiguous sequences of elements.

### Immutable Slices

```vais
F sum(s: &[i64]) -> i64 {
    total := mut 0
    L i := 0; i < s.len(); i += 1 {
        total = total + s[i]
    }
    total
}

F main() {
    arr := [1, 2, 3, 4, 5]
    slice := &arr[1..4]  # [2, 3, 4]
    result := sum(slice)
    print_i64(result)    # Prints: 9
}
```

### Mutable Slices

```vais
F double_elements(s: &mut [i64]) {
    L i := 0; i < s.len(); i += 1 {
        s[i] = s[i] * 2
    }
}

F main() {
    arr := mut [10, 20, 30, 40]
    mut_slice := &mut arr[0..2]
    double_elements(mut_slice)
    # arr is now [20, 40, 30, 40]
}
```

### Slice Operations

```vais
F main() {
    arr := [1, 2, 3, 4, 5]

    # Full slice
    all := &arr[..]

    # From index to end
    tail := &arr[2..]   # [3, 4, 5]

    # From start to index
    head := &arr[..3]   # [1, 2, 3]

    # Length
    len := arr.len()
}
```

## Closures and Lambdas

### Basic Closures

```vais
F main() {
    # Simple closure
    add_one := |x| x + 1
    result := add_one(5)     # 6

    # Multi-parameter closure
    multiply := |x, y| x * y
    product := multiply(3, 4)  # 12

    # Closure with body
    complex := |x| {
        temp := x * 2
        temp + 1
    }
}
```

### Closure Capture

```vais
F main() {
    multiplier := 10

    # Closure captures 'multiplier' from environment
    scale := |x| x * multiplier

    result := scale(5)       # 50
}
```

### Higher-Order Functions

```vais
F apply<T>(f: fn(T) -> T, x: T) -> T {
    f(x)
}

F main() {
    double := |x| x * 2
    result := apply(double, 21)  # 42
}
```

## Defer Statement

Execute code when function exits:

```vais
U std/io

F process_file(filename: str) -> Result<i64, str> {
    file := open_file(filename)?
    D close_file(file)  # Deferred - runs when function exits

    # Process file
    data := read_from_file(file)?
    Ok(data.length())
}
```

## Extern Functions

Declare and call foreign functions:

```vais
# Declare external C functions
N F malloc(size: i64) -> i64
N F free(ptr: i64)
N F printf(format: str, ...) -> i64

F main() {
    # Allocate memory
    ptr := malloc(100)

    # Use memory
    printf("Allocated at: %p\n", ptr)

    # Free memory
    free(ptr)
}
```

## Iterators and Ranges

### Range Loops

```vais
# Exclusive range (0..10 means 0 to 9)
L i : 0..10 {
    print_i64(i)
}

# Inclusive range (0..=10 means 0 to 10)
L i : 0..=10 {
    print_i64(i)
}

# Iterate over array
arr := [1, 2, 3, 4, 5]
L x : arr {
    print_i64(x)
}
```

### Iterator Traits

```vais
W Iterator<T> {
    F next(&mut self) -> Option<T>
}

S Counter {
    current: i64,
    max: i64
}

X Counter: Iterator<i64> {
    F next(&mut self) -> Option<i64> {
        I self.current >= self.max {
            R None
        }
        result := self.current
        self.current = self.current + 1
        Some(result)
    }
}
```

## Attributes

Annotate items with metadata:

### Common Attributes

```vais
# Derive trait implementations
#[derive(Clone, Debug)]
S Point { x: f64, y: f64 }

# Derive error trait
#[derive(Error)]
E MyError {
    NotFound(str),
    Invalid(str)
}

# Conditional compilation
#[cfg(target_os = "linux")]
F platform_specific() {
    puts("Running on Linux")
}

# WASM imports/exports
#[wasm_import("env", "log")]
N F js_log(msg: str)

#[wasm_export("add")]
F add(a: i64, b: i64) -> i64 {
    a + b
}
```

## Macros

Declarative macros for code generation:

```vais
# Define a macro
macro_rules! vec {
    ($($x:expr),*) => {
        {
            v := Vec::new()
            $(
                v.push($x)
            )*
            v
        }
    }
}

# Use the macro
F main() {
    v := vec!(1, 2, 3, 4, 5)
    print_i64(v.len())  # Prints: 5
}
```

## Operator Precedence

Operators listed from highest to lowest precedence:

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

**Note:** Bitwise `&` has higher precedence than comparison operators. Use parentheses: `(a == b) & (c == d)`.

## Compilation Targets

### Native Binary

```bash
vaisc build program.vais -o program
./program
```

### JavaScript (ESM)

```bash
vaisc build --target js program.vais -o program.mjs
node program.mjs
```

### WebAssembly

```bash
vaisc build --target wasm32-unknown-unknown program.vais -o program.wasm
# Use with JavaScript runtime
```

## Learn More

- [Tutorial](../getting-started/tutorial.md) - Step-by-step guide from basics to advanced
- [Standard Library](https://github.com/vaislang/vais/tree/main/std) - 74 built-in modules
- [Examples](https://github.com/vaislang/vais/tree/main/examples) - 189 real-world code samples
- [Memory Safety](../advanced/memory-safety.md) - Deep dive into ownership and borrowing
- [Playground](https://vais.dev/playground/) - Try Vais online
