#!/usr/bin/env bash
# Deterministic document/VaisDB workflow gate.
#
# This groups the product-facing document helpers into one reproducible smoke:
# line parsing, metadata snapshots, Result-style parse/helper flow, and the
# first Vais-authored index/query prototype. The examples intentionally return
# 42, so the wrapper compares exit status instead of relying on shell success.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

expect_exit() {
    local label="$1"
    local want="$2"
    shift 2

    set +e
    "$@" >/dev/null
    local got=$?
    set -e

    if [ "$got" -eq "$want" ]; then
        printf '  PASS %s (= %s)\n' "$label" "$got"
        return 0
    fi

    printf '  FAIL %s: got=%s expect=%s\n' "$label" "$got" "$want"
    return 1
}

expect_pair() {
    local label="$1"
    local src="$2"

    expect_exit "$label direct" 42 "$ROOT/scripts/vaisc" run "$src" --engine direct
    expect_exit "$label default" 42 "$ROOT/scripts/vaisc" run "$src"
}

expect_pair_args() {
    local label="$1"
    local src="$2"
    shift 2

    expect_exit "$label direct" 42 "$ROOT/scripts/vaisc" run "$src" --engine direct -- "$@"
    expect_exit "$label default" 42 "$ROOT/scripts/vaisc" run "$src" -- "$@"
}

write_file_ingest_inputs() {
    local dir="$1"

    printf 'VaisDB Guide\nAI cache ai CACHE vector ai\n' > "$dir/doc-a.txt"
    printf 'Cache Notes\ncache cache vector vector\n' > "$dir/doc-b.txt"
    printf 'ai cache ai\n' > "$dir/query.txt"
}

expect_package_output() {
    local label="$1"
    local engine_flag="$2"
    local dist="$3"

    rm -rf "$dist"
    if [ -n "$engine_flag" ]; then
        expect_exit "$label package" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e323_cli_package" -o "$dist" --engine "$engine_flag"
    else
        expect_exit "$label package" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e323_cli_package" -o "$dist"
    fi
    expect_exit "$label binary" 42 "$dist/bin/e323_cli_package"
    expect_exit "$label argv binary" 42 "$dist/bin/e323_cli_package" vaisdb cache
    expect_exit "$label manifest" 0 test -f "$dist/vais.toml"
}

expect_binary_target_output() {
    local label="$1"
    local engine_flag="$2"
    local dist="$3"

    rm -rf "$dist"
    if [ -n "$engine_flag" ]; then
        expect_exit "$label package" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e326_cli_binary_target" -o "$dist" --engine "$engine_flag"
    else
        expect_exit "$label package" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e326_cli_binary_target" -o "$dist"
    fi
    expect_exit "$label binary" 42 "$dist/bin/veriqel-demo"
    expect_exit "$label argv binary" 42 "$dist/bin/veriqel-demo" veriqel package
    expect_exit "$label manifest" 0 test -f "$dist/vais.toml"
    expect_exit "$label no package-name binary" 1 test -e "$dist/bin/e326_cli_binary_target"
}

expect_binary_target_archive_output() {
    local label="$1"
    local engine_flag="$2"
    local dist="$3"
    local archive="$dist/veriqel-demo-0.1.0.tar.gz"
    local extract="$dist.extract"
    local root="$extract/veriqel-demo-0.1.0"

    rm -rf "$dist" "$extract"
    if [ -n "$engine_flag" ]; then
        expect_exit "$label package archive" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e326_cli_binary_target" -o "$dist" --engine "$engine_flag" --archive
    else
        expect_exit "$label package archive" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e326_cli_binary_target" -o "$dist" --archive
    fi
    expect_exit "$label archive exists" 0 test -f "$archive"
    mkdir -p "$extract"
    expect_exit "$label archive extracts" 0 tar -C "$extract" -xzf "$archive"
    expect_exit "$label archived binary" 42 "$root/bin/veriqel-demo" veriqel package
    expect_exit "$label archived manifest" 0 test -f "$root/vais.toml"
}

expect_assets_package_output() {
    local label="$1"
    local engine_flag="$2"
    local dist="$3"
    local archive="$dist/veriqel-assets-0.1.0.tar.gz"
    local extract="$dist.extract"
    local root="$extract/veriqel-assets-0.1.0"

    rm -rf "$dist" "$extract"
    if [ -n "$engine_flag" ]; then
        expect_exit "$label package archive" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e328_cli_package_assets" -o "$dist" --engine "$engine_flag" --archive
    else
        expect_exit "$label package archive" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e328_cli_package_assets" -o "$dist" --archive
    fi
    expect_exit "$label binary" 42 "$dist/bin/veriqel-assets" assets package
    expect_exit "$label dist asset" 0 grep -q "veriqel packaged assets" "$dist/assets/docs/guide.txt"
    expect_exit "$label archive exists" 0 test -f "$archive"
    mkdir -p "$extract"
    expect_exit "$label archive extracts" 0 tar -C "$extract" -xzf "$archive"
    expect_exit "$label archived binary" 42 "$root/bin/veriqel-assets" assets package
    expect_exit "$label archived asset" 0 grep -q "veriqel packaged assets" "$root/assets/docs/guide.txt"
    expect_exit "$label archived manifest" 0 test -f "$root/vais.toml"
}

echo "VaisDB document workflow gate"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

expect_pair "line split" "$ROOT/examples/e292_str_split_lines_into.vais"
expect_pair "metadata snapshot" "$ROOT/examples/e293_map_str_str_snapshot_builtin.vais"
expect_pair "parse/error flow" "$ROOT/examples/e294_result_try_parse_error_flow.vais"
expect_pair "result map helper flow" "$ROOT/examples/e296_result_map_param_flow.vais"
expect_pair "indexer prototype" "$ROOT/examples/e295_vaisdb_indexer_prototype.vais"
expect_pair "file ingest generated workflow" "$ROOT/examples/e297_vaisdb_file_ingest_workflow.vais"
expect_pair "file ingest Result flow" "$ROOT/examples/e298_vaisdb_file_ingest_result_flow.vais"
expect_pair "ingest Str error message flow" "$ROOT/examples/e330_vaisdb_ingest_error_message_flow.vais"
expect_pair "top-k ranking report" "$ROOT/examples/e332_vaisdb_topk_ranking_report.vais"
expect_pair "snapshot version migration" "$ROOT/examples/e333_vaisdb_snapshot_version_migration.vais"
expect_pair "index persistence incremental" "$ROOT/examples/e334_vaisdb_index_persistence_incremental.vais"
expect_pair "file read Result Str payload flow" "$ROOT/examples/e301_result_str_int_file_read.vais"
expect_pair "Result Str parameter flow" "$ROOT/examples/e302_result_str_int_param_flow.vais"
expect_pair "Result Metric struct payload flow" "$ROOT/examples/e303_result_metric_int_struct_payload.vais"
expect_pair "Result declared struct payload flow" "$ROOT/examples/e304_result_record_int_struct_payload.vais"
expect_pair "Result multiline struct payload flow" "$ROOT/examples/e305_result_multiline_struct_payload.vais"
expect_pair "Result struct Str fields payload flow" "$ROOT/examples/e306_result_struct_str_fields.vais"
expect_pair "Result struct question payload flow" "$ROOT/examples/e307_result_struct_try_payload.vais"
expect_pair "VaisDB artifact record workflow" "$ROOT/examples/e308_vaisdb_artifact_record_workflow.vais"
expect_pair "VaisDB artifact store snapshot workflow" "$ROOT/examples/e309_vaisdb_artifact_store_snapshot.vais"
expect_pair "VaisDB artifact query report workflow" "$ROOT/examples/e310_vaisdb_artifact_query_report.vais"
expect_pair "Result call argument flow" "$ROOT/examples/e311_result_call_argument_flow.vais"
expect_pair "Result struct local wrapper flow" "$ROOT/examples/e312_result_struct_local_wrapper_flow.vais"
expect_pair "Result struct Str match flow" "$ROOT/examples/e313_result_struct_str_match_flow.vais"
expect_pair "Result struct Str concat match flow" "$ROOT/examples/e314_result_struct_str_concat_match_flow.vais"
expect_pair "Result struct Str transform match flow" "$ROOT/examples/e315_result_struct_str_transform_match_flow.vais"
expect_pair "Result struct Str transform len match flow" "$ROOT/examples/e316_result_struct_str_transform_len_match_flow.vais"
expect_pair "Result struct payload helper-call score flow" "$ROOT/examples/e317_result_struct_payload_helper_call_score.vais"
expect_pair "Result struct payload helper-call arithmetic flow" "$ROOT/examples/e318_result_struct_payload_helper_call_arithmetic.vais"
expect_pair "Result struct payload field helper-call arithmetic flow" "$ROOT/examples/e319_result_struct_payload_field_helper_call_arithmetic.vais"
expect_pair "Result struct payload Int field helper-call arithmetic flow" "$ROOT/examples/e320_result_struct_payload_int_field_helper_call_arithmetic.vais"
expect_pair "Result struct payload Bool match condition flow" "$ROOT/examples/e321_result_struct_payload_bool_match_condition.vais"
expect_pair "VaisDB imported module boundary workflow" "$ROOT/examples/e322_vaisdb_module_boundary/main.vais"
expect_pair "VaisDB CLI package directory workflow" "$ROOT/examples/e323_cli_package"
expect_pair_args "VaisDB CLI package argv workflow" "$ROOT/examples/e323_cli_package" vaisdb cache
expect_package_output "VaisDB CLI package output default" "" "$tmp/package-default"
expect_package_output "VaisDB CLI package output direct" "direct" "$tmp/package-direct"
expect_pair_args "VaisDB CLI binary target package argv workflow" "$ROOT/examples/e326_cli_binary_target" veriqel package
expect_binary_target_output "VaisDB CLI binary target output default" "" "$tmp/package-binary-default"
expect_binary_target_output "VaisDB CLI binary target output direct" "direct" "$tmp/package-binary-direct"
expect_binary_target_archive_output "VaisDB CLI binary target archive default" "" "$tmp/package-binary-archive-default"
expect_binary_target_archive_output "VaisDB CLI binary target archive direct" "direct" "$tmp/package-binary-archive-direct"
expect_pair_args "VaisDB CLI package assets argv workflow" "$ROOT/examples/e328_cli_package_assets" assets package
expect_assets_package_output "VaisDB CLI package assets output default" "" "$tmp/package-assets-default"
expect_assets_package_output "VaisDB CLI package assets output direct" "direct" "$tmp/package-assets-direct"
expect_pair "benchmark report workflow" "$ROOT/examples/e299_vaisdb_benchmark_report.vais"
expect_pair_args "benchmark CLI report workflow" "$ROOT/examples/e300_vaisdb_benchmark_cli_report.vais" "$ROOT"

