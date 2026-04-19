# Vais Codegen Feature Matrix

> **Phase 3.12**: authoritative source for "what the LLVM codegen actually supports today". Paired with `docs/COMPILER_STAGES.md` which defines the 6-stage pipeline and `docs/LANGUAGE_SPEC.md` "Construct Status Matrix" which annotates construct-level support with ✓/◐/✗.
>
> This document is the **codegen-specific** view: for each feature, we record the LLVM-IR lowering status (ok / partial / unsupported) and the concrete symptom when a user hits an unsupported case.

---

## 1. Primitive operations

| Feature | Codegen | Notes |
|---------|---------|-------|
| Integer arith (`+`, `-`, `*`, `/`, `%`) | ✓ | All widths (i8..i128, u8..u128). |
| Float arith (`f32`, `f64`) | ✓ | Float literal inference propagates context. |
| Comparison (`<`, `<=`, `>`, `>=`, `==`, `!=`) | ✓ | Binary op operands auto-peel `&T` (Phase 2.12). |
| Bitwise (`&`, `\|`, `^`, `<<`, `>>`, `~`) | ✓ | |
| Boolean (`!`, short-circuit `&&`, `\|\|`) | ✓ | |
| Cast (`as`) | ✓ | Rust-style strict rules (Phase 158). See `CLAUDE.md §Type Conversion Rules`. |
| Pipe (`\|>`) | ✓ | |
| Range (`..`, `..=`) | ✓ | |
| Ternary (`cond ? a : b`) | ✓ | |

## 2. Control flow

| Feature | Codegen | Notes |
|---------|---------|-------|
| `I cond { … } EL { … }` | ✓ | |
| `I c1 { … } EL I c2 { … } EL { … }` | ✓ | |
| `M expr { pat => body, … }` | ✓ | |
| Match guard `pat I cond => body` | ✓ | Phase 1.11. |
| Infinite loop `L { … }` | ✓ | |
| While `LW cond { … }` | ✓ | |
| For-each `LF pat: iter { … }` | ✓ | |
| `B` (plain break) | ✓ | |
| `B <expr>` (break with value) | ◐ | TC: Phase 1.14 complete. **Full LLVM codegen** (phi node for loop-as-expression with non-trivial types): Phase 3.x. |
| `C` (continue) | ✓ | |
| `R` (return) | ✓ | |
| `D expr` / `D { block }` (defer) | ◐ | Basic scope-exit works. Edge cases (early return / break / nested loops) incomplete — Phase 3.16. |

## 3. Functions

| Feature | Codegen | Notes |
|---------|---------|-------|
| `F name(p: T) -> U { … }` | ✓ | |
| Expression body `F f(x) -> T = expr` | ✓ | |
| Generic function | ✓ | Monomorphization (Phase 114). |
| Async function `A F` / `.Y` | ✓ | Partial runtime — basic scheduler works. |
| `partial F` modifier | ✓ | Parsed; E034 totality analysis live. |
| `pure` / `io` effect modifier | ◐ | Parsed; effect inference not yet enforced — Phase 4.18. |
| `unsafe F` modifier | ✓ | Phase 1.18 — pass-through codegen. |
| Extern function `N F` / `N "C" { … }` | ✓ | |
| Self-recursion `@` | ✓ | |
| Higher-order param `f: (T) -> U` | ✓ | Phase 1.15. |

## 4. Types

| Feature | Codegen | Notes |
|---------|---------|-------|
| All primitives (i8..u128, f32/f64, bool, str) | ✓ | |
| `Vec<T>` | ✓ | Grow-on-push. Ref-counted storage. |
| `HashMap<K, V>` | ✓ | Open-addressing, deterministic ordering disabled. |
| `Option<T>` | ✓ | Both primitive `Optional(T)` and Named spelling. Phase 2.13 bridge. |
| `Result<T, E>` | ✓ | Both spellings. |
| Tuple `(T1, T2, …)` | ✓ | |
| Array lit `[1, 2, 3]` | ✓ | Bidirectional inference from expected Vec/Array type (Phase 1.12). |
| Reference `&T`, `&mut T` | ✓ | |
| Pointer `*T`, `*const T`, `*mut T` | ✓ | |
| Slice `&[T]`, `&mut [T]` | ✓ | |
| ConstArray `[T; N]` | ✓ | |
| SIMD `Vec2f32`/`Vec4f32`/… | ◐ | Type lexed/parsed. LLVM vector intrinsics incomplete — Phase 3.15. |
| `fn(T) -> U` (function type) | ✓ | |
| `dyn Trait` | ◐ | Parsed. Vtable codegen incomplete — Phase 4.21. |
| `linear T` / `affine T` | ◐ | Type stored; borrow checker integration incomplete — Phase 4.19. |
| Dependent `{x: T where pred}` | ◐ | Compile-time predicate check for literal values only. |

