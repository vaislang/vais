# Hot Reloading in Vais

Hot reloading allows you to update your code while the program is running, without restarting. This is especially useful for game development, interactive applications, and rapid prototyping.

## Table of Contents

1. [Quick Start](#quick-start)
2. [How It Works](#how-it-works)
3. [Marking Functions for Hot Reload](#marking-functions-for-hot-reload)
4. [CLI Commands](#cli-commands)
5. [Runtime API](#runtime-api)
6. [Examples](#examples)
7. [Best Practices](#best-practices)
8. [Limitations](#limitations)
9. [Advanced Usage](#advanced-usage)

## Quick Start

### 1. Mark Functions as Hot-Reloadable

Add the `#[hot]` attribute to functions you want to reload at runtime:

```vais
#[hot]
F game_update(state: *GameState) -> i64 {
    # This function can be modified while running
    state.x = state.x + 1
    0
}
```

### 2. Build with Hot Reload

```bash
# Compile to dynamic library
vaisc build --hot game.vais
```

This creates:
- macOS: `libgame.dylib`
- Linux: `libgame.so`
- Windows: `libgame.dll`

### 3. Watch for Changes

In a separate terminal, start the watch mode:

```bash
vaisc watch game.vais
```

Now you can:
1. Run your program
2. Edit the `#[hot]` function
3. Save the file
4. The program automatically reloads the new code

## How It Works

```
┌─────────────────┐     ┌──────────────┐     ┌─────────────┐
│  FileWatcher    │────▶│   Compiler   │────▶│  Reloader   │
│  (notify crate) │     │  (vaisc)     │     │  (dylib)    │
└─────────────────┘     └──────────────┘     └─────────────┘
         │                                          │
         └────────────── File change ───────────────┘
                                │
                         ┌──────▼──────┐
                         │  Running    │
                         │  Program    │
                         └─────────────┘
```

### Process

1. **Compilation**: With `--hot` flag, functions marked `#[hot]` are compiled into a shared library (dylib)
2. **File Watching**: The system monitors source files for changes
3. **Recompilation**: When a change is detected, the source is recompiled
4. **Library Reload**: The new dylib is loaded, replacing the old version
5. **Function Pointers Updated**: References to hot functions are updated to point to new code

### Key Features

- **Automatic Detection**: File system watcher detects changes instantly
- **Debouncing**: Multiple rapid saves are batched to avoid excessive recompilation
- **Version Management**: Old dylib versions are cleaned up automatically
- **State Preservation**: Program state (variables, memory) is preserved across reloads

## Marking Functions for Hot Reload

### Syntax

```vais
#[hot]
F function_name(params) -> return_type {
    # Function body
}
```

### What Can Be Hot-Reloaded

✅ **Supported:**
- Function logic
- Control flow (if, loops, etc.)
- Calculations and expressions
- Function calls
- External function calls

❌ **Not Supported:**
- Function signatures (parameters, return type)
- Struct definitions
- Global variables
- Constants

### Example

```vais
S GameState {
    x: i64,
    y: i64,
    score: i64,
}

# Hot-reloadable update function
#[hot]
F update_game(state: *GameState, dt: f64) -> i64 {
    # You can modify this logic while running
    state.x = state.x + (10.0 * dt) as i64
    state.y = state.y + (5.0 * dt) as i64

    # Try changing the scoring logic!
    I state.x > 100 {
        state.score = state.score + 10
    }

    0
}

# Hot-reloadable render function
#[hot]
F render_game(state: *GameState) -> i64 {
    # Try changing the display format!
    printf("Position: (%d, %d) Score: %d\n",
           state.x, state.y, state.score)
    0
}

# Main loop - not hot-reloadable (holds state)
F main() -> i64 {
    state := GameState { x: 0, y: 0, score: 0 }

    L true {
        update_game(&state, 0.016)
        render_game(&state)
        usleep(16666)  # ~60 FPS
    }

    0
}
```

## CLI Commands

### Build Commands

#### Standard Build
```bash
vaisc build game.vais
```
Creates a regular executable.

#### Hot Reload Build
```bash
vaisc build --hot game.vais
```
Creates a shared library for hot reloading.

#### With Options
```bash
# With optimization
vaisc build --hot -O2 game.vais

# With debug symbols
vaisc build --hot -g game.vais

# Custom output path
vaisc build --hot game.vais -o bin/libgame.dylib
```

### Watch Command

#### Basic Watch
```bash
vaisc watch game.vais
```
Watches file and recompiles on changes.

#### Watch and Execute
```bash
vaisc watch game.vais --exec ./game
```
After successful compilation, runs the specified command.

#### With Arguments
```bash
vaisc watch game.vais --exec ./game -- arg1 arg2
```
Passes arguments to the executed command.

### Package Commands

```bash
# Build package with hot reload
vaisc pkg build --hot

# Watch package
vaisc watch src/main.vais
```

## Runtime API

The `std/hot.vais` module provides runtime hot reload functions.

### Functions

#### `hot_init(path: *i8) -> i64`
Initialize hot reload system.

```vais
hot_init("./game.vais")
```

#### `hot_check() -> i64`
Check for changes and reload if necessary.

Returns:
- `1`: Code was reloaded
- `0`: No changes
- `< 0`: Error

```vais
L true {
    status := hot_check()
    I status > 0 {
        printf("Code reloaded!\n")
    }

    # Run game logic
    game_update()
}
```

#### `hot_reload() -> i64`
Manually trigger a reload.

```vais
# Force reload
hot_reload()
```

#### `hot_version() -> i64`
Get current version number.

```vais
version := hot_version()
printf("Running version %d\n", version)
```

#### `hot_cleanup() -> i64`
Cleanup hot reload resources.

```vais
# Before exit
hot_cleanup()
```

### Higher-Level API

#### `hot_start(source_path: *i8) -> i64`
Convenience function to start hot reload.

```vais
hot_start("./game.vais")
```

#### `hot_loop(update_fn: fn() -> i64) -> i64`
Run an update function in a loop with automatic hot reload checking.

```vais
#[hot]
F game_update() -> i64 {
    # Game logic
    0
}

F main() -> i64 {
    hot_start("./game.vais")
    hot_loop(game_update)
    0
}
```

## Examples

### Simple Example

```vais
# examples/hot_reload_simple.vais

S GameState {
    frame: i64,
    x: i64,
    y: i64,
}

#[hot]
F game_update(state: *GameState) -> i64 {
    state.frame = state.frame + 1
    state.x = state.x + 1
    state.y = state.y + 2

    printf("Frame %d: x=%d, y=%d\n", state.frame, state.x, state.y)
    0
}

@[extern "C"]
F printf(fmt: *i8, ...) -> i64

@[extern "C"]
F usleep(usec: i64) -> i64

F main() -> i64 {
    state := GameState { frame: 0, x: 0, y: 0 }

    printf("Hot Reload Example - Try modifying game_update!\n\n")

    L state.frame < 1000 {
        game_update(&state)
        usleep(16666)  # ~60 FPS
    }

    0
}
```

### Advanced Example

See `examples/hot_reload_advanced.vais` for:
- Multiple hot functions
- Particle system simulation
- Physics updates
- Rendering

## Best Practices

### 1. Separate Logic from State

Keep state in the main function, logic in hot functions:

```vais
# ✅ Good: State in main, logic hot-reloadable
F main() -> i64 {
    state := GameState { ... }

    L true {
        update_logic(&state)  # Hot-reloadable
    }
}

#[hot]
F update_logic(state: *GameState) -> i64 {
    # All logic here
}
```

```vais
# ❌ Bad: State in hot function
#[hot]
F update() -> i64 {
    G state: GameState  # State lost on reload!
    # ...
}
```

### 2. Keep Function Signatures Stable

Don't change parameters or return types:

```vais
# ✅ Good: Same signature
#[hot]
F update(state: *GameState) -> i64 {
    # Logic can change
}

# Later...
#[hot]
F update(state: *GameState) -> i64 {
    # Different logic, same signature
}
```

```vais
# ❌ Bad: Changed signature
#[hot]
F update(state: *GameState) -> i64 { ... }

# Later...
#[hot]
F update(state: *GameState, delta: f64) -> i64 {  # Breaks!
```

### 3. Use Small, Focused Functions

Smaller functions reload faster and are easier to debug:

```vais
# ✅ Good: Focused functions
#[hot]
F update_physics(state: *GameState) -> i64 { ... }

#[hot]
F update_ai(state: *GameState) -> i64 { ... }

#[hot]
F render(state: *GameState) -> i64 { ... }
```

### 4. Handle Errors Gracefully

Check return values from hot functions:

```vais
result := game_update(&state)
I result != 0 {
    printf("Error in game_update: %d\n", result)
}
```

### 5. Test Before Deploying

Use watch mode during development:

```bash
# Terminal 1: Watch and recompile
vaisc watch game.vais

# Terminal 2: Run game
./game
```

## Limitations

### Technical Limitations

1. **Function Signatures**: Cannot change parameters or return types
2. **Data Structures**: Cannot modify struct/enum definitions while running
3. **Global State**: Global variables are not preserved across reloads
4. **Platform Dependencies**: Some platforms may lock loaded libraries

### Performance Considerations

1. **Compilation Time**: Large files take longer to recompile
2. **Reload Overhead**: Each reload has a small performance cost (~1-10ms)
3. **Memory**: Multiple versions use more memory (cleaned up automatically)

### Debugging

- Reloaded code may have different line numbers
- Breakpoints may need to be reset
- Stack traces may show old version numbers

## Advanced Usage

### Custom Reload Callbacks

```rust
// Using the Rust API
use vais_hotreload::{HotReloader, HotReloadConfig};

let mut reloader = HotReloader::new(config)?;

reloader.set_reload_callback(|path, version| {
    println!("Reloaded {} (v{})", path.display(), version);
    // Custom logic: clear caches, reset state, etc.
});
```

### Multiple Source Files

Watch multiple files:

```bash
vaisc watch game.vais &
vaisc watch renderer.vais &
```

### Configuration

```rust
let config = HotReloadConfig::new("game.vais")
    .with_debounce(200)           // Debounce in ms
    .with_compiler_args(vec![
        "-O2".to_string(),
        "-g".to_string(),
    ])
    .with_verbose(true);
```

### Integration with Build Systems

```bash
# Makefile
watch:
    vaisc watch src/main.vais --exec ./build/game

# Shell script
#!/bin/bash
vaisc build --hot src/*.vais
vaisc watch src/main.vais &
./game
```

## Troubleshooting

### Common Issues

**Q: Changes not detected**
- Check file watcher is running
- Verify file permissions
- Check debounce timing

**Q: Compilation errors**
- Syntax errors in hot function
- Invalid function signature change
- Missing dependencies

**Q: Crash on reload**
- Function signature changed
- Invalid memory access
- Null pointer in hot function

**Q: Performance degradation**
- Too many hot functions
- Large compilation times
- Memory leak in reload cycle

### Debug Mode

```bash
# Verbose output
vaisc build --hot -g game.vais

# Watch with verbose
vaisc watch game.vais --verbose
```

## Resources

- [vais-hotreload crate documentation](../crates/vais-hotreload/README.md)
- [Examples directory](../examples/)
- [Integration tests](../crates/vais-hotreload/tests/)

## Future Enhancements

Planned features:
- [ ] State serialization/deserialization
- [ ] Multi-threaded hot reload
- [ ] Remote hot reload (network)
- [ ] IDE integration
- [ ] Visual Studio Code extension
- [ ] Hot reload profiling tools
