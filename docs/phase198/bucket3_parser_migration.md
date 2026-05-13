# Phase 198 Bucket 3 — Parser grammar migration

**Date**: 2026-04-18
**Scope**: Migrate P001 parser-error source patterns in vaisdb + vais-server tests.
**Status**: **Deferred to Phase 199** (scope too large for a single sub-agent pass).

## What we tried

A sonnet background agent was dispatched with the recon+fix plan and a 35-tool-call budget. It returned after 33 tool calls mid-analysis ("Let me look at the existing valid Vais syntax to understand correct equivalents:") having written **zero** file changes.

## Why deferred

Manual sampling of the 47 vaisdb P001 errors revealed they are **not a single sub-pattern** that can be batch-substituted. Minimum three distinct root causes, each requiring case-by-case analysis:

1. **Missing comma between match arms**
   - `src/fulltext/ddl.vais:109` — `Ok(x) => { ... } Err(e) => { ... }` (no comma after `}`)
   - Vais requires comma between arms; downstream projects commonly omit it.
   - **Fix**: add `,` before every `Err(...) =>`/`Ok(...) =>` arm that follows a block arm. Risk: needs to distinguish match arms from nested blocks — a simple regex is brittle.

2. **Old trait impl syntax**
   - `src/fulltext/concurrency.vais:204` — `X Drop for FullTextReadGuard { ... }` (Rust-style).
   - Current Vais: `X FullTextReadGuard: Drop { ... }`.
   - **Fix**: per-file rewrite of the header line and (where present) `self` parameter conventions.

3. **Unknown / not yet sampled**
   - 45 other P001 files unexamined. Likely more sub-patterns (the `+ "str"` continuation-line pattern from vais-server oauth.vais suggests at least one more).

## vais-server tests P001 × 7

Phase 197 vais-server report already classified these: all C-style `I i = 0; i < n; i = i + 1 { }` loops. Single sub-pattern → tractable. But even this requires manual per-file translation because `LF i:0..n` only works when the condition is a simple `<` on a known bound.

## Scope decision

Bucket 3 in its current form spans **≥3 root causes across 54 files**. Trying to batch-migrate them risks introducing new errors (missing commas, misplaced `LF` scopes, broken trait impl blocks) faster than it fixes existing ones. Given:

- The compiler side is fully green (179/179 examples, E2E 2596/0/0, clippy 0/0) — the stability baseline is preserved.
- Phase 197 established that these are **pre-existing dialect drift**, not Phase 195/196 regressions.
- A proper fix needs per-file review with `vaisc check` after each edit.

**Defer to Phase 199.** Phase 199 plan should split Bucket 3 into:
- **B3-a**: match-arm comma (sub-pattern 1) — probably scriptable after we write a `.vais` comma-inserter helper.
- **B3-b**: trait impl syntax (sub-pattern 2) — per-file manual migration, each file touched and `vaisc check`-ed.
- **B3-c**: C-style for-loop (vais-server tests + rest of vaisdb) — `LF` conversion with case-by-case bound analysis.
- **B3-d**: everything else — catalog what's left after a/b/c collapse, judge per case.

## What Phase 198 does instead

Continue with Bucket 4 (stdlib function audit) and Bucket 7 (tail cleanup). Those are better suited to single-agent scope:
- Bucket 4: name-mapping is algorithmic (grep + diff), high leverage.
- Bucket 7: small (~8 files), each independent.

Bucket 3's **7 vais-server test P001s** may be picked up opportunistically during Bucket 7 since they're all the C-style for-loop sub-pattern.

## PROMISE

**PROMISE: INCOMPLETE** — deferred, rationale above. Task reverted to `pending`; Phase 199 plan to carry it forward.
