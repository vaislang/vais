//! Type unification logic for the Vais type system.
//!
//! Contains the core unification algorithm (occurs-check, unify),
//! integer type checking, and type variable detection.

use crate::types::{ResolvedType, TypeError, TypeResult};
use crate::TypeChecker;

impl TypeChecker {
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
            | ResolvedType::Affine(inner)
            | ResolvedType::Lazy(inner) => Self::occurs_in(id, inner),
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
            (ResolvedType::Ref(a), ResolvedType::Ref(b)) => self.unify(a, b),
            (ResolvedType::RefMut(a), ResolvedType::RefMut(b)) => self.unify(a, b),
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
            // bool↔integer, int↔float, str↔i64 coercion is FORBIDDEN (Phase 158).
            (a, b) if Self::is_integer_type(a) && Self::is_integer_type(b) => Ok(()),
            // Allow f32 ↔ f64 unification (float literal inference).
            // Float literals like `0.0` default to f64, but should adapt to the expected
            // type when the context is f32 (same as Rust's float literal inference).
            // f32→f64 is lossless widening; f64→f32 narrowing is intentional when the
            // context expects f32. This is distinct from the forbidden int↔float coercion.
            (ResolvedType::F32, ResolvedType::F64)
            | (ResolvedType::F64, ResolvedType::F32) => Ok(()),
            // Allow unit () ↔ i64 (void context: i64 return in void function)
            (ResolvedType::Unit, ResolvedType::I64) | (ResolvedType::I64, ResolvedType::Unit) => {
                Ok(())
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
                    Ok(()) // Permissive: allow Vec ↔ &T
                }
            }
            // Pointer <-> i64 implicit unification.
            // Vais represents all pointers as i64 at the IR level (no opaque pointer distinction).
            // This allows builtins like vec_new() -> i64 and malloc() -> i64 to unify with *T
            // parameters, and swap(ptr, i, j) to accept either pointer or i64 arguments.
            // Scope: unification only — does not enable arbitrary pointer arithmetic in user code.
            (ResolvedType::Pointer(_), ResolvedType::I64)
            | (ResolvedType::I64, ResolvedType::Pointer(_)) => Ok(()),
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
            (ResolvedType::RefLifetime { inner, .. }, ResolvedType::Ref(other))
            | (ResolvedType::Ref(other), ResolvedType::RefLifetime { inner, .. }) => {
                self.unify(inner, other)
            }
            (ResolvedType::RefMutLifetime { inner, .. }, ResolvedType::RefMut(other))
            | (ResolvedType::RefMut(other), ResolvedType::RefMutLifetime { inner, .. }) => {
                self.unify(inner, other)
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
            // Lazy type unification
            (ResolvedType::Lazy(a), ResolvedType::Lazy(b)) => self.unify(a, b),
            // DynTrait: dyn Trait accepts any concrete type that implements the trait
            (ResolvedType::DynTrait { .. }, _) | (_, ResolvedType::DynTrait { .. }) => Ok(()),
            // ImplTrait: unification accepts any concrete type.
            // Bound checking happens at the TypeChecker level (type_implements_trait),
            // not in the inference engine, since TypeInference has no trait impl data.
            // This is consistent with DynTrait handling above.
            (ResolvedType::ImplTrait { .. }, _) | (_, ResolvedType::ImplTrait { .. }) => Ok(()),
            // HigherKinded: type constructor parameters unify with any type.
            // At monomorphization time, F<_> gets replaced with a concrete type constructor.
            // SAFETY: Trait bounds on HKT params are deferred to monomorphization.
            // The TC validates bounds when concrete types are substituted, not during unification.
            // This matches ImplTrait/DynTrait patterns above.
            (ResolvedType::HigherKinded { .. }, _) | (_, ResolvedType::HigherKinded { .. }) => {
                Ok(())
            }
            // Auto-deref: &T unifies with T (implicit dereference)
            (ResolvedType::Ref(inner), other) | (other, ResolvedType::Ref(inner)) => {
                self.unify(inner, other)
            }
            (ResolvedType::RefMut(inner), other) | (other, ResolvedType::RefMut(inner)) => {
                self.unify(inner, other)
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
            | ResolvedType::HigherKinded { .. }
            | ResolvedType::Lifetime(_)
            | ResolvedType::ImplTrait { .. } => false,
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
            | ResolvedType::Affine(inner)
            | ResolvedType::Lazy(inner) => Self::contains_var(inner),
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
