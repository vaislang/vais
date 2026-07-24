# Vais Prelude Reference

This file is the v1-candidate prelude reference for Vais. It records
prelude-like APIs and whether they are currently verified by a gate.

- "Verified" means protected by a value, compiler, parity, or release gate.
- "Partial" means only the described concrete slice is a public claim.
- "Specified" means intended or designed, but not a public release claim yet.

The verified entries below are frozen for the current v1 candidate: changing a
signature, behavior, or status requires an updated gate and synchronized
documentation in `docs/reference/LANGUAGE.md`.

## Output

| API | Status |
| --- | --- |
| `print(EXPR)` | Verified |
| `putchar(Int)` | Verified |

## Control Flow

| API | Status |
| --- | --- |
| `if COND then A else B` | Verified for scalar `Int`, `Bool`, `Str`, and `Char` value expressions in local assignments, reassignments, helper-call arguments, and returns |

## Host Files, Paths, And Processes

The Phase 3 host API is documented in
[../docs/design/HOST_IO.md](../docs/design/HOST_IO.md). `fs_exists`,
`fs_read_text`, `fs_write_text`, and `fs_mkdirs` are verified for the full
engine, the file/argv/path/process subset used by e297 through e301 is
verified in the direct engine, the first path and string helpers are verified,
and `proc_argc() ->
Int`, `proc_arg(index: Int) -> Str`, `proc_run(argv: List<Str>) -> Int`, and
`proc_run_env(argv: List<Str>, env: List<Str>) -> Int`,
`proc_capture_stdout(argv: List<Str>) -> Str`,
`proc_capture_stderr(argv: List<Str>) -> Str`, and
`proc_capture_to(argv: List<Str>, stdout_path: Str, stderr_path: Str) -> Int`,
plus `proc_capture(argv: List<Str>) -> ProcessResult` for the standard
`ProcessResult { code: Int, stdout: Str, stderr: Str }` shape,
are verified through the same host gate. `proc_argc` and `proc_arg` are
verified for `vaisc run -- ...` and for binaries produced by `vaisc build`.
`examples/e137_map_str_str_snapshot.vais` also protects the combined
`Map<Str,Str>` entry-snapshot and host file IO pattern, while
`examples/e138_map_str_str_snapshot_load.vais` protects parsing that snapshot
back into a `Map<Str,Str>`. `examples/e293_map_str_str_snapshot_builtin.vais`
promotes that storage pattern to builtins for in-memory line snapshots.
`examples/e139_map_return_infer.vais` protects unannotated Map-return local
initialization and `.len()` chains on Str-valued Map reads.
`examples/e297_vaisdb_file_ingest_workflow.vais` protects the combined
file-backed workflow over `fs_read_text`, `fs_write_text`, `fs_temp_dir`,
`path_join`, `proc_argc`, and `proc_arg` in both generated-temp-file and
argv-file modes. `examples/e298_vaisdb_file_ingest_result_flow.vais` protects
the follow-on `fs_exists` guarded file-read recipe that returns
`Result<Int,Int>` error codes for missing or malformed inputs.
`examples/e301_result_str_int_file_read.vais` protects the next concrete
Result file-read recipe: helpers return `Result<Str,Int>`, local-binding `?`
binds the `Str` payload, and inline matches recover either string payloads or
integer error codes. `examples/e302_result_str_int_param_flow.vais` extends
that concrete slice so `Result<Str,Int>` values can be passed through helper
parameters, forwarded to other helpers, and matched inside those helpers.
`examples/e329_result_str_str_error_message.vais` opens the first non-`Int`
error payload slice: `Result<Str,Str>` returns `Ok(Str)` or `Err(Str)` so a
failure carries a human-readable message, with local-binding `?` propagation and
inline match recovery of the `Str` value or `Str` message. It is verified in the
native direct engine and the full self-host compiler.
`examples/e330_vaisdb_ingest_error_message_flow.vais` dogfoods that surface in a
VaisDB file-ingest workflow: document ingest, metadata snapshot round trips, and
query scoring each report failures as descriptive `Result<Str,Str>` messages
instead of opaque integer codes.
`examples/e332_vaisdb_topk_ranking_report.vais` orders scored documents with a
hand-written `List<Struct>` selection sort (whole-element swaps through a
temporary struct local) and renders top-k report lines; a built-in list sort
remains an open ergonomics gap.
`examples/e333_vaisdb_snapshot_version_migration.vais` adds a `version=N`
header to metadata snapshots, migrates the v1 key layout on load, and reports
missing or unknown versions as `Result<Str,Str>` messages.
`examples/e334_vaisdb_index_persistence_incremental.vais` persists per-document
term counts under `docid.term` keys, reloads, extends incrementally without a
rebuild, and scores identically to a fresh build.
`examples/e335_list_int_sort.vais` promotes the dogfood-2 gap feedback into a
built-in: `List<Int>.sort()` sorts in place (ascending) as a statement on local
and parameter receivers; promoting it also root-fixed the full-engine
pointer-aliased list element write used by parameter receivers.
`examples/e336_list_struct_sort_by.vais` completes that gap feedback:
`List<Struct>.sort_by(|x| x.int_field)` and `sort_by_desc` order records in
place by an Int key, replacing the hand-written ranking sort in product flows.
`tools/vaisdb_cli.vais` promotes those recipes into a reusable Vais-authored
command with `ingest`, `query`, and `report` subcommands over the persisted
index; `scripts/vaisdb-cli.sh` is the shell entrypoint, and the VaisDB workflow
gate covers both engines, the wrapper, and every error exit code.
`examples/e338_fs_list_files.vais` promotes directory enumeration:
`fs_list_files(dir, out)` fills a `List<Str>` with the sorted regular-file
names so Vais tools can ingest whole directories; the vaisdb package's
`ingest-dir` and ranked `rank` subcommands build on it.
`examples/e343_self_recursion_at.vais` promotes `@(args)` self-recursion for
every expression position (the driver lowers it to the enclosing function's
name, so both engines share the named-recursion path).
Bare `xs.remove_at(i)` / `xs.pop()` statements (result discarded) are
verified on both engines for Int and struct lists — the driver desugars them
into the assigned form (`examples/e347_list_discard_statements.vais`).
`List<List<Int>>` literal double-index reads now compose in any expression
position on both engines (`examples/e348_nested_list_expr_reads.vais`), and
value if-expressions use the `then`/`else` form only — brace blocks in value
position are rejected at the front.
`examples/e350_vaisbench_package` is the fifth installable tool: `vaisbench`
times a child command over repeated `proc_run` runs with `time_millis` (the
first product use of the clock) and reports min/median/avg/max over a sorted
sample — variable trailing arguments pass straight through to the child, so
it benchmarks the repo's own gate scripts, and the `-b <budget-ms>` mode
turns it into the ladder's `perf` regression watch (median over budget
exits 3).
`examples/e346_vaisfmt_package` is the fourth installable tool: `vaisfmt`
normalizes Vais source whitespace (trailing spaces/tabs stripped, exactly one
trailing newline) with `-c` check and in-place fix modes over recursive
`.vais` tree walks, rebuilding file text through the `str_builder_*` chain —
now verified on both engines. Path `-` turns it into a pipe filter: `-c -`
exits 1 on dirty stdin, and the plain form writes the normalized text to
stdout via `stdout_write` (no added newline), so
`vaisgrep ... - | vaisfmt - | vaisgrep ... -` chains compose. vaisdb joins
the pipeline with `ingest-stdin <index> <doc-id>` (so search hits pipe
straight into the index), and the filter tools route their error messages
through `stderr_write`, keeping stdout byte-pure for downstream consumers. Its line scan walks byte offsets instead of
materializing a `List<Str>`, because the fixed list contract holds at most
4095 entries: `str_split_lines_into` (and the other `*_into` fillers) abort
past that with a `vais list trap: capacity exceeded` diagnostic on stderr
(list bounds and empty-access traps are diagnosed the same way on both
engines), so whole-repo tools stream instead
(`scripts/vaisfmt-check.sh` gates every tracked `.vais` tree, the ~23k-line
self-host source included).
`examples/e344_vaismake_package` is the third installable tool: `vaismake`
runs named tasks from a plain `name = command args...` file through
`proc_run`/`proc_capture` (whitespace-split argv, no shell) — the first
product use of the process surface — and `!env NAME=VALUE` lines overlay the
child environment through `proc_run_env` in run mode
(`examples/e345_proc_run_env.vais` covers the built-in on both engines), while
`!needs task dep...` lines run dependencies first, once each, stopping on the
first failing child and refusing dependency cycles. `tools/gates.tasks` runs
this repository's own gate ladder through the tool
(`scripts/vaismake-ladder.sh`).
`examples/e341_vaisgrep_package` is the second installable tool: `vaisgrep`
searches files, directories, or standard input (path `-`, via the new
`stdin_read_all()` host read — so it composes in shell pipelines) for
substring-matching lines (with `-c` counts and `-r` recursive tree walks over
`fs_list_dirs`), dispatching on the `fs_is_dir(path)` host test before
reading.
`examples/e337_vaisdb_cli_package` productizes that command as an installable
package: the index/report logic is split across `vaisdb.index` and
`vaisdb.report` modules, `scripts/vaisc package` builds `dist/bin/vaisdb` and a
`vaisdb-0.1.0.tar.gz` release archive, and the workflow gate runs the packaged
binary's subcommands and self-test from both the dist tree and the extracted
archive.
`examples/e303_result_metric_int_struct_payload.vais` opens the first
structured payload Result slice: a `Metric` struct can be returned as
`Result<Metric,Int>`, passed through helpers, and matched to recover payload
fields or integer error codes.
`examples/e304_result_record_int_struct_payload.vais` extends that path beyond
`Metric`: declared Int-field struct payloads such as `Record` can flow through
`Result<DeclaredStruct,Int>` helpers and recover multiple fields through inline
matches.
`examples/e305_result_multiline_struct_payload.vais` verifies the same
declared-struct payload flow when the Int-field payload struct is written over
multiple lines.
`examples/e306_result_struct_str_fields.vais` extends that declared-struct
Result payload path to document-like records with `Str` fields, recovering
string field lengths together with Int fields through inline matches.
`examples/e307_result_struct_try_payload.vais` adds local-binding `?` for the
same declared-struct Result payload path, so a `DocSummary` can be extracted
from `Result<DocSummary,Int>`, reused through fields, and still propagate
integer errors early.
`examples/e308_vaisdb_artifact_record_workflow.vais` applies that surface to a
small VaisDB artifact record workflow: `DocArtifact` values are produced as
`Result<DocArtifact,Int>`, extracted with `?`, stored through
`List<DocArtifact>` output parameters, and paired with `Map<Str,Str>` metadata
snapshots.
`examples/e309_vaisdb_artifact_store_snapshot.vais` persists those records as
a tab-delimited artifact store snapshot, writes and reads it through host file
helpers, rebuilds `DocArtifact` values through `Result<DocArtifact,Int>`
parsing helpers, and queries the best loaded record.
`examples/e310_vaisdb_artifact_query_report.vais` loads the persisted artifact
store into `List<DocArtifact>`, ranks records with `Map<Str,Int>` term scoring,
and returns a reusable text report as `Result<Str,Int>` while preserving
missing-store and empty-query error codes.
`examples/e311_result_call_argument_flow.vais` removes the next Result
ergonomics gap: `Result<Str,Int>` and `Result<DocArtifact,Int>` returning
helpers can feed other helper calls directly without manual local binding.
`examples/e312_result_struct_local_wrapper_flow.vais` removes the next
self-host wrapper ergonomics gap: `VaisResult<Struct>Int` payloads can be
copied through local struct variables and returned without losing fields.
`examples/e313_result_struct_str_match_flow.vais` removes the next structured
Result report ergonomics gap: `Result<Struct,Int>` matches can recover string
fields such as document titles and IDs while error arms stringify integer
codes.
`examples/e314_result_struct_str_concat_match_flow.vais` removes the follow-on
report-label ergonomics gap: `Result<Struct,Int>` matches can compose string
payload fields with nested `str_concat(...)` expressions while error arms
stringify integer codes.
`examples/e315_result_struct_str_transform_match_flow.vais` removes the next
normalization ergonomics gap: `Result<Struct,Int>` matches can transform string
payload fields with `str_replace`, `str_trim`, `str_upper`, `str_lower`, and
local-prefix `str_concat(...)` expressions while error arms stringify integer
codes.
`examples/e316_result_struct_str_transform_len_match_flow.vais` removes the
follow-on scoring ergonomics gap: `Result<Struct,Int>` matches can compute
integer scores from transformed string payload fields with chained `.len()`
terms while still mixing normal integer payload fields.
`examples/e317_result_struct_payload_helper_call_score.vais` removes the
reusable-helper ergonomics gap: `Result<Struct,Int>` matches can pass the `Ok`
payload struct directly to an `Int` scoring helper such as
`score_artifact(artifact)` while error arms recover integer codes.
`examples/e318_result_struct_payload_helper_call_arithmetic.vais` removes the
helper-composition ergonomics gap: `Result<Struct,Int>` matches can use a
reusable `Ok` payload helper-call result as one integer score term and add
ordinary payload fields in the same arm.
`examples/e319_result_struct_payload_field_helper_call_arithmetic.vais` removes
the field-helper composition ergonomics gap: `Result<Struct,Int>` matches can
pass `Ok` payload `Str` fields to reusable `Int` helpers and combine those
helper-call terms with ordinary payload fields in the same arm.
`examples/e320_result_struct_payload_int_field_helper_call_arithmetic.vais`
removes the numeric field-helper composition ergonomics gap:
`Result<Struct,Int>` matches can pass `Ok` payload `Int` fields to reusable
`Int` helpers and combine those helper-call terms with string-field helper
terms in the same arm.
`examples/e321_result_struct_payload_bool_match_condition.vais` removes the
Bool-return ergonomics gap: `Result<Struct,Int>` matches can return
conditions directly from `Ok` payload helper terms and `Err(Int)` code
comparisons.
`examples/e322_vaisdb_module_boundary/main.vais` removes the first reusable
module-boundary ergonomics gap: imported VaisDB scoring/artifact modules can
share `DocArtifact`, structured Result helpers, List outputs, and Map-backed
scoring helpers in direct/default runs.
`examples/e323_cli_package` removes the first package-directory CLI ergonomics
gap: `scripts/vaisc` can run or build a manifest-backed package directory,
resolve `source/main.vais`, preserve imports, and forward argv to the compiled
program.
`scripts/vaisc package <package-dir> -o <dist-dir>` removes the next packaging
ergonomics gap: it builds an argv-capable `dist/bin/<package-name>` binary and
copies `vais.toml` for the same direct/default package workflow.
`examples/e326_cli_binary_target` adds the first package target metadata:
optional `binary = "cmd-name"` changes the packaged command filename without
changing `source/main.vais` entry resolution. `scripts/vaisc package
<package-dir> -o <dist-dir> --archive` adds the first user-package release
payload: an extractable `<binary-or-name>-<version>.tar.gz` containing the
packaged binary and copied manifest.
`examples/e328_cli_package_assets` adds the first static asset package recipe:
optional `assets = "assets"` copies regular files/directories into
`dist/assets` and the archive payload as `assets/`.
`examples/e299_vaisdb_benchmark_report.vais` protects the follow-on benchmark
report recipe that uses `time_millis()`, document term counts, weighted
scoring, and file-backed report persistence in both direct and full paths.
`examples/e300_vaisdb_benchmark_cli_report.vais` protects the first
Vais-authored CLI-style benchmark/report recipe: it discovers the repository
root from `fs_cwd`, `path_dirname`, and `path_basename`, runs the indexer
through `proc_capture`, records elapsed milliseconds, and persists
direct/default status metrics.
`tools/vaisdb_benchmark_report.vais` promotes that recipe into a reusable
Vais-authored developer tool: it runs the indexer, parses saved report metrics
with line splitting, prefix checks, slicing, and `parse_int`, then writes a
direct/default summary report.

