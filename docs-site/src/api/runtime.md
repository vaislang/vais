# Runtime API Reference

> Async task executor with cooperative scheduling and I/O reactor support

## Import

```vais
U std/runtime
```

## Constants

### Task Status

| Name | Value | Description |
|------|-------|-------------|
| `TASK_PENDING` | 0 | Task not yet started or waiting |
| `TASK_RUNNING` | 1 | Task currently executing |
| `TASK_READY` | 2 | Task ready to be polled (I/O available) |
| `TASK_COMPLETED` | 3 | Task finished successfully |

### Runtime Limits

| Name | Value | Description |
|------|-------|-------------|
| `MAX_TASKS` | 256 | Maximum concurrent tasks |

### Event Loop Constants

| Name | Value | Description |
|------|-------|-------------|
| `EVFILT_READ` | -1 | Read filter for kqueue |
| `EVFILT_WRITE` | -2 | Write filter for kqueue |
| `EVFILT_TIMER` | -7 | Timer filter for kqueue |
| `EV_ADD` | 1 | Add event to kqueue |
| `EV_DELETE` | 2 | Delete event from kqueue |
| `EV_ONESHOT` | 16 | One-shot event flag |
| `MAX_EVENTS` | 64 | Max events per kevent call |
| `WAKER_TOKEN` | -999 | Special token for waking event loop |

### Event Source Types

| Name | Value | Description |
|------|-------|-------------|
| `SOURCE_FD_READ` | 1 | Waiting for readable file descriptor |
| `SOURCE_FD_WRITE` | 2 | Waiting for writable file descriptor |
| `SOURCE_TIMER` | 3 | Waiting for timer deadline |

## Structs

### TaskNode

Represents a spawned async task in the executor.

**Fields:**
- `id: i64` - Unique task identifier
- `future_ptr: i64` - Pointer to future state
- `poll_fn: i64` - Function pointer for polling
- `status: i64` - Current task status
- `result: i64` - Task result value
- `next: i64` - Next task in linked list

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(id: i64, future_ptr: i64, poll_fn: i64) -> TaskNode` | Create new task node |
| `is_completed` | `F is_completed(&self) -> i64` | Check if task completed |
| `is_pending` | `F is_pending(&self) -> i64` | Check if task pending |

### JoinHandle

Handle returned when spawning a task, used to await results.

**Fields:**
- `task_id: i64` - Task identifier
- `task_ptr: i64` - Pointer to TaskNode

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(task_id: i64, task_ptr: i64) -> JoinHandle` | Create join handle |

### Runtime

Single-threaded async task scheduler (cooperative multitasking).

**Fields:**
- `task_count: i64` - Number of active tasks
- `next_task_id: i64` - Counter for task IDs
- `head: i64` - First task in queue
- `tail: i64` - Last task in queue
- `current_task: i64` - Currently executing task

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> Runtime` | Create new runtime |
| `spawn` | `F spawn(&self, future_ptr: i64, poll_fn: i64) -> JoinHandle` | Spawn task, returns handle |
| `run` | `F run(&self) -> i64` | Run all tasks to completion |
| `block_on` | `F block_on(&self, future_ptr: i64, poll_fn: i64) -> i64` | Block on single future |

### EventLoop

I/O reactor using kqueue (macOS/BSD) or epoll (Linux).

**Fields:**
- `kq: i64` - kqueue file descriptor
- `waker_read_fd: i64` - Pipe read end for waking
- `waker_write_fd: i64` - Pipe write end
- `sources_head: i64` - Event source list head
- `sources_tail: i64` - Event source list tail
- `source_count: i64` - Number of registered sources
- `events_buf: i64` - Buffer for kevent results
- `running: i64` - 1 if loop is active

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> EventLoop` | Create event loop |
| `register_read` | `F register_read(&self, fd: i64, task_ptr: i64) -> i64` | Wait for fd to be readable |
| `register_write` | `F register_write(&self, fd: i64, task_ptr: i64) -> i64` | Wait for fd to be writable |
| `register_timer` | `F register_timer(&self, timer_id: i64, delay_ms: i64, task_ptr: i64) -> i64` | Wait for timer |
| `deregister` | `F deregister(&self, fd: i64, filter: i64) -> i64` | Remove event source |
| `wake` | `F wake(&self) -> i64` | Wake up event loop |
| `poll_events` | `F poll_events(&self, timeout_ms: i64) -> i64` | Poll for I/O events |
| `event_fd` | `F event_fd(&self, index: i64) -> i64` | Get fd from event |
| `event_filter` | `F event_filter(&self, index: i64) -> i64` | Get filter from event |
| `find_task_for_fd` | `F find_task_for_fd(&self, fd: i64) -> i64` | Find waiting task |
| `cleanup` | `F cleanup(&self) -> i64` | Free all resources |

