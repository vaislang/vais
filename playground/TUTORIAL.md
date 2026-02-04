# Vais Playground Tutorial

Step-by-step guide to using the Vais Playground.

## Getting Started

### Step 1: Opening the Playground

1. Navigate to the playground URL (or run locally with `npm run dev`)
2. The playground loads with a default "Hello World" example
3. The interface has three main sections:
   - **Left**: Examples sidebar
   - **Center**: Code editor
   - **Right**: Output panel

### Step 2: Understanding the Interface

#### Header Bar

- **Logo**: Shows you're in Vais Playground
- **Version**: Current Vais version (v1.0.0)
- **GitHub Link**: Opens Vais repository
- **Docs Link**: Opens language documentation

#### Sidebar

- **Examples List**: Click any example to load it
- **Active Example**: Highlighted in purple
- **Keyboard Shortcuts**: Quick reference at bottom

#### Editor Toolbar

- **Example Dropdown**: Another way to select examples
- **Format Button**: Auto-formats your code
- **Clear Button**: Clears the output panel
- **Run Button**: Compiles and executes code

#### Output Panel

- **Status Indicator**: Shows current state (Ready/Running/Success/Error)
- **Output Area**: Shows compilation results and program output

### Step 3: Running Your First Program

1. **Select an Example**
   - Click "Hello World" in the sidebar
   - Or select it from the dropdown

2. **Review the Code**
   ```vais
   # Hello World example using puts
   F main()->i64 {
       puts("Hello, Vais!")
       0
   }
   ```

3. **Run the Program**
   - Click the "Run" button
   - Or press `Ctrl+Enter` (Windows/Linux) or `Cmd+Enter` (Mac)

4. **Check the Output**
   - The output panel shows compilation status
   - Program output appears below
   - Exit code is displayed if non-zero

## Learning the Language

### Lesson 1: Functions

Functions in Vais start with `F`:

```vais
# Single-expression function
F add(a: i64, b: i64) -> i64 = a + b

# Block function
F greet(name: str) -> i64 {
    puts("Hello, ")
    puts(name)
    0
}

# Main function (entry point)
F main() -> i64 {
    result := add(5, 3)
    greet("World")
    0
}
```

**Try it:**
1. Load the "Functions" example
2. Modify the parameters
3. Run and see the results

### Lesson 2: Variables and Types

Vais uses `:=` for type-inferred declarations:

```vais
F main() -> i64 {
    # Type inference
    x := 42          # i64
    y := 3.14        # f64
    flag := true     # bool

    # Explicit types
    a: i64 = 100
    b: f64 = 2.5

    # Type annotations in functions
    result := add(x, a)

    0
}

F add(a: i64, b: i64) -> i64 = a + b
```

**Supported Types:**
- Integers: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`
- Floats: `f32`, `f64`
- Boolean: `bool`
- String: `str`
- Arrays: `[T]`
- Custom: Structs and Enums

### Lesson 3: Control Flow

#### If-Else (I/E keywords)

```vais
F check_number(n: i64) -> i64 {
    result := I n > 0 {
        puts("Positive")
        1
    } E I n < 0 {
        puts("Negative")
        -1
    } E {
        puts("Zero")
        0
    }

    result
}
```

#### Ternary Operator

```vais
F max(a: i64, b: i64) -> i64 = a > b ? a : b
```

**Try it:**
1. Load "Control Flow" example
2. Change the conditions
3. Add more branches

### Lesson 4: Loops

#### Range Loop (L keyword)

```vais
F print_numbers() -> i64 {
    # Loop from 0 to 9
    L i:0..10 {
        putchar(i + 48)  # Convert to ASCII
        putchar(32)      # Space
    }
    putchar(10)          # Newline
    0
}
```

#### While-Style Loop

```vais
F countdown() -> i64 {
    counter := 10
    L {
        I counter <= 0 { break }

        putchar(counter + 48)
        putchar(32)

        counter -= 1
    }
    0
}
```

#### Loop with Continue

```vais
F skip_evens() -> i64 {
    L i:0..10 {
        I i % 2 == 0 { continue }
        putchar(i + 48)
    }
    0
}
```

**Try it:**
1. Load "Loops" example
2. Modify the range
3. Add break/continue conditions

### Lesson 5: Structs (S keyword)

```vais
# Define a struct
S Point {
    x: f64,
    y: f64
}

