# Step 15 (I-4 dual syntax) findings

This file records empirical findings during Order step 15 (I-4 dual
syntax). Mirrors STEP7 / STEP10 / STEP11 / STEP13 / STEP17 structure.

Master Plan v25 §Step 15 was BLOCKED (2026-05-03) after a naive lexer
change to add multi-character keywords (`fn` / `struct` / `match` / ...)
broke INTEGRITY due to identifier-keyword collisions in
`compiler/std/`, `lang/packages/vaisdb/`, and `vais-server`. The status
text recommended a `vaisc fmt --rename-keyword-collisions` codemod as
the prerequisite.

## Index (F-15-NN → 한 줄 요약)

| ID | 한 줄 요약 |
|---|---|
| F-15-01 | `vaisc fmt --rename-keyword-collisions` skeleton LANDED + 13-keyword baseline scan: 106 collisions total (2026-05-05) |
| F-15-02 | Stage 1 first wave LANDED: 5 zero-collision multi-char keywords as Logos token aliases (2026-05-05) |
| F-15-03 | Stage 1 second wave first sub-batch LANDED: else / match / return aliases + bare keyword shadow gate (2026-05-05) |
| F-15-04 | Stage 1 second wave second sub-batch LANDED (use / type) + fn deferred + mod out-of-scope (2026-05-06) |
| F-15-05 | fn alias LANDED via single-iter parser refactor (refute F-15-04 multi-iter prediction) (2026-05-06) |
| F-15-06 | Stage 3 round-trip codemod LANDED (`vaisc fmt --to=multi|single`) + 266/266 selfhost-roundtrip gate (2026-05-06) |

## Findings

### F-15-01 — Codemod skeleton + baseline scan (2026-05-05)

Stage 0a (skeleton) LANDED. New module
`compiler/crates/vaisc/src/commands/fmt_rename.rs`:

- `STEP15_CANDIDATE_KEYWORDS` const array (13 entries, alphabetically sorted per L-007).
- `is_collision(ident, kw)` predicate: true iff identifier starts with
  `kw` followed by `_` or ASCII alpha (mirrors Logos longest-match).
- `scan_module(module, keywords)` AST visitor that checks function
  names, parameters, struct names + fields, enum names + variants,
  trait names, and type aliases.
- `cmd_fmt_rename_keywords(input, options)` CLI entry. Stage 0 is
  dry-run only; reports findings without modifying files.
- 2 unit tests pass (`is_collision_basic`, `candidate_list_sorted`).

CLI wiring: `vaisc fmt --rename-keyword-collisions [--keyword=NAME]
[--dry-run] <input>`. The flag short-circuits the regular `cmd_fmt`.

Stage 0b (baseline measurement) LANDED. Per-keyword collision counts
across `compiler/std/`, `lang/packages/`, and
`docs/language/LIVING_SPEC/` (`.vais` files):

| keyword | collisions |
|---|---|
| const | 1 |
| else | 1 |
| enum | 0 |
| fn | 9 |
| impl | 0 |
| match | 2 |
| mod | 24 |
| pub | 0 |
| return | 3 |
| struct | 0 |
| trait | 0 |
| type | 10 |
| use | 56 |
| **total** | **106** |

This is **far lower than the prior estimate** (raw `grep` from the
2026-05-03 BLOCKED status note suggested >250 collisions across `match`
+ `fn` alone). The discrepancy is a combination of:

1. **AST-level vs raw grep**: the codemod only flags identifiers used
   in declaration positions (function names / parameters / struct
   fields / enum variants / etc.). Raw `grep` would also count
   occurrences inside expressions, comments, and string literals,
   inflating the number even though those sites do not produce a
   Logos collision.
2. **Logos longest-match boundary**: the predicate is `starts with kw
   then `_` or alpha`. A bare `fn` (exactly the keyword) is *not* a
   collision; only `fn_handler`, `fnHandler`, etc. trigger.

Five keywords have **zero** collisions: `enum`, `impl`, `pub`,
`struct`, `trait`. These five could be added to the lexer in
isolation today without any rename codemod.

### Stage 1+ scope (deferred)

Stage 0 is scan-only. Stage 1 needs:

- Apply rename: `<colliding>` → `_<colliding>` (or user-chosen prefix).
- Cross-module reference update: an identifier renamed in
  `module_a.vais` must propagate to every `use module_a::<colliding>`
  site and every call site.
- Stage 2: lexer change adds the multi-char keyword (now safe because
  baseline is collision-free).
- Stage 3: vaisc `fmt --to=multi` and `--to=single` for round-trip.

