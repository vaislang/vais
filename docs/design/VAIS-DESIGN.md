# Vais Design Notes

Vais is the active language line in this repository.

## Principles

- One spelling for each core construct.
- Error messages should provide a concrete correction.
- The compiler command accepts `.vais` source files.
- Examples are executable specifications and must stay value-correct.
- The self-host compiler path is the mainline; direct native emission grows by promoted slices.

## Current Compiler Contract

```text
Vais source (.vais)
  -> scripts/vaisc emit-ir/build/run
  -> native host driver
  -> reusable self-host compiler core
  -> LLVM IR
  -> native executable
```

The product compiler path reads `.vais` files through a native host driver linked
with the reusable self-host core. Python remains for internal checks and
development-only direct-engine fallback paths.
