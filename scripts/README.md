# scripts/

Helper scripts for Vais compiler development and CI.

## Integrity Gate

### `cargo integrity` — fast test runner

```bash
cargo integrity
```

Runs the full integrity test matrix (compiler_syntax, compiler_stages,
ecosystem_health) against the built compiler. This is a pure `cargo test`
invocation: it exits 0 if all tests pass, 1 if any test fails. No
regression comparison is performed.

Use this during development to quickly verify the compiler produces the
expected output categories.

### `./scripts/check-integrity.sh` — full CI gate

```bash
./scripts/check-integrity.sh
```

The canonical CI entry point. It:

1. Ensures `/tmp/vais-lib/std` symlink exists (idempotent).
2. Runs Core v0 certification and logs output to `/tmp/core-certify.log`.
3. Runs MIR strict-lowering, Core strict fixture, interpreter, and structural
   validation tests and logs output to `/tmp/mir-validation.log`.
4. Runs codegen invariant tests and logs output to `/tmp/codegen-invariants.log`.
5. Runs `cargo integrity` and logs output to `/tmp/integrity.log`.
6. Runs `cargo test -p vaisc --test e2e --release phase158` and logs
   output to `/tmp/phase158.log`.
7. Runs VaisDB runtime smoke tests and logs output to
   `/tmp/vaisdb-runtime-smoke.log`.
8. Runs vais-server runtime smoke tests and logs output to
   `/tmp/vais-server-runtime-smoke.log`.
9. Parses the `INTEGRITY std_files pass=N` and
   `INTEGRITY vaisdb_files pass=N` lines from the log.
10. Compares those pass counts against baseline thresholds. If either
   is below the threshold, exits 1 with a `REGRESSION:` message.

The final output is split by gate: `CORE`, `MIR`, `CODEGEN`, `UNSAFE AUDIT`,
`ECOSYSTEM`, `BACKEND`, std runtime smokes, `VAISDB RUNTIME`,
`SERVER RUNTIME`, vais-web runtime/unit/package/build gates,
cross-package/product gates, and package full-build smoke.
Core/MIR/codegen failures must be fixed before using downstream package counts
as a progress signal.

## W1-E Regression Script Classification

T-532 classifies the regression scripts for W1 closure:

| Script | W1-E classification |
|---|---|
| `check-integrity.sh` | Canonical W1 aggregate regression gate. |
| `core-certify.sh` | Core certification gate used by W1-E. |
| `check-empirical.sh` | W1 language-surface evidence gate for A1/A2/A4 and rejected/nonclaim fixtures. |
| `check-ai-docs-sync.mjs`, `check-ai-reference-app.sh`, `check-public-claims.mjs` | W1 AI/public docs evidence gates. |
| `vaisdb-regression.sh` | Historical same-class diagnostic with known clang-error baselines; not a W1 closure gate. Current W1 package/codegen evidence is `test_vaisdb_files_codegen_ok` plus runtime/package smokes. Broader DB regression ownership belongs to W2. |
| `vais-server-regression.sh` | Historical same-class diagnostic with known baselines; not a W1 closure gate. Production server regression ownership belongs to W3. |
| `vais-web-regression.sh` | Historical vais-web workspace diagnostic; not a W1 closure gate. Current W1 evidence comes through `check-integrity.sh`; production web/admin regression ownership belongs to W4. |

For T-532, these scripts were syntax-checked with `bash -n`. The historical
regression scripts were not promoted into the W1 closure bundle because they
encode old known-failure baselines and product-domain follow-up work rather
than current language/compiler certification.

### Exit codes

| Code | Meaning |
|------|---------|
| 0    | All gates pass — integrity OK |
| 1    | Test suite failure, regression detected, phase158 failure, or runtime smoke failure |

### Baseline threshold overrides

The thresholds can be overridden via environment variables:

```bash
INTEGRITY_STD_MIN=82 INTEGRITY_VAISDB_MIN=261 ./scripts/check-integrity.sh
```

| Variable | Default | Meaning |
|----------|---------|---------|
| `INTEGRITY_STD_MIN` | `82` | Minimum std_files pass count |
| `INTEGRITY_VAISDB_MIN` | `261` | Minimum vaisdb_files pass count |
| `INTEGRITY_HTTP_CLIENT_RUNTIME_MIN` | `15` | Minimum std/http_client runtime smoke pass count |
| `INTEGRITY_TLS_RUNTIME_MIN` | `2` | Minimum std/tls runtime smoke pass count |
| `INTEGRITY_SQLITE_RUNTIME_MIN` | `3` | Minimum std/sqlite runtime smoke pass count |
| `INTEGRITY_POSTGRES_RUNTIME_MIN` | `1` | Minimum std/postgres runtime smoke pass count |
| `INTEGRITY_VAISDB_RUNTIME_MIN` | `37` | Minimum VaisDB runtime smoke pass count |
| `INTEGRITY_SERVER_RUNTIME_MIN` | `25` | Minimum vais-server runtime smoke pass count |
| `INTEGRITY_WEB_RUNTIME_MIN` | `61` | Minimum vais-web runtime smoke pass count in skip-mode |
| `INTEGRITY_WEB_UNIT_MIN` | `390` | Minimum vais-web unit test pass count |
| `INTEGRITY_WEB_PACKAGES_MIN` | `3296` | Minimum vais-web non-kit package test pass count |
| `INTEGRITY_WEB_FULL_BUILD_MIN` | `24` | Minimum vais-web full-build package count |
| `INTEGRITY_BACKEND_PHASE158_MIN` | `18` | Minimum backend phase158 smoke pass count |
| `INTEGRITY_CROSS_PACKAGE_SCHEMA_MIN` | `15` | Minimum cross-package schema gate count |
| `INTEGRITY_MULTI_DOMAIN_PRODUCT_MIN` | `9` | Minimum multi-domain product gate count |
| `INTEGRITY_PKG_FULL_BUILD_MIN` | `2` | Minimum package full-build smoke count |

To deliberately trigger a regression failure (for testing the gate itself):

```bash
INTEGRITY_STD_MIN=999 ./scripts/check-integrity.sh
# Exits 1 with: REGRESSION: std_files baseline=999 current=82/82
```