The 5 zero-collision keywords (`enum`, `impl`, `pub`, `struct`,
`trait`) are the natural first wave for stage 2 — they can land
without any stage 1 rename work. The remaining 8 (`const`, `else`,
`fn`, `match`, `mod`, `return`, `type`, `use`) need stage 1 rename
first.

Status: Step 15 BLOCKED → IN_PROGRESS (master-plan v25 → v26). Stage
0 (skeleton + baseline) LANDED. Stage 1+ tracked as multi-iter.

### F-15-02 — Stage 1 first wave LANDED (2026-05-05)

5 zero-collision multi-char keywords added to lexer as Logos token
aliases. Both syntaxes work:

```vais
struct Point { x: i64, y: i64 }    # multi-char form
trait Display { F show(self) -> i64 }
impl Point: Display { F show(self) -> i64 { self.x + self.y } }
enum Color { Red, Green, Blue }
pub F main() -> i64 { Point { x: 3, y: 4 }.show() }
```

```vais
S Point { x: i64, y: i64 }         # single-char form
W Display { F show(self) -> i64 }
X Point: Display { F show(self) -> i64 { self.x + self.y } }
E Color { Red, Green, Blue }
P F main() -> i64 { Point { x: 3, y: 4 }.show() }
```

Both probes type-check with exit 0, OK No errors found.

Lexer changes (crates/vais-lexer/src/lib.rs):

```rust
#[token("S", priority = 3)]
#[token("struct", priority = 3)]
Struct,

#[token("E", priority = 3)]
#[token("enum", priority = 3)]
Enum,

#[token("P", priority = 3)]
#[token("pub", priority = 3)]
Pub,

#[token("W", priority = 3)]
#[token("trait", priority = 3)]
Trait,

#[token("X", priority = 3)]
#[token("impl", priority = 3)]
Impl,
```

Logos derive macro produces a single matcher that accepts either
literal and emits the same Token variant. No parser change needed —
parser sees Token::Struct (etc.) and operates identically regardless
of source spelling.

INTEGRITY OK preserved (no baseline regression). The F-15-01
prediction (zero AST-level collisions across baseline) was validated
empirically by INTEGRITY survival.

### Stage 1 second wave (deferred — needs rename codemod first)

The remaining 8 collision keywords cannot be added the same way:
- use 56 collisions
- mod 24
- type 10
- fn 9
- return 3
- match 2
- const 1
- else 1

Stage 1.2 = `vaisc fmt --rename-keyword-collisions` apply mode (rewrite
identifiers `<colliding>` → `_<colliding>`). Stage 1.3 = lexer adds the
remaining 8 keywords (now safe). Multi-iter.

Status: Step 15 stage 1 first wave LANDED. master-plan v26 → v27.
Stage 1.2+ remains IN_PROGRESS.

### F-15-03 — Stage 1 second wave first sub-batch LANDED (2026-05-05)

3 multi-char keywords added as Logos token aliases on top of v27's
5-keyword first wave:

```rust
#[token("EL", priority = 4)]
#[token("else", priority = 4)]
Else,

#[token("M", priority = 3)]
#[token("match", priority = 3)]
Match,

#[token("R", priority = 3)]
#[token("return", priority = 3)]
Return,
```

`const` itself was already a lexer keyword from before Step 15. The
LIVING_SPEC `const_*` collisions in
`docs/language/LIVING_SPEC/01_keywords/comptime_function_body.vais`
(6 sites: 3 function names + 3 call sites of `const_thirtytwo` /
`const_sum` / `const_branch`) were renamed to `_const_*` so the file
no longer trips Logos longest-match. No lexer change needed.

Per-keyword cost (manual identifier renames before lexer alias):
- else: 0 renames. `else_result` and `else_expr` are use-site
  identifiers that Logos longest-match resolves as `Token::Ident`
  even after `else` becomes a keyword (verified: integrity green
  immediately after adding `#[token("else", priority = 4)]`).
- match: 3 renames in `compiler/std/url.vais` (line 321/325/330,
  bare `match := mut 1`, `match = 0`, `I match == 1` → `_match`).
  Without the rename, `cargo build` succeeded but `vaisc check
  std/url.vais` failed `error[P001] Unexpected token ColonEq`
  because the bare identifier exactly equals the new keyword and
  is no longer an Ident — Logos picks the keyword variant.
- return: 0 renames. Baseline grep `\breturn\b\s*(:=|=)` returns
  zero across compiler/std + lang/packages + LIVING_SPEC.

INTEGRITY OK preserved post-batch (std=82/82, vaisdb=261/261,
runtime smokes all green).

#### Empirical lesson — bare keyword shadow gate

