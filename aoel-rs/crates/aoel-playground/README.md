# AOEL Playground

Browser-based AOEL code editor and runner using WebAssembly.

## Features

- Interactive code editor with syntax highlighting
- Real-time code execution in the browser
- Code formatting
- Example code snippets
- Execution time measurement
- Error highlighting

## Building

### Prerequisites

1. Install Rust toolchain: https://rustup.rs/
2. Install wasm-pack:
   ```bash
   cargo install wasm-pack
   ```

### Build WASM

```bash
# Using the build script
./build.sh

# Or manually
wasm-pack build --target web --out-dir www/pkg
```

### Run Locally

```bash
cd www
python3 -m http.server 8080
```

Then open http://localhost:8080 in your browser.

## Project Structure

```
aoel-playground/
├── src/
│   ├── lib.rs      # WASM bindings
│   └── wasm_vm.rs  # Lightweight VM for WASM
├── www/
│   ├── index.html  # Web interface
│   └── pkg/        # Generated WASM files
├── build.sh        # Build script
└── Cargo.toml
```

## API

The following functions are exported to JavaScript:

- `execute(source: string) -> string` - Execute AOEL code, returns JSON result
- `check(source: string) -> string` - Parse and type-check code
- `format_code(source: string) -> string` - Format AOEL code
- `get_ast(source: string) -> string` - Get AST representation
- `get_tokens(source: string) -> string` - Get token list

### Example Usage

```javascript
import init, { execute, format_code } from './pkg/aoel_playground.js';

await init();

const result = JSON.parse(execute(`
    factorial(n) = n < 2 ? 1 : n * $(n - 1)
    print(factorial(5))
`));

if (result.success) {
    console.log(result.output);          // "120"
    console.log(result.execution_time_ms);
} else {
    console.error(result.error);
}
```

## Limitations

The WASM VM has some limitations compared to the full VM:

- No file I/O (std.io functions)
- No network access (std.net functions)
- Limited recursion depth (500)
- No FFI support

These limitations are intentional for security in the browser environment.
