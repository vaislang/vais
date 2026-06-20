#!/usr/bin/env bash
# NV-C1 front-contract gate for the Vais `vaisc` command.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$HERE/scripts/vaisc" run "$HERE/tools/vaisc_front_check.vais" -- "$HERE" "$tmp/work"
