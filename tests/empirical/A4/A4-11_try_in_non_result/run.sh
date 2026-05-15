#!/usr/bin/env bash
# A4-11 ? operator in non-Result function (post-removal: check_fails form).
# Step 13 stage 1 LANDED: surface caught at type-check (default-mode strict
# since 2026-05-04). Empirical baseline footprint = 0 std + 0 vaisdb;
# zero migration cost. The ? operator now requires the enclosing
# function's return type to be Result<_,_> or Option<_>; mismatched
# returns produce E001 at vaisc check rather than the previous
# clang IR mismatch.

set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
COMPILER_ROOT="$(cd "$DIR/../../../.." && pwd)"
VAISC="${VAISC:-${COMPILER_ROOT}/target/release/vaisc}"
[[ -x "$VAISC" ]] || { echo "FIXTURE_BROKEN: vaisc not found" >&2; exit 2; }

WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
cp "$DIR/probe.vais" "$WORK/probe.vais"

OUT="$( "$VAISC" check "$WORK/probe.vais" 2>&1 || true )"
EXIT=0
"$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1 && EXIT=0 || EXIT=$?

if [[ "$EXIT" == "0" ]]; then
  echo "DRIFT: A4-11 check now succeeds — strict default may have regressed." >&2
  exit 1
fi

if ! grep -qE "E001|Type mismatch|expected" <<< "$OUT"; then
  echo "DRIFT: A4-11 check failed but stderr lacks expected E001 pattern:" >&2
  echo "$OUT" >&2
  exit 1
fi

if ! grep -qE "Result|Option" <<< "$OUT"; then
  echo "DRIFT: A4-11 check failed but stderr lacks Result/Option mention:" >&2
  echo "$OUT" >&2
  exit 1
fi

echo "A4-11 OK: vaisc check rejects probe with E001 'expected Result<_,_> or Option<_>' (silent surface removed; default-mode strict)."
