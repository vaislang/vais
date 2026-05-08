# Vais Deferred and Experimental Features

## Purpose

This document removes unstable surfaces from the first correctness target.
Exclusion does not mean deletion. It means the feature cannot be used to
claim Core language correctness until it passes the promotion rule in
`VAIS_CORE_V0.md`.

## Important: "Deferred" does not mean "not implemented"

`Deferred` here means **not part of the certified Core proof**. Several of
the features below are implemented in the compiler at meaningful depth and
will accept user code without an error today. The risk is that they have
**no certified gate**, so subtle behavior or edge-case failures are not
guaranteed to be caught by the existing CI/integrity surface.

If you are evaluating Vais for production use, treat the depth columns
below as the actual constraint, not the `deferred` label alone:

- **deep** — the feature is implemented broadly, has e2e fixtures, and
  works for the common case. The reason it is `deferred` is administrative
  (no dedicated certification gate yet), not technical.
- **partial** — implemented enough to compile and sometimes run, but with
  known gaps that the test suite does not police. Edge cases can fail
  silently or panic.
- **stub** — parser/AST recognizes the syntax but later stages do not
  fully handle it. Using this in user code is unsafe.
- **none** — explicitly disabled or not implemented; the parser may still
  accept the syntax shape, but lowering will fail or silently no-op.

The `Promotion gate` column states what would be required to move a row
out of this document and into the Core proof.

## Deferred Language Features

| Feature | Depth (2026-05-03) | User-path risk if used today | Reason for exclusion | Promotion gate |
|---|---|---|---|---|
| broad implicit coercion (the user-facing surfaces verified by runtime fixtures) | **REMOVAL TARGET (A4-runtime-silent / A4-late-codegen-silent)** — see "Removal queue (A4)" below for the 9-entry verified inventory. | **high — silent acceptance** (proven by `/tmp/vais-fix-tests/sites/` fixtures producing wrong runtime exit codes or late codegen failures). | AI-native frame requires every conversion to be explicit. No promotion path. | A4 removal queue. Not promoted under any condition. |
| complex type inference (HKT/ImplTrait corner cases) | **partially obsolete claim** — codex v5 review verified that HKT/ImplTrait variants were already removed in ROADMAP #18 (see `unification.rs:569` comment). The `i64` fallback no longer exists in the current code. | n/a — entry kept for historical accuracy; current code does not exhibit the fallback. | n/a | n/a |
| integer truthy control-flow predicates | **REMOVAL TARGET (A4-design)** — verified at `control_flow.rs:188` (else-if), :243 (ternary), :273 (if), :396 (while). All four are the same Phase 254 "lenient cond" pattern. Each accepts `I x { ... }` (x: i64) without `!= 0`. | **high — silent (design)**: runtime behavior is correct (non-zero is truthy) but the explicit token `!= 0` is missing. AI may generate this pattern from other-language training. Violates north star "every check is an explicit token". | Migration to `I x != 0 { ... }` for every site. Not promoted. | A4 removal queue. Parser/type-check rejects with stable diagnostic after baseline migration. |
| `?` error propagation | stub (~80–120 LOC, parser-level only) | medium — partial Result handling, cross-module/wrong-receiver interactions are unstable | Cross-module and wrong receiver interactions are unstable in downstream code. | Result fixture suite plus stage-specific negative tests. |
| trait objects / vtables | partial (`vtable.rs`, ~1,186 LOC) | **high** — `dyn Trait` parses; method-dispatch resolution has uncovered paths and 0 e2e fixtures | Broad backend and ABI surface. | Dedicated design doc and call dispatch invariant. |
| advanced generics (monomorphizable subset) | deep (`vais-types`, ~5,459 LOC in `calls.rs`) | low for the monomorphizable subset (which already passes baseline). HKT/ImplTrait variants were removed in ROADMAP #18 (see `unification.rs:569`); the previous "i64 fallback" no longer exists in current code. | Promotion of the monomorphizable subset is an A2 candidate (formal predicate required). HKT/ImplTrait re-introduction would require a dedicated A2 with full fixture coverage, not a fallback. | A2 promotion with `A2_SUBSETS.md` formal predicate. |
| macros | partial (`vais-macro`, ~5,754 LOC, 12+ e2e fixtures) | medium — basic forms work, hygiene/cross-module expansion has gaps | Expands the language before Core semantics are stable. | Macro expansion stage contract and hygiene tests. |
| async / await runtime semantics | **partial — codegen is skeleton-level** (`async_gen.rs`, ~377 LOC, 0 e2e fixtures) | **high — silent failure risk**: `A F foo() -> T { ... }` parses and compiles, but the state-machine lowering is incomplete. Programs may build and then misbehave at runtime. | Runtime and lowering complexity beyond Core. | MIR/runtime contract and deterministic fixtures. |
| closures beyond simple certified cases | partial (~369 LOC, 2–3 simple-case e2e fixtures) | high — capture analysis exists; closures cannot escape (no first-class storage of the closure value past the inline call) | Existing closure inference work is still active. | Closure parameter/return type invariant and fixtures. |
| first-class function pointers | stub (~370 LOC, 0 e2e fixtures) | high — `Token::Fn` is recognized, but there is no `FunctionPointer` type in the type system | Marked unsupported in existing safe subset. | New RFC and parser/type/codegen tests. |
| `drop` / auto-free semantics | none (drop calls are explicitly disabled) | n/a — does not run; resource cleanup must be manual | Existing docs say drop calls are disabled. | Ownership/destructor design and run-time tests. |
| unsafe blocks and FFI-heavy patterns | partial (`ffi.rs`, ~763 LOC, 0 e2e fixtures) | high — works for the bounded patterns the compiler internally uses; user-facing FFI is not bounded by audit | Safety story is not Core-ready. | FFI safety contract and negative tests. |

