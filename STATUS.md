# Vais Compiler Status

> Last updated: 2026-04-25 (Phase 0.C stdlib self-test green)

## Conformance Test Results

`tests/lang/` — language feature conformance suite.

| Category | Tests | Passing | Status |
|----------|-------|---------|--------|
| 01_primitives | 11 | 11/11 | ✅ |
| 02_control_flow | 10 | 10/10 | ✅ |
| 03_match | 6 | 6/6 | ✅ |
| 04_struct | 7 | 7/7 | ✅ |
| 05_enum | 5 | 5/5 | ✅ |
| 06_generic | 5 | 5/5 | ✅ |
| 07_collections | 2 | 2/2 | ✅ |
| 08_strings | 2 | 2/2 | ✅ |
| 09_traits | 0 | — | not yet |
| 10_ffi | 0 | — | not yet |
| 99_integration | 6 | 6/6 | ✅ |
| **Total** | **55** | **55/55 (100%)** | 🎉 |

Run yourself:
```bash
cd compiler/tests/lang && bash run_lang_tests.sh
```

## Hello World Examples — `examples/hello_world_v2/` (12/12)

| File | Demonstrates | Exit | Status |
|------|--------------|-----:|--------|
| `01_hello.vais` | minimum program | 0 | ✅ |
| `02_arithmetic.vais` | int arithmetic | 5 | ✅ |
| `03_struct.vais` | struct + method | 7 | ✅ |
| `04_option.vais` | Option<T> + match | 3 | ✅ |
| `05_recursion.vais` | recursive fibonacci | 21 | ✅ |
| `06_loop.vais` | while accumulator | 28 | ✅ |
| `07_generic.vais` | generic max | 15 | ✅ |
| `08_match.vais` | match on int | 3 | ✅ |
| `09_result.vais` | Result + match | 5 | ✅ |
| `10_vec.vais` | Vec push/index | 15 | ✅ |
| `11_nested_struct.vais` | nested struct | 20 | ✅ |
| `12_combo.vais` | generic + Vec + recursion | 33 | ✅ |
| **Total** | | | **12/12 (100%)** |

Run yourself:
```bash
cd compiler/examples/hello_world_v2 && make check
```

## Stdlib Self-Tests — `compiler/std/tests/` (5/5)

| Test | Module | Assertions | Status |
|------|--------|-----------:|--------|
| `test_vec.vais` | std/vec | 10 | ✅ |
| `test_option.vais` | builtin Option<T> + std/option methods | 9 | ✅ |
| `test_result.vais` | builtin Result<T,E> | 5 | ✅ |
| `test_bytebuffer.vais` | std/bytebuffer | 21 | ✅ |
| `test_string.vais` | std/string + helpers | 28 | ✅ |
| `xfail_hashmap_strhashmap.vais` | std/hashmap StrHashMap | — | ⚠️ XFAIL (compiler bug) |
| `xfail_sync_mutexguard_specialization.vais` | std/sync Mutex | — | ⚠️ XFAIL (compiler bug) |

Run yourself:
```bash
cd compiler/std/tests && bash run.sh
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

### Phase 0.C-discovered compiler bugs

| # | Bug | Trigger | Workaround | Test |
|---|-----|---------|------------|------|
| C1 | `B <value>` (break-with-value) lowered as constant 0 in phi | `L { I cond { B 1 } I cond2 { B 0 } i = i + 1 }` returns wrong values | rewrite as `R <value>` outside the loop | std/string `str_eq` patched in this session |
| C2 ✅ | ~~`:= <int>` immutable bindings reassigned via `=` silently miscompile~~ FIXED in checker_expr/special.rs — TC now emits `ImmutableAssign` (E009) on Expr::Assign / Expr::AssignOp when target is a non-mut Ident binding. Regression test: `tests/lang/02_control_flow/mut_reassign.vais`. | — | — | mut_reassign.vais |
| C3 | StrHashMap<i64> generic specialization duplicate symbol cross-module | `U std/hashmap; m: StrHashMap<i64> := ...` | none — link fails | `xfail_hashmap_strhashmap.vais` |
| C4 | `Mutex<T>::lock` returns `MutexGuard` (unspecialized) instead of `MutexGuard$T` | calling `.lock()` on a `Mutex<i64>` | none — link fails on type mismatch | `xfail_sync_mutexguard_specialization.vais` |
| C5 ✅ | ~~`String.with_capacity(n)` segfaults when `n < 16`~~ FIXED in std/string.vais: `new_cap := self.cap * 2` is now `:= mut`. Root cause: codegen alloca'd `new_cap` but skipped the initial store; only the `< 16` branch wrote to it, leaving the else branch reading uninitialized memory → `malloc(garbage)` crash. Underlying codegen bug remains (separate finding: alloca without initial store when binding is later reassigned). | — | — | test_string.vais cap=4 |

### Phase 17 Wave 1-4a discovered bugs (prior sessions)

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
| Enum payload of >8B struct | `tests/lang/05_enum/enum_struct_payload.vais` | ✅ landed |
| Vec→slice auto-coercion | TODO `tests/lang/07_collections/vec_to_slice.vais` | pending |
| `slice.to_vec()` builtin | (stdlib gap, not lang feature) | n/a |

## Active Work

- **Phase 0.A**: surface area audit (in progress, doc landed)
- **Phase 0.B**: conformance suite — 54 tests landed, target 300+
- **Phase 0.C**: stdlib self-tests — 5 tests landed (1 XFAIL each for hashmap, sync)
- **Phase 0.D**: hello world examples — 12/12 ✅
- **Phase 0.F**: CI policy (doc landed, CI not wired)

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
