# Vais Language Stability Declaration

**Version:** 1.0.0
**Effective Date:** February 2026
**Status:** STABLE

---

## Introduction

This document defines the stability guarantees, backward compatibility policy, and support timeline for the Vais programming language. As of version 1.0.0, Vais transitions from experimental development to a production-ready language with formal stability commitments.

The goal of this policy is to provide users with confidence that:
1. Code written for Vais 1.x will continue to work in future 1.x releases
2. Breaking changes are limited to major version bumps
3. Deprecation follows a predictable timeline
4. The language specification is frozen for the 1.x series

---

## Language Specification

### 1.0 Specification Status: FROZEN

The Vais language specification version 1.0 is **frozen** as of this release. This includes:

- **Syntax** - All keyword meanings, operators, and expression forms
- **Semantics** - Type system rules, ownership semantics, lifetime rules
- **Core language features**:
  - Single-character keywords (`F`, `S`, `E`, `I`, `L`, `M`, `V`, `R`, `T`, `W`)
  - Self-recursion operator (`@`)
  - Expression-oriented design
  - Generics and traits
  - Ownership and borrowing
  - Lifetime annotations
  - Pattern matching
  - Async/await
  - Closures
  - Modules and imports

### Changes Allowed Within 1.x

The following changes are **allowed** in minor releases (1.1, 1.2, etc.) without breaking compatibility:

1. **Bug fixes** - Corrections to specification violations or compiler bugs
2. **Clarifications** - Better documentation of existing behavior
3. **New features** - Additive features that don't change existing semantics (e.g., new syntax for optional features)
4. **Performance improvements** - Optimization that doesn't change observable behavior
5. **Error message improvements** - Better diagnostics and suggestions

### Changes Requiring Major Version (2.0+)

The following changes **require** a major version bump:

1. **Syntax changes** - Changing keyword meanings or removing syntax
2. **Semantic changes** - Altering type system rules or ownership behavior
3. **Removing features** - Deprecating and removing language constructs
4. **Breaking ABI changes** - Incompatible binary interface modifications
5. **Standard library breaking changes** - Removing public APIs or changing signatures

---

## Backward Compatibility Policy

### Semantic Versioning

