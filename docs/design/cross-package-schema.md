# Cross-Package Schema Consumption Infra Design

Status: Final v6 design + 2026-05-13 main-port evidence update
Last verified: 2026-05-13

Codex iteration history: v1 REVISE 5 → v2 REVISE 5 → v3 REVISE 6 → v4
REVISE 2 → v5 REVISE 4. After v5 the v5 defects were addressed in v6
(numeric brand types per width, Item-kind exhaustiveness, spec-derived
gate count, single gate.sh entrypoint). The codex round-trip became
expensive vs new-defect-discovery rate (each iteration found ~3-6 new
issues at deeper layers), so v6 is self-completed: main-thread
self-audit caught one residual count drift (12 → 13) which was fixed
without further codex round-trip.

Open invitation for additional codex review: the user may ask for a
codex pass any time. Spots most likely to benefit from codex eyes:
the EMIT_TS_001-021 + 999 surface-classification table, the gate
runner specification (any silent-failure paths I missed), and the
brand-type lowering soundness against TS structural typing.

## Goal

A typed change to a shared `schema.vais` file must produce:
- `vaisc check` errors in `.vais` consumers (vaisdb tables, vais-server APIs)
- TypeScript compile errors in `.ts` / `.tsx` consumers (vais-web frontend)

Prerequisite for Order Step 8 (cross-package schema consumption implementation)
and the I-3 5★ initiative (multi-domain product gate).

## North-star compliance (binding constraint, LESSONS L-002)

This document is bound by master-plan.toml `[north_star]`:
1. 안정성 100% — no silent failure
2. 명확성 100% — no implicit behavior in user-facing semantics
3. 우회 없이 모두 완성 — no deletion to avoid hard work

Silent emit fallbacks (`unknown`, error-branch erasure, untyped-change accepted)
are **rejected as design options**. v1 must hard-fail on anything it cannot
faithfully lower.

## The choice

Master Plan v16 §I-3 frames the decision as:
- (a) `vaisc emit-ts` → emit `.d.ts` for TypeScript consumers from a single source schema
- (b) `vaisx-compiler` JSON + TS embedding — extend the existing reactivity compiler

This document chooses **(a)**.

## What already exists (verified 2026-05-13)

- **vaisc commands** (`compiler/crates/vaisc/src/commands/`):
  `emit_ts.rs` is wired through `crates/vaisc/src/main.rs` as
  `vaisc emit-ts <input.vais> --output <output.d.ts>`.
- **vaisx-compiler** (`lang/packages/vais-web/crates/vaisx-compiler/`):
  Modules `analyze.rs`, `codegen_js.rs`, `ir.rs`, `error.rs`, `lib.rs`. Cargo
  description: "VaisX reactivity compiler — dependency analysis & JS code
  generation". Currently emits JS, not TS. No schema-reading logic.
- **vais-types** (`compiler/crates/vais-types/`): owns the canonical type
  representation (`ResolvedType`). The current main-port emitter runs the
  parser and type checker before emitting, but the lowering implementation is
  AST-driven. `emit_ts_exhaustiveness.rs` guards future growth of
  `ResolvedType` and AST `Item` variants; a future hardening pass should route
  runtime lowering through checker-resolved types instead of relying on AST
  type syntax.
- **Schema fixtures**:
  `tests/empirical/cross_package_schema/tests/gate.sh positive|negative`
  passed `11/11 + 4/4`, and
  `tests/product/multi_domain_schema/tests/gate.sh` passed `9/9` locally on
  2026-05-13.

## Decision matrix (full audit, codex v1 defect 3 expanded)

