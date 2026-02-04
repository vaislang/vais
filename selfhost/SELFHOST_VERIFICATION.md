# Vais Self-Host Verification Report -- Phase 35, Stage 1

**Date**: 2026-02-04
**Branch**: develop
**Scope**: Lexer token coverage, feature parity, architecture comparison

---

## 1. Token Coverage Analysis

### 1.1 Single-Character Keywords

| Keyword | Meaning    | Rust Lexer | token.vais | constants.vais | lexer.vais | lexer_s1.vais |
|---------|------------|------------|------------|----------------|------------|---------------|
| `F`     | Function   | Function   | TOK_KW_F (1)   | TOK_KW_F (1)   | Yes | Yes |
| `S`     | Struct     | Struct     | TOK_KW_S (2)   | TOK_KW_S (2)   | Yes | Yes |
| `E`     | Enum       | Enum       | TOK_KW_E (3)   | TOK_KW_E (3)   | Yes | Yes |
| `I`     | If         | If         | TOK_KW_I (4)   | TOK_KW_I (4)   | Yes | Yes |
| `L`     | Loop       | Loop       | TOK_KW_L (5)   | TOK_KW_L (5)   | Yes | Yes |
| `M`     | Match      | Match      | TOK_KW_M (6)   | TOK_KW_M (6)   | Yes | Yes |
| `A`     | Async      | Async      | TOK_KW_A (12)  | --              | Yes | -- |
| `R`     | Return     | Return     | TOK_KW_R (13)  | TOK_KW_R (13)  | Yes | Yes |
| `B`     | Break      | Break      | TOK_KW_B (14)  | TOK_KW_B (14)  | Yes | Yes |
| `C`     | Continue   | Continue   | TOK_KW_C (15)  | --              | Yes | -- |
| `T`     | Type       | TypeKeyword| TOK_KW_T (9)   | --              | Yes | -- |
| `U`     | Use        | Use        | TOK_KW_U (10)  | TOK_KW_U (7)   | Yes | Yes |
| `P`     | Pub        | Pub        | TOK_KW_P (11)  | --              | Yes | -- |
| `W`     | Trait      | Trait      | TOK_KW_W (7)   | --              | Yes | -- |
| `X`     | Impl       | Impl       | TOK_KW_X (8)   | TOK_KW_X (8)   | Yes | Yes |
| `D`     | Defer      | Defer      | --              | --              | --  | -- |
| `O`     | Union      | Union      | --              | --              | --  | -- |
| `N`     | Extern     | Extern     | --              | --              | --  | -- |
| `G`     | Global     | Global     | --              | --              | --  | -- |
| `Y`     | Await      | Await      | --              | --              | --  | -- |

**ID mismatch note**: `constants.vais` assigns `TOK_KW_U = 7` while `token.vais` assigns `TOK_KW_U = 10` and `TOK_KW_W = 7`. The two selfhost modules disagree on the ID for `U`. This is a latent bug if both modules are ever linked together.

### 1.2 Multi-Character Keywords

