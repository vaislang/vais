#!/usr/bin/env bash
# A1-04 — macro declaration hard-block (master-plan v22 §A1).
#
# Probe: a user program declares `macro foo! { () => { 0 } }`. The
# `macro` (Token::Macro) keyword is reserved with zero baseline use
# across compiler/std + lang/packages + docs/language/LIVING_SPEC.
# vaisc check must reject with P001 + the explicit `A1 hard block`
# substring.
#
# assertion_kind = "check_fails".

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
  echo "DRIFT: A1-04 vaisc check now succeeds — `macro` may have" >&2
  echo "  gained a certified surface. Re-classify A1 → A2/Certified." >&2
  exit 1
fi

REQUIRED=("P001" "Macro" "A1 hard block")
for pat in "${REQUIRED[@]}"; do
  if ! grep -qF "$pat" <<< "$CHECK_OUTPUT"; then
    echo "DRIFT: A1-04 check failed but stderr lacks '$pat':" >&2
    echo "$CHECK_OUTPUT" >&2
    exit 1
  fi
done

echo "A1-04 OK: vaisc check rejects \`macro foo! { ... }\` with P001 (A1 hard block)."