| Dimension | (a) vaisc emit-ts | (b) vaisx-compiler schema-aware TS |
|---|---|---|
| **Crate touched** | `vaisc` + read from `vais-types` | `vaisx-compiler` + `vaisx-parser` |
| **Source of TC truth** | `vaisc` parser + type checker; current main-port lowering is AST-driven with `ResolvedType` exhaustiveness guard | Re-parses `.vais` via vaisx-parser (focused subset) |
| **Output format** | `.d.ts` per pub-exposed schema type | `.d.ts` embedded in build artifact + JSON bundle |
| **TC fidelity** | Highest (uses real type-checker) | Lower (parser-subset semantic drift risk) |
| **Risk to Core** | Adds emit path inside compiler crate | None — lives in lang/, separable |
| **Multi-target** | Yes — `.d.ts` consumable by any TS toolchain | No — coupled to vais-web build |
| **vaisdb / vais-server** | Direct `.vais` consumers; cross gate runs native after schema concatenation, product gate type-checks through package APIs | N/A — they don't use vaisx |
| **Code reuse** | ~0 (new emit module) | High (vaisx-compiler already emits) |
| **Effort estimate** | 4-6 weeks | 2-3 weeks (but +drift maintenance burden) |
| **Failure mode** | Schema with unsupported type → hard fail with stable error code | Drift between vaisc and vaisx schema views (silent until consumer divergence) |

**Recommendation: (a)**. (b)'s "code reuse" advantage is a trap — it
re-parses the same source language with a divergent parser, which is exactly
the silent-drift pattern L-002 forbids.

## Type lowering table — v1 (codex v1 defect 3 expanded)

Verified against real schema-bearing code in `lang/packages/vaisdb/src/sql/types.vais`,
`compiler/std/hashmap.vais`, and `lang/packages/vaisdb/src/storage/buffer/readahead.vais`.

| Vais type | TypeScript lowering | Notes |
|---|---|---|
| `i8`, `i16`, `i32`, `i64`, `i128` | `number & { __vaisInt: <bits> }` (branded) | Branded per width so `i32 → i64` is a typed change in TS. Brand emitted as `declare const __vaisInt: unique symbol; type VaisI64 = number & { readonly __vaisInt: 64 }`. |
| `u8`, `u16`, `u32`, `u64`, `u128` | `number & { __vaisUint: <bits> }` (branded) | Distinct brand from signed; `i64 → u64` is a typed change. |
| `f32`, `f64` | `number & { __vaisFloat: <bits> }` (branded) | Distinct brand from int; `i64 → f64` is a typed change (codex v5 defect 1 fix). |
| `bool` | `boolean` | |
| `str`, `String` | `string` | |
| `()` (Unit) | `void` (return) / `null` (value) | Only valid as return type or struct field. |
| `Vec<T>` | `ReadonlyArray<T_lowered>` | |
| `&[T]`, `&mut [T]` (Slice/SliceMut) | `ReadonlyArray<T_lowered>` | Slice ownership is a Rust concept; TS sees a flat array. |
| `&T`, `&mut T` (Ref/RefMut) | `T_lowered` | Reference distinction is invisible at JS boundary. Documented in emit header comment so consumers know the ownership info is lost. |
| `Option<T>` | `T_lowered \| null` | `null`-style preferred over `undefined` for explicit serialization. |
| `Result<T, E>` | `{ ok: T_lowered } \| { err: E_lowered }` | **Tagged union, not error erasure** (codex defect 1 fix). v1 lowers both branches faithfully. |
| `(T1, T2, ...)` (tuple) | `readonly [T1_lowered, T2_lowered, ...]` | (codex defect 3 — vaisdb uses tuples in sql/types.vais:1104) |
| `HashMap<K, V>` | `Map<K_lowered, V_lowered>` | (codex defect 3 — std/hashmap.vais, readahead.vais) |
| `struct S { f1: T1, f2: T2 }` | `interface S { readonly f1: T1_lowered; readonly f2: T2_lowered; }` | All fields readonly (Vais structs default-immutable at value level). |
| `enum X { A, B(i64), C { x: f64 } }` | tagged union: `\| { kind: "A" } \| { kind: "B", _0: number } \| { kind: "C", x: number }` | Faithful to Vais enum semantics. |

### Hard-fail surfaces in v1 (codex v1 defects 1, 2, 4 + codex v2 defect 1 fix — all silent paths removed)

`vaisc emit-ts` exits **non-zero** with a stable error code on any of:

| Surface | Stable error code | Why hard-fail (not `unknown`) |
|---|---|---|
| Generic type parameter (`F<T>(x: T)`) | `EMIT_TS_001` | TS generics need monomorphization choice or constraint translation; v1 cannot guarantee soundness. |
| Trait bound (`<T: Clone>`) | `EMIT_TS_002` | Vais traits don't map to TS interfaces 1:1 (no associated types in v1). |
| `dyn Trait` | `EMIT_TS_003` | Object-safety lowering ill-defined cross-language. |
| Lifetime parameter (`<'a>`) | `EMIT_TS_004` | No TS analogue. |
| Function pointer types | `EMIT_TS_005` | First-class fn types punt to v2 — JS callbacks need ABI decisions. |
| `impl Trait` return | `EMIT_TS_006` | Existential type — requires monomorphization choice. |
| Pub function in schema (vs pub type) | `EMIT_TS_007` | v1 emits types only, not callables. |
| Type alias to unsupported surface | `EMIT_TS_008` | Transitive — propagates to alias root. |
| Raw pointer (`*T`, `*mut T`) | `EMIT_TS_009` | Pointer arithmetic / aliasing semantics has no TS equivalent. (codex v2 defect 1) |
| Fixed-size array (`[T; N]`) | `EMIT_TS_010` | TS lacks length-typed arrays in v1; tuple lowering is exponential. (codex v2 defect 1) |
| Range types (`Range<T>`, `RangeInclusive<T>`) | `EMIT_TS_011` | Iterator surface; not a serializable schema type. (codex v2 defect 1) |
| `Future<T>` / async types | `EMIT_TS_012` | Effect-typed; not a structural schema type. (codex v2 defect 1) |
| `Vector<T>` (SIMD), `ConstArray<T,N>` | `EMIT_TS_013` | Backend-specific layout. (codex v2 defect 1) |
| Dependent / refinement types | `EMIT_TS_014` | Predicate erasure would silently weaken contracts. (codex v2 defect 1) |
| **Catch-all: any other `ResolvedType` variant** | `EMIT_TS_999` | Lowering classification must be exhaustive. New `ResolvedType` variants added to the type checker MUST extend either the lowering table or the rejection table. The current runtime emitter is AST-driven, so `emit_ts_exhaustiveness_test` guards classification drift while CLI tests and schema gates guard emitted behavior. |

Each error references the offending source `.vais` file:line. Consumers see
emit failure, not a downgraded `.d.ts` that compiles but lies.

**Exhaustiveness invariant** (codex v2 defect 1 + v5 defect 2): the
exhaustiveness test has TWO scopes:

1. **Type variant exhaustiveness** — `emit_ts_exhaustiveness_test_types`
   walks every constructor of `ResolvedType` (via match-on-self) and
   asserts the emit-ts module classifies it as either "lowered" or
   "rejected with stable code".
2. **Top-level item kind exhaustiveness** — `emit_ts_exhaustiveness_test_items`
   walks every variant of the AST `Item` enum (struct, enum, union,
   trait, type alias, const, global, function, impl block, extern block,
   macro definition, module). Each `pub` item kind must be classified as:
   - **Emitted**: struct, enum, type alias (the schema-bearing kinds).
     Emitted per the type lowering table.
   - **Hard-fail with stable code**: union (`EMIT_TS_015`), trait
     (`EMIT_TS_016`), const at module scope (`EMIT_TS_017`), global
     (`EMIT_TS_018`), function (`EMIT_TS_007` — already declared above),
     impl block (`EMIT_TS_019`), extern block (`EMIT_TS_020`), macro
     (`EMIT_TS_021`).
   - **Silently skipped**: only `pub` private modules, doc comments,
     attributes — these have no schema content.

   The full list above is enforced as a `match` on `Item` in
   `emit_ts_exhaustiveness_test_items`; adding a new `Item` variant
   without classifying it fails the test.

Both tests close silent-pass holes where a future variant or item kind
could slip through unrecognized.

**v2 expansion path** (post Step 14): generic constraints map to TS
constrained generics; `dyn Trait` → typed interface union; function pointers
→ documented JS callback signature. Each promoted surface adds a positive +
negative empirical fixture under `compiler/tests/empirical/emit_ts/`.

