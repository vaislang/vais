# Vais Worklog

## 2026-06-14

- Vais-only source surface enforced.
- Public compiler input is `.vais`.
- Removed wrapper tools and non-Vais gates.
- Updated README, ROADMAP, AGENTS, language reference, examples README, prelude notes, and self-host notes to current Vais status.
- Renamed temporary test sources to `.vais`.
- Added `.vais` suffix validation to `scripts/vaisc`, `scripts/build.sh`, `tools/embed_self_source.py`, and the internal source passthrough helper.
- `scripts/vaisc --engine` now exposes `full` and `direct`.
- `scripts/vaisc` full mode now uses `compiler/self/vaisc_core.ll` and reads `.vais` inputs directly.

## Current Remaining Work

- Repair pure core regeneration from `compiler/self/fixpoint_full.vais` into `compiler/self/vaisc_core.ll`.
- Expand the native direct emitter until it can cover the trusted self-host tier.
- Broaden release gates as the self-host surface grows.
