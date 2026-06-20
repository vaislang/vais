#!/usr/bin/env bash
# Value-correctness test runner (P7b: compile != correct).
#
# Usage:  scripts/test.sh          # run release-subset annotated examples
#         scripts/test.sh c4       # run one (by basename)
#
# The value-corpus logic is implemented in tools/vais_value_check.vais. This
# shell file remains only as the temp-dir bootstrap wrapper around that
# Vais-authored gate.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
SOURCE_ROOT="${VAIS_TEST_ROOT:-$HERE}"
MANIFEST="$HERE/tools/vaisc-parity.tsv"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

if [ "$#" -ge 1 ]; then
    "$HERE/scripts/vaisc" run "$HERE/tools/vais_value_check.vais" -- "$HERE" "$MANIFEST" "$SOURCE_ROOT" "$tmp/work" "$1"
else
    "$HERE/scripts/vaisc" run "$HERE/tools/vais_value_check.vais" -- "$HERE" "$MANIFEST" "$SOURCE_ROOT" "$tmp/work"
fi
