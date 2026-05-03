#!/usr/bin/env bash
# A4-09 Lifetime ref erasure — empirical fixture runner.
#
# Surface (per master-plan.toml [[order.A4.late_codegen_silent]] entry id="A4-09"):
#   site:    unification.rs:450 (vais-types/src/inference/unification.rs)
#   probe:   F take_lifetime_ref<'a>(r: &'a i64) -> i64 { ... }
#            called with a plain &i64 argument
#   expected (per design): vaisc check rejects with stable error code
#                          (lifetime parameter not properly tracked through
#                          monomorphization → emitted symbol mangling
#                          differs from call site's expectation).
#   actual   (current behavior):
#            type-checks, codegen produces LLVM IR that references
#            `_take_lifetime_ref`, but the function definition is emitted
#            under a different mangled name (or not at all) — clang linker
#            fails with "Undefined symbols: _take_lifetime_ref".
#
# This is a LATE-CODEGEN-SILENT failure: the type checker accepts the
# program; the failure surfaces at link time, after most of the build
# pipeline has run.  Worse than runtime corruption because the bad output
# was almost shipped.
#
# Exit codes from this runner:
#   0 — surface still has the v1-documented behavior (linker fails on probe).
#   1 — runtime exit code does not match expected pattern — surface DRIFTED.
#       Either: probe now type-check-fails (good news, the type checker
#       caught it), or the linker now succeeds (very strange — investigate).
#   2 — fixture itself broken.

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

# Phase 1: type-check must currently PASS (this is what A4-09 documents —
# the type checker fails to reject).
if ! "$VAISC" check "$WORK/probe.vais" >/dev/null 2>&1; then
  echo "DRIFT: probe now fails type-check." >&2
  echo "  This is GOOD news — the type checker caught the lifetime erasure." >&2
  echo "  Migrate this fixture to a negative form per" >&2
  echo "  compiler/tests/empirical/A4/README.md guidance." >&2
  exit 1
fi

# Phase 2: full build must FAIL with linker error referencing
# _take_lifetime_ref.
BUILD_OUTPUT=""
BUILD_EXIT=0
BUILD_OUTPUT="$( cd "$WORK" && "$VAISC" probe.vais 2>&1 || true )"
BUILD_EXIT=$( cd "$WORK" && "$VAISC" probe.vais >/dev/null 2>&1 ; echo $? )

if [[ "$BUILD_EXIT" == "0" ]]; then
  echo "DRIFT: build succeeded — linker did not fail as expected." >&2
  echo "  Either codegen now emits the correct symbol, or runtime is now" >&2
  echo "  silently wrong.  Investigate before assuming this is a fix." >&2
  exit 1
fi

# Verify the linker error mentions the symbol (loose match).
if ! grep -qF '_take_lifetime_ref' <<< "$BUILD_OUTPUT"; then
  echo "DRIFT: build failed but error did not reference _take_lifetime_ref:" >&2
  echo "$BUILD_OUTPUT" >&2
  exit 1
fi

if ! grep -qiE 'undefined|symbol|linker|ld:' <<< "$BUILD_OUTPUT"; then
  echo "DRIFT: build failed but error doesn't look like a linker error:" >&2
  echo "$BUILD_OUTPUT" >&2
  exit 1
fi

echo "A4-09 OK: probe type-checks, codegen emits, linker fails on _take_lifetime_ref (matches expected late-codegen-silent failure)."