## 5. Structs / Enums / Impls

| Feature | Codegen | Notes |
|---------|---------|-------|
| `S Name { f: T, … }` | ✓ | |
| Struct literal `Name { f: v, … }` | ✓ | |
| Field access `s.f` | ✓ | |
| Field mutation `s.f = v` | ✓ (when `mut`) | |
| `Vec<Struct>[i].field = v` write | ◐ | Read OK. Write through index may fail — Phase 3.14. |
| `EN Name { Variant, … }` | ✓ | Use `EN` unambiguous. |
| `E Name` (legacy enum) | ✓ | Backward-compat path. |
| Tuple variant `Circle(i64)` | ✓ | |
| Unit variant `None` | ✓ | Phase 2.10 registered as pattern, not binding. |
| Generic struct `S Box<T>` | ✓ | |
| Generic enum `EN Box<T>` | ✓ | |
| `X Type { F method … }` (inherent impl) | ✓ | |
| `X Type: Trait { … }` (trait impl) | ✓ | |
| Cross-file impl split | ✗ | Phase 2.9 decision (a) — co-locate `S` and `X`. |
| `W Trait { F method … }` with default | ✓ | |
| Object safety check | ✓ | |

## 6. Patterns

| Feature | Codegen | Notes |
|---------|---------|-------|
| Wildcard `_` | ✓ | |
| Ident binding `x` | ✓ | Phase 2.10: enum variant names don't bind. |
| Literal | ✓ | |
| Tuple `(a, b)` | ✓ | |
| Struct `Name { f: p }` | ✓ | |
| Variant `Some(x)` / `Ok(x)` / `Err(e)` | ✓ | |
| Range `1..=5` | ✓ | |
| Or `a \| b \| c` | ✓ | |
| Alias `x @ pattern` | ✓ | |
| Guard `pat I cond => body` | ✓ | Phase 1.11. |

## 7. Stdlib methods

> For the canonical list of method return types, see
> `crates/vais-types/src/builtins/method_returns.rs` (Phase 2.11).

| Receiver | Method | Codegen | Notes |
|----------|--------|---------|-------|
| `Vec<T>` | `.new()` | ✓ | |
| `Vec<T>` | `.push(v)` | ✓ | |
| `Vec<T>` | `.pop()` | ✓ | Returns `Option<T>`. |
| `Vec<T>` | `.len()`, `.is_empty()`, `.capacity()` | ✓ | |
| `Vec<T>` | `.get(i)` | ✓ | Returns `Option<&T>` — binary op auto-deref (Phase 2.12). |
| `Vec<T>` | `.contains(v)` | ✓ | |
| `Vec<T>` | `.first()`, `.last()` | ✓ | |
| `Vec<T>` | `.reverse()`, `.sort()`, `.clear()`, `.truncate()` | ✓ | |
| `HashMap<K,V>` | `.new()`, `.insert(k,v)`, `.get(k)`, `.contains_key(k)`, `.remove(k)`, `.clear()` | ✓ | |
| `Str` / `&str` | `.len()`, `.is_empty()`, `.contains()`, `.starts_with()`, `.ends_with()` | ✓ | |
| `Str` / `&str` | `.to_upper()`, `.to_lower()`, `.trim()` | ◐ | Present in stdlib; codegen may route through runtime helpers. |
| `Option<T>` | `.is_some()`, `.is_none()`, `.unwrap()` | ✓ | |
| `Result<T,E>` | `.is_ok()`, `.is_err()`, `.unwrap()` | ✓ | |

