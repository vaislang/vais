#!/usr/bin/env bash
# Gate-time regression watch: package vaisbench and time a command under a
# median budget (exit 3 when exceeded). Budgets should sit well above the
# docs/PERF-BASELINE.md figures (3x or more) so only real regressions fire.
#   scripts/vaisbench-gate.sh <budget-ms> <runs> <cmd> [args...]
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
cd "$HERE"
dist="$HERE/build/vaisbench-dist"
"$HERE/scripts/vaisc" package "$HERE/examples/e350_vaisbench_package" -o "$dist" >/dev/null
exec "$dist/bin/vaisbench" -b "$@"