| API | Status |
| --- | --- |
| `fs_exists(path: Str) -> Bool` | Verified |
| `fs_is_dir(path: Str) -> Bool` | Verified; full/direct — directory test (missing paths yield 0) |
| `stdin_read_all() -> Str` | Verified; full/direct — read standard input to EOF (empty input yields "") |
| `stdout_write(text: Str) -> Int` | Verified; full/direct — raw stdout write, no added newline (returns bytes written) |
| `stderr_write(text: Str) -> Int` | Verified; full/direct — raw stderr write, no added newline (returns bytes written) |
| `fs_read_text(path: Str) -> Str` | Verified |
| `fs_write_text(path: Str, text: Str) -> Int` | Verified |
| `fs_mkdirs(path: Str) -> Int` | Verified |
| `fs_remove(path: Str) -> Int` | Verified |
| `fs_cwd() -> Str` | Verified; full/direct |
| `fs_temp_dir() -> Str` | Verified |
| `path_join(base: Str, child: Str) -> Str` | Verified |
| `path_basename(path: Str) -> Str` | Verified; full/direct |
| `path_dirname(path: Str) -> Str` | Verified; full/direct |
| `str_concat(left: Str, right: Str) -> Str` | Verified; full/direct |
| `str_cmp(left: Str, right: Str) -> Int` | Verified; full/direct — three-way byte comparison (-1/0/1) |
| `str_slice(text: Str, start: Int, len: Int) -> Str` | Verified; full/direct |
| `str_contains(text: Str, needle: Str) -> Int` | Verified |
| `str_index_of(text: Str, needle: Str) -> Int` | Verified; full/direct |
| `str_starts_with(text: Str, prefix: Str) -> Int` | Verified; full/direct |
| `str_ends_with(text: Str, suffix: Str) -> Int` | Verified; full/direct |
| `str_replace(text: Str, needle: Str, replacement: Str) -> Str` | Verified; full/direct |
| `str_trim(text: Str) -> Str` | Verified |
| `str_lower(text: Str) -> Str` | Verified |
| `str_upper(text: Str) -> Str` | Verified |
| `str_split_ws_into(text: Str, out: List<Str>) -> Int` | Verified; full/direct |
| `str_split_into(text: Str, sep: Str, out: List<Str>) -> Int` | Verified; full/direct |
| `str_split_lines_into(text: Str, out: List<Str>) -> Int` | Verified; full/direct |
| `str_join(parts: List<Str>, sep: Str) -> Str` | Verified; full/direct |
| `map_str_str_snapshot(docs: Map<Str,Str>) -> Str` | Verified; full/direct |
| `map_str_str_load_snapshot(text: Str, out: Map<Str,Str>) -> Int` | Verified; full/direct |
| `doc_term_counts_into(text: Str, out: Map<Str,Int>) -> Int` | Verified; full/direct |
| `doc_term_overlap_score(query: Map<Str,Int>, doc: Map<Str,Int>) -> Int` | Verified; full/direct |
| `doc_term_weighted_score(query: Map<Str,Int>, doc: Map<Str,Int>) -> Int` | Verified; full/direct |
| `str_byte(value: Int) -> Str` | Verified; full/direct |
| `fs_list_files(dir: Str, out: List<Str>) -> Int` | Verified; full/direct — sorted regular-file names, subdirectories skipped, missing directory yields 0 |
| `fs_list_dirs(dir: Str, out: List<Str>) -> Int` | Verified; full/direct — sorted subdirectory names, regular files skipped, missing directory yields 0 |
| `time_millis() -> Int` | Verified; full/direct |
| `proc_argc() -> Int` | Verified |
| `proc_arg(index: Int) -> Str` | Verified |
| `proc_run(argv: List<Str>) -> Int` | Verified |
| `proc_run_env(argv: List<Str>, env: List<Str>) -> Int` | Verified |
| `proc_capture_stdout(argv: List<Str>) -> Str` | Verified |
| `proc_capture_stderr(argv: List<Str>) -> Str` | Verified |
| `proc_capture_to(argv: List<Str>, stdout_path: Str, stderr_path: Str) -> Int` | Verified |
| `proc_capture(argv: List<Str>) -> ProcessResult` | Verified; full/direct |