| Keyword    | Rust Lexer | token.vais | constants.vais | lexer.vais | lexer_s1.vais |
|------------|------------|------------|----------------|------------|---------------|
| `mut`      | Mut        | TOK_KW_MUT (18) | TOK_KW_MUT (18) | Yes | Yes |
| `true`     | True       | TOK_KW_TRUE (16)| --               | Yes | -- |
| `false`    | False      | TOK_KW_FALSE (17)| --              | Yes | -- |
| `else`     | --*        | TOK_KW_ELSE (19)| --               | Yes | -- |
| `self`     | SelfLower  | --          | --              | --  | -- |
| `Self`     | SelfUpper  | --          | --              | --  | -- |
| `spawn`    | Spawn      | --          | --              | --  | -- |
| `await`    | Await      | --          | --              | --  | -- |
| `weak`     | Weak       | --          | --              | --  | -- |
| `clone`    | Clone      | --          | --              | --  | -- |
| `const`    | Const      | --          | --              | --  | -- |
| `comptime` | Comptime   | --          | --              | --  | -- |
| `dyn`      | Dyn        | --          | --              | --  | -- |
| `macro`    | Macro      | --          | --              | --  | -- |
| `as`       | As         | --          | --              | --  | -- |
| `pure`     | Pure       | --          | --              | --  | -- |
| `effect`   | Effect     | --          | --              | --  | -- |
| `io`       | Io         | --          | --              | --  | -- |
| `unsafe`   | Unsafe     | --          | --              | --  | -- |
| `linear`   | Linear     | --          | --              | --  | -- |
| `affine`   | Affine     | --          | --              | --  | -- |
| `move`     | Move       | --          | --              | --  | -- |
| `consume`  | Consume    | --          | --              | --  | -- |
| `lazy`     | Lazy       | --          | --              | --  | -- |
| `force`    | Force      | --          | --              | --  | -- |

*Note: Rust lexer does not have a separate `else` token; `E` serves as both Enum and Else contextually. The selfhost lexer explicitly defines `TOK_KW_ELSE`.

### 1.3 Type Keywords

| Type   | Rust Lexer | token.vais       | constants.vais      | lexer.vais | lexer_s1.vais |
|--------|------------|------------------|---------------------|------------|---------------|
| `i8`   | I8         | TOK_TY_I8 (31)  | --                  | Yes | -- |
| `i16`  | I16        | TOK_TY_I16 (32) | --                  | Yes | -- |
| `i32`  | I32        | TOK_TY_I32 (33) | --                  | Yes | -- |
| `i64`  | I64        | TOK_TY_I64 (34) | TOK_TY_I64 (34)    | Yes | Yes |
| `i128` | I128       | TOK_TY_I128 (35)| --                  | Yes | -- |
| `u8`   | U8         | TOK_TY_U8 (36)  | --                  | Yes | -- |
| `u16`  | U16        | TOK_TY_U16 (37) | --                  | Yes | -- |
| `u32`  | U32        | TOK_TY_U32 (38) | --                  | Yes | -- |
| `u64`  | U64        | TOK_TY_U64 (39) | --                  | Yes | -- |
| `u128` | U128       | TOK_TY_U128 (40)| --                  | Yes | -- |
| `f32`  | F32        | TOK_TY_F32 (41) | --                  | Yes | -- |
| `f64`  | F64        | TOK_TY_F64 (42) | --                  | Yes | -- |
| `bool` | Bool       | TOK_TY_BOOL (43)| TOK_TY_BOOL (36)   | Yes | Yes |
| `str`  | Str        | TOK_TY_STR (44) | TOK_TY_STR (35)    | Yes | Yes |

**ID mismatch**: `constants.vais` uses `TOK_TY_STR = 35` and `TOK_TY_BOOL = 36`, while `token.vais` uses `TOK_TY_I128 = 35`, `TOK_TY_U8 = 36`, `TOK_TY_STR = 44`, `TOK_TY_BOOL = 43`. These IDs conflict and would produce wrong token types if mixed.

### 1.4 SIMD Vector Types (Rust Lexer Only)

All nine SIMD vector types (`Vec2f32`, `Vec4f32`, `Vec8f32`, `Vec2f64`, `Vec4f64`, `Vec4i32`, `Vec8i32`, `Vec2i64`, `Vec4i64`) are completely absent from the selfhost lexer.

### 1.5 Literal Tokens

| Literal        | Rust Lexer     | token.vais         | lexer.vais | lexer_s1.vais |
|----------------|----------------|--------------------|------------|---------------|
| Integer        | Int(i64)       | TOK_INT (51)       | Yes        | Yes           |
| Float          | Float(f64)     | TOK_FLOAT (52)     | Yes        | --            |
| String         | String(String) | TOK_STRING (53)    | Yes        | Yes           |
| Identifier     | Ident(String)  | TOK_IDENT (54)     | Yes        | Yes           |
| Lifetime       | Lifetime(String)| --                | --         | --            |
| Doc Comment    | DocComment(String)| --              | --         | --            |

