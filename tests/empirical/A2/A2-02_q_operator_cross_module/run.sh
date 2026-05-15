#!/usr/bin/env bash
# A2-02 — `?` operator across modules (cross-module Result/Option).
# Master Plan v17 Order Step 9 (A2 promotions).
# Predicate: see compiler/docs/certification/A2_SUBSETS.md §A2-02 (TBD).
#
# Two-probe runner (each probe is multi-file: <name>_inner.vais +
# <name>_main.vais; main imports inner via `U inner`):
#
#   probe_pos_*  — outer wraps cross-module Result `?` in a Result-typed
#                  enclosing function. Builds + runs + exits 43.
#   probe_neg_*  — outer's enclosing function returns plain i64; the
#                  cross-module `?` propagation violates the predicate.
#                  Strict A4-11 rejects at vaisc check (E001).

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

# NOTE: vaisc build requires the source path to include a directory
# component to resolve the project base. Bare `main.vais` from cwd
# fails with "Cannot resolve base directory:". We mkdir a `src/`
# subfolder and invoke as `src/main.vais`.

# ── Positive probe ────────────────────────────────────────────────────────
mkdir -p "$WORK/src"
cp "$DIR/probe_pos_inner.vais" "$WORK/src/inner.vais"
cp "$DIR/probe_pos_main.vais"  "$WORK/src/main.vais"

if ! ( cd "$WORK" && "$VAISC" check src/main.vais >/dev/null 2>&1 ); then
  echo "DRIFT: A2-02 positive probe no longer type-checks." >&2
  exit 1
fi

( cd "$WORK" && "$VAISC" build src/main.vais -o ./probe_pos >/dev/null 2>&1 )
if [[ ! -x "$WORK/probe_pos" ]]; then
  echo "FIXTURE_BROKEN: vaisc did not produce probe_pos binary" >&2
  exit 2
fi

POS_EXIT=0
"$WORK/probe_pos" || POS_EXIT=$?
if [[ "$POS_EXIT" != "43" ]]; then
  echo "DRIFT: A2-02 positive exit=${POS_EXIT}, expected 43" >&2
  exit 1
fi

# ── Negative probe ────────────────────────────────────────────────────────
rm -f "$WORK/src/inner.vais" "$WORK/src/main.vais"
cp "$DIR/probe_neg_inner.vais" "$WORK/src/inner.vais"
cp "$DIR/probe_neg_main.vais"  "$WORK/src/main.vais"

NEG_OUT="$( cd "$WORK" && "$VAISC" check src/main.vais 2>&1 || true )"
NEG_EXIT=0
( cd "$WORK" && "$VAISC" check src/main.vais >/dev/null 2>&1 ) || NEG_EXIT=$?

if [[ "$NEG_EXIT" == "0" ]]; then
  echo "DRIFT: A2-02 negative probe now type-checks — A4-11 may have regressed." >&2
  exit 1
fi

if ! grep -qE "Result|Option|expected" <<< "$NEG_OUT"; then
  echo "DRIFT: A2-02 negative check failed but stderr unfamiliar:" >&2
  echo "$NEG_OUT" >&2
  exit 1
fi

echo "A2-02 OK: positive cross-module ? exits 43; negative rejected at vaisc check (predicate enforcement via A4-11)."
