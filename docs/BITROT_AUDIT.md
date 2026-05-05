# Bit-rot Crate Audit

Last audited: 2026-05-05.

Master Plan v23 §Optional/non-blocking lists "Bit-rot crate decision (4 crates)"
without naming the 4. This audit identifies the 4 candidates by the operational
heuristic *(last touched ≥ 3 months ago) AND (workspace dep count ≤ 1)*.

## Audit table

| Crate | Last commit (relative) | Workspace dependents | `cargo check` |
|---|---|---|---|
| `vais-playground-server` | 3 months ago | 0 | OK |
| `vais-query` | 3 months ago | 1 (vaisc) | OK |
| `vais-supply-chain` | 3 months ago | 0 | OK |
| `vais-testgen` | 3 months ago | 0 | OK |

These 4 match the master-plan headline of "4 crates". All compile cleanly,
so they are not breaking the build today — the bit-rot question is whether
they pull their weight given near-zero workspace integration.

## Per-crate notes

### `vais-playground-server`
- Self-contained Axum/SQLite server for the web playground.
- Web playground (`playground/` folder) is the consumer, but as a separate
  binary deployment, not a workspace lib import.
- Decision rationale candidates:
  - **Keep**: documented playground feature, quarantine status acceptable.
  - **Move out of workspace**: ship as a separate repo if playground itself
    moves out of the monorepo.

### `vais-query`
- Salsa-style query database. Used by `vaisc` (1 workspace dep).
- Not bit-rot in the strict sense; passes the heuristic only because the
  Cargo.toml hasn't changed in 3 months. The dependency from vaisc keeps it
  load-bearing.
- Decision: **Keep**. Heuristic is a false positive for this crate.

### `vais-supply-chain`
- SBOM and dependency-audit tooling. Standalone binary.
- Useful for release/audit workflows but not in CI today.
- Decision rationale candidates:
  - **Keep**: future-use insurance, low maintenance burden.
  - **Wire into release CI**: justify presence by adding a CI job that
    consumes its output.

### `vais-testgen`
- Property-based test generator. ~1,367 LOC implementation per memory.
- Not wired into any current test harness (tests/empirical/, integration
  tests, fuzz targets all skip it).
- Decision rationale candidates:
  - **Keep**: future test infrastructure.
  - **Wire into fuzz**: property gen + fuzz_full_pipeline could
    cross-check, justifying presence.
  - **Quarantine via Cargo `[workspace.exclude]`**: stop building it in
    `cargo test --workspace` until a consumer lands.

## Decision

Decision deferred to a follow-up plan-driven session. This audit identifies
the 4 candidates and lays out trade-offs per crate; the actual keep / wire /
quarantine choice is a destructive action that benefits from explicit user
approval.

The minimal mechanical action right now is to update master-plan.toml so
the optional bullet names the 4 crates (this file is the named source).

## Tracker

- `master-plan.toml` `[[optional]]` "Bit-rot crate decision (4 crates)" —
  awaiting user decision; this audit is the prerequisite recon.
