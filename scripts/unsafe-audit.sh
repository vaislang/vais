#!/usr/bin/env bash
# unsafe-audit.sh — production unsafe documentation gate

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

echo "unsafe-audit: checking vais-codegen unsafe documentation..."
cargo clippy -p vais-codegen --quiet -- -A warnings -D clippy::undocumented_unsafe_blocks
echo "UNSAFE AUDIT OK: vais-codegen undocumented_unsafe_blocks=0"
