# Vais Self-Host Compiler

`compiler/self/*.vais` contains the compiler work written in Vais.

## Trusted Tier

| Source | Coverage | Gate |
| --- | --- | --- |
| `fixpoint.vais` | arithmetic tokenizer/evaluator tier | `scripts/test-fixpoint.sh` |
| `fixpoint2.vais` | variables and symbol lookup tier | `scripts/test-fixpoint2.sh` |
| `fixpoint3.vais` | functions, calls, recursion tier | `scripts/test-fixpoint-full-self.sh` |
| `fixpoint_full.vais` | unified source-to-LLVM compiler tier | product `scripts/vaisc` release gates |

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
- Lists, structs, string indexing, control flow, function calls, and print emission are covered by the release gates.

## Next Work

Keep core regeneration, self-host fixpoint, and release gates green as compiler coverage expands.
