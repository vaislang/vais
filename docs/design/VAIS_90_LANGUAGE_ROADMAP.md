# Vais 90% Language Roadmap

## Goal

Bring Vais from the current gate-backed compiler/language surface to a practical
product-development language for Veriqel and VaisDB prototypes. The target is
not parity with Python or Rust. The near-term 90% target means a developer can
write document ingestion, indexing, search, CLI tooling, snapshots, and small
services mostly in Vais without falling back to C, shell, or another language
for routine text/data work.

## Non-goals

- Do not claim Python/Rust replacement status in this phase.
- Do not start a separate database product before the language can express the
  core document-indexing workflow in Vais.
- Do not add broad unverified syntax. Every promoted slice must have direct,
  full self-host, parity, and release-gate evidence.
- Do not rewrite the compiler architecture while incremental surface expansion
  remains faster and lower risk.

## Current State

Vais now has a self-host compiler core, native direct engine, front contracts,
parity manifest, release gates, and a growing verified standard surface. The
current language can already express small document-processing programs using
`Str`, `List<Str>`, `Map<Str,Int>`, `Map<Str,Str>`, structs, basic host IO,
process capture, string normalization, split/join, list pipelines, and
term-frequency scoring.

The biggest remaining gap is not another isolated string helper. The gap is a
continuous product workflow: read files, split structured text, parse metadata,
 build records, index terms, persist snapshots, query them, and report results
