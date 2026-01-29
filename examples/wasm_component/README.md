# WebAssembly Component Model Examples

This directory contains examples demonstrating Vais's WebAssembly Component Model support.

## Prerequisites

1. Install WASI SDK:
```bash
curl -LO https://github.com/WebAssembly/wasi-sdk/releases/latest/download/wasi-sdk-21.0-macos.tar.gz
tar xf wasi-sdk-21.0-macos.tar.gz
export WASI_SDK_PATH=$PWD/wasi-sdk-21.0
```

2. Install wasm-tools:
```bash
cargo install wasm-tools
```

3. Install wasmtime:
```bash
curl https://wasmtime.dev/install.sh -sSf | bash
```

## Examples

### calculator.vais

A simple calculator component that exports arithmetic functions.

**Build:**
```bash
vaisc --target wasi-preview2 calculator.vais -o calculator.wasm
```

**Create Component:**
```bash
wasm-tools component new calculator.wasm -o calculator.component.wasm
```

**Run:**
```bash
wasmtime run calculator.component.wasm
```

**Generate WIT:**
```wit
package vais:calculator@0.1.0;

interface calculator {
  add: func(a: s32, b: s32) -> s32;
  subtract: func(a: s32, b: s32) -> s32;
  multiply: func(a: s32, b: s32) -> s32;
  divide: func(a: s32, b: s32) -> result<s32, string>;
}

world calculator-world {
  export calculator;
}
```

## Notes

- All examples target wasi-preview2 for full component model support
- Functions marked with `export` become part of the component interface
- Use Result types for error handling in exported functions
- Component validation can be done with: `wasm-tools validate --features component-model <file>`
