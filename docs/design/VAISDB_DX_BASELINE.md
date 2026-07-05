# VaisDB Developer Workflow Baseline

This document fixes the current developer workflow for the Vais-authored
document/index/query slice. It is not a database product specification. It is
the reproducible baseline a contributor should use before changing the language
surface that Veriqel or VaisDB prototypes depend on.

## Verified Workflow

The workflow is intentionally small and gate-backed:

1. Split LF/CRLF document text with `str_split_lines_into`.
2. Persist metadata through `map_str_str_snapshot` and
   `map_str_str_load_snapshot`.
3. Express parse/error branches with the current concrete
   `Result<Int,Int>`/`Option<Int>` surface.
4. Pass `Map<Str,Str>` metadata through helper chains that return
   `Result<Int,Int>` and use local-binding `?`.
5. Ingest two documents, build `Map<Str,Int>` term counts, and score a query
   with `doc_term_weighted_score`.
6. Read document/query files with `fs_read_text`, create deterministic temp
   fixtures with `fs_temp_dir`, `path_join`, and `fs_write_text`, and accept
   external file paths with `proc_argc`/`proc_arg`.
7. Guard file-backed ingest with `fs_exists` and report missing or malformed
   inputs through the current concrete `Result<Int,Int>` error-code pattern.
8. Guard text reads with `Result<Str,Int>` so successful file content can flow
   through `?` while missing files remain integer error codes.
9. Pass a small structured `Metric` payload through `Result<Metric,Int>`
   helper returns, parameters, and inline matches.
10. Extend structured Result payloads beyond `Metric` with declared Int-field
    structs such as `Result<Record,Int>`.
11. Extend declared structured Result payloads to document-like records with
    `Str` fields, recovering string field lengths through inline matches.
12. Produce a small Vais-authored benchmark report with `time_millis`,
   `doc_term_counts_into`, `doc_term_weighted_score`, `fs_write_text`, and
   `fs_read_text`.
13. Produce a CLI-style Vais-authored benchmark report that discovers the repo
   root, runs the indexer through `proc_capture`, records direct/default
   elapsed milliseconds, and persists status metrics.
14. Run a reusable Vais-authored benchmark report tool that parses saved metric
    lines and writes a direct/default summary report.
15. Pass `Result<Str,Int>` and declared-struct Result-returning helpers
    directly as helper-call arguments without temporary locals.
16. Copy declared-struct Result wrapper payloads through local struct variables
    and return those locals without losing nested fields.
17. Recover `Str` fields from declared-struct Result matches for report-style
    title/ID selection while converting `Err(Int)` codes with `Str(code)`.
18. Run a manifest-backed package directory directly through `scripts/vaisc`
    while preserving imports and forwarding CLI argv to the compiled program.

The canonical examples are:

- `examples/e292_str_split_lines_into.vais`
- `examples/e293_map_str_str_snapshot_builtin.vais`
- `examples/e294_result_try_parse_error_flow.vais`
- `examples/e296_result_map_param_flow.vais`
- `examples/e295_vaisdb_indexer_prototype.vais`
- `examples/e297_vaisdb_file_ingest_workflow.vais`
- `examples/e298_vaisdb_file_ingest_result_flow.vais`
- `examples/e301_result_str_int_file_read.vais`
- `examples/e302_result_str_int_param_flow.vais`
- `examples/e303_result_metric_int_struct_payload.vais`
- `examples/e304_result_record_int_struct_payload.vais`
- `examples/e305_result_multiline_struct_payload.vais`
- `examples/e306_result_struct_str_fields.vais`
- `examples/e307_result_struct_try_payload.vais`
- `examples/e308_vaisdb_artifact_record_workflow.vais`
- `examples/e309_vaisdb_artifact_store_snapshot.vais`
- `examples/e310_vaisdb_artifact_query_report.vais`
- `examples/e311_result_call_argument_flow.vais`
- `examples/e312_result_struct_local_wrapper_flow.vais`
- `examples/e313_result_struct_str_match_flow.vais`
- `examples/e314_result_struct_str_concat_match_flow.vais`
- `examples/e315_result_struct_str_transform_match_flow.vais`
- `examples/e316_result_struct_str_transform_len_match_flow.vais`
- `examples/e317_result_struct_payload_helper_call_score.vais`
- `examples/e318_result_struct_payload_helper_call_arithmetic.vais`
- `examples/e319_result_struct_payload_field_helper_call_arithmetic.vais`
- `examples/e320_result_struct_payload_int_field_helper_call_arithmetic.vais`
- `examples/e321_result_struct_payload_bool_match_condition.vais`
- `examples/e322_vaisdb_module_boundary/main.vais`
- `examples/e323_cli_package`
- `examples/e299_vaisdb_benchmark_report.vais`
- `examples/e300_vaisdb_benchmark_cli_report.vais`
- `tools/vaisdb_benchmark_report.vais`

