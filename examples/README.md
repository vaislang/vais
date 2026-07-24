# Vais Examples

This directory contains `.vais` examples. The release value-correctness corpus is
the subset listed as `native-supported` in `tools/vaisc-parity.tsv`.

Each release-corpus file starts with `# expect: N`. `scripts/test.sh` compiles and
runs those files and compares the process exit code with `N % 256`.

Run the release corpus:

```bash
bash scripts/test.sh
```

Run a single example by basename:

```bash
bash scripts/test.sh c4
```

Run the multi-file import example:

```bash
scripts/vaisc run examples/module_basic/main.vais
```

Run the package-manifest example:

```bash
scripts/vaisc run examples/package_basic/src/main.vais
```

Run a package directory directly:

```bash
scripts/vaisc run examples/e323_cli_package
scripts/vaisc run examples/e323_cli_package --engine direct -- vaisdb cache
scripts/vaisc package examples/e323_cli_package -o /tmp/e323-dist
scripts/vaisc package examples/e326_cli_binary_target -o /tmp/e326-dist
scripts/vaisc package examples/e326_cli_binary_target -o /tmp/e326-dist --archive
scripts/vaisc package examples/e328_cli_package_assets -o /tmp/e328-dist --archive
```

Run the local dependency package example:

```bash
scripts/vaisc run examples/dependency_basic/app/src/main.vais
```

These module, package, and dependency examples are part of the release
value-correctness corpus.

Compiler parity coverage is tracked in `tools/vaisc-parity.tsv`:

```bash
bash scripts/test-vaisc-parity.sh
```

