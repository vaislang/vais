# Rosetta Code Algorithm Examples - Vais Language

This directory contains implementations of classic algorithms in the Vais programming language. These examples demonstrate the syntax and capabilities of Vais while following the Rosetta Code tradition of implementing the same algorithms across different programming languages.

## Algorithm Index

### 1. Fibonacci (`fibonacci.vais`)
Computes the nth Fibonacci number using recursion. Demonstrates the `@` operator for self-recursion and base case handling with conditional statements.

**Key features:**
- Recursive function definition
- Self-recursion with `@` operator
- Conditional logic with `I` statement

### 2. Factorial (`factorial.vais`)
Calculates n! using recursion. Shows how to handle recursive base cases and mathematical operations.

**Key features:**
- Recursive multiplication
- Self-recursion with `@` operator
- Integer arithmetic

### 3. FizzBuzz (`fizzbuzz.vais`)
Classic FizzBuzz algorithm that prints numbers 1-100 with special rules for multiples of 3, 5, and 15.

**Key features:**
- Loop with `L` statement
- Modulo operations
- Nested conditional logic
- String output with `puts()`

### 4. GCD - Euclidean Algorithm (`gcd.vais`)
Computes the Greatest Common Divisor using the efficient Euclidean algorithm with recursion.

**Key features:**
- Multi-parameter recursion
- Self-recursion with `@` operator
- Modulo operation for algorithm implementation

### 5. Primality Test (`is_prime.vais`)
Checks if a number is prime by testing divisibility up to the square root.

**Key features:**
- Boolean return type
- Loop termination condition
- Mutable variable binding with `mut`
- Efficient prime checking algorithm

### 6. Bubble Sort (`bubble_sort.vais`)
Sorts an array in ascending order using the bubble sort algorithm.

**Key features:**
- Array pointers (`*i64`)
- Nested loops
- Array indexing and mutation
- Function that operates on arrays

### 7. Binary Search (`binary_search.vais`)
Searches for a target value in a sorted array using divide-and-conquer recursion.

**Key features:**
- Recursive search algorithm
- Array indexing
- Integer division
- Return -1 for "not found"

### 8. Towers of Hanoi (`towers_of_hanoi.vais`)
Classic recursive puzzle solution that prints the moves needed to solve the puzzle.

**Key features:**
- Deep recursion (3+ levels)
- Self-recursion with `@` operator
- Problem decomposition
- Output formatting

### 9. Palindrome Check (`palindrome.vais`)
Checks if a string is a palindrome by comparing characters from both ends.

**Key features:**
- String handling
- Character access
- Bidirectional iteration
- Boolean logic

### 10. Collatz Conjecture (`collatz.vais`)
Generates the Collatz sequence starting from a number and counts the steps to reach 1.

**Key features:**
- Loop with exit condition
- Modulo and arithmetic operations
- Mutable state tracking
- Sequence generation

## Vais Language Syntax Reference

### Basic Syntax
- **Functions:** `F name(param: type) -> return_type { ... }`
- **Structs:** `S name { field: type }`
- **Enums:** `E name { Variant1, Variant2 }`
- **Loops:** `L init; condition; update { ... }`
- **Conditionals:** `I condition { ... } { ... }`
- **Pattern Matching:** `M value { pattern => action }`
- **Return:** `R value`

### Operators and Keywords
- **Self-recursion:** `@` (calls the current function)
- **Variable binding:** `:=` (immutable) or `mut` (mutable)
- **Ternary:** `condition ? true_value : false_value`

### Types
- **Integers:** `i64`
- **Floats:** `f64`
- **Booleans:** `bool`
- **Strings:** `str`
- **Pointers:** `*i64` (and other types)
- **Arrays:** `arr: *i64 = [1, 2, 3]`

### I/O Functions
- **Print with format:** `println("text {}", value)`
- **Print string:** `puts("string")`
- **Print character:** `putchar(ch)`

### Main Function
Every executable Vais program requires a `main()` function that returns an `i64`:
```vais
F main() -> i64 {
    // program code
    R 0  // return success
}
```

## Running the Examples

To compile and run any example:
```bash
vais compile fibonacci.vais
./fibonacci
```

Or directly:
```bash
vais run fibonacci.vais
```

## References

- [Rosetta Code](https://rosettacode.org/) - The original source of many algorithm examples
- Vais Language Documentation

