# Vais Changelog

## Unreleased

### Changed

- Whitespace hygiene is now a ladder gate: `scripts/vaisfmt-check.sh`
  packages vaisfmt and checks the std/examples/compiler/tools `.vais`
  trees, and `tools/gates.tasks` runs it as the `fmt` task in both the
  quick and ladder chains. Dogfooding it exposed a real capacity limit —
  `str_split_lines_into` traps past the 4095-entry fixed list contract on
  the ~23k-line self-host source — so vaisfmt now streams lines by byte
  offset with no per-line list (a generated 5000-line self-test case
  protects the path), and the list-contract ceiling is documented.

- Added the fourth installable Vais tool: `examples/e346_vaisfmt_package`
  builds `dist/bin/vaisfmt`, normalizing Vais source whitespace (trailing
  spaces/tabs stripped, exactly one trailing newline, blank lines
  preserved) with `-c` check and in-place fix modes over recursive
  `.vais` tree walks. The sprint promoted the `str_builder_new/push/
  append/finish` chain to the direct engine (predicates, argument
  wiring with per-position Int/Str expectations, prototypes, and
  inference), which only the full engine accepted before. The workflow
  gate covers check/fix/recheck plus a live `std/` cleanliness check.

- The Vais gate ladder is now runnable through vaismake itself:
  `tools/gates.tasks` chains every gate with `!needs` (plus a
  front/direct/check `quick` subset) and `scripts/vaismake-ladder.sh`
  packages the tool and drives the full ladder with first-failure stop
  and exit propagation. The workflow gate asserts the tasks file parses
  (13 tasks). Self-referential dogfooding: a Vais-built tool now
  operates the Vais project's own verification.

- vaismake gains `!needs <task> <dep...>` dependency lines: dependencies
  run before the task (each at most once via a visited map), the first
  failing child stops the chain with its exit code, and dependency cycles
  are refused with exit 4. The resolver is `@` self-recursion over a
  `Map<Str,Int>` state in product code; the workflow gate adds
  deps-first, failure-stop, and cycle cases. Zero compiler gaps.

- Promoted `proc_run_env(argv, env)` to the direct engine (setenv-overlay
  child environments, mirroring the proc_run wiring with a two-list
  argument shape) and taught vaismake `!env NAME=VALUE` task-file lines
  that overlay the child environment in run mode. The workflow gate adds
  an env-overlay case and a three-tool chain case where a vaismake task
  runs the packaged vaisgrep binary and propagates its exit code.
  `examples/e345_proc_run_env.vais` covers the built-in on both engines.

- Added the third installable Vais tool: `examples/e344_vaismake_package`
  builds `dist/bin/vaismake` (named tasks from a plain
  `name = command args...` file, whitespace-split argv with no shell,
  exit = child exit, plus `-o` stdout capture and a task listing mode).
  The sprint promoted `proc_run(argv)` to the direct engine, which only
  had `proc_capture`: a lean fork/execvp/waitpid helper plus expression,
  statement, and inference wiring, so both engines now run children
  without requiring the ProcessResult struct.

- Promoted `@(args)` self-recursion for every expression position: the
  driver now rewrites `@(` to the enclosing function's name before either
  engine runs (one text lowering serves the full, embed, and direct
  pipelines), fixing the previous full-engine mistype and direct-engine
  rejection outside tail positions. The vaisgrep tree walk now uses `@`
  in product code; `examples/e343_self_recursion_at.vais` covers tail,
  compound, and nested call-argument shapes.

- Added the built-in `fs_list_dirs(dir, out)` host API (sorted subdirectory
  names, regular files skipped, missing directories yield 0) and a `-r`
  recursive tree-walk mode to vaisgrep that prints `sub/dir/name:N: line`
  relative paths (`examples/e342_fs_list_dirs.vais`). Fixed the direct
  engine double-rewriting helper-call arguments: builtin calls nested
  inside a declared helper's arguments (`ident(path_join(dir, xs[j])`)
  were rewritten twice, so the parse-builtin and Str-conversion passes now
  treat already-rewritten helper-call groups as opaque. Named self-recursion
  is the verified recursion surface (`@(...)` in argument/compound positions
  is registered as a candidate).

- Added the second installable Vais tool: `examples/e341_vaisgrep_package`
  builds `dist/bin/vaisgrep` (substring line search over a file or a
  directory of text files, plus `-c` per-file counts, with a deterministic
  no-argument self-test). The sprint promoted the `fs_is_dir(path)` host
  API it exposed as a gap: `fs_exists` answers 1 for directories too, so
  path dispatch needed a real directory test on both engines.

- Promoted the `List<Str>` ordering surface (dogfood-4 gap feedback):
  `List<Str>` element assignment (`words[i] = value`) now works on both
  engines (the full engine converts pointer element values before the i64
  slot store; the direct engine accepts Str list element targets), the new
  `str_cmp(left, right) -> Int` built-in compares strings three-way
  (-1/0/1), and `List<Str>.sort()` sorts in place on local and parameter
  receivers through the shared sort desugar. The vaisdb `docs` subcommand
  now prints doc ids in sorted order. Covered by
  `examples/e340_list_str_sort.vais`.

- The vaisdb package gains document management: `docs <index>` (lists unique
  doc ids, exit = count), `remove <index> <doc-id>` (drops every key of a
  document by rebuilding the index, exit 3 when missing), and
  `stats <index>` (docs/terms summary line). All covered by packaged-binary
  workflow gate cases; the direct-engine `fs_mkdirs` prototype is now
  const-correct (warning-free build).

- The full engine now rejects calls to unknown functions at the front
  (`error: call to an unknown function`) instead of emitting a bare call
  that surfaced later as a confusing clang type or link error. Declared
  functions (including `pub fn`), imported modules, locals/parameters, and
  the documented built-in surface stay accepted; the front gate covers the
  rejection.

- Fixed the direct engine rejecting `List<Struct>` indexed field reads inside
  `Str(...)` conversion arguments (`str_concat("=", Str(xs[j].score))`): the
  list rewriter now skips conversion-call interiors like other string-helper
  builtins, so the conversion pass translates raw source instead of
  double-rewriting already-translated text. Covered by
  `examples/e339_list_struct_field_in_call_args.vais` and the vaisdb package
  `rank` report, which now reads ranked entries inline without a let-binding
  workaround.

- Added the built-in `fs_list_files(dir, out)` host API: fills a `List<Str>`
  with the sorted regular-file names in a directory (subdirectories skipped,
  missing directories yield 0), verified on both engines
  (`examples/e338_fs_list_files.vais`), and promoted `fs_mkdirs` to the direct
  engine. The vaisdb package gains `ingest-dir <index> <dir>` (ingests every
  `.txt`, doc id = file name without extension) and `rank <index> <query> <k>`
  (top-k lines ordered with the new `sort_by_desc`), both covered by the
  workflow gate against the packaged binary.

- Added the installable vaisdb CLI package
  (`examples/e337_vaisdb_cli_package`): the ingest/query/report command logic
  is split across `vaisdb.index`/`vaisdb.report` package modules,
  `scripts/vaisc package` builds `dist/bin/vaisdb` plus a versioned release
  archive, and the VaisDB workflow gate exercises the packaged binary's
  subcommands and self-test from the dist tree and the extracted archive —
  the first distributable Vais-authored tool.
