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

## Usage

```vais
U std/math

F main() -> i64 {
    angle := deg_to_rad(45.0)
    s := sin(angle)
    c := cos(angle)
    hyp := sqrt(s * s + c * c)  # ~1.0
    0
}
```
