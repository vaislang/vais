//! Async, await, spawn, yield, lazy, force expression checking

use crate::types::{ResolvedType, TypeError, TypeResult};
use crate::TypeChecker;
use vais_ast::*;

impl TypeChecker {
    #[inline]
    pub(crate) fn check_async_expr(
        &mut self,
        expr: &Spanned<Expr>,
    ) -> Option<TypeResult<ResolvedType>> {
        match &expr.node {
            Expr::Await(inner) => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };

                // Verify that the inner expression is a Future type
                if let ResolvedType::Future(output_type) = inner_type {
                    // Extract and return the inner type from Future<T>
                    Some(Ok(*output_type))
                } else {
                    Some(Err(TypeError::Mismatch {
                        expected: "Future<T>".to_string(),
                        found: inner_type.to_string(),
                        span: Some(inner.span),
                    }))
                }
            }

            Expr::Yield(inner) => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                // Yield suspends the generator and returns the yielded value to the caller.
                // The yield expression evaluates to the type of the yielded value.
                Some(Ok(inner_type))
            }

            _ => None,
        }
    }
}
