// Example code snippets for the playground
export const examples = {
  'hello-world': {
    name: 'Hello World',
    description: 'Simple Hello World program',
    code: `# Hello World example
F main() -> i64 {
    puts("Hello, Vais!")
    puts("Welcome to Vais — an AI-optimized systems language")
    0
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
    description: 'Embed expressions in strings with {expr}',
    code: `# String interpolation example
F main() -> i64 {
    name := "Vais"
    version := 1

    # Variable interpolation
    puts("Hello, {name}!")

    # Expression interpolation
    puts("Version: {version}")
    puts("1 + 2 = {1 + 2}")

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
    name: 'Destructuring',
    description: 'Tuple destructuring and multi-value returns',
    code: `# Tuple destructuring
F get_pair() -> (i64, i64) = (10, 20)
F get_triple() -> (i64, i64, i64) = (1, 2, 3)

F main() -> i64 {
    # Destructure pair
    (x, y) := get_pair()
    puts("Pair: ({x}, {y})")

    # Destructure triple
    (a, b, c) := get_triple()
    puts("Triple: ({a}, {b}, {c})")

    # x=10, y=20, a=1, b=2, c=3 -> sum=36
    x + y + a + b + c
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
    name: 'Arrays & Pointers',
    description: 'Array operations with pointer passing',
    code: `# Array operations with pointer passing
F sum(arr: *i64, len: i64) -> i64 {
    total := mut 0
    i := mut 0
    L i < len {
        total = total + arr[i]
        i = i + 1
    }
    total
}

F find_max(arr: *i64, len: i64) -> i64 {
    max := mut arr[0]
    i := mut 1
    L i < len {
        I arr[i] > max { max = arr[i] }
        i = i + 1
    }
    max
}

F main() -> i64 {
    arr: *i64 = [10, 20, 30, 40, 50]

    total := sum(arr, 5)        # 150
    max := find_max(arr, 5)     # 50

    puts("Sum: {total}")
    puts("Max: {max}")

    total + max     # 200
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
    name: 'Async/Await (syntax preview)',
    description: 'Asynchronous programming syntax — requires async runtime (compile-only)',
    code: `# Async functions with A keyword (syntax preview)
# Note: async codegen requires runtime support
# This example shows the syntax for future use

# A F fetch_data(id: i64) -> i64 {
#     id * 10
# }
#
# A F process() -> i64 {
#     a := Y fetch_data(1)    # Y = await
#     b := Y fetch_data(2)
#     a + b
# }

# Synchronous equivalent:
F fetch_data(id: i64) -> i64 = id * 10

F process() -> i64 {
    a := fetch_data(1)
    b := fetch_data(2)
    a + b
}

F main() -> i64 {
    result := process()
    puts("Result: {result}")    # 30
    result
}`
  },

  'ownership': {
    name: 'Ownership (syntax preview)',
    description: 'Move semantics — Vais tracks ownership to prevent use-after-move',
    code: `# Ownership: Vais prevents use-after-move errors
# Move semantics are always active

F double(x: i64) -> i64 = x * 2

F main() -> i64 {
    # Immutable binding
    a := 42
    result := double(a)
    puts("Result: {result}")

    # Mutable variables
    counter := mut 0
    L counter < 5 {
        counter = counter + 1
    }
    puts("Counter: {counter}")

    # Ownership ensures memory safety
    # Variables are moved when passed by value
    # Use mut for variables that need reassignment

    result + counter    # 89
}`
  },

  'wasm-interop': {
    name: 'WASM Interop (syntax preview)',
    description: 'WASM import/export attributes — use --target wasm32 to compile',
    code: `# WASM export example (syntax preview)
# Compile with: vaisc --target wasm32-unknown-unknown file.vais
#
# Import syntax:
#   #[wasm_import("env", "log")]
#   N F log(ptr: i64, len: i64) -> i64
#
# Export syntax:
#   #[wasm_export("add")]
#   F add(a: i64, b: i64) -> i64 = a + b

# These functions can be exported to WASM
F add(a: i64, b: i64) -> i64 = a + b

F fib(n: i64) -> i64 {
    I n <= 1 { n }
    E { @(n - 1) + @(n - 2) }
}

F main() -> i64 {
    result := add(10, 20)
    f := fib(10)
    puts("add(10,20) = {result}")
    puts("fib(10) = {f}")
    0
}`
  },

  'lambda-capture': {
    name: 'Lambda & Closures',
    description: 'Lambda expressions with type-annotated parameters',
    code: `# Lambda expressions
F square(x: i64) -> i64 = x * x

F main() -> i64 {
    # Lambda with type annotation
    double := |x: i64| x * 2
    add_ten := |x: i64| x + 10

    # Call lambdas directly
    a := double(5)         # 10
    b := add_ten(a)        # 20

    # Combine with regular functions
    c := square(b)         # 400

    puts("double(5) = {a}")
    puts("add_ten(10) = {b}")
    puts("square(20) = {c}")

    c    # 400
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
    name: 'Result Type',
    description: 'Error handling with Result type and pattern matching',
    code: `# Result type for error handling
E Result { Ok(i64), Err(i64) }

F safe_divide(a: i64, b: i64) -> Result {
    I b == 0 { R Err(-1) }
    Ok(a / b)
}

F main() -> i64 {
    # Match on successful division
    r1 := safe_divide(10, 2)
    v1 := M r1 {
        Ok(v) => {
            puts("Division OK")
            v
        },
        Err(e) => {
            puts("Division failed")
            e
        }
    }

    # Match on division by zero
    r2 := safe_divide(10, 0)
    v2 := M r2 {
        Ok(v) => v,
        Err(e) => {
            puts("Error: divide by zero")
            e
        }
    }

    puts("v1 = {v1}")     # 5
    puts("v2 = {v2}")     # -1
    v1 + v2               # 4
}`
  },

  'try-operator': {
    name: 'Try Operator (syntax preview)',
    description: 'Propagate errors with the ? operator — syntax preview',
    code: `# Try operator (?) for error propagation (syntax preview)
# The ? operator unwraps Ok or returns Err early
#
# F process(input: str) -> Result {
#     n := parse_number(input)?    # early return Err
#     result := validate(n)?       # early return Err
#     Ok(result)
# }

# Manual error propagation (equivalent pattern):
E Result { Ok(i64), Err(i64) }

F validate(n: i64) -> Result {
    I n < 0 { R Err(-1) }
    Ok(n * 2)
}

F process(n: i64) -> Result {
    r := validate(n)
    M r {
        Ok(v) => Ok(v + 1),
        Err(e) => Err(e)
    }
}

F main() -> i64 {
    # Success path
    r1 := process(5)
    v1 := M r1 { Ok(v) => v, Err(e) => e }
    puts("process(5) = {v1}")      # 11

    # Error path
    r2 := process(-3)
    v2 := M r2 { Ok(v) => v, Err(e) => e }
    puts("process(-3) = {v2}")     # -1

    v1 + v2    # 10
}`
  },

  'unwrap-operator': {
    name: 'Unwrap Operator (!)',
    description: 'Unwrap Ok values or panic with the ! operator',
    code: `# Unwrap operator (!) for assertive access
E Result { Ok(i64), Err(i64) }

F get_config() -> Result {
    Ok(42)
}

F compute() -> Result {
    Ok(100)
}

F main() -> i64 {
    # ! unwraps Ok, panics on Err
    config := get_config()!      # 42
    value := compute()!          # 100

    puts("config = {config}")
    puts("value = {value}")

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
  },

  'fizzbuzz': {
    name: 'FizzBuzz',
    description: 'Classic FizzBuzz with if-else chains',
    code: `# FizzBuzz — classic interview problem
F fizzbuzz(n: i64) -> i64 {
    L i:1..n+1 {
        I i % 15 == 0 {
            puts("FizzBuzz")
        } E I i % 3 == 0 {
            puts("Fizz")
        } E I i % 5 == 0 {
            puts("Buzz")
        } E {
            puts("{i}")
        }
    }
    0
}

F main() -> i64 {
    fizzbuzz(20)
}`
  },

  'binary-search': {
    name: 'Binary Search',
    description: 'Efficient search on sorted arrays',
    code: `# Binary search on a sorted array
F binary_search(arr: *i64, len: i64, target: i64) -> i64 {
    lo := mut 0
    hi := mut len - 1

    L lo <= hi {
        mid := lo + (hi - lo) / 2
        I arr[mid] == target { R mid }
        E I arr[mid] < target { lo = mid + 1 }
        E { hi = mid - 1 }
    }
    -1
}

F main() -> i64 {
    arr: *i64 = [2, 5, 8, 12, 16, 23, 38, 56, 72, 91]

    idx := binary_search(arr, 10, 23)
    puts("Search 23: index = {idx}")   # 5

    idx2 := binary_search(arr, 10, 42)
    puts("Search 42: index = {idx2}")  # -1

    0
}`
  },

  'bubble-sort': {
    name: 'Bubble Sort',
    description: 'Sorting algorithm with pointer arrays',
    code: `# Bubble sort implementation
F bubble_sort(arr: *i64, len: i64) -> i64 {
    i := mut 0
    L i < len - 1 {
        j := mut 0
        L j < len - 1 - i {
            I arr[j] > arr[j + 1] {
                tmp := arr[j]
                arr[j] = arr[j + 1]
                arr[j + 1] = tmp
            }
            j = j + 1
        }
        i = i + 1
    }
    0
}

F main() -> i64 {
    arr: *i64 = [64, 25, 12, 22, 11]

    puts("Before: 64 25 12 22 11")
    bubble_sort(arr, 5)

    puts("After:")
    L i:0..5 { puts("{arr[i]}") }

    0
}`
  },

  'gcd-lcm': {
    name: 'GCD & LCM',
    description: 'Euclidean algorithm with self-recursion',
    code: `# GCD using @ (self-recursion) and ternary
F gcd(a: i64, b: i64) -> i64 = b == 0 ? a : @(b, a % b)

F lcm(a: i64, b: i64) -> i64 = a / gcd(a, b) * b

F coprime(a: i64, b: i64) -> i64 = gcd(a, b) == 1 ? 1 : 0

F main() -> i64 {
    puts("gcd(48, 18) = {gcd(48, 18)}")   # 6
    puts("lcm(12, 18) = {lcm(12, 18)}")   # 36
    puts("coprime(15, 28) = {coprime(15, 28)}") # 1
    0
}`
  },

  'calculator-enum': {
    name: 'Calculator Enum',
    description: 'Enum variants with data and pattern matching',
    code: `# Calculator using enum with data
E Op {
    Add(i64, i64),
    Sub(i64, i64),
    Mul(i64, i64),
    Div(i64, i64)
}

F eval(op: Op) -> i64 {
    M op {
        Op.Add(a, b) => a + b,
        Op.Sub(a, b) => a - b,
        Op.Mul(a, b) => a * b,
        Op.Div(a, b) => {
            I b == 0 { R 0 }
            a / b
        }
    }
}

F main() -> i64 {
    puts("10 + 5 = {eval(Add(10, 5))}")
    puts("10 - 5 = {eval(Sub(10, 5))}")
    puts("6 * 7 = {eval(Mul(6, 7))}")
    puts("100 / 4 = {eval(Div(100, 4))}")
    0
}`
  },

  'state-machine': {
    name: 'State Machine',
    description: 'Traffic light simulation with enum transitions',
    code: `# Traffic light state machine
E Light { Red, Yellow, Green }

F next_state(current: Light) -> Light {
    M current {
        Light.Red => Green,
        Light.Green => Yellow,
        Light.Yellow => Red
    }
}

F duration(state: Light) -> i64 {
    M state {
        Light.Red => 3,
        Light.Green => 4,
        Light.Yellow => 1
    }
}

F main() -> i64 {
    state := mut Red
    total := mut 0

    L i:0..6 {
        ticks := duration(state)
        total = total + ticks
        state = next_state(state)
    }

    puts("6 transitions, total ticks = {total}")
    0
}`
  },

  'pipe-chain': {
    name: 'Pipe Chains',
    description: 'Functional data transformation with |>',
    code: `# Chaining transformations with pipe operator
F double(x: i64) -> i64 = x * 2
F add_one(x: i64) -> i64 = x + 1
F square(x: i64) -> i64 = x * x
F clamp(x: i64) -> i64 = I x > 100 { 100 } E { x }

F apply_n(f: |i64| -> i64, x: i64, n: i64) -> i64 {
    result := mut x
    L i:0..n { result = f(result) }
    result
}

F main() -> i64 {
    r1 := 5 |> double |> add_one
    puts("5 |> double |> add_one = {r1}")

    r2 := 3 |> square |> double |> add_one
    puts("3 |> square |> double |> add_one = {r2}")

    r3 := 8 |> square |> double |> clamp
    puts("8 |> square |> double |> clamp = {r3}")

    # Apply double 4 times to 1: 1->2->4->8->16
    r4 := apply_n(double, 1, 4)
    puts("double applied 4x to 1 = {r4}")

    0
}`
  },

  'matrix-ops': {
    name: 'Matrix Operations',
    description: '2x2 matrix math with pointer arrays',
    code: `# 2x2 matrix operations
F mat_mul(a: *i64, b: *i64, out: *i64) -> i64 {
    out[0] = a[0]*b[0] + a[1]*b[2]
    out[1] = a[0]*b[1] + a[1]*b[3]
    out[2] = a[2]*b[0] + a[3]*b[2]
    out[3] = a[2]*b[1] + a[3]*b[3]
    0
}

F mat_det(m: *i64) -> i64 = m[0]*m[3] - m[1]*m[2]
F mat_trace(m: *i64) -> i64 = m[0] + m[3]

F main() -> i64 {
    a: *i64 = [1, 2, 3, 4]
    b: *i64 = [5, 6, 7, 8]
    result: *i64 = [0, 0, 0, 0]

    mat_mul(a, b, result)

    det_a := mat_det(a)
    det_b := mat_det(b)
    det_ab := mat_det(result)

    puts("det(A) = {det_a}")
    puts("det(B) = {det_b}")
    puts("det(A*B) = {det_ab}")
    puts("det(A)*det(B) = {det_a * det_b}")

    0
}`
  },

  'linked-list': {
    name: 'Linked List',
    description: 'Manual memory management with malloc/free',
    code: `# Linked list with manual memory
# Node layout: [value:i64][next:i64] = 16 bytes

F node_new(value: i64) -> i64 {
    ptr := malloc(16)
    store_i64(ptr, value)
    store_i64(ptr + 8, 0)
    ptr
}

F node_value(n: i64) -> i64 = load_i64(n)
F node_next(n: i64) -> i64 = load_i64(n + 8)

F list_push(head: i64, value: i64) -> i64 {
    new_node := node_new(value)
    store_i64(new_node + 8, head)
    new_node
}

F list_sum(head: i64) -> i64 {
    total := mut 0
    cur := mut head
    L cur != 0 {
        total = total + node_value(cur)
        cur = node_next(cur)
    }
    total
}

F main() -> i64 {
    head := mut 0
    L i:1..6 { head = list_push(head, i) }

    total := list_sum(head)
    puts("Sum of 1..5 = {total}")   # 15
    0
}`
  },

  'vec-struct-access': {
    name: 'Vec<Struct> Direct Access',
    description: 'Phase 182: v[i].field direct field access on Vec of structs',
    code: `# Vec<Struct> direct field access (Phase 182)
# v[i].field pattern — no intermediate binding needed

S Point {
    x: i64,
    y: i64
}

F main() -> i64 {
    # Array of structs
    points: *Point = [
        Point { x: 1, y: 2 },
        Point { x: 3, y: 4 },
        Point { x: 5, y: 6 }
    ]

    # Direct field access via index: v[i].field
    puts("points[0] = ({points[0].x}, {points[0].y})")   # (1, 2)
    puts("points[1] = ({points[1].x}, {points[1].y})")   # (3, 4)
    puts("points[2] = ({points[2].x}, {points[2].y})")   # (5, 6)

    # Use in expressions directly
    sum_x := points[0].x + points[1].x + points[2].x
    sum_y := points[0].y + points[1].y + points[2].y
    puts("sum_x = {sum_x}")   # 9
    puts("sum_y = {sum_y}")   # 12

    # Access inside a loop
    total := mut 0
    L i:0..3 {
        total = total + points[i].x + points[i].y
    }
    puts("total = {total}")   # 21

    0
}`
  },

  'type-casting': {
    name: 'Type Safe Casting',
    description: 'Phase 158: explicit as keyword for all type conversions',
    code: `# Explicit type casting with the as keyword (Phase 158)
# Vais enforces strict type safety — no implicit coercions between
# int/float/bool. All conversions require the as keyword.

F main() -> i64 {
    # Integer to float conversion
    n: i64 = 42
    f: f64 = n as f64
    puts("i64 42 as f64 = {f}")   # 42

    # Float to integer (truncates)
    pi: f64 = 3.14159
    truncated: i64 = pi as i64
    puts("3.14159 as i64 = {truncated}")   # 3

    # Integer widening (also explicit for clarity)
    small: i32 = 100
    wide: i64 = small as i64
    puts("i32 100 as i64 = {wide}")   # 100

    # bool to integer (explicit required)
    flag: bool = true
    as_int: i64 = flag as i64
    puts("true as i64 = {as_int}")   # 1

    # Casting in arithmetic expressions
    a: i64 = 7
    b: i64 = 2
    ratio: f64 = a as f64 / b as f64
    puts("7.0 / 2.0 = {ratio}")   # 3.5

    0
}`
  },

  'vais-server-hello': {
    name: 'vais-server Hello World',
    description: 'Simple HTTP server with vais-server',
    code: `U core/app
U core/config
U core/context

C PORT: u16 = 8080

F handle_hello(ctx: Context) -> Response {
    ctx.text(200, "Hello from vais-server!")
}

F main() -> i64 {
    config := ServerConfig.default()
    app := mut App.new(config)
    app.get("/", "handle_hello")

    M app.listen(":{PORT}") {
        Ok(_) => { 0 },
        Err(e) => { println("Error: {e.message}"); 1 },
    }
}`
  },

  'vaisdb-query': {
    name: 'VaisDB Basic Query',
    description: 'SQL, vector, graph, and full-text search with VaisDB',
    code: `U vaisdb/client

F main() -> i64 {
    db := mut VaisDB.open("myapp.vaisdb")

    # SQL: 테이블 생성
    db.execute("CREATE TABLE users (id INT, name TEXT, bio TEXT)")
    db.execute("INSERT INTO users VALUES (1, 'Alice', 'Engineer')")

    # Vector: 시맨틱 검색
    db.execute("VECTOR_SEARCH(users, [0.1, 0.2, 0.3], 5)")

    # Graph: 관계 탐색
    db.execute("GRAPH_TRAVERSE('user_1', 'outbound', 2)")

    # Full-text: 텍스트 검색
    db.execute("FULLTEXT_MATCH(users, 'engineer')")

    0
}`
  },

  'fullstack': {
    name: 'Full-Stack Vais',
    description: 'vais-web + vais-server + vaisdb end-to-end example',
    code: `# Full-Stack Vais: vais-web + vais-server + vaisdb
# Frontend → API Server → Database

# --- Database Layer (VaisDB) ---
S Todo { id: i64, text: str, done: bool }

# --- API Server (vais-server) ---
U core/app
U db/query

F handle_list_todos(ctx: Context) -> Response {
    sql := QueryBuilder.new()
        .select("todos")
        .column("id").column("text").column("done")
        .order_by("id", SortDirection.Asc)
        .build()
    ctx.json(200, db.execute(sql))
}

F main() -> i64 {
    app := mut App.new(ServerConfig.default())
    app.get("/api/todos", "handle_list_todos")
    app.listen(":8080")
    0
}`
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
