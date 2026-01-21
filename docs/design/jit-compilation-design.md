# JIT Compilation Design for Vais REPL

## Overview

This document describes the JIT (Just-In-Time) compilation architecture for the Vais REPL, enabling immediate code execution without writing to disk or invoking external compilers.

## Goals

1. **Eliminate Disk I/O**: No temporary files for compilation
2. **Reduce Latency**: Sub-millisecond compilation for small expressions
3. **Maintain Compatibility**: Reuse existing codegen infrastructure where possible
4. **Incremental Compilation**: Support accumulating definitions across REPL sessions

## Architecture Options

### Option A: Cranelift JIT (Recommended)

**Pros:**
- Pure Rust, no external dependencies
- Fast compilation (optimized for JIT use cases)
- Works on all platforms without LLVM installation
- Smaller binary size
- Well-suited for interpreter/REPL scenarios

**Cons:**
- Less optimized code than LLVM
- Fewer target architectures
- Different IR format requires translation

### Option B: LLVM MCJIT via Inkwell

**Pros:**
- Reuses existing inkwell integration
- Highly optimized code generation
- Same IR as AOT compilation

**Cons:**
- Requires LLVM 17+ installed
- Slower compilation than Cranelift
- More complex setup

### Decision: Cranelift JIT

We choose Cranelift for the REPL JIT because:
1. The REPL prioritizes fast compilation over optimal runtime performance
2. No external dependencies (works out of the box)
3. Pure Rust implementation is easier to maintain

## Implementation Plan

### 1. New Crate Structure

```
crates/vais-jit/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API
│   ├── compiler.rs      # JIT compiler implementation
│   ├── runtime.rs       # JIT runtime management
│   └── types.rs         # Type mapping to Cranelift types
```

### 2. Dependencies

```toml
[dependencies]
cranelift = "0.115"
cranelift-jit = "0.115"
cranelift-module = "0.115"
cranelift-native = "0.115"
vais-ast = { path = "../vais-ast" }
vais-types = { path = "../vais-types" }
```

### 3. Core Components

#### JitCompiler

```rust
pub struct JitCompiler {
    module: JITModule,
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    data_description: DataDescription,
}

impl JitCompiler {
    pub fn new() -> Result<Self, String>;
    pub fn compile_function(&mut self, func: &Function) -> Result<*const u8, String>;
    pub fn compile_expression(&mut self, expr: &Expr) -> Result<i64, String>;
}
```

#### Type Mapping

| Vais Type | Cranelift Type |
|-----------|----------------|
| i8        | I8             |
| i16       | I16            |
| i32       | I32            |
| i64       | I64            |
| f32       | F32            |
| f64       | F64            |
| bool      | I8             |
| *T        | I64 (pointer)  |

### 4. REPL Integration

```rust
// In repl.rs
fn evaluate_expr_jit(source: &str, jit: &mut JitCompiler) -> Result<String, String> {
    let ast = parse(source)?;
    let mut checker = TypeChecker::new();
    checker.check_module(&ast)?;

    // Use JIT instead of codegen + clang
    let result = jit.compile_expression(&ast.items[0])?;
    Ok(format!("{}", result))
}
```

### 5. Feature Flag

```toml
# In vaisc/Cargo.toml
[features]
default = []
jit = ["vais-jit"]
```

The REPL will fallback to the current clang-based evaluation if JIT is not enabled.

## Compilation Pipeline

```
Source Code
    │
    ▼
┌─────────┐
│  Parse  │  (vais-parser)
└────┬────┘
     │
     ▼
┌─────────┐
│  Type   │  (vais-types)
│  Check  │
└────┬────┘
     │
     ▼
┌─────────────┐
│  Cranelift  │  (vais-jit)
│     IR      │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Machine   │  (cranelift-jit)
│    Code     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Execute   │  (direct call)
└─────────────┘
```

## Memory Management

### Function Lifetime

- Functions compiled in a REPL session persist until `:clear` command
- Memory is freed when JitCompiler is dropped
- External function references (printf, malloc) are resolved at compile time

### Data Segment

- String literals stored in data section
- Global variables supported through data segment

## Error Handling

1. **Parse Errors**: Return immediately with line/column info
2. **Type Errors**: Return with detailed type mismatch info
3. **JIT Errors**: Translate Cranelift errors to user-friendly messages
4. **Runtime Errors**: Catch SIGSEGV/SIGFPE and report gracefully

## Testing Strategy

1. **Unit Tests**: Each JIT compiler method
2. **Integration Tests**: Full REPL session scenarios
3. **Performance Tests**: Compare JIT vs clang compilation time

## Future Enhancements

1. **Debug Info**: DWARF generation for JIT code
2. **Profiling**: JIT code instrumentation
3. **Caching**: Cache compiled functions by hash
4. **Optimization**: Add optimization passes for hot functions

## Timeline

- Phase 1: Basic JIT compiler with integer expressions
- Phase 2: Function definitions and calls
- Phase 3: Control flow (if/loop/match)
- Phase 4: Struct and enum support
- Phase 5: REPL integration and testing