## Collections

| API | Status |
| --- | --- |
| `[1, 2, 3]` | Verified for Int lists |
| `List<Int>` | Verified for local/parameter values, `contains`, `index_of`, `count`, `remove_at`, `insert_at`, `extend` from named lists, inline list literals, and list-returning helper calls, `first`, `max`, `min`, filtered `sum`/`len`/`max`/`min` lowering, and the documented list basics |
| `List<Struct>` | Verified for declared structs in the documented local, parameter, return, typed local literal including multiline trailing-comma literals, multiline inline literal call arguments including standalone call statements, assignment from inline struct list literals, `push` from struct values including multiline trailing-comma and single-field nested struct literals, list element values, mutating and non-mutating list method return values, filtered first/last whole-record selections, and struct-returning helper calls, `insert_at` including multiline struct literals, list element values, mutating and non-mutating list method return values, filtered first/last whole-record selections, and struct-returning helper calls, `clear`, `first`, `remove_at`, `extend` from named lists, inline struct list literals including multiline struct literal elements, and list-returning helper calls, method-result field chains including `Str` fields through `first`/`last`/`pop`/`remove_at`, single-field nested chains, and verified multi-field nested non-mutating method chains, indexed single-field nested field-chain reads/writes, indexed multi-field nested read/write/copy/push/parameter slices, indexed whole-element assignment including multiline struct literals and struct-returning helper calls, indexed field assignment including `Str` fields on local and parameter lists, filtered record count returns/assignments, filtered record score sum and max/min projection, filtered first/last field projection including inferred `Int`/`Str` locals, `Str` field `.len()` reads, and direct scalar `push`/`insert_at` arguments, filtered first/last whole-record selection including direct `push`/`insert_at` arguments, filtered projected scalar result lists including direct helper returns and direct helper-call arguments, and full/direct for-each slices |
| `List<Str>` | Partial; verified for typed local literals, inline literal call arguments, local/parameter-target assignment copy, literal assignment, return-call assignment, local/helper-parameter/helper-return `push`, `clear`, index, `first`, `last`, `pop`, `remove_at`, `insert_at`, `extend` from named lists, inline list literals, and list-returning helper calls, `map`/`filter` result lists when receiver type is known from a local or function parameter, direct `filter(...).map(...)` and `map(...).filter(...)` result-list contexts, direct `map(...).filter(...).len/contains/index_of/count` and `filter(...).map(...).len/contains/index_of/count` scalar contexts including arithmetic-tail reassignments, negated Bool expressions, Bool if-expression locals/reassignments/helper-call arguments/Bool returns, Int if-expression locals/reassignments/helper-call arguments/returns, nested helper-call arguments inside reassignments, and composite Bool local inference, `len`, `is_empty`, `contains`, `index_of`, `count`, full/direct for-each over local/parameter values, direct `.len()` chains and string equality on string element results, element assignment (`words[i] = value`), in-place ascending `sort()` statements on local and parameter lists, and host process arguments |
| `List<T>` | Partial |
| `Map<Int,Int>` | Verified for local values, local/parameter/return-call assignment copy, parameter reference/mutation, annotated or inferred return-value local initialization, `insert`, `remove`, `clear`, `get`, `get_opt`, `contains`, `len`, `key_at`, and `value_at` |
| `Map<Int,Bool>` | Verified for local values, local/parameter/return-call assignment copy, parameter reference/mutation, annotated or inferred return-value local initialization, `insert`, `remove`, `clear`, `get`, `get_opt`, `contains`, `len`, `key_at`, and `value_at` |
| `Map<Int,Char>` | Verified for local values, local/parameter/return-call assignment copy, parameter reference/mutation, annotated or inferred return-value local initialization, `insert`, `remove`, `clear`, `get`, `get_opt`, `contains`, `len`, `key_at`, and `value_at` |
| `Map<Str,Int>` | Verified for local values, local/parameter/return-call assignment copy, parameter reference/mutation, annotated or inferred return-value local initialization, `insert`, `remove`, `clear`, `get`, `get_opt`, `contains`, `len`, `key_at`, and `value_at` |
| `Map<Str,Bool>` | Verified for local values, local/parameter/return-call assignment copy, parameter reference/mutation, annotated or inferred return-value local initialization, `insert`, `remove`, `clear`, `get`, `get_opt`, `contains`, `len`, `key_at`, and `value_at` |
| `Map<Str,Char>` | Verified for local values, local/parameter/return-call assignment copy, parameter reference/mutation, annotated or inferred return-value local initialization, `insert`, `remove`, `clear`, `get`, `get_opt`, `contains`, `len`, `key_at`, and `value_at` |
| `Map<Str,Str>` | Verified for local values, local/parameter/return-call assignment copy, parameter reference/mutation, annotated or inferred return-value local initialization, `insert`, `remove`, `clear`, `get`, `get_opt` match binding and string match expression contexts including return-inferred locals and string helper transforms, reassigned `Str` locals with runtime `.len()`, Result helper flows over Map parameters, `contains`, `len`, `key_at`, `value_at`, and line snapshot round trips via `map_str_str_snapshot` / `map_str_str_load_snapshot` |
| `Map<K,V>` | Design-specified beyond the verified concrete local Map slices; not verified |
| `Option<Int>` | Verified for `Some(Int)`, `None`, helper returns, struct/local storage, statement `match`, expression-match binding, and local-binding `?` propagation, including native direct concrete value lowering |
| `Option<T>` | Specified beyond the `Option<Int>` slice |
| `Result<Int,Int>` | Verified for `Ok(Int)`, `Err(Int)`, helper returns, statement `match`, expression-match binding, local-binding `?` propagation, helper flows over `Map<Str,Str>` parameters with `get_opt` matches, and `fs_exists` guarded file-read error flows |
| `Result<Str,Int>` | Verified for file-read helper returns, helper parameters/forwarding, direct call-argument use of Result-returning helpers, `Ok(Str)`, `Err(Int)`, local-binding `?` propagation, and inline match recovery to `Int` or `Str` values |
| `Result<Metric,Int>` | Verified for the first structured payload slice: `Ok(Metric)`, `Err(Int)`, helper returns, helper parameters/forwarding, and inline match recovery of `Metric` fields to `Int` |
| `Result<DeclaredStruct,Int>` | Verified for declared struct payloads such as Int-field `Record`, multiline `Entry`, Str-field `DocSummary`, and VaisDB `DocArtifact`, including helper returns, parameters/forwarding, direct call-argument use of Result-returning helpers, explicit wrapper payload local copies, local-binding `?`, `List<Struct>` output storage, persisted store reload parsing, imported module-boundary reuse, inline match recovery of multiple fields or string field lengths, direct `Str` field recovery to string locals, nested `str_concat(...)` composition of `Str` fields, `str_replace`/`str_trim`/`str_upper`/`str_lower` normalization in match arms, transformed string `.len()` scoring in `Int` arms, `Ok` payload handoff to reusable `Int` scoring helpers, helper-call terms composed with normal payload fields, and Bool returns from payload helper terms plus `Err(Int)` comparisons |
| `Result<T,E>` | Specified beyond the `Result<Int,Int>`, `Result<Str,Int>`, and declared `Result<Struct,Int>` slices, including broader generic payload/error combinations |
| `v.len()` | Verified |
| `v.is_empty()` | Verified |
| `v.first()` | Verified for local and parameter `List<Int>`, `List<Str>`, and declared-struct list values |
| `v.last()` | Verified |
| `v.pop()` | Verified |
| `v[i]` | Verified |
| `v[i].field = x` | Verified for local and parameter declared-struct list values |
| `v.sum()` | Verified for local and parameter Int lists |
| `v.max()` | Verified for local and parameter `List<Int>` values; traps on empty lists |
| `v.min()` | Verified for local and parameter `List<Int>` values; traps on empty lists |
| `v.push(x)` | Verified for Int lists and selected self-host shapes, including declared-struct list element values, list method return values, and filtered first/last whole-record selections |
| `v.clear()` | Verified for `List<Int>`, `List<Str>`, and declared-struct list local/parameter slices |
| `v.contains(x)` | Verified for local and parameter `List<Int>` and `List<Str>` values |
| `v.index_of(x)` | Verified for local and parameter `List<Int>` and `List<Str>` values; returns first index or `-1` |
| `v.count(x)` | Verified for local and parameter `List<Int>` and `List<Str>` values; returns matching value count |
| `v.remove_at(i)` | Verified for local and parameter `List<Int>`, `List<Str>`, and declared-struct list values; returns the removed element and shifts following elements left |
| `v.insert_at(i, x)` | Verified for local and parameter `List<Int>`, `List<Str>`, and declared-struct list values, including declared-struct list element values, list method return values, and filtered first/last whole-record selections; shifts elements right and permits append-at-len |
| `v.extend(other)` | Verified for local and parameter `List<Int>`, `List<Str>`, and declared-struct list values with a same-type named list source; `List<Int>`, `List<Str>`, and declared-struct list values also accept inline list literals and same-type list-returning helper calls; `List<Int>`/`List<Str>` targets also accept direct `List<Struct>.filter(...).map(...)` projected scalar sources |
| `v.map(|x| ...)` | Verified for `List<Int>.map` and `List<Str>.map` with verified string builtin bodies, including `str_concat`; map bodies may capture known `Int` values for `List<Int>` or known `Str` values for `List<Str>`; declared `List<Struct>` values can project fields into reusable `List<Int>` locals, typed `List<Str>` locals, direct helper returns, helper-call arguments including `if`/`while`/`else if` conditions, `extend(...)` sources, and existing-list reassignments; same-item `List<Int>` map results and declared `List<Struct>` `Int` projections can aggregate directly with `sum()`, `max()`, and `min()` in returns, typed/inferred locals, helper-call arguments including simple arithmetic suffixes, standalone simple arithmetic suffixes, direct `List<Int>` mutation arguments, known `Int` reassignments, broader `Int` expressions, and broader `if`/`while`/`else if` condition expressions |
| `v.map(|x| INT).sum()` / `.max()` / `.min()` | Verified for same-item `List<Int>` transformed aggregation/ranking in direct `Int` returns, typed or inferred `Int` locals, helper-call arguments, direct `List<Int>` mutation arguments, known `Int` reassignments, broader `Int` expressions, and broader `if`/`while`/`else if` condition expressions; lowers directly over the source list |
| `v.filter(|x| BOOL)` | Verified for `List<Int>`, `List<Str>`, and declared `List<Struct>` result lists when the receiver type is known from a local or function parameter; predicates may capture known `Int` values for `List<Int>`, known `Str` values for `List<Str>`, or known locals/parameters plus struct fields for `List<Struct>` |
| `v.filter(|x| BOOL).map(|x| STR)` | Verified for `List<Str>` direct result lists in typed or inferred locals, helper returns, helper-call arguments including `if`/`while`/`else if` conditions, direct `extend(...)` sources, and existing-list reassignments |
| `v.map(|x| STR).filter(|x| BOOL)` | Verified for `List<Str>` direct result lists in typed or inferred locals, helper returns, helper-call arguments including `if`/`while`/`else if` conditions, direct `extend(...)` sources, and existing-list reassignments |
| `v.map(|x| STR).filter(|x| BOOL).len()` / `.contains(x)` / `.index_of(x)` / `.count(x)` | Verified for `List<Str>` direct scalar contexts in typed or inferred locals, helper returns, helper-call arguments, direct `List<Int>` mutation arguments, reassignments including arithmetic tails and nested helper-call arguments, negated Bool locals/reassignments/conditions, Bool if-expression locals/reassignments/helper-call arguments/Bool returns, Int if-expression locals/reassignments/helper-call arguments/returns, and `if`/`while`/`else if` conditions; multiple map-filter scalar calls from this same family, or mixed with filter-map scalar calls, may appear in one expression; composite Bool locals from these scalar conditions infer `Bool`; lowers directly over the source list without materializing a mapped result list |
| `v.filter(|x| BOOL).map(|x| STR).len()` / `.contains(x)` / `.index_of(x)` / `.count(x)` | Verified for `List<Str>` direct scalar contexts in typed or inferred locals, helper returns, helper-call arguments, direct `List<Int>` mutation arguments, reassignments including arithmetic tails and nested helper-call arguments, negated Bool locals/reassignments/conditions, Bool if-expression locals/reassignments/helper-call arguments/Bool returns, Int if-expression locals/reassignments/helper-call arguments/returns, and `if`/`while`/`else if` conditions; multiple filter-map scalar calls from this same family, or mixed with map-filter scalar calls, may appear in one expression; composite Bool locals from these scalar conditions infer `Bool` |
| `v.filter(|x| BOOL).sum()` | Verified for `List<Int>` filter-sum in `return` expressions and typed or inferred `Int` local assignments, including known local/parameter captures in the lowered predicate |
| `v.filter(|x| BOOL).len()` | Verified for `List<Int>`, `List<Str>`, and declared `List<Struct>` filtered counts in `return` expressions and typed or inferred `Int` local assignments, including known local/parameter captures and struct field predicates in the lowered predicate |
| `v.filter(|x| BOOL).max()` / `.min()` | Verified for `List<Int>` filtered extrema in `return` expressions and typed or inferred `Int` local assignments; lowers directly to a guarded selection loop and traps when no item matches |
| `v.filter(|x| BOOL).first().field` / `.last().field` | Verified for declared `List<Struct>` values in `return` expressions with `Int`/`Str` function return types, typed or inferred `Int`/`Str` local assignments, direct scalar `push`/`insert_at` arguments, and `Int`/`Str` helper-call arguments including helpers declared later, simple arithmetic suffixes on `Int` helper calls, and helper calls starting `if`, `else if`, or `while` conditions; `Str` field `.len()` chains are verified for `Int` returns, typed or inferred `Int` locals, direct `List<Int>` mutation arguments, and `Int` helper-call arguments; lowers directly to a guarded field-selection loop and traps when no item matches |
| `v.filter(|x| BOOL).first()` / `.last()` | Verified for declared `List<Struct>` values, including multiline struct declarations, in same-struct `return` expressions, typed or inferred same-struct local assignments, direct same-struct `push`/`insert_at` arguments, and same-struct helper-call arguments including helpers declared later, simple arithmetic suffixes on `Int` helper calls, and helper calls starting `if`, `else if`, or `while` conditions; lowers directly to a guarded record-selection loop and traps when no item matches |
| `v.filter(|x| BOOL).map(|x| x.field)` | Verified for declared `List<Struct>` projected result lists in reusable `List<Int>` locals, annotated `List<Str>` locals, existing `List<Int>`/`List<Str>` variable reassignments, direct `List<Int>`/`List<Str>` helper returns, direct `List<Int>`/`List<Str>` helper-call arguments, helper calls that start `if`, `while`, or `else if` condition expressions, and direct `List<Int>`/`List<Str>.extend(...)` arguments; lowers directly to a filtered projection loop without an intermediate record list |
| `v.filter(|x| BOOL).map(|x| INT).sum()` | Verified for same-item `List<Int>` transformed aggregation and declared `List<Struct>` score aggregation in `return` expressions, typed or inferred `Int` local assignments, direct `Int` helper-call arguments including helper calls starting `if`, `else if`, or `while` condition expressions, broader `Int` expressions used by locals, helper-call arguments, direct `List<Int>` mutation arguments, reassignments, returns, and broader `if`/`while`/`else if` condition expressions; lowers directly to an accumulator over the source list |
| `v.filter(|x| BOOL).map(|x| INT).max()` / `.min()` | Verified for same-item `List<Int>` transformed ranking and declared `List<Struct>` score ranking in `return` expressions, typed or inferred `Int` local assignments, direct `Int` helper-call arguments including helper calls starting `if`, `else if`, or `while` condition expressions, broader `Int` expressions used by locals, helper-call arguments, direct `List<Int>` mutation arguments, reassignments, returns, and broader `if`/`while`/`else if` condition expressions; lowers directly to a guarded selection loop |

