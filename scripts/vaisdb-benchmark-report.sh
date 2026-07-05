#!/usr/bin/env bash
# Run the reusable Vais-authored VaisDB benchmark report tool.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

exec "$ROOT/scripts/vaisc" run "$ROOT/tools/vaisdb_benchmark_report.vais" -- "$ROOT" "$@"
