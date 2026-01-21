# Vais Language Bindings

This document provides an overview of the Python and Node.js bindings for the Vais compiler.

## Overview

The Vais compiler now provides native bindings for both Python and Node.js, allowing developers to integrate Vais compilation capabilities into their Python and JavaScript/TypeScript projects.

## Crates

### vais-python (`crates/vais-python/`)

Python bindings using PyO3, providing a native Python module for compiling, checking, parsing, and tokenizing Vais source code.

**Technology:** PyO3 0.22+

**Key Features:**
- Native Python extension (compiled to `.so`/`.pyd`)
- Full access to Vais compiler pipeline
- Pythonic error handling
- Type-safe Python classes for errors and tokens

### vais-node (`crates/vais-node/`)

Node.js bindings using napi-rs, providing a native Node.js addon for compiling, checking, parsing, and tokenizing Vais source code.

**Technology:** napi-rs 2.16+

**Key Features:**
- Native Node.js addon (compiled to `.node`)
- Full access to Vais compiler pipeline
- JavaScript-friendly error handling
- TypeScript type definitions available

## API Comparison

| Feature | Python | Node.js |
|---------|--------|---------|
| Compile to LLVM IR | `compile(source, opt_level, module_name, target)` | `compile(source, options)` |
| Type Check | `check(source)` | `check(source)` |
| Parse to AST | `parse(source)` | `parse(source)` |
| Tokenize | `tokenize(source)` | `tokenize(source)` |

## Common Functionality

Both bindings expose the same core functionality:

1. **Tokenization** - Break source code into tokens
2. **Parsing** - Convert source code into an Abstract Syntax Tree (AST)
3. **Type Checking** - Validate type correctness
4. **Code Generation** - Compile to LLVM IR with optimization support

## Building

### Python Bindings

```bash
# Development build (requires Python interpreter)
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo build -p vais-python

# Check compilation without linking
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo check -p vais-python

# Release build
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo build --release -p vais-python
```

The compiled module will be named `vais.so` (Linux), `vais.dylib` (macOS), or `vais.pyd` (Windows).

### Node.js Bindings

```bash
# Development build
cargo build -p vais-node

# Release build
cargo build --release -p vais-node
```

The compiled addon will be named `libvais_node.node` (or platform-specific equivalent).

## Usage Examples

### Python Example

```python
import vais

# Compile Vais source to LLVM IR
source = """
F factorial(n:i64)->i64={
    I n<=1 {
        R 1
    }
    R n * factorial(n - 1)
}
"""

# Check for errors
errors = vais.check(source)
if errors:
    for err in errors:
        print(f"{err.error_type}: {err.message}")
else:
    # Compile with optimization
    ir = vais.compile(source, opt_level=2, module_name="factorial")
    print(ir)
```

### Node.js Example

```javascript
const vais = require('./target/release/vais_node.node');

// Compile Vais source to LLVM IR
const source = `
F factorial(n:i64)->i64={
    I n<=1 {
        R 1
    }
    R n * factorial(n - 1)
}
`;

// Check for errors
const errors = vais.check(source);
if (errors.length > 0) {
    errors.forEach(err => {
        console.log(`${err.errorType}: ${err.message}`);
    });
} else {
    // Compile with optimization
    const ir = vais.compile(source, {
        optLevel: 2,
        moduleName: "factorial"
    });
    console.log(ir);
}
```

## Implementation Details

### Shared Token Conversion

Both bindings share a common token conversion module (`token_conv.rs`) that maps Vais lexer tokens to language-specific representations:

- Python: Returns `TokenInfo` PyClass objects
- Node.js: Returns `VaisToken` NAPI objects

### AST Serialization

Currently, both bindings provide a simplified AST representation with:
- Module type identifier
- Item count
- Empty items array (placeholder for full AST serialization)

Full AST serialization can be added in the future by implementing comprehensive type mapping.

### Error Handling

Both bindings provide consistent error reporting:

**Python:**
- Uses `PyValueError` for lexer/parser errors
- Uses `PyRuntimeError` for codegen errors
- Returns list of `Error` objects for type checking

**Node.js:**
- Uses `Error` with `Status::InvalidArg` for lexer/parser errors
- Uses `Error` with `Status::GenericFailure` for codegen errors
- Returns array of `VaisError` objects for type checking

## Optimization Support

Both bindings support LLVM IR optimization levels:
- **O0** - No optimization (default)
- **O1** - Basic optimization
- **O2** - Standard optimization
- **O3** - Aggressive optimization

## Target Triple Support

Both bindings support cross-compilation targets:
- **Native** - Host platform (default)
- **wasm32-unknown-unknown** - WebAssembly (no OS)
- **wasm32-wasi** - WebAssembly System Interface
- Custom target triples via string parameter

## Testing

While full Python/Node.js runtime testing is deferred, both crates compile successfully:

```bash
# Verify Python bindings compile
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo check -p vais-python

# Verify Node.js bindings compile
cargo build -p vais-node
```

## Future Enhancements

1. **Full AST Serialization** - Complete AST to dict/object conversion
2. **Streaming Compilation** - Support for large files
3. **Source Maps** - Better error reporting with source context
4. **Async APIs** - Non-blocking compilation for Node.js
5. **Type Stubs** - Python .pyi files for better IDE support
6. **NPM Package** - Publish to npm with pre-built binaries
7. **PyPI Package** - Publish to PyPI with wheels for common platforms

## Dependencies

### Python (vais-python)
- pyo3 = 0.22
- All vais-* workspace crates

### Node.js (vais-node)
- napi = 2.16
- napi-derive = 2.16
- napi-build = 2.1 (build dependency)
- All vais-* workspace crates

## Workspace Integration

Both crates are integrated into the workspace:

```toml
# Cargo.toml
[workspace]
members = [
    # ... existing members ...
    "crates/vais-python",
    "crates/vais-node",
]
```

## License

Both bindings inherit the MIT license from the main Vais project.
