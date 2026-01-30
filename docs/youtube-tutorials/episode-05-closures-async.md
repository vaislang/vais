# Building X in Vais - Episode 05: Closures and Async Programming

**Duration:** 10 minutes
**Difficulty:** Intermediate to Advanced
**Series:** Building X in Vais

## Introduction

Welcome to the final episode of our introductory series! Today we're diving into two powerful features that make Vais a modern language: closures and async programming.

Closures let you create anonymous functions that capture their environment. Async programming enables concurrent execution without blocking. Together, they unlock powerful programming patterns.

We'll build:
- Function generators using closures
- Event handlers with captured state
- Async task processors
- Concurrent computations with spawn/await

Let's get started!

## Step 1: Basic Closures (2 minutes)

Closures are anonymous functions defined with `|params| expression` syntax:

```vais
F main() -> i64 {
    puts("=== Basic Closures ===")

    # Simple closure that adds 10
    add_ten := |x: i64| x + 10

    result := add_ten(5)
    puts("add_ten(5) = 15")

    # Closure with multiple parameters
    multiply := |a: i64, b: i64| a * b

    product := multiply(6, 7)
    puts("multiply(6, 7) = 42")

    0
}
```

Key points:
- `|x: i64|` defines parameters with types
- `x + 10` is the closure body (single expression)
- Assign closures to variables like any value
- Call closures like regular functions

## Step 2: Capturing Variables (2 minutes)

Closures can capture variables from their surrounding scope:

```vais
F main() -> i64 {
    puts("=== Closure Capture ===")

    # Capture a multiplier from outer scope
    multiplier := 10
    scale := |x: i64| x * multiplier

    result1 := scale(5)
    puts("scale(5) with multiplier=10:")
    putchar((result1 / 10) + 48)
    putchar((result1 % 10) + 48)
    putchar(10)  # 50

    # Capture multiple variables
    base := 20
    offset := 3
    compute := |x: i64| base + x + offset

    result2 := compute(7)
    puts("compute(7) with base=20, offset=3:")
    putchar((result2 / 10) + 48)
    putchar((result2 % 10) + 48)
    putchar(10)  # 30

    0
}
```

This demonstrates:
- Closures access variables from outer scopes
- Multiple variables can be captured
- Captured values are used in the closure body
- Creates a "closure" over the environment

## Step 3: Higher-Order Functions (2 minutes)

Functions that take or return closures are higher-order functions:

```vais
# Apply a function to a value twice
F apply_twice(f: |i64| -> i64, x: i64) -> i64 {
    result1 := f(x)
    f(result1)
}

# Create a function that adds n
F make_adder(n: i64) -> |i64| -> i64 {
    |x: i64| x + n
}

F main() -> i64 {
    puts("=== Higher-Order Functions ===")

    # Double a value twice
    double := |x: i64| x * 2
    result := apply_twice(double, 5)
    puts("apply_twice(double, 5) = 20")

    # Create custom adders
    add_five := make_adder(5)
    add_ten := make_adder(10)

    r1 := add_five(10)  # 15
    r2 := add_ten(10)   # 20

    puts("add_five(10) = 15")
    puts("add_ten(10) = 20")

    0
}
```

Patterns shown:
- Functions accepting closure parameters
- Functions returning closures
- Factory functions that create customized closures
- Composition of operations

## Step 4: Async Functions Basics (2 minutes)

Async functions are marked with `A` and return values that can be awaited:

```vais
# Simple async function
A F compute(x: i64) -> i64 {
    x * 2
}

# Another async function
A F add_values(a: i64, b: i64) -> i64 {
    a + b
}

F main() -> i64 {
    puts("=== Async Basics ===")

    # Call async function and await result
    result := compute(21).await

    puts("compute(21).await =")
    putchar((result / 10) + 48)
    putchar((result % 10) + 48)
    putchar(10)  # 42

    # Chain async calls
    sum := add_values(10, 5).await

    puts("add_values(10, 5).await =")
    putchar((sum / 10) + 48)
    putchar((sum % 10) + 48)
    putchar(10)  # 15

    puts("Async complete!")
    0
}
```

Understanding async:
- `A F` marks a function as async
- `.await` blocks until the result is ready
- Async functions can do work concurrently
- Return values are wrapped in futures

## Step 5: Concurrent Execution with Spawn (2 minutes)

Use `spawn` to run tasks concurrently:

```vais
# Async task that computes a value
A F task1() -> i64 {
    puts("Task 1 running...")
    100
}

A F task2() -> i64 {
    puts("Task 2 running...")
    200
}

A F task3() -> i64 {
    puts("Task 3 running...")
    300
}

F main() -> i64 {
    puts("=== Concurrent Tasks ===")

    # Spawn tasks to run concurrently
    t1 := spawn task1()
    t2 := spawn task2()
    t3 := spawn task3()

    # All three tasks run concurrently
    puts("All tasks spawned, waiting for results...")

    # Await results
    r1 := t1.await
    r2 := t2.await
    r3 := t3.await

    total := r1 + r2 + r3

    puts("Total from all tasks:")
    putchar((total / 100) + 48)
    putchar(((total / 10) % 10) + 48)
    putchar((total % 10) + 48)
    putchar(10)  # 600

    puts("=== All Tasks Complete ===")
    0
}
```

