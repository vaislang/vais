# Developer Experience (DX) Feedback Report

## Project: Math CLI & Data Processing Tool in Vais
## Date: 2026-01-29

---

## Overview

Two real-world CLI tools were implemented in Vais to validate language capabilities:
1. **Math CLI** - Fibonacci, factorial, primes, GCD/LCM, power, sqrt
2. **Data Processing** - Array statistics (sum, min, max, mean, count)

Both projects compiled and ran successfully with correct results.

---

## What Works Well

### Strengths
- **Self-recursion operator `@`**: Elegant and concise for recursive algorithms
- **Expression-oriented syntax**: `F func(x: i64) -> i64 = expr` is very clean
- **Ternary operator**: Natural for simple conditionals
- **If/Else blocks (`I`/`E`)**: Readable control flow
- **Array literals**: `[10, 20, 30]` works as expected
- **Multi-digit number output**: Can be built from `putchar()` primitives
- **Compilation speed**: Very fast compilation pipeline
- **Error messages**: Clear error reporting when syntax is wrong

### Language Design Wins
- Functional style with expression-oriented design encourages immutability
- `@` operator eliminates boilerplate in recursive functions
- Minimal syntax overhead for common operations

---

## Issues Found

### Critical DX Issues

1. **No `print()` or `printf()` with formatting**
   - `puts()` adds a newline, making inline output impossible
   - Must use `putchar()` one character at a time for formatted output
   - Had to write custom `print_num()` function to output integers
   - **Impact**: Every project needs to reinvent string/number output
   - **Suggestion**: Add `print()` (no newline), `println()` (with newline), and format strings

2. **No string concatenation or formatting**
   - Cannot combine strings and numbers in output
   - Labels like "fib(10) = 55" require separate puts/print_num calls
   - **Suggestion**: Add string interpolation or `printf()`-style formatting

3. **No mutable array element assignment**
   - Cannot do `arr[i] = value` to mutate array elements
   - Makes sorting algorithms, in-place processing impossible
   - **Impact**: Can only create arrays, not modify them
   - **Suggestion**: Support `arr[i] = expr` syntax for mutable arrays

4. **No loop constructs in practice**
   - While `L` (loop) syntax exists in parser, it doesn't work reliably in E2E
   - All iteration must be done via recursion
   - Tail-call optimization not guaranteed, so deep recursion may stack overflow
   - **Impact**: Simple counting loops require verbose recursive functions
   - **Suggestion**: Ensure `L` loops compile to correct LLVM IR

### Moderate DX Issues

5. **No `bool` type**
   - Must use `i64` with 0/1 convention for boolean logic
   - `is_prime()` returns `i64` instead of `bool`
   - **Suggestion**: Add first-class `bool` type

6. **No `void` return type**
   - Functions that only produce side effects (output) must return `i64`
   - **Suggestion**: Allow `F print_stuff() { ... }` without return type

7. **Struct limitations**
   - Cannot pass structs as function arguments (codegen pointer mismatch)
   - No nested struct field access (`a.b.c`)
   - **Impact**: Structs are only useful for local data grouping
   - **Suggestion**: Fix struct value passing in codegen

8. **No standard library integration**
   - `U math` import doesn't provide usable functions in compiled output
   - stdlib modules exist but aren't linkable in the compile pipeline
   - **Impact**: Every project must implement basic utilities from scratch

### Minor DX Issues

9. **No comments after expressions on same line**
   - Code organization relies on separate comment lines

10. **No multi-file projects**
    - Cannot split code across multiple `.vais` files
    - All code must be in a single file

11. **No generic types in practice**
    - `array_sum`, `array_min`, `array_max` all hardcoded to `i64`
    - Generic definitions exist but aren't usable in compiled code

---

## Missing stdlib Functions Identified

| Function | Category | Priority |
|----------|----------|----------|
| `print(str)` | I/O | Critical |
| `print_int(n)` | I/O | Critical |
| `println(str)` | I/O | Critical |
| `format(str, args...)` | I/O | High |
| `strlen(str)` | String | High |
| `strcmp(a, b)` | String | High |
| `atoi(str)` | Conversion | High |
| `itoa(n)` | Conversion | High |
| `array_len(arr)` | Array | High |
| `malloc(size)` / `free(ptr)` | Memory | Medium |
| `read_line()` | I/O | Medium |
| `exit(code)` | System | Medium |
| `time()` | System | Low |
| `rand()` | Math | Low |

---

## Performance Observations

- **Compilation**: Near-instant for single files (~10ms)
- **Fibonacci(15)**: Runs correctly but naive recursion is slow for larger values
- **Prime counting to 100**: Fast enough for demo purposes
- **No observable issues** with the recursive approach for small datasets

---

## Recommendations for Language Improvement

### Priority 1 (Blocks real-world use)
- [ ] Add `print()` without newline
- [ ] Add integer-to-string formatting
- [ ] Fix mutable array element assignment
- [ ] Ensure loop constructs work in codegen

### Priority 2 (Improves developer experience)
- [ ] Add `bool` type
- [ ] Fix struct passing to functions
- [ ] Enable stdlib linking in compile pipeline
- [ ] Support multi-file projects

### Priority 3 (Nice to have)
- [ ] Generic type instantiation in codegen
- [ ] String type with operations
- [ ] Command-line argument parsing (argc/argv)
- [ ] File I/O integration
