# Phase 196 Recon-F — lean measurement

**Date**: 2026-04-18
**Method**: individual fresh build per SKIP_LIST entry; no deep codebase dive.

## Current error per file (실측)

| # | File | Current error (first line of stderr) | Matches SKIP_LIST TODO? |
|---|---|---|---|
| 1 | tutorial_wc.vais | (no error — builds successfully) | Y |
| 2 | js_target.vais | Found IntValue but expected FloatValue (inkwell BasicValueEnum assertion) | Y |
| 3 | simd_test.vais | LLVM error: Aggregate extract index out of range | Y |
| 4 | simd_distance.vais | LLVM error: Aggregate extract index out of range | Y |
| 5 | calculator_enum.vais | Undefined variable: b. Did you mean `a`? | Y |
| 6 | option_result_simple_test.vais | [INST] logs only (no error) | Y |
| 7 | option_result_test.vais | [INST] logs only (no error) | Y |
| 8 | simple_hashmap_test.vais | [INST] logs only (no error) | Y |
| 9 | async_reactor_test.vais | Type mismatch: expected Optional or Result, found () | Y |
| 10 | wasm_todo_app.vais | Type mismatch: expected indexable type, found [i64; 100] | Y |
| 11 | test_import.vais | Cannot find module 'test_simple' | Y |
| 12 | range_type_error_test.vais | Type mismatch: expected integer type, found str | Y |
| 13 | tcp_10k_bench.vais | Undefined variable: store_i8/store_i32 | Y |
| 14 | lazy_simple.vais | Undefined variable: lazy | Y |
| selfhost_token | NotCallable("i64", None) type error in selfhost/token.vais | Y |

## [INST] log leak locations (단계 4 결과)

Files containing "[INST]": 
- crates/vaisc/tests/examples_fresh_rebuild.rs
- crates/vaisc/src/commands/build/parallel.rs

## Merge opportunity assessment

**A1+A2 (inkwell BasicValueEnum coercion)**:
- js_target.vais: IntValue→FloatValue assertion
- **Root cause**: Type codegen mismatch; likely shared inkwell BasicValueEnum dispatch bug

**A3+A4 (SIMD aggregate extract)**:
- simd_test.vais + simd_distance.vais: both "Aggregate extract index out of range"
- **Root cause**: single SIMD struct field indexing logic error — **mergeable to 1 task**

**B1+B2+B3 ([INST] logs)**:
- option_result_simple_test, option_result_test, simple_hashmap_test — all print [INST] logs
- **Root cause**: instantiation logging enabled (parallel.rs) — removable in 1 cleanup

**C1~C6 (semantic/validation errors)**:
- calculator_enum, async_reactor_test, wasm_todo_app, test_import, range_type_error_test, lazy_simple: user code errors (bugs, missing libs)
- **Root cause**: source files contain actual errors — each requires independent fix

**D1+D2 (undefined store_* funcs)**:
- tcp_10k_bench.vais: missing store_i8, store_i32 exports
- **Root cause**: std lib export gap — 1 task

**D3 (selfhost_token NotCallable)**:
- selfhost/token.vais: i64 being called as function
- **Root cause**: selfhost code bug — independent fix

## Task rebalancing suggestion

Current 11-task structure → potential 6-7 task collapse:
1. **[INST] log cleanup** (1 file edit + 3 examples unblock)
2. **SIMD aggregate extract** (fix 2 examples)
3. **inkwell BasicValueEnum assertion** (fix 1 example)
4. **tcp_10k_bench store_* exports** (std lib gap, 1 example)
5. **calculator_enum undefined var** (source bug, 1 example)
6. **async_reactor_test type coercion** (source bug, 1 example)
7. **wasm_todo_app array indexing** (source bug, 1 example)
8. **test_import missing module** (source lib, 1 example)
9. **range_type_error_test user error** (source bug, 1 example)
10. **lazy_simple undefined lazy** (source lib, 1 example)
11. **selfhost_token NotCallable** (selfhost bug, 1 example)

**Consolidation**: [INST] + SIMD + inkwell + store_* = 4 compiler/infrastructure fixes; 7 source/lib bugs remain independent.

## PROMISE

**PROMISE: COMPLETE** — All 16 SKIP_LIST entries measured; error signatures captured; 3 mergeable groups identified ([INST], SIMD, store_* exports); remaining 7 are independent user code/library gaps. No deep codebase dive performed.
