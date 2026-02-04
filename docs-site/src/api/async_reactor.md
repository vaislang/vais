# AsyncReactor API Reference

> Platform-independent event loop (kqueue/epoll/IOCP)

## Import

```vais
U std/async_reactor
```

## Constants

### Platform Constants

| Constant | Value | Backend |
|----------|-------|---------|
| `PLATFORM_UNKNOWN` | 0 | Unknown platform |
| `PLATFORM_MACOS` | 1 | kqueue |
| `PLATFORM_LINUX` | 2 | epoll |
| `PLATFORM_WINDOWS` | 3 | IOCP |

### Event Filter Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `REACTOR_READ` | -1 | Read readiness |
| `REACTOR_WRITE` | -2 | Write readiness |
| `REACTOR_TIMER` | -7 | Timer event |

### Event Action Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `REACTOR_ADD` | 1 | Add event |
| `REACTOR_DELETE` | 2 | Delete event |
| `REACTOR_ONESHOT` | 16 | Oneshot flag |

### Configuration Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `REACTOR_MAX_EVENTS` | 64 | Maximum events per poll |

## Structs

### ReactorEvent

A single I/O event.

```vais
S ReactorEvent { fd: i64, filter: i64 }
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(fd: i64, filter: i64) -> ReactorEvent` | Create event |
| `is_read` | `F is_read(&self) -> i64` | Check if read event |
| `is_write` | `F is_write(&self) -> i64` | Check if write event |
| `is_timer` | `F is_timer(&self) -> i64` | Check if timer event |

### ReactorSource

Tracks what a task is waiting on.

```vais
S ReactorSource { source_type: i64, fd: i64, task_ptr: i64, deadline_ms: i64, next: i64 }
```

### Reactor

Platform-independent event loop.

```vais
S Reactor {
    backend_fd: i64,
    platform: i64,
    waker_read_fd: i64,
    waker_write_fd: i64,
    sources_head: i64,
    sources_tail: i64,
    source_count: i64,
    events_buf: i64,
    running: i64
}
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> Reactor` | Create reactor for current platform |
| `get_platform` | `F get_platform(&self) -> i64` | Get current platform ID |
| `platform_name` | `F platform_name(&self) -> i64` | Get platform name string |
| `register_read` | `F register_read(&self, fd: i64, task_ptr: i64) -> i64` | Watch fd for read |
| `register_write` | `F register_write(&self, fd: i64, task_ptr: i64) -> i64` | Watch fd for write |
| `register_timer` | `F register_timer(&self, timer_id: i64, delay_ms: i64, task_ptr: i64) -> i64` | Register timer |
| `deregister` | `F deregister(&self, fd: i64, filter: i64) -> i64` | Remove event source |
| `add_source` | `F add_source(&self, source_type: i64, fd: i64, task_ptr: i64, deadline_ms: i64) -> i64` | Add to tracking list |
| `remove_source` | `F remove_source(&self, fd: i64) -> i64` | Remove from tracking list |
| `find_task_for_fd` | `F find_task_for_fd(&self, fd: i64) -> i64` | Find task waiting on fd |
| `wake` | `F wake(&self) -> i64` | Wake up reactor |
| `drain_waker` | `F drain_waker(&self) -> i64` | Drain waker pipe |
| `poll` | `F poll(&self, timeout_ms: i64) -> i64` | Poll for events (returns count) |
| `event_fd` | `F event_fd(&self, index: i64) -> i64` | Get fd of event at index |
| `event_filter` | `F event_filter(&self, index: i64) -> i64` | Get filter of event at index |
| `is_waker_event` | `F is_waker_event(&self, index: i64) -> i64` | Check if event is from waker |
| `process_events` | `F process_events(&self, n_events: i64) -> i64` | Wake tasks (returns tasks woken) |
| `cleanup` | `F cleanup(&self) -> i64` | Free all resources |

## Global Reactor Functions

These functions operate on the global reactor instance.

| Function | Signature | Description |
|----------|-----------|-------------|
| `reactor_instance_init` | `F reactor_instance_init() -> i64` | Initialize global reactor |
| `get_reactor_instance` | `F get_reactor_instance() -> i64` | Get global reactor |
| `reactor_register_read` | `F reactor_register_read(fd: i64, task_ptr: i64) -> i64` | Register read on global reactor |
| `reactor_register_write` | `F reactor_register_write(fd: i64, task_ptr: i64) -> i64` | Register write on global reactor |
| `reactor_register_timer` | `F reactor_register_timer(timer_id: i64, delay_ms: i64, task_ptr: i64) -> i64` | Register timer on global reactor |
| `reactor_poll` | `F reactor_poll(timeout_ms: i64) -> i64` | Poll global reactor |
| `reactor_process_events` | `F reactor_process_events(n_events: i64) -> i64` | Process events on global reactor |
| `reactor_wake` | `F reactor_wake() -> i64` | Wake global reactor |
| `reactor_cleanup` | `F reactor_cleanup() -> i64` | Cleanup global reactor |
| `reactor_get_platform` | `F reactor_get_platform() -> i64` | Get platform ID |

## Usage

### Using a local Reactor instance

```vais
U std/async_reactor

F main() -> i64 {
    reactor := Reactor::new()
    platform := reactor.get_platform()  # 1, 2, or 3

    reactor.register_read(socket_fd, task_ptr)
    reactor.register_timer(42, 1000, task_ptr)

    n := reactor.poll(1000)  # Wait 1 second
    reactor.process_events(n)

    reactor.cleanup()
    0
}
```

### Using the global reactor

```vais
U std/async_reactor

F main() -> i64 {
    reactor_register_read(socket_fd, task_ptr)

    n := reactor_poll(1000)
    reactor_process_events(n)

    reactor_cleanup()
    0
}
```
