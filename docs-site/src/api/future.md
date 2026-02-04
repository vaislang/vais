# Future API Reference

> Stackless coroutine-based async/await with future combinators

## Import

```vais
U std/future
```

## Core Types

### Poll

```vais
E Poll {
    Pending,
    Ready(i64)
}
```

Result type for polling futures.

| Method | Signature | Description |
|--------|-----------|-------------|
| `is_ready` | `F is_ready(&self) -> i64` | Returns 1 if Ready, 0 otherwise |
| `is_pending` | `F is_pending(&self) -> i64` | Returns 1 if Pending, 0 otherwise |
| `unwrap` | `F unwrap(&self) -> i64` | Get value (0 if Pending, should panic) |

### Future (trait)

```vais
W Future {
    F poll(&self, ctx: i64) -> Poll
}
```

Trait for asynchronous values. Types implementing this can be polled for completion.

### Context

```vais
S Context {
    waker_ptr: i64,
    runtime_ptr: i64
}
```

Context passed to poll operations, contains waker and runtime information.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> Context` | Create empty context |
| `with_runtime` | `F with_runtime(runtime_ptr: i64) -> Context` | Create context with runtime pointer |
| `wake` | `F wake(&self) -> i64` | Wake up the task associated with this context |

### Waker

```vais
S Waker {
    task_ptr: i64,
    wake_fn: i64
}
```

Mechanism to wake up a suspended task.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(task_ptr: i64, wake_fn: i64) -> Waker` | Create waker with task and wake function |
| `wake` | `F wake(&self) -> i64` | Signal the runtime that this task is ready |

## Future Combinators

### MapFuture

```vais
S MapFuture {
    inner_ptr: i64,     # Pointer to inner future
    inner_poll: i64,    # Poll function of inner future
    map_fn: i64,        # Mapping function pointer
    state: i64          # 0 = not polled, 1 = complete
}
```

Transforms the output of a future using a mapping function.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(inner_ptr: i64, inner_poll: i64, map_fn: i64) -> MapFuture` | Create map combinator |

### AndThenFuture

```vais
S AndThenFuture {
    first_ptr: i64,     # First future
    first_poll: i64,
    second_fn: i64,     # Function that creates second future from first result
    state: i64,         # 0 = running first, 1 = running second
    second_ptr: i64,    # Created second future (when state = 1)
    second_poll: i64
}
```

Chains futures sequentially - runs second future after first completes.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(first_ptr: i64, first_poll: i64, second_fn: i64) -> AndThenFuture` | Create sequential chain |

### JoinFuture

```vais
S JoinFuture {
    first_ptr: i64,
    first_poll: i64,
    second_ptr: i64,
    second_poll: i64,
    first_done: i64,
    second_done: i64,
    first_result: i64,
    second_result: i64
}
```

Runs two futures concurrently, completes when both finish.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(first_ptr: i64, first_poll: i64, second_ptr: i64, second_poll: i64) -> JoinFuture` | Create join combinator |

### SelectFuture

```vais
S SelectFuture {
    first_ptr: i64,
    first_poll: i64,
    second_ptr: i64,
    second_poll: i64
}
```

Returns when either future completes (race condition).

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(first_ptr: i64, first_poll: i64, second_ptr: i64, second_poll: i64) -> SelectFuture` | Create select combinator |

### ReadyFuture

```vais
S ReadyFuture {
    value: i64
}
```

Future that immediately resolves to a value.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(value: i64) -> ReadyFuture` | Create immediately-ready future |

### PendingFuture

```vais
S PendingFuture {
    _dummy: i64
}
```

Future that never resolves (always returns Pending).

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> PendingFuture` | Create never-resolving future |

### TimerFuture

```vais
S TimerFuture {
    deadline: i64,      # Target tick count
    started: i64
}
```

Future that completes after N iterations.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(ticks: i64) -> TimerFuture` | Create timer future |

### YieldNow

```vais
S YieldNow {
    yielded: i64
}
```

Cooperative scheduling yield point. Returns Pending on first poll, Ready on second.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> YieldNow` | Create yield point |

## AsyncDrop Support

### AsyncDrop (trait)

```vais
W AsyncDrop {
    A F async_drop(&self) -> i64
}
```

Trait for types that need async cleanup when they go out of scope.

### AsyncDropGuard

```vais
S AsyncDropGuard {
    value_ptr: i64,         # Pointer to the value
    drop_fn: i64,           # Async drop function pointer (poll-based)
    dropped: i64            # 1 if already dropped
}
```

Wraps a value implementing AsyncDrop and ensures async_drop is called.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(value_ptr: i64, drop_fn: i64) -> AsyncDropGuard` | Create guard for value |
| `drop_async` | `A F drop_async(&self) -> i64` | Manually trigger async drop |
| `is_dropped` | `F is_dropped(&self) -> i64` | Check if already dropped |
| `get` | `F get(&self) -> i64` | Get wrapped value pointer |

### AsyncDropScope

```vais
S AsyncDropScope {
    head: i64,          # First guard in linked list
    count: i64          # Number of guards
}
```

Manages multiple AsyncDrop resources. All resources are dropped in reverse order (LIFO) when scope ends.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> AsyncDropScope` | Create empty scope |
| `register` | `F register(&self, value_ptr: i64, drop_fn: i64) -> AsyncDropGuard` | Register resource for async drop |
| `drop_all` | `F drop_all(&self) -> i64` | Drop all resources in reverse order, returns error count |
| `len` | `F len(&self) -> i64` | Get number of registered resources |

## Helper Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `ready` | `F ready(value: i64) -> ReadyFuture` | Create immediately-ready future |
| `pending` | `F pending() -> PendingFuture` | Create never-resolving future |
| `join` | `F join(first_ptr: i64, first_poll: i64, second_ptr: i64, second_poll: i64) -> JoinFuture` | Join two futures (both must complete) |
| `select` | `F select(first_ptr: i64, first_poll: i64, second_ptr: i64, second_poll: i64) -> SelectFuture` | Select first completing future |
| `delay` | `F delay(ticks: i64) -> TimerFuture` | Create timer that completes after n iterations |
| `yield_now` | `F yield_now() -> YieldNow` | Create yield point for cooperative scheduling |
| `async_drop_guard` | `F async_drop_guard(value_ptr: i64, drop_fn: i64) -> AsyncDropGuard` | Create async drop guard |
| `async_drop_scope` | `F async_drop_scope() -> AsyncDropScope` | Create async drop scope |

## Usage

### Basic Future

```vais
U std/future

F main() -> i64 {
    # Create immediately-ready future
    f := ready(42)

    # In async context: poll the future
    ctx := Context::new()
    result := f.poll(ctx)

    I result.is_ready() {
        value := result.unwrap()  # value = 42
    }
    0
}
```

### Join Futures

```vais
U std/future

F main() -> i64 {
    f1 := ready(10)
    f2 := ready(20)

    # Join both futures
    joined := join(f1_ptr, f1_poll, f2_ptr, f2_poll)

    # Poll until both complete
    0
}
```

### Async Drop

```vais
U std/future

# Example: Connection with async cleanup
S MyConn {
    handle: i64
}

X MyConn: AsyncDrop {
    A F async_drop(&self) -> i64 {
        # Close connection asynchronously
        close_connection(self.handle)
    }
}

F main() -> i64 {
    scope := async_drop_scope()

    # Register resource for async cleanup
    conn := MyConn { handle: open_connection() }
    guard := scope.register(conn_ptr, drop_fn)

    # Do work...

    # All resources cleaned up in reverse order
    scope.drop_all()
    0
}
```
