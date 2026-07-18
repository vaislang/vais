#!/usr/bin/env bash
# Repo whitespace hygiene gate: package vaisfmt and check every tracked .vais
# tree (trailing spaces/tabs, single trailing newline). Exits non-zero with
# the dirty-file count per tree when anything needs formatting.
set -euo pipefail
HERE="$(cd "$(dirname "$0")/.." && pwd)"
cd "$HERE"
dist="$HERE/build/vaisfmt-dist"
"$HERE/scripts/vaisc" package "$HERE/examples/e346_vaisfmt_package" -o "$dist" >/dev/null
fail=0
for tree in std examples compiler tools; do
    if ! "$dist/bin/vaisfmt" -c "$HERE/$tree"; then
        echo "FAIL vaisfmt: $tree needs formatting"
        fail=1
    fi
done
if [ "$fail" -ne 0 ]; then
    exit 1
fi
echo "RESULT: Vais source whitespace hygiene OK"
