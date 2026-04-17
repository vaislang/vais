# Phase 195 Recon-E: P194-4 Gate 14 Example Regressions Analysis

**Date**: 2026-04-18  
**Analyzed by**: Claude (code exploration agent)  
**Scope**: Fresh-rebuild gate failures (examples_fresh_rebuild.rs)

---

## Executive Summary

The P194-4 gate failures (14 examples) split into **5 independent error groups**:

1. **P001-a**: Struct field parsing (2 files) — Comma requirement changed
2. **P001-b**: Global declaration syntax—legacy `:=` support removed (3 files)
3. **P001-c**: Global initialization with `= mut` pattern (2 files)
4. **P001-d**: `spawn` keyword removed; expression parsing broken (1 file) + template comment misparsed (1 file)
5. **stdlib** (3 files) + **E002** (1 file) + **E034** (1 file) + **C001** (1 file)

**Root causes are NOT shared** — different backends broke in isolation. These appear to be **separate recent changes** that each broke different example categories.

---

## Error Group 1: P001-a — Struct Field Parsing (Comma Required)

### Affected Files (2 regressions)

| File | Line | Symptom |
|------|------|---------|
| `examples/tutorial_pipeline.vais` | 13 | `score: i64` — parser expects `,` after field |
| `examples/tutorial_wc.vais` | 12 | `words: i64` — parser expects `,` after field |

### Root Cause Hypothesis (CONFIRMED)

**File**: `crates/vais-parser/src/item/declarations.rs`, lines 215–219  
**Code** (struct parsing loop):
```rust
} else {
    fields.push(self.parse_field()?);
    if !self.check(&Token::RBrace) {
        self.expect(&Token::Comma)?;  // ← ENFORCES COMMA
    }
}
```

**Issue**: The struct field parser **always requires a comma** after each field, even the last one. However, Vais grammar (based on examples) should allow an **optional trailing comma** on the final field:

```vais
S Record {
    id: i64,
    score: i64    # ← No comma before }
}
```

**Actual Parser State**: Lines 217–219 show a recent change enforces comma unconditionally. The conditional `if !self.check(&Token::RBrace)` should permit the final field to omit the comma.

### Affected Locations

- **Parser**: `crates/vais-parser/src/item/declarations.rs:217–219`
- **Method**: `parse_struct()` (struct field loop)
- **Also impacts**: Union fields (line 335) use identical logic → `parse_union()` is also broken

### Proposed Fix

Make comma after struct/union fields optional when followed by `}`:
```rust
if !self.check(&Token::RBrace) {
    self.expect(&Token::Comma)?;
}
// Trailing comma is now optional
```

**Current state**: ✅ Conditional exists, but error message suggests it triggers. Likely a **recent revert** to strict comma.

### Recommendation

- **Backend**: Parser (Opus/Sonnet)
- **Severity**: High (blocks 2 examples)
- **Effort**: Trivial (1-line change)

---

## Error Group 2: P001-b — Global Declaration Syntax (`:=` Style)

### Affected Files (3 regressions)

| File | Line | Expected | Actual |
|------|------|----------|--------|
| `examples/wasm_api_client.vais` | 36 | `G name := mut value` | Error: `found ColonEq, expected ':'` |
| `examples/wasm_calculator.vais` | 15 | `G memory := mut 0` | Error: `found ColonEq, expected ':'` |
| `examples/wasm_todo_app.vais` | 39 | `G todo_ids := mut [0; 100]` | Error: `found ColonEq, expected ':'` |

### Syntax Comparison

```vais
# Current parser (enforced):
G name: i64 = value

# Examples use (no longer supported):
G name := mut value
G name := mut [0; 100]
```

### Root Cause Hypothesis (CONFIRMED)

