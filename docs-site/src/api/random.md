# Random API Reference

> Pseudo-random number generation (LCG and xorshift)

## Import

```vais
U std/random
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `random_seed` | `F random_seed(seed: i64) -> i64` | Set RNG seed |
| `random_init` | `F random_init() -> i64` | Seed from current time |
| `random_i64` | `F random_i64() -> i64` | Random i64 in [0, 2^31) |
| `random_range` | `F random_range(min: i64, max: i64) -> i64` | Random in [min, max] |
| `random_below` | `F random_below(n: i64) -> i64` | Random in [0, n) |
| `random_f64` | `F random_f64() -> f64` | Random f64 in [0.0, 1.0) |
| `random_f64_range` | `F random_f64_range(min: f64, max: f64) -> f64` | Random f64 in [min, max) |
| `random_bool` | `F random_bool() -> i64` | Random 0 or 1 |
| `random_shuffle` | `F random_shuffle(arr: i64, len: i64) -> i64` | Fisher-Yates shuffle |
| `random_bytes` | `F random_bytes(buffer: i64, count: i64) -> i64` | Fill buffer with random bytes |
| `random_alnum` | `F random_alnum() -> i64` | Random alphanumeric char |
| `random_hex` | `F random_hex() -> i64` | Random hex char |
| `random_xorshift` | `F random_xorshift() -> i64` | Fast xorshift random |

## Usage

```vais
U std/random

F main() -> i64 {
    random_init()
    dice := random_range(1, 6)
    coin := random_bool()
    0
}
```
