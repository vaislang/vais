# PropTest API Reference

> Property-based testing with random generation and shrinking

## Import

```vais
U std/proptest
```

## Constants

| Name | Value | Description |
|------|-------|-------------|
| `DEFAULT_TEST_CASES` | 100 | Random cases per property |
| `MAX_SHRINK_ATTEMPTS` | 100 | Maximum shrink iterations |
| `PROP_PASSED` | 0 | Property held |
| `PROP_FAILED` | 1 | Property violated |
| `PROP_DISCARD` | 2 | Input discarded |

## Generator Types

| Constant | Description |
|----------|-------------|
| `GEN_I64` | Random i64 |
| `GEN_I64_RANGE` | i64 in range |
| `GEN_BOOL` | Random boolean |
| `GEN_F64` | Random f64 |
| `GEN_VEC` | Random vector |

## Key Functions

| Function | Description |
|----------|-------------|
| `gen_i64()` | Create i64 generator |
| `gen_i64_range(min, max)` | Create range generator |
| `gen_bool()` | Create bool generator |
| `prop_test(gen, property_fn, cases)` | Run property test |

## Usage

```vais
U std/proptest

# Test that addition is commutative
F prop_commutative(a: i64, b: i64) -> i64 {
    I a + b == b + a { PROP_PASSED }
    E { PROP_FAILED }
}
```