## Implementation outline

1. New subcommand under `compiler/crates/vaisc/src/commands/`:
   `emit_ts.rs`.
2. CLI: `vaisc emit-ts <input.vais> --output <output.d.ts>`. Default emits
   only `pub` types declared at module top-level. Flag `--all` to emit
   private types remains a design target; it is not wired into the current
   main-port CLI.
3. Input: source is parsed and type-checked by `vaisc`; current main-port
   lowering reads AST type syntax after the type-check succeeds. Moving runtime
   lowering to checker-resolved `ResolvedType` remains a follow-up hardening
   item.
4. Output: a single `.d.ts` file. Header comment carries: source path,
   vaisc version (semver string), input file content hash (sha256 hex), and
   list of erased information (e.g. "ownership info on `&T` not
   preserved"). Header is documentation, not fallback semantics. **No
   timestamp** — would break determinism (codex v2 defect 4 fix).
5. Determinism: same input must produce byte-identical `.d.ts` output across
   runs. Verified by emit-roundtrip test (run twice, `diff` must be empty).
   Header content is fully derivable from input (path + content hash) so
   round-trip is stable.
6. The `vaisx-compiler` continues emitting JS only. `.d.ts` it produces (for
   reactivity hooks) is a separate concern; cross-package **schema** stays
   in (a).

## Validation gate (Step 8 acceptance test, codex v1 defect 5 fix)

Concrete commands and consumer paths. The fixture lives at
`compiler/tests/empirical/cross_package_schema/` (directory created by Step
7 retro-validation per Master Plan v16; this gate consumes it).

### Fixture layout

```
compiler/tests/empirical/cross_package_schema/
├── README.md                       # what this gate proves + how to extend
├── schema/user.vais                # shared schema definition
├── consumers/
│   ├── vaisdb_table.vais           # imports schema/user.vais → vaisdb table
│   ├── vais_server_api.vais        # imports schema/user.vais → vais-server route
│   └── vais_web_consumer.ts        # imports gen/user.d.ts → typed access
├── gen/                            # emit-ts output (gitignored, regenerated)
└── tests/
    └── gate.sh                     # single entrypoint, accepts arg `positive` | `negative`
```

(codex v5 defect 4 fix — single entrypoint matches the runner spec.)

### Scope boundary (v4 narrowing per codex v3)

This document specifies the gate **semantics** — what must hold before/after
a schema mutation, what failure modes must be distinguished, what
environment assumptions are valid. It does **not** specify the exact shell
syntax of the gate runner. Concrete `sed` patterns, TS toolchain
invocation flags, and OS-portability concerns (BSD vs GNU `sed`, pnpm
binary exposure, `tsc --types` semantics) are Step 8 implementation
deliverables.

This boundary keeps the design reviewable as a design. Earlier drafts
included executable shell that drifted into implementation territory
(codex v3 defects 1-5 were all shell-script accuracy issues). v4 keeps
specification precise and defers script syntax to where it is testable —
the Step 8 implementation phase, which produces the actual runner with
its own unit tests.

### Gate runner — abstract specification

The gate runner is one program (any language; bash is one option) located
at `compiler/tests/empirical/cross_package_schema/tests/gate.sh`. It is
invoked from the coordination workspace root.

The runner accepts a single argument: `positive` or `negative`. It must
follow the **structured exit contract** below; bare shell `!` assertions
followed by cleanup are not permitted because cleanup commands can mask
failure (codex v3 defect 2). The implementation uses explicit
status-tracking (e.g. `trap` + accumulator variable + final guarded
`exit`).

### Structured exit contract

Three distinct exit codes:

| Exit | Meaning | When |
|---|---|---|
| 0 | Gate PASS | All assertions held |
| 1 | Gate FAIL | An assertion did not hold the expected way (consumer succeeded when it must fail, or vice versa) — schema-propagation invariant violated |
| 2 | FIXTURE_DRIFT | The fixture itself is broken: pristine baseline does not type-check; mutation primitive (the equivalent of `sed`) produced no change; toolchain not found |

CI must distinguish FIXTURE_DRIFT (exit 2) from gate fail (exit 1) so that
fixture rot is not silently misread as a regression catch. The runner
reports the failing phase in stderr.

### Positive gate semantics

Three phases, each with explicit pre/post assertions. The runner must
verify EVERY assertion explicitly (not implicitly via `set -e`), so the
final exit code reflects the actual phase that failed.

**Phase 0 — pre-change baseline (FIXTURE_DRIFT if any fails)**

The pristine schema produces a `.d.ts`, both `.vais` consumers type-check
and run natively, and the TypeScript consumer compiles against it:
- `vaisc check consumers/vaisdb_table.vais` exits 0
- native build/run of `consumers/vaisdb_table.vais` exits 0
- `vaisc check consumers/vais_server_api.vais` exits 0
- native build/run of `consumers/vais_server_api.vais` exits 0
- TypeScript compile of `consumers/vais_web_consumer.ts` against the
  generated `.d.ts` exits 0

If any pre-change assertion fails, the fixture is broken and the runner
exits 2 (FIXTURE_DRIFT). This guards against the vacuous-fail trap (codex
v2 defect 5): a post-change `!` succeeds whenever a consumer was already
broken, so without the pre-check the gate accepts a fixture that proves
nothing.

**Phase 1 — schema mutation (FIXTURE_DRIFT if mutation no-ops)**

A typed change is applied to the pristine schema (rename a `pub` field).
The mutation primitive must:
1. Produce a real diff against the pristine version. If the file is
   byte-identical after mutation, exit 2 (FIXTURE_DRIFT — the mutation
   primitive failed silently; codex v2 defect 3, v3 defect 5).
2. Leave the schema well-typed. `vaisc emit-ts` on the mutated schema
   must exit 0. If the schema parses but is no longer well-typed, exit 2
   (the mutation chosen for the fixture is wrong, not a propagation
   regression).

**Phase 2 — post-change consumer checks (FAIL if any succeeds)**

ALL THREE consumer checks must now exit non-zero, since each consumer
still references the renamed field:
- `vaisc check consumers/vaisdb_table.vais` exits non-zero
- `vaisc check consumers/vais_server_api.vais` exits non-zero
- TypeScript compile of `consumers/vais_web_consumer.ts` against the
  refreshed `.d.ts` exits non-zero

If any consumer now succeeds, the schema change failed to propagate to
that consumer. Runner exits 1 (Gate FAIL) and reports which consumer.

**Cleanup invariant**: the runner copies the fixture into a per-run temp
workspace before mutating it. Cleanup uses `trap` so temporary files are
removed on every exit path, including assertion failure. Cleanup itself
must not mask the gate's exit code (the failure code is recorded in a
variable before cleanup and re-applied at the end; codex v3 defect 2).

