#!/usr/bin/env bash
# A3-01 — grpc API surface (master-plan v16 A3 candidate
#   lang/packages/vais-server/src/api/grpc.vais).
# Classification: A3 (intended-but-unfinished public API).
#
# Probe: a user program imports `server/api/grpc`. Under strict-imports
# mode (VAIS_STRICT_IMPORTS=1), vaisc check rejects the unresolved
# import with E_IMPORT_NOT_FOUND, satisfying the master-plan v16
# Step 11 deliverable "cross-package import returns stable error[CODE]".
#
# Without strict mode the resolver still emits a `warning:` and falls
# back; STEP11_FINDINGS F-A3-01 documents the finding and the strict
# mode introduced to address it.
#
# assertion_kind = "check_fails" (strict-mode env required).
# Required stderr patterns: E_IMPORT_NOT_FOUND + the surface name.

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

CHECK_OUTPUT="$( VAIS_STRICT_IMPORTS=1 "$VAISC" check "$WORK/probe.vais" 2>&1 || true )"
CHECK_EXIT=0
VAIS_STRICT_IMPORTS=1 "$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1 \
  && CHECK_EXIT=0 || CHECK_EXIT=$?

if [[ "$CHECK_EXIT" == "0" ]]; then
  echo "DRIFT: A3-01 vaisc check (strict mode) now succeeds — the grpc" >&2
  echo "  surface may have been certified, or strict mode regressed." >&2
  exit 1
fi

REQUIRED=("E_IMPORT_NOT_FOUND" "grpc")
for pat in "${REQUIRED[@]}"; do
  if ! grep -qF "$pat" <<< "$CHECK_OUTPUT"; then
    echo "DRIFT: A3-01 strict-mode check failed but stderr lacks '$pat':" >&2
    echo "$CHECK_OUTPUT" >&2
    exit 1
  fi
done

echo "A3-01 OK: VAIS_STRICT_IMPORTS=1 rejects U server/api/grpc with E_IMPORT_NOT_FOUND."
