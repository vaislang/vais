# Step 7 retro-validation findings (first iteration, 2026-05-03)

This file records what the first iteration of Order Step 7 retro-validation
empirically discovered. It complements the per-A4 fixture directories.

## Status summary (2026-05-08, master-plan v37)

All 24 findings (F-01 through F-24) have reached terminal status. Step 7
retro-validation no longer carries open work items here. Future surface
discoveries will be filed under Step 7 next iteration in a new findings
file (e.g., STEP7_FINDINGS_2026-MM.md) or directly into master-plan
A4 inventory + per-fixture meta.toml.

| Status | Count | Findings |
|---|---|---|
| LANDED (fixture exists, surface tracked) | 14 | F-01, F-02c, F-02d, F-02e, F-06, F-09, F-10, F-12, F-13, F-14, F-15, F-19, F-20, F-23 |
| RECLASSIFIED (master-plan inventory amended) | 7 | F-02, F-02b (→ Controlled v17 via F-22), F-07 (→ A4-14 v37), F-11 (→ A4-13 via F-24), F-16 (→ Controlled v17), F-17 (→ Controlled v17), F-18 (→ A4-15 v37), F-21 (→ Controlled v17 via F-22), F-24 (→ A4-13 v24) |
| INFORMATIONAL (no fixture, no inventory change) | 3 | F-03 (directory bootstrap), F-04 (site path uniqueness), F-05 (environment-stability classification — protocol revision LANDED) |
| CLOSED (deferred at the time, resolved via other path) | 1 | F-08 (C-04/08/09 deferred Controlled probes) — C-04 resolved via A2-05 (fn-pointer params LANDED Step 9), C-08 resolved via A2-03 (dyn dispatch LANDED Step 9 v22), C-09 resolved via Step 16 master-plan v36 reclassify (Controlled-style affine annotation, see DEFERRED #21 close). |
| PROTOCOL (assertion-kind taxonomy) | 2 | F-05 (tri-form), F-14 (5th form check_fails) |

(F-22 is both reclassification target and inventory amendment — counted once under RECLASSIFIED.)

## Index (F-NN → 한 줄 요약 + status)

| ID | 한 줄 요약 | Status |
|---|---|---|
| F-01 | A4-01 Unit ↔ i64 v1 sentinel reproduces (exit 96) | LANDED |
| F-02 | A4-02 Pointer<T> ↔ i64 v1 expected drifted by environment | LANDED via exit_not protocol (F-05) |
| F-02b | A4-03 Auto-deref &T ↔ T same env drift as A4-02 | RECLASSIFIED → Controlled v17 (F-17/F-22) |
| F-02c | A4-06 Integer truthy v1 sentinel reproduces (exit 100) | LANDED + strict default v22 (F-19) |
| F-02d | A4-07 Numeric widening v1 sentinel reproduces (exit 42, runtime correct) | RECLASSIFIED → Controlled v17 (F-21/F-22) |
| F-02e | A4-09 Lifetime ref erasure v1 sentinel reproduces (linker fail) | LANDED + strict default v25 |
| F-03 | `compiler/tests/empirical/` directory did not exist before this iteration | INFORMATIONAL (bootstrap done) |
| F-04 | A4 site paths are unique by filename in the codebase | INFORMATIONAL |
| F-05 | Environment-stability classification of A4 surfaces | PROTOCOL (tri-form LANDED + 5th form added F-14) |
| F-06 | A4-08 Vec ↔ &T permissive v1 sentinel symptom drifted, surface persists | LANDED + strict default v25 |
| F-07 | Controlled-06 (Vec ↔ Slice .len() path) NOT actually controlled | RECLASSIFIED → A4-14 v37 (loop 32) |
| F-08 | Several Controlled probes fail to construct in current parser | CLOSED (C-04 → A2-05 LANDED Step 9; C-08 → A2-03 LANDED Step 9; C-09 → DEFERRED #21 close v36) |
| F-09 | Controlled fixtures LANDED this iteration (C-01 / C-02 / C-05) | LANDED |
| F-10 | Rejected-01 LANDED (Box raw generic) | LANDED |
| F-11 | Rejected-02 (Box ↔ T) v1 sentinel does NOT reproduce | RECLASSIFIED → A4-13 via F-24 (v24, DEFERRED #20 LANDED) |
| F-12 | Rejected-03 LANDED (Optional ↔ T, bare i64) | LANDED |
| F-13 | Untested-01 (Result ↔ Unit auto Ok wrap) → RECLASSIFY to Rejected | LANDED + RECLASSIFIED |
| F-14 | `check_fails` assertion kind added (5th form) | PROTOCOL (LANDED) |
| F-15 | NEW A4 candidate: struct partial-init silent acceptance | LANDED → A4-10 (v25) |
| F-16 | A4-05 Array→Pointer is structural, not user-level (2026-05-04) | RECLASSIFIED → Controlled v17 |
| F-17 | A4-03 Auto-deref &T↔T also IR-lowering glue (2026-05-04) | RECLASSIFIED → Controlled v17 |
| F-18 | Escape closure silent capture loss (NEW A4 candidate, 2026-05-04) | RECLASSIFIED → A4-15 v37 (loop 32) |
| F-19 | A4-06 strict mode emits "expected i64, found bool" + std codemod LANDED + strict default LANDED | LANDED |
| F-20 | A4-07 std codemod LANDED (Step 13 stage 0 std slice) | LANDED + RECLASSIFIED → Controlled v17 (F-22) |
| F-21 | A4-07 strict scope is broader than master-plan v16 estimated | RECLASSIFIED → Controlled v17 (F-22) |
| F-22 | A4-03 / A4-05 / A4-07 reclass to Controlled LANDED (master-plan v17) | LANDED |
| F-23 | A2-03 dyn dispatch silently calls first impl (NEW A4 candidate, A4-12) | LANDED step 1 (v17) + step 2 LANDED (v22) — A2-03 promoted Step 9 |
| F-24 | Rejected-02 (Box ↔ T) re-probe REPRODUCES silent accept (2026-05-05) | LANDED → A4-13 (v24, DEFERRED #20 LANDED) |

## Findings

### F-01 — A4-01 Unit ↔ i64: v1 sentinel reproduces (exit 96)

`probe.vais`:
```vais
F void_fn() { R }
F main() -> i64 {
    x: i64 = void_fn()
    R x
}
```

Build: macOS arm64, vaisc release build, 2026-05-03.
Result: type-checks, compiles, exits 96 — matches master-plan.toml v1 entry.

**Status: A4-01 fixture LANDED** at
`compiler/tests/empirical/A4/A4-01_unit_i64/` with v1-reverified evidence.

---

### F-02 — A4-02 Pointer<T> ↔ i64: v1 expected value drifted by environment

`probe.vais`:
```vais
F take_i64(x: i64) -> i64 { R x }
F main() -> i64 {
    val: i64 = 42
    p: *i64 = &val
    R take_i64(p)
}
```

Build: macOS arm64, vaisc release build, 2026-05-03.
Master-plan.toml v1 expected: 184.
Observed: 56 (also 40 with `&val` non-raw-pointer variant).

The probe still demonstrates the surface (Pointer ↔ i64 type-check passes
silently and runs producing a non-42 result), but the **specific exit
code is a function of LLVM's stack layout for `val`**, not a property of
the unification rule. v1 single-sentinel pinned a value that does not
generalize.

**Implication for the Empirical verification protocol v2**:
exit-code-only assertion is too tight. v2 must assert:
- (a) probe type-checks (proves the unification rule fires), AND
- (b) exit code is **not 42** (proves the rule produced wrong runtime
  semantics, regardless of the specific corruption value), AND
- (c) optionally pin the v1 specific value as informational, not as a
  blocking assertion.

This is not a v2 protocol violation discovery — the protocol allowed for
multi-sentinel evolution. F-02 supplies concrete evidence why exit-code
equality across environments was always brittle.

**Status: A4-02 fixture deferred.** It will land after the v2 assertion
shape is updated to "exit ≠ 42 (the would-have-been-correct value if
type-check rejected, by virtue of forcing the probe to be rewritten)".
That update is a small Step 7 protocol revision — separate task.

---

### F-02d — A4-07 Numeric widening: v1 sentinel reproduces (exit 42, runtime correct)

`probe.vais`:
```vais
F take_i64(x: i64) -> i64 { R x }
F main() -> i64 {
    small: i32 = 42
    R take_i64(small)
}
```

Build: macOS arm64, vaisc release build, 2026-05-03.
Master-plan.toml v1 expected: 42 (runtime correct, design pending).
Observed: 42 — exact match.

A4-07 is unique among the 9 A4 entries: runtime semantics are correct
(no value corruption). Classification as A4 is design-driven.

**Status: A4-07 fixture LANDED**.

---

### F-02e — A4-09 Lifetime ref erasure: v1 sentinel reproduces (linker fail)

`probe.vais`:
```vais
F take_lifetime_ref<'a>(r: &'a i64) -> i64 { R 42 }
F main() -> i64 {
    val: i64 = 100
    r: &i64 = &val
    R take_lifetime_ref(r)
}
```

Build: macOS arm64, clang linker, 2026-05-03.
Master-plan.toml v1 expected: linker undefined symbol _take_lifetime_ref.
Observed: exact match. Build exits non-zero with:

```
Undefined symbols for architecture arm64:
  "_take_lifetime_ref", referenced from:
      _main in ir_O0_*.o
ld: symbol(s) not found for architecture arm64
```

**Status: A4-09 fixture LANDED** with `assertion_kind = "build_fails"`
matching kind 3 of F-05 protocol revision below.

---

### F-05 — Environment-stability classification of A4 surfaces

After three iterations the empirical pattern crystallizes:

| Class | A4 surfaces | Why exit is environment-stable |
|---|---|---|
| **Source-constant return** | A4-01 (96 from void slot), A4-06 (100 from truthy branch), A4-07 (42 from widened literal) | Runtime returns a value the source code explicitly named (or LLVM-default for unwritten slot). Stable across architectures because LLVM's default-value semantics are deterministic per platform. |
| **Memory-load corruption** | A4-02, A4-03 (and likely A4-04, A4-05, A4-08) | Runtime returns a value derived from `load i64` against a stack/heap address that wasn't supposed to be loaded as i64. Result is the address itself, the lower bits of an adjacent value, or whatever LLVM's allocator placed there. NOT stable across optimization levels, OS, arch. |
| **Linker reject** | A4-09 | Compiles but linker fails — exit code is the linker's exit code (typically 1) on every environment that has a linker. Stable. |
| **Late codegen reject** | A4-08 | Same — clang IR mismatch error. Stable. |

This classification suggests Step 7 protocol revision should accept
THREE assertion shapes:
  1. `assertion_kind = "exact_exit"` for source-constant cases.
  2. `assertion_kind = "exit_not"` (negation list) for memory-load cases.
     Probe asserts exit ≠ {value the source intended}.
  3. `assertion_kind = "build_fails"` for late-codegen / linker cases.
     Probe asserts compile or link command exits non-zero with a specific
     error pattern in stderr.

The 4 environment-stable A4 entries (01, 06, 07, 09) can land fixtures
under the current `exact_exit` form. The 5 environment-volatile entries
(02, 03, 04, 05, 08) need protocol kind 2 or 3 before fixtures land.

---

### F-02c — A4-06 Integer truthy: v1 sentinel reproduces (exit 100)

`probe.vais`:
```vais
F main() -> i64 {
    x: i64 = 5
    I x { R 100 } EL { R 200 }
}
```

Build: macOS arm64, vaisc release build, 2026-05-03.
Master-plan.toml v1 expected: 100 (truthy branch — runtime correct,
design violated).
Observed: 100 — exact match.

Unlike A4-02/A4-03, this surface's runtime observable is **the expected
branch value** (a constant in the source), not a memory-layout artifact.
So the v1 single-sentinel exit code IS environment-stable here. The
defect is purely in the type checker, not in runtime memory behavior.

**Status: A4-06 fixture LANDED** at
`compiler/tests/empirical/A4/A4-06_integer_truthy/`.

---

### F-02b — A4-03 Auto-deref &T ↔ T: same environment drift as A4-02

`probe.vais`:
```vais
F take_i64(x: i64) -> i64 { R x }
F main() -> i64 {
    val: i64 = 42
    r: &i64 = &val
    R take_i64(r)
}
```

Build: macOS arm64, vaisc release build, 2026-05-03.
Master-plan.toml v1 expected: 200.
Observed: 56.

Same drift mode as F-02 (A4-02). The probe still demonstrates the
surface (auto-deref & ↔ T type-check passes silently), but the specific
exit code is environment-dependent. Reinforces F-02's protocol revision
recommendation.

**Status: A4-03 fixture deferred** pending Step 7 protocol v2 revision.

---

### F-03 — `compiler/tests/empirical/` directory did not exist before this iteration

Master Plan v16 declared 9 A4 entries with `compiler/tests/empirical/`
referenced as the permanent fixture location, but the directory was not
yet created. First iteration of Step 7 created:
- `compiler/tests/empirical/README.md`
- `compiler/tests/empirical/A4/`
- `compiler/tests/empirical/A4/A4-01_unit_i64/` (4 files: probe.vais,
  expected.txt, run.sh, meta.toml)
- `compiler/tests/empirical/A4/STEP7_FINDINGS.md` (this file)

---

### F-04 — A4 site paths are unique by filename in the codebase

`unification.rs` and `control_flow.rs` each exist exactly once in
`compiler/crates/`. master-plan.toml's `unification.rs:N` form is
therefore unambiguous despite omitting the directory prefix. No drift to
correct in toml.

---

### F-06 — A4-08 Vec ↔ &T permissive: v1 sentinel symptom drifted, surface persists

Updated 2026-05-03 (second pass).

**v1 expected**: clang IR mismatch ({ptr,i64} vs ptr) — build-time
late codegen failure.

**Current observation** (macOS arm64, vaisc release, 2026-05-03):
- Type-check: PASSES (surface still firing — `unification.rs:384`
  `Ok(()) // Permissive: allow Vec ↔ &T` is unchanged).
- Build: SUCCEEDS (no clang IR mismatch — codegen has become more
  robust, or the IR layout is now compatible enough for clang).
- Runtime: When the `&str` parameter is actually CONSUMED (passed to
  `puts()` or any function that reads it as a C string), the program
  crashes with **SIGSEGV (exit 139)** because the Vec fat pointer is
  misinterpreted as a str.

**Reproducer (consuming probe)**:
```vais
N {
    F puts(s: str) -> i32
}

F take_str_ref(s: &str) -> i64 {
    puts(*s)
    R 0
}

F main() -> i64 {
    v: Vec<i64> = [42, 100, 999]
    take_str_ref(v)
    R 0
}
```
Result: build OK, runtime SIGSEGV (exit 139).

**Earlier non-consuming probe** (build-only, runtime returns body
constant):
```vais
F take_str_ref(s: &str) -> i64 { R 1 }   # body never reads s
```
Result: build OK, runtime exits 1. The defect is masked because the
function body never reads the misinterpreted parameter.

**Reclassification**:
- The surface itself (Vec ↔ &T permissive unification) is **still
  present** — `unification.rs:384` is unchanged.
- The v1 symptom (clang IR mismatch) no longer reproduces, but a worse
  symptom emerged: the program builds successfully and SIGSEGVs at
  runtime when the falsely-typed `&str` is actually consumed.
- A4-08 should remain in the A4 inventory but **migrate from the
  late-codegen-silent class to a runtime-crash class**, OR remain
  classed as build-fails on the conservative reading that the v1
  symptom was the deliberately documented one and runtime crashes are
  out of scope.

**Decision (this iteration)**: keep A4-08 classified as
`A4-late-codegen-silent` per master-plan.toml — the
`assertion_kind = "build_fails"` form would now fail (no build error).
Land the fixture under a new `assertion_kind` form: `runtime_crashes`,
which asserts `vaisc check` passes, build succeeds, and runtime exit
is 139 (SIGSEGV) when the parameter is actually consumed. The protocol
is amended to support this fourth assertion kind (see protocol revision
v8 below).

**Status: A4-08 fixture LANDS this iteration** with the consuming
probe and `assertion_kind = "runtime_crashes"`.

---

## Step 7 protocol revision (LANDED)

The protocol revision has landed in
`compiler/docs/certification/EXCLUDED_FEATURES.md §Empirical verification
protocol § Assertion-kind tri-form (NEW v7 — Step 7 first iteration F-05)`.

The tri-form (`exact_exit` / `exit_not` / `build_fails`) covers the four
environment-stability classes identified in F-05. The four already-landed
fixtures use:

- A4-01: `exact_exit` (source-constant, void slot LLVM-default 96)
- A4-06: `exact_exit` (source-constant, truthy branch literal 100)
- A4-07: `exact_exit` (source-constant, widened literal 42)
- A4-09: `build_fails` (linker undefined symbol _take_lifetime_ref)

The 5 deferred fixtures (A4-02, A4-03, A4-04, A4-05, A4-08) will use
`exit_not` with `forbidden_set` enumerating the value the well-typed
program would have returned. Step 7 second iteration lands them.

---

## Next iterations

- F-02 → Step 7 protocol revision → re-attempt A4-02.
- A4-03 through A4-09: each iteration re-runs the v1 probe, captures
  observed result in this findings file, and lands the fixture if the
  evidence is stable across local environments. Discrepancies feed back
  into the protocol.

This iterative structure is exactly what Master Plan v16 §Order Step 7
"fixed-point iteration" calls for — discoveries go here, the protocol
adapts, fixtures land as evidence stabilizes.

---

## Controlled v2 retro-validation — third iteration findings

### F-07 — Controlled-06 (Vec ↔ Slice .len() path) NOT actually controlled

Probe:
```vais
F len_of(s: &[i64]) -> i64 { R s.len() as i64 }
F main() -> i64 {
    v: Vec<i64> = [1, 2, 3]
    R len_of(&v)
}
```
Expected (per Controlled classification): exit 3 (Vec length).
Observed (macOS arm64, vaisc release, 2026-05-03): exit 184 — same
memory-load corruption pattern as A4-02/A4-03/A4-05.

Master-plan.toml lists this as Controlled, but empirical evidence
shows runtime corruption — should likely be reclassified A4
(`A4-runtime-silent` with `exit_not = [3]` form).

**Status: Controlled-06 fixture deferred** pending classification
review. Either (a) the Controlled marking is wrong and this is A4-10,
or (b) the probe construction is wrong (need a probe that actually
exercises the documented Controlled behavior).

### F-08 — Several Controlled probes fail to construct in current parser

- **C-04 (Fn ↔ FnPtr)**: `f: F(i64) -> i64` syntax for function-pointer
  parameter is not currently parsed; lexer emits LParen before comma.
  Fixture deferred until Vais surface for fn-pointer parameters is
  documented.
- **C-08 (DynTrait dispatch)**: dyn trait surface complex; deferred.
- **C-09 (Linear/Affine wrapper erasure)**: internal compiler concept;
  no clear user-level surface; deferred.

### F-09 — Controlled fixtures LANDED this iteration

- **C-01** Str/str/String alias — exit 7 (function-return constant)
- **C-02** Unknown unify-any — exit 11 (id<T>(11))
- **C-05** Numeric widening — exit 17 (take_i64(16) + 1; overlaps A4-07)

**Three of nine** Controlled entries have v2-reverified fixtures.
Six deferred (C-03 Never, C-04 Fn↔FnPtr, C-06 Vec↔Slice .len(), C-07
&Vec↔&[T], C-08 DynTrait, C-09 Linear/Affine). Step 7 next iterations
investigate parser/syntax constraints and reclassify C-06 as A4-10
candidate.

---

## Rejected/Untested v2 retro-validation — fourth iteration findings

### F-10 — Rejected-01 LANDED (Box raw generic)

Probe: `F take_box(b: Box) -> i64 { R b.value }` → vaisc check exits
non-zero with **E030 'no field value on type Box'**. Type-check is the
documented stable defense, exactly as master-plan v16 records.

Status: Rejected-01 fixture LANDED with `assertion_kind = "check_fails"`.

### F-11 — Rejected-02 (Box ↔ T) v1 sentinel does NOT reproduce

Probe: `F take_box(b: Box<i64>) -> i64 { R take_i64(b) }` → vaisc
check now PASSES, no E001. Master-plan documented E001 reject; current
behavior accepts. Either the surface was tightened (likely fix), or
the v1 probe wording is under-specified.

Status: Rejected-02 fixture deferred. Investigation needed: is
unification.rs:130 still the same code? What probe does fire E001?

### F-12 — Rejected-03 LANDED (Optional ↔ T, bare i64)

Probe: `F take_opt(x: Option<i64>); main passes 42` → E001 'expected
Option<i64>, found i64'. Master-plan v1 sentinel reproduces exactly.

Status: Rejected-03 fixture LANDED.

### F-13 — Untested-01 (Result ↔ Unit auto Ok wrap) → RECLASSIFY to Rejected

Probe: `F do_nothing() { R }; F take_result() -> Result<i64,str> { R do_nothing() }`
→ E001 'expected Result<i64,str>, found ()'. The type checker DOES
reject. Master-plan v16 listed this as Untested with "treat as A4
candidate by default" — but empirical evidence shows it is already
safe (Rejected). Recommendation: master-plan.toml should reclassify
from Untested to Rejected.

Status: Untested-01 fixture LANDED with reclassification recommendation
recorded in meta.toml. After master-plan amendment, fixture moves to
compiler/tests/empirical/Rejected/.

### F-14 — `check_fails` assertion kind added (5th form)

The Rejected/Untested fixtures introduced a new failure mode: the
type checker (not the linker, not the runtime) is the stable defense.
Existing `build_fails` form requires the type-check to PASS and the
build to fail; that does not fit.

Added `check_fails` to the Empirical verification protocol (§Assertion-
kind tri-form, now five-form): `vaisc check` exits non-zero AND stderr
matches every regex in `required_stderr_patterns`. Used by Rejected-01,
Rejected-03, Untested-01 (reclassification candidate).

Protocol now five-form: `exact_exit | exit_not | build_fails |
runtime_crashes | check_fails`.

---

### F-15 — NEW A4 candidate: struct partial-init silent acceptance

Discovered while building Stage 5 cross_package_schema validation gate.

Probe:
```vais
P S User { id: i64, email: str, name: str, age: i64 }
F main() -> i64 {
    u: User = User { id: 1, email: "a", name: "n" }   # age omitted
    R 0
}
```

Expected (per L-002): type-check rejects the partial init with a stable
diagnostic — every required field must be present at the constructor.

Observed (macOS arm64, vaisc release, 2026-05-03): vaisc check exits 0.
The runtime presumably zeroes the missing field.

This is a NEW A4-candidate surface, not in master-plan v16's A4-01..A4-09
inventory. Step 7 next iteration should add it as A4-10 and write the
empirical fixture (`assertion_kind = "exit_not"` form, forbidden_set
contains the value the user expects when they think the field was set
explicitly).

Workaround applied to Stage 5: the negative gate uses field TYPE-CHANGE
(email str → i64) instead of field ADDITION, since type-change does
propagate via E001 'expected str, found i64' at the consumer's
`R u.email` site.

---

### F-16 — A4-05 Array→Pointer is structural, not user-level (2026-05-04)

Stage 1 attempt for A4-05 (Array → Pointer decay) found that the
surface fires on EVERY fixed-size-array indexing expression in the
language, not just the master-plan v1 probe. Minimal repro:

```vais
S Holder { arr: [i64; 3], }
F main() -> i64 {
    h: Holder = Holder { arr: [1, 2, 3] }
    R h.arr[0]    # fires A4-05 under strict mode
}
```

Trace:
- `h.arr` resolves to `[i64; 3]` (ConstArray<i64, 3>).
- Indexing lowers internally as `ptr_arith(arr_base, 0)` → produces
  `Pointer<i64>`.
- The unifier compares the resulting `Pointer<i64>` to the expected
  element type `i64` → routes through Array↔Pointer arm → A4-05.

Implications:
- A4-05 is NOT a user-facing implicit coercion in the same sense as
  A4-01 (Unit↔i64) or A4-02 (Pointer↔i64). It is the lowering glue
  between source-level Array indexing and codegen-level pointer
  arithmetic.
- Removing A4-05 at the unifier level would require codegen to
  expose Array indexing as a typed operation (so the result is
  `i64`, not `Pointer<i64>`). That is a Step 16 (memory protocol)
  scope question more than a Step 13 (A4 removal) one.

Recommendation:
- Reclassify A4-05 from A4-runtime-silent to **Controlled
  (compiler-internal IR lowering)** per L-002 scope clause —
  "compiler-internal IR lowering coercions are out of scope". The
  vaisdb hnsw/cow.vais "single dependency" reported earlier in
  master-plan status is the same indexing pattern.
- Keep VAIS_REJECT_A4_05=1 as an opt-in for users who want their
  source-level `as *T` casts to be visible (e.g. when implementing
  raw-pointer manipulation at the user level), but do not flip
  default to strict. Master-plan §A4-05 entry should be amended to
  reflect this scope decision in the next plan revision.

Status: A4-05 fixture continues to use the override-via-A4-02 path
(probe trips A4-02 first); no user-level migration was needed and
none is recommended.

---

### F-17 — A4-03 Auto-deref &T↔T also IR-lowering glue (2026-05-04)

Stage 1 attempt for A4-03 (Auto-deref &T ↔ T) found that strict mode
fires not just on the user-level "ptr-as-value" case but also on
generic-method receivers where inference produces `Ref(Var)`
intermediate types. Repro:

```vais
F take_i64(x: i64) -> i64 { R x }
F main() -> i64 {
    val: i64 = 42
    r: &i64 = &val
    R take_i64(r)            # this IS the A4-03 surface — should reject
}

# but also:
S ByteBuffer { ... }
X ByteBuffer {
    F from_buf(other: &ByteBuffer) -> ByteBuffer { ... }
    F clone(self) -> ByteBuffer {
        ByteBuffer.from_buf(&self)   # this is NOT a user-level coercion —
                                      # both sides are Ref. But strict A4-03
                                      # still rejects because inference
                                      # produces Ref(Var) intermediate that
                                      # routes through the (Ref, *) arm before
                                      # the (Ref, Ref) arm at line 252 fires.
    }
}
```

Strict-mode footprint reflects this: 4 std + 149 vaisdb files. The
vast majority are Ref(X) ↔ Ref(Var) generic-inference unifications,
not the actual implicit-deref pattern.

Tightening attempt (only fire strict when other is NOT a Ref) was
implemented and reverted — it produced identical footprint (still
4/149) because the offending paths use `Var` inner types that do
match the legacy `Ref(_), other` arm before reaching the typed
(Ref, Ref) arm.

Recommendation:
- Reclassify A4-03 from A4-runtime-silent to **Controlled
  (compiler-internal IR lowering)**, joining A4-05 per F-16. Both
  are unifier glue rather than user-level implicit coercions.
- Keep VAIS_REJECT_A4_03=1 as an opt-in for users who want to
  surface the actual ptr-as-value cases (probe runs prove strict
  mode catches `take_i64(r)` where `r: &i64`).
- Decision deferred to next master-plan revision; no compiler change
  in this commit.

Status: A4-03 stays Stage 0 opt-in. master-plan §A4-03 candidate for
Controlled reclassification.

---

### F-18 — Escape closure silent capture loss (NEW A4 candidate, 2026-05-04)

Discovered during Step 9 A2-04 promotion empirical work. Escape
closure (closure returned from a function and called later) passes
type-check, builds, runs — but produces a wrong runtime result.
Captured environment is not preserved across the call boundary.

Probe (compiler/tests/empirical/A2/A2-04_inline_closure/probe_neg.vais):
```vais
F make_adder(x: i64) -> |i64| -> i64 {
    |n| n + x
}
F main() -> i64 {
    add5 := make_adder(5)
    R add5(37)
}
```

Build: macOS arm64, vaisc release, 2026-05-04.
Expected (well-typed): 42 (= 37 + 5).
Observed: 245 / 69 (varies by build cache).

Different runs produce different numbers — classic escape-capture
silent corruption (capture frame is freed, the closure reads stale
stack memory).

assertion_kind = "exit_not", forbidden_set = [42].

Status: NEW A4 candidate ("A4-12 Escape-closure capture loss"). Not
yet in master-plan.toml [[phase.A4.runtime_silent]]. Empirical
fixture LANDED at compiler/tests/empirical/A2/A2-04_inline_closure/
(double duty: A2 promotion fixture for inline subset + A4 evidence
for escape surface). Master-plan v17+ inventory expansion
candidate.

Inline closure pattern (A2-04 positive) works correctly: `apply(|n|
n + 1, 41)` returns 42 deterministically. The split between safe
inline use and unsafe escape mirrors the predicate proposed in
A2_SUBSETS.md §A2-04.

---

### F-19 — A4-06 strict mode emits "expected i64, found bool" in std/args.vais (2026-05-04)

Discovered while reconning strict-default flip cost for A4-06
(integer-as-truthy in if/else-if/ternary cond positions).

Setup: env `VAIS_REJECT_A4_06=1` switches the four sites in
crates/vais-types/src/checker_expr/control_flow.rs from lenient to
strict — cond expressions of integer type are unified against
`Bool` instead of being accepted as truthy.

Baseline cost (`bash scripts/check-integrity.sh` with env on):
- std_files: 82 → 73 (delta=-9)
- vaisdb_files: 261 → 236 (delta=-25)
- vaisdb runtime smoke: 28 → 23 (5 new failures)

Per-file probe of the 9 std failures shows two distinct error
shapes:

1. `expected bool, found i64` — the expected A4-06 surface. Cond
   site receives an i64 expression and is rejected. Migration:
   add explicit `!= 0`. Files: std/async.vais, std/fmt.vais,
   std/http.vais, std/http_server.vais, std/runtime.vais
   (5 of 9).

2. `expected i64, found bool` — REVERSE direction. Files:
   std/args.vais, std/path.vais, std/proptest.vais, std/url.vais
   (4 of 9). All cond sites in std/args.vais are pure
   comparisons (`>=`, `==`, `<`) which already produce bool, so
   this error cannot be a cond-site mismatch. The strict A4-06
   path appears to perturb downstream type inference such that
   some i64-consuming context now sees a bool.

Implication (revised 2026-05-04): The two error directions are a
unify-orientation artefact of `unify(cond_type, Bool)` —  when
cond_type is concrete `i64` (e.g. function call returning i64) the
TypeError::Mismatch fields land as `expected: i64, found: bool`.
Concrete example std/args.vais:141:
  `I @.str_eq_internal(spec_long, long_name) { R i }`
where `F str_eq_internal(...) -> i64`. Migration: `... != 0`.

Both error directions describe the SAME A4-06 surface. No separate
inference side-effect. Codemod plan: rewrite each call-cond /
identifier-cond of i64 type to explicit `!= 0`. Tractable.

Status: A4-06 stays Stage 0 (opt-in via VAIS_REJECT_A4_06=1) until
codemod lands across baseline (std 9 + vaisdb 25). Strict default
flip is a follow-up commit after codemod completes.

---

### F-19 std codemod LANDED — 2026-05-04

All 9 failing std files migrated and re-verified strict-clean.
Baseline (default mode) integrity preserved.

Migration patterns applied:
- `LW 1 { ... }` → `LW true { ... }` (12 sites across async,
  fmt, http, http_server, runtime — canonical infinite loop).
- `I <i64-call>(...)` → `I <i64-call>(...) != 0` (10 sites
  across args:141, url:127–198, proptest:379, path:214).
- `I <i64-var>` → `I <i64-var> != 0` (6 sites across path —
  `ends_with_slash` and `needs_sep` flag vars).

Verification (per-file):
  for f in args async fmt http http_server path proptest runtime url; do
    VAIS_REJECT_A4_06=1 vaisc check std/$f.vais
  done
  → 9/9 OK No errors found.

Default-mode (baseline) integrity preserved (re-run via
scripts/check-integrity.sh — INTEGRITY OK).

Strict default flip still NOT enabled — `VAIS_REJECT_A4_06=1` opt-in
remains the gate. Reason: vaisdb baseline still has 25 sites failing
strict (per F-19 pre-codemod recon). vaisdb codemod is a separate
follow-up iter; the std-only commit is a clean intermediate
checkpoint where the std slice is strict-ready and the strict default
flip can land after vaisdb migrates.


---

### F-19 — A4-06 strict default LANDED 2026-05-04 (Step 13 stage 1)

After std codemod (commit 965fdaae) and vaisdb codemod (lang commit
4c9400f, 22 files / 43 sites), all 4 strict-mode sites in
crates/vais-types/src/checker_expr/control_flow.rs were flipped from
opt-in `VAIS_REJECT_A4_06=1` to strict default with legacy escape
hatch `VAIS_REJECT_A4_06=0`.

Verification

  bash scripts/check-integrity.sh:
    INTEGRITY OK: core=ok mir=ok codegen=ok unsafe_audit=ok
    ecosystem=ok (std=82/82 vaisdb=261/261) backend=ok
    http_client_runtime=ok (smoke=1/1) vaisdb_runtime=ok (smoke=28/28)
    server_runtime=ok (smoke=13/13) web_runtime=ok (smoke=23/23)
    cross_package_schema=ok (gate=2/2)

  bash scripts/check-empirical.sh:
    EMPIRICAL FIXTURES: 28 pass / 0 drift / 0 broken / 0 skipped.

Fixture migration

  compiler/tests/empirical/A4/A4-06_integer_truthy/ migrated from
  v1 retro-validation form (probe.vais + expected.txt) to two-probe
  LANDED form:
    probe_pos.vais — `I x != 0 { 100 } EL { 200 }` exits 100.
    probe_neg.vais — `I x { 100 } EL { 200 }` rejected at vaisc check
                      (E001 type mismatch).
    run.sh         — two-probe runner.
    meta.toml      — assertion_kind kind_negative=check_fails.

Impact

  A4-06 surface formally CLOSED. 8/11 A4 strict-default + 2/11
  reclassified (A4-03/05) + 1/11 remaining (A4-07 numeric widening,
  scope ~221 sites — significantly larger than A4-06's 75 total).


---

### F-20 — A4-07 std codemod LANDED 2026-05-04 (Step 13 stage 0 std slice)

Recon for A4-07 strict-default flip cost. Probing std and vaisdb with
`VAIS_REJECT_A4_07=1` shows std 82→76 (-6) and vaisdb 261→45 (-216).

The vaisdb scope (216 files) is significantly larger than A4-06's
25 files — the master-plan v16 estimate of "std 6 + vaisdb 215"
is confirmed. This is the bulk of the remaining §A4 timeline.

Migration patterns (from std)

  exit(1)                      → exit(1 as i32)
  setenv(name, value, 1)       → setenv(name, value, 1 as i32)
  srand(seed)                  → srand(seed as i32)
  puts_ptr(self.data)          → puts_ptr(self.data) as i64
  rand()                       → rand() as i64
  usleep(micros)               → usleep(micros) as i64

Common cause: builtin extern functions (`exit`, `setenv`, `srand`,
`usleep`, `rand`, `puts_ptr`) take or return i32 per C ABI, but Vais
literal integers default to i64 and call sites unify silently. Fix
is explicit `as iN` cast at call site OR (for return-position) cast
the call expression result.

Std side LANDED (6 files / 8 sites): std/contract.vais, std/env.vais,
std/random.vais, std/string.vais, std/owned_string.vais, std/time.vais.

Vaisdb side DEFERRED (216 files / ~213 sites estimated). Heterogeneous
patterns include: u32 vs i64 (atomic store sites: `active_writer.store(txn_id)`),
function return widening, struct field assignment, comparison RHS
type mismatch. Multi-iter codemod required.

Status: A4-07 stays Stage 0 (opt-in via VAIS_REJECT_A4_07=1) until
vaisdb codemod completes. Strict default flip is multi-iter follow-up.


---

### F-21 — A4-07 strict scope is broader than master-plan v16 estimated (2026-05-04)

Continuing from F-20: vaisdb codemod attempt for A4-07 surfaced a
deeper problem than the v1 master-plan entry described.

The unification.rs site (line 351) is `(a, b) if is_integer_type(a)
&& is_integer_type(b)`. Strict mode rejects ANY integer-to-integer
unification regardless of whether one side is a literal.

Vais integer literals default to i64. So under strict A4-07, EVERY
literal in a non-i64 context (struct fields, function calls, atomic
stores, bit operations, etc.) becomes a hard error:

  x: u32 = 100        → expected u32, found i64
  y := x & 255        → expected u32, found i64
  buf.push(0)         → expected u8, found i64 (Vec<u8>)
  txn_id_field: u64   → expected u64, found i64 (literal init)

This is far more pervasive than "implicit widening at function call
sites" (master-plan v1 description). The vaisdb count (216 files /
~213 sites) reflects literal sites, not just call-site widenings.

Two viable paths

(a) **Refined strict mode**: distinguish literal types from concrete
    types in unification. Allow `Literal(i64) ⇄ uN/iN` (the literal
    adopts the context type) while rejecting `Concrete(i64) ⇄ Concrete(uN)`.
    Requires plumbing literal-ness through ResolvedType or a parallel
    inference channel. Substantial work (estimated 500-1000 LOC across
    ast / types / inference), comparable to a Stage B refactor.

(b) **Reclassify A4-07 to Controlled**: admit default-i64-literal
    promotion as a documented well-defined behavior (matches Rust's
    `0` literal in `let x: u32 = 0` — Rust integer literals are
    polymorphic, but Vais literals are concrete i64 so the analogue
    is leaky). EXCLUDED_FEATURES.md §Controlled coercions gets a new
    entry; A4-07 inventory shrinks to "non-literal call-site widening
    only", which is the std-side scope (already LANDED).

Path (a) is the L-002-pure choice but expensive. Path (b) is the
pragmatic ship-now choice and aligns with how A4-03 / A4-05 already
got reclassified (F-16 / F-17). 

Recommendation: Path (b) reclass — but only after path (a) is
estimated against current AI-multi-session capacity. Master-plan v17
should make this decision explicitly.

Status: A4-07 vaisdb codemod paused. std side (6 files / 9 sites)
LANDED. Strict default flip pending the (a)-vs-(b) decision.

---

### F-22 — A4-03 / A4-05 / A4-07 reclass to Controlled LANDED in master-plan v17 (2026-05-04)

Decision: **Path (b)** — reclassify all three to Controlled per the
LESSONS L-002 §Scope clause ("compiler-internal IR lowering coercions
are out of scope of 'implicit anything is a defect'"). Empirical
baseline footprints re-measured 2026-05-04:

| Site | std impact | vaisdb impact | Root cause |
|---|---|---|---|
| A4-03 (`VAIS_REJECT_A4_03=1`) | -4 / 82 | -150 / 261 | Generic-inference produces `Ref(Var)` intermediate; legacy `(Ref, _)` arm fires before typed `(Ref, Ref)` arm (F-17 confirmed) |
| A4-05 (`VAIS_REJECT_A4_05=1`) | 0 / 82 | -1 / 261 | ConstArray field initialization via function-return: array literal lowers to `Pointer<T>` at IR boundary, struct-init unify routes through Array↔Pointer arm (F-16 + this iter's standalone repro `S Foo { arr: [i64;3] } F mk() -> [i64;3] { [1,2,3] } F main() { Foo { arr: mk() } }`) |
| A4-07 (`VAIS_REJECT_A4_07=1`) | 0 / 82 | -216 / 261 | Default-i64 literal in any non-i64 context (struct fields, `Vec<u8>.push(0)`, bit-ops, atomic stores) — pervasive (F-21 confirmed) |

All three sites remain in `unification.rs` with strict-mode escape
hatch (`VAIS_REJECT_A4_{03,05,07}=1` → fail; default unset → preserve
legacy unify). Users who want to surface the actual user-level
patterns can opt in.

Path (a) (refined strict mode with literal-ness propagation, ~500-1000
LOC across ast/types/inference) is documented but deferred — Path (b)
ships value now, and a future iteration may revisit (a) when MIR
borrow / Step 16 work creates broader leverage on the inference layer.

Master-plan deltas:
- `meta.version`: 16 → 17
- `[phase.A4]`: total 9 → 6 (runtime_silent 7 → 4)
- `[controlled]`: total 9 → 12 (added 3 entries)
- ROADMAP / EXCLUDED_FEATURES re-rendered; check-plan-consistency.sh PASS
- INTEGRITY OK preserved (toml/doc/comment-only changes; no compiler logic delta)

Status: A4-03 / A4-05 / A4-07 reclass complete. A4 inventory now
6 entries — all already strict-default (verified via grep
`VAIS_REJECT_A4_*.as_deref\(\) (==|!=) Ok\("0"\)`):
- runtime-silent 4: A4-01, A4-02 (`unification.rs:381,461`),
  A4-04 (`unification.rs:481`), A4-06 (`control_flow.rs:195,250,282,407`)
- late-codegen-silent 2: A4-08 (`unification.rs:416`),
  A4-09 (`unification.rs:538,550`)
- additional already-LANDED strict defaults outside the 6-entry inventory:
  A4-10 (`collections.rs:885`), A4-11 (`special.rs:30`)

This means **Step 13 (A4 removals) is now substantively complete**.
Remaining work: ROADMAP Step 13 status string update + permanent
fixture audit per master-plan §Empirical protocol step (positive
post-migration + negative pre-migration rejected per entry).

---

### F-23 — A2-03 dyn dispatch silently calls first impl (NEW A4 candidate, 2026-05-04)

Discovered during A2-03 (dyn / trait object dispatch) promotion
attempt for Step 9. Probe:

```vais
W Greeter { F greet(self) -> i64 }
S Hello {}
S World {}
X Hello: Greeter { F greet(self) -> i64 { R 42 } }
X World: Greeter { F greet(self) -> i64 { R  7 } }

F call_dyn(g: &dyn Greeter) -> i64 { g.greet() }

F main() -> i64 {
    w := World {}
    call_dyn(&w)         # expected 7, observed 42
}
```

Direct dispatch (`World{}.greet()`) returns 7 — correct. Cross-impl
dyn dispatch routes to the FIRST registered impl method (Hello.greet),
ignoring the runtime type. Silent corruption.

vaisdb impact (potential): `lang/packages/vaisdb/src/sql/executor/
sort_agg.vais` uses `Box<dyn Executor>` chains for SortExecutor /
DistinctExecutor / etc. Each `.next()` call on the dyn-boxed inner
executor may dispatch to the wrong impl. **vaisdb runtime smoke 28/28
masks this** because the smoke fixtures may not exercise the multi-
impl-per-trait paths that surface the bug. Step 18 product-broad
fuzzing or a dedicated runtime probe would surface real downstream
defects.

vector/hnsw also uses `&mut dyn NodeStore` (delete.vais:106,248,460;
insert.vais:258). Probability of cross-impl in same binary unknown
without further recon.

Implications:
- A2-03 promotion is BLOCKED. No predicate over the current dispatch
  surface is honest until the impl-selection bug is fixed.
- This is a TRUE silent surface (type-checks, runs, returns wrong
  value). Per L-002 it qualifies for A4 inventory expansion.

Recommendation:
- Defer A2-03 promotion until the dyn-dispatch bug is fixed in
  codegen / vtable lowering.
- Add candidate **A4-12: dyn dispatch impl-selection bug** to the
  next master plan revision after v17 (reclass round), with this
  finding as the v1 evidence + the Hello/World probe as the
  permanent fixture.

Root cause line, identified during A4-12 reconnaissance (2026-05-04):

  `compiler/crates/vais-codegen/src/inkwell/gen_aggregate.rs:885-899`

  When `infer_struct_name(receiver)` returns Err (the case for
  `g: &dyn Greeter` parameters because `extract_struct_type_name`
  in gen_special.rs:145 has no `Type::DynTrait` arm and falls
  through to `_ => None`), method dispatch enters a fallback that
  iterates `generated_structs.keys()` sorted alphabetically and
  picks the first `<StructName>_<method_name>` that exists. For
  the Hello/World probe:

    candidates (sorted) = ["Hello", "World"]
    first match for "_greet" = "Hello_greet"  ← bound regardless of receiver

  IR confirmation (vaisc --emit-ir on the probe):

    define i64 @call_dyn(ptr %g) {
      ...
      %method_call = call i64 @Hello_greet(ptr %g1)   ← static, not vtable-indirect
      ret i64 %method_call
    }

  The vtable infrastructure already exists
  (`vais-codegen/src/vtable.rs::generate_dynamic_call` +
  `trait_dispatch.rs::generate_dyn_method_call`) but is not invoked
  from the method-call dispatcher. Wiring that path is the A4-12
  fix scope.

  A non-corrupting guard was prototyped (refuse with
  CodegenError::Unsupported when `var_resolved_types[name]` is
  DynTrait/Ref(DynTrait)) but did not trigger because the
  type-checker does not populate `var_resolved_types` for dyn
  parameters reliably. The first deliverable of the A4-12 fix
  must therefore extend the type checker to surface dyn receiver
  info to codegen before the dispatcher can branch on it.

### F-23 step 1 reconnaissance result (2026-05-04, A4-12 attempt)

`vaisc --inkwell build` with debug instrumentation in
`gen_aggregate.rs::generate_method_call` revealed the actual content
of `var_resolved_types["g"]` for the Hello/World probe:

```
[F23-GUARD-INKWELL] method='greet' receiver=Ident("g")
                   var_resolved_types[receiver]=Some(Ref(I64))
```

i.e. **the type-checker is reducing `&dyn Greeter` to `&i64` before
the parameter is registered into codegen's `var_resolved_types`**.
Codegen's guard cannot trigger because it sees `Ref(I64)`, not
`Ref(DynTrait { ... })`. This identifies the precise step-1 fix
location (in vais-types substitution / parameter-binding paths) and
confirms why the prototype guard at the codegen layer was
necessarily dead.

Risk evaluation for landing step 1 in this session:
- vais-types changes that preserve `DynTrait` through parameter
  resolution have potentially broad reach (vaisdb sql/executor
  `Box<dyn Executor>`, vector/hnsw `&mut dyn NodeStore`, vais-server
  middleware traits). INTEGRITY breakage probability is non-trivial.
- step 1 must be carefully scoped: only stop reducing `&dyn T` to
  `&i64`; downstream codegen sites that *intentionally* consume the
  i64-shape (Box<dyn> as a fat pointer i64) must keep working.
- Recommendation: defer step 1 to a plan-driven session with explicit
  vais-types reading + INTEGRITY-after-each-edit measurement, rather
  than a single-session fix attempt.

Status: F-23 step 1 root cause CONFIRMED in vais-types parameter
resolution. Codegen guard remains in place (currently dead because
preceding type-checker output never satisfies the predicate; once
step 1 lands, the guard activates and converts silent corruption to
loud CodegenError::Unsupported). Step 1 implementation deferred —
DEFERRED_TASKS.md #15 next_check 2026-05-05.

### F-23 step 1 LANDED — A4-12 step 1 fix (2026-05-04, same-day continuation)

The root cause was NOT in vais-types itself but in the **codegen-side
ast→resolved conversion** that codegen runs *after* receiving the AST:

  vais-codegen/src/inkwell/gen_types.rs::ast_type_to_resolved
  (the function gen_special.rs:466-472 invokes when binding a parameter
  to var_resolved_types)

This function is a `match` over `vais_ast::Type`. It had arms for
Type::Ref, Type::RefMut, Type::Slice, etc. but NO arm for
Type::DynTrait. Consequently `Type::Ref(Type::DynTrait{...})` recursed
into the inner DynTrait, which fell into the catch-all
`_ => ResolvedType::I64` arm — that's the I64 we observed.

Fix (compiler 283029e0):

```rust
Type::DynTrait { trait_name, generics } => {
    let resolved_generics: Vec<ResolvedType> = generics
        .iter()
        .map(|g| self.ast_type_to_resolved(&g.node))
        .collect();
    ResolvedType::DynTrait {
        trait_name: trait_name.clone(),
        generics: resolved_generics,
    }
}
```

Verification:

A2-03 Hello/World probe:
- Before: `vaisc build probe.vais` → success, `./probe_bin` → exit 42
  (silent: Hello.greet bound regardless of W{} receiver)
- After: `vaisc build probe.vais` → exit 1, stderr:
  `error[C005] Unsupported feature: dyn trait method call .greet on
   &dyn/&mut dyn receiver g: vtable-indirected dispatch is not yet
   wired (F-23, A4-12 candidate, STEP7_FINDINGS)`. probe_bin not
  produced. Silent corruption → loud rejection. L-002 north star
  recovered.

INTEGRITY OK preserved: vaisdb 261/261, std 82/82, all runtime
smokes green. Important empirical observation: vaisdb has 39 `&dyn` /
`Box<dyn>` sites but NONE of them triggered the new guard. This
suggests:
- Either each vaisdb dyn site is a single-impl-per-trait scenario
  (no cross-impl dispatch ambiguity to exploit);
- Or the method dispatch goes through a different expression form
  (e.g. `Expr::Field` for `self.input.open()?` — the guard matches
  only `Expr::Ident(name)` in the receiver position).

Either way, silent corruption surface is **narrower than feared** —
it requires the specific multi-impl + bare-Ident-receiver pattern
that the Hello/World probe uses. vaisdb baseline does not exhibit
this pattern, so step 1 LANDED with zero migration cost.

Step 2 (vtable.rs::generate_dynamic_call wiring for the loud case)
is a separate follow-up. After step 1, multi-impl dyn dispatch users
can either refactor to fn-pointer params (A2-05, certified subset)
or wait for step 2's vtable indirection.

### F-23 step 2 reconnaissance (2026-05-05, A4-12 step 2 attempt)

Goal: replace the codegen guard's `CodegenError::Unsupported` with
real vtable-indirected dispatch via vtable.rs::generate_dynamic_call.

Empirical finding from `grep -nE "generate_dyn_method_call|generate_dynamic_call"
compiler/crates/vais-codegen/src/`:

| caller of TraitDispatcher::generate_dyn_method_call | hit count |
|---|---|
| inkwell backend (default for `vaisc build`) | **0** |
| text-IR backend | **0** |

`vtable_generator` field is held by the text-IR `CodeGenerator` struct
(lib.rs:227) but no method-dispatch site invokes
`TraitDispatcher::generate_dyn_method_call`. The vtable infrastructure
is fully written (vtable.rs ~415 LOC, trait_dispatch.rs ~250 LOC) but
**dead** — no caller in the production dispatch path.

Inkwell backend has no vtable infrastructure at all. The text-IR
infrastructure cannot be reused as-is because it emits string IR
incompatible with inkwell's builder API.

Step 2 proper scope is therefore TWO independent wirings:

a) **text-IR side**: invoke `TraitDispatcher::generate_dyn_method_call`
   from `expr_helpers_call/method_call.rs::generate_method_call_expr`
   when receiver type is `ResolvedType::DynTrait` / `Ref(DynTrait)`.
   Requires plumbing TraitDispatcher into the generate_method_call_expr
   call path (TraitDispatcher is in CodeGenerator but accessing it from
   the inner method-call dispatcher needs ergonomic access). Estimated
   ~50-100 LOC + tests.

b) **inkwell side (default backend)**: NEW vtable infrastructure using
   inkwell's BuilderValue / StructType / GlobalValue APIs, mirroring
   text-IR vtable.rs's behavior. ~400-600 LOC + integration tests +
   careful trait/method discovery from CodeGenerator state.

(b) is the larger work and is the user-facing impact (default backend).
(a) is the smaller work but vais users by default won't see it.

### Decision (2026-05-05)

Step 2 splits into two sub-tasks:

- **2a (text-IR side, smaller)**: ~50-100 LOC, single-session
  candidate but value to users is opt-in only (`vaisc check` /
  emit-ir; inkwell-default `vaisc build` unaffected).
- **2b (inkwell side, larger)**: 400-600 LOC, multi-iter, real user
  impact.

Both deferred. The current step 1 LANDED state is sufficient for L-002
north star (silent → loud); production users get a clear error
instead of wrong values. step 2 lifts the restriction to allow
multi-impl dyn dispatch to actually work.

Status: step 2 reconnaissance LANDED. step 2 implementation deferred
into 2 sub-tasks (DEFERRED_TASKS.md to be updated).

Status: A2-03 promotion BLOCKED. F-23 logged as NEW A4 candidate.
Master plan v17 Step 9 status retained at "A2-03 DEFERRED" (now with
explicit silent-surface evidence rather than just parser/resolver
hand-wave).


### F-24 — Rejected-02 (Box ↔ T) re-probe REPRODUCES silent accept (2026-05-05)

F-11 (2026-05-03) reported "v1 sentinel does NOT reproduce" for
Rejected-02 (Box<T> ↔ T auto-unwrap). Re-probed 2026-05-05 with the
master-plan v1 wording adapted to current Vais syntax:

```vais
F take_i64(x: i64) -> i64 { R x }
F take_box(b: Box<i64>) -> i64 { R take_i64(b) }
F main() -> i64 { R 0 }
```

Result: `vaisc check` → `OK No errors found`. The call `take_i64(b)`
silently passes a `Box<i64>` where `i64` is required. Master-plan v23
classifies this surface as **Rejected** (E001 expected), but empirical
evidence shows it is in fact a **silent A4 candidate**.

The earlier F-11 "does not reproduce" was caused by a probe formulation
issue (the previous probe declared `b: Box<i64>` in let-binding, which
trips a parser error before the type-check check runs). Once the probe
uses `b := Box::new(42)` style let, the parser succeeds and the type
checker silently accepts.

Status: NEW A4 candidate proposed (call it A4-13). Fixture + master-plan
reclassification deferred to next Step 7 iteration. Master-plan v23
Rejected entry for Box ↔ T is therefore **stale by empirical evidence**;
update will accompany A4-13 fixture LANDED.

Runtime probe was attempted (`Box::new(42)` in main) but failed with
C002 Undefined function: `Box::new` is not in current std — that is
a separate surface gap (Box constructor). The silent-accept bug lives
in the function-call type-check path, independent of std availability.

Recommendation: open a deferred multi-iter task for A4-13 — fixture
draft + master-plan reclassify (Rejected → A4) + check-empirical wiring
+ runtime probe (synthesize a Box value via std primitives or extern).