from Vais code with predictable errors. The first file-backed version of that
workflow is now gate-backed by `examples/e297_vaisdb_file_ingest_workflow.vais`;
`examples/e298_vaisdb_file_ingest_result_flow.vais` adds the first
missing-file/malformed-input `Result<Int,Int>` control-flow recipe for that
file path. `examples/e299_vaisdb_benchmark_report.vais` adds the first
Vais-authored timing/report fixture for that workflow using `time_millis`,
document term counts, weighted scoring, and persisted report validation.
`examples/e300_vaisdb_benchmark_cli_report.vais` adds the first CLI-style
report fixture that discovers the repo root, runs the Vais-authored indexer
through `proc_capture`, records direct/default timing/status metrics, and
persists the result from Vais code.
`tools/vaisdb_benchmark_report.vais` is the first reusable tool form of that
workflow: it generates a raw report, parses metric lines, computes a
direct/default delta, and writes a summary from Vais code.
`examples/e301_result_str_int_file_read.vais` closes the first string-payload
Result gap for file reads: guarded helpers can now return `Result<Str,Int>`,
propagate the text payload with `?`, and recover missing-file codes through
inline match.
`examples/e302_result_str_int_param_flow.vais` closes the next string-payload
Result gap: `Result<Str,Int>` locals can now be passed through helper
parameters, forwarded into other helpers, and matched inside those helpers.
`examples/e303_result_metric_int_struct_payload.vais` opens the first
structured Result payload slice: `Result<Metric,Int>` helpers can return,
accept, and forward a `Metric` payload, then recover fields through inline
matches.
`examples/e304_result_record_int_struct_payload.vais` extends that path beyond
`Metric`: declared Int-field structs such as `Record` can flow through
`Result<DeclaredStruct,Int>` helpers and recover multiple fields through
inline matches.
`examples/e305_result_multiline_struct_payload.vais` removes the one-line
declaration limitation from that path: multiline Int-field structs such as
`Entry` can flow through `Result<DeclaredStruct,Int>` helpers and recover four
fields through inline matches.
`examples/e306_result_struct_str_fields.vais` extends the same structured
Result path to document-like payloads with `Str` fields, so `DocSummary` can
flow through `Result<DeclaredStruct,Int>` helpers and recover string field
lengths mixed with Int fields.
`examples/e307_result_struct_try_payload.vais` adds the matching local-binding
`?` step: `DocSummary` can be extracted from `Result<DocSummary,Int>`, reused
through `Str`/`Int` fields, and still propagate integer errors early.
`examples/e308_vaisdb_artifact_record_workflow.vais` turns that surface into a
small product-shaped workflow: `DocArtifact` records are built through
`Result<DocArtifact,Int>`, extracted with `?`, stored through
`List<DocArtifact>` output parameters, and paired with `Map<Str,Str>` metadata
snapshots.
`examples/e309_vaisdb_artifact_store_snapshot.vais` adds the persistable store
step: `List<DocArtifact>` records are serialized to a text snapshot, written
and read through host file helpers, parsed back through
`Result<DocArtifact,Int>` helpers, and queried after reload.
`examples/e310_vaisdb_artifact_query_report.vais` adds the reusable
query/report step on top of that store: records are loaded into
`List<DocArtifact>`, ranked with `Map<Str,Int>` term scoring, rendered as a
`Result<Str,Int>` report payload, persisted with file helpers, and validated
against missing-store and empty-query error codes.
`examples/e311_result_call_argument_flow.vais` removes the next Result
call-site friction: `Result<Str,Int>` and `Result<DocArtifact,Int>` returning
helpers can be passed directly as helper arguments without temporary locals.
`examples/e312_result_struct_local_wrapper_flow.vais` removes the next
self-host structured-payload friction: explicit `VaisResult<Struct>Int`
wrappers can bind `flow.value` to a local struct, read all payload fields, and
return that local in another wrapper without losing nested fields.
`examples/e313_result_struct_str_match_flow.vais` removes the next report
composition friction: `Result<DeclaredStruct,Int>` matches can recover `Str`
fields such as `artifact.title` into string locals while `Err(Int)` arms
convert codes with `Str(code)`.
`examples/e314_result_struct_str_concat_match_flow.vais` removes the follow-on
report-label friction: `Result<DeclaredStruct,Int>` matches can compose `Str`
payload fields with nested `str_concat(...)` calls inside the `Ok` arm while
preserving stringified `Err(Int)` recovery.
`examples/e315_result_struct_str_transform_match_flow.vais` removes the next
normalization friction: `Result<DeclaredStruct,Int>` matches can transform `Str`
payload fields with `str_replace`, `str_trim`, `str_upper`, `str_lower`, and
local-prefix `str_concat(...)` expressions inside the `Ok` arm while preserving
stringified `Err(Int)` recovery.
`examples/e316_result_struct_str_transform_len_match_flow.vais` removes the
follow-on scoring friction: `Result<DeclaredStruct,Int>` matches can turn
transformed `Str` payload fields into `Int` score terms with chained `.len()`
calls while combining them with ordinary integer payload fields.
`examples/e317_result_struct_payload_helper_call_score.vais` removes the
reusable-helper friction: `Result<DeclaredStruct,Int>` matches can pass the
`Ok` payload struct directly to an `Int` scoring helper such as
`score_artifact(artifact)` instead of forcing all scoring logic into the match
arm.
`examples/e318_result_struct_payload_helper_call_arithmetic.vais` removes the
helper-composition friction: a reusable `Ok` payload helper-call result can now
be one `Int` term in a structured Result match arm, combined with normal
payload fields such as `artifact.terms + artifact.score`.
`examples/e319_result_struct_payload_field_helper_call_arithmetic.vais` removes
the field-helper composition friction: reusable `Int` helpers can receive
`Ok` payload `Str` fields such as `artifact.title` and `artifact.body` inside
structured Result match arms, then combine those terms with normal payload
fields.
`examples/e320_result_struct_payload_int_field_helper_call_arithmetic.vais`
removes the numeric field-helper composition friction: reusable `Int` helpers
can receive `Ok` payload integer fields such as `artifact.terms` and
`artifact.score` inside structured Result match arms, then combine those terms
with string-field helper terms.
`examples/e321_result_struct_payload_bool_match_condition.vais` removes the
Bool-return friction: reusable validation/filter helpers can now return
conditions directly from structured Result payload helper terms, while
`Err(Int)` arms compare error codes.
`examples/e322_vaisdb_module_boundary/main.vais` removes the first reusable
library-boundary friction: a VaisDB-style workflow can split scoring and
artifact helpers across imported modules while sharing `DocArtifact`,
`Result<DocArtifact,Int>`, `List<DocArtifact>`, and `Map<Str,Int>` surfaces in
direct/default runs.
`examples/e323_cli_package` removes the first package-directory CLI friction:
`scripts/vaisc emit-ir`, `build`, and `run` can take a manifest-backed package
directory directly, resolve `source/main.vais`, preserve imports, and forward
argv to the compiled program. e324 removes the next packaging friction:
`scripts/vaisc package <package-dir> -o <dist-dir>` builds an installable
`dist/bin/<package-name>` binary and copies `vais.toml` for direct/default
package workflows. e325 hardens that output as a real CLI artifact: native,
direct, and workflow gates run packaged binaries with forwarded argv, and
unsafe manifest names are rejected before they can become output paths.
e326 adds the first package target metadata: optional `binary = "cmd-name"`
lets a package keep its package identity separate from the installed command
name. e327 adds user-package release archive integration: `--archive` creates
an extractable `<binary-or-name>-<version>.tar.gz` payload and gates it through
native/direct/workflow checks. e328 adds optional package assets:
`assets = "assets"` copies one regular-file/directory tree into `dist/assets`
and the archive payload as `assets/`. Remaining work should push from package
examples toward richer reusable package layout, additional package diagnostics,
structured payload diagnostics, and reporting tools.