expect_pair_args \
    "benchmark report tool workflow" \
    "$ROOT/tools/vaisdb_benchmark_report.vais" \
    "$ROOT" \
    "$tmp/tool-raw-report.txt" \
    "$tmp/tool-summary-report.txt"
expect_exit \
    "benchmark report script workflow" \
    42 \
    bash "$ROOT/scripts/vaisdb-benchmark-report.sh" \
    "$tmp/script-raw-report.txt" \
    "$tmp/script-summary-report.txt"

# Vais-authored vaisdb CLI: ingest/query/report subcommands over the persisted
# docid.term index, plus readable error paths with distinct exit codes.
cli_index="$tmp/vaisdb-cli-index.txt"
printf 'ai ai ai cache\n' > "$tmp/vaisdb-cli-d1.txt"
printf 'ai cache cache\n' > "$tmp/vaisdb-cli-d2.txt"
expect_exit "vaisdb cli ingest d1" 0 "$ROOT/scripts/vaisc" run "$ROOT/tools/vaisdb_cli.vais" -- ingest "$cli_index" d1 "$tmp/vaisdb-cli-d1.txt"
expect_exit "vaisdb cli ingest d2" 0 "$ROOT/scripts/vaisc" run "$ROOT/tools/vaisdb_cli.vais" -- ingest "$cli_index" d2 "$tmp/vaisdb-cli-d2.txt"
expect_exit "vaisdb cli query d1" 4 "$ROOT/scripts/vaisc" run "$ROOT/tools/vaisdb_cli.vais" -- query "$cli_index" d1 "ai cache"
expect_exit "vaisdb cli query d2" 3 "$ROOT/scripts/vaisc" run "$ROOT/tools/vaisdb_cli.vais" -- query "$cli_index" d2 "ai cache"
expect_exit "vaisdb cli report top" 4 "$ROOT/scripts/vaisc" run "$ROOT/tools/vaisdb_cli.vais" -- report "$cli_index" "ai cache"
expect_exit "vaisdb cli report direct" 4 "$ROOT/scripts/vaisc" run "$ROOT/tools/vaisdb_cli.vais" --engine direct -- report "$cli_index" "ai cache"
expect_exit "vaisdb cli missing index" 3 "$ROOT/scripts/vaisc" run "$ROOT/tools/vaisdb_cli.vais" -- query "$tmp/vaisdb-cli-none.txt" d1 "ai"
expect_exit "vaisdb cli unknown subcommand" 2 "$ROOT/scripts/vaisc" run "$ROOT/tools/vaisdb_cli.vais" -- frobnicate
expect_exit "vaisdb cli usage" 1 "$ROOT/scripts/vaisc" run "$ROOT/tools/vaisdb_cli.vais" --
expect_exit "vaisdb cli script wrapper" 4 bash "$ROOT/scripts/vaisdb-cli.sh" report "$cli_index" "ai cache"

# Installable vaisdb package: multi-module package builds to dist/bin/vaisdb,
# the packaged binary serves the CLI subcommands and self-test, and the
# release archive round-trips.
vdb_dist="$tmp/vaisdb-dist"
vdb_extract="$tmp/vaisdb-extract"
vdb_index="$tmp/vaisdb-pkg-index.txt"
rm -rf "$vdb_dist" "$vdb_extract"
expect_exit "vaisdb package build" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e337_vaisdb_cli_package" -o "$vdb_dist" --archive
expect_exit "vaisdb package self-test" 42 "$vdb_dist/bin/vaisdb"
expect_exit "vaisdb package ingest" 0 "$vdb_dist/bin/vaisdb" ingest "$vdb_index" d1 "$tmp/vaisdb-cli-d1.txt"
expect_exit "vaisdb package query" 4 "$vdb_dist/bin/vaisdb" query "$vdb_index" d1 "ai cache"
expect_exit "vaisdb package report" 4 "$vdb_dist/bin/vaisdb" report "$vdb_index" "ai cache"
vdb_docs="$tmp/vaisdb-pkg-docs"
mkdir -p "$vdb_docs"
printf 'ai ai ai cache\n' > "$vdb_docs/pd1.txt"
printf 'ai cache cache\n' > "$vdb_docs/pd2.txt"
printf 'not ingested\n' > "$vdb_docs/skip.bin"
vdb_dir_index="$tmp/vaisdb-pkg-dir-index.txt"
expect_exit "vaisdb package ingest-dir" 0 "$vdb_dist/bin/vaisdb" ingest-dir "$vdb_dir_index" "$vdb_docs"
expect_exit "vaisdb package rank" 4 "$vdb_dist/bin/vaisdb" rank "$vdb_dir_index" "ai cache" 2
expect_exit "vaisdb package ingest-dir missing" 3 "$vdb_dist/bin/vaisdb" ingest-dir "$vdb_dir_index" "$tmp/vaisdb-no-such-docs"
expect_exit "vaisdb package rank bad k" 1 "$vdb_dist/bin/vaisdb" rank "$vdb_dir_index" "ai cache" 0
expect_exit "vaisdb package docs" 2 "$vdb_dist/bin/vaisdb" docs "$vdb_dir_index"
expect_exit "vaisdb package stats" 2 "$vdb_dist/bin/vaisdb" stats "$vdb_dir_index"
expect_exit "vaisdb package remove" 0 "$vdb_dist/bin/vaisdb" remove "$vdb_dir_index" pd1
expect_exit "vaisdb package docs after remove" 1 "$vdb_dist/bin/vaisdb" docs "$vdb_dir_index"
expect_exit "vaisdb package remove missing" 3 "$vdb_dist/bin/vaisdb" remove "$vdb_dir_index" ghost
vdb_stdin_index="$tmp/vaisdb-stdin-index.txt"
expect_exit "vaisdb ingest-stdin" 0 /bin/sh -c "printf 'ai cache ai\n' | '$vdb_dist/bin/vaisdb' ingest-stdin '$vdb_stdin_index' d1"
expect_exit "vaisdb ingest-stdin empty" 1 /bin/sh -c "'$vdb_dist/bin/vaisdb' ingest-stdin '$vdb_stdin_index' d2 < /dev/null"
expect_exit "vaisdb ingest-stdin query" 3 "$vdb_dist/bin/vaisdb" query "$vdb_stdin_index" d1 "ai cache"

vgrep_dist="$tmp/vaisgrep-dist"
vgrep_docs="$tmp/vaisgrep-docs"
rm -rf "$vgrep_dist" "$vgrep_docs"
mkdir -p "$vgrep_docs"
printf 'cache one\nplain\ncache two\n' > "$vgrep_docs/a.txt"
printf 'let cache = 1\n' > "$vgrep_docs/b.vais"
printf 'cache\n' > "$vgrep_docs/skip.bin"
expect_exit "vaisgrep package build" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e341_vaisgrep_package" -o "$vgrep_dist"
expect_exit "vaisgrep package self-test" 42 "$vgrep_dist/bin/vaisgrep"
expect_exit "vaisgrep file search" 2 "$vgrep_dist/bin/vaisgrep" cache "$vgrep_docs/a.txt"
expect_exit "vaisgrep dir search" 3 "$vgrep_dist/bin/vaisgrep" cache "$vgrep_docs"
expect_exit "vaisgrep count mode" 3 "$vgrep_dist/bin/vaisgrep" -c cache "$vgrep_docs"
expect_exit "vaisgrep missing path" 3 "$vgrep_dist/bin/vaisgrep" cache "$vgrep_docs/no-such"
expect_exit "vaisgrep empty pattern" 1 "$vgrep_dist/bin/vaisgrep" "" "$vgrep_docs/a.txt"
mkdir -p "$vgrep_docs/sub/deeper"
printf 'cache sub\n' > "$vgrep_docs/sub/b2.txt"
printf 'cache deep\ncache again\n' > "$vgrep_docs/sub/deeper/c2.md"
expect_exit "vaisgrep recursive search" 6 "$vgrep_dist/bin/vaisgrep" -r cache "$vgrep_docs"
expect_exit "vaisgrep single level unchanged" 3 "$vgrep_dist/bin/vaisgrep" cache "$vgrep_docs"
expect_exit "vaisgrep stdin lines" 2 /bin/sh -c "printf 'one cache\nplain\ntwo cache\n' | '$vgrep_dist/bin/vaisgrep' cache -"
expect_exit "vaisgrep stdin count" 2 /bin/sh -c "printf 'one cache\nplain\ntwo cache\n' | '$vgrep_dist/bin/vaisgrep' -c cache -"
expect_exit "vaisgrep empty stdin" 0 /bin/sh -c "'$vgrep_dist/bin/vaisgrep' cache - < /dev/null"

