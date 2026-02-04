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
| `front` | `F front(&self) -> i64` | Peek at front value |
| `back` | `F back(&self) -> i64` | Peek at back value |
| `len` | `F len(&self) -> i64` | Get length |
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `clear` | `F clear(&self) -> i64` | Remove all elements |
| `contains` | `F contains(&self, value: i64) -> i64` | Check if value exists |

## RingBuffer

Fixed-capacity circular buffer included directly in this module.

```vais
S RingBuffer { data: i64, capacity: i64, head: i64, tail: i64, len: i64 }
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(capacity: i64) -> RingBuffer` | Create with fixed capacity |
| `push` | `F push(&self, value: i64) -> i64` | Push to back (returns 1 if full) |
| `pop` | `F pop(&self) -> i64` | Pop from front (returns 0 if empty) |
| `front` | `F front(&self) -> i64` | Peek at front value |
| `len` | `F len(&self) -> i64` | Get current length |
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `is_full` | `F is_full(&self) -> i64` | Check if full |
| `capacity` | `F capacity(&self) -> i64` | Get capacity |
| `clear` | `F clear(&self) -> i64` | Remove all elements |

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
