# I-5 Recovery Procedure (Master Plan Step 16)

This document is the **adjudication policy** for Step 16 (I-5 memory protocol).
Step 16 is the highest-risk item in the master plan: it adds a borrow-check
default that may break the existing self-hosting compiler. This file pins the
recovery decision tree so the project can roll back deterministically when
catastrophic breakage occurs.

> **Status**: pre-step 15a snapshot NOT yet executed (DEFERRED #26).
> This document is the *prerequisite recovery doc* per the master plan
> Step 16 sub_step_15a contract. The snapshot itself lands when Step 16
> formally starts.

## Pre-step 15a sub-step (snapshot)

### Deliverable

`compiler/selfhost.legacy/` directory containing the current `compiler/selfhost/`
contents at the moment Step 16 begins. The snapshot is the canonical
"last known good" reference for the borrow-check rewrite work.

### Constraints

- The snapshot must be produced via `git mv compiler/selfhost compiler/selfhost.legacy`
  (single rename, preserves git history per file).
- After `git mv`, **76 hardcoded `selfhost/` path references** across 7
  files must be updated to either keep working with the new active
  `selfhost/` (after re-creation) OR be split: tests that exercise the
  *legacy* selfhost continue to point at `selfhost.legacy/`; tests that
  exercise the *new* borrow-check selfhost point at `selfhost/`.
- Path reference inventory (verify before snapshot):
  - `crates/vaisc/tests/selfhost_clang_tests.rs` (21 refs)
  - `crates/vaisc/tests/cross_verify_tests.rs` (2)
  - `crates/vaisc/tests/selfhost_mir_tests.rs` (43)
  - `crates/vaisc/tests/selfhost_lexer_tests.rs` (7)
  - `crates/vaisc/tests/bootstrap_tests.rs` (1)
  - `crates/vaisc/src/commands/simple.rs` (1)
  - `crates/vaisc/tests/selfhost_stdlib_tests.rs` (1)
  - `scripts/cross-verify.sh` (multiple references; use `SELFHOST_DIR` env)
- Verification gate: `bash compiler/scripts/check-integrity.sh` must report
  `INTEGRITY OK` post-snapshot. Ecosystem and Core gates must stay green.

### Snapshot procedure (when Step 16 starts)

1. Record the git SHA before snapshot: `git -C compiler rev-parse HEAD`.
2. `cd compiler && git mv selfhost selfhost.legacy && git commit -m "step-16-pre-step-15a: selfhost → selfhost.legacy snapshot"`.
3. Update all 7 files' hardcoded `selfhost/` paths. Strategy:
   - If the test exercises legacy selfhost behavior, point at `selfhost.legacy/`.
   - If the test will be rewritten under the borrow-check model, leave
     pointing at `selfhost/` (which will be re-created with new content).
4. Run `bash compiler/scripts/check-integrity.sh`. Must report INTEGRITY OK.
5. Commit the path update separately: `step-16-pre-step-15a-paths: rewire 76 path refs`.
6. Record the post-snapshot SHA: `git -C compiler rev-parse HEAD`.

### Verification artifact

After snapshot, this section is updated with:
- Pre-snapshot SHA
- Post-snapshot SHA (after `git mv`)
- Post-path-rewire SHA (after step 5)
- INTEGRITY summary at each SHA

## Recovery decision tree

When Step 16 sub-stages (Owned/Borrow type definitions → MIR borrow check →
defer/drop runtime → unsafe-only raw pointer) cause INTEGRITY to drop, this
tree dictates the rollback decision.

### Trigger conditions

A "catastrophic breakage" event is when ANY of the following holds for ≥ 24
hours after a Step 16 commit:

- INTEGRITY std count drops > 5 files (e.g. 82 → 76).
- INTEGRITY vaisdb count drops > 10 files (e.g. 261 → 250).
- Any runtime smoke gate (vaisdb_runtime / vais-server_runtime /
  vais-web_runtime / http_client_runtime) regresses below the locked
  threshold.
- The `core_certification` gate fails on a previously-LANDED Core fixture.
- `cross_package_schema` gate fails on a previously-LANDED schema.

### Decision tree

```
Catastrophic breakage detected
│
├─ Cause is the most recent Step 16 commit?
│   └─ YES: git revert <commit>. Re-run check-integrity.sh.
│            If INTEGRITY OK: continue with smaller per-stage iter.
│            If still broken: cascade further reverts, one at a time.
│
├─ Cause is multiple commits across sub-stages?
│   └─ YES: revert to post-snapshot SHA (recorded above).
│            Rebuild Step 16 work in smaller increments with
│            INTEGRITY measurement after each.
│
└─ Cause unclear / appears systemic (e.g. selfhost.legacy itself broken)?
    └─ YES: revert to pre-snapshot SHA (recorded above).
             selfhost.legacy directory removed; the entire Step 16
             attempt is rolled back to the pre-step 15a state.
             Re-evaluate Step 16 scope before next attempt.
```

### Rollback verification

After any rollback:
1. `bash compiler/scripts/check-integrity.sh` must report INTEGRITY OK.
2. `bash scripts/check-plan-consistency.sh` must report PLAN CONSISTENCY OK
   (master-plan version may need to be reverted too, depending on what
   changed).
3. WORKLOG entry recording: rollback reason, commits reverted, new SHA,
   re-attempt plan.

### Adjudication ownership

- **Single revert** (most recent commit): claude can execute autonomously.
- **Cascade revert** (≥ 2 Step 16 commits reverted): claude must report to
  human before proceeding.
- **Pre-snapshot revert** (entire Step 16 attempt rolled back): human
  decision required. This is a strategic restart.

## Per-stage Step 16 plan (informational)

The actual sub-stages of Step 16 (per master plan) are:

1. Owned/Borrow/MutBorrow type definitions in vais-types.
2. MIR borrow check default (currently opt-in).
3. defer/drop runtime smoke certified.
4. Raw pointer / i64 conversion → `unsafe` only.
5. std rewrite using explicit Owned/Borrow.
6. vaisdb security TLS module certified.

Each sub-stage is ~1-2 months. Total budget 12-18 months including the
+6 month detour reserve. INTEGRITY measurement after every sub-stage
commit (not just per-month). Anti-pattern: "land all 6 sub-stages then
measure" — that produces unrecoverable state if step 4 broke step 1's
contract.

## L-005 application

Per LESSONS L-005 (multi-iter prior estimate vs empirical narrow scope):
each sub-stage's *actual* scope must be re-measured at the start. The
master-plan budget (1-2 months per sub-stage) is the *reserve*, not the
target. Single-session close is acceptable when measurement shows the
sub-stage is narrower than feared (precedent: Step 11 root fix landed
single-session despite multi-iter prior estimate).

## L-002 application

Per LESSONS L-002 (no silent failure / no implicit behavior): every
Step 16 sub-stage must add a TC-level diagnostic for the new strict
behavior. Silent acceptance of pre-borrow-check patterns under the
new default is a regression even if INTEGRITY count happens to stay
green. Example: A4-15 hard-block (loop 34, master-plan v39) is the
template — TC detector + opt-out env + permanent fixture.
