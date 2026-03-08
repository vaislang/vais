# WASI Preview 2 API Reference

> WASI Preview 2 interface bindings (Component Model)

> **Implementation:** WASM-only module. Requires `--target wasm32-wasi` compilation. Functions are extern declarations bound to WASI Preview 2 host APIs via the Component Model.

## Import

```vais
U std/wasi_p2
```

## Overview

The `wasi_p2` module provides Vais bindings for [WASI Preview 2](https://github.com/WebAssembly/WASI/tree/main/preview2), which uses the Component Model's canonical ABI with typed resources and streams instead of the linear-memory-based Preview 1 syscalls.

## Type Aliases

```vais
T StreamError = i64      # Stream error (0 = success)
T InputStream = i64      # Input stream handle
T OutputStream = i64     # Output stream handle
```

## Interfaces

### wasi:io/streams@0.2.0

Stream-based I/O primitives.

```vais
F wasi_io_stream_read(stream: i64, buf_ptr: i64, buf_len: i64) -> i64
F wasi_io_stream_write(stream: i64, buf_ptr: i64, buf_len: i64) -> i64
```

### wasi:filesystem/types@0.2.0

File system operations with typed descriptors.

### wasi:cli/stdin@0.2.0 / stdout / stderr

Standard I/O stream accessors.

### wasi:clocks/monotonic-clock@0.2.0

High-resolution monotonic clock.

### wasi:random/random@0.2.0

Cryptographically-secure random number generation.

## Example

```vais
U std/wasi_p2

F main() {
    # Read from stdin
    buf := malloc(1024)
    bytes_read := wasi_io_stream_read(0, buf, 1024)
}
```

## References

- [WASI Preview 2 Spec](https://github.com/WebAssembly/WASI/tree/main/preview2)
- [Component Model](https://github.com/WebAssembly/component-model)
