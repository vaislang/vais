#!/usr/bin/env bash
# A4-05 Array → Pointer decay (post-overlap-removal: check_fails form).
#
# A4-05's probe takes the form `take_i64(&arr as *i64)`. It exercises
# Array→Pointer decay (the A4-05 surface) AND Pointer↔i64 (the A4-02
# surface) on the way to take_i64. Once A4-02 became strict-default
# (commit <pending>), the probe trips A4-02 first; vaisc check rejects
# at type-check before A4-05's lowering even fires.
#
# This is a happy outcome: the probe demonstrates that ANY route from
# Array to i64 is now closed at type-check. A4-05's own strict default
# is still pending (lang/packages/vaisdb/src/vector/hnsw/cow.vais needs
# migration), but A4-02 strict-default already provides safety here.
# Fixture migrates to check_fails for that reason.

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
  echo "DRIFT: A4-05 check now succeeds — A4-02 strict default may have regressed." >&2
  exit 1
fi

if ! grep -qE "E001|Type mismatch|expected" <<< "$OUT"; then
  echo "DRIFT: A4-05 check failed but stderr lacks expected E001 pattern:" >&2
  echo "$OUT" >&2
  exit 1
fi

echo "A4-05 OK: vaisc check rejects probe with E001 (Pointer→i64 caught at type-check; A4-05's own surface still opt-in pending hnsw/cow.vais migration)."
