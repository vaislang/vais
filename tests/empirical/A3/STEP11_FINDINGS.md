# Step 11 (A3 quarantine) findings — first iteration (2026-05-03)

This file records empirical findings from the first iteration of
Order Step 11 (A3 quarantine). Mirrors `compiler/tests/empirical/A4/
STEP7_FINDINGS.md`.

## Context

Master Plan v16 declares A3 (Quarantine intended-but-unfinished public
APIs) with the following candidates:

- lang/packages/vais-server/src/api/grpc.vais
- lang/packages/vais-server/src/api/graphql.vais
- lang/packages/vais-server/src/ws/
- vais-server HTTPS path
- lang/packages/vaisdb/src/security/ (TLS/encryption/RBAC, ~1.8K LOC)
- vaisdb graph traversal executor / fulltext executor / advanced SQL /
  WAL bulk graph

Step 11 deliverable: "cross-package import returns stable error[CODE]
or Err(UNCERTIFIED_FEATURE) (per surface)".

## Findings

### F-A3-01 — vaisc import resolution falls back SILENTLY on missing modules

Probe:
```vais
U server/api/grpc

F main() -> i64 { R 0 }
```

Run: `vaisc check probe.vais`.

Observed (macOS arm64, vaisc release, 2026-05-03):
```
warning: import resolution failed, falling back to single-file parse:
  Cannot find module 'grpc': tried '/tmp/server/api/grpc.vais' and
  '/tmp/server/api/grpc/mod.vais'
  Hint: run from the compiler directory or set VAIS_STD_PATH to the stdlib.
OK No errors found
```

`vaisc check` exits **0** despite the unresolved import. The user's
program is silently treated as `No errors found` even though it
referenced an external module that does not exist. This is a
silent-failure pattern (north star L-002 violation).

**Impact on Step 11 deliverable**: the documented quarantine path
("import returns stable error[CODE]") cannot be implemented as a
*per-surface* gate while vaisc's general import-resolution path
silently downgrades to a single-file parse. Even if A3 surfaces (grpc,
graphql, etc.) attached a stable rejection, `U server/api/<surface>`
would never reach that rejection — the resolver would warn and fall
back to single-file parse before any A3-specific check fires.

**Recommendation**: Step 11 stage 0 first lands a compiler-side fix
that turns the warning into a stable error[E_IMPORT_NOT_FOUND]. After
that lands, A3 per-surface fixtures can demonstrate the documented
quarantine behavior.

This finding is logged here because the A3 fixture work cannot
empirically demonstrate "stable error[CODE]" without that compiler
change — running the obvious probe today silently passes, which is
the opposite of what the deliverable promises.

### F-A3-02 — `lang/packages/<name>` cannot be imported via the toml-declared path

Probe attempted: `U vais-server/src/api/grpc`. Lexer rejects the
hyphen in `vais-server` (parsed as `vais` minus `server`), so the
toml-declared paths (which contain hyphens) are not even valid
import strings.

Even with the simpler `server/api/grpc` form, F-A3-01 applies — the
resolver does not find the module and silently falls back.

**Recommendation**: master-plan A3 candidate paths should be expressed
in import-syntax form, not filesystem-path form, in master-plan.toml's
A3 inventory (when it gains structured entries). Document the import
form alongside the filesystem path so the test harness knows what
string to feed to `U`.

## Next iterations

- Compiler fix: `import resolution failed → ERROR not WARNING`. Lands
  a new error code (E_IMPORT_NOT_FOUND or E_UNCERTIFIED_FEATURE).
- Per-surface A3 fixtures using assertion_kind = "check_fails" (the
  protocol form added during Step 7 fourth iteration for Rejected
  surfaces).
- master-plan.toml structured A3 entries with both filesystem path
  and import-syntax form.

Until the compiler change lands, Step 11 is BLOCKED on the silent-
fallback issue, not on per-surface fixture authoring.

---

## Update — Step 11 UNBLOCKED (2026-05-03 second iteration)

### F-A3-03 — VAIS_STRICT_IMPORTS=1 introduced

The recommended compiler-side fix landed in
`compiler/crates/vaisc/src/commands/simple.rs`. New behaviour:

- Default (env unset or != "1"): unchanged — emit `warning:` and fall
  back to single-file parse. Preserves regression-locked baseline.
- `VAIS_STRICT_IMPORTS=1`: hard fail with
  `error[E_IMPORT_NOT_FOUND]: import resolution failed in strict mode`
  followed by the original resolver message. Exit code 1.

The hint emitted under default mode now also tells the user the env
variable name, so users see the strict-mode opt-in alongside the
warning. The new hint reads:

  Hint: set VAIS_STRICT_IMPORTS=1 to make this fatal instead of a warning.

INTEGRITY OK preserved (no baseline regression: strict mode is opt-in
and check-integrity.sh does not set the env variable).

### F-A3-04 — A3 fixtures use `required_env` in meta.toml

A new convention in `meta.toml [assertion_kind]` for fixtures whose
documented behaviour requires a specific environment:

```toml
[assertion_kind]
kind = "check_fails"
required_env = { VAIS_STRICT_IMPORTS = "1" }
required_stderr_patterns = ["E_IMPORT_NOT_FOUND", "grpc"]
```

The runner exports the listed env vars before invoking `vaisc check`.
This form is OPTIONAL — fixtures whose probes work in default
environment do not declare `required_env`.

### A3-01 fixture LANDED

`compiler/tests/empirical/A3/A3-01_grpc_uncertified/` covers the
master-plan v16 A3 candidate `lang/packages/vais-server/src/api/
grpc.vais` (import form `server/api/grpc`). Runner asserts strict-
mode `vaisc check` rejects with E_IMPORT_NOT_FOUND + 'grpc' in stderr.

### Next iterations

- A3-02 graphql, A3-03 ws, A3-04 vaisdb security, etc. — same fixture
  shape, swap surface name.
- master-plan.toml structured A3 entries (currently just a narrative
  list). Each entry carries import_form + filesystem_path so future
  fixtures can be generated.
- Consider extending `compiler/scripts/check-integrity.sh` to opt
  into VAIS_STRICT_IMPORTS=1 once all stdlib + selfhost paths
  resolve cleanly under strict mode. Today this would regress the
  baseline so it's deferred.
