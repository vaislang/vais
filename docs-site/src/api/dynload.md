# DynLoad API Reference

> Dynamic module loading, WASM sandboxing, and hot reload support

## Import

```vais
U std/dynload
```

## Overview

The `dynload` module provides comprehensive support for:
- Runtime loading and unloading of dynamic libraries
- WASM plugin sandboxing with resource limits and capabilities
- Hot reload for development workflows
- Plugin discovery from standard paths

## Constants

### Loading Flags

| Name | Value | Description |
|------|-------|-------------|
| `RTLD_NOW` | 2 | Load library immediately (resolve all symbols) |
| `RTLD_LAZY` | 1 | Lazy loading (resolve symbols as needed) |
| `RTLD_GLOBAL` | 256 | Make symbols available to subsequently loaded libraries |

### Plugin Capabilities

| Name | Value | Description |
|------|-------|-------------|
| `CAP_NONE` | 0 | No capabilities |
| `CAP_CONSOLE` | 1 | Console output |
| `CAP_TIME` | 2 | Time/clock access |
| `CAP_RANDOM` | 4 | Random number generation |
| `CAP_FS_READ` | 8 | File system read |
| `CAP_FS_WRITE` | 16 | File system write |
| `CAP_NETWORK` | 32 | Network access |
| `CAP_ENV` | 64 | Environment variables |
| `CAP_PROCESS` | 128 | Process spawning |
| `CAP_THREADING` | 256 | Multi-threading |
| `CAP_GPU` | 512 | GPU/compute access |

## Structs

### ModuleHandle

Represents a loaded dynamic module.

```vais
S ModuleHandle {
    handle: i64,      # dlopen handle
    path: i64,        # Path to the module (string pointer)
    version: i64,     # Module version (incremented on reload)
    loaded: i64       # 1 if loaded, 0 if not
}
```

**Functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `module_handle_new` | `F module_handle_new() -> ModuleHandle` | Create empty module handle |
| `module_is_loaded` | `F module_is_loaded(m: ModuleHandle) -> i64` | Check if module is loaded |
| `module_version` | `F module_version(m: ModuleHandle) -> i64` | Get module version |

### ResourceLimits

Resource limits for sandboxed execution.

```vais
S ResourceLimits {
    max_memory_bytes: i64,     # Maximum memory in bytes
    max_time_ms: i64,          # Maximum execution time in milliseconds
    max_stack_bytes: i64,      # Maximum stack size
    max_call_depth: i64        # Maximum function call depth
}
```

**Functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `default_limits` | `F default_limits() -> ResourceLimits` | 64MB RAM, 5s timeout, 1MB stack, 1000 depth |
| `restrictive_limits` | `F restrictive_limits() -> ResourceLimits` | 16MB RAM, 1s timeout, 256KB stack, 500 depth |
| `permissive_limits` | `F permissive_limits() -> ResourceLimits` | 256MB RAM, 60s timeout, 4MB stack, 5000 depth |

### WasmSandbox

WASM sandbox handle for plugin execution.

```vais
S WasmSandbox {
    handle: i64,
    capabilities: i64
}
```

**Functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `sandbox_new` | `F sandbox_new() -> WasmSandbox` | Create new sandbox with default settings (console enabled) |
| `sandbox_restrictive` | `F sandbox_restrictive() -> WasmSandbox` | Create restrictive sandbox for untrusted plugins (no caps) |
| `sandbox_destroy` | `F sandbox_destroy(s: WasmSandbox) -> i64` | Destroy sandbox |
| `sandbox_grant` | `F sandbox_grant(s: WasmSandbox, cap: i64) -> WasmSandbox` | Grant capability to sandbox |
| `sandbox_revoke` | `F sandbox_revoke(s: WasmSandbox, cap: i64) -> WasmSandbox` | Revoke capability from sandbox |
| `sandbox_load` | `F sandbox_load(s: WasmSandbox, bytes: i64, len: i64) -> WasmInstance` | Load WASM module into sandbox |

### WasmInstance

WASM instance handle for calling functions.

```vais
S WasmInstance {
    handle: i64,
    sandbox: i64
}
```

**Functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `wasm_call` | `F wasm_call(inst: WasmInstance, name: i64) -> i64` | Call function with no args |
| `wasm_call1` | `F wasm_call1(inst: WasmInstance, name: i64, arg: i64) -> i64` | Call function with one i64 arg |
| `wasm_call2` | `F wasm_call2(inst: WasmInstance, name: i64, arg1: i64, arg2: i64) -> i64` | Call function with two i64 args |
| `wasm_is_valid` | `F wasm_is_valid(inst: WasmInstance) -> i64` | Check if instance is valid |

### HotReloadConfig

Configuration for hot reload.

```vais
S HotReloadConfig {
    source_path: i64,      # Path to source file
    output_dir: i64,       # Output directory for compiled modules
    debounce_ms: i64,      # Debounce time for file changes
    verbose: i64           # Enable verbose logging
}
```

**Functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `hot_reload_config` | `F hot_reload_config(source_path: i64) -> HotReloadConfig` | Create default hot reload config (100ms debounce, non-verbose) |

### HotReloader

Hot reloader handle.

```vais
S HotReloader {
    handle: i64,
    version: i64,
    running: i64
}
```

**Functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `hot_reloader_new` | `F hot_reloader_new(source_path: i64) -> HotReloader` | Create new hot reloader |
| `hot_reloader_start` | `F hot_reloader_start(r: HotReloader) -> HotReloader` | Start hot reloading |
| `hot_reloader_stop` | `F hot_reloader_stop(r: HotReloader) -> HotReloader` | Stop hot reloading |
| `hot_reloader_check` | `F hot_reloader_check(r: HotReloader) -> i64` | Check for changes and reload (returns 1 if reloaded) |
| `hot_reloader_version` | `F hot_reloader_version(r: HotReloader) -> i64` | Get current version |

## Module Loading Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `load_module` | `F load_module(path: i64) -> ModuleHandle` | Load dynamic library with RTLD_NOW |
| `load_module_lazy` | `F load_module_lazy(path: i64) -> ModuleHandle` | Load with lazy binding (RTLD_LAZY) |
| `unload_module` | `F unload_module(m: ModuleHandle) -> i64` | Unload module (returns 1 on success) |
| `reload_module` | `F reload_module(m: ModuleHandle) -> ModuleHandle` | Reload module with incremented version |
| `get_function` | `F get_function(m: ModuleHandle, name: i64) -> i64` | Get function pointer by name |
| `get_load_error` | `F get_load_error() -> i64` | Get last error message |

## Capability System Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `has_capability` | `F has_capability(caps: i64, required: i64) -> i64` | Check if capability is granted |
| `add_capability` | `F add_capability(caps: i64, cap: i64) -> i64` | Add capability to flags |
| `remove_capability` | `F remove_capability(caps: i64, cap: i64) -> i64` | Remove capability from flags |
| `has_dangerous_capabilities` | `F has_dangerous_capabilities(caps: i64) -> i64` | Check for FS_WRITE, NETWORK, PROCESS, or ENV |

## Utility Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `get_library_extension` | `F get_library_extension() -> i64` | Get platform-specific library extension (dylib/so/dll) |
| `is_plugin_library` | `F is_plugin_library(path: i64) -> i64` | Check if file is a plugin library |

## External C Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `dlopen` | `X F dlopen(path: i64, flags: i64) -> i64` | Open dynamic library |
| `dlclose` | `X F dlclose(handle: i64) -> i64` | Close library |
| `dlsym` | `X F dlsym(handle: i64, symbol: i64) -> i64` | Get symbol address |
| `dlerror` | `X F dlerror() -> i64` | Get error message |
| `vais_get_user_plugin_dir` | `X F vais_get_user_plugin_dir() -> i64` | Get user plugin directory (~/.vais/plugins/) |
| `vais_get_system_plugin_dir` | `X F vais_get_system_plugin_dir() -> i64` | Get system plugin directory (/usr/local/lib/vais/plugins/) |
| `getenv` | `X F getenv(name: i64) -> i64` | Get environment variable |

## Examples

### Basic Dynamic Loading

```vais
U std/dynload

F main() -> i64 {
    # Load a dynamic library
    module := load_module("/path/to/plugin.dylib")

    I module_is_loaded(module) == 1 {
        # Get function pointer
        init_fn := get_function(module, "plugin_init")

        # Call the function (would need to cast and call)
        # ...

        # Unload when done
        unload_module(module)
    } E {
        # Error loading
        err := get_load_error()
        # Handle error
    }

    0
}
```

### WASM Sandbox with Capabilities

```vais
U std/dynload

F main() -> i64 {
    # Create restrictive sandbox
    sandbox := sandbox_restrictive()

    # Grant specific capabilities
    sandbox = sandbox_grant(sandbox, CAP_CONSOLE)
    sandbox = sandbox_grant(sandbox, CAP_TIME)

    # Load WASM plugin
    wasm_bytes := load_wasm_file("plugin.wasm")
    wasm_len := get_wasm_size(wasm_bytes)

    instance := sandbox_load(sandbox, wasm_bytes, wasm_len)

    I wasm_is_valid(instance) == 1 {
        # Call plugin function
        result := wasm_call1(instance, "process", 42)
    }

    # Cleanup
    sandbox_destroy(sandbox)
    0
}
```

### Hot Reload

```vais
U std/dynload

F main() -> i64 {
    # Create hot reloader for a source file
    reloader := hot_reloader_new("./src/plugin.vais")
    reloader = hot_reloader_start(reloader)

    # Main loop
    L 1 {
        # Check for changes
        changed := hot_reloader_check(reloader)

        I changed == 1 {
            version := hot_reloader_version(reloader)
            # Plugin was reloaded, update references
        }

        # Do work
        # ...
    }

    reloader = hot_reloader_stop(reloader)
    0
}
```

### Capability Checking

```vais
U std/dynload

F verify_plugin_safety(plugin_caps: i64) -> i64 {
    # Check if plugin has dangerous capabilities
    I has_dangerous_capabilities(plugin_caps) == 1 {
        # Warn user or reject plugin
        0
    } E {
        # Safe to load
        1
    }
}

F main() -> i64 {
    # Build capability set for a plugin
    caps := CAP_CONSOLE
    caps = add_capability(caps, CAP_TIME)
    caps = add_capability(caps, CAP_RANDOM)

    # Check specific capability
    I has_capability(caps, CAP_NETWORK) == 1 {
        # Has network access
    }

    verify_plugin_safety(caps)
    0
}
```
