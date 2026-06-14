# Vais Documentation

This directory is the canonical documentation source for the current Vais
compiler workspace.

## Start Here

| Document | Purpose |
| --- | --- |
| [../README.md](../README.md) | Repository overview and quick commands |
| [../CHANGELOG.md](../CHANGELOG.md) | Current source release notes |
| [reference/LANGUAGE.md](reference/LANGUAGE.md) | Current Vais syntax and verified language surface |
| [../examples/README.md](../examples/README.md) | Value-checked example corpus |
| [../std/PRELUDE.md](../std/PRELUDE.md) | Prelude APIs and their verification status |
| [../compiler/self/SELF_HOST.md](../compiler/self/SELF_HOST.md) | Self-host compiler notes |
| [../website/DEPLOYMENT.md](../website/DEPLOYMENT.md) | Official site source and deployment notes |
| [design/VAIS-DESIGN.md](design/VAIS-DESIGN.md) | Short design contract |
| [design/vais-compiler-mainline-2026-06-13.md](design/vais-compiler-mainline-2026-06-13.md) | Compiler mainline contract |

## Documentation Rules

- Public docs must describe only the current `.vais` language and `scripts/vaisc`
  compiler path.
- Claims marked as verified must be backed by a gate in `scripts/` or
  `tools/vaisc-parity.tsv`.
- Release notes must be sourced from `CHANGELOG.md`.
- `ROADMAP.md`, `WORKLOG.md`, and `AGENTS.md` are project coordination files, not
  first-read user documentation.
- Do not publish older language, compiler, package, or ecosystem claims as
  current Vais documentation unless they are backed by this repository's gates.

## Public Site And GitHub Sync

The GitHub README and official site source live in this repository. The site
source is `website/`, and the public entry point should contain only:

1. What Vais is today.
2. How to compile and run a `.vais` file.
3. The current verified language surface.
4. The self-host compiler status.
5. Links to the exact gates that protect those claims.

Any public site content that advertises a different language syntax or
unverified ecosystem readiness is stale relative to this repository unless it is
rebuilt from `website/` and verified on the current compiler.
