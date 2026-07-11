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
printf 'not ingested\n' > "$vdb_docs/skip.md"
vdb_dir_index="$tmp/vaisdb-pkg-dir-index.txt"
expect_exit "vaisdb package ingest-dir" 0 "$vdb_dist/bin/vaisdb" ingest-dir "$vdb_dir_index" "$vdb_docs"
expect_exit "vaisdb package rank" 4 "$vdb_dist/bin/vaisdb" rank "$vdb_dir_index" "ai cache" 2
expect_exit "vaisdb package ingest-dir missing" 3 "$vdb_dist/bin/vaisdb" ingest-dir "$vdb_dir_index" "$tmp/vaisdb-no-such-docs"
expect_exit "vaisdb package rank bad k" 1 "$vdb_dist/bin/vaisdb" rank "$vdb_dir_index" "ai cache" 0
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
