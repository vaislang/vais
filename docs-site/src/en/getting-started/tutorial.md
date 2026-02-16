# Tutorial

This tutorial will guide you through programming in Vais, from basic concepts to advanced features.

## Hello World

Let's start with the classic first program:

```vais
F main() {
    puts("Hello, Vais!")
}
```

**Key points:**
- `F` declares a function
- `main` is the entry point
- `puts` prints a string to stdout

## Variables and Types

### Type Inference

Use `:=` for automatic type inference:

```vais
F main() {
    x := 10          # i64 inferred
    y := 3.14        # f64 inferred
    name := "Alice"  # str inferred
    flag := true     # bool inferred
}
```

### Explicit Types

Specify types when needed:

```vais
F main() {
    count: i64 = 100
    price: f64 = 19.99
    is_active: bool = true
}
```

### Mutable Variables

Use `mut` for variables that can be reassigned:

```vais
F main() {
    x := mut 0
    x = 10  # OK: x is mutable
    x = 20  # OK
}
```

## Functions

### Basic Functions

```vais
F add(a: i64, b: i64) -> i64 {
    a + b  # Last expression is return value
}

F greet(name: str) {
    puts("Hello, ")
    puts(name)
}
```

### Early Return

Use `R` to return early:

```vais
F abs(x: i64) -> i64 {
    I x < 0 { R -x }
    x
}
```

### Self-Recursion

Use `@` to call the current function:

```vais
F factorial(n: i64) -> i64 {
    I n <= 1 { R 1 }
    n * @(n - 1)
}

F fibonacci(n: i64) -> i64 {
    I n <= 1 { R n }
    @(n-1) + @(n-2)
}
```

## Control Flow

### If Expressions

Everything in Vais is an expression:

```vais
F main() {
    x := 10

    # If returns a value
    result := I x > 5 { "big" } E { "small" }
    puts(result)  # Prints: big
}
```

### Loops

```vais
F main() {
    # C-style loop
    L i := 0; i < 10; i += 1 {
        print_i64(i)
    }

    # Infinite loop with break
    counter := mut 0
    L {
        counter = counter + 1
        I counter >= 5 { B }
    }
}
```

## Structs

Define custom data types:

```vais
S Point {
    x: f64,
    y: f64
}

F main() {
    p := Point { x: 3.0, y: 4.0 }
    puts("Point created")
}
```

### Methods

Implement methods for structs:

```vais
S Point { x: f64, y: f64 }

X Point {
    F distance(self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y)
    }
}

F main() {
    p := Point { x: 3.0, y: 4.0 }
    d := p.distance()
    print_f64(d)  # Prints: 5.0
}
```

## Enums

Define variant types:

```vais
E Color {
    Red,
    Green,
    Blue
}

E Option<T> {
    Some(T),
    None
}
```

## Pattern Matching

Use `M` to match patterns:

```vais
E Color { Red, Green, Blue }

F color_name(c: Color) -> str {
    M c {
        Red => "red",
        Green => "green",
        Blue => "blue"
    }
}

F main() {
    c := Red
    puts(color_name(c))  # Prints: red
}
```

## Error Handling

### Result and Option Types

```vais
E Result<T, E> {
    Ok(T),
    Err(E)
}

F divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 { R Err("Division by zero") }
    Ok(a / b)
}
```

### Try Operator

Use `?` to propagate errors:

```vais
F compute() -> Result<i64, str> {
    x := divide(10, 2)?  # Propagates error if Err
    y := divide(x, 0)?   # Will return Err here
    Ok(y)
}
```

### Unwrap Operator

Use `!` to unwrap or panic:

```vais
F main() {
    result := divide(10, 2)
    value := result!  # Unwraps Ok value, panics on Err
    print_i64(value)
}
```

## Generics

Write code that works with any type:

```vais
F identity<T>(x: T) -> T {
    x
}

S Box<T> {
    value: T
}

F main() {
    x := identity(42)      # T = i64
    y := identity(3.14)    # T = f64

    b := Box { value: 100 }
}
```

## Traits

Define shared behavior:

```vais
W Printable {
    F print(self)
}

S Point { x: f64, y: f64 }

X Point: Printable {
    F print(self) {
        puts("Point(")
        print_f64(self.x)
        puts(", ")
        print_f64(self.y)
        puts(")")
    }
}
```

## Standard Library

### Collections

```vais
U std/vec
U std/hashmap

F main() {
    # Vector
    v := Vec::new()
    v.push(1)
    v.push(2)
    v.push(3)

    # HashMap
    m := HashMap::new()
    m.insert("name", "Alice")
    m.insert("city", "Paris")
}
```

### File I/O

```vais
U std/io

F main() {
    # Read file
    content := read_file("data.txt")
    puts(content)

    # Write file
    write_file("output.txt", "Hello, file!")
}
```

## Advanced Collections

### Working with Vectors

