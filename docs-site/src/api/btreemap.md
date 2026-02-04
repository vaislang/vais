# BTreeMap API Reference

> Self-balancing ordered map using B-tree (keys in sorted order)

## Import

```vais
U std/btreemap
```

## Struct

```vais
S BTreeMap { root: i64, size: i64 }
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> BTreeMap` | Create empty B-tree map |
| `len` | `F len(&self) -> i64` | Number of entries |
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `get` | `F get(&self, key: i64) -> i64` | Get value by key |
| `get_opt` | `F get_opt(&self, key: i64) -> Option<i64>` | Get as Option |
| `contains` | `F contains(&self, key: i64) -> i64` | Check if key exists |
| `insert` | `F insert(&self, key: i64, value: i64) -> i64` | Insert key-value |
| `min_key` | `F min_key(&self) -> i64` | Get minimum key |
| `max_key` | `F max_key(&self) -> i64` | Get maximum key |
| `drop` | `F drop(&self) -> i64` | Free all memory |

## Usage

```vais
U std/btreemap

F main() -> i64 {
    m := BTreeMap::new()
    m.insert(3, 30)
    m.insert(1, 10)
    m.insert(2, 20)
    min := m.min_key()  # 1
    m.drop()
    0
}
```
