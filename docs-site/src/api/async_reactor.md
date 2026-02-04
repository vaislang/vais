# AsyncReactor API Reference

> Platform-independent event loop (kqueue/epoll/IOCP)

## Import

```vais
U std/async_reactor
```

## Platforms

| Constant | Value | Backend |
|----------|-------|---------|
| `PLATFORM_MACOS` | 1 | kqueue |
| `PLATFORM_LINUX` | 2 | epoll |
| `PLATFORM_WINDOWS` | 3 | IOCP |

## Event Filters

| Constant | Value | Description |
|----------|-------|-------------|
| `REACTOR_READ` | -1 | Read readiness |
| `REACTOR_WRITE` | -2 | Write readiness |
| `REACTOR_TIMER` | -7 | Timer event |

## Key Struct: Reactor

| Method | Description |
|--------|-------------|
| `new()` | Create reactor for current platform |
| `register_read(fd, task_ptr)` | Watch fd for read |
| `register_write(fd, task_ptr)` | Watch fd for write |
| `register_timer(id, delay_ms, task_ptr)` | Register timer |
| `deregister(fd)` | Remove fd |
| `poll(timeout_ms)` | Poll for events |
| `event_fd(index)` | Get fd of event at index |
| `event_data(index)` | Get user data of event |
| `free()` | Free reactor |

## Usage

```vais
U std/async_reactor

F main() -> i64 {
    reactor := Reactor::new()
    reactor.register_read(socket_fd, task_ptr)
    n := reactor.poll(1000)  # Wait 1 second
    # Process n events
    reactor.free()
    0
}
```
