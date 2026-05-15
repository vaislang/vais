# Step 13 (A4 removals) findings

This file records empirical findings during Order step 13 (A4 removal queue).
Mirrors STEP7 / STEP10 / STEP11 / STEP17 structure.

Master Plan v103 §[phase.A4] inventory (12 entries total):
- strict_default_landed (11): A4-01 / A4-02 / A4-03 / A4-04 / A4-06 / A4-08 / A4-09 / A4-10 / A4-11 / A4-13 / A4-15
- specified_safe_landed (1): A4-14
- still_silent (0): none

2026-05-11 update: A4-03 remains default-silent, but strict audit mode now
rejects concrete Ref/value mismatches and the baseline impact is reduced to
54 vaisdb files. `std/` is strict-clean under `VAIS_REJECT_A4_03=1`.

2026-05-12 update: A4-03 strict-mode migration is clean for all vaisdb files
(`261/261`). Default legacy behavior remains silent pending default flip
measurement and runtime subset recheck.

2026-05-12 default update: A4-03 is default-strict. `VAIS_REJECT_A4_03=0`
is now the legacy drift opt-out, and full integrity is green.

Step 2 LANDED `vaisc fix --explicit --site=A4-NN [--dry-run]` skeleton.
A4-01 detection is implemented; A4-02..A4-09 stubbed `NotImplemented`.

## Index (F-NN → 한 줄 요약)

| ID | 한 줄 요약 |
|---|---|
| F-13-01 | A4-01 (Unit ↔ i64) baseline scan: 0 findings across 404 .vais files (2026-05-05) |
| F-13-04 | A4-03 strict audit narrowed: std 82/82, vaisdb 207/261; default baseline unchanged (2026-05-11) |
| F-13-05 | A4-03 strict vaisdb migration clean: std 82/82, vaisdb 261/261; default baseline unchanged (2026-05-12) |
| F-13-06 | A4-03 default-strict landed: full integrity OK; A4 still_silent=0 (2026-05-12) |

## Findings

### F-13-06 — A4-03 default-strict landed (2026-05-12)

Scope:
- `crates/vais-types/src/inference/unification.rs`
- `crates/vais-types/src/checker_fn.rs`
- `crates/vais-types/src/checker_expr/stmts.rs`
- `crates/vais-codegen/src/inkwell/gen_expr/misc.rs`
- `compiler/std/`
- `docs/language/LIVING_SPEC/`
- `lang/packages/vaisdb/src/`
- `lang/packages/vais-server/src/`
- `tests/empirical/A4/A4-03_auto_deref/`

Result:
- default A4-03 behavior: strict reject with E001 (`expected i64, found &i64`)
- legacy opt-out: `VAIS_REJECT_A4_03=0` accepts the old probe for drift
  investigation only
- positive explicit deref: `positive.vais` exits 42
- `std`: `82/82`
- `LIVING_SPEC`: `117/117`
- `vaisdb`: `261/261`
- `vais-server` runtime: `16/16`
- full integrity: `INTEGRITY OK`

Compiler-side change:
- A4-03 unification is default-strict; legacy behavior requires
  `VAIS_REJECT_A4_03=0`.
- Function, method, trailing-expression, and return-statement hidden
  auto-deref bypasses were removed.
- Inkwell explicit deref now treats already-materialized aggregate pointee
  values as no-op, avoiding ICE when source writes `(*self)`.

Source migration performed:
- Builder-style self returns use explicit `(*self)`, not implicit return of
  `self`.
- `(*self)` is parenthesized because a line-start `*self` can be parsed as
  multiplication with the previous line in current grammar.
- LIVING_SPEC HashMap/Vec examples now use explicit borrowed keys and deref.
- vais-server router/header generic Vec locals were annotated to avoid
  unspecialized `%Vec` monomorph drift in promoted runtime smoke tests.

Status: A4-03 is default-closed. The A4 queue now has 0 still-silent entries.

### F-13-05 — A4-03 strict vaisdb migration clean (2026-05-12)

Scope:
- `lang/packages/vaisdb/src/`
- `tests/empirical/A4/A4-03_auto_deref/`

Result:
- strict `vaisdb`: improved from `207/261` to `261/261`
- default `vaisdb`: remains `261/261`
- default A4-03 behavior: still silent for the legacy fixture at this point;
  resolved by F-13-06.

Source migration performed:
- Made the remaining planner, SQL executor/parser, RAG, security/server,
  storage/WAL, and vector/HNSW ref/value contracts explicit.
- Replaced clone-through-reference and reference-return ambiguity with explicit
  `(*x).clone()` and `&self.items[i]` forms.
- Normalized HashMap key APIs to by-value `contains_key` and borrowed `get`
  where required.

Remaining work:
- Resolved by F-13-06; default flip and promoted runtime subset recheck are
  complete.

### F-13-04 — A4-03 strict audit narrowed (2026-05-11)

