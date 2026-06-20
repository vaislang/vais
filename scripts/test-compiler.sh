#!/usr/bin/env bash
# End-to-end self-host check for the legacy compiler/self/compiler.vais smoke.
set -uo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$HERE/scripts/vaisc" run "$HERE/tools/compiler_smoke_check.vais" -- "$HERE" "$tmp/work"
