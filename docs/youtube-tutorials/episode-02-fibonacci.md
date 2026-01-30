# Building X in Vais - Episode 02: Building a Fibonacci Calculator

**Duration:** 10 minutes
**Difficulty:** Beginner to Intermediate
**Series:** Building X in Vais

## Introduction

Welcome back! In Episode 01, we learned the basics of Vais. Now we're going to build something useful: a Fibonacci calculator. This is a perfect project to demonstrate Vais's elegant recursion model and compact syntax.

The Fibonacci sequence (0, 1, 1, 2, 3, 5, 8, 13, 21...) is where each number is the sum of the two preceding ones. We'll implement it multiple ways and compare approaches.

## Step 1: Classic Recursive Fibonacci (2 minutes)

Let's start with the most elegant solution using the `@` operator:

```vais
# Recursive Fibonacci with self-recursion operator
F fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)

F main() -> i64 {
    result := fib(10)
    puts("fib(10) =")
    result  # Returns 55
}
```

Breaking it down:
- `n < 2 ? n` - Base case: if n is 0 or 1, return n
- `: @(n - 1) + @(n - 2)` - Recursive case: sum of previous two
- `@` calls `fib` recursively without typing the name
- Entire function is one clean expression

This is incredibly concise! Compare to other languages - Vais achieves this in just one line.

## Step 2: Adding Output Formatting (2 minutes)

Let's make our calculator interactive and show multiple results:

```vais
F fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)

# Helper to print a number (digits by ASCII conversion)
F print_num(n: i64) -> i64 {
    I n >= 10 {
        print_num(n / 10)
        putchar((n % 10) + 48)
        0
    } E {
        putchar(n + 48)
        0
    }
}

# Print Fibonacci result with label
F print_fib(n: i64) -> i64 {
    puts("fib(")
    print_num(n)
    puts(") = ")
    print_num(fib(n))
    putchar(10)  # Newline
    0
}

F main() -> i64 {
    puts("=== Fibonacci Calculator ===")
    print_fib(0)
    print_fib(1)
    print_fib(5)
    print_fib(10)
    print_fib(15)
    puts("=== Complete ===")
    0
}
```

Notice how we build up functionality:
- `print_num` uses recursion to print each digit
- `putchar(n + 48)` converts numbers to ASCII characters
- `print_fib` combines everything for formatted output

## Step 3: Iterative Fibonacci (3 minutes)

The recursive version is elegant but inefficient for large numbers. Let's build an iterative version:

```vais
# Iterative Fibonacci using a helper with accumulation
F fib_iter_helper(n: i64, a: i64, b: i64, count: i64) -> i64 {
    I count == 0 {
        a
    } E {
        @(n, b, a + b, count - 1)
    }
}

F fib_iter(n: i64) -> i64 = fib_iter_helper(n, 0, 1, n)

F main() -> i64 {
    puts("Recursive fib(20):")
    result1 := fib(20)
    print_num(result1)
    putchar(10)

    puts("Iterative fib(20):")
    result2 := fib_iter(20)
    print_num(result2)
    putchar(10)

    0
}
```

How it works:
- `a` and `b` are the current two Fibonacci numbers
- `count` tracks how many iterations remain
- Each recursive call updates `a` and `b` to the next pair
- This is tail-recursive, which compilers can optimize

## Step 4: Building a Range Calculator (2 minutes)

Let's calculate multiple Fibonacci numbers in a range:

```vais
F fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)

F print_num(n: i64) -> i64 {
    I n >= 10 {
        print_num(n / 10)
        putchar((n % 10) + 48)
        0
    } E {
        putchar(n + 48)
        0
    }
}

# Calculate Fibonacci for range [start, end]
F fib_range(start: i64, end: i64) -> i64 {
    I start > end {
        0
    } E {
        puts("fib(")
        print_num(start)
        puts(") = ")
        print_num(fib(start))
        putchar(10)
        @(start + 1, end)
    }
}

F main() -> i64 {
    puts("=== Fibonacci Range: 0 to 10 ===")
    fib_range(0, 10)
    puts("=== Complete ===")
    0
}
```

Output will be:
```
=== Fibonacci Range: 0 to 10 ===
fib(0) = 0
fib(1) = 1
fib(2) = 1
fib(3) = 2
fib(4) = 3
fib(5) = 5
fib(6) = 8
fib(7) = 13
fib(8) = 21
fib(9) = 34
fib(10) = 55
=== Complete ===
```

## Step 5: Complete Calculator with Validation (1 minute)

Let's add input validation and make it robust:

```vais
F fib(n: i64) -> i64 = n < 2 ? n : @(n - 1) + @(n - 2)

F print_num(n: i64) -> i64 {
    I n >= 10 { print_num(n / 10); putchar((n % 10) + 48); 0 }
    E { putchar(n + 48); 0 }
}

# Safe Fibonacci with validation
F fib_safe(n: i64) -> i64 {
    I n < 0 {
        puts("Error: Negative input not allowed")
        0 - 1  # Return -1 for error
    } E I n > 40 {
        puts("Error: Input too large (max 40)")
        0 - 1
    } E {
        fib(n)
    }
}

F main() -> i64 {
    puts("=== Safe Fibonacci Calculator ===")

    # Valid input
    puts("fib_safe(10):")
    print_num(fib_safe(10))
    putchar(10)

    # Invalid: negative
    puts("fib_safe(-5):")
    print_num(fib_safe(0 - 5))
    putchar(10)

    # Invalid: too large
    puts("fib_safe(50):")
    print_num(fib_safe(50))
    putchar(10)

    0
}
```

## Key Takeaways

1. **Self-Recursion Power**: `@` makes recursive definitions incredibly clean
2. **Ternary Expressions**: `condition ? then : else` for compact logic
3. **Multiple Approaches**: Both recursive and iterative solutions are possible
4. **Composition**: Build complex programs from simple functions
5. **Validation**: Always validate inputs for robust code

## Performance Insights

- **Recursive**: Simple and elegant, but O(2^n) time for standard implementation
- **Iterative**: Efficient O(n) time with tail recursion optimization
- **For n > 30**: Use the iterative version or add memoization

## Next Episode Preview

In Episode 03, we'll explore structs and traits by building:
- A `Point` struct with methods
- A `Shape` trait for geometric calculations
- Implementations for Circle and Rectangle
- How to organize complex data with Vais's type system

## Try It Yourself

Challenges:
1. Implement memoized Fibonacci using an array
2. Calculate the sum of all Fibonacci numbers up to n
3. Find the first Fibonacci number greater than 1000
4. Implement the tribonacci sequence (sum of three previous numbers)

## Resources

- Episode code: `examples/fib.vais`
- Quickstart: `docs/QUICKSTART.md`
- Full tutorial: `docs/TUTORIAL.md`

---

See you in Episode 03 where we dive into structs and traits!