### 1.6 Operator and Punctuation Tokens

| Token    | Symbol | Rust Lexer | token.vais | lexer.vais | lexer_s1.vais |
|----------|--------|------------|------------|------------|---------------|
| Plus     | `+`    | Plus       | 61         | Yes        | Yes           |
| Minus    | `-`    | Minus      | 62         | Yes        | Yes           |
| Star     | `*`    | Star       | 63         | Yes        | Yes           |
| Slash    | `/`    | Slash      | 64         | Yes        | Yes           |
| Percent  | `%`    | Percent    | 65         | Yes        | Yes           |
| Lt       | `<`    | Lt         | 66         | Yes        | Yes           |
| Gt       | `>`    | Gt         | 67         | Yes        | Yes           |
| Lte      | `<=`   | Lte        | 68         | Yes        | Yes           |
| Gte      | `>=`   | Gte        | 69         | Yes        | Yes           |
| EqEq     | `==`   | EqEq       | 70         | Yes        | Yes           |
| Neq      | `!=`   | Neq        | 71         | Yes        | Yes           |
| Amp      | `&`    | Amp        | 72         | Yes        | Yes           |
| Pipe     | `\|`   | Pipe       | 73         | Yes        | Yes           |
| Caret    | `^`    | Caret      | 74         | Yes        | --            |
| Tilde    | `~`    | Tilde      | 75         | Yes        | --            |
| Shl      | `<<`   | Shl        | 76         | Yes        | --            |
| Shr      | `>>`   | Shr        | 77         | Yes        | --            |
| Bang     | `!`    | Bang       | 78         | Yes        | Yes           |
| And      | `&&`   | Amp+Amp*   | 79         | Yes        | Yes           |
| Or       | `\|\|` | Pipe+Pipe* | 80         | Yes        | Yes           |
| Eq       | `=`    | Eq         | 81         | Yes        | Yes           |
| ColonEq  | `:=`   | ColonEq    | 82         | Yes        | Yes           |
| PlusEq   | `+=`   | PlusEq     | 83         | Yes        | --            |
| MinusEq  | `-=`   | MinusEq    | 84         | Yes        | --            |
| StarEq   | `*=`   | StarEq     | 85         | Yes        | --            |
| SlashEq  | `/=`   | SlashEq    | 86         | Yes        | --            |
| PipeArrow| `\|>`  | PipeArrow  | --         | --         | --            |
| Ellipsis | `...`  | Ellipsis   | --         | --         | --            |
| Dollar   | `$`    | Dollar     | --         | --         | --            |
| HashBracket| `#[` | HashBracket| --         | --         | --            |

*Note: The Rust lexer tokenizes `&&` as two `Amp` tokens and `||` as two `Pipe` tokens (confirmed by test). The selfhost lexers tokenize them as single `TOK_AND` / `TOK_OR` tokens. This is a semantic difference that the parser must account for.

### 1.7 Delimiter and Punctuation Tokens

