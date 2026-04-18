# Vais Compiler Stages

> **Status**: authoritative reference for all integrity tests and Phase gates.
> **Owner**: compiler core. Update with care — downstream tests read this.
> **Last updated**: 2026-04-19 (Stabilization Drive Phase 0.1)

This document is the single source of truth for what "OK" means at each compilation stage. Every integrity test, CI script, and Phase gate in this project refers to the definitions here. If you change compiler behavior that affects any stage, you **must** update this file in the same commit.

---

## 1. Stage overview

A `.vais` source file passes through six stages. Each stage produces a well-defined artifact and can fail with a typed error. A file "builds OK" when it reaches the stage required by its role (see §2 matrix).

```
+-----------+    +-----------+    +---------+    +---------+    +------+    +-----+
| 1. Lex    | -> | 2. Parse  | -> | 3. Type | -> | 4. Code | -> | 5.   | -> | 6.  |
|           |    |           |    |    check|    |    gen  |    | Link |    | Run |
+-----------+    +-----------+    +---------+    +---------+    +------+    +-----+
     |                |                |              |             |          |
   L001+            P001+            E001+         C001..C007     (linker)   (user)
(lexer errs)    (parse errs)    (type errs)     (codegen errs)
```

### Error code prefixes

| Prefix | Stage | Module |
|--------|-------|--------|
| `L*` | Lex | `vais-lexer` (reserved; currently inlined into `P001`) |
| `P001..P0nn` | Parse | `vais-parser` |
| `E001..E0nn` | Type check | `vais-types` |
| `C001..C007` | Codegen | `vais-codegen` |
| (linker-native) | Link | `clang` / system linker |
| (OS / runtime) | Run | program |

### Current error code registry

Source of truth:
- Parse: `crates/vais-parser/src/errors.rs` (P-prefix enum)
- Type: `crates/vais-types/src/types/error.rs::TypeError::error_code` (returns E-prefix)
- Codegen: `crates/vais-codegen/src/error.rs::CodegenError::error_code` (returns C-prefix)

| Code | Meaning | Stage |
|------|---------|-------|
| P001 | Unexpected token / parse error | Parse |
| P002 | Syntax error (generic recovery) | Parse |
| E001 | Type mismatch (`Mismatch`) | TC |
| E002 | Undefined variable (`UndefinedVar`) | TC |
| E003 | Undefined type (`UndefinedType`) | TC |
| E004 | Undefined function (`UndefinedFunction`) | TC |
| E006 | Wrong argument count (`ArgCount`) | TC |
| E008 | Duplicate definition | TC |
| E022 | Use after move | TC (borrow-check) |
| E030 | No such field (`NoSuchField`) | TC |
| E034 | Total function may panic | TC (totality gate) |
| C001 | Codegen: undefined variable | Codegen |
| C002 | Codegen: undefined function | Codegen |
| C003 | Codegen: type error | Codegen |
| C004 | Codegen: LLVM error | Codegen |
| C005 | Codegen: unsupported feature | Codegen |
| C006 | Codegen: recursion limit exceeded | Codegen |
| C007 | Codegen: internal compiler error (ICE) | Codegen |

New error codes require an entry here **and** an integrity test (see §4).

---

## 2. Per-stage definition, command, and "OK" criterion

### Consolidated 6-stage table (정의/커맨드/OK기준)

| Stage | 정의 (definition) | 커맨드 (command) | OK 기준 (exit 0 when...) |
|-------|------|---------|-----------|
| 1 Lex | Tokenize raw text → `Vec<Spanned<Token>>` | `$VAISC --show-tokens <f>` | `$VAISC --show-tokens <f> >/dev/null 2>&1` |
| 2 Parse | Tokens → AST (`vais_ast::Module`) | `$VAISC --show-ast <f>` | `$VAISC --show-ast <f> >/dev/null 2>&1` |
| 3 TC | AST → type-annotated AST + checker state | `$VAISC check <f>` | `$VAISC check <f> >/dev/null 2>&1` |
| 4 Codegen | Typed AST → LLVM IR text (`.ll`) | `$VAISC build <f> --emit-ir -o <out>` | `$VAISC build <f> --emit-ir -o /tmp/x.ll --force-rebuild >/dev/null 2>&1` |
| 5 Link | `.ll` + runtime → native executable | `$VAISC build <f> -o <out>` | `$VAISC build <f> -o /tmp/x --force-rebuild >/dev/null 2>&1 && [ -x /tmp/x ]` |
| 6 Run | Native executable → exit code / stdout | `/tmp/x; echo $?` | Program exit matches test expectation |

