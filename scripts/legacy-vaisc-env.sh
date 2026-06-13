#!/usr/bin/env bash
# Resolve the Legacy Vais compiler used by bootstrap/oracle scripts.
#
# Source this after HERE is set to the New Vais repo root. It intentionally
# prefers the Legacy repo build artifacts over PATH so tests do not accidentally
# pick up an older installed `vaisc`.

VAIS_COMPILER_ROOT="${VAIS_COMPILER_ROOT:-/Users/sswoo/study/projects/vais/compiler}"
VAIS_ROOT="$VAIS_COMPILER_ROOT"

_legacy_candidate="${LEGACY_VAISC:-}"
if [ -z "$_legacy_candidate" ]; then
    for _candidate in \
        "$VAIS_COMPILER_ROOT/target/debug/vaisc" \
        "$VAIS_COMPILER_ROOT/target/release/vaisc"
    do
        if [ -x "$_candidate" ]; then
            _legacy_candidate="$_candidate"
            break
        fi
    done
fi
if [ -z "$_legacy_candidate" ]; then
    _legacy_candidate="$(command -v vaisc || true)"
fi
if [ -z "$_legacy_candidate" ]; then
    echo "error: Legacy vaisc not found. Set LEGACY_VAISC=/path/to/legacy/vaisc" >&2
    return 1 2>/dev/null || exit 1
fi

LEGACY_VAISC_RESOLVED="$(cd "$(dirname "$_legacy_candidate")" && pwd)/$(basename "$_legacy_candidate")"
_repo_vaisc="$(cd "${HERE:-$(pwd)}/scripts" 2>/dev/null && pwd)/vaisc"
if [ "$LEGACY_VAISC_RESOLVED" = "$_repo_vaisc" ]; then
    echo "error: Legacy bootstrap needs Legacy vaisc, but PATH resolved repo-local scripts/vaisc" >&2
    echo "hint: set LEGACY_VAISC=/path/to/legacy/vaisc" >&2
    return 1 2>/dev/null || exit 1
fi

legacy_vaisc_build() {
    ( cd "$VAIS_COMPILER_ROOT" && rm -rf /tmp/.vais-cache && "$LEGACY_VAISC_RESOLVED" build "$@" )
}
