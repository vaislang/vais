# Set API Reference

> Hash-based set collection for i64 values

## Import

```vais
U std/set
```

## Struct

```vais
S Set { buckets: i64, size: i64, cap: i64 }
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `with_capacity` | `F with_capacity(capacity: i64) -> Set` | Create with capacity |
| `len` | `F len(&self) -> i64` | Number of elements |
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `contains` | `F contains(&self, value: i64) -> i64` | Check membership |
| `insert` | `F insert(&self, value: i64) -> i64` | Add element |
| `remove` | `F remove(&self, value: i64) -> i64` | Remove element |
| `clear` | `F clear(&self) -> i64` | Remove all |
| `drop` | `F drop(&self) -> i64` | Free memory |

## Usage

```vais
U std/set

F main() -> i64 {
    s := Set.with_capacity(16)
    s.insert(10)
    s.insert(20)
    s.contains(10)  # 1
    s.remove(10)
    s.drop()
    0
}
```
