# Mutation Testing

Vais uses [cargo-mutants](https://mutants.rs/) to verify that the test suite
catches logic changes in the core compiler crates.

## Scope

Mutations are generated for:

| Crate | Path |
|---|---|
| `vais-ast` | `crates/vais-ast/src/**` |
| `vais-types` | `crates/vais-types/src/**` |
| `vais-codegen` | `crates/vais-codegen/src/**` |
| `vaisc` | `crates/vaisc/src/**` |

Test files (`*_tests.rs`, `tests.rs`, `tests/`), fuzz targets, and non-core
crates are excluded via `mutants.toml`.

## Running Locally

### Install cargo-mutants

```bash
cargo install cargo-mutants --locked
```

### List mutants in scope (no tests run)

```bash
cargo mutants --config mutants.toml --list
```

### Run mutation tests (2 parallel jobs, deterministic order)

```bash
cargo mutants --config mutants.toml --no-shuffle -j 2
```

Results are written to `mutants.out/`.

### Run with more parallelism on a beefy machine

```bash
cargo mutants --config mutants.toml --no-shuffle -j $(nproc)
```

## CI

The `mutants.yml` workflow runs every Sunday at 00:00 UTC and on manual
dispatch. It is intentionally **not** triggered on pull requests because a
full mutation run takes several hours.

To trigger manually:

1. Go to **Actions** → **Mutation Testing**.
2. Click **Run workflow**.
3. Optionally set the number of parallel jobs (default: 2).

The `mutants-out` artifact is uploaded after each run and retained for 30 days.

## Interpreting Results

| Status | Meaning |
|---|---|
| **Caught** | A test failed when this mutation was applied — good. |
| **Missed** | No test detected the mutation — consider adding a test. |
| **Timeout** | The test suite exceeded the per-mutant timeout (120 s). |
| **Unviable** | The mutated code did not compile. |

A high **catch rate** (caught / (caught + missed)) indicates a well-tested
codebase.  Missed mutants are recorded in `mutants.out/missed.txt` and are
candidates for new test cases.

## Configuration

See `mutants.toml` at the workspace root for full configuration including
timeouts, skip patterns, and examine/exclude globs.