vmake_dist="$tmp/vaismake-dist"
vmake_tasks="$tmp/vaismake-tasks.txt"
rm -rf "$vmake_dist"
printf 'hello = /bin/echo hi there\nok = /usr/bin/true\nbad = /usr/bin/false\n' > "$vmake_tasks"
expect_exit "vaismake package build" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e344_vaismake_package" -o "$vmake_dist"
expect_exit "vaismake package self-test" 42 "$vmake_dist/bin/vaismake"
expect_exit "vaismake list" 3 "$vmake_dist/bin/vaismake" "$vmake_tasks"
expect_exit "vaismake run ok" 0 "$vmake_dist/bin/vaismake" "$vmake_tasks" ok
expect_exit "vaismake run bad" 1 "$vmake_dist/bin/vaismake" "$vmake_tasks" bad
expect_exit "vaismake capture" 0 "$vmake_dist/bin/vaismake" -o "$vmake_tasks" hello
expect_exit "vaismake unknown task" 3 "$vmake_dist/bin/vaismake" "$vmake_tasks" nope
expect_exit "vaismake missing file" 3 "$vmake_dist/bin/vaismake" "$tmp/no-such-tasks.txt" ok
vmake_env_tasks="$tmp/vaismake-env-tasks.txt"
printf '!env VAIS_MAKE_GATE_FLAG=on\nflag = /usr/bin/printenv VAIS_MAKE_GATE_FLAG\n' > "$vmake_env_tasks"
expect_exit "vaismake env overlay" 0 "$vmake_dist/bin/vaismake" "$vmake_env_tasks" flag
vmake_chain_tasks="$tmp/vaismake-chain-tasks.txt"
printf 'search = %s cache %s\n' "$vgrep_dist/bin/vaisgrep" "$vgrep_docs/a.txt" > "$vmake_chain_tasks"
expect_exit "vaismake chains vaisgrep" 2 "$vmake_dist/bin/vaismake" "$vmake_chain_tasks" search
vmake_dep_tasks="$tmp/vaismake-dep-tasks.txt"
printf 'prep = /usr/bin/true\nbuild = /bin/echo built\nfail = /usr/bin/false\nbroken = /bin/echo never\nloopa = /usr/bin/true\nloopb = /usr/bin/true\n!needs build prep\n!needs broken fail\n!needs loopa loopb\n!needs loopb loopa\n' > "$vmake_dep_tasks"
expect_exit "vaismake deps run first" 0 "$vmake_dist/bin/vaismake" "$vmake_dep_tasks" build
expect_exit "vaismake dep failure stops" 1 "$vmake_dist/bin/vaismake" "$vmake_dep_tasks" broken
expect_exit "vaismake dep cycle detected" 4 "$vmake_dist/bin/vaismake" "$vmake_dep_tasks" loopa
expect_exit "vaismake gates.tasks parses" 17 "$vmake_dist/bin/vaismake" "$ROOT/tools/gates.tasks"

vfmt_dist="$tmp/vaisfmt-dist"
vfmt_src="$tmp/vaisfmt-src"
rm -rf "$vfmt_dist" "$vfmt_src"
mkdir -p "$vfmt_src/sub"
printf 'fn main() -> Int {   \n    return 42\n}\n' > "$vfmt_src/dirty.vais"
printf 'fn helper() -> Int {\n    return 1\n}\n' > "$vfmt_src/sub/clean.vais"
expect_exit "vaisfmt package build" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e346_vaisfmt_package" -o "$vfmt_dist"
expect_exit "vaisfmt package self-test" 42 "$vfmt_dist/bin/vaisfmt"
expect_exit "vaisfmt check finds dirty" 1 "$vfmt_dist/bin/vaisfmt" -c "$vfmt_src"
expect_exit "vaisfmt fix rewrites" 1 "$vfmt_dist/bin/vaisfmt" "$vfmt_src"
expect_exit "vaisfmt recheck clean" 0 "$vfmt_dist/bin/vaisfmt" -c "$vfmt_src"
expect_exit "vaisfmt missing path" 3 "$vfmt_dist/bin/vaisfmt" -c "$tmp/vaisfmt-no-such"
expect_exit "vaisfmt repo std clean" 0 "$vfmt_dist/bin/vaisfmt" -c "$ROOT/std"
expect_exit "vaisfmt stdin dirty check" 1 /bin/sh -c "printf 'x   \n' | '$vfmt_dist/bin/vaisfmt' -c -"
expect_exit "vaisfmt stdin clean check" 0 /bin/sh -c "printf 'x\n' | '$vfmt_dist/bin/vaisfmt' -c -"
expect_exit "vaisfmt stdin filter output" 0 /bin/sh -c "printf 'x   \ny\t\n' | '$vfmt_dist/bin/vaisfmt' - > '$tmp/fmt-filter.out' && printf 'x\ny\n' | cmp -s - '$tmp/fmt-filter.out'"
expect_exit "three-tool pipe grep fmt grep" 2 /bin/sh -c "printf 'a cache   \nplain\nb cache\t\n' | '$vgrep_dist/bin/vaisgrep' cache - | '$vfmt_dist/bin/vaisfmt' - | '$vgrep_dist/bin/vaisgrep' -c cache -"
expect_exit "grep to db chain ingest" 0 /bin/sh -c "printf 'cache one\ncache two\n' | '$vgrep_dist/bin/vaisgrep' cache - | '$vdb_dist/bin/vaisdb' ingest-stdin '$tmp/vaisdb-chain-index.txt' hits"
expect_exit "grep error stdout stays empty" 0 /bin/sh -c "out=\$('$vgrep_dist/bin/vaisgrep' cache '$tmp/no-such-path' 2>/dev/null); test -z \"\$out\""
expect_exit "fmt error stdout stays empty" 0 /bin/sh -c "out=\$('$vfmt_dist/bin/vaisfmt' -c '$tmp/no-such-path' 2>/dev/null); test -z \"\$out\""

vbench_dist="$tmp/vaisbench-dist"
rm -rf "$vbench_dist"
expect_exit "vaisbench package build" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e350_vaisbench_package" -o "$vbench_dist"
expect_exit "vaisbench package self-test" 42 "$vbench_dist/bin/vaisbench"
expect_exit "vaisbench times true" 0 "$vbench_dist/bin/vaisbench" 3 /usr/bin/true
expect_exit "vaisbench propagates failure" 1 "$vbench_dist/bin/vaisbench" 3 /usr/bin/false
expect_exit "vaisbench rejects bad count" 2 "$vbench_dist/bin/vaisbench" 0 /usr/bin/true
expect_exit "vaisbench budget passes" 0 "$vbench_dist/bin/vaisbench" -b 60000 2 /usr/bin/true
expect_exit "vaisbench budget exceeded" 3 "$vbench_dist/bin/vaisbench" -b -1 2 /usr/bin/true

vdiff_dist="$tmp/vaisdiff-dist"
rm -rf "$vdiff_dist"
printf 'alpha\nbeta\ngamma\n' > "$tmp/vaisdiff-a.txt"
printf 'alpha\nBETA\ngamma\n' > "$tmp/vaisdiff-b.txt"
expect_exit "vaisdiff package build" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e351_vaisdiff_package" -o "$vdiff_dist"
expect_exit "vaisdiff package self-test" 42 "$vdiff_dist/bin/vaisdiff"
expect_exit "vaisdiff identical" 0 "$vdiff_dist/bin/vaisdiff" "$tmp/vaisdiff-a.txt" "$tmp/vaisdiff-a.txt"
expect_exit "vaisdiff differ" 1 "$vdiff_dist/bin/vaisdiff" "$tmp/vaisdiff-a.txt" "$tmp/vaisdiff-b.txt"
expect_exit "vaisdiff stdin side" 1 /bin/sh -c "printf 'alpha\nBETA\ngamma\n' | '$vdiff_dist/bin/vaisdiff' '$tmp/vaisdiff-a.txt' -"
expect_exit "vaisdiff missing file" 3 /bin/sh -c "'$vdiff_dist/bin/vaisdiff' '$tmp/vaisdiff-no-such' '$tmp/vaisdiff-a.txt' 2>/dev/null"
expect_exit "vaisdiff both stdin rejected" 2 /bin/sh -c "'$vdiff_dist/bin/vaisdiff' - - 2>/dev/null"

vwc_dist="$tmp/vaiswc-dist"
rm -rf "$vwc_dist"
printf 'one two three\nfour five\n' > "$tmp/vaiswc-a.txt"
printf 'alpha beta gamma delta\n' > "$tmp/vaiswc-b.txt"
expect_exit "vaiswc package build" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e354_vaiswc_package" -o "$vwc_dist"
expect_exit "vaiswc package self-test" 42 "$vwc_dist/bin/vaiswc"
expect_exit "vaiswc single file" 0 "$vwc_dist/bin/vaiswc" "$tmp/vaiswc-a.txt"
expect_exit "vaiswc total row shape" 0 /bin/sh -c "'$vwc_dist/bin/vaiswc' '$tmp/vaiswc-a.txt' '$tmp/vaiswc-b.txt' | tail -1 | grep -qx '3 9 47 total'"
expect_exit "vaiswc stdin word count" 0 /bin/sh -c "printf 'pipe words here\n' | '$vwc_dist/bin/vaiswc' - | grep -qx '1 3 16 -'"
expect_exit "vaiswc missing keeps counting" 3 /bin/sh -c "'$vwc_dist/bin/vaiswc' '$tmp/vaiswc-no-such' '$tmp/vaiswc-b.txt' 2>/dev/null"
expect_exit "grep to wc chain" 0 /bin/sh -c "printf 'cache one\nplain\ncache two three\n' | '$vgrep_dist/bin/vaisgrep' cache - | '$vwc_dist/bin/vaiswc' - | grep -qx '2 7 32 -'"

vsort_dist="$tmp/vaissort-dist"
rm -rf "$vsort_dist"
printf 'banana\napple\ncherry\napple\n' > "$tmp/vaissort-in.txt"
printf 'apple\napple\nbanana\ncherry\n' > "$tmp/vaissort-want-basic.txt"
printf 'apple\nbanana\ncherry\n' > "$tmp/vaissort-want-unique.txt"
printf 'cherry\nbanana\napple\napple\n' > "$tmp/vaissort-want-reverse.txt"
printf 'cherry\nbanana\napple\n' > "$tmp/vaissort-want-ur.txt"
printf 'a\nb\n' > "$tmp/vaissort-want-stdin.txt"
printf '3: cache a\n1: cache b\n' > "$tmp/vaissort-want-chain.txt"
expect_exit "vaissort package build" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e357_vaissort_package" -o "$vsort_dist"
expect_exit "vaissort package self-test" 42 "$vsort_dist/bin/vaissort"
expect_exit "vaissort sorted output" 0 /bin/sh -c "'$vsort_dist/bin/vaissort' '$tmp/vaissort-in.txt' | cmp -s - '$tmp/vaissort-want-basic.txt'"
expect_exit "vaissort unique output" 0 /bin/sh -c "'$vsort_dist/bin/vaissort' -u '$tmp/vaissort-in.txt' | cmp -s - '$tmp/vaissort-want-unique.txt'"
expect_exit "vaissort reverse output" 0 /bin/sh -c "'$vsort_dist/bin/vaissort' -r '$tmp/vaissort-in.txt' | cmp -s - '$tmp/vaissort-want-reverse.txt'"
expect_exit "vaissort unique reverse output" 0 /bin/sh -c "'$vsort_dist/bin/vaissort' -u -r '$tmp/vaissort-in.txt' | cmp -s - '$tmp/vaissort-want-ur.txt'"
expect_exit "vaissort stdin" 0 /bin/sh -c "printf 'b\na\n' | '$vsort_dist/bin/vaissort' - | cmp -s - '$tmp/vaissort-want-stdin.txt'"
expect_exit "vaissort empty stdin" 0 /bin/sh -c "printf '' | '$vsort_dist/bin/vaissort' - | cmp -s - /dev/null"
expect_exit "vaissort missing still sorts" 3 /bin/sh -c "'$vsort_dist/bin/vaissort' '$tmp/vaissort-no-such' '$tmp/vaissort-in.txt' 2>/dev/null"
expect_exit "vaissort missing stdout clean" 0 /bin/sh -c "'$vsort_dist/bin/vaissort' '$tmp/vaissort-no-such' 2>/dev/null | cmp -s - /dev/null"
expect_exit "vaissort usage stdout clean" 0 /bin/sh -c "'$vsort_dist/bin/vaissort' -u 2>/dev/null | cmp -s - /dev/null"
expect_exit "grep to sort chain" 0 /bin/sh -c "printf 'cache b\nplain\ncache a\n' | '$vgrep_dist/bin/vaisgrep' cache - | '$vsort_dist/bin/vaissort' -r - | cmp -s - '$tmp/vaissort-want-chain.txt'"

