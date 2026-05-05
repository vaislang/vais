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
