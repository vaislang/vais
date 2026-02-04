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
| `is_locked` | `F is_locked(&self) -> i64` | Check if locked |

## RwLock\<T\>

Read-write lock allowing multiple readers or one writer.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(value: T) -> RwLock<T>` | Create with initial value |
| `read` | `F read(&self) -> RwLockReadGuard<T>` | Acquire read lock |
| `write` | `F write(&self) -> RwLockWriteGuard<T>` | Acquire write lock |
| `try_read` | `F try_read(&self) -> RwLockReadGuard<T>?` | Try non-blocking read lock |
| `try_write` | `F try_write(&self) -> RwLockWriteGuard<T>?` | Try non-blocking write lock |

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
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `len` | `F len(&self) -> i64` | Get message count |

### Sender\<T\> / Receiver\<T\>

Channel endpoint handles.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(channel: &Channel<T>)` | Create from channel |
| `send` | `F send(&self, value: T) -> i64` | Send message |
| `recv` | `F recv(&self) -> T?` | Receive message |
| `try_send` | `F try_send(&self, value: T) -> i64` | Non-blocking send |
| `try_recv` | `F try_recv(&self) -> T?` | Non-blocking receive |

### Channel Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `channel` | `F channel<T>(capacity: i64) -> (Sender<T>, Receiver<T>)` | Create channel pair |

## AtomicI64

Lock-free atomic integer.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(value: i64) -> AtomicI64` | Create atomic |
| `load` | `F load(&self) -> i64` | Atomic load |
| `store` | `F store(&self, value: i64) -> i64` | Atomic store |
| `swap` | `F swap(&self, value: i64) -> i64` | Atomic swap, returns old |
| `compare_exchange` | `F compare_exchange(&self, expected: i64, new_value: i64) -> i64` | CAS |
| `fetch_add` | `F fetch_add(&self, value: i64) -> i64` | Atomic add, returns old |
| `fetch_sub` | `F fetch_sub(&self, value: i64) -> i64` | Atomic subtract, returns old |
| `fetch_and` | `F fetch_and(&self, value: i64) -> i64` | Atomic AND, returns old |
| `fetch_or` | `F fetch_or(&self, value: i64) -> i64` | Atomic OR, returns old |
| `fetch_xor` | `F fetch_xor(&self, value: i64) -> i64` | Atomic XOR, returns old |

## AtomicBool

Lock-free atomic boolean.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(value: i64) -> AtomicBool` | Create atomic (0=false, 1=true) |
| `load` | `F load(&self) -> i64` | Atomic load |
| `store` | `F store(&self, value: i64) -> i64` | Atomic store |
| `swap` | `F swap(&self, value: i64) -> i64` | Atomic swap, returns old |
| `compare_exchange` | `F compare_exchange(&self, expected: i64, new_value: i64) -> i64` | CAS |

## Condvar

Condition variable for wait/notify.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> Condvar` | Create condvar |
| `wait` | `F wait(&self, mutex_guard: &MutexGuard<i64>) -> i64` | Wait on condition |
| `wait_timeout` | `F wait_timeout(&self, mutex_guard: &MutexGuard<i64>, timeout_ms: i64) -> i64` | Wait with timeout |
| `notify_one` | `F notify_one(&self) -> i64` | Wake one waiting thread |
| `notify_all` | `F notify_all(&self) -> i64` | Wake all waiting threads |

## Barrier

Synchronization barrier for N threads.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(n: i64) -> Barrier` | Create barrier for n threads |
| `wait` | `F wait(&self) -> i64` | Wait at barrier |

## Semaphore

Counting semaphore.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(permits: i64) -> Semaphore` | Create with permit count |
| `acquire` | `F acquire(&self) -> i64` | Acquire permit (blocking) |
| `try_acquire` | `F try_acquire(&self) -> i64` | Try non-blocking acquire |
| `release` | `F release(&self) -> i64` | Release permit |
| `available` | `F available(&self) -> i64` | Get available permits |

## Once

One-time initialization.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> Once` | Create once |
| `call_once` | `F call_once(&self, fn_ptr: i64) -> i64` | Call function exactly once |
| `is_completed` | `F is_completed(&self) -> i64` | Check if completed |

## SpinLock

Busy-wait lock for short critical sections.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> SpinLock` | Create spinlock |
| `lock` | `F lock(&self) -> SpinLockGuard` | Acquire lock (spin) |
| `try_lock` | `F try_lock(&self) -> SpinLockGuard?` | Try non-blocking lock |
| `unlock` | `F unlock(&self) -> i64` | Release lock |

## WaitGroup

Wait for a group of tasks to complete.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> WaitGroup` | Create wait group |
| `add` | `F add(&self, delta: i64) -> i64` | Add delta to counter |
| `done` | `F done(&self) -> i64` | Decrement counter by 1 |
| `wait` | `F wait(&self) -> i64` | Wait until counter reaches zero |
| `count` | `F count(&self) -> i64` | Get current count |
| `free` | `F free(&self) -> i64` | Free resources |

## CancellationTokenSource

Creates and controls cancellation tokens.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> CancellationTokenSource` | Create token source |
| `token` | `F token(&self) -> CancellationToken` | Get a token |
| `cancel` | `F cancel(&self) -> i64` | Cancel all tokens |
| `is_cancelled` | `F is_cancelled(&self) -> i64` | Check if cancelled |
| `create_linked_source` | `F create_linked_source(&self) -> CancellationTokenSource` | Create linked source |
| `token_count` | `F token_count(&self) -> i64` | Get active token count |
| `free` | `F free(&self) -> i64` | Free resources |

## CancellationToken

Handle to check for cancellation.

| Method | Signature | Description |
|--------|-----------|-------------|
| `is_cancelled` | `F is_cancelled(&self) -> i64` | Check if cancelled |
| `throw_if_cancelled` | `F throw_if_cancelled(&self) -> i64` | Return -1 if cancelled |
| `wait_for_cancellation` | `F wait_for_cancellation(&self) -> i64` | Block until cancelled |
| `wait_for_cancellation_timeout` | `F wait_for_cancellation_timeout(&self, timeout_ms: i64) -> i64` | Wait with timeout |
| `register` | `F register(&self, callback: i64) -> CancellationRegistration` | Register callback |
| `none` | `F none() -> CancellationToken` | Create never-cancelled token |
| `drop` | `F drop(&self) -> i64` | Drop token reference |

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
