#!/usr/bin/env bash
# Thin bootstrap wrapper for the Vais-authored package manifest contract gate.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$ROOT/scripts/vaisc" run "$ROOT/tools/vais_manifest_contract_check.vais" -- "$ROOT" "$tmp/work"
