# Math API Reference

> Mathematical functions, constants, and trigonometry

## Import

```vais
U std/math
```

## Constants

| Name | Type | Value |
|------|------|-------|
| `PI` | `f64` | 3.141592653589793 |
| `EULER` | `f64` | 2.718281828459045 |
| `TAU` | `f64` | 6.283185307179586 |

## Functions

### Basic Arithmetic

| Function | Signature | Description |
|----------|-----------|-------------|
| `abs` | `F abs(x: f64) -> f64` | Absolute value (f64) |
| `abs_i64` | `F abs_i64(x: i64) -> i64` | Absolute value (i64) |
| `min` | `F min(a: f64, b: f64) -> f64` | Minimum of two f64 |
| `max` | `F max(a: f64, b: f64) -> f64` | Maximum of two f64 |
| `min_i64` | `F min_i64(a: i64, b: i64) -> i64` | Minimum of two i64 |
| `max_i64` | `F max_i64(a: i64, b: i64) -> i64` | Maximum of two i64 |
| `clamp` | `F clamp(x: f64, min_val: f64, max_val: f64) -> f64` | Clamp to range |
| `clamp_i64` | `F clamp_i64(x: i64, min_val: i64, max_val: i64) -> i64` | Clamp to range (i64) |

### Power and Roots

| Function | Signature | Description |
|----------|-----------|-------------|
| `sqrt` | `F sqrt(x: f64) -> f64` | Square root |
| `pow` | `F pow(x: f64, y: f64) -> f64` | Power (x^y) |

### Rounding

| Function | Signature | Description |
|----------|-----------|-------------|
| `floor` | `F floor(x: f64) -> f64` | Round down |
| `ceil` | `F ceil(x: f64) -> f64` | Round up |
| `round` | `F round(x: f64) -> f64` | Round to nearest |

### Trigonometry

| Function | Signature | Description |
|----------|-----------|-------------|
| `sin` | `F sin(x: f64) -> f64` | Sine |
| `cos` | `F cos(x: f64) -> f64` | Cosine |
| `tan` | `F tan(x: f64) -> f64` | Tangent |
| `asin` | `F asin(x: f64) -> f64` | Arc sine |
| `acos` | `F acos(x: f64) -> f64` | Arc cosine |
| `atan` | `F atan(x: f64) -> f64` | Arc tangent |
| `atan2` | `F atan2(y: f64, x: f64) -> f64` | Two-argument arc tangent |

### Logarithmic / Exponential

| Function | Signature | Description |
|----------|-----------|-------------|
| `log` | `F log(x: f64) -> f64` | Natural logarithm |
| `log10` | `F log10(x: f64) -> f64` | Base-10 logarithm |
| `log2` | `F log2(x: f64) -> f64` | Base-2 logarithm |
| `exp` | `F exp(x: f64) -> f64` | Exponential (e^x) |

### Helpers

| Function | Signature | Description |
|----------|-----------|-------------|
| `deg_to_rad` | `F deg_to_rad(degrees: f64) -> f64` | Degrees to radians |
| `rad_to_deg` | `F rad_to_deg(radians: f64) -> f64` | Radians to degrees |
| `approx_eq` | `F approx_eq(a: f64, b: f64, epsilon: f64) -> i64` | Approximate equality check |

## Usage Examples

### Basic Calculations

```vais
U std/math

F main() -> i64 {
    # Absolute value
    x := abs(-42.5)  # 42.5
    y := abs_i64(-10)  # 10

    # Min/max
    smaller := min(3.5, 7.2)  # 3.5
    larger := max_i64(10, 20)  # 20

    # Clamping
    val := clamp(15.0, 0.0, 10.0)  # 10.0

    0
}
```

### Trigonometry

```vais
U std/math

F main() -> i64 {
    # Convert degrees to radians
    angle := deg_to_rad(45.0)

    # Compute sine and cosine
    s := sin(angle)
    c := cos(angle)

    # Pythagorean identity: sin²(x) + cos²(x) = 1
    hyp := sqrt(s * s + c * c)  # ~1.0

    # Inverse trigonometric functions
    radians := asin(0.707)
    degrees := rad_to_deg(radians)

    0
}
```

### Power and Logarithms

```vais
U std/math

F main() -> i64 {
    # Exponentiation
    squared := pow(5.0, 2.0)  # 25.0
    cubed := pow(2.0, 3.0)    # 8.0

    # Square root
    root := sqrt(16.0)  # 4.0

    # Natural logarithm
    ln := log(EULER)  # ~1.0

    # Exponential
    result := exp(1.0)  # ~2.718 (EULER)

    # Other logarithms
    log_10 := log10(100.0)  # 2.0
    log_2 := log2(8.0)      # 3.0

    0
}
```

### Rounding Operations

```vais
U std/math

F main() -> i64 {
    x := 3.7
    y := 3.2

    a := floor(x)  # 3.0
    b := ceil(x)   # 4.0
    c := round(x)  # 4.0

    d := floor(y)  # 3.0
    e := ceil(y)   # 4.0
    f := round(y)  # 3.0

    0
}
```

### Floating-Point Comparison

```vais
U std/math

F main() -> i64 {
    a := 0.1 + 0.2
    b := 0.3

    # Direct comparison may fail due to floating-point precision
    # I a == b { ... }

    # Use approximate equality instead
    epsilon := 0.0001
    I approx_eq(a, b, epsilon) == 1 {
        # Values are approximately equal
    }

    0
}
```

### Practical Example: Distance Calculation

```vais
U std/math

# Calculate Euclidean distance between two points
F distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    dx := x2 - x1
    dy := y2 - y1
    sqrt(dx * dx + dy * dy)
}

F main() -> i64 {
    dist := distance(0.0, 0.0, 3.0, 4.0)  # 5.0
    0
}
```

### Practical Example: Circle Calculations

```vais
U std/math

F circle_area(radius: f64) -> f64 {
    PI * radius * radius
}

F circle_circumference(radius: f64) -> f64 {
    TAU * radius  # or 2.0 * PI * radius
}

F main() -> i64 {
    r := 5.0
    area := circle_area(r)
    circ := circle_circumference(r)
    0
}
```
