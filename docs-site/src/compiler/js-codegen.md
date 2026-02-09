# JavaScript Code Generation

Vais can compile to JavaScript (ESM modules), enabling Vais code to run in browsers and Node.js environments.

## Overview

The `vais-codegen-js` crate provides a JavaScript backend alongside the primary LLVM backend. It generates clean, readable ES module output.

## Usage

```bash
# Compile to JavaScript
vaisc --target js input.vais -o output.js

# Compile to JavaScript module
vaisc --target js --module input.vais -o output.mjs
```

## Features

- **ES Module output** — generates standard `import`/`export` syntax
- **Type-safe codegen** — preserves Vais type semantics in JavaScript
- **Struct mapping** — Vais structs compile to JavaScript classes
- **Enum support** — tagged unions with pattern matching
- **String interpolation** — maps to template literals
- **Operator overloading** — preserves Vais operator semantics

## Example

**Vais source:**

```vais
S Point { x: f64, y: f64 }

F dist(p: Point) -> f64 {
    sqrt(p.x * p.x + p.y * p.y)
}

F main() -> i64 {
    p := Point { x: 3.0, y: 4.0 }
    puts("distance = {dist(p)}")
    0
}
```

**Generated JavaScript:**

```javascript
export class Point {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }
}

export function dist(p) {
    return Math.sqrt(p.x * p.x + p.y * p.y);
}

export function main() {
    const p = new Point(3.0, 4.0);
    console.log(`distance = ${dist(p)}`);
    return 0;
}
```

## Architecture

The JS codegen pipeline:

```
AST → Type Checker → JsCodegen → ESM output (.js/.mjs)
```

Key components in `crates/vais-codegen-js/`:
- `lib.rs` — main entry point and module generation
- `expr.rs` — expression code generation
- `stmt.rs` — statement code generation
- `types.rs` — type mapping (Vais types → JS representations)

## Limitations

- No direct memory management (pointers compile to references)
- Integer arithmetic uses JavaScript's `Number` type (no true i64 for values > 2^53)
- FFI/extern functions are not supported in JS target
- GPU codegen is not available for JS target

## See Also

- [WASM JS Interop](../advanced/wasm/js-interop.md) — using `#[wasm_import]` and `#[wasm_export]`
- [Architecture](./architecture.md) — compiler pipeline overview