Invalid list access traps at runtime. This includes negative or out-of-range
`v[i]`, `v.first()` on an empty list, `v.last()` on an empty list, and
`v.pop()`/`v.max()`/`v.min()` on an empty list, plus filtered
`max()`/`min()`, filtered first/last field projection, and filtered
first/last whole-record selection when no element matches. `v.pop()`
checks before mutating the list length. In the full self-host path and native
direct engine, `v.push(x)` traps before writing when a fixed backing list
buffer is full; the native direct engine applies the same check to list literal
initialization. `v.clear()` sets the list length to zero and reuses the same
backing storage.
`v.insert_at(i, x)` traps before mutating when `i < 0`, `i > len`, or the fixed
backing list buffer is full.
`v.extend(other)` traps before mutating when the combined target and source
length exceeds the fixed backing list buffer.
For `List<Str>`, string-returning `v[i]`, `v.first()`, `v.last()`, `v.pop()`, and
`v.remove_at(i)` results can feed directly into `.len()` and string equality in
the full self-host path and native direct engine.

### Map Slice

The verified Map surface is deliberately small:

| API | Verified behavior |
| --- | --- |
| `let m: Map<Int,Int> = {}` / `let m: Map<Int,Bool> = {}` / `let m: Map<Int,Char> = {}` / `let m: Map<Str,Int> = {}` / `let m: Map<Str,Bool> = {}` / `let m: Map<Str,Char> = {}` / `let m: Map<Str,Str> = {}` | Construct an empty local map |
| `fn f(m: Map<Int,Int>) -> Int` / `fn f(m: Map<Int,Bool>) -> Int` / `fn f(m: Map<Int,Char>) -> Int` / `fn f(m: Map<Str,Int>) -> Int` / `fn f(m: Map<Str,Bool>) -> Int` / `fn f(m: Map<Str,Char>) -> Int` / `fn f(m: Map<Str,Str>) -> Int` | Pass a Map by reference so the callee can read or mutate the caller-visible map |
| `fn make() -> Map<Int,Int>` / `let m = make()` / `let m: Map<Int,Int> = make()` / `fn make() -> Map<Str,Str>` / `let m = make()` / `let m: Map<Str,Str> = make()` | Return a verified concrete Map into caller-owned local storage, with optional local annotation |
| `target = source` | Copy a same-type local map, same-type Map parameter, or same-type Map-returning call into a Map target |
| `m.insert(key, value)` | Insert or replace a value by key; verified keys are `Int` for `Map<Int,V>` and `Str` for `Map<Str,Int>` / `Map<Str,Bool>` / `Map<Str,Char>` / `Map<Str,Str>` |
| `m.remove(key)` | Remove a key if present; missing keys are ignored |
| `m.clear()` | Remove all keys and allow the map to be reused |
| `m.get(key, default)` | Return the value for `key`, or `default` when absent |
| `m.get_opt(key)` | Return `Some(value)` for a present key or `None` when absent on `Map<Int,Int>`, `Map<Int,Bool>`, `Map<Int,Char>`, `Map<Str,Int>`, `Map<Str,Bool>`, `Map<Str,Char>`, or `Map<Str,Str>`; string-valued maps support direct match binding and `Str` match expressions in returns, reassignments, helper-call arguments, embedded conditions including `while` and `else if` chains, and embedded Int returns, including locals inferred from `Map<Str,Str>` helper returns, `str_concat`/`str_trim`/`str_lower`/`str_upper`/`str_replace` payload transforms, runtime `.len()` after reassignment, direct `.len()` after `str_trim`/`str_lower` match-arm transforms, and stable presence/value lowering instead of pointer-tagged string payload integers, rather than a general `Option<Str>` type annotation |
| `m.contains(key)` | Return whether `key` is present |
| `m.len()` | Return the number of present keys |
| `m.key_at(index)` / `m.value_at(index)` | Return the key/value at a compact entry index in `0..m.len()` for verified concrete Maps; intended for serialization/debugging, not sorted ordering |
| `map_str_str_snapshot(m)` | Return a `key=value\n` text snapshot for a local/parameter `Map<Str,Str>` in compact map entry order; intended for small metadata snapshots, not sorted output or escaping |
| `map_str_str_load_snapshot(text, out)` | Clear `out`, load `key=value` LF/CRLF lines into a local/parameter `Map<Str,Str>`, skip blank/malformed/no-key lines, preserve additional `=` bytes in values, allow empty values, and return the number of loaded entries |