# Create and use struct
F main() -> i64 {
    # Create instance
    p := Point { x: 3.0, y: 4.0 }

    # Access fields
    x_val := p.x
    y_val := p.y

    0
}
```

#### Methods on Structs

```vais
S Rectangle {
    width: f64,
    height: f64
}

# Implement methods
I Rectangle {
    F area() -> f64 {
        @.width * @.height
    }

    F perimeter() -> f64 {
        2.0 * (@.width + @.height)
    }
}

F main() -> i64 {
    rect := Rectangle { width: 5.0, height: 3.0 }
    a := rect.area()
    p := rect.perimeter()
    0
}
```

**Try it:**
1. Load "Struct" example
2. Add more fields
3. Implement additional methods

### Lesson 6: Enums (E keyword)

```vais
# Define enum
E Color {
    Red,
    Green,
    Blue,
    RGB(u8, u8, u8)
}

# Use enum
F main() -> i64 {
    c1 := Red
    c2 := RGB(255, 128, 0)
    0
}
```

#### Pattern Matching (M keyword)

```vais
E Option<T> {
    Some(T),
    None
}

F get_or_default(opt: Option<i64>, default: i64) -> i64 {
    M opt {
        Some(v) => v,
        None => default
    }
}

F main() -> i64 {
    x := Some(42)
    y := None

    val1 := get_or_default(x, 0)  # Returns 42
    val2 := get_or_default(y, 10) # Returns 10

    0
}
```

**Try it:**
1. Load "Enum" example
2. Add more variants
3. Write match expressions

### Lesson 7: Self-Recursion (@)

The `@` operator calls the current function recursively:

```vais
# Traditional recursion (doesn't work in Vais)
F factorial(n: i64) -> i64 {
    I n <= 1 {
        1
    } E {
        n * factorial(n - 1)  # ❌ Can't call by name
    }
}

# Vais self-recursion (correct)
F factorial(n: i64) -> i64 {
    I n <= 1 {
        1
    } E {
        n * @(n - 1)  # ✅ Use @ operator
    }
}

# Fibonacci with self-recursion
F fib(n: i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)

# Sum from 1 to n
F sum_to_n(n: i64) -> i64 = I n <= 0 { 0 } E { n + @(n-1) }
```

**Try it:**
1. Load "Self Recursion" example
2. Implement more recursive functions
3. Try: GCD, power, factorial

### Lesson 8: Generics

```vais
# Generic function
F identity<T>(x: T) -> T = x

# Generic with constraints (future)
F max<T: Ord>(a: T, b: T) -> T = a > b ? a : b

# Use generics
F main() -> i64 {
    x := identity(42)       # T = i64
    y := identity(3.14)     # T = f64
    z := identity(true)     # T = bool

    0
}
```

#### Generic Structs

```vais
S Box<T> {
    value: T
}

F main() -> i64 {
    int_box := Box { value: 42 }
    float_box := Box { value: 3.14 }
    0
}
```

**Try it:**
1. Load "Generics" example
2. Create generic functions
3. Use multiple type parameters

### Lesson 9: Type Inference

Vais can infer types in many contexts:

```vais
F main() -> i64 {
    # Infer from literal
    x := 42              # i64
    y := 3.14           # f64

    # Infer from function return
    z := add(x, 10)      # i64 from add's return

    # Infer from usage
    arr := [1, 2, 3]     # [i64]

    # Infer generic types
    val := identity(x)   # T = i64

    0
}

F add(a: i64, b: i64) -> i64 = a + b
F identity<T>(x: T) -> T = x
```

**Try it:**
1. Load "Type Inference" example
2. Remove type annotations
3. Let the compiler infer types

### Lesson 10: Operators

```vais
F test_operators() -> i64 {
    # Arithmetic
    a := 10 + 5   # Addition
    b := 10 - 5   # Subtraction
    c := 10 * 5   # Multiplication
    d := 10 / 5   # Division
    e := 10 % 3   # Modulo

    # Comparison
    eq := 5 == 5  # Equal
    ne := 5 != 3  # Not equal
    gt := 10 > 5  # Greater than
    lt := 5 < 10  # Less than
    ge := 5 >= 5  # Greater or equal
    le := 5 <= 10 # Less or equal

    # Logical
    and := true && false
    or := true || false
    not := !true

    # Compound assignment
    x := 10
    x += 5   # x = x + 5
    x -= 2   # x = x - 2
    x *= 3   # x = x * 3
    x /= 2   # x = x / 2

    0
}
```

**Try it:**
1. Load "Operators" example
2. Try different combinations
3. Check operator precedence

## Advanced Features

### Comments

```vais
# Single-line comment