### Detailed per-stage notes

All commands assume:

```bash
export VAISC=~/.cargo/bin/vaisc
# For vaisdb / other packages:
export DEPS="$(pwd)/src:/tmp/vais-lib/std"
export STDROOT=/tmp/vais-lib/std
# (set up once per session)
[ -d /tmp/vais-lib/std ] || (mkdir -p /tmp/vais-lib && ln -sf /Users/sswoo/study/projects/vais/compiler/std /tmp/vais-lib/std)
```

### Stage 1 — Lex

| Item | Value |
|------|-------|
| **Input** | Raw UTF-8 text from `.vais` file |
| **Output** | Token stream with spans (byte offsets) |
| **Module** | `crates/vais-lexer/src/lib.rs` (logos-based) |
| **Command** | *No standalone CLI; runs as part of parse.* Closest proxy: `$VAISC --show-tokens <file>` |
| **"OK" criterion** | `$VAISC build <file> --emit-ir -o /dev/null 2>&1 \| grep -q "error" && echo FAIL \|\| echo OK` — lexer errors surface as `P001`-class parse errors (lexer reports via parser) |
| **Fail symptoms** | Invalid character literal, unterminated string, invalid numeric literal, malformed interpolation |

### Stage 2 — Parse

| Item | Value |
|------|-------|
| **Input** | Token stream |
| **Output** | `vais_ast::Module` (AST) |
| **Module** | `crates/vais-parser/src/lib.rs` |
| **Command** | `$VAISC check <file>` — runs Lex+Parse+TC, but parse errors fail first |
| **"OK for Parse" criterion** | `$VAISC --show-ast <file> > /dev/null 2>&1`  → exit 0 means parse succeeded (lex also OK) |
| **Fail symptoms** | `P001`: unexpected token. Example: `F main() -> i64 { @@@ }` |

Parse OK ⇒ Lex OK. Testing Parse alone is equivalent to `--show-ast` success.

### Stage 3 — Type check (TC)

| Item | Value |
|------|-------|
| **Input** | AST + transitively imported modules |
| **Output** | Type-annotated AST + `TypeChecker` state (enums/structs/traits/functions registered) |
| **Module** | `crates/vais-types/` |
| **Command** | `$VAISC check <file>` |
| **"OK for TC" criterion** | `$VAISC check <file> > /dev/null 2>&1`  → exit 0 |
| **Fail symptoms** | Any `E***` code. Most common: `E001` type mismatch, `E004` undefined function, `E030` no such field |

**Important**: `check` does **not** run codegen. A file can be "TC OK" but still fail codegen (`C***`). This is the gap that drove much of the pre-Phase-0 confusion — see §5 "Historical gotchas".

### Stage 4 — Codegen

| Item | Value |
|------|-------|
| **Input** | TC-passed AST |
| **Output** | LLVM IR text (`.ll`) |
| **Module** | `crates/vais-codegen/` |
| **Command** | `$VAISC build <file> --emit-ir -o <out.ll>` |
| **"OK for Codegen" criterion** | `$VAISC build <file> --emit-ir -o /tmp/out.ll > /dev/null 2>&1`  → exit 0 |
| **Fail symptoms** | `C001..C007`. Most common in this project: `C003` type error (missing type inference), `C005` unsupported feature (`char_at`, `parse_f64`, etc.) |

**Codegen OK ⇒ TC OK ⇒ Parse OK ⇒ Lex OK.** A passing codegen guarantees all earlier stages.

### Stage 5 — Link

