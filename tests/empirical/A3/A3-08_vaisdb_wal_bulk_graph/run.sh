#!/usr/bin/env bash
# A3-08 — vaisdb WAL bulk graph
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
COMPILER_ROOT="$(cd "$DIR/../../../.." && pwd)"
VAISC="${VAISC:-${COMPILER_ROOT}/target/release/vaisc}"
[[ -x "$VAISC" ]] || { echo "FIXTURE_BROKEN: vaisc not found" >&2; exit 2; }
WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
cp "$DIR/probe.vais" "$WORK/probe.vais"
OUT="$( VAIS_STRICT_IMPORTS=1 "$VAISC" check "$WORK/probe.vais" 2>&1 || true )"
EXIT=0
VAIS_STRICT_IMPORTS=1 "$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1 \
  && EXIT=0 || EXIT=$?
if [[ "$EXIT" == "0" ]]; then
  echo "DRIFT: A3-08 strict-mode check now succeeds — surface may be certified" >&2
  exit 1
fi
for pat in "E_IMPORT_NOT_FOUND" "bulk_graph"; do
  if ! grep -qF "$pat" <<< "$OUT"; then
    echo "DRIFT: A3-08 strict-mode check failed but stderr lacks '$pat':" >&2
    echo "$OUT" >&2
    exit 1
  fi
done
echo "A3-08 OK: VAIS_STRICT_IMPORTS=1 rejects U vaisdb/wal/bulk_graph with E_IMPORT_NOT_FOUND."
