#!/usr/bin/env bash
# Thin bootstrap wrapper for the Vais-authored self-source embedding gate.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$ROOT/scripts/vaisc" run "$ROOT/tools/embed_self_source_check.vais" -- "$ROOT" "$tmp/work"
