#!/usr/bin/env bash
# Run the Vais-authored VaisDB command-line tool.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

exec "$ROOT/scripts/vaisc" run "$ROOT/tools/vaisdb_cli.vais" -- "$@"
