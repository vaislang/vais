#!/usr/bin/env bash
# NV-C3 P4 diagnostic gate for the Vais `vaisc` native path.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$HERE/scripts/vaisc" run "$HERE/tools/vaisc_errors_check.vais" -- "$HERE" "$tmp/work"
