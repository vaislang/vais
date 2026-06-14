# Vais

Vais is the mainline language and compiler workspace.

## Status

- Source files use `.vais`.
- The user-facing compiler command is `scripts/vaisc`.
- `scripts/vaisc emit-ir`, `scripts/vaisc build`, and `scripts/vaisc run` compile Vais source through the self-host compiler path.
- `--engine direct` is available for the narrow native LLVM emitter slice.
- `tools/vais-check.py` reports non-Vais spellings with source coordinates, `help:`, and a concrete fix.

## Layout

```text
compiler/
  self/                 self-host compiler sources
  self/vaisc_core.ll    reusable self-host compiler core
docs/
  reference/LANGUAGE.md language reference
examples/               value-checked Vais examples
scripts/
  vaisc                 compiler command
  test.sh               value-correctness corpus
  test-vaisc*.sh        compiler gates
tools/
  vais-check.py         lint/error-help tool
  vaisc.py              compiler CLI implementation
```

## Common Commands

```bash
scripts/vaisc emit-ir examples/c4.vais -o /tmp/c4.ll
scripts/vaisc build examples/c4.vais -o /tmp/c4
scripts/vaisc run examples/c4.vais

python3 tools/vais-check.py examples/c4.vais

python3 tests/vais_check_test.py
bash scripts/test-vaisc.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test.sh
```

## Current Direction

The compiler mainline is Vais. The release command uses the reusable self-host compiler core and normal toolchain dependencies only.