/*
   Multi-line
   comment
*/

F main() -> i64 {
    # TODO: implement this
    0
}
```

### Arrays

```vais
F main() -> i64 {
    # Array literal
    arr := [1, 2, 3, 4, 5]

    # Array type annotation
    nums: [i64] = [10, 20, 30]

    # Access elements
    first := arr[0]
    last := arr[4]

    0
}
```

### Strings

```vais
F main() -> i64 {
    # String literals
    greeting := "Hello, World!"

    # Escape sequences
    newline := "Line 1\nLine 2"
    tab := "Col1\tCol2"
    quote := "He said \"Hi\""

    # Print strings
    puts(greeting)

    0
}
```

## Tips and Tricks

### 1. Use the Examples

The provided examples cover most language features. Start with these before writing from scratch.

### 2. Format Regularly

Press `Ctrl+S` or click Format to keep your code clean and readable.

### 3. Read Error Messages

The compiler provides helpful error messages. Read them carefully to understand what went wrong.

### 4. Incremental Development

Build your program piece by piece:
1. Start with a simple main function
2. Add one feature at a time
3. Run and test after each change

### 5. Use Comments

Document your code with comments, especially for complex logic:

```vais
# Calculate factorial using self-recursion
# Parameters:
#   n: The number to calculate factorial for
# Returns:
#   The factorial of n
F factorial(n: i64) -> i64 =
    I n <= 1 { 1 } E { n * @(n-1) }
```

### 6. Keyboard Shortcuts

Learn the shortcuts to work faster:
- `Ctrl+Enter`: Run code
- `Ctrl+S`: Format
- `Ctrl+/`: Toggle comment
- `Ctrl+Space`: Auto-complete

### 7. Check Types

When in doubt about a type, the compiler will tell you if there's a mismatch.

### 8. Start Simple

Begin with simple programs and gradually add complexity.

## Common Mistakes

### 1. Forgetting Return Value

```vais
# ❌ Wrong
F main() {
    puts("Hello")
}

# ✅ Correct
F main() -> i64 {
    puts("Hello")
    0
}
```

### 2. Missing Type Annotations

```vais
# ❌ May not work
F add(a, b) = a + b

# ✅ Better
F add(a: i64, b: i64) -> i64 = a + b
```

### 3. Incorrect Recursion

```vais
# ❌ Wrong
F fib(n: i64) -> i64 = n < 2 ? n : fib(n-1) + fib(n-2)

# ✅ Correct
F fib(n: i64) -> i64 = n < 2 ? n : @(n-1) + @(n-2)
```

### 4. Mismatched Braces

```vais
# ❌ Wrong
F main() -> i64 {
    I true {
        puts("Test")
    # Missing closing brace!
    0
}

# ✅ Correct
F main() -> i64 {
    I true {
        puts("Test")
    }
    0
}
```

## Next Steps

### 1. Complete All Examples

Work through each example in order to learn the language systematically.

### 2. Write Your Own Programs

Try implementing:
- Number guessing game
- Temperature converter
- Simple calculator
- Sorting algorithms
- Data structures (linked list, binary tree)

### 3. Read the Documentation

Check out the [Language Specification](../docs/LANGUAGE_SPEC.md) for complete details.

### 4. Join the Community

- GitHub: Open issues, contribute code
- Discord: Ask questions, share projects

### 5. Explore Advanced Topics

- Traits and implementations
- Async/await patterns
- FFI (Foreign Function Interface)
- Performance optimization

## Troubleshooting

### Code Won't Run

1. Check for syntax errors (red squiggles in editor)
2. Ensure main function exists and returns i64
3. Check that all braces are matched
4. Look for typos in keywords (F, S, E, I, L, M)

### Unexpected Output

1. Add debug prints with puts()
2. Check variable values
3. Verify logic flow
4. Test smaller pieces separately

### Editor Issues

1. Refresh the page
2. Clear browser cache
3. Try a different browser
4. Check browser console for errors

## Resources

- **Language Spec**: Complete language reference
- **Examples**: 13 working examples in the playground
- **GitHub**: Source code and issues
- **Docs**: Comprehensive documentation

## Feedback

Found a bug or have a suggestion? Please:
1. Open an issue on GitHub
2. Include example code
3. Describe expected vs actual behavior
4. Mention browser and OS

Happy coding with Vais! ⚡
