#!/usr/bin/env bash
# Full pre-tag release gate for the Vais mainline.
#
# This script is intentionally stricter than the smallest local change gate. Run
# it from a clean checkout before cutting a public source tag.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

run() {
    printf '\n==> %s\n' "$*"
    "$@"
}

for script in "$ROOT"/scripts/*.sh; do
    run bash -n "$script"
done

run bash "$ROOT/scripts/test-vais-check-vais.sh"
run bash "$ROOT/scripts/test-vais-manifest-check-vais.sh"
run bash "$ROOT/scripts/test-vais-import-graph-check-vais.sh"
run bash "$ROOT/scripts/test-vaisc-native.sh"
run bash "$ROOT/scripts/test-vaisc-install.sh"
run bash "$ROOT/scripts/test-vaisc.sh"
run bash "$ROOT/scripts/test-vaisc-front.sh"
run bash "$ROOT/scripts/test-vaisc-direct.sh"
run bash "$ROOT/scripts/test-vaisc-errors.sh"
run bash "$ROOT/scripts/test-vaisc-parity.sh"
run bash "$ROOT/scripts/test-vaisc-host.sh"
run bash "$ROOT/scripts/test-embed-self-source-vais.sh"
run bash "$ROOT/scripts/test-normalize-stage-ir-vais.sh"
run bash "$ROOT/scripts/test-compiler.sh"
run bash "$ROOT/scripts/test-fixpoint.sh"
run bash "$ROOT/scripts/test-fixpoint2.sh"
run bash "$ROOT/scripts/test.sh"
run bash "$ROOT/scripts/test-fixpoint-full-self.sh"
run bash "$ROOT/scripts/test-fixpoint-full.sh"
run bash "$ROOT/scripts/package-vaisc-release.sh"

if [ -d "$ROOT/website" ]; then
    (cd "$ROOT/website" && run npm run build)
fi

run git -C "$ROOT" diff --check

echo
echo "RESULT: Vais release gates OK"
