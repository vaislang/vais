#!/usr/bin/env bash
# A1-01 — effect keyword hard-block (master-plan v22 §A1 candidate).
#
# Probe: a user program declares `effect Logger {}`. `effect` is a
# reserved lexer keyword with zero grammar production (per
# compiler/docs/language/LEXER_KEYWORDS.md and STEP10_FINDINGS.md
# F-A1-01). vaisc check must reject with P001 Unexpected token,
# satisfying master-plan v22 Step 10 deliverable
# "parser/type-check rejection + negative fixture".
#
# assertion_kind = "check_fails".
# Required stderr patterns: P001 + Effect (Logos token name).

set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
COMPILER_ROOT="$(cd "$DIR/../../../.." && pwd)"
VAISC="${VAISC:-${COMPILER_ROOT}/target/release/vaisc}"

if [[ ! -x "$VAISC" ]]; then
  echo "FIXTURE_BROKEN: vaisc not found at $VAISC" >&2
  exit 2
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cp "$DIR/probe.vais" "$WORK/probe.vais"

CHECK_OUTPUT="$( "$VAISC" check "$WORK/probe.vais" 2>&1 || true )"
CHECK_EXIT=0
"$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1 \
  && CHECK_EXIT=0 || CHECK_EXIT=$?

if [[ "$CHECK_EXIT" == "0" ]]; then
  echo "DRIFT: A1-01 vaisc check now succeeds — `effect` may have" >&2
  echo "  gained a grammar production. Re-classify A1 → A2/Certified." >&2
  exit 1
fi

REQUIRED=("P001" "Effect")
for pat in "${REQUIRED[@]}"; do
  if ! grep -qF "$pat" <<< "$CHECK_OUTPUT"; then
    echo "DRIFT: A1-01 check failed but stderr lacks '$pat':" >&2
    echo "$CHECK_OUTPUT" >&2
    exit 1
  fi
done

echo "A1-01 OK: vaisc check rejects \`effect Logger {}\` with P001."