venv_dist="$tmp/vaisenv-dist"
rm -rf "$venv_dist"
expect_exit "vaisenv package build" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e359_vaisenv_package" -o "$venv_dist"
expect_exit "vaisenv package self-test" 42 "$venv_dist/bin/vaisenv"
expect_exit "vaisenv set variable" 0 /bin/sh -c "VAIS_WF_ENV=hello '$venv_dist/bin/vaisenv' VAIS_WF_ENV | grep -qx 'hello'"
expect_exit "vaisenv unset variable" 3 /bin/sh -c "'$venv_dist/bin/vaisenv' VAIS_WF_ENV_MISSING 2>/dev/null"
expect_exit "vaisenv unset stdout clean" 0 /bin/sh -c "'$venv_dist/bin/vaisenv' VAIS_WF_ENV_MISSING 2>/dev/null | cmp -s - /dev/null"
expect_exit "vaisenv mixed exit" 3 /bin/sh -c "VAIS_WF_ENV=hello '$venv_dist/bin/vaisenv' VAIS_WF_ENV_MISSING VAIS_WF_ENV >/dev/null 2>/dev/null"
expect_exit "vaisenv mixed still prints" 0 /bin/sh -c "VAIS_WF_ENV=hello '$venv_dist/bin/vaisenv' VAIS_WF_ENV_MISSING VAIS_WF_ENV 2>/dev/null | grep -qx 'hello'"
expect_exit "env to grep chain" 1 /bin/sh -c "VAIS_WF_ENV=cache_hit '$venv_dist/bin/vaisenv' VAIS_WF_ENV | '$vgrep_dist/bin/vaisgrep' -c cache -"

vtee_dist="$tmp/vaistee-dist"
rm -rf "$vtee_dist"
printf 'hello\n' > "$tmp/vaistee-want.txt"
printf 'hello\nhello\n' > "$tmp/vaistee-want-double.txt"
expect_exit "vaistee package build" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e362_vaistee_package" -o "$vtee_dist"
expect_exit "vaistee package self-test" 42 /bin/sh -c "'$vtee_dist/bin/vaistee' < /dev/null 2>/dev/null"
expect_exit "vaistee stdout passthrough" 0 /bin/sh -c "printf 'hello\n' | '$vtee_dist/bin/vaistee' '$tmp/vaistee-f1.txt' | cmp -s - '$tmp/vaistee-want.txt'"
expect_exit "vaistee file content" 0 cmp -s "$tmp/vaistee-f1.txt" "$tmp/vaistee-want.txt"
expect_exit "vaistee multi file" 0 /bin/sh -c "printf 'hello\n' | '$vtee_dist/bin/vaistee' '$tmp/vaistee-f2.txt' '$tmp/vaistee-f3.txt' >/dev/null && cmp -s '$tmp/vaistee-f2.txt' '$tmp/vaistee-want.txt' && cmp -s '$tmp/vaistee-f3.txt' '$tmp/vaistee-want.txt'"
expect_exit "vaistee append accumulates" 0 /bin/sh -c "printf 'hello\n' | '$vtee_dist/bin/vaistee' '$tmp/vaistee-f4.txt' >/dev/null && printf 'hello\n' | '$vtee_dist/bin/vaistee' -a '$tmp/vaistee-f4.txt' >/dev/null && cmp -s '$tmp/vaistee-f4.txt' '$tmp/vaistee-want-double.txt'"
expect_exit "vaistee append creates missing" 0 /bin/sh -c "rm -f '$tmp/vaistee-f5.txt' && printf 'new\n' | '$vtee_dist/bin/vaistee' -a '$tmp/vaistee-f5.txt' >/dev/null && printf 'new\n' | cmp -s - '$tmp/vaistee-f5.txt'"
expect_exit "vaistee bad path exit" 3 /bin/sh -c "printf 'x\n' | '$vtee_dist/bin/vaistee' '$tmp/vaistee-no-dir/y.txt' >/dev/null 2>/dev/null"
expect_exit "vaistee bad path keeps stdout" 0 /bin/sh -c "printf 'x\n' | '$vtee_dist/bin/vaistee' '$tmp/vaistee-no-dir/y.txt' 2>/dev/null | grep -qx 'x'"
expect_exit "grep tee wc chain" 0 /bin/sh -c "printf 'cache a\nplain\n' | '$vgrep_dist/bin/vaisgrep' cache - | '$vtee_dist/bin/vaistee' '$tmp/vaistee-f6.txt' | '$vwc_dist/bin/vaiswc' - | grep -qx '1 3 11 -'"
expect_exit "grep tee chain file copy" 0 /bin/sh -c "printf '1: cache a\n' | cmp -s - '$tmp/vaistee-f6.txt'"

vcut_dist="$tmp/vaiscut-dist"
rm -rf "$vcut_dist"
printf 'a,b,c\nplain\nx,,z\n' > "$tmp/vaiscut-in.txt"
printf 'b\nplain\n\n' > "$tmp/vaiscut-want-f2.txt"
printf 'one\ttwo\nsolo\n' > "$tmp/vaiscut-tab.txt"
printf 'two\nsolo\n' > "$tmp/vaiscut-want-tab.txt"
printf '\n\n\n' > "$tmp/vaiscut-want-over.txt"
printf '1\n2\n' > "$tmp/vaiscut-want-chain.txt"
expect_exit "vaiscut package build" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e365_vaiscut_package" -o "$vcut_dist"
expect_exit "vaiscut package self-test" 42 "$vcut_dist/bin/vaiscut"
expect_exit "vaiscut field two" 0 /bin/sh -c "'$vcut_dist/bin/vaiscut' -f 2 -d , '$tmp/vaiscut-in.txt' | cmp -s - '$tmp/vaiscut-want-f2.txt'"
expect_exit "vaiscut tab default" 0 /bin/sh -c "'$vcut_dist/bin/vaiscut' -f 2 '$tmp/vaiscut-tab.txt' | cmp -s - '$tmp/vaiscut-want-tab.txt'"
expect_exit "vaiscut over field empty lines" 0 /bin/sh -c "'$vcut_dist/bin/vaiscut' -f 9 -d , '$tmp/vaiscut-in.txt' | grep -c '' | grep -qx '3'"
expect_exit "vaiscut stdin no files" 0 /bin/sh -c "printf 'k=v\n' | '$vcut_dist/bin/vaiscut' -f 2 -d = | grep -qx 'v'"
expect_exit "vaiscut missing keeps cutting" 3 /bin/sh -c "'$vcut_dist/bin/vaiscut' -f 1 -d , '$tmp/vaiscut-no-such' '$tmp/vaiscut-in.txt' >/dev/null 2>/dev/null"
expect_exit "vaiscut usage without -f" 2 /bin/sh -c "'$vcut_dist/bin/vaiscut' -d , < /dev/null 2>/dev/null"
expect_exit "vaiscut usage stdout clean" 0 /bin/sh -c "'$vcut_dist/bin/vaiscut' -d , < /dev/null 2>/dev/null | cmp -s - /dev/null"
expect_exit "grep cut sort chain" 0 /bin/sh -c "printf 'cache b,2\nplain\ncache a,1\n' | '$vgrep_dist/bin/vaisgrep' cache - | '$vcut_dist/bin/vaiscut' -f 2 -d , | '$vsort_dist/bin/vaissort' - | cmp -s - '$tmp/vaiscut-want-chain.txt'"