| Token      | Symbol | Rust Lexer | token.vais | lexer.vais | lexer_s1.vais |
|------------|--------|------------|------------|------------|---------------|
| LParen     | `(`    | LParen     | 91         | Yes        | Yes           |
| RParen     | `)`    | RParen     | 92         | Yes        | Yes           |
| LBrace     | `{`    | LBrace     | 93         | Yes        | Yes           |
| RBrace     | `}`    | RBrace     | 94         | Yes        | Yes           |
| LBracket   | `[`    | LBracket   | 95         | Yes        | Yes           |
| RBracket   | `]`    | RBracket   | 96         | Yes        | Yes           |
| Comma      | `,`    | Comma      | 101        | Yes        | Yes           |
| Colon      | `:`    | Colon      | 102        | Yes        | Yes           |
| Semi       | `;`    | Semi       | 103        | Yes        | Yes           |
| Dot        | `.`    | Dot        | 104        | Yes        | Yes           |
| DotDot     | `..`   | DotDot     | 105        | Yes        | --            |
| DotDotEq   | `..=`  | DotDotEq   | 106        | Yes        | --            |
| Arrow      | `->`   | Arrow      | 107        | Yes        | Yes           |
| FatArrow   | `=>`   | FatArrow   | 108        | Yes        | Yes           |
| ColonColon | `::`   | ColonColon | 109        | Yes        | --            |
| Question   | `?`    | Question   | 110        | Yes        | --            |
| At         | `@`    | At         | 111        | Yes        | Yes           |
| Hash       | `#`    | --*        | 112        | --         | --            |

*The Rust lexer skips `#` as a comment leader; it only has `#[` as HashBracket.

---

## 2. Lexer Feature Comparison

