# Hot Reload Quick Start Guide

Get started with hot reloading in Vais in under 5 minutes!

## 1. Create a Hot-Reloadable Program

Create `game.vais`:

```vais
# Game state
S GameState {
    x: i64,
    y: i64,
    message: *i8,
}

# Mark this function for hot reload
#[hot]
F update_game(state: *GameState) -> i64 {
    # Update position - TRY CHANGING THESE VALUES!
    state.x = state.x + 1
    state.y = state.y + 2

    # Print state - TRY CHANGING THIS MESSAGE!
    printf("%s: x=%d, y=%d\n", state.message, state.x, state.y)

    0
}

# External C functions
@[extern "C"]
F printf(fmt: *i8, ...) -> i64

@[extern "C"]
F usleep(usec: i64) -> i64

# Main function (keeps state)
F main() -> i64 {
    state := GameState {
        x: 0,
        y: 0,
        message: "Position"
    }

    printf("Hot Reload Demo - Try changing update_game() while running!\n\n")

    # Main loop
    frame := 0
    L frame < 200 {
        update_game(&state)
        usleep(50000)  # 50ms delay
        frame = frame + 1
    }

    printf("\nDone!\n")
    0
}
```

## 2. Build with Hot Reload

```bash
vaisc build --hot game.vais
```

This creates `libgame.dylib` (macOS), `libgame.so` (Linux), or `libgame.dll` (Windows).

## 3. Watch for Changes (Optional)

In a **separate terminal**, start watch mode:

```bash
vaisc watch game.vais
```

This will automatically recompile whenever you save `game.vais`.

## 4. Run Your Program

In your **first terminal**, run the program:

```bash
./game
```

## 5. Try It Out!

While the program is running:

1. Open `game.vais` in your editor
2. Find the `update_game` function
3. Change the update logic, for example:
   ```vais
   state.x = state.x + 5  # Changed from 1 to 5
   state.y = state.y + 10 # Changed from 2 to 10
   ```
4. Save the file
5. Watch the running program immediately use the new code!

## What You'll See

### Before Changes
```
Hot Reload Demo - Try changing update_game() while running!

Position: x=1, y=2
Position: x=2, y=4
Position: x=3, y=6
...
```

### After Changes (while still running!)
```
...
Position: x=15, y=30
Position: x=20, y=40  # Values change instantly!
Position: x=25, y=50
...
```

## Common Use Cases

### Game Development
```vais
#[hot]
F update_physics(entity: *Entity, dt: f64) -> i64 {
    # Tweak physics in real-time!
}

#[hot]
F render(scene: *Scene) -> i64 {
    # Adjust rendering without restart!
}
```

### Web Servers
```vais
#[hot]
F handle_request(req: *Request) -> *Response {
    # Update API logic live!
}
```

### Data Processing
```vais
#[hot]
F process_data(input: *Data) -> *Result {
    # Refine algorithms on the fly!
}
```

## Tips

1. **Mark functions with #[hot]**: Only functions with `#[hot]` can be reloaded
2. **Keep state in main**: Don't store state in hot functions
3. **Use watch mode**: It auto-recompiles, saves time
4. **Test frequently**: Small changes are safer than big ones

## Next Steps

- Read the [full documentation](docs/HOT_RELOAD.md)
- Check out [advanced examples](examples/hot_reload_advanced.vais)
- Learn about [best practices](docs/HOT_RELOAD.md#best-practices)

## Help

If something goes wrong:

```bash
# Build with verbose output
vaisc build --hot -g game.vais

# Watch with verbose
vaisc watch game.vais --verbose

# Check for errors
vaisc check game.vais
```

## That's It!

You're now using hot reload in Vais. Happy coding! 🚀
