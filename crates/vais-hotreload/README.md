# vais-hotreload

Hot reloading support for the Vais programming language. Enables runtime code updates without restarting your program.

## Features

- **File Watching**: Automatically detects source file changes using the `notify` crate
- **Dynamic Library Loading**: Loads and reloads compiled code at runtime using `libloading`
- **Debouncing**: Prevents excessive recompilations with configurable debounce timing
- **Version Management**: Tracks reload versions and cleans up old library files
- **Cross-platform**: Supports macOS (.dylib), Linux (.so), and Windows (.dll)

## Architecture

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

## Usage

### Rust API

```rust
use vais_hotreload::{HotReloader, HotReloadConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure hot reloader
    let config = HotReloadConfig::new("./game.vais")
        .with_debounce(100)
        .with_verbose(true);

    // Create and start reloader
    let mut reloader = HotReloader::new(config)?;
    reloader.start()?;

    // Main loop
    loop {
        // Check for code changes
        if reloader.check()? {
            println!("Code reloaded!");
        }

        // Get and call hot function
        let update_fn: libloading::Symbol<unsafe extern fn()> =
            reloader.get_function("game_update")?;
        unsafe { update_fn() };

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
```

### Vais Language

Mark functions with `#[hot]` attribute to enable hot reloading:

```vais
# Hot-reloadable function
#[hot]
F game_update(state: *GameState) -> i64 {
    # This function can be modified at runtime
    state.x = state.x + 1
    0
}

F main() -> i64 {
    state := GameState { x: 0, y: 0 }

    # Main loop - the hot function can be updated while running
    L true {
        game_update(&state)
        sleep(16)  # ~60fps
    }
    0
}
```

### CLI Usage

#### Build with Hot Reload

```bash
# Compile to dynamic library
vaisc build --hot game.vais

# This creates libgame.dylib (macOS) or libgame.so (Linux)
```

#### Watch Mode

```bash
# Watch file and recompile on changes
vaisc watch game.vais

# Watch and execute command after successful compilation
vaisc watch game.vais --exec ./game
```

## Configuration

### HotReloadConfig Options

- `source_path`: Source file to watch (required)
- `output_dir`: Directory for compiled dylib (default: same as source)
- `compiler_command`: Compiler to use (default: "vaisc")
- `compiler_args`: Additional compiler arguments
- `debounce_ms`: Debounce duration in milliseconds (default: 100)
- `compile_timeout_secs`: Compilation timeout (default: 30)
- `verbose`: Enable verbose output (default: false)

## Components

### FileWatcher

Monitors source files for changes using the `notify` crate.

```rust
use vais_hotreload::FileWatcher;

let mut watcher = FileWatcher::new()?;
watcher.watch("game.vais")?;

// Non-blocking check
if let Some(event) = watcher.check()? {
    println!("File changed: {:?}", event);
}

// Blocking wait
let event = watcher.wait()?;
```

### DylibLoader

Manages dynamic library loading and unloading.

```rust
use vais_hotreload::DylibLoader;

let mut loader = DylibLoader::new("libgame.dylib")?;
loader.load()?;

// Get function symbol
let func: libloading::Symbol<extern fn()> = loader.get_function("my_function")?;

// Reload
loader.load()?;  // Automatically unloads old version

// Cleanup old versions
loader.cleanup_old_versions()?;
```

### HotReloader

High-level coordinator that combines file watching and library loading.

```rust
use vais_hotreload::{HotReloader, HotReloadConfig};

let config = HotReloadConfig::new("game.vais");
let mut reloader = HotReloader::new(config)?;
reloader.start()?;

// Set reload callback
reloader.set_reload_callback(|path, version| {
    println!("Reloaded {} (version {})", path.display(), version);
});

// Check for changes
if reloader.check()? {
    println!("New version: {}", reloader.version());
}
```

## Examples

See the `examples/` directory for complete examples:

- `hot_reload_simple.vais`: Basic hot reload example
- `hot_reload_advanced.vais`: Multiple hot functions with particle system

## Implementation Details

### Dylib Versioning

To allow reloading on platforms that lock loaded libraries, the loader creates versioned copies:

```
libgame.dylib          # Original
libgame.v1.dylib       # Loaded version 1
libgame.v2.dylib       # Loaded version 2 (after reload)
```

Old versions are automatically cleaned up.

### Debouncing

File system events can fire multiple times for a single save. The debouncer prevents excessive recompilations by ignoring events within the configured window (default: 100ms).

### Compilation

When using `--hot` flag, vaisc compiles code as a shared library:

```bash
clang -shared -fPIC -o libgame.dylib game.ll
```

## Limitations

- Hot-reloadable functions cannot change their signature
- Static/global state is not preserved across reloads (by design)
- Some platforms may have limitations on library unloading

## Testing

```bash
# Run unit tests
cargo test -p vais-hotreload

# Run integration tests
cargo test -p vais-hotreload --test integration_test
```

## License

MIT
