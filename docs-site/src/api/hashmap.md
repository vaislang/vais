# HashMap API Reference

> Generic hash table with separate chaining collision resolution

## Import

```vais
U std/hashmap
```

## Struct

```vais
S HashMap<K, V> {
    buckets: i64,   # Bucket array pointer
    size: i64,      # Number of key-value pairs
    cap: i64        # Number of buckets
}
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `with_capacity` | `F with_capacity(capacity: i64) -> HashMap<K, V>` | Create with given bucket count |
| `len` | `F len(&self) -> i64` | Get number of entries |
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `get` | `F get(&self, key: K) -> V` | Get value (0 if not found) |
| `get_opt` | `F get_opt(&self, key: K) -> Option<V>` | Get value as Option |
| `contains` | `F contains(&self, key: K) -> i64` | Check if key exists |
| `set` | `F set(&self, key: K, value: V) -> V` | Insert/update, returns old value |
| `remove` | `F remove(&self, key: K) -> V` | Remove key, returns value |
| `remove_opt` | `F remove_opt(&self, key: K) -> Option<V>` | Remove with Option return |
| `clear` | `F clear(&self) -> i64` | Remove all entries |
| `drop` | `F drop(&self) -> i64` | Free all memory |

## Free Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `hashmap_new` | `F hashmap_new() -> HashMap<i64, i64>` | Create with default capacity (16) |

## Usage

```vais
U std/hashmap

F main() -> i64 {
    m := hashmap_new()
    m.set(1, 100)
    m.set(2, 200)
    val := m.get(1)     # val = 100
    m.remove(2)
    m.drop()
    0
}
```
