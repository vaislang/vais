#!/usr/bin/env bash
# A4-08 Vec<T> ↔ &T permissive — empirical fixture runner (runtime_crashes).
#
# Surface: Vec<T> silently unifies with &T (any reference type, not just
# the layout-compatible &[T] case) at unification.rs:384.
# Site: unification.rs:384 (`Ok(()) // Permissive: allow Vec ↔ &T`)
# assertion_kind = "runtime_crashes"
# expected_exit = 139 (SIGSEGV)
#
# History (per STEP7_FINDINGS F-06): master-plan.toml v1 sentinel
# documented "clang IR mismatch ({ptr,i64} vs ptr) — late codegen
# failure". Step 7 second pass found that the build-time symptom no
# longer reproduces (codegen has become more robust), but the surface
# itself persists at type-check, and the defect now surfaces at runtime
# as SIGSEGV when the falsely-typed &str is actually consumed (passed
# to puts() which reads it as a C string).
#
# The probe MUST consume the parameter — a probe whose body never reads
# `s` masks the defect (build succeeds, runtime returns the body
# constant). This runner uses `puts(*s)` to force a memory read.
#
# Exit codes from this runner:
#   0 — surface still has the documented silent runtime crash (exit 139).
#   1 — runtime exit ≠ 139 (DRIFT — surface fixed or probe broken).
#   2 — fixture itself broken (vaisc not found, type-check fails, etc.).

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

# Phase 1: type-check must currently PASS (this is the surface).
if ! "$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1; then
  echo "DRIFT: probe no longer type-checks (A4-08 may be removed)." >&2
  exit 1
fi

# Phase 2: build must succeed (the v1 build-time symptom is gone; that's
# part of what makes A4-08 a runtime_crashes case rather than build_fails).
if ! ( cd "$WORK" && "$VAISC" probe.vais >/dev/null 2>&1 ); then
  echo "DRIFT: build now fails — A4-08 may have reverted to v1 build_fails" >&2
  echo "  symptom (clang IR mismatch). Investigate before assuming this" >&2
  echo "  is a fix — the surface is still firing if build_fails returns." >&2
  exit 1
fi

if [[ ! -x "$WORK/probe" ]]; then
  echo "FIXTURE_BROKEN: vaisc reported success but did not produce binary" >&2
  exit 2
fi

# Phase 3: runtime must exit with SIGSEGV (139).
ACTUAL_EXIT=0
"$WORK/probe" >/dev/null 2>&1 || ACTUAL_EXIT=$?

EXPECTED=139
if [[ "$ACTUAL_EXIT" != "$EXPECTED" ]]; then
  echo "DRIFT: A4-08 runtime exit changed." >&2
  echo "  expected: $EXPECTED (SIGSEGV — Vec fat pointer misread as str)" >&2
  echo "  actual:   $ACTUAL_EXIT" >&2
  if [[ "$ACTUAL_EXIT" == "0" ]]; then
    echo "  Build + runtime succeed — surface MAY have been fixed." >&2
    echo "  Investigate: is unification.rs:384 still permissive?" >&2
  fi
  exit 1
fi

echo "A4-08 OK: probe type-checks, builds, runtime crashes with SIGSEGV (exit 139) — silent runtime acceptance confirmed."
