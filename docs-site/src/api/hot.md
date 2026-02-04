# Hot Reload API Reference

> Hot reloading support for rapid development iteration

**Status:** Stub implementation (not yet functional)

## Import

```vais
U std/hot
```

## Functions

## Structures

### HotReloadContext

```vais
S HotReloadContext { internal: i64 }
```

## Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `hot_init` | `F hot_init(path: i64) -> i64` | Initialize hot reload for source file |
| `hot_check` | `F hot_check() -> i64` | Check for file changes |
| `hot_reload` | `F hot_reload() -> i64` | Force reload |
| `hot_version` | `F hot_version() -> i64` | Get current version number |
| `hot_on_reload` | `F hot_on_reload(callback: fn(i64) -> void) -> i64` | Set reload callback |
| `hot_cleanup` | `F hot_cleanup() -> i64` | Cleanup hot reload system |
| `hot_start` | `F hot_start(path: i64) -> i64` | Start with default config |
| `hot_loop` | `F hot_loop(update_fn: fn() -> i64) -> i64` | Main hot reload loop |

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
