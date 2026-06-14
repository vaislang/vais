#!/usr/bin/env bash
# Remove the standalone native Vais compiler binary.
set -euo pipefail

PREFIX="${PREFIX:-/usr/local}"
DESTDIR="${DESTDIR:-}"
BINDIR=""

usage() {
    cat <<'USAGE'
usage: scripts/uninstall-vaisc.sh [--prefix DIR] [--destdir DIR] [--bindir DIR]

Removes:

  $PREFIX/bin/vaisc

Environment:
  PREFIX   install prefix, defaults to /usr/local
  DESTDIR  staging root prepended to the install path
USAGE
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --prefix)
            PREFIX="${2:?missing value for --prefix}"
            shift 2
            ;;
        --prefix=*)
            PREFIX="${1#--prefix=}"
            shift
            ;;
        --destdir)
            DESTDIR="${2:?missing value for --destdir}"
            shift 2
            ;;
        --destdir=*)
            DESTDIR="${1#--destdir=}"
            shift
            ;;
        --bindir)
            BINDIR="${2:?missing value for --bindir}"
            shift 2
            ;;
        --bindir=*)
            BINDIR="${1#--bindir=}"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "error: unexpected argument: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

if [ -z "$BINDIR" ]; then
    BINDIR="$PREFIX/bin"
fi

target="$DESTDIR$BINDIR/vaisc"
if [ -e "$target" ]; then
    rm -f "$target"
    echo "removed: $target"
else
    echo "not installed: $target"
fi
