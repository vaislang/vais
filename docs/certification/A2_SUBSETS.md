# A2 Certified Subsets

Per Master Plan v16 Order Step 9. This file records each A2 surface
that has been formally promoted into the certified-subset set:
the **formal predicate** that defines the safe subset, the
**positive fixture** that demonstrates the subset compiles and runs
correctly, and the **negative fixture** that demonstrates uses
*outside* the predicate are rejected (or at least not accepted by
mistake).

A surface stays in the candidate list (master-plan §A2) until both
fixtures land here AND `bash compiler/scripts/check-integrity.sh`
shows INTEGRITY OK with no baseline regression.

## Status summary

| ID | Surface | Status |
|---|---|---|
| A2-01 | `?` operator on Result/Option (Core-typed, single-module) | LANDED 2026-05-03 (A2-NEG-DRIFT RESOLVED 2026-05-04 via A4-11 fix) |
| A2-02 | `?` operator cross-module (vaisdb baseline path) | LANDED 2026-05-03 (cross-module `?` baseline = vaisdb sql/types.vais:339 → storage/bytes.vais:40) |
| A2-03 | `dyn` / trait object dispatch (narrow subset) | LANDED 2026-05-05 (multi-impl dispatch via vtable both backends; master-plan v22) |
| A2-04 | Closures (no escape, inline-only) | LANDED 2026-05-04 (positive inline closure exit 42; negative escape closure was F-18 silent corruption, hard-blocked at TC layer 2026-05-08 = A4-15 v39) |
| A2-05 | Function-pointer types in std API (bounded) | LANDED 2026-05-04 (multi-impl fn-pointer dispatch exit 50; bare `fn(...)` parameter syntax accepted by parser) |
| A2-06 | (was: affine ownership annotation) | RECLASSIFIED to Controlled at master-plan v36 (Linear/Affine wrapper erasure entry, unification.rs:512-518 — type-carrier only, no enforcement; not a formal A2 predicate) |

All 5 named A2 candidates LANDED. A2-06 reclassified to Controlled
because the current `affine` annotation has no use-count enforcement
(wrapper-erasure only at vais-types/inference/unification.rs:512-518)
— without enforcement no negative predicate can be defined, so it does
not fit the A2 shape. If Step 16 (I-5 memory protocol) later adds
enforcement, affine can graduate Controlled → A2 at that time.

---

## A2-01 — `?` operator on Result/Option (Core-typed, single-module)

### Formal predicate

A use of the `?` operator is in the certified A2-01 subset iff ALL of
the following hold:

1. The receiver expression has type `Result<T, E>` or `Option<T>` where
   `T` and `E` are Core-typed (no generics, no lifetimes, no impl
   Trait).
2. The enclosing function's return type is `Result<U, E>` (for the
   `Result` case) or `Option<U>` (for the `Option` case) where `E`
   matches the receiver's `E`.
3. The use is within a single source file (no cross-module
   propagation). Cross-module uses are A2-02 (deferred).

When all three hold the `?` lowers to a standard early-return:
- `Ok(v)?  → v`, continuation
- `Err(e)? → return Err(e)`
- `Some(v)? → v`, continuation
- `None?    → return None`

### Positive fixture

`compiler/tests/empirical/A2/A2-01_q_operator_core/probe_pos.vais`:

```vais
F inner() -> Result<i64, str> {
    R Ok(42)
}
F outer() -> Result<i64, str> {
    x: i64 = inner()?
    R Ok(x + 1)
}
F main() -> i64 {
    res := outer()
    M res {
        Ok(v) => v,
        Err(_) => -1,
    }
}
```

Expected: `vaisc check` exits 0; binary runs; exit code = 43.

### Negative fixture

`compiler/tests/empirical/A2/A2-01_q_operator_core/probe_neg.vais`:

The negative fixture demonstrates a use OUTSIDE the predicate — the
enclosing function's return type does not match. Type checker should
reject (or at minimum not accept silently).

```vais
F inner() -> Result<i64, str> {
    R Ok(42)
}
F outer() -> i64 {                    # NOT Result — predicate violated
    x: i64 = inner()?
    R x + 1
}
F main() -> i64 { R outer() }
```

Expected: `vaisc check` exits non-zero with a stable diagnostic
(`error[E001]` or similar) explaining that `?` cannot be used in a
function whose return type is plain `i64`.

### v2 retro-validation