<!-- inventory:auto-start -->

<!-- Generated by scripts/render-excluded-features.py from master-plan.toml v45. -->
<!-- DO NOT HAND-EDIT this section. Edit master-plan.toml then run the script. -->

## Active promotion candidates

Active promotion candidates are tracked exclusively in ROADMAP Master Plan v45 §Phase A2 with formal subset predicates. Each requires lifecycle stage 1 (impact preflight) before assignment.

As of this plan revision the A2 candidates are:

- ? operator subset (Result/Option, Core-typed, cross-module per real vaisdb baseline sql/types.vais:339 → storage/bytes.vais:40)
- dyn / trait object dispatch (narrow subset matching baseline)
- closures (no escape, inline-only)
- function pointer types in std API (bounded to existing std use)

Subset definition format: `compiler/docs/certification/A2_SUBSETS.md` (Order step 9 deliverable).

## Removal queue (A4) — 11 verified entries

Implicit-behavior surfaces verified to silently accept user code that should be rejected. Each candidate site was probed with a `.vais` fixture, compiled with `vaisc`, executed, and the runtime exit code compared to the expected value.

**Status**: v1-verified (single-sentinel discovery 2026-05-03); v2 retro-validation pending Order step 7. v17 (2026-05-04): A4-03/05/07 reclassified to Controlled per F-16/F-17/F-21 (IR-internal lowering, not user-facing source semantics). v23 (2026-05-05): A4-13 added — Box ↔ T silent accept at type-check call site (reclassified from rejected per STEP7 F-24 + permanent fixture A4-13_box_t_auto_unwrap). v25 (2026-05-05): empirical re-audit on all 12 fixtures — 8 entries are strict-default LANDED (env opt-out only), A4-13 is the sole remaining still-silent entry, A4-03/05/07 stay Controlled per v17 (silent or runtime-correct by design). A4-10 / A4-11 added to inventory (strict-default LANDED but missing from schema).

Codemod dependency: Order step 2.

### A4-runtime-silent (9 entries) — type checker accepts; runtime produces wrong result

