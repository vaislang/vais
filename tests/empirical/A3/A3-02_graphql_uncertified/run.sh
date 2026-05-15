#!/usr/bin/env bash
# A3-02 — graphql API in vais-server
# A3 quarantine fixture; assertion_kind = check_fails under
# VAIS_STRICT_IMPORTS=1.

set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
COMPILER_ROOT="$(cd "$DIR/../../../.." && pwd)"
VAISC="${VAISC:-${COMPILER_ROOT}/target/release/vaisc}"
[[ -x "$VAISC" ]] || { echo "FIXTURE_BROKEN: vaisc not found" >&2; exit 2; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
cp "$DIR/probe.vais" "$WORK/probe.vais"

OUT="$( "$VAISC" check "$WORK/probe.vais" 2>&1 || true )"
EXIT=0
"$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1 \
  && EXIT=0 || EXIT=$?

if [[ "$EXIT" == "0" ]]; then
  echo "DRIFT: A3-02 strict-mode check now succeeds — surface may be certified" >&2
  exit 1
fi

for pat in "E_IMPORT_NOT_FOUND" "graphql"; do
  if ! grep -qF "$pat" <<< "$OUT"; then
    echo "DRIFT: A3-02 strict-mode check failed but stderr lacks '$pat':" >&2
    echo "$OUT" >&2
    exit 1
  fi
done

echo "A3-02 OK: default-strict rejects U server/api/graphql with E_IMPORT_NOT_FOUND."
