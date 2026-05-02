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
                match inner_type {
                    ResolvedType::Ref(t) | ResolvedType::RefMut(t) | ResolvedType::Pointer(t) => {
                        Some(Ok(*t))
                    }
                    ResolvedType::Named { name, generics }
                        if name == "Box" && generics.len() == 1 =>
                    {
                        Some(Ok(generics[0].clone()))
                    }
                    // Phase 253: lenient deref. vaisdb often dereferences
                    // Option<T> or generic ?-types directly. Treat Optional
                    // and Result as identity (return inner type), and
                    // unbound Var as identity too. Strict ownership/borrow
                    // checking is enforced separately.
                    ResolvedType::Optional(t) => Some(Ok(*t)),
                    ResolvedType::Result(ok, _) => Some(Ok(*ok)),
                    ResolvedType::Var(_) | ResolvedType::Unknown => Some(Ok(inner_type)),
                    _ => Some(Ok(inner_type.clone())),
                }
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
