#!/usr/bin/env bash
# A4-10 Struct partial-init silent acceptance (post-removal: check_fails).
# Step 13 stage 1 LANDED: silent surface removed (default-mode strict
# since 2026-05-04). Empirical baseline footprint = 0 std + 0 vaisdb
# after 3 vaisdb fixes:
#   lang/packages/vaisdb/src/sql/executor/scan.vais (1 BTree literal)
#   lang/packages/vaisdb/src/sql/executor/dml.vais (3 BTree literals)
#   lang/packages/vaisdb/src/storage/btree/wal_integration.vais (1 PageAllocPayload)
# Each was relying on silent zero-init for entry_count / latch_table / page_type.

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
  echo "DRIFT: A4-10 check now succeeds — strict default may have regressed." >&2
  exit 1
fi

if ! grep -qE "missing fields?|expected all required fields" <<< "$OUT"; then
  echo "DRIFT: A4-10 check failed but stderr lacks 'missing fields' pattern:" >&2
  echo "$OUT" >&2
  exit 1
fi

if ! grep -qF "age" <<< "$OUT"; then
  echo "DRIFT: A4-10 check failed but stderr does not mention the missing field 'age':" >&2
  echo "$OUT" >&2
  exit 1
fi

echo "A4-10 OK: vaisc check rejects probe with 'missing fields: age' (silent surface removed; default-mode strict)."
