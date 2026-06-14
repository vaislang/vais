# Vais

Vais is a `.vais` language and self-host compiler workspace. The public compiler
command is `scripts/vaisc`.

The current mainline is intentionally small and gate-backed: source files compile
through the reusable self-host compiler core in
`compiler/self/vaisc_core.ll`, emit LLVM IR, and are linked with `clang`.

## Quick Start

Requirements: Python 3 and `clang`.

```bash
scripts/vaisc run examples/c4.vais
scripts/vaisc emit-ir examples/c4.vais -o /tmp/c4.ll
scripts/vaisc build examples/c4.vais -o /tmp/c4
```

The main command accepts `.vais` files only. A narrow direct LLVM emitter is
available for promoted slices:

```bash
scripts/vaisc run examples/c4.vais --engine direct
```

## Documentation

| Start here | Purpose |
| --- | --- |
| [docs/README.md](docs/README.md) | Documentation map and publication rules |
| [docs/reference/LANGUAGE.md](docs/reference/LANGUAGE.md) | Current syntax and verified language surface |
| [examples/README.md](examples/README.md) | Value-checked example corpus |
| [compiler/self/SELF_HOST.md](compiler/self/SELF_HOST.md) | Self-host compiler status |
| [website/DEPLOYMENT.md](website/DEPLOYMENT.md) | Official site source and deployment notes |

## Repository Layout

```text
compiler/self/          self-host compiler sources and reusable core
docs/                   canonical documentation
examples/               .vais examples; release subset tracked by parity manifest
scripts/vaisc           compiler command
scripts/test*.sh        compiler and value-correctness gates
std/PRELUDE.md          prelude API status
tools/                  compiler CLI, checks, parity manifest, source embedding tools
website/                official vaislang.dev static site source
```

## Verification

```bash
python3 tests/vais_check_test.py
bash scripts/test-vaisc.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test.sh

bash scripts/test-fixpoint-full-self.sh
bash scripts/test-fixpoint-full.sh
```

The public language and compiler claims in this repository are limited to what
these gates protect.