```vais
U std/vec

F main() {
    # Create vector
    v := Vec::new()
    v.push(1)
    v.push(2)
    v.push(3)

    # Access elements
    first := v.get(0)   # Returns Option<i64>
    M first {
        Some(x) => print_i64(x),
        None => puts("Empty")
    }

    # Iterate
    L i := 0; i < v.len(); i += 1 {
        val := v.get(i)!
        print_i64(val)
    }

    # Pop elements
    last := v.pop()     # Returns Option<i64>
}
```

### Working with HashMaps

```vais
U std/hashmap

F main() {
    # Create HashMap
    m := HashMap::new()
    m.insert("name", "Alice")
    m.insert("city", "Paris")
    m.insert("country", "France")

    # Get values
    name := m.get("name")
    M name {
        Some(v) => puts(v),
        None => puts("Not found")
    }

    # Check if key exists
    has_age := m.contains_key("age")  # false

    # Remove entries
    removed := m.remove("city")
}
```

## Memory Management

### Ownership and Borrowing

Vais uses an ownership system similar to Rust for memory safety:

```vais
F main() {
    # Ownership transfer (move)
    s1 := "Hello"
    s2 := s1         # s1 is moved to s2
    # puts(s1)       # Error: s1 no longer valid

    # Borrowing (reference)
    x := 10
    y := &x          # y borrows x
    print_i64(*y)    # Dereference to access value
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

## Error Handling in Depth

### Result Type Chaining

```vais
E Result<T, E> { Ok(T), Err(E) }

F parse_number(s: str) -> Result<i64, str> {
    # Simplified parsing
    I s == "42" { R Ok(42) }
    Err("Invalid number")
}

F double_number(s: str) -> Result<i64, str> {
    n := parse_number(s)?   # Propagates error
    Ok(n * 2)
}

F main() {
    result := double_number("42")
    M result {
        Ok(n) => print_i64(n),      # Prints: 84
        Err(msg) => puts(msg)
    }
}
```

### Option Type Methods

```vais
E Option<T> { Some(T), None }

F main() {
    x := Some(10)
    y := None

    # unwrap_or - provide default value
    v1 := x.unwrap_or(0)    # 10
    v2 := y.unwrap_or(0)    # 0

    # is_some / is_none
    I x.is_some() { puts("Has value") }
    I y.is_none() { puts("No value") }

    # map - transform inner value
    doubled := x.map(|v| v * 2)  # Some(20)
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

## Async Programming

### Async Functions

```vais
# Mark function as async with 'A'
A F fetch_data(id: i64) -> str {
    # Simulate async operation
    "Data loaded"
}

A F process_data(data: str) -> i64 {
    # Process data
    42
}

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

## Working with Strings

### String Operations

```vais
U std/string

F main() {
    # String concatenation
    s1 := "Hello, "
    s2 := "World!"
    s3 := string_concat(s1, s2)

    # String length
    len := string_length(s3)

    # String interpolation
    name := "Alice"
    age := 25
    msg := "Name: ~{name}, Age: ~{age}"
    puts(msg)  # Prints: Name: Alice, Age: 25
}
```

## File I/O

### Reading Files

```vais
U std/io

F main() {
    # Read entire file
    content := read_file("data.txt")
    M content {
        Ok(data) => puts(data),
        Err(msg) => {
            puts("Error: ")
            puts(msg)
        }
    }

    # Read file line by line
    lines := read_lines("data.txt")
    L line : lines {
        puts(line)
    }
}
```

### Writing Files

```vais
U std/io

F main() {
    # Write to file
    result := write_file("output.txt", "Hello, file!")
    M result {
        Ok(_) => puts("File written successfully"),
        Err(msg) => puts(msg)
    }

    # Append to file
    append_result := append_file("output.txt", "\nNew line")
}
```

## Range-Based Loops

### Inclusive and Exclusive Ranges

```vais
F main() {
    # Exclusive range (0..10 means 0 to 9)
    L i : 0..10 {
        print_i64(i)  # Prints: 0 1 2 3 4 5 6 7 8 9
    }

    # Inclusive range (0..=10 means 0 to 10)
    L i : 0..=10 {
        print_i64(i)  # Prints: 0 1 2 3 4 5 6 7 8 9 10
    }

    # Open-ended range
    arr := [1, 2, 3, 4, 5]
    L x : arr[2..] {  # From index 2 to end
        print_i64(x)  # Prints: 3 4 5
    }
}
```

## Slice Types

### Working with Slices

```vais
F print_slice(s: &[i64]) {
    L i := 0; i < s.len(); i += 1 {
        print_i64(s[i])
    }
}

F main() {
    arr := [1, 2, 3, 4, 5]

    # Immutable slice
    slice := &arr[1..4]  # [2, 3, 4]
    print_slice(slice)

    # Mutable slice
    mut_arr := mut [10, 20, 30, 40]
    mut_slice := &mut mut_arr[0..2]
    mut_slice[0] = 99
}
```

## Defer Statement

### Cleanup with Defer

```vais
U std/io

F process_file(filename: str) -> Result<i64, str> {
    file := open_file(filename)?
    D close_file(file)  # Deferred - runs when function exits

    # Process file
    data := read_from_file(file)?
    Ok(data.length())
}

F main() {
    result := process_file("data.txt")
    M result {
        Ok(len) => print_i64(len),
        Err(msg) => puts(msg)
    }
}
```

## Extern Functions

### Calling C Functions

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

## Practical Examples

### Building a Simple Calculator

```vais
U std/io

F add(a: i64, b: i64) -> i64 = a + b
F subtract(a: i64, b: i64) -> i64 = a - b
F multiply(a: i64, b: i64) -> i64 = a * b
F divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 { R Err("Division by zero") }
    Ok(a / b)
}

