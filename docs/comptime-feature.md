# Compile-Time Evaluation (comptime) Feature

## Overview

The `comptime` feature allows Vais code to execute expressions at compile time, enabling powerful metaprogramming capabilities and compile-time optimizations. This is inspired by Zig's comptime feature.

## Syntax

### Basic comptime Expression

```vais
C ARRAY_SIZE = comptime { 4 * 8 }
```

### comptime Block

```vais
F calculate_hash()->i64 = comptime {
    x := 5381
    L i:0..10 {
        x = x * 33 + i
    }
    x
}
```

### comptime Function (Future Feature)

```vais
comptime F factorial(n: i64) -> i64 = I n <= 1 { 1 } E { n * @(n - 1) }
```

## Features

### Supported Operations

The comptime evaluator supports the following operations:

1. **Arithmetic Operations**
   - Integer: `+`, `-`, `*`, `/`, `%`
   - Float: `+`, `-`, `*`, `/`
   - Bitwise: `&`, `|`, `^`, `<<`, `>>`

2. **Logical Operations**
   - Boolean: `&&`, `||`, `!`

3. **Comparison Operations**
   - `<`, `<=`, `>`, `>=`, `==`, `!=`

4. **Control Flow**
   - Conditionals: `I cond { ... } E { ... }`
   - Ternary: `cond ? then : else`
   - Loops: `L var:range { ... }`

5. **Variables**
   - Local variable bindings: `x := value`
   - Variable reassignment: `x = new_value`

### Restrictions

The following are NOT supported in comptime blocks (to ensure purity):

- I/O operations (file operations, printing, etc.)
- Memory allocation (heap allocations)
- External function calls (except pure, compile-time evaluable functions)
- Mutable global state
- Side effects

## Implementation

### Architecture

The comptime feature is implemented in several layers:

1. **Lexer** (`vais-lexer`)
   - Added `comptime` keyword token

2. **AST** (`vais-ast`)
   - Added `Expr::Comptime { body: Box<Spanned<Expr>> }` variant

3. **Parser** (`vais-parser`)
   - Parses `comptime { expr }` syntax
   - Validates block structure

4. **Evaluator** (`vais-types/src/comptime.rs`)
   - Interprets AST at compile time
   - Evaluates expressions to concrete values
   - Maintains compile-time scope and variable bindings

5. **Type Checker** (`vais-types`)
   - Evaluates comptime expressions during type checking
   - Verifies type consistency
   - Returns the type of the evaluated result

6. **Code Generator** (`vais-codegen`)
   - Replaces comptime expressions with their evaluated constants
   - Emits LLVM IR for constant values

### Evaluation Process

```
Source Code → Parse → Type Check (Evaluate comptime) → Codegen (Emit constant)
```

Example:

```vais
F test()->i64 = comptime { 4 * 8 }
```

1. Parser creates: `Expr::Comptime { body: Binary { op: Mul, left: 4, right: 8 } }`
2. Type checker evaluates: `ComptimeValue::Int(32)`
3. Type checker returns: `ResolvedType::I64`
4. Codegen emits: `32` (constant in LLVM IR)

## Examples

### Example 1: Simple Arithmetic

```vais
F array_size()->i64 = comptime { 4 * 8 }
# Evaluates to: F array_size()->i64 = 32
```

### Example 2: Loop Calculation

```vais
F compute_hash()->i64 = comptime {
    x := 5381
    L i:0..10 {
        x = x * 33 + i
    }
    x
}
# Evaluates to a constant computed at compile time
```

### Example 3: Conditional Logic

```vais
F get_config()->i64 = comptime {
    debug := true
    I debug {
        100
    } E {
        50
    }
}
# Evaluates to: F get_config()->i64 = 100
```

### Example 4: Factorial

```vais
F factorial_five()->i64 = comptime {
    n := 5
    result := 1
    L i:1..=n {
        result = result * i
    }
    result
}
# Evaluates to: F factorial_five()->i64 = 120
```

### Example 5: Power Calculation

```vais
F power_of_two()->i64 = comptime {
    base := 2
    exp := 10
    result := 1
    L i:0..exp {
        result = result * base
    }
    result
}
# Evaluates to: F power_of_two()->i64 = 1024
```

## Use Cases

### 1. Constant Array Sizes

```vais
C SIZE = comptime { calculate_optimal_size() }
arr: [i64; SIZE]
```

### 2. Configuration Values

```vais
C MAX_CONNECTIONS = comptime {
    I is_production() { 1000 } E { 10 }
}
```

### 3. Compile-Time Hashing

```vais
C STRING_HASH = comptime { hash("my_constant_string") }
```

### 4. Type-Level Computation

```vais
C ELEMENT_SIZE = comptime { size_of::<MyType>() }
C ARRAY_ELEMENTS = comptime { BUFFER_SIZE / ELEMENT_SIZE }
```

## Performance Benefits

1. **Zero Runtime Cost**: Comptime expressions are evaluated during compilation, resulting in zero runtime overhead.

2. **Optimization**: The compiler can better optimize code with compile-time constants.

3. **Compile-Time Validation**: Errors in comptime expressions are caught at compile time, not runtime.

4. **Code Size Reduction**: Eliminates need for runtime calculation code.

## Error Handling

Comptime evaluation can fail with the following errors:

- **Type Mismatch**: Incompatible types in operations
- **Division by Zero**: Attempting to divide by zero at compile time
- **Undefined Variable**: Using an undeclared variable
- **Overflow**: Integer overflow in arithmetic operations
- **Non-Pure Operation**: Attempting I/O or other impure operations

Example error:

```vais
F test()->i64 = comptime {
    x := 10
    y := 0
    x / y  # Error: division by zero at compile time
}
```

## Future Enhancements

1. **Comptime Functions**: Functions marked as `comptime` that can only be called at compile time

2. **Comptime Reflection**: Access to type information at compile time

3. **Comptime String Manipulation**: String operations in comptime blocks

4. **Comptime Type Generation**: Generate types based on comptime calculations

5. **Comptime Assertions**: Static assertions that must hold at compile time

```vais
# Future syntax
comptime_assert(SIZE > 0, "Size must be positive")
```

## Comparison with Other Languages

### Zig

Vais comptime is inspired by Zig's comptime:

```zig
// Zig
const size = comptime calculateSize();
```

```vais
// Vais
C size = comptime { calculateSize() }
```

### C++ constexpr

Similar to C++ constexpr but with a more explicit syntax:

```cpp
// C++
constexpr int factorial(int n) {
    return n <= 1 ? 1 : n * factorial(n - 1);
}
```

```vais
// Vais
comptime F factorial(n: i64) -> i64 = I n <= 1 { 1 } E { n * @(n - 1) }
```

### Rust const fn

Similar to Rust's const fn:

```rust
// Rust
const fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

```vais
// Vais
comptime F add(a: i64, b: i64) -> i64 = a + b
```

## Testing

Comprehensive test suite in `examples/comptime_test.vais` covers:

- Simple arithmetic
- Variables and assignments
- Loops and iterations
- Conditionals
- Bitwise operations
- Boolean logic
- Float arithmetic
- Nested expressions

Run tests with:

```bash
cargo test --lib -p vais-types comptime
```

## References

- [Zig comptime documentation](https://ziglang.org/documentation/master/#comptime)
- [C++ constexpr reference](https://en.cppreference.com/w/cpp/language/constexpr)
- [Rust const fn](https://doc.rust-lang.org/reference/const_eval.html)