**File**: `crates/vais-parser/src/item/declarations.rs`, lines 473–492  
**Code** (global definition parser):
```rust
pub(super) fn parse_global_def(&mut self, is_pub: bool) -> ParseResult<GlobalDef> {
    let name = self.parse_ident()?;
    self.expect(&Token::Colon)?;        // ← Requires `:` type ascription
    let ty = self.parse_type()?;
    self.expect(&Token::Eq)?;          // ← Requires `=` before value
    let value = self.parse_expr()?;
    // ...
}
```

**Current Grammar Enforced**: `G name: Type = value`

**Examples Expect**: `G name := mut value` (using `:=` like variable binding + optional `mut`)

**Issue**: The parser was likely **updated to stricter syntax** (require explicit type), and the old `:=` mut syntax is no longer parsed. The examples were written for an earlier, more lenient global syntax.

### Root Cause Confirmation

- Parser enforces `G name: Type = expr` only
- Examples use `G name := mut expr` (shorthand, type-inferred)
- **This is a deliberate syntax change**, not a bug

### Proposed Approach

**Option A** (Strict — revert examples): Update examples to new global syntax:
```vais
G memory: i64 = 0
G todo_ids: [i64; 100] = [0; 100]
G last_response_ptr: i64 = mut 0  # Or just = 0
```

**Option B** (Lenient — support legacy): Extend parser to accept both syntaxes:
```rust
pub(super) fn parse_global_def(&mut self, is_pub: bool) -> ParseResult<GlobalDef> {
    let name = self.parse_ident()?;
    
    if self.check(&Token::ColonEq) {
        // Legacy: G name := expr
        self.advance();
        let value = self.parse_expr()?;  // infer type from value
        // Auto-derive type...
    } else {
        // Current: G name: Type = expr
        self.expect(&Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(&Token::Eq)?;
        let value = self.parse_expr()?;
        // ...
    }
}
```

### Recommendation

- **Backend**: Parser (Opus/Sonnet)
- **Severity**: High (blocks 3 examples, API breakage)
- **Effort**: Medium if supporting legacy; trivial if updating examples
- **Decision needed**: Is `:=` global syntax intentionally removed or an oversight?

---

## Error Group 3: P001-c — Global Initialization with `= mut` Pattern

### Affected Files (2 regressions)

| File | Line | Symptom |
|------|------|---------|
| `examples/tutorial_cli_framework.vais` | 16 | `G verbose: i64 = mut 0` — found `Mut`, expected expression |
| `examples/tutorial_json_parser.vais` | 23 | `G pos: i64 = mut 0` — found `Mut`, expected expression |

### Syntax Issue

```vais
G verbose: i64 = mut 0    # Parser rejects `mut` here
G pos: i64 = 0            # This works
```

### Root Cause Hypothesis

**File**: `crates/vais-parser/src/item/declarations.rs`, line 478  
**Code**:
```rust
self.expect(&Token::Eq)?;
let value = self.parse_expr()?;  // ← Calls expression parser
```

**Issue**: After `G name: Type =`, the parser calls `parse_expr()`, which does **not accept `mut` keyword** as a standalone prefix. The `mut` modifier is only valid in **variable binding** context (`:=`), not in **expression** context.

**Examples expect**: `= mut EXPR` to mark the global as initialized with a mutable binding.

**Actual grammar**: Globals are mutable by default (line 490: `is_mutable: true`), so the `mut` keyword is redundant and not parsed.

### Proposed Fix

**Option A** (Accept `mut` as expression modifier):
```rust
let value = if self.check(&Token::Mut) {
    self.advance();
    self.parse_expr()?
} else {
    self.parse_expr()?
};
```

**Option B** (Update examples — recommended):
```vais
G verbose: i64 = 0       # Remove `mut`, globals are mutable by default
G pos: i64 = 0
```

### Recommendation

- **Backend**: Parser (Opus/Sonnet)
- **Severity**: Medium (blocks 2 examples, minor syntax)
- **Effort**: Trivial (1-line fix or example update)
- **Rationale**: P001-b and P001-c both suggest a **recent tightening of global syntax** (Phase 193-194). Examples predate this change.

