#!/usr/bin/env bash
# NV-C4 parity gate for the Vais `vaisc` native path.
#
# The manifest records the release subset. Entries marked native-supported must
# match their `# expect:` value through scripts/vaisc. The parity logic itself is
# implemented in tools/vais_parity_check.vais; this shell file is only the
# temp-dir bootstrap wrapper around that Vais-authored gate.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="${1:-$HERE/tools/vaisc-parity.tsv}"
SOURCE_ROOT="${VAISC_PARITY_ROOT:-$HERE}"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

"$HERE/scripts/vaisc" run "$HERE/tools/vais_parity_check.vais" -- "$HERE" "$MANIFEST" "$SOURCE_ROOT" "$tmp/work"
