//! Reference and pointer type checking

use vais_ast::*;
use crate::TypeChecker;
use crate::types::{ResolvedType, TypeError, TypeResult};

impl TypeChecker {
    pub(crate) fn check_reference_expr(&mut self, expr: &Spanned<Expr>) -> Option<TypeResult<ResolvedType>> {
        match &expr.node {
            Expr::Ref(inner) => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                Some(Ok(ResolvedType::Ref(Box::new(inner_type))))
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
                    _ => Some(Err(TypeError::Mismatch {
                        expected: "reference or pointer".to_string(),
                        found: inner_type.to_string(),
                        span: Some(inner.span),
                    })),
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