Vais follows [Semantic Versioning 2.0.0](https://semver.org/) with the format `MAJOR.MINOR.PATCH`:

- **MAJOR** (e.g., 1.0 → 2.0)
  - Breaking changes to language syntax or semantics
  - Breaking changes to stable APIs (compiler, standard library)
  - ABI-incompatible changes
  - Removal of deprecated features

- **MINOR** (e.g., 1.0 → 1.1)
  - New language features (backward compatible)
  - New standard library modules or functions
  - New compiler features (optimization passes, diagnostics)
  - Deprecation warnings (feature still works)
  - Performance improvements

- **PATCH** (e.g., 1.0.0 → 1.0.1)
  - Bug fixes (correcting incorrect behavior)
  - Security patches
  - Documentation fixes
  - Minor error message improvements

### Compatibility Guarantees

#### Source Compatibility
Code that compiles without warnings on Vais 1.0.0 will:
- Compile without errors on all 1.x releases
- Produce identical behavior (except for bug fixes)
- Generate equivalent or better performance

#### Binary Compatibility (ABI)
- Binaries compiled with Vais 1.x are compatible with Vais 1.y (where y ≥ x)
- Libraries compiled with 1.0 can be linked with programs compiled with 1.5
- ABI version is tagged in binaries (`@__vais_abi_version = "1.0.0"`)
- ABI changes require major version bump to 2.0

#### Tooling Compatibility
- Language Server Protocol (LSP) remains compatible within 1.x
- Debug symbols (DWARF) maintain backward compatibility
- Package manifest format (`vais.toml`) remains compatible

---

## Stable APIs

The following APIs are **stable** and covered by backward compatibility guarantees:

### 1. Compiler APIs

#### Command-Line Interface
All `vaisc` command-line flags and their behavior:
```bash
vaisc build [--release|--debug] [--target TARGET] [--parallel] ...
vaisc run <file.vais>
vaisc repl
vaisc test
vaisc doc
vaisc fmt
vaisc pkg [init|install|publish|search]
vaisc analyze [--security]
```

#### Exit Codes
- `0` - Success
- `1` - Compilation error
- `2` - Runtime error
- `101` - Internal compiler error (ICE)

#### Error Message Format
JSON error output format (`--error-format json`) is stable:
```json
{
  "type": "error",
  "code": "E001",
  "message": "Type mismatch",
  "span": { "file": "main.vais", "line": 10, "col": 5 },
  "suggestions": [...]
}
```

### 2. Standard Library Public APIs

All public functions, types, and traits in the standard library are stable:

- **Core types**: `Option<T>`, `Result<T,E>`, `Vec<T>`, `String`, `HashMap<K,V>`, etc.
- **Traits**: `Iterator`, `Clone`, `Copy`, `Debug`, `Display`, `Eq`, `Ord`, etc.
- **Modules**: `std::io`, `std::net`, `std::fs`, `std::sync`, `std::thread`, etc.

Private APIs (marked `internal` or undocumented) are **not stable**.

### 3. FFI/ABI

#### C ABI Compatibility
- Vais FFI follows System V AMD64 ABI (Linux/macOS) or Microsoft x64 ABI (Windows)
- `extern "C"` functions match C calling convention
- Struct layout matches C layout for `#[repr(C)]` types

#### ABI Version
- Current ABI version: **1.0.0**
- Encoded in binaries as `@__vais_abi_version` global constant
- Compatibility checked at link time:
  - **Compatible**: 1.0.x with 1.0.y (any x, y)
  - **Compatible**: 1.0.x with 1.y.z (y > 0, any x, z)
  - **Incompatible**: 1.x with 2.y (requires recompilation)

#### Calling Conventions
Supported calling conventions are stable:
- `cdecl` (default)
- `stdcall` (Windows)
- `fastcall`
- `system` (platform-specific)

### 4. Language Server Protocol (LSP)

LSP features are stable for 1.x:
- `textDocument/completion`
- `textDocument/hover`
- `textDocument/definition`
- `textDocument/references`
- `textDocument/rename`
- `textDocument/formatting`
- `textDocument/codeAction` (refactorings)
- `textDocument/inlayHint`
- `textDocument/codeLens`

New LSP features may be added in minor releases.

### 5. Package Manifest (`vais.toml`)

The package manifest format is stable:
```toml
[package]
name = "myproject"
version = "0.1.0"
authors = ["..."]
license = "MIT"

[dependencies]
stdlib = "1.0"
```

New optional fields may be added in minor releases.

---

## Unstable Features

The following features are **unstable** and may change in minor releases:

### 1. Experimental Language Features

- **Dependent types** - Full dependent type support is experimental
- **Linear types** - Linear type system integration is incomplete
- **Effect system internals** - Implementation may change
- **GPU backend** - CUDA/Metal/AVX-512/NEON code generation is experimental

These features require explicit opt-in (e.g., `#![feature(dependent_types)]`).

### 2. Compiler Internals

- **Macro system internals** - TokenStream representation may change
- **Compiler plugin API** - Plugin loading mechanism may be revised
- **MIR (Mid-level IR)** - Internal representation subject to change
- **Query system internals** - Salsa-based caching implementation

User-facing macro API (`macro_rules!`, procedural macros) is stable.

### 3. JIT Compilation API

- **Cranelift JIT internals** - Tiered JIT implementation may change
- **Runtime code generation API** - Unstable and may be revised

JIT functionality is available but API is not guaranteed.

### 4. Development Tools

- **Hot reloading API** - Implementation may change
- **Profiler output format** - May be revised for better tooling
- **Debug visualizers** - Custom type rendering in debugger

These tools work but their interfaces are not frozen.

---

## Deprecation Policy

### Deprecation Process

When a feature needs to be removed, it follows this process:

1. **Deprecation Warning** (Minor release, e.g., 1.5.0)
   - Feature marked as deprecated in documentation
   - Compiler emits deprecation warning (suppressible with `#[allow(deprecated)]`)
   - Alternative suggested in warning message
   - Minimum 6-month deprecation period before removal

2. **Deprecation Period** (At least 6 months)
   - Deprecated feature continues to work
   - Documentation updated with migration guide
   - Community notified via blog post and release notes

3. **Removal** (Next major release, e.g., 2.0.0)
   - Feature removed from language/standard library
   - Compiler error if deprecated feature is used
   - Migration tool provided when possible

### Deprecation Guarantees

- **Minimum deprecation period**: 6 months
- **Notice**: All deprecations announced in release notes
- **Migration path**: Clear alternative provided
- **Tool support**: Automated migration when feasible (e.g., `vaisc migrate`)

### Example Deprecation Timeline

```
v1.5.0 (Jan 2027)  - Feature X deprecated, warning emitted
                     Migration guide published
v1.6.0 (Apr 2027)  - Feature X still works with warning
v1.7.0 (Jul 2027)  - Feature X still works with warning
v2.0.0 (Jan 2028)  - Feature X removed
                     Compiler error if used
```

---

## Support Timeline

### Long-Term Support (LTS)

Vais provides long-term support for major releases:

| Version | Release Date | End of Support | Support Type |
|---------|--------------|----------------|--------------|
| 1.0.x   | Feb 2026     | Feb 2028       | Security patches, critical bugs |
| 1.x     | Feb 2026+    | Next major release | Active development |
| 2.0.x   | ~2027        | TBD            | TBD |

### Support Levels

1. **Active Development** (current minor version, e.g., 1.7)
   - New features
   - Bug fixes
   - Performance improvements
   - Security patches

2. **Maintenance** (older minor versions within major, e.g., 1.5, 1.6)
   - Critical bug fixes
   - Security patches
   - No new features

3. **Long-Term Support** (1.0.x series)
   - Security patches only
   - Critical bug fixes only
   - 2-year support window

4. **End-of-Life** (after support period)
   - No updates
   - Community support only

### Patching Policy

- **Security vulnerabilities**: Patched within 48 hours for critical issues
- **Critical bugs** (data loss, crashes): Patched within 1 week
- **Non-critical bugs**: Fixed in next minor release
- **Enhancement requests**: Considered for next minor release

---

## Version Compatibility Matrix

### Compiler Versions

| Source Code Version | Can Compile With | Binary Compatible With |
|---------------------|------------------|------------------------|
| 1.0.x               | 1.0+             | 1.0+                   |
| 1.1.x               | 1.1+             | 1.0+                   |
| 1.5.x               | 1.5+             | 1.0+                   |
| 2.0.x               | 2.0+             | 2.0+ (not 1.x)         |

### Standard Library Versions

Standard library versioning follows compiler versioning:
- `stdlib 1.0` ships with `vaisc 1.0.0`
- `stdlib 1.5` ships with `vaisc 1.5.0`
- All `stdlib 1.x` versions are compatible within `vaisc 1.x`

### Cross-Version Compatibility

**Forward compatibility**: Code written for 1.0 compiles with 1.5 ✅
**Backward compatibility**: Code written for 1.5 may not compile with 1.0 ⚠️
  - New syntax/features will error
  - New stdlib functions will error

**Recommendation**: Specify minimum compiler version in `vais.toml`:
```toml
[package]
rust-version = "1.5.0"  # Requires Vais 1.5 or later
```

---

## Experimental Feature Opt-In

Unstable features require explicit opt-in to use:

### In Source Code
```vais
#![feature(dependent_types)]
#![feature(linear_types)]

F my_dependent_function() { ... }
```

### In `vais.toml`
```toml
[features]
experimental = ["dependent_types", "linear_types"]
```

### Compiler Flag
```bash
vaisc build --features experimental
```

### Guarantees for Experimental Features

- **No stability guarantee** - May change in any release (including patches)
- **No deprecation policy** - Can be removed without deprecation period
- **Documentation** - Clearly marked as "experimental" in docs
- **Feature gates** - Cannot be used without explicit opt-in

---

## Stability Exceptions

### Security Vulnerabilities

Security fixes may introduce breaking changes in patch releases if necessary to prevent exploitation. Such changes will be:
1. Clearly documented in release notes
2. Announced via security advisory
3. Minimized to the smallest necessary change

### Compiler Bugs

Compiler bugs that cause incorrect code generation may be fixed in patch releases, even if existing code relied on the buggy behavior. Fixes will be:
1. Documented in release notes
2. Announced if widely used code is affected
3. Accompanied by migration guide when feasible

---

## Edition System (Future)

Starting with Vais 2.0, we may introduce an **Edition** system similar to Rust:

```toml
[package]
edition = "2026"  # Language edition
```

Editions allow:
- **Incompatible changes** within the same compiler version
- **Gradual migration** with mixed-edition projects
- **Long-term stability** for older editions

This system is **not active** in Vais 1.x but may be introduced in 2.0.

---

## Communication Channels

### Release Announcements

- **Blog**: https://vaislang.dev/blog
- **GitHub Releases**: https://github.com/sswoo88/vais/releases
- **Twitter/X**: [@vaislang](https://twitter.com/vaislang)
- **Discord**: https://discord.gg/vais (announcements channel)

### Deprecation Notices

- **Compiler warnings**: Inline deprecation notices
- **Release notes**: Highlighted deprecation section
- **Migration guide**: Dedicated documentation page

### Security Advisories

- **GitHub Security Advisories**: https://github.com/sswoo88/vais/security/advisories
- **Email**: security@vaislang.dev
- **CVE Database**: Common Vulnerabilities and Exposures

---

## Review and Updates

This stability policy will be reviewed:
- **Annually** - January of each year
- **Before major releases** - Before 2.0, 3.0, etc.
- **As needed** - If significant issues arise

Changes to this policy will be:
1. Proposed via RFC (Request for Comments) process
2. Discussed with community
3. Announced in advance (minimum 3 months)
4. Documented in policy changelog

---

## Conclusion

Vais 1.0 represents a commitment to stability and predictability. Users can confidently build production systems knowing that:

- **Code written today will work tomorrow** (within 1.x)
- **Breaking changes are rare and planned** (major versions only)
- **Deprecations follow a predictable process** (6+ month notice)
- **Security and critical bugs are promptly addressed**

We value backward compatibility and will prioritize it in all decisions. When breaking changes are necessary, they will be well-justified, clearly communicated, and supported with migration tooling.

Thank you for choosing Vais for your projects.

---

**Questions?** Contact the team:
- **GitHub Discussions**: https://github.com/sswoo88/vais/discussions
- **Email**: team@vaislang.dev
- **Discord**: https://discord.gg/vais

**Document Version**: 1.0.0
**Last Updated**: February 2026
