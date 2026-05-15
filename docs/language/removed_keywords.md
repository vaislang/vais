# Removed Keywords

Vais has removed several experimental keywords during its stabilization
work. This page is the single source of truth for what was removed,
when, why, and how to migrate existing code. New code should not use
any of these keywords; the lexer no longer recognizes them.

## `lazy` / `force`

**Removed in**: commit `8c60c075` — `refactor(compiler): remove
lazy/force keywords across all layers (ROADMAP #16+#17)`

**Was**:

```vais
x := lazy 42          // build a thunk
y := force x          // evaluate it
```

**Rationale**: lazy evaluation added pervasive complexity to the lexer,
parser, type checker, effect system, and code generators (text IR,
inkwell, JS). Real-world usage in `vais-apps` was zero before removal.
The cost of maintaining the feature outweighed the benefit, so the
whole layer came out.

**Migration**:
- Plain values that were wrapped in `lazy` can be used directly.
- Deferred side effects should go through a closure
  (`() -> T` function) that the caller invokes when needed.

```vais
// Before
x := lazy expensive_compute(input)
y := force x

// After
x := |_| expensive_compute(input)   // closure
y := x(())
```

## `spawn`

**Removed in**: commit `12592076` — `refactor(compiler): remove spawn
keyword across all layers (ROADMAP #15)`

**Was**:

```vais
handle := spawn compute(x)       // fire-and-forget async
value  := (spawn compute(x)).await
```

**Rationale**: `spawn` introduced a distinct runtime concept — a
future that runs independently of the enclosing `A F` (async function)
— without a clear executor model. Every codegen target (text IR,
inkwell, JS) had to carry its own scheduler shim. The design was
always described as "experimental", and no production code had adopted
it; removing it simplified the async story to just `A F` + `.await`.

**Migration**:
- Every `spawn expr` that was immediately `.await`-ed can drop the
  `spawn`: `(spawn f(x)).await` → `f(x).await`.
- Fire-and-forget patterns must be reworked against a real executor
  (none ships today). Track such rewrites as Phase 197+ work if the
  executor story matures.

```vais
// Before
result := (spawn slow_compute(10)).await

// After
result := slow_compute(10).await
```

## Single-char declaration / control / modifier forms (Step 19 P4)

**Removed in**: commit `2b485860` — `phase2-step19-p4: lexer single-char retire LANDED + 4 test file 정정` (loop 25, 2026-05-08).

**Was**: `F` `S` `E` `EN` `EL` `M` `R` `T` `U` `P` `W` `X` (12 forms) all lexed to keyword tokens (Function / Struct / Enum / EnumKeyword / Else / Match / Return / TypeKeyword / Use / Pub / Trait / Impl).

```vais
F add(a: i64, b: i64) -> i64 { R a + b }
S Point { x: f64, y: f64 }
E Color { Red, Green }
EN Result<T, E> { Ok(T), Err(E) }     // unambiguous form
W Show { F show(&self) -> str }
X Show: Color { F show(&self) -> str { "color" } }
M x { 1 => "one", _ => "?" }
T MyId = i64
U std::io
P F public_fn() -> i64 { 42 }
```

**Rationale**: Two empirical findings invalidated the original "AI token efficiency" justification for single-char keywords:

1. **LESSONS L-009 (codemod readability trap)**: a token-level `single → multi` codemod could not avoid clobbering generic-param positions. `EN Result<T, E>` re-rendered as `enum Result<type, enum>` because `T` and `E` lex as keywords in type position. The codemod was byte-correct under round-trip, but the intermediate output regressed readability — the very property single-char keywords were supposed to deliver.
2. **LESSONS L-010 (token-efficiency hypothesis empirically false)**: measured against `cl100k_base` (Anthropic / OpenAI BPE family), a 27-line factorial / Point / Result fixture lexed to **156 tokens single-form vs 156 tokens multi-form** — zero token difference. The 7.7% byte saving translated to ~0.1% of a 1M-token context window. Single-char saved disk bytes, not LLM tokens.

The justification therefore shrank to qualitative aesthetics, while the cost (codemod traps, dual-syntax infrastructure, OOD against Rust/Swift/Carbon training corpora) was structural. User decision (2026-05-06) retired all 12 forms.

Design doc: `docs/design/single-char-keyword-retirement.md`.
6-phase migration record (P1 deprecation warning → P2 generic param rename [aborted, L-011] → P3 codemod migration → P4 lexer atomic [this commit] → P5 doc cleanup → P6 infra retire) is in `WORKLOG.md` loops 16~25.

**Migration**: token-by-token canonical replacement.

| Retired | Canonical |
|---------|-----------|
| `F`     | `fn`      |
| `S`     | `struct`  |
| `E`     | `enum` (declaration) or `else` (after `}`) — context determines which |
| `EN`    | `enum`    |
| `EL`    | `else`    |
| `M`     | `match`   |
| `R`     | `return`  |
| `T`     | `type`    |
| `U`     | `use`     |
| `P`     | `pub`     |
| `W`     | `trait`   |
| `X`     | `impl`    |

Non-retired single-char keywords (`I` `L` `A` `B` `C` `D` `O` `N` `G` `Y`) remain. They were not retired because no multi-char alias was canonicalized for them and they are less prone to the L-009 generic-param trap (`I` vs identifier `i` differs only in case, and the parser does not encounter `I` in type position the way it encounters `T` / `E`).

```vais
// Before (Step 19 retired, 2026-05-08)
F add(a: i64, b: i64) -> i64 { R a + b }
EN Result<T, E> { Ok(T), Err(E) }
M x { 1 => "one", _ => "?" }

// After (canonical)
fn add(a: i64, b: i64) -> i64 { return a + b }
enum Result<T, E> { Ok(T), Err(E) }
match x { 1 => "one", _ => "?" }
```

For mechanical rewrites use `vaisc fmt --to=multi` (the dual-syntax codemod from Step 15 stage 3, retained even after I-4 retirement because it is the migration tool, not a runtime feature). Apply only at declaration-leading positions: `(^|\n|\\n|{|;)` followed by retired-form + space. Do NOT apply inside type positions (`<T>`, `(x: T)`, `-> T`) — see LESSONS L-009.

## How a future removal should be documented

If another keyword ever needs to come out:

1. Land the removal in one commit that touches **every** layer (lexer,
   AST, parser, type checker, effects, codegen text, codegen inkwell,
   codegen JS, LSP, macros). Include a ROADMAP entry linking to the
   removal commit.
2. Add a row to this file with the same **removed in / was /
   rationale / migration** shape. Migration examples are load-bearing
   — this file is the first place an AI or a human will look when a
   legacy example stops compiling.
3. Move any examples that still use the keyword into
   `examples/archive/` with a line in `examples/archive/README.md`
   tying them back to this document. Do **not** add them to
   `crates/vaisc/tests/examples_fresh_rebuild.rs::SKIP_LIST` — the
   archive directory is auto-excluded and carries its own contract.
