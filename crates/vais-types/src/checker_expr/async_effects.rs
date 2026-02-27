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

            Expr::Spawn(inner) => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                // Spawn semantics:
                // - `spawn async_fn()` where inner is Future<T>: returns Future<T> as-is
                //   (the async function already produces a Future; spawn schedules it)
                // - `spawn expr` where inner is non-Future T: wraps in Future<T>
                //   (creates an immediately-completed task; useful for lifting sync values
                //   into async context, e.g., `spawn 42` → Future<i64>)
                // Note: Current runtime uses synchronous fallback (no green threads/coroutines).
                // The Future<T> wrapper preserves type-level async semantics for future
                // runtime upgrades (e.g., work-stealing scheduler, coroutine state machines).
                match inner_type {
                    ResolvedType::Future(_) => Some(Ok(inner_type)),
                    other => Some(Ok(ResolvedType::Future(Box::new(other)))),
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

            Expr::Lazy(inner) => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                // lazy expr creates a Lazy<T> thunk
                Some(Ok(ResolvedType::Lazy(Box::new(inner_type))))
            }

            Expr::Force(inner) => {
                let inner_type = match self.check_expr(inner) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                // force expr evaluates a Lazy<T> and returns T
                match inner_type {
                    ResolvedType::Lazy(t) => Some(Ok(*t)),
                    _ => {
                        // If not a Lazy type, force is a no-op (identity)
                        Some(Ok(inner_type))
                    }
                }
            }

            _ => None,
        }
    }
}
