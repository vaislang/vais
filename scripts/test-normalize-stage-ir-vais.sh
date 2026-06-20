#!/usr/bin/env bash
# Thin bootstrap wrapper for the Vais-authored stage IR normalizer gate.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$ROOT/scripts/vaisc" run "$ROOT/tools/normalize_stage_ir_check.vais" -- "$ROOT" "$tmp/work"
