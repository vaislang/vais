#!/usr/bin/env bash
# Host intrinsic smoke gate for the public Vais compiler command.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$HERE/scripts/vaisc" run "$HERE/tools/vais_host_check.vais" -- "$HERE" "$tmp/work"
