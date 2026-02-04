# PropTest API Reference

> Property-based testing with random generation and shrinking (QuickCheck/PropTest style)

## Import

```vais
U std/proptest
U std/random
U std/test
```

## Overview

The `proptest` module provides QuickCheck-style property-based testing for Vais, featuring:
- Random test case generation with configurable generators
- Automatic shrinking of failing inputs to minimal counterexamples
- Multiple generator types (i64, f64, bool, ranges)
- Built-in property predicates (idempotent, commutative, associative)

## Constants

### Test Configuration

| Name | Value | Description |
|------|-------|-------------|
| `DEFAULT_TEST_CASES` | 100 | Default number of random test cases per property |
| `MAX_SHRINK_ATTEMPTS` | 100 | Maximum shrinking iterations |
| `DEFAULT_SEED` | 42 | Default random seed for reproducibility |

### Property Result Status

| Name | Value | Description |
|------|-------|-------------|
| `PROP_PASSED` | 0 | Property held for all inputs |
| `PROP_FAILED` | 1 | Property violated |
| `PROP_DISCARD` | 2 | Input doesn't satisfy precondition |

### Generator Types

| Name | Value | Description |
|------|-------|-------------|
| `GEN_I64` | 1 | Random i64 in full range |
| `GEN_I64_RANGE` | 2 | Random i64 in specified range |
| `GEN_BOOL` | 3 | Random boolean |
| `GEN_F64` | 4 | Random f64 |
| `GEN_F64_RANGE` | 5 | Random f64 in specified range |
| `GEN_OPTION` | 6 | Optional value generator |
| `GEN_VEC` | 7 | Vector generator |

## Structs

### Generator

Produces random test values with shrinking support.

```vais
S Generator {
    gen_type: i64,
    min: i64,
    max: i64,
    min_f: f64,
    max_f: f64,
    inner_gen: i64,    # Pointer to inner generator for Option/Vec
    max_len: i64       # Max length for Vec generator
}
```

**Methods (via `X Generator`):**

| Method | Signature | Description |
|--------|-----------|-------------|
| `i64_any` | `F i64_any() -> Generator` | Generate any i64 value |
| `i64_range` | `F i64_range(min: i64, max: i64) -> Generator` | Generate i64 in range [min, max] |
| `i64_small` | `F i64_small() -> Generator` | Generate small positive i64 (0-100) |
| `i64_positive` | `F i64_positive() -> Generator` | Generate positive i64 (1 to MAX) |
| `i64_nonzero` | `F i64_nonzero() -> Generator` | Generate non-zero i64 |
| `bool_any` | `F bool_any() -> Generator` | Generate random boolean |
| `f64_unit` | `F f64_unit() -> Generator` | Generate f64 in [0, 1) |
| `f64_range` | `F f64_range(min: f64, max: f64) -> Generator` | Generate f64 in range |
| `generate` | `F generate(&self) -> i64` | Generate a random value |
| `shrink` | `F shrink(&self, value: i64) -> i64` | Shrink value towards smaller counterexample |
| `can_shrink` | `F can_shrink(&self, value: i64) -> i64` | Check if value can be shrunk further |

### PropertyResult

Result of running a property test.

```vais
S PropertyResult {
    status: i64,
    message: str,
    counterexample: i64,    # The failing input value
    shrunk_value: i64,      # Minimized counterexample
    test_cases: i64,        # Number of test cases run
    shrink_steps: i64       # Number of shrinking steps
}
```

**Methods (via `X PropertyResult`):**

| Method | Signature | Description |
|--------|-----------|-------------|
| `passed` | `F passed(test_cases: i64) -> PropertyResult` | Create passed result |
| `failed` | `F failed(message: str, counterexample: i64, shrunk_value: i64, test_cases: i64, shrink_steps: i64) -> PropertyResult` | Create failed result |
| `is_passed` | `F is_passed(&self) -> i64` | Check if test passed |
| `is_failed` | `F is_failed(&self) -> i64` | Check if test failed |

### Property

A testable property with generator and test function.

```vais
S Property {
    name: str,
    test_fn: i64,          # Function pointer: (i64) -> i64 (0=pass, non-0=fail)
    generator: Generator,
    num_tests: i64,
    seed: i64
}
```

