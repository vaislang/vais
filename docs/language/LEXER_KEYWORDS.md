# Vais Lexer Keyword Registry (Phase 1.7)

Single source of truth for **every reserved token** the Vais lexer emits as a keyword. Any identifier that is NOT in this list is passed through as `Token::Ident` and never interpreted as a keyword. Adding/removing/renaming keywords requires:

1. Edit `crates/vais-lexer/src/lib.rs`.
2. Update this file.
3. Update `docs/LANGUAGE_SPEC.md` "Keywords" section.
4. If removing: add entry to `docs/language/removed_keywords.md` and bump Phase note.
5. Run `./scripts/check-integrity.sh` — must stay green.

Source of truth: `crates/vais-lexer/src/lib.rs` (logos derive attributes).
Cross-reference: `docs/LANGUAGE_SPEC.md` "Keywords" + "Construct Status Matrix".

---

## Single-letter Declaration / Statement Keywords (priority 3)

| Token | Lexer variant | Grammar role | Phase added |
|-------|---------------|--------------|-------------|
| `F`   | `Function`      | Function declaration | 0.0.1 |
| `S`   | `Struct`        | Struct declaration | 0.0.1 |
| `E`   | `Enum`          | Enum decl (context-dependent, legacy) | 0.0.1 — `EN`/`EL` preferred |
| `I`   | `If`            | If-expression | 0.0.1 |
| `L`   | `Loop`          | Infinite loop | 0.0.1 |
| `M`   | `Match`         | Match expression | 0.0.1 |
| `R`   | `Return`        | Early return | 0.0.1 |
| `B`   | `Break`         | Loop break | 0.0.1 |
| `C`   | `Continue`      | Loop continue **(never Const)** | 0.0.1 |
| `T`   | `TypeKeyword`   | Type alias / trait alias | 0.0.1 |
| `U`   | `Use`           | Import | 0.0.1 |
| `P`   | `Pub`           | Public visibility | 0.0.1 |
| `W`   | `Trait`         | Trait (interface) declaration | 0.0.1 |
| `X`   | `Impl`          | Impl block (methods / trait impl) | 0.0.1 |
| `D`   | `Defer`         | Defer block | 0.0.1 |
| `O`   | `Union`         | C-style untagged union | 0.0.1 |
| `N`   | `Extern`        | Extern "C" block | 0.0.1 |
| `G`   | `Global`        | Global variable | 0.0.1 |
| `A`   | `Async`         | Async modifier | 0.0.1 |
| `Y`   | `Await`         | Postfix-await shortcut | Phase 29 |

## Two-letter Unambiguous Keywords (priority 4 > single-letter priority 3)

| Token | Lexer variant | Grammar role |
|-------|---------------|--------------|
| `EN`  | `EnumKeyword`   | Unambiguous enum declaration |
| `EL`  | `Else`          | Else branch (`I … { … } EL { … }`) |
| `LF`  | `ForEach`       | For-each loop (`LF i: range`) |
| `LW`  | `While`         | While loop (`LW cond { … }`) |

## Multi-letter Word Keywords

| Token | Lexer variant | Role |
|-------|---------------|------|
| `mut`       | `Mut`        | Mutability marker for bindings/refs |
| `self`      | `SelfLower`  | Instance receiver |
| `Self`      | `SelfUpper`  | Self type |
| `true`      | `True`       | Boolean literal |
| `false`     | `False`      | Boolean literal |
| `await`     | `Await`      | Long form of `Y` (both emit same token) |
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

- `alloc` — effect prefix when it occurs before `F`/`A F`/`P F`. Remains a valid identifier elsewhere (used in `std/allocator.vais` and `std/arena.vais` ~40 times as method/variable name).

---

## Removed Keywords (DO NOT RE-INTRODUCE)

| Keyword | Removed in | Commit | Reason |
|---------|-----------|--------|--------|
| `spawn` | Phase 195 | `12592076` | Replaced by runtime task APIs |
| `lazy`  | Phase 194 | `8c60c075` | Paired with `force`; replaced by `LazyCell`-style stdlib |
| `force` | Phase 194 | `8c60c075` | Paired with `lazy` |

Re-introducing requires RFC + update to `docs/language/removed_keywords.md`.

---

## Ambiguity Rules

1. **`E` vs `EN`/`EL`**: `E` is a single-letter, context-dependent token. The parser accepts `E` as both enum-head (`E Color { Red, Green }`) and else-tail (`I c { a } E { b }`). New code MUST prefer `EN` and `EL` (higher lexer priority: 4 > 3, eliminates ambiguity).
2. **`C` is always Continue**: compile-time constants use the lowercase `const` keyword. Historical docs that claimed "C can also mean Const" are incorrect — the lexer has never produced a `Const` token from `C`.
3. **`Y` vs `await`**: both emit `Token::Await`. Choose based on readability; compiler is indifferent.

---

## Invariant

For any identifier `x` that is NOT listed above:

- `tokenize("x")[0] == Token::Ident("x")` — ALWAYS.
- Parser never rewrites an `Ident` into a keyword (except for the single contextual case `alloc` documented above).

This invariant is tested in `crates/vaisc/tests/integrity/compiler_syntax.rs` §11-23 (parser coverage) and `crates/vais-lexer/tests/` (lexer-level unit tests).
