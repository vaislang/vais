# DynLoad API Reference

> Dynamic module loading and WASM sandboxing

## Import

```vais
U std/dynload
```

## Features

- Load/unload shared libraries at runtime
- Symbol lookup (dlsym)
- WASM plugin sandboxing with resource limits
- Automatic plugin discovery

## Key Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `module_load` | `F module_load(path: i64) -> i64` | Load dynamic library |
| `module_unload` | `F module_unload(handle: i64) -> i64` | Unload library |
| `module_symbol` | `F module_symbol(handle: i64, name: i64) -> i64` | Get symbol address |
| `module_call` | `F module_call(handle: i64, fn_name: i64, arg: i64) -> i64` | Call function by name |

## Low-Level (C FFI)

| Function | Description |
|----------|-------------|
| `dlopen(path, flags)` | Open shared library |
| `dlclose(handle)` | Close library |
| `dlsym(handle, symbol)` | Get symbol |
| `dlerror()` | Get error string |

## Usage

```vais
U std/dynload

F main() -> i64 {
    lib := module_load("./plugin.so")
    fn_ptr := module_symbol(lib, "plugin_init")
    # Call the function
    module_unload(lib)
    0
}
```