This slice is currently available through the full self-host compiler path and
`scripts/vaisc --engine direct`.
The slice does not include broader generic key/value lowering, broader
broader `Map<Str,V>` return values, generic Map return values, iteration,
`Result`, hashing controls, or map literals with entries.
Unverified generic Map function parameters, unverified return values, and
non-promoted assignment sources are rejected by front diagnostics.
Future Map ABI and generic expansion rules are design-specified in
`docs/design/MAP_ABI.md`; they are not verified prelude APIs until compiler
gates cover them.

## Types And Conversion

| API | Status |
| --- | --- |
| `Int` | Verified |
| `Int8`..`Int128` | Specified |
| `UInt8`..`UInt128` | Specified |
| `F32`, `F64` | Specified |
| `Bool` | Verified for comparisons, boolean literals, local annotations, helper parameters, helper returns, and unary `not` |
| `Str` | Verified for literals, local annotations, scalar helper parameters and returns, length, index, equality, substring contains, first-index search, reassignment, `Str(Int)` conversion, and host-backed construction helpers |
| `Char` | Verified for single-byte literals, equality, annotations, helper parameters, and helper returns as Int-compatible scalar values |
| declared `struct` | Verified for literal construction, field reads/writes including helper-return field-chain reads, helper parameters/returns, helper-return assignment including direct returns of nested struct literals, single-field nested struct read/write including direct flattening for previously declared single-Int-field nested structs, scalar multi-field nested struct local literals/direct returns/field-chain reads, indexed field-chain reads/writes through `List<Struct>` elements containing single-field and verified multi-field nested shapes, and multiline struct literals in local initialization, same-type local assignment, and call arguments |
| `Int(x)` | Verified |
| `Str(x)` | Verified for Int-compatible scalar values |
| `parse_uint(s)` | Verified for `Str`; parses a leading unsigned decimal run |
| `parse_int(s)` | Verified for `Str`; accepts a leading `-` before the decimal run |
| `F64(x)`, `UInt8(x)` | Specified |

