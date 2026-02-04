# Collections API Reference

> Unified re-export of all collection types plus LinkedList

## Import

```vais
U std/collections
```

## Re-exported Modules

Importing `std/collections` gives access to:

- `Vec<T>` from `std/vec`
- `HashMap<K,V>` from `std/hashmap`
- `BTreeMap<K,V>` from `std/btreemap`
- `HashSet<T>` from `std/set`
- `Deque<T>` from `std/deque`
- `PriorityQueue` from `std/priority_queue`

## LinkedList

Doubly-linked list included directly in this module.

```vais
S LinkedList { head: i64, tail: i64, len: i64 }
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> LinkedList` | Create empty list |
| `push_front` | `F push_front(&self, value: i64) -> i64` | Add to front |
| `push_back` | `F push_back(&self, value: i64) -> i64` | Add to back |
| `pop_front` | `F pop_front(&self) -> i64` | Remove from front |
| `pop_back` | `F pop_back(&self) -> i64` | Remove from back |
| `len` | `F len(&self) -> i64` | Get length |

## Usage

```vais
U std/collections

F main() -> i64 {
    list := LinkedList::new()
    list.push_back(1)
    list.push_back(2)
    val := list.pop_front()  # 1
    0
}
```
