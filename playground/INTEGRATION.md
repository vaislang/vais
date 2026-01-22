# WASM Integration Guide

This guide explains how to integrate the actual Vais compiler (vaisc) with the playground using WebAssembly.

## Overview

The playground currently uses a mock compiler for demonstration. To enable real compilation:

1. Compile `vaisc` to WASM target
2. Create JavaScript bindings
3. Load and initialize the WASM module
4. Update the compiler interface

## Step 1: Compile vaisc to WASM

### Prerequisites

```bash
# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen CLI
cargo install wasm-bindgen-cli
```

### Build Configuration

Create a new crate for WASM bindings:

```bash
cd ../crates
cargo new vais-wasm --lib
```

Edit `crates/vais-wasm/Cargo.toml`:

```toml
[package]
name = "vais-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
vais-lexer = { path = "../vais-lexer" }
vais-parser = { path = "../vais-parser" }
vais-types = { path = "../vais-types" }
vais-codegen = { path = "../vais-codegen" }
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
```

### WASM Bindings

Create `crates/vais-wasm/src/lib.rs`:

```rust
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct CompileResult {
    success: bool,
    ir: Option<String>,
    errors: Vec<CompileError>,
    warnings: Vec<CompileWarning>,
}

#[derive(Serialize, Deserialize)]
pub struct CompileError {
    line: usize,
    column: usize,
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct CompileWarning {
    line: usize,
    column: usize,
    message: String,
}

#[wasm_bindgen]
pub fn compile_vais(source: &str) -> JsValue {
    let result = compile_vais_internal(source);
    serde_wasm_bindgen::to_value(&result).unwrap()
}

fn compile_vais_internal(source: &str) -> CompileResult {
    // Tokenize
    let tokens = match vais_lexer::tokenize(source) {
        Ok(t) => t,
        Err(e) => {
            return CompileResult {
                success: false,
                ir: None,
                errors: vec![CompileError {
                    line: 0,
                    column: 0,
                    message: format!("Lexer error: {}", e),
                }],
                warnings: vec![],
            };
        }
    };

    // Parse
    let ast = match vais_parser::parse(source) {
        Ok(a) => a,
        Err(e) => {
            return CompileResult {
                success: false,
                ir: None,
                errors: vec![CompileError {
                    line: 0,
                    column: 0,
                    message: format!("Parse error: {}", e),
                }],
                warnings: vec![],
            };
        }
    };

    // Type check
    let mut checker = vais_types::TypeChecker::new();
    if let Err(e) = checker.check_module(&ast) {
        return CompileResult {
            success: false,
            ir: None,
            errors: vec![CompileError {
                line: 0,
                column: 0,
                message: format!("Type error: {}", e),
            }],
            warnings: vec![],
        };
    }

    // Generate IR
    let mut codegen = vais_codegen::CodeGenerator::new("playground");
    let ir = match codegen.generate_module(&ast) {
        Ok(ir) => ir,
        Err(e) => {
            return CompileResult {
                success: false,
                ir: None,
                errors: vec![CompileError {
                    line: 0,
                    column: 0,
                    message: format!("Codegen error: {}", e),
                }],
                warnings: vec![],
            };
        }
    };

    CompileResult {
        success: true,
        ir: Some(ir),
        errors: vec![],
        warnings: vec![],
    }
}

#[wasm_bindgen(start)]
pub fn init() {
    // Initialize panic hook for better error messages
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
```

### Build WASM Module

```bash
cd crates/vais-wasm
wasm-pack build --target web --out-dir ../../playground/public/wasm
```

## Step 2: Update JavaScript Compiler Interface

Modify `src/compiler.js`:

```javascript
import init, { compile_vais } from '../public/wasm/vais_wasm.js';

export class VaisCompiler {
  constructor() {
    this.isReady = false;
    this.wasmModule = null;
  }

  async initialize() {
    try {
      // Initialize WASM module
      this.wasmModule = await init();
      this.isReady = true;
      return true;
    } catch (error) {
      console.error('Failed to initialize WASM:', error);
      throw new Error('WASM initialization failed: ' + error.message);
    }
  }

  async compile(sourceCode) {
    if (!this.isReady) {
      await this.initialize();
    }

    try {
      // Call WASM compile function
      const result = compile_vais(sourceCode);
      return result;
    } catch (error) {
      return {
        success: false,
        errors: [{
          line: 0,
          column: 0,
          message: `Compilation error: ${error.message}`
        }],
        warnings: [],
        ir: null
      };
    }
  }

  // ... rest of the methods
}
```

## Step 3: Execution Engine

For executing compiled code, you have two options:

### Option A: Compile to WASM and Execute

1. Modify the compiler to generate WASM directly (instead of LLVM IR)
2. Load the generated WASM module
3. Execute the exported functions

### Option B: LLVM IR Interpreter

