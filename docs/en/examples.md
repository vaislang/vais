# Vais Examples

A collection of practical examples demonstrating Vais's features.

## Table of Contents

- [Basic Examples](#basic-examples)
- [Recursion](#recursion)
- [Collection Operations](#collection-operations)
- [Data Processing](#data-processing)
- [Algorithms](#algorithms)
- [Practical Applications](#practical-applications)
- [Running Examples](#running-examples)

---

## Basic Examples

### Hello World

```vais
print("Hello, World!")
```

### Variables and Expressions

```vais
// Variables
name = "Vais"
version = 1.0
is_ready = true

// Arithmetic
x = 10
y = 3
print("Add:", x + y)      // 13
print("Sub:", x - y)      // 7
print("Mul:", x * y)      // 30
print("Div:", x / y)      // 3
print("Mod:", x % y)      // 1

// String concatenation
greeting = "Hello, " ++ name ++ "!"
print(greeting)           // "Hello, Vais!"
```

### Functions

```vais
// Simple function
add(a, b) = a + b

// Function with multiple expressions
greet(name) = "Hello, " ++ name ++ "!"

// Higher-order function
apply_twice(f, x) = f(f(x))

double(x) = x * 2
print(apply_twice(double, 5))   // 20
```

---

## Recursion

### Factorial

```vais
// Using self-recursion ($)
factorial(n) = n < 2 ? 1 : n * $(n - 1)

print(factorial(5))   // 120
print(factorial(10))  // 3628800

// Calculate factorials 1-10
[1..11].@(factorial(_))
// [1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800]
```

### Fibonacci

```vais
// Recursive Fibonacci
fib(n) = n < 2 ? n : $(n - 1) + $(n - 2)

// First 15 Fibonacci numbers
[0..15].@(fib(_))
// [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377]

// Sum of first 10 Fibonacci numbers
[0..10].@(fib(_))./+(0, _ + _)   // 88
```

### GCD (Greatest Common Divisor)

```vais
gcd(a, b) = b == 0 ? a : $(b, a % b)

print(gcd(48, 18))    // 6
print(gcd(100, 35))   // 5
```

### Power Function

```vais
// Recursive power
power(base, exp) = exp == 0 ? 1 : base * $(base, exp - 1)

print(power(2, 10))   // 1024
print(power(3, 4))    // 81
```

---

## Collection Operations

### Map Examples

```vais
numbers = [1, 2, 3, 4, 5]

// Double each element
doubled = numbers.@(_ * 2)
print(doubled)   // [2, 4, 6, 8, 10]

// Square each element
squared = numbers.@(_ * _)
print(squared)   // [1, 4, 9, 16, 25]

// Convert to strings
strings = numbers.@(str(_))
print(strings)   // ["1", "2", "3", "4", "5"]

// Extract field from objects
users = [{name: "Alice", age: 30}, {name: "Bob", age: 25}]
names = users.@(_.name)
print(names)     // ["Alice", "Bob"]
```

### Filter Examples

```vais
numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

// Keep evens
evens = numbers.?(_ % 2 == 0)
print(evens)     // [2, 4, 6, 8, 10]

// Keep odds
odds = numbers.?(_ % 2 != 0)
print(odds)      // [1, 3, 5, 7, 9]

// Keep numbers > 5
large = numbers.?(_ > 5)
print(large)     // [6, 7, 8, 9, 10]

// Filter objects
users = [{name: "Alice", age: 30}, {name: "Bob", age: 17}]
adults = users.?(_.age >= 18)
print(adults)    // [{name: "Alice", age: 30}]
```

### Reduce Examples

```vais
numbers = [1, 2, 3, 4, 5]

// Sum
sum = numbers./+(0, _ + _)
print(sum)       // 15

// Product
product = numbers./*(1, _ * _)
print(product)   // 120

// Find max
max_val = numbers./(numbers[0], _ > _ ? _1 : _2)
print(max_val)   // 5

// Find min
min_val = numbers./(numbers[0], _ < _ ? _1 : _2)
print(min_val)   // 1

// Count elements
count = numbers./(0, _1 + 1)
print(count)     // 5

// Join strings
words = ["Hello", "World", "Vais"]
joined = words./("", _1 ++ " " ++ _2)
print(trim(joined))   // "Hello World Vais"
```

### Chaining Operations

```vais
// Pipeline: filter evens, square them, sum
result = [1..11]
    .?(_ % 2 == 0)      // [2, 4, 6, 8, 10]
    .@(_ * _)           // [4, 16, 36, 64, 100]
    ./+(0, _ + _)       // 220

print(result)

// Get names of adult users, uppercase
users = [
    {name: "alice", age: 30},
    {name: "bob", age: 17},
    {name: "charlie", age: 25}
]

adult_names = users
    .?(_.age >= 18)
    .@(_.name)
    .@(upper(_))

print(adult_names)   // ["ALICE", "CHARLIE"]
```

---

## Data Processing

### Statistics

```vais
data = [23, 45, 67, 12, 89, 34, 56, 78, 90, 11]

// Sum
total = data./+(0, _ + _)
print("Sum:", total)         // 505

// Average
avg = total / len(data)
print("Average:", avg)       // 50.5

// Min and Max
min_val = data./(data[0], _ < _ ? _1 : _2)
max_val = data./(data[0], _ > _ ? _1 : _2)
print("Min:", min_val)       // 11
print("Max:", max_val)       // 90

// Count above average
above_avg = data.?(_ > avg)
print("Above average:", len(above_avg))   // 5
```

### String Processing

```vais
text = "  Hello, World! Welcome to Vais.  "

// Trim whitespace
trimmed = trim(text)
print(trimmed)

// Split into words
words = split(trimmed, " ")
print(words)

// Count words
word_count = len(words.?(_ != ""))
print("Word count:", word_count)

// Convert to uppercase
upper_text = upper(text)
print(upper_text)

// Replace
replaced = replace(text, "Vais", "Rust")
print(replaced)
```

### JSON Processing

```vais
// Parse JSON
json_str = '{"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}'
data = json_parse(json_str)

// Extract users
users = json_get(data, "users")
print(users)

// Get all names
names = users.@(_.name)
print("Names:", names)

// Filter and transform
adults = users
    .?(_.age >= 18)
    .@({name: upper(_.name), adult: true})

print(json_stringify_pretty(adults))
```

---

## Algorithms

### Prime Numbers

```vais
// Check if number is prime
is_prime(n) = n < 2 ? false :
    n == 2 ? true :
    n % 2 == 0 ? false :
    check_divisors(n, 3)

check_divisors(n, i) =
    i * i > n ? true :
    n % i == 0 ? false :
    $(n, i + 2)

// Find all primes up to n
primes_up_to(n) = [2..n+1].?(is_prime(_))

print(primes_up_to(50))
// [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]
```

### Quicksort

```vais
quicksort(arr) =
    len(arr) <= 1 ? arr :
    let pivot = arr[0] in
    let less = tail(arr).?(_ < pivot) in
    let greater = tail(arr).?(_ >= pivot) in
    $(less) ++ [pivot] ++ $(greater)

unsorted = [64, 34, 25, 12, 22, 11, 90]
sorted = quicksort(unsorted)
print(sorted)   // [11, 12, 22, 25, 34, 64, 90]
```

### Binary Search

```vais
binary_search(arr, target) = search(arr, target, 0, len(arr) - 1)

search(arr, target, low, high) =
    low > high ? -1 :
    let mid = (low + high) / 2 in
    arr[mid] == target ? mid :
    arr[mid] < target ? $(arr, target, mid + 1, high) :
    $(arr, target, low, mid - 1)

sorted = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19]
print(binary_search(sorted, 7))    // 3
print(binary_search(sorted, 6))    // -1
```

### Merge Sort

```vais
merge_sort(arr) =
    len(arr) <= 1 ? arr :
    let mid = len(arr) / 2 in
    let left = $(take(arr, mid)) in
    let right = $(drop(arr, mid)) in
    merge(left, right)

merge(left, right) =
    len(left) == 0 ? right :
    len(right) == 0 ? left :
    head(left) <= head(right) ?
        [head(left)] ++ $(tail(left), right) :
        [head(right)] ++ $(left, tail(right))

arr = [38, 27, 43, 3, 9, 82, 10]
print(merge_sort(arr))   // [3, 9, 10, 27, 38, 43, 82]
```

---

## Practical Applications

### Temperature Converter

```vais
celsius_to_fahrenheit(c) = c * 9 / 5 + 32
fahrenheit_to_celsius(f) = (f - 32) * 5 / 9

// Convert temperatures
temps_c = [0, 10, 20, 30, 100]
temps_f = temps_c.@(celsius_to_fahrenheit(_))
print("Celsius:", temps_c)
print("Fahrenheit:", temps_f)
```

### Grade Calculator

```vais
grade(score) =
    score >= 90 ? "A" :
    score >= 80 ? "B" :
    score >= 70 ? "C" :
    score >= 60 ? "D" : "F"

scores = [95, 82, 76, 65, 58, 91, 73]
grades = scores.@(grade(_))
print(zip(scores, grades))

// Count each grade
count_grade(grades, g) = len(grades.?(_ == g))
print("A:", count_grade(grades, "A"))
print("B:", count_grade(grades, "B"))
print("C:", count_grade(grades, "C"))
```

### Shopping Cart

```vais
cart = [
    {name: "Apple", price: 1.50, qty: 4},
    {name: "Bread", price: 2.50, qty: 2},
    {name: "Milk", price: 3.00, qty: 1}
]

// Calculate line totals
line_totals = cart.@(_.price * _.qty)
print("Line totals:", line_totals)

// Total
total = line_totals./+(0, _ + _)
print("Total:", total)

// Apply 10% discount
discount = total * 0.10
final = total - discount
print("Discount:", discount)
print("Final:", final)
```

### Word Frequency Counter

```vais
text = "the quick brown fox jumps over the lazy dog the fox"

// Split into words
words = split(lower(text), " ")

// Get unique words
unique_words = unique(words)

// Count each word
count_word(word) = len(words.?(_ == word))

// Create frequency map
frequencies = unique_words.@({word: _, count: count_word(_)})

// Sort by count (descending)
sorted_freq = sort(frequencies.@(_.count))
print(reverse(sorted_freq))
```

### FizzBuzz

```vais
fizzbuzz(n) =
    n % 15 == 0 ? "FizzBuzz" :
    n % 3 == 0 ? "Fizz" :
    n % 5 == 0 ? "Buzz" :
    str(n)

// FizzBuzz 1-20
result = [1..21].@(fizzbuzz(_))
print(result)
```

### Palindrome Checker

```vais
is_palindrome(s) =
    let cleaned = lower(replace(s, " ", "")) in
    cleaned == reverse(cleaned)

print(is_palindrome("racecar"))        // true
print(is_palindrome("A man a plan"))   // false (spaces matter here)
print(is_palindrome("hello"))          // false
```

---

## Running Examples

Save any example to a `.vais` file and run:

```bash
# Run with interpreter
vais run example.vais

# Run with JIT (faster)
vais run example.vais --jit

# Interactive REPL
vais repl
```

---

## Related Documentation

- [Getting Started](./getting-started.md) - Installation and setup
- [Syntax Guide](./syntax.md) - Language syntax reference
- [API Reference](./api.md) - Built-in functions