Representative promoted examples include `examples/e02_enum_payload.vais` for
multi-field `Int` payload enum expression-arm match lowering,
`examples/e24_struct_enum_field.vais` for payload-free enum values stored in
struct fields and matched through field access,
`examples/e17_struct_return.vais`, `examples/e28_struct_rebuild.vais`,
`examples/e37_struct_area.vais`, `examples/e41_recursion_struct.vais`,
`examples/e54_inventory.vais`, and `examples/e62_struct_multi_return.vais` for
struct helper parameters, returns, reassignment, recursion, and aggregation,
`examples/e09_struct_method.vais` for simple `impl` methods lowered to helper
functions through a return-chain,
`examples/e78_trait_impl_for.vais` for a simple `trait` plus
`impl Trait for Struct` method call lowered to a helper function,
`examples/d5run.vais` for public struct/function modifiers with a `Str` field,
`examples/e63_generic_struct_def.vais` for generic marker syntax on a simple
struct used with `Int` values,
`examples/e46_generic_struct.vais` for a generic identity helper applied
directly to a struct literal,
`examples/e51_index_ast.vais` for a flat index AST encoded in a 20-field struct
with recursive evaluation,
`examples/e59_tuple.vais` for `Int` tuple return and local destructuring,
`examples/e64_enum_struct_payload.vais` for a single-field struct payload enum
matched through payload field access,
`examples/e79_nested_match.vais` for an enum carrying a single `Option<Int>`
payload and matching that payload through a nested Option match arm,
`examples/e55_match_wildcard.vais` for Int match literal arms with a `_`
catch-all,
`examples/e90_enum_wildcard.vais` for payload-free enum match with a `_`
catch-all,
`examples/e120_enum_payload_wildcard.vais` for payload enum match with a `_`
catch-all,
`examples/e80_closure_return.vais` and
`examples/e81_closure_return_apply.vais` for the verified single-`Int` closure
return and returned-closure higher-order helper slices,
`examples/e49_closure_arg.vais` for a non-capturing inline closure literal
passed to the same single-closure `Int` helper shape,
`examples/c5.vais` for a local closure with one `Int` capture called in the
same function,
`examples/e76_list_map.vais`, `examples/e203_list_filter_result.vais`,
`examples/e204_list_str_map.vais`, `examples/e205_list_str_filter.vais`,
`examples/e206_list_str_filter_infer.vais`,
`examples/e207_list_str_param_map_filter.vais`,
`examples/e208_list_str_map_concat.vais`,
`examples/e209_list_str_closure_capture.vais`,
`examples/e263_list_str_filter_map_result_contexts.vais`,
`examples/e264_list_str_map_filter_result_contexts.vais`,
`examples/e265_list_str_map_filter_scalar_contexts.vais`,
`examples/e266_list_str_filter_map_scalar_contexts.vais`,
`examples/e267_list_str_pipeline_scalar_multi_expr.vais`,
`examples/e268_list_str_pipeline_scalar_mixed_expr.vais`,
`examples/e269_list_str_pipeline_scalar_bool_infer.vais`,
`examples/e270_list_str_pipeline_scalar_reassign_arithmetic_tail.vais`,
`examples/e271_list_str_pipeline_scalar_bool_negation.vais`,
`examples/e272_list_str_pipeline_scalar_bool_if_expr.vais`,
`examples/e273_list_str_pipeline_scalar_bool_if_expr_call_return.vais`,
`examples/e274_list_str_pipeline_scalar_bool_if_expr_nested_call_reassign.vais`,
`examples/e275_list_str_pipeline_scalar_int_if_expr.vais`,
`examples/e276_scalar_value_if_expr_embedded_call_args.vais`,
`examples/e277_scalar_bool_value_if_expr.vais`,
`examples/e278_scalar_str_value_if_expr.vais`,
`examples/e279_scalar_char_value_if_expr.vais`,
`examples/e280_map_str_str_get_opt_match_contexts.vais`,
`examples/e281_map_str_str_return_infer_get_opt_match_contexts.vais`,
`examples/e282_map_str_str_get_opt_match_str_transforms.vais`,
`examples/e283_str_len_reassigned_match_transform.vais`,
`examples/e284_map_str_str_get_opt_match_transform_len.vais`,
`examples/e285_map_str_str_get_opt_str_payload_stability.vais`,
`examples/e286_map_str_str_get_opt_condition_chains.vais`,
`examples/e210_list_int_closure_capture.vais`,
`examples/e211_list_filter_sum_assignment.vais`,
`examples/e212_list_filter_len_count.vais`,
`examples/e213_list_struct_filter_result.vais`,
`examples/e214_list_struct_map_projection.vais`,
`examples/e245_list_struct_map_projection_direct_contexts.vais`,
`examples/e246_list_struct_map_projection_call_arg_conditions.vais`,
`examples/e247_list_struct_map_projection_aggregates.vais`,
`examples/e248_list_struct_map_projection_aggregate_conditions.vais`,
`examples/e249_list_struct_map_projection_aggregate_call_args.vais`,
`examples/e250_list_struct_map_projection_aggregate_arithmetic_tail.vais`,
`examples/e251_list_struct_map_projection_aggregate_call_arg_arithmetic_tail.vais`,
`examples/e252_list_struct_map_projection_aggregate_mutation_args.vais`,
`examples/e253_list_struct_map_projection_aggregate_reassign.vais`,
`examples/e254_list_struct_map_projection_aggregate_embedded_expr.vais`,
`examples/e255_list_struct_map_projection_aggregate_embedded_conditions.vais`,
`examples/e215_list_struct_filter_len_count.vais`,
`examples/e216_list_struct_filter_map_sum.vais`,
`examples/e217_list_int_max.vais`,
`examples/e218_list_int_min.vais`,
`examples/e219_list_filter_max_min.vais`,
`examples/e220_list_struct_filter_map_max_min.vais`,
`examples/e256_list_struct_filter_map_aggregate_call_args.vais`,
`examples/e257_list_struct_filter_map_aggregate_call_arg_conditions.vais`,
`examples/e258_list_struct_filter_map_aggregate_embedded_expr.vais`,
`examples/e259_list_struct_filter_map_aggregate_embedded_conditions.vais`,
`examples/e239_list_struct_filter_map_result_chain.vais`,
`examples/e240_list_struct_filter_map_return_chain.vais`,
`examples/e241_list_struct_filter_map_call_arg.vais`,
`examples/e242_list_struct_filter_map_call_arg_conditions.vais`,
`examples/e243_list_struct_filter_map_extend_arg.vais`,
`examples/e244_list_struct_filter_map_reassign.vais`,
`examples/e221_list_filter_map_max_min.vais`,
`examples/e222_list_filter_map_sum.vais`,
`examples/e260_list_filter_map_aggregate_embedded_expr.vais`,
`examples/e261_list_filter_map_aggregate_embedded_conditions.vais`,
`examples/e262_list_map_aggregate_embedded_expr_conditions.vais`,
`examples/e223_list_struct_filter_first_last_field.vais`,
`examples/e224_list_struct_filter_first_last_field_len.vais`,
`examples/e225_list_struct_filter_first_last_value.vais`,
`examples/e226_list_struct_filter_first_last_multiline_value.vais`,
`examples/e227_list_struct_filter_first_last_push_insert.vais`,
`examples/e228_list_struct_filter_first_last_field_push_insert.vais`,
`examples/e229_list_struct_filter_first_last_field_infer.vais`,
`examples/e230_list_struct_filter_first_last_field_call_arg.vais`,
`examples/e231_list_struct_filter_first_last_value_call_arg.vais`,
`examples/e232_list_struct_filter_first_last_late_helper_call_arg.vais`,
`examples/e233_list_struct_filter_first_last_call_arg_expr_tail.vais`,
`examples/e234_list_struct_filter_first_last_call_arg_if_condition.vais`,
`examples/e235_list_struct_filter_first_last_call_arg_while_condition.vais`,
`examples/e236_list_struct_filter_first_last_call_arg_else_if_condition.vais`,
`examples/e237_list_struct_filter_first_last_call_arg_else_if_chain.vais`,
`examples/e238_list_struct_filter_first_last_call_arg_else_if_chain_return.vais`,
and
`examples/d6run.vais` for verified `List<Int>`, `List<Str>`, and
`List<Struct>` map/filter method slices, including known `Str` captures in
`List<Str>` closures, direct `List<Str>.filter(...).map(...)` and
`List<Str>.map(...).filter(...)` result-list contexts, direct
`List<Str>.map(...).filter(...).len/contains/index_of/count` and
`List<Str>.filter(...).map(...).len/contains/index_of/count` scalar contexts,
same-family multiple `List<Str>` pipeline scalar calls inside one expression,
mixed map-filter/filter-map scalar calls inside one expression,
composite Bool local inference for `List<Str>` pipeline scalar conditions,
arithmetic-tail reassignment expressions using `List<Str>` pipeline scalars,
negated `List<Str>` pipeline scalar Bool locals/reassignments/conditions,
known `Int` captures in
`List<Int>` closures, reusable
`List<Int>` filter-sum assignments plus reusable filtered count
assignments/returns, `List<Struct>` record-filter result lists, and
`List<Struct>` field projection into scalar lists plus direct record-filter
count returns/assignments, direct record score aggregation, `List<Int>`
maximum/minimum selection, and filtered `List<Int>` extrema without an
intermediate result list, plus direct `List<Struct>` score max/min projection,
first/last record field projection, string field length reads, and whole-record
first/last selection without an intermediate record list including direct
`push`/`insert_at` arguments and helper-call arguments, and scalar
field/length selections feeding `List<Int>`/`List<Str>` mutations plus
inferred local field selections and helper-call arguments,
`examples/e16_option_match.vais` for the first `Option<Int>` helper-return and
statement-match slice,
`examples/e21_result_match.vais` for the first `Result<Int,Int>`
helper-return and statement-match slice,
`examples/e23_option_flow.vais` for `Option<Int>` expression-match binding,
`examples/d2.vais` for multiline `Option<Int>` expression-match binding,
`examples/e93_option_question.vais` for `Option<Int>` `?` propagation,
`examples/e08_option_chain.vais` for direct `Option<Int>` helper-return
matching from `main`,
`examples/e39_error_propagate.vais` for `Result<Int,Int>` `?` propagation,
`examples/e91_result_flow.vais` for `Result<Int,Int>` expression-match binding,
`examples/e92_result_question_success.vais` for the `Result<Int,Int>` `?`
success path,
`examples/e294_result_try_parse_error_flow.vais` for a direct/full
`Result<Int,Int>` parse/error flow using `?` and inline `match`,
`examples/e296_result_map_param_flow.vais` for a `Result<Int,Int>` helper chain
over `Map<Str,Str>` metadata parameters, using `Map<Str,Str>.get_opt` match in
a helper and local-binding `?` propagation,
`examples/e295_vaisdb_indexer_prototype.vais` for a Vais-authored document
indexer prototype that ingests two documents, persists metadata through a
`Map<Str,Str>` line snapshot, indexes `Map<Str,Int>` term counts, and runs a
weighted query score,
`examples/e297_vaisdb_file_ingest_workflow.vais` for the file-backed follow-up
that reads document/query files, can create deterministic temp inputs, accepts
argv-supplied paths, and runs the same snapshot/index/query workflow,
`examples/e298_vaisdb_file_ingest_result_flow.vais` for the `fs_exists`
guarded follow-up that returns `Result<Int,Int>` error codes for missing or
malformed file-backed inputs,
`examples/e329_result_str_str_error_message.vais` for the first
`Result<Str,Str>` non-Int error payload slice that carries both a `Str` value
and a `Str` error message through helper returns, inline match, and `?`,
`examples/e330_vaisdb_ingest_error_message_flow.vais` for the VaisDB
file-ingest workflow that reports every failure path as a human-readable
`Result<Str,Str>` message instead of an opaque integer code,
`examples/e332_vaisdb_topk_ranking_report.vais` for the top-k ranking report
that orders scored documents with a hand-written `List<Struct>` selection sort
(whole-element swaps through a temporary struct local) and renders the top
lines as a `Result<Str,Str>` report with a blank-query error message,
`examples/e333_vaisdb_snapshot_version_migration.vais` for versioned metadata
snapshots whose loader migrates the v1 key layout, accepts the current
version, and reports missing or unknown versions as `Result<Str,Str>`
messages,
`examples/e334_vaisdb_index_persistence_incremental.vais` for the persisted
term index that reloads from disk, extends incrementally without a rebuild,
scores identically to a fresh build, and reports missing index files as
`Result<Str,Str>` messages,
`examples/e335_list_int_sort.vais` for the built-in `List<Int>.sort()`
statement that sorts in place through local and parameter receivers,
`examples/e336_list_struct_sort_by.vais` for the built-in
`List<Struct>.sort_by(|x| x.int_field)` and `sort_by_desc` key sorts that
order records in place for ranking flows,
`examples/e337_vaisdb_cli_package` for the installable vaisdb CLI package
whose multi-module source builds to `dist/bin/vaisdb` with
ingest/query/report/ingest-dir/rank/docs/remove/stats subcommands, a
deterministic no-argument self-test, and a release archive,
`examples/e338_fs_list_files.vais` for the built-in `fs_list_files(dir, out)`
directory enumeration that fills a `List<Str>` with sorted regular-file names,
`examples/e339_list_struct_field_in_call_args.vais` for `List<Struct>` indexed
field reads inside nested call arguments including `Str(...)` conversions,
`examples/e340_list_str_sort.vais` for `str_cmp` three-way comparison and
in-place `List<Str>.sort()` with element assignment on both engines,
`examples/e341_vaisgrep_package` for the installable vaisgrep CLI package
(substring line search over files, directories, and stdin via `-`, with `-c`
counts and `-r` recursive tree walks) built on the `fs_is_dir(path)` host
dispatch and the `stdin_read_all()` pipeline read,
`examples/e342_fs_list_dirs.vais` for the built-in `fs_list_dirs(dir, out)`
subdirectory enumeration that pairs with `fs_list_files` for tree walks,
`examples/e343_self_recursion_at.vais` for `@(args)` self-recursion in tail,
compound, and nested call-argument positions on both engines,
`examples/e344_vaismake_package` for the installable vaismake task runner
(named tasks from a plain file, whitespace argv, `proc_run`/`-o` capture,
`!env` environment overlays, `!needs` dependency chains with cycle detection),
with `tools/gates.tasks` driving this repository's own gate ladder through it
(`scripts/vaismake-ladder.sh`),
`examples/e345_proc_run_env.vais` for the built-in `proc_run_env(argv, env)`
child-environment overlay on both engines,
`examples/e346_vaisfmt_package` for the installable vaisfmt whitespace
normalizer (`-c` check / in-place fix over recursive `.vais` trees, plus `-`
as a stdin-to-stdout pipe filter) built on the direct-verified
`str_builder_*` chain and the raw `stdout_write` host call,
`examples/e347_list_discard_statements.vais` for bare `remove_at`/`pop`
statements on Int and struct lists on both engines,
`examples/e350_vaisbench_package` for the installable vaisbench command timer
(repeated `proc_run` with `time_millis`, min/median/avg/max reporting,
variable argument passthrough),
`examples/e351_vaisdiff_package` for the installable vaisdiff line comparer
(byte-level trim, middle-block `-N:`/`+N:` reports, one-side stdin),
`examples/e352_str_param_equality.vais` for runtime Str equality between
identifier operands without literal keys,
`examples/e348_nested_list_expr_reads.vais` for composed `List<List<Int>>`
double-index reads (arithmetic, call arguments, dynamic columns),
`examples/e301_result_str_int_file_read.vais` for the `fs_exists` guarded
`Result<Str,Int>` follow-up that propagates file text with `?` and recovers
missing-file error codes through inline match,
`examples/e302_result_str_int_param_flow.vais` for passing `Result<Str,Int>`
values through helper parameters and forwarding them before inline recovery,
`examples/e303_result_metric_int_struct_payload.vais` for passing a
`Result<Metric,Int>` structured payload through helper parameters and
recovering payload fields with inline matches,
`examples/e304_result_record_int_struct_payload.vais` for passing declared
Int-field struct payloads through `Result<DeclaredStruct,Int>` helpers and
recovering multiple fields with inline matches,
`examples/e305_result_multiline_struct_payload.vais` for the same
declared-struct Result flow when the payload struct is declared over multiple
lines,
`examples/e306_result_struct_str_fields.vais` for declared-struct Result
payloads with `Str` fields and inline recovery of string field lengths,
`examples/e307_result_struct_try_payload.vais` for declared-struct Result
payloads extracted with local-binding `?` and reused through fields,
`examples/e308_vaisdb_artifact_record_workflow.vais` for a VaisDB-style
artifact record workflow that builds `Result<DocArtifact,Int>` values, pushes
them into `List<DocArtifact>` outputs, snapshots metadata, and propagates
integer errors,
`examples/e309_vaisdb_artifact_store_snapshot.vais` for persisting
`List<DocArtifact>` values as a text artifact-store snapshot, reloading them
through `Result<DocArtifact,Int>` parsing helpers, and querying the best loaded
record,
`examples/e310_vaisdb_artifact_query_report.vais` for loading persisted
artifact stores into `List<DocArtifact>`, ranking them with `Map<Str,Int>` term
scoring, returning reusable `Result<Str,Int>` report payloads, and preserving
missing-store/empty-query errors,
`examples/e311_result_call_argument_flow.vais` for passing `Result<Str,Int>`
and `Result<DocArtifact,Int>` returning helpers directly as call arguments
without explicit temporary locals,
`examples/e312_result_struct_local_wrapper_flow.vais` for copying
declared-struct Result wrapper payloads through local struct variables and
returning them without losing nested fields,
`examples/e313_result_struct_str_match_flow.vais` for recovering `Str` fields
from declared-struct Result matches while converting `Err(Int)` codes to
strings,
`examples/e314_result_struct_str_concat_match_flow.vais` for composing
declared-struct Result `Str` fields with nested `str_concat(...)` match arms
while converting `Err(Int)` codes to strings,
`examples/e315_result_struct_str_transform_match_flow.vais` for normalizing
declared-struct Result `Str` fields with `str_replace`, `str_trim`,
`str_upper`, `str_lower`, and local-prefix `str_concat(...)` match arms while
converting `Err(Int)` codes to strings,
`examples/e316_result_struct_str_transform_len_match_flow.vais` for scoring
declared-struct Result payloads by applying string transforms in match arms and
using chained `.len()` terms alongside integer fields,
`examples/e317_result_struct_payload_helper_call_score.vais` for passing
declared-struct Result `Ok` payloads to reusable scoring helpers from match
arms while preserving integer error recovery,
`examples/e318_result_struct_payload_helper_call_arithmetic.vais` for composing
reusable declared-struct Result `Ok` payload helper-call terms with normal
payload fields in integer match arms,
`examples/e319_result_struct_payload_field_helper_call_arithmetic.vais` for
passing declared-struct Result `Ok` payload string fields to reusable integer
helpers and composing those helper-call terms with normal payload fields,
`examples/e320_result_struct_payload_int_field_helper_call_arithmetic.vais` for
passing declared-struct Result `Ok` payload integer fields to reusable integer
helpers and composing those helper-call terms with string-field helper terms,
`examples/e321_result_struct_payload_bool_match_condition.vais` for returning
Bool conditions from declared-struct Result payload helper terms and error-code
comparisons,
`examples/e322_vaisdb_module_boundary/main.vais` for splitting reusable
VaisDB-style scoring and artifact helpers across imported modules while sharing
`DocArtifact`, structured Result helpers, List outputs, and Map-backed scoring,
`examples/e323_cli_package` for running a manifest-backed package directory
directly while preserving imports and CLI argv forwarding, plus packaging it as
an argv-capable `dist/bin/e323_cli_package` binary with a copied manifest,
`examples/e326_cli_binary_target` for package manifests that install a command
name from optional `binary` metadata instead of the package name,
`examples/e299_vaisdb_benchmark_report.vais` for a Vais-authored
benchmark/report workflow using `time_millis`, term counts, weighted scoring,
and persisted report validation,
`examples/e300_vaisdb_benchmark_cli_report.vais` for the CLI-style follow-up
that locates the repo, runs the indexer through `proc_capture`, records
direct/default status metrics, and persists the report,
`tools/vaisdb_benchmark_report.vais` for the reusable Vais-authored benchmark
report command that parses saved metrics and writes a direct/default summary,
`scripts/test-vaisdb-workflow.sh` for the focused direct/default workflow gate
covering `examples/e292_*.vais` through the e324 package output workflow plus
the benchmark report tool, and
`scripts/bench-vaisdb-indexer.sh` for the local compile+run timing baseline,
`examples/d3run.vais` for a `Result<Int,Int>` helper propagation chain,
`examples/e40_option_in_struct.vais` for `Option<Int>` stored in a struct field,
`examples/e79_nested_match.vais` for `Option<Int>` carried as an enum payload
and matched in a nested statement arm,
`examples/e120_enum_payload_wildcard.vais` for payload enum match with a `_`
catch-all,
`examples/e19_interpolation_print.vais` for print interpolation and `putchar`
output calls,
`examples/e25_for_filter_sum.vais` for collection for-each over integer values,
`examples/e27_list_max.vais` for `List<Int>` parameter for-each with a running
max,
`examples/e217_list_int_max.vais` for the `List<Int>.max()` method over local
and parameter lists,
`examples/e218_list_int_min.vais` for the `List<Int>.min()` method over local
and parameter lists,
`examples/e219_list_filter_max_min.vais` for filtered `List<Int>` maximum and
minimum selection in helper returns and reusable locals,
`examples/e220_list_struct_filter_map_max_min.vais` for filtered record score
maximum/minimum projection without an intermediate score list,
`examples/e256_list_struct_filter_map_aggregate_call_args.vais` for passing
filtered record score `sum()`/`max()`/`min()` aggregates directly to helpers,
`examples/e257_list_struct_filter_map_aggregate_call_arg_conditions.vais` for
using those aggregate helper-call arguments in `if`, `while`, and `else if`
conditions,
`examples/e258_list_struct_filter_map_aggregate_embedded_expr.vais` for
embedding filtered record score `sum()`/`max()`/`min()` aggregates inside
broader `Int` expressions used by locals, helper-call arguments,
`List<Int>` mutation arguments, reassignments, and returns,
`examples/e259_list_struct_filter_map_aggregate_embedded_conditions.vais` for
using those filtered score aggregates inside broader `if`, `while`, and
`else if` condition expressions,
`examples/e245_list_struct_map_projection_direct_contexts.vais` for direct
returns, helper-call arguments, `extend(...)` sources, and reassignment from
record field projections,
`examples/e246_list_struct_map_projection_call_arg_conditions.vais` for using
those unfiltered record field projections in helper calls that start `if`,
`while`, and `else if` condition expressions,
`examples/e247_list_struct_map_projection_aggregates.vais` for direct
`sum()`/`max()`/`min()` aggregation over unfiltered record `Int` field
projections in returns and reusable locals,
`examples/e248_list_struct_map_projection_aggregate_conditions.vais` for using
those direct unfiltered `Int` field aggregations in `if`, `while`, and
`else if` condition expressions,
`examples/e249_list_struct_map_projection_aggregate_call_args.vais` for using
those direct unfiltered `Int` field aggregations as helper-call arguments in
`return`, `let`, `if`, `while`, and `else if` contexts,
`examples/e250_list_struct_map_projection_aggregate_arithmetic_tail.vais` for
keeping simple arithmetic suffixes on those direct unfiltered `Int` field
aggregations in return expressions and reusable locals,
`examples/e251_list_struct_map_projection_aggregate_call_arg_arithmetic_tail.vais`
for keeping simple arithmetic suffixes when those direct unfiltered `Int` field
aggregations are used as helper-call arguments,
`examples/e252_list_struct_map_projection_aggregate_mutation_args.vais` for
using those direct unfiltered `Int` field aggregations as `List<Int>.push` and
`insert_at` arguments,
`examples/e253_list_struct_map_projection_aggregate_reassign.vais` for
reassigning known `Int` locals and parameters from those direct unfiltered
`Int` field aggregations,
`examples/e254_list_struct_map_projection_aggregate_embedded_expr.vais` for
embedding those direct unfiltered `Int` field aggregations inside broader
`Int` expressions used by locals, helper-call arguments, `List<Int>` mutation
arguments, reassignments, and returns,
`examples/e255_list_struct_map_projection_aggregate_embedded_conditions.vais`
for embedding those direct unfiltered `Int` field aggregations inside broader
`if`, `while`, and `else if` condition expressions,
`examples/e239_list_struct_filter_map_result_chain.vais` for filtered record
field projection into reusable scalar result lists without an intermediate
record list,
`examples/e240_list_struct_filter_map_return_chain.vais` for returning those
filtered projected scalar result lists directly from helpers,
`examples/e241_list_struct_filter_map_call_arg.vais` for passing those
filtered projected scalar result lists directly to helpers,
`examples/e242_list_struct_filter_map_call_arg_conditions.vais` for using
those helper-call arguments in `if`, `while`, and `else if` conditions,
`examples/e243_list_struct_filter_map_extend_arg.vais` for extending
`List<Int>`/`List<Str>` buffers directly from filtered record field projections,
`examples/e244_list_struct_filter_map_reassign.vais` for reassigning existing
`List<Int>`/`List<Str>` buffers from filtered record field projections,
`examples/e221_list_filter_map_max_min.vais` for filtered `List<Int>` score
transform maximum/minimum selection without an intermediate list,
`examples/e222_list_filter_map_sum.vais` for filtered `List<Int>` score
transform aggregation without an intermediate list,
`examples/e260_list_filter_map_aggregate_embedded_expr.vais` for embedding
filtered `List<Int>` transformed `sum()`/`max()`/`min()` aggregates inside
broader `Int` expressions used by locals, helper-call arguments,
`List<Int>` mutation arguments, reassignments, and returns,
`examples/e261_list_filter_map_aggregate_embedded_conditions.vais` for using
those filtered `List<Int>` transformed aggregates inside broader `if`,
`while`, and `else if` condition expressions,
`examples/e262_list_map_aggregate_embedded_expr_conditions.vais` for using
unfiltered `List<Int>.map(...)` transformed aggregates inside broader `Int`
expressions and broader `if`/`while`/`else if` condition expressions,
`examples/e223_list_struct_filter_first_last_field.vais` for filtered
`List<Struct>` first/last record field selection without an intermediate list,
`examples/e224_list_struct_filter_first_last_field_len.vais` for filtered
`List<Struct>` first/last string field length reads without an intermediate
list,
`examples/e225_list_struct_filter_first_last_value.vais` for filtered
`List<Struct>` first/last whole-record selection without an intermediate list,
`examples/e226_list_struct_filter_first_last_multiline_value.vais` for the
same whole-record selection with multiline struct declarations,
`examples/e227_list_struct_filter_first_last_push_insert.vais` for using
filtered first/last whole-record selections directly as `push` and `insert_at`
arguments,
`examples/e228_list_struct_filter_first_last_field_push_insert.vais` for using
filtered first/last field and string-length selections directly as scalar list
mutation arguments,
`examples/e229_list_struct_filter_first_last_field_infer.vais` for inferred
`Int`/`Str` locals from filtered first/last field and string-length selections,
`examples/e230_list_struct_filter_first_last_field_call_arg.vais` for passing
filtered first/last field and string-length selections directly as helper-call
arguments,
`examples/e231_list_struct_filter_first_last_value_call_arg.vais` for passing
filtered first/last whole-record selections directly as struct helper-call
arguments,
`examples/e232_list_struct_filter_first_last_late_helper_call_arg.vais` for
using later-declared helper signatures when lowering filtered first/last
helper-call arguments,
`examples/e233_list_struct_filter_first_last_call_arg_expr_tail.vais` for
preserving simple arithmetic suffixes on helper calls with filtered first/last
arguments,
`examples/e234_list_struct_filter_first_last_call_arg_if_condition.vais` for
using helper calls with filtered first/last arguments at the start of `if`
condition expressions,
`examples/e235_list_struct_filter_first_last_call_arg_while_condition.vais`
for using helper calls with filtered first/last arguments at the start of
`while` condition expressions with per-iteration recomputation,
`examples/e236_list_struct_filter_first_last_call_arg_else_if_condition.vais`
for using helper calls with filtered first/last arguments at the start of
`else if` condition expressions while preserving the prior `if` guard,
`examples/e237_list_struct_filter_first_last_call_arg_else_if_chain.vais`
for preserving chained `else if` flow and a final `else` after filtered
first/last helper-call argument lowering,
`examples/e238_list_struct_filter_first_last_call_arg_else_if_chain_return.vais`
for compiling chained filtered first/last `else if` conditions whose branches
all return,
`examples/e82_list_literal_direct_arg.vais` for an inline `List<Int>` literal
passed directly to a `List<Int>` parameter,
`examples/d4b.vais` for an inline `List<Int>` literal iterated through a
`List<Int>` parameter,
`examples/e77_nested_list.vais` for a local `List<List<Int>>` literal
double-index read,
`examples/e15_list_recursion.vais` and `examples/e68_binary_search.vais` for
borrowed `&List<Int>` helper parameters,
`examples/e94_map_get_opt.vais` for `Map<Int,Int>.get_opt(key)` returning
`Option<Int>`, `examples/e95_map_assignment.vais` for local `Map<Int,Int>`
assignment copy semantics, `examples/e96_map_bool.vais` for local
`Map<Int,Bool>` insert/get/contains/len and assignment-copy semantics,
`examples/e97_map_char.vais` for local `Map<Int,Char>`
insert/get/contains/len and assignment-copy semantics,
`examples/e98_map_param.vais` for `Map<Int,Int>` parameter mutation by
reference, `examples/e99_map_bool_param.vais` for `Map<Int,Bool>` parameter
mutation by reference, `examples/e100_map_char_param.vais` for
`Map<Int,Char>` parameter mutation by reference,
`examples/e101_map_return.vais` for a `Map<Int,Int>` return value initializing
a local, `examples/e102_map_bool_return.vais` for a `Map<Int,Bool>` return
value initializing a local, `examples/e103_map_char_return.vais` for a
`Map<Int,Char>` return value initializing a local, `examples/e104_map_remove.vais`
for concrete Map key removal, `examples/e105_map_scalar_get_opt.vais` for
`Map<Int,Bool>` and `Map<Int,Char>` get_opt match payloads,
`examples/e106_map_clear.vais` for concrete Map clear and reuse,
`examples/e107_map_str_int.vais` for `Map<Str,Int>` string-key operations and
assignment copy, `examples/e108_map_str_int_param.vais` for `Map<Str,Int>`
parameter mutation by reference, `examples/e109_map_str_int_return.vais` for a
`Map<Str,Int>` return value initializing a local,
`examples/e110_map_str_bool.vais` for `Map<Str,Bool>` string-key operations and
assignment copy, `examples/e111_map_str_bool_param.vais` for `Map<Str,Bool>`
parameter mutation by reference, `examples/e112_map_str_bool_return.vais` for a
`Map<Str,Bool>` return value initializing a local,
`examples/e113_map_str_char.vais` for local `Map<Str,Char>` string-key
operations and assignment copy, `examples/e114_map_str_char_param.vais` for
`Map<Str,Char>` parameter mutation by reference,
`examples/e115_map_str_char_return.vais` for a `Map<Str,Char>` return value
initializing a local, `examples/e116_map_param_assignment.vais` for concrete
Map parameter-source and parameter-target assignment copies,
`examples/e117_map_return_assignment.vais` for concrete Map-returning call
assignment copies, `examples/e118_map_return_assignment_args.vais` for
argument-bearing Map-returning call assignment copies,
`examples/e119_map_param_target_assignment.vais` for all concrete
Map parameter-target assignment copies,
`examples/e132_map_str_str.vais` for local `Map<Str,Str>` string-key/string-value
operations and assignment copy,
`examples/e133_map_str_str_param.vais` for `Map<Str,Str>` parameter mutation by
reference, `examples/e134_map_str_str_return.vais` for a `Map<Str,Str>` return
value initializing a local, `examples/e135_map_str_str_get_opt.vais` for
`Map<Str,Str>.get_opt` string payload match and fallback behavior,
`examples/e280_map_str_str_get_opt_match_contexts.vais` for using those string
payload match expressions in returns, reassignments, helper-call arguments, and
embedded Int returns, `examples/e281_map_str_str_return_infer_get_opt_match_contexts.vais`
for the same contexts after local Map type inference from a `Map<Str,Str>`
returning helper, and `examples/e282_map_str_str_get_opt_match_str_transforms.vais`
for `str_concat`/`str_trim`/`str_lower` transforms in string payload match
expressions, `examples/e283_str_len_reassigned_match_transform.vais` for
runtime `.len()` over locals reassigned from those match-transform results,
`examples/e284_map_str_str_get_opt_match_transform_len.vais` for direct
`.len()` after `str_trim`/`str_lower` match-arm transforms,
`examples/e285_map_str_str_get_opt_str_payload_stability.vais` for lowering
string payload matches through presence checks and value loads instead of
pointer-tagged payload integers,
`examples/e286_map_str_str_get_opt_condition_chains.vais` for using those
embedded string payload matches in `while` and `else if` conditions while
preserving loop reevaluation and else-chain structure, and
`examples/e136_map_entries.vais` for concrete Map `key_at`/`value_at` entry
reads over string and scalar maps, `examples/e137_map_str_str_snapshot.vais`
for writing and reading a `Map<Str,Str>` text snapshot with host file IO, and
`examples/e138_map_str_str_snapshot_load.vais` for parsing that snapshot back
into a `Map<Str,Str>`,
`examples/e293_map_str_str_snapshot_builtin.vais` for the gate-backed
`map_str_str_snapshot` / `map_str_str_load_snapshot` line snapshot builtins,
and `examples/e139_map_return_infer.vais` for inferring
local Map types from Map-returning calls and using `.len()` on Str-returning Map
methods,
`examples/e83_parse_helpers.vais` for the named `parse_uint(s)` and
`parse_int(s)` prelude helpers,
`examples/e73_int_to_string.vais` for `Str(Int)` decimal conversion,
`examples/e140_str_contains.vais` for built-in `str_contains(text, needle)`
over literals, `Map<Str,Str>` document fields, and `List<Str>` elements,
`examples/e149_str_index_of_builtin.vais` for built-in
`str_index_of(text, needle)` returning the first byte index or `-1`,
`examples/e150_str_starts_with_builtin.vais` for built-in
`str_starts_with(text, prefix)` prefix checks including an empty prefix,
`examples/e288_str_ends_with.vais` for built-in
`str_ends_with(text, suffix)` suffix checks including an empty suffix and
Map/List/get_opt match string values,
`examples/e289_str_replace.vais` for built-in
`str_replace(text, needle, replacement)` all-occurrence rewriting over
literals, Map/List string values, and get_opt match strings,
`examples/e290_str_split_into.vais` for built-in
`str_split_into(text, sep, out)` delimiter tokenization into a `List<Str>`
out-param while preserving empty fields,
`examples/e291_str_join.vais` for built-in `str_join(parts, sep)` string
reconstruction from `List<Str>` values, including split/join round trips,
`examples/e292_str_split_lines_into.vais` for built-in
`str_split_lines_into(text, out)` LF/CRLF line tokenization into a `List<Str>`
out-param without adding a final empty line for a trailing line break,
`examples/e151_line_comment_skip.vais` for line comments after expressions while
preserving `#` bytes inside string literals,
`examples/e152_list_clear.vais` for `List<Int>` and `List<Struct>` `clear()`
reuse across local lists and caller-visible list parameters,
`examples/e153_list_contains.vais` for local and parameter `List<Int>`
`contains(value)` membership checks,
`examples/e157_list_int_index_count.vais` for local and parameter `List<Int>`
`index_of(value)` first-index search and `count(value)` duplicate-count search,
`examples/e158_list_remove_at.vais` for local and parameter `List<Int>` and
`List<Str>` `remove_at(index)` deletion with left-shifted elements,
`examples/e159_list_insert_at.vais` for local and parameter `List<Int>` and
`List<Str>` `insert_at(index, value)` insertion with right-shifted elements,
`examples/e160_list_extend.vais` for local and parameter `List<Int>` and
`List<Str>` `extend(other)` appends from same-type named lists,
`examples/e161_list_first.vais` for local and parameter `List<Int>` and
`List<Str>` `first()` head reads with empty-list trap semantics,
`examples/e162_list_struct_first.vais` for local and parameter `List<Struct>`
`first()` head reads copied into struct locals,
`examples/e163_list_struct_remove_at.vais` for local and parameter
`List<Struct>` `remove_at(index)` deletion with left-shifted struct elements,
`examples/e164_list_struct_insert_at.vais` for local and parameter
`List<Struct>` `insert_at(index, value)` insertion with right-shifted struct elements,
`examples/e165_list_struct_extend.vais` for local and parameter
`List<Struct>` `extend(other)` appends from same-type named struct lists,
`examples/e166_list_struct_for_each.vais` for local and parameter
`List<Struct>` for-each loops that copy each element into a struct loop variable,
`examples/e167_list_struct_field_write.vais` for local and parameter
`List<Struct>` indexed field assignments,
`examples/e168_list_index_assignment.vais` for semicolon-free local
`List<Int>` indexed element assignments,
`examples/e169_list_struct_element_assignment.vais` for local and parameter
`List<Struct>` indexed whole-element assignments,
`examples/e170_list_struct_element_return_call.vais` for `List<Struct>`
indexed whole-element assignments from struct-returning helper calls,
`examples/e171_list_struct_push_return_call.vais` for local and parameter
`List<Struct>.push(make_struct(...))` from struct-returning helper calls,
`examples/e172_list_struct_insert_return_call.vais` for local and parameter
`List<Struct>.insert_at(index, make_struct(...))` from struct-returning helper calls,
`examples/e173_list_struct_push_struct_value.vais` for local and parameter
`List<Struct>.push(value)` from struct local/parameter values,
`examples/e174_list_struct_push_insert_element_value.vais` for local and parameter
`List<Struct>.push(xs[i])` and `insert_at(index, xs[i])` from list element values,
`examples/e175_list_struct_push_insert_method_value.vais` for local and parameter
`List<Struct>.push(xs.pop()/remove_at(...))` and `insert_at(index, xs.pop()/remove_at(...))`
from list method return values,
`examples/e176_list_struct_push_insert_first_last_value.vais` for local and
parameter `List<Struct>.push(xs.first()/xs.last())` and
`insert_at(index, xs.first()/xs.last())` from non-mutating list method return
values, including same-list insertion,
`examples/e177_list_struct_extend_return_call.vais` for local and parameter
`List<Struct>.extend(make_list(...))` from same-type list-returning helper
calls,
`examples/e178_list_scalar_str_extend_return_call.vais` for local and
parameter `List<Int>.extend(make_list(...))` and
`List<Str>.extend(make_list(...))` from same-type list-returning helper calls,
`examples/e179_list_extend_inline_literal_source.vais` for local and parameter
`List<Int>.extend([..])` and `List<Str>.extend([..])` from inline list literal
sources,
`examples/e180_list_struct_extend_inline_literal_source.vais` for local and
parameter `List<Struct>.extend([Struct { .. }])` from inline struct list
literal sources,
`examples/e181_list_struct_literal_assignment.vais` for `List<Struct>` typed
local initialization and local/parameter assignment from inline struct list
literals,
`examples/e182_list_struct_method_field_chain.vais` for direct
`List<Struct>.first().field`, `.last().field`, `.pop().field`, and
`.remove_at(index).field` reads on local and parameter lists,
`examples/e183_list_struct_multiline_literal.vais` for multiline typed
`List<Struct>` literals with trailing commas,
`examples/e184_list_struct_multiline_inline_arg.vais` for multiline inline
`List<Struct>` literal arguments with trailing commas,
`examples/e185_list_struct_multiline_inline_arg_statement.vais` for standalone
call statements with multiline inline `List<Struct>` literal arguments,
`examples/e186_list_struct_push_multiline_literal.vais` for
`List<Struct>.push` with multiline struct literals and trailing commas,
`examples/e187_list_struct_multiline_assignment_return.vais` for
`List<Struct>` indexed element assignment and struct returns with multiline
struct literals,
`examples/e188_struct_multiline_local_assignment_call.vais` for plain struct
local initialization, same-type local assignment, and call arguments with
multiline struct literals,
`examples/e189_list_struct_multiline_insert_extend.vais` for
`List<Struct>.insert_at` and `List<Struct>.extend` with multiline struct
literal sources,
`examples/e190_direct_nested_struct_multiline.vais` for single-field nested
struct literals, reads, and writes through full and direct flattening,
`examples/e191_list_nested_struct_field_chain.vais` for indexed
`List<Struct>` element field-chain reads through the same single-field nested
struct flattening,
`examples/e192_list_nested_struct_field_chain_write.vais` for indexed
`List<Struct>` element field-chain writes through that flattened nested struct
slot,
`examples/e193_list_nested_struct_method_field_chain.vais` for
`List<Struct>` method-result field-chain reads through the same nested struct
slot,
`examples/e194_struct_return_field_chain.vais` for struct-returning helper
field-chain reads through top-level and single-field nested fields,
`examples/e195_nested_struct_literal_return.vais` for helpers returning
single-field nested struct literals directly,
`examples/e196_multi_field_nested_struct.vais` for scalar multi-field nested
struct local literals, direct helper returns, and field-chain reads,
`examples/e197_list_multi_field_nested_struct.vais` for `List<Struct>`
elements with multi-field nested structs, including push, element copy, indexed
nested reads/writes, parameter mutation, and method-result nested field chains,
`examples/e198_struct_str_fields.vais` for document-like structs with `Str`
fields, equality, string helper calls, and `.len()` chains,
`examples/e199_list_struct_str_fields.vais` for `List<Struct>` document records
with `Str` field reads through for-each, index, first, and last,
`examples/e200_list_struct_str_field_write.vais` for local and parameter
`List<Struct>` document records whose indexed `Str` fields are reassigned,
`examples/e201_list_struct_str_method_fields.vais` for `List<Struct>` document
records whose `pop()`/`remove_at()` method results expose `Str` fields,
`examples/e202_proc_capture_result.vais` for full/direct `proc_capture`
returning the standard `ProcessResult { code, stdout, stderr }` struct,
`examples/e154_list_str_contains.vais` for local and parameter `List<Str>`
`contains(value)` membership checks,
`examples/e155_list_str_index_of.vais` for local and parameter `List<Str>`
`index_of(value)` first-index search,
`examples/e156_list_str_count.vais` for local and parameter `List<Str>`
`count(value)` duplicate-count search,
`examples/e141_str_trim.vais` for built-in `str_trim(text)` over literals,
`Map<Str,Str>` document fields, and `List<Str>` elements,
`examples/e142_str_lower.vais` for built-in `str_lower(text)` over literals,
trimmed document fields, `Map<Str,Str>` document fields, and `List<Str>`
elements,
`examples/e287_str_upper.vais` for built-in `str_upper(text)` over literals,
trimmed document fields, `Map<Str,Str>` document fields, `List<Str>` elements,
and `Map<Str,Str>.get_opt` match payloads,
`examples/e143_doc_tokenize.vais` for a document tokenization pipeline using
`str_slice`, `str_trim`, `str_lower`, `str_contains`, and a `List<Str>` return,
`examples/e144_doc_score_for_each.vais` for scoring normalized document tokens
with `List<Str>` for-each, `examples/e145_str_split_ws_into.vais` for built-in
whitespace tokenization into a `List<Str>` out-param,
`examples/e290_str_split_into.vais` for delimiter tokenization that preserves
empty fields,
`examples/e291_str_join.vais` for joining `List<Str>` values back into a
separator-delimited string, and
`examples/e292_str_split_lines_into.vais` for splitting LF/CRLF document text
into reusable line buffers while omitting a trailing empty line, and
`examples/e146_doc_term_counts_into.vais` for built-in document term-frequency
indexing into a `Map<Str,Int>` out-param, and
`examples/e147_doc_term_overlap_score.vais` for built-in overlap scoring across
query/document term-frequency maps, and
`examples/e148_doc_term_weighted_score.vais` for built-in weighted scoring that
rewards repeated query/document term hits,
`examples/e297_vaisdb_file_ingest_workflow.vais` for combining those document
helpers with host file IO and argv paths in a Vais-authored workflow,
`examples/e298_vaisdb_file_ingest_result_flow.vais` for adding `fs_exists`
guarded `Result<Int,Int>` missing-file and malformed-input handling to the same
file workflow,
`examples/e301_result_str_int_file_read.vais` for adding `Result<Str,Int>`
guarded text reads with string-payload `?` propagation and missing-file error
recovery,
`examples/e302_result_str_int_param_flow.vais` for adding `Result<Str,Int>`
helper parameter forwarding and inline payload/error recovery,
`examples/e303_result_metric_int_struct_payload.vais` for adding the first
`Result<Metric,Int>` structured payload helper flow with field recovery,
`examples/e304_result_record_int_struct_payload.vais` for extending structured
Result payload helper flows beyond the previous `Metric`-only slice,
`examples/e305_result_multiline_struct_payload.vais` for multiline
declared-struct Result payload helper flows,
`examples/e306_result_struct_str_fields.vais` for declared-struct Result
payload helper flows with `Str` fields and length recovery,
`examples/e307_result_struct_try_payload.vais` for declared-struct Result
payload helper flows with local-binding `?` and field reuse,
`examples/e308_vaisdb_artifact_record_workflow.vais` for combining
`Result<DocArtifact,Int>`, `List<DocArtifact>` outputs, metadata snapshots, and
error propagation in a VaisDB-style record workflow,
`examples/e309_vaisdb_artifact_store_snapshot.vais` for serializing and
reloading a persisted `List<DocArtifact>` store through declared-struct Result
parsing helpers,
`examples/e310_vaisdb_artifact_query_report.vais` for querying persisted
artifact stores and returning reusable `Result<Str,Int>` report payloads with
store/query error codes,
`examples/e311_result_call_argument_flow.vais` for direct call-argument use of
`Result<Str,Int>` and declared-struct Result-returning helpers,
`examples/e312_result_struct_local_wrapper_flow.vais` for explicit
`VaisResult<Struct>Int` wrapper payload local copies,
`examples/e313_result_struct_str_match_flow.vais` for declared-struct Result
matches that recover title/ID string fields,
`examples/e314_result_struct_str_concat_match_flow.vais` for declared-struct
Result matches that compose title/ID string fields into report labels,
`examples/e315_result_struct_str_transform_match_flow.vais` for declared-struct
Result matches that normalize title/ID string fields for report labels,
`examples/e316_result_struct_str_transform_len_match_flow.vais` for
declared-struct Result matches that score transformed title/ID/body string
fields with chained `.len()` terms,
`examples/e317_result_struct_payload_helper_call_score.vais` for
declared-struct Result matches that pass `Ok` payloads to reusable scoring
helpers,
`examples/e318_result_struct_payload_helper_call_arithmetic.vais` for
declared-struct Result matches that compose reusable helper-call score terms
with payload fields,
`examples/e319_result_struct_payload_field_helper_call_arithmetic.vais` for
declared-struct Result matches that pass payload string fields to reusable
integer helpers and compose the results with payload fields,
`examples/e320_result_struct_payload_int_field_helper_call_arithmetic.vais` for
declared-struct Result matches that pass payload integer fields to reusable
integer helpers and compose the results with string-field helper terms,
`examples/e321_result_struct_payload_bool_match_condition.vais` for
declared-struct Result matches that return Bool conditions from payload helper
terms and error-code comparisons,
`examples/e322_vaisdb_module_boundary/main.vais` for imported VaisDB modules
that share declared structs, structured Result helpers, List outputs, and
Map-backed scoring helpers across files,
`examples/e323_cli_package` for package-directory execution with manifest
entry resolution, imports, and CLI argv forwarding,
`examples/e326_cli_binary_target` for optional package `binary` metadata that
installs `veriqel-demo` while preserving package-directory entry resolution,
`scripts/vaisc package examples/e326_cli_binary_target -o <dist-dir>
--archive` for an extractable user-package release archive containing
`veriqel-demo-0.1.0/bin/veriqel-demo` and `vais.toml`,
`examples/e328_cli_package_assets` for optional package `assets` metadata that
copies static files into `dist/assets` and the archive payload while preserving
argv-capable package execution,
`examples/e299_vaisdb_benchmark_report.vais` for timing and persisting a small
VaisDB benchmark report from Vais code,
`examples/e300_vaisdb_benchmark_cli_report.vais` for invoking the e295 indexer
from Vais code with `proc_capture` and saving direct/default timing metrics,
`tools/vaisdb_benchmark_report.vais` for turning that benchmark report into a
reusable Vais-authored developer command with metric parsing and summary
output,
`examples/e69_palindrome_string.vais` for two-pointer `Str` scans with
computed byte indexes,
`examples/e71_string_index_of.vais` for `Str` substring search with computed
byte indexes,
`examples/e74_map_basic.vais` for the verified local `Map<Int,Int>` slice, and
`examples/e84_list_methods.vais` for `List<T>.is_empty()`, `last()`, and
`pop()`, `examples/e121_list_str_methods.vais` for local `List<Str>` push,
string equality, and chained `.len()` on index, `last`, and `pop` results,
`examples/e122_list_str_param.vais` for `List<Str>` helper parameter reads and
methods, `examples/e123_list_str_return.vais` for `List<Str>` helper return
values with string equality and chained `.len()` on returned-list string
elements,
`examples/e124_list_str_literal.vais` for typed local `List<Str>`
literals, `examples/e125_list_str_assignment.vais` for `List<Str>` assignment
copy, `examples/e126_list_str_literal_assignment.vais` for `List<Str>` literal
assignment, `examples/e127_list_str_return_assignment.vais` for return-call
assignment, `examples/e128_list_str_param_target_assignment.vais` for
parameter-target assignment copy,
`examples/e129_list_str_param_literal_assignment.vais` for parameter-target
literal assignment, `examples/e130_list_str_param_return_assignment.vais` for
parameter-target return-call assignment,
`examples/e131_list_str_inline_literal_arg.vais` for inline literal arguments,
and
`examples/e85_char_type.vais` for the promoted Int-compatible
`Char` scalar slice, and `examples/e86_for_loop.vais` for exclusive and
inclusive range `for` loops, and `examples/e87_break_continue.vais` for loop
control flow, and `examples/e88_bool_type.vais` for explicit `Bool` locals,
helper parameters/returns, and unary `not`, and `examples/e89_str_type.vais`
for explicit `Str` locals, helper parameters/returns, reassignment, length,
index, and equality.
The release corpus also includes smaller control-flow and scanner examples:
`examples/e06_for_sum.vais`, `examples/e10_bool_logic.vais`,
`examples/e12_exclusive_range.vais`, `examples/e13_nested_for.vais`,
`examples/e36_bool_predicate.vais`, `examples/e44_string_len.vais`,
`examples/e52_state_machine.vais`, `examples/e53_word_count.vais`,
`examples/e57_break.vais`, `examples/e58_continue.vais`,
`examples/e61_array_index_expr.vais`, and
`examples/e65_loop_break_acc.vais`, `examples/fr1.vais`,
`examples/fr2.vais`, `examples/t2.vais`, `examples/t3.vais`,
`examples/t4.vais`, `examples/t5.vais`, and `examples/t6.vais`.

Files not listed as `native-supported` are retained as examples or future
coverage candidates, but they are not public release claims until promoted into
the parity manifest.
