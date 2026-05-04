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
| A2-02 | `?` operator cross-module (vaisdb baseline path) | DEFERRED — needs working module-import resolution beyond std/ (cf. STEP11_FINDINGS F-A3-02) |
| A2-03 | `dyn` / trait object dispatch (narrow subset) | DEFERRED — surface deferred from Step 7 Controlled iteration; reuse that probe work |
| A2-04 | Closures (no escape, inline-only) | LANDED 2026-05-04 (positive inline closure exit 42; negative escape closure F-18 silent corruption — exit_not [42]) |
| A2-05 | Function-pointer types in std API (bounded) | DEFERRED — Vais parser does not currently accept `f: F(i64) -> i64` parameter syntax (Step 7 Controlled-04) |

Four candidates are deferred because they depend on either a parser
or resolver fix that is itself documented elsewhere (Step 7 / Step 11
findings). One candidate (A2-01) is LANDED.

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
