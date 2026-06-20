#!/usr/bin/env bash
# NV-C2 direct-emitter gate for the Vais `vaisc` command.
#
# This script is a thin bootstrap wrapper. The check orchestration and the
# direct smoke, reject/trap, feature, and no-Python environment checks are
# implemented as Vais-authored harnesses.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$HERE/scripts/vaisc" run "$HERE/tools/vaisc_direct_gate.vais" -- "$HERE" "$tmp"
