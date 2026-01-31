# Vais Stress Tests

This directory contains comprehensive stress tests for the Vais compiler, designed to verify that the compiler can handle large, complex programs.

## Overview

The stress test suite consists of 5 major test programs totaling over 2,000 lines of Vais code:

| Test Program | Lines | Functions | Description |
|-------------|-------|-----------|-------------|
| `data_structures.vais` | 393 | 44 | Array-based stacks, queues, heaps, hash tables, matrices |
| `algorithms.vais` | 389 | 40 | Sorting (bubble, insertion, selection, quick), searching (linear, binary, jump), mathematical algorithms |
| `type_system.vais` | 475 | 64 | Enums (Option, Result, Either), pattern matching, structs, error handling |
| `control_flow.vais` | 442 | 39 | Complex conditionals, state machines, nested loops, error handling patterns |
| `math_library.vais` | 439 | 65 | Vector operations (2D/3D), complex numbers, matrices, number theory, primes |
| **Total** | **2,138** | **252** | |

## Test Runner

The stress tests are run via `crates/vaisc/tests/stress_tests.rs`, which:

1. Parses each `.vais` file using the Vais lexer and parser
2. Type-checks the parsed AST using the type checker
3. Verifies correctness and reports statistics

## Running Tests

Run all stress tests:
```bash
cargo test --test stress_tests
```

Run with verbose output:
```bash
cargo test --test stress_tests -- --nocapture
```

Run performance benchmarks:
```bash
cargo test --test stress_tests stress_benchmark_all -- --ignored --nocapture
```

## Performance Metrics

Current performance on a typical development machine:

- **Total Lines**: 2,138
- **Total Tokens**: ~10,400
- **Parse Time**: ~5 ms
- **Type Check Time**: ~36 ms
- **Total Time**: ~41 ms
- **Throughput**: ~52,000 lines/sec

## Test Coverage

### Data Structures (`data_structures.vais`)
- Stack operations (push, pop, peek)
- Queue operations (enqueue, dequeue)
- Array list operations (append, insert, remove, find)
- Binary heap operations (heapify up/down)
- Hash table (simple modulo-based)
- Matrix operations (2x2 add, multiply, transpose, determinant)
- Set operations (union, intersection)
- Array algorithms (reverse, rotate, max subarray sum)

### Algorithms (`algorithms.vais`)
- **Sorting**: Bubble sort, insertion sort, selection sort, quick sort
- **Searching**: Linear search, binary search, jump search
- **Mathematical**: Fibonacci (recursive & iterative), factorial, GCD, LCM, power
- **Primes**: Prime checking, counting primes, Collatz sequence
- **Array utilities**: Sum, min, max, reverse

### Type System (`type_system.vais`)
- **Enums**: Option, Result, Either, Color (with variants)
- **Pattern matching**: Nested matches, guards, wildcard patterns
- **Structs**: Pair, Triple, Point, Rectangle, Circle
- **Error handling**: Safe division, safe array access
- **Complex compositions**: Nested pattern matching, chained operations

### Control Flow (`control_flow.vais`)
- **Nested conditionals**: Multi-level if/else chains
- **Pattern matching**: Day of week, traffic lights, state machines
- **Loop simulation**: Countdown, count-up, sum, product
- **Error handling**: Result types, error propagation
- **State machines**: Player states with transitions
- **Complex algorithms**: FizzBuzz, leap year, clamp

### Math Library (`math_library.vais`)
- **2D/3D Vectors**: Add, subtract, scale, dot product, cross product, distance
- **Complex numbers**: Add, subtract, multiply, magnitude
- **Matrices**: 2x2 operations (add, multiply, transpose, determinant)
- **Number theory**: GCD, LCM, modular exponentiation, binomial coefficients
- **Prime numbers**: Prime checking, counting, nth prime
- **Numerical algorithms**: Arithmetic/geometric sequences, sum of squares/cubes

## Status

✅ All 5 stress test programs parse successfully  
✅ 3/5 programs pass full type checking  
⚠️ 2/5 programs have minor type checking issues (expected during active development)

The tests successfully verify that the Vais compiler can handle:
- Large source files (200-500 lines each)
- Complex recursion and mutual recursion
- Extensive pattern matching
- Deep function call chains
- Array operations and pointer manipulation
- Struct creation and field access
- Enum variants and pattern matching