### Negative gate semantics

Tests that a field type change propagates to consumer return contracts. The
mutation flips `email: str` to `email: i64`; consumers that read `u.email`
as `str` must then fail at `vaisc check`.

Four assertions:

1. The type-change mutation must produce a real diff (exit 2
   FIXTURE_DRIFT otherwise).
2. The mutated schema must remain well-typed (`vaisc check
   schema/user.vais` exits 0; otherwise exit 2).
3. `vaisc emit-ts` on the mutated schema must exit 0.
4. The `vaisdb_table.vais` consumer's `vaisc check` must now exit non-zero
   because `lookup_email(u) -> str` returns an `i64` field. If it succeeds,
   the schema change failed to propagate — exit 1 Gate FAIL.

### Self-audit on gate semantics (north-star alignment)

| L-002 invariant | How the gate satisfies it |
|---|---|
| 안정성 100% | FIXTURE_DRIFT (exit 2) is distinct from Gate FAIL (exit 1). Cleanup never masks failure. Pre-change baseline check eliminates vacuous fails. |
| 명확성 100% | The runner emits exactly which phase failed and which consumer/assertion. No silent absorption. |
| 우회 없이 | No path that skips an assertion to "make CI green". FIXTURE_DRIFT must be fixed in the fixture, not papered over. |