| ID | Surface | Site | Probe | Expected | Actual exit |
|---|---|---|---|---|---|
| A4-01 | Unit ↔ i64 (void return as i64) | `unification.rs:361` | x: i64 = void_fn() | 0 | 96 |
| A4-02 | Pointer<T> ↔ i64 | `unification.rs:410` | take_i64(p) where p: *i64 | 42 | 184 |
| A4-04 | Pointer<T> ↔ Slice<T>/SliceMut<T> (Phase 162) | `unification.rs:417` | take_slice(p as *u8) | 4 | 72 |
| A4-06 | Integer truthy (4 sites: else-if, ternary, if, while) | `control_flow.rs:188,243,273,396` | I x { 100 } EL { 200 } where x: i64 | design violation (no `!= 0` token) | 100 — runtime correct, design violated |
| A4-10 | Struct partial-init (silent acceptance of missing fields) | `vais-types struct constructor checker` | S Person { name: str, age: i64 }; let p: Person = Person { name: "x" } | E001 missing fields | (historical) silent accept; struct created with garbage age field |
| A4-11 | Try operator (?) in function returning non-Result/Option | `vais-types try expression checker` | F take(x: i64) -> i64 { x? } | E001 expected Result<_,_> or Option<_> | (historical) silent accept; ? lowered as identity |
| A4-13 | Box<T> ↔ T auto-unwrap (call site) | `vais-types/checker_expr/calls.rs (Box<T> arg accepted where T expected)` | F take_i64(x: i64) -> i64 { R x }; F take_box(b: Box<i64>) -> i64 { R take_i64(b) } | E001 Type mismatch (Box<i64> is not i64) | OK No errors found (silent accept; type checker auto-unwraps Box<T> at call site) |
| A4-14 | Vec<T> ↔ &[T] permissive (.len() path) — runtime corruption | `vais-types/inference/unification.rs (Vec ↔ Slice arm)` | fn len_of(s: &[i64]) -> i64 { s.len() as i64 }; main: v: Vec<i64> = [1,2,3]; len_of(&v); expects 3 | exit 3 (Vec length) | exit ≠ 3 (memory-load corruption — fat-pointer Vec layout misread as Slice header) |
| A4-15 | Escape closure: closure returned from function loses captured environment | `vais-types/checker_expr/stmts.rs Stmt::Return + vais-types/checker_fn.rs check_function trailing-expression branch (TC detector)` | fn make_adder(x: i64) -> |i64| -> i64 { |n| n + x } | E001 'expected named function pointer or capture-free closure, found escape closure capturing N variable(s)' | rejected at TC layer (default-strict, opt-out via VAIS_REJECT_A4_15=0) |

### A4-late-codegen-silent (2 entries) — type checker accepts; codegen/linker fails with obscure error

| ID | Surface | Site | Probe | Failure |
|---|---|---|---|---|
| A4-08 | Vec<T> ↔ &T permissive | `unification.rs:384` | take_str_ref(v) where v: Vec<i64> | clang IR mismatch ({ptr,i64} vs ptr) — late codegen failure |
| A4-09 | Lifetime ref erasure (function definition) | `unification.rs:450` | F take_lifetime_ref<'a>(r: &'a i64) -> i64 { ... } and call with plain &i64 | linker undefined symbol _take_lifetime_ref |

All entries are v1-discovery (single-sentinel exit-code probe). v2 retro-validation is mandatory at Order step 7 (multi-sentinel + stdout assertion + permanent fixture in `compiler/tests/empirical/`).

## Controlled coercions (11 entries — verified, NOT A4)

Sites where `unification.rs` accepts the conversion AND runtime behavior is empirically correct. **Not A4 candidates.**

**Status**: v1-verified (single-sentinel); v2 retro-validation pending Order step 7. v17 (2026-05-04): A4-03/05/07 added — IR-internal lowering, opt-in strict via VAIS_REJECT_A4_{03,05,07}=1.

| Surface | Site |
|---|---|
| Str/str/String alias | `unification.rs:70-86` |
| Unknown unify-any | `unification.rs:220` |
| Never unify-any | `unification.rs:224` |
| Fn ↔ FnPtr | `unification.rs:282` |
| Numeric widening (runtime correctness only) | `unification.rs:346` |
| &Vec ↔ &[T] | `unification.rs:173` |
| DynTrait dispatch | `unification.rs:567` |
| Linear/Affine wrapper erasure (covers `x := affine T` / `x := linear T` annotation — wrapper unwrapped at unify, no use-count enforcement; A2-06 reclassified here v36) | `unification.rs:512-518` |
| Auto-deref &T ↔ T (IR-internal generic-inference glue, F-17) | `unification.rs:672` |
| Array → Pointer decay (IR-internal lowering of array-indexing to pointer-arith, F-16) | `unification.rs:498-511` |
| Default-i64 literal → context integer type (Rust-style polymorphic-literal analogue, F-21) | `unification.rs:351` |

## Rejected at type-check (3 entries — NOT A4, NOT Controlled)

Sites where `unification.rs` has the coercion code but a separate compiler stage (type checker for downstream usage, field access, or direct `unify` rejection) catches the misuse before codegen. The user already gets a stable diagnostic. **Not A4 candidates.**

