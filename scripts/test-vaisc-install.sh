#!/usr/bin/env bash
# Standalone install/package gate for the native Vais compiler.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

exec "$HERE/scripts/vaisc" run "$HERE/tools/vaisc_install_check.vais" -- "$HERE" "$tmp"
