#!/usr/bin/env bash
# Controlled-01 — Str/str/String alias unification.
# Site: unification.rs:70-86 (vais-types/src/inference/unification.rs)
# Classification: Controlled (NOT A4) — runtime correct, type-checker
# allows the alias intentionally. v2 retro-validation confirms.
#
# Probe: pass a string literal "hello" to a function expecting `str`.
#        Function returns 7. Runtime exit must be 7 (the literal return).
#
# assertion_kind = "exact_exit" — Controlled surfaces have well-defined
# correct runtime behavior; the body's return value is what to assert.

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

if ! "$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1; then
  echo "DRIFT: probe no longer type-checks. Controlled coercion may have" >&2
  echo "  been removed; investigate before assuming this is correct." >&2
  exit 1
fi

( cd "$WORK" && "$VAISC" probe.vais >/dev/null 2>&1 )
if [[ ! -x "$WORK/probe" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce binary" >&2
  exit 2
fi

ACTUAL_EXIT=0
"$WORK/probe" || ACTUAL_EXIT=$?

EXPECTED="$(cat "$DIR/expected.txt" | tr -d '[:space:]')"

if [[ "$ACTUAL_EXIT" != "$EXPECTED" ]]; then
  echo "DRIFT: Controlled-01 exit changed." >&2
  echo "  expected: $EXPECTED (function returns 7)" >&2
  echo "  actual:   $ACTUAL_EXIT" >&2
  exit 1
fi

echo "Controlled-01 OK: Str/str/String alias unifies, runtime returns ${ACTUAL_EXIT} (correct — surface is Controlled, not A4)."