Both probes run via the standard empirical-fixture runner under
`compiler/tests/empirical/A2/A2-01_q_operator_core/`. The runner
asserts:
- positive: `vaisc check` exit 0 + binary exit 43.
- negative: `vaisc check` exit non-zero + stderr mentions Result.

### Promotion gate

A2-01 is promoted because:
- vaisdb depends on this exact pattern (lang/packages/vaisdb/src/
  sql/types.vais:339 `read_u8_checked(buf)?`).
- The lowering is mechanically simple (early-return).
- The negative fixture demonstrates the safe subset is bounded —
  uses outside the predicate are caught by the existing type checker.

INTEGRITY OK preserved (this promotion records existing behaviour;
no compiler change is required).

---

## A2-02 — `?` operator cross-module (vaisdb baseline path)

### Formal predicate

A use of the `?` operator is in the certified A2-02 subset iff ALL of
the following hold:

1. The receiver type and enclosing return type satisfy A2-01 conditions
   1 and 2 (Result/Option, Core-typed, matching error type).
2. The receiver expression resolves to a function defined in a
   different source file (cross-module). Module resolution must
   succeed under default imports — uncertified imports are A3
   quarantine territory (Step 11).
3. Both the receiver function's return type and the enclosing
   function's return type stay Core-typed across the module boundary.

### Positive fixture

`compiler/tests/empirical/A2/A2-02_q_operator_cross_module/probe_pos_inner.vais`
+ `probe_pos_main.vais` (two-file fixture). The inner file defines a
function that returns `Result<i64, str>`; the main file imports it
and uses `?` to propagate the error. Verifies cross-module `?`
lowers correctly. Baseline reference: vaisdb `lang/packages/vaisdb/
src/sql/types.vais:339` calls `read_u8_checked(buf)?` where
`read_u8_checked` is defined in `lang/packages/vaisdb/src/storage/
bytes.vais:40`.

Expected: `vaisc check` exits 0 on both files; binary runs; exit code
= 43 (= 42 + 1, same as A2-01 positive shape).

### Negative fixture

`probe_neg_inner.vais` + `probe_neg_main.vais` violates predicate
clause 1 (A2-01 derivative — return type does not match). Same
rejection mechanism as A2-01 negative — type checker rejects with
E001 (predicate enforcement via A4-11 typecheck-silent fix).

### Promotion gate

A2-02 is promoted because vaisdb depends on this exact pattern
(`read_u8_checked(buf)?` cross-module). The fixture pair codifies
the cross-module shape so future imports/resolver work cannot
silently regress this surface.

---

## A2-03 — `dyn` / trait object dispatch (narrow subset, multi-impl)

### Formal predicate

A `dyn Trait` dispatch is in the certified A2-03 subset iff ALL of
the following hold:

1. The trait `Trait` is declared and at least one `impl Trait for S`
   block exists for some struct `S`.
2. The receiver expression has type `&dyn Trait` or `&mut dyn Trait`
   or `Box<dyn Trait>`.
3. The dispatch site is a method call on the receiver (e.g.
   `g.method()` where `g: &dyn Trait`).
4. The trait method is one of those declared in the trait
   declaration; method name resolves via vtable indirection.

The compiler emits a vtable for each `Trait`, with one entry per
declared method, sorted alphabetically by method name (per LESSONS
L-007 — HashMap iteration determinism). Both backends (text-IR and
inkwell) use the same sorted-method ordering.

### Positive fixture

`compiler/tests/empirical/A2/A2-03_dyn_trait_dispatch/probe_pos.vais`:
defines `trait Greeter` + two impls (`Hello.greet → 42`,
`World.greet → 7`), then dispatches via `&dyn Greeter` parameter.
Calling `greet()` on a `World` value via the dyn parameter must
return 7 (correct vtable indirection), not 42 (the first-registered
impl bug F-23 fixed in v17).

Expected: `vaisc check` exits 0; binary runs; exit code = 49
(`Hello.greet() + World.greet() = 42 + 7`).

### Negative fixture

`probe_neg.vais`: passes a value that is not actually a trait
implementer (i64 cast as `&dyn Trait`). At runtime the vtable lookup
crashes (SIGSEGV / exit ≠ 0). Type-checker-level rejection of
i64-as-dyn is a separate silent surface (follow-up).

### Promotion gate

