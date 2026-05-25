#!/usr/bin/env bash
# Rejected-04 — W1-B automatic Drop nonclaims.
#
# This fixture is a source-audit guard, not a behavior promotion. It proves
# that Token, parser AST Statement, and EmbeddedResult still rely on explicit
# cleanup APIs and do not silently gain automatic Drop behavior.

set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
COMPILER_ROOT="$(cd "$DIR/../../../.." && pwd)"
REPO_ROOT="$(cd "$COMPILER_ROOT/.." && pwd)"

TOKEN="$REPO_ROOT/lang/packages/vaisdb/src/sql/parser/token.vais"
AST="$REPO_ROOT/lang/packages/vaisdb/src/sql/parser/ast.vais"
EMBEDDED="$REPO_ROOT/lang/packages/vaisdb/src/server/embedded.vais"
EXCLUDED="$COMPILER_ROOT/docs/certification/EXCLUDED_FEATURES.md"

for f in "$TOKEN" "$AST" "$EMBEDDED" "$EXCLUDED"; do
  if [[ ! -f "$f" ]]; then
    echo "FIXTURE_BROKEN: expected file missing: $f" >&2
    exit 2
  fi
done

require_pattern() {
  local file="$1"
  local pattern="$2"
  local label="$3"
  if ! grep -qF "$pattern" "$file"; then
    echo "DRIFT: missing $label in $file" >&2
    exit 1
  fi
}

require_pattern "$TOKEN" "fn free_owned_text(&mut self) -> ()" "manual Token cleanup"
require_pattern "$TOKEN" "fn free_token_vec_owned_text(tokens: &mut Vec<Token>) -> ()" "manual Token vector cleanup"
require_pattern "$AST" "fn free_parser_owned_text(self) -> Statement" "manual parser AST cleanup"
require_pattern "$EMBEDDED" "fn free_owned_text(&mut self) -> ()" "manual EmbeddedResult cleanup"
require_pattern "$EXCLUDED" "Token/AST/EmbeddedResult automatic Drop" "W1-B Drop nonclaim text"
require_pattern "$EXCLUDED" "free_token_vec_owned_text()" "manual Token cleanup nonclaim"
require_pattern "$EXCLUDED" "Statement.free_parser_owned_text()" "manual AST cleanup nonclaim"
require_pattern "$EXCLUDED" "EmbeddedResult.free_owned_text()" "manual result cleanup nonclaim"

if grep -R -nE 'impl[[:space:]]+(Token|Statement|EmbeddedResult)[[:space:]]*:[[:space:]]*Drop|impl[[:space:]]+Drop[[:space:]]+for[[:space:]]+(Token|Statement|EmbeddedResult)' "$TOKEN" "$AST" "$EMBEDDED" >/tmp/vais_r04_drop_matches.txt; then
  echo "DRIFT: unpromoted automatic Drop implementation appeared:" >&2
  cat /tmp/vais_r04_drop_matches.txt >&2
  exit 1
fi

echo "Rejected-04 OK: Token/AST/EmbeddedResult automatic Drop remains unpromoted; manual cleanup APIs and EXCLUDED_FEATURES nonclaims are present."
