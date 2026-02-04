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
| `capacity` | `F capacity(&self) -> i64` | Number of buckets |
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `contains` | `F contains(&self, value: i64) -> i64` | Check membership |
| `insert` | `F insert(&self, value: i64) -> i64` | Add element (returns 1 if newly inserted) |
| `remove` | `F remove(&self, value: i64) -> i64` | Remove element (returns 1 if found) |
| `clear` | `F clear(&self) -> i64` | Remove all |
| `union` | `F union(&self, other: Set) -> Set` | Create set with all elements from both |
| `intersection` | `F intersection(&self, other: Set) -> Set` | Create set with common elements |
| `difference` | `F difference(&self, other: Set) -> Set` | Create set with elements in self but not other |
| `symmetric_difference` | `F symmetric_difference(&self, other: Set) -> Set` | Elements in either but not both |
| `is_subset` | `F is_subset(&self, other: Set) -> i64` | Check if all elements in other |
| `is_superset` | `F is_superset(&self, other: Set) -> i64` | Check if contains all of other |
| `is_disjoint` | `F is_disjoint(&self, other: Set) -> i64` | Check if no common elements |
| `drop` | `F drop(&self) -> i64` | Free memory |

## Free Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `set_new` | `F set_new() -> Set` | Create new Set with default capacity (16) |

## Usage

```vais
U std/set

F main() -> i64 {
    s1 := set_new()
    s1.insert(10)
    s1.insert(20)
    s1.contains(10)  # 1

    s2 := Set.with_capacity(16)
    s2.insert(20)
    s2.insert(30)

    union_set := s1.union(s2)  # {10, 20, 30}
    inter_set := s1.intersection(s2)  # {20}

    s1.drop()
    s2.drop()
    union_set.drop()
    inter_set.drop()
    0
}
```
