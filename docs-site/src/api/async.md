# Async API Reference

> High-level async utilities: timeout, retry, race, async channels

## Import

```vais
U std/async
```

## Types

### TimeoutFuture

Wraps a future with a deadline. Returns -1 if not completed in time.

### RetryConfig

Configurable retry logic with exponential backoff.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(max_retries: i64) -> RetryConfig` | Create with default backoff |
| `with_backoff` | `F with_backoff(max: i64, base: i64, factor: i64) -> RetryConfig` | Custom backoff |
| `should_retry` | `F should_retry(&self) -> i64` | Check if retries remain |
| `record_retry` | `F record_retry(&self) -> i64` | Record attempt, returns delay |
| `retries` | `F retries(&self) -> i64` | Get current retry count |
| `remaining` | `F remaining(&self) -> i64` | Get remaining retries |

### RaceFuture

Races multiple futures and returns the first completed result.

```vais
S RaceFuture { futures: i64, count: i64, completed: i64 }
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(count: i64) -> RaceFuture` | Create with capacity for N futures |
| `add` | `F add(&self, index: i64, future_ptr: i64, poll_fn: i64) -> i64` | Add future to race |
| `winner` | `F winner(&self) -> i64` | Get index of completed future |
| `cleanup` | `F cleanup(&self) -> i64` | Free memory |

### AsyncMutex

Async-aware mutual exclusion (non-blocking try_lock).

```vais
S AsyncMutex { locked: i64, value: i64, waiters: i64, waiter_head: i64 }
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(value: i64) -> AsyncMutex` | Create with protected value |
| `try_lock` | `F try_lock(&self) -> i64` | Try to acquire (returns 1 if acquired) |
| `unlock` | `F unlock(&self) -> i64` | Release the lock |
| `get` | `F get(&self) -> i64` | Get protected value (must be locked) |
| `set` | `F set(&self, value: i64) -> i64` | Set protected value (must be locked) |
| `is_locked` | `F is_locked(&self) -> i64` | Check if locked |

### AsyncChannel

Async-aware bounded channel with non-blocking send/receive.

```vais
S AsyncChannel { buffer: i64, capacity: i64, head: i64, tail: i64, len: i64, closed: i64 }
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(capacity: i64) -> AsyncChannel` | Create channel |
| `try_send` | `F try_send(&self, value: i64) -> i64` | Non-blocking send (0=ok, 1=full, 2=closed) |
| `try_recv` | `F try_recv(&self) -> i64` | Non-blocking receive (value or -1=empty, -2=closed) |
| `close` | `F close(&self) -> i64` | Close channel |
| `is_empty` | `F is_empty(&self) -> i64` | Check if empty |
| `is_full` | `F is_full(&self) -> i64` | Check if full |
| `pending` | `F pending(&self) -> i64` | Count pending messages |
| `is_closed` | `F is_closed(&self) -> i64` | Check if closed |
| `cleanup` | `F cleanup(&self) -> i64` | Free buffer memory |

### Debounce

Delays execution until input settles.

```vais
S Debounce { delay: i64, last_trigger: i64, pending: i64 }
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(delay: i64) -> Debounce` | Create with delay in ticks |
| `trigger` | `F trigger(&self, current_tick: i64) -> i64` | Trigger (resets timer) |
| `should_execute` | `F should_execute(&self, current_tick: i64) -> i64` | Check if should execute |

### Throttle

Limits execution rate.

```vais
S Throttle { interval: i64, last_exec: i64 }
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(interval: i64) -> Throttle` | Create with minimum interval |
| `try_execute` | `F try_execute(&self, current_tick: i64) -> i64` | Check if allowed (returns 1 if executed) |

## Helper Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `timeout` | `F timeout(inner_ptr, inner_poll, deadline) -> TimeoutFuture` | Wrap with timeout |
| `with_timeout` | `F with_timeout(inner_ptr, inner_poll, deadline) -> TimeoutFuture` | Create a timeout (alias) |
| `retry` | `F retry(max_retries: i64) -> RetryConfig` | Create retry config |
| `async_mutex` | `F async_mutex(value: i64) -> AsyncMutex` | Create async mutex |
| `async_channel` | `F async_channel(capacity: i64) -> AsyncChannel` | Create async channel |

## Usage

```vais
U std/async

F main() -> i64 {
    # AsyncChannel example
    ch := async_channel(32)
    ch.try_send(42)
    val := ch.try_recv()  # 42
    ch.close()

    # AsyncMutex example
    mtx := async_mutex(100)
    mtx.try_lock()
    val := mtx.get()  # 100
    mtx.set(200)
    mtx.unlock()

    # RetryConfig example
    cfg := retry(3)
    cfg.retries()  # 0
    cfg.remaining()  # 3

    # Debounce example
    db := Debounce::new(10)
    db.trigger(0)
    db.should_execute(15)  # 1

    # Throttle example
    th := Throttle::new(5)
    th.try_execute(0)  # 1
    th.try_execute(3)  # 0 (too soon)

    0
}
```
