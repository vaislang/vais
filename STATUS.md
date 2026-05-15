# Vais Compiler Status

> Last updated: 2026-04-25 (Phase 0.C stdlib self-test green)

## Conformance Test Results

`tests/lang/` — language feature conformance suite.

| Category | Tests | Passing | Status |
|----------|-------|---------|--------|
| 01_primitives | 47 | 47/47 | ✅ |
| 02_control_flow | 41 | 41/41 | ✅ |
| 03_match | 27 | 27/27 | ✅ |
| 04_struct | 33 | 33/33 | ✅ |
| 05_enum | 27 | 27/27 | ✅ |
| 06_generic | 28 | 28/28 | ✅ |
| 07_collections | 22 | 22/22 | ✅ |
| 08_strings | 11 | 11/11 | ✅ |
| 09_traits | 4 | 4/4 | ✅ |
| 10_ffi | 3 | 3/3 | ✅ |
| 99_integration | 68 | 68/68 | ✅ |
| **Total** | **311** | **311/311 (100%) — ALL GREEN, ZERO XFAIL** | 🎉 |

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

## Stdlib Self-Tests — `compiler/std/tests/` (7/7) — ZERO XFAIL

| Test | Module | Assertions | Status |
|------|--------|-----------:|--------|
| `test_vec.vais` | std/vec | 10 | ✅ |
| `test_option.vais` | builtin Option<T> + std/option methods | 9 | ✅ |
| `test_result.vais` | builtin Result<T,E> | 5 | ✅ |
| `test_bytebuffer.vais` | std/bytebuffer | 21 | ✅ |
| `test_string.vais` | std/string + helpers | 28 | ✅ |
| `test_hashmap.vais` | std/hashmap StrHashMap | 16 | ✅ |
| `test_sync.vais` | std/sync (AtomicI64/Bool, Mutex, MutexGuard) | 17 | ✅ |

Run yourself:
```bash
cd compiler/std/tests && bash run.sh
```

## Stability Tiers

### Core (production-track)
- `vais-lexer`, `vais-parser`, `vais-ast`, `vais-types`, `vais-codegen`, `vais-mir`, `vaisc`
- These crates produce LLVM IR. Trust target: 100% conformance suite green.

#### Two codegen backends — both green as of 2026-04-26
The `vais-codegen` crate currently houses two parallel lowering paths:
- **Inkwell** (default for `vaisc build`) — what the lang conformance suite,
  stdlib self-tests, hello world examples, and bootstrap_tests all exercise.
  All four green: lang 311/311, stdlib 7/7, hello 12/12, bootstrap 17/17.
