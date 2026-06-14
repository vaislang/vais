#!/usr/bin/env bash
# Build a standalone Vais compiler release archive.
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="${OUT_DIR:-$HERE/dist}"
VERSION="${VAIS_VERSION:-}"

usage() {
    cat <<'USAGE'
usage: scripts/package-vaisc-release.sh [--out-dir DIR] [--version VERSION]

Builds:

  dist/vais-VERSION-OS-ARCH.tar.gz

The archive contains a standalone bin/vaisc binary plus current first-read docs.
USAGE
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --out-dir)
            OUT_DIR="${2:?missing value for --out-dir}"
            shift 2
            ;;
        --out-dir=*)
            OUT_DIR="${1#--out-dir=}"
            shift
            ;;
        --version)
            VERSION="${2:?missing value for --version}"
            shift 2
            ;;
        --version=*)
            VERSION="${1#--version=}"
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

if [ -z "$VERSION" ]; then
    VERSION="$(sed -n 's/^#define VAIS_VERSION "\(.*\)"/\1/p' "$HERE/tools/vaisc_native.c" | head -1)"
fi
if [ -z "$VERSION" ]; then
    echo "error: cannot determine Vais version" >&2
    exit 1
fi

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"
case "$arch" in
    x86_64|amd64) arch="x64" ;;
    aarch64) arch="arm64" ;;
esac
name="vais-$VERSION-$os-$arch"
pkg="$OUT_DIR/$name"
archive="$OUT_DIR/$name.tar.gz"

rm -rf "$pkg" "$archive"
mkdir -p "$pkg/bin" "$pkg/docs/reference" "$pkg/compiler/self"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
"$HERE/scripts/build-vaisc-native.sh" "$tmp/vaisc" >/dev/null
install -m 0755 "$tmp/vaisc" "$pkg/bin/vaisc"
cp "$HERE/README.md" "$pkg/README.md"
cp "$HERE/CHANGELOG.md" "$pkg/CHANGELOG.md"
cp "$HERE/docs/reference/LANGUAGE.md" "$pkg/docs/reference/LANGUAGE.md"
cp "$HERE/compiler/self/SELF_HOST.md" "$pkg/compiler/self/SELF_HOST.md"

cat > "$pkg/INSTALL.md" <<EOF
# Vais $VERSION

This archive contains a standalone native Vais compiler binary.

## Install

\`\`\`bash
mkdir -p /usr/local/bin
install -m 0755 bin/vaisc /usr/local/bin/vaisc
vaisc doctor
\`\`\`

## Run

\`\`\`bash
vaisc run path/to/file.vais
vaisc emit-ir path/to/file.vais -o /tmp/file.ll
vaisc build path/to/file.vais -o /tmp/file
\`\`\`

Requirement: \`clang\` for \`doctor\`, \`build\`, and \`run\`.
EOF

tar -C "$OUT_DIR" -czf "$archive" "$name"
echo "archive: $archive"