### ReactorRuntime

Async executor combined with I/O event loop (event-driven scheduling).

**Fields:**
- `task_count: i64` - Number of active tasks
- `next_task_id: i64` - Task ID counter
- `head: i64` - Task queue head
- `tail: i64` - Task queue tail
- `current_task: i64` - Currently running task
- `event_loop: i64` - Pointer to EventLoop
- `next_timer_id: i64` - Timer ID counter

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> ReactorRuntime` | Create reactor runtime |
| `spawn` | `F spawn(&self, future_ptr: i64, poll_fn: i64) -> JoinHandle` | Spawn task |
| `get_event_loop` | `F get_event_loop(&self) -> i64` | Get event loop pointer |
| `wait_readable` | `F wait_readable(&self, fd: i64) -> i64` | Wait for readable |
| `wait_writable` | `F wait_writable(&self, fd: i64) -> i64` | Wait for writable |
| `sleep_ms` | `F sleep_ms(&self, delay_ms: i64) -> i64` | Sleep for milliseconds |
| `run` | `F run(&self) -> i64` | Run all tasks with I/O |
| `block_on` | `F block_on(&self, future_ptr: i64, poll_fn: i64) -> i64` | Block on future with I/O |

### TaskGroup

Structured concurrency: spawn and manage a group of tasks. All tasks must complete before the group completes.

**Fields:**
- `name: i64` - Name pointer (for debugging)
- `head: i64` - Task entry list head
- `tail: i64` - Task entry list tail
- `task_count: i64` - Total tasks spawned
- `completed_count: i64` - Tasks completed
- `cancelled: i64` - 1 if group cancelled
- `cancel_on_error: i64` - Cancel siblings on error
- `results: i64` - Results array pointer
- `max_concurrency: i64` - Max concurrent tasks (0 = unlimited)

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> TaskGroup` | Create task group |
| `named` | `F named(name_ptr: i64) -> TaskGroup` | Create named group |
| `set_cancel_on_error` | `F set_cancel_on_error(&self, enabled: i64) -> i64` | Enable/disable error cancellation |
| `set_max_concurrency` | `F set_max_concurrency(&self, max: i64) -> i64` | Set concurrency limit |
| `spawn` | `F spawn(&self, future_ptr: i64, poll_fn: i64) -> i64` | Spawn task into group |
| `run` | `F run(&self) -> i64` | Run all tasks (0=success, 1=error) |
| `cancel` | `F cancel(&self) -> i64` | Cancel entire group |
| `cancel_remaining` | `F cancel_remaining(&self) -> i64` | Cancel pending tasks |
| `completed` | `F completed(&self) -> i64` | Get completed count |
| `total` | `F total(&self) -> i64` | Get total task count |
| `is_done` | `F is_done(&self) -> i64` | Check if all done |
| `is_cancelled` | `F is_cancelled(&self) -> i64` | Check if cancelled |
| `result` | `F result(&self, index: i64) -> i64` | Get result by index |
| `has_error` | `F has_error(&self, index: i64) -> i64` | Check error by index |
| `task_status` | `F task_status(&self, index: i64) -> i64` | Get status by index |
| `cleanup` | `F cleanup(&self) -> i64` | Free all entries |

