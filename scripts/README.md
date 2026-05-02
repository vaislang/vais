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

The final output is split by gate: `CORE`, `MIR`, `CODEGEN`,
`ECOSYSTEM`, `BACKEND`, `VAISDB RUNTIME`, and `SERVER RUNTIME`.
Core/MIR/codegen failures must be fixed before using downstream package counts
as a progress signal.

### Exit codes

| Code | Meaning |
|------|---------|
| 0    | All gates pass — integrity OK |
| 1    | Test suite failure, regression detected, phase158 failure, or runtime smoke failure |

### Baseline threshold overrides

The thresholds can be overridden via environment variables:

```bash
INTEGRITY_STD_MIN=82 INTEGRITY_VAISDB_MIN=219 ./scripts/check-integrity.sh
```

| Variable | Default | Meaning |
|----------|---------|---------|
| `INTEGRITY_STD_MIN` | `82` | Minimum std_files pass count |
| `INTEGRITY_VAISDB_MIN` | `219` | Minimum vaisdb_files pass count |

To deliberately trigger a regression failure (for testing the gate itself):

```bash
INTEGRITY_STD_MIN=999 ./scripts/check-integrity.sh
# Exits 1 with: REGRESSION: std_files baseline=999 current=82/82
```
