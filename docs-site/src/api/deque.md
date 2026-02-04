# Deque API Reference

> Double-ended queue with O(1) push/pop at both ends (circular buffer)

## Import

```vais
U std/deque
```

## Struct

```vais
S Deque { data: i64, head: i64, tail: i64, len: i64, cap: i64 }
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `with_capacity` | `F with_capacity(capacity: i64) -> Deque` | Create with capacity |
| `len` | `F len(&self) -> i64` | Number of elements |
| `capacity` | `F capacity(&self) -> i64` | Allocated capacity |
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `push_front` | `F push_front(&self, value: i64) -> i64` | Push to front |
| `push_back` | `F push_back(&self, value: i64) -> i64` | Push to back |
| `pop_front` | `F pop_front(&self) -> i64` | Pop from front (returns 0 if empty) |
| `pop_front_opt` | `F pop_front_opt(&self) -> Option<i64>` | Pop from front using Option |
| `pop_back` | `F pop_back(&self) -> i64` | Pop from back (returns 0 if empty) |
| `pop_back_opt` | `F pop_back_opt(&self) -> Option<i64>` | Pop from back using Option |
| `front` | `F front(&self) -> i64` | Peek front (returns 0 if empty) |
| `front_opt` | `F front_opt(&self) -> Option<i64>` | Peek front using Option |
| `back` | `F back(&self) -> i64` | Peek back (returns 0 if empty) |
| `back_opt` | `F back_opt(&self) -> Option<i64>` | Peek back using Option |
| `get` | `F get(&self, index: i64) -> i64` | Access by index (returns 0 if out of bounds) |
| `get_opt` | `F get_opt(&self, index: i64) -> Option<i64>` | Access by index using Option |
| `set` | `F set(&self, index: i64, value: i64) -> i64` | Set element at index (returns 1 on success) |
| `clear` | `F clear(&self) -> i64` | Remove all |
| `drop` | `F drop(&self) -> i64` | Free memory |

## Free Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `deque_new` | `F deque_new() -> Deque` | Create new Deque with default capacity (8) |

## Usage

```vais
U std/deque

F main() -> i64 {
    dq := deque_new()
    dq.push_back(1)
    dq.push_front(0)

    # Using basic methods
    val := dq.pop_front()  # 0

    # Using Option methods
    opt_val := dq.pop_back_opt()  # Some(1)
    empty_val := dq.pop_front_opt()  # None

    # Index access
    dq.push_back(10)
    dq.set(0, 20)
    val := dq.get(0)  # 20

    dq.drop()
    0
}
```