# vaislisp: the integer Lisp interpreter package — self-test, a piped
# stdin_read_line REPL with cross-line defun persistence, and file mode
# whose exit code is the last top-level value.
lisp_dist="$tmp/vaislisp-dist"
rm -rf "$lisp_dist"
expect_exit "vaislisp package build" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e393_vaislisp_package" -o "$lisp_dist"
expect_exit "vaislisp package self-test" 42 /bin/sh -c "'$lisp_dist/bin/vaislisp' < /dev/null"
expect_exit "vaislisp repl evaluates piped lines" 0 /bin/sh -c "printf '(+ 1 2)\n(define x 40)\n(+ x 2)\n' | '$lisp_dist/bin/vaislisp' repl | grep -qx '= 42'"
expect_exit "vaislisp repl defun persists across lines" 0 /bin/sh -c "printf '(defun inc (n) (+ n 1))\n(inc 41)\n' | '$lisp_dist/bin/vaislisp' repl | grep -qx '= 42'"
expect_exit "vaislisp file mode prints fib" 0 /bin/sh -c "printf '(defun fib (n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2)))))\n(print (fib 20))\n42\n' > '$tmp/lisp-fib.lisp'; '$lisp_dist/bin/vaislisp' '$tmp/lisp-fib.lisp' | grep -qx '6765'"
expect_exit "vaislisp file mode exit is last value" 42 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$tmp/lisp-fib.lisp' >/dev/null"
expect_exit "vaislisp missing file errors" 3 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$tmp/lisp-none.lisp' >/dev/null"
expect_exit "vaislisp strings pool and compare" 0 /bin/sh -c "printf '(if (= \"ab\" (str-cat \"a\" \"b\")) 42 0)\n' | '$lisp_dist/bin/vaislisp' repl | grep -qx '= 42'"
expect_exit "vaislisp lists render and reduce" 0 /bin/sh -c "printf '(define xs (list 1 2 3))\n(defun lsum (l) (if (null? l) 0 (+ (car l) (lsum (cdr l)))))\n(lsum xs)\n' | '$lisp_dist/bin/vaislisp' repl | grep -qx '= 6'"
expect_exit "vaislisp cons prints list shape" 0 /bin/sh -c "printf '(list 1 2 3)\n' | '$lisp_dist/bin/vaislisp' repl | grep -qx '= (1 2 3)'"
expect_exit "vaislisp lambda composes higher-order" 0 /bin/sh -c "printf '(define twice (lambda (f x) (f (f x))))\n(twice (lambda (n) (* n 3)) 4)\n' | '$lisp_dist/bin/vaislisp' repl | grep -qx '= 36'"
expect_exit "vaislisp let binds and restores shadows" 0 /bin/sh -c "printf '(begin (define x 42) (let ((x 7)) x) x)\n' | '$lisp_dist/bin/vaislisp' repl | grep -qx '= 42'"
expect_exit "vaislisp let bindings see earlier ones" 0 /bin/sh -c "printf '(let ((a 6) (b (* a 7))) b)\n' | '$lisp_dist/bin/vaislisp' repl | grep -qx '= 42'"
expect_exit "vaislisp quote sugar builds data" 0 /bin/sh -c "printf \"'(1 2 3)\\n\" | '$lisp_dist/bin/vaislisp' repl | grep -qx '= (1 2 3)'"
expect_exit "vaislisp quoted symbol interns as string" 0 /bin/sh -c "printf \"(str-len 'hello)\\n\" | '$lisp_dist/bin/vaislisp' repl | grep -qx '= 5'"
expect_exit "vaislisp str-ref and str-byte index strings" 0 /bin/sh -c "printf '(str-cat (str-ref \"vais\" 0) \"ok\")\n(str-byte \"abc\" 2)\n' | '$lisp_dist/bin/vaislisp' repl | grep -qx '= 99'"
expect_exit "vaislisp cond picks first truthy clause" 0 /bin/sh -c "printf '(cond ((= 1 2) 7) ((> 3 1) 42) (else 9))\n' | '$lisp_dist/bin/vaislisp' repl | grep -qx '= 42'"
expect_exit "vaislisp and or short-circuit with values" 0 /bin/sh -c "printf '(begin (define t 40) (and 0 (set t 0)) (or 1 (set t 0)) (+ t 2))\n' | '$lisp_dist/bin/vaislisp' repl | grep -qx '= 42'"
expect_exit "vaislisp if without else is nil" 0 /bin/sh -c "printf '(if 0 7)\n' | '$lisp_dist/bin/vaislisp' repl | grep -qx '= nil'"
# The programs/ corpus: real .lisp files run in file mode, each locking
# printed output and a computed exit of 42.
lisp_progs="$ROOT/examples/e393_vaislisp_package/programs"
printf '1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz\n16\n17\nFizz\n19\nBuzz\n' > "$tmp/want-fizzbuzz.txt"
expect_exit "vaislisp program fizzbuzz output" 0 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$lisp_progs/fizzbuzz.lisp' | cmp -s - '$tmp/want-fizzbuzz.txt'"
expect_exit "vaislisp program fizzbuzz exit" 42 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$lisp_progs/fizzbuzz.lisp' >/dev/null"
expect_exit "vaislisp program vowels output" 0 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$lisp_progs/vowels.lisp' | grep -qx '10'"
expect_exit "vaislisp program vowels exit" 42 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$lisp_progs/vowels.lisp' >/dev/null"
expect_exit "vaislisp program assoc output" 0 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$lisp_progs/assoc.lisp' | grep -qx '(beta 22)'"
expect_exit "vaislisp program assoc exit" 42 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$lisp_progs/assoc.lisp' >/dev/null"
expect_exit "vaislisp program hof output" 0 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$lisp_progs/hof.lisp' | grep -qx '(1 4 9 16)'"
expect_exit "vaislisp program hof exit" 42 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$lisp_progs/hof.lisp' >/dev/null"
expect_exit "vaislisp program sort output" 0 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$lisp_progs/sort.lisp' | grep -qx '(1 2 3 5 8 9)'"
expect_exit "vaislisp program sort exit" 42 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$lisp_progs/sort.lisp' >/dev/null"
expect_exit "vaislisp program words output" 0 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$lisp_progs/words.lisp' | grep -qx '(vais lisp dog food)'"
expect_exit "vaislisp program words exit" 42 /bin/sh -c "'$lisp_dist/bin/vaislisp' '$lisp_progs/words.lisp' >/dev/null"
expect_exit "vaislisp repl exits clean at EOF" 0 /bin/sh -c "printf '(str-cat \"a\" \"b\")\n' | '$lisp_dist/bin/vaislisp' repl >/dev/null"

# vaisjq: the integer JSON parser / jq-subset query tool — self-test,
# path queries, iteration, pipe tails, both render modes, and the
# loud parse/type/file error exits.
jq_dist="$tmp/vaisjq-dist"
rm -rf "$jq_dist"
expect_exit "vaisjq package build" 0 "$ROOT/scripts/vaisc" package "$ROOT/examples/e396_vaisjq_package" -o "$jq_dist"
expect_exit "vaisjq package self-test" 42 /bin/sh -c "'$jq_dist/bin/vaisjq' < /dev/null"
printf '{"users":[{"name":"ann","age":30},{"name":"bo","age":25}],"total":2}' > "$tmp/vaisjq-t.json"
expect_exit "vaisjq field chain" 0 /bin/sh -c "'$jq_dist/bin/vaisjq' -c '.users[0].name' '$tmp/vaisjq-t.json' | grep -qx '\"ann\"'"
expect_exit "vaisjq iteration emits per element" 0 /bin/sh -c "'$jq_dist/bin/vaisjq' '.users[].age' '$tmp/vaisjq-t.json' | tr '\n' ' ' | grep -q '30 25 '"
expect_exit "vaisjq keys sorts" 0 /bin/sh -c "'$jq_dist/bin/vaisjq' -c '. | keys' '$tmp/vaisjq-t.json' | grep -qx '\[\"total\",\"users\"\]'"
expect_exit "vaisjq length via stdin" 0 /bin/sh -c "printf '{\"a\":[1,2,3]}' | '$jq_dist/bin/vaisjq' '.a | length' | grep -qx '3'"
expect_exit "vaisjq pretty renders nested" 0 /bin/sh -c "printf '{\"b\":{\"c\":[1]}}' | '$jq_dist/bin/vaisjq' '.' | grep -qx '      1'"
expect_exit "vaisjq missing field is null" 0 /bin/sh -c "printf '{\"a\":1}' | '$jq_dist/bin/vaisjq' '.nope' | grep -qx 'null'"
expect_exit "vaisjq parse error exits loud" 3 /bin/sh -c "printf '{bad' | '$jq_dist/bin/vaisjq' '.' >/dev/null 2>&1"
expect_exit "vaisjq float rejects loud" 3 /bin/sh -c "printf '1.5' | '$jq_dist/bin/vaisjq' '.' >/dev/null 2>&1"
expect_exit "vaisjq type error exits loud" 3 /bin/sh -c "printf '[1]' | '$jq_dist/bin/vaisjq' '.a' >/dev/null 2>&1"
expect_exit "vaisjq missing file exits loud" 3 /bin/sh -c "'$jq_dist/bin/vaisjq' '.' '$tmp/vaisjq-none.json' < /dev/null >/dev/null 2>&1"
expect_exit "vaisjq select ordered filters" 0 /bin/sh -c "'$jq_dist/bin/vaisjq' '.users[] | select(.age > 26) | .name' '$tmp/vaisjq-t.json' | tr '\n' ' ' | grep -q '\"ann\" '"
expect_exit "vaisjq select string equality" 0 /bin/sh -c "'$jq_dist/bin/vaisjq' -c '.users[] | select(.name == \"bo\") | .age' '$tmp/vaisjq-t.json' | grep -qx '25'"
expect_exit "vaisjq negative index from end" 0 /bin/sh -c "'$jq_dist/bin/vaisjq' -c '.users[-1].name' '$tmp/vaisjq-t.json' | grep -qx '\"bo\"'"
expect_exit "vaisjq pipe stages concatenate" 0 /bin/sh -c "printf '{\"a\":{\"b\":[1,2,3]}}' | '$jq_dist/bin/vaisjq' '.a | .b | length' | grep -qx '3'"
expect_exit "vaisjq select iteration rejects loud" 3 /bin/sh -c "printf '[1]' | '$jq_dist/bin/vaisjq' '.[] | select(.a[] > 1)' >/dev/null 2>&1"
expect_exit "vaisjq map projects arrays" 0 /bin/sh -c "'$jq_dist/bin/vaisjq' -c '.users | map(.name)' '$tmp/vaisjq-t.json' | grep -qx '\[\"ann\",\"bo\"\]'"
expect_exit "vaisjq has checks keys" 0 /bin/sh -c "'$jq_dist/bin/vaisjq' '.users[0] | has(\"age\")' '$tmp/vaisjq-t.json' | grep -qx 'true'"
expect_exit "vaisjq first and last are index sugar" 0 /bin/sh -c "'$jq_dist/bin/vaisjq' -c '.users | last | .name' '$tmp/vaisjq-t.json' | grep -qx '\"bo\"'"
expect_exit "vaisjq select and or with precedence" 0 /bin/sh -c "'$jq_dist/bin/vaisjq' '.users[] | select(.age > 29 and .name == \"ann\" or .age < 26) | .age' '$tmp/vaisjq-t.json' | tr '\n' ' ' | grep -q '30 25 '"
expect_exit "vaisjq map on non-array rejects loud" 3 /bin/sh -c "printf '{\"a\":1}' | '$jq_dist/bin/vaisjq' '. | map(.a)' >/dev/null 2>&1"

