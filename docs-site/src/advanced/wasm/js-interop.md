# WASM-JavaScript Interop

Vais provides first-class WebAssembly and JavaScript interop through `#[wasm_import]` and `#[wasm_export]` attributes.

## Importing JS Functions

Use `#[wasm_import("module", "name")]` to import JavaScript functions into Vais:

```vais
#[wasm_import("env", "console_log")]
N F console_log(msg: str)

#[wasm_import("env", "get_time")]
N F get_time() -> f64

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

## TypeScript Declarations

The `vais-bindgen` tool can automatically generate TypeScript `.d.ts` files for type safety:

```bash
vais-bindgen --wasm-js --typescript input.vais -o bindings.js
```

This generates `bindings.d.ts`:
```typescript
export interface VaisModule {
    memory: WebAssembly.Memory;
    add(a: number, b: number): number;
    fibonacci(n: number): number;
}

export function load(wasmPath: string): Promise<VaisModule>;
```

Use in TypeScript:
```typescript
import { load, VaisModule } from './bindings';

const module: VaisModule = await load('output.wasm');
const result: number = module.add(2, 3);
```

The type checker ensures you call exported functions with correct types.

## Memory Management

WASM modules use linear memory, accessible from both Vais and JavaScript.

### Accessing Linear Memory

WASM memory is a resizable `ArrayBuffer`:
```javascript
const memory = instance.exports.memory;
const bytes = new Uint8Array(memory.buffer);

// Read i64 at offset 0
const view = new DataView(memory.buffer);
const value = view.getBigInt64(0, true); // true = little-endian
```

### String Passing

Strings require encoding/decoding across the boundary:

**Vais → JavaScript** (export string):
```vais
#[wasm_export("get_message")]
F get_message() -> str {
    R "Hello from Vais"
}
```

JavaScript needs to read from memory:
```javascript
// Vais returns {ptr, len} struct
function readString(ptr, len, memory) {
    const bytes = new Uint8Array(memory.buffer, ptr, len);
    return new TextDecoder('utf-8').decode(bytes);
}

const { ptr, len } = instance.exports.get_message();
const msg = readString(ptr, len, instance.exports.memory);
```

**JavaScript → Vais** (import string):
```vais
#[wasm_import("env", "log_str")]
N F log_str(ptr: i64, len: i64)

F main() {
    msg := "Hello"
    log_str(str_to_ptr(msg), strlen(msg))
}
```

JavaScript implementation:
```javascript
env: {
    log_str: (ptr, len) => {
        const msg = readString(ptr, len, instance.exports.memory);
        console.log(msg);
    }
}
```

### Memory Growth

WASM memory can grow dynamically:
```vais
# Grow by 1 page (64KB)
old_size := memory_grow(1)

I old_size == -1 {
    # Growth failed
}
```

JavaScript can also grow memory:
```javascript
const oldPages = instance.exports.memory.grow(1);
```

## Complex Types

Passing structs and arrays requires serialization.

### Struct Serialization

Vais struct:
```vais
S Point {
    x: f64,
    y: f64
}

#[wasm_export("process_point")]
F process_point(ptr: i64) -> i64 {
    # Read struct from linear memory
    x := load_f64(ptr)
    y := load_f64(ptr + 8)

    # Process and write back
    result_x := x * 2.0
    result_y := y * 2.0

    result_ptr := wasm_heap_alloc(16)
    store_f64(result_ptr, result_x)
    store_f64(result_ptr + 8, result_y)

    R result_ptr
}
```

JavaScript caller:
```javascript
function writePoint(point, memory) {
    const ptr = instance.exports.wasm_heap_alloc(16);
    const view = new DataView(memory.buffer);
    view.setFloat64(ptr, point.x, true);
    view.setFloat64(ptr + 8, point.y, true);
    return ptr;
}

function readPoint(ptr, memory) {
    const view = new DataView(memory.buffer);
    return {
        x: view.getFloat64(ptr, true),
        y: view.getFloat64(ptr + 8, true)
    };
}