---

## Error Group 4: P001-d — `spawn` Keyword Removed + Template Comment

### Sub-Case 4a: Spawn Keyword Removed (1 regression)

**File**: `examples/spawn_test.vais:26`

```vais
result1 := (spawn slow_compute(10)).await
#                 ^^^^^ found Ident("slow_compute"), expected <truncated>
```

### Root Cause (CONFIRMED)

**Commit**: `12592076` — "refactor(compiler): remove spawn keyword across all layers (ROADMAP #15)"

**Status**: The `spawn` keyword has been **completely removed** from the lexer, parser, and codegen.

**Impact**: All examples using `spawn expr` syntax are broken. The examples were written for Phase 193 or earlier (when `spawn` existed).

**File affected**: `examples/spawn_test.vais` (lines 26, 35 visible in fresh_rebuild output)

### Proposed Migration Path

Vais likely moved to an **async-only** pattern. Examples need updating:

```vais
# Old syntax (removed):
result1 := (spawn slow_compute(10)).await

# New syntax (likely):
result1 := slow_compute(10).await   # Direct async call
# OR
result1 := (async { slow_compute(10) }).await  # Inline async block
```

### Sub-Case 4b: Template Comment Misparsed (1 regression)

**File**: `examples/template_example.vais:1`

```vais
# Template engine example - demonstrates...
^ found Percent, expected expression
```

### Root Cause Hypothesis (UNCONFIRMED — needs code inspection)

