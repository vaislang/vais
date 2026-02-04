# Hash API Reference

> Hash functions for collections and generic hashing

## Import

```vais
U std/hash
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `mult_hash` | `F mult_hash(value: i64) -> i64` | Multiplicative hash (golden ratio) |
| `hash_string` | `F hash_string(str_ptr: i64) -> i64` | DJB2 hash for strings |
| `hash_i64` | `F hash_i64(value: i64) -> i64` | Hash an integer (alias for mult_hash) |
| `hash_bool` | `F hash_bool(value: i64) -> i64` | Hash a boolean |
| `hash_pair` | `F hash_pair(a: i64, b: i64) -> i64` | Hash a pair of values |
| `hash_triple` | `F hash_triple(a: i64, b: i64, c: i64) -> i64` | Hash a triple |
| `combine_hash` | `F combine_hash(h1: i64, h2: i64) -> i64` | Combine two hashes |

## Usage

```vais
U std/hash

F main() -> i64 {
    h1 := hash_i64(42)
    h2 := hash_string("hello")
    combined := combine_hash(h1, h2)
    0
}
```
