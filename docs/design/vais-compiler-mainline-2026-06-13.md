# Vais Compiler Mainline

Date: 2026-06-13

## Contract

`scripts/vaisc` is the compiler command for `.vais` source files.

```text
Vais source
  -> front checks and small source lowerings
  -> self-host compiler path
  -> LLVM IR
  -> clang/native binary
```

The direct engine is a deliberately narrow native LLVM emitter used to promote slices independently:

```bash
scripts/vaisc run examples/c4.vais --engine direct
```

## Current Implementation

- Full engine: `compiler/self/vaisc_core.ll` is linked with a small file-reading runner and calls the self-host `compile(src)` function.
- Direct engine: literal Int expression `main` slice.
- Front diagnostics: `tools/vaisc.py` and `tools/vais-check.py`.
- Value corpus: release-subset `examples/*.vais` files tracked in `tools/vaisc-parity.tsv`.

## Gates

```bash
python3 tests/vais_check_test.py
bash scripts/test-vaisc.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test.sh
```

## Next Step

Keep pure core regeneration from `compiler/self/fixpoint_full.vais` green, then broaden the release gates.
