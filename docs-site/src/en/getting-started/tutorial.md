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

## Next Steps

You now know the basics of Vais! Continue learning:

- [Language Specification](../language/language-spec.md) - Complete syntax reference
- [Standard Library](https://github.com/vaislang/vais/tree/main/std) - Explore built-in modules
- [Examples](https://github.com/vaislang/vais/tree/main/examples) - Real-world code samples
