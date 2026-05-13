# Phase 194 Recon-D

## Bug 1: Option<T> match-arm getelementptr type mismatch

**Error (measured):**
```
/tmp/.vais-cache/p194_b1.ll:167:42: error: '%t10' defined with type
'%Option = type { i32, { i64 } }' but expected 'ptr'
  167 |   %t12 = getelementptr %Option, %Option* %t10, i32 0, i32 0
```

**IR snippet (measured, /tmp/p194_b1_p194_b1.ll):**
```llvm
8:  %Option = type { i32, { i64 } }
163:  %t10 = call %Option @Vec_get_opt$i64(%Vec$i64* %v.1, i64 0)
165:  br label %match.check1
166:match.check1:
167:  %t12 = getelementptr %Option, %Option* %t10, i32 0, i32 0
```

`%t10` is the SSA **value** (`%Option = {i32, {i64}}`) returned by
`Vec_get_opt$i64`. The match-arm treats it as a pointer operand to
`getelementptr`.

**Real root cause (measured, NOT monomorphization):**

`crates/vais-codegen/src/type_inference.rs:681-690` — the Vec<T>
method hardcoded return-type table:
```rust
if let Some(elem_ty) = vec_elem {
    return match method.node.as_str() {
        "len" | "capacity" => ResolvedType::U64,
        "push" | "insert" | ... => ResolvedType::I64,
        "pop" | "get" | "last" | "first" => elem_ty,
        "clone" => recv_type,
        "data" => ResolvedType::I64,
        _ => ResolvedType::I64,   // ← get_opt / pop_opt fall here
    };
}
```

`get_opt` returns `Option<T>`, but `infer_expr_type` reports `I64` for
it. Then in `stmt.rs:137-142`, `let opt := v.get_opt(0)` takes the
use_ssa branch (`!matches!(resolved_ty, Named)` → true when ty is I64),
so `opt` is registered as an SSA **value** aliasing the `%Option` call
result. The match scrutinee reads this value and GEPs it as if it
were a pointer.

**Hypothesis about Phase 193 C4 fix**: the TC fix made `Some(p)` bind
`p` correctly, but codegen's type-inference table never learned that
`get_opt` returns `Option<T>`. So the real P194-1 scope is NOT enum
monomorphization at all; it's a hardcoded-table gap. Enum
monomorphization exists today (`%Option` with payload `{ i64 }` works
for all `T` because the payload slot is uniformly `i64`).

**Likely Fix Sites (ranked):**
1. `crates/vais-codegen/src/type_inference.rs:681-690` — add
   `"get_opt" | "pop_opt" | "first_opt" | "last_opt" => Named{ "Option", [elem_ty] }`
   to the Vec<T> method return-type table. **1–3 line fix.**
2. `crates/vais-types` stdlib signature path may also need to register
   `get_opt` return; verify `std/vec.vais:164` `F get_opt(...) -> Option<T>`
   is reflected in `resolved_function_sigs`. Low priority — the
   type_inference.rs gap is the dominant cause.
3. (secondary) Future-proof: teach `infer_expr_type` MethodCall to
   consult `resolved_function_sigs` BEFORE the hardcoded Vec/Str
   tables, so unknown methods on generic containers fall through to
   the ground-truth TC signature.

---

## Bug 2: Closure unit body → lambda signature mismatch

**Error (measured, /tmp/p194_b2.vais):**
```
/tmp/p194_b2.ll:795:7: error: value doesn't match function result type 'i64'
  795 |   ret {} zeroinitializer
```

**Lambda IR (measured):**
```llvm
define i64 @__lambda_0(i64 %0) {
entry:
  %__cap_n = alloca i64, align 8
  store i64 %0, ptr %__cap_n, align 4
  %n = load i64, ptr %__cap_n, align 4
  %printf_call = call i32 (ptr, ...) @printf(ptr @.str.0, i64 %n)
  %puts_nl = call i32 (ptr, ...) @printf(ptr @.str.1)
  ret {} zeroinitializer
}
```

**Root cause (measured):**

`crates/vais-codegen/src/expr_helpers_misc.rs:369-377`:
```rust
let mut lambda_ir = format!(
    "define i64 @{}({}) {{\nentry:\n",
    lambda_name,
    param_strs.join(", ")
);
...
write_ir!(lambda_ir, "  ret i64 {}\n}}", body_val);
```

