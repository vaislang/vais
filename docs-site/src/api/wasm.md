# WASM API Reference

> WebAssembly runtime utilities and polyfills

## Import

```vais
U std/wasm
```

## Overview

The `wasm` module provides WebAssembly-specific memory management, I/O, and WASI bindings. It is designed for programs compiled to the `wasm32-unknown-unknown` target and includes polyfills for standard library functions that are unavailable in the WASM environment.

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `WASM_TARGET` | 1 | WASM environment flag |
| `WASM_PAGE_SIZE` | 65536 | WASM memory page size (64KB) |
| `WASM_STDIN` | 0 | Standard input fd (WASI) |
| `WASM_STDOUT` | 1 | Standard output fd (WASI) |
| `WASM_STDERR` | 2 | Standard error fd (WASI) |
| `WASI_NS` | 0 | wasi_snapshot_preview1 namespace |
| `ENV_NS` | 1 | env namespace (custom host functions) |

## Memory Management

### wasm_memory_size

```vais
F wasm_memory_size() -> i64
```

Get current memory size in pages (each page = 64KB).

### wasm_memory_grow

```vais
F wasm_memory_grow(pages: i64) -> i64
```

Grow memory by N pages. Returns previous size or `-1` on failure.

## I/O Functions

The module provides WASI-compatible I/O functions for writing to stdout/stderr within a WebAssembly environment.

## Example

```vais
U std/wasm

F main() {
    # Check memory size
    pages := wasm_memory_size()
    total_bytes := pages * WASM_PAGE_SIZE

    # Grow memory
    old_pages := wasm_memory_grow(1)
}
```
