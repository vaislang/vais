# Language Specification

This page includes the repository's authoritative language specification so the
docs site does not drift from the compiler-facing contract.

## Overview

Vais is a systems programming language designed for:

- **Canonical clarity** - current keyword forms match the promoted lexer surface
- **Type safety** - Strong static typing with full inference
- **Native performance** - LLVM-based compilation to native code
- **Modern features** - Generics, traits, async/await, pattern matching

## Keywords

Vais retired several early single-character forms in Step 19. New code uses the canonical keyword forms below.

| Keyword | Meaning | Example |
|---------|---------|---------|
| `fn` | Function | `fn add(a: i64, b: i64) -> i64 { a + b }` |
| `struct` | Struct | `struct Point { x: f64, y: f64 }` |
| `enum` / `else` | Enum / else branch | `enum Color { Red, Green, Blue }` / `else { fallback }` |
| `I` | If | `I x > 0 { "positive" }` |
| `L` | Loop | `L i := 0; i < 10; i += 1 { ... }` |
| `LF` | Range / foreach loop | `LF i: 0..10 { ... }` |
| `match` | Match | `match x { 1 => "one", _ => "other" }` |
| `return` | Return | `return 42` |
| `B` | Break | `B` |
| `C` | Continue | `C` |
| `trait` | Trait | `trait Printable { fn print(self) }` |
| `impl` | Impl | `impl Point: Printable { ... }` |
| `use` | Use/Import | `use std/io` |
| `pub` | Public | `pub fn public_fn() {}` |
| `type` | Type alias | `type Int = i64` |
| `A` | Async | `A fn fetch() -> str { ... }` |
| `Y` | Await | `result := Y fetch()` |
| `N` | Extern | `N fn malloc(size: i64) -> i64` |
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
fn add(a: i64, b: i64) -> i64 {
    a + b
}
```

### No Return Value

```vais
fn greet(name: str) {
    puts("Hello, ")
    puts(name)
}
```

### Self-Recursion

```vais
fn factorial(n: i64) -> i64 {
    I n <= 1 { return 1 }
    n * @(n - 1)
}
```

### Generic Functions

```vais
fn identity<T>(x: T) -> T {
    x
}

fn max<T>(a: T, b: T) -> T {
    I a > b { a } else { b }
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
} else {
    puts("negative or zero")
}