vbox_dist="$tmp/vaisbox-dist"
vbox_bin="$tmp/vaisbox-bin"
rm -rf "$vbox_dist" "$vbox_bin"
mkdir -p "$vbox_bin"
"$ROOT/scripts/vaisc" package "$ROOT/examples/e355_vaisbox_package" -o "$vbox_dist" >/dev/null
vfind_dist="$tmp/vaisfind-dist"
vfreq_dist="$tmp/vaisfreq-dist"
rm -rf "$vfind_dist" "$vfreq_dist"
"$ROOT/scripts/vaisc" package "$ROOT/examples/e381_vaisfind_package" -o "$vfind_dist" >/dev/null
"$ROOT/scripts/vaisc" package "$ROOT/examples/e390_vaisfreq_package" -o "$vfreq_dist" >/dev/null
cp "$vbox_dist/bin/vaisbox" "$vbox_bin/vaisbox"
cp "$vwc_dist/bin/vaiswc" "$vbox_bin/vaiswc"
cp "$vgrep_dist/bin/vaisgrep" "$vbox_bin/vaisgrep"
cp "$vsort_dist/bin/vaissort" "$vbox_bin/vaissort"
cp "$venv_dist/bin/vaisenv" "$vbox_bin/vaisenv"
cp "$vtee_dist/bin/vaistee" "$vbox_bin/vaistee"
cp "$vcut_dist/bin/vaiscut" "$vbox_bin/vaiscut"
cp "$vfind_dist/bin/vaisfind" "$vbox_bin/vaisfind"
cp "$vfreq_dist/bin/vaisfreq" "$vbox_bin/vaisfreq"
cp "$lisp_dist/bin/vaislisp" "$vbox_bin/vaislisp"
cp "$jq_dist/bin/vaisjq" "$vbox_bin/vaisjq"
mkdir -p "$tmp/vaisbox-find-tree/sub"
printf 'x\n' > "$tmp/vaisbox-find-tree/notes.txt"
printf 'x\n' > "$tmp/vaisbox-find-tree/sub/notes.md"
printf 'one two\nthree\n' > "$tmp/vaisbox-in.txt"
expect_exit "vaisbox package self-test" 42 "$vbox_dist/bin/vaisbox"
expect_exit "vaisbox list count" 0 /bin/sh -c "'$vbox_bin/vaisbox' list | wc -l | grep -q 15"
expect_exit "vaisbox dispatch vaisfreq" 0 /bin/sh -c "printf 'ai cache ai\n' | '$vbox_bin/vaisbox' vaisfreq -n 1 - | grep -qx '2 ai'"
expect_exit "vaisbox dispatch vaislisp" 0 /bin/sh -c "printf '(+ 40 2)\n' | '$vbox_bin/vaisbox' vaislisp repl | grep -qx '= 42'"
expect_exit "vaisbox dispatch vaisjq" 0 /bin/sh -c "printf '{\"n\":42}' | '$vbox_bin/vaisbox' vaisjq '.n' | grep -qx '42'"
expect_exit "vaisbox dispatch vaisfind" 2 /bin/sh -c "'$vbox_bin/vaisbox' vaisfind notes '$tmp/vaisbox-find-tree' < /dev/null | grep -q 'sub/notes.md' && exit 2"
expect_exit "vaisbox dispatch vaiscut" 0 /bin/sh -c "printf 'k=v\n' | '$vbox_bin/vaisbox' vaiscut -f 2 -d = | grep -qx 'v'"
expect_exit "vaisbox dispatch vaistee" 0 /bin/sh -c "printf 'b\n' | '$vbox_bin/vaisbox' vaistee '$tmp/vaisbox-tee.txt' | grep -qx 'b'"
expect_exit "vaisbox dispatch vaisenv" 0 /bin/sh -c "VAIS_WF_ENV=hello '$vbox_bin/vaisbox' vaisenv VAIS_WF_ENV < /dev/null | grep -qx 'hello'"
expect_exit "vaisbox dispatch vaissort" 0 /bin/sh -c "printf 'b\na\n' | '$vbox_bin/vaisbox' vaissort - | cmp -s - '$tmp/vaissort-want-stdin.txt'"
expect_exit "vaisbox dispatch vaiswc" 0 /bin/sh -c "'$vbox_bin/vaisbox' vaiswc '$tmp/vaisbox-in.txt' < /dev/null | grep -qx '2 3 14 $tmp/vaisbox-in.txt'"
expect_exit "vaisbox dispatch grep pipe" 2 /bin/sh -c "printf 'cache x\nplain\ncache y\n' | '$vbox_bin/vaisbox' vaisgrep -c cache -"
expect_exit "vaisbox unknown tool" 2 /bin/sh -c "'$vbox_bin/vaisbox' nope < /dev/null 2>/dev/null"
expect_exit "vaisbox missing sibling guard" 3 /bin/sh -c "'$vbox_dist/bin/vaisbox' vaisdb x < /dev/null 2>/dev/null"

overflow_src="$tmp/list-cap-overflow.vais"
cat > "$overflow_src" <<'VAIS'
fn main() -> Int {
    let b = str_builder_new()
    let mut k = 0
    while k < 4200 {
        let r1 = str_builder_append(b, "x")
        let r2 = str_builder_push(b, 10)
        k = k + 1
    }
    let text = str_builder_finish(b)
    let lines: List<Str> = []
    let n = str_split_lines_into(text, lines)
    return n
}
VAIS
expect_exit "list cap overflow full build" 0 "$ROOT/scripts/vaisc" build "$overflow_src" -o "$tmp/list-cap-overflow-full"
expect_exit "list cap overflow full traps loud" 134 "$tmp/list-cap-overflow-full"
expect_exit "list cap overflow direct build" 0 "$ROOT/scripts/vaisc" build "$overflow_src" --engine direct -o "$tmp/list-cap-overflow-direct"
expect_exit "list cap overflow direct traps loud" 134 "$tmp/list-cap-overflow-direct"
empty_pop_src="$tmp/list-empty-pop.vais"
cat > "$empty_pop_src" <<'VAIS'
fn main() -> Int {
    let xs: List<Int> = []
    xs.pop()
    return 0
}
VAIS
expect_exit "list empty pop full build" 0 "$ROOT/scripts/vaisc" build "$empty_pop_src" -o "$tmp/list-empty-pop-full"
expect_exit "list empty pop full traps loud" 134 "$tmp/list-empty-pop-full"
expect_exit "list empty pop direct build" 0 "$ROOT/scripts/vaisc" build "$empty_pop_src" --engine direct -o "$tmp/list-empty-pop-direct"
expect_exit "list empty pop direct traps loud" 134 "$tmp/list-empty-pop-direct"

huge_docs_dir="$tmp/vaisdb-huge-docs"
huge_idx="$tmp/vaisdb-huge-idx"
rm -rf "$huge_docs_dir" "$huge_idx"
mkdir -p "$huge_docs_dir"
seq -s ' ' 0 4200 > "$huge_docs_dir/big.txt"
printf 'alpha beta\n' > "$huge_docs_dir/ok1.txt"
printf 'beta gamma\n' > "$huge_docs_dir/ok2.txt"
expect_exit "vaisdb big-vocab doc ingests in batch" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' ingest-dir '$huge_idx' '$huge_docs_dir' 2>/dev/null | grep -qx 'ingested 3 documents'"
expect_exit "vaisdb big-vocab ingest reports no skip" 0 /bin/sh -c "rm -rf '$huge_idx'; '$vdb_dist/bin/vaisdb' ingest-dir '$huge_idx' '$huge_docs_dir' 2>&1 >/dev/null | grep -q 'skipped (too large)' && exit 1; exit 0"
expect_exit "vaisdb big-vocab doc searchable" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$huge_idx' 2100 2 | grep -q '^1\. big=1 '"
expect_exit "vaisdb big-vocab index top works" 5 /bin/sh -c "'$vdb_dist/bin/vaisdb' top '$huge_idx' 5 >/dev/null"
expect_exit "vaisdb batch survivors queryable" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' stats '$huge_idx' | grep -qx 'docs=3 postings=4205'"

search_docs="$tmp/vaisdb-search-docs"
search_idx="$tmp/vaisdb-search-idx"
rm -rf "$search_docs" "$search_idx"
mkdir -p "$search_docs"
printf 'plain filler line\ncache hit line here\n' > "$search_docs/hits.txt"
printf 'markdown body without match\n' > "$search_docs/note.md"
expect_exit "vaisdb search ingest mixed extensions" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' ingest-dir '$search_idx' '$search_docs' | grep -qx 'ingested 2 documents'"
expect_exit "vaisdb search md doc listed" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' docs '$search_idx' | grep -qx 'note'"
expect_exit "vaisdb search ranks and snippets" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$search_idx' cache 3 | grep -q '^1\. hits=1 '"
expect_exit "vaisdb search snippet highlights the term" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$search_idx' cache 3 | grep -qx '    \[cache\] hit line here'"
expect_exit "vaisdb search snippet highlights every query term" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$search_idx' 'cache line' 3 | grep -qx '    plain filler \[line\]'"
expect_exit "vaisdb search hides zero scores" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$search_idx' zzzabsent 3 | cmp -s - /dev/null"

reindex_docs="$tmp/vaisdb-reindex-docs"
reindex_idx="$tmp/vaisdb-reindex-idx"
rm -rf "$reindex_docs" "$reindex_idx"
mkdir -p "$reindex_docs"
printf 'alpha beta\n' > "$reindex_docs/r1.txt"
printf 'gamma delta\n' > "$reindex_docs/r2.txt"
touch -t 202001010000 "$reindex_docs/r1.txt"
expect_exit "vaisdb reindex adds fresh docs" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$reindex_idx' '$reindex_docs' | grep -qx 'reindexed added=2 updated=0 removed=0 skipped=0'"
expect_exit "vaisdb reindex skips unchanged" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$reindex_idx' '$reindex_docs' | grep -qx 'reindexed added=0 updated=0 removed=0 skipped=2'"
printf 'alpha beta epsilon\n' > "$reindex_docs/r1.txt"
expect_exit "vaisdb reindex updates changed doc" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$reindex_idx' '$reindex_docs' | grep -qx 'reindexed added=0 updated=1 removed=0 skipped=1'"
expect_exit "vaisdb reindex refreshed content searchable" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$reindex_idx' epsilon 2 | grep -q '^1\. r1=1 '"
expect_exit "vaisdb reindex stats stay exact" 0 /bin/sh -c "out=\$('$vdb_dist/bin/vaisdb' stats '$reindex_idx' 2>/dev/null); [ \"\$out\" = 'docs=2 postings=5' ]"
expect_exit "vaisdb why breaks down the score" 3 "$vdb_dist/bin/vaisdb" why "$reindex_idx" "alpha epsilon epsilon" r1
expect_exit "vaisdb why per-term line" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' why '$reindex_idx' 'alpha epsilon epsilon' r1 | grep -qx '  epsilon q=2 doc=1 adds 2'"
expect_exit "vaisdb why total line" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' why '$reindex_idx' 'alpha epsilon epsilon' r1 | grep -qx 'score=3'"
expect_exit "vaisdb why absent term shows zero" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' why '$reindex_idx' 'alpha zzz' r1 | grep -qx '  zzz q=1 doc=0 adds 0'"
expect_exit "vaisdb why unknown doc errors" 3 /bin/sh -c "'$vdb_dist/bin/vaisdb' why '$reindex_idx' alpha ghost | grep -qx 'error: doc not found' && exit 3"