1. Use an LLVM IR interpreter written in JavaScript/WASM
2. Feed the generated IR to the interpreter
3. Capture stdout/stderr

### Option C: Server-Side Execution

1. Send the code to a backend server
2. Compile and execute on the server
3. Return the output to the frontend

Example server setup:

```javascript
// In compiler.js
async executeOnServer(sourceCode) {
  try {
    const response = await fetch('/api/compile', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ code: sourceCode })
    });

    const result = await response.json();
    return result;
  } catch (error) {
    return {
      success: false,
      output: null,
      error: error.message
    };
  }
}
```

## Step 4: Testing

```bash
# Test WASM build
cd crates/vais-wasm
wasm-pack test --node

# Test in browser
cd ../../playground
npm run dev
```

## Performance Considerations

1. **Lazy Loading**: Load WASM module only when needed
2. **Worker Threads**: Run compilation in a Web Worker
3. **Caching**: Cache compiled modules
4. **Streaming**: Use streaming compilation for large files

### Worker Implementation

Create `src/compiler.worker.js`:

```javascript
import init, { compile_vais } from '../public/wasm/vais_wasm.js';

let initialized = false;

self.onmessage = async function(e) {
  const { id, type, data } = e.data;

  if (type === 'init') {
    try {
      await init();
      initialized = true;
      self.postMessage({ id, type: 'init', success: true });
    } catch (error) {
      self.postMessage({ id, type: 'init', success: false, error: error.message });
    }
  }

  if (type === 'compile') {
    if (!initialized) {
      self.postMessage({ id, type: 'compile', success: false, error: 'Not initialized' });
      return;
    }

    try {
      const result = compile_vais(data.source);
      self.postMessage({ id, type: 'compile', result });
    } catch (error) {
      self.postMessage({ id, type: 'compile', success: false, error: error.message });
    }
  }
};
```

Update `src/compiler.js`:

```javascript
export class VaisCompiler {
  constructor() {
    this.worker = new Worker(new URL('./compiler.worker.js', import.meta.url), {
      type: 'module'
    });
    this.requestId = 0;
    this.pending = new Map();

    this.worker.onmessage = (e) => {
      const { id, type, result, error } = e.data;
      const resolve = this.pending.get(id);
      if (resolve) {
        if (error) {
          resolve({ success: false, error });
        } else {
          resolve(result);
        }
        this.pending.delete(id);
      }
    };
  }

  async initialize() {
    return new Promise((resolve) => {
      const id = this.requestId++;
      this.pending.set(id, resolve);
      this.worker.postMessage({ id, type: 'init' });
    });
  }

  async compile(sourceCode) {
    return new Promise((resolve) => {
      const id = this.requestId++;
      this.pending.set(id, resolve);
      this.worker.postMessage({
        id,
        type: 'compile',
        data: { source: sourceCode }
      });
    });
  }
}
```

## Debugging

### Enable Debug Output

```rust
// In lib.rs
#[wasm_bindgen]
pub fn set_debug_mode(enabled: bool) {
    // Enable verbose logging
}
```

### Console Logging

```rust
use web_sys::console;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Usage
log(&format!("Compiling: {} lines", source.lines().count()));
```

## Deployment

### Build for Production

```bash
# Build WASM with optimizations
cd crates/vais-wasm
wasm-pack build --target web --release --out-dir ../../playground/public/wasm

# Optimize WASM binary
wasm-opt -Oz -o vais_wasm_bg.wasm.opt vais_wasm_bg.wasm

# Build playground
cd ../../playground
npm run build
```

### Size Optimization

Add to `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### CDN Hosting

Upload WASM files to a CDN for faster loading:

```javascript
// In compiler.js
const WASM_URL = 'https://cdn.example.com/vais_wasm_bg.wasm';

async initialize() {
  const response = await fetch(WASM_URL);
  const bytes = await response.arrayBuffer();
  this.wasmModule = await init(bytes);
}
```

## Troubleshooting

### Common Issues

1. **WASM not loading**: Check MIME type is `application/wasm`
2. **Import errors**: Ensure wasm-bindgen versions match
3. **Memory issues**: Increase WASM memory limit
4. **CORS errors**: Configure server headers correctly

### Browser Console

```javascript
// Check WASM support
console.log('WASM support:', typeof WebAssembly !== 'undefined');

// Log module size
fetch('/wasm/vais_wasm_bg.wasm')
  .then(r => r.arrayBuffer())
  .then(b => console.log('WASM size:', (b.byteLength / 1024).toFixed(2), 'KB'));
```

## Resources

- [wasm-bindgen documentation](https://rustwasm.github.io/wasm-bindgen/)
- [wasm-pack guide](https://rustwasm.github.io/wasm-pack/)
- [MDN WebAssembly](https://developer.mozilla.org/en-US/Web/JavaScript/Reference/Global_Objects/WebAssembly)
