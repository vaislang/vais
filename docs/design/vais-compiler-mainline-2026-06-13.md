# Vais Compiler Mainline

Date: 2026-06-13

## Contract

`scripts/vaisc` is the compiler command for `.vais` source files.

```text
Vais source
  -> front checks and small source lowerings
  -> native host driver linked with the self-host compiler core
  -> LLVM IR
  -> clang/native binary
```

The direct engine is a deliberately narrow native LLVM emitter used to promote slices independently:

```bash
scripts/vaisc run examples/c4.vais --engine direct
```

## Current Implementation

- Full engine: `tools/vaisc_native.c` is linked with `compiler/self/vaisc_core.ll`
  and calls the self-host `compile(src)` function.
- Direct engine: promoted native slices for Int helpers, locals, control flow,
  calls, returns, simple Int-field struct local literal/read/write, struct
  parameter/return helpers, and `List<Int>` local, parameter, inline, return,
  returned-argument, and loop-hoisted operations, plus local `List<Struct>`
  construction with `[]`, `list()`, list literals, `push`, `len`, index, and
  field reads.
- Front diagnostics: native `scripts/vaisc`, `tools/vaisc.py` for internal
  repository checks, and `tools/vais-check.py`.
- Value corpus: release-subset `examples/*.vais` files tracked in `tools/vaisc-parity.tsv`.

## Gates

```bash
python3 tests/vais_check_test.py
bash scripts/test-vaisc-native.sh
bash scripts/test-vaisc.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test.sh
```

## Next Step

Keep pure core regeneration from `compiler/self/fixpoint_full.vais` green, then broaden the release gates.
