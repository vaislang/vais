#!/usr/bin/env bash
# Thin bootstrap wrapper for the Vais-authored checker contract gate.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

cd "$HERE" || exit 1

"$HERE/scripts/vaisc" run "$HERE/tools/vais_check_contract_check.vais" -- "$HERE" "$tmp/work"
