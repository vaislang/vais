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

### AsyncMutex

Async-aware mutual exclusion (non-blocking try_lock).

### AsyncChannel

Async-aware bounded channel with non-blocking send/receive.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(capacity: i64) -> AsyncChannel` | Create channel |
| `try_send` | `F try_send(&self, value: i64) -> i64` | Non-blocking send (0=ok, 1=full) |
| `try_recv` | `F try_recv(&self) -> i64` | Non-blocking receive |
| `close` | `F close(&self) -> i64` | Close channel |
| `pending` | `F pending(&self) -> i64` | Count pending messages |

### Debounce / Throttle

Rate-limiting utilities for event handling.

## Helper Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `timeout` | `F timeout(inner_ptr, inner_poll, deadline) -> TimeoutFuture` | Wrap with timeout |
| `retry` | `F retry(max_retries: i64) -> RetryConfig` | Create retry config |
| `async_mutex` | `F async_mutex(value: i64) -> AsyncMutex` | Create async mutex |
| `async_channel` | `F async_channel(capacity: i64) -> AsyncChannel` | Create async channel |

## Usage

```vais
U std/async

F main() -> i64 {
    ch := async_channel(32)
    ch.try_send(42)
    val := ch.try_recv()  # 42
    ch.close()
    0
}
```
