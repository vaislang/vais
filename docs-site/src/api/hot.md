# Hot Reload API Reference

> Hot reloading support for rapid development iteration

**Status:** Stub implementation (not yet functional)

## Import

```vais
U std/hot
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `hot_init` | `F hot_init(path: i64) -> i64` | Initialize hot reload for source file |
| `hot_check` | `F hot_check() -> i64` | Check for file changes |
| `hot_reload` | `F hot_reload() -> i64` | Force reload |
| `hot_shutdown` | `F hot_shutdown() -> i64` | Shutdown hot reload system |

## Overview

Functions marked with `#[hot]` can be reloaded at runtime without restarting the program. The hot reload system watches source files for changes and recompiles/reloads modified functions.

## Usage

```vais
U std/hot

#[hot]
F game_update(state: i64) -> i64 { 0 }

F main() -> i64 {
    hot_init("./game.vais")
    L 1 {
        hot_check()
        game_update(state)
    }
    0
}
```
