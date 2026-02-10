# Component Model

The WASM Component Model is a proposal for composable, language-agnostic WebAssembly components. It extends core WASM with high-level types and interfaces.

## What is the Component Model?

The Component Model adds:
- **WIT (WebAssembly Interface Types)** — A language for defining interfaces
- **High-level types** — Strings, lists, records, variants beyond core WASM
- **Composability** — Link components written in different languages
- **Virtualization** — Sandbox components with fine-grained capabilities

This enables true polyglot composition where Vais, Rust, Go, and C++ components can interoperate seamlessly.

## WIT (WebAssembly Interface Types)

WIT is an IDL (Interface Definition Language) for WASM components. It looks like this:

```wit
// calculator.wit
interface calculator {
    add: func(a: s32, b: s32) -> s32
    divide: func(a: f64, b: f64) -> result<f64, string>
}

world math-service {
    export calculator
}
```

This defines:
- An `interface` with two functions
- A `world` (entry point) that exports the interface

## Vais Type Mapping to WIT

Vais types map to WIT types as follows:

| Vais Type | WIT Type | Notes |
|-----------|----------|-------|
| `i8`, `i16`, `i32`, `i64` | `s8`, `s16`, `s32`, `s64` | Signed integers |
| `u8`, `u16`, `u32`, `u64` | `u8`, `u16`, `u32`, `u64` | Unsigned integers |
| `f32`, `f64` | `f32`, `f64` | Floating point |
| `bool` | `bool` | Boolean |
| `str` | `string` | UTF-8 string |
| `Vec<T>` | `list<T>` | Dynamic array |
| `Option<T>` | `option<T>` | Optional value |
| `Result<T, E>` | `result<T, E>` | Error handling |
| Struct | `record` | Named fields |
| Enum | `variant` | Tagged union |
| Tuple | `tuple<...>` | Fixed-size sequence |

### Example: Struct Mapping

Vais struct:
```vais
S Person {
    name: str,
    age: i32,
    active: bool
}
```

WIT record:
```wit
record person {
    name: string,
    age: s32,
    active: bool,
}
```

### Example: Enum Mapping

Vais enum:
```vais
E Status {
    Pending,
    Running(i64),
    Complete(str)
}
```

WIT variant:
```wit
variant status {
    pending,
    running(s64),
    complete(string),
}
```

## Defining Component Interfaces

You can define WIT interfaces for Vais components using attributes:

```vais
# Annotate with WIT interface
#[wit_interface("calculator")]
S Calculator {}

#[wit_export("add")]
F add(a: i32, b: i32) -> i32 {
    a + b
}

#[wit_export("divide")]
F divide(a: f64, b: f64) -> Result<f64, str> {
    I b == 0.0 {
        R Err("Division by zero")
    }
    R Ok(a / b)
}
```

The Vais compiler generates corresponding WIT:
```wit
interface calculator {
    add: func(a: s32, b: s32) -> s32
    divide: func(a: f64, b: f64) -> result<f64, string>
}
```

## Component Linking

Components can import and export interfaces:

### Producer Component (Vais)
```vais
# Export a logger interface
#[wit_export("log")]
F log_message(level: str, msg: str) {
    # Implementation
}
```

### Consumer Component (Any Language)
```wit
// consumer.wit
import logger: interface {
    log: func(level: string, msg: string)
}

export consumer: interface {
    run: func()
}
```

The consumer can call Vais's `log_message` through the WIT interface.

## Composing Components

WASM components can be linked together:

```bash
# Compile Vais to component
vaisc --target wasm32-component logger.vais -o logger.wasm

# Compile Rust to component
cargo component build

# Link components
wasm-tools compose logger.wasm consumer.wasm -o app.wasm
```

The composed `app.wasm` contains both components, with type-safe calls across the boundary.

## Resource Types

WIT supports "resources" (opaque handles):

```wit
resource database {
    constructor(url: string)
    query: func(sql: string) -> list<record { ... }>
    close: func()
}
```