Run the focused workflow gate with:

```bash
bash scripts/test-vaisdb-workflow.sh
```

That gate runs each example through both `--engine direct` and the default
self-host-backed compiler path and expects the product smoke value `42`. The
e297 workflow is checked twice: once with generated temp files and once with
argv-supplied document/query paths. The e298 workflow is checked in the same
generated-file and argv-file modes, plus a missing-document path that must
return `Err(10)` as process exit code `10` instead of trapping in
`fs_read_text`. The e301 workflow checks the string-payload Result follow-up:
guarded helpers return `Result<Str,Int>`, propagate the read text with `?`, and
recover both normalized strings and missing-file error codes through inline
match. The e302 workflow checks the next string-payload Result step: those
`Result<Str,Int>` values can flow through helper parameters, be forwarded to
other helpers, and recover payload/error values through inline matches. The
e303 workflow checks the first structured Result payload step:
`Result<Metric,Int>` values can flow through helper parameters and recover
struct fields through inline matches. The e304 workflow checks the generalized
declared Int-field struct payload step with `Result<Record,Int>` and
three-field inline recovery. The e305 workflow checks the same
declared-struct Result path with a multiline Int-field payload declaration and
four-field inline recovery. The e306 workflow checks declared-struct Result
payloads with `Str` fields by recovering title/summary lengths inside the Ok
arm. The e307 workflow checks local-binding `?` for the same declared-struct
Result payload path, including field reuse and early integer error
propagation. The e308 workflow checks the first artifact-record composition:
`Result<DocArtifact,Int>` payloads are extracted with `?`, pushed into
caller-visible `List<DocArtifact>` output parameters, snapshotted through
`Map<Str,Str>` metadata, and still propagate integer errors.
The e309 workflow checks the persistable artifact-store follow-up:
`List<DocArtifact>` records are serialized to a text snapshot, persisted with
file helpers, reloaded through `Result<DocArtifact,Int>` parsing helpers, and
queried after reload while malformed/missing stores remain integer Result
errors. The e310 workflow checks the reusable query/report follow-up:
persisted artifact records are loaded into `List<DocArtifact>`, ranked through
`Map<Str,Int>` term scoring, returned as a `Result<Str,Int>` report payload,
persisted again as text, and checked for missing-store and empty-query error
codes. The e311 workflow checks the call-site ergonomics follow-up:
`Result<Str,Int>` and declared-struct Result-returning helpers can be passed
directly as helper arguments without temporary locals. The e312 workflow checks
the self-host structured-payload follow-up: explicit
`VaisResult<Struct>Int` wrapper payloads can be copied through local struct
variables and returned without losing nested fields. The e313 workflow checks
the report-building follow-up: declared-struct Result matches can recover
string payload fields such as titles and IDs while error arms stringify integer
codes. The e314 workflow checks the report-label follow-up: declared-struct
Result matches can compose payload string fields with nested `str_concat(...)`
while error arms stringify integer codes. The e315 workflow checks the
normalization follow-up: declared-struct Result matches can transform payload
string fields with `str_replace`, `str_trim`, `str_upper`, `str_lower`, and
local-prefix `str_concat(...)` while error arms stringify integer codes. The
e316 workflow checks the scoring follow-up: declared-struct Result matches can
turn transformed string payload fields into integer `.len()` score terms while
mixing normal integer payload fields. The e317 workflow checks the reusable
helper follow-up: declared-struct Result matches can pass `Ok` payload structs
to scoring helpers while preserving integer error codes. The e318 workflow
checks helper composition: those reusable helper-call results can be mixed with
normal payload fields in the same integer match arm. The e319 workflow checks
field-helper composition: reusable integer helpers can
receive `Ok` payload string fields such as `artifact.title` and
`artifact.body`, then compose those helper-call terms with normal payload
fields in the same match arm. The e320 workflow checks numeric field-helper
composition: reusable integer helpers can receive `Ok` payload integer fields
such as `artifact.terms` and `artifact.score`, then compose those terms with
string-field helper terms in the same match arm. The e321 workflow checks
Bool-return composition: validation helpers can return conditions directly from
structured Result payload helper terms and error-code comparisons. The e322
workflow checks the first reusable module boundary: imported VaisDB scoring and
artifact modules share `DocArtifact`, `Result<DocArtifact,Int>`,
`List<DocArtifact>`, and `Map<Str,Int>` helper surfaces in both direct and
default runs. The e323 workflow checks the first package-directory CLI surface:
`scripts/vaisc run examples/e323_cli_package` resolves the manifest source
entry, preserves imports, and forwards argv in both direct and default runs.
The e299 workflow is checked as a deterministic in-language
report fixture that records elapsed milliseconds and validates the saved
metrics. The e300 workflow is checked as the CLI-style follow-up that invokes
the e295 indexer from Vais code with `proc_capture`, compares direct/default
status, and validates the persisted report. The reusable benchmark report tool
is checked in direct/default modes and through `scripts/vaisdb-benchmark-report.sh`;
it parses the raw report and writes a summary.

