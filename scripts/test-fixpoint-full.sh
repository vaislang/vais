#!/usr/bin/env bash
# Long full-codegen regression gate. The fixture orchestration lives in Vais so
# the remaining shell boundary is only process setup and temp-dir cleanup.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$HERE/scripts/vaisc" run "$HERE/tools/fixpoint_full_codegen_check.vais" -- "$HERE" "$tmp"
