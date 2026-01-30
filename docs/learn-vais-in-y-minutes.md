---
language: Vais
filename: learnvais.vais
contributors:
    - ["Vais Contributors", "https://github.com/YOUR_USERNAME/vais"]
---

Vais is a token-efficient, AI-optimized systems programming language with an
LLVM backend. It features single-character keywords, expression-oriented syntax,
a unique self-recursion operator, and modern features like generics, traits,
pattern matching, and async/await.

```vais
# Single-line comments start with #

# ------------------------------------------------------------
# 1. Primitive Types and Literals
# ------------------------------------------------------------

# Integer types: i8, i16, i32, i64, i128 (signed)
#                u8, u16, u32, u64, u128 (unsigned)
# Floating-point: f32, f64
# Boolean: bool (true or false)
# String: str

# Integer literals
42
1_000_000      # Underscores for readability
-42            # Negative numbers

# Float literals
3.14
1.0e10
2.5e-3

# Boolean literals
true
false

# String literals
"Hello, Vais!"
"Escape with \"quotes\""

# ------------------------------------------------------------
# 2. Single-Character Keywords (Token Efficiency)
# ------------------------------------------------------------

# F = Function
# S = Struct
# E = Enum (or Else in if expressions)
# W = Trait (Where/interface)
# X = Impl (eXtend)
# I = If
# L = Loop
# M = Match
# R = Return
# B = Break
# C = Continue (also used for Const)
# T = Type alias
# U = Use (import)
# P = Pub (public visibility)
# A = Async

# ------------------------------------------------------------
# 3. Variables and Type Inference
# ------------------------------------------------------------

# Type-inferred binding with :=
x := 42               # i64 inferred
y := 3.14             # f64 inferred
name := "Alice"       # str inferred

# Explicit type annotation
z: i64 = 100

# Mutable variables
mut counter := 0
counter = counter + 1

# ------------------------------------------------------------
# 4. Functions
# ------------------------------------------------------------

# Expression form (single expression, no block)
F add(a:i64, b:i64)->i64 = a + b

# Block form (multiple statements)
F greet(name: str) -> i64 {
    puts("Hello, ")
    puts(name)
    0    # Last expression is the return value
}

# Self-recursion operator @ (calls current function)
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)

# Equivalent to:
# F fib(n:i64)->i64 = n<2 ? n : fib(n-1) + fib(n-2)

# Early return with R keyword
F compute(x: i64) -> i64 {
    I x < 0 {
        R 0    # Early return
    }
    x * 2
}

# Main function - entry point
F main()->i64 {
    puts("Hello, Vais!")
    0    # Return 0 for success
}

# ------------------------------------------------------------
# 5. Operators
# ------------------------------------------------------------

# Arithmetic: +, -, *, /, %
sum := 10 + 5         # 15
diff := 10 - 5        # 5
product := 10 * 5     # 50
quotient := 10 / 5    # 2
remainder := 10 % 3   # 1

# Comparison: ==, !=, <, >, <=, >=
is_equal := 5 == 5    # true
not_equal := 5 != 3   # true
less_than := 3 < 5    # true

# Logical/Bitwise: &, |, ^, !, ~, <<, >>
and_result := 5 & 3   # Bitwise AND
or_result := 5 | 3    # Bitwise OR
xor_result := 5 ^ 3   # Bitwise XOR
not_result := !true   # false
bit_not := ~0         # Bitwise NOT
left_shift := 1 << 3  # 8
right_shift := 8 >> 1 # 4

# Ternary conditional: condition ? true_value : false_value
abs := x < 0 ? -x : x
max := a > b ? a : b

# Assignment: =, :=, +=, -=, *=, /=
x = 10
x += 5    # x = x + 5
x -= 2    # x = x - 2
x *= 3    # x = x * 3
x /= 2    # x = x / 2

# ------------------------------------------------------------
# 6. Control Flow
# ------------------------------------------------------------

# If-Else (E is used for "else")
I x > 0 {
    puts("positive")
} E I x < 0 {
    puts("negative")
} E {
    puts("zero")
}

# If is an expression (returns a value)
sign := I x > 0 { 1 } E I x < 0 { -1 } E { 0 }

# Ternary is more concise for simple cases
sign2 := x > 0 ? 1 : (x < 0 ? -1 : 0)

# Loop - infinite
L {
    # ... do work
    I condition { B }    # Break with B
}

# Loop with range
L i: 0..10 {
    putchar(i + 48)    # Print digits 0-9
}

# Loop with array iteration
arr := [10, 20, 30]
L item: arr {
    print_i64(item)
}

# Continue to next iteration with C
L i: 0..10 {
    I i % 2 == 0 { C }    # Skip even numbers
    print_i64(i)
}

# ------------------------------------------------------------
# 7. Pattern Matching
# ------------------------------------------------------------

# Match expression with M keyword
F classify(n: i64) -> str {
    M n {
        0 => "zero",
        1 => "one",
        2 => "two",
        _ => "other"    # Wildcard pattern
    }
}

# Match with enum variants (see Enums section)
E Option {
    None,
    Some(i64)
}

F unwrap_or(opt: Option, default: i64) -> i64 {
    M opt {
        Some(x) => x,       # Bind value to x
        None => default
    }
}

# ------------------------------------------------------------
# 8. Structs
# ------------------------------------------------------------

# Define a struct with S keyword
S Point {
    x: f64,
    y: f64
}

# Instantiate
p := Point { x: 10.0, y: 20.0 }

# Access fields
x_coord := p.x
y_coord := p.y

# Generic struct
S Pair<T> {
    first: T,
    second: T
}

# Use generic struct
pair_i64 := Pair { first: 10, second: 20 }
pair_f64 := Pair { first: 1.5, second: 2.5 }

# ------------------------------------------------------------
# 9. Enums
# ------------------------------------------------------------

# Simple enum
E Color {
    Red,
    Green,
    Blue
}

color := Red

# Enum with associated data
E Option {
    None,
    Some(i64)
}

E Result {
    Ok(i64),
    Err(str)
}

# Use enums
opt := Some(42)
result := Ok(100)
error := Err("file not found")

# Enum with struct variants
E Message {
    Quit,
    Move { x: i64, y: i64 },
    Write(str)
}

# ------------------------------------------------------------
# 10. Methods and Impl Blocks
# ------------------------------------------------------------

S Counter {
    value: i64
}

# Implement methods with X keyword
X Counter {
    # Expression form method
    F increment(&self) -> i64 = self.value + 1

    # Block form method
    F double(&self) -> i64 {
        self.value * 2
    }

    F reset(&self) -> Counter {
        Counter { value: 0 }
    }
}

# Call methods
c := Counter { value: 42 }
incremented := c.increment()    # 43
doubled := c.double()           # 84

# ------------------------------------------------------------
# 11. Traits (Interfaces)
# ------------------------------------------------------------

# Define a trait with W keyword
W Printable {
    F print(&self) -> i64
}

W Comparable {
    F compare(&self, other: &Self) -> i64
}

# Implement trait for a type
X Counter: Printable {
    F print(&self) -> i64 {
        puts("Counter value:")
        print_i64(self.value)
        0
    }
}

# Use trait method
c.print()

# ------------------------------------------------------------
# 12. Generics
# ------------------------------------------------------------

# Generic function
F identity<T>(x: T) -> T = x

F swap<A, B>(a: A, b: B) -> (B, A) {
    (b, a)
}

# Generic struct (already shown above)
S Box<T> {
    value: T
}

# Generic enum
E Option<T> {
    None,
    Some(T)
}

E Result<T, E> {
    Ok(T),
    Err(E)
}

# Generic with trait bounds
F print_all<T: Printable>(items: [T]) -> i64 {
    L item: items {
        item.print()
    }
    0
}

# ------------------------------------------------------------
# 13. Closures (Lambdas)
# ------------------------------------------------------------

# Closure syntax: |params| expression_or_block
double := |x: i64| x * 2
result := double(5)    # 10

# Closures capture variables from environment
multiplier := 10
scale := |x: i64| x * multiplier
result2 := scale(5)    # 50

# Multiple captures
base := 20
offset := 3
compute := |x: i64| base + x + offset
result3 := compute(7)    # 30

# ------------------------------------------------------------
# 14. Arrays and Pointers
# ------------------------------------------------------------

# Array literal
arr: *i64 = [10, 20, 30]

# Array access
first := arr[0]       # 10
second := arr[1]      # 20

# Pointer type
ptr: *i64 = malloc(64)
store_i64(ptr, 42)
value := load_i64(ptr)
free(ptr)

# ------------------------------------------------------------
# 15. Strings
# ------------------------------------------------------------

# String literals
greeting := "Hello, Vais!"

# String operations (from stdlib)
puts("Hello")              # Print with newline
puts_ptr(str_ptr)          # Print from pointer
len := strlen(str_ptr)     # Get length

# Format strings (if supported)
# println("Value: {}", value)

# ------------------------------------------------------------
# 16. Module System
# ------------------------------------------------------------

# Import with U keyword
U std/math
U std/io
U std/collections

# Use imported items
pi_value := PI              # Constant from std/math
sqrt_val := sqrt(16.0)      # Function from std/math

# ------------------------------------------------------------
# 17. Async/Await
# ------------------------------------------------------------

# Define async function with A F
A F fetch_data(url: str) -> str {
    # ... async operations
    "data"
}

A F compute_async(x: i64) -> i64 {
    x * 2
}

F main() -> i64 {
    # Call async function and await result
    result := compute_async(21).await    # 42

    # Spawn concurrent task
    task := spawn fetch_data("example.com")

    # Do other work...

    # Await task result later
    data := task.await

    0
}

# ------------------------------------------------------------
# 18. Constants
# ------------------------------------------------------------

# Define constants with C keyword
C PI: f64 = 3.141592653589793
C MAX_SIZE: i64 = 1024
C VERSION: str = "0.0.1"

# Use constants
circumference := 2.0 * PI * radius

# ------------------------------------------------------------
# 19. External Functions (FFI)
# ------------------------------------------------------------

# Declare external C functions with X F
X F puts(s: i64) -> i64
X F malloc(size: i64) -> i64
X F free(ptr: i64) -> i64
X F sqrt(x: f64) -> f64

# Use external functions
puts("Hello from C!")
mem := malloc(1024)
free(mem)

# ------------------------------------------------------------
# 20. Built-in Functions
# ------------------------------------------------------------

# I/O
puts("text")              # Print string with newline
putchar(65)               # Print character 'A'
puts_ptr(ptr)             # Print C string from pointer

# Memory operations
ptr := malloc(64)         # Allocate
free(ptr)                 # Free
memcpy(dst, src, n)       # Copy memory
store_i64(ptr, 42)        # Store i64
value := load_i64(ptr)    # Load i64
store_byte(ptr, 65)       # Store byte
b := load_byte(ptr)       # Load byte

# Utility
print_i64(42)             # Print integer
print_f64(3.14)           # Print float
len := strlen(str_ptr)    # String length

# ------------------------------------------------------------
# 21. Standard Library
# ------------------------------------------------------------

# Option type (from std/option)
U std/option

opt := Some(42)
opt2 := None

value := M opt {
    Some(x) => x,
    None => 0
}

# Result type (from std/result)
U std/result

res := Ok(100)
err := Err("error message")

# Box - unique ownership (from std/box)
U std/box

b := Box::new(42)
val := b.get()

# Rc - reference counting (from std/rc)
U std/rc

rc := Rc::new(100)
rc2 := rc.clone()    # Increment refcount
val2 := rc.get()

# Arena allocator (from std/arena)
U std/arena

arena := Arena::with_capacity(1024)
ptr := arena.alloc(64)
arena.reset()    # Free all at once

# Collections (from std/collections)
U std/collections

# Vector, HashMap, BTreeMap, Set, Deque, PriorityQueue available

# Math functions (from std/math)
U std/math

sqrt_val := sqrt(16.0)
pow_val := pow(2.0, 8.0)
sin_val := sin(PI / 2.0)

# ------------------------------------------------------------
# 22. Comprehensive Example
# ------------------------------------------------------------

# Define an enum for a binary tree
E Tree<T> {
    Empty,
    Node { value: T, left: *Tree<T>, right: *Tree<T> }
}

# Define a trait for summation
W Summable {
    F sum(&self) -> i64
}

# Implement the trait for Tree
X Tree: Summable {
    F sum(&self) -> i64 {
        M self {
            Empty => 0,
            Node { value, left, right } => {
                value + left.sum() + right.sum()
            }
        }
    }
}

# Helper function to create a node
F node(val: i64) -> Tree<i64> {
    Node {
        value: val,
        left: Empty,
        right: Empty
    }
}

# Factorial with self-recursion
F factorial(n: i64) -> i64 = n < 2 ? 1 : n * @(n - 1)

# Quick sort implementation
F quicksort(arr: *i64, len: i64) -> i64 {
    I len <= 1 { R 0 }

    pivot := arr[0]
    # ... partition logic
    # @(left_arr, left_len)
    # @(right_arr, right_len)
    0
}

# Async web request (conceptual)
A F fetch_user(id: i64) -> str {
    url := format("https://api.example.com/users/{}", id)
    response := http_get(url).await
    response.body
}

F main() -> i64 {
    # Test fibonacci
    fib_10 := fib(10)    # 55
    puts("Fibonacci(10) = 55")

    # Test factorial
    fact_5 := factorial(5)    # 120
    puts("Factorial(5) = 120")

    # Test pattern matching
    opt := Some(42)
    value := unwrap_or(opt, 0)
    puts("unwrap_or(Some(42), 0) = 42")

    # Test generics
    pair := Pair { first: 10, second: 20 }
    puts("Pair created")

    # Test closures
    multiplier := 5
    times_five := |x: i64| x * multiplier
    result := times_five(8)    # 40
    puts("Closure: 8 * 5 = 40")

    # Test async (requires runtime)
    # user := fetch_user(123).await

    puts("All tests complete!")
    0
}

# ------------------------------------------------------------
# 23. Best Practices
# ------------------------------------------------------------

# 1. Use type inference (:=) when type is obvious
good := 42                    # Good
bad: i64 = 42                # Unnecessary type annotation

# 2. Prefer expression form for simple functions
F add(a:i64, b:i64)->i64 = a + b    # Good
# vs multi-line block for one-liners

# 3. Use self-recursion operator @
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)    # Concise

# 4. Match instead of nested if-else
M value {                    # Good - exhaustive
    0 => "zero",
    1 => "one",
    _ => "other"
}

# 5. Leverage generics to reduce duplication
S Container<T> { data: T }   # Works for any T

# 6. Use single-letter keywords for token efficiency
F main()->i64 { 0 }          # Very concise

# 7. Pattern match to destructure data
M result {
    Ok(value) => value,
    Err(msg) => 0
}

```

## Further Reading

* [Official Vais Repository](https://github.com/YOUR_USERNAME/vais)
* [Language Specification](docs/LANGUAGE_SPEC.md)
* [Tutorial](docs/TUTORIAL.md)
* [Standard Library Documentation](docs/STDLIB.md)
* [Generic Programming Guide](docs/generic_tutorial.md)
* [Async Programming Guide](docs/async_tutorial.md)

## Key Takeaways

- **Token-efficient**: Single-letter keywords reduce token count in AI code generation
- **Expression-oriented**: Everything returns a value, enabling concise code
- **Self-recursion operator @**: Makes recursive functions more readable
- **Modern features**: Generics, traits, pattern matching, async/await
- **Systems programming**: LLVM backend provides native performance
- **Type inference**: Less annotation, more productivity
- **Zero-cost abstractions**: High-level features compile to efficient machine code
