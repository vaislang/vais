# Building X in Vais - Episode 04: Pattern Matching and Enums

**Duration:** 10 minutes
**Difficulty:** Intermediate
**Series:** Building X in Vais

## Introduction

Welcome to Episode 04! Pattern matching is one of the most powerful features in modern programming languages. In Vais, we use `M` for match expressions and `E` for enum definitions. Together, they enable elegant handling of different states and values.

Today we'll build:
- Color enums with RGB codes
- Option types for handling missing values
- Result types for error handling
- A state machine for a traffic light system

Let's dive in!

## Step 1: Basic Pattern Matching (2 minutes)

The `M` keyword introduces a match expression:

```vais
# Match on simple values
F describe_number(n: i64) -> i64 {
    M n {
        0 => 100,
        1 => 200,
        2 => 300,
        _ => 999  # Wildcard catches everything else
    }
}

F main() -> i64 {
    result := describe_number(1)
    puts("describe_number(1):")
    putchar((result / 100) + 48)
    putchar(((result / 10) % 10) + 48)
    putchar((result % 10) + 48)
    putchar(10)  # 200

    fallback := describe_number(5)
    puts("describe_number(5):")
    putchar((fallback / 100) + 48)
    putchar(((fallback / 10) % 10) + 48)
    putchar((fallback % 10) + 48)
    putchar(10)  # 999

    0
}
```

Key concepts:
- `M value { ... }` starts a match expression
- `pattern => result` for each case
- `_` is the wildcard that matches anything
- All cases must be covered (exhaustive matching)

## Step 2: Defining Enums (2 minutes)

Enums represent a value that can be one of several variants:

```vais
# Simple enum without data
E Color {
    Red,
    Green,
    Blue
}

# Convert color to RGB code
F color_to_code(c: Color) -> i64 {
    M c {
        Red => 255,
        Green => 65280,
        Blue => 16711680
    }
}

F main() -> i64 {
    red := Red
    green := Green

    red_code := color_to_code(red)
    green_code := color_to_code(green)

    puts("Red code:")
    putchar((red_code / 100) + 48)
    putchar(((red_code / 10) % 10) + 48)
    putchar((red_code % 10) + 48)
    putchar(10)  # 255

    puts("Green code: 65280")

    0
}
```

Breakdown:
- `E Color { ... }` defines an enum
- Variants are just names: `Red`, `Green`, `Blue`
- Create enum values directly: `red := Red`
- Match exhaustively handles all variants

## Step 3: Enums with Data (2 minutes)

Enums can carry data in their variants:

```vais
# Option type - represents optional values
E Option {
    None,
    Some(i64)
}

# Safely unwrap Option with default
F unwrap_or(opt: Option, default: i64) -> i64 {
    M opt {
        Some(x) => x,        # Bind the value to x
        None => default
    }
}

F main() -> i64 {
    # Option with a value
    opt1 := Some(42)
    value1 := unwrap_or(opt1, 0)

    puts("opt1 value:")
    putchar((value1 / 10) + 48)
    putchar((value1 % 10) + 48)
    putchar(10)  # 42

    # Option without a value
    opt2 := None
    value2 := unwrap_or(opt2, 99)

    puts("opt2 value:")
    putchar((value2 / 10) + 48)
    putchar((value2 % 10) + 48)
    putchar(10)  # 99

    0
}
```

Pattern binding:
- `Some(x) => x` extracts the value into variable `x`
- `None => default` matches the empty case
- The extracted value can be used in the result expression

## Step 4: Result Type for Error Handling (2 minutes)

Result types handle operations that might fail:

```vais
E Result {
    Ok(i64),
    Err(i64)  # Error code
}

# Safe division
F divide(a: i64, b: i64) -> Result {
    I b == 0 {
        Err(1)  # Error code 1 = division by zero
    } E {
        Ok(a / b)
    }
}

# Handle Result and return value or error code
F process_result(res: Result) -> i64 {
    M res {
        Ok(value) => value,
        Err(code) => {
            puts("Error occurred!")
            0 - code  # Return negative error code
        }
    }
}

F main() -> i64 {
    puts("=== Division Test ===")

    # Successful division
    result1 := divide(10, 2)
    value1 := process_result(result1)
    puts("10 / 2 = 5")

    # Division by zero
    result2 := divide(10, 0)
    value2 := process_result(result2)
    puts("10 / 0 = Error!")

    0
}
```

