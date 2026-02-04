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
| `capacity` | `F capacity(&self) -> i64` | Allocated capacity |
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `push` | `F push(&self, value: i64) -> i64` | Insert element |
| `pop` | `F pop(&self) -> i64` | Remove and return minimum (returns 0 if empty) |
| `pop_opt` | `F pop_opt(&self) -> Option` | Remove and return minimum using Option |
| `peek` | `F peek(&self) -> i64` | View minimum without removing (returns 0 if empty) |
| `peek_opt` | `F peek_opt(&self) -> Option` | View minimum using Option |
| `clear` | `F clear(&self) -> i64` | Remove all elements |
| `drop` | `F drop(&self) -> i64` | Free memory |

## Free Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `pq_new` | `F pq_new() -> PriorityQueue` | Create new PriorityQueue with default capacity (8) |
| `pq_push` | `F pq_push(pq: PriorityQueue, value: i64) -> i64` | Push element into priority queue |
| `pq_pop` | `F pq_pop(pq: PriorityQueue) -> i64` | Pop minimum element |
| `pq_peek` | `F pq_peek(pq: PriorityQueue) -> i64` | Peek at minimum element |
| `pq_size` | `F pq_size(pq: PriorityQueue) -> i64` | Get size |
| `pq_is_empty` | `F pq_is_empty(pq: PriorityQueue) -> i64` | Check if empty |
| `pq_clear` | `F pq_clear(pq: PriorityQueue) -> i64` | Clear all elements |
| `pq_free` | `F pq_free(pq: PriorityQueue) -> i64` | Free memory |

## Usage

```vais
U std/priority_queue

F main() -> i64 {
    # Using methods
    pq := PriorityQueue.with_capacity(16)
    pq.push(30)
    pq.push(10)
    pq.push(20)
    min := pq.pop()  # 10

    # Using Option methods
    opt_min := pq.peek_opt()  # Some(20)

    # Using free functions
    pq2 := pq_new()
    pq_push(pq2, 5)
    pq_push(pq2, 15)
    val := pq_pop(pq2)  # 5

    pq.drop()
    pq_free(pq2)
    0
}
```
