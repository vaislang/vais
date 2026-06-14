# Vais

Vais is a `.vais` language and self-host compiler workspace. The public compiler
command is `scripts/vaisc`.

The current mainline is intentionally small and gate-backed: `scripts/vaisc`
builds a native host driver, links the reusable self-host compiler core in
`compiler/self/vaisc_core.ll`, emits LLVM IR, and links programs with `clang`.

## Quick Start

Requirement for the public compiler command: `clang`.

```bash
scripts/vaisc doctor
scripts/vaisc run examples/c4.vais
scripts/vaisc emit-ir examples/c4.vais -o /tmp/c4.ll
scripts/vaisc build examples/c4.vais -o /tmp/c4
```

The main command accepts `.vais` files only.

## Install

For a standalone `vaisc` binary outside the checkout:

```bash
scripts/install-vaisc.sh --prefix "$HOME/.local"
"$HOME/.local/bin/vaisc" doctor
"$HOME/.local/bin/vaisc" run examples/c4.vais
```

To remove that install:

```bash
scripts/uninstall-vaisc.sh --prefix "$HOME/.local"
```

To build a release archive:

```bash
scripts/package-vaisc-release.sh
```

## Documentation

| Start here | Purpose |
| --- | --- |
| [docs/README.md](docs/README.md) | Documentation map and publication rules |
| [CHANGELOG.md](CHANGELOG.md) | Current source release notes |
| [docs/reference/LANGUAGE.md](docs/reference/LANGUAGE.md) | Current syntax and verified language surface |
| [examples/README.md](examples/README.md) | Value-checked example corpus |
| [compiler/self/SELF_HOST.md](compiler/self/SELF_HOST.md) | Self-host compiler status |
| [website/DEPLOYMENT.md](website/DEPLOYMENT.md) | Official site source and deployment notes |

## Repository Layout

```text
compiler/self/          self-host compiler sources and reusable core
docs/                   canonical documentation
examples/               .vais examples; release subset tracked by parity manifest
scripts/vaisc           native public compiler command
scripts/install*.sh     standalone compiler install/uninstall helpers
scripts/package*.sh     standalone release archive helper
scripts/test*.sh        compiler and value-correctness gates
std/PRELUDE.md          prelude API status
tools/                  native driver source, internal checks, parity manifest, source embedding tools
website/                official vaislang.dev static site source
```

## Verification

Public compiler smoke:

```bash
scripts/vaisc doctor
bash scripts/test-vaisc-native.sh
bash scripts/test-vaisc-install.sh
bash scripts/test-vaisc.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test.sh

bash scripts/test-fixpoint-full-self.sh
bash scripts/test-fixpoint-full.sh
```

The public language and compiler claims in this repository are limited to what
these gates protect.