The AST-level F-15-01 baseline (`is_collision` predicate counting
declaration-position only) is correct for `else_result`-style
**use-site prefix collisions**: Logos longest-match handles them
because the identifier is *longer* than the keyword. But that gate
**misses bare keyword shadows** — identifiers that equal the keyword
exactly, in any position. Example: `match := mut 1` in url.vais.

Wave 2 protocol must therefore add a complementary raw-grep gate:

```bash
grep -rEn '\b<kw>\b\s*(:=|=)' \
    compiler/std/ \
    lang/packages/ \
    docs/language/LIVING_SPEC/ \
    --include='*.vais'
```

If the grep hits any line, that bare identifier needs to be renamed
to `_<kw>` before the lexer alias is added. Otherwise `vaisc check`
on that file fails with P001 Unexpected token at the `:=` position.

The full Wave 2 protocol is now:
1. AST-level codemod gate (F-15-01) — counts declaration-position
   collisions; gives a ceiling on AST-aware rename work.
2. **Bare keyword shadow gate (F-15-03)** — `\b<kw>\b\s*(:=|=)` raw
   grep; catches identifiers that equal the keyword exactly.
3. Lexer alias addition.
4. INTEGRITY measurement; revert per CLAUDE rule 4 if any decrease.

#### Stage 1 second wave remainder (deferred)

The remaining 4 collision keywords still need stage 1 rename:
- fn 9 (declaration-position) + raw-grep TBD
- mod 24 (declaration-position) + raw-grep TBD
- type 10 (declaration-position) + raw-grep TBD
- use 56 (declaration-position) + raw-grep TBD — largest surface

Status: Step 15 stage 1 second wave first sub-batch LANDED.
master-plan v27 → v28. Stage 1 remainder + stage 3 (vaisc fmt
--to= flags + selfhost migration) remain IN_PROGRESS.

### F-15-04 — Stage 1 second wave second sub-batch LANDED (2026-05-06)

2 more multi-char keyword aliases on top of v28's batch:

```rust
#[token("U", priority = 3)]
#[token("use", priority = 3)]
Use,

#[token("T", priority = 3)]
#[token("type", priority = 3)]
TypeKeyword,
```

Total dual-syntax keywords now 10/13.

Per-keyword cost:
- use: 0 manual renames. Raw grep `\buse\b\s*(:=|=|:)` returned 0
  hits (no bare-keyword shadow). The 56 `use_*` use-site
  identifiers are all longer than `use`, so Logos longest-match
  resolves them as `Token::Ident` — verified empirically by the
  green INTEGRITY measurement immediately after the alias add.
- type: 0 manual renames. Raw grep `\btype\b\s*(:=|=)` returned
  only string-literal / comment hits (e.g. `"...sqlite_master
  WHERE type='table'..."`, `# type=END`, `"NestedLoopJoin
  type="`), none of which are bare identifiers. AST-level
  collisions were 10 in F-15-01 — all `type_*` longer-form
  identifiers handled by longest-match.

INTEGRITY OK preserved post-batch.

#### Scope decision: fn alias deferred (multi-iter)

Adding `#[token("fn", priority = 3)]` to `Token::Function`
(alongside `"F"`) breaks 5+ negative parser fixtures plus core
certification (16/16 → 15/16):

- `syntax_type_fn_pointer` (positive, line 1180): `F takes_fn(f:
  fn(i64) -> i64) -> i64 { f(0) }` — expects parse OK. Fails when
  `fn` is a Token::Function because the function-pointer type
  syntax `fn(i64) -> i64` is currently parsed via the Token::Ident
  path with a parser-level special-case for the literal text `fn`.
  Promoting `fn` to a real keyword changes which parser branch
  fires.
- `syntax_neg_mod_missing_fn_keyword` (line 655): `P main() ...`
  expected NOT to parse without `F`. With `fn` as a keyword the
  parser may accept it on a different path.
- `syntax_neg_trait_missing_fn_keyword` (line 1679), `syntax_
  closure_in_variable` (line 1879), `syntax_extra_fn_as_arg`
  (line 2257) — same family.

Resolution requires a parser refactor that distinguishes:
1. `fn` as function-declaration head (multi-char alias of `F`).
2. `fn` as type-position function-pointer head (`fn(...) -> T`).

Both forms are syntactically `fn` + `(`, disambiguated only by
context (statement vs type position). The current parser handles
case 2 by recognizing the lexeme `"fn"` inside Token::Ident; once
`fn` becomes its own token, case 2 needs a parser-side rewire to
accept Token::Function in type position too. Multi-iter; deferred
to a follow-up wave.

#### Scope decision: mod removed from wave 2

