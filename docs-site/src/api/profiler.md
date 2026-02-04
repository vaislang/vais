# Profiler API Reference

> Runtime performance profiling (timing, memory, call counts)

## Import

```vais
U std/profiler
```

## Constants

| Name | Value | Description |
|------|-------|-------------|
| `PROFILER_DISABLED` | 0 | Profiling off |
| `PROFILER_ENABLED` | 1 | Profiling on |
| `PROFILER_SAMPLING` | 2 | Sampling mode |

## Timer

High-resolution timer for measuring code sections.

```vais
S Timer { start_ns: i64, end_ns: i64, running: i64 }
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new() -> Timer` | Create timer |
| `start` | `F start(&self) -> Timer` | Start timing |
| `stop` | `F stop(&self) -> Timer` | Stop timing |
| `elapsed_ns` | `F elapsed_ns(&self) -> i64` | Nanoseconds elapsed |
| `elapsed_ms` | `F elapsed_ms(&self) -> i64` | Milliseconds elapsed |

## Usage

```vais
U std/profiler

F main() -> i64 {
    t := Timer::new()
    t.start()
    # ... code to measure ...
    t.stop()
    ms := t.elapsed_ms()
    0
}
```
