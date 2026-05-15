//! Type unification logic for the Vais type system.
//!
//! Contains the core unification algorithm (occurs-check, unify),
//! integer type checking, and type variable detection.

use crate::types::{ResolvedType, TypeError, TypeResult};
use crate::TypeChecker;

impl TypeChecker {
    pub(crate) fn allow_legacy_a4_03_auto_deref() -> bool {
        std::env::var("VAIS_REJECT_A4_03").as_deref() == Ok("0")
    }

    /// Check if a type variable `id` occurs anywhere inside `ty`.
    /// Prevents creating cyclic substitutions (e.g., T0 -> `Vec<T0>`) which would
    /// cause `apply_substitutions` to recurse infinitely.
    pub(crate) fn occurs_in(id: usize, ty: &ResolvedType) -> bool {
        match ty {
            ResolvedType::Var(other_id) => *other_id == id,
            ResolvedType::Array(inner)
            | ResolvedType::Optional(inner)
            | ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Slice(inner)
            | ResolvedType::SliceMut(inner)
            | ResolvedType::Pointer(inner)
            | ResolvedType::Range(inner)
            | ResolvedType::Future(inner)
            | ResolvedType::Linear(inner)
            | ResolvedType::Affine(inner) => Self::occurs_in(id, inner),
            ResolvedType::Result(ok, err) | ResolvedType::Map(ok, err) => {
                Self::occurs_in(id, ok) || Self::occurs_in(id, err)
            }
            ResolvedType::Tuple(types) => types.iter().any(|t| Self::occurs_in(id, t)),
            ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
                params.iter().any(|t| Self::occurs_in(id, t)) || Self::occurs_in(id, ret)
            }
            ResolvedType::Named { generics, .. } | ResolvedType::DynTrait { generics, .. } => {
                generics.iter().any(|t| Self::occurs_in(id, t))
            }
            ResolvedType::RefLifetime { inner, .. }
            | ResolvedType::RefMutLifetime { inner, .. } => Self::occurs_in(id, inner),
            ResolvedType::Dependent { base, .. } => Self::occurs_in(id, base),
            ResolvedType::ConstArray { element, .. } | ResolvedType::Vector { element, .. } => {
                Self::occurs_in(id, element)
            }
            ResolvedType::Associated { base, generics, .. } => {
                Self::occurs_in(id, base) || generics.iter().any(|t| Self::occurs_in(id, t))
            }
            // Leaf types: no type variables inside
            _ => false,
        }
    }

    /// Unify two types
    #[inline]
    pub(crate) fn unify(
        &mut self,
        expected: &ResolvedType,
        found: &ResolvedType,
    ) -> TypeResult<()> {
        // Fast path: pointer identity means same type object, skip everything
        if std::ptr::eq(expected, found) {
            return Ok(());
        }

        let expected = self.apply_substitutions(expected);
        let found = self.apply_substitutions(found);

        if expected == found {
            return Ok(());
        }

        // Phase 237: Str / str alias coercion. `T Str = str` should make
        // them unify in either direction. Without this, vaisdb's `Str` type
        // alias produces 'expected str, found Str' E001 in many call sites.
        // Phase 238: extend to also accept &Str ↔ &str.
        // Phase 267: also include "String" (Vais owned-string struct in stdlib)
        // — vaisdb returns String from .to_string()/.from() and passes to
        // functions expecting str/Str. Distinct ABI but interchangeable in
        // type checking; codegen handles the conversion.
        let str_aliases = |t: &ResolvedType| -> bool {
            matches!(t, ResolvedType::Str)
                || matches!(t, ResolvedType::Named { name, generics }
                    if (name == "Str" || name == "str" || name == "String")
                        && generics.is_empty())
        };
        if str_aliases(&expected) && str_aliases(&found) {
            return Ok(());
        }
        // &Str ↔ &str ↔ &mut Str ↔ &mut str
        let str_ref = |t: &ResolvedType| -> bool {
            matches!(t,
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner)
                    if str_aliases(inner)
            )
        };
        if str_ref(&expected) && str_ref(&found) {
            return Ok(());
        }
        let ref_to_str_alias = |t: &ResolvedType| -> bool {
            matches!(t,
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner)
                    if str_aliases(inner)
            )
        };
        if (str_aliases(&expected) && ref_to_str_alias(&found))
            || (ref_to_str_alias(&expected) && str_aliases(&found))
        {
            return Ok(());
        }

        // Phase 276: Optional<T> ↔ T coercion (one direction only — accept
        // bare T where Option<T> expected, NOT vice versa). This is a
        // permissive coercion since vaisdb often unwraps and passes the
        // inner directly. Pattern: f(opt.unwrap_or(default)) where param
        // is Option<T>, or pass owned T into Option<T> param.
        if let ResolvedType::Optional(e_inner) = &expected {
            if self.unify(e_inner, &found).is_ok() {
                return Ok(());
            }
        }
        if let ResolvedType::Optional(f_inner) = &found {
            if self.unify(&expected, f_inner).is_ok() {
                return Ok(());
            }
        }

        // Phase 279: Box<T> ↔ Box (no generics) — degenerate form from incomplete
        // inference. Treat as equal regardless of generics.
        let is_raw_box = |t: &ResolvedType| -> bool {
            matches!(t, ResolvedType::Named { name, generics }
                if name == "Box" && generics.is_empty())
        };
        let is_boxed = |t: &ResolvedType| -> bool {
            matches!(t, ResolvedType::Named { name, .. } if name == "Box")
        };
        if is_boxed(&expected) && is_raw_box(&found) {
            return Ok(());
        }
        if is_boxed(&found) && is_raw_box(&expected) {
            return Ok(());
        }

        // Phase 268: legacy Box<T> ↔ T coercion.
        //
        // A4-13: direct Box<T> ↔ T at call sites is now rejected by default
        // because it silently unwraps a user-facing value without an explicit
        // deref. The narrower &Box<T> ↔ &T / &mut Box<T> ↔ &mut T path below
        // stays enabled because downstream recursive AST/codegen paths depend
        // on reference-level projection.
        let unbox = |t: &ResolvedType| -> Option<ResolvedType> {
            if let ResolvedType::Named { name, generics } = t {
                if name == "Box" && generics.len() == 1 {
                    return Some(generics[0].clone());
                }
            }
            None
        };
        if let Some(e_inner) = unbox(&expected) {
            if self.unify(&e_inner, &found).is_ok() {
                if std::env::var("VAIS_REJECT_A4_13").as_deref() == Ok("0") {
                    return Ok(());
                }
                return Err(crate::TypeError::Mismatch {
                    expected: expected.to_string(),
                    found: found.to_string(),
                    span: None,
                });
            }
        }
        if let Some(f_inner) = unbox(&found) {
            if self.unify(&expected, &f_inner).is_ok() {
                if std::env::var("VAIS_REJECT_A4_13").as_deref() == Ok("0") {
                    return Ok(());
                }
                return Err(crate::TypeError::Mismatch {
                    expected: expected.to_string(),
                    found: found.to_string(),
                    span: None,
                });
            }
        }
        // &Box<T> ↔ &T / &mut Box<T> ↔ &mut T
        let unbox_ref = |t: &ResolvedType| -> Option<ResolvedType> {
            match t {
                ResolvedType::Ref(inner) => unbox(inner).map(|u| ResolvedType::Ref(Box::new(u))),
                ResolvedType::RefMut(inner) => {
                    unbox(inner).map(|u| ResolvedType::RefMut(Box::new(u)))
                }
                _ => None,
            }
        };
        if let Some(e_inner) = unbox_ref(&expected) {
            if self.unify(&e_inner, &found).is_ok() {
                return Ok(());
            }
        }
        if let Some(f_inner) = unbox_ref(&found) {
            if self.unify(&expected, &f_inner).is_ok() {
                return Ok(());
            }
        }

        // Phase 239+240: slice-like element unification.
        //
        // A4-14: Vec<T> / &Vec<T> ↔ &[T] is a specified slice coercion.
        // Codegen must materialize a real slice fat value before the call
        // boundary; passing the Vec storage layout directly is forbidden. Set
        // VAIS_REJECT_A4_14=1 to force a strict migration audit.
        let is_vec_like = |t: &ResolvedType| -> bool {
            match t {
                ResolvedType::Named { name, generics } => name == "Vec" && generics.len() == 1,
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                    matches!(inner.as_ref(), ResolvedType::Named { name, generics } if name == "Vec" && generics.len() == 1)
                }
                _ => false,
            }
        };
        let is_slice_like = |t: &ResolvedType| -> bool {
            match t {
                ResolvedType::Slice(_) | ResolvedType::SliceMut(_) => true,
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                    matches!(
                        inner.as_ref(),
                        ResolvedType::Slice(_) | ResolvedType::SliceMut(_)
                    )
                }
                _ => false,
            }
        };
        if ((is_vec_like(&expected) && is_slice_like(&found))
            || (is_slice_like(&expected) && is_vec_like(&found)))
            && std::env::var("VAIS_REJECT_A4_14").as_deref() == Ok("1")
        {
            return Err(crate::TypeError::Mismatch {
                expected: expected.to_string(),
                found: found.to_string(),
                span: None,
            });
        }

        let extract_slice_elem = |t: &ResolvedType| -> Option<ResolvedType> {
            match t {
                ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => match inner.as_ref() {
                    ResolvedType::Named { name, generics }
                        if name == "Vec" && generics.len() == 1 =>
                    {
                        Some(generics[0].clone())
                    }
                    ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => {
                        Some((**elem).clone())
                    }
                    _ => None,
                },
                ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) => Some((**elem).clone()),
                ResolvedType::Named { name, generics } if name == "Vec" && generics.len() == 1 => {
                    Some(generics[0].clone())
                }
                _ => None,
            }
        };
        if let (Some(e_elem), Some(f_elem)) =
            (extract_slice_elem(&expected), extract_slice_elem(&found))
        {
            // Both are slice-like — unify element types.
            return self.unify(&e_elem, &f_elem);
        }

        match (&expected, &found) {
            // Type variables can unify with anything
            (ResolvedType::Var(id), t) | (t, ResolvedType::Var(id)) => {
                // Self-referential check: Var(id) unifying with itself is trivially Ok
                if let ResolvedType::Var(other_id) = t {
                    if *other_id == *id {
                        return Ok(());
                    }
                }
                // Occurs-check: prevent cyclic substitutions (e.g., T0 -> Option<T0>)
                // which would cause apply_substitutions to recurse infinitely
                if Self::occurs_in(*id, t) {
                    return Ok(());
                }
                self.substitutions.insert(*id, t.clone());
                Ok(())
            }
            // Unknown type unifies with anything (used as placeholder)
            (ResolvedType::Unknown, _) | (_, ResolvedType::Unknown) => Ok(()),
            // Never type unifies with any type (represents non-returning expressions like return, break)
            (ResolvedType::Never, _) | (_, ResolvedType::Never) => Ok(()),
            // Generic type parameters match with any type (type erasure)
            (ResolvedType::Generic(_), _) | (_, ResolvedType::Generic(_)) => Ok(()),
            (ResolvedType::Array(a), ResolvedType::Array(b)) => self.unify(a, b),
            (ResolvedType::Optional(a), ResolvedType::Optional(b)) => self.unify(a, b),
            (ResolvedType::Result(a_ok, a_err), ResolvedType::Result(b_ok, b_err)) => {
                self.unify(a_ok, b_ok)?;
                self.unify(a_err, b_err)
            }
            // Phase 326: bridge Named{"Option", [T]} ↔ Optional(T) and
            // Named{"Result", [T, E]} ↔ Result(T, E). User-written `Option<T>`
            // / `Result<T, E>` resolve to the Named form (stdlib enum path),
            // while the sugar `T?` and builtin dispatches produce the
            // Optional / Result variants. Without this bridge, the two
            // representations are treated as distinct by unify even though
            // they're semantically identical.
            (ResolvedType::Named { name, generics }, ResolvedType::Optional(inner))
            | (ResolvedType::Optional(inner), ResolvedType::Named { name, generics })
                if name == "Option" && generics.len() == 1 =>
            {
                self.unify(&generics[0], inner)
            }
            (ResolvedType::Named { name, generics }, ResolvedType::Result(ok, err))
            | (ResolvedType::Result(ok, err), ResolvedType::Named { name, generics })
                if name == "Result" && generics.len() == 2 =>
            {
                self.unify(&generics[0], ok)?;
                self.unify(&generics[1], err)
            }
            (ResolvedType::Ref(a), ResolvedType::Ref(b)) => match (a.as_ref(), b.as_ref()) {
                // Treat redundant reborrows as reference-level normalization:
                // &T ↔ &&T should not route through the A4-03 value
                // auto-deref fallback below.
                (ResolvedType::Ref(inner), other) | (other, ResolvedType::Ref(inner))
                    if inner.as_ref() == other =>
                {
                    Ok(())
                }
                _ => self.unify(a, b),
            },
            (ResolvedType::RefMut(a), ResolvedType::RefMut(b)) => self.unify(a, b),
            // Reference mutability is a separate language audit surface from
            // A4-03. Keep it out of the Ref↔value strict-mode measurement.
            (ResolvedType::Ref(a), ResolvedType::RefMut(b))
            | (ResolvedType::RefMut(b), ResolvedType::Ref(a)) => self.unify(a, b),
            (ResolvedType::Slice(a), ResolvedType::Slice(b)) => self.unify(a, b),
            (ResolvedType::SliceMut(a), ResolvedType::SliceMut(b)) => self.unify(a, b),
            (ResolvedType::Pointer(a), ResolvedType::Pointer(b)) => self.unify(a, b),
            (ResolvedType::Range(a), ResolvedType::Range(b)) => self.unify(a, b),
            (ResolvedType::Future(a), ResolvedType::Future(b)) => self.unify(a, b),
            (ResolvedType::Tuple(a), ResolvedType::Tuple(b)) if a.len() == b.len() => {
                for (ta, tb) in a.iter().zip(b.iter()) {
                    self.unify(ta, tb)?;
                }
                Ok(())
            }
            (
                ResolvedType::Fn {
                    params: pa,
                    ret: ra,
                    ..
                },
                ResolvedType::Fn {
                    params: pb,
                    ret: rb,
                    ..
                },
            ) if pa.len() == pb.len() => {
                for (ta, tb) in pa.iter().zip(pb.iter()) {
                    self.unify(ta, tb)?;
                }
                self.unify(ra, rb)
            }
            // Allow Fn to unify with FnPtr (function values can be used as function pointers)
            (
                ResolvedType::Fn {
                    params: pa,
                    ret: ra,
                    ..
                },
                ResolvedType::FnPtr {
                    params: pb,
                    ret: rb,
                    ..
                },
            )
            | (
                ResolvedType::FnPtr {
                    params: pa,
                    ret: ra,
                    ..
                },
                ResolvedType::Fn {
                    params: pb,
                    ret: rb,
                    ..
                },
            ) if pa.len() == pb.len() => {
                for (ta, tb) in pa.iter().zip(pb.iter()) {
                    self.unify(ta, tb)?;
                }
                self.unify(ra, rb)
            }
            // FnPtr to FnPtr unification
            (
                ResolvedType::FnPtr {
                    params: pa,
                    ret: ra,
                    ..
                },
                ResolvedType::FnPtr {
                    params: pb,
                    ret: rb,
                    ..
                },
            ) if pa.len() == pb.len() => {
                for (ta, tb) in pa.iter().zip(pb.iter()) {
                    self.unify(ta, tb)?;
                }
                self.unify(ra, rb)
            }
            // Named types with generics
            (
                ResolvedType::Named {
                    name: na,
                    generics: ga,
                },
                ResolvedType::Named {
                    name: nb,
                    generics: gb,
                },
            ) if na == nb && ga.len() == gb.len() => {
                for (ta, tb) in ga.iter().zip(gb.iter()) {
                    self.unify(ta, tb)?;
                }
                Ok(())
            }
            // Allow implicit integer type unification (widening within same signedness family).
            // Vais integer literals default to i64, so this enables `a:i8 = 1` patterns.
            // A4-07 (Master Plan v16 §A4 + Step 13 stage 0): opt-in strict mode
            // via VAIS_REJECT_A4_07=1. Default preserves the legacy silent
            // coercion so the baseline does not move.
            (a, b) if Self::is_integer_type(a) && Self::is_integer_type(b) => {
                if std::env::var("VAIS_REJECT_A4_07").as_deref() == Ok("1") {
                    Err(crate::TypeError::Mismatch {
                        expected: expected.to_string(),
                        found: found.to_string(),
                        span: None,
                    })
                } else {
                    Ok(())
                }
            }
            // Allow int ↔ float unification (Phase 160-A numeric promotion).
            // Integer literals like `0` adapt to f32/f64 context. Enables `x: f32 = 0`.
            (a, b)
                if (Self::is_integer_type(a) && Self::is_float_type(b))
                    || (Self::is_float_type(a) && Self::is_integer_type(b)) =>
            {
                Ok(())
            }
            // Allow f32 ↔ f64 unification (float literal inference).
            (ResolvedType::F32, ResolvedType::F64) | (ResolvedType::F64, ResolvedType::F32) => {
                Ok(())
            }
            // A4-01 (Master Plan v16 §A4 + Step 13 stage 1): strict default.
            // Empirical baseline footprint = 0 std + 0 vaisdb after the
            // compiler/std/http.vais migration (replaced `LW handler != 0
            // { ... } ! { ... }` if-with-else statement form with a ternary
            // expression so the response type propagates through inference).
            // Set VAIS_REJECT_A4_01=0 to restore the legacy silent coercion.
            (ResolvedType::Unit, ResolvedType::I64) | (ResolvedType::I64, ResolvedType::Unit) => {
                if std::env::var("VAIS_REJECT_A4_01").as_deref() == Ok("0") {
                    Ok(())
                } else {
                    Err(crate::TypeError::Mismatch {
                        expected: ResolvedType::I64.to_string(),
                        found: ResolvedType::Unit.to_string(),
                        span: None,
                    })
                }
            }
            // Allow Result/Optional ↔ unit (implicit Ok(()) wrapping)
            (ResolvedType::Result(_, _), ResolvedType::Unit)
            | (ResolvedType::Unit, ResolvedType::Result(_, _))
            | (ResolvedType::Optional(_), ResolvedType::Unit)
            | (ResolvedType::Unit, ResolvedType::Optional(_)) => Ok(()),
            // Vec<T> ↔ Slice/Ref — Vec<u8> and &[u8] are compatible
            (ResolvedType::Named { name, generics }, ResolvedType::Slice(elem))
            | (ResolvedType::Slice(elem), ResolvedType::Named { name, generics })
                if name == "Vec" && !generics.is_empty() =>
            {
                self.unify(&generics[0], elem)
            }
            (ResolvedType::Named { name, generics }, ResolvedType::Ref(inner))
            | (ResolvedType::Ref(inner), ResolvedType::Named { name, generics })
                if name == "Vec" && !generics.is_empty() =>
            {
                if let ResolvedType::Slice(elem) = inner.as_ref() {
                    self.unify(&generics[0], elem)
                } else {
                    // A4-08 (Master Plan v16 §A4 + Step 13 stage 1): strict default
                    // — Vec<T> ↔ &T silent permissive coercion is now rejected by
                    // default. Empirical baseline footprint = 0 std files (verified
                    // 2026-05-04). Set VAIS_REJECT_A4_08=0 to restore the legacy
                    // silent coercion if a previously-unmeasured downstream
                    // consumer breaks; document any such case in STEP7_FINDINGS.
                    if std::env::var("VAIS_REJECT_A4_08").as_deref() == Ok("0") {
                        Ok(()) // Permissive: allow Vec ↔ &T (legacy)
                    } else {
                        Err(crate::TypeError::Mismatch {
                            expected: format!("Vec<T>"),
                            found: format!("&T"),
                            span: None,
                        })
                    }
                }
            }
            // Ref(Vec<T>) ↔ Slice(T) auto-coercion (Phase 163).
            // &Vec<&[u8]> and &[&[u8]] are compatible — Rust-style auto-deref from &Vec<T> to &[T].
            (ResolvedType::Ref(inner), ResolvedType::Slice(elem))
            | (ResolvedType::Slice(elem), ResolvedType::Ref(inner))
                if matches!(inner.as_ref(), ResolvedType::Named { name, generics, .. } if name == "Vec" && !generics.is_empty()) =>
            {
                if let ResolvedType::Named { generics, .. } = inner.as_ref() {
                    self.unify(&generics[0], elem)
                } else {
                    Ok(())
                }
            }
            // Ref(Pointer<T>) ↔ Slice(T) auto-coercion for pointer-backed array literals.
            // This is a narrow slice coercion, not a general &T ↔ T conversion.
            (ResolvedType::Ref(inner), ResolvedType::Slice(elem))
            | (ResolvedType::Slice(elem), ResolvedType::Ref(inner))
                if matches!(inner.as_ref(), ResolvedType::Pointer(_)) =>
            {
                if let ResolvedType::Pointer(pointer_elem) = inner.as_ref() {
                    self.unify(pointer_elem, elem)
                } else {
                    Ok(())
                }
            }
            // RefMut(Vec<T>) ↔ SliceMut(T) auto-coercion (Phase 163).
            (ResolvedType::RefMut(inner), ResolvedType::SliceMut(elem))
            | (ResolvedType::SliceMut(elem), ResolvedType::RefMut(inner))
                if matches!(inner.as_ref(), ResolvedType::Named { name, generics, .. } if name == "Vec" && !generics.is_empty()) =>
            {
                if let ResolvedType::Named { generics, .. } = inner.as_ref() {
                    self.unify(&generics[0], elem)
                } else {
                    Ok(())
                }
            }
            // Pointer <-> i64 implicit unification.
            // Vais represents all pointers as i64 at the IR level (no opaque pointer distinction).
            // This allows builtins like vec_new() -> i64 and malloc() -> i64 to unify with *T
            // parameters, and swap(ptr, i, j) to accept either pointer or i64 arguments.
            // Scope: unification only — does not enable arbitrary pointer arithmetic in user code.
            // A4-02 (Master Plan v16 §A4 + Step 13 stage 1): strict default.
            // Empirical baseline footprint = 0 std + 0 vaisdb after std/gpu.vais
            // migration (2026-05-04). Set VAIS_REJECT_A4_02=0 to restore the
            // legacy silent coercion.
            (ResolvedType::Pointer(_), ResolvedType::I64)
            | (ResolvedType::I64, ResolvedType::Pointer(_)) => {
                if std::env::var("VAIS_REJECT_A4_02").as_deref() == Ok("0") {
                    Ok(())
                } else {
                    Err(crate::TypeError::Mismatch {
                        expected: format!("Pointer<T>"),
                        found: format!("i64"),
                        span: None,
                    })
                }
            }
            // Pointer<T> ↔ Slice<T> / SliceMut<T> auto-coercion (Phase 162).
            // *u8 and &[u8] are compatible in systems code — both represent byte buffers.
            // Unifies element types to maintain generic consistency.
            // A4-04 (Master Plan v16 §A4 + Step 13 stage 1): strict default.
            // Empirical baseline footprint = 0 std + 0 vaisdb sample (2026-05-04).
            // Set VAIS_REJECT_A4_04=0 to restore legacy silent coercion.
            (ResolvedType::Pointer(p), ResolvedType::Slice(s))
            | (ResolvedType::Slice(s), ResolvedType::Pointer(p))
            | (ResolvedType::Pointer(p), ResolvedType::SliceMut(s))
            | (ResolvedType::SliceMut(s), ResolvedType::Pointer(p)) => {
                if std::env::var("VAIS_REJECT_A4_04").as_deref() == Ok("0") {
                    self.unify(p, s)
                } else {
                    Err(crate::TypeError::Mismatch {
                        expected: format!("Pointer<T>"),
                        found: format!("Slice<T>"),
                        span: None,
                    })
                }
            }
            // Array/ConstArray ↔ Pointer auto-coercion (Phase 162).
            // [u64] / [u64; N] and *i64 are compatible (C-style array decay to pointer).
            // A4-05 (Master Plan v16 §A4 + Step 13 stage 0): opt-in strict mode.
            // Stage-1 attempt 2026-05-04 found ONE vaisdb dependency
            // (lang/packages/vaisdb/src/vector/hnsw/cow.vais), so default
            // remains legacy until that site is migrated. Set
            // VAIS_REJECT_A4_05=1 to enable strict; default-mode INTEGRITY OK.
            (ResolvedType::ConstArray { element, .. }, ResolvedType::Pointer(p))
            | (ResolvedType::Pointer(p), ResolvedType::ConstArray { element, .. })
            | (ResolvedType::Array(element), ResolvedType::Pointer(p))
            | (ResolvedType::Pointer(p), ResolvedType::Array(element)) => {
                if std::env::var("VAIS_REJECT_A4_05").as_deref() == Ok("1") {
                    Err(crate::TypeError::Mismatch {
                        expected: format!("Array<T>"),
                        found: format!("Pointer<T>"),
                        span: None,
                    })
                } else {
                    self.unify(element, p)
                }
            }
            // Linear type: unwrap and unify with inner type
            (ResolvedType::Linear(inner), other) | (other, ResolvedType::Linear(inner)) => {
                self.unify(inner, other)
            }
            // Affine type: unwrap and unify with inner type
            (ResolvedType::Affine(inner), other) | (other, ResolvedType::Affine(inner)) => {
                self.unify(inner, other)
            }
            // Dependent type: unify the base type only (predicate is checked separately)
            (ResolvedType::Dependent { base, .. }, other)
            | (other, ResolvedType::Dependent { base, .. }) => self.unify(base, other),
            // Lifetime references: unify inner types (lifetime is tracked separately)
            (
                ResolvedType::RefLifetime { inner: a, .. },
                ResolvedType::RefLifetime { inner: b, .. },
            ) => self.unify(a, b),
            (
                ResolvedType::RefMutLifetime { inner: a, .. },
                ResolvedType::RefMutLifetime { inner: b, .. },
            ) => self.unify(a, b),
            // Allow ref with lifetime to unify with plain ref
            // A4-09 (Master Plan v16 §A4 + Step 13 stage 0): opt-in strict mode
            // via VAIS_REJECT_A4_09=1. Default preserves the legacy silent
            // coercion so the baseline does not move.
            (ResolvedType::RefLifetime { inner, .. }, ResolvedType::Ref(other))
            | (ResolvedType::Ref(other), ResolvedType::RefLifetime { inner, .. }) => {
                if std::env::var("VAIS_REJECT_A4_09").as_deref() == Ok("0") {
                    self.unify(inner, other)
                } else {
                    Err(crate::TypeError::Mismatch {
                        expected: format!("&'a T"),
                        found: format!("&T"),
                        span: None,
                    })
                }
            }
            (ResolvedType::RefMutLifetime { inner, .. }, ResolvedType::RefMut(other))
            | (ResolvedType::RefMut(other), ResolvedType::RefMutLifetime { inner, .. }) => {
                if std::env::var("VAIS_REJECT_A4_09").as_deref() == Ok("0") {
                    self.unify(inner, other)
                } else {
                    Err(crate::TypeError::Mismatch {
                        expected: format!("&'a mut T"),
                        found: format!("&mut T"),
                        span: None,
                    })
                }
            }
            // ConstArray: element type unification + size equality
            (
                ResolvedType::ConstArray {
                    element: ea,
                    size: sa,
                },
                ResolvedType::ConstArray {
                    element: eb,
                    size: sb,
                },
            ) => {
                if sa != sb {
                    return Err(TypeError::Mismatch {
                        expected: expected.to_string(),
                        found: found.to_string(),
                        span: None,
                    });
                }
                self.unify(ea, eb)
            }
            // Vector: element type unification + lanes equality
            // SIMD vectors require exact element type match (no implicit float coercion)
            // because <4 x f32> (128-bit) is fundamentally different from <4 x f64> (256-bit).
            (
                ResolvedType::Vector {
                    element: ea,
                    lanes: la,
                },
                ResolvedType::Vector {
                    element: eb,
                    lanes: lb,
                },
            ) => {
                if la != lb {
                    return Err(TypeError::Mismatch {
                        expected: expected.to_string(),
                        found: found.to_string(),
                        span: None,
                    });
                }
                // Strict element comparison: resolve substitutions and compare directly
                let ea_resolved = self.apply_substitutions(ea);
                let eb_resolved = self.apply_substitutions(eb);
                if ea_resolved == eb_resolved {
                    Ok(())
                } else if let (ResolvedType::Var(_), _) | (_, ResolvedType::Var(_)) =
                    (&ea_resolved, &eb_resolved)
                {
                    // Allow type variable unification (inference)
                    self.unify(ea, eb)
                } else {
                    Err(TypeError::Mismatch {
                        expected: expected.to_string(),
                        found: found.to_string(),
                        span: None,
                    })
                }
            }
            // Map: key and value recursive unification
            (ResolvedType::Map(ka, va), ResolvedType::Map(kb, vb)) => {
                self.unify(ka, kb)?;
                self.unify(va, vb)
            }
            // ConstGeneric: structural name equality
            (ResolvedType::ConstGeneric(na), ResolvedType::ConstGeneric(nb)) => {
                if na == nb {
                    Ok(())
                } else {
                    Err(TypeError::Mismatch {
                        expected: expected.to_string(),
                        found: found.to_string(),
                        span: None,
                    })
                }
            }
            // Associated type: structural equality (base, trait, assoc_name, generics)
            (
                ResolvedType::Associated {
                    base: ba,
                    trait_name: tna,
                    assoc_name: ana,
                    generics: ga,
                },
                ResolvedType::Associated {
                    base: bb,
                    trait_name: tnb,
                    assoc_name: anb,
                    generics: gb,
                },
            ) if tna == tnb && ana == anb && ga.len() == gb.len() => {
                self.unify(ba, bb)?;
                for (ta, tb) in ga.iter().zip(gb.iter()) {
                    self.unify(ta, tb)?;
                }
                Ok(())
            }
            // Lifetime: structural name equality
            (ResolvedType::Lifetime(na), ResolvedType::Lifetime(nb)) => {
                if na == nb {
                    Ok(())
                } else {
                    Err(TypeError::Mismatch {
                        expected: expected.to_string(),
                        found: found.to_string(),
                        span: None,
                    })
                }
            }
            // DynTrait: dyn Trait accepts any concrete type that implements the trait
            (ResolvedType::DynTrait { .. }, _) | (_, ResolvedType::DynTrait { .. }) => Ok(()),
            // ImplTrait / HigherKinded were removed in ROADMAP #18.
            // A4-03: default-strict rejection of &T ↔ T implicit deref.
            //
            // Keep unresolved inference-variable glue working; it is not a
            // concrete user-facing implicit deref decision. Set
            // VAIS_REJECT_A4_03=0 only for legacy drift investigation.
            (ResolvedType::Ref(inner), other) | (other, ResolvedType::Ref(inner)) => {
                let inference_glue = Self::contains_var(inner) || Self::contains_var(other);
                if !Self::allow_legacy_a4_03_auto_deref() && !inference_glue {
                    Err(crate::TypeError::Mismatch {
                        expected: expected.to_string(),
                        found: found.to_string(),
                        span: None,
                    })
                } else {
                    self.unify(inner, other)
                }
            }
            (ResolvedType::RefMut(inner), other) | (other, ResolvedType::RefMut(inner)) => {
                let inference_glue = Self::contains_var(inner) || Self::contains_var(other);
                if !Self::allow_legacy_a4_03_auto_deref() && !inference_glue {
                    Err(crate::TypeError::Mismatch {
                        expected: expected.to_string(),
                        found: found.to_string(),
                        span: None,
                    })
                } else {
                    self.unify(inner, other)
                }
            }
            _ => Err(TypeError::Mismatch {
                expected: expected.to_string(),
                found: found.to_string(),
                span: None, // No span available in unify()
            }),
        }
    }

    /// Check if type is an integer type.
    /// Bool is NOT included: bool is a distinct type in Vais's type system.
    #[inline]
    pub(crate) fn is_integer_type(ty: &ResolvedType) -> bool {
        matches!(
            ty,
            ResolvedType::I8
                | ResolvedType::I16
                | ResolvedType::I32
                | ResolvedType::I64
                | ResolvedType::U8
                | ResolvedType::U16
                | ResolvedType::U32
                | ResolvedType::U64
        )
    }

    /// Check if type is a float type
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn is_float_type(ty: &ResolvedType) -> bool {
        matches!(ty, ResolvedType::F32 | ResolvedType::F64)
    }

    /// Check if a type contains any type variables (Var).
    /// Types without Var nodes cannot be affected by apply_substitutions.
    #[inline]
    pub(crate) fn contains_var(ty: &ResolvedType) -> bool {
        match ty {
            ResolvedType::Var(_) => true,
            // Leaf types: no type variables
            ResolvedType::I8
            | ResolvedType::I16
            | ResolvedType::I32
            | ResolvedType::I64
            | ResolvedType::I128
            | ResolvedType::U8
            | ResolvedType::U16
            | ResolvedType::U32
            | ResolvedType::U64
            | ResolvedType::U128
            | ResolvedType::F32
            | ResolvedType::F64
            | ResolvedType::Bool
            | ResolvedType::Str
            | ResolvedType::Unit
            | ResolvedType::Never
            | ResolvedType::Unknown
            | ResolvedType::Generic(_)
            | ResolvedType::Lifetime(_) => false,
            // Wrapper types with one inner
            ResolvedType::Array(inner)
            | ResolvedType::Optional(inner)
            | ResolvedType::Ref(inner)
            | ResolvedType::RefMut(inner)
            | ResolvedType::Slice(inner)
            | ResolvedType::SliceMut(inner)
            | ResolvedType::Pointer(inner)
            | ResolvedType::Range(inner)
            | ResolvedType::Future(inner)
            | ResolvedType::Linear(inner)
            | ResolvedType::Affine(inner) => Self::contains_var(inner),
            ResolvedType::Result(ok, err) | ResolvedType::Map(ok, err) => {
                Self::contains_var(ok) || Self::contains_var(err)
            }
            ResolvedType::Tuple(types) => types.iter().any(Self::contains_var),
            ResolvedType::Fn { params, ret, .. } | ResolvedType::FnPtr { params, ret, .. } => {
                params.iter().any(Self::contains_var) || Self::contains_var(ret)
            }
            ResolvedType::Named { generics, .. } | ResolvedType::DynTrait { generics, .. } => {
                generics.iter().any(Self::contains_var)
            }
            ResolvedType::RefLifetime { inner, .. }
            | ResolvedType::RefMutLifetime { inner, .. } => Self::contains_var(inner),
            ResolvedType::Dependent { base, .. } => Self::contains_var(base),
            ResolvedType::ConstArray { element, .. } | ResolvedType::Vector { element, .. } => {
                Self::contains_var(element)
            }
            ResolvedType::Associated { base, generics, .. } => {
                Self::contains_var(base) || generics.iter().any(Self::contains_var)
            }
            // ConstGeneric and other types without nesting
            _ => false,
        }
    }
}