**Methods (via `X Property`):**

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(name: str, test_fn: i64, gen: Generator) -> Property` | Create new property test |
| `with_tests` | `F with_tests(&self, n: i64) -> Property` | Set number of test cases |
| `with_seed` | `F with_seed(&self, seed: i64) -> Property` | Set random seed |
| `check` | `F check(&self) -> PropertyResult` | Run the property test |

### ShrinkResult

Result of shrinking a counterexample.

```vais
S ShrinkResult {
    value: i64,
    steps: i64
}
```

## Top-Level Functions

### Test Execution

| Function | Signature | Description |
|----------|-----------|-------------|
| `prop_check` | `F prop_check(name: str, test_fn: i64, gen: Generator) -> PropertyResult` | Run a simple property test |
| `prop_assert` | `F prop_assert(name: str, test_fn: i64, gen: Generator) -> i64` | Assert property holds, panic if not |

### Shrinking

| Function | Signature | Description |
|----------|-----------|-------------|
| `shrink_counterexample` | `F shrink_counterexample(gen: &Generator, test_fn: i64, initial: i64) -> ShrinkResult` | Shrink counterexample to minimal failing case |
| `call_test_fn` | `F call_test_fn(fn_ptr: i64, arg: i64) -> i64` | Helper to call test function pointer |

### Built-in Property Predicates

| Function | Signature | Description |
|----------|-----------|-------------|
| `prop_idempotent` | `F prop_idempotent(f: i64, x: i64) -> i64` | Check f(f(x)) == f(x) |
| `prop_commutative` | `F prop_commutative(f: i64, a: i64, b: i64) -> i64` | Check f(a, b) == f(b, a) |
| `prop_associative` | `F prop_associative(f: i64, a: i64, b: i64, c: i64) -> i64` | Check f(f(a, b), c) == f(a, f(b, c)) |

### Float Bit Manipulation

| Function | Signature | Description |
|----------|-----------|-------------|
| `f64_to_i64_bits` | `F f64_to_i64_bits(f: f64) -> i64` | Convert f64 to bit representation as i64 |
| `i64_bits_to_f64` | `F i64_bits_to_f64(bits: i64) -> f64` | Convert i64 bits back to f64 |

### Memory Helpers

| Function | Signature | Description |
|----------|-----------|-------------|
| `store_f64` | `F store_f64(ptr: i64, val: f64) -> i64` | Store f64 at pointer |
| `load_f64` | `F load_f64(ptr: i64) -> f64` | Load f64 from pointer |
| `store_i64` | `F store_i64(ptr: i64, val: i64) -> i64` | Store i64 at pointer |
| `load_i64` | `F load_i64(ptr: i64) -> i64` | Load i64 from pointer |

## Examples

### Basic Property Test

```vais
U std/proptest

# Test function: returns 0 if property holds, non-zero if violated
F test_abs_positive(x: i64) -> i64 {
    abs_x := I x < 0 { -x } E { x }
    I abs_x >= 0 { 0 } E { 1 }  # 0 = pass
}

F main() -> i64 {
    gen := Generator.i64_any()
    result := prop_check("abs is always positive", test_abs_positive, gen)

    I result.is_passed() == 1 {
        # Test passed
        0
    } E {
        # Test failed
        1
    }
}
```

### Using Property Struct

```vais
U std/proptest

F test_addition_commutative(x: i64) -> i64 {
    # In real code, would need to pass two args
    # For now, test with x and x+1
    a := x
    b := x + 1
    I a + b == b + a { 0 } E { 1 }
}

F main() -> i64 {
    gen := Generator.i64_range(0, 1000)

    prop := Property.new("addition is commutative", test_addition_commutative, gen)
    prop = prop.with_tests(200)  # Run 200 test cases
    prop = prop.with_seed(12345)  # Custom seed

    result := prop.check()

    I result.is_failed() == 1 {
        # Get details about failure
        counterexample := result.counterexample
        shrunk := result.shrunk_value
    }

    0
}
```

### Assert Property

```vais
U std/proptest

F test_double_half(x: i64) -> i64 {
    # For non-zero x, (x * 2) / 2 should equal x
    I x == 0 {
        0  # Skip zero (precondition)
    } E {
        result := (x * 2) / 2
        I result == x { 0 } E { 1 }
    }
}

F main() -> i64 {
    gen := Generator.i64_nonzero()

    # This will panic with details if the property fails
    prop_assert("double then half returns original", test_double_half, gen)

    0
}
```

### Range Generators

```vais
U std/proptest

F test_in_range(x: i64) -> i64 {
    # Verify generated value is in expected range
    I x >= 10 && x <= 100 { 0 } E { 1 }
}

F main() -> i64 {
    # Small positive values
    small_gen := Generator.i64_small()  # 0-100

    # Custom range
    range_gen := Generator.i64_range(10, 100)

    # Positive only
    pos_gen := Generator.i64_positive()  # 1 to MAX

    result := prop_check("values in range", test_in_range, range_gen)
    0
}
```

### Float Property Testing

```vais
U std/proptest

F test_float_sum(bits: i64) -> i64 {
    # Convert i64 bits to f64
    x := i64_bits_to_f64(bits)

    # Test that x + 0.0 == x
    sum := x + 0.0

    # Convert back for comparison
    sum_bits := f64_to_i64_bits(sum)
    I sum_bits == bits { 0 } E { 1 }
}

F main() -> i64 {
    gen := Generator.f64_unit()  # [0, 1)
    result := prop_check("adding zero preserves value", test_float_sum, gen)
    0
}
```

### With Shrinking

```vais
U std/proptest

F test_always_fails(x: i64) -> i64 {
    # This will fail for any x > 10
    I x > 10 { 1 } E { 0 }
}

F main() -> i64 {
    gen := Generator.i64_range(0, 1000)
    result := prop_check("always passes", test_always_fails, gen)

    I result.is_failed() == 1 {
        # Shrinking will reduce counterexample to minimal failing case
        # Original failure might be x=500, but shrunk to x=11
        minimal := result.shrunk_value
        steps := result.shrink_steps
    }

    0
}
```

### Idempotence Testing

```vais
U std/proptest

# Define a function to test
F abs_value(x: i64) -> i64 {
    I x < 0 { -x } E { x }
}

F main() -> i64 {
    gen := Generator.i64_any()

    # Create property that tests idempotence
    # Note: prop_idempotent expects function pointer
    result := prop_check("abs is idempotent", prop_idempotent, gen)

    0
}
```
