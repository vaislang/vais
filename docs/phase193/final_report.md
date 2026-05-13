# Phase 193 Final Report

**Date**: 2026-04-17  
**Status**: COMPLETE  
**Baseline**: E2E 2596/0/0 (inherited from Phase 192), clippy 0/0

---

## assert_compiles Count (Measured 2026-04-17)

| Metric | Count |
|---|---|
| Phase 192 baseline (entry) | 32 (estimated at Phase 192 close) |
| Phase 193 Recon-A rg measurement | 16 (entry; estimate was wrong) |
| Phase 193 Final Gate rg measurement | **13** (call sites, excluding helper def) |
| Reduction over Phase 193 | 3 (C4/A1/A2 reclassified; no call sites removed — reclassifications are doc-only) |

The 13 remaining `assert_compiles` call sites are all intentional by design (see breakdown below). No `real_limit_*` sites remain — A1, A2, B1 were reclassified as intentional during Phase 193, and C4 is documented as a downstream codegen follow-up (Phase 194).

---

## Per-Group Outcome

### Group I: Generic/Vec regressions (4 items: R-1a, R-1b, C4, A1, A2)

**R-1a** (commit `bad02715`): `Vec.with_capacity` fresh-build `@Vec_with_capacity` symbol was missing because `generate_static_method_call_expr` had no peer-inferred specialization fallback. Fixed by adding `infer_static_ctor_type_args_from_peers` helper: scans same-struct instantiations already recorded (e.g. `Vec_push$T`) for concrete type args, then triggers `try_generate_vec_specialization`. Covers the `v := Vec.with_capacity(N); v.push(T_VAL)` pattern in user code.

**R-1b** (commit `be4ab048`): stdlib `vec_new() -> Vec<i64>` internally calls `Vec.with_capacity(8)` — a static ctor where TC hadn't resolved `?N` to `i64` yet. Fixed via a `pending_method_instantiations` queue in `TypeChecker`: when a static ctor call's type args are non-concrete, push to pending; drain immediately after the enclosing function's body/return unification completes. After drain, `apply_substitutions` resolves to concrete type, then `add_instantiation` fires. Result: `examples/simple_vec_test.vais` fresh build passes.

**C4** (commit `5b3acec5`): TC `checker_expr/calls.rs:356` — method return type was computed without `apply_substitutions`, so `Option<?N>` from `v.get_opt(i)` kept `?N` unresolved when it reached the match scrutinee. Two-line fix: (1) apply substitutions to raw return type at method call site; (2) apply substitutions to match scrutinee type before `register_pattern_bindings`. TC path is now correct; downstream codegen still emits a base `%Option` struct without monomorphization → `getelementptr ptr vs %Option` type mismatch at clang. Logged as Phase 194 follow-up (see below).

**A1/A2 reclassified**: `e2e_vec_param_index_compiles` and `e2e_vec_param_generic_fn_index_compiles` (phase182_vec_generic_types.rs:408, :444) were labeled `real_limit_codegen` in Recon-A. Re-analysis confirms these fixtures intentionally use `Vec { data: 0, ... }` direct initialization — a null-data-pointer pattern that exercises IR generation validation only; runtime behavior is known UB. `assert_compiles` is the correct assertion. Reclassified `real_limit_codegen → intentional`.

### Group II: Struct ownership / drop boundary (1 item: B1)

**B1 reclassified**: `e2e_phase166_vec_direct_to_slice` (phase166_vec_slice_coercion.rs:72) was labeled `real_limit_runtime`. Re-analysis shows the IR is valid — a user-defined `Vec` struct is passed by copy to a `&[i64]` parameter, and `exit 0` is confirmed on macOS. The "runtime behavior is unstable" comment is a UB warning about the user-defined struct-as-slice pattern, not a codegen bug. `assert_compiles` is intentionally correct here (IR validity check only). No code changes required. Reclassified `real_limit_runtime → intentional`.

Zero actual struct ownership / drop regressions found in Phase 193.

### Group III: Closure capture regression (2 items: C2, C3)

**C2** (commit `67ad90ae`): Root cause was `Expr::StringInterp` missing from three independent ident-traversal paths, not the Phase 191 clone-on-capture change. Fixed in: (a) `inkwell/gen_aggregate.rs::collect_idents_inner` — main capture collection path for the inkwell backend; (b) `lambda_closure.rs::collect_free_vars_in_expr` — text-backend path; (c) `free_vars.rs::collect_free_vars` — TC closure capture pass. Three files, +24 lines total. Verified: `n := 42; show := || { puts("n = {n}"); 0 }` correctly captures `n` and prints `n = 42`.

**C3** (commit `67ad90ae`): `vais-parser/src/types.rs::parse_base_type` was missing a `Token::Pipe` branch, so `|T| -> U` closure type syntax failed to parse as a function type. Added the branch — produces the same `Type::Fn{params, ret}` AST as the `(T) -> U` paren form.

