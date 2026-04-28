//! Collection and aggregate type checking (arrays, tuples, structs, ranges, etc.)

use crate::types::{self, GenericInstantiation, ResolvedType, TypeError, TypeResult};
use crate::TypeChecker;
use std::collections::HashMap;
use vais_ast::*;

impl TypeChecker {
    /// Check collection expressions
    pub(crate) fn check_collection_expr(
        &mut self,
        expr: &Spanned<Expr>,
    ) -> Option<TypeResult<ResolvedType>> {
        match &expr.node {
            Expr::Binary { op, left, right } => {
                let left_type_raw = match self.check_expr(left) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let right_type_raw = match self.check_expr(right) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };

                // Phase 2.12: auto-deref &T → T for arithmetic/comparison
                // operands. Vec.get(i)/HashMap.get(k) return Option<&T>; after
                // `Some(n) => ...`, `n` is `&T`. Rather than forcing users to
                // write `*n`, strip the outer Ref in binary-op contexts.
                // Mut refs and nested refs both handled.
                fn peel_ref(t: &ResolvedType) -> ResolvedType {
                    match t {
                        ResolvedType::Ref(inner) | ResolvedType::RefMut(inner) => {
                            peel_ref(inner)
                        }
                        _ => t.clone(),
                    }
                }
                let (left_type, right_type) = match op {
                    BinOp::Add
                    | BinOp::Sub
                    | BinOp::Mul
                    | BinOp::Div
                    | BinOp::Mod
                    | BinOp::Lt
                    | BinOp::Lte
                    | BinOp::Gt
                    | BinOp::Gte
                    | BinOp::Eq
                    | BinOp::Neq => (peel_ref(&left_type_raw), peel_ref(&right_type_raw)),
                    _ => (left_type_raw.clone(), right_type_raw.clone()),
                };
                // Keep raw versions available for paths that actually need them
                // (string concat sees through Ref in is_str_like).
                let _ = &left_type_raw;
                let _ = &right_type_raw;

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        // Allow string concatenation with + (Phase 272: include
                        // named Str/String/&str for vaisdb's owned-string flow).
                        let is_str_like = |t: &ResolvedType| -> bool {
                            matches!(t, ResolvedType::Str)
                                || matches!(t, ResolvedType::Named { name, generics }
                                    if (name == "Str" || name == "str" || name == "String")
                                        && generics.is_empty())
                                || matches!(t,
                                    ResolvedType::Ref(inner) | ResolvedType::RefMut(inner)
                                    if matches!(inner.as_ref(), ResolvedType::Str)
                                        || matches!(inner.as_ref(),
                                            ResolvedType::Named { name, generics }
                                            if (name == "Str" || name == "str"
                                                || name == "String") && generics.is_empty()))
                        };
                        if matches!(op, BinOp::Add) && is_str_like(&left_type) {
                            // Permissive: ignore right_type unify failure since
                            // codegen handles the concat via runtime helpers.
                            let _ = self.unify(&left_type, &right_type);
                            return Some(Ok(ResolvedType::Str));
                        }
                        if !left_type.is_numeric() {
                            return Some(Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: left_type.to_string(),
                                span: Some(left.span),
                            }));
                        }
                        if let Err(e) = self.unify(&left_type, &right_type) {
                            return Some(Err(e));
                        }
                        Some(Ok(left_type))
                    }
                    BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte => {
                        // Allow string comparison with <, >, <=, >=
                        if matches!(left_type, ResolvedType::Str) {
                            if let Err(e) = self.unify(&left_type, &right_type) {
                                return Some(Err(e));
                            }
                            return Some(Ok(ResolvedType::Bool));
                        }
                        if !left_type.is_numeric() {
                            return Some(Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: left_type.to_string(),
                                span: Some(left.span),
                            }));
                        }
                        if let Err(e) = self.unify(&left_type, &right_type) {
                            return Some(Err(e));
                        }
                        Some(Ok(ResolvedType::Bool))
                    }
                    BinOp::Eq | BinOp::Neq => {
                        // Phase 258: lenient == / != — bool↔integer compare allowed.
                        // Many vaisdb checks compare a bool field against an i64 result
                        // (e.g. `obj.flag == get_status()` where get_status returns i64).
                        let bool_int_pair = matches!(
                            (&left_type, &right_type),
                            (ResolvedType::Bool, t) | (t, ResolvedType::Bool) if t.is_integer()
                        );
                        if !bool_int_pair {
                            if let Err(e) = self.unify(&left_type, &right_type) {
                                return Some(Err(e));
                            }
                        }
                        Some(Ok(ResolvedType::Bool))
                    }
                    BinOp::And | BinOp::Or => {
                        // Phase 257: lenient && / || — accept Bool or integer truthy.
                        let leniency = |t: &ResolvedType| -> bool {
                            matches!(t, ResolvedType::Bool)
                                || t.is_integer()
                                || matches!(t, ResolvedType::Var(_) | ResolvedType::Unknown)
                        };
                        if !leniency(&left_type) {
                            if let Err(e) = self.unify(&left_type, &ResolvedType::Bool) {
                                return Some(Err(e));
                            }
                        }
                        if !leniency(&right_type) {
                            if let Err(e) = self.unify(&right_type, &ResolvedType::Bool) {
                                return Some(Err(e));
                            }
                        }
                        Some(Ok(ResolvedType::Bool))
                    }
                    BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => {
                        // Allow bool operands for BitAnd (&) and BitOr (|) as logical and/or
                        if matches!(left_type, ResolvedType::Bool)
                            && matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor)
                        {
                            if let Err(e) = self.unify(&left_type, &right_type) {
                                return Some(Err(e));
                            }
                            return Some(Ok(ResolvedType::Bool));
                        }
                        if !left_type.is_integer() {
                            return Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: left_type.to_string(),
                                span: Some(left.span),
                            }));
                        }
                        // Phase 275: lenient bitwise — int op bool treated as
                        // int op (bool as int). Common in vaisdb permission
                        // mask building: `priv_mask | (flag ? BIT : 0)`.
                        if matches!(right_type, ResolvedType::Bool) {
                            return Some(Ok(left_type));
                        }
                        if let Err(e) = self.unify(&left_type, &right_type) {
                            return Some(Err(e));
                        }
                        Some(Ok(left_type))
                    }
                }
            }

            Expr::Unary { op, expr: inner } => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                match op {
                    UnaryOp::Neg => {
                        if !inner_type.is_numeric() {
                            return Some(Err(TypeError::Mismatch {
                                expected: "numeric".to_string(),
                                found: inner_type.to_string(),
                                span: Some(inner.span),
                            }));
                        }
                        Some(Ok(inner_type))
                    }
                    UnaryOp::Not => {
                        // Phase 256: lenient ! — accept Bool or integer (truthy 0/1).
                        // vaisdb stdlib has many i64-returning predicates; the result
                        // is still Bool semantically.
                        if !matches!(inner_type, ResolvedType::Bool)
                            && !inner_type.is_integer()
                            && !matches!(
                                inner_type,
                                ResolvedType::Var(_) | ResolvedType::Unknown
                            )
                        {
                            if let Err(e) = self.unify(&inner_type, &ResolvedType::Bool) {
                                return Some(Err(e));
                            }
                        }
                        Some(Ok(ResolvedType::Bool))
                    }
                    UnaryOp::BitNot => {
                        if !inner_type.is_integer() {
                            return Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: inner_type.to_string(),
                                span: Some(inner.span),
                            }));
                        }
                        Some(Ok(inner_type))
                    }
                }
            }

            Expr::Field { expr: inner, field } => {
                // Phase 6.27b iteration 50: disambiguate `EnumName.Variant`
                // when `EnumName` is ALSO a variant name of another enum in
                // scope. Without this, lookup finds the variant first and
                // resolves inner to the OTHER enum's type, then field access
                // for `Variant` fails on that other enum. Prefer: if the
                // inner is an Ident naming an enum directly, treat it as
                // that enum's variant-access.
                if let Expr::Ident(n) = &inner.node {
                    if let Some(enum_def) = self.enums.get(n) {
                        if enum_def.variants.contains_key(&field.node) {
                            let generics: Vec<ResolvedType> = enum_def
                                .generics
                                .iter()
                                .map(|_| self.fresh_type_var())
                                .collect();
                            return Some(Ok(ResolvedType::Named {
                                name: n.clone(),
                                generics,
                            }));
                        }
                    }
                }

                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };

                // Handle direct Named types, references, and pointers (auto-deref).
                // Phase 259: also auto-unwrap Option<T>/Result<T,E> for field access.
                // vaisdb stdlib often returns Option<T> from .get/get_mut and
                // immediately accesses fields. Strict ownership/borrow checks are
                // enforced separately.
                let type_name = match &inner_type {
                    ResolvedType::Named { name, .. } => Some(name.clone()),
                    ResolvedType::Ref(inner)
                    | ResolvedType::RefMut(inner)
                    | ResolvedType::Pointer(inner) => {
                        if let ResolvedType::Named { name, .. } = inner.as_ref() {
                            Some(name.clone())
                        } else {
                            None
                        }
                    }
                    ResolvedType::Optional(inner) | ResolvedType::Result(inner, _) => {
                        match inner.as_ref() {
                            ResolvedType::Named { name, .. } => Some(name.clone()),
                            ResolvedType::Ref(t)
                            | ResolvedType::RefMut(t)
                            | ResolvedType::Pointer(t) => {
                                if let ResolvedType::Named { name, .. } = t.as_ref() {
                                    Some(name.clone())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                };

                if let Some(name) = type_name.clone() {
                    // Check struct fields
                    if let Some(struct_def) = self.structs.get(&name) {
                        if let Some(field_type) = struct_def.fields.get(&field.node) {
                            return Some(Ok(field_type.clone()));
                        }
                    }
                    // Check enum variant access: EnumType.Variant
                    if let Some(enum_def) = self.enums.get(&name) {
                        if enum_def.variants.contains_key(&field.node) {
                            let generics: Vec<ResolvedType> = enum_def
                                .generics
                                .iter()
                                .map(|_| self.fresh_type_var())
                                .collect();
                            return Some(Ok(ResolvedType::Named {
                                name: name.clone(),
                                generics,
                            }));
                        }
                    }
                    // Check union fields
                    if let Some(union_def) = self.unions.get(&name) {
                        if let Some(field_type) = union_def.fields.get(&field.node) {
                            return Some(Ok(field_type.clone()));
                        }
                    }
                }

                // Get field names for did-you-mean suggestion
                let suggestion = if let Some(ref name) = type_name {
                    if let Some(struct_def) = self.structs.get(name) {
                        types::find_similar_name(
                            &field.node,
                            struct_def.fields.keys().map(|s| s.as_str()),
                        )
                    } else if let Some(union_def) = self.unions.get(name) {
                        types::find_similar_name(
                            &field.node,
                            union_def.fields.keys().map(|s| s.as_str()),
                        )
                    } else {
                        None
                    }
                } else {
                    None
                };

                let display_type_name = type_name.unwrap_or_else(|| inner_type.to_string());
                Some(Err(TypeError::NoSuchField {
                    field: field.node.clone(),
                    type_name: display_type_name,
                    suggestion,
                    span: Some(field.span),
                }))
            }

            Expr::TupleFieldAccess { expr: inner, index } => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                // Phase 280: apply substitutions before matching so that
                // type variables resolved to tuples are correctly handled.
                let inner_type = self.apply_substitutions(&inner_type);
                // Unwrap references and Optional wrappers (auto-deref).
                // Phase 280: also handle Unknown/Var (unresolved inference vars)
                // and Named structs leniently — return Unknown instead of error.
                // This avoids cascading E001 errors when HashMap iteration yields
                // an unresolved item type.
                let tuple_type = match &inner_type {
                    ResolvedType::Tuple(_) => inner_type.clone(),
                    ResolvedType::Ref(t) | ResolvedType::RefMut(t) => {
                        let inner_resolved = self.apply_substitutions(t);
                        match inner_resolved {
                            ResolvedType::Tuple(_) => inner_resolved,
                            // Doubly-wrapped ref: peel one more layer
                            ResolvedType::Ref(ref t2) | ResolvedType::RefMut(ref t2) => {
                                self.apply_substitutions(t2)
                            }
                            // Unresolved inference variable or unknown — lenient
                            ResolvedType::Var(_) | ResolvedType::Unknown => {
                                return Some(Ok(ResolvedType::Unknown));
                            }
                            other => other,
                        }
                    }
                    // Unresolved inference variable — lenient fallback
                    ResolvedType::Var(_) | ResolvedType::Unknown => {
                        return Some(Ok(ResolvedType::Unknown));
                    }
                    // Phase 280: Named struct used with tuple-field syntax (bare, not ref-wrapped).
                    // Lenient: return Unknown to avoid cascading E001. Codegen validates field access.
                    ResolvedType::Named { .. } => {
                        return Some(Ok(ResolvedType::Unknown));
                    }
                    other => {
                        return Some(Err(TypeError::Mismatch {
                            expected: "tuple type".to_string(),
                            found: other.to_string(),
                            span: Some(inner.span),
                        }));
                    }
                };
                match tuple_type {
                    ResolvedType::Tuple(ref fields) => {
                        if *index >= fields.len() {
                            return Some(Err(TypeError::Mismatch {
                                expected: format!(
                                    "tuple index in range 0..{} for {}-tuple",
                                    fields.len(),
                                    fields.len()
                                ),
                                found: format!("index {}", index),
                                span: Some(expr.span),
                            }));
                        }
                        Some(Ok(fields[*index].clone()))
                    }
                    // Phase 280: unresolved type or struct — lenient fallback
                    ResolvedType::Var(_) | ResolvedType::Unknown => {
                        Some(Ok(ResolvedType::Unknown))
                    }
                    // Named struct used with tuple-field syntax (e.g., struct.0):
                    // lenient — return Unknown to avoid cascading errors in vaisdb
                    // code that uses this pattern. The actual field access validation
                    // is enforced at codegen level.
                    ResolvedType::Named { .. } => {
                        Some(Ok(ResolvedType::Unknown))
                    }
                    other => Some(Err(TypeError::Mismatch {
                        expected: "tuple type".to_string(),
                        found: other.to_string(),
                        span: Some(inner.span),
                    })),
                }
            }

            Expr::Index { expr: inner, index } => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let index_type = match self.check_expr(index) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };

                // Check if this is a slice operation (index is a Range)
                let is_slice = matches!(index.node, Expr::Range { .. });

                match inner_type {
                    ResolvedType::Array(elem_type) => {
                        if is_slice {
                            // Slice returns a pointer to array elements
                            Some(Ok(ResolvedType::Pointer(elem_type)))
                        } else if !index_type.is_integer() {
                            Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            }))
                        } else {
                            Some(Ok(*elem_type))
                        }
                    }
                    ResolvedType::Map(key_type, value_type) => {
                        if let Err(e) = self.unify(&key_type, &index_type) {
                            return Some(Err(e));
                        }
                        Some(Ok(*value_type))
                    }
                    // Pointers can be indexed like arrays
                    ResolvedType::Pointer(elem_type) => {
                        if is_slice {
                            // Slice of pointer returns a pointer
                            Some(Ok(ResolvedType::Pointer(elem_type)))
                        } else if !index_type.is_integer() {
                            Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            }))
                        } else {
                            Some(Ok(*elem_type))
                        }
                    }
                    ResolvedType::Slice(elem_type) => {
                        if is_slice {
                            Some(Ok(ResolvedType::Slice(elem_type)))
                        } else if !index_type.is_integer() {
                            Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            }))
                        } else {
                            Some(Ok(*elem_type))
                        }
                    }
                    ResolvedType::SliceMut(elem_type) => {
                        if is_slice {
                            Some(Ok(ResolvedType::SliceMut(elem_type)))
                        } else if !index_type.is_integer() {
                            Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            }))
                        } else {
                            Some(Ok(*elem_type))
                        }
                    }
                    // Fixed-size array `[T; N]` (ConstArray) is indexable
                    // — decays to T at the index expression. Bounds-checking
                    // is codegen/runtime territory, not the type checker.
                    ResolvedType::ConstArray { element, size: _ } => {
                        if is_slice {
                            Some(Ok(ResolvedType::Slice(element)))
                        } else if !index_type.is_integer() {
                            Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            }))
                        } else {
                            Some(Ok(*element))
                        }
                    }
                    // Vec<T> is indexable — vec[idx] returns T
                    ResolvedType::Named {
                        ref name,
                        ref generics,
                    } if name == "Vec" && !generics.is_empty() => {
                        if is_slice {
                            // Phase Ω P1.7 (iter 134): Vec<T>[range] → Slice<T>
                            // (was Pointer<T> — pre-fix, downstream `.to_vec()`
                            // and other slice methods could not dispatch
                            // because they only match `Slice`/`SliceMut`).
                            // The codegen treats Pointer/Slice equivalently at
                            // the IR level, so this is TC-only refinement.
                            Some(Ok(ResolvedType::Slice(Box::new(
                                self.apply_substitutions(&generics[0]),
                            ))))
                        } else if !index_type.is_integer() {
                            Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            }))
                        } else {
                            Some(Ok(self.apply_substitutions(&generics[0])))
                        }
                    }
                    // Ref or RefMut to Vec<T> is also indexable
                    ResolvedType::Ref(ref inner) | ResolvedType::RefMut(ref inner) => {
                        // Phase 266: peel one extra Ref layer (&&[T] → &[T]).
                        let inner_peeled: &ResolvedType = match inner.as_ref() {
                            ResolvedType::Ref(t) | ResolvedType::RefMut(t) => t.as_ref(),
                            _ => inner.as_ref(),
                        };
                        if let ResolvedType::Named {
                            ref name,
                            ref generics,
                        } = inner_peeled
                        {
                            if name == "Vec" && !generics.is_empty() {
                                // Phase 262: &Vec<T>[range] → Slice<T>.
                                if is_slice {
                                    return Some(Ok(ResolvedType::Slice(Box::new(
                                        self.apply_substitutions(&generics[0]),
                                    ))));
                                }
                                if !index_type.is_integer() {
                                    return Some(Err(TypeError::Mismatch {
                                        expected: "integer".to_string(),
                                        found: index_type.to_string(),
                                        span: Some(index.span),
                                    }));
                                }
                                return Some(Ok(self.apply_substitutions(&generics[0])));
                            }
                        }
                        // Phase 266: &&[T] / &[T] indexing returns T.
                        if let ResolvedType::Slice(elem) | ResolvedType::SliceMut(elem) =
                            inner_peeled
                        {
                            if is_slice {
                                return Some(Ok(ResolvedType::Slice(elem.clone())));
                            }
                            if !index_type.is_integer() {
                                return Some(Err(TypeError::Mismatch {
                                    expected: "integer".to_string(),
                                    found: index_type.to_string(),
                                    span: Some(index.span),
                                }));
                            }
                            return Some(Ok((**elem).clone()));
                        }
                        // Phase 252: &str / &Str indexing returns I64 (byte).
                        if matches!(**inner, ResolvedType::Str)
                            || matches!(&**inner, ResolvedType::Named { name, generics }
                                if (name == "Str" || name == "str") && generics.is_empty())
                        {
                            if !index_type.is_integer() {
                                return Some(Err(TypeError::Mismatch {
                                    expected: "integer".to_string(),
                                    found: index_type.to_string(),
                                    span: Some(index.span),
                                }));
                            }
                            return Some(Ok(ResolvedType::I64));
                        }
                        Some(Err(TypeError::Mismatch {
                            expected: "indexable type".to_string(),
                            found: inner_type.to_string(),
                            span: Some(expr.span),
                        }))
                    }
                    // Phase 252: str / Str (primitive or alias) indexing.
                    ResolvedType::Str => {
                        if !index_type.is_integer() {
                            return Some(Err(TypeError::Mismatch {
                                expected: "integer".to_string(),
                                found: index_type.to_string(),
                                span: Some(index.span),
                            }));
                        }
                        Some(Ok(ResolvedType::I64))
                    }
                    // Phase 6.27c.2: MutexGuard<T>/RwLockReadGuard<T>/RwLockWriteGuard<T>
                    // forward indexing to inner T — parallel to the Phase 338
                    // method-call forwarding in calls.rs. Needed for vaisdb
                    // concurrency code: `queue[0]` where queue is
                    // MutexGuard<Vec<u64>>.
                    ResolvedType::Named {
                        ref name,
                        ref generics,
                    } if matches!(name.as_str(), "MutexGuard" | "RwLockReadGuard" | "RwLockWriteGuard")
                        && !generics.is_empty() =>
                    {
                        let inner = generics[0].clone();
                        // Delegate to Vec<T> / ConstArray / Slice / Str branches
                        // by pattern-matching on the inner type.
                        match inner {
                            ResolvedType::Named {
                                ref name,
                                ref generics,
                            } if name == "Vec" && !generics.is_empty() => {
                                if !index_type.is_integer() {
                                    return Some(Err(TypeError::Mismatch {
                                        expected: "integer".to_string(),
                                        found: index_type.to_string(),
                                        span: Some(index.span),
                                    }));
                                }
                                Some(Ok(self.apply_substitutions(&generics[0])))
                            }
                            ResolvedType::Str => {
                                if !index_type.is_integer() {
                                    return Some(Err(TypeError::Mismatch {
                                        expected: "integer".to_string(),
                                        found: index_type.to_string(),
                                        span: Some(index.span),
                                    }));
                                }
                                Some(Ok(ResolvedType::I64))
                            }
                            ResolvedType::Array(elem) | ResolvedType::Slice(elem) => {
                                if !index_type.is_integer() {
                                    return Some(Err(TypeError::Mismatch {
                                        expected: "integer".to_string(),
                                        found: index_type.to_string(),
                                        span: Some(index.span),
                                    }));
                                }
                                Some(Ok(*elem))
                            }
                            _ => Some(Err(TypeError::Mismatch {
                                expected: "indexable type".to_string(),
                                found: inner_type.to_string(),
                                span: Some(expr.span),
                            })),
                        }
                    }
                    _ => Some(Err(TypeError::Mismatch {
                        expected: "indexable type".to_string(),
                        found: inner_type.to_string(),
                        span: Some(expr.span),
                    })),
                }
            }

            Expr::Array(exprs) => {
                if exprs.is_empty() {
                    let var = self.fresh_type_var();
                    // Array literals decay to pointers in Vais
                    return Some(Ok(ResolvedType::Pointer(Box::new(var))));
                }

                // Helper: get element type from an array element (handles Spread)
                let get_elem_type =
                    |checker: &mut Self, e: &Spanned<Expr>| -> TypeResult<ResolvedType> {
                        if let Expr::Spread(inner) = &e.node {
                            let inner_type = checker.check_expr(inner)?;
                            // Spread must be on a pointer/array type
                            match inner_type {
                                ResolvedType::Pointer(elem) => Ok(*elem),
                                ResolvedType::Array(elem) => Ok(*elem),
                                _ => Ok(inner_type),
                            }
                        } else {
                            checker.check_expr(e)
                        }
                    };

                let first_type = match get_elem_type(self, &exprs[0]) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                for expr in &exprs[1..] {
                    let t = match get_elem_type(self, expr) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    if let Err(e) = self.unify(&first_type, &t) {
                        return Some(Err(e));
                    }
                }

                // Array literals produce pointers to first element
                Some(Ok(ResolvedType::Pointer(Box::new(first_type))))
            }

            Expr::Tuple(exprs) => {
                let mut types = Vec::new();
                for e in exprs {
                    match self.check_expr(e) {
                        Ok(t) => types.push(t),
                        Err(e) => return Some(Err(e)),
                    }
                }
                Some(Ok(ResolvedType::Tuple(types)))
            }

            Expr::MapLit(pairs) => {
                if pairs.is_empty() {
                    let k = self.fresh_type_var();
                    let v = self.fresh_type_var();
                    return Some(Ok(ResolvedType::Map(Box::new(k), Box::new(v))));
                }
                let first_key_type = match self.check_expr(&pairs[0].0) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let first_val_type = match self.check_expr(&pairs[0].1) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                for (k, v) in &pairs[1..] {
                    let kt = match self.check_expr(k) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    let vt = match self.check_expr(v) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    if let Err(e) = self.unify(&first_key_type, &kt) {
                        return Some(Err(e));
                    }
                    if let Err(e) = self.unify(&first_val_type, &vt) {
                        return Some(Err(e));
                    }
                }
                Some(Ok(ResolvedType::Map(
                    Box::new(first_key_type),
                    Box::new(first_val_type),
                )))
            }

            Expr::StructLit {
                name,
                fields,
                enum_name,
            } => {
                // Check for enum struct variant first (e.g., Shape.Circle { radius: 5.0 })
                if let Some(ref ename) = enum_name {
                    if let Some(enum_def) = self.enums.get(ename).cloned() {
                        let variant_name = &name.node;
                        if let Some(variant_fields) = enum_def.variants.get(variant_name) {
                            // It's a valid enum variant — check the fields
                            if let crate::types::VariantFieldTypes::Struct(expected_fields) =
                                variant_fields
                            {
                                // Check each provided field (Phase 6.27c.3:
                                // push enum hint for the field type so bare
                                // variants resolve to the right enum).
                                for (field_name, value) in fields {
                                    let expected_ty_opt = expected_fields
                                        .get(&field_name.node)
                                        .cloned();
                                    let hint =
                                        expected_ty_opt.as_ref().and_then(Self::enum_name_hint_from);
                                    if let Some(ref h) = hint {
                                        self.push_enum_hint(h.clone());
                                    }
                                    let value_type_res = self.check_expr(value);
                                    if hint.is_some() {
                                        self.pop_enum_hint();
                                    }
                                    let value_type = match value_type_res {
                                        Ok(t) => t,
                                        Err(e) => return Some(Err(e)),
                                    };
                                    if let Some(expected_type) = expected_ty_opt {
                                        if let Err(e) = self.unify(&expected_type, &value_type) {
                                            return Some(Err(e));
                                        }
                                    }
                                }
                            }
                            // Return the enum type
                            return Some(Ok(ResolvedType::Named {
                                name: ename.clone(),
                                generics: enum_def
                                    .generics
                                    .iter()
                                    .map(|_| self.fresh_type_var())
                                    .collect(),
                            }));
                        }
                    }
                }

                // First check for struct
                if let Some(struct_def) = self.structs.get(&name.node).cloned() {
                    // Create fresh type variables for generic parameters
                    let generic_substitutions: HashMap<String, ResolvedType> = struct_def
                        .generics
                        .iter()
                        .map(|param| (param.clone(), self.fresh_type_var()))
                        .collect();

                    // Check each field and unify with expected type
                    for (field_name, value) in fields {
                        // Phase 6.27c.3: if we can see the expected field
                        // type, push its enum name so bare-variant idents
                        // inside the value resolve to that enum first.
                        let pre_subst = struct_def.fields.get(&field_name.node).cloned();
                        let expected_ty_subst = pre_subst.as_ref().map(|et| {
                            self.substitute_generics(et, &generic_substitutions)
                        });
                        let hint = expected_ty_subst
                            .as_ref()
                            .and_then(Self::enum_name_hint_from);
                        if let Some(ref h) = hint {
                            self.push_enum_hint(h.clone());
                        }
                        // Phase 17.H4.15: push the full expected field type
                        // so zero-arg generic static methods like
                        // `Vec.new()` can unify their fresh type vars with
                        // the field's concrete type args before stamping.
                        let pushed_expected = expected_ty_subst.is_some();
                        if let Some(ref et) = expected_ty_subst {
                            self.push_expected_type(et.clone());
                        }
                        let value_type_res = self.check_expr(value);
                        if pushed_expected {
                            self.pop_expected_type();
                        }
                        if hint.is_some() {
                            self.pop_enum_hint();
                        }
                        let value_type = match value_type_res {
                            Ok(t) => t,
                            Err(e) => return Some(Err(e)),
                        };
                        if let Some(expected_type) = expected_ty_subst {
                            if let Err(e) = self.unify(&expected_type, &value_type) {
                                return Some(Err(e));
                            }
                        } else if pre_subst.is_none() {
                            let suggestion = types::find_similar_name(
                                &field_name.node,
                                struct_def.fields.keys().map(|s| s.as_str()),
                            );
                            return Some(Err(TypeError::UndefinedVar {
                                name: field_name.node.clone(),
                                span: Some(field_name.span),
                                suggestion,
                            }));
                        }
                    }

                    // Apply substitutions to infer concrete generic types
                    let inferred_generics: Vec<_> = struct_def
                        .generics
                        .iter()
                        .map(|param| {
                            let ty = generic_substitutions
                                .get(param)
                                .unwrap_or(&ResolvedType::Unknown);
                            self.apply_substitutions(ty)
                        })
                        .collect();

                    // Record generic struct instantiation if the struct has generic parameters
                    if !struct_def.generics.is_empty() {
                        // Only record if all type arguments are concrete (not type variables)
                        let all_concrete = inferred_generics
                            .iter()
                            .all(|t| !matches!(t, ResolvedType::Var(_)));
                        if all_concrete {
                            let inst = GenericInstantiation::struct_type(
                                &name.node,
                                inferred_generics.clone(),
                            );
                            self.add_instantiation(inst);
                        }
                    }

                    Some(Ok(ResolvedType::Named {
                        name: name.node.clone(),
                        generics: inferred_generics,
                    }))
                // Then check for union (uses same syntax: `UnionName { field: value }`)
                } else if let Some(union_def) = self.unions.get(&name.node).cloned() {
                    // Create fresh type variables for generic parameters
                    let generic_substitutions: HashMap<String, ResolvedType> = union_def
                        .generics
                        .iter()
                        .map(|param| (param.clone(), self.fresh_type_var()))
                        .collect();

                    // Union literal should have exactly one field
                    if fields.len() != 1 {
                        return Some(Err(TypeError::Mismatch {
                            expected: "exactly one field for union initialization".to_string(),
                            found: format!("{} fields", fields.len()),
                            span: Some(expr.span),
                        }));
                    }

                    // Check the field
                    let (field_name, value) = &fields[0];
                    let value_type = match self.check_expr(value) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    if let Some(expected_type) = union_def.fields.get(&field_name.node).cloned() {
                        let expected_type =
                            self.substitute_generics(&expected_type, &generic_substitutions);
                        if let Err(e) = self.unify(&expected_type, &value_type) {
                            return Some(Err(e));
                        }
                    } else {
                        let suggestion = types::find_similar_name(
                            &field_name.node,
                            union_def.fields.keys().map(|s| s.as_str()),
                        );
                        return Some(Err(TypeError::UndefinedVar {
                            name: field_name.node.clone(),
                            span: Some(field_name.span),
                            suggestion,
                        }));
                    }

                    // Apply substitutions to infer concrete generic types
                    let inferred_generics: Vec<_> = union_def
                        .generics
                        .iter()
                        .map(|param| {
                            let ty = generic_substitutions
                                .get(param)
                                .unwrap_or(&ResolvedType::Unknown);
                            self.apply_substitutions(ty)
                        })
                        .collect();

                    Some(Ok(ResolvedType::Named {
                        name: name.node.clone(),
                        generics: inferred_generics,
                    }))
                } else {
                    // Phase 6.27b iteration 52: fallback — name might be a
                    // Struct variant of some enum (short-form). Find the
                    // first enum whose Struct variant has this name AND
                    // whose field set covers the literal's provided fields
                    // (the covers-check avoids picking a same-named variant
                    // in a different enum — see iter-35 disambiguation).
                    let provided_field_names: std::collections::HashSet<String> =
                        fields.iter().map(|(fn_, _)| fn_.node.clone()).collect();
                    // Snapshot enum entries to avoid borrow issues during
                    // subsequent check_expr/unify calls.
                    let mut enum_snapshots: Vec<(
                        String,
                        Vec<String>,
                        std::collections::HashMap<String, ResolvedType>,
                    )> = Vec::new();
                    {
                        let mut enum_entries: Vec<_> = self.enums.iter().collect();
                        enum_entries.sort_by(|(a, _), (b, _)| {
                            let a_builtin = matches!(a.as_str(), "Option" | "Result");
                            let b_builtin = matches!(b.as_str(), "Option" | "Result");
                            a_builtin.cmp(&b_builtin).then_with(|| a.cmp(b))
                        });
                        for (enum_name_str, enum_def) in enum_entries {
                            if let Some(variant_fields) = enum_def.variants.get(&name.node) {
                                if let crate::types::VariantFieldTypes::Struct(expected_fields) =
                                    variant_fields
                                {
                                    let covers_all = provided_field_names.iter().all(|pfn| {
                                        expected_fields.contains_key(pfn.as_str())
                                    });
                                    if covers_all {
                                        enum_snapshots.push((
                                            enum_name_str.clone(),
                                            enum_def.generics.clone(),
                                            expected_fields.clone(),
                                        ));
                                        break; // prefer first sorted match
                                    }
                                }
                            }
                        }
                    }
                    if let Some((enum_name_str, enum_generics, expected_fields)) =
                        enum_snapshots.into_iter().next()
                    {
                        // Type-check each provided field against expected
                        // (Phase 6.27c.3: propagate enum hint like above).
                        for (field_name, value) in fields {
                            let expected_ty_opt = expected_fields
                                .get(&field_name.node)
                                .cloned();
                            let hint = expected_ty_opt
                                .as_ref()
                                .and_then(Self::enum_name_hint_from);
                            if let Some(ref h) = hint {
                                self.push_enum_hint(h.clone());
                            }
                            let value_type_res = self.check_expr(value);
                            if hint.is_some() {
                                self.pop_enum_hint();
                            }
                            let value_type = match value_type_res {
                                Ok(t) => t,
                                Err(e) => return Some(Err(e)),
                            };
                            if let Some(expected_type) = expected_ty_opt {
                                if let Err(e) = self.unify(&expected_type, &value_type) {
                                    return Some(Err(e));
                                }
                            }
                        }
                        let generics: Vec<ResolvedType> = enum_generics
                            .iter()
                            .map(|_| self.fresh_type_var())
                            .collect();
                        return Some(Ok(ResolvedType::Named {
                            name: enum_name_str,
                            generics,
                        }));
                    }

                    // Get all type names for suggestion
                    let mut type_candidates: Vec<&str> = Vec::new();
                    type_candidates.extend(self.structs.keys().map(|s| s.as_str()));
                    type_candidates.extend(self.enums.keys().map(|s| s.as_str()));
                    type_candidates.extend(self.unions.keys().map(|s| s.as_str()));
                    type_candidates.extend(self.type_aliases.keys().map(|s| s.as_str()));

                    let suggestion =
                        types::find_similar_name(&name.node, type_candidates.into_iter());
                    Some(Err(TypeError::UndefinedType {
                        name: name.node.clone(),
                        span: Some(name.span),
                        suggestion,
                    }))
                }
            }

            Expr::Range {
                start,
                end,
                inclusive: _,
            } => {
                // Infer the element type from start or end expressions
                let elem_type = if let Some(start_expr) = start {
                    let start_type = match self.check_expr(start_expr) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    // Ensure start is a numeric type (integer)
                    if !start_type.is_integer() {
                        return Some(Err(TypeError::Mismatch {
                            expected: "integer type".to_string(),
                            found: start_type.to_string(),
                            span: Some(start_expr.span),
                        }));
                    }

                    // If end is present, unify the types
                    if let Some(end_expr) = end {
                        let end_type = match self.check_expr(end_expr) {
                            Ok(t) => t,
                            Err(e) => return Some(Err(e)),
                        };
                        if !end_type.is_integer() {
                            return Some(Err(TypeError::Mismatch {
                                expected: "integer type".to_string(),
                                found: end_type.to_string(),
                                span: Some(end_expr.span),
                            }));
                        }
                        if let Err(e) = self.unify(&start_type, &end_type) {
                            return Some(Err(e));
                        }
                    }

                    start_type
                } else if let Some(end_expr) = end {
                    // Only end is present (e.g., ..10)
                    let end_type = match self.check_expr(end_expr) {
                        Ok(t) => t,
                        Err(e) => return Some(Err(e)),
                    };
                    if !end_type.is_integer() {
                        return Some(Err(TypeError::Mismatch {
                            expected: "integer type".to_string(),
                            found: end_type.to_string(),
                            span: Some(end_expr.span),
                        }));
                    }
                    end_type
                } else {
                    // Neither start nor end (e.g., ..) - default to i64
                    ResolvedType::I64
                };

                Some(Ok(ResolvedType::Range(Box::new(elem_type))))
            }

            _ => None,
        }
    }
}
