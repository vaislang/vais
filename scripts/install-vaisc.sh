#!/usr/bin/env bash
# Install the standalone native Vais compiler binary.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
PREFIX="${PREFIX:-/usr/local}"
DESTDIR="${DESTDIR:-}"
BINDIR=""

usage() {
    cat <<'USAGE'
usage: scripts/install-vaisc.sh [--prefix DIR] [--destdir DIR] [--bindir DIR]

Builds the native self-host-backed compiler and installs it as:

  $PREFIX/bin/vaisc

Environment:
  PREFIX   install prefix, defaults to /usr/local
  DESTDIR  staging root prepended to the install path
  CLANG    compiler used to build the native vaisc binary
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

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

"$HERE/scripts/build-vaisc-native.sh" "$tmp/vaisc" >/dev/null

install_dir="$DESTDIR$BINDIR"
mkdir -p "$install_dir"
install -m 0755 "$tmp/vaisc" "$install_dir/vaisc"

echo "installed: $install_dir/vaisc"