| Surface | Site | How rejected |
|---|---|---|
| Box raw generic (no type param) | `unification.rs:114` | Field access fails with E030 (Site 02) |
| Optional ↔ T (3 paths probed) | `unification.rs:98 + Named bridge unification.rs:232-244` | Direct E001 on all 3 probes: Site 23 (bare i64 → Option<i64>), Site 23b (Named form), Site 23c (reverse — explicitly NOT-allowed direction per Phase 276 comment, but rejection serves as downstream defense evidence) |
| Result ↔ Unit (auto Ok/Some wrap) | `unification.rs:366` | Direct E001 Type mismatch — empirical fixture compiler/tests/empirical/Untested/U-01_result_unit_auto_ok/ verifies the type checker rejects bare Unit where Result<T,E>/Option<T> expected. Reclassified from Untested to Rejected on 2026-05-04 (STEP7_FINDINGS F-13). The fixture remains under empirical/Untested/ for now; physical relocation is cosmetic and deferred. |

## Untested / classification deferred (0 entry)

Sites where the empirical fixture could not produce a probe that exercises the suspected silent path. **Treat as A4 candidate by default** until a fixture proves Controlled or Rejected.

(empty — all surfaces previously listed here have been reclassified into A4 / Controlled / Rejected based on empirical evidence.)

<!-- inventory:auto-end -->

## Empirical verification protocol (v2 — coincidence-resistant)

For any future addition or removal in the A4 / Controlled / Rejected
lists, the verification protocol is:

1. Write a minimal `.vais` fixture that exercises the suspected coercion
   at a single site.
2. **Multi-sentinel oracle (NEW v6)**: the fixture must compute the
   expected value from at least 2 distinct inputs and aggregate them
   into a value `> 255`. Shell exit code is 8-bit (0-255), so a single
   sentinel can coincide with a garbage byte. Two-input aggregation
   makes coincidence ~1/65,536. Example: `take_i64(x)` where x = 42
   should not just return 42; aggregate two probes into `42 * 1000 +
   second_probe` so a coincidental garbage exit is detectable.
3. `vaisc check <file>` — record output.
4. `vaisc <file>` — record output.
5. `./<binary>; echo "exit=$?"` — record runtime exit code.
6. **Stdout assertion (NEW v6)**: where possible, the fixture also
   prints intermediate values to stdout. Diff stdout against a fixed
   expected string. Stdout is not byte-limited like exit code.
7. Classify by combining steps 3-6:
   - type-check fails (`vaisc check` reports `error[CODE]`) →
     **REJECTED** (already safe, not A4).
   - type-check passes + binary built + runtime exit matches multi-
     sentinel expected + stdout matches → **CONTROLLED** (not A4).
   - type-check passes + binary built + runtime exit / stdout differs
     → **A4-runtime-silent** (record actual vs expected).
   - type-check passes + binary build fails (clang/linker error) →
     **A4-late-codegen-silent**.
   - type-check passes + binary build succeeds + cannot construct a
     probe that exercises the suspected silent path →
     **Untested / deferred** (treat as A4 candidate by default).

8. **Permanent fixture suite (NEW v6)**: the fixture used for any
   classification must be checked into the repository under
   `compiler/tests/empirical/<surface>.vais` (directory created at
   Order step 2 deliverable ). Transient fixtures
   in `/tmp/` are not acceptable evidence for permanent classification
   — only initial discovery.

### Assertion-kind tri-form (NEW v7 — Step 7 first iteration F-05)

Step 7's first retro-validation iteration discovered that the v6 single
"runtime exit matches expected" form is too tight for some A4 surfaces.
Memory-load-corruption surfaces (the runtime returns a value derived
from `load i64` against an address that was not supposed to be read as
i64) produce exit codes that depend on stack/heap layout, optimization
level, and OS — not on the unification rule itself. v1 single-sentinel
exit codes for these surfaces (e.g. master-plan v1 says A4-02 = 184) do
not generalize across environments (macOS arm64 release observed 56).

The fixture's `meta.toml` `[assertion_kind]` block selects ONE of three
forms; the runner enforces only the selected form:

- **`exact_exit`** — runtime exit code must equal `expected.txt`
  exactly. Use when the runtime observable is a source-named constant
  (e.g. A4-01: void slot returns LLVM-default 96; A4-06: truthy branch
  returns the literal 100; A4-07: widened literal returns 42). Stable
  across environments because the value is determined by source, not
  memory layout.

