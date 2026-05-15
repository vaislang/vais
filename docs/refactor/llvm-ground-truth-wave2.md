# Refactor Design: Wave 2 — Composite Emission Ground-Truth

**Status**: Draft — pending review
**Author**: Phase 17.H4 iter 33 (2026-04-24)
**Parent doc**: [llvm-ground-truth.md](./llvm-ground-truth.md) (Wave 1-5 master plan)
**Scope**: `crates/vais-codegen/src/` — `load`, `call`, `getelementptr`, `alloca` emission sites
**Prerequisite**: Wave 1a (infrastructure), Wave 1b (ptrtoint), Wave 1c.1~1c.5 (trunc/sext/zext/icmp/fcmp) — all landed (commits `0aec7bd8`, `788cffde`, `8b9814a6`, `f6a44a3c`, `14bc417f`, `95c23fe5`, `115c3f5b`)

---

## 1. Status After Wave 1

### 1.1 Cumulative migration state

After Wave 1c.5 (commit `115c3f5b`):

- **99 sites migrated** to ground-truth track (`actual_llvm_type`).
- Wave 1 covered scalar primitives: `ptrtoint`, `trunc`, `sext`, `zext`, `icmp`, `fcmp`.
- All Wave 1 gates held: cargo 796/796 + 355/355, vaisdb codegen 13-15/15 (flake band), linked 1/15 hold.
- Per-Wave link error trajectory (4-run avg): 164 → 159.5 → 166.5 → 164.75 → 165.75 → 157.5 → 157.
- **Wave 1c.5 cascade**: 5 width-coerce sites (method_call zext/sext, expr_helpers_assign trunc/sext closure, function_gen ret-trunc) caused +18.5 errors when migrated; reverted. Cause confirmed identical to memory `phase17_iter22_23_ptrtoint_cascade`: recording narrow types (i8/i16/i32) breaks consumers that depend on the i64-default fallback.

### 1.2 Implication for Wave 2

The Wave 1c.5 cascade is the **first concrete evidence** that ground-truth migration is not unconditionally safe. Width-coerce results live in a regime where downstream consumers were silently relying on the default-i64 lie. Wave 2 sites (load/call/gep/alloca) span a **much larger** consumer surface — **277 emission sites**, vs. ~107 in Wave 1c. Each migrated load/call carries the same cascade risk if downstream consumers depend on the previous (often `i64`) default.

This document plans Wave 2 with that risk made explicit, batching by consumer-pressure homogeneity rather than by sheer site count.

---

## 2. Site Inventory

Counts via `rg --multiline -c 'write_ir!\([^)]*"\s*\{\}\s*=\s*<op>'` over `crates/vais-codegen/src/`:

| Instruction      | Sites | Notes |
|------------------|-------|-------|
| `load`           | 133   | Largest. Result type == pointee type, available in IR string itself. |
| `call`           | 54    | Result type from callee signature. Generic specialization complicates lookup. |
| `getelementptr`  | 76    | Result is pointer to element type. Requires struct/element layout lookup. |
| `alloca`         | 14    | Result type == pointer to allocated type. Most localized. |
| **Wave 2 total** | **277** | |

Plus **~138 raw `ir.push_str("  %tN = <op> ...")` sites** (not via `write_ir!`), concentrated in:
- `function_gen/runtime.rs` (87) — runtime helper extern bodies, no per-function SSA scope (out of `actual_llvm_type` purview)
- `string_ops.rs` (40) — `generate_struct_shallow_free_helper` emitted-helper IR (similar — emits into a separate `String` for a separately-defined `__vais_struct_shallow_free_*` LLVM function)
- `function_gen/async_gen.rs` (6) — async poll function body
- Other (5)

