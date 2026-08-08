#!/usr/bin/env bash
# Whole-repo search, dogfooding the vaisdb package on this repository's
# own text corpus: docs, examples, scripts, tools, compiler, and std
# recursively, plus the top-level working notes.
#
#   bash scripts/vaisdb-repo.sh                 # incremental reindex
#   bash scripts/vaisdb-repo.sh search <term> [k]
#   bash scripts/vaisdb-repo.sh <any vaisdb subcommand + args>
#   (the index path is inserted as the subcommand's first argument)
#
# The source trees mirror into one staging corpus under build/ with
# mtimes preserved (cp -p), so a single recursive walk yields
# collision-free relative doc ids (`tools/vaisc_native`,
# `notes/WORKLOG`) and the incremental reindex still skips unchanged
# files; deleted sources vanish from the fresh mirror and the
# deletion sync removes them. build/, .git/, and website/ never enter
# the mirror.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$HERE/build/vaisdb-repo-dist"
IDX="$HERE/build/repo-docs-index"
CORPUS="$HERE/build/repo-corpus"

if [ ! -x "$DIST/bin/vaisdb" ] || [ "$HERE/examples/e337_vaisdb_cli_package/src/main.vais" -nt "$DIST/bin/vaisdb" ] || [ "$HERE/examples/e337_vaisdb_cli_package/src/vaisdb/index.vais" -nt "$DIST/bin/vaisdb" ]; then
    "$HERE/scripts/vaisc" package "$HERE/examples/e337_vaisdb_cli_package" -o "$DIST" >/dev/null
    # A rebuilt binary may carry tokenizer or format changes; drop the
    # index so the next reindex rebuilds it under the current contract.
    rm -rf "$IDX"
fi
VDB="$DIST/bin/vaisdb"

rm -rf "$CORPUS"
mkdir -p "$CORPUS/notes"
for d in docs examples scripts tools compiler std; do
    if [ -d "$HERE/$d" ]; then
        cp -Rp "$HERE/$d" "$CORPUS/$d"
    fi
done
for f in "$HERE"/*.md; do
    if [ -f "$f" ]; then
        cp -p "$f" "$CORPUS/notes/$(basename "$f")"
    fi
done

"$VDB" reindex "$IDX" "$CORPUS" -r | sed 's/^/repo: /'

if [ "$#" -eq 0 ]; then
    "$VDB" stats "$IDX"
    exit 0
fi

set +e
"$VDB" "$1" "$IDX" "${@:2}"
rc=$?
set -e
exit "$rc"
