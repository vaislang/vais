# vais-node

Node.js bindings for the Vais compiler, providing a native JavaScript/TypeScript interface to compile, check, parse, and tokenize Vais source code.

## Features

- **compile(source, options?)** - Compile Vais source to LLVM IR
- **check(source)** - Type check Vais source code
- **parse(source)** - Parse Vais source into an AST
- **tokenize(source)** - Tokenize Vais source code

## Installation

```bash
# Build the Node.js addon
cargo build --release -p vais-node

# The compiled addon will be in target/release/
```

## Usage

```javascript
const vais = require('./target/release/vais_node.node');

// Tokenize source code
const source = "F add(a:i64,b:i64)->i64=a+b";
const tokens = vais.tokenize(source);
tokens.forEach(token => {
    console.log(`${token.tokenType} at ${token.span.start}-${token.span.end}: ${token.text}`);
});

// Check for type errors
const errors = vais.check(source);
if (errors.length === 0) {
    console.log("No type errors!");
} else {
    errors.forEach(err => {
        console.log(`${err.errorType}: ${err.message}`);
    });
}

// Parse to AST
const ast = vais.parse(source);
console.log(`Module with ${ast.items_count} items`);

// Compile to LLVM IR
const ir = vais.compile(source, {
    optLevel: 2,
    moduleName: "my_module"
});
console.log(ir);

// Compile with target specification
const wasmIr = vais.compile(source, {
    target: "wasm32-unknown-unknown"
});
```

## TypeScript Usage

```typescript
interface CompileOptions {
    optLevel?: number;      // 0-3
    moduleName?: string;    // default: "main"
    target?: string;        // e.g., "wasm32-unknown-unknown"
}

interface VaisError {
    message: string;
    span?: { start: number; end: number };
    errorType: string;
}

interface VaisToken {
    tokenType: string;
    span: { start: number; end: number };
    text?: string;
}

declare module 'vais' {
    export function compile(source: string, options?: CompileOptions): string;
    export function check(source: string): VaisError[];
    export function parse(source: string): object;
    export function tokenize(source: string): VaisToken[];
}
```

## API Reference

### Functions

#### compile(source, options?)

Compiles Vais source code to LLVM IR.

**Parameters:**
- `source` (string): The Vais source code
- `options` (CompileOptions, optional): Compilation options
  - `optLevel` (number): Optimization level 0-3 (default: 0)
  - `moduleName` (string): Name of the module (default: "main")
  - `target` (string): Target triple (default: native)

**Returns:** string - The compiled LLVM IR

**Throws:** Error if compilation fails

#### check(source)

Type checks Vais source code.

**Parameters:**
- `source` (string): The Vais source code

**Returns:** VaisError[] - Array of errors (empty if no errors)

#### parse(source)

Parses Vais source code into an AST representation.

**Parameters:**
- `source` (string): The Vais source code

**Returns:** object - Object representing the AST

**Throws:** Error if parsing fails

#### tokenize(source)

Tokenizes Vais source code.

**Parameters:**
- `source` (string): The Vais source code

**Returns:** VaisToken[] - Array of tokens

**Throws:** Error if tokenization fails

### Types

#### VaisError

Represents a compilation or type error.

**Properties:**
- `message` (string): Error message
- `span` ({ start: number, end: number } | undefined): Source location
- `errorType` (string): Type of error (e.g., "TypeError", "ParseError")

#### VaisToken

Represents a token from the lexer.

**Properties:**
- `tokenType` (string): Type of token (e.g., "Function", "Ident", "Int")
- `span` ({ start: number, end: number }): Source location
- `text` (string | undefined): Token text if applicable

#### CompileOptions

Compilation options.

**Properties:**
- `optLevel` (number, optional): Optimization level 0-3
- `moduleName` (string, optional): Module name
- `target` (string, optional): Target triple

## Building

```bash
# Development build
cargo build -p vais-node

# Release build
cargo build --release -p vais-node

# Build with npm (requires package.json setup)
npm install
npm run build
```

## Requirements

- Rust 1.70+
- Node.js 14+
- napi-rs 2.16+

## License

MIT