F main() {
    puts("Simple Calculator")
    puts("Enter first number: ")
    a := read_i64()

    puts("Enter second number: ")
    b := read_i64()

    puts("Choose operation (+, -, *, /): ")
    op := read_char()

    M op {
        '+' => print_i64(add(a, b)),
        '-' => print_i64(subtract(a, b)),
        '*' => print_i64(multiply(a, b)),
        '/' => {
            result := divide(a, b)
            M result {
                Ok(v) => print_i64(v),
                Err(msg) => puts(msg)
            }
        },
        _ => puts("Invalid operation")
    }
}
```

### Implementing a Stack

```vais
S Stack<T> {
    items: Vec<T>,
    capacity: i64
}

X Stack {
    F new(capacity: i64) -> Stack<T> {
        Stack {
            items: Vec::new(),
            capacity: capacity
        }
    }

    F push(&mut self, item: T) -> Result<(), str> {
        I self.items.len() >= self.capacity {
            R Err("Stack overflow")
        }
        self.items.push(item)
        Ok(())
    }

    F pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    F peek(&self) -> Option<&T> {
        I self.items.len() == 0 {
            R None
        }
        Some(&self.items[self.items.len() - 1])
    }

    F is_empty(&self) -> bool {
        self.items.len() == 0
    }
}
```

## Next Steps

You now have a comprehensive understanding of Vais! Continue learning:

### Documentation

- **[Language Specification](../language/language-spec.md)** - Complete syntax reference and formal grammar
- **[Standard Library Guide](../stdlib/README.md)** - Full API reference for all 74 modules
- **[Memory Safety](../advanced/memory-safety.md)** - Deep dive into ownership and borrowing

### Practice

- **[Examples Directory](https://github.com/vaislang/vais/tree/main/examples)** - 189 example programs covering all features
- **[REPL](../tools/repl.md)** - Interactive environment for experimentation
- **[Playground](https://vais.dev/playground/)** - Online editor with instant compilation

### Advanced Topics

- **Macros** - Metaprogramming with declarative macros
- **GPU Programming** - CUDA, Metal, OpenCL, WebGPU support
- **WASM Targets** - Compiling to WebAssembly
- **Package Management** - Publishing and using packages
- **Profiling & Optimization** - Performance tuning

### Community Projects

Build something interesting:
1. **CLI Tool** - Command-line utility with argument parsing
2. **Web Server** - Async HTTP server with routing
3. **Data Processor** - Parse CSV/JSON files and transform data
4. **Game** - Simple game with graphics (using GPU modules)
5. **Compiler Plugin** - Extend Vais with custom lints

### Get Involved

- **GitHub**: [vaislang/vais](https://github.com/vaislang/vais)
- **Issues**: Report bugs or request features
- **Discussions**: Ask questions and share projects
- **Contributing**: See [CONTRIBUTING.md](https://github.com/vaislang/vais/blob/main/CONTRIBUTING.md)

## Quick Reference

### Keywords

| Keyword | Meaning | Example |
|---------|---------|---------|
| `F` | Function | `F add(a: i64, b: i64) -> i64` |
| `S` | Struct | `S Point { x: f64, y: f64 }` |
| `E` | Enum/Else | `E Color { Red, Green }` |
| `I` | If | `I x > 0 { "positive" }` |
| `L` | Loop | `L i : 0..10 { ... }` |
| `M` | Match | `M x { 0 => "zero", _ => "other" }` |
| `W` | Trait | `W Printable { F print(self) }` |
| `X` | Impl | `X Point { F new() -> Point }` |
| `R` | Return | `R 42` |
| `B` | Break | `B` |
| `C` | Continue | `C` |
| `U` | Use | `U std/io` |
| `A` | Async | `A F fetch() -> str` |
| `D` | Defer | `D cleanup()` |

### Operators

| Operator | Meaning |
|----------|---------|
| `@` | Self-recursion |
| `:=` | Variable binding |
| `?` | Try (error propagation) |
| `!` | Unwrap |
| `\|>` | Pipe |
| `..` / `..=` | Range (exclusive/inclusive) |

### Common Patterns

```vais
# Variable declaration
x := 42                      # Inferred
x: i64 = 42                  # Explicit
x := mut 0                   # Mutable

# Function
F name(p: T) -> R { body }
F name(p: T) -> R = expr

# Control flow
I cond { a } E { b }
L { body }
L i : 0..10 { body }
M val { pat => expr, _ => default }

# Self-recursion
F fib(n: i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)

# Error handling
result?                      # Propagate error
option!                      # Unwrap or panic

# Collections
v := Vec::new()
m := HashMap::new()
```
