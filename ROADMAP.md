# Vais Roadmap

This file tracks current work only.

## Done

- Project path is `/Users/sswoo/study/projects/vais`.
- Checked-in language sources use `.vais`.
- `scripts/vaisc` is the canonical compiler command.
- `tools/vais-check.py` is the canonical lint/error-help command.
- The workspace now exposes only Vais source and Vais commands.
- The compiler gates cover CLI smoke, front-contract diagnostics, direct LLVM emission, parity, and the value corpus.
- The trusted self-host tier is `compiler/self/fixpoint.vais`, `fixpoint2.vais`, `fixpoint3.vais`, and `fixpoint_full.vais`.
- `compiler/self/vaisc_core.ll` is the reusable self-host compiler core used by `scripts/vaisc`.
- The full compiler path reads `.vais` source files directly through the self-host core.
- Pure regeneration of `compiler/self/vaisc_core.ll` from `compiler/self/fixpoint_full.vais` is green.

## Current Reality

- The full compiler path emits LLVM IR through the self-host compiler source in `compiler/self/fixpoint_full.vais`.
- The direct engine is intentionally narrow and currently supports a single `fn main() -> Int { return <Int expr> }` style slice.
- The release compiler command uses a native host driver for normal user
  `emit-ir`, `build`, and `run`; Python remains for internal checks and the
  development-only direct engine fallback.
- Internal compiler gates no longer depend on a source pass-through helper.
- Public documentation now starts at `README.md` and `docs/README.md`.
- `docs/reference/LANGUAGE.md` describes only the current gate-backed language surface.
- Local official website source was refreshed and rebuilt from the canonical Vais docs.
- Official site source now lives in `website/` in this repository.
- GitHub Pages workflow was added for `website/` build and artifact deployment.
- `vaislang.dev` deploys from the `website/` GitHub Pages workflow on `main`.
- `CHANGELOG.md` records the current `v0.2.0` source release baseline.
- GitHub `main` now points to the current Vais-only history; old remote `main`
  is preserved at `archive/old-main-2026-06-14`.

## Next Work

1. Package the native `scripts/vaisc` path for release installs outside a
   checkout, including install/uninstall scripts or release archives.
2. Keep README, language docs, website copy, and `CHANGELOG.md` synced with the
   Python-free public command path.
3. Replace the remaining Python-only internal checks and direct-engine fallback
   when the language has enough file/process support.
4. After the release command is Python-free, expand the direct emitter beyond
   the current slice: helper calls, locals, control flow, structs, lists, and
   the trusted self-host tier.
5. Keep source release tags, GitHub Releases, GitHub Pages, self-host regeneration, and parity gates green.

## Active Milestone: Python-Free Public `vaisc`

Mode: sequential

- [x] 1. Native driver skeleton and build script.
- [x] 2. Release source-preparation parity with the current Python path.
- [x] 3. `scripts/vaisc` default switch and install/doctor UX.
- [x] 4. Documentation/site/changelog sync and release gates.

### Task Briefs

#### 1. Native driver skeleton and build script

- Target files: `tools/`, `scripts/`, `README.md`, `ROADMAP.md`.
- Requirements: compile a native `vaisc` binary from a small host driver and `compiler/self/vaisc_core.ll`; support `emit-ir`, `build`, `run`, `--help`, `--version`, and `doctor` for the full engine path.
- Done: a local native binary can compile and run `examples/c4.vais` without invoking Python.

#### 2. Release source-preparation parity

- Target files: native driver/source-prep implementation and existing `scripts/test-vaisc*.sh` gates.
- Requirements: match the release subset behavior currently implemented by `tools/vaisc.py`, including enum/match, payload enum, closure-return, typed `Int`, `print`, comments, and semicolon normalization.
- Done: `bash scripts/test-vaisc.sh`, `bash scripts/test-vaisc-front.sh`, `bash scripts/test-vaisc-errors.sh`, `bash scripts/test-vaisc-parity.sh`, and `bash scripts/test.sh` pass through the native public command.

#### 3. Public command switch and install UX

- Target files: `scripts/vaisc`, packaging/install scripts, README docs.
- Requirements: `scripts/vaisc` uses the native driver by default; Python is not required for normal user `emit-ir`, `build`, or `run`; `doctor` reports missing `clang` or missing native driver clearly.
- Done: a fresh checkout can build the native driver and run `scripts/vaisc doctor`, `scripts/vaisc run examples/c4.vais`, and `scripts/vaisc build examples/c4.vais -o /tmp/c4`.

#### 4. Documentation, release, and gates

- Target files: `README.md`, `docs/README.md`, `docs/reference/LANGUAGE.md`, `compiler/self/SELF_HOST.md`, `website/`, `CHANGELOG.md`, `WORKLOG.md`.
- Requirements: public docs describe the Python-free command path only after verification; internal Python tools remain documented as checks, not as user runtime prerequisites.
- Done: verification baseline plus self-host gates pass, the website builds, stale public-claim scan is clean, and GitHub/site release notes are synced.

## Verification Baseline

Run before closing compiler changes:

```bash
python3 -m py_compile tools/vaisc.py tools/vais-check.py tools/embed_self_source.py tests/vais_check_test.py
bash -n scripts/*.sh
python3 tests/vais_check_test.py
bash scripts/test-vaisc.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test.sh
```