- **Text-IR** (used by `crates/vaisc/tests/e2e/`) — older string-based
  emit path. **2625 / 2625 e2e tests pass, 0 failed** (down from 176
  fails on the same suite — full -100%). The text-IR backend will
  still be retired in favour of inkwell long-term, but it is no
  longer a known-broken path.

  Cleanup that landed across the run:
  - `function_gen/runtime.rs::generate_wasm_runtime` no longer
    redeclares `llvm.memcpy.p0i8.p0i8.i64`.
  - `string_ops.rs` no longer redeclares `strstr`.
  - `emit.rs` skips its `abort` / `llvm.memcpy` declares when the
    standard runtime helpers already emit them.
  - `type_inference.rs::infer_expr_type Expr::Index` returns
    `Slice(elem)` (was `Pointer(elem)`) when index is a Range, so the
    text-IR slice-element index path takes the `is_fat_ptr=true`
    branch and emits `extractvalue` + GEP — fixed the slice ABI gap
    that crashed `phase109_bounds_check::slice_*` and the
    json2toml/REST API pilots (5 + 3 = 8 tests).
  - `expr_helpers.rs::generate_cast_expr` now emits `zext` (not
    `sext`) when widening `i1` → wider int — text-IR companion to the
    inkwell C8 fix. Eliminates the `bool as i64 → 255` regression in
    `phase158_type_strict`.
  - 7 runtime helper definitions (`__malloc`, `__free`, `__memcpy`,
    `__strlen`, `__store_ptr`, `__str_eq`, `__call_fn`) now emit as
    `linkonce_odr` so they coexist with the same symbols defined in
    the e2e helper's `.c` runtime sidecars (`http_runtime.c`,
    `thread_runtime.c`). Resolved 10 duplicate-symbol link failures
    in `async_runtime::e2e_http_*` and `concurrency::e2e_*`.
  - 130-binding `:= mut` patch across 15 e2e test files +
    32-binding patch in `examples/pilot_*.vais` to comply with C2
    strict-immutable enforcement (eliminated 80+ ImmutableAssign
    failures that were previously masked by silent miscompile).

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
| C4 ✅ | ~~Specialized generic body's call to another generic's static method~~ FIXED via two changes: (1) `expr_helpers_call/method_call.rs` substitute `expected_ret` using current spec context, and prefer expected_ret's generics over arg-type inference when struct has generics + spec context active. `MutexGuard::new(&self)` inside `Mutex_lock$i64` now mangles to `MutexGuard_new$i64`. (2) C9 layout fix below unblocks the remaining link mismatch. Promoted `xfail_sync_mutexguard_specialization.vais` → `test_sync.vais` covering AtomicI64/Bool, Mutex<i64>, MutexGuard<i64> end-to-end. | — | — | test_sync.vais |
| C9 ✅ | ~~`?` postfix type fell to ResolvedType::Unknown → i64 in signature~~ AND ~~`Some(<user_struct>)` constructor uses global `%Option={i32,{i64}}` layout, mismatching declared `{i8,%T}` return slot~~ — both FIXED. Part 1: `types/conversion.rs::ast_type_to_resolved_impl`: `Type::Optional` → `ResolvedType::Optional`, `Type::Result` → `ResolvedType::Result(_, i64)`. Part 2: `function_gen/codegen.rs` (3 ret paths — Expr/Block in `generate_function`, Block in `generate_method_with_span`) detects universal `%Option`/`%Result` body value flowing into a specialized `{ i8, %T }` ret slot and emits alloca + store + bitcast + load to reinterpret the layout. The two layouts share an i64-word payload, so the reinterpret is sound for ≤8B inline payloads (covers user-struct cases that fit; >8B paths heap-allocate via existing ptrtoint logic). | — | — | test_sync.vais |
| C18 ✅ | ~~match arm `On =>` (unit variant ident) was alloca'd as a fresh local in codegen~~ FIXED in `inkwell/gen_match.rs::generate_pattern_bindings`: Pattern::Ident now skips binding if the name is a known unit-variant (matches TC's logic). Test: `tests/lang/05_enum/enum_two_units.vais` exercises `flip(t) -> Toggle` with enum constructors as match arm bodies. | — | — | enum_two_units.vais |
| C10 ✅ | ~~match arm guards not recognized~~ NOT A BUG — vais uses single-char `I` keyword for guards (`pattern I expr => body`); spelled-out `if` is just an ident. Test was using wrong syntax. Updated `tests/lang/03_match/match_guard.vais` to use `I` and the test passes. | — | — | match_guard.vais |
| C11 ✅ | ~~trait default methods not dispatched on impls that don't override~~ FIXED via two changes: (1) `checker_module/traits.rs::register_impl` now adds synthetic `FunctionSig` for trait defaults to the struct/enum methods table when an impl doesn't override (so TC resolves the call); (2) `inkwell/generator.rs` synthesizes the AST `Function` for unoverridden trait defaults in both the declare-pass and the emit-pass, so the per-impl method actually gets lowered to IR (was "Undefined function: S_a" link error). Test: `tests/lang/09_traits/trait_default.vais` exercises both override and inherited-default forms. | — | — | trait_default.vais |
| C12 ✅ | ~~`X F builtin_name(...)` re-declares an already-builtin function, emitting double `declare` + suffixed `@name.1` call~~ FIXED in `inkwell/gen_declaration.rs::declare_extern_block`: check `module.get_function(name)` first and reuse the existing declaration if present, instead of `add_function`-ing a duplicate (LLVM auto-renamed the duplicate to `name.1`, breaking link). Test: `tests/lang/10_ffi/ffi_extern_redecl.vais` redeclares `sqrt` and `cos` and asserts they call the runtime libm symbols. | — | — | ffi_extern_redecl.vais |
| C13 ✅ | ~~storing an i64 literal into a u32/i32/u8 alloca writes 8 bytes against a 4-byte slot, clobbering adjacent allocas~~ FIXED in `inkwell/gen_stmt.rs`: when the binding has a narrower-than-i64 type and the RHS is wider, truncate (or zext when widening) before `build_store`. | — | — | int_unsigned.vais |
| C14 ✅ | ~~bare `None` literal triggered an extra `$unknown` specialization with Unit-erased T~~ FIXED in `module_gen/instantiations.rs`: skip Function instantiations whose type_args contain Unknown/Generic/Var (the well-typed siblings already cover the real call sites). Same skip already existed for Method instantiations; this brings function path into parity. | — | — | generic_unwrap.vais |
| C15 ✅ | ~~array literal `[1, 2, 3]` is typed `[i64]` (slice), not `[i64; 3]` (fixed-size array)~~ FIXED via two coordinated changes: (1) `inference/inference_modes.rs::check_array_with_expected` — when expected is `ConstArray { size }` and the literal length matches, type the literal as `ConstArray` (not slice). (2) `inkwell/gen_stmt.rs::Stmt::Let` — when the annotated `var_type` is an array and the RHS produced a pointer (`Expr::Array` returns `[N x T]*`), load through the pointer first then store-by-value. Without (2), the binding would write 8 bytes (the pointer) into the N*sizeof(T)-byte alloca, leaving the rest as garbage. Test: `tests/lang/01_primitives/array_fixed_size.vais`. | — | — | array_fixed_size.vais |
| C16 ✅ | ~~`Vec<EnumType>` from a generic constructor fails~~ FIXED with three coordinated changes: (1) `inference_modes.rs::check_expr_bidirectional` Expr::Call branch — register the generic instantiation when the call's expected type drives T and propagate transitively. (2) `checker_expr/calls.rs::check_static_method_call` records static method calls as method-flavored `generic_callees`, and `propagate_transitive_instantiations` registers Method-kind instantiations for them. (3) `checker_fn.rs::check_function` pushes the declared return type onto `expected_type_stack` so tail-position generic calls bind T → Generic("T"). Also added `vec_new_t<T>()` to std/vec.vais. Test: `tests/lang/05_enum/enum_in_vec.vais`. | — | — | enum_in_vec.vais |
| C17 ✅ | ~~cascade generic specialization left unspecialized refs~~ FIXED in `inkwell/gen_expr/call.rs` (rewrite call site to mangled spec when in specialized body) AND `inkwell/generator.rs` (pre-declare ALL function specs in pass 1 so any body in pass 2 can find any other spec by name regardless of iteration order). | — | — | generic_chain.vais, generic_chain_two.vais, generic_helper_pyramid.vais |
| C5 ✅ | ~~`String.with_capacity(n)` segfaults when `n < 16`~~ FIXED in std/string.vais: `new_cap := self.cap * 2` is now `:= mut`. Root cause: codegen alloca'd `new_cap` but skipped the initial store; only the `< 16` branch wrote to it, leaving the else branch reading uninitialized memory → `malloc(garbage)` crash. Underlying codegen bug remains (separate finding: alloca without initial store when binding is later reassigned). | — | — | test_string.vais cap=4 |
| C6 ✅ | ~~struct field of fixed-size array `[T; N]` triggers ICE on read~~ FIXED in `inkwell/gen_aggregate.rs::generate_index` (handle ArrayValue by spilling to alloca + GEP) AND `inkwell/gen_advanced.rs::generate_struct_literal` (load array through pointer when field type is array). Two distinct codegen paths needed updating because `generate_array` returns the alloca pointer, not the array value. Test: `tests/lang/04_struct/struct_array_field.vais`. | — | — | struct_array_field.vais |
| C7 ✅ | ~~match arms with nested variant patterns produce invalid phi/wrong arm dispatch~~ FIXED in `inkwell/gen_match.rs` — two-part fix: (1) `push_inner_scrutinee_for_variant` threads the inner Option/Result type onto the scrutinee stack while recursing into nested patterns so payload-decoding lookups resolve correctly; (2) `Pattern::Variant` pattern-check now AND-s the inner pattern's check when any field is itself a Variant/Literal/Range, so sibling arms like `Some(Some(v))` and `Some(None)` no longer collapse to the same outer-tag check. Test: `tests/lang/03_match/match_nested_enum.vais`. | — | — | match_nested_enum.vais |
| C8 ✅ | ~~`bool as i64` returns 255~~ FIXED in `inkwell/gen_expr/misc.rs`: when widening i1 → wider int, use `zext` (zero-extend) instead of `sext`. Sign-extending i1 1 produces all-1s; zext gives 1 as expected. Regression test: `tests/lang/01_primitives/bool_short_circuit.vais` (11 assertions exercise `bool as i64`). | — | — | bool_short_circuit.vais |
| C19 ✅ | ~~`&self.x as i64` parses as `&((self.x) as i64)` instead of `(&self.x) as i64`~~ FIXED in `vais-parser/src/expr/unary.rs`: after `parse_unary` returns for the inner of `&`, if the inner is `Cast { expr, ty }`, re-shape the AST to `Cast { expr: Ref(expr), ty }` so the cast wraps the reference. This matches Rust precedence (unary `&` binds tighter than `as`) without changing `parse_postfix`. Reverted std/sync.vais's `(&self.value) as i64` paren-wrap to natural `&self.value as i64`. Test: `tests/lang/01_primitives/ref_field_as_cast.vais`. | — | — | ref_field_as_cast.vais |
| C20 (parser) | `parse_unary`/`parse_postfix` recursive descent burns ~400KB+ of stack per `(` level; 20 levels of nesting overflow the default 8MB `cargo test` thread stack. Function correctness is fine (the lang suite passes complex programs daily) — this is a pure depth-bound robustness gap, distinct from the `MAX_PARSE_DEPTH` runtime check (which only fires *after* the Rust stack already blew up). Test `test_error_deeply_nested_expressions` is `#[ignore]`d until the parser is rewritten with explicit-stack handling for the `(...)` and unary-prefix paths. | avoid >15 levels of literal `(...)` nesting | — | n/a (negative test, not a real-world failure) |

