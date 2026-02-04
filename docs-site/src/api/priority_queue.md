# PriorityQueue API Reference

> Min-heap priority queue (smaller values have higher priority)

## Import

```vais
U std/priority_queue
```

## Struct

```vais
S PriorityQueue { data: i64, size: i64, capacity: i64 }
```

## Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `with_capacity` | `F with_capacity(capacity: i64) -> PriorityQueue` | Create with capacity |
| `len` | `F len(&self) -> i64` | Number of elements |
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `push` | `F push(&self, value: i64) -> i64` | Insert element |
| `pop` | `F pop(&self) -> i64` | Remove and return minimum |
| `peek` | `F peek(&self) -> i64` | View minimum without removing |
| `clear` | `F clear(&self) -> i64` | Remove all elements |
| `drop` | `F drop(&self) -> i64` | Free memory |

## Usage

```vais
U std/priority_queue

F main() -> i64 {
    pq := PriorityQueue.with_capacity(16)
    pq.push(30)
    pq.push(10)
    pq.push(20)
    min := pq.pop()  # 10
    pq.drop()
    0
}
```
