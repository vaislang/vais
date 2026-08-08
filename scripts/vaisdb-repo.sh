#!/usr/bin/env bash
# Repo documentation search, dogfooding the vaisdb package on this
# repository's own docs tree (docs/ recursively via `-r`).
#
#   bash scripts/vaisdb-repo.sh                 # incremental reindex
#   bash scripts/vaisdb-repo.sh search <term> [k]
#   bash scripts/vaisdb-repo.sh <any vaisdb subcommand + args>
#   (the index path is inserted as the subcommand's first argument)
#
# The index lives under build/ (ignored); every invocation reindexes
# first, so results always reflect the working tree. The top-level
# working notes (WORKLOG/ROADMAP/CHANGELOG) are deliberately not
# indexed: each one's vocabulary already exceeds the 4096 per-document
# map window, which is recorded demand for the next scale arc.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$HERE/build/vaisdb-repo-dist"
IDX="$HERE/build/repo-docs-index"

if [ ! -x "$DIST/bin/vaisdb" ] || [ "$HERE/examples/e337_vaisdb_cli_package/src/main.vais" -nt "$DIST/bin/vaisdb" ] || [ "$HERE/examples/e337_vaisdb_cli_package/src/vaisdb/index.vais" -nt "$DIST/bin/vaisdb" ]; then
    "$HERE/scripts/vaisc" package "$HERE/examples/e337_vaisdb_cli_package" -o "$DIST" >/dev/null
fi
VDB="$DIST/bin/vaisdb"

"$VDB" reindex "$IDX" "$HERE/docs" -r | sed 's/^/docs: /'

if [ "$#" -eq 0 ]; then
    "$VDB" stats "$IDX"
    exit 0
fi

set +e
"$VDB" "$1" "$IDX" "${@:2}"
rc=$?
set -e
exit "$rc"
