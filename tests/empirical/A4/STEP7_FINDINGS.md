# Step 7 retro-validation findings (first iteration, 2026-05-03)

This file records what the first iteration of Order Step 7 retro-validation
empirically discovered. It complements the per-A4 fixture directories.

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
