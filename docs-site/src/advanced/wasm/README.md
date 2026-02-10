# WASM Components

Vais provides first-class support for WebAssembly (WASM) compilation, enabling your code to run in browsers, Node.js, and WASI runtimes.

## Overview

WebAssembly is a portable binary instruction format designed for safe, fast execution in web browsers and beyond. Vais can compile to WASM targets, allowing you to:

- **Run in Browsers** — Build web applications with near-native performance
- **Server-Side WASM** — Use WASI for command-line tools and microservices
- **Edge Computing** — Deploy to Cloudflare Workers, Fastly Compute, and other edge platforms
- **Embedded Systems** — Lightweight, sandboxed execution environments

## Supported Targets

Vais supports multiple WASM compilation targets:

| Target | Description | Use Case |
|--------|-------------|----------|
| `wasm32-unknown-unknown` | Generic WASM for browsers | Web applications, JavaScript interop |
| `wasm32-wasi` | WASM with WASI syscalls | CLI tools, server-side apps |
| `wasm32-unknown-emscripten` | Emscripten-compatible | Legacy browser support |

## Key Features

### 1. Bidirectional JavaScript Interop

Import JavaScript functions into Vais:
```vais
#[wasm_import("env", "console_log")]
N F console_log(ptr: i64, len: i64)
```

Export Vais functions to JavaScript:
```vais
#[wasm_export("add")]
F add(a: i64, b: i64) -> i64 {
    a + b
}
```

### 2. Standard Web APIs

Pre-built bindings in `std/web.vais`:
- Console, DOM manipulation, Timers
- Fetch API, LocalStorage, Canvas
- WebSocket, Geolocation, History API

### 3. WASI System Interface

Access files, environment, and syscalls via `std/wasm.vais`:
- File I/O (read, write, seek)
- Command-line arguments and environment variables
- Clock, random number generation

### 4. Component Model

Support for WASM Component Model and WIT (WebAssembly Interface Types):
- Language-agnostic interfaces
- Composable components
- Advanced type system

### 5. TypeScript Declarations

Automatic `.d.ts` generation with `vais-bindgen`:
```bash
vais-bindgen --wasm-js input.vais -o bindings.js
```

## Quick Start

### Compile to WASM
```bash
vaisc --target wasm32-unknown-unknown hello.vais -o hello.wasm
```

### Load in JavaScript
```javascript
const response = await fetch('hello.wasm');
const { instance } = await WebAssembly.instantiate(
    await response.arrayBuffer(),
    { env: { /* imports */ } }
);

instance.exports.hello();
```

### Use in Node.js with WASI
```bash
wasmtime hello.wasm
```

## Documentation Sections

### [Getting Started](./getting-started.md)
Step-by-step guide to compiling and running your first WASM program:
- Setting up LLVM and wasm-ld
- Hello World example
- Browser and Node.js deployment
- Debugging techniques

### [Component Model](./component-model.md)
Learn about WASM Component Model and WIT:
- Interface definition language (WIT)
- Type mapping between Vais and WIT
- Component composition and linking
- Future roadmap

### [JavaScript Interop](./js-interop.md)
Advanced JavaScript integration:
- Importing and exporting functions
- Memory management patterns
- Complex type serialization
- Error handling across boundaries

### [WASI](./wasi.md)
WASM System Interface for non-browser environments:
- WASI syscall reference
- File I/O and filesystem access
- Environment variables and arguments
- Running with Wasmtime, Wasmer, Node.js

## Performance Characteristics

WASM compiled from Vais achieves:
- **Near-native speed** — Typically 70-90% of native performance
- **Small binary size** — With LTO and optimization, often <100KB for typical apps
- **Fast startup** — Instant module instantiation
- **No garbage collection pauses** — Deterministic memory management

## Security Model

WASM execution is sandboxed:
- **Memory isolation** — Linear memory is separate from host
- **No direct I/O** — All system access via explicit imports
- **Capability-based** — WASI grants explicit filesystem/network access
- **Type-safe** — Strong typing prevents common vulnerabilities

## Ecosystem Integration

### Web Frameworks
- Use Vais with React, Vue, Svelte via WASM components
- Framework-agnostic — works with any JavaScript framework

### Edge Platforms
- Cloudflare Workers
- Fastly Compute@Edge
- Deno Deploy
- AWS Lambda (WASM runtime)

### Tooling
- Vais Playground — Try WASM in your browser
- wasm-bindgen — JavaScript glue code generator
- wasmtime — Reference WASI runtime
- Chrome DevTools — Built-in WASM debugging

## Comparison with Other Languages

| Language | Binary Size | Performance | Browser Support | WASI Support |
|----------|-------------|-------------|-----------------|--------------|
| Vais | Small (100KB) | Fast (85%) | Yes | Yes |
| Rust | Medium (200KB) | Fast (90%) | Yes | Yes |
| AssemblyScript | Small (50KB) | Good (75%) | Yes | Limited |
| C/C++ | Medium (150KB) | Fast (90%) | Yes (Emscripten) | Yes |
| Go | Large (2MB) | Good (70%) | Yes | Limited |

*Performance percentages are relative to native execution*

## Limitations

Current WASM limitations in Vais:
- **No threads** — WASM threads are experimental (WASI preview2 will add support)
- **32-bit pointers** — Memory limited to 4GB per module
- **No dynamic linking** — Each module is self-contained
- **Limited SIMD** — WASM SIMD proposal not yet fully supported

These are WASM platform limitations, not specific to Vais.

## Examples

### Web Application
```vais
U std/web

#[wasm_export("init")]
F init() {
    log_str("App initialized")
    elem := get_element_by_id("root")
    set_text_content(elem, "Hello from Vais!")
}
```

### CLI Tool with WASI
```vais
U std/wasm

F main() -> i64 {
    # Read from stdin
    input := wasi_read_stdin(1024)

    # Process and write to stdout
    output := process(input)
    wasi_write_stdout(output)

    R 0
}
```

### Computational Kernel
```vais
#[wasm_export("fib")]
F fib(n: i64) -> i64 {
    I n <= 1 { R n }
    R @(n - 1) + @(n - 2)
}
```

## Next Steps

1. **Read the [Getting Started Guide](./getting-started.md)** to compile your first WASM module
2. **Explore [std/web.vais](https://github.com/vaislang/vais/blob/main/std/web.vais)** for browser API bindings
3. **Try the [Vais Playground](https://play.vaislang.org)** to experiment online
4. **Check [Examples](https://github.com/vaislang/vais/tree/main/examples)** for real-world WASM programs

## Resources

- [WebAssembly.org](https://webassembly.org/) — Official WASM specification
- [WASI](https://wasi.dev/) — WebAssembly System Interface
- [WIT](https://github.com/WebAssembly/component-model) — Component Model specification
- [MDN WASM Guide](https://developer.mozilla.org/en-US/docs/WebAssembly) — Browser integration
