# Vais Compiler Status

> Last updated: 2026-04-25 (Phase 0 kickoff)

## Conformance Test Results

`tests/lang/` — language feature conformance suite.

| Category | Tests | Passing | Status |
|----------|-------|---------|--------|
| 01_primitives | 9 | 9/9 | ✅ |
| 02_control_flow | 9 | 9/9 | ✅ |
| 03_match | 5 | 5/5 | ✅ |
| 04_struct | 6 | 6/6 | ✅ |
| 05_enum | 4 | 4/4 | ✅ |
| 06_generic | 4 | 4/4 | ✅ |
| 07_collections | 1 | 1/1 | ✅ |
| 08_strings | 2 | 2/2 | ✅ |
| 09_traits | 0 | — | not yet |
| 10_ffi | 0 | — | not yet |
| 99_integration | 4 | 4/4 | ✅ |
| **Total** | **44** | **44/44 (100%)** | 🎉 |

### Hello world examples — `examples/hello_world_v2/` (7/7 — 100%)

| File | Demonstrates | Exit |
|------|--------------|-----:|
| `01_hello.vais` | minimum program | 0 |
| `02_arithmetic.vais` | int arithmetic | 5 |
| `03_struct.vais` | struct + method | 7 |
| `04_option.vais` | Option<T> + match | 3 |
| `05_recursion.vais` | recursive fibonacci | 21 |
| `06_loop.vais` | while loop accumulator | 28 |
| `07_generic.vais` | generic max function | 15 |

### Hello world examples

`examples/hello_world_v2/` — user-facing demos.

| File | Demonstrates | Exit | Status |
|------|--------------|-----:|--------|
| `01_hello.vais` | minimum program | 0 | ✅ |
| `02_arithmetic.vais` | int arithmetic | 5 | ✅ |
| `03_struct.vais` | struct + method | 7 | ✅ |
| `04_option.vais` | Option<T> + match | 3 | ✅ |
| `05_recursion.vais` | recursive fibonacci | 21 | ✅ |
| **Total** | | | **5/5 (100%)** |

Run yourself:
```bash
cd compiler/tests/lang && ./run_lang_tests.sh
```

## Stability Tiers

### Core (production-track)
- `vais-lexer`, `vais-parser`, `vais-ast`, `vais-types`, `vais-codegen`, `vais-mir`, `vaisc`
- These crates produce LLVM IR. Trust target: 100% conformance suite green.

### Auxiliary (best-effort)
- `vais-lsp`, `vais-dap`, `vais-i18n`, `vais-plugin`, `vais-macro`, `vais-bindgen`
- Compile but not exhaustively tested. Known to break on uncommon edge cases.

### Experimental (may break)
- `vais-jit`, `vais-gpu`, `vais-gc`, `vais-codegen-js`, `vais-hotreload`,
  `vais-dynload`, `vais-profiler`, `vais-registry-server`,
  `vais-playground-server`, `vais-tutorial`, `vais-python`, `vais-node`,
  `vais-query`, `vais-testgen`, `vais-supply-chain`, `vais-security`
- Not part of v1.0 release scope. May not compile against current core.

## Known Issues

### 07_collections/vec_basic — link failure
- **What**: `Vec<i64>` push + indexing test compiles to IR but fails at clang link stage.
- **Symptom**: undefined symbol or type-mismatch link error.
- **Reason**: stdlib `Vec` requires multi-module specialization that the
  current vaisc driver doesn't fully wire for standalone test files.
- **Workaround**: vaisdb-style multi-module builds work; standalone
  single-file Vec usage doesn't.
- **Fix path**: Phase 0.B continued — extend test runner to detect and
  bundle stdlib dependencies, OR add a `vec_new` mono-typed shim to core
  stdlib.

### Phase 17 Wave 1-4a discovered bugs (this session)

These bugs were fixed in commits `7c3aed52`, `72616dc2`, `039df2f7`,
`32d1ed83`, `5a11bcf0` during Phase 17 Wave 4a probe and Phase 0
kickoff. Each has a corresponding `tests/lang/` regression test (or
should — see TODO):

| Bug | Test | Status |
|-----|------|--------|
| match default arm null literal | `tests/lang/03_match/match_phi_default_zero.vais` | ✅ landed |
| `vec[i] = struct_value` ptr store | TODO `tests/lang/07_collections/vec_struct_assign.vais` | pending |
| 4-byte Named struct narrow store | TODO `tests/lang/04_struct/struct_4_bytes_in_vec.vais` | pending |
| match arm phi narrow-int width | TODO `tests/lang/03_match/match_phi_narrow_int.vais` | pending |
| Specialized enum match (`%Unknown`) | TODO `tests/lang/05_enum/result_specialized_match.vais` | pending |
| Enum payload of >8B struct | TODO `tests/lang/05_enum/enum_struct_payload.vais` | pending |
| Vec→slice auto-coercion | TODO `tests/lang/07_collections/vec_to_slice.vais` | pending |
| `slice.to_vec()` builtin | (stdlib gap, not lang feature) | n/a |

## Active Work

- **Phase 0.A**: surface area audit (in progress)
- **Phase 0.B**: conformance suite — 13 tests landed, target 300+
- **Phase 0.C**: stdlib self-tests (not started)
- **Phase 0.D**: hello world examples (not started)

See `compiler/docs/PHASE_0_LANGUAGE_STABILIZATION.md` for full roadmap.

## Downstream Project Status

Following projects depend on Phase 0 v1.0 completion:
- **vaisdb** (`packages/vaisdb`): paused, see `packages/vaisdb/docs/MASTER_ROADMAP.md`
- **vais-web**: not started
- **vais-server**: experimental, depends on vaisdb

## Honesty Pledge

This file reflects current actual state, not aspiration. If a row says
✅, it really runs green. If a test count is N/M, those are the real
numbers from the most recent CI run.

If you read this file and find it inaccurate, please open an issue
or PR fixing it. Inaccurate STATUS.md is a P0 bug.
