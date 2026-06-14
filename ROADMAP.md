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
- The native compiler can be installed as a standalone `vaisc` binary outside
  the checkout and packaged as a release archive.
- Source tag builds have a release archive workflow for standalone compiler
  assets.
- The native direct engine covers Int helper calls, locals, assignment, `if`,
  `while`, and returns without invoking Python.

## Current Reality

- The full compiler path emits LLVM IR through the self-host compiler source in `compiler/self/fixpoint_full.vais`.
- The direct engine is intentionally narrow and currently supports Int-only
  helpers, locals, assignment, calls, `if`, `while`, and returns.
- The release compiler command uses a native host driver for normal user
  `emit-ir`, `build`, and `run`; Python remains for internal checks and the
  development-only direct engine fallback.
- Standalone install, uninstall, package, and install/package verification
  scripts exist for the native compiler binary.
- Internal compiler gates no longer depend on a source pass-through helper.
- Public documentation now starts at `README.md` and `docs/README.md`.
- `docs/reference/LANGUAGE.md` describes only the current gate-backed language surface.
- Local official website source was refreshed and rebuilt from the canonical Vais docs.
- Official site source now lives in `website/` in this repository.
- GitHub Pages workflow was added for `website/` build and artifact deployment.
- `vaislang.dev` deploys from the `website/` GitHub Pages workflow on `main`.
- `CHANGELOG.md` records the current `v0.2.1` source release baseline.
- GitHub `main` now points to the current Vais-only history; old remote `main`
  is preserved at `archive/old-main-2026-06-14`.

## Next Work

1. Keep standalone release archives attached to future source tags.
2. Extend the direct engine toward structs and lists after the Int control-flow
   slice.
3. Keep README, language docs, website copy, and `CHANGELOG.md` synced with the
   Python-free public command path.
4. Replace the remaining Python-only internal checks when the language has
   enough file/process support.
5. Keep source release tags, GitHub Releases, GitHub Pages, self-host regeneration, and parity gates green.

## Completed Milestone: Release Automation And Native Direct Int Slice

Mode: sequential

- [x] 1. Add release archive workflow for source tags.
- [x] 2. Remove the public direct-engine Python fallback.
- [x] 3. Expand the native direct engine through Int helper calls, locals, assignment, `if`, and `while`.
- [x] 4. Sync README, language docs, website copy, changelog, and gates.

### Task Briefs

#### 1. Release archive workflow

- Target files: `.github/workflows/release-archives.yml`, `scripts/package-vaisc-release.sh`.
- Requirements: tag builds package standalone compiler archives and upload them to the matching GitHub Release.
- Done: workflow builds Linux/macOS archive jobs, smokes packaged `vaisc`, creates the release when needed, and uploads archives.

#### 2. Native direct path

- Target files: `scripts/vaisc`, `tools/vaisc_native.c`.
- Requirements: `--engine direct` must stay on the native driver and must not invoke Python.
- Done: `scripts/test-vaisc-direct.sh` proves direct mode still works with a failing `python3` shim first in `PATH`.

#### 3. Direct Int control-flow slice

- Target files: `tools/vaisc_native.c`, `scripts/test-vaisc-direct.sh`, `scripts/test-vaisc-errors.sh`.
- Requirements: direct mode accepts Int helper functions, locals, assignment, calls, `if`, `while`, and returns; unsupported identifiers keep P4 diagnostics.
- Done: direct tests cover arithmetic, helper calls, locals, control flow, full-engine parity, and P4 errors.

#### 4. Documentation and gates

- Target files: `README.md`, `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `AGENTS.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements: public docs describe current native direct and release archive automation without publishing unsupported direct structs/lists/self-host claims.
- Done: docs/site/changelog are synced and release gates pass.

## Completed Milestone: Standalone Install And Release Archive

Mode: sequential

- [x] 1. Add install and uninstall scripts for standalone `vaisc`.
- [x] 2. Add release archive packaging for the native binary and first-read docs.
- [x] 3. Add an install/package gate that proves installed and packaged binaries run.
- [x] 4. Sync docs/site/changelog and run release gates.

### Task Briefs

#### 1. Standalone install and uninstall

- Target files: `scripts/install-vaisc.sh`, `scripts/uninstall-vaisc.sh`.
- Requirements: build the native compiler from the checked-in self-host core and install it as `PREFIX/bin/vaisc`; uninstall removes that binary.
- Done: installing into a temporary prefix creates an executable `vaisc`, and uninstall removes it.

#### 2. Release archive packaging

- Target files: `scripts/package-vaisc-release.sh`, `.gitignore`.
- Requirements: build a standalone archive containing `bin/vaisc` and the current first-read docs; keep generated archives out of git.
- Done: the package script creates `dist/vais-VERSION-OS-ARCH.tar.gz`.

#### 3. Install/package gate

- Target files: `scripts/test-vaisc-install.sh`, `AGENTS.md`, `README.md`.
- Requirements: verify installed and packaged binaries can report version, run `doctor`, and compile/run a `.vais` smoke source.
- Done: `bash scripts/test-vaisc-install.sh` passes without writing outside a temporary directory.

#### 4. Documentation, site, and gates

- Target files: `README.md`, `docs/README.md`, `docs/reference/LANGUAGE.md`, `website/`, `CHANGELOG.md`, `ROADMAP.md`, `WORKLOG.md`.
- Requirements: public docs describe checkout use, standalone install, uninstall, package, and the gate protecting them.
- Done: docs and site are synced, website builds, stale public-claim scan is clean, and release gates pass.

## Completed Milestone: Python-Free Public `vaisc`

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
bash scripts/test-vaisc-native.sh
bash scripts/test-vaisc-install.sh
bash scripts/test-vaisc.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test.sh
```