Both the signature and the `ret` instruction hardcode `i64`, but when
the lambda body is `puts("...")` (unit-returning call), `body_val` ends
up as `{} zeroinitializer` (codegen's unit sentinel).

**Likely Fix Sites (ranked):**
1. `expr_helpers_misc.rs:367` — after `generate_expr(body, ...)`,
   compute the body's LLVM type via `self.llvm_type_of(&body_val)` and
   use it in the signature + ret. Guard: if body type is `i64` (most
   closures today), keep existing path.
2. Alternative: TC coerce unit body with implicit `; 0` — but that
   changes semantics. Reject.
3. Alternative: at lambda-emit time, if body_val is the unit sentinel,
   substitute `ret i64 0`. Simplest, matches current caller assumption
   that `show()` returns i64.

**Decision recommendation**: option 3 (auto-coerce unit → 0). Rationale:
all existing lambda callers assume `call i64 %lambda(...)`, so changing
the signature breaks the callsite. A unit body is a side-effect-only
closure; coercing to 0 preserves all caller code paths.

inkwell backend check (hypothesized): `inkwell/gen_aggregate.rs`
`generate_lambda` likely has the same pattern. Must inspect during fix.

---

## Bug 3: Higher-order `f(x)` param call — Undefined function

**Error (measured, /tmp/p194_b3.vais):**
```
error[C002] Undefined function
Undefined function: f
```

**Lookup flow (measured):**

`crates/vais-codegen/src/generate_expr_call.rs:237-291`:
```rust
let (fn_name, is_indirect) = if let Expr::Ident(name) = &func.node {
    if let Some(instantiations_list) = self.generics.fn_instantiations.get(name) {
        ...
    } else if self.types.functions.contains_key(name) {
        (name.clone(), false)
    } else if self.fn_ctx.locals.contains_key(name) {
        (name.clone(), true) // Lambda call
    } else if self.types.declared_functions.contains(name) {
        ...
    } else if matches!(name.as_str(), "malloc" | ...) {
        ...
    } else {
        ... UndefinedFunction ...
    }
```

L249 already handles `locals.contains_key(name)` as an indirect call.
So params registered in `fn_ctx.locals` SHOULD be reachable. Params
are registered in `function_gen/codegen.rs:114` via
`LocalVar::param(ty.clone(), llvm_name)`.

Why does `f` miss? **Hypothesized** (needs direct measurement): the
failing path may be inkwell backend, not the text backend
`generate_expr_call.rs`. The text backend looks correct on paper. Need
to trace which backend is emitting main() here by checking the error
span or adding an eprintln in each backend's Call handler.

Candidate inkwell sites:
- `crates/vais-codegen/src/inkwell/gen_aggregate.rs:711` and `:737` —
  both raise UndefinedFunction.

**Likely Fix Sites (ranked, hypothesized — needs backend confirmation):**
1. `crates/vais-codegen/src/inkwell/gen_aggregate.rs:~700-740` — ensure
   the Ident-Call lookup consults `self.locals` before failing, and
   emits `call TYPE %f(ARGS)` when found (function-pointer call).
2. If text backend: the `is_indirect=true` branch's downstream IR
   emission must handle function-pointer types for the callee. Grep
   for how `is_indirect` is consumed later in generate_expr_call.rs.
3. TC-level: confirm param type `(i64) -> i64` is resolved to
   `ResolvedType::Fn { params, ret }` and reaches the codegen's
   type_to_llvm as a function pointer type.

---

## Bug 4: examples/ fresh-rebuild E2E gate (design only, no bug)

**Counts (measured):**
- `ls examples/*.vais | wc -l` → **188 files**
- `grep -l "^F main" examples/*.vais | wc -l` → **188 files** (all
  examples have main — no skip list needed)

**Fresh compile time (measured, 3 samples, --emit-ir):**
- examples/hello.vais: <1s
- examples/closure_counter.vais: <1s
- examples/simple_vec_test.vais: <1s

Extrapolated total for `--emit-ir` on all 188: ~3 minutes.
Full clang link for all 188: likely ~5-8 minutes (clang dominates).

**Template (measured, crates/vaisc/tests/selfhost_lexer_tests.rs:14-50):**
```rust
fn compile_to_ir(source: &str) -> Result<String, String> {
    let _tokens = tokenize(source).map_err(|e| format!("Lexer error: {:?}", e))?;
    let module = parse(source).map_err(|e| format!("Parser error: {:?}", e))?;
    let mut checker = TypeChecker::new();
    checker.check_module(&module)?;
    let mut gen = CodeGenerator::new("selfhost_test");
    gen.set_resolved_functions(checker.get_all_functions().clone());
    gen.set_type_aliases(checker.get_type_aliases().clone());
    let ir = gen.generate_module(&module)?;
    Ok(ir)
}
```

The file-based E2E template (same file, `compile_and_run`) also exists
and runs clang + executes.

**Design recommendation:**
- **One `#[test]` per file**, table-driven via a `[std::fs::read_dir]`
  loop that generates subtests would be ideal but Rust's `#[test]`
  macro doesn't support runtime test generation. Pragmatic: generate
  the list at build time via `build.rs` or `include_str!`, or just
  hand-write a single test that iterates all 188 files and collects
  failures.
- **`#[ignore]` by default**: 3-5 min extra CI time is nontrivial.
  Run via `cargo test -- --ignored examples_fresh_rebuild` in a
  dedicated CI job, not every PR.
- **IR-only check** (`--emit-ir` equivalent, skip clang link): much
  faster (~3 min), catches TC/codegen regressions which is the
  stated Recon-C concern (cache hiding codegen regressions). Skip
  the binary-link stage.

---

## Summary table

| # | Bug | Fix site (ranked #1) | Lines | Backend |
|---|---|---|---|---|
| 1 | Option<T> GEP value-as-ptr | type_inference.rs:681-690 Vec method table | ~3 | text |
| 2 | Closure unit return mismatch | expr_helpers_misc.rs:377 ret i64 hardcode | ~5 | text (+ inkwell mirror) |
| 3 | Higher-order f(x) C002 | inkwell/gen_aggregate.rs:~711 Ident-Call lookup | ~8 | inkwell (hypothesized) |
| 4 | fresh-rebuild gate | NEW tests/examples_fresh_rebuild.rs | ~50 | n/a (test) |

Total estimated impact: ~70 lines across 3-4 files + 1 new test file.
All four are narrower than the Phase 194 Plan described. Bug 1's scope
in particular shrinks dramatically — from "enum monomorphization" to
"one table entry".

PROMISE: COMPLETE
