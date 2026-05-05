# Step 13 (A4 removals) findings

This file records empirical findings during Order step 13 (A4 removal queue).
Mirrors STEP7 / STEP10 / STEP11 / STEP17 structure.

Master Plan v24 §[phase.A4] inventory (7 entries total):
- runtime_silent (5): A4-01 / A4-02 / A4-04 / A4-06 / A4-13
- late_codegen_silent (2): A4-08 / A4-09

Step 2 LANDED `vaisc fix --explicit --site=A4-NN [--dry-run]` skeleton.
A4-01 detection is implemented; A4-02..A4-09 stubbed `NotImplemented`.

## Index (F-NN → 한 줄 요약)

| ID | 한 줄 요약 |
|---|---|
| F-13-01 | A4-01 (Unit ↔ i64) baseline scan: 0 findings across 404 .vais files (2026-05-05) |

## Findings

### F-13-01 — A4-01 baseline scan: 0 findings (2026-05-05)

Scope: every `.vais` file in
- `compiler/std/`
- `lang/packages/vaisdb/src/`
- `lang/packages/vais-server/src/`
- `docs/language/LIVING_SPEC/`

Total scanned: 404 files.

Command per file:
```bash
./target/release/vaisc fix --explicit --site=A4-01 --dry-run "$f"
```

Result: **0 files report an A4-01 finding.**

A4-01 codemod (`check_a4_01` in `crates/vaisc/src/commands/fix/explicit.rs`)
detects `<ident>: i64 = <call_to_void_fn>()` bindings. The Stage 0 heuristic
matches a let with explicit `i64` annotation whose RHS is a bare `Call` to
a function whose declaration has no return type.

Implications:

1. **The current baseline does not exercise A4-01 at all.** The runtime-silent
   surface (master-plan v24 [[phase.A4.runtime_silent]] A4-01) is a real type
   checker hole, but no fixture, std module, or downstream package writes the
   pattern.

2. **A4-01 codemod fix transformation (Stage 1) is not blocking.** Step 13
   stage 1 for A4-01 was scoped as "detect → emit fix → migrate baseline"; the
   migrate step is empty work. Stage 1 for A4-01 can ship as a no-op fix
   transformation alongside the hard-block flip in the type checker, since
   there is nothing to migrate.

3. **Hard-block ordering**: with baseline migration cost = 0, A4-01 hard-block
   can land in a single commit (type checker reject + permanent fixture flip
   from `check_succeeds`-style v1 sentinel to `check_fails`). Estimated risk:
   low (no baseline impact, only user code that intentionally triggers the
   silent surface).

4. **Recommendation**: defer A4-01 hard-block to a small-batch lifecycle
   (lightweight 3-stage path per master-plan v24 §lifecycle.lightweight). Detect
   runtime probe + flip + permanent negative fixture. Budget: ≤ 1 file in
   compiler src/, ≤ 50 LOC.

Status: A4-01 baseline impact RECONNAISSANCE LANDED. Stage 1 transformation
work for A4-01 reduces to "no-op codemod + lightweight hard-block".
Other A4 entries (A4-02 / A4-04 / A4-06 / A4-08 / A4-09 / A4-13) need
their own per-site baseline scans to determine migration cost — those are
blocked on Stage 0 detector implementations (currently `NotImplemented`).

### F-13-02 — Master plan A4 inventory drift (2026-05-05)

While preparing A4-01 stage 1 hard-block, source-level grep showed the
A4 strict-mode rejection is already implemented for **most** A4 entries
in `crates/vais-types/src/inference/unification.rs`:

| ID | Compiler default mode | Master plan v24 inventory state |
|---|---|---|
| A4-01 | **strict reject** (env opt-out: `VAIS_REJECT_A4_01=0`) | runtime_silent (stale) |
| A4-02 | **strict reject** (env opt-out: `VAIS_REJECT_A4_02=0`) | runtime_silent (stale) |
| A4-04 | **strict reject** (env opt-out: `VAIS_REJECT_A4_04=0`) | runtime_silent (stale) |
| A4-06 | **strict reject** (env opt-out: `VAIS_REJECT_A4_06=0`) | runtime_silent (stale) |
| A4-08 | **strict reject** (env opt-out: `VAIS_REJECT_A4_08=0`) | late_codegen_silent (stale) |
| A4-09 | **strict reject** (env opt-out: `VAIS_REJECT_A4_09=0`) | late_codegen_silent (stale) |
| A4-13 | silent accept | runtime_silent (correct, this session 2026-05-05) |
| A4-03 | opt-in strict (env opt-in: `VAIS_REJECT_A4_03=1`) | controlled (v17 reclass, correct) |
| A4-05 | opt-in strict (env opt-in: `VAIS_REJECT_A4_05=1`) | controlled (v17 reclass, correct) |
| A4-07 | opt-in strict (env opt-in: `VAIS_REJECT_A4_07=1`) | controlled (v17 reclass, correct) |

Implications:

1. The master-plan v24 [[phase.A4]] runtime_silent / late_codegen_silent
   listings describe the *historical* silent state that the v1 sentinels
   discovered in 2026-05-03. The compiler has since hard-blocked the
   silent path for A4-01 / A4-02 / A4-04 / A4-06 / A4-08 / A4-09 (with
   env-gated escape hatch retained for backward compatibility).
2. Permanent fixtures already exist for all A4 entries plus A4-10 / A4-11.
   A4-10 (struct partial-init) and A4-11 (try in non-Result) are present
   in `compiler/tests/empirical/A4/` but **not** in master-plan v24
   inventory at all — additional drift.
3. Step 13 stage 1 work for A4-01..A4-09 is therefore **mostly retrospective
   inventory cleanup** rather than new compiler work. The remaining
   compiler-side gap is A4-13 (silent today, blocked on Box constructor for
   runtime probe) and the A4-10 / A4-11 inventory bring-in.

Recommendation: a follow-up session should re-audit A4 entries empirically
(probe each, observe default-mode behavior, classify as
strict-default / opt-in-strict / silent / removed) and rewrite master-plan
[[phase.A4]] to match observed reality. Compiler default behavior is in
fact ahead of the inventory document — this is the opposite of the more
common "doc ahead of reality" drift, but equally a problem because it
makes the inventory a poor planning tool for future work.

Status: Step 13 inventory drift identified. Inventory rewrite is a
multi-iter master-plan revision (would land as v25 or later). No compiler
change in this finding; informational only.
