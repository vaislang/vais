# math-ext

Extended mathematical functions for Vais programs.

## Features

- Basic operations (abs, min, max, clamp)
- Number theory (gcd, lcm, is_prime)
- Power and exponentiation
- Fibonacci sequence
- Factorials and combinatorics
- Square root and perfect squares
- Digit manipulation
- Modular arithmetic

## Usage

```vais
U math-ext

F main() -> i64 {
    # Basic operations
    a := abs(-42)           # 42
    m := min(10, 20)        # 10
    x := clamp(150, 0, 100) # 100

    # Number theory
    g := gcd(48, 18)        # 6
    l := lcm(12, 18)        # 36

    # Power
    p := pow(2, 10)         # 1024
    pf := pow_fast(2, 20)   # 1048576

    # Prime check
    I is_prime(17) {
        puts_ptr("17 is prime")
    }

    # Fibonacci
    fib := fibonacci(10)    # 55

    # Factorial
    fact := factorial(5)    # 120

    # Square root
    root := isqrt(100)      # 10

    # Combinatorics
    c := binomial(5, 2)     # 10
    p := permutation(5, 2)  # 20

    0
}
```

## API

### Basic Operations

- `abs(x: i64) -> i64` - Absolute value
- `min(a: i64, b: i64) -> i64` - Minimum of two values
- `max(a: i64, b: i64) -> i64` - Maximum of two values
- `min3(a: i64, b: i64, c: i64) -> i64` - Minimum of three values
- `max3(a: i64, b: i64, c: i64) -> i64` - Maximum of three values
- `clamp(x: i64, lo: i64, hi: i64) -> i64` - Clamp to range [lo, hi]
- `sign(x: i64) -> i64` - Sign function (-1, 0, or 1)

### Number Theory

- `gcd(a: i64, b: i64) -> i64` - Greatest common divisor
- `lcm(a: i64, b: i64) -> i64` - Least common multiple
- `is_prime(n: i64) -> i64` - Check if prime
- `is_even(n: i64) -> i64` - Check if even
- `is_odd(n: i64) -> i64` - Check if odd

### Power and Root

- `pow(base: i64, exp: i64) -> i64` - Power (base^exp)
- `pow_fast(base: i64, exp: i64) -> i64` - Fast exponentiation by squaring
- `isqrt(n: i64) -> i64` - Integer square root (floor)
- `is_square(n: i64) -> i64` - Check if perfect square
- `mod_pow(base: i64, exp: i64, mod: i64) -> i64` - Modular exponentiation

### Sequences

- `fibonacci(n: i64) -> i64` - nth Fibonacci number
- `factorial(n: i64) -> i64` - Factorial (n!)
- `sum_to_n(n: i64) -> i64` - Sum 1+2+...+n
- `sum_of_squares(n: i64) -> i64` - Sum 1²+2²+...+n²

### Combinatorics

- `binomial(n: i64, k: i64) -> i64` - Binomial coefficient C(n,k)
- `permutation(n: i64, k: i64) -> i64` - Permutation P(n,k)

### Digit Operations

- `digit_count(n: i64) -> i64` - Number of digits
- `digit_sum(n: i64) -> i64` - Sum of digits
- `reverse_digits(n: i64) -> i64` - Reverse digits
- `is_palindrome(n: i64) -> i64` - Check if palindrome

## Examples

### Prime Numbers

```vais
F find_primes_up_to(n: i64) -> i64 {
    count := 0
    i := 2

    L i <= n {
        I is_prime(i) {
            printf("%d ", i)
            count = count + 1
        }
        i = i + 1
    }

    count
}

# Find all primes up to 100
count := find_primes_up_to(100)
printf("\nFound %d primes\n", count)
```

### GCD and LCM

```vais
# Simplify fraction
numerator := 48
denominator := 18
g := gcd(numerator, denominator)

simplified_num := numerator / g      # 8
simplified_den := denominator / g    # 3

printf("%d/%d simplifies to %d/%d\n",
    numerator, denominator,
    simplified_num, simplified_den)
```

### Fibonacci Sequence

```vais
# Print first 10 Fibonacci numbers
i := 0
L i < 10 {
    fib := fibonacci(i)
    printf("F(%d) = %d\n", i, fib)
    i = i + 1
}

# Output: 0, 1, 1, 2, 3, 5, 8, 13, 21, 34
```

### Combinatorics

```vais
# Calculate combinations
n := 5
k := 2
c := binomial(n, k)  # C(5,2) = 10
printf("C(%d,%d) = %d\n", n, k, c)

# Calculate permutations
p := permutation(n, k)  # P(5,2) = 20
printf("P(%d,%d) = %d\n", n, k, p)
```

### Power Calculation

```vais
# Regular power
result := pow(2, 10)         # 1024

# Fast power for large exponents
result = pow_fast(2, 30)     # 1073741824

# Modular exponentiation
result = mod_pow(2, 100, 1000)  # (2^100) % 1000
```

### Square Root

```vais
# Integer square root
n := 100
root := isqrt(n)  # 10

I is_square(n) {
    printf("%d is a perfect square: %d²\n", n, root)
}

# Binary search for closest square
target := 50
approx := isqrt(target)  # 7 (since 7² = 49)
```

### Digit Manipulation

```vais
n := 12321

# Count digits
count := digit_count(n)  # 5

# Sum digits
sum := digit_sum(n)  # 9

# Check palindrome
I is_palindrome(n) {
    puts_ptr("Number is palindrome")
}

# Reverse digits
rev := reverse_digits(n)  # 12321 (same, it's a palindrome)
```

## Algorithms

### Prime Check
Uses trial division with 6k±1 optimization up to √n.

### GCD
Euclidean algorithm with O(log min(a,b)) complexity.

### Fast Power
Exponentiation by squaring: O(log exp) instead of O(exp).

### Integer Square Root
Binary search algorithm with O(log n) complexity.

### Fibonacci
Iterative approach with O(n) time and O(1) space.

## Notes

- All functions work with signed 64-bit integers
- Negative exponents not supported (returns 0)
- Division by zero not checked
- Factorial overflows quickly (max ~20!)
- No floating-point operations

## Performance Tips

- Use `pow_fast()` instead of `pow()` for large exponents
- Use `mod_pow()` for modular exponentiation to prevent overflow
- GCD is efficient even for large numbers
- Prime checking is optimized but slow for very large numbers

## License

MIT