## Proposed Changes

### Phase 1: Document Text Ergonomics

Add the small builtins that make file and document parsing pleasant:
`str_split_lines_into`, robust CRLF handling, blank-line behavior, key/value
line parsing patterns, and fixtures that combine host file IO with line parsing.

### Phase 2: Structured Data

Promote practical structured-text support before full JSON complexity:
first `map_str_str_snapshot(docs: Map<Str,Str>) -> Str` and
`map_str_str_load_snapshot(text: Str, out: Map<Str,Str>) -> Int` for
line-based `key=value\n` document metadata snapshots. Loading clears the output
map, accepts LF/CRLF lines, skips blank/malformed/no-key lines, preserves
additional `=` bytes in values, allows empty values, and returns the number of
loaded entries. Keep later CSV/TSV or escaped-field work narrow and
gate-backed; do not treat this phase as a full JSON parser.

### Phase 3: Error and Result Ergonomics

Improve enough `Result`/`Option` usage to let real programs report parse/file
failures cleanly. Preserve the current verified concrete generic constraints
until broader generic support is justified by a product workflow.

Current slice: keep `Option<Int>` and `Result<Int,Int>` concrete, but promote
native direct value lowering for helper return/parameter/local types,
constructors, inline expression-match binding, and local-binding `?`
propagation. `examples/e294_result_try_parse_error_flow.vais` fixes the product
pattern for parsing document metadata with explicit integer error codes before
the Vais-authored indexer prototype starts.
`examples/e296_result_map_param_flow.vais` extends the same concrete Result
slice to helpers that take `Map<Str,Str>` metadata parameters, match
`Map<Str,Str>.get_opt`, and propagate errors with local-binding `?`.
`examples/e298_vaisdb_file_ingest_result_flow.vais` extends the concrete
Result pattern to file-backed ingest: `fs_exists` guards raw `fs_read_text`
calls, missing files return `Err(10)`, malformed text returns additional
integer error codes, and callers compose those helpers with local-binding `?`.
`examples/e301_result_str_int_file_read.vais` extends the concrete Result
pattern from integer-only success payloads to guarded text reads:
`Result<Str,Int>` helpers can return `Ok(text)`/`Err(code)`, compose with
local-binding `?`, and recover the string payload or error code through inline
match. `examples/e302_result_str_int_param_flow.vais` then verifies
`Result<Str,Int>` helper parameters and forwarding, including inline matches
that combine an Err binder with an existing Int parameter. This does not open
generic `Result<T,E>` yet.

### Phase 4: VaisDB Prototype in Vais

Write a small Vais-authored document indexer using the verified language
surface. The prototype should ingest text documents, normalize terms, store
term counts and metadata snapshots, and answer simple ranked queries.
`examples/e295_vaisdb_indexer_prototype.vais` is the first gate-backed
dogfooding slice: it combines metadata ingest, line snapshot round trip,
term-frequency maps, and weighted query scoring in one Vais program. It keeps
Map-mutating helpers returning `Int` for prototype stability, while
`examples/e296_result_map_param_flow.vais` separately verifies direct
`Result<Int,Int>` helper flows over `Map<Str,Str>` parameters.
`examples/e297_vaisdb_file_ingest_workflow.vais` extends the prototype to
host file IO and CLI-style inputs: it reads document/query files, can generate
deterministic temp fixtures, accepts argv paths, snapshots metadata, indexes
terms, and scores a query in a single Vais workflow.
`examples/e298_vaisdb_file_ingest_result_flow.vais` keeps that workflow shape
but makes failure explicit enough for the next VaisDB/Veriqel prototype slice:
file existence checks and parsing failures are normal `Result<Int,Int>` values
instead of hidden fallback integers or raw host traps.
`examples/e299_vaisdb_benchmark_report.vais` adds a report-oriented dogfooding
slice on top of the same primitives: it records elapsed milliseconds around
term counting/scoring, writes a report file, reads it back, and validates the
metrics in direct, full, workflow, and parity gates.
`examples/e300_vaisdb_benchmark_cli_report.vais` turns that into a
CLI-oriented dogfooding slice: Vais code uses path helpers to locate the repo,
runs `scripts/vaisc run examples/e295_vaisdb_indexer_prototype.vais` through
`proc_capture`, records direct/default elapsed milliseconds, and validates the
persisted report in direct, full, workflow, and parity gates.
`tools/vaisdb_benchmark_report.vais` promotes the same flow into a reusable
Vais-authored developer command. It is gate-backed through the focused
workflow, direct/default execution, native parity/value corpus, and full
codegen summary parsing fixture.
`examples/e301_result_str_int_file_read.vais` is the first targeted feedback
slice from that workflow: real file ingest wants the successful text, not just
an integer status, while retaining deterministic integer error codes.

