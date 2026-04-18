# Vais Type System — Authoritative Reference

> **Phase 2.8 Document** — Single source of truth for unification rules,
> coercion policy, and type variable lifecycle. Future type-system work MUST
> consult this file and keep it up to date.
>
> All claims are cross-referenced to a file:line or to CLAUDE.md. No
> speculation.

---

## Table of Contents

1. [Overview — ResolvedType Variants](#1-overview--resolvedtype-variants)
2. [Unification Algorithm](#2-unification-algorithm)
3. [ResolvedType × ResolvedType Unification Table](#3-resolvedtype--resolvedtype-unification-table)
4. [Type Variable / Fresh Var Creation](#4-type-variable--fresh-var-creation)
5. [Coercion Rules](#5-coercion-rules)
6. [Named ↔ Optional/Result Bridge (Phase 326)](#6-named-optionalresult-bridge-phase-326)
7. [Auto-deref Rules](#7-auto-deref-rules)
8. [Generic Instantiation](#8-generic-instantiation)
9. [Known Gaps / TODO](#9-known-gaps--todo)
10. [How to Extend](#10-how-to-extend)

---

## 1. Overview — ResolvedType Variants

Source: `crates/vais-types/src/types/resolved.rs:114`

Every value in the Vais type system is represented as a `ResolvedType` variant.
The enum is `#[derive(Debug, Clone, PartialEq, Eq, Hash)]`.

### 1.1 Primitive Types

| Variant | Vais spelling | Description |
|---------|--------------|-------------|
| `I8`    | `i8`  | Signed 8-bit integer |
| `I16`   | `i16` | Signed 16-bit integer |
| `I32`   | `i32` | Signed 32-bit integer |
| `I64`   | `i64` | Signed 64-bit integer (default integer literal type) |
| `I128`  | `i128`| Signed 128-bit integer |
| `U8`    | `u8`  | Unsigned 8-bit integer |
| `U16`   | `u16` | Unsigned 16-bit integer |
| `U32`   | `u32` | Unsigned 32-bit integer |
| `U64`   | `u64` | Unsigned 64-bit integer |
| `U128`  | `u128`| Unsigned 128-bit integer |
| `F32`   | `f32` | 32-bit float |
| `F64`   | `f64` | 64-bit float |
| `Bool`  | `bool`| Boolean — NOT an integer (no implicit coercion to/from integers) |
| `Str`   | `str` | String slice |
| `Unit`  | `()`  | Unit / void type |

Source: `crates/vais-types/src/types/resolved.rs:115–130`

### 1.2 Compound Types

| Variant | Vais spelling | Description |
|---------|--------------|-------------|
| `Array(Box<ResolvedType>)` | `[T]` | Dynamic-length array |
| `ConstArray { element, size }` | `[T; N]` | Fixed-size array, size is a `ResolvedConst` |
| `Map(Box<K>, Box<V>)` | `[K:V]` | Hash map |
| `Tuple(Vec<ResolvedType>)` | `(T1, T2, ...)` | Heterogeneous product type |
| `Optional(Box<T>)` | `T?` | Sugar for `Option<T>` — see §6 for bridge |
| `Result(Box<T>, Box<E>)` | `Result<T,E>` | Error-propagation type |
| `Pointer(Box<T>)` | `*T` | Raw pointer (IR-level i64) |
| `Ref(Box<T>)` | `&T` | Immutable reference |
| `RefMut(Box<T>)` | `&mut T` | Mutable reference |
| `Slice(Box<T>)` | `&[T]` | Immutable fat pointer slice |
| `SliceMut(Box<T>)` | `&mut [T]` | Mutable fat pointer slice |
| `Range(Box<T>)` | `Range<T>` | Range over element type |
| `Future(Box<T>)` | `Future<T>` | Async future |

Source: `crates/vais-types/src/types/resolved.rs:132–151`

### 1.3 Function Types

| Variant | Description |
|---------|-------------|
| `Fn { params, ret, effects }` | First-class function value with optional effect annotation |
| `FnPtr { params, ret, is_vararg, effects }` | C-ABI function pointer; supports varargs |

Source: `crates/vais-types/src/types/resolved.rs:153–168`

### 1.4 Named / Generic Types

| Variant | Description |
|---------|-------------|
| `Named { name, generics }` | User-defined struct / enum, or stdlib type like `Vec<T>`, `Box<T>` |
| `Generic(String)` | Type parameter placeholder, e.g. `T` in `F foo<T>` |
| `ConstGeneric(String)` | Const generic parameter, e.g. `N` in `[T; N]` |

Source: `crates/vais-types/src/types/resolved.rs:171–183`

### 1.5 Special / Inference Types

| Variant | Description |
|---------|-------------|
| `Var(usize)` | Fresh type variable allocated by inference engine — unifies with anything |
| `Unknown` | Placeholder for unresolvable types; unifies with anything (non-error) |
| `Never` | Bottom type `!`; unifies with any type (diverging expressions: `return`, `break`) |
| `DynTrait { trait_name, generics }` | Dynamic trait object `dyn Trait<T>` — accepts any concrete type |

Source: `crates/vais-types/src/types/resolved.rs:176–205`

### 1.6 Advanced Types

| Variant | Description |
|---------|-------------|
| `Vector { element, lanes }` | SIMD vector `<N x T>` — strict lane + element matching |
| `Associated { base, trait_name, assoc_name, generics }` | Associated type `<T as Trait>::Item`; GAT-capable |
| `Linear(Box<T>)` | Linear type wrapper — must be used exactly once |
| `Affine(Box<T>)` | Affine type wrapper — can be used at most once |
| `Dependent { var_name, base, predicate }` | Refinement type `{n: i64 \| n > 0}` |
| `RefLifetime { lifetime, inner }` | Reference with explicit lifetime `&'a T` |
| `RefMutLifetime { lifetime, inner }` | Mutable reference with explicit lifetime `&'a mut T` |
| `Lifetime(String)` | Lifetime parameter `'a`, `'static` |

Source: `crates/vais-types/src/types/resolved.rs:193–253`

---

## 2. Unification Algorithm

Source: `crates/vais-types/src/inference/unification.rs`

### 2.1 Entry Point: `unify`

```
TypeChecker::unify(expected: &ResolvedType, found: &ResolvedType) -> TypeResult<()>
```

Location: `crates/vais-types/src/inference/unification.rs:53`

The function is `#[inline]` and `pub(crate)`. Callers are in
`checker_expr/`, `checker_fn.rs`, and `inference/inference_modes.rs`.

### 2.2 Execution Flow

```
unify(expected, found)
  │
  ├─ 1. Fast path: ptr identity → Ok(())                [line 59]
  │
  ├─ 2. apply_substitutions(expected)                   [line 63]
  │    apply_substitutions(found)                        [line 64]
  │
  ├─ 3. Structural equality check → Ok(())              [line 66]
  │
  ├─ 4. str / Str / String alias coercion               [line 78]
  │
  ├─ 5. &Str ↔ &str ref alias coercion                  [line 88]
  │
  ├─ 6. Optional<T> ↔ T permissive coercion             [line 103]
  │
  ├─ 7. Box<T> ↔ Box (no generics) degenerate form      [line 116]
  │
  ├─ 8. Box<T> ↔ T coercion (both directions)           [line 134]
  │
  ├─ 9. &Box<T> ↔ &T / &mut Box<T> ↔ &mut T            [line 153]
  │
  ├─ 10. Slice coercion: &Vec<T> ↔ &[T] ↔ Vec<T>       [line 176]
  │
  └─ 11. Main match arms (§3 table below)               [line 209]
       └─ fallthrough: Err(TypeError::Mismatch)          [line 583]
```

### 2.3 `occurs_in` (Occurs Check)

Location: `crates/vais-types/src/inference/unification.rs:13`

Before binding `Var(id) → t`, `occurs_in(id, t)` checks whether `id` appears
anywhere inside `t`. If it does, the binding is silently skipped (returns
`Ok(())`) to prevent cyclic substitutions that would cause
`apply_substitutions` to loop infinitely.

Traversed variants:
- Single-inner wrappers: `Array`, `Optional`, `Ref`, `RefMut`, `Slice`,
  `SliceMut`, `Pointer`, `Range`, `Future`, `Linear`, `Affine`
- Two-inner: `Result`, `Map`
- Multi-inner: `Tuple`
- Function types: `Fn`, `FnPtr`
- Named / DynTrait — traverses `generics`
- `RefLifetime`, `RefMutLifetime` — traverses `inner`
- `Dependent` — traverses `base`
- `ConstArray`, `Vector` — traverses `element`
- `Associated` — traverses `base` and `generics`
- Leaf types (primitives, `Bool`, `Str`, `Unit`, etc.) → always `false`

Source: `crates/vais-types/src/inference/unification.rs:13–49`

### 2.4 `apply_substitutions`

Location: `crates/vais-types/src/inference/substitution.rs:16`

Recursively replaces every `Var(id)` node with its substitution from
`TypeChecker::substitutions: HashMap<usize, ResolvedType>`.

Fast paths (in order):
1. `substitutions` map is empty → clone and return.
2. Primitive type → clone and return (primitives never contain `Var`).
3. `!contains_var(ty)` → clone and return.

`Var(id)` resolution is recursive: if `substitutions[id]` itself contains
another `Var`, `apply_substitutions` is called again on the result.

Source: `crates/vais-types/src/inference/substitution.rs:16–172`

---

## 3. ResolvedType × ResolvedType Unification Table

All rows below correspond to explicit `match` arms in `unify` or to pre-match
coercion guards. Ordering follows the actual source order.

| # | LHS | RHS | Result | Notes | Code ref |
|---|-----|-----|--------|-------|----------|
| 1 | `Var(id)` | any `t` | `Ok(())` — binds `id → t` | Occurs-check skips binding if cyclic | `unification.rs:211` |
| 2 | any `t` | `Var(id)` | `Ok(())` — binds `id → t` | Symmetric with row 1 | `unification.rs:211` |
| 3 | `Unknown` | any | `Ok(())` | Placeholder — never causes error | `unification.rs:227` |
| 4 | any | `Unknown` | `Ok(())` | Symmetric with row 3 | `unification.rs:227` |
| 5 | `Never` | any | `Ok(())` | Bottom type — diverging expression | `unification.rs:229` |
| 6 | any | `Never` | `Ok(())` | Symmetric with row 5 | `unification.rs:229` |
| 7 | `Generic(name)` | any | `Ok(())` | Type erasure for generic params | `unification.rs:231` |
| 8 | any | `Generic(name)` | `Ok(())` | Symmetric with row 7 | `unification.rs:231` |
| 9 | `Array(a)` | `Array(b)` | `unify(a, b)` | Recursive element unification | `unification.rs:232` |
| 10 | `Optional(a)` | `Optional(b)` | `unify(a, b)` | Recursive inner unification | `unification.rs:233` |
| 11 | `Result(a_ok, a_err)` | `Result(b_ok, b_err)` | `unify(ok)?; unify(err)` | Both components must unify | `unification.rs:234` |
| 12 | `Named{"Option",[T]}` | `Optional(U)` | `unify(T, U)` | Phase 326 bridge (§6) | `unification.rs:245` |
| 13 | `Optional(U)` | `Named{"Option",[T]}` | `unify(T, U)` | Symmetric Phase 326 bridge | `unification.rs:246` |
| 14 | `Named{"Result",[T,E]}` | `Result(ok,err)` | `unify(T,ok)?; unify(E,err)` | Phase 326 bridge (§6) | `unification.rs:251` |
| 15 | `Result(ok,err)` | `Named{"Result",[T,E]}` | `unify(T,ok)?; unify(E,err)` | Symmetric Phase 326 bridge | `unification.rs:252` |
| 16 | `Ref(a)` | `Ref(b)` | `unify(a, b)` | Inner type must unify | `unification.rs:258` |
| 17 | `RefMut(a)` | `RefMut(b)` | `unify(a, b)` | Inner type must unify | `unification.rs:259` |
| 18 | `Slice(a)` | `Slice(b)` | `unify(a, b)` | Element types must unify | `unification.rs:260` |
| 19 | `SliceMut(a)` | `SliceMut(b)` | `unify(a, b)` | Element types must unify | `unification.rs:261` |
| 20 | `Pointer(a)` | `Pointer(b)` | `unify(a, b)` | Pointee types must unify | `unification.rs:262` |
| 21 | `Range(a)` | `Range(b)` | `unify(a, b)` | Range element types must unify | `unification.rs:263` |
| 22 | `Future(a)` | `Future(b)` | `unify(a, b)` | Output types must unify | `unification.rs:264` |
| 23 | `Tuple(as)` | `Tuple(bs)` | `unify each pairwise` (requires same len) | Length mismatch → fallthrough to Err | `unification.rs:265` |
| 24 | `Fn{pa,ra}` | `Fn{pb,rb}` | `unify params?; unify rets` (same arity) | Effect annotations ignored in unification | `unification.rs:271` |
| 25 | `Fn{pa,ra}` | `FnPtr{pb,rb}` | `unify params?; unify rets` (same arity) | Functions coerce to function pointers | `unification.rs:289` |
| 26 | `FnPtr{pa,ra}` | `Fn{pb,rb}` | `unify params?; unify rets` (same arity) | Symmetric with row 25 | `unification.rs:301` |
| 27 | `FnPtr{pa,ra}` | `FnPtr{pb,rb}` | `unify params?; unify rets` (same arity) | C-ABI pointer ↔ C-ABI pointer | `unification.rs:319` |
| 28 | `Named{na,ga}` | `Named{nb,gb}` | `unify each generic pairwise` (name equal, same generic count) | Name mismatch → fallthrough to Err | `unification.rs:337` |
| 29 | integer | integer | `Ok(())` | Any integer type unifies with any other (§5) | `unification.rs:354` |
| 30 | integer | float | `Ok(())` | Integer literal adapts to float context (Phase 160-A) | `unification.rs:357` |
| 31 | float | integer | `Ok(())` | Symmetric with row 30 | `unification.rs:358` |
| 32 | `F32` | `F64` | `Ok(())` | Float literal inference (§5) | `unification.rs:364` |
| 33 | `F64` | `F32` | `Ok(())` | Symmetric with row 32 | `unification.rs:364` |
| 34 | `Unit` | `I64` | `Ok(())` | Void context: i64 return in void function | `unification.rs:368` |
| 35 | `I64` | `Unit` | `Ok(())` | Symmetric with row 34 | `unification.rs:368` |
| 36 | `Result(_, _)` | `Unit` | `Ok(())` | Implicit `Ok(())` wrapping | `unification.rs:372` |
| 37 | `Unit` | `Result(_, _)` | `Ok(())` | Symmetric with row 36 | `unification.rs:373` |
| 38 | `Optional(_)` | `Unit` | `Ok(())` | Implicit `None`/unit in optional context | `unification.rs:374` |
| 39 | `Unit` | `Optional(_)` | `Ok(())` | Symmetric with row 38 | `unification.rs:375` |
| 40 | `Named{"Vec",[T]}` | `Slice(E)` | `unify(T, E)` | Vec coerces to slice | `unification.rs:377` |
| 41 | `Slice(E)` | `Named{"Vec",[T]}` | `unify(T, E)` | Symmetric with row 40 | `unification.rs:378` |
| 42 | `Named{"Vec",[T]}` | `Ref(Slice(E))` | `unify(T, E)` | Vec coerces to &[T] | `unification.rs:383` |
| 43 | `Ref(Slice(E))` | `Named{"Vec",[T]}` | `unify(T, E)` | Symmetric with row 42 | `unification.rs:384` |
| 44 | `Named{"Vec",[T]}` | `Ref(other)` | `Ok(())` | Permissive: Vec ↔ &T fallback | `unification.rs:390` |
| 45 | `Ref(Vec<T>)` | `Slice(E)` | `unify(T, E)` | Rust-style auto-deref &Vec<T> → &[T] (Phase 163) | `unification.rs:395` |
| 46 | `Slice(E)` | `Ref(Vec<T>)` | `unify(T, E)` | Symmetric with row 45 | `unification.rs:396` |
| 47 | `RefMut(Vec<T>)` | `SliceMut(E)` | `unify(T, E)` | Mutable auto-deref (Phase 163) | `unification.rs:406` |
| 48 | `SliceMut(E)` | `RefMut(Vec<T>)` | `unify(T, E)` | Symmetric with row 47 | `unification.rs:407` |
| 49 | `Pointer(_)` | `I64` | `Ok(())` | Pointers are i64 at IR level | `unification.rs:421` |
| 50 | `I64` | `Pointer(_)` | `Ok(())` | Symmetric with row 49 | `unification.rs:422` |
| 51 | `Pointer(p)` | `Slice(s)` | `unify(p, s)` | *u8 ↔ &[u8] byte buffer (Phase 162) | `unification.rs:426` |
| 52 | `Slice(s)` | `Pointer(p)` | `unify(p, s)` | Symmetric with row 51 | `unification.rs:427` |
| 53 | `Pointer(p)` | `SliceMut(s)` | `unify(p, s)` | Mutable variant (Phase 162) | `unification.rs:428` |
| 54 | `SliceMut(s)` | `Pointer(p)` | `unify(p, s)` | Symmetric with row 53 | `unification.rs:429` |
| 55 | `ConstArray{element,..}` | `Pointer(p)` | `unify(element, p)` | C-style array decay to pointer (Phase 162) | `unification.rs:432` |
| 56 | `Pointer(p)` | `ConstArray{element,..}` | `unify(element, p)` | Symmetric with row 55 | `unification.rs:433` |
| 57 | `Array(element)` | `Pointer(p)` | `unify(element, p)` | Dynamic array decay to pointer (Phase 162) | `unification.rs:434` |
| 58 | `Pointer(p)` | `Array(element)` | `unify(element, p)` | Symmetric with row 57 | `unification.rs:435` |
| 59 | `Linear(inner)` | any | `unify(inner, other)` | Unwrap linear type | `unification.rs:437` |
| 60 | any | `Linear(inner)` | `unify(inner, other)` | Symmetric with row 59 | `unification.rs:438` |
| 61 | `Affine(inner)` | any | `unify(inner, other)` | Unwrap affine type | `unification.rs:440` |
| 62 | any | `Affine(inner)` | `unify(inner, other)` | Symmetric with row 61 | `unification.rs:441` |
| 63 | `Dependent{base,..}` | any | `unify(base, other)` | Predicate ignored in unification (checked separately) | `unification.rs:445` |
| 64 | any | `Dependent{base,..}` | `unify(base, other)` | Symmetric with row 63 | `unification.rs:446` |
| 65 | `RefLifetime{inner:a,..}` | `RefLifetime{inner:b,..}` | `unify(a, b)` | Inner types must match; lifetime tracked separately | `unification.rs:449` |
| 66 | `RefMutLifetime{inner:a,..}` | `RefMutLifetime{inner:b,..}` | `unify(a, b)` | Mutable variant | `unification.rs:454` |
| 67 | `RefLifetime{inner,..}` | `Ref(other)` | `unify(inner, other)` | Lifetime-annotated ref ↔ plain ref | `unification.rs:457` |
| 68 | `Ref(other)` | `RefLifetime{inner,..}` | `unify(inner, other)` | Symmetric with row 67 | `unification.rs:458` |
| 69 | `RefMutLifetime{inner,..}` | `RefMut(other)` | `unify(inner, other)` | Mutable variant | `unification.rs:461` |
| 70 | `RefMut(other)` | `RefMutLifetime{inner,..}` | `unify(inner, other)` | Symmetric with row 69 | `unification.rs:462` |
| 71 | `ConstArray{element:ea,size:sa}` | `ConstArray{element:eb,size:sb}` | `Err` if sa≠sb, else `unify(ea,eb)` | Size must be equal | `unification.rs:466` |
| 72 | `Vector{element:ea,lanes:la}` | `Vector{element:eb,lanes:lb}` | `Err` if la≠lb; strict element match | No float coercion for SIMD | `unification.rs:489` |
| 73 | `Map(ka,va)` | `Map(kb,vb)` | `unify(ka,kb)?; unify(va,vb)` | Key and value must each unify | `unification.rs:524` |
| 74 | `ConstGeneric(na)` | `ConstGeneric(nb)` | `Ok(())` if na==nb else `Err` | Structural name equality | `unification.rs:529` |
| 75 | `Associated{..a}` | `Associated{..b}` | `unify bases; unify generics` (trait_name, assoc_name equal) | GAT-aware structural check | `unification.rs:541` |
| 76 | `Lifetime(na)` | `Lifetime(nb)` | `Ok(())` if na==nb else `Err` | Structural name equality | `unification.rs:562` |
| 77 | `DynTrait{..}` | any | `Ok(())` | Dynamic dispatch accepts any concrete type | `unification.rs:574` |
| 78 | any | `DynTrait{..}` | `Ok(())` | Symmetric with row 77 | `unification.rs:574` |
| 79 | `Ref(inner)` | any | `unify(inner, other)` | Auto-deref: &T unifies with T (§7) | `unification.rs:577` |
| 80 | any | `Ref(inner)` | `unify(inner, other)` | Symmetric with row 79 | `unification.rs:577` |
| 81 | `RefMut(inner)` | any | `unify(inner, other)` | Auto-deref: &mut T unifies with T (§7) | `unification.rs:580` |
| 82 | any | `RefMut(inner)` | `unify(inner, other)` | Symmetric with row 81 | `unification.rs:580` |
| — | any other pair | any other pair | `Err(TypeError::Mismatch)` | Fallthrough — hard mismatch | `unification.rs:583` |

### 3.1 Pre-match Coercion Guards (evaluated before the main match)

These are checked before the `match (&expected, &found)` block. They
short-circuit with `Ok(())` or a recursive call on success.

| Guard | Condition | Action | Phase | Code ref |
|-------|-----------|--------|-------|----------|
| `str_aliases` | Both sides are `Str`, `str`, or `Named{"Str"/"str"/"String",[]}` | `Ok(())` | 237, 267 | `unification.rs:78` |
| `str_ref` | Both sides are `&Str`/`&str`/`&mut Str`/`&mut str` | `Ok(())` | 238 | `unification.rs:88` |
| `Optional ↔ T` | Expected is `Optional(E)` and `unify(E, found)` succeeds | `Ok(())` | 276 | `unification.rs:103` |
| `T ↔ Optional` | Found is `Optional(F)` and `unify(expected, F)` succeeds | `Ok(())` | 276 | `unification.rs:108` |
| `Box<T> ↔ Box` | One side is `Named{"Box",[T]}`, other is `Named{"Box",[]}` | `Ok(())` | 279 | `unification.rs:116` |
| `Box<T> ↔ T` | One side is `Named{"Box",[T]}` — `unify(T, other)` | recursive | 268 | `unification.rs:134` |
| `&Box<T> ↔ &T` | One side is `Ref(Box<T>)` or `RefMut(Box<T>)` | recursive | 268 | `unification.rs:153` |
| Slice coercion | Both sides are slice-like (`&Vec<T>`, `Vec<T>`, `&[T]`, `[T]`) | `unify elements` | 239–240 | `unification.rs:202` |

---

## 4. Type Variable / Fresh Var Creation

### 4.1 Allocation

```rust
// crates/vais-types/src/inference/substitution.rs:173
pub(crate) fn fresh_type_var(&self) -> ResolvedType {
    let id = self.next_type_var.get();
    self.next_type_var.set(id + 1);
    ResolvedType::Var(id)
}
```

`next_type_var` is a `Cell<usize>` initialised to `0` at
`TypeChecker::new()`.
Source: `crates/vais-types/src/lib.rs:128, 223`

IDs are monotonically increasing per `TypeChecker` instance. IDs are never
reused within a single compilation unit.

### 4.2 When Fresh Vars Are Allocated

| Site | Trigger | Code ref |
|------|---------|----------|
| `resolve.rs:123` | `Type::Infer` (the `_` wildcard annotation in source) | `resolve.rs:123` |
| `checker_fn.rs:80` | Unknown generic param type during function signature registration | `checker_fn.rs:80` |
| `checker_expr/mod.rs:156` | Generic function call with unresolved type arguments | `checker_expr/mod.rs:156` |
| `checker_expr/calls.rs:84` | Method call with unresolved generic params | `checker_expr/calls.rs:84` |
| `checker_expr/calls.rs:416` | Per-method-generic-param during method resolution | `checker_expr/calls.rs:416` |
| `checker_expr/calls.rs:1682` | Builtin `push`/`pop` collection element type | `checker_expr/calls.rs:1682` |
| `checker_expr/calls.rs:1712` | Builtin `HashMap` value construction | `checker_expr/calls.rs:1712` |
| `checker_expr/collections.rs:244` | Named struct instantiation with unresolved generics | `checker_expr/collections.rs:244` |
| `checker_expr/collections.rs:565` | Array/Vec literal with unknown element type | `checker_expr/collections.rs:565` |
| `checker_expr/collections.rs:617` | Map literal with unknown key/value types | `checker_expr/collections.rs:617` |
| `inference/inference_modes.rs:151` | Empty array literal `[]` | `inference/inference_modes.rs:151` |
| `lookup.rs:77` | Trait method signature generics during lookup | `lookup.rs:77` |
| `lookup.rs:190` | Struct method signature generics during lookup | `lookup.rs:190` |
| `inference/substitution.rs:412` | Generic function instantiation — one fresh var per generic param | `inference/substitution.rs:412` |

### 4.3 Substitution Binding

When `Var(id)` appears on either side of `unify`, the algorithm binds:
```
substitutions.insert(id, t.clone())
```
after the occurs-check. Binding is permanent within the `TypeChecker`
instance lifetime.

Source: `crates/vais-types/src/inference/unification.rs:223`

### 4.4 Resolution

`apply_substitutions` resolves `Var(id)` transitively: if
`substitutions[id]` is itself a `Var(j)`, the function recurses until a
concrete type or an unbound variable is reached.

Source: `crates/vais-types/src/inference/substitution.rs:16`

---

## 5. Coercion Rules

> CRITICAL: These rules are load-bearing (Phase 158). Do NOT change without
> RFC + E2E test update. Source: CLAUDE.md §"Type Conversion Rules".

### 5.1 Allowed Implicit Coercions (in unification)

| Rule | LHS | RHS | Direction | Code ref |
|------|-----|-----|-----------|----------|
| Integer widening | any integer (`I8..U64`) | any integer (`I8..U64`) | both | `unification.rs:354` |
| Integer → float | integer | `F32` or `F64` | literal adapts | `unification.rs:357` |
| Float → integer | `F32` or `F64` | integer | reverse | `unification.rs:358` |
| Float size | `F32` | `F64` | both | `unification.rs:364` |

Integer types recognized by `is_integer_type`:
`I8, I16, I32, I64, U8, U16, U32, U64`.
`I128` and `U128` are NOT in this set (source:
`unification.rs:594–606`). `Bool` is explicitly excluded.

Float types recognized by `is_float_type`:
`F32, F64` only (source: `unification.rs:611–613`).

### 5.2 Prohibited Coercions (no unification rule exists for these)

| Pair | Status | Note |
|------|--------|------|
| `Bool` ↔ integer | PROHIBITED | `Bool` not in `is_integer_type` — CLAUDE.md |
| `Bool` ↔ float | PROHIBITED | `Bool` not in `is_float_type` — CLAUDE.md |
| `Str` ↔ integer | PROHIBITED | No rule in `unify` — CLAUDE.md |
| `Str` ↔ float | PROHIBITED | No rule in `unify` — CLAUDE.md |
| `I64` narrowing to `I32` | TECHNICALLY ALLOWED in unification (all integers unify) | But semantically narrowing is an IR concern — see §5.3 |

### 5.3 Explicit Conversion (`as`)

All type conversions outside §5.1 require an explicit `as` keyword in source:
```
x as i64
y as f64
flag as i64    # bool to integer — must be explicit
```

This policy is enforced at the language/AST level, not the unification level.
The unification engine would accept some combinations that the language policy
rejects (e.g., integer widening is intentionally permissive).

Source: CLAUDE.md §"Type Conversion Rules"

### 5.4 Phase 160-A Numeric Promotion

Integer literals default to `i64`. When a literal is used in a `f32`/`f64`
context, unification succeeds via the integer↔float rule. This enables:
```
x: f32 = 0   # literal 0 infers as i64; unifies with f32 → ok
```

Source: `crates/vais-types/src/inference/unification.rs:356`

---

## 6. Named ↔ Optional/Result Bridge (Phase 326)

### 6.1 Problem

The Vais parser produces two distinct representations for the same semantic
type:

- User writes `Option<T>` → resolves to `ResolvedType::Named { name: "Option", generics: [T] }`.
- Vais sugar `T?` or builtin dispatch → produces `ResolvedType::Optional(T)`.

Without a bridge, `unify(Named{"Option",[T]}, Optional(U))` falls through to
`Err(Mismatch)` even though the types are semantically identical.

### 6.2 Bridge Rules

```
Named{"Option", [T]}  ↔  Optional(U)    →  unify(T, U)
Named{"Result", [T,E]} ↔  Result(ok,err) →  unify(T,ok) + unify(E,err)
```

Both rules apply in both directions (two match arms each).

Source: `crates/vais-types/src/inference/unification.rs:238–257`

### 6.3 Scope

- Only applies when `name == "Option"` and `generics.len() == 1`.
- Only applies when `name == "Result"` and `generics.len() == 2`.
- `Named{"Option",[]}` (no generics) does NOT trigger this bridge.
- `Named{"Result",[T]}` (one generic) does NOT trigger this bridge.

---

## 7. Auto-deref Rules

### 7.1 `Ref(T)` Auto-deref

When neither operand is handled by any earlier arm, `Ref(inner)` unifies
with any other type by unwrapping the reference:

```
unify(Ref(inner), other)  →  unify(inner, other)
unify(other, Ref(inner))  →  unify(inner, other)
```

Source: `crates/vais-types/src/inference/unification.rs:577`

This is a catch-all at the bottom of the match. It fires only after all
more-specific arms (rows 16, 42–46, 67–68 in §3) have been checked.

### 7.2 `RefMut(T)` Auto-deref

Identical logic for mutable references:

```
unify(RefMut(inner), other)  →  unify(inner, other)
unify(other, RefMut(inner))  →  unify(inner, other)
```

Source: `crates/vais-types/src/inference/unification.rs:580`

### 7.3 `&Box<T>` Auto-deref

An additional pre-match guard (before the main `match`) handles
`Ref(Box<T>)` and `RefMut(Box<T>)` by producing `Ref(T)` / `RefMut(T)`
and recursing:

```
&Box<T>     ↔  &U      →  unify(&T, &U)
&mut Box<T> ↔  &mut U  →  unify(&mut T, &mut U)
```

Source: `crates/vais-types/src/inference/unification.rs:153`

### 7.4 What Does NOT Auto-deref

| Type | Behaviour |
|------|-----------|
| `Pointer(T)` | Does NOT auto-deref through `unify`. Handled by specific rules (rows 49–58). |
| `RefLifetime{..}` | Has dedicated arms for `RefLifetime ↔ Ref` (rows 67–68), not the catch-all. |

---

## 8. Generic Instantiation

### 8.1 Monomorphization Trigger

Each call to a generic function (or method) causes the checker to record a
`GenericInstantiation` into `TypeChecker::generic_instantiations: HashSet<GenericInstantiation>`.

This set drives monomorphization during codegen. The type alias
`GenericInstantiation` is exported from `crates/vais-types/src/types/`.

Source: `crates/vais-types/src/lib.rs:140`

### 8.2 Fresh Var Per Generic Param

At each call site, the substitution engine allocates one fresh `Var` per
generic parameter of the callee:

```rust
// crates/vais-types/src/inference/substitution.rs:412
.map(|param| (param.clone(), self.fresh_type_var()))
```

This creates a one-to-one mapping: `generic_name → Var(id)`.

### 8.3 Unification Resolves Vars

After fresh vars are created, unification of call-site argument types with
the callee's parameter types (substituted with the fresh vars) binds each
`Var(id)` to a concrete type. `apply_substitutions` then resolves the vars
to produce the concrete instantiation.

### 8.4 Where-clause Resolution

Generic bounds (e.g., `T: Display`) are stored in
`TypeChecker::current_generic_bounds: HashMap<String, Vec<String>>`.

During method lookup, bounds are checked via
`TypeChecker::trait_impls: Vec<TraitImpl>`.

Source: `crates/vais-types/src/lib.rs:119–120, 107`

### 8.5 Pending Method Instantiations

When a method call is type-checked inside a function body, the type arguments
may still contain unresolved `Var` nodes (because the return type of the
enclosing function has not yet been unified). These are stored in:

```
TypeChecker::pending_method_instantiations: Vec<(String, String, Vec<ResolvedType>)>
```

They are drained at the end of `check_function` after full body/return
unification, at which point the vars have resolved to concrete types.

Source: `crates/vais-types/src/lib.rs:197–200`

### 8.6 Bidirectional Checking and Generics

`CheckMode::Check(expected)` propagates the expected type top-down into
lambda parameter types and array element types.

For lambda expressions, if the expected type is `Fn{params, ret}`:
- Parameter types are taken from the expected `params` list.
- Body is checked against the expected `ret`.

Source: `crates/vais-types/src/inference/inference_modes.rs:74`

---

## 9. Known Gaps / TODO

These gaps are tracked as future work items. They do NOT represent bugs in
the current system — they are intentional simplifications with known
edge-case fallout.

### Phase 2.9 — Cross-file Impl Dispatch — **DECISION: Option (a) accepted**

**Problem**: When a struct `S Parser { ... }` is in `parser.vais` and
`X Parser { F parse_select(self) ... }` is in `parser_select.vais`,
compiling `parser.vais` standalone does not see `parse_select` because
module load order is source-file-driven.

**Decision (2026-04-19, Phase 2.9)**: Keep current behavior — the impl
block and its type declaration must co-exist in the same compilation
unit (either same file, or the importing file's transitive closure).

Considered alternatives:

| Option | Summary | Verdict |
|--------|---------|---------|
| (a) Status quo + consolidation guidance | Users split impls at their own risk; document co-location rule | **chosen** |
| (b) Introduce `#[extend]` annotation | New attribute to signal "this impl block extends a type declared elsewhere"; TC delays resolution | rejected — new surface for little benefit given no current caller needs it |
| (c) Allow benign circular imports | Detect A imports B imports A, but only error if the cycle has data-dependency, not just impl discovery | rejected — complicates `test_circular_import_detection` invariant and makes dependency graph ambiguous |

**Rationale for (a)**:
1. The selfhost compiler (50,000+ LOC) already co-locates impl blocks.
2. std/ follows the same convention.
3. No vaisdb file currently splits impls across files.
4. `test_circular_import_detection` (crates/vaisc/tests/e2e/modules_system.rs:303)
   codifies the invariant and currently passes.
5. Option (b) would require a new AST node, codegen change, and TC two-pass
   — too much work to solve a problem nobody has.

**Workaround for users who hit this**: move the `X Type { … }` block into
the same file as the `S Type { … }` declaration, or use a single file that
`U`-imports both (the importing file's TC sees both declarations).

**Future Phase**: if vais ecosystem expansion forces impl splits, revisit
with RFC. Until then, document the co-location rule in CLAUDE.md and
LANGUAGE_SPEC.md.

**Test invariant**: `test_circular_import_detection` (modules_system.rs:303)
asserts that `A imports B` + `B imports A` produces a `circular` / `cycle`
error. This is the load-bearing contract of option (a) — breaking it would
re-open the cross-file-impl can of worms. Do NOT weaken this test.

### Phase 2.10 — Option Match-Arm Constructor Re-wrapping (partially investigated)

**Reproducer** (confirmed 2026-04-19, baseline commit `dc0069f4`):

```vais
S Role { role_id: u64, }

F simpler(opt: Option<Role>) -> Option<u64> {
    M opt {
        Some(r) => Some(r.role_id),
        None => None,
    }
}
```

**Error**: `E001: Type mismatch. expected u64, found Role`, pointed at
the `Option<u64>` return type annotation.

**Root cause** (from `crates/vais-types/src/checker_expr/calls.rs:55-87`):
When the `Some(r.role_id)` constructor call is type-checked, the code
unifies the argument type (`u64`) against the variant's field type —
but that field type is still the raw generic placeholder `Generic("T")`
from the enum definition. Then the constructor returns
`ResolvedType::Named { name: "Option", generics: [fresh_type_var()] }`
where the fresh var is **not** connected to the unification that just
happened. So `Some(u64 arg)` yields `Option<?N>` with `?N` unconstrained,
and downstream unification with the match-arm `None => None` (whose type
comes from the scrutinee `Option<Role>`) binds `?N := Role`.

**Naive fix attempted**: instantiate fresh vars first, substitute them
into the variant's field types before unifying, use those same fresh
vars as the return's generics. This **regressed vaisdb by 1 file** — so
some existing code relies on the old disconnected-fresh-var behavior
(possibly via the Named↔Optional bridge at `unification.rs:247`).

**Status**: deferred. Requires a focused session with:
1. Identify which vaisdb file regressed and why.
2. Decide whether the fix should be in the constructor call (calls.rs)
   or in the Named↔Optional bridge (unification.rs).
3. Protect the fix with both the reproducer above AND the regressed
   vaisdb case.

**Also related** (deeper case — not yet reproduced):
`Option<&T>` with `Some(r)` binds `r: &T`. For nested `Option<&&T>`
the auto-deref may fail in 2+ deref levels. Separate from the core bug
above; may or may not share a fix.

**Role.vais get_role_id** — concrete impact:
`vaisdb/src/security/role.vais:427` is the canonical example. Status:
**currently fails codegen** per baseline measurement, already counted
in the 176/261 floor.

### Phase 2.11 — Method Inference Dispersion

**Problem**: When a generic struct method (e.g., `Vec<T>::push`) is called
inside a nested closure that is itself passed to another generic function,
the fresh `Var` allocated for `T` at the method call site can fail to
propagate back to the closure's inferred type.

**Symptom**: Compiler emits `?0` (unresolved type variable) in error
messages for closures that interact with generic collection methods.

**Planned fix**: Extend `pending_method_instantiations` collection to cover
closures, not just top-level function bodies.

---

## 10. How to Extend

### 10.1 Adding a New `ResolvedType` Variant

1. Add the variant to `crates/vais-types/src/types/resolved.rs` inside the
   `ResolvedType` enum.

2. Add a `Display` implementation arm in the `impl std::fmt::Display for ResolvedType`
   block in the same file.

3. Add an `occurs_in` arm in `crates/vais-types/src/inference/unification.rs`
   inside the `occurs_in` function. If the new type wraps inner types, recurse
   into them. If it is a leaf (no inner types), add it to the `_ => false` arm
   or add an explicit arm that returns `false`.

4. Add a `contains_var` arm in the same file for the same reasons.

5. Add `apply_substitutions` arms in
   `crates/vais-types/src/inference/substitution.rs` to recursively apply
   substitutions to all inner `ResolvedType` fields.

6. Add one or more `match` arms in the `unify` function
   (`crates/vais-types/src/inference/unification.rs`) specifying how the new
   variant unifies with itself and/or with other variants.

7. Update this document (§1 table) and the unification table (§3).

8. Add a unit test in `crates/vais-types/src/types/resolved.rs` (§tests) and
   in `crates/vais-types/src/tests.rs` or a new test file.

### 10.2 Adding a New Coercion Rule

> WARNING: Adding coercions is a breaking change to the type system. Phase 158
> established strict Rust-style coercion rules. Any change requires:
> (a) a written RFC, (b) updates to `docs/rfcs/`, and (c) E2E protection
> tests in `crates/vaisc/tests/e2e/`.
>
> Source: CLAUDE.md §"Type Conversion Rules"

The yoyo pattern to avoid: `unification.rs` previously had `Bool`, `Str↔I64`,
and `Float↔Int` coercions added and removed 5 times across phases. Phase 158
was the definitive fix. Do not re-add these.

Steps for a legitimate new coercion:

1. Verify the coercion is not already prohibited by CLAUDE.md §"Type
   Conversion Rules".

2. Write an RFC in `docs/rfcs/` describing: motivation, user-visible behaviour,
   impact on existing code, and which existing tests would change.

3. Add the coercion as a pre-match guard (before the `match (&expected, &found)`
   block) in `crates/vais-types/src/inference/unification.rs`. Document the
   phase number and reasoning in a comment.

4. Add an E2E test that:
   - Verifies the coercion succeeds for the intended case.
   - Verifies the inverse does NOT coerce (if directional).

5. Update the table in §3 of this document with the new row, including the
   code reference.

### 10.3 Adding a New Named↔Builtin Bridge (like Phase 326)

Follow the same pattern as the `Named{"Option"} ↔ Optional` bridge:

1. Add two `match` arms (both directions) before or after the existing bridge
   arms at `unification.rs:245`.

2. Guard on both `name` equality and `generics.len()` equality.

3. Recursively unify the generic arguments positionally.

4. Document in §6 of this file.

---

*Document created: Phase 2.8. Authoritative source of truth for the Vais
type unification engine. Update this file whenever the unification rules
change.*
