#!/usr/bin/env bash
# Rejected-01 — Box raw generic (no type parameter).
# Site: unification.rs:114 (vais-types/src/inference/unification.rs)
# Classification: REJECTED at type-check (NOT A4, NOT Controlled)
#
# Probe: declare a parameter of type `Box` (no <T>), then access a
#        field on it. The type checker rejects the field access with
#        E030 "no field 'value' on type 'Box'" before codegen runs.
#
# assertion_kind = "check_fails" — `vaisc check` itself must exit
# non-zero with the documented diagnostic. The type checker is the
# stable defense; users get a clear error before any silent path
# can be reached.

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

# vaisc check MUST exit non-zero.
CHECK_OUTPUT="$( "$VAISC" check "$WORK/probe.vais" 2>&1 || true )"
CHECK_EXIT=0
"$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1 && CHECK_EXIT=0 || CHECK_EXIT=$?

if [[ "$CHECK_EXIT" == "0" ]]; then
  echo "DRIFT: Rejected-01 vaisc check now succeeds — surface may be" >&2
  echo "  Controlled or simply not firing.  Investigate before assuming" >&2
  echo "  this is a fix." >&2
  exit 1
fi

# Required patterns (specific to E030 field-access rejection).
REQUIRED=(
  "E030"
  "no field 'value'"
  "type 'Box'"
)

for pat in "${REQUIRED[@]}"; do
  if ! grep -qF "$pat" <<< "$CHECK_OUTPUT"; then
    echo "DRIFT: Rejected-01 vaisc check failed but stderr did not contain '$pat':" >&2
    echo "$CHECK_OUTPUT" >&2
    exit 1
  fi
done

echo "Rejected-01 OK: vaisc check exits ${CHECK_EXIT} with E030 'no field value on type Box' — type-check rejects raw Box field access as documented."