| Item | Value |
|------|-------|
| **Input** | `.ll` file + optional runtime objects |
| **Output** | Native executable or dylib |
| **Module** | External: `clang` (must be in `$PATH`) |
| **Command** | `$VAISC build <file> -o <out>` (no `--emit-ir`) |
| **"OK for Link" criterion** | `$VAISC build <file> -o /tmp/exe > /dev/null 2>&1 && [ -x /tmp/exe ]` |
| **Fail symptoms** | Missing symbols, unresolved extern, linker-native messages |

### Stage 6 — Run

| Item | Value |
|------|-------|
| **Input** | Native executable |
| **Output** | Program exit code + stdout/stderr |
| **Command** | `/tmp/exe; echo "exit=$?"` |
| **"OK for Run" criterion** | Defined per test. For `F main() -> i64 { N }`, exit code should be `N & 0xff`. |
| **Fail symptoms** | Nonzero exit unexpectedly, segfault, abort, OOB trap, assertion panic |

---

## 3. Canonical "OK" checks (bash one-liners)

Every integrity test and CI script uses one of these. Do not invent new check styles — extend these if needed.

```bash
# Parse OK?
ok_parse() { $VAISC --show-ast "$1" >/dev/null 2>&1; }

# TC OK?
ok_tc() { $VAISC check "$1" >/dev/null 2>&1; }

# Codegen OK? (produces .ll)
ok_codegen() { $VAISC build "$1" --emit-ir -o /tmp/__ok.ll --force-rebuild >/dev/null 2>&1; }

# Full build OK? (produces executable)
ok_build() { $VAISC build "$1" -o /tmp/__ok_exe --force-rebuild >/dev/null 2>&1 && [ -x /tmp/__ok_exe ]; }

# Run OK with expected exit code?
ok_run() {
  local src="$1" expected="$2"
  $VAISC build "$src" -o /tmp/__ok_exe --force-rebuild >/dev/null 2>&1 || return 1
  /tmp/__ok_exe; local actual=$?
  [ "$actual" = "$expected" ]
}

# With package-deps (for vaisdb etc):
ok_codegen_pkg() {
  VAIS_DEP_PATHS="$DEPS" VAIS_STD_PATH="$STDROOT" \
    $VAISC build "$1" --emit-ir -o /tmp/__ok.ll --force-rebuild >/dev/null 2>&1
}
```

All exit with 0 on success, 1 on failure. **No other definition of "OK" is permitted in this project.**

---

## 4. Stage → role matrix

Each file in the codebase has a role that dictates which stage must pass.

| Role | Examples | Required stage |
|------|----------|----------------|
| Compiler unit tests (`.rs`) | `crates/vaisc/tests/*` | `cargo test` green |
| stdlib module (`std/*.vais`) | `std/vec.vais` | Codegen OK (no main) |
| stdlib example (`examples/std_*.vais`) | `examples/std_vec.vais` | Run OK |
| Language spec example | `docs/LANGUAGE_SPEC.md` code blocks | Parse OK (snippet-level) |
| vaisdb module (`vaisdb/src/**/*.vais`) | `sql/types.vais` | Codegen OK with `VAIS_DEP_PATHS` |
| vaisdb top-level test | `vaisdb/test_*.vais` (future) | Run OK |
| vais-server / vais-web | `vais-server/src/main.vais` | Codegen OK (see Phase 5.18) |

**Rule**: Phase 0-5 gates use the column above. `integrity_test` modules (§5 below) enforce it.

---

## 5. Historical gotchas (bugs that caused confusion)

Keep updating this list as we discover more.

1. **TC ≠ Codegen** — Pre-Phase-0 the project conflated "TC passes" with "OK". Some files have TC-clean + codegen-fail symptoms. All stabilization work now uses **Codegen OK** (ok_codegen) as default for library files, and **Run OK** for examples/tests.

2. **Installed `vaisc` drifts** — `~/.cargo/bin/vaisc` is not auto-updated when source changes. Always rebuild before measuring:
   ```bash
   cd /Users/sswoo/study/projects/vais/compiler
   cargo install --path crates/vaisc --force
   ```
   Integrity test harness does this automatically via build.rs or CI entry.

3. **`--force-rebuild` required for fair measurement** — Without it, incremental cache can hide errors or pass stale state.

