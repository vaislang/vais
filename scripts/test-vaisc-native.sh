#!/usr/bin/env bash
# Native driver smoke gate. This proves the native host can compile through
# the checked-in self-host core.
set -uo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

native_tmp="$tmp/native-tmp"
mkdir -p "$native_tmp"

native_tmp_count() {
    find "$native_tmp" -maxdepth 1 -mindepth 1 -type d -name 'vaisc-native-*' 2>/dev/null | wc -l | tr -d ' '
}

TMPDIR="$native_tmp" "$HERE/scripts/vaisc" run "$HERE/tools/vaisc_native_check.vais" -- "$HERE" "$tmp/work"
rc=$?
if [ "$rc" -ne 0 ]; then
    exit "$rc"
fi

leftover="$(native_tmp_count)"
if [ "$leftover" != "0" ]; then
    echo "error: native vaisc left $leftover temporary directory/directories in $native_tmp" >&2
    find "$native_tmp" -maxdepth 1 -mindepth 1 -type d -name 'vaisc-native-*' -print >&2
    exit 1
fi
