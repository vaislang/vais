# Thread API Reference

> OS-level threading with thread pools and scoped threads

## Import

```vais
U std/thread
```

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `THREAD_NEW` | 0 | Thread created but not started |
| `THREAD_RUNNING` | 1 | Thread is running |
| `THREAD_FINISHED` | 2 | Thread has finished |
| `THREAD_DETACHED` | 3 | Thread is detached |

## Structs

### JoinHandle\<T\>

Handle for a spawned thread. Call `join()` to wait for completion.

| Method | Signature | Description |
|--------|-----------|-------------|
| `join` | `F join(&self) -> T?` | Wait and get result |
| `is_finished` | `F is_finished(&self) -> i64` | Check if done |
| `id` | `F id(&self) -> i64` | Get thread ID |

### Thread

Represents a running thread with id and name.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(id: i64, handle: i64) -> Thread` | Create thread handle |
| `with_name` | `F with_name(id: i64, handle: i64, name: str) -> Thread` | Create with name |
| `id` | `F id(&self) -> i64` | Get thread ID |
| `name` | `F name(&self) -> str` | Get thread name |
| `unpark` | `F unpark(&self) -> i64` | Unpark the thread |

### ThreadBuilder

Configurable thread spawner (set name, stack size).

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> ThreadBuilder` | Create builder |
| `name` | `F name(&self, name: str) -> ThreadBuilder` | Set thread name |
| `stack_size` | `F stack_size(&self, size: i64) -> ThreadBuilder` | Set stack size |
| `spawn` | `F spawn(&self, fn_ptr: i64, arg: i64) -> JoinHandle<i64>` | Spawn with options |

### ThreadLocal\<T\>

Thread-local storage.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(init_fn: i64) -> ThreadLocal<T>` | Create with initializer |
| `get` | `F get(&self) -> &T` | Get thread-local value |
| `set` | `F set(&self, value: T) -> i64` | Set thread-local value |

### ThreadPool

Basic thread pool for submitting tasks.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(num_threads: i64) -> ThreadPool` | Create pool |
| `submit` | `F submit(&self, fn_ptr: i64, arg: i64) -> i64` | Submit task |
| `shutdown` | `F shutdown(&self) -> i64` | Shutdown pool |

### Scope

Scoped threads that auto-join on scope exit.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> Scope` | Create scope |
| `spawn` | `F spawn(&self, fn_ptr: i64, arg: i64) -> i64` | Spawn scoped thread |
| `join_all` | `F join_all(&self) -> i64` | Wait for all threads |

## Key Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `spawn` | `F spawn(fn_ptr: i64, arg: i64) -> JoinHandle<i64>` | Spawn a new thread |
| `spawn_closure` | `F spawn_closure(closure_ptr: i64, env_ptr: i64) -> JoinHandle<i64>` | Spawn with closure |
| `current` | `F current() -> Thread` | Get current thread |
| `yield_now` | `F yield_now() -> i64` | Yield to other threads |
| `sleep_ms` | `F sleep_ms(ms: i64) -> i64` | Sleep milliseconds |
| `sleep` | `F sleep(secs: i64) -> i64` | Sleep seconds |
| `park` | `F park() -> i64` | Park current thread |
| `park_timeout` | `F park_timeout(ms: i64) -> i64` | Park with timeout |
| `builder` | `F builder() -> ThreadBuilder` | Create thread builder |
| `create_pool` | `F create_pool(num_threads: i64) -> ThreadPool` | Create thread pool |
| `scope` | `F scope(scope_fn: i64) -> i64` | Run with scoped threads |
| `available_parallelism` | `F available_parallelism() -> i64` | Get CPU core count |

## Usage

```vais
U std/thread

F worker(arg: i64) -> i64 { arg * 2 }

F main() -> i64 {
    handle := spawn(worker, 21)
    result := handle.join()  # Some(42)
    0
}
```
