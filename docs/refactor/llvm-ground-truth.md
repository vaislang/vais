# Refactor Design: `llvm_type_of` Ground-Truth

**Status**: Draft — pending review
**Author**: Phase 17.H4 iter 25 (2026-04-24)
**Scope**: `crates/vais-codegen/src/` — SSA type registry, `llvm_type_of`, and 34 downstream consumer sites
**Motivation**: Unblock vaisdb 15/15 link+run after 3 consecutive single-site fix regressions (iter 22/23/24)

---

## 1. Problem Statement

### 1.1 Current behavior

`llvm_type_of(val: &str) -> String` reads the **registered `ResolvedType`** of an SSA value from
`FunctionContext.temp_var_types: HashMap<String, ResolvedType>` and projects it through
`type_to_llvm()` to an LLVM type string. It does NOT reflect the actual LLVM type of the
instruction that produced the value.

Definition (`crates/vais-codegen/src/types/coercion.rs:20-63`):

```rust
pub(crate) fn llvm_type_of_checked(&self, val: &str) -> Option<String> {
    if let Some(ty) = self.fn_ctx.get_temp_type(val) {     // ← registered ResolvedType
        return Some(self.type_to_llvm(ty));                // ← projection, not emission
    }
    let local_name = val.strip_prefix('%').unwrap_or(val);
    if let Some(local) = self.fn_ctx.locals.get(local_name) {
        return Some(self.type_to_llvm(&local.ty));
    }
    // ... literal inspection + "i64" fallback
}
```

### 1.2 Why this is wrong

An SSA value's actual LLVM type is determined by the instruction that emitted it:

| Instruction                                   | Result LLVM type |
|-----------------------------------------------|------------------|
| `%t = ptrtoint %Struct* %p to i64`            | `i64`            |
| `%t = add i64 %a, %b`                         | `i64`            |
| `%t = load %Vec$u8, %Vec$u8* %p`              | `%Vec$u8`        |
| `%t = call {i8*, i64} @fn(...)`               | `{i8*, i64}`     |
| `%t = trunc i64 %x to i32`                    | `i32`            |

But the registry records whatever the caller of `register_temp_type` chose — often derived from
the *AST* (`infer_expr_type`) or the *expected* result type, not the emitted instruction.

The most egregious site (`generate_expr/mod.rs:298`):

```rust
if let Ok((ref val, _)) = result {
    if val.starts_with('%') && self.fn_ctx.get_temp_type(val).is_none() {
        self.fn_ctx.register_temp_type(val, inferred_type);  // AST inference, not emission
    }
}
```

This catch-all registers an `inferred_type` from AST walk, which can be `%Vec$u8` even though
the actual emission was `add i64`. When `llvm_type_of` is later queried, it reports `%Vec$u8`,
and 34 downstream consumers make wrong coerce decisions.

### 1.3 Cascade evidence

Three consecutive iterations tried to fix one consumer or one registration site at a time:

| iter | Approach                                              | Site                                          | 6-run avg error delta | Outcome  |
|------|-------------------------------------------------------|-----------------------------------------------|-----------------------|----------|
| 22   | AST-shape guard on `%Struct` ptrtoint branch          | `generate_expr_call.rs:692` (consumer)        | +13                   | Reverted |
| 23   | Skip Binary/Unary+Named at catch-all registration     | `generate_expr/mod.rs:~310` (registration)    | +9.5                  | Reverted |
| 24   | int-width phi coerce at match arm block               | `match_gen.rs:62-99`   (consumer)             | +34, linked 1→0       | Reverted |

Pattern: the **specific targeted site** was verified fixed in each case. Regression came from
**other sites** that were implicitly relying on the same misaligned registry for their own
(coincidentally correct) compensations.

This confirms the structural claim: the system contains a **compensation chain** where every
consumer compensates for upstream inaccuracy; removing one compensation exposes the inaccuracy
elsewhere. A single-site fix is mathematically impossible while the ground-truth gap persists.

### 1.4 Scope of affected code

- **45 registration sites** (`register_temp_type(...)`): spread across 10 files.
  - 1 catch-all (`generate_expr/mod.rs:298`) ← largest liability
  - 12 PHI/control-flow
  - 8 call/method-result
  - 8 field/element access
  - 7 assignment/binding
  - 6 control/interpolation
  - 3 other
- **34 consumer sites** (`llvm_type_of(...)`): spread across 11 files.
  - Decision patterns: `ptrtoint` vs pass-through, `icmp` vs identity, int-width `trunc`/`sext`,
    fat-pointer extract, struct-pointer bitcast, PHI incoming coerce.
