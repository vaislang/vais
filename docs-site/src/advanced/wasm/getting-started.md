# Getting Started with WASM

This guide will walk you through compiling Vais to WebAssembly and running it in a browser or Node.js environment.

## Prerequisites

To compile Vais to WASM, you need:

- **Vais compiler** (`vaisc`)
- **LLVM 17** with wasm32 target support
- **wasm-ld** linker (included with LLVM)

### Installing LLVM with WASM Support

On macOS (Homebrew):
```bash
brew install llvm@17
export PATH="/opt/homebrew/opt/llvm@17/bin:$PATH"
```

On Linux (APT):
```bash
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 17
```

Verify wasm-ld is available:
```bash
wasm-ld --version
```

## Quick Start: Hello World

### Step 1: Write a Simple Vais Program

Create `hello.vais`:
```vais
# Import console API from JavaScript host
#[wasm_import("env", "console_log")]
N F console_log(ptr: i64, len: i64)

# Export a function for JavaScript to call
#[wasm_export("hello")]
F hello() {
    msg := "Hello from Vais WASM!"
    console_log(str_to_ptr(msg), strlen(msg))
}

# WASM entry point
F _start() {
    hello()
}
```

### Step 2: Compile to WASM

```bash
vaisc --target wasm32-unknown-unknown hello.vais -o hello.wasm
```

This generates `hello.wasm` in the current directory.

### Step 3: Load from JavaScript

Create `index.html`:
```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Vais WASM Example</title>
</head>
<body>
    <h1>Vais WASM</h1>
    <div id="output"></div>

    <script>
        // Helper to read strings from WASM memory
        function readString(ptr, len, memory) {
            const bytes = new Uint8Array(memory.buffer, ptr, len);
            return new TextDecoder().decode(bytes);
        }

        // Load and instantiate WASM module
        async function loadWasm() {
            const response = await fetch('hello.wasm');
            const buffer = await response.arrayBuffer();

            const { instance } = await WebAssembly.instantiate(buffer, {
                env: {
                    console_log: (ptr, len) => {
                        const msg = readString(ptr, len, instance.exports.memory);
                        console.log(msg);
                        document.getElementById('output').textContent = msg;
                    }
                }
            });

            // Call the exported function
            instance.exports.hello();
        }

        loadWasm().catch(console.error);
    </script>
</body>
</html>
```

### Step 4: Serve and Test

Start a local HTTP server (required for WASM):
```bash
python3 -m http.server 8000
```

Open `http://localhost:8000` in your browser. You should see "Hello from Vais WASM!" in both the console and on the page.

## Using Standard Web APIs

The `std/web.vais` module provides pre-built bindings for common browser APIs:

```vais
U std/web

#[wasm_export("init")]
F init() {
    # Console logging
    log_str("Application initialized")

    # DOM manipulation
    elem := get_element_by_id("app")
    set_text_content(elem, "Welcome to Vais!")

    # Timer
    timer := set_timeout(0, 1000)  # 1 second delay
}
```

Available API modules:
- **Console** — `log_str`, `warn_str`, `error_str`
- **DOM** — `get_element_by_id`, `set_text_content`, `set_inner_html`
- **Timers** — `set_timeout`, `set_interval`, `clear_timeout`
- **Fetch** — HTTP requests
- **Storage** — LocalStorage/SessionStorage
- **Canvas** — 2D graphics

See [std/web.vais source](https://github.com/vaislang/vais/blob/main/std/web.vais) for the complete API.

## Debugging WASM

### Viewing WASM Text Format

Convert binary WASM to readable text format:
```bash
wasm2wat hello.wasm -o hello.wat
```

### Inspecting with Browser DevTools

Modern browsers have built-in WASM debugging:
1. Open DevTools (F12)
2. Go to "Sources" or "Debugger" tab
3. Find your WASM module in the file tree
4. Set breakpoints and inspect memory

### Adding Debug Symbols

Compile with debug info:
```bash
vaisc --target wasm32-unknown-unknown --debug hello.vais -o hello.wasm
```

This includes function names and source maps in the WASM binary.

## Using the Vais Playground

The easiest way to experiment with WASM is the [Vais Playground](https://play.vaislang.org):

1. Write your Vais code
2. Click "Compile" and select "WASM" target
3. The playground automatically loads and runs the WASM module
4. View output in the console pane

The playground handles all the JavaScript glue code automatically.

## Node.js WASM Execution

You can also run WASM in Node.js:

```javascript
const fs = require('fs');

async function runWasm() {
    const wasmBuffer = fs.readFileSync('./hello.wasm');

    const { instance } = await WebAssembly.instantiate(wasmBuffer, {
        env: {
            console_log: (ptr, len) => {
                // Read string from WASM memory
                const view = new Uint8Array(instance.exports.memory.buffer);
                const bytes = view.slice(ptr, ptr + len);
                const msg = Buffer.from(bytes).toString('utf8');
                console.log(msg);
            }
        }
    });

    instance.exports.hello();
}

runWasm();
```

Run with:
```bash
node run.js
```

## Next Steps

- [Component Model](./component-model.md) — Learn about WASM Component Model and WIT types
- [JS Interop](./js-interop.md) — Advanced JavaScript integration patterns
- [WASI](./wasi.md) — Use WASI for system interfaces and file I/O
