# Empirical fixtures

Permanent fixtures for Master Plan v16 Order Step 7 (surface inventory lock + retro-validation).

## What lives here

Each subdirectory holds permanent `.vais` (and supporting) fixtures that
empirically prove a classification decision. v1 single-sentinel discoveries
are retro-validated to v2 (multi-sentinel + stdout + permanent fixture)
per Master Plan v16 Order Step 7 retro-validation contract.

```
empirical/
├── README.md            # this file
├── A4/                  # A4 removal queue fixtures (9 entries, runtime/late-codegen-silent)
│   ├── A4-01_unit_i64/  # Unit ↔ i64 (void return as i64)
│   ├── A4-02_pointer_i64/
│   ├── ...
│   └── A4-09_lifetime_ref_erasure/
├── controlled/          # Controlled coercions (9 entries) — proves they remain
│   │                      classified as Controlled, not A4
└── cross_package_schema/ # Cross-package schema gate fixture (consumed by
                            Order Step 8; design at compiler/docs/design/
                            cross-package-schema.md)
```

## Fixture format (v2 protocol — Empirical verification protocol §EXCLUDED_FEATURES.md)

Each fixture directory contains:

1. `README.md` — what surface this fixture probes, expected behavior, the
   classification it justifies (A4 / Controlled / Rejected / Untested).
2. `probe.vais` — the minimal Vais source that exhibits the surface.
3. `expected.txt` — the expected stdout / exit code.
4. `run.sh` — the runner that compiles `probe.vais`, executes it, and diffs
   against `expected.txt`. Exit 0 = surface still has the documented
   behavior; non-zero = compiler behavior changed (investigate before
   deciding direction).
5. `meta.toml` — machine-readable metadata: classification, declared site,
   v1 evidence, v2 evidence (multi-sentinel result), date.

The runner script is intentionally minimal — it exists so CI can iterate
all fixtures without a custom test harness. CI integration: a future
`compiler/scripts/check-empirical.sh` walks `compiler/tests/empirical/*/`
and runs each `run.sh`, summarizing pass/fail. Wiring is **not** required
by Step 7 itself — that is Step 13 (A4 removals) deliverable scope.

## v1 vs v2 evidence

- **v1 single-sentinel**: one probe demonstrates the surface. This is what
  the 9 existing A4 entries currently have (declared in master-plan.toml
  with single expected/actual values).
- **v2 multi-sentinel**: at least 3 distinct probes per surface, each
  producing a stable stdout that pins the surface to a specific
  observable. Plus the runner asserts on the exact stdout, not just
  exit code, so a future "wrong but exits 0" regression cannot pass.

Order Step 7 retro-validation contract: every v1 entry MUST be re-verified
under v2 before any A4 removal (Step 13) executes against it.

## Order of population

Per Master Plan v16 Step 7 (budget 2-4 months): fixed-point iteration.
Each round adds discoveries until no new entries. The 9 A4 entries are
seeded first; controlled/rejected/untested follow.

## How to add a new fixture

1. Pick the surface (must be classified per `EXCLUDED_FEATURES.md` already,
   or be a new candidate).
2. Create `<class>/<id>_<short_name>/` with the 5 files above.
3. Run `bash run.sh` locally — it must exit 0.
4. Commit. Step 13 / Step 7 closeout will pick up the new fixture.

## Discovery (2026-05-03, Step 7 first iteration)

When Step 7 began, this directory did not exist. Master Plan v16 declared
9 A4 entries as v1-verified, with the retro-validation contract pending
this directory's creation. The directory is created now; the first
fixture (A4-01) lands in the same iteration.