### ScopedTask

Scoped task runner that guarantees cleanup on completion.

**Fields:**
- `group: i64` - Pointer to TaskGroup

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> ScopedTask` | Create scoped task |
| `spawn` | `F spawn(&self, future_ptr: i64, poll_fn: i64) -> i64` | Spawn task |
| `run_and_cleanup` | `F run_and_cleanup(&self) -> i64` | Run and cleanup |

## Global Functions

### Basic Runtime

| Function | Signature | Description |
|----------|-----------|-------------|
| `runtime_init` | `F runtime_init() -> i64` | Initialize global runtime |
| `get_runtime` | `F get_runtime() -> i64` | Get global runtime |
| `spawn` | `F spawn(future_ptr: i64, poll_fn: i64) -> JoinHandle` | Spawn on global runtime |
| `block_on` | `F block_on(future_ptr: i64, poll_fn: i64) -> i64` | Block on global runtime |
| `run_all` | `F run_all() -> i64` | Run all spawned tasks |

### Reactor Runtime

| Function | Signature | Description |
|----------|-----------|-------------|
| `reactor_init` | `F reactor_init() -> i64` | Initialize global reactor |
| `get_reactor` | `F get_reactor() -> i64` | Get global reactor |
| `reactor_spawn` | `F reactor_spawn(future_ptr: i64, poll_fn: i64) -> JoinHandle` | Spawn on reactor |
| `reactor_block_on` | `F reactor_block_on(future_ptr: i64, poll_fn: i64) -> i64` | Block on reactor |
| `reactor_run` | `F reactor_run() -> i64` | Run reactor event loop |
| `wait_readable` | `F wait_readable(fd: i64) -> i64` | Wait for readable |
| `wait_writable` | `F wait_writable(fd: i64) -> i64` | Wait for writable |
| `sleep_ms` | `F sleep_ms(delay_ms: i64) -> i64` | Sleep milliseconds |

### Structured Concurrency

| Function | Signature | Description |
|----------|-----------|-------------|
| `task_group` | `F task_group() -> TaskGroup` | Create task group |
| `task_group_named` | `F task_group_named(name_ptr: i64) -> TaskGroup` | Create named group |
| `scoped_task` | `F scoped_task() -> ScopedTask` | Create scoped task |

## Usage

### Basic Async Runtime

```vais
U std/runtime

# Initialize runtime
runtime_init()

# Spawn tasks
handle := spawn(future_ptr, poll_fn)

# Run until completion
run_all()
```

### I/O Reactor Runtime

```vais
U std/runtime

# Initialize reactor
reactor_init()

# Spawn I/O tasks
handle := reactor_spawn(future_ptr, poll_fn)

# Wait for I/O events
reactor_run()
```

### Structured Concurrency

```vais
U std/runtime

F main() -> i64 {
    group := task_group()
    group.set_cancel_on_error(1)

    # Spawn multiple tasks
    group.spawn(future1, poll_fn1)
    group.spawn(future2, poll_fn2)
    group.spawn(future3, poll_fn3)

    # Run all to completion
    result := group.run()

    # Cleanup
    group.cleanup()

    result
}
```

### Scoped Tasks

```vais
U std/runtime

F main() -> i64 {
    scoped := scoped_task()

    scoped.spawn(future1, poll_fn1)
    scoped.spawn(future2, poll_fn2)

    # Guaranteed cleanup even on error
    scoped.run_and_cleanup()
}
```

## Overview

The runtime module provides three levels of async execution:

1. **Basic Runtime**: Single-threaded cooperative multitasking. Tasks poll until Pending, then yield to the next task.

2. **Reactor Runtime**: Event-driven I/O scheduler using kqueue/epoll. Tasks can wait for file descriptor readiness, timers, and other events.

3. **Structured Concurrency**: TaskGroup and ScopedTask ensure all spawned tasks complete before the scope exits, preventing orphaned tasks.

All runtimes use cooperative scheduling: tasks must explicitly yield by returning Pending. This provides predictable execution without preemption.
