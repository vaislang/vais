# Sync API Reference

> Synchronization primitives: Mutex, RwLock, Channel, atomics, and more

## Import

```vais
U std/sync
```

## Mutex\<T\>

Mutual exclusion lock protecting a value.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(value: T) -> Mutex<T>` | Create with initial value |
| `lock` | `F lock(&self) -> MutexGuard<T>` | Acquire lock (blocking) |
| `try_lock` | `F try_lock(&self) -> MutexGuard<T>?` | Try non-blocking lock |

## RwLock\<T\>

Read-write lock allowing multiple readers or one writer.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(value: T) -> RwLock<T>` | Create with initial value |
| `read` | `F read(&self) -> RwLockReadGuard<T>` | Acquire read lock |
| `write` | `F write(&self) -> RwLockWriteGuard<T>` | Acquire write lock |

## Channel\<T\>

Bounded MPSC message-passing channel.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(capacity: i64) -> Channel<T>` | Create bounded channel |
| `send` | `F send(&self, value: T) -> i64` | Send (blocks if full) |
| `recv` | `F recv(&self) -> T?` | Receive (blocks if empty) |
| `try_send` | `F try_send(&self, value: T) -> i64` | Non-blocking send |
| `try_recv` | `F try_recv(&self) -> T?` | Non-blocking receive |
| `close` | `F close(&self) -> i64` | Close channel |

## AtomicI64

Lock-free atomic integer.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(value: i64) -> AtomicI64` | Create atomic |
| `load` | `F load(&self) -> i64` | Atomic load |
| `store` | `F store(&self, value: i64) -> i64` | Atomic store |
| `fetch_add` | `F fetch_add(&self, value: i64) -> i64` | Atomic add, returns old |
| `compare_exchange` | `F compare_exchange(&self, expected: i64, new_value: i64) -> i64` | CAS |

## Other Primitives

- **Condvar** - Condition variable for wait/notify
- **Barrier** - Synchronization barrier for N threads
- **Semaphore** - Counting semaphore with acquire/release
- **Once** - One-time initialization
- **SpinLock** - Busy-wait lock for short critical sections
- **WaitGroup** - Wait for a group of tasks to complete
- **CancellationToken** - Cooperative cancellation for async ops

## Usage

```vais
U std/sync

F main() -> i64 {
    m := Mutex::new(0)
    guard := m.lock()
    guard.set(42)
    guard.unlock()
    0
}
```
