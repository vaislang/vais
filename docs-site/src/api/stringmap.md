# StringMap API Reference

> Hash table with string keys and i64 values (content-based comparison)

## Import

```vais
U std/stringmap
```

## Struct

```vais
S StringMap { buckets: i64, size: i64, cap: i64 }
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `with_capacity` | `F with_capacity(capacity: i64) -> StringMap` | Create with capacity |
| `len` | `F len(&self) -> i64` | Number of entries |
| `capacity` | `F capacity(&self) -> i64` | Get capacity |
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `get` | `F get(&self, key: i64) -> i64` | Get value by string key |
| `get_opt` | `F get_opt(&self, key: i64) -> Option<i64>` | Get as Option |
| `contains` | `F contains(&self, key: i64) -> i64` | Check if key exists |
| `set` | `F set(&self, key: i64, value: i64) -> i64` | Insert/update |
| `remove` | `F remove(&self, key: i64) -> i64` | Remove key |
| `clear` | `F clear(&self) -> i64` | Remove all entries |
| `drop` | `F drop(&self) -> i64` | Free all memory |

## Overview

Unlike HashMap which uses integer key comparison, StringMap compares keys by string content using byte-by-byte comparison. Keys are duplicated on insert. Uses DJB2 hash from `std/hash`.

## Usage

```vais
U std/stringmap

F main() -> i64 {
    m := StringMap.with_capacity(16)
    m.set("name", 42)
    val := m.get("name")  # 42
    m.drop()
    0
}
```