- **`exit_not`** — runtime exit code must **NOT** be in a `forbidden_set`
  that enumerates the value(s) the well-typed program (with the surface
  correctly rejected) would have returned. Use for memory-load-corruption
  surfaces (A4-02, A4-03, A4-04, A4-05). Example for A4-02:
  `forbidden_set = [42]` — the well-typed `take_i64(*p)` where val=42
  would return 42, so the runner FAILS if exit lands on 42 and PASSES
  for any other value.

  **Collision caveat (codex v1 review)**: shell exit is 8-bit (0-255)
  so a corrupted memory load can coincidentally hit a value in
  `forbidden_set`. When a hit occurs the runner reports
  `DRIFT: A4-NN exit landed on forbidden value F` and exits 1 —
  treat as drift requiring investigation, NOT proof that the surface
  was fixed. Investigation distinguishes (a) the surface was actually
  fixed (good news, migrate fixture to negative form) from (b) the
  corruption coincided with the well-typed value (re-craft probe to
  use a forbidden value harder to hit, e.g. `42_000_000_007 % 256` or
  a struct field at a stable offset).

  A4-08 (Vec<T> ↔ &T permissive) is **not** an `exit_not` candidate —
  the surface persists at type-check (`unification.rs:384`) but its v1
  build-time symptom drifted to a runtime SIGSEGV. A4-08 uses the
  fourth assertion form `runtime_crashes` introduced below (see
  STEP7_FINDINGS F-06 for the symptom-drift discovery).

- **`build_fails`** — `vaisc check` passes but the full build (codegen,
  link) exits non-zero, and stderr matches every regex in a list of
  required patterns. Use for late-codegen-silent surfaces (A4-09
  linker undefined symbol).

  **Specificity requirement (codex v1 review)**: each pattern in
  `required_stderr_patterns` must distinguish the documented failure
  from incidental ones (e.g. duplicate-symbol, redefinition, missing
  library). For A4-09 the patterns combine the specific mangled symbol
  name (`_take_lifetime_ref`) AND a clang/ld message form that is
  specific to undefined-symbol failures — `Undefined symbols.*for
  architecture` plus `ld: symbol\(s\) not found` — not the looser
  `undefined|symbol|linker|ld:` which would match unrelated linker
  errors. Each fixture's `meta.toml` documents the rationale for its
  patterns.

- **`check_fails`** — `vaisc check` itself exits non-zero AND stderr
  matches every regex in a list of required patterns. Use for
  Rejected-class surfaces: the type checker (or a downstream pre-codegen
  pass) catches the misuse before any IR is emitted. Distinct from
  `build_fails` because the failure happens at `vaisc check`, not after
  it (no build needed).

  Required `meta.toml` fields:
  - `required_stderr_patterns` — same form as `build_fails`, must
    distinguish the documented rejection from incidental check errors.

  Use for: Box raw generic (E030), Box ↔ T (E001), Optional ↔ T (E001).

- **`runtime_crashes`** — `vaisc check` passes, build succeeds, but
  runtime exits with a specific signal-class exit code when the
  defective surface is actually exercised (parameter consumed, etc.).
  Use for surfaces whose v1 build-time symptom drifted to runtime
  (A4-08 Vec<T>↔&T permissive: build now succeeds, runtime SIGSEGVs
  when the misinterpreted &str is read).

  Required `meta.toml` fields:
  - `expected_exit = 139` (or other signal-class code: 134 SIGABRT,
    136 SIGFPE, 137 SIGKILL, 138 SIGBUS — pick the one observed and
    document why that signal corresponds to the surface).
  - `consuming_probe_required = true` — explicitly documents that
    the probe must consume the misinterpreted parameter (a
    non-consuming probe whose body returns a constant masks the
    defect; see STEP7_FINDINGS F-06).

  This is a fourth assertion kind separate from `build_fails` because
  the failure mode is fundamentally different — clang/ld would have
  produced an error message; SIGSEGV produces no diagnostic, only an
  exit code, which is exactly the "silent" property the A4 class
  documents.

The `meta.toml` schema additions:

```toml
[assertion_kind]
kind = "exact_exit" | "exit_not" | "build_fails" | "runtime_crashes" | "check_fails"

# Required when kind = "exit_not":
forbidden_set = [42]  # if exit lands on any of these, runner exits 1
                       # with DRIFT — investigate (fix vs collision).

# Required when kind = "runtime_crashes":
expected_exit = 139            # signal-class exit code (139 = SIGSEGV).
consuming_probe_required = true  # probe must actually consume the
                                  # misinterpreted parameter; a body that
                                  # ignores it masks the defect.

# Required when kind = "build_fails":
required_stderr_patterns = [
  "_take_lifetime_ref",                    # specific mangled symbol
  "Undefined symbols.*for architecture",   # specific clang form
  "ld: symbol\\(s\\) not found",           # specific ld form
]
```