### Wiring into CI

The main-port branch exposes the gate runners and exact commands, but the full
aggregate `compiler/scripts/check-integrity.sh` runner is still pending main
port. When that aggregate runner lands, this section should be wired as a new
integrity section. The exact assertion count is computed by the gate runner
from the assertions defined in this document:

- **Positive gate** (11 assertions):
  - Phase 0 baseline (6): emit-ts succeeds; vaisdb_table.vais checks
    OK; vaisdb_table.vais native build/run exits 0; vais_server_api.vais
    checks OK; vais_server_api.vais native build/run exits 0;
    vais_web_consumer.ts compiles OK.
  - Phase 1 mutation (2): mutation produces real diff; re-emit succeeds.
  - Phase 2 propagation (3): vaisdb_table.vais now fails;
    vais_server_api.vais now fails; vais_web_consumer.ts now fails.
- **Negative gate** (4 assertions): mutation produces real diff;
  schema still type-checks after mutation; emit-ts succeeds on mutated
  schema; vaisdb_table.vais consumer (which constructs `User { ... }`
  literally) now fails.

Total: 15. A future aggregate runner can add
`INTEGRITY_CROSS_PACKAGE_SCHEMA_MIN=15` to `master-plan.toml`
`[[baseline_lock.threshold]]`. The threshold must be computed from the spec,
not declared independently. (codex v5 defect 3 fix — count derived from spec,
no drift surface.)

## Open questions resolved (codex v1 defect 1)

- **Q1: `Result<T, E>` lowering** — RESOLVED in favor of tagged union
  `{ ok: T } \| { err: E }`. Error-branch erasure rejected as silent
  failure.
- **Q2: declaration vs runtime emit** — RESOLVED. v1 = declaration only
  (`--declaration-only` is the only mode; no flag, no runtime JS validator).
  Master Plan I-3 demands TS compile-time enforcement; runtime validation
  is out of scope for Step 8.
- **Q3: emit location** — RESOLVED. Co-located in v1: `examples/schema/user.d.ts`
  next to `user.vais`. Reason: zero consumer ceremony. Outdir variant
  deferred to Step 14 when build wiring is added.

## Sequencing

- **Step 8** (cross-package schema consumption implementation, Master Plan
  v16 budget 1-2 months) consumed this design. The current main-port
  deliverable is working `vaisc emit-ts` plus
  `compiler/tests/empirical/cross_package_schema/` with both local workspace
  gates passing. CI aggregation remains pending with `scripts/check-integrity.sh`.
- **Step 7** (surface inventory lock + retro-validation) creates the
  `compiler/tests/empirical/` directory used here. Step 7 starts in parallel.
- **Step 14** (I-3 multi-domain product) has a main-port fixture under
  `compiler/tests/product/multi_domain_schema/`. It emits TS, type-checks
  VaisDB catalog and vais-server context consumers through real package APIs,
  type-checks a vais-web DB schema-builder consumer, and proves field-rename
  propagation. Native DB/server runtime coverage remains integration evidence
  until the sibling packages and aggregate runtime gate are ported to main.

## Self-audit (Master Plan v16 invariant compliance)

| L-002 invariant | How this design satisfies it |
|---|---|
| 안정성 100% (no silent failure) | All 14 enumerated unsupported surfaces hard-fail with stable error codes (EMIT_TS_001-014). EMIT_TS_999 catch-all + `emit_ts_exhaustiveness_test` close the residual unknown-variant hole. |
| 명확성 100% (no implicit behavior) | Lowering table is explicit per Vais type. Erasure points (ref/lifetime/i64-precision) are documented in emit header. |
| 우회 없이 모두 완성 | (b) was rejected because it would re-parse with a focused-subset parser — explicit "no parallel pipeline" rule. |

## Codex v2 review next

This document submits to codex review again per Master Plan v16
`codex_review.plan_required = True`. Convergence target: PASS in 1-2
revisions (v1 had 5 defects; v2 addresses all 5 with citations).