## 8. Async / await

| Feature | Codegen | Notes |
|---------|---------|-------|
| `A F fn(…)` declaration | ✓ | |
| `.Y` / `.await` postfix | ✓ | |
| `Future<T>` type | ✓ | |
| Task spawn | ✗ | `spawn` keyword removed Phase 195. Use runtime task API. |

## 9. Effect system

| Feature | Codegen | Notes |
|---------|---------|-------|
| `partial F` | ✓ | E034 totality analysis active. |
| `pure F` | ◐ | Modifier parsed; TC not enforcing pure constraint — Phase 4.18. |
| `io F` | ◐ | Parsed, not enforced. |
| `effect F` (reserved) | ⊖ | Token reserved, no grammar production. |

## 10. Advanced language features

| Feature | Codegen | Notes |
|---------|---------|-------|
| `comptime { … }` block | ◐ | Partial evaluation — Phase 4.20. |
| Declarative macro `macro name!(…)` | ◐ | Experimental. |
| Yield iterator `yield expr` | ◐ | Experimental — Phase 4.22. |
| Move closure `move \|x\| …` | ◐ | Basic support; full move semantics — Phase 4.23. |

---

## Known TC-passes-but-codegen-fails cases

Codegen failures after TC success should be logged here as they're
discovered. The goal of Phase 3.x is **zero** drop from TC to codegen.

Current known cases (as of 2026-04-19):

| Trigger | Symptom | Target phase |
|---------|---------|--------------|
| `F f(opt: Option<Struct>) -> Option<Primitive>` with naked Option param | LLVM IR: `invalid type for function argument %Option$Role %opt` | Phase 3.14 / 3.15 — struct-named Optional param lowering |
| `[]` / `[1,2,3]` literal without type hint | TC: `expected Vec<i64>, found *?0` (pre-Phase 1.12) | **Resolved** Phase 1.12 |
| `Some(r.field)` re-wrap in match arm | TC: `expected u64, found Role` (pre-Phase 2.10) | **Resolved** Phase 2.10 |
| `V[i].field = expr` on Vec of struct | codegen: partial write-through | Phase 3.14 |
| Complex `L { … B expr }` loop-as-expr with non-trivial type | phi-node generation incomplete | Phase 3.x |
| `s.parse_i64()`, `s.parse_u64()`, `s.parse_i32()`, `s.parse_u32()` | TC knows return = `Result<iN, str>`; codegen `C002: Undefined function` | Phase 3.13 — runtime impl |
| `s.parse_f64()`, `s.parse_f32()` | TC knows return; codegen missing | Phase 3.13 — runtime impl |
| `Vec4f32::new(...)` / `Vec2i64::new(...)` (SIMD constructors) | Parser rejects SIMD type token as expression head (P001 "found Vec4f32, expected expression"). Lexer has tokens. | Phase 3.15 — parser constructor + LLVM vector intrinsics |

---

## How to extend

Adding a new codegen feature requires:

1. Parser support (`crates/vais-parser/`).
2. TC support (`crates/vais-types/`). For types with generic params,
   extend `ResolvedType` and `inference/unification.rs`.
3. AST node (`crates/vais-ast/`).
4. Codegen lowering (`crates/vais-codegen/src/inkwell/`).
5. Update this matrix: row in the relevant section with ✓ / ◐ / ✗.
6. Add e2e test in `crates/vaisc/tests/e2e/`.
7. Run `./scripts/check-integrity.sh` — baseline must not regress.

## Reference sources

- `docs/COMPILER_STAGES.md` — 6-stage pipeline.
- `docs/LANGUAGE_SPEC.md` §Construct Status Matrix — syntax-level ✓/◐/✗.
- `docs/TYPE_SYSTEM.md` — unification, coercion, move/borrow rules.
- `docs/language/COOKBOOK.md` — common agent pitfalls + resolutions.
- `crates/vais-types/src/builtins/method_returns.rs` — canonical method
  return-type table.
- `crates/vais-types/src/inference/option_result_bridge.rs` — canonical
  Option/Result normalization (Phase 2.13).
