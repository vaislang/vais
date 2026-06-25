# Vais Self-Host Compiler

`compiler/self/*.vais` contains the compiler work written in Vais.

## Trusted Tier

| Source | Coverage | Gate |
| --- | --- | --- |
| `fixpoint.vais` | arithmetic tokenizer/evaluator tier | `scripts/test-fixpoint.sh` |
| `fixpoint2.vais` | variables and symbol lookup tier | `scripts/test-fixpoint2.sh` |
| `fixpoint3.vais` | functions, calls, recursion tier | `scripts/test-fixpoint-full-self.sh` |
| `fixpoint_full.vais` | unified source-to-LLVM compiler tier | product `scripts/vaisc` release gates |

The focused `fixpoint.vais` and `fixpoint2.vais` gates are driven by
`tools/fixpoint_tier_check.vais`; their shell scripts only build and launch
that Vais-authored harness.

## Compiler-Core Regeneration

`compiler/self/vaisc_core.ll` is the checked-in compiler core linked into the
native `scripts/vaisc` public command. The core regenerates from
`compiler/self/fixpoint_full.vais` and is verified by the self-host gates.

## Architecture Notes

- Tokens carry source ranges `(nstart, nlen)` so identifier comparison is byte-accurate.
- Recursive evaluator/compiler tiers use explicit symbol tables and function tables.
- Codegen emits real LLVM IR and validates it by compiling the emitted IR with clang.
- `tools/vaisc_native.c` is the host driver for CLI, source preparation, file IO,
  and clang invocation; the compiler core remains the self-host Vais tier.
- `tools/embed_self_source.vais` is the Vais-native helper used to retarget
  `fixpoint_full.vais` at real `.vais` source files during self-host gates.
  It also has raw compact-program modes used by `scripts/test-fixpoint.sh`,
  `scripts/test-fixpoint2.sh`, and `scripts/test-fixpoint-full.sh`, so the
  fixpoint gates no longer need Python to synthesize compiler harness inputs.
  `tools/embed_self_source_check.vais` exercises normalized source-file
  retargeting, raw compile embedding, generated compiler build/run behavior,
  and raw string-call retargeting; `scripts/test-embed-self-source-vais.sh`
  is only the bootstrap wrapper for that Vais-authored gate.
- `tools/normalize_stage_ir.vais` is the Vais-native helper used for
  source-position-independent stage IR comparison in the long self-host gate.
  `scripts/test-normalize-stage-ir-vais.sh` checks the expected normalized IR
  shape directly through the Vais helper.
- `tools/vais_manifest_check.vais` is the Vais-authored package manifest
  contract checker. `scripts/test-vais-manifest-check-vais.sh` exercises the
  current manifest diagnostic surface, including missing source directories and
  local dependency cycles. It can also validate an optional entry path against
  the manifest source root while the product driver remains responsible for
  OS-facing package discovery and module graph loading.
- `tools/vais_import_graph_check.vais` is the Vais-authored local import graph
  contract checker for the manifest-free missing import, duplicate symbol, and
  import cycle diagnostics.
- Lists, structs, string indexing, control flow, function calls, and print emission are covered by the release gates.

## Next Work

Keep core regeneration, self-host fixpoint, and release gates green as compiler coverage expands.
