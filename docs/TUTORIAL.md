# Vais Programming Tutorial

Welcome to Vais! This tutorial will guide you through the basics of programming in Vais, from installation to writing your first programs.

## Table of Contents

1. [Installation](#installation)
2. [Hello World](#hello-world)
3. [Variables and Types](#variables-and-types)
4. [Functions](#functions)
5. [Control Flow](#control-flow)
6. [Structs and Enums](#structs-and-enums)
7. [Pattern Matching](#pattern-matching)
8. [Traits and Methods](#traits-and-methods)
9. [Generics](#generics)
10. [Standard Library Basics](#standard-library-basics)
11. [Async Programming](#async-programming)
12. [Next Steps](#next-steps)

---

## Installation

### Prerequisites

- Rust toolchain (for building the compiler)
- LLVM (for code generation)
- Clang (for compiling generated LLVM IR)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/sswoo88/vais.git
cd vais

# Build the compiler
cargo build --release

# The compiler will be available at:
./target/release/vaisc
```

### Verify Installation

```bash
./target/release/vaisc --version
# Should output: Vais 0.0.1
```

---

## Hello World

Let's write your first Vais program!

### Create a file `hello.vais`:

```vais
# Hello World example
F main()->i64 {
    puts("Hello, Vais!")
    0
}
```

### Compile and run:

```bash
./target/release/vaisc hello.vais
./hello
```

**Output:**
```
Hello, Vais!
```

### Understanding the code:

- `F` - Keyword for function definition
- `main` - Entry point function name
- `()->i64` - Function signature: no parameters, returns i64
- `puts("Hello, Vais!")` - Print a string
- `0` - Return value (convention: 0 for success)

---

## Variables and Types

### Type-Inferred Variables

Use `:=` for automatic type inference:

```vais
F main()->i64 {
    x := 10          # i64 inferred
    y := 3.14        # f64 inferred
    name := "Alice"  # str inferred
    flag := true     # bool inferred

    puts("Variables declared!")
    0
}
```

### Explicit Types

Specify types explicitly with `:`:

```vais
F main()->i64 {
    x: i64 = 100
    y: f64 = 2.5
    count: i32 = 42

    puts("Typed variables declared!")
    0
}
```

### Primitive Types

**Integers:**
```vais
a: i8 = 127          # 8-bit signed
b: i16 = 32000       # 16-bit signed
c: i32 = 1000000     # 32-bit signed
d: i64 = 999999999   # 64-bit signed

ua: u8 = 255         # 8-bit unsigned
ub: u32 = 4294967295 # 32-bit unsigned
```

**Floating-point:**
```vais
x: f32 = 3.14        # 32-bit float
y: f64 = 2.718281828 # 64-bit float
```

**Boolean:**
```vais
is_ready := true
is_done := false
```

### Using Variables

```vais
F main()->i64 {
    x := 10
    y := 20
    sum := x + y

    puts("Sum calculated!")
    0
}
```

---

## Functions

### Simple Functions

**Expression form** (single expression):

```vais
F add(a:i64, b:i64)->i64 = a + b

F square(x:i64)->i64 = x * x

F max(a:i64, b:i64)->i64 = a > b ? a : b
```

**Block form** (multiple statements):

```vais
F greet(name: str)->i64 {
    puts("Hello, ")
    puts(name)
    puts("!")
    0
}
```

### Function Parameters

```vais
# Multiple parameters with different types
F calculate(x: i64, y: f64, multiplier: i64) -> f64 {
    result := x * multiplier
    result * y
}
```

### Calling Functions

```vais
F main()->i64 {
    sum := add(10, 20)
    squared := square(5)
    maximum := max(100, 200)

    puts("Functions called!")
    0
}
```

### Self-Recursion with `@`

The `@` operator calls the current function recursively:

```vais
# Fibonacci using self-recursion
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)

# Factorial
F factorial(n:i64)->i64 = n<2 ? 1 : n * @(n-1)

# Countdown
F countdown(n:i64)->i64 {
    I n <= 0 {
        puts("Done!")
        0
    } E {
        puts("Counting...")
        @(n-1)
    }
}
```

**Why use `@`?**
- More concise than writing the function name
- Fewer tokens for AI code generation
- Clear indicator of recursion

---

## Control Flow

### If-Else Expressions

**Ternary form** (single expression):

```vais
F abs(x:i64)->i64 = x < 0 ? -x : x

F sign(x:i64)->i64 = x < 0 ? -1 : x > 0 ? 1 : 0
```

**Block form:**

```vais
F classify(x:i64)->str {
    I x < 0 {
        "negative"
    } E I x == 0 {
        "zero"
    } E {
        "positive"
    }
}
```

Note: `E` is used for "else". Context determines whether `E` means "enum" or "else".

### Loops

**Infinite loop:**

```vais
F loop_forever()->i64 {
    L {
        puts("Looping...")
        # Need break condition
    }
    0
}
```

**Range loop:**

```vais
F count_to_ten()->i64 {
    L i: 0..10 {
        puts("Number: ")
        print_i64(i)
        putchar(10)
    }
    0
}
```

**With break and continue:**

```vais
F find_first_even()->i64 {
    L i: 0..100 {
        I i % 2 == 0 {
            puts("Found even number:")
            print_i64(i)
            B  # Break
        }
        C  # Continue
    }
    0
}
```

### Early Return

```vais
F validate(x: i64)->i64 {
    I x < 0 {
        puts("Error: negative value")
        R -1  # Early return
    }
    I x == 0 {
        puts("Error: zero value")
        R -1
    }

    # Process valid value
    puts("Valid!")
    x * 2
}
```

---

## Structs and Enums

### Defining Structs

```vais
S Point {
    x: f64,
    y: f64
}

S Person {
    name: str,
    age: i64
}

S Rectangle {
    top_left: Point,
    bottom_right: Point
}
```

### Creating Struct Instances

```vais
F main()->i64 {
    # Create a Point
    p := Point { x: 10.0, y: 20.0 }

    # Create a Person
    person := Person { name: "Bob", age: 25 }

    # Nested structs
    rect := Rectangle {
        top_left: Point { x: 0.0, y: 10.0 },
        bottom_right: Point { x: 10.0, y: 0.0 }
    }

    0
}
```

### Accessing Fields

```vais
F main()->i64 {
    p := Point { x: 5.0, y: 15.0 }

    x_coord := p.x
    y_coord := p.y

    puts("Point coordinates:")
    print_f64(x_coord)
    print_f64(y_coord)

    0
}
```

### Defining Enums

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

E Message {
    Quit,
    Move(i64, i64),
    Write(str)
}
```

### Using Enums

```vais
F main()->i64 {
    color := Red
    opt := Some(42)
    result := Ok(100)
    msg := Move(10, 20)

    puts("Enums created!")
    0
}
```

---

## Pattern Matching

Pattern matching with `M` (match) is powerful for working with enums and values.

### Basic Match

```vais
F describe_number(n: i64)->str {
    M n {
        0 => "zero",
        1 => "one",
        2 => "two",
        _ => "other"  # Wildcard: matches everything else
    }
}
```

### Match with Binding

Extract values from matched patterns:

```vais
E Option {
    None,
    Some(i64)
}

F unwrap_or(opt: Option, default: i64) -> i64 {
    M opt {
        Some(x) => x,        # Bind value to 'x'
        None => default
    }
}

F main()->i64 {
    opt1 := Some(42)
    opt2 := None

    v1 := unwrap_or(opt1, 0)  # Returns 42
    v2 := unwrap_or(opt2, 99) # Returns 99

    print_i64(v1)
    print_i64(v2)
    0
}
```

### Match with Result Types

```vais
E Result {
    Ok(i64),
    Err(str)
}

F handle_result(res: Result) -> i64 {
    M res {
        Ok(value) => value,
        Err(msg) => {
            puts("Error: ")
            puts(msg)
            0
        }
    }
}
```

### Complete Example

```vais
E Color {
    Red,
    Green,
    Blue
}

F color_to_code(c: Color) -> i64 {
    M c {
        Red => 0xFF0000,
        Green => 0x00FF00,
        Blue => 0x0000FF
    }
}

F main()->i64 {
    red_code := color_to_code(Red)
    green_code := color_to_code(Green)

    puts("Color codes calculated!")
    0
}
```

---

## Traits and Methods

### Defining Traits

Traits define interfaces that types can implement:

```vais
W Printable {
    F print(&self) -> i64
}

W Comparable {
    F compare(&self, other: &Self) -> i64
}
```

### Implementing Traits

```vais
S Counter {
    value: i64
}

# Implement Printable trait for Counter
X Counter: Printable {
    F print(&self) -> i64 {
        puts("Counter value: ")
        print_i64(self.value)
        putchar(10)
        0
    }
}
```

### Adding Methods

Add methods without traits using `X`:

```vais
X Counter {
    F increment(&self) -> i64 {
        self.value + 1
    }

    F double(&self) -> i64 {
        self.value * 2
    }

    F reset() -> Counter {
        Counter { value: 0 }
    }
}
```

### Using Methods

```vais
F main()->i64 {
    c := Counter { value: 10 }

    # Call trait method
    c.print()

    # Call impl methods
    inc := c.increment()
    dbl := c.double()

    puts("Incremented: ")
    print_i64(inc)
    puts("Doubled: ")
    print_i64(dbl)

    0
}
```

### Complete Example

```vais
W Shape {
    F area(&self) -> f64
}

S Circle {
    radius: f64
}

S Rectangle {
    width: f64,
    height: f64
}

X Circle: Shape {
    F area(&self) -> f64 {
        pi := 3.14159
        pi * self.radius * self.radius
    }
}

X Rectangle: Shape {
    F area(&self) -> f64 {
        self.width * self.height
    }
}

F main()->i64 {
    circle := Circle { radius: 5.0 }
    rect := Rectangle { width: 4.0, height: 6.0 }

    circle_area := circle.area()
    rect_area := rect.area()

    puts("Circle area: ")
    print_f64(circle_area)

    puts("Rectangle area: ")
    print_f64(rect_area)

    0
}
```

---

## Generics

Generics allow you to write code that works with multiple types.

### Generic Functions

```vais
F identity<T>(x: T) -> T = x

F first<T>(a: T, b: T) -> T = a

F swap<A, B>(a: A, b: B) -> (B, A) {
    (b, a)
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

S Container<K, V> {
    key: K,
    value: V
}
```

### Using Generic Structs

```vais
F main()->i64 {
    # Pair of integers
    int_pair := Pair { first: 10, second: 20 }

    # Pair of floats
    float_pair := Pair { first: 1.5, second: 2.5 }

    # Container with different types
    container := Container { key: 1, value: "hello" }

    0
}
```

### Methods on Generic Types

```vais
S Pair<T> {
    first: T,
    second: T
}

X Pair {
    F sum(&self) -> i64 {
        self.first + self.second
    }

    F swap(&self) -> Pair {
        Pair { first: self.second, second: self.first }
    }
}

F main()->i64 {
    p := Pair { first: 10, second: 20 }
    total := p.sum()
    swapped := p.swap()

    print_i64(total)  # 30
    0
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

F main()->i64 {
    # Option of i64
    opt_int := Some(42)

    # Option of str
    opt_str := Some("hello")

    # Result with i64 value and str error
    result := Ok(100)

    0
}
```

---

## Standard Library Basics

### Using the Math Library

```vais
U std/math

F main()->i64 {
    # Constants
    pi := PI
    e := E

    # Basic math
    x := abs(-10.0)          # Absolute value
    min_val := min(5.0, 10.0)
    max_val := max(5.0, 10.0)

    # Advanced math
    root := sqrt(16.0)       # Square root: 4.0
    power := pow(2.0, 8.0)   # 2^8 = 256.0

    # Trigonometry
    sine := sin(PI / 2.0)    # sin(90°) = 1.0
    cosine := cos(0.0)       # cos(0°) = 1.0

    # Logarithms
    natural_log := log(E)    # ln(e) = 1.0
    log_base_10 := log10(100.0)  # 2.0

    print_f64(root)
    0
}
```

### Using the IO Library

```vais
U std/io

F main()->i64 {
    # Read an integer
    puts("Enter a number: ")
    num := read_i64()
    puts("You entered: ")
    print_i64(num)
    putchar(10)

    # Read a float
    puts("Enter a decimal: ")
    decimal := read_f64()
    puts("You entered: ")
    print_f64(decimal)
    putchar(10)

    # Prompt functions
    age := prompt_i64("Enter your age: ")
    height := prompt_f64("Enter your height: ")

    puts("Age: ")
    print_i64(age)
    puts("Height: ")
    print_f64(height)

    0
}
```

### Using Option and Result

```vais
U std/option
U std/result

F divide(a: i64, b: i64) -> Option {
    I b == 0 {
        None
    } E {
        Some(a / b)
    }
}

F main()->i64 {
    result := divide(10, 2)
    value := result.unwrap_or(0)  # Returns 5

    error_result := divide(10, 0)
    default_value := error_result.unwrap_or(-1)  # Returns -1

    print_i64(value)
    print_i64(default_value)
    0
}
```

---

## Async Programming

Vais supports async/await for concurrent programming.

### Defining Async Functions

```vais
# Mark function as async with 'A'
A F compute(x: i64) -> i64 {
    x * 2
}

A F fetch_data(id: i64) -> str {
    # Simulate async operation
    "Data loaded"
}
```

### Awaiting Async Functions

```vais
F main()->i64 {
    # Call async function and await the result
    result := compute(21).await

    puts("Result: ")
    print_i64(result)  # 42

    # Chain async calls
    data := fetch_data(1).await
    puts(data)

    0
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

F main()->i64 {
    # Spawn tasks to run concurrently
    t1 := spawn task1()
    t2 := spawn task2()

    # Await results
    r1 := t1.await
    r2 := t2.await

    total := r1 + r2
    print_i64(total)  # 300

    0
}
```

---

## Next Steps

### Complete Examples

Explore the `examples/` directory for more complete programs:

- `fib.vais` - Fibonacci with self-recursion
- `pattern_match_test.vais` - Pattern matching examples
- `trait_test.vais` - Traits and implementations
- `generic_struct_test.vais` - Generic types
- `async_test.vais` - Async/await examples
- `io_test.vais` - Interactive I/O examples

### Further Reading

- **Language Specification**: See `LANGUAGE_SPEC.md` for complete language reference
- **Standard Library**: See `STDLIB.md` for all available modules and functions
- **REPL**: Try the interactive REPL with `vaisc repl`

### Practice Projects

1. **Calculator**: Build a simple calculator using the IO library
2. **File Processor**: Read and process files using `std/file`
3. **Data Structures**: Implement your own Vector or HashMap
4. **Async Web Server**: Build a simple server using async/await

### Community

- GitHub: [https://github.com/yourusername/vais](https://github.com/yourusername/vais)
- Issues: Report bugs or request features
- Discussions: Ask questions and share projects

---

## Quick Reference

### Function Definition
```vais
F name(param: type)->return_type = expr
F name(param: type)->return_type { body }
```

### Variables
```vais
x := value        # Type inferred
x: type = value   # Explicit type
```

### Control Flow
```vais
I condition { then } E { else }
L { loop_body }
L var: range { body }
M value { pattern => expr, ... }
```

### Self-Recursion
```vais
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)
```

### Struct
```vais
S Name { field: type, ... }
X Name { F method(&self)->type { body } }
```

### Enum
```vais
E Name { Variant, Variant(type), ... }
```

### Trait
```vais
W Trait { F method(&self)->type }
X Type: Trait { F method(&self)->type { body } }
```

### Generics
```vais
F name<T>(x: T) -> T { body }
S Name<T> { field: T }
```

### Async
```vais
A F name() -> type { body }
result := async_func().await
```

---

Happy coding with Vais!
