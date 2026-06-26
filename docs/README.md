# Vais Documentation

This directory is the canonical documentation source for the current Vais
compiler workspace.

Current source release: `v0.3.2`, with standalone compiler/checker archives
published by the release workflow for Linux x64, macOS arm64, and macOS x64.

## Start Here

| Document | Purpose |
| --- | --- |
| [https://vaislang.dev/](https://vaislang.dev/) | Korean/English public homepage and separate tutorial playground |
| [../README.md](../README.md) | Repository overview and quick commands |
| [../CHANGELOG.md](../CHANGELOG.md) | Current source release notes |
| [release/RELEASE_CHECKLIST.md](release/RELEASE_CHECKLIST.md) | Pre-tag release checklist |
| [reference/LANGUAGE.md](reference/LANGUAGE.md) | Current Vais syntax and verified language surface |
| [../examples/README.md](../examples/README.md) | Value-checked example corpus |
| [../std/PRELUDE.md](../std/PRELUDE.md) | v1-candidate prelude APIs and their verification status |
| [../compiler/self/SELF_HOST.md](../compiler/self/SELF_HOST.md) | Self-host compiler notes |
| [../website/DEPLOYMENT.md](../website/DEPLOYMENT.md) | Official site source and deployment notes |
| [design/VAIS-DESIGN.md](design/VAIS-DESIGN.md) | Short design contract |
| [design/vais-compiler-mainline-2026-06-13.md](design/vais-compiler-mainline-2026-06-13.md) | Compiler mainline contract |
| [design/MODULES.md](design/MODULES.md) | Current Phase 2 module/package/import model |
| [design/HOST_IO.md](design/HOST_IO.md) | Phase 3 file/process host API model and file/path/process intrinsic gate |
| [design/MAP_ABI.md](design/MAP_ABI.md) | Map ABI and generic expansion contract before broader `Map<K,V>` gates |
| [../tools/vais_check_core.vais](../tools/vais_check_core.vais) | Vais-authored checker rules, gated by fixture contract tests |
| [../tools/vais_check_cli.vais](../tools/vais_check_cli.vais) | Source for the public `scripts/vais-check` checker command |

## Documentation Rules

- Public docs must describe only the current `.vais` language, `scripts/vaisc`
  compiler path, and `scripts/vais-check` checker path.
- Claims marked as verified must be backed by a gate in `scripts/` or
  `tools/vaisc-parity.tsv`.
- Release notes must be sourced from `CHANGELOG.md`.
- Public release tags must follow [release/RELEASE_CHECKLIST.md](release/RELEASE_CHECKLIST.md).
- `ROADMAP.md`, `WORKLOG.md`, and `AGENTS.md` are project coordination files, not
  first-read user documentation.
- Do not publish older language, compiler, package, or ecosystem claims as
  current Vais documentation unless they are backed by this repository's gates.

## Public Site And GitHub Sync

The GitHub README and official site source live in this repository. The site
source is `website/`, and the public entry point should contain only:

1. What Vais is today.
2. How to compile and run a `.vais` file.
3. How to install or package the standalone native `vaisc` and `vais-check`
   binaries.
4. Which release archives and automation publish standalone compiler assets.
5. The current verified language surface.
6. The self-host compiler status.
7. Links to the exact gates that protect those claims.
8. The separate Korean/English playground for editable tutorial examples,
   clearly labeled as a browser subset runner rather than a full hosted
   compiler.

Any public site content that advertises a different language syntax or
unverified ecosystem readiness is stale relative to this repository unless it is
rebuilt from `website/` and verified on the current compiler.