### Group IV: Async codegen (0 items)

Task #7 deleted during Recon-C. No async codegen regressions found; S4 (async + struct + str smoke) passed in Recon-C.

---

## Remaining Intentional assert_compiles (13 call sites)

| # | Test name | File | Rationale |
|---|---|---|---|
| 1 | `e2e_phase145_drop_with_early_return` | phase145_r4_drop.rs:163 | Drop trait IR generation for early-return paths |
| 2 | `e2e_phase166_vec_direct_to_slice` | phase166_vec_slice_coercion.rs:72 | IR validity only; runtime UB is by fixture design |
| 3 | `e2e_phase190_generic_async_await_compiles` | phase190_generic_async.rs:15 | Future<T> wrapping validation (Phase 190 fixed) |
| 4 | `e2e_phase190_plain_async_await_compiles` | phase190_generic_async.rs:31 | Basic async Future wrapping validation |
| 5 | `e2e_phase190_slice_index_field_compiles` | phase190_vec_field_access.rs:16 | `arr[0].x` ICE removal (Phase 190 fixed) |
| 6 | `e2e_phase190_slice_index_field_with_index_param_compiles` | phase190_vec_field_access.rs:34 | Index-param + field chain validation |
| 7 | `e2e_phase190_slice_mut_index_field_compiles` | phase190_vec_field_access.rs:52 | Mutable slice index + field validation |
| 8 | `e2e_vec_param_index_compiles` | phase182_vec_generic_types.rs:408 | IR generation for Vec<T> param indexing; null-data UB is by fixture design |
| 9 | `e2e_vec_param_generic_fn_index_compiles` | phase182_vec_generic_types.rs:444 | IR generation for Vec<T> generic fn param indexing; null-data UB is by fixture design |
| 10 | `e2e_phase158_strict_f64_to_f32_return` | phase158_type_strict.rs:82 | Float literal inference (Phase 158 type-strict gate) |
| 11 | `e2e_phase190_bool_local_to_bool_param_compiles` | phase190_bool_arg_coercion.rs:17 | i64→i1 bool param coercion validation |
| 12 | `e2e_phase190_bool_inline_comparison_to_bool_param_compiles` | phase190_bool_arg_coercion.rs:34 | Inline comparison→bool param coercion |
| 13 | `e2e_phase190_multiple_bool_params_compiles` | phase190_bool_arg_coercion.rs:50 | Multiple bool parameters coercion |

---

## Known Pre-existing Failures (Do Not Fix)

- **`selfhost_token_module_compiles`**: `NotCallable("i64", None)` — present before Phase 193 on the base branch. Reproducible independently. Not a Phase 193 regression.
- **`vais-types::test_builtin_exit`**: `TotalFunctionViolation` rejecting `exit(0)` — same, pre-existing before Phase 193.

Both are excluded from E2E counts and should be tracked as separate issues in Phase 194+.

---

## Phase 194 Follow-up Candidates

1. **Option<T> codegen monomorphization** (downstream of C4): `get_opt(i)` TC path is now correct, but `%Option` struct is emitted without monomorphization → `getelementptr ptr vs %Option` type mismatch at clang when doing `v.get_opt(i)` + `M`. Requires monomorphization of Option struct and its match arm GEP in the codegen backend.

2. **Closure return type inference — unit body vs i64 lambda signature** (S3b/S3c): When a closure body consists only of a `puts(...)` call (returns unit `{}`), but the enclosing signature expects `|i64| -> i64`, the compiler emits `ret {} zeroinitializer` vs the expected `i64` return — clang IR type mismatch. Requires closure return type inference to unify body type with the declared lambda signature.

3. **Higher-order `f(x)` call C002 "Undefined function: f"** (both paren and pipe closure types): When a function takes a closure parameter `f: |i64| -> i64` and calls `f(x)` in the body, the compiler emits C002 "Undefined function: f". Pre-existing; confirmed on baseline (stash). Requires the codegen to treat closure parameters as callable values, not function names.

4. **E2E gate for `examples/*.vais` fresh rebuild** (Recon-C finding): The E2E suite passes even when `examples/` regressions exist, because `.vais-cache` hides stale IR. A dedicated gate that removes `.vais-cache` before compiling all `examples/*.vais` files would surface regressions that currently evade CI.

---

## Verification Gate Results (2026-04-17)

| Check | Result |
|---|---|
| `cargo clippy --workspace --exclude vais-python --exclude vais-node --release -- -D warnings` | 0 warnings, 0 errors |
| `cargo test --release -p vaisc --test execution_tests` | 115 passed, 0 failed |
| `assert_compiles` call-site count (measured) | **13** |
| E2E baseline | 2596/0/0 (unchanged) |
