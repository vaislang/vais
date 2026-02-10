# WASI (WebAssembly System Interface)

WASI is a standardized system interface for WebAssembly, providing portable access to system calls like file I/O, environment variables, and command-line arguments.

## What is WASI?

WASI defines a modular system interface that allows WASM modules to:
- Read and write files
- Access environment variables and command-line arguments
- Get the current time
- Generate random numbers
- Interact with sockets (in WASI preview 2+)

Unlike browser-based WASM, WASI is designed for server-side and command-line applications.

## Supported WASI Functions in Vais

Vais provides WASI bindings through the `std/wasm.vais` module:

### File Descriptors
- `fd_read(fd, iov_ptr, iov_len, nread_ptr)` — Read from file descriptor
- `fd_write(fd, iov_ptr, iov_len, nwritten_ptr)` — Write to file descriptor
- `fd_close(fd)` — Close file descriptor
- `fd_seek(fd, offset, whence, newoffset_ptr)` — Seek in file

### Standard Streams
Constants for standard file descriptors:
- `WASM_STDIN = 0` — Standard input
- `WASM_STDOUT = 1` — Standard output
- `WASM_STDERR = 2` — Standard error

### Environment
- `args_get(argv_ptr, argv_buf_ptr)` — Get command-line arguments
- `args_sizes_get(argc_ptr, argv_buf_size_ptr)` — Get argument count and buffer size
- `environ_get(env_ptr, env_buf_ptr)` — Get environment variables
- `environ_sizes_get(env_count_ptr, env_buf_size_ptr)` — Get environment sizes

### System
- `clock_time_get(clock_id, precision, time_ptr)` — Get current time
- `random_get(buf_ptr, buf_len)` — Generate random bytes
- `proc_exit(exit_code)` — Exit process

## Example: Hello World with WASI

```vais
U std/wasm

F main() -> i64 {
    # Write to stdout using WASI
    msg := "Hello from WASI!\n"
    ptr := str_to_ptr(msg)
    len := strlen(msg)

    # Create iovec structure (pointer + length)
    iov := wasm_heap_alloc(16)  # 2 x i64 = 16 bytes
    store_i64(iov, ptr)
    store_i64(iov + 8, len)

    # Write to stdout
    nwritten_ptr := wasm_heap_alloc(8)
    result := fd_write(WASM_STDOUT, iov, 1, nwritten_ptr)

    I result == 0 {
        R 0  # Success
    } E {
        R 1  # Failure
    }
}

# WASI entry point
F _start() {
    exit_code := main()
    proc_exit(exit_code)
}
```

## Compiling for WASI

Compile with the wasm32-wasi target:
```bash
vaisc --target wasm32-wasi hello.vais -o hello.wasm
```

## Running WASI Applications

### Wasmtime

[Wasmtime](https://wasmtime.dev/) is the reference WASI runtime:

```bash
# Install wasmtime
curl https://wasmtime.dev/install.sh -sSf | bash

# Run the WASM module
wasmtime hello.wasm

# With arguments
wasmtime hello.wasm -- arg1 arg2
```

### Wasmer

[Wasmer](https://wasmer.io/) is another popular WASI runtime:

```bash
# Install wasmer
curl https://get.wasmer.io -sSfL | sh

# Run the module
wasmer run hello.wasm
```

### Node.js WASI

Node.js has built-in WASI support:

```javascript
const { WASI } = require('wasi');
const fs = require('fs');

const wasi = new WASI({
    args: process.argv,
    env: process.env,
    preopens: {
        '/sandbox': '/tmp'  // Map /tmp to /sandbox in WASM
    }
});

const wasmBuffer = fs.readFileSync('./hello.wasm');

(async () => {
    const { instance } = await WebAssembly.instantiate(wasmBuffer, {
        wasi_snapshot_preview1: wasi.wasiImport
    });

    wasi.start(instance);
})();
```

## File I/O Example

Reading a file with WASI:

```vais
U std/wasm

F read_file(path: str) -> str {
    # Open file (path_open syscall - simplified)
    fd := wasm_open_file(path)  # Helper function
    I fd < 0 {
        R "Error opening file"
    }

    # Allocate read buffer
    buf_size := 4096
    buf := wasm_heap_alloc(buf_size)

    # Create iovec for fd_read
    iov := wasm_heap_alloc(16)
    store_i64(iov, buf)
    store_i64(iov + 8, buf_size)

    # Read from file
    nread_ptr := wasm_heap_alloc(8)
    result := fd_read(fd, iov, 1, nread_ptr)

    I result == 0 {
        nread := load_i64(nread_ptr)
        content := ptr_to_str(buf, nread)
        fd_close(fd)
        R content
    } E {
        fd_close(fd)
        R "Error reading file"
    }
}
```

## WASI Modules

WASI is organized into capability-based modules:

### wasi_snapshot_preview1 (Current)
The current stable WASI version includes:
- File I/O
- Environment access
- Clock and random number generation
- Process control

### wasi_snapshot_preview2 (Future)
Preview 2 adds:
- Sockets and networking
- Async I/O
- Component Model integration
- Enhanced security capabilities

Vais will support preview2 when it becomes stable.

## Security Considerations

WASI follows a capability-based security model:

1. **No Ambient Authority** — WASM modules can't access files unless explicitly granted
2. **Preopened Directories** — Runtimes specify which directories are accessible
3. **No Network by Default** — Networking requires explicit capability grants

Example with restricted file access:
```bash
# Grant read-only access to /data directory
wasmtime --dir=/data::ro hello.wasm

# Grant read-write access to /output
wasmtime --dir=/output hello.wasm
```

## WASI vs Browser WASM

| Feature | WASI | Browser WASM |
|---------|------|--------------|
| File I/O | Yes (with capabilities) | No (use Fetch API) |
| Environment Vars | Yes | No |
| Networking | Yes (preview2) | Fetch API only |
| Entry Point | `_start()` | Custom exports |
| Security Model | Capabilities | Same-origin policy |
| Use Case | CLI tools, servers | Web apps |

## Combining WASI with Custom Imports

You can mix WASI syscalls with custom JavaScript imports:

```vais
# WASI file I/O
U std/wasm

# Custom browser API
#[wasm_import("env", "alert")]
N F js_alert(ptr: i64, len: i64)

F main() -> i64 {
    # Use WASI to read config
    config := read_file("config.txt")

    # Use custom import to show alert
    js_alert(str_to_ptr(config), strlen(config))

    R 0
}
```

## See Also

- [Getting Started](./getting-started.md) — Basic WASM compilation
- [Component Model](./component-model.md) — Advanced WASM interfaces
- [std/wasm.vais source](https://github.com/vaislang/vais/blob/main/std/wasm.vais)
