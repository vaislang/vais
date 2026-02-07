# Vais Self-Hosting Compiler Roadmap

## Current Status: v0.8.0 — Full Bootstrap + Toolchain 🎉

Bootstrap achieved (Stage1→Stage2→Stage3 fixed point).
MIR pipeline, LSP server, code formatter, and doc generator implemented.

---

## Completed Milestones

### Bootstrap (v0.7.0) ✅
- [x] Stage1→Stage2→Stage3 fixed point (SHA256: e14776a6..., 17,807줄)
- [x] Inkwell builtins: fopen_ptr/memcpy_str wrappers + realloc
- [x] E2E: 241 tests, selfhost lexer: 114 tests — all passing

### Core Language ✅
- [x] All keywords: F, S, E, I, L, R, B, C, M, X, W, T, U, P, mut
- [x] All types: i64, i32, i16, i8, u64, u32, u16, u8, f64, f32, bool, str
- [x] Binary operators: +, -, *, /, %, <, >, <=, >=, ==, !=, &&, ||, &, |, ^, <<, >>
- [x] Unary operators: -, !
- [x] Index expressions [i], Array literals [e1, e2, ...]
- [x] Bitwise operators: &, |, ^, <<, >>
- [x] Function calls (multi-param), method calls, field access, self calls (@)
- [x] Block, if-else, loop, match expressions
- [x] Struct literals (Name { field: value })
- [x] Let bindings (:=, := mut, : Type = expr)
- [x] Import system (U module)

### Type System ✅
- [x] Generic types <T> parsing + resolution
- [x] Trait resolution (TraitDefInfo/TraitImplInfo)
- [x] Type checker: 32/32 expression types + 7/7 item types
- [x] Codegen: 32/32 expression types
- [x] Module system: 100%
- [x] Type mismatch detailed descriptions
- [x] Error recovery (continue checking after errors)

### MIR Pipeline ✅ (8,000+ LOC)
- [x] MIR data structures (mir.vais, 659 LOC)
- [x] MIR builder API (mir_builder.vais, 297 LOC)
- [x] AST → MIR lowering (mir_lower.vais, 1,420 LOC)
- [x] MIR → LLVM IR emission (mir_emit_llvm.vais, 1,228 LOC)
- [x] MIR optimizer: constant prop, const fold, DCE, unreachable elimination (mir_optimizer.vais, 756 LOC)
- [x] MIR analysis: CFG, liveness, dominance, loops, reaching defs, use-def (mir_analysis.vais, 1,536 LOC)
- [x] MIR borrow checker: loans, move/copy, borrow conflicts, lifetimes (mir_borrow.vais, 1,357 LOC)
- [x] Pipeline integration (mir_main.vais, ~350 LOC)

### Toolchain ✅
- [x] LSP Server (2,437 LOC): hover, go-to-def, find-refs, completion, document-symbols, diagnostics
  - lsp_json.vais (608) + lsp_symbols.vais (688) + lsp_handlers.vais (847) + lsp_main.vais (294)
- [x] Code Formatter (1,475 LOC): AST→pretty-print, --check, --write modes
  - fmt.vais (1,289) + fmt_main.vais (186)
- [x] Doc Generator (1,236 LOC): Markdown output with signatures, field tables, doc comments
  - doc_gen.vais (1,046) + doc_gen_main.vais (190)

### Parser Fixes (2026-02-07) ✅
- [x] Field access: `p.x` now correctly creates EXPR_FIELD nodes
- [x] Else parsing: `E` (token ID 3) used instead of `TOK_KW_ELSE` (token ID 19)
- [x] Mutable let: `:= mut` pattern now supported
- [x] Struct literal ambiguity: only uppercase identifiers attempt struct literal parsing
- [x] Impl/trait method params_len storage

---

## Future Work

### Language Extensions
- [ ] While loop sugar
- [ ] Negative number literals in lexer
- [ ] Pattern matching enhancements (wildcard, variable binding, guards)
- [ ] Option<T> / Result<T, E> support
- [ ] Pointer types (*T), references (&T, &mut T)
- [ ] Defer statement

### Standard Library
- [ ] Vec<T>
- [ ] String (owned)
- [ ] HashMap<K, V>
- [ ] File I/O wrappers
- [ ] Better print functions

### Known Limitations
1. Match scrutinee: must be simple identifier (not complex expression) due to `{` ambiguity
2. Selfhost parser: doesn't handle all Vais syntax (e.g., attributes, closures) — designed for selfhost source files
3. `str` type is non-Copy: requires `--no-ownership-check` for selfhost compilation

---

## Build Instructions

```bash
# Compile with Rust compiler
cargo run --bin vaisc -- selfhost/main.vais -o /tmp/vaisc-stage1 --no-ownership-check

# Compile formatter
cargo run --bin vaisc -- selfhost/fmt_main.vais -o /tmp/vais-fmt --no-ownership-check

# Compile doc generator
cargo run --bin vaisc -- selfhost/doc_gen_main.vais -o /tmp/vais-doc --no-ownership-check

# Compile MIR pipeline
cargo run --bin vaisc -- selfhost/mir_main.vais -o /tmp/vais-mir --no-ownership-check
```

---

## Version History

- **v0.8.0** - Toolchain: Formatter + Doc Generator + Parser Fixes
  - Code formatter: AST-based pretty-printing with --check/--write modes (1,475 LOC)
  - Doc generator: Markdown output with signatures, field tables, doc comments (1,236 LOC)
  - Parser fixes: field access, else token, mut let, struct literal ambiguity
- **v0.7.1** - LSP Server + MIR Pipeline Integration
  - LSP: hover, go-to-def, find-refs, completion, document-symbols, diagnostics (2,437 LOC)
  - MIR pipeline: Source→Lex→Parse→MIR→Opt→LLVM IR (~350 LOC)
- **v0.7.0** - Bootstrap achieved! Stage1→Stage2→Stage3 fixed point
- **v0.6.1** - Type Checker 100% + E001 resolved
- **v0.6.0** - TC 95%+, Codegen 100%, Module 100%
- **v0.5.2** - Array literal support
- **v0.5.1** - Bitwise operators + Index expressions
- **v0.5.0** - Generic type resolution
- **v0.4.1** - Multi-param function fix + SIGBUS crash fix
- **v0.4.0** - Import system, module separation
- **v0.3.0** - Match expressions
- **v0.2.0** - Multi-function, structs, impl blocks
- **v0.1.0** - Basic single-function compilation