`mod` has no Vais single-char counterpart. Lexer has `U` for `use`
but no separate `mod` keyword — Vais uses a single use+module-path
syntax. The 24 `mod_*` collisions in
F-15-01 are user identifiers in vaisdb modules
(`mod_load`, `mod_register`, etc.), not a dual-syntax target.
Removing `mod` from wave 2 leaves it as an A1-class candidate (if
ever reserved) but not part of Step 15 dual-syntax work.

#### Wave 2 final state

| keyword | status | sub-batch | renames |
|---|---|---|---|
| else | LANDED | v28 | 0 |
| match | LANDED | v28 | 3 (url.vais) |
| return | LANDED | v28 | 0 |
| const | already-keyword | v28 (cleanup only) | 6 (LIVING_SPEC) |
| use | LANDED | v29 | 0 |
| type | LANDED | v29 | 0 |
| fn | DEFERRED (multi-iter parser refactor) | — | TBD |
| mod | OUT OF SCOPE (no single-char counterpart) | — | — |

Status: Step 15 stage 1 wave 2 second sub-batch LANDED.
master-plan v28 → v29. fn alias deferred. mod removed from
scope. Stage 3 (vaisc fmt --to= flags + selfhost migration) still
ahead — selfhost migration cannot fully complete until fn lands.

### F-15-05 — fn alias LANDED via single-iter parser refactor (2026-05-06)

F-15-04's deferred-as-multi-iter classification of fn was empirically
refuted. Actual cost: 1 commit, ~15 LOC parser change + 1 lexer line.

#### Lexer change

```rust
#[token("F", priority = 3)]
#[token("fn", priority = 3)]
Function,
```

Total dual-syntax keywords now 11/13.

#### Parser change

`vais-parser/src/types.rs::parse_base_type` (the type-position
dispatch that recognizes `fn(...) -> T` as a fn-pointer type) was
reading only `Token::Ident("fn")`. After the lexer change, the
input lexes as `Token::Function` instead, so the dispatch missed.

Fix: add a second arm next to the existing one. Both paths reach
the same `parse_fn_ptr_type` continuation:

```rust
if let Token::Ident(s) = &tok.token {
    if s == "fn" {
        self.advance();
        return Ok(Spanned::new(
            self.parse_fn_ptr_type()?,
            Span::new(start, self.prev_span().end),
        ));
    }
}
if matches!(tok.token, Token::Function) {
    let next_is_lparen = self
        .peek_next()
        .map(|t| matches!(t.token, Token::LParen))
        .unwrap_or(false);
    if next_is_lparen {
        self.advance();
        return Ok(Spanned::new(
            self.parse_fn_ptr_type()?,
            Span::new(start, self.prev_span().end),
        ));
    }
}
```

The LParen lookahead is the disambiguator: a bare Token::Function
in type position without `(` is malformed and should fall through
to the regular type-name path so the existing error reporting
stays intact. This avoids accidentally consuming an fn-decl head
when the user mistypes a type.

The fn-decl path (item/declarations.rs `parse_function_decl` and
the 20+ other Token::Function uses across parser modules) is
unchanged. Function declarations still recognize Token::Function
unambiguously because they appear at item-statement position, not
in a type slot.

#### Empirical learning — L-005 8th instance

F-15-04 predicted 'multi-iter parser refactor (decl head vs
pointer-type head 구분)'. The actual conflict was a single
dispatch site that needed Token::Function added next to the
existing Ident("fn") match. v29 deferred-as-multi-iter
classification was over-conservative.

Pattern repeats from prior L-005 instances: pre-empirical risk
estimate cites "광범위 사용처 영향" (here: 5+ broken negative
fixtures + core_certification fail) as multi-iter justification,
but the true root cause is *narrower* than the symptom surface.

Mitigation for future Step 15 / similar refactor: when a lexer
change breaks N parser fixtures, grep for the Token enum variant
in parser source first. If the broken fixtures all hit the same
variant, the fix is likely a single-arm addition, not a
multi-iter refactor. Distinguish "many broken tests" from
"many fix sites".

#### Wave 2 final state (v30)

| keyword | status | sub-batch |
|---|---|---|
| else | LANDED | v28 |
| match | LANDED | v28 |
| return | LANDED | v28 |
| const | already-keyword | v28 cleanup |
| use | LANDED | v29 |
| type | LANDED | v29 |
| fn | LANDED | v30 |
| mod | OUT OF SCOPE | — |