- **Backend**: text IR via `write_ir!` macro → `String` buffers. No inkwell. No `BasicValueEnum`
  carrying its own type. The IR string itself is the only post-emission type witness.

---

## 2. Design Goal

**Establish `llvm_type_of` as the emission-facing ground truth**: when an instruction emits
`%tN`, the function that records its type MUST record what LLVM will actually see, not what
the AST/TC thinks should be there.

### 2.1 Non-goals

- Do not re-architect the IR backend (text → inkwell). Out of scope.
- Do not attempt to eliminate `ResolvedType` from codegen. It remains the authoritative semantic
  type. The refactor adds a **parallel, emission-facing track** without displacing it.
- Do not touch TC (`vais-types`). This is a pure codegen refactor.

### 2.2 Success criteria

- After migration, `llvm_type_of(val)` returns the exact LLVM type string that the emitting
  instruction of `val` produced.
- vaisdb baseline: `cargo test -p vais-codegen --lib` 796/796 maintained at every
  intermediate commit of the migration.
- vaisdb baseline: 15/15 standalone codegen (strict multi-module force-rebuild).
- vaisdb 6-run avg link errors trend monotonically ↓ (no regression permitted).
- At least one iter during migration unblocks a new linked test (1/15 → 2/15+).

---

## 3. Options Analysis

### Option A — Parse emitted IR strings to extract `%tN` types

**Sketch**: maintain an IR parser invoked after each `write_ir!` that scans for `%t = <op>`
patterns and derives the LLVM result type from the opcode+operands.

**Pros**:
- Ground truth by construction — the IR string IS the source.
- No new discipline required at emission sites.

**Cons**:
- Requires an LLVM IR parser (partial grammar). Complexity: high.
- Per-emission parse cost. Not prohibitive but noisy.
- Error-prone on edge cases (metadata, attributes, inline comments).
- Violates "no rearchitecture" intent — effectively a shadow backend.

**Verdict**: REJECTED. Cost/benefit unfavorable; we have 45 emission sites that can self-report
more cheaply than a parser can reconstruct.

### Option B — Parallel `actual_llvm_type: HashMap<String, String>`

**Sketch**: add a second registry indexed by SSA name, populated at emission time with the
actual LLVM type string. `llvm_type_of` reads from this track.

**Pros**:
- Ground truth IS recorded at the site that knows it (the emitter).
- Discipline requirement localized: every `write_ir!` that assigns `%tN` must also record.
- Cheap — one string insert per emit.
- Allows incremental migration: both tracks coexist; consumer can read new track with fallback
  to old, and migration advances as emission sites opt in.

**Cons**:
- 45+ emission sites to update. High mechanical touch.
- Discipline drift risk — new emission sites added without corresponding record call → silent
  regression.
- Requires a lint or macro gate to enforce.

**Verdict**: **PREFERRED** — balances cost and correctness. Discipline risk mitigated via
`write_ir_typed!` macro (see §5).

### Option C — Restrict consumers to signature-directed coercion only

**Sketch**: instead of having consumers ask `llvm_type_of(val)` and decide coerce by its
answer, pass the target type through explicitly at call sites; consumers always coerce to the
declared signature without querying the SSA type.

**Pros**:
- Makes consumers self-contained. Source of truth is caller-provided, not registry.
- Aligns with a principle many compilers use: "types flow downward, not sideways."

**Cons**:
- Doesn't eliminate the need for `llvm_type_of` — coerce emit still requires knowing the
  *actual* type to choose trunc/sext/zext/bitcast.
- 34 consumer sites to refactor simultaneously. High risk of cascade.
- Preserves the underlying problem: if a consumer still has to pick `ptrtoint` vs pass-through
  based on actual type, it still needs ground truth.

**Verdict**: REJECTED as primary path; may partially apply as a **complementary cleanup** after
Option B lands (reducing the 34 consumer set by removing consumers that only need signature info).

---

## 4. Chosen Approach — Option B (detailed)

### 4.1 Data structure change

`crates/vais-codegen/src/state.rs`:

```rust
pub(crate) struct FunctionContext {
    // existing
    pub(crate) temp_var_types: HashMap<String, ResolvedType>,

    // NEW: actual LLVM type emitted for each SSA temp.
    // Invariant: if a write_ir!-style emission produces "%tN = ... : T", then
    // actual_llvm_type["%tN"] == T (as LLVM type string).
    pub(crate) actual_llvm_type: HashMap<String, String>,
}

impl FunctionContext {
    pub(crate) fn record_emitted_type(&mut self, name: &str, llvm_ty: &str) {
        self.actual_llvm_type.insert(name.to_string(), llvm_ty.to_string());
    }

    pub(crate) fn get_emitted_type(&self, name: &str) -> Option<&str> {
        self.actual_llvm_type.get(name).map(String::as_str)
    }
}
```

