// Example code snippets for the playground
export const examples = {
  'hello-world': {
    name: 'Hello World',
    description: 'Simple Hello World program',
    code: `# Hello World example using puts
F main()->i64 {
    puts("Hello, Vais!")
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
    puts("1 + 2 = {1 + 2}")

    # Escaped braces
    puts("Use {{braces}} for literal braces")

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

  'tilde-mut': {
    name: 'Tilde Mut (~)',
    description: 'Shorthand ~ for mut keyword',
    code: `# ~ is shorthand for mut
F main() -> i64 {
    # Traditional mutable
    counter := mut 0

    # ~ shorthand (equivalent)
    ~ total := 0

    L i: 0..10 {
        counter = counter + 1
        total = total + i
    }

    0
}`
  },

  'destructuring': {
    name: 'Destructuring',
    description: 'Tuple destructuring with :=',
    code: `# Tuple destructuring example
F get_pair() -> (i64, i64) = (10, 20)

F swap(a: i64, b: i64) -> (i64, i64) = (b, a)

F main() -> i64 {
    # Destructure tuple
    (x, y) := get_pair()

    # Destructure swap result
    (a, b) := swap(1, 2)

    0
}`
  },

  'type-infer-params': {
    name: 'Type Inference',
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
    code: `# Minimal Vais program
F main() -> i64 = 0`
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
