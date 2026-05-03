#!/usr/bin/env bash
# A4-11 — `?` operator silently accepted in non-Result function.
# Discovered during Step 9 A2-01 promotion (A2-NEG-DRIFT).
# NOT yet in master-plan.toml [[phase.A4.runtime_silent]].
# Site: TBD — likely vais-types/src/checker_expr/ try-operator handler.
#
# Surface: when a function with return type `i64` (or anything that is
# NOT Result<_,_> / Option<_>) contains `expr?` where expr is Result-
# typed, the type checker accepts. The codegen then emits
#   ret { i8, i64 } %call
# in a function whose signature declares it returns plain i64. clang
# rejects with "value doesn't match function result type 'i64'".
#
# Per L-002 north star, this should be caught at type-check with a
# stable diagnostic (proposal: E_TRY_REQUIRES_MATCHING_RETURN), not
# silently passed to codegen.
#
# assertion_kind = "build_fails"

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

# Phase 1: vaisc check must currently PASS (this IS the surface).
if ! "$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1; then
  echo "DRIFT: A4-11 probe now fails type-check." >&2
  echo "  This is GOOD news — the type checker caught the predicate" >&2
  echo "  violation. Migrate fixture to a check_fails form." >&2
  exit 1
fi

# Phase 2: full build must FAIL with clang IR mismatch.
BUILD_OUT="$( cd "$WORK" && "$VAISC" probe.vais 2>&1 || true )"
BUILD_EXIT=0
( cd "$WORK" && "$VAISC" probe.vais >/dev/null 2>&1 ) || BUILD_EXIT=$?

if [[ "$BUILD_EXIT" == "0" ]]; then
  echo "DRIFT: A4-11 build now succeeds — the IR mismatch must have" >&2
  echo "  been worked around in codegen. Investigate before assuming" >&2
  echo "  this is a fix; the surface might still be silent at runtime." >&2
  exit 1
fi

# Confirm the failure is the documented IR-mismatch, not something else.
REQUIRED=("clang compilation failed" "doesn't match function result type")
for pat in "${REQUIRED[@]}"; do
  if ! grep -qF "$pat" <<< "$BUILD_OUT"; then
    echo "DRIFT: A4-11 build failed but stderr lacks '$pat':" >&2
    echo "$BUILD_OUT" >&2
    exit 1
  fi
done

echo "A4-11 OK: probe type-checks (silent surface), build fails at clang with IR mismatch — confirms late-codegen-silent classification."
