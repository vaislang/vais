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
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `push_front` | `F push_front(&self, value: i64) -> i64` | Push to front |
| `push_back` | `F push_back(&self, value: i64) -> i64` | Push to back |
| `pop_front` | `F pop_front(&self) -> i64` | Pop from front |
| `pop_back` | `F pop_back(&self) -> i64` | Pop from back |
| `front` | `F front(&self) -> i64` | Peek front |
| `back` | `F back(&self) -> i64` | Peek back |
| `get` | `F get(&self, index: i64) -> i64` | Access by index |
| `clear` | `F clear(&self) -> i64` | Remove all |
| `drop` | `F drop(&self) -> i64` | Free memory |

## Usage

```vais
U std/deque

F main() -> i64 {
    dq := Deque.with_capacity(16)
    dq.push_back(1)
    dq.push_front(0)
    val := dq.pop_front()  # 0
    dq.drop()
    0
}
```
