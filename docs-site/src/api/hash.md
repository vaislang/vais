# Hash API Reference

> Generic hash functions for collections and hash-based data structures

## Import

```vais
U std/hash
```

## Functions

### Basic Hash Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `mult_hash` | `F mult_hash(value: i64) -> i64` | Multiplicative hash using golden ratio prime (2654435769) |
| `hash_i64` | `F hash_i64(value: i64) -> i64` | Hash an integer (alias for mult_hash) |
| `hash_bool` | `F hash_bool(value: i64) -> i64` | Hash a boolean value (returns 0 or 1) |
| `hash_string` | `F hash_string(str_ptr: i64) -> i64` | DJB2 hash for null-terminated strings |

### Composite Hash Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `hash_pair` | `F hash_pair(a: i64, b: i64) -> i64` | Hash a pair of values (for tuple keys) |
| `hash_triple` | `F hash_triple(a: i64, b: i64, c: i64) -> i64` | Hash three values (for 3-tuple keys) |
| `combine_hash` | `F combine_hash(h1: i64, h2: i64) -> i64` | Combine two hash values (for compound keys) |

## Details

### mult_hash

Simple multiplicative hash with excellent distribution for integers. Uses the golden ratio prime constant (2654435769) which is well-known for good hash distribution.

**Formula:** `h = value * 2654435769`, then absolute value

### hash_string

Uses the DJB2 algorithm, a fast and effective hash function for strings.

**Formula:** `hash = hash * 33 + byte` for each byte, starting with `hash = 5381`

### combine_hash

Combines two hashes using shift and XOR operations for uniform distribution.

**Formula:** `(h1 * 31) ^ h2`

## Usage

### Hash Integers

```vais
U std/hash

F main() -> i64 {
    h := hash_i64(42)
    # Use in hash table or set
    0
}
```

### Hash Strings

```vais
U std/hash

F main() -> i64 {
    str := "hello world"
    h := hash_string(str)
    0
}
```

### Hash Composite Keys

```vais
U std/hash

F main() -> i64 {
    # Hash a pair (e.g., for (x, y) coordinate key)
    x := 10
    y := 20
    h_pair := hash_pair(x, y)

    # Hash a triple (e.g., for (x, y, z) 3D coordinate)
    z := 30
    h_triple := hash_triple(x, y, z)

    # Combine independent hashes
    h1 := hash_i64(42)
    h2 := hash_string("key")
    combined := combine_hash(h1, h2)

    0
}
```

### Custom Struct Hash

```vais
U std/hash

S Point {
    x: i64,
    y: i64
}

X Point {
    # Custom hash for Point
    F hash(&self) -> i64 {
        hash_pair(self.x, self.y)
    }
}

F main() -> i64 {
    p := Point { x: 10, y: 20 }
    h := p.hash()
    0
}
```

## Notes

- All hash functions return non-negative i64 values (absolute value taken)
- Hash functions are designed for use with hash tables (HashMap, HashSet)
- DJB2 algorithm is used for string hashing (fast with good distribution)
- Golden ratio prime constant provides good hash distribution for integers
- Composite hash functions are useful for multi-field keys in hash tables
