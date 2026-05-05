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