In Vais:
```vais
#[wit_resource("database")]
S Database {
    conn: i64  # Opaque handle
}

#[wit_constructor]
F Database::new(url: str) -> Database {
    conn := internal_connect(url)
    Database { conn }
}

#[wit_method]
F Database::query(self, sql: str) -> Vec<Row> {
    # Implementation
}

#[wit_destructor]
F Database::close(self) {
    internal_close(self.conn)
}
```

Resources ensure proper lifetime management across component boundaries.

## Canonical ABI

The Component Model defines a Canonical ABI for lowering/lifting types:

### Lifting (WASM → Host)
Convert WASM linear memory representation to host language types:
```
WASM i32 + i32 (ptr, len) → Host String
```

### Lowering (Host → WASM)
Convert host types to WASM linear memory:
```
Host String → WASM i32 + i32 (ptr, len)
```

Vais handles this automatically through `WasmSerde`:

```vais
# Automatic serialization for component exports
#[wasm_export("process")]
F process(data: Person) -> Result<str, str> {
    # `data` is automatically lifted from linear memory
    # Return value is automatically lowered
    R Ok("Processed: ~{data.name}")
}
```

## Generating WIT from Vais

Use `vais-bindgen` to extract WIT interfaces:

```bash
vais-bindgen --wit input.vais -o output.wit
```

This generates WIT definitions for all `#[wit_export]` annotated items.

## Current Status & Roadmap

### Supported (v0.1)
- ✅ Basic type mapping (primitives, strings, lists)
- ✅ `#[wasm_export]` for function exports
- ✅ Manual WIT generation
- ✅ Compatible with `wasm-tools`

### In Progress (v0.2)
- 🚧 `#[wit_interface]` attribute
- 🚧 Automatic WIT generation from Vais types
- 🚧 Resource types
- 🚧 Component imports

### Planned (v0.3+)
- 📋 Full Canonical ABI support
- 📋 Futures/streams for async
- 📋 WASI preview2 integration
- 📋 Component registry/package manager

## Component Model vs wasm-bindgen

| Feature | Component Model | wasm-bindgen |
|---------|-----------------|--------------|
| Language | Any | Rust ↔ JS only |
| Type System | WIT (language-agnostic) | JS-specific |
| Composition | Multi-language | Single boundary |
| Standardization | W3C proposal | Rust ecosystem |
| Browser Support | Future | Current |

wasm-bindgen is production-ready for Rust+JS. Component Model is the future standard for all languages.

## Example: Multi-Language App

A complete example using Vais, Rust, and JS:

### 1. Vais Component (logger.vais)
```vais
#[wit_export("log")]
F log(msg: str) {
    # WASI syscall
    fd_write(WASM_STDOUT, str_to_ptr(msg), strlen(msg), 0)
}
```

### 2. Rust Component (processor.rs)
```rust
wit_bindgen::generate!("processor");

impl Processor for Component {
    fn process(data: String) -> String {
        // Call Vais logger
        logger::log(&format!("Processing: {}", data));
        data.to_uppercase()
    }
}
```

### 3. JavaScript Host
```javascript
import { instantiate } from './bindings.js';

const { process } = await instantiate();
const result = process("hello");
console.log(result); // "HELLO"
// Logger output: "Processing: hello"
```

All three languages interoperate through WIT interfaces.

## Tools

- **wasm-tools** — Compose and inspect components ([GitHub](https://github.com/bytecodealliance/wasm-tools))
- **wit-bindgen** — Generate bindings from WIT ([GitHub](https://github.com/bytecodealliance/wit-bindgen))
- **componentize-js** — Turn JS into components ([GitHub](https://github.com/bytecodealliance/ComponentizeJS))

## See Also

- [Getting Started](./getting-started.md) — Basic WASM compilation
- [JS Interop](./js-interop.md) — Current JavaScript integration (without Component Model)
- [W3C Component Model Proposal](https://github.com/WebAssembly/component-model)
- [WIT Specification](https://component-model.bytecodealliance.org/design/wit.html)