const inputPtr = writePoint({ x: 1.5, y: 2.5 }, instance.exports.memory);
const outputPtr = instance.exports.process_point(inputPtr);
const result = readPoint(outputPtr, instance.exports.memory);
console.log(result); // { x: 3.0, y: 5.0 }
```

### Array/Vec Passing

Vais expects arrays as `{ptr, len}` fat pointers:
```vais
#[wasm_export("sum_array")]
F sum_array(arr_ptr: i64, arr_len: i64) -> i64 {
    total := 0
    i := 0
    L i < arr_len {
        elem := load_i64(arr_ptr + i * 8)
        total = total + elem
        i = i + 1
    }
    R total
}
```

JavaScript:
```javascript
function writeI64Array(arr, memory) {
    const ptr = instance.exports.wasm_heap_alloc(arr.length * 8);
    const view = new DataView(memory.buffer);
    arr.forEach((val, i) => {
        view.setBigInt64(ptr + i * 8, BigInt(val), true);
    });
    return { ptr, len: arr.length };
}

const { ptr, len } = writeI64Array([1, 2, 3, 4, 5], instance.exports.memory);
const sum = instance.exports.sum_array(ptr, len);
console.log(sum); // 15n (BigInt)
```

## Error Handling

Vais uses `Result<T, E>` for error handling. Across WASM boundary:

```vais
#[wasm_export("divide")]
F divide(a: i64, b: i64) -> i64 {
    I b == 0 {
        # Return error code
        R -1
    }
    R a / b
}
```

JavaScript checks the result:
```javascript
const result = instance.exports.divide(10, 2);
if (result === -1n) {
    console.error("Division error");
} else {
    console.log(result); // 5n
}
```

For richer errors, return a struct with `{ok: bool, value: i64, error: str}`.

### Exception Handling

WASM doesn't have native exceptions. Use error codes or Result types:

```vais
# Error codes
C ERR_NONE: i64 = 0
C ERR_INVALID_INPUT: i64 = 1
C ERR_OUT_OF_MEMORY: i64 = 2

S ResultI64 {
    ok: bool,
    value: i64,
    error_code: i64
}

#[wasm_export("safe_divide")]
F safe_divide(a: i64, b: i64) -> ResultI64 {
    I b == 0 {
        R ResultI64 { ok: false, value: 0, error_code: ERR_INVALID_INPUT }
    }
    R ResultI64 { ok: true, value: a / b, error_code: ERR_NONE }
}
```

JavaScript:
```javascript
const result = readResult(instance.exports.safe_divide(10, 0));
if (!result.ok) {
    throw new Error(`Vais error: ${result.error_code}`);
}
```

## Performance Tips

### Minimize Boundary Crossings

Each call between WASM and JS has overhead. Batch operations:

**Slow** (many calls):
```javascript
for (let i = 0; i < 1000; i++) {
    instance.exports.process_item(i);
}
```

**Fast** (one call):
```javascript
instance.exports.process_batch(startIndex, count);
```

### Use Typed Arrays

Direct memory access via `TypedArray` views is faster than individual reads:
```javascript
// Fast: direct view
const i32Array = new Int32Array(memory.buffer, ptr, length);
const sum = i32Array.reduce((a, b) => a + b, 0);

// Slow: individual loads
let sum = 0;
for (let i = 0; i < length; i++) {
    sum += view.getInt32(ptr + i * 4);
}
```

### Preallocate Memory

Allocate large buffers once, reuse them:
```vais
# Global buffer
C BUFFER_SIZE: i64 = 1048576  # 1MB
mut global_buffer: i64 = 0

F init() {
    global_buffer = wasm_heap_alloc(BUFFER_SIZE)
}

F process_data(size: i64) {
    # Reuse global_buffer instead of allocating
}
```

## See Also

- [WASM Component Model](./component-model.md) — High-level type system
- [Getting Started](./getting-started.md) — Basic WASM setup
- [JavaScript Code Generation](../../compiler/js-codegen.md) — Direct JS compilation
