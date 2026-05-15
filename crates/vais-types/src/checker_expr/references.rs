//! Reference and pointer type checking

use crate::types::{ResolvedType, TypeResult};
use crate::TypeChecker;
use vais_ast::*;

impl TypeChecker {
    #[inline]
    pub(crate) fn check_reference_expr(
        &mut self,
        expr: &Spanned<Expr>,
    ) -> Option<TypeResult<ResolvedType>> {
        match &expr.node {
            Expr::Ref(inner) => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                // Special case: &[...] (reference to array literal) should be a slice
                // Array literals have type Pointer(T), so &[...] becomes Ref(Pointer(T))
                // We convert this to Slice(T) to match the slice type semantics
                match &inner_type {
                    ResolvedType::Pointer(elem_ty) if matches!(inner.node, Expr::Array(_)) => {
                        Some(Ok(ResolvedType::Slice(elem_ty.clone())))
                    }
                    _ => Some(Ok(ResolvedType::Ref(Box::new(inner_type)))),
                }
            }

            Expr::Deref(inner) => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                Some(Ok(deref_type(inner_type)))
            }

            Expr::Spread(inner) => {
                // Spread is valid inside array literals; standalone spread just checks inner
                match self.check_expr(inner) {
                    Ok(t) => Some(Ok(t)),
                    Err(e) => Some(Err(e)),
                }
            }

            _ => None,
        }
    }
}

fn deref_type(ty: ResolvedType) -> ResolvedType {
    match ty {
        ResolvedType::Ref(t) | ResolvedType::RefMut(t) | ResolvedType::Pointer(t) => *t,
        ResolvedType::Named { name, generics } if name == "Box" && generics.len() == 1 => {
            generics[0].clone()
        }
        // Phase 253: lenient deref. vaisdb often dereferences Option<T> or
        // Result<T,E> directly. If the wrapped success type is itself a
        // reference, `*opt_ref` should expose the pointee, not leave `&T`.
        ResolvedType::Optional(t) => deref_type(*t),
        ResolvedType::Result(ok, _) => deref_type(*ok),
        other @ (ResolvedType::Var(_) | ResolvedType::Unknown) => other,
        other => other,
    }
}