Scope:
- `crates/vais-types/src/inference/unification.rs`
- `compiler/std/`
- `lang/packages/vaisdb/src/`
- `tests/empirical/A4/A4-03_auto_deref/`

Result:
- default A4-03 behavior: still silent for the legacy fixture
- strict A4-03 behavior: `VAIS_REJECT_A4_03=1` rejects the fixture with E001
  (`expected i64, found &i64`)
- strict `std/`: improved from `78/82` to `82/82`
- strict `vaisdb`: improved from `114/261` to `207/261`
- default `std/`: `82/82`
- default `vaisdb`: `261/261`

Compiler-side change:
- A4-03 strict mode now rejects only fully concrete Ref/value mismatches.
  Unifications containing unresolved inference variables remain generic
  inference glue rather than user-facing implicit deref decisions.
- A4-03 diagnostics now report the concrete expected/found types instead of
  the placeholder `&T`.

Source migration performed:
- `std/set.vais` set algebra APIs now take `&Set` explicitly.
- `vaisdb` ByteBuffer readers, metadata snapshots, selected clone-on-reference
  sites, and shared storage/graph/fulltext references were made explicit.

Remaining work:
- 54 `vaisdb` files still fail strict A4-03. The dominant categories are
  `Option<&T>` / `Vec<&T>` return contracts whose bodies produce owned values,
  planner/SQL/Expr value-vs-reference calls, RAG reference collections, and a
  few server/storage call signatures.
- Resolved by F-13-05; the 54 remaining strict vaisdb files are now 0.

Status: A4-03 is **not** default-closed. It is now a smaller, explicit source
migration queue with a strict audit gate and no default baseline regression.

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
| A4-13 | **strict reject** (env opt-out: `VAIS_REJECT_A4_13=0`) | strict_default_landed (v99) |
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
   compiler-side gap was A4-13 until v99. A4-13 now rejects the direct
   Box<T> to T call-site form with E001 while keeping reference-level
   &Box<T> to &T projection. The vaisdb baseline was migrated to explicit
   `.as_ref()` recursive AST/plan calls so this stricter rule keeps
   `INTEGRITY vaisdb_files pass=261 fail=0 total=261`.

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

### F-13-03 — A4 inventory rewrite LANDED (master-plan v25, 2026-05-05)

F-13-02 reported the drift between master-plan v24 [[phase.A4]] inventory
and the compiler's actual default-mode behavior. This finding records the
close.

Empirical re-audit results — all 12 A4 fixtures run from
`compiler/tests/empirical/A4/`:

| Fixture | Result line (last line of run.sh output) |
|---|---|
| A4-01_unit_i64 | rejects probe with E001 (silent surface removed; default-mode strict) |
| A4-02_pointer_i64 | rejects probe with E001 (silent surface removed; default-mode strict) |
| A4-03_auto_deref | type-checks, compiles, runs, exits 168 (≠ forbidden 42 — silent corruption confirmed) |
| A4-04_pointer_slice | rejects probe with E001 (silent surface removed; default-mode strict) |
| A4-05_array_pointer_decay | rejects probe with E001 (Pointer→i64 caught at type-check; A4-05's own surface still opt-in pending hnsw/cow.vais migration) |
| A4-06_integer_truthy | positive (`!= 0`) exits 100; negative (integer-as-truthy) rejected at vaisc check |
| A4-07_numeric_widening | type-checks, compiles, runs, exits 42 (runtime correct, design pending) |
| A4-08_vec_str_permissive | rejects probe with E001 (silent surface removed; default-mode strict) |
| A4-09_lifetime_ref_erasure | rejects probe with E001 (silent surface removed; default-mode strict) |
| A4-10_struct_partial_init | rejects probe with 'missing fields: age' (silent surface removed; default-mode strict) |
| A4-11_try_in_non_result | rejects probe with E001 'expected Result<_,_> or Option<_>' (silent surface removed; default-mode strict) |
| A4-13_box_t_auto_unwrap | rejects Box<i64> where i64 expected with E001 (strict default; `VAIS_REJECT_A4_13=0` legacy opt-out) |
| A4-14_vec_slice_len | still silent; runtime exit differs from expected Vec length |
| A4-15_escape_closure | rejects escaping capture closure with E001 (silent surface removed; default-mode strict) |

Classification (current as of master-plan v99):

- strict_default_landed (10): A4-01, A4-02, A4-04, A4-06, A4-08, A4-09,
  A4-10, A4-11, A4-13, A4-15.
- still_silent (2): A4-14, A4-03.
- controlled (2): A4-05, A4-07. A4-03 was reclassified back into A4 in v47.

Schema changes in master-plan v25 (informational, no checker impact):
per-entry current_status + current_evidence; [phase.A4] gains
strict_default_landed_count + still_silent_count; A4-10 / A4-11 added.

Result: PLAN CONSISTENCY OK; INTEGRITY OK; check-empirical 35 PASS;
DEFERRED #23 closed.