INTEGRITY OK preserved at v30 (std=82/82, vaisdb=261/261, runtime
smokes all green, core_certification 16/16). Negative fixtures
that broke at the v29 fn-attempt (syntax_extra_fn_as_arg /
syntax_closure_in_variable / syntax_neg_mod_missing_fn_keyword /
syntax_neg_trait_missing_fn_keyword / syntax_type_fn_pointer) all
green at v30 because the parser now handles both Token forms.

Status: Step 15 stage 1 wave 2 closed (modulo mod out-of-scope).
master-plan v29 → v30. Stage 3 (vaisc fmt --to= flags + selfhost
migration) unblocked.

### F-15-06 — Stage 3 round-trip codemod LANDED (2026-05-06)

#### Implementation

New CLI form on the existing `vaisc fmt` command:

```
vaisc fmt --to=multi  [--check] <input>
vaisc fmt --to=single [--check] <input>
```

Implementation: `compiler/crates/vaisc/src/commands/fmt_dual.rs`
(~280 LOC). Algorithm: re-lex the source via `vais_lexer::tokenize`,
walk tokens in order, and for each token whose span text equals one
of the 11 dual-syntax keyword pairs (`F`↔`fn`, `S`↔`struct`,
`E`↔`enum`, `EL`↔`else`, `M`↔`match`, `R`↔`return`, `T`↔`type`,
`U`↔`use`, `P`↔`pub`, `W`↔`trait`, `X`↔`impl`) substitute the
canonical form for the target. Non-keyword bytes (whitespace,
comments, string literals, identifiers) are copied through verbatim
by tracking the byte cursor between adjacent token spans.

Why span-based and not AST-based: AST round-trip from token to text
loses formatting (whitespace inside expressions, trailing commas,
comment placement). Span-based substitution preserves the input
byte-for-byte except where a keyword spelling actually changed.

9 unit tests in `fmt_dual::tests`:
- dual_form_parse — `multi` / `multi-char` / `single` / `single-char`
  CLI form recognition.
- replacement_table — 11 pairs map both directions; identifiers like
  `fn_handler` and `else_result` do NOT trigger.
- convert_simple_function — `F double(...) → fn double(...)` round-trip.
- convert_struct_and_match — multi-keyword + nested match arms.
- convert_else_and_return — keywords with no multi-char counterpart
  (here: `I` → no change) stay put.
- preserves_string_literal_with_keyword_word — `"fn or struct"`
  inside a string is untouched.
- preserves_comment_with_keyword_word — `# this F is a struct` inside
  a comment is untouched.
- ident_starting_with_keyword_unchanged — `call_test_fn(fn_ptr: i64)`
  round-trips bit-exact.
- round_trip_use_and_type — wave 2 keywords.

#### `selfhost-roundtrip.sh` invariant gate

New `scripts/selfhost-roundtrip.sh`. Two-pass protocol per file:

1. canonical = vaisc fmt --to=single <file>
2. multi     = vaisc fmt --to=multi  canonical
3. back      = vaisc fmt --to=single multi
4. assert canonical == back

Empirical baseline: **266/266 .vais files round-trip clean** across
`compiler/selfhost` + `compiler/std` +
`compiler/docs/language/LIVING_SPEC`.

#### Why two-pass instead of one-pass

A naive one-pass round-trip (input → multi → single == input)
fails on the existing baseline because **the baseline mixes forms
at write time**. Concrete example:

```vais
# compiler/std/result.vais line ~84
F map(&self, f: fn(T) -> T) -> Result<T, E> { ... }
```

This declares the method head with the single-form keyword `F` but
types its function-pointer parameter with the multi-form `fn(T) ->
T`. Both spellings lex to the same `Token::Function` so the parser
is happy and runtime is correct, but the file is "mixed" at the
byte level. A naive round-trip would diff because the codemod
normalizes everything to one canonical form on the first pass.

The fix: pass 1 picks the canonical form (single by default). Pass
2 then validates the round-trip on that canonical baseline. This
is the correct invariant — what we want to assert is that *once
the source is canonicalized, the dual-syntax codemod is bijective
at the byte level*. The fact that real-world source can be mixed
at write time is a normal consequence of `fn` and `F` aliasing the
same Token.

#### Status

Stage 3 partial close. The remaining sub-step is selfhost
migration: rewriting `compiler/std/` (and possibly
`compiler/selfhost/`) in canonical multi form and verifying that
the resulting binary is bit-identical to the current baseline. The
migration is now mechanically routine via:

```bash
find compiler/std -name '*.vais' \
  -exec ./target/release/vaisc fmt --to=multi {} \;
bash compiler/scripts/check-integrity.sh
```

Selfhost migration is deferred to a follow-up entry because it
changes ~80 files and benefits from a separate review surface.

master-plan v30 → v31.