Result pattern:
- `Ok(value)` for successful operations
- `Err(code)` for failures with error information
- Pattern matching forces you to handle both cases

## Step 5: Building a State Machine (3 minutes)

Let's build a traffic light controller using enums and pattern matching:

```vais
# Traffic light states
E TrafficLight {
    Red,
    Yellow,
    Green
}

# Get next state in the cycle
F next_state(current: TrafficLight) -> TrafficLight {
    M current {
        Red => Green,
        Green => Yellow,
        Yellow => Red
    }
}

# Get duration for each light (in seconds)
F get_duration(light: TrafficLight) -> i64 {
    M light {
        Red => 30,
        Yellow => 5,
        Green => 25
    }
}

# Should pedestrians wait?
F should_wait(light: TrafficLight) -> i64 {
    M light {
        Red => 0,      # Can cross
        Yellow => 1,   # Wait
        Green => 1     # Wait
    }
}

# Simulate traffic light cycle
F simulate_cycle(light: TrafficLight, count: i64) -> i64 {
    I count == 0 {
        puts("Simulation complete")
        0
    } E {
        duration := get_duration(light)
        wait := should_wait(light)

        M light {
            Red => puts("RED - Pedestrians can cross"),
            Yellow => puts("YELLOW - Prepare to stop"),
            Green => puts("GREEN - Cars go")
        };

        next := next_state(light)
        @(next, count - 1)
    }
}

F main() -> i64 {
    puts("=== Traffic Light Simulation ===")

    # Start at red, simulate 6 state changes
    simulate_cycle(Red, 6)

    puts("=== Simulation End ===")
    0
}
```

This demonstrates:
- State transitions with pattern matching
- Multiple functions operating on the same enum
- Using recursion to iterate through states
- Clean, readable state machine logic

## Step 6: Complex Matching with Multiple Variants (1 minute)

Enums can have many variants with different data:

```vais
E Message {
    Quit,
    Move(i64, i64),
    Write(i64),
    ChangeColor(i64, i64, i64)
}

F handle_message(msg: Message) -> i64 {
    M msg {
        Quit => {
            puts("Quitting...")
            0
        },
        Move(x, y) => {
            puts("Moving to position")
            x + y
        },
        Write(val) => {
            puts("Writing value")
            val
        },
        ChangeColor(r, g, b) => {
            puts("Changing color")
            r + g + b
        }
    }
}

F main() -> i64 {
    msg1 := Move(10, 20)
    msg2 := Write(42)
    msg3 := Quit

    handle_message(msg1)
    handle_message(msg2)
    handle_message(msg3)

    0
}
```

## Key Takeaways

1. **Pattern Matching (`M`)**: Elegant way to handle different cases
2. **Enums (`E`)**: Define types with multiple variants
3. **Data in Variants**: Enums can carry associated data
4. **Binding**: Extract data with pattern `Variant(x) => x`
5. **Exhaustiveness**: Compiler ensures all cases are covered
6. **State Machines**: Perfect use case for enums + matching

## Common Patterns

- **Option Type**: Handle missing values safely
- **Result Type**: Handle errors explicitly
- **State Machines**: Model system states and transitions
- **Message Passing**: Define protocol messages as enum variants

## Next Episode Preview

In Episode 05, we'll explore closures and async programming:
- Defining closures with `|x| expr` syntax
- Capturing variables from outer scope
- Async functions with the `A` keyword
- Using `.await` to handle async operations
- Building concurrent programs with `spawn`

## Try It Yourself

Challenges:
1. Create a `Direction` enum (North, South, East, West) with turn functions
2. Build a `FileOperation` enum with Read, Write, Delete variants
3. Implement a simple calculator that returns `Result<i64, String>`
4. Create a game state machine (Menu, Playing, Paused, GameOver)

## Resources

- Example: `examples/pattern_match_test.vais`
- Example: `examples/enum_test.vais`
- Example: `examples/option_test.vais`
- Tutorial: `docs/TUTORIAL.md` (Pattern Matching section)

---

See you in Episode 05 where we explore closures and async!
