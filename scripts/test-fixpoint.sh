#!/usr/bin/env bash
# Thin bootstrap wrapper for the Vais-authored fixpoint tier gate.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$HERE/scripts/vaisc" run "$HERE/tools/fixpoint_tier_check.vais" -- "$HERE" "$tmp/work" fixpoint
