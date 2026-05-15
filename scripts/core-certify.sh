#!/usr/bin/env bash
# core-certify.sh — Vais Core v0 certification gate.
#
# This gate is intentionally narrower than check-integrity.sh. It validates the
# current Core contract only; ecosystem packages remain separate promotion gates.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

STD_LINK="/tmp/vais-lib/std"
STD_SRC="${REPO_ROOT}/std"

mkdir -p "/tmp/vais-lib"
if [ -L "${STD_LINK}" ]; then
    CURRENT_TARGET="$(readlink "${STD_LINK}")"
    if [ "${CURRENT_TARGET}" != "${STD_SRC}" ]; then
        rm -f "${STD_LINK}"
        ln -s "${STD_SRC}" "${STD_LINK}"
    fi
elif [ -e "${STD_LINK}" ]; then
    rm -rf "${STD_LINK}"
    ln -s "${STD_SRC}" "${STD_LINK}"
else
    ln -s "${STD_SRC}" "${STD_LINK}"
fi

echo "core-certify: std symlink ${STD_LINK} -> ${STD_SRC}"
echo "core-certify: running Core v0 manifest..."

cargo test -p vaisc --test core_certification --release -- --nocapture --test-threads=1

echo "CORE CERTIFICATION OK"