**Decision**: raw `push_str` sites are deferred to Wave 2.x audit. Most are in helper IR strings that are emitted as standalone LLVM function definitions (their SSA names are scoped inside that emitted function, not the caller's `FunctionContext`). Only the in-scope ones (`async_gen.rs` body emissions inside `generate_async_function`) need tracking; these can be migrated as part of Wave 2c (load batch) if relevant.

---

## 3. Per-Instruction Ground-Truth Rules

For each Wave 2 instruction, the ground-truth LLVM type recorded for `%tN` is:

### 3.1 `alloca` — Wave 2a

```llvm
%tN = alloca <T>           → record_emitted_type(name, "<T>*")
%tN = alloca <T>, i64 %n   → record_emitted_type(name, "<T>*")
```

The result is **always a pointer** to the allocated type. The allocated type is the literal first operand of `alloca` and is statically present at every emission site. Most sites use a `String` variable holding the `<T>` form, e.g.:

```rust
write_ir!(ir, "  {} = alloca {}", tmp, struct_ty);
self.fn_ctx.record_emitted_type(&tmp, &format!("{}*", struct_ty));
```

**Risk**: Low. `alloca`-produced names are usually consumed via `store`/`load`/`gep` whose own type discipline is local. No prior compensation chain expected.

### 3.2 `getelementptr` — Wave 2b

```llvm
%tN = getelementptr <BaseT>, <BaseT>* %p, i32 0, i32 K
  → record_emitted_type(name, "<FieldKType>*")
```

**Determination requires struct/element layout lookup**:
- For `getelementptr inbounds <Vec>, <Vec>* %v, i32 0, i32 0` (e.g., `data` field), the result type depends on Vec's struct layout (likely `i8**`).
- For pointer arithmetic `getelementptr i8, i8* %p, i64 %off`, the result type is `i8*`.
- For array indexing `getelementptr [N x T], [N x T]* %p, i64 0, i64 %i`, the result is `T*`.

**Implementation approach**:
- Most `gep` sites already compute the result type to embed in subsequent `load`/`store` instructions. **Reuse that information**: extract a `result_ty` local at each site and pass it both into `write_ir!` and `record_emitted_type`.
- For sites where the result type isn't currently named (computed inline), introduce a `let result_ty = ...` binding before the emission.

**Risk**: Medium. Many gep results feed into `load`/`store` which currently re-derive type via `llvm_type_of` on the pointer base. Migrating gep changes what those consumers see. Sample audit needed before batch migration (see §5).

### 3.3 `load` — Wave 2c

```llvm
%tN = load <T>, <T>* %p
  → record_emitted_type(name, "<T>")
```

The pointee type is the literal first operand of `load`. Direct extraction:

```rust
write_ir!(ir, "  {} = load {}, {}* {}", tmp, ty, ty, ptr);
self.fn_ctx.record_emitted_type(&tmp, ty);
```

**Risk**: Medium-high. `load` is the largest single category (133 sites) and produces values that flow into nearly every downstream consumer — arithmetic, calls, returns, stores. The cascade surface is the broadest of any Wave 2 instruction. **Batch by category, not by file** (see §5.3).

### 3.4 `call` — Wave 2d

```llvm
%tN = call <RetT> @fname(...)
  → record_emitted_type(name, "<RetT>")
```

**Determination via**:
1. Literal `<RetT>` in the IR string — present at the emission site as a variable holding the return type.
2. Callee signature lookup via `resolved_function_sigs` / `expr_types` (already used by Phase 17.H4 iter 20 fallback).

**Caveats**:
- Generic specialization: `Vec_new$T` — return type depends on T.
- Cross-module Vais fn calls vs C-extern: Wave 1b/iter 18 already handled extern ABI declare/call mismatch. The actual return type at the call site is the same in both cases (the IR-emitted text type).
- Unregistered builtins (Phase 17.H4 iter 19/20 lesson): `Str_new`, `to_vec` etc. previously fell back to `i64`; iter 20 added TC `expr_types` fallback. Wave 2d should record the **actually emitted** call return type, regardless of which fallback path produced it.

**Risk**: Highest. Every prior cascade event (iter 22, 23, 24, 1c.5) involved either a call site or a width-coerce-after-call. Wave 2d MUST follow Wave 2a/2b/2c so that intervening loads/geps already provide ground truth.

---

## 4. Known Hazards

### 4.1 Width-coerce cascade pattern (Wave 1c.5)

**Pattern**: Recording the *correct* narrow type (e.g., `i32` for a `trunc i64→i32`) breaks consumers that previously got the i64 default and emitted compensating coerces. The compensation now produces wrong-width arithmetic or wrong-store-width.

**Wave 2 application**: Loads of narrow types (i8/i16/i32 fields) pose the same risk. Compare:

```llvm
; Before Wave 2c — load on a u8 field
%t1 = load i8, i8* %field_ptr
; consumer (binary op) currently does:
%t2 = add i64 %t1, 1     ; ← INVALID IR if we trust llvm_type_of(%t1) = i8
                          ; current code "luckily works" because llvm_type_of returns i64 default
                          ;  → consumer emits add i64 against an i8 value, which IS an LLVM error,
                          ;  → BUT the emission probably has an upstream zext that makes %t1 actually i64
                          ;  → OR the consumer reads %t1's registered ResolvedType and emits its OWN zext
```

**Mitigation**:
- For load Wave 2c, segregate **wide-type loads** (i64, double, struct, pointer) from **narrow-type loads** (i8, i16, i32, float). Migrate wide first; narrow second with consumer audit.
- For each narrow-type load, grep for downstream consumers that access `%tN` as `i64` operand. If found, audit before migrating.

### 4.2 Composite type discipline (load/gep results)

A load/gep producing `%Vec$u8` or `{i8*, i64}` puts a struct value or fat pointer into ground-truth track. Consumers like `extractvalue` need this to be exactly the struct type (not `i64`).

**Verification approach**: After each Wave 2x batch, sample-check that newly migrated sites' downstream uses see the correct type (find one site, trace its consumer, verify IR is valid).

### 4.3 Unscoped emission (helper IR strings)

`generate_struct_shallow_free_helper` (string_ops.rs:1072) emits into a separate `String` that becomes a standalone `__vais_struct_shallow_free_*` LLVM function definition. SSA names there (`%mask`, `%bit_and_*`, `%fat_*`) are scoped inside that emitted function, NOT the calling function's `FunctionContext`. Recording them in the caller's `actual_llvm_type` would pollute the wrong scope.

**Decision**: Helper-IR sites stay out of Wave 2. They're already excluded from Wave 4 catch-all removal coverage by the same logic. Document the exclusion in Wave 4 design.

### 4.4 Cross-module specialization (call Wave 2d)

Generic `Vec_new$T` call site emits a return type that depends on T. The local `String` variable at the emission site is correct. But cross-module imports of the same template may have a stale or different `T` cached in `resolved_function_sigs`.

**Decision**: Use the **literal IR-string return type** at the emission site as the ground truth, NOT the registry lookup. The IR string is what LLVM will see; that's the definition of ground truth.

---

## 5. Migration Plan

### 5.1 Wave order (risk-ascending)

| Sub-wave | Instruction | Sites | Difficulty | Expected error delta |
|----------|-------------|-------|------------|----------------------|
| 2a       | `alloca`    | 14    | Low        | Flat to slight ↓     |
| 2b       | `getelementptr` | 76 | Medium    | Slight ↓ (gep-derived loads currently re-derive incorrectly) |
| 2c.1     | `load` (wide types)   | ~70 (est) | Medium | Moderate ↓ (struct loads currently default to i64) |
| 2c.2     | `load` (narrow types) | ~63 (est) | Medium-high | Risk: cascade. Audit per file. |
| 2d       | `call`      | 54    | High       | Risk: cascade. Combined with TC fallback iter 20. |

**Estimated sessions**: 2a (1), 2b (1-2), 2c.1 (1), 2c.2 (1-2 with audit), 2d (1-2). Total **6-8 sessions**.

### 5.2 Per-batch protocol

Each batch (Wave 2x.k) follows the same rhythm as Wave 1c.x:
1. Pick a file or coherent file group.
2. Edit each emission site: add `record_emitted_type(name, &actual_ty)` after the `write_ir!`.
3. Build vaisc.
4. Run 4-run gate via `/tmp/wave1c5_gate.sh` (reuse, rename per Wave).
5. Verify cargo 796/796 + 355/355.
6. Compare 4-run avg to previous Wave's avg. **Hard gate: must not increase by more than +5 errors** (Wave 1c noise band has been ±7).
7. If +5 or more: bisect within the batch (revert half, re-test, narrow). Identify cascade-trigger sites; defer them with a `// Wave 2x.k deferred — cascade trigger` comment + ROADMAP note.
8. Commit with format: `feat(codegen): Phase 17.H4 Wave 2x.k — record_emitted_type for <op> in <files> (N sites)`.

### 5.3 Wave 2c.2 (narrow load) special protocol

Because Wave 1c.5 demonstrated that narrow-type ground truth cascades, Wave 2c.2 requires consumer audit BEFORE migration:

For each narrow-type load site:
1. Identify the SSA name `%tN`.
2. `rg "%tN" <output IR file>` after a baseline build.
3. Classify each consumer:
   - `add/sub/mul/and/or/xor/shl <iN>` of matching width → safe, migrate.
   - `add/sub/... i64` against `%tN` (narrow-on-wide arithmetic) → cascade trigger, **defer**.
   - `store iN` matching width → safe.
   - `call ... iN` matching parameter width → safe.
   - `icmp/zext/sext` → safe (emission instruction declares its own width).
4. Migrate only sites where ALL consumers are safe. Defer cascade-trigger sites for Wave 5 cleanup (which will fix the consumers).

### 5.4 Verification gates

After each Wave 2x.k:
- **cargo test -p vais-codegen --lib**: 796/796 (hard).
- **cargo test -p vais-types --lib**: 355/355 (hard).
- **vaisdb 4-run codegen**: 13-15/15 (flake band; soft).
- **vaisdb 4-run linked**: 1/15 hold; ≥2/15 a positive signal.
- **vaisdb 4-run avg link errors**: ≤ previous Wave avg + 5 (hard for sub-wave; trend monitored).
- **Total link error noise band** (cumulative across Wave 2): allowed range {145..195} — slight expansion from Wave 1's {145..184} to accommodate composite type coverage growth.

### 5.5 Rollback policy

Same as Wave 1: each Wave 2x.k is a standalone commit. `git revert` removes only the new records, falling back to the legacy ResolvedType track. No invariants depend on the new records existing for sites NOT yet migrated.

---

## 6. Wave 4 Coverage Implication

Wave 4 (catch-all removal at `generate_expr/mod.rs:298`) was designed for ≥90% (later strict 100%) emission-site coverage. Cumulative coverage trajectory:

| Wave end | Migrated sites | Cumulative % of estimated total (~430 sites) |
|----------|----------------|-----------------------------------------------|
| 1c.5     | 99             | ~23%                                          |
| 2a       | ~113           | ~26%                                          |
| 2b       | ~189           | ~44%                                          |
| 2c.1     | ~259           | ~60%                                          |
| 2c.2     | ~322           | ~75% (some narrow-load deferred)              |
| 2d       | ~376           | ~87%                                          |
| Wave 3 done | ~430+       | ~100%                                         |

Wave 2 alone moves coverage from 23% to ~87%. After Wave 3 (phi/extract/insert/bitcast), Wave 4 can land.

**Open**: Width-coerce 5 sites (Wave 1c.5 deferred) and narrow-load deferrals from Wave 2c.2 cumulatively form a "deferred set" that Wave 5 (consumer cleanup) must address before Wave 4 can be strict-100%.

---

## 7. Open Questions (for reviewer)

1. **Wave 2c.2 audit cost**: per-site downstream consumer trace is ~5-10 minutes per site × 63 sites = several hours of audit. Acceptable given the cascade risk, OR migrate optimistically and revert per cascade event (Wave 1c.5 protocol)?
2. **Helper-IR exclusion**: `generate_struct_shallow_free_helper` and similar emit standalone LLVM functions inside the caller's IR string. Should Wave 4 strict-100% include these (requiring a new "helper FunctionContext"), or document them as permanently excluded?
3. **Cross-module call return type**: The IR string at emission site is ground truth, but after Phase 17.H4 iter 20, TC `expr_types` fallback overrides for unregistered calls. Should Wave 2d record the IR-string type or the iter 20 resolved type? **Tentative answer**: IR-string type — that's what LLVM sees. Iter 20 resolved type is upstream of emission and may differ.
4. **Coverage gate for Wave 4**: ≥87% after Wave 2, ≥95% after Wave 3. Strict 100% adds the deferred set (width 5 + narrow-load deferrals). Defer 100% target to Wave 5?
5. **Macro vs explicit method call**: Wave 1 chose explicit `self.fn_ctx.record_emitted_type(...)` (greppable, self-borrow visible). Wave 2 has 277 sites; macro would reduce mechanical noise. Trade-off: macro hides the borrow, explicit is verbose. **Tentative**: keep explicit for consistency with Wave 1; revisit if Wave 2c boilerplate becomes oppressive.

---

## 8. Exit Criteria (Wave 2)

- All 4 sub-waves (2a-2d) landed.
- Cumulative migrated sites ≥ 250 (~87% of total non-helper emission).
- vaisdb linked count ≥ 1/15 (no regression); 2-3/15 a positive signal.
- Cargo tests 796/796 + 355/355 maintained at every commit.
- Deferred set documented in ROADMAP (width 5 + Wave 2c.2 narrow-load deferrals).
- Wave 3 (phi/extract/insert/bitcast) ready to start; Wave 4 (catch-all removal) feasible after Wave 3.

---

## 9. References

- [`llvm-ground-truth.md`](./llvm-ground-truth.md) — master design doc, Waves 1-5.
- `MEMORY.md` entries:
  - `phase17_wave1_progress.md` — Wave 1 progress + Wave 1c.5 cascade lesson
  - `phase17_iter22_23_ptrtoint_cascade.md` — original cascade analysis
  - `phase17_3_negatives_escalation.md` — single-site fix policy
- ROADMAP.md iter 25-32 — empirical Wave 1 trajectory.
- Compiler commits: `0aec7bd8` (1a infra) → `115c3f5b` (1c.5).
