#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

"$ROOT/scripts/vaisc" run "$ROOT/tools/vais_import_graph_contract_check.vais" -- "$ROOT" "$tmp/work"