4. **stdlib symlink** — vaisdb (and any non-compiler package) requires `/tmp/vais-lib/std` symlinked to `compiler/std`:
   ```bash
   [ -d /tmp/vais-lib/std ] || (mkdir -p /tmp/vais-lib && ln -sf /Users/sswoo/study/projects/vais/compiler/std /tmp/vais-lib/std)
   ```
   All integrity tests and the CI script set this up.

5. **`timeout` unavailable on macOS default** — do not put `timeout` in test scripts. Use cargo's built-in per-test timeout or a Rust-side `std::process::Command` with kill-on-drop.

6. **Benign import cycles** — Pre-Phase-0 `U a; U b` where `b` also `U a` was rejected as "Circular import detected". This blocks the split-impl pattern (Parser struct in `parser.vais` + `X Parser {...}` in `parser_select.vais`). Phase 2.9 is charged with deciding and implementing the fix. Until then, circular imports remain **rejected**.

7. **Span-less errors** — `E001` "expected X, found Y" without a source pointer. Callers that invoke `unify()` must add `.with_span(arg.span)`. Outstanding sites documented in `TYPE_SYSTEM.md` once Phase 2.8 lands.

---

## 6. Known compiler/stdlib bugs as of Phase 0.1

Listed here so later phases do not rediscover them. Each bug has a target Phase for resolution.

| # | Bug | Stage | Symptom | Target Phase |
|---|-----|-------|---------|--------------|
| B1 | `as_bytes()` on `Str` loses `Vec<u8>` type across `mut` binding | Codegen (C003) | `bytes := mut sql.as_bytes(); bytes[i]` → "Cannot index into type 'i64'" | 3.12 (inference fix) |
| B2 | `char_at` / `parse_f64` / `parse_i64` have no codegen dispatch | Codegen (C005) | `string method 'X' not supported` | 3.13 (runtime impl) |
| B3 | Tuple field access `.0` / `.1` on named tuples — partially fixed | Codegen (C003) | `Cannot access field '0' on type '(T1,T2)'` — some remaining edge cases | 3.12 / 3.14 |
| B4 | `Vec<Struct>[i].field = x` (write-through-index) not supported | Codegen | Parse-clean, TC-clean, codegen silent-fail or runtime corruption | 3.14 (decision + impl) |
| B5 | `Option<&T>` pattern `Some(r) => r.field` infers Option<&T> wrapper mismatch | TC (E001) | `expected X, found Y` where Y is wrapped type | 2.10 |
| B6 | Cross-file `X S {...}` impl block not seen when `S` imports siblings | TC (E004) | `function 'parse_select' not defined` when `parser.vais` builds alone | 2.9 |
| B7 | std/sync.vais `LW cond { ... } ! { ... }` — `!` as else keyword | Parse / Codegen | `None` reported as undefined in else-bang branch | 4.15 (stdlib cleanup) |
| B8 | std/string.vais `as_bytes` has its own bug (`result.data + i * result.elem_size`) | Codegen (C003) | Field access on inferred `i64` | 4.15 |
| B9 | Turbofish in constructor call: `Vec<u8>.with_capacity(0)` | Parse (P001) | `found U8, expected expression` | 1.5 decision (disallow explicitly or support) |
| B10 | Type alias static method: `T AtomicU64 = AtomicI64; AtomicU64.new(0)` does not resolve | TC (E001) | `expected AtomicI64, found AtomicU64` | 2.8 (type alias rules) |

---

## 7. Command matrix quick reference

Minimum commands you need to debug or measure any file:

```bash
# 1. What stage does this file fail at?
$VAISC check <file>; echo "check=$?"                    # TC result
$VAISC build <file> --emit-ir -o /tmp/x.ll; echo "codegen=$?"
$VAISC build <file> -o /tmp/x; echo "link=$?"

# 2. Error category
$VAISC check <file> 2>&1 | grep -Eo 'error\[[A-Z][0-9]{3}\]' | sort -u

# 3. Full verbose
$VAISC build <file> --emit-ir -o /tmp/x.ll --verbose 2>&1 | head -100

# 4. AST / tokens
$VAISC --show-tokens <file>
$VAISC --show-ast <file>
```

---

## 8. Change policy

