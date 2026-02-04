# Runtime API Reference

> Single-threaded async task executor with cooperative scheduling

## Import

```vais
U std/runtime
```

## Constants

| Name | Value | Description |
|------|-------|-------------|
| `TASK_PENDING` | 0 | Task not yet started |
| `TASK_RUNNING` | 1 | Task currently running |
| `TASK_READY` | 2 | Task ready to be polled |
| `TASK_COMPLETED` | 3 | Task finished |
| `MAX_TASKS` | 256 | Maximum concurrent tasks |

## Structs

### TaskNode

Represents a spawned async task in the executor.

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `F new(id: i64, future_ptr: i64, poll_fn: i64) -> TaskNode` | Create task |
| `is_completed` | `F is_completed(&self) -> i64` | Check if done |
| `is_pending` | `F is_pending(&self) -> i64` | Check if pending |

## Overview

The runtime provides a single-threaded event loop that polls futures until completion. Tasks are scheduled cooperatively -- each task runs until it returns `Pending`, then the runtime moves to the next task.

## Usage

```vais
U std/runtime

# Tasks are spawned into the runtime and polled
# until all complete or return Pending
```