### 4.2 `llvm_type_of` resolution order (post-migration)

```rust
pub(crate) fn llvm_type_of_checked(&self, val: &str) -> Option<String> {
    // 1. Emission ground truth — FIRST (new)
    if let Some(actual) = self.fn_ctx.get_emitted_type(val) {
        return Some(actual.to_string());
    }

    // 2. Registered ResolvedType projection — FALLBACK (kept during migration)
    if let Some(ty) = self.fn_ctx.get_temp_type(val) {
        return Some(self.type_to_llvm(ty));
    }

    // 3. Local variable
    let local_name = val.strip_prefix('%').unwrap_or(val);
    if let Some(local) = self.fn_ctx.locals.get(local_name) {
        return Some(self.type_to_llvm(&local.ty));
    }

    // 4. Literal inspection (existing)
    // ...
}
```

During the migration, sites that do NOT yet record via `record_emitted_type` still resolve
through the ResolvedType track. This is the key property that allows incremental rollout
without breaking tests.

### 4.3 Migration unit: per emission site

Each emission site is an **atomic migration unit**. "Migrated" means: after every
`write_ir!(ir, "  {} = <op> ...", name, ...)` that defines an SSA temp, there is a
corresponding `self.fn_ctx.record_emitted_type(name, "<actual LLVM type>")` call.

Enforcement aid — a macro that combines both:

```rust
macro_rules! write_ir_typed {
    ($self:expr, $ir:expr, $name:expr, $llvm_ty:expr, $($arg:tt)*) => {{
        use std::fmt::Write as _;
        let _ = writeln!($ir, $($arg)*);
        $self.fn_ctx.record_emitted_type($name, $llvm_ty);
    }};
}
```

Usage:
```rust
write_ir_typed!(self, ir, &tmp, "i64", "  {} = ptrtoint {}* {} to i64", tmp, val_ty, val);
```

Once an emission site adopts `write_ir_typed!`, its `%tN` is guaranteed to have a ground-truth
entry in `actual_llvm_type` before any consumer can read it.

### 4.4 Migration order

Prioritize by **consumer pressure** × **ease of conversion**:

**Wave 1 — high-pressure primitives (expected biggest signal)**:
1. `ptrtoint` emissions (all sites) → always `i64` result.
2. `trunc`/`sext`/`zext`/`fptosi`/`sitofp` emissions → result type trivially readable.
3. `icmp`/`fcmp` → always `i1`.
4. `add`/`sub`/`mul`/`and`/`or`/`xor`/`shl`/`lshr`/`ashr` → result type == operand type.

These are ~40-60% of emissions by count and dominate the cascade surface area.

**Wave 2 — composite ops**:
5. `getelementptr` → pointer to element type (requires struct table lookup).
6. `load` → pointee type (available in the instruction, just needs capture).
7. `call` → callee signature return type (requires function registry).
8. `alloca` → pointer to allocated type.

**Wave 3 — aggregate & PHI**:
9. `phi` → incoming type (must be consistent per SSA semantics).
10. `extractvalue`/`insertvalue`/`extractelement`/`insertelement`.
11. `bitcast`/`addrspacecast` → target type from instruction.

**Wave 4 — remove the catch-all**:
12. After Waves 1-3 cover ≥90% of emission sites, `generate_expr/mod.rs:298` catch-all becomes
    **dead weight** for migrated sites. Delete it. Any remaining reliance surfaces as explicit
    registration failures.

**Wave 5 — clean up consumers**:
13. Audit the 34 consumer sites. Sites that were compensating for the old registry misalignment
    can simplify. Some may be deletable entirely.

### 4.5 Verification gates

After each Wave:
- `cargo test -p vais-codegen --lib` → 796/796 required.
- `cargo test -p vais-types --lib` → 355/355 required.
- vaisdb 15-test suite standalone codegen: 15/15 required.
- vaisdb 6-run avg link errors: **must not increase**. Record baseline per Wave in ROADMAP.
- linked count (currently 1/15): must not decrease.

Any Wave failing any gate → revert that Wave only, investigate, retry. No stacking.

### 4.6 Estimated effort

- Wave 1 (primitives): 1 session. ~15-25 sites. Mechanical.
- Wave 2 (composite): 1-2 sessions. ~10-15 sites. Some require lookups (call signature, struct layout).
- Wave 3 (aggregate+phi): 1 session. ~8-10 sites. PHI coerce sites are the subtle ones.
- Wave 4 (catch-all removal): 0.5 session. Delete + run tests.
- Wave 5 (consumer cleanup): 1 session. Signature-directed migration for sites that no longer need `llvm_type_of`.