### Phase 5: Developer Experience

Add the minimum tooling needed for repeated product development: formatting or
formatter checks, clearer diagnostics for the new APIs, examples organized as
recipes, and performance baselines for indexing/query slices.
`docs/design/VAISDB_DX_BASELINE.md` now fixes the current document/VaisDB
developer workflow, `scripts/test-vaisdb-workflow.sh` provides a focused
direct/default reproducibility gate for e292-e324, and
`scripts/bench-vaisdb-indexer.sh` records the local compile+run timing protocol
for the first Vais-authored indexer prototype. `time_millis() -> Int` is now
verified for in-language benchmark/report fixtures, while fixed performance
thresholds remain out of scope for release gates. The e300 CLI report fixture
is now the first gate-backed step toward reusable Vais-authored developer
tools; `tools/vaisdb_benchmark_report.vais` is the first such reusable
developer command.

## Interfaces and Contracts

All new public APIs must be documented in `std/PRELUDE.md` and
`docs/reference/LANGUAGE.md`, covered by `examples/eNNN_*.vais`, added to
`tools/vaisc-parity.tsv`, and exercised in:

- `tools/vaisc_front_check.vais`
- `tools/vaisc_direct_feature_check.vais`
- `tools/fixpoint_full_codegen_check.vais`
- `compiler/self/fixpoint_full.vais`
- `tools/vaisc_native.c`

For compiler-core changes, regenerate `compiler/self/vaisc_core.ll` through the
direct bootstrap and the canonical full self-host path before running release
gates.

## Files to Touch

- `compiler/self/fixpoint_full.vais`
- `compiler/self/vaisc_core.ll`
- `tools/vaisc_native.c`
- `tools/vaisc_front_check.vais`
- `tools/vaisc_direct_feature_check.vais`
- `tools/fixpoint_full_codegen_check.vais`
- `tools/vaisc-parity.tsv`
- `examples/eNNN_*.vais`
- `std/PRELUDE.md`
- `docs/reference/LANGUAGE.md`
- `examples/README.md`
- `CHANGELOG.md`
- `WORKLOG.md`
- `ROADMAP.md`

## Risks and Edge Cases

- `fixpoint_full.vais` is large; every builtin addition can expose stage1/stage2
  bootstrap drift or temporary-register mistakes.
- Native direct and full self-host runtimes must agree on list ABI and string
  allocation behavior.
- Text helpers must define empty input, trailing delimiter, CRLF, and capacity
  overflow behavior explicitly.
- Broader JSON/generic support can balloon scope; keep product-driven slices.

## Validation Plan

For each language slice:

1. Add a value example under `examples/`.
2. Verify direct and full execution both return the expected value.
3. Verify emitted IR shape for the new runtime helper.
4. Update front, direct, full codegen, and parity fixtures.
5. Regenerate `compiler/self/vaisc_core.ll`.
6. Run focused gates.
7. Run `git diff --check`.
8. Run `bash scripts/test-release-gates.sh` before closing the slice.

## Assumptions

- The immediate product target is Veriqel/VaisDB document ingestion and search,
  not web servers or general systems programming.
- The fastest route to 90% is dogfooding: implement small VaisDB components in
  Vais and fill only the missing language/API pieces exposed by that work.
- Current concrete collection support is acceptable while product prototypes
  remain document/text-heavy.

## Open Questions

- Whether the first VaisDB snapshot format should be TSV, line-based key/value,
  or minimal JSON. Current answer: start with line-based `Map<Str,Str>`
  `key=value\n` snapshots; revisit TSV/JSON only when the VaisDB prototype
  exposes a concrete need.
- How much Unicode support is required before a public Veriqel pilot.
- Whether the formatter should be a native tool first or a Vais-authored tool
  once enough parsing APIs exist.