- Added the built-in `List<Struct>.sort_by(|x| x.int_field)` and
  `sort_by_desc` statements, completing dogfood-2 gap feedback #1: in-place
  key sorts over an Int field on local and parameter receivers, sharing the
  same driver desugar as `List<Int>.sort()`
  (`examples/e336_list_struct_sort_by.vais`).
- Added the built-in `List<Int>.sort()` statement (dogfood-2 gap feedback #1):
  an in-place ascending sort desugared once in the driver so the full and
  direct engines share the lowering, verified for local and parameter
  receivers, duplicates, sorted input, and empty lists
  (`examples/e335_list_int_sort.vais`). Promoting it exposed and root-fixed a
  full-engine crash on element writes through pointer-aliased `List<Int>`
  slots (parameter receivers), which previously clobbered the stored buffer
  pointer.

- Added the VaisDB dogfooding-2 workflow slice: a top-k ranking report over a
  hand-written `List<Struct>` selection sort
  (`examples/e332_vaisdb_topk_ranking_report.vais`), versioned metadata
  snapshots with v1 migration and readable version errors
  (`examples/e333_vaisdb_snapshot_version_migration.vais`), a persisted
  incrementally-extended term index
  (`examples/e334_vaisdb_index_persistence_incremental.vais`), and the
  Vais-authored `vaisdb` CLI (`tools/vaisdb_cli.vais`,
  `scripts/vaisdb-cli.sh`) with ingest/query/report subcommands, all covered
  by the VaisDB workflow, parity, and value gates.

- Promoted `Result<Str,Str>`, the first non-`Int` error payload slice: `Ok(Str)`
  or `Err(Str)` so failures carry human-readable messages, with helper returns,
  local-binding `?` propagation, and inline match recovery, verified in the
  native direct engine and the full self-host compiler
  (`examples/e329_result_str_str_error_message.vais`) and dogfooded in a VaisDB
  file-ingest workflow (`examples/e330_vaisdb_ingest_error_message_flow.vais`).
- Hardened the `Option`/`Result` diagnostics: misused concrete shapes
  (non-`Int`/`Str` error payloads, undeclared struct payloads) and nested
  `Option`/`Result` payloads are rejected with a `help:` message listing the
  verified shapes, now pinned by the `bad.vais` diagnostic-count gate and
  dedicated front-contract reject cases. The verified/rejected surface is
  documented in `docs/reference/LANGUAGE.md`.
- Added optional package static assets: `assets = "assets"` in `vais.toml`
  makes `scripts/vaisc package` copy regular files/directories to
  `<dist-dir>/assets`; `--archive` includes the same payload as
  `<binary-or-name>-<version>/assets/`. Native, direct, manifest-contract, and
  VaisDB workflow gates now cover the asset package example and diagnostics for
  unsafe or missing asset directories.
- Added package release archives: `scripts/vaisc package <package-dir> -o
  <dist-dir> --archive` now also writes
  `<dist-dir>/<binary-or-name>-<version>.tar.gz`, containing the packaged
  binary under `bin/` and the copied `vais.toml`. Native, direct, and VaisDB
  workflow gates extract the archive and run the packaged command, and unsafe
  manifest versions are rejected before they can become archive filenames.
- Added optional package binary target metadata: `binary = "cmd-name"` in
  `vais.toml` lets `scripts/vaisc package` write
  `dist/bin/<cmd-name>` while `emit-ir`, `build`, and `run` still resolve the
  package entry through `source/main.vais`.
- Hardened installable package output gates: packaged
  `examples/e323_cli_package` binaries are now verified with real CLI argv in
  native, direct, and VaisDB workflow checks, and `vaisc package` rejects
  manifest names that cannot be used safely as binary filenames.
- Added installable package output: `scripts/vaisc package <package-dir> -o
  <dist-dir>` now resolves a manifest-backed package directory, builds
  `dist/bin/<package-name>`, copies `vais.toml` to `dist/vais.toml`, and is
  covered in full/default and direct package workflow gates.
- Added `examples/e323_cli_package`; `scripts/vaisc emit-ir`, `build`, and
  `run` now accept a package directory as well as an explicit `.vais` entry
  file. A directory with `vais.toml` resolves to `source/main.vais`, direct and
  full/default engines share that entry resolution, and the workflow gates
  cover package-directory execution with CLI argv forwarding.
- Added `examples/e322_vaisdb_module_boundary/main.vais`; direct/default,
  front, workflow, and parity gates now protect imported VaisDB-style modules
  sharing `DocArtifact` structs, `Result<DocArtifact,Int>` helpers,
  `List<DocArtifact>` outputs, and `Map<Str,Int>` scoring helpers. The direct
  native engine now resolves the same static dotted local imports as the full
  engine before lowering.
- Added `examples/e321_result_struct_payload_bool_match_condition.vais`;
  direct/default, front, workflow, parity, and full codegen gates now protect
  `Result<DeclaredStruct,Int>` matches that return `Bool` conditions from
  `Ok` payload helper terms and `Err(Int)` code comparisons.
- Added `examples/e320_result_struct_payload_int_field_helper_call_arithmetic.vais`;
  direct/default, front, workflow, parity, and full codegen gates now protect
  `Result<DeclaredStruct,Int>` matches that pass `Ok` payload `Int` fields to
  reusable `Int` helpers and compose those helper-call terms with normal struct
  field terms while preserving `Err(Int)` recovery.
- Added `examples/e319_result_struct_payload_field_helper_call_arithmetic.vais`;
  direct/default, front, workflow, parity, and full codegen gates now protect
  `Result<DeclaredStruct,Int>` matches that pass `Ok` payload `Str` fields to
  reusable `Int` helpers and compose those helper-call terms with normal struct
  field terms while preserving `Err(Int)` recovery.
- Added `examples/e318_result_struct_payload_helper_call_arithmetic.vais`;
  direct/default, front, workflow, parity, and full codegen gates now protect
  `Result<DeclaredStruct,Int>` matches that compose reusable `Ok` payload
  helper-call terms with normal struct field terms while preserving
  `Err(Int)` recovery.
- Added `examples/e317_result_struct_payload_helper_call_score.vais`;
  direct/default, front, workflow, parity, and full codegen gates now protect
  `Result<DeclaredStruct,Int>` matches that pass `Ok` payload structs directly
  to reusable `Int` scoring helpers while preserving `Err(Int)` recovery.
- Added `examples/e316_result_struct_str_transform_len_match_flow.vais`;
  direct/default, front, workflow, parity, and full codegen gates now protect
  `Result<DeclaredStruct,Int>` matches that compute `Int` scores from
  transformed `Str` payload fields with chained `.len()` calls while preserving
  `Err(Int)` recovery.
- Added `examples/e315_result_struct_str_transform_match_flow.vais`; direct/default,
  front, workflow, parity, and full codegen gates now protect
  `Result<DeclaredStruct,Int>` matches that normalize `Str` payload fields with
  `str_replace`, `str_trim`, `str_upper`, `str_lower`, and local-prefix
  `str_concat(...)` while converting `Err(Int)` codes to strings.
- Added `examples/e314_result_struct_str_concat_match_flow.vais`; direct/default,
  front, workflow, parity, and full codegen gates now protect
  `Result<DeclaredStruct,Int>` matches that compose `Str` payload fields with
  `str_concat(...)` while converting `Err(Int)` codes to strings. Full
  self-host lowering now treats struct Result string arms as recursive string
  expressions rather than single-field-only arms.
- Added `examples/e313_result_struct_str_match_flow.vais`; direct/default,
  front, workflow, parity, and full codegen gates now protect
  `Result<DeclaredStruct,Int>` matches that recover `Str` fields from
  struct payloads and convert `Err(Int)` codes to strings. Native lowering now
  reuses payload-local copies for struct Result match binders, and full
  self-host lowering emits i8* stores for matched struct string fields.
- Added `examples/e312_result_struct_local_wrapper_flow.vais`; direct/default,
  front, workflow, parity, and full codegen gates now protect copying
  declared-struct Result wrapper payloads through local struct variables and
  returning those locals without losing nested fields. Full self-host lowering
  now recognizes struct-typed field-chain lets such as `let artifact =
  flow.value` and copies nested struct locals field-by-field into wrapper
  literals.
- Added `examples/e311_result_call_argument_flow.vais`; direct/default,
  front, workflow, parity, and full codegen gates now protect passing
  `Result<Str,Int>` and `Result<DeclaredStruct,Int>` returning helpers directly
  as helper-call arguments without first binding manual locals. Native lowering
  now hoists Result-returning call arguments and clears Result local-name caches
  at function boundaries so same-named parameters in later helpers do not leak
  stale payload lowering state.
- Hardened native `vaisc` temporary-file handling: generated native driver
  intermediates now live under a per-run temp root, are removed on normal
  exit, and have a smoke-test regression guard while `--keep-tmp` still
  preserves artifacts for debugging.
- Added `examples/e310_vaisdb_artifact_query_report.vais`; direct/default,
  front, workflow, parity, and full codegen gates now protect a reusable
  persisted artifact-store query/report workflow that loads
  `List<DocArtifact>` records, ranks them through term scoring, composes
  `Result<Str,Int>` report payloads, and preserves missing-store/empty-query
  integer errors. Native and full lowering now handle `Result<Int,Int>?` and
  `Result<DeclaredStruct,Int>?` propagation inside `Result<Str,Int>` functions.
- Added `examples/e309_vaisdb_artifact_store_snapshot.vais`; direct/default,
  front, workflow, parity, and full codegen gates now protect a persistable
  VaisDB artifact store workflow that serializes `List<DocArtifact>` records,
  reloads them through `Result<DocArtifact,Int>` parsing helpers, queries the
  best loaded record, and reports malformed/missing store errors.
- Added `examples/e308_vaisdb_artifact_record_workflow.vais`; direct/default,
  front/checker, workflow, parity, and full codegen gates now protect a
  VaisDB-style artifact record workflow that builds
  `Result<DeclaredStruct,Int>` document payloads, stores them in
  `List<Struct>` outputs, snapshots `Map<Str,Str>` metadata, and propagates
  integer errors.
- Added `examples/e307_result_struct_try_payload.vais`; direct/default,
  front/checker, workflow, parity, and full codegen gates now protect
  local-binding `?` propagation for `Result<DeclaredStruct,Int>` payload
  structs, including reuse of extracted `Str` and `Int` fields.
- Added `examples/e306_result_struct_str_fields.vais`; direct/default,
  front/checker, workflow, parity, and full codegen gates now protect
  `Result<DeclaredStruct,Int>` payload structs with `Str` fields, including
  inline recovery of string field lengths plus integer fields.
- Added `examples/e305_result_multiline_struct_payload.vais`; direct/default,
  front/checker, workflow, parity, and full codegen gates now protect
  `Result<DeclaredStruct,Int>` payloads declared with multiline Int-field
  structs, including 4-field inline recovery.
- Added `examples/e304_result_record_int_struct_payload.vais`; direct/default,
  front/checker, workflow, parity, and full codegen gates now protect
  `Result<DeclaredStruct,Int>` structured payload forwarding beyond the
  previous `Metric`-only slice, including 3-field inline recovery.
- Added `examples/e303_result_metric_int_struct_payload.vais`; direct/default,
  front/checker, workflow, parity, and full codegen gates now protect the first
  concrete `Result<Metric,Int>` structured payload slice, including helper
  returns, helper parameters, forwarding, and inline recovery of struct fields
  or integer error values.
- Added `examples/e302_result_str_int_param_flow.vais`; direct/default/full,
  front/checker, workflow, and parity gates now protect `Result<Str,Int>`
  helper parameters, forwarding between helpers, and inline recovery of
  string payloads or integer error values.
- Added `examples/e301_result_str_int_file_read.vais`; direct/default/full,
  front/checker, workflow, and parity gates now protect the first
  `Result<Str,Int>` file-read slice with `Ok(Str)`, `Err(Int)`,
  local-binding `?`, and inline match recovery to both `Int` and `Str` values.
- Added `tools/vaisdb_benchmark_report.vais` and
  `scripts/vaisdb-benchmark-report.sh`; Vais now has a reusable benchmark
  report command that runs the e295 indexer, parses raw metric lines with
  `str_split_lines_into`/`str_starts_with`/`str_slice`/`parse_int`, and writes
  a direct/default summary. Front/direct/full/workflow/parity gates cover the
  new tool path.
- Added `examples/e300_vaisdb_benchmark_cli_report.vais`; direct/default/full
  and parity gates now protect a Vais-authored CLI-style benchmark report that
  discovers the repo root with `fs_cwd`/`path_dirname`/`path_basename`, invokes
  the e295 indexer through `proc_capture`, records direct/default elapsed
  milliseconds, and persists status metrics.
- Added `time_millis() -> Int` to the verified host intrinsic surface and
  `examples/e299_vaisdb_benchmark_report.vais`; direct/default/full/parity
  gates now protect a Vais-authored benchmark report workflow that times term
  counting/scoring and persists the report through `fs_write_text`/`fs_read_text`.
- Added `examples/e298_vaisdb_file_ingest_result_flow.vais`; the native direct
  emitter now lowers `fs_exists(path)`, the focused VaisDB gate checks
  generated-file, argv-file, and missing-document `Err(10)` modes, and the
  full codegen gate protects the standalone `fs_exists` + `Result<Int,Int>`
  file-ingest shape.
- Added `examples/e297_vaisdb_file_ingest_workflow.vais`; the native direct
  emitter now lowers the minimal file/argv host helpers needed for the
  workflow (`fs_read_text`, `fs_write_text`, `fs_temp_dir`, `path_join`,
  `proc_argc`, and `proc_arg`), the focused VaisDB gate checks generated-file
  and argv-file ingest modes, and the full codegen gate links a host runtime
  for standalone generated-IR file workflows.
- Promoted `examples/e296_result_map_param_flow.vais`; full self-host
  `fixpoint_full.vais` now lowers `Ok`/`Err`, local-binding `?`, and
  payload-only `match Result` shapes needed for `Result<Int,Int>` helper flows
  over `Map<Str,Str>` parameters, then regenerated
  `compiler/self/vaisc_core.ll`.
- Added `docs/design/VAISDB_DX_BASELINE.md`,
  `scripts/test-vaisdb-workflow.sh`, and
  `scripts/bench-vaisdb-indexer.sh` so the document/VaisDB workflow has a
  focused direct/default reproducibility gate and a local performance baseline
  protocol.
- Added `examples/e295_vaisdb_indexer_prototype.vais`, a gate-backed
  Vais-authored document indexer prototype that combines ingest, metadata
  snapshot round trips, term-frequency maps, and weighted query scoring.
- Promoted concrete native direct `Option<Int>`/`Result<Int,Int>` value
  lowering for helper return/parameter/local types, constructors, inline
  expression-match bindings, and local-binding `?`; added
  `examples/e294_result_try_parse_error_flow.vais` to lock a document-style
  parse/error flow without opening generic `Option<T>`/`Result<T,E>`.
- Added gate-backed `map_str_str_snapshot(docs)` and
  `map_str_str_load_snapshot(text, out)` support across the full self-host and
  native direct paths for small `Map<Str,Str>` line snapshots used by VaisDB
  metadata round trips.
- Added gate-backed `str_split_lines_into(text, out)` support across the full
  self-host and native direct paths for LF/CRLF line tokenization into
  `List<Str>` out-params, including interior blank lines, empty input, and
  trailing-line-break handling.
- Promoted full self-host lowering for `str_concat(left, right)` and
  `str_byte(value)` to self-contained runtime helpers so standalone generated
  IR no longer depends on external host string-construction calls for those
  helpers.
- Added gate-backed `str_join(parts, sep)` support across the full self-host
  and native direct paths for reconstructing `List<Str>` values with a
  separator, including empty-list handling and split/join delimiter round trips.
- Added gate-backed `str_split_into(text, sep, out)` support across the full
  self-host and native direct paths for delimiter tokenization into `List<Str>`
  out-params, including empty-field preservation.
- Added gate-backed ASCII `str_upper(text)` support across the full self-host
  and native direct paths, including Map/List string reads and
  `Map<Str,Str>.get_opt` match payload transforms.
- Added gate-backed `str_ends_with(text, suffix)` support across the full
  self-host and native direct paths, including normalized strings, Map/List
  string reads, and `Map<Str,Str>.get_opt` match values.
- Added gate-backed `str_replace(text, needle, replacement)` support across the
  full self-host and native direct paths for all-occurrence string rewriting
  over literals, Map/List string reads, and `Map<Str,Str>.get_opt` match values.
- Raised the self-host `List<Token>` retarget capacity to keep the enlarged
  `fixpoint_full.vais` stage1/stage2 bootstrap path green.
- Tightened native front keyword diagnostics so identifiers containing
  `match` or `enum` are not mistaken for unsupported syntax.

## v1.0.1 - 2026-06-26

Current stable Vais source release.

### Changed

- Promoted the gate-backed `v0.3.2` release-candidate surface to the first
  current stable release line without changing the verified language surface.
- Published current docs and site copy for the stable release, including
  standalone archive links for Linux x64, macOS arm64, and macOS x64.
- Kept the historical public `v1.0.0` tag archived and unmoved; `v1.0.1` is
  the current stable tag for this mainline.

## v0.3.2 - 2026-06-26

Previous Vais source release candidate.

### Changed

- Extended the Vais-authored package manifest checker contract to reject
  missing manifest source directories, matching the native compiler diagnostic
  while keeping package discovery in the driver boundary.
- Extended the same manifest checker contract to reject local dependency cycles
  using normalized local manifest paths.
- Added optional entry-path source-root containment checking to the
  Vais-authored package manifest checker contract.
- Added a Vais-authored local import graph contract checker and release gate for
  manifest-free missing import, duplicate top-level symbol, and import cycle
  diagnostics.
- Extended that import graph checker to follow the first package manifest local
  dependency alias and dependency-internal plain imports.
- Extended that import graph checker to follow all declared entry-package local
  dependency aliases.
- Wired `scripts/vaisc` to run cached Vais-authored package manifest and import
  graph preflight tools before native `emit-ir`, `build`, and `run`.
- Closed the Phase 5 self-host expansion checklist after release gates confirmed
  regenerated core, preflight, import graph, and self-host paths remain green.
- Froze the v1-candidate language and prelude reference docs around the current
  gate-backed surface.
- Promoted collection for-each over integer values through the full self-host
  path, native direct engine, parity, and value gates, with
  `examples/e25_for_filter_sum.vais`, `examples/e27_list_max.vais`, and
  `examples/fr2.vais` added to the release corpus, raising it to 103
  native-supported examples. The full self-host path now also lowers typed
  non-empty local `List<Int>` literals for `List<Int>` parameter calls and
  for-each iteration, plus inline integer list literals passed directly to
  `List<Int>` parameters.
- Promoted `examples/e82_list_literal_direct_arg.vais` as the release-corpus
  example for inline `List<Int>` literal call arguments, raising the corpus to
  104 native-supported examples.
- Promoted `examples/e63_generic_struct_def.vais` as the release-corpus example
  for generic marker syntax on a simple struct used with `Int` values, raising
  the corpus to 105 native-supported examples.
- Promoted struct helper parameter and return values through the public front,
  full self-host compiler, native direct engine, parity, and value gates, with
  `examples/e17_struct_return.vais`, `examples/e28_struct_rebuild.vais`,
  `examples/e37_struct_area.vais`, `examples/e41_recursion_struct.vais`,
  `examples/e54_inventory.vais`, and `examples/e62_struct_multi_return.vais`
  added to the release corpus, raising it to 111 native-supported examples.
  The full self-host path now also lowers assignment from struct-returning calls
  into existing struct locals.
- Promoted `examples/module_basic/main.vais`,
  `examples/package_basic/src/main.vais`, and
  `examples/dependency_basic/app/src/main.vais` into the release corpus as the
  public local import, package source-root, and local dependency package smokes,
  raising it to 114 native-supported examples.
- Promoted single-field nested struct literal/read/write lowering through the
  full self-host compiler and release corpus, with
  `examples/e01_nested_struct.vais` and
  `examples/e32_nested_field_mut.vais` added to the corpus, raising it to 116
  native-supported examples.
- Promoted `examples/d3run.vais`, `examples/d4b.vais`, and
  `examples/e08_option_chain.vais` into the release corpus as additional
  Result propagation, inline `List<Int>` parameter iteration, and direct
  `Option<Int>` match smokes, raising it to 119 native-supported examples.
- Promoted borrowed `&List<Int>` helper parameters through the public front,
  parity manifest, and value corpus, with `examples/e15_list_recursion.vais`
  and `examples/e68_binary_search.vais` added to the corpus, raising it to 121
  native-supported examples.
- Promoted public struct/function modifiers through the checker, public front,
  full self-host compiler, parity manifest, and value corpus, with
  `examples/d5run.vais` added to the corpus, raising it to 122
  native-supported examples. Struct literal lowering now stores `Str` fields
  through the same pointer-to-integer representation used by verified string-key
  collections.
- Promoted the already-supported `examples/t2.vais`, `examples/t3.vais`, and
  `examples/t5.vais` smoke files into the release corpus, covering
  payload-free enum dispatch, bitwise-not negative results, and `Option<Int>`
  matching, raising it to 125 native-supported examples.
- Promoted multiline `Option<Int>` expression-match binding through the public
  compiler driver, front fixture, parity manifest, and value corpus, with
  `examples/d2.vais` added to the release corpus, raising it to 126
  native-supported examples.
- Promoted `Str(Int)` decimal conversion through the public compiler driver,
  front fixture, full self-host compiler, native direct engine, parity manifest,
  and value corpus, with `examples/e73_int_to_string.vais` added to the release
  corpus, raising it to 127 native-supported examples.
- Promoted generic identity helpers applied directly to struct literals through
  the public compiler driver, front fixture, parity manifest, and value corpus,
  with `examples/e46_generic_struct.vais` added to the release corpus, raising
  it to 128 native-supported examples.
- Promoted `examples/e51_index_ast.vais` as a 20-field flat-struct recursive
  AST evaluation example through the full self-host compiler, parity manifest,
  and value corpus, raising the release corpus to 129 native-supported
  examples.
- Promoted `examples/e59_tuple.vais` as the first `Int` tuple return and local
  destructuring slice through public driver lowering, front contract, parity
  manifest, and value corpus, raising the release corpus to 130
  native-supported examples.
- Promoted `examples/e81_closure_return_apply.vais` as a returned single-`Int`
  closure passed to an `Int` higher-order helper through public driver lowering,
  front contract, parity manifest, and value corpus, raising the release corpus
  to 131 native-supported examples.
- Promoted `examples/e09_struct_method.vais` as the first simple `impl` struct
  method return-chain slice through public driver lowering, front contract,
  parity manifest, and value corpus, raising the release corpus to 132
  native-supported examples.
- Promoted `examples/e49_closure_arg.vais` as the first non-capturing inline
  closure literal passed directly to a single-closure `Int` higher-order helper,
  raising the release corpus to 133 native-supported examples.
- Promoted `examples/c5.vais` as the first local single-capture `Int` closure
  call slice, raising the release corpus to 134 native-supported examples.
- Promoted `examples/e78_trait_impl_for.vais` as the first simple `trait` plus
  `impl Trait for Struct` method call slice, raising the release corpus to 135
  native-supported examples.
- Promoted `examples/e76_list_map.vais` and `examples/d6run.vais` as
  non-capturing `List<Int>` map and filter-sum method slices, raising the
  release corpus to 137 native-supported examples.
- Promoted `examples/e77_nested_list.vais` as the first local
  `List<List<Int>>` literal double-index slice, raising the release corpus to
  138 native-supported examples.
- Promoted `examples/e79_nested_match.vais` as the first enum `Option<Int>`
  payload with a nested Option match arm, raising the release corpus to 139
  native-supported examples.
- Promoted `examples/e119_map_param_target_assignment.vais` to cover
  parameter-target assignment copies for every verified concrete Map type,
  raising the release corpus to 140 native-supported examples and adding a
  matching full self-host codegen regression case.
- Promoted `examples/e120_enum_payload_wildcard.vais` as the first payload
  enum `match` with `_` catch-all slice through the public front, parity
  manifest, and value corpus, raising the release corpus to 141
  native-supported examples.
- Promoted `examples/t4.vais` and `examples/t6.vais` as simple struct smoke
  examples, raising the release corpus to 100 native-supported examples.
- Promoted `examples/fr1.vais` as an inclusive range for-loop summation smoke,
  raising the release corpus to 98 native-supported examples.
- Promoted print interpolation and `putchar` output calls through native
  direct, parity, and value gates, with
  `examples/e19_interpolation_print.vais` added to the release corpus.
- Promoted explicit `Bool` locals, helper parameters, helper returns, and unary
  `not` through full self-host, native direct, parity, and value gates, with
  `examples/e88_bool_type.vais` added to the release corpus.
- Promoted explicit `Str` locals, helper parameters, helper returns,
  reassignment, length, index, and equality through full self-host, native
  direct, parity, and value gates, with `examples/e89_str_type.vais` added to
  the release corpus.
- Promoted `Str` substring search with computed byte indexes through full
  self-host, native direct, parity, and value gates, with
  `examples/e71_string_index_of.vais` added to the release corpus.
- Promoted two-pointer `Str` scans with computed byte indexes through full
  self-host, native direct, parity, and value gates, with
  `examples/e69_palindrome_string.vais` added to the release corpus.
- Promoted 12 additional control-flow, Bool, integer-list, and `Str` scanner
  examples through parity and value gates, raising the release corpus to 96
  native-supported examples.
- Promoted simple enum expression-arm `match` lowering for multi-field `Int`
  payload variants, with `examples/e02_enum_payload.vais` added to the release
  corpus.
- Promoted payload-free enum values stored in simple struct fields and matched
  through field access, with `examples/e24_struct_enum_field.vais` added to the
  release corpus.
- Added `vais-check` guidance for Rust-style top-level `use` imports and `pub`
  visibility so they fail with Vais replacements instead of blending into later
  parser errors.
- Promoted single-field struct payload enum lowering with payload field access,
  with `examples/e64_enum_struct_payload.vais` added to the release corpus.
- Promoted Int `match` literal arms with `_` catch-all lowering, with
  `examples/e55_match_wildcard.vais` added to the release corpus.
- Promoted payload-free enum `match` with `_` catch-all, with
  `examples/e90_enum_wildcard.vais` added to the release corpus.
- Promoted the first `Option<Int>` slice for `Some(Int)`/`None`, helper returns,
  struct/local storage, and statement-form `match`, with
  `examples/e16_option_match.vais` and `examples/e40_option_in_struct.vais`
  added to the release corpus.
- Promoted the first `Result<Int,Int>` slice for `Ok(Int)`/`Err(Int)`, helper
  returns, and statement-form `match`, with `examples/e21_result_match.vais`
  added to the release corpus.
- Promoted `Option<Int>` expression-form match binding, with
  `examples/e23_option_flow.vais` added to the release corpus.
- Promoted `Result<Int,Int>` expression-form match binding, with
  `examples/e91_result_flow.vais` added to the release corpus.
- Promoted `Option<Int>` local-binding `?` propagation for success and `None`
  paths, with `examples/e93_option_question.vais` added to the release corpus.
- Promoted `Result<Int,Int>` local-binding `?` propagation, with
  `examples/e39_error_propagate.vais` added to the release corpus.
- Promoted local `Map<Int,Int>.get_opt(key) -> Option<Int>` on the full compiler
  path and native direct engine, with `examples/e94_map_get_opt.vais` added to
  the release corpus.
- Added `examples/e92_result_question_success.vais` to cover the
  `Result<Int,Int>` `?` success path in the release corpus.
- Added front diagnostics for `Option`/`Result` generic forms beyond the
  verified `Option<Int>` and `Result<Int,Int>` slices.
- Added explicit front diagnostics for unverified `Map<Int,Int>` function
  parameters and return values, keeping Map ABI claims behind future gates.
- Added explicit front and direct diagnostics for unverified `Map<Int,Int>`
  value assignment, keeping Map storage and ABI behavior behind future gates.
- Added `docs/design/MAP_ABI.md` to specify future Map parameter, return, and
  concrete generic-expansion rules before broader `Map<K,V>` gates.
- Promoted local `Map<Int,Int>` assignment copy through the full compiler path
  and native direct engine, with `examples/e95_map_assignment.vais` added to
  the release corpus.
- Promoted local `Map<Int,Bool>` construction, assignment copy, `insert`,
  `get(key, default)`, `contains`, and `len` through the full compiler path and
  native direct engine, with `examples/e96_map_bool.vais` added to the release
  corpus.
- Promoted local `Map<Int,Char>` construction, assignment copy, `insert`,
  `get(key, default)`, `contains`, and `len` through the full compiler path and
  native direct engine, with `examples/e97_map_char.vais` added to the release
  corpus.
- Promoted `Map<Int,Int>` function parameters by reference through the full
  compiler path and native direct engine, with `examples/e98_map_param.vais`
  added to the release corpus. Non-promoted Map returns and broader Map
  parameters remain behind future ABI gates.
- Promoted `Map<Int,Bool>` function parameters by reference through the full
  compiler path and native direct engine, with
  `examples/e99_map_bool_param.vais` added to the release corpus.
  Broader Map parameters and non-promoted Map returns remain behind future ABI
  gates.
- Promoted `Map<Int,Char>` function parameters by reference through the full
  compiler path and native direct engine, with
  `examples/e100_map_char_param.vais` added to the release corpus.
  Non-promoted Map returns and generic Map parameters remain behind future ABI
  gates.
- Promoted `Map<Int,Int>` return values for explicitly annotated local
  initialization through the full compiler path and native direct engine, with
  `examples/e101_map_return.vais` added to the release corpus.
- Promoted `Map<Int,Bool>` return values for explicitly annotated local
  initialization through the full compiler path and native direct engine, with
  `examples/e102_map_bool_return.vais` added to the release corpus.
- Promoted `Map<Int,Char>` return values for explicitly annotated local
  initialization through the full compiler path and native direct engine, with
  `examples/e103_map_char_return.vais` added to the release corpus.
  Generic Map returns remain gated.
- Promoted `remove(key)` for the concrete `Map<Int,Int>`, `Map<Int,Bool>`, and
  `Map<Int,Char>` slices through the full compiler path and native direct
  engine, with `examples/e104_map_remove.vais` added to the release corpus.
- Promoted `get_opt(key)` match payloads for the concrete `Map<Int,Bool>` and
  `Map<Int,Char>` slices through the full compiler path and native direct
  engine, with `examples/e105_map_scalar_get_opt.vais` added to the release
  corpus.
- Promoted `clear()` for the concrete `Map<Int,Int>`, `Map<Int,Bool>`, and
  `Map<Int,Char>` slices through the full compiler path and native direct
  engine, with `examples/e106_map_clear.vais` added to the release corpus.
- Promoted `Map<Str,Int>` construction, assignment copy, `insert`,
  `remove`, `clear`, `get(key, default)`, `get_opt(key)`, `contains`, and `len`
  through the full compiler path and native direct engine, with
  `examples/e107_map_str_int.vais` added to the release corpus.
- Promoted `Map<Str,Int>` function parameters by reference through the full
  compiler path and native direct engine, with
  `examples/e108_map_str_int_param.vais` added to the release corpus.
- Promoted `Map<Str,Int>` return values for explicitly annotated local
  initialization through the full compiler path and native direct engine, with
  `examples/e109_map_str_int_return.vais` added to the release corpus.
  Broader `Map<Str,V>` and generic Map returns remain gated.
- Promoted local `Map<Str,Bool>` string-key operations through the full compiler
  path and native direct engine, with `examples/e110_map_str_bool.vais` added to
  the release corpus.
- Promoted `Map<Str,Bool>` function parameters by reference through the full
  compiler path and native direct engine, with
  `examples/e111_map_str_bool_param.vais` added to the release corpus.
- Promoted `Map<Str,Bool>` return values for explicitly annotated local
  initialization through the full compiler path and native direct engine, with
  `examples/e112_map_str_bool_return.vais` added to the release corpus.
- Promoted local `Map<Str,Char>` string-key operations through the full
  compiler path and native direct engine, with
  `examples/e113_map_str_char.vais` added to the release corpus.
- Promoted `Map<Str,Char>` function parameters by reference through the full
  compiler path and native direct engine, with
  `examples/e114_map_str_char_param.vais` added to the release corpus.
- Promoted `Map<Str,Char>` return values for explicitly annotated local
  initialization through the full compiler path and native direct engine, with
  `examples/e115_map_str_char_return.vais` added to the release corpus.
- Promoted concrete Map parameter-source assignment copies through the full
  compiler path and native direct engine, with
  `examples/e116_map_param_assignment.vais` added to the release corpus.
- Promoted concrete Map-returning call assignment copies through the full
  compiler path and native direct engine, with
  `examples/e117_map_return_assignment.vais` added to the release corpus.
- Added argument-bearing Map-returning call assignment coverage through the full
  compiler path and native direct engine, with
  `examples/e118_map_return_assignment_args.vais` added to the release corpus.

## v0.3.1 - 2026-06-20

Previous Vais source release.

### Fixed

- Fixed self-host `print`/`puts` lowering for string-expression arguments so
  generated LLVM IR calls `i32 @puts(i8*)` consistently across release archive
  builders.

## v0.3.0 - 2026-06-20

Previous Vais source release.

### Changed

- Added verified `List<T>.is_empty()` support in the full self-host compiler and
  native direct engine, with front, direct, error, full, and release-gate
  coverage.
- Added verified `List<T>.last()` support for non-empty lists in the full
  self-host compiler and native direct engine, including struct-list local
  binding coverage.
- Added verified `List<T>.pop()` support for non-empty lists in the full
  self-host compiler and native direct engine, including caller-visible length
  mutation for list parameters.
- Defined verified runtime trap behavior for invalid `List` access: negative or
  out-of-range index operations, `last()` on an empty list, and `pop()` on an
  empty list.
- Promoted the first `Str` tool-helper slice: public front contracts now accept
  `Bool` and `Str` helper signatures, native direct mode lowers string
  literals, `s.len()`, `s[i]`, and `Str` equality/inequality, and parity now
  covers string indexing, user-defined integer parsing, and identifier scanning.
- Promoted single-byte `Char` literal equality plus explicit `Char` locals,
  helper parameters, and helper returns through public front, native direct,
  full self-host, and parity gates as Int-compatible scalar values, with
  `examples/e85_char_type.vais` added to the release corpus.
- Promoted range `for` loops with exclusive `..` and inclusive `..=` bounds
  through public front, native direct, full self-host, and parity gates, with
  `examples/e86_for_loop.vais` added to the release corpus.
- Promoted `break` and `continue` inside `while` and range `for` loops through
  public front, native direct, full self-host, and parity gates, with
  `examples/e87_break_continue.vais` added to the release corpus.
- Promoted named integer parsing prelude helpers: `parse_uint(s)` and
  `parse_int(s)` now lower through the full self-host compiler and native direct
  engine, with front, direct, parity, value, and self-host gate coverage.
- Added verified local `Map<Int,Int>` support across the full self-host compiler
  and native direct engine with `{}`, `insert`, `get(key, default)`, `contains`,
  and `len`; front diagnostics still reject Map parameters, return values,
  assignment, and generic key/value forms.
- Added release-corpus examples for local `Map<Int,Int>` and `List<T>`
  `is_empty()`, `last()`, and `pop()` so promoted prelude APIs have value-test
  coverage.
- Specified the Phase 2 module/package/import model and added public front
  diagnostics for reserved `module` and `package` declarations.
- Added the first full-engine local import implementation for single-package
  multi-file builds, including missing-import, duplicate-symbol, and
  import-cycle diagnostics.
- Added source-root `vais.toml` package manifest support for the full engine,
  with required `name`, `version`, and `source` keys plus manifest diagnostics.
- Added local dependency package paths in `vais.toml` `[dependencies]` for the
  full engine, including native gates for dependency imports and manifest
  diagnostics.
- Specified the Phase 3 host file/path/process API plan for future
  checker/tool port work, with the APIs marked as specified rather than
  verified.
- Added the first verified Phase 3 host file intrinsics, `fs_exists(path: Str)
  -> Bool`, `fs_read_text(path: Str) -> Str`,
  `fs_write_text(path: Str, text: Str) -> Int`, and `fs_mkdirs(path: Str) ->
  Int`, for full-engine `scripts/vaisc build` and `scripts/vaisc run`, with a
  native-driver runtime and `scripts/test-vaisc-host.sh` coverage.
- Added verified Phase 3 path helpers, `fs_cwd() -> Str`,
  `fs_temp_dir() -> Str`, `path_join(base: Str, child: Str) -> Str`,
  `path_basename(path: Str) -> Str`, and `path_dirname(path: Str) -> Str`, with
  native-driver runtime support and host gate coverage.
- Added the first verified Phase 3 process intrinsic,
  `proc_run(argv: List<Str>) -> Int`, plus full-engine `List<Str>` local
  `push` support for argv construction, with native-driver runtime support and
  host gate coverage.
- Added verified `proc_capture_stdout(argv: List<Str>) -> Str` for full-engine
  builds and runs, giving Vais-authored repository tools access to child
  process stdout without shell-string APIs.
- Added verified `proc_capture_stderr(argv: List<Str>) -> Str` for full-engine
  builds and runs, giving Vais-authored diagnostics tools access to child
  process stderr without shell-string APIs.
- Added verified `proc_run_env(argv: List<Str>, env: List<Str>) -> Int` for
  child-process environment overrides, and moved the direct-engine no-Python
  PATH check into a Vais-authored harness.
- Added verified `proc_capture_to(argv: List<Str>, stdout_path: Str,
  stderr_path: Str) -> Int` for status-sensitive process checks that need
  captured output files without a struct-returning host ABI.
- Added verified `fs_remove(path: Str) -> Int` and moved standalone uninstall
  option parsing plus binary removal into `tools/uninstall_vaisc.vais`.
- Added `tools/vaisc_install_check.vais` and moved standalone install/package
  verification assertions out of `scripts/test-vaisc-install.sh`.
- Moved the direct-engine arithmetic/build/run smoke checks into
  `tools/vaisc_direct_smoke_check.vais`, further reducing the NV-C2 shell
  fixture.
- Moved the direct-engine import handling and List bounds trap checks into
  `tools/vaisc_direct_error_check.vais`, using `proc_capture_to` to keep status
  and stderr/trap output handling in Vais code.
- Moved the direct helper/control-flow, range `for`, struct-local, and struct
  ABI success fixtures into `tools/vaisc_direct_feature_check.vais`.
- Expanded `tools/vaisc_direct_feature_check.vais` with the direct local
  `List<Int>`, `Str`, `Char`, `parse_uint`/`parse_int`, local `Map<Int,Int>`,
  and local `List<Struct>` success fixtures, further shrinking the NV-C2 shell
  wrapper.
- Moved the remaining direct List ABI, list assignment, and returned-list
  argument hoist fixtures into `tools/vaisc_direct_feature_check.vais`, leaving
  `scripts/test-vaisc-direct.sh` as a thin bootstrap wrapper around
  Vais-authored direct validators.
- Added `tools/vaisc_direct_gate.vais`, so the NV-C2 direct-emitter gate
  orchestration now runs from Vais and the shell entrypoint only supplies the
  temp-dir bootstrap boundary.
- Reduced the remaining single-tool focused shell wrappers to call their
  Vais-authored gates through `scripts/vaisc run`, keeping shell only for
  temporary directories and bootstrap arguments.
- Added `tools/normalize_stage_ir_check.vais` and moved the stage IR
  normalizer sample/expected fixture plus replacement-shape checks out of
  `scripts/test-normalize-stage-ir-vais.sh`.
- Added `tools/embed_self_source_check.vais` and moved the self-source embed
  focused gate fixture generation, generated compiler build/run checks, and
  result assertions out of `scripts/test-embed-self-source-vais.sh`.
- Added `tools/vais_check_contract_check.vais` and moved checker output-count,
  diagnostic-pattern, path, help, and public-wrapper assertions out of
  `scripts/test-vais-check-vais.sh`.
- Added `tools/fixpoint_tier_check.vais` and moved the short `fixpoint.vais`
  and `fixpoint2.vais` tier fixture lists, raw-call embedding, generated
  compiler builds, clang IR checks, and result assertions out of their shell
  gates.
- Added `tools/fixpoint_full_self_check.vais` and moved the long full-source
  self-host gate orchestration out of `scripts/test-fixpoint-full-self.sh`,
  including compiler retargeting, generated compiler build/run checks, final
  binary assertions, and normalized stage comparison.
- Added `tools/fixpoint_full_codegen_check.vais` and moved the long
  full-codegen regression runner out of `scripts/test-fixpoint-full.sh`,
  including compact fixture embedding, trap/stdout cases, source-file checks,
  and emitted-IR shape assertions.
- Audited the remaining shell/host boundary after the full-codegen port:
  native C bootstrap, public command cache wrappers, release/CI orchestration,
  website build tooling, tar/install/clang system tools, and temp-dir wrappers
  remain explicit.
- Fixed native front-contract scanning so unsupported-syntax probes ignore text
  inside string, raw-string, character literals, and comments instead of
  reporting diagnostics for fixture-generator text.
- Added the first Vais-authored checker slice in `tools/vais_check_core.vais`,
  with fixture-based contract checks through `scripts/test-vais-check-vais.sh`
  and the release gate.
- Expanded the Vais checker slice to cover the main non-Vais spelling
  diagnostics in the public fixture catalog, with fixture issue counts checked
  in the checker contract gate.
- Added path, line, column, and help output to the Vais checker slice, with the
  checker contract gate checking diagnostic shape.
- Added `proc_argc()` and `proc_arg(index)` for arguments passed through
  `scripts/vaisc run -- ...`, plus an argv-backed
  `tools/vais_check_cli.vais` checker entrypoint gated by fixture contracts.
- Extended `proc_argc()` and `proc_arg(index)` to binaries produced by
  `scripts/vaisc build`, so standalone Vais tools receive normal OS argv.
- Promoted the Vais-authored checker to the public `scripts/vais-check`
  command, installed and packaged as standalone `bin/vais-check` alongside
  `bin/vaisc`.
- Moved the checker clean/false-positive catalog into fixture-backed Vais
  checker gates and removed the separate checker unit test from the release
  gate.
- Removed checker oracle use from the checker release gate; `scripts/vais-check`
  is now verified by Vais-authored fixture contract checks.
- Moved invalid import path, invalid `main` signature, missing helper
  return-type, unsupported generic `Option<T>`/`Result<T,E>`, and non-verified
  `Map<K,V>` surface diagnostics into the Vais-authored checker contract while
  keeping the public checker and install/package issue counts aligned.
- Classified the remaining front-contract rejects so native-front-only
  closure/enum/match limits stay out of the public checker, while
  manifest/import graph/source-path diagnostics remain explicit host-boundary
  work.
- Added a Vais-authored package manifest contract gate that covers required
  keys, safe `source` paths, unsupported keys/sections, invalid entries,
  dependency path safety, missing dependency manifests, and duplicate
  key/alias diagnostics.
- Moved the NV-C4 parity manifest gate into `tools/vais_parity_check.vais`, so
  release-corpus manifest parsing and native result comparison run through a
  Vais-authored harness.
- Moved the value-corpus gate behind `scripts/test.sh` into
  `tools/vais_value_check.vais`, so release example build/run/exit-code checks
  are driven by a Vais-authored harness.
- Moved the host file/path/string/process smoke gate into
  `tools/vais_host_check.vais`, so IR-shape checks, build/run checks, argv
  checks, and file-output assertions are driven by a Vais-authored harness.
- Moved the NV-C0 public compiler smoke gate into
  `tools/vaisc_smoke_check.vais`, so `emit-ir`, direct `clang`, `build`, and
  `run` contract checks are driven by a Vais-authored harness.
- Moved the NV-C1 front contract gate into `tools/vaisc_front_check.vais`, so
  accepted/rejected source fixture generation, multi-file package setup, stdout
  checks, and diagnostic-shape checks are driven by a Vais-authored harness.
- Moved the native driver smoke gate into `tools/vaisc_native_check.vais`, so
  native-driver build, version, doctor, emit, build, and run checks are driven
  by a Vais-authored harness after the C build bootstrap.
- Moved the NV-C3 diagnostics gate into `tools/vaisc_errors_check.vais`, so
  compiler diagnostic fixture generation and stderr shape checks are driven by
  a Vais-authored harness.
- Moved the legacy self-host compiler smoke gate into
  `tools/compiler_smoke_check.vais`, so program retargeting, generated compiler
  execution, IR staging, and final binary checks are driven by a Vais-authored
  harness and run from the pre-tag release gate.
- Added verified host-backed string construction helpers `str_concat`,
  `str_slice`, and `str_byte`, with native-driver runtime support plus
  `scripts/test-vaisc-host.sh` coverage.
- Extended full-engine `Str` lowering for reassignment and user-defined
  `-> Str` helper returns, covered by the host smoke gate.
- Extended full self-host lowering for runtime `Str` equality/inequality and
  regenerated the reusable compiler core, allowing Vais-authored tools to use
  idiomatic string comparisons.
- Moved release archive packaging orchestration into
  `tools/package_vaisc_release.vais`; `scripts/package-vaisc-release.sh` now
  delegates option parsing, version/platform detection, binary staging, docs
  staging, and archive creation to a Vais-authored tool.
- Moved standalone install orchestration into `tools/install_vaisc.vais`;
  `scripts/install-vaisc.sh` now delegates option parsing, compiler/checker
  staging, and installation to a Vais-authored tool while preserving existing
  CLI and environment inputs.
- Moved internal self-host helper builds onto the native `scripts/vaisc`
  trust-root path, removed the compiler escape hatch, and verified the embed
  helper, stage normalizer, fixpoint, full codegen, and full self-host gates
  through the native path.

## v0.2.2 - 2026-06-15

Current Vais source release.

### Changed

- Added `scripts/test-release-gates.sh` and
  `docs/release/RELEASE_CHECKLIST.md` as the pre-tag release contract for
  future source releases.
- Added a GitHub Actions release archive workflow for tag builds.
- `scripts/vaisc --engine direct` now stays on the native driver.
- The native direct engine now covers Int helper calls, locals, assignment,
  `if`, `while`, return expressions, and simple Int-field struct local
  literal/read/write plus struct parameter/return helper ABI.
- The native direct engine now covers local `List<Int>` initialization with
  `[]`, `list()`, and small integer list literals, plus `push`, `len`, index,
  and `sum`.
- The native direct engine now accepts `List<Int>` function signatures and
  return values through the direct ABI.
- `List<Int>` direct-engine parameters are now native references for local list
  arguments, so callee `push` operations mutate the caller's list.
- Inline `List<Int>` literals and `list()` now lower in direct-engine call
  arguments and return expressions.
- `List<Int>`-returning helper calls now hoist into direct-engine temporaries
  when passed directly to `List<Int>` parameters in statement contexts.
- Direct-engine `while` conditions now hoist returned-list arguments per
  iteration instead of requiring a local list binding.
- Local `List<Struct>` values now lower through the direct engine for typed
  `[]`, `list()`, list literals, `push`, `len`, index, and field reads.
- `List<Struct>` direct-engine function parameters, return values, inline list
  arguments, and returned-list argument hoisting now use the native list ABI.
- `List<Int>` and `List<Struct>` direct-engine assignment now supports
  context-typed `[]`, `list()`, list literals, local lists, and returned lists.
- `List<Struct>` direct-engine indexed field assignment now supports local and
  parameter writes such as `xs[0].value = 42`.
- `List<Int>` and `List<Struct>` direct-engine element assignment now supports
  local and parameter writes such as `xs[0] = value`.
- `List<Int>` and `List<Struct>` returned-list arguments now lower inside
  direct-engine `if` and `else if` conditions.

### Requirements

- `clang`

### Verification

The release baseline is protected by:

```bash
bash scripts/test-release-gates.sh
```

## v0.2.1 - 2026-06-14

Previous Vais source release.

### Changed

- `scripts/vaisc` now defaults to a native public driver that links the checked-in
  self-host compiler core.
- Normal user `emit-ir`, `build`, `run`, and `doctor` use the native driver.
- Added standalone install, uninstall, package, and native install/package test
  scripts.

### Requirements

- `clang`

### Verification

The release baseline is protected by:

```bash
bash scripts/test-release-gates.sh
```

## v0.2.0 - 2026-06-14

Previous Vais source release.

### Included

- `.vais` is the checked-in source extension.
- `scripts/vaisc` is the public compiler command.
- `scripts/vaisc emit-ir`, `scripts/vaisc build`, and `scripts/vaisc run` compile
  `.vais` files through the self-host compiler core and link with `clang`.
- `compiler/self/fixpoint_full.vais` is the trusted full compiler source.
- `compiler/self/vaisc_core.ll` is the reusable self-host compiler core used by
  `scripts/vaisc`.
- `docs/reference/LANGUAGE.md` is the current gate-backed language guide.
- `website/` is the official `vaislang.dev` source and deploys through GitHub
  Pages Actions.

### Requirements

- `clang`

### Verification

The release baseline is protected by:

```bash
bash scripts/test-release-gates.sh
```
