# LLVM Ground-Truth Refactor — Wave 4 Design

> **Status**: design landed iter 57 (2026-04-25)
> **Scope**: Catch-all `default → "i64"` fallback removal in `generate_expr/mod.rs:298`
> **Predecessor**: Wave 1 (primitives, 99 sites), Wave 2 (composite alloca/gep/load/call, 117 sites), Wave 3 (phi/extract/insert/bitcast, 62 sites). Cumulative: ~278 sites migrated.

## 1. Status After Waves 1–3

### 1.1 Migrated coverage

| Wave | Category | Sites |
|------|----------|-------|
| 1 | ptrtoint, trunc/sext/zext, icmp/fcmp, algebraic | 99 |
| 2a | alloca | 9 (9 deferred) |
| 2b | getelementptr | 17 (5+ deferred) |
| 2c.1 | load (wide: i64/ptr/double/float/fat) | 40 |
| 2c.2 | load (narrow: tag i32/i16/i8) | 0 new (Wave 1 covered) |
| 2d | call (malloc/strlen/user fn/memcpy/__vais_str/SIMD) | 25 |
| 3 | phi/extractvalue/insertvalue/bitcast | 62 |
| **Total** | | **~252 + 26 phi-record duplicates = ~278** |

### 1.2 Deferred site classes

Three structural classes remain unmigrated due to cascade or signature constraints:

**Class A: i32 narrow-return cascade** (~8 sites)
- `pattern.rs` tag_val i32 (3): `load i32, i32* tag_ptr` → consumer expects i32 vs catch-all i64
- `print_format.rs` snprintf len_i32 (1): `call i32 @snprintf(...)` → sext-then-mix consumers
- `string_ops.rs` strcmp result (5): `call i32 @strcmp(...)` → icmp eq i32 result, 0
- **Reason**: Recording i32 ground-truth conflicts with catch-all i64 fallback used by chained consumers. Removing catch-all (this Wave) eliminates conflict.

**Class B: data-chain typed_ptr cascade** (~6 sites)
- `expr_helpers_data.rs` (4): array/tuple/struct/union literal alloca + bitcast i8* to {T}*
- `expr_helpers_misc.rs` (2): try/unwrap payload_ptr gep → bitcast cast
- **Reason**: Downstream field access / index operations re-derive type via `llvm_type_of` and rely on catch-all i64. Recording the actual struct ptr cascades.

**Class C: `&self` signatures** (~7 sites)
- `helpers.rs:329` (`&self` malloc helper)
- `vtable.rs` 5 sites (`create_trait_object` &self, vtable-slot bitcast)
- **Reason**: `&self` callers cannot mutate `self.fn_ctx`. Either change signature to `&mut self` (preferred) or introduce a separate scoped FunctionContext (Q2 helper-IR plan).

**Class D: helper-IR (raw push_str)** (~140 sites total, deferred to Wave 4 helper-IR FunctionContext)
- `function_gen/runtime.rs` (87)
- `string_ops.rs` `__vais_struct_shallow_free_*` emitted helpers (40)
- `function_gen/async_gen.rs` (6)
- Other (5)
- **Reason**: These emit standalone LLVM functions inside the caller's IR string. SSA names are scoped to the emitted function, not the parent `FunctionContext`. Q2 decision: include in strict 100% coverage by introducing per-helper FunctionContext.

### 1.3 Gate baseline

- cargo test: `vais-codegen` 796/796, `vais-types` 355/355 — held at every commit.
- vaisdb 4-run / 8-run avg link errors: baseline ~21.75 (post-Wave 1c.5), maintained or improved per batch (range 14.4–22 across Waves 2–3).
- linked count: 0/15 held throughout. (Note: pre-Wave-1 baseline was 1/15; one test regressed during Wave 1c iteration but link errors continually decreased.)
- Cargo CI: variance ~ ±15 errors per run due to vaisdb cross-module non-determinism documented in `phase16_cascade_pattern` memory.

## 2. Wave 4 Goals

**Primary**: Remove `generate_expr/mod.rs:298` catch-all `default → "i64"`.

**Secondary** (enabled by primary):
1. Class A i32 narrow-return migrations (cascade resolves once catch-all is gone).
2. Class B data-chain migrations (consumer rederivation now sees ground-truth).
3. Class C `&self` signature change to `&mut self` for affected helpers.
4. Class D helper-IR FunctionContext infra (Q2 strict 100% coverage decision).

**Tertiary** (post-Wave 4 cleanup):
- Audit `temp_var_types` (ResolvedType registry) — if `actual_llvm_type` covers 100% of consumers, the legacy track can be retired. Per Q4 decision: target strict 100%, not 95%.

## 3. Implementation Plan

### 3.1 Sub-Wave 4a: catch-all replacement

`generate_expr/mod.rs:298` currently:
```rust
fn llvm_type_of_checked(&self, name: &str) -> Option<String> {
    // 1. Try actual_llvm_type (ground-truth track)
    if let Some(ty) = self.fn_ctx.actual_llvm_type.get(name) {
        return Some(ty.clone());
    }
    // 2. Fallback to ResolvedType track
    if let Some(rt) = self.fn_ctx.temp_var_types.get(name) {
        return Some(self.type_to_llvm(rt));
    }
    None
}

fn llvm_type_of(&self, name: &str) -> String {
    self.llvm_type_of_checked(name).unwrap_or_else(|| "i64".to_string())  // ← catch-all
}
```