Concurrency model:
- `spawn` starts a task without blocking
- Multiple tasks can run at the same time
- `.await` waits for a specific task to complete
- Enables parallel computation

## Step 6: Complete Example - Async Data Pipeline (2 minutes)

Let's build a data processing pipeline combining closures and async:

```vais
# Async function to fetch data
A F fetch_data(id: i64) -> i64 {
    puts("Fetching data...")
    id * 10
}

# Async function to process data
A F process_data(data: i64) -> i64 {
    puts("Processing data...")
    data * 2
}

# Async function to save result
A F save_result(result: i64) -> i64 {
    puts("Saving result...")
    result + 100
}

# Pipeline coordinator
A F run_pipeline(id: i64) -> i64 {
    # Sequential async operations
    data := fetch_data(id).await
    processed := process_data(data).await
    saved := save_result(processed).await
    saved
}

# Process multiple items concurrently
F process_batch(ids: *i64, count: i64) -> i64 {
    puts("=== Batch Processing ===")

    # Spawn pipelines for each ID
    task1 := spawn run_pipeline(1)
    task2 := spawn run_pipeline(2)
    task3 := spawn run_pipeline(3)

    # Wait for all pipelines
    r1 := task1.await  # 1 * 10 * 2 + 100 = 120
    r2 := task2.await  # 2 * 10 * 2 + 100 = 140
    r3 := task3.await  # 3 * 10 * 2 + 100 = 160

    total := r1 + r2 + r3
    puts("Total results:")
    putchar((total / 100) + 48)
    putchar(((total / 10) % 10) + 48)
    putchar((total % 10) + 48)
    putchar(10)  # 420

    puts("=== Batch Complete ===")
    0
}

F main() -> i64 {
    ids: *i64 = [1, 2, 3]
    process_batch(ids, 3)
    0
}
```

This demonstrates:
- Async functions calling other async functions
- Sequential async operations with `.await`
- Concurrent batch processing with `spawn`
- Real-world async pipeline pattern

## Step 7: Closures + Async Combination (bonus)

Combine closures with async for powerful abstractions:

```vais
# Higher-order async function
A F map_async(f: |i64| -> i64, x: i64) -> i64 {
    result := f(x)
    result * 2
}

F main() -> i64 {
    puts("=== Closures + Async ===")

    # Closure to add 10
    add_ten := |x: i64| x + 10

    # Pass closure to async function
    result := map_async(add_ten, 5).await

    puts("map_async(add_ten, 5).await:")
    putchar((result / 10) + 48)
    putchar((result % 10) + 48)
    putchar(10)  # 30 = (5 + 10) * 2

    0
}
```

## Key Takeaways

1. **Closures**: Anonymous functions with `|params| expr` syntax
2. **Capture**: Closures can access outer scope variables
3. **Higher-Order**: Functions can take/return closures
4. **Async (`A F`)**: Mark functions for async execution
5. **Await**: Use `.await` to get async results
6. **Spawn**: Launch concurrent tasks that run in parallel
7. **Composition**: Combine closures and async for powerful abstractions

## When to Use What

**Closures:**
- Callbacks and event handlers
- Custom operations for map/filter/reduce
- Configuration and customization
- Temporary function definitions

**Async/Await:**
- I/O operations (file, network)
- Long-running computations
- Concurrent task coordination
- Event-driven architectures

## Series Wrap-Up

Congratulations! You've completed the "Building X in Vais" introductory series:

- **Episode 01**: Hello World and basics
- **Episode 02**: Fibonacci and recursion
- **Episode 03**: Structs and traits
- **Episode 04**: Pattern matching and enums
- **Episode 05**: Closures and async

You now have the foundation to build real projects in Vais!

## Try It Yourself

Challenges:
1. Build a `map` function that applies a closure to an array
2. Create an async web request simulator with timeouts
3. Implement a event handler system using closures
4. Build a concurrent file processor with spawn/await
5. Create a task scheduler with priority queues

## Advanced Topics to Explore

- Generics with closures
- Trait objects for dynamic dispatch
- Advanced async patterns (select, join)
- Memory management with Rc and Box
- Standard library collections

## Resources

- Closures: `examples/closure_simple.vais`, `examples/closure_test.vais`
- Async: `examples/async_test.vais`, `examples/spawn_test.vais`
- Tutorial: `docs/TUTORIAL.md`
- Async Guide: `docs/async_tutorial.md`
- Standard Library: `docs/STDLIB.md`

---

Thank you for watching "Building X in Vais"! Happy coding with Vais!

**Next Steps:**
- Explore the examples directory
- Build your own projects
- Join the Vais community on GitHub
- Share what you build!