msearch_docs="$tmp/vaisdb-msearch-docs"
msearch_idx="$tmp/vaisdb-msearch-idx"
rm -rf "$msearch_docs" "$msearch_idx"
mkdir -p "$msearch_docs"
printf 'cache cache beta\n' > "$msearch_docs/m1.txt"
expect_exit "vaisdb msearch second index builds" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$msearch_idx' '$msearch_docs' | grep -qx 'reindexed added=1 updated=0 removed=0 skipped=0'"
expect_exit "vaisdb msearch exit is best merged score" 2 "$vdb_dist/bin/vaisdb" msearch cache 3 "$msearch_idx" "$search_idx"
expect_exit "vaisdb msearch top line labels source index" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' msearch cache 3 '$msearch_idx' '$search_idx' | grep -q '^1\. vaisdb-msearch-idx/m1=2 '"
expect_exit "vaisdb msearch merges the other index" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' msearch cache 3 '$msearch_idx' '$search_idx' | grep -q '^2\. vaisdb-search-idx/'"
expect_exit "vaisdb msearch hides zero scores" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' msearch zzzabsent 3 '$msearch_idx' '$search_idx' | cmp -s - /dev/null"
expect_exit "vaisdb msearch missing index errors" 3 /bin/sh -c "'$vdb_dist/bin/vaisdb' msearch cache 3 '$msearch_idx' '$tmp/vaisdb-msearch-missing' | grep -q 'error: index not found' && exit 3"

similar_docs="$tmp/vaisdb-similar-docs"
similar_idx="$tmp/vaisdb-similar-idx"
rm -rf "$similar_docs" "$similar_idx"
mkdir -p "$similar_docs"
printf 'alpha beta gamma\n' > "$similar_docs/s1.txt"
printf 'alpha beta\n' > "$similar_docs/s2.txt"
printf 'zeta\n' > "$similar_docs/s3.txt"
expect_exit "vaisdb similar index builds" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$similar_idx' '$similar_docs' | grep -qx 'reindexed added=3 updated=0 removed=0 skipped=0'"
expect_exit "vaisdb similar exit is shown count" 1 "$vdb_dist/bin/vaisdb" similar "$similar_idx" s1
expect_exit "vaisdb similar top row excludes self" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' similar '$similar_idx' s1 | grep -qx '1. s2=2'"
expect_exit "vaisdb similar no overlap prints nothing" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' similar '$similar_idx' s3 | cmp -s - /dev/null"
expect_exit "vaisdb similar unknown doc errors" 3 /bin/sh -c "'$vdb_dist/bin/vaisdb' similar '$similar_idx' ghost | grep -qx 'error: doc not found' && exit 3"

top_docs="$tmp/vaisdb-top-docs"
top_idx="$tmp/vaisdb-top-idx"
rm -rf "$top_docs" "$top_idx"
mkdir -p "$top_docs"
printf 'cache cache cache alpha\n' > "$top_docs/w1.txt"
printf 'cache alpha beta\n' > "$top_docs/w2.txt"
expect_exit "vaisdb top index builds" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$top_idx' '$top_docs' | grep -qx 'reindexed added=2 updated=0 removed=0 skipped=0'"
expect_exit "vaisdb top exit is shown count" 3 "$vdb_dist/bin/vaisdb" top "$top_idx"
expect_exit "vaisdb top leading term aggregates shards" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' top '$top_idx' | grep -qx '1. cache=4'"
expect_exit "vaisdb top honors k" 0 /bin/sh -c "out=\$('$vdb_dist/bin/vaisdb' top '$top_idx' 2); [ \"\$out\" = '1. cache=4
2. alpha=2' ]"
expect_exit "vaisdb top missing index errors" 3 /bin/sh -c "'$vdb_dist/bin/vaisdb' top '$tmp/vaisdb-top-missing' | grep -qx 'error: index not found' && exit 3"

phrase_docs="$tmp/vaisdb-phrase-docs"
phrase_idx="$tmp/vaisdb-phrase-idx"
rm -rf "$phrase_docs" "$phrase_idx"
mkdir -p "$phrase_docs"
printf 'big cache win big cache win\n' > "$phrase_docs/p1.txt"
printf 'big\ncache\n' > "$phrase_docs/p2.txt"
printf 'cache big\n' > "$phrase_docs/p3.txt"
expect_exit "vaisdb phrase index builds" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$phrase_idx' '$phrase_docs' | grep -qx 'reindexed added=3 updated=0 removed=0 skipped=0'"
expect_exit "vaisdb phrase exit is shown count" 2 "$vdb_dist/bin/vaisdb" phrase "$phrase_idx" "big cache"
expect_exit "vaisdb phrase ranks occurrences and spans newlines" 0 /bin/sh -c "out=\$('$vdb_dist/bin/vaisdb' phrase '$phrase_idx' 'big cache'); [ \"\$out\" = '1. p1=2
2. p2=1' ]"
expect_exit "vaisdb phrase word order matters" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' phrase '$phrase_idx' 'win big cache' | grep -qx '1. p1=1'"
expect_exit "vaisdb phrase empty phrase errors" 1 /bin/sh -c "'$vdb_dist/bin/vaisdb' phrase '$phrase_idx' '   ' | grep -qx 'error: phrase has no terms' && exit 1"

# Partial-failure invariants: a crash mid-ingest leaves either duplicate
# postings (retry after the shard appends) or orphan postings (no
# registry commit). Queries must stay exact through both, rewrites must
# never leave temp litter behind, and stale temp litter must never leak
# into results.
atomic_docs="$tmp/vaisdb-atomic-docs"
atomic_idx="$tmp/vaisdb-atomic-idx"
rm -rf "$atomic_docs" "$atomic_idx"
mkdir -p "$atomic_docs"
printf 'alpha beta alpha\n' > "$atomic_docs/a1.txt"
printf 'beta gamma\n' > "$atomic_docs/a2.txt"
touch -t 202001010000 "$atomic_docs/a1.txt"
expect_exit "vaisdb atomic index builds" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$atomic_idx' '$atomic_docs' | grep -qx 'reindexed added=2 updated=0 removed=0 skipped=0'"
expect_exit "vaisdb atomic baseline score" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$atomic_idx' alpha 3 | grep -q '^1\. a1=2 '"
alpha_pat="$(printf 'alpha\ta1\t')"
expect_exit "vaisdb atomic duplicate posting keeps scores exact" 0 /bin/sh -c "shard=\$(grep -l \"^$alpha_pat\" '$atomic_idx/terms/'*.txt); grep -h \"^$alpha_pat\" \"\$shard\" >> \"\$shard\"; '$vdb_dist/bin/vaisdb' search '$atomic_idx' alpha 3 | grep -q '^1\. a1=2 '"
expect_exit "vaisdb atomic why agrees over duplicates" 2 "$vdb_dist/bin/vaisdb" why "$atomic_idx" alpha a1
expect_exit "vaisdb atomic orphan postings stay invisible" 0 /bin/sh -c "shard=\$(grep -l \"^$alpha_pat\" '$atomic_idx/terms/'*.txt); printf 'alpha\tghost\t9\n' >> \"\$shard\"; ! '$vdb_dist/bin/vaisdb' search '$atomic_idx' alpha 5 | grep -q ghost"
expect_exit "vaisdb atomic ranking unchanged beside orphans" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$atomic_idx' alpha 3 | grep -q '^1\. a1=2 '"
printf 'alpha beta alpha\n' > "$atomic_docs/a1.txt"
expect_exit "vaisdb atomic reindex converges over litter" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$atomic_idx' '$atomic_docs' | grep -qx 'reindexed added=0 updated=1 removed=0 skipped=1'"
expect_exit "vaisdb atomic converged score stays exact" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$atomic_idx' alpha 3 | grep -q '^1\. a1=2 '"
expect_exit "vaisdb atomic remove leaves no temp litter" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' remove '$atomic_idx' a2 >/dev/null; ls '$atomic_idx/terms/'*.txt.tmp 2>/dev/null && exit 1; test -f '$atomic_idx/docs.txt.tmp' && exit 1; exit 0"
expect_exit "vaisdb atomic stale temp litter is ignored" 0 /bin/sh -c "printf 'junk with no tabs\n' > '$atomic_idx/terms/s5.txt.tmp'; '$vdb_dist/bin/vaisdb' search '$atomic_idx' alpha 3 | grep -q '^1\. a1=2 '"