**Total**: 4-5 sessions. Budget allowance: +1 session for unexpected coerce cascade cleanup.

---

## 5. Risk Register

| Risk | Mitigation |
|------|------------|
| Emission site added without `record_emitted_type` call | Introduce `write_ir_typed!` macro; lint CI for raw `write_ir!` on `%tN =` patterns in post-migration code. |
| Wave N breaks vaisdb linked count | Per-Wave verification gate; revert on any regression. No stacking. |
| Ground-truth record disagrees with old registry for same SSA | During migration, new track wins. Old mismatches were the bug; they're now visible and fixable. |
| Consumer relies on old track's *wrong* answer "coincidentally correctly" | Expected. Waves 1-3 will surface these as new errors. Wave 5 cleans them up after the ground-truth signal is stable. |
| Session-level interleave (someone else edits during migration) | Short Wave chunks; commit after each Wave. |
| Catch-all removal (Wave 4) exposes Waves 1-3 coverage gap | Gate Wave 4 on coverage audit: count sites that still lack `record_emitted_type`; require ≥90%. |

---

## 6. Rollback Plan

Each Wave is a standalone commit. If Wave N breaks an invariant:
- `git revert <wave-N-commit>` — safe because waves are additive (only add new records + reads).
- Old ResolvedType track remains fully intact throughout migration; reverting a Wave simply
  removes the new records, resolution falls back to the old path.
- Catch-all (Wave 4) removal is the ONLY destructive change — it must land last and is
  independently revertable.

---

## 7. Alternatives Considered and Deferred

### 7.1 Eliminate `temp_var_types` entirely

Attractive in principle — one registry only. Deferred because:
- `ResolvedType` carries information (generic instantiation, enum variant layout) that isn't
  recoverable from the LLVM type string alone. Consumers like `call_gen.rs:105` use it for
  memcpy size computation.
- Two-track model lets us migrate incrementally without a flag day.

### 7.2 Move to inkwell / `BasicValueEnum`

Correct long-term architecture. Deferred because:
- Rewrite scope is 10-100x larger than this refactor.
- Would block Phase 18/19 indefinitely.
- Option B achieves the immediate goal (ground truth) without touching the backend.

### 7.3 Push types through AST via explicit annotations

Defers the problem to TC. Out of scope for codegen refactor. Could be a future Phase 18 I3
(conformance spec) concern.

---

## 8. Exit Criteria

The refactor is considered complete when:

1. `generate_expr/mod.rs:298` catch-all registration is **deleted**, and tests pass.
2. All 45 emission sites either (a) use `write_ir_typed!` or (b) explicitly document why they
   don't (e.g., void instructions that don't produce an SSA value).
3. `llvm_type_of` resolution-order comment reflects ground-truth-first policy.
4. vaisdb linked count improves from 1/15 to ≥5/15 (leading indicator; not a hard gate).
5. A new `golden IR` test (Phase 18.I1 prereq) catches a deliberate emission-site omission.

---

## 9. Open Questions (for reviewer)

1. Is the `write_ir_typed!` macro acceptable, or prefer a `CodeGen` method like
   `self.emit(ir, name, ty, fmt_args...)` that wraps both?
2. Should the `actual_llvm_type` HashMap be per-function (current `FunctionContext`) or
   per-module? Per-function matches SSA scope but loses info across inlined boundaries.
3. Wave 4 gates on "≥90% coverage" — should this be ≥100%? Strict is safer; 90% lets the
   catch-all die before truly-terminal leftovers (which may be in error paths) migrate.
4. Do we want a `debug_assert!` on `llvm_type_of` that flags disagreement between the two
   tracks during the migration (both return a value but they differ)? Would surface
   previously-silent compensations as loud panics in debug builds.

---

## 10. References

- `MEMORY.md` entries:
  - `phase17_3_negatives_escalation.md` — cascade analysis, why single-site fixes fail
  - `phase17_iter22_23_ptrtoint_cascade.md` — iter 22/23 detail
  - `phase17_iter22_ast_guard_wrong.md` — iter 22 detail
- `ROADMAP.md` Active Phase iter 20-24 — empirical evidence of the cascade
- `crates/vais-codegen/src/types/coercion.rs:20-63` — `llvm_type_of` current implementation
- `crates/vais-codegen/src/state.rs:135-197` — `FunctionContext` and `temp_var_types`
- `crates/vais-codegen/src/generate_expr/mod.rs:298` — catch-all registration (the root liability)