A2-03 is promoted because vaisdb sql/executor uses
`Box<dyn Executor>` chains (sort_agg.vais et al). Multi-impl
dispatch is the actual production pattern. Master-plan v22 history
records the inkwell + text-IR wiring (commits 27585530..ce54b903 +
70655014..daa795e2). LESSONS L-007 (HashMap iteration determinism)
is the empirical finding from this promotion.

---

## A2-04 — Closures (no escape, inline-only)

### Formal predicate

A closure expression `|params| body` is in the certified A2-04 subset
iff ALL of the following hold:

1. The closure is invoked at the same callee position where it is
   constructed, OR passed as an argument to a function that calls it
   synchronously and does not store it past the call.
2. The closure may capture variables from the enclosing scope by
   value, by reference, or by mutable reference (per `CaptureMode`).
3. The closure does NOT escape the function in which it is defined
   — it is not returned, not assigned to a struct field that
   outlives the function, not stored in a global.

The escape form (predicate clause 3 violated) is A4-15 (escape
closure capture loss), hard-blocked at TC layer in master-plan v39.

### Positive fixture

`compiler/tests/empirical/A2/A2-04_inline_closure/probe_pos.vais`:
`apply(|n| n + 1, 41)` returns 42 deterministically. The closure is
constructed at the call site and consumed inline.

Expected: `vaisc check` exits 0; binary runs; exit code = 42.

### Negative fixture

`probe_neg.vais`: `make_adder(x: i64) -> |i64| -> i64 { |n| n + x }`.
Closure escapes (captured `x` referenced after `make_adder` returns).
Pre-2026-05-08 was silent corruption (exit ≠ 42, varies by build).
Post-2026-05-08 (master-plan v39, A4-15 hard-block): `vaisc check`
rejects with E001 mentioning 'escape closure' + 'A4-15' marker.

### Promotion gate

A2-04 is promoted because the inline form is the safe subset that
the existing call-site machinery already supports correctly. The
hard-block on the escape form (A4-15) is what makes the predicate
"no escape" enforceable at TC layer.

---

## A2-05 — Function-pointer types in std API (bounded)

### Formal predicate

A function-pointer parameter is in the certified A2-05 subset iff
ALL of the following hold:

1. The parameter type is `fn(P1, P2, ...) -> R` (bare fn-pointer
   syntax) where each `Pi` and `R` are Core-typed.
2. The argument passed at the call site is a named function (no
   capture) that matches the parameter signature.
3. The fn-pointer is invoked synchronously at the call site or
   passed to another function that does the same.

Vais parser accepts `f: fn(i64) -> i64` parameter syntax (Step 7
Controlled-04 was a transient parser limit, resolved by 2026-05-04).

### Positive fixture

`compiler/tests/empirical/A2/A2-05_fn_pointer_param/probe_pos.vais`:
defines `apply(f: fn(i64) -> i64, x: i64) -> i64 { f(x) }`, then
two named functions `double` and `triple`. Multi-impl dispatch:
`apply(double, 10) + apply(triple, 10) = 50`.

Expected: `vaisc check` exits 0; binary runs; exit code = 50.

### Negative fixture

`probe_neg.vais`: passes an i64 value where a fn-pointer is expected.
Type checker rejects with E001 (i64 ≠ fn-pointer signature).

### Promotion gate

A2-05 is promoted because std API uses fn-pointer parameters in
several places (sort comparators, callback registration). The
fixture codifies the bounded subset (named functions, no closures)
that codegen already lowers correctly via direct call.

---

## How to add a new A2 entry

1. Define the formal predicate explicitly (every type/syntax
   constraint that a use must satisfy to be in-subset).
2. Author a positive `.vais` fixture that uses ONLY in-subset forms.
   Verify `vaisc check` and runtime succeed.
3. Author a negative `.vais` fixture that violates exactly one
   predicate clause. Verify `vaisc check` rejects with a stable
   diagnostic.
4. Add an entry to the table at the top of this file with status
   LANDED + the date.
5. Land the fixture pair under
   `compiler/tests/empirical/A2/<id>_<short_name>/`.
6. Run `bash compiler/scripts/check-empirical.sh A2` and confirm pass.
7. Run `bash compiler/scripts/check-integrity.sh` and confirm
   INTEGRITY OK with no baseline regression.

If steps 6-7 do not pass cleanly, the surface is NOT yet A2 — leave
the candidate in the master-plan §A2 list with a `status` annotation
explaining the blocker.
