# Signal API Reference

> POSIX signal handling operations

## Import

```vais
U std/signal
```

## Overview

The `signal` module provides POSIX signal constants and functions for registering signal handlers and sending signals. It wraps the C standard library `signal()` and `raise()` functions.

## Constants

### POSIX Signal Numbers (Portable)

| Constant | Value | Description |
|----------|-------|-------------|
| `SIGHUP` | 1 | Hangup |
| `SIGINT` | 2 | Interrupt (Ctrl+C) |
| `SIGQUIT` | 3 | Quit |
| `SIGILL` | 4 | Illegal instruction |
| `SIGTRAP` | 5 | Trace trap |
| `SIGABRT` | 6 | Abort |
| `SIGFPE` | 8 | Floating point exception |
| `SIGKILL` | 9 | Kill (cannot be caught) |
| `SIGSEGV` | 11 | Segmentation fault |
| `SIGPIPE` | 13 | Broken pipe |
| `SIGALRM` | 14 | Alarm clock |
| `SIGTERM` | 15 | Termination |

### Platform-Dependent Signal Numbers

| Constant | macOS | Linux |
|----------|-------|-------|
| `SIGBUS` | 10 | 7 |
| `SIGUSR1` | 30 | 10 |
| `SIGUSR2` | 31 | 12 |

### Special Signal Handler Values

| Constant | Value | Description |
|----------|-------|-------------|
| `SIG_DFL` | 0 | Default signal handling |
| `SIG_IGN` | 1 | Ignore signal |

## Functions

### signal_handle

```vais
F signal_handle(signum: i32, handler: i64) -> i64
```

Register a signal handler for the given signal.

**Parameters:**
- `signum`: The signal number (use constants above)
- `handler`: A function pointer to the handler, `SIG_DFL`, or `SIG_IGN`

**Returns:** The previous handler value.

### signal_raise

```vais
F signal_raise(signum: i32) -> i32
```

Send a signal to the current process.

**Parameters:**
- `signum`: The signal number to send

**Returns:** `0` on success, `-1` on error.

## Example

```vais
U std/signal

F main() {
    # Ignore SIGPIPE
    signal_handle(SIGPIPE, SIG_IGN)

    # Send SIGTERM to self
    signal_raise(SIGTERM)
}
```
