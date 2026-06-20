#!/usr/bin/env bash
# NV-C0 smoke gate for the Vais `vaisc` command contract.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$HERE/scripts/vaisc" run "$HERE/tools/vaisc_smoke_check.vais" -- "$HERE" "$tmp/work"
