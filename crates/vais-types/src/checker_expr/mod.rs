//! Expression and statement type checking.

mod async_effects;
mod calls;
mod collections;
mod control_flow;
mod literals;
mod references;
mod special;
mod stmts;

use crate::types::{ResolvedType, TypeResult};
use crate::TypeChecker;
use vais_ast::*;

impl TypeChecker {
    /// Check an expression - main dispatcher
    pub(crate) fn check_expr(&mut self, expr: &Spanned<Expr>) -> TypeResult<ResolvedType> {
        // Try each category of expression checkers in order

        // 1. Literals and identifiers
        if let Some(result) = self.check_literal_or_ident(expr) {
            return result;
        }

        // 2. Self call
        if matches!(&expr.node, Expr::SelfCall) {
            return self.check_self_call(expr.span);
        }

        // 3. Calls (Call, MethodCall, StaticMethodCall)
        if let Expr::Call { func, args } = &expr.node {
            return self.check_call_expr(func, args, expr.span);
        }
        if let Expr::MethodCall {
            receiver,
            method,
            args,
        } = &expr.node
        {
            return self.check_method_call(receiver, method, args, expr.span);
        }
        if let Expr::StaticMethodCall {
            type_name,
            method,
            args,
        } = &expr.node
        {
            return self.check_static_method_call(type_name, method, args, expr.span);
        }

        // 4. Control flow
        if let Some(result) = self.check_control_flow(expr) {
            return result;
        }

        // 5. Collections (includes binary/unary ops, field, index, array, tuple, map, struct, range)
        if let Some(result) = self.check_collection_expr(expr) {
            return result;
        }

        // 6. References
        if let Some(result) = self.check_reference_expr(expr) {
            return result;
        }

        // 7. Async effects
        if let Some(result) = self.check_async_expr(expr) {
            return result;
        }

        // 8. Special constructs
        if let Some(result) = self.check_special_expr(expr) {
            return result;
        }

        // If we reach here, we've missed an expression type
        // This shouldn't happen - it's a compiler bug
        Err(crate::types::TypeError::InternalError {
            message: format!("unhandled expression type in check_expr: {:?}", expr.node),
            span: Some(expr.span),
        })
    }
}
