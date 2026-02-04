# Vais Migration Guide

This guide helps you migrate Vais projects from pre-release versions (v0.x) to the stable v1.0.0 release.

---

## Migrating from v0.x to v1.0

### Overview

Vais v1.0.0 is the first stable release. Since there was no prior stable version, there are **no breaking changes** to migrate from. However, if you have been using Vais from its development branches, this section documents the changes that occurred during the pre-release period.

The v1.0.0 release stabilizes all public APIs. From this point forward, the 1.x series follows strict semantic versioning: no breaking changes until v2.0.

---

## Stabilized APIs

The following APIs are now considered stable and covered by backward-compatibility guarantees:

### Language Syntax
- All single-character keywords: `F`, `S`, `E`, `I`, `L`, `M`, `T`, `R`
- Self-recursion operator `@`
- Variable binding `:=` and `mut` modifier
- Ternary operator `cond ? a : b`
- Comment syntax `#`
- Generic syntax `<T>` with trait bounds
- Trait definition `T Name { ... }` and `impl Name for Type { ... }`
- Pattern matching `M expr { pattern => result }`
- Async function declaration `async F`
- Lifetime annotations (where explicitly used)

### Compiler CLI (`vaisc`)
- `vaisc <file.vais>` -- compile to LLVM IR
- `vaisc --repl` -- interactive REPL
- `vaisc --parallel` -- parallel compilation
- `vaisc --format <file.vais>` -- code formatting
- `vaisc --check <file.vais>` -- type checking without codegen

### Standard Library Modules
All 65+ modules under `std/` are stable. Public function signatures in these modules will not change in backward-incompatible ways during the 1.x series.

### LSP and DAP Protocols
The LSP server (`vais-lsp`) and DAP server (`vais-dap`) follow their respective protocol specifications and are considered stable.

---

## Deprecated Features

No features are deprecated in v1.0.0. This section will be updated in future minor releases if any deprecations are introduced.

---

## Syntax Changes History

The following syntax decisions were finalized during the pre-release development:

### v0.1.0 -> v0.2.0
- Generic functions now fully monomorphized at runtime (previously type-checked only)
- Trait dynamic dispatch via `&dyn Trait` with vtable-based dispatch
- Built-in `print()`/`println()` replaced manual printf FFI calls
- String operations (`+`, comparisons, `.len()`, `.contains()`) added
- Array mutation `arr[i] = val` enabled
- `format()` function for formatted strings

### v0.2.0 -> v0.3.0
- Generics syntax finalized as `<T>` with where clauses
- Trait syntax finalized as `T TraitName { ... }`
- Pattern matching `M` keyword stabilized with exhaustiveness checking
- Async/await syntax finalized as `async F` with `Future<T>` return types

### v0.3.0 -> v1.0.0-rc.1
- Ownership and borrow checking added (no syntax change, semantic enforcement)
- Lifetime elision rules aligned with Rust's 3-rule system
- GATs and trait specialization added
- Error messages overhauled with "did you mean?" suggestions
- Stable ABI v1.0.0 declared with FFI compatibility guarantees

### v1.0.0-rc.1 -> v1.0.0
- No syntax changes; stabilization and documentation only
- All 14 security audit findings resolved
- Performance regression test baselines established

---

## Build System Changes

### Workspace Structure

The Cargo workspace now includes 30+ crates. If you previously depended on a subset, ensure your workspace members list is up to date. Key additions since early development:

| Crate | Purpose | Added In |
|-------|---------|----------|
| `vais-mir` | Middle IR for optimization | v0.2.0 |
| `vais-dap` | Debug Adapter Protocol | v0.3.0 |
| `vais-macro` | Declarative macro system | v0.3.0 |
| `vais-security` | Security analysis | v0.3.0 |
| `vais-supply-chain` | SBOM and license audit | v0.3.0 |
| `vais-testgen` | Property-based test generation | v0.3.0 |
| `vais-registry-server` | Package registry | v0.3.0 |
| `vais-query` | Incremental compilation | v0.3.0 |

### LLVM Version

Vais v1.0.0 requires **LLVM 17**. If you were using an earlier LLVM version, upgrade before building:

```bash
# macOS
brew install llvm@17

# Ubuntu/Debian
sudo apt install llvm-17-dev

# Verify
llvm-config-17 --version
```

### Rust Toolchain

Minimum Supported Rust Version (MSRV) is **1.75**. The project uses Rust edition 2021.

```bash
rustup update stable
rustc --version  # Should be >= 1.75.0
```

### Python and Node.js Bindings

If you use the language bindings, the build process has not changed:

```bash
# Python (requires maturin)
cd crates/vais-python && maturin develop

# Node.js (requires napi-rs)
cd crates/vais-node && npm run build
```

---

## Recommended Migration Steps

1. **Update your toolchain**: Ensure Rust >= 1.75, LLVM 17, and clang 17 are installed.
2. **Pull the latest code**: `git checkout main && git pull`
3. **Clean build**: `cargo clean && cargo build --release`
4. **Run tests**: `cargo test` to verify everything passes (400+ unit tests, 165+ E2E tests).
5. **Check your `.vais` files**: Run `vaisc --check` on your source files to catch any type errors surfaced by the new borrow checker and lifetime inference.
6. **Review warnings**: The compiler may emit new warnings for patterns that were previously unchecked. Address these before they become errors in future versions.

---

## Getting Help

- **Documentation**: [https://vaislang.github.io/vais/](https://vaislang.github.io/vais/)
- **Issue Tracker**: [https://github.com/vaislang/vais/issues](https://github.com/vaislang/vais/issues)
- **Contributing**: See [CONTRIBUTING.md](./CONTRIBUTING.md) for development setup and PR guidelines.
