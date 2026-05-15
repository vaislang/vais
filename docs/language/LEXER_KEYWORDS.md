# Vais Lexer Keyword Registry

Single source of truth for **every reserved token** the Vais lexer emits as a keyword. Any identifier that is NOT in this list is passed through as `Token::Ident` and never interpreted as a keyword. Adding/removing/renaming keywords requires:

1. Edit `crates/vais-lexer/src/lib.rs`.
2. Update this file.
3. Update `docs/LANGUAGE_SPEC.md` "Keywords" section.
4. If removing: add entry to `docs/language/removed_keywords.md` and bump phase note.
5. Run `./scripts/check-integrity.sh` — must stay green.

Source of truth: `crates/vais-lexer/src/lib.rs` (logos derive attributes).
Cross-reference: `docs/LANGUAGE_SPEC.md` "Keywords" + "Construct Status Matrix".

---

## Step 19 P4 retirement (2026-05-08)

The single-char declaration / control / modifier forms `F` / `S` / `E` / `EN` / `EL` / `M` / `R` / `T` / `U` / `P` / `W` / `X` were retired by Step 19 P4. The lexer no longer accepts those spellings as keywords; they lex as `Token::Ident`. Canonical multi-char forms (`fn` / `struct` / `enum` / `else` / `match` / `return` / `type` / `use` / `pub` / `trait` / `impl`) are the only accepted spellings. Rationale + 6-phase migration record: `docs/design/single-char-keyword-retirement.md` and LESSONS L-009 / L-010.

Non-retired single-char keywords below remain as tokens.

## Single-letter Declaration / Statement Keywords (priority 3, post-P4)

| Token | Lexer variant | Grammar role | Note |
|-------|---------------|--------------|------|
| `I`   | `If`            | If-expression | retained (no multi-char alias) |
| `L`   | `Loop`          | Infinite loop | retained |
| `B`   | `Break`         | Loop break | retained |
| `C`   | `Continue`      | Loop continue **(never Const)** | retained |
| `D`   | `Defer`         | Defer block | retained |
| `O`   | `Union`         | C-style untagged union | retained |
| `N`   | `Extern`        | Extern "C" block | retained |
| `G`   | `Global`        | Global variable | retained |
| `A`   | `Async`         | Async modifier | retained |
| `Y`   | `Await`         | Postfix-await shortcut (alias of `await`) | retained |

## Two-letter Unambiguous Keywords (priority 4)

| Token | Lexer variant | Grammar role |
|-------|---------------|--------------|
| `LF`  | `ForEach`       | For-each loop (`LF i: range`) |
| `LW`  | `While`         | While loop (`LW cond { … }`) |

(`EN` and `EL` were retired with P4 — the variants `EnumKeyword` / `Else` survive in code but `Else` is now reached only via the `else` spelling; `EnumKeyword` is unreachable from the lexer and retained only for downstream parser arm compatibility.)

## Multi-letter Declaration / Item Keywords (priority 3, P4 canonical forms)

| Token | Lexer variant | Role |
|-------|---------------|------|
| `fn`        | `Function`     | Function declaration |
| `struct`    | `Struct`       | Struct declaration |
| `enum`      | `Enum`         | Enum declaration |
| `match`     | `Match`        | Match expression |
| `return`    | `Return`       | Early return |
| `type`      | `TypeKeyword`  | Type alias / trait alias |
| `use`       | `Use`          | Import |
| `pub`       | `Pub`          | Public visibility |
| `trait`     | `Trait`        | Trait (interface) declaration |
| `impl`      | `Impl`         | Impl block (methods / trait impl) |
| `else`      | `Else`         | Else branch |

## Multi-letter Word Keywords

| Token | Lexer variant | Role |
|-------|---------------|------|
| `mut`       | `Mut`        | Mutability marker for bindings/refs |
| `self`      | `SelfLower`  | Instance receiver |
| `Self`      | `SelfUpper`  | Self type |
| `true`      | `True`       | Boolean literal |
| `false`     | `False`      | Boolean literal |
| `await`     | `Await`      | Long form (also emitted by `Y`) |
| `yield`     | `Yield`      | Iterator/coroutine yield |
| `const`     | `Const`      | Compile-time constant |
| `comptime`  | `Comptime`   | Compile-time expr/block |
| `dyn`       | `Dyn`        | Dynamic dispatch trait object |
| `macro`     | `Macro`      | Declarative macro declaration |
| `as`        | `As`         | Type cast |
| `pure`      | `Pure`       | Pure-function effect marker |
| `io`        | `Io`         | I/O effect marker |
| `effect`    | `Effect`     | Effect declaration (reserved; no grammar production yet) |
| `unsafe`    | `Unsafe`     | Unsafe block/function modifier |
| `partial`   | `Partial`    | Panic-permitting function modifier |
| `linear`    | `Linear`     | Linear type (must-use) |
| `affine`    | `Affine`     | Affine type (at-most-once) |
| `move`      | `Move`       | Closure move capture |
| `where`     | `Where`      | Generic bound clause |

## Primitive Type Keywords

`i8` `i16` `i32` `i64` `i128` `u8` `u16` `u32` `u64` `u128` `f32` `f64` `bool` `str` — all primitive type tokens, never used as identifiers.

## SIMD Vector Type Keywords

`Vec2f32` `Vec4f32` `Vec8f32` `Vec2f64` `Vec4f64` `Vec4i32` `Vec8i32` `Vec2i64` `Vec4i64` — codegen-gated (LLVM vector intrinsics).

## Contextual Keywords

These words are NOT reserved — the lexer emits them as `Token::Ident` and the parser reinterprets them only in specific positions:

- `alloc` — effect prefix when it occurs before `fn` / `A fn` / `pub fn`. Remains a valid identifier elsewhere (used in `std/allocator.vais` and `std/arena.vais` ~40 times as method/variable name).

---

## Removed Keywords (DO NOT RE-INTRODUCE)

| Keyword | Removed in | Commit | Reason |
|---------|-----------|--------|--------|
| `spawn` | Phase 195 | `12592076` | Replaced by runtime task APIs |
| `lazy`  | Phase 194 | `8c60c075` | Paired with `force`; replaced by `LazyCell`-style stdlib |
| `force` | Phase 194 | `8c60c075` | Paired with `lazy` |
| `F` `S` `E` `EN` `EL` `M` `R` `T` `U` `P` `W` `X` (single-char form) | Step 19 P4 (loop 25, 2026-05-08) | `2b485860` | LESSONS L-009 (codemod readability trap) + L-010 (token-efficiency hypothesis empirically false) |

Re-introducing any of these requires RFC + update to `docs/language/removed_keywords.md`.

---

## Ambiguity Rules

1. **`Y` vs `await`**: both emit `Token::Await`. Choose based on readability; compiler is indifferent.
2. **`C` is always Continue**: compile-time constants use the lowercase `const` keyword. Historical docs that claimed "C can also mean Const" are incorrect — the lexer has never produced a `Const` token from `C`.

---

## Invariant

For any identifier `x` that is NOT listed above:

- `tokenize("x")[0] == Token::Ident("x")` — ALWAYS.
- Parser never rewrites an `Ident` into a keyword (except for the single contextual case `alloc` documented above).

This invariant is tested in `crates/vaisc/tests/integrity/compiler_syntax.rs` §11-23 (parser coverage) and `crates/vais-lexer/tests/` (lexer-level unit tests).