# If as expression
sign := I x > 0 { 1 } else I x < 0 { -1 } else { 0 }
```

### Match Expression

```vais
match x {
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
struct Point {
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
impl Point {
    fn distance(self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y)
    }

    fn translate(self, dx: f64, dy: f64) -> Point {
        Point { x: self.x + dx, y: self.y + dy }
    }
}
```

## Enums

```vais
# Simple enum
enum Color {
    Red,
    Green,
    Blue
}

# Enum with data
enum Option<T> {
    Some(T),
    None
}

enum Result<T, E> {
    Ok(T),
    Err(E)
}
```

## Pattern Matching

Pattern matching provides exhaustive checking of values:

### Basic Patterns

```vais
# Literal patterns
match x {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "other"  # Wildcard - matches anything
}

# Variable binding
match value {
    x => x * 2  # Binds value to x
}
```

### Enum Patterns

```vais
enum Option<T> { Some(T), None }

fn unwrap_or<T>(opt: Option<T>, default: T) -> T {
    match opt {
        Some(v) => v,    # Destructure and bind inner value
        None => default
    }
}

enum Result<T, E> { Ok(T), Err(E) }

fn handle_result(r: Result<i64, str>) -> i64 {
    match r {
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
match value {
    x if x > 0 => "positive",
    x if x < 0 => "negative",
    _ => "zero"
}

# With enum destructuring
enum Result<T, E> { Ok(T), Err(E) }

match result {
    Ok(val) if val > 100 => "large success",
    Ok(val) => "small success",
    Err(msg) => "error"
}
```

### Pattern Alias

Bind a name to the matched value using `@`:

```vais
fn describe(n: i64) -> str {
    match n {
        x @ 1..10 => "small: {x}",
        x @ 10..100 => "medium: {x}",
        x @ 100..1000 => "large: {x}",
        _ => "very large"
    }
}

# With enum variants
enum Option<T> { None, Some(T) }

fn process(opt: Option<i64>) -> i64 {
    match opt {
        val @ Some(x) => x * 2,  # 'val' is the whole Some, 'x' is inner value
        None => 0
    }
}
```

## Error Handling

### The Try Operator `?`

Propagate errors to the caller:

```vais
enum Result<T, E> { Ok(T), Err(E) }

fn read_file(path: str) -> Result<str, str> {
    file := open(path)?        # If Err, return early
    data := file.read()?       # If Err, return early
    Ok(data)
}

fn process() -> Result<i64, str> {
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
fn main() {
    config := get_config()!  # Panics if None
    puts(config)
}
```

### Error Type Derivation

```vais
#[derive(Error)]
enum AppError {
    NotFound(str),
    Permission(str),
    Network(str)
}

fn find_user(id: i64) -> Result<str, AppError> {
    I id < 0 {
        Err(NotFound("User ID cannot be negative"))
    } else {
        Ok("User data")
    }
}
```

## Traits

Define shared behavior across types.

### Trait Definition

```vais
trait Printable {
    fn print(&self) -> i64
}

trait Comparable {
    fn compare(&self, other: &Self) -> i64
}

trait Shape {
    fn area(&self) -> f64
    fn perimeter(&self) -> f64
}
```

### Trait Implementation

```vais
struct Point { x: f64, y: f64 }

impl Point: Printable {
    fn print(&self) -> i64 {
        puts("Point(")
        print_f64(self.x)
        puts(", ")
        print_f64(self.y)
        puts(")")
        0
    }
}

struct Circle { radius: f64 }

impl Circle: Shape {
    fn area(&self) -> f64 {
        pi := 3.14159
        pi * self.radius * self.radius
    }

    fn perimeter(&self) -> f64 {
        pi := 3.14159
        2.0 * pi * self.radius
    }
}
```

### Method Implementation

Implement methods without a trait:

```vais
struct Counter { value: i64 }

impl Counter {
    fn new() -> Counter {
        Counter { value: 0 }
    }

    fn increment(&mut self) {
        self.value = self.value + 1
    }

    fn get(&self) -> i64 {
        self.value
    }
}

fn main() {
    c := Counter::new()
    c.increment()
    print_i64(c.get())  # Prints: 1
}
```

### Generic Traits

```vais
trait Container<T> {
    fn add(&mut self, item: T)
    fn get(&self, index: i64) -> Option<T>
}

struct Vec<T> {
    items: [type],
    len: i64
}

impl Vec<T>: Container<T> {
    fn add(&mut self, item: T) {
        # Add item implementation
    }

    fn get(&self, index: i64) -> Option<T> {
        I index < 0 || index >= self.len {
            return None
        }
        Some(self.items[index])
    }
}
```

### Trait Bounds

```vais
# Function requiring trait bound
fn print_all<T: Printable>(items: [type]) {
    L item : items {
        item.print()
    }
}

# Multiple trait bounds
fn compare_and_print<T: Comparable + Printable>(a: T, b: T) {
    I a.compare(&b) > 0 {
        a.print()
    } else {
        b.print()
    }
}
```

### Where Clauses

Alternative syntax for complex constraints:

```vais
# Basic where clause
fn find_max<T>(list: Vec<T>) -> T where type: Ord {
    result := mut list.get(0)!
    L i : 1..list.len() {
        I list.get(i)! > result {
            result = list.get(i)!
        }
    }
    result
}

# Multiple constraints
fn process<T, use>(a: T, b: use) -> i64
where
    type: Printable + Clone,
    use: Comparable
{
    a.print()
    b.compare(&b)
}
```

### Trait Alias

Define aliases for trait combinations:

```vais
# Define trait alias
type Drawable = Printable + Shape

# Use in function signatures
fn draw<T: Drawable>(obj: T) {
    obj.print()
    print_f64(obj.area())
}
```

## Generics

Write code that works with any type.

### Generic Functions

```vais
# Basic generic function
fn identity<T>(x: T) -> T {
    x
}

# Multiple type parameters
fn swap<T, use>(a: T, b: use) -> (use, T) {
    (b, a)
}

# Generic trait
trait Container<T> {
    fn get(self) -> T
}
```

## Error Handling

### Option Type

```vais
enum Option<T> { Some(T), None }

fn find(arr: &[i64], target: i64) -> Option<i64> {
    # ... search logic
    Some(index)  # or None
}
```

### Result Type

```vais
enum Result<T, E> { Ok(T), Err(E) }

fn divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 { return Err("division by zero") }
    Ok(a / b)
}
```

### Try Operator `?`

```vais
fn compute() -> Result<i64, str> {
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
use std/io
use std/vec

# Use items from module
v := Vec::new()
content := read_file("data.txt")
```

## Comments

```vais
# Single-line comment

fn main() {
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
fn fib(n: i64) -> i64 {
    I n <= 1 { return n }
    @(n-1) + @(n-2)
}
```

### Linked List

```vais
struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>
}

impl Node<T> {
    fn new(value: T) -> Node<T> {
        Node { value: value, next: None }
    }
}
```

### Error Handling

```vais
fn parse_number(s: str) -> Result<i64, str> {
    # Parsing logic
    I is_valid {
        Ok(number)
    } else {
        Err("Invalid number")
    }
}
```

## Async/Await

Vais supports asynchronous programming with the `A` (async) and `Y` (await/yield) keywords.

### Async Functions

```vais
# Mark function as async with 'A'
A fn fetch_data(id: i64) -> str {
    # Async operation
    "Data loaded"
}

A fn process_data(data: str) -> i64 {
    # Process data
    42
}
```

### Awaiting Results

```vais
fn main() {
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
A fn task1() -> i64 {
    puts("Task 1 running")
    100
}

A fn task2() -> i64 {
    puts("Task 2 running")
    200
}

fn main() {
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
fn main() {
    # Ownership transfer (move)
    s1 := "Hello"
    s2 := s1         # s1 is moved to s2
    # puts(s1)       # Error: s1 no longer valid
}
```

### Borrowing

```vais
fn print_value(x: &i64) {
    print_i64(*x)
}

fn main() {
    n := 42
    print_value(&n)  # Borrow n
    print_i64(n)     # n still valid
}
```

### Mutable References

```vais
fn increment(x: &mut i64) {
    *x = *x + 1
}

fn main() {
    n := mut 5
    increment(&mut n)
    print_i64(n)     # Prints: 6
}
```

### Lifetimes

Specify explicit lifetimes for references:

```vais
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    I x.len() > y.len() { x } else { y }
}
```

## Slice Types

Work with contiguous sequences of elements.

### Immutable Slices

```vais
fn sum(s: &[i64]) -> i64 {
    total := mut 0
    L i := 0; i < s.len(); i += 1 {
        total = total + s[i]
    }
    total
}

fn main() {
    arr := [1, 2, 3, 4, 5]
    slice := &arr[1..4]  # [2, 3, 4]
    result := sum(slice)
    print_i64(result)    # Prints: 9
}
```

### Mutable Slices

```vais
fn double_elements(s: &mut [i64]) {
    L i := 0; i < s.len(); i += 1 {
        s[i] = s[i] * 2
    }
}

fn main() {
    arr := mut [10, 20, 30, 40]
    mut_slice := &mut arr[0..2]
    double_elements(mut_slice)
    # arr is now [20, 40, 30, 40]
}
```

### Slice Operations

```vais
fn main() {
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
fn main() {
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
fn main() {
    multiplier := 10

    # Closure captures 'multiplier' from environment
    scale := |x| x * multiplier

    result := scale(5)       # 50
}
```

### Higher-Order Functions

```vais
fn apply<T>(f: fn(T) -> T, x: T) -> T {
    f(x)
}

fn main() {
    double := |x| x * 2
    result := apply(double, 21)  # 42
}
```

## Defer Statement

Execute code when function exits:

```vais
use std/io

fn process_file(filename: str) -> Result<i64, str> {
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
N fn malloc(size: i64) -> i64
N fn free(ptr: i64)
N fn printf(format: str, ...) -> i64

fn main() {
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
trait Iterator<T> {
    fn next(&mut self) -> Option<T>
}

struct Counter {
    current: i64,
    max: i64
}

impl Counter: Iterator<i64> {
    fn next(&mut self) -> Option<i64> {
        I self.current >= self.max {
            return None
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
struct Point { x: f64, y: f64 }

# Derive error trait
#[derive(Error)]
enum MyError {
    NotFound(str),
    Invalid(str)
}

# Conditional compilation
#[cfg(target_os = "linux")]
fn platform_specific() {
    puts("Running on Linux")
}

# WASM imports/exports
#[wasm_import("env", "log")]
N fn js_log(msg: str)

#[wasm_export("add")]
fn add(a: i64, b: i64) -> i64 {
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
fn main() {
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
- [Playground](https://vaislang.dev/playground/) - Try Vais online