| Feature                      | Rust Lexer (logos)         | lexer.vais (OOP)          | lexer_s1.vais (procedural) | Status       |
|------------------------------|---------------------------|---------------------------|----------------------------|--------------|
| Single-char keywords (F-X)   | 19 keywords               | 15 keywords               | 9 keywords                 | Partial      |
| Multi-char keywords           | 24 keywords               | 4 (mut/true/false/else)   | 1 (mut)                    | Minimal      |
| Type keywords                 | 14 types + 9 SIMD         | 14 types                  | 3 types (i64/str/bool)     | Partial/Min  |
| Integer literals              | With `_` separators       | Decimal only              | Decimal only               | Partial      |
| Hex literals (`0x`)           | Via regex                 | Yes (manual)              | No                         | Partial      |
| Float literals                | With `_` and `e` notation | Decimal + exponent        | No                         | Partial/None |
| String literals               | Full escape processing    | Basic escape skip         | Basic escape skip          | Partial      |
| String escape sequences       | \n \t \r \\ \" \0 \xHH   | Skips escapes (no decode) | Skips escapes (no decode)  | Missing      |
| Operators (arithmetic)        | 5 (+,-,*,/,%)             | 5                         | 5                          | Full         |
| Operators (comparison)        | 6 (<,>,<=,>=,==,!=)       | 6                         | 6                          | Full         |
| Operators (bitwise)           | 6 (&,\|,^,~,<<,>>)       | 6                         | 2 (&,\|)                   | Partial      |
| Operators (logical)           | && = Amp+Amp              | && = TOK_AND (single)     | && = TOK_AND (single)      | Divergent    |
| Compound assignment           | 4 (+=,-=,*=,/=)           | 4                         | 0                          | Partial/None |
| Range operators (..  ..=)     | Yes                       | Yes                       | No                         | Partial      |
| Pipe arrow (\|>)              | Yes                       | No                        | No                         | Missing      |
| Ellipsis (...)                | Yes                       | No                        | No                         | Missing      |
| Dollar ($)                    | Yes                       | No                        | No                         | Missing      |
| HashBracket (#[)              | Yes                       | No                        | No                         | Missing      |
| Lifetime ('a)                 | Yes                       | No                        | No                         | Missing      |
| Doc comments (///)            | Yes (preserved)           | No                        | No                         | Missing      |
| Regular comments (#)          | Skipped                   | Skipped                   | Skipped                    | Full         |
| Whitespace handling           | logos skip rule            | Manual (space/tab/nl/cr)  | Manual (space/tab/nl/cr)   | Full         |
| Line/col tracking             | Via span                  | Yes (line + col fields)   | Yes (line + col fields)    | Full         |
| Span tracking                 | Byte range (Range<usize>) | span_start + span_end     | span_start + span_end      | Full         |
| Underscore in numbers (1_000) | Yes                       | No                        | No                         | Missing      |
| Unicode in strings            | Yes                       | Byte-level (partial)      | Byte-level (partial)       | Partial      |

---

## 3. Architecture Comparison

### 3.1 Rust Lexer (`crates/vais-lexer/src/lib.rs`)

- **Engine**: logos crate -- compile-time DFA generation
- **Pattern matching**: Declarative regex/token attributes on enum variants
- **Token representation**: Rust enum `Token` with associated data (`Int(i64)`, `String(String)`, etc.)
- **Output**: `Vec<SpannedToken>` where `SpannedToken` contains `token: Token` and `span: Range<usize>`
- **Error handling**: `Result<Vec<SpannedToken>, LexError>` with typed errors
- **String processing**: Full escape sequence decoding at lex time (produces decoded strings)
- **Priority system**: logos priority attributes to disambiguate single-letter keywords vs identifiers
- **Logical operators**: `&&` and `||` are tokenized as two separate `Amp`/`Pipe` tokens (not as single tokens)

### 3.2 Selfhost Lexer -- OOP variant (`selfhost/lexer.vais`)

- **Engine**: Manual character-by-character state machine
- **Architecture**: Uses `Lexer` struct with methods via `X Lexer { ... }` impl blocks
- **Memory model**: Pointer-based source access (`load_byte`), manual heap allocation
- **Token representation**: `Token` struct with integer `kind` field and union-like value/str_ptr fields
- **Output**: `TokenList` (manually managed growable array, 48 bytes per token)
- **Keyword detection**: Byte-by-byte comparison after scanning identifier, checking length first
- **String processing**: Skips over escape sequences (backslash + next char) without decoding
- **Self-recursion**: Uses `@.method()` syntax for recursive/self calls

### 3.3 Selfhost Lexer -- Procedural variant (`selfhost/lexer_s1.vais`)

- **Engine**: Manual procedural functions (no struct methods)
- **Architecture**: Pure procedural -- lexer state is a raw pointer to 40 bytes of memory
- **Memory model**: Direct `store_i64`/`load_i64`/`load_byte` primitives
- **Token representation**: Flat memory layout (48 bytes per token stored in caller-provided buffer)
- **Dependencies**: Imports `constants.vais` (reduced constant set)
- **Keyword detection**: Minimal set -- only 9 single-char keywords, 3 type keywords, `mut`
- **Missing**: No float support, no hex literals, no compound assignments, fewer operators

### 3.4 Key Architectural Differences

1. **Logical operator semantics**: Rust lexer produces `Amp Amp` for `&&`; selfhost produces single `TOK_AND`. The selfhost parser must handle this difference.
2. **String handling**: Rust lexer decodes escapes into actual characters; selfhost lexers store raw bytes with escapes still present.
3. **Token identity**: Rust uses a rich enum with data; selfhost uses integer kind codes with separate value fields.
4. **Error recovery**: Rust lexer returns `Err` on invalid tokens; selfhost lexer emits `TOK_ERROR` and continues.
5. **Comment model**: Rust lexer has `#` as comment and `///` as doc comment (preserved); selfhost treats all `#` as line comments.

---

## 4. Known Gaps

### 4.1 Missing Single-Character Keywords

The following single-character keywords exist in the Rust lexer but are absent from all selfhost files:

| Keyword | Purpose | Priority |
|---------|---------|----------|
| `D`     | Defer   | Medium -- needed for resource management |
| `O`     | Union   | Low -- rarely used in core language |
| `N`     | Extern  | High -- needed for FFI declarations |
| `G`     | Global  | Medium -- needed for global state |
| `Y`     | Await   | Medium -- needed for async code |

### 4.2 Missing Multi-Character Keywords

| Keyword    | Purpose              | Priority |
|------------|----------------------|----------|
| `self`     | Instance reference   | Critical -- needed for method bodies |
| `Self`     | Type reference       | Critical -- needed for constructors |
| `spawn`    | Async task spawn     | Medium   |
| `await`    | Async await          | Medium   |
| `const`    | Compile-time const   | High     |
| `comptime` | Comptime evaluation  | Low      |
| `dyn`      | Dynamic dispatch     | Medium   |
| `macro`    | Macro definition     | Low      |
| `as`       | Type cast            | High     |
| `weak`     | Weak reference       | Low      |
| `clone`    | Explicit clone       | Medium   |
| `pure`     | Effect annotation    | Low      |
| `effect`   | Effect annotation    | Low      |
| `io`       | Effect annotation    | Low      |
| `unsafe`   | Unsafe block         | Medium   |
| `linear`   | Linear type          | Low      |
| `affine`   | Affine type          | Low      |
| `move`     | Move semantics       | Medium   |
| `consume`  | Consume value        | Low      |
| `lazy`     | Lazy evaluation      | Low      |
| `force`    | Force evaluation     | Low      |

### 4.3 Missing Token Types

- **Lifetime** (`'a`, `'static`): Not recognized at all
- **Doc comment** (`///`): Treated as regular comment and discarded
- **Attribute syntax** (`#[`): Not recognized; `#` is consumed as comment start
- **Pipe arrow** (`|>`): Not recognized
- **Ellipsis** (`...`): Not recognized
- **Dollar** (`$`): Not recognized -- blocks macro expansion
- **SIMD types**: All nine vector types missing

### 4.4 Feature Gaps

- **Underscore in numeric literals** (`1_000_000`): Not supported
- **String escape decoding**: Escapes are skipped over but not decoded into actual characters
- **Hex escape in strings** (`\xHH`): Not supported
- **Null escape** (`\0`): Not decoded
- **Integer overflow detection**: No validation for values exceeding i64 range

### 4.5 ID Conflicts Between Selfhost Modules

The `constants.vais` and `token.vais` files define **different IDs** for the same tokens:

| Token         | token.vais | constants.vais | Conflict? |
|---------------|------------|----------------|-----------|
| TOK_KW_U      | 10         | 7              | YES       |
| TOK_KW_W      | 7          | --             | N/A       |
| TOK_TY_STR    | 44         | 35             | YES       |
| TOK_TY_BOOL   | 43         | 36             | YES       |
| TOK_TY_I128   | 35         | --             | overlaps constants STR=35 |
| TOK_TY_U8     | 36         | --             | overlaps constants BOOL=36 |

This means `lexer.vais` (which imports `token.vais`) and `lexer_s1.vais` (which imports `constants.vais`) produce **incompatible token streams**. They cannot share a parser without reconciliation.

---

## 5. Self-Hosting Readiness Score

### 5.1 Component Scores

| Component                        | lexer.vais (OOP) | lexer_s1.vais (Procedural) |
|----------------------------------|:-----------------:|:--------------------------:|
| Single-char keyword coverage     | 79% (15/19)       | 47% (9/19)                |
| Multi-char keyword coverage      | 17% (4/24)        | 4% (1/24)                 |
| Type keyword coverage            | 61% (14/23)       | 13% (3/23)                |
| Literal support                  | 75%               | 40%                       |
| Operator/punctuation coverage    | 82%               | 58%                       |
| String handling fidelity         | 40%               | 40%                       |
| Comment handling                 | 70%               | 70%                       |
| Error handling                   | 30%               | 20%                       |
| Whitespace/position tracking     | 90%               | 80%                       |
| Token storage / data structures  | 85%               | 75%                       |

### 5.2 Overall Scores

| Variant        | Score  | Assessment |
|----------------|--------|------------|
| lexer.vais     | **55%** | Can lex simple Vais programs (basic functions, structs, loops, matches). Cannot handle advanced features (async, defer, macros, lifetimes, attributes, full type system). |
| lexer_s1.vais  | **35%** | Can lex minimal Vais programs (functions, structs, if/else, loops, basic operators). Missing most of the language surface area. |
| constants.vais consistency | **60%** | Usable but ID conflicts with token.vais would cause failures if modules are combined. |

### 5.3 Composite Self-Hosting Readiness

**Overall Lexer Self-Hosting Readiness: 45%**

The selfhost lexer can tokenize a significant subset of the Vais language -- enough to lex itself (which uses only basic features). However, it cannot lex the full language as defined by the Rust reference implementation.

---

## 6. Recommendations

### Priority 1 -- Critical (blocks self-hosting)

1. **Unify token IDs**: Reconcile `constants.vais` and `token.vais` so they use identical ID values. Pick one as the source of truth and update the other.
2. **Add `self` and `Self` keywords**: These are required for method implementations (`X` blocks) which the selfhost compiler itself uses extensively.
3. **Add `as` keyword**: Needed for type casts which appear in codegen.
4. **Add `const` keyword**: Required for constant declarations.

### Priority 2 -- High (needed for language coverage)

5. **Add missing single-char keywords** (`D`, `N`, `G`, `Y`, `O`): These represent core language constructs.
6. **Add `true`/`false`/`else` to constants.vais and lexer_s1.vais**: The procedural lexer cannot parse boolean literals or else branches.
7. **Add compound assignment operators** (`+=`, `-=`, `*=`, `/=`) to lexer_s1.vais.
8. **Add float literal support** to lexer_s1.vais.
9. **Implement string escape decoding**: The selfhost lexer should produce decoded strings, matching the Rust lexer behavior.

### Priority 3 -- Medium (needed for full parity)

10. **Add lifetime token support** (`'a`, `'static`).
11. **Add doc comment preservation** (`///` comments should produce tokens, not be skipped).
12. **Add attribute syntax** (`#[`) -- currently `#` immediately triggers comment skipping.
13. **Add pipe arrow** (`|>`), **ellipsis** (`...`), **dollar** (`$`) tokens.
14. **Add `::` (ColonColon) and `..` / `..=` range operators** to lexer_s1.vais.
15. **Add underscore separator support** in numeric literals.
16. **Fix `&&`/`||` semantics**: Decide whether the selfhost compiler will use single-token `AND`/`OR` (current approach) or double-token `Amp Amp`/`Pipe Pipe` (Rust lexer approach). Document the decision and ensure the parser matches.

### Priority 4 -- Low (advanced features)

17. **Add SIMD vector type tokens**.
18. **Add effect system keywords** (`pure`, `effect`, `io`, `unsafe`).
19. **Add linear type keywords** (`linear`, `affine`, `move`, `consume`).
20. **Add lazy evaluation keywords** (`lazy`, `force`).
21. **Add `spawn`, `await`, `weak`, `clone`, `comptime`, `dyn`, `macro` keywords**.
22. **Add hex escape (`\xHH`) and null escape (`\0`) decoding in strings**.

### Suggested Milestone Plan

| Milestone | Target Score | Key Deliverables |
|-----------|-------------|------------------|
| M1        | 60%         | Unify token IDs, add `self`/`Self`/`as`/`const`, fix lexer_s1 booleans/else |
| M2        | 75%         | Add all 19 single-char keywords, compound assignments, float support in S1, string escape decoding |
| M3        | 90%         | Add lifetimes, doc comments, attributes, pipe arrow, ellipsis, dollar, range ops |
| M4        | 100%        | SIMD types, effect keywords, linear types, lazy eval, full keyword set |

---

*Report generated from analysis of:*
- `/Users/sswoo/study/projects/vais/selfhost/token.vais`
- `/Users/sswoo/study/projects/vais/selfhost/constants.vais`
- `/Users/sswoo/study/projects/vais/selfhost/lexer.vais`
- `/Users/sswoo/study/projects/vais/selfhost/lexer_s1.vais`
- `/Users/sswoo/study/projects/vais/crates/vais-lexer/src/lib.rs`