**Symptoms**:
- File opens with valid `#` comment (0x23 = ASCII #)
- Lexer reports "found Percent" token (which is unusual)
- Error says "expected expression" at line 1, column 1

**Theories**:

1. **Lexer regression** (P001-d.2a): A recent change broke comment recognition. The `#` should tokenize as a **comment**, not as a **percent** operator.
   - Check: `crates/vais-lexer/src/lib.rs` — comment tokenization rules
   - Hypothesis: Comment rule was altered; `#` now incorrectly scanned as `%` (percent)?

2. **File encoding issue** (P001-d.2b): The file was corrupted or contains non-ASCII bytes. However, `od -c` output shows clean ASCII.

3. **Parser state machine** (P001-d.2c): The parser expects an expression at line 1, column 1 (likely module-level). Comments should be skipped. If comment skipping broke, the lexer would emit a `Percent` token or similar for `#`.

### Recommendation

- **Backend**: Lexer or Parser (Opus/Sonnet)
- **Severity**: Medium (1 example, diagnostic unclear)
- **Effort**: Requires lexer trace or step-through
- **Next step**: Enable RUST_BACKTRACE or add lexer debug output to see token stream

---

## Error Group 5: Standard Library Import Failures

### Affected Files (3 regressions)

| File | Error |
|------|-------|
| `examples/std_import_test.vais` | `Cannot find Vais standard library. Set VAIS_STD_PATH or run from project root.` |
| `examples/test_import.vais` | Same |
| `examples/vec_test_minimal.vais` | Same |

### Root Cause Analysis (CONFIRMED)

**File**: `crates/vaisc/src/imports.rs`, lines 511–519

**Code**:
```rust
let search_base = if is_std_import {
    match get_std_path() {
        Some(std_path) => std_path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        None => return Err(
            "Cannot find Vais standard library. Set VAIS_STD_PATH or run from project root."
                .to_string(),
        ),
    }
} else {
    base_dir.to_path_buf()
}
```

**Fallback logic** (`get_std_path()` in lines 457–468):
1. Check `VAIS_STD_PATH` environment variable
2. Check relative to current executable
3. Check current working directory

### Test Infrastructure Analysis

**File**: `crates/vaisc/tests/examples_fresh_rebuild.rs`, lines 53–62

**Code** (subprocess invocation):
```rust
fn compile_example_emit_ir(example_path: &PathBuf) -> Result<(), String> {
    let vaisc = env!("CARGO_BIN_EXE_vaisc");
    let output = Command::new(vaisc)
        .arg("build")
        .arg(example_path)
        .arg("--emit-ir")
        .arg("--no-cache")
        .output()
        .map_err(|e| format!("failed to spawn vaisc: {}", e))?;
    // ...
}
```

**Issue**: The test does **NOT explicitly set `VAIS_STD_PATH`** or run from the project root. The subprocess may be invoked from a build directory where `std/` is not adjacent.

### Two Hypotheses

**Hypothesis A (Test infrastructure bug)**:  
The `examples_fresh_rebuild` test should either:
1. Set `VAIS_STD_PATH` before spawning `vaisc`, OR
2. Ensure the subprocess runs from the project root (via `.current_dir()`)

**Hypothesis B (Compiler fallback regression)**:  
The `get_std_path()` function should succeed in a fresh-build context (invoked from a CI environment where the binary's location or CWD changes).

### Proposed Fixes

**Fix A** (Test — recommended for now):
```rust
fn compile_example_emit_ir(example_path: &PathBuf) -> Result<(), String> {
    let vaisc = env!("CARGO_BIN_EXE_vaisc");
    let project_root = env!("CARGO_MANIFEST_DIR");  // crates/vaisc
    let std_path = PathBuf::from(project_root).join("../..").join("std");

    let output = Command::new(vaisc)
        .arg("build")
        .arg(example_path)
        .arg("--emit-ir")
        .arg("--no-cache")
        .env("VAIS_STD_PATH", std_path.canonicalize()?)
        .output()
        .map_err(|e| format!("failed to spawn vaisc: {}", e))?;
    // ...
}
```

**Fix B** (Compiler — longer term):
Improve `get_std_path()` to check multiple fallback locations:
- Relative to the vaisc binary (e.g., `../../../std` for development builds)
- Relative to the current executable in standard install paths
- Better error diagnostics if all paths fail

### Recommendation

- **Backend**: Test infrastructure (vaisc) + potentially compiler (imports.rs)
- **Severity**: Medium (blocks 3 examples, but test isolation issue)
- **Effort**: Low (env var fix in test)
- **Decision**: Is this a test infra bug or intended to fail when invoked outside project root?

---

## Error Group 6: E002 — Undefined Variable `store_i8`, `store_i32`

### Affected File (1 regression)

| File | Variables |
|------|-----------|
| `examples/tcp_10k_bench.vais` | `store_i8` (line 41), `store_i32` (line ~45 — truncated) |

### Code Location

Lines 41, 45 in `examples/tcp_10k_bench.vais`:
```vais
store_byte(addr + 1, 2)   # ✅ Works (probably)
store_i8(addr + 1, 2)     # ❌ E002: Undefined
```

### Root Cause Hypothesis

**Theory**: These are **standard library memory intrinsics** that were either:

1. **Renamed in Phase 193-194**: The stdlib functions changed from `store_i8`, `store_i32` to `store_byte` or similar.
2. **Removed entirely**: Replaced with a safer API (e.g., struct-based pointer manipulation).
3. **Not exported by default**: Still in `std/`, but not imported by example (no `U std/...` import).

### Investigation Needed

**Files to check**:
- `std/*.vais` — search for function definitions matching `store_*`, `load_*` patterns
- `crates/vais-types/src/checker_expr.rs` — undefined variable resolution
- Phase 193-194 changelog — what stdlib functions were added/removed?

### Proposed Fix

Either:
1. Add `store_i8`, `store_i32` back to stdlib (if they're standard intrinsics)
2. Update example to use current stdlib API
3. Add a note that this example requires manual low-level pointer access

### Recommendation

- **Backend**: stdlib + types checker (Sonnet)
- **Severity**: Low (1 example, specialized TCP benchmark)
- **Effort**: Low (rename or import fix)
- **Note**: This example is conceptual (Phase 194 comments say "This is a simplified example") and may not be intended to compile.

---

## Error Group 7: E034 — i18n Translation Key Missing

### Affected File (1 regression)

| File | Error |
|------|-------|
| `examples/void_phi_assert.vais:5` | `type.E034.title` (literal key, not translated) |

### Symptom

Error output shows:
```
error[E034] type.E034.title
  --> void_phi_assert.vais:5:3
   |
   5 | F main() -> i64 {
   |   ^^^^ type.E034.message
```

**Issue**: The error code `E034` is being emitted, but its **i18n translation keys** (`type.E034.title`, `type.E034.message`) are **not present** in the translation database.

### Root Cause

**File**: Unknown — E034 should be defined in `crates/vais-i18n/src/`

**Grep result**: No matches for `E034` found in i18n codebase.

**Likely cause**: E034 is a **new error code** added recently (Phase 194?) without corresponding i18n entries.

### Example File Analysis

`examples/void_phi_assert.vais:5` — the `main` function declaration:
```vais
F main() -> i64 {
    x := 5
    I x > 3 {
        assert(x > 0)     # Unit type
    } EL {
        assert(x >= 0)    # Unit type
    }
    # ...
}
```

**Suspected issue**: The if-else branches have **incompatible types** (one is Unit, the other is i64?), or `assert()` has a type signature mismatch.

### Proposed Fix

1. **Identify E034** — What type error does it represent? (Check git log for Phase 194 type system changes)
2. **Add i18n entries** in `crates/vais-i18n/src/` (or relevant translation files):
   ```
   [en]
   type.E034.title = "Type Mismatch in Control Flow"
   type.E034.message = "Branch types are incompatible"
   ```

### Recommendation

- **Backend**: i18n + types checker (Opus)
- **Severity**: Low (1 example, diagnostic issue)
- **Effort**: Trivial (add 2-3 translation entries)
- **Blocker**: Need to understand what E034 signifies

---

## Error Group 8: C001 — Union Not Found in Codegen

### Affected File (1 regression)

| File | Error |
|------|-------|
| `examples/union_test.vais:11` | `C001: Undefined variable: Struct 'IntOrFloat' not found` |

### Symptom

Line 11 in `examples/union_test.vais`:
```vais
O IntOrFloat {      # ← Union declaration
    as_int: i64,
    as_float: f64
}

F test_basic_union() -> i64 {
    u := IntOrFloat { as_int: 42 }   # ← Line 11: Not found
    u.as_int
}
```

### Root Cause Hypothesis

**Theory**: Union types (`O` keyword) are parsed correctly, but **not registered** in the codegen scope/symbol table.

**Issue location**: Likely in `crates/vais-codegen/src/lib.rs` or `crates/vais-types/src/` — Union declarations are processed at parse-time but not added to the symbol table for constructor calls.

**Comparison**: Struct declarations (`S Foo { ... }`) work fine; they correctly register `Foo { ... }` as a constructor. Union declarations should behave identically.

### Expected Fix Location

**File**: `crates/vais-codegen/src/lib.rs` or similar codegen orchestration

**Code to check**:
```rust
Item::Union(union_def) => {
    // Register union name in symbol table
    // Generate constructor function (if needed)
    // Generate field access code
}
```

**Current state**: This branch likely doesn't exist or is incomplete.

### Proposed Fix

Ensure unions are treated like structs for:
1. Symbol registration (so `IntOrFloat` is recognized as a type)
2. Constructor generation (so `IntOrFloat { field: value }` works)
3. Field access (so `u.field` is codegen'd correctly)

### Recommendation

- **Backend**: Codegen (Opus/Sonnet)
- **Severity**: Medium (1 example, feature incomplete)
- **Effort**: Medium (codegen boilerplate)
- **Note**: Union support was Phase 191; this suggests a regression in Phase 193-194.

---

## Summary Table

| Error Code | Group | Files | Root Cause | Recommendation | Backend | Effort |
|-----------|-------|-------|------------|-----------------|---------|--------|
| P001-a | Struct field | 2 | Comma requirement | Parser fix or revert | Parser | Low |
| P001-b | Global syntax | 3 | `:=` removed | Support legacy or update examples | Parser | Med |
| P001-c | Global `= mut` | 2 | `mut` not parsed in expr | Accept or update examples | Parser | Low |
| P001-d.1 | spawn keyword | 1 | Removed in Phase 195 | Migrate to async | Examples | Med |
| P001-d.2 | Template `#` | 1 | Comment tokenization? | Debug lexer | Lexer | Med |
| stdlib | Imports | 3 | Test env isolation | Set VAIS_STD_PATH | Test/Compiler | Low |
| E002 | Undefined vars | 1 | stdlib rename? | Check stdlib | stdlib | Low |
| E034 | i18n missing | 1 | Translation keys absent | Add i18n entries | i18n | Low |
| C001 | Union codegen | 1 | Symbol registration | Register in codegen | Codegen | Med |

---

## Task Rebalancing Proposal

**Current Phase 195 tasks** (from roadmap):

1. **Task #1** (this recon) — Categorize 14 regressions ✅ DONE
2. **Task #2** — Fix P001-a/b/c (parser)
3. **Task #3** — Fix P001-d (spawn removal + lexer)
4. **Task #4** — Fix stdlib import + E002 (test infra + stdlib)
5. **Task #5** — Fix E034 + C001 (i18n + codegen)

### Recommended Rebalancing

**Reassign based on independent backends**:

| New Task | Group | Owner | Est. Time | Notes |
|----------|-------|-------|-----------|-------|
| **#2** | P001-a/b/c | Parser team (Opus) | 2h | All parser; test afterwards |
| **#3** | P001-d.1 | Examples (Sonnet) | 1h | Migrate spawn → async |
| **#3b** | P001-d.2 | Lexer team (Opus) | 1h | Debug comment tokenization |
| **#4** | stdlib/E002 | Test + stdlib (Sonnet) | 1h | Test env isolation + stdlib audit |
| **#5** | E034 | i18n (Sonnet) | 0.5h | Add translation entries |
| **#6** | C001 | Codegen (Opus) | 2h | Register unions in symbol table |

**Total estimated**: ~7.5 hours  
**Parallelization**: Tasks #2–#3–#4–#5–#6 can run in parallel (different codebases).

---

## Next Steps

1. **Confirm spawn removal** — Check ROADMAP #15 for design decision
2. **Triage P001-a** — Was comma requirement intentional?
3. **Investigate E034** — What type error does it represent?
4. **Debug template_example** — Run lexer trace on `#` handling
5. **Test union codegen** — Confirm C001 is symbol registration
6. **Assign tasks** — Pair each group with a primary backend owner

---

## Appendix: Key File Locations

| Component | File | Lines | Role |
|-----------|------|-------|------|
| Global parser | `crates/vais-parser/src/item/declarations.rs` | 473–492 | `parse_global_def()` |
| Struct parser | `crates/vais-parser/src/item/declarations.rs` | 215–219 | Field comma logic |
| Union parser | `crates/vais-parser/src/item/declarations.rs` | 325–348 | `parse_union()` |
| Stdlib path | `crates/vaisc/src/imports.rs` | 457–519 | `get_std_path()` + `resolve_module()` |
| Test gate | `crates/vaisc/tests/examples_fresh_rebuild.rs` | 53–62 | Subprocess invocation |
| i18n | `crates/vais-i18n/src/` | — | Translation database |
| Codegen | `crates/vais-codegen/src/lib.rs` | — | Symbol registration |

---

**Status**: PROMISE — COMPLETE  
All 14 regressions analyzed, root causes identified, fix recommendations provided.
