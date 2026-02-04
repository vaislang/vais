# Option API Reference

> Represents an optional value: `Some(T)` or `None`

## Import

```vais
U std/option
```

## Overview

The `Option<T>` type is used to represent a value that may or may not be present. It is a generic enum with two variants:
- `Some(T)`: Contains a value of type `T`
- `None`: Represents the absence of a value

This is useful for functions that may fail to produce a value, or for representing nullable values in a type-safe way.

## Enum Definition

```vais
E Option<T> {
    None,
    Some(T)
}
```

## Methods

### is_some

```vais
F is_some(&self) -> i64
```

Check if the option contains a value.

**Returns:** `1` if `Some`, `0` if `None`

**Example:**
```vais
x := Some(42)
I x.is_some() == 1 {
    # x contains a value
}
```

---

### is_none

```vais
F is_none(&self) -> i64
```

Check if the option is empty.

**Returns:** `1` if `None`, `0` if `Some`

**Example:**
```vais
y := None
I y.is_none() == 1 {
    # y is empty
}
```

---

### unwrap_or

```vais
F unwrap_or(&self, default: T) -> T
```

Extract the value from the option, or return a default value if `None`.

**Parameters:**
- `default`: Value to return if the option is `None`

**Returns:** The contained value if `Some`, otherwise `default`

**Example:**
```vais
x := Some(42)
y := None

val1 := x.unwrap_or(0)  # val1 = 42
val2 := y.unwrap_or(0)  # val2 = 0
```

## Usage Examples

### Basic Pattern Matching

```vais
U std/option

F divide(a: i64, b: i64) -> Option<i64> {
    I b == 0 {
        None
    } E {
        Some(a / b)
    }
}

F main() -> i64 {
    result := divide(10, 2)

    M result {
        Some(v) => v,      # Returns 5
        None => 0          # Returns 0 on division by zero
    }
}
```

### Using Methods

```vais
U std/option

F main() -> i64 {
    x := Some(42)
    y := None

    # Check if option has a value
    I x.is_some() == 1 {
        # Process x
    }

    I y.is_none() == 1 {
        # Handle empty case
    }

    # Safe unwrap with default
    val := y.unwrap_or(10)  # val = 10

    0
}
```

### Optional Function Return Values

```vais
U std/option

F find_first_positive(arr: i64, len: i64) -> Option<i64> {
    i := 0
    L i < len {
        val := load_i64(arr + i * 8)
        I val > 0 {
            R Some(val)
        }
        i = i + 1
    }
    None
}

F main() -> i64 {
    arr := malloc(5 * 8)
    store_i64(arr, -1)
    store_i64(arr + 8, 0)
    store_i64(arr + 16, 5)
    store_i64(arr + 24, 10)
    store_i64(arr + 32, -3)

    result := find_first_positive(arr, 5)

    M result {
        Some(v) => v,   # Returns 5
        None => -1      # Returns -1 if no positive found
    }

    free(arr)
    0
}
```

### Chaining Optional Operations

```vais
U std/option

F safe_divide(a: i64, b: i64) -> Option<i64> {
    I b == 0 { None } E { Some(a / b) }
}

F main() -> i64 {
    # First division
    step1 := safe_divide(100, 5)  # Some(20)

    # Process result
    final := M step1 {
        Some(v) => safe_divide(v, 4),  # Some(5)
        None => None
    }

    # Extract with default
    answer := M final {
        Some(v) => v,
        None => 0
    }

    # answer = 5
    0
}
```

### Using Options with User Input

```vais
U std/option
U std/io

F parse_positive_int() -> Option<i64> {
    num := read_i64()
    I num > 0 {
        Some(num)
    } E {
        None
    }
}

F main() -> i64 {
    result := parse_positive_int()

    I result.is_some() == 1 {
        val := result.unwrap_or(0)
        # Process valid input
    } E {
        # Handle invalid input
    }

    0
}
```

### Generic Option with Different Types

```vais
U std/option

F main() -> i64 {
    # Option<i64>
    int_opt := Some(42)
    int_val := int_opt.unwrap_or(0)

    # Option<f64>
    float_opt := Some(3.14)
    float_val := float_opt.unwrap_or(0.0)

    # Option can work with any type T
    0
}
```

## Best Practices

1. **Use pattern matching** for explicit handling of both cases
2. **Use `unwrap_or`** when you have a sensible default value
3. **Use `is_some`/`is_none`** for conditional checks before unwrapping
4. **Return `Option<T>`** instead of using sentinel values (like -1 or null) to indicate failure
5. **Prefer Option over nullable pointers** for type safety