**Target state**:
```rust
fn llvm_type_of(&self, name: &str) -> String {
    self.llvm_type_of_checked(name).unwrap_or_else(|| {
        debug_assert!(false, "no ground-truth for SSA value '{}'", name);
        "i64".to_string()  // graceful degradation in release
    })
}
```

**Pre-conditions**:
- Class A/B/C/D resolved (at least the most-impacted subset).
- `actual_llvm_type` coverage measured to ≥95% across `vaisdb` IR generation.

**Risk**: Any uncovered SSA value will trip `debug_assert!`, surfacing missed registrations. Per phase16 cascade memory, vaisdb has known flake — first run will reveal a long-tail of missed registrations. Iteration plan: panic-driven coverage hunt, register one-by-one until tests pass.

**Rollback**: Single-line change. Trivial revert.

### 3.2 Sub-Wave 4b: Class A (i32 narrow)

After 4a, Class A consumers get correct i32 from `llvm_type_of` instead of i64 catch-all. Re-attempt 8 sites in single batch. Expected net-zero error change (consumers were already i32-aware where they mattered).

### 3.3 Sub-Wave 4c: Class B (data-chain)

After 4a, `expr_helpers_data` typed_ptr bitcasts no longer cascade because `llvm_type_of(typed_ptr)` returns `{T}*` not `i64`. Re-attempt 6 sites. Possible secondary cascades in deeper field-access; bisect per file.

### 3.4 Sub-Wave 4d: Class C (`&self`)

Change signatures `&self → &mut self` for:
- `helpers.rs::generate_<fn>(&self, ..., temp_counter: &mut usize)` — vtable allocation helper
- `vtable.rs::create_trait_object(&self, ...)` — change to `&mut self`

Each caller already holds `&mut self` (codegen), so propagation is mechanical.

### 3.5 Sub-Wave 4e: Class D (helper-IR FunctionContext)

Introduce `HelperFunctionContext` mirroring `FunctionContext` but scoped to a single emitted helper function. Each `runtime.rs` / `string_ops.rs` helper-emit site:

```rust
// Before
ir.push_str("  %tmp = load i64, i64* %p\n");

// After
let mut helper_ctx = HelperFunctionContext::new();
helper_ctx.emit_load(&mut ir, "%tmp", "i64", "%p");
helper_ctx.record("%tmp", "i64");
```

Or, alternatively, defer Class D to a Wave 5 cleanup if Wave 4a-d already achieve catch-all removal under the current coverage. Per Q4 strict 100%, Class D must be in Wave 4 final.

### 3.6 Verification gates per sub-wave

Same protocol as Waves 1-3:
1. cargo test -p vais-codegen --lib: 796/796
2. cargo test -p vais-types --lib: 355/355
3. vaisdb 8-run codegen + link gate: baseline ~21.75, pass if avg within ±5 noise
4. Per-sub-wave commit, ROADMAP iter increment

### 3.7 Rollback policy

- Sub-wave 4a (catch-all removal): trivial revert (1 line). Top priority safety.
- Sub-waves 4b-e: per-file bisect → revert offending file → defer to next iter.
- Catastrophic failure: revert entire Wave 4 to last green = Wave 3 final commit `f3da3db6`.

## 4. Coverage Metric

Define **ground-truth coverage** as:
```
coverage = |{SSA values with actual_llvm_type entry}| / |{all SSA values emitted}|
```

Measurement: instrument `next_temp` to count emissions, count `actual_llvm_type` keys at end of function. Aggregate across all `vaisdb` test IR.

**Wave 3 end state** (estimate, not measured): ~80-85% (252 migrate sites + extensive `register_temp_type` from Wave 1c sites).

**Wave 4a target**: ≥95% (Class A/B/C resolved or catch-all-tolerant).
**Wave 4 final target**: 100% (Class D helper-IR included).

## 5. Open Questions

1. **Coverage instrumentation**: do we add a measurement pass (one-time `--measure-coverage` CLI flag) or rely on debug_assert! panic-driven coverage?
   - **Tentative**: panic-driven first (cheap), measurement pass if needed.
2. **Class D scope creep**: helper-IR has 140 sites, much larger than primary catch-all goal. Defer to Wave 5?
   - **Per Q4 (iter 34)**: strict 100%, no Wave 5 deferral. But timing-wise, Wave 4a-d alone gates major safety, and Class D could be incremental.
3. **vtable.rs `&self` change**: is there a hidden interior mutability that justifies keeping `&self`? Audit first.
4. **`temp_var_types` deprecation**: Wave 4 final removes catch-all but keeps legacy ResolvedType track. Is there a follow-up Wave to retire it, or is it permanently dual-track?

## 6. Exit Criteria

- `generate_expr/mod.rs:298` catch-all replaced by `debug_assert!`.
- All Class A + B + C deferred sites resolved.
- Class D helper-IR FunctionContext infra landed (or explicit Wave 5 deferral with rationale).
- cargo 796 + 355 maintained.
- vaisdb gate within ±5 of Wave 3 final baseline.
- No `debug_assert!` panics on `vaisdb` test IR generation.

## 7. References

- iter 25 design doc: `llvm-ground-truth.md` (initial 5-Wave plan)
- iter 33 Wave 2 design doc: `llvm-ground-truth-wave2.md`
- ROADMAP `iter_34_strategy` Open Questions Q1-Q5 decisions (safety-first defaults)
- phase16 cascade memory: vaisdb non-determinism explanation