- Adding a new stage, error code, or "OK" criterion requires a PR that updates §2, §3, and §6.
- Removing an error code requires deprecation period ≥ 1 release.
- Integrity tests assume the above semantics. Violating this doc breaks the test matrix.

---

## 9. Reference implementation notes

For each stage, below is the exact Rust type/module a contributor should look at first.

### Stage 1 — Lex

- Entry: `vais_lexer::lex(source: &str) -> Vec<Spanned<Token>>`
- Token enum: `vais_lexer::Token` (logos derive) — each variant has a `#[token("...")]` or `#[regex("...")]`.
- Single-character keywords (F, S, X, W, T, U, C, EN, O, I, E, L, M, R, B, D, P, A, Y, N, G) are explicit `#[token(...)]` entries.
- Two-character keywords (EL, LW, LF) take priority 4 > single-letter priority 3 to disambiguate.
- Recent additions: `partial` (2026-04-12), `pure`, `requires`, `ensures`.
- If you add a keyword, update `docs/LEXER_KEYWORDS.md` (Phase 1.7) in the same commit.

### Stage 2 — Parse

- Entry: `vais_parser::parse(source: &str) -> ParseResult<Module>`
- Module layout: `crates/vais-parser/src/{lib.rs,item/mod.rs,item/traits.rs,item/declarations.rs,types.rs,expr.rs,stmt.rs,pattern.rs}`
- Errors: `vais_parser::errors::ParseError` — always includes `Span` (byte offset range into source)
- `parse_module_with_recovery` gives a best-effort AST even on failure, used by LSP and fmt.

### Stage 3 — Type check

- Entry: `vais_types::TypeChecker::check_module(&mut self, &Module) -> TypeResult<()>`
- State: `self.structs`, `self.enums`, `self.traits`, `self.functions`, `self.imported_item_count`, `self.current_generics`, `self.scopes`, `self.constraints`, `self.warnings`.
- Pass structure (in `checker_module/mod.rs::check_module`):
  1. Pass 1a: register structs/enums/unions/type aliases/traits
  2. Pass 1b: register functions, impl blocks, constants, externs
  3. Pass 2: check function bodies (and impl method bodies); imported items get silent-suppress
  4. Totality gate (Phase 4c.2): non-partial functions must be panic-free
- Error: `vais_types::TypeError` — see §1 registry. `error_code()` returns the `E***` string.

### Stage 4 — Codegen

- Entry: `vais_codegen::generate_ir(module: &Module, types: &TypeChecker) -> CodegenResult<String>` (writes LLVM IR text)
- Inkwell-based backend under `crates/vais-codegen/src/inkwell/`
- Method dispatch for built-in types is split between:
  - Type inference: `vais_codegen::type_inference::infer_method_call` (return type)
  - Codegen emission: `vais_codegen::string_ops` (Str methods), `vais_codegen::expr_helpers_data` (field/index), `vais_codegen::inkwell::*` (struct/vec/enum)
- Runtime declarations: `vais_codegen::string_ops::declare_runtime_helpers` emits `declare { i8*, i64 } @__vais_str_*` etc.
- Errors: `vais_codegen::error::CodegenError` — see §1 registry.

### Stage 5 — Link

- Driven by `crates/vaisc/src/commands/build.rs`. After IR generation, it invokes `clang <out.ll> -o <out> <runtime_objs>`.
- Runtime: `/tmp/test_runtime.o` (compiled once from `runtime/*.c`).
- Target triple controlled by `--target`. Native is default.

### Stage 6 — Run

- No dedicated module. Examples and integration tests call the binary directly.
- `$VAISC run <file>` is a convenience: build + exec in one command.

---

## 10. Next: Phase 0.2 — integrity test matrix

With this contract in place, Phase 0.2 builds `crates/vaisc/tests/integrity/` that mechanically implements the checks in §3 across the files listed in §4. Phase 0.3 runs them once to produce the official baseline.

Phase 0.4 wires the matrix into a CI-friendly `cargo integrity` alias that must exit 0 on the committed baseline and exit 1 on any regression.

Downstream phases (1.x onward) extend specific sub-tables and rely on the invariants defined here. Keep this document evergreen.
