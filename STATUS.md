# Vais Compiler Status

> Last updated: 2026-04-25 (Phase 0.C stdlib self-test green)

## Conformance Test Results

`tests/lang/` — language feature conformance suite.

| Category | Tests | Passing | Status |
|----------|-------|---------|--------|
| 01_primitives | 14 | 14/14 | ✅ |
| 02_control_flow | 14 | 14/14 | ✅ |
| 03_match | 7 | 7/7 | ✅ |
| 04_struct | 9 | 9/9 | ✅ |
| 05_enum | 6 | 6/6 | ✅ |
| 06_generic | 7 | 7/7 | ✅ |
| 07_collections | 3 | 3/3 | ✅ |
| 08_strings | 2 | 2/2 | ✅ |
| 09_traits | 0 | — | not yet |
| 10_ffi | 0 | — | not yet |
| 99_integration | 9 | 9/9 | ✅ |
| **Total** | **73** | **73/73 (100%)** | 🎉 |

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
| `test_hashmap.vais` | std/hashmap StrHashMap | 16 | ✅ |
| `xfail_sync_mutexguard_specialization.vais` | std/sync Mutex | — | ⚠️ XFAIL (compiler bug C4) |

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
| C1 ✅ | ~~`B <value>` (break-with-value) emits invalid IR~~ FIXED in inkwell/gen_stmt.rs::generate_break: hoist the `loop_break_value` alloca into the function entry block before recording it on the loop ctx, so it dominates every break site (which may be inside `I`/`M` arms) AND the loop-end load. Also added a parallel fix to the IR-string fallback in generate_expr/loops.rs (pre-scan body for `B <value>`, alloca + zero-init before loop entry, load at loop end). Regression test: `tests/lang/02_control_flow/break_with_value.vais`. | — | — | break_with_value.vais |
| C2 ✅ | ~~`:= <int>` immutable bindings reassigned via `=` silently miscompile~~ FIXED in checker_expr/special.rs — TC now emits `ImmutableAssign` (E009) on Expr::Assign / Expr::AssignOp when target is a non-mut Ident binding. Regression test: `tests/lang/02_control_flow/mut_reassign.vais`. | — | — | mut_reassign.vais |
| C3 ✅ | ~~StrHashMap<i64> generic specialization duplicate symbol cross-module~~ FIXED in function_gen/generics.rs by emitting specialized definitions with `linkonce_odr` linkage. Standard C++-template-instantiation discipline — equivalent monomorphizations from multiple consumer modules merge at link time. Promoted xfail_hashmap_strhashmap.vais → test_hashmap.vais. | — | — | test_hashmap.vais |
| C4 partial | Specialized generic body's call to another generic's static method now correctly resolves to the inner specialization. `MutexGuard::new(&self)` inside `Mutex_lock$i64` mangles to `MutexGuard_new$i64` (was bare `MutexGuard_new`). Two changes in `expr_helpers_call/method_call.rs`: (1) substitute `expected_ret` using current spec context; (2) when struct has generics + spec context active + concrete expected_ret, prefer expected_ret's generics over arg-type inference. STILL XFAIL — std/sync exercises a deeper unrelated bug C9. | — | — | xfail_sync_mutexguard_specialization.vais |
| C9 partial | ~~`?` postfix type fell to ResolvedType::Unknown → i64 in signature~~ FIXED in `types/conversion.rs::ast_type_to_resolved_impl`: `Type::Optional` → `ResolvedType::Optional`, `Type::Result` → `ResolvedType::Result(_, i64)`. Function signatures now match their declared return type. STILL latent: when the function body returns `Some(<user_struct>)`, the `Some(...)` constructor uses the global `%Option = {i32, {i64}}` layout instead of `{i8, %Struct}`. That deeper "specialized Optional layout per call site" issue is what still blocks C4. | use named `Option<T>` instead of `T?` (which now also works) | — | partial in conversion.rs |
| C5 ✅ | ~~`String.with_capacity(n)` segfaults when `n < 16`~~ FIXED in std/string.vais: `new_cap := self.cap * 2` is now `:= mut`. Root cause: codegen alloca'd `new_cap` but skipped the initial store; only the `< 16` branch wrote to it, leaving the else branch reading uninitialized memory → `malloc(garbage)` crash. Underlying codegen bug remains (separate finding: alloca without initial store when binding is later reassigned). | — | — | test_string.vais cap=4 |
| C6 ✅ | ~~struct field of fixed-size array `[T; N]` triggers ICE on read~~ FIXED in `inkwell/gen_aggregate.rs::generate_index` (handle ArrayValue by spilling to alloca + GEP) AND `inkwell/gen_advanced.rs::generate_struct_literal` (load array through pointer when field type is array). Two distinct codegen paths needed updating because `generate_array` returns the alloca pointer, not the array value. Test: `tests/lang/04_struct/struct_array_field.vais`. | — | — | struct_array_field.vais |
| C7 ✅ | ~~match arms with nested variant patterns produce invalid phi/wrong arm dispatch~~ FIXED in `inkwell/gen_match.rs` — two-part fix: (1) `push_inner_scrutinee_for_variant` threads the inner Option/Result type onto the scrutinee stack while recursing into nested patterns so payload-decoding lookups resolve correctly; (2) `Pattern::Variant` pattern-check now AND-s the inner pattern's check when any field is itself a Variant/Literal/Range, so sibling arms like `Some(Some(v))` and `Some(None)` no longer collapse to the same outer-tag check. Test: `tests/lang/03_match/match_nested_enum.vais`. | — | — | match_nested_enum.vais |
| C8 ✅ | ~~`bool as i64` returns 255~~ FIXED in `inkwell/gen_expr/misc.rs`: when widening i1 → wider int, use `zext` (zero-extend) instead of `sext`. Sign-extending i1 1 produces all-1s; zext gives 1 as expected. Regression test: `tests/lang/01_primitives/bool_short_circuit.vais` (11 assertions exercise `bool as i64`). | — | — | bool_short_circuit.vais |

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
