#!/usr/bin/env bash
# Native driver smoke gate. This proves the native host can compile through
# the checked-in self-host core.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$HERE/scripts/vaisc" run "$HERE/tools/vaisc_native_check.vais" -- "$HERE" "$tmp/work"