## Diagnostics

Use the normal public checker and compiler gates before widening this workflow:

```bash
scripts/vais-check examples/e295_vaisdb_indexer_prototype.vais
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisdb-workflow.sh
```

The dedicated workflow gate is deliberately narrower than the full release
gate. It answers one question: can a contributor still reproduce the current
document ingest, snapshot, parse/error, Result helper, and query path without
knowing the full compiler test matrix?

## Formatting Direction

There is no public `vais fmt` contract yet. Until a formatter is promoted,
contributors should keep new examples in the style used by the value corpus:
four-space indentation inside blocks, one top-level declaration per section,
and explicit helper names for product workflow steps.

Whitespace regressions are still checked by:

```bash
git diff --check
```

A future formatter should be Vais-authored once the parser APIs are stable
enough to avoid regex-only source rewriting.

## Performance Baseline

Use the benchmark harness for local compare-before/after measurements:

```bash
bash scripts/bench-vaisdb-indexer.sh
bash scripts/bench-vaisdb-indexer.sh 20
VAISDB_BENCH_ITERATIONS=20 bash scripts/bench-vaisdb-indexer.sh
```

The script measures compile+run time for `examples/e295_vaisdb_indexer_prototype.vais`
through both the direct and default engines. It validates the expected exit
value on every iteration, but it does not enforce a fixed time threshold because
developer machines and debug builds vary.
`examples/e299_vaisdb_benchmark_report.vais` complements that shell benchmark
by proving that Vais code itself can create a timing report; it is still a
functional fixture, not a machine-independent performance threshold.
`examples/e300_vaisdb_benchmark_cli_report.vais` goes one step further by
driving the e295 indexer through `proc_capture` and recording direct/default
status metrics from Vais code.
`tools/vaisdb_benchmark_report.vais` is the reusable developer-tool form: it
generates the raw report, parses metric lines, computes the direct/default
delta, and writes a summary report.

When recording a performance note, include:

- commit or worktree description,
- machine/OS,
- iteration count,
- direct engine timing,
- default engine timing,
- whether `bash scripts/test-vaisdb-workflow.sh` passed.

## Release Gate

The focused workflow gate is part of `bash scripts/test-release-gates.sh`.
The shell benchmark is intentionally not part of the release gate because it is
a measurement tool, not a deterministic pass/fail contract. The e299/e300
Vais-authored report fixtures and the reusable benchmark report tool are
functional checks and are covered by the focused workflow gate.
