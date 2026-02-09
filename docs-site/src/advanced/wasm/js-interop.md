# WASM-JavaScript Interop

Vais provides first-class WebAssembly and JavaScript interop through `#[wasm_import]` and `#[wasm_export]` attributes.

## Importing JS Functions

Use `#[wasm_import("module", "name")]` to import JavaScript functions into Vais:

```vais
#[wasm_import("env", "console_log")]
X F console_log(msg: str)

#[wasm_import("env", "get_time")]
X F get_time() -> f64

F main() -> i64 {
    console_log("Hello from Vais WASM!")
    t := get_time()
    0
}
```

The first argument is the WASM import module name, and the second is the function name within that module.

## Exporting Vais Functions

Use `#[wasm_export("name")]` to export Vais functions for JavaScript consumption:

```vais
#[wasm_export("add")]
F add(a: i64, b: i64) -> i64 {
    a + b
}

#[wasm_export("fibonacci")]
F fib(n: i64) -> i64 {
    I n <= 1 { n }
    E { @(n - 1) + @(n - 2) }
}
```

These functions become callable from JavaScript after loading the WASM module.

## Standard Web API Bindings

The `std/web.vais` module provides pre-built bindings for common Web APIs:

```vais
U std/web

F main() -> i64 {
    console_log("Hello, browser!")
    dom_set_text("output", "Vais is running in the browser")
    0
}
```

Available APIs in `std/web.vais`:
- **Console** — `console_log`, `console_warn`, `console_error`
- **Timer** — `set_timeout`, `set_interval`, `clear_interval`
- **DOM** — `dom_get_text`, `dom_set_text`, `dom_set_html`, `dom_add_class`
- **Fetch** — `fetch_text`, `fetch_json`
- **Storage** — `storage_get`, `storage_set`, `storage_remove`
- **Window** — `get_url`, `navigate_to`
- **Canvas** — `canvas_fill_rect`, `canvas_stroke_rect`, `canvas_fill_text`

## Compiling to WASM

```bash
# Compile to WebAssembly
vaisc --target wasm32-unknown-unknown input.vais -o output.wasm
```

## Using from JavaScript

```javascript
// Load the WASM module
const response = await fetch('output.wasm');
const { instance } = await WebAssembly.instantiate(
    await response.arrayBuffer(),
    {
        env: {
            console_log: (ptr, len) => console.log(readString(ptr, len)),
            get_time: () => performance.now(),
        }
    }
);

// Call exported functions
const result = instance.exports.add(2, 3);
console.log(result); // 5
```

## Bindgen

The `vais-bindgen` crate can automatically generate JavaScript glue code:

```bash
vais-bindgen --wasm-js input.vais -o bindings.js
```

This generates:
- `createImports()` — JavaScript import object for `WebAssembly.instantiate`
- `load()` — async loader function
- TypeScript `.d.ts` type declarations

## Serialization

Complex types (structs, enums) are serialized across the WASM boundary using a compact binary format. The `WasmSerde` system handles:

- Struct field layout and alignment
- Enum tag encoding
- String encoding (UTF-8 with length prefix)
- Array/Vec serialization

## See Also

- [WASM Component Model](./component-model.md)
- [JavaScript Code Generation](../../compiler/js-codegen.md)