## Strings

| API | Status |
| --- | --- |
| `"text"` and `` `text` `` | Verified |
| `s.len()` | Verified |
| `s[i]` | Verified |
| `a == b`, `a != b` | Verified for `Str` in the full self-host path and native direct engine |
| `str_concat(left, right)` | Verified |
| `str_slice(text, start, len)` | Verified in the full host runtime and native direct engine; invalid ranges trap |
| `str_contains(text, needle)` | Verified; returns `1` when `needle` occurs in `text`, including empty `needle`; accepts verified Str-returning Map/List/struct-field expressions |
| `str_index_of(text, needle)` | Verified; returns the first byte index of `needle`, `-1` when absent, and `0` for an empty `needle` |
| `str_starts_with(text, prefix)` | Verified; returns `1` when `text` begins with `prefix`, including an empty `prefix`, otherwise `0` |
| `str_ends_with(text, suffix)` | Verified; returns `1` when `text` ends with `suffix`, including an empty `suffix`, otherwise `0`; accepts verified Str-returning Map/List and get_opt match expressions |
| `str_replace(text, needle, replacement)` | Verified; replaces all non-overlapping `needle` occurrences, returns a copy unchanged for an empty `needle`, and accepts verified Str-returning Map/List and get_opt match expressions |
| `str_trim(text)` | Verified; trims ASCII whitespace from both ends and accepts verified Str-returning Map/List expressions |
| `str_lower(text)` | Verified; lowercases ASCII `A-Z` and accepts verified Str-returning Map/List expressions |
| `str_upper(text)` | Verified; uppercases ASCII `a-z` and accepts verified Str-returning Map/List and get_opt match expressions |
| `str_split_ws_into(text, out)` | Verified; clears and fills a local/parameter `List<Str>` with ASCII-whitespace-delimited slices, then returns the token count |
| `str_split_into(text, sep, out)` | Verified; clears and fills a local/parameter `List<Str>` with delimiter-separated slices, preserves empty fields, treats an empty separator as one whole-text field, then returns the field count |
| `str_split_lines_into(text, out)` | Verified; clears and fills a local/parameter `List<Str>` with LF-delimited lines, trims a trailing CR from CRLF lines, preserves interior blank lines, omits the final empty line for a trailing line break, and returns the line count |
| `str_join(parts, sep)` | Verified; joins a local/parameter `List<Str>` with `sep` between elements, returns `""` for an empty list, and preserves delimiter round trips when paired with `str_split_into` |
| `doc_term_counts_into(text, out)` | Verified; clears and fills a local/parameter `Map<Str,Int>` with lowercase ASCII-whitespace-delimited term frequencies, then returns the total token count |
| `doc_term_overlap_score(query, doc)` | Verified; sums `min(query_count, doc_count)` for each query term in two `Map<Str,Int>` term-frequency maps |
| `doc_term_weighted_score(query, doc)` | Verified; sums `query_count * doc_count` for each query term in two `Map<Str,Int>` term-frequency maps |
| `examples/e296_result_map_param_flow.vais` | Verified recipe for `Result<Int,Int>` helper chains over `Map<Str,Str>` metadata parameters, including `get_opt` match and local-binding `?` propagation |
| `examples/e295_vaisdb_indexer_prototype.vais` | Verified recipe combining metadata snapshot, document ingest, term counts, and weighted query scoring in Vais; `scripts/test-vaisdb-workflow.sh` and `scripts/bench-vaisdb-indexer.sh` cover reproducibility and local timing |
| `examples/e297_vaisdb_file_ingest_workflow.vais` | Verified recipe for file-backed VaisDB ingest with `fs_read_text`, generated temp files, argv-supplied document/query paths, line splitting, metadata snapshots, term counts, and weighted query scoring |
| `examples/e298_vaisdb_file_ingest_result_flow.vais` | Verified recipe for `fs_exists` guarded file-backed ingest that reports missing or malformed document/query paths with `Result<Int,Int>` error codes |
| `examples/e301_result_str_int_file_read.vais` | Verified recipe for `fs_exists` guarded text reads that return `Result<Str,Int>`, propagate string payloads with `?`, and recover missing-file error codes through inline match |
| `examples/e302_result_str_int_param_flow.vais` | Verified recipe for passing `Result<Str,Int>` values through helper parameters, forwarding them to other helpers, and recovering payload/error values with inline matches |
| `examples/e303_result_metric_int_struct_payload.vais` | Verified recipe for passing `Result<Metric,Int>` structured payload values through helper parameters and recovering payload fields with inline matches |
| `examples/e304_result_record_int_struct_payload.vais` | Verified recipe for passing declared Int-field struct payloads through `Result<DeclaredStruct,Int>` helpers and recovering multiple fields with inline matches |
| `examples/e305_result_multiline_struct_payload.vais` | Verified recipe for passing multiline declared Int-field struct payloads through `Result<DeclaredStruct,Int>` helpers and recovering multiple fields with inline matches |
| `examples/e306_result_struct_str_fields.vais` | Verified recipe for passing declared struct payloads with `Str` fields through `Result<DeclaredStruct,Int>` helpers and recovering string field lengths with inline matches |
| `examples/e307_result_struct_try_payload.vais` | Verified recipe for extracting declared struct payloads from `Result<DeclaredStruct,Int>` with local-binding `?`, reusing `Str`/`Int` fields, and propagating integer errors early |
| `examples/e308_vaisdb_artifact_record_workflow.vais` | Verified recipe for building VaisDB-style `DocArtifact` records through `Result<DeclaredStruct,Int>`, pushing them into `List<Struct>` outputs, snapshotting metadata, and propagating integer errors |
| `examples/e309_vaisdb_artifact_store_snapshot.vais` | Verified recipe for serializing `List<DocArtifact>` values to a text artifact-store snapshot, writing/reading it with host file helpers, parsing records back through `Result<DeclaredStruct,Int>`, and querying the best loaded record |
| `examples/e310_vaisdb_artifact_query_report.vais` | Verified recipe for loading persisted `List<DocArtifact>` stores, ranking records with `Map<Str,Int>` term scoring, returning reusable `Result<Str,Int>` report payloads, and preserving query/store error codes |
| `examples/e311_result_call_argument_flow.vais` | Verified recipe for passing `Result<Str,Int>` and `Result<DeclaredStruct,Int>` returning helpers directly as call arguments without manual temporary locals |
| `examples/e312_result_struct_local_wrapper_flow.vais` | Verified recipe for copying declared-struct Result wrapper payloads through local struct variables and returning them without losing nested fields |
| `examples/e313_result_struct_str_match_flow.vais` | Verified recipe for recovering `Str` fields from declared-struct Result matches and converting `Err(Int)` codes to strings |
| `examples/e314_result_struct_str_concat_match_flow.vais` | Verified recipe for composing declared-struct Result `Str` fields with nested `str_concat(...)` match arms and converting `Err(Int)` codes to strings |
| `examples/e315_result_struct_str_transform_match_flow.vais` | Verified recipe for normalizing declared-struct Result `Str` fields with `str_replace`, `str_trim`, `str_upper`, `str_lower`, and local-prefix `str_concat(...)` match arms while converting `Err(Int)` codes to strings |
| `examples/e316_result_struct_str_transform_len_match_flow.vais` | Verified recipe for scoring declared-struct Result payloads by applying string transforms in match arms and using chained `.len()` terms alongside integer fields |
| `examples/e317_result_struct_payload_helper_call_score.vais` | Verified recipe for passing declared-struct Result `Ok` payloads to reusable scoring helpers from match arms while preserving integer error recovery |
| `examples/e318_result_struct_payload_helper_call_arithmetic.vais` | Verified recipe for composing reusable declared-struct Result `Ok` payload helper-call terms with normal payload fields in integer match arms |
| `examples/e319_result_struct_payload_field_helper_call_arithmetic.vais` | Verified recipe for passing declared-struct Result `Ok` payload string fields to reusable integer helpers and composing the results with normal payload fields |
| `examples/e320_result_struct_payload_int_field_helper_call_arithmetic.vais` | Verified recipe for passing declared-struct Result `Ok` payload integer fields to reusable integer helpers and composing the results with string-field helper terms |
| `examples/e321_result_struct_payload_bool_match_condition.vais` | Verified recipe for returning Bool conditions from declared-struct Result payload helper terms and error-code comparisons |
| `examples/e322_vaisdb_module_boundary/main.vais` | Verified recipe for imported VaisDB modules that share `DocArtifact`, structured Result helpers, `List<DocArtifact>` outputs, and `Map<Str,Int>` scoring helpers |
| `examples/e323_cli_package` | Verified recipe for package-directory execution with manifest entry resolution, imports, and CLI argv forwarding |
| `scripts/vaisc package examples/e323_cli_package -o <dist-dir>` | Verified recipe for installable package output with argv-capable `<dist-dir>/bin/e323_cli_package`, copied `<dist-dir>/vais.toml`, and unsafe manifest-name rejection in native/direct/workflow gates |
| `examples/e326_cli_binary_target` | Verified recipe for optional package `binary` metadata that installs `<dist-dir>/bin/veriqel-demo` instead of the package name while keeping package-directory entry resolution and argv forwarding |
| `scripts/vaisc package examples/e326_cli_binary_target -o <dist-dir> --archive` | Verified recipe for extractable user-package release archives with `<binary-or-name>-<version>/bin/<binary-or-name>` and copied manifest payloads |
| `examples/e328_cli_package_assets` | Verified recipe for optional package `assets` metadata copied to `<dist-dir>/assets` and archived as `<binary-or-name>-<version>/assets/` alongside an argv-capable packaged binary |
| `examples/e299_vaisdb_benchmark_report.vais` | Verified recipe for a Vais-authored benchmark/report workflow using `time_millis`, document term counts, weighted scoring, and persisted report validation |
| `examples/e300_vaisdb_benchmark_cli_report.vais` | Verified recipe for a Vais-authored CLI-style benchmark/report workflow that discovers the repo root, invokes the indexer through `proc_capture`, times direct/default runs, and persists status metrics |
| `tools/vaisdb_benchmark_report.vais` | Verified reusable tool for generating a raw VaisDB benchmark report, parsing metric lines, and writing a direct/default summary report |
| `str_byte(value)` | Verified; full/direct; values outside `0..255` trap |
| `Str(x)` | Verified for Int-compatible scalar values |
| byte-classification helpers such as `is_digit(c: Int) -> Bool` | Verified pattern |
| single-byte `Char` literals such as `'A'` | Verified for equality, `Char` locals, helper parameters, and helper returns |
| `parse_uint(s)`, `parse_int(s)` | Verified |

`parse_uint` and `parse_int` stop at the first non-decimal byte. Empty input or
input with no leading decimal digit returns `0`; `parse_int("-5")` returns `-5`.

## Control And Operators

| API | Status |
| --- | --- |
| `and`, `or`, `not` | Verified |
| `bitnot(x)` | Verified |
| `bitand(a,b)`, `bitor(a,b)`, `bitxor(a,b)` | Verified |
| `shl(x,n)`, `shr(x,n)` | Verified |
| `break`, `continue` | Verified |

## Freeze Rules

New prelude entries must land with examples and value-correctness tests before
they are described as verified. Broader generic collection or result APIs remain
specified only until their concrete compiler and runtime gates exist.