# Recursive directory walks: `-r` collects subdirectory documents under
# relative-path doc ids while the flat default keeps its historical
# top-level-only contract; the incremental reindex semantics (skip /
# update / deletion sync) must hold for subdirectory files too.
recur_docs="$tmp/vaisdb-recur-docs"
recur_flat_idx="$tmp/vaisdb-recur-flat-idx"
recur_idx="$tmp/vaisdb-recur-idx"
rm -rf "$recur_docs" "$recur_flat_idx" "$recur_idx"
mkdir -p "$recur_docs/sub/deep"
printf 'alpha beta\n' > "$recur_docs/r1.txt"
printf 'beta gamma\n' > "$recur_docs/sub/s1.txt"
printf 'gamma delta\n' > "$recur_docs/sub/deep/d1.md"
touch -t 202001010000 "$recur_docs/sub/s1.txt"
expect_exit "vaisdb flat reindex stays top-level" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$recur_flat_idx' '$recur_docs' | grep -qx 'reindexed added=1 updated=0 removed=0 skipped=0'"
expect_exit "vaisdb recursive reindex walks subdirs" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$recur_idx' '$recur_docs' -r | grep -qx 'reindexed added=3 updated=0 removed=0 skipped=0'"
expect_exit "vaisdb recursive doc ids carry relative paths" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' docs '$recur_idx' | grep -qx 'sub/deep/d1'"
expect_exit "vaisdb recursive search hits subdir docs" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$recur_idx' gamma 3 | grep -q '^1\. sub/'"
expect_exit "vaisdb recursive warm reindex skips all" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$recur_idx' '$recur_docs' -r | grep -qx 'reindexed added=0 updated=0 removed=0 skipped=3'"
printf 'beta gamma epsilon\n' > "$recur_docs/sub/s1.txt"
expect_exit "vaisdb recursive update tracks subdir mtime" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$recur_idx' '$recur_docs' -r | grep -qx 'reindexed added=0 updated=1 removed=0 skipped=2'"
expect_exit "vaisdb recursive updated subdir content searchable" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$recur_idx' epsilon 2 | grep -q '^1\. sub/s1=1 '"
rm "$recur_docs/sub/deep/d1.md"
expect_exit "vaisdb recursive deletion sync reaches subdirs" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$recur_idx' '$recur_docs' -r | grep -qx 'reindexed added=0 updated=0 removed=1 skipped=2'"
expect_exit "vaisdb recursive removed subdir doc leaves search" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$recur_idx' delta 2 | cmp -s - /dev/null"
expect_exit "vaisdb recursive ingest-dir walks subdirs" 0 /bin/sh -c "rm -rf '$tmp/vaisdb-recur-ing'; '$vdb_dist/bin/vaisdb' ingest-dir '$tmp/vaisdb-recur-ing' '$recur_docs' -r | grep -qx 'ingested 2 documents'"
expect_exit "vaisdb reindex rejects unknown flag" 1 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$recur_idx' '$recur_docs' -x >/dev/null 2>&1"

# Search tokenization splits compounds on non-word ASCII bytes for
# index and query alike, similar sources partition per shard past the
# whole-document window, and top's -min filter drops short terms
# explicitly (stopword filtering stays user-side).
tok_docs="$tmp/vaisdb-tok-docs"
tok_idx="$tmp/vaisdb-tok-idx"
rm -rf "$tok_docs" "$tok_idx"
mkdir -p "$tok_docs"
printf 'the test-fixpoint-full gate uses fixpoint_full.vais here\n' > "$tok_docs/tok.txt"
expect_exit "vaisdb tokenizer index builds" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$tok_idx' '$tok_docs' | grep -qx 'reindexed added=1 updated=0 removed=0 skipped=0'"
expect_exit "vaisdb tokenizer splits indexed compounds" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$tok_idx' fixpoint 2 | grep -q '^1\. tok=2 '"
expect_exit "vaisdb tokenizer splits query compounds" 6 /bin/sh -c "'$vdb_dist/bin/vaisdb' query '$tok_idx' tok 'test-fixpoint-full.vais' >/dev/null"
simbig_docs="$tmp/vaisdb-simbig-docs"
simbig_idx="$tmp/vaisdb-simbig-idx"
rm -rf "$simbig_docs" "$simbig_idx"
mkdir -p "$simbig_docs"
seq -s ' ' 0 4500 > "$simbig_docs/big.txt"
seq -s ' ' 4000 8500 > "$simbig_docs/big2.txt"
printf 'zeta only\n' > "$simbig_docs/small.txt"
expect_exit "vaisdb similar big-source index builds" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' ingest-dir '$simbig_idx' '$simbig_docs' | grep -qx 'ingested 3 documents'"
expect_exit "vaisdb similar handles big-vocab sources" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' similar '$simbig_idx' big 2 | grep -q '^1\. big2=501'"
expect_exit "vaisdb top -min filters short terms" 3 /bin/sh -c "'$vdb_dist/bin/vaisdb' top '$simbig_idx' 3 -min 4 >/dev/null"
expect_exit "vaisdb top -min output excludes short terms" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' top '$simbig_idx' 3 -min 4 | grep -q '^[0-9]\. [0-9]\{1,3\}=' && exit 1; exit 0"

# search -all requires every query term: partial matches drop while
# survivors keep the plain contribution sum (OR stays the default).
printf 'gate here alone\n' > "$tok_docs/tok2.txt"
expect_exit "vaisdb -all corpus grows" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$tok_idx' '$tok_docs' | grep -qx 'reindexed added=1 updated=0 removed=0 skipped=1'"
expect_exit "vaisdb search OR keeps partial matches" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$tok_idx' 'fixpoint gate' 3 | grep -q '^2\. tok2=1'"
expect_exit "vaisdb search -all drops partial matches" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$tok_idx' 'fixpoint gate' 3 -all | grep -q 'tok2' && exit 1; exit 0"
expect_exit "vaisdb search -all ranks by rarity-weighted counts" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$tok_idx' 'fixpoint gate' 3 -all | grep -q '^1\. tok=5 '"
expect_exit "vaisdb search -all exit is top score" 5 "$vdb_dist/bin/vaisdb" search "$tok_idx" "fixpoint gate" 3 -all

rm "$reindex_docs/r2.txt"
expect_exit "vaisdb reindex removes deleted docs" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' reindex '$reindex_idx' '$reindex_docs' | grep -qx 'reindexed added=0 updated=0 removed=1 skipped=1'"
expect_exit "vaisdb reindex removed doc leaves search" 0 /bin/sh -c "'$vdb_dist/bin/vaisdb' search '$reindex_idx' gamma 2 | cmp -s - /dev/null"
expect_exit "vaisdb reindex stats drop removed doc" 0 /bin/sh -c "out=\$('$vdb_dist/bin/vaisdb' stats '$reindex_idx' 2>/dev/null); [ \"\$out\" = 'docs=1 postings=3' ]"

map_cap_src="$tmp/map-cap-boundary.vais"
cat > "$map_cap_src" <<'VAIS'
fn main() -> Int {
    let m: Map<Int,Int> = {}
    let mut i = 0
    while i < 4096 {
        m.insert(i, i)
        i = i + 1
    }
    if m.len() != 4096 { return 1 }
    return 0
}
VAIS
map_over_src="$tmp/map-cap-overflow.vais"
cat > "$map_over_src" <<'VAIS'
fn main() -> Int {
    let m: Map<Int,Int> = {}
    let mut i = 0
    while i < 4097 {
        m.insert(i, i)
        i = i + 1
    }
    return m.len()
}
VAIS
expect_exit "map cap 4096 full build" 0 "$ROOT/scripts/vaisc" build "$map_cap_src" -o "$tmp/map-cap-full"
expect_exit "map cap 4096 full holds" 0 "$tmp/map-cap-full"
expect_exit "map cap 4096 direct build" 0 "$ROOT/scripts/vaisc" build "$map_cap_src" --engine direct -o "$tmp/map-cap-direct"
expect_exit "map cap 4096 direct holds" 0 "$tmp/map-cap-direct"
expect_exit "map cap overflow full build" 0 "$ROOT/scripts/vaisc" build "$map_over_src" -o "$tmp/map-over-full"
expect_exit "map cap overflow full traps loud" 134 "$tmp/map-over-full"
expect_exit "map cap overflow direct build" 0 "$ROOT/scripts/vaisc" build "$map_over_src" --engine direct -o "$tmp/map-over-direct"
expect_exit "map cap overflow direct traps loud" 134 "$tmp/map-over-direct"

range_trap_src="$tmp/str-range-trap.vais"
cat > "$range_trap_src" <<'VAIS'
fn main() -> Int {
    let s = str_slice("abc", 0, 99)
    return s.len()
}
VAIS
expect_exit "str range trap full build" 0 "$ROOT/scripts/vaisc" build "$range_trap_src" -o "$tmp/str-range-trap-full"
expect_exit "str range trap full loud" 134 "$tmp/str-range-trap-full"
expect_exit "str range trap full message" 0 /bin/sh -c "'$tmp/str-range-trap-full' 2>&1 | grep -q 'vais str trap: slice or byte out of range'"
expect_exit "str range trap direct build" 0 "$ROOT/scripts/vaisc" build "$range_trap_src" --engine direct -o "$tmp/str-range-trap-direct"
expect_exit "str range trap direct loud" 134 "$tmp/str-range-trap-direct"
expect_exit "str range trap direct message" 0 /bin/sh -c "'$tmp/str-range-trap-direct' 2>&1 | grep -q 'vais str trap: slice or byte out of range'"
expect_exit "vaisdb package archive exists" 0 test -f "$vdb_dist/vaisdb-0.1.0.tar.gz"
mkdir -p "$vdb_extract"
expect_exit "vaisdb package archive extracts" 0 tar -C "$vdb_extract" -xzf "$vdb_dist/vaisdb-0.1.0.tar.gz"
expect_exit "vaisdb archived self-test" 42 "$vdb_extract/vaisdb-0.1.0/bin/vaisdb"
write_file_ingest_inputs "$tmp"
expect_pair_args \
    "file ingest argv workflow" \
    "$ROOT/examples/e297_vaisdb_file_ingest_workflow.vais" \
    "$tmp/doc-a.txt" \
    "$tmp/doc-b.txt" \
    "$tmp/query.txt"
expect_pair_args \
    "file ingest Result argv workflow" \
    "$ROOT/examples/e298_vaisdb_file_ingest_result_flow.vais" \
    "$tmp/doc-a.txt" \
    "$tmp/doc-b.txt" \
    "$tmp/query.txt"
expect_exit \
    "file ingest Result missing doc direct" \
    10 \
    "$ROOT/scripts/vaisc" run "$ROOT/examples/e298_vaisdb_file_ingest_result_flow.vais" --engine direct -- \
    "$tmp/missing-doc.txt" \
    "$tmp/doc-b.txt" \
    "$tmp/query.txt"
expect_exit \
    "file ingest Result missing doc default" \
    10 \
    "$ROOT/scripts/vaisc" run "$ROOT/examples/e298_vaisdb_file_ingest_result_flow.vais" -- \
    "$tmp/missing-doc.txt" \
    "$tmp/doc-b.txt" \
    "$tmp/query.txt"

echo
echo "RESULT: VaisDB workflow gate OK"
