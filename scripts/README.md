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
2. Runs `cargo integrity` and logs output to `/tmp/integrity.log`.
3. Runs `cargo test -p vaisc --test e2e --release phase158` and logs
   output to `/tmp/phase158.log`.
4. Parses the `INTEGRITY std_files pass=N` and
   `INTEGRITY vaisdb_files pass=N` lines from the log.
5. Compares those pass counts against baseline thresholds. If either
   is below the threshold, exits 1 with a `REGRESSION:` message.

### Exit codes

| Code | Meaning |
|------|---------|
| 0    | All gates pass — integrity OK |
| 1    | Test suite failure, regression detected, or phase158 failure |

### Baseline threshold overrides

The thresholds can be overridden via environment variables:

```bash
INTEGRITY_STD_MIN=37 INTEGRITY_VAISDB_MIN=176 ./scripts/check-integrity.sh
```

| Variable | Default | Meaning |
|----------|---------|---------|
| `INTEGRITY_STD_MIN` | `37` | Minimum std_files pass count |
| `INTEGRITY_VAISDB_MIN` | `176` | Minimum vaisdb_files pass count |

To deliberately trigger a regression failure (for testing the gate itself):

```bash
INTEGRITY_STD_MIN=999 ./scripts/check-integrity.sh
# Exits 1 with: REGRESSION: std_files baseline=999 current=37/82
```