The runner script (`run.sh`) reads the kind and applies the matching
assertion. The previous "exit code matches expected.txt" form is
preserved as the default `exact_exit` for backward compatibility with
already-landed fixtures (A4-01, A4-06, A4-07 all use `exact_exit`;
A4-09 uses `build_fails`).

This protocol revision unblocks fixture creation for the 5 memory-load-
corruption A4 entries (A4-02 through A4-05 and A4-08), which were
deferred during Step 7's first iteration. Subsequent iterations land
those fixtures using `exit_not`.

This protocol is the only acceptable evidence for A4 / Controlled /
Rejected classification. Speculation from `unification.rs` reading
alone is insufficient (codex v5 review's 22+ speculative citations had
~60% false-positive rate against this protocol's single-sentinel
predecessor).



## Silent-failure risk surface (use with caution)

These features compile user code without an error today, but are not
covered by any active gate. Failures may surface only at runtime:

- **async/await** — codegen is skeleton; await points are not fully
  lowered. Highest priority to either complete or reject at parse.
- **closures escaping inline use** — capture analysis runs; storing the
  closure value can fail later.
- **trait objects / `dyn Trait`** — parses; dispatch resolution has gaps.

Until each item has a certified gate, treat user code that depends on
them as experimental.

## Deferred Syntax Forms

| Syntax | Reason | Replacement in Core |
|---|---|---|
| legacy context-dependent `E` for new enum/else fixtures | Ambiguous for agents and docs. | Use `EN` for enum and `EL` for else. |
| tuple structs and tuple variant multi-field binding | Existing docs mention parser/codegen gaps. | Use named struct fields or single-field variants. |
| match returning `str` in complex arms | Existing safe subset warns about phi type mismatch. | Use if/else or bind strings before match until certified. |
| `Vec<struct>[i].field` direct access | Existing safe subset lists erasure/field access hazards. | Bind indexed value to a temporary in non-Core code. |

## Deferred Stdlib and Runtime Domains

| Area | Reason | Promotion gate |
|---|---|---|
| networking and HTTP server stdlib | Broad platform/runtime surface. | Separate integration gate after Core. |
| websocket, oauth, yaml, GPU/OpenCL helpers | Not needed for compiler Core. | Package-specific certification. |
| async platform backends | Runtime-specific and difficult to prove early. | MIR/runtime design plus platform tests. |
| package registry and playground server | Product tooling, not compiler correctness. | Auxiliary service gate. |

## Experimental Compiler Crates

The `compiler/docs/CRATE_AUDIT.md` experimental tier remains outside Core
v0: JIT, GPU, GC, JS codegen, hot reload, dynamic loading, profiler,
registry server, playground server, tutorial, Python/Node bindings, query,
testgen, supply-chain, and security crates.

These crates may continue to exist and build opportunistically, but they
do not block Core certification unless explicitly promoted.

## Current Certification Exclusion Audit

`tests/core/certification_exclusions.tsv` is the machine-readable source
of truth for ignored tests, plus any future partial markers, that still
appear inside the canonical certification gate.
`core_certification_exclusion_manifest_is_current` fails if:

- a new `#[ignore]` appears in the audited gate files without a manifest
  entry,
- a known ignore or partial marker is removed without updating the
  manifest,
- an ignore reason changes silently,
- `tests/core/mir_deferred.tsv` gains a deferred Core fixture.

The manifest may be empty when the audited quarantine surface is empty.
This keeps temporary quarantine visible while preserving the narrower Core
v0 proof boundary.

For the current dated pass/fail evidence, use `CURRENT_STATUS.md`. This
file defines what remains outside Core; it is not a substitute for a fresh
gate run.

## Downstream Ecosystem

`lang/packages/vaisdb`, `lang/packages/vais-server`, and
`lang/packages/vais-web` are promotion gates after Core. They are not the
first proof of language correctness.

The order is:

1. Core v0 fixtures pass.
2. Type/codegen boundary invariants pass.
3. MIR structural validation passes for the lowered Core subset.
4. Reference interpreter and LLVM execution agree for Core run fixtures.
5. Downstream packages are reintroduced one at a time.

## Reintroduction Rule

To reintroduce an excluded feature, create a task with:

- a short design document,
- exact Core or non-Core status,
- positive and negative fixtures,
- required compiler stage,
- same-class audit command,
- rollback trigger if any existing Core fixture regresses.
