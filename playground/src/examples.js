// Example code snippets for the playground
export const examples = {
  'hello-world': {
    name: 'Hello World',
    description: 'Simple Hello World program',
    code: `# Hello World example using puts
F main() {
    puts("Hello, Vais!")
}`
  },

  'fibonacci': {
    name: 'Fibonacci',
    description: 'Recursive Fibonacci calculation',
    code: `# Fibonacci function with self-recursion
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)

# Simple addition
F add(a:i64, b:i64)->i64 = a + b

# Main: compute fib(10) = 55
F main()->i64 = fib(10)`
  },

  'generics': {
    name: 'Generics',
    description: 'Generic function example',
    code: `# Generic function test (simple identity)
F identity<T>(x: T) -> T = x

F main() -> i64 {
    puts("Testing generics:")

    a := identity(42)
    puts("identity(42) =")
    putchar(a / 10 + 48)
    putchar(a % 10 + 48)
    putchar(10)

    0
}`
  },

  'control-flow': {
    name: 'Control Flow',
    description: 'If-else and loops',
    code: `# Control flow example
F main()->i64 {
    # If-else expression
    x := 10
    result := I x > 5 {
        puts("x is greater than 5")
        1
    } E {
        puts("x is less than or equal to 5")
        0
    }

    # Loop with range
    L i:0..5 {
        putchar(i + 48)
        putchar(32)  # space
    }
    putchar(10)  # newline

    0
}`
  },

  'struct': {
    name: 'Struct',
    description: 'Struct definition and usage',
    code: `# Struct definition
S Point {
    x: f64,
    y: f64
}

# Method on struct
X Point {
    F distance_from_origin(&self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y)
    }
}

F main() -> i64 {
    p := Point { x: 3.0, y: 4.0 }
    puts("Point created")

    # Access fields and call method
    dist := p.distance_from_origin()

    0
}`
  },

  'enum': {
    name: 'Enum',
    description: 'Enum types with pattern matching',
    code: `# Enum definition
E Color {
    Red,
    Green,
    Blue
}

# Match on enum variants
F color_value(c: Color) -> i64 {
    M c {
        Red => 1,
        Green => 2,
        Blue => 3,
        _ => 0
    }
}

F main() -> i64 {
    r := Red
    g := Green
    b := Blue

    rv := color_value(r)
    gv := color_value(g)
    bv := color_value(b)

    0
}`
  },

  'match': {
    name: 'Pattern Matching',
    description: 'Match expressions',
    code: `# Pattern matching example
F classify(n: i64) -> i64 {
    M n {
        0 => {
            puts("zero")
            0
        },
        1 => {
            puts("one")
            1
        },
        _ => {
            puts("other")
            -1
        }
    }
}

F main() -> i64 {
    classify(0)
    classify(1)
    classify(42)
    0
}`
  },

  'loop': {
    name: 'Loops',
    description: 'Different loop types',
    code: `# Loop examples
F main() -> i64 {
    # Range loop
    puts("Range loop:")
    L i:0..5 {
        putchar(i + 48)
        putchar(32)
    }
    putchar(10)

    # While-style loop with B (break)
    puts("While-style loop:")
    counter := mut 0
    L counter < 5 {
        putchar(counter + 48)
        putchar(32)
        counter = counter + 1
    }
    putchar(10)

    # Infinite loop with break
    puts("Loop with break:")
    n := mut 0
    L {
        I n >= 3 {
            B
        }
        putchar(n + 65)
        putchar(32)
        n = n + 1
    }
    putchar(10)

    0
}`
  },

  'self-recursion': {
    name: 'Self Recursion',
    description: 'Using the @ operator for recursion',
    code: `# Self-recursion operator @ example
F factorial(n: i64) -> i64 = I n <= 1 { 1 } E { n * @(n - 1) }

F sum_to_n(n: i64) -> i64 = I n <= 0 { 0 } E { n + @(n - 1) }

F main() -> i64 {
    # factorial(5) = 120
    fact := factorial(5)

    # sum_to_n(10) = 55
    sum := sum_to_n(10)

    0
}`
  },

  'type-inference': {
    name: 'Type Inference',
    description: 'Automatic type inference',
    code: `# Type inference example
F main() -> i64 {
    # Infer integer type
    x := 42

    # Infer float type
    y := 3.14

    # Infer from function return
    z := add(10, 20)

    # Infer array type
    arr := [1, 2, 3, 4, 5]

    0
}

F add(a: i64, b: i64) -> i64 = a + b`
  },

  'operators': {
    name: 'Operators',
    description: 'Arithmetic and comparison operators',
    code: `# Operators example
F main() -> i64 {
    # Arithmetic
    a := 10 + 5
    b := 10 - 5
    c := 10 * 5
    d := 10 / 5
    e := 10 % 3

    # Comparison
    eq := 5 == 5
    ne := 5 != 3
    gt := 10 > 5
    lt := 5 < 10
    ge := 10 >= 10
    le := 5 <= 10

    # Logical
    and := true && false
    or := true || false
    not := !true

    # Ternary
    max := a > b ? a : b

    0
}`
  },

  'functions': {
    name: 'Functions',
    description: 'Function definitions and calls',
    code: `# Function examples

# Single expression function
F square(x: i64) -> i64 = x * x

# Block function
F print_square(x: i64) -> i64 {
    result := square(x)
    puts("Square calculated")
    result
}

# Function with multiple parameters
F add_three(a: i64, b: i64, c: i64) -> i64 = a + b + c

F main() -> i64 {
    sq := square(5)
    ps := print_square(7)
    sum := add_three(1, 2, 3)

    0
}`
  },

  'string-interpolation': {
    name: 'String Interpolation',
    description: 'Embed expressions in strings with ~{expr}',
    code: `# String interpolation example
F main() -> i64 {
    name := "Vais"
    version := 1

    # Variable interpolation
    puts("Hello, ~{name}!")

    # Expression interpolation
    puts("1 + 2 = ~{1 + 2}")

    0
}`
  },

  'pipe-operator': {
    name: 'Pipe Operator',
    description: 'Chain functions with |>',
    code: `# Pipe operator |> example
F double(x: i64) -> i64 = x * 2
F add_one(x: i64) -> i64 = x + 1
F square(x: i64) -> i64 = x * x

F main() -> i64 {
    # 5 |> double |> add_one = add_one(double(5)) = 11
    result := 5 |> double |> add_one

    # Chain multiple transformations
    result2 := 3 |> square |> double |> add_one

    0
}`
  },

  'mutable-variables': {
    name: 'Mutable Variables',
    description: 'Declare mutable variables with mut',
    code: `# Mutable variable example
F main() -> i64 {
    # Mutable variable declaration
    counter := mut 0
    total := mut 0

    L i: 0..10 {
        counter = counter + 1
        total = total + i
    }

    0
}`
  },

  'destructuring': {
    name: 'Destructuring & Swap',
    description: 'Tuple destructuring and swap builtin',
    code: `# Tuple destructuring and swap builtin
F get_pair() -> (i64, i64) = (10, 20)

F main() -> i64 {
    # Destructure tuple
    (x, y) := get_pair()

    # swap builtin swaps array elements
    arr := [10, 20, 30]
    swap(arr, 0, 2)    # arr = [30, 20, 10]

    0
}`
  },

  'type-infer-params': {
    name: 'Parameter Inference',
    description: 'Variables and return values are inferred',
    code: `# Single expression functions
F add(a: i64, b: i64) -> i64 = a + b
F multiply(a: i64, b: i64) -> i64 = a * b

# Variable types are inferred from expressions
F main() -> i64 {
    # Inferred as i64
    result := add(10, 20)
    product := multiply(3, 4)

    # Inferred from conditional
    max := result > product ? result : product

    0
}`
  },

  'minimal': {
    name: 'Minimal Program',
    description: 'Simplest valid program',
    code: `# Minimal Vais program — expression body
F main() -> i64 = 0`
  },

  'slice-types': {
    name: 'Slice Types',
    description: 'Array slices with fat pointers',
    code: `# Slice types: &[T] and &mut [T]
F sum(data: &[i64]) -> i64 {
    total := mut 0
    i := mut 0
    L i < data.len() {
        total = total + data[i]
        i = i + 1
    }
    R total
}

F main() -> i64 {
    arr := [10, 20, 30, 40, 50]
    slice := arr[1..4]     # &[i64] — [20, 30, 40]

    result := sum(slice)   # 90
    len := slice.len()     # 3

    result
}`
  },

  'traits': {
    name: 'Traits',
    description: 'Define and implement traits',
    code: `# Trait definition with W keyword
W Shape {
    F area(&self) -> f64
    F name(&self) -> str
}

S Circle {
    radius: f64,
}

S Rectangle {
    width: f64,
    height: f64,
}

# Trait implementation with X keyword
X Circle: Shape {
    F area(&self) -> f64 {
        3.14159 * self.radius * self.radius
    }
    F name(&self) -> str { "Circle" }
}

X Rectangle: Shape {
    F area(&self) -> f64 {
        self.width * self.height
    }
    F name(&self) -> str { "Rectangle" }
}

F main() -> i64 {
    c := Circle { radius: 5.0 }
    r := Rectangle { width: 4.0, height: 6.0 }
    0
}`
  },

  'async-await': {
    name: 'Async/Await (compile only)',
    description: 'Asynchronous programming (requires async runtime)',
    code: `# Async functions with A keyword
A F fetch_data(id: i64) -> i64 {
    # Simulate async work
    id * 10
}

A F process() -> i64 {
    # Y keyword for await
    a := Y fetch_data(1)    # 10
    b := Y fetch_data(2)    # 20
    a + b                   # 30
}

A F main() -> i64 {
    result := Y process()
    result
}`
  },

  'ownership': {
    name: 'Ownership',
    description: 'Move semantics and borrowing',
    code: `# Ownership and borrowing (--strict-borrow)
F read_only(data: &Vec<i64>) -> i64 {
    data[0]
}

F modify(data: &mut Vec<i64>) {
    data.push(42)
}

F main() -> i64 {
    items := mut Vec::new()
    items.push(1)
    items.push(2)

    # Immutable borrow
    first := read_only(&items)

    # Mutable borrow
    modify(&mut items)

    items.len()
}`
  },

  'wasm-interop': {
    name: 'WASM Interop',
    description: 'Import JS functions and export Vais functions',
    code: `# WASM Import/Export example
#[wasm_import("env", "console_log")]
N F console_log(ptr: i64, len: i64) -> i64

#[wasm_export("add")]
F add(a: i64, b: i64) -> i64 = a + b

#[wasm_export("fibonacci")]
F fib(n: i64) -> i64 {
    I n <= 1 { n }
    E { @(n - 1) + @(n - 2) }
}

F main() -> i64 {
    result := add(10, 20)
    0
}`
  },

  'lambda-capture': {
    name: 'Lambda Capture',
    description: 'Closure capture modes: ByValue, Move, ByRef',
    code: `# Lambda capture modes
F apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x)

F main() -> i64 {
    # Basic lambda (ByValue capture)
    multiplier := 3
    triple := |x: i64| x * multiplier

    result := apply(triple, 10)    # 30

    # Move capture — ownership transfer
    data := 100
    consumer := move |x: i64| x + data

    result2 := consumer(5)    # 105

    0
}`
  },

  'range-loop': {
    name: 'Range Loops',
    description: 'Iterate with exclusive and inclusive ranges',
    code: `# Range loop examples
F main() -> i64 {
    # Exclusive range: 0, 1, 2, 3, 4
    puts("Exclusive 0..5:")
    L i:0..5 {
        putchar(i + 48)
        putchar(32)
    }
    putchar(10)

    # Sum with range
    total := mut 0
    L i:1..11 {
        total = total + i
    }

    # total = 55 (1+2+...+10)
    total
}`
  },

  'lazy-evaluation': {
    name: 'Lazy Evaluation (compile only)',
    description: 'Deferred computation with lazy/force (codegen in progress)',
    code: `# Lazy evaluation with caching
F expensive(n: i64) -> i64 {
    # Simulate heavy computation
    n * n + n * 2 + 1
}

F main() -> i64 {
    # Defer evaluation
    val := lazy expensive(10)

    # Force evaluates and caches result
    result := force val      # 121

    # Second force returns cached value
    result2 := force val     # 121 (cached)

    result
}`
  },

  'result-option': {
    name: 'Result & Option',
    description: 'Error handling with Result and Option types',
    code: `# Result and Option pattern matching
E Option<T> {
    Some(T),
    None
}

E Result<T, E> {
    Ok(T),
    Err(E)
}

F find_value(key: i64) -> Option<i64> {
    I key == 1 { Some(100) }
    E I key == 2 { Some(200) }
    E { None }
}

F divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 { Err("division by zero") }
    E { Ok(a / b) }
}

F main() -> i64 {
    # Pattern match on Option
    M find_value(1) {
        Some(v) => {
            puts("Found:")
            print_i64(v)
        },
        None => puts("Not found")
    }

    # Pattern match on Result
    M divide(10, 2) {
        Ok(v) => {
            puts("Result:")
            print_i64(v)
        },
        Err(msg) => puts(msg)
    }

    0
}`
  },

  'try-operator': {
    name: 'Try Operator (?)',
    description: 'Propagate errors with the ? operator',
    code: `# Try operator (?) for error propagation
E Result<T, E> {
    Ok(T),
    Err(E)
}

F parse_number(s: str) -> Result<i64, str> {
    I s == "42" { Ok(42) }
    E I s == "0" { Ok(0) }
    E { Err("parse error") }
}

F validate(n: i64) -> Result<i64, str> {
    I n < 0 { Err("negative") }
    E { Ok(n * 2) }
}

# ? operator unwraps Ok or returns Err early
F process(input: str) -> Result<i64, str> {
    n := parse_number(input)?
    result := validate(n)?
    Ok(result)
}

F main() -> i64 {
    M process("42") {
        Ok(v) => {
            puts("Success:")
            print_i64(v)    # 84
        },
        Err(e) => puts(e)
    }

    M process("bad") {
        Ok(v) => print_i64(v),
        Err(e) => {
            puts("Error:")
            puts(e)         # "parse error"
        }
    }

    0
}`
  },

  'unwrap-operator': {
    name: 'Unwrap Operator (!)',
    description: 'Unwrap values or panic with the ! operator',
    code: `# Unwrap operator (!) for assertive access
E Option<T> {
    Some(T),
    None
}

E Result<T, E> {
    Ok(T),
    Err(E)
}

F get_config() -> Option<i64> {
    Some(42)
}

F compute() -> Result<i64, str> {
    Ok(100)
}

F main() -> i64 {
    # ! unwraps Some/Ok, panics on None/Err
    config := get_config()!      # 42
    value := compute()!          # 100

    print_i64(config)
    print_i64(value)

    config + value    # 142
}`
  },

  'where-clause': {
    name: 'Where Clause',
    description: 'Constrain generic types with where clauses',
    code: `# Where clause for generic constraints
W Printable {
    F to_string(&self) -> str
}

W Comparable {
    F compare(&self, other: &Self) -> i64
}

# Generic function with where clause
F print_if_positive<T>(val: T, n: i64) -> i64
    where T: Printable
{
    I n > 0 {
        puts("Value is associated with positive number")
        1
    } E {
        0
    }
}

# Multiple constraints
F process<T>(item: T) -> i64
    where T: Printable + Comparable
{
    0
}

F main() -> i64 {
    puts("Where clause example")
    0
}`
  },

  'defer-statement': {
    name: 'Defer Statement',
    description: 'Execute cleanup code when scope exits',
    code: `# Defer statement (D keyword)
F main() -> i64 {
    # Deferred actions run in LIFO order at scope exit
    D puts("cleanup: third (runs first)")
    D puts("cleanup: second")
    D puts("cleanup: first (runs last)")

    puts("main body executing")
    puts("doing work...")

    0
}
# Output order:
# main body executing
# doing work...
# cleanup: first (runs last)
# cleanup: second
# cleanup: third (runs first)`
  }
};

// Get array of example keys for the UI
export function getExampleList() {
  return Object.entries(examples).map(([key, value]) => ({
    key,
    name: value.name,
    description: value.description
  }));
}

// Get code for a specific example
export function getExampleCode(key) {
  return examples[key]?.code || '';
}
