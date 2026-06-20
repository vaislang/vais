#!/usr/bin/env bash
# Long self-host gate for compiler/self/fixpoint_full.vais.
#
# This verifies the full-source path, not just snippet-level codegen:
#   reference fixpoint_full -> generated first-generation compiler IR -> clang/run.
# It also checks that first-generation compilers can consume file-sized embedded
# sources again by retargeting their default compile("...") program to the real
# compiler/self/fixpoint*.vais sources, including fixpoint_full.vais itself.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$HERE/scripts/vaisc" run "$HERE/tools/fixpoint_full_self_check.vais" -- "$HERE" "$tmp"