### Phase 17 Wave 1-4a discovered bugs (prior sessions)

These bugs were fixed in commits `7c3aed52`, `72616dc2`, `039df2f7`,
`32d1ed83`, `5a11bcf0` during Phase 17 Wave 4a probe and Phase 0
kickoff. Each has a corresponding `tests/lang/` regression test (or
should — see TODO):

| Bug | Test | Status |
|-----|------|--------|
| match default arm null literal | `tests/lang/03_match/match_phi_default_zero.vais` | ✅ landed |
| `vec[i] = struct_value` ptr store | `tests/lang/07_collections/vec_struct_assign.vais` | ✅ landed |
| 4-byte Named struct narrow store | `tests/lang/04_struct/struct_4_bytes_in_vec.vais` | ✅ landed |
| match arm phi narrow-int width | `tests/lang/03_match/match_phi_narrow_int.vais` | ✅ landed |
| Specialized enum match (`%Unknown`) | `tests/lang/05_enum/result_specialized_match.vais` | ✅ landed |
| Enum payload of >8B struct | `tests/lang/05_enum/enum_struct_payload.vais` | ✅ landed |
| Vec→slice via explicit `&v[0..v.len()]` | `tests/lang/07_collections/vec_to_slice.vais` | ✅ landed (implicit coercion `f(v)` without `&` remains a separate enhancement) |
| `slice.to_vec()` builtin | (stdlib gap, not lang feature) | n/a |

## Active Work

- **Phase 0.A**: surface area audit + workspace pruning ✅ — default-members reduced from 28 to 13 (7 core + 6 auxiliary). Experimental crates remain in `members` so `cargo build -p <name>` still works, but `cargo build` / `cargo test` no longer compile them by default. CRATE_AUDIT.md is the source of truth for tier definitions.
- **Phase 0.B**: conformance suite — 54 tests landed, target 300+
- **Phase 0.C**: stdlib self-tests — 7/7 ✅ (zero XFAIL)
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
