#!/usr/bin/env bash
# check-ai-reference-app.sh - verify the docs-driven AI reference app.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

APP="tests/ai_reference_app/notes_service.vais"
OUT="${TMPDIR:-/tmp}/vais-ai-reference-app"
LOG="${TMPDIR:-/tmp}/vais-ai-reference-app-build.log"

if [ ! -f "${APP}" ]; then
    echo "missing AI reference app: ${APP}" >&2
    exit 1
fi

node scripts/check-ai-docs-sync.mjs

if rg -n '\b(F|S|EN|EL|M|R|T|U|P|W|X)\b' "${APP}"; then
    echo "AI reference app must use canonical public syntax, not retired aliases" >&2
    exit 1
fi

if ! rg -q 'fn ' "${APP}" || ! rg -q 'struct ' "${APP}" || ! rg -q 'match ' "${APP}"; then
    echo "AI reference app must exercise canonical fn/struct/match syntax" >&2
    exit 1
fi

if ! rg -q 'Result<' "${APP}" || ! rg -q 'Option<' "${APP}" || ! rg -q '&i64' "${APP}"; then
    echo "AI reference app must exercise Result, Option, and reference boundaries" >&2
    exit 1
fi

if [ -x target/debug/vaisc ]; then
    VAISC=(target/debug/vaisc)
else
    VAISC=(cargo run -q -p vaisc --)
fi

rm -f "${OUT}" "${LOG}"
"${VAISC[@]}" build "${APP}" -o "${OUT}" --force-rebuild >"${LOG}" 2>&1
"${OUT}"

echo "AI reference app gate passed."
